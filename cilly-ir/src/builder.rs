use std::num::{NonZeroU8, NonZeroU32};

use crate::{
    AllocA, Binop, Fnc, FuncRef, InstrList, Instruction, Label, Local, Locals, Module, Operand,
    SSAVal, Type, to_body,
};

pub struct FunctionBuilder {
    pub(crate) id: FuncRef,
    pub(crate) fnc: Fnc,
    pub(crate) locals: Locals,
    pub(crate) bbs: Vec<BasicBlock>,
    pub(crate) ssas: Vec<Type>,
    pub(crate) pos: Option<(Label, usize)>,
}
impl FunctionBuilder {
    pub fn return_type(&self) -> &Type {
        let Fnc::Decl { output, .. } = &self.fnc else {
            panic!()
        };
        &output.ty
    }
    // Bb management
    pub fn new_block(&mut self) -> Label {
        let bid = self.bbs.len();
        self.bbs.push(BasicBlock {
            instrs: InstrList { instrs: vec![] },
            term: None,
        });
        Label { id: bid as u32 }
    }
    pub fn position_at_end(&mut self, block: Label) -> Result<(), BuilderError> {
        let id = block.id as usize;
        let last_pos = self
            .bbs
            .get(id)
            .ok_or(BuilderError::BlockLabelInvalid { block })?
            .instrs
            .instrs
            .len();
        self.pos = Some((block, last_pos));
        Ok(())
    }
    pub fn position_at_pos(&mut self, block: Label, pos: usize) -> Result<(), BuilderError> {
        let id = block.id as usize;
        let last_pos = self
            .bbs
            .get(id)
            .ok_or(BuilderError::BlockLabelInvalid { block })?
            .instrs
            .instrs
            .len();
        if last_pos < pos {
            Err(BuilderError::PosOutOfRange { last_pos, pos })?;
        }
        self.pos = Some((block, last_pos));
        Ok(())
    }
    pub fn position_at_start(&mut self, block: Label) -> Result<(), BuilderError> {
        let id = block.id as usize;
        self.bbs
            .get(id)
            .ok_or(BuilderError::BlockLabelInvalid { block })?;
        self.pos = Some((block, 0));
        Ok(())
    }
    pub fn curr_pos(&mut self) -> Option<(Label, usize)> {
        self.pos
    }
    pub fn curr_label(&mut self) -> Option<Label> {
        self.pos.map(|(l, _)| l)
    }
    // Terminators
    fn build_term(&mut self, val: Termiantor) -> Result<(), BuilderError> {
        let term = &mut self.curr_bb_mut()?.term;
        if term.is_some() {
            let term = term.as_ref().unwrap().clone();
            Err(BuilderError::BlockAlreadyTerminated {
                bb: self.pos.unwrap().0,
                old_term: term,
                new_term: val,
            })
        } else {
            *term = Some(val);
            Ok(())
        }
    }
    pub fn build_br(&mut self, label: Label) -> Result<(), BuilderError> {
        self.check_label(label)?;
        self.build_term(Termiantor::Br(label))
    }
    pub fn build_condbr(
        &mut self,
        cond: Operand,
        then: Label,
        els: Label,
    ) -> Result<(), BuilderError> {
        let i1 = Type::Int {
            bitwidth: NonZeroU8::new(1).unwrap(),
        };
        let got = self.get_type(&cond, &i1)?;
        if *got != i1 {
            return Err(BuilderError::CondBrCondNotI1 {
                cond,
                got: got.clone(),
            });
        }
        self.check_label(then)?;
        self.check_label(els)?;
        self.build_term(Termiantor::BrCond { cond, then, els })
    }
    pub fn build_ret(&mut self, val: Option<Operand>) -> Result<(), BuilderError> {
        match val {
            None => {
                if !self.return_type().is_void() {
                    Err(BuilderError::VoidRetInNonVoidFnc)?;
                }
                self.build_term(Termiantor::VoidRet)?;
            }
            Some(val) => {
                let ret_ty = self.return_type();
                if self.get_type(&val, ret_ty)? != self.return_type() {
                    Err(BuilderError::RetTypeMismatch {
                        got: self.get_type(&val, ret_ty)?.clone(),
                        expected: self.return_type().clone(),
                    })?;
                }
                self.build_term(Termiantor::Ret(val))?;
            }
        }

        Ok(())
    }
    // Helpers
    fn check_label(&self, block: Label) -> Result<(), BuilderError> {
        self.bbs
            .get(block.id as usize)
            .ok_or(BuilderError::BlockLabelInvalid { block })?;
        Ok(())
    }
    pub(crate) fn alloc_ssa_id(&mut self, ty: Type) -> SSAVal {
        let id = self.ssas.len() as u32;
        self.ssas.push(ty);
        SSAVal(id)
    }
    fn curr_bb_mut(&mut self) -> Result<&mut BasicBlock, BuilderError> {
        let (label, _) = self.pos.ok_or(BuilderError::BuilderPosNotSet)?;
        Ok(&mut self.bbs[label.id as usize])
    }
    fn insert_at_pos(&mut self, instr: Instruction) -> Result<(), BuilderError> {
        let (label, pos) = self.pos.ok_or(BuilderError::BuilderPosNotSet)?;
        self.bbs[label.id as usize].instrs.instrs.insert(pos, instr);
        Ok(())
    }
    fn build_binop(
        &mut self,
        op: Binop,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::Binop {
            dst,
            ty,
            lhs,
            rhs,
            op,
        })?;
        Ok(Operand::SSA(dst))
    }
    fn build_ibinop(
        &mut self,
        op: Binop,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int_or_vecint() {
            return Err(BuilderError::IntOpTypeNotIntOrVecInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int_or_vecint() {
            return Err(BuilderError::IntOpOperandNotIntOrVecInt { ty });
        }
        self.build_binop(op, ty, lhs, rhs)
    }
    pub fn get_type<'ty, 's: 'ty>(
        &'s self,
        op: &Operand,
        implicit_ty: &'ty Type,
    ) -> Result<&'ty Type, BuilderError> {
        const PTR_TY: Type = Type::Ptr;
        match op {
            Operand::SSA(ssaval) => {
                self.ssas
                    .get(ssaval.0 as usize)
                    .ok_or(BuilderError::InvalidSassOperand {
                        operand: op.clone(),
                    })
            }
            Operand::Global(_) => Ok(&PTR_TY),
            Operand::Constant(cst) => cst.get_ty(implicit_ty),
        }
    }
    // Binops
    pub fn build_add(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::Add, ty, lhs, rhs)
    }
    pub fn build_sub(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::Sub, ty, lhs, rhs)
    }
    pub fn build_mul(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::Mul, ty, lhs, rhs)
    }
    // Position-less
    pub fn add_alloca(
        &mut self,
        size: NonZeroU32,
        align: NonZeroU32,
    ) -> Result<Operand, BuilderError> {
        if !align.get().is_power_of_two() {
            Err(BuilderError::AllocaAInvalidAlign { align })?;
        }
        let ssa_id = self.alloc_ssa_id(Type::Ptr);
        self.locals.allocas.push(AllocA {
            ssa_id,
            size,
            align,
        });
        Ok(Operand::SSA(ssa_id))
    }
    pub fn get_param(&mut self, param: u32) -> Result<Operand, BuilderError> {
        let Fnc::Decl { inputs, .. } = &self.fnc else {
            todo!()
        };
        if param as usize >= inputs.args.len() {
            return Err(BuilderError::ParamOutOfRange {
                param,
                len: inputs.args.len() as u32,
            });
        }
        Ok(Operand::SSA(SSAVal(param)))
    }
    pub fn add_local(&mut self, ty: Type) -> Result<Local, BuilderError> {
        if ty.is_void() {
            return Err(BuilderError::VoidLocal);
        }
        Ok(self.locals.add_local(ty))
    }
    pub fn finish(mut self, module: &mut Module) {
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
        let body = to_body(self.bbs, &mut self.locals, &mut self.ssas, &output.ty);
        *module.functions.get_mut(self.id.0).unwrap() = Fnc::Def {
            src_loc,
            linkage,
            output,
            name,
            inputs,
            locals: self.locals,
            body,
        };
    }
}

