// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY §6.1-0011 (the FP8 E5M2 format constants) and §6.2-0002 (the special values of every
//! float dtype, by pinned bit pattern).
//!
//! The decode below is written from the FORMAT DEFINITIONS (sign / exponent width / mantissa width / bias /
//! the NaN-and-infinity convention), never from `fp.rs`, so the reference crate's own decoders
//! (`e5m2_to_f32`, `e4m3_to_f32`, `bf16_to_f32`, `e8m0_decode`) and the platform's `f32`/`f64` are held
//! against an independent reading over every byte, not spot values.
//!
//! Scope, stated: these pin the formats' constants and special-value patterns. They exercise no storage
//! of any implementation (a foreign reader, R4.1), and the `f8e6m2` clause (-0013) "defines no special
//! values" is a statement of absence this file does not and cannot test.

use kiss_conformance::dtype::{e8m0_decode, E8M0};
use kiss_conformance::fp::{bf16_to_f32, e4m3_to_f32, e5m2_to_f32};

/// What a bit pattern IS, independent of any value computation.
#[derive(Debug, Clone, Copy, PartialEq)]
enum V {
    Zero { neg: bool },
    Inf { neg: bool },
    Nan,
    Finite(f64),
}

/// How a format spends the all-ones exponent / the top code.
#[derive(Clone, Copy)]
enum Conv {
    /// IEEE: exponent all ones; mantissa 0 = infinity, else NaN.
    Ieee,
    /// OCP E4M3FN: no infinity; only `S.1111.111` is NaN; `-0` exists.
    Fn,
    /// FNUZ: no infinity, no `-0`; the single NaN is the sign-only pattern `0x80`.
    Fnuz,
}

fn decode(bits: u32, exp_bits: u32, man_bits: u32, bias: i32, conv: Conv) -> V {
    let width = 1 + exp_bits + man_bits;
    let neg = bits >> (width - 1) & 1 == 1;
    let e = (bits >> man_bits) & ((1 << exp_bits) - 1);
    let m = bits & ((1 << man_bits) - 1);
    let top = (1u32 << exp_bits) - 1;
    match conv {
        Conv::Ieee if e == top => return if m == 0 { V::Inf { neg } } else { V::Nan },
        Conv::Fn if e == top && m == (1 << man_bits) - 1 => return V::Nan,
        Conv::Fnuz if bits == 1 << (width - 1) => return V::Nan,
        _ => {}
    }
    if e == 0 && m == 0 {
        return V::Zero { neg };
    }
    let mag = if e == 0 {
        f64::from(m) / f64::from(1u32 << man_bits) * 2f64.powi(1 - bias)
    } else {
        (1.0 + f64::from(m) / f64::from(1u32 << man_bits)) * 2f64.powi(e as i32 - bias)
    };
    V::Finite(if neg { -mag } else { mag })
}

fn e5m2(b: u8) -> V {
    decode(u32::from(b), 5, 2, 15, Conv::Ieee)
}
fn e5m2fnuz(b: u8) -> V {
    decode(u32::from(b), 5, 2, 16, Conv::Fnuz)
}
fn e4m3fn(b: u8) -> V {
    decode(u32::from(b), 4, 3, 7, Conv::Fn)
}
fn e4m3fnuz(b: u8) -> V {
    decode(u32::from(b), 4, 3, 8, Conv::Fnuz)
}

/// What the reference crate's `f32` result for a byte IS, in the same vocabulary.
fn classify_f32(x: f32) -> V {
    if x.is_nan() {
        V::Nan
    } else if x.is_infinite() {
        V::Inf { neg: x.is_sign_negative() }
    } else if x == 0.0 {
        V::Zero { neg: x.is_sign_negative() }
    } else {
        V::Finite(f64::from(x))
    }
}

