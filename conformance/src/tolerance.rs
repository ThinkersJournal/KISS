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
