//! §6.4-0012 — the **anti-fork op-tag rule** (#29 item 1).
//!
//! The op name is the sole identity anchor of an advertisable op. A party that meets an
//! operation it cannot re-base onto a KISS-Ops op name MUST typed-decline and MUST **never
//! fabricate a tag**. This module witnesses the positive emit-side invariant Baracuda's
//! matcher already guarantees (`recipe.rs`: emit confirmed KISS-Ops tokens, honest-miss
//! otherwise): every emitted `op_name` is a member of the confirmed KISS-Ops vocabulary; an
//! unrecognized op surfaces as a typed decline, never a fabricated tag.

/// A typed decline (never a panic, never a fabricated tag — §7.1) for an operation with no
/// KISS-Ops op name in the pinned vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownOp {
    pub candidate: String,
}

/// Emit the confirmed KISS-Ops op tag for a candidate operation, gating on the confirmed
/// KISS-Ops vocabulary of the pinned version. Returns the confirmed tag iff the candidate
/// names a KISS-Ops op; otherwise a **typed decline**. It NEVER invents, guesses, or
/// approximates a tag for an unrecognized op, and never re-bases a near-miss onto a core name
/// — that is the anti-fork rule (§6.4-0012).
pub fn emit_op_tag<'a>(candidate: &str, kiss_ops_vocab: &'a [&'a str]) -> Result<&'a str, UnknownOp> {
    kiss_ops_vocab
        .iter()
        .copied()
        .find(|name| *name == candidate)
        .ok_or_else(|| UnknownOp { candidate: candidate.to_string() })
}

/// Witness the positive anti-fork invariant over a set of emitted op tags: every emitted tag
/// MUST be a member of the confirmed KISS-Ops vocabulary — no fabricated tag ever escapes.
/// (An op with no confirmed name must instead surface via [`emit_op_tag`]'s typed decline.)
pub fn no_fabricated_tag(emitted: &[&str], kiss_ops_vocab: &[&str]) -> bool {
    emitted.iter().all(|tag| kiss_ops_vocab.contains(tag))
}
