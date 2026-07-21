//! Plan B Slice-0 Task T2 — `round-ziv` kernel vectors.
//!
//! These exercise the rounding/decidability core added to `hp.rs`: the
//! single-rounding round-ties-to-even reducer from a wide `BigFloat<N>` to
//! f64/f32, the exact distance-to-midpoint `D`, the Ziv `D > err_ulps`
//! decidability test, the 256 -> 512 -> 1024 escalation driver, and the
//! self-certifying `{hardness_margin_bits, stabilized_precision_bits}`
//! certificate. See the `round-ziv` section of
//! `docs/superpowers/specs/2026-07-21-planb-slice0-workflow-output.md`
//! (incl. its Review findings — the critical `avail == 0` fix).
//!
//! The atoms (exp/log/sin) are T5/T6 and do not exist yet, so every "spec
//! vector" feeds the kernel a directly-constructed `BigFloat` holding the
//! KNOWN wide value of the transcendental (mantissa computed offline with
//! mpmath at 2000-bit precision, RNE-rounded to 256 bits) plus a known
//! `err_ulps`. The kernel's job — correctly-rounded target bits + certificate —
//! is fully testable this way.

use kiss_conformance::hp::{
    round_atom_to_f32, round_atom_to_f64, round_to_f32, round_to_f64, Atom, BigFloat, EvalPrecision,
    Evaluated, FpClass,
};

const MSB: u64 = 0x8000_0000_0000_0000;

// 256-bit RNE mantissas (MSW-first, bit 255 set) of the named transcendentals.
// Cross-checked: each rounds to the spec's pinned f64/f32 bits (asserted below).
const E_MANT: [u64; 4] =
    [0xADF8_5458_A2BB_4A9A, 0xAFDC_5620_273D_3CF1, 0xD8B9_C583_CE2D_3695, 0xA9E1_3641_1464_33FC];
const LN2_MANT: [u64; 4] =
    [0xB172_17F7_D1CF_79AB, 0xC9E3_B398_03F2_F6AF, 0x40F3_4326_7298_B62D, 0x8A0D_175B_8BAA_FA2C];
const SIN1_MANT: [u64; 4] =
    [0xD76A_A478_4867_7020, 0xC6E9_E909_C50F_3C32, 0x89E5_1113_2F51_8B4D, 0xEFB6_CA5F_D6C6_49BE];
// exp(-745) = 2.82e-324 = 0.571 * smallest subnormal, ebin = -1075 (avail == 0).
const EXP_M745_MANT: [u64; 4] =
    [0x923D_731D_392D_C6DB, 0xC853_0055_E810_730B, 0xD9FD_4275_9F52_87F0, 0xCF7E_7692_646F_9040];

fn ev<const N: usize>(val: BigFloat<N>, err_ulps: u128) -> Evaluated<N> {
    Evaluated { val, err_ulps }
}

fn msb<const N: usize>() -> [u64; N] {
    let mut m = [0u64; N];
    m[0] = MSB;
    m
}

/// The exact power of two `2^ebin`, at any width.
fn pow2<const N: usize>(ebin: i32) -> BigFloat<N> {
    BigFloat::<N>::from_limbs_ebin(false, ebin, msb::<N>(), FpClass::Normal)
}

// =============================================================================
// Decided-at-256 spec vectors (clear margin, D >> err_ulps).
// =============================================================================

#[test]
fn e_f64_decides_at_256() {
    let e = BigFloat::<4>::from_limbs_ebin(false, 1, E_MANT, FpClass::Normal);
    let r = round_to_f64(&ev(e, 4));
    assert!(r.decided);
    assert_eq!(r.bits, 0x4005_BF0A_8B14_5769, "exp(1) = e");
    assert_eq!(r.eval_precision, EvalPrecision::WiderThanCompute);
    assert_eq!(r.cert.stabilized_precision_bits, 256);
    assert!(r.cert.stabilized_precision_bits > 53, "strict > f64 compute width (§6.5-0009)");
    assert_eq!(r.cert.hardness_margin_bits, 1);
}

