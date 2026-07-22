//! §6.4-0009/-0010 typed-decline taxonomy tests (#17): a CLOSED normative `decline_code` the
//! consumer binds on + an OPEN, seeded, informative `blocker_reason` SET that never gates.
//!
//! Oracle: [`kiss_conformance::decline`].

use kiss_conformance::decline::{
    is_seed_reason, DeclineCode, DeclineFrame, SEED_BLOCKER_REASONS,
};

/// §6.4-0009 — the decline_code is a CLOSED set of exactly seven, keyed by an unambiguous wire
/// token; an unknown token is rejected.
#[test]
fn test_consume_decline_code_closed_set() {
    // Exactly seven codes, no more.
    assert_eq!(DeclineCode::ALL.len(), 7);

    // Every code round-trips through its wire token, and the token set is exactly the pinned one.
    let tokens: Vec<&str> = DeclineCode::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(
        tokens,
        [
            "not-a-region",
            "multi-output",
            "bind-set-mismatch",
            "unknown-op",
            "operand-tuple-inexpressible",
            "attrs-can't-carry",
            "out-of-range-index",
        ]
    );
    for c in DeclineCode::ALL {
        assert_eq!(DeclineCode::parse(c.as_str()), Ok(*c));
    }

    // A token outside the closed set is rejected — the consumer binds on this, so it must be closed.
    assert!(DeclineCode::parse("some-vendor-decline").is_err());
    assert!(DeclineCode::parse("vocabulary").is_err()); // that's a blocker_reason, not a code

    // Phase consistency (§6.4-0008): structural codes are Phase-A, lift-miss codes are Phase-B.
    assert!(DeclineCode::NotARegion.is_phase_a_structural());
    assert!(DeclineCode::OutOfRangeIndex.is_phase_a_structural());
    assert!(!DeclineCode::UnknownOp.is_phase_a_structural());
    assert!(!DeclineCode::AttrsCantCarry.is_phase_a_structural());
}

/// §6.4-0010 — blocker_reason is an OPEN, seeded, informative SET: the seeds are the recommended
/// spellings, a frame can couple multiple, a novel reason is admissible, and it never gates.
#[test]
fn test_consume_blocker_reason_open_seeded_set() {
    // The six seed reasons are the recommended spellings.
    assert_eq!(SEED_BLOCKER_REASONS.len(), 6);
    for r in ["vocabulary", "attrs-channel", "keying", "determinism", "shape-layout-inexpressible", "dispatch-envelope"] {
        assert!(is_seed_reason(r), "{r} should be a seed reason");
    }

    // A single lift-miss can COUPLE multiple blockers (Baracuda's gather = attrs+keying).
    let gather = DeclineFrame {
        decline_code: DeclineCode::AttrsCantCarry,
        blocker_reason: vec!["attrs-channel".into(), "keying".into()],
    };
    assert!(gather.is_well_formed());
    assert_eq!(gather.blocker_reason.len(), 2);

    // The registry is OPEN: a genuinely novel blocker (none of us have hit) is admissible without
    // a spec change — it is purely informative, never a gate.
    let novel = DeclineFrame {
        decline_code: DeclineCode::UnknownOp,
        blocker_reason: vec!["mobile-fp16-mad-envelope-gap".into()],
    };
    assert!(novel.is_well_formed());
    assert!(!is_seed_reason("mobile-fp16-mad-envelope-gap"));

    // A frame may carry NO blocker_reason (it is optional — the decline_code alone is normative).
    let bare = DeclineFrame { decline_code: DeclineCode::MultiOutput, blocker_reason: vec![] };
    assert!(bare.is_well_formed());
}
