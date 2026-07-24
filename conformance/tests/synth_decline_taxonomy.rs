//! KISS-Conform tests for KISS-Synth §6.6 (the never-panic obligation and the
//! provision decline taxonomy) plus the two level-2 availability "identity-only"
//! corollaries (§6.2-0007a, §6.7-0003a). Continues the #91 SYNTH coverage burndown.
//!
//! SYNTH mints NO wire shapes of its own: a provision request REUSES the Announce
//! contract-query request frame (tag `CYRQ`), a provision decline REUSES the
//! Announce decline response frame (tag `CDEC`) carrying the Announce decline-code
//! enum, and level-2 availability REUSES the Announce availability record. So most
//! of these tests EXERCISE the real `conformance/src/announce.rs` codec
//! (`decode_query_request`, `DeclineResponse::encode`, `AvailabilityList`/
//! `decode_availability_list`, the `AnnounceDecline` typed-decline enum, the
//! `decline_code` constants) and assert those bytes/behaviors satisfy the cited
//! SYNTH clause. Two clauses (§6.6-0002a, §6.6-0003f) are DOC-LINTs: they extract
//! SYNTH's own §6.6 tables from `spec/synth.md` at runtime and set-compare against
//! the crate enum. Cross-standard constructs (the reused Announce enum) are named
//! only by their Rust symbol / constant value, never by an Announce clause-id, so
//! kiss_trace does not reverse-bind or over-credit Announce.
//!
//! Bound here: §6.6-0001, §6.6-0001a, §6.6-0001b, §6.6-0002, §6.6-0002a, §6.6-0003b,
//! §6.6-0003f, §6.6-0005, §6.2-0007a, §6.7-0003a (10 clauses). The provider/build-
//! engine/consumer-classifier clauses (unknown-key / cannot-provision / query-not-
//! supported / version-unsupported / unknown-revision mapping, bounded latency,
//! unknown-decline handling, build-failure) genuinely need a provider, a JIT build
//! engine, or a CDEC decode path that this crate does not have, and are SKIPPED (see
//! the PR body); a tautology to claim them would be worse than an honest gap.

use kiss_conformance::announce::*;
use kiss_conformance::assert_golden;
use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a phrase. `spec/` lives one
// directory above the conformance crate's CARGO_MANIFEST_DIR (the established
// pattern from emit_partition.rs). These helpers take the clause id as a PARAMETER
// so no foreign clause-id is baked into the harness source.
// ---------------------------------------------------------------------------

/// Read `spec/synth.md` from the repo root (one level above the crate).
fn read_synth_md() -> String {
    let path = format!("{}/../spec/synth.md", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read spec/synth.md at `{path}`: {e}"))
}

/// The body text of one clause: from its `**KISS-…**` bold anchor up to the next
/// clause definition or the next markdown heading.
fn clause_block<'a>(md: &'a str, id: &str) -> &'a str {
    let anchor = format!("**{id}**");
    let start = md
        .find(&anchor)
        .unwrap_or_else(|| panic!("clause `{id}` is not defined in spec/synth.md"));
    let rest = &md[start..];
    let mut end = rest.len();
    for pat in ["\n- **KISS-", "\n#"] {
        if let Some(i) = rest.get(1..).and_then(|r| r.find(pat)) {
            end = end.min(i + 1);
        }
    }
    &rest[..end]
}

/// Parse a markdown `| \`0x……\` | \`NAME\` | … |` value→NAME table anchored at the
/// first line equal to `header`. Only rows whose NAME cell is a backtick-wrapped
/// SCREAMING_SNAKE identifier are collected (so the `reserved`/`0x00000000` row is
/// skipped). Returns {code_value → NAME}.
fn parse_value_name_table(md: &str, header: &str) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    let start = md
        .find(header)
        .unwrap_or_else(|| panic!("table header `{header}` not found in spec/synth.md"));
    for line in md[start + header.len()..].lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            if !out.is_empty() {
                break; // first non-table line after the rows ends the table
            }
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 2 {
            continue;
        }
        let val = cells[0].trim_matches('`').trim();
        let name = cells[1].trim_matches('`').trim();
        if val.starts_with("---") {
            continue; // the header/body separator row
        }
        if let Some(hex) = val.strip_prefix("0x") {
            if let Ok(v) = u32::from_str_radix(hex, 16) {
                if is_screaming_snake(name) {
                    out.insert(v, name.to_string());
                }
            }
        }
    }
    out
}

