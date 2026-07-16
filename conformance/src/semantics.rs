//! From-scratch CPU oracle for the pinned floating-point primitive semantics
//! (KISS-Ops §2.3 and the §6.15 reference decompositions).
//!
//! This is a Conform §6.5 differential oracle: each function is the op's *reference
//! decomposition transcribed directly*, sharing no lowering code with any generator.
//! Its purpose is to make the load-bearing numeric distinctions the spec pins in
//! prose — NaN-propagating vs NaN-suppressing min/max, and `relu` ≠ `max(x,0)` —
//! executable, so an implementation under test can be differenced against them.
//!
//! Each `!= a` test below is exactly `cmp_ne(a, a)` (true iff `a` is NaN), and each
//! `>=` / `<=` is `cmp_ge` / `cmp_le`; the branch structure mirrors the §6.15
//! `select(...)` decomposition one-for-one.

/// `max_prop` — NaN-**propagating** maximum (`torch.maximum`). §6.15:
/// `select(cmp_ne(a,a), a, select(cmp_ne(b,b), b, select(cmp_ge(a,b), a, b)))`.
pub fn max_prop(a: f32, b: f32) -> f32 {
    if a != a { a } else if b != b { b } else if a >= b { a } else { b }
}

/// `min_prop` — NaN-**propagating** minimum (`torch.minimum`). §6.15:
/// `select(cmp_ne(a,a), a, select(cmp_ne(b,b), b, select(cmp_le(a,b), a, b)))`.
pub fn min_prop(a: f32, b: f32) -> f32 {
    if a != a { a } else if b != b { b } else if a <= b { a } else { b }
}

/// `fmax_ieee` — NaN-**suppressing** maximum (IEEE-754 maxNum). §6.15:
/// `select(cmp_ne(a,a), b, select(cmp_ne(b,b), a, select(cmp_ge(a,b), a, b)))`.
pub fn fmax_ieee(a: f32, b: f32) -> f32 {
    if a != a { b } else if b != b { a } else if a >= b { a } else { b }
}

/// `fmin_ieee` — NaN-**suppressing** minimum (IEEE-754 minNum). §6.15:
/// `select(cmp_ne(a,a), b, select(cmp_ne(b,b), a, select(cmp_le(a,b), a, b)))`.
pub fn fmin_ieee(a: f32, b: f32) -> f32 {
    if a != a { b } else if b != b { a } else if a <= b { a } else { b }
}

/// `relu` — `x < 0 ? 0 : x`. NaN-**propagating** and `-0.0`-preserving (§2.3).
/// It is **not** `max(x, 0)`, which would scrub NaN and normalize `-0.0`.
pub fn relu(x: f32) -> f32 {
    if x < 0.0 {
        0.0
    } else {
        x
    }
}

/// The wrong-but-common `max(x, 0)` that §2.3 warns `relu` is **not** — provided
/// so the divergence is a test rather than a footnote. (Its sign of zero is
/// platform-unspecified; only its NaN-scrubbing is asserted.)
pub fn naive_max_x_zero(x: f32) -> f32 {
    x.max(0.0)
}

// ---- scalar arithmetic atoms (IEEE-754) -------------------------------------

pub fn add(a: f32, b: f32) -> f32 { a + b }
pub fn sub(a: f32, b: f32) -> f32 { a - b }
pub fn mul(a: f32, b: f32) -> f32 { a * b }
pub fn div(a: f32, b: f32) -> f32 { a / b }

// ---- sign-bit atoms: raw-bit manipulation (the "select moves raw bits" rule) -

/// `neg` — flip the sign bit (so `neg(NaN)` keeps the payload, flips the sign;
/// `neg(+0.0) = -0.0`).
pub fn neg(x: f32) -> f32 {
    f32::from_bits(x.to_bits() ^ 0x8000_0000)
}

/// `abs` — clear the sign bit (`abs(-0.0) = +0.0`; `abs(NaN)` keeps the payload).
pub fn abs(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7FFF_FFFF)
}

