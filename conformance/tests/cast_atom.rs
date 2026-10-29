// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.21 (the conversion atom `cast`), its OpAttrs encoding (§6.19-0041/-0042) and the shape rules
//! of `cast` and `concat` (§6.20-0011). The values here are the ones the clauses pin; the cases Appendix D
//! item 12 leaves open are asserted to be reported as "no pinned value" (`None`), never given one.

use kiss_conformance::assert_golden;
use kiss_conformance::cast::{self, CastDecline};
use kiss_conformance::fp;
use kiss_conformance::opattrs::{self, CastRounding as R, CastSaturation as S, Decline};
use kiss_conformance::structural::concat_f32;

const ALL_ROUNDING: [R; 4] = [R::NearestEven, R::TowardZero, R::TowardNegInf, R::TowardPosInf];

/// `1 + 2^-8` in `f32`: exactly halfway between the `bf16` neighbours `1.0` (0x3F80) and `1 + 2^-7` (0x3F81).
fn tie_low() -> f32 {
    f32::from_bits(0x3F80_8000)
}

// Backs[enforced]: KISS-OPS-6.21-0002 — each mode rounds the exact value once, in its own direction
#[test]
fn test_ops_cast_rounding_modes() {
    let up = |x: f32, r| cast::f32_to_bf16(x, r, S::None);
    // an exact tie below an even neighbour: nearest-even goes to the even one (1.0), the directed modes go their way.
    let x = tie_low();
    assert_eq!(up(x, R::NearestEven), 0x3F80);
    assert_eq!(up(x, R::TowardZero), 0x3F80);
    assert_eq!(up(x, R::TowardNegInf), 0x3F80);
    assert_eq!(up(x, R::TowardPosInf), 0x3F81);
    // the same magnitude negated: toward-negative-infinity now bumps the magnitude, toward-positive does not.
    let n = -x;
    assert_eq!(up(n, R::NearestEven), 0xBF80);
    assert_eq!(up(n, R::TowardZero), 0xBF80);
    assert_eq!(up(n, R::TowardNegInf), 0xBF81);
    assert_eq!(up(n, R::TowardPosInf), 0xBF80);
    // a tie above an odd neighbour (0x3F81): nearest-even goes UP to the even one (0x3F82); toward-zero truncates.
    let odd = f32::from_bits(0x3F81_8000);
    assert_eq!(up(odd, R::NearestEven), 0x3F82);
    assert_eq!(up(odd, R::TowardZero), 0x3F81);
    // an exact conversion is unchanged by the mode: 1.5 is 0x3FC0 in bf16 under all four.
    for r in ALL_ROUNDING {
        assert_eq!(up(1.5, r), 0x3FC0, "{r:?}");
    }
}

// Backs[enforced]: KISS-OPS-6.21-0002 — one rounding of the EXACT source, not two through an f32 stage
#[test]
fn cast_rounds_the_exact_source_once() {
    // 1 + 2^-8 + 2^-30: above the tie in f64, so a single rounding goes UP to 0x3F81. Staged through f32 the
    // 2^-30 term is lost, the value lands exactly on the tie, and nearest-even then goes DOWN to 0x3F80.
    let x = 1.0f64 + 2f64.powi(-8) + 2f64.powi(-30);
    let direct = cast::f64_to_bf16_nearest_even(x).unwrap();
    let staged = cast::f32_to_bf16(x as f32, R::NearestEven, S::None);
    assert_eq!(direct, 0x3F81);
    assert_eq!(staged, 0x3F80, "the control: staging through f32 DOES change the answer");
    assert_ne!(direct, staged);
}

// Backs[enforced]: KISS-OPS-6.21-0002 — fp8 and MX destinations accept nearest-even only
#[test]
fn cast_fixed_rounding_destinations_decline_other_modes() {
    for dst in ["f8e4m3fn", "f8e5m2", "f4e2m1", "f6e2m3", "f6e3m2"] {
        assert_eq!(cast::check_cast("f32", dst, R::NearestEven, S::Clamp), Ok(()), "{dst}");
        for r in [R::TowardZero, R::TowardNegInf, R::TowardPosInf] {
            assert_eq!(cast::check_cast("f32", dst, r, S::Clamp), Err(CastDecline::RoundingNotDefined), "{dst} {r:?}");
        }
    }
    // control: bf16 takes every mode
    for r in ALL_ROUNDING {
        assert_eq!(cast::check_cast("f32", "bf16", r, S::None), Ok(()));
    }
}

