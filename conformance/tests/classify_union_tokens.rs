// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY-6.1-0017 .. -0020: the union-batch-1 dtype tokens `i128`, `u128`, `i256`, `u256`
//! (wide integers), `f128` (IEEE binary128), `f8e3m4` (IEEE-style FP8 E3M4) and `f8ue4m3` (the unsigned
//! NVFP4 scale). CireSnave board 501 "A" (EXPECTATIONS 6.35), union batch 1.
//!
//! Every format constant below is a LITERAL transcribed from the cited source, not computed from the
//! decoder under test, so the codec and the anchors can disagree.

use kiss_conformance::dtype::{
    f128_class, f8e3m4_decode, ue4m3_decode, wide_int_bounds, F128Class, F8Value, Ue4m3,
};
use kiss_conformance::grammar::dtype_storage_bits;
use kiss_conformance::structure_key::{from_token, DTYPES, RESERVED_DTYPES, UNION_BATCH1_DTYPES};

const FIRST_27: [&str; 27] = [
    "f16", "bf16", "f32", "f64", "i8", "i16", "u8", "u16", "i32", "i64", "u32", "u64", "bool",
    "f8e4m3fn", "f8e4m3fnuz", "f8e5m2", "f8e5m2fnuz", "f8e8m0", "f8e6m2", "i4", "u4", "b1", "c64", "c128",
    "f4e2m1", "f6e2m3", "f6e3m2",
];

