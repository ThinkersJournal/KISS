//! T9 step 3 — the transcendental COMPUTED-NaN bundle round-trips through `load_cell`, and
//! every emitted `expected` NaN is QUIET. A minter emitting a SIGNALING expected would be a
//! SILENT conformance hole: `load_cell` accepts it (is-NaN ∧ provenance present satisfies the
//! biconditional), but the cell would then demand an sNaN where a conformant implementation
//! produces a qNaN — the suite would enforce the exact §6.16-0010 defect it exists to catch.
//! So this test re-derives every `expected` from the oracle (`semantics::*`) AND asserts
//! quietness by name (through the comparator the corpus feeds).
//!
//! POPULATION EMITTED (stated so complete is distinguishable from sampled — the minter's own
//! doc carries the same list and the NOT-emitted set):
//!
//! - Family B, §6.16-0010 (arithmetic quiets a signaling NaN OPERAND): exp/log/sin of an sNaN
//!   input → a QUIET NaN. The discriminating case — a promote-and-round impl that returns the
//!   operand's signaling bits unchanged fails the quietness comparison.
//! - Family A, §6.2-0001/IEEE invalid-op (a NaN MINTED from a non-NaN input): log(x<0),
//!   log(-inf), sin(±inf) → a QUIET NaN.
//!
//! NOT emitted, and why (see the minter doc): qNaN-operand propagation (quiet-in→quiet-out is
//! non-discriminating), any MOVED transcendental NaN (none exist — a transcendental contains
//! arithmetic, so §6.16-0010 forbids the raw-bit move), narrow dtypes (atoms are f64/f32 only;
//! f8e4m3fn additionally has vacuous quietness), and an exp domain-mint (exp mints no NaN from
//! a non-NaN input — a genuine absence, not an omission).

use kiss_conformance::nan_provenance::{compare_nan_output, is_nan, NanProvenance::Computed};
use kiss_conformance::{corpus, semantics};

fn f32_be(b: &[u8]) -> f32 {
    f32::from_bits(u32::from_be_bytes(b.try_into().expect("f32 input is 4 bytes")))
}
fn f64_be(b: &[u8]) -> f64 {
    f64::from_bits(u64::from_be_bytes(b.try_into().expect("f64 input is 8 bytes")))
}

/// The oracle's output bytes for a unary transcendental cell — the SAME front door the minter
/// uses, so a minter that diverged from `semantics` reds here.
fn oracle_be(op: &str, dtype: &str, input: &[u8]) -> Vec<u8> {
    match (op, dtype) {
        ("exp", "f32") => semantics::exp(f32_be(input)).to_bits().to_be_bytes().to_vec(),
        ("log", "f32") => semantics::log(f32_be(input)).to_bits().to_be_bytes().to_vec(),
        ("sin", "f32") => semantics::sin(f32_be(input)).to_bits().to_be_bytes().to_vec(),
        ("exp", "f64") => semantics::exp_f64(f64_be(input)).to_bits().to_be_bytes().to_vec(),
        ("log", "f64") => semantics::log_f64(f64_be(input)).to_bits().to_be_bytes().to_vec(),
        ("sin", "f64") => semantics::sin_f64(f64_be(input)).to_bits().to_be_bytes().to_vec(),
        _ => panic!("unexpected op/dtype in transcendental-NaN bundle: {op} {dtype}"),
    }
}

/// A canonical QUIET NaN and a canonical SIGNALING NaN in `dtype`, for the by-name quietness
/// assertion. (Both admit-sNaN dtypes; the bundle emits only f32/f64.)
fn canon(dtype: &str) -> (Vec<u8>, Vec<u8>) {
    match dtype {
        "f32" => (0x7FC0_0000u32.to_be_bytes().to_vec(), 0x7F80_0001u32.to_be_bytes().to_vec()),
        "f64" => (
            0x7FF8_0000_0000_0000u64.to_be_bytes().to_vec(),
            0x7FF0_0000_0000_0001u64.to_be_bytes().to_vec(),
        ),
        _ => panic!("bundle should emit only f32/f64: {dtype}"),
    }
}

#[test]
fn transcendental_nan_bundle_round_trips_computed_and_quiet() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/ops-transcendental-nan.json"
    ))
    .expect("bundle must be committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert!(!c.vectors.is_empty(), "bundle is non-empty");

    for v in &c.vectors {
        let where_ = format!("{} {} tc {}", v.op, v.dtype, v.tc_id);
        // Every cell here is a COMPUTED NaN.
        assert_eq!(v.nan_provenance, Some(Computed), "{where_}: nan_provenance must be computed");
        assert_eq!(v.inputs.len(), 1, "{where_}: transcendentals are unary");

        // 1. The emitted `expected` equals the oracle for this exact input — catches ANY minter
        //    bug (wrong value, wrong dtype, and wrong quietness, since the oracle is quiet).
        let oracle = oracle_be(&v.op, &v.dtype, &v.inputs[0]);
        assert_eq!(v.expected, oracle, "{where_}: emitted expected must equal semantics::{}", v.op);
        assert!(is_nan(&v.dtype, &v.expected), "{where_}: expected must be a NaN");

        // 2. Expected-quietness BY NAME (the architect's required assertion), asserted through the
        //    comparator the corpus feeds: a computed comparison against a canonical QUIET NaN
        //    passes, and against a canonical SIGNALING NaN fails — pinning expected as quiet.
        let (quiet, snan) = canon(&v.dtype);
        assert!(
            compare_nan_output(&v.dtype, &v.expected, &quiet, Computed).is_ok(),
            "{where_}: computed NaN expected MUST be quiet (§6.16-0010)"
        );
        assert!(
            compare_nan_output(&v.dtype, &v.expected, &snan, Computed).is_err(),
            "{where_}: computed NaN expected MUST NOT be signaling"
        );
    }

    // Both families are actually present (a sampled bundle that dropped one would still pass the
    // per-cell loop above). Family B: an sNaN operand quieted. Family A: a domain-error mint.
    let has_snan_operand = c.vectors.iter().any(|v| is_nan(&v.dtype, &v.inputs[0]));
    let has_domain_mint = c.vectors.iter().any(|v| !is_nan(&v.dtype, &v.inputs[0]));
    assert!(has_snan_operand, "family B (sNaN operand → quiet, §6.16-0010) must be present");
    assert!(has_domain_mint, "family A (domain-error mint → quiet, §6.2-0001) must be present");
}
