use crate::{BuilderError, FunctionBuilder, Intrinsic, Operand, Type};

impl FunctionBuilder {
    pub fn sadd_with_ovf(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        self.build_intrinsic(Intrinsic::SAddWithOverflow { ty, lhs, rhs })
    }
    pub fn ssub_with_ovf(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        self.build_intrinsic(Intrinsic::SSubWithOverflow { ty, lhs, rhs })
    }
    pub fn smul_with_ovf(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        self.build_intrinsic(Intrinsic::SMulWithOverflow { ty, lhs, rhs })
    }
    pub fn uadd_with_ovf(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        self.build_intrinsic(Intrinsic::UAddWithOverflow { ty, lhs, rhs })
    }
    pub fn usub_with_ovf(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        self.build_intrinsic(Intrinsic::USubWithOverflow { ty, lhs, rhs })
    }
    pub fn umul_with_ovf(
        &mut self,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&lhs, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let lhs_ty = self.get_type(&lhs, &ty)?;
        let rhs_ty = self.get_type(&rhs, &ty)?;
        if lhs_ty != rhs_ty {
            return Err(BuilderError::BinopTypeMismatch {
                lhs_ty: lhs_ty.clone(),
                rhs_ty: rhs_ty.clone(),
            });
        }
        self.build_intrinsic(Intrinsic::UMulWithOverflow { ty, lhs, rhs })
    }
}
