//! Second KISS-Contract conformance module (sibling to `contract_golden.rs`),
//! continuing the #91 coverage burndown. Two sections:
//!
//!   (A) TRANSPORT-READER DISCIPLINE — behavioral tests that drive the real
//!       `conformance/src/contract.rs` codec (`read_document`, `crc32_ieee`,
//!       `Document::encode`). Every negative document is rebuilt through
//!       `Document::encode` so its `len`/`crc32` stay self-consistent and exactly
//!       ONE property is under test; each asserts the exact `ContractDecline`
//!       variant (never a bool). These supersede the ledger's `lint:kiss_wire`
//!       earmark for §6.1-0007/-0008 with stronger, behavioral teeth.
//!
//!   (B) DOCUMENT / SECTION-SCHEMA CONSISTENCY — honest doc-lints that parse the
//!       normative clause text (and the transcribed Appendix C golden bytes) and
//!       set/order-compare extracted STRUCTURE. Every lint stays WITHIN
//!       KISS-Contract, except the determinism enum, which is compared against a
//!       cross-standard CONSTANT token-set (never read from KISS-Ops BY CLAUSE-ID,
//!       so it does not reverse-bind or over-credit KISS-Ops).
//!
//! Clauses bound: KISS-CONTRACT-6.1-0001, -6.1-0003, -6.1-0007, -6.1-0008,
//! -6.2-0001, -6.3-0001, -6.8-0001, -6.8-0003, -6.9-0001, -6.11-0003, -6.11-0004.
//!
//! No new golden bytes are invented: the only golden used (Identity) is
//! transcribed verbatim from Contract Appendix C, identical to the copy in
//! `contract_golden.rs` (integration test files compile as separate crates and
//! cannot share a `const`).

use kiss_conformance::contract::*;

// ---------------------------------------------------------------------------
// Transcribed golden + block builders (Contract Appendix C; LF terminators).
// ---------------------------------------------------------------------------

/// Appendix C Identity block, transcribed verbatim (§6.11-0004/-0005; heading id
/// 1). Field lines in §6.3-0001 field-schema order; `revision_hash` in the
/// `<n>:<hex>` opaque-blob form (§6.11-0001).
const IDENTITY_GOLDEN: &str = "\
[section:1:identity]
contract_kind = kiss-contract
contract_version = 1
kernel_name = add_f32_strided_sm89
revision_hash = 4:deadbeef
accept_predicate = bin/f32,f32,f32/strided/cuda:sm89
op_identity = add
target_capability = cuda:sm89
";

/// The Appendix C Identity block, emitted from field data in §6.3-0001 order via
/// the codec's `render_block`.
fn identity_block() -> Vec<u8> {
    render_block(
        1,
        "identity",
        &[
            ("contract_kind", Value::Str("kiss-contract".into())),
            ("contract_version", Value::Str("1".into())),
            ("kernel_name", Value::Str("add_f32_strided_sm89".into())),
            ("revision_hash", Value::Blob(vec![0xde, 0xad, 0xbe, 0xef])),
            ("accept_predicate", Value::Str("bin/f32,f32,f32/strided/cuda:sm89".into())),
            ("op_identity", Value::Str("add".into())),
            ("target_capability", Value::Str("cuda:sm89".into())),
        ],
    )
}

/// The Appendix C Semantics block; `op_dag` rendered by the canonical serializer.
fn semantics_block() -> Vec<u8> {
    render_block(
        2,
        "semantics",
        &[
            ("semantics_kind", Value::Str("machine-checkable-IR".into())),
            ("op_dag", Value::Str(serialize_op_dag(&OpTree::leaf("add")))),
        ],
    )
}

/// The two-block Appendix C document body (Identity + Semantics).
fn appendix_c_body() -> Vec<u8> {
    let mut body = identity_block();
    body.extend_from_slice(&semantics_block());
    body
}

