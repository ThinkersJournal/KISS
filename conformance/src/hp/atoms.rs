//! Plan B Slice-0 T5/T6 — transcendental ATOMS (exp/log/sin) over the wide
//! `BigFloat` core.
//!
//! Each atom implements [`Atom::eval`] at generic width `N`: reduce the argument
//! (T4), evaluate the reduced-range series, fold a RIGOROUS `err_ulps` from the
//! reduction error + the per-op rounding + the truncation tail, then apply the
//! exact octave / `ldexp` reconstruction. The Ziv driver
//! ([`super::round_atom_to_f64`]) re-derives at 256→512→1024 until `D > err_ulps`.
//!
//! `err_ulps` is a ULP-COUNT at the value's LSB exponent (`exp2()`); sub-term
//! errors are folded onto the running result ULP with [`rescale_ulps`], which
//! rounds UP, so every bound here is a rigorous over-estimate.

use super::reduction::{self, rescale_ulps};
use super::{consts, Atom, BigFloat, Evaluated};

/// Precision-floor guard (bits below the working ULP a term must fall before the
/// series is truncated). The tail past that term is `< term·|r|/(1−|r|) < term`.
const SERIES_GUARD: i32 = 8;

/// `exp(x)` for a finite `x` inside the atom's reduced domain. Large-|x| pre-clamp
/// to `±Inf`/`±0` and the `NaN` front door are the semantics layer's job (T5,
/// `semantics.rs`); this atom assumes `reduce_exp`'s `|k|` precondition holds.
#[derive(Clone, Copy, Debug)]
pub struct Exp {
    pub x: f64,
}

impl Atom for Exp {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        // x = k·ln2 + r, |r| ≤ ln2/2 ≈ 0.347, so exp(x) = 2^k · exp(r).
        let red = reduction::reduce_exp::<N>(self.x);
        let r = &red.r;

        // Maclaurin: exp(r) = Σ_{n≥0} r^n/n!. Sum until a term's MSB (`ebin`) falls
        // GUARD bits below s's LSB (`exp2`), i.e. the term is fully sub-precision;
        // the omitted tail is then strictly smaller still (ratio |r|/(1−|r|) < 0.53
        // < 1), so it cannot perturb s at width N.
        let mut s = BigFloat::<N>::one();
        let mut term = BigFloat::<N>::one();
        // Running Σ of per-op rounding, kept in ULPs at `acc_exp` (= current
        // s.exp2()); rescaled whenever s.exp2() drifts (s ≈ exp(r) ⇒ ≤ 1 bit).
        let mut op_err: u128 = 0;
        let mut acc_exp = s.exp2();
        let mut n: i64 = 1;
        loop {
            let (t_mul, e_mul) = term.mul(r); //   term·r
            let (t_div, e_div) = t_mul.div_small(n as u64); //   /n
            term = t_div;
            let (s_new, e_add) = s.add(&term);

            if s_new.exp2() != acc_exp {
                op_err = rescale_ulps(op_err, acc_exp, s_new.exp2());
                acc_exp = s_new.exp2();
            }
            op_err = op_err
                .saturating_add(rescale_ulps(e_mul, t_mul.exp2(), acc_exp))
                .saturating_add(rescale_ulps(e_div, term.exp2(), acc_exp))
                .saturating_add(e_add); // already at s_new.exp2() == acc_exp
            s = s_new;

            if term.is_zero() || term.ebin() < s.exp2() - SERIES_GUARD {
                break;
            }
            n += 1;
        }

        // Truncation tail < one floor-ULP ⇒ bound as 1 ULP at s.exp2().
        let trunc_err: u128 = 1;

        // Reduction error δ = red.err_ulps ULPs at r.exp2() propagates through exp:
        // |exp(r+δ) − exp(r)| = exp(r)·|e^δ − 1| ≤ 2·exp(r)·|δ| < 4·|δ|
        // (exp(r) < 1.42, δ tiny). Rescale 4·red.err_ulps from r.exp2() to s.exp2().
        let red_err = rescale_ulps(4u128.saturating_mul(red.err_ulps), r.exp2(), s.exp2());

        // exp(x) = 2^k · exp(r): ldexp is EXACT — it shifts the value and its ULP
        // together, so the ULP-COUNT `err_ulps` is unchanged.
        let val = s.ldexp(red.k as i32);
        let err_ulps = op_err.saturating_add(trunc_err).saturating_add(red_err);
        Evaluated { val, err_ulps }
    }
}

