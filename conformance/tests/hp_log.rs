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
