//! Plan B Slice-0 Task T1 — `hp-core` `BigFloat<N>` correctness vectors.
//!
//! These are the §11 test vectors of the recovered hp-core design spec
//! (`docs/superpowers/specs/2026-07-21-planb-slice0-hp-core-spec.md`), encoded
//! as failing-first tests per the plan's TDD discipline. Every expected value
//! is hand-derived from the round-ziv §0 substrate convention:
//!
//!   value = (-1)^sign * (mant as P-bit integer) * 2^exp,   P = 64*N
//!   mant is NORMALIZED for Normal: bit (P-1) set, `mant[0]` MOST significant.
//!   ebin() = exp + P - 1   (MSB binary exponent, width-invariant).
//!
//! N=4 => P=256 throughout unless noted.

use kiss_conformance::hp::{BigFloat, FpClass};
use std::cmp::Ordering;

const MSB: u64 = 0x8000_0000_0000_0000; // bit 63 of the top limb = bit P-1
const A: u64 = 0xAAAA_AAAA_AAAA_AAAA;
const AB: u64 = 0xAAAA_AAAA_AAAA_AAAB;

// --- C: constructors (exact) ------------------------------------------------

/// C1 — from_f64(1.0) @ N=4: exp=-255, mant=[0x8000..,0,0,0], ebin=0, exact.
#[test]
fn c1_from_f64_one() {
    let x = BigFloat::<4>::from_f64(1.0);
    assert_eq!(x.class, FpClass::Normal);
    assert!(!x.sign);
    assert_eq!(x.exp, -255);
    assert_eq!(x.mant, [MSB, 0, 0, 0]);
    assert_eq!(x.ebin(), 0);
    x.check_invariant();
}

/// C2 — from_f64(3.0): exp=-254, mant=[0xC000..,0,0,0], ebin=1.
#[test]
fn c2_from_f64_three() {
    let x = BigFloat::<4>::from_f64(3.0);
    assert_eq!(x.exp, -254);
    assert_eq!(x.mant, [0xC000_0000_0000_0000, 0, 0, 0]);
    assert_eq!(x.ebin(), 1);
    x.check_invariant();
}

/// C3 — from_f64 of the smallest subnormal double 2^-1074 (0x00..01): the
/// clz-normalize branch. exp=-1329, mant=[0x8000..,0,0,0], ebin=-1074.
#[test]
fn c3_from_f64_min_subnormal() {
    let x = BigFloat::<4>::from_f64(f64::from_bits(1));
    assert_eq!(x.class, FpClass::Normal);
    assert_eq!(x.exp, -1329);
    assert_eq!(x.mant, [MSB, 0, 0, 0]);
    assert_eq!(x.ebin(), -1074);
    x.check_invariant();
}

// --- M: multiply ------------------------------------------------------------

/// M0 — exact: 3.0 * 3.0 = 9.0. exp=-252, mant=[0x9000..,0,0,0], err=0.
#[test]
fn m0_mul_exact() {
    let three = BigFloat::<4>::from_f64(3.0);
    let (r, err) = three.mul(&three);
    assert_eq!(err, 0);
    assert_eq!(r.exp, -252);
    assert_eq!(r.mant, [0x9000_0000_0000_0000, 0, 0, 0]);
    r.check_invariant();
}

/// M1 — truncate: (2^255+1, e0)*(2^255+2, e0). The low 2^1 is dropped, R=0.
/// exp=255, mant=[0x8000..,0,0,3], err=1.
#[test]
fn m1_mul_truncate() {
    let a = BigFloat::<4>::from_limbs(false, 0, [MSB, 0, 0, 1], FpClass::Normal);
    let b = BigFloat::<4>::from_limbs(false, 0, [MSB, 0, 0, 2], FpClass::Normal);
    let (r, err) = a.mul(&b);
    assert_eq!(err, 1);
    assert_eq!(r.exp, 255);
    assert_eq!(r.mant, [MSB, 0, 0, 3]);
}

