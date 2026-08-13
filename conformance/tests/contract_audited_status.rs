//! The `audited_status` DERIVATION — four clauses, four tests, four proofs.
//!
//! `audited_status` is a **derived** trust field: it says whether a kernel's
//! precision was measured against something. The wrong implementation is not
//! exotic — it is an emitter that writes the field it wants and a suite that
//! reads it back. Three KISS-Contract clauses pin the rule and one KISS-Conform
//! clause requires the suite to check it, and each names its own test:
//!
//!   §6.8-0008  `test_contract_audited_status_derived`      derived, not authored
//!   §6.8-0009  `test_contract_audited_derivation_rule`     the `audited` arm
//!   §6.8-0010  `test_contract_unaudited_derivation_rule`   the `unaudited` arm + totality
//!   §6.13-0021 `test_conform_contract_audited_status`      the suite verifies the derivation
//!
//! They are FOUR tests, not one credited four times: each asserts a property its
//! own clause states and no sibling states, and each is proven by a mutation that
//! leaves the other three green. A single assertion spread across four clause IDs
//! would be a 4× coverage overstatement wearing the shape of efficiency.
//!
//! The subtle arm is §6.8-0009's **inclusion**: an
//! `order-invariant/nondeterministic` kernel whose nondeterminism is declared
//! against a named reference under a stated tolerance derives `audited`. The
//! plausible wrong rule — "nondeterministic, therefore unaudited" — reads as
//! conservative and is simply incorrect.
//!
//! CITATION DISCIPLINE: each test cites ONLY its own clause id; cross-references
//! use the `§<sec>-<nnnn>` short form, which does not match the citation grammar.

use kiss_conformance::contract::{
    derive_audited_status, verify_audited_status, AuditedStatus, DeclaredAccuracyTier, Guarantees,
};
use kiss_conformance::DeterminismClass;

/// A tier declaring 4 ULP against the named reference.
fn bounded_tier() -> DeclaredAccuracyTier {
    DeclaredAccuracyTier { max_ulp: Some(4), ..DeclaredAccuracyTier::default() }
}

/// A tier carrying none of `{max_ulp, max_relative, max_absolute}` — a tier that
/// declares no bound (§6.8-0002).
fn unbounded_tier() -> DeclaredAccuracyTier {
    DeclaredAccuracyTier::default()
}

/// Guarantees declaring a bounded precision against a named reference.
fn audited_guarantees() -> Guarantees {
    Guarantees {
        reference_function: Some("exp".into()),
        per_backend_ulp_tiers: vec![("cuda:sm89".into(), bounded_tier())],
        determinism_class: DeterminismClass::UlpTolerance,
        bit_stability: false,
    }
}

// ---------------------------------------------------------------------------
// KISS-CONTRACT-6.8-0008 — derived from the Guarantees, never an authored constant.
// ---------------------------------------------------------------------------

/// Enforces KISS-CONTRACT-6.8-0008 — `audited_status` is a FUNCTION of the
/// Guarantees fields it is derived from, not a value an implementation may author
/// independently of them.
///
/// TEETH: a derivation that returns a stored constant. The property that catches
/// it is *responsiveness* — mutating a field the rule reads MUST move the result.
/// A constant-returning rule fails whichever direction it is constant in, so both
/// directions are asserted. This is the only test here that varies a field and
/// demands the output change; the arm tests below fix the inputs and check the
/// value, and would both pass against a constant.
#[test]
fn test_contract_audited_status_derived() {
    let mut g = audited_guarantees();
    assert_eq!(derive_audited_status(&g), AuditedStatus::Audited);

    // Withdraw the named reference: the same kernel must now derive differently.
    g.reference_function = None;
    assert_eq!(
        derive_audited_status(&g),
        AuditedStatus::Unaudited,
        "KISS-CONTRACT-6.8-0008: dropping the named `reference_function` did not change the \
         derived value — the field is authored, not derived"
    );

    // Restore the reference but withdraw the bound: also a derived difference.
    g = audited_guarantees();
    g.per_backend_ulp_tiers = vec![("cuda:sm89".into(), unbounded_tier())];
    assert_eq!(
        derive_audited_status(&g),
        AuditedStatus::Unaudited,
        "KISS-CONTRACT-6.8-0008: emptying the declared bound did not change the derived value"
    );

    // An empty/whitespace `reference_function` is not a named reference — the
    // cheapest way to author `audited` while appearing to name something.
    g = audited_guarantees();
    g.reference_function = Some("   ".into());
    assert_eq!(
        derive_audited_status(&g),
        AuditedStatus::Unaudited,
        "KISS-CONTRACT-6.8-0008: a blank `reference_function` was accepted as a named reference"
    );
}

