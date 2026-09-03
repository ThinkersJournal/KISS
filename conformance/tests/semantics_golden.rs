//! KISS-Conform differential/semantic vectors for the pinned floating-point
//! primitives (KISS-Ops §2.3 / §6.15). These are the numeric distinctions the
//! spec spends prose pinning — now executable: NaN-propagating vs NaN-suppressing
//! min/max, `relu` ≠ `max(x,0)`, and the §6.8 "declared-ULP, not bit-identity"
//! transcendental model. Determinism-class comparators from Conform §6.8.

use kiss_conformance::fp::{bf16_to_f32, f32_to_bf16};
use kiss_conformance::semantics::*;
use kiss_conformance::{compare_c32_transcendental, compare_f32, ulp_distance_f32, DeterminismClass};

/// Raw-bit equality — the exact-byte determinism class for a scalar (§6.8).
fn bits_eq(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

// ---- the load-bearing distinction: NaN-propagating vs NaN-suppressing --------

/// Enforces KISS-OPS-6.15-0001 — max_prop/min_prop (NaN-propagating) and
/// fmax_ieee/fmin_ieee (NaN-suppressing) are four distinct ops. Catches an
/// `fmax_ieee` built from `max_prop`: it returns NaN for fmax_ieee(NaN, 5.0)
/// where the clause pins 5.0.
#[test]
fn max_propagates_nan_but_fmax_suppresses_it() {
    let nan = f32::NAN;
    // max_prop keeps a NaN operand; fmax_ieee returns the other — the whole reason
    // KISS-Ops keeps these as four distinct, non-mergeable ops.
    assert!(max_prop(nan, 5.0).is_nan());
    assert!(max_prop(5.0, nan).is_nan());
    assert_eq!(fmax_ieee(nan, 5.0), 5.0);
    assert_eq!(fmax_ieee(5.0, nan), 5.0);
    assert!(fmax_ieee(nan, nan).is_nan()); // both NaN -> NaN
    // they agree on ordinary values
    assert_eq!(max_prop(3.0, 5.0), 5.0);
    assert_eq!(fmax_ieee(3.0, 5.0), 5.0);
}

#[test]
fn min_propagates_nan_but_fmin_suppresses_it() {
    let nan = f32::NAN;
    assert!(min_prop(nan, 5.0).is_nan());
    assert!(min_prop(5.0, nan).is_nan());
    assert_eq!(fmin_ieee(nan, 5.0), 5.0);
    assert_eq!(fmin_ieee(5.0, nan), 5.0);
    assert_eq!(min_prop(3.0, 5.0), 3.0);
}

#[test]
fn max_prop_returns_the_nan_operand_bit_for_bit() {
    // the §6.15 decomposition returns operand `a` itself when it is NaN, so the
    // NaN payload is preserved exactly.
    let nan = f32::from_bits(0x7FC0_1234);
    assert_eq!(max_prop(nan, 5.0).to_bits(), nan.to_bits());
}

#[test]
fn signed_zero_follows_cmp_ge_operand_order() {
    // fmax_ieee(-0, +0): both numbers, cmp_ge(-0,+0) is true (equal) -> returns a.
    assert!(bits_eq(fmax_ieee(-0.0, 0.0), -0.0));
    assert!(bits_eq(fmax_ieee(0.0, -0.0), 0.0));
}

// ---- §6.13 signed-zero tie vectors (issue #74): operand `a` wins every tie ---
//
// All four §6.13 minmax decompositions share the IDENTICAL innermost select —
// `cmp_ge(a,b) -> a` for max_prop/fmax_ieee, `cmp_le(a,b) -> a` for
// min_prop/fmin_ieee (the §6.13 table) — and differ ONLY in the NaN arms. Under
// §6.6 signed-zero equality `cmp_ge(-0.0, +0.0)` and `cmp_le(-0.0, +0.0)` are
// both TRUE, so operand `a` wins every ±0 tie in ALL FOUR ops; no op in the
// minmax family is b-biased on a tie.
//
// The 16-row vector set, applied at each covered dtype (expected = operand `a`,
// bit-for-bit):
//
// |     op      |   a  |   b  | expected |
// |-------------|------|------|----------|
// | `max_prop`  | +0.0 | -0.0 |   +0.0   |
// | `max_prop`  | -0.0 | +0.0 |   -0.0   |
// | `max_prop`  | +0.0 | +0.0 |   +0.0   |
// | `max_prop`  | -0.0 | -0.0 |   -0.0   |
// | `min_prop`  | +0.0 | -0.0 |   +0.0   |
// | `min_prop`  | -0.0 | +0.0 |   -0.0   |
// | `min_prop`  | +0.0 | +0.0 |   +0.0   |
// | `min_prop`  | -0.0 | -0.0 |   -0.0   |
// | `fmax_ieee` | +0.0 | -0.0 |   +0.0   |
// | `fmax_ieee` | -0.0 | +0.0 |   -0.0   |
// | `fmax_ieee` | +0.0 | +0.0 |   +0.0   |
// | `fmax_ieee` | -0.0 | -0.0 |   -0.0   |
// | `fmin_ieee` | +0.0 | -0.0 |   +0.0   |
// | `fmin_ieee` | -0.0 | +0.0 |   -0.0   |
// | `fmin_ieee` | +0.0 | +0.0 |   +0.0   |
// | `fmin_ieee` | -0.0 | -0.0 |   -0.0   |
//
// HARNESS RULE: the comparison MUST be raw bits (`to_bits()`), never a float
// value compare — `0.0 == -0.0` is true, so a value compare passes VACUOUSLY;
// the expected column above is a BIT PATTERN. Dtype coverage is f32 + f64 +
// one narrow float dtype (bf16): a promote-compute-round implementation must
// prove the sign bit survives the widen -> compute -> round-back trip.
//
// Provenance (non-normative): the `max_prop` rows are decomposition-traced AND
// empirically cross-confirmed by the kiss-ref<->Baracuda step-2 recipe
// differential (Baracuda fix 7297f17d); the `min_prop`/`fmax_ieee`/`fmin_ieee`
// rows are decomposition-traced. Drafted by kiss-ref; cosigned kiss-ref +
// Baracuda (issue #74). The same 16 rows are frozen per-dtype as fixture cells
// in `corpus/ops-minmax-signed-zero.json` and run differentially by
// `tests/corpus_differential.rs`.

/// The 16 f32 rows: every ±0 tie returns operand `a` bit-for-bit in all four
/// minmax ops. Catches a `>`-spelled tie (`a > b ? a : b` — b on ties), the
/// exact cross-implementation divergence that motivated the set, and any
/// tie-normalizing lowering (e.g. `+0.0` on every tie).
#[test]
fn minmax_signed_zero_ties_keep_operand_a_f32() {
    let ops: [(&str, fn(f32, f32) -> f32); 4] = [
        ("max_prop", max_prop),
        ("min_prop", min_prop),
        ("fmax_ieee", fmax_ieee),
        ("fmin_ieee", fmin_ieee),
    ];
    let ties: [(f32, f32); 4] = [(0.0, -0.0), (-0.0, 0.0), (0.0, 0.0), (-0.0, -0.0)];
    for (name, f) in ops {
        for (a, b) in ties {
            assert_eq!(
                f(a, b).to_bits(),
                a.to_bits(),
                "{name}({a:?}, {b:?}) must keep operand `a` bit-for-bit on the ±0 tie"
            );
        }
    }
}

/// The 16 f64 rows — the same decompositions at the f64 compute dtype.
#[test]
fn minmax_signed_zero_ties_keep_operand_a_f64() {
    let ops: [(&str, fn(f64, f64) -> f64); 4] = [
        ("max_prop", max_prop_f64),
        ("min_prop", min_prop_f64),
        ("fmax_ieee", fmax_ieee_f64),
        ("fmin_ieee", fmin_ieee_f64),
    ];
    let ties: [(f64, f64); 4] = [(0.0, -0.0), (-0.0, 0.0), (0.0, 0.0), (-0.0, -0.0)];
    for (name, f) in ops {
        for (a, b) in ties {
            assert_eq!(
                f(a, b).to_bits(),
                a.to_bits(),
                "{name}({a:?}, {b:?}) must keep operand `a` bit-for-bit on the ±0 tie"
            );
        }
    }
}

/// The 16 bf16 rows through the promote -> compute -> round path (§6.16-0003
/// RNE store rounding): widen each bf16 operand exactly to f32, run the f32
/// oracle, round the result back to bf16. The tie result is ±0 — exactly
/// representable — so the ONLY thing the round-trip could lose is the sign
/// bit; these rows prove it survives.
// Backs: KISS-OPS-6.16-0003
#[test]
fn minmax_signed_zero_ties_keep_operand_a_bf16_roundtrip() {
    const PZ: u16 = 0x0000; // bf16 +0.0
    const NZ: u16 = 0x8000; // bf16 -0.0
    let ops: [(&str, fn(f32, f32) -> f32); 4] = [
        ("max_prop", max_prop),
        ("min_prop", min_prop),
        ("fmax_ieee", fmax_ieee),
        ("fmin_ieee", fmin_ieee),
    ];
    let ties: [(u16, u16); 4] = [(PZ, NZ), (NZ, PZ), (PZ, PZ), (NZ, NZ)];
    for (name, f) in ops {
        for (a, b) in ties {
            let r = f32_to_bf16(f(bf16_to_f32(a), bf16_to_f32(b)));
            assert_eq!(
                r, a,
                "{name}(bf16 {a:#06X}, {b:#06X}) must keep operand `a`'s sign bit \
                 through promote-compute-round"
            );
        }
    }
}

// ---- relu is NOT max(x, 0) ---------------------------------------------------

#[test]
fn relu_propagates_nan_and_preserves_negative_zero() {
    assert!(relu(f32::NAN).is_nan(), "relu propagates NaN");
    assert!(bits_eq(relu(-0.0), -0.0), "relu preserves -0.0");
    assert!(bits_eq(relu(-1.0), 0.0), "relu of a negative yields +0.0");
    assert_eq!(relu(2.5), 2.5);
    assert!(bits_eq(relu(0.0), 0.0));
}

/// Enforces KISS-OPS-6.15-0002 — `relu` is NaN-propagating and -0.0-preserving,
/// and MUST NOT be implemented as `max(x, 0)`. Catches exactly that wrong
/// lowering: `naive_max_x_zero` scrubs a NaN input to 0 where `relu` keeps it.
#[test]
fn relu_diverges_from_max_x_zero_at_nan() {
    // The deterministic divergence §2.3 warns about: relu keeps NaN, max(x,0)
    // scrubs it to 0. (max(x,0)'s sign of zero is platform-unspecified, so only
    // the NaN case is asserted for the naive form.)
    assert!(relu(f32::NAN).is_nan());
    assert!(!naive_max_x_zero(f32::NAN).is_nan(), "max(x,0) scrubs NaN to 0");
}

// ---- the comparators themselves (Conform §6.8) ------------------------------

#[test]
fn exact_byte_f32_distinguishes_signed_zero_and_nan() {
    assert!(compare_f32(DeterminismClass::ExactByte, -0.0, -0.0, 0).is_ok());
    assert!(compare_f32(DeterminismClass::ExactByte, -0.0, 0.0, 0).is_err(), "±0 differ under exact-byte");
    let n = f32::NAN;
    assert!(compare_f32(DeterminismClass::ExactByte, n, n, 0).is_ok());
}

#[test]
fn ulp_comparator_measures_distance() {
    let x = 1.0f32;
    let next = f32::from_bits(x.to_bits() + 1); // next representable f32 above 1.0
    assert_eq!(ulp_distance_f32(x, x), 0);
    assert_eq!(ulp_distance_f32(x, next), 1);
    assert!(compare_f32(DeterminismClass::UlpTolerance, next, x, 1).is_ok());
    assert!(compare_f32(DeterminismClass::UlpTolerance, next, x, 0).is_err());
}

// ---- broadened primitive floor: sign-bit, sign/step, rounding, comparisons --

#[test]
fn sign_bit_atoms_signed_zero() {
    assert!(bits_eq(neg(0.0), -0.0));
    assert!(bits_eq(neg(-0.0), 0.0));
    assert!(bits_eq(abs(-0.0), 0.0));
    assert!(bits_eq(abs(-3.5), 3.5));
    assert!(neg(f32::NAN).is_nan()); // flips sign, keeps NaN
    assert!(bits_eq(copysign(3.0, -1.0), -3.0));
    assert!(bits_eq(copysign(-3.0, 0.0), 3.0));
    assert!(bits_eq(copysign(3.0, -0.0), -3.0)); // sign of a signed zero carries
}

#[test]
fn sign_and_step_at_nan_and_zero() {
    assert_eq!(sign(2.0), 1.0);
    assert_eq!(sign(-3.0), -1.0);
    assert_eq!(sign(0.0), 0.0);
    assert_eq!(sign(-0.0), 0.0);
    assert_eq!(sign(f32::NAN), 0.0); // NaN -> 0 (both cmp_gt and cmp_lt are false)
    assert_eq!(step(2.0), 1.0);
    assert_eq!(step(0.0), 0.0);
    assert_eq!(step(-1.0), 0.0);
    assert_eq!(step(f32::NAN), 0.0);
}

/// Enforces KISS-OPS-6.7-0001 — floor→−∞, ceil→+∞, trunc→zero, and round_even
/// to nearest ties-to-even. Catches `round_even` lowered to C `roundf` / Rust
/// `f32::round` (ties away from zero): those give round_even(2.5)==3.0 where the
/// clause pins 2.0.
#[test]
fn round_even_is_bankers_rounding() {
    // round-half-to-even, NOT round-half-away-from-zero
    assert_eq!(round_even(0.5), 0.0);
    assert_eq!(round_even(1.5), 2.0);
    assert_eq!(round_even(2.5), 2.0);
    assert_eq!(round_even(3.5), 4.0);
    assert_eq!(round_even(-2.5), -2.0);
    assert_eq!(floor(1.7), 1.0);
    assert_eq!(ceil(1.2), 2.0);
    assert_eq!(trunc(-1.7), -1.0);
}

/// Enforces KISS-OPS-6.6-0002 — cmp_eq/lt/le/gt/ge each yield 0 (false) when
/// either operand is NaN. Catches `cmp_ge` lowered as `!(a < b)` (the standard
/// trick on an lt-only ISA): that returns true for cmp_ge(NaN, NaN) where the
/// clause pins false.
#[test]
fn comparisons_are_ieee_ordered() {
    assert!(cmp_eq(2.0, 2.0) && !cmp_eq(2.0, 3.0));
    assert!(cmp_lt(1.0, 2.0) && cmp_ge(2.0, 2.0) && cmp_le(2.0, 2.0));
    // a NaN operand makes every ordered comparison false ...
    assert!(!cmp_eq(f32::NAN, f32::NAN));
    assert!(!cmp_lt(f32::NAN, 1.0) && !cmp_gt(f32::NAN, 1.0) && !cmp_ge(f32::NAN, f32::NAN));
    // ... except cmp_ne, which is true — and isnan(x) == cmp_ne(x, x)
    assert!(cmp_ne(f32::NAN, f32::NAN));
    assert!(isnan(f32::NAN) && !isnan(0.0));
}

/// Enforces KISS-OPS-6.15-0003 — `rem_floor` (sign of divisor) and `rem_trunc`
/// (sign of dividend) are distinct ops. Catches an implementation that merges
/// them or substitutes one for the other: they diverge on any operand pair whose
/// quotient is negative and non-integer.
#[test]
fn test_ops_rem_floor_vs_trunc() {
    // -7 / 3 = -2.33: floor -> -3, trunc -> -2, so the remainders differ in sign.
    assert_eq!(rem_floor(-7.0, 3.0), 2.0); // sign of the divisor (+)
    assert_eq!(rem_trunc(-7.0, 3.0), -1.0); // sign of the dividend (-)
    assert_ne!(rem_floor(-7.0, 3.0), rem_trunc(-7.0, 3.0), "the two ops must not merge");
    // they agree when the quotient is exact or positive
    assert_eq!(rem_floor(7.0, 3.0), rem_trunc(7.0, 3.0));
    assert_eq!(rem_floor(6.0, 3.0), 0.0);
}

/// Enforces KISS-OPS-6.13-0007 — `hypot` yields +∞ whenever either operand is
/// infinite, even if the other is NaN, overriding the naive `sqrt(a²+b²)`.
/// Catches exactly that naive lowering: it returns NaN for `(±∞, NaN)`.
#[test]
fn test_ops_hypot_inf_nan() {
    let inf = f32::INFINITY;
    let nan = f32::NAN;
    // the infinity rule overrides NaN on an infinite input ...
    assert_eq!(hypot(inf, nan), inf);
    assert_eq!(hypot(nan, -inf), inf);
    assert_eq!(hypot(inf, 3.0), inf);
    // ... but the naive reference (which the clause overrides) would give NaN:
    assert!((inf * inf + nan * nan).sqrt().is_nan());
    // for finite inputs a NaN operand propagates, and ordinary values are exact.
    assert!(hypot(3.0, nan).is_nan());
    assert_eq!(hypot(3.0, 4.0), 5.0);
}

#[test]
fn transcendental_agrees_within_declared_ulp() {
    // §6.8: a transcendental's semantics is "the named function to within a
    // declared ULP", not bit-identity. A high-accuracy f64 oracle and the
    // f32-native path are two independent computations that must agree within a
    // modest declared bound (a stand-in for the §6.8 per-atom ULP ceiling).
    let bound = 4;
    for &x in &[-2.0f32, -0.5, 0.0, 0.5, 1.0, 2.0, 3.5] {
        let oracle = (x as f64).exp() as f32; // compute in f64, round once
        let under_test = x.exp(); // f32-native
        assert!(
            compare_f32(DeterminismClass::UlpTolerance, under_test, oracle, bound).is_ok(),
            "exp({x}): {} ULP exceeds bound {bound}",
            ulp_distance_f32(under_test, oracle)
        );
    }
}

/// Enforces KISS-OPS-6.5-0003 — `select`'s `cond` test is the IEEE compare
/// `cond != 0`: `-0.0` compares equal to zero (FALSE, choose `b`) and any NaN
/// (quiet or signaling) is non-zero (TRUE, choose `a`). Catches a `cond` test
/// lowered as an integer bit-nonzero check (`cond.to_bits() != 0`), which treats
/// `-0.0` (bits 0x8000_0000) as true, and a `cond > 0` test, which treats NaN as
/// false.
#[test]
fn test_ops_select_cond_zero_nan() {
    // -0.0 compares EQUAL to zero => FALSE => choose b.
    assert!(bits_eq(select(-0.0, 10.0, 20.0), 20.0));
    // +0.0 is likewise false.
    assert!(bits_eq(select(0.0, 10.0, 20.0), 20.0));
    // any NaN is non-zero => TRUE => choose a (quiet ...
    assert!(bits_eq(select(f32::NAN, 10.0, 20.0), 10.0));
    // ... and signaling).
    let snan = f32::from_bits(0x7F80_0001);
    assert!(bits_eq(select(snan, 10.0, 20.0), 10.0));
    // an ordinary nonzero cond is true; a negative nonzero cond is still true.
    assert!(bits_eq(select(1.0, 10.0, 20.0), 10.0));
    assert!(bits_eq(select(-3.0, 10.0, 20.0), 10.0));
}

/// Enforces KISS-OPS-6.18-0004 — `cadd`/`csub`/`cneg`/`cconj` are componentwise,
/// preserving IEEE signed zero per lane; `cconj` flips the SIGN BIT of the
/// imaginary lane so `+0.0 -> -0.0` (and `-0.0 -> +0.0`), a bit flip, not `0 - b`.
/// Catches `cconj`/`cneg` built from `sub(0.0, b)`: `0.0 - (+0.0)` and
/// `0.0 - (-0.0)` both yield `+0.0`, so a `+0.0` imaginary lane never becomes
/// `-0.0` as the clause pins.
#[test]
fn test_ops_complex_add_sub_neg_conj() {
    fn c_bits_eq(a: C32, b: C32) -> bool {
        a[0].to_bits() == b[0].to_bits() && a[1].to_bits() == b[1].to_bits()
    }
    let z = [1.0f32, 2.0];
    let w = [3.0f32, 4.0];
    assert!(c_bits_eq(cadd(z, w), [4.0, 6.0]));
    assert!(c_bits_eq(csub(z, w), [-2.0, -2.0]));
    assert!(c_bits_eq(cneg(z), [-1.0, -2.0]));
    assert!(c_bits_eq(cconj(z), [1.0, -2.0]));
    // cconj flips the sign BIT of the imaginary lane: +0.0 -> -0.0 ...
    assert!(c_bits_eq(cconj([5.0, 0.0]), [5.0, -0.0]));
    // ... and -0.0 -> +0.0 (a bit flip, NOT 0 - b) ...
    assert!(c_bits_eq(cconj([5.0, -0.0]), [5.0, 0.0]));
    // ... while the real lane is a raw move that preserves -0.0.
    assert!(c_bits_eq(cconj([-0.0, 7.0]), [-0.0, -7.0]));
    // cneg flips both lanes' sign bits: (+0.0,+0.0) -> (-0.0,-0.0).
    assert!(c_bits_eq(cneg([0.0, 0.0]), [-0.0, -0.0]));
}

/// Enforces KISS-OPS-6.18-0006 — `cdiv` applies the Annex G recovery with pinned
/// component signs and MUST NOT route the zero-denominator / infinite-denominator
/// cases through the real `div` atom. A nonzero finite `(a,b)` over a complex
/// ZERO denominator yields `copysign(∞,a) + i·copysign(∞,b)`, NOT `0/0 = NaN`;
/// a complex-infinity denominator yields a complex zero, NOT `inf/inf = NaN`.
/// Catches exactly the forbidden real-`div` route: [`cdiv_via_real_div`] yields
/// NaN components where Annex G pins signed infinities / a zero.
#[test]
fn test_ops_cdiv_annexg() {
    fn c_bits_eq(a: C32, b: C32) -> bool {
        a[0].to_bits() == b[0].to_bits() && a[1].to_bits() == b[1].to_bits()
    }
    let inf = f32::INFINITY;
    // nonzero finite (a,b) over (±0,±0): Annex G G.5.1 pins copysign(inf,a) + i*copysign(inf,b).
    assert!(c_bits_eq(cdiv([2.0, 3.0], [0.0, 0.0]), [inf, inf]));
    assert!(c_bits_eq(cdiv([-2.0, 3.0], [0.0, 0.0]), [-inf, inf]));
    // the signs follow the NUMERATOR (a,b), not the denominator's signed zeros.
    assert!(c_bits_eq(cdiv([2.0, -3.0], [-0.0, -0.0]), [inf, -inf]));
    // the forbidden real-div route yields 0/0 = NaN in BOTH lanes -- the exact
    // reason cdiv must not evaluate this case through the real `div` atom.
    let bad_zero = cdiv_via_real_div([2.0, 3.0], [0.0, 0.0]);
    assert!(bad_zero[0].is_nan() && bad_zero[1].is_nan());
    // complex-infinity denominator, finite numerator -> a complex zero (pinned
    // magnitude), where the real-div route would give inf/inf = NaN.
    let q = cdiv([1.0, 1.0], [inf, 4.0]);
    assert!(q[0] == 0.0 && q[1] == 0.0);
    let bad_inf = cdiv_via_real_div([1.0, 1.0], [inf, 4.0]);
    assert!(bad_inf[0].is_nan() && bad_inf[1].is_nan());
    // finite operands with a nonzero denominator: the real decomposition
    // ((ac+bd) + (bc-ad)i) / (c^2+d^2) -- a smoke check the guards don't misfire.
    let f = cdiv([1.0, 2.0], [3.0, 4.0]);
    assert!(c_bits_eq(
        f,
        [(1.0f32 * 3.0 + 2.0 * 4.0) / 25.0, (2.0f32 * 3.0 - 1.0 * 4.0) / 25.0],
    ));
}

/// Enforces KISS-OPS-6.13-0005 — `pow` over its full domain: `pow(0,0)=1`;
/// signed-zero bases with the IEEE odd/even-integer sign rule; and negative bases
/// with an integer exponent giving `|a|^b` (even) or `-(|a|^b)` (odd) rather than
/// NaN. Catches a `pow` that is only the `a>0` reference `exp(b*log(a))`
/// ([`pow_via_exp_log`]): `log(a)` is NaN for `a<0` (so `pow(-2,3)` is NaN, not
/// -8) and `0*log(0)=NaN` (so `pow(0,0)` is NaN, not 1).
#[test]
fn test_ops_pow_full_domain() {
    let inf = f32::INFINITY;
    // pow(0,0) = 1 (and the reference exp(0*log(0)) = NaN -- the forbidden form).
    assert_eq!(pow(0.0, 0.0), 1.0);
    assert!(pow_via_exp_log(0.0, 0.0).is_nan());
    // negative base, integer exponent: sign follows odd/even.
    assert_eq!(pow(-2.0, 3.0), -8.0); // odd -> -(|a|^b)
    assert_eq!(pow(-2.0, 2.0), 4.0);  // even -> |a|^b
    assert!(pow_via_exp_log(-2.0, 3.0).is_nan()); // the forbidden reference is NaN here
    // negative base, NON-integer exponent -> NaN.
    assert!(pow(-2.0, 2.5).is_nan());
    // +0.0 base: +0.0 for b>0, +inf for b<0.
    assert!(bits_eq(pow(0.0, 3.0), 0.0));
    assert_eq!(pow(0.0, -3.0), inf);
    // -0.0 base: sign flips ONLY for an odd-integer exponent.
    assert!(bits_eq(pow(-0.0, 3.0), -0.0));   // positive odd integer -> -0.0
    assert!(bits_eq(pow(-0.0, 2.0), 0.0));    // positive even integer -> +0.0
    assert!(bits_eq(pow(-0.0, 0.5), 0.0));    // positive non-integer   -> +0.0
    assert_eq!(pow(-0.0, -3.0), -inf);         // negative odd integer   -> -inf
    assert_eq!(pow(-0.0, -2.0), inf);          // negative even integer  -> +inf
    assert_eq!(pow(-0.0, -0.5), inf);          // negative non-integer   -> +inf
    // a>0 reference (exact power) still holds.
    assert_eq!(pow(2.0, 10.0), 1024.0);
}

/// Enforces KISS-OPS-6.13-0003 — a Refine-marked exp-of-large-argument op MUST
/// deviate from the literal reference decomposition where that reference would
/// overflow while the true function is finite, yet agree with the reference's
/// pinned meaning within the declared ULP elsewhere. Catches a kernel that emits
/// the literal `tanh` reference `(exp(x)-exp(-x))/(exp(x)+exp(-x))` verbatim: at
/// x=100 `exp(100)` overflows to +inf and `inf/inf = NaN`, where tanh(100)=1.
#[test]
fn test_ops_decomposition_accuracy_refinement() {
    // The literal reference OVERFLOWS to NaN at x=100 (exp(100) > f32::MAX -> +inf,
    // inf/inf = NaN) while the true tanh(100) = 1.
    assert!(tanh_literal_ref(100.0).is_nan());
    // §6.13-0003 REQUIRES the refined (overflow-safe) form here.
    assert_eq!(tanh_refined(100.0), 1.0);
    assert_eq!(tanh_refined(-100.0), -1.0);
    // Yet in the ordinary range the refined form still agrees with the
    // reference's pinned mathematical meaning within the op's declared ULP.
    let bound = 4;
    for &x in &[-1.5f32, -0.5, 0.5, 1.0, 2.0] {
        let r = tanh_refined(x);
        let lit = tanh_literal_ref(x);
        assert!(
            compare_f32(DeterminismClass::UlpTolerance, r, lit, bound).is_ok(),
            "tanh refined vs literal at {x}: {} ULP exceeds {bound}",
            ulp_distance_f32(r, lit)
        );
    }
}

/// Enforces KISS-OPS-6.18-0017 — the split comparator for carg/clog/csqrt/cexp.
/// A plain ULP comparator cannot see a wrong sign of zero (ulp_distance(-0,+0)==1),
/// so it PASSES a +0.0 result where -0.0 is pinned; the split comparator CATCHES
/// it exact-bit. Teeth: this is the divergence between the two comparators.
#[test]
fn test_ops_complex_branch_sign_exact() {
    // A carg-style result with a signed-zero imaginary component: -0.0 is pinned,
    // the candidate returns +0.0 (a real sign-of-zero error).
    let expected = [1.0f32, -0.0];
    let candidate = [1.0f32, 0.0];
    // the plain ULP comparator (bound 2) is BLIND to it: ulp_distance(-0,+0)==1.
    assert_eq!(ulp_distance_f32(-0.0, 0.0), 1);
    assert!(compare_f32(DeterminismClass::UlpTolerance, 0.0, -0.0, 2).is_ok(),
        "the plain ULP comparator wrongly accepts the sign-of-zero error");
    // the §6.18-0017 split comparator CATCHES it exact-bit ...
    assert!(compare_c32_transcendental(candidate, expected, 2).is_err(),
        "split comparator must reject a wrong sign of zero");
    // ... while accepting a correct result within ULP tolerance on the real lane.
    let near = [1.0f32 + f32::from_bits(1.0f32.to_bits() + 1) - 1.0, -0.0];
    assert!(compare_c32_transcendental(near, expected, 2).is_ok());
    // A ±pi branch endpoint: +pi pinned, candidate returns -pi (wrong branch) -> caught.
    let pi = std::f32::consts::PI;
    assert!(compare_c32_transcendental([-pi, 0.0], [pi, 0.0], 2).is_err());
    assert!(compare_c32_transcendental([pi, 0.0], [pi, 0.0], 2).is_ok());
}

// ============================================================================
// KISS-OPS §6.2/§6.4/§6.5/§6.6/§6.7/§6.9 scalar primitive-floor clauses (#91).
//
// Twelve remaining UNTESTED scalar clauses, each with a live reference fn in
// `conformance/src/semantics.rs` — no new oracle, no new deps. The teeth are
// behavioral: raw-bit sign moves (abs/copysign/select) vs the arithmetic/branch
// forms that perturb -0.0 and NaN payloads; IEEE signed-zero comparison equality
// vs a bit-compare; NaN propagation of the default atoms vs the pinned
// NaN-suppressing exception; and non-primitive results coming from the §6.13
// decomposition (sign/step(NaN)=0) rather than default propagation.
//
// Every test asserts `to_bits()` / `is_nan()` / predicate truth on the reference
// fn and FAILS under a concrete plausible drift (FTZ -0.0 flush, fast-math NaN
// scrub, raw-bit -> branch, bit-compare equality, mask-multiply, operand-order
// swap, integer-cast rounder). Ranked strongest teeth first. Each cites the one
// clause it binds so `kiss_trace.py --update-ledger` drops exactly these ids.
// ============================================================================

/// Enforces KISS-OPS-6.4-0004 — `abs` clears the sign bit as a RAW-BIT operation:
/// `abs(-0.0)` is `+0.0`, and a NaN's payload survives with only the sign cleared.
/// Catches the branch mutation `abs := if x < 0.0 { -x } else { x }`: `-0.0 < 0.0`
/// is false (IEEE `-0.0 == 0.0`), so that form returns `-0.0` (bits 0x8000_0000)
/// where the clause pins `+0.0`, and it leaves a negative NaN's sign uncleared.
#[test]
fn test_ops_abs_raw_bit() {
    // KISS-OPS-6.4-0004
    assert_eq!(abs(-0.0).to_bits(), 0x0000_0000, "abs(-0.0) clears the sign bit -> +0.0");
    assert_eq!(abs(0.0).to_bits(), 0x0000_0000);
    // a negative payload NaN: ONLY the sign bit is cleared, the payload is kept.
    let neg_nan = f32::from_bits(0xFF80_0001);
    assert!(neg_nan.is_nan());
    assert_eq!(abs(neg_nan).to_bits(), 0x7F80_0001, "abs clears only the sign bit; payload preserved");
    assert_eq!(abs(-3.5).to_bits(), 3.5f32.to_bits());
    // the forbidden branch form leaves -0.0 as -0.0 -- the exact drift this rejects.
    let x = -0.0f32;
    let branch_abs = if x < 0.0 { -x } else { x };
    assert_eq!(branch_abs.to_bits(), 0x8000_0000);
    assert_ne!(abs(-0.0).to_bits(), branch_abs.to_bits());
}

/// Enforces KISS-OPS-6.4-0003 — `neg` flips the sign bit as a RAW-BIT operation:
/// `neg(-0.0)` is `+0.0`, `neg(+0.0)` is `-0.0`, and a NaN's payload survives with only
/// the sign flipped. The sibling of `test_ops_abs_raw_bit`, which this mirrors.
///
/// ⚠️ THE OBVIOUS ASSERTION DOES NOT DISCRIMINATE. Rust's `-x` on `f32` already flips the
/// sign bit, so `assert_eq!(neg(nan).to_bits(), …)` passes against the reference AND
/// against a wrong implementation that happens to use native negation. Each case therefore
/// also measures a PLAUSIBLE WRONG FORM on the same input and asserts it DIVERGES — and
/// each wrong form is shown AGREEING on a neighbouring case first, so the divergence is
/// attributable to the property under test rather than to the form being broken generally.
///
/// The narrow-float arm — promote-to-`f32`, negate, round back, which QUIETS a signaling
/// NaN — is not repeated here: it is covered for `bf16` by
/// `test_ops_bf16_move_ops_preserve_snan_bits` in `corpus_differential.rs` (#393), and the
/// two blocks would otherwise carry the same fixtures.
#[test]
fn test_ops_neg_raw_bit() {
    // KISS-OPS-6.4-0003
    assert_eq!(neg(-0.0).to_bits(), 0x0000_0000, "neg(-0.0) flips the sign bit -> +0.0");
    assert_eq!(neg(0.0).to_bits(), 0x8000_0000, "neg(+0.0) flips the sign bit -> -0.0");
    assert_eq!(neg(-3.5).to_bits(), 3.5f32.to_bits());
    assert_eq!(neg(3.5).to_bits(), (-3.5f32).to_bits());

    // a payload NaN: ONLY the sign bit moves, the payload is kept -- in both directions,
    // so a form that zeroed or canonicalized the payload fails whichever sign it is given.
    let pos_nan = f32::from_bits(0x7F80_0001);
    let neg_nan = f32::from_bits(0xFF80_0001);
    assert!(pos_nan.is_nan() && neg_nan.is_nan());
    assert_eq!(neg(pos_nan).to_bits(), 0xFF80_0001, "neg flips only the sign bit; payload preserved");
    assert_eq!(neg(neg_nan).to_bits(), 0x7F80_0001, "and back, payload still preserved");

    // WRONG FORM 1 -- `0.0 - x`, the classic. IEEE gives `(+0) - (+0) = +0`, so it AGREES
    // on -0.0 and is WRONG on +0.0: it cannot produce a negative zero at all.
    let sub_neg = |x: f32| 0.0f32 - x;
    assert_eq!(sub_neg(-0.0).to_bits(), neg(-0.0).to_bits(), "the `0.0 - x` form agrees on -0.0");
    assert_ne!(
        neg(0.0).to_bits(),
        sub_neg(0.0).to_bits(),
        "the `0.0 - x` form returns +0.0 where the clause pins neg(+0.0) = -0.0"
    );

    // WRONG FORM 2 -- canonicalizing the NaN. Agrees on every finite input and drops the
    // payload, which is exactly the entailment `by flipping the sign bit` carries and which
    // "a NaN operand propagates a NaN" alone would NOT catch: a canonical NaN IS a NaN.
    let canon_neg = |x: f32| if x.is_nan() { f32::NAN } else { -x };
    assert_eq!(canon_neg(-3.5).to_bits(), neg(-3.5).to_bits(), "the canonicalizing form agrees on finites");
    assert_ne!(
        neg(pos_nan).to_bits(),
        canon_neg(pos_nan).to_bits(),
        "canonicalizing drops the NaN payload the clause requires preserved"
    );
}

/// Enforces KISS-OPS-6.5-0002 — `select` moves the chosen arm as a RAW-BIT copy, so
/// a chosen `-0.0` stays `-0.0` and a chosen signaling NaN keeps its payload and its
/// signaling bit. Catches any arithmetic-carrying select (e.g. `cond*a+(1-cond)*b`):
/// `1.0*(-0.0)+0.0*b` canonicalizes to `+0.0` and `1.0*sNaN` quiets the NaN.
#[test]
fn test_ops_select_raw_bit() {
    // KISS-OPS-6.5-0002
    assert_eq!(select(1.0, -0.0, 5.0).to_bits(), 0x8000_0000, "chosen -0.0 preserved bit-for-bit");
    let snan = f32::from_bits(0x7FA0_0001); // signaling NaN (mantissa MSB clear)
    assert!(snan.is_nan());
    assert_eq!(select(1.0, snan, 5.0).to_bits(), 0x7FA0_0001, "chosen sNaN payload + signaling bit preserved");
    // an arithmetic-carrying select canonicalizes the -0.0 away -- the forbidden form:
    let arith = 1.0f32 * (-0.0) + (1.0 - 1.0) * 5.0;
    assert_eq!(arith.to_bits(), 0x0000_0000, "mask-multiply loses the -0.0 sign");
    assert_ne!(select(1.0, -0.0, 5.0).to_bits(), arith.to_bits());
}

/// Enforces KISS-OPS-6.9-0002 — `copysign(a, b)` is the magnitude of `a` with the
/// RAW sign bit of `b`, so the signed zero of `b` and the sign of a NaN `b` carry
/// into the result. Catches the branch mutation `copysign := if b < 0.0 { -|a| }
/// else { |a| }`: for `b = -0.0`, `b < 0.0` is false -> it returns `+1.0` where the
/// clause pins `-1.0`.
#[test]
fn test_ops_copysign_raw_bit() {
    // KISS-OPS-6.9-0002  (semantics::copysign(a = magnitude, b = sign source))
    assert_eq!(copysign(1.0, -0.0).to_bits(), 0xBF80_0000, "sign bit of -0.0 carried -> -1.0");
    assert_eq!(copysign(1.0, 0.0).to_bits(), 0x3F80_0000);
    // the sign bit of a negative NaN `b` is carried into the result.
    let neg_nan = f32::from_bits(0xFFC0_0000);
    assert!(copysign(5.0, neg_nan).is_sign_negative(), "sign of a NaN b is carried");
    // the forbidden branch form returns +1.0 for b = -0.0 (b < 0.0 is false):
    let b = -0.0f32;
    let branch = if b < 0.0 { -(1.0f32) } else { 1.0f32 };
    assert_eq!(branch.to_bits(), 0x3F80_0000);
    assert_ne!(copysign(1.0, -0.0).to_bits(), branch.to_bits());
}

/// Enforces KISS-OPS-6.6-0004 — comparisons honor IEEE signed-zero equality:
/// `cmp_eq(-0.0,+0.0)`, `cmp_le(-0.0,+0.0)`, and `cmp_ge(-0.0,+0.0)` are all true.
/// Catches a raw-bit comparator (`a.to_bits() == b.to_bits()`): -0.0 (0x8000_0000)
/// and +0.0 (0x0000_0000) differ in bits, so it reports NOT-equal where IEEE pins equal.
#[test]
fn test_ops_compare_signed_zero() {
    // KISS-OPS-6.6-0004
    assert!(cmp_eq(-0.0, 0.0), "-0.0 == +0.0 (IEEE numeric equality)");
    assert!(cmp_le(-0.0, 0.0));
    assert!(cmp_ge(-0.0, 0.0));
    // the forbidden raw-bit comparator would call these UNequal -- the bits differ:
    let neg_zero_bits = (-0.0f32).to_bits();
    let pos_zero_bits = (0.0f32).to_bits();
    assert_ne!(neg_zero_bits, pos_zero_bits, "a bit-compare would wrongly see -0.0 != +0.0");
    assert!(cmp_eq(-0.0, 0.0) && (neg_zero_bits != pos_zero_bits),
        "cmp_eq diverges from the forbidden bit-compare on signed zero");
}

/// Enforces KISS-OPS-6.6-0003 — `cmp_ne` is true whenever an operand is NaN, and
/// `cmp_ne(x, x)` serves as the `isnan` predicate. Catches `cmp_ne := a.to_bits()
/// != b.to_bits()`: two same-payload NaNs share bits, so that form reports
/// `cmp_ne(NaN, NaN) == false` where the clause pins true (breaking the isnan floor).
#[test]
fn test_ops_cmp_ne_nan_true() {
    // KISS-OPS-6.6-0003
    let nan = f32::NAN;
    assert!(cmp_ne(nan, 5.0), "cmp_ne is true with a NaN operand");
    assert!(cmp_ne(nan, nan), "cmp_ne(NaN, NaN) is true even with identical bits");
    assert!(cmp_ne(5.0, nan));
    // cmp_ne(x, x) is exactly the isnan predicate -- the concrete truth table:
    let cases: [(f32, bool); 4] = [(nan, true), (1.0, false), (f32::INFINITY, false), (-0.0, false)];
    for (x, expect_nan) in cases {
        assert_eq!(cmp_ne(x, x), expect_nan, "cmp_ne(x,x) is the isnan floor for {x:?}");
        assert_eq!(isnan(x), expect_nan, "isnan agrees for {x:?}");
    }
    assert!(cmp_ne(3.0, 5.0) && !cmp_ne(3.0, 3.0), "ordinary distinct / equal values");
    // the forbidden bit-compare form sees two NaNs as equal -> would call cmp_ne false:
    let (l, r) = (f32::NAN, f32::NAN);
    let bit_compare_ne = l.to_bits() != r.to_bits();
    assert!(!bit_compare_ne, "same-payload NaNs share bits");
    assert!(cmp_ne(l, r) != bit_compare_ne, "cmp_ne diverges from the forbidden bit-compare on NaN,NaN");
}

/// Enforces KISS-OPS-6.7-0002 — every rounding atom propagates a NaN operand and
/// preserves the sign of a zero (`trunc(-0.0)=-0.0`, likewise floor/ceil/round_even).
/// Catches an integer-cast rounder `(x as i64) as f32`: it maps `-0.0` to `+0.0`
/// and NaN to `0`, dropping both the sign of zero and the NaN.
#[test]
fn test_ops_rounding_nan_signed_zero() {
    // KISS-OPS-6.7-0002
    let nan = f32::NAN;
    assert!(
        floor(nan).is_nan() && ceil(nan).is_nan() && trunc(nan).is_nan() && round_even(nan).is_nan(),
        "rounding atoms propagate NaN"
    );
    assert_eq!(floor(-0.0).to_bits(), 0x8000_0000, "floor(-0.0) = -0.0");
    assert_eq!(ceil(-0.0).to_bits(), 0x8000_0000, "ceil(-0.0) = -0.0");
    assert_eq!(trunc(-0.0).to_bits(), 0x8000_0000, "trunc(-0.0) = -0.0");
    assert_eq!(round_even(-0.0).to_bits(), 0x8000_0000, "round_even(-0.0) = -0.0");
    // the forbidden integer-cast rounder loses BOTH properties:
    let cast_round = |x: f32| (x as i64) as f32;
    assert_eq!(cast_round(-0.0).to_bits(), 0x0000_0000, "int-cast flushes -0.0 to +0.0");
    assert!(!cast_round(nan).is_nan(), "int-cast drops the NaN");
    assert_ne!(trunc(-0.0).to_bits(), cast_round(-0.0).to_bits());
}

/// Enforces KISS-OPS-6.5-0004 — `select` MUST NOT be rewritten to a mask-multiply
/// `cond*a + (1-cond)*b`, which perturbs signed zero. For cond=1, a=-0.0, b=+0.0 the
/// raw-bit `select` yields -0.0 while the mask form yields +0.0. The forbidden form
/// is computed inline as a foil (like naive_max_x_zero / cdiv_via_real_div) and the
/// test asserts `select` DIVERGES from it: a select lowered to mask-multiply FAILS.
#[test]
fn test_ops_select_no_mask_multiply() {
    // KISS-OPS-6.5-0004
    let (cond, a, b) = (1.0f32, -0.0f32, 0.0f32);
    let mask = cond * a + (1.0 - cond) * b; // the forbidden mask-multiply form
    assert_eq!(select(cond, a, b).to_bits(), 0x8000_0000, "select keeps the chosen -0.0");
    assert_eq!(mask.to_bits(), 0x0000_0000, "mask-multiply normalizes it to +0.0");
    assert_ne!(
        select(cond, a, b).to_bits(),
        mask.to_bits(),
        "a select rewritten to mask-multiply would fail this divergence"
    );
}

/// Enforces KISS-OPS-6.2-0009 — a non-primitive op's edge behavior comes from its
/// §6.13 reference decomposition, NOT the default §6.2-0003 atom NaN-propagation:
/// `sign(NaN)=0` and `step(NaN)=0` because `cmp_gt`/`cmp_lt(NaN,_)` are both false in
/// the select-decomposition. Catches a naive `sign := x/|x|`, which propagates NaN
/// (`sign(NaN)=NaN`) instead of the decomposition's `0`.
#[test]
fn test_ops_nonprimitive_semantics_from_decomposition() {
    // KISS-OPS-6.2-0009
    let nan = f32::NAN;
    // decomposition-derived edge results (NOT default NaN propagation):
    assert_eq!(sign(nan), 0.0, "sign(NaN)=0 from the select-decomposition");
    assert_eq!(step(nan), 0.0, "step(NaN)=0 from the select-decomposition");
    // ordinary behavior still holds.
    assert_eq!(sign(2.0), 1.0);
    assert_eq!(sign(-3.0), -1.0);
    assert_eq!(step(2.0), 1.0);
    assert_eq!(step(-1.0), 0.0);
    // the forbidden default-propagation form `x/|x|` yields NaN, diverging from the
    // decomposition's pinned 0 -- proving the edge is decomposition-derived:
    let naive_sign = nan / abs(nan);
    assert!(naive_sign.is_nan(), "default atom propagation would give sign(NaN)=NaN");
    assert!(sign(nan) != naive_sign, "sign's decomposition overrides default propagation");
}

/// Enforces KISS-OPS-6.5-0001 — `select` takes operands `(cond, a, b)` and yields
/// `a` when `cond != 0` and `b` when `cond == 0`. Catches an operand-order swap
/// (returning `b` on `cond != 0`) and a flipped truth sense: with distinct a, b the
/// wrong arm is observable.
#[test]
fn test_ops_select_order() {
    // KISS-OPS-6.5-0001
    assert_eq!(select(1.0, 10.0, 20.0), 10.0, "cond != 0 selects a");
    assert_eq!(select(5.0, 10.0, 20.0), 10.0, "any nonzero cond selects a");
    assert_eq!(select(0.0, 10.0, 20.0), 20.0, "cond == 0 selects b");
}

/// Enforces KISS-OPS-6.6-0001 — each comparison computes its §6.6 table predicate.
/// Catches an off-by-one strictness drift on the boundary-equal case: `cmp_lt := <=`
/// flips `cmp_lt(1,1)` to true; `cmp_gt := >=` flips `cmp_gt(1,1)` to true; `cmp_ge
/// := >` flips `cmp_ge(1,1)` to false; `cmp_le := <` flips `cmp_le(1,1)` to false.
#[test]
fn test_ops_compare_predicates() {
    // KISS-OPS-6.6-0001
    // strict/non-strict on the boundary-EQUAL case pins the strictness of each op:
    assert!(cmp_lt(1.0, 2.0) && !cmp_lt(2.0, 1.0) && !cmp_lt(1.0, 1.0), "cmp_lt is strict");
    assert!(cmp_gt(2.0, 1.0) && !cmp_gt(1.0, 2.0) && !cmp_gt(1.0, 1.0), "cmp_gt is strict");
    assert!(cmp_le(1.0, 1.0) && cmp_le(1.0, 2.0) && !cmp_le(2.0, 1.0), "cmp_le is non-strict");
    assert!(cmp_ge(1.0, 1.0) && cmp_ge(2.0, 1.0) && !cmp_ge(1.0, 2.0), "cmp_ge is non-strict");
    // eq / ne are complementary on ordinary values:
    assert!(cmp_eq(1.0, 1.0) && !cmp_eq(1.0, 2.0));
    assert!(cmp_ne(1.0, 2.0) && !cmp_ne(1.0, 1.0));
}

/// Enforces KISS-OPS-6.2-0004 — signed zero is preserved except at the two pinned
/// clears (`neg`/`abs` of -0.0): `add(-0.0,-0.0)` stays -0.0, `mul(-0.0,1.0)` stays
/// -0.0, `sub(0.0,0.0)` is +0.0. Catches a flush-to-zero (FTZ/DAZ) arithmetic that
/// emits +0.0 for `add(-0.0,-0.0)`. The comparison MUST be raw bits (`0.0 == -0.0`).
#[test]
fn test_ops_signed_zero_preserved() {
    // KISS-OPS-6.2-0004
    assert_eq!(add(-0.0, -0.0).to_bits(), 0x8000_0000, "-0.0 + -0.0 stays -0.0 (no FTZ)");
    assert_eq!(mul(-0.0, 1.0).to_bits(), 0x8000_0000, "-0.0 * 1.0 stays -0.0");
    assert_eq!(sub(0.0, 0.0).to_bits(), 0x0000_0000, "0.0 - 0.0 is +0.0");
    // the two pinned exceptions explicitly CLEAR the sign:
    assert_eq!(neg(-0.0).to_bits(), 0x0000_0000, "neg(-0.0) = +0.0 (pinned clear)");
    assert_eq!(abs(-0.0).to_bits(), 0x0000_0000, "abs(-0.0) = +0.0 (pinned clear)");
}

/// Enforces KISS-OPS-6.2-0003 — default primitive-floor atoms propagate a NaN
/// operand, IN CONTRAST to the §6.15 NaN-suppressing `fmax_ieee`. Catches a
/// `-ffinite-math-only` / fast-math atom that assumes no-NaN and drops the NaN
/// operand. Lowest-drift of the set but still fails a real finite-math build.
#[test]
fn test_ops_default_nan_propagation() {
    // KISS-OPS-6.2-0003
    let nan = f32::NAN;
    assert!(add(nan, 5.0).is_nan());
    assert!(sub(nan, 5.0).is_nan());
    assert!(mul(nan, 0.0).is_nan(), "mul(NaN, 0.0) is NaN, not 0");
    assert!(div(nan, 5.0).is_nan());
    assert!(floor(nan).is_nan());
    // the pinned §6.15 exception goes the OTHER way (NaN-suppressing): the default
    // rule is not universal, which is exactly what §6.2-0003 scopes.
    assert_eq!(fmax_ieee(nan, 5.0), 5.0);
    // a finite-math atom that assumes no NaN would scrub it -- the drift this rejects:
    let finite_math_add = |a: f32, b: f32| if a.is_nan() { b } else if b.is_nan() { a } else { a + b };
    assert_eq!(finite_math_add(nan, 5.0), 5.0, "a fast-math atom drops the NaN operand");
    assert!(
        add(nan, 5.0).is_nan() && finite_math_add(nan, 5.0) == 5.0,
        "the default atom propagates where the fast-math form scrubs"
    );
}
