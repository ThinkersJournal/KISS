//! KISS-Announce golden vector artifact (`conformance/corpus/announce_vectors.json`) — the
//! machine-readable golden handshake frames a foreign (non-Rust) reader byte-diffs to check
//! endianness, field width and structure padding (umbrella §5.3 condition 2, which a Markdown
//! appendix cannot be executed against). The wire handshake is where a prose appendix fails
//! hardest. Companion to #365 (`contract_vectors`) and #469 (grammar); the fifth `.json` golden.
//!
//! Four obligations (the #365 / #469 pattern):
//!   1. FRESHNESS (#161): the committed artifact equals `emit_announce_vectors_json()` byte-for-byte,
//!      so a stale artifact fails CI (mirrors the contract-vectors / manifest freshness gates).
//!   2. APPENDIX-AGREEMENT: the artifact renders the LIBRARY reference builders (`announce::
//!      reference_*`), and `announce_golden.rs` asserts THOSE builders against the §2.5 transcribed
//!      appendix hex — so artifact == builder == §2.5 (the loop is closed across the two tests). A
//!      builder↔appendix mismatch is a per-instance DECISION in announce_golden, never a silent
//!      reconcile — the appendix is hand-maintained prose.
//!   3. POPULATION STATED (16(d)): the artifact names the 5 frames it covers AND what it does NOT,
//!      so a reader cannot mistake it for the complete handshake surface.
//!   4. SINGLE DEFINITION: the frame builders live ONCE in the library; the artifact and both test
//!      files (announce_golden, announce_frames) render/consume that one source — the reference
//!      envelope was previously copied in three places (the #469 g1_region shadowing hazard).

use kiss_conformance::announce;

fn committed() -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/announce_vectors.json"))
        .expect("conformance/corpus/announce_vectors.json must exist")
}

/// (1) FRESHNESS — the committed artifact is byte-identical to a fresh generation.
#[test]
fn test_announce_vectors_artifact_is_fresh() {
    assert_eq!(
        committed(),
        announce::emit_announce_vectors_json(),
        "announce_vectors.json is STALE — regenerate: \
         `cargo run --bin emit_announce_vectors > conformance/corpus/announce_vectors.json`"
    );
}

/// (2)+(4) The artifact renders the LIBRARY reference builders — each frame's exact `encode()`
/// bytes, under its clause id. This is the SINGLE-DEFINITION half (the artifact consumes the one
/// library source, not a copy) and the codec-side of APPENDIX-AGREEMENT; announce_golden.rs closes
/// the loop by asserting those same builders against the §2.5 transcribed appendix hex.
#[test]
fn test_announce_vectors_render_the_library_builders() {
    let json = announce::emit_announce_vectors_json();
    let hexs = |b: &[u8]| b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ");
    let frames: [(&str, Vec<u8>); 5] = [
        ("KISS-ANNOUNCE-6.1-0002", announce::reference_envelope().encode()),
        ("KISS-ANNOUNCE-6.3-0005", announce::reference_availability_list().encode()),
        ("KISS-ANNOUNCE-6.4-0001", announce::reference_cyrq().encode()),
        ("KISS-ANNOUNCE-6.4-0004", announce::reference_crsp().encode()),
        ("KISS-ANNOUNCE-6.4-0007", announce::reference_cdec().encode()),
    ];
    for (clause, bytes) in &frames {
        assert!(json.contains(clause), "artifact must carry frame {clause}");
        assert!(
            json.contains(&hexs(bytes)),
            "artifact's {clause} frame must be the library builder's EXACT bytes (single source, no copy)"
        );
    }
    // Anchor to the §2.5 example: SEAM magic (53 45 41 4D) little-endian + envelope_version 01.
    assert!(
        announce::reference_envelope().encode().starts_with(&[0x53, 0x45, 0x41, 0x4D, 0x01]),
        "reference_envelope must be the §2.5 SEAM envelope, version 1"
    );
}

/// (3) POPULATION STATED — the 16(d) principle: the artifact names what it covers AND what it does
/// not, so its partiality is stated, not discovered. A staged corpus with no stated boundary is
/// indistinguishable from a complete one.
#[test]
fn test_announce_vectors_state_their_population() {
    let json = announce::emit_announce_vectors_json();
    assert!(json.contains("\"population\""), "must carry a population field");
    assert!(
        json.contains("FIVE") && json.contains("NOT rendered here"),
        "population must state the 5 covered frames AND the boundary (decline vectors / other frame types)"
    );
    assert!(
        json.contains("DIFFABLE, not COVERED"),
        "the scope note must state a byte-diff does NOT cover the untested KISS-Announce clauses"
    );
}
