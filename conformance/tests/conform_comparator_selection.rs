//! KISS-Conform §6.0 / §6.5 / §6.8 — determinism-class comparator **selection**.
//!
//! A sibling of `conform_comparators.rs` (which backs §6.8-0010, the computed-NaN
//! refinement). This module binds the determinism-class-**selection** clauses and the
//! three canonical comparators plus the oracle-differential harness — every one
//! provable by MUTATION against an already-built function in the crate's public
//! surface (`lib.rs`, `structural.rs`, `differential.rs`, `determinism.rs`,
//! `semantics.rs`). No new crate code is introduced.
//!
//! Each `#[test]` fn is named EXACTLY per its clause's `*Test:*` tag, cites its own
//! KISS-CONFORM clause id as a full token in the body (the reverse half of the §6.1
//! binding), and asserts a behavioral verdict that FLIPS on a plausible codec/spec
//! drift — never a substring grep, a self-comparison, or a derive check. All
//! cross-references to other clauses use the §-shorthand so no foreign or skip-list
//! clause id is accidentally reverse-bound.
//!
//! The comparator is **selected, never chosen** (§2.5): a test author never picks how
//! two results are compared; the clause's declared determinism/fidelity class (the
//! canonical KISS-Ops enum, imported verbatim) selects it. The distinct teeth of the
//! four selection clauses are kept apart:
//!   * §6.0-0002 — the verdict is a PURE FUNCTION of the class argument.
//!   * §6.8-0006 — the selection key is the OP's advertised class (Monoid/Combine),
//!     not a free class literal.
//!   * §6.8-0009 — the DECLARED class is authoritative even when an op "looks like"
//!     another comparator's family (Max is order-invariant-in-value yet exact-byte).
//!   * §6.0-0001 — the enum has exactly three members (an exhaustive `match` is the
//!     compile-time guard); the split comparator is a free fn, not a fourth member.

use kiss_conformance::differential::{agree, corpus_f32, diff_binary};
use kiss_conformance::determinism::{atom_determinism_class, TRANSCENDENTAL_ATOMS};
use kiss_conformance::semantics::add;
use kiss_conformance::structural::{
    compare_monoid_reduced_f32, compare_order_invariant, compare_reduced_f32,
    compare_scattered_f32, reassoc_bound_f32, Combine, Monoid,
};
use kiss_conformance::{
    compare, compare_c32_transcendental, compare_f32, ulp_distance_f32, DeterminismClass,
};

// The f32 one ULP above 1.0 — a value with DIFFERENT bits but adjacent magnitude, so
// a ULP-tolerance comparator at bound 1 accepts it while a byte/exact-byte comparator
// rejects it. This single fact separates the ULP arm from the exact-byte arm.
fn one_ulp_above_one() -> f32 {
    f32::from_bits(1.0f32.to_bits() + 1)
}

// A genuine floating-point reassociation divergence: the SAME multiset {1e8, -1e8,
// 1.0} summed in two valid orders. Order A keeps the 1.0 (`((0+1e8)-1e8)+1 = 1.0`);
// order B loses it to rounding (`(1e8+1)-1e8 = 0.0`). Both are conformant orderings
// of a nondeterministic float atomic-add / sum reduction, computed here with the
// oracle `add` atom so the divergence is real, not stipulated.
fn reassoc_pair() -> (f32, f32) {
    let order_a = [1e8_f32, -1e8, 1.0];
    let order_b = [1e8_f32, 1.0, -1e8];
    let s1 = order_a.iter().fold(0.0f32, |acc, &x| add(acc, x));
    let s2 = order_b.iter().fold(0.0f32, |acc, &x| add(acc, x));
    (s1, s2)
}

// The contract-declared order-invariant tolerance justified by the reassociation
// bound `(n-1)·u·Σ|xᵢ|` (doubled: two orders each within one bound of the exact sum).
fn reassoc_tol() -> f32 {
    2.0 * reassoc_bound_f32(3, 1e8_f32 + 1e8 + 1.0)
}

