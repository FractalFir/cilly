use crate::{Binop, BuilderError, FunctionBuilder, Operand, Type};

impl FunctionBuilder {
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
}
