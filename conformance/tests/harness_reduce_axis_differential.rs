#![cfg(windows)]
//! KISS-OPS-6.11-0002 (axis reduction) + KISS-CONFORM-6.13-0006b (Contract-sourced
//! comparator selection) via the differential harness: two dissimilar C
//! `reduce(monoid, axis=1)` kernels agree with the resolved oracle per output row,
//! while a WRONG-AXIS kernel (reduces axis 0), a wrong-VALUE kernel (double-counts),
//! and a wrong-OP kernel (min for max) are caught. The comparator is NOT hardcoded by
//! monoid: it is SELECTED from a determinism class parsed out of a real Contract
//! Guarantees block (order-invariant for Sum, exact-byte for Max), so this wires
//! 6.13-0006b into the shipped C-kernel differential — not only a synthetic test.
//!
//! Honest scope: reverse-cites KISS-OPS-6.11-0002 (forward test test_ops_reduce_monoids)
//! and KISS-CONFORM-6.13-0006b (forward test test_conform_ops_class_comparator_selection).
//! Does NOT cite §6.11-0008 (its stride-0 keepdim BROADCAST teeth are not exercised —
//! only the extent-1 output shape).

mod common;
use common::compile_and_load_axis_reduce;
use kiss_conformance::contract::{field_line, parse_guarantees_class, Value};
use kiss_conformance::harness::abi::invoke_axis_reduce;
use kiss_conformance::harness::corpus::tagged_axis_corpus;
use kiss_conformance::harness::differ::run_axis_reduce_advertised;
use kiss_conformance::harness::resolver::{reduce_axis_abs_tol, reduce_axis_oracle};
use kiss_conformance::structural::Monoid;
use kiss_conformance::DeterminismClass;

/// The determinism class advertised by a real Contract Guarantees block carrying
/// `determinism_class = <token>`, round-tripped through the codec. The differential's
/// comparator is SOURCED from this parsed advertisement (KISS-CONFORM-6.13-0006b),
/// never chosen by monoid.
fn advertised_class(token: &str) -> DeterminismClass {
    let mut body = b"[section:1:identity]\n".to_vec();
    body.extend_from_slice(b"[section:6:guarantees]\n");
    body.extend_from_slice(field_line("determinism_class", &Value::Str(token.into())).as_bytes());
    parse_guarantees_class(&body).expect("advertised class parses")
}

