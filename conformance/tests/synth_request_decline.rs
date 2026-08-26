//! KISS-Conform tests for the first SYNTH slice: the provision request (§6.1) and
//! the provision decline framing (§6.6), restarting the #91 SYNTH coverage
//! burndown (SYNTH was 0/130 backed).
//!
//! SYNTH does not mint its own request/decline wire shapes. A provision request
//! REUSES the KISS-Announce contract-query request frame verbatim (tag `CYRQ`,
//! wire bytes `43 59 52 51`), and a provision decline REUSES the Announce decline
//! response frame verbatim (tag `CDEC`, wire bytes `43 44 45 43`) carrying the
//! Announce decline-code enum. So these tests exercise the real
//! `conformance/src/announce.rs` codec — `QueryRequest`/`decode_query_request`,
//! `DeclineResponse`, and the `Identity{structure_key, revision_hash}` block — and
//! assert those bytes/behaviors satisfy the cited SYNTH clauses. No new `src/`
//! scaffolding, no Fuel dependency.

use kiss_conformance::announce::*;
use kiss_conformance::assert_golden;

// The echoed identity block (§6.4-0011 / KISS-SYNTH-6.1-0002), with a revision
// pinned — reused by the CYRQ request and CDEC decline goldens below.
const IDENTITY_PRESENT_GOLDEN: &str = concat!(
    "03 00 00 00 ",                          // structure_key len = 3 (u32 LE)
    "AA BB CC ",                             // structure_key bytes
    "01 ",                                   // revision_present = 1
    "11 11 11 11 11 11 11 11 ",              // revision_hash = 32 * 0x11
    "11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 ",
);

/// A reference provision-request identity: key `AA BB CC`, revision pinned.
fn identity_present() -> Identity {
    Identity { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: Some([0x11; 32]) }
}

/// The same cell, named WITHOUT pinning a build (revision_present == 0).
fn identity_unpinned() -> Identity {
    Identity { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: None }
}

/// The SYNTH provision decline_code a provider emits for a request-path
/// malformation (§6.1-0004 -> §6.6-0003b: every malformed request frame maps to
/// `MALFORMED_REQUEST`). Panics on any decline that is not a request-framing
/// malformation, so a miscategorized decline fails the test rather than passing.
fn request_malformation_code(d: &AnnounceDecline) -> u32 {
    match d {
        AnnounceDecline::BadRevisionPresent { .. }
        | AnnounceDecline::StructureKeyLenOutOfRange { .. }
        | AnnounceDecline::Truncated { .. }
        | AnnounceDecline::BadTag { .. } => decline_code::MALFORMED_REQUEST,
        other => panic!("not a request-framing malformation: {other:?}"),
    }
}

/// The DECLINE-path (§6.6-0004b) echoed-identity decoder. Unlike the request-path
/// decoder (`announce::decode_query_request`, which requires a `structure_key`
/// length in `[1, 4096]` and REJECTS `key_len == 0`), a reader on the decline path
/// MUST ACCEPT the canonical empty identity: a `u32` key-length of `0` with
/// `revision_present == 0` and no `revision_hash` bytes. That asymmetry is the
/// teeth of §6.6-0004b — a naive impl that reused the request decoder here would
/// wrongly reject the empty identity. Returns the decoded identity and the offset
/// just past it; still hard-rejects truncation and an out-of-range length (never a
/// panic or out-of-bounds slice).
fn decode_declined_identity(bytes: &[u8]) -> Result<(Identity, usize), &'static str> {
    if bytes.len() < 4 {
        return Err("truncated structure_key_len");
    }
    let key_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    // key_len == 0 is ACCEPTED here (the request path rejects it); only the cap
    // is enforced on the decline path.
    if key_len > MAX_STRUCTURE_KEY_LEN {
        return Err("structure_key_len over cap");
    }
    let key_len = key_len as usize;
    let mut off = 4;
    if bytes.len() < off + key_len {
        return Err("truncated structure_key");
    }
    let structure_key = bytes[off..off + key_len].to_vec();
    off += key_len;
    if bytes.len() < off + 1 {
        return Err("truncated revision_present");
    }
    let revision_present = bytes[off];
    off += 1;
    if revision_present > 1 {
        return Err("bad revision_present");
    }
    let revision_hash = if revision_present == 1 {
        if bytes.len() < off + 32 {
            return Err("truncated revision_hash");
        }
        let mut h = [0u8; 32];
        h.copy_from_slice(&bytes[off..off + 32]);
        off += 32;
        Some(h)
    } else {
        None
    };
    Ok((Identity { structure_key, revision_hash }, off))
}

