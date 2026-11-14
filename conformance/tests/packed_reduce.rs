// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.11-0026 to §6.11-0029: the tuple-state reductions `welford`, `online_softmax` and `argmax`
//! (CireSnave 2026-11-12, board 427 "2) d)"). The reference (`packed_reduce`) is held against INDEPENDENT
//! derivations: an `f64` two-pass variance, an `f64` logsumexp, and a manual §6.11-0007 scan, none of which
//! share the merge formulas.

use kiss_conformance::opattrs::{encode_reduce, Monoid, MathFidelityOrd, ACCUMULATOR_COMPUTE_DTYPE};
use kiss_conformance::packed_reduce::*;

/// A deterministic xorshift stream of floats in a mixed range, with the occasional exact repeat.
fn stream(seed: u64, n: usize) -> Vec<f32> {
    let mut s = seed;
    (0..n)
        .map(|i| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let v = ((s >> 11) % 2001) as f32 / 100.0 - 10.0;
            if i % 5 == 4 { 1.5 } else { v }
        })
        .collect()
}

// harness-internal: the committed packed-reduction vectors JSON equals a fresh generation; artifact freshness, no clause is verified.
#[test]
fn test_packed_reduction_vectors_artifact_is_fresh() {
    let committed = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/packed_reduction_vectors.json"))
        .expect("run `cargo run --bin emit_packed_reduction_vectors > conformance/corpus/packed_reduction_vectors.json`");
    assert_eq!(committed.replace("\r\n", "\n"), emit_json(), "stale packed-reduction vectors");
}

// Backs[enforced]: KISS-OPS-6.11-0026 — a tuple-state reduction returns one packed tensor: the keepdim result (or its reduced-axis-removed form) plus one trailing state axis of extent 3 (welford) or 2 (online_softmax, argmax); a packed rank above MAX_RANK is a typed decline (a rank-8 operand with keepdim needs 9); argmax reduces exactly one axis; the state dtype is the compute dtype for f32/f64 and f32 below, none for an integer; the monoid ordinals are 5, 6, 7 and the accumulator stays 0x00
#[test]
fn test_ops_packed_reduction_shape() {
    assert_eq!(packed_output_shape(&[4, 8], &[1], true, Kind::Welford), Ok(vec![4, 1, 3]));
    assert_eq!(packed_output_shape(&[4, 8], &[1], false, Kind::Welford), Ok(vec![4, 3]));
    assert_eq!(packed_output_shape(&[4, 8], &[0, 1], false, Kind::OnlineSoftmax), Ok(vec![2]), "every axis reduced, no keepdim: only the state axis is left");
    assert_eq!(packed_output_shape(&[4, 8], &[1], true, Kind::ArgMax), Ok(vec![4, 1, 2]));
    // MAX_RANK: the boundary is exact.
    let r8 = [2usize; 8];
    assert_eq!(packed_output_shape(&r8, &[7], true, Kind::Welford), Err(Decline::RankExceedsMax { rank: 9 }));
    assert_eq!(packed_output_shape(&r8, &[7], false, Kind::Welford).map(|s| s.len()), Ok(8), "keepdim = 0 removes the axis, so rank 7 + 1 fits");
    assert_eq!(packed_output_shape(&r8[..7], &[6], true, Kind::OnlineSoftmax).map(|s| s.len()), Ok(8), "rank 7 with keepdim fits exactly");
    // argmax names exactly one axis.
    assert_eq!(packed_output_shape(&[4, 8], &[0, 1], true, Kind::ArgMax), Err(Decline::ArgmaxAxisCount { got: 2 }));
    assert_eq!(packed_output_shape(&[4, 8], &[], true, Kind::ArgMax), Err(Decline::ArgmaxAxisCount { got: 0 }));
    // state dtype.
    for (c, s) in [("f32", "f32"), ("f64", "f64"), ("f16", "f32"), ("bf16", "f32"), ("f8e4m3fn", "f32")] {
        assert_eq!(state_dtype(c), Some(s), "{c}");
    }
    assert_eq!(state_dtype("i32"), None);
    // wire: the three ordinals, with the accumulator byte 0x00 (state dtype derived) and bit-stable fidelity.
    for (m, ord, k) in [(Monoid::Welford, 5, Kind::Welford), (Monoid::OnlineSoftmax, 6, Kind::OnlineSoftmax), (Monoid::ArgMax, 7, Kind::ArgMax)] {
        assert_eq!(m.ordinal(), ord);
        assert_eq!(k.ordinal(), ord);
        assert_eq!(encode_reduce(m, &[1], 2, true, ACCUMULATOR_COMPUTE_DTYPE, MathFidelityOrd::BitStable), vec![ord, 0xFE, 0xFF, 0x01, 0x00, 0x01]);
    }
    assert!((1..=4).all(scan_accepts_monoid) && (5..=7).all(|o| !scan_accepts_monoid(o)) && !scan_accepts_monoid(0), "a scan takes only the scalar folds");
    let rejected = std::panic::catch_unwind(|| encode_reduce(Monoid::Welford, &[1], 2, true, 3, MathFidelityOrd::BitStable));
    assert!(rejected.is_err(), "a non-zero accumulator on a tuple-state monoid is refused");
}

