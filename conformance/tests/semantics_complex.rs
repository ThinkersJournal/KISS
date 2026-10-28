// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform tests for the §6.18 complex-arithmetic ops beyond the algebraic four, driven
//! through the from-scratch oracle in `conformance/src/semantics.rs` (`cmake`/`cre`/`cim`,
//! `cmul`, `cabs` via `hypot`, `carg`, `cexp`, `clog`, `csqrt`, `cpow`).
//!
//! Every expected value below is written out from the clause text (the enumerated Annex G rows
//! of §6.18-0009 / -0010 / -0011, the IEEE `atan2` table behind §6.18-0008, the recovery trigger
//! of §6.18-0005), never produced by the code under test. Each op is paired with its NAIVE
//! decomposition, so the rows where Annex G governs are shown to differ from the naive chain.
//! Rows the spec leaves with unspecified signs are asserted by class (zero, infinity, NaN), not
//! by sign.
//!
//! Clauses bound: KISS-OPS-6.18-0003, -0005, -0007, -0008, -0009, -0010, -0011, -0012, -0013.

use kiss_conformance::semantics::*;

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn bits(z: C32) -> [u32; 2] {
    [z[0].to_bits(), z[1].to_bits()]
}

/// Bit-exact lane comparison (the split comparator of §6.18-0017: sign of zero and ±π endpoint
/// are exact), NaN written as "any NaN".
fn lane_eq(got: f32, want: f32) -> bool {
    if want.is_nan() { got.is_nan() } else { got.to_bits() == want.to_bits() }
}

#[track_caller]
fn assert_c(got: C32, want: C32, what: &str) {
    assert!(
        lane_eq(got[0], want[0]) && lane_eq(got[1], want[1]),
        "{what}: got ({:?}, {:?}), want ({:?}, {:?})",
        got[0],
        got[1],
        want[0],
        want[1]
    );
}

#[track_caller]
fn assert_zero(x: f32, what: &str) {
    assert!(x == 0.0, "{what}: want a zero lane, got {x:?}");
}

#[track_caller]
fn assert_close(got: f32, want: f64, rel: f64, what: &str) {
    let err = (got as f64 - want).abs();
    assert!(err <= rel * want.abs().max(1e-30), "{what}: got {got:?}, want {want:?}");
}

const PI: f32 = std::f32::consts::PI;
const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
fn pi_over_4() -> f32 { (std::f64::consts::PI / 4.0) as f32 }
fn three_pi_over_4() -> f32 { (3.0 * std::f64::consts::PI / 4.0) as f32 }

// ---- the NAIVE decompositions, kept to show where Annex G governs ------------------------

fn naive_cmul(z: C32, w: C32) -> C32 {
    [z[0] * w[0] - z[1] * w[1], z[0] * w[1] + z[1] * w[0]]
}
fn naive_cexp(z: C32) -> C32 {
    let ea = z[0].exp();
    [ea * (z[1] as f64).cos() as f32, ea * (z[1] as f64).sin() as f32]
}
fn naive_cabs(z: C32) -> f32 {
    (z[0] * z[0] + z[1] * z[1]).sqrt()
}
fn naive_csqrt(z: C32) -> C32 {
    let m = naive_cabs(z);
    [((m + z[0]) / 2.0).sqrt(), 1.0f32.copysign(z[1]) * ((m - z[0]) / 2.0).sqrt()]
}
fn naive_clog(z: C32) -> C32 {
    [naive_cabs(z).ln(), (z[1] as f64).atan2(z[0] as f64) as f32]
}
fn naive_cpow(z: C32, w: C32) -> C32 {
    naive_cexp(naive_cmul(w, naive_clog(z)))
}