// Backs: KISS-CLASSIFY-6.1-0011 — f8e5m2 is sign 1 / exp 5 / mantissa 2 / bias 15, IEEE-style infinity and
// NaN, maximum finite 57344; f8e5m2fnuz is bias 16, has no -0 and no infinity, and is byte-incompatible.
#[test]
fn test_classify_e5m2_format() {
    // every byte: the crate's decoder against the independent one
    for b in 0..=255u8 {
        assert_eq!(classify_f32(e5m2_to_f32(b)), e5m2(b), "e5m2 byte {b:#04x}");
    }
    // pinned constants
    assert_eq!(e5m2(0x7B), V::Finite(57344.0), "maximum finite magnitude");
    assert_eq!(e5m2(0xFB), V::Finite(-57344.0));
    assert_eq!(e5m2(0x7C), V::Inf { neg: false });
    assert_eq!(e5m2(0xFC), V::Inf { neg: true });
    let nans: Vec<u8> = (0..=255u8).filter(|&b| e5m2(b) == V::Nan).collect();
    assert_eq!(nans, [0x7D, 0x7E, 0x7F, 0xFD, 0xFE, 0xFF], "IEEE-style NaN: exponent 31, mantissa != 0");
    assert_eq!(e5m2(0x3C), V::Finite(1.0), "bias 15: S.01111.00 is 1.0");
    assert_eq!(e5m2(0x80), V::Zero { neg: true }, "e5m2 defines -0");

    // the reserved fnuz variant
    assert_eq!(e5m2fnuz(0x3C), V::Finite(0.5), "bias 16: the SAME byte is 0.5, not 1.0");
    assert_ne!(e5m2fnuz(0x3C), e5m2(0x3C), "byte-incompatible with f8e5m2");
    assert_eq!(e5m2fnuz(0x80), V::Nan, "no -0: the sign-only pattern is the NaN");
    assert!((0..=255u8).all(|b| !matches!(e5m2fnuz(b), V::Inf { .. })), "defines no infinities");
    assert!((0..=255u8).all(|b| e5m2fnuz(b) != V::Zero { neg: true }), "defines no -0");
    assert_eq!(e5m2fnuz(0x7F), V::Finite(57344.0), "top exponent is an ordinary finite code");
}

