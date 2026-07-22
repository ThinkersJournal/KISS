//! §6.11-0011 — bundle / catalog member isolation (#29 item 2).
//!
//! A provider **catalog** is a bundle of multiple contract documents carried together on the
//! wire. This module witnesses the §6.11-0011 isolation guarantee directly on top of the real
//! §6.11 self-delimiting frame (the `crate::contract` transport codec): each member is
//! delimited by its own §6.11-0003 header `len=<N>`, so a member whose *body* is malformed
//! (bad CRC, headingless, unsupported version, …) is typed-declined in isolation while its
//! well-formed siblings still parse. No malformed member is ever bundle-fatal, and reading a
//! bundle never panics (§6.1-0004).
//!
//! The member extent is taken from the FRAME (`member_extent`) before the body is validated,
//! so a body-level malformation declines exactly one member and the cursor still advances by
//! the frame's declared length to the next sibling. A member whose frame itself is
//! unrecoverable ends bundle reading with a terminal typed [`FrameError`] — but every sibling
//! already read is preserved (a frame break cannot un-read an earlier well-formed member).

use crate::contract::{read_document, ContractDecline, ContractHeader};

/// The per-member outcome of reading a bundle: a delimited member either parsed (yielding its
/// validated §6.1 transport header) or was typed-declined. Never a panic (§6.1-0004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberOutcome {
    /// The member's frame and body validated (§6.1 / §6.11).
    Accepted(ContractHeader),
    /// The member was well-framed but its body was malformed — a typed decline isolated to
    /// this member (§6.11-0011); its siblings are unaffected.
    Declined(ContractDecline),
}

/// A terminal frame error: a member whose self-delimiting frame (§6.11-0003) is unrecoverable,
/// so the next member boundary cannot be located. Reading stops here with a typed decline
/// (never a panic); members already read are preserved. Distinct from a body-malformed member,
/// which is delimited and isolated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// The member does not begin with the pinned magic `KISC` (§6.11-0002).
    NoMagic,
    /// The header line has no terminating LF (§6.11-0002).
    UnterminatedHeader,
    /// The header line does not carry a parsable `len=<N>` frame, or the declared length
    /// overruns the remaining bundle bytes (§6.11-0003).
    MalformedLen,
}

/// Parse ONLY the self-delimiting frame of the member at the front of `stream`: the header
/// line (through its LF) and the declared body length `N` from `len=<N>`. Returns the member's
/// total byte extent (header-line length + `N`) WITHOUT validating the body — this is the
/// §6.11-0003 extent that delimits the member from its siblings. Bounds-checked throughout;
/// never panics, never reads out of range, and never allocates on the declared length.
fn member_extent(stream: &[u8]) -> Result<usize, FrameError> {
    if stream.len() < 4 || &stream[0..4] != b"KISC" {
        return Err(FrameError::NoMagic);
    }
    let lf = stream
        .iter()
        .position(|&b| b == b'\n')
        .ok_or(FrameError::UnterminatedHeader)?;
    let header = std::str::from_utf8(&stream[..lf]).map_err(|_| FrameError::MalformedLen)?;
    let parts: Vec<&str> = header.split(' ').collect();
    // Pinned form: `KISC <kind> <version> len=<N> crc32=<H>` — five space-joined fields.
    if parts.len() != 5 {
        return Err(FrameError::MalformedLen);
    }
    let declared: usize = parts[3]
        .strip_prefix("len=")
        .and_then(|s| s.parse().ok())
        .ok_or(FrameError::MalformedLen)?;
    let header_len = lf + 1; // the header line through its LF
    // The member extent is header_len + declared body bytes. A declared length that overruns
    // the remaining bundle is a frame error — the slice is never taken out of range.
    let end = header_len.checked_add(declared).ok_or(FrameError::MalformedLen)?;
    if end > stream.len() {
        return Err(FrameError::MalformedLen);
    }
    Ok(end)
}

/// Read a bundle / catalog of self-delimiting contract-document members (§6.11-0011). Each
/// member is delimited by its own §6.11-0003 `len=<N>` frame, so a member whose BODY is
/// malformed (bad CRC, headingless, unsupported version, …) is isolated: it is typed-declined
/// ([`MemberOutcome::Declined`]) while its well-formed siblings still parse
/// ([`MemberOutcome::Accepted`]). Per-member isolation is NEVER bundle-fatal and NEVER a panic.
///
/// The returned vector holds one outcome per successfully-DELIMITED member, in stream order.
/// If a member's FRAME itself is unrecoverable (its `len=<N>` cannot be parsed, so the next
/// boundary cannot be located), reading stops with a terminal [`FrameError`] AFTER the isolated
/// siblings already read — a frame break cannot un-read a well-formed earlier sibling.
pub fn read_bundle(stream: &[u8]) -> (Vec<MemberOutcome>, Option<FrameError>) {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while cursor < stream.len() {
        let rest = &stream[cursor..];
        let extent = match member_extent(rest) {
            Ok(e) => e,
            Err(fe) => return (out, Some(fe)),
        };
        // Validate exactly this member's bytes in isolation. A body-level malformation declines
        // THIS member only; the cursor still advances by the frame extent, so the sibling at
        // `cursor + extent` is untouched.
        let member = &rest[..extent];
        match read_document(member) {
            Ok(h) => out.push(MemberOutcome::Accepted(h)),
            Err(d) => out.push(MemberOutcome::Declined(d)),
        }
        cursor += extent;
    }
    (out, None)
}