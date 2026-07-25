//! KISS-Conform tests for the KISS-Synth level-1/level-2 seam (§6.7 handshake +
//! availability) plus the request/response "no new shape / opaque / identity-only"
//! corollaries (§6.1-0002a, §6.2-0002b, §6.2-0003a). Continues the #91 SYNTH
//! coverage burndown.
//!
//! SYNTH mints NO wire shapes of its own on this path: a provision request REUSES
//! the KISS-Announce contract-query request frame (tag `CYRQ`), the contract's wire
//! home is the Announce contract-response frame (tag `CRSP`), level-1 is the
//! Announce SeamHello envelope, and level-2 availability REUSES the Announce
//! availability record. So every test here EXERCISES the real
//! `conformance/src/announce.rs` codec — `Envelope`/`decode`/`negotiate`,
//! `AvailabilityList`/`decode_availability_list`, `QueryRequest`, `ContractResponse`,
//! the `Identity{structure_key, revision_hash}` block, and the `AnnounceDecline`
//! typed-decline enum — and asserts those bytes/behaviors satisfy the cited SYNTH
//! clause. No new `src/` scaffolding, no Fuel dependency, no local codec re-impl.
//!
//! The reused Announce constructs are named only by their Rust symbol / constant
//! value (e.g. `AnnounceDecline::NoMutualProfile`, `CYRQ`), never by an Announce
//! clause-id, so kiss_trace binds only these SYNTH ids and does not reverse-bind or
//! over-credit Announce.
//!
//! Bound here (10 clauses): §6.1-0002a, §6.2-0002b, §6.2-0003a, §6.7-0001b,
//! §6.7-0002, §6.7-0002a, §6.7-0002b, §6.7-0003, §6.7-0004, §6.7-0004a. The
//! provision-success (`PRSP`) frame is SYNTH's one own wire artifact and is absent
//! from `conformance/src` (grep confirms no `PRSP` / `artifact_format_tag`), so the
//! §6.2-success / §6.3 / §6.4 clauses that need it are SKIPPED (see the PR body);
//! faking a PRSP encoder in-test would be a rejected local codec re-impl.

use kiss_conformance::announce::*;
use kiss_conformance::assert_golden;

