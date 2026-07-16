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
