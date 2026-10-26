// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.8-0002a: a ULP result is measured against an audited wide-precision reference,
//! never against another implementation. The failing vector is a shared defect: two implementations
//! of `exp(1)` that return the same wrong bits agree exactly with each other and are far from the
//! reference.

use kiss_conformance::accuracy::{ulp_result_is_recordable, UlpReferent, UlpResult};
use kiss_conformance::ulp_distance_f32;

/// The audited reference value of `exp(1)` rounded to `f32` (the f64 constant `e`, a stand-in for
/// the audited wide-precision reference).
fn reference() -> f32 {
    std::f64::consts::E as f32
}

/// Both implementations return the reference plus six ULP: the same defect, so they agree.
fn shared_defect() -> f32 {
    f32::from_bits(reference().to_bits() + 6)
}

/// KISS-CONFORM-6.8-0002a. The declared bound is 4 ULP.
#[test]
fn test_conform_ulp_referent_is_wide_reference() {
    // Clause: KISS-CONFORM-6.8-0002a.
    let (x, y) = (shared_defect(), shared_defect());
    let bound = 4.0;

    // The differential: X against Y is 0 ULP, which would pass the 4 ULP bound by agreement.
    let differential = UlpResult { ulp: ulp_distance_f32(x, y) as f64, referent: UlpReferent::OtherImplementation };
    assert_eq!(differential.ulp, 0.0);
    assert!(differential.ulp <= bound, "the differential passes the bound: that is the trap");
    assert!(
        !ulp_result_is_recordable(differential),
        "agreement between two implementations MUST NOT be recorded as a ULP result"
    );

    // The same pair measured against the reference: 6 ULP, outside the bound, and recordable.
    let truth = UlpResult { ulp: ulp_distance_f32(x, reference()) as f64, referent: UlpReferent::AuditedWideReference };
    assert_eq!(truth.ulp, 6.0);
    assert!(ulp_result_is_recordable(truth));
    assert!(truth.ulp > bound, "measured against the reference, the shared defect fails the bound");
}