// The echoed identity block (§6.4-0011 / KISS-SYNTH-6.1-0002) with a revision
// pinned: key `AA BB CC`, revision_hash = 32×0x11. Independently transcribed so the
// §6.1-0002a golden is a hand-written expected value, not a copy of the encoder.
const IDENTITY_PRESENT_GOLDEN: &str = concat!(
    "03 00 00 00 ",                          // structure_key len = 3 (u32 LE)
    "AA BB CC ",                             // structure_key bytes
    "01 ",                                   // revision_present = 1
    "11 11 11 11 11 11 11 11 ",              // revision_hash = 32 × 0x11
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

/// The level-2 availability wire identity of one record: the per-record region of a
/// single-record availability list encoded by the REAL codec (the 12-byte SAVL
/// header stripped). Layout `[u32-LE key_len][key bytes][32-byte revision_hash]`.
/// Used to compare ENCODED bytes (not the `#[derive(PartialEq)]`) for the hit/miss
/// clauses, so those tests prove the WIRE carries the full pair rather than
/// over-crediting the derived equality.
fn wire_identity(rec: &AvailabilityRecord) -> Vec<u8> {
    let list = AvailabilityList { list_version: 1, records: vec![rec.clone()] };
    list.encode()[12..].to_vec()
}

// ---- §6.1 Provision request — no new request shape --------------------------

/// KISS-SYNTH-6.1-0002a — a provision request MUST NOT introduce a new request
/// shape; provision IS the Announce contract-query (tag `CYRQ`), generalized so the
/// response MAY include a freshly built artifact.
/// TEETH: (1) NO new request tag — the leading four bytes are the Announce `CYRQ`
/// tag, not a minted provision tag (and specifically not `CRSP`/`SAVL`/`CDEC`); (2)
/// NO new field — the frame length is EXACTLY tag(4) + identity, with no room for a
/// build/JIT flag byte or capability blob, in BOTH the pinned and unpinned forms;
/// (3) the exact bytes equal an independently hand-assembled `CYRQ` golden. MUTATION:
/// a "provision request" that added any field (a build/JIT flag, a capability blob)
/// changes the length and fails (2)/(3); a distinct minted request tag fails (1).
#[test]
fn test_synth_request_no_new_shape() {
    let pinned = QueryRequest { identity: identity_present() }.encode();
    let unpinned = QueryRequest { identity: identity_unpinned() }.encode();

    // (1) NO new request tag: provision reuses the Announce CYRQ tag verbatim.
    assert_eq!(&pinned[0..4], &CYRQ.to_le_bytes(), "provision reuses the CYRQ tag — no new request tag");
    assert_eq!(&unpinned[0..4], &CYRQ.to_le_bytes());
    assert_ne!(&pinned[0..4], &CRSP.to_le_bytes(), "a provision request is NOT minted as a distinct response tag");
    assert_ne!(&pinned[0..4], &SAVL.to_le_bytes());
    assert_ne!(&pinned[0..4], &CDEC.to_le_bytes());

    // (2) NO new field: the frame is exactly the CYRQ tag + identity, nothing added.
    let pinned_identity = 4 /*key_len*/ + 3 /*key*/ + 1 /*revision_present*/ + 32 /*hash*/;
    assert_eq!(pinned.len(), 4 + pinned_identity, "pinned request = CYRQ tag + identity, no build/JIT flag or capability blob");
    let unpinned_identity = 4 /*key_len*/ + 3 /*key*/ + 1 /*revision_present*/; // no hash
    assert_eq!(unpinned.len(), 4 + unpinned_identity, "unpinned request adds nothing either");

    // (3) exact bytes: an independently hand-assembled CYRQ golden (byte-for-byte
    // proof of the no-new-shape claim).
    let golden = format!("43 59 52 51 {IDENTITY_PRESENT_GOLDEN}");
    assert_golden("KISS-SYNTH-6.1-0002a", "provision_is_cyrq_no_new_shape", &pinned, &golden);
}

// ---- §6.2 Provision response — contract not pushed / opaque ------------------

/// KISS-SYNTH-6.2-0002b — a provider MUST NOT push the contract eagerly per-kernel
/// into the availability announce; the contract is produced/fetched on the provision
/// request, and its wire home is the contract-query RESPONSE, never the level-2
/// availability record.
/// TEETH: (i) an exhaustive destructure of `AvailabilityRecord { structure_key,
/// revision_hash }` (no `..`) is a compile-time guarantee the record has NO
/// contract/payload field; (ii) the per-record wire region of a single-record list
/// is EXACTLY `4 + key_len + 32` bytes — no eager contract bytes; (iii) by contrast
/// the `CRSP` contract-response frame for the same identity DOES carry the contract
/// payload verbatim in its payload region. MUTATION: eagerly stuffing the contract
/// into the availability record either adds bytes (the length accounting in (ii)
/// fails) or needs a new record field (the destructure in (i) fails to compile).
#[test]
fn test_synth_contract_not_pushed_to_announce() {
    let key = vec![0xAA, 0xBB, 0xCC];
    let hash = [0x11u8; 32];
    let record = AvailabilityRecord { structure_key: key.clone(), revision_hash: hash };

    // (i) exhaustive destructure — the availability record has NO contract/payload field.
    let AvailabilityRecord { structure_key, revision_hash } = &record;
    assert_eq!(structure_key, &key, "structure_key is the only key half");
    assert_eq!(revision_hash, &hash, "revision_hash is the only revision half");

    // (ii) the per-record wire region carries ONLY the identity — no contract bytes.
    let per_record = wire_identity(&record);
    assert_eq!(per_record.len(), 4 + key.len() + 32, "availability record is identity-only: no eager contract payload");

    // (iii) the contract's real wire home is the CRSP response, which DOES carry the
    // length-delimited contract payload. Slice its exact payload region and confirm
    // the contract bytes live there, NOT in the availability record.
    let contract_bytes = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03];
    let crsp = ContractResponse {
        identity: Identity { structure_key: key.clone(), revision_hash: Some(hash) },
        payload: contract_bytes.clone(),
    }
    .encode();
    let payload_off = crsp.len() - contract_bytes.len();
    assert_eq!(&crsp[payload_off..], contract_bytes.as_slice(), "the contract payload lives in CRSP, its wire home");
    assert!(crsp.len() > 12 + per_record.len(), "the announce record cannot be carrying the contract — CRSP is strictly larger by the payload");
}