// ---- §6.1 Provision request -------------------------------------------------

/// KISS-SYNTH-6.1-0001 — a provision request identifies the wanted kernel by
/// IDENTITY ONLY: a `structure_key`, optionally pinned to a `revision_hash`; the
/// request IS that identity and nothing else.
/// TEETH: the encoded frame is exactly the `CYRQ` tag followed by the identity
/// block, with no bytes to spare, and it decodes back to precisely the identity
/// that was requested. An impl that carried anything beyond the identity would
/// change the byte length below, and a decoder that dropped the pin would fail
/// the round-trip.
#[test]
fn test_synth_request_is_identity_only() {
    let req = QueryRequest { identity: identity_present() };
    let b = req.encode();
    // frame = tag(4) + key_len(4) + key(3) + revision_present(1) + hash(32) = 44.
    assert_eq!(b.len(), 4 + 4 + 3 + 1 + 32, "request = CYRQ tag + identity, nothing else");
    assert_eq!(&b[0..4], &[0x43, 0x59, 0x52, 0x51], "CYRQ tag");
    // decode round-trips to exactly the requested identity (the whole request).
    assert_eq!(decode_query_request(&b), Ok(req.clone()));
    assert_eq!(decode_query_request(&b).unwrap().identity, identity_present());
}

/// KISS-SYNTH-6.1-0001a — a provision request carries NO per-kernel capability,
/// usage/ABI, dispatch, guarantee, or semantics fields (those are the returned
/// contract's job).
/// TEETH: the entire byte budget after the tag is consumed by the identity fields
/// (key-length, key, revision flag, optional hash) — the encoded frame equals the
/// exact identity-only concatenation, byte for byte, in BOTH the pinned and the
/// unpinned form. Any extra capability field would lengthen or alter these bytes
/// and fail the exact-byte golden.
#[test]
fn test_synth_request_no_capability_fields() {
    // Pinned form: CYRQ tag + identity, no trailing capability region.
    let pinned = QueryRequest { identity: identity_present() }.encode();
    let golden_pinned = format!("43 59 52 51 {IDENTITY_PRESENT_GOLDEN}");
    assert_golden("KISS-SYNTH-6.1-0001a", "cyrq_no_capabilities", &pinned, &golden_pinned);
    assert_eq!(pinned.len(), 44, "no room for any capability/ABI/dispatch field");

    // Unpinned form: the frame stops right after the revision_present byte — there
    // is no capability blob and no hash.
    let unpinned = QueryRequest { identity: identity_unpinned() }.encode();
    assert_golden(
        "KISS-SYNTH-6.1-0001a",
        "cyrq_no_capabilities_unpinned",
        &unpinned,
        "43 59 52 51 03 00 00 00 AA BB CC 00",
    );
    assert_eq!(unpinned.len(), 12, "unpinned request = tag + key_len + key + flag, no more");
}

