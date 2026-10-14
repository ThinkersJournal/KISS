// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.12 / §6.13-0011 / §7.2 — the consumer-verify check and its opt-in profile.
//!
//! Each test runs the check on a candidate that is CORRECT and on a mutated one that must be
//! caught, so a check that always passes (or always fails) cannot back a clause.

use kiss_conformance::consumer_verify::{
    consumer_verify, Profile, SuiteOutcome, VerifyFailure, VerifyReport, BUILD_PROFILE,
};
use kiss_conformance::differential::corpus_f32;

const KEY: &str = "sk5|ew1|f16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f;co/00/v8/d16/f|-";
const OTHER_KEY: &str = "sk5|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-";

fn body(class: &str) -> Vec<u8> {
    format!("[section:6:guarantees]\nreference_function = negate\ndeterminism_class = {class}\n")
        .into_bytes()
}

fn finite_corpus() -> Vec<f32> {
    corpus_f32(7, 64).into_iter().filter(|v| v.is_finite()).collect()
}

fn neg(x: f32) -> f32 {
    -x
}

fn failing_report() -> VerifyReport {
    consumer_verify(&body("exact-byte"), 0, KEY, KEY, neg, |x| x, &finite_corpus())
}

/// KISS-CONFORM-6.12-0001 — an executable check exists, accepts a conforming kernel and
/// rejects a non-conforming one.
#[test]
fn test_conform_consumer_verify_check() {
    let c = finite_corpus();
    let ok = consumer_verify(&body("exact-byte"), 0, KEY, KEY, neg, neg, &c);
    assert!(ok.verified(), "KISS-CONFORM-6.12-0001: a conforming kernel must verify: {:?}", ok.failures);
    let bad = consumer_verify(&body("exact-byte"), 0, KEY, KEY, neg, |x| x, &c);
    assert!(!bad.verified(), "KISS-CONFORM-6.12-0001: a wrong kernel must not verify");
}

/// KISS-CONFORM-6.12-0002 — the declared class SELECTS the shared comparator: a 1-ULP-off
/// candidate fails exact-byte, passes ULP/tolerance(1), fails ULP/tolerance(0); the
/// accept-predicate is part of the verdict.
#[test]
fn test_conform_consumer_verify_reuses_oracle() {
    let c = finite_corpus();
    let off_by_one = |x: f32| f32::from_bits((-x).to_bits().wrapping_add(1));
    let exact = consumer_verify(&body("exact-byte"), 8, KEY, KEY, neg, off_by_one, &c);
    assert!(!exact.verified(), "KISS-CONFORM-6.12-0002: exact-byte must reject a 1-ULP drift");
    let ulp1 = consumer_verify(&body("ULP/tolerance"), 1, KEY, KEY, neg, off_by_one, &c);
    assert!(ulp1.verified(), "KISS-CONFORM-6.12-0002: ULP/tolerance(1) must accept 1 ULP: {:?}", ulp1.failures);
    let ulp0 = consumer_verify(&body("ULP/tolerance"), 0, KEY, KEY, neg, off_by_one, &c);
    assert!(!ulp0.verified(), "KISS-CONFORM-6.12-0002: ULP/tolerance(0) must reject 1 ULP");
    // A signed-zero flip is seen by exact-byte.
    let zero_flip = consumer_verify(&body("exact-byte"), 0, KEY, KEY, |_| 0.0, |_| -0.0, &[1.0]);
    assert!(!zero_flip.verified(), "KISS-CONFORM-6.12-0002: exact-byte must see a sign-of-zero flip");
    // Accept-predicate (= structure_key): a kernel advertising another cell's structure_key is refused.
    let wrong_cell = consumer_verify(&body("exact-byte"), 0, OTHER_KEY, KEY, neg, neg, &c);
    assert_eq!(wrong_cell.failures, vec![VerifyFailure::AcceptPredicateMismatch]);
    // A malformed advertised key is a typed decline, not a panic.
    let junk = consumer_verify(&body("exact-byte"), 0, "not-a-key", KEY, neg, neg, &c);
    assert!(matches!(junk.failures.first(), Some(VerifyFailure::AdvertisedKeyDeclined(_))));
    // An off-enum class is a typed contract decline.
    let bad_class = consumer_verify(&body("bit-reproducible"), 0, KEY, KEY, neg, neg, &c);
    assert!(matches!(bad_class.failures.first(), Some(VerifyFailure::ContractUnreadable(_))));
}

