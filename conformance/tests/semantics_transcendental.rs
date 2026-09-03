//! Plan B Slice-0 Task T5/T6 — the `semantics.rs` special-value FRONT DOOR for the
//! transcendental atoms (exp/log/sin at f32 & f64).
//!
//! The atoms (`hp::Exp/Log/Sin`) assume a finite in-range argument — `reduce_*`
//! debug-asserts it. The front door is where a transcendental's NaN is DECIDED
//! (minted vs propagated) and where the domain / over-underflow extremes route to
//! `±Inf`/`±0`/`NaN` before the atom sees them; everything else delegates to the
//! atom, whose Ziv round already maps value overflow → Inf and underflow →
//! subnormal/0 correctly. exp additionally clamps |x| beyond its representable
//! range (which also keeps `reduce_exp`'s `|k|` inside range).
//!
//! Finite in-range results are cross-checked against the atom's oracle-validated
//! bits (MPFR ≡ Arb ≡ mpmath); the delegation is what the front door must preserve.

use kiss_conformance::semantics::{exp, exp_f64, log, log_f64, sin, sin_f64};

const NEG0_64: u64 = 0x8000_0000_0000_0000;
const NEG0_32: u32 = 0x8000_0000;

// ---- exp: NaN/±Inf front door, over/underflow clamp, finite delegation --------

#[test]
fn exp_f64_special_and_boundary() {
    assert!(exp_f64(f64::NAN).is_nan(), "exp(NaN) = NaN");
    assert_eq!(exp_f64(f64::INFINITY), f64::INFINITY, "exp(+Inf) = +Inf");
    assert_eq!(exp_f64(f64::NEG_INFINITY).to_bits(), 0, "exp(-Inf) = +0 (positive zero)");
    // clamp: definitely out of range
    assert_eq!(exp_f64(710.0), f64::INFINITY, "exp(710) overflows to +Inf");
    assert_eq!(exp_f64(-746.0).to_bits(), 0, "exp(-746) underflows to +0");
    // delegation: large-but-finite must NOT be clamped
    assert_eq!(exp_f64(709.0).to_bits(), 0x7FDD_422D_2BE5_DC9B, "exp(709) is a large finite");
    assert_eq!(exp_f64(1.0).to_bits(), 0x4005_BF0A_8B14_5769, "exp(1) = e via the atom");
    assert_eq!(exp_f64(0.0), 1.0, "exp(0) = 1");
}

#[test]
fn exp_f32_special_and_boundary() {
    assert!(exp(f32::NAN).is_nan(), "exp(NaN) = NaN");
    assert_eq!(exp(f32::INFINITY), f32::INFINITY, "exp(+Inf) = +Inf");
    assert_eq!(exp(f32::NEG_INFINITY).to_bits(), 0, "exp(-Inf) = +0");
    assert_eq!(exp(100.0), f32::INFINITY, "exp(100) overflows f32 to +Inf");
    assert_eq!(exp(-105.0).to_bits(), 0, "exp(-105) underflows f32 to +0");
    // delegation: the atom's near-overflow / subnormal anchors
    assert_eq!(exp(88.0).to_bits(), 0x7EF8_82B7, "exp(88) near-overflow finite via the atom");
    assert_eq!(exp(-87.0).to_bits(), 0x00B3_3687, "exp(-87) f32 subnormal via the atom");
}

// ---- log: NaN/domain/zero/Inf front door, finite delegation -------------------

#[test]
fn log_f64_special_and_boundary() {
    assert!(log_f64(f64::NAN).is_nan(), "log(NaN) = NaN");
    assert!(log_f64(-1.0).is_nan(), "log(x<0) = NaN");
    assert!(log_f64(f64::NEG_INFINITY).is_nan(), "log(-Inf) = NaN");
    assert_eq!(log_f64(0.0), f64::NEG_INFINITY, "log(+0) = -Inf");
    assert_eq!(log_f64(-0.0), f64::NEG_INFINITY, "log(-0) = -Inf");
    assert_eq!(log_f64(f64::INFINITY), f64::INFINITY, "log(+Inf) = +Inf");
    // delegation
    assert_eq!(log_f64(1.0).to_bits(), 0, "log(1) = +0 via the atom");
    assert_eq!(log_f64(2.0).to_bits(), 0x3FE6_2E42_FEFA_39EF, "log(2) = ln2 via the atom");
}

#[test]
fn log_f32_special_and_boundary() {
    assert!(log(f32::NAN).is_nan(), "log(NaN) = NaN");
    assert!(log(-2.0).is_nan(), "log(x<0) = NaN");
    assert_eq!(log(0.0), f32::NEG_INFINITY, "log(+0) = -Inf");
    assert_eq!(log(f32::INFINITY), f32::INFINITY, "log(+Inf) = +Inf");
    assert_eq!(log(2.0).to_bits(), 0x3F31_7218, "log(2) = ln2 via the atom");
}

// ---- sin: NaN/±Inf front door (both mint NaN), finite delegation --------------

#[test]
fn sin_f64_special_and_boundary() {
    assert!(sin_f64(f64::NAN).is_nan(), "sin(NaN) = NaN");
    assert!(sin_f64(f64::INFINITY).is_nan(), "sin(+Inf) = NaN (minted)");
    assert!(sin_f64(f64::NEG_INFINITY).is_nan(), "sin(-Inf) = NaN (minted)");
    assert_eq!(sin_f64(0.0).to_bits(), 0, "sin(+0) = +0");
    assert_eq!(sin_f64(-0.0).to_bits(), NEG0_64, "sin(-0) = -0 (odd)");
    assert_eq!(sin_f64(1.0).to_bits(), 0x3FEA_ED54_8F09_0CEE, "sin(1) via the atom");
}

#[test]
fn sin_f32_special_and_boundary() {
    assert!(sin(f32::NAN).is_nan(), "sin(NaN) = NaN");
    assert!(sin(f32::INFINITY).is_nan(), "sin(+Inf) = NaN");
    assert_eq!(sin(-0.0f32).to_bits(), NEG0_32, "sin(-0) = -0");
    assert_eq!(sin(1.0).to_bits(), 0x3F57_6AA4, "sin(1) via the atom");
}
