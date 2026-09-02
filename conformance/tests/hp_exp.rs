//! Plan B Slice-0 Task T5 — `exp` atom vectors (f64 & f32 correctly-rounded).
//!
//! The `Exp` atom composes `reduce_exp` (T4: x = k·ln2 + r, |r| ≤ ln2/2) with a
//! Maclaurin series for `exp(r)`, folds the reduction + truncation + rounding
//! error into `err_ulps`, and applies the exact `ldexp(k)`. `round_atom_to_f64/f32`
//! then runs the Ziv escalation (256→512→1024) to correctly-rounded target bits.
//!
//! ANCHORS below are correctly-rounded-to-nearest, cross-validated by THREE
//! independent oracles — MPFR (gmpy2 2.3.1), Arb (python-flint 0.9.0), and mpmath
//! 1.4.1 — which agree bit-for-bit (a disagreement would be a finding, not noise).
//! Regenerate/verify with `tools/validate_corpus.py` (T13) or the exp-anchor
//! generator in the T5 workflow notes.
//!
//! Each row: (x, correctly-rounded exp(x) as f64 bits, as f32 bits).

use kiss_conformance::hp::{round_atom_to_f32, round_atom_to_f64, Exp};

/// (x, f64 bits, f32 bits) — oracle-agreed (MPFR ≡ Arb ≡ mpmath).
const EXP_ANCHORS: &[(f64, u64, u32)] = &[
    (0.0, 0x3FF0000000000000, 0x3F800000),               // exp(0) = 1 exactly
    (1.0, 0x4005BF0A8B145769, 0x402DF854),               // e
    (-1.0, 0x3FD78B56362CEF38, 0x3EBC5AB2),              // 1/e
    (0.5, 0x3FFA61298E1E069C, 0x3FD3094C),
    (-0.5, 0x3FE368B2FC6F960A, 0x3F1B4598),
    (0.6931471805599453, 0x4000000000000000, 0x40000000), // exp(ln2) ≈ 2
    (2.0, 0x401D8E64B8D4DDAE, 0x40EC7326),
    (-2.0, 0x3FC152AAA3BF81CC, 0x3E0A9555),
    (10.0, 0x40D5829DCF950560, 0x46AC14EE),
    (-10.0, 0x3F07CD79B5647C9B, 0x383E6BCE),
    (88.0, 0x47DF1056DC7BF22D, 0x7EF882B7),               // f32 near-overflow, still finite
    (-87.0, 0x381666D0DAD2961D, 0x00B33687),              // f32 subnormal result
];

#[test]
fn exp_f64_matches_oracle_anchors() {
    for &(x, want64, _want32) in EXP_ANCHORS {
        let got = round_atom_to_f64(&Exp { x }).bits;
        assert_eq!(
            got, want64,
            "exp({x}) f64: got 0x{got:016X}, want 0x{want64:016X} (oracle: MPFR≡Arb≡mpmath)"
        );
    }
}

#[test]
fn exp_f32_matches_oracle_anchors() {
    for &(x, _want64, want32) in EXP_ANCHORS {
        let got = round_atom_to_f32(&Exp { x }).bits as u32;
        assert_eq!(
            got, want32,
            "exp({x}) f32: got 0x{got:08X}, want 0x{want32:08X} (oracle: MPFR≡Arb≡mpmath)"
        );
    }
}
