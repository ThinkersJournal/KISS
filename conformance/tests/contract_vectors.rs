//! KISS-Contract golden vector artifact (`conformance/corpus/contract_vectors.json`) — the
//! machine-readable golden document + decline set that now EXISTS in the corpus, fulfilling the
//! reference Appendix C makes to it ("the complete document … is carried in the machine-readable
//! golden-vector file") and giving `KISS-CONTRACT-8-0005` (a foreign reader must reproduce the
//! golden vectors) something to reproduce FROM. #349.
//!
//! Three obligations:
//!   1. FRESHNESS (#161): the committed artifact equals `emit_contract_vectors_json()` byte-for-byte,
//!      so a stale artifact fails CI — mirrors `structure_key_vectors` / the dtype-manifest gate.
//!   2. APPENDIX-C AGREEMENT: the codec's rendering of the blocks Appendix C shows matches the
//!      appendix's transcribed golden. A spec↔codec divergence here is a DECISION (which side moves
//!      is per-instance, escalated), never a silent reconcile — the appendix is hand-maintained prose.
//!   3. 16(d) PARTIAL, STATED not discovered: the artifact renders the 3 blocks Appendix C SHOWS, not
//!      the 7 its preamble PROMISES; blocks 4-7 have no builder. This pins that the artifact does not
//!      silently claim completeness.

use kiss_conformance::contract;

fn committed_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/contract_vectors.json")
}

/// (1) FRESHNESS — the committed artifact is byte-identical to a fresh generation. If this fails,
/// regenerate: `cargo run --bin emit_contract_vectors > conformance/corpus/contract_vectors.json`.
#[test]
fn test_contract_vectors_artifact_is_fresh() {
    let committed = std::fs::read_to_string(committed_path())
        .expect("conformance/corpus/contract_vectors.json must exist");
    let fresh = contract::emit_contract_vectors_json();
    assert_eq!(
        committed, fresh,
        "contract_vectors.json is STALE — regenerate with `cargo run --bin emit_contract_vectors > \
         conformance/corpus/contract_vectors.json`"
    );
}

/// (2) APPENDIX-C AGREEMENT — the codec reproduces the blocks Appendix C shows. The Identity block
/// must render byte-for-byte to the transcribed golden; the Semantics block carries the shown
/// `op_dag`; and the assembled golden document reads cleanly as a self-consistent contract (§6.11).
/// A mismatch is a spec↔codec DECISION, escalated per-instance, never reconciled silently — the
/// appendix is hand-maintained, as likely wrong as the codec.
#[test]
fn test_contract_vectors_render_matches_appendix_c() {
    let identity = contract::appendix_c_identity_block();
    let identity_text = std::str::from_utf8(&identity).unwrap();
    assert_eq!(
        identity_text,
        contract::APPENDIX_C_IDENTITY_GOLDEN,
        "codec's Identity block must render byte-for-byte to Appendix C's transcribed Identity golden"
    );

    let semantics = contract::appendix_c_semantics_block();
    let semantics_text = std::str::from_utf8(&semantics).unwrap();
    assert!(
        semantics_text.contains("op_dag = [Op{add; ; []}]"),
        "codec's Semantics block must carry Appendix C's shown one-node add DAG; got:\n{semantics_text}"
    );

    let doc = contract::appendix_c_golden_document();
    assert!(
        contract::read_document(&doc).is_ok(),
        "the Appendix C golden document must read as a self-consistent contract (§6.11)"
    );
}

/// (3) 16(d) PARTIAL, STATED — the artifact renders the 3 blocks Appendix C SHOWS, not the 7 its
/// preamble promises in the machine-readable file. blocks 4-7 have no builder. The artifact's own
/// note states this, so a reader cannot mistake it for the complete document; the seven-block
/// resolution is a NORMATIVE decision, filed separately.
#[test]
fn test_contract_vectors_states_the_three_of_seven_gap() {
    let json = contract::emit_contract_vectors_json();
    assert!(json.contains("\"blocks_shown\": 3"), "must state it renders 3 blocks");
    assert!(
        json.contains("\"blocks_promised_by_appendix\": 7"),
        "must state Appendix C promises 7 blocks in the machine-readable file"
    );
    assert!(
        json.contains("PARTIALLY fulfils") && json.contains("NORMATIVE decision"),
        "the 16(d) note must state the artifact is PARTIAL and the gap-closure is a normative decision"
    );
}

/// The decline set carries every single-fault corruption with its exact typed decline — DIFFABLE:
/// a foreign reader byte-diffs each malformed document and checks the pinned decline, not "an error".
#[test]
fn test_contract_vectors_carry_the_decline_set() {
    let json = contract::emit_contract_vectors_json();
    let n = contract::malformed_contract_vectors().len();
    assert!(n > 0, "there must be decline vectors");
    // every decline vector's name appears in the emitted artifact
    for nv in contract::malformed_contract_vectors() {
        assert!(
            json.contains(nv.name),
            "decline vector `{}` must appear in the emitted artifact",
            nv.name
        );
    }
}

/// The decline wire tags are the artifact's STABLE schema for `expect` — pinned per variant,
/// decoupled from the Rust identifier so a variant gaining a field or being renamed cannot silently
/// re-spell the foreign-reader artifact. The wildcard-free match in `wire_tag()` forces a NEW variant
/// to be given a spelling (compile error otherwise); this test pins the spellings themselves and
/// checks the emitted artifact carries only pinned tags — no `{:?}` Debug string leaks in.
#[test]
fn test_contract_decline_wire_tags() {
    use kiss_conformance::contract::ContractDecline::*;
    let pinned: &[(kiss_conformance::contract::ContractDecline, &str)] = &[
        (NoMagic, "no-magic"),
        (MalformedHeader, "malformed-header"),
        (UnknownKind { got: "x".into() }, "unknown-kind"),
        (UnknownVersion { got: "9".into() }, "unknown-version"),
        (BadLength { declared: 1, actual: 2 }, "bad-length"),
        (BadChecksum { declared: 1, computed: 2 }, "bad-checksum"),
        (Headingless, "headingless"),
        (MissingGuaranteesClass, "missing-guarantees-class"),
        (UnknownDeterminismClass { got: "z".into() }, "unknown-determinism-class"),
    ];
    for (decline, tag) in pinned {
        assert_eq!(decline.wire_tag(), *tag, "wire tag for {decline:?} is pinned");
    }

    // Every emitted decline vector's `expect` is its pinned tag — no Rust Debug string leaks in.
    let json = contract::emit_contract_vectors_json();
    for nv in contract::malformed_contract_vectors() {
        assert!(
            json.contains(&format!("\"expect\": \"{}\"", nv.expect.wire_tag())),
            "artifact must carry the pinned wire tag for {:?}",
            nv.expect
        );
    }
}
