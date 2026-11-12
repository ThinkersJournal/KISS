// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.21-0005 (`bitcast`) and its OpAttrs blob (§6.19-0048), CireSnave 2026-11-12, board 427 "B": add
//! `bitcast` to the primitive floor now. A pure bit move between same-width dtypes: nothing computed, nothing
//! canonicalised.

use kiss_conformance::opattrs::{self, Decline};
use kiss_conformance::structural::{bitcast_bits, bitcast_width, BitcastDecline};

fn spec() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md"))
        .expect("read spec/ops.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Backs[enforced]: KISS-OPS-6.21-0005 — bitcast reinterprets the bits of a same-width integer or float dtype and nothing else: a NaN keeps its payload, quiet/signalling bit and sign, a signed zero keeps its sign, a round trip is the identity on every pattern; sub-byte, MX, f8e8m0, reserved, complex and bool dtypes and a width change are typed declines; it is on the floor and in the manifest
#[test]
fn test_ops_bitcast_semantics() {
    // f32 <-> u32 / i32: the bits are unchanged, the reading differs.
    let snan = 0x7F80_0001u64; // a signalling NaN with the lowest payload bit
    let qnan = 0x7FC0_0001u64; // a quiet NaN with payload 1
    let neg_zero = 0x8000_0000u64;
    let sub = 0x0000_0001u64; // the smallest subnormal
    for bits in [snan, qnan, neg_zero, sub, 0x3F80_0000, 0xFF80_0000, 0x7F7F_FFFF] {
        assert_eq!(bitcast_bits("f32", "u32", bits), Ok(bits), "{bits:#x}");
        assert_eq!(bitcast_bits("u32", "f32", bits), Ok(bits), "round trip {bits:#x}");
        assert_eq!(bitcast_bits("f32", "i32", bits), Ok(bits));
    }
    // the signed reading is two's complement: the same bits are a negative i32.
    assert_eq!(f32::from_bits(0x8000_0000).is_sign_negative(), true);
    assert_eq!(0xFF80_0000u32 as i32, -8_388_608, "-inf read as i32");
    // the other widths: f16/bf16 <-> 16-bit ints, f64 <-> 64-bit ints, fp8 <-> 8-bit ints.
    assert_eq!(bitcast_bits("f16", "i16", 0x7E01), Ok(0x7E01), "an f16 NaN keeps its payload");
    assert_eq!(bitcast_bits("bf16", "u16", 0xFFC1), Ok(0xFFC1));
    assert_eq!(bitcast_bits("f64", "u64", 0x7FF8_0000_0000_0001), Ok(0x7FF8_0000_0000_0001));
    assert_eq!(bitcast_bits("f64", "i64", u64::MAX), Ok(u64::MAX));
    assert_eq!(bitcast_bits("f8e4m3fn", "u8", 0x7F), Ok(0x7F), "the e4m3fn NaN pattern");
    assert_eq!(bitcast_bits("f8e5m2", "i8", 0xFC), Ok(0xFC));
    // identity, and ints among themselves (same width).
    assert_eq!(bitcast_bits("f32", "f32", snan), Ok(snan));
    assert_eq!(bitcast_bits("i16", "u16", 0xFFFF), Ok(0xFFFF));

    // a different width is a typed decline, both ways.
    assert_eq!(bitcast_bits("f32", "u16", 0), Err(BitcastDecline::Width { src: 32, dst: 16 }));
    assert_eq!(bitcast_bits("u64", "f32", 0), Err(BitcastDecline::Width { src: 64, dst: 32 }));
    // dtypes the op refuses: sub-byte, MX, f8e8m0, reserved floats, complex, bool, unknown.
    for d in ["i4", "u4", "b1", "f4e2m1", "f6e2m3", "f6e3m2", "f8e8m0", "f8e4m3fnuz", "f8e5m2fnuz", "f8e6m2", "c64", "c128", "bool", "nope"] {
        assert_eq!(bitcast_bits(d, "u8", 0), Err(BitcastDecline::Dtype(d.to_string())), "src {d}");
        assert_eq!(bitcast_bits("u8", d, 0), Err(BitcastDecline::Dtype(d.to_string())), "dst {d}");
    }
    assert_eq!(bitcast_width("f64"), Some(64));

    // the spec states it: a floor member, in the table, an exact-byte primitive, with the Draft-change note.
    let text = spec();
    assert!(text.contains("**KISS-OPS-6.21-0005** — `bitcast` MUST take exactly one input operand"));
    assert!(text.contains("`iota`, `bitcast`. *Test:* `test_ops_primitive_floor_set`"));
    assert!(text.contains("Draft change (CireSnave 2026-11-12, board 427 \"B\""));
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/op_manifest.json")).unwrap();
    assert!(manifest.contains("\"bitcast\""), "bitcast must be a manifest op");
}

// Backs[enforced]: KISS-OPS-6.19-0048 — the bitcast blob is one target_dtype byte; it round-trips and a zero, out-of-table or reserved ordinal and a wrong-length blob are typed declines
#[test]
fn test_ops_opattrs_bitcast_schema() {
    // f32 is ordinal 3, i32 is 9.
    assert_eq!(opattrs::encode_bitcast(3), vec![0x03]);
    assert_eq!(opattrs::decode_bitcast(&[0x09]), Ok(9));
    assert_eq!(opattrs::decode_bitcast(&[]), Err(Decline::TruncatedBlob { need: 1, got: 0 }));
    assert_eq!(opattrs::decode_bitcast(&[0x03, 0x00]), Err(Decline::TruncatedBlob { need: 1, got: 2 }));
    assert_eq!(opattrs::decode_bitcast(&[0x00]), Err(Decline::ReservedZeroOrdinal { field: "target_dtype" }));
    assert!(matches!(opattrs::decode_bitcast(&[0xFF]), Err(Decline::FieldOutOfRange { field: "target_dtype", .. })));
}
