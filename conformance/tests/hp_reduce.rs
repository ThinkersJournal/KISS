//! Plan B Slice-0 Task T4 — `hp::reduction` argument-reduction vectors.
//!
//! Payne–Hanek trig reduction + Cody–Waite exp reduction + log octave reduction.
//! Every EXPECTED reduced argument is constructed INDEPENDENTLY of this crate:
//! the large-|x| / near-k·π trig references and the exp references were computed
//! offline with **mpmath at 3000-bit precision** (a different engine and a
//! different `2/π`/`ln2` than the crate's fdlibm tables), then pinned here as
//! normalized `BigFloat<4>` significands (MSW-first, RNE to 256 bits). The
//! reduction retains ≥ ~240 good bits, so agreement to 240 bits proves the deep
//! `2/π` tail bits are load-bearing and correct — the whole point of Payne–Hanek.
//!
//! Reference generator (reproduce): `mp.prec=3000; q=x*2/pi; n=nint(q);
//! octant=int(n)%8; r=(q-n)*(pi/2)` then normalize |r| to a 256-bit significand.

use kiss_conformance::hp::consts;
use kiss_conformance::hp::reduction::{
    reduce_exp, reduce_log, reduce_trig, reduce_trig_core, require_ln2_bits,
};
use kiss_conformance::hp::{BigFloat, FpClass};
use std::cmp::Ordering;

// --- helpers ---------------------------------------------------------------

/// `2^k` as an exact f64 (k in the normal f64 exponent range).
fn p2(k: i32) -> f64 {
    f64::from_bits(((1023 + k) as u64) << 52)
}

/// Build the pinned expected `BigFloat<4>` from an mpmath-derived
/// `(sign_neg, ebin, MSW-first limbs)`.
fn expect4(sign_neg: bool, ebin: i32, limbs: [u64; 4]) -> BigFloat<4> {
    BigFloat::<4>::from_limbs_ebin(sign_neg, ebin, limbs, FpClass::Normal)
}

/// Assert `|r − expected| ≤ 2^(r.ebin − min_agree_bits)` — i.e. `r` agrees with
/// the independent high-precision reference to at least `min_agree_bits` bits.
#[track_caller]
fn assert_agrees(r: &BigFloat<4>, expected: &BigFloat<4>, min_agree_bits: i32) {
    assert_eq!(r.sign, expected.sign, "sign mismatch");
    let (d, _) = r.sub(expected);
    let d = d.abs();
    if d.is_zero() {
        return;
    }
    assert!(
        d.ebin() <= r.ebin() - min_agree_bits,
        "reduced-arg mismatch: |r − ref| ≈ 2^{}, r ≈ 2^{} — only {} bits agree (need ≥ {})",
        d.ebin(),
        r.ebin(),
        r.ebin() - d.ebin(),
        min_agree_bits
    );
}

// ===========================================================================
// Payne–Hanek trig reduction
// ===========================================================================

/// sin(1.0) regime: 1.0 > π/4, so PH runs. The reduction-spec NOTE that says
/// "octant 0, r = 1.0" is SELF-CONTRADICTORY (r must lie in [−π/4, π/4] and
/// 1 > π/4); the review's correction is octant == 1, r = 1 − π/2 ≈ −0.5708,
/// sin = cos(r). This test pins the CORRECT reduction.
#[test]
fn trig_one_is_octant1_r_is_one_minus_half_pi() {
    let red = reduce_trig::<4>(1.0);
    assert_eq!(red.octant, 1, "sin(1.0): octant is 1 (NOT the bad spec note's 0)");
    // r = 1 − π/2 (mpmath, RNE-256): ebin −1, negative.
    let expected = expect4(
        true,
        -1,
        [0x921F_B544_42D1_8469, 0x898C_C517_01B8_39A2, 0x5204_9C11_14CF_98E8, 0x0417_7D4C_7627_3645],
    );
    assert_agrees(&red.r, &expected, 240);
    // leading-f64 sanity: r ≈ 1 − π/2.
    assert!((red.r.leading_f64() - (1.0 - std::f64::consts::FRAC_PI_2)).abs() < 1e-15);
    assert!(red.err_ulps < 64, "trig reduction err_ulps should be tiny, got {}", red.err_ulps);
}

