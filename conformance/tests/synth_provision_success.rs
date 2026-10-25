// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform tests for the KISS-Synth provision-success frame `PRSP` (Synth §6.4-0001b),
//! KISS-Synth's one own wire frame, driven through the reference codec in
//! `conformance/src/synth.rs`. The request (`CYRQ`) and decline (`CDEC`) frames are reused
//! KISS-Announce frames and are covered in `synth_request_decline.rs`.
//!
//! Each golden below is written out by hand from the clause text (tag, identity block,
//! `artifact_format_tag`, `u64` artifact length, artifact, `u32` contract length, contract),
//! never produced by the encoder under test.
//!
//! Clauses bound: KISS-SYNTH-6.4-0001, -0001b, -0001c, -0001d, 6.2-0003, -0003a, -0003b, -0004.
//! NOT bound: 6.4-0001a (an absent artifact block is a truncation decline, but "contract
//! without artifact" also names a provider's behaviour), 6.3-0006/-0006a and 6.2-0006 (the
//! frame half is enforced here, the provider half and the contract Identity half are not).

use kiss_conformance::announce::*;
use kiss_conformance::assert_golden;
use kiss_conformance::contract::{read_document, well_formed_document, ContractDecline};
use kiss_conformance::synth::*;

/// Echoed identity block: key `AA BB CC`, op_identity `add`, `revision_present = 1`, hash `11`*32.
const IDENTITY_GOLDEN: &str = concat!(
    "03 00 00 00 AA BB CC ",
    "03 00 00 00 61 64 64 ",
    "01 ",
    "11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 ",
);

fn identity() -> Identity {
    Identity { op_identity: b"add".to_vec(), structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: Some([0x11; 32]), correlation: None }
}

fn success() -> ProvisionSuccess {
    ProvisionSuccess {
        identity: identity(),
        artifact_format_tag: *b"CUBN",
        artifact: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
        contract: b"KISS-CONTRACT-STUB".to_vec(),
    }
}

/// The same frame, spelled out left to right.
fn success_golden() -> String {
    format!(
        "50 52 53 50 {IDENTITY_GOLDEN}43 55 42 4E 05 00 00 00 00 00 00 00 DE AD BE EF 01 \
         12 00 00 00 4B 49 53 53 2D 43 4F 4E 54 52 41 43 54 2D 53 54 55 42"
    )
}