// ---------------------------------------------------------------------------
// KISS-CONTRACT-6.8-0009 — the `audited` arm, INCLUDING the nondeterministic case.
// ---------------------------------------------------------------------------

/// Enforces KISS-CONTRACT-6.8-0009 — the rule yields `audited` for a kernel whose
/// Guarantees declare a bounded precision against a named `reference_function`
/// **under its determinism class**, expressly including an
/// `order-invariant/nondeterministic` kernel whose nondeterminism is declared
/// against a named reference under a stated tolerance.
///
/// TEETH: a rule that gates `audited` on the determinism class — "exact-byte only",
/// or "anything but nondeterministic". Both read as conservative and both are
/// wrong. All three classes are asserted with identical bounded precision, so a
/// class-gated rule fails on whichever class it excludes.
///
/// The clause's second prohibition — the rule MUST NOT **set** `bit_stability`
/// (§6.8-0005 owns it) — is carried by the signature rather than an assertion;
/// see the note at the end of the test body.
#[test]
fn test_contract_audited_derivation_rule() {
    for class in [
        DeterminismClass::ExactByte,
        DeterminismClass::UlpTolerance,
        DeterminismClass::OrderInvariant,
    ] {
        let g = Guarantees { determinism_class: class, ..audited_guarantees() };
        assert_eq!(
            derive_audited_status(&g),
            AuditedStatus::Audited,
            "KISS-CONTRACT-6.8-0009: bounded precision against a named reference under \
             {class:?} did not derive `audited` — the rule is gated on the determinism class"
        );
    }

    // A bound expressed as a relative or absolute tolerance is a bound too — the
    // tier is "at least one of {max_ulp, max_relative, max_absolute}".
    for tier in [
        DeclaredAccuracyTier { max_relative: Some(kiss_conformance::contract::Real::f32(1e-6)), ..DeclaredAccuracyTier::default() },
        DeclaredAccuracyTier { max_absolute: Some(kiss_conformance::contract::Real::f32(1e-9)), ..DeclaredAccuracyTier::default() },
        // `correctly-rounded` / `bit-reproducible` ride in as tier 0.
        DeclaredAccuracyTier { max_ulp: Some(0), ..DeclaredAccuracyTier::default() },
    ] {
        let g = Guarantees {
            per_backend_ulp_tiers: vec![("cpu".into(), tier)],
            ..audited_guarantees()
        };
        assert_eq!(
            derive_audited_status(&g),
            AuditedStatus::Audited,
            "KISS-CONTRACT-6.8-0009: a tier carrying {tier:?} is a declared bound and must \
             derive `audited`"
        );
    }

    // NOT ASSERTED, deliberately: `bit_stability`.
    //
    // §6.8-0009's prohibition is on SETTING that field, and §6.8-0005 says in
    // terms that the derivation "reads this field and MUST NOT set it". Reading
    // is permitted. An earlier version of this test asserted the opposite — that
    // the derivation must not READ it — which would have failed any
    // implementation following the clause. Caught in review.
    //
    // The must-not-set half needs no assertion: `derive_audited_status` takes
    // `&Guarantees` and returns a value, so it cannot write the field at all. A
    // compile-time impossibility is a stronger guarantee than a runtime check,
    // and a runtime check for it would be a tautology.
}

// ---------------------------------------------------------------------------
// KISS-CONTRACT-6.8-0010 — the `unaudited` arm, and totality.
// ---------------------------------------------------------------------------