/// A well-formed document over the given kind/version and the Appendix C body,
/// with `len`/`crc32` made self-consistent by `Document::encode`.
fn document(kind: &str, version: &str) -> Vec<u8> {
    Document {
        contract_kind: kind.into(),
        contract_version: version.into(),
        body: appendix_c_body(),
    }
    .encode()
}

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a phrase. (Established by
// the KISS-Emit modules; `spec/` is one directory above the crate.)
// ---------------------------------------------------------------------------

/// Read a `spec/<name>` markdown file.
fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// The body text of a single clause: from its `**KISS-…**` bold anchor up to the
/// next clause definition or the next markdown heading.
fn clause_block<'a>(md: &'a str, id: &str) -> &'a str {
    let anchor = format!("**{id}**");
    let start = md
        .find(&anchor)
        .unwrap_or_else(|| panic!("clause `{id}` is not defined in the spec text"));
    let rest = &md[start..];
    let mut end = rest.len();
    for pat in ["\n- **KISS-", "\n#"] {
        if let Some(i) = rest.get(1..).and_then(|r| r.find(pat)) {
            end = end.min(i + 1);
        }
    }
    &rest[..end]
}

/// The ORDERED field list inside the first `{…}` literal of `block`, each field
/// stripped of all internal whitespace so a line-wrapped list still compares
/// verbatim. Order is preserved (field-schema order is normative, §6.11-0005).
fn field_list_ordered(block: &str) -> Vec<String> {
    let open = block.find('{').expect("no `{…}` field list in clause block");
    let close = block[open..].find('}').expect("unterminated `{…}` field list") + open;
    block[open + 1..close]
        .split(',')
        .map(|m| m.split_whitespace().collect::<String>())
        .filter(|m| !m.is_empty())
        .collect()
}

/// The members of the first `{…}` enum literal as an unordered set (an enum has
/// no ordering obligation, unlike a field schema).
fn enum_set(block: &str) -> std::collections::BTreeSet<String> {
    field_list_ordered(block).into_iter().collect()
}

/// Every all-ASCII-lowercase backtick token of `block`, in document order. In
/// §6.11-0004 these are exactly the seven lowercase section names (`<id>`,
/// `<name>`, `[section:…]`, `1`, `7` are all excluded by the lowercase-word
/// filter, so a capitalized or angle-bracketed token never leaks in).
fn backtick_lower_words(block: &str) -> Vec<String> {
    block
        .split('`')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, s)| s.to_string())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase()))
        .collect()
}

/// The capitalized section names that immediately precede each `(§6.` reference
/// in `block`, lowercased, in document order (the §6.2-0001 restatement form
/// `Identity (§6.3), Semantics (§6.4), …`).
fn section_names_before_refs(block: &str) -> Vec<String> {
    let parts: Vec<&str> = block.split("(§6.").collect();
    if parts.len() < 2 {
        return Vec::new();
    }
    parts[..parts.len() - 1]
        .iter()
        .filter_map(|seg| {
            seg.split(|c: char| !c.is_ascii_alphabetic())
                .filter(|w| !w.is_empty())
                .last()
                .map(|w| w.to_ascii_lowercase())
        })
        .collect()
}

/// The `<key>` of every `<key> = <value>` field line in a rendered block/golden,
/// in document order (the heading line, carrying no ` = `, is skipped).
fn field_keys(rendered: &str) -> Vec<String> {
    rendered
        .lines()
        .filter(|l| l.contains(" = "))
        .map(|l| l.split(" = ").next().unwrap().trim().to_string())
        .collect()
}

/// Collapse all runs of whitespace (healing markdown line wraps) to single
/// spaces, so a phrase split across a wrapped line still matches. Backtick chars
/// and single-token values (no internal whitespace) survive unchanged.
fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The first backtick token appearing after `phrase` in `block` (e.g. the pinned
/// value in "the exact token `X`").
fn backtick_after<'a>(block: &'a str, phrase: &str) -> &'a str {
    let p = block
        .find(phrase)
        .unwrap_or_else(|| panic!("phrase `{phrase}` not found in clause block"));
    let rest = &block[p + phrase.len()..];
    let start = rest.find('`').expect("no backtick token after phrase") + 1;
    let end = rest[start..].find('`').expect("unterminated backtick token") + start;
    &rest[start..end]
}

