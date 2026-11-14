// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.19-0049 (the `matmul` OpAttrs blob), §6.17-0010 (the integer accumulator), §6.17-0011 (the requested
//! accuracy bound) and §6.17-0012 (a reduced-precision cell declares its tier): CireSnave 2026-11-12, boards 426 6
//! "A" and 427, and 2026-11-14, board 443 (which removed the 0.17.0 `precision_tag`).

use kiss_conformance::opattrs::{self, AccuracyKind, Decline, MathFidelityOrd};
use kiss_conformance::structural::{accuracy_request_admits, int_dot_exact, matmul_attrs_check, reduced_cell_declares_tier, MatmulAttrDecline, ReducedPrecisionDecline};

// Backs[enforced]: KISS-OPS-6.19-0049 — the matmul blob is exactly accumulator, math_fidelity, accuracy_kind (u8) and accuracy_value (u64 LE), eleven bytes, emitted explicitly (defaults 00 01 01 + 8 zero bytes); it round-trips and a wrong length, a reserved zero, an out-of-range ordinal, a value its kind forbids and a bad accumulator each decline.
#[test]
fn test_ops_opattrs_matmul_schema() {
    let z = [0u8; 8];
    let with = |head: [u8; 3], tail: [u8; 8]| head.iter().chain(tail.iter()).copied().collect::<Vec<u8>>();
    assert_eq!(opattrs::encode_matmul(0, MathFidelityOrd::BitStable, AccuracyKind::None, 0), with([0, 1, 1], z), "the resolved defaults");
    // f32 compute, f64 accumulator (ordinal 4), reduced mantissa, at most 4 ULP.
    let b = opattrs::encode_matmul(4, MathFidelityOrd::ReducedMantissa, AccuracyKind::MaxUlp, 4);
    assert_eq!(b, with([4, 2, 2], [4, 0, 0, 0, 0, 0, 0, 0]));
    assert_eq!(opattrs::decode_matmul(&b), Ok((4, MathFidelityOrd::ReducedMantissa, AccuracyKind::MaxUlp, 4)));
    // a relative bound of 2^-10: the binary64 0x3F50000000000000, little-endian 00 00 00 00 00 00 50 3F.
    let r = opattrs::encode_matmul(0, MathFidelityOrd::BitStable, AccuracyKind::MaxRelative, 0x3F50_0000_0000_0000);
    assert_eq!(r, with([0, 1, 3], [0, 0, 0, 0, 0, 0, 0x50, 0x3F]));
    assert_eq!(opattrs::decode_matmul(&r), Ok((0, MathFidelityOrd::BitStable, AccuracyKind::MaxRelative, 0x3F50_0000_0000_0000)));
    // integer accumulator i32 = ordinal 9.
    assert_eq!(opattrs::encode_matmul(9, MathFidelityOrd::BitStable, AccuracyKind::None, 0), with([9, 1, 1], z));
    for len in [0usize, 1, 2, 3, 10, 12] {
        assert_eq!(opattrs::decode_matmul(&vec![0u8; len]), Err(Decline::TruncatedBlob { need: 11, got: len }), "length {len}");
    }
    assert_eq!(opattrs::decode_matmul(&with([0, 0, 1], z)), Err(Decline::ReservedZeroOrdinal { field: "math_fidelity" }));
    assert_eq!(opattrs::decode_matmul(&with([0, 1, 0], z)), Err(Decline::ReservedZeroOrdinal { field: "accuracy_kind" }));
    assert!(matches!(opattrs::decode_matmul(&with([0, 3, 1], z)), Err(Decline::FieldOutOfRange { field: "math_fidelity", .. })));
    assert!(matches!(opattrs::decode_matmul(&with([0, 1, 5], z)), Err(Decline::FieldOutOfRange { field: "accuracy_kind", .. })));
    assert!(matches!(opattrs::decode_matmul(&with([0xFF, 1, 1], z)), Err(Decline::FieldOutOfRange { field: "accumulator", .. })));
    // f8e4m3fnuz (ordinal 15) and f8e6m2 (19) are reserved dtypes: never an accumulator.
    assert!(matches!(opattrs::decode_matmul(&with([15, 1, 1], z)), Err(Decline::FieldOutOfRange { field: "accumulator", .. })));
    // a value the kind forbids: non-zero under none; NaN, infinite or negative under a float bound.
    assert_eq!(opattrs::decode_matmul(&with([0, 1, 1], [1, 0, 0, 0, 0, 0, 0, 0])), Err(Decline::AccuracyValue { kind: 1 }));
    for bits in [f64::NAN.to_bits(), f64::INFINITY.to_bits(), (-1.0f64).to_bits(), (-0.0f64).to_bits()] {
        for kind in [3u8, 4] {
            assert_eq!(opattrs::decode_matmul(&with([0, 1, kind], bits.to_le_bytes())), Err(Decline::AccuracyValue { kind }), "{bits:#x} kind {kind}");
        }
    }
    assert!(opattrs::decode_matmul(&with([0, 1, 4], 0.0f64.to_bits().to_le_bytes())).is_ok(), "a zero absolute bound (exact) is legal");
    assert!(opattrs::decode_matmul(&with([0, 1, 2], u64::MAX.to_le_bytes())).is_ok(), "any u64 count is legal under max_ulp");
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(spec.contains("**KISS-OPS-6.19-0049** — The `matmul` OpAttrs blob MUST be exactly `accumulator` (`u8`), `math_fidelity` (`u8`),"));
    assert!(spec.contains("`reduce_mean`, `reduce_norm2`, `logsumexp`, `any`, `all`, `argmax`, `argmin`, and `matmul`"), "matmul is in the carrier list");
}

