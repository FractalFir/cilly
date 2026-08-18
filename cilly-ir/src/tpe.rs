use std::num::NonZeroU8;

use arbitrary::Arbitrary;

#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug)]
pub enum Type {
    #[qparse("void")]
    Void,
    #[qparse("i{bitwidth}")]
    Int { bitwidth: NonZeroU8 },
    #[qparse("ptr")]
    Ptr,
}