/// The KISS-Ops-owned canonical determinism/fidelity enum
/// `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`. This is a
/// reference CONSTANT, deliberately NOT a re-read of KISS-Ops by clause-id: a
/// clause-id cross-read of KISS-Ops would reverse-bind and over-credit the
/// KISS-Ops owner clause this module does not exercise. KISS-Contract's job here
/// is only to prove its §6.8-0003 restatement imports the enum verbatim.
fn ops_determinism_enum() -> std::collections::BTreeSet<String> {
    ["exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

// ===========================================================================
// (A) Transport-reader discipline — the §6.1 / §6.11 codec.
// ===========================================================================

/// KISS-CONTRACT-6.11-0003 — the header line declares the document's inner
/// framing: `len=<N>` is the exact body byte count and `crc32=<HHHHHHHH>` is the
/// CRC-32 (IEEE 802.3, reflected `0xEDB88320`, init/final `0xFFFFFFFF`) of exactly
/// those N body bytes; a reader MUST reject a body that is not exactly N bytes or
/// whose CRC-32 does not match. This is the len/crc integrity SEMANTICS, distinct
/// from the header FORM already backed by §6.11-0002.
///
/// TEETH: a length-only reader that accepts a stale-CRC body after a single
/// flipped byte (caught by `BadChecksum`); a stub/zero/wrong-polynomial CRC
/// (caught by the canonical `crc32_ieee(b"123456789") == 0xCBF43926` check value);
/// a reader that trusts the declared length over the actual bytes (caught by
/// `BadLength`).
#[test]
fn test_contract_document_inner_length_and_crc() {
    // The CRC-32 is the real IEEE 802.3 CRC: its canonical check value over
    // ASCII "123456789" is 0xCBF43926. A stub / zero / wrong-polynomial CRC fails.
    assert_eq!(
        crc32_ieee(b"123456789"),
        0xCBF4_3926,
        "KISS-CONTRACT-6.11-0003: crc32_ieee is not the canonical IEEE-802.3 CRC-32"
    );

    let body = appendix_c_body();
    let good = Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "1".into(),
        body: body.clone(),
    }
    .encode();

    // Baseline: the reader returns the ACTUAL body length and the body's real CRC
    // (it validated both against the actual bytes, not just echoed the header).
    let hdr = read_document(&good).expect("KISS-CONTRACT-6.11-0003: a self-consistent document must read");
    assert_eq!(hdr.body_len, body.len(), "KISS-CONTRACT-6.11-0003: body_len must equal the actual body byte count");
    assert_eq!(hdr.crc32, crc32_ieee(&body), "KISS-CONTRACT-6.11-0003: crc32 must equal the body's computed CRC-32");

    // Length integrity: append one stray byte so the body is one byte longer than
    // `len=<N>` declares. A reader trusting the declared length would still accept;
    // the actual-vs-declared check MUST decline with the exact overrun reported.
    let mut too_long = good.clone();
    too_long.push(b'x');
    assert_eq!(
        read_document(&too_long),
        Err(ContractDecline::BadLength { declared: body.len() as u64, actual: body.len() + 1 }),
        "KISS-CONTRACT-6.11-0003: a body longer than the declared length must decline BadLength"
    );

    // CRC integrity: flip one body byte, leaving the declared length unchanged. A
    // length-only reader would accept it; the CRC MUST catch the flip, and the
    // reported computed CRC differs from the (now stale) declared CRC.
    let mut flipped = good.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 0xFF; // the final LF of the body — still within the N body bytes
    match read_document(&flipped) {
        Err(ContractDecline::BadChecksum { declared, computed }) => {
            assert_eq!(declared, crc32_ieee(&body), "KISS-CONTRACT-6.11-0003: declared CRC is the original body's");
            assert_ne!(declared, computed, "KISS-CONTRACT-6.11-0003: a flipped body must compute a different CRC");
        }
        other => panic!("KISS-CONTRACT-6.11-0003: expected BadChecksum on a corrupted body, got {other:?}"),
    }
}

/// KISS-CONTRACT-6.1-0003 — a reader MUST reject, with a typed decline, a contract
/// whose declared `contract_version` it does not support, and MUST NOT partially
/// import it. Distinct from §6.1-0008 (which pins the exact ACCEPTED token and
/// its byte-exactness); this is the general unsupported-version decline.
///
/// TEETH: a tolerant reader that accepts any version, or partially imports an
/// unsupported one (returns Ok with a best-effort header).
#[test]
fn test_contract_reject_unknown_version() {
    // Each rebuilt via Document::encode so len/crc stay self-consistent and ONLY
    // the version triggers the decline.
    for v in ["2", "99", "0"] {
        let doc = document("kiss-contract", v);
        assert_eq!(
            read_document(&doc),
            Err(ContractDecline::UnknownVersion { got: v.to_string() }),
            "KISS-CONTRACT-6.1-0003: unsupported version `{v}` must decline UnknownVersion, never Ok/partial-import"
        );
        // The no-partial-import half needs no separate assertion: the `assert_eq!`
        // above pins the whole `Result`, so an `Ok` with a best-effort header fails
        // it. A follow-up `is_err()` here restated the same fact in a weaker form.
    }
}

/// KISS-CONTRACT-6.1-0007 — the only recognized `contract_kind` at this schema
/// version is the exact token `kiss-contract` (UTF-8, byte-exact); a reader MUST
/// reject any other. The accepted token is EXTRACTED from §6.1-0007's own "the
/// exact token `X`" prose, so a spec/reader rename drifts loudly. (Behavioral;
/// supersedes the ledger's `lint:kiss_wire` earmark with stronger teeth.)
///
/// TEETH: a case-folding reader (accepts `KISS-CONTRACT`), a separator-tolerant
/// reader (accepts `kiss_contract`), or a prefix/suffix-accepting reader
/// (accepts `kiss-contracts`).
#[test]
fn test_contract_kind_recognized_token() {
    let contract = read_spec("contract.md");
    let block = norm(clause_block(&contract, "KISS-CONTRACT-6.1-0007"));
    let accepted = backtick_after(&block, "the exact token ");
    assert_eq!(
        accepted, "kiss-contract",
        "KISS-CONTRACT-6.1-0007: the §6.1-0007 recognized token drifted from `kiss-contract`"
    );

    // The reader accepts EXACTLY the token the spec names.
    let good = document(accepted, "1");
    let hdr = read_document(&good).expect("KISS-CONTRACT-6.1-0007: the spec-declared kind must be accepted");
    assert_eq!(
        hdr.contract_kind, accepted,
        "KISS-CONTRACT-6.1-0007: the reader accepted a different token than the spec pins"
    );

    // Byte-mutated near-misses are each declined with the offending token echoed.
    for near in ["KISS-CONTRACT", "kiss_contract", "kiss-contracts", "kisscontract"] {
        assert_eq!(
            read_document(&document(near, "1")),
            Err(ContractDecline::UnknownKind { got: near.to_string() }),
            "KISS-CONTRACT-6.1-0007: near-miss kind `{near}` must decline UnknownKind (no case-fold / prefix accept)"
        );
    }
}

/// KISS-CONTRACT-6.1-0008 — the `contract_version` of THIS schema is the exact
/// decimal token `1`; a reader MUST accept exactly `1` (UTF-8, byte-exact) and
/// reject any other value. The version is compared BYTE-for-byte, never
/// integer-parsed. Distinct from §6.1-0003 (general unsupported decline); this
/// pins the exact accepted byte-token of this schema.
///
/// TEETH: a reader that numeric-parses the version would accept `01` or `1.0`
/// (both equal to 1 numerically); byte-exact comparison declines them.
#[test]
fn test_contract_version_value_pinned() {
    let contract = read_spec("contract.md");
    let block = norm(clause_block(&contract, "KISS-CONTRACT-6.1-0008"));
    let pinned = backtick_after(&block, "the exact decimal token ");
    assert_eq!(pinned, "1", "KISS-CONTRACT-6.1-0008: the §6.1-0008 pinned version drifted from `1`");

    // Exactly the pinned token is accepted.
    let hdr = read_document(&document("kiss-contract", pinned))
        .expect("KISS-CONTRACT-6.1-0008: version `1` must be accepted");
    assert_eq!(hdr.contract_version, "1", "KISS-CONTRACT-6.1-0008: reader accepted a non-`1` version");

    // Numeric-equal-but-byte-different spellings are declined UnknownVersion — a
    // numeric parser would wrongly accept `01` / `1.0` / `+1`.
    for near in ["01", "1.0", "10", "2", "+1"] {
        assert_eq!(
            read_document(&document("kiss-contract", near)),
            Err(ContractDecline::UnknownVersion { got: near.to_string() }),
            "KISS-CONTRACT-6.1-0008: `{near}` must decline (byte-exact, never integer-parsed)"
        );
    }
    // A whitespace-padded version breaks the single-space header field split
    // (§6.11-0002 pins single-space separators), so the fault is in the FRAMING and
    // the required decline is MalformedHeader — not UnknownVersion, which would mean
    // the reader had successfully split the fields and merely disliked the value.
    //
    // Pinning the exact code rather than `is_err()` is what gives this teeth: a
    // reader with a lenient field-count check (`parts.len() < 5`) splits
    // `KISC kiss-contract  1 …` into six, reads the empty string between the two
    // spaces as the version, and declines `UnknownVersion { got: "" }`. That reader
    // has silently accepted a non-single-space separator — a real framing defect —
    // and `is_err()` cannot see it, because a wrong-code decline is still a decline.
    assert_eq!(
        read_document(&document("kiss-contract", " 1")),
        Err(ContractDecline::MalformedHeader),
        "KISS-CONTRACT-6.1-0008: a whitespace-padded version must decline as a FRAMING \
         fault — a decline naming the version instead means the reader tolerated a \
         double separator and split the fields anyway"
    );
}

/// KISS-CONTRACT-6.1-0001 — a contract is a SELF-DELIMITING structured/text
/// document: a reader determines the exact byte extent from the document itself
/// (the header line through its LF, plus the declared body length), independent of
/// any enclosing transport frame. It MUST NOT be framed as a length-prefixed
/// binary envelope. Distinct angle from §6.11-0003 (which owns len/crc VALIDATION).
///
/// TEETH: a length-prefixed binary framing (the first 4 bytes would be a binary
/// length word, not ASCII `KISC`); an off-by-one extent computation.
#[test]
fn test_contract_self_delimiting_document() {
    let body = appendix_c_body();
    let doc = Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "1".into(),
        body: body.clone(),
    }
    .encode();

    // The document opens with the 4 ASCII magic bytes `KISC`, NOT a binary length
    // prefix. A length-prefixed binary envelope would carry a u32 byte-length here
    // (for this document that word would contain a non-printable high byte).
    assert_eq!(&doc[0..4], b"KISC", "KISS-CONTRACT-6.1-0001: document must open with the ASCII magic, not a binary length");
    assert!(
        doc[0..4].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        "KISS-CONTRACT-6.1-0001: the leading 4 bytes must be printable ASCII (text framing, not a binary length word)"
    );

    // The length rides as decimal-ASCII text `len=<N>` INSIDE the header line, not
    // as a leading binary word. Recover N from that text alone.
    let lf = doc.iter().position(|&b| b == b'\n').expect("header must terminate in LF");
    let header = std::str::from_utf8(&doc[..lf]).expect("header is UTF-8 text");
    let len_field = header
        .split(' ')
        .find_map(|f| f.strip_prefix("len="))
        .expect("KISS-CONTRACT-6.1-0001: header must carry a decimal-ASCII `len=<N>` field");
    let declared_n: usize = len_field.parse().expect("KISS-CONTRACT-6.1-0001: `len=<N>` must be decimal ASCII");

    // The extent is fully determined by the header alone: (header line + LF) + N.
    let extent = (lf + 1) + declared_n;
    assert_eq!(
        extent,
        doc.len(),
        "KISS-CONTRACT-6.1-0001: extent from the header (header+LF+N) must equal the document length (self-delimiting, off-by-one free)"
    );
    assert_eq!(declared_n, body.len(), "KISS-CONTRACT-6.1-0001: the declared N must be the actual body byte count");

    // The reader's recovered extent (body_len) matches the header-declared N.
    let hdr = read_document(&doc).expect("KISS-CONTRACT-6.1-0001: a self-delimiting document must read");
    assert_eq!(hdr.body_len, declared_n, "KISS-CONTRACT-6.1-0001: read_document must recover the header-declared extent");
}

