// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bind-time evaluation of a Dispatch expression (KISS-Contract §6.12).
//!
//! A **bound environment** is a [`LaunchScalars`]; **bind time** is the moment a consumer evaluates
//! an expression against it. [`bind_expr`] is the reference for the three bind-time declines
//! (§6.12-0003..-0005) and for the order in which they are found (§6.12-0006). It evaluates in
//! [`EvalMode::Structural`] (no per-node sign gate), then applies the range gate **last**, so a
//! negative intermediate is legal and only the final value is gated.
//!
//! An expression that is malformed is a **build-time** fault (§6.12-0002), carried as
//! [`BindDecline::BuildTime`]; a reader MUST NOT defer it to bind time. `UnknownSymbol` is mapped to
//! `unbound_symbol` because the evaluator cannot tell "a vocabulary symbol the environment does not
//! assign" from "an identifier outside the vocabulary"; either way no zero or default is substituted.

// `eval` is this crate's own Dispatch-expression parser/evaluator (a bounded arithmetic grammar over
// launch-scalar symbols), not dynamic code execution.
use crate::dispatch_expr::{eval, Decline, EvalMode, LaunchScalars};
use crate::grammar::dtype_storage_bits;
use crate::layout::{IntTuple, Layout, Slot};

/// A typed bind-time decline (§6.12). Never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindDecline {
    /// `unbound_symbol` (§6.12-0003): the expression names a symbol the environment does not assign.
    UnboundSymbol(String),
    /// `extent_out_of_range` (§6.12-0004): the final value is negative or outside a declared range.
    ExtentOutOfRange(i64),
    /// `affine_overflow` (§6.12-0005): a `+`, `-` or `*` left the signed 64-bit range.
    AffineOverflow,
    /// `slot_overlap` (§6.5-0019): two bundle slots' byte hulls intersect; carries both names.
    SlotOverlap(String, String),
    /// A build-time fault (malformed expression, bad subscript): not a bind-time decline.
    BuildTime(Decline),
}

impl BindDecline {
    /// The typed decline code the spec names, or `build_time` for a build-time fault.
    pub fn code(&self) -> &'static str {
        match self {
            BindDecline::UnboundSymbol(_) => "unbound_symbol",
            BindDecline::ExtentOutOfRange(_) => "extent_out_of_range",
            BindDecline::AffineOverflow => "affine_overflow",
            BindDecline::SlotOverlap(..) => "slot_overlap",
            BindDecline::BuildTime(_) => "build_time",
        }
    }
}

/// Evaluate `expr` against the bound environment `env` (§6.12). `range` is the inclusive range the
/// field declares, if any; the value must also be non-negative (§6.12-0004).
pub fn bind_expr(
    expr: &str,
    env: &LaunchScalars,
    range: Option<(i64, i64)>,
) -> Result<i64, BindDecline> {
    let v = eval(expr, env, EvalMode::Structural).map_err(|d| match d {
        Decline::UnknownSymbol(s) => BindDecline::UnboundSymbol(s),
        Decline::Overflow => BindDecline::AffineOverflow,
        other => BindDecline::BuildTime(other),
    })?;
    let in_range = range.map_or(true, |(lo, hi)| (lo..=hi).contains(&v));
    if v < 0 || !in_range {
        return Err(BindDecline::ExtentOutOfRange(v));
    }
    Ok(v)
}

fn leaves(t: &IntTuple, out: &mut Vec<u64>) {
    match t {
        IntTuple::Leaf(v) => out.push(*v),
        IntTuple::Tup(v) => v.iter().for_each(|x| leaves(x, out)),
    }
}

/// Element-offset hull `[lo, hi)` of `extents` under `strides` (strides may be negative); `None`
/// when any extent is 0 (no element, so no bytes).
fn hull(extents: &[i128], strides: &[i128]) -> Option<(i128, i128)> {
    if extents.contains(&0) {
        return None;
    }
    let (mut lo, mut hi) = (0i128, 0i128);
    for (e, s) in extents.iter().zip(strides) {
        let span = (e - 1) * s;
        if span < 0 { lo += span } else { hi += span }
    }
    Some((lo, hi + 1))
}

fn eval_i128(expr: &str, env: &LaunchScalars) -> Result<i128, BindDecline> {
    let v = eval(expr, env, EvalMode::Structural).map_err(|d| match d {
        Decline::UnknownSymbol(s) => BindDecline::UnboundSymbol(s),
        Decline::Overflow => BindDecline::AffineOverflow,
        other => BindDecline::BuildTime(other),
    })?;
    Ok(i128::from(v))
}

/// The byte hull `[start, end)` of one slot under `env`, or `None` for an empty slot.
fn slot_hull(slot: &Slot, env: &LaunchScalars) -> Result<Option<(i128, i128)>, BindDecline> {
    let off = i128::from(bind_expr(&slot.offset, env, None)?);
    let ext = slot.shape.iter().map(|e| bind_expr(e, env, None).map(i128::from)).collect::<Result<Vec<_>, _>>()?;
    let packed = |ext: &[i128]| -> Vec<i128> {
        let mut st = vec![1i128; ext.len()];
        for i in (0..ext.len().saturating_sub(1)).rev() {
            st[i] = st[i + 1] * ext[i + 1];
        }
        st
    };
    let (st, window) = match &slot.layout {
        Layout::DenseOrder(order) => {
            let by_pos = packed(&order.iter().map(|&a| ext[a as usize]).collect::<Vec<_>>());
            let mut st = vec![0i128; ext.len()];
            for (p, &a) in order.iter().enumerate() {
                st[a as usize] = by_pos[p];
            }
            (st, 1i128)
        }
        Layout::Strided(exprs) => (exprs.iter().map(|e| eval_i128(e, env)).collect::<Result<Vec<_>, _>>()?, 1),
        Layout::Hier(h) => {
            let (mut sh, mut sd) = (Vec::new(), Vec::new());
            leaves(&h.shape, &mut sh);
            leaves(&h.stride, &mut sd);
            let w = h.swizzle.map_or(1i128, |(b, m, _)| 1i128 << (b + m));
            let (e, s): (Vec<i128>, Vec<i128>) = (sh.iter().map(|&v| i128::from(v)).collect(), sd.iter().map(|&v| i128::from(v)).collect());
            return Ok(hull(&e, &s).map(|(lo, hi)| bytes(&slot.dtype, off, lo, (hi + w - 1) / w * w)));
        }
        // dense, same_as(i) with a free rank: packed row-major is the hull a reader can compute.
        _ => (packed(&ext), 1),
    };
    Ok(hull(&ext, &st).map(|(lo, hi)| bytes(&slot.dtype, off, lo, (hi + window - 1) / window * window)))
}