// =============================================================================
// §6.8 — the three canonical comparators
// =============================================================================

/// KISS-CONFORM-6.8-0001 — the **exact-byte** comparator is a bit/byte-identical
/// compare (memcmp) and MUST NOT be relaxed to a tolerance for an exact-byte clause.
#[test]
fn test_conform_exact_byte_comparator() {
    // Clause: KISS-CONFORM-6.8-0001.
    // Byte path: an identical buffer matches; a ONE-BYTE drift is caught.
    assert!(compare(DeterminismClass::ExactByte, &[1, 2], &[1, 2]).is_ok());
    assert!(
        compare(DeterminismClass::ExactByte, &[1, 2], &[1, 3]).is_err(),
        "a single-byte difference MUST fail the exact-byte comparator"
    );

    // f32 path: the comparator is bit-exact, NEVER relaxed to the passed tolerance.
    // ±0 differ only in the sign bit; even an UNBOUNDED tolerance MUST NOT rescue it.
    assert!(
        compare_f32(DeterminismClass::ExactByte, -0.0, 0.0, u64::MAX).is_err(),
        "exact-byte MUST distinguish -0.0 from +0.0 regardless of any tolerance"
    );
    // A 1-ULP different-bits pair with a maxed-out bound is STILL rejected: this
    // proves the exact-byte arm ignores the ulp_bound entirely (it is not a ULP arm).
    assert!(
        compare_f32(DeterminismClass::ExactByte, 1.0, one_ulp_above_one(), u64::MAX).is_err(),
        "exact-byte MUST ignore the ULP bound — a 1-ULP bit difference still fails"
    );
    // MUTATION: relaxing the exact-byte arm to any tolerance ≥ 1 ULP would make the
    // ±0 case and the 1-ULP case PASS — both asserts above then fail.
}

/// KISS-CONFORM-6.8-0002 — the **ULP/tolerance** comparator compares within the
/// declared per-target ULP bound and MUST NOT be a byte compare; it applies to any op
/// whose decomposition transitively contains a transcendental atom.
#[test]
fn test_conform_ulp_comparator() {
    // Clause: KISS-CONFORM-6.8-0002.
    let one = 1.0f32;
    let nudged = one_ulp_above_one();
    // Precondition: the two values have DIFFERENT bits but are 1 ULP apart.
    assert_ne!(one.to_bits(), nudged.to_bits());
    assert_eq!(ulp_distance_f32(one, nudged), 1);

    // Within-bound: accepted at bound 1 — so this is NOT a byte compare (a byte
    // compare would reject two different bit patterns whatever the bound).
    assert!(
        compare_f32(DeterminismClass::UlpTolerance, one, nudged, 1).is_ok(),
        "ULP/tolerance MUST accept a 1-ULP difference at bound 1 (not a byte compare)"
    );
    // Out-of-bound: rejected at bound 0 — so the bound is genuinely enforced.
    assert!(
        compare_f32(DeterminismClass::UlpTolerance, one, nudged, 0).is_err(),
        "ULP/tolerance MUST reject a 1-ULP difference at bound 0"
    );
    // A large gap (millions of ULP) is rejected even at bound 1 — not a constant Ok.
    assert!(compare_f32(DeterminismClass::UlpTolerance, 1.0, 2.0, 1).is_err());

    // Selection: every transcendental atom carries the ULP/tolerance class, so the
    // ULP comparator is the one selected for it (§6.8-0002 / KISS-OPS §6.8).
    for &atom in TRANSCENDENTAL_ATOMS {
        assert_eq!(
            atom_determinism_class(atom),
            Some(DeterminismClass::UlpTolerance),
            "transcendental atom `{atom}` MUST be ULP/tolerance, never exact-byte"
        );
    }
    // The named ULP atoms of the plan, explicitly.
    for &atom in &["exp", "log", "sin", "sqrt", "atan2"] {
        assert_eq!(atom_determinism_class(atom), Some(DeterminismClass::UlpTolerance));
    }
    // Contrast so the classifier is not a constant returning UlpTolerance: an
    // arithmetic atom is exact-byte.
    assert_eq!(atom_determinism_class("add"), Some(DeterminismClass::ExactByte));
    // MUTATION: making the ULP arm a bits-equal compare fails the within-bound case;
    // reclassifying a transcendental (e.g. `sqrt`) to exact-byte fails the loop.
}

