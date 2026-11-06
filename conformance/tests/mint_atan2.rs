// SPDX-License-Identifier: MIT OR Apache-2.0
//! `atan2` quadrant vectors (KISS-OPS-6.9-0001): the four-quadrant arctangent with the IEEE 754
//! +-0 conventions. `atan(y/x)` passes every x > 0 cell and fails every x < 0 and x = +-0 cell.
//!
//! Teeth: every cell's expected value is recomputed from a RULE TABLE written here (IEEE 754 Annex F
//! values as hard-coded f32 bit patterns, not `f64::atan2`, which `kiss_mint` uses), and ordinary
//! cells are additionally checked to lie in their quadrant. The cells are class ULP, never exact-byte
//! (§6.8-0005), and the test asserts that too.

use kiss_conformance::{corpus, DeterminismClass};

const PI: u32 = 0x4049_0FDB; // f32(pi)
const PI_2: u32 = 0x3FC9_0FDB; // f32(pi/2)
const PI_4: u32 = 0x3F49_0FDB; // f32(pi/4)
const PI_3_4: u32 = 0x4016_CBE4; // f32(3pi/4)
const SIGN: u32 = 0x8000_0000;

/// Expected bits for the special (zero / infinite operand) cells, from IEEE 754-2019 Annex F rules;
/// `None` for an ordinary finite non-zero pair (checked by quadrant and a 1-ULP-of-f64 bound).
fn rule(y: f32, x: f32) -> Option<u32> {
    let ys = if y.is_sign_negative() { SIGN } else { 0 };
    let mag = if y.is_infinite() && x.is_infinite() {
        if x > 0.0 { PI_4 } else { PI_3_4 }
    } else if y.is_infinite() {
        PI_2
    } else if x.is_infinite() {
        if x > 0.0 { 0 } else { PI }
    } else if y == 0.0 {
        if x.is_sign_negative() { PI } else { 0 } // includes x = -0 and x = +-0 vs y = +-0
    } else if x == 0.0 {
        PI_2
    } else {
        return None;
    };
    Some(ys | mag)
}

// Backs[enforced]: KISS-OPS-6.9-0001 — atan2 is the four-quadrant arctangent; the sign of a zero operand
// selects the quadrant (the 28 cells cover each quadrant, the +-0 matrix and the +-inf matrix).
#[test]
fn test_ops_atan2_quadrants() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-atan2-quadrants.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint -- --write`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 28);
    let (mut special, mut ordinary, mut x_neg_ordinary, mut neg_zero_x) = (0, 0, 0, 0);
    for v in &c.vectors {
        let w = format!("atan2 tc {}", v.tc_id);
        assert_eq!(v.op, "atan2", "{w}");
        assert_eq!(v.dtype, "f32", "{w}");
        assert_eq!(v.class, DeterminismClass::UlpTolerance, "{w}: atan2 is class ULP, never exact-byte (6.8-0005)");
        assert_eq!(v.inputs.len(), 2, "{w}: binary (a=y, b=x)");
        let f = |b: &Vec<u8>| f32::from_bits(u32::from_be_bytes(b.clone().try_into().expect("4 bytes")));
        let (y, x) = (f(&v.inputs[0]), f(&v.inputs[1]));
        let got = u32::from_be_bytes(v.expected.clone().try_into().expect("4 bytes"));
        match rule(y, x) {
            Some(want) => {
                assert_eq!(got, want, "{w}: atan2({y:?},{x:?}) expected bits differ from the rule table");
                special += 1;
                if x == 0.0 && x.is_sign_negative() && y == 0.0 {
                    assert_eq!(got & !SIGN, PI, "{w}: x = -0 puts a zero y on the pi branch");
                    neg_zero_x += 1;
                }
            }
            None => {
                let r = f32::from_bits(got);
                assert_eq!(r.is_sign_negative(), y < 0.0, "{w}: result sign follows y");
                let (lo, hi) = if x > 0.0 { (0.0, std::f32::consts::FRAC_PI_2) } else { (std::f32::consts::FRAC_PI_2, std::f32::consts::PI) };
                assert!(r.abs() >= lo - 1e-6 && r.abs() <= hi + 1e-6, "{w}: {r} outside its quadrant");
                // the atan(y/x) mutation lands in (-pi/2, pi/2): distinct for every x < 0 cell
                if x < 0.0 {
                    assert!(r.abs() > std::f32::consts::FRAC_PI_2, "{w}: x<0 must leave the right half-plane");
                    x_neg_ordinary += 1;
                }
                ordinary += 1;
            }
        }
    }
    assert!(special >= 20 && ordinary == 8, "population: {special} special, {ordinary} ordinary");
    assert!(x_neg_ordinary >= 4, "the x<0 ordinary cells (quadrants 2 and 3) must be present");
    assert!(neg_zero_x >= 2, "atan2(+-0, -0) cells must be present");
}
