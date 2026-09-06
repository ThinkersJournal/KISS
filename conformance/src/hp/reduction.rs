//! Plan B Slice-0 Task T4 — `hp::reduction`: argument reduction that feeds the
//! transcendental atoms (T5 exp/log, T6 sin).
//!
//! Three reductions sit between the [`BigFloat`](super::BigFloat) core (T1) + the
//! wide constant tables (`hp::consts`, T3) and the atoms:
//!
//!   * [`reduce_trig`] — **Payne–Hanek** trig reduction: any finite `|x|` up to
//!     ~`2^1023` → an `octant` (`k mod 8`) and a reduced `r ∈ [-π/4, π/4]`,
//!     retaining FULL RELATIVE precision even under the ~61–67-bit cancellation
//!     that hits `x` near a multiple of `π/2` (near-k·π). This is the whole point
//!     of the wide `2/π` table: the low fraction bits are load-bearing when the
//!     product's integer part is large.
//!   * [`reduce_exp`] — **Cody–Waite** exp reduction: `x = k·ln2 + r`,
//!     `k = round(x·(1/ln2))`, `r ∈ [-ln2/2, ln2/2]`.
//!   * [`reduce_log`] — log octave reduction: `x = 2^e · m`, `m ∈ [√2/2, √2)`, so
//!     `log(x) = e·ln2 + log(m)` with `m` centred around 1.
//!
//! # Error contract (matches round-ziv §0)
//!
//! Each reduction returns a rigorous `err_ulps: u128` such that the returned
//! reduced value `v` satisfies `|v − v_true| ≤ err_ulps · 2^(v.exp)` — i.e. in
//! units of the returned value's own ULP, ready to hand [`super::Evaluated`] to
//! the round-ziv kernel. The constant-reader errors (`ln2`, `π/2` tails) and the
//! `2/π` truncation tail are FOLDED into that budget, rescaled to the result ULP.
//!
//! # Sign / octant convention (see the crate report — a T4 judgment call)
//!
//! [`reduce_trig`] reduces `|x|` and returns `octant = round(|x|·2/π) mod 8` with
//! `r` the reduced argument of `|x|`. The atom applies the odd/even symmetry for
//! `x < 0` itself (`sin(-x) = -sin(x)`), exactly as the reduction spec §2.1
//! mandates ("reduce `|x|` only; the caller applies … symmetry"). sin/cos select
//! their identity from `octant mod 4`, tan from `octant mod 2`.
//!
//! # Table-length guards (mandatory — reduction spec §2.3 / §3 / §5)
//!
//! A too-short constant table is the kernel's MOST dangerous failure: the error
//! is systematic across precisions, so the Ziv escalation is *blind* to it and
//! would certify a confidently-wrong value. Every reduction therefore begins with
//! a hard `assert!` that the required constant window fits the stored table, and
//! [`super::consts::two_over_pi_window`] re-checks the `2/π` bound at the table's
//! own edge (defense-in-depth). These panic rather than certify.

use super::consts;
use super::{
    lsw_bit, lsw_mask_low, lsw_mul, lsw_set_bit, lsw_shr, lsw_sub, normalize_round, BigFloat, FpClass,
};

// ===========================================================================
// Return types
// ===========================================================================

/// Payne–Hanek trig reduction result for `|x|`. `sin(|x|)`/`cos(|x|)` are
/// reconstructed by the atom from `octant mod 4` over `{sin(r), cos(r), −sin(r),
/// −cos(r)}`; tan from `octant mod 2`.
#[derive(Clone, Debug)]
pub struct TrigReduction<const N: usize> {
    /// `round(|x|·2/π) mod 8` (0..=7).
    pub octant: u32,
    /// reduced argument `r ∈ [-π/4, π/4]`, at working width `N`.
    pub r: BigFloat<N>,
    /// rigorous bound: `|r − r_true| ≤ err_ulps · 2^(r.exp)`.
    pub err_ulps: u128,
}

/// Cody–Waite exp reduction: `x = k·ln2 + r`, so `exp(x) = 2^k · exp(r)`.
#[derive(Clone, Debug)]
pub struct ExpReduction<const N: usize> {
    /// the integer power-of-two the atom applies as an exact `ldexp` after the
    /// series. The atom pre-clamps large `|x|` to `±Inf/±0`, so `|k| ≤ ~2^11`.
    pub k: i64,
    /// reduced argument `r ∈ [-ln2/2, ln2/2]` (roughly; see the `round_to_i64`
    /// note), at working width `N`.
    pub r: BigFloat<N>,
    /// rigorous bound: `|r − r_true| ≤ err_ulps · 2^(r.exp)`. Folds the `ln2`
    /// truncation (`|k|·tail`), the `k·ln2` rounding, and the subtraction round.
    pub err_ulps: u128,
}

