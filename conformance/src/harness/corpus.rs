//! A deterministic, provenance-tagged corpus of `add` invocations. Each vector
//! carries a tag naming the source of its expected value (KISS-CONFORM-6.5-0003)
//! — here, the from-scratch oracle. Reuses the `differential` edge set + PRNG.

use crate::differential::{edge_f32, SplitMix64};

/// One conformance vector: inputs, the oracle's expected output, and a
/// derivation-provenance tag (§6.5-0003).
#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub a: f32,
    pub b: f32,
    pub expected: f32,
    pub provenance: &'static str,
}

/// The derivation source tag for increment 1: the `add` reference decomposition
/// is the primitive-floor op itself, evaluated by `semantics::add`.
const PROVENANCE: &str = "oracle:KISS-OPS-6.4-0001/semantics::add";

/// The edge f32 × edge f32 grid, then `n` seeded-random pairs. Deterministic:
/// same seed → same vectors.
pub fn tagged_corpus(seed: u64, n: usize) -> Vec<Vector> {
    let make = |a: f32, b: f32| Vector { a, b, expected: crate::semantics::add(a, b), provenance: PROVENANCE };
    let edges = edge_f32();
    let mut v = Vec::new();
    for &a in &edges {
        for &b in &edges {
            v.push(make(a, b));
        }
    }
    let mut rng = SplitMix64::new(seed);
    for _ in 0..n {
        let a = f32::from_bits(rng.next_u64() as u32);
        let b = f32::from_bits(rng.next_u64() as u32);
        v.push(make(a, b));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_vector_is_tagged_and_expected_matches_the_oracle() {
        let n = 64;
        let c = tagged_corpus(0xC0FFEE, n);
        let edge_len = edge_f32().len();
        assert_eq!(c.len(), edge_len * edge_len + n, "edge×edge grid + n random");
        for v in &c {
            assert!(!v.provenance.is_empty(), "vector missing provenance tag (§6.5-0003)");
            // The expected value is the oracle's, bit-for-bit (NaN-relaxed).
            let o = crate::semantics::add(v.a, v.b);
            assert!(o.to_bits() == v.expected.to_bits() || (o.is_nan() && v.expected.is_nan()));
        }
    }

    #[test]
    fn corpus_is_deterministic() {
        assert_eq!(
            tagged_corpus(7, 32).iter().map(|v| (v.a.to_bits(), v.b.to_bits())).collect::<Vec<_>>(),
            tagged_corpus(7, 32).iter().map(|v| (v.a.to_bits(), v.b.to_bits())).collect::<Vec<_>>(),
        );
    }
}
