//! Gate for the generated reference-vectors artifact
//! (`conformance/corpus/structure_key_vectors.json`, Classify §6.7, sk4).
//!
//! Four checks, each closing a distinct failure mode:
//!   1. FRESHNESS — the committed file equals `emit_reference_vectors_json()`
//!      byte-for-byte. Byte equality, not parsed: a consumer byte-hashing the file
//!      sees exactly what this test sees, so the test cannot pass on a projection of
//!      the artifact its consumers will trip on.
//!   2. SELF-CONSISTENCY — every positive vector's constructed key serializes to its
//!      pinned token AND round-trips through `from_token` byte-identically (§6.7-0008).
//!   3. DECLINE INTEGRITY — every decline vector's malformed input yields the pinned
//!      typed `KeyDecline`.
//!   4. COVERAGE — the artifact is a superset of every `sk4|` token literal in the
//!      frozen `structure_key_golden.rs`, bound by scanning that source. This is what
//!      lets the artifact be additive: a golden cell can never be silently dropped,
//!      and no second transcription is introduced to drift against the first.

use kiss_conformance::reference_vectors::*;
use kiss_conformance::structure_key::{from_token, DTYPES, RESERVED_DTYPES};
use std::path::Path;

fn committed_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/structure_key_vectors.json")
}

/// (1) The committed artifact is byte-identical to a fresh generation. If this fails,
/// regenerate with `cargo run --bin emit_structure_key_vectors > conformance/corpus/structure_key_vectors.json`.
#[test]
fn structure_key_vectors_artifact_is_fresh() {
    let committed = std::fs::read(committed_path())
        .expect("conformance/corpus/structure_key_vectors.json is missing — generate it with the emit_structure_key_vectors bin");
    let fresh = emit_reference_vectors_json();
    // Byte equality (the strong check): CRLF or any drift fails here, exactly the
    // class of difference a Windows consumer byte-hashing the file would see.
    assert_eq!(
        committed,
        fresh.as_bytes(),
        "structure_key_vectors.json is STALE or not LF-normalized — regenerate with \
         `cargo run --bin emit_structure_key_vectors > conformance/corpus/structure_key_vectors.json` \
         (and confirm .gitattributes forces LF on it)"
    );
}

/// (2) Every positive vector's key serializes to its pinned token (the Appendix-A
/// spec-tie) and round-trips byte-identically — the same both-directions check the
/// golden test makes, so the artifact's tokens carry the same guarantee.
#[test]
fn positive_vectors_serialize_and_round_trip() {
    for v in positive_vectors() {
        assert_eq!(v.key.to_token(), v.token, "[{}] to_token != pinned token", v.clause);
        let parsed = from_token(v.token)
            .unwrap_or_else(|e| panic!("[{}] from_token declined a positive vector `{}`: {e:?}", v.clause, v.name));
        assert_eq!(parsed, v.key, "[{}] from_token produced a different key for `{}`", v.clause, v.name);
        assert_eq!(parsed.to_token(), v.token, "[{}] round-trip is not byte-identical for `{}`", v.clause, v.name);
    }
}

/// (3) Every decline vector's malformed input yields the pinned typed decline.
#[test]
fn decline_vectors_answer_the_pinned_decline() {
    for d in decline_vectors() {
        assert_eq!(
            from_token(&d.token),
            Err(d.expected.clone()),
            "[{}] decline vector `{}` did not answer the pinned decline (token: {})",
            d.clause, d.name, d.token
        );
    }
}

/// Extract a JSON string-array field from the parsed artifact. Panics if the key is
/// ABSENT — so a deleted `reserved_dtypes` array (the exact vacuous-guard failure this
/// file defends against) makes the dual-axis test go red, where a whole-document
/// substring search would still find the token in some other array.
fn str_array(doc: &kiss_conformance::json::Json, key: &str) -> Vec<String> {
    doc.get(key)
        .unwrap_or_else(|| panic!("artifact is missing the `{key}` array"))
        .as_arr()
        .unwrap_or_else(|| panic!("artifact `{key}` is not an array"))
        .iter()
        .map(|j| j.as_str().expect("array element must be a string").to_string())
        .collect()
}