/// KISS-SYNTH-6.4-0001b — a provision success is the `PRSP` frame: the 4-byte tag
/// (`50 52 53 50`), the echoed identity block with `revision_present == 1`, a 4-byte
/// `artifact_format_tag`, a little-endian `u64` artifact length, the artifact, a little-endian
/// `u32` contract length, then the contract.
/// TEETH: the encoder's bytes equal the hand-written golden; the decoder returns exactly the
/// frame that was encoded; every strict prefix of the frame is a typed decline (never a panic or
/// slice error); a hostile `u64::MAX` artifact length declines rather than allocating.
/// MUTATION: a big-endian length or tag, a swapped field, or a dropped `revision_present` byte
/// changes the golden bytes.
#[test]
fn test_synth_provision_success_prsp_frame() {
    let bytes = success().encode().unwrap();
    assert_golden("KISS-SYNTH-6.4-0001b", "prsp_add_hit", &bytes, &success_golden());
    assert_eq!(&bytes[0..4], &[0x50, 0x52, 0x53, 0x50], "PRSP wire bytes");
    assert_eq!(PRSP.to_le_bytes(), [0x50, 0x52, 0x53, 0x50]);
    assert_eq!(decode_provision_success(&bytes), Ok(success()));

    for n in 0..bytes.len() {
        assert!(decode_provision_success(&bytes[..n]).is_err(), "prefix of {n} bytes must decline");
    }

    // artifact_len = u64::MAX, placed right after the 4-byte format tag.
    let len_off = 4 + parse_len(IDENTITY_GOLDEN) + 4;
    let mut hostile = bytes.clone();
    hostile[len_off..len_off + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(decode_provision_success(&hostile), Err(SynthDecline::Truncated { region: "artifact" }));
}

fn parse_len(hex: &str) -> usize {
    hex.split_whitespace().count()
}

/// KISS-SYNTH-6.4-0001 — the artifact is the FIRST payload element: it precedes the contract
/// in wire order.
/// TEETH: in the golden the artifact bytes sit before the `u32` contract length and the contract
/// is the tail; a frame with the two blocks swapped does NOT decode to the intended pair.
#[test]
fn test_synth_artifact_is_first_element() {
    let s = success();
    let bytes = s.encode().unwrap();
    let art_at = bytes.windows(s.artifact.len()).position(|w| w == s.artifact.as_slice()).unwrap();
    let ctr_at = bytes.windows(s.contract.len()).position(|w| w == s.contract.as_slice()).unwrap();
    assert!(art_at < ctr_at, "artifact precedes contract in wire order");
    assert_eq!(&bytes[bytes.len() - s.contract.len()..], s.contract.as_slice(), "contract is the tail");

    // Swapped: contract block (u32 len + bytes) placed where the artifact block (u64 len + bytes) goes.
    let mut swapped = Vec::new();
    swapped.extend_from_slice(&PRSP.to_le_bytes());
    swapped.extend_from_slice(&parse_hex_bytes(IDENTITY_GOLDEN));
    swapped.extend_from_slice(b"CUBN");
    swapped.extend_from_slice(&(s.contract.len() as u32).to_le_bytes());
    swapped.extend_from_slice(&s.contract);
    swapped.extend_from_slice(&(s.artifact.len() as u64).to_le_bytes());
    swapped.extend_from_slice(&s.artifact);
    assert_ne!(decode_provision_success(&swapped), Ok(s), "contract-first is not the pinned order");
}

fn parse_hex_bytes(hex: &str) -> Vec<u8> {
    hex.split_whitespace().map(|h| u8::from_str_radix(h, 16).unwrap()).collect()
}

/// KISS-SYNTH-6.4-0001c — a provider MUST NOT return a bare `CRSP` contract-only response as a
/// provision success; the distinguishing byte condition is the leading 4-byte tag, `PRSP`
/// (`50 52 53 50`) versus `CRSP` (`43 52 53 50`).
/// TEETH: over the SAME identity and contract, the two frames differ in the leading tag; the
/// `PRSP` decoder declines the `CRSP` frame as `ContractOnlyResponse`, and also declines a `PRSP`
/// frame whose tag was rewritten to `CRSP`; an unrelated tag is `BadTag`.
#[test]
fn test_synth_success_distinct_from_contract_only() {
    let s = success();
    let prsp = s.encode().unwrap();
    let crsp = s.to_contract_response().encode();
    assert_eq!(&prsp[0..4], &[0x50, 0x52, 0x53, 0x50]);
    assert_eq!(&crsp[0..4], &[0x43, 0x52, 0x53, 0x50]);
    assert_eq!(decode_provision_success(&crsp), Err(SynthDecline::ContractOnlyResponse));

    let mut retagged = prsp.clone();
    retagged[0..4].copy_from_slice(&CRSP.to_le_bytes());
    assert_eq!(decode_provision_success(&retagged), Err(SynthDecline::ContractOnlyResponse));

    let mut other = prsp.clone();
    other[0..4].copy_from_slice(b"ZZZZ");
    assert_eq!(decode_provision_success(&other), Err(SynthDecline::BadTag { got: u32::from_le_bytes(*b"ZZZZ") }));
}

/// KISS-SYNTH-6.4-0001d — `artifact_format_tag` is a 4-byte tag, matched byte-for-byte and
/// otherwise opaque, and MUST NOT be the reserved `00 00 00 00`.
/// TEETH: the reserved value is refused on both encode and decode; assorted other tags (ASCII,
/// non-ASCII, a lone high or low byte) round-trip byte-for-byte, so the codec neither interprets
/// nor normalizes the tag.
#[test]
fn test_synth_artifact_format_tag() {
    let mut zero = success();
    zero.artifact_format_tag = [0; 4];
    assert_eq!(zero.encode(), Err(SynthDecline::ZeroArtifactFormatTag));

    let mut bytes = success().encode().unwrap();
    let tag_off = 4 + parse_len(IDENTITY_GOLDEN);
    bytes[tag_off..tag_off + 4].copy_from_slice(&[0; 4]);
    assert_eq!(decode_provision_success(&bytes), Err(SynthDecline::ZeroArtifactFormatTag));

    for tag in [*b"CUBN", *b"SPRV", [0xFF; 4], [0, 0, 0, 1], [1, 0, 0, 0], [0x80, 0x00, 0xC3, 0x28]] {
        let mut s = success();
        s.artifact_format_tag = tag;
        let enc = s.encode().unwrap();
        assert_eq!(&enc[tag_off..tag_off + 4], &tag, "tag written verbatim");
        assert_eq!(decode_provision_success(&enc).unwrap().artifact_format_tag, tag);
    }
}

/// KISS-SYNTH-6.2-0003 — the contract is a little-endian `u32` byte-length followed by that many
/// bytes, and those bytes are byte-identical to the payload the `CRSP` frame would carry.
/// TEETH: the four bytes before the contract are its length, little-endian; the `CRSP` built from
/// the same success carries the identical payload bytes; a length larger than what remains
/// declines, and a length one short yields a contract one byte short rather than reading on.
#[test]
fn test_synth_contract_length_delimited() {
    let s = success();
    let bytes = s.encode().unwrap();
    let n = s.contract.len();
    let len_off = bytes.len() - n - 4;
    assert_eq!(&bytes[len_off..len_off + 4], &(n as u32).to_le_bytes(), "u32 LE contract length");

    let crsp = s.to_contract_response().encode();
    assert_eq!(&crsp[crsp.len() - n..], &bytes[bytes.len() - n..], "PRSP contract bytes == CRSP payload bytes");
    assert_eq!(decode_provision_success(&bytes).unwrap().contract, s.contract);

    let mut long = bytes.clone();
    long[len_off..len_off + 4].copy_from_slice(&(n as u32 + 1).to_le_bytes());
    assert_eq!(decode_provision_success(&long), Err(SynthDecline::Truncated { region: "contract" }));

    let mut short = bytes.clone();
    short[len_off..len_off + 4].copy_from_slice(&(n as u32 - 1).to_le_bytes());
    assert_eq!(decode_provision_success(&short).unwrap().contract.len(), n - 1);
}

/// KISS-SYNTH-6.2-0003a — the contract is carried OPAQUELY; the provision path MUST NOT
/// reinterpret or re-encode its bytes.
/// TEETH: a contract of every byte value 0x00..=0xFF, followed by text that looks like a `PRSP`
/// tag, a section heading and CRLF line ends, comes back byte-identical and occupies the frame's
/// tail unchanged. An encoder that normalized line ends, validated UTF-8 or re-framed would alter it.
#[test]
fn test_synth_contract_opaque() {
    let mut contract: Vec<u8> = (0u8..=255).collect();
    contract.extend_from_slice(b"PRSP\r\n[section:0:identity]\r\n\x00\x00");
    let mut s = success();
    s.contract = contract.clone();
    let bytes = s.encode().unwrap();
    assert_eq!(&bytes[bytes.len() - contract.len()..], contract.as_slice(), "contract bytes verbatim in the frame");
    assert_eq!(decode_provision_success(&bytes).unwrap().contract, contract);
}

/// KISS-SYNTH-6.2-0003b — the provision path MUST NOT require the enclosing transport or the
/// Announce layer to parse the contract's internals.
/// TEETH: a frame whose contract is NOT a contract at all (and which the contract reader itself
/// declines) decodes fine at the frame level — the frame decoder never looks inside.
#[test]
fn test_synth_contract_not_parsed_by_host() {
    let mut s = success();
    s.contract = b"this is not a KISS-Contract document".to_vec();
    assert!(read_document(&s.contract).is_err(), "control: the contract reader rejects these bytes");
    let bytes = s.encode().unwrap();
    assert_eq!(decode_provision_success(&bytes).unwrap().contract, s.contract);
}

/// KISS-SYNTH-6.2-0004 — the returned contract's own inner framing (magic, header line, inner
/// length and checksum, headings) MUST fail loudly on malformation INDEPENDENTLY of the `PRSP`
/// contract-length prefix.
/// TEETH: (i) a well-formed `PRSP` wrapping a contract with one corrupted body byte decodes at the
/// frame level and is declined by the contract reader on its own checksum; (ii) the intact
/// contract in a `PRSP` whose prefix overstates its length is declined by the frame decoder while
/// the contract reader still accepts the contract. Neither layer's check substitutes for the other.
#[test]
fn test_synth_contract_inner_framing_independent() {
    let good = well_formed_document();
    assert!(read_document(&good).is_ok(), "control: the document reads cleanly");

    let mut corrupt = good.clone();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 0x01;
    let mut s = success();
    s.contract = corrupt;
    let frame = s.encode().unwrap();
    let inner = decode_provision_success(&frame).unwrap().contract;
    assert!(
        matches!(read_document(&inner), Err(ContractDecline::BadChecksum { .. }) | Err(ContractDecline::BadLength { .. })),
        "the inner framing declines on its own, got {:?}",
        read_document(&inner)
    );

    let mut s = success();
    s.contract = good.clone();
    let mut frame = s.encode().unwrap();
    let len_off = frame.len() - good.len() - 4;
    frame[len_off..len_off + 4].copy_from_slice(&(good.len() as u32 + 1).to_le_bytes());
    assert_eq!(decode_provision_success(&frame), Err(SynthDecline::Truncated { region: "contract" }));
    assert!(read_document(&good).is_ok(), "the intact contract is still accepted by its own reader");
}

/// Not a bound clause: the echoed identity of a success always carries `revision_present == 1`
/// and the 32-byte hash (§6.3-0006 / §6.3-0006a, frame half only). An unpinned identity is neither
/// encodable nor decodable here.
// harness-internal: the suite's own PRSP frame codec refuses an echo without a pinned revision; the Synth 6.3-0006/-0006a provider obligations are recorded unbacked (UNBACKED.tsv)
#[test]
fn test_synth_prsp_echo_requires_revision() {
    let mut s = success();
    s.identity.revision_hash = None;
    assert_eq!(s.encode(), Err(SynthDecline::RevisionNotPresent));

    // A frame built by hand with revision_present = 0 in the echo.
    let mut b = Vec::new();
    b.extend_from_slice(&PRSP.to_le_bytes());
    b.extend_from_slice(&[3, 0, 0, 0, 0xAA, 0xBB, 0xCC, 3, 0, 0, 0, b'a', b'd', b'd', 0]);
    b.extend_from_slice(b"CUBN");
    b.extend_from_slice(&0u64.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(decode_provision_success(&b), Err(SynthDecline::RevisionNotPresent));
}