// Backs: KISS-CLASSIFY-6.2-0002 — each float format's special values are identified by pinned bit patterns:
// +-0 (except fnuz), subnormals, infinities (f16/bf16/f32/f64/f8e5m2 only), quiet and signaling NaN where
// both exist, and the single NaN of the FP8 variants; f8e8m0 follows its own restatement (-0013).
#[test]
fn test_classify_float_special_values_pinned() {
    // ---- f32 / f64 against the platform ----
    assert_eq!(f32::INFINITY.to_bits(), 0x7F80_0000);
    assert_eq!(f32::NEG_INFINITY.to_bits(), 0xFF80_0000);
    assert_eq!((-0.0f32).to_bits(), 0x8000_0000);
    assert!(f32::from_bits(0x0000_0001).is_subnormal());
    let (qnan, snan) = (0x7FC0_0000u32, 0x7F80_0001u32);
    assert!(f32::from_bits(qnan).is_nan() && f32::from_bits(snan).is_nan());
    assert_ne!(qnan, snan, "quiet and signaling NaN are distinct encodings");
    assert_eq!(qnan & 0x0040_0000, 0x0040_0000, "quiet bit set");
    assert_eq!(snan & 0x0040_0000, 0, "quiet bit clear");
    assert_eq!(f64::INFINITY.to_bits(), 0x7FF0_0000_0000_0000);
    assert_eq!((-0.0f64).to_bits(), 0x8000_0000_0000_0000);
    assert!(f64::from_bits(1).is_subnormal());

    // ---- the same patterns by the format definition (f32 w/ 8,23,127; f64 w/ 11,52,1023; f16 w/ 5,10,15) ----
    assert_eq!(decode(0x7F80_0000, 8, 23, 127, Conv::Ieee), V::Inf { neg: false });
    assert_eq!(decode(0x8000_0000, 8, 23, 127, Conv::Ieee), V::Zero { neg: true });
    assert_eq!(decode(snan, 8, 23, 127, Conv::Ieee), V::Nan);
    assert_eq!(decode(0x7C00, 5, 10, 15, Conv::Ieee), V::Inf { neg: false }, "f16 +inf");
    assert_eq!(decode(0xFC00, 5, 10, 15, Conv::Ieee), V::Inf { neg: true }, "f16 -inf");
    assert_eq!(decode(0x8000, 5, 10, 15, Conv::Ieee), V::Zero { neg: true }, "f16 -0");
    assert_eq!(decode(0x7E00, 5, 10, 15, Conv::Ieee), V::Nan, "f16 quiet NaN");
    assert_eq!(decode(0x7C01, 5, 10, 15, Conv::Ieee), V::Nan, "f16 signaling NaN");
    assert_eq!(decode(0x0001, 5, 10, 15, Conv::Ieee), V::Finite(2f64.powi(-24)), "f16 smallest subnormal");

    // ---- bf16 through the crate's decoder ----
    assert_eq!(classify_f32(bf16_to_f32(0x7F80)), V::Inf { neg: false });
    assert_eq!(classify_f32(bf16_to_f32(0xFF80)), V::Inf { neg: true });
    assert_eq!(classify_f32(bf16_to_f32(0x8000)), V::Zero { neg: true });
    assert_eq!(classify_f32(bf16_to_f32(0x7FC0)), V::Nan, "bf16 quiet NaN");
    assert_eq!(classify_f32(bf16_to_f32(0x7F81)), V::Nan, "bf16 signaling NaN");
    assert_eq!(classify_f32(bf16_to_f32(0x0001)), V::Finite(2f64.powi(-133)), "bf16 smallest subnormal");

    // ---- f8e4m3fn: no infinity, single NaN (both signs), -0 exists, subnormals exist ----
    for b in 0..=255u8 {
        assert_eq!(classify_f32(e4m3_to_f32(b)), e4m3fn(b), "e4m3fn byte {b:#04x}");
    }
    let nans: Vec<u8> = (0..=255u8).filter(|&b| e4m3fn(b) == V::Nan).collect();
    assert_eq!(nans, [0x7F, 0xFF], "single NaN encoding per sign");
    assert!((0..=255u8).all(|b| !matches!(e4m3fn(b), V::Inf { .. })), "defines no infinity");
    assert_eq!(e4m3fn(0x80), V::Zero { neg: true });
    assert_eq!(e4m3fn(0x01), V::Finite(2f64.powi(-9)), "smallest subnormal");

    // ---- f8e5m2: infinities, NaNs, -0, subnormals (pinned in the 6.1-0011 test; the subnormal here) ----
    assert_eq!(e5m2(0x01), V::Finite(2f64.powi(-16)), "smallest subnormal");

    // ---- the fnuz variants: no -0, no infinity, one NaN, subnormals exist ----
    for (name, f) in [("f8e4m3fnuz", e4m3fnuz as fn(u8) -> V), ("f8e5m2fnuz", e5m2fnuz as fn(u8) -> V)] {
        let nan: Vec<u8> = (0..=255u8).filter(|&b| f(b) == V::Nan).collect();
        assert_eq!(nan, [0x80], "{name}: the single NaN is the sign-only pattern");
        assert!((0..=255u8).all(|b| !matches!(f(b), V::Inf { .. })), "{name}: no infinity");
        assert!((0..=255u8).all(|b| f(b) != V::Zero { neg: true }), "{name}: no -0");
        assert_eq!(f(0x00), V::Zero { neg: false }, "{name}: +0 is the all-zero byte");
        assert!(matches!(f(0x01), V::Finite(x) if x > 0.0), "{name}: subnormal exists");
    }
    assert_eq!(e4m3fnuz(0x38), V::Finite(0.5), "bias 8: S.0111.000 is 0.5, not the e4m3fn 1.0");
    assert_eq!(e4m3fn(0x38), V::Finite(1.0));

    // ---- f8e8m0 (§6.1-0013): 0xFF NaN, no infinity, NO zero ----
    for e in 0..=0xFEu8 {
        assert_eq!(e8m0_decode(e), E8M0::Pow2(i32::from(e) - 127), "byte {e:#04x}");
    }
    assert_eq!(e8m0_decode(0xFF), E8M0::Nan);
}

// ---- controls: the independent decode must discriminate, or the comparison above proves nothing ----

#[test]
fn control_the_decode_distinguishes_formats_and_catches_a_wrong_bias() {
    // a wrong bias or a wrong convention changes what a pinned pattern means
    assert_ne!(decode(0x3C, 5, 2, 15, Conv::Ieee), decode(0x3C, 5, 2, 16, Conv::Ieee), "bias matters");
    assert_ne!(decode(0x7C, 5, 2, 15, Conv::Ieee), decode(0x7C, 5, 2, 15, Conv::Fn), "infinity convention matters");
    assert_ne!(decode(0x80, 5, 2, 16, Conv::Fnuz), decode(0x80, 5, 2, 15, Conv::Ieee), "fnuz 0x80 is NaN, ieee is -0");
    // and the crate's decoder is compared per byte, so a swapped inf/NaN would be reported
    let wrong = |b: u8| if b == 0x7C { f32::NAN } else { e5m2_to_f32(b) };
    assert_ne!(classify_f32(wrong(0x7C)), e5m2(0x7C));
}