/// M2 — tie to even, rounds UP: (2^255+1)*(2^255+2^254). R=1,S=0 tie; retained
/// value is odd -> up. exp=255, mant=[0xC000..,0,0,2], err=1.
#[test]
fn m2_mul_tie_to_even_up() {
    let a = BigFloat::<4>::from_limbs(false, 0, [MSB, 0, 0, 1], FpClass::Normal);
    let b = BigFloat::<4>::from_limbs(false, 0, [0xC000_0000_0000_0000, 0, 0, 0], FpClass::Normal);
    let (r, err) = a.mul(&b);
    assert_eq!(err, 1);
    assert_eq!(r.exp, 255);
    assert_eq!(r.mant, [0xC000_0000_0000_0000, 0, 0, 2]);
}

// --- S/A: subtract / add ----------------------------------------------------

/// S1 — massive cancellation, EXACT: 1.0 - (1 - 2^-100) = 2^-100. The result
/// exp drops ~100 (to -355); no bits are rounded so err=0.
#[test]
fn s1_sub_cancellation_exact() {
    let one = BigFloat::<4>::one();
    let eps = one.ldexp(-100); // 2^-100
    let almost = one.sub(&eps).0; // exactly 1 - 2^-100
    // Sanity: 1 - 2^-100 has ebin -1.
    assert_eq!(almost.ebin(), -1);
    let (r, err) = one.sub(&almost);
    assert_eq!(err, 0);
    assert_eq!(r.exp, -355);
    assert_eq!(r.mant, [MSB, 0, 0, 0]);
    assert_eq!(r.ebin(), -100);
    // ...and it equals 2^-100 exactly.
    assert_eq!(r.cmp(&eps), Ordering::Equal);
}

/// A1 — guard/sticky: 1.0 + 2^-300 = 1.0. The addend lands entirely in the
/// sticky region (R=0), so it rounds away but is inexact => err=1.
#[test]
fn a1_add_guard_sticky() {
    let one = BigFloat::<4>::one();
    let tiny = one.ldexp(-300); // 2^-300
    let (r, err) = one.add(&tiny);
    assert_eq!(err, 1);
    assert_eq!(r.exp, -255);
    assert_eq!(r.mant, [MSB, 0, 0, 0]);
    assert_eq!(r.ebin(), 0);
}

// --- D: divide (general long division + single-limb) ------------------------

/// D1 — GENERAL div (restoring long division): 1.0 / 3.0. Correctly rounded
/// RNE (rounds up in the last limb). exp=-257, mant=[0xAA..AA x3, 0xAA..AB].
#[test]
fn d1_div_general() {
    let (r, err) = BigFloat::<4>::from_f64(1.0).div(&BigFloat::<4>::from_f64(3.0));
    assert_eq!(err, 1);
    assert_eq!(r.exp, -257);
    assert_eq!(r.mant, [A, A, A, AB]);
    assert_eq!(r.ebin(), -2);
    r.check_invariant();
}

/// D2 — div_small cross-checks D1: from_f64(1.0).div_small(3) is bit-identical.
#[test]
fn d2_div_small_matches_general() {
    let d1 = BigFloat::<4>::from_f64(1.0).div(&BigFloat::<4>::from_f64(3.0));
    let d2 = BigFloat::<4>::from_f64(1.0).div_small(3);
    assert_eq!(d2.1, d1.1, "err_ulps agree");
    assert_eq!(d2.0.exp, d1.0.exp, "exp agree");
    assert_eq!(d2.0.mant, d1.0.mant, "mant agree");
    assert_eq!(d2.0.sign, d1.0.sign);
    assert_eq!(d2.1, 1);
    assert_eq!(d2.0.mant, [A, A, A, AB]);
}

/// D3 — div_small exact: 7.0 / 7 = 1.0, err=0.
#[test]
fn d3_div_small_exact() {
    let (r, err) = BigFloat::<4>::from_f64(7.0).div_small(7);
    assert_eq!(err, 0);
    assert_eq!(r.exp, -255);
    assert_eq!(r.mant, [MSB, 0, 0, 0]);
}

/// D4 — div_small exact halving: 1.0 / 2 = 0.5. exp=-256, mant=[0x8000..], err=0.
#[test]
fn d4_div_small_half() {
    let (r, err) = BigFloat::<4>::from_f64(1.0).div_small(2);
    assert_eq!(err, 0);
    assert_eq!(r.exp, -256);
    assert_eq!(r.mant, [MSB, 0, 0, 0]);
    assert_eq!(r.ebin(), -1);
}

