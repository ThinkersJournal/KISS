// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §7.2-0005: the opt-in profile's three outcomes, the durable record per
//! (kernel, cell, target), and invalidation on a new `revision_hash`.

use kiss_conformance::consumer_verify::{VerifyFailure, VerifyReport};
use kiss_conformance::verify_record::{outcome_of, RecordKey, VerifyLedger, VerifyOutcome};

fn mismatch() -> VerifyFailure {
    VerifyFailure::ValueMismatch { input: 1.0, oracle: 1.0, candidate: 2.0 }
}

/// KISS-CONFORM-7.2-0005.
#[test]
fn test_conform_verify_outcomes_and_record() {
    // Clause: KISS-CONFORM-7.2-0005.
    // Three outcomes.
    let clean = VerifyReport::default();
    let off_by_value = VerifyReport { failures: vec![mismatch()] };
    let wrong_cell = VerifyReport { failures: vec![VerifyFailure::AcceptPredicateMismatch] };
    let mixed = VerifyReport { failures: vec![mismatch(), VerifyFailure::AcceptPredicateMismatch] };
    assert_eq!(outcome_of(&clean, true), VerifyOutcome::Adopted);
    assert_eq!(outcome_of(&off_by_value, true), VerifyOutcome::AdoptedDemoted);
    // Demotion is opt-in: without it a value difference is a rejection.
    assert_eq!(outcome_of(&off_by_value, false), VerifyOutcome::Rejected);
    // Only a value difference can be demoted; a key mismatch, alone or mixed in, is a rejection.
    assert_eq!(outcome_of(&wrong_cell, true), VerifyOutcome::Rejected);
    assert_eq!(outcome_of(&mixed, true), VerifyOutcome::Rejected);

    // The record is per (kernel, cell, target) and valid only at the verified revision.
    let key = RecordKey { kernel: "add".into(), cell: "sk6|ew2|f32".into(), target: "cuda:sm89".into() };
    let other_target = RecordKey { target: "cuda:sm80".into(), ..key.clone() };
    let (r1, r2) = ([1u8; 32], [2u8; 32]);
    let mut ledger = VerifyLedger::default();
    assert_eq!(ledger.outcome_at(&key, r1), None, "nothing recorded yet");
    ledger.record(key.clone(), r1, VerifyOutcome::AdoptedDemoted);
    assert_eq!(ledger.outcome_at(&key, r1), Some(VerifyOutcome::AdoptedDemoted));
    assert_eq!(ledger.outcome_at(&other_target, r1), None, "another target has its own record");
    // A new revision invalidates the record: verify again before reuse.
    assert_eq!(ledger.outcome_at(&key, r2), None);
    ledger.record(key.clone(), r2, VerifyOutcome::Adopted);
    assert_eq!(ledger.outcome_at(&key, r2), Some(VerifyOutcome::Adopted));
    assert_eq!(ledger.outcome_at(&key, r1), None, "the old revision no longer has a record");
}