// Enforces KISS-OPS-6.11-0002 (`reduce` folds an axis with an associative monoid) and
// KISS-CONFORM-6.13-0006b (the differential comparator is selected from the op's
// advertised determinism class, never hardcoded): the Sum path's comparator comes from
// a contract advertising `order-invariant/nondeterministic`, the Max path's from one
// advertising `exact-byte`, both parsed from a real Guarantees block via
// parse_guarantees_class. 6.11-0002's forward test is test_ops_reduce_monoids;
// 6.13-0006b's is test_conform_ops_class_comparator_selection — this is a
// cross-implementation witness for both, wiring the advertised selection into the
// shipped C-kernel differential rather than leaving it a self-contained mechanism.
#[test]
fn test_ops_reduce_axis_differential() {
    let Some(sum_a) = compile_and_load_axis_reduce("reduce_axis1_sum_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C axis-reduce slice needs cl.exe");
        return;
    };
    let sum_b = compile_and_load_axis_reduce("reduce_axis1_sum_b").unwrap();
    let sum_wrong = compile_and_load_axis_reduce("reduce_axis1_sum_wrong").unwrap();
    let sum_offband = compile_and_load_axis_reduce("reduce_axis1_sum_offband").unwrap();
    let max_a = compile_and_load_axis_reduce("reduce_axis1_max_a").unwrap();
    let max_b = compile_and_load_axis_reduce("reduce_axis1_max_b").unwrap();
    let max_wrong = compile_and_load_axis_reduce("reduce_axis1_max_wrong").unwrap();

    // Comparators SOURCED from parsed contract advertisements, never the monoid: the
    // whole point of 6.13-0006b. Sum's true class is order-invariant, Max's exact-byte.
    let sum_class = advertised_class("order-invariant/nondeterministic");
    let max_class = advertised_class("exact-byte");

    // The Sum reassociation band is only meaningful if some conforming candidate lands
    // a NONZERO distance from the oracle yet is still accepted. Prove it: at least one
    // corpus vector must make the pairwise kernel differ from the sequential oracle in
    // raw bits (a band of 0 would then reject it). Guards a vacuous "all agreed" pass.
    let mut band_exercised = false;

    for v in tagged_axis_corpus(0x5EED) {
        let ein = [v.extents[0] as i64, v.extents[1] as i64];
        let eout = [v.extents[0] as i64, 1]; // axis-1 keepdim: out is rows×1
        let cols = v.extents[1];

        // ---- Sum: two dissimilar orders both agree within the advertised band ----
        let sum_oracle = reduce_axis_oracle(&v.data, v.extents, 1, Monoid::Sum);
        let sum_tol = reduce_axis_abs_tol(&v.data, v.extents, 1);
        let a = invoke_axis_reduce(sum_a, &v.data, ein, eout);
        let b = invoke_axis_reduce(sum_b, &v.data, ein, eout);
        assert!(
            run_axis_reduce_advertised(&a, &sum_oracle, "reduce", Monoid::Sum, sum_class, &sum_tol).is_empty(),
            "Sum sum_a (sequential) diverged from oracle: {:?}",
            v.extents
        );
        assert!(
            run_axis_reduce_advertised(&b, &sum_oracle, "reduce", Monoid::Sum, sum_class, &sum_tol).is_empty(),
            "Sum sum_b (pairwise) diverged from oracle beyond band: {:?}",
            v.extents
        );
        if b.iter().zip(&sum_oracle).any(|(&x, &e)| x.to_bits() != e.to_bits()) {
            band_exercised = true;
        }

        // ---- Sum wrong-AXIS (reduces axis 0) is caught — the failure mode a
        // rank-1-only harness cannot express. The wrong kernel writes `cols` cells:
        // a non-square shape makes the output length differ; the asymmetric square
        // makes at least one value differ. ----
        let w = invoke_axis_reduce(sum_wrong, &v.data, ein, [1, cols as i64]);
        let m = sum_oracle.len().min(w.len());
        // NB: `sum_tol` is the per-ROW tolerance applied to the wrong kernel's per-COLUMN
        // output — deliberate and harmless (a wrong-axis divergence is O(magnitude) >= 1,
        // every row tolerance ~1e-6, so which axis's tolerance we use can't change the
        // verdict; the slice just keeps lengths aligned when rows == cols, the square).
        assert!(
            w.len() != sum_oracle.len()
                || !run_axis_reduce_advertised(&w[..m], &sum_oracle[..m], "reduce", Monoid::Sum, sum_class, &sum_tol[..m]).is_empty(),
            "the band wrongly ABSORBED a wrong-axis (axis-0) reduction: {:?}",
            v.extents
        );

        // ---- Sum wrong-VALUE, right shape: a magnitude error the band must REJECT.
        // sum_offband double-counts each row's first element, so the output is the same
        // rows-shaped buffer (length/axis checks are blind to it) but every row is off
        // by its first element (>= 1), which exceeds that row's band on every corpus
        // vector — including the 1e8 exerciser (error 1e8 vs band ~179). This is the
        // band's REJECTION edge; sum_a/sum_b only prove acceptance. ----
        let ob = invoke_axis_reduce(sum_offband, &v.data, ein, eout);
        assert_eq!(ob.len(), sum_oracle.len(), "offband must keep the oracle's shape");
        assert!(
            !run_axis_reduce_advertised(&ob, &sum_oracle, "reduce", Monoid::Sum, sum_class, &sum_tol).is_empty(),
            "the band wrongly ABSORBED a wrong-VALUE (double-counted) sum: {:?}",
            v.extents
        );

        // ---- Max: two dissimilar orders agree exact-byte (±0-canon); the wrong op
        // (min) is caught. Max ignores the band (zero tol). ----
        let max_oracle = reduce_axis_oracle(&v.data, v.extents, 1, Monoid::Max);
        let zero_tol = vec![0.0f32; max_oracle.len()];
        let ma = invoke_axis_reduce(max_a, &v.data, ein, eout);
        let mb = invoke_axis_reduce(max_b, &v.data, ein, eout);
        assert!(
            run_axis_reduce_advertised(&ma, &max_oracle, "reduce", Monoid::Max, max_class, &zero_tol).is_empty(),
            "Max max_a (forward) diverged: {:?}",
            v.extents
        );
        assert!(
            run_axis_reduce_advertised(&mb, &max_oracle, "reduce", Monoid::Max, max_class, &zero_tol).is_empty(),
            "Max max_b (reverse) diverged: {:?}",
            v.extents
        );
        // A per-row MIN differs from MAX only when a row has ≥2 distinct values —
        // a single-column shape ([8,1]) cannot express it, so guard on cols ≥ 2.
        if cols >= 2 {
            let mw = invoke_axis_reduce(max_wrong, &v.data, ein, eout);
            assert!(
                !run_axis_reduce_advertised(&mw, &max_oracle, "reduce", Monoid::Max, max_class, &zero_tol).is_empty(),
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