/// (4) The dual axis is machine-readable, correct, AND its guard DISCRIMINATES the two
/// axes — the defect this whole file defends against, previously present inside its own
/// test. Every assertion is against the PARSED per-key arrays, not a whole-document
/// substring: a reserved token being present "somewhere" (it is always in the
/// recognition set) can no longer make a reserved/usable assertion pass vacuously.
/// Deleting `reserved_dtypes` panics in `str_array`; emptying or mis-filling it fails
/// the membership/length assertions — verified by scratch deletion in the PR #161 review.
#[test]
fn dual_axis_is_present_and_discriminates() {
    assert_eq!(DTYPES.len(), 24, "recognition set must be 24 tokens");
    assert_eq!(RESERVED_DTYPES.len(), 2, "two reserved tokens");
    assert_eq!(DTYPES.iter().filter(|d| !RESERVED_DTYPES.contains(d)).count(), 22, "usable const = 22");

    let doc = kiss_conformance::json::parse(&emit_reference_vectors_json())
        .expect("the emitted artifact must be valid JSON");

    let recognition = str_array(&doc, "dtype_recognition_set");
    let usable = str_array(&doc, "dtype_usable_set");
    let reserved = str_array(&doc, "reserved_dtypes");

    // counts equal the ACTUAL array lengths — a declared count that lies about its
    // array is caught here, not assumed equal to 24/22.
    assert_eq!(doc.get("recognition_count").and_then(|j| j.as_u64()), Some(recognition.len() as u64), "recognition_count must equal its array length");
    assert_eq!(doc.get("usable_count").and_then(|j| j.as_u64()), Some(usable.len() as u64), "usable_count must equal its array length");
    assert_eq!(recognition.len(), 24);
    assert_eq!(usable.len(), 22);
    assert_eq!(reserved.len(), 2);
    assert_eq!(doc.get("structure_key_schema_version").and_then(|j| j.as_u64()), Some(4));
    assert_eq!(doc.get("token_prefix").and_then(|j| j.as_str()), Some("sk4"));
    assert_eq!(doc.get("source_commit").and_then(|j| j.as_str()), Some(SOURCE_COMMIT));

    // the discriminating per-axis assertions: each reserved token is in the reserved
    // ARRAY, present in recognition, and ABSENT from usable — the three a whole-document
    // substring search could not tell apart.
    for r in RESERVED_DTYPES {
        let r = r.to_string();
        assert!(reserved.contains(&r), "{r} must be in the reserved_dtypes array");
        assert!(recognition.contains(&r), "{r} must be recognized (recognition surface)");
        assert!(!usable.contains(&r), "{r} must NOT be usable (byte-match surface)");
    }
    // a KNOWN-usable token must NOT be in the reserved array (so `reserved` is not just
    // "everything") — the counterexample a whole-document search would wrongly pass.
    assert!(!reserved.contains(&"f32".to_string()), "a usable dtype must not appear in reserved_dtypes");
    // usable is exactly recognition minus reserved.
    for d in &recognition {
        assert_eq!(usable.contains(d), !reserved.contains(d), "usable = recognition minus reserved (failed for {d})");
    }
}

/// Extract every double-quoted string literal in `src` that looks like a full
/// `structure_key` token: begins `sk4|` and carries at least the 9 mandatory fields.
/// Fragments used as `replacen` needles (`"sk4|"`, `"sk4|bin|f32"`) have fewer fields
/// and are excluded, as are the other-version literals (`"sk9"`, `"sk04"`) which do
/// not begin `sk4|`. No regex crate — the harness is stdlib-only (§6.5).
fn scan_sk4_token_literals(src: &str) -> Vec<String> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            // capture until the next unescaped double quote (token literals contain
            // no backslash escapes, but handle `\"` defensively).
            let mut j = i + 1;
            let mut lit = String::new();
            while j < bytes.len() && bytes[j] != b'"' {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    lit.push(bytes[j + 1] as char);
                    j += 2;
                } else {
                    lit.push(bytes[j] as char);
                    j += 1;
                }
            }
            if lit.starts_with("sk4|") && lit.split('|').count() >= 9 {
                out.push(lit);
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// (4-coverage) The artifact is a superset of every full `sk4|` token literal the
/// frozen golden test pins — bound by scanning the golden source, additively, without
/// refactoring it. A golden cell dropped from the artifact fails here, naming the token.
#[test]
fn artifact_covers_every_golden_token_literal() {
    let golden_src = include_str!("structure_key_golden.rs");
    let golden_tokens = scan_sk4_token_literals(golden_src);
    assert!(
        golden_tokens.len() >= 20,
        "scan found only {} golden token literals — the scanner likely broke, not the coverage",
        golden_tokens.len()
    );

    let published: std::collections::HashSet<String> = positive_vectors()
        .iter()
        .map(|v| v.key.to_token())
        .chain(decline_vectors().into_iter().map(|d| d.token))
        .collect();

    let missing: Vec<&String> = golden_tokens.iter().filter(|t| !published.contains(*t)).collect();
    assert!(
        missing.is_empty(),
        "the reference artifact is missing golden token literal(s) — publish them or the byte-match \
         set under-covers the frozen golden:\n{}",
        missing.iter().map(|t| format!("  {t}")).collect::<Vec<_>>().join("\n")
    );
}
