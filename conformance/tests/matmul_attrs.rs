// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.19-0049 (the `matmul` OpAttrs blob), §6.17-0010 (the integer accumulator) and §6.17-0011
//! (precision tags): CireSnave 2026-11-12, board 426 6 "A" and board 427 "1: A".

use kiss_conformance::opattrs::{self, Decline, MathFidelityOrd, PrecisionTag};
use kiss_conformance::structural::{int_dot_exact, matmul_attrs_check, MatmulAttrDecline};

// Backs[enforced]: KISS-OPS-6.19-0049 — the matmul blob is exactly accumulator, math_fidelity, precision_tag (three u8, emitted explicitly, defaults 00 01 01); it round-trips and a wrong length, a reserved zero fidelity or tag, an out-of-table ordinal and a reserved or out-of-table accumulator are typed declines; matmul is a carrier op in the §6.19 table
#[test]
fn test_ops_opattrs_matmul_schema() {
    assert_eq!(opattrs::encode_matmul(0, MathFidelityOrd::BitStable, PrecisionTag::None), vec![0x00, 0x01, 0x01], "the resolved defaults");
    // f32 compute with an f64 accumulator, reduced mantissa, 3xtf32: f64 is ordinal 4.
    let b = opattrs::encode_matmul(4, MathFidelityOrd::ReducedMantissa, PrecisionTag::ThreeXTf32);
    assert_eq!(b, vec![0x04, 0x02, 0x02]);
    assert_eq!(opattrs::decode_matmul(&b), Ok((4, MathFidelityOrd::ReducedMantissa, PrecisionTag::ThreeXTf32)));
    // integer accumulator i32 = ordinal 9.
    assert_eq!(opattrs::encode_matmul(9, MathFidelityOrd::BitStable, PrecisionTag::None), vec![0x09, 0x01, 0x01]);
    for len in [0usize, 1, 2, 4] {
        assert_eq!(opattrs::decode_matmul(&vec![0u8; len]), Err(Decline::TruncatedBlob { need: 3, got: len }));
    }
    assert_eq!(opattrs::decode_matmul(&[0, 0, 1]), Err(Decline::ReservedZeroOrdinal { field: "math_fidelity" }));
    assert_eq!(opattrs::decode_matmul(&[0, 1, 0]), Err(Decline::ReservedZeroOrdinal { field: "precision_tag" }));
    assert!(matches!(opattrs::decode_matmul(&[0, 3, 1]), Err(Decline::FieldOutOfRange { field: "math_fidelity", .. })));
    assert!(matches!(opattrs::decode_matmul(&[0, 1, 4]), Err(Decline::FieldOutOfRange { field: "precision_tag", .. })));
    assert!(matches!(opattrs::decode_matmul(&[0xFF, 1, 1]), Err(Decline::FieldOutOfRange { field: "accumulator", .. })));
    // f8e4m3fnuz (ordinal 15) and f8e6m2 (19) are reserved dtypes: never an accumulator.
    assert!(matches!(opattrs::decode_matmul(&[15, 1, 1]), Err(Decline::FieldOutOfRange { field: "accumulator", .. })));
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(spec.contains("**KISS-OPS-6.19-0049** — The `matmul` OpAttrs blob MUST be exactly `accumulator`"));
    assert!(spec.contains("`reduce_mean`, `reduce_norm2`, `logsumexp`, `any`, `all`, `argmax`, `argmin`, and `matmul`"), "matmul is in the carrier list");
}

