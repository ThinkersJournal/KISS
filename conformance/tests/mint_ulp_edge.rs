// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ULP-distance edge bundle `corpus/ops-ulp-edge.json` (KISS-CONFORM-6.8-0002b, KISS#20 item 3)
//! checked against an INDEPENDENT model: a signed-integer reading of the bit pattern that shares no
//! code with `ulp_key` (which minted the cells).
//!
//! Model: read a pattern as a signed integer `-(magnitude)` when its sign bit is set and
//! `+(magnitude)` otherwise. Then the distance between two values is `|ia - ib|`, plus ONE when
//! they lie on opposite sides of the doubled zero (a negative-signed pattern, including `-0.0`,
//! against a non-negative-signed one), because `-0.0` and `+0.0` are distinct neighbours.
//!
//! Deliberately NOT a backing test for a clause (the bundle is offered, not required: no consumer
//! obligation), so it is tagged harness-internal.

use kiss_conformance::corpus;

fn signed(width: u32, bits: u64) -> i128 {
    let sign = 1u64 << (width - 1);
    let mag = (bits & (sign - 1)) as i128;
    if bits & sign != 0 {
        -mag
    } else {
        mag
    }
}

fn model(width: u32, a: u64, b: u64) -> u64 {
    let sign = 1u64 << (width - 1);
    let (neg_a, neg_b) = (a & sign != 0, b & sign != 0);
    let base = (signed(width, a) - signed(width, b)).unsigned_abs() as u64;
    base + u64::from(neg_a != neg_b)
}

fn be(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0u64, |acc, b| (acc << 8) | u64::from(*b))
}

// harness-internal: checks the committed ULP-distance edge bundle against an independent signed-integer model (the bundle is offered, not a required obligation), no clause asserted
#[test]
fn ulp_edge_cells_match_the_signed_integer_model() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-ulp-edge.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint -- --write`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 4 * 14, "14 edge pairs at each of f16, bf16, f32, f64");

    let (mut raw_wrong, mut seen_dtypes) = (0usize, std::collections::BTreeSet::new());
    for v in &c.vectors {
        let width = match v.dtype.as_str() {
            "f16" | "bf16" => 16,
            "f32" => 32,
            "f64" => 64,
            other => panic!("unexpected dtype {other}"),
        };
        let w = format!("ulp_distance tc {} {}", v.tc_id, v.dtype);
        assert_eq!(v.op, "ulp_distance", "{w}");
        assert_eq!(v.inputs.len(), 2, "{w}");
        assert_eq!(v.expected.len(), 8, "{w}: the distance is a u64");
        let (a, b) = (be(&v.inputs[0]), be(&v.inputs[1]));
        let got = be(&v.expected);
        assert_eq!(got, model(width, a, b), "{w}: a={a:#x} b={b:#x}");
        // The trap the clause rejects: raw subtraction of the sign-magnitude patterns.
        if a.abs_diff(b) != got {
            raw_wrong += 1;
        }
        seen_dtypes.insert(v.dtype.clone());
    }
    assert_eq!(seen_dtypes.len(), 4, "every format is represented");
    // Discrimination: a raw-subtraction implementation is wrong on a real share of the cells.
    // Seven of the fourteen pairs straddle the sign boundary (zero-signs, straddles-zero, inf-opposite,
    // cross-zero-one, extremes and the two symmetry swaps), at each of four formats.
    assert_eq!(raw_wrong, 4 * 7, "raw subtraction must disagree on exactly the cross-sign cells");

    // Spot goldens that do not come from either model: +-0 are one apart at every width.
    for v in c.vectors.iter().filter(|v| v.tags.iter().any(|t| t == "zero-signs")) {
        assert_eq!(be(&v.expected), 1, "{} +-0 are one apart", v.dtype);
    }
}
