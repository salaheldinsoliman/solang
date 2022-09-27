// SPDX-License-Identifier: Apache-2.0

use num_bigint::BigInt;
use num_bigint::Sign;
use num_rational::BigRational;
use num_traits::One;
use num_traits::ToPrimitive;
use num_traits::Zero;

use super::Recurse;
use super::ast::{Diagnostic, Expression, Namespace};
use solang_parser::pt;
use solang_parser::pt::CodeLocation;
use std::ops::{Add, Mul, Shl, Shr, Sub};

use crate::sema::ast::RetrieveType;
use crate::sema::ast::Type;
use solang_parser::pt::Loc;

/// Resolve an expression where a compile-time constant is expected
pub fn eval_const_number(
    expr: &Expression,
    ns: &Namespace,
) -> Result<(pt::Loc, BigInt), Diagnostic> {
    match expr {
        Expression::Add(loc, _, _, l, r) => Ok((
            *loc,
            eval_const_number(l, ns)?.1 + eval_const_number(r, ns)?.1,
        )),
        Expression::Subtract(loc, _, _, l, r) => Ok((
            *loc,
            eval_const_number(l, ns)?.1 - eval_const_number(r, ns)?.1,
        )),
        Expression::Multiply(loc, _, _, l, r) => Ok((
            *loc,
            eval_const_number(l, ns)?.1 * eval_const_number(r, ns)?.1,
        )),
        Expression::Divide(loc, _, l, r) => {
            let divisor = eval_const_number(r, ns)?.1;

            if divisor.is_zero() {
                Err(Diagnostic::error(*loc, "divide by zero".to_string()))
            } else {
                Ok((*loc, eval_const_number(l, ns)?.1 / divisor))
            }
        }
        Expression::Modulo(loc, _, l, r) => {
            let divisor = eval_const_number(r, ns)?.1;

            if divisor.is_zero() {
                Err(Diagnostic::error(*loc, "divide by zero".to_string()))
            } else {
                Ok((*loc, eval_const_number(l, ns)?.1 % divisor))
            }
        }
        Expression::BitwiseAnd(loc, _, l, r) => Ok((
            *loc,
            eval_const_number(l, ns)?.1 & eval_const_number(r, ns)?.1,
        )),
        Expression::BitwiseOr(loc, _, l, r) => Ok((
            *loc,
            eval_const_number(l, ns)?.1 | eval_const_number(r, ns)?.1,
        )),
        Expression::BitwiseXor(loc, _, l, r) => Ok((
            *loc,
            eval_const_number(l, ns)?.1 ^ eval_const_number(r, ns)?.1,
        )),
        Expression::Power(loc, _, _, base, exp) => {
            let b = eval_const_number(base, ns)?.1;
            let mut e = eval_const_number(exp, ns)?.1;

            if e.sign() == Sign::Minus {
                Err(Diagnostic::error(
                    expr.loc(),
                    "power cannot take negative number as exponent".to_string(),
                ))
            } else if e.sign() == Sign::NoSign {
                Ok((*loc, BigInt::one()))
            } else {
                let mut res = b.clone();
                e -= BigInt::one();
                while e.sign() == Sign::Plus {
                    res *= b.clone();
                    e -= BigInt::one();
                }
                Ok((*loc, res))
            }
        }
        Expression::ShiftLeft(loc, _, left, right) => {
            let l = eval_const_number(left, ns)?.1;
            let r = eval_const_number(right, ns)?.1;
            let r = match r.to_usize() {
                Some(r) => r,
                None => {
                    return Err(Diagnostic::error(
                        expr.loc(),
                        format!("cannot left shift by {}", r),
                    ));
                }
            };
            Ok((*loc, l << r))
        }
        Expression::ShiftRight(loc, _, left, right, _) => {
            let l = eval_const_number(left, ns)?.1;
            let r = eval_const_number(right, ns)?.1;
            let r = match r.to_usize() {
                Some(r) => r,
                None => {
                    return Err(Diagnostic::error(
                        expr.loc(),
                        format!("cannot right shift by {}", r),
                    ));
                }
            };
            Ok((*loc, l >> r))
        }
        Expression::NumberLiteral(loc, _, n) => Ok((*loc, n.clone())),
        Expression::ZeroExt(loc, _, n) => Ok((*loc, eval_const_number(n, ns)?.1)),
        Expression::SignExt(loc, _, n) => Ok((*loc, eval_const_number(n, ns)?.1)),
        Expression::Cast(loc, _, n) => Ok((*loc, eval_const_number(n, ns)?.1)),
        Expression::Not(loc, n) => Ok((*loc, !eval_const_number(n, ns)?.1)),
        Expression::Complement(loc, _, n) => Ok((*loc, !eval_const_number(n, ns)?.1)),
        Expression::UnaryMinus(loc, _, n) => Ok((*loc, -eval_const_number(n, ns)?.1)),
        Expression::ConstantVariable(_, _, Some(contract_no), var_no) => {
            let expr = ns.contracts[*contract_no].variables[*var_no]
                .initializer
                .as_ref()
                .unwrap()
                .clone();

            eval_const_number(&expr, ns)
        }
        Expression::ConstantVariable(_, _, None, var_no) => {
            let expr = ns.constants[*var_no].initializer.as_ref().unwrap().clone();

            eval_const_number(&expr, ns)
        }
        _ => Err(Diagnostic::error(
            expr.loc(),
            "expression not allowed in constant number expression".to_string(),
        )),
    }
}