/// `log(x)` (natural log) for a positive finite `x`. Domain routing (x ≤ 0 → NaN,
/// x = +0 → −Inf, +Inf → +Inf) is the semantics front door's job (T5).
#[derive(Clone, Copy, Debug)]
pub struct Log {
    pub x: f64,
}

impl Atom for Log {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        // x = 2^e · m, m ∈ [√2/2, √2) EXACT ⇒ log(x) = e·ln2 + log(m).
        let red = reduction::reduce_log::<N>(self.x);
        let one = BigFloat::<N>::one();

        // log(m) = 2·atanh(t), t = (m−1)/(m+1), |t| ≤ 0.172. m is EXACT, so t's
        // only errors are the sub/add rounding + the div. Bound the propagation
        // rigorously: |dt| ≤ |dnum|/|den| + |num|·|dden|/|den|² with |den| ≥ 1.7,
        // |num| ≤ 0.5 ⇒ each ≤ its own rescaled ULP-count (round-up).
        let (num, e_num) = red.m.sub(&one);
        let (den, e_den) = red.m.add(&one);
        let (t, e_t) = num.div(&den);

        // m == 1 exactly (x is an exact power of two) ⇒ t == 0 ⇒ log(m) = 0 with NO
        // series error. Short-circuit here: a zero-valued t/num carries a degenerate
        // exponent that would drive rescale_ulps past its 128-bit shift and return
        // u128::MAX — a VACUOUS bound (found by the err_ulps soundness test on log(2)).
        let (sum, sum_err) = if t.is_zero() {
            (t.clone(), 0u128)
        } else {
            // t's only errors are the sub/add rounding + the div. Bound the
            // propagation rigorously: |dt| ≤ |dnum|/|den| + |num|·|dden|/|den|² with
            // |den| ≥ 1.7, |num| ≤ 0.5 ⇒ each ≤ its own rescaled ULP-count (round-up).
            let err_t = e_t
                .saturating_add(rescale_ulps(e_num, num.exp2(), t.exp2()))
                .saturating_add(rescale_ulps(e_den, den.exp2(), t.exp2()));
            let (t2, _e_t2) = t.mul(&t); // t²; its rounding is folded via the series ops

            // Σ_{j≥0} t^(2j+1)/(2j+1) = atanh(t). Truncate when a term's MSB falls
            // GUARD bits below the sum's LSB (sub-precision; the tail is smaller still).
            let mut sum = t.clone();
            let mut pow = t.clone(); // t^(2j+1)
            let mut op_err: u128 = 0;
            let mut acc_exp = sum.exp2();
            let mut j: i64 = 1;
            loop {
                let (p_new, e_pm) = pow.mul(&t2); // t^(2j+1)
                pow = p_new;
                let (term, e_td) = pow.div_small((2 * j + 1) as u64);
                let (sum_new, e_sa) = sum.add(&term);
                if sum_new.exp2() != acc_exp {
                    op_err = rescale_ulps(op_err, acc_exp, sum_new.exp2());
                    acc_exp = sum_new.exp2();
                }
                op_err = op_err
                    .saturating_add(rescale_ulps(e_pm, pow.exp2(), acc_exp))
                    .saturating_add(rescale_ulps(e_td, term.exp2(), acc_exp))
                    .saturating_add(e_sa);
                sum = sum_new;
                if term.is_zero() || term.ebin() < sum.exp2() - SERIES_GUARD {
                    break;
                }
                j += 1;
            }
            // atanh derivative 1/(1−t²) ≤ 1.04 propagates err_t onto the sum; factor 2
            // covers it. Plus the truncation tail (1 sub-precision ULP).
            let sum_err = op_err
                .saturating_add(rescale_ulps(2u128.saturating_mul(err_t), t.exp2(), sum.exp2()))
                .saturating_add(1);
            (sum, sum_err)
        };

        // log(m) = 2·sum: ldexp is EXACT (ULP-count preserved).
        let logm = sum.ldexp(1);

        if red.e == 0 {
            // x ∈ [√2/2, √2): no octave term, val = log(m) directly.
            return Evaluated { val: logm, err_ulps: sum_err };
        }

