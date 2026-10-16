// SPDX-License-Identifier: MIT OR Apache-2.0
//! The cell-carrying derivation vectors (`corpus/structure_key_derivation_vectors.json`):
//! freshness, the Appendix A.1 anchors, and that every derived token is a token the codec reads back.

use kiss_conformance::derivation_vectors::{emit_json, expect_of, vectors, Expect};
use kiss_conformance::structure_key::from_token;

#[test]
fn test_derivation_vectors_artifact_is_fresh() {
    let committed = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/structure_key_derivation_vectors.json"))
        .expect("artifact committed; run `cargo run --bin emit_structure_key_derivation_vectors`");
    assert_eq!(committed.replace("
", "
"), emit_json(), "stale derivation-vectors artifact");
}

// Backs: KISS-CLASSIFY-6.6-0011 — the same cell derives the same bytes; and the A.1 goldens, derived
// here from RAW operands, equal the tokens hand-derived in the spec text (the independent anchor).
#[test]
fn test_classify_structure_key_derivation_canonical() {
    let vs = vectors();
    let (mut anchored, mut tokens, mut declines) = (0, 0, 0);
    for d in &vs {
        let first = expect_of(d);
        let second = expect_of(d);
        match (&first, &second) {
            (Expect::Token(a), Expect::Token(b)) => assert_eq!(a, b, "{}: derivation is not deterministic", d.name),
            (Expect::Decline(a), Expect::Decline(b)) => assert_eq!(a, b, "{}", d.name),
            _ => panic!("{}: derivation flipped between a token and a decline", d.name),
        }
        match first {
            Expect::Token(t) => {
                tokens += 1;
                // the derived token is well-formed: the codec reads it back byte-identically.
                let k = from_token(&t).unwrap_or_else(|e| panic!("{}: derived token declines on read: {e:?} ({t})", d.name));
                assert_eq!(k.to_token(), t, "{}", d.name);
                if let Some(g) = d.spec_golden {
                    assert_eq!(t, g, "{}: derived from the raw cell, differs from the Appendix A.1 golden", d.name);
                    anchored += 1;
                }
            }
            Expect::Decline(_) => {
                declines += 1;
                assert!(d.spec_golden.is_none(), "{}", d.name);
            }
        }
    }
    assert!(anchored >= 9, "the A.1 anchors must be present ({anchored})");
    assert!(declines >= 5, "the decline cases must be present ({declines})");
    assert!(tokens > declines);
}

// Backs: KISS-CLASSIFY-6.6-0008 — an extent-1 axis derives the same sub-key whatever stride it records.
// Discrimination the byte-match alone cannot give: pairs of cells that a mis-reading deriver would
// collapse (or split) must come out the way the clauses say.
#[test]
fn test_derivation_vectors_pairs_that_must_agree_or_differ() {
    let tok = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Token(t) => t,
        Expect::Decline(e) => panic!("{name} declines: {e:?}"),
    };
    // #520: an extent-1 axis derives the same sub-key whatever stride is recorded on it.
    assert_eq!(tok("extent1_axis_with_nonzero_stride"), tok("extent1_axis_with_zero_stride"));
    // §6.5-0013: a flipped run is not a forward-unit run — it must NOT vectorize like the plain one.
    assert_ne!(tok("flipped_inner_axis"), tok("a1_binary_add"));
    // the 2^31 offset boundary is exclusive.
    assert!(tok("index_width_just_below_2_31").contains("|ix32|"));
    assert!(tok("index_width_at_2_31").contains("|ix64|"));
    // work-class boundaries: <=32 warp, <=1024 block, else grid.
    assert!(tok("work_class_32_is_warp").contains("|warp|"));
    assert!(tok("work_class_1024_is_block").contains("|block|"));
    assert!(tok("work_class_1025_is_grid").contains("|grid|"));
}

// The innermost ACTIVE axis is axis `rank-1` whatever its extent (§6.3-0011, §6.3-0002, §6.5-0012): the
// two worked examples of the clause, and the innermost-extent-0 case a consumer held open. Each token's
// operand sub-key is `<layout>/<mask>/<width>/<bucket>/<flag>`; the bucket is its fourth `/` part. The
// three buckets are DIFFERENT, so a deriver that skips unit/zero axes (or reads axis 0) cannot pass.
#[test]
fn derivation_vectors_innermost_axis_is_rank_minus_1_including_extents_one_and_zero() {
    let tok = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Token(t) => t,
        Expect::Decline(e) => panic!("{name} declines: {e:?}"),
    };
    let bucket = |t: &str| t.split('|').nth(7).unwrap().split(';').next().unwrap().split('/').nth(3).unwrap().to_string();
    assert_eq!(bucket(&tok("innermost_axis_extent_one_is_axis_rank_minus_1")), "da", "[4,1]: axis 1, E = 1");
    assert_eq!(bucket(&tok("innermost_axis_extent_eight_after_unit_axis")), "d8", "[1,8]: axis 1, E = 8");
    assert_eq!(bucket(&tok("innermost_axis_extent_zero_is_axis_rank_minus_1")), "da", "[8,0]: axis 1, E = 0, not d8");
}