/// log octave reduction: `x = 2^e · m`, so `log(x) = e·ln2 + log(m)`.
#[derive(Clone, Debug)]
pub struct LogReduction<const N: usize> {
    /// the exact octave exponent (`|e| ≤ 1074`). The atom forms `e·ln2` and adds
    /// it to `log(m)`.
    pub e: i32,
    /// the reduced mantissa `m ∈ [√2/2, √2)`, **EXACT** (frexp + the ×2 octave
    /// fold are exact), at working width `N`.
    pub m: BigFloat<N>,
    /// error of `m` in units of `2^(m.exp)`. Always **0**: the reduction adds no
    /// error to `m` (reduction spec §4). The `e·ln2` additive term's error is the
    /// atom's to fold from the `ln2` reader (bound `|e|·2^(ln2.exp)`) — it depends
    /// on the final result ULP, which only the atom knows.
    pub err_ulps: u128,
}

// ===========================================================================
// Small numeric helpers
// ===========================================================================

/// Bit length of `|k|` (0 for k == 0).
fn bitlen_i64(k: i64) -> usize {
    (64 - k.unsigned_abs().leading_zeros()) as usize
}

/// Rescale an error of `count` ULPs measured at exponent `src_exp` into ULPs
/// measured at exponent `dst_exp`, rounding **up** (a rigorous over-estimate).
/// Public so the transcendental atoms (T5/T6) fold their sub-term errors onto the
/// result ULP with the same rounding-up discipline the reduction uses.
pub fn rescale_ulps(count: u128, src_exp: i32, dst_exp: i32) -> u128 {
    if count == 0 {
        return 0;
    }
    let d = src_exp - dst_exp;
    if d >= 0 {
        // src ULP is coarser (or equal): scale up.
        let sh = d as u32;
        if sh >= 128 {
            u128::MAX
        } else {
            count.saturating_mul(1u128 << sh)
        }
    } else {
        // src ULP is finer: the error is < count · 2^d dst-ULPs; ceil it.
        let sh = (-d) as u32;
        if sh >= 128 {
            1 // strictly less than one dst-ULP, but nonzero ⇒ round up to 1
        } else {
            let q = count >> sh;
            let rem = count & ((1u128 << sh) - 1);
            q + u128::from(rem != 0)
        }
    }
}

/// The `ln2`-table width guard shared by [`reduce_exp`] and [`reduce_log`]
/// (reduction spec §3 / §5): a too-short `ln2` yields a systematic,
/// precision-independent error the Ziv loop cannot see, so a table that is ever
/// too short must PANIC, never certify. Exposed so the guard itself is directly
/// testable (no naturally-occurring f64 argument drives it past the table).
pub fn require_ln2_bits(need_bits: usize) {
    assert!(
        need_bits <= consts::LN2_BITS,
        "reduce: ln2 table too short: need {need_bits} bits, have {} \
         (§5 systematic-error guard — panic, never certify)",
        consts::LN2_BITS
    );
}

/// Reduction guard slack (bits) beyond the working precision + cancellation.
const LN2_REDUCTION_GUARD: usize = 64;

// ===========================================================================
// Payne–Hanek trig reduction
// ===========================================================================

/// Payne–Hanek reduce a finite `x` (any magnitude up to the f64 range) to
/// `(octant, r ∈ [-π/4, π/4])`. Reduces `|x|`; see the module note on the
/// sign/octant convention.
///
/// Small `|x| < π/4` takes a fast path (`octant = 0`, `r = |x|` exact); it agrees
/// with the general Payne–Hanek path, which returns the same `octant 0, r = |x|`
/// for `|x|·2/π < 1/2` (verified by tests). The fast path is also *required* for
/// tiny `|x|` where the general window would underflow (`i1 < i0`).
pub fn reduce_trig<const N: usize>(x: f64) -> TrigReduction<N> {
    debug_assert!(x.is_finite(), "reduce_trig: non-finite x must be special-valued upstream");
    let ax = x.abs();
    if ax < std::f64::consts::FRAC_PI_4 {
        // |x| < π/4 ⇒ |x|·2/π < 1/2 (FRAC_PI_4 ≤ true π/4), so octant 0, r = |x|.
        return TrigReduction { octant: 0, r: BigFloat::<N>::from_f64(ax), err_ulps: 0 };
    }
    let (m, e) = decompose_f64_pos(ax);
    reduce_trig_core::<N>(m, e)
}

/// Decompose a positive **normal** f64 `x = M · 2^E`, `M` the 53-bit integer
/// significand (`2^52 ≤ M < 2^53`), `E` its exponent. `x ≥ π/4` is normal, so no
/// subnormal branch is needed here.
fn decompose_f64_pos(x: f64) -> (u64, i32) {
    let bits = x.to_bits();
    let exp_field = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & 0x000f_ffff_ffff_ffff;
    debug_assert!(exp_field != 0 && exp_field != 0x7ff, "decompose_f64_pos: x not a positive normal");
    ((1u64 << 52) | frac, exp_field - 1075)
}

