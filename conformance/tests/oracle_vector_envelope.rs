//! KISS-CONFORM-6.5-0017 — the oracle-vector envelope. Required fields, a typed decline on an
//! unrecognized `schema`/`schema_version` (the reader gap: before this clause `schema` was read and
//! never validated and `schema_version` was never read), and comparator selection under §6.8-0008's
//! precedence rather than by a vector's declared `class` directly.
use kiss_conformance::corpus;
use kiss_conformance::{comparator_for, Comparator, DeterminismClass};

fn arith() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-arith.json")).unwrap()
}
fn minmax() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-minmax-signed-zero.json"))
        .unwrap()
}

#[test]
fn test_conform_oracle_vector_envelope() {
    // --- Convention 16: the two committed bundles MUST still validate under the new clause. If the
    //     clause rejected an existing file, the clause would be wrong, not the file.
    let a = corpus::load(&arith()).expect("ops-arith.json validates under §6.5-0017");
    assert_eq!(a.schema, "kiss-oracle-vectors-v1.json");
    assert_eq!(a.schema_version, 1);
    let m = corpus::load(&minmax()).expect("ops-minmax-signed-zero.json validates under §6.5-0017");
    assert_eq!(m.schema_version, 1);

    // --- Unrecognized `schema` MUST typed-decline, not run as v1. Paired control: the real one loads.
    let bad_schema = arith().replace("kiss-oracle-vectors-v1.json", "kiss-oracle-vectors-v99.json");
    assert!(
        corpus::load(&bad_schema).is_err(),
        "§6.5-0017: an unrecognized `schema` must decline, not load as if v1"
    );
    assert!(corpus::load(&arith()).is_ok(), "control: the recognized `schema` loads");

    // --- Unrecognized `schema_version` MUST typed-decline. Paired control: version 1 loads (above).
    let bad_version = arith().replace("\"schema_version\": 1", "\"schema_version\": 999");
    assert!(
        corpus::load(&bad_version).is_err(),
        "§6.5-0017: an unrecognized `schema_version` must decline, not load as if v1"
    );

    // --- A MISSING required field (`schema_version`) MUST decline — it was never read before this
    //     clause, so a bundle lacking it used to load silently.
    let missing_version = arith().replace("\"schema_version\": 1,", "");
    assert!(
        corpus::load(&missing_version).is_err(),
        "§6.5-0017: a bundle missing `schema_version` must decline"
    );

    // --- Comparator selection is under §6.8-0008 PRECEDENCE, not the declared `class` directly. A
    //     §6.8-0005 op takes the split refinement even against an exact-byte declaration; applying the
    //     class directly is the silent false-red hazard the clause closes.
    assert_eq!(
        comparator_for("carg", DeterminismClass::ExactByte),
        Comparator::OpNamedRefinement("split"),
        "a §6.8-0005 op must select the refinement, not its declared class"
    );
    assert_eq!(
        comparator_for("add", DeterminismClass::ExactByte),
        Comparator::ClassDefault(DeterminismClass::ExactByte),
        "an ordinary op falls to its declared class (the default)"
    );

    // --- The invariance §6.5-0017 claims explicitly: every op in the two committed files falls to
    //     ClassDefault, so routing selection through the precedence changes nothing for them today.
    for cell in a.vectors.iter().chain(m.vectors.iter()) {
        assert!(
            matches!(comparator_for(&cell.op, cell.class), Comparator::ClassDefault(_)),
            "committed op `{}` must fall to its declared class (no refinement in the current corpus)",
            cell.op
        );
    }
}
