//! Reference encoder/decoder for the KISS-Announce 56-byte handshake envelope
//! (Announce §6.1), plus the hard-reject reader discipline (§6.2).
//!
//! Fixed-size POD, all multi-byte fields little-endian. Layout (§6.1):
//! ```text
//!   0  4  magic            u32 LE  == 0x4D414553 ("SEAM", wire 53 45 41 4D)
//!   4  1  envelope_version u8      1
//!   5  3  reserved0        u8[3]   MBZ
//!   8  2  profiles_len     u16 LE  <= 16
//!  10 32  profiles         u16[16] live entries >=1 ascending, trailing 0
//!  42  6  reserved1        u8[6]   MBZ
//!  48  8  capabilities     u64 LE
//!         total            56 bytes
//! ```

/// The envelope magic (§6.1-0004): the u32-LE value of wire bytes `53 45 41 4D`.
pub const MAGIC: u32 = 0x4D41_4553;
/// The fixed envelope length (§6.1-0001).
pub const ENVELOPE_LEN: usize = 56;
/// The maximum number of live profiles (§6.1-0007).
pub const MAX_PROFILES: usize = 16;

/// A decoded handshake envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub envelope_version: u8,
    pub profiles: Vec<u16>,
    pub capabilities: u64,
}

/// A typed decline from the POD reader (§6.2 hard-reject; never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnounceDecline {
    WrongLength { got: usize },
    BadMagic { got: u32 },
    UnsupportedVersion { got: u8 },
    ReservedNonZero { region: &'static str },
    ProfilesLenOverflow { got: u16 },
    ZeroLiveProfile,
    ProfilesNotStrictlyAscending,
    TrailingProfileNonZero,
    /// Version negotiation found no profile common to both live-profile sets (§7.1-0002).
    NoMutualProfile,
    /// A 4-byte message tag did not match the expected code (§2.4, §6.3-0011, §6.4).
    BadTag { region: &'static str, got: u32 },
    /// An availability list carried an unsupported `list_version` (§6.3-0012).
    ListVersionUnsupported { got: u8 },
    /// Input ended before a fixed-width field could be read — the truncation guard
    /// that keeps a reader from an out-of-bounds slice (§6.4-0006).
    Truncated { region: &'static str },
    /// A declared `structure_key` byte-length fell outside `[1, 4096]` (§6.3-0009, §6.4-0001).
    StructureKeyLenOutOfRange { got: u32 },
    /// A declared `record_count` exceeded `MAX_AVAILABILITY_RECORDS` (§6.3-0010).
    RecordCountOverflow { got: u32 },
    /// A `revision_present` flag was neither `0` nor `1` (§6.4-0001).
    BadRevisionPresent { got: u8 },
}

// ---- §6.3 Availability list + §6.4 contract-query frames ---------------------
//
// Every 4-character code is four ASCII bytes in wire order (first char at the
// lowest offset); reading those same four bytes as a little-endian u32 yields the
// numeric constant (§2.4). We store the u32 and serialize with `to_le_bytes()`,
// exactly as the envelope `MAGIC` does — so a big-endian (`htonl` reflex) tag
// write is a *wrong* implementation the golden vectors catch.

/// Availability-list tag `SAVL` (§2.4, §6.3-0011): wire bytes `53 41 56 4C`.
pub const SAVL: u32 = 0x4C56_4153;
/// Contract-query request tag `CYRQ` (§2.4, §6.4-0001): wire bytes `43 59 52 51`.
pub const CYRQ: u32 = 0x5152_5943;
/// Contract response tag `CRSP` (§2.4, §6.4-0004): wire bytes `43 52 53 50`.
pub const CRSP: u32 = 0x5053_5243;
/// Decline response tag `CDEC` (§2.4, §6.4-0007): wire bytes `43 44 45 43`.
pub const CDEC: u32 = 0x4345_4443;

/// Inclusive upper bound on a `structure_key` byte-length (§6.3-0009, §6.4-0001).
pub const MAX_STRUCTURE_KEY_LEN: u32 = 4096;
/// Upper bound on an availability list's `record_count` (§6.3-0010, `2^20`).
pub const MAX_AVAILABILITY_RECORDS: u32 = 1 << 20;

/// The pinned little-endian u32 `decline_code` values (§6.4-0009).
pub mod decline_code {
    pub const UNKNOWN_STRUCTURE_KEY: u32 = 0x0000_0001;
    pub const CANNOT_PROVISION: u32 = 0x0000_0002;
    pub const MALFORMED_REQUEST: u32 = 0x0000_0003;
    pub const QUERY_NOT_SUPPORTED: u32 = 0x0000_0004;
    pub const VERSION_UNSUPPORTED: u32 = 0x0000_0005;
    pub const UNKNOWN_REVISION: u32 = 0x0000_0006;
}

/// One availability record: the identity pair `(structure_key, revision_hash)`
/// and nothing else (§6.3-0001). `revision_hash` is exactly 32 bytes (§6.3-0003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailabilityRecord {
    pub structure_key: Vec<u8>,
    pub revision_hash: [u8; 32],
}

/// A decoded availability list (§6.3): tag, `list_version` block, `record_count`,
/// then the records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailabilityList {
    pub list_version: u8,
    pub records: Vec<AvailabilityRecord>,
}

