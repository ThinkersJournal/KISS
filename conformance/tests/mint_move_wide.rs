// SPDX-License-Identifier: MIT OR Apache-2.0
//! The f32/f64 sign-edit MOVE bundle `corpus/ops-move-wide-nan.json` (informative; no consumer
//! obligation) checked against an INDEPENDENT model: the sign edit done on typed integers
//! (`u32`/`u64` bit operations), where the minter edits byte 0 of a big-endian buffer.
//!
//! The cells must also DISCRIMINATE: for a signaling NaN, an implementation that quiets the NaN on
//! the way through (the promotion hazard) must produce different bits from the pinned expected.
//! That is asserted from the bit layout alone, so it does not depend on what this CPU does.
//!
//! Deliberately not a backing test for a clause (the bundle is offered, not required).

use kiss_conformance::corpus;

fn be(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0u64, |acc, b| (acc << 8) | u64::from(*b))
}

/// The expected result of a sign edit on a `width`-bit pattern, on integers.
fn model(op: &str, width: u32, a: u64, b: Option<u64>) -> u64 {
    let sign = 1u64 << (width - 1);
    match op {
        "neg" => a ^ sign,
        "abs" => a & !sign,
        "copysign" => (a & !sign) | (b.expect("copysign needs b") & sign),
        other => panic!("unexpected op {other}"),
    }
}

// harness-internal: checks the committed f32/f64 NaN-move bundle against an independent integer model and that its signaling-NaN cells would catch a quieting path (the bundle is offered, not a required obligation), no clause asserted
#[test]
fn move_wide_cells_match_the_integer_model_and_discriminate() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-move-wide-nan.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint -- --write`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 16, "8 cells at each of f32 and f64");

    let (mut snan_cells, mut finite_cells) = (0usize, 0usize);
    for v in &c.vectors {
        let (width, quiet_bit) = match v.dtype.as_str() {
            "f32" => (32u32, 1u64 << 22),
            "f64" => (64u32, 1u64 << 51),
            other => panic!("unexpected dtype {other}"),
        };
        let w = format!("{} tc {} {}", v.op, v.tc_id, v.dtype);
        let a = be(&v.inputs[0]);
        let b = v.inputs.get(1).map(|i| be(i));
        let got = be(&v.expected);
        assert_eq!(got, model(&v.op, width, a, b), "{w}: a={a:#x}");

        let exp_mask = if width == 32 { 0x7F80_0000u64 } else { 0x7FF0_0000_0000_0000u64 };
        let man_mask = (1u64 << (if width == 32 { 23 } else { 52 })) - 1;
        let is_nan = got & exp_mask == exp_mask && got & man_mask != 0;
        if is_nan && got & quiet_bit == 0 {
            // Signaling NaN moved: a quieting path returns `got | quiet_bit`, which is not the pinned bits.
            assert_ne!(got | quiet_bit, got, "{w}: the quieted result differs from the pinned one");
            snan_cells += 1;
        }
        if !is_nan {
            finite_cells += 1;
        }
    }
    // Per format: neg and abs on the signaling NaN, and copysign on it, are signaling; plus one finite control.
    assert_eq!(snan_cells, 2 * 3, "three signaling-NaN cells at each format");
    assert_eq!(finite_cells, 2, "one finite control at each format");
}