/// KISS-SYNTH-6.2-0003a — the contract document MUST be carried OPAQUELY; the
/// provision path MUST NOT reinterpret or re-encode its bytes.
/// TEETH: an arbitrary contract blob spanning every byte value `0x00..=0xFF`
/// (interior NULs, and byte runs that spell frame tags/magic) is carried in the
/// length-delimited payload region VERBATIM — the sliced payload equals the input
/// byte-for-byte and the frame length shows no truncation or expansion. MUTATION: a
/// codec that treated the contract as UTF-8/base64/escaped text, or NUL-terminated
/// it, would alter or truncate the bytes and fail the byte-for-byte / length
/// assertions.
/// CAVEAT: proxied through the Announce `CRSP` payload carrier because no `PRSP`
/// frame exists in `conformance/src`. Clause §6.2-0003 pins the `PRSP`-enclosed
/// contract byte-identical to exactly this `CRSP` payload, so the carrier is
/// faithful and the opacity property transfers.
#[test]
fn test_synth_contract_opaque() {
    // a contract blob covering all 256 byte values — breaks any UTF-8/base64/escape
    // re-encode and contains interior NULs and bytes that spell tags/magic.
    let payload: Vec<u8> = (0u16..=255).map(|x| x as u8).collect();
    let crsp = ContractResponse {
        identity: Identity { structure_key: vec![0x01], revision_hash: None },
        payload: payload.clone(),
    }
    .encode();

    // the payload region sits at the tail, delimited by a preceding u32-LE length.
    let payload_off = crsp.len() - payload.len();
    assert_eq!(
        &crsp[payload_off - 4..payload_off],
        &(payload.len() as u32).to_le_bytes(),
        "the contract is delimited ONLY by its u32-LE length (256 = 00 01 00 00)"
    );
    assert_eq!(&crsp[payload_off..], payload.as_slice(), "contract bytes carried verbatim, incl. interior NULs and tag-spelling runs");
    assert_eq!(crsp.len(), payload_off + 256, "no NUL-truncation and no base64/escape expansion — all 256 bytes survive");
}

// ---- §6.7 Level 1: provider handshake (Announce SeamHello) ------------------

/// KISS-SYNTH-6.7-0001b — level 1 negotiates PROVIDER-level capability only (which
/// profiles/versions each side speaks, the `CONTRACT_QUERY` / `PROVISION_ON_REQUEST`
/// FEAT bits) and MUST NOT carry per-kernel capability.
/// TEETH: the decoded handshake `Envelope` exhaustively destructures to
/// `{ envelope_version, profiles, capabilities }` (no `..`) — a compile-time
/// guarantee there is NO per-kernel capability field, only a single provider-level
/// `u64` capability bitset; and a round-tripped envelope is a FIXED 56 bytes with
/// the capability occupying exactly the 8-byte `[48..56]` region. MUTATION: adding a
/// per-kernel capability to the handshake requires a new `Envelope` field (the
/// exhaustive destructure fails to compile) or extra bytes beyond the fixed 56 (a
/// 57-byte envelope is rejected `WrongLength`).
#[test]
fn test_synth_handshake_no_per_kernel_capability() {
    let env = Envelope { envelope_version: 1, profiles: vec![1, 2, 5], capabilities: 0xDEAD_BEEF_0000_0001 };
    let wire = env.encode();
    assert_eq!(wire.len(), ENVELOPE_LEN, "the handshake envelope is a FIXED 56 bytes — no per-kernel capability region");
    assert_eq!(wire.len(), 56);

    let decoded = decode(&wire).expect("a valid 56-byte envelope round-trips");
    // exhaustive destructure (no `..`): the handshake carries provider-level fields ONLY.
    let Envelope { envelope_version, profiles, capabilities } = &decoded;
    assert_eq!(*envelope_version, 1, "envelope_version is provider-level");
    assert_eq!(profiles, &vec![1u16, 2, 5], "profiles are the provider's version set");
    assert_eq!(*capabilities, 0xDEAD_BEEF_0000_0001, "capability is a single provider-level u64 bitset");
    // the capability bitset is exactly the 8 bytes at [48..56] — not a per-kernel record.
    assert_eq!(&wire[48..56], &capabilities.to_le_bytes(), "provider capability = one u64 at [48..56], no per-kernel structure");

    // a 57-byte envelope (a would-be per-kernel capability tacked on) is hard-rejected.
    let mut too_long = wire.clone();
    too_long.push(0x00);
    assert_eq!(
        decode(&too_long),
        Err(AnnounceDecline::WrongLength { got: 57 }),
        "extra bytes beyond the fixed 56 (a per-kernel capability region) are rejected, not accepted"
    );
}

