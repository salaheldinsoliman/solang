// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) struct EvmTarget;

impl TargetCodegen for EvmTarget {
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
    ) -> Expression {
        match array_ty {
            Type::Bytes(length) => Expression::NumberLiteral {
                loc: *loc,
                ty: ty.clone(),
                value: BigInt::from(*length),
            },
            Type::DynamicBytes | Type::String => Expression::StorageArrayLength {
                loc: *loc,
                ty: ty.clone(),
                array: Box::new(array),
                elem_ty: elem_ty.clone(),
            },
            Type::Array(_, dim) => match dim.last().unwrap() {
                ArrayLength::Dynamic => {
                    load_storage(loc, &ns.storage_type(), array, cfg, vartab, None, ns)
                }
                ArrayLength::Fixed(length) => Expression::NumberLiteral {
                    loc: *loc,
                    ty: ty.clone(),
                    value: length.clone(),
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn gasprice_units_arg(&self) -> bool {
        true
    }

    fn default_gas(&self) -> BigInt {
        BigInt::from(i64::MAX)
    }

    fn payable_send(
        &self,
        loc: &pt::Loc,
        address: Expression,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        _ns: &Namespace,
        vartab: &mut Vartable,
        _opt: &Options,
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
            Instr::ExternalCall {
                loc: *loc,
                success: Some(success),
                address: Some(address),
                accounts: ExternalCallAccounts::AbsentArgument,
                seeds: None,
                payload: empty_external_call_payload(loc),
                value,
                gas: Expression::NumberLiteral {
                    loc: *loc,
                    ty: Type::Uint(64),
                    value: BigInt::from(i64::MAX),
                },
                callty: CallTy::Regular,
                contract_function_no: None,
                flags: None,
            },
        );

        Expression::Variable {
            loc: *loc,
            ty: Type::Bool,
            var_no: success,
        }
    }

    fn payable_transfer(
        &self,
        loc: &pt::Loc,
        address: Expression,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        _ns: &Namespace,
        vartab: &mut Vartable,
        _opt: &Options,
    ) -> Expression {
        cfg.add(
            vartab,
            Instr::ExternalCall {
                loc: *loc,
                success: None,
                accounts: ExternalCallAccounts::AbsentArgument,
                seeds: None,
                address: Some(address),
                payload: empty_external_call_payload(loc),
                value,
                gas: Expression::NumberLiteral {
                    loc: *loc,
                    ty: Type::Uint(64),
                    value: BigInt::from(i64::MAX),
                },
                callty: CallTy::Regular,
                contract_function_no: None,
                flags: None,
            },
        );

        Expression::Poison
    }
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
    ) -> Expression {
        if array_ty.is_storage_bytes() {
            return Expression::Subscript {
                loc: *loc,
                ty: elem_ty.clone(),
                array_ty: array_ty.clone(),
                expr: Box::new(expression(array, cfg, contract_no, func, ns, vartab, opt)),
                index: Box::new(expression(index, cfg, contract_no, func, ns, vartab, opt)),
            };
        }

        if array_ty.is_mapping() {
            let array = expression(array, cfg, contract_no, func, ns, vartab, opt);
            let index = expression(index, cfg, contract_no, func, ns, vartab, opt);

            return self.mapping_subscript(loc, elem_ty, array_ty, array, index);
        }

        let mut array = expression(array, cfg, contract_no, func, ns, vartab, opt);
        let index_ty = index.ty();
        let index = expression(index, cfg, contract_no, func, ns, vartab, opt);
        let index_loc = index.loc();

        let index_width = index_ty.bits(ns);

        let array_length = match array_ty.deref_any() {
            Type::Bytes(n) => {
                let ast_bigint = bigint_to_expression(
                    &array.loc(),
                    &BigInt::from(*n),
                    ns,
                    &mut Diagnostics::default(),
                    ResolveTo::Unknown,
                    None,
                )
                .unwrap();
                expression(&ast_bigint, cfg, contract_no, func, ns, vartab, opt)
            }
            Type::Array(..) => match array_ty.array_length() {
                None => {
                    if let Type::StorageRef(..) = array_ty {
                        let (array_length, updated_array) =
                            self.array_len_subscript(loc, array, array_ty, cfg, vartab, ns);
                        array = updated_array;
                        array_length
                    } else {
                        // If a subscript is encountered array length will be called

                        // Return array length by default
                        let mut returned = Expression::Builtin {
                            loc: *loc,
                            tys: vec![Type::Uint(32)],
                            kind: Builtin::ArrayLength,
                            args: vec![array.clone()],
                        };

                        if let Expression::Variable {
                            loc, var_no: num, ..
                        } = &array
                        {
                            // If the size is known (is in cfg.array_length_map), do the replacement

                            if let Some(array_length_var) = cfg.array_lengths_temps.get(num) {
                                returned = Expression::Variable {
                                    loc: *loc,
                                    ty: Type::Uint(32),
                                    var_no: *array_length_var,
                                };
                            }
                        }
                        returned
                    }
                }
                Some(l) => {
                    let ast_big_int = bigint_to_expression(
                        loc,
                        l,
                        ns,
                        &mut Diagnostics::default(),
                        ResolveTo::Unknown,
                        None,
                    )
                    .unwrap();
                    expression(&ast_big_int, cfg, contract_no, func, ns, vartab, opt)
                }
            },
            Type::DynamicBytes | Type::Slice(_) => Expression::Builtin {
                loc: *loc,
                tys: vec![Type::Uint(32)],
                kind: Builtin::ArrayLength,
                args: vec![array.clone()],
            },
            _ => {
                unreachable!();
            }
        };

        let array_width = array_length.ty().bits(ns);
        let width = std::cmp::max(array_width, index.ty().bits(ns));
        let coerced_ty = Type::Uint(width);

        let pos = vartab.temp(
            &pt::Identifier {
                name: "index".to_owned(),
                loc: *loc,
            },
            &coerced_ty,
        );

        let expr = index.cast(&coerced_ty, ns);
        cfg.add(
            vartab,
            Instr::Set {
                loc: expr.loc(),
                res: pos,
                expr,
            },
        );

        // If the array is fixed length and the index also constant, the
        // branch will be optimized away.
        let out_of_bounds = cfg.new_basic_block("out_of_bounds".to_string());
        let in_bounds = cfg.new_basic_block("in_bounds".to_string());

        cfg.add(
            vartab,
            Instr::BranchCond {
                cond: Expression::MoreEqual {
                    loc: *loc,
                    signed: false,
                    left: Box::new(Expression::Variable {
                        loc: index_loc,
                        ty: coerced_ty.clone(),
                        var_no: pos,
                    }),
                    right: Box::new(array_length.cast(&coerced_ty, ns)),
                },
                true_block: out_of_bounds,
                false_block: in_bounds,
            },
        );

        cfg.set_basic_block(out_of_bounds);
        log_runtime_error(
            opt.log_runtime_errors,
            "array index out of bounds",
            *loc,
            cfg,
            vartab,
            ns,
        );
        let error = SolidityError::Panic(PanicCode::ArrayIndexOob);
        assert_failure(loc, error, ns, cfg, vartab);

        cfg.set_basic_block(in_bounds);

        if let Type::Bytes(array_length) = array_ty.deref_any() {
            let res_ty = Type::Bytes(1);
            let from_ty = Type::Bytes(*array_length);
            let index_ty = Type::Uint(*array_length as u16 * 8);

            let to_width = array_ty.bits(ns);
            let shift_arg_raw = Expression::Variable {
                loc: index_loc,
                ty: coerced_ty.clone(),
                var_no: pos,
            };

            let shift_arg = match index_width.cmp(&to_width) {
                Ordering::Equal => shift_arg_raw,
                Ordering::Less => Expression::ZeroExt {
                    loc: *loc,
                    ty: index_ty.clone(),
                    expr: shift_arg_raw.into(),
                },
                Ordering::Greater => Expression::Trunc {
                    loc: *loc,
                    ty: index_ty.clone(),
                    expr: shift_arg_raw.into(),
                },
            };

            return Expression::Trunc {
                loc: *loc,
                ty: res_ty,
                expr: Expression::ShiftRight {
                    loc: *loc,
                    ty: from_ty,
                    left: array.into(),
                    right: Expression::ShiftLeft {
                        loc: *loc,
                        ty: index_ty.clone(),
                        left: Box::new(Expression::Subtract {
                            loc: *loc,
                            ty: index_ty.clone(),
                            overflowing: true,
                            left: Expression::NumberLiteral {
                                loc: *loc,
                                ty: index_ty.clone(),
                                value: BigInt::from_u8(array_length - 1).unwrap(),
                            }
                            .into(),
                            right: shift_arg.into(),
                        }),
                        right: Expression::NumberLiteral {
                            loc: *loc,
                            ty: index_ty,
                            value: BigInt::from_u8(3).unwrap(),
                        }
                        .into(),
                    }
                    .into(),
                    signed: false,
                }
                .into(),
            };
        }

        if let Type::StorageRef(_, storage_array_ty) = &array_ty {
            return self.storage_subscript(
                loc,
                array_ty,
                storage_array_ty,
                array,
                &array_length,
                index,
                index_loc,
                &coerced_ty,
                pos,
                cfg,
                ns,
                vartab,
            );
        } else {
            let (effective_array_ty, effective_elem_ty) =
                self.subscript_runtime_types(array_ty, elem_ty, array.ty().deref_any());

            match effective_array_ty.deref_memory() {
                Type::DynamicBytes | Type::Array(..) | Type::Slice(_) => Expression::Subscript {
                    loc: *loc,
                    ty: effective_elem_ty,
                    array_ty: effective_array_ty,
                    expr: Box::new(array),
                    index: Box::new(Expression::Variable {
                        loc: index_loc,
                        ty: coerced_ty,
                        var_no: pos,
                    }),
                },
                _ => {
                    // should not happen as type-checking already done
                    unreachable!();
                }
            }
        }
    }
}
