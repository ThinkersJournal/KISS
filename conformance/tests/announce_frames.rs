//! KISS-Conform behavioral vectors for the "rest of the KISS-Announce codec"
//! beyond the transcribed golden in `announce_golden.rs`: producer MBZ/size
//! layout discipline (§6.1), the reader hard-reject / never-repair discipline
//! (§6.2), availability-record identity (§6.3), and the cross-frame identity
//! echo (§6.4-0011).
//!
//! Every test drives the REAL codec (`announce::Envelope::encode`/`decode`,
//! `AvailabilityList`/`decode_availability_list`, `QueryRequest`/
//! `ContractResponse`/`DeclineResponse::encode`). Reference values are
//! spec-derived (built from labeled §6.1/§6.3 fields), so no new transcribed
//! golden vector is needed — the teeth come from encode/decode BEHAVIOR, never
//! from string matching. Adversarial byte arrays are hand-built only to feed a
//! malformed input to the reader, never to re-encode a valid frame by hand.
//!
//! Each `#[test] fn` is named EXACTLY per its spec `*Test:*` tag (forward
//! binding) and cites its own clause id in the doc-comment (reverse binding).
//! Cross-references to OTHER clauses use the short `§x.y-nnnn` form so they do
//! not reverse-bind (and over-credit) a clause this module is not backing.

use kiss_conformance::announce::*;

/// The Announce §2.5 reference envelope: version 1, one profile {1}, capabilities
/// = EXT bits 0–5 | FEAT bit 32 | FEAT bit 33 = 0x0000_0003_0000_003F.
fn reference() -> Envelope {
    Envelope { envelope_version: 1, profiles: vec![1], capabilities: 0x0000_0003_0000_003F }
}

/// A maximally-populated envelope: all 16 profile slots live + ascending, and a
/// capabilities word with every bit set. Used to prove the fixed-offset reserved
/// pads stay zero regardless of field values (not just on the single golden).
fn saturated() -> Envelope {
    Envelope {
        envelope_version: 1,
        profiles: (1u16..=16).collect(), // 16 live, strictly ascending, each >= 1
        capabilities: u64::MAX,
    }
}

// ---- §6.1 producer layout discipline ----------------------------------------

/// Enforces KISS-ANNOUNCE-6.1-0010 — a producer MUST write all-zero bytes to
/// `reserved1` (offset 42, length 6, the alignment padding).
/// TEETH: asserted on a VARIED envelope set incl. a fully-set `capabilities`
/// u64. A packed 50-byte layout with no 6-byte pad would write `capabilities`
/// at offset 42, bleeding its low bytes (0xFF… under `saturated()`) into
/// 42..48 — so this fails for any encoder that drops the pad, independent of
/// the single 56-byte golden.
#[test]
fn test_announce_reserved1_pad_zero() {
    let mut envelopes = vec![reference(), saturated()];
    // 16 assorted profile/capability profiles so the property is not a
    // one-vector coincidence.
    for k in 0u16..14 {
        envelopes.push(Envelope {
            envelope_version: 1,
            profiles: (1u16..=(k + 1)).collect(),
            capabilities: 0xDEAD_BEEF_0000_0000u64.wrapping_mul(u64::from(k) + 1),
        });
    }
    for e in &envelopes {
        let b = e.encode();
        assert_eq!(b.len(), ENVELOPE_LEN);
        assert_eq!(
            &b[42..48],
            &[0u8; 6],
            "KISS-ANNOUNCE-6.1-0010: reserved1 (42..48) MUST be all-zero even with capabilities={:#018x}",
            e.capabilities
        );
    }
}