/// KISS-CONFORM-6.8-0004 — the **order-invariant/nondeterministic** comparator MUST
/// NOT require byte-exact reproduction, and the tolerance MUST be the one declared in
/// the contract Guarantees, never an implementation-chosen implicit default.
#[test]
fn test_conform_nondeterministic_comparator() {
    // Clause: KISS-CONFORM-6.8-0004.
    let (s1, s2) = reassoc_pair();
    // Precondition: two valid visit orders genuinely diverge in bits.
    assert_eq!(s1, 1.0);
    assert_eq!(s2, 0.0);
    assert_ne!(s1.to_bits(), s2.to_bits());

    // Exact-byte is the WRONG comparator here — it rejects the real reassociation.
    assert!(
        compare_f32(DeterminismClass::ExactByte, s1, s2, 0).is_err(),
        "exact-byte MUST reject a nondeterministic reassociation divergence"
    );
    // The order-invariant comparator ACCEPTS it within the contract-declared bound.
    let tol = reassoc_tol();
    assert!(
        compare_order_invariant(&[s1], &[s2], tol, 0.0).is_ok(),
        "order-invariant MUST accept the divergence within the declared tolerance"
    );
    // The tolerance is the PASSED PARAMETER, not a hidden default: at a declared
    // tolerance of 0.0 the very same divergence MUST be rejected.
    assert!(
        compare_order_invariant(&[s1], &[s2], 0.0, 0.0).is_err(),
        "the tolerance MUST be the declared parameter — 0.0 rejects the divergence"
    );
    // MUTATION: if the comparator used a hidden non-zero default instead of the passed
    // abs_tol, the 0.0 case would still accept the reassociation → this assert fails.
}

/// KISS-CONFORM-6.8-0005 — the complex-transcendental **split comparator**: exact-bit
/// on the sign of every zero-valued component, ULP/tolerance on the magnitude. It is a
/// hybrid of the three canonical classes and MUST NOT be a fourth enum member.
#[test]
fn test_conform_split_comparator() {
    // Clause: KISS-CONFORM-6.8-0005. `compare_c32_transcendental` is a FREE FUNCTION,
    // not a `DeterminismClass` member (keeping the §6.0-0001 enum at three members).

    // (a) A zero-valued component matches its SIGN BIT exactly. Expected -0.0, actual
    // +0.0 → fail — even though a plain ULP comparator would accept it, because
    // ulp_distance(-0.0, +0.0) == 1. The contrast is the whole point of the split.
    assert_eq!(ulp_distance_f32(-0.0, 0.0), 1);
    assert!(
        compare_f32(DeterminismClass::UlpTolerance, 0.0, -0.0, 1).is_ok(),
        "a PLAIN ULP comparator wrongly accepts a flipped zero sign (bound ≥ 1)"
    );
    assert!(
        compare_c32_transcendental([0.0, 1.0], [-0.0, 1.0], u64::MAX).is_err(),
        "split comparator MUST reject a flipped zero-component sign, any bound"
    );

    // (b) A ±π branch endpoint matches its SIGN exactly. Expected +π, actual -π → fail.
    let pi = std::f32::consts::PI;
    assert!(
        compare_c32_transcendental([1.0, -pi], [1.0, pi], u64::MAX).is_err(),
        "split comparator MUST reject a flipped ±π branch-endpoint sign"
    );

    // (c) An ordinary component is compared under ULP-tolerance within the bound.
    assert!(
        compare_c32_transcendental([1.0, one_ulp_above_one()], [1.0, 1.0], 1).is_ok(),
        "an ordinary component MUST pass within the ULP bound"
    );
    assert!(
        compare_c32_transcendental([1.0, one_ulp_above_one()], [1.0, 1.0], 0).is_err(),
        "an ordinary component MUST fail outside the ULP bound"
    );
    // MUTATION: routing the zero-valued component through the ULP arm (dropping the
    // exact-sign check) makes case (a) PASS → its assert fails.
}

