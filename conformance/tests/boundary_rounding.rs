//! KISS-Conform §6.5-0006 / §6.5-0007 — two oracle-hygiene disciplines a
//! differential oracle must hold, surfaced by a Baracuda↔KISS alignment review
//! (the reference oracle's `round_to_compute`, oracle.rs:507, and its
//! libm-quality f64 transcendental references).
//!
//! 1. **Boundary rounding** (§6.5-0006): a discontinuous op (`cmp_*`, a `select`
//!    condition, `sign`, `step`) resolves its boundary in the op's *compute*
//!    dtype, so the oracle must narrow each operand to that dtype BEFORE the
//!    comparison. An f64 differential oracle that skips this flips spuriously at a
//!    boundary the kernel decides in f32.
//! 2. **Oracle tightness** (§6.5-0007): the oracle's transcendental reference must
//!    be *strictly tighter* than the declared ULP tolerance it enforces, else the
//!    differential measures the oracle's error, not the implementation's — a false
//!    pass. Evaluating wider (f64) and rounding once is that accuracy floor.

use kiss_conformance::semantics::{round_to_compute_f32, sign, step};
use kiss_conformance::ulp_distance_f32;

// ---- §6.5-0006 : boundary decision on the compute-dtype-rounded operand -------

#[test]
fn test_conform_boundary_decision_compute_dtype() {
    // Pinned boundary golden vector (Appendix A.2), op = cmp_gt, compute dtype f32:
    //   x = 1.0 + 2^-30   f64 bits 0x3FF0000000400000  — distinct from 1.0 in f64,
    //                     but 2^-30 << half-ULP(1.0 f32)=2^-24, so it rounds to 1.0f32
    //   t = 1.0           f64 bits 0x3FF0000000000000
    let x = f64::from_bits(0x3FF0_0000_0040_0000);
    let t = 1.0_f64;
    assert_eq!(x.to_bits(), 0x3FF0_0000_0040_0000);
    assert_ne!(x, t, "the two operands are distinct in the differential's f64 precision");

    // The SPURIOUS decision the clause forbids: deciding on the un-narrowed f64.
    let unrounded_decision = x > t; // true — the spurious flip
    assert!(unrounded_decision);

    // The REQUIRED decision: narrow each operand to the f32 compute dtype first.
    let (xr, tr) = (round_to_compute_f32(x), round_to_compute_f32(t));
    assert_eq!(xr, 1.0, "x narrows to exactly 1.0 f32");
    assert_eq!(xr, tr, "both operands are the SAME value at the op's compute dtype");
    let rounded_decision = xr > tr; // false — the pinned oracle output
    assert!(!rounded_decision, "pinned boundary golden vector: cmp_gt = 0 (false)");

    // Load-bearing: the two decisions disagree, so the rounding is not cosmetic.
    assert_ne!(unrounded_decision, rounded_decision);

    // cmp_eq is the mirror image: equal only after rounding to the compute dtype.
    assert!(!(x == t), "cmp_eq on un-narrowed f64 is false");
    assert!(xr == tr, "cmp_eq on compute-dtype-rounded operands is true");
}

#[test]
fn test_conform_boundary_rounding_underflow_sign_and_step() {
    // A magnitude nonzero in f64 but rounding to 0.0 in f32 (below half the
    // smallest f32 subnormal): sign/step must be decided in the compute dtype.
    let tiny_pos = 2.0_f64.powi(-160); // > 0 in f64, underflows to +0.0 in f32
    assert!(tiny_pos > 0.0);
    assert_eq!(round_to_compute_f32(tiny_pos), 0.0, "underflows to 0.0 at f32");

    // sign / step decided on the f32-rounded operand (what the kernel computes).
    let xr = round_to_compute_f32(tiny_pos) as f32;
    assert_eq!(sign(xr), 0.0, "sign of the f32-rounded operand is 0 (it is +0.0)");
    assert_eq!(step(xr), 0.0, "step of the f32-rounded operand is 0");

    // Deciding on the un-narrowed f64 would spuriously give sign=+1 / step=1.
    let unrounded_sign = if tiny_pos > 0.0 { 1.0 } else if tiny_pos < 0.0 { -1.0 } else { 0.0 };
    assert_eq!(unrounded_sign, 1.0, "the forbidden un-narrowed decision diverges");
    assert_ne!(unrounded_sign, sign(xr) as f64);
}

// ---- §6.5-0007 : the oracle is strictly tighter than the declared tolerance ---

#[test]
fn test_conform_oracle_tighter_than_declared_ulp() {
    use std::f64::consts::E;

    // A stand-in for a §6.8-0003 *declared* per-target ULP ceiling (always >= 1).
    let declared_ulp: u64 = 3;

    // The oracle-accuracy FLOOR: evaluate wider (f64) and round once to f32. Its
    // own error vs the true value is <= 0.5 ULP of the compute dtype, by
    // round-to-nearest — strictly under any declared tolerance (>= 1 ULP).
    // Checked against the math constant e (exp(1)), an independent reference.
    let oracle_e = (1.0_f64).exp() as f32; // wider eval, round once
    let ulp_at_e = f32::from_bits((E as f32).to_bits() + 1) as f64 - (E as f32) as f64;
    let oracle_err_ulp = ((oracle_e as f64) - E).abs() / ulp_at_e;
    assert!(
        oracle_err_ulp <= 0.5 + 1e-6,
        "oracle exp(1) error {oracle_err_ulp} ULP exceeds the 0.5-ULP accuracy floor"
    );
    assert!(
        (oracle_err_ulp as u64) < declared_ulp,
        "oracle must be STRICTLY tighter than the declared {declared_ulp}-ULP tolerance"
    );

    // exp(0) = 1.0 exactly — a zero-error anchor point for the wider-eval oracle.
    assert_eq!(((0.0_f64).exp() as f32).to_bits(), 1.0_f32.to_bits());

    // The false-pass trap the floor prevents: an "oracle" computed at the SAME
    // compute-dtype precision as the implementation under test reports 0 ULP for
    // that implementation and can certify nothing — it is inadmissible (§6.5-0007).
    let implementation = (1.0_f32).exp(); // f32-native path under test
    let same_precision_oracle = (1.0_f32).exp(); // NOT tighter than the tolerance
    assert_eq!(
        ulp_distance_f32(implementation, same_precision_oracle),
        0,
        "a same-precision oracle yields a vacuous 0-ULP false pass"
    );

    // The wider-precision oracle, by contrast, is a genuinely independent
    // reference: it is within the accuracy floor of the true value at every point.
    for &x in &[-2.0f32, -0.5, 0.0, 0.5, 1.0, 2.0, 3.5] {
        let truth = (x as f64).exp();
        let wide_oracle = truth as f32; // wider eval, round once
        let ulp_at = f32::from_bits(wide_oracle.to_bits().wrapping_add(1)) as f64
            - wide_oracle as f64;
        let err = ((wide_oracle as f64) - truth).abs() / ulp_at.abs();
        assert!(err <= 0.5 + 1e-6, "oracle exp({x}) error {err} ULP exceeds 0.5 floor");
    }
}