/// f64-nearest-π: THE near-k·π cancellation reduction test (reduction spec §2.8,
/// the classic sin(M_PI) value). octant == 2, r = π_double − π ≈ −1.2246e-16 —
/// needs full RELATIVE precision (~53-bit cancellation) that only the wide 2/π +
/// π/2 tables deliver.
#[test]
fn trig_pi_double_near_k_pi_cancellation() {
    let red = reduce_trig::<4>(std::f64::consts::PI);
    assert_eq!(red.octant, 2);
    // reduction-only assertion from spec §6: |r + 1.2246467991473532e-16| < 2^-100.
    let (d, _) = red.r.sub(&BigFloat::<4>::from_f64(-1.224_646_799_147_353_2e-16));
    let d = d.abs();
    assert!(
        d.is_zero() || d.ebin() <= -101,
        "|r + 1.2246467991473532e-16| = 2^{} is not < 2^-100",
        d.ebin()
    );
    // deep-bit reference (mpmath 3000-bit, RNE-256): ebin −53.
    let expected = expect4(
        true,
        -53,
        [0x8D31_3198_A2E0_3707, 0x344A_4093_8222_99F3, 0x1D00_82EF_A98E_C4E6, 0xC894_5282_1E63_8D01],
    );
    assert_agrees(&red.r, &expected, 240);
}

/// x just below a multiple of π/2 (f64-nearest π/2 < true π/2): octant == 1 (the
/// round-UP boundary), r ≈ 6.1e-17 tiny. Exercises the octant-1 half_set path.
#[test]
fn trig_half_pi_boundary_octant1() {
    let red = reduce_trig::<4>(std::f64::consts::FRAC_PI_2);
    assert_eq!(red.octant, 1);
    assert!(red.r.leading_f64().abs() < 1e-15, "r near the π/2 boundary must be tiny");
}

/// Large |x| = 2^60: pure Payne–Hanek, no near-π special. octant 3, ebin −1.
#[test]
fn trig_large_2p60() {
    let red = reduce_trig::<4>(p2(60));
    assert_eq!(red.octant, 3);
    let expected = expect4(
        true,
        -1,
        [0x972C_8F1A_770E_59FF, 0x07CA_1F2E_9690_0B99, 0xC234_0B79_146B_1A60, 0xD534_05E6_8A74_E421],
    );
    assert_agrees(&red.r, &expected, 240);
}

/// Large |x| = 2^100: octant 7 (≡ 3 mod 4), ebin −1. Reaches deeper 2/π bits.
#[test]
fn trig_large_2p100() {
    let red = reduce_trig::<4>(p2(100));
    assert_eq!(red.octant, 7);
    let expected = expect4(
        false,
        -1,
        [0x82DA_94F7_70B8_637E, 0x2FA8_BE3F_5CE9_7250, 0x0337_7D18_C0F0_19A6, 0x2375_3632_44E1_93AC],
    );
    assert_agrees(&red.r, &expected, 240);
}

/// THE canonical large-arg regression `x = 1e22` (= 5^22·2^22, historic glibc
/// misround). octant 3, ebin −1. A truncated 2/π table misrounds ~100%.
#[test]
fn trig_large_1e22() {
    let red = reduce_trig::<4>(1e22);
    assert_eq!(red.octant, 3);
    let expected = expect4(
        false,
        -1,
        [0x8CF5_5CCB_19E6_BF6C, 0xB0F8_1FCE_9B42_008C, 0x5C97_BE49_71EE_3BC0, 0x40A2_FAA8_4BB9_7E2B],
    );
    assert_agrees(&red.r, &expected, 240);
}

