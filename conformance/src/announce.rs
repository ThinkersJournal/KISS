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
