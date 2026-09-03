use arbitrary::Arbitrary;
use traversable::{Traversable, TraversableMut};

#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary, PartialEq, Traversable, TraversableMut)]
pub enum CType {
    #[qparse("unsigned char")]
    UChar,
}
