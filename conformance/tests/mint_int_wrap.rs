// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integer-wrapping vectors (KISS-OPS-6.2-0002): integer `add`/`sub`/`mul` are wrapping
//! two's-complement modulo 2^bits — not saturating. Before this bundle the corpus held no integer
//! cell, so a saturating implementation passed every vector.
//!
//! Teeth, not just a round trip: every cell is recomputed from an INDEPENDENT model — the operands
//! are read as signed or unsigned per the dtype, the operation is done in exact i128 arithmetic, and
//! the result is reduced modulo 2^bits (`kiss_mint` uses `u64::wrapping_*` on raw patterns). Then the
//! test asserts the discrimination a wrong implementation falls into: a SATURATING implementation
//! must disagree with the corpus on at least one cell per dtype and per op.

use kiss_conformance::corpus;
use std::collections::BTreeMap;

fn lane(dtype: &str) -> (u32, bool) {
    match dtype {
        "i8" => (8, true),
        "u8" => (8, false),
        "i16" => (16, true),
        "u16" => (16, false),
        "i32" => (32, true),
        "u32" => (32, false),
        "i64" => (64, true),
        "u64" => (64, false),
        other => panic!("unexpected dtype {other}"),
    }
}

fn read(bytes: &[u8], bits: u32, signed: bool) -> i128 {
    let mut v: u128 = 0;
    for b in bytes {
        v = (v << 8) | u128::from(*b);
    }
    assert_eq!(bytes.len() as u32 * 8, bits);
    if signed && (v >> (bits - 1)) & 1 == 1 {
        v as i128 - (1i128 << bits)
    } else {
        v as i128
    }
}

/// `v` reduced modulo 2^bits, then re-read as a (signed or unsigned) value of that width.
fn wrap(v: i128, bits: u32, signed: bool) -> i128 {
    let m = 1i128 << bits;
    let mut r = v.rem_euclid(m);
    if signed && r >= m / 2 {
        r -= m;
    }
    r
}

fn saturate(v: i128, bits: u32, signed: bool) -> i128 {
    let (lo, hi) = if signed { (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1) } else { (0, (1i128 << bits) - 1) };
    v.clamp(lo, hi)
}

// Backs[enforced]: KISS-OPS-6.2-0002 — integer add/sub/mul wrap modulo 2^bitwidth (never saturate) at the
// boundary patterns of every 8/16/32/64-bit signed and unsigned lane.
#[test]
fn test_ops_int_wrapping() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-int-wrap.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    // 8 dtypes x (add 15 + mul 15 + sub 25) cells.
    assert_eq!(c.vectors.len(), 8 * (15 + 15 + 25));

    // (dtype, op) -> cells where a SATURATING implementation would differ from the corpus.
    let mut sat_differs: BTreeMap<(String, String), u32> = BTreeMap::new();
    for v in &c.vectors {
        let w = format!("{} {} tc {}", v.dtype, v.op, v.tc_id);
        let (bits, signed) = lane(&v.dtype);
        assert_eq!(v.inputs.len(), 2, "{w}: binary");
        let a = read(&v.inputs[0], bits, signed);
        let b = read(&v.inputs[1], bits, signed);
        // The exact result. u64 x u64 can exceed i128, so `mul` keeps the exact value when it fits and
        // otherwise the SIGN of the overflowed product: the wrapping model needs only the value
        // mod 2^bits (i128 wrapping multiplication preserves it, 2^bits dividing 2^128), and the
        // saturating model only needs to know the true result is beyond the lane.
        let (exact, beyond) = match v.op.as_str() {
            "add" => (a + b, None),
            "sub" => (a - b, None),
            "mul" => match a.checked_mul(b) {
                Some(p) => (p, None),
                None => (a.wrapping_mul(b), Some(if (a < 0) == (b < 0) { 1 } else { -1 })),
            },
            other => panic!("{w}: unexpected op {other}"),
        };
        let got = read(&v.expected, bits, signed);
        assert_eq!(got, wrap(exact, bits, signed), "{w}: a={a} b={b} expected differs from the wrapping model");
        let sat = match beyond {
            Some(1) => saturate(i128::MAX, bits, signed),
            Some(_) => saturate(i128::MIN, bits, signed),
            None => saturate(exact, bits, signed),
        };
        if got != sat {
            *sat_differs.entry((v.dtype.clone(), v.op.clone())).or_default() += 1;
        }
    }
    for dtype in ["i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64"] {
        for op in ["add", "sub", "mul"] {
            let n = sat_differs.get(&(dtype.to_string(), op.to_string())).copied().unwrap_or(0);
            assert!(n >= 1, "{dtype} {op}: no cell separates wrapping from saturating");
        }
    }
}
