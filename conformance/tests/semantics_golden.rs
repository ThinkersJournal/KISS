//! KISS-Conform differential/semantic vectors for the pinned floating-point
//! primitives (KISS-Ops §2.3 / §6.15). These are the numeric distinctions the
//! spec spends prose pinning — now executable: NaN-propagating vs NaN-suppressing
//! min/max, `relu` ≠ `max(x,0)`, and the §6.8 "declared-ULP, not bit-identity"
//! transcendental model. Determinism-class comparators from Conform §6.8.

use kiss_conformance::semantics::*;
use kiss_conformance::{compare_f32, ulp_distance_f32, DeterminismClass};

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
