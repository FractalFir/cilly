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
pub enum Type {
    #[qparse("void")]
    Void,
    #[qparse("i{bitwidth}")]
    Int { bitwidth: NonZeroU8 },
    #[qparse("ptr")]
    Ptr,
    #[qparse("{0}")]
    Float(FloatTy),
}
impl Type {
    pub const I1: Type = Type::Int {
        bitwidth: NonZeroU8::new(1).unwrap(),
    };
    pub fn is_int_or_vecint(&self) -> bool {
        matches!(self, Self::Int { .. })
    }
    pub fn is_float_or_vecfloat(&self) -> bool {
        matches!(self, Self::Float(_))
    }
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int { .. })
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float { .. })
    }
    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Ptr)
    }
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }
    pub fn try_bitsize(&self) -> Option<u32> {
        match self {
            Type::Void => Some(0),
            Type::Int { bitwidth } => Some(bitwidth.get() as _),
            Type::Ptr => None,
            Type::Float(float_ty) => Some(float_ty.bitwidth()),
        }
    }
}
