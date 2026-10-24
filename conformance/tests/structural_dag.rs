// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.9 — the structural op-DAG equality comparator (tier-1 round-trip).
//!
//! The decomposition table is read from the KISS-Ops spec text itself (§6.13), so "resolve to
//! the primitive floor" runs through the owner's own rows, not a table the harness authored.
//! Every test pairs an EQUAL case with an UNEQUAL control that differs in exactly the property
//! under test, so a comparator that always says "equal" (or always "unequal") backs nothing.

use kiss_conformance::structural_dag::{structurally_equal, Dag, DagError, Decompositions, Node};

const OPS_SPEC: &str = include_str!("../../spec/ops.md");

fn table() -> Decompositions {
    let t = Decompositions::from_ops_spec(OPS_SPEC);
    // control: the reader found the rows this file relies on (a table that read nothing would
    // make every "equal" below vacuous: unresolved non-primitives would stay opaque nodes).
    for op in ["sigmoid", "relu", "silu", "hypot"] {
        assert!(t.templates.contains_key(op), "decomposition row for `{op}` not read from ops.md");
    }
    t
}

fn d(text: &str) -> Dag {
    Dag::parse_expr(text, &["x", "a", "b"]).unwrap_or_else(|e| panic!("parse `{text}`: {e:?}"))
}

fn eq(a: &str, b: &str) -> bool {
    structurally_equal(&d(a), &d(b), &table()).unwrap_or_else(|e| panic!("compare `{a}` vs `{b}`: {e:?}"))
}

/// KISS-CONFORM-6.9-0001 — resolve-to-floor, node/edge order, commutative order are normalized;
/// operand POSITION of a non-commutative op, OpAttrs bytes and sharing are not.
#[test]
fn test_conform_structural_dag_equality() {
    // (1) non-primitive vs floor-resolved form (sigmoid's §6.13 row: recip(add(const(1), exp(neg(x))))).
    assert!(eq("sigmoid(x)", "recip(add(const(1), exp(neg(x))))"), "KISS-CONFORM-6.9-0001: floor resolution");
    assert!(!eq("sigmoid(x)", "recip(add(const(1), exp(x)))"), "KISS-CONFORM-6.9-0001: a changed floor form must differ");
    // a chain: silu -> sigmoid -> floor, two levels.
    assert!(eq("silu(x)", "mul(x, recip(add(const(1), exp(neg(x)))))"), "KISS-CONFORM-6.9-0001: two-level resolution");

    // (3) commutative operand order is normalized; a positional op's is not.
    assert!(eq("add(a, b)", "add(b, a)"), "KISS-CONFORM-6.9-0001: commutative order");
    assert!(eq("mul(add(a, b), x)", "mul(x, add(b, a))"), "KISS-CONFORM-6.9-0001: nested commutative order");
    assert!(!eq("sub(a, b)", "sub(b, a)"), "KISS-CONFORM-6.9-0001: sub is positional");
    // hypot's row is sqrt(add(sqr(a), sqr(b))): swapping its operands is the commutative add inside.
    assert!(eq("hypot(a, b)", "hypot(b, a)"), "KISS-CONFORM-6.9-0001: commutativity reached through resolution");

    // (2) node order: the same DAG laid out with its nodes permuted compares equal.
    let a = d("add(mul(a, b), neg(x))");
    // post-order is [a, b, mul, x, neg, add]; rebuild with the x/neg branch laid out FIRST.
    let nodes = vec![
        Node { op: "input".into(), attrs: b"1".to_vec(), args: vec![] },    // 0: a
        Node { op: "input".into(), attrs: b"2".to_vec(), args: vec![] },    // 1: b
        Node { op: "input".into(), attrs: b"0".to_vec(), args: vec![] },    // 2: x
        Node { op: "neg".into(), attrs: vec![], args: vec![2] },            // 3
        Node { op: "mul".into(), attrs: vec![], args: vec![0, 1] },         // 4
        Node { op: "add".into(), attrs: vec![], args: vec![4, 3] },         // 5
    ];
    let permuted = Dag { nodes, root: 5 };
    assert_ne!(a.nodes, permuted.nodes, "control: the two layouts really are different arrays");
    assert!(structurally_equal(&a, &permuted, &table()).unwrap(), "KISS-CONFORM-6.9-0001: node order");

    // the OpAttrs byte channel is compared (identical ops, different attrs bytes).
    let mk = |attrs: &[u8]| Dag {
        nodes: vec![
            Node { op: "input".into(), attrs: b"0".to_vec(), args: vec![] },
            Node { op: "reduce".into(), attrs: attrs.to_vec(), args: vec![0] },
        ],
        root: 1,
    };
    let t = table();
    assert!(structurally_equal(&mk(&[1, 0, 0]), &mk(&[1, 0, 0]), &t).unwrap(), "KISS-CONFORM-6.9-0001");
    assert!(!structurally_equal(&mk(&[1, 0, 0]), &mk(&[1, 1, 0]), &t).unwrap(), "KISS-CONFORM-6.9-0001: OpAttrs bytes differ");

    // node SET: one shared `neg(x)` is not the same DAG as two separate `neg(x)` nodes.
    let shared = Dag {
        nodes: vec![
            Node { op: "input".into(), attrs: b"0".to_vec(), args: vec![] },
            Node { op: "neg".into(), attrs: vec![], args: vec![0] },
            Node { op: "sub".into(), attrs: vec![], args: vec![1, 1] },
        ],
        root: 2,
    };
    let duplicated = Dag {
        nodes: vec![
            Node { op: "input".into(), attrs: b"0".to_vec(), args: vec![] },
            Node { op: "neg".into(), attrs: vec![], args: vec![0] },
            Node { op: "neg".into(), attrs: vec![], args: vec![0] },
            Node { op: "sub".into(), attrs: vec![], args: vec![1, 2] },
        ],
        root: 3,
    };
    assert!(!structurally_equal(&shared, &duplicated, &t).unwrap(), "KISS-CONFORM-6.9-0001: node sets differ");

    // structural, not numeric: x*2 and x+x denote the same value but are different DAGs.
    assert!(!eq("mul(x, const(2))", "add(x, x)"), "KISS-CONFORM-6.9-0001: not a value comparator");
}

