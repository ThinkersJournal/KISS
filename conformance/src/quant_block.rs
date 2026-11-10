//! The Classify `quant` record's `block_shape` (KISS-CLASSIFY-6.3-0009b): one block extent per
//! operand axis. This models only the shape rule Classify pins (length and positivity); which
//! shapes a quantization `family` permits is the registry's, and Classify does not interpret it.

/// A `block_shape` that Classify answers with a typed decline (§6.7-0009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockShapeDecline {
    /// The list does not have exactly the operand's `rank` entries.
    RankMismatch { want: usize, got: usize },
    /// The entry on `axis` is `0`; a block extent is at least `1`.
    ZeroExtent { axis: usize },
}

/// The number of elements that share one scale (the product of the entries), or the decline
/// for a list that is not one entry per axis of an operand of `rank`, or has a `0` entry.
/// The length is checked first, so a wrong-length list that also holds a `0` reports the length.
pub fn block_elems(rank: usize, block_shape: &[u16]) -> Result<u64, BlockShapeDecline> {
    if block_shape.len() != rank {
        return Err(BlockShapeDecline::RankMismatch { want: rank, got: block_shape.len() });
    }
    let mut product: u64 = 1;
    for (axis, &e) in block_shape.iter().enumerate() {
        if e == 0 {
            return Err(BlockShapeDecline::ZeroExtent { axis });
        }
        product *= u64::from(e);
    }
    Ok(product)
}
