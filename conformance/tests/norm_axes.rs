// SPDX-License-Identifier: MIT OR Apache-2.0
//! `norm_axes`: normalization over several axes (KISS-OPS §6.13-0008, §6.19-0031).

use kiss_conformance::opattrs;
use kiss_conformance::structural::*;

/// Enforces[enforced]: KISS-OPS-6.13-0008 — the statistics are formed over ALL selected axes at once (a
/// PyTorch `normalized_shape` of several dims), not axis by axis; `gamma` and `beta` have the normalized
/// sub-shape (the extents on the selected axes in ascending order, contiguous or not) and are read at the
/// coordinate restricted to those axes. TEETH: a norm applied axis by axis, a `gamma` indexed by the full
/// coordinate, and a sub-shape that keeps an unselected axis.
#[test]
fn test_ops_norm_axes_all_four() {
    // [2,2] over BOTH axes: one group of four, mean 2.5, population variance 1.25
    let x = [1.0, 2.0, 3.0, 4.0];
    let one = [1.0; 4];
    let zero = [0.0; 4];
    let joint = layer_norm_ref(&x, &[2, 2], &[0, 1], &one, &zero, 0.0);
    let s = 1.25f64.sqrt();
    for (g, w) in joint.iter().zip([-1.5, -0.5, 0.5, 1.5]) {
        assert!((g - w / s).abs() < 1e-12, "{g} vs {}", w / s);
    }
    // axis by axis (rows, then columns) is a different function
    let rows = layer_norm_ref(&x, &[2, 2], &[1], &[1.0; 2], &[0.0; 2], 1e-9);
    let seq = layer_norm_ref(&rows, &[2, 2], &[0], &[1.0; 2], &[0.0; 2], 1e-9);
    assert!(joint.iter().zip(&seq).any(|(a, b)| (a - b).abs() > 1e-6), "joint statistics differ from axis-by-axis");
    // a single axis is the case of one: [2,3] over axis 1, two groups
    let y = [1.0, 2.0, 3.0, 10.0, 20.0, 30.0];
    let one_axis = layer_norm_ref(&y, &[2, 3], &[1], &[1.0; 3], &[0.0; 3], 0.0);
    assert!((one_axis[0] + one_axis[2]).abs() < 1e-12 && (one_axis[3] + one_axis[5]).abs() < 1e-12);
    assert!((one_axis[0] - one_axis[3]).abs() < 1e-12, "scale invariance within each group");

    // gamma / beta shape for contiguous and non-contiguous sets
    assert_eq!(normalized_sub_shape(&[2, 3, 4], &[1, 2]), vec![3, 4]);
    assert_eq!(normalized_sub_shape(&[2, 3, 4], &[0, 2]), vec![2, 4], "non-contiguous: the unselected axis is absent");
    assert_eq!(normalized_sub_shape(&[2, 3, 4], &[2, 0]), vec![2, 4], "ascending axis order, not the order given");
    assert_eq!(norm_param_offset(&[1, 2, 3], &[2, 3, 4], &[0, 2]), 1 * 4 + 3);
    assert_eq!(norm_param_offset(&[1, 2, 3], &[2, 3, 4], &[1, 2]), 2 * 4 + 3);
    // gamma really is read per selected coordinate: a per-column scale over [1,0] of [2,2]
    let g = [10.0, 100.0]; // normalized sub-shape over axis 1 only
    let out = layer_norm_ref(&x, &[2, 2], &[1], &g, &[0.0; 2], 0.0);
    assert!((out[0].abs() - 10.0 * 0.999999).abs() < 1e-3 && (out[1].abs() - 100.0 * 0.999999).abs() < 1e-2);

    // the blob: the §6.19-0020 word, two bytes
    assert_eq!(opattrs::encode_norm_axes(&[1, 2], 3), vec![0x06, 0x00]);
    assert_eq!(opattrs::encode_norm_axes(&[0, 2], 4), vec![0x05, 0x00]);
    assert_eq!(opattrs::encode_norm_axes(&[2], 3), vec![0xFE, 0xFF]);
    assert_eq!(opattrs::encode_norm_axes(&[0], 1), vec![0xFF, 0xFF], "rank 1 is all-axes, never the subset mask 0x0001");
}
