//! KISS-Grammar golden REGION vector artifact (`conformance/corpus/grammar_vectors.json`) — the
//! machine-readable golden region set that now EXISTS in the corpus, giving `KISS-GRAMMAR-8-0004`
//! (≥2 dissimilar implementations interoperate on the golden region vectors) and `-8-0005` (a
//! foreign reader written outside the reference language consumes the region wire form) something
//! to reproduce FROM. #466, on the #349/#365 pattern.
//!
//! ⚠️ THE GAP THIS CLOSES WAS A BINDING, NOT A COVERAGE, GAP. Before this the goldens lived in two
//! places that were not tied together: prose in `spec/grammar.md` Appendix A.1 (headed
//! **informative**), and `const G1_GOLDEN` / `G4_GOLDEN` in `grammar_golden.rs`. Measured at the
//! time: **zero** `read_spec`/`include_str!`/`grammar.md` references across the three golden test
//! files, against 9 in `contract_framing.rs` as a control. The two agreed — checked, they had not
//! drifted — but nothing made them agree. Obligation (2) below is that missing tie.
//!
//! Three obligations, mirroring `contract_vectors.rs`:
//!   1. FRESHNESS (#161): the committed artifact equals `emit_grammar_vectors_json()` byte-for-byte,
//!      so a stale artifact fails CI.
//!   2. APPENDIX AGREEMENT: the codec's bytes equal the hex transcribed in Appendix A.1. A
//!      spec↔codec divergence here is a DECISION (which side moves is per-instance, escalated),
//!      never a silent reconcile — the appendix is hand-maintained prose, as likely wrong as the
//!      codec.
//!   3. PARTIAL, STATED not discovered (16(d)): the artifact renders 2 of the 5 vectors Appendix
//!      A.1 declares, and NAMES the other three rather than omitting them quietly.

use kiss_conformance::grammar;

fn committed_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/grammar_vectors.json")
}

fn read_spec() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("spec/grammar.md");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// Concatenated lowercase hex of the first fenced block after `marker`, taking each line's
/// LEADING hex bytes and stopping at the first non-hex token (the annotation column).
///
/// ⚠️ THIS RETURNS AN EMPTY STRING IF THE MARKER OR FENCE MOVES, AND AN EMPTY STRING WOULD
/// COMPARE UNEQUAL TO THE CODEC BYTES AND FAIL — which is the direction a broken extractor must
/// fail in. The caller additionally asserts a non-trivial length, so "extracted nothing" can never
/// read as "agrees".
fn appendix_hex(md: &str, marker: &str) -> String {
    let start = match md.find(marker) {
        Some(i) => i,
        None => return String::new(),
    };
    let rest = &md[start..];
    let open = match rest.find("```") {
        Some(i) => i,
        None => return String::new(),
    };
    let body = &rest[open + 3..];
    let close = match body.find("```") {
        Some(i) => i,
        None => return String::new(),
    };
    let mut out = String::new();
    for line in body[..close].lines() {
        for tok in line.split_whitespace() {
            let is_hex_byte =
                tok.len() == 2 && tok.chars().all(|c| c.is_ascii_hexdigit());
            if is_hex_byte {
                out.push_str(&tok.to_ascii_lowercase());
            } else {
                break; // the annotation column starts here
            }
        }
    }
    out
}

fn artifact_hex(json_text: &str, id: &str) -> String {
    // ⚠️ PARSED, NOT SLICED. A hand-rolled string search is sensitive to whitespace and field
    // order, so a reformatting of the generator's output would redden these tests while the DATA
    // was still correct — a false red that trains people to edit the test. The crate already has
    // a parser; using it makes the assertion about the data rather than about the layout.
    let doc = kiss_conformance::json::parse(json_text).expect("the artifact must be valid JSON");
    let vectors = doc
        .get("vectors")
        .and_then(|v| v.as_arr())
        .expect("the artifact carries a `vectors` array");
    for v in vectors {
        if v.get("id").and_then(|j| j.as_str()) == Some(id) {
            return v
                .get("wire_hex")
                .and_then(|j| j.as_str())
                .expect("each vector carries `wire_hex`")
                .to_string();
        }
    }
    panic!("vector {id} absent from the artifact");
}