/// `copysign(x, y)` — magnitude of `x` with the sign bit of `y`.
pub fn copysign(x: f32, y: f32) -> f32 {
    f32::from_bits((x.to_bits() & 0x7FFF_FFFF) | (y.to_bits() & 0x8000_0000))
}

// ---- sign / step (§6.15 decompositions) -------------------------------------

/// `sign` — §6.15 `select(cmp_gt(x,0),1,select(cmp_lt(x,0),-1,0))`; `sign(NaN)=0`,
/// `sign(±0)=0`.
pub fn sign(x: f32) -> f32 {
    if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}

/// `step` — §6.15 `select(cmp_gt(x,0),1,0)`; `step(NaN)=0`, `step(0)=0`.
pub fn step(x: f32) -> f32 {
    if x > 0.0 { 1.0 } else { 0.0 }
}

// ---- rounding atoms ----------------------------------------------------------

pub fn floor(x: f32) -> f32 { x.floor() }
pub fn ceil(x: f32) -> f32 { x.ceil() }
pub fn trunc(x: f32) -> f32 { x.trunc() }

/// `round_even` — round-half-to-even (banker's rounding): `0.5 -> 0`, `1.5 -> 2`,
/// `2.5 -> 2` — distinct from round-half-away-from-zero.
pub fn round_even(x: f32) -> f32 {
    x.round_ties_even()
}

/// `rem_floor` — floored remainder, taking the sign of the **divisor** (§6.15-0003).
/// `a - b * floor(a / b)`. Distinct from `rem_trunc`; MUST NOT be substituted.
pub fn rem_floor(a: f32, b: f32) -> f32 {
    sub(a, mul(b, floor(div(a, b))))
}

/// `rem_trunc` — truncated remainder / `fmod`, taking the sign of the **dividend**
/// (§6.15-0003). `a - b * trunc(a / b)`. Distinct from `rem_floor`.
pub fn rem_trunc(a: f32, b: f32) -> f32 {
    sub(a, mul(b, trunc(div(a, b))))
}

/// `hypot(a, b)` (§6.13-0007). Yields `+∞` whenever **either** operand is infinite,
/// even if the other is NaN (the IEEE-754 hypot infinity rule), overriding the
/// naive `sqrt(a² + b²)` — which yields NaN for `(±∞, NaN)`. For finite inputs a
/// NaN operand propagates NaN.
pub fn hypot(a: f32, b: f32) -> f32 {
    if a.is_infinite() || b.is_infinite() {
        return f32::INFINITY;
    }
    (a * a + b * b).sqrt()
}

// ---- oracle boundary rounding (Conform §6.5-0006) ---------------------------

/// Narrow an f64 differential value to an f32 compute dtype and back
/// (KISS-Conform §6.5-0006). A discontinuous op (`cmp_*`, a `select` condition,
/// `sign`, `step`) resolves its boundary in the op's compute dtype, so the oracle
/// MUST round each operand to that dtype BEFORE the comparison — a single
/// round-to-nearest, mirroring the kernel's own store/compute rounding. This is
/// the reference oracle's `round_to_compute` discipline (Baracuda
/// `oracle.rs:507`): deciding on the un-narrowed f64 value flips spuriously when
/// two operands are distinct in wide precision but equal after rounding to f32.
pub fn round_to_compute_f32(x: f64) -> f64 {
    x as f32 as f64
}

// ---- ordered comparisons (IEEE; a NaN operand => all false except `cmp_ne`) ---

pub fn cmp_eq(a: f32, b: f32) -> bool { a == b }
pub fn cmp_ne(a: f32, b: f32) -> bool { a != b }
pub fn cmp_lt(a: f32, b: f32) -> bool { a < b }
pub fn cmp_le(a: f32, b: f32) -> bool { a <= b }
pub fn cmp_gt(a: f32, b: f32) -> bool { a > b }
pub fn cmp_ge(a: f32, b: f32) -> bool { a >= b }

/// `isnan(x)` is exactly `cmp_ne(x, x)` (the Ops primitive-floor identity).
pub fn isnan(x: f32) -> bool {
    cmp_ne(x, x)
}

