//! KISS-Conform golden + decline vectors for the KISS-Contract structured/text
//! **document** framing (Contract §6.11) and its transport hard-reject reader
//! (§6.1). The golden document bytes are transcribed verbatim from Contract
//! Appendix C — the §2.5 strided `add` contract's header line, Identity block,
//! and Semantics block.

use kiss_conformance::assert_golden;
use kiss_conformance::contract::*;

// #349: the transcribed Appendix C goldens and their block builders are the LIBRARY's now, imported
// here under their familiar local names. This file, contract_framing.rs, and the emitted
// contract_vectors.json therefore share ONE builder and ONE transcription — a hand-transcribed copy
// here would be exactly the duplication #349 exists to end (§6.5-0002: the golden's authority belongs
// to the codec, not a test).
use kiss_conformance::contract::{
    appendix_c_golden_document as good_document, appendix_c_identity_block as identity_block,
    appendix_c_semantics_block as semantics_block, APPENDIX_C_IDENTITY_GOLDEN as IDENTITY_GOLDEN,
    APPENDIX_C_SEMANTICS_GOLDEN as SEMANTICS_GOLDEN,
};

/// Hex-encode a byte slice as the space-separated uppercase hex `assert_golden`
/// expects, so a transcribed ASCII golden can be passed through the harness's
/// clause-citing exact-byte comparator.
fn to_golden_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

/// Enforces KISS-CONTRACT-6.11-0001 — the four value encodings and the exact
/// `<key> = <value>` spacing. TEETH: a `key=value` emitter with no spaces around
/// `=`; an `[a,b]` array emitter with no space after the comma; an uppercase or
/// wrong-count opaque-blob emitter.
// Proven: KISS-CONTRACT-6.11-0001 (subject: impl; ref: PROVEN_BATCH2.md)
#[test]
fn test_contract_text_field_encoding() {
    // `<key> = <value>`: exactly one space each side of `=`, LF-terminated.
    // Catches a `key=value` no-space emitter.
    assert_eq!(
        field_line("contract_kind", &Value::Str("kiss-contract".into())),
        "contract_kind = kiss-contract\n"
    );

    // Array: `[` elems `, ` `]` — comma THEN one space. Catches `[f32,f32,f32]`.
    assert_eq!(
        Value::Array(vec!["f32".into(), "f32".into(), "f32".into()]).render(),
        "[f32, f32, f32]"
    );

    // Opaque blob: `<n>:<hex>`, 2·n LOWERCASE hex digits. Catches `4:DEADBEEF`
    // (uppercase) or a wrong byte count.
    assert_eq!(Value::Blob(vec![0xde, 0xad, 0xbe, 0xef]).render(), "4:deadbeef");
    // An empty blob is exactly `0:` (§6.11-0001).
    assert_eq!(Value::Blob(vec![]).render(), "0:");

    // Integer: decimal ASCII, a negative value carrying a leading `-`.
    assert_eq!(Value::Int(42).render(), "42");
    assert_eq!(Value::Int(-5).render(), "-5");
}

/// Enforces KISS-CONTRACT-6.11-0002 — the pinned header line
/// `KISC kiss-contract 1 len=<N> crc32=<HHHHHHHH>\n`: 4-byte magic `KISC`
/// (`0x4B 0x49 0x53 0x43`), single-space separators, `len=<N>` decimal body
/// length, `crc32=<HHHHHHHH>` 8 lowercase hex digits, then a single LF. TEETH: a
/// CRLF emitter; a wrong magic; a renamed/missing `len=`/`crc32=` field; a stub
/// CRC (guarded by the canonical `0xCBF43926` check value of §6.11-0003).
// Proven: KISS-CONTRACT-6.11-0002 (subject: impl; ref: PROVEN_BATCH2.md)
#[test]
fn test_contract_document_header_line() {
    // The CRC-32 must be the real IEEE 802.3 CRC: its canonical check value over
    // ASCII "123456789" is 0xCBF43926. Catches a stub/zero/wrong-polynomial CRC.
    assert_eq!(crc32_ieee(b"123456789"), 0xCBF4_3926);

    // A document over a known body — the two pinned Appendix C blocks.
    let mut body = identity_block();
    body.extend_from_slice(&semantics_block());
    let doc = Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "1".into(),
        body: body.clone(),
    }
    .encode();

    // Split the header line (through, but excluding, the terminating LF).
    let lf = doc.iter().position(|&b| b == b'\n').expect("header must end in LF");
    let header = &doc[..lf];

    // Magic: the literal wire bytes 0x4B 0x49 0x53 0x43 ("KISC").
    assert_eq!(&doc[0..4], &[0x4B, 0x49, 0x53, 0x43]);

    // LF-only line ending: no CR anywhere in the header. Catches a CRLF emitter.
    assert!(!header.contains(&b'\r'), "§6.11-0002: LF-only, a CR is not permitted");
    // The byte immediately before the body is a lone LF, not preceded by CR.
    assert_eq!(doc[lf], b'\n');
    assert_ne!(doc[lf - 1], b'\r');

    // Exact pinned form: single-space separators, `len=<N>`, `crc32=<8 hex>`.
    let expected_header = format!(
        "KISC kiss-contract 1 len={} crc32={:08x}",
        body.len(),
        crc32_ieee(&body)
    );
    assert_eq!(std::str::from_utf8(header).unwrap(), expected_header);

    // The declared length is the decimal body byte count, and the body follows
    // the header LF verbatim.
    assert_eq!(&doc[lf + 1..], &body[..]);
    // The crc32 field is 8 lowercase hex digits.
    let crc_field = expected_header.rsplit(' ').next().unwrap();
    let crc_hex = crc_field.strip_prefix("crc32=").unwrap();
    assert_eq!(crc_hex.len(), 8);
    assert!(crc_hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')));
}

