// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.8-0004a: the declared tolerance, read from a contract's Guarantees, is the
//! comparator's threshold, with no implicit default. One worked vector: an f32 sum with an f32
//! accumulator, `k = 1`, `N = 1024`, `|max partial sum| = 4096`, so the band is
//! `1 · 1024 · 2^-24 · 4096 = 0.25` exactly.

use kiss_conformance::contract::{DeclaredAccuracyTier, Real};
use kiss_conformance::tolerance::{
    comparator_threshold, threshold_admits_f32, AccumulatorBand, DeclaredTolerance,
    NoDeclaredTolerance, Threshold,
};

fn band() -> DeclaredTolerance {
    DeclaredTolerance::Band(AccumulatorBand { k: 1.0, eps_a: f64::from(f32::EPSILON) / 2.0 })
}

/// KISS-CONFORM-6.8-0004a.
#[test]
fn test_conform_tolerance_threshold_from_declared() {
    // Clause: KISS-CONFORM-6.8-0004a.
    // The band: exactly 0.25, evaluated per invocation from the runtime N and partial sum.
    let th = comparator_threshold(Some(band()), 1024, 4096.0).unwrap();
    assert_eq!(th, Threshold::AbsoluteBand(0.25));
    // The same contract at another extent gives another threshold: N is not keyed.
    assert_eq!(
        comparator_threshold(Some(band()), 2048, 4096.0).unwrap(),
        Threshold::AbsoluteBand(0.5)
    );

    // Worked vector: 0.2 apart is inside the band, 0.3 apart is outside it.
    assert!(threshold_admits_f32(th, 100.2, 100.0));
    assert!(!threshold_admits_f32(th, 100.3, 100.0));
    // Absolute, not result-relative (§6.8-0012): under cancellation the result is near zero,
    // and a 0.2 difference is still admitted; a result-relative band would reject it.
    assert!(threshold_admits_f32(th, 0.2, 0.0));

    // A flat tier maps to the ULP/tolerance comparator and admits within every declared bound.
    let tier = DeclaredAccuracyTier { max_ulp: Some(4), ..Default::default() };
    let th = comparator_threshold(Some(DeclaredTolerance::Tier(tier)), 0, 0.0).unwrap();
    assert_eq!(th, Threshold::Tier(tier));
    let one = 1.0f32;
    assert!(threshold_admits_f32(th, f32::from_bits(one.to_bits() + 4), one));
    assert!(!threshold_admits_f32(th, f32::from_bits(one.to_bits() + 5), one));
    let both = DeclaredAccuracyTier {
        max_ulp: Some(4),
        max_absolute: Some(Real::f32(0.0)),
        ..Default::default()
    };
    let th = comparator_threshold(Some(DeclaredTolerance::Tier(both)), 0, 0.0).unwrap();
    assert!(!threshold_admits_f32(th, f32::from_bits(one.to_bits() + 1), one), "every bound applies");

    // No implicit default: nothing declared, or a tier with no bound, yields no threshold.
    assert_eq!(comparator_threshold(None, 1024, 4096.0), Err(NoDeclaredTolerance));
    let empty = DeclaredAccuracyTier::default();
    assert_eq!(
        comparator_threshold(Some(DeclaredTolerance::Tier(empty)), 1024, 4096.0),
        Err(NoDeclaredTolerance)
    );
}
