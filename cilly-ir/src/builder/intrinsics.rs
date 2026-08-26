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
    pub fn build_bswap(&mut self, ty: Type, val: Operand) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&val, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        let bitsize = ty.try_bitsize().unwrap();
        if bitsize == 8 {
            return Ok(val);
        }
        if bitsize % 16 != 0 {
            return Err(BuilderError::BswapByteSizeNotEven { bitsize, val });
        }
        self.build_intrinsic(Intrinsic::Bswap { ty, val })
    }
    pub fn build_ctlz(
        &mut self,
        ty: Type,
        val: Operand,
        is_zero_poison: bool,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&val, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        self.build_intrinsic(Intrinsic::Ctlz {
            ty,
            val,
            is_zero_poison,
        })
    }
    pub fn build_cttz(
        &mut self,
        ty: Type,
        val: Operand,
        is_zero_poison: bool,
    ) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&val, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        self.build_intrinsic(Intrinsic::Cttz {
            ty,
            val,
            is_zero_poison,
        })
    }
    pub fn build_ctpop(&mut self, ty: Type, val: Operand) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&val, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        self.build_intrinsic(Intrinsic::Ctpop { ty, val })
    }
    pub fn build_bitreverse(&mut self, ty: Type, val: Operand) -> Result<Operand, BuilderError> {
        if !ty.is_int() {
            return Err(BuilderError::IntOpTypeNotInt { ty });
        }
        if !self.get_type(&val, &ty)?.is_int() {
            return Err(BuilderError::IntOpOperandNot { ty });
        }
        self.build_intrinsic(Intrinsic::Bitreverse { ty, val })
    }
    pub fn build_uadd_sat(
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
        self.build_intrinsic(Intrinsic::UAddSat { ty, lhs, rhs })
    }
    pub fn build_sadd_sat(
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
        self.build_intrinsic(Intrinsic::SAddSat { ty, lhs, rhs })
    }
    pub fn build_usub_sat(
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
        self.build_intrinsic(Intrinsic::USubSat { ty, lhs, rhs })
    }
    pub fn build_ssub_sat(
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
        self.build_intrinsic(Intrinsic::SSubSat { ty, lhs, rhs })
    }
}
