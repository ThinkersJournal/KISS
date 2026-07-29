//! Differential-conformance harness — increment 1 (capability demonstration).
//!
//! Differences a foreign C `add` kernel — invoked through the KISS-Contract §6.5
//! positional C-ABI — against the from-scratch `kiss_conformance::semantics`
//! oracle, and proves TWO dissimilar correct kernels agree on floor semantics
//! while a wrong one is CAUGHT.
//!
//! Relationship to Conform §6.13-0006 (deliberately NOT claimed as backing yet):
//! this test realizes only the §6.13-0006 SUBSET increment 1 actually exercises —
//! owning an independent CPU-oracle differential harness that shares no lowering
//! with the impls under test (Rust oracle vs foreign C), and the freeze-gate
//! obligation that ≥2 dissimilar impls agree on a primitive-floor op's semantics
//! (plus catch-a-wrong-kernel). It does NOT satisfy §6.13-0006's other MUST
//! obligations: resolving a NON-primitive op's decomposition to the floor (`add`
//! is itself a floor op, so there is no decomposition to resolve), and selecting
//! the comparator from the advertised per-op class (§7.4-0001) rather than the
//! hardcoded exact-byte comparator. So this test is written to NOT bind
//! §6.13-0006 in the coverage ledger by BOTH traceability directions: its fn is
//! deliberately NOT named `test_conform_ops_oracle_and_freeze_gate` (the clause's
//! §9 *Test:* tag — the forward direction), and it cites the clause only in the
//! bare `§`-form (which kiss_trace's `CLAUSE_ID` does not match — the reverse
//! direction). So §6.13-0006 stays UNTESTED until a follow-up increment renames
//! this fn to the tag AND lands the decomposition + advertised-class obligations.
//!
//! FOLLOW-UP: §6.13-0006 bundles four MUST obligations in one clause; per the
//! §6.13 preamble ("each obligation stated as its own atomic clause"), it should
//! be atomized so the harness-ownership + ≥2-impls-agree part can be backed
//! independently of decomposition-resolution and class-based comparator selection.

mod common;
use common::compile_and_load;
use kiss_conformance::harness::abi::{invoke_binary, BinaryKernel};
use kiss_conformance::harness::corpus::tagged_corpus;
use kiss_conformance::harness::differ::run_binary;

/// Run a kernel over the corpus and return its outputs (1:1 with the corpus).
fn outputs_of(k: BinaryKernel, corpus: &[kiss_conformance::harness::corpus::Vector]) -> Vec<f32> {
    let a: Vec<f32> = corpus.iter().map(|v| v.a).collect();
    let b: Vec<f32> = corpus.iter().map(|v| v.b).collect();
    invoke_binary(k, &a, &b)
}

#[test]
fn differential_harness_add_freeze_gate() {
    // Conform §6.13-0006 (inc.1 subset) — the differential harness + ≥2-dissimilar-impls gate.
    let Some(add_a) = compile_and_load("add_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C differential slice needs cl.exe");
        return;
    };
    let add_b = compile_and_load("add_b").unwrap();
    let add_wrong = compile_and_load("add_wrong").unwrap();

    let corpus = tagged_corpus(0x5EED_1234, 256);
    let out_a = outputs_of(add_a, &corpus);
    let out_b = outputs_of(add_b, &corpus);
    let out_w = outputs_of(add_wrong, &corpus);

    // (1) Both dissimilar correct impls agree with the oracle: zero divergences.
    assert!(run_binary(&out_a, &corpus).is_empty(),
        "Conform §6.13-0006 (inc.1 subset): add_a diverged from the oracle");
    assert!(run_binary(&out_b, &corpus).is_empty(),
        "Conform §6.13-0006 (inc.1 subset): add_b diverged from the oracle");

    // (2) The two dissimilar impls agree with EACH OTHER, bit-for-bit (NaN-relaxed).
    for (i, (&x, &y)) in out_a.iter().zip(&out_b).enumerate() {
        assert!(x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
            "Conform §6.13-0006 (inc.1 subset): the two dissimilar impls disagree at index {i}");
    }

    // (3) TEETH: a wrong kernel is CAUGHT — ≥1 divergence, reproducible by index.
    let caught = run_binary(&out_w, &corpus);
    assert!(!caught.is_empty(),
        "Conform §6.13-0006 (inc.1 subset): the harness FAILED to catch a wrong kernel");
    let d = caught[0];
    // Re-running the single caught invocation reproduces the divergence.
    let repro = invoke_binary(add_wrong, &[d.a], &[d.b]);
    assert_eq!(repro[0].to_bits(), d.actual.to_bits(),
        "a caught divergence must reproduce from its recorded inputs");
}
