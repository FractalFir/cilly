use arbitrary::Arbitrary;

use crate::{AttrAndTy, Local, Operand, PlaceHolder, SSAVal, Type};
pub(crate) type CallArgs = PlaceHolder;
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, Arbitrary)]
pub(crate) enum Binop {
    #[qparse("add")]
    Add,
    #[qparse("sub")]
    Sub,
    #[qparse("mul")]
    Mul,
    #[qparse("udiv")]
    UDiv,
    #[qparse("sdiv")]
    SDiv,
    #[qparse("urem")]
    URem,
    #[qparse("srem")]
    SRem,
    #[qparse("shl")]
    Shl,
    #[qparse("lshr")]
    LShr,
    #[qparse("ashr")]
    AShr,
    #[qparse("and")]
    And,
    #[qparse("or")]
    Or,
    #[qparse("xor")]
    Xor,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, Arbitrary)]
pub(crate) enum ICmp {
    #[qparse("eq")]
    Eq,
    #[qparse("ne")]
    Ne,
    #[qparse("ugt")]
    UGt,
    #[qparse("uge")]
    UGe,
    #[qparse("ult")]
    ULt,
    #[qparse("ule")]
    ULe,
    #[qparse("sgt")]
    SGt,
    #[qparse("sge")]
    SGe,
    #[qparse("slt")]
    SLt,
    #[qparse("sle")]
    SLe,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary)]
pub(crate) enum Instruction {
    #[qparse("{dst} = icmp {cmp} {ty} {lhs}, {rhs}")]
    ICmp {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        cmp: ICmp,
    },
    #[qparse("{dst} = {op} {ty} {lhs}, {rhs}")]
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
    #[qparse("{dst} = load {ty}, ptr {local}")]
    LoadLocal { dst: SSAVal, local: Local, ty: Type },
    #[qparse("store {ty} {val}, ptr {local}")]
    StoreLocal {
        local: Local,
        ty: Type,
        val: Operand,
    },
    #[qparse("{dst} = select {sel_ty} {cond}, {ty} {then}, {ty} {els}")]
    Select {
        dst: SSAVal,
        cond: Operand,
        ty: Type,
        then: Operand,
        els: Operand,
        sel_ty: Type,
    },
}
