use arbitrary::{Arbitrary, Unstructured};
use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::satisfy,
    combinator::{map, map_res},
    multi::many0,
    sequence::preceded,
};
use std::fmt::Write;
use traversable::{Traversable, TraversableMut};
#[derive(Clone, PartialEq, Eq, Debug, Hash,Traversable, TraversableMut)]
pub struct GlobalIdent {
    #[traverse(skip)]
    name: String,
}
impl<'a> Arbitrary<'a> for GlobalIdent {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut name: String = u.arbitrary()?;
        if name.is_empty() {
            name.push('a');
        }
        Ok(Self { name })
    }
}
impl GlobalIdent {
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if name.is_empty() {
            None
        } else {
            Some(Self { name })
        }
    }
}
const IDENT_CHARS: &str = "abcdefghijklmnoprstuwxyzvqABCDEFGHIJKLMNOPRSTUWXYZVQ$._";
impl std::fmt::Display for GlobalIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('@')?;
        let needs_escape = self
            .name
            .chars()
            .enumerate()
            .any(|(idx, c)| !(IDENT_CHARS.contains(c) || (c.is_ascii_digit() && idx != 0)));
        if needs_escape {
            f.write_char('"')?;
        }
        for (idx, c) in self.name.chars().enumerate() {
            if IDENT_CHARS.contains(c) || (c.is_ascii_digit() && idx != 0) {
                f.write_char(c)?;
            } else {
                write_escaped(f, c)?;
            }
        }
        if needs_escape {
            f.write_char('"')?;
        }
        Ok(())
    }
}
fn write_escaped(f: &mut std::fmt::Formatter<'_>, c: char) -> std::fmt::Result {
    let mut buf = [0u8; 4];
    for b in c.encode_utf8(&mut buf).bytes() {
        write!(f, "\\{b:02X}")?;
    }
    Ok(())
}
impl qparse::Parseable<qparse::Display> for GlobalIdent {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        (
            tag("@"),
            alt((
                (ident_char(true), many0(ident_char(false))),
                (
                    tag("\""),
                    ident_char(true),
                    many0(ident_char(false)),
                    tag("\""),
                )
                    .map(|(_, a, b, _)| (a, b)),
            )),
        )
            .map(|(_, (prefix, body))| {
                std::iter::once(prefix)
                    .chain(body)
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .map_opt(|v| String::from_utf8(v).ok().map(|name| Self { name }))
            .parse(input)
    }
}
// Parse a byte in the \xx LLVM form.
fn escaped_byte<'a, E>(input: &'a str) -> nom::IResult<&'a str, u8, E>
where
    E: qparse::QParseError<'a>,
{
    nom::combinator::map_parser(
        preceded(tag("\\"), take(2usize)),
        nom::combinator::all_consuming(<u8 as qparse::Parseable<qparse::LowerHex>>::parse),
    )
    .parse(input)
}
/// Parser for individual identifier parts.
fn ident_char<'a, E>(is_first: bool) -> impl FnMut(&'a str) -> nom::IResult<&'a str, Vec<u8>, E>
where
    E: qparse::QParseError<'a>,
{
    move |input| {
        alt((
            // Either this is an escaped byte
            map(escaped_byte, |b| vec![b]),
            // or a raw one.
            map(
                satisfy(move |c| IDENT_CHARS.contains(c) || (c.is_ascii_digit() && !is_first)),
                |c| {
                    let mut buf = [0u8; 4];
                    c.encode_utf8(&mut buf).as_bytes().to_vec()
                },
            ),
        ))
        .parse(input)
    }
}