/// Enforces KISS-CONTRACT-6.8-0010 — the rule yields `unaudited` for a kernel
/// whose Guarantees do NOT declare a bounded precision against a named
/// `reference_function`, and yields no value outside `{audited, unaudited}`.
///
/// TEETH: a rule that defaults to `audited` when it cannot tell — the failure
/// direction that matters, because it grants trust that was never measured. Each
/// way of failing to declare a bound is asserted separately (no reference; no
/// tiers at all; tiers present but carrying no quantity), because a rule can
/// easily catch one and miss another. Totality is asserted rather than assumed:
/// the clause forbids producing a value neither arm yields.
#[test]
fn test_contract_unaudited_derivation_rule() {
    let base = audited_guarantees();

    let no_reference = Guarantees { reference_function: None, ..base.clone() };
    let no_tiers = Guarantees { per_backend_ulp_tiers: vec![], ..base.clone() };
    let empty_tier = Guarantees {
        per_backend_ulp_tiers: vec![("cuda:sm89".into(), unbounded_tier())],
        ..base.clone()
    };
    let neither =
        Guarantees { reference_function: None, per_backend_ulp_tiers: vec![], ..base.clone() };

    for (name, g) in [
        ("no named reference_function", &no_reference),
        ("no per-backend tiers at all", &no_tiers),
        ("a tier carrying no bound quantity", &empty_tier),
        ("neither a reference nor a tier", &neither),
    ] {
        assert_eq!(
            derive_audited_status(g),
            AuditedStatus::Unaudited,
            "KISS-CONTRACT-6.8-0010: {name} must derive `unaudited` — the rule is granting \
             audited trust to a kernel that declares no measured bound"
        );
    }

    // Totality: every input lands in exactly one of the two values, and the arms
    // are disjoint. A rule producing a third outcome cannot satisfy both arms.
    for g in [&no_reference, &no_tiers, &empty_tier, &neither, &base] {
        let d = derive_audited_status(g);
        assert!(
            d == AuditedStatus::Audited || d == AuditedStatus::Unaudited,
            "KISS-CONTRACT-6.8-0010: derivation produced a value neither arm yields"
        );
    }
    assert_ne!(
        derive_audited_status(&base),
        derive_audited_status(&neither),
        "KISS-CONTRACT-6.8-0010: the two arms collapsed to one value — the rule decides nothing"
    );
}

// ---------------------------------------------------------------------------
// KISS-CONFORM-6.13-0021 — the SUITE verifies the derivation.
// ---------------------------------------------------------------------------

/// Enforces KISS-CONFORM-6.13-0021 — KISS-Conform verifies the `audited_status`
/// derivation, i.e. a contract's DECLARED value must equal the value derived from
/// its own Guarantees.
///
/// This is the Conform-side obligation and it is not the same property as the
/// KISS-Contract clauses above: those say what the rule computes, this says the
/// suite must *confront the declaration with it*. A suite could implement the rule
/// perfectly and never compare it to anything.
///
/// TEETH: a verifier that reads the declared field and reports agreement with
/// itself — the check that always passes. Caught by the mismatch case, which pins
/// BOTH sides of the reported disagreement so a verifier cannot pass by declining
/// everything either.
#[test]
fn test_conform_contract_audited_status() {
    let g = audited_guarantees();

    // Agreement: the declared value matches what the Guarantees derive.
    assert!(
        verify_audited_status(AuditedStatus::Audited, &g).is_ok(),
        "KISS-CONFORM-6.13-0021: a contract whose declared status matches its own Guarantees \
         was rejected"
    );

    // The authored-constant signature: `audited` declared over Guarantees that
    // name no reference. This is precisely what §6.8-0008 forbids, and the suite
    // is what has to notice.
    let unmeasured = Guarantees { reference_function: None, ..g.clone() };
    let err = verify_audited_status(AuditedStatus::Audited, &unmeasured).expect_err(
        "KISS-CONFORM-6.13-0021: a contract declaring `audited` over Guarantees with no named \
         reference was ACCEPTED — the suite is trusting the declared field",
    );
    assert_eq!(err.declared, AuditedStatus::Audited);
    assert_eq!(
        err.derived,
        AuditedStatus::Unaudited,
        "KISS-CONFORM-6.13-0021: the mismatch must report the DERIVED value, not echo the \
         declared one"
    );

    // The opposite mismatch is also a mismatch: understating is still a contract
    // that does not describe itself. Asserting both directions stops a verifier
    // passing by rejecting everything.
    assert!(
        verify_audited_status(AuditedStatus::Unaudited, &g).is_err(),
        "KISS-CONFORM-6.13-0021: `unaudited` declared over Guarantees that derive `audited` \
         was accepted"
    );
    assert!(
        verify_audited_status(AuditedStatus::Unaudited, &unmeasured).is_ok(),
        "KISS-CONFORM-6.13-0021: the verifier rejects even a correct declaration — it declines \
         everything rather than comparing"
    );
}
