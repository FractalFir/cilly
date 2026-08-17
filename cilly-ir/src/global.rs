use std::{fmt::Write, num::NonZeroU32};

use crate::{GlobalIdent, Linkage, ModuleBuilderError};

#[qparse_macros::qparse("")]
pub(crate) enum GlobalKind {
    #[qparse("global")]
    Global,
    #[qparse("constant")]
    Constant,
}
#[qparse_macros::qparse("{name} = {linkage} {kind} {initializer}, align {align}")]
pub(crate) struct Global {
    pub(crate) name: GlobalIdent,
    pub(crate) linkage: Linkage,
    pub(crate) kind: GlobalKind,
    pub(crate) initializer: ConstInit,
    pub(crate) align: NonZeroU32,
}
#[qparse_macros::qparse("")]
#[derive(Debug)]
enum ConstInitFrag {
    #[qparse("ptr {0}")]
    Ptr(GlobalIdent),
    #[qparse("c\"{0}\"")]
    ByteRun(ByteRun),
}
#[derive(Debug)]
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
                write!(f, "\\{b:x}").unwrap();
            }
        }
        f.write_char('"')?;
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for ByteRun {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        todo!()
    }
}
#[derive(Debug)]
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
        todo!()
    }
}

impl ConstInit {
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
        &Global {
            name: GlobalIdent::new("hello_func🦆").unwrap(),
            linkage: Linkage::External,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(b"FELLING CUTE :3 \xF0\x9F\xA6\x86".to_vec(), vec![], 8)
                .unwrap(),
            align: NonZeroU32::new(128).unwrap()
        }
        .to_string(),
        "@\"hello_func\\F0\\9F\\A6\\86\" = external constant [20 x i8] c\"FELLING CUTE :3 \\f0\\9f\\a6\\86\", align 128"
    )
}
#[test]
fn ptr_global() {
    assert_eq!(
        &Global {
            name: GlobalIdent::new("DUCKS_PTR🦆").unwrap(),
            linkage: Linkage::Internal,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(
                b"\0\0\0\0\0\0\0\0".to_vec(),
                vec![(0, GlobalIdent::new("DUCK").unwrap())],
                8
            )
            .unwrap(),
            align: NonZeroU32::new(8).unwrap()
        }
        .to_string(),
        "@\"DUCKS_PTR\\F0\\9F\\A6\\86\" = internal constant ptr @DUCK, align 8"
    )
}
#[test]
fn ptr_global_and_const() {
    assert_eq!(
        &Global {
            name: GlobalIdent::new("🦆DUCKS_PTR🦆").unwrap(),
            linkage: Linkage::Internal,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(
                b"12345678\0\0\0\0\0\0\0\0UWU FELLING ADORABLE :3".to_vec(),
                vec![(8, GlobalIdent::new("DUCK").unwrap())],
                8
            )
            .unwrap(),
            align: NonZeroU32::new(8).unwrap()
        }
        .to_string(),
        "@\"\\F0\\9F\\A6\\86DUCKS_PTR\\F0\\9F\\A6\\86\" = internal constant <{[8 x i8],ptr,[23 x i8]}> <{[8 x i8] c\"12345678\",ptr @DUCK,[23 x i8] c\"UWU FELLING ADORABLE :3\"}>, align 8"
    )
}
#[test]
fn extern_global() {
    assert_eq!(
        &Global {
            name: GlobalIdent::new("EXTERN_DUCKS_PTR🦆").unwrap(),
            linkage: Linkage::External,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(vec![], vec![], 0).unwrap(),
            align: NonZeroU32::new(8).unwrap()
        }
        .to_string(),
        "@\"EXTERN_DUCKS_PTR\\F0\\9F\\A6\\86\" = external constant {}, align 8"
    )
}