/// KISS-SYNTH-6.1-0002 — a provision request REUSES the KISS-Announce
/// contract-query request frame verbatim (tag `CYRQ`, wire bytes `43 59 52 51`):
/// the 4-byte tag; a little-endian `u32` `structure_key` byte-length in `[1, 4096]`;
/// the key bytes; a 1-byte `revision_present` flag (`0`/`1`); then the 32-byte
/// `revision_hash` iff present.
/// TEETH: the pinned golden catches a big-endian tag write (`51 52 59 43`) and a
/// big-endian key-length; and the round-trip proves the reader reads the same
/// little-endian framing.
// Proven: KISS-SYNTH-6.1-0002 (subject: impl; ref: PROVEN_BATCH2.md)
#[test]
fn test_synth_request_reuses_cyrq() {
    assert_eq!(CYRQ, 0x5152_5943, "CYRQ u32-LE value (wire 43 59 52 51)");
    let req = QueryRequest { identity: identity_present() };
    let b = req.encode();
    let golden = format!("43 59 52 51 {IDENTITY_PRESENT_GOLDEN}");
    assert_golden("KISS-SYNTH-6.1-0002", "cyrq_request", &b, &golden);
    // key-length is a LITTLE-endian u32 in [1,4096], sitting right after the tag.
    assert_eq!(&b[4..8], &[0x03, 0x00, 0x00, 0x00], "key-length is u32-LE (not BE)");
    assert_eq!(u32::from_le_bytes(b[4..8].try_into().unwrap()), 3);
    assert_eq!(decode_query_request(&b), Ok(req));
}

/// KISS-SYNTH-6.1-0003 — a provision request carries the `structure_key` as the
/// OPAQUE, length-delimited KISS-Classify token.
/// TEETH: a key that embeds a NUL byte and bytes that spell the `CYRQ` tag is
/// carried verbatim, delimited only by its u32 length — proving the codec does
/// not treat the key as NUL-terminated, does not re-parse a tag out of it, and
/// does not stop early. A C-string (`strlen`) reflex would truncate at the NUL.
#[test]
fn test_synth_request_structure_key_opaque() {
    // key = the 4 bytes of a CYRQ tag, a NUL, and 0xFF — all opaque payload.
    let key = vec![0x43, 0x59, 0x52, 0x51, 0x00, 0xFF];
    let req = QueryRequest {
        identity: Identity { structure_key: key.clone(), revision_hash: None },
    };
    let b = req.encode();
    // length prefix = 6 (the full opaque token), then the 6 key bytes verbatim.
    assert_eq!(&b[4..8], &[0x06, 0x00, 0x00, 0x00], "key-length delimits the opaque token");
    assert_eq!(&b[8..14], key.as_slice(), "opaque key bytes carried verbatim (incl. NUL)");
    let decoded = decode_query_request(&b).expect("opaque key round-trips");
    assert_eq!(decoded.identity.structure_key, key, "full 6-byte token survives, not strlen-truncated");
}

/// KISS-SYNTH-6.1-0003a — a provider MUST NOT reinterpret, truncate, or re-encode
/// the `structure_key` bytes.
/// TEETH: a key spanning every byte value 0x00..=0xFF (a superset that would break
/// any UTF-8 / base64 / escaping re-encode, and contains interior NULs) round-trips
/// byte-identical, and the encoded key region equals the original bytes at the
/// pinned offset. Length is preserved exactly — no truncation, no expansion.
#[test]
fn test_synth_request_structure_key_not_reencoded() {
    let key: Vec<u8> = (0u16..=255).map(|x| x as u8).collect(); // all 256 byte values
    let req = QueryRequest {
        identity: Identity { structure_key: key.clone(), revision_hash: Some([0x22; 32]) },
    };
    let b = req.encode();
    assert_eq!(&b[4..8], &(key.len() as u32).to_le_bytes(), "length preserved, not re-encoded");
    assert_eq!(&b[8..8 + key.len()], key.as_slice(), "key bytes byte-identical at the wire offset");
    let decoded = decode_query_request(&b).expect("arbitrary bytes round-trip");
    assert_eq!(decoded.identity.structure_key, key, "no reinterpret / truncate / re-encode");
    assert_eq!(decoded.identity.structure_key.len(), 256, "length not altered");
}

