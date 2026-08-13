//! KISS-CONFORM-6.13-0020 — the free-text blurb is OUTSIDE the exact-byte scope.
//!
//! The clause is a **prohibition**: KISS-Conform MUST NOT byte-compare the optional
//! `human_annotation` field. A prohibition still has teeth, because the natural
//! wrong implementation is the obvious one — memcmp the whole contract. Under that
//! comparator two contracts whose machine-checkable content is byte-identical are
//! reported different because a human edited a comment, which is exactly the
//! outcome §6.0-0001 places the field outside the exact-byte scope to prevent.
//!
//! TEETH — five wrong implementations this catches. Assertions are labelled
//! (A)-(F) in the body; each entry names the one that catches it.
//!   1. **The whole-document memcmp.** Compares the blurb. Caught by (A).
//!   2. **The tautology** (`fn compare(_, _) -> Equal`). Passes any exclusion test
//!      that only ever asserts equality. Caught by (C), which differs in `op_dag` —
//!      an in-scope field in the *same block* as the blurb — and demands NOT-equal.
//!   3. **The over-stripper** that drops the whole Semantics block, or every
//!      optional-looking line. Passes (A) and (B) trivially. Also caught by (C),
//!      for the same reason: the block it would delete carries `op_dag`.
//!   4. **The document-wide stripper** that removes `human_annotation` from any
//!      block. Hides an *unknown-field* decline (§6.11-0005) behind this exclusion.
//!      Caught by (D).
//!   5. **The substring stripper** that removes any line CONTAINING the field's
//!      text rather than the field itself, silently dropping an `op_dag` that
//!      mentions it. Caught by (F).
//!
//! The remaining assertion, (E), is not a wrong implementation but the ordering
//! trap that makes the exclusion defeatable: the §6.11-0002 header carries `len=`
//! and `crc32=` over the body, so a blurb edit moves them even when the scoped
//! body is identical. The projection must precede header derivation.
//!
//! CITATION DISCIPLINE: this test cites ONLY `KISS-CONFORM-6.13-0020`.
//! Cross-references to KISS-Contract use the `§<sec>-<nnnn>` short form, which
//! does not match the citation grammar and so credits no other clause.

use kiss_conformance::contract::{
    exact_byte_scoped, render_block, serialize_op_dag, Document, OpTree, Value,
};

/// The Semantics block (§6.4-0001 field order), with an optional blurb and a
/// choice of `op_dag` root so an in-scope field can be varied independently.
fn semantics_block(op: &str, blurb: Option<&str>) -> Vec<u8> {
    let mut fields = vec![
        ("semantics_kind", Value::Str("machine-checkable-IR".into())),
        ("op_dag", Value::Str(serialize_op_dag(&OpTree::leaf(op)))),
    ];
    if let Some(b) = blurb {
        fields.push(("human_annotation", Value::Str(b.into())));
    }
    render_block(2, "semantics", &fields)
}

/// The Identity block (§6.3-0001 field order) — the in-scope neighbour that the
/// projection must leave completely alone.
fn identity_block() -> Vec<u8> {
    render_block(
        1,
        "identity",
        &[
            ("contract_kind", Value::Str("kiss-contract".into())),
            ("contract_version", Value::Str("1".into())),
            ("kernel_name", Value::Str("add_f32_strided_sm89".into())),
            ("revision_hash", Value::Blob(vec![0xde, 0xad, 0xbe, 0xef])),
            ("accept_predicate", Value::Str("bin/f32,f32,f32/strided/cuda:sm89".into())),
            ("op_identity", Value::Str("add".into())),
            ("target_capability", Value::Str("cuda:sm89".into())),
        ],
    )
}

/// A two-block contract body: Identity then Semantics.
fn body(op: &str, blurb: Option<&str>) -> Vec<u8> {
    let mut b = identity_block();
    b.extend_from_slice(&semantics_block(op, blurb));
    b
}

fn document(body: Vec<u8>) -> Vec<u8> {
    Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "1".into(),
        body,
    }
    .encode()
}

