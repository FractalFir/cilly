use arbitrary::Arbitrary;
use nom::{
    Parser,
    character::complete::multispace0,
    multi::{many0, many1},
    sequence::delimited,
};
use qparse::Parseable;

use crate::{Instruction, Operand, PlaceHolder, Type, comment};
#[derive(Clone, Debug)]
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
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        many1((multispace0, Instruction::parse, comment).map(|(_, i, _)| i))
            .map(|instrs| Self { instrs })
            .parse(input)
    }
}

pub(crate) type Local = PlaceHolder;
#[derive(Copy, Clone, Arbitrary, PartialEq, Eq, Debug)]
#[qparse_macros::qparse("l{id:x}")]
pub struct Label {
    pub(crate) id: u32,
}
#[derive(Clone, Debug)]
pub(crate) struct Body {
    elems: Vec<CFGElem>,
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
    fn parse(input: &str) -> nom::IResult<&str, Self> {
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
#[derive(Clone, Arbitrary, Debug)]
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

#[test]
fn body_fmt() {
    println!(
        "{:?}",
        <Instruction as Parseable<qparse::Display>>::parse("")
    );
    println!("{:?}", comment(""));
    println!("{:?}", <InstrList as Parseable<qparse::Display>>::parse(""));
    crate::arbitrary::<Body>(
        |b| {
            let mut body_str = b.to_string();
            eprintln!("body_str:{body_str}");
            if let Err(err) = Body::parse(&body_str) {
                panic!("{body_str} {err:?}");
            }
            let mut reparsed = Body::parse(&body_str).unwrap().1.to_string();

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
