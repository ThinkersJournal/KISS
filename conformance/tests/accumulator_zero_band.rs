// SPDX-License-Identifier: MIT OR Apache-2.0
//! The zero band of the accumulator tolerance (KISS#413, ROADMAP R1.16): when the
//! `|max partial sum|` of the ascending reference evaluation is 0, is a zero band vacuous?
//!
//! Claim under test (a SUM in IEEE arithmetic, no flush-to-zero): if every ascending
//! running partial is 0, every input is 0, so every schedule also yields 0 and an exact-
//! equality requirement on the zero band loses nothing. The argument is induction
//! (`p1 = x1`; `p_i = fl(0 + x_i) = x_i`); this file runs it, including subnormal inputs.
//!
//! Scope: hardware `f32` and `f64` only (the two accumulator dtypes Rust can run here); it
//! says nothing about an accumulator narrower than `f32`, nor about a product-fold
//! contraction whose per-term products underflow.
//!
//! The control is the same search with flush-to-zero emulated: the claim is FALSE there
//! (a lone subnormal input is flushed, every partial is 0, the input is not), so the
//! search can go red, and the clause's "without flush-to-zero" qualifier is load-bearing.

macro_rules! zero_band_suite {
    ($modname:ident, $t:ty, $bits:ty, $pool:expr) => {
        mod $modname {
            const MIN_NORMAL_EXP: $bits = (1 as $bits) << (<$t>::MANTISSA_DIGITS - 1);

            fn flush(x: $t, ftz: bool) -> $t {
                // A subnormal has a zero exponent field and a nonzero mantissa.
                if ftz && x != 0.0 && x.abs() < <$t>::MIN_POSITIVE { 0.0 * x.signum() } else { x }
            }

            fn pool() -> Vec<$t> {
                let sub_max = <$t>::from_bits(MIN_NORMAL_EXP - 1);
                let mut v: Vec<$t> = vec![0.0, -0.0, 1.0, -1.0, 3.0, <$t>::MIN_POSITIVE, -<$t>::MIN_POSITIVE,
                    <$t>::from_bits(1), -<$t>::from_bits(1), sub_max, -sub_max, <$t>::MAX, -<$t>::MAX];
                v.extend_from_slice(&$pool);
                v
            }

            /// Every sequence of length 1..=4 over the pool whose ascending partials are all 0
            /// and which has a nonzero input. Empty means the claim holds.
            fn violations(ftz: bool) -> Vec<Vec<$t>> {
                let p = pool();
                let mut bad = Vec::new();
                let mut stack: Vec<Vec<$t>> = p.iter().map(|&x| vec![x]).collect();
                while let Some(seq) = stack.pop() {
                    let mut acc: $t = 0.0;
                    let mut all_zero = true;
                    for &x in &seq {
                        acc = flush(flush(acc, ftz) + flush(x, ftz), ftz);
                        all_zero &= acc == 0.0;
                    }
                    if all_zero && seq.iter().any(|&x| x != 0.0) {
                        bad.push(seq.clone());
                    }
                    if seq.len() < 4 {
                        for &x in &p {
                            let mut s = seq.clone();
                            s.push(x);
                            stack.push(s);
                        }
                    }
                }
                bad
            }

            // harness-internal: runs the zero-band argument of the accumulator tolerance as a property over a value pool; the clause is only partly covered, so none is cited
            #[test]
            fn zero_partials_imply_zero_inputs_without_ftz() {
                let bad = violations(false);
                assert!(bad.is_empty(), "{} counterexamples, first {:?}", bad.len(), bad.first());
            }

            // harness-internal: control showing the property search can go red under flush-to-zero emulation, no clause asserted
            #[test]
            fn control_flush_to_zero_breaks_the_claim() {
                let bad = violations(true);
                assert!(
                    bad.iter().any(|s| s.len() == 1),
                    "the control must find a lone flushed subnormal; found {} counterexamples",
                    bad.len()
                );
            }
        }
    };
}

zero_band_suite!(f32_sum, f32, u32, [1.0e-30_f32, -1.0e-30_f32, 8.0e-45_f32]);
zero_band_suite!(f64_sum, f64, u64, [1.0e-300_f64, -1.0e-300_f64, 5.0e-324_f64]);
