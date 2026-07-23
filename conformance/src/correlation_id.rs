//! Witness for the OPTIONAL request-correlation id on the KISS-Announce
//! contract-query / KISS-Synth provision request+response frames
//! (KISS-ANNOUNCE-6.4-0013).
//!
//! The correlation id lets a **concurrent** transport match a response to the
//! right in-flight request. It is OPTIONAL and reserved: a synchronous seam
//! omits it, and it MUST NOT be required for v1 conformance. When a request
//! carries one, the response answering it MUST echo the identical 8 bytes
//! unchanged.
//!
//! Wire tail, appended after the §6.4-0001 / §6.4-0004 / §6.4-0007 frame body:
//! ```text
//!   +0  1  correlation_present  u8      0 or 1
//!   +1  8  correlation_id       u8[8]   present only when correlation_present == 1
//! ```
//! An empty tail (`&[]`) is a valid **absent** field — the byte-for-byte state a
//! synchronous seam that has not adopted the extension produces.

/// A reserved/optional 8-byte request-correlation id (§6.4-0013).
pub type CorrelationId = [u8; 8];

/// Reason a correlation tail was rejected (§6.4-0013): a `MALFORMED_REQUEST`
/// typed decline (Announce §6.4-0009), never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrelationError {
    /// `correlation_present` was a byte other than `0` or `1`.
    BadPresentFlag { got: u8 },
    /// The tail ended before the 8 id bytes a `correlation_present == 1` promised.
    Truncated,
}

/// Serialize the OPTIONAL correlation tail (§6.4-0013). `None` = a synchronous
/// seam that omits the field entirely: the frame body carries no tail at all
/// (an empty byte slice), so the frame is byte-identical to §6.4-0001/0004/0007.
pub fn encode_tail(id: Option<CorrelationId>) -> Vec<u8> {
    match id {
        None => Vec::new(),
        Some(bytes) => {
            let mut out = Vec::with_capacity(9);
            out.push(1u8); // correlation_present
            out.extend_from_slice(&bytes);
            out
        }
    }
}

/// Parse a correlation tail from the bytes following a frame body (§6.4-0013).
///
/// An empty tail (`&[]`) is a valid absent field -> `Ok(None)`; a reader MUST
/// accept it, because v1 conformance never requires the field. A present flag of
/// `0` is likewise absent. A present flag of `1` yields the 8 id bytes. Any other
/// flag, or a truncated present tail, is a typed error — never an out-of-bounds
/// panic.
pub fn parse_tail(tail: &[u8]) -> Result<Option<CorrelationId>, CorrelationError> {
    match tail.first() {
        None => Ok(None),      // absent field: valid on a synchronous seam
        Some(&0) => Ok(None),  // explicit correlation_present == 0
        Some(&1) => {
            if tail.len() < 9 {
                return Err(CorrelationError::Truncated);
            }
            let mut id = [0u8; 8];
            id.copy_from_slice(&tail[1..9]);
            Ok(Some(id))
        }
        Some(&other) => Err(CorrelationError::BadPresentFlag { got: other }),
    }
}

/// The provider echo obligation (§6.4-0013): the response answering a request
/// MUST carry the request's correlation id **unchanged** when the request carried
/// one, and MUST carry none when the request carried none. Returns the id the
/// response tail is required to advertise.
pub fn echo(request_id: Option<CorrelationId>) -> Option<CorrelationId> {
    request_id
}
