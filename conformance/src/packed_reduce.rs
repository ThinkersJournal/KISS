// SPDX-License-Identifier: MIT OR Apache-2.0
//! The tuple-state reductions of KISS-Ops §6.11-0026 to §6.11-0029 (CireSnave 2026-11-12, board 427 "2) d)"):
//! `welford` (variance), `online_softmax` and `argmax`, each returning ONE packed tensor with a trailing state
//! axis. This module is the reference of their state, identity, merge and finalise at an `f32` state dtype
//! (count and index lanes are `u32`, stored by `bitcast`), the packed output shape with its `MAX_RANK` decline,
//! and the generator of `corpus/packed_reduction_vectors.json`.
//!
//! The merge and finalise formulas are written in ordinary arithmetic, each operation rounded to `f32`, in
//! exactly the form the spec states, so a foreign reader reproduces a bit pattern from the prose.

/// The closed `MAX_RANK` of §6.19-0037.
pub const MAX_RANK: usize = 8;

/// The three tuple-state monoids (the `monoid` ordinals 5, 6, 7 of §6.19-0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Welford,
    OnlineSoftmax,
    ArgMax,
}

impl Kind {
    /// The extent of the trailing state axis: 3 for `welford`, 2 for the others.
    pub fn lanes(self) -> usize {
        match self {
            Kind::Welford => 3,
            Kind::OnlineSoftmax | Kind::ArgMax => 2,
        }
    }
    /// The `monoid` wire ordinal (§6.19-0014).
    pub fn ordinal(self) -> u8 {
        match self {
            Kind::Welford => 5,
            Kind::OnlineSoftmax => 6,
            Kind::ArgMax => 7,
        }
    }
}

/// Why a tuple-state reduction is refused (§6.11-0026).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decline {
    /// The packed output would have more than `MAX_RANK` axes (a rank-`MAX_RANK` operand with `keepdim = 1`
    /// needs `MAX_RANK + 1`).
    RankExceedsMax { rank: usize },
    /// `argmax` reduces exactly one axis.
    ArgmaxAxisCount { got: usize },
    /// A tuple-state monoid's `accumulator` field MUST be `0x00` (the state dtype is derived).
    AccumulatorNotCompute,
    /// The compute dtype has no tuple-state dtype (only floats do).
    StateDtype(String),
    /// A `welford` count would exceed the count lane.
    CountOverflow,
}

/// Whether a `prefix_scan` (§6.11-0003) accepts a `monoid` ordinal: only the four scalar folds (1 to 4), never a
/// tuple-state monoid (§6.11-0026).
pub fn scan_accepts_monoid(ordinal: u8) -> bool {
    (1..=4).contains(&ordinal)
}

/// The state dtype of a tuple-state reduction over `compute`: the compute dtype when it is `f32` or `f64`,
/// `f32` when it is a narrower float (§6.11-0026); `None` for a non-float compute dtype.
pub fn state_dtype(compute: &str) -> Option<&'static str> {
    match compute {
        "f64" => Some("f64"),
        "f32" | "f16" | "bf16" | "f8e4m3fn" | "f8e5m2" | "f4e2m1" | "f6e2m3" | "f6e3m2" => Some("f32"),
        _ => None,
    }
}

/// The packed output shape: the `keepdim = 1` result (reduced axes kept at extent 1), or with `keepdim = 0` that
/// result with the reduced axes removed, plus ONE trailing state axis (§6.11-0026). A rank above `MAX_RANK` is a
/// typed decline; `argmax` over a set that is not exactly one axis is one too.
pub fn packed_output_shape(in_shape: &[usize], reduced: &[usize], keepdim: bool, kind: Kind) -> Result<Vec<usize>, Decline> {
    if kind == Kind::ArgMax && reduced.len() != 1 {
        return Err(Decline::ArgmaxAxisCount { got: reduced.len() });
    }
    let mut out: Vec<usize> = Vec::new();
    for (i, &e) in in_shape.iter().enumerate() {
        if reduced.contains(&i) {
            if keepdim {
                out.push(1);
            }
        } else {
            out.push(e);
        }
    }
    out.push(kind.lanes());
    if out.len() > MAX_RANK {
        return Err(Decline::RankExceedsMax { rank: out.len() });
    }
    Ok(out)
}

// ---- welford: state (n, mean, M2) --------------------------------------------------------------------------

/// A `welford` state at `f32` (§6.11-0027): the count lane is a `u32`, the other two are `f32`.
#[derive(Debug, Clone, Copy)]
pub struct Welford {
    pub n: u32,
    pub mean: f32,
    pub m2: f32,
}

/// The identity (starting value): `(0, +0, +0)`.
pub const WELFORD_IDENTITY: Welford = Welford { n: 0, mean: 0.0, m2: 0.0 };