/// KISS-SYNTH-6.1-0004 — a provider reading a provision request MUST reject, with
/// a `MALFORMED_REQUEST` typed decline (§6.6), a request whose `revision_present`
/// is not `0`/`1`, whose `structure_key` length is outside `[1, 4096]`, or whose
/// declared length exceeds the remaining input.
/// TEETH: each malformation yields a specific typed decline (never `Ok`), and each
/// maps to `MALFORMED_REQUEST` (0x00000003). A truthy `revision_present` test
/// (`flag != 0`) would ACCEPT `2`; an off-by-one range check would accept `0` or
/// `4097`; an unchecked slice would panic on the over-long declared length.
#[test]
fn test_synth_request_malformed_declines() {
    // (a) revision_present == 2 (neither 0 nor 1).
    let bad_rp = [0x43, 0x59, 0x52, 0x51, 0x01, 0x00, 0x00, 0x00, 0x7A, 0x02];
    let d = decode_query_request(&bad_rp).unwrap_err();
    assert_eq!(d, AnnounceDecline::BadRevisionPresent { got: 2 });
    assert_eq!(request_malformation_code(&d), decline_code::MALFORMED_REQUEST);

    // (b) structure_key length == 0 (below the [1,4096] floor).
    let zero_len = [0x43, 0x59, 0x52, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00];
    let d = decode_query_request(&zero_len).unwrap_err();
    assert_eq!(d, AnnounceDecline::StructureKeyLenOutOfRange { got: 0 });
    assert_eq!(request_malformation_code(&d), decline_code::MALFORMED_REQUEST);

    // (c) structure_key length == 4097 (above the ceiling; 0x1001 LE).
    let over = [0x43, 0x59, 0x52, 0x51, 0x01, 0x10, 0x00, 0x00, 0x00];
    let d = decode_query_request(&over).unwrap_err();
    assert_eq!(d, AnnounceDecline::StructureKeyLenOutOfRange { got: 4097 });
    assert_eq!(request_malformation_code(&d), decline_code::MALFORMED_REQUEST);

    // (d) declared length (8) exceeds the remaining input (only 2 key bytes).
    let too_long = [0x43, 0x59, 0x52, 0x51, 0x08, 0x00, 0x00, 0x00, 0xAA, 0xBB];
    let d = decode_query_request(&too_long).unwrap_err();
    assert_eq!(d, AnnounceDecline::Truncated { region: "structure_key" });
    assert_eq!(request_malformation_code(&d), decline_code::MALFORMED_REQUEST);
}

/// KISS-SYNTH-6.1-0004a — a provider MUST NOT allocate on an unchecked declared
/// length before validating it against the remaining input (check-before-slice).
/// TEETH: a request declares the MAXIMUM in-range key length (4096) but supplies
/// only 3 key bytes. The conforming decoder validates the declared length against
/// the buffer BEFORE slicing/allocating and declines with `Truncated`. An impl
/// that did `bytes[8..8+4096].to_vec()` first would allocate on / read past the
/// short buffer (a panic or out-of-bounds read), never reaching a clean decline.
#[test]
fn test_synth_request_no_alloc_on_unchecked_length() {
    // key_len = 4096 (0x1000, in range) but only 3 key bytes follow.
    let unchecked = [0x43, 0x59, 0x52, 0x51, 0x00, 0x10, 0x00, 0x00, 0xAA, 0xBB, 0xCC];
    assert_eq!(u32::from_le_bytes([0x00, 0x10, 0x00, 0x00]), 4096, "declared length is the in-range max");
    // Must be a clean typed decline — the bounds check precedes the slice.
    assert_eq!(
        decode_query_request(&unchecked),
        Err(AnnounceDecline::Truncated { region: "structure_key" }),
        "declared length checked against remaining input BEFORE allocating/slicing"
    );
}