/// KISS-CONFORM-6.8-0006 — the comparator MUST be **selected** by the class travelling
/// with the artifact (an op advertises its class); a test author MUST NOT override the
/// class-selected comparator. The selection key is the OP (its advertised class).
#[test]
fn test_conform_comparator_selection_rule() {
    // Clause: KISS-CONFORM-6.8-0006. The dispatchers `compare_monoid_reduced_f32` /
    // `compare_scattered_f32` key on the OP's advertised class (Monoid::class_f32 /
    // Combine::class_f32), NOT on a free class literal the caller supplies.
    let (s1, s2) = reassoc_pair();
    let tol = reassoc_tol();

    // A Sum reduction advertises order-invariant, so its dispatcher ACCEPTS the
    // reassociation divergence within the declared tolerance.
    assert_eq!(Monoid::Sum.class_f32(), DeterminismClass::OrderInvariant);
    assert!(
        compare_monoid_reduced_f32(Monoid::Sum, s1, s2, tol, 0.0).is_ok(),
        "a Sum reduction (order-invariant) MUST accept a reassociation diff within tol"
    );

    // A Max reduction advertises exact-byte, so its dispatcher BYTE-REJECTS a genuine
    // (non-±0) bit difference — even with a HUGE tolerance that a tolerance comparator
    // would accept. The tolerance cannot leak into the exact-byte path.
    assert_eq!(Monoid::Max.class_f32(), DeterminismClass::ExactByte);
    let nudged = one_ulp_above_one();
    assert_ne!(1.0f32.to_bits(), nudged.to_bits());
    assert!(
        compare_monoid_reduced_f32(Monoid::Max, 1.0, nudged, 1e30, 1e30).is_err(),
        "a Max reduction (exact-byte) MUST byte-reject a real bit diff despite huge tol"
    );

    // The same for the scatter combine dispatcher: AtomicAdd is order-invariant,
    // AtomicMax is exact-byte — the combine, not the caller, selects the comparator.
    assert_eq!(Combine::AtomicAdd.class_f32(), DeterminismClass::OrderInvariant);
    assert!(compare_scattered_f32(Combine::AtomicAdd, &[s1], &[s2], tol, 0.0).is_ok());
    assert_eq!(Combine::AtomicMax.class_f32(), DeterminismClass::ExactByte);
    assert!(
        compare_scattered_f32(Combine::AtomicMax, &[1.0], &[nudged], 1e30, 1e30).is_err(),
        "an AtomicMax scatter (exact-byte) MUST byte-reject a real bit diff despite huge tol"
    );
    // MUTATION: if the dispatcher ignored the monoid/combine and used a fixed
    // comparator, Sum would REJECT (if fixed exact-byte) or Max would ACCEPT (if fixed
    // order-invariant) → one of the asserts fails.
}

