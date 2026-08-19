use arbitrary::Arbitrary;

use crate::{AttrAndTy, Operand, PlaceHolder, SSAVal, Type};
pub type CallArgs = PlaceHolder;
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, Arbitrary)]
pub(crate) enum Binop {
    #[qparse("add")]
    Add,
    #[qparse("sub")]
    Sub,
    #[qparse("mul")]
    Mul,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary)]
pub(crate) enum Instruction {
    #[qparse("{dst} = {op} {ty} {lhs} {rhs}")]
    Binop {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        op: Binop,
    },
    #[qparse("call void {callee}({call_args})")]
    VoidCall {
        callee: Operand,
        call_args: CallArgs,
    },
    #[qparse("{dst} = call {output} {callee}({call_args})")]
    Call {
        dst: SSAVal,
        output: AttrAndTy,
        callee: Operand,
        call_args: CallArgs,
    },
}
