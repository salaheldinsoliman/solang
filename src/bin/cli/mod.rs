// SPDX-License-Identifier: Apache-2.0

use clap::{
    builder::ValueParser, parser::ValueSource, value_parser, ArgAction, ArgMatches, Args, Id,
    Parser, Subcommand,
};
use clap_complete::Shell;

use itertools::Itertools;
use semver::Version;
use serde::Deserialize;
use solang::{
    file_resolver::FileResolver,
    Target,
};
use std::{ffi::OsString, path::PathBuf, process::exit};

#[derive(Parser)]
#[command(author = env!("CARGO_PKG_AUTHORS"), version = concat!("version ", "solang version"), about = env!("CARGO_PKG_DESCRIPTION"), subcommand_required = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {

    #[command(about = "Start LSP language server on stdin/stdout")]
    LanguageServer(LanguageServerCommand),
}

#[derive(Args)]
pub struct New {
    #[arg(name = "TARGETNAME",required= true, long = "target", value_parser = ["solana", "polkadot", "evm"], help = "Target to build for [possible values: solana, polkadot]", num_args = 1, hide_possible_values = true)]
    pub target_name: String,

    #[arg(name = "INPUT", help = "Name of the project", num_args = 1, value_parser = ValueParser::os_string())]
    pub project_name: Option<OsString>,
}

#[derive(Args)]
pub struct IdlCommand {
    #[arg(name = "INPUT", help = "Convert IDL files", required= true, value_parser = ValueParser::os_string(), num_args = 1..)]
    pub input: Vec<OsString>,

    #[arg(name = "OUTPUT",help = "output file", short = 'o', long = "output", num_args = 1, value_parser =ValueParser::path_buf())]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct LanguageServerCommand {
    #[clap(flatten)]
    pub target: TargetArg,

    #[arg(name = "IMPORTPATH", help = "Directory to search for solidity files", value_parser = ValueParser::path_buf(), action = ArgAction::Append, long = "importpath", short = 'I', num_args = 1)]
    pub import_path: Option<Vec<PathBuf>>,

    #[arg(name = "IMPORTMAP", help = "Map directory to search for solidity files [format: map=path]",value_parser = ValueParser::new(parse_import_map), action = ArgAction::Append, long = "importmap", short = 'm', num_args = 1)]
    pub import_map: Option<Vec<(String, PathBuf)>>,
}

#[derive(Args)]
pub struct ShellComplete {
    #[arg(required = true, value_parser = value_parser!(Shell), help = "Name of a supported shell")]
    pub shell_complete: Shell,
}

#[derive(Args)]
pub struct Doc {
    #[clap(flatten)]
    pub package: DocPackage,

    #[clap(flatten)]
    pub target: TargetArg,

    #[arg(name = "VERBOSE" ,help = "show debug messages", short = 'v', action = ArgAction::SetTrue, long = "verbose")]
    pub verbose: bool,

    #[arg(name = "OUTPUT",help = "output directory", short = 'o', long = "output", num_args = 1, value_parser =ValueParser::string())]
    pub output_directory: Option<OsString>,
}

#[derive(Args, Deserialize, Default, Debug, PartialEq)]
pub struct CompilerOutput {
    #[arg(name = "EMIT", help = "Emit compiler state at early stage", long = "emit", num_args = 1, value_parser = ["ast-dot", "cfg", "llvm-ir", "llvm-bc", "object", "asm"])]
    #[serde(deserialize_with = "deserialize_emit", default)]
    pub emit: Option<String>,

    #[arg(name = "STD-JSON",help = "mimic solidity json output on stdout", conflicts_with_all = ["VERBOSE", "OUTPUT", "EMIT"], action = ArgAction::SetTrue, long = "standard-json")]
    #[serde(default)]
    pub std_json_output: bool,

    #[arg(name = "OUTPUT",help = "output directory", short = 'o', long = "output", num_args = 1, value_parser =ValueParser::string())]
    #[serde(default)]
    pub output_directory: Option<String>,

    #[arg(name = "OUTPUTMETA",help = "output directory for metadata", long = "output-meta", num_args = 1, value_parser = ValueParser::string())]
    #[serde(default)]
    pub output_meta: Option<String>,

