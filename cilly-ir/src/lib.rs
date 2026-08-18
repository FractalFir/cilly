mod func;
use arbitrary::{Arbitrary, Unstructured};
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
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::multispace0,
    combinator::success,
};
#[cfg(test)]
use rand::{Rng, SeedableRng};
pub use tpe::*;
mod attr;
pub use attr::*;
mod ctype;
pub use ctype::*;
mod locals;
pub use locals::*;
mod body;
pub use body::*;
mod builder;
pub use builder::*;
#[qparse_macros::qparse("PlaceHolder")]
#[derive(Default, PartialEq, Eq, Arbitrary, Clone, Debug)]
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
#[qparse_macros::qparse(
    "; source {file}:{line}:{col}
"
)]
#[derive(Clone)]
pub(crate) struct SourceLocationInner {
    file: GlobalIdent,
    line: u32,
    col: u32,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, PartialEq, Eq, Debug)]
pub enum Operand {
    #[qparse("%v{0}")]
    SSA(u32),
    #[qparse("{0}")]
    Global(GlobalIdent),
}
pub(crate) fn comment(input: &str) -> IResult<&str, ()> {
    use nom::Parser;
    alt((
        (multispace0, tag(";"), take_until("\n"), tag("\n")).map(|_| ()),
        (take_until("\n"), tag("\n")).map(|_| ()),
    ))
    .parse(input)
}
#[cfg(test)]
pub fn arbitrary<T: for<'a> Arbitrary<'a>>(f: impl Fn(T), og_iters: usize) {
    let mut rng = rand::rngs::SmallRng::from_seed(*b"THIS IS A SEED. IT SEEDS THE RNG");
    let mut buff = vec![0; 1024];
    let mut c = 0;
    let mut iters = og_iters;
    while iters > 0 {
        rng.fill_bytes(&mut buff);
        let mut u = Unstructured::new(&buff);
        c += 1;
        if c > og_iters * 1024 {
            panic!("arbitrary gen loop stuck?? {iters} {c}")
        }
        let Ok(t) = T::arbitrary(&mut u) else {
            continue;
        };
        iters -= 1;
        f(t);
    }
}
