use crate::{BuilderError, FCmp, FunctionBuilder, ICmp, Operand, Type};

impl FunctionBuilder {
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
}