fn assert_same(a: Welford, b: Welford, what: &str) {
    assert_eq!((a.n, a.mean.to_bits(), a.m2.to_bits()), (b.n, b.mean.to_bits(), b.m2.to_bits()), "{what}");
}

// Backs[enforced]: KISS-OPS-6.11-0027 — welford keeps (n, mean, M2) with identity (0, +0, +0) and element state (1, x, +0); the Chan merge makes any grouping agree within the declared band and agree with an independent f64 two-pass variance; an empty side returns the other, a NaN input makes mean and M2 NaN, a count past the lane is a typed decline, and the finalise is M2/n or M2/(n-1)
#[test]
fn test_ops_welford_reduction() {
    // identity laws, exact.
    for xs in [vec![], vec![2.5], vec![1.0, 2.0, 3.0, 4.0], stream(1, 33)] {
        let s = welford_fold(&xs).unwrap();
        assert_same(welford_merge(WELFORD_IDENTITY, s).unwrap(), s, "identity on the left");
        assert_same(welford_merge(s, WELFORD_IDENTITY).unwrap(), s, "identity on the right");
    }
    assert_same(welford_fold(&[]).unwrap(), WELFORD_IDENTITY, "an empty reduction yields the identity");
    let one = welford_elem(3.5);
    assert_eq!((one.n, one.mean, one.m2.to_bits()), (1, 3.5, 0), "element state (1, x, +0)");
    // independent oracle: f64 two-pass mean and sum of squared deviations.
    for seed in 1..=20u64 {
        let xs = stream(seed, 1 + (seed as usize * 7) % 97);
        let n = xs.len() as f64;
        let mean = xs.iter().map(|&x| x as f64).sum::<f64>() / n;
        let m2: f64 = xs.iter().map(|&x| (x as f64 - mean).powi(2)).sum();
        let seq = welford_fold(&xs).unwrap();
        assert_eq!(seq.n as usize, xs.len());
        assert!((seq.mean as f64 - mean).abs() <= 1e-5 * (1.0 + mean.abs()), "mean seed {seed}: {} vs {mean}", seq.mean);
        assert!((seq.m2 as f64 - m2).abs() <= 1e-4 * (1.0 + m2), "M2 seed {seed}: {} vs {m2}", seq.m2);
        // a different grouping (two halves, then merged) agrees within the band, not necessarily bit for bit.
        let h = xs.len() / 2;
        let tree = welford_merge(welford_fold(&xs[..h]).unwrap(), welford_fold(&xs[h..]).unwrap()).unwrap();
        assert_eq!(tree.n, seq.n);
        assert!((tree.m2 as f64 - seq.m2 as f64).abs() <= 1e-4 * (1.0 + m2), "grouping seed {seed}");
        assert!(seq.m2 >= 0.0, "M2 is non-negative by construction");
    }
    // the worked values of the spec: 1..4 -> n 4, mean 2.5, M2 5; population 1.25, Bessel 5/3.
    let s = welford_fold(&[1.0, 2.0, 3.0, 4.0]).unwrap();
    assert_eq!((s.n, s.mean, s.m2), (4, 2.5, 5.0));
    assert_eq!(welford_variance(s, false), 1.25);
    assert!((welford_variance(s, true) - 5.0 / 3.0).abs() < 1e-6);
    assert!(welford_variance(WELFORD_IDENTITY, false).is_nan() && welford_variance(welford_elem(1.0), true).is_nan(), "empty and n = 1 Bessel are 0/0");
    // NaN input: the count counts it, mean and M2 are NaN.
    let s = welford_fold(&[1.0, f32::NAN, 3.0]).unwrap();
    assert!(s.n == 3 && s.mean.is_nan() && s.m2.is_nan());
    // a count past the u32 lane is a typed decline, not a wrap.
    let big = Welford { n: u32::MAX, mean: 0.0, m2: 0.0 };
    assert_eq!(welford_merge(big, welford_elem(1.0)).err(), Some(Decline::CountOverflow));
}

