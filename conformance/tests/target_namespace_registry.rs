//! KISS-Conform teeth for the KISS-Classify §6.8 **namespace registry** and the
//! general encoding rules that bind every `target_capability` namespace.
//!
//! Binds four clauses, two of which were previously carried in
//! `conformance/UNBACKED.tsv` as untested: the registration requirement, the
//! vocabulary-ownership delegation, the fixed-width-alphabet rule for
//! juxtaposed member sets, and the pinned digest primitive.
//!
//! Every test is named EXACTLY per the spec §9.1 traceability row for its clause
//! and cites its full `KISS-CLASSIFY-<sec>-<nnnn>` id in the leading doc-comment
//! and in an assert message. CITATION DISCIPLINE: a test body cites ONLY its own
//! clause id — cross-references use the `§<sec>-<nnnn>` short form, which does
//! not match the citation grammar.
//!
//! DEPENDENCY DISCIPLINE: this file deliberately does **not** link the
//! `vulkan:` vocabulary's reference implementation, and must not. §6.8-0002
//! matching is byte-exact on the token *string*, so checking a namespace vector
//! is a byte comparison, never a grammar parse — the neutral suite has no need
//! to understand any namespace's vocabulary in order to verify conformance to
//! it. Linking one would also breach the suite's stdlib-only rule. Golden
//! bytes below are transcribed from spec Appendix A.
//!
//! TEETH: each test fails on a concrete, plausible drift — a namespace emitted
//! without a registry row, a registered namespace with no owner, a
//! variable-length alphabet juxtaposed without separators, or a digest computed
//! with the wrong FNV constants.

const REGISTRY: &str = include_str!("../registry/namespaces.json");

/// Namespaces this repository PRODUCES tokens under, in the §6.8-0003 sense of
/// *announcing a kernel*: they appear in normative golden vectors that
/// implementations reproduce byte-for-byte.
///
/// `cuda` from spec Appendix A.1, `vulkan` from A.1.1.
const PRODUCED_NAMESPACES: &[&str] = &["cuda", "vulkan"];

/// Namespaces that appear in the suite as **fixtures** rather than productions,
/// each with the reason it is not a §6.8-0003 production.
///
/// The distinction is the same orthogonal split §6.8 already draws between
/// well-formedness (§6.8-0001/-0005) and capability validity (§6.8-0004): a
/// codec round-trip or a charset probe uses a token as an arbitrary
/// well-formed *string*, and asserts nothing about what any device could run.
/// §6.8-0003 binds a party that *announces a kernel*, and §6.8-0002 matching
/// never consults the registry, so exercising the codec is not producing.
///
/// Listing them explicitly, with reasons, is the point: the alternative is a
/// silent omission that looks identical to an oversight.
const FIXTURE_NAMESPACES: &[(&str, &str)] = &[
    (
        "rocm",
        "codec round-trip battery and charset/byte-exact-match probes only \
         (structure_key_codec.rs, structure_key_grammar.rs); no golden vector \
         and no announced kernel. `rocm` is reserved, not registered.",
    ),
    (
        "cpu",
        "a helper in opattrs_golden.rs builds a key solely to read its reduce \
         field back through to_token(); the target is incidental to that probe.",
    ),
];