/// Resolve an expression where a compile-time constant(rational) is expected
pub fn eval_const_rational(
    expr: &Expression,
    ns: &Namespace,
) -> Result<(pt::Loc, BigRational), Diagnostic> {
    match expr {
        Expression::Add(loc, _, _, l, r) => Ok((
            *loc,
            eval_const_rational(l, ns)?.1 + eval_const_rational(r, ns)?.1,
        )),
        Expression::Subtract(loc, _, _, l, r) => Ok((
            *loc,
            eval_const_rational(l, ns)?.1 - eval_const_rational(r, ns)?.1,
        )),
        Expression::Multiply(loc, _, _, l, r) => Ok((
            *loc,
            eval_const_rational(l, ns)?.1 * eval_const_rational(r, ns)?.1,
        )),
        Expression::Divide(loc, _, l, r) => {
            let divisor = eval_const_rational(r, ns)?.1;

            if divisor.is_zero() {
                Err(Diagnostic::error(*loc, "divide by zero".to_string()))
            } else {
                Ok((*loc, eval_const_rational(l, ns)?.1 / divisor))
            }
        }
        Expression::Modulo(loc, _, l, r) => {
            let divisor = eval_const_rational(r, ns)?.1;

            if divisor.is_zero() {
                Err(Diagnostic::error(*loc, "divide by zero".to_string()))
            } else {
                Ok((*loc, eval_const_rational(l, ns)?.1 % divisor))
            }
        }
        Expression::NumberLiteral(loc, _, n) => Ok((*loc, BigRational::from_integer(n.clone()))),
        Expression::RationalNumberLiteral(loc, _, n) => Ok((*loc, n.clone())),
        Expression::Cast(loc, _, n) => Ok((*loc, eval_const_rational(n, ns)?.1)),
        Expression::UnaryMinus(loc, _, n) => Ok((*loc, -eval_const_rational(n, ns)?.1)),
        Expression::ConstantVariable(_, _, Some(contract_no), var_no) => {
            let expr = ns.contracts[*contract_no].variables[*var_no]
                .initializer
                .as_ref()
                .unwrap()
                .clone();

            eval_const_rational(&expr, ns)
        }
        Expression::ConstantVariable(_, _, None, var_no) => {
            let expr = ns.constants[*var_no].initializer.as_ref().unwrap().clone();

            eval_const_rational(&expr, ns)
        }
        _ => Err(Diagnostic::error(
            expr.loc(),
            "expression not allowed in constant rational number expression".to_string(),
        )),
    }
}



 
fn auxiliar(expr: &Expression, recursed_expressions: & mut Vec<Expression>) -> bool {

//println!("EXPR {:?}", expr );
match  expr{

    Expression::Add( ..)
    | Expression::Subtract(_, ..)
    | Expression::Multiply(_, ..)
    | Expression::Divide(..)
    | Expression::Modulo(..)
    | Expression::Power(_, ..)
    | Expression::BitwiseOr(..)
    | Expression::BitwiseAnd(..)
    | Expression::BitwiseXor(..)
    | Expression::ShiftLeft(..)
    | Expression::ShiftRight(..) => {
        
        recursed_expressions.push(expr.clone());

        return  false;
    }
    _ => {}
}


return true;

}