/// Every backtick-wrapped SCREAMING_SNAKE identifier inside `block`.
fn backtick_screaming_names(block: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut i = 0;
    while let Some(rel) = block[i..].find('`') {
        let open = i + rel;
        if let Some(clen) = block[open + 1..].find('`') {
            let tok = &block[open + 1..open + 1 + clen];
            if is_screaming_snake(tok) {
                out.insert(tok.to_string());
            }
            i = open + 1 + clen + 1;
        } else {
            break;
        }
    }
    out
}

/// True for a non-empty `[A-Z0-9_]+` token that contains at least one letter — the
/// shape of a decline-code enum name (excludes lowercase words like `reserved` and
/// lowercase test names like `test_synth_…`).
fn is_screaming_snake(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && s.chars().any(|c| c.is_ascii_uppercase())
}

/// Every `0x……` hexadecimal literal in `s`, parsed as `u32`, in source order.
fn parse_hex_u32_tokens(s: &str) -> Vec<u32> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'0' && (b[i + 1] | 0x20) == b'x' {
            let mut j = i + 2;
            while j < b.len() && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            if j > i + 2 {
                if let Ok(v) = u32::from_str_radix(&s[i + 2..j], 16) {
                    out.push(v);
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// The six core decline codes, taken from the REAL crate `decline_code` constants
/// (values from the crate, so a value drift is caught) paired with their names.
fn core_decline_codes() -> [(u32, &'static str); 6] {
    [
        (decline_code::UNKNOWN_STRUCTURE_KEY, "UNKNOWN_STRUCTURE_KEY"),
        (decline_code::CANNOT_PROVISION, "CANNOT_PROVISION"),
        (decline_code::MALFORMED_REQUEST, "MALFORMED_REQUEST"),
        (decline_code::QUERY_NOT_SUPPORTED, "QUERY_NOT_SUPPORTED"),
        (decline_code::VERSION_UNSUPPORTED, "VERSION_UNSUPPORTED"),
        (decline_code::UNKNOWN_REVISION, "UNKNOWN_REVISION"),
    ]
}

/// A reference identity: key `AA BB CC`, revision pinned to 32×`0x11`.
fn identity_present() -> Identity {
    Identity { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: Some([0x11; 32]) }
}

// ---- §6.6 The never-panic obligation ----------------------------------------

/// KISS-SYNTH-6.6-0001 — every failure on the provision path (unknown cell, build
/// failure, unsupported version, missing revision, malformed request, ...) is
/// answered with a TYPED decline.
/// TEETH: a battery of malformed `CYRQ` frames (bad tag, bad `revision_present`,
/// `structure_key` length 0 and 4097, over-declared length) each returns
/// `Err(AnnounceDecline::<specific variant>)` from the REAL decoder — a typed value,
/// never `Ok` and never a panic. A drift that returned `Ok` on a malformed frame, or
/// erased the variant into a bare `bool`, fails an exact-variant assertion.
#[test]
fn test_synth_failure_is_typed_decline() {
    let cyrq = CYRQ.to_le_bytes().to_vec();
    let frame = |tail: &[u8]| {
        let mut v = cyrq.clone();
        v.extend_from_slice(tail);
        v
    };
    let cases: Vec<(Vec<u8>, AnnounceDecline)> = vec![
        // bad tag (not CYRQ): four zero bytes then a body.
        (
            vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x7A, 0x00],
            AnnounceDecline::BadTag { region: "cyrq", got: 0 },
        ),
        // revision_present == 2 (neither 0 nor 1).
        (
            frame(&[0x01, 0x00, 0x00, 0x00, 0x7A, 0x02]),
            AnnounceDecline::BadRevisionPresent { got: 2 },
        ),
        // structure_key length == 0 (below the [1,4096] floor).
        (
            frame(&[0x00, 0x00, 0x00, 0x00, 0x00]),
            AnnounceDecline::StructureKeyLenOutOfRange { got: 0 },
        ),
        // structure_key length == 4097 (above the ceiling; 0x1001 LE).
        (
            frame(&[0x01, 0x10, 0x00, 0x00, 0x00]),
            AnnounceDecline::StructureKeyLenOutOfRange { got: 4097 },
        ),
        // declared length (8) exceeds the remaining input (2 key bytes).
        (
            frame(&[0x08, 0x00, 0x00, 0x00, 0xAA, 0xBB]),
            AnnounceDecline::Truncated { region: "structure_key" },
        ),
    ];
    for (bytes, expected) in &cases {
        let got = decode_query_request(bytes);
        assert!(got.is_err(), "malformed frame wrongly accepted as Ok: {bytes:?}");
        assert_eq!(
            &got.unwrap_err(),
            expected,
            "KISS-SYNTH-6.6-0001: malformed frame must yield its specific typed decline: {bytes:?}"
        );
    }
}

/// KISS-SYNTH-6.6-0001a — a provider handling any provision request MUST NOT panic,
/// abort, or crash. This obligation is fuzz-testable and owned by this sub-standard.
/// TEETH: a mini-fuzz over the REAL decoders — every truncation prefix `0..=len` of
/// a valid `CYRQ` request and of a valid availability list, an all-`0xFF` buffer, an
/// empty buffer, and a frame declaring the IN-RANGE max `structure_key` length
/// (4096) with a short tail — each call must return a `Result` without panicking.
/// A missing bounds-check (e.g. slicing `bytes[off..off+key_len]` before validating)
/// would panic/OOB and fail; the concluding assertion pins the check-before-slice
/// behavior on the in-range-max frame. This is the real panic-freedom property, not
/// `assert!(true)`.
#[test]
fn test_synth_never_panics() {
    // Fuzz every truncation prefix of a valid request and a valid availability list.
    let request = QueryRequest { identity: identity_present() }.encode();
    let list = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: vec![1, 2, 3], revision_hash: [7; 32] }],
    }
    .encode();
    for n in 0..=request.len() {
        let _ = decode_query_request(&request[..n]);
        let _ = decode_availability_list(&request[..n]);
    }
    for n in 0..=list.len() {
        let _ = decode_availability_list(&list[..n]);
        let _ = decode_query_request(&list[..n]);
    }
    // Adversarial buffers: all-ones and empty, on both decoders.
    let all_ff = vec![0xFFu8; 96];
    let _ = decode_query_request(&all_ff);
    let _ = decode_availability_list(&all_ff);
    let _ = decode_query_request(&[]);
    let _ = decode_availability_list(&[]);

    // A frame declaring the IN-RANGE MAX key length (4096) with only a short tail:
    // the check-before-slice guard must DECLINE, not OOB/panic.
    let mut maxkey = CYRQ.to_le_bytes().to_vec();
    maxkey.extend_from_slice(&4096u32.to_le_bytes()); // key_len = 4096 (in range)
    maxkey.extend_from_slice(&[0xAA, 0xBB, 0xCC]); // only 3 bytes follow
    assert_eq!(
        decode_query_request(&maxkey),
        Err(AnnounceDecline::Truncated { region: "structure_key" }),
        "KISS-SYNTH-6.6-0001a: an in-range-max declared length must decline (check-before-slice), \
         never panic or read out of bounds"
    );
}