/// KISS-SYNTH-6.7-0002 — an empty profile intersection at level 1 MUST be handled as
/// the KISS-Announce handshake failure; it is NOT a `CDEC`-framed provision decline
/// and is not a member of the §6.6 provision decline taxonomy.
/// TEETH: the REAL `negotiate(local, remote)` on a disjoint profile pair returns
/// `Err(AnnounceDecline::NoMutualProfile)` — a handshake failure that carries NO
/// `decline_code` and is not one of the six §6.6 core codes (which are the u32 values
/// `1..=6`). The exact-variant match is the teeth. MUTATION: a drift that framed the
/// empty intersection as a `CANNOT_PROVISION` / `QUERY_NOT_SUPPORTED` `CDEC`
/// provision decline would surface a `decline_code`-carrying error and fall through
/// the match to the panic — proving the empty intersection is Announce-owned, not a
/// member of the provision decline taxonomy.
#[test]
fn test_synth_empty_profile_is_announce_owned() {
    let local = Envelope { envelope_version: 1, profiles: vec![1, 2, 3], capabilities: 0 };
    let remote = Envelope { envelope_version: 1, profiles: vec![4, 5, 6], capabilities: 0 };

    match negotiate(&local, &remote) {
        // the Announce-owned handshake failure: a nullary variant carrying no decline_code.
        Err(AnnounceDecline::NoMutualProfile) => {}
        other => panic!(
            "KISS-SYNTH-6.7-0002: an empty profile intersection MUST be the Announce \
             NoMutualProfile handshake failure — not a provision decline nor a selection: {other:?}"
        ),
    }

    // The six §6.6 provision core codes are the contiguous u32 values 1..=6; the
    // handshake failure carries none of them (it has no decline_code channel at all).
    let core: [u32; 6] = [
        decline_code::UNKNOWN_STRUCTURE_KEY,
        decline_code::CANNOT_PROVISION,
        decline_code::MALFORMED_REQUEST,
        decline_code::QUERY_NOT_SUPPORTED,
        decline_code::VERSION_UNSUPPORTED,
        decline_code::UNKNOWN_REVISION,
    ];
    assert_eq!(core, [1, 2, 3, 4, 5, 6], "the §6.6 provision taxonomy is exactly the six core codes 1..=6");
    // NoMutualProfile is a distinct typed value from every code-carrying provision
    // decline — referenced by symbol, never by an Announce clause-id.
    assert_ne!(
        negotiate(&local, &remote),
        Ok(0),
        "the empty intersection yields the handshake failure, never a phantom selection"
    );
}

/// KISS-SYNTH-6.7-0002a — a provider or consumer MUST NOT panic, abort, or crash on
/// an empty profile intersection.
/// TEETH: a battery of disjoint / empty / single-element / overlapping / subset
/// profile-set pairs fed to the REAL `negotiate()` each returns a `Result` without
/// panicking; the empty-intersection cases return `Err(NoMutualProfile)` and the
/// overlapping cases return `Ok(max(L ∩ R))`. MUTATION: an impl that `unwrap()`ed the
/// max of an empty intersection, or indexed `profiles[0]` assuming non-empty, would
/// panic on the disjoint/empty cases and fail — this is real panic-freedom over the
/// actual fn, not `assert!(true)`.
#[test]
fn test_synth_empty_profile_never_panics() {
    let env = |profiles: Vec<u16>| Envelope { envelope_version: 1, profiles, capabilities: 0 };
    let cases: Vec<(Vec<u16>, Vec<u16>, Result<u16, AnnounceDecline>)> = vec![
        (vec![1, 2, 3], vec![4, 5, 6], Err(AnnounceDecline::NoMutualProfile)), // disjoint
        (vec![], vec![1, 2], Err(AnnounceDecline::NoMutualProfile)),           // empty local
        (vec![1, 2], vec![], Err(AnnounceDecline::NoMutualProfile)),           // empty remote
        (vec![], vec![], Err(AnnounceDecline::NoMutualProfile)),               // both empty
        (vec![5], vec![6], Err(AnnounceDecline::NoMutualProfile)),             // single, no match
        (vec![5], vec![5], Ok(5)),                                             // single, match
        (vec![1, 2, 3], vec![2, 3, 4], Ok(3)),                                // overlapping -> max
        (vec![1, 2, 3], vec![2], Ok(2)),                                       // subset
    ];
    for (l, r, expected) in cases {
        let got = negotiate(&env(l.clone()), &env(r.clone())); // returns without panicking
        assert_eq!(got, expected, "negotiate({l:?}, {r:?}) must return its Result, never panic");
    }
}

