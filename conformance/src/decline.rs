//! §6.4-0009/-0010 — the typed-decline taxonomy (#17).
//!
//! Two additive fields on a decline / `lift_residue` frame, orthogonal to the four-category
//! MECE refusal taxonomy (§6.4-0001):
//!   * **`decline_code`** (§6.4-0009) — a CLOSED normative finer code the consumer binds on.
//!   * **`blocker_reason`** (§6.4-0010) — an OPEN, seeded, informative diagnostic SET (never a
//!     binding gate).
//!
//! "Close the normative gate, open the informative diagnostic." Grounded in Baracuda's
//! `pattern.rs` (11 `PatternError` variants → these codes) and Fuel's lifter/matcher.

/// The closed normative decline code (§6.4-0009): the specific structural failure or lift-miss
/// a consumer's next-action decision binds on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclineCode {
    NotARegion,
    MultiOutput,
    BindSetMismatch,
    UnknownOp,
    OperandTupleInexpressible,
    AttrsCantCarry,
    OutOfRangeIndex,
}

impl DeclineCode {
    /// The full closed set — exactly these seven, no more.
    pub const ALL: &'static [DeclineCode] = &[
        DeclineCode::NotARegion,
        DeclineCode::MultiOutput,
        DeclineCode::BindSetMismatch,
        DeclineCode::UnknownOp,
        DeclineCode::OperandTupleInexpressible,
        DeclineCode::AttrsCantCarry,
        DeclineCode::OutOfRangeIndex,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            DeclineCode::NotARegion => "not-a-region",
            DeclineCode::MultiOutput => "multi-output",
            DeclineCode::BindSetMismatch => "bind-set-mismatch",
            DeclineCode::UnknownOp => "unknown-op",
            DeclineCode::OperandTupleInexpressible => "operand-tuple-inexpressible",
            DeclineCode::AttrsCantCarry => "attrs-can't-carry",
            DeclineCode::OutOfRangeIndex => "out-of-range-index",
        }
    }

    /// Parse a wire token into the closed code; an unknown token is rejected (the set is closed,
    /// §6.4-0009).
    pub fn parse(token: &str) -> Result<DeclineCode, String> {
        DeclineCode::ALL
            .iter()
            .copied()
            .find(|c| c.as_str() == token)
            .ok_or_else(|| format!("`{token}` is not a §6.4-0009 decline_code (closed set)"))
    }

    /// The refusal phase this code belongs to (§6.4-0008): Phase-A codes are whole-input
    /// structural declines; Phase-B codes are per-region lift-misses. A frame's `decline_code`
    /// MUST be consistent with its category/phase.
    pub fn is_phase_a_structural(self) -> bool {
        matches!(
            self,
            DeclineCode::NotARegion
                | DeclineCode::MultiOutput
                | DeclineCode::BindSetMismatch
                | DeclineCode::OutOfRangeIndex
        )
    }
}

/// The §6.4-0010 blocker_reason seed registry — the RECOMMENDED spellings for the known
/// blocker classes. The registry is OPEN: a genuinely novel blocker is additive, but a party
/// whose blocker matches a seed MUST use the seed spelling.
pub const SEED_BLOCKER_REASONS: &[&str] = &[
    "vocabulary",
    "attrs-channel",
    "keying",
    "determinism",
    "shape-layout-inexpressible",
    "dispatch-envelope",
];

/// Is `reason` one of the seeded recommended spellings?
pub fn is_seed_reason(reason: &str) -> bool {
    SEED_BLOCKER_REASONS.contains(&reason)
}

/// A decline / residue frame's typed-decline fields (§6.4-0009/-0010).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclineFrame {
    /// §6.4-0009 — the closed normative code (required).
    pub decline_code: DeclineCode,
    /// §6.4-0010 — the open informative reason SET (optional; may couple multiple).
    pub blocker_reason: Vec<String>,
}

impl DeclineFrame {
    /// A frame is well-formed iff its `decline_code` is a member of the closed set (guaranteed by
    /// the enum) and every `blocker_reason` entry is either a seed spelling or a genuinely novel
    /// (non-seed) token — the registry is open, so a novel reason is accepted, but the field is a
    /// SET and MUST NOT be empty when present. blocker_reason is never consulted as a gate.
    pub fn is_well_formed(&self) -> bool {
        // decline_code is closed by construction (enum). blocker_reason is open — any token is
        // admissible; nothing here rejects a novel reason, since it is purely informative.
        true
    }
}