fn same_soft(a: Soft, b: Soft) -> bool {
    a.m.to_bits() == b.m.to_bits() && a.l.to_bits() == b.l.to_bits()
}

// Backs[enforced]: KISS-OPS-6.11-0028 — online_softmax keeps (m, l) with identity (-inf, +0) and element state (x, 1); the merge rescales by exp(m - max) and agrees with an independent f64 logsumexp; both sides -inf stay the identity and never NaN; a -inf element adds no weight; a NaN or +inf input gives the NaN state; the finalise is m + log(l)
#[test]
fn test_ops_online_softmax_reduction() {
    for xs in [vec![], vec![3.0], vec![0.0, 0.0], stream(2, 40)] {
        let s = soft_fold(&xs);
        assert!(same_soft(soft_merge(SOFT_IDENTITY, s), s) && same_soft(soft_merge(s, SOFT_IDENTITY), s), "identity law {xs:?}");
    }
    assert!(same_soft(soft_fold(&[f32::NEG_INFINITY, f32::NEG_INFINITY]), SOFT_IDENTITY), "both -inf: the identity, not NaN");
    let s = soft_fold(&[f32::NEG_INFINITY, 2.0]);
    assert_eq!((s.m, s.l), (2.0, 1.0), "a -inf element adds no weight");
    let s = soft_fold(&[0.0, 0.0]);
    assert_eq!((s.m, s.l), (0.0, 2.0), "exp(0) = 1 exactly");
    assert!(soft_fold(&[1.0, f32::NAN]).m.is_nan() && soft_fold(&[1.0, f32::NAN]).l.is_nan());
    assert!(soft_fold(&[1.0, f32::INFINITY]).l.is_nan(), "+inf input is the NaN state");
    // independent oracle: f64 logsumexp.
    for seed in 1..=20u64 {
        let xs = stream(seed + 100, 2 + (seed as usize * 5) % 61);
        let m = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max) as f64;
        let lse = m + xs.iter().map(|&x| (x as f64 - m).exp()).sum::<f64>().ln();
        let got = soft_logsumexp(soft_fold(&xs)) as f64;
        assert!((got - lse).abs() <= 2e-5 * (1.0 + lse.abs()), "logsumexp seed {seed}: {got} vs {lse}");
        // two groupings agree within the band.
        let h = xs.len() / 2;
        let tree = soft_merge(soft_fold(&xs[..h]), soft_fold(&xs[h..]));
        assert!((soft_logsumexp(tree) as f64 - lse).abs() <= 2e-5 * (1.0 + lse.abs()), "grouping seed {seed}");
    }
}

/// An independent §6.11-0007 scan: keys by (NaN greatest, then value), ties to the lower index.
fn manual_argmax(xs: &[f32]) -> Option<(f32, u32)> {
    let mut best: Option<(f32, u32)> = None;
    for (i, &x) in xs.iter().enumerate() {
        best = match best {
            None => Some((x, i as u32)),
            Some((bv, bi)) => {
                let better = if x.is_nan() { !bv.is_nan() } else if bv.is_nan() { false } else { x > bv };
                if better { Some((x, i as u32)) } else { Some((bv, bi)) }
            }
        };
    }
    best
}

