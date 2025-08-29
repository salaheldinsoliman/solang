// SPDX-License-Identifier: Apache-2.0

use crate::codegen::{cfg::ControlFlowGraph, HostFunctions, Options};

use crate::emit::cfg::emit_cfg;
use crate::{emit::Binary, sema::ast};
use funty::Fundamental;
use inkwell::{
    context::Context,
    module::{Linkage, Module},
    types::FunctionType,
};


pub struct Midentarget;

impl Midentarget {

    pub fn build<'a>(
        context: &'a Context,
        std_lib: &Module<'a>,
        contract: &'a ast::Contract,
        ns: &'a ast::Namespace,
        opt: &'a Options,
        contract_no: usize,
    ) -> Binary<'a> {
        let filename = ns.files[contract.loc.file_no()].file_name();
        let mut bin = Binary::new(
            context,
            ns,
            &contract.id.name,
            &filename,
            opt,
            std_lib,
            None,
        );

        println!("Generating Miden code...");

        //println!("{:?} functions", contract.cfg);

        let cfg = &contract.cfg[0];

        println!("cfg: {:#?}", cfg.blocks[0].instr);


        for instr in  contract.cfg[0].blocks[0].instr.iter() {
            let instr_string = format!("{:?}", instr);
            bin.miden_instrs.as_ref().unwrap().borrow_mut().push(instr_string);
        }
        // let mut export_list = Vec::new();
        // Self::declare_externals(&mut bin);
        // Self::emit_functions_with_spec(contract, &mut bin, context, contract_no, &mut export_list);
        // bin.internalize(export_list.as_slice());

        // //Self::emit_initializer(&mut binary, ns, contract.constructors(ns).first());

        // Self::emit_env_meta_entries(context, &mut bin, opt);

        bin
    }



}