// SPDX-License-Identifier: MIT OR Apache-2.0
//! The pooled window view's output extent and the window-overlap rule
//! (KISS-OPS-6.13-0010 / -0011, ROADMAP R1.37, CireSnave board 345 "B").
//!
//! The brute-force scan below lists every tap of every output window and so shares nothing with
//! `Axis::check` (a closed-form range of tap indices); the grid test holds the two against each
//! other and the controls prove the comparison can go red.

use kiss_conformance::window::{check_axes, Axis, Decline};

/// A symmetric-padding axis (`padding_lo == padding_hi == p`).
const fn axis(extent: u64, k: u64, d: u64, s: u64, p: u64) -> Axis {
    Axis { extent, window_size: k, dilation: d, stride: s, padding_lo: p, padding_hi: p }
}

/// An axis with independent low-side and high-side padding.
const fn axis2(extent: u64, k: u64, d: u64, s: u64, lo: u64, hi: u64) -> Axis {
    Axis { extent, window_size: k, dilation: d, stride: s, padding_lo: lo, padding_hi: hi }
}

/// Brute force: enumerate window origins while the dilated window fits in the padded extent, and
/// scan every tap against `[0, n)`.
fn brute(a: &Axis) -> Result<u64, Decline> {
    let (n, k, d, s, p, hi) = (a.extent as i64, a.window_size as i64, a.dilation as i64, a.stride as i64, a.padding_lo as i64, a.padding_hi as i64);
    if k == 0 || d == 0 || s == 0 {
        return Err(Decline::ZeroParameter);
    }
    let dilated = d * (k - 1) + 1;
    let mut out = 0u64;
    while out as i64 * s + dilated <= n + p + hi {
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
    // The same extent written in the shape-expression vocabulary (§6.20-0002, §6.20-0012): the `Max`
    // clamp is what makes "no window fits" 0, over the whole grid, against the closed form above.
    use kiss_conformance::shape_expr::{eval_dim, Dim, DimValue};
    let b = |d: Dim| Box::new(d);
    let c = |v: i64| b(Dim::Const(v));
    let (n, k, s, d, plo, phi) = (b(Dim::Extent { operand: 0, axis: 0 }), b(Dim::Param(0)), b(Dim::Param(1)), b(Dim::Param(2)), b(Dim::Param(3)), b(Dim::Param(4)));
    let dilated = Dim::Add(b(Dim::Mul(d, b(Dim::Sub(k, c(1))))), c(1));
    let span = Dim::Sub(b(Dim::Add(b(Dim::Add(n, plo)), phi)), b(dilated));
    let extent = Dim::Max(b(Dim::Add(b(Dim::Div(b(span), s)), c(1))), c(0));
    for a in grid() {
        if matches!(brute(&a), Err(Decline::NoRealTap { .. })) {
            continue;
        }
        let want = a.output_extent().expect("grid axes have non-zero parameters") as i64;
        let got = eval_dim(&extent, &[vec![a.extent as i64]], &[a.window_size as i64, a.stride as i64, a.dilation as i64, a.padding_lo as i64, a.padding_hi as i64]);
        assert_eq!(got, Ok(DimValue::Concrete(want)), "{a:?}");
    }
    // input 2, window 8, stride 1: -5 before the clamp, 0 after
    let small = eval_dim(&extent, &[vec![2]], &[8, 1, 1, 0, 0]);
    assert_eq!(small, Ok(DimValue::Concrete(0)));
    for zero in [axis(5, 0, 1, 1, 0), axis(5, 3, 0, 1, 0), axis(5, 3, 1, 0, 0)] {
        assert_eq!(zero.output_extent(), Err(Decline::ZeroParameter), "{zero:?}");
    }
}

/// Enforces[enforced]: KISS-OPS-6.13-0011 — every axis shape is accepted or declined exactly as the
/// brute force says (the verdict and the first offending window), over the whole grid; the
/// named negative control `n=1, k=2, d=2, s=1, p=1` is declined at window 0; and the relayed
/// attribute-only bound `padding_lo + padding_hi <= window_size` is shown insufficient at dilation above 1.
#[test]
fn test_ops_window_overlap_declined() {
    let (mut checked, mut declined, mut bound_holes) = (0u32, 0u32, Vec::new());
    for a in grid() {
        let got = a.check();
        assert_eq!(got, brute(&a), "{a:?}");
        checked += 1;
        declined += got.is_err() as u32;
        if a.extent >= 1 && a.padding_lo + a.padding_hi <= a.window_size && got.is_err() {
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
    assert!(control.padding_lo + control.padding_hi <= control.window_size && brute(&control).is_err());
}

/// Enforces[enforced]: KISS-OPS-6.13-0010 — with independent `padding_lo` and `padding_hi` the output
/// extent is `floor((n + p_lo + p_hi - D) / s) + 1` and tap `j` of window `o` is at `o*s - p_lo + j*d`: the
/// closed form agrees with the brute force over an asymmetric grid, the sides are not interchangeable
/// (a pair whose verdict differs from its swap exists), and a zero low side with a high side is not the
/// same axis as the reverse.
#[test]
fn test_ops_window_asymmetric_padding() {
    let (mut checked, mut asym_differs) = (0u32, 0u32);
    for n in 0..=8u64 {
        for k in 1..=4u64 {
            for d in 1..=3u64 {
                for s in 1..=3u64 {
                    for lo in 0..=5u64 {
                        for hi in 0..=5u64 {
                            let a = axis2(n, k, d, s, lo, hi);
                            assert_eq!(a.check(), brute(&a), "{a:?}");
                            checked += 1;
                            if lo != hi && a.check() != axis2(n, k, d, s, hi, lo).check() {
                                asym_differs += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(checked > 10_000);
    assert!(asym_differs > 0, "swapping the sides must change a verdict somewhere, or the sides are not independent");
    // hand-worked: n=4, k=3, d=1, s=1, lo=2, hi=0 -> span 6, out (6-3)/1+1 = 4; first window origin -2 has taps -2,-1,0.
    assert_eq!(axis2(4, 3, 1, 1, 2, 0).check(), Ok(4));
    assert_eq!(axis2(4, 3, 1, 1, 0, 2).check(), Ok(4));
    // a window wholly in the low padding has no real tap: lo=3 with k=3, n=4 -> window 0 taps -3,-2,-1.
    assert_eq!(axis2(4, 3, 1, 1, 3, 0).check(), Err(Decline::NoRealTap { window: 0 }));
    // while the same pad on the high side declines the window past the end: span 7 gives 5 windows, and window 4 sits at 4,5,6.
    assert_eq!(axis2(4, 3, 1, 1, 0, 3).check(), Err(Decline::NoRealTap { window: 4 }));
}