impl AvailabilityList {
    /// Serialize per §6.3-0011/§6.3-0012/§6.3-0005:
    /// `SAVL` tag, `list_version` (1 byte) + 3 MBZ bytes, u32-LE `record_count`,
    /// then each record as {u32-LE `structure_key` length, key bytes, 32-byte
    /// `revision_hash`}. All multi-byte fields little-endian.
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&SAVL.to_le_bytes()); // [0] tag  (LE => 53 41 56 4C)
        b.push(self.list_version); // [4] list_version
        b.extend_from_slice(&[0u8; 3]); // [5] 3 MBZ bytes
        b.extend_from_slice(&(self.records.len() as u32).to_le_bytes()); // [8] record_count
        for r in &self.records {
            b.extend_from_slice(&(r.structure_key.len() as u32).to_le_bytes()); // u32-LE key_len
            b.extend_from_slice(&r.structure_key);
            b.extend_from_slice(&r.revision_hash); // 32 bytes
        }
        b
    }
}

/// Decode + hard-reject an availability list (§6.3). Every malformation yields a
/// typed decline; no panic, abort, or out-of-bounds read. The `record_count` is
/// *not* pre-allocated on (§6.3-0010): records are pushed as they are validated.
pub fn decode_availability_list(bytes: &[u8]) -> Result<AvailabilityList, AnnounceDecline> {
    // tag + list_version block = 8 bytes minimum
    if bytes.len() < 8 {
        return Err(AnnounceDecline::Truncated { region: "savl_header" });
    }
    let tag = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if tag != SAVL {
        return Err(AnnounceDecline::BadTag { region: "savl", got: tag });
    }
    let list_version = bytes[4];
    if list_version != 1 {
        return Err(AnnounceDecline::ListVersionUnsupported { got: list_version });
    }
    if bytes[5..8] != [0, 0, 0] {
        return Err(AnnounceDecline::ReservedNonZero { region: "savl_mbz" });
    }
    if bytes.len() < 12 {
        return Err(AnnounceDecline::Truncated { region: "record_count" });
    }
    let record_count = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if record_count > MAX_AVAILABILITY_RECORDS {
        return Err(AnnounceDecline::RecordCountOverflow { got: record_count });
    }
    let mut off = 12usize;
    let mut records = Vec::new(); // NOT sized to record_count (§6.3-0010)
    for _ in 0..record_count {
        if bytes.len() < off + 4 {
            return Err(AnnounceDecline::Truncated { region: "key_len" });
        }
        let key_len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        off += 4;
        if key_len < 1 || key_len > MAX_STRUCTURE_KEY_LEN {
            return Err(AnnounceDecline::StructureKeyLenOutOfRange { got: key_len });
        }
        let key_len = key_len as usize;
        if bytes.len() < off + key_len {
            // checked before allocating on the unchecked length (§6.3-0009)
            return Err(AnnounceDecline::Truncated { region: "structure_key" });
        }
        let structure_key = bytes[off..off + key_len].to_vec();
        off += key_len;
        if bytes.len() < off + 32 {
            return Err(AnnounceDecline::Truncated { region: "revision_hash" });
        }
        let mut revision_hash = [0u8; 32];
        revision_hash.copy_from_slice(&bytes[off..off + 32]);
        off += 32;
        records.push(AvailabilityRecord { structure_key, revision_hash });
    }
    Ok(AvailabilityList { list_version, records })
}

