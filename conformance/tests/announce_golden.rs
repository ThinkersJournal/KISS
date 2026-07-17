//! KISS-Conform golden + decline vectors for the KISS-Announce 56-byte handshake
//! envelope (Announce §6.1–§6.2). The golden envelope is transcribed from the
//! Announce §2.5 worked example.

use kiss_conformance::announce::*;
use kiss_conformance::{assert_golden, hex};

/// The Announce §2.5 reference envelope: version 1, one profile {1}, capabilities
/// = EXT bits 0–5 | FEAT bit 32 | FEAT bit 33 = 0x0000_0003_0000_003F.
fn reference() -> Envelope {
    Envelope { envelope_version: 1, profiles: vec![1], capabilities: 0x0000_0003_0000_003F }
}

// Built from labeled fields so each boundary is exact (§6.1 layout, 56 bytes).
const GOLDEN: &str = concat!(
    "53 45 41 4D ",                        // [0]  magic  (u32 LE, "SEAM")
    "01 ",                                 // [4]  envelope_version = 1
    "00 00 00 ",                           // [5]  reserved0 (3)
    "01 00 ",                              // [8]  profiles_len = 1
    "01 00 ",                              // [10] profiles[0] = 1
    "00 00 00 00 00 00 00 00 00 00 ",      // [12] profiles[1..16] : 15 * u16 =
    "00 00 00 00 00 00 00 00 00 00 ",      //      30 bytes, all zero
    "00 00 00 00 00 00 00 00 00 00 ",      //
    "00 00 00 00 00 00 ",                  // [42] reserved1 (6)
    "3F 00 00 00 03 00 00 00",             // [48] capabilities (u64 LE)
);

#[test]
fn envelope_golden_bytes() {
    let b = reference().encode();
    assert_eq!(b.len(), ENVELOPE_LEN, "envelope must be 56 bytes (§6.1-0001)");
    assert_golden("KISS-ANNOUNCE-6.1-0002", "reference_envelope", &b, GOLDEN);
}

/// Enforces KISS-ANNOUNCE-6.1-0004 — the pinned magic constant + its wire byte order.
#[test]
fn magic_is_seam_wire_order() {
    // §6.1-0004: magic == 0x4D414553, on-wire bytes 53 45 41 4D ("SEAM")
    assert_eq!(MAGIC, 0x4D41_4553);
    assert_eq!(&reference().encode()[0..4], &[0x53, 0x45, 0x41, 0x4D]);
}

/// Enforces KISS-ANNOUNCE-6.1-0003 — the pinned field offsets of the §6.1 layout table.
#[test]
fn field_offsets_match_table() {
    // §6.1-0003 / §6.1-0011: spot-check pinned offsets on the reference envelope.
    let b = reference().encode();
    assert_eq!(b[4], 1, "envelope_version @ offset 4");
    assert_eq!(u16::from_le_bytes([b[8], b[9]]), 1, "profiles_len @ offset 8");
    assert_eq!(u16::from_le_bytes([b[10], b[11]]), 1, "profiles[0] @ offset 10");
    assert_eq!(u64::from_le_bytes(b[48..56].try_into().unwrap()), 0x0000_0003_0000_003F);
}

/// Enforces KISS-ANNOUNCE-6.1-0012 — every field little-endian: a decode/encode round-trip.
#[test]
fn decode_roundtrips_the_reference() {
    let e = reference();
    assert_eq!(decode(&e.encode()), Ok(e));
}

// ---- Conform modality 4: hard-reject decline vectors (§6.2) ------------------

/// Enforces KISS-ANNOUNCE-6.2-0001 — reject an input whose length != the version's mandate.
#[test]
fn reject_wrong_length() {
    assert_eq!(decode(&[0u8; 55]), Err(AnnounceDecline::WrongLength { got: 55 }));
}

/// Enforces KISS-ANNOUNCE-6.2-0002 — reject a bad magic.
#[test]
fn reject_bad_magic() {
    let mut b = reference().encode();
    b[0] = 0x00; // corrupt the magic
    match decode(&b) {
        Err(AnnounceDecline::BadMagic { .. }) => {}
        other => panic!("expected BadMagic, got {other:?}"),
    }
}

/// Enforces KISS-ANNOUNCE-6.2-0003 — reject an unsupported envelope_version.
#[test]
fn reject_unknown_version() {
    let mut b = reference().encode();
    b[4] = 2; // a version this envelope definition does not implement
    assert_eq!(decode(&b), Err(AnnounceDecline::UnsupportedVersion { got: 2 }));
}

/// Enforces KISS-ANNOUNCE-6.2-0004 — reject a nonzero reserved0.
#[test]
fn reject_nonzero_reserved0() {
    let mut b = reference().encode();
    b[5] = 1; // reserved0 MBZ (§6.1-0006)
    assert_eq!(decode(&b), Err(AnnounceDecline::ReservedNonZero { region: "reserved0" }));
}

/// Enforces KISS-ANNOUNCE-6.2-0011 — reject a nonzero reserved1.
#[test]
fn reject_nonzero_reserved1() {
    let mut b = reference().encode();
    b[42] = 1; // reserved1 MBZ (§6.1-0010)
    assert_eq!(decode(&b), Err(AnnounceDecline::ReservedNonZero { region: "reserved1" }));
}

/// Enforces KISS-ANNOUNCE-6.2-0005 — reject profiles_len > the cap.
#[test]
fn reject_profiles_len_overflow() {
    let mut b = reference().encode();
    b[8] = 17; // profiles_len > 16 (§6.1-0007)
    assert_eq!(decode(&b), Err(AnnounceDecline::ProfilesLenOverflow { got: 17 }));
}

/// Enforces KISS-ANNOUNCE-6.2-0012 — reject a nonzero trailing profile.
#[test]
fn reject_trailing_profile_nonzero() {
    let mut b = reference().encode();
    b[12] = 5; // profiles[1] nonzero while profiles_len == 1 (§6.1-0008)
    assert_eq!(decode(&b), Err(AnnounceDecline::TrailingProfileNonZero));
}

/// Enforces KISS-ANNOUNCE-6.2-0013 — reject a zero live profile entry.
#[test]
fn reject_zero_live_profile() {
    // profiles_len = 2 but profiles[1] = 0 (a live entry must be >= 1, §6.1-0015)
    let mut b = reference().encode();
    b[8] = 2; // profiles_len = 2
    b[12] = 0; // profiles[1] = 0 (live but zero)  -> also non-ascending, but zero-live is checked first
    assert_eq!(decode(&b), Err(AnnounceDecline::ZeroLiveProfile));
}

/// Enforces KISS-ANNOUNCE-6.2-0006 — reject non-strictly-ascending profiles.
#[test]
fn reject_non_ascending_profiles() {
    // profiles_len = 2, {1, 1} — not strictly ascending (§6.1-0009)
    let mut b = reference().encode();
    b[8] = 2; // profiles_len = 2
    b[12] = 1;
    b[13] = 0; // profiles[1] = 1
    assert_eq!(decode(&b), Err(AnnounceDecline::ProfilesNotStrictlyAscending));
}

// ---- §6.3 availability list + §6.4 contract-query frames ---------------------

/// A reference availability list: list_version 1, two records. Its byte layout is
/// spec-derived (no transcribed golden exists for these frames) but every field
/// width/order/endianness is pinned verbatim by §6.3-0011/0012/0005.
fn reference_list() -> AvailabilityList {
    AvailabilityList {
        list_version: 1,
        records: vec![
            AvailabilityRecord { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: [0x11; 32] },
            AvailabilityRecord { structure_key: vec![0xDE, 0xAD], revision_hash: [0x22; 32] },
        ],
    }
}

// Built from labeled fields so each boundary is exact (§6.3 framing).
const AVAIL_GOLDEN: &str = concat!(
    "53 41 56 4C ",                          // [0]  SAVL tag       (u32 LE => 53 41 56 4C)
    "01 ",                                   // [4]  list_version = 1
    "00 00 00 ",                             // [5]  3 MBZ bytes
    "02 00 00 00 ",                          // [8]  record_count = 2  (u32 LE)
    // -- record 0 --
    "03 00 00 00 ",                          // [12] structure_key len = 3 (u32 LE)
    "AA BB CC ",                             // [16] structure_key bytes
    "11 11 11 11 11 11 11 11 ",              // [19] revision_hash = 32 * 0x11
    "11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 ",
    // -- record 1 --
    "02 00 00 00 ",                          // [51] structure_key len = 2 (u32 LE)
    "DE AD ",                                // [55] structure_key bytes
    "22 22 22 22 22 22 22 22 ",              // [57] revision_hash = 32 * 0x22
    "22 22 22 22 22 22 22 22 ",
    "22 22 22 22 22 22 22 22 ",
    "22 22 22 22 22 22 22 22 ",
);

/// Enforces KISS-ANNOUNCE-6.3-0011 — an availability list begins with the 4-byte
/// tag `SAVL` (wire bytes 53 41 56 4C) at offset 0.
/// TEETH: a big-endian tag write (the `htonl` reflex) would emit 4C 56 41 53.
#[test]
fn test_announce_availability_list_tag() {
    assert_eq!(SAVL, 0x4C56_4153, "SAVL u32-LE value (§2.4 table)");
    let b = reference_list().encode();
    assert_eq!(&b[0..4], &[0x53, 0x41, 0x56, 0x4C], "SAVL wire bytes at offset 0");
}

/// Enforces KISS-ANNOUNCE-6.3-0012 — after the tag, a 1-byte `list_version` (== 1)
/// then 3 MBZ bytes; and a reader declines a nonzero MBZ byte or a version != 1.
/// TEETH (producer): an impl that packs `record_count` immediately after the tag
/// would put 02 at offset 4 (record_count == 2) instead of the version byte 01,
/// and the record_count would not sit at offset 8. TEETH (reader): a reader that
/// tolerates a nonzero MBZ byte, or accepts list_version != 1, is caught.
#[test]
fn test_announce_availability_list_version() {
    let b = reference_list().encode();
    assert_eq!(b[4], 1, "list_version == 1 at offset 4 (not record_count's low byte 02)");
    assert_eq!(&b[5..8], &[0, 0, 0], "3 MBZ bytes at offset 5");
    assert_eq!(&b[8..12], &[0x02, 0x00, 0x00, 0x00], "record_count sits at offset 8, u32 LE");

    // reader: unsupported list_version -> typed decline
    let mut bad = b.clone();
    bad[4] = 2;
    assert_eq!(
        decode_availability_list(&bad),
        Err(AnnounceDecline::ListVersionUnsupported { got: 2 })
    );
    // reader: a nonzero MBZ byte -> typed decline
    let mut bad = b.clone();
    bad[5] = 1;
    assert_eq!(
        decode_availability_list(&bad),
        Err(AnnounceDecline::ReservedNonZero { region: "savl_mbz" })
    );
}

/// Enforces KISS-ANNOUNCE-6.3-0005 — the availability-list framing: tag, version
/// block, u32-LE `record_count`, then per record {u32-LE `structure_key` length,
/// key bytes, 32-byte `revision_hash`}.
/// TEETH: a big-endian `record_count` or `key_len` (the `htonl` reflex) would emit
/// 00 00 00 02 / 00 00 00 03 where the spec pins 02 00 00 00 / 03 00 00 00.
#[test]
fn test_announce_availability_framing() {
    let b = reference_list().encode();
    assert_golden("KISS-ANNOUNCE-6.3-0005", "availability_list", &b, AVAIL_GOLDEN);
    // round-trip proves the reader reads the same little-endian framing.
    assert_eq!(decode_availability_list(&b), Ok(reference_list()));
}

/// A reference echoed identity block: structure_key AA BB CC, revision present.
fn reference_identity() -> Identity {
    Identity { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: Some([0x11; 32]) }
}

// The identity block (§6.4-0011), reused by CYRQ/CRSP/CDEC goldens.
const IDENTITY_GOLDEN: &str = concat!(
    "03 00 00 00 ",                          // structure_key len = 3 (u32 LE)
    "AA BB CC ",                             // structure_key bytes
    "01 ",                                   // revision_present = 1
    "11 11 11 11 11 11 11 11 ",              // revision_hash = 32 * 0x11
    "11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 ",
    "11 11 11 11 11 11 11 11 ",
);

/// Enforces KISS-ANNOUNCE-6.4-0001 — the contract-query request framing (`CYRQ`
/// tag, u32-LE key length in [1,4096], key bytes, 1-byte `revision_present` that
/// is exactly 0 or 1, then the 32-byte `revision_hash` iff present).
/// TEETH: treating `revision_present` as truthy (`if flag != 0`) would ACCEPT a
/// flag of 2 as "present"; the spec requires a typed decline for any value other
/// than 0/1. Also catches a big-endian tag write and an out-of-range key length.
#[test]
fn test_announce_query_request_shape() {
    assert_eq!(CYRQ, 0x5152_5943, "CYRQ u32-LE value (§2.4 table)");
    let req = QueryRequest { identity: reference_identity() };
    let b = req.encode();
    let golden = format!("{} {}", "43 59 52 51", IDENTITY_GOLDEN); // CYRQ tag + identity
    assert_golden("KISS-ANNOUNCE-6.4-0001", "cyrq_request", &b, &golden);
    assert_eq!(decode_query_request(&b), Ok(req));

    // revision_present == 2 is NOT truthy-accepted -> typed decline.
    // Raw request: CYRQ, key_len=1, key=[0x7A], revision_present=2.
    let bad = [0x43, 0x59, 0x52, 0x51, 0x01, 0x00, 0x00, 0x00, 0x7A, 0x02];
    assert_eq!(
        decode_query_request(&bad),
        Err(AnnounceDecline::BadRevisionPresent { got: 2 })
    );

    // key length 0 and > 4096 are both out of range.
    let zero_len = [0x43, 0x59, 0x52, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(
        decode_query_request(&zero_len),
        Err(AnnounceDecline::StructureKeyLenOutOfRange { got: 0 })
    );
    let over = [0x43, 0x59, 0x52, 0x51, 0x01, 0x10, 0x00, 0x00, 0x00]; // 0x1001 = 4097
    assert_eq!(
        decode_query_request(&over),
        Err(AnnounceDecline::StructureKeyLenOutOfRange { got: 4097 })
    );
}

/// Enforces KISS-ANNOUNCE-6.4-0006 — a contract-query reader never panics or reads
/// out of bounds on a truncated request.
/// TEETH: an impl that does `bytes[off..off+32].try_into().unwrap()` on a request
/// cut off mid-`revision_hash` panics; the conforming reader returns a typed
/// decline for every truncation point.
#[test]
fn test_announce_query_never_panics() {
    // A full valid request, then every proper prefix of it (each is a truncation).
    let full = QueryRequest { identity: reference_identity() }.encode();
    // Explicitly include the mid-revision-hash cut: key_len=3 (+3 key) + rp=1 read,
    // then only 10 of 32 hash bytes present.
    let mid_hash = &full[..full.len() - 22];
    assert!(
        matches!(decode_query_request(mid_hash), Err(AnnounceDecline::Truncated { .. })),
        "mid-revision-hash truncation must be a typed decline, not a panic"
    );
    // Sweep every prefix length; none may panic, all decline (or, at full length, Ok).
    for n in 0..=full.len() {
        let out = decode_query_request(&full[..n]);
        if n == full.len() {
            assert!(out.is_ok(), "the full request decodes");
        } else {
            assert!(out.is_err(), "prefix of length {n} must decline, never panic");
        }
    }
}

/// Enforces KISS-ANNOUNCE-6.4-0004 — the contract-response framing: `CRSP` tag,
/// echoed identity block, u32-LE payload byte-length, then the payload.
/// TEETH: a big-endian tag write emits 50 53 52 43 instead of 43 52 53 50; a
/// big-endian payload length is also caught by the pinned golden.
#[test]
fn test_announce_contract_response_framing() {
    assert_eq!(CRSP, 0x5053_5243, "CRSP u32-LE value (§2.4 table)");
    let resp = ContractResponse {
        identity: reference_identity(),
        payload: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01], // 5-byte contract document
    };
    let b = resp.encode();
    let golden = format!(
        "{} {} {}",
        "43 52 53 50",       // CRSP tag
        IDENTITY_GOLDEN,     // echoed identity
        "05 00 00 00 DE AD BE EF 01" // payload_len = 5 (u32 LE) + payload
    );
    assert_golden("KISS-ANNOUNCE-6.4-0004", "crsp_response", &b, &golden);
    assert_eq!(&b[0..4], &[0x43, 0x52, 0x53, 0x50], "CRSP wire bytes at offset 0");
}

/// Enforces KISS-ANNOUNCE-6.4-0007 — the decline-response framing: `CDEC` tag,
/// echoed identity block, then a little-endian u32 `decline_code`.
/// TEETH: emitting `decline_code` as a u16 drops the frame by 2 bytes and truncates
/// the field; the golden pins a 4-byte little-endian code. A big-endian tag write
/// is caught too.
#[test]
fn test_announce_decline_response_framing() {
    assert_eq!(CDEC, 0x4345_4443, "CDEC u32-LE value (§2.4 table)");
    let resp = DeclineResponse {
        identity: reference_identity(),
        decline_code: decline_code::UNKNOWN_REVISION, // 0x00000006
    };
    let b = resp.encode();
    let golden = format!(
        "{} {} {}",
        "43 44 45 43",   // CDEC tag
        IDENTITY_GOLDEN, // echoed identity
        "06 00 00 00"    // decline_code = 6 (u32 LE, four bytes)
    );
    assert_golden("KISS-ANNOUNCE-6.4-0007", "cdec_response", &b, &golden);
    // Width is load-bearing: the code occupies the final 4 bytes, not 2.
    assert_eq!(b.len(), 48, "CDEC frame = tag(4) + identity(40) + decline_code(4)");
    assert_eq!(&b[44..48], &[0x06, 0x00, 0x00, 0x00], "decline_code is u32-LE, not u16");
}

#[test]
fn hex_of_golden_is_stable() {
    // documents the exact wire bytes in the test output on demand
    assert!(hex(&reference().encode()).starts_with("53 45 41 4D 01"));
}

/// Enforces KISS-ANNOUNCE-7.1-0001 — negotiate returns max(L ∩ R), not the first mutual.
#[test]
fn test_announce_negotiate_selects_highest_mutual() {
    // L = {2, 5, 9}, R = {1, 5, 9}  ->  L ∩ R = {5, 9}  ->  max = 9.
    // The lowest mutual profile is 5; a "return the first mutual" impl would pick 5.
    let local = Envelope { envelope_version: 1, profiles: vec![2, 5, 9], capabilities: 0 };
    let remote = Envelope { envelope_version: 1, profiles: vec![1, 5, 9], capabilities: 0 };
    assert_eq!(negotiate(&local, &remote), Ok(9));
    // Symmetry: swapping roles yields the same highest-mutual profile.
    assert_eq!(negotiate(&remote, &local), Ok(9));
}

/// Enforces KISS-ANNOUNCE-7.1-0002 — disjoint live-profile sets yield a typed decline, no panic.
#[test]
fn test_announce_negotiate_empty_intersection_declines() {
    // L = {1, 3}, R = {2, 4}  ->  L ∩ R = {}  ->  typed NoMutualProfile decline.
    let local = Envelope { envelope_version: 1, profiles: vec![1, 3], capabilities: 0 };
    let remote = Envelope { envelope_version: 1, profiles: vec![2, 4], capabilities: 0 };
    assert_eq!(negotiate(&local, &remote), Err(AnnounceDecline::NoMutualProfile));
}

/// Enforces KISS-ANNOUNCE-7.2-0007 — an unrecognized capability bit is ignored (not rejected),
/// while recognized bits still round-trip. Regression lock over decode()'s pass-through.
#[test]
fn test_announce_reader_ignores_unknown_capability_bits() {
    // bit 48 = SPEAKS_ANNOUNCE (recognized, §7.2-0004); bit 63 = SUB vendor range,
    // unassigned in the first-draft registry (§7.2 table) -> unrecognized by this reader.
    let recognized = 1u64 << 48; // 0x0001_0000_0000_0000
    let unknown = 1u64 << 63;    // 0x8000_0000_0000_0000
    let caps = recognized | unknown; // 0x8001_0000_0000_0000
    let e = Envelope { envelope_version: 1, profiles: vec![1], capabilities: caps };
    // decode MUST accept the envelope despite the unrecognized bit, and preserve
    // the full bitset verbatim (no masking, no hard-reject).
    let decoded = decode(&e.encode())
        .expect("§7.2-0007: an unrecognized capability bit MUST NOT cause rejection");
    assert_eq!(decoded.capabilities, caps);
    // Contrast with §6.2: an unknown byte in a reserved region IS hard-rejected,
    // proving the ignore-unknown rule is confined to the capabilities bitset.
    let mut bad = e.encode();
    bad[42] = 1; // reserved1 (offset 42) MBZ
    assert_eq!(decode(&bad), Err(AnnounceDecline::ReservedNonZero { region: "reserved1" }));
}
