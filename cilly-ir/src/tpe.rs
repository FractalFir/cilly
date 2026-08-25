use std::num::NonZeroU8;

use arbitrary::Arbitrary;
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq)]
pub enum FloatTy {
    #[qparse("float")]
    Float,
    #[qparse("half")]
    Half,
    #[qparse("double")]
    Double,
}
impl FloatTy {
    pub fn bitwidth(&self) -> u32 {
        match self {
            FloatTy::Float => u32::BITS,
            FloatTy::Half => u16::BITS,
            FloatTy::Double => u64::BITS,
        }
    }
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq)]
pub enum ScalarTy {
    #[qparse("{0}")]
    Int(IntTy),
    #[qparse("ptr")]
    Ptr,
    #[qparse("{0}")]
    Float(FloatTy),
}
impl ScalarTy {
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }
    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Ptr)
    }
    pub fn try_bitsize(&self) -> Option<u32> {
        match self {
            ScalarTy::Int(IntTy { bitwidth }) => Some(bitwidth.get() as u32),
            ScalarTy::Ptr => None,
            ScalarTy::Float(float_ty) => Some(float_ty.bitwidth()),
        }
    }
}

#[qparse_macros::qparse("i{bitwidth}")]
#[derive(Clone, Arbitrary, Debug, PartialEq)]
pub struct IntTy {
    pub(crate) bitwidth: NonZeroU8,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq)]
pub enum Type {
    #[qparse("void")]
    Void,
    #[qparse("{0}")]
    ScalarTy(ScalarTy),
    #[qparse("<{element_count} x {element_ty}>")]
    VectorTy {
        element_ty: ScalarTy,
        element_count: NonZeroU8,
    },
}
impl Type {
    pub const I1: Type = Type::ix(NonZeroU8::new(1).unwrap());
    pub const fn ix(bitwidth: NonZeroU8) -> Self {
        Self::ScalarTy(ScalarTy::Int(IntTy { bitwidth }))
    }
    pub const PTR: Type = Type::ScalarTy(ScalarTy::Ptr);
    pub fn is_int(&self) -> bool {
        match self {
            Self::ScalarTy(ScalarTy::Int(_)) => true,
            _ => false,
        }
    }
    pub fn is_ptr(&self) -> bool {
        match self {
            Self::ScalarTy(ScalarTy::Ptr) => true,
            _ => false,
        }
    }
    pub fn is_int_or_vecint(&self) -> bool {
        match self {
            Self::ScalarTy(ScalarTy::Int(_))
            | Self::VectorTy {
                element_ty: ScalarTy::Int(_),
                ..
            } => true,
            _ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            Self::ScalarTy(ScalarTy::Float(_)) => true,
            _ => false,
        }
    }
    pub fn is_float_or_vecfloat(&self) -> bool {
        match self {
            Self::ScalarTy(ScalarTy::Float(_))
            | Self::VectorTy {
                element_ty: ScalarTy::Float(_),
                ..
            } => true,
            _ => false,
        }
    }
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }
    pub fn try_bitsize(&self) -> Option<u32> {
        match self {
            Type::Void => Some(0),
            Type::ScalarTy(scalar) => scalar.try_bitsize(),
            Type::VectorTy {
                element_ty,
                element_count,
            } => Some(element_ty.try_bitsize()? * element_count.get() as u32),
        }
    }
}
