use std::num::NonZeroU32;

use arbitrary::Arbitrary;

use crate::{AttrAndTy, Intrinsic, Local, Operand, PlaceHolder, SSAVal, Type};
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
    #[qparse("fadd")]
    FAdd,
    #[qparse("fsub")]
    FSub,
    #[qparse("fmul")]
    FMul,
    #[qparse("fdiv")]
    FDiv,
    #[qparse("frem")]
    FRem,
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
#[derive(Clone, Copy, Debug, Arbitrary)]
pub(crate) enum CastOp {
    #[qparse("trunc")]
    Trunc,
    #[qparse("zext")]
    ZExt,
    #[qparse("sext")]
    SExt,
    #[qparse("fptrunc")]
    FPTrunc,
    #[qparse("fpext")]
    FPExt,
    #[qparse("fptoui")]
    FPToUI,
    #[qparse("fptosi")]
    FPToSI,
    #[qparse("uitofp")]
    UIToFP,
    #[qparse("sitofp")]
    SIToFP,
    #[qparse("ptrtoint")]
    PtrToInt,
    #[qparse("inttoptr")]
    IntToPtr,
    #[qparse("bitcast")]
    BitCast,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, Arbitrary)]
pub(crate) enum FCmp {
    #[qparse("oeq")]
    OEq,
    #[qparse("ogt")]
    OGt,
    #[qparse("oge")]
    OGe,
    #[qparse("olt")]
    OLt,
    #[qparse("ole")]
    OLe,
    #[qparse("one")]
    ONe,
    #[qparse("ueq")]
    UEq,
    #[qparse("ugt")]
    UGt,
    #[qparse("uge")]
    UGe,
    #[qparse("ult")]
    ULt,
    #[qparse("ule")]
    ULe,
    #[qparse("une")]
    UNe,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Arbitrary)]
pub enum AtomicRmwOp {
    #[qparse("xchg")]
    Xchg,
    #[qparse("add")]
    Add,
    #[qparse("sub")]
    Sub,
    #[qparse("and")]
    And,
    #[qparse("nand")]
    Nand,
    #[qparse("or")]
    Or,
    #[qparse("xor")]
    Xor,
    #[qparse("max")]
    Max,
    #[qparse("min")]
    Min,
    #[qparse("umax")]
    UMax,
    #[qparse("umin")]
    UMin,
    #[qparse("fadd")]
    FAdd,
    #[qparse("fsub")]
    FSub,
    #[qparse("fmax")]
    FMax,
    #[qparse("fmin")]
    FMin,
}
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Arbitrary)]
pub enum AtomOrdering {
    #[qparse("unordered")]
    Unordered,
    #[qparse("monotonic")]
    Monotonic,
    #[qparse("acquire")]
    Acquire,
    #[qparse("release")]
    Release,
    #[qparse("acq_rel")]
    AcqRel,
    #[qparse("seq_cst")]
    SeqCst,
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
    #[qparse("{dst} = fcmp {cmp} {ty} {lhs}, {rhs}")]
    FCmp {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        cmp: FCmp,
    },
    #[qparse("{dst} = {op} {ty} {lhs}, {rhs}")]
    Binop {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        op: Binop,
    },
    #[qparse("{dst} = {op} {src_ty} {val} to {dst_ty}")]
    Cast {
        dst: SSAVal,
        op: CastOp,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    },
    #[qparse("call void {callee}({call_args})")]
    VoidCall {
        callee: Operand,
        call_args: CallArgs,
    },
    #[qparse("{dst} = call {intrinsic}")]
    CallIntrinsic { dst: SSAVal, intrinsic: Intrinsic },
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
    #[qparse("{dst} = select {cond_ty} {cond}, {ty} {then}, {ty} {els}")]
    Select {
        dst: SSAVal,
        cond: Operand,
        ty: Type,
        then: Operand,
        els: Operand,
        cond_ty: Type,
    },
    #[qparse("{dst} = getelementptr{inbounds:present( inbounds)} i8, ptr {ptr}, {off_ty} {off}")]
    PtrOffset {
        dst: SSAVal,
        ptr: Type,
        off_ty: Type,
        off: Operand,
        inbounds: bool,
    },
    #[qparse("{dst} = load{volatile:present( volatile)} {ty}, ptr {ptr}, align {align}")]
    Load {
        dst: SSAVal,
        ptr: Operand,
        ty: Type,
        align: NonZeroU32,
        volatile: bool,
    },
    #[qparse("{dst} = load atomic {ty}, ptr {ptr} {ordering}, align {align}")]
    LoadAtomic {
        dst: SSAVal,
        ptr: Operand,
        ty: Type,
        align: NonZeroU32,
        ordering: AtomOrdering,
    },
    #[qparse("store{volatile:present( volatile)} {ty} {val}, ptr {ptr}, align {align}")]
    Store {
        ptr: Operand,
        ty: Type,
        val: Operand,
        align: NonZeroU32,
        volatile: bool,
    },
    #[qparse("store atomic {ty} {val}, ptr {ptr} {ordering}, align {align}")]
    StoreAtomic {
        ptr: Operand,
        ty: Type,
        val: Operand,
        align: NonZeroU32,
        ordering: AtomOrdering,
    },
    #[qparse("{dst} = atomicrmw {op} ptr {ptr}, {ty} {val} {ordering}, align {align}")]
    AtomicRmw {
        dst: SSAVal,
        op: AtomicRmwOp,
        ptr: Operand,
        ty: Type,
        val: Operand,
        ordering: AtomOrdering,
        align: NonZeroU32,
    },
}