/// KISS-SYNTH-6.6-0001b — a provider MUST NOT read beyond the bytes declared by the
/// request framing (the fixed `CYRQ` fields, the declared `structure_key` length,
/// and the 32-byte `revision_hash` when `revision_present == 1`).
/// TEETH: (a) a valid request followed by trailing POISON bytes decodes to EXACTLY
/// the clean identity — the poison is not folded into `structure_key` or
/// `revision_hash`, proving the reader stops at the declared framing; (b) an
/// over-declared `structure_key` length with a short buffer yields
/// `Truncated{region:"structure_key"}` — bounds-checked BEFORE the slice. A decoder
/// that consumed to end-of-buffer, or sliced before checking, fails one of the two.
#[test]
fn test_synth_no_oob_read() {
    // (a) valid request + 6 trailing poison bytes → decodes to exactly the identity.
    let ident = identity_present();
    let mut framed = QueryRequest { identity: ident.clone() }.encode();
    framed.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0xFF, 0x00]); // poison tail
    let decoded = decode_query_request(&framed)
        .expect("KISS-SYNTH-6.6-0001b: a well-framed request decodes despite trailing bytes")
        .identity;
    assert_eq!(decoded, ident, "trailing bytes MUST NOT fold into the decoded identity");
    assert_eq!(decoded.structure_key, vec![0xAA, 0xBB, 0xCC], "structure_key stops at its declared length");
    assert_eq!(decoded.revision_hash, Some([0x11; 32]), "revision_hash is exactly 32 declared bytes, no poison");

    // (b) over-declared structure_key length with a short buffer → checked before slice.
    let mut over = CYRQ.to_le_bytes().to_vec();
    over.extend_from_slice(&64u32.to_le_bytes()); // declares 64 key bytes
    over.extend_from_slice(&[0x01, 0x02, 0x03]); // supplies only 3
    assert_eq!(
        decode_query_request(&over),
        Err(AnnounceDecline::Truncated { region: "structure_key" }),
        "over-declared length is bounds-checked before the slice — no read past the declared bytes"
    );
}

