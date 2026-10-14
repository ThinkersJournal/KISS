// SPDX-License-Identifier: MIT OR Apache-2.0
//! f32/f64 min/max NaN vectors (#329). `max_prop`/`min_prop` and `fmax_ieee`/`fmin_ieee` differ ONLY
//! on a NaN operand, and before this bundle no f32/f64 corpus cell carried one — so the existing 72
//! minmax vectors passed for an implementation with the two arms SWAPPED.
//!
//! This file is the acceptance criterion of #329, not just a round trip:
//!   1. every cell's `expected` equals the §6.13 decomposition (`select(cmp_ne(a,a), …)`) evaluated
//!      with native float compares — an oracle independent of `kiss_mint`'s raw-bit helper;
//!   2. a SWAPPED implementation (prop <-> ieee) is computed for every cell and MUST differ from the
//!      published `expected` on every cell (the "swap and redden" control), while the correct one
//!      agrees on every cell (the "restore and pass" control);
//!   3. both NaN positions, both signaling and quiet, a negative-signed NaN and both-NaN are present.

use kiss_conformance::corpus;
use kiss_conformance::nan_provenance::is_nan;

macro_rules! oracle {
    ($name:ident, $t:ty) => {
        /// §6.13 expansions, verbatim, on native floats; returns the selected operand's bits.
        fn $name(op: &str, a: $t, b: $t) -> $t {
            let sel = |c: bool, x: $t, y: $t| if c { x } else { y };
            let (an, bn) = (a != a, b != b);
            match op {
                "max_prop" => sel(an, a, sel(bn, b, sel(a >= b, a, b))),
                "min_prop" => sel(an, a, sel(bn, b, sel(a <= b, a, b))),
                "fmax_ieee" => sel(an, b, sel(bn, a, sel(a >= b, a, b))),
                "fmin_ieee" => sel(an, b, sel(bn, a, sel(a <= b, a, b))),
                _ => panic!("unknown op {op}"),
            }
        }
    };
}
oracle!(oracle_f32, f32);
oracle!(oracle_f64, f64);

fn swapped(op: &str) -> &'static str {
    match op {
        "max_prop" => "fmax_ieee",
        "min_prop" => "fmin_ieee",
        "fmax_ieee" => "max_prop",
        "fmin_ieee" => "min_prop",
        _ => panic!("unknown op {op}"),
    }
}

fn eval_bits(dtype: &str, op: &str, a: &[u8], b: &[u8]) -> Vec<u8> {
    match dtype {
        "f32" => {
            let r = oracle_f32(
                op,
                f32::from_bits(u32::from_be_bytes(a.try_into().unwrap())),
                f32::from_bits(u32::from_be_bytes(b.try_into().unwrap())),
            );
            r.to_bits().to_be_bytes().to_vec()
        }
        "f64" => {
            let r = oracle_f64(
                op,
                f64::from_bits(u64::from_be_bytes(a.try_into().unwrap())),
                f64::from_bits(u64::from_be_bytes(b.try_into().unwrap())),
            );
            r.to_bits().to_be_bytes().to_vec()
        }
        _ => panic!("dtype {dtype}"),
    }
}

// Backs: KISS-OPS-6.16-0009 — a selection that MOVES a NaN operand preserves its bits; the propagating
// and IEEE arms are distinguishable on f32/f64 (#329's acceptance: swap them and the corpus reddens).
#[test]
fn minmax_nan_bundle_matches_the_decomposition_and_redden_when_the_arms_are_swapped() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-minmax-nan.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 2 * (2 * 6 + 2 * 5), "2 dtypes x (2 prop ops x 6 + 2 ieee ops x 5)");

    let (mut swap_reds, mut a_pos, mut b_pos, mut snan, mut negq, mut both, mut quiet) = (0, 0, 0, 0, 0, 0, 0);
    let mut dtypes = std::collections::BTreeSet::new();
    let mut ops = std::collections::BTreeSet::new();
    for v in &c.vectors {
        let w = format!("{} {} tc {}", v.op, v.dtype, v.tc_id);
        assert_eq!(v.inputs.len(), 2, "{w}");
        let (a, b) = (&v.inputs[0], &v.inputs[1]);
        dtypes.insert(v.dtype.clone());
        ops.insert(v.op.clone());
        let (an, bn) = (is_nan(&v.dtype, a), is_nan(&v.dtype, b));
        assert!(an || bn, "{w}: every cell has a NaN operand (the value arm is covered elsewhere)");

        // 1. the published expectation is the decomposition's result, bit for bit.
        let want = eval_bits(&v.dtype, &v.op, a, b);
        assert_eq!(v.expected, want, "{w}: expected bits differ from the §6.13 decomposition");

        // 2. swap the arms: the swapped implementation MUST disagree with the published bytes.
        let swapped_out = eval_bits(&v.dtype, swapped(&v.op), a, b);
        let reds = swapped_out != v.expected;
        // the one cell a swap cannot see is both-NaN with equal bits; none is authored.
        assert!(reds, "{w}: a prop<->ieee swap would PASS this cell — it does not discriminate");
        swap_reds += 1;

        // 3. population
        if an && !bn {
            a_pos += 1;
        }
        if bn && !an {
            b_pos += 1;
        }
        if an && bn {
            both += 1;
            assert_ne!(a, b, "{w}: both-NaN cells carry distinct payloads so the survivor is observable");
        }
        let nan_bytes = if an { a } else { b };
        let top = nan_bytes[0];
        if top & 0x80 != 0 {
            negq += 1;
        }
        // signaling = quiet bit clear: f32 byte1 bit6 of the mantissa MSB / f64 byte1 bit3
        let quiet_set = match v.dtype.as_str() {
            "f32" => nan_bytes[1] & 0x40 != 0,
            _ => nan_bytes[1] & 0x08 != 0,
        };
        if quiet_set { quiet += 1 } else { snan += 1 }
    }
    assert_eq!(swap_reds, c.vectors.len());
    for op in ["max_prop", "min_prop", "fmax_ieee", "fmin_ieee"] {
        assert!(ops.contains(op), "op {op}");
    }
    assert!(dtypes.contains("f32") && dtypes.contains("f64"));
    assert!(a_pos > 0 && b_pos > 0, "NaN in a and NaN in b are distinct cases");
    assert!(snan > 0 && quiet > 0, "both signaling and quiet NaNs");
    assert!(negq > 0, "a negative-signed NaN: the sign is part of the moved bits");
    assert!(both > 0, "both-NaN pins the survivor identity (prop returns a, IEEE returns b)");
}