// Backs[enforced]: KISS-CLASSIFY-6.1-0001 — the closed set is the earlier twenty-seven tokens in their
// ordinal order (so no §6.19-0025 accumulator ordinal moves) followed by the seven union-batch-1 tokens.
#[test]
fn test_classify_union_tokens_closed_set() {
    assert_eq!(DTYPES.len(), 34);
    assert_eq!(&DTYPES[..27], &FIRST_27[..], "no earlier token moves");
    assert_eq!(&DTYPES[27..], &["f128", "i128", "u128", "i256", "u256", "f8e3m4", "f8ue4m3"]);
    assert_eq!(UNION_BATCH1_DTYPES, ["f128", "i128", "u128", "i256", "u256", "f8e3m4", "f8ue4m3"]);
    assert_eq!(RESERVED_DTYPES.len(), 3, "none of the seven is reserved");
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0002 — exact storage widths.
#[test]
fn test_classify_union_tokens_bit_widths() {
    for (t, w) in [("f128", 128), ("i128", 128), ("u128", 128), ("i256", 256), ("u256", 256), ("f8e3m4", 8), ("f8ue4m3", 8)] {
        assert_eq!(dtype_storage_bits(t), Some(w), "{t}");
    }
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0017 — i128/i256 are two's complement, u128/u256 unsigned: the
// bounds are the literal bit patterns of the minimum and maximum, as big-endian 64-bit limbs.
#[test]
fn test_classify_wide_integer_bounds() {
    let max = u64::MAX;
    assert_eq!(wide_int_bounds("i128"), Some(([0x8000_0000_0000_0000, 0, 0, 0], [0x7FFF_FFFF_FFFF_FFFF, max, 0, 0], 128, true)).map(norm));
    assert_eq!(wide_int_bounds("u128"), Some(([0, 0, 0, 0], [max, max, 0, 0], 128, false)).map(norm));
    assert_eq!(wide_int_bounds("i256"), Some(([0x8000_0000_0000_0000, 0, 0, 0], [0x7FFF_FFFF_FFFF_FFFF, max, max, max], 256, true)).map(norm));
    assert_eq!(wide_int_bounds("u256"), Some(([0, 0, 0, 0], [max, max, max, max], 256, false)).map(norm));
    assert_eq!(wide_int_bounds("i64"), None, "only the four wide integers");
}

/// The test's literals are written most-significant-limb first at the token's own width; `wide_int_bounds`
/// returns `(min, max, bits, signed)` with the limbs right-aligned to the width. Normalise the literal.
fn norm(t: ([u64; 4], [u64; 4], u32, bool)) -> ([u64; 4], [u64; 4], u32, bool) {
    let (lo, hi, bits, signed) = t;
    let limbs = (bits / 64) as usize;
    let take = |a: [u64; 4]| {
        let mut o = [0u64; 4];
        o[..limbs].copy_from_slice(&a[..limbs]);
        o
    };
    (take(lo), take(hi), bits, signed)
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0018 — `f128` is IEEE-754 binary128: sign 1, exponent 15, fraction
// 112, bias 16383. IEEE 754-2019 Table 3.5 (interchange formats): k=128, p=113, emax=16383.
#[test]
fn test_classify_f128_format() {
    let exp_all = 0x7FFFu128 << 112;
    assert_eq!(f128_class(0), F128Class::Zero { negative: false });
    assert_eq!(f128_class(1u128 << 127), F128Class::Zero { negative: true });
    assert_eq!(f128_class(1), F128Class::Subnormal { negative: false });
    assert_eq!(f128_class(1u128 << 112), F128Class::Normal { negative: false }, "the smallest normal, biased exponent 1");
    assert_eq!(f128_class(exp_all), F128Class::Infinity { negative: false });
    assert_eq!(f128_class(exp_all | (1u128 << 127)), F128Class::Infinity { negative: true });
    // quiet NaN has the top fraction bit set (IEEE 754-2019 §6.2.1); signaling NaN has it clear, nonzero rest
    assert_eq!(f128_class(exp_all | (1u128 << 111)), F128Class::Nan { signaling: false });
    assert_eq!(f128_class(exp_all | 1), F128Class::Nan { signaling: true });
    assert_eq!(f128_class((0x7FFEu128 << 112) | ((1u128 << 112) - 1)), F128Class::Normal { negative: false }, "max finite");
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0019 — `f8e3m4` is the IEEE-style FP8 E3M4: sign 1, exponent 3,
// mantissa 4, bias 3; exponent field 7 with mantissa 0 is infinity and with mantissa non-zero is NaN;
// max finite 15.5 (0x6F), min normal 0.25 (0x10), max subnormal 0.234375 (0x0F), min subnormal 0.015625 (0x01).
// Literals: llvm/llvm-project 7c710c60 APFloat.cpp 300-313 (semFloat8E3M4: max exp 3, min exp -2, precision 5, IEEE754
// non-finite, IEEE NaN) and jax-ml/ml_dtypes 035edbc4 float8.h 666-728 (max 0b0'110'1111 = 15.5, min normal 1/4,
// infinity 0b0'111'0000, denorm_min 0b0'000'0001 = 1/64, isnan = abs(rep) > infinity rep at line 1256). Mojo's docstring
// (modular/modular bead9c4f dtype.mojo 225-238) says finite-only and DISAGREES; see KISS-CLASSIFY-6.1-0019.
#[test]
fn test_classify_f8e3m4_format() {
    assert_eq!(f8e3m4_decode(0x00), F8Value::Finite(0.0));
    assert!(matches!(f8e3m4_decode(0x80), F8Value::Finite(v) if v == 0.0 && v.is_sign_negative()), "-0 is distinct");
    assert_eq!(f8e3m4_decode(0x01), F8Value::Finite(0.015625));
    assert_eq!(f8e3m4_decode(0x0F), F8Value::Finite(0.234375));
    assert_eq!(f8e3m4_decode(0x10), F8Value::Finite(0.25));
    assert_eq!(f8e3m4_decode(0x30), F8Value::Finite(1.0), "exponent field 3 = bias, mantissa 0");
    assert_eq!(f8e3m4_decode(0x6F), F8Value::Finite(15.5));
    assert_eq!(f8e3m4_decode(0xEF), F8Value::Finite(-15.5));
    assert_eq!(f8e3m4_decode(0x70), F8Value::Infinity { negative: false });
    assert_eq!(f8e3m4_decode(0xF0), F8Value::Infinity { negative: true });
    for b in [0x71u8, 0x78, 0x7F, 0xF1, 0xFF] {
        assert_eq!(f8e3m4_decode(b), F8Value::Nan, "{b:#04x}");
    }
    // every finite value is exactly representable and the 256 codes split 2*112 finite-nonzero... :
    let nan = (0..=255u8).filter(|&b| f8e3m4_decode(b) == F8Value::Nan).count();
    let inf = (0..=255u8).filter(|&b| matches!(f8e3m4_decode(b), F8Value::Infinity { .. })).count();
    assert_eq!((nan, inf), (30, 2));
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0020 — `f8ue4m3` is the unsigned NVFP4 scale: exponent 4, mantissa 3,
// bias 7, NO sign bit (bit 7 is not part of the value), no infinity, a single NaN at 0x7F, subnormals
// defined, finite range [0, 448]. Source: NVIDIA/cutlass 5c179ede `include/cutlass/float8.h` lines 1075-1085
// ("Range: [0:448]; has_inf: false; has_NaN: true; has_denorm: true; Exponent bias (exp_bias): 7") and
// 1138-1141 (`isnan` is `storage == 0x7f`); `include/cutlass/exmy_base.h` lines 870-872 and 917-919 (`uint8_t`
// storage, 8 bits, 4 exponent bits, 3 mantissa bits, `CANONICAL_ONLY` NaN, unsigned).
#[test]
fn test_classify_ue4m3_format() {
    assert_eq!(ue4m3_decode(0x00), Ue4m3::Value(0.0));
    assert_eq!(ue4m3_decode(0x01), Ue4m3::Value(0.001953125), "min subnormal 2^-9");
    assert_eq!(ue4m3_decode(0x08), Ue4m3::Value(0.015625), "min normal 2^-6");
    assert_eq!(ue4m3_decode(0x38), Ue4m3::Value(1.0));
    assert_eq!(ue4m3_decode(0x7E), Ue4m3::Value(448.0), "max finite");
    assert_eq!(ue4m3_decode(0x7F), Ue4m3::Nan, "the single NaN");
    for b in 0x80..=0xFFu8 {
        assert_eq!(ue4m3_decode(b), Ue4m3::OutsideFormat, "bit 7 set: no sign bit exists, {b:#04x}");
    }
    assert_eq!((0..=0x7Fu8).filter(|&b| ue4m3_decode(b) == Ue4m3::Nan).count(), 1);
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0001 — the new tokens parse in every dtype position a usable dtype
// does (field 2) and derive; none is reserved, so none typed-declines.
#[test]
fn test_classify_union_tokens_usable_in_keys() {
    for t in UNION_BATCH1_DTYPES {
        let tok = format!("sk8|ew2|{t}|cuda:sm89|ix32|grid|r2|co/00/v1/d16/f/{t};co/00/v1/d16/f/{t}|-");
        let key = from_token(&tok).unwrap_or_else(|e| panic!("{t}: {e:?}"));
        assert_eq!(key.to_token(), tok, "{t} round-trips byte-identically");
    }
}

fn derived(name: &str) -> Result<String, kiss_conformance::derive::DeriveDecline> {
    let v = kiss_conformance::derivation_vectors::vectors();
    let d = v.iter().find(|d| d.name == name).unwrap_or_else(|| panic!("vector {name}"));
    kiss_conformance::derive::derive_structure_key(&d.cell).map(|k| k.to_token())
}

// Backs[enforced]: KISS-CLASSIFY-6.1-0001, -0017, -0018, -0019 — each new element token derives in the dtype
// field and its operand sub-keys, with the vector width the byte cap allows: 1-byte `f8e3m4` vectorizes to v8,
// the 16- and 32-byte tokens stay v1 (they exceed the 16-byte default cap at width 2).
#[test]
fn test_classify_union_tokens_derive() {
    assert_eq!(
        derived("union_f8e3m4_elementwise_add").unwrap(),
        "sk8|ew2|f8e3m4|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f/f8e3m4;co/00/v8/d16/f/f8e3m4;co/00/v8/d16/f/f8e3m4|-"
    );
    for t in ["f128", "i128", "u128", "i256", "u256"] {
        assert_eq!(
            derived(&format!("union_{t}_elementwise_add")).unwrap(),
            format!("sk8|ew2|{t}|cuda:sm89|ix32|grid|r2|co/00/v1/d16/f/{t};co/00/v1/d16/f/{t};co/00/v1/d16/f/{t}|-"),
            "{t}"
        );
    }
}

// Backs[enforced]: KISS-CLASSIFY-6.6-0021 + 6.1-0020 — `f8ue4m3` is a scale type: operand 0 declines, and as the
// scale of an f4e2m1 weight it spells `qf8ue4m3` with the block length `g16` (the NVFP4 shape).
#[test]
fn test_classify_ue4m3_is_a_scale_type() {
    assert_eq!(
        derived("union_ue4m3_at_operand0_declines").unwrap_err(),
        kiss_conformance::derive::DeriveDecline::ScaleDtypeAtOperand0
    );
    assert_eq!(
        derived("union_ctr_nvfp4_scale_ue4m3").unwrap(),
        "sk8|ctr|bf16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f/bf16;co/00/v1/d16/f/f4e2m1/dl;co/00/v8/d16/f/f8ue4m3;co/00/v8/d16/f/bf16|-|ctll/d16/f4e2m1/f32/bf16/st/qf8ue4m3/g16"
    );
}

/// The contraction field (the part after the last `|`) of a derived token.
fn ctr_field(t: &str) -> &str {
    t.rsplit('|').next().unwrap()
}

// Backs[enforced]: KISS-CLASSIFY-6.6-0023a — K is a role SET keyed by the PRODUCT of its extents: K = 8 x 16 and
// the single K = 128 derive the same contraction field; three axes 16 x 16 x 16 give the Large class (4096).
// Teeth: a deriver that read only the first K axis would key 8 (Tiny) and fail the equality.
#[test]
fn test_classify_k_role_set_is_a_product() {
    let two = derived("ctr_two_k_axes_product_8x16").unwrap();
    let one = derived("ctr_single_k_128_same_contraction_as_two_k_8x16").unwrap();
    assert_eq!(ctr_field(&two), "csms/d16/f16/f16/f16/st");
    assert_eq!(ctr_field(&two), ctr_field(&one), "product 128 = single K 128");
    assert_eq!(ctr_field(&derived("ctr_three_k_axes_product_large").unwrap()), "csml/d16/f16/f16/f16/st");
}

// Backs[enforced]: KISS-CLASSIFY-6.6-0023a — a further K axis that repeats K or is M/N is a typed decline.
#[test]
fn test_classify_k_role_set_declines_malformed() {
    use kiss_conformance::derive::DeriveDecline::ContractionAxisInvalid;
    assert_eq!(derived("ctr_extra_k_axis_repeats_k_declines").unwrap_err(), ContractionAxisInvalid);
    assert_eq!(derived("ctr_extra_k_axis_is_m_declines").unwrap_err(), ContractionAxisInvalid);
}