// ---- §6.6 The provision decline taxonomy ------------------------------------

/// KISS-SYNTH-6.6-0002 — a provision decline uses the pinned decline-code enum,
/// reused verbatim as little-endian `u32` values (the six core codes 1..6).
/// TEETH: for each of the six core codes, build a `DeclineResponse` with the REAL
/// codec and assert the `CDEC` frame's trailing four bytes equal `code.to_le_bytes()`
/// (and NOT `code.to_be_bytes()`), the tag is `CDEC`, and the frame width proves the
/// code is a `u32` (4 bytes), not a `u16`. Catches a big-endian `decline_code` write,
/// a `u16` width, or a wrong constant value.
#[test]
fn test_synth_decline_code_enum() {
    // The six core codes are the contiguous little-endian u32 values 1..6.
    let values: Vec<u32> = core_decline_codes().iter().map(|(v, _)| *v).collect();
    assert_eq!(values, vec![1, 2, 3, 4, 5, 6], "core decline_code constants are contiguous 1..6");

    for &(code, name) in &core_decline_codes() {
        // one-byte key + no revision => identity block is key_len(4)+key(1)+flag(1) = 6.
        let resp = DeclineResponse {
            identity: Identity { structure_key: vec![0x41], revision_hash: None },
            decline_code: code,
        };
        let b = resp.encode();
        assert_eq!(&b[0..4], &[0x43, 0x44, 0x45, 0x43], "KISS-SYNTH-6.6-0002: CDEC tag (wire 43 44 45 43) for {name}");
        assert_eq!(&b[0..4], &CDEC.to_le_bytes(), "the decline frame tag is the CDEC u32-LE constant");
        let tail: [u8; 4] = b[b.len() - 4..].try_into().unwrap();
        assert_eq!(tail, code.to_le_bytes(), "decline_code {name} is written little-endian");
        assert_ne!(tail, code.to_be_bytes(), "a big-endian decline_code write is a WRONG impl for {name}");
        assert_eq!(b.len(), 4 + 6 + 4, "decline_code occupies 4 bytes (u32), not 2 (u16), for {name}");
    }
}