    #[arg(name = "VERBOSE" ,help = "show debug messages", short = 'v', action = ArgAction::SetTrue, long = "verbose")]
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Args)]
pub struct TargetArg {
    #[arg(name = "TARGET",required= true, long = "target", value_parser = ["solana", "polkadot", "evm"], help = "Target to build for [possible values: solana, polkadot]", num_args = 1, hide_possible_values = true)]
    pub name: String,

    #[arg(name = "ADDRESS_LENGTH", help = "Address length on the Polkadot Parachain", long = "address-length", num_args = 1, value_parser = value_parser!(u64).range(4..1024))]
    pub address_length: Option<u64>,

    #[arg(name = "VALUE_LENGTH", help = "Value length on the Polkadot Parachain", long = "value-length", num_args = 1, value_parser = value_parser!(u64).range(4..1024))]
    pub value_length: Option<u64>,
}

#[derive(Args, Deserialize, Debug, PartialEq)]
pub struct CompileTargetArg {
    #[arg(name = "TARGET", long = "target", value_parser = ["solana", "polkadot", "evm"], help = "Target to build for [possible values: solana, polkadot]", num_args = 1, hide_possible_values = true)]
    pub name: Option<String>,

    #[arg(name = "ADDRESS_LENGTH", help = "Address length on the Polkadot Parachain", long = "address-length", num_args = 1, value_parser = value_parser!(u64).range(4..1024))]
    pub address_length: Option<u64>,

    #[arg(name = "VALUE_LENGTH", help = "Value length on the Polkadot Parachain", long = "value-length", num_args = 1, value_parser = value_parser!(u64).range(4..1024))]
    pub value_length: Option<u64>,
}

#[derive(Args)]
pub struct DocPackage {
    #[arg(name = "INPUT", help = "Solidity input files",value_parser = ValueParser::path_buf(), num_args = 1.., required = true)]
    pub input: Vec<PathBuf>,

    #[arg(name = "CONTRACT", help = "Contract names to compile (defaults to all)", value_delimiter = ',', action = ArgAction::Append, long = "contract")]
    pub contracts: Option<Vec<String>>,

    #[arg(name = "IMPORTPATH", help = "Directory to search for solidity files",value_parser = ValueParser::path_buf(), action = ArgAction::Append, long = "importpath", short = 'I', num_args = 1)]
    pub import_path: Option<Vec<PathBuf>>,

    #[arg(name = "IMPORTMAP", help = "Map directory to search for solidity files [format: map=path]",value_parser = ValueParser::new(parse_import_map), action = ArgAction::Append, long = "importmap", short = 'm', num_args = 1)]
    pub import_map: Option<Vec<(String, PathBuf)>>,
}

#[derive(Args, Deserialize, Debug, PartialEq)]
pub struct CompilePackage {
    #[arg(name = "INPUT", help = "Solidity input files",value_parser = ValueParser::path_buf(), num_args = 1..)]
    #[serde(rename(deserialize = "input_files"))]
    pub input: Option<Vec<PathBuf>>,

    #[arg(name = "CONTRACT", help = "Contract names to compile (defaults to all)", value_delimiter = ',', action = ArgAction::Append, long = "contract")]
    pub contracts: Option<Vec<String>>,

    #[arg(name = "IMPORTPATH", help = "Directory to search for solidity files", value_parser = ValueParser::path_buf(), action = ArgAction::Append, long = "importpath", short = 'I', num_args = 1)]
    pub import_path: Option<Vec<PathBuf>>,

    #[arg(name = "IMPORTMAP", help = "Map directory to search for solidity files [format: map=path]",value_parser = ValueParser::new(parse_import_map), action = ArgAction::Append, long = "importmap", short = 'm', num_args = 1)]
    #[serde(deserialize_with = "deserialize_inline_table", default)]
    pub import_map: Option<Vec<(String, PathBuf)>>,

    #[arg(name = "AUTHOR", help = "specify contracts authors", long = "contract-authors", value_delimiter = ',', action = ArgAction::Append)]
    #[serde(default)]
    pub authors: Option<Vec<String>>,

