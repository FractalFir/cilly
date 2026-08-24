use arbitrary::Arbitrary;

use crate::{Operand, Type};

#[qparse_macros::qparse("")]
#[derive(Clone,Debug,Arbitrary)]
pub(crate) enum Intrinsic{
    #[qparse("{dst_ty} @llvm.fptoui.sat.{dst_ty}.{src_ty}({src_ty} {val})")]
    FpToUiSat{
        dst_ty:Type,
        src_ty:Type,
        val:Operand,
    },
    #[qparse("{dst_ty} @llvm.fptosi.sat.{dst_ty}.{src_ty}({src_ty} {val})")]
    FpToSiSat{
        dst_ty:Type,
        src_ty:Type,
        val:Operand,
    }
}
impl Intrinsic{
    pub(crate) fn res_ty(&self)->&Type{
        match self{
            Intrinsic::FpToUiSat { dst_ty,.. } |
            Intrinsic::FpToSiSat { dst_ty,.. } => dst_ty,
        }
    }
}