// SPDX-License-Identifier: MIT OR Apache-2.0
//! The closed precision-class token set, its derivation and its tier correspondence
//! (KISS-OPS-6.8-0007 / -0008 / -0009).

use kiss_conformance::accuracy::{derive_precision_class, AccuracyTier, PrecisionClass, TierKind};
use kiss_conformance::contract::BitStability;

const SCOPES: [BitStability; 3] = [BitStability::Portable, BitStability::SameHardware, BitStability::None];

/// Enforces: KISS-OPS-6.8-0007 — the set is exactly the five spellings, ordered tightest to
/// loosest, round-trips, and refuses `bit-reproducible` (folded into `strict`).
#[test]
fn test_ops_precision_class_token_set() {
    assert_eq!(
        PrecisionClass::ALL.map(|c| c.token()),
        ["strict", "correctly-rounded", "bounded-ulp", "bounded-tolerance", "unbounded"]
    );
    for c in PrecisionClass::ALL {
        assert_eq!(PrecisionClass::from_token(c.token()), Some(c));
    }
    for bad in ["bit-reproducible", "Strict", "exact", "correctly_rounded", "bounded", ""] {
        assert_eq!(PrecisionClass::from_token(bad), None, "{bad:?} is not in the closed set");
    }
    // The ordering is the declared tightest-first order: strict is least, unbounded greatest.
    let mut sorted = PrecisionClass::ALL;
    sorted.sort();
    assert_eq!(sorted, PrecisionClass::ALL);
    assert!(PrecisionClass::Strict < PrecisionClass::CorrectlyRounded);
    assert!(PrecisionClass::BoundedTolerance < PrecisionClass::Unbounded);
}

/// Enforces: KISS-OPS-6.8-0008 — the full 4x3 derivation table, the loosest-tier rule across
/// targets, and the no-tier case. Every cell is asserted so a rule ignoring either axis differs
/// from this table in at least one cell.
#[test]
fn test_ops_precision_class_derivation() {
    use PrecisionClass::*;
    use TierKind::*;
    let table = |k: TierKind, s: BitStability| match (k, s) {
        (T0, BitStability::Portable) => Strict,
        (T0, _) => CorrectlyRounded,
        (Tulp, _) => BoundedUlp,
        (Tother, _) => BoundedTolerance,
        (NoBound, _) => Unbounded,
    };
    for k in [T0, Tulp, Tother, NoBound] {
        for s in SCOPES {
            assert_eq!(derive_precision_class([k], s), table(k, s), "{k:?} x {s:?}");
        }
    }
    // Tier kinds from real tiers.
    assert_eq!(AccuracyTier::ulp(0.0).kind(), T0);
    assert_eq!(AccuracyTier::ulp(4.0).kind(), Tulp);
    assert_eq!(AccuracyTier::relative(1e-6).kind(), Tother);
    assert_eq!(AccuracyTier::absolute(1e-9).kind(), Tother);
    assert_eq!(AccuracyTier::default().kind(), NoBound);
    // max_ulp = 0 alongside a relative bound is still tier 0.
    let both = AccuracyTier { max_ulp: Some(0.0), max_relative: Some(1e-6), max_absolute: None };
    assert_eq!(both.kind(), T0);
    // Several targets: the LOOSEST governs, in every order.
    assert_eq!(derive_precision_class([T0, Tulp], BitStability::Portable), BoundedUlp);
    assert_eq!(derive_precision_class([Tulp, T0], BitStability::Portable), BoundedUlp);
    assert_eq!(derive_precision_class([T0, Tother, Tulp], BitStability::None), BoundedTolerance);
    assert_eq!(derive_precision_class([T0, NoBound], BitStability::Portable), Unbounded);
    // No tier declared at all is `unbounded`.
    assert_eq!(derive_precision_class([], BitStability::Portable), Unbounded);
}

/// Enforces: KISS-OPS-6.8-0009 — the class<->tier correspondence: both tier-0 classes map to tier 0,
/// the looser classes to strictly looser tier kinds, and no looser class maps to a tighter tier.
#[test]
fn test_ops_precision_class_tier_correspondence() {
    use PrecisionClass::*;
    assert_eq!(Strict.tier_kind(), TierKind::T0);
    assert_eq!(CorrectlyRounded.tier_kind(), TierKind::T0);
    assert_eq!(BoundedUlp.tier_kind(), TierKind::Tulp);
    assert_eq!(BoundedTolerance.tier_kind(), TierKind::Tother);
    assert_eq!(Unbounded.tier_kind(), TierKind::NoBound);
    // Monotone: a looser class never maps to a tighter tier kind.
    for a in PrecisionClass::ALL {
        for b in PrecisionClass::ALL {
            if a < b {
                assert!(a.tier_kind() <= b.tier_kind(), "{a:?} < {b:?} but its tier kind is looser");
            }
        }
    }
    // The correspondence inverts the derivation: the derived class's tier kind is the input kind.
    for k in [TierKind::T0, TierKind::Tulp, TierKind::Tother, TierKind::NoBound] {
        for s in SCOPES {
            assert_eq!(derive_precision_class([k], s).tier_kind(), k);
        }
    }
}
