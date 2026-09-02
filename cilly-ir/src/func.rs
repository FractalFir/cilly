use std::fmt::Write;

use arbitrary::Arbitrary;
use nom::{
    bytes::complete::tag, character::complete::multispace0, multi::separated_list0,
    sequence::delimited,
};
use qparse_macros::qparse;

use crate::{Attr, AttrList, Body, GlobalIdent, Linkage, SourceLocation, Type, locals::Locals};

/// Function Declaration or Definition.
#[qparse_macros::qparse("")]
#[derive(Clone, Debug)]
pub(crate) enum Fnc {
    #[qparse("{src_loc}declare {linkage}{output} {name}({inputs})")]
    Decl {
        src_loc: SourceLocation,
        linkage: Linkage,
        output: AttrAndTy,
        name: GlobalIdent,
        inputs: InputArgs,
    },
    #[qparse(
        "{src_loc}define {linkage}{output} {name}({inputs}){{
{locals}{body}}}"
    )]
    Def {
        src_loc: SourceLocation,
        linkage: Linkage,
        output: AttrAndTy,
        name: GlobalIdent,
        inputs: InputArgs,
        locals: Locals,
        body: Body,
    },
}
impl Fnc {
    pub(crate) fn is_def(&self) -> bool {
        matches!(self, Fnc::Def { .. })
    }
    pub(crate) fn name(&self) -> &GlobalIdent {
        match self {
            Self::Decl { name, .. } => name,
            Self::Def { name, .. } => name,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct InputArgs {
    pub(crate) args: Vec<TyAndAttr>,
    pub(crate) va_args: bool,
}
#[qparse("{attr}{ty}")]
#[derive(Clone, Debug, Arbitrary)]
pub(crate) struct AttrAndTy {
    pub(crate) attr: AttrList,
    pub(crate) ty: Type,
}
#[qparse("{ty} {attr}")]
#[derive(Clone, Debug, Arbitrary)]
pub struct TyAndAttr {
    pub(crate) attr: AttrList,
    pub(crate) ty: Type,
}
impl TyAndAttr {
    pub fn add_attr(&mut self, attr: Attr) {
        self.attr.add_attr(attr);
    }
}
impl From<Type> for TyAndAttr {
    fn from(ty: Type) -> Self {
        Self {
            ty,
            attr: AttrList::default(),
        }
    }
}
impl std::fmt::Display for InputArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, arg) in self.args.iter().enumerate() {
            if n != 0 {
                f.write_char(',')?;
            }
            write!(f, "{arg}%v{n}")?;
        }
        if self.va_args {
            if !self.args.is_empty() {
                f.write_char(',');
            }
            write!(f, "...")?;
        }
        Ok(())
    }
}

impl qparse::Parseable<qparse::Display> for InputArgs {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        use nom::Parser;
        separated_list0(
            tag(","),
            delimited(
                multispace0,
                (
                    <TyAndAttr as qparse::Parseable<qparse::Display>>::parse,
                    tag("%v"),
                    <u64 as qparse::Parseable<qparse::Display>>::parse,
                )
                    .map(|(ty, _, _)| ty),
                multispace0,
            ),
        )
        .map(|args| InputArgs {
            args,
            va_args: false,
        })
        .parse(input)
    }
}
#[test]
fn extern_global() {
    use crate::Attr;
    use std::num::NonZeroU8;
    let i8 = Type::ix(NonZeroU8::new(8).unwrap());
    assert_eq!(
        &Fnc::Decl {
            linkage: Linkage::External,
            name: GlobalIdent::new("HELLO").unwrap(),
            inputs: InputArgs {
                args: vec![TyAndAttr {
                    attr: AttrList::new(vec![Attr::Sext]),
                    ty: i8.clone()
                }],
                va_args: false,
            },
            output: AttrAndTy {
                ty: i8.clone(),
                attr: AttrList::new(vec![Attr::Zext])
            },
            src_loc: Default::default(),
        }
        .to_string(),
        "declare external zext i8 @HELLO (i8 sext %v0)"
    );
    <Fnc as qparse::Parseable<qparse::Display>>::simple_parse(
        "declare external zext i8 @HELLO (i8 sext %v0)",
    )
    .unwrap();
    <Fnc as qparse::Parseable<qparse::Display>>::simple_parse(
        "define external zext i8 @HELLO (i8 sext %v0){ 
    %v5 = alloca i8, i32 8, align 8 ; this is an alloca :3
    %l6 = alloca i127
    ret void
}",
    )
    .unwrap();
    <Fnc as qparse::Parseable<qparse::Display>>::simple_parse(
        "; source @a:10:20 declare external zext i8 @HELLO (i8 sext %v0)",
    )
    .unwrap();
}