// Backs[enforced]: KISS-OPS-6.21-0003 — overflow: none follows the rounding direction, clamp takes the largest finite
#[test]
fn test_ops_cast_saturation() {
    let c = |x: f32, r, s| cast::f32_to_bf16(x, r, s);
    // f32::MAX is past the largest finite bf16 (0x7F7F): nearest-even rounds it up to infinity ...
    assert_eq!(c(f32::MAX, R::NearestEven, S::None), 0x7F80);
    // ... clamp takes the largest finite instead, with the value's sign ...
    assert_eq!(c(f32::MAX, R::NearestEven, S::Clamp), 0x7F7F);
    assert_eq!(c(-f32::MAX, R::NearestEven, S::Clamp), 0xFF7F);
    // ... and a direction that points away from the overflow side stops at the largest finite value with no clamp.
    assert_eq!(c(f32::MAX, R::TowardZero, S::None), 0x7F7F);
    assert_eq!(c(f32::MAX, R::TowardNegInf, S::None), 0x7F7F);
    assert_eq!(c(f32::MAX, R::TowardPosInf, S::None), 0x7F80);
    // an infinity source: clamp saturates it, none keeps it
    assert_eq!(c(f32::INFINITY, R::NearestEven, S::Clamp), 0x7F7F);
    assert_eq!(c(f32::NEG_INFINITY, R::NearestEven, S::None), 0xFF80);
    // a NaN into a float that has a NaN stays a NaN and keeps its sign, whatever the attributes
    let nan = c(f32::from_bits(0xFFC0_0001), R::NearestEven, S::Clamp);
    assert!(nan & 0x7F80 == 0x7F80 && nan & 0x007F != 0 && nan & 0x8000 != 0, "{nan:#06x}");
    // fp8 destinations pin saturation themselves (6.16-0004/-0005): 1000 -> 448, 1e6 -> 57344
    assert_eq!(fp::e4m3_to_f32(fp::f32_to_e4m3(1000.0)), 448.0);
    assert_eq!(fp::e5m2_to_f32(fp::f32_to_e5m2(1.0e6)), 57344.0);
    // integers: clamp takes the nearest bound, none wraps modulo 2^width
    assert_eq!(cast::int_to_int(300, 8, true, S::Clamp), 127);
    assert_eq!(cast::int_to_int(300, 8, true, S::None), 44);
    assert_eq!(cast::int_to_int(-129, 8, true, S::None), 127);
    assert_eq!(cast::int_to_int(-1, 8, false, S::None), 255);
    assert_eq!(cast::f32_to_int(300.0, R::NearestEven, S::Clamp, 8, true), Some(127));
    assert_eq!(cast::f32_to_int(-300.0, R::NearestEven, S::Clamp, 8, true), Some(-128));
    assert_eq!(cast::f32_to_int(2.5, R::NearestEven, S::Clamp, 8, true), Some(2));
    assert_eq!(cast::f32_to_int(2.5, R::TowardPosInf, S::Clamp, 8, true), Some(3));
}

// Backs[enforced]: KISS-OPS-6.21-0003 — a destination that pins saturation cannot be requested with none
#[test]
fn cast_saturating_destinations_require_clamp() {
    for dst in ["f8e4m3fn", "f8e5m2", "f4e2m1", "f6e2m3", "f6e3m2"] {
        assert_eq!(cast::check_cast("f32", dst, R::NearestEven, S::None), Err(CastDecline::SaturationRequired), "{dst}");
    }
    assert_eq!(cast::check_cast("f32", "i32", R::TowardZero, S::None), Ok(()), "control: i32 may wrap");
}

// Backs[enforced]: KISS-OPS-6.21-0001 — one operand in, same shape out, destination dtype from the attribute
#[test]
fn test_ops_cast_atom_shape_and_dtype() {
    let src = [1.5f32, -2.25, 0.0, -0.0, 65504.0, 1.0e-3];
    let out: Vec<u16> = src.iter().map(|x| cast::f32_to_bf16(*x, R::NearestEven, S::None)).collect();
    assert_eq!(out.len(), src.len(), "element for element");
    assert_eq!(out[3], 0x8000, "a signed zero keeps its sign");
    // a complex source or destination is not defined here
    assert_eq!(cast::check_cast("c64", "f32", R::NearestEven, S::None), Err(CastDecline::UndefinedPair));
    assert_eq!(cast::check_cast("f32", "c128", R::NearestEven, S::None), Err(CastDecline::UndefinedPair));
    // the attribute that names the destination is never the compute-dtype diagonal
    assert_eq!(opattrs::decode_cast(&[0, 1, 1]), Err(Decline::ReservedZeroOrdinal { field: "target_dtype" }));
}

