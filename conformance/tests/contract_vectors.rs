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

/// (3) THE APPENDIX AND THE ARTIFACT ARE ONE DOCUMENT, ASSERTED PER BLOCK (#496).
///
/// This replaces `test_contract_vectors_states_the_three_of_seven_gap`, which asserted the
/// artifact DECLARED a 3-of-7 gap. The gap is closed, so a test whose subject is the gap's
/// statement has no subject left; keeping it green would have required keeping the gap.
///
/// It reads Appendix C out of `spec/contract.md` and requires each of the seven codec-rendered
/// blocks to appear there VERBATIM. The old `test_contract_vectors_render_matches_appendix_c`
/// pinned Identity byte-for-byte, substring-checked Semantics, and never looked at the rest --
/// so it passed with the codec rendering seven blocks against an appendix showing three. A test
/// named "render matches appendix C" that cannot see a four-block divergence is the vacuity
/// family: its name is a claim about coverage its assertions do not make.
#[test]
fn test_appendix_c_shows_every_block_the_codec_renders() {
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/contract.md"))
        .expect("spec/contract.md must be readable");
    // NORMALIZE LINE ENDINGS BEFORE COMPARING. render_block emits LF; read_to_string returns
    // the file bytes verbatim, so on a CRLF checkout every appendix line ends CRLF and every
    // contains() below fails on a document that is textually identical.
    //
    // Asymmetric in a way that hides it: Rust NORMALIZES CRLF inside string literals, so the
    // sibling APPENDIX_C_*_GOLDEN constants are LF whatever the .rs file holds -- a test
    // comparing against THOSE passes on any checkout, while this one, which reads the spec
    // from disk, does not. It surfaced on a rebase that re-checked-out spec/contract.md as
    // CRLF, having passed moments earlier on identical content stored as LF.
    let spec = spec.replace("\r\n", "\n");
    let start = spec.find("## Appendix C").expect("spec must carry Appendix C");
    let appendix = &spec[start..];

    let blocks: [(&str, Vec<u8>); 7] = [
        ("identity", contract::appendix_c_identity_block()),
        ("semantics", contract::appendix_c_semantics_block()),
        ("interface", contract::appendix_c_interface_block()),
        ("dispatch", contract::appendix_c_dispatch_block()),
        ("capabilities", contract::appendix_c_capabilities_block()),
        ("guarantees", contract::appendix_c_guarantees_block()),
        ("provenance", contract::appendix_c_provenance_block()),
    ];

    for (name, bytes) in &blocks {
        let text = std::str::from_utf8(bytes).expect("block must be UTF-8");
        assert!(
            appendix.contains(text),
            "Appendix C does not show the codec's `{name}` block verbatim. The appendix and the \
             machine-readable golden are one document (§8-0004/-0005 name Appendix C as the \
             target a foreign reader reproduces), so a block the codec renders and the appendix \
             omits is a divergence in the thing under obligation.\n\
             --- codec renders ---\n{text}"
        );
    }

    // CONTROL: the extraction is not vacuously true. A block the codec does NOT render must be
    // absent -- otherwise `contains` would pass for anything and the seven assertions above
    // would be measuring the appendix's length rather than its content.
    assert!(
        !appendix.contains("[section:8:"),
        "control: Appendix C must not show an eighth section block; §6.11-0004 pins seven"
    );
    assert!(
        !appendix.contains("dispatch_model = provider-internal"),
        "control: the appendix must not carry a §6.6-0001-forbidden dispatch value"
    );
}

/// (3b) The artifact DECLARES what remains partial, rather than leaving it to be discovered.
///
/// The 16(d) note is gone because its subject is gone. What replaces it is not silence: two real
/// limits are stated in the artifact itself -- the Dispatch sentinel (so a foreign reader never
/// exercises the five-field geometry path) and the AUDIT-checklist nature of §8-0004/-0005.
#[test]
fn test_contract_vectors_states_what_remains_partial() {
    let json = contract::emit_contract_vectors_json();
    assert!(json.contains("\"blocks_shown\": 7"), "must state it renders all 7 blocks");
    assert!(
        json.contains("\"blocks_promised_by_appendix\": 7"),
        "must still state Appendix C promises 7 blocks"
    );
    assert!(
        json.contains("geometry-agnostic sentinel") && json.contains("never exercises"),
        "must state that the Dispatch sentinel leaves the five-field geometry path unexercised"
    );
    assert!(
        json.contains("AUDIT-role checklist") || json.contains("UNBACKED.tsv records both as untested"),
        "must state that the freeze gates naming Appendix C are audit-checklist, not automated"
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