fn bytes(dtype: &str, off: i128, lo: i128, hi: i128) -> (i128, i128) {
    let bits = i128::from(dtype_storage_bits(dtype).unwrap_or(8));
    (off + (lo * bits).div_euclid(8), off + (hi * bits + 7).div_euclid(8))
}

/// Bind-time check of a bundle's slots (§6.5-0019): decline `slot_overlap` when two slots' byte
/// hulls intersect. A hull is `[offset + lowest element, offset + highest element + 1)` in bytes
/// (sub-byte dtypes rounded outward); an empty slot (a zero extent) has no bytes and overlaps nothing.
/// Hulls are compared pairwise in declaration order, so the first overlapping pair names the decline.
pub fn check_slot_overlap(slots: &[Slot], env: &LaunchScalars) -> Result<(), BindDecline> {
    let hulls = slots.iter().map(|s| slot_hull(s, env)).collect::<Result<Vec<_>, _>>()?;
    for (i, a) in hulls.iter().enumerate() {
        for (j, b) in hulls.iter().enumerate().skip(i + 1) {
            if let (Some((a0, a1)), Some((b0, b1))) = (a, b) {
                if a0 < b1 && b0 < a1 {
                    return Err(BindDecline::SlotOverlap(slots[i].name.clone(), slots[j].name.clone()));
                }
            }
        }
    }
    Ok(())
}

/// One slot-overlap vector: a bundle layout text, the launch scalar `n` (rank 2, extents `[3,4]`,
/// strides `[4,1]`), the expected code (`ok` or `slot_overlap`), and whether it is the **pinned v1**
/// behaviour of a valid layout the byte-hull rule refuses (§6.5-0019).
pub const SLOT_OVERLAP_VECTORS: [(&str, &str, i64, &str, bool); 6] = [
    ("touching_hulls_ok", "bundle(a:f32[4]=dense@0,b:f32[4]=dense@16)", 12, "ok", false),
    ("overlapping_hulls", "bundle(a:f32[4]=dense@0,b:f32[4]=dense@15)", 12, "slot_overlap", false),
    ("verdict_follows_environment_ok", "bundle(a:f32[n]=dense@0,b:f32[4]=dense@32)", 8, "ok", false),
    ("verdict_follows_environment_overlap", "bundle(a:f32[n]=dense@0,b:f32[4]=dense@32)", 9, "slot_overlap", false),
    ("empty_slot_overlaps_nothing", "bundle(a:f32[0]=dense@8,b:f32[4]=dense@0)", 12, "ok", false),
    // a's elements sit at bytes 0..4 and 8..12; b's 4 bytes at 4..8 fall in the gap: element-disjoint
    // and valid, but the byte hulls [0,12) and [4,8) intersect, so v1 declines (conservative).
    ("interleaved_valid_refused_v1", "bundle(a:f32[1,2]=strided(2,2)@0,b:f32[1]=dense@4)", 12, "slot_overlap", true),
];

/// The scalars every [`SLOT_OVERLAP_VECTORS`] case binds against.
pub fn slot_overlap_env(n: i64) -> LaunchScalars {
    LaunchScalars {
        rank: 2,
        n,
        extents: vec![vec![3, 4]],
        strides: vec![vec![4, 1]],
        idx_extents: vec![],
        off: vec![0],
        ws_bytes: None,
        param: vec![],
    }
}

/// The committed `conformance/corpus/slot_overlap_vectors.json`, byte for byte.
pub fn emit_slot_overlap_vectors_json() -> String {
    let mut s = String::from("{
  \"schema\": \"kiss-slot-overlap-vectors-v1.json\",
");
    s.push_str("  \"generated_from\": \"conformance/src/bind.rs::emit_slot_overlap_vectors_json\",
");
    s.push_str("  \"clause\": \"KISS-CONTRACT-6.5-0019\",
");
    s.push_str("  \"environment\": \"rank 2, n as stated, extents0 [3,4], strides0 [4,1], no workspace\",
");
    s.push_str("  \"hull_rule\": \"a slot's byte hull is [offset + lowest element, offset + highest element + 1); two slots overlap when their hulls intersect\",
");
    s.push_str("  \"cases\": [
");
    for (i, (name, text, n, expect, pinned)) in SLOT_OVERLAP_VECTORS.iter().enumerate() {
        let comma = if i + 1 < SLOT_OVERLAP_VECTORS.len() { "," } else { "" };
        s.push_str(&format!(
            "    {{\"name\": \"{name}\", \"bundle\": \"{text}\", \"n\": {n}, \"expect\": \"{expect}\", \"pinned_v1_conservative\": {pinned}}}{comma}
"
        ));
    }
    s.push_str("  ]
}
");
    s
}
