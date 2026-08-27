//! A reproducible, randomized differential loop (Conform §6.5).
//!
//! Instead of only hand-picked golden vectors, a candidate implementation is
//! differenced against the from-scratch [`crate::semantics`] oracle over a corpus
//! of edge cases plus seeded-random bit patterns. The corpus is **deterministic**
//! — same seed → same inputs — so a failing case reproduces exactly (the
//! fuzzer-corpus-reproducibility concern KISS-Conform flags). The point is not
//! that a correct candidate passes; it is that an *incorrect* one is caught.

/// splitmix64 — a tiny deterministic PRNG, no external dependency (a conformance
/// harness stays dependency-free, Conform §6.5).
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// The edge-case f32 values every corpus includes: zeros, units, infinities, a
/// bare and a payload NaN, the extremes, and a subnormal. These are exactly the
/// values where sloppy implementations diverge.
pub fn edge_f32() -> Vec<f32> {
    vec![
        0.0, -0.0, 1.0, -1.0, 2.0, -2.0, 0.5, -0.5,
        f32::INFINITY, f32::NEG_INFINITY,
        f32::NAN, f32::from_bits(0x7FC0_1234), // quiet NaN carrying a payload
        f32::MIN, f32::MAX, f32::MIN_POSITIVE,
        f32::from_bits(0x0000_0001), // smallest positive subnormal
    ]
}

/// A reproducible corpus: the edge cases, then `n` seeded-random bit patterns
/// (uniform over the 32-bit space, so NaNs / infinities / subnormals / normals
/// all appear).
pub fn corpus_f32(seed: u64, n: usize) -> Vec<f32> {
    let mut rng = SplitMix64::new(seed);
    let mut v = edge_f32();
    for _ in 0..n {
        v.push(f32::from_bits(rng.next_u64() as u32));
    }
    v
}

/// Two results **agree** iff both are NaN, or their bits are identical: exact-byte
/// for finite/infinite values, with a NaN-equivalence relaxation that treats ANY NaN as
/// equal to any NaN — payload and sign are NOT compared.
///
/// SCOPE — correct ONLY for the COMPUTED-NaN op class (`add` and the arithmetic/
/// transcendental ops, Conform §6.8-0010): the op GENERATES its NaN, so the payload is
/// architectural, not semantic, and blindness to it is the right relaxation — it still
/// catches a propagate-vs-suppress bug (a NaN where a number is required, or vice versa),
/// which is the conformance-relevant fact there. `diff_binary` below (two freshly-computed
/// results) is in scope.
///
/// It MUST NOT be used for a MOVED-NaN / exact-byte-payload op — `minmax` (its decomposition
/// is a `select`), `select`, `gather` — whose NaN output is a MOVED input value that
/// Conform §6.8-0010(a) pins payload-included. Those need the exact-byte path
/// (`compare_f32(DeterminismClass::ExactByte, ..)`), never this relation: routing such a vector
/// here makes it PASS regardless of whether the payload survived — a control that cannot
/// fail. A pinned-vector runner (`run_binary`) MUST therefore select its comparator by op
/// under Conform §6.8-0008 precedence (KISS #339(a)) and never route a moved-NaN vector to `agree`.
pub fn agree(x: f32, y: f32) -> bool {
    (x.is_nan() && y.is_nan()) || x.to_bits() == y.to_bits()
}

/// One divergence found by the differential runner.
#[derive(Debug, Clone, Copy)]
pub struct Divergence {
    pub a: f32,
    pub b: f32,
    pub oracle: f32,
    pub candidate: f32,
}

/// Difference a binary candidate against the oracle over all ordered pairs of the
/// corpus; return every divergence (empty ⇒ the candidate matches the oracle on
/// every pair). A unary op is differenced by ignoring the second argument.
pub fn diff_binary(
    oracle: impl Fn(f32, f32) -> f32,
    candidate: impl Fn(f32, f32) -> f32,
    corpus: &[f32],
) -> Vec<Divergence> {
    let mut out = Vec::new();
    for &a in corpus {
        for &b in corpus {
            let (o, c) = (oracle(a, b), candidate(a, b));
            if !agree(o, c) {
                out.push(Divergence { a, b, oracle: o, candidate: c });
            }
        }
    }
    out
}
