use std::fmt::Write;

use arbitrary::Arbitrary;
use nom::{
    bytes::complete::tag, character::complete::multispace0, multi::separated_list0,
    sequence::delimited,
};
use qparse_macros::qparse;

use crate::{AttrList, Body, GlobalIdent, Linkage, SourceLocation, Type, locals::Locals};

/// Function Declaration or Definition.
#[qparse_macros::qparse("")]
#[derive(Clone)]
pub(crate) enum Fnc {
    #[qparse("{src_loc}declare {linkage} {output} {name} ({inputs})")]
    Decl {
        src_loc: SourceLocation,
        linkage: Linkage,
        output: AttrAndTy,
        name: GlobalIdent,
        inputs: InputArgs,
    },
    #[qparse(
        "{src_loc}define {linkage} {output} {name} ({inputs}){{
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
#[derive(Clone)]
pub(crate) struct InputArgs {
    pub(crate) args: Vec<TyAndAttr>,
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

impl std::fmt::Display for InputArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, arg) in self.args.iter().enumerate() {
            if n != 0 {
                f.write_char(',')?;
            }
            write!(f, "{arg}%v{n}")?;
        }
        Ok(())
    }
}

impl qparse::Parseable<qparse::Display> for InputArgs {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
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
        .map(|args| InputArgs { args })
        .parse(input)
    }
}
#[test]
fn extern_global() {
    use crate::Attr;
    use std::num::NonZeroU8;
    let i32 = Type::Int {
        bitwidth: NonZeroU8::new(8).unwrap(),
    };
    assert_eq!(
        &Fnc::Decl {
            linkage: Linkage::External,
            name: GlobalIdent::new("HELLO").unwrap(),
            inputs: InputArgs {
                args: vec![TyAndAttr {
                    attr: AttrList::new(vec![Attr::Sext]),
                    ty: i32.clone()
                }]
            },
            output: AttrAndTy {
                ty: i32.clone(),
                attr: AttrList::new(vec![Attr::Zext])
            },
            src_loc: Default::default(),
        }
        .to_string(),
        "declare external zext i8 @HELLO (i8 sext %v0)"
    );
    <Fnc as qparse::Parseable<qparse::Display>>::parse(
        "declare external zext i8 @HELLO (i8 sext %v0)",
    )
    .unwrap();
    <Fnc as qparse::Parseable<qparse::Display>>::parse(
        "define external zext i8 @HELLO (i8 sext %v0){ 
    %v5 = alloca i8, i32 8, align 8 ; this is an alloca :3
    %l6 = alloca i127
    ret void
}",
    )
    .unwrap();
    <Fnc as qparse::Parseable<qparse::Display>>::parse(
        "; source @a:10:20 declare external zext i8 @HELLO (i8 sext %v0)",
    )
    .unwrap();
}