/// Enforces KISS-ANNOUNCE-6.1-0011 — the `capabilities` field MUST occupy offset
/// 48 as an 8-byte little-endian unsigned integer.
/// TEETH: distinct-byte value 0x0807_0605_0403_0201, so a big-endian write (the
/// `htonl` reflex) or a wrong offset produces a byte transposition the golden's
/// 0x…003F cannot catch; a decode round-trip proves the reader reads the same
/// 8 little-endian bytes back into the u64.
#[test]
fn test_announce_capabilities_field() {
    let caps = 0x0807_0605_0403_0201u64;
    let e = Envelope { envelope_version: 1, profiles: vec![1], capabilities: caps };
    let b = e.encode();
    assert_eq!(
        &b[48..56],
        &caps.to_le_bytes(),
        "KISS-ANNOUNCE-6.1-0011: capabilities is 8-byte LE at offset 48 (01 02 03 04 05 06 07 08)"
    );
    assert_eq!(&b[48..56], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    // reader reconstructs the identical u64 from the same 8 LE bytes.
    assert_eq!(decode(&b).unwrap().capabilities, caps);
}

/// Enforces KISS-ANNOUNCE-6.1-0013 — every field MUST occupy the exact SIZE in
/// the §6.1 layout table.
/// TEETH: the field byte-WIDTHS are pinned from the spec table; the test derives
/// each field's offset by prefix-sum and checks the REAL codec output lands
/// there, and that the widths total ENVELOPE_LEN == 56. A drift such as
/// `profiles_len` encoded as u8 (1) or `profiles` as u16[8] (16) shifts every
/// following field: `capabilities` no longer sits at the derived offset 48, and
/// the width total no longer equals 56 — both assertions fail.
#[test]
fn test_announce_field_sizes_match_table() {
    // (field, width) transcribed from the §6.1 layout table.
    let fields: [(&str, usize); 7] = [
        ("magic", 4),
        ("envelope_version", 1),
        ("reserved0", 3),
        ("profiles_len", 2),
        ("profiles", 32),
        ("reserved1", 6),
        ("capabilities", 8),
    ];
    let total: usize = fields.iter().map(|(_, w)| *w).sum();
    assert_eq!(total, 56, "KISS-ANNOUNCE-6.1-0013: field widths must sum to 56");
    assert_eq!(total, ENVELOPE_LEN);

    // prefix-sum the widths into the offset each field MUST begin at.
    let mut off = std::collections::HashMap::new();
    let mut acc = 0usize;
    for (name, w) in fields {
        off.insert(name, acc);
        acc += w;
    }
    assert_eq!(off["magic"], 0);
    assert_eq!(off["capabilities"], 48, "capabilities offset is fixed by the preceding widths");

    let caps = 0x0807_0605_0403_0201u64;
    let e = Envelope { envelope_version: 1, profiles: vec![1, 2, 3], capabilities: caps };
    let b = e.encode();
    assert_eq!(b.len(), total);
    // each field's bytes land at the width-derived offset in the real codec.
    assert_eq!(&b[off["magic"]..off["magic"] + 4], &MAGIC.to_le_bytes());
    assert_eq!(b[off["envelope_version"]], 1);
    assert_eq!(
        u16::from_le_bytes(b[off["profiles_len"]..off["profiles_len"] + 2].try_into().unwrap()),
        3,
        "profiles_len is a 2-byte field at offset 8"
    );
    assert_eq!(
        u64::from_le_bytes(b[off["capabilities"]..off["capabilities"] + 8].try_into().unwrap()),
        caps,
        "capabilities lands at the width-derived offset 48 (a shorter profiles_len would misplace it)"
    );
}

/// Enforces KISS-ANNOUNCE-6.1-0006 — a producer MUST write all-zero bytes to
/// `reserved0` (offset 5, length 3).
/// TEETH: asserts the producer ZEROES reserved0 across the reference and a
/// non-default envelope (distinct from the reader-reject §6.2-0004). Fails on an
/// encoder that widens `envelope_version` past its 1-byte field, or that emits
/// from a reused/dirty buffer, leaving nonzero bytes at 5..8.
#[test]
fn test_announce_reserved0_is_zero() {
    for e in [reference(), saturated()] {
        let b = e.encode();
        assert_eq!(
            &b[5..8],
            &[0u8; 3],
            "KISS-ANNOUNCE-6.1-0006: reserved0 (5..8) MUST be all-zero"
        );
    }
}

/// Enforces KISS-ANNOUNCE-6.1-0008 — a producer MUST write zero to every
/// `profiles` entry at index `>= profiles_len`.
/// TEETH: the boundary is checked exactly at `profiles_len`: the LIVE entries
/// are asserted present (nonzero, at the right slots) AND every trailing slot up
/// to offset 42 is zero. An off-by-one that clobbers the last live profile or
/// leaves a stale trailing slot is caught (distinct from reader-reject §6.2-0012).
#[test]
fn test_announce_trailing_profiles_zero() {
    for profiles in [vec![1u16], vec![1u16, 3]] {
        let e = Envelope { envelope_version: 1, profiles: profiles.clone(), capabilities: 0 };
        let b = e.encode();
        let len = profiles.len();
        // live entries are present at slots [0, len).
        for (i, &p) in profiles.iter().enumerate() {
            let o = 10 + i * 2;
            assert_eq!(
                u16::from_le_bytes(b[o..o + 2].try_into().unwrap()),
                p,
                "KISS-ANNOUNCE-6.1-0008: live profile {i} must remain at slot {i}"
            );
        }
        // every trailing slot [len, 16) is zero, up to reserved1 at offset 42.
        for j in (10 + len * 2)..42 {
            assert_eq!(
                b[j], 0,
                "KISS-ANNOUNCE-6.1-0008: trailing profiles byte {j} (index >= profiles_len={len}) MUST be zero"
            );
        }
    }
}

// ---- §6.2 reader hard-reject discipline -------------------------------------

/// Enforces KISS-ANNOUNCE-6.2-0007 — on any rejection, a reader MUST return a
/// typed decline and MUST NOT panic, abort, crash, hang, or read outside the
/// input buffer.
/// TEETH: (a) a battery of malformations each yields `Err(AnnounceDecline::_)`;
/// (b) EVERY input length 0..=64 is swept, and any length != 56 must be a typed
/// `WrongLength` (never a panic). A reader that does `bytes[0..4].try_into()
/// .unwrap()` before checking length panics on a short input — the sweep fails it.
#[test]
fn test_announce_rejection_is_typed_decline() {
    // (a) each malformation is a typed decline, not a panic.
    let mut bad_magic = reference().encode();
    bad_magic[0] ^= 0xFF;
    let mut bad_version = reference().encode();
    bad_version[4] = 9;
    let mut bad_r0 = reference().encode();
    bad_r0[6] = 1;
    let mut bad_r1 = reference().encode();
    bad_r1[44] = 1;
    let mut over_len = reference().encode();
    over_len[8] = 17; // profiles_len > 16
    let mut zero_live = reference().encode();
    zero_live[8] = 2; // profiles_len = 2
    zero_live[12] = 0; // profiles[1] = 0 (live but zero)
    let mut non_asc = reference().encode();
    non_asc[8] = 2;
    non_asc[12] = 1; // profiles[1] = 1 -> {1,1} not strictly ascending
    let mut trailing = reference().encode();
    trailing[12] = 7; // profiles[1] nonzero while profiles_len == 1

    for (name, buf) in [
        ("bad_magic", bad_magic),
        ("bad_version", bad_version),
        ("nonzero_reserved0", bad_r0),
        ("nonzero_reserved1", bad_r1),
        ("profiles_len_overflow", over_len),
        ("zero_live_profile", zero_live),
        ("non_ascending", non_asc),
        ("trailing_nonzero", trailing),
    ] {
        assert!(
            matches!(decode(&buf), Err(_)),
            "KISS-ANNOUNCE-6.2-0007: malformation `{name}` MUST be a typed decline, got Ok"
        );
    }

    // (b) length sweep: no length may panic; only len == 56 with valid content
    // decodes, every other length is the typed WrongLength.
    let valid = reference().encode();
    for n in 0..=64usize {
        let slice: Vec<u8> = if n <= valid.len() {
            valid[..n].to_vec()
        } else {
            let mut v = valid.clone();
            v.resize(n, 0); // pad past 56 to sweep over-length inputs too
            v
        };
        let out = decode(&slice);
        if n == ENVELOPE_LEN {
            assert_eq!(out, Ok(reference()), "the full 56-byte reference decodes");
        } else {
            assert_eq!(
                out,
                Err(AnnounceDecline::WrongLength { got: n }),
                "KISS-ANNOUNCE-6.2-0007: length {n} != 56 MUST be typed WrongLength, never a panic"
            );
        }
    }
}

/// Enforces KISS-ANNOUNCE-6.2-0008 — a reader MUST NOT tolerate, silently
/// ignore, or attempt to repair a malformed envelope.
/// TEETH: hard-reject vs soft-repair. For each repair-TEMPTING malformation the
/// reader must return `Err`, NEVER `Ok`: a nonzero reserved region is not
/// silently zeroed; `profiles_len = 17` is not clamped to 16 and accepted; a
/// descending or duplicate profile set is not silently sorted/deduped then
/// accepted. A lenient reader that "fixes" garbage and returns a repaired
/// Envelope is caught. (Contrast the ignore-unknown rule for §7.2-0007 caps.)
#[test]
fn test_announce_reader_never_repairs() {
    // reserved not silently zeroed:
    let mut dirty_r0 = reference().encode();
    dirty_r0[5] = 0x5A;
    let mut dirty_r1 = reference().encode();
    dirty_r1[47] = 0x5A;
    // profiles_len not clamped to the cap:
    let mut clamp = reference().encode();
    clamp[8] = 17;
    // descending profiles not silently sorted-then-accepted:
    let mut descending = reference().encode();
    descending[8] = 2;
    descending[10] = 3; // profiles[0] = 3
    descending[12] = 1; // profiles[1] = 1  -> {3,1}
    // duplicate profiles not silently deduped-then-accepted:
    let mut dup = reference().encode();
    dup[8] = 2;
    dup[10] = 2; // profiles[0] = 2
    dup[12] = 2; // profiles[1] = 2  -> {2,2}

    for (name, buf) in [
        ("dirty_reserved0", dirty_r0),
        ("dirty_reserved1", dirty_r1),
        ("profiles_len_17", clamp),
        ("descending_profiles", descending),
        ("duplicate_profiles", dup),
    ] {
        assert!(
            decode(&buf).is_err(),
            "KISS-ANNOUNCE-6.2-0008: reader MUST hard-reject `{name}`, not repair it into Ok"
        );
    }
}

// ---- §6.3 kernel availability (identity only) -------------------------------

/// Enforces KISS-ANNOUNCE-6.3-0001 — a provider MUST announce each available
/// kernel as an availability record consisting SOLELY of the pair
/// `(structure_key, revision_hash)`.
/// TEETH: the per-record wire footprint is the closed form
/// 4 (u32 key_len) + key_len + 32 (hash) with ZERO unexplained bytes before the
/// next record; a decode round-trip recovers the identical two-field record. Any
/// added per-record field (a capability/dispatch/guarantee slot) enlarges the
/// footprint and breaks the closed-form length and the round-trip.
#[test]
fn test_announce_availability_is_identity_pair() {
    const HDR: usize = 12; // SAVL tag (4) + list_version block (4) + record_count (4)
    let one = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: [0x11; 32] }],
    };
    let b = one.encode();
    let footprint = b.len() - HDR;
    let key_len = one.records[0].structure_key.len();
    assert_eq!(
        footprint,
        4 + key_len + 32,
        "KISS-ANNOUNCE-6.3-0001: a record is exactly (u32 key_len, key, 32-byte hash) — no extra field"
    );
    assert_eq!(decode_availability_list(&b), Ok(one));

    // two records: back-to-back, no inter-record padding.
    let two = AvailabilityList {
        list_version: 1,
        records: vec![
            AvailabilityRecord { structure_key: vec![0x01], revision_hash: [0x00; 32] },
            AvailabilityRecord { structure_key: vec![0xDE, 0xAD, 0xBE, 0xEF], revision_hash: [0xFF; 32] },
        ],
    };
    let b2 = two.encode();
    let expected = HDR + (4 + 1 + 32) + (4 + 4 + 32);
    assert_eq!(
        b2.len(),
        expected,
        "KISS-ANNOUNCE-6.3-0001: records pack contiguously with no per-record slack"
    );
    assert_eq!(decode_availability_list(&b2), Ok(two));
}