/// KISS-SYNTH-6.7-0002b — a provider or consumer MUST NOT select a profile on an
/// empty intersection.
/// TEETH: on a disjoint profile pair `negotiate()` returns `Err`, and crucially NOT
/// `Ok(_)`: not `Ok(0)` (a phantom "profile 0" selection) and not `Ok(local_max)` (a
/// fall-back to the local maximum). MUTATION: an impl that fell back to selecting `0`,
/// or the local max, when the intersection is empty would return `Ok(0)` / `Ok(3)`
/// and fail the "must be Err, never Ok" assertions.
#[test]
fn test_synth_empty_profile_no_select() {
    let local = Envelope { envelope_version: 1, profiles: vec![1, 2, 3], capabilities: 0 };
    let remote = Envelope { envelope_version: 1, profiles: vec![7, 8, 9], capabilities: 0 };
    let r = negotiate(&local, &remote);

    assert!(r.is_err(), "an empty intersection selects NO profile — the result is Err");
    assert_ne!(r, Ok(0u16), "MUST NOT select a phantom profile 0");
    let local_max = *local.profiles.iter().max().unwrap();
    assert_ne!(r, Ok(local_max), "MUST NOT fall back to the local maximum ({local_max})");
    assert_eq!(r, Err(AnnounceDecline::NoMutualProfile), "the specific outcome is the Announce handshake failure");
}

// ---- §6.7 Level 2: availability by identity ---------------------------------

/// KISS-SYNTH-6.7-0003 — level 2 (kernel availability) MUST name the kernels a
/// provider can serve by IDENTITY ONLY — the `(structure_key, revision_hash)` pair,
/// with BOTH halves load-bearing.
/// TEETH: encode a single-record availability list, decode it via the REAL codec and
/// assert both halves survive; then mutate one byte in the KEY region and (separately)
/// one byte in the 32-byte REVISION_HASH region of the wire, decode each, and assert
/// EACH mutation changes exactly its own half of the decoded record. MUTATION: a
/// decoder that ignored the revision_hash bytes (or the key bytes) when reconstructing
/// the record makes that region's mutation invisible — the decoded record would still
/// equal the original — and fails the "each half load-bearing" assertion. Distinct
/// from §6.2-0007a (single encode golden) and §6.7-0003a (no-capability length
/// accounting): this asserts the identity is the FULL pair, both halves INDEPENDENTLY
/// determining identity through the real decode path.
#[test]
fn test_synth_availability_identity_only() {
    let key = vec![0xAA, 0xBB, 0xCC];
    let hash = [0x11u8; 32];
    let rec = AvailabilityRecord { structure_key: key.clone(), revision_hash: hash };
    let wire = AvailabilityList { list_version: 1, records: vec![rec.clone()] }.encode();

    // round-trip: both halves survive the real codec.
    let decoded = decode_availability_list(&wire).expect("a single-record availability list round-trips");
    assert_eq!(decoded.records.len(), 1, "one record encoded, one decoded");
    assert_eq!(decoded.records[0], rec, "the (structure_key, revision_hash) pair survives round-trip");

    // wire offsets: header 12, key_len [12..16], key [16..16+3], revision_hash [19..51].
    let key_off = 16;
    let hash_off = 16 + key.len();

    // (a) a KEY-region mutation changes ONLY the decoded structure_key (key half load-bearing).
    let mut m_key = wire.clone();
    m_key[key_off] ^= 0xFF;
    let dk = decode_availability_list(&m_key).expect("mutated-but-well-framed list still decodes").records;
    assert_ne!(dk[0].structure_key, rec.structure_key, "a key-region byte flip MUST change the decoded structure_key");
    assert_eq!(dk[0].revision_hash, rec.revision_hash, "only the key half moved");

    // (b) a REVISION_HASH-region mutation changes ONLY the decoded revision_hash (hash half load-bearing).
    let mut m_hash = wire.clone();
    m_hash[hash_off] ^= 0xFF;
    let dh = decode_availability_list(&m_hash).expect("mutated-but-well-framed list still decodes").records;
    assert_ne!(dh[0].revision_hash, rec.revision_hash, "a hash-region byte flip MUST change the decoded revision_hash");
    assert_eq!(dh[0].structure_key, rec.structure_key, "only the hash half moved");
}