// harness-internal: negative control for the unpinned cases, no clause asserted
#[test]
fn unpinned_cases_report_no_pinned_value() {
    // §6.21-0004 / App. D item 12: a NaN into an integer, and an out-of-range float under saturation = none
    assert_eq!(cast::f32_to_int(f32::NAN, R::NearestEven, S::Clamp, 32, true), None);
    assert_eq!(cast::f32_to_int(1.0e10, R::NearestEven, S::None, 16, true), None);
    assert_eq!(cast::f32_to_int(1.0e10, R::NearestEven, S::Clamp, 16, true), Some(32767), "control: clamp has a value");
}

// Backs[enforced]: KISS-OPS-6.19-0041 — the two cast enums are frozen u8 ordinals with 0 reserved
#[test]
fn test_ops_opattrs_cast_enums() {
    assert_eq!(R::NearestEven.ordinal(), 1);
    assert_eq!(R::TowardZero.ordinal(), 2);
    assert_eq!(R::TowardNegInf.ordinal(), 3);
    assert_eq!(R::TowardPosInf.ordinal(), 4);
    assert_eq!(S::None.ordinal(), 1);
    assert_eq!(S::Clamp.ordinal(), 2);
    // 0 and anything past the set are typed declines, in either field
    assert_eq!(opattrs::decode_cast(&[2, 0, 1]), Err(Decline::ReservedZeroOrdinal { field: "rounding" }));
    assert_eq!(opattrs::decode_cast(&[2, 5, 1]), Err(Decline::ReservedZeroOrdinal { field: "rounding" }));
    assert_eq!(opattrs::decode_cast(&[2, 1, 0]), Err(Decline::ReservedZeroOrdinal { field: "saturation" }));
    assert_eq!(opattrs::decode_cast(&[2, 1, 3]), Err(Decline::ReservedZeroOrdinal { field: "saturation" }));
}

// Backs[enforced]: KISS-OPS-6.19-0042 — exact blobs, and the declines a reader owes
#[test]
fn test_ops_opattrs_cast_concat_schema() {
    // cast to bf16 (ordinal 2), nearest-even, none
    assert_golden("KISS-OPS-6.19-0042", "cast_bf16_rne_none", &opattrs::encode_cast(2, R::NearestEven, S::None), "02 01 01");
    // cast to f8e4m3fn (ordinal 14), nearest-even, clamp
    assert_golden("KISS-OPS-6.19-0042", "cast_e4m3_rne_clamp", &opattrs::encode_cast(14, R::NearestEven, S::Clamp), "0E 01 02");
    // concat along axis 1
    assert_golden("KISS-OPS-6.19-0042", "concat_axis1", &opattrs::encode_concat(1), "01");
    // round trips
    assert_eq!(opattrs::decode_cast(&[2, 1, 1]), Ok((2, R::NearestEven, S::None)));
    assert_eq!(opattrs::decode_concat(&[7]), Ok(7));
    // declines: a reserved or unknown dtype ordinal, a short blob, an out-of-range axis
    assert_eq!(opattrs::decode_cast(&[15, 1, 1]), Err(Decline::FieldOutOfRange { field: "target_dtype", value: 15, max: 27 }), "f8e4m3fnuz is reserved");
    assert_eq!(opattrs::decode_cast(&[28, 1, 1]), Err(Decline::FieldOutOfRange { field: "target_dtype", value: 28, max: 27 }));
    assert_eq!(opattrs::decode_cast(&[2, 1]), Err(Decline::TruncatedBlob { need: 3, got: 2 }));
    assert_eq!(opattrs::decode_concat(&[8]), Err(Decline::FieldOutOfRange { field: "axis", value: 8, max: 8 }));
    assert_eq!(opattrs::decode_concat(&[]), Err(Decline::TruncatedBlob { need: 1, got: 0 }));
}

// Backs[enforced]: KISS-OPS-6.20-0011 — cast is SameAs(0); concat sums the axis extent; no view op exists
#[test]
fn test_shape_rule_cast_concat_no_view_ops() {
    // concat: operand 0's shape with the axis extent replaced by the sum
    let a = vec![0.0f32; 2 * 3];
    let b = vec![1.0f32; 2 * 5];
    let (_, shape) = concat_f32(&[(&a, &[2, 3]), (&b, &[2, 5])], 1).unwrap();
    assert_eq!(shape, vec![2, 8]);
    let (_, shape0) = concat_f32(&[(&a, &[2, 3]), (&a, &[2, 3]), (&a, &[2, 3])], 0).unwrap();
    assert_eq!(shape0, vec![6, 3]);
    // no transpose / permute / unsqueeze op token exists in the generated op manifest (control: concat and cast do)
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/op_manifest.json")).unwrap();
    for present in ["\"concat\"", "\"cast\""] {
        assert!(manifest.contains(present), "control: {present} must be a manifest op");
    }
    for absent in ["\"transpose\"", "\"permute\"", "\"unsqueeze\""] {
        assert!(!manifest.contains(absent), "{absent} must not be a manifest op");
    }
}
