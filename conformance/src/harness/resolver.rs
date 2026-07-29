//! Decomposition-resolver: evaluates a non-primitive op's KISS-Ops reference
//! decomposition down to the primitive floor (KISS-CONFORM-6.5-0004). Increment 2
//! is focused on `reduce_mean = div(reduce(sum, x), reduced_count)` (ops.md §6.13).

use crate::structural::{reassoc_bound_f32, reduce_f32, Monoid};

/// The `reduce_mean` oracle as its reference decomposition, evaluated at the floor:
/// `div(reduce(sum, x), count)`. It composes the floor `reduce_f32(_, Sum)` (a fold
/// of the `add` primitive) with a divide — it never terminates above the floor and
/// is not a monolithic mean.
pub fn reduce_mean_oracle(xs: &[f32]) -> f32 {
    reduce_f32(xs, Monoid::Sum) / xs.len() as f32
}

/// The absolute tolerance for comparing a candidate `reduce_mean` against the oracle:
/// the reassociation band of the interior sum (`reassoc_bound_f32(n, Σ|x|)`) divided
/// by `count`, since the mean divides the sum by `n`. An absolute band (§6.8-0004 /
/// the #92 accumulator-tolerance model), not a ULP count.
pub fn reduce_mean_abs_tol(xs: &[f32]) -> f32 {
    let sum_abs = xs.iter().map(|x| x.abs()).sum::<f32>();
    reassoc_bound_f32(xs.len(), sum_abs) / xs.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    // KISS-CONFORM-6.5-0004: the oracle for the non-primitive `reduce_mean` MUST be
    // its reference decomposition div(reduce(sum,x), count) evaluated at the floor —
    // NOT a monolithic mean. Teeth: on an input where a running-average mean
    // (m += (x-m)/(i+1)) diverges from sum/n, the resolver matches sum/n, and it is
    // built from the floor `reduce_f32(_, Sum)` + a divide (a wrong monoid or wrong
    // divisor would change the value on the checked input).
    #[test]
    fn test_conform_oracle_resolves_to_floor() {
        let xs = [1.0f32, 2.0, 3.0, 4.0];
        // decomposition value: sum then divide.
        let expected = crate::structural::reduce_f32(&xs, crate::structural::Monoid::Sum) / xs.len() as f32;
        assert_eq!(reduce_mean_oracle(&xs), expected);

        // A running-average mean diverges from sum/n in FP; the resolver must be the
        // sum/n decomposition, not that alternative.
        let mut running = 0.0f32;
        for (i, &x) in xs.iter().enumerate() {
            running += (x - running) / (i as f32 + 1.0);
        }
        // NOTE(deviation from brief): the brief's literal `[1e7, 1.0, 1.0, 1.0, 1.0]`
        // does NOT actually diverge in f32 (verified: both round to bit-identical
        // 0x49f42406) — the coarse ULP at that magnitude swallows the tiny difference
        // between the two algorithms' rounding paths. Substituted with a
        // brute-force-verified genuinely-divergent input (dec=0x4b3ebc23 vs
        // running=0x4b3ebc22) so this assertion has real teeth.
        let big = [1e8f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]; // sum/n and running-avg differ here
        let mut r2 = 0.0f32;
        for (i, &x) in big.iter().enumerate() { r2 += (x - r2) / (i as f32 + 1.0); }
        let dec = crate::structural::reduce_f32(&big, crate::structural::Monoid::Sum) / big.len() as f32;
        assert_eq!(reduce_mean_oracle(&big), dec);
        assert_ne!(reduce_mean_oracle(&big).to_bits(), r2.to_bits(),
            "the resolver must be the sum/n decomposition, distinguishable from a running-average mean");
        let _ = running;
    }

    #[test]
    fn abs_tol_is_the_reassociation_band_over_n() {
        let xs = [1.0f32, -2.0, 3.0, -4.0, 5.0];
        let sum_abs = xs.iter().map(|x| x.abs()).sum::<f32>();
        let expected = crate::structural::reassoc_bound_f32(xs.len(), sum_abs) / xs.len() as f32;
        assert_eq!(reduce_mean_abs_tol(&xs), expected);
    }
}