pub(crate) struct BasicBlock {
    pub(crate) instrs: InstrList,
    pub(crate) term: Option<Termiantor>,
}
#[derive(Debug, Clone)]
pub(crate) enum Termiantor {
    VoidRet,
    Ret(Operand),
    Br(Label),
    BrCond {
        cond: Operand,
        then: Label,
        els: Label,
    },
}
impl Termiantor {
    pub fn sucessors(&self) -> Vec<Label> {
        match self {
            Termiantor::VoidRet | Termiantor::Ret(_) => vec![],
            Termiantor::Br(label) => vec![*label],
            Termiantor::BrCond { cond: _, then, els } => vec![*then, *els],
        }
    }
}
#[derive(Debug)]
// thin erorr type - peps not meant to inspect this, only get the error message.
#[allow(private_interfaces)]
pub enum BuilderError {
    AllocaAInvalidAlign {
        align: NonZeroU32,
    },
    BlockLabelInvalid {
        block: Label,
    },
    PosOutOfRange {
        last_pos: usize,
        pos: usize,
    },
    BuilderPosNotSet,
    IntOpTypeNotIntOrVecInt {
        ty: Type,
    },
    BlockAlreadyTerminated {
        bb: Label,
        old_term: Termiantor,
        new_term: Termiantor,
    },
    VoidRetInNonVoidFnc,
    InvalidSassOperand {
        operand: Operand,
    },
    IntOpOperandNotIntOrVecInt {
        ty: Type,
    },
    BinopTypeMismatch {
        lhs_ty: Type,
        rhs_ty: Type,
    },
    ParamOutOfRange {
        param: u32,
        len: u32,
    },
    RetTypeMismatch {
        got: Type,
        expected: Type,
    },
    VoidLocal,
    ConstIntWhereNonIntExpected {
        hint_ty: Type,
    },
    ConstIntOutOfRange {
        val: i128,
        bitwidth: std::num::NonZero<u8>,
    },
    CondBrCondNotI1 {
        cond: Operand,
        got: Type,
    },
}
