//! KISS-CONFORM-6.13-0006 (increment 1): the differential harness differences a
//! foreign C `add` kernel against the from-scratch oracle, and TWO dissimilar
//! correct kernels agree on floor semantics while a wrong one is CAUGHT.
//!
//! Scope: `add` is a primitive-floor op, so this exercises the ≥2-dissimilar-
//! impls-agree + harness-catches-divergence obligations of 6.13-0006. The
//! decomposition-resolution obligation (a non-primitive op → floor) is a later
//! increment and is NOT claimed here.

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
fn test_conform_ops_oracle_and_freeze_gate() {
    // KISS-CONFORM-6.13-0006 — the differential harness + ≥2-dissimilar-impls gate.
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
        "KISS-CONFORM-6.13-0006: add_a diverged from the oracle");
    assert!(run_binary(&out_b, &corpus).is_empty(),
        "KISS-CONFORM-6.13-0006: add_b diverged from the oracle");

    // (2) The two dissimilar impls agree with EACH OTHER, bit-for-bit (NaN-relaxed).
    for (i, (&x, &y)) in out_a.iter().zip(&out_b).enumerate() {
        assert!(x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
            "KISS-CONFORM-6.13-0006: the two dissimilar impls disagree at index {i}");
    }

    // (3) TEETH: a wrong kernel is CAUGHT — ≥1 divergence, reproducible by index.
    let caught = run_binary(&out_w, &corpus);
    assert!(!caught.is_empty(),
        "KISS-CONFORM-6.13-0006: the harness FAILED to catch a wrong kernel");
    let d = caught[0];
    // Re-running the single caught invocation reproduces the divergence.
    let repro = invoke_binary(add_wrong, &[d.a], &[d.b]);
    assert_eq!(repro[0].to_bits(), d.actual.to_bits(),
        "a caught divergence must reproduce from its recorded inputs");
}
