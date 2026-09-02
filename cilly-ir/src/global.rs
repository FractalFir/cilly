use std::{fmt::Write, num::NonZeroU32};

use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::satisfy,
    combinator::{map, map_res, value},
    multi::many0,
    sequence::delimited,
};

use crate::{
    GlobalIdent,
    Linkage::{self, External},
    ModuleBuilderError,
};

/// This is a stupid fucking hack for LLVM being kind of STUPID here
/// and requiring the external linkage to be omitted for defintions,  
/// but not for declarations(w h y ???)
#[qparse_macros::qparse("")]
#[derive(Clone, Copy)]
pub(crate) enum GlobalDeclLinkage {
    #[qparse("external ")]
    External,
    #[qparse("extern_weak ")]
    ExternWeak,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Copy)]
pub(crate) enum GlobalKind {
    #[qparse("global")]
    Global,
    #[qparse("constant")]
    Constant,
}
#[derive(Clone)]
pub(crate) struct Section(pub(crate) Option<String>);
impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = &self.0 {
            write!(f, "section {s:?},")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for Section {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        todo!()
    }
}
impl Section {
    pub fn empty() -> Self {
        Self(None)
    }
}
#[qparse_macros::qparse("")]
#[derive(Clone)]
pub(crate) enum Global {
    #[qparse(
        "{name} = {linkage}{thread_local:present(thread_local )}{kind} {initializer},{link_section} align {align}"
    )]
    Def {
        name: GlobalIdent,
        linkage: Linkage,
        kind: GlobalKind,
        initializer: ConstInit,
        align: NonZeroU32,
        thread_local: bool,
        link_section: Section,
    },
    #[qparse(
        "{name} = {linkage}{thread_local:present(thread_local )}{kind} [0 x i8], {link_section} align {align}"
    )]
    Decl {
        linkage: GlobalDeclLinkage,
        name: GlobalIdent,
        kind: GlobalKind,
        align: NonZeroU32,
        thread_local: bool,
        link_section: Section,
    },
}
impl Global {
    pub(crate) fn is_def(&self) -> bool {
        matches!(self, Self::Def { .. })
    }
    pub(crate) fn name(&self) -> &GlobalIdent {
        match self {
            Global::Def { name, .. } | Global::Decl { name, .. } => name,
        }
    }
}
#[qparse_macros::qparse("")]
#[derive(Debug, Clone)]
enum ConstInitFrag {
    #[qparse("ptr {0}")]
    Ptr(GlobalIdent),
    #[qparse("{0}")]
    ByteRun(ByteRun),
}
#[derive(Debug, Clone)]
pub(crate) struct ByteRun {
    bytes: Vec<u8>,
}
impl std::fmt::Display for ByteRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("c\"")?;
        for b in &self.bytes {
            let c = *b as char;
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | ':' | ';') {
                f.write_char(c)?;
            } else {
                write!(f, "\\{b:02x}")?;
            }
        }
        f.write_char('"')?;
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for ByteRun {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        use nom::Parser;
        map(
            delimited(
                tag("c\""),
                many0(alt((
                    (
                        tag("\\"),
                        map_res(
                            take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit()),
                            |s: &str| u8::from_str_radix(s, 16),
                        ),
                    )
                        .map(|(_prefix, val)| val),
                    map(satisfy(|c| c.is_ascii() && c != '"' && c != '\\'), |c| {
                        c as u8
                    }),
                ))),
                tag("\""),
            ),
            |bytes| ByteRun { bytes },
        )
        .parse(input)
    }
}
#[derive(Debug, Clone)]
pub(crate) struct ConstInit {
    /// A constant initializer is composed of the following
    fragments: Vec<ConstInitFrag>,
}
impl std::fmt::Display for ConstInit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let [] = &self.fragments[..] {
            return write!(f, "{{}}");
        };
        if let [ConstInitFrag::ByteRun(run)] = &self.fragments[..] {
            return write!(f, "[{len} x i8] {run}", len = run.bytes.len());
        };
        if let [ConstInitFrag::Ptr(ident)] = &self.fragments[..] {
            return write!(f, "ptr {ident}");
        };
        f.write_str("<{")?;
        for (idx, frag) in (&self.fragments).iter().enumerate() {
            if idx != 0 {
                f.write_char(',')?;
            }
            match frag {
                ConstInitFrag::Ptr(_) => f.write_str("ptr"),
                ConstInitFrag::ByteRun(byte_run) => write!(f, "[{} x i8]", byte_run.bytes.len()),
            }?;
        }
        f.write_str("}> <{")?;
        for (idx, frag) in (&self.fragments).iter().enumerate() {
            if idx != 0 {
                f.write_char(',')?;
            }
            match frag {
                ConstInitFrag::Ptr(ident) => write!(f, "ptr {ident}"),
                ConstInitFrag::ByteRun(byte_run) => {
                    write!(f, "[{len} x i8] {byte_run}", len = byte_run.bytes.len())
                }
            }?;
        }
        f.write_str("}>")?;
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for ConstInit {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((value(Self { fragments: vec![] }, tag("{}")),)).parse(input)
    }
}

