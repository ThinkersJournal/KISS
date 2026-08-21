//! KISS-Conform oracle-differential vectors for the **structural access atoms**
//! (KISS-Ops §6.11): `reduce`, `prefix_scan`, `gather`, `scatter`. These transcribe
//! the pinned monoid identities, empty-reduction rule, OOB policies, combine algebra
//! and the ONE nondeterministic op (fp `scatter` atomic-add), each vector citing its
//! clause. Comparators are selected by determinism class (Conform §6.8): exact-byte
//! for the deterministic atoms, the order-invariant comparator for fp sum/prod folds
//! and atomic-add.

use kiss_conformance::differential::SplitMix64;
use kiss_conformance::semantics::{add, fmax_ieee, fmin_ieee, mul};
use kiss_conformance::structural::*;
use kiss_conformance::{compare_f32, DeterminismClass};

fn bits_eq(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

// ---- reduce: monoid identities + empty-reduction (KISS-OPS §6.11-0002) -------

// Backs: KISS-OPS-6.11-0002
#[test]
fn reduce_empty_axis_yields_monoid_identity() {
    // §6.11-0002: "a reduction over an empty axis MUST yield the monoid identity."
    assert!(bits_eq(reduce_f32(&[], Monoid::Sum), 0.0)); // +0.0
    assert_eq!(reduce_f32(&[], Monoid::Prod), 1.0);
    assert_eq!(reduce_f32(&[], Monoid::Max), f32::NEG_INFINITY); // dtype minimum
    assert_eq!(reduce_f32(&[], Monoid::Min), f32::INFINITY); // dtype maximum
}

#[test]
fn reduce_ordinary_values() {
    // Golden multiset {1,2,3,4}: sum=10, prod=24, max=4, min=1.
    let xs = [1.0, 2.0, 3.0, 4.0];
    assert_eq!(reduce_f32(&xs, Monoid::Sum), 10.0);
    assert_eq!(reduce_f32(&xs, Monoid::Prod), 24.0);
    assert_eq!(reduce_f32(&xs, Monoid::Max), 4.0);
    assert_eq!(reduce_f32(&xs, Monoid::Min), 1.0);
}

// Backs: KISS-OPS-6.11-0002
#[test]
fn reduce_maxmin_are_nan_propagating() {
    // §6.11-0002: max/min monoids MUST be NaN-propagating (not IEEE maxNum).
    let xs = [3.0, f32::NAN, 1.0];
    assert!(reduce_f32(&xs, Monoid::Max).is_nan());
    assert!(reduce_f32(&xs, Monoid::Min).is_nan());
}

// Backs: KISS-OPS-6.2-0004
#[test]
fn reduce_maxmin_preserve_signed_zero() {
    // §6.2-0004 via the max_prop/min_prop decomposition: -0.0 not normalized.
    assert!(bits_eq(reduce_f32(&[-0.0], Monoid::Max), -0.0));
    assert!(bits_eq(reduce_f32(&[-0.0], Monoid::Min), -0.0));
    // sum identity is +0.0, and +0.0 + -0.0 = +0.0 per IEEE-754.
    assert!(bits_eq(reduce_f32(&[-0.0], Monoid::Sum), 0.0));
}

// Backs: KISS-OPS-6.0-0002, KISS-OPS-6.0-0004
#[test]
fn reduce_class_is_selected_per_spec() {
    // §6.0-0002: max/min reduce is exact-byte; §6.0-0004: fp sum/prod is
    // order-invariant/nondeterministic.
    assert_eq!(Monoid::Max.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Monoid::Min.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Monoid::Sum.class_f32(), DeterminismClass::OrderInvariant);
    assert_eq!(Monoid::Prod.class_f32(), DeterminismClass::OrderInvariant);
}

// ---- prefix_scan: inclusive/exclusive, length-preserving (§6.11-0003) --------

// Proven: KISS-OPS-6.11-0003 (subject: impl; ref: PROVEN_BATCH1.md)
#[test]
fn scan_is_length_preserving() {
    // Backs: KISS-OPS-6.11-0003 — one output element per input position, distinct from reduce.
    let xs = [5.0, 6.0, 7.0];
    assert_eq!(prefix_scan_f32(&xs, Monoid::Sum, ScanKind::Inclusive).len(), 3);
    assert_eq!(prefix_scan_f32(&xs, Monoid::Sum, ScanKind::Exclusive).len(), 3);
    // empty input -> empty output (length preserved).
    assert_eq!(prefix_scan_f32(&[], Monoid::Sum, ScanKind::Inclusive), Vec::<f32>::new());
}

#[test]
fn scan_inclusive_vs_exclusive_golden() {
    // cumsum = prefix_scan(sum, inclusive) (§6.13 table).
    let xs = [1.0, 2.0, 3.0, 4.0];
    assert_eq!(
        prefix_scan_f32(&xs, Monoid::Sum, ScanKind::Inclusive),
        vec![1.0, 3.0, 6.0, 10.0]
    );
    // exclusive: out[0] = identity (+0.0), out[i] excludes xs[i].
    assert_eq!(
        prefix_scan_f32(&xs, Monoid::Sum, ScanKind::Exclusive),
        vec![0.0, 1.0, 3.0, 6.0]
    );
    // cumprod inclusive; cummax inclusive.
    assert_eq!(
        prefix_scan_f32(&xs, Monoid::Prod, ScanKind::Inclusive),
        vec![1.0, 2.0, 6.0, 24.0]
    );
    assert_eq!(
        prefix_scan_f32(&[3.0, 1.0, 4.0, 1.0], Monoid::Max, ScanKind::Inclusive),
        vec![3.0, 3.0, 4.0, 4.0]
    );
}

#[test]
fn scan_exclusive_max_first_is_identity() {
    // exclusive max: out[0] = -inf (Max identity), then the running max of the prefix.
    let out = prefix_scan_f32(&[3.0, 1.0, 4.0], Monoid::Max, ScanKind::Exclusive);
    assert_eq!(out[0], f32::NEG_INFINITY);
    assert_eq!(out[1], 3.0);
    assert_eq!(out[2], 3.0);
}

// ---- gather: OOB policy {skip, clamp, zero-fill} (§6.11-0004) -----------------

// Backs: KISS-OPS-6.11-0004
#[test]
fn gather_in_bounds_reads_datum() {
    let d = [10.0, 20.0, 30.0];
    assert_eq!(
        gather_f32(&d, &[2, 0, 1], OobRead::Skip),
        vec![Some(30.0), Some(10.0), Some(20.0)]
    );
}

#[test]
fn gather_oob_policies() {
    // KISS-OPS-6.11-0004: {skip -> unwritten, clamp -> nearest in-range, zero-fill -> 0}.
    let d = [10.0, 20.0, 30.0];
    assert_eq!(gather_f32(&d, &[9], OobRead::Skip), vec![None]);
    assert_eq!(gather_f32(&d, &[9], OobRead::ZeroFill), vec![Some(0.0)]);
    assert_eq!(gather_f32(&d, &[9], OobRead::Clamp), vec![Some(30.0)]);
    assert_eq!(gather_f32(&d, &[0], OobRead::Clamp), vec![Some(10.0)]);
}

#[test]
fn gather_negative_index_never_wraps() {
    // KISS-OPS-6.11-0004: a negative index (signed dtype) is ALWAYS OOB, no from-end wrap.
    let d = [10.0, 20.0, 30.0];
    assert_eq!(gather_f32(&d, &[-1], OobRead::Skip), vec![None]);
    assert_eq!(gather_f32(&d, &[-3], OobRead::ZeroFill), vec![Some(0.0)]);
    // clamp of a negative goes to index 0 (the low end), not the tail.
    assert_eq!(gather_f32(&d, &[-100], OobRead::Clamp), vec![Some(10.0)]);
}

// Backs: KISS-OPS-6.0-0002
#[test]
fn gather_preserves_read_datum_bits() {
    // gather is exact-byte (§6.0-0002): a -0.0 or NaN datum is read verbatim.
    let d = [-0.0f32, f32::from_bits(0x7FC0_1234)];
    let g = gather_f32(&d, &[0, 1], OobRead::Skip);
    assert!(bits_eq(g[0].unwrap(), -0.0));
    assert_eq!(g[1].unwrap().to_bits(), 0x7FC0_1234);
}

// ---- scatter: combine algebra + OOB skip (§6.11-0005/-0006/-0010) ------------

#[test]
fn scatter_assign_last_writer_in_iteration_order_wins() {
    // KISS-OPS-6.11-0006: pinned tie-break — the highest source (row-major) index wins.
    let mut dest = [0.0, 0.0];
    scatter_f32(&mut dest, &[1, 1, 1], &[7.0, 8.0, 9.0], Combine::Assign);
    assert_eq!(dest, [0.0, 9.0]);
}

// Proven: KISS-OPS-6.11-0005 (subject: impl; ref: PROVEN_BATCH1.md)
#[test]
fn scatter_oob_writes_are_skipped() {
    // Backs: KISS-OPS-6.11-0005 — OOB write skipped; a negative index is OOB (cf. KISS-OPS-6.11-0004, the OOB-policy clause — referenced, not backed here).
    let mut dest = [1.0, 2.0, 3.0];
    scatter_f32(&mut dest, &[-1, 5, 1], &[10.0, 20.0, 30.0], Combine::Assign);
    assert_eq!(dest, [1.0, 30.0, 3.0]); // only the in-bounds idx 1 wrote
}

// Proven: KISS-OPS-6.11-0010 (subject: impl; ref: PROVEN_BATCH1.md)
#[test]
fn scatter_atomic_max_min_nan_propagating() {
    // Backs: KISS-OPS-6.11-0010 — fp atomic-max/min NaN-propagating — NaN scattered OR already
    // present yields NaN.
    let mut a = [1.0];
    scatter_f32(&mut a, &[0], &[f32::NAN], Combine::AtomicMax);
    assert!(a[0].is_nan());
    let mut b = [f32::NAN];
    scatter_f32(&mut b, &[0], &[5.0], Combine::AtomicMin);
    assert!(b[0].is_nan());
    // ordinary values behave as max_prop/min_prop.
    let mut c = [2.0];
    scatter_f32(&mut c, &[0, 0], &[5.0, 3.0], Combine::AtomicMax);
    assert_eq!(c[0], 5.0);
}

#[test]
fn scatter_deterministic_combines_are_exact_byte() {
    // KISS-OPS-6.11-0006: assign / atomic-max / atomic-min are deterministic (exact-byte).
    assert_eq!(Combine::Assign.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Combine::AtomicMax.class_f32(), DeterminismClass::ExactByte);
    assert_eq!(Combine::AtomicMin.class_f32(), DeterminismClass::ExactByte);
}

// ---- the ONE nondeterministic op: fp scatter atomic-add (§6.11-0006) ---------

// Backs: KISS-OPS-6.0-0004, KISS-OPS-6.11-0006
#[test]
fn scatter_fp_atomic_add_is_order_invariant_class() {
    // §6.11-0006 / §6.0-0004: the fp atomic-add combine is the one nondeterministic
    // op — class order-invariant/nondeterministic.
    assert_eq!(Combine::AtomicAdd.class_f32(), DeterminismClass::OrderInvariant);
}

// Backs: KISS-CONFORM-6.8-0004
#[test]
fn atomic_add_two_orderings_diverge_but_agree_order_invariantly() {
    // THE crux (Conform §6.8-0004): two visit orders of the same atomic-add multiset
    // give different f32 sums; exact-byte is the WRONG comparator, the order-invariant
    // comparator is the RIGHT one.
    let mut a = [0.0]; // ((0+1e8) + -1e8) + 1.0 = 1.0
    scatter_f32(&mut a, &[0, 0, 0], &[1e8_f32, -1e8, 1.0], Combine::AtomicAdd);
    let mut b = [0.0]; // ((0+1e8) + 1.0) + -1e8 = 0.0
    scatter_f32(&mut b, &[0, 0, 0], &[1e8_f32, 1.0, -1e8], Combine::AtomicAdd);
    assert_eq!(a[0], 1.0);
    assert_eq!(b[0], 0.0);

    // exact-byte rejects the real divergence ...
    assert!(compare_f32(DeterminismClass::ExactByte, a[0], b[0], 0).is_err());
    // ... order-invariant accepts it within the contract-declared tolerance.
    let abs_sum = 1e8_f32 + 1e8 + 1.0;
    let tol = 2.0 * reassoc_bound_f32(3, abs_sum);
    assert!(order_invariant_agree(a[0], b[0], tol, 0.0));
    assert!(compare_order_invariant(&a, &b, tol, 0.0).is_ok());
}

#[test]
fn order_invariant_still_catches_a_real_error() {
    // The comparator is not a rubber stamp: a divergence far beyond the reassociation
    // bound (a genuine bug, not reordering) is still rejected.
    let tol = 2.0 * reassoc_bound_f32(3, 3.0); // tiny magnitudes -> tiny tol
    assert!(!order_invariant_agree(1.0, 2.0, tol, 0.0));
    assert!(compare_order_invariant(&[1.0], &[2.0], tol, 0.0).is_err());
    // NaN-equivalence still holds (both NaN agree); a NaN vs number does not.
    assert!(order_invariant_agree(f32::NAN, f32::NAN, 0.0, 0.0));
    assert!(!order_invariant_agree(f32::NAN, 1.0, 1e30, 1.0));
}

// Backs: KISS-CONFORM-6.8-0006
#[test]
fn class_aware_dispatch_selects_the_comparator() {
    // Conform §6.8-0006: the comparator is SELECTED by the declared class, never
    // chosen by the test author. compare_reduced_f32 dispatches:
    //  - Max reduce (exact-byte) compares bit-for-bit,
    //  - Sum reduce (order-invariant) accepts reassociation.
    let exact_ok = compare_reduced_f32(Monoid::Max.class_f32(), 4.0, 4.0, 0.0, 0.0);
    assert!(exact_ok.is_ok());
    let one_ulp_above_4 = f32::from_bits(4.0f32.to_bits() + 1); // 4.0 + 1 ULP
    let exact_bad = compare_reduced_f32(Monoid::Max.class_f32(), 4.0, one_ulp_above_4, 0.0, 0.0);
    assert!(exact_bad.is_err(), "max reduce must compare exact-byte");
    let oi_ok = compare_reduced_f32(Monoid::Sum.class_f32(), 10.0, one_ulp_above_4 + 6.0, 1e-3, 0.0);
    assert!(oi_ok.is_ok(), "sum reduce accepts reassociation within tolerance");
}

// ---- randomized differential loop (Conform §6.5): reproducible input + index
// arrays via the seeded SplitMix64 corpus idea. The point is not that a correct
// scatter passes — it is that an INCORRECT one is caught, reproducibly. -----------

/// A deliberately WRONG scatter: floating-point atomic-add implemented as plain
/// `assign` — the classic §6.11-0006 bug that keeps only the last contribution to
/// each destination and drops every earlier addend. The differential must catch it.
fn wrong_scatter_assign_instead_of_add(dest: &mut [f32], indices: &[i64], src: &[f32]) {
    let len = dest.len();
    for (k, &idx) in indices.iter().enumerate() {
        if idx >= 0 && (idx as u64) < len as u64 {
            dest[idx as usize] = src[k]; // BUG: assigns, never accumulates.
        }
    }
}

/// A deliberately WRONG gather: ignores the OOB policy and Python-style tail-wraps a
/// negative or overrun index — exactly the from-end wrap §6.11-0004 forbids. It must
/// diverge from the spec `Clamp` policy on every OOB position.
fn wrong_gather_wraps(data: &[f32], indices: &[i64]) -> Vec<Option<f32>> {
    let len = data.len() as i64;
    indices
        .iter()
        .map(|&idx| {
            let w = ((idx % len) + len) % len; // BUG: wraps instead of clamping.
            Some(data[w as usize])
        })
        .collect()
}

/// Reproducible finite f32s in ~[-128, 128) from a seeded SplitMix64 stream (the
/// differential-corpus idea, kept finite so a genuine bug — not NaN/inf masking or
/// float noise — is what diverges).
fn seeded_finite_f32(rng: &mut SplitMix64) -> f32 {
    ((rng.next_u64() >> 40) as f32 - 8_388_608.0) / 65_536.0
}

// Backs: KISS-CONFORM-6.8-0004, KISS-OPS-6.11-0006
// (§6.11-0004 is the GATHER OOB clause — this SCATTER test does not exercise gather; its
// backing is the forward-named gather_oob_policies. #247 dropped the loose cite.)
#[test]
fn differential_scatter_atomic_add_reproducible() {
    // Reproducible src + index arrays (Conform §6.5: same seed -> same inputs, so a
    // failing case reproduces exactly).
    let n = 40usize;
    let dest_len = 4usize;
    let mut rng = SplitMix64::new(0x5CA7_7E12_ADD0_0001);
    let src: Vec<f32> = (0..n).map(|_| seeded_finite_f32(&mut rng)).collect();
    // indices in [-3, dest_len+3): a mix of in-bounds, negative-OOB and overrun-OOB.
    let indices: Vec<i64> = (0..n)
        .map(|_| (rng.next_u64() % (dest_len as u64 + 6)) as i64 - 3)
        .collect();

    // Canonical (row-major k) ordering — the oracle.
    let mut good = vec![0.0f32; dest_len];
    scatter_f32(&mut good, &indices, &src, Combine::AtomicAdd);

    // A DIFFERENT but legal visit order of the SAME multiset (reversed): a valid
    // reassociation, not a semantic change.
    let mut ridx = indices.clone();
    ridx.reverse();
    let mut rsrc = src.clone();
    rsrc.reverse();
    let mut reordered = vec![0.0f32; dest_len];
    scatter_f32(&mut reordered, &ridx, &rsrc, Combine::AtomicAdd);

    // Per-cell reassociation tolerance, conservatively bounded by the total addend
    // magnitude (Conform §6.8-0004: the declared, derived tolerance — not a byte
    // compare).
    let abs_sum: f32 = src.iter().map(|x| x.abs()).sum();
    let tol = 2.0 * reassoc_bound_f32(n, abs_sum);

    // Two legal visit orders AGREE under the order-invariant comparator ...
    assert!(
        compare_order_invariant(&good, &reordered, tol, 0.0).is_ok(),
        "two visit orders of the same atomic-add multiset must agree order-invariantly"
    );
    // ... but the lossy scatter (assign-instead-of-add) does NOT — the differential
    // catches the bug well beyond any reassociation tolerance.
    let mut bad = vec![0.0f32; dest_len];
    wrong_scatter_assign_instead_of_add(&mut bad, &indices, &src);
    assert!(
        compare_order_invariant(&good, &bad, tol, 0.0).is_err(),
        "assign-instead-of-add scatter must be caught by the differential"
    );
}

#[test]
fn differential_catches_gather_ignoring_oob_policy() {
    // Reproducible data + index arrays; the index array deliberately spans OOB on
    // both ends so the spec Clamp policy and a forbidden wrapping gather MUST differ.
    let mut rng = SplitMix64::new(0x6A7E_0B0B_C0DE_0007);
    let data: Vec<f32> = (0..6).map(|_| seeded_finite_f32(&mut rng)).collect();
    let indices: Vec<i64> = (0..32)
        .map(|_| (rng.next_u64() % (data.len() as u64 + 8)) as i64 - 4)
        .collect();
    // At least one OOB index is present (spans [-4, len+4)); assert so the test is
    // exercising the divergence, not a vacuously in-bounds run.
    assert!(
        indices.iter().any(|&i| i < 0 || i as usize >= data.len()),
        "corpus must contain an OOB index to exercise the policy divergence"
    );

    let good = gather_f32(&data, &indices, OobRead::Clamp);
    let bad = wrong_gather_wraps(&data, &indices);
    // In-bounds positions agree; every OOB position clamps vs wraps, so the vectors
    // differ — the differential catches a gather that ignores the OOB policy.
    assert!(
        good != bad,
        "a wrapping gather must be caught by the differential vs the clamp policy"
    );
}

/// KISS-OPS-6.11-0002: `reduce` folds an axis under a monoid from {sum,prod,max,min}
/// with the pinned identities; `max`/`min` are **NaN-propagating**; an empty axis
/// yields the identity.
/// Teeth: a reduce(max) built on the NaN-*suppressing* IEEE `fmax` (seeded at −∞)
/// returns a finite value on a NaN-containing axis instead of NaN — caught here.
#[test]
fn test_ops_reduce_monoids() {
    // (a) pinned identities — also the empty-axis result (§6.11-0002).
    assert!(bits_eq(reduce_f32(&[], Monoid::Sum), 0.0)); // sum identity +0.0
    assert_eq!(reduce_f32(&[], Monoid::Prod), 1.0); // prod identity 1
    assert_eq!(reduce_f32(&[], Monoid::Max), f32::NEG_INFINITY); // max identity −∞ (dtype min)
    assert_eq!(reduce_f32(&[], Monoid::Min), f32::INFINITY); // min identity +∞ (dtype max)

    // (b) ordinary fold values over the golden multiset {2, 4, 3, 1}.
    let xs = [2.0f32, 4.0, 3.0, 1.0];
    assert_eq!(reduce_f32(&xs, Monoid::Sum), 10.0);
    assert_eq!(reduce_f32(&xs, Monoid::Prod), 24.0);
    assert_eq!(reduce_f32(&xs, Monoid::Max), 4.0);
    assert_eq!(reduce_f32(&xs, Monoid::Min), 1.0);

    // (c) THE teeth: max/min are NaN-PROPAGATING (§6.11-0002), NOT IEEE maxNum/minNum.
    let with_nan = [3.0f32, f32::NAN, 1.0];
    assert!(reduce_f32(&with_nan, Monoid::Max).is_nan());
    assert!(reduce_f32(&with_nan, Monoid::Min).is_nan());

    // A reduce(max) built on the NaN-SUPPRESSING IEEE fmax (seeded at −∞) silently
    // drops the NaN and returns 3.0 — the exact wrong impl this clause forbids.
    let wrong_max = with_nan
        .iter()
        .fold(f32::NEG_INFINITY, |acc, &x| fmax_ieee(acc, x));
    let wrong_min = with_nan
        .iter()
        .fold(f32::INFINITY, |acc, &x| fmin_ieee(acc, x));
    assert_eq!(wrong_max, 3.0);
    assert_eq!(wrong_min, 1.0);
    assert!(reduce_f32(&with_nan, Monoid::Max).to_bits() != wrong_max.to_bits());
    assert!(reduce_f32(&with_nan, Monoid::Min).to_bits() != wrong_min.to_bits());
}

/// KISS-OPS-6.11-0002a: a compute dtype with no infinity encoding (`e4m3fn`)
/// materializes the ±∞ `max`/`min` monoid identity as its finite extreme (∓448),
/// since it cannot represent ±∞. The teeth: −∞ is genuinely unrepresentable here,
/// so the finite extreme is FORCED — and it is a real monoid identity (≤/≥ every
/// element), not an arbitrary sentinel.
#[test]
fn test_ops_reduce_identity_no_inf_dtype() {
    use kiss_conformance::dtype::{e4m3_decode, E4M3_MAX_FINITE};

    // every representable e4m3fn value (all 256 bytes; the NaN pattern(s) yield None).
    let finite: Vec<f32> = (0u8..=255).filter_map(|b| e4m3_decode(b).value()).collect();
    let nan_bytes = 256 - finite.len();
    assert!((1..=2).contains(&nan_bytes), "e4m3fn has 1–2 NaN byte patterns, rest finite (saw {nan_bytes} NaN)");

    // e4m3fn has NO infinity encoding — the §6.11-0002 abstract identity −∞ is not
    // representable, so it MUST be materialized as something else.
    assert!(!finite.iter().any(|v| v.is_infinite()), "e4m3fn cannot encode ±∞");

    // …that something is the finite extreme: −448 for max, +448 for min, and both
    // ARE representable (some byte decodes to each).
    let max_identity = -E4M3_MAX_FINITE; // −448
    let min_identity = E4M3_MAX_FINITE; //  +448
    assert!(finite.iter().any(|&v| v == max_identity), "−448 is representable in e4m3fn");
    assert!(finite.iter().any(|&v| v == min_identity), "+448 is representable in e4m3fn");

    // the monoid law — the reason these finite extremes are valid identities and not
    // arbitrary sentinels: the max identity is ≤ every element, the min identity ≥
    // every element, because ±448 are exactly the dtype's finite bounds.
    for &v in &finite {
        assert!(max_identity <= v, "max identity −448 must be ≤ every e4m3fn value (saw {v})");
        assert!(min_identity >= v, "min identity +448 must be ≥ every e4m3fn value (saw {v})");
    }
}

/// KISS-OPS-6.0-0006: for an exact-byte op over a float dtype each arithmetic atom
/// rounds independently — `add(mul(a,b),c)` MUST round the `mul` then the `add`, and
/// MUST NOT contract to a single-rounding `fma(a,b,c)`.
/// Teeth: an oracle that fuses `mul`+`add` into `f32::mul_add` produces a different
/// byte result on the chosen a,b,c — caught by the exact-byte comparator.
#[test]
fn test_ops_no_fma_contraction_exact_byte() {
    // a = 1 + 2^-12 (exactly representable). a*a = 1 + 2^-11 + 2^-24; the trailing
    // 2^-24 is exactly half a ULP at 1.0, so round-to-nearest-even rounds it DOWN to
    // mul = 1 + 2^-11 (0x3F801000). add(mul, -1) = 2^-11 (0x3A000000).
    let a = f32::from_bits(0x3F80_0800); // 1.0002441 = 1 + 2^-12
    let c = -1.0f32;

    // Separate per-atom rounding: round(mul) THEN round(add) — the required behavior.
    let sep = add(mul(a, a), c);
    assert_eq!(mul(a, a).to_bits(), 0x3F80_1000); // 1 + 2^-11
    assert_eq!(sep.to_bits(), 0x3A00_0000); // 2^-11, exactly

    // The contracted (fused) alternative keeps the exact product, so a*a-1 rounds only
    // ONCE to 2^-11 + 2^-24 (0x3A000400) — a genuinely different byte pattern.
    let fused = a.mul_add(a, c);
    assert_eq!(fused.to_bits(), 0x3A00_0400);

    // §6.0-0006: the exact-byte decomposition MUST equal the separately-rounded value
    // and MUST NOT equal the fused one; an FMA-contracted oracle fails the comparator.
    assert_ne!(sep.to_bits(), fused.to_bits());
    assert!(
        compare_f32(DeterminismClass::ExactByte, sep, fused, 0).is_err(),
        "an FMA-contracted oracle would fail the exact-byte comparator"
    );
}

/// A deliberately WRONG `sort_network` comparator: the raw `a < b` relation, which is
/// **non-transitive on NaN** (every comparison involving NaN is `false`, so NaN
/// compares "equal" to every key). It leaves NaNs wherever they started and scrambles
/// the order — exactly the bug §6.11-0007's total order forbids.
fn wrong_sort_less_than(row: &[f32]) -> Vec<usize> {
    use std::cmp::Ordering;
    let mut order: Vec<usize> = (0..row.len()).collect();
    order.sort_by(|&i, &j| {
        let (a, b) = (row[i], row[j]);
        if a < b {
            Ordering::Less
        } else if a > b {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    order
}

/// KISS-OPS-6.11-0007: `sort_network` is a stable per-row permutation under a total
/// order on (key, original-index) where NaN orders greatest (ascending → NaN last,
/// descending → NaN first) and ties break by the lower original index. Two outputs:
/// the raw-bit value permutation and the original-index vector.
/// Teeth: an `a < b` comparator is non-transitive on NaN, so it leaves NaN unsorted
/// and scrambles the order — caught by asserting the exact index vector.
#[test]
fn test_ops_sort_network_total_order() {
    let nan = f32::NAN;
    // keys 3,1,NaN,1,2,NaN — duplicate 1.0s (idx 1,3) and two NaNs (idx 2,5) exercise
    // the stability tie-break in both the finite and the NaN region.
    let row = [3.0f32, 1.0, nan, 1.0, 2.0, nan];

    // Ascending: NaN LAST; ties (the two 1.0s, the two NaNs) keep input order.
    let (av, ai) = sort_network(&row, SortDirection::Ascending);
    assert_eq!(ai, vec![1, 3, 4, 0, 2, 5]);
    assert_eq!(av[0], 1.0);
    assert_eq!(av[1], 1.0);
    assert_eq!(av[2], 2.0);
    assert_eq!(av[3], 3.0);
    assert!(av[4].is_nan() && av[5].is_nan());

    // Descending: NaN FIRST; ties STILL break by lower original index (stability does
    // NOT flip with direction).
    let (dv, di) = sort_network(&row, SortDirection::Descending);
    assert_eq!(di, vec![2, 5, 0, 4, 1, 3]);
    assert!(dv[0].is_nan() && dv[1].is_nan());
    assert_eq!(dv[2], 3.0);
    assert_eq!(dv[3], 2.0);
    assert_eq!(dv[4], 1.0);
    assert_eq!(dv[5], 1.0);

    // argmax = rank 0 of the descending index vector (§6.13 table). Use a NaN-free row
    // (NaN-first would otherwise mask the numeric maximum).
    let clean = [3.0f32, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0];
    let (_, ci) = sort_network(&clean, SortDirection::Descending);
    assert_eq!(ci[0], 5); // argmax → index of 9.0

    // Raw-bit value permutation carries ±0 signs verbatim (no normalization): with
    // all-equal zero keys the order is pure stability and each sign bit rides along.
    let zeros = [0.0f32, -0.0, 0.0];
    let (zv, zi) = sort_network(&zeros, SortDirection::Ascending);
    assert_eq!(zi, vec![0, 1, 2]);
    assert_eq!(zv[0].to_bits(), 0x0000_0000);
    assert_eq!(zv[1].to_bits(), 0x8000_0000); // -0.0 preserved
    assert_eq!(zv[2].to_bits(), 0x0000_0000);

    // THE teeth: a raw `a < b` comparator is non-transitive on NaN, leaving NaN
    // unsorted and scrambling the permutation — it does NOT reproduce the total order.
    let wrong = wrong_sort_less_than(&row);
    assert_ne!(
        wrong, ai,
        "an `a < b` comparator must not reproduce the §6.11-0007 total order"
    );
}