/// Enforces KISS-ANNOUNCE-6.3-0003 — `revision_hash` MUST be exactly 32 bytes.
/// TEETH: two-sided. Producer — the encoded record's hash region is exactly 32
/// bytes (`record_len - 4 - key_len == 32`), so a 16- or 20-byte hash fails the
/// closed form. Reader — a list whose final record's hash is cut to 31 bytes
/// declines `Truncated{region:"revision_hash"}` rather than reading a short/OOB
/// hash.
#[test]
fn test_announce_revision_hash_is_32_bytes() {
    const HDR: usize = 12;
    let key = vec![0x7A, 0x7B, 0x7C, 0x7D, 0x7E];
    let list = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: key.clone(), revision_hash: [0x33; 32] }],
    };
    let b = list.encode();
    // producer: the hash region is exactly 32 bytes.
    let hash_region = b.len() - HDR - 4 - key.len();
    assert_eq!(hash_region, 32, "KISS-ANNOUNCE-6.3-0003: encoded revision_hash is exactly 32 bytes");

    // reader: chop the last hash byte (31 present) -> typed truncation decline.
    let cut = &b[..b.len() - 1];
    assert_eq!(
        decode_availability_list(cut),
        Err(AnnounceDecline::Truncated { region: "revision_hash" }),
        "KISS-ANNOUNCE-6.3-0003: a 31-byte hash tail must decline, not read a short/OOB hash"
    );
}

