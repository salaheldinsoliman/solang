use super::vartable::Vartable;
use crate::codegen::cfg::{ControlFlowGraph, Instr};
use crate::codegen::Expression;
use crate::sema::ast::Type;
use indexmap::IndexMap;
use num_bigint::{BigInt, Sign};
use solang_parser::pt::Loc;

// IndexMap <ArrayVariable res , (res of temp variable, expression of temp variable)>
pub type ArrayTempVars = IndexMap<usize, (usize, Expression)>;

//function to modify arraye length temp
pub(crate) fn modify_temp_array_size(
    cfg: &mut ControlFlowGraph,
    minus: bool,        //if the function is called from pushMemory or popMemory
    address_arr: usize, //the res of array that push/pop is performed on
    vartab: &mut Vartable,
) {
    //get the res and the expression of the of the array
    let temp_var_index = cfg.array_lengths_temps.get(&address_arr);

    //if not empty
    if let Some(..) = temp_var_index {
        let to_add = &*temp_var_index.unwrap();

        let add_expr = if minus {
            Expression::Subtract(
                Loc::Codegen,
                Type::Uint(32),
                false,
                //Box::new(to_add.1.clone()),
                Box::new(Expression::Variable(Loc::Codegen, Type::Uint(32), to_add.0)),
                Box::new(Expression::NumberLiteral(
                    Loc::Codegen,
                    Type::Uint(32),
                    BigInt::new(Sign::Plus, vec![1]),
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
                    BigInt::new(Sign::Plus, vec![1]),
                )),
            )
        };

        //set instruction to add to the cfg
        let set_ins = Instr::Set {
            loc: Loc::Codegen,
            res: to_add.0,
            expr: add_expr,
        };

        //add instruction to the cfg
        cfg.add(vartab, set_ins);
    }
}