/// KISS-SYNTH-6.6-0002a — a provider MUST NOT emit a core code (0x1..0x6) with a
/// meaning other than the one pinned in §6.6-0002.
/// TEETH (DOC-LINT): parse the pinned §6.6-0002 (value → NAME) table out of
/// spec/synth.md (anchored on its unique `| Value | Name | Meaning |` header, so no
/// foreign clause-id is cited) and set-compare it, for equality, against the value →
/// NAME map built from the REAL crate `decline_code` constants (values taken from
/// the crate). A rename in the spec table, or a value reassignment of a crate
/// constant, desyncs the two maps and fails. This is a cross-check of SYNTH's own
/// doc against the crate enum by value/name — not a set compared to a copy of itself.
#[test]
fn test_synth_decline_core_meaning_fixed() {
    let md = read_synth_md();
    let spec_map = parse_value_name_table(&md, "| Value | Name | Meaning |");
    assert_eq!(spec_map.len(), 6, "the §6.6-0002 table pins exactly six core codes");
    assert_eq!(
        spec_map.keys().copied().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6],
        "the pinned core codes are contiguous 1..6"
    );

    // crate side: value → NAME, value taken from the REAL crate constants.
    let crate_map: BTreeMap<u32, String> =
        core_decline_codes().iter().map(|&(v, n)| (v, n.to_string())).collect();

    assert_eq!(
        spec_map, crate_map,
        "KISS-SYNTH-6.6-0002a: the §6.6-0002 decline-code table and the crate decline_code \
         constants disagree — a rename or value reassignment in either desyncs the fixed meanings"
    );
}

/// KISS-SYNTH-6.6-0003b — a provider MUST map a malformed request frame to
/// `MALFORMED_REQUEST`.
/// TEETH: feed the REAL decoder the four request-framing malformations; each yields
/// a distinct typed variant (`BadRevisionPresent`, `StructureKeyLenOutOfRange` ×2 at
/// distinct `got` values, `Truncated`). A test-local categorizer maps each
/// request-framing variant → `MALFORMED_REQUEST` (0x3) and PANICS on any
/// non-request-framing decline, so a mis-typed malformation or a wrong-code mapping
/// fails. The categorizer is over the decoder's OUTPUT — the codec itself is the
/// real `decode_query_request`, not a re-implementation.
#[test]
fn test_synth_map_malformed_request() {
    // Maps a request-framing decline to the code a provider emits for it; PANICS on
    // any decline that is not a request-framing malformation.
    fn request_framing_to_code(d: &AnnounceDecline) -> u32 {
        match d {
            AnnounceDecline::BadRevisionPresent { .. }
            | AnnounceDecline::StructureKeyLenOutOfRange { .. }
            | AnnounceDecline::Truncated { .. }
            | AnnounceDecline::BadTag { .. } => decline_code::MALFORMED_REQUEST,
            other => panic!("not a request-framing malformation: {other:?}"),
        }
    }

    let cyrq = CYRQ.to_le_bytes().to_vec();
    let frame = |tail: &[u8]| {
        let mut v = cyrq.clone();
        v.extend_from_slice(tail);
        v
    };
    let d_rp = decode_query_request(&frame(&[0x01, 0x00, 0x00, 0x00, 0x7A, 0x02])).unwrap_err();
    let d_zero = decode_query_request(&frame(&[0x00, 0x00, 0x00, 0x00, 0x00])).unwrap_err();
    let d_over = decode_query_request(&frame(&[0x01, 0x10, 0x00, 0x00, 0x00])).unwrap_err();
    let d_trunc = decode_query_request(&frame(&[0x08, 0x00, 0x00, 0x00, 0xAA, 0xBB])).unwrap_err();

    // distinct typed variants — not all collapsed into one.
    assert_eq!(d_rp, AnnounceDecline::BadRevisionPresent { got: 2 });
    assert_eq!(d_zero, AnnounceDecline::StructureKeyLenOutOfRange { got: 0 });
    assert_eq!(d_over, AnnounceDecline::StructureKeyLenOutOfRange { got: 4097 });
    assert_eq!(d_trunc, AnnounceDecline::Truncated { region: "structure_key" });

    // every request-framing malformation maps to MALFORMED_REQUEST (0x3).
    for d in [&d_rp, &d_zero, &d_over, &d_trunc] {
        assert_eq!(
            request_framing_to_code(d),
            decline_code::MALFORMED_REQUEST,
            "KISS-SYNTH-6.6-0003b: a malformed request frame maps to MALFORMED_REQUEST"
        );
        assert_eq!(request_framing_to_code(d), 0x0000_0003, "MALFORMED_REQUEST is the little-endian u32 0x3");
    }
}