/// The state of one element `x`: `(1, x, +0)`.
pub fn welford_elem(x: f32) -> Welford {
    Welford { n: 1, mean: x, m2: 0.0 }
}

/// The merge (Chan et al.): an empty side returns the other; otherwise `n = nA + nB`, `delta = meanB - meanA`,
/// `mean = meanA + delta * (nB / n)`, `M2 = (M2A + M2B) + (delta * delta) * ((nA * nB) / n)`, with the counts
/// converted to `f32` (round to nearest even) and every operation rounded to `f32`.
pub fn welford_merge(a: Welford, b: Welford) -> Result<Welford, Decline> {
    if a.n == 0 {
        return Ok(b);
    }
    if b.n == 0 {
        return Ok(a);
    }
    let n = a.n.checked_add(b.n).ok_or(Decline::CountOverflow)?;
    let (na, nb, nn) = (a.n as f32, b.n as f32, n as f32);
    let delta = b.mean - a.mean;
    let mean = a.mean + delta * (nb / nn);
    let m2 = (a.m2 + b.m2) + (delta * delta) * ((na * nb) / nn);
    Ok(Welford { n, mean, m2 })
}

/// The reference fold: ascending element order, `state = merge(state, elem(x))` (§6.17-0009 order).
pub fn welford_fold(xs: &[f32]) -> Result<Welford, Decline> {
    xs.iter().try_fold(WELFORD_IDENTITY, |s, &x| welford_merge(s, welford_elem(x)))
}

/// The finalise: `M2 / n` (population) or `M2 / (n - 1)` (Bessel); an empty state is `0/0 = NaN`.
pub fn welford_variance(s: Welford, bessel: bool) -> f32 {
    let d = if bessel { s.n as f32 - 1.0 } else { s.n as f32 };
    s.m2 / d
}

/// The packed lanes in order `(n bits, mean bits, M2 bits)`.
pub fn welford_lanes(s: Welford) -> [u32; 3] {
    [s.n, s.mean.to_bits(), s.m2.to_bits()]
}

// ---- online softmax: state (m, l) --------------------------------------------------------------------------

/// An `online_softmax` state (§6.11-0028): the running maximum and the running sum of `exp(x - m)`.
#[derive(Debug, Clone, Copy)]
pub struct Soft {
    pub m: f32,
    pub l: f32,
}

/// The identity: `(-inf, +0)`.
pub const SOFT_IDENTITY: Soft = Soft { m: f32::NEG_INFINITY, l: 0.0 };

/// The NaN state: both lanes NaN (any NaN or `+inf` input).
pub const SOFT_NAN: Soft = Soft { m: f32::NAN, l: f32::NAN };

/// The state of one element: `(x, 1)`; `-inf` is the identity (it contributes weight `exp(-inf) = 0`); a NaN or
/// `+inf` element is the NaN state.
pub fn soft_elem(x: f32) -> Soft {
    if x.is_nan() || x == f32::INFINITY {
        SOFT_NAN
    } else if x == f32::NEG_INFINITY {
        SOFT_IDENTITY
    } else {
        Soft { m: x, l: 1.0 }
    }
}

/// The merge: a NaN lane on either side gives the NaN state; `m = max(mA, mB)`; `m = -inf` gives the identity;
/// otherwise `l = lA * exp(mA - m) + lB * exp(mB - m)`.
pub fn soft_merge(a: Soft, b: Soft) -> Soft {
    if a.m.is_nan() || a.l.is_nan() || b.m.is_nan() || b.l.is_nan() {
        return SOFT_NAN;
    }
    let m = if a.m >= b.m { a.m } else { b.m };
    if m == f32::NEG_INFINITY {
        return SOFT_IDENTITY;
    }
    Soft { m, l: a.l * (a.m - m).exp() + b.l * (b.m - m).exp() }
}

pub fn soft_fold(xs: &[f32]) -> Soft {
    xs.iter().fold(SOFT_IDENTITY, |s, &x| soft_merge(s, soft_elem(x)))
}

/// The finalise: `logsumexp = m + log(l)`; the probability of an element `x` is `exp(x - m) / l`.
pub fn soft_logsumexp(s: Soft) -> f32 {
    s.m + s.l.ln()
}

pub fn soft_lanes(s: Soft) -> [u32; 2] {
    [s.m.to_bits(), s.l.to_bits()]
}

// ---- argmax: state (value, index) --------------------------------------------------------------------------

/// An `argmax` state (§6.11-0029): the extremal value and its position along the reduced axis (a `u32` lane).
#[derive(Debug, Clone, Copy)]
pub struct Arg {
    pub v: f32,
    pub i: u32,
}

