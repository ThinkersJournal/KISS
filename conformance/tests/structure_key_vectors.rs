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
use std::collections::HashSet;
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

/// (5) The target axis is machine-readable AND its per-vector tag is cross-checked
/// against the SERIALIZED token, never against its own derivation — so it cannot pass
/// vacuously. There is no typed namespace enum on `StructureKey.target`, so the emitter
/// derives `target_namespace` from the target; this test re-derives the namespace by its
/// OWN independent split of the token's field-3 and asserts they agree, which catches an
/// emitter that split differently (proven by mutation in the PR #161 review). It also
/// pins `target` == the token's serialized field-3, which catches a `to_token` that
/// mangled the target.
#[test]
fn target_axis_is_machine_readable_and_cross_checked() {
    let doc = kiss_conformance::json::parse(&emit_reference_vectors_json())
        .expect("artifact must be valid JSON");

    let namespaces = str_array(&doc, "target_namespaces");
    assert!(namespaces.contains(&"cuda".to_string()), "cuda must be a target namespace");
    assert!(namespaces.contains(&"vulkan".to_string()), "vulkan must be a target namespace");

    let positives = doc.get("positive_vectors").and_then(|j| j.as_arr()).expect("positive_vectors array");
    let mut saw_vulkan = false;
    for v in positives {
        let target = v.get("target").and_then(|j| j.as_str()).expect("each positive has a target");
        let ns = v.get("target_namespace").and_then(|j| j.as_str()).expect("each positive has a target_namespace");
        let token = v.get("token").and_then(|j| j.as_str()).expect("each positive has a token");

        // cross-check 1 (direct field vs serialized token): `target` must equal the
        // token's field-3 — a `to_token` that mangled the target breaks this.
        let field3 = token.split('|').nth(3).expect("token has a field 3");
        assert_eq!(target, field3, "target field must equal the token's serialized field-3 ({token})");

        // cross-check 2 (INDEPENDENT re-derivation): the namespace, split HERE by the
        // test from the serialized token, must equal the emitted `target_namespace`. The
        // test's split does not call the emitter's helper, so an emitter that split on the
        // wrong byte fails here rather than agreeing with itself.
        let expected_ns = field3.split(':').next().expect("field-3 namespace");
        assert_eq!(ns, expected_ns, "target_namespace must be the namespace of the serialized token target ({token})");

        if ns == "vulkan" {
            saw_vulkan = true;
        }
    }
    assert!(saw_vulkan, "the vulkan-target vector must be tagged target_namespace=vulkan");
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

/// (6) INJECTIVITY of the decline wire. `decline_wire`'s exhaustiveness (E0004) proves
/// every `KeyDecline` variant maps to SOMETHING; nothing proves two variants don't map to
/// the SAME string. That failure is silent — a green build, a complete leg report, two
/// declines collapsed into one, and a run that then agrees with the reference for the
/// WRONG reason (worse than disagreeing, which gets investigated). Assert the wire kinds
/// are pairwise distinct against a PINNED count (asserted counts are pinned, not computed:
/// a new variant is an E0004 in decline_wire, and this count is the deliberate second
/// gate). Discrimination proven by seeding a duplicate name in the PR #161 review.
#[test]
fn decline_wire_kinds_are_injective() {
    let kinds = all_decline_wire_kinds();
    // pinned — bump deliberately when a KeyDecline variant is added.
    assert_eq!(kinds.len(), 21, "pinned decline-variant count; bump when KeyDecline grows");
    let distinct: HashSet<&&str> = kinds.iter().collect();
    assert_eq!(
        distinct.len(),
        kinds.len(),
        "decline wire kinds MUST be pairwise distinct (injectivity) — two variants share a string: {kinds:?}"
    );
}

/// (7) BOUNDARY of the byte-match's dtype coverage — makes `coverage_note` EXECUTABLE so its
/// claim cannot quietly stop being true. The positive vectors place only a few of the 22
/// usable dtypes in the dtype position, so a byte-match against this set cannot see a
/// dtype-spelling divergence on a token that never appears (Unpopped's c128→c127 survived
/// their leg green; the dtype MANIFEST is the vocabulary instrument and catches it by name).
/// Counts are PINNED so a future reader sees "3 → 7", not a bare boolean flip.
///
/// IF THIS FAILS: the positive vectors now exercise MORE dtype vocabulary than `coverage_note`
/// claims. THAT IS AN IMPROVEMENT, not a regression. The fix is to update `coverage_note` in
/// `src/reference_vectors.rs` AND the pinned counts in THIS test to the new numbers. Do NOT
/// delete vectors to make this green — that reverts the very improvement the guard detects.
#[test]
fn coverage_note_boundary_dtype_position_is_a_strict_subset() {
    let doc = kiss_conformance::json::parse(&emit_reference_vectors_json()).expect("valid JSON");
    let positives = doc.get("positive_vectors").and_then(|j| j.as_arr()).expect("positive_vectors array");
    let usable = str_array(&doc, "dtype_usable_set");
    let usable_set: HashSet<&str> = usable.iter().map(|s| s.as_str()).collect();

    // distinct dtypes in the DTYPE POSITION (token field 2). `unwrap_or_else` names the
    // token+field so a format drift reports the format, not `Option::unwrap()`.
    let in_dtype_pos: HashSet<&str> = positives
        .iter()
        .map(|v| {
            let token = v
                .get("token")
                .and_then(|j| j.as_str())
                .unwrap_or_else(|| panic!("positive vector has no string `token`"));
            token
                .split('|')
                .nth(2)
                .unwrap_or_else(|| panic!("token has no field-2 (dtype position): {token}"))
        })
        .collect();
    // distinct USABLE dtypes appearing ANYWHERE (dtype pos + contraction/acc-mp subfields).
    let anywhere: HashSet<&str> = positives
        .iter()
        .flat_map(|v| {
            let token = v
                .get("token")
                .and_then(|j| j.as_str())
                .unwrap_or_else(|| panic!("positive vector has no string `token`"));
            token
                .split(|c| c == '|' || c == '/' || c == ';')
                .filter(|f| usable_set.contains(f))
                .collect::<Vec<_>>()
        })
        .collect();

    // direction-NEUTRAL pinned guard (see `check_pinned_coverage`): it states what it OBSERVED
    // before prescribing, so an equality miss on a DECREASE isn't reported as an improvement.
    check_pinned_coverage("dtypes in the dtype position", in_dtype_pos.len(), 3, format!("{in_dtype_pos:?}"));
    check_pinned_coverage("usable dtypes appearing anywhere", anywhere.len(), 5, format!("{anywhere:?}"));
    // the boundary the note states: strictly fewer than the 22 usable tokens are exercised.
    assert!(
        in_dtype_pos.len() < usable.len(),
        "dtype-position coverage must be a strict subset of the {} usable tokens",
        usable.len()
    );
}

/// Direction-neutral pinned-count check for the coverage boundary. Reports observed vs pinned,
/// says WHICH WAY it moved, and prescribes per direction: an INCREASE is a coverage improvement
/// (update `coverage_note` + the pin); a DECREASE is a regression (a vector was lost or changed —
/// RESTORE it, do NOT lower the pin). A guard that prescribes a response must first state what it
/// observed, or an equality miss on a decrease is reported as the improvement it is not.
fn check_pinned_coverage(what: &str, observed: usize, pinned: usize, detail: String) {
    if observed == pinned {
        return;
    }
    let (dir, prescribe) = if observed > pinned {
        (
            "INCREASED — coverage improved",
            "update `coverage_note` in src/reference_vectors.rs AND this pinned count to the new number",
        )
    } else {
        (
            "DECREASED — coverage regressed",
            "a vector was lost or changed; RESTORE it — do NOT lower the pin to match",
        )
    };
    panic!("{what}: observed {observed} vs pinned {pinned} — {dir}. {prescribe}. Never silently re-baseline. Saw: {detail}");
}

/// (8) `coverage_note` claims rule (d) — rejecting the all-default redundant `(acc+mp)` —
/// is covered by a DECLINE vector, because the POSITIVE byte-match is structurally blind to
/// it (a decline-side rule; Fuel found this by seeded sabotage: removing the enforcement left
/// the positive byte-match green because every published `(acc+mp)` positive genuinely
/// deviates). Make the "covered here" half of the note executable: if the redundant decline
/// vector were removed, the note would be a lie and this test names why.
#[test]
fn rule_d_is_covered_by_a_redundant_decline_vector() {
    let doc = kiss_conformance::json::parse(&emit_reference_vectors_json()).expect("valid JSON");
    let declines = doc.get("decline_vectors").and_then(|j| j.as_arr()).expect("decline_vectors array");
    let has_rule_d = declines
        .iter()
        .any(|d| d.get("decline").and_then(|j| j.as_str()) == Some("RedundantAccMpField"));
    assert!(
        has_rule_d,
        "coverage_note claims rule (d) is covered by a RedundantAccMpField decline vector \
         (redundant_acc_mp_all_default) — that vector is gone, so the note is now false. The positive \
         byte-match is blind to rule (d); restore the decline vector, do NOT weaken the note."
    );
}

/// (9) Every repo path cited in `coverage_note` must EXIST. The note's job is to tell a reader
/// WHICH instrument closes the gap; a citation to a path that does not resolve fails in exactly
/// the "gap unclosed" direction — the reader searches, finds nothing, concludes coverage is
/// missing. This catches the rotted/wrong-citation class that put `kiss_dtype_manifest` (a
/// nonexistent tool) into the note before this guard existed.
#[test]
fn coverage_note_cited_paths_exist() {
    let doc = kiss_conformance::json::parse(&emit_reference_vectors_json()).expect("valid JSON");
    let note = doc.get("coverage_note").and_then(|j| j.as_str()).expect("coverage_note is a string");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crate dir has a parent (repo root)");
    // repo-relative path-like tokens: contain a '/' and end in a source/data extension.
    let cited: Vec<&str> = note
        .split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | ',' | ';'))
        .filter(|t| t.contains('/') && (t.ends_with(".py") || t.ends_with(".json") || t.ends_with(".rs")))
        .collect();
    assert!(!cited.is_empty(), "coverage_note should cite at least one instrument path so the gap is attributable");
    for p in cited {
        assert!(
            repo_root.join(p).exists(),
            "coverage_note cites `{p}`, which does not exist in the repo — a dead citation reads as \
             'coverage missing'. Point at the real path (fix in src/reference_vectors.rs)."
        );
    }
}

