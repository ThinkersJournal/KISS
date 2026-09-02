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
use super::{Atom, BigFloat, Evaluated};

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