/// The identity: `(-inf, 0xFFFFFFFF)`; the maximal index loses every tie, so any element replaces it.
pub const ARG_IDENTITY: Arg = Arg { v: f32::NEG_INFINITY, i: u32::MAX };

pub fn arg_elem(x: f32, position: u32) -> Arg {
    Arg { v: x, i: position }
}

/// The §6.11-0007 key order: NaN is the greatest value; `+0` and `-0` are equal; any two NaNs are equal.
fn key_gt(a: f32, b: f32) -> bool {
    match (a.is_nan(), b.is_nan()) {
        (true, false) => true,
        (_, true) => false,
        _ => a > b,
    }
}

fn key_eq(a: f32, b: f32) -> bool {
    (a.is_nan() && b.is_nan()) || a == b
}

/// The merge: `B` replaces `A` iff `B`'s key is greater, or the keys are equal and `B`'s index is lower
/// (`tie_rule = lowest`, fixed). This is the maximum of a total order, so it is exact and associative.
pub fn arg_merge(a: Arg, b: Arg) -> Arg {
    if key_gt(b.v, a.v) || (key_eq(a.v, b.v) && b.i < a.i) {
        b
    } else {
        a
    }
}

pub fn arg_fold(xs: &[f32]) -> Arg {
    xs.iter().enumerate().fold(ARG_IDENTITY, |s, (p, &x)| arg_merge(s, arg_elem(x, p as u32)))
}

pub fn arg_lanes(s: Arg) -> [u32; 2] {
    [s.v.to_bits(), s.i]
}

// ---- the vectors -------------------------------------------------------------------------------------------

fn hexw(bits: &[u32]) -> String {
    bits.iter().map(|b| format!("{b:08X}")).collect::<Vec<_>>().join(" ")
}

fn input_json(xs: &[f32]) -> String {
    xs.iter().map(|x| format!("\"{:08X}\"", x.to_bits())).collect::<Vec<_>>().join(", ")
}

