// SPDX-License-Identifier: MIT OR Apache-2.0
//! Comparison-mask and bool-normalization vectors (KISS-OPS-6.2-0005 / -0006). Before this bundle the
//! corpus carried no comparison cell and no `bool` cell, so an implementation returning a 4-byte
//! `1.0f32` for `cmp_lt`, or forwarding the byte `0x02` out of a `logical_and`, passed everything.
//!
//! Teeth: every cell is recomputed from an INDEPENDENT model (f32 widened to f64 for the float
//! predicates, i32/u32 read both ways for the integer ones, "non-zero byte" for the bool consumers), and
//! the tests assert the discriminations: every expected is ONE byte, 0 or 1, and declared `bool`; the u32
//! cells include pairs on which a signed reading flips the answer; the bool cells include operands that
//! are neither 0 nor 1.

use kiss_conformance::corpus;

fn f32_of(b: &[u8]) -> f64 {
    f64::from(f32::from_bits(u32::from_be_bytes(b.try_into().expect("4 bytes"))))
}

fn u32_of(b: &[u8]) -> u32 {
    u32::from_be_bytes(b.try_into().expect("4 bytes"))
}

fn pred(op: &str, lt: bool, eq: bool, gt: bool) -> bool {
    match op {
        "cmp_eq" => eq,
        "cmp_ne" => !eq,
        "cmp_lt" => lt,
        "cmp_le" => lt || eq,
        "cmp_gt" => gt,
        "cmp_ge" => gt || eq,
        other => panic!("unexpected comparison {other}"),
    }
}

fn load() -> corpus::Corpus {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-compare-bool.json");
    let text = std::fs::read_to_string(path).expect("bundle committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 6 * (12 + 5 + 5) + 2 * 7 * 3 + 7);
    // Every cell declares its expected dtype as `bool` (the loader does not keep it; read the text).
    assert_eq!(text.matches("\"expected\": {\"dtype\":\"bool\"").count(), c.vectors.len());
    c
}

// Backs: KISS-OPS-6.2-0005 - a comparison yields a one-byte bool mask, strictly 0 or 1, for f32/i32/u32
// operands, whatever the compute dtype.
#[test]
fn test_ops_compare_result_zero_one() {
    let c = load();
    let (mut cmp, mut unsigned_flip, mut nan_cells) = (0, 0, 0);
    for v in c.vectors.iter().filter(|v| v.op.starts_with("cmp_")) {
        let w = format!("{} {} tc {}", v.dtype, v.op, v.tc_id);
        assert_eq!(v.expected.len(), 1, "{w}: a mask is ONE byte, not a compute-dtype value");
        assert!(v.expected[0] <= 1, "{w}: a mask byte is strictly 0 or 1, got {:#04x}", v.expected[0]);
        cmp += 1;
        let (a, b) = (&v.inputs[0], &v.inputs[1]);
        let (lt, eq, gt) = match v.dtype.as_str() {
            "f32" => {
                let (x, y) = (f32_of(a), f32_of(b));
                if x.is_nan() || y.is_nan() {
                    nan_cells += 1;
                }
                (x < y, x == y, x > y)
            }
            "i32" => {
                let (x, y) = (u32_of(a) as i32, u32_of(b) as i32);
                (x < y, x == y, x > y)
            }
            "u32" => {
                let (x, y) = (u32_of(a), u32_of(b));
                if ((x as i32) < (y as i32)) != (x < y) {
                    unsigned_flip += 1;
                }
                (x < y, x == y, x > y)
            }
            other => panic!("{w}: unexpected dtype {other}"),
        };
        assert_eq!(v.expected[0] == 1, pred(&v.op, lt, eq, gt), "{w}");
    }
    assert_eq!(cmp, 6 * 22);
    assert!(nan_cells >= 6 * 3, "NaN cells present for every comparison");
    assert!(unsigned_flip >= 6, "u32 cells on which a signed reading differs must be present");
}

// Backs: KISS-OPS-6.2-0006 - a bool consumer reads any non-zero byte as true, and its result is
// normalized to strictly 0 or 1.
#[test]
fn test_ops_bool_normalization() {
    let c = load();
    let (mut n, mut odd_in, mut odd_true) = (0, 0, 0);
    for v in c.vectors.iter().filter(|v| v.op.starts_with("logical_")) {
        let w = format!("{} tc {}", v.op, v.tc_id);
        assert_eq!(v.dtype, "bool", "{w}");
        assert_eq!(v.expected.len(), 1, "{w}");
        assert!(v.expected[0] <= 1, "{w}: result normalized to strictly 0/1, got {:#04x}", v.expected[0]);
        let ins: Vec<bool> = v.inputs.iter().map(|i| i[0] != 0).collect();
        let want = match v.op.as_str() {
            "logical_and" => ins[0] && ins[1],
            "logical_or" => ins[0] || ins[1],
            "logical_not" => !ins[0],
            other => panic!("{w}: unexpected op {other}"),
        };
        assert_eq!(v.expected[0] == 1, want, "{w}");
        n += 1;
        if v.inputs.iter().any(|i| i[0] > 1) {
            odd_in += 1;
            if v.expected[0] == 1 {
                odd_true += 1;
            }
        }
    }
    assert_eq!(n, 2 * 7 * 3 + 7);
    assert!(odd_in >= 20 && odd_true >= 5, "bool cells with non-0/1 operands that read true");
}