/// KISS-CONFORM-6.8-0009 — the exact-byte comparator is the only admissible one for
/// the exact-byte class; the "order-invariant-in-value" families (max/min monoid
/// reductions, assign/atomic-max/atomic-min scatter) CARRY exact-byte, and the
/// declared class is authoritative when an op appears to match two enumerations.
#[test]
fn test_conform_exact_byte_admissibility() {
    // Clause: KISS-CONFORM-6.8-0009. Max/Min "look" order-invariant (they are
    // invariant in VALUE across fold orders) yet the declared class is exact-byte.
    assert_eq!(Monoid::Max.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Monoid::Min.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Combine::Assign.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Combine::AtomicMax.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Combine::AtomicMin.class_f32(), DeterminismClass::ExactByte);

    // The declared class is AUTHORITATIVE: a Max reduction of two plainly-different
    // values (1.0 vs 2.0 — a full unit apart, which ANY tolerance ≥ 1 would accept) is
    // REJECTED under a huge declared tolerance, because the byte path ignores it.
    assert!(
        compare_monoid_reduced_f32(Monoid::Max, 1.0, 2.0, 1e30, 1e30).is_err(),
        "Max carries exact-byte: a huge tolerance MUST NOT leak into its compare"
    );
    // The contrast pins that the difference is the CLASS, not the values: the same
    // 1.0-vs-2.0 pair under the order-invariant Sum monoid (with the same huge tol) is
    // ACCEPTED. Same values, same tolerance, opposite verdict — selected by class.
    assert_eq!(Monoid::Sum.class_f32(), DeterminismClass::OrderInvariant);
    assert!(
        compare_monoid_reduced_f32(Monoid::Sum, 1.0, 2.0, 1e30, 1e30).is_ok(),
        "Sum carries order-invariant: the huge declared tolerance DOES apply"
    );
    // MUTATION: reclassifying Max to order-invariant, or letting the declared tol leak
    // into the Max byte path, makes the 1.0-vs-2.0 Max case PASS → its assert fails.
}

// =============================================================================
// §6.0 — class import and class-driven selection
// =============================================================================

/// KISS-CONFORM-6.0-0002 — for every clause KISS-Conform evaluates, the comparator
/// MUST be **selected** by the clause's declared determinism/fidelity class and MUST
/// NOT be chosen by the test author.
#[test]
fn test_conform_comparator_selected_by_class() {
    // Clause: KISS-CONFORM-6.0-0002. `compare_reduced_f32(class, a, e, abs_tol,
    // rel_tol)` takes the class as a first-class ARGUMENT; the verdict is a pure
    // function of it. Hold (a, e, tol) fixed to a reassociation-magnitude-apart pair
    // and flip ONLY the class: the verdict flips Ok/Err.
    let (s1, s2) = reassoc_pair();
    let tol = reassoc_tol();
    assert_ne!(s1.to_bits(), s2.to_bits());

    let ok = compare_reduced_f32(DeterminismClass::OrderInvariant, s1, s2, tol, 0.0);
    let err = compare_reduced_f32(DeterminismClass::ExactByte, s1, s2, tol, 0.0);
    assert!(
        ok.is_ok(),
        "OrderInvariant class MUST accept the divergence within the declared tolerance"
    );
    assert!(
        err.is_err(),
        "ExactByte class MUST reject the SAME values with the SAME tolerance"
    );
    // MUTATION: if `compare_reduced_f32` hardcoded one comparator (ignoring `class`),
    // both calls — identical but for the class argument — would return the SAME verdict
    // → one of these two asserts fails.
}