/// Enforces[enforced]: KISS-OPS-6.18-0003 — `cmake`, `cre`, `cim` are raw-lane moves between a complex
/// value and its two real lanes (real lane at offset 0, imaginary at offset 1), preserving the
/// signed zero and the NaN payload of each lane.
/// TEETH: lanes of -0.0, a payload NaN (0x7FC12345), a signaling-pattern NaN (0x7F800001) and an
/// infinity come back bit-identical, and the lanes are not swapped. A bridge built from
/// arithmetic (`x + 0.0`, `0.0 - x`) would turn -0.0 into +0.0 or quiet the NaN.
#[test]
fn test_ops_complex_component_bridge() {
    let lanes = [
        -0.0f32,
        0.0,
        f32::from_bits(0x7FC1_2345),
        f32::from_bits(0x7F80_0001),
        f32::from_bits(0xFFC0_0001),
        INF,
        -INF,
        1.5,
        f32::from_bits(1),
    ];
    for &x in &lanes {
        for &y in &lanes {
            let z = cmake(x, y);
            assert_eq!(bits(z), [x.to_bits(), y.to_bits()], "cmake keeps both lanes");
            assert_eq!(cre(z).to_bits(), x.to_bits(), "cre reads lane 0");
            assert_eq!(cim(z).to_bits(), y.to_bits(), "cim reads lane 1");
        }
    }
    assert_ne!(cre(cmake(1.0, 2.0)), cim(cmake(1.0, 2.0)), "the lanes are distinct");
}

/// Enforces[enforced]: KISS-OPS-6.18-0005 — `cmul` is `(ac − bd) + (ad + bc)i` for finite operands, and
/// applies the Annex G infinity recovery if and only if BOTH result components are NaN AND an
/// operand has an infinite lane; with only one NaN component recovery MUST NOT be applied.
/// TEETH: a finite product; the recovery cases return a complex infinity with Annex G signs where
/// the naive chain returns NaN in both lanes; a single-NaN result is left as computed (a recovery
/// that fired on "any NaN" fails it); both-NaN with no infinite operand lane stays NaN (a recovery
/// that fired without the infinity condition fails it).
#[test]
fn test_ops_cmul_annexg() {
    assert_c(cmul([1.0, 2.0], [3.0, 4.0]), [-5.0, 10.0], "finite (1+2i)(3+4i)");
    assert_c(cmul([0.0, -0.0], [-0.0, 0.0]), naive_cmul([0.0, -0.0], [-0.0, 0.0]), "finite zeros follow the decomposition");

    // Both result components NaN with an infinite operand lane: an infinity times a nonzero
    // finite or infinite operand is an infinity.
    let both_nan = [(INF, INF), (-INF, INF), (INF, -INF)];
    for (a, b) in both_nan {
        let n = naive_cmul([a, b], [1.0, 0.0]);
        assert!(n[0].is_nan() && n[1].is_nan(), "control: the naive chain is NaN/NaN for ({a}, {b})");
        let r = cmul([a, b], [1.0, 0.0]);
        // (±inf + i·±inf) · 1 = (±inf, ±inf), signs those of the infinite operand.
        assert_c(r, [a, b], "recovered complex infinity keeps the operand's signs");
    }
    // The infinity may sit in the second operand.
    assert_c(cmul([1.0, 0.0], [INF, INF]), [INF, INF], "infinity in the second operand");
    assert_c(cmul([1.0, 0.0], [-INF, INF]), [-INF, INF], "signs from the infinite operand");

    // Only ONE result component is NaN: no recovery. (inf + i0)(1 + i0): re = inf, im = inf*0 = NaN.
    let r = cmul([INF, 0.0], [1.0, 0.0]);
    assert!(r[0] == INF && r[1].is_nan(), "single-NaN result is left as computed, got {r:?}");

    // Another single-NaN result with an infinite operand: (1 + i·1e30)(1e30 − i∞) has re = ∞, im = −∞ + ∞ = NaN.
    let r = cmul([1.0, 1e30], [1e30, -INF]);
    assert!(r[0] == INF && r[1].is_nan(), "single-NaN result must not be recovered, got {r:?}");

    // Both NaN but NO infinite operand lane: no recovery.
    let r = cmul([NAN, 1.0], [1.0, 1.0]);
    assert!(r[0].is_nan() && r[1].is_nan(), "a NaN operand without an infinite lane stays NaN");
}