impl ConstInit {
    pub(crate) fn is_present(&self) -> bool {
        !self.fragments.is_empty()
    }
    pub(crate) fn new(
        bytes: Vec<u8>,
        mut refs: Vec<(u32, GlobalIdent)>,
        ptr_size: u32,
    ) -> Result<Self, ModuleBuilderError> {
        refs.sort_by_key(|(offset, _)| *offset);
        let mut biter = bytes.iter();
        let mut last_offset = 0;
        let mut fragments = Vec::new();
        for (offset, rf) in refs {
            let delta = offset - last_offset;
            let bytes: Vec<u8> = (0..delta).flat_map(|_| biter.next()).copied().collect();
            if bytes.len() != delta as _ {
                return Err(ModuleBuilderError::GlobalInitNotEnoughBytes);
            }
            if !bytes.is_empty() {
                fragments.push(ConstInitFrag::ByteRun(ByteRun { bytes }));
            }
            // Pop dem bytes
            if !(0..ptr_size).fold(true, |ok, _| ok & biter.next().is_some_and(|v| *v == 0)) {
                return Err(ModuleBuilderError::GlobalInitAddrByteNonzero);
            }
            fragments.push(ConstInitFrag::Ptr(rf));
            last_offset = offset + ptr_size;
        }
        let bytes: Vec<u8> = biter.copied().collect();
        if !bytes.is_empty() {
            fragments.push(ConstInitFrag::ByteRun(ByteRun { bytes: bytes }));
        }
        Ok(Self { fragments })
    }
}

#[test]
fn text_global() {
    assert_eq!(
        &Global::Def {
            name: GlobalIdent::new("hello_func🦆").unwrap(),
            linkage: Linkage::External,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(b"FELLING CUTE :3 \xF0\x9F\xA6\x86".to_vec(), vec![], 8)
                .unwrap(),
            align: NonZeroU32::new(128).unwrap(),
            thread_local: false,
            link_section: Section::empty(),
        }
        .to_string(),
        "@\"hello_func\\F0\\9F\\A6\\86\" = external constant [20 x i8] c\"FELLING CUTE :3 \\f0\\9f\\a6\\86\", align 128"
    )
}
#[test]
fn ptr_global() {
    assert_eq!(
        &Global::Def {
            name: GlobalIdent::new("DUCKS_PTR🦆").unwrap(),
            linkage: Linkage::Internal,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(
                b"\0\0\0\0\0\0\0\0".to_vec(),
                vec![(0, GlobalIdent::new("DUCK").unwrap())],
                8
            )
            .unwrap(),
            align: NonZeroU32::new(8).unwrap(),
            thread_local: false,
            link_section: Section::empty(),
        }
        .to_string(),
        "@\"DUCKS_PTR\\F0\\9F\\A6\\86\" = internal constant ptr @DUCK, align 8"
    )
}
#[test]
fn ptr_global_and_const() {
    assert_eq!(
        &Global::Def {
            name: GlobalIdent::new("🦆DUCKS_PTR🦆").unwrap(),
            linkage: Linkage::Internal,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(
                b"12345678\0\0\0\0\0\0\0\0UWU FELLING ADORABLE :3".to_vec(),
                vec![(8, GlobalIdent::new("DUCK").unwrap())],
                8
            )
            .unwrap(),
            align: NonZeroU32::new(8).unwrap(),
            thread_local: false,
            link_section: Section::empty(),
        }
        .to_string(),
        "@\"\\F0\\9F\\A6\\86DUCKS_PTR\\F0\\9F\\A6\\86\" = internal constant <{[8 x i8],ptr,[23 x i8]}> <{[8 x i8] c\"12345678\",ptr @DUCK,[23 x i8] c\"UWU FELLING ADORABLE :3\"}>, align 8"
    )
}
#[test]
fn extern_global() {
    assert_eq!(
        &Global::Decl {
            name: GlobalIdent::new("EXTERN_DUCKS_PTR🦆").unwrap(),
            kind: GlobalKind::Constant,
            align: NonZeroU32::new(8).unwrap(),
            thread_local: false,
            link_section: Section::empty(),
            linkage: GlobalDeclLinkage::External,
        }
        .to_string(),
        "@\"EXTERN_DUCKS_PTR\\F0\\9F\\A6\\86\" = constant {}, align 8"
    )
}