/// The 2^1023 stressor — window `i1 ≈ 969` deep in the table; confirms the table
/// length + guard hold at the top of the finite f64 range. octant 6, ebin −1.
#[test]
fn trig_large_2p1023_stressor() {
    let x = f64::from_bits(0x7FE0_0000_0000_0000); // 2^1023
    let red = reduce_trig::<4>(x);
    assert_eq!(red.octant, 6);
    let expected = expect4(
        true,
        -1,
        [0x9921_6693_9266_6FDF, 0xBD32_1155_7AB1_98DC, 0x9EBD_5638_2F04_E388, 0xCA8D_43A5_3222_3E9D],
    );
    assert_agrees(&red.r, &expected, 240);
}

/// Small |x| < π/4: fast path returns octant 0, r = |x| exact — AND agrees with
/// the general Payne–Hanek path fed the same (M, E) (spec: both give octant 0,
/// r = |x| when |x|·2/π < 1/2).
#[test]
fn trig_small_fast_path_agrees_with_general() {
    let x = 0.7f64; // < π/4 ≈ 0.7854
    let fast = reduce_trig::<4>(x);
    assert_eq!(fast.octant, 0);
    assert_eq!(fast.err_ulps, 0);
    assert_eq!(fast.r.cmp(&BigFloat::<4>::from_f64(x)), Ordering::Equal, "fast path: r == |x| exact");

    // Force the general path on the same value via the (M, E) core entry.
    let bits = x.to_bits();
    let m = (1u64 << 52) | (bits & 0x000f_ffff_ffff_ffff);
    let e = ((bits >> 52) & 0x7ff) as i32 - 1075;
    let gen = reduce_trig_core::<4>(m, e);
    assert_eq!(gen.octant, 0);
    // general-path r reconstructs |x| to ≥240 bits.
    assert_agrees(&gen.r, &BigFloat::<4>::from_f64(x), 240);
}

/// The |x| convention: reduce_trig reduces the MAGNITUDE, so ±x give the same
/// octant and r (the atom applies sin's odd symmetry itself).
#[test]
fn trig_reduces_magnitude() {
    let pos = reduce_trig::<4>(1.0);
    let neg = reduce_trig::<4>(-1.0);
    assert_eq!(pos.octant, neg.octant);
    assert_eq!(pos.r.cmp(&neg.r), Ordering::Equal);
}

/// Escalation consistency: reducing 1e22 at 512 bits agrees with the 256-bit
/// result to the 256-bit precision (the wider table view adds correct bits, it
/// does not move the top ones).
#[test]
fn trig_escalation_512_agrees_with_256() {
    let r4 = reduce_trig::<4>(1e22);
    let r8 = reduce_trig::<8>(1e22);
    assert_eq!(r4.octant, r8.octant);
    // narrow the 512-bit r to 256 and compare.
    let (r8_narrow, _) = r8.r.narrow::<4>();
    assert_agrees(&r4.r, &r8_narrow, 240);
}

// ===========================================================================
// exp reduction (Cody–Waite)
// ===========================================================================

/// exp(1.0): k = 1, r = 1 − ln2 ≈ 0.30685 ∈ [−ln2/2, ln2/2].
#[test]
fn exp_one() {
    let red = reduce_exp::<4>(1.0);
    assert_eq!(red.k, 1);
    // r = 1 − ln2 (mpmath RNE-256): ebin −2, positive.
    let expected = expect4(
        false,
        -2,
        [0x9D1B_D010_5C61_0CA8, 0x6C38_98CF_F81A_12A1, 0x7E19_79B3_1ACE_93A4, 0xEBE5_D148_E8AA_0BA8],
    );
    assert_agrees(&red.r, &expected, 240);
    assert!((red.r.leading_f64() - (1.0 - std::f64::consts::LN_2)).abs() < 1e-15);
    assert!(red.err_ulps < 64, "exp(1.0) reduction err_ulps small, got {}", red.err_ulps);
}

