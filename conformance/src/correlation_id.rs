// SPDX-License-Identifier: MIT OR Apache-2.0
//! Witness for the request-correlation field on the KISS-Announce contract-query frames and the
//! KISS-Synth provision-success frame (KISS-ANNOUNCE-6.4-0013).
//!
//! The correlation id lets a **concurrent** transport match a response to the right in-flight request.
//! The FIELD is always present, so a frame is self-delimiting (§6.2-0014); the ID is optional: a synchronous
//! seam spells it as the single byte `00`. When a request carries one, the response answering it MUST echo
//! the identical 8 bytes unchanged.
//!
//! Wire field, the last component of the echoed identity block (§6.4-0011), after the revision:
//! ```text
//!   +0  1  correlation_len  u8      0 or 8
//!   +1  8  correlation_id   u8[8]   present only when correlation_len == 8
//! ```

use crate::announce::{read_correlation, AnnounceDecline};

pub use crate::announce::{CorrelationId, CORRELATION_ID_LEN};

/// Serialize the always-present correlation field (§6.4-0013): `00` for none, `08` then the id otherwise.
pub fn encode_field(id: Option<CorrelationId>) -> Vec<u8> {
    match id {
        None => vec![0],
        Some(id) => {
            let mut out = vec![CORRELATION_ID_LEN];
            out.extend_from_slice(&id);
            out
        }
    }
}

/// Parse the correlation field at the start of `bytes`; returns the id and the number of bytes the field
/// occupied. A length byte other than `0` or `8` is [`AnnounceDecline::BadCorrelationLen`]; a field cut
/// short is [`AnnounceDecline::Truncated`] — never a panic.
pub fn parse_field(bytes: &[u8]) -> Result<(Option<CorrelationId>, usize), AnnounceDecline> {
    read_correlation(bytes, 0)
}

/// The id a response MUST carry for a request that carried `request_id`: the identical bytes, unchanged
/// (§6.4-0013). A request that carried none gets none.
pub fn echo(request_id: Option<CorrelationId>) -> Option<CorrelationId> {
    request_id
}
