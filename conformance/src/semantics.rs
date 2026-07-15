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