// ---------------------------------------------------------------------------
// #200 — vocabulary provenance for maintainer-owned namespaces.
// ---------------------------------------------------------------------------

/// The `**Vocabulary version:** N` header of a namespace document.
///
/// A missing document, or a header this cannot parse, is a HARD FAILURE — never a
/// skip. A checker that passes when it could not find the thing it checks is
/// strictly worse than no checker: it now reports on a property it did not
/// examine, which is the exact shape #200 exists to close.
fn doc_vocab_version(ns: &str) -> u32 {
    let path = format!("{}/../spec/namespaces/{}.md", env!("CARGO_MANIFEST_DIR"), ns);
    let md = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("#200: no vocabulary document for namespace `{ns}:` at {path} ({e}). \
                The artifact embeds a token in this namespace, so its grammar MUST be \
                locatable; a validator that cannot find it must fail, not pass.")
    });
    let marker = "**Vocabulary version:**";
    let i = md.find(marker).unwrap_or_else(|| {
        panic!("#200: {path} carries no `{marker} N` header — the version this \
                artifact's tokens are generated against is unstateable.")
    });
    md[i + marker.len()..]
        .split_whitespace()
        .next()
        .and_then(|t| t.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok())
        .unwrap_or_else(|| panic!("#200: {path}'s `{marker}` header is unparseable"))
}

