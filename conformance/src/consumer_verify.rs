// SPDX-License-Identifier: MIT OR Apache-2.0
//! The consumer-verify check (Conform §6.12, §7.2) — the one operative SHOULD KISS-Conform owns.
//!
//! A consumer SHOULD verify a received kernel against its contract before trusting it
//! (KISS-SYNTH-6.4-0004). This module is that check: it reads the contract's declared
//! determinism class ([`crate::contract::parse_guarantees_class`]), selects the comparator through
//! the §6.8-0008 precedence ([`crate::comparator_for`]), and differences the received kernel
//! against the §6.5 oracle ([`crate::differential`] corpus, [`crate::semantics`] oracle supplied by
//! the caller). It introduces **no comparator and no oracle of its own** (§6.12-0002): every
//! value comparison goes through [`crate::compare_f32`].
//!
//! Accept-predicate (`= structure_key`): the kernel's advertised key must parse
//! ([`crate::structure_key::from_token`]) and equal the key the consumer asked for.
//!
//! Force: the check is a governance SHOULD. In the default profile a failing report gates
//! nothing (§6.12-0003, §7.2-0001); only an implementation that ADVERTISES the opt-in
//! trust-but-verify profile is gated on it (§7.2-0003), and that profile never alters the
//! default clause set (§7.2-0004).

use crate::contract::{parse_guarantees_class, ContractDecline};
use crate::structure_key::{from_token, KeyDecline};
use crate::{comparator_for, compare_f32, Comparator, DeterminismClass};

/// Why a received kernel failed verification, or why verification could not run.
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyFailure {
    /// The contract's Guarantees block did not yield a determinism class (typed, never a panic).
    ContractUnreadable(ContractDecline),
    /// The kernel's advertised structure_key is not a well-formed key.
    AdvertisedKeyDeclined(KeyDecline),
    /// The requested structure_key is not a well-formed key.
    RequestedKeyDeclined(KeyDecline),
    /// Both keys parse but differ: the kernel does not answer the cell that was asked for.
    AcceptPredicateMismatch,
    /// The declared class is `order-invariant/nondeterministic`: a per-element scalar comparison is
    /// not defined for it (§6.8), so this check declines rather than invent a comparator.
    ClassNotScalarComparable,
    /// An input on which the candidate disagrees with the oracle under the declared comparator.
    ValueMismatch { input: f32, oracle: f32, candidate: f32 },
}

/// The outcome of one consumer-verify run: every failure found, empty = verified.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifyReport {
    pub failures: Vec<VerifyFailure>,
}

impl VerifyReport {
    pub fn verified(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Verify a received unary f32 kernel against its contract.
///
/// * `contract_body` — the contract body carrying the `[section:6:guarantees]` block.
/// * `declared_ulp` — the declared tolerance (read by the caller from the Guarantees tier); used
///   only when the declared class is ULP/tolerance.
/// * `advertised_key` / `requested_key` — the accept-predicate operands.
/// * `oracle` / `candidate` / `corpus` — the §6.5 differential inputs.
pub fn consumer_verify(
    contract_body: &[u8],
    declared_ulp: u64,
    advertised_key: &str,
    requested_key: &str,
    oracle: impl Fn(f32) -> f32,
    candidate: impl Fn(f32) -> f32,
    corpus: &[f32],
) -> VerifyReport {
    let mut r = VerifyReport::default();

    match (from_token(advertised_key), from_token(requested_key)) {
        (Err(e), _) => r.failures.push(VerifyFailure::AdvertisedKeyDeclined(e)),
        (_, Err(e)) => r.failures.push(VerifyFailure::RequestedKeyDeclined(e)),
        (Ok(a), Ok(q)) if a != q => r.failures.push(VerifyFailure::AcceptPredicateMismatch),
        _ => {}
    }

    let class = match parse_guarantees_class(contract_body) {
        Ok(c) => c,
        Err(e) => {
            r.failures.push(VerifyFailure::ContractUnreadable(e));
            return r;
        }
    };
    // §6.8-0008 precedence: the declared class is the default; the op is not named here, so
    // no op-named refinement can apply and `ClassDefault(class)` is the selection.
    let Comparator::ClassDefault(class) = comparator_for("", class) else {
        unreachable!("an empty op name carries no §6.8-0005 refinement");
    };
    if class == DeterminismClass::OrderInvariant {
        r.failures.push(VerifyFailure::ClassNotScalarComparable);
        return r;
    }
    for &x in corpus {
        let (o, c) = (oracle(x), candidate(x));
        if compare_f32(class, c, o, declared_ulp).is_err() {
            r.failures.push(VerifyFailure::ValueMismatch { input: x, oracle: o, candidate: c });
        }
    }
    r
}

/// The two profiles of §7.2. `Default` gates nothing on consumer-verify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Default,
    TrustButVerify,
}

/// The profile this build was compiled for: the `trust-but-verify` cargo feature is the
/// build-profile switch (§7.2-0003); it is off by default.
pub const BUILD_PROFILE: Profile = if cfg!(feature = "trust-but-verify") {
    Profile::TrustButVerify
} else {
    Profile::Default
};

/// A suite result: the default-profile clause verdict (never altered by the profile, §7.2-0004),
/// the profile it was produced under, and the consumer-verify report.
#[derive(Debug, Clone, PartialEq)]
pub struct SuiteOutcome {
    pub profile: Profile,
    pub default_clauses_pass: bool,
    pub verify: VerifyReport,
}

impl SuiteOutcome {
    /// Whether the build passes. The default clause verdict always counts; the consumer-verify
    /// report counts ONLY under the opt-in profile AND only for an implementation that advertises
    /// trust-but-verify (§7.2-0003) — otherwise it is informational (§6.12-0003, §7.2-0001).
    pub fn build_passes(&self, implementation_advertises_tbv: bool) -> bool {
        let gated = self.profile == Profile::TrustButVerify && implementation_advertises_tbv;
        self.default_clauses_pass && (!gated || self.verify.verified())
    }
}
