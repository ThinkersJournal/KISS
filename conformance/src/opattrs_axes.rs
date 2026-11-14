// SPDX-License-Identifier: MIT OR Apache-2.0
//! The `reduce_axes` words of the value-bearing OpAttrs vectors (§6.19-0020), by bitmask arithmetic: split out of
//! [`crate::opattrs_value`] (which re-exports both functions) to keep that file under the repository size limit.

use crate::opattrs_value::MAX_RANK;

/// `reduce_axes` word (§6.19-0020 total precedence), by bitmask arithmetic.
pub fn reduce_axes_word(axes: &[usize], rank: usize) -> Result<u16, String> {
    if !(1..=MAX_RANK as usize).contains(&rank) {
        return Err(format!("rank {rank} outside 1..=MAX_RANK"));
    }
    if axes.is_empty() || axes.iter().any(|&a| a >= rank) {
        return Err("reduced axes empty or out of range".into());
    }
    let mask = axes.iter().fold(0u32, |m, &a| m | (1 << a));
    let full = (1u32 << rank) - 1;
    Ok(if mask == full {
        0xFFFF
    } else if rank > 1 && mask == 1 << (rank - 1) {
        0xFFFE
    } else {
        mask as u16
    })
}

/// `reduce_axes` word for a `prefix_scan` (§6.19-0020 scan precedence): exactly ONE axis; the trailing
/// axis of a rank>1 operand is `0xFFFE`, every other axis its single-bit subset mask. Never `0xFFFF`
/// (a rank-1 scan is `0x0001`, not all-axes) and never `0x0000`.
pub fn scan_axes_word(axes: &[usize], rank: usize) -> Result<u16, String> {
    if !(1..=MAX_RANK as usize).contains(&rank) {
        return Err(format!("rank {rank} outside 1..=MAX_RANK"));
    }
    match axes {
        [a] if *a < rank => Ok(if rank > 1 && *a == rank - 1 { 0xFFFE } else { 1u16 << a }),
        _ => Err(format!("a prefix_scan folds exactly one in-range axis, got {axes:?} of rank {rank}")),
    }
}
