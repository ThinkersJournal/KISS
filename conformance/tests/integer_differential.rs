//! KISS-Conform §6.5 randomized differential loop for the **integer** atoms.
//!
//! As with the float differential, the value is that it *catches* a wrong
//! implementation, not merely that a right one passes. These run over a
//! reproducible integer corpus (edges + splitmix-seeded draws) and demonstrate
//! both directions: a correct alternate implementation agrees everywhere, and
//! the three classic integer bugs — a **saturating** add, a **saturating** abs at
//! INT_MIN, and an **always-logical** shr — are each caught, on exactly the
//! boundary/negative inputs where the pinned semantics (§6.2-0002, §6.4-0005,
//! §6.10-0003) forbids them.

use kiss_conformance::integer::*;

const SEED: u64 = 0xC0FFEE;
const N: usize = 400;

// ---- correct alternate implementations agree over the whole corpus ----------

/// An independently written wrapping add via u32 modular arithmetic (i32) — the
/// "correct implementation under test", different code shape, same §6.2-0002
/// semantics.
fn add_alt_i32(a: i32, b: i32) -> i32 {
    ((a as u32).wrapping_add(b as u32)) as i32
}

#[test]
fn correct_add_candidate_agrees_over_the_whole_corpus() {
    let corpus = int_corpus::<i32>(SEED, N);
    let divs = diff_int_binary(add, add_alt_i32, &corpus);
    assert!(divs.is_empty(), "a correct wrapping-add candidate diverged: {:?}", divs.first());
}

// ---- bug 1: a SATURATING add is caught at the MAX/MIN boundary (§6.2-0002) ---

/// The exact bug §6.2-0002 forbids: clamping instead of wrapping on overflow.
fn saturating_add_i32(a: i32, b: i32) -> i32 {
    a.saturating_add(b)
}

#[test]
fn differential_catches_the_saturating_add_bug() {
    let corpus = int_corpus::<i32>(SEED, N);
    let divs = diff_int_binary(add, saturating_add_i32, &corpus);
    assert!(!divs.is_empty(), "differential failed to catch the saturating-add bug");
    // every divergence is a genuine overflow: the true wrapping sum differs from
    // the clamped one only when the mathematical sum leaves [MIN, MAX].
    assert!(
        divs.iter().all(|d| (d.a as i64 + d.b as i64) < i32::MIN as i64
            || (d.a as i64 + d.b as i64) > i32::MAX as i64),
        "a divergence appeared where no overflow occurred"
    );
}

// ---- bug 2: a SATURATING abs is caught at INT_MIN (§6.4-0005) ----------------

/// The bug §6.4-0005 forbids: `abs` that saturates INT_MIN to INT_MAX instead of
/// wrapping to INT_MIN.
fn saturating_abs_i32(a: i32) -> i32 {
    if a == i32::MIN { i32::MAX } else { a.wrapping_abs() }
}

#[test]
fn differential_catches_the_saturating_abs_bug() {
    let corpus = int_corpus::<i32>(SEED, N);
    let divs = diff_int_unary(abs, saturating_abs_i32, &corpus);
    assert!(!divs.is_empty(), "differential failed to catch the saturating-abs bug");
    // the ONLY input where wrapping and saturating abs differ is INT_MIN.
    assert!(divs.iter().all(|d| d.a == i32::MIN), "divergence outside INT_MIN");
    assert!(divs.iter().any(|d| d.a == i32::MIN), "the INT_MIN divergence must appear");
}

// ---- bug 3: an ALWAYS-LOGICAL shr is caught on negative signed inputs --------
// (§6.10-0003: shr on a signed dtype MUST be arithmetic / sign-replicating.)

/// The single most bug-prone atom implemented wrong: a right shift that always
/// zero-fills (logical) even for signed operands — correct for non-negatives,
/// wrong for every negative (`shr(-1,k)` becomes a large positive, not `-1`).
fn logical_shr_i32(a: i32, amt: u32) -> i32 {
    ((a as u32) >> amt) as i32
}

