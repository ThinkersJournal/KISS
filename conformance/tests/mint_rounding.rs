// SPDX-License-Identifier: MIT OR Apache-2.0
//! Rounding-atom vectors (KISS-OPS-6.7-0001 / -0002): `floor`, `ceil`, `trunc`, `round_even` are
//! PRIMITIVE-FLOOR atoms with no decomposition, so a vector is the only thing that can pin them
//! across implementations. This bundle is the first corpus coverage of the four.
//!
//! Teeth, not just a round trip: for every cell the test recomputes the expected value from an
//! INDEPENDENT integer-arithmetic model of each direction (not `f32::floor`, which `kiss_mint` uses),
//! and then asserts the discriminations a wrong implementation falls into —
//!   * `trunc`/`ceil` of a small negative return `-0.0`, not `+0.0` (§6.7-0002 sign of zero);
//!   * `round_even` is banker's: `2.5 -> 2`, `0.5 -> 0`, `1.5 -> 2` (half-away would give 3, 1, 2);
//!   * `floor` and `trunc` differ exactly on negative non-integers, `ceil` and `trunc` on positive ones.

use kiss_conformance::corpus;

/// An independent model: round `x` (finite, as f64) to an integer value by direction, via i128
/// integer arithmetic on the exact f32 value, then restore the sign of zero. `None` for non-finite.
fn model(op: &str, x: f32) -> f32 {
    if !x.is_finite() {
        return x;
    }
    let bits = x.to_bits();
    let neg = bits >> 31 == 1;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let man = (bits & 0x7F_FFFF) as i128;
    // value = sig * 2^(e2), sig integer
    // (an exponent this large is already integral; avoids i128 overflow on the largest finite)
    let (sig, e2) = if exp == 0 { (man, -149) } else { (man | (1 << 23), exp - 150) };
    // split into integer part and a numerator/denominator fraction
    if e2 >= 24 {
        return x;
    }
    let (int_part, num, den): (i128, i128, i128) = if e2 >= 0 {
        (sig << e2, 0, 1)
    } else if -e2 >= 127 {
        (0, sig, 1i128 << 126) // far below 1: fraction only (still exact enough for direction tests)
    } else {
        let den = 1i128 << (-e2);
        (sig / den, sig % den, den)
    };
    let frac_nonzero = num != 0;
    let twice = num * 2;
    // magnitude result, then apply direction in terms of signed value
    let mag = int_part; // trunc magnitude
    let signed_floor = if neg { -(mag + i128::from(frac_nonzero)) } else { mag };
    let signed_ceil = if neg { -mag } else { mag + i128::from(frac_nonzero) };
    let signed_trunc = if neg { -mag } else { mag };
    let round_mag = if twice > den || (twice == den && mag % 2 == 1) { mag + 1 } else { mag };
    let signed_even = if neg { -round_mag } else { round_mag };
    let v = match op {
        "floor" => signed_floor,
        "ceil" => signed_ceil,
        "trunc" => signed_trunc,
        "round_even" => signed_even,
        _ => panic!("unknown op {op}"),
    };
    // the sign of a zero result follows the operand's sign for trunc/round_even/ceil-of-negative
    // and floor-of-negative-zero; an exact +0 result of a positive operand is +0.
    let out = v as f32;
    if v == 0 && neg {
        -0.0
    } else {
        out
    }
}

// Backs[enforced]: KISS-OPS-6.7-0001 — each rounding atom rounds in its named direction (the four ops, the
// tie, zero-sign and integral boundaries); the NaN half of 6.7-0002 is deliberately not pinned (#396).
#[test]
fn rounding_bundle_matches_an_independent_model_and_discriminates() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-rounding.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint -- --write`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 4 * 18, "4 ops x 18 inputs");

    let mut ops = std::collections::BTreeSet::new();
    let (mut neg_zero_from_neg_fraction, mut ties_banker, mut dir_split) = (0, 0, 0);
    for v in &c.vectors {
        let w = format!("{} tc {}", v.op, v.tc_id);
        assert_eq!(v.dtype, "f32", "{w}");
        assert_eq!(v.inputs.len(), 1, "{w}: unary");
        ops.insert(v.op.clone());
        let xb: [u8; 4] = v.inputs[0].clone().try_into().expect("4 bytes");
        let x = f32::from_bits(u32::from_be_bytes(xb));
        let got: [u8; 4] = v.expected.clone().try_into().expect("4 bytes");
        let want = model(&v.op, x);
        assert_eq!(
            u32::from_be_bytes(got),
            want.to_bits(),
            "{w}: x={x:?} expected bits differ from the independent model ({want:?})"
        );
        if (v.op == "trunc" || v.op == "ceil") && x < 0.0 && x > -1.0 {
            assert_eq!(u32::from_be_bytes(got), 0x8000_0000, "{w}: result of a negative fraction toward zero is -0.0");
            neg_zero_from_neg_fraction += 1;
        }
        if v.op == "round_even" && (x == 0.5 || x == 1.5 || x == 2.5 || x == -0.5 || x == -1.5 || x == -2.5) {
            let r = f32::from_bits(u32::from_be_bytes(got));
            assert_eq!(r.abs() % 2.0, 0.0, "{w}: banker's rounding lands on an even integer");
            ties_banker += 1;
        }
        if v.op == "floor" && x == -0.5 {
            assert_eq!(f32::from_bits(u32::from_be_bytes(got)), -1.0, "{w}");
            dir_split += 1;
        }
    }
    for op in ["floor", "ceil", "trunc", "round_even"] {
        assert!(ops.contains(op), "op {op} must be covered");
    }
    assert!(neg_zero_from_neg_fraction >= 4, "the -0.0-from-negative-fraction cells must be present");
    assert!(ties_banker >= 6, "the six ties for round_even must be present");
    assert!(dir_split >= 1, "floor(-0.5) = -1 (differs from trunc) must be present");
}