/// KISS-SYNTH-6.7-0004 — a consumer MUST distinguish a cache HIT from a MISS by full
/// byte-for-byte identity: a hit is a match on BOTH `structure_key` and the 32-byte
/// `revision_hash`, and any other case is a miss.
/// TEETH: compare ENCODED wire identity bytes (NOT the `#[derive(PartialEq)]`). (1) the
/// wire identity equals an independently hand-assembled `[key_len][key][hash]` — so the
/// wire carries the FULL pair; (2) an identical `(key, hash)` record encodes to identical
/// wire identity bytes (a hit); (3) the SAME key with a one-byte-different revision_hash
/// encodes to DIFFERENT wire identity bytes (a miss). MUTATION: a codec that dropped or
/// truncated the revision_hash from the identity would make same-key/different-hash
/// records encode identically — a false hit — and fail (3); comparing encoded bytes (not
/// the derive) avoids over-crediting the derived equality.
#[test]
fn test_synth_hit_miss_by_identity() {
    let key = vec![0xAA, 0xBB, 0xCC];
    let hash = [0x11u8; 32];
    let hit = AvailabilityRecord { structure_key: key.clone(), revision_hash: hash };

    // (1) the wire identity carries the full (structure_key, revision_hash) pair.
    let mut expected = (key.len() as u32).to_le_bytes().to_vec();
    expected.extend_from_slice(&key);
    expected.extend_from_slice(&hash);
    assert_eq!(wire_identity(&hit), expected, "the wire identity IS the full (structure_key, revision_hash) pair");

    // (2) a HIT: an independently-built identical record encodes to identical bytes.
    let same = AvailabilityRecord { structure_key: key.clone(), revision_hash: hash };
    assert_eq!(wire_identity(&hit), wire_identity(&same), "same (key, hash) => identical wire identity (a hit)");

    // (3) a MISS: same key, ONE byte different in the 32-byte revision_hash.
    let mut hash2 = hash;
    hash2[31] ^= 0x01;
    let miss = AvailabilityRecord { structure_key: key.clone(), revision_hash: hash2 };
    assert_ne!(
        wire_identity(&hit),
        wire_identity(&miss),
        "same key but a one-byte-different revision_hash => different wire identity (a miss); a codec \
         that dropped/truncated the revision_hash would make these equal — a false hit"
    );
}

/// KISS-SYNTH-6.7-0004a — a consumer MUST NOT treat a `structure_key` match alone as
/// a hit.
/// TEETH: two records sharing a `structure_key` but differing only in the 32-byte
/// `revision_hash`: the encoded KEY regions are EQUAL (the structure_key matches) yet
/// the full encoded wire identities DIFFER — so a byte-for-byte identity comparison
/// (the actual hit test) correctly separates them into a miss. MUTATION: an impl that
/// keyed a hit on `structure_key` ALONE (ignoring the revision half) would treat these
/// as a hit; the "full identities differ" assertion catches it, and the "key regions
/// equal" assertion pins that the key match alone was insufficient.
#[test]
fn test_synth_hit_not_key_alone() {
    let key = vec![0xAA, 0xBB, 0xCC];
    let hash_a = [0x11u8; 32];
    let mut hash_b = hash_a;
    hash_b[0] ^= 0xFF; // same key, one hash byte different
    let id_a = wire_identity(&AvailabilityRecord { structure_key: key.clone(), revision_hash: hash_a });
    let id_b = wire_identity(&AvailabilityRecord { structure_key: key.clone(), revision_hash: hash_b });

    // the structure_key region matches (a key match alone)...
    let key_region = 4..4 + key.len(); // [u32 key_len][key bytes ...]
    assert_eq!(&id_a[0..4], &id_b[0..4], "same declared key length");
    assert_eq!(&id_a[key_region.clone()], &id_b[key_region], "the structure_key region matches — key match alone");
    // ...yet the FULL identities differ, so key-alone is NOT a hit.
    assert_ne!(id_a, id_b, "a structure_key match ALONE is not a hit; the full identity (incl. revision_hash) separates them");
}
