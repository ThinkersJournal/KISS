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

