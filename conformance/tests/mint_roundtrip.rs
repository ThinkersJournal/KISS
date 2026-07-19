//! The minted bundle must load back through the corpus reader and carry the
//! signed-zero add cell (the point of the slice: -0 vs +0 is normative, exact-byte).
use kiss_conformance::corpus;

#[test]
fn frozen_arith_bundle_loads_and_has_signed_zero_cell() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-arith.json"))
        .expect("frozen bundle must be committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses");
    assert!(c.vectors.iter().all(|v| v.op == "add"));
    // (-0) + (+0) = +0 under RNE — a cell that a normalize-to-+0 bug would still pass,
    // but (-0)+(-0) = -0 is the one that bites; both must be present.
    let neg_zero_sum = c.vectors.iter().find(|v| v.tags.iter().any(|t| t == "signed-zero")
        && v.inputs[0] == vec![0x80,0,0,0] && v.inputs[1] == vec![0x80,0,0,0]);
    let cell = neg_zero_sum.expect("(-0)+(-0) cell present");
    assert_eq!(cell.expected, vec![0x80, 0, 0, 0], "(-0)+(-0) = -0.0");
}
