use std::num::NonZeroU32;

use nom::{Parser, character::complete::multispace0, multi::many0};

use crate::{Type, comment};
#[qparse_macros::qparse("%v{ssa_id} = alloca i8, i32 {size}, align {align}")]
pub(crate) struct AllocA {
    pub(crate) ssa_id: u32,
    pub(crate) size: NonZeroU32,
    pub(crate) align: NonZeroU32,
}
#[qparse_macros::qparse("%l{local_id} = alloca {ty}")]
pub(crate) struct LocalDef {
    pub(crate) local_id: u32,
    pub(crate) ty: Type,
}
pub(crate) struct Locals {
    /// alloca - stack allocation, whose address can be taken.
    pub(crate) allocas: Vec<AllocA>,
    /// local - typed local var, whose address can't be taken. Used for phis.
    pub(crate) locals: Vec<LocalDef>,
}
impl std::fmt::Display for Locals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for alloca in &self.allocas {
            writeln!(f, "{alloca}")?;
        }
        for local in &self.locals {
            writeln!(f, "{local}")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for Locals {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        (
            many0((multispace0, AllocA::parse, comment).map(|(_, a, __)| a)),
            many0((multispace0, LocalDef::parse, comment).map(|(_, l, __)| l)),
        )
            .map(|(allocas, locals)| Locals { allocas, locals })
            .parse(input)
    }
}
