// SPDX-License-Identifier: Apache-2.0

use super::{
    cfg::{ControlFlowGraph, Instr, InternalCallTy},
    constructor::call_constructor,
    expression::{expression, load_storage},
    polkadot as codegen_polkadot,
    revert::{assert_failure, log_runtime_error, PanicCode, SolidityError},
    soroban::{soroban_storage_pop, soroban_storage_push},
    storage::array_offset,
    vartable::Vartable,
    Options,
};
use crate::codegen::encoding::soroban_encoding::{soroban_decode_arg, soroban_encode_arg};
use crate::codegen::{Builtin, Expression, HostFunctions};
use crate::sema::ast::{
    self, ArrayLength, CallTy, ExternalCallAccounts, Namespace, RetrieveType, StructType, Type,
};
use crate::sema::diagnostics::Diagnostics;
use crate::sema::expression::integers::bigint_to_expression;
use crate::sema::expression::ResolveTo;
use crate::Target;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One, ToPrimitive, Zero};
use solang_parser::pt::{self, CodeLocation};
use std::cmp::Ordering;
use std::ops::Mul;

mod evm;
mod polkadot;
mod solana;
mod soroban;

use self::evm::EvmTarget;
use self::polkadot::PolkadotTarget;
use self::solana::SolanaTarget;
use self::soroban::SorobanTarget;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageArrayBuiltinKind {
    PushPopInstruction,
    SlotBased,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrintBehavior {
    Plain,
    PrefixAndDelimiter,
}

pub(super) fn empty_external_call_payload(loc: &pt::Loc) -> Expression {
    Expression::AllocDynamicBytes {
        loc: *loc,
        ty: Type::DynamicBytes,
        size: Box::new(Expression::NumberLiteral {
            loc: *loc,
            ty: Type::Uint(32),
            value: BigInt::from(0),
        }),
        initializer: Some(vec![]),
    }
}

pub(crate) trait TargetCodegen {
    fn constructor_success(&self, _vartab: &mut Vartable) -> Option<usize> {
        None
    }

    fn constructor_result(
        &self,
        _loc: pt::Loc,
        _success: Option<usize>,
        _cfg: &mut ControlFlowGraph,
        _ns: &Namespace,
        _opt: &Options,
        _vartab: &mut Vartable,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    fn subscript(
        &self,
        loc: &pt::Loc,
        elem_ty: &Type,
        array_ty: &Type,
        array: &ast::Expression,
        index: &ast::Expression,
        cfg: &mut ControlFlowGraph,
        contract_no: usize,
        func: Option<&ast::Function>,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression;

    #[allow(clippy::too_many_arguments)]
    fn constructor(
        &self,
        loc: &pt::Loc,
        constructor_contract: usize,
        caller_contract_no: usize,
        constructor_no: &Option<usize>,
        args: &[ast::Expression],
        call_args: &ast::CallArgs,
        cfg: &mut ControlFlowGraph,
        func: Option<&ast::Function>,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression {
        let address_res = vartab.temp_anonymous(&Type::Contract(constructor_contract));
        let success = self.constructor_success(vartab);

        call_constructor(
            loc,
            constructor_contract,
            caller_contract_no,
            constructor_no,
            args,
            call_args,
            address_res,
            success,
            func,
            ns,
            vartab,
            cfg,
            opt,
        );

        self.constructor_result(*loc, success, cfg, ns, opt, vartab);

        Expression::Variable {
            loc: *loc,
            ty: Type::Contract(constructor_contract),
            var_no: address_res,
        }
    }

    fn array_kind(&self, array_ty: &Type) -> StorageArrayBuiltinKind {
        if array_ty.is_storage_bytes() {
            StorageArrayBuiltinKind::PushPopInstruction
        } else {
            StorageArrayBuiltinKind::SlotBased
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn storage_slots_array_push(
        &self,
        loc: &pt::Loc,
        args: &[ast::Expression],
        cfg: &mut ControlFlowGraph,
        contract_no: usize,
        func: Option<&ast::Function>,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression {
        let slot_ty = ns.storage_type();
        let length_pos = vartab.temp_anonymous(&slot_ty);

        let var_expr = expression(&args[0], cfg, contract_no, func, ns, vartab, opt);

        let expr = load_storage(loc, &slot_ty, var_expr.clone(), cfg, vartab, None, ns);

        cfg.add(
            vartab,
            Instr::Set {
                loc: pt::Loc::Codegen,
                res: length_pos,
                expr,
            },
        );

        let elem_ty = args[0].ty().storage_array_elem();
        let entry_pos = vartab.temp_anonymous(&slot_ty);
        let offset = array_offset(
            loc,
            Expression::Keccak256 {
                loc: *loc,
                ty: slot_ty.clone(),
                exprs: vec![var_expr.clone()],
            },
            Expression::Variable {
                loc: *loc,
                ty: slot_ty.clone(),
                var_no: length_pos,
            },
            elem_ty.clone(),
            ns,
        );

        cfg.add(
            vartab,
            Instr::Set {
                loc: pt::Loc::Codegen,
                res: entry_pos,
                expr: offset,
            },
        );

        if args.len() == 2 {
            let value = expression(&args[1], cfg, contract_no, func, ns, vartab, opt);
            let value = self.store_storage(value, cfg, vartab, ns);

            cfg.add(
                vartab,
                Instr::SetStorage {
                    ty: elem_ty.clone(),
                    value,
                    storage: Expression::Variable {
                        loc: *loc,
                        ty: slot_ty.clone(),
                        var_no: entry_pos,
                    },
                    storage_type: None,
                },
            );
        }

        let new_length = Expression::Add {
            loc: *loc,
            ty: slot_ty.clone(),
            overflowing: true,
            left: Box::new(Expression::Variable {
                loc: *loc,
                ty: slot_ty.clone(),
                var_no: length_pos,
            }),
            right: Box::new(Expression::NumberLiteral {
                loc: *loc,
                ty: slot_ty.clone(),
                value: BigInt::one(),
            }),
        };
        let new_length = self.store_storage(new_length, cfg, vartab, ns);

        cfg.add(
            vartab,
            Instr::SetStorage {
                ty: slot_ty,
                value: new_length,
                storage: var_expr,
                storage_type: None,
            },
        );

        if args.len() == 1 {
            Expression::Variable {
                loc: *loc,
                ty: elem_ty,
                var_no: entry_pos,
            }
        } else {
            Expression::Poison
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn storage_slots_array_pop(
        &self,
        loc: &pt::Loc,
        args: &[ast::Expression],
        return_ty: &Type,
        cfg: &mut ControlFlowGraph,
        contract_no: usize,
        func: Option<&ast::Function>,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression {
        let slot_ty = ns.storage_type();
        let length_ty = ns.storage_type();
        let length_pos = vartab.temp_anonymous(&slot_ty);

        let ty = args[0].ty();
        let var_expr = expression(&args[0], cfg, contract_no, func, ns, vartab, opt);
        let expr = load_storage(loc, &length_ty, var_expr.clone(), cfg, vartab, None, ns);

        cfg.add(
            vartab,
            Instr::Set {
                loc: pt::Loc::Codegen,
                res: length_pos,
                expr,
            },
        );

        let empty_array = cfg.new_basic_block("empty_array".to_string());
        let has_elements = cfg.new_basic_block("has_elements".to_string());

        cfg.add(
            vartab,
            Instr::BranchCond {
                cond: Expression::Equal {
                    loc: *loc,
                    left: Box::new(Expression::Variable {
                        loc: *loc,
                        ty: length_ty.clone(),
                        var_no: length_pos,
                    }),
                    right: Box::new(Expression::NumberLiteral {
                        loc: *loc,
                        ty: length_ty.clone(),
                        value: BigInt::zero(),
                    }),
                },
                true_block: empty_array,
                false_block: has_elements,
            },
        );

        cfg.set_basic_block(empty_array);
        log_runtime_error(
            opt.log_runtime_errors,
            "pop from empty storage array",
            *loc,
            cfg,
            vartab,
            ns,
        );
        let error = SolidityError::Panic(PanicCode::EmptyArrayPop);
        assert_failure(loc, error, ns, cfg, vartab);

        cfg.set_basic_block(has_elements);
        let new_length = vartab.temp_anonymous(&slot_ty);

        let subtract = Expression::Subtract {
            loc: *loc,
            ty: length_ty.clone(),
            overflowing: true,
            left: Box::new(Expression::Variable {
                loc: *loc,
                ty: length_ty.clone(),
                var_no: length_pos,
            }),
            right: Box::new(Expression::NumberLiteral {
                loc: *loc,
                ty: length_ty.clone(),
                value: BigInt::one(),
            }),
        };

        cfg.add(
            vartab,
            Instr::Set {
                loc: pt::Loc::Codegen,
                res: new_length,
                expr: subtract,
            },
        );

        let elem_ty = ty.storage_array_elem().deref_any().clone();
        let entry_pos = vartab.temp_anonymous(&slot_ty);
        let offset = array_offset(
            loc,
            Expression::Keccak256 {
                loc: *loc,
                ty: slot_ty.clone(),
                exprs: vec![var_expr.clone()],
            },
            Expression::Variable {
                loc: *loc,
                ty: slot_ty.clone(),
                var_no: new_length,
            },
            elem_ty.clone(),
            ns,
        );

        cfg.add(
            vartab,
            Instr::Set {
                loc: pt::Loc::Codegen,
                res: entry_pos,
                expr: offset,
            },
        );

        let val = if *return_ty != Type::Void {
            let res_pos = vartab.temp_anonymous(&elem_ty);

            let expr = load_storage(
                loc,
                &elem_ty,
                Expression::Variable {
                    loc: *loc,
                    ty: elem_ty.clone(),
                    var_no: entry_pos,
                },
                cfg,
                vartab,
                None,
                ns,
            );

            cfg.add(
                vartab,
                Instr::Set {
                    loc: *loc,
                    res: res_pos,
                    expr,
                },
            );
            Expression::Variable {
                loc: *loc,
                ty: elem_ty.clone(),
                var_no: res_pos,
            }
        } else {
            Expression::Undefined {
                ty: elem_ty.clone(),
            }
        };

        cfg.add(
            vartab,
            Instr::ClearStorage {
                ty: elem_ty,
                storage: Expression::Variable {
                    loc: *loc,
                    ty: slot_ty.clone(),
                    var_no: entry_pos,
                },
            },
        );

        cfg.add(
            vartab,
            Instr::SetStorage {
                ty: slot_ty.clone(),
                value: Expression::Variable {
                    loc: *loc,
                    ty: slot_ty,
                    var_no: new_length,
                },
                storage: var_expr,
                storage_type: None,
            },
        );

        val
    }

    #[allow(clippy::too_many_arguments)]
    fn array_push(
        &self,
        loc: &pt::Loc,
        args: &[ast::Expression],
        cfg: &mut ControlFlowGraph,
        contract_no: usize,
        func: Option<&ast::Function>,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression {
        let storage = expression(&args[0], cfg, contract_no, func, ns, vartab, opt);

        let mut ty = args[0].ty().storage_array_elem();

        let value = if args.len() > 1 {
            Some(expression(
                &args[1],
                cfg,
                contract_no,
                func,
                ns,
                vartab,
                opt,
            ))
        } else {
            ty.deref_any().default(ns)
        };

        if !ty.is_reference_type(ns) {
            ty = ty.deref_into();
        }

        let res = vartab.temp_anonymous(&ty);

        cfg.add(
            vartab,
            Instr::PushStorage {
                res,
                ty: ty.deref_any().clone(),
                storage,
                value,
            },
        );

        Expression::Variable {
            loc: *loc,
            ty,
            var_no: res,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn array_pop(
        &self,
        loc: &pt::Loc,
        args: &[ast::Expression],
        return_ty: &Type,
        cfg: &mut ControlFlowGraph,
        contract_no: usize,
        func: Option<&ast::Function>,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression {
        let storage = expression(&args[0], cfg, contract_no, func, ns, vartab, opt);
        let ty = args[0].ty().storage_array_elem().deref_into();

        let res = if *return_ty != Type::Void {
            Some(vartab.temp_anonymous(&ty))
        } else {
            None
        };

        cfg.add(
            vartab,
            Instr::PopStorage {
                res,
                ty: ty.clone(),
                storage,
            },
        );

        if let Some(res) = res {
            Expression::Variable {
                loc: *loc,
                ty,
                var_no: res,
            }
        } else {
            Expression::Undefined { ty }
        }
    }

    fn array_length(
        &self,
        loc: &pt::Loc,
        ty: &Type,
        array_ty: &Type,
        array: Expression,
        elem_ty: &Type,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> Expression;

    fn array_len_subscript(
        &self,
        loc: &pt::Loc,
        array: Expression,
        storage_array_ty: &Type,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> (Expression, Expression) {
        // Default EVM/Polkadot layout: first slot is length, data starts at keccak(slot).
        let array_length = load_storage(
            loc,
            &ns.storage_type(),
            array.clone(),
            cfg,
            vartab,
            None,
            ns,
        );
        let hashed_base = Expression::Keccak256 {
            loc: *loc,
            ty: Type::Uint(256),
            exprs: vec![array],
        };

        // Keep `storage_array_ty` in the signature to make target overrides explicit and future-proof.
        let _ = storage_array_ty;

        (array_length, hashed_base)
    }

    fn struct_offset(&self, struct_ty: StructType, field_no: usize, ns: &Namespace) -> BigInt {
        struct_ty.definition(ns).fields[..field_no]
            .iter()
            .filter(|field| !field.infinite_size)
            .map(|field| field.ty.storage_slots(ns))
            .sum()
    }

    fn struct_member(
        &self,
        loc: &pt::Loc,
        base: Expression,
        offset: BigInt,
        storage_ty: Type,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Expression {
        Expression::Add {
            loc: *loc,
            ty: storage_ty.clone(),
            overflowing: true,
            left: Box::new(base),
            right: Box::new(Expression::NumberLiteral {
                loc: *loc,
                ty: storage_ty,
                value: offset,
            }),
        }
    }

    fn load(
        &self,
        loc: &pt::Loc,
        ty: Type,
        ptr: Box<Expression>,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Expression {
        Expression::Load {
            loc: *loc,
            ty,
            expr: ptr,
        }
    }

    fn store_storage(
        &self,
        value: Expression,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Expression {
        value
    }

    fn storage_initializer_default(
        &self,
        _loc: &pt::Loc,
        _ty: &Type,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Option<Expression> {
        None
    }

    fn store_ref(
        &self,
        _dest: &Expression,
        value: Expression,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Expression {
        value
    }

    fn print(&self) -> PrintBehavior {
        PrintBehavior::Plain
    }

    fn gasprice_units_arg(&self) -> bool {
        false
    }

    fn selector_len(&self, ns: &Namespace) -> u8 {
        ns.target.selector_length()
    }

    fn abi_sig_hash(&self) -> ast::Builtin {
        ast::Builtin::Keccak256
    }

    fn get_address(
        &self,
        loc: &pt::Loc,
        _ns: &Namespace,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
    ) -> Expression {
        let codegen_expr = Expression::Builtin {
            loc: *loc,
            tys: vec![Type::Ref(Box::new(Type::Address(false)))],
            kind: Builtin::GetAddress,
            args: vec![],
        };

        Expression::Load {
            loc: *loc,
            ty: Type::Address(false),
            expr: Box::new(codegen_expr),
        }
    }

    fn default_gas(&self) -> BigInt {
        BigInt::zero()
    }

    fn payable_send(
        &self,
        loc: &pt::Loc,
        address: Expression,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        ns: &Namespace,
        vartab: &mut Vartable,
        opt: &Options,
    ) -> Expression {
        let success = vartab.temp(
            &solang_parser::pt::Identifier {
                loc: *loc,
                name: "success".to_owned(),
            },
            &Type::Uint(32),
        );

        cfg.add(
            vartab,
            Instr::ValueTransfer {
                success: Some(success),
                address,
                value,
            },
        );

        codegen_polkadot::check_transfer_ret(loc, success, cfg, ns, opt, vartab, false).unwrap()
    }

    fn payable_transfer(
        &self,
        _loc: &pt::Loc,
        address: Expression,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        _ns: &Namespace,
        vartab: &mut Vartable,
        _opt: &Options,
    ) -> Expression {
        cfg.add(
            vartab,
            Instr::ValueTransfer {
                success: None,
                address,
                value,
            },
        );

        Expression::Poison
    }

    fn storage_subscript(
        &self,
        loc: &pt::Loc,
        _array_ty: &Type,
        storage_array_ty: &Type,
        array: Expression,
        array_length: &Expression,
        _index: Expression,
        index_loc: pt::Loc,
        coerced_ty: &Type,
        pos: usize,
        _cfg: &mut ControlFlowGraph,
        ns: &Namespace,
        _vartab: &mut Vartable,
    ) -> Expression {
        let elem_ty = storage_array_ty.storage_array_elem();
        let slot_ty = ns.storage_type();
        let elem_size = elem_ty.storage_slots(ns);

        if let Expression::NumberLiteral {
            value: arr_length, ..
        } = array_length
        {
            if arr_length.mul(elem_size.clone()).to_u64().is_some() {
                // Use a narrower path when offsets fit in u64 for cheaper wasm arithmetic.
                return Expression::Add {
                    loc: *loc,
                    ty: elem_ty.clone(),
                    overflowing: true,
                    left: Box::new(array),
                    right: Box::new(Expression::ZeroExt {
                        loc: *loc,
                        ty: slot_ty,
                        expr: Box::new(Expression::Multiply {
                            loc: *loc,
                            ty: Type::Uint(64),
                            overflowing: true,
                            left: Box::new(
                                Expression::Variable {
                                    loc: index_loc,
                                    ty: coerced_ty.clone(),
                                    var_no: pos,
                                }
                                .cast(&Type::Uint(64), ns),
                            ),
                            right: Box::new(Expression::NumberLiteral {
                                loc: *loc,
                                ty: Type::Uint(64),
                                value: elem_size,
                            }),
                        }),
                    }),
                };
            }
        }

        array_offset(
            loc,
            array,
            Expression::Variable {
                loc: index_loc,
                ty: coerced_ty.clone(),
                var_no: pos,
            }
            .cast(&ns.storage_type(), ns),
            elem_ty,
            ns,
        )
    }

    fn mapping_subscript(
        &self,
        loc: &pt::Loc,
        elem_ty: &Type,
        array_ty: &Type,
        array: Expression,
        index: Expression,
    ) -> Expression {
        Expression::Subscript {
            loc: *loc,
            ty: elem_ty.clone(),
            array_ty: array_ty.clone(),
            expr: Box::new(array),
            index: Box::new(index),
        }
    }

    fn raw_call_success(&self, loc: &pt::Loc, success_var: usize) -> Expression {
        Expression::Variable {
            loc: *loc,
            ty: Type::Uint(32),
            var_no: success_var,
        }
    }

    fn external_call_success(&self, _vartab: &mut Vartable) -> Option<usize> {
        None
    }

    fn external_call_status(
        &self,
        _loc: pt::Loc,
        _success: Option<usize>,
        _cfg: &mut ControlFlowGraph,
        _ns: &Namespace,
        _opt: &Options,
        _vartab: &mut Vartable,
        _msg: &'static str,
    ) {
    }

    fn load_storage(
        &self,
        value: Expression,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Expression {
        value
    }

    fn subscript_runtime_types(
        &self,
        declared_array_ty: &Type,
        declared_elem_ty: &Type,
        _runtime_array_ty: &Type,
    ) -> (Type, Type) {
        (declared_array_ty.clone(), declared_elem_ty.clone())
    }
}

static POLKADOT_TARGET: PolkadotTarget = PolkadotTarget;
static SOLANA_TARGET: SolanaTarget = SolanaTarget;
static SOROBAN_TARGET: SorobanTarget = SorobanTarget;
static EVM_TARGET: EvmTarget = EvmTarget;

pub(crate) fn target_codegen(ns: &Namespace) -> &'static dyn TargetCodegen {
    match ns.target {
        Target::Polkadot { .. } => &POLKADOT_TARGET,
        Target::Solana => &SOLANA_TARGET,
        Target::Soroban => &SOROBAN_TARGET,
        Target::EVM => &EVM_TARGET,
    }
}
