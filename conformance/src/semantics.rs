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