/// KISS-SYNTH-6.6-0003f — the provision decline taxonomy MUST be complete over
/// EXACTLY the closed set of six failure kinds; the empty-profile-intersection
/// handshake failure is owned by KISS-Announce and is NOT a member.
/// TEETH (DOC-LINT): extract the six failure-kind code NAMES enumerated in the
/// §6.6-0003f clause body from spec/synth.md and assert the set equals EXACTLY the
/// six crate core `decline_code` constant names (count == 6, values contiguous 1..6);
/// and assert the handshake failure — referenced by the crate variant
/// `AnnounceDecline::NoMutualProfile` (a symbol, never an Announce clause-id) — is
/// NOT a member of the taxonomy set. Adding a 7th core kind, dropping one, or leaking
/// the handshake failure into the set all fail.
#[test]
fn test_synth_decline_taxonomy_complete() {
    let md = read_synth_md();
    let spec_names = backtick_screaming_names(clause_block(&md, "KISS-SYNTH-6.6-0003f"));
    let crate_names: BTreeSet<String> =
        core_decline_codes().iter().map(|&(_, n)| n.to_string()).collect();

    assert_eq!(crate_names.len(), 6, "there are exactly six crate core decline codes");
    assert_eq!(
        spec_names, crate_names,
        "KISS-SYNTH-6.6-0003f: the §6.6-0003f taxonomy enumeration and the crate core decline codes \
         must be the same closed set of six (a 7th kind, a dropped kind, or a leaked failure fails)"
    );

    // values contiguous 1..6 (the closed set has no gap / no extension code).
    let mut values: Vec<u32> = core_decline_codes().iter().map(|&(v, _)| v).collect();
    values.sort_unstable();
    assert_eq!(values, vec![1, 2, 3, 4, 5, 6], "the six core kinds are the contiguous values 1..6");

    // The empty-profile-intersection handshake failure is the crate variant
    // NoMutualProfile — a DISTINCT typed value, referenced by symbol. It carries no
    // decline_code and its name must NOT appear in the §6.6 provision taxonomy.
    let _handshake = AnnounceDecline::NoMutualProfile; // compile-fails if the variant is removed/renamed
    assert!(
        !spec_names.contains("NO_MUTUAL_PROFILE"),
        "the handshake (empty-profile-intersection) failure must NOT leak into the §6.6 taxonomy"
    );
}

/// KISS-SYNTH-6.6-0005 — a producer MUST confine any non-core decline code to the
/// experimental range `[0x40000000, 0x80000000)` or the vendor range
/// `[0x80000000, 2^32)`.
/// TEETH (partial-proxy, structural disjointness): parse the two extension-range
/// floors pinned in the §6.6-0005 clause body from spec/synth.md, assert
/// experimental-floor < vendor-floor < 2^32, and assert every crate core
/// `decline_code` is strictly BELOW the parsed experimental floor — so the core enum
/// is disjoint from BOTH extension ranges. A drift assigning a core code into an
/// extension range, or the spec moving the experimental floor below a core value,
/// fails. CAVEAT: this does not exercise a producer confining a NON-core code (no
/// producer/registry exists in the crate); it is a disjointness invariant only.
#[test]
fn test_synth_decline_ranges() {
    let md = read_synth_md();
    let mut floors = parse_hex_u32_tokens(clause_block(&md, "KISS-SYNTH-6.6-0005"));
    floors.sort_unstable();
    floors.dedup();
    assert_eq!(
        floors,
        vec![0x4000_0000u32, 0x8000_0000u32],
        "KISS-SYNTH-6.6-0005: the two extension-range floors pinned in the clause"
    );
    let (exp_floor, vendor_floor) = (floors[0], floors[1]);
    assert!(exp_floor < vendor_floor, "the experimental range sits strictly below the vendor range");
    assert!((vendor_floor as u64) < (1u64 << 32), "the vendor range top is 2^32");

    // every crate CORE decline code is strictly below the experimental floor.
    for &(code, name) in &core_decline_codes() {
        assert_ne!(code, 0, "0x00000000 is reserved and MUST NOT be a core code ({name})");
        assert!(
            code < exp_floor,
            "core decline code {name} ({code:#010x}) must be disjoint from the extension ranges"
        );
    }
}