/// KISS-CONFORM-6.0-0001 — KISS-Conform MUST import the canonical determinism/fidelity
/// enum verbatim and MUST NOT re-fork, extend, or re-order it; the three comparators of
/// §6.8 are the selection targets of exactly these three members.
#[test]
fn test_conform_determinism_enum_imported() {
    // Clause: KISS-CONFORM-6.0-0001. The EXHAUSTIVE `match` below — with NO wildcard
    // arm — is the compile-time guard: a fourth enum member (e.g. a `Split` arm) would
    // make it non-exhaustive and BREAK THE BUILD. Each of the three members drives a
    // BEHAVIORALLY DISTINCT comparator, so collapsing two members to one comparator
    // fails a distinctness assert. (The "imported from KISS-Ops" provenance is a
    // source property, not behaviorally observable; the behavioral proxy bound here is
    // "exactly three members, each a distinct comparator target".)
    let (s1, s2) = reassoc_pair();
    let tol = reassoc_tol();
    let nudged = one_ulp_above_one();

    for &class in &[
        DeterminismClass::ExactByte,
        DeterminismClass::UlpTolerance,
        DeterminismClass::OrderInvariant,
    ] {
        match class {
            DeterminismClass::ExactByte => {
                // byte-rejects a ±0 sign flip even at an unbounded tolerance
                assert!(compare_f32(class, -0.0, 0.0, u64::MAX).is_err());
            }
            DeterminismClass::UlpTolerance => {
                // ULP-accepts a 1-ULP different-bits pair, rejects it at bound 0
                assert!(compare_f32(class, 1.0, nudged, 1).is_ok());
                assert!(compare_f32(class, 1.0, nudged, 0).is_err());
            }
            DeterminismClass::OrderInvariant => {
                // order-invariant-accepts a reassociation divergence within tol, and
                // rejects the same divergence under the exact-byte class (distinctness)
                assert!(compare_reduced_f32(class, s1, s2, tol, 0.0).is_ok());
                assert!(compare_reduced_f32(DeterminismClass::ExactByte, s1, s2, tol, 0.0).is_err());
            }
        }
    }

    // The split comparator (§6.8-0005) is exercised as a FREE FUNCTION — it needs no
    // enum member, so the enum stays three members (no `DeterminismClass::Split`).
    assert!(compare_c32_transcendental([0.0, 1.0], [-0.0, 1.0], u64::MAX).is_err());
    // MUTATION: adding a fourth enum member makes the `match` non-exhaustive → the file
    // fails to COMPILE; collapsing two comparator arms fails a distinctness assert.
}

// =============================================================================
// §6.5 — the oracle-differential harness
// =============================================================================

/// KISS-CONFORM-6.5-0001 — KISS-Conform MUST provide an oracle-differential harness in
/// which a reference CPU oracle computes each op independently and the harness compares
/// an implementation's output against it under the op's declared class comparator.
#[test]
fn test_conform_oracle_differential_harness() {
    // Clause: KISS-CONFORM-6.5-0001. `diff_binary` differences a candidate against the
    // independent `semantics::add` CPU oracle over a reproducible corpus, under the
    // exact-byte / NaN-equivalence agreement relation `agree` (§6.8-0001 with the
    // computed-NaN relaxation). The point is that an INCORRECT candidate is CAUGHT.
    let corpus = corpus_f32(0xC0FFEE, 64);

    // A conforming candidate (the oracle itself) produces NO divergence.
    assert!(
        diff_binary(add, add, &corpus).is_empty(),
        "a candidate identical to the oracle MUST show no divergence"
    );

    // A plainly wrong candidate (mul where add is required) DOES diverge.
    assert!(
        !diff_binary(add, |a, b| a * b, &corpus).is_empty(),
        "a wrong candidate (mul vs add) MUST be caught by the differential harness"
    );

    // A NaN-SUPPRESSING candidate returns a finite 0.0 where the oracle computes NaN
    // (e.g. add(+inf, -inf) = NaN). The agreement relation MUST catch the one-sided
    // NaN — a propagate-vs-suppress bug is exactly the conformance-relevant fact.
    let nan_suppress = |a: f32, b: f32| {
        let r = add(a, b);
        if r.is_nan() {
            0.0
        } else {
            r
        }
    };
    let divs = diff_binary(add, nan_suppress, &corpus);
    assert!(
        !divs.is_empty(),
        "a NaN-suppressing candidate MUST be caught (one-sided NaN is a mismatch)"
    );
    for d in &divs {
        assert!(
            d.oracle.is_nan() && d.candidate == 0.0,
            "every caught divergence here is an oracle NaN vs a suppressed finite 0.0"
        );
    }
    // The agreement relation directly: a computed NaN vs a finite value is a mismatch,
    // while two NaNs (any payload) agree — so agree() is not a byte-fragile compare.
    assert!(!agree(f32::NAN, 0.0));
    assert!(agree(f32::NAN, f32::from_bits(0x7FC0_1234)));
    // MUTATION: if `agree` returned true for everything, all three divergence sets
    // above would be empty → the two "MUST be caught" asserts fail.
}