/// Enforces KISS-ANNOUNCE-6.3-0004 — a provider MUST carry `structure_key` as
/// the opaque, length-delimited KISS-Classify token and MUST NOT reinterpret,
/// truncate, or re-encode its bytes.
/// TEETH: an adversarial key {0x00, 0x00, 0xFF, 0x80, 0x41} (embedded NUL, high
/// bytes, non-UTF8) round-trips verbatim through BOTH an availability record and
/// a CYRQ identity block. A reader that treats the key as a C-string truncates
/// at the NUL (yielding an empty key); one that treats it as UTF-8 mangles or
/// rejects 0xFF/0x80 — both break the verbatim equality.
#[test]
fn test_announce_structure_key_is_opaque() {
    let key = vec![0x00u8, 0x00, 0xFF, 0x80, 0x41]; // NUL-embedded, high bytes, non-UTF8

    // via an availability record.
    let list = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: key.clone(), revision_hash: [0x55; 32] }],
    };
    let decoded = decode_availability_list(&list.encode()).expect("opaque key must decode");
    assert_eq!(
        decoded.records[0].structure_key, key,
        "KISS-ANNOUNCE-6.3-0004: availability key bytes MUST round-trip verbatim (no NUL-truncate/UTF-8 mangle)"
    );

    // via a CYRQ identity block.
    let req = QueryRequest { identity: Identity { structure_key: key.clone(), revision_hash: None } };
    let back = decode_query_request(&req.encode()).expect("opaque key must decode");
    assert_eq!(
        back.identity.structure_key, key,
        "KISS-ANNOUNCE-6.3-0004: CYRQ identity key bytes MUST round-trip verbatim"
    );
}

