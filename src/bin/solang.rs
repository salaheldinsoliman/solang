// SPDX-License-Identifier: Apache-2.0

use clap::{ CommandFactory, FromArgMatches};

use wasm_bindgen::prelude::*;

use crate::cli::{
    Cli, Commands, 
};

mod cli;
mod languageserver;

#[wasm_bindgen]
pub fn main() {  
    let matches = Cli::command().get_matches();

    let cli = Cli::from_arg_matches(&matches).unwrap();

    match cli.command {
        Commands::LanguageServer(server_args) => languageserver::start_server(&server_args),
    }
}