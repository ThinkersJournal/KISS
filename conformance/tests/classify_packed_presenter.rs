// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY-6.6-0025 (CireSnave 2026-11-12, board 427 "Lets go with i."): a tuple-state reduction is
//! presented to the `structure_key` derivation through its keepdim view WITHOUT the state axis. The key carries
//! neither the monoid nor the state axis; a state axis left in is a typed decline.

use kiss_conformance::derive::{derive_structure_key, CellOperand, DeriveDecline, DerivationCell};
use kiss_conformance::structure_key::{MathFidelity, OperandUse};
use kiss_conformance::packed_reduce::{packed_output_shape, Kind};

fn strides(extents: &[i64]) -> Vec<i64> {
    let mut s = vec![1i64; extents.len()];
    for i in (0..extents.len().saturating_sub(1)).rev() {
        s[i] = s[i + 1] * extents[i + 1];
    }
    s
}

fn operand(usage: OperandUse, extents: &[i64]) -> CellOperand {
    CellOperand { usage, dtype: "f32", extents: extents.to_vec(), strides: strides(extents), alignment: 256, dequant: None }
}

fn reduction(in_shape: &[i64], out_shape: &[i64], reduced: &[usize]) -> DerivationCell {
    DerivationCell {
        target: "cuda:sm89",
        operands: vec![operand(OperandUse::Input, in_shape), operand(OperandUse::Output, out_shape)],
        index_operand: None,
        reduced_axes: reduced.to_vec(),
        accumulator: None,
        math_fidelity: MathFidelity::Stable,
        contraction: None,
        vec_cap_bytes: None,
    }
}

/// How the producer of a packed reduction MUST present it: the packed shape with the state axis dropped.
fn present_packed(in_shape: &[i64], reduced: &[usize], kind: Kind) -> DerivationCell {
    let ins: Vec<usize> = in_shape.iter().map(|&e| e as usize).collect();
    let mut packed: Vec<i64> = packed_output_shape(&ins, reduced, true, kind).expect("fits").iter().map(|&e| e as i64).collect();
    packed.pop(); // the trailing state axis is NOT a frame axis (§6.6-0025)
    reduction(in_shape, &packed, reduced)
}

// Backs[enforced]: KISS-CLASSIFY-6.6-0025 — a tuple-state reduction presented through its keepdim view without the state axis derives the byte-identical key of the scalar reduction over the same operands (the key has no monoid and no state axis), a different input rank derives a different key, and an output that still carries the state axis (wider than every input) declines with StateAxisPresented, while the same cell without it derives
#[test]
fn test_classify_packed_reduction_presenter() {
    let scalar = reduction(&[4, 8], &[4, 1], &[1]);
    let base = derive_structure_key(&scalar).expect("control: the scalar reduction derives").to_token();
    for kind in [Kind::Welford, Kind::OnlineSoftmax, Kind::ArgMax] {
        let k = derive_structure_key(&present_packed(&[4, 8], &[1], kind)).expect("presented without the state axis derives");
        assert_eq!(k.to_token(), base, "{kind:?}: identical to the scalar reduction's key, state axis stripped");
    }
    // a different rank is a different key.
    let r3 = derive_structure_key(&present_packed(&[2, 4, 8], &[2], Kind::Welford)).unwrap().to_token();
    assert_ne!(r3, base, "rank 3 and rank 2 reductions are different cells");
    // a state axis left in is declined, never keyed as a rank-(r+1) cell.
    for lanes in [2i64, 3] {
        let left_in = reduction(&[4, 8], &[4, 1, lanes], &[1]);
        assert_eq!(derive_structure_key(&left_in).err(), Some(DeriveDecline::StateAxisPresented), "state axis of extent {lanes}");
    }
    assert_eq!(DeriveDecline::StateAxisPresented.wire(), "StateAxisPresented");
    // the output of a collapsed reduction is still the older decline (a different mistake).
    assert_eq!(derive_structure_key(&reduction(&[4, 8], &[4], &[1])).err(), Some(DeriveDecline::CollapsedReduction));
    // MAX_RANK: a rank-8 input has a derivable keepdim view, but its packed tensor (rank 9) does not exist.
    let r8 = [2i64; 8];
    assert!(derive_structure_key(&present_packed_unchecked(&r8, &[7])).is_ok(), "the keepdim view without the state axis is rank 8");
    assert!(packed_output_shape(&[2usize; 8], &[7], true, Kind::Welford).is_err(), "the packed tensor would be rank 9");
}

fn present_packed_unchecked(in_shape: &[i64], reduced: &[usize]) -> DerivationCell {
    let mut out = in_shape.to_vec();
    for &a in reduced {
        out[a] = 1;
    }
    reduction(in_shape, &out, reduced)
}