/// Enforces KISS-ANNOUNCE-6.3-0008 — `revision_hash` MUST be treated as an
/// opaque provider-assigned identifier compared only for equality; an impl MUST
/// NOT assume any hash algorithm or recomputable input domain.
/// TEETH: an all-zero [0u8;32] hash is a VALID record (not rejected as
/// "unhashed"/"missing"), and an arbitrary 32-byte value round-trips verbatim
/// (no canonicalization/normalization). An impl that special-cases the zero hash
/// or folds the hash through a domain check is caught.
#[test]
fn test_announce_revision_hash_opaque_identity() {
    // all-zero hash is a valid, non-special record.
    let zero = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: vec![0x01, 0x02], revision_hash: [0u8; 32] }],
    };
    let d0 = decode_availability_list(&zero.encode())
        .expect("KISS-ANNOUNCE-6.3-0008: an all-zero revision_hash MUST be a valid record");
    assert_eq!(d0.records[0].revision_hash, [0u8; 32]);

    // an arbitrary 32-byte value round-trips verbatim (no normalization).
    let mut arb = [0u8; 32];
    for (i, b) in arb.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(7).wrapping_add(3);
    }
    let list = AvailabilityList {
        list_version: 1,
        records: vec![AvailabilityRecord { structure_key: vec![0x09], revision_hash: arb }],
    };
    let d = decode_availability_list(&list.encode()).unwrap();
    assert_eq!(
        d.records[0].revision_hash, arb,
        "KISS-ANNOUNCE-6.3-0008: an opaque revision_hash MUST round-trip byte-verbatim"
    );
}

