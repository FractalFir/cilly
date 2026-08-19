use arbitrary::Arbitrary;

use crate::GlobalIdent;

#[qparse_macros::qparse("%v{0}")]
#[derive(Copy, Clone, Arbitrary, PartialEq, Eq, Debug)]
pub struct SSAVal(pub(crate) u32);
#[qparse_macros::qparse("")]
#[derive(Clone, Arbitrary, PartialEq, Eq, Debug)]
pub enum Operand {
    #[qparse("{0}")]
    SSA(SSAVal),
    #[qparse("{0}")]
    Global(GlobalIdent),
}
