// SPDX-License-Identifier: Apache-2.0

use crate::sema::ast::{Builtin, CallTy, Expression, Namespace, Type};
use crate::sema::contracts::is_base;
use crate::sema::diagnostics::Diagnostics;
use crate::Target;
use base58::{FromBase58, FromBase58Error};
use num_bigint::{BigInt, Sign};
use solang_parser::diagnostics::Diagnostic;
use solang_parser::pt;

#[derive(Clone, Copy)]
pub(crate) enum CallArgKind {
    Value,
    Gas,
    Salt,
    Accounts,
    Seeds,
    ProgramId,
    Flags,
}

impl CallArgKind {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "value" => Some(Self::Value),
            "gas" => Some(Self::Gas),
            "salt" => Some(Self::Salt),
            "accounts" => Some(Self::Accounts),
            "seeds" => Some(Self::Seeds),
            "program_id" => Some(Self::ProgramId),
            "flags" => Some(Self::Flags),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Gas => "gas",
            Self::Salt => "salt",
            Self::Accounts => "accounts",
            Self::Seeds => "seeds",
            Self::ProgramId => "program_id",
            Self::Flags => "flags",
        }
    }
}

pub(crate) enum CallArgPolicy {
    Allowed,
    Rejected { message: String, recover: bool },
}