// NOTE: KISS-ANNOUNCE-6.3-0006 (consumer cache hit/miss logic) is intentionally
// left UNBACKED here — it governs a consumer's dedup comparator, and no such
// consumer-comparator surface exists in conformance/src. A test binding it to
// AvailabilityRecord's derived PartialEq would be inert (a codec drift does not
// fail it), so per the honest-teeth rule it is skipped until a real consumer
// surface exists rather than over-credited.

// ---- §6.4 contract-query protocol -------------------------------------------

/// Enforces KISS-ANNOUNCE-6.4-0011 — a contract or decline RESPONSE MUST echo
/// the `(structure_key, revision_hash)` identity it is answering for.
/// TEETH: one `Identity` is encoded into a CYRQ request, a CRSP response, and a
/// CDEC response; the identity block (every byte after the 4-byte tag, derived
/// from the identity-only CYRQ frame) MUST be BYTE-IDENTICAL across all three.
/// A CDEC that jumps straight to `decline_code` (dropping the echoed identity)
/// or a CRSP that omits the `revision_hash` breaks the cross-frame echo —
/// distinct from the per-frame goldens §6.4-0004 / §6.4-0007.
#[test]
fn test_announce_response_echoes_identity() {
    let identity = Identity { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: Some([0x11; 32]) };

    // CYRQ is tag(4) + identity ONLY, so the identity block == everything past the tag.
    let req = QueryRequest { identity: identity.clone() }.encode();
    let block = &req[4..];
    assert!(!block.is_empty());

    let crsp = ContractResponse { identity: identity.clone(), payload: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01] }.encode();
    let cdec = DeclineResponse { identity: identity.clone(), decline_code: decline_code::UNKNOWN_REVISION }.encode();

    assert_eq!(
        &crsp[4..4 + block.len()],
        block,
        "KISS-ANNOUNCE-6.4-0011: CRSP MUST echo the identity block byte-identically (incl. the revision_hash)"
    );
    assert_eq!(
        &cdec[4..4 + block.len()],
        block,
        "KISS-ANNOUNCE-6.4-0011: CDEC MUST echo the identity block byte-identically, not skip to decline_code"
    );
    // and the frames continue past the echoed block (payload / decline_code), so the
    // echo is a genuine prefix-after-tag, not the whole frame coinciding.
    assert!(crsp.len() > 4 + block.len(), "CRSP carries a payload after the echoed identity");
    assert!(cdec.len() > 4 + block.len(), "CDEC carries a decline_code after the echoed identity");
}