/// `corpus/packed_reduction_vectors.json`: the packed lanes (as `u32` hex, count and index lanes as plain
/// integers, float lanes as `f32` bit patterns) of every reduction below, folded in ascending order, plus the
/// shape cases. `class` says how a foreign reader compares: `exact-byte`, or `tolerance` for a float fold whose
/// reassociation is bounded by the declared tier (§6.17-0008), never against a wide truth across widths.
pub fn emit_json() -> String {
    let nan = f32::NAN;
    let inf = f32::INFINITY;
    let mut cases: Vec<String> = Vec::new();
    let mut add = |name: &str, kind: &str, xs: &[f32], lanes: &[u32], class: &str, note: &str| {
        cases.push(format!(
            "    {{\"name\": \"{name}\", \"kind\": \"{kind}\", \"input\": [{}], \"lanes\": \"{}\", \"class\": \"{class}\", \"note\": \"{note}\"}}",
            input_json(xs),
            hexw(lanes)
        ));
    };
    let wf = |xs: &[f32]| welford_lanes(welford_fold(xs).expect("count"));
    add("welford_empty_is_identity", "welford", &[], &wf(&[]), "exact-byte", "an empty reduction yields the identity (0, +0, +0)");
    add("welford_one_element", "welford", &[3.5], &wf(&[3.5]), "exact-byte", "(1, x, +0)");
    add("welford_constant", "welford", &[5.0, 5.0, 5.0], &wf(&[5.0, 5.0, 5.0]), "exact-byte", "M2 stays +0");
    add("welford_one_to_four", "welford", &[1.0, 2.0, 3.0, 4.0], &wf(&[1.0, 2.0, 3.0, 4.0]), "tolerance", "mean 2.5, M2 5; population variance 1.25, Bessel 5/3");
    add("welford_nan_input", "welford", &[1.0, nan, 3.0], &wf(&[1.0, nan, 3.0]), "exact-byte", "n counts the NaN; mean and M2 are NaN");
    let sf = |xs: &[f32]| soft_lanes(soft_fold(xs));
    add("softmax_empty_is_identity", "online_softmax", &[], &sf(&[]), "exact-byte", "(-inf, +0)");
    add("softmax_all_neg_inf_is_identity", "online_softmax", &[-inf, -inf], &sf(&[-inf, -inf]), "exact-byte", "both sides -inf stay the identity, never NaN");
    add("softmax_single", "online_softmax", &[3.0], &sf(&[3.0]), "exact-byte", "(x, 1)");
    add("softmax_equal_pair", "online_softmax", &[0.0, 0.0], &sf(&[0.0, 0.0]), "exact-byte", "exp(0) = 1 exactly: (0, 2)");
    add("softmax_one_two", "online_softmax", &[1.0, 2.0], &sf(&[1.0, 2.0]), "tolerance", "m = 2, l = 1 + exp(-1)");
    add("softmax_with_neg_inf", "online_softmax", &[-inf, 2.0], &sf(&[-inf, 2.0]), "exact-byte", "a -inf element adds weight 0: (2, 1)");
    add("softmax_nan_input", "online_softmax", &[1.0, nan], &sf(&[1.0, nan]), "exact-byte", "the NaN state");
    add("softmax_pos_inf_input", "online_softmax", &[1.0, inf], &sf(&[1.0, inf]), "exact-byte", "the NaN state");
    let ag = |xs: &[f32]| arg_lanes(arg_fold(xs));
    add("argmax_empty_is_identity", "argmax", &[], &ag(&[]), "exact-byte", "(-inf, 0xFFFFFFFF)");
    add("argmax_ordinary", "argmax", &[1.0, 3.0, 2.0], &ag(&[1.0, 3.0, 2.0]), "exact-byte", "(3, 1)");
    add("argmax_tie_lowest_index", "argmax", &[2.0, 2.0], &ag(&[2.0, 2.0]), "exact-byte", "equal values: the lower index");
    add("argmax_lowest_index_nan", "argmax", &[nan, 1.0, nan], &ag(&[nan, 1.0, nan]), "exact-byte", "NaN is the greatest value; the lowest-index NaN wins (§6.11-0007)");
    add("argmax_signed_zeros_equal", "argmax", &[-0.0, 0.0], &ag(&[-0.0, 0.0]), "exact-byte", "-0 and +0 are equal; the lower index (and its bits) win");
    add("argmax_all_neg_inf", "argmax", &[-inf, -inf], &ag(&[-inf, -inf]), "exact-byte", "equal to the identity's value: index 0 beats 0xFFFFFFFF");
    let shape = |s: &[usize], r: &[usize], k: bool, kind: Kind| match packed_output_shape(s, r, k, kind) {
        Ok(v) => format!("{v:?}"),
        Err(e) => format!("decline:{e:?}"),
    };
    let mut shapes: Vec<String> = Vec::new();
    let mut add_shape = |name: &str, kind: &str, s: &[usize], r: &[usize], k: bool, out: String| {
        shapes.push(format!(
            "    {{\"name\": \"{name}\", \"kind\": \"{kind}\", \"in_shape\": {s:?}, \"reduced_axes\": {r:?}, \"keepdim\": {k}, \"out\": \"{out}\"}}"
        ));
    };
    add_shape("welford_keepdim", "welford", &[4, 8], &[1], true, shape(&[4, 8], &[1], true, Kind::Welford));
    add_shape("welford_no_keepdim", "welford", &[4, 8], &[1], false, shape(&[4, 8], &[1], false, Kind::Welford));
    add_shape("softmax_all_axes_no_keepdim", "online_softmax", &[4, 8], &[0, 1], false, shape(&[4, 8], &[0, 1], false, Kind::OnlineSoftmax));
    add_shape("argmax_keepdim", "argmax", &[4, 8], &[1], true, shape(&[4, 8], &[1], true, Kind::ArgMax));
    add_shape("argmax_two_axes_declines", "argmax", &[4, 8], &[0, 1], true, shape(&[4, 8], &[0, 1], true, Kind::ArgMax));
    let r8 = [2usize; 8];
    add_shape("rank8_keepdim_needs_rank9", "welford", &r8, &[7], true, shape(&r8, &[7], true, Kind::Welford));
    add_shape("rank8_no_keepdim_fits", "welford", &r8, &[7], false, shape(&r8, &[7], false, Kind::Welford));
    add_shape("rank7_keepdim_fits", "online_softmax", &r8[..7], &[6], true, shape(&r8[..7], &[6], true, Kind::OnlineSoftmax));
    let mut s = String::new();
    s.push_str("{\n  \"schema\": \"kiss-packed-reduction-vectors-v1.json\",\n");
    s.push_str("  \"generated_from\": \"conformance/src/packed_reduce.rs::emit_json (the reference)\",\n");
    s.push_str("  \"clauses\": \"KISS-OPS-6.11-0026 (packed output, state dtype, MAX_RANK decline), -0027 (welford), -0028 (online_softmax), -0029 (argmax)\",\n");
    s.push_str("  \"encoding\": \"f32 state: the lanes are 8-hex-digit words in trailing-axis order; a welford count lane and an argmax index lane are u32 values written as hex words (read through bitcast), every other lane is an f32 bit pattern. Folds are ascending: state = merge(state, element), index = position along the reduced axis.\",\n");
    s.push_str("  \"cases\": [\n");
    s.push_str(&cases.join(",\n"));
    s.push_str("\n  ],\n  \"shape_cases\": [\n");
    s.push_str(&shapes.join(",\n"));
    s.push_str("\n  ]\n}\n");
    s
}
