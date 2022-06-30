use super::vartable::Vartable;
use crate::codegen::cfg::{ControlFlowGraph, Instr};
use crate::codegen::Expression;
use crate::sema::ast::Type;
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::One;
use solang_parser::pt::Loc;

// IndexMap <ArrayVariable res , (res of temp variable, expression of temp variable)>
pub type ArrayTempVars = IndexMap<usize, (usize, Expression)>;

// Function to modify arraye length temp
pub(crate) fn modify_temp_array_size(
    cfg: &mut ControlFlowGraph,
    minus: bool,        // If the function is called from pushMemory or popMemory
    address_arr: usize, // The res of array that push/pop is performed on
    vartab: &mut Vartable,
) {
    // If not empty
    if let Some(to_add) = cfg.clone().array_lengths_temps.get(&address_arr) {
        let add_expr = if minus {
            Expression::Subtract(
                Loc::Codegen,
                Type::Uint(32),
                false,
                Box::new(Expression::Variable(Loc::Codegen, Type::Uint(32), to_add.0)),
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
                Box::new(Expression::Variable(Loc::Codegen, Type::Uint(32), to_add.0)),
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
                res: to_add.0,
                expr: add_expr,
            },
        );
    }
}
