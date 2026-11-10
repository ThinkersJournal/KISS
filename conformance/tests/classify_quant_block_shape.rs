//! KISS-CLASSIFY-6.3-0009b: the `quant` record's `block_shape` has one `u16` entry per operand
//! axis, each at least 1; a wrong length or a `0` entry is a typed decline.

use kiss_conformance::quant_block::{block_elems, BlockShapeDecline};

// Backs[enforced]: KISS-CLASSIFY-6.3-0009b
#[test]
fn test_classify_quant_block_shape() {
    // A one-dimensional block of 32 along the last axis of a rank-2 operand: 32 at that
    // position, 1 elsewhere; 32 elements share one scale.
    assert_eq!(block_elems(2, &[1, 32]), Ok(32));
    // An N-dimensional block: the elements per scale are the product of the entries.
    assert_eq!(block_elems(3, &[1, 16, 16]), Ok(256));
    assert_eq!(block_elems(2, &[8, 4]), Ok(32));
    // A rank-0 operand has the empty list (one scale for the one element).
    assert_eq!(block_elems(0, &[]), Ok(1));
    // Length must equal rank: too short, too long, and the old scalar form on a rank-2 operand.
    assert_eq!(block_elems(2, &[32]), Err(BlockShapeDecline::RankMismatch { want: 2, got: 1 }));
    assert_eq!(block_elems(1, &[4, 4]), Err(BlockShapeDecline::RankMismatch { want: 1, got: 2 }));
    assert_eq!(block_elems(0, &[1]), Err(BlockShapeDecline::RankMismatch { want: 0, got: 1 }));
    // A `0` entry declines and names its axis; the length is checked first.
    assert_eq!(block_elems(3, &[1, 0, 16]), Err(BlockShapeDecline::ZeroExtent { axis: 1 }));
    assert_eq!(block_elems(2, &[0]), Err(BlockShapeDecline::RankMismatch { want: 2, got: 1 }));
    // The largest legal entries do not overflow the product.
    assert_eq!(block_elems(4, &[65535; 4]), Ok(65535u64.pow(4)));
}
