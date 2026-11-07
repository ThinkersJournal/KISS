// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Ops §6.11-0022 (`squeeze`), §6.11-0023 (`unsqueeze`) and their OpAttrs blob (§6.19-0046): a plain
//! `u8` axes mask, extent-1 axes only, the empty mask refused (board 392 (c), CireSnave 2026-11-07:
//! "Refuse the empty rank."), the `keepdim = 0` identity that ties both ops to §6.11-0008.

use kiss_conformance::opattrs::{self, Decline};
use kiss_conformance::opattrs_schema::parse_table;
use kiss_conformance::shape_expr::reduce_shape;
use kiss_conformance::structural::{squeeze_f32, squeeze_shape, unsqueeze_f32, unsqueeze_shape};

/// Bit patterns that must survive a pure-shape op: a signaling NaN, a quiet NaN with payload, `-0`, `+0`,
/// a subnormal.
fn awkward() -> Vec<f32> {
    vec![f32::from_bits(0x7FA0_0001), f32::from_bits(0xFFC0_1234), -0.0, 0.0, f32::from_bits(1), 1.5]
}

fn same_bits(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

// Backs[enforced]: KISS-OPS-6.11-0022 — squeeze removes exactly the named extent-1 axes, keeps the bits, and
// declines the empty mask, an out-of-range bit and a non-extent-1 axis. TEETH: an implementation that
// squeezes every extent-1 axis when the mask is empty (the PyTorch default), one that drops a named axis
// of extent > 1, and one that removes the wrong one of two equal-extent axes.
#[test]
fn test_ops_squeeze_semantics() {
    assert_eq!(squeeze_shape(&[2, 1, 4], 0b010), Ok(vec![2, 4]));
    assert_eq!(squeeze_shape(&[1, 5, 1], 0b101), Ok(vec![5]));
    assert_eq!(squeeze_shape(&[1, 1], 0b11), Ok(vec![]), "every axis of an all-ones operand gives rank 0");
    // two extent-1 axes, only one named: the mask decides, not the extents
    assert_eq!(squeeze_shape(&[1, 1, 3], 0b001), Ok(vec![1, 3]));
    assert_eq!(squeeze_shape(&[1, 1, 3], 0b010), Ok(vec![1, 3]));
    // refusals, each beside a control that a neighbouring request on the same shape works
    assert!(squeeze_shape(&[1, 3], 0).is_err(), "the empty mask is refused, not 'squeeze every extent-1 axis'");
    assert!(squeeze_shape(&[1, 3], 0b01).is_ok(), "control: the same shape with a named extent-1 axis");
    assert!(squeeze_shape(&[2, 1, 4], 0b001).is_err(), "axis 0 has extent 2");
    assert!(squeeze_shape(&[1, 3], 0b100).is_err(), "bit 2 is at or above rank 2");
    assert!(squeeze_shape(&[], 0b1).is_err(), "a scalar has no axis to squeeze");
    // the rank is the operand rank minus the popcount whatever the extents are
    for shape in [[1usize, 1, 1], [1, 1, 7], [1, 7, 1]] {
        assert_eq!(squeeze_shape(&shape, 0b001).map(|s| s.len()), Ok(2), "{shape:?}");
    }
    // bits preserved
    let data = awkward();
    let (out, shape) = squeeze_f32(&data, &[6, 1], 0b10).unwrap();
    assert_eq!(shape, vec![6]);
    assert!(same_bits(&out, &data));
    assert!(squeeze_f32(&data, &[2, 2], 0b1).is_err(), "data length must agree with the extents");
}

// Backs[enforced]: KISS-OPS-6.11-0023 — unsqueeze inserts extent-1 axes at the named OUTPUT positions, keeps
// the bits, refuses the empty mask, a bit at or above the output rank, and an output rank above MAX_RANK;
// and `unsqueeze(squeeze(x, S), S) = x`.
#[test]
fn test_ops_unsqueeze_semantics() {
    assert_eq!(unsqueeze_shape(&[3], 0b01), Ok(vec![1, 3]));
    assert_eq!(unsqueeze_shape(&[3], 0b10), Ok(vec![3, 1]));
    assert_eq!(unsqueeze_shape(&[2, 4], 0b010), Ok(vec![2, 1, 4]));
    assert_eq!(unsqueeze_shape(&[2, 4], 0b101), Ok(vec![1, 2, 1, 4]));
    assert_eq!(unsqueeze_shape(&[], 0b111), Ok(vec![1, 1, 1]), "a scalar to output rank 3");
    assert_eq!(unsqueeze_shape(&[], 0b1), Ok(vec![1]));
    assert!(unsqueeze_shape(&[3], 0).is_err(), "the empty mask is refused");
    assert!(unsqueeze_shape(&[3], 0b100).is_err(), "bit 2 is at or above output rank 2");
    assert!(unsqueeze_shape(&[1; 7], 0b11).is_err(), "7 + 2 > MAX_RANK");
    assert_eq!(unsqueeze_shape(&[1; 7], 0b1).map(|s| s.len()), Ok(8), "control: 7 + 1 = MAX_RANK is legal");
    // round trip over a grid of shapes and masks
    let mut round_trips = 0usize;
    for shape in [vec![], vec![5], vec![2, 3], vec![1, 4, 1]] {
        for mask in 1u8..=0x0F {
            if let Ok(up) = unsqueeze_shape(&shape, mask) {
                assert_eq!(squeeze_shape(&up, mask), Ok(shape.clone()), "{shape:?} {mask:#06b}");
                round_trips += 1;
            }
        }
    }
    assert!(round_trips > 20, "the grid must exercise the round trip, not skip it: {round_trips}");
    let data = awkward();
    let (out, shape) = unsqueeze_f32(&data, &[2, 3], 0b100).unwrap();
    assert_eq!(shape, vec![2, 3, 1]);
    assert!(same_bits(&out, &data));
}

// Backs[enforced]: KISS-OPS-6.11-0022 — the `keepdim = 0` result is the `keepdim = 1` result squeezed over
// the reduced axes, over every shape of rank 1..=4 with extents in {1, 2, 3} and every non-empty axis set.
// TEETH: a `reduce_shape` that drops the wrong axis; the all-axes case must reach rank 0.
#[test]
fn squeeze_of_keepdim_one_is_keepdim_zero() {
    let mut checked = 0usize;
    for rank in 1..=4usize {
        let count = 3usize.pow(rank as u32);
        for code in 0..count {
            let shape: Vec<usize> = (0..rank).map(|k| 1 + (code / 3usize.pow(k as u32)) % 3).collect();
            for set in 1u8..(1 << rank) {
                let axes: Vec<usize> = (0..rank).filter(|k| set >> k & 1 == 1).collect();
                let s64: Vec<i64> = shape.iter().map(|e| *e as i64).collect();
                let kept: Vec<usize> = reduce_shape(&s64, &axes, true).iter().map(|e| *e as usize).collect();
                let dropped: Vec<usize> = reduce_shape(&s64, &axes, false).iter().map(|e| *e as usize).collect();
                assert_eq!(squeeze_shape(&kept, set), Ok(dropped), "{shape:?} axes {axes:?}");
                checked += 1;
            }
        }
    }
    // population pin: for rank r, 3^r shapes x (2^r - 1) non-empty sets
    assert_eq!(checked, 3 * 1 + 9 * 3 + 27 * 7 + 81 * 15);
}

// Backs[enforced]: KISS-OPS-6.19-0046 — one plain `u8` axes byte per op, `0x00` refused, the bytes
// independent of the rank, and both ops present in the §6.19.3 table. TEETH: a reader that accepts `0x00`,
// one that reads a `u16`, and the table rows going missing.
#[test]
fn test_ops_opattrs_axes_mask_schema() {
    // the five worked rows of docs/design-board-380-squeeze-unsqueeze.md
    assert_eq!(opattrs::encode_axes_mask(0b010), vec![0x02], "squeeze axis 1 of [2,1,4]");
    assert_eq!(opattrs::encode_axes_mask(0b101), vec![0x05], "squeeze axes {{0,2}} of [1,5,1]");
    assert_eq!(opattrs::encode_axes_mask(0b011), vec![0x03], "squeeze every axis of [1,1]");
    assert_eq!(opattrs::encode_axes_mask(0b001), vec![0x01], "unsqueeze a [3] operand at output position 0");
    assert_eq!(opattrs::encode_axes_mask(0b111), vec![0x07], "unsqueeze a scalar to output rank 3");
    // decode
    assert_eq!(opattrs::decode_axes_mask(&[0x05]), Ok(0x05));
    assert_eq!(opattrs::decode_axes_mask(&[0xFF]), Ok(0xFF), "all eight bits are a legal byte; the op checks the rank");
    assert_eq!(opattrs::decode_axes_mask(&[0x00]), Err(Decline::ReservedZeroOrdinal { field: "axes" }));
    assert!(matches!(opattrs::decode_axes_mask(&[]), Err(Decline::TruncatedBlob { .. })));
    assert!(matches!(opattrs::decode_axes_mask(&[0x01, 0x00]), Err(Decline::TruncatedBlob { .. })), "no u16 word");
    // the table carries both ops with exactly one `axes` u8 field
    let table = parse_table();
    for op in ["squeeze", "unsqueeze"] {
        let s = table.iter().find(|s| s.op == op).unwrap_or_else(|| panic!("{op} missing from the §6.19.3 table"));
        assert_eq!(s.fields.len(), 1, "{op}");
        assert_eq!(s.fields[0].name, "axes");
    }
    assert!(table.iter().any(|s| s.op == "concat"), "control: the parser finds a neighbouring row");
}
