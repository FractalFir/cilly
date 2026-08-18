use arbitrary::Arbitrary;
use nom::{
    Parser,
    branch::alt,
    bytes::{complete::take, tag},
    character::complete::satisfy,
    combinator::{map, map_res},
    multi::many0,
    sequence::preceded,
};
use std::fmt::Write;
#[derive(Clone, Arbitrary, PartialEq, Eq, Debug)]
pub struct GlobalIdent {
    name: String,
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
const IDENT_CHARS: &str = "abcdefghijklmnoprstuwxyzABCDEFGHIJKLMNOPRSTUWXYZ$._";
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
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        (tag("@"), ident_char(true), many0(ident_char(false)))
            .map(|(_, prefix, body)| {
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
fn escaped_byte(input: &str) -> nom::IResult<&str, u8> {
    map_res(
        preceded(tag("\\"), take(2usize)),
        <u8 as qparse::Parseable<qparse::Display>>::parse,
    )
    .map(|(_, val)| val)
    .parse(input)
}
/// Parser for individual identifier parts.
fn ident_char<'a>(is_first: bool) -> impl FnMut(&'a str) -> nom::IResult<&'a str, Vec<u8>> {
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
