//! Plan B Slice-0 Task T5 — `log` (natural log) atom vectors, correctly-rounded.
//!
//! The `Log` atom composes `reduce_log` (T4: x = 2^e·m, m ∈ [√2/2, √2), m EXACT)
//! with the atanh series `log(m) = 2·atanh(t)`, `t = (m−1)/(m+1)` (|t| ≤ 0.172),
//! then adds the exact octave term `e·ln2`. `err_ulps` folds the series rounding,
//! the t-division propagation, and the `|e|·ln2`-truncation term.
//!
//! ANCHORS are correctly-rounded-to-nearest, cross-validated by MPFR (gmpy2) ≡
//! Arb (python-flint) ≡ mpmath — bit-for-bit.

use kiss_conformance::hp::{round_atom_to_f32, round_atom_to_f64, Log};

/// (x, f64 bits, f32 bits) of ln(x) — oracle-agreed.
// anchor INPUTS are exact f64 evaluation points (e, √2, 1/√2); deliberately literal so the whole
// oracle-agreed table is ONE self-contained record, not std::consts-derived (clippy::approx_constant).
#[allow(clippy::approx_constant)]
const LOG_ANCHORS: &[(f64, u64, u32)] = &[
    (1.0, 0x0000000000000000, 0x00000000),               // log(1) = +0 exactly
    (2.0, 0x3FE62E42FEFA39EF, 0x3F317218),               // ln2
    (0.5, 0xBFE62E42FEFA39EF, 0xBF317218),               // -ln2
    (2.718281828459045, 0x3FF0000000000000, 0x3F800000), // log(e) ≈ 1
    (10.0, 0x40026BB1BBB55516, 0x40135D8E),
    (0.1, 0xC0026BB1BBB55515, 0xC0135D8E),
    (1.5, 0x3FD9F323ECBF984C, 0x3ECF991F),
    (1.4142135623730951, 0x3FD62E42FEFA39F0, 0x3EB17218), // log(√2) = ln2/2
    (0.7071067811865476, 0xBFD62E42FEFA39EE, 0xBEB17218), // log(1/√2)
    (3.0, 0x3FF193EA7AAD030B, 0x3F8C9F54),
    (100.0, 0x40126BB1BBB55516, 0x40935D8E),
    (1e-10, 0xC037069E2AA2AA5B, 0xC1B834F1),              // large octave e ≈ -34
];

#[test]
fn log_f64_matches_oracle_anchors() {
    for &(x, want64, _want32) in LOG_ANCHORS {
        let got = round_atom_to_f64(&Log { x }).bits;
        assert_eq!(
            got, want64,
            "log({x}) f64: got 0x{got:016X}, want 0x{want64:016X} (oracle: MPFR≡Arb≡mpmath)"
        );
    }
}

#[test]
fn log_f32_matches_oracle_anchors() {
    for &(x, _want64, want32) in LOG_ANCHORS {
        let got = round_atom_to_f32(&Log { x }).bits as u32;
        assert_eq!(
            got, want32,
            "log({x}) f32: got 0x{got:08X}, want 0x{want32:08X} (oracle: MPFR≡Arb≡mpmath)"
        );
    }
}

/// KISS-OPS §6.15 log argument reduction — reduce_log's SUBNORMAL branch (frexp pre-scales a
/// subnormal by 2^54 then corrects the exponent) is otherwise dead in tests, and it DISCRIMINATES a
/// correct decompose from a naive one. For x = 2^-1074 (smallest positive f64, exponent field 0) a
/// plausible-but-wrong impl that reads the exponent field directly WITHOUT a subnormal special-case
/// takes e = 0 − 1023 = −1023 (ignoring the 51-bit leading-zero shift) and returns log ≈ −709
/// instead of the correct ≈ −744.44 — off by ~35 = 51·ln2. The atom (frexp subnormal branch)
/// returns the 3-oracle value (MPFR ≡ Arb ≡ mpmath). Values live here, not in the const array:
/// a subnormal's exact f64 needs `f64::from_bits`, const-stable only since Rust 1.83 (MSRV 1.77).
#[test]
fn log_subnormal_reddens_naive_decompose() {
    // Smallest subnormal 2^-1074 (extreme leading-zero shift) and largest subnormal (full mantissa).
    for (xbits, want64, want32) in [
        (0x0000_0000_0000_0001u64, 0xC087_4385_446D_71C3u64, 0xC43A_1C2Au32),
        (0x000F_FFFF_FFFF_FFFFu64, 0xC086_232B_DD7A_BCD2u64, 0xC431_195Fu32),
    ] {
        let x = f64::from_bits(xbits);
        assert!(x.is_subnormal(), "0x{xbits:016X} must be subnormal");
        let g64 = round_atom_to_f64(&Log { x }).bits;
        assert_eq!(g64, want64, "log(0x{xbits:016X}) f64: got 0x{g64:016X}, want 0x{want64:016X}");
        let g32 = round_atom_to_f32(&Log { x }).bits as u32;
        assert_eq!(g32, want32, "log(0x{xbits:016X}) f32: got 0x{g32:08X}, want 0x{want32:08X}");
    }

    // The discrimination, made explicit: a naive exp-field decompose (no subnormal branch) on
    // 2^-1074 is catastrophically wrong, so the subnormal branch is what pins the correct value.
    let x = f64::from_bits(1); // 2^-1074
    let correct = f64::from_bits(round_atom_to_f64(&Log { x }).bits);
    let ef = ((x.to_bits() >> 52) & 0x7FF) as i32; // 0 for a subnormal
    let naive_e = ef - 1023; // −1023, ignoring the leading-zero shift a subnormal branch corrects
    let naive_m = f64::from_bits((x.to_bits() & 0x000F_FFFF_FFFF_FFFF) | 0x3FF0_0000_0000_0000);
    let naive_log = naive_e as f64 * std::f64::consts::LN_2 + naive_m.ln();
    assert!(
        (naive_log - correct).abs() > 1.0,
        "naive exp-field decompose (no subnormal branch) is catastrophically wrong: {naive_log} vs {correct}"
    );
}