    #[arg(name = "VERSION", help = "specify contracts version", long = "version", num_args = 1, value_parser = ValueParser::new(parse_version))]
    #[serde(default, deserialize_with = "deserialize_version")]
    pub version: Option<String>,
}

#[derive(Args, Deserialize, Debug, PartialEq)]
pub struct DebugFeatures {
    #[arg(name = "NOLOGAPIRETURNS", help = "Disable logging the return codes of runtime API calls in the environment", long = "no-log-api-return-codes", action = ArgAction::SetFalse)]
    #[serde(default, rename(deserialize = "log-api-return-codes"))]
    pub log_api_return_codes: bool,

    #[arg(name = "NOLOGRUNTIMEERRORS", help = "Disable logging runtime errors in the environment", long = "no-log-runtime-errors", action = ArgAction::SetFalse)]
    #[serde(default, rename(deserialize = "log-runtime-errors"))]
    pub log_runtime_errors: bool,

    #[arg(name = "NOPRINTS", help = "Disable logging prints in the environment", long = "no-prints", action = ArgAction::SetFalse)]
    #[serde(default = "default_true", rename(deserialize = "prints"))]
    pub log_prints: bool,

    #[arg(name = "GENERATEDEBUGINFORMATION", help = "Enable generating debug information for LLVM IR", long = "generate-debug-info", action = ArgAction::SetTrue, short = 'g')]
    #[serde(default, rename(deserialize = "generate-debug-info"))]
    pub generate_debug_info: bool,

    #[arg(name = "RELEASE", help = "Disable all debugging features such as prints, logging runtime errors, and logging api return codes", long = "release", action = ArgAction::SetTrue)]
    #[serde(default)]
    pub release: bool,
}

impl Default for DebugFeatures {
    fn default() -> Self {
        DebugFeatures {
            log_api_return_codes: true,
            log_runtime_errors: true,
            log_prints: true,
            generate_debug_info: false,
            release: false,
        }
    }
}


 

pub trait TargetArgTrait {
    fn get_name(&self) -> &String;
    fn get_address_length(&self) -> &Option<u64>;
    fn get_value_length(&self) -> &Option<u64>;
}

impl TargetArgTrait for TargetArg {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_address_length(&self) -> &Option<u64> {
        &self.address_length
    }

    fn get_value_length(&self) -> &Option<u64> {
        &self.value_length
    }
}

impl TargetArgTrait for CompileTargetArg {
    fn get_name(&self) -> &String {
        if let Some(name) = &self.name {
            name
        } else {
            eprintln!("error: no target name specified");
            exit(1);
        }
    }

    fn get_address_length(&self) -> &Option<u64> {
        &self.address_length
    }

    fn get_value_length(&self) -> &Option<u64> {
        &self.value_length
    }
}

pub(crate) fn target_arg<T: TargetArgTrait>(target_arg: &T) -> Target {
    let target_name = target_arg.get_name();

    if target_name == "solana" || target_name == "evm" {
        if target_arg.get_address_length().is_some() {
            eprintln!("error: address length cannot be modified except for polkadot target");
            exit(1);
        }

        if target_arg.get_value_length().is_some() {
            eprintln!("error: value length cannot be modified except for polkadot target");
            exit(1);
        }
    }

    let target = match target_name.as_str() {
        "solana" => solang::Target::Solana,
        "polkadot" => solang::Target::Polkadot {
            address_length: target_arg.get_address_length().unwrap_or(32) as usize,
            value_length: target_arg.get_value_length().unwrap_or(16) as usize,
        },
        "evm" => solang::Target::EVM,
        _ => unreachable!(),
    };

    target
}

/// This trait is used to avoid code repetition when dealing with two implementations of the Package type:
/// `CompilePackage` and `DocPackage`. Each struct represents a group of arguments for the compile and doc commands.
/// Throughout the code, these two structs are treated the same, and this trait allows for unified handling.
pub trait PackageTrait {
    fn get_input(&self) -> &Vec<PathBuf>;
    fn get_import_path(&self) -> &Option<Vec<PathBuf>>;
    fn get_import_map(&self) -> &Option<Vec<(String, PathBuf)>>;
}