pub(crate) enum ContractNameCallResolution {
    InternalBase,
    External,
    RejectNonBase { message: String },
    Ignore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContractNewCallPolicy {
    TreatAsFunction,
    TreatAsConstructor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstructorNamePolicy {
    UnnamedOnly { synthesized_name: &'static str },
    OptionalName { synthesized_name: &'static str },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstructorOverloadPolicy {
    SingleOnly,
    MultipleRequireConsistentPayability,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReceiveFunctionPolicy {
    Supported,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnnotationPolicy {
    Supported,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParameterAnnotationPolicy {
    Disallowed,
    ConstructorsOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorOverridePolicy {
    FunctionOnly,
    FunctionAndConstructor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DelegatecallGasPolicy {
    NoWarning,
    WarnIgnored,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContractCallArgRequirements {
    None,
    RequireAccountsAndProgramId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccountInfoMemberAccessPolicy {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddressBalancePolicy {
    Allowed,
    ThisOnly,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddressCodePolicy {
    Supported,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeCodePolicy {
    SkipImmutableCheck,
    CheckImmutables,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineAssemblyFlagPolicy {
    MemorySafeSupported,
    MemorySafeUnsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomErrorRevertPolicy {
    Supported,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TryCatchPolicy {
    Supported,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContractAnnotationPolicy {
    None,
    ProgramId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndexedEventFieldPolicy {
    NamesOptional,
    RequireNamedIndexedFields,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructOffsetPolicy {
    Standard,
    AddFieldSize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalFunctionTypePolicy {
    FixedReference,
    ValueLike,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageLayoutPolicy {
    Standard,
    Solana,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltinVarPolicy {
    Allowed,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GaspriceCallPolicy {
    NoWarning,
    WarnWhenOne,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConcreteContractPolicy {
    NotRequired,
    RequirePublicMessage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataAccountPolicy {
    Manual,
    AutoInsert,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VariableStorageTypePolicy {
    Unsupported,
    SupportedWithDefault { default_storage: &'static str },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContractTypePolicy {
    Allowed,
    FunctionsOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChecksumAddressLiteralPolicy {
    EnforceChecksumAndParse,
    RejectChecksummedLiteral,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CurrencyUnitSystem {
    Ethereum,
    Solana,
    Neutral,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FunctionSelectorStrategy {
    Keccak4,
    SolanaDiscriminator,
}

pub(crate) trait SemaHooks {
    fn constructor_name_policy(&self) -> ConstructorNamePolicy {
        ConstructorNamePolicy::UnnamedOnly {
            synthesized_name: "",
        }
    }

    fn constructor_overload_policy(&self) -> ConstructorOverloadPolicy {
        ConstructorOverloadPolicy::MultipleRequireConsistentPayability
    }

    fn receive_function_policy(&self) -> ReceiveFunctionPolicy {
        ReceiveFunctionPolicy::Supported
    }

    fn account_annotation_policy(&self) -> AnnotationPolicy {
        AnnotationPolicy::Unsupported
    }

    fn selector_override_policy(&self) -> SelectorOverridePolicy {
        SelectorOverridePolicy::FunctionOnly
    }

    fn constructor_annotation_policy(&self) -> AnnotationPolicy {
        AnnotationPolicy::Unsupported
    }

    fn parameter_annotation_policy(&self) -> ParameterAnnotationPolicy {
        ParameterAnnotationPolicy::Disallowed
    }

    fn resolve_contract_name_call(
        &self,
        callee_contract_no: usize,
        caller_contract_no: Option<usize>,
        _has_call_args: bool,
        ns: &Namespace,
    ) -> ContractNameCallResolution {
        if let Some(caller_contract_no) = caller_contract_no {
            if is_base(callee_contract_no, caller_contract_no, ns) {
                ContractNameCallResolution::InternalBase
            } else {
                ContractNameCallResolution::RejectNonBase {
                    message: "function calls via contract name are only valid for base contracts"
                        .into(),
                }
            }
        } else {
            ContractNameCallResolution::Ignore
        }
    }

    fn identifier_path_external_contract(
        &self,
        _list: &[(pt::Loc, usize)],
        _caller_contract_no: Option<usize>,
        _call_args_loc: Option<pt::Loc>,
        _ns: &Namespace,
    ) -> Option<usize> {
        None
    }

    fn contract_new_call_policy(&self) -> ContractNewCallPolicy {
        ContractNewCallPolicy::TreatAsFunction
    }

    fn address_value_transfer_error(&self, _method: &str) -> Option<String> {
        None
    }

    fn address_raw_call_type(&self, method: &str) -> Option<CallTy> {
        match method {
            "call" => Some(CallTy::Regular),
            "delegatecall" => Some(CallTy::Delegate),
            _ => None,
        }
    }

    fn delegatecall_gas_policy(&self) -> DelegatecallGasPolicy {
        DelegatecallGasPolicy::NoWarning
    }

    fn call_arg_policy(
        &self,
        arg: CallArgKind,
        _external_call: bool,
        ns: &Namespace,
    ) -> CallArgPolicy {
        match arg {
            CallArgKind::Value | CallArgKind::Gas | CallArgKind::Salt => CallArgPolicy::Allowed,
            CallArgKind::Accounts | CallArgKind::Seeds | CallArgKind::ProgramId => {
                CallArgPolicy::Rejected {
                    message: format!(
                        "'{}' not permitted for external calls or constructors on {}",
                        arg.as_str(),
                        ns.target
                    ),
                    recover: false,
                }
            }
            CallArgKind::Flags => CallArgPolicy::Rejected {
                message: "'flags' are only permitted for external calls on polkadot".into(),
                recover: false,
            },
        }
    }

    fn contract_call_arg_requirements(&self) -> ContractCallArgRequirements {
        ContractCallArgRequirements::None
    }

    fn accountinfo_member_access_policy(&self) -> AccountInfoMemberAccessPolicy {
        AccountInfoMemberAccessPolicy::Disabled
    }

    fn address_balance_policy(&self) -> AddressBalancePolicy {
        AddressBalancePolicy::Allowed
    }

    fn address_code_policy(&self) -> AddressCodePolicy {
        AddressCodePolicy::Unsupported
    }

    fn event_selector_length(&self) -> u8 {
        32
    }

    fn runtime_code_policy(&self) -> RuntimeCodePolicy {
        RuntimeCodePolicy::SkipImmutableCheck
    }

    fn inline_assembly_flag_policy(&self) -> InlineAssemblyFlagPolicy {
        InlineAssemblyFlagPolicy::MemorySafeUnsupported
    }

    fn custom_error_revert_policy(&self) -> CustomErrorRevertPolicy {
        CustomErrorRevertPolicy::Supported
    }

    fn try_catch_policy(&self) -> TryCatchPolicy {
        TryCatchPolicy::Supported
    }

    fn contract_annotation_policy(&self) -> ContractAnnotationPolicy {
        ContractAnnotationPolicy::None
    }

    fn indexed_event_field_policy(&self) -> IndexedEventFieldPolicy {
        IndexedEventFieldPolicy::NamesOptional
    }

    fn struct_offset_policy(&self) -> StructOffsetPolicy {
        StructOffsetPolicy::Standard
    }

    fn external_function_type_policy(&self) -> ExternalFunctionTypePolicy {
        ExternalFunctionTypePolicy::FixedReference
    }

    fn storage_layout_policy(&self) -> StorageLayoutPolicy {
        StorageLayoutPolicy::Standard
    }

    fn builtin_var_policy(&self, _builtin: Builtin) -> BuiltinVarPolicy {
        BuiltinVarPolicy::Allowed
    }

    fn builtin_var_error(&self, _builtin: Builtin) -> Option<&'static str> {
        None
    }

    fn gasprice_call_policy(&self) -> GaspriceCallPolicy {
        GaspriceCallPolicy::NoWarning
    }

    fn concrete_contract_policy(&self) -> ConcreteContractPolicy {
        ConcreteContractPolicy::NotRequired
    }

    fn data_account_policy(&self) -> DataAccountPolicy {
        DataAccountPolicy::Manual
    }

    fn variable_storage_type_policy(&self) -> VariableStorageTypePolicy {
        VariableStorageTypePolicy::Unsupported
    }

    fn max_array_dimension(&self) -> Option<BigInt> {
        None
    }

    fn normalize_resolved_primitive_type(
        &self,
        ty: Type,
        _ns: &mut Namespace,
        _loc: pt::Loc,
    ) -> Type {
        ty
    }

    fn contract_type_policy(&self, _in_function_type: bool) -> ContractTypePolicy {
        ContractTypePolicy::Allowed
    }

    fn checksum_address_literal_policy(&self) -> ChecksumAddressLiteralPolicy {
        ChecksumAddressLiteralPolicy::RejectChecksummedLiteral
    }

    fn currency_unit_system(&self) -> CurrencyUnitSystem {
        CurrencyUnitSystem::Neutral
    }

    fn function_selector_strategy(&self) -> FunctionSelectorStrategy {
        FunctionSelectorStrategy::Keccak4
    }

    fn resolve_address_literal(
        &self,
        loc: &pt::Loc,
        address: &str,
        ns: &Namespace,
        diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()>;
}

struct PolkadotHooks;
struct SolanaHooks;
struct SorobanHooks;
struct EvmHooks;
struct DefaultHooks;

impl SemaHooks for PolkadotHooks {
    fn constructor_name_policy(&self) -> ConstructorNamePolicy {
        ConstructorNamePolicy::OptionalName {
            synthesized_name: "new",
        }
    }

    fn selector_override_policy(&self) -> SelectorOverridePolicy {
        SelectorOverridePolicy::FunctionAndConstructor
    }

    fn delegatecall_gas_policy(&self) -> DelegatecallGasPolicy {
        DelegatecallGasPolicy::WarnIgnored
    }

    fn address_balance_policy(&self) -> AddressBalancePolicy {
        AddressBalancePolicy::ThisOnly
    }

    fn indexed_event_field_policy(&self) -> IndexedEventFieldPolicy {
        IndexedEventFieldPolicy::RequireNamedIndexedFields
    }

    fn builtin_var_policy(&self, builtin: Builtin) -> BuiltinVarPolicy {
        if builtin == Builtin::Gasprice {
            BuiltinVarPolicy::Error
        } else {
            BuiltinVarPolicy::Allowed
        }
    }

    fn builtin_var_error(&self, builtin: Builtin) -> Option<&'static str> {
        if builtin == Builtin::Gasprice {
            Some(
                "use the function 'tx.gasprice(gas)' in stead, as 'tx.gasprice' may round down to zero. See https://solang.readthedocs.io/en/latest/language/builtins.html#gasprice",
            )
        } else {
            None
        }
    }

    fn gasprice_call_policy(&self) -> GaspriceCallPolicy {
        GaspriceCallPolicy::WarnWhenOne
    }

    fn concrete_contract_policy(&self) -> ConcreteContractPolicy {
        ConcreteContractPolicy::RequirePublicMessage
    }

    fn call_arg_policy(
        &self,
        arg: CallArgKind,
        external_call: bool,
        ns: &Namespace,
    ) -> CallArgPolicy {
        if matches!(arg, CallArgKind::Flags) && external_call {
            CallArgPolicy::Allowed
        } else {
            DefaultHooks.call_arg_policy(arg, external_call, ns)
        }
    }

    fn max_array_dimension(&self) -> Option<BigInt> {
        Some(u32::MAX.into())
    }

    fn resolve_address_literal(
        &self,
        loc: &pt::Loc,
        address: &str,
        ns: &Namespace,
        diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()> {
        match address.from_base58() {
            Ok(v) => {
                if v.len() != ns.address_length + 3 {
                    diagnostics.push(Diagnostic::error(
                        *loc,
                        format!(
                            "address literal {} incorrect length of {}",
                            address,
                            v.len()
                        ),
                    ));
                    return Err(());
                }

                let hash_data: Vec<u8> = b"SS58PRE"
                    .iter()
                    .chain(v[..=ns.address_length].iter())
                    .cloned()
                    .collect();

                let hash = blake2_rfc::blake2b::blake2b(64, &[], &hash_data);
                let hash = hash.as_bytes();

                if v[ns.address_length + 1] != hash[0] || v[ns.address_length + 2] != hash[1] {
                    diagnostics.push(Diagnostic::error(
                        *loc,
                        format!("address literal {address} hash incorrect checksum"),
                    ));
                    return Err(());
                }

                Ok(Expression::NumberLiteral {
                    loc: *loc,
                    ty: Type::Address(false),
                    value: BigInt::from_bytes_be(Sign::Plus, &v[1..ns.address_length + 1]),
                })
            }
            Err(FromBase58Error::InvalidBase58Length) => {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    format!("address literal {address} invalid base58 length"),
                ));
                Err(())
            }
            Err(FromBase58Error::InvalidBase58Character(ch, pos)) => {
                let mut err_loc = *loc;
                if let pt::Loc::File(_, start, end) = &mut err_loc {
                    *start += pos;
                    *end = *start;
                }

                diagnostics.push(Diagnostic::error(
                    err_loc,
                    format!("address literal {address} invalid character '{ch}'"),
                ));
                Err(())
            }
        }
    }
}

impl SemaHooks for SolanaHooks {
    fn receive_function_policy(&self) -> ReceiveFunctionPolicy {
        ReceiveFunctionPolicy::Unsupported
    }

    fn account_annotation_policy(&self) -> AnnotationPolicy {
        AnnotationPolicy::Supported
    }

    fn constructor_annotation_policy(&self) -> AnnotationPolicy {
        AnnotationPolicy::Supported
    }

    fn parameter_annotation_policy(&self) -> ParameterAnnotationPolicy {
        ParameterAnnotationPolicy::ConstructorsOnly
    }

    fn resolve_contract_name_call(
        &self,
        callee_contract_no: usize,
        caller_contract_no: Option<usize>,
        has_call_args: bool,
        ns: &Namespace,
    ) -> ContractNameCallResolution {
        if let Some(caller_contract_no) = caller_contract_no {
            if is_base(callee_contract_no, caller_contract_no, ns) && !has_call_args {
                return ContractNameCallResolution::InternalBase;
            }
        }

        ContractNameCallResolution::External
    }

    fn identifier_path_external_contract(
        &self,
        list: &[(pt::Loc, usize)],
        caller_contract_no: Option<usize>,
        call_args_loc: Option<pt::Loc>,
        ns: &Namespace,
    ) -> Option<usize> {
        if list.len() == 1 && ns.functions[list[0].1].contract_no != caller_contract_no {
            if let (Some(callee), Some(caller)) =
                (ns.functions[list[0].1].contract_no, caller_contract_no)
            {
                if is_base(callee, caller, ns) && call_args_loc.is_none() {
                    return None;
                }
            }

            return ns.functions[list[0].1].contract_no;
        }

        None
    }

    fn contract_new_call_policy(&self) -> ContractNewCallPolicy {
        ContractNewCallPolicy::TreatAsConstructor
    }

    fn address_value_transfer_error(&self, method: &str) -> Option<String> {
        Some(format!(
            "method '{}' not available on Solana. Use the lamports \
field from the AccountInfo struct directly to operate on balances.",
            method
        ))
    }

    fn address_raw_call_type(&self, method: &str) -> Option<CallTy> {
        match method {
            "call" => Some(CallTy::Regular),
            _ => None,
        }
    }

    fn call_arg_policy(
        &self,
        arg: CallArgKind,
        _external_call: bool,
        ns: &Namespace,
    ) -> CallArgPolicy {
        match arg {
            CallArgKind::Value => CallArgPolicy::Rejected {
                message: "Solana Cross Program Invocation (CPI) cannot transfer native value. See https://solang.readthedocs.io/en/latest/language/functions.html#value_transfer".to_string(),
                recover: true,
            },
            CallArgKind::Gas | CallArgKind::Salt => CallArgPolicy::Rejected {
                message: format!(
                    "'{}' not permitted for external calls or constructors on {}",
                    arg.as_str(),
                    ns.target
                ),
                recover: false,
            },
            CallArgKind::Accounts | CallArgKind::Seeds | CallArgKind::ProgramId => {
                CallArgPolicy::Allowed
            }
            CallArgKind::Flags => CallArgPolicy::Rejected {
                message: "'flags' are only permitted for external calls on polkadot".into(),
                recover: false,
            },
        }
    }

    fn contract_call_arg_requirements(&self) -> ContractCallArgRequirements {
        ContractCallArgRequirements::RequireAccountsAndProgramId
    }

    fn accountinfo_member_access_policy(&self) -> AccountInfoMemberAccessPolicy {
        AccountInfoMemberAccessPolicy::Enabled
    }

    fn address_balance_policy(&self) -> AddressBalancePolicy {
        AddressBalancePolicy::Unsupported
    }

    fn event_selector_length(&self) -> u8 {
        8
    }

    fn custom_error_revert_policy(&self) -> CustomErrorRevertPolicy {
        CustomErrorRevertPolicy::Unsupported
    }

    fn try_catch_policy(&self) -> TryCatchPolicy {
        TryCatchPolicy::Unsupported
    }

    fn contract_annotation_policy(&self) -> ContractAnnotationPolicy {
        ContractAnnotationPolicy::ProgramId
    }

    fn struct_offset_policy(&self) -> StructOffsetPolicy {
        StructOffsetPolicy::AddFieldSize
    }

    fn storage_layout_policy(&self) -> StorageLayoutPolicy {
        StorageLayoutPolicy::Solana
    }

    fn builtin_var_policy(&self, builtin: Builtin) -> BuiltinVarPolicy {
        if matches!(builtin, Builtin::Value | Builtin::Sender) {
            BuiltinVarPolicy::Error
        } else {
            BuiltinVarPolicy::Allowed
        }
    }

    fn builtin_var_error(&self, builtin: Builtin) -> Option<&'static str> {
        match builtin {
            Builtin::Value => Some(
                "Solana Cross Program Invocation (CPI) cannot transfer native value. See https://solang.readthedocs.io/en/latest/language/functions.html#value_transfer",
            ),
            Builtin::Sender => Some(
                "'msg.sender' is not available on Solana. See https://solang.readthedocs.io/en/latest/targets/solana.html#msg-sender-solana",
            ),
            _ => None,
        }
    }

    fn data_account_policy(&self) -> DataAccountPolicy {
        DataAccountPolicy::AutoInsert
    }

    fn contract_type_policy(&self, in_function_type: bool) -> ContractTypePolicy {
        if in_function_type {
            ContractTypePolicy::Allowed
        } else {
            ContractTypePolicy::FunctionsOnly
        }
    }

    fn currency_unit_system(&self) -> CurrencyUnitSystem {
        CurrencyUnitSystem::Solana
    }

    fn function_selector_strategy(&self) -> FunctionSelectorStrategy {
        FunctionSelectorStrategy::SolanaDiscriminator
    }

    fn resolve_address_literal(
        &self,
        loc: &pt::Loc,
        address: &str,
        ns: &Namespace,
        diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()> {
        match address.from_base58() {
            Ok(v) => {
                if v.len() != ns.address_length {
                    diagnostics.push(Diagnostic::error(
                        *loc,
                        format!(
                            "address literal {} incorrect length of {}",
                            address,
                            v.len()
                        ),
                    ));
                    Err(())
                } else {
                    Ok(Expression::NumberLiteral {
                        loc: *loc,
                        ty: Type::Address(false),
                        value: BigInt::from_bytes_be(Sign::Plus, &v),
                    })
                }
            }
            Err(FromBase58Error::InvalidBase58Length) => {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    format!("address literal {address} invalid base58 length"),
                ));
                Err(())
            }
            Err(FromBase58Error::InvalidBase58Character(ch, pos)) => {
                let mut err_loc = *loc;
                if let pt::Loc::File(_, start, end) = &mut err_loc {
                    *start += pos;
                    *end = *start;
                }

                diagnostics.push(Diagnostic::error(
                    err_loc,
                    format!("address literal {address} invalid character '{ch}'"),
                ));
                Err(())
            }
        }
    }
}

impl SemaHooks for SorobanHooks {
    fn variable_storage_type_policy(&self) -> VariableStorageTypePolicy {
        VariableStorageTypePolicy::SupportedWithDefault {
            default_storage: "persistent",
        }
    }

    fn normalize_resolved_primitive_type(
        &self,
        ty: Type,
        ns: &mut Namespace,
        loc: pt::Loc,
    ) -> Type {
        ty.round_soroban_width(ns, loc)
    }

    fn resolve_address_literal(
        &self,
        loc: &pt::Loc,
        address: &str,
        _ns: &Namespace,
        _diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()> {
        Ok(Expression::BytesLiteral {
            loc: *loc,
            ty: Type::Address(true),
            value: address.as_bytes().to_vec(),
        })
    }
}

impl SemaHooks for EvmHooks {
    fn constructor_overload_policy(&self) -> ConstructorOverloadPolicy {
        ConstructorOverloadPolicy::SingleOnly
    }

    fn address_raw_call_type(&self, method: &str) -> Option<CallTy> {
        match method {
            "staticcall" => Some(CallTy::Static),
            _ => DefaultHooks.address_raw_call_type(method),
        }
    }

    fn address_code_policy(&self) -> AddressCodePolicy {
        AddressCodePolicy::Supported
    }

    fn runtime_code_policy(&self) -> RuntimeCodePolicy {
        RuntimeCodePolicy::CheckImmutables
    }

    fn inline_assembly_flag_policy(&self) -> InlineAssemblyFlagPolicy {
        InlineAssemblyFlagPolicy::MemorySafeSupported
    }

    fn external_function_type_policy(&self) -> ExternalFunctionTypePolicy {
        ExternalFunctionTypePolicy::ValueLike
    }

    fn checksum_address_literal_policy(&self) -> ChecksumAddressLiteralPolicy {
        ChecksumAddressLiteralPolicy::EnforceChecksumAndParse
    }

    fn currency_unit_system(&self) -> CurrencyUnitSystem {
        CurrencyUnitSystem::Ethereum
    }

    fn resolve_address_literal(
        &self,
        loc: &pt::Loc,
        address: &str,
        ns: &Namespace,
        diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()> {
        DefaultHooks.resolve_address_literal(loc, address, ns, diagnostics)
    }
}

impl SemaHooks for DefaultHooks {
    fn resolve_address_literal(
        &self,
        loc: &pt::Loc,
        address: &str,
        ns: &Namespace,
        diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()> {
        diagnostics.push(Diagnostic::error(
            *loc,
            format!("address literal {} not supported on {}", address, ns.target),
        ));
        Err(())
    }
}

static POLKADOT_HOOKS: PolkadotHooks = PolkadotHooks;
static SOLANA_HOOKS: SolanaHooks = SolanaHooks;
static SOROBAN_HOOKS: SorobanHooks = SorobanHooks;
static EVM_HOOKS: EvmHooks = EvmHooks;

pub(crate) fn sema_hooks(ns: &Namespace) -> &'static dyn SemaHooks {
    match ns.target {
        Target::Polkadot { .. } => &POLKADOT_HOOKS,
        Target::Solana => &SOLANA_HOOKS,
        Target::Soroban => &SOROBAN_HOOKS,
        Target::EVM => &EVM_HOOKS,
    }
}