/// The echoed `(structure_key, revision_hash)` identity block shared by a
/// contract-query request (§6.4-0001) and a response (§6.4-0011): a u32-LE
/// `structure_key` length, the key bytes, a 1-byte `revision_present` flag, and —
/// only when the flag is `1` — the 32-byte `revision_hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub structure_key: Vec<u8>,
    pub revision_hash: Option<[u8; 32]>,
}

impl Identity {
    fn encode_into(&self, b: &mut Vec<u8>) {
        b.extend_from_slice(&(self.structure_key.len() as u32).to_le_bytes());
        b.extend_from_slice(&self.structure_key);
        b.push(if self.revision_hash.is_some() { 1 } else { 0 });
        if let Some(h) = &self.revision_hash {
            b.extend_from_slice(h);
        }
    }

    /// Decode an identity block starting at `off`; returns the block and the new
    /// offset. Truncation and out-of-range length yield a typed decline, never a
    /// panic or OOB slice (§6.4-0006).
    fn decode_at(bytes: &[u8], mut off: usize) -> Result<(Identity, usize), AnnounceDecline> {
        if bytes.len() < off + 4 {
            return Err(AnnounceDecline::Truncated { region: "structure_key_len" });
        }
        let key_len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        off += 4;
        if key_len < 1 || key_len > MAX_STRUCTURE_KEY_LEN {
            return Err(AnnounceDecline::StructureKeyLenOutOfRange { got: key_len });
        }
        let key_len = key_len as usize;
        if bytes.len() < off + key_len {
            return Err(AnnounceDecline::Truncated { region: "structure_key" });
        }
        let structure_key = bytes[off..off + key_len].to_vec();
        off += key_len;
        if bytes.len() < off + 1 {
            return Err(AnnounceDecline::Truncated { region: "revision_present" });
        }
        let revision_present = bytes[off];
        off += 1;
        // MUST be exactly 0 or 1 — never a truthy `flag != 0` test (§6.4-0001).
        if revision_present > 1 {
            return Err(AnnounceDecline::BadRevisionPresent { got: revision_present });
        }
        let revision_hash = if revision_present == 1 {
            if bytes.len() < off + 32 {
                return Err(AnnounceDecline::Truncated { region: "revision_hash" });
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
}

/// A contract-query request (§6.4-0001): `CYRQ` tag then the identity block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRequest {
    pub identity: Identity,
}

impl QueryRequest {
    /// Serialize per §6.4-0001: `CYRQ` tag, u32-LE `structure_key` length, key
    /// bytes, 1-byte `revision_present`, and the 32-byte `revision_hash` iff present.
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&CYRQ.to_le_bytes());
        self.identity.encode_into(&mut b);
        b
    }
}

/// Decode + hard-reject a contract-query request (§6.4-0001, §6.4-0006). A
/// truncated request (e.g. cut off mid-`revision_hash`) yields a typed
/// [`AnnounceDecline::Truncated`] — never a panic or out-of-bounds slice.
pub fn decode_query_request(bytes: &[u8]) -> Result<QueryRequest, AnnounceDecline> {
    if bytes.len() < 4 {
        return Err(AnnounceDecline::Truncated { region: "cyrq_tag" });
    }
    let tag = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if tag != CYRQ {
        return Err(AnnounceDecline::BadTag { region: "cyrq", got: tag });
    }
    let (identity, _off) = Identity::decode_at(bytes, 4)?;
    Ok(QueryRequest { identity })
}

/// A contract response (§6.4-0004): `CRSP` tag, echoed identity, u32-LE payload
/// byte-length, then that many bytes of the KISS-Contract document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractResponse {
    pub identity: Identity,
    pub payload: Vec<u8>,
}

impl ContractResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&CRSP.to_le_bytes());
        self.identity.encode_into(&mut b);
        b.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        b.extend_from_slice(&self.payload);
        b
    }
}

/// A decline response (§6.4-0007): `CDEC` tag, echoed identity, u32-LE
/// `decline_code` (§6.4-0009).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclineResponse {
    pub identity: Identity,
    pub decline_code: u32,
}

impl DeclineResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&CDEC.to_le_bytes());
        self.identity.encode_into(&mut b);
        b.extend_from_slice(&self.decline_code.to_le_bytes()); // u32-LE, not u16
        b
    }
}

