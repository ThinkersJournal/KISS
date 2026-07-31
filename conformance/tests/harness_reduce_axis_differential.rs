#![cfg(windows)]
//! KISS-OPS-6.11-0002 (axis reduction) via the differential harness: two dissimilar
//! C `reduce(monoid, axis=1)` kernels agree with the resolved oracle per output row
//! — within the reassociation band for Sum, bit-exact (±0-canon) for Max — while a
//! WRONG-AXIS kernel (reduces axis 0) and a wrong-value kernel (computes min for max)
//! are caught. The comparator is dispatched BY MONOID (Sum → order-invariant band,
//! Max → exact-byte).
//!
//! Honest scope: this is the multi-class FOUNDATION for increment 3b's
//! Contract-sourced comparator selection, which is what binds KISS-CONFORM-6.13-0006b.
//! 3a does NOT claim 0006b — the comparator here is chosen STRUCTURALLY by monoid,
//! not read from an advertised Contract determinism class.

mod common;
use common::compile_and_load_axis_reduce;
use kiss_conformance::harness::abi::invoke_axis_reduce;
use kiss_conformance::harness::corpus::tagged_axis_corpus;
use kiss_conformance::harness::differ::run_axis_reduce;
use kiss_conformance::harness::resolver::{reduce_axis_abs_tol, reduce_axis_oracle};
use kiss_conformance::structural::Monoid;

// Enforces KISS-OPS-6.11-0002 (`reduce` folds an axis with an associative monoid):
// Sum and Max axis-1 folds are differentially checked against the resolved oracle
// across two dissimilar C orderings, and a wrong-axis (axis-0) / wrong-op (min-for-
// max) kernel is caught. This is a REVERSE-citation witness — the clause's forward
// test is `test_ops_reduce_monoids`; this adds a cross-implementation corroboration.
// Deliberately cites NOTHING else: §6.11-0008's teeth are the extent-1/stride-0
// keepdim BROADCAST view (not exercised here — only the extent-1 output shape), and
// the Contract-sourced comparator of §6.13-0006b is increment 3b, not this test.
#[test]
fn test_ops_reduce_axis_differential() {
    let Some(sum_a) = compile_and_load_axis_reduce("reduce_axis1_sum_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C axis-reduce slice needs cl.exe");
        return;
    };
    let sum_b = compile_and_load_axis_reduce("reduce_axis1_sum_b").unwrap();
    let sum_wrong = compile_and_load_axis_reduce("reduce_axis1_sum_wrong").unwrap();
    let max_a = compile_and_load_axis_reduce("reduce_axis1_max_a").unwrap();
    let max_b = compile_and_load_axis_reduce("reduce_axis1_max_b").unwrap();
    let max_wrong = compile_and_load_axis_reduce("reduce_axis1_max_wrong").unwrap();

    // The Sum reassociation band is only meaningful if some conforming candidate
    // actually lands a NONZERO distance from the oracle yet is still accepted. Prove
    // it: at least one corpus vector must make the pairwise kernel differ from the
    // sequential oracle in raw bits (a band of 0 would then reject it). Guards
    // against a vacuous "everything agreed exactly" pass.
    let mut band_exercised = false;

    for v in tagged_axis_corpus(0x5EED) {
        let ein = [v.extents[0] as i64, v.extents[1] as i64];
        let eout = [v.extents[0] as i64, 1]; // axis-1 keepdim: out is rows×1
        let cols = v.extents[1];

        // ---- Sum: two dissimilar orders both agree with the oracle within band ----
        let sum_oracle = reduce_axis_oracle(&v.data, v.extents, 1, Monoid::Sum);
        let sum_tol = reduce_axis_abs_tol(&v.data, v.extents, 1);
        let a = invoke_axis_reduce(sum_a, &v.data, ein, eout);
        let b = invoke_axis_reduce(sum_b, &v.data, ein, eout);
        assert!(
            run_axis_reduce(&a, &sum_oracle, Monoid::Sum, &sum_tol).is_empty(),
            "Sum sum_a (sequential) diverged from oracle: {:?}",
            v.extents
        );
        assert!(
            run_axis_reduce(&b, &sum_oracle, Monoid::Sum, &sum_tol).is_empty(),
            "Sum sum_b (pairwise) diverged from oracle beyond band: {:?}",
            v.extents
        );
        if b.iter().zip(&sum_oracle).any(|(&x, &e)| x.to_bits() != e.to_bits()) {
            band_exercised = true;
        }

        // ---- Sum wrong-AXIS (reduces axis 0) is caught — the new failure mode a
        // rank-1-only harness cannot express. The wrong kernel writes `cols` cells:
        // a non-square shape makes the output length differ; the asymmetric square
        // makes at least one value differ. ----
        let w = invoke_axis_reduce(sum_wrong, &v.data, ein, [1, cols as i64]);
        let m = sum_oracle.len().min(w.len());
        assert!(
            w.len() != sum_oracle.len()
                || !run_axis_reduce(&w[..m], &sum_oracle[..m], Monoid::Sum, &sum_tol[..m]).is_empty(),
            "the band wrongly ABSORBED a wrong-axis (axis-0) reduction: {:?}",
            v.extents
        );

        // ---- Max: two dissimilar orders agree exact-byte; the wrong op (min) is
        // caught. Max ignores the band (zero tol). ----
        let max_oracle = reduce_axis_oracle(&v.data, v.extents, 1, Monoid::Max);
        let zero_tol = vec![0.0f32; max_oracle.len()];
        let ma = invoke_axis_reduce(max_a, &v.data, ein, eout);
        let mb = invoke_axis_reduce(max_b, &v.data, ein, eout);
        assert!(
            run_axis_reduce(&ma, &max_oracle, Monoid::Max, &zero_tol).is_empty(),
            "Max max_a (forward) diverged: {:?}",
            v.extents
        );
        assert!(
            run_axis_reduce(&mb, &max_oracle, Monoid::Max, &zero_tol).is_empty(),
            "Max max_b (reverse) diverged: {:?}",
            v.extents
        );
        // A per-row MIN differs from MAX only when a row has ≥2 distinct values —
        // a single-column shape ([8,1]) cannot express it, so guard on cols ≥ 2.
        if cols >= 2 {
            let mw = invoke_axis_reduce(max_wrong, &v.data, ein, eout);
            assert!(
                !run_axis_reduce(&mw, &max_oracle, Monoid::Max, &zero_tol).is_empty(),
                "wrong Max (computed min) was not caught: {:?}",
                v.extents
            );
        }
    }

    assert!(
        band_exercised,
        "no corpus vector exercised the Sum reassociation band (pairwise never \
         differed from the sequential oracle in raw bits) — the band tolerance is \
         untested; widen the large-magnitude-spread corpus vector"
    );
}
