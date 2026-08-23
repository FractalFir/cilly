use std::num::{NonZeroU8, NonZeroU32};

use crate::{
    AllocA, Binop, FCmp, Fnc, FuncRef, ICmp, InstrList, Instruction, Label, Local, Locals, Module,
    Operand, SSAVal, Type, to_body,
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
        self.pos = Some((block, pos));
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
        let got = self.get_type(&cond, &Type::I1)?;
        if *got != Type::I1 {
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
    pub fn get_local_ty(&self, idx: Local) -> Result<&Type, BuilderError> {
        let local_idx = idx.id as usize;
        self.locals
            .locals
            .get(local_idx)
            .ok_or(BuilderError::LocalInvalid { idx })
            .map(|l| &l.ty)
    }
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
        self.pos = Some((label, pos + 1));
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
    fn build_icmp(
        &mut self,
        cmp: ICmp,
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
        let dst = self.alloc_ssa_id(Type::I1.clone());
        self.insert_at_pos(Instruction::ICmp {
            dst,
            ty,
            lhs,
            rhs,
            cmp,
        })?;
        Ok(Operand::SSA(dst))
    }
    fn build_fcmp(
        &mut self,
        cmp: FCmp,
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
        let dst = self.alloc_ssa_id(Type::I1.clone());
        self.insert_at_pos(Instruction::FCmp {
            dst,
            ty,
            lhs,
            rhs,
            cmp,
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
    fn build_fbinop(
        &mut self,
        op: Binop,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_float_or_vecfloat() {
            return Err(BuilderError::FloatOpTypeNotFloatOrVecInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_float_or_vecfloat() {
            return Err(BuilderError::FloatOpOperandNotFloatOrVecInt { ty });
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
    pub fn build_xor(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::Xor, ty, lhs, rhs)
    }
    pub fn build_udiv(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::UDiv, ty, lhs, rhs)
    }
    pub fn build_sdiv(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::SDiv, ty, lhs, rhs)
    }
    pub fn build_urem(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::URem, ty, lhs, rhs)
    }
    pub fn build_srem(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::SRem, ty, lhs, rhs)
    }
    pub fn build_shl(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::Shl, ty, lhs, rhs)
    }
    pub fn build_lshr(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::LShr, ty, lhs, rhs)
    }
    pub fn build_ashr(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::AShr, ty, lhs, rhs)
    }
    pub fn build_and(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::And, ty, lhs, rhs)
    }
    pub fn build_or(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_ibinop(Binop::Or, ty, lhs, rhs)
    }
    // Float binops
    pub fn build_fadd(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fbinop(Binop::FAdd, ty, lhs, rhs)
    }
    pub fn build_fsub(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fbinop(Binop::FSub, ty, lhs, rhs)
    }
    pub fn build_fmul(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fbinop(Binop::FMul, ty, lhs, rhs)
    }
    pub fn build_fdiv(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fbinop(Binop::FDiv, ty, lhs, rhs)
    }
    pub fn build_frem(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fbinop(Binop::FRem, ty, lhs, rhs)
    }
    // icmps
    pub fn build_eq(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::Eq, ty, lhs, rhs)
    }
    pub fn build_ne(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::Ne, ty, lhs, rhs)
    }
    pub fn build_ugt(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::UGt, ty, lhs, rhs)
    }
    pub fn build_uge(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::UGe, ty, lhs, rhs)
    }
    pub fn build_ult(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::ULt, ty, lhs, rhs)
    }
    pub fn build_ule(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::ULe, ty, lhs, rhs)
    }
    pub fn build_sgt(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::SGt, ty, lhs, rhs)
    }
    pub fn build_sge(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::SGe, ty, lhs, rhs)
    }
    pub fn build_slt(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::SLt, ty, lhs, rhs)
    }
    // fcmp
    pub fn build_foeq(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::OEq, ty, lhs, rhs)
    }
    pub fn build_fogt(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::OGt, ty, lhs, rhs)
    }
    pub fn build_foge(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::OGe, ty, lhs, rhs)
    }
    pub fn build_folt(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::OLt, ty, lhs, rhs)
    }
    pub fn build_fole(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::OLe, ty, lhs, rhs)
    }
    pub fn build_fone(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::ONe, ty, lhs, rhs)
    }
    pub fn build_fueq(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::UEq, ty, lhs, rhs)
    }
    pub fn build_fugt(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::UGt, ty, lhs, rhs)
    }
    pub fn build_fuge(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::UGe, ty, lhs, rhs)
    }
    pub fn build_fult(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::ULt, ty, lhs, rhs)
    }
    pub fn build_fule(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::ULe, ty, lhs, rhs)
    }
    pub fn build_fune(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_fcmp(FCmp::UNe, ty, lhs, rhs)
    }
    pub fn build_sle(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        self.build_icmp(ICmp::SLe, ty, lhs, rhs)
    }
    pub fn build_select(
        &mut self,
        cond_ty: Type,
        ty: Type,
        cond: Operand,
        then: Operand,
        els: Operand,
    ) -> Result<Operand, BuilderError> {
        let lhs_ty = self.get_type(&then, &ty)?;
        let rhs_ty = self.get_type(&els, &ty)?;
        let cond_real_ty = self.get_type(&cond, &cond_ty)?;
        if *cond_real_ty != cond_ty {
            return Err(BuilderError::InvalidSelCond {
                expected: cond_ty.clone(),
                got: cond_real_ty.clone(),
            });
        }
        if lhs_ty != rhs_ty || *lhs_ty != ty {
            return Err(BuilderError::SelInputTypeMismatch {
                lhs: lhs_ty.clone(),
                rhs: rhs_ty.clone(),
                expected: ty,
            });
        }
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::Select {
            dst,
            cond,
            ty,
            then,
            els,
            cond_ty,
        })?;
        Ok(Operand::SSA(dst))
    }
    // locals - our "poor man's phi-s"
    // Locals
    pub fn build_load_local(&mut self, local: Local) -> Result<Operand, BuilderError> {
        let ty = self.get_local_ty(local)?.clone();
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::LoadLocal { dst, local, ty })?;
        Ok(Operand::SSA(dst))
    }
    pub fn build_store_local(&mut self, local: Local, val: Operand) -> Result<(), BuilderError> {
        let ty = self.get_local_ty(local)?.clone();
        let got = self.get_type(&val, &ty)?;
        if *got != ty {
            return Err(BuilderError::StoreLocalTypeMismatch {
                local,
                got: got.clone(),
                expected: ty,
            });
        }
        self.insert_at_pos(Instruction::StoreLocal { local, ty, val })
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
            unreachable!()
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
#[derive(Debug)]
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
    InvalidSelCond {
        expected: Type,
        got: Type,
    },
    SelInputTypeMismatch {
        lhs: Type,
        rhs: Type,
        expected: Type,
    },
    LocalInvalid {
        idx: Local,
    },
    StoreLocalTypeMismatch {
        local: Local,
        got: Type,
        expected: Type,
    },
    FloatOpTypeNotFloatOrVecInt {
        ty: Type,
    },
    FloatOpOperandNotFloatOrVecInt {
        ty: Type,
    },
}