        // val = e·ln2 + log(m).
        let (l2, err_l2) = consts::ln2::<N>();
        let (kl2, e_kl2) = l2.mul_small_int(red.e as i64);
        let (val, e_final) = kl2.add(&logm);
        // kl2's error: the mul rounding (e_kl2 @ kl2.exp2()) + the ln2 truncation
        // (|e|·err_l2 @ l2.exp2()), both rescaled to val.exp2(); plus log(m)'s
        // error and the final add, all onto val.exp2().
        let kl2_err = rescale_ulps(e_kl2, kl2.exp2(), val.exp2()).saturating_add(rescale_ulps(
            (red.e.unsigned_abs() as u128).saturating_mul(err_l2),
            l2.exp2(),
            val.exp2(),
        ));
        let err_ulps = kl2_err
            .saturating_add(rescale_ulps(sum_err, logm.exp2(), val.exp2()))
            .saturating_add(e_final);
        Evaluated { val, err_ulps }
    }
}

/// Maclaurin `sin(r)` (`cos = false`) or `cos(r)` (`cos = true`) on |r| ≤ π/4.
/// Terms are carried SIGNED (`term_j = term_{j-1}·(−r²)/den_j`), so a negative r
/// is handled directly. Returns the value and its op-rounding `err_ulps` at the
/// result's ULP (including the sub-precision truncation tail).
fn maclaurin_trig<const N: usize>(r: &BigFloat<N>, cos: bool) -> (BigFloat<N>, u128) {
    let (r2, _e_r2) = r.mul(r);
    let neg_r2 = r2.neg();
    let (mut sum, mut term) = if cos {
        (BigFloat::<N>::one(), BigFloat::<N>::one())
    } else {
        (r.clone(), r.clone())
    };
    let mut op_err: u128 = 0;
    let mut acc_exp = sum.exp2();
    let mut j: i64 = 1;
    loop {
        // divisor: cos → (2j−1)(2j); sin → (2j)(2j+1).
        let d = if cos { (2 * j - 1) * (2 * j) } else { (2 * j) * (2 * j + 1) };
        let (t1, e1) = term.mul(&neg_r2);
        let (t2, e2) = t1.div_small(d as u64);
        term = t2;
        let (s_new, e3) = sum.add(&term);
        if s_new.exp2() != acc_exp {
            op_err = rescale_ulps(op_err, acc_exp, s_new.exp2());
            acc_exp = s_new.exp2();
        }
        op_err = op_err
            .saturating_add(rescale_ulps(e1, t1.exp2(), acc_exp))
            .saturating_add(rescale_ulps(e2, term.exp2(), acc_exp))
            .saturating_add(e3);
        sum = s_new;
        if term.is_zero() || term.ebin() < sum.exp2() - SERIES_GUARD {
            break;
        }
        j += 1;
    }
    (sum, op_err.saturating_add(1))
}

/// `sin(x)` for finite `x`. `NaN`/`±Inf` → `NaN` is the semantics front door (T6).
#[derive(Clone, Copy, Debug)]
pub struct Sin {
    pub x: f64,
}

impl Atom for Sin {
    fn eval<const N: usize>(&self) -> Evaluated<N> {
        // reduce_trig works on |x|: octant = round(|x|·2/π) mod 8, r ∈ [−π/4, π/4].
        let red = reduction::reduce_trig::<N>(self.x);
        let q = red.octant % 4;
        // sin(oct·π/2 + r): q=0 → sin r, 1 → cos r, 2 → −sin r, 3 → −cos r.
        let use_cos = q == 1 || q == 3;
        let (base, base_err) = maclaurin_trig::<N>(&red.r, use_cos);
        // octant sign (q ∈ {2,3}) XOR sign(x) (sin is odd). `neg` preserves exp2,
        // so `base_err` stays valid at `val.exp2()` without rescaling.
        let negate = (q == 2 || q == 3) ^ self.x.is_sign_negative();
        let val = if negate { base.neg() } else { base };
        // reduction error δ on r: |d sin r| = |cos r|·|δ| ≤ |δ|, |d cos r| =
        // |sin r|·|δ| ≤ 0.71·|δ|. Factor 2 covers both; rescale to val.exp2().
        let red_err =
            rescale_ulps(2u128.saturating_mul(red.err_ulps), red.r.exp2(), val.exp2());
        Evaluated { val, err_ulps: base_err.saturating_add(red_err) }
    }
}
