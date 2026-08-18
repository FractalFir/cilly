use std::{collections::HashMap, num::NonZeroU32};

use crate::{AllocA, Body, Fnc, FuncRef, Label, Local, Locals, Module, Operand, Type};
pub struct FunctionBuilder {
    id: FuncRef,
    fnc: Fnc,
    locals: Locals,
    bbs: HashMap<Label, BasicBlock>,
    last_id: u32,
}
impl FunctionBuilder {
    fn alloc_ssa_id(&mut self) -> u32 {
        let id = self.last_id;
        self.last_id += 1;
        id
    }
    pub fn add_alloca(
        &mut self,
        size: NonZeroU32,
        align: NonZeroU32,
    ) -> Result<Operand, BuilderError> {
        if !align.get().is_power_of_two() {
            Err(BuilderError::AllocaAInvalidAlign { align })?;
        }
        let ssa_id = self.alloc_ssa_id();
        self.locals.allocas.push(AllocA {
            ssa_id,
            size,
            align,
        });
        Ok(Operand::SSA(ssa_id))
    }
    pub fn add_local(&mut self, ty: Type) -> Result<Local, BuilderError> {
        todo!()
    }
    pub fn finish(self, module: &mut Module) {
        let Fnc::Decl {
            src_loc,
            linkage,
            output,
            name,
            inputs,
        } = self.fnc
        else {
            panic!();
        };
        *module.functions.get_mut(self.id.0).unwrap() = Fnc::Def {
            src_loc,
            linkage,
            output,
            name,
            inputs,
            locals: self.locals,
            body: to_body(self.bbs),
        };
    }
}

fn to_body(bbs: HashMap<Label, BasicBlock>) -> Body {
    todo!()
}
struct BasicBlock {}
#[derive(Debug)]
pub enum BuilderError {
    AllocaAInvalidAlign { align: NonZeroU32 },
}
