// SPDX-License-Identifier: MIT OR Apache-2.0
//! `keepdim` and `stability` as real flags, the `argmax`/`argmin` tie rule and the reduction-family
//! carriers (KISS-OPS §6.11-0007, §6.11-0008, §6.11-0021, §6.19-0043..-0045).

use kiss_conformance::opattrs;
use kiss_conformance::structural::*;

const NAN: f32 = f32::NAN;

/// Enforces[enforced]: KISS-OPS-6.11-0008 — `keepdim = 1` keeps every reduced axis as extent 1;
/// `keepdim = 0` removes it (the output rank drops by the number of reduced axes) and an all-axes
/// reduction gives rank 0, never a rank-1 extent-1 tensor. TEETH: an implementation that squeezes to
/// `[1]` for an all-axes drop, or removes the wrong axis when two extents are equal.
#[test]
fn test_ops_reduce_keepdim_broadcast() {
    let s = [2usize, 3, 4];
    assert_eq!(reduce_output_shape(&s, &[1], true), Some(vec![2, 1, 4]));
    assert_eq!(reduce_output_shape(&s, &[1], false), Some(vec![2, 4]));
    assert_eq!(reduce_output_shape(&s, &[0, 2], false), Some(vec![3]));
    assert_eq!(reduce_output_shape(&s, &[0, 1, 2], true), Some(vec![1, 1, 1]));
    assert_eq!(reduce_output_shape(&s, &[0, 1, 2], false), Some(vec![]), "an all-axes drop is rank 0");
    // equal extents: which axis went is decided by the axis set, not the extents
    assert_eq!(reduce_output_shape(&[4, 4], &[0], false), Some(vec![4]));
    assert_eq!(reduce_output_shape(&[4, 4], &[1], false), Some(vec![4]));
    assert_eq!(reduce_output_shape(&[4, 8], &[1], false), Some(vec![4]));
    assert_eq!(reduce_output_shape(&[4, 8], &[0], false), Some(vec![8]));
    // keepdim = 0 is the keepdim = 1 shape with the reduced extent-1 axes squeezed out
    let squeeze = |kept: Vec<usize>, axes: &[usize]| -> Vec<usize> {
        kept.into_iter().enumerate().filter(|(d, _)| !axes.contains(d)).map(|(_, e)| e).collect()
    };
    assert_eq!(
        reduce_output_shape(&s, &[0, 2], false).unwrap(),
        squeeze(reduce_output_shape(&s, &[0, 2], true).unwrap(), &[0, 2])
    );
    assert_eq!(reduce_output_shape(&s, &[3], true), None, "an axis past the rank is a decline");
}

/// Enforces[enforced]: KISS-OPS-6.11-0007 — with `stability = 1` the output is the one stable
/// permutation; with `stability = 0` every permutation whose keys are ordered (NaN greatest, `+0`
/// and `-0` and any two NaNs tied) and whose two outputs agree bit for bit is conformant, and any
/// other is not. TEETH: a checker that accepts the stable order only, one that ignores the
/// value/index agreement, and one that treats NaN as unordered.
#[test]
fn test_ops_sort_network_unstable_any_tie_order() {
    // keys 1, 3, 1, 3 ascending: indices 0,2 tie at 1 and 1,3 tie at 3
    let row = [1.0f32, 3.0, 1.0, 3.0];
    let stable = sort_network(&row, SortDirection::Ascending);
    assert_eq!(stable.1, vec![0, 2, 1, 3]);
    let swapped_idx = vec![2usize, 0, 3, 1];
    let swapped_val: Vec<f32> = swapped_idx.iter().map(|&i| row[i]).collect();
    assert!(sort_network_conforms(&row, SortDirection::Ascending, true, &stable.0, &stable.1));
    assert!(!sort_network_conforms(&row, SortDirection::Ascending, true, &swapped_val, &swapped_idx), "stable = 1 admits one order only");
    assert!(sort_network_conforms(&row, SortDirection::Ascending, false, &swapped_val, &swapped_idx), "stable = 0 admits a tie swap");
    // a permutation that breaks the key order is refused either way
    let bad_idx = vec![1usize, 0, 2, 3];
    let bad_val: Vec<f32> = bad_idx.iter().map(|&i| row[i]).collect();
    assert!(!sort_network_conforms(&row, SortDirection::Ascending, false, &bad_val, &bad_idx));
    // values that disagree with the index vector bit for bit are refused (+0 and -0 tie, but their bits differ)
    let z = [0.0f32, -0.0];
    assert!(sort_network_conforms(&z, SortDirection::Ascending, false, &[0.0, -0.0], &[0, 1]));
    assert!(sort_network_conforms(&z, SortDirection::Ascending, false, &[-0.0, 0.0], &[1, 0]), "a tie swap with matching bits");
    assert!(!sort_network_conforms(&z, SortDirection::Ascending, false, &[0.0, -0.0], &[1, 0]), "value bits disagree with the index");
    // not a permutation (a repeated index)
    assert!(!sort_network_conforms(&row, SortDirection::Ascending, false, &[1.0, 1.0, 3.0, 3.0], &[0, 0, 1, 3]));
    // NaN greatest, two NaNs tied, +0/-0 tied: descending puts the NaNs first in either order
    let r2 = [0.0f32, NAN, -0.0, f32::from_bits(0x7FC0_0001)];
    let (v, i) = (vec![r2[3], r2[1], r2[2], r2[0]], vec![3usize, 1, 2, 0]);
    assert!(sort_network_conforms(&r2, SortDirection::Descending, false, &v, &i), "NaN tie and a zero tie in any order");
    let (v2, i2) = (vec![r2[1], r2[0], r2[3], r2[2]], vec![1usize, 0, 3, 2]);
    assert!(!sort_network_conforms(&r2, SortDirection::Descending, false, &v2, &i2), "a zero before a NaN breaks the descending order");
}

// Backs[enforced]: KISS-OPS-6.11-0021 — tie_rule 1 lowest, 2 highest, 3 any tied index (and no other); NaN greatest; +0/-0 tie; independent of sort_network stability
// TEETH: a rule that returns the lowest index for 2, or accepts a non-tied index for 3.
#[test]
fn test_ops_argmax_tie_rule() {
    let row = [2.0f32, 5.0, 1.0, 5.0, 5.0];
    assert_eq!(arg_extreme(&row, true, TieRule::Lowest), Some(1));
    assert_eq!(arg_extreme(&row, true, TieRule::Highest), Some(4));
    for i in 0..row.len() {
        assert_eq!(arg_extreme_conforms(&row, true, TieRule::Any, i), matches!(i, 1 | 3 | 4), "any: index {i}");
        assert_eq!(arg_extreme_conforms(&row, true, TieRule::Lowest, i), i == 1);
        assert_eq!(arg_extreme_conforms(&row, true, TieRule::Highest, i), i == 4);
    }
    // argmin ties at 1.0 (positions 2 and 5)
    let r = [3.0f32, 2.0, 1.0, 9.0, 4.0, 1.0];
    assert_eq!(arg_extreme(&r, false, TieRule::Lowest), Some(2));
    assert_eq!(arg_extreme(&r, false, TieRule::Highest), Some(5));
    // NaN greatest: argmax picks a NaN (lowest / highest of the NaN ties), argmin avoids it
    let n = [NAN, 1.0, NAN, 0.5];
    assert_eq!(arg_extreme(&n, true, TieRule::Lowest), Some(0));
    assert_eq!(arg_extreme(&n, true, TieRule::Highest), Some(2));
    assert_eq!(arg_extreme(&n, false, TieRule::Lowest), Some(3));
    // an all-NaN row: argmin ties at every position
    assert_eq!(arg_extreme(&[NAN, NAN, NAN], false, TieRule::Highest), Some(2));
    // +0 and -0 tie
    assert_eq!(arg_extreme(&[0.0, -0.0, -0.0], true, TieRule::Highest), Some(2));
    // lowest agrees with the stable sort_network's rank 0 under the argmax direction (the default rule)
    let (_, idx) = sort_network(&row, SortDirection::Descending);
    assert_eq!(idx[0], arg_extreme(&row, true, TieRule::Lowest).unwrap());
    // independence: an UNSTABLE sort may put index 4 first, yet argmax with tie_rule 1 is still 1
    let unstable_idx = vec![4usize, 3, 1, 0, 2];
    let unstable_val: Vec<f32> = unstable_idx.iter().map(|&i| row[i]).collect();
    assert!(sort_network_conforms(&row, SortDirection::Descending, false, &unstable_val, &unstable_idx));
    assert_eq!(arg_extreme(&row, true, TieRule::Lowest), Some(1));
}

// Backs[enforced]: KISS-OPS-6.19-0044 — the argmax/argmin blob is keepdim then tie_rule, in that order
// TEETH: a blob with the two bytes swapped, or one that drops keepdim.
#[test]
fn test_ops_argmax_blob_order() {
    assert_eq!(opattrs::encode_arg_reduce(true, TieRule::Lowest), vec![0x01, 0x01]);
    assert_eq!(opattrs::encode_arg_reduce(false, TieRule::Highest), vec![0x00, 0x02]);
    assert_eq!(opattrs::encode_arg_reduce(true, TieRule::Any), vec![0x01, 0x03]);
    assert_ne!(opattrs::encode_arg_reduce(true, TieRule::Highest), opattrs::encode_arg_reduce(false, TieRule::Any), "swapped fields must differ");
}

// Backs[enforced]: KISS-OPS-6.19-0045 — the tie_rule ordinals are 1, 2, 3 and 0 is reserved
// TEETH: an enum that starts at 0.
#[test]
fn test_ops_tie_rule_enum_ordinals() {
    assert_eq!(
        [TieRule::Lowest.ordinal(), TieRule::Highest.ordinal(), TieRule::Any.ordinal()],
        [1, 2, 3]
    );
}

/// Enforces[enforced]: KISS-OPS-6.19-0043 — `reduce_mean`, `reduce_norm2`, `logsumexp`, `any` and `all`
/// carry `reduce_axes` (`u16` LE, the §6.19-0020 word) then `keepdim` (`u8`); toggling `keepdim` changes
/// exactly the last byte. TEETH: a blob that drops `keepdim`, puts it first, or encodes it as a mask.
#[test]
fn test_ops_reduction_family_keepdim_blob() {
    // rank 3: axes {1,2} is a subset (0x0006), all axes 0xFFFF, the lone trailing axis 0xFFFE
    assert_eq!(opattrs::encode_reduction_family(&[1, 2], 3, true), vec![0x06, 0x00, 0x01]);
    assert_eq!(opattrs::encode_reduction_family(&[1, 2], 3, false), vec![0x06, 0x00, 0x00]);
    assert_eq!(opattrs::encode_reduction_family(&[0, 1, 2], 3, true), vec![0xFF, 0xFF, 0x01]);
    assert_eq!(opattrs::encode_reduction_family(&[2], 3, false), vec![0xFE, 0xFF, 0x00]);
    let k1 = opattrs::encode_reduction_family(&[0, 2], 3, true);
    let k0 = opattrs::encode_reduction_family(&[0, 2], 3, false);
    assert_eq!((k1.len(), k0.len()), (3, 3));
    assert_eq!((0..3).filter(|&i| k1[i] != k0[i]).collect::<Vec<_>>(), vec![2], "only the keepdim byte differs");
    assert_eq!((k1[2], k0[2]), (0x01, 0x00));
}