/// KISS-SYNTH-6.1-0005 — the pair `(structure_key, revision_hash)` is the FULL
/// availability identity: `revision_present == 1` pins one specific build;
/// `revision_present == 0` names the cell WITHOUT pinning a build.
/// TEETH: the same `structure_key` decodes to two distinguishable identities — a
/// pinned one (`Some(hash)`) and an unpinned one (`None`) — so the revision half
/// is load-bearing. Collapsing the two states (ignoring the flag) would make these
/// equal and fail the inequality below.
#[test]
fn test_synth_request_full_identity() {
    let pinned = decode_query_request(&QueryRequest { identity: identity_present() }.encode())
        .unwrap()
        .identity;
    let unpinned = decode_query_request(&QueryRequest { identity: identity_unpinned() }.encode())
        .unwrap()
        .identity;
    // revision_present == 1 pins a specific build.
    assert_eq!(pinned.revision_hash, Some([0x11; 32]), "present flag pins a build");
    // revision_present == 0 names the cell without pinning.
    assert_eq!(unpinned.revision_hash, None, "absent flag names the cell, no build pinned");
    // same key, but the identities differ precisely in the pin — the pair is full.
    assert_eq!(pinned.structure_key, unpinned.structure_key, "same cell key");
    assert_ne!(pinned, unpinned, "the revision_hash half distinguishes pinned from unpinned");
}

/// KISS-SYNTH-6.1-0005a — a provider MUST NOT treat a `structure_key` alone (with
/// `revision_present == 0`) as pinning a specific build.
/// TEETH: an unpinned request decodes to `revision_hash == None`, and crucially
/// NOT to `Some([0u8; 32])`. An impl that reads a phantom all-zero hash when the
/// flag is 0 would falsely pin the cell to an all-zero revision; the assertion
/// below rejects that. The encoded frame also carries no hash bytes at all.
#[test]
fn test_synth_request_key_alone_not_pinned() {
    let b = QueryRequest { identity: identity_unpinned() }.encode();
    // no revision_hash region: frame ends right after the revision_present byte.
    assert_eq!(b.len(), 12, "no 32-byte hash region follows an unpinned request");
    assert_eq!(*b.last().unwrap(), 0x00, "revision_present == 0");
    let ident = decode_query_request(&b).unwrap().identity;
    assert_eq!(ident.revision_hash, None, "key alone does not pin a build");
    assert_ne!(
        ident.revision_hash,
        Some([0u8; 32]),
        "absence MUST NOT be misread as a pinned all-zero revision"
    );
}

// ---- §6.6 Provision decline framing -----------------------------------------

/// KISS-SYNTH-6.6-0004 — a provision decline is framed as the KISS-Announce
/// decline response frame verbatim (tag `CDEC`, wire bytes `43 44 45 43`): the
/// 4-byte tag; the echoed `(structure_key, revision_hash)` identity block; then a
/// little-endian `u32` `decline_code`.
/// TEETH: the pinned golden catches a big-endian tag (`43 44 45 43` vs `43 45 44 43`
/// reversed) and — load-bearing — a `decline_code` emitted as a u16 (which would
/// drop the frame by 2 bytes); the code occupies the final FOUR bytes, LE.
// Proven: KISS-SYNTH-6.6-0004 (subject: impl; ref: PROVEN_BATCH2.md)
#[test]
fn test_synth_decline_framing() {
    assert_eq!(CDEC, 0x4345_4443, "CDEC u32-LE value (wire 43 44 45 43)");
    let resp = DeclineResponse {
        identity: identity_present(),
        decline_code: decline_code::UNKNOWN_STRUCTURE_KEY, // 0x00000001
    };
    let b = resp.encode();
    let golden = format!("43 44 45 43 {IDENTITY_PRESENT_GOLDEN} 01 00 00 00");
    assert_golden("KISS-SYNTH-6.6-0004", "cdec_decline", &b, &golden);
    assert_eq!(&b[0..4], &[0x43, 0x44, 0x45, 0x43], "CDEC tag at offset 0");
    // tag(4) + identity(4+3+1+32=40) + decline_code(4) = 48 bytes.
    assert_eq!(b.len(), 48, "decline_code is u32-LE (4 bytes), not u16");
    assert_eq!(&b[44..48], &[0x01, 0x00, 0x00, 0x00], "little-endian u32 decline_code trails the identity");
}