fn eval_constants_in_expression(
    expr: &Expression,
    ns: &mut Namespace,
    results: &mut Vec<BigInt>,
    recursed: &mut Vec<Expression>
) -> Expression {
    match expr {
        Expression::Add(loc, ty, unchecked, left, right) => {
            let left = eval_constants_in_expression(left, ns, results, recursed);
            let right = eval_constants_in_expression(right, ns, results, recursed);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left, &right)
            {
                results.push(left.add(right));
                Expression::NumberLiteral(*loc, ty.clone(), left.add(right))
            } else {
                Expression::Add(
                    *loc,
                    ty.clone(),
                    *unchecked,
                    Box::new(left),
                    Box::new(right),
                )
            }
        }
        Expression::Subtract(loc, ty, unchecked, left, right) => {
            let left = eval_constants_in_expression(left, ns, results, recursed);
            let right = eval_constants_in_expression(right, ns, results, recursed);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left, &right)
            {
                results.push(left.sub(right));
                Expression::NumberLiteral(*loc, ty.clone(), left.sub(right))
            } else {
                Expression::Subtract(
                    *loc,
                    ty.clone(),
                    *unchecked,
                    Box::new(left),
                    Box::new(right),
                )
            }
        }

        Expression::Multiply(loc, ty, unchecked, left, right) => {
            let left = eval_constants_in_expression(left, ns, results, recursed);
            let right = eval_constants_in_expression(right, ns, results, recursed);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left, &right)
            {
                results.push(left.mul(right.to_u32().unwrap()));
                Expression::NumberLiteral(*loc, ty.clone(), left.mul(right.to_u32().unwrap()))
            } else {
                Expression::Multiply(
                    *loc,
                    ty.clone(),
                    *unchecked,
                    Box::new(left),
                    Box::new(right),
                )
            }
        }

        Expression::Power(loc, ty, unchecked, left, right) => {
            let left = eval_constants_in_expression(left, ns, results, recursed);
            let right = eval_constants_in_expression(right, ns, results, recursed);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left, &right)
            {
                results.push(left.pow(right.to_u32().unwrap()));
                Expression::NumberLiteral(*loc, ty.clone(), left.pow(right.to_u32().unwrap()))
            } else {
                Expression::Power(
                    *loc,
                    ty.clone(),
                    *unchecked,
                    Box::new(left),
                    Box::new(right),
                )
            }
        }

        Expression::ShiftLeft(loc, ty, left, right) => {
            let left = eval_constants_in_expression(left, ns, results, recursed);
            let right = eval_constants_in_expression(right, ns, results, recursed);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left, &right)
            {
                results.push(left.shl(right.to_u32().unwrap()));
                Expression::NumberLiteral(*loc, ty.clone(), left.shl(right.to_u32().unwrap()))
            } else {
                Expression::ShiftLeft(*loc, ty.clone(), Box::new(left), Box::new(right))
            }
        }

        Expression::ShiftRight(loc, ty, left, right, _) => {
            let left = eval_constants_in_expression(left, ns, results, recursed);
            let right = eval_constants_in_expression(right, ns, results, recursed);

            if let (Expression::NumberLiteral(_, _, left), Expression::NumberLiteral(_, _, right)) =
                (&left, &right)
            {
                results.push(left.shr(right.to_u32().unwrap()));
                Expression::NumberLiteral(*loc, ty.clone(), left.shr(right.to_u32().unwrap()))
            } else {
                Expression::ShiftLeft(*loc, ty.clone(), Box::new(left), Box::new(right))
            }
        }
        Expression::NumberLiteral(_, _, n) => {
            results.push(n.clone());
            expr.clone()
        }

        _ => {
       
        expr.recurse( recursed, auxiliar);
            
            
        
            expr.clone()},
    }
}

fn overflow_check(ns: &mut Namespace, result: BigInt, ty: Type, loc: Loc) {
    if let Type::Uint(bits) = ty {
        // If the result sign is minus, throw an error.
        if let Sign::Minus = result.sign() {
            ns.diagnostics.push(Diagnostic::error(
                loc,
            format!( "negative value {} does not fit into type {}. Cannot implicitly convert signed literal to unsigned type.",result,ty.to_string(ns)),
            ));
        }

        // If bits of the result is more than bits of the type, throw and error.
        if result.bits() > bits as u64 {
            ns.diagnostics.push(Diagnostic::error(
                loc,
                format!(
                    "value {} does not fit into type {}.",
                    result,
                    ty.to_string(ns)
                ),
            ));
        }
    }

    if let Type::Int(bits) = ty {
        // If number of bits is more than what the type can hold. BigInt.bits() is not used here since it disregards the sign.
        if result.to_signed_bytes_be().len() * 8 > (bits as usize) {
            ns.diagnostics.push(Diagnostic::error(
                loc,
                format!(
                    "value {} does not fit into type {}.",
                    result,
                    ty.to_string(ns)
                ),
            ));
        }
    }
}

pub fn verify_result(expr: &Expression, ns: &mut Namespace, loc: &Loc) {
    let results: &mut Vec<BigInt> = &mut Vec::new();
    let recursed: &mut Vec<Expression> = &mut Vec::new();
    let _ = eval_constants_in_expression(expr, ns, results, recursed);
    if !recursed.is_empty() {
        
        for  iter in recursed {
            verify_result(iter, ns, &iter.loc());
        }

    }
    if results.last().is_some() {
        overflow_check(ns, results.last().unwrap().clone(), expr.ty(), *loc);
    }
}
