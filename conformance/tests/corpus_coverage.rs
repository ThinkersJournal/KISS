//! KISS-CONFORM-6.5-0008 (coverage completeness) and 6.5-0009 (inline wide-precision
//! stored value), enforced against the frozen bundle + the ops.md-derived manifest.
use kiss_conformance::{corpus, json};

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{}/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The committed corpus op-coverage floor (#459). Mirrors the numerator floor the full ratchet
/// (`kiss_ops.py --corpus-coverage`) enforces, so a coverage LOSS reds in the cargo job too.
fn read_corpus_floor() -> usize {
    for line in read("CORPUS_COVERAGE_FLOOR.tsv").lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('\t') {
            if k.trim() == "declared" {
                return v.trim().parse().expect("floor `declared` is an integer");
            }
        }
    }
    panic!("CORPUS_COVERAGE_FLOOR.tsv has no `declared` row");
}

#[test]
fn test_conform_oracle_vector_coverage_complete() {
    // §6.5-0008: every op in the manifest's declared coverage set MUST appear in the
    // corpus. (Plan A's declared set is the exact-byte arithmetic floor; it grows per slice.)
    let corpus = corpus::load(&read("corpus/ops-arith.json")).unwrap();
    let manifest = json::parse(&read("corpus/op_manifest.json")).unwrap();
    let declared: Vec<&str> = manifest.get("declared_coverage_set").unwrap()
        .as_arr().unwrap().iter().filter_map(|j| j.as_str()).collect();
    let covered: std::collections::BTreeSet<&str> =
        corpus.vectors.iter().map(|c| c.op.as_str()).collect();
    // #459: a FLOOR-AWARE guard, not the old `!declared.is_empty()` — which was degenerate-blind
    // (n=1 passes emptiness while being as vacuous as 0 over the derived op domain). The full
    // fraction ratchet (declared/derived-total, with regression/stale/domain-moved verdicts) is
    // `kiss_ops.py --corpus-coverage`; this mirrors its numerator floor so a coverage LOSS reds
    // here too. The floor MUST be ≥ 1, which subsumes the old emptiness check.
    let floor = read_corpus_floor();
    assert!(floor >= 1, "#459: the corpus-coverage floor must be ≥ 1 (a 0 floor is the vacuity the ratchet forbids)");
    assert!(
        declared.len() >= floor,
        "§6.5-0008 / #459: declared coverage {} is below the committed floor {} \
         (CORPUS_COVERAGE_FLOOR.tsv) — an op lost its oracle-vector coverage",
        declared.len(), floor
    );
    for op in &declared {
        assert!(covered.contains(op), "§6.5-0008: declared op `{op}` has no oracle vectors");
    }
}

/// Bit width of the significand (including any implicit leading bit) for a dtype's
/// storage format. Used as the floor that a cell's stored certificate precision must
/// meet or exceed (§6.5-0009).
fn significand_bits(dtype: &str) -> u64 {
    match dtype {
        "f32" => 24,
        "f64" => 53,
        "f16" => 11,
        "bf16" => 8,
        other => panic!("unknown dtype {other} — extend significand_bits"),
    }
}

#[test]
fn test_conform_oracle_vector_stores_wide_precision_value() {
    // §6.5-0009: every cell stores an inline expected value AND a certificate; no cell
    // defers its value to a live run. The certificate must also attest a stabilized
    // precision at least as wide as the cell's own dtype — a certificate present but
    // claiming e.g. 1 bit (or an empty object) would be vacuous for a clause whose whole
    // point is a *wide-precision* stored value.
    //
    // Plan B tightens this further: for transcendental cells (ops in
    // op_manifest.json's transcendental_atoms) the stored value must be computed
    // STRICTLY WIDER than the compute dtype (certificate_precision_bits > dtype width),
    // since a transcendental's exact value is irrational and can only be trusted if the
    // oracle computed it at extra precision before rounding down. An exact-byte cell
    // (e.g. `add` of two representable floats) legitimately equals the dtype width,
    // since the true mathematical result is exactly representable.
    let corpus = corpus::load(&read("corpus/ops-arith.json")).unwrap();
    for c in &corpus.vectors {
        assert!(!c.expected.is_empty(), "§6.5-0009: tcId {} has no inline expected value", c.tc_id);
        assert!(c.has_certificate, "§6.5-0009: tcId {} lacks a certificate", c.tc_id);
        let required = significand_bits(&c.dtype);
        match c.certificate_precision_bits {
            Some(bits) => assert!(
                bits >= required,
                "§6.5-0009: tcId {} certificate_precision_bits {bits} is narrower than dtype {} floor {required}",
                c.tc_id, c.dtype
            ),
            None => panic!(
                "§6.5-0009: tcId {} certificate lacks stabilized_precision_bits (or it is not a number)",
                c.tc_id
            ),
        }
    }
}