/// Enforces[enforced]: KISS-OPS-6.18-0007 — `cabs(z)` is `hypot(cre z, cim z)`, non-negative, and `+∞`
/// whenever either lane is infinite even if the other lane is NaN.
/// TEETH: (3,4) is 5 and the sign of the lanes is irrelevant; (±∞, NaN) and (NaN, ±∞) are +∞ where
/// the naive `sqrt(a²+b²)` is NaN; a NaN lane without an infinity gives NaN.
#[test]
fn test_ops_cabs_annexg() {
    assert_eq!(hypot(3.0, 4.0), 5.0);
    assert_eq!(hypot(-3.0, -4.0), 5.0);
    for z in [[INF, NAN], [-INF, NAN], [NAN, INF], [NAN, -INF], [INF, 1.0], [-INF, -INF]] {
        assert_eq!(hypot(z[0], z[1]), INF, "cabs{z:?}");
    }
    assert!(naive_cabs([INF, NAN]).is_nan(), "control: the naive chain is NaN here");
    assert!(hypot(NAN, 1.0).is_nan());
    assert!(hypot(1.0, NAN).is_nan());
    assert_eq!(hypot(0.0, -0.0).to_bits(), 0.0f32.to_bits(), "cabs of a zero is +0");
}

/// Enforces[enforced]: KISS-OPS-6.18-0008 — `carg(z)` is `atan2(cim z, cre z)`, the principal argument in
/// `[−π, +π]`; the signed zero of each lane selects the quadrant.
/// TEETH: the IEEE `atan2` zero table, bit-exact: the sign of a zero result and the ±π endpoint
/// follow the sign of the imaginary zero. A `carg` that took `atan(b/a)`, or ignored the sign of
/// a zero lane, fails it.
#[test]
fn test_ops_carg_principal() {
    assert_eq!(carg([1.0, 0.0]).to_bits(), 0.0f32.to_bits(), "carg(+1 + i0) = +0");
    assert_eq!(carg([1.0, -0.0]).to_bits(), (-0.0f32).to_bits(), "carg(+1 - i0) = -0");
    assert_eq!(carg([-1.0, 0.0]), PI, "carg(-1 + i0) = +π");
    assert_eq!(carg([-1.0, -0.0]), -PI, "carg(-1 - i0) = -π (both endpoints reachable)");
    assert_eq!(carg([0.0, 0.0]).to_bits(), 0.0f32.to_bits());
    assert_eq!(carg([-0.0, 0.0]), PI, "atan2(+0, -0) = +π");
    assert_eq!(carg([-0.0, -0.0]), -PI, "atan2(-0, -0) = -π");
    assert_eq!(carg([0.0, 1.0]), FRAC_PI_2);
    assert_eq!(carg([-0.0, 1.0]), FRAC_PI_2, "a zero real lane of either sign is the imaginary axis");
    assert_eq!(carg([0.0, -1.0]), -FRAC_PI_2);
    assert_eq!(carg([-1.0, 1.0]), three_pi_over_4());
    assert_eq!(carg([-1.0, -1.0]), -three_pi_over_4());
    assert!(carg([NAN, 1.0]).is_nan() && carg([1.0, NAN]).is_nan());
}

