// SPDX-License-Identifier: MIT OR Apache-2.0
//! The trust-boundary clauses of R5.3 (umbrella §10.5 items 1, 3, 4, 5): length caps on the response
//! direction (KISS-SYNTH-6.4-0001e, KISS-CONTRACT-6.11-0003a), what `revision_hash` and the contract
//! checksum are NOT (KISS-ANNOUNCE-6.3-0008a, KISS-CONTRACT-6.11-0003b), and the reserved attestation bit
//! (KISS-ANNOUNCE-7.2-0011). The untrusted-code statement (KISS-SYNTH-6.4-0001f) binds a consumer's loader
//! and has no in-repo behaviour to witness; it is in `UNBACKED.tsv` as `untested`.

use kiss_conformance::announce::*;
use kiss_conformance::contract::*;
use kiss_conformance::synth::*;

fn success() -> ProvisionSuccess {
    ProvisionSuccess {
        identity: reference_identity(),
        artifact_format_tag: *b"CUBN",
        artifact: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
        contract: b"KISS-CONTRACT-STUB".to_vec(),
    }
}

/// Offsets of the two length fields in an encoded PRSP built from [`success`].
fn length_offsets(bytes: &[u8], s: &ProvisionSuccess) -> (usize, usize) {
    let contract_len_off = bytes.len() - s.contract.len() - 4;
    let artifact_len_off = contract_len_off - s.artifact.len() - 8;
    (artifact_len_off, contract_len_off)
}

// Backs: KISS-SYNTH-6.4-0001e — a declared artifact or contract length above its cap is a typed decline before any byte is read or allocated, whether or not that many bytes follow; a length at the cap is not refused for its size
#[test]
fn test_synth_length_caps() {
    let s = success();
    let good = s.encode().unwrap();
    assert_eq!(decode_provision_success(&good), Ok(s.clone()), "positive control");
    let (a_off, c_off) = length_offsets(&good, &s);

    // artifact: one over the cap declines for its size, with nothing like that many bytes supplied
    let mut b = good.clone();
    b[a_off..a_off + 8].copy_from_slice(&(MAX_ARTIFACT_LEN + 1).to_le_bytes());
    assert_eq!(
        decode_provision_success(&b),
        Err(SynthDecline::LengthOutOfRange { region: "artifact", got: MAX_ARTIFACT_LEN + 1 })
    );
    // the u64::MAX frame the old text only survived by not allocating
    b[a_off..a_off + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(decode_provision_success(&b), Err(SynthDecline::LengthOutOfRange { region: "artifact", .. })));
    // exactly at the cap is NOT refused for size: the decline is the truncation of the missing bytes
    b[a_off..a_off + 8].copy_from_slice(&MAX_ARTIFACT_LEN.to_le_bytes());
    assert_eq!(decode_provision_success(&b), Err(SynthDecline::Truncated { region: "artifact" }));

    // contract: same shape against the Announce payload cap
    let mut c = good.clone();
    c[c_off..c_off + 4].copy_from_slice(&(MAX_CONTRACT_PAYLOAD_LEN + 1).to_le_bytes());
    assert_eq!(
        decode_provision_success(&c),
        Err(SynthDecline::LengthOutOfRange { region: "contract", got: u64::from(MAX_CONTRACT_PAYLOAD_LEN) + 1 })
    );
    c[c_off..c_off + 4].copy_from_slice(&MAX_CONTRACT_PAYLOAD_LEN.to_le_bytes());
    assert_eq!(decode_provision_success(&c), Err(SynthDecline::Truncated { region: "contract" }));
}

fn doc_with_body(body: Vec<u8>) -> Vec<u8> {
    Document { contract_kind: "kiss-contract".into(), contract_version: "4".into(), body }.encode()
}

// Backs: KISS-CONTRACT-6.11-0003a — a document header declaring a body longer than 2^24 bytes is refused even when the body really is that long and its CRC is right; a body at the cap is not refused for its length
#[test]
fn test_contract_document_len_cap() {
    let over = doc_with_body(vec![b'x'; MAX_DOCUMENT_BODY_LEN as usize + 1]);
    assert!(matches!(read_document(&over), Err(ContractDecline::BadLength { .. })), "len and CRC are consistent, only the cap refuses it");
    let at = doc_with_body(vec![b'x'; MAX_DOCUMENT_BODY_LEN as usize]);
    assert!(
        !matches!(read_document(&at), Err(ContractDecline::BadLength { .. })),
        "a body AT the cap is not refused for its length (it fails later, for lacking a heading)"
    );
}

// Backs: KISS-CONTRACT-6.11-0003b — the CRC-32 detects accidental corruption and truncation but not modification: a body altered and re-checksummed by the party that alters it is accepted
#[test]
fn test_contract_crc_is_not_a_signature() {
    let original = Document { contract_kind: "kiss-contract".into(), contract_version: "4".into(), body: b"[section:1:identity]\nop = add\n".to_vec() };
    let good = original.encode();
    assert!(read_document(&good).is_ok(), "positive control: {:?}", read_document(&good));
    // accidental corruption IS caught: flip a body byte, leave the header
    let mut flipped = good.clone();
    let last = flipped.len() - 2;
    flipped[last] ^= 0x01;
    assert!(matches!(read_document(&flipped), Err(ContractDecline::BadChecksum { .. })));
    // deliberate modification is NOT: change the content and recompute the header
    let forged = Document { body: b"[section:1:identity]\nop = mul\n".to_vec(), ..original }.encode();
    assert_ne!(forged, good);
    assert!(read_document(&forged).is_ok(), "an altered, re-checksummed body passes: the CRC is not a defence against modification");
}

// Backs: KISS-ANNOUNCE-6.3-0008a — revision_hash is not an integrity mechanism: two frames with different artifact bytes under one revision_hash are both well-formed and carry the same identity
#[test]
fn test_announce_revision_hash_is_not_integrity() {
    let a = success();
    let mut b = success();
    b.artifact = vec![0xFF; 64]; // different code, same revision_hash
    assert_eq!(a.identity.revision_hash, b.identity.revision_hash);
    let (da, db) = (decode_provision_success(&a.encode().unwrap()).unwrap(), decode_provision_success(&b.encode().unwrap()).unwrap());
    assert_eq!(da.identity, db.identity, "same identity, same revision_hash");
    assert_ne!(da.artifact, db.artifact, "different artifact bytes: nothing in the frame detects the substitution");
}

// Backs: KISS-ANNOUNCE-7.2-0011 — FEAT bit 34 is reserved by name for the attestation extension: it lies in the FEAT core range, collides with no assigned bit, is zero in the reference envelope, and a reader that sees it set ignores it
#[test]
fn test_announce_attestation_bit_reserved() {
    let bit = FEAT_ARTIFACT_ATTESTATION_BIT;
    assert_eq!(bit, 34);
    assert!((24..40).contains(&bit), "inside the FEAT core sub-range [24,40)");
    assert!(![32, 33].contains(&bit), "not PROVISION_ON_REQUEST or CONTRACT_QUERY");
    let env = reference_envelope();
    assert_eq!(env.capabilities >> bit & 1, 0, "producers write zero to a reserved bit (§7.2-0006)");
    let mut set = env.clone();
    set.capabilities |= 1u64 << bit;
    assert_eq!(decode(&set.encode()), Ok(set), "an unrecognized set bit is ignored, never rejected (§7.2-0007)");
}
