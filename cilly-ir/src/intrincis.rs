use arbitrary::Arbitrary;
use traversable::{Traversable, TraversableMut};

use crate::{Operand, Type};

#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary, Traversable, TraversableMut)]
pub enum Intrinsic {
    /// Float to unsgiend int cast, clamps to range.
    #[qparse("{dst_ty} @llvm.fptoui.sat.{dst_ty}.{src_ty}({src_ty} {val})")]
    FpToUiSat {
        dst_ty: Type,
        src_ty: Type,
        val: Operand,
    },
    /// Float to signed int cast, clamps to range.
    #[qparse("{dst_ty} @llvm.fptosi.sat.{dst_ty}.{src_ty}({src_ty} {val})")]
    FpToSiSat {
        dst_ty: Type,
        src_ty: Type,
        val: Operand,
    },
    /// Checked signed add. Retruns
    /// the two's-complement wrapped sum and a signed-overflow flag.
    #[qparse("{{{ty}, i1}} @llvm.sadd.with.overflow.{ty}({ty} {lhs}, {ty} {rhs})")]
    SAddWithOverflow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Checked unsigned add. Retruns
    /// the wrapped sum and an unsigned-overflow flag.
    #[qparse("{{{ty}, i1}} @llvm.uadd.with.overflow.{ty}({ty} {lhs}, {ty} {rhs})")]
    UAddWithOverflow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Checked signed sub. Returns wraped + flag.  
    #[qparse("{{{ty}, i1}} @llvm.ssub.with.overflow.{ty}({ty} {lhs}, {ty} {rhs})")]
    SSubWithOverflow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Checked unsigned sub - Returns wraped + flag.  
    #[qparse("{{{ty}, i1}} @llvm.usub.with.overflow.{ty}({ty} {lhs}, {ty} {rhs})")]
    USubWithOverflow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Checked signed mul. Returns wraped + flag.  
    #[qparse("{{{ty}, i1}} @llvm.smul.with.overflow.{ty}({ty} {lhs}, {ty} {rhs})")]
    SMulWithOverflow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Checked unsigned mul. Returns wraped + flag.  
    #[qparse("{{{ty}, i1}} @llvm.umul.with.overflow.{ty}({ty} {lhs}, {ty} {rhs})")]
    UMulWithOverflow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Saturating signed add: clapms for out-of range values
    #[qparse("{ty} @llvm.sadd.sat.{ty}({ty} {lhs}, {ty} {rhs})")]
    SAddSat {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Saturating unsigned add: clamps out of range values.
    #[qparse("{ty} @llvm.uadd.sat.{ty}({ty} {lhs}, {ty} {rhs})")]
    UAddSat {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Saturating signed sub - sub, clamps out of range values.
    #[qparse("{ty} @llvm.ssub.sat.{ty}({ty} {lhs}, {ty} {rhs})")]
    SSubSat {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Saturating unsigned sub -  sub, clamps out of range values.
    #[qparse("{ty} @llvm.usub.sat.{ty}({ty} {lhs}, {ty} {rhs})")]
    USubSat {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Count leading zeros. `is_zero_poison`: when `false`, a
    /// zero input yields the bit width; when `true`, a zero input is UB.
    /// This is usefull for C: in C, zero input is Ub, so we need to guard against that.
    /// this flags allows us to not do so.  
    #[qparse("{ty} @llvm.ctlz.{ty}({ty} {val}, i1 {is_zero_poison})")]
    /// Well-defined for all bit widths.
    Ctlz {
        ty: Type,
        val: Operand,
        is_zero_poison: bool,
    },
    /// Count trailing zeros. `is_zero_poison`: when `false`, a
    /// zero input yields the bit width; when `true`, a zero input is UB.
    /// is_zero_poison - same UB gains as ctlz.
    #[qparse("{ty} @llvm.cttz.{ty}({ty} {val}, i1 {is_zero_poison})")]
    /// Well-defined for all bit widths.
    Cttz {
        ty: Type,
        val: Operand,
        is_zero_poison: bool,
    },
    /// Population count: number of set bits. Produces `iN`.
    #[qparse("{ty} @llvm.ctpop.{ty}({ty} {val})")]
    /// Well-defined for all bit widths.
    Ctpop { ty: Type, val: Operand },
    /// Byte swap: reverse byte order.
    /// Useless nop for single bytes.
    #[qparse("{ty} @llvm.bswap.{ty}({ty} {val})")]
    /// Well-defined for a single byte, and even widths.
    Bswap { ty: Type, val: Operand },
    /// Bit reverse: reverse bit order.
    #[qparse("{ty} @llvm.bitreverse.{ty}({ty} {val})")]
    /// Well-defined for all bit widths.
    Bitreverse { ty: Type, val: Operand },
    /// Square root.
    #[qparse("{ty} @llvm.sqrt.{ty}({ty} {val})")]
    /// Returns NaN for %val < 0, +inf for +inf, 0.0 for +/- 0.0.
    Sqrt { ty: Type, val: Operand },
    /// Absolute fp value. If NaN is passed in, an unspecifed NaN is returned.
    #[qparse("{ty} @llvm.fabs.{ty}({ty} {val})")]
    Fabs { ty: Type, val: Operand },
    /// Sine of `%val`.
    #[qparse("{ty} @llvm.sin.{ty}({ty} {val})")]
    /// sin(+/- inf) = NaN
    Sin { ty: Type, val: Operand },
    /// Cosine of `%val`.
    #[qparse("{ty} @llvm.cos.{ty}({ty} {val})")]
    /// cos(+/- inf) = NaN
    Cos { ty: Type, val: Operand },
    /// Base-e exponential.
    #[qparse("{ty} @llvm.exp.{ty}({ty} {val})")]
    Exp { ty: Type, val: Operand },
    /// Base-2 exponential (`llvm.exp2.fN`).
    #[qparse("{ty} @llvm.exp2.{ty}({ty} {val})")]
    Exp2 { ty: Type, val: Operand },
    /// Natural log.
    #[qparse("{ty} @llvm.log.{ty}({ty} {val})")]
    /// NaN for negative values.
    Log { ty: Type, val: Operand },
    /// Base-2 log (`llvm.log2.fN`).
    #[qparse("{ty} @llvm.log2.{ty}({ty} {val})")]
    /// NaN for negative values.
    Log2 { ty: Type, val: Operand },
    /// Base-10 log (`llvm.log10.fN`).
    #[qparse("{ty} @llvm.log10.{ty}({ty} {val})")]
    /// NaN for negative values.
    Log10 { ty: Type, val: Operand },
    /// Round toward -inf.
    #[qparse("{ty} @llvm.floor.{ty}({ty} {val})")]
    Floor { ty: Type, val: Operand },
    /// Round toward +inf.
    #[qparse("{ty} @llvm.ceil.{ty}({ty} {val})")]
    Ceil { ty: Type, val: Operand },
    /// Round toward zero.
    #[qparse("{ty} @llvm.trunc.{ty}({ty} {val})")]
    FpTrunc { ty: Type, val: Operand },
    /// Round to nearest, ties away from zero.
    #[qparse("{ty} @llvm.round.{ty}({ty} {val})")]
    Round { ty: Type, val: Operand },
    /// Round to nearest, ties to even.
    #[qparse("{ty} @llvm.roundeven.{ty}({ty} {val})")]
    RoundEven { ty: Type, val: Operand },
    /// Round to nearest per the current rounding mode.
    /// FIXME: do we need nearbyint + rint? Current startegy is to just ape
    /// LLVM, but with the rewrite, we can drop some of those.
    /// less intrinsics - less fallbacks?
    #[qparse("{ty} @llvm.rint.{ty}({ty} {val})")]
    Rint { ty: Type, val: Operand },
    /// Like `rint` but never raises the inexact exception.
    #[qparse("{ty} @llvm.nearbyint.{ty}({ty} {val})")]
    NearbyInt { ty: Type, val: Operand },
    /// Power: `lhs` raised to `rhs`.
    #[qparse("{ty} @llvm.pow.{ty}({ty} {lhs}, {ty} {rhs})")]
    /// # NaN / edge cases
    /// Per C standard, Annex F:
    /// pow(x, 0) = 1, including x = NaN
    /// pow(0,y), returns inf for y < 0, or 0 if y > 0
    /// pow(1,y) = 1, including y = NaN
    /// pow(<0, y) = NaN, if y is not an int.
    Pow {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Power with intiger exponent: `base` raised to `exp`,
    /// where `exp` is a signed integer (`exp_ty`), not a float.
    /// we lower this to a libc call if possibel, but powi can also be
    /// efficently implemented with a loop.
    #[qparse("{ty} @llvm.powi.{ty}.{exp_ty}({ty} {base}, {exp_ty} {exp})")]
    /// # NaN / edge cases
    /// Per C standard, Annex F:
    /// pow(x, 0) = 1, including x = NaN
    /// pow(0,y), returns inf for y < 0, or 0 if y > 0
    /// $exp_ty must be an int
    Powi {
        ty: Type,
        base: Operand,
        exp_ty: Type,
        exp: Operand,
    },
    /// Copy sign: `lhs` with sign of `rhs`.
    #[qparse("{ty} @llvm.copysign.{ty}({ty} {lhs}, {ty} {rhs})")]
    /// Copies the sign bit **literally** - effecitvely:
    /// res = bitcast((bitcast(lhs) & SIGN_MASK) | bitcast(rhs) & (!SIGN_MASK)))
    Copysign {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Minimum: IEEE minNum, returns the non-NaN operand if
    /// exactly one is NaN.
    /// Lowers to libc fminf.
    #[qparse("{ty} @llvm.minnum.{ty}({ty} {lhs}, {ty} {rhs})")]
    MinNum {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Maximum: IEEE maxNum.
    /// Lowers to libc fmaxf - returns the non-nan operand if a nan provided.
    #[qparse("{ty} @llvm.maxnum.{ty}({ty} {lhs}, {ty} {rhs})")]
    MaxNum {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Minimum: propagates NaN and orders -0.0 < +0.0.
    #[qparse("{ty} @llvm.minimum.{ty}({ty} {lhs}, {ty} {rhs})")]
    Minimum {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Maximum: propagates NaN and orders -0.0 < +0.0.
    #[qparse("{ty} @llvm.maximum.{ty}({ty} {lhs}, {ty} {rhs})")]
    Maximum {
        ty: Type,
        lhs: Operand,
        rhs: Operand,
    },
    /// Fused multiply-add: `a * b + c` with a single rounding.
    #[qparse("{ty} @llvm.fma.{ty}({ty} {a}, {ty} {b}, {ty} {c})")]
    Fma {
        ty: Type,
        a: Operand,
        b: Operand,
        c: Operand,
    },
    /// Multiply-add: `a * b + c`; the backend may or may not
    /// fuse (unlike `fma`, which guarantees a single rounding).
    ///
    /// FIXME: any point exposing this? We can always lower it to
    /// a * b  + c ig.
    #[qparse("{ty} @llvm.fmuladd.{ty}({ty} {a}, {ty} {b}, {ty} {c})")]
    FmulAdd {
        ty: Type,
        a: Operand,
        b: Operand,
        c: Operand,
    },
}
impl Intrinsic {
    pub(crate) fn res_ty(&self) -> Type {
        match self {
            Intrinsic::FpToUiSat { dst_ty, .. } | Intrinsic::FpToSiSat { dst_ty, .. } => {
                dst_ty.clone()
            }
            Intrinsic::SAddWithOverflow { ty, .. }
            | Intrinsic::UAddWithOverflow { ty, .. }
            | Intrinsic::SSubWithOverflow { ty, .. }
            | Intrinsic::USubWithOverflow { ty, .. }
            | Intrinsic::SMulWithOverflow { ty, .. }
            | Intrinsic::UMulWithOverflow { ty, .. } => Type::ty_and_flag(ty.clone()),
            Intrinsic::SAddSat { ty, .. }
            | Intrinsic::UAddSat { ty, .. }
            | Intrinsic::SSubSat { ty, .. }
            | Intrinsic::USubSat { ty, .. } => ty.clone(),
            Intrinsic::Ctlz { ty, .. }
            | Intrinsic::Cttz { ty, .. }
            | Intrinsic::Ctpop { ty, .. }
            | Intrinsic::Bswap { ty, .. }
            | Intrinsic::Bitreverse { ty, .. } => ty.clone(),
            Intrinsic::Sqrt { ty, .. }
            | Intrinsic::Fabs { ty, .. }
            | Intrinsic::Sin { ty, .. }
            | Intrinsic::Cos { ty, .. }
            | Intrinsic::Exp { ty, .. }
            | Intrinsic::Exp2 { ty, .. }
            | Intrinsic::Log { ty, .. }
            | Intrinsic::Log2 { ty, .. }
            | Intrinsic::Log10 { ty, .. }
            | Intrinsic::Floor { ty, .. }
            | Intrinsic::Ceil { ty, .. }
            | Intrinsic::FpTrunc { ty, .. }
            | Intrinsic::Round { ty, .. }
            | Intrinsic::RoundEven { ty, .. }
            | Intrinsic::Rint { ty, .. }
            | Intrinsic::NearbyInt { ty, .. } => ty.clone(),
            Intrinsic::Pow { ty, .. } => ty.clone(),
            Intrinsic::Powi { ty, .. } => ty.clone(),
            Intrinsic::Copysign { ty, .. } => ty.clone(),
            Intrinsic::MinNum { ty, .. }
            | Intrinsic::MaxNum { ty, .. }
            | Intrinsic::Minimum { ty, .. }
            | Intrinsic::Maximum { ty, .. } => ty.clone(),
            Intrinsic::Fma { ty, .. } | Intrinsic::FmulAdd { ty, .. } => ty.clone(),
        }
    }
}