// ============================================================================
// select condition semantics (§6.5-0003) and the complex/pow/refine oracles
// (§6.18-0004, §6.18-0006, §6.13-0005, §6.13-0003). Appended to semantics.rs.
// ============================================================================

/// `select(cond, a, b)` — §6.5-0001: yields `a` when `cond != 0` and `b` when
/// `cond == 0`, moving the chosen arm as a raw-bit copy (§6.5-0002). The `cond`
/// test is the IEEE compare `cond != 0` (§6.5-0003): `-0.0` compares **equal** to
/// zero (false, takes `b`) and any NaN is non-zero (true, takes `a`). A bit-nonzero
/// test (`cond.to_bits() != 0`) would wrongly treat `-0.0` (bits 0x8000_0000) as
/// true, and a `cond > 0` test would wrongly treat NaN as false.
pub fn select(cond: f32, a: f32, b: f32) -> f32 {
    if cond != 0.0 { a } else { b }
}

/// An interleaved (re, im) `c32` value: `[real, imaginary]` (§6.16 `c32` layout).
pub type C32 = [f32; 2];

/// `cadd` — componentwise `add` per lane (§6.18-0004).
pub fn cadd(z: C32, w: C32) -> C32 { [add(z[0], w[0]), add(z[1], w[1])] }

/// `csub` — componentwise `sub` per lane (§6.18-0004).
pub fn csub(z: C32, w: C32) -> C32 { [sub(z[0], w[0]), sub(z[1], w[1])] }

/// `cneg` — negate (flip the sign bit of) **both** lanes (§6.18-0004). `neg`
/// flips the sign bit, so `+0.0 -> -0.0` on each lane (not `0 - x`).
pub fn cneg(z: C32) -> C32 { [neg(z[0]), neg(z[1])] }

/// `cconj` — raw-copy the real lane, flip the **sign bit** of the imaginary lane
/// (§6.18-0004): `+0.0 -> -0.0`. This uses `neg` (a bit flip), NOT `sub(0.0, b)`
/// — the latter yields `+0.0` for `b = +0.0` under round-to-nearest.
pub fn cconj(z: C32) -> C32 { [z[0], neg(z[1])] }

fn is_zero_c(z: C32) -> bool { z[0] == 0.0 && z[1] == 0.0 }
fn is_finite_c(z: C32) -> bool { z[0].is_finite() && z[1].is_finite() }
fn has_inf_lane(z: C32) -> bool { z[0].is_infinite() || z[1].is_infinite() }

/// `cdiv` — `((ac + bd) + (bc − ad)i) / (c² + d²)` for finite operands with a
/// nonzero denominator, plus the ISO C99/C11 **Annex G** recovery (§6.18-0006).
/// The zero-denominator and complex-infinity-denominator cases MUST NOT be routed
/// through the real `div` atom (real `div`/0 is target-UB, §6.4-0002); they are
/// produced here by the Annex-G recovery with pinned component signs:
///   * nonzero finite `(a,b)` over `(±0,±0)` -> `copysign(∞,a) + i·copysign(∞,b)`
///     (Annex G G.5.1), NOT `0/0 = NaN`;
///   * complex-infinity denominator, finite numerator -> a complex **zero**.
pub fn cdiv(z: C32, w: C32) -> C32 {
    let (a, b) = (z[0], z[1]);
    let (c, d) = (w[0], w[1]);
    // Annex G G.5.1: nonzero finite numerator over a complex ZERO denominator.
    if is_zero_c(w) && is_finite_c(z) && !is_zero_c(z) {
        return [copysign(f32::INFINITY, a), copysign(f32::INFINITY, b)];
    }
    // Annex G: complex-infinity denominator, finite numerator -> complex zero.
    if has_inf_lane(w) && is_finite_c(z) {
        return [copysign(0.0, a), copysign(0.0, b)];
    }
    // Finite nonzero denominator (also inf-numerator / finite-denominator, which
    // has no div-by-zero): the real decomposition of the §6.18 table.
    let t = add(mul(c, c), mul(d, d));
    [
        div(add(mul(a, c), mul(b, d)), t),
        div(sub(mul(b, c), mul(a, d)), t),
    ]
}

