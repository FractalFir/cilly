use std::num::NonZeroU32;

use nom::{Parser, character::complete::multispace0, multi::many0};

use crate::{Local, SSAVal, Type, comment};
#[qparse_macros::qparse("{ssa_id} = alloca i8, i32 {size}, align {align}")]
#[derive(Clone, Debug)]
pub(crate) struct AllocA {
    pub(crate) ssa_id: SSAVal,
    pub(crate) size: NonZeroU32,
    pub(crate) align: NonZeroU32,
}
#[qparse_macros::qparse("{local} = alloca {ty}")]
#[derive(Clone, Debug)]
pub(crate) struct LocalDef {
    pub(crate) local: Local,
    pub(crate) ty: Type,
}
#[derive(Clone, Debug)]
pub(crate) struct Locals {
    /// alloca - stack allocation, whose address can be taken.
    pub(crate) allocas: Vec<AllocA>,
    /// local - typed local var, whose address can't be taken. Used for phis.
    pub(crate) locals: Vec<LocalDef>,
}

impl Locals {
    pub(crate) fn empty() -> Self {
        Self {
            allocas: vec![],
            locals: vec![],
        }
    }
    pub(crate) fn add_local(&mut self, ty: Type) -> Local {
        let id = self.locals.len() as u32;
        let local = Local { id };
        self.locals.push(LocalDef { local, ty });
        local
    }
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