// ===========================================================================
// (B) Document / section-schema consistency — honest doc-lints.
// ===========================================================================

/// KISS-CONTRACT-6.3-0001 — the Identity section carries exactly the seven fields
/// `{contract_kind, contract_version, kernel_name, revision_hash,
/// accept_predicate, op_identity, target_capability}`, serialized in that order.
/// Two INDEPENDENT surfaces are ordered-compared: the §6.3-0001 spec field list
/// and the `<key> = ` lines of the transcribed Appendix C Identity golden.
///
/// TEETH: any rename, drop, reorder, or addition of an Identity field on either
/// surface (spec prose vs golden bytes) makes the two ordered lists disagree.
#[test]
fn test_contract_identity_field_schema() {
    let contract = read_spec("contract.md");
    let spec_fields = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.3-0001"));
    let golden_keys = field_keys(IDENTITY_GOLDEN);

    assert_eq!(spec_fields.len(), 7, "KISS-CONTRACT-6.3-0001: §6.3-0001 must list exactly 7 Identity fields, got {spec_fields:?}");
    assert_eq!(golden_keys.len(), 7, "KISS-CONTRACT-6.3-0001: the Appendix C Identity golden must carry exactly 7 field lines, got {golden_keys:?}");
    assert_eq!(
        spec_fields, golden_keys,
        "KISS-CONTRACT-6.3-0001: the §6.3-0001 field list and the Appendix C Identity golden drifted (rename/drop/reorder)"
    );
}