/// The WRONG `cdiv` the clause forbids: it always routes through the real `div`
/// atom, so a complex zero denominator gives `0/0 = NaN` (and an infinite
/// denominator gives `inf/inf = NaN`) where Annex G pins a complex infinity /
/// complex zero. Provided so the divergence is a test, not a footnote.
pub fn cdiv_via_real_div(z: C32, w: C32) -> C32 {
    let (a, b) = (z[0], z[1]);
    let (c, d) = (w[0], w[1]);
    let t = add(mul(c, c), mul(d, d));
    [
        div(add(mul(a, c), mul(b, d)), t),
        div(sub(mul(b, c), mul(a, d)), t),
    ]
}

fn is_integer(b: f32) -> bool { b.is_finite() && b == trunc(b) }
fn is_odd_integer(b: f32) -> bool { is_integer(b) && rem_trunc(abs(b), 2.0) == 1.0 }

/// `a^b` for `a > 0` — the reference `exp(b·log(a))` at an accuracy the refinement
/// clause (§6.13-0003) permits (computed in f64, rounded once).
fn f32_pow_pos(a: f32, b: f32) -> f32 {
    (a as f64).powf(b as f64) as f32
}

/// `pow` pinned over its full domain (§6.13-0005). Transcribes the domain table:
/// `pow(_,0)=1` (incl. `pow(0,0)=1`); for `a>0` the reference `exp(b·log(a))`;
/// for `±0` bases the IEEE signed-zero rule (sign flips only for an odd-integer
/// exponent); for `a<0`, NaN unless `b` is an exact integer, then `|a|^b` (even) or
/// `-(|a|^b)` (odd). A naive `exp(b·log(a))` (see [`pow_via_exp_log`]) gets the
/// negative-base and `0^0` cases wrong (NaN).
pub fn pow(a: f32, b: f32) -> f32 {
    // pow(_, 0) = 1 for every base, including 0^0 = 1 and (-2)^0 = 1.
    if b == 0.0 {
        return 1.0;
    }
    if a > 0.0 {
        return f32_pow_pos(a, b);
    }
    if a == 0.0 {
        // ±0 base: magnitude is +0 for b>0, +inf for b<0; sign flips only for a
        // negative-zero base raised to a positive/negative ODD-INTEGER exponent.
        let mag = if b > 0.0 { 0.0f32 } else { f32::INFINITY };
        if a.is_sign_negative() && is_odd_integer(b) {
            return copysign(mag, -1.0); // -0.0 or -inf
        }
        return mag; // +0.0 or +inf
    }
    // a < 0.
    if is_integer(b) {
        let m = f32_pow_pos(abs(a), b); // |a|^b
        return if is_odd_integer(b) { neg(m) } else { m };
    }
    f32::NAN // a<0 with a non-integer exponent
}

/// The WRONG `pow` that computes only the `a>0` reference `exp(b·log(a))` for every
/// input: `log(a)` is NaN for `a<0` (so `pow(-2,3)` is NaN, not `-8`) and
/// `0·log(0) = 0·(-inf) = NaN` (so `pow(0,0)` is NaN, not `1`).
pub fn pow_via_exp_log(a: f32, b: f32) -> f32 {
    mul(b, a.ln()).exp()
}

/// `tanh` — the literal §6.13 reference decomposition
/// `(exp(x)-exp(-x)) / (exp(x)+exp(-x))`. For large `|x|` `exp(x)` overflows to
/// `+inf`, so `inf/inf = NaN` — the exact failure §6.13-0003 requires a conforming
/// kernel to avoid where the true function is finite.
pub fn tanh_literal_ref(x: f32) -> f32 {
    let ex = x.exp();
    let enx = neg(x).exp();
    div(sub(ex, enx), add(ex, enx))
}

/// `tanh` — the overflow-safe **refined** form §6.13-0003 REQUIRES for the
/// exp-of-large-argument family: finite where the literal reference overflows
/// (`tanh(100)=1`, `tanh(-100)=-1`), and equal to the reference's pinned meaning
/// within the op's declared ULP elsewhere.
pub fn tanh_refined(x: f32) -> f32 {
    x.tanh()
}
