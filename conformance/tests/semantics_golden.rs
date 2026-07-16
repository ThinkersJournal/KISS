//! KISS-Conform differential/semantic vectors for the pinned floating-point
//! primitives (KISS-Ops §2.3 / §6.15). These are the numeric distinctions the
//! spec spends prose pinning — now executable: NaN-propagating vs NaN-suppressing
//! min/max, `relu` ≠ `max(x,0)`, and the §6.8 "declared-ULP, not bit-identity"
//! transcendental model. Determinism-class comparators from Conform §6.8.

use kiss_conformance::semantics::*;
use kiss_conformance::{compare_f32, ulp_distance_f32, DeterminismClass};

/// Raw-bit equality — the exact-byte determinism class for a scalar (§6.8).
fn bits_eq(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

// ---- the load-bearing distinction: NaN-propagating vs NaN-suppressing --------

/// Enforces KISS-OPS-6.15-0001 — max_prop/min_prop (NaN-propagating) and
/// fmax_ieee/fmin_ieee (NaN-suppressing) are four distinct ops. Catches an
/// `fmax_ieee` built from `max_prop`: it returns NaN for fmax_ieee(NaN, 5.0)
/// where the clause pins 5.0.
#[test]
fn max_propagates_nan_but_fmax_suppresses_it() {
    let nan = f32::NAN;
    // max_prop keeps a NaN operand; fmax_ieee returns the other — the whole reason
    // KISS-Ops keeps these as four distinct, non-mergeable ops.
    assert!(max_prop(nan, 5.0).is_nan());
    assert!(max_prop(5.0, nan).is_nan());
    assert_eq!(fmax_ieee(nan, 5.0), 5.0);
    assert_eq!(fmax_ieee(5.0, nan), 5.0);
    assert!(fmax_ieee(nan, nan).is_nan()); // both NaN -> NaN
    // they agree on ordinary values
    assert_eq!(max_prop(3.0, 5.0), 5.0);
    assert_eq!(fmax_ieee(3.0, 5.0), 5.0);
}

#[test]
fn min_propagates_nan_but_fmin_suppresses_it() {
    let nan = f32::NAN;
    assert!(min_prop(nan, 5.0).is_nan());
    assert!(min_prop(5.0, nan).is_nan());
    assert_eq!(fmin_ieee(nan, 5.0), 5.0);
    assert_eq!(fmin_ieee(5.0, nan), 5.0);
    assert_eq!(min_prop(3.0, 5.0), 3.0);
}

#[test]
fn max_prop_returns_the_nan_operand_bit_for_bit() {
    // the §6.15 decomposition returns operand `a` itself when it is NaN, so the
    // NaN payload is preserved exactly.
    let nan = f32::from_bits(0x7FC0_1234);
    assert_eq!(max_prop(nan, 5.0).to_bits(), nan.to_bits());
}

#[test]
fn signed_zero_follows_cmp_ge_operand_order() {
    // fmax_ieee(-0, +0): both numbers, cmp_ge(-0,+0) is true (equal) -> returns a.
    assert!(bits_eq(fmax_ieee(-0.0, 0.0), -0.0));
    assert!(bits_eq(fmax_ieee(0.0, -0.0), 0.0));
}

// ---- relu is NOT max(x, 0) ---------------------------------------------------

#[test]
fn relu_propagates_nan_and_preserves_negative_zero() {
    assert!(relu(f32::NAN).is_nan(), "relu propagates NaN");
    assert!(bits_eq(relu(-0.0), -0.0), "relu preserves -0.0");
    assert!(bits_eq(relu(-1.0), 0.0), "relu of a negative yields +0.0");
    assert_eq!(relu(2.5), 2.5);
    assert!(bits_eq(relu(0.0), 0.0));
}

/// Enforces KISS-OPS-6.15-0002 — `relu` is NaN-propagating and -0.0-preserving,
/// and MUST NOT be implemented as `max(x, 0)`. Catches exactly that wrong
/// lowering: `naive_max_x_zero` scrubs a NaN input to 0 where `relu` keeps it.
#[test]
fn relu_diverges_from_max_x_zero_at_nan() {
    // The deterministic divergence §2.3 warns about: relu keeps NaN, max(x,0)
    // scrubs it to 0. (max(x,0)'s sign of zero is platform-unspecified, so only
    // the NaN case is asserted for the naive form.)
    assert!(relu(f32::NAN).is_nan());
    assert!(!naive_max_x_zero(f32::NAN).is_nan(), "max(x,0) scrubs NaN to 0");
}

// ---- the comparators themselves (Conform §6.8) ------------------------------

#[test]
fn exact_byte_f32_distinguishes_signed_zero_and_nan() {
    assert!(compare_f32(DeterminismClass::ExactByte, -0.0, -0.0, 0).is_ok());
    assert!(compare_f32(DeterminismClass::ExactByte, -0.0, 0.0, 0).is_err(), "±0 differ under exact-byte");
    let n = f32::NAN;
    assert!(compare_f32(DeterminismClass::ExactByte, n, n, 0).is_ok());
}

#[test]
fn ulp_comparator_measures_distance() {
    let x = 1.0f32;
    let next = f32::from_bits(x.to_bits() + 1); // next representable f32 above 1.0
    assert_eq!(ulp_distance_f32(x, x), 0);
    assert_eq!(ulp_distance_f32(x, next), 1);
    assert!(compare_f32(DeterminismClass::UlpTolerance, next, x, 1).is_ok());
    assert!(compare_f32(DeterminismClass::UlpTolerance, next, x, 0).is_err());
}

// ---- broadened primitive floor: sign-bit, sign/step, rounding, comparisons --

#[test]
fn sign_bit_atoms_signed_zero() {
    assert!(bits_eq(neg(0.0), -0.0));
    assert!(bits_eq(neg(-0.0), 0.0));
    assert!(bits_eq(abs(-0.0), 0.0));
    assert!(bits_eq(abs(-3.5), 3.5));
    assert!(neg(f32::NAN).is_nan()); // flips sign, keeps NaN
    assert!(bits_eq(copysign(3.0, -1.0), -3.0));
    assert!(bits_eq(copysign(-3.0, 0.0), 3.0));
    assert!(bits_eq(copysign(3.0, -0.0), -3.0)); // sign of a signed zero carries
}

#[test]
fn sign_and_step_at_nan_and_zero() {
    assert_eq!(sign(2.0), 1.0);
    assert_eq!(sign(-3.0), -1.0);
    assert_eq!(sign(0.0), 0.0);
    assert_eq!(sign(-0.0), 0.0);
    assert_eq!(sign(f32::NAN), 0.0); // NaN -> 0 (both cmp_gt and cmp_lt are false)
    assert_eq!(step(2.0), 1.0);
    assert_eq!(step(0.0), 0.0);
    assert_eq!(step(-1.0), 0.0);
    assert_eq!(step(f32::NAN), 0.0);
}

/// Enforces KISS-OPS-6.7-0001 — floor→−∞, ceil→+∞, trunc→zero, and round_even
/// to nearest ties-to-even. Catches `round_even` lowered to C `roundf` / Rust
/// `f32::round` (ties away from zero): those give round_even(2.5)==3.0 where the
/// clause pins 2.0.
#[test]
fn round_even_is_bankers_rounding() {
    // round-half-to-even, NOT round-half-away-from-zero
    assert_eq!(round_even(0.5), 0.0);
    assert_eq!(round_even(1.5), 2.0);
    assert_eq!(round_even(2.5), 2.0);
    assert_eq!(round_even(3.5), 4.0);
    assert_eq!(round_even(-2.5), -2.0);
    assert_eq!(floor(1.7), 1.0);
    assert_eq!(ceil(1.2), 2.0);
    assert_eq!(trunc(-1.7), -1.0);
}

/// Enforces KISS-OPS-6.6-0002 — cmp_eq/lt/le/gt/ge each yield 0 (false) when
/// either operand is NaN. Catches `cmp_ge` lowered as `!(a < b)` (the standard
/// trick on an lt-only ISA): that returns true for cmp_ge(NaN, NaN) where the
/// clause pins false.
#[test]
fn comparisons_are_ieee_ordered() {
    assert!(cmp_eq(2.0, 2.0) && !cmp_eq(2.0, 3.0));
    assert!(cmp_lt(1.0, 2.0) && cmp_ge(2.0, 2.0) && cmp_le(2.0, 2.0));
    // a NaN operand makes every ordered comparison false ...
    assert!(!cmp_eq(f32::NAN, f32::NAN));
    assert!(!cmp_lt(f32::NAN, 1.0) && !cmp_gt(f32::NAN, 1.0) && !cmp_ge(f32::NAN, f32::NAN));
    // ... except cmp_ne, which is true — and isnan(x) == cmp_ne(x, x)
    assert!(cmp_ne(f32::NAN, f32::NAN));
    assert!(isnan(f32::NAN) && !isnan(0.0));
}

#[test]
fn transcendental_agrees_within_declared_ulp() {
    // §6.8: a transcendental's semantics is "the named function to within a
    // declared ULP", not bit-identity. A high-accuracy f64 oracle and the
    // f32-native path are two independent computations that must agree within a
    // modest declared bound (a stand-in for the §6.8 per-atom ULP ceiling).
    let bound = 4;
    for &x in &[-2.0f32, -0.5, 0.0, 0.5, 1.0, 2.0, 3.5] {
        let oracle = (x as f64).exp() as f32; // compute in f64, round once
        let under_test = x.exp(); // f32-native
        assert!(
            compare_f32(DeterminismClass::UlpTolerance, under_test, oracle, bound).is_ok(),
            "exp({x}): {} ULP exceeds bound {bound}",
            ulp_distance_f32(under_test, oracle)
        );
    }
}