/// exp(709): large k = 1023, still finite; |r| ≤ ln2/2. Exercises the |k|·ln2-tail
/// error amplification (folded, rescaled to r's ULP).
#[test]
fn exp_large_k_709() {
    let red = reduce_exp::<4>(709.0);
    assert_eq!(red.k, 1023);
    assert!(
        red.r.leading_f64().abs() < std::f64::consts::LN_2 / 2.0 + 1e-12,
        "|r| must be ≤ ln2/2, got {}",
        red.r.leading_f64()
    );
    assert!((red.r.leading_f64() - (-0.089_565_712_824_051_53)).abs() < 1e-13);
    // err is amplified by |k| ≈ 2^10 but stays a small integer (≪ 2^20).
    assert!(red.err_ulps < (1u128 << 20), "exp(709) err_ulps = {}", red.err_ulps);
}

/// exp near 0 (x = 2^-30): k = 0, r = x EXACT, err 0.
#[test]
fn exp_near_zero() {
    let x = p2(-30);
    let red = reduce_exp::<4>(x);
    assert_eq!(red.k, 0);
    assert_eq!(red.err_ulps, 0);
    assert_eq!(red.r.cmp(&BigFloat::<4>::from_f64(x)), Ordering::Equal);
}

// ===========================================================================
// log reduction (octave split)
// ===========================================================================

#[test]
fn log_one_is_e0_m1() {
    let red = reduce_log::<4>(1.0);
    assert_eq!(red.e, 0);
    assert_eq!(red.err_ulps, 0, "m is exact");
    assert_eq!(red.m.cmp(&BigFloat::<4>::from_f64(1.0)), Ordering::Equal);
}

#[test]
fn log_two_is_e1_m1() {
    let red = reduce_log::<4>(2.0);
    assert_eq!(red.e, 1);
    assert_eq!(red.m.cmp(&BigFloat::<4>::from_f64(1.0)), Ordering::Equal, "log(2)=1·ln2+log(1)");
}

#[test]
fn log_three_is_e2_m075() {
    let red = reduce_log::<4>(3.0);
    assert_eq!(red.e, 2);
    // frexp(3) = (0.75, 2); 0.75 ≥ √2/2 ⇒ m = 0.75, e = 2. m ∈ [√2/2, √2).
    assert_eq!(red.m.cmp(&BigFloat::<4>::from_f64(0.75)), Ordering::Equal);
    // sanity: m within the centred octave.
    assert!(red.m.leading_f64() >= std::f64::consts::FRAC_1_SQRT_2);
    assert!(red.m.leading_f64() < std::f64::consts::SQRT_2);
}

// ===========================================================================
// Mandatory table-length guards (reduction spec §2.3 / §3 / §5)
// ===========================================================================

/// The trig 2/π guard FIRES for a synthetic over-range exponent (no finite f64
/// reaches i1 > 2304, since E_max = 971 ⇒ i1_max = 2176).
#[test]
#[should_panic(expected = "2/pi table too short")]
fn trig_guard_panics_over_range() {
    // e = 2200 ⇒ i1 = 2200 + (256+128) + 53 = 2637 > 2304.
    let _ = reduce_trig_core::<4>(1u64 << 52, 2200);
}

/// Defense-in-depth: the `two_over_pi_window` accessor also panics past the table
/// edge (rather than zero-filling into a silently-wrong large-|x| trig value).
#[test]
#[should_panic(expected = "past the stored table")]
fn window_accessor_guard_panics_over_range() {
    let _ = consts::two_over_pi_window(1, consts::TWO_OVER_PI_BITS + 1);
}

/// The exp/log ln2 width guard panics when ln2 is ever too short.
#[test]
#[should_panic(expected = "ln2 table too short")]
fn ln2_guard_panics_when_too_short() {
    require_ln2_bits(consts::LN2_BITS + 1);
}

/// ...and does NOT panic at the exact table width (boundary is inclusive).
#[test]
fn ln2_guard_ok_at_table_width() {
    require_ln2_bits(consts::LN2_BITS);
    require_ln2_bits(64 * 16 + 11 + 64); // the real N=16 worst case (1099 ≤ 1152)
}