#[test]
fn e_f32_direct_rounding() {
    // The SAME 256-bit value rounded DIRECTLY with F32 params (no f64 intermediate).
    let e = BigFloat::<4>::from_limbs_ebin(false, 1, E_MANT, FpClass::Normal);
    let r = round_to_f32(&ev(e, 4));
    assert!(r.decided);
    assert_eq!(r.bits as u32, 0x402D_F854, "exp(1) f32");
    assert_eq!(r.cert.stabilized_precision_bits, 256);
    assert!(r.cert.stabilized_precision_bits > 24, "strict > f32 compute width");
    assert_eq!(r.cert.hardness_margin_bits, 1);
}

#[test]
fn ln2_f64_and_f32() {
    let ln2 = BigFloat::<4>::from_limbs_ebin(false, -1, LN2_MANT, FpClass::Normal);
    let r64 = round_to_f64(&ev(ln2.clone(), 4));
    assert!(r64.decided);
    assert_eq!(r64.bits, 0x3FE6_2E42_FEFA_39EF, "log(2) = ln2");
    assert_eq!(r64.cert.hardness_margin_bits, 0);
    let r32 = round_to_f32(&ev(ln2, 4));
    assert_eq!(r32.bits as u32, 0x3F31_7218, "ln2 f32");
    assert_eq!(r32.cert.hardness_margin_bits, 0);
}

#[test]
fn sin1_f64_and_f32() {
    let s = BigFloat::<4>::from_limbs_ebin(false, -1, SIN1_MANT, FpClass::Normal);
    let r64 = round_to_f64(&ev(s.clone(), 4));
    assert_eq!(r64.bits, 0x3FEA_ED54_8F09_0CEE, "sin(1)");
    assert_eq!(r64.cert.hardness_margin_bits, 0);
    let r32 = round_to_f32(&ev(s, 4));
    assert_eq!(r32.bits as u32, 0x3F57_6AA4, "sin(1) f32");
    assert_eq!(r32.cert.hardness_margin_bits, 4);
}

// =============================================================================
// CRITICAL FIX: the subnormal `avail == 0` straddle boundary.
// =============================================================================

/// exp(-745) = 1.1735·2^-1075 (0.571× the smallest subnormal): E = -1075,
/// avail = 53 - (-1022 - (-1075)) = 0. Under the OLD `avail <= 0 => signed zero`
/// rule this misrounded to 0x0; RNE actually rounds it UP to 0x…01 because the
/// sticky bits push it past the midpoint. This is the spec's own anchor.
#[test]
fn avail0_subnormal_rounds_up_not_to_zero() {
    let x = BigFloat::<4>::from_limbs_ebin(false, -1075, EXP_M745_MANT, FpClass::Normal);
    let r = round_to_f64(&ev(x, 1));
    assert!(r.decided);
    assert_eq!(r.bits, 0x0000_0000_0000_0001, "smallest positive subnormal, NOT +0");
}

/// The other side of the same boundary: exactly 2^-1075 (the midpoint between 0
/// and the smallest subnormal) with zero error rounds to +0 by ties-to-even
/// (0 is even). avail == 0, D == 0, err == 0 => exact => decided.
#[test]
fn avail0_exact_half_ties_to_even_zero() {
    let half = BigFloat::<4>::from_limbs_ebin(false, -1075, [MSB, 0, 0, 0], FpClass::Normal);
    let r = round_to_f64(&ev(half, 0));
    assert!(r.decided, "an exact value (err_ulps==0) always decides, even on a midpoint");
    assert_eq!(r.bits, 0x0000_0000_0000_0000, "ties-to-even -> +0");
}

// =============================================================================
// Overflow / underflow / special classes (round-ziv §7).
// =============================================================================

#[test]
fn overflow_to_inf() {
    // value = 2^1024, E = 1024 > emax = 1023 -> +Inf (the exp(710) anchor shape).
    let big = BigFloat::<4>::from_limbs_ebin(false, 1024, [MSB, 0, 0, 0], FpClass::Normal);
    let r = round_to_f64(&ev(big, 0));
    assert!(r.decided);
    assert_eq!(r.bits, 0x7FF0_0000_0000_0000, "+Inf");
}

#[test]
fn underflow_to_zero() {
    // value = 2^-1077, avail = 53 - 55 = -2 < 0 -> +0 (the exp(-746) anchor shape).
    let tiny = BigFloat::<4>::from_limbs_ebin(false, -1077, [MSB, 0, 0, 0], FpClass::Normal);
    let r = round_to_f64(&ev(tiny, 0));
    assert!(r.decided);
    assert_eq!(r.bits, 0x0000_0000_0000_0000, "+0");
}