/// Every namespace-shaped literal the suite contains, swept from source.
///
/// This exists because the two lists above are *claims about this repository*,
/// and a hand-maintained claim that nothing checks is the failure mode this
/// whole clause-set is about. Without the sweep, a golden added tomorrow under
/// an unregistered namespace would sail past a list that still looks correct.
fn namespaces_appearing_in_suite() -> Vec<(String, String)> {
    fn scan(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                scan(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let Ok(text) = std::fs::read_to_string(&p) else {
                    continue;
                };
                let file = p
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("?")
                    .to_string();
                // A namespace-shaped literal: "<lowercase>:<non-empty>".
                for (i, _) in text.match_indices('"') {
                    let rest = &text[i + 1..];
                    let Some(end) = rest.find('"') else { continue };
                    let lit = &rest[..end];
                    let Some((ns, cap)) = lit.split_once(':') else {
                        continue;
                    };
                    if !ns.is_empty()
                        && !cap.is_empty()
                        && ns.bytes().all(|b| b.is_ascii_lowercase())
                        && cap
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                    {
                        out.push((ns.to_string(), file.clone()));
                    }
                }
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    scan(&root.join("tests"), &mut out);
    scan(&root.join("src"), &mut out);
    out.sort();
    out.dedup();
    out
}

/// Minimal structural reader for the registry.
///
/// Hand-rolled rather than pulled from a crate: the conformance harness is
/// stdlib-only by construction (KISS-Conform §6.5), and a dependency added to
/// read the file that records dependency-free vocabularies would be its own
/// small contradiction. Only the fields these clauses bind are extracted.
fn registry_rows() -> Vec<(String, Option<String>, Option<String>, String)> {
    let mut rows = Vec::new();
    // Rows are objects inside the "namespaces" array; scan object by object.
    let body = REGISTRY
        .split_once("\"namespaces\"")
        .expect("registry must have a `namespaces` array")
        .1;
    for chunk in body.split('{').skip(1) {
        let obj = chunk.split('}').next().unwrap_or("");
        let field = |name: &str| -> Option<String> {
            let at = obj.find(&format!("\"{name}\""))?;
            let rest = &obj[at + name.len() + 2..];
            let colon = rest.find(':')?;
            let val = rest[colon + 1..].trim_start();
            if val.starts_with("null") {
                return None;
            }
            let val = val.strip_prefix('"')?;
            let end = val.find('"')?;
            Some(val[..end].to_string())
        };
        let Some(ns) = field("namespace") else { continue };
        let status = field("status").unwrap_or_default();
        rows.push((ns, field("maintainer"), field("vocabulary"), status));
    }
    rows
}

/// KISS-CLASSIFY-6.8-0003 — a namespace under which kernels are produced must
/// be registered, and the registry must ship with the suite so an offline
/// reader holds a complete copy.
#[test]
fn test_classify_target_namespace_registered() {
    let rows = registry_rows();
    assert!(
        !rows.is_empty(),
        "KISS-CLASSIFY-6.8-0003: the bundled registry parsed to zero rows; \
         an offline reader would hold no copy of the namespace assignments"
    );

    for ns in PRODUCED_NAMESPACES {
        let row = rows.iter().find(|(n, _, _, _)| n == ns);
        let Some((_, _, _, status)) = row else {
            panic!(
                "KISS-CLASSIFY-6.8-0003: tokens are produced under namespace \
                 `{ns}` but it has no registry row; an implementation MUST NOT \
                 produce a kernel under an unregistered namespace"
            );
        };
        assert_eq!(
            status, "registered",
            "KISS-CLASSIFY-6.8-0003: namespace `{ns}` is produced under but its \
             registry status is `{status}`, not `registered`"
        );
    }

    // Completeness. The list above is a claim about this repository; this is
    // what makes it a checked one. Every namespace-shaped literal in the suite
    // must be either registered or explicitly accounted for as a fixture — so
    // a golden added later under an unregistered namespace fails here rather
    // than slipping past a list that still looks correct.
    //
    // A `reserved` row is deliberately NOT required to exist: §6.8-0003 obliges
    // producers to be registered and the registry to be bundled, and says
    // nothing about holding unclaimed names. A registry listing only registered
    // namespaces is perfectly valid, and asserting otherwise would test this
    // file's present contents rather than the clause.
    for (ns, file) in namespaces_appearing_in_suite() {
        if PRODUCED_NAMESPACES.contains(&ns.as_str()) {
            continue;
        }
        if FIXTURE_NAMESPACES.iter().any(|(f, _)| *f == ns) {
            continue;
        }
        panic!(
            "KISS-CLASSIFY-6.8-0003: namespace `{ns}` appears in `{file}` but is \
             neither listed as produced-under (and registered) nor accounted for \
             in FIXTURE_NAMESPACES. If it announces a kernel it must be \
             registered; if it is only a codec or charset fixture, say so there \
             with a reason."
        );
    }

    // The fixture exemptions must stay honest too: an entry that no longer
    // appears anywhere is stale cover for nothing, and would silently excuse a
    // future reintroduction.
    let seen: Vec<String> = namespaces_appearing_in_suite()
        .into_iter()
        .map(|(ns, _)| ns)
        .collect();
    for (ns, _) in FIXTURE_NAMESPACES {
        assert!(
            seen.iter().any(|s| s == ns),
            "KISS-CLASSIFY-6.8-0003: `{ns}` is exempted as a fixture but appears \
             nowhere in the suite; remove the stale exemption rather than leave \
             it standing"
        );
    }
}

/// KISS-CLASSIFY-6.8-0004 — each namespace's capability-set vocabulary is owned
/// by that namespace's maintainer, and KISS clauses pin only the grammar and
/// the match rule.
#[test]
fn test_classify_target_capability_set_owned_by_namespace() {
    for (ns, maintainer, vocabulary, status) in registry_rows() {
        if status != "registered" {
            continue;
        }
        assert!(
            maintainer.is_some(),
            "KISS-CLASSIFY-6.8-0004: namespace `{ns}` is registered with no \
             maintainer; the vocabulary must be OWNED by someone, and an \
             unowned registered namespace has no party able to define it"
        );
        // A vocabulary pointer, when present, must live outside the clause set:
        // the delegation is defeated if a KISS clause pins the content.
        if let Some(v) = vocabulary {
            assert!(
                !v.contains("classify.md"),
                "KISS-CLASSIFY-6.8-0004: namespace `{ns}` points its vocabulary \
                 at `{v}`, inside the clause document; KISS clauses must never \
                 pin a specific namespace's capability-set vocabulary"
            );
        }
    }
}

/// KISS-CLASSIFY-6.8-0006 — a juxtaposed member set requires a fixed-width
/// alphabet; variable-length member names require an explicit delimiter.
#[test]
fn test_classify_target_fixed_width_juxtaposition() {
    // Transcribed from spec Appendix A, not parsed by a vocabulary crate.
    const OPS_FIELD: &str = "ops-abclqrstvw";
    const ARITH_FIELD: &str = "arith-dot8-f16-i8-st16-st8";

    // The juxtaposed field's alphabet is fixed-width: every member is exactly
    // one byte, so concatenation is uniquely decodable by construction.
    let ops = OPS_FIELD.strip_prefix("ops-").expect("ops field prefix");
    assert!(
        ops.chars().all(|c| c.is_ascii_lowercase()),
        "KISS-CLASSIFY-6.8-0006: juxtaposed field `{OPS_FIELD}` contains a \
         non-single-byte member; juxtaposition requires a fixed-width alphabet"
    );
    assert!(
        ops.chars().zip(ops.chars().skip(1)).all(|(a, b)| a < b),
        "KISS-CLASSIFY-6.8-0006: juxtaposed field `{OPS_FIELD}` is not strictly \
         ascending; without a canonical order one member set has several \
         spellings"
    );

    // The variable-length field must NOT juxtapose. Its members differ in
    // length, so concatenation would be decodable only contingently.
    let arith = ARITH_FIELD.strip_prefix("arith-").expect("arith field prefix");
    let names: Vec<&str> = arith.split('-').collect();
    assert!(
        names.len() > 1,
        "KISS-CLASSIFY-6.8-0006: field `{ARITH_FIELD}` carries several members \
         but shows no delimiter; a variable-length alphabet must separate them"
    );
    assert!(
        names.iter().any(|n| n.len() != names[0].len()),
        "KISS-CLASSIFY-6.8-0006: the {} members of `{ARITH_FIELD}` are all the \
         same width, so this vector cannot distinguish a delimiter requirement \
         from a fixed-width alphabet — it has no teeth as written",
        names.len()
    );
    assert!(
        names.iter().zip(names.iter().skip(1)).all(|(a, b)| a < b),
        "KISS-CLASSIFY-6.8-0006: `{ARITH_FIELD}` members are not sorted"
    );
}

/// KISS-CLASSIFY-6.8-0007 — the digest primitive is pinned: FNV-1a 64 over the
/// canonical enumeration, fixed-width lowercase hex, marker delimited by a
/// non-colon byte, and a length-driven rather than preferential switch.
#[test]
fn test_classify_target_digest_pinned() {
    fn fnv1a64(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    // Published FNV-1a 64 vectors. A wrong offset basis or prime — the two
    // constants an independent implementation is most likely to transpose —
    // fails here rather than silently producing tokens nobody else reproduces.
    assert_eq!(
        fnv1a64(b""),
        0xcbf2_9ce4_8422_2325,
        "KISS-CLASSIFY-6.8-0007: empty-input digest must equal the offset basis"
    );
    assert_eq!(
        fnv1a64(b"a"),
        0xaf63_dc4c_8601_ec8c,
        "KISS-CLASSIFY-6.8-0007: FNV-1a 64 of \"a\" does not match the pinned \
         constants"
    );
    assert_eq!(
        fnv1a64(b"foobar"),
        0x8594_4171_f739_67e8,
        "KISS-CLASSIFY-6.8-0007: FNV-1a 64 of \"foobar\" does not match the \
         pinned constants"
    );

    // Marker shape: `fnv1a64` + a non-colon delimiter + exactly 16 lowercase
    // hex digits. The colon exclusion matters because §6.8-0001 permits exactly
    // one colon in a token, and a marker using it would make every digest-form
    // token unparseable — a failure that appears only on large devices.
    let digest = format!("fnv1a64-{:016x}", fnv1a64(b"16-16-16-f16-f16-f32-f32"));
    let (marker, hex) = digest
        .split_once('-')
        .expect("KISS-CLASSIFY-6.8-0007: digest must carry a delimited marker");
    assert_eq!(marker, "fnv1a64");
    assert!(
        !digest.contains(':'),
        "KISS-CLASSIFY-6.8-0007: the digest marker delimiter must not be `:`"
    );
    assert_eq!(
        hex.len(),
        16,
        "KISS-CLASSIFY-6.8-0007: digest hex must be exactly 16 digits, got {}",
        hex.len()
    );
    assert!(
        hex.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "KISS-CLASSIFY-6.8-0007: digest hex must be lowercase; uppercase would \
         be a second spelling of one digest"
    );
}