// --- X: comparison + exact midpoint primitive -------------------------------

/// X1 — abs_diff_limbs with a cross-limb borrow: |2^64 - (2^64 - 1)| = 1.
/// The two operands share an exponent; the difference borrows from limb 2
/// down through limb 3, yielding the big-int 1 = [0,0,0,1].
#[test]
fn x1_abs_diff_cross_limb_borrow() {
    let a = BigFloat::<4>::from_limbs(false, 0, [0, 0, 1, 0], FpClass::Normal); // 2^64
    let b = BigFloat::<4>::from_limbs(false, 0, [0, 0, 0, u64::MAX], FpClass::Normal); // 2^64 - 1
    assert_eq!(a.abs_diff_limbs(&b), [0, 0, 0, 1]);
    assert_eq!(b.abs_diff_limbs(&a), [0, 0, 0, 1]); // symmetric
}

/// X2 — cmp: 1.0 < 1 + 2^-52 (the next double after 1.0).
#[test]
fn x2_cmp_less() {
    let one = BigFloat::<4>::from_f64(1.0);
    let up = BigFloat::<4>::from_f64(f64::from_bits(0x3FF0_0000_0000_0001)); // 1 + 2^-52
    assert_eq!(one.cmp(&up), Ordering::Less);
    assert_eq!(up.cmp(&one), Ordering::Greater);
    assert_eq!(one.cmp(&one), Ordering::Equal);
}

// --- round_to_i64 (T1 hardening: previously the only untested hp API) -------

/// round_to_i64 on small magnitudes and the ties-to-even boundary, all within
/// |value| < 2^62. Ties round to the EVEN neighbour (2.5 -> 2, 3.5 -> 4).
#[test]
fn round_to_i64_small_and_ties_to_even() {
    let f = |x: f64| BigFloat::<4>::from_f64(x).round_to_i64();
    // small magnitudes, ordinary rounding
    assert_eq!(f(0.0), 0);
    assert_eq!(f(0.4), 0);
    assert_eq!(f(0.6), 1);
    assert_eq!(f(2.4), 2);
    assert_eq!(f(2.6), 3);
    assert_eq!(f(-2.6), -3);
    // exact integers are exact
    assert_eq!(BigFloat::<4>::from_i64(41).round_to_i64(), 41);
    assert_eq!(BigFloat::<4>::from_i64(-41).round_to_i64(), -41);
    // ties-to-even (the round-bit-set, sticky-clear case)
    assert_eq!(f(2.5), 2, "2.5 ties down to even 2");
    assert_eq!(f(3.5), 4, "3.5 ties up to even 4");
    assert_eq!(f(-2.5), -2, "-2.5 ties to even -2");
    assert_eq!(f(0.5), 0, "0.5 ties to even 0");
    assert_eq!(f(1.5), 2, "1.5 ties to even 2");
    // a large-but-in-range value (|value| < 2^62): 2^61 is exact.
    let big = (1i64) << 61;
    assert_eq!(BigFloat::<4>::from_i64(big).round_to_i64(), big);
    // zero of either sign -> 0
    assert_eq!(BigFloat::<4>::zero(true).round_to_i64(), 0);
}

// --- W: escalation ----------------------------------------------------------

