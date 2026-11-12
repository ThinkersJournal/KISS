// SPDX-License-Identifier: MIT OR Apache-2.0
//! The step between a contract's declared tolerance and a comparator's threshold
//! (KISS-CONFORM §6.8-0004a, CireSnave decision 311, Q2 A).
//!
//! A contract declares its tolerance as an accuracy tier (KISS-CONTRACT §6.8-0002) or, for an
//! accumulator-keyed reduction, as a ULP-of-A band (§6.8-0012). [`comparator_threshold`] reads the
//! declared field and returns the threshold the comparator applies; it invents no default, and a
//! contract that declares nothing yields [`NoDeclaredTolerance`] rather than a guess.

use crate::contract::{DeclaredAccuracyTier, Real};
use crate::structural::compare_order_invariant;
use crate::ulp_distance_f32;

/// The `k(S,A) · N · eps_A · |max partial sum|` band of KISS-CONTRACT §6.8-0012, minus the two
/// per-invocation inputs (`N` and the maximum partial sum): the per-cell calibrated constant and
/// the accumulator dtype's unit roundoff.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccumulatorBand {
    pub k: f64,
    pub eps_a: f64,
}

/// What a contract's Guarantees declare as the tolerance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeclaredTolerance {
    Tier(DeclaredAccuracyTier),
    Band(AccumulatorBand),
}

/// The threshold a comparator applies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Threshold {
    /// The ULP/tolerance comparator, admitting a result within every bound the tier declares.
    Tier(DeclaredAccuracyTier),
    /// The order-invariant comparator's **absolute** band (§6.8-0012: never result-relative).
    AbsoluteBand(f64),
}

/// The contract declared no usable tolerance, so no threshold may be derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoDeclaredTolerance;

/// The value of a declared real bound.
fn real_value(r: Real) -> f64 {
    match r {
        Real::F32(bits) => f64::from(f32::from_bits(bits)),
        Real::F64(bits) => f64::from_bits(bits),
    }
}

/// KISS-CONFORM §6.8-0004a: map a declared tolerance to the comparator threshold. `n` is the
/// runtime reduction extent and `max_partial_sum` the invocation's largest partial sum; both are
/// consulted only by a [`DeclaredTolerance::Band`].
pub fn comparator_threshold(
    declared: Option<DeclaredTolerance>,
    n: u64,
    max_partial_sum: f64,
) -> Result<Threshold, NoDeclaredTolerance> {
    match declared {
        Some(DeclaredTolerance::Tier(t)) if t.is_bounded() => Ok(Threshold::Tier(t)),
        Some(DeclaredTolerance::Band(b)) => {
            Ok(Threshold::AbsoluteBand(b.k * n as f64 * b.eps_a * max_partial_sum.abs()))
        }
        _ => Err(NoDeclaredTolerance),
    }
}

/// Whether `actual` is admitted against `expected` under `threshold`.
pub fn threshold_admits_f32(threshold: Threshold, actual: f32, expected: f32) -> bool {
    match threshold {
        Threshold::AbsoluteBand(abs) => {
            compare_order_invariant(&[actual], &[expected], abs as f32, 0.0).is_ok()
        }
        Threshold::Tier(t) => {
            let diff = (f64::from(actual) - f64::from(expected)).abs();
            t.max_ulp.map_or(true, |u| ulp_distance_f32(actual, expected) <= u64::from(u))
                && t.max_relative
                    .map_or(true, |r| diff <= real_value(r) * f64::from(expected).abs())
                && t.max_absolute.map_or(true, |a| diff <= real_value(a))
        }
    }
}

/// The per-destination-position inputs of the floating-point `atomic-add` tolerance-cell
/// (KISS-OPS §6.11-0024): the §6.17-0009 reference value, the contribution count `N`, and the
/// maximum absolute running partial (the destination's own value included).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicAddCell {
    /// `dest`, then each update in ascending source index, every accumulate rounded to `f32`.
    pub reference: f32,
    /// The number of updates that scatter to this position.
    pub n: u64,
    /// `max |p_i|` over `p_0 = dest, p_i = p_(i-1) + u_i`.
    pub max_partial_sum: f64,
}