impl PackageTrait for CompilePackage {
    fn get_input(&self) -> &Vec<PathBuf> {
        if let Some(files) = &self.input {
            files
        } else {
            eprintln!(
                "No input files specified, please specifiy them in solang.toml or in command line"
            );
            exit(1);
        }
    }

    fn get_import_path(&self) -> &Option<Vec<PathBuf>> {
        &self.import_path
    }

    fn get_import_map(&self) -> &Option<Vec<(String, PathBuf)>> {
        &self.import_map
    }
}

impl PackageTrait for DocPackage {
    fn get_input(&self) -> &Vec<PathBuf> {
        &self.input
    }

    fn get_import_path(&self) -> &Option<Vec<PathBuf>> {
        &self.import_path
    }

    fn get_import_map(&self) -> &Option<Vec<(String, PathBuf)>> {
        &self.import_map
    }
}

pub fn imports_arg<T: PackageTrait>(package: &T) -> FileResolver {
    let mut resolver = FileResolver::default();

    if let Some(paths) = package.get_import_path() {
        let dups: Vec<_> = paths.iter().duplicates().collect();

        if !dups.is_empty() {
            eprintln!(
                "error: import paths {} specifed more than once",
                dups.iter().map(|p| format!("'{}'", p.display())).join(", ")
            );
            exit(1);
        }

        for path in paths {
            resolver.add_import_path(path);
        }
    }

    if let Some(maps) = package.get_import_map() {
        for (map, path) in maps {
            let os_map = OsString::from(map);
            if let Some((_, existing_path)) = resolver
                .get_import_paths()
                .iter()
                .find(|(m, _)| *m == Some(os_map.clone()))
            {
                eprintln!(
                    "warning: mapping '{}' to '{}' is overwritten",
                    map,
                    existing_path.display()
                )
            }
            resolver.add_import_map(os_map, path.clone());
        }
    }

    resolver
}




// Parse the import map argument. This takes the form
/// --import-map openzeppelin=/opt/openzeppelin-contracts/contract,
/// and returns the name of the map and the path.
fn parse_import_map(map: &str) -> Result<(String, PathBuf), String> {
    if let Some((var, value)) = map.split_once('=') {
        Ok((var.to_owned(), PathBuf::from(value)))
    } else {
        Err("contains no '='".to_owned())
    }
}

fn parse_version(version: &str) -> Result<String, String> {
    match Version::parse(version) {
        Ok(version) => Ok(version.to_string()),
        Err(err) => Err(err.to_string()),
    }
}

fn deserialize_inline_table<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<(String, PathBuf)>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let res: Option<toml::Table> = Option::deserialize(deserializer)?;

    match res {
        Some(table) => Ok(Some(
            table
                .iter()
                .map(|f| {
                    (
                        f.0.clone(),
                        if f.1.is_str() {
                            PathBuf::from(f.1.as_str().unwrap())
                        } else {
                            let key = f.1;
                            eprintln!("error: invalid value for import map {key}");
                            exit(1)
                        },
                    )
                })
                .collect(),
        )),
        None => Ok(None),
    }
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let res: Option<String> = Option::deserialize(deserializer)?;

    match res {
        Some(version) => match Version::parse(&version) {
            Ok(version) => Ok(Some(version.to_string())),
            Err(err) => Err(serde::de::Error::custom(err)),
        },
        None => Ok(None),
    }
}

fn deserialize_emit<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str: Option<String> = Option::deserialize(deserializer)?;
    match str {
        Some(value) => {
            match value.as_str() {
                "ast-dot"|"cfg"|"llvm-ir"|"llvm-bc"|"object"|"asm" =>
                    Ok(Some(value))
                ,
                _ => Err(serde::de::Error::custom("Invalid option for `emit`. Valid options are: `ast-dot`, `cfg`, `llvm-ir`, `llvm-bc`, `object`, `asm`"))
            }
        }
        None => Ok(None),
    }
}

fn default_true() -> bool {
    true
}

/// Get args provided explicitly at runtime.
fn explicit_args(matches: &ArgMatches) -> Vec<&Id> {
    matches
        .ids()
        .filter(|x| {
            matches!(
                matches.value_source(x.as_str()),
                Some(ValueSource::CommandLine)
            )
        })
        .collect()
}