// Backs[enforced]: KISS-OPS-6.17-0010 — a float compute dtype takes a float accumulator at least as wide, an integer compute dtype takes only a signed 32- or 64-bit integer accumulator at least as wide as the compute dtype (never a float one, never unsigned, never narrower), and the integer fold is exact: the same bits for every order and a 32-bit two's-complement wrap on overflow, so it is exact-byte
#[test]
fn test_ops_matmul_integer_accumulator() {
    // ordinals: f16 1, bf16 2, f32 3, f64 4, i8 5, i16 6, u8 7, u16 8, i32 9, i64 10, u32 11, u64 12.
    let ok = |c: &str, a: u8| matmul_attrs_check(c, a);
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

// Backs[enforced]: KISS-OPS-6.17-0011 — a requested accuracy bound has the Contract tier shape (none, max_ulp, max_relative, max_absolute); a kernel is admitted iff it declares a tier of the same kind no looser than the request, a request of kind none admits every kernel, kinds are not convertible, and the 0.17.0 precision_tag vocabulary is gone.
#[test]
fn test_ops_requested_accuracy_bound() {
    use AccuracyKind::{MaxAbsolute, MaxRelative, MaxUlp, None as NoBound};
    // the vocabulary: ordinals 1..4, none = 1.
    assert_eq!([NoBound.ordinal(), MaxUlp.ordinal(), MaxRelative.ordinal(), MaxAbsolute.ordinal()], [1, 2, 3, 4]);
    let rel = |x: f64| (MaxRelative, x.to_bits());
    // a request of kind none admits every kernel, including one that declares nothing.
    assert!(accuracy_request_admits((NoBound, 0), &[]));
    assert!(accuracy_request_admits((NoBound, 0), &[(MaxUlp, 99)]));
    // same kind, declared no looser than requested (<=), equality admitted.
    assert!(accuracy_request_admits((MaxUlp, 4), &[(MaxUlp, 4)]));
    assert!(accuracy_request_admits((MaxUlp, 4), &[(MaxUlp, 2)]));
    assert!(!accuracy_request_admits((MaxUlp, 4), &[(MaxUlp, 5)]), "looser than requested");
    assert!(accuracy_request_admits(rel(1e-3), &[rel(1e-4)]));
    assert!(!accuracy_request_admits(rel(1e-3), &[rel(2e-3)]));
    // kinds are not convertible: a ULP tier does not answer a relative request, and no tier answers none.
    assert!(!accuracy_request_admits(rel(1e-3), &[(MaxUlp, 0)]));
    assert!(!accuracy_request_admits((MaxUlp, 4), &[]));
    // several declared kinds: any one of the requested kind that is no looser suffices.
    assert!(accuracy_request_admits(rel(1e-3), &[(MaxUlp, 99), rel(5e-4)]));
    // a declared NaN is never admitted (it compares false).
    assert!(!accuracy_request_admits(rel(1e-3), &[(MaxRelative, f64::NAN.to_bits())]));
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(spec.contains("**KISS-OPS-6.17-0011** — The accuracy a `matmul` requester needs, if any, MUST be stated as a **requested accuracy bound**"));
    assert!(spec.contains("| `accuracy_kind` | enum, `u8` | `0`=RESERVED, `1`=none, `2`=max_ulp, `3`=max_relative, `4`=max_absolute |"));
    assert!(!spec.contains("| `precision_tag` |"), "the 0.17.0 precision_tag vocabulary row is removed");
}

// Backs[enforced]: KISS-OPS-6.17-0012 — a kernel advertising a reduced-mantissa-permitted cell, for any op, must declare a tier for it; a bit-stable cell may declare none.
#[test]
fn test_ops_reduced_precision_declares_tier() {
    assert_eq!(reduced_cell_declares_tier(true, 0), Err(ReducedPrecisionDecline::NoTier), "an rm cell with no tier is refused");
    assert_eq!(reduced_cell_declares_tier(true, 1), Ok(()));
    assert_eq!(reduced_cell_declares_tier(true, 3), Ok(()));
    assert_eq!(reduced_cell_declares_tier(false, 0), Ok(()), "a bit-stable cell with no finite bound may declare no tier");
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(spec.contains("**KISS-OPS-6.17-0012** — A kernel that **advertises a cell whose MathFidelity is reduced-mantissa-permitted**"));
    let contract = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/contract.md")).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(contract.contains("this allowance does not apply to a cell that advertises `reduced-mantissa-permitted`, which MUST declare a tier"));
}