/// Evaluate the reference cell for an `f32` destination position (compute dtype = accumulator = `f32`).
pub fn atomic_add_cell_f32(dest: f32, updates: &[f32]) -> AtomicAddCell {
    let mut p = dest;
    let mut max = f64::from(p).abs();
    for &u in updates {
        p += u;
        max = max.max(f64::from(p).abs());
    }
    AtomicAddCell { reference: p, n: updates.len() as u64, max_partial_sum: max }
}

/// Whether `candidate` (the result of any accumulation order) is admitted for this position under
/// the §6.8-0012 band `k * N * eps_A * |max partial sum|`; a zero band demands exact equality.
pub fn atomic_add_admits_f32(band: AccumulatorBand, dest: f32, updates: &[f32], candidate: f32) -> bool {
    let cell = atomic_add_cell_f32(dest, updates);
    let abs = band.k * cell.n as f64 * band.eps_a * cell.max_partial_sum;
    if abs == 0.0 {
        return candidate.to_bits() == cell.reference.to_bits() || candidate == cell.reference;
    }
    (f64::from(candidate) - f64::from(cell.reference)).abs() <= abs
}

/// One atomic-add band vector: name, `dest`, `updates`, a candidate result (the value some other
/// accumulation order produced), and what a reader says about it.
pub const ATOMIC_ADD_BAND_VECTORS: [(&str, f32, &[f32], f32); 6] = [
    ("exact_sum_same_in_every_order", 0.0, &[1.0, 2.0, 3.0, 4.0], 10.0),
    // ascending: 1e8 + 1 -> 1e8 (f32), -1e8 -> 0, +1 -> 1 = reference; another order gives 2; band 4*u*1e8.
    ("reassociation_inside_band", 0.0, &[1.0e8, 1.0, -1.0e8, 1.0], 2.0),
    ("reassociation_outside_band", 0.0, &[1.0e8, 1.0, -1.0e8, 1.0], 100.0),
    // dest is part of the sequence: reference stays 2^24, band 2 * 2^-24 * 2^24 = 2, candidate 2^24 + 2 is on the edge.
    ("dest_in_the_partials_edge_admitted", 16_777_216.0, &[1.0, 1.0], 16_777_218.0),
    ("dest_in_the_partials_past_edge_refused", 16_777_216.0, &[1.0, 1.0], 16_777_220.0),
    ("zero_partials_demand_exact_zero", 0.0, &[0.0, 0.0, 0.0], 1.0e-30),
];

/// The committed `conformance/corpus/atomic_add_band_vectors.json`, byte for byte. `k = 1`, `eps_A = 2^-24`.
pub fn emit_atomic_add_band_vectors_json() -> String {
    let band = AccumulatorBand { k: 1.0, eps_a: f64::from(f32::EPSILON) / 2.0 };
    let bits = |x: f32| format!("{:08X}", x.to_bits());
    let list = |v: &[f32]| v.iter().map(|x| format!("\"{}\"", bits(*x))).collect::<Vec<_>>().join(", ");
    let mut s = String::from("{
  \"schema\": \"kiss-atomic-add-band-vectors-v1.json\",
");
    s.push_str("  \"generated_from\": \"conformance/src/tolerance.rs::emit_atomic_add_band_vectors_json\",
");
    s.push_str("  \"clause\": \"KISS-OPS-6.11-0024 (band as KISS-CONTRACT-6.8-0012 defines it)\",
");
    s.push_str("  \"band\": \"k = 1, eps_A = 2^-24 (f32), per position: k * N * eps_A * |max partial sum|; values are f32 bit patterns in hex\",
");
    s.push_str("  \"cases\": [
");
    for (i, (name, dest, updates, cand)) in ATOMIC_ADD_BAND_VECTORS.iter().enumerate() {
        let cell = atomic_add_cell_f32(*dest, updates);
        let admitted = atomic_add_admits_f32(band, *dest, updates, *cand);
        let comma = if i + 1 < ATOMIC_ADD_BAND_VECTORS.len() { "," } else { "" };
        s.push_str(&format!(
            "    {{\"name\": \"{name}\", \"dest\": \"{}\", \"updates\": [{}], \"reference\": \"{}\", \"n\": {}, \"candidate\": \"{}\", \"admitted\": {admitted}}}{comma}
",
            bits(*dest), list(updates), bits(cell.reference), cell.n, bits(*cand)
        ));
    }
    s.push_str("  ]
}
");
    s
}