/// Enforces KISS-CONTRACT-6.11-0005 — within a section block the field lines
/// appear in the exact field-schema order (§6.3-0001 for Identity). The emitted
/// Identity block is compared byte-for-byte to the Appendix C golden. TEETH: a
/// HashMap-backed emitter whose field order is nondeterministic (any permutation
/// of the seven Identity fields fails this exact-byte golden).
// Proven: KISS-CONTRACT-6.11-0005 (subject: impl; ref: PROVEN_BATCH2.md)
#[test]
fn test_contract_document_field_order() {
    assert_golden(
        "KISS-CONTRACT-6.11-0005",
        "identity_block",
        &identity_block(),
        &to_golden_hex(IDENTITY_GOLDEN.as_bytes()),
    );
}

/// Enforces KISS-CONTRACT-6.11-0007 — the `op_dag` array in KISS-Grammar
/// canonical node order: every child strictly earlier than its parent, each
/// node's `child_edges` referencing strictly-earlier indices, the **root last**.
/// The one-node `add` DAG is compared to the Appendix C Semantics golden; a
/// two-node fusion pins the child-before-parent ordering. TEETH: a root-FIRST
/// recursive-descent serializer (it would place the root at index 0 and its
/// child at a later index).
#[test]
fn test_contract_text_op_dag() {
    // One-node golden: `op_dag = [Op{add; ; []}]` (root = only, last, element).
    assert_golden(
        "KISS-CONTRACT-6.11-0007",
        "semantics_block",
        &semantics_block(),
        &to_golden_hex(SEMANTICS_GOLDEN.as_bytes()),
    );

    // A two-node fusion: root `mul` over one child `add`. Canonical order emits
    // the child (index 0) before the root (index 1), whose child_edges = [0].
    let fusion = OpTree {
        op_name: "mul".into(),
        op_attrs: vec![],
        children: vec![OpTree::leaf("add")],
    };
    let s = serialize_op_dag(&fusion);
    assert_eq!(s, "[Op{add; ; []}, Op{mul; ; [0]}]");
    // Root strictly last; a root-first emitter would produce `[Op{mul; ...`.
    assert!(s.starts_with("[Op{add"), "child MUST appear before its parent");
    assert!(s.ends_with("Op{mul; ; [0]}]"), "root MUST appear last");
    // The root's only child edge references the strictly-earlier index 0.
    assert!(s.contains("Op{mul; ; [0]}"), "child_edges MUST reference an earlier index");

    // A node's op_attrs render as a `, `-separated key=value list (empty elided).
    let gather = OpTree {
        op_name: "gather".into(),
        op_attrs: vec![("axis".into(), "k".into()), ("oob".into(), "clamp".into())],
        children: vec![],
    };
    assert_eq!(serialize_op_dag(&gather), "[Op{gather; axis=k, oob=clamp; []}]");
}

/// Enforces KISS-CONTRACT-6.1-0002 — a reader MUST fail loudly with a typed
/// decline on: no magic, an absent/malformed header line, an unrecognized
/// `contract_kind`, or a checksum that does not validate; it MUST NOT repair,
/// skip, or import a headingless/magic-less block. TEETH: a tolerant reader that
/// scans past junk to find a heading, ignores an unknown kind, or accepts a body
/// whose CRC does not match.
#[test]
fn test_contract_reject_malformed_header() {
    // Baseline: the well-formed document reads (transport framing valid).
    let good = good_document();
    assert!(read_document(&good).is_ok(), "a well-formed document MUST read");

    // (a) No magic: junk before any heading is NOT scanned past.
    assert_eq!(
        read_document(b"garbage\n[section:1:identity]\ncontract_kind = kiss-contract\n"),
        Err(ContractDecline::NoMagic)
    );

    // (b) Malformed header line: right magic, but not the pinned five-field form.
    assert_eq!(
        read_document(b"KISC not a valid header line\n[section:1:identity]\n"),
        Err(ContractDecline::MalformedHeader)
    );

    // (c) Unrecognized contract_kind: `kiss-kontract` is not the pinned token.
    // Rebuild via the emitter so len/crc stay self-consistent and only the KIND
    // triggers the decline (a tolerant reader would ignore the misspelling).
    let mut body = identity_block();
    body.extend_from_slice(&semantics_block());
    let bad_kind = Document {
        contract_kind: "kiss-kontract".into(),
        contract_version: "1".into(),
        body,
    }
    .encode();
    assert_eq!(
        read_document(&bad_kind),
        Err(ContractDecline::UnknownKind { got: "kiss-kontract".into() })
    );

    // (d) Bad checksum: corrupt one body byte, leaving the declared crc32 stale.
    // The declared length still matches, so a length-only reader would accept it;
    // the CRC MUST catch the flip.
    let mut bad_crc = good.clone();
    let last = bad_crc.len() - 1;
    bad_crc[last] ^= 0xFF; // flip a body byte (after the header)
    match read_document(&bad_crc) {
        Err(ContractDecline::BadChecksum { .. }) => {}
        other => panic!("expected BadChecksum on a corrupted body, got {other:?}"),
    }
}