// Backs: KISS-CLASSIFY-6.6-0014 — operands derive in CANONICAL order (inputs and in-place operands in
// call order, then outputs), whatever order the caller listed them; an in-place operand is one input,
// counted once, and is not repeated among the outputs.
#[test]
fn test_classify_operand_canonical_order() {
    let tok = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Token(t) => t,
        Expect::Decline(e) => panic!("{name} declines: {e:?}"),
    };
    // the output listed FIRST in call order derives the SAME bytes as inputs-then-output.
    assert_eq!(tok("call_order_is_not_canonical_order"), tok("a1_binary_add"));
    // an in-place accumulate (acc, b) is TWO operands, not three: acc is not repeated as an output.
    let n_ops = |t: &str| t.split('|').nth(7).unwrap().split(';').count();
    assert_eq!(n_ops(&tok("a1_inplace_accumulate")), 2);
    assert_eq!(n_ops(&tok("a1_binary_add")), 3);
    // operand 0 is the first INPUT: the primary dtype comes from it, not from the output listed first.
    let t = tok("call_order_is_not_canonical_order");
    assert_eq!(t.split('|').nth(2), Some("f32"));
}


// Backs: KISS-CLASSIFY-6.6-0016 — the M/N/K size classes and the K-divisibility bucket come from the
// CALLER-DECLARED axes: the same operands with M and N swapped give a different key, and K's bucket comes
// from the K axis, not from the output's innermost axis. Also backs the §2.8(d) worked example.
#[test]
fn test_classify_contraction_axis_roles() {
    let tok = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Token(t) => t,
        Expect::Decline(e) => panic!("{name} declines: {e:?}"),
    };
    let ctr = |t: &str| t.split('|').nth(9).unwrap().to_string();
    assert_eq!(ctr(&tok("ctr_gemm_small_m_large_nk")), "ctll/d16/f16/f16/f16/st", "spec §2.8(d): M tiny, N and K large, d16");
    assert_eq!(ctr(&tok("ctr_gemm_roles_swapped_m_and_n")), "cltl/d16/f16/f16/f16/st", "hint swapped, operands held");
    assert_ne!(tok("ctr_gemm_small_m_large_nk"), tok("ctr_gemm_roles_swapped_m_and_n"));
    assert_eq!(ctr(&tok("ctr_k_divisibility_from_k_axis")), "ctll/d4/f16/f16/f16/st", "K = 4100 -> d4; N = 4096 would be d16");
}

// Backs: KISS-CLASSIFY-6.6-0023 — the batch coordinate is a declaration: absent when none is declared,
// present for a declared axis of extent 1, classed by the PRODUCT of the declared extents, rank-independent.
#[test]
fn test_classify_batch_role_derivation_in_cells() {
    let tok = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Token(t) => t,
        Expect::Decline(e) => panic!("{name} declines: {e:?}"),
    };
    let ctr = |name: &str| tok(name).split('|').nth(9).unwrap().to_string();
    assert_eq!(ctr("ctr_rank3_batch_declared"), "ctlm/d16/bs/f16/f16/f16/st");
    assert_eq!(ctr("ctr_rank3_batch_not_declared"), "ctlm/d16/f16/f16/f16/st");
    assert_eq!(ctr("ctr_batch_declared_extent_one"), "ctlm/d16/bt/f16/f16/f16/st");
    assert_eq!(ctr("ctr_rank4_two_batch_axes_product"), "ctlm/d16/bm/f16/f16/f16/st", "4 * 64 = 256 -> mid");
    assert_eq!(ctr("ctr_rank4_one_batch_axis_declared"), "ctlm/d16/bs/f16/f16/f16/st", "only axis 1 declared -> 64");
    assert_eq!(ctr("ctr_rank4_no_batch_declared"), "ctlm/d16/f16/f16/f16/st", "rank 4 alone does not batch");
    let decline = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Decline(e) => e.wire(),
        Expect::Token(t) => panic!("{name} derived {t}"),
    };
    assert_eq!(decline("decline_ctr_batch_axis_out_of_range"), "BatchAxisOutOfRange");
    assert_eq!(decline("decline_ctr_batch_axis_duplicate"), "BatchAxisDuplicate");
    assert_eq!(decline("decline_ctr_batch_axis_is_an_mnk_axis"), "BatchAxisOverlapsContractionRole");
    assert_eq!(decline("decline_ctr_batch_axis_not_in_output"), "BatchAxisNotInOutput");
    assert_eq!(decline("decline_ctr_batch_axis_zero_extent"), "BatchAxisNonPositiveExtent");
}