/// KISS-CONFORM-6.9-0002 — the comparator is over STRUCTURE: emitted sources that differ byte for
/// byte (spacing, input names) lift to equal DAGs, and a source that differs in structure does not.
#[test]
fn test_conform_structural_not_source_bytes() {
    let src_a = "mul(  x ,add(a,b) )";
    let src_b = "mul(x, add(b, a))";
    assert_ne!(src_a.as_bytes(), src_b.as_bytes(), "control: the two sources differ byte for byte");
    assert!(eq(src_a, src_b), "KISS-CONFORM-6.9-0002: byte-different sources, one structure");
    // the same op names, different structure: byte-LIKE sources must still differ.
    assert!(!eq("mul(x, add(a, b))", "add(x, mul(a, b))"), "KISS-CONFORM-6.9-0002: structure decides");
    // a source with a different input-binding order is a different DAG (inputs are positional).
    let t = table();
    let p = Dag::parse_expr("sub(p, q)", &["p", "q"]).unwrap();
    let q = Dag::parse_expr("sub(q, p)", &["p", "q"]).unwrap();
    assert!(!structurally_equal(&p, &q, &t).unwrap(), "KISS-CONFORM-6.9-0002: inputs are positional");
}

/// An op this module cannot read is a typed decline, never a silent primitive (guards the
/// "equal" verdicts above against a table that quietly read too little).
// harness-internal: guard that the decomposition-table reader declines an unreadable row rather than treating it as a primitive
#[test]
fn structural_dag_unreadable_decomposition_declines() {
    let t = table();
    assert!(!t.unreadable.is_empty(), "control: some §6.13 rows are let-bound and unreadable");
    let op = t.unreadable.iter().next().unwrap().clone();
    let dag = Dag::parse_expr(&format!("{op}(x)"), &["x"]).unwrap();
    // A guard, deliberately NOT a clause citation: it keeps the equal-verdicts above honest.
    assert_eq!(structurally_equal(&dag, &dag, &t), Err(DagError::Unresolvable(op)));
}
