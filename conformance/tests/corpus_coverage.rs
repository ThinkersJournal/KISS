//! KISS-CONFORM-6.5-0008 (coverage completeness) and 6.5-0009 (inline wide-precision
//! stored value), enforced against the frozen bundle + the ops.md-derived manifest.
use kiss_conformance::{corpus, json};

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{}/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap()
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
    for op in &declared {
        assert!(covered.contains(op), "§6.5-0008: declared op `{op}` has no oracle vectors");
    }
}

#[test]
fn test_conform_oracle_vector_stores_wide_precision_value() {
    // §6.5-0009: every cell stores an inline expected value AND a certificate; no cell
    // defers its value to a live run.
    let corpus = corpus::load(&read("corpus/ops-arith.json")).unwrap();
    for c in &corpus.vectors {
        assert!(!c.expected.is_empty(), "§6.5-0009: tcId {} has no inline expected value", c.tc_id);
        assert!(c.has_certificate, "§6.5-0009: tcId {} lacks a certificate", c.tc_id);
    }
}
