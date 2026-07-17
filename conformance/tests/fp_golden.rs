//! KISS-Conform vectors for the reduced-float storage codecs (KISS-Ops §6.16).

use kiss_conformance::fp::*;

/// KISS-OPS-6.16-0003 — bf16 is a 1-sign / 8-exp / 7-mantissa format (bias 127)
/// carrying the binary32 exponent range, and any op producing a bf16 result MUST
/// round to nearest, ties to even. Verbatim (spec/ops.md:1259-1262).
#[test]
fn test_ops_bf16_layout() {
    // --- layout: bf16 is the high 16 bits of the binary32 encoding ---------
    // 1.0 f32 = 0x3F800000 -> bf16 0x3F80. Field decode: sign 0, exp field
    // 0x7F = 127 (bias 127 -> 2^0), mantissa 0.
    assert_eq!(f32_to_bf16(1.0), 0x3F80);
    let b = f32_to_bf16(1.0);
    assert_eq!(b >> 15, 0, "sign bit");
    assert_eq!((b >> 7) & 0xFF, 127, "8-bit exponent field, bias 127");
    assert_eq!(b & 0x7F, 0, "7-bit mantissa field");
    assert_eq!(bf16_to_f32(0x3F80), 1.0, "decode is exact widening");

    // --- RNE round-UP: catches the naive truncation (bits >> 16) ----------
    // 0x3FCCCCCD (~1.6): low 16 bits 0xCCCD > 0x8000 -> round up.
    let x = f32::from_bits(0x3FCC_CCCD);
    assert_eq!(f32_to_bf16(x), 0x3FCD, "RNE rounds up");
    assert_ne!(f32_to_bf16(x), (x.to_bits() >> 16) as u16, "differs from truncation 0x3FCC");
    assert_eq!((x.to_bits() >> 16) as u16, 0x3FCC, "the wrong-impl value, pinned for contrast");

    // --- ties-to-even, tie rounds UP to even (also != truncation) ----------
    // 0x3F818000: low 16 == 0x8000 (exact tie); retained value 0x3F81 is odd
    // -> round up to even 0x3F82.
    assert_eq!(f32_to_bf16(f32::from_bits(0x3F81_8000)), 0x3F82);

    // --- ties-to-even, tie rounds DOWN (catches round-half-AWAY) -----------
    // 0x3F808000: exact tie; retained 0x3F80 is even -> stays 0x3F80.
    // (round-half-away-from-zero would wrongly give 0x3F81.)
    assert_eq!(f32_to_bf16(f32::from_bits(0x3F80_8000)), 0x3F80);

    // --- mantissa carry propagates into the exponent ----------------------
    // 0x3FFF8000: mantissa all ones + tie up -> carry to exp+1, mantissa 0.
    assert_eq!(f32_to_bf16(f32::from_bits(0x3FFF_8000)), 0x4000);
    assert_eq!(bf16_to_f32(0x4000), 2.0);

    // --- bf16 shares f32's exponent range => NOT saturating: overflow rounds
    //     to infinity (catches an impl that saturates like the fp8 formats). --
    let max_f32 = f32::from_bits(0x7F7F_FFFF); // largest finite binary32
    assert_eq!(f32_to_bf16(max_f32), 0x7F80, "rounds up to +inf, does not saturate");
    assert!(bf16_to_f32(f32_to_bf16(max_f32)).is_infinite());

    // --- sign preserved ----------------------------------------------------
    assert_eq!(f32_to_bf16(-1.0), 0xBF80);
}

/// KISS-OPS-6.16-0004 — e4m3 per OCP OFP8: 1-sign / 4-exp / 3-mantissa (bias 7),
/// max finite magnitude ±448, NO infinity encoding, a single NaN encoding;
/// conversion into e4m3 saturates to the max finite magnitude under
/// round-half-to-even. Verbatim (spec/ops.md:1263-1266).
#[test]
fn test_ops_e4m3_layout() {
    // --- layout / bias 7 anchors ------------------------------------------
    // 1.0 = S.0111.000 (exp field 7 = bias => 2^0)         -> 0x38
    assert_eq!(f32_to_e4m3(1.0), 0x38);
    // smallest normal 2^-6 = S.0001.000 (exp field 1)      -> 0x08
    assert_eq!(f32_to_e4m3(2f32.powi(-6)), 0x08);
    // smallest subnormal 2^-9 = S.0000.001                 -> 0x01
    assert_eq!(f32_to_e4m3(2f32.powi(-9)), 0x01);
    // max finite 448 = S.1111.110                          -> 0x7E
    assert_eq!(f32_to_e4m3(448.0), E4M3_MAX_FINITE_MAG);
    assert_eq!(E4M3_MAX_FINITE_MAG, 0x7E);
    assert_eq!(e4m3_to_f32(0x7E), 448.0, "±448 is the largest finite magnitude");

    // --- NO infinity; single NaN at S.1111.111 ----------------------------
    // The top-exponent codes below 0x7F are ordinary FINITE values, proving
    // there is no infinity encoding: 0x7C = S.1111.100 = 1.5 * 2^8 = 384.
    assert!(e4m3_to_f32(0x7C).is_finite());
    assert_eq!(e4m3_to_f32(0x7C), 384.0);
    // The one NaN encoding.
    assert!(e4m3_to_f32(0x7F).is_nan());
    assert!(e4m3_to_f32(0xFF).is_nan());

    // --- conversion SATURATES overflow to ±448, never inf/NaN -------------
    assert_eq!(f32_to_e4m3(1000.0), 0x7E);
    assert_eq!(f32_to_e4m3(1e30), 0x7E);
    assert!(e4m3_to_f32(f32_to_e4m3(1e30)).is_finite(), "overflow gives a finite, not NaN/inf");
    // 470 rounds toward 480 = S.1111.111 — but that slot IS the NaN, so it
    // MUST saturate to 448 (0x7E), NOT emit 0x7F. Catches a rounder that
    // ignores the NaN slot / max-finite clamp.
    assert_eq!(f32_to_e4m3(470.0), 0x7E);
    assert_ne!(f32_to_e4m3(470.0), E4M3_NAN_MAG);

    // --- round-half-to-even in the subnormal region -----------------------
    // 2^-10 is exactly half the smallest subnormal (2^-9): tie -> even 0.
    assert_eq!(f32_to_e4m3(2f32.powi(-10)), 0x00);
    // 3 * 2^-11 (= 1.5 * 2^-10) is above the tie -> rounds up to 2^-9.
    assert_eq!(f32_to_e4m3(3.0 * 2f32.powi(-11)), 0x01);

    // --- sign preserved (incl. -0.0) --------------------------------------
    assert_eq!(f32_to_e4m3(-448.0), 0xFE);
    assert_eq!(f32_to_e4m3(-0.0), 0x80);
}