#[test]
fn special_classes_bypass_rounding() {
    // Inf / NaN / signed zero from the atom -> target special, decided, cert emits
    // stabilized = 64*N (keeps the §6.5-0009 strict-> invariant true).
    let pinf = round_to_f64(&ev(BigFloat::<4>::from_f64(f64::INFINITY), 0));
    assert_eq!(pinf.bits, 0x7FF0_0000_0000_0000);
    assert!(pinf.decided);
    assert_eq!(pinf.cert.stabilized_precision_bits, 256);

    let ninf = round_to_f64(&ev(BigFloat::<4>::from_f64(f64::NEG_INFINITY), 0));
    assert_eq!(ninf.bits, 0xFFF0_0000_0000_0000);

    let nan = round_to_f64(&ev(BigFloat::<4>::from_f64(f64::NAN), 0));
    assert_eq!(nan.bits, 0x7FF8_0000_0000_0000, "canonical quiet NaN");

    let nzero = round_to_f64(&ev(BigFloat::<4>::from_f64(-0.0), 0));
    assert_eq!(nzero.bits, 0x8000_0000_0000_0000, "-0.0");

    // f32 specials
    let pinf32 = round_to_f32(&ev(BigFloat::<4>::from_f64(f64::INFINITY), 0));
    assert_eq!(pinf32.bits as u32, 0x7F80_0000);
    let nan32 = round_to_f32(&ev(BigFloat::<4>::from_f64(f64::NAN), 0));
    assert_eq!(nan32.bits as u32, 0x7FC0_0000);
}

// =============================================================================
// Overflow / underflow BOUNDARY straddles (review fix): the max-finite<->Inf and
// 0<->smallest-subnormal midpoints run the Ziv D-test, NOT an unconditional
// short-circuit, so a value whose error interval crosses the boundary escalates
// instead of being mis-certified.
// =============================================================================

// The overflow (max-finite <-> Inf) midpoint is 2^1024 - 2^970 = the value
// halfway between max-finite (2^1024 - 2^971) and 2^1024. Built exactly from
// powers of two, at any width. `- 5 ULPs` sits just below it (rounds to
// max-finite), `+ 5 ULPs` just above (carries to Inf); D = 5 in both cases.
fn overflow_mid_minus_5<const N: usize>() -> BigFloat<N> {
    let mid = pow2::<N>(1024).sub(&pow2::<N>(970)).0; // 2^1024 - 2^970
    mid.sub(&BigFloat::<N>::from_i64(5).ldexp(768)).0 // - 5 * 2^768  (a fixed real)
}

#[test]
fn overflow_boundary_below_decides_to_max_finite() {
    // clear margin below the midpoint (D = 5 > err = 4) -> decided, max-finite.
    let r = round_to_f64(&ev(overflow_mid_minus_5::<4>(), 4));
    assert!(r.decided);
    assert_eq!(r.bits, 0x7FEF_FFFF_FFFF_FFFF, "largest finite f64");
}

#[test]
fn overflow_boundary_straddle_does_not_decide() {
    // err interval reaches the midpoint (D = 5, err = 5) -> straddle, NOT decided.
    let r = round_to_f64(&ev(overflow_mid_minus_5::<4>(), 5));
    assert!(!r.decided, "interval straddles the max-finite<->Inf midpoint => escalate");
}

#[test]
fn overflow_boundary_above_decides_to_inf() {
    // just above the midpoint (carry past max-finite), clear margin -> decided, Inf.
    let mid = pow2::<4>(1024).sub(&pow2::<4>(970)).0;
    let above = mid.add(&BigFloat::<4>::from_i64(5).ldexp(768)).0; // + 5 ULPs
    let r = round_to_f64(&ev(above, 4));
    assert!(r.decided);
    assert_eq!(r.bits, 0x7FF0_0000_0000_0000, "+Inf via the RNE overflow threshold");
}

/// A value below the overflow midpoint whose error bound crosses it at 256 bits
/// but tightens by 512. The driver must escalate and resolve to max-finite — NOT
/// force Inf/decided at 256. (The value re-derives faithfully at each width; the
/// err_ulps shrink models an atom whose accumulated bound tightens with precision.)
struct OverflowStraddle;
impl Atom for OverflowStraddle {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        let err = if N == 4 { 5 } else { 1 };
        Evaluated { val: overflow_mid_minus_5::<N>(), err_ulps: err }
    }
}

