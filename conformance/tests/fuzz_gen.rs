// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.6 — the generator and its validity self-check (the IN half of board 203 item 3).
//!
//! 6.6-0005 is backed under its §9 name. The generator test is deliberately NOT named for
//! 6.6-0001: that clause also names operand roles, ranks, dtypes and OpAttrs sub-vocabularies,
//! which this generator does not model, so 6.6-0001 stays recorded PARTIAL in `UNBACKED.tsv`.

use kiss_conformance::differential::SplitMix64;
use kiss_conformance::fuzz::{
    check_valid, corrupt, gen_valid, to_region, Malformation, OpSigs, Violation, ALL_MALFORMATIONS,
};
use kiss_conformance::grammar::{decode, encode, Decline};
use kiss_conformance::structural_dag::Decompositions;

const OPS_SPEC: &str = include_str!("../../spec/ops.md");

fn fixture() -> (Decompositions, OpSigs) {
    let t = Decompositions::from_ops_spec(OPS_SPEC);
    let s = OpSigs::from_decompositions(&t);
    // control: the arity table is read from the owner's rows and holds the ops this file needs.
    for (op, a) in [("add", 2usize), ("neg", 1), ("exp", 1), ("select", 3)] {
        assert_eq!(s.arity.get(op), Some(&a), "arity of `{op}` not derived from ops.md");
    }
    (t, s)
}

// harness-internal: tests the fuzz DAG generator; KISS-CONFORM-6.6-0001 is recorded PARTIAL in UNBACKED.tsv because the generator models no roles, ranks, dtypes or OpAttrs, so this test does not cite it
#[test]
fn fuzz_generator_valid_dags_reproducible_and_encodable() {
    let (t, s) = fixture();
    let mut distinct = std::collections::BTreeSet::new();
    for seed in 0..200u64 {
        let n_inputs = 1 + (seed % 3) as usize;
        let mk = || gen_valid(&mut SplitMix64::new(seed), &s, n_inputs, 4 + (seed % 5) as usize);
        let dag = mk();
        assert_eq!(dag, mk(), "seed {seed}: same seed must give the same DAG");
        assert_eq!(check_valid(&dag, &s, &t, n_inputs), Ok(()), "seed {seed}: generator emitted an invalid DAG");
        // the real Grammar codec accepts it and the wire form decodes back.
        let region = to_region(&dag, n_inputs as u32);
        let bytes = encode(&region).unwrap_or_else(|e| panic!("seed {seed}: encode declined {e:?}"));
        assert!(decode(&bytes).is_ok(), "seed {seed}: wire form did not decode");
        distinct.insert(format!("{:?}", dag.nodes));
    }
    assert!(distinct.len() > 150, "generator is degenerate: only {} distinct DAGs in 200 seeds", distinct.len());
}

/// KISS-CONFORM-6.6-0005 — malformed / near-boundary DAGs are generated and each is refused with
/// the expected TYPED violation; the same DAGs uncorrupted pass (so the self-check is not vacuous).
#[test]
fn test_conform_fuzzer_feeds_negative() {
    let (t, s) = fixture();
    let mut hit = std::collections::BTreeMap::<Malformation, u32>::new();
    for seed in 0..120u64 {
        let n_inputs = 1 + (seed % 3) as usize;
        let valid = gen_valid(&mut SplitMix64::new(seed), &s, n_inputs, 4 + (seed % 4) as usize);
        assert_eq!(check_valid(&valid, &s, &t, n_inputs), Ok(()), "KISS-CONFORM-6.6-0005: control must be valid");
        for kind in ALL_MALFORMATIONS {
            let Some(bad) = corrupt(&valid, kind, n_inputs) else { continue };
            let got = check_valid(&bad, &s, &t, n_inputs);
            let ok = match kind {
                Malformation::ForwardEdge | Malformation::SelfLoop => matches!(got, Err(Violation::ForwardEdge { .. })),
                Malformation::UnknownOp => matches!(got, Err(Violation::UnknownOp(_))),
                Malformation::ArityPlusOne | Malformation::ArityMinusOne => matches!(got, Err(Violation::Arity { .. })),
                Malformation::InputIndexAtBound => matches!(got, Err(Violation::InputOutOfRange { .. })),
                Malformation::RootPastEnd => got == Err(Violation::BadRoot),
            };
            assert!(ok, "KISS-CONFORM-6.6-0005: seed {seed} {kind:?} gave {got:?}");
            *hit.entry(kind).or_default() += 1;
        }
        // The codec's own negative path: declared n_inputs one too high / too low is a typed decline.
        let r = to_region(&valid, n_inputs as u32 + 1);
        assert!(matches!(encode(&r), Err(Decline::BindSetMismatch { .. })), "KISS-CONFORM-6.6-0005");
        if n_inputs > 1 {
            let r = to_region(&valid, n_inputs as u32 - 1);
            assert!(matches!(encode(&r), Err(Decline::BindIndexOutOfRange { .. })), "KISS-CONFORM-6.6-0005");
        }
    }
    for kind in ALL_MALFORMATIONS {
        assert!(hit.get(&kind).copied().unwrap_or(0) >= 100, "KISS-CONFORM-6.6-0005: class {kind:?} under-exercised: {hit:?}");
    }
}
