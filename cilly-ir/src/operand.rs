use arbitrary::Arbitrary;

use crate::{
    BuilderError, F16_TY, F32_TY, F64_TY, F128_TY, GlobalIdent, I1_TY, IntTy, PTR_TY, ScalarTy,
    Type,
};

#[qparse_macros::qparse("%v{0:cut()}")]
#[derive(Copy, Clone, Arbitrary, PartialEq, Eq, Debug)]
pub struct SSAVal(pub(crate) u32);
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, PartialEq, Debug)]
pub enum Operand {
    #[qparse("{0}")]
    SSA(SSAVal),
    #[qparse("{0}")]
    Global(GlobalIdent),
    #[qparse("{0}")]
    Constant(Constant),
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, PartialEq, Debug)]
pub enum Constant {
    #[qparse("true")]
    True,
    #[qparse("false")]
    False,
    #[qparse("{0}")]
    Int(i128),
    #[qparse("0xH{0:x}")]
    F16(u16),
    #[qparse("f0x{0:x}")]
    Float(u32),
    #[qparse("{0}")]
    Double(f64),
    #[qparse("0xL{0:x}")]
    F128(u128),
    #[qparse("undef")]
    Undef,
    #[qparse("null")]
    Null,
}
impl Constant {
    pub fn as_i128(&self) -> i128 {
        match self {
            Constant::True => 1,
            Constant::False => 0,
            Constant::Int(val) => *val,
            Constant::Float(_) => todo!(),
            Constant::Double(_) => todo!(),
            Constant::Undef => 0,
            Constant::Null => 0,
            Constant::F16(_) => todo!(),
            Constant::F128(_) => todo!(),
        }
    }
    pub fn get_ty<'ty>(&self, hint_ty: &'ty Type) -> Result<&'ty Type, BuilderError> {
        match self {
            Constant::False | Constant::True => Ok(&I1_TY),
            Constant::Int(int) => {
                let Type::ScalarTy(ScalarTy::Int(IntTy { bitwidth })) = hint_ty else {
                    return Err(BuilderError::ConstIntWhereNonIntExpected {
                        hint_ty: hint_ty.clone(),
                    });
                };
                let bits = bitwidth.get();
                if bits < 128 {
                    let int_ty_min = -(1i128 << (bits - 1));
                    let int_ty_max = ((1u128 << bits) - 1) as i128;
                    if !(int_ty_min <= *int && *int <= int_ty_max) {
                        return Err(BuilderError::ConstIntOutOfRange {
                            val: *int,
                            bitwidth: *bitwidth,
                        });
                    }
                }
                Ok(hint_ty)
            }
            Constant::F16(_) => Ok(&F16_TY),
            Constant::Float(_) => Ok(&F32_TY),
            Constant::Double(_) => Ok(&F64_TY),
            Constant::F128(_) => Ok(&F128_TY),
            Constant::Undef => Ok(hint_ty),
            Constant::Null => Ok(&PTR_TY),
        }
    }
}
