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
// anchor INPUTS are exact f64 evaluation points (π/2, π/4, π, 2π); deliberately literal so the whole
// oracle-agreed table is ONE self-contained record, not std::consts-derived (clippy::approx_constant).
#[allow(clippy::approx_constant)]
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
// NB: the deep-cancellation OCTANT-5 case (x = 6381956970095103·2^797) is NOT in this const array —
// its exact f64 needs `f64::from_bits`, which is const-stable only since Rust 1.83 while the crate's
// MSRV is 1.77 (a const literal for a 2^849 value is impractical). It lives in the test below, where
// `from_bits` is a non-const call and covers the same f64+f32 obligations plus the discrimination.

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

/// KISS-OPS §6.15 sin argument reduction — the deep-cancellation worst case DISCRIMINATES a wide
/// (Payne–Hanek) reduction from a naive one, which is the point of carrying it. x =
/// 6381956970095103·2^797 sits ≈2^-61 past a multiple of π/2 (octant 5). The atom, using the
/// 2304-bit 2/π table, returns the 3-oracle-agreed sin ≈ 1.0; a plausible-but-wrong
/// `sin(fmod(x, 2π))` (double-precision argument reduction — what a lesser implementation writes)
/// loses all significance and lands FAR from 1.0. A named, wrong implementation this anchor kills.
#[test]
fn sin_deep_cancellation_reddens_naive_reduction() {
    let x = f64::from_bits(0x7506AC5B262CA1FF); // 6381956970095103 · 2^797 (exact); |x mod π/2| ≈ 2^-61
    let atom = round_atom_to_f64(&Sin { x }).bits;
    assert_eq!(atom, 0x3FF0000000000000, "Payne–Hanek atom: sin(deep-cancellation x) rounds to 1.0 (f64)");
    // f32 too — same obligation the const anchor array would carry (3-oracle agreed: MPFR≡Arb≡mpmath).
    let atom32 = round_atom_to_f32(&Sin { x }).bits as u32;
    assert_eq!(atom32, 0x3F800000, "Payne–Hanek atom: sin(deep-cancellation x) rounds to 1.0 (f32)");

    // The naive reduction a lesser impl uses, computed here to SHOW the anchor discriminates:
    // fmod(x, 2π) in f64 keeps only ~2 bits of the ~849-bit argument, so sin of it is unrelated to
    // the true value. Assert it is CATASTROPHICALLY wrong (far from 1.0), not a last-bit diff —
    // robust to libm variation in the exact wrong value, which is not the point.
    let naive = (x % (2.0 * std::f64::consts::PI)).sin();
    assert_ne!(naive.to_bits(), atom, "naive fmod-2π must differ from the correct result");
    assert!(
        (naive - 1.0).abs() > 0.5,
        "naive fmod-2π reduction is catastrophically wrong ({naive}), not within tolerance of 1.0 — \
         the correctly-rounded sin only a wide reduction reaches"
    );
}
