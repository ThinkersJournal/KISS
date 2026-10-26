// SPDX-License-Identifier: MIT OR Apache-2.0
//! The outcomes and the durable record of the opt-in trust-but-verify profile
//! (KISS-CONFORM §7.2-0005; CireSnave decision 310, R1.24 Q5 A).
//!
//! Verifying a received kernel ends in one of three outcomes. The outcome is a **local**
//! classification by the consumer, not a claim that the provider's contract is wrong. The record is
//! keyed by (kernel, cell, target) and holds the `revision_hash` that was verified: a different
//! revision invalidates the record, so the kernel is verified again before it is reused.

use crate::consumer_verify::{VerifyFailure, VerifyReport};
use std::collections::BTreeMap;

/// What the consumer decided about a received kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// Verified as its contract declares.
    Adopted,
    /// Differs from its contract in value only, adopted under a lower local classification and recorded
    /// as demoted. There is a single demoted state (no lattice of how far it missed).
    AdoptedDemoted,
    /// Not adopted.
    Rejected,
}

/// Decide the outcome of a verification. Only a value difference can be demoted, and only when the
/// consumer allows demotion; a contract it cannot read, a key mismatch, or a class it cannot compare
/// is a rejection.
pub fn outcome_of(report: &VerifyReport, allow_demotion: bool) -> VerifyOutcome {
    if report.verified() {
        return VerifyOutcome::Adopted;
    }
    let only_values = report.failures.iter().all(|f| matches!(f, VerifyFailure::ValueMismatch { .. }));
    if only_values && allow_demotion {
        VerifyOutcome::AdoptedDemoted
    } else {
        VerifyOutcome::Rejected
    }
}

/// The key of a record: the kernel, the cell (its `structure_key`) and the target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordKey {
    pub kernel: String,
    pub cell: String,
    pub target: String,
}

/// One durable record: the outcome and the `revision_hash` it was reached at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyRecord {
    pub revision_hash: [u8; 32],
    pub outcome: VerifyOutcome,
}

/// The consumer's record of verifications.
#[derive(Debug, Default)]
pub struct VerifyLedger {
    records: BTreeMap<RecordKey, VerifyRecord>,
}

impl VerifyLedger {
    pub fn record(&mut self, key: RecordKey, revision_hash: [u8; 32], outcome: VerifyOutcome) {
        self.records.insert(key, VerifyRecord { revision_hash, outcome });
    }

    /// The outcome recorded for `key`, valid only for the revision that was verified. A record made at a
    /// different `revision_hash` is invalidated: this returns `None` and the kernel must be verified again.
    pub fn outcome_at(&self, key: &RecordKey, revision_hash: [u8; 32]) -> Option<VerifyOutcome> {
        self.records.get(key).filter(|r| r.revision_hash == revision_hash).map(|r| r.outcome)
    }
}
