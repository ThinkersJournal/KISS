//! Class-aware comparison of a candidate's outputs against the oracle-tagged
//! corpus. For `add` (a bit-stable exact-byte op) the comparator is the
//! NaN-relaxed exact-byte `agree` from `differential`. Each divergence is data.

use crate::differential::agree;
use crate::harness::advertised::select_and_compare_reduced;
use crate::harness::corpus::Vector;
use crate::structural::{compare_monoid_reduced_f32, Monoid};
use crate::DeterminismClass;

/// One caught divergence, reproducible by `index` into the corpus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Divergence {
    pub index: usize,
    pub a: f32,
    pub b: f32,
    pub expected: f32,
    pub actual: f32,
}

/// Difference a candidate's `outputs` (aligned 1:1 with `corpus`) against each
/// vector's oracle `expected`. Returns every divergence (empty ⇒ conformant).
pub fn run_binary(outputs: &[f32], corpus: &[Vector]) -> Vec<Divergence> {
    assert_eq!(outputs.len(), corpus.len(), "one output per corpus vector");
    let mut out = Vec::new();
    for (i, (v, &actual)) in corpus.iter().zip(outputs).enumerate() {
        if !agree(v.expected, actual) {
            out.push(Divergence { index: i, a: v.a, b: v.b, expected: v.expected, actual });
        }
    }
    out
}

/// One caught divergence in an array-valued (axis) reduction, at output `cell`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisDivergence {
    pub cell: usize,
    pub expected: f32,
    pub actual: f32,
}

/// Difference a candidate axis-reduction's `actual` cells against the oracle
/// `expected`, dispatching the comparator BY MONOID (Sum → order-invariant band
/// using `tol[i]`; Max → ±0-canon exact-byte, ignoring `tol`) — the STRUCTURAL
/// selection (3a). Its Contract-sourced sibling is [`run_axis_reduce_advertised`],
/// which the freeze-gate differential actually uses; this variant remains the
/// structural witness. Returns every divergence (empty ⇒ conformant).
pub fn run_axis_reduce(actual: &[f32], expected: &[f32], monoid: Monoid, tol: &[f32]) -> Vec<AxisDivergence> {
    assert_eq!(actual.len(), expected.len(), "one actual per expected cell");
    assert_eq!(tol.len(), expected.len(), "one tolerance per cell");
    let mut out = Vec::new();
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        if compare_monoid_reduced_f32(monoid, a, e, tol[i], 0.0).is_err() {
            out.push(AxisDivergence { cell: i, expected: e, actual: a });
        }
    }
    out
}

/// Difference an axis reduction with a **Contract-sourced** comparator: each cell is
/// compared via [`select_and_compare_reduced`], whose comparator is selected from the
/// `advertised` determinism class (honesty-checked against the op's true class),
/// **never** the monoid. This is the wired KISS-CONFORM-6.13-0006b path — the array
/// companion the freeze-gate differential runs, so the advertised class actually
/// drives the shipped comparison rather than only a self-contained test. `op`/`monoid`
/// identify the fold for the honesty check and the ±0 exception; `tol` is the per-cell
/// band (used only when the selected class is order-invariant). Returns every
/// divergence (empty ⇒ conformant); a too-permissive advertisement diverges every cell.
pub fn run_axis_reduce_advertised(
    actual: &[f32],
    expected: &[f32],
    op: &str,
    monoid: Monoid,
    advertised: DeterminismClass,
    tol: &[f32],
) -> Vec<AxisDivergence> {
    assert_eq!(actual.len(), expected.len(), "one actual per expected cell");
    assert_eq!(tol.len(), expected.len(), "one tolerance per cell");
    let mut out = Vec::new();
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        if select_and_compare_reduced(op, Some(monoid), advertised, a, e, tol[i], 0.0).is_err() {
            out.push(AxisDivergence { cell: i, expected: e, actual: a });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::corpus::tagged_corpus;

    #[test]
    fn identical_outputs_have_no_divergences() {
        let c = tagged_corpus(1, 16);
        let outs: Vec<f32> = c.iter().map(|v| v.expected).collect();
        assert!(run_binary(&outs, &c).is_empty());
    }

    #[test]
    fn a_single_wrong_output_is_caught_with_its_index() {
        let c = tagged_corpus(1, 16);
        let mut outs: Vec<f32> = c.iter().map(|v| v.expected).collect();
        outs[3] = outs[3] + 1.0; // perturb one (finite edge cases guarantee this differs)
        let d = run_binary(&outs, &c);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].index, 3);
    }

    #[test]
    fn axis_reduce_catches_out_of_band_and_wrong_max() {
        // Sum: within band accepted, outside caught.
        let exp = [6.0f32, 15.0];
        let tol = [1e-3f32, 1e-3];
        assert!(run_axis_reduce(&[6.0, 15.0], &exp, Monoid::Sum, &tol).is_empty());
        let d = run_axis_reduce(&[6.0, 15.5], &exp, Monoid::Sum, &tol);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].cell, 1);
        // Max: exact-byte — a 1-ULP diff is caught even with a nonzero tol array.
        let d2 = run_axis_reduce(
            &[3.0, f32::from_bits(6.0f32.to_bits() + 1)],
            &[3.0, 6.0],
            Monoid::Max,
            &tol,
        );
        assert_eq!(d2.len(), 1);
    }
}
