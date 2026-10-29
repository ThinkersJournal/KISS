// SPDX-License-Identifier: MIT OR Apache-2.0
//! `nextafter` value-arm vectors (§6.9-0003, first half; the clause is NOT counted as backed: its
//! typed-decline half for f16/bf16/f8e4m3fn/f8e5m2 has no vector form).
//!
//! Teeth: each expected value is recomputed by an INDEPENDENT model — map the bit pattern to a
//! monotone integer key (negatives mirrored, both zeros at key 0), step the key by one toward `b`,
//! map back (a step that lands on zero keeps `a`'s sign) — not the sign-magnitude `bits +- 1` stepping `kiss_mint` uses. Then the discriminations a
//! wrong implementation falls into are asserted: an f32 result differs from the f64 neighbour of the
//! same value (the promotion hazard), zero crossing yields the smallest subnormal, and equal operands
//! return `b` (so `nextafter(+0,-0)` is `-0`).

use kiss_conformance::{corpus, DeterminismClass};

fn key32(bits: u32) -> i64 {
    let mag = i64::from(bits & 0x7FFF_FFFF);
    if bits >> 31 == 1 { -mag } else { mag }
}
fn unkey32(k: i64) -> u32 {
    if k < 0 { 0x8000_0000 | ((-k) as u32) } else { k as u32 }
}
fn key64(bits: u64) -> i128 {
    let mag = i128::from(bits & 0x7FFF_FFFF_FFFF_FFFF);
    if bits >> 63 == 1 { -mag } else { mag }
}
fn unkey64(k: i128) -> u64 {
    if k < 0 { 0x8000_0000_0000_0000 | ((-k) as u64) } else { k as u64 }
}

/// The independent model: step the monotone key one place toward `b` (equal -> `b`'s bits).
fn model32(a: u32, b: u32) -> u32 {
    let (ka, kb) = (key32(a), key32(b));
    if ka == kb {
        return b;
    }
    let k = if kb > ka { ka + 1 } else { ka - 1 };
    // stepping onto the shared zero key keeps the sign of `a` (nextafter(-min_subnormal, 0) is -0)
    if k == 0 { a & 0x8000_0000 } else { unkey32(k) }
}
fn model64(a: u64, b: u64) -> u64 {
    let (ka, kb) = (key64(a), key64(b));
    if ka == kb {
        return b;
    }
    let k = if kb > ka { ka + 1 } else { ka - 1 };
    if k == 0 { a & 0x8000_0000_0000_0000 } else { unkey64(k) }
}

// Deliberately NOT a backing test for the clause (§6.9-0003 has a decline half no vector can carry):
// the vector instrument is checked here and the ledger row stays `untested` with a note.
// harness-internal: checks the frozen nextafter corpus bundle against an independent key model (file states it is deliberately not a backing test)
#[test]
fn nextafter_value_cells_match_the_monotone_key_model() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-nextafter.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint -- --write`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 40, "14 shapes + 6 edge pairs, each at f32 and f64");
    let (mut cross_zero, mut equal_returns_b, mut f32_not_f64_step) = (0, 0, 0);
    for v in &c.vectors {
        let w = format!("nextafter tc {} {}", v.tc_id, v.dtype);
        assert_eq!(v.op, "nextafter", "{w}");
        assert_eq!(v.class, DeterminismClass::ExactByte, "{w}");
        assert_eq!(v.inputs.len(), 2, "{w}");
        match v.dtype.as_str() {
            "f32" => {
                let a = u32::from_be_bytes(v.inputs[0].clone().try_into().expect("4"));
                let b = u32::from_be_bytes(v.inputs[1].clone().try_into().expect("4"));
                let got = u32::from_be_bytes(v.expected.clone().try_into().expect("4"));
                assert_eq!(got, model32(a, b), "{w}: a={a:#x} b={b:#x}");
                if f32::from_bits(a) == 0.0 && f32::from_bits(b) != 0.0 {
                    assert_eq!(got & 0x7FFF_FFFF, 1, "{w}: a zero steps to the smallest subnormal");
                    cross_zero += 1;
                }
                if f32::from_bits(a) == f32::from_bits(b) {
                    assert_eq!(got, b, "{w}: equal operands return b (signed zero carried)");
                    equal_returns_b += 1;
                }
                // promotion hazard: the f64 neighbour of an f32 value is not an f32 step
                if a == 0x3F80_0000 && f32::from_bits(b) > 1.0 {
                    let promoted = (f64::from_bits(f64::from(f32::from_bits(a)).to_bits() + 1)) as f32;
                    assert_ne!(promoted.to_bits(), got, "{w}: promote-through-f64 returns 1.0 again");
                    f32_not_f64_step += 1;
                }
            }
            "f64" => {
                let a = u64::from_be_bytes(v.inputs[0].clone().try_into().expect("8"));
                let b = u64::from_be_bytes(v.inputs[1].clone().try_into().expect("8"));
                let got = u64::from_be_bytes(v.expected.clone().try_into().expect("8"));
                assert_eq!(got, model64(a, b), "{w}: a={a:#x} b={b:#x}");
            }
            other => panic!("{w}: unexpected dtype {other}"),
        }
    }
    assert!(cross_zero >= 2, "zero-crossing cells present: {cross_zero}");
    assert!(equal_returns_b >= 3, "equal-operand cells present: {equal_returns_b}");
    assert!(f32_not_f64_step >= 1, "the promotion-hazard discrimination ran");
}
