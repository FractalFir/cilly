use arbitrary::Arbitrary;
#[cfg(test)]
use arbitrary::Unstructured;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::multispace0,
    combinator::success,
};
#[cfg(test)]
use rand::{Rng, SeedableRng};
mod func;
pub use func::*;
mod global;
pub(crate) use global::*;
mod module;
pub use module::*;
mod linkage;
pub use linkage::*;
mod global_ident;
pub use global_ident::*;
mod tpe;
pub use tpe::*;
mod attr;
pub use attr::*;
mod ctype;
pub use ctype::*;
mod locals;
pub(crate) use locals::*;
mod body;
pub use body::*;
mod builder;
pub use builder::*;
mod instr;
pub use instr::*;
mod operand;
pub use operand::*;
mod structurize;
pub(crate) use structurize::*;
mod intrincis;
pub(crate) use intrincis::*;
mod fallback;
pub use fallback::*;
use traversable::{Traversable, TraversableMut};
#[cfg(test)]
mod tests;
#[qparse_macros::qparse("PlaceHolder")]
#[derive(Default, PartialEq, Eq, Arbitrary, Clone, Debug)]
pub(crate) struct PlaceHolder;
#[derive(Clone, Default, Debug, Traversable, TraversableMut)]
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
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
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
#[derive(Clone, Debug, Traversable, TraversableMut)]
pub(crate) struct SourceLocationInner {
    file: GlobalIdent,
    line: u32,
    col: u32,
}

pub(crate) fn comment<'a, E>(input: &'a str) -> nom::IResult<&'a str, (), E>
where
    E: qparse::QParseError<'a>,
{
    use nom::Parser;
    alt((
        (multispace0, tag(";"), take_until("\n"), tag("\n")).map(|_| ()),
        (take_until("\n"), tag("\n")).map(|_| ()),
    ))
    .parse(input)
}
#[cfg(test)]
pub fn arbitrary<T: for<'a> Arbitrary<'a>>(f: impl Fn(T), og_iters: usize, budget: usize) {
    unstructured(
        |u| {
            f(T::arbitrary(u)?);
            Ok(())
        },
        og_iters,
        budget,
    )
}
#[cfg(test)]
pub fn unstructured(
    f: impl Fn(&mut Unstructured) -> arbitrary::Result<()>,
    og_iters: usize,
    budget: usize,
) {
    let mut rng = rand::rngs::SmallRng::from_seed(*b"THIS IS A SEED. IT SEEDS THE RNG");
    let mut buff = vec![0; budget];
    let mut c = 0;
    let mut iters = og_iters;
    while iters > 0 {
        rng.fill_bytes(&mut buff);
        let mut u = arbitrary::Unstructured::new(&buff);
        c += 1;
        if c > og_iters * 1024 {
            panic!("arbitrary gen loop stuck?? {iters} {c}")
        }
        let Ok(_) = f(&mut u) else {
            continue;
        };
        iters -= 1;
    }
}