/// The capability-set field arity the document's grammar line declares, counted
/// from the fenced `\u{3c}namespace\u{3e}:` template. Same hard-failure discipline.
fn doc_field_arity(ns: &str) -> usize {
    let path = format!("{}/../spec/namespaces/{}.md", env!("CARGO_MANIFEST_DIR"), ns);
    let md = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("#200: no vocabulary document for `{ns}:` at {path} ({e})"));
    let line = md
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with(&format!("{ns}:")))
        .unwrap_or_else(|| panic!("#200: {path} declares no `{ns}:` grammar line to count fields from"));
    line[ns.len() + 1..].split('.').count()
}

/// #200 — every namespace the artifact embeds states a vocabulary version and a
/// field arity, and BOTH agree with the maintainer's document.
///
/// This is the tie that makes the pinned table falsifiable. Without it the table is
/// a second copy of a fact owned elsewhere, and a vocabulary bump moves the document
/// while the artifact keeps publishing the old spelling — which is exactly what
/// happened: `vulkan.md` reached v4 (five fields) while this artifact published a
/// four-field token, and three consumers byte-matched it because every one of them
/// copied the token from here rather than deriving it.
#[test]
fn reference_vectors_match_the_namespace_documents() {
    use kiss_conformance::reference_vectors::*;
    assert!(!NAMESPACE_VOCAB_VERSIONS.is_empty(), "#200: the version table is empty");
    for (ns, version, arity) in NAMESPACE_VOCAB_VERSIONS {
        assert_eq!(
            *version, doc_vocab_version(ns),
            "#200: `{ns}` pinned at vocabulary version {version}, but spec/namespaces/{ns}.md \
             declares a different one. The artifact's tokens were generated against the pinned \
             version; the document has moved. Regenerate, then update the pin."
        );
        assert_eq!(
            *arity, doc_field_arity(ns),
            "#200: `{ns}` pinned at {arity} capability-set field(s), but its grammar line \
             declares a different count."
        );
    }
}

/// #200 req 1/2 — the artifact states a version for EVERY namespace it embeds, and
/// states it unconditionally.
///
/// Unconditional is the point: a field that appears only when something moves is
/// indistinguishable from a field nobody remembered to write.
#[test]
fn reference_vectors_state_a_vocabulary_version_for_every_namespace() {
    use kiss_conformance::reference_vectors::*;
    let json = emit_reference_vectors_json();
    assert!(json.contains("\"namespace_vocabulary_versions\""), "#200: the artifact states none");
    for v in positive_vectors() {
        let (ns, _) = v.key.target.split_once(':').unwrap_or_else(|| {
            panic!("#200: vector `{}` has target `{}` with no `<namespace>:` prefix (§6.8-0001)", v.name, v.key.target)
        });
        assert!(
            json.contains(&format!("\"{ns}\": {}", namespace_vocab_version(ns))),
            "#200: vector `{}` embeds namespace `{ns}:` but the artifact states no version for it",
            v.name
        );
    }
}
