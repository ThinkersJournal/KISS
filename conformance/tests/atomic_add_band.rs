// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.11-0024: the floating-point `atomic-add` combine is a tolerance-cell whose band is the one
//! KISS-CONTRACT §6.8-0012 defines (CireSnave 2026-11-12, board 417 "A": "adopt the float atomic add band in
//! v1, bound defined as 6.8-0012 defines it so the clauses agree").

use kiss_conformance::tolerance::{
    atomic_add_admits_f32, atomic_add_cell_f32, comparator_threshold, emit_atomic_add_band_vectors_json,
    threshold_admits_f32, AccumulatorBand, DeclaredTolerance, Threshold, ATOMIC_ADD_BAND_VECTORS,
};

fn band() -> AccumulatorBand {
    AccumulatorBand { k: 1.0, eps_a: f64::from(f32::EPSILON) / 2.0 }
}

fn spec(file: &str) -> String {
    let p = format!("{}/../spec/{file}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(p).expect("read spec").split_whitespace().collect::<Vec<_>>().join(" ")
}

// Backs[enforced]: KISS-OPS-6.11-0024 — the committed atomic-add band vectors equal the emitter's output; each replays to the pinned verdict (reassociation inside the band admitted, outside refused, dest counted in the partials, a zero band exact), whatever accumulation order produced the candidate
#[test]
fn test_ops_scatter_atomic_add_band() {
    let committed = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/atomic_add_band_vectors.json")).expect("corpus file");
    assert_eq!(
        committed.replace("\r\n", "\n"),
        emit_atomic_add_band_vectors_json(),
        "stale: cargo run --bin emit_atomic_add_band_vectors > conformance/corpus/atomic_add_band_vectors.json"
    );
    let want = [true, true, false, true, false, false];
    for ((name, dest, updates, cand), want) in ATOMIC_ADD_BAND_VECTORS.iter().zip(want) {
        assert_eq!(atomic_add_admits_f32(band(), *dest, updates, *cand), want, "{name}");
    }
    // the reference is the ascending-order evaluation with dest first, each accumulate rounded to f32.
    let c = atomic_add_cell_f32(0.0, &[1.0e8, 1.0, -1.0e8, 1.0]);
    assert_eq!((c.reference, c.n, c.max_partial_sum), (1.0, 4, 1.0e8));
    let c = atomic_add_cell_f32(16_777_216.0, &[1.0, 1.0]);
    assert_eq!((c.reference, c.max_partial_sum), (16_777_216.0, 16_777_216.0), "dest is in the partials");
    // a position no update reaches: N = 0, band 0, exactly dest.
    assert!(atomic_add_admits_f32(band(), 3.5, &[], 3.5));
    assert!(!atomic_add_admits_f32(band(), 3.5, &[], f32::from_bits(3.5f32.to_bits() + 1)));
    // order independence: every permutation of the updates is admitted against the one reference.
    let u = [1.0e8f32, 1.0, -1.0e8, 1.0];
    for perm in [[0, 1, 2, 3], [0, 2, 1, 3], [1, 3, 0, 2], [2, 0, 3, 1]] {
        let mut p = 0.0f32;
        for i in perm {
            p += u[i];
        }
        assert!(atomic_add_admits_f32(band(), 0.0, &u, p), "order {perm:?} gave {p}");
    }
}

// Backs[enforced]: KISS-CONTRACT-6.8-0012 — the atomic-add cell's band is the 6.8-0012 band, not a second formula: the same threshold function yields it, it is absolute, and both clauses name each other
#[test]
fn test_contract_accumulator_tolerance_band() {
    let c = atomic_add_cell_f32(0.0, &[1.0e8, 1.0, -1.0e8, 1.0]);
    let th = comparator_threshold(Some(DeclaredTolerance::Band(band())), c.n, c.max_partial_sum).unwrap();
    assert_eq!(th, Threshold::AbsoluteBand(4.0 * f64::from(f32::EPSILON) / 2.0 * 1.0e8));
    // absolute, not result-relative: a result near zero is still admitted within the band.
    assert!(threshold_admits_f32(th, 2.0, 1.0));
    let ops = spec("ops.md");
    let contract = spec("contract.md");
    assert!(ops.contains("**KISS-OPS-6.11-0024** — The `scatter` `atomic-add` combine"), "clause missing");
    assert!(ops.contains("ULP-of-A band of KISS-CONTRACT-6.8-0012"));
    assert!(contract.contains("and for the floating-point `atomic-add` cell of KISS-OPS §6.11-0024"));
}