/// Enforces[enforced]: KISS-OPS-6.18-0009 — `cexp` is `exp(a)·(cos b + i·sin b)` for finite arguments, and
/// for an infinite or NaN lane the enumerated Annex G rows govern over the naive decomposition.
/// TEETH: each governing row of the clause, one assertion per row, with zero signs and infinity
/// signs bit-exact where the clause fixes them and class-only where it writes `±`. The naive chain
/// is shown to differ on `cexp(+∞ + i0)`, where it yields `∞·0 = NaN` in the imaginary lane.
#[test]
fn test_ops_cexp_annexg() {
    // cexp(±0 + i0) = 1 + i0.
    assert_c(cexp([0.0, 0.0]), [1.0, 0.0], "cexp(+0 + i0)");
    assert_c(cexp([-0.0, 0.0]), [1.0, 0.0], "cexp(-0 + i0)");
    // finite: cexp(0 + i1) = cos 1 + i sin 1 (within a couple of ulp).
    let r = cexp([0.0, 1.0]);
    assert_close(r[0], 1.0f64.cos(), 4.0 * f32::EPSILON as f64, "cos 1");
    assert_close(r[1], 1.0f64.sin(), 4.0 * f32::EPSILON as f64, "sin 1");
    // cexp(x + i∞) = NaN + iNaN (finite x); cexp(x + iNaN) = NaN + iNaN (finite x).
    assert_c(cexp([1.0, INF]), [NAN, NAN], "cexp(x + i∞)");
    assert_c(cexp([1.0, NAN]), [NAN, NAN], "cexp(x + iNaN)");
    // cexp(+∞ + i0) = +∞ + i0 (the naive chain has ∞·0 = NaN in the imaginary lane).
    assert_c(cexp([INF, 0.0]), [INF, 0.0], "cexp(+∞ + i0)");
    assert!(naive_cexp([INF, 0.0])[1].is_nan(), "control: the naive chain is NaN here");
    // cexp(−∞ + iy) = +0·(cos y + i·sin y): signs from cos y / sin y. y = 2: cos < 0, sin > 0.
    assert_c(cexp([-INF, 2.0]), [-0.0, 0.0], "cexp(-∞ + i2)");
    assert_c(cexp([-INF, -2.0]), [-0.0, -0.0], "cexp(-∞ - i2)");
    assert_c(cexp([-INF, 0.0]), [0.0, 0.0], "cexp(-∞ + i0)");
    // cexp(+∞ + iy) = +∞·(cos y + i·sin y) (finite nonzero y): y = 2 -> (-∞, +∞); y = 1 -> (+∞, +∞).
    assert_c(cexp([INF, 2.0]), [-INF, INF], "cexp(+∞ + i2)");
    assert_c(cexp([INF, 1.0]), [INF, INF], "cexp(+∞ + i1)");
    // cexp(−∞ + i∞) = ±0 ± i0; cexp(−∞ + iNaN) = ±0 ± i0.
    let r = cexp([-INF, INF]);
    assert_zero(r[0], "cexp(-∞ + i∞) re");
    assert_zero(r[1], "cexp(-∞ + i∞) im");
    let r = cexp([-INF, NAN]);
    assert_zero(r[0], "cexp(-∞ + iNaN) re");
    assert_zero(r[1], "cexp(-∞ + iNaN) im");
    // cexp(+∞ + i∞) = ±∞ + iNaN; cexp(+∞ + iNaN) = ±∞ + iNaN.
    let r = cexp([INF, INF]);
    assert!(r[0].is_infinite() && r[1].is_nan(), "cexp(+∞ + i∞), got {r:?}");
    let r = cexp([INF, NAN]);
    assert!(r[0].is_infinite() && r[1].is_nan(), "cexp(+∞ + iNaN), got {r:?}");
    // cexp(NaN + i0) = NaN + i0; cexp(NaN + iy) = NaN + iNaN (nonzero y); cexp(NaN + iNaN).
    assert_c(cexp([NAN, 0.0]), [NAN, 0.0], "cexp(NaN + i0)");
    assert_c(cexp([NAN, 1.0]), [NAN, NAN], "cexp(NaN + iy)");
    assert_c(cexp([NAN, NAN]), [NAN, NAN], "cexp(NaN + iNaN)");
}

