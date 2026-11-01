// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CONFORM-6.5-0011 — the oracle-differential harness is accompanied by executable
//! negative controls: for EACH determinism-class comparator (§6.8) it employs, one
//! deliberately incorrect candidate the harness detects as divergent and one independently
//! written correct candidate it accepts, with the wrong candidate diverging ONLY on the
//! inputs where the pinned behaviour is violated.
//!
//! Scope, stated rather than implied: this exercises the comparator layer on the CPU
//! (`compare_f32`, `compare_reduced_f32`) over a reproducible corpus, on every platform. The
//! foreign-C-kernel legs (`harness_*differential.rs`) are Windows-gated and add live-ABI
//! evidence per op; they are not what this test rests on. Cross-reference, not a
//! substitution: §6.5-0016 (a sweep's incompleteness is surfaced) is the attribution half of
//! the same pattern and is satisfied separately.

use kiss_conformance::structural::compare_reduced_f32;
use kiss_conformance::{compare_f32, DeterminismClass};

/// Every comparator class, with an EXHAUSTIVE match so adding a class to the enum breaks
/// this file's build until a control is written for it (a fourth comparator with no control
/// is the confident-negative-forever failure the clause exists to rule out).
fn control_slot(c: DeterminismClass) -> usize {
    match c {
        DeterminismClass::ExactByte => 0,
        DeterminismClass::UlpTolerance => 1,
        DeterminismClass::OrderInvariant => 2,
    }
}
const ALL_CLASSES: [DeterminismClass; 3] = [
    DeterminismClass::ExactByte,
    DeterminismClass::UlpTolerance,
    DeterminismClass::OrderInvariant,
];

/// A reproducible generator (xorshift64*), so the corpus regenerates byte-identically.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in [-1, 1).
    fn unit(&mut self) -> f32 {
        ((self.next() >> 40) as f32 / (1u64 << 23) as f32) - 1.0
    }
}

/// One control: the divergence set of the wrong candidate must equal the set of inputs the
/// control's own predicate says violate the pinned behaviour, and the correct candidate must
/// be accepted on every input.
fn run_control<I>(
    name: &str,
    corpus: &[I],
    accepts_correct: impl Fn(&I) -> Result<(), String>,
    accepts_wrong: impl Fn(&I) -> Result<(), String>,
    violates_pinned: impl Fn(&I) -> bool,
) {
    for (i, x) in corpus.iter().enumerate() {
        if let Err(e) = accepts_correct(x) {
            panic!("{name}: the independently written correct candidate was rejected at corpus index {i}: {e}");
        }
    }
    let diverged: Vec<usize> = (0..corpus.len()).filter(|&i| accepts_wrong(&corpus[i]).is_err()).collect();
    let violating: Vec<usize> = (0..corpus.len()).filter(|&i| violates_pinned(&corpus[i])).collect();
    assert!(
        !violating.is_empty(),
        "{name}: the corpus holds no input violating the pinned behaviour — the control would be vacuous"
    );
    assert_eq!(
        diverged, violating,
        "{name}: the wrong candidate must diverge on EXACTLY the inputs violating the pinned behaviour \
         (empty/short = the harness missed the bug; extra = it diverges on unrelated inputs, i.e. noise)"
    );
}

// ---- exact-byte: signed-zero sum (§6.4, IEEE-754 add) ------------------------------------

fn add_pairs() -> Vec<(f32, f32)> {
    let edges = [
        0.0f32,
        -0.0,
        1.0,
        -1.0,
        f32::MIN_POSITIVE,
        -f32::MIN_POSITIVE,
        f32::from_bits(1),
        f32::from_bits(0x8000_0001),
        f32::MAX,
        -f32::MAX,
    ];
    let mut v: Vec<(f32, f32)> = edges.iter().flat_map(|&a| edges.iter().map(move |&b| (a, b))).collect();
    // Finite draws only: a NaN-producing pair would compare payload/sign, a different control.
    let mut r = Rng(0xC0FFEE);
    for _ in 0..400 {
        let a = r.unit() * 1.0e3;
        let b = r.unit() * 1.0e3;
        v.push((a, b));
    }
    // Overflow-to-inf pairs are excluded for the same reason as NaN.
    v.retain(|&(a, b)| (a + b).is_finite());
    v
}

