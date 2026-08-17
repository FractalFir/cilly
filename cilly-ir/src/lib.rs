mod func;
pub(crate) use func::*;
mod global;
pub(crate) use global::*;
mod module;
pub use module::*;
mod linkage;
pub use linkage::*;
mod global_ident;
pub use global_ident::*;
mod tpe;
use nom::{Parser, combinator::success};
pub use tpe::*;
mod attr;
pub use attr::*;
mod ctype;
pub use ctype::*;
#[qparse_macros::qparse("PlaceHolder")]
#[derive(Default)]
pub(crate) struct PlaceHolder;
#[derive(Clone, Default)]
pub(crate) struct SourceLocation {
    pub(crate) opt: Option<SourceLocationInner>,
}
impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = &self.opt {
            write!(f, "{s}")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for SourceLocation {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom::branch::alt((
            SourceLocationInner::parse.map(|s| Self { opt: Some(s) }),
            success(SourceLocation { opt: None }),
        ))
        .parse(input)
    }
}
#[qparse_macros::qparse("; source {file}:{line}:{col}
")]
#[derive(Clone)]
pub(crate) struct SourceLocationInner {
    file: GlobalIdent,
    line: u32,
    col: u32,
}
