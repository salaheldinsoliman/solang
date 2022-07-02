use super::vartable::Vartable;
use crate::ast;
use crate::codegen::cfg::{ControlFlowGraph, Instr};
use crate::codegen::Expression;
use crate::sema::ast::Type;
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::One;
use solang_parser::pt::Loc;

// IndexMap <ArrayVariable res , res of temp variable>
pub type ArrayTempVars = IndexMap<usize, usize>;

// Function to modify arraye length temp
pub(crate) fn modify_temp_array_size(
    cfg: &mut ControlFlowGraph,
    minus: bool,        // If the function is called from pushMemory or popMemory
    address_arr: usize, // The res of array that push/pop is performed on
    vartab: &mut Vartable,
) {
    // If not empty
    if let Some(to_add) = cfg.array_lengths_temps.clone().get(&address_arr) {
        let add_expr = if minus {
            Expression::Subtract(
                Loc::Codegen,
                Type::Uint(32),
                false,
                Box::new(Expression::Variable(Loc::Codegen, Type::Uint(32), *to_add)),
                Box::new(Expression::NumberLiteral(
                    Loc::Codegen,
                    Type::Uint(32),
                    BigInt::one(),
                )),
            )
        } else {
            Expression::Add(
                Loc::Codegen,
                Type::Uint(32),
                false,
                Box::new(Expression::Variable(Loc::Codegen, Type::Uint(32), *to_add)),
                Box::new(Expression::NumberLiteral(
                    Loc::Codegen,
                    Type::Uint(32),
                    BigInt::one(),
                )),
            )
        };

        // Add instruction to the cfg
        cfg.add(
            vartab,
            Instr::Set {
                loc: Loc::Codegen,
                res: *to_add,
                expr: add_expr,
            },
        );
    }
}

pub(crate) fn handle_array_assign(
    right: &ast::Expression,
    cfg: &mut ControlFlowGraph,
    vartab: &mut Vartable,
    pos: &usize,
) {
    if let ast::Expression::Variable(_, _, right_res) = *right {
        // If we have initialized a temp var for this var
        if let Some(to_update) = cfg.array_lengths_temps.clone().get(&right_res) {
            let num = Expression::Variable(Loc::Codegen, Type::Uint(32), *to_update);
            let temp_res = vartab.temp_name("array_size", &Type::Uint(32));
            cfg.add(
                vartab,
                Instr::Set {
                    loc: Loc::Codegen,
                    res: temp_res,
                    expr: num,
                },
            );

            cfg.array_lengths_temps.insert(*pos, temp_res);
        } else {
            // If the right hand side doesn't have a temp, it must be a function parameter.
            cfg.array_lengths_temps.remove(pos);
        }
    } else if let ast::Expression::AllocDynamicArray(_, ty, size, _) = right {
        let a = if cfg.array_lengths_temps.get(pos) != None {
            // If there was a temp variable, reassign it to the new value
            cfg.array_lengths_temps.remove(pos).unwrap()
        } else {
            //create a new temp variable
            vartab.temp_name("array_size", ty)
        };

        if let ast::Expression::Variable(_, _, position) = **size {
            let num = Expression::Variable(Loc::Codegen, Type::Uint(32), position);
            cfg.add(
                vartab,
                Instr::Set {
                    loc: Loc::Codegen,
                    res: a,
                    expr: num,
                },
            );

            cfg.array_lengths_temps.insert(*pos, a);
        }
        // If size a uint and bits > 32
        else if let ast::Expression::Trunc(_, _, ref index) = **size {
            if let ast::Expression::Variable(_, _, index) = &**index {
                // A number var holding array length
                let num = Expression::Variable(Loc::Codegen, Type::Uint(32), *index);
                let num_trunced = Expression::Trunc(Loc::Codegen, Type::Uint(32), Box::new(num));

                cfg.add(
                    vartab,
                    Instr::Set {
                        loc: Loc::Codegen,
                        res: a,
                        expr: num_trunced,
                    },
                );
                cfg.array_lengths_temps.insert(*pos, a);
            }
        } else if let ast::Expression::ZeroExt(_, _, ref index) = **size {
            if let ast::Expression::Variable(_, _, index) = &**index {
                // A number var holding array length
                let num = Expression::Variable(Loc::Codegen, Type::Uint(32), *index);
                let num_trunced = Expression::ZeroExt(Loc::Codegen, Type::Uint(32), Box::new(num));

                cfg.add(
                    vartab,
                    Instr::Set {
                        loc: Loc::Codegen,
                        res: a,
                        expr: num_trunced,
                    },
                );
                cfg.array_lengths_temps.insert(*pos, a);
            }
        }
        // If the size is a number literal
        else if let ast::Expression::NumberLiteral(_, _, size_of_array) = &**size {
            //a number literal holding the array length
            let num =
                Expression::NumberLiteral(Loc::Codegen, Type::Uint(32), size_of_array.clone());
            // let temp_res = vartab.temp_name("array_size", &Type::Uint(32));
            cfg.add(
                vartab,
                Instr::Set {
                    loc: Loc::Codegen,
                    res: a,
                    expr: num,
                },
            );
            cfg.array_lengths_temps.insert(*pos, a);
        }
    }
}