// Backs[enforced]: KISS-OPS-6.17-0010 — a float compute dtype takes a float accumulator at least as wide, an integer compute dtype takes only a signed 32- or 64-bit integer accumulator at least as wide as the compute dtype (never a float one, never unsigned, never narrower), and the integer fold is exact: the same bits for every order and a 32-bit two's-complement wrap on overflow, so it is exact-byte
#[test]
fn test_ops_matmul_integer_accumulator() {
    // ordinals: f16 1, bf16 2, f32 3, f64 4, i8 5, i16 6, u8 7, u16 8, i32 9, i64 10, u32 11, u64 12.
    let ok = |c: &str, a: u8| matmul_attrs_check(c, a, 1, 1, None);
    assert_eq!(ok("i8", 9), Ok(()), "int8 into an int32 accumulator");
    assert_eq!(ok("u8", 9), Ok(()));
    assert_eq!(ok("i8", 10), Ok(()));
    assert_eq!(ok("i16", 9), Ok(()));
    assert_eq!(ok("i4", 9), Ok(()));
    assert_eq!(ok("i32", 9), Ok(()), "the same signed width");
    assert_eq!(ok("i32", 10), Ok(()));
    assert_eq!(ok("i8", 0), Ok(()), "the diagonal default");
    assert_eq!(ok("i8", 5), Err(MatmulAttrDecline::IntegerAccumulator), "an i8 accumulator is narrower than 32 bits");
    assert_eq!(ok("i8", 6), Err(MatmulAttrDecline::IntegerAccumulator), "i16 is narrower than 32 bits");
    assert_eq!(ok("i8", 11), Err(MatmulAttrDecline::IntegerAccumulator), "u32 is not signed");
    assert_eq!(ok("i8", 3), Err(MatmulAttrDecline::IntegerAccumulator), "a float accumulator for an integer compute dtype is not requestable");
    assert_eq!(ok("i64", 9), Err(MatmulAttrDecline::IntegerAccumulator), "narrower than the compute dtype");
    assert_eq!(ok("u32", 9), Err(MatmulAttrDecline::IntegerAccumulator), "unsigned compute into a same-width signed accumulator");
    assert_eq!(ok("u16", 9), Ok(()));
    // a float compute dtype never takes an integer accumulator, and never a narrower float.
    assert_eq!(ok("f32", 9), Err(MatmulAttrDecline::FloatAccumulator));
    assert_eq!(ok("f32", 1), Err(MatmulAttrDecline::FloatAccumulator), "f16 is narrower than f32");
    assert_eq!(ok("f16", 3), Ok(()));
    assert_eq!(ok("f16", 4), Ok(()));
    assert_eq!(ok("f8e4m3fn", 3), Ok(()));
    assert_eq!(ok("f32", 15), Err(MatmulAttrDecline::AccumulatorOrdinal(15)), "a reserved dtype ordinal");
    assert_eq!(ok("bool", 0), Err(MatmulAttrDecline::ComputeDtype("bool".into())));
    assert_eq!(ok("c64", 0), Err(MatmulAttrDecline::ComputeDtype("c64".into())));
    // exactness: every order gives the same bits, against an independent i128 sum wrapped to the width.
    let a: Vec<i64> = vec![127, -128, 99, -7, 55, 127, -128, 3];
    let b: Vec<i64> = vec![-128, -128, 127, 90, -1, 127, 127, -100];
    let exact: i128 = a.iter().zip(&b).map(|(x, y)| *x as i128 * *y as i128).sum();
    assert_eq!(int_dot_exact(&a, &b, 32), exact as i32 as i64);
    assert_eq!(int_dot_exact(&a, &b, 64), exact as i64);
    let (ar, br): (Vec<i64>, Vec<i64>) = (a.iter().rev().cloned().collect(), b.iter().rev().cloned().collect());
    assert_eq!(int_dot_exact(&a, &b, 32), int_dot_exact(&ar, &br, 32), "reduction order does not matter");
    // overflow wraps at 32 bits (two's complement), where a float accumulator would round.
    let big_a = vec![i64::from(i32::MAX); 2];
    let big_b = vec![1i64; 2];
    assert_eq!(int_dot_exact(&big_a, &big_b, 32), (2 * i64::from(i32::MAX)) as i32 as i64);
    assert_eq!(int_dot_exact(&big_a, &big_b, 64), 2 * i64::from(i32::MAX));
}

// Backs[enforced]: KISS-OPS-6.17-0011 — a precision tag other than none needs reduced-mantissa-permitted, applies only to the compute dtype it names (3xtf32 to f32, fp8-fast-accumulate to an fp8 element dtype), and a tagged cell with no declared bound is refused; the tag is in the maintainer-owned vocabulary and the spec pins its name and duty, not a recipe
#[test]
fn test_ops_precision_tag() {
    let chk = |c: &str, fid: u8, tag: u8, bound: Option<u32>| matmul_attrs_check(c, 0, fid, tag, bound);
    assert_eq!(chk("f32", 1, 1, None), Ok(()), "no tag, no bound needed");
    assert_eq!(chk("f32", 2, 2, Some(4)), Ok(()), "3xtf32 over f32, reduced mantissa, bound declared");
    assert_eq!(chk("f8e4m3fn", 2, 3, Some(64)), Ok(()));
    assert_eq!(chk("f8e5m2", 2, 3, Some(64)), Ok(()));
    assert_eq!(chk("f32", 1, 2, Some(4)), Err(MatmulAttrDecline::TagNeedsReducedMantissa), "a tag under bit-stable");
    assert_eq!(chk("f16", 2, 2, Some(4)), Err(MatmulAttrDecline::TagComputeDtype), "3xtf32 is for f32");
    assert_eq!(chk("f32", 2, 3, Some(4)), Err(MatmulAttrDecline::TagComputeDtype), "fp8-fast-accumulate is for an fp8 element");
    assert_eq!(chk("f32", 2, 2, None), Err(MatmulAttrDecline::TagWithoutBound), "a tag without a declared bound is refused");
    assert_eq!(chk("f8e4m3fn", 2, 3, None), Err(MatmulAttrDecline::TagWithoutBound));
    // the vocabulary: ordinals 1..3, none = 1.
    assert_eq!([PrecisionTag::None.ordinal(), PrecisionTag::ThreeXTf32.ordinal(), PrecisionTag::Fp8FastAccumulate.ordinal()], [1, 2, 3]);
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(spec.contains("**KISS-OPS-6.17-0011** — The **precision tag** of a `matmul` is a named reduced-precision strategy"));
    assert!(spec.contains("| `precision_tag` | enum, `u8` | `0`=RESERVED, `1`=none, `2`=3xtf32, `3`=fp8-fast-accumulate |"));
}
