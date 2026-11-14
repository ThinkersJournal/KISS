// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Classify §6.6-0024 (CireSnave 2026-11-13, board 430 Q4): a request states each operand's dtype as the
//! operand actually is. The reference deriver takes every dtype the key carries from the operand it belongs to.

use kiss_conformance::derivation_vectors::vectors;
use kiss_conformance::derive::derive_structure_key;

fn cell_named(name: &str) -> kiss_conformance::derive::DerivationCell {
    vectors().iter().find(|d| d.name == name).unwrap_or_else(|| panic!("vector {name}")).cell.clone()
}

// Backs[enforced]: KISS-CLASSIFY-6.6-0024 — the deriver takes each dtype position of the key from the operand it belongs to (operand 0, the designated weight, the output), never from a default or a neighbour; at this schema version a non-primary operand of a non-ctr cell is not a key position (§6.6-0015), which the clause states and this test pins as the boundary
#[test]
fn test_classify_request_states_input_types() {
    // the control: a ctr GEMM [lhs, rhs=weight, out], all f16.
    let base = cell_named("ctr_gemm_small_m_large_nk");
    let k0 = derive_structure_key(&base).expect("control derives");
    assert_eq!((k0.dtype.as_str(), k0.contraction.as_ref().unwrap().wdt.as_str(), k0.contraction.as_ref().unwrap().out.as_str()), ("f16", "f16", "f16"));

    // change ONE operand's stated dtype; only the position that belongs to it moves.
    let mut c = base.clone();
    c.operands[1].dtype = "bf16"; // the designated weight (slot 1)
    let k = derive_structure_key(&c).expect("derives");
    assert_eq!(k.contraction.as_ref().unwrap().wdt, "bf16", "the weight dtype is the weight operand's, as stated");
    assert_eq!(k.dtype, "f16", "operand 0's dtype did not move");
    assert_eq!(k.contraction.as_ref().unwrap().out, "f16", "the output's did not move");
    assert_ne!(k.to_token(), k0.to_token(), "a misstated weight dtype would be a different request");

    let mut c = base.clone();
    c.operands[2].dtype = "bf16"; // the output
    let k = derive_structure_key(&c).expect("derives");
    assert_eq!(k.contraction.as_ref().unwrap().out, "bf16", "the output dtype is the output operand's");
    assert_eq!((k.dtype.as_str(), k.contraction.as_ref().unwrap().wdt.as_str()), ("f16", "f16"));

    let mut c = base.clone();
    c.operands[0].dtype = "bf16"; // the primary operand
    let k = derive_structure_key(&c).expect("derives");
    assert_eq!(k.dtype, "bf16", "field 2 is operand 0's dtype (§6.6-0005)");
    assert_eq!(k.contraction.as_ref().unwrap().wdt, "f16", "the weight stays the weight operand's, not operand 0's");

    // the boundary the clause states: a non-primary operand of a NON-ctr cell is not a key position at this
    // schema version (§6.6-0015), so a misstatement there changes no byte. The clause still binds the caller;
    // this control flips when the per-operand-dtype event lands (rfcs/per-operand-dtype-sk8.md).
    let ew = vectors()
        .iter()
        .find(|d| d.cell.contraction.is_none() && d.cell.operands.len() == 3 && derive_structure_key(&d.cell).map_or(false, |k| k.cell_class.starts_with("ew")))
        .expect("a two-input elementwise vector")
        .cell
        .clone();
    let ke = derive_structure_key(&ew).expect("control derives");
    let other = if ew.operands[1].dtype == "u32" { "i32" } else { "u32" };
    let mut c = ew.clone();
    c.operands[1].dtype = other;
    assert_eq!(derive_structure_key(&c).map(|k| k.to_token()), Ok(ke.to_token()), "sk7: a non-primary dtype is not keyed, so the key cannot witness it");

    // the spec states it, beside the derivation input (§6.6-0012).
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/classify.md"))
        .expect("read spec/classify.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(spec.contains("**KISS-CLASSIFY-6.6-0024** — A consumer that derives a `structure_key` for a request MUST state"));
    assert!(spec.contains("a **caller obligation**; it adds no field to a request"));
}