/// Enforces[enforced]: KISS-OPS-6.18-0010 — `clog` is `cmake(log(cabs z), carg z)`, the principal logarithm:
/// the imaginary part lies in the closed `[−π, +π]` with the sign of a zero imaginary lane
/// selecting the endpoint, and the enumerated Annex G rows govern on an infinite or NaN lane and
/// at the origin.
/// TEETH: `clog(−1 ± i0)` is `0 ± iπ` bit-exact (both endpoints); the origin rows; one assertion
/// per governing row of the clause, with π/2, π, 3π/4 and π/4 as the exact f32 roundings of the
/// f64 constants.
#[test]
fn test_ops_clog_principal_branch() {
    assert_c(clog([-1.0, 0.0]), [0.0, PI], "clog(-1 + i0) = 0 + iπ");
    assert_c(clog([-1.0, -0.0]), [0.0, -PI], "clog(-1 - i0) = 0 - iπ");
    assert_c(clog([1.0, 0.0]), [0.0, 0.0], "clog(1) = 0");
    // origin rows.
    assert_c(clog([-0.0, 0.0]), [-INF, PI], "clog(-0 + i0) = -∞ + iπ");
    assert_c(clog([0.0, 0.0]), [-INF, 0.0], "clog(+0 + i0) = -∞ + i0");
    // clog(x + i∞) = +∞ + iπ/2 (finite x); clog(x + iNaN) = NaN + iNaN (finite x).
    assert_c(clog([3.0, INF]), [INF, FRAC_PI_2], "clog(x + i∞)");
    assert_c(clog([3.0, NAN]), [NAN, NAN], "clog(x + iNaN)");
    // clog(−∞ + iy) = +∞ + iπ; clog(+∞ + iy) = +∞ + i0 (finite y > 0).
    assert_c(clog([-INF, 2.0]), [INF, PI], "clog(-∞ + iy)");
    assert_c(clog([INF, 2.0]), [INF, 0.0], "clog(+∞ + iy)");
    // clog(−∞ + i∞) = +∞ + i3π/4; clog(+∞ + i∞) = +∞ + iπ/4.
    assert_c(clog([-INF, INF]), [INF, three_pi_over_4()], "clog(-∞ + i∞)");
    assert_c(clog([INF, INF]), [INF, pi_over_4()], "clog(+∞ + i∞)");
    // clog(±∞ + iNaN) = +∞ + iNaN; clog(NaN + i∞) = +∞ + iNaN.
    assert_c(clog([INF, NAN]), [INF, NAN], "clog(+∞ + iNaN)");
    assert_c(clog([-INF, NAN]), [INF, NAN], "clog(-∞ + iNaN)");
    assert_c(clog([NAN, INF]), [INF, NAN], "clog(NaN + i∞)");
    // clog(NaN + iy) = NaN + iNaN (finite y); clog(NaN + iNaN) = NaN + iNaN.
    assert_c(clog([NAN, 2.0]), [NAN, NAN], "clog(NaN + iy)");
    assert_c(clog([NAN, NAN]), [NAN, NAN], "clog(NaN + iNaN)");
    // finite magnitude: clog(e^2 · i) = 2 + iπ/2 within tolerance.
    let z = [0.0, (2.0f64).exp() as f32];
    let r = clog(z);
    assert_close(r[0], 2.0, 4.0 * f32::EPSILON as f64, "log of the modulus");
    assert_eq!(r[1], FRAC_PI_2);
}