#[test]
fn overflow_boundary_escalates_to_max_finite() {
    assert!(!round_to_f64(&OverflowStraddle.eval::<4>()).decided, "straddles at 256");
    let r = round_atom_to_f64(&OverflowStraddle);
    assert!(r.decided);
    assert_eq!(r.bits, 0x7FEF_FFFF_FFFF_FFFF, "resolves to max-finite, not Inf");
    assert_eq!(r.cert.stabilized_precision_bits, 512);
}

// The underflow (0 <-> smallest-subnormal) midpoint is 2^-1075. A value in
// [2^-1076, 2^-1075) has avail == -1: strictly below the midpoint (RNE -> +0),
// but its interval can cross into the smallest-subnormal cell. `2^-1075 - 5 ULPs`
// gives D = 5.
fn underflow_below_mid_by_5<const N: usize>() -> BigFloat<N> {
    // 2^-1075 - 5 * 2^-1331  (a fixed real; ebin = -1076 => avail == -1).
    pow2::<N>(-1075).sub(&BigFloat::<N>::from_i64(5).ldexp(-1331)).0
}

#[test]
fn underflow_boundary_below_decides_to_signed_zero() {
    // clear margin below the midpoint (D = 5 > err = 4) -> decided, +0 / -0.
    let v = underflow_below_mid_by_5::<4>();
    let rp = round_to_f64(&ev(v.clone(), 4));
    assert!(rp.decided);
    assert_eq!(rp.bits, 0x0000_0000_0000_0000, "+0");
    let rn = round_to_f64(&ev(v.neg(), 4));
    assert!(rn.decided);
    assert_eq!(rn.bits, 0x8000_0000_0000_0000, "-0 (sign preserved)");
}

#[test]
fn underflow_boundary_straddle_does_not_decide() {
    // err interval reaches the midpoint (D = 5, err = 5) -> straddle, NOT decided.
    let r = round_to_f64(&ev(underflow_below_mid_by_5::<4>(), 5));
    assert!(!r.decided, "interval straddles the 0<->subnormal midpoint => escalate");
}

#[test]
fn underflow_avail_neg2_is_unconditional_zero() {
    // avail <= -2: >= 2^P ULPs below the midpoint, decided even for a huge err
    // (which a real u128 err can never reach) -> the unconditional short-circuit.
    let v = BigFloat::<4>::from_limbs_ebin(false, -1077, [MSB, 0, 0, 0], FpClass::Normal);
    let r = round_to_f64(&ev(v, 1u128 << 100));
    assert!(r.decided);
    assert_eq!(r.bits, 0x0000_0000_0000_0000);
}

/// Symmetric to `OverflowStraddle`: a value below the underflow midpoint whose
/// bound crosses it at 256 bits and tightens by 512. The driver escalates and
/// resolves to signed zero. (The "above the midpoint -> smallest subnormal" side
/// is the decided `avail == 0` case pinned by `avail0_subnormal_rounds_up...`.)
struct UnderflowStraddle;
impl Atom for UnderflowStraddle {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        let err = if N == 4 { 5 } else { 1 };
        Evaluated { val: underflow_below_mid_by_5::<N>(), err_ulps: err }
    }
}

#[test]
fn underflow_boundary_escalates_to_zero() {
    assert!(!round_to_f64(&UnderflowStraddle.eval::<4>()).decided, "straddles at 256");
    let r = round_atom_to_f64(&UnderflowStraddle);
    assert!(r.decided);
    assert_eq!(r.bits, 0x0000_0000_0000_0000, "resolves to +0");
    assert_eq!(r.cert.stabilized_precision_bits, 512);
}

// =============================================================================
// f32 single-rounding discipline (no double rounding).
// =============================================================================