#[test]
fn differential_catches_the_always_logical_shr_bug() {
    let corpus = int_corpus::<i32>(SEED, N);
    // Sweep EVERY in-domain shift amount 0..BITS (not just a single mid-range k):
    // the whole [0, BITS) domain is differenced, so the low boundary k==0 (where
    // both shapes are the identity and MUST agree) and the high boundary
    // k==BITS-1 are exercised, not only k=4.
    let mut all_divs = Vec::new();
    let mut checked_k0 = false;
    for k in 0..(<i32 as IntDtype>::BITS as i64) {
        let oracle = |a: i32| shr(a, k).unwrap();
        let candidate = |a: i32| logical_shr_i32(a, k as u32);
        let divs = diff_int_unary(oracle, candidate, &corpus);
        if k == 0 {
            // shr(a, 0) == a for BOTH arithmetic and logical — no divergence at
            // the low edge of the domain.
            assert!(divs.is_empty(), "shr-by-0 is the identity; both shapes must agree");
            checked_k0 = true;
        } else {
            // every k>0 catches the bug, and only on negatives (arithmetic fills 1s).
            assert!(!divs.is_empty(), "always-logical shr not caught at k={k}");
            assert!(divs.iter().all(|d| d.a < 0), "divergence on a non-negative input at k={k}");
        }
        all_divs.extend(divs);
    }
    assert!(checked_k0, "the shift-by-0 low-boundary case must be exercised");
    assert!(!all_divs.is_empty(), "differential failed to catch the always-logical shr bug");
    // across the sweep, shr(-1,k) diverges for every k>0 (arith=-1, logical!=-1).
    assert!(all_divs.iter().any(|d| d.a == -1), "shr(-1,k) must diverge (arith=-1, logical!=-1)");
}

// ---- unsigned shr is ALREADY logical — the same candidate agrees on u32 ------

#[test]
fn unsigned_shr_is_logical_and_agrees() {
    // on an unsigned dtype the pinned shr IS logical, so a logical candidate must
    // agree everywhere (the split is signedness-keyed, §6.10-0003).
    let corpus = int_corpus::<u32>(SEED, N);
    let k: i64 = 4;
    let oracle = |a: u32| shr(a, k).unwrap();
    let candidate = |a: u32| a >> (k as u32);
    let divs = diff_int_unary(oracle, candidate, &corpus);
    assert!(divs.is_empty(), "unsigned shr diverged from a logical shift: {:?}", divs.first());
}

// ---- self-consistency and corpus reproducibility (Conform §6.5) -------------

#[test]
fn oracle_is_self_consistent_under_fuzz() {
    let corpus = int_corpus::<i64>(7, 300);
    assert!(diff_int_binary(add, add, &corpus).is_empty());
    assert!(diff_int_binary(mul, mul, &corpus).is_empty());
    assert!(diff_int_binary(bit_xor, bit_xor, &corpus).is_empty());
    assert!(diff_int_unary(neg, neg, &corpus).is_empty());
    assert!(diff_int_unary(abs, abs, &corpus).is_empty());
}

#[test]
fn corpus_is_reproducible() {
    // same seed/n -> identical corpus (deterministic fuzzer reproducibility).
    let a: Vec<u64> = int_corpus::<i64>(42, 100).iter().map(|x| x.to_bits_u64()).collect();
    let b: Vec<u64> = int_corpus::<i64>(42, 100).iter().map(|x| x.to_bits_u64()).collect();
    assert_eq!(a, b);
    // the edge vector actually contains the pinned boundaries.
    let edges = int_edge_vec::<i32>();
    assert!(edges.contains(&i32::MIN));
    assert!(edges.contains(&i32::MAX));
    assert!(edges.contains(&-1i32));
    assert!(edges.contains(&0i32));
}
