use std::num::NonZeroU8;

use arbitrary::Arbitrary;

#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, PartialEq)]
pub enum Type {
    #[qparse("void")]
    Void,
    #[qparse("i{bitwidth}")]
    Int { bitwidth: NonZeroU8 },
    #[qparse("ptr")]
    Ptr,
}
impl Type {
    pub fn is_int_or_vecint(&self) -> bool {
        matches!(self, Self::Int { .. })
    }
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int { .. })
    }
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }
}