/// KISS-CONFORM-6.12-0003 — a failing consumer-verify report does not fail the default build.
#[test]
fn test_conform_consumer_verify_not_gating() {
    let r = failing_report();
    assert!(!r.verified(), "KISS-CONFORM-6.12-0003: the control report must actually fail");
    let o = SuiteOutcome { profile: Profile::Default, default_clauses_pass: true, verify: r };
    assert!(o.build_passes(false), "KISS-CONFORM-6.12-0003: default profile must not gate on it");
    assert!(o.build_passes(true), "KISS-CONFORM-6.12-0003: advertising does not gate in the default profile");
}

/// KISS-CONFORM-6.13-0011 — the contract-tier closure: the shipped check runs a received kernel
/// against the Contract codec's own Appendix C Guarantees block, with the structure_key accept-predicate.
#[test]
fn test_conform_contract_consumer_verify() {
    use kiss_conformance::contract::appendix_c_body;
    let c = finite_corpus();
    let ok = consumer_verify(&appendix_c_body(), 0, KEY, KEY, neg, neg, &c);
    assert!(ok.verified(), "KISS-CONFORM-6.13-0011: {:?}", ok.failures);
    let bad = consumer_verify(&appendix_c_body(), 0, KEY, KEY, neg, |x| x + 1.0, &c);
    assert!(!bad.verified(), "KISS-CONFORM-6.13-0011: a wrong kernel must fail the Appendix C contract");
}

/// KISS-CONFORM-7.2-0001 — in the build as shipped (default profile) consumer-verify gates nothing.
#[test]
fn test_conform_consumer_verify_profile() {
    assert_eq!(
        BUILD_PROFILE == Profile::Default,
        !cfg!(feature = "trust-but-verify"),
        "KISS-CONFORM-7.2-0001: BUILD_PROFILE must follow the feature switch"
    );
    let o = SuiteOutcome { profile: Profile::Default, default_clauses_pass: true, verify: failing_report() };
    assert!(o.build_passes(true), "KISS-CONFORM-7.2-0001: default profile must not gate");
}

/// KISS-CONFORM-7.2-0003 — the opt-in profile gates, and only for an implementation that advertises it.
#[test]
fn test_conform_optin_profile_defined() {
    let o = SuiteOutcome { profile: Profile::TrustButVerify, default_clauses_pass: true, verify: failing_report() };
    assert!(!o.build_passes(true), "KISS-CONFORM-7.2-0003: opt-in + advertised + failing must gate");
    assert!(o.build_passes(false), "KISS-CONFORM-7.2-0003: not advertised must not gate");
    let good = SuiteOutcome {
        profile: Profile::TrustButVerify,
        default_clauses_pass: true,
        verify: consumer_verify(&body("exact-byte"), 0, KEY, KEY, neg, neg, &finite_corpus()),
    };
    assert!(good.build_passes(true), "KISS-CONFORM-7.2-0003: a verifying kernel passes the gate");
}

/// KISS-CONFORM-7.2-0004 — the opt-in profile does not alter the default clause verdict, and the
/// two results are distinguishable by their profile.
#[test]
fn test_conform_optin_profile_isolated() {
    let mk = |p, pass| SuiteOutcome { profile: p, default_clauses_pass: pass, verify: failing_report() };
    for pass in [true, false] {
        let (d, t) = (mk(Profile::Default, pass), mk(Profile::TrustButVerify, pass));
        assert_eq!(d.default_clauses_pass, t.default_clauses_pass, "KISS-CONFORM-7.2-0004");
        assert_ne!(d, t, "KISS-CONFORM-7.2-0004: results must be distinguishable by profile");
        if !pass {
            // A default-clause failure fails the build under BOTH profiles; opt-in cannot rescue it.
            assert!(!d.build_passes(true) && !t.build_passes(false), "KISS-CONFORM-7.2-0004");
        }
    }
}
