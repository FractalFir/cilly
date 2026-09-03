use std::num::{NonZeroU8, NonZeroU32};

use arbitrary::Arbitrary;
use nom::{bytes::complete::tag, combinator::map, multi::separated_list0, sequence::delimited};
use traversable::{Traversable, TraversableMut};
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq, Traversable, TraversableMut)]
pub enum FloatTy {
    #[qparse("float")]
    Float,
    #[qparse("half")]
    Half,
    #[qparse("double")]
    Double,
    #[qparse("f128p")]
    F128,
}
impl FloatTy {
    pub fn bitwidth(&self) -> u32 {
        match self {
            FloatTy::Float => u32::BITS,
            FloatTy::Half => u16::BITS,
            FloatTy::Double => u64::BITS,
            FloatTy::F128 => u128::BITS,
        }
    }
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq, Traversable, TraversableMut)]
pub enum ScalarTy {
    #[qparse("{0}")]
    Int(IntTy),
    #[qparse("ptr")]
    Ptr,
    #[qparse("{0}")]
    Float(FloatTy),
}
impl ScalarTy {
    pub const I1: Self = ScalarTy::Int(IntTy {
        bitwidth: NonZeroU8::new(1).unwrap(),
    });
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
#[derive(Clone, Arbitrary, Debug, PartialEq, Traversable, TraversableMut)]
pub struct IntTy {
    #[traverse(skip)]
    pub(crate) bitwidth: NonZeroU8,
}
#[derive(Clone, Arbitrary, Debug, PartialEq, Traversable, TraversableMut)]
pub struct StructTy {
    elems: Vec<Type>,
}
impl std::fmt::Display for StructTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (n, elem) in self.elems.iter().enumerate() {
            if n != 0 {
                f.write_str(", ")?;
            }
            write!(f, "{elem}")?;
        }
        write!(f, "}}")
    }
}
impl qparse::Parseable<qparse::Display> for StructTy {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        use nom::Parser;
        map(
            delimited(
                tag("{"),
                separated_list0(
                    tag(", "),
                    <Type as qparse::Parseable<qparse::Display>>::parse,
                ),
                tag("}"),
            ),
            |elems| StructTy { elems },
        )
        .parse(input)
    }
}

#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq, Traversable, TraversableMut)]
pub enum Type {
    #[qparse("void")]
    Void,
    #[qparse("{0}")]
    ScalarTy(ScalarTy),
    #[qparse("<{element_count} x {element_ty}>")]
    VectorTy {
        element_ty: ScalarTy,
        #[traverse(skip)]
        element_count: NonZeroU8,
    },
    #[qparse("[{element_count} x {element_ty}]")]
    ArrayTy {
        element_ty: ScalarTy,
        #[traverse(skip)]
        element_count: NonZeroU32,
    },
    #[qparse("{0}")]
    Struct(StructTy),
}
pub static F16_TY: Type = Type::ScalarTy(ScalarTy::Float(FloatTy::Half));
pub static F32_TY: Type = Type::ScalarTy(ScalarTy::Float(FloatTy::Float));
pub static F64_TY: Type = Type::ScalarTy(ScalarTy::Float(FloatTy::Double));
pub static F128_TY: Type = Type::ScalarTy(ScalarTy::Float(FloatTy::F128));
pub static I1_TY: Type = Type::ix(NonZeroU8::new(1).unwrap());
pub static I8_TY: Type = Type::ix(NonZeroU8::new(8).unwrap());
pub static I64_TY: Type = Type::ix(NonZeroU8::new(64).unwrap());
pub static PTR_TY: Type = Type::ScalarTy(ScalarTy::Ptr);
impl Type {
    pub const fn ix(bitwidth: NonZeroU8) -> Self {
        Self::ScalarTy(ScalarTy::Int(IntTy { bitwidth }))
    }
    pub fn ty_and_flag(ty: Type) -> Self {
        Self::Struct(StructTy {
            elems: vec![ty, I1_TY.clone()],
        })
    }
    pub fn strct(elems: Vec<Type>) -> Self {
        Self::Struct(StructTy { elems })
    }
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
    pub fn struct_fields(&self) -> Option<&[Type]> {
        let Self::Struct(strct) = self else {
            return None;
        };
        Some(&strct.elems[..])
    }
    pub fn try_bitsize(&self) -> Option<u32> {
        match self {
            Type::Void => Some(0),
            Type::ScalarTy(scalar) => scalar.try_bitsize(),
            Type::VectorTy {
                element_ty,
                element_count,
            } => Some(element_ty.try_bitsize()? * element_count.get() as u32),
            Type::ArrayTy {
                element_ty,
                element_count,
            } => Some(element_ty.try_bitsize()? * element_count.get()),
            Type::Struct(_) => None,
        }
    }
    pub fn vec_elem_count(&self) -> Option<NonZeroU8> {
        let Type::VectorTy { element_count, .. } = self else {
            return None;
        };
        Some(*element_count)
    }
    pub fn vec_elem_ty(&self) -> Option<ScalarTy> {
        let Type::VectorTy { element_ty, .. } = self else {
            return None;
        };
        Some(element_ty.clone())
    }
}
