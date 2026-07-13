//! KISS-Conform §6.5 randomized differential loop.
//!
//! The value of a differential harness is that it *catches* a wrong
//! implementation, not merely that a right one passes. These tests run over a
//! reproducible corpus (edge cases + seeded-random bit patterns) and demonstrate
//! both directions.

use kiss_conformance::differential::*;
use kiss_conformance::semantics::*;

/// An independently-written but semantically-identical `fmax_ieee` — the "correct
/// implementation under test". Different code shape, same §6.15 semantics.
fn fmax_alt(a: f32, b: f32) -> f32 {
    match (a.is_nan(), b.is_nan()) {
        (true, _) => b,
        (false, true) => a,
        (false, false) => if a >= b { a } else { b },
    }
}

const SEED: u64 = 0xC0FFEE;
const N: usize = 400;

#[test]
fn correct_candidate_agrees_over_the_whole_corpus() {
    let corpus = corpus_f32(SEED, N);
    let divs = diff_binary(fmax_ieee, fmax_alt, &corpus);
    assert!(divs.is_empty(), "a correct fmax candidate diverged: {:?}", divs.first());
}

#[test]
fn differential_catches_the_nan_propagation_bug() {
    // The exact bug the spec's four-way min/max split guards against: implementing
    // IEEE fmax with torch-style NaN-*propagating* max (max_prop). The randomized
    // corpus — which includes NaN — must catch it, and every divergence must be a
    // NaN-operand case (the only place the two ops differ).
    let corpus = corpus_f32(SEED, N);
    let divs = diff_binary(fmax_ieee, max_prop, &corpus);
    assert!(!divs.is_empty(), "differential failed to catch the NaN-propagation bug");
    assert!(
        divs.iter().all(|d| d.a.is_nan() || d.b.is_nan()),
        "divergence outside a NaN operand — the two ops should agree everywhere else"
    );
}

#[test]
fn relu_diverges_from_max_x_zero_under_fuzz() {
    // relu is NOT max(x,0): they diverge (deterministically) at NaN.
    let corpus = corpus_f32(0x1234, N);
    let divs = diff_binary(|a, _| relu(a), |a, _| naive_max_x_zero(a), &corpus);
    assert!(!divs.is_empty(), "relu vs max(x,0) should diverge");
    assert!(divs.iter().any(|d| d.a.is_nan()), "the NaN divergence must appear");
}

#[test]
fn oracle_is_self_consistent_under_fuzz() {
    // sanity: each oracle agrees with itself over the whole corpus (0 divergences)
    let corpus = corpus_f32(7, 300);
    assert!(diff_binary(max_prop, max_prop, &corpus).is_empty());
    assert!(diff_binary(fmin_ieee, fmin_ieee, &corpus).is_empty());
    assert!(diff_binary(add, add, &corpus).is_empty());
    assert!(diff_binary(copysign, copysign, &corpus).is_empty());
}

#[test]
fn corpus_is_reproducible() {
    // same seed -> byte-identical corpus (deterministic fuzzer reproducibility)
    let a: Vec<u32> = corpus_f32(42, 100).iter().map(|x| x.to_bits()).collect();
    let b: Vec<u32> = corpus_f32(42, 100).iter().map(|x| x.to_bits()).collect();
    assert_eq!(a, b);
    // the corpus actually contains NaN and infinities (so the loop exercises them)
    let c = corpus_f32(42, 400);
    assert!(c.iter().any(|x| x.is_nan()));
    assert!(c.iter().any(|x| x.is_infinite()));
}
