use crate::{
    AtomOrdering, AtomicRmwOp, BuilderError, FunctionBuilder, I8_TY, I64_TY, Instruction, Operand,
    PTR_TY, Type,
};
use std::num::NonZeroU32;
impl FunctionBuilder {
    pub fn build_load(
        &mut self,
        ty: Type,
        ptr: Operand,
        align: NonZeroU32,
        volatile: bool,
    ) -> Result<Operand, BuilderError> {
        let ptr_ty = self.get_type(&ptr, &PTR_TY)?;
        if !ptr_ty.is_ptr() {
            return Err(BuilderError::LoadAddrNotPtr {
                ptr,
                ptr_ty: ptr_ty.clone(),
            });
        }
        if ty.is_void() {
            return Err(BuilderError::LoadTyVoid);
        }
        if !align.is_power_of_two() {
            return Err(BuilderError::MemAccessAlignNotPowerOf2 { align });
        }
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::Load {
            dst,
            ptr,
            ty,
            align,
            volatile,
        })?;
        Ok(Operand::SSA(dst))
    }
    pub fn build_load_atomic(
        &mut self,
        ty: Type,
        ptr: Operand,
        align: NonZeroU32,
        ordering: AtomOrdering,
    ) -> Result<Operand, BuilderError> {
        let ptr_ty = self.get_type(&ptr, &PTR_TY)?;
        if !ptr_ty.is_ptr() {
            return Err(BuilderError::LoadAddrNotPtr {
                ptr,
                ptr_ty: ptr_ty.clone(),
            });
        }
        if ty.is_void() {
            return Err(BuilderError::LoadTyVoid);
        }
        if !align.is_power_of_two() {
            return Err(BuilderError::MemAccessAlignNotPowerOf2 { align });
        }
        if !matches!(
            ordering,
            AtomOrdering::Unordered
                | AtomOrdering::Monotonic
                | AtomOrdering::Acquire
                | AtomOrdering::SeqCst
        ) {
            return Err(BuilderError::AtomicLoadInvalidOrdering { ordering });
        }
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::LoadAtomic {
            dst,
            ptr,
            ty,
            align,
            ordering,
        })?;
        Ok(Operand::SSA(dst))
    }
    pub fn build_store(
        &mut self,
        ptr: Operand,
        ty: Type,
        val: Operand,
        align: NonZeroU32,
        volatile: bool,
    ) -> Result<(), BuilderError> {
        let ptr_ty = self.get_type(&ptr, &PTR_TY)?;
        if !ptr_ty.is_ptr() {
            return Err(BuilderError::StoreAddrNotPtr {
                ptr,
                ptr_ty: ptr_ty.clone(),
            });
        }
        if ty.is_void() {
            return Err(BuilderError::StoreTyVoid);
        }
        if !align.is_power_of_two() {
            return Err(BuilderError::MemAccessAlignNotPowerOf2 { align });
        }
        self.insert_at_pos(Instruction::Store {
            ptr,
            ty,
            val,
            align,
            volatile,
        })
    }
    pub fn build_store_atomic(
        &mut self,
        ptr: Operand,
        ty: Type,
        val: Operand,
        align: NonZeroU32,
        ordering: AtomOrdering,
    ) -> Result<(), BuilderError> {
        let ptr_ty = self.get_type(&ptr, &PTR_TY)?;
        if !ptr_ty.is_ptr() {
            return Err(BuilderError::StoreAddrNotPtr {
                ptr,
                ptr_ty: ptr_ty.clone(),
            });
        }
        if ty.is_void() {
            return Err(BuilderError::StoreTyVoid);
        }
        if !align.is_power_of_two() {
            return Err(BuilderError::MemAccessAlignNotPowerOf2 { align });
        }
        if !matches!(
            ordering,
            AtomOrdering::Unordered
                | AtomOrdering::Monotonic
                | AtomOrdering::Release
                | AtomOrdering::SeqCst
        ) {
            return Err(BuilderError::AtomicStoreInvalidOrdering { ordering });
        }
        self.insert_at_pos(Instruction::StoreAtomic {
            ptr,
            ty,
            val,
            align,
            ordering,
        })
    }
    pub fn build_memmove(
        &mut self,
        dest: Operand,
        src: Operand,
        len_ty: Type,
        len: Operand,
        volatile: bool,
    ) -> Result<(), BuilderError> {
        let dst_ty = self.get_type(&dest, &PTR_TY)?;
        if dst_ty != &PTR_TY {
            return Err(BuilderError::MemMoveDestNotPtr {
                dst_ty: dst_ty.clone(),
            });
        }
        let src_ty = self.get_type(&src, &PTR_TY)?;
        if src_ty != &PTR_TY {
            return Err(BuilderError::MemMoveSrcNotPtr {
                src_ty: src_ty.clone(),
            });
        }
        if !len_ty.is_int() {
            return Err(BuilderError::MemMoveLenTyNotInt {
                len_ty: len_ty.clone(),
            })?;
        }
        let got_len_ty = self.get_type(&len, &I64_TY)?;
        if *got_len_ty != len_ty {
            return Err(BuilderError::MemMoveLenNotInt {
                len_ty: got_len_ty.clone(),
                len,
            })?;
        }
        self.insert_at_pos(Instruction::MemMove {
            dest,
            src,
            len_ty,
            len,
            volatile,
        })
    }
    pub fn build_memcpy(
        &mut self,
        dest: Operand,
        src: Operand,
        len_ty: Type,
        len: Operand,
        volatile: bool,
    ) -> Result<(), BuilderError> {
        let dst_ty = self.get_type(&dest, &PTR_TY)?;
        if dst_ty != &PTR_TY {
            return Err(BuilderError::MemCpyDestNotPtr {
                dst_ty: dst_ty.clone(),
            });
        }
        let src_ty = self.get_type(&src, &PTR_TY)?;
        if src_ty != &PTR_TY {
            return Err(BuilderError::MemCpySrcNotPtr {
                src_ty: src_ty.clone(),
            });
        }
        if !len_ty.is_int() {
            return Err(BuilderError::MemCpyLenTyNotInt {
                len_ty: len_ty.clone(),
            })?;
        }
        let got_len_ty = self.get_type(&len, &I64_TY)?;
        if *got_len_ty != len_ty {
            return Err(BuilderError::MemCpyLenNotInt {
                len_ty: got_len_ty.clone(),
                len,
            })?;
        }
        self.insert_at_pos(Instruction::MemCpy {
            dest,
            src,
            len_ty,
            len,
            volatile,
        })
    }
    pub fn build_memset(
        &mut self,
        dest: Operand,
        val: Operand,
        len_ty: Type,
        len: Operand,
        volatile: bool,
    ) -> Result<(), BuilderError> {
        let dst_ty = self.get_type(&dest, &PTR_TY)?;
        if dst_ty != &PTR_TY {
            return Err(BuilderError::MemSetDestNotPtr {
                dst_ty: dst_ty.clone(),
            });
        }
        if !len_ty.is_int() {
            return Err(BuilderError::MemSetLenTyNotInt {
                len_ty: len_ty.clone(),
            })?;
        }
        let got_len_ty = self.get_type(&len, &I64_TY)?;
        if *got_len_ty != len_ty {
            return Err(BuilderError::MemSetLenNotInt {
                len_ty: got_len_ty.clone(),
                len,
            })?;
        }
        let val_ty = self.get_type(&val, &I8_TY)?;
        if val_ty.is_int() {
            return Err(BuilderError::MemSetValNotInt {
                len_ty: got_len_ty.clone(),
                len,
            })?;
        }
        self.insert_at_pos(Instruction::MemSet {
            dest,
            val,
            len_ty,
            len,
            volatile,
        })
    }
    // atomic mem
    pub fn build_fence(&mut self, ordering: AtomOrdering) -> Result<(), BuilderError> {
        self.insert_at_pos(Instruction::Fence { ordering })
    }
    pub fn build_atomic_rmw(
        &mut self,
        op: AtomicRmwOp,
        ptr: Operand,
        ty: Type,
        val: Operand,
        ordering: AtomOrdering,
        align: NonZeroU32,
    ) -> Result<Operand, BuilderError> {
        let ptr_ty = self.get_type(&ptr, &PTR_TY)?;
        if !ptr_ty.is_ptr() {
            return Err(BuilderError::AtomicRmwAddrNotPtr {
                ptr,
                ptr_ty: ptr_ty.clone(),
            });
        }
        match op {
            // this is sometimes used with pointers OR floats.
            AtomicRmwOp::Xchg => (),
            AtomicRmwOp::Add
            | AtomicRmwOp::Sub
            | AtomicRmwOp::And
            | AtomicRmwOp::Nand
            | AtomicRmwOp::Or
            | AtomicRmwOp::Xor
            | AtomicRmwOp::Max
            | AtomicRmwOp::Min
            | AtomicRmwOp::UMax
            | AtomicRmwOp::UMin => {
                if !ty.is_int() {
                    return Err(BuilderError::NonIntInAtomicRMWIntOp { op, ty });
                }
            }
            AtomicRmwOp::FAdd | AtomicRmwOp::FSub | AtomicRmwOp::FMax | AtomicRmwOp::FMin => {
                if !ty.is_float() {
                    return Err(BuilderError::NonFloatInAtomicRMWFloatOp { op, ty });
                }
            }
        }
        let val_ty = self.get_type(&val, &ty)?;
        if *val_ty != ty {
            return Err(BuilderError::AtomicRmwWrongValType {
                val_ty: val_ty.clone(),
                val,
                ty,
            });
        }
        if !align.is_power_of_two() {
            return Err(BuilderError::MemAccessAlignNotPowerOf2 { align });
        }
        let dst = self.alloc_ssa_id(ty.clone());
        self.insert_at_pos(Instruction::AtomicRmw {
            dst,
            op,
            ptr,
            ty,
            val,
            ordering,
            align,
        })?;
        Ok(Operand::SSA(dst))
    }
}