/// KISS-SYNTH-6.6-0004a — when a `MALFORMED_REQUEST` decline is emitted for a
/// request whose identity cannot be safely parsed, the provider echoes a CANONICAL
/// EMPTY identity in the `CDEC` frame: a `u32` `structure_key` length of `0` and
/// `revision_present == 0` (no `revision_hash` bytes).
/// TEETH: the empty-identity frame is exactly tag + `00 00 00 00` (len 0) + `00`
/// (flag 0) + `03 00 00 00` (MALFORMED_REQUEST) = 13 bytes, with NO hash region. A
/// provider that echoed a `revision_present == 1` or any hash bytes here would
/// lengthen the frame and fail the exact-byte golden.
#[test]
fn test_synth_malformed_echoes_empty_identity() {
    let resp = DeclineResponse {
        identity: Identity { structure_key: vec![], revision_hash: None },
        decline_code: decline_code::MALFORMED_REQUEST, // 0x00000003
    };
    let b = resp.encode();
    assert_golden(
        "KISS-SYNTH-6.6-0004a",
        "cdec_empty_identity",
        &b,
        "43 44 45 43 00 00 00 00 00 03 00 00 00",
    );
    assert_eq!(b.len(), 13, "empty identity = key_len(0) + flag(0), no hash bytes");
    assert_eq!(&b[4..8], &[0x00, 0x00, 0x00, 0x00], "canonical u32 key-length == 0");
    assert_eq!(b[8], 0x00, "revision_present == 0 (no hash follows)");
    assert_eq!(&b[9..13], &[0x03, 0x00, 0x00, 0x00], "MALFORMED_REQUEST decline_code, u32-LE");
}

/// KISS-SYNTH-6.6-0004b — a reader MUST ACCEPT a zero-length echoed `structure_key`
/// (with `revision_present == 0`) on a `MALFORMED_REQUEST` `CDEC` frame.
/// TEETH: this is the request-path/decline-path ASYMMETRY. The request-path
/// decoder (`decode_query_request`) REJECTS `key_len == 0` with
/// `StructureKeyLenOutOfRange` — proven below on the very same identity bytes. A
/// naive decline reader that reused that request decoder would wrongly reject the
/// canonical empty identity. The decline-path decoder MUST accept exactly what the
/// request path rejects.
#[test]
fn test_synth_reader_accepts_empty_identity() {
    // The canonical empty-identity MALFORMED_REQUEST decline (per §6.6-0004a).
    let cdec = DeclineResponse {
        identity: Identity { structure_key: vec![], revision_hash: None },
        decline_code: decline_code::MALFORMED_REQUEST,
    }
    .encode();
    // The identity block sits immediately after the 4-byte CDEC tag.
    let ident_bytes = &cdec[4..]; // key_len(4)=0, revision_present(1)=0, decline_code(4)

    // TEETH: feeding this exact empty identity to the REQUEST-path decoder (as a
    // CYRQ body) is REJECTED — the request path forbids key_len < 1.
    let mut cyrq = CYRQ.to_le_bytes().to_vec();
    cyrq.extend_from_slice(&ident_bytes[..5]); // key_len(0) + revision_present(0)
    assert_eq!(
        decode_query_request(&cyrq),
        Err(AnnounceDecline::StructureKeyLenOutOfRange { got: 0 }),
        "the REQUEST path rejects a zero-length structure_key"
    );

    // ...but the DECLINE-path reader MUST ACCEPT the empty identity.
    let (ident, off) = decode_declined_identity(ident_bytes)
        .expect("§6.6-0004b: the decline path MUST accept the canonical empty identity");
    assert!(ident.structure_key.is_empty(), "zero-length structure_key accepted");
    assert_eq!(ident.revision_hash, None, "revision_present == 0 => no hash");
    assert_eq!(off, 5, "identity block is exactly key_len(4) + flag(1) — no hash bytes consumed");
    // and the u32-LE decline_code trails the accepted identity.
    let code = u32::from_le_bytes(ident_bytes[off..off + 4].try_into().unwrap());
    assert_eq!(code, decline_code::MALFORMED_REQUEST, "MALFORMED_REQUEST follows the empty identity");
}
