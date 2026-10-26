// SPDX-License-Identifier: MIT OR Apache-2.0
//! What a kernel does with an invocation whose derived `structure_key` byte-matches its own
//! (KISS-CLASSIFY §6.6-0001 says the match is necessary, not sufficient; KISS-CONSUME §6.5-0004 says a
//! refusal must be typed; CireSnave decision 310, R1.24 Q6 A).

/// The answer to an invocation. There is no silent value: a kernel that does not admit says why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMatchAnswer {
    /// The keys differ by at least one byte: not admitted (Classify §6.6-0001).
    KeyMismatch,
    /// The keys match and the kernel computes what was asked.
    Admitted,
    /// The keys match and the kernel declines: a typed decline the caller can act on, never a silent
    /// refusal, an abort, or the result of a different computation.
    Declined(DeclineReason),
}

/// Why a key-matching invocation was declined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclineReason {
    /// The cell matches but this kernel does not compute the requested op (a cell-mate).
    DoesNotComputeRequestedOp,
}

/// Decide an invocation. `computes_requested_op` is the kernel's own answer to "do I compute this?".
pub fn answer(invocation_key: &[u8], kernel_key: &[u8], computes_requested_op: bool) -> KeyMatchAnswer {
    if invocation_key != kernel_key {
        KeyMatchAnswer::KeyMismatch
    } else if computes_requested_op {
        KeyMatchAnswer::Admitted
    } else {
        KeyMatchAnswer::Declined(DeclineReason::DoesNotComputeRequestedOp)
    }
}
