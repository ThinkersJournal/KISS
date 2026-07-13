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