/// (1) FRESHNESS — the committed artifact is byte-identical to a fresh generation. If this fails,
/// regenerate: `cargo run --bin emit_grammar_vectors > conformance/corpus/grammar_vectors.json`.
#[test]
fn test_grammar_vectors_artifact_is_fresh() {
    let committed = std::fs::read_to_string(committed_path())
        .expect("conformance/corpus/grammar_vectors.json must exist");
    let fresh = grammar::emit_grammar_vectors_json();
    assert_eq!(
        committed, fresh,
        "grammar_vectors.json is STALE — regenerate with `cargo run --bin emit_grammar_vectors > \
         conformance/corpus/grammar_vectors.json`"
    );
}

/// (2) ⚠️ APPENDIX AGREEMENT — THE BINDING THAT DID NOT EXIST. The codec's region bytes equal the
/// hex Appendix A.1 transcribes. Before this test the spec's bytes and the codec's bytes could
/// diverge with nothing to notice.
///
/// A mismatch is a DECISION, escalated per-instance, never reconciled silently: the appendix is
/// hand-maintained and is as likely to be the wrong side as the codec.
#[test]
fn test_grammar_vectors_match_appendix_a1() {
    let md = read_spec();
    let json = grammar::emit_grammar_vectors_json();

    for (marker, id, min_bytes) in [
        ("*Vector G1", "G1", 60usize),
        ("*Vector G4", "G4", 40usize),
    ] {
        let spec_hex = appendix_hex(&md, marker);
        // ⚠️ The extractor must not be able to "agree" by finding nothing.
        assert!(
            spec_hex.len() >= min_bytes * 2,
            "Appendix A.1 extraction for {id} produced {} hex chars — the marker or fence moved, \
             and an empty extraction must never read as agreement",
            spec_hex.len()
        );
        assert_eq!(
            spec_hex,
            artifact_hex(&json, id),
            "spec/grammar.md Appendix A.1 vector {id} and the region codec DISAGREE. This is a \
             decision, not a reconcile: the appendix is hand-maintained prose and may be the wrong \
             side. Escalate rather than editing either to match."
        );
    }
}

/// (3) PARTIAL, STATED not discovered — the artifact declares all five Appendix A.1 vectors, names
/// the two it renders and the three it does not, and its coverage note says what a passing
/// byte-match does and does not establish.
///
/// ⚠️ WITHOUT THIS, A PARTIAL ARTIFACT IS INDISTINGUISHABLE FROM A COMPLETE ONE. That is the whole
/// defect #466 reports one level up: a corpus that does not state its own population lets a reader
/// conclude the gate ranges over more than it does.
#[test]
fn test_grammar_vectors_state_their_own_population() {
    let json = grammar::emit_grammar_vectors_json();
    for declared in ["G1", "G2", "G3", "G4", "G5"] {
        assert!(
            json.contains(&format!("\"{declared}\"")),
            "Appendix A.1 declares {declared}; the artifact must name it as rendered or unrendered"
        );
    }
    assert!(json.contains("\"rendered_vectors\": [\"G1\", \"G4\"]"), "rendered set must be stated");
    assert!(
        json.contains("\"unrendered_vectors\": [\"G2\", \"G3\", \"G5\"]"),
        "the UNRENDERED vectors must be named — a quiet omission is the defect this closes"
    );
    // The note must say what the byte-match does NOT establish, not merely what it covers.
    for phrase in [
        "WHAT THIS DOES NOT CARRY",
        "op vocabulary is NOT exercised",
        "op-CATEGORY",
    ] {
        assert!(
            json.contains(phrase),
            "the coverage note must state its blind spots; missing: {phrase:?}"
        );
    }
}

/// The library builders are the SINGLE definition. Before consolidation `g1_region()` was defined
/// three times across `grammar_golden.rs`, `grammar_canonical.rs` and `grammar_tag.rs` — identical
/// in substance at the time (the only textual difference was a trailing comma), so this pins a
/// hazard closed rather than repairing a drift.
#[test]
fn test_grammar_golden_regions_are_singly_defined() {
    let regions = grammar::golden_regions();
    assert_eq!(regions.len(), 2, "G1 and G4 are the buildable Appendix A.1 vectors");
    assert_eq!(regions[0].0, "G1");
    assert_eq!(regions[1].0, "G4");
    // The canonical builder and the artifact agree by construction — encode the region here and
    // compare to what the generator emitted, so a divergence between the two paths reddens.
    let json = grammar::emit_grammar_vectors_json();
    for (id, _desc, region) in regions {
        let bytes = grammar::encode(&region).expect("a golden region must encode");
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, artifact_hex(&json, id), "builder and artifact disagree for {id}");
    }
}
