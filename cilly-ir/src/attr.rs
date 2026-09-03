use std::num::NonZeroU32;

use arbitrary::Arbitrary;
use nom::multi::many0;
use traversable::{Traversable, TraversableMut};

use crate::CType;

#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary, PartialEq, Traversable, TraversableMut)]
pub enum Attr {
    #[qparse("zext ")]
    Zext,
    #[qparse("sext ")]
    Sext,
    #[qparse("sret([{size} x i8]) ")]
    Sret {
        #[traverse(skip)]
        size: NonZeroU32,
    },
    #[qparse("likec({0})")]
    /// The value is passed in a special, target-defined manner. It is passed
    /// exactly the same way a given C type would be passed on this target.
    LikeC(CType),
}
#[derive(Clone, Debug, Arbitrary, Default, Traversable, TraversableMut)]
pub(crate) struct AttrList {
    attrs: Vec<Attr>,
}

impl AttrList {
    pub(crate) fn new(attrs: Vec<Attr>) -> Self {
        Self { attrs }
    }

    pub(crate) fn add_attr(&mut self, attr: Attr) {
        if !self.attrs.contains(&attr) {
            self.attrs.push(attr);
        }
    }
}
impl std::fmt::Display for AttrList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for attr in &self.attrs {
            write!(f, "{attr}")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for AttrList {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        use nom::Parser;
        many0(Attr::parse).map(|attrs| Self { attrs }).parse(input)
    }
}