#[test]
fn test_conform_contract_blurb_excluded() {
    // (A) Two contracts differing ONLY in the blurb compare EQUAL once scoped —
    // and the raw bodies differ, so the exclusion is doing real work rather than
    // asserting something already true.
    let a = body("add", Some("tuned for sm89 in Q3"));
    let b = body("add", Some("DO NOT SHIP — scratch note"));
    assert_ne!(
        a, b,
        "KISS-CONFORM-6.13-0020: the two raw bodies must differ, or (A) is vacuous"
    );
    assert_eq!(
        exact_byte_scoped(&a),
        exact_byte_scoped(&b),
        "KISS-CONFORM-6.13-0020: contracts differing only in `human_annotation` were \
         reported different — the blurb was byte-compared"
    );

    // (B) Present-vs-absent is the same case: §6.4-0001 permits the field's
    // absence and omits its line, so an absent blurb must not read as a change.
    let none = body("add", None);
    assert_eq!(
        exact_byte_scoped(&a),
        exact_byte_scoped(&none),
        "KISS-CONFORM-6.13-0020: a contract with a blurb and one without must be \
         indistinguishable under the scoped comparator"
    );
    // The projection of an already-blurbless body is that body, byte for byte:
    // the exclusion removes the blurb and nothing else.
    assert_eq!(
        exact_byte_scoped(&none),
        none,
        "KISS-CONFORM-6.13-0020: the projection altered a body that has no blurb"
    );

    // (C) NOT vacuously equal. `op_dag` is in scope (§6.0-0001) and lives in the
    // SAME block as the blurb, so this single assertion kills both the tautology
    // and any stripper that deletes more than the one field.
    let other_op = body("mul", Some("tuned for sm89 in Q3"));
    assert_ne!(
        exact_byte_scoped(&a),
        exact_byte_scoped(&other_op),
        "KISS-CONFORM-6.13-0020: an in-scope Semantics field (`op_dag`) changed and \
         the comparator did not notice — it is excluding more than the blurb"
    );

    // (D) The exclusion is scoped to the Semantics block. The same key in Identity
    // is an unknown field, which a reader MUST decline (§6.11-0005); stripping it
    // here would hide that decline behind this clause.
    let mut smuggled = render_block(
        1,
        "identity",
        &[
            ("contract_kind", Value::Str("kiss-contract".into())),
            ("human_annotation", Value::Str("smuggled into Identity".into())),
        ],
    );
    smuggled.extend_from_slice(&semantics_block("add", None));
    assert_eq!(
        exact_byte_scoped(&smuggled),
        smuggled,
        "KISS-CONFORM-6.13-0020: `human_annotation` was stripped from a block that \
         does not define it — that hides an unknown-field decline"
    );

    // (F) The match is anchored at a line start, not a substring. An in-scope
    // value that merely CONTAINS the field's text stays. Catches a `contains()`
    // stripper, which would delete this `op_dag` line and silently drop the DAG.
    let embedded = render_block(
        2,
        "semantics",
        &[
            ("semantics_kind", Value::Str("declared-op-tag".into())),
            ("op_dag", Value::Str("[Op{add; note=human_annotation = x; []}]".into())),
        ],
    );
    assert_eq!(
        exact_byte_scoped(&embedded),
        embedded,
        "KISS-CONFORM-6.13-0020: a line was stripped for CONTAINING the field text \
         rather than being the field — the match must be anchored at the line start"
    );

    // (E) Order matters. The §6.11-0002 header derives `len=`/`crc32=` from the
    // body, so a blurb edit moves them: comparing whole documents defeats the
    // exclusion unless the projection is applied FIRST.
    assert_ne!(
        document(a.clone()),
        document(b.clone()),
        "KISS-CONFORM-6.13-0020: the blurb must reach the header, or (E) proves nothing"
    );
    assert_eq!(
        document(exact_byte_scoped(&a)),
        document(exact_byte_scoped(&b)),
        "KISS-CONFORM-6.13-0020: the blurb leaked back in through the header \
         `len=`/`crc32=` — project the body BEFORE deriving the header"
    );
}
