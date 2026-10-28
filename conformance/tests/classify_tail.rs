// SPDX-License-Identifier: MIT OR Apache-2.0
//! Two KISS-Classify clauses that had a named test in the matrix and none in the harness (R3.2).
//! Each asserts the obligation against the reference codec, not the codec against itself.

use kiss_conformance::structure_key::*;

fn sub() -> OperandSubKey {
    OperandSubKey {
        contig: Contig::Contiguous,
        bcast_mask: 0x00,
        vec: VecWidth::V1,
        div: DivBucket::D8,
        flipped: false,
        dequant: None,
    }
}

fn red_key(dtype: &str, reduce: Reduce, acc_mp: Option<AccMp>) -> StructureKey {
    StructureKey {
        cell_class: "rdc".to_string(),
        dtype: dtype.to_string(),
        target: "cuda:sm89".to_string(),
        index_width: "ix32".to_string(),
        work_class: WorkClass::Warp,
        rank: 2,
        operands: vec![sub(), sub()],
        reduce,
        contraction: None,
        acc_mp,
    }
}

// Backs[enforced]: KISS-CLASSIFY-6.6-0005 — `structure_key.dtype` is operand-0's dtype, where operand-0 is fixed
// by the canonical ordering; it is NOT the output's, the last operand's, or any other position's.
#[test]
fn test_classify_structure_key_primary_dtype() {
    // Position is the whole rule: the same dtype multiset, reordered, derives a different primary.
    assert_eq!(derive_primary_dtype(&["f16", "f32", "f32"]), Some("f16"));
    assert_eq!(derive_primary_dtype(&["f32", "f16", "f32"]), Some("f32"));
    assert_eq!(derive_primary_dtype(&["bf16", "f32"]), Some("bf16"), "not the output's dtype");
    // operand-0 as a scale type has no primary dtype (§6.6-0021): undefined, not "the next operand".
    assert_eq!(derive_primary_dtype(&["f8e8m0", "f16"]), None);
    assert_eq!(derive_primary_dtype(&[]), None);

    // and it is what lands in token field 2 — the key's `dtype` field is the derived primary.
    for dts in [["f16", "f32", "f32"], ["i8", "f16", "f16"]] {
        let primary = derive_primary_dtype(&dts).expect("a value dtype at operand 0");
        let tok = red_key(primary, Reduce::All, None).to_token();
        assert_eq!(tok.split('|').nth(2), Some(dts[0]), "field 2 is operand-0's dtype: {tok}");
    }
}

// Backs[enforced]: KISS-CLASSIFY-6.7-0012 — a non-contraction reduction whose accumulator dtype differs from its
// compute dtype MUST carry an accumulator coordinate from the closed §6.1 set; absent means "equals
// the compute dtype". (Realized at sk4 by §6.7-0013's `<acc>/<mp>` field.)
#[test]
fn test_classify_reduction_accumulator_coordinate() {
    // absent coordinate: nine fields, no trailing precision field — the diagonal (acc == compute).
    let diag = red_key("f16", Reduce::Trailing, None).to_token();
    assert_eq!(diag.split('|').count(), 9, "{diag}");

    // a deviating accumulator is carried, drawn from the closed dtype set, and round-trips.
    let dev = red_key("f16", Reduce::Trailing, Some(AccMp { acc: "f32".into(), mp: MathFidelity::Stable }));
    let tok = dev.to_token();
    assert!(tok.ends_with("|rlast|f32/st"), "{tok}");
    assert_eq!(from_token(&tok).unwrap().acc_mp.map(|a| a.acc), Some("f32".to_string()));
    assert_ne!(tok, diag, "the f16-accumulating and f32-accumulating reductions are different cells");

    // the coordinate is a CLOSED-set dtype: an unknown or reserved spelling is a typed decline,
    // never silently accepted as "some accumulator".
    let stem = diag.clone();
    assert_eq!(from_token(&format!("{stem}|f99/st")), Err(KeyDecline::UnknownDtype));
    assert_eq!(from_token(&format!("{stem}|f8e6m2/st")), Err(KeyDecline::ReservedDtype));

    // the redundant diagonal spelling is not a second way to say "absent" (rule d).
    assert_eq!(from_token(&format!("{stem}|f16/st")), Err(KeyDecline::RedundantAccMpField));
}
