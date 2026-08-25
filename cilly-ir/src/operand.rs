use std::num::NonZeroU8;

use arbitrary::Arbitrary;

use crate::{BuilderError, GlobalIdent, IntTy, ScalarTy, Type};

#[qparse_macros::qparse("%v{0}")]
#[derive(Copy, Clone, Arbitrary, PartialEq, Eq, Debug)]
pub struct SSAVal(pub(crate) u32);
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, PartialEq, Eq, Debug)]
pub enum Operand {
    #[qparse("{0}")]
    SSA(SSAVal),
    #[qparse("{0}")]
    Global(GlobalIdent),
    #[qparse("{0}")]
    Constant(Constant),
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, PartialEq, Eq, Debug)]
pub enum Constant {
    #[qparse("true")]
    True,
    #[qparse("false")]
    False,
    #[qparse("{0}")]
    Int(i128),
}
impl Constant {
    pub fn get_ty<'ty>(&self, hint_ty: &'ty Type) -> Result<&'ty Type, BuilderError> {
        match self {
            Constant::False | Constant::True => Ok(&Type::I1),
            Constant::Int(int) => {
                let Type::ScalarTy(ScalarTy::Int(IntTy { bitwidth })) = hint_ty else {
                    return Err(BuilderError::ConstIntWhereNonIntExpected {
                        hint_ty: hint_ty.clone(),
                    });
                };
                let bits = bitwidth.get();
                let int_ty_min = -(1i128 << (bits - 1));
                let int_ty_max = ((1u128 << bits) - 1) as i128;
                if int_ty_min <= *int && *int <= int_ty_max {
                    return Err(BuilderError::ConstIntOutOfRange {
                        val: *int,
                        bitwidth: *bitwidth,
                    });
                }
                Ok(hint_ty)
            }
        }
    }
}
