// SPDX-License-Identifier: Apache-2.0

use crate::codegen::Expression;
use crate::sema::ast;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use num_traits::One;
use num_traits::Zero;

use super::Options;
use super::{cfg::ControlFlowGraph, target::target_codegen, vartable::Vartable};
use crate::sema::ast::{Function, Namespace, Type};
use solang_parser::pt;

/// Given a storage slot which is the start of the array, calculate the
/// offset of the array element. This function exists to avoid doing
/// 256 bit multiply if possible.
pub fn array_offset(
    loc: &pt::Loc,
    start: Expression,
    index: Expression,
    elem_ty: Type,
    ns: &Namespace,
) -> Expression {
    let elem_size = elem_ty.storage_slots(ns);
    let slot_ty = ns.storage_type();

    // the index needs to be cast to i256 and multiplied by the number
    // of slots for each element
    if elem_size == BigInt::one() {
        Expression::Add {
            loc: *loc,
            ty: slot_ty,
            overflowing: true,
            left: Box::new(start),
            right: Box::new(index),
        }
    } else if (elem_size.clone() & (elem_size.clone() - BigInt::one())) == BigInt::zero() {
        // elem_size is power of 2
        Expression::Add {
            loc: *loc,
            ty: slot_ty.clone(),
            overflowing: true,
            left: Box::new(start),
            right: Box::new(Expression::ShiftLeft {
                loc: *loc,
                ty: slot_ty.clone(),
                left: Box::new(index),
                right: Box::new(Expression::NumberLiteral {
                    loc: *loc,
                    ty: slot_ty,
                    value: BigInt::from_u64(elem_size.bits() - 1).unwrap(),
                }),
            }),
        }
    } else {
        Expression::Add {
            loc: *loc,
            ty: slot_ty.clone(),
            overflowing: true,
            left: Box::new(start),
            right: Box::new(Expression::Multiply {
                loc: *loc,
                ty: slot_ty.clone(),
                overflowing: true,
                left: Box::new(index),
                right: Box::new(Expression::NumberLiteral {
                    loc: *loc,
                    ty: slot_ty,
                    value: elem_size,
                }),
            }),
        }
    }
}

/// Push() method on dynamic array in storage
pub fn storage_slots_array_push(
    loc: &pt::Loc,
    args: &[ast::Expression],
    cfg: &mut ControlFlowGraph,
    contract_no: usize,
    func: Option<&Function>,
    ns: &Namespace,
    vartab: &mut Vartable,
    opt: &Options,
) -> Expression {
    target_codegen(ns).storage_slots_array_push(loc, args, cfg, contract_no, func, ns, vartab, opt)
}

/// Pop() method on dynamic array in storage
pub fn storage_slots_array_pop(
    loc: &pt::Loc,
    args: &[ast::Expression],
    return_ty: &Type,
    cfg: &mut ControlFlowGraph,
    contract_no: usize,
    func: Option<&Function>,
    ns: &Namespace,
    vartab: &mut Vartable,
    opt: &Options,
) -> Expression {
    target_codegen(ns).storage_slots_array_pop(
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

/// Push() method on array or bytes in storage
pub fn array_push(
    loc: &pt::Loc,
    args: &[ast::Expression],
    cfg: &mut ControlFlowGraph,
    contract_no: usize,
    func: Option<&Function>,
    ns: &Namespace,
    vartab: &mut Vartable,
    opt: &Options,
) -> Expression {
    target_codegen(ns).array_push(loc, args, cfg, contract_no, func, ns, vartab, opt)
}

/// Pop() method on array or bytes in storage
pub fn array_pop(
    loc: &pt::Loc,
    args: &[ast::Expression],
    return_ty: &Type,
    cfg: &mut ControlFlowGraph,
    contract_no: usize,
    func: Option<&Function>,
    ns: &Namespace,
    vartab: &mut Vartable,
    opt: &Options,
) -> Expression {
    target_codegen(ns).array_pop(
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
