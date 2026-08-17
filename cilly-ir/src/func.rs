use std::fmt::Write;

use crate::{GlobalIdent, Linkage, TyAndAttr};

/// Function Declaration or Definition.
#[qparse_macros::qparse("")]
pub(crate) enum Fnc {
    #[qparse("declare {linkage} {output} {name} ({inputs})")]
    Decl {
        linkage: Linkage,
        output: TyAndAttr,
        name: GlobalIdent,
        inputs: InputArgs,
    },
    #[qparse("define {linkage} {output} {name} ({inputs}){{}}")]
    Def {
        linkage: Linkage,
        output: TyAndAttr,
        name: GlobalIdent,
        inputs: InputArgs,
    },
}
pub(crate) struct InputArgs {
    args: Vec<TyAndAttr>,
}
impl std::fmt::Display for InputArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, arg) in self.args.iter().enumerate() {
            if n != 0 {
                f.write_char(',')?;
            }
            write!(f, "{arg}")?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for InputArgs {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        todo!()
    }
}
