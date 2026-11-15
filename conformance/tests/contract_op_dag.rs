// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform tests for the Semantics `op_dag` and the Identity consistency assertion, driven through
//! `section_vectors::read_full_document` (the reference whole-document reader) and the vectors of
//! `op_dag_vectors` that are published as `contract_vectors.json`'s `op_dag_vectors` key.
//!
//! Each test pairs the accepted documents (a reader that declined everything fails there) with
//! single-fault documents that must decline at the stated `(section, tag)` (a reader that accepted
//! everything fails there), so neither a vacuous accept nor a vacuous decline passes.

use kiss_conformance::op_dag::{check_op_dag, op_facts, parse_op_dag, OpDagDecline};
use kiss_conformance::op_dag_vectors::{accept_vectors, decline_vectors};
use kiss_conformance::section_vectors::{read_full_document, FullDecline};

fn declines_with(tags: &[&str]) {
    let selected: Vec<_> = decline_vectors()
        .into_iter()
        .filter(|v| tags.contains(&v.expect))
        .collect();
    assert!(!selected.is_empty(), "no vector for {tags:?}");
    for v in &selected {
        assert_eq!(
            read_full_document(&v.doc),
            Err(FullDecline {
                section: v.section,
                tag: v.expect
            }),
            "decline vector {:?}",
            v.name
        );
    }
}

// harness-internal: every op_dag accept document reads cleanly under the reference reader, the control of the decline tests below
#[test]
fn control_op_dag_accept_documents_read_cleanly() {
    let accepts = accept_vectors();
    assert_eq!(accepts.len(), 10);
    for v in &accepts {
        assert_eq!(
            read_full_document(&v.doc),
            Ok(()),
            "accept vector {:?}",
            v.name
        );
    }
}

// harness-internal: every op_dag decline document declines at the section and with the tag the generator states
#[test]
fn test_op_dag_decline_vectors_decline_with_the_stated_tag() {
    let declines = decline_vectors();
    assert_eq!(declines.len(), 29);
    for v in &declines {
        assert_eq!(
            read_full_document(&v.doc),
            Err(FullDecline {
                section: v.section,
                tag: v.expect
            }),
            "decline vector {:?}",
            v.name
        );
    }
}

/// Enforces[enforced]: KISS-CONTRACT-6.11-0007 — every child node appears strictly earlier than its
/// parent, so `child_edges` reference strictly-earlier indices and the root appears last; `Source` nodes
/// are spelled exactly; a reader MUST NOT place the root first nor reference a later index.
///
/// TEETH: the accept documents (a chain in post-order, a DAG using every `Source` tag and a `Bind`) pass;
/// a forward edge, a self edge, an out-of-range edge, a node nothing uses (root not last) and each
/// malformed `Source` spelling decline.
// Proven: KISS-CONTRACT-6.11-0007 (subject: impl; ref: contract_op_dag.rs)
#[test]
fn test_contract_op_dag_order_and_source_spelling() {
    for v in accept_vectors() {
        assert_eq!(read_full_document(&v.doc), Ok(()), "{:?}", v.name);
    }
    declines_with(&[
        "op-dag-child-not-earlier",
        "op-dag-multiple-roots",
        "op-dag-bad-source",
        "op-dag-malformed",
    ]);
    // The parser alone: the root-first document is well-formed text and is declined by the order rule.
    let root_first =
        parse_op_dag("[Op{add; ; [1, 2]}, Source{param; index=0}, Source{param; index=1}]")
            .unwrap();
    assert_eq!(
        check_op_dag(&root_first),
        Err(OpDagDecline::ChildNotEarlier)
    );
    let post_order =
        parse_op_dag("[Source{param; index=0}, Source{param; index=1}, Op{add; ; [0, 1]}]")
            .unwrap();
    assert_eq!(check_op_dag(&post_order), Ok(()));
}

/// Enforces[enforced]: KISS-CONTRACT-6.4-0002 — the op_dag's nodes carry bare KISS-Ops op names (no other
/// vocabulary, no Grammar tag inside `op_name`), and the DAG root equals the op the `op_identity` names.
///
/// TEETH: a DAG whose root and `op_identity` agree passes; a root of another op, an identity naming a
/// non-root op, a Source root, an op name outside KISS-Ops and a Grammar tag spelled inside `op_name`
/// each decline, and a `declared-op-tag` empty DAG is the accepted exception.
// Proven: KISS-CONTRACT-6.4-0002 (subject: impl; ref: contract_op_dag.rs)
#[test]
fn test_contract_op_dag_over_ops_and_grammar() {
    for v in accept_vectors() {
        assert_eq!(read_full_document(&v.doc), Ok(()), "{:?}", v.name);
    }
    declines_with(&["op-dag-unknown-op", "op-identity-root-mismatch"]);
    assert!(
        op_facts("matmul").is_some()
            && op_facts("frobnicate").is_none()
            && op_facts("add{axis=0}").is_none()
    );
}

/// Enforces[enforced]: KISS-CONTRACT-6.3-0006 — the cell class of the `accept_predicate` MUST be supported
/// by the op the `op_identity` names, decided from the facts the op declares: `ctr` a contraction, `ndx`
/// the index-operand role, `rdc` a declared reduce axis (an OpAttrs field or the inner carrier of a
/// non-carrier, KISS-OPS-6.19-0036); for a fusion any op of the DAG; a `declared-op-tag` with no node is
/// not rejected; the shape-decided classes ask nothing.
///
/// TEETH: each class passes on an op that has its fact (a scan, `argmax` and a pool for `rdc`, so the
/// inner-carrier arm is exercised) and declines on one that lacks it, including the facts crossed (`ndx`
/// on a contraction); the fusion passes on a non-root fact and declines without any.
// Proven: KISS-CONTRACT-6.3-0006 (subject: impl; ref: contract_op_dag.rs)
#[test]
fn test_contract_identity_consistency() {
    for v in accept_vectors() {
        assert_eq!(read_full_document(&v.doc), Ok(()), "{:?}", v.name);
    }
    declines_with(&["cell-class-unsupported-by-op"]);
}

/// Enforces[enforced]: KISS-CONTRACT-6.4-0007 — a `machine-checkable-IR` Semantics is a complete IR, so an
/// empty `op_dag` (no root) is declined; `semantics_kind` is exactly one of its two values.
///
/// TEETH: the empty DAG declines under `machine-checkable-IR` and is accepted under `declared-op-tag`.
// Proven: KISS-CONTRACT-6.4-0007 (subject: impl; ref: contract_op_dag.rs)
#[test]
fn test_contract_semantics_kind_matches_lift() {
    declines_with(&["op-dag-empty", "bad-semantics-kind"]);
    let exempt = accept_vectors()
        .into_iter()
        .find(|v| v.name.starts_with("declared-op-tag"))
        .expect("exempt vector");
    assert_eq!(read_full_document(&exempt.doc), Ok(()));
}
