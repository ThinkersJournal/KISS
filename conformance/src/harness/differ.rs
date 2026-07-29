//! Class-aware comparison of a candidate's outputs against the oracle-tagged
//! corpus. For `add` (a bit-stable exact-byte op) the comparator is the
//! NaN-relaxed exact-byte `agree` from `differential`. Each divergence is data.

use crate::differential::agree;
use crate::harness::corpus::Vector;

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
}