/// Enforces KISS-CONTRACT-6.1-0004 — on any rejection the reader returns a typed
/// decline and MUST NOT panic, read out of bounds, or allocate on an unchecked
/// declared length. TEETH: a reader that runs `Vec::with_capacity(declared_len)`
/// before bounds-checking (a huge declared length would OOM/abort instead of
/// declining); a reader that indexes a short buffer out of bounds.
#[test]
fn test_contract_rejection_is_typed_decline() {
    // A header declaring a 4-billion-byte body over a tiny buffer. A reader that
    // allocates `declared_len` before checking would attempt a ~4 GB allocation
    // and OOM/abort; a length-checking reader declines with BadLength. The CRC
    // and body are never touched with the bogus length.
    let attack = b"KISC kiss-contract 1 len=4000000000 crc32=00000000\n[section:1:identity]\n";
    assert_eq!(
        read_document(attack),
        Err(ContractDecline::BadLength { declared: 4_000_000_000, actual: 21 })
    );

    // A truncated body (declared length exceeds the actual body) declines, no OOB.
    let good = good_document();
    let truncated = &good[..good.len() - 3];
    match read_document(truncated) {
        Err(ContractDecline::BadLength { .. }) => {}
        other => panic!("expected BadLength on a truncated body, got {other:?}"),
    }

    // Inputs shorter than the 4-byte magic decline without an out-of-bounds read.
    assert_eq!(read_document(b""), Err(ContractDecline::NoMagic));
    assert_eq!(read_document(b"KI"), Err(ContractDecline::NoMagic));
    // Magic present but nothing after it (no LF) is a malformed header, not OOB.
    assert_eq!(read_document(b"KISC"), Err(ContractDecline::MalformedHeader));

    // A magic-only buffer with no LF is a malformed header, not a panic.
    assert_eq!(
        read_document(b"KISC kiss-contract 1 len=0 crc32=00000000"),
        Err(ContractDecline::MalformedHeader)
    );
}

/// KISS-CONTRACT-6.11-0010: a real value encodes as `<dtype>:<hex>`, the IEEE-754
/// bit pattern in big-endian lowercase hex, bit-exact for finite and special values.
#[test]
fn test_contract_text_real_encoding() {
    // f32: 8 hex digits, big-endian bit pattern.
    assert_eq!(Real::f32(1.0).render(), "f32:3f800000");
    assert_eq!(Real::f32(0.5).render(), "f32:3f000000");
    assert_eq!(Real::f32(-0.0).render(), "f32:80000000"); // signed zero preserved
    assert_eq!(Real::f32(f32::INFINITY).render(), "f32:7f800000");
    assert_eq!(Real::f32(f32::NEG_INFINITY).render(), "f32:ff800000");
    assert_eq!(Real::f32(f32::from_bits(0x7fc0_0000)).render(), "f32:7fc00000"); // qNaN

    // f64: 16 hex digits, big-endian bit pattern.
    assert_eq!(Real::f64(1.0).render(), "f64:3ff0000000000000");
    assert_eq!(Real::f64(0.1).render(), "f64:3fb999999999999a");
    assert_eq!(Real::f64(-0.0).render(), "f64:8000000000000000");
    assert_eq!(Real::f64(f64::NEG_INFINITY).render(), "f64:fff0000000000000");

    // The value is pinned by bits and round-trips exactly (umbrella §3.5), including a
    // NaN payload and a subnormal that no decimal spelling could carry unambiguously.
    for bits in [0x0000_0001u32, 0x7fc0_1234, 0x8000_0000, 0xffff_ffff] {
        let rendered = Real::F32(bits).render();
        let hex = rendered.strip_prefix("f32:").unwrap();
        assert_eq!(u32::from_str_radix(hex, 16).unwrap(), bits, "f32 bits round-trip");
        assert_eq!(hex.len(), 8, "f32 is exactly 8 hex digits");
    }

    // As a full field line through the §6.11-0001 `<key> = <value>` framing.
    assert_eq!(
        field_line("max_relative", &Value::Real(Real::f64(0.1))),
        "max_relative = f64:3fb999999999999a\n"
    );
}