// Backs[enforced]: KISS-OPS-6.11-0029 — argmax keeps (value, index) with identity (-inf, 0xFFFFFFFF); the merge takes the greater key under the NaN-greatest total order, a tie to the lower index (the lowest-index NaN wins, -0 and +0 tie), is exact and gives the same state for every grouping, and equals an independent scan
#[test]
fn test_ops_argmax_reduction() {
    let nan = f32::NAN;
    let cases: Vec<Vec<f32>> = vec![
        vec![],
        vec![1.0, 3.0, 2.0],
        vec![2.0, 2.0],
        vec![nan, 1.0, nan],
        vec![-0.0, 0.0],
        vec![0.0, -0.0],
        vec![f32::NEG_INFINITY, f32::NEG_INFINITY],
        stream(3, 25),
        vec![1.0, nan, 5.0, nan],
    ];
    for xs in &cases {
        let got = arg_fold(xs);
        match manual_argmax(xs) {
            None => assert_eq!((got.v.to_bits(), got.i), (ARG_IDENTITY.v.to_bits(), u32::MAX), "empty is the identity"),
            Some((v, i)) => assert_eq!((got.v.to_bits(), got.i), (v.to_bits(), i), "{xs:?}"),
        }
        // grouping independence: every split point merges to the same bits.
        for cut in 0..=xs.len() {
            let left = xs[..cut].iter().enumerate().fold(ARG_IDENTITY, |s, (p, &x)| arg_merge(s, arg_elem(x, p as u32)));
            let right = xs[cut..].iter().enumerate().fold(ARG_IDENTITY, |s, (p, &x)| arg_merge(s, arg_elem(x, (cut + p) as u32)));
            let m = arg_merge(left, right);
            assert_eq!((m.v.to_bits(), m.i), (got.v.to_bits(), got.i), "split {cut} of {xs:?}");
        }
        assert_eq!((arg_merge(ARG_IDENTITY, got).v.to_bits(), arg_merge(ARG_IDENTITY, got).i), (got.v.to_bits(), got.i), "identity on the left");
    }
    // the worked cases.
    assert_eq!(arg_fold(&[nan, 1.0, nan]).i, 0, "lowest-index NaN");
    let z = arg_fold(&[-0.0, 0.0]);
    assert_eq!((z.v.to_bits(), z.i), (0x8000_0000, 0), "-0 and +0 are equal, the lower index and its bits win");
    assert_eq!(arg_fold(&[f32::NEG_INFINITY, f32::NEG_INFINITY]).i, 0, "an element equal to the identity's value still replaces it");
}

// Backs[enforced]: KISS-OPS-6.11-0026 — the committed vectors are the reference output: every case recomputes to its lane words from its input words, in ascending fold order
#[test]
fn test_ops_packed_reduction_vectors_recompute() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/packed_reduction_vectors.json")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| l.contains("\"input\"")) {
        let field = |k: &str| -> String {
            let i = line.find(&format!("\"{k}\": ")).unwrap() + k.len() + 4;
            let rest = &line[i..];
            if let Some(r) = rest.strip_prefix('"') { r[..r.find('"').unwrap()].to_string() } else { rest[1..rest.find(']').unwrap()].to_string() }
        };
        let kind = field("kind");
        let input: Vec<f32> = field("input").split(',').filter(|w| !w.trim().is_empty()).map(|w| f32::from_bits(u32::from_str_radix(w.trim().trim_matches('"'), 16).unwrap())).collect();
        let lanes: Vec<u32> = field("lanes").split(' ').map(|w| u32::from_str_radix(w, 16).unwrap()).collect();
        let want: Vec<u32> = match kind.as_str() {
            "welford" => welford_lanes(welford_fold(&input).unwrap()).to_vec(),
            "online_softmax" => soft_lanes(soft_fold(&input)).to_vec(),
            "argmax" => arg_lanes(arg_fold(&input)).to_vec(),
            k => panic!("kind {k}"),
        };
        assert_eq!(lanes, want, "{line}");
        checked += 1;
    }
    assert_eq!(checked, 19, "the pinned case population");
}
