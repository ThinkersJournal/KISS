#![cfg(windows)]
//! KISS-CONFORM-6.13-0006a (increment 2): the differential harness resolves the
//! non-primitive `reduce_mean` to its floor decomposition and proves ≥2 dissimilar
//! implementations agree ON THE DECOMPOSITION within the reassociation band, while a
//! wrong divisor is caught. The oracle is the resolved floor value (resolver.rs), the
//! comparator is the order-invariant/reassociation-band comparator (§6.0-0004), NOT
//! exact-byte — legitimate reassociation between the two summation orders is accepted.

mod common;
use common::compile_and_load_reduce;
use kiss_conformance::harness::abi::invoke_reduce;
use kiss_conformance::harness::resolver::{reduce_mean_abs_tol, reduce_mean_oracle};
use kiss_conformance::structural::compare_reduced_f32;
use kiss_conformance::DeterminismClass;

// A deterministic corpus of reduction inputs: a few fixed vectors that keep the two
// correct orders inside the band and the wrong divisor outside it.
fn inputs() -> Vec<Vec<f32>> {
    vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        vec![1e6, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],   // large-magnitude spread
        (0..64).map(|i| (i as f32 * 0.1) - 3.0).collect(), // 64 mixed-sign
        vec![0.5; 100],
    ]
}

#[test]
fn test_conform_ops_decomposition_agreement() {
    let Some(mean_a) = compile_and_load_reduce("mean_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C reduction slice needs cl.exe");
        return;
    };
    let mean_b = compile_and_load_reduce("mean_b").unwrap();
    let mean_wrong = compile_and_load_reduce("mean_wrong").unwrap();

    for xs in inputs() {
        let oracle = reduce_mean_oracle(&xs);
        let tol = reduce_mean_abs_tol(&xs);
        let a = invoke_reduce(mean_a, &xs);
        let b = invoke_reduce(mean_b, &xs);
        let w = invoke_reduce(mean_wrong, &xs);

        // (1) Both dissimilar orders agree with the resolved decomposition within the band.
        compare_reduced_f32(DeterminismClass::OrderInvariant, a, oracle, tol, 0.0)
            .expect("KISS-CONFORM-6.13-0006a: mean_a diverged beyond the reassociation band");
        compare_reduced_f32(DeterminismClass::OrderInvariant, b, oracle, tol, 0.0)
            .expect("KISS-CONFORM-6.13-0006a: mean_b diverged beyond the reassociation band");
        // (2) ...and with each other.
        compare_reduced_f32(DeterminismClass::OrderInvariant, a, b, tol, 0.0)
            .expect("KISS-CONFORM-6.13-0006a: the two dissimilar orders disagree beyond the band");

        // (3) TEETH: the wrong divisor is CAUGHT — outside the band. (Skip degenerate
        // n where ÷n and ÷(n-1) coincide within tol; all `inputs()` have n>=8 and a
        // nonzero mean, so the wrong result is well outside the band.)
        assert!(
            compare_reduced_f32(DeterminismClass::OrderInvariant, w, oracle, tol, 0.0).is_err(),
            "KISS-CONFORM-6.13-0006a: the band wrongly ABSORBED a ÷(n-1) error on {xs:?}"
        );
    }
}