impl Envelope {
    /// Serialize to exactly 56 bytes (§6.1-0002). The caller supplies a valid
    /// profile set (≤16, ascending, each ≥1); this is asserted in debug builds.
    pub fn encode(&self) -> Vec<u8> {
        debug_assert!(self.profiles.len() <= MAX_PROFILES);
        debug_assert!(self.profiles.windows(2).all(|w| w[0] < w[1]), "profiles must be strictly ascending");
        debug_assert!(self.profiles.iter().all(|&p| p >= 1), "live profiles must be >= 1");

        let mut b = vec![0u8; ENVELOPE_LEN];
        b[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        b[4] = self.envelope_version;
        // reserved0 (5..8) already zero
        b[8..10].copy_from_slice(&(self.profiles.len() as u16).to_le_bytes());
        for (i, &p) in self.profiles.iter().enumerate() {
            let off = 10 + i * 2;
            b[off..off + 2].copy_from_slice(&p.to_le_bytes());
        }
        // profiles[len..16] and reserved1 (42..48) already zero
        b[48..56].copy_from_slice(&self.capabilities.to_le_bytes());
        b
    }
}

/// Decode + hard-reject a 56-byte envelope (§6.1, §6.2). Every malformation
/// yields a typed decline; no panic, abort, or out-of-bounds read.
pub fn decode(bytes: &[u8]) -> Result<Envelope, AnnounceDecline> {
    if bytes.len() != ENVELOPE_LEN {
        return Err(AnnounceDecline::WrongLength { got: bytes.len() });
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if magic != MAGIC {
        return Err(AnnounceDecline::BadMagic { got: magic });
    }
    let version = bytes[4];
    if version != 1 {
        return Err(AnnounceDecline::UnsupportedVersion { got: version });
    }
    if bytes[5..8] != [0, 0, 0] {
        return Err(AnnounceDecline::ReservedNonZero { region: "reserved0" });
    }
    let profiles_len = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
    if profiles_len as usize > MAX_PROFILES {
        return Err(AnnounceDecline::ProfilesLenOverflow { got: profiles_len });
    }
    let mut profiles = Vec::with_capacity(profiles_len as usize);
    for i in 0..MAX_PROFILES {
        let off = 10 + i * 2;
        let p = u16::from_le_bytes(bytes[off..off + 2].try_into().unwrap());
        if i < profiles_len as usize {
            if p == 0 {
                return Err(AnnounceDecline::ZeroLiveProfile);
            }
            profiles.push(p);
        } else if p != 0 {
            return Err(AnnounceDecline::TrailingProfileNonZero);
        }
    }
    if profiles.windows(2).any(|w| w[0] >= w[1]) {
        return Err(AnnounceDecline::ProfilesNotStrictlyAscending);
    }
    if bytes[42..48] != [0u8; 6] {
        return Err(AnnounceDecline::ReservedNonZero { region: "reserved1" });
    }
    let capabilities = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
    Ok(Envelope { envelope_version: version, profiles, capabilities })
}

// ---- §7.1 Version-negotiation algorithm -------------------------------------

/// Negotiate the mutually-highest live profile (§7.1-0001, §7.1-0002).
///
/// `local` and `remote` carry the live-profile sets `L` and `R` (each the nonzero
/// `profiles[0..profiles_len]` of the respective envelope, per §7.1). The negotiated
/// profile is `max(L ∩ R)` — the highest integer present in both sets (§7.1-0001).
/// If the intersection is empty, this returns a typed
/// [`AnnounceDecline::NoMutualProfile`] and never panics, aborts, or selects a
/// profile (§7.1-0002).
pub fn negotiate(local: &Envelope, remote: &Envelope) -> Result<u16, AnnounceDecline> {
    local
        .profiles
        .iter()
        .copied()
        .filter(|p| remote.profiles.contains(p))
        .max()
        .ok_or(AnnounceDecline::NoMutualProfile)
}

// ---- Golden handshake-frame reference builders + the machine-readable artifact -------------
//
// The §2.5 worked-example handshake frames are built ONCE, here in the library, so the golden
// bytes have a SINGLE definition (KISS-Conform §4.3): `announce_golden.rs` asserts them against the
// §2.x appendix hex (appendix-agreement), `announce_frames.rs` reuses the envelope, and
// `emit_announce_vectors_json` renders them into the foreign-reader artifact. Before this, the
// reference envelope was defined in BOTH test files — the shadowing hazard #469 hit with g1_region.

/// §6.1-0002 reference envelope: version 1, one profile {1}, caps EXT[0..5]|FEAT[32]|FEAT[33].
pub fn reference_envelope() -> Envelope {
    Envelope { envelope_version: 1, profiles: vec![1], capabilities: 0x0000_0003_0000_003F }
}

/// §6.3-0005 reference availability list: two records (keys AA BB CC / DE AD).
pub fn reference_availability_list() -> AvailabilityList {
    AvailabilityList {
        list_version: 1,
        records: vec![
            AvailabilityRecord { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: [0x11; 32] },
            AvailabilityRecord { structure_key: vec![0xDE, 0xAD], revision_hash: [0x22; 32] },
        ],
    }
}

/// §6.4-0011 reference identity block, echoed by CYRQ/CRSP/CDEC: key AA BB CC, revision present.
pub fn reference_identity() -> Identity {
    Identity { structure_key: vec![0xAA, 0xBB, 0xCC], revision_hash: Some([0x11; 32]) }
}

/// §6.4-0001 reference contract-query request.
pub fn reference_cyrq() -> QueryRequest {
    QueryRequest { identity: reference_identity() }
}

/// §6.4-0004 reference contract response (5-byte payload).
pub fn reference_crsp() -> ContractResponse {
    ContractResponse { identity: reference_identity(), payload: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01] }
}

/// §6.4-0007 reference decline response (decline_code UNKNOWN_REVISION).
pub fn reference_cdec() -> DeclineResponse {
    DeclineResponse { identity: reference_identity(), decline_code: decline_code::UNKNOWN_REVISION }
}

/// Emit `conformance/corpus/announce_vectors.json` — the machine-readable golden vector set for the
/// KISS-Announce wire handshake, generated from THIS module (the reference codec), mirroring
/// `contract::emit_contract_vectors_json`. A foreign (non-Rust) reader byte-diffs each frame against
/// its own encoder to check endianness, field width and structure padding — the §5.3 condition-2
/// obligation a Markdown appendix cannot be executed against.
pub fn emit_announce_vectors_json() -> String {
    fn hexs(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
    }
    let frames: [(&str, &str, Vec<u8>); 5] = [
        ("reference_envelope", "KISS-ANNOUNCE-6.1-0002", reference_envelope().encode()),
        ("availability_list", "KISS-ANNOUNCE-6.3-0005", reference_availability_list().encode()),
        ("cyrq_request", "KISS-ANNOUNCE-6.4-0001", reference_cyrq().encode()),
        ("crsp_response", "KISS-ANNOUNCE-6.4-0004", reference_crsp().encode()),
        ("cdec_response", "KISS-ANNOUNCE-6.4-0007", reference_cdec().encode()),
    ];
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"schema\": \"kiss-announce-vectors-v1.json\",\n");
    s.push_str("  \"generated_from\": \"conformance/src/announce.rs::emit_announce_vectors_json (the reference codec)\",\n");
    s.push_str("  \"spec_reference\": \"spec/announce.md \u{00a7}2.5 worked handshake example (informative) + the \u{00a7}6.1/\u{00a7}6.3/\u{00a7}6.4 framing clauses\",\n");
    s.push_str("  \"scope_note\": \"DIFFABLE, not COVERED: a foreign reader byte-diffs each frame's exact wire bytes (endianness, field width, padding) against its own encoder; this does NOT cover the untested KISS-Announce clauses or the reader/decline paths.\",\n");
    s.push_str("  \"population\": \"the FIVE \u{00a7}2.5 worked-example POSITIVE frames only: the 56-byte envelope, the availability list, and the CYRQ/CRSP/CDEC request-response frames. NOT rendered here: malformed/decline vectors (announce_golden.rs exercises the reader's typed declines in Rust), envelopes with >1 profile or other capability sets, and any frame type absent from \u{00a7}2.5 — that is the burn-down this artifact makes measurable, not part of it.\",\n");
    s.push_str("  \"frames\": [\n");
    for (i, (name, clause, bytes)) in frames.iter().enumerate() {
        let comma = if i + 1 < frames.len() { "," } else { "" };
        s.push_str("    {\n");
        s.push_str(&format!("      \"name\": \"{name}\",\n"));
        s.push_str(&format!("      \"clause\": \"{clause}\",\n"));
        s.push_str(&format!("      \"byte_length\": {},\n", bytes.len()));
        s.push_str(&format!("      \"bytes_hex\": \"{}\"\n", hexs(bytes)));
        s.push_str(&format!("    }}{comma}\n"));
    }
    s.push_str("  ]\n");
    s.push_str("}\n");
    s
}