/// KISS-CONTRACT-6.11-0004 — the body is exactly the seven section blocks in the
/// fixed order Identity(1)…Provenance(7); each begins with the pinned heading line
/// `[section:<id>:<name>]` + LF, `<name>` the lowercase section name matching the
/// decimal id. The seven (id,name) pairs are DRIVEN from the §6.11-0004 name list
/// through the codec's `render_block`. Stronger than the ledger's `lint:kiss_tables`
/// earmark: it exercises the codec's heading BYTES against the spec table.
///
/// TEETH: a capitalized/misspelled heading (`[section:1:Identity]`), a wrong
/// id-to-name pairing, or a name-less `[section:1]` form — all fail the exact
/// per-heading byte comparison; a drift in the §6.11-0004 name list (reorder,
/// misspell, capitalization) fails the ordered-name check.
#[test]
fn test_contract_document_section_headings() {
    let contract = read_spec("contract.md");
    let names = backtick_lower_words(clause_block(&contract, "KISS-CONTRACT-6.11-0004"));
    let canonical = [
        "identity", "semantics", "interface", "dispatch", "capabilities", "guarantees", "provenance",
    ];
    assert_eq!(
        names, canonical,
        "KISS-CONTRACT-6.11-0004: the §6.11-0004 section-name list drifted (a capitalized name would drop out of the lowercase filter)"
    );

    // Drive the codec with each spec-derived (id, name) pair; the rendered heading
    // must be EXACTLY `[section:<id>:<name>]\n` — lowercase name, decimal id,
    // colon-delimited, name present.
    for (i, name) in names.iter().enumerate() {
        let id = (i + 1) as u8;
        let rendered = render_block(id, name, &[]);
        let expected = format!("[section:{id}:{name}]\n");
        assert_eq!(
            std::str::from_utf8(&rendered).unwrap(),
            expected,
            "KISS-CONTRACT-6.11-0004: render_block must emit the pinned heading `[section:{id}:{name}]` verbatim"
        );
        // Explicit negative checks: no capitalized name, no name-less form.
        assert!(
            rendered.starts_with(expected.as_bytes()),
            "KISS-CONTRACT-6.11-0004: heading id {id} is not the pinned lowercase, named, colon-delimited form"
        );
    }
}

/// KISS-CONTRACT-6.2-0001 — every contract carries the universal required core of
/// seven sections (Identity, Semantics, Interface, Dispatch, Capabilities,
/// Guarantees, Provenance). The seven names restated in §6.2-0001 are ordered-
/// compared against the seven names in §6.11-0004's fixed body order. Both clauses
/// are KISS-Contract's own — a within-standard consistency check, no reverse-bind.
///
/// TEETH: either CONTRACT clause dropping, adding, or reordering a section makes
/// the two ordered seven-name sequences disagree (or breaks the cardinality-7
/// assertion).
#[test]
fn test_contract_seven_section_core() {
    let contract = read_spec("contract.md");
    let core = section_names_before_refs(clause_block(&contract, "KISS-CONTRACT-6.2-0001"));
    let body_order = backtick_lower_words(clause_block(&contract, "KISS-CONTRACT-6.11-0004"));

    assert_eq!(core.len(), 7, "KISS-CONTRACT-6.2-0001: §6.2-0001 must enumerate exactly 7 core sections, got {core:?}");
    assert_eq!(body_order.len(), 7, "KISS-CONTRACT-6.2-0001: §6.11-0004 must fix exactly 7 body sections, got {body_order:?}");
    assert_eq!(
        core, body_order,
        "KISS-CONTRACT-6.2-0001: the §6.2-0001 core and the §6.11-0004 body order disagree (a section was dropped/added/reordered)"
    );
}

