use crate::{
    AllocA, AttrAndTy, Binop, CallArgs, CastOp, Constant, FCmp, Fnc, FuncRef, I1_TY, ICmp,
    InstrList, Instruction, Intrinsic, Label, Local, Locals, Module, Operand, PTR_TY, SSAVal,
    ScalarTy, TyAndAttr, Type, to_body,
};
use std::num::NonZeroU32;
mod binop;
mod cast;
mod cmp;
mod err;
pub use err::*;
mod intrinsics;
mod mem;
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
    pub fn build_trap(&mut self) -> Result<(), BuilderError> {
        self.build_term(Termiantor::Trap)
    }
    pub fn build_condbr(
        &mut self,
        cond: Operand,
        then: Label,
        els: Label,
    ) -> Result<(), BuilderError> {
        let got = self.get_type(&cond, &I1_TY)?;
        if *got != I1_TY {
            return Err(BuilderError::CondBrCondNotI1 {
                cond,
                got: got.clone(),
            });
        }
        self.check_label(then)?;
        self.check_label(els)?;
        self.build_term(Termiantor::BrCond { cond, then, els })
    }
    pub fn build_switch(
        &mut self,
        default: Label,
        ty: Type,
        cases: Vec<(Constant, Label)>,
        val: Operand,
    ) -> Result<(), BuilderError> {
        self.check_label(default)?;
        for (cst, case) in &cases {
            self.check_label(*case)?;
            cst.get_ty(&ty)?;
        }
        let val_ty = self.get_type(&val, &ty)?;
        if val_ty != &ty {
            return Err(BuilderError::SwitchValTyWtong {
                val_ty: val_ty.clone(),
                val,
            });
        }
        self.build_term(Termiantor::Switch {
            default,
            ty,
            cases,
            val,
        })
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
    fn build_intrinsic(&mut self, intrinsic: Intrinsic) -> Result<Operand, BuilderError> {
        let dst = self.alloc_ssa_id(intrinsic.res_ty().clone());
        self.insert_at_pos(Instruction::CallIntrinsic { dst, intrinsic })?;
        Ok(Operand::SSA(dst))
    }
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
        let dst = self.alloc_ssa_id(
            ty.vec_elem_count()
                .map(|e| Type::VectorTy {
                    element_ty: ScalarTy::I1,
                    element_count: e,
                })
                .unwrap_or_else(|| I1_TY.clone()),
        );
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
        let dst = self.alloc_ssa_id(
            ty.vec_elem_count()
                .map(|e| Type::VectorTy {
                    element_ty: ScalarTy::I1,
                    element_count: e,
                })
                .unwrap_or_else(|| I1_TY.clone()),
        );
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
    pub fn build_extractelement(
        &mut self,
        vector_ty: Type,
        vector: Operand,
        index: Operand,
    ) -> Result<Operand, BuilderError> {
        // FIXME: better checks
        let dst = self.alloc_ssa_id(Type::ScalarTy(vector_ty.vec_elem_ty().unwrap()));
        self.insert_at_pos(Instruction::ExtractElement {
            dst,
            vector_ty,
            vector,
            index,
        })?;
        Ok(Operand::SSA(dst))
    }
    pub fn build_insertelement(
        &mut self,
        vector_ty: Type,
        vector: Operand,
        element: Operand,
        element_ty: Type,
        index: Operand,
    ) -> Result<Operand, BuilderError> {
        let Some(ety) = vector_ty.vec_elem_ty() else {
            return Err(BuilderError::InsertElementArgNotVec{vector_ty});
        };
        if Type::ScalarTy(ety) != element_ty{
            return Err(BuilderError::InsertElementArgNotVecOfVecTy{expected:element_ty, got:ety});
        }
        let ety = self.get_type(&element, &element_ty)?;
        if *ety != element_ty{
            return Err(BuilderError::InsertElementElementOperandWrongTy{expected:element_ty, got:ety})
        }
        let dst = self.alloc_ssa_id(vector_ty.clone());
        self.insert_at_pos(Instruction::InsertElement {
            dst,
            vector_ty,
            vector,
            element,
            element_ty,
            index,
        })?;
        Ok(Operand::SSA(dst))
    }
    pub fn get_type<'ty, 's: 'ty>(
        &'s self,
        op: &Operand,
        implicit_ty: &'ty Type,
    ) -> Result<&'ty Type, BuilderError> {
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
    // unops
    pub fn build_fneg(&mut self, ty: Type, val: Operand) -> Result<Operand, BuilderError> {
        if !ty.is_float_or_vecfloat() {
            return Err(BuilderError::FloatOpTypeNotFloatOrVecInt { ty });
        }
        let got = self.get_type(&val, &ty)?;
        if *got != ty {
            return Err(BuilderError::FloatOpOperandNotFloatOrVecInt { ty: got.clone() });
        }
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::Fneg { dst, ty, val })?;
        Ok(Operand::SSA(dst))
    }

    pub fn build_voidcall(
        &mut self,
        callee: Operand,
        call_args: Vec<(TyAndAttr, Operand)>,
    ) -> Result<(), BuilderError> {
        let call_args = CallArgs { args: call_args };
        let calle_ty = self.get_type(&callee, &PTR_TY)?;
        if !calle_ty.is_ptr() {
            return Err(BuilderError::CalleeNotPtrOrFn {
                callee,
                calle_ty: calle_ty.clone(),
            });
        }
        self.insert_at_pos(Instruction::VoidCall { callee, call_args })
    }
    pub fn build_call(
        &mut self,
        output: TyAndAttr,
        callee: Operand,
        call_args: Vec<(TyAndAttr, Operand)>,
    ) -> Result<Operand, BuilderError> {
        let call_args = CallArgs { args: call_args };
        let calle_ty = self.get_type(&callee, &PTR_TY)?;
        if !calle_ty.is_ptr() {
            return Err(BuilderError::CalleeNotPtrOrFn {
                callee,
                calle_ty: calle_ty.clone(),
            });
        }
        let dst = self.alloc_ssa_id(output.ty.clone());
        self.insert_at_pos(Instruction::Call {
            dst,
            output: AttrAndTy {
                attr: output.attr,
                ty: output.ty,
            },
            callee,
            call_args,
        })?;
        Ok(Operand::SSA(dst))
    }
    pub fn ptr_offset(
        &mut self,
        ptr: Operand,
        off_ty: Type,
        off: Operand,
        inbounds: bool,
    ) -> Result<Operand, BuilderError> {
        let off_got_ty = self.get_type(&off, &off_ty)?;
        if off_ty != *off_got_ty {
            return Err(BuilderError::PtrOffsetOffsetOperandWrongType {
                off,
                off_ty: off_ty.clone(),
                got: off_got_ty.clone(),
            });
        }
        if !off_ty.is_int() {
            return Err(BuilderError::PtrOffsetOffsetWrongType { off_ty });
        }
        let ptr_ty = self.get_type(&ptr, &PTR_TY)?;
        if !ptr_ty.is_ptr() {
            return Err(BuilderError::PtrOffsetPtrIsNotPtr {
                ptr_ty: ptr_ty.clone(),
                ptr,
            });
        }
        let dst = self.alloc_ssa_id(ptr_ty.clone());
        self.insert_at_pos(Instruction::PtrOffset {
            dst,
            ptr,
            off_ty,
            off,
            inbounds,
        })?;
        Ok(Operand::SSA(dst))
    }
    // select
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
    // casts
    fn build_cast(
        &mut self,
        op: CastOp,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        let dst = self.alloc_ssa_id(dst_ty.clone());
        let src_got_ty = self.get_type(&val, &src_ty)?;
        if *src_got_ty != src_ty {
            return Err(BuilderError::InvalidCastSrc {
                got: src_got_ty.clone(),
                expected: src_ty,
            });
        }
        self.insert_at_pos(Instruction::Cast {
            dst,
            op,
            src_ty,
            val,
            dst_ty,
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
    pub fn build_insertvalue(
        &mut self,
        aggregate_ty: Type,
        value_ty: Type,
        aggregate: Operand,
        element: Operand,
        index: u64,
    ) -> Result<Operand, BuilderError> {
        // FIXME: better error checks
        let dst = self.alloc_ssa_id(aggregate_ty.clone());
        self.insert_at_pos(Instruction::InsertValue {
            dst,
            aggregate_ty,
            value_ty,
            aggregate,
            element,
            index,
        })?;
        Ok(Operand::SSA(dst))
    }
    pub fn build_extractvalue(
        &mut self,
        aggregate_ty: Type,
        aggregate: Operand,
        index: u64,
    ) -> Result<Operand, BuilderError> {
        let Some(fields) = aggregate_ty.struct_fields() else {
            return Err(BuilderError::ExtractValueAggregateTyNotStruct { aggregate_ty });
        };
        let fld = fields
            .get(index as usize)
            .ok_or(BuilderError::FieldIndexOOB {
                aggregate_ty: aggregate_ty.clone(),
                index,
            })?;
        let dst = self.alloc_ssa_id(fld.clone());
        // FIXME: better error checks
        self.insert_at_pos(Instruction::ExtractValue {
            dst,
            aggregate_ty,
            aggregate,
            index,
        })?;
        Ok(Operand::SSA(dst))
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
        let ssa_id = self.alloc_ssa_id(PTR_TY.clone());
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
    Switch {
        default: Label,
        ty: Type,
        cases: Vec<(Constant, Label)>,
        val: Operand,
    },
    Trap,
}
impl Termiantor {
    pub fn sucessors(&self) -> Vec<Label> {
        match self {
            Termiantor::VoidRet | Termiantor::Ret(_) | Termiantor::Trap => vec![],
            Termiantor::Br(label) => vec![*label],
            Termiantor::BrCond { cond: _, then, els } => vec![*then, *els],
            Termiantor::Switch { default, cases, .. } => std::iter::once(*default)
                .chain(cases.iter().map(|(_, l)| *l))
                .collect::<_>(),
        }
    }
}
