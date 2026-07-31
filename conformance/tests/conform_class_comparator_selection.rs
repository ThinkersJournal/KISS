//! KISS-CONFORM-6.13-0006b: the comparator is selected from the op's ADVERTISED
//! determinism class, never hardcoded. The crux: the SAME op and monoid, advertised
//! two different ways, yield OPPOSITE verdicts on the same result — proving selection
//! follows the advertisement, not 3a's structural monoid map. Pure Rust (a real
//! within-band reassociated result), so it runs on both CI legs.

use kiss_conformance::harness::advertised::select_and_compare_reduced;
use kiss_conformance::structural::{reassoc_bound_f32, Monoid};
use kiss_conformance::DeterminismClass;

// Enforces KISS-CONFORM-6.13-0006b: comparator selected from the advertised class.
#[test]
fn test_conform_ops_class_comparator_selection() {
    // A legitimately reassociated Sum result: true sum 1e8, one order lands 1e8+8.
    let expected = 1e8f32;
    let actual = 1e8f32 + 8.0; // within the reassociation band for ~16 addends @1e8
    let abs_tol = 2.0 * reassoc_bound_f32(16, 1e8); // ~179 » 8
    let rel_tol = 0.0;

    // Advertised order-invariant (the TRUE class of a Sum fold) → band comparator → ACCEPT.
    assert!(select_and_compare_reduced(
        "reduce", Some(Monoid::Sum), DeterminismClass::OrderInvariant,
        actual, expected, abs_tol, rel_tol,
    ).is_ok());

    // SAME op, SAME monoid, SAME result — advertised exact-byte → byte comparator → REJECT.
    // (An over-strict over-claim: the honesty lint permits it; the differential exposes it.)
    assert!(select_and_compare_reduced(
        "reduce", Some(Monoid::Sum), DeterminismClass::ExactByte,
        actual, expected, abs_tol, rel_tol,
    ).is_err());

    // Too-permissive advertisement is rejected before any compare: advertise a Max
    // reduce (true class exact-byte) as order-invariant → honesty lint Err.
    assert!(select_and_compare_reduced(
        "reduce", Some(Monoid::Max), DeterminismClass::OrderInvariant,
        1.0, 1.0, 0.0, 0.0,
    ).is_err());
}