/// W1 — widen::<8> preserves value: from_f64(1.0)@N4 -> N8. exp=-511,
/// mant=[0x8000.., 7 zeros], ebin() still 0 (width-invariant).
#[test]
fn w1_widen_preserves_value() {
    let x4 = BigFloat::<4>::from_f64(1.0);
    let x8: BigFloat<8> = x4.widen::<8>();
    assert_eq!(x8.exp, -511);
    assert_eq!(x8.mant, [MSB, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(x8.ebin(), 0); // width-invariant MSB exponent
    x8.check_invariant();
}

// --- signed zero + convention spot checks -----------------------------------

/// Signed zero is carried and distinguished; +0 and -0 compare Equal.
#[test]
fn signed_zero() {
    let pz = BigFloat::<4>::zero(false);
    let nz = BigFloat::<4>::zero(true);
    assert!(pz.is_zero() && nz.is_zero());
    assert!(!pz.sign && nz.sign);
    assert_eq!(pz.cmp(&nz), Ordering::Equal);
    // from_f64 preserves the sign of zero.
    assert!(BigFloat::<4>::from_f64(-0.0).sign);
    assert!(!BigFloat::<4>::from_f64(0.0).sign);
    // neg flips a zero's sign; (-0)+(-0) = -0.
    assert!(pz.neg().sign);
    assert!(nz.add(&nz).0.sign);
}

/// dist_gt_err compares the big-int distance against the u128 err bound.
#[test]
fn dist_gt_err_basic() {
    // D = 5, err = 4 -> D > err
    assert!(BigFloat::<4>::dist_gt_err(&[0, 0, 0, 5], 4));
    // D = 5, err = 5 -> not strictly greater
    assert!(!BigFloat::<4>::dist_gt_err(&[0, 0, 0, 5], 5));
    // any high limb set exceeds any u128 err
    assert!(BigFloat::<4>::dist_gt_err(&[1, 0, 0, 0], u128::MAX));
}

// --- escalation across N in {4,8,16} (design spec §8) -----------------------
//
// The LSB-weight `exp` is width-dependent, but `ebin()` is width-invariant and
// is what higher kernels compare. from_f64/div/div_small are re-derived (not
// zero-extended) at each width, and 1/3 rounds up in its last limb at every N.

/// N=8: from_f64(1.0) has exp=-511 but the same ebin=0 as N=4.
#[test]
fn e1_width8_one() {
    let x = BigFloat::<8>::from_f64(1.0);
    assert_eq!(x.exp, -511);
    assert_eq!(x.ebin(), 0);
    assert_eq!(x.mant[0], MSB);
    x.check_invariant();
}

/// N=8: div(1,3) is re-derived at 512-bit precision — ebin=-2 (width-invariant),
/// mant = [0xAA.. x7, 0xAA..AB], err=1; div_small(3) agrees bit-for-bit.
#[test]
fn e2_width8_div_one_third() {
    let (r, err) = BigFloat::<8>::from_f64(1.0).div(&BigFloat::<8>::from_f64(3.0));
    assert_eq!(err, 1);
    assert_eq!(r.ebin(), -2);
    assert_eq!(r.exp, -513);
    assert_eq!(r.mant, [A, A, A, A, A, A, A, AB]);
    r.check_invariant();
    let s = BigFloat::<8>::from_f64(1.0).div_small(3);
    assert_eq!(s.0.mant, r.mant);
    assert_eq!(s.0.exp, r.exp);
    assert_eq!(s.1, err);
}

/// N=16: 1024-bit div(1,3) — ebin=-2, mant = [0xAA.. x15, 0xAA..AB], err=1.
#[test]
fn e3_width16_div_one_third() {
    let (r, err) = BigFloat::<16>::from_f64(1.0).div(&BigFloat::<16>::from_f64(3.0));
    assert_eq!(err, 1);
    assert_eq!(r.ebin(), -2);
    let mut expect = [A; 16];
    expect[15] = AB;
    assert_eq!(r.mant, expect);
    r.check_invariant();
}

/// widen::<8> then narrow::<4> round-trips an EXACT value with zero recorded
/// error in both directions (1.0 fits in 4 limbs).
#[test]
fn e4_widen_narrow_roundtrip_exact() {
    let x4 = BigFloat::<4>::from_f64(1.0);
    let x8 = x4.widen::<8>();
    let (back, err) = x8.narrow::<4>();
    assert_eq!(err, 0, "narrowing an exact value records no error");
    assert_eq!(back.exp, x4.exp);
    assert_eq!(back.mant, x4.mant);
    assert_eq!(back.ebin(), 0);
}

/// narrow of an INEXACT wide value keeps the top M limbs (truncation) and
/// records +1 error for the dropped tail. Narrowing 1/3@N8 to N4 yields the
/// TRUNCATED [0xAA.. x4] (one ULP below the correctly-rounded N4 value), err=1.
#[test]
fn e5_narrow_records_dropped_tail() {
    let third8 = BigFloat::<8>::from_f64(1.0).div(&BigFloat::<8>::from_f64(3.0)).0;
    let (n4, err) = third8.narrow::<4>();
    assert_eq!(err, 1, "dropped low limbs are nonzero -> +1");
    assert_eq!(n4.mant, [A, A, A, A]); // truncated, not rounded up
    assert_eq!(n4.ebin(), -2);
}
