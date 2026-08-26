use crate::{BuilderError, CastOp, FunctionBuilder, Intrinsic, Operand, Type};

impl FunctionBuilder {
    pub fn build_trunc(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2IntCastOutputNotInt { output: dst_ty });
        }
        if !src_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2IntCastInputNotInt { input: src_ty });
        }
        self.build_cast(CastOp::Trunc, src_ty, val, dst_ty)
    }
    pub fn build_zext(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2IntCastOutputNotInt { output: dst_ty });
        }
        if !src_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2IntCastInputNotInt { input: src_ty });
        }
        self.build_cast(CastOp::ZExt, src_ty, val, dst_ty)
    }
    pub fn build_sext(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2IntCastOutputNotInt { output: dst_ty });
        }
        if !src_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2IntCastInputNotInt { input: src_ty });
        }
        self.build_cast(CastOp::SExt, src_ty, val, dst_ty)
    }
    pub fn build_fptrunc(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Float2FloatCastOutputNotFloat { output: dst_ty });
        }
        if !src_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Float2FloatCastInputNotFloat { input: src_ty });
        }
        self.build_cast(CastOp::FPTrunc, src_ty, val, dst_ty)
    }
    pub fn build_fpext(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Float2FloatCastOutputNotFloat { output: dst_ty });
        }
        if !src_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Float2FloatCastInputNotFloat { input: src_ty });
        }
        self.build_cast(CastOp::FPExt, src_ty, val, dst_ty)
    }
    pub fn build_sitofp(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Int2FloatCastOutputNotFloat { output: dst_ty });
        }
        if !src_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2FloatCastInputNotInt { input: src_ty });
        }
        self.build_cast(CastOp::SIToFP, src_ty, val, dst_ty)
    }
    pub fn build_uitofp(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Int2FloatCastOutputNotFloat { output: dst_ty });
        }
        if !src_ty.is_int_or_vecint() {
            return Err(BuilderError::Int2FloatCastInputNotInt { input: src_ty });
        }
        self.build_cast(CastOp::UIToFP, src_ty, val, dst_ty)
    }
    pub fn build_fptosi(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
        sat: bool,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_int_or_vecint() {
            return Err(BuilderError::Float2IntCastOutputNotInt { output: dst_ty });
        }
        if !src_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Float2IntCastInputNotFloat { input: src_ty });
        }
        if sat {
            self.build_intrinsic(Intrinsic::FpToSiSat {
                dst_ty,
                src_ty,
                val,
            })
        } else {
            self.build_cast(CastOp::FPToSI, src_ty, val, dst_ty)
        }
    }
    pub fn build_fptoui(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
        sat: bool,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_int_or_vecint() {
            return Err(BuilderError::Float2IntCastOutputNotInt { output: dst_ty });
        }
        if !src_ty.is_float_or_vecfloat() {
            return Err(BuilderError::Float2IntCastInputNotFloat { input: src_ty });
        }
        if sat {
            self.build_intrinsic(Intrinsic::FpToUiSat {
                dst_ty,
                src_ty,
                val,
            })
        } else {
            self.build_cast(CastOp::FPToUI, src_ty, val, dst_ty)
        }
    }
    pub fn build_inttoptr(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_ptr() {
            return Err(BuilderError::Int2PtrCastOutputNotPtr { output: dst_ty });
        }
        if !src_ty.is_int() {
            return Err(BuilderError::Int2PtrCastInputNotInt { input: src_ty });
        }
        self.build_cast(CastOp::IntToPtr, src_ty, val, dst_ty)
    }
    pub fn build_ptrtoint(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if !dst_ty.is_int() {
            return Err(BuilderError::Ptr2IntCastOutputNotInt { output: dst_ty });
        }
        if !src_ty.is_ptr() {
            return Err(BuilderError::Ptr2IntCastInputNotPtr { input: src_ty });
        }
        self.build_cast(CastOp::PtrToInt, src_ty, val, dst_ty)
    }
    pub fn build_bitcast(
        &mut self,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    ) -> Result<Operand, BuilderError> {
        if let Some((src_size, dst_size)) = src_ty.try_bitsize().zip(dst_ty.try_bitsize()) {
            if src_size != dst_size {
                return Err(BuilderError::BitcastSizeMismatch { src_size, dst_size });
            }
        }
        self.build_cast(CastOp::BitCast, src_ty, val, dst_ty)
    }
}
