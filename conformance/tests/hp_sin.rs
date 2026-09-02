//! Plan B Slice-0 Task T6 — `sin` atom vectors, correctly-rounded f64/f32.
//!
//! The `Sin` atom composes `reduce_trig` (T4: Payne–Hanek, octant = round(|x|·2/π)
//! mod 8, r ∈ [−π/4, π/4]) with a Maclaurin series for sin/cos on the reduced
//! range, reconstructs by octant (q = octant mod 4 → {sin, cos, −sin, −cos}(r)),
//! and applies sin's oddness in `x`. `err_ulps` folds the reduction error (which
//! dominates the tiny-cancellation cases like sin(π)) + the series rounding.
//!
//! ANCHORS are correctly-rounded-to-nearest, cross-validated MPFR (gmpy2) ≡ Arb
//! (python-flint) ≡ mpmath — bit-for-bit, including the tiny sin(π) ≈ 1.2e-16 and
//! sin(2π) that exercise the wide 2/π table.

use kiss_conformance::hp::{round_atom_to_f32, round_atom_to_f64, Sin};

/// (x, f64 bits, f32 bits) of sin(x) — oracle-agreed.
const SIN_ANCHORS: &[(f64, u64, u32)] = &[
    (0.0, 0x0000000000000000, 0x00000000),               // sin(0) = +0
    (0.5, 0x3FDEAEE8744B05F0, 0x3EF57744),
    (1.0, 0x3FEAED548F090CEE, 0x3F576AA4),
    (1.5707963267948966, 0x3FF0000000000000, 0x3F800000), // sin(π/2) ≈ 1 (octant boundary)
    (0.7853981633974483, 0x3FE6A09E667F3BCC, 0x3F3504F3), // sin(π/4) = √2/2
    (3.141592653589793, 0x3CA1A62633145C07, 0x250D3132),  // sin(π) ≈ 1.2e-16 — killer cancellation
    (2.0, 0x3FED18F6EAD1B446, 0x3F68C7B7),                 // octant 1 (cos branch)
    (3.0, 0x3FC210386DB6D55B, 0x3E1081C3),
    (10.0, 0xBFE1689EF5F34F52, 0xBF0B44F8),                // negative, multi-octave reduction
    (-1.0, 0xBFEAED548F090CEE, 0xBF576AA4),                // oddness in x
    (100.0, 0xBFE03425B78C4DB8, 0xBF01A12E),
    (6.283185307179586, 0xBCB1A62633145C07, 0xA58D3132),   // sin(2π) ≈ -2.4e-16
];

#[test]
fn sin_f64_matches_oracle_anchors() {
    for &(x, want64, _want32) in SIN_ANCHORS {
        let got = round_atom_to_f64(&Sin { x }).bits;
        assert_eq!(
            got, want64,
            "sin({x}) f64: got 0x{got:016X}, want 0x{want64:016X} (oracle: MPFR≡Arb≡mpmath)"
        );
    }
}

#[test]
fn sin_f32_matches_oracle_anchors() {
    for &(x, _want64, want32) in SIN_ANCHORS {
        let got = round_atom_to_f32(&Sin { x }).bits as u32;
        assert_eq!(
            got, want32,
            "sin({x}) f32: got 0x{got:08X}, want 0x{want32:08X} (oracle: MPFR≡Arb≡mpmath)"
        );
    }
}
