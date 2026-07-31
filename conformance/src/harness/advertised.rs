//! Contract-sourced comparator selection (KISS-CONFORM-6.13-0006b): select the
//! differential comparator from the op's advertised determinism class, never a
//! hardcoded structural map. The advertised class is honesty-checked against the
//! op's true class (§6.0-0005) before it drives the comparator.

use crate::determinism::{check_advertisement, op_true_class};
use crate::structural::{compare_reduced_f32, Monoid};
use crate::DeterminismClass;

/// Honesty-check the `advertised` class against the op's true class, then compare
/// `actual` vs `expected` with the comparator that **advertised** class selects
/// (via [`compare_reduced_f32`]). This is the 6.13-0006b path: the comparator is a
/// function of the advertisement, not of the monoid. Errors (before any compare) if
/// the op's true class is unknown, or the advertisement is too permissive (§6.0-0005).
pub fn select_and_compare_reduced(
    op: &str,
    monoid: Option<Monoid>,
    advertised: DeterminismClass,
    actual: f32,
    expected: f32,
    abs_tol: f32,
    rel_tol: f32,
) -> Result<(), String> {
    let true_class = op_true_class(op, monoid)
        .ok_or_else(|| format!("cannot honesty-check `{op}` (monoid {monoid:?}): true class unknown"))?;
    let selected = check_advertisement(advertised, true_class)?;
    compare_reduced_f32(selected, actual, expected, abs_tol, rel_tol)
}