/// The Payne–Hanek core, parameterised by the integer significand `m` (53-bit)
/// and exponent `e` of `|x| = m·2^e`. Public so the mandatory table-length guard
/// is directly exercisable with a synthetic over-range `e` (no finite f64 reaches
/// `i1 > 2304`, since `E_max = 971 ⇒ i1_max = 2176`), and so a future wider input
/// path can reuse it.
pub fn reduce_trig_core<const N: usize>(m: u64, e: i32) -> TrigReduction<N> {
    const P_DTYPE: i32 = 53; // f64 significand width (a superset of f32's 24)
    let f_bits = 64 * N as i32 + consts::G_C as i32; // F = 64·N + G_C  (retained fraction bits)

    // --- window + the HARD table-length guard (reduction spec §2.3 / §5). ---
    let i0 = (e - 2).max(1);
    let i1 = e + f_bits + P_DTYPE;
    assert!(
        i1 <= consts::TWO_OVER_PI_BITS as i32,
        "reduce_trig: 2/pi table too short: need bit {i1} for |x| = M·2^{e} (E), \
         table is {} bits — panic, never certify (reduction spec §5)",
        consts::TWO_OVER_PI_BITS
    );

    // --- gather the 2/π window, multiply by M, align (reduction spec §2.4/§2.5). ---
    let (win, _nbits) = consts::two_over_pi_window(i0 as usize, i1 as usize);
    let prod = lsw_mul(&win, &[m]); // LSW-first big int
    // scaled = prod >> P_DTYPE == floor(x·(2/π)·2^F), because E − i1 + F = −p.
    let mut scaled = prod;
    lsw_shr(&mut scaled, P_DTYPE as u32);

    let f = f_bits as usize;
    // n_low3 = integer part of x·2/π, mod 8 = bits [F, F+3) of `scaled`.
    let n_low3 = (lsw_bit(&scaled, f) | (lsw_bit(&scaled, f + 1) << 1) | (lsw_bit(&scaled, f + 2) << 2))
        as u32;
    // half_set = f ≥ 1/2 ? = bit (F−1) of the fraction.
    let half_set = lsw_bit(&scaled, f - 1) == 1;
    // frac = scaled mod 2^F (the F fractional bits).
    let mut frac = scaled;
    lsw_mask_low(&mut frac, f);

    // --- round to nearest integer; form octant and f' (reduction spec §2.6). ---
    let (fprime, fprime_neg, octant) = if half_set {
        // fprime = 2^F − frac  ⇒ |f'| = 1 − f ∈ (0, 1/2]; f' < 0.
        let mut two_f = vec![0u64; frac.len() + 1];
        lsw_set_bit(&mut two_f, f);
        lsw_sub(&mut two_f, &frac); // 2^F − frac (frac < 2^F ⇒ no final borrow)
        (two_f, true, (n_low3 + 1) & 7)
    } else {
        // fprime = frac ⇒ f' = f ∈ [0, 1/2); f' ≥ 0.
        (frac, false, n_low3 & 7)
    };

    // --- build r = f'·(π/2) (reduction spec §2.7). fprime has LSB weight 2^-F. ---
    let (r, err_ulps) = match normalize_round::<N>(&fprime, -(f as i32), false) {
        None => {
            // fprime == 0 ⇒ exact multiple of π/2. Impossible for a finite x ≠ 0
            // (x·2/π is irrational), but return a clean signed-zero if it happens.
            return TrigReduction { octant, r: BigFloat::<N>::zero(false), err_ulps: 0 };
        }
        Some((mant, exp, inexact)) => {
            let rp = BigFloat::<N> { sign: false, exp, mant, class: FpClass::Normal };
            // e_rp (in rp's ULP): the discarded 2/π tail perturbs r by < 2^-F,
            // which is < 1 ULP of rp for every reachable |r| ≥ 2^-68 (count 1),
            // plus the fraction-truncation round (`inexact`).
            let e_rp = 1u128 + u128::from(inexact);
            let (p2, e_p2) = consts::pi_over_2::<N>();
            let (r_full, mul_err) = rp.mul(&p2);
            // Both operands are P-bit normalized ⇒ 1 ULP of either == 1 ULP of the
            // product (relative), so the mul error is additive (design spec §9).
            let err = e_rp + e_p2 + mul_err;
            let r = if fprime_neg { r_full.neg() } else { r_full };
            (r, err)
        }
    };
    TrigReduction { octant, r, err_ulps }
}

// ===========================================================================
// exp reduction (Cody–Waite)
// ===========================================================================