/// The pinned behaviour: IEEE-754 round-to-nearest add, so `(-0) + (-0) = -0`.
fn add_oracle(a: f32, b: f32) -> f32 {
    a + b
}
/// Independently written correct candidate: widen, add, round once (exact for binary32 add).
fn add_correct(a: f32, b: f32) -> f32 {
    ((a as f64) + (b as f64)) as f32
}
/// Deliberately incorrect: normalises a zero sum to +0 (loses the sign of zero).
fn add_normalise_zero(a: f32, b: f32) -> f32 {
    let r = a + b;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

// ---- ulp-tolerance: declared-ULP transcendental (§6.8) -----------------------------------

const DECLARED_ULP: u64 = 2;

fn exp_inputs() -> Vec<f32> {
    let mut v = vec![-80.0f32, -10.0, -1.0, 0.0, 1.0, 10.0, 10.5, 40.0, 80.0];
    let mut r = Rng(0xBADC0DE);
    for _ in 0..300 {
        v.push(r.unit() * 80.0);
    }
    v
}
fn exp_oracle(x: f32) -> f32 {
    (x as f64).exp() as f32
}
/// Independently written correct candidate: a different formulation (base-2 route).
fn exp_correct(x: f32) -> f32 {
    ((x as f64) * std::f64::consts::LOG2_E).exp2() as f32
}
/// Deliberately incorrect: right except for large arguments, where it is `DECLARED_ULP + 1`
/// ULPs high (a bad large-argument reduction) — outside the declared bound.
fn exp_bad_large_arg(x: f32) -> f32 {
    let r = exp_oracle(x);
    if x > 10.0 {
        f32::from_bits(r.to_bits() + DECLARED_ULP as u32 + 1)
    } else {
        r
    }
}

// ---- order-invariant: sum within the reassociation band (§6.0-0004) ----------------------

fn sum_corpus() -> Vec<Vec<f32>> {
    let mut v: Vec<Vec<f32>> = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        // Large-magnitude spread; the large element is last so dropping it is far outside the band.
        vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0e6],
        (0..64).map(|i| (i as f32 * 0.1) - 3.0).collect(),
        vec![0.5; 100],
        // Last element zero: dropping it is NOT a violation here — the wrong candidate must
        // agree on these, which is what shows it diverges only where the behaviour is violated.
        vec![1.0, 2.0, 3.0, 0.0],
        vec![0.25, -0.5, 8.0, 0.0],
    ];
    let mut r = Rng(0x5EED);
    for _ in 0..200 {
        let n = 3 + (r.next() % 60) as usize;
        let mut x: Vec<f32> = (0..n).map(|_| r.unit() * 100.0).collect();
        if let Some(last) = x.last_mut() {
            if last.abs() < 1.0 {
                *last += 2.0; // keep the dropped element unambiguously outside the band
            }
        }
        v.push(x);
    }
    v
}
fn band(x: &[f32]) -> f32 {
    x.len() as f32 * f32::EPSILON * x.iter().map(|e| e.abs()).sum::<f32>()
}
fn sum_oracle(x: &[f32]) -> f32 {
    x.iter().map(|&e| e as f64).sum::<f64>() as f32
}
/// Independently written correct candidate: pairwise (tree) summation, a different order.
fn sum_pairwise(x: &[f32]) -> f32 {
    match x.len() {
        0 => 0.0,
        1 => x[0],
        n => sum_pairwise(&x[..n / 2]) + sum_pairwise(&x[n / 2..]),
    }
}
/// Deliberately incorrect: drops the last element.
fn sum_drops_last(x: &[f32]) -> f32 {
    sum_pairwise(&x[..x.len() - 1])
}

// Backs[enforced]: KISS-CONFORM-6.5-0011
#[test]
fn test_conform_harness_negative_controls() {
    // The three controls, indexed by comparator class; the exhaustive `control_slot` match
    // above ties this list to the enum.
    let mut covered = [false; 3];

    let pairs = add_pairs();
    run_control(
        "exact-byte",
        &pairs,
        |&(a, b)| compare_f32(DeterminismClass::ExactByte, add_correct(a, b), add_oracle(a, b), 0),
        |&(a, b)| compare_f32(DeterminismClass::ExactByte, add_normalise_zero(a, b), add_oracle(a, b), 0),
        // The pinned behaviour is violated exactly where the oracle's sum is -0, which for
        // round-to-nearest add arises only from (-0) + (-0).
        |&(a, b)| add_oracle(a, b).to_bits() == 0x8000_0000,
    );
    covered[control_slot(DeterminismClass::ExactByte)] = true;

    let xs = exp_inputs();
    run_control(
        "ulp-tolerance",
        &xs,
        |&x| compare_f32(DeterminismClass::UlpTolerance, exp_correct(x), exp_oracle(x), DECLARED_ULP),
        |&x| compare_f32(DeterminismClass::UlpTolerance, exp_bad_large_arg(x), exp_oracle(x), DECLARED_ULP),
        |&x| x > 10.0,
    );
    // The threshold is exact: a deviation of precisely the declared bound is accepted.
    for &x in &xs {
        let at_bound = f32::from_bits(exp_oracle(x).to_bits() + DECLARED_ULP as u32);
        assert!(
            compare_f32(DeterminismClass::UlpTolerance, at_bound, exp_oracle(x), DECLARED_ULP).is_ok(),
            "ulp-tolerance: a deviation of exactly the declared {DECLARED_ULP} ULP must be accepted (x={x})"
        );
    }
    covered[control_slot(DeterminismClass::UlpTolerance)] = true;

    let vs = sum_corpus();
    // Guard the predicate's premise: every vector's last element is either zero or clearly
    // outside the band, so "violates" is not decided by rounding noise.
    for x in &vs {
        let last = *x.last().unwrap();
        assert!(last == 0.0 || last.abs() > 2.0 * band(x), "order-invariant: ambiguous last element {last} for band {}", band(x));
    }
    run_control(
        "order-invariant",
        &vs,
        |x| compare_reduced_f32(DeterminismClass::OrderInvariant, sum_pairwise(x), sum_oracle(x), band(x), 0.0),
        |x| compare_reduced_f32(DeterminismClass::OrderInvariant, sum_drops_last(x), sum_oracle(x), band(x), 0.0),
        |x| *x.last().unwrap() != 0.0,
    );
    covered[control_slot(DeterminismClass::OrderInvariant)] = true;

    for c in ALL_CLASSES {
        assert!(covered[control_slot(c)], "no negative control for comparator class {c:?}");
    }
}
