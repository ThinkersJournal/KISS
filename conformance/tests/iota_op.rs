// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.11-0025 (`iota`) and its OpAttrs blob (§6.19-0047), CireSnave 2026-11-12, board 422 "A": add the
//! `iota` operation (a counting sequence) to KISS-Ops now, with test vectors. No operand; a static shape; the
//! count along one axis, exactly, in a stated dtype.

use kiss_conformance::opattrs::{self, Decline};
use kiss_conformance::structural::{iota_max_exact, iota_values, IotaDecline};

fn spec() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md"))
        .expect("read spec/ops.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Backs[enforced]: KISS-OPS-6.11-0025 — iota has no operand and counts along one axis of a static shape, constant along the others, exactly in the stated dtype; a dtype that cannot hold the largest count, a non-count dtype, a zero extent, a bad axis or rank are typed declines; arange, a triangular mask and a one-hot are compositions of it
#[test]
fn test_ops_iota_semantics() {
    // rank 1: arange(0, 5).
    assert_eq!(iota_values("i32", &[5], 0), Ok(vec![0, 1, 2, 3, 4]));
    // rank 2 [2,3]: axis 0 counts rows, axis 1 counts columns, each constant along the other.
    assert_eq!(iota_values("f32", &[2, 3], 0), Ok(vec![0, 0, 0, 1, 1, 1]));
    assert_eq!(iota_values("f32", &[2, 3], 1), Ok(vec![0, 1, 2, 0, 1, 2]));
    // rank 3 [2,2,2], the middle axis.
    assert_eq!(iota_values("u8", &[2, 2, 2], 1), Ok(vec![0, 0, 1, 1, 0, 0, 1, 1]));
    // an extent of 1 counts only 0.
    assert_eq!(iota_values("i64", &[1, 4], 0), Ok(vec![0, 0, 0, 0]));

    // exactness: the largest count, extent - 1, must be representable.
    assert_eq!(iota_max_exact("f32"), Some(1 << 24), "last count 2^24 is exact in f32 (not materialised: 128 MiB)");
    assert_eq!(iota_values("f32", &[(1 << 24) + 2], 0), Err(IotaDecline::Unrepresentable { last: (1 << 24) + 1 }));
    assert_eq!(iota_values("bf16", &[258], 0), Err(IotaDecline::Unrepresentable { last: 257 }));
    assert!(iota_values("bf16", &[257], 0).is_ok(), "last count 256 is exact in bf16");
    assert_eq!(iota_values("i8", &[129], 0), Err(IotaDecline::Unrepresentable { last: 128 }));
    assert!(iota_values("i8", &[128], 0).is_ok());
    assert_eq!(iota_max_exact("f16"), Some(2048));

    // dtypes that are not a count: bool, sub-byte, MX, f8, complex, reserved, unknown.
    for d in ["bool", "i4", "u4", "b1", "f8e4m3fn", "f4e2m1", "c64", "c128", "f8e4m3fnuz", "nope"] {
        assert_eq!(iota_values(d, &[4], 0), Err(IotaDecline::Dtype(d.to_string())), "{d}");
    }
    // shape and axis declines.
    assert_eq!(iota_values("i32", &[], 0), Err(IotaDecline::Rank(0)));
    assert_eq!(iota_values("i32", &[1; 9], 0), Err(IotaDecline::Rank(9)));
    assert_eq!(iota_values("i32", &[3, 0], 0), Err(IotaDecline::ZeroExtent));
    assert_eq!(iota_values("i32", &[3, 4], 2), Err(IotaDecline::Axis { axis: 2, rank: 2 }));

    // compositions the clause names, computed over the reference values (no new op needed):
    // arange(start = 3, limit, delta = 2) = iota * delta + start.
    let arange: Vec<u64> = iota_values("i32", &[4], 0).unwrap().iter().map(|i| i * 2 + 3).collect();
    assert_eq!(arange, vec![3, 5, 7, 9]);
    // lower-triangular mask over a 3x3: cmp_ge(iota axis 0, iota axis 1).
    let (r, c) = (iota_values("i32", &[3, 3], 0).unwrap(), iota_values("i32", &[3, 3], 1).unwrap());
    let tri: Vec<bool> = r.iter().zip(&c).map(|(r, c)| r >= c).collect();
    assert_eq!(tri, vec![true, false, false, true, true, false, true, true, true]);
    // one-hot of index 2 over 4 classes: cmp_eq(iota, 2).
    let hot: Vec<bool> = iota_values("i32", &[4], 0).unwrap().iter().map(|i| *i == 2).collect();
    assert_eq!(hot, vec![false, false, true, false]);

    // the spec states it: a zero-operand primitive on the floor, in the op table, with the draft-change note.
    let text = spec();
    assert!(text.contains("**KISS-OPS-6.11-0025** — `iota` MUST take **no** input operand"));
    assert!(text.contains("`unsqueeze`, `iota`. *Test:* `test_ops_primitive_floor_set`"));
    assert!(text.contains("Draft change (CireSnave 2026-11-12, board 422"));
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/op_manifest.json")).unwrap();
    assert!(manifest.contains("\"iota\""), "iota must be a manifest op");
}

// Backs[enforced]: KISS-OPS-6.19-0047 — the iota blob is target_dtype, then the length-prefixed u32 extents vector, then axis; it round-trips, and a zero or reserved dtype, an empty vector, a zero extent, an axis at or above the extent count and a truncated blob are typed declines
#[test]
fn test_ops_opattrs_iota_schema() {
    // f32 is ordinal 3; extents [2, 3]; axis 1:  03 · 02 · 02 00 00 00 · 03 00 00 00 · 01
    let blob = opattrs::encode_iota(3, &[2, 3], 1);
    assert_eq!(blob, vec![0x03, 0x02, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01]);
    assert_eq!(opattrs::decode_iota(&blob), Ok((3, vec![2, 3], 1)));
    // rank 1, i32 (ordinal 9), 5 elements.
    assert_eq!(opattrs::encode_iota(9, &[5], 0), vec![0x09, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(opattrs::decode_iota(&[]), Err(Decline::TruncatedBlob { need: 1, got: 0 }));
    assert_eq!(opattrs::decode_iota(&[0x00, 0x01, 5, 0, 0, 0, 0]), Err(Decline::ReservedZeroOrdinal { field: "target_dtype" }));
    assert!(matches!(opattrs::decode_iota(&[0xFF, 0x01, 5, 0, 0, 0, 0]), Err(Decline::FieldOutOfRange { field: "target_dtype", .. })));
    assert_eq!(opattrs::decode_iota(&[0x03, 0x00, 0x00]), Err(Decline::ReservedZeroOrdinal { field: "extents" }), "an empty extent vector");
    assert_eq!(opattrs::decode_iota(&[0x03, 0x01, 0, 0, 0, 0, 0x00]), Err(Decline::ReservedZeroOrdinal { field: "extents" }), "a zero extent");
    assert!(matches!(opattrs::decode_iota(&[0x03, 0x01, 5, 0, 0, 0, 0x01]), Err(Decline::FieldOutOfRange { field: "axis", .. })), "axis 1 of a rank-1 shape");
    assert!(matches!(opattrs::decode_iota(&[0x03, 0x01, 5, 0, 0]), Err(Decline::TruncatedBlob { .. })), "a cut-off extent");
    assert!(matches!(opattrs::decode_iota(&[0x03, 0x01, 5, 0, 0, 0]), Err(Decline::TruncatedBlob { .. })), "the axis byte is missing");
    // the extent vector count is bounded by MAX_RANK.
    assert!(matches!(opattrs::decode_iota(&[0x03, 0x09]), Err(Decline::FieldOutOfRange { .. })));
}
