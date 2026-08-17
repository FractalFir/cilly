use std::num::NonZeroU32;

use nom::multi::many0;

use crate::CType;

#[qparse_macros::qparse("")]
pub enum Attr {
    #[qparse("zext ")]
    Zext,
    #[qparse("sext ")]
    Sext,
    #[qparse("sret([{size} x i8]) ")]
    Sret { size: NonZeroU32 },
    #[qparse("likec({0})")]
    /// The value is passed in a special, target-defined manner. It is passed
    /// exactly the same way a given C type would be passed on this target.
    LikeC(CType),
}
pub(crate) struct AttrList {
    attrs: Vec<Attr>,
}

impl AttrList {
    pub(crate) fn new(attrs: Vec<Attr>) -> Self {
        Self { attrs }
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
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        use nom::Parser;
        many0(Attr::parse).map(|attrs| Self { attrs }).parse(input)
    }
}
