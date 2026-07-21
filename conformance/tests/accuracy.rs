//! §6.8 accuracy-tier tests (D2 / #39): the declared tier is the sole gate, the ceiling is
//! retired to an *informative advisory floor*, and the v1 tier is flat with the
//! argument-dependent form reserved (§6.8-0006).
//!
//! Oracle: [`kiss_conformance::accuracy`].
//!   * §6.8-0001 — the declared tier is the sole gate (`test_ops_transcendental_declared_tier_is_gate`)
//!   * §6.8-0006 — the v1 tier is flat; arg-dependent form is reserved (`test_ops_accuracy_tier_flat_v1`)

use kiss_conformance::accuracy::{advisory_floor_ulp, AccuracyModel, AccuracyTier, MeasuredError};

fn measured_ulp(ulp: f64) -> MeasuredError {
    MeasuredError { ulp, relative: 0.0, absolute: 0.0 }
}

/// §6.8-0001 — the declared per-target tier is the **sole** gate; a truthful tier looser
/// than the retired ceiling MUST NOT be rejected, and the gate is the declaration itself,
/// never the advisory-floor table.
#[test]
fn test_ops_transcendental_declared_tier_is_gate() {
    // A Khronos-conformant `atan` at 5 ULP — the OLD 4-ULP ceiling rejected this truthful
    // provider (the #39 defect). It is a well-formed tier...
    let atan_tier = AccuracyTier::ulp(5.0);
    assert!(atan_tier.is_well_formed());
    // ...and it is NOT capped by the advisory floor: 5.0 exceeds the informative floor of
    // 4.0, yet the tier stands as declared.
    assert!(atan_tier.max_ulp.unwrap() > advisory_floor_ulp("atan").unwrap());

    // The gate is the DECLARED tier: measured 5.0 is admitted, 6.0 is not.
    assert!(atan_tier.admits(measured_ulp(5.0)));
    assert!(!atan_tier.admits(measured_ulp(6.0)));

    // A Vulkan `atan` permitted 4096 ULP is likewise a well-formed declaration and gates at
    // 4096 — proving there is no fixed suite-wide cap the tier must also satisfy.
    let vk_atan = AccuracyTier::ulp(4096.0);
    assert!(vk_atan.admits(measured_ulp(4096.0)));
    assert!(!vk_atan.admits(measured_ulp(4097.0)));

    // A tier that declares NO bound is ill-formed (§6.8-0001 requires ≥1) and admits nothing.
    let empty = AccuracyTier::default();
    assert!(!empty.is_well_formed());
    assert!(!empty.admits(measured_ulp(0.0)));

    // The advisory floor is INFORMATIVE: the gate must not consult it. A correctly-rounded
    // provider (0 ULP) is admitted by its own tier, and a 0.5-ULP miss is rejected by that
    // tier — not by any table value.
    let cr = AccuracyTier::ulp(0.0);
    assert!(cr.admits(measured_ulp(0.0)));
    assert!(!cr.admits(measured_ulp(0.5)));
}

/// §6.8-0006 — the v1 tier is flat (argument-independent); the argument-dependent /
/// range-scoped form is a RESERVED post-v1 model and is not a valid v1 declaration.
#[test]
fn test_ops_accuracy_tier_flat_v1() {
    // A flat tier in any of the three units is a valid v1 declaration.
    assert!(AccuracyModel::FlatV1(AccuracyTier::ulp(4.0)).is_v1());
    assert!(AccuracyModel::FlatV1(AccuracyTier::relative(1e-6)).is_v1());
    assert!(AccuracyModel::FlatV1(AccuracyTier::absolute(2.0_f64.powi(-11))).is_v1());

    // A flat tier with NO bound is not a valid declaration (well-formedness, §6.8-0001).
    assert!(!AccuracyModel::FlatV1(AccuracyTier::default()).is_v1());

    // The argument-dependent / range-scoped form (Vulkan `exp = 3 + 2·|x|` ULP; `sin`/`cos`
    // absolute-error-over-a-range) is RESERVED post-v1 — NOT admitted as a v1 declaration.
    let vk_exp = AccuracyModel::ReservedArgDependent("3 + 2*|x| ULP");
    let vk_sin = AccuracyModel::ReservedArgDependent("abs <= 2^-11 on [-pi, pi]");
    assert!(!vk_exp.is_v1());
    assert!(!vk_sin.is_v1());
}