/// Enforces[enforced]: KISS-OPS-6.18-0011 — `csqrt` is the principal square root (non-negative real part,
/// cut on the negative real axis), the imaginary sign taken from the imaginary lane so
/// `csqrt(−4 ± i0) = 0 ± 2i`, and the enumerated Annex G rows govern on an infinite or NaN lane.
/// TEETH: the signed-zero pair bit-exact; one assertion per governing row; the naive chain is shown
/// to differ on `csqrt(−∞ + i2)`, where it yields NaN in the real lane.
#[test]
fn test_ops_csqrt_principal_branch() {
    assert_c(csqrt([-4.0, 0.0]), [0.0, 2.0], "csqrt(-4 + i0) = 0 + 2i");
    assert_c(csqrt([-4.0, -0.0]), [0.0, -2.0], "csqrt(-4 - i0) = 0 - 2i");
    assert_c(csqrt([3.0, 4.0]), [2.0, 1.0], "csqrt(3 + 4i) = 2 + i");
    assert_c(csqrt([3.0, -4.0]), [2.0, -1.0], "csqrt(3 - 4i) = 2 - i");
    assert_c(csqrt([0.0, 2.0]), [1.0, 1.0], "csqrt(2i) = 1 + i");
    // csqrt(±0 + i0) = +0 + i0.
    assert_c(csqrt([0.0, 0.0]), [0.0, 0.0], "csqrt(+0 + i0)");
    assert_c(csqrt([-0.0, 0.0]), [0.0, 0.0], "csqrt(-0 + i0)");
    // csqrt(x + i∞) = +∞ + i∞ for any x, including ±∞ and NaN.
    for x in [1.0, -1.0, INF, -INF, NAN] {
        assert_c(csqrt([x, INF]), [INF, INF], "csqrt(x + i∞)");
    }
    // csqrt(x + iNaN) = NaN + iNaN (finite x).
    assert_c(csqrt([1.0, NAN]), [NAN, NAN], "csqrt(x + iNaN)");
    // csqrt(−∞ + iy) = +0 + i∞; csqrt(+∞ + iy) = +∞ + i0 (finite positive y).
    assert_c(csqrt([-INF, 2.0]), [0.0, INF], "csqrt(-∞ + iy)");
    assert_c(csqrt([INF, 2.0]), [INF, 0.0], "csqrt(+∞ + iy)");
    let n = naive_csqrt([-INF, 2.0]);
    assert!(n[0].is_nan(), "control: the naive chain is NaN in the real lane, got {n:?}");
    // csqrt(−∞ + iNaN) = NaN ± i∞; csqrt(+∞ + iNaN) = +∞ + iNaN.
    let r = csqrt([-INF, NAN]);
    assert!(r[0].is_nan() && r[1].is_infinite(), "csqrt(-∞ + iNaN), got {r:?}");
    assert_c(csqrt([INF, NAN]), [INF, NAN], "csqrt(+∞ + iNaN)");
    // csqrt(NaN + iy) = NaN + iNaN (finite y); csqrt(NaN + iNaN).
    assert_c(csqrt([NAN, 1.0]), [NAN, NAN], "csqrt(NaN + iy)");
    assert_c(csqrt([NAN, NAN]), [NAN, NAN], "csqrt(NaN + iNaN)");
}

