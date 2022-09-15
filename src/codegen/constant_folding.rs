// SPDX-License-Identifier: Apache-2.0

use super::cfg::{ControlFlowGraph, Instr, InstrOrigin};
use super::reaching_definitions;
use crate::codegen::{Builtin, Expression};
use crate::sema::Recurse;
use crate::sema::ast::RetrieveType;
use crate::sema::ast::{Diagnostic, Namespace, StringLocation, Type};
use num_bigint::{BigInt, Sign};
use num_traits::{ToPrimitive, Zero};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use solang_parser::pt::Loc;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use tiny_keccak::{Hasher, Keccak};

/// Constant folding pass on the given cfg. During constant folding, we may find issues
/// like divide by zero, so this function returns a list of diagnostics which should
/// be added to the namespace.
pub fn constant_folding(cfg: &mut ControlFlowGraph, ns: &mut Namespace) {
    // for each block, instruction
    for block_no in 0..cfg.blocks.len() {
        let mut vars = cfg.blocks[block_no].defs.clone();

        for instr_no in 0..cfg.blocks[block_no].instr.len() {
            let instr_origin = cfg.blocks[block_no].instr[instr_no].0;
            match &cfg.blocks[block_no].instr[instr_no].1 {
                Instr::Set { loc, res, expr, .. } => {
                    let (expr, expr_constant) =
                        expression(expr, Some(&vars), cfg, ns, instr_origin);

                    if expr_constant {
                        ns.var_constants.insert(*loc, expr.clone());
                    }

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::Set {
                        loc: *loc,
                        res: *res,
                        expr: expr.clone(),
                    };
                    println!("EXPR == > {:?}", expr.clone()); 
                    //let sesa = expr.recurse(&mut Expression::NumberLiteral{0:*loc,1:ty,2:num}, f);
                    println!("EXPR TYPE ==> {:?}", cfg.blocks[block_no].instr[instr_no].0);
                }
                Instr::Call {
                    res,
                    call,
                    args,
                    return_tys,
                } => {
                    let args = args
                        .iter()
                        .map(|e| expression(e, Some(&vars), cfg, ns, instr_origin).0)
                        .collect();

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::Call {
                        res: res.clone(),
                        call: call.clone(),
                        args,
                        return_tys: return_tys.clone(),
                    };
                }
                Instr::Return { value } => {
                    let value = value
                        .iter()
                        .map(|e| expression(e, Some(&vars), cfg, ns, instr_origin).0)
                        .collect();

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::Return { value };
                }
                Instr::BranchCond {
                    cond,
                    true_block,
                    false_block,
                } => {
                    let (cond, _) = expression(cond, Some(&vars), cfg, ns, instr_origin);

                    if let Expression::BoolLiteral(_, cond) = cond {
                        cfg.blocks[block_no].instr[instr_no].1 = Instr::Branch {
                            block: if cond { *true_block } else { *false_block },
                        };
                    } else {
                        cfg.blocks[block_no].instr[instr_no].1 = Instr::BranchCond {
                            cond,
                            true_block: *true_block,
                            false_block: *false_block,
                        };
                    }
                }
                Instr::Store { dest, data } => {
                    let (dest, _) = expression(dest, Some(&vars), cfg, ns, instr_origin);
                    let (data, _) = expression(data, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::Store { dest, data };
                }
                Instr::AssertFailure { expr: Some(expr) } => {
                    let (expr, _) = expression(expr, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 =
                        Instr::AssertFailure { expr: Some(expr) };
                }
                Instr::Print { expr } => {
                    let (expr, _) = expression(expr, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::Print { expr };
                }
                Instr::ClearStorage { ty, storage } => {
                    let (storage, _) = expression(storage, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::ClearStorage {
                        ty: ty.clone(),
                        storage,
                    };
                }
                Instr::SetStorage { ty, storage, value } => {
                    let (storage, _) = expression(storage, Some(&vars), cfg, ns, instr_origin);
                    let (value, _) = expression(value, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::SetStorage {
                        ty: ty.clone(),
                        storage,
                        value,
                    };
                }
                Instr::LoadStorage { ty, storage, res } => {
                    let (storage, _) = expression(storage, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::LoadStorage {
                        ty: ty.clone(),
                        storage,
                        res: *res,
                    };
                }
                Instr::SetStorageBytes {
                    storage,
                    value,
                    offset,
                } => {
                    let (storage, _) = expression(storage, Some(&vars), cfg, ns, instr_origin);
                    let (value, _) = expression(value, Some(&vars), cfg, ns, instr_origin);
                    let (offset, _) = expression(offset, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::SetStorageBytes {
                        storage,
                        value,
                        offset,
                    };
                }
                Instr::PushStorage {
                    res,
                    ty,
                    storage,
                    value,
                } => {
                    let (storage, _) = expression(storage, Some(&vars), cfg, ns, instr_origin);
                    let value = value
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::PushStorage {
                        res: *res,
                        ty: ty.clone(),
                        storage,
                        value,
                    };
                }
                Instr::PopStorage { res, ty, storage } => {
                    let (storage, _) = expression(storage, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::PopStorage {
                        res: *res,
                        ty: ty.clone(),
                        storage,
                    };
                }
                Instr::PushMemory {
                    res,
                    ty,
                    array,
                    value,
                } => {
                    let (value, _) = expression(value, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::PushMemory {
                        res: *res,
                        ty: ty.clone(),
                        array: *array,
                        value: Box::new(value),
                    };
                }
                Instr::Constructor {
                    success,
                    res,
                    contract_no,
                    constructor_no,
                    args,
                    value,
                    gas,
                    salt,
                    space,
                } => {
                    let args = args
                        .iter()
                        .map(|e| expression(e, Some(&vars), cfg, ns, instr_origin).0)
                        .collect();
                    let value = value
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);
                    let gas = expression(gas, Some(&vars), cfg, ns, instr_origin).0;
                    let salt = salt
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);
                    let space = space
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::Constructor {
                        success: *success,
                        res: *res,
                        contract_no: *contract_no,
                        constructor_no: *constructor_no,
                        args,
                        value,
                        gas,
                        salt,
                        space,
                    };
                }
                Instr::ExternalCall {
                    success,
                    address,
                    payload,
                    value,
                    gas,
                    accounts,
                    seeds,
                    callty,
                } => {
                    let value = expression(value, Some(&vars), cfg, ns, instr_origin).0;
                    let gas = expression(gas, Some(&vars), cfg, ns, instr_origin).0;
                    let payload = expression(payload, Some(&vars), cfg, ns, instr_origin).0;
                    let address = address
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);
                    let accounts = accounts
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);
                    let seeds = seeds
                        .as_ref()
                        .map(|expr| expression(expr, Some(&vars), cfg, ns, instr_origin).0);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::ExternalCall {
                        success: *success,
                        address,
                        accounts,
                        seeds,
                        payload,
                        value,
                        gas,
                        callty: callty.clone(),
                    };
                }
                Instr::AbiDecode {
                    res,
                    selector,
                    exception_block,
                    tys,
                    data,
                } => {
                    let (data, _) = expression(data, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::AbiDecode {
                        res: res.clone(),
                        selector: *selector,
                        exception_block: *exception_block,
                        tys: tys.clone(),
                        data,
                    }
                }
                Instr::SelfDestruct { recipient } => {
                    let (recipient, _) = expression(recipient, Some(&vars), cfg, ns, instr_origin);

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::SelfDestruct { recipient };
                }
                Instr::EmitEvent {
                    event_no,
                    data,
                    data_tys,
                    topics,
                    topic_tys,
                } => {
                    let data = data
                        .iter()
                        .map(|e| expression(e, Some(&vars), cfg, ns, instr_origin).0)
                        .collect();

                    let topics = topics
                        .iter()
                        .map(|e| expression(e, Some(&vars), cfg, ns, instr_origin).0)
                        .collect();

                    cfg.blocks[block_no].instr[instr_no].1 = Instr::EmitEvent {
                        event_no: *event_no,
                        data,
                        data_tys: data_tys.clone(),
                        topics,
                        topic_tys: topic_tys.clone(),
                    }
                }
                Instr::MemCopy {
                    source,
                    destination,
                    bytes,
                } => {
                    let bytes = expression(bytes, Some(&vars), cfg, ns, instr_origin);
                    let source = expression(source, Some(&vars), cfg, ns, instr_origin);
                    let destination = expression(destination, Some(&vars), cfg, ns, instr_origin);
                    cfg.blocks[block_no].instr[instr_no].1 = Instr::MemCopy {
                        source: source.0,
                        destination: destination.0,
                        bytes: bytes.0,
                    };
                }
                _ => (),
            }

            reaching_definitions::apply_transfers(
                &cfg.blocks[block_no].transfers[instr_no],
                &mut vars,
            );
        }
    }
}

/// Recursively walk the expression and fold any constant expressions or variables. This function returns the
/// constant folded expression, and a boolean which is true if the value is "pure", the value does not depend
/// on context. This is used for constant folding, so that e.g. an external function call is not constant
/// folded (and moved/copied as a result).
fn expression(
    expr: &Expression,
    vars: Option<&reaching_definitions::VarDefs>,
    cfg: &ControlFlowGraph,
    ns: &mut Namespace,
    instr_origin: InstrOrigin,
) -> (Expression, bool, BigInt) {
    match expr {
        Expression::Add(loc, ty, unchecked, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                if instr_origin == InstrOrigin::Solidity {
                    // overflow_check(ns, left.add(right), ty.clone(), *loc);
                }
                //println!( "IS const")
                bigint_to_expression(instr_origin, loc, ty, left.add(right), ns)
            } else {
                (
                    Expression::Add(
                        *loc,
                        ty.clone(),
                        *unchecked,
                        Box::new(left.0),
                        Box::new(right.0),
                    ),
                    left.1 && right.1,
                )
            }
        }
        Expression::Subtract(loc, ty, unchecked, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                if instr_origin == InstrOrigin::Solidity {
                    // overflow_check(ns, left.sub(right), ty.clone(), *loc);
                }
                bigint_to_expression(instr_origin, loc, ty, left.sub(right), ns)
            } else {
                (
                    Expression::Subtract(
                        *loc,
                        ty.clone(),
                        *unchecked,
                        Box::new(left.0),
                        Box::new(right.0),
                    ),
                    left.1 && right.1,
                )
            }
        }
        Expression::AdvancePointer {
            pointer,
            bytes_offset: offset,
        } => {
            // Only the offset can be simplified
            let offset = expression(offset, vars, cfg, ns, instr_origin);

            match &offset.0 {
                // There is no reason to advance the pointer by a zero offset
                Expression::NumberLiteral(_, _, num) if num.is_zero() => (*pointer.clone(), false),

                _ => (
                    Expression::AdvancePointer {
                        pointer: pointer.clone(),
                        bytes_offset: Box::new(offset.0),
                    },
                    offset.1,
                ),
            }
        }
        Expression::Multiply(loc, ty, unchecked, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                // overflow_check(ns, left.mul(right), ty.clone(), *loc);
                bigint_to_expression(instr_origin, loc, ty, left.mul(right), ns)
            } else {
                (
                    Expression::Multiply(
                        *loc,
                        ty.clone(),
                        *unchecked,
                        Box::new(left.0),
                        Box::new(right.0),
                    ),
                    left.1 && right.1,
                )
            }
        }
        Expression::BitwiseAnd(loc, ty, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                bigint_to_expression(instr_origin, loc, ty, left.bitand(right), ns)
            } else {
                (
                    Expression::BitwiseAnd(*loc, ty.clone(), Box::new(left.0), Box::new(right.0)),
                    left.1 && right.1,
                )
            }
        }
        Expression::BitwiseOr(loc, ty, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                bigint_to_expression(instr_origin, loc, ty, left.bitor(right), ns)
            } else {
                (
                    Expression::BitwiseOr(*loc, ty.clone(), Box::new(left.0), Box::new(right.0)),
                    left.1 && right.1,
                )
            }
        }
        Expression::BitwiseXor(loc, ty, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                bigint_to_expression(instr_origin, loc, ty, left.bitxor(right),ns)
            } else {
                (
                    Expression::BitwiseXor(*loc, ty.clone(), Box::new(left.0), Box::new(right.0)),
                    left.1 && right.1,
                )
            }
        }
        Expression::ShiftLeft(loc, ty, left_expr, right_expr) => {
            let left = expression(left_expr, vars, cfg, ns, instr_origin);
            let right = expression(right_expr, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                if right.sign() == Sign::Minus || right >= &BigInt::from(left_expr.ty().bits(ns)) {
                    ns.diagnostics.push(Diagnostic::error(
                        *loc,
                        format!("left shift by {} is not possible", right),
                    ));
                } else {
                    let right: u64 = right.to_u64().unwrap();
                    if instr_origin == InstrOrigin::Solidity {
                        // overflow_check(ns, left.shl(right), ty.clone(), *loc);
                    }
                    return bigint_to_expression(instr_origin, loc, ty, left.shl(&right),ns);
                }
            }
            (
                Expression::ShiftLeft(*loc, ty.clone(), Box::new(left.0), Box::new(right.0)),
                left.1 && right.1,
            )
        }
        Expression::ShiftRight(loc, ty, left_expr, right_expr, signed) => {
            let left = expression(left_expr, vars, cfg, ns, instr_origin);
            let right = expression(right_expr, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                if right.sign() == Sign::Minus || right >= &BigInt::from(left_expr.ty().bits(ns)) {
                    ns.diagnostics.push(Diagnostic::error(
                        *loc,
                        format!("right shift by {} is not possible", right),
                    ));
                } else {
                    let right: u64 = right.to_u64().unwrap();
                    if instr_origin == InstrOrigin::Solidity {
                        // overflow_check(ns, left.shr(right), ty.clone(), *loc);
                    }
                    return bigint_to_expression(instr_origin, loc, ty, left.shr(&right),ns);
                }
            }

            (
                Expression::ShiftRight(
                    *loc,
                    ty.clone(),
                    Box::new(left.0),
                    Box::new(right.0),
                    *signed,
                ),
                left.1 && right.1,
            )
        }
        Expression::Power(loc, ty, unchecked, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left.0, &right.0)
            {
                if right.sign() == Sign::Minus || right >= &BigInt::from(u32::MAX) {
                    ns.diagnostics.push(Diagnostic::error(
                        *loc,
                        format!("power {} not possible", right),
                    ));
                } else {
                    let right: u32 = right.to_u32().unwrap();
                    if instr_origin == InstrOrigin::Solidity {
                        // overflow_check(ns, left.pow(right), ty.clone(), *loc);
                    }
                    return bigint_to_expression(instr_origin, loc, ty, left.pow(right),ns);
                }
            }

            (
                Expression::Power(
                    *loc,
                    ty.clone(),
                    *unchecked,
                    Box::new(left.0),
                    Box::new(right.0),
                ),
                left.1 && right.1,
            )
        }
        Expression::UnsignedDivide(loc, ty, left, right)
        | Expression::SignedDivide(loc, ty, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let Expression::NumberLiteral(_, _, right) = &right.0 {
                if right.is_zero() {
                    ns.diagnostics
                        .push(Diagnostic::error(*loc, String::from("divide by zero")));
                } else if let Expression::NumberLiteral(_, _, left) = &left.0 {
                    if instr_origin == InstrOrigin::Solidity {
                        // overflow_check(ns, left.div(right), ty.clone(), *loc);
                    }
                    return bigint_to_expression(instr_origin, loc, ty, left.div(right),ns);
                }
            }
            (
                if matches!(expr, Expression::SignedDivide(..)) {
                    Expression::SignedDivide(*loc, ty.clone(), Box::new(left.0), Box::new(right.0))
                } else {
                    Expression::UnsignedDivide(
                        *loc,
                        ty.clone(),
                        Box::new(left.0),
                        Box::new(right.0),
                    )
                },
                left.1 && right.1,
            )
        }
        Expression::SignedModulo(loc, ty, left, right)
        | Expression::UnsignedModulo(loc, ty, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            if let Expression::NumberLiteral(_, _, right) = &right.0 {
                if right.is_zero() {
                    ns.diagnostics
                        .push(Diagnostic::error(*loc, String::from("divide by zero")));
                } else if let Expression::NumberLiteral(_, _, left) = &left.0 {
                    return bigint_to_expression(instr_origin, loc, ty, left.rem(right),ns);
                }
            }

            (
                if matches!(expr, Expression::SignedModulo(..)) {
                    Expression::SignedModulo(*loc, ty.clone(), Box::new(left.0), Box::new(right.0))
                } else {
                    Expression::UnsignedModulo(
                        *loc,
                        ty.clone(),
                        Box::new(left.0),
                        Box::new(right.0),
                    )
                },
                left.1 && right.1,
            )
        }
        Expression::ZeroExt(loc, ty, expr) => {
            let expr = expression(expr, vars, cfg, ns, instr_origin);
            if let Expression::NumberLiteral(_, _, n) = expr.0 {
                (Expression::NumberLiteral(*loc, ty.clone(), n), true)
            } else {
                (
                    Expression::ZeroExt(*loc, ty.clone(), Box::new(expr.0)),
                    expr.1,
                )
            }
        }
        Expression::SignExt(loc, ty, expr) => {
            let expr = expression(expr, vars, cfg, ns, instr_origin);
            if let Expression::NumberLiteral(_, _, n) = expr.0 {
                (Expression::NumberLiteral(*loc, ty.clone(), n), true)
            } else {
                (
                    Expression::SignExt(*loc, ty.clone(), Box::new(expr.0)),
                    expr.1,
                )
            }
        }
        Expression::Trunc(loc, ty, expr) => {
            let expr = expression(expr, vars, cfg, ns, instr_origin);
            if let Expression::NumberLiteral(_, _, n) = expr.0 {
                bigint_to_expression(instr_origin, loc, ty, n, ns)
            } else {
                (
                    Expression::Trunc(*loc, ty.clone(), Box::new(expr.0)),
                    expr.1,
                )
            }
        }
        Expression::Complement(loc, ty, expr) => {
            let expr = expression(expr, vars, cfg, ns, instr_origin);
            if let Expression::NumberLiteral(_, _, n) = expr.0 {
                bigint_to_expression(instr_origin, loc, ty, !n, ns)
            } else {
                (
                    Expression::Complement(*loc, ty.clone(), Box::new(expr.0)),
                    expr.1,
                )
            }
        }
        Expression::UnaryMinus(loc, ty, expr) => {
            let expr = expression(expr, vars, cfg, ns, instr_origin);
            if let Expression::NumberLiteral(_, _, n) = expr.0 {
                if instr_origin == InstrOrigin::Solidity {
                    // overflow_check(ns, -n.clone(), ty.clone(), *loc)
                }
                bigint_to_expression(instr_origin, loc, ty, -n, ns)
            } else {
                (
                    Expression::UnaryMinus(*loc, ty.clone(), Box::new(expr.0)),
                    expr.1,
                )
            }
        }
        Expression::Variable(loc, ty, var) => {
            if !matches!(ty, Type::Ref(_) | Type::StorageRef(..)) {
                if let Some(vars) = vars {
                    if let Some(defs) = vars.get(var) {
                        // There must be at least one definition, and all should evaluate to the same value
                        let mut v = None;

                        for def in defs.keys() {
                            if let Some(expr) = get_definition(def, cfg) {
                                let expr = expression(expr, None, cfg, ns, instr_origin);

                                if expr.1 {
                                    if let Some(last) = &v {
                                        if !constants_equal(last, &expr.0) {
                                            v = None;
                                            break;
                                        }
                                    }

                                    v = Some(expr.0);
                                } else {
                                    v = None;
                                    break;
                                }
                            } else {
                                v = None;
                                break;
                            }
                        }

                        if let Some(expr) = v {
                            if *loc != Loc::Codegen {
                                ns.var_constants.insert(*loc, expr.clone());
                            }
                            return (expr, true);
                        }
                    }
                }
            }

            (expr.clone(), false)
        }
        Expression::Builtin(loc, tys, Builtin::Keccak256, args) => {
            let arg = expression(&args[0], vars, cfg, ns, instr_origin);

            if let Expression::AllocDynamicArray(_, _, _, Some(bs)) = arg.0 {
                let mut hasher = Keccak::v256();
                hasher.update(&bs);
                let mut hash = [0u8; 32];
                hasher.finalize(&mut hash);

                (
                    Expression::BytesLiteral(*loc, tys[0].clone(), hash.to_vec()),
                    true,
                )
            } else {
                (
                    Expression::Builtin(*loc, tys.clone(), Builtin::Keccak256, vec![arg.0]),
                    false,
                )
            }
        }
        Expression::Builtin(loc, tys, Builtin::Ripemd160, args) => {
            let arg = expression(&args[0], vars, cfg, ns, instr_origin);

            if let Expression::AllocDynamicArray(_, _, _, Some(bs)) = arg.0 {
                let mut hasher = Ripemd160::new();
                hasher.update(&bs);
                let result = hasher.finalize();

                (
                    Expression::BytesLiteral(*loc, tys[0].clone(), result[..].to_vec()),
                    true,
                )
            } else {
                (
                    Expression::Builtin(*loc, tys.clone(), Builtin::Ripemd160, vec![arg.0]),
                    false,
                )
            }
        }
        Expression::Builtin(loc, tys, Builtin::Blake2_256, args) => {
            let arg = expression(&args[0], vars, cfg, ns, instr_origin);

            if let Expression::AllocDynamicArray(_, _, _, Some(bs)) = arg.0 {
                let hash = blake2_rfc::blake2b::blake2b(32, &[], &bs);

                (
                    Expression::BytesLiteral(*loc, tys[0].clone(), hash.as_bytes().to_vec()),
                    true,
                )
            } else {
                (
                    Expression::Builtin(*loc, tys.clone(), Builtin::Blake2_256, vec![arg.0]),
                    false,
                )
            }
        }
        Expression::Builtin(loc, tys, Builtin::Blake2_128, args) => {
            let arg = expression(&args[0], vars, cfg, ns, instr_origin);

            if let Expression::AllocDynamicArray(_, _, _, Some(bs)) = arg.0 {
                let hash = blake2_rfc::blake2b::blake2b(16, &[], &bs);

                (
                    Expression::BytesLiteral(*loc, tys[0].clone(), hash.as_bytes().to_vec()),
                    true,
                )
            } else {
                (
                    Expression::Builtin(*loc, tys.clone(), Builtin::Blake2_128, vec![arg.0]),
                    false,
                )
            }
        }
        Expression::Builtin(loc, tys, Builtin::Sha256, args) => {
            let arg = expression(&args[0], vars, cfg, ns, instr_origin);

            if let Expression::AllocDynamicArray(_, _, _, Some(bs)) = arg.0 {
                let mut hasher = Sha256::new();

                // write input message
                hasher.update(&bs);

                // read hash digest and consume hasher
                let result = hasher.finalize();

                (
                    Expression::BytesLiteral(*loc, tys[0].clone(), result[..].to_vec()),
                    true,
                )
            } else {
                (
                    Expression::Builtin(*loc, tys.clone(), Builtin::Sha256, vec![arg.0]),
                    false,
                )
            }
        }
        Expression::Keccak256(loc, ty, args) => {
            let mut all_constant = true;
            let mut hasher = Keccak::v256();

            let args = args
                .iter()
                .map(|expr| {
                    let (expr, _) = expression(expr, vars, cfg, ns, instr_origin);

                    if all_constant {
                        match &expr {
                            Expression::AllocDynamicArray(_, _, _, Some(bs))
                            | Expression::BytesLiteral(_, _, bs) => {
                                hasher.update(bs);
                            }
                            Expression::NumberLiteral(_, ty, n) => {
                                let (sign, mut bs) = n.to_bytes_le();

                                match ty {
                                    Type::Uint(bits) => bs.resize(*bits as usize / 8, 0),
                                    Type::Int(bits) => {
                                        let v = if sign == Sign::Minus { 0xffu8 } else { 0 };

                                        bs.resize(*bits as usize / 8, v);
                                    }
                                    Type::Bytes(n) => {
                                        while (*n as usize) < bs.len() {
                                            bs.insert(0, 0);
                                        }
                                    }
                                    Type::Address(_) => {
                                        bs.resize(ns.address_length, 0);
                                    }
                                    _ => unreachable!(),
                                }

                                hasher.update(&bs);
                            }
                            _ => {
                                all_constant = false;
                            }
                        }
                    }

                    expr
                })
                .collect();

            if all_constant {
                let mut hash = [0u8; 32];
                hasher.finalize(&mut hash);
                let mut hash = hash.to_vec();
                hash.reverse();

                (Expression::BytesLiteral(*loc, ty.clone(), hash), true)
            } else {
                (Expression::Keccak256(*loc, ty.clone(), args), false)
            }
        }
        // The rest is simply for recursing; no constant expansion should be done
        Expression::StructLiteral(loc, ty, args) => {
            let args = args
                .iter()
                .map(|expr| expression(expr, vars, cfg, ns, instr_origin).0)
                .collect();

            (Expression::StructLiteral(*loc, ty.clone(), args), false)
        }
        Expression::ArrayLiteral(loc, ty, lengths, args) => {
            let args = args
                .iter()
                .map(|expr| expression(expr, vars, cfg, ns, instr_origin).0)
                .collect();

            (
                Expression::ArrayLiteral(*loc, ty.clone(), lengths.clone(), args),
                false,
            )
        }
        Expression::ConstArrayLiteral(loc, ty, lengths, args) => {
            let args = args
                .iter()
                .map(|expr| expression(expr, vars, cfg, ns, instr_origin).0)
                .collect();

            (
                Expression::ConstArrayLiteral(*loc, ty.clone(), lengths.clone(), args),
                false,
            )
        }
        Expression::Load(loc, ty, expr) => {
            let (expr, _) = expression(expr, vars, cfg, ns, instr_origin);

            (Expression::Load(*loc, ty.clone(), Box::new(expr)), false)
        }
        Expression::Cast(loc, ty, expr) => {
            let (expr, _) = expression(expr, vars, cfg, ns, instr_origin);

            (Expression::Cast(*loc, ty.clone(), Box::new(expr)), false)
        }
        Expression::BytesCast(loc, from, to, expr) => {
            let (expr, _) = expression(expr, vars, cfg, ns, instr_origin);

            (
                Expression::BytesCast(*loc, from.clone(), to.clone(), Box::new(expr)),
                false,
            )
        }
        Expression::UnsignedMore(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::UnsignedMore(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::SignedMore(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::SignedMore(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::SignedLess(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::SignedLess(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::UnsignedLess(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::UnsignedLess(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::MoreEqual(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::MoreEqual(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::LessEqual(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::LessEqual(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::Equal(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::Equal(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::NotEqual(loc, left, right) => {
            let left = expression(left, vars, cfg, ns, instr_origin);
            let right = expression(right, vars, cfg, ns, instr_origin);

            (
                Expression::NotEqual(*loc, Box::new(left.0), Box::new(right.0)),
                false,
            )
        }
        Expression::Not(loc, expr) => {
            let expr = expression(expr, vars, cfg, ns, instr_origin);

            (Expression::Not(*loc, Box::new(expr.0)), expr.1)
        }
        Expression::Subscript(loc, ty, array_ty, array, index) => {
            let array = expression(array, vars, cfg, ns, instr_origin);
            let index = expression(index, vars, cfg, ns, instr_origin);

            (
                Expression::Subscript(
                    *loc,
                    ty.clone(),
                    array_ty.clone(),
                    Box::new(array.0),
                    Box::new(index.0),
                ),
                false,
            )
        }
        Expression::StructMember(loc, ty, strct, member) => {
            let strct = expression(strct, vars, cfg, ns, instr_origin);

            (
                Expression::StructMember(*loc, ty.clone(), Box::new(strct.0), *member),
                false,
            )
        }

        Expression::StorageArrayLength {
            loc,
            ty,
            array,
            elem_ty,
        } => {
            let array = expression(array, vars, cfg, ns, instr_origin);

            (
                Expression::StorageArrayLength {
                    loc: *loc,
                    ty: ty.clone(),
                    array: Box::new(array.0),
                    elem_ty: elem_ty.clone(),
                },
                false,
            )
        }
        Expression::StringCompare(loc, left, right) => {
            if let (StringLocation::CompileTime(left), StringLocation::CompileTime(right)) =
                (left, right)
            {
                (Expression::BoolLiteral(*loc, left == right), true)
            } else {
                let left = if let StringLocation::RunTime(left) = left {
                    StringLocation::RunTime(Box::new(
                        expression(left, vars, cfg, ns, instr_origin).0,
                    ))
                } else {
                    left.clone()
                };

                let right = if let StringLocation::RunTime(right) = right {
                    StringLocation::RunTime(Box::new(
                        expression(right, vars, cfg, ns, instr_origin).0,
                    ))
                } else {
                    right.clone()
                };

                (Expression::StringCompare(*loc, left, right), false)
            }
        }
        Expression::StringConcat(loc, ty, left, right) => {
            if let (StringLocation::CompileTime(left), StringLocation::CompileTime(right)) =
                (left, right)
            {
                let mut bs = Vec::with_capacity(left.len() + right.len());

                bs.extend(left);
                bs.extend(right);

                (Expression::BytesLiteral(*loc, ty.clone(), bs), true)
            } else {
                let left = if let StringLocation::RunTime(left) = left {
                    StringLocation::RunTime(Box::new(
                        expression(left, vars, cfg, ns, instr_origin).0,
                    ))
                } else {
                    left.clone()
                };

                let right = if let StringLocation::RunTime(right) = right {
                    StringLocation::RunTime(Box::new(
                        expression(right, vars, cfg, ns, instr_origin).0,
                    ))
                } else {
                    right.clone()
                };

                (
                    Expression::StringConcat(*loc, ty.clone(), left, right),
                    false,
                )
            }
        }
        Expression::Builtin(loc, tys, builtin, args) => {
            let args = args
                .iter()
                .map(|expr| expression(expr, vars, cfg, ns, instr_origin).0)
                .collect();

            (
                Expression::Builtin(*loc, tys.clone(), *builtin, args),
                false,
            )
        }
        Expression::AbiEncode {
            loc,
            tys,
            packed,
            args,
        } => {
            let packed = packed
                .iter()
                .map(|expr| expression(expr, vars, cfg, ns, instr_origin).0)
                .collect();
            let args = args
                .iter()
                .map(|expr| expression(expr, vars, cfg, ns, instr_origin).0)
                .collect();

            (
                Expression::AbiEncode {
                    loc: *loc,
                    tys: tys.clone(),
                    packed,
                    args,
                },
                false,
            )
        }
        Expression::NumberLiteral(..)
        | Expression::RationalNumberLiteral(..)
        | Expression::BoolLiteral(..)
        | Expression::BytesLiteral(..)
        | Expression::CodeLiteral(..)
        | Expression::FunctionArg(..) => (expr.clone(), true),
        Expression::AllocDynamicArray(..)
        | Expression::ReturnData(_)
        | Expression::Undefined(_)
        | Expression::FormatString { .. }
        | Expression::GetRef(..)
        | Expression::InternalFunctionCfg(_) => (expr.clone(), false),
        // nothing else is permitted in cfg
        _ => panic!("expr should not be in cfg: {:?}", expr),
    }
}

fn bigint_to_expression(instr_origin: InstrOrigin, loc: &Loc, ty: &Type, n: BigInt, ns: &mut Namespace) -> (Expression, bool, BigInt) {
    println! {"FINAL : {:?} ", n};
    if instr_origin == InstrOrigin::Solidity {
    overflow_check(ns, n.clone(), ty.clone(), *loc);
    }
    let n = match ty {
        Type::Uint(bits) => {
            if n.bits() > *bits as u64 {
                let (_, mut bs) = n.to_bytes_le();
                bs.truncate(*bits as usize / 8);

                BigInt::from_bytes_le(Sign::Plus, &bs)
            } else {
                n
            }
        }
        Type::Int(bits) => {
            if n.bits() > *bits as u64 {
                let mut bs = n.to_signed_bytes_le();
                bs.truncate(*bits as usize / 8);

                BigInt::from_signed_bytes_le(&bs)
            } else {
                n
            }
        }
        Type::StorageRef(..) => n,
        _ => unreachable!(),
    };

    (Expression::NumberLiteral(*loc, ty.clone(), n), true, n)
}

fn get_definition<'a>(
    def: &reaching_definitions::Def,
    cfg: &'a ControlFlowGraph,
) -> Option<&'a Expression> {
    if let Instr::Set { expr, .. } = &cfg.blocks[def.block_no].instr[def.instr_no].1 {
        Some(expr)
    } else {
        None
    }
}

/// Are these two expressions the same constant-folded value?
fn constants_equal(left: &Expression, right: &Expression) -> bool {
    match left {
        Expression::NumberLiteral(_, _, left) => match right {
            Expression::NumberLiteral(_, _, right) => left == right,
            _ => false,
        },
        Expression::BytesLiteral(_, _, left)
        | Expression::AllocDynamicArray(_, _, _, Some(left)) => match right {
            Expression::BytesLiteral(_, _, right)
            | Expression::AllocDynamicArray(_, _, _, Some(right)) => left == right,
            _ => false,
        },
        _ => false,
    }
}

fn  overflow_check(ns: &mut Namespace, result: BigInt, ty: Type, loc: Loc) {
    if let Type::Uint(bits) = ty {
        // If the result sign is minus, throw an error.
        if let Sign::Minus = result.sign() {
            ns.diagnostics.push(Diagnostic::error(
                loc,
            format!( "Type int_const {:?} is not implicitly convertible to expected type {:?}. Cannot implicitly convert signed literal to unsigned type.",result,ty),
            ));
        }

        // If bits of the result is more than bits of the type, throw and error.
        if result.bits() > bits as u64 {
            ns.diagnostics.push(Diagnostic::error(
                loc,
                format!("Type int_const {:?} is not implicitly convertible to expected type {:?}. Literal is too large to fit in {:?}.",result,ty,ty),
            ));
        }
    }

    if let Type::Int(bits) = ty {
        // If number of bits is more than what the type can hold. BigInt.bits() is not used here since it disregards the sign.
        if result.to_signed_bytes_be().len() * 8 > (bits as usize) {
            ns.diagnostics.push(Diagnostic::error(
                loc,
                format!("Type int_const {:?} is not implicitly convertible to expected type {:?}. Literal is too large to fit in {:?}.",result,ty,ty),
            ));
        }
    }
}
