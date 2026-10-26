// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Consume §6.5-0004: a kernel that declines a key-matching invocation reports a typed decline.
//! `relu(a + b)` and `a + b` share a key (Classify §6.6-0001), so the key match alone cannot admit.

use kiss_conformance::key_match::{answer, DeclineReason, KeyMatchAnswer};

/// KISS-CONSUME-6.5-0004.
#[test]
fn test_consume_key_matched_decline_is_typed() {
    // Clause: KISS-CONSUME-6.5-0004.
    let key = b"sk6|ew2|f32|cuda:sm89";
    // The kernel computes what was asked: admitted.
    assert_eq!(answer(key, key, true), KeyMatchAnswer::Admitted);
    // A cell-mate: same key, a different computation. A typed decline, not Admitted and not a mismatch.
    assert_eq!(
        answer(key, key, false),
        KeyMatchAnswer::Declined(DeclineReason::DoesNotComputeRequestedOp)
    );
    // Any one-byte difference in the key is a mismatch whatever the kernel computes.
    let mut other = key.to_vec();
    other[3] ^= 1;
    assert_eq!(answer(key, &other, true), KeyMatchAnswer::KeyMismatch);
    assert_eq!(answer(key, &other, false), KeyMatchAnswer::KeyMismatch);
    // The decline is distinguishable from both other answers, so a caller can fall back on it.
    assert_ne!(answer(key, key, false), KeyMatchAnswer::Admitted);
    assert_ne!(answer(key, key, false), KeyMatchAnswer::KeyMismatch);
}