/// Enforces[enforced]: KISS-OPS-6.18-0012 — `cpow(z, w)` is the principal `cexp(cmul(w, clog z))`, evaluated
/// through the GOVERNED `clog`, `cmul` and `cexp`; `cpow(z, 0) = 1 + 0i` for every `z`;
/// `cpow(0 + i0, w)` is `0 + i0` for `Re w > 0`, a complex infinity for `Re w < 0`, and NaN
/// components for `Re w = 0`, `w ≠ 0`.
/// TEETH: the exponent-zero rule holds for NaN, infinite and zero bases; the three zero-base rows;
/// a finite principal value (8 and the cube root of −8); and the naive chain is shown to produce
/// NaN on `cpow(0, 2)` where the clause pins `0 + i0`.
#[test]
fn test_ops_cpow_principal() {
    for z in [[NAN, NAN], [INF, 3.0], [0.0, 0.0], [2.0, 2.0], [-0.0, -0.0]] {
        assert_c(cpow(z, [0.0, 0.0]), [1.0, 0.0], "cpow(z, 0) = 1 + 0i");
        assert_c(cpow(z, [-0.0, -0.0]), [1.0, 0.0], "cpow(z, -0) = 1 + 0i");
    }
    // cpow(0 + i0, w).
    let r = cpow([0.0, 0.0], [2.0, 0.0]);
    assert!(r[0] == 0.0 && r[1] == 0.0, "Re w > 0: 0 + i0, got {r:?}");
    let n = naive_cpow([0.0, 0.0], [2.0, 0.0]);
    assert!(n[0].is_nan() && n[1].is_nan(), "control: the naive chain is NaN/NaN, got {n:?}");
    let r = cpow([0.0, 0.0], [-2.0, 0.0]);
    assert!(r[0].is_infinite() || r[1].is_infinite(), "Re w < 0: a complex infinity, got {r:?}");
    let r = cpow([0.0, 0.0], [0.0, 1.0]);
    assert!(r[0].is_nan() && r[1].is_nan(), "Re w = 0, w != 0: NaN components, got {r:?}");
    // finite principal values: 2^3 = 8 (imaginary lane exactly +0), (-8)^(1/3) = 1 + i·sqrt 3.
    let r = cpow([2.0, 0.0], [3.0, 0.0]);
    assert_close(r[0], 8.0, 1e-5, "2^3");
    assert_eq!(r[1].to_bits(), 0.0f32.to_bits(), "2^3 has a +0 imaginary lane");
    let third = 1.0f32 / 3.0;
    let r = cpow([-8.0, 0.0], [third, 0.0]);
    assert_close(r[0], 1.0, 1e-5, "(-8)^(1/3) real");
    assert_close(r[1], 3.0f64.sqrt(), 1e-5, "(-8)^(1/3) imaginary");
}

/// Enforces[enforced]: KISS-OPS-6.18-0013 — on any input with an infinite or NaN lane, the Annex G rows and
/// recovery GOVERN over the naive real decomposition: a complex op MUST NOT return what the naive
/// decomposition would where the governing rule pins a different value.
/// TEETH: one input per op where naive and governed DISAGREE, the governed value being the one the
/// clause rows pin: `cmul` (NaN/NaN versus a complex infinity), `cexp` (`∞·0` versus `+∞ + i0`),
/// `cabs` (NaN versus `+∞`), `csqrt` (NaN real lane versus `+0`), and `cpow` (NaN/NaN versus
/// `0 + i0`). A reference that fell back to the naive chain on a non-finite lane fails each.
#[test]
fn test_ops_complex_nan_inf_annexg() {
    let cases: [(&str, C32, C32, C32, C32); 4] = [
        ("cmul(inf+i·inf, 1)", naive_cmul([INF, INF], [1.0, 0.0]), cmul([INF, INF], [1.0, 0.0]), [NAN, NAN], [INF, INF]),
        ("cexp(+inf + i0)", naive_cexp([INF, 0.0]), cexp([INF, 0.0]), [INF, NAN], [INF, 0.0]),
        ("csqrt(-inf + i2)", naive_csqrt([-INF, 2.0]), csqrt([-INF, 2.0]), [NAN, INF], [0.0, INF]),
        ("cpow(0, 2)", naive_cpow([0.0, 0.0], [2.0, 0.0]), cpow([0.0, 0.0], [2.0, 0.0]), [NAN, NAN], [0.0, 0.0]),
    ];
    for (what, naive, governed, want_naive, want_governed) in cases {
        assert_c(naive, want_naive, &format!("{what}: the naive chain"));
        assert_c(governed, want_governed, &format!("{what}: the governed value"));
        assert!(
            !(lane_eq(naive[0], governed[0]) && lane_eq(naive[1], governed[1])),
            "{what}: naive and governed must differ"
        );
    }
    assert!(naive_cabs([INF, NAN]).is_nan() && hypot(INF, NAN) == INF, "cabs(inf, NaN)");
}
