use std::num::NonZeroU32;

use arbitrary::Arbitrary;
use traversable::{Traversable, TraversableMut};

use crate::{AttrAndTy, AttrList, Intrinsic, Local, Operand, SSAVal, TyAndAttr, Type};
#[derive(Clone, Debug, Arbitrary, Traversable, TraversableMut)]
pub(crate) struct CallArgs {
    pub(crate) args: Vec<(TyAndAttr, Operand)>,
}
impl std::fmt::Display for CallArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, arg) in self.args.iter().enumerate() {
            if n != 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}{}", arg.0, arg.1)?;
        }
        Ok(())
    }
}
impl qparse::Parseable<qparse::Display> for CallArgs {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: qparse::QParseError<'a>,
    {
        todo!()
    }
}
#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, Arbitrary, Traversable, TraversableMut)]
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
#[derive(Clone, Copy, Debug, Arbitrary, Traversable, TraversableMut)]
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
#[derive(Clone, Copy, Debug, Arbitrary, Traversable, TraversableMut)]
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
#[derive(Clone, Copy, Debug, Arbitrary, Traversable, TraversableMut)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Arbitrary, Traversable, TraversableMut)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Arbitrary, Traversable, TraversableMut)]
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
#[derive(Clone, Debug, Arbitrary, Traversable, TraversableMut)]
pub(crate) enum Instruction {
    #[qparse("{dst} = icmp {cmp:cut()} {ty:cut()} {lhs:cut()}, {rhs:cut()}")]
    ICmp {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        cmp: ICmp,
    },
    #[qparse("{dst} = fcmp {cmp:cut()} {ty:cut()} {lhs:cut()}, {rhs:cut()}")]
    FCmp {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        cmp: FCmp,
    },
    #[qparse("{dst} = {op} {ty:cut()} {lhs:cut()}, {rhs:cut()}")]
    Binop {
        dst: SSAVal,
        ty: Type,
        lhs: Operand,
        rhs: Operand,
        op: Binop,
    },
    #[qparse("{dst} = fneg {ty:cut()} {val:cut()}")]
    Fneg { dst: SSAVal, ty: Type, val: Operand },
    #[qparse("{dst} = {op} {src_ty:cut()} {val:cut()} to {dst_ty:cut()}")]
    Cast {
        dst: SSAVal,
        op: CastOp,
        src_ty: Type,
        val: Operand,
        dst_ty: Type,
    },
    #[qparse(
        "call void @llvm.memcpy.p0.p0.{len_ty:cut()}(ptr {dest:cut()}, ptr {src:cut()}, {len_ty:cut()} {len:cut()}, i1 {volatile:cut()})"
    )]
    MemCpy {
        dest: Operand,
        src: Operand,
        len_ty: Type,
        len: Operand,
        volatile: bool,
    },
    #[qparse(
        "call void @llvm.memmove.p0.p0.{len_ty:cut()}(ptr {dest:cut()}, ptr {src:cut()}, {len_ty:cut()} {len:cut()}, i1 {volatile:cut()})"
    )]
    MemMove {
        dest: Operand,
        src: Operand,
        len_ty: Type,
        len: Operand,
        volatile: bool,
    },
    #[qparse(
        "call void @llvm.memset.p0.{len_ty:cut()}(ptr {dest:cut()}, i8 {val:cut()}, {len_ty:cut()} {len:cut()}, i1 {volatile:cut()})"
    )]
    MemSet {
        dest: Operand,
        val: Operand,
        len_ty: Type,
        len: Operand,
        volatile: bool,
    },
    #[qparse("{dst} = call {intrinsic}")]
    CallIntrinsic { dst: SSAVal, intrinsic: Intrinsic },
    #[qparse("call void {callee:cut()}({call_args})")]
    VoidCall {
        callee: Operand,
        call_args: CallArgs,
    },
    #[qparse("{dst} = call {output} {callee:cut()}({call_args})")]
    Call {
        dst: SSAVal,
        output: AttrAndTy,
        callee: Operand,
        call_args: CallArgs,
    },
    #[qparse("{dst} = load {ty}, ptr {local}")]
    LoadLocal { dst: SSAVal, local: Local, ty: Type },
    #[qparse("store {ty} {val:cut()}, ptr {local}")]
    StoreLocal {
        local: Local,
        ty: Type,
        val: Operand,
    },
    #[qparse(
        "{dst} = select {cond_ty:cut()} {cond:cut()}, {ty:cut()} {then:cut()}, {ty:cut()} {els:cut()}"
    )]
    Select {
        dst: SSAVal,
        cond: Operand,
        ty: Type,
        then: Operand,
        els: Operand,
        cond_ty: Type,
    },
    #[qparse(
        "{dst} = getelementptr{inbounds:present( inbounds)} i8, ptr {ptr:cut()}, {off_ty:cut()} {off:cut()}"
    )]
    PtrOffset {
        dst: SSAVal,
        ptr: Operand,
        off_ty: Type,
        off: Operand,
        inbounds: bool,
    },
    #[qparse(
        "{dst} = load{volatile:present( volatile)} {ty}, ptr {ptr:cut()}, align {align:cut()}"
    )]
    Load {
        dst: SSAVal,
        ptr: Operand,
        ty: Type,
        #[traverse(skip)]
        align: NonZeroU32,
        volatile: bool,
    },
    #[qparse(
        "{dst} = load atomic {ty:cut()}, ptr {ptr:cut()} {ordering:cut()}, align {align:cut()}"
    )]
    LoadAtomic {
        dst: SSAVal,
        ptr: Operand,
        ty: Type,
        #[traverse(skip)]
        align: NonZeroU32,
        ordering: AtomOrdering,
    },
    #[qparse("store{volatile:present( volatile)} {ty} {val:cut()}, ptr {ptr}, align {align:cut()}")]
    Store {
        ptr: Operand,
        ty: Type,
        val: Operand,
        #[traverse(skip)]
        align: NonZeroU32,
        volatile: bool,
    },
    #[qparse(
        "store atomic {ty:cut()} {val:cut()}, ptr {ptr:cut()} {ordering:cut()}, align {align:cut()}"
    )]
    StoreAtomic {
        ptr: Operand,
        ty: Type,
        val: Operand,
        #[traverse(skip)]
        align: NonZeroU32,
        ordering: AtomOrdering,
    },
    #[qparse(
        "{dst} = cmpxchg {weak:present(weak )}ptr {ptr:cut()}, {ty:cut()} {expected:cut()}, {ty:cut()} {desired:cut()} {success:cut()} {failure:cut()}"
    )]
    AtomicCmpxchg {
        ty: Type,
        ptr: Operand,
        expected: Operand,
        desired: Operand,
        success: AtomOrdering,
        failure: AtomOrdering,
        weak: bool,
        dst: SSAVal,
    },
    #[qparse("fence {ordering:cut()}")]
    Fence { ordering: AtomOrdering },
    #[qparse(
        "{dst} = atomicrmw {op:cut()} ptr {ptr:cut()}, {ty:cut()} {val:cut()} {ordering:cut()}, align {align:cut()}"
    )]
    AtomicRmw {
        dst: SSAVal,
        op: AtomicRmwOp,
        ptr: Operand,
        ty: Type,
        val: Operand,
        ordering: AtomOrdering,
        #[traverse(skip)]
        align: NonZeroU32,
    },
    #[qparse("{dst} = extractvalue {aggregate_ty:cut()} {aggregate:cut()}, {index:cut()}")]
    ExtractValue {
        dst: SSAVal,
        aggregate_ty: Type,
        aggregate: Operand,
        index: u64,
    },
    #[qparse(
        "{dst} = insertvalue {aggregate_ty:cut()} {aggregate:cut()}, {value_ty:cut()} {element:cut()}, {index:cut()}"
    )]
    InsertValue {
        dst: SSAVal,
        aggregate_ty: Type,
        value_ty: Type,
        aggregate: Operand,
        element: Operand,
        index: u64,
    },
    #[qparse("{dst} = extractelement {vector_ty:cut()} {vector:cut()}, i32 {index:cut()}")]
    ExtractElement {
        dst: SSAVal,
        vector_ty: Type,
        vector: Operand,
        index: Operand,
    },
    #[qparse(
        "{dst} = insertelement {vector_ty:cut()} {vector:cut()}, {element_ty:cut()} {element:cut()}, i32 {index:cut()}"
    )]
    InsertElement {
        dst: SSAVal,
        vector_ty: Type,
        vector: Operand,
        element: Operand,
        element_ty: Type,
        index: Operand,
    },
}
impl Instruction {
    // Helper for calling functions in the fallback generator. 
    pub(crate) fn call_fnc(fnc: &crate::Fnc, args: &[Operand], dst: SSAVal) -> Self {
        let inputs = fnc.inputs();
        let inputs = &inputs.args;
        assert_eq!(args.len(), inputs.len());
        let args = args.iter().zip(inputs).map(|(o,ty)|(ty.clone(),o.clone())).collect();
        Self::Call {
            dst,
            output: AttrAndTy {
                attr: AttrList::default(),
                ty: fnc.output().ty.clone(),
            },
            callee: Operand::Global(fnc.name().clone()),
            call_args: CallArgs { args},
        }
    }
}