/// Cody–Waite reduce `exp` argument `x` to `(k, r)` with `x = k·ln2 + r`,
/// `k = round(x·(1/ln2))`. The atom pre-clamps `|x| > ~709.78` / `< ~-745.13` to
/// `±Inf/±0` BEFORE calling, so `k` fits `i64` (asserted).
pub fn reduce_exp<const N: usize>(x: f64) -> ExpReduction<N> {
    debug_assert!(x.is_finite(), "reduce_exp: non-finite x must be special-valued upstream");
    let xb = BigFloat::<N>::from_f64(x);

    // k = round(x·(1/ln2)) via the BigFloat reference (guarantees |r| ≤ ln2/2,
    // avoiding the f64-fast-path half-integer drift the review's MINOR flags).
    let (inv, _e_inv) = consts::inv_ln2::<N>();
    let (prod, _e_prod) = xb.mul(&inv);
    let k = prod.round_to_i64();
    // Defense (spec §3): the atom pre-clamps, so k is tiny; assert it stays well
    // inside round_to_i64's < 2^62 precondition rather than trust the caller.
    assert!(
        k.unsigned_abs() < (1u64 << 32),
        "reduce_exp: k = {k} out of range — the atom must pre-clamp large |x| to ±Inf/±0"
    );
    // ln2 width guard (spec §3/§5): need working precision + k-cancellation + guard.
    require_ln2_bits(64 * N + bitlen_i64(k) + LN2_REDUCTION_GUARD);

    if k == 0 {
        // r = x exactly (no ln2 involved).
        return ExpReduction { k: 0, r: xb, err_ulps: 0 };
    }

    let (l2, err_l2) = consts::ln2::<N>();
    let (kl2, e_mul) = l2.mul_small_int(k);
    let (r, e_sub) = xb.sub(&kl2);

    // err in r's ULP = ln2 truncation (|k|·err_l2 @ l2.exp) + k·ln2 rounding
    // (e_mul @ kl2.exp) + the subtraction round (e_sub, already @ r.exp). The
    // rescale-to-r.exp automatically inflates the input error under the
    // x ≈ k·ln2 cancellation (r tiny ⇒ r.exp very negative) — exactly right.
    let mut err = e_sub;
    err += rescale_ulps((k.unsigned_abs() as u128) * err_l2, l2.exp2(), r.exp2());
    err += rescale_ulps(e_mul, kl2.exp2(), r.exp2());
    ExpReduction { k, r, err_ulps: err }
}

// ===========================================================================
// log reduction (octave split)
// ===========================================================================

/// Octave-reduce a positive finite `log` argument `x` to `(e, m)` with
/// `x = 2^e · m`, `m ∈ [√2/2, √2)` (centred around 1 via the `SQRT1_2` split), so
/// `log(x) = e·ln2 + log(m)`. `m` is EXACT.
pub fn reduce_log<const N: usize>(x: f64) -> LogReduction<N> {
    debug_assert!(x.is_finite() && x > 0.0, "reduce_log: x must be a positive finite (domain routed upstream)");
    let (m0, e0) = frexp(x); // x = m0·2^e0, m0 ∈ [0.5, 1)
    let (m_f64, e) = if m0 < consts::SQRT1_2 {
        (m0 * 2.0, e0 - 1) // ×2 is exact; centres m ∈ [√2/2, 1)·… ⇒ [√2/2, √2)
    } else {
        (m0, e0)
    };
    // ln2 width guard (spec §5): the atom will add e·ln2; ln2 must cover it.
    require_ln2_bits(64 * N + bitlen_i64(e as i64) + LN2_REDUCTION_GUARD);
    let m = BigFloat::<N>::from_f64(m_f64); // EXACT (frexp + ×2 are exact)
    LogReduction { e, m, err_ulps: 0 }
}

/// `frexp`: split a finite `x ≠ 0` into `(m, e)` with `x = m·2^e`, `m ∈ [0.5, 1)`
/// (sign preserved). Handles subnormals by pre-scaling. Exact.
fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 {
        return (x, 0); // preserves signed zero (not reached by reduce_log)
    }
    let mut e = 0i32;
    let mut ax = x;
    if ax.abs() < f64::MIN_POSITIVE {
        // subnormal: scale into the normal range, then correct the exponent.
        ax *= (1u64 << 54) as f64; // 2^54
        e -= 54;
    }
    let bits = ax.to_bits();
    let exp_field = ((bits >> 52) & 0x7ff) as i32;
    // Force the biased exponent to 1022 (unbiased −1) ⇒ mantissa ∈ [0.5, 1).
    let m = f64::from_bits((bits & 0x800f_ffff_ffff_ffff) | (1022u64 << 52));
    e += exp_field - 1022;
    (m, e)
}