/// X = 1 + 2^-24 + 2^-60. Direct RN-f32(X) rounds UP (X is above the f32 midpoint
/// 1+2^-24) to 0x3F800001. But RN-f64(X) = 1+2^-24 exactly (the 2^-60 tail is
/// below the f64 ULP), and RN-f32 of THAT ties-to-even DOWN to 0x3F800000. The
/// kernel must round the wide value directly to f32 and give 0x3F800001.
#[test]
fn f32_single_rounding_avoids_double_rounding() {
    let x = {
        let one = BigFloat::<4>::from_f64(1.0);
        let (a, _) = one.add(&BigFloat::<4>::from_f64(2f64.powi(-24)));
        let (b, _) = a.add(&BigFloat::<4>::from_f64(2f64.powi(-60)));
        b
    };
    let direct = round_to_f32(&ev(x.clone(), 0)).bits as u32;
    assert_eq!(direct, 0x3F80_0001, "direct single-rounding rounds up");

    // Demonstrate the hazard: rounding to f64 first then narrowing gives the
    // WRONG answer, which the kernel avoids.
    let f64_bits = round_to_f64(&ev(x, 0)).bits;
    let via_f64 = (f64::from_bits(f64_bits) as f32).to_bits();
    assert_eq!(via_f64, 0x3F80_0000, "double rounding would tie-to-even down");
    assert_ne!(direct, via_f64, "single rounding differs from double rounding here");
}

// =============================================================================
// Ziv straddle + escalation (256 -> 512).
// =============================================================================

/// A wide value sitting EXACTLY on an f64 midpoint with err_ulps > 0 straddles:
/// the error interval contains the midpoint, so the single-width rounding does
/// NOT decide (the driver must escalate). M = 1 + 2^-53 is the midpoint between
/// 1.0 and the next double.
#[test]
fn straddle_single_width_does_not_decide() {
    let one = BigFloat::<4>::from_f64(1.0);
    let (m, _) = one.add(&BigFloat::<4>::from_f64(2f64.powi(-53)));
    let r = round_to_f64(&ev(m, 1));
    assert!(!r.decided, "D == 0 with err_ulps > 0 is a straddle, not decided");
}

/// The true value M + 2^-300 straddles at 256 bits (the 2^-300 offset is below
/// the 256-bit ULP so the value collapses onto the midpoint, D == 0) but
/// resolves at 512 bits (the offset becomes a real bit, D > 0). The escalation
/// driver re-derives at the wider width and returns the 512-bit decision.
struct MidStraddle;
impl Atom for MidStraddle {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        let one = BigFloat::<N>::from_f64(1.0);
        let (m, e1) = one.add(&BigFloat::<N>::from_f64(2f64.powi(-53))); // f64 midpoint
        let (v, e2) = m.add(&BigFloat::<N>::from_f64(2f64.powi(-300))); // + tiny offset
        Evaluated { val: v, err_ulps: e1 + e2 }
    }
}

#[test]
fn escalation_resolves_at_512() {
    // Single-shot at N=4 straddles...
    let at256 = round_to_f64(&MidStraddle.eval::<4>());
    assert!(!at256.decided, "straddles at 256");
    // ...and at N=8 decides.
    let at512 = round_to_f64(&MidStraddle.eval::<8>());
    assert!(at512.decided, "decides at 512");

    // The driver walks 256 -> 512 and returns the 512-bit decision.
    let r = round_atom_to_f64(&MidStraddle);
    assert!(r.decided);
    assert_eq!(r.bits, 0x3FF0_0000_0000_0001, "rounds UP: value is above the midpoint");
    assert_eq!(r.cert.stabilized_precision_bits, 512, "P at which D > err_ulps first held");
    assert!(r.cert.stabilized_precision_bits > 53);
    // f32 driver on the same atom decides immediately at 256 (offset >> f32 ULP).
    let r32 = round_atom_to_f32(&MidStraddle);
    assert!(r32.decided);
    assert_eq!(r32.cert.stabilized_precision_bits, 256);
}

/// A value that is exactly an f64 midpoint at EVERY width with err_ulps > 0 never
/// decides — the red-flag case (a mis-routed exact/special value or an unsound
/// error bound). The driver must panic at 1024, not loop or emit garbage.
struct AlwaysMidpoint;
impl Atom for AlwaysMidpoint {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        let one = BigFloat::<N>::from_f64(1.0);
        let (m, _) = one.add(&BigFloat::<N>::from_f64(2f64.powi(-53)));
        Evaluated { val: m, err_ulps: 1 } // exact midpoint, but a nonzero bound
    }
}

#[test]
#[should_panic(expected = "unresolved at 1024")]
fn escalation_panics_when_unresolvable() {
    let _ = round_atom_to_f64(&AlwaysMidpoint);
}