/// KISS-OPS-6.16-0005 — e5m2 per OCP OFP8: 1-sign / 5-exp / 2-mantissa (bias 15),
/// max finite magnitude ±57344, IEEE-style infinity and NaN encodings; conversion
/// into e5m2 saturates to the max finite magnitude under round-half-to-even.
/// Verbatim (spec/ops.md:1267-1270).
#[test]
fn test_ops_e5m2_layout() {
    // --- layout / bias 15 anchors -----------------------------------------
    // 1.0 = S.01111.00 (exp field 15 = bias => 2^0)        -> 0x3C
    assert_eq!(f32_to_e5m2(1.0), 0x3C);
    // smallest normal 2^-14 = S.00001.00 (exp field 1)     -> 0x04
    assert_eq!(f32_to_e5m2(2f32.powi(-14)), 0x04);
    // smallest subnormal 2^-16 = S.00000.01                -> 0x01
    assert_eq!(f32_to_e5m2(2f32.powi(-16)), 0x01);
    // max finite 57344 = S.11110.11                        -> 0x7B
    assert_eq!(f32_to_e5m2(57344.0), E5M2_MAX_FINITE_MAG);
    assert_eq!(E5M2_MAX_FINITE_MAG, 0x7B);
    assert_eq!(e5m2_to_f32(0x7B), 57344.0);

    // --- e5m2 HAS IEEE-style inf/NaN at exp field 31 ----------------------
    assert!(e5m2_to_f32(0x7C).is_infinite() && e5m2_to_f32(0x7C) > 0.0, "S.11111.00 = +inf");
    assert!(e5m2_to_f32(0xFC).is_infinite() && e5m2_to_f32(0xFC) < 0.0, "-inf");
    assert!(e5m2_to_f32(0x7E).is_nan(), "S.11111.10 = NaN");
    // CONTRAST vs e4m3: the SAME byte 0x7C is a finite value in e4m3 (no inf)
    // but infinity in e5m2 (has inf) — this is the load-bearing format split.
    assert!(e4m3_to_f32(0x7C).is_finite());
    assert!(e5m2_to_f32(0x7C).is_infinite());

    // --- conversion of FINITE overflow SATURATES to ±57344, NOT inf -------
    // Even though e5m2 has an infinity encoding, f32->e5m2 must saturate.
    assert_eq!(f32_to_e5m2(1e30), 0x7B);
    assert_ne!(f32_to_e5m2(1e30), E5M2_INF_MAG, "finite overflow must NOT become +inf");
    assert_eq!(f32_to_e5m2(100000.0), 0x7B);
    assert!(e5m2_to_f32(f32_to_e5m2(1e30)).is_finite());
    // An infinite INPUT, by contrast, maps to the infinity encoding.
    assert_eq!(f32_to_e5m2(f32::INFINITY), E5M2_INF_MAG);

    // --- round-half-to-even in the normal region --------------------------
    // 1.375 is the exact midpoint of 1.25 (M=1) and 1.5 (M=2): tie -> even M=2.
    assert_eq!(f32_to_e5m2(1.375), 0x3E, "tie to even mantissa (M=2)");
    assert_eq!(e5m2_to_f32(0x3E), 1.5);
    // 1.3 rounds to 1.25 (M=1); catches truncation-toward-zero which agrees
    // here, so pair it with the 1.375 tie above which truncation gets wrong.
    assert_eq!(f32_to_e5m2(1.3), 0x3D);

    // --- sign preserved ----------------------------------------------------
    assert_eq!(f32_to_e5m2(-57344.0), 0xFB);
}