/// KISS-CONTRACT-6.8-0001 — the Guarantees section carries exactly the eight
/// fields `{reference_function, per_backend_ulp_tiers, determinism_class,
/// math_precision, accumulation_type, bit_stability, audited_status,
/// cost_provenance}`, in that order; `audited_status`'s single normative home is
/// Guarantees and it MUST NOT appear in Provenance (§6.9-0001). Cross-checked
/// against §6.9-0001 (both KISS-Contract's own).
///
/// TEETH: a drift that scatters `audited_status` into the Provenance field list,
/// or drops/renames `accumulation_type`, breaks either the ordered-list equality
/// or the single-home disjointness.
#[test]
fn test_contract_guarantees_field_schema() {
    let contract = read_spec("contract.md");
    let g = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.8-0001"));
    let expected = [
        "reference_function",
        "per_backend_ulp_tiers",
        "determinism_class",
        "math_precision",
        "accumulation_type",
        "bit_stability",
        "audited_status",
        "cost_provenance",
    ];
    assert_eq!(
        g, expected,
        "KISS-CONTRACT-6.8-0001: the Guarantees field list drifted from the pinned 8-field ordered schema"
    );
    assert!(g.iter().any(|f| f == "accumulation_type"), "KISS-CONTRACT-6.8-0001: `accumulation_type` must be present");
    assert!(g.iter().any(|f| f == "audited_status"), "KISS-CONTRACT-6.8-0001: `audited_status` must be present in Guarantees");

    // Single home: `audited_status` MUST NOT also live in the Provenance schema.
    let prov = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.9-0001"));
    assert!(
        !prov.iter().any(|f| f == "audited_status"),
        "KISS-CONTRACT-6.8-0001: `audited_status` leaked into the Provenance (§6.9-0001) field list — its single home is Guarantees"
    );
}

/// KISS-CONTRACT-6.9-0001 — the Provenance section carries exactly the five fields
/// `{kernel_source, revision_base, revision_hash, cost_provenance,
/// negotiation_metadata}`, in that order, and MUST NOT carry `audited_status`
/// (a derived Guarantees field). Pairs with §6.8-0001 from the opposite side of
/// the single-home disjointness.
///
/// TEETH: `audited_status` migrating INTO the Provenance list, or a Provenance
/// field rename/drop/reorder, breaks the ordered-list equality; the companion
/// check confirms `audited_status` DOES live in §6.8-0001, so the disjointness is
/// proven from both sides.
#[test]
fn test_contract_provenance_field_schema() {
    let contract = read_spec("contract.md");
    let prov = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.9-0001"));
    let expected = [
        "kernel_source",
        "revision_base",
        "revision_hash",
        "cost_provenance",
        "negotiation_metadata",
    ];
    assert_eq!(
        prov, expected,
        "KISS-CONTRACT-6.9-0001: the Provenance field list drifted from the pinned 5-field ordered schema"
    );
    assert!(
        !prov.iter().any(|f| f == "audited_status"),
        "KISS-CONTRACT-6.9-0001: Provenance MUST NOT carry `audited_status`"
    );

    // Opposite side of the single-home invariant: `audited_status` IS in Guarantees.
    let g = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.8-0001"));
    assert!(
        g.iter().any(|f| f == "audited_status"),
        "KISS-CONTRACT-6.9-0001: `audited_status` must live in the Guarantees (§6.8-0001) schema, its single home"
    );
}

/// KISS-CONTRACT-6.8-0003 — the `determinism_class` is the single canonical
/// KISS-Ops enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`,
/// spelled verbatim and imported from KISS-Ops; an implementation MUST NOT re-fork
/// or re-spell the enum. The enum set parsed out of §6.8-0003's restatement is
/// compared byte-for-byte against a reference CONSTANT token-set — NOT read from
/// KISS-Ops by clause-id, so this does not reverse-bind/over-credit the KISS-Ops
/// owner clause.
///
/// TEETH: KISS-Contract re-spelling a member (`exact_byte`, `ulp-tolerance`,
/// `order_invariant`) makes the parsed set differ from the canonical constant set.
#[test]
fn test_contract_determinism_class_imported() {
    let contract = read_spec("contract.md");
    let parsed = enum_set(clause_block(&contract, "KISS-CONTRACT-6.8-0003"));
    let canonical = ops_determinism_enum();
    assert_eq!(
        canonical.len(),
        3,
        "KISS-CONTRACT-6.8-0003: the canonical determinism enum must have 3 members"
    );
    assert_eq!(
        parsed, canonical,
        "KISS-CONTRACT-6.8-0003: §6.8-0003's determinism enum {parsed:?} is not the verbatim canonical set {canonical:?} (a member was re-forked/re-spelled)"
    );
}
