// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) struct SorobanTarget;

impl TargetCodegen for SorobanTarget {
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
        let inner_ty = if let Type::StorageRef(_, inner) = args[0].ty() {
            if let Type::Array(elem_ty, _) = inner.deref_any() {
                elem_ty.clone()
            } else {
                panic!("expected storage array type");
            }
        } else {
            panic!("expected storage reference type");
        };

        if !inner_ty.is_reference_type(ns) {
            return soroban_storage_push(loc, args, cfg, contract_no, func, ns, vartab, opt);
        }

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
        let index = Expression::Variable {
            loc: *loc,
            ty: slot_ty.clone(),
            var_no: length_pos,
        };
        let index_encoded = soroban_encode_arg(index, cfg, vartab, ns);

        cfg.add(
            vartab,
            Instr::Set {
                loc: pt::Loc::Codegen,
                res: entry_pos,
                expr: Expression::Subscript {
                    loc: *loc,
                    ty: elem_ty.clone(),
                    array_ty: Type::StorageRef(false, Box::new(elem_ty.clone())),
                    expr: Box::new(var_expr.clone()),
                    index: Box::new(index_encoded),
                },
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

        let new_length = self.store_storage(
            Expression::Add {
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
            },
            cfg,
            vartab,
            ns,
        );

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
        soroban_storage_pop(
            loc,
            args,
            return_ty,
            cfg,
            contract_no,
            func,
            ns,
            vartab,
            opt,
        )
    }

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
        soroban_storage_push(loc, args, cfg, contract_no, func, ns, vartab, opt)
    }

    fn storage_initializer_default(
        &self,
        loc: &pt::Loc,
        ty: &Type,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        _ns: &Namespace,
    ) -> Option<Expression> {
        if ty.is_dynamic_memory() {
            Some(crate::codegen::soroban::soroban_vec_new(
                loc, ty, cfg, vartab,
            ))
        } else {
            None
        }
    }

    fn array_length(
        &self,
        loc: &pt::Loc,
        ty: &Type,
        array_ty: &Type,
        array: Expression,
        elem_ty: &Type,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
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
                ArrayLength::Dynamic => Expression::StorageArrayLength {
                    loc: *loc,
                    ty: ns.storage_type(),
                    array: Box::new(array),
                    elem_ty: elem_ty.clone(),
                },
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

    fn array_len_subscript(
        &self,
        loc: &pt::Loc,
        array: Expression,
        storage_array_ty: &Type,
        _cfg: &mut ControlFlowGraph,
        _vartab: &mut Vartable,
        ns: &Namespace,
    ) -> (Expression, Expression) {
        (
            Expression::StorageArrayLength {
                loc: *loc,
                ty: ns.storage_type(),
                array: Box::new(array.clone()),
                elem_ty: storage_array_ty.storage_array_elem().deref_into(),
            },
            array,
        )
    }

    fn struct_member(
        &self,
        loc: &pt::Loc,
        base: Expression,
        offset: BigInt,
        _storage_ty: Type,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> Expression {
        // Soroban storage keys are vectors; append the member offset to derive a field key.
        let offset_encoded = soroban_encode_arg(
            Expression::NumberLiteral {
                loc: *loc,
                ty: Type::Uint(32),
                value: offset,
            },
            cfg,
            vartab,
            ns,
        );

        let res = vartab.temp_name("vec_push_codegen", &Type::Uint(64));
        let var = Expression::Variable {
            loc: pt::Loc::Codegen,
            ty: Type::Uint(64),
            var_no: res,
        };

        cfg.add(
            vartab,
            Instr::Call {
                res: vec![res],
                return_tys: vec![Type::Uint(64)],
                call: InternalCallTy::HostFunction {
                    name: HostFunctions::VecPushBack.name().to_string(),
                },
                args: vec![base, offset_encoded],
            },
        );

        var
    }

    fn load(
        &self,
        loc: &pt::Loc,
        ty: Type,
        ptr: Box<Expression>,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> Expression {
        if let Type::Ref(inner) = ptr.ty() {
            if matches!(inner.as_ref(), Type::SorobanHandle(_)) {
                let load_handle = Expression::Load {
                    loc: *loc,
                    ty: inner.as_ref().clone(),
                    expr: ptr,
                };

                return soroban_decode_arg(load_handle, cfg, vartab, ns, None);
            }
        }

        Expression::Load {
            loc: *loc,
            ty,
            expr: ptr,
        }
    }

    fn store_storage(
        &self,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> Expression {
        soroban_encode_arg(value, cfg, vartab, ns)
    }

    fn store_ref(
        &self,
        dest: &Expression,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> Expression {
        if matches!(
            dest.ty(),
            Type::Ref(inner) if matches!(inner.as_ref(), Type::SorobanHandle(_))
        ) {
            soroban_encode_arg(value, cfg, vartab, ns)
        } else {
            value
        }
    }

    fn get_address(
        &self,
        loc: &pt::Loc,
        _ns: &Namespace,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
    ) -> Expression {
        let address_var_no = vartab.temp_anonymous(&Type::Uint(64));
        let address_var = Expression::Variable {
            loc: *loc,
            ty: Type::Address(false),
            var_no: address_var_no,
        };

        cfg.add(
            vartab,
            Instr::Call {
                res: vec![address_var_no],
                return_tys: vec![Type::Uint(64)],
                call: InternalCallTy::HostFunction {
                    name: HostFunctions::GetCurrentContractAddress.name().to_string(),
                },
                args: vec![],
            },
        );

        address_var
    }

    fn load_storage(
        &self,
        value: Expression,
        cfg: &mut ControlFlowGraph,
        vartab: &mut Vartable,
        ns: &Namespace,
    ) -> Expression {
        soroban_decode_arg(value, cfg, vartab, ns, None)
    }

    fn subscript_runtime_types(
        &self,
        declared_array_ty: &Type,
        declared_elem_ty: &Type,
        runtime_array_ty: &Type,
    ) -> (Type, Type) {
        if let Type::Array(runtime_elem_ty, runtime_dims) = runtime_array_ty {
            if matches!(runtime_elem_ty.as_ref(), Type::SorobanHandle(_)) {
                let effective_array_ty = Type::Array(runtime_elem_ty.clone(), runtime_dims.clone());
                let effective_elem_ty = if matches!(declared_elem_ty, Type::Ref(_)) {
                    Type::Ref(runtime_elem_ty.clone())
                } else {
                    runtime_elem_ty.as_ref().clone()
                };

                return (effective_array_ty, effective_elem_ty);
            }
        }

        (declared_array_ty.clone(), declared_elem_ty.clone())
    }

    fn storage_subscript(
        &self,
        loc: &pt::Loc,
        array_ty: &Type,
        storage_array_ty: &Type,
        array: Expression,
        _array_length: &Expression,
        index: Expression,
        _index_loc: pt::Loc,
        _coerced_ty: &Type,
        _pos: usize,
        cfg: &mut ControlFlowGraph,
        ns: &Namespace,
        vartab: &mut Vartable,
    ) -> Expression {
        let elem_ty = storage_array_ty.storage_array_elem();
        let index = index.cast(&Type::Uint(64), ns);
        let index = if elem_ty.is_reference_type(ns) {
            soroban_encode_arg(index, cfg, vartab, ns)
        } else {
            index
        };

        Expression::Subscript {
            loc: *loc,
            ty: elem_ty,
            array_ty: array_ty.clone(),
            expr: Box::new(array),
            index: Box::new(index),
        }
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
