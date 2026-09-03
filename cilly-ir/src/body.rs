use arbitrary::Arbitrary;
use nom::{Parser, character::complete::multispace0, multi::many1};
use traversable::{Traversable, TraversableMut};

use crate::{Constant, Instruction, Legalzer, Operand, Type, comment};
#[derive(Clone, Debug, Traversable, TraversableMut)]
pub(crate) struct InstrList {
    pub(crate) instrs: Vec<Instruction>,
}
impl<'a> Arbitrary<'a> for InstrList {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut instrs = vec![Instruction::arbitrary(u)?];
        instrs.extend(
            u.arbitrary_iter::<Instruction>()?
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(Self { instrs })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(
            <Instruction as Arbitrary>::size_hint(depth),
            <Vec<Instruction> as Arbitrary>::size_hint(depth),
        )
    }
}
impl std::fmt::Display for InstrList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instr in &self.instrs {
            writeln!(f, "{instr}")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for InstrList {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        many1(
            (
                multispace0,
                nom::combinator::not(nom::bytes::complete::tag("call void @llvm.trap()")),
                Instruction::parse,
                comment,
            )
                .map(|(_, _, i, _)| i),
        )
        .map(|instrs| Self { instrs })
        .parse(input)
    }
}

#[derive(Copy, Clone, Arbitrary, PartialEq, Eq, Debug, Traversable, TraversableMut)]
#[qparse_macros::qparse("%l{id:x}")]
pub struct Local {
    pub(crate) id: u32,
}
#[derive(Copy, Clone, Arbitrary, PartialEq, Eq, Debug, Hash, Traversable, TraversableMut)]
#[qparse_macros::qparse("l{id:x}")]
pub struct Label {
    pub(crate) id: u32,
}
#[derive(Clone, Debug, Traversable, TraversableMut)]
pub(crate) struct Body {
    pub(crate) elems: Vec<CFGElem>,
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for elem in &self.elems {
            writeln!(f, "{elem}")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for Body {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        many1(
            (
                multispace0,
                <CFGElem as qparse::Parseable<qparse::Display>>::parse,
                multispace0,
            )
                .map(|(_, e, _)| e),
        )
        .map(|elems| Body { elems })
        .parse(input)
    }
}
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, Debug, Traversable, TraversableMut)]
pub(crate) enum CFGElem {
    #[qparse("ret {ty} {operand}")]
    Return { ty: Type, operand: Operand },
    #[qparse("ret void")]
    VoidRet,
    #[qparse("{instrs}")]
    Instructions { instrs: InstrList },
    #[qparse(
        "br i1 {cond}, label %if_body_{label}, label %if_join_{label}
if_body_{label}:
{body}br label %if_join_{label}
if_join_{label}:"
    )]
    If {
        cond: Operand,
        label: Label,
        body: Body,
    },
    #[qparse(
        "br i1 {cond}, label %if_then_{label}, label %if_else_{label}
if_then_{label}:
{then}br label %if_join_{label}
if_else_{label}:
{els}br label %if_join_{label}
if_join_{label}:"
    )]
    Elif {
        cond: Operand,
        label: Label,
        then: Body,
        els: Body,
    },
    #[qparse(
        "%cond_{label} = load i1, ptr {cond}
br i1 %cond_{label}, label %loop_body_{label}, label %loop_exit_{label}
loop_body_{label}:
{body}%cond2_{label} = load i1, ptr {cond}
br i1 %cond2_{label}, label %loop_body_{label}, label %loop_exit_{label}
loop_exit_{label}:"
    )]
    Loop {
        cond: Local,
        label: Label,
        body: Body,
    },
    #[qparse(
        "br label %dowhile_body_{label}
dowhile_body_{label}:
{body}br i1 {cond}, label %dowhile_body_{label}, label %dowhile_exit_{label}
dowhile_exit_{label}:"
    )]
    DoWhile {
        cond: Operand,
        label: Label,
        body: Body,
    },
    #[qparse("{0}")]
    Switch(Switch),
    #[qparse(
        "call void @llvm.trap()
unreachable"
    )]
    Trap,
}
impl<'a> Arbitrary<'a> for Body {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut elems = vec![CFGElem::arbitrary(u)?];
        elems.extend(
            u.arbitrary_iter::<CFGElem>()?
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(Body { elems })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(
            <CFGElem as Arbitrary>::size_hint(depth),
            <Vec<CFGElem> as Arbitrary>::size_hint(depth),
        )
    }
}
#[derive(Clone, Arbitrary, Debug, Traversable, TraversableMut)]
pub(crate) struct Switch {
    pub(crate) default_label: Label,
    pub(crate) default: Body,
    pub(crate) ty: Type,
    pub(crate) cases: Vec<(Constant, Label, Body)>,
    pub(crate) val: Operand,
}
impl std::fmt::Display for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            default,
            default_label,
            ty,
            cases,
            val,
        } = self;
        writeln!(f, "switch {ty} {val}, label %default_{default_label} [")?;
        for (case, label, _) in cases {
            writeln!(f, "{ty} {case}, label %case_{label}")?;
        }
        writeln!(f, "]")?;
        for (_, label, body) in cases {
            writeln!(f, "case_{label}:")?;
            writeln!(f, "{body}br label %switch_{default_label}_join")?;
        }
        writeln!(f, "default_{default_label}:")?;
        writeln!(f, "{default}br label %switch_{default_label}_join")?;
        writeln!(f, "switch_{default_label}_join:")?;
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for Switch {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        nom::bytes::complete::tag("UNPARSABLE FOR NOW todo!() I AM LAZYYYY")
            .map(|_| todo!())
            .parse(input)
    }
}
#[test]
fn body_fmt() {
    use qparse::Parseable;
    println!(
        "{:?}",
        <Instruction as qparse::Parseable<qparse::Display>>::simple_parse("")
    );

    println!(
        "{:?}",
        <InstrList as qparse::Parseable<qparse::Display>>::simple_parse("")
    );
    crate::arbitrary::<Body>(
        |b| {
            let mut body_str = b.to_string();
            eprintln!("body_str:{body_str}");
            if let Err(err) = Body::simple_parse(&body_str) {
                panic!("{body_str} {err:?}");
            }
            let mut reparsed = Body::simple_parse(&body_str).unwrap().1.to_string();

            if reparsed != *body_str {
                while reparsed.chars().last() == body_str.chars().last() && !body_str.is_empty() {
                    reparsed.remove(reparsed.char_indices().next_back().unwrap().0);
                    body_str.remove(body_str.char_indices().next_back().unwrap().0);
                }
                while reparsed.chars().next() == body_str.chars().next() && !body_str.is_empty() {
                    reparsed.remove(0);
                    body_str.remove(0);
                }
            }
            assert_eq!(reparsed, body_str);
        },
        256,
        8,
    );
}
