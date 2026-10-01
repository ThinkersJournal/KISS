// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Guarantees `bit_stability` reproducibility-scope axis (KISS-CONTRACT-6.8-0013).

use kiss_conformance::contract::BitStability;
use kiss_conformance::DeterminismClass;

/// Enforces: KISS-CONTRACT-6.8-0013 — the value set is exactly `{portable, same-hardware,
/// none}`; `portable` iff `exact-byte`; and an `order-invariant/nondeterministic` kernel MAY
/// declare `same-hardware` (the former forced-`false` coupling is withdrawn).
#[test]
fn test_contract_bit_stability_scope() {
    use BitStability::*;
    use DeterminismClass::*;
    let all = [Portable, SameHardware, None];

    // Closed set, spelled verbatim, round-tripping; the withdrawn two-value spelling is refused.
    for v in all {
        assert_eq!(BitStability::from_token(v.token()), Some(v));
    }
    assert_eq!(all.map(|v| v.token()), ["portable", "same-hardware", "none"]);
    for old in ["bit-stable", "bit-unstable", "true", "false", "same_hardware_bitwise", ""] {
        assert_eq!(BitStability::from_token(old), Option::None, "{old:?} is not in the closed set");
    }

    // Consistency table, all nine (scope, class) cells -- a rule that ignored either axis
    // would differ from this table in at least one cell.
    let expect = |s: BitStability, c: DeterminismClass| match (s, c) {
        (Portable, ExactByte) => true,
        (Portable, _) => false,
        (_, ExactByte) => false,
        (SameHardware | None, UlpTolerance | OrderInvariant) => true,
    };
    for s in all {
        for c in [ExactByte, UlpTolerance, OrderInvariant] {
            assert_eq!(s.consistent_with(c), expect(s, c), "{s:?} x {c:?}");
        }
    }
    // The relaxation: a deterministic-order float-sum kernel (class order-invariant) may be
    // same-hardware stable. Under the old rule this cell was forbidden.
    assert!(SameHardware.consistent_with(OrderInvariant));
}
