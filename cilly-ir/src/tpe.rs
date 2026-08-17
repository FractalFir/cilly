use std::num::NonZeroU8;

#[qparse_macros::qparse("")]
#[derive(Clone)]
pub enum Type {
    #[qparse("void")]
    Void,
    #[qparse("i{bitwidth}")]
    Int { bitwidth: NonZeroU8 },
}
