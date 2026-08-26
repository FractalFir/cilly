use crate::{AtomOrdering, AtomicRmwOp, Label, Local, Operand, ScalarTy, Termiantor, Type};
use std::num::NonZeroU32;

// thin erorr type - peps not meant to inspect this, only get the error message.
#[derive(Debug)]
#[allow(private_interfaces)]
pub enum BuilderError {
    AllocaAInvalidAlign {
        align: NonZeroU32,
    },
    BlockLabelInvalid {
        block: Label,
    },
    PosOutOfRange {
        last_pos: usize,
        pos: usize,
    },
    BuilderPosNotSet,
    IntOpTypeNotIntOrVecInt {
        ty: Type,
    },
    BlockAlreadyTerminated {
        bb: Label,
        old_term: Termiantor,
        new_term: Termiantor,
    },
    VoidRetInNonVoidFnc,
    InvalidSassOperand {
        operand: Operand,
    },
    IntOpOperandNotIntOrVecInt {
        ty: Type,
    },
    BinopTypeMismatch {
        lhs_ty: Type,
        rhs_ty: Type,
    },
    ParamOutOfRange {
        param: u32,
        len: u32,
    },
    RetTypeMismatch {
        got: Type,
        expected: Type,
    },
    VoidLocal,
    ConstIntWhereNonIntExpected {
        hint_ty: Type,
    },
    ConstIntOutOfRange {
        val: i128,
        bitwidth: std::num::NonZero<u8>,
    },
    CondBrCondNotI1 {
        cond: Operand,
        got: Type,
    },
    InvalidSelCond {
        expected: Type,
        got: Type,
    },
    SelInputTypeMismatch {
        lhs: Type,
        rhs: Type,
        expected: Type,
    },
    LocalInvalid {
        idx: Local,
    },
    StoreLocalTypeMismatch {
        local: Local,
        got: Type,
        expected: Type,
    },
    FloatOpTypeNotFloatOrVecInt {
        ty: Type,
    },
    FloatOpOperandNotFloatOrVecInt {
        ty: Type,
    },
    InvalidCastSrc {
        expected: Type,
        got: Type,
    },
    Int2IntCastOutputNotInt {
        output: Type,
    },
    Int2IntCastInputNotInt {
        input: Type,
    },
    Float2FloatCastInputNotFloat {
        input: Type,
    },
    Float2FloatCastOutputNotFloat {
        output: Type,
    },
    Int2FloatCastOutputNotFloat {
        output: Type,
    },
    Int2FloatCastInputNotInt {
        input: Type,
    },
    Int2PtrCastOutputNotPtr {
        output: Type,
    },
    Int2PtrCastInputNotInt {
        input: Type,
    },
    Ptr2IntCastOutputNotInt {
        output: Type,
    },
    Ptr2IntCastInputNotPtr {
        input: Type,
    },
    BitcastSizeMismatch {
        src_size: u32,
        dst_size: u32,
    },
    Float2IntCastOutputNotInt {
        output: Type,
    },
    Float2IntCastInputNotFloat {
        input: Type,
    },
    LoadTyVoid,
    LoadAddrNotPtr {
        ptr: Operand,
        ptr_ty: Type,
    },
    MemAccessAlignNotPowerOf2 {
        align: std::num::NonZero<u32>,
    },
    AtomicLoadInvalidOrdering {
        ordering: AtomOrdering,
    },
    StoreAddrNotPtr {
        ptr: Operand,
        ptr_ty: Type,
    },
    StoreTyVoid,
    AtomicStoreInvalidOrdering {
        ordering: AtomOrdering,
    },
    AtomicRmwAddrNotPtr {
        ptr: Operand,
        ptr_ty: Type,
    },
    NonFloatInAtomicRMWFloatOp {
        op: AtomicRmwOp,
        ty: Type,
    },
    NonIntInAtomicRMWIntOp {
        op: AtomicRmwOp,
        ty: Type,
    },
    AtomicRmwWrongValType {
        val_ty: Type,
        val: Operand,
        ty: Type,
    },
    MemMoveDestNotPtr {
        dst_ty: Type,
    },
    MemMoveSrcNotPtr {
        src_ty: Type,
    },
    MemMoveLenNotInt {
        len_ty: Type,
        len: Operand,
    },
    MemMoveLenTyNotInt {
        len_ty: Type,
    },
    MemCpyDestNotPtr {
        dst_ty: Type,
    },
    MemCpySrcNotPtr {
        src_ty: Type,
    },
    MemCpyLenTyNotInt {
        len_ty: Type,
    },
    MemCpyLenNotInt {
        len_ty: Type,
        len: Operand,
    },
    MemSetDestNotPtr {
        dst_ty: Type,
    },
    MemSetLenTyNotInt {
        len_ty: Type,
    },
    MemSetLenNotInt {
        len_ty: Type,
        len: Operand,
    },
    MemSetValNotInt {
        val_ty: Type,
        val: Operand,
    },
    CalleeNotPtrOrFn {
        callee: Operand,
        calle_ty: Type,
    },
    PtrOffsetOffsetOperandWrongType {
        off: Operand,
        off_ty: Type,
        got: Type,
    },
    PtrOffsetOffsetWrongType {
        off_ty: Type,
    },
    PtrOffsetPtrIsNotPtr {
        ptr_ty: Type,
        ptr: Operand,
    },
    ExtractValueAggregateTyNotStruct {
        aggregate_ty: Type,
    },
    FieldIndexOOB {
        aggregate_ty: Type,
        index: u64,
    },
    IntOpTypeNotInt {
        ty: Type,
    },
    IntOpOperandNot {
        ty: Type,
    },
    SwitchValTyWtong {
        val_ty: Type,
        val: Operand,
    },
    BswapByteSizeNotEven {
        bitsize: u32,
        val: Operand,
    },
    InsertElementArgNotVec { vector_ty: Type },
    InsertElementArgNotVecOfVecTy { expected: Type, got: ScalarTy },
    InsertElementElementOperandWrongTy { expected: Type, got: &Type },
}
