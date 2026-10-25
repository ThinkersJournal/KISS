// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference encoder/decoder for the KISS-Synth provision-success frame `PRSP`
//! (Synth §6.4-0001b) — KISS-Synth's one own wire frame. A provision request
//! (`CYRQ`) and a provision decline (`CDEC`) are KISS-Announce frames reused
//! verbatim and live in [`crate::announce`].
//!
//! All multi-byte fields little-endian. Layout, in wire order:
//! ```text
//!   4        tag                 "PRSP" = 50 52 53 50
//!   var      identity block      (structure_key, op_identity, revision_hash) as Announce
//!                                §6.4-0011, with revision_present == 1 always
//!   4        artifact_format_tag opaque, never 00 00 00 00
//!   8        artifact_len        u64 LE
//!   var      artifact            artifact_len bytes
//!   4        contract_len        u32 LE
//!   var      contract            contract_len bytes (an opaque KISS-Contract document)
//! ```
//! The contract is carried opaquely: this codec never inspects, re-encodes or
//! validates its bytes (§6.2-0003a/-0003b); its own framing is read by
//! [`crate::contract::read_document`], independently (§6.2-0004).

use crate::announce::{AnnounceDecline, ContractResponse, Identity, CRSP};

/// The provision-success tag `PRSP` (§6.4-0001b): the u32-LE value of wire bytes
/// `50 52 53 50`.
pub const PRSP: u32 = 0x5053_5250;

/// A typed decline from the `PRSP` codec. Never a panic, never an out-of-bounds slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynthDecline {
    /// Input ended before a fixed-width field or a declared region could be read.
    Truncated { region: &'static str },
    /// The leading tag is neither `PRSP` nor `CRSP`.
    BadTag { got: u32 },
    /// The leading tag is `CRSP`: a bare contract-only response is not a provision success (§6.4-0001c).
    ContractOnlyResponse,
    /// The echoed identity block failed Announce's identity framing (§6.4-0011).
    Identity(AnnounceDecline),
    /// The echoed identity carried `revision_present == 0` (§6.3-0006 / §6.3-0006a).
    RevisionNotPresent,
    /// The `artifact_format_tag` was the reserved all-zero value (§6.4-0001d).
    ZeroArtifactFormatTag,
    /// A declared `artifact` or `contract` byte-length exceeded its cap (§6.4-0001e); checked before any
    /// byte is read or allocated, whether or not that many bytes were supplied.
    LengthOutOfRange { region: &'static str, got: u64 },
}

/// The largest `artifact` byte-length a provision success may declare (§6.4-0001e): 2^28 = 256 MiB. **A
/// placeholder, revisable before any freeze** (no measurement exists); the largest realistic artifact (a
/// PTX or cubin module) is orders of magnitude smaller.
pub const MAX_ARTIFACT_LEN: u64 = 1 << 28;

/// A provision success: the pair `{artifact, contract}` under its echoed identity (§6.2-0001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionSuccess {
    /// The echoed identity. Its `revision_hash` MUST be `Some` (the echo always carries one).
    pub identity: Identity,
    /// Opaque 4-byte format/target-family tag; MUST NOT be all zero.
    pub artifact_format_tag: [u8; 4],
    pub artifact: Vec<u8>,
    /// The opaque, self-delimiting KISS-Contract document.
    pub contract: Vec<u8>,
}

impl ProvisionSuccess {
    /// Serialize per §6.4-0001b. Refuses to emit a frame the spec forbids: an echo
    /// without a `revision_hash`, or the reserved zero format tag.
    pub fn encode(&self) -> Result<Vec<u8>, SynthDecline> {
        if self.identity.revision_hash.is_none() {
            return Err(SynthDecline::RevisionNotPresent);
        }
        if self.artifact_format_tag == [0; 4] {
            return Err(SynthDecline::ZeroArtifactFormatTag);
        }
        let mut b = Vec::new();
        b.extend_from_slice(&PRSP.to_le_bytes());
        self.identity.encode_into(&mut b);
        b.extend_from_slice(&self.artifact_format_tag);
        b.extend_from_slice(&(self.artifact.len() as u64).to_le_bytes());
        b.extend_from_slice(&self.artifact);
        b.extend_from_slice(&(self.contract.len() as u32).to_le_bytes());
        b.extend_from_slice(&self.contract);
        Ok(b)
    }

    /// The KISS-Announce `CRSP` contract-only response for the same contract (§6.2-0003:
    /// the `PRSP` contract bytes are byte-identical to the `CRSP` payload).
    pub fn to_contract_response(&self) -> ContractResponse {
        ContractResponse { identity: self.identity.clone(), payload: self.contract.clone() }
    }
}

/// Decode and hard-reject a provision-success frame. Bytes after the contract are
/// not inspected (as with Announce's frame decoders, the frame is self-delimiting).
pub fn decode_provision_success(bytes: &[u8]) -> Result<ProvisionSuccess, SynthDecline> {
    if bytes.len() < 4 {
        return Err(SynthDecline::Truncated { region: "prsp_tag" });
    }
    let tag = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if tag == CRSP {
        return Err(SynthDecline::ContractOnlyResponse);
    }
    if tag != PRSP {
        return Err(SynthDecline::BadTag { got: tag });
    }
    let (identity, mut off) = Identity::decode_at(bytes, 4).map_err(SynthDecline::Identity)?;
    if identity.revision_hash.is_none() {
        return Err(SynthDecline::RevisionNotPresent);
    }
    if bytes.len() < off + 4 {
        return Err(SynthDecline::Truncated { region: "artifact_format_tag" });
    }
    let mut artifact_format_tag = [0u8; 4];
    artifact_format_tag.copy_from_slice(&bytes[off..off + 4]);
    off += 4;
    if artifact_format_tag == [0; 4] {
        return Err(SynthDecline::ZeroArtifactFormatTag);
    }
    if bytes.len() < off + 8 {
        return Err(SynthDecline::Truncated { region: "artifact_len" });
    }
    let artifact_len = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
    off += 8;
    // §6.4-0001e: the cap is checked before anything keyed on the length, supplied bytes or not
    if artifact_len > MAX_ARTIFACT_LEN {
        return Err(SynthDecline::LengthOutOfRange { region: "artifact", got: artifact_len });
    }
    // Compare in u64 against what is actually left, so a hostile length can neither
    // overflow an offset nor drive an allocation.
    if artifact_len > (bytes.len() - off) as u64 {
        return Err(SynthDecline::Truncated { region: "artifact" });
    }
    let artifact_len = artifact_len as usize;
    let artifact = bytes[off..off + artifact_len].to_vec();
    off += artifact_len;
    if bytes.len() < off + 4 {
        return Err(SynthDecline::Truncated { region: "contract_len" });
    }
    let contract_len_raw = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
    if contract_len_raw > crate::announce::MAX_CONTRACT_PAYLOAD_LEN {
        return Err(SynthDecline::LengthOutOfRange { region: "contract", got: u64::from(contract_len_raw) });
    }
    let contract_len = contract_len_raw as usize;
    off += 4;
    if contract_len > bytes.len() - off {
        return Err(SynthDecline::Truncated { region: "contract" });
    }
    let contract = bytes[off..off + contract_len].to_vec();
    Ok(ProvisionSuccess { identity, artifact_format_tag, artifact, contract })
}
