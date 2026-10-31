// SPDX-License-Identifier: MIT OR Apache-2.0
//! The pooled window view's output extent and the window-overlap rule
//! (KISS-OPS-6.13-0010 / -0011, ROADMAP R1.37, CireSnave board 345 "B").
//!
//! The brute-force scan below lists every tap of every output window and so shares nothing with
//! `Axis::check` (a closed-form range of tap indices); the grid test holds the two against each
//! other and the controls prove the comparison can go red.

use kiss_conformance::window::{check_axes, Axis, Decline};

const fn axis(extent: u64, k: u64, d: u64, s: u64, p: u64) -> Axis {
    Axis { extent, window_size: k, dilation: d, stride: s, padding: p }
}

/// Brute force: enumerate window origins while the dilated window fits in the padded extent, and
/// scan every tap against `[0, n)`.
fn brute(a: &Axis) -> Result<u64, Decline> {
    let (n, k, d, s, p) = (a.extent as i64, a.window_size as i64, a.dilation as i64, a.stride as i64, a.padding as i64);
    if k == 0 || d == 0 || s == 0 {
        return Err(Decline::ZeroParameter);
    }
    let dilated = d * (k - 1) + 1;
    let mut out = 0u64;
    while out as i64 * s + dilated <= n + 2 * p {
        let origin = out as i64 * s - p;
        if !(0..k).any(|j| (0..n).contains(&(origin + j * d))) {
            return Err(Decline::NoRealTap { window: out });
        }
        out += 1;
    }
    Ok(out)
}

/// The grid of the R1.37 options note, extended to extent 0.
fn grid() -> impl Iterator<Item = Axis> {
    (0..=12u64).flat_map(|n| {
        (1..=6u64).flat_map(move |k| {
            (1..=4u64).flat_map(move |d| {
                (1..=4u64).flat_map(move |s| (0..=11u64).map(move |p| axis(n, k, d, s, p)))
            })
        })
    })
}

/// Enforces[enforced]: KISS-OPS-6.13-0010 — the output extent `floor((n + 2p - D) / s) + 1`
/// (0 when no window fits), against the window-origin count of the brute force, over the whole
/// grid; plus hand-worked cells and the zero-parameter refusal.
#[test]
fn test_ops_window_output_extent() {
    for a in grid() {
        let expected = match brute(&a) {
            Ok(n) => Some(n),
            Err(Decline::NoRealTap { .. }) => None,
            Err(e) => panic!("{a:?}: {e:?}"),
        };
        if let Some(n) = expected {
            assert_eq!(a.output_extent(), Ok(n), "{a:?}");
        }
    }
    assert_eq!(axis(5, 3, 1, 1, 0).output_extent(), Ok(3));
    assert_eq!(axis(5, 3, 1, 1, 1).output_extent(), Ok(5));
    assert_eq!(axis(7, 3, 2, 2, 0).output_extent(), Ok(2)); // D = 5, (7-5)/2 + 1
    assert_eq!(axis(2, 3, 2, 1, 0).output_extent(), Ok(0)); // D = 5 > 2: no window fits
    for zero in [axis(5, 0, 1, 1, 0), axis(5, 3, 0, 1, 0), axis(5, 3, 1, 0, 0)] {
        assert_eq!(zero.output_extent(), Err(Decline::ZeroParameter), "{zero:?}");
    }
}

/// Enforces[enforced]: KISS-OPS-6.13-0011 — every axis shape is accepted or declined exactly as the
/// brute force says (the verdict and the first offending window), over the whole grid; the
/// named negative control `n=1, k=2, d=2, s=1, p=1` is declined at window 0; and the relayed
/// attribute-only bound `2*padding <= window_size` is shown insufficient at dilation above 1.
#[test]
fn test_ops_window_overlap_declined() {
    let (mut checked, mut declined, mut bound_holes) = (0u32, 0u32, Vec::new());
    for a in grid() {
        let got = a.check();
        assert_eq!(got, brute(&a), "{a:?}");
        checked += 1;
        declined += got.is_err() as u32;
        if a.extent >= 1 && 2 * a.padding <= a.window_size && got.is_err() {
            bound_holes.push(a);
        }
    }
    // Negative control: the counterexample to the attribute-only bound is declined, at window 0.
    let control = axis(1, 2, 2, 1, 1);
    assert_eq!(control.output_extent(), Ok(1));
    assert_eq!(control.check(), Err(Decline::NoRealTap { window: 0 }));
    assert_eq!(check_axes(&[axis(5, 3, 1, 1, 1), control]), Err((1, Decline::NoRealTap { window: 0 })));
    // Positive control: the same attributes over an operand with room are accepted.
    assert_eq!(axis(3, 2, 2, 1, 1).check(), Ok(3));
    assert_eq!(check_axes(&[axis(5, 3, 1, 1, 1), axis(4, 2, 1, 2, 0)]), Ok(vec![5, 2]));
    // The population is not vacuous: the grid has accepted and declined shapes.
    assert!(declined > 0 && declined < checked, "{declined} of {checked}");
    // The bound holes: none at dilation 1, and the control is among those above 1 (R1.37 note: 12).
    assert!(bound_holes.iter().all(|a| a.dilation > 1), "dilation 1: 2*padding <= window_size is exact");
    assert!(bound_holes.contains(&control), "{bound_holes:?}");
    assert_eq!(bound_holes.len(), 12, "{bound_holes:?}");
    // A rule that kept only the attribute bound would accept the control, so it differs from brute force.
    assert!(2 * control.padding <= control.window_size && brute(&control).is_err());
}