// ---- §6.2 / §6.7 level-2 availability: identity only ------------------------

/// KISS-SYNTH-6.2-0007a — a provider MUST NOT carry per-kernel capability, usage,
/// guarantee, or semantics in the level-2 availability record — only the
/// `(structure_key, revision_hash)` identity.
/// TEETH: a one-record availability list serialized by the REAL codec has a
/// per-record region (after the 12-byte SAVL header) that is EXACTLY
/// `[u32-LE key_len][key bytes][32-byte revision_hash]` and nothing else, pinned as
/// an exact-byte golden. Any added per-kernel capability/usage/guarantee/semantics
/// field would lengthen or alter these bytes and fail the golden.
#[test]
fn test_synth_availability_no_capability() {
    let list = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: [0x11; 32] }],
    };
    let b = list.encode();

    // header = SAVL tag(4) + list_version(1) + 3 MBZ + record_count(4) = 12 bytes.
    assert_golden("KISS-SYNTH-6.2-0007a", "savl_header", &b[..12], "53 41 56 4C 01 00 00 00 01 00 00 00");

    // per-record region: key_len(4, LE) + key(3) + revision_hash(32), nothing else.
    let record_golden = concat!(
        "03 00 00 00 ",
        "AA BB CC ",
        "11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 ",
        "11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 11",
    );
    assert_golden("KISS-SYNTH-6.2-0007a", "savl_record_identity_only", &b[12..], record_golden);
    assert_eq!(b.len() - 12, 4 + 3 + 32, "the record carries ONLY (structure_key, revision_hash)");
    assert_eq!(b.len(), 12 + 39, "no room for any per-kernel capability/usage/guarantee/semantics field");
}

/// KISS-SYNTH-6.7-0003a — the level-2 availability record MUST NOT carry any
/// per-kernel capability, usage/ABI, dispatch, guarantee, or semantics field.
/// TEETH: encode a MULTI-record list, decode it back through the REAL codec, and
/// assert (i) round-trip equality and (ii) the total wire length is FULLY accounted
/// for by the 12-byte header + Σ over records of (4 + key_len + 32) — no leftover /
/// interleaved bytes for a capability region — and each decoded record destructures
/// to EXACTLY `{structure_key, revision_hash}` (an exhaustive pattern that fails to
/// compile if a field is added). Distinct from §6.2-0007a (list length-accounting +
/// decode over multiple records, vs a single-record encode golden).
#[test]
fn test_synth_availability_no_capability_fields() {
    let records = vec![
        AvailabilityRecord { structure_key: vec![0x01], revision_hash: [0x10; 32] },
        AvailabilityRecord { structure_key: vec![0xAA, 0xBB, 0xCC, 0xDD], revision_hash: [0x20; 32] },
        AvailabilityRecord { structure_key: (0u16..=63).map(|x| x as u8).collect(), revision_hash: [0x33; 32] },
    ];
    let list = AvailabilityList { list_version: 1, records: records.clone() };
    let wire = list.encode();

    // (i) round-trip equality through the REAL codec.
    let decoded = decode_availability_list(&wire)
        .expect("KISS-SYNTH-6.7-0003a: a multi-record availability list round-trips");
    assert_eq!(decoded, list, "decode∘encode is the identity on the availability list");

    // (ii) total wire length is fully accounted for by header + Σ(4 + key_len + 32).
    let accounted: usize =
        12 + records.iter().map(|r| 4 + r.structure_key.len() + 32).sum::<usize>();
    assert_eq!(
        wire.len(),
        accounted,
        "no leftover/interleaved bytes: a capability region would desync the length accounting"
    );

    // each decoded record exposes ONLY {structure_key, revision_hash} — an exhaustive
    // destructure (no `..`) is a compile-time guarantee the record has no other field.
    for (got, want) in decoded.records.iter().zip(records.iter()) {
        let AvailabilityRecord { structure_key, revision_hash } = got;
        assert_eq!(structure_key, &want.structure_key, "structure_key survives");
        assert_eq!(revision_hash, &want.revision_hash, "revision_hash survives");
    }
}