// Backs: KISS-CLASSIFY-6.6-0019 — <wdt> is the dtype of the DECLARED weight operand: with call order
// [weight, scale, activation] a positional operand-1 read would name f8e8m0.
// Backs: KISS-CLASSIFY-6.6-0022 — the scale dtype is keyed (e8m0 and f16 scales are different cells) and is
// omitted, not empty, when no scale operand is designated.
#[test]
fn test_classify_weight_and_scale_roles_in_cells() {
    let tok = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Token(t) => t,
        Expect::Decline(e) => panic!("{name} declines: {e:?}"),
    };
    let ctr = |name: &str| tok(name).split('|').nth(9).unwrap().to_string();
    assert_eq!(ctr("ctr_quantized_scale_e8m0"), "ctll/d16/i4/bf16/bf16/st/qf8e8m0");
    assert_eq!(ctr("ctr_quantized_scale_f16"), "ctll/d16/i4/bf16/bf16/st/qf16");
    assert_eq!(ctr("ctr_quantized_no_scale_designated"), "ctll/d16/i4/bf16/bf16/st");
    assert_ne!(tok("ctr_quantized_scale_e8m0"), tok("ctr_quantized_scale_f16"));
    // weight at operand 0, scale at operand 1: wdt is the declared weight (i4), not the operand-1 dtype.
    assert_eq!(ctr("ctr_weight_role_not_operand_one"), "ctll/d16/i4/bf16/bf16/st/qf8e8m0");
    // weight NOT at operand 0, accumulator undeclared: operand 0's bf16, not the weight's i4.
    assert_eq!(ctr("ctr_weight_second_undeclared_accumulator_defaults_to_operand0_dtype"), "ctll/d16/i4/bf16/bf16/st/qf8e8m0");
    // the trap: undeclared accumulator on a weight-first call order defaults to the weight's i4 (the primary dtype);
    // the two cells differ in <acc> only.
    assert_eq!(ctr("ctr_weight_first_undeclared_accumulator_defaults_to_operand0_dtype"), "ctll/d16/i4/i4/bf16/st/qf8e8m0");
    let decline = |name: &str| match expect_of(vectors().iter().find(|d| d.name == name).expect(name)) {
        Expect::Decline(e) => e.wire(),
        Expect::Token(t) => panic!("{name} derived {t}"),
    };
    assert_eq!(decline("decline_ctr_weight_slot_out_of_range"), "RoleSlotOutOfRange");
    assert_eq!(decline("decline_ctr_scale_slot_out_of_range"), "RoleSlotOutOfRange");
    assert_eq!(decline("decline_ctr_reserved_weight_dtype"), "ReservedDtype");
}

// Backs: KISS-CLASSIFY-6.6-0012 — the role hints are INPUTS, the cell class is not: the same operands derive
// `ctr` iff the caller declares the contraction roles (never from shape), and every hint moves the key.
// The index-slot and reduced-axis hints are exercised in classify_storage_dtypes.rs and the A.1 reductions.
#[test]
fn test_classify_derivation_input_tuple() {
    use kiss_conformance::derive::derive_structure_key;
    let all = vectors();
    let gemm = &all.iter().find(|d| d.name == "ctr_gemm_small_m_large_nk").expect("ctr vector").cell;
    let declared = derive_structure_key(gemm).expect("control: the declared cell derives");
    assert_eq!(declared.cell_class, "ctr");
    let mut undeclared = gemm.clone();
    undeclared.contraction = None;
    let k = derive_structure_key(&undeclared).expect("the same operands derive without the roles");
    assert_ne!(k.cell_class, "ctr", "a GEMM-shaped cell is not ctr unless the caller declares the roles");
    assert!(k.contraction.is_none() && declared.contraction.is_some());
    // each hint is an input: changing only the hint changes the key (operands, target held).
    let base = declared.to_token();
    let mut c = gemm.clone();
    c.contraction.as_mut().unwrap().scale = Some(0);
    assert_ne!(derive_structure_key(&c).unwrap().to_token(), base, "a designated scale operand is keyed");
    let mut c = gemm.clone();
    c.contraction.as_mut().unwrap().batch = vec![0];
    assert_ne!(derive_structure_key(&c).map(|k| k.to_token()), Ok(base), "a declared batch axis is keyed or declines, never ignored");
}
