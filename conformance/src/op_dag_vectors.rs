// SPDX-License-Identifier: MIT OR Apache-2.0
//! `op_dag` and Identity-consistency contract vectors (KISS-CONTRACT-6.11-0007, 6.4-0002, 6.4-0007,
//! 6.3-0006): whole seven-section documents a reader must ACCEPT, and single-fault documents it must
//! DECLINE with a stable `(section, tag)`.
//!
//! Built like the section vectors (`section_vectors.rs`): the base is the Appendix C document, one
//! document differs from it in the Identity `op_identity`/`accept_predicate` (and the Capabilities copy of
//! the predicate, §6.7-0002) and the Semantics `semantics_kind`/`op_dag`; the generator states the
//! `(section, tag)` it intends and [`read_full_document`], the reader the tests run, must return exactly
//! that pair, so an intended tag the reader does not produce fails the build instead of being emitted.
//!
//! The facts the cell-class assertion reads are `conformance/corpus/op_facts.json`, generated from
//! `spec/ops.md`; no per-op table is written here.

use crate::section_vectors::{
    assemble, base_blocks, fields_of, render, set, AcceptVector, SectionDeclineVector,
};

/// Real `sk8` keys (the `structure_key_vectors.json` positives of the same name) of the classes the
/// assertion reads. Only field 2 (the cell class) matters to the assertion; the rest is a well-formed key.
const CTR: &str = "sk8|ctr|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f/f32;co/00/v4/d16/f/f32;co/00/v4/d16/f/f32|-|ctll/d16/f32/f32/f32/st";
const NDX: &str =
    "sk8|ndx|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f/f32;co/00/v4/d16/f/u32;co/00/v4/d16/f/f32|-";
const RDC: &str = "sk8|rdc|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f/f32;co/03/v1/da/f/f32|rall";
const EW2: &str =
    "sk8|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f/f32;st/00/v1/d16/f/f32;co/00/v4/d16/f/f32|-";

const IR: &str = "machine-checkable-IR";
const TAG: &str = "declared-op-tag";

/// The Appendix C document with the four Identity/Semantics fields (and the Capabilities predicate) replaced.
fn doc(kind: &str, op_dag: &str, op_identity: &str, accept: &str) -> Vec<u8> {
    let mut blocks = base_blocks();
    let mut id = fields_of(&blocks[0]);
    set(&mut id, "op_identity", op_identity);
    set(&mut id, "accept_predicate", accept);
    blocks[0] = render(0, &id);
    let mut sem = fields_of(&blocks[1]);
    set(&mut sem, "semantics_kind", kind);
    set(&mut sem, "op_dag", op_dag);
    blocks[1] = render(1, &sem);
    let mut caps = fields_of(&blocks[4]);
    set(&mut caps, "accept_predicate", accept);
    blocks[4] = render(4, &caps);
    assemble(&blocks)
}

const P0: &str = "Source{param; index=0}";
const P1: &str = "Source{param; index=1}";

fn dag(nodes: &[&str]) -> String {
    format!("[{}]", nodes.join(", "))
}

/// A one-node DAG whose root is `op` with no attributes and no children.
fn one(op: &str) -> String {
    dag(&[&format!("Op{{{op}; ; []}}")])
}

/// An `Op{neg; ; [edges]}` node.
fn neg(edges: &str) -> String {
    format!("Op{{neg; ; [{edges}]}}")
}

fn v(name: &str, d: Vec<u8>) -> AcceptVector {
    AcceptVector {
        name: name.to_string(),
        doc: d,
    }
}

/// The documents a reader accepts.
pub fn accept_vectors() -> Vec<AcceptVector> {
    let mut out = accept_class_vectors();
    out.extend(accept_shape_vectors());
    out
}

/// One accepted cell class per vector: the op's fact is what lets the class stand.
fn accept_class_vectors() -> Vec<AcceptVector> {
    vec![
        v(
            "ctr-class-on-a-contraction",
            doc(IR, &one("matmul"), "matmul", CTR),
        ),
        v(
            "ndx-class-on-an-op-with-the-index-role",
            doc(IR, &one("gather"), "gather", NDX),
        ),
        v(
            "rdc-class-on-a-carrier-reduce",
            doc(IR, &one("reduce_mean"), "reduce_mean", RDC),
        ),
        v(
            "rdc-class-on-a-scan-via-its-inner-carrier",
            doc(IR, &one("cumsum"), "cumsum", RDC),
        ),
        v(
            "rdc-class-on-argmax-via-sort-network",
            doc(IR, &one("argmax"), "argmax", RDC),
        ),
        v(
            "rdc-class-on-a-pool-via-reduce",
            doc(IR, &one("avg_pool"), "avg_pool", RDC),
        ),
    ]
}

/// DAG shapes a reader accepts: no fact needed, a fused graph, every Source tag, an exempt tag.
fn accept_shape_vectors() -> Vec<AcceptVector> {
    let sources = dag(&[
        "Source{const; dtype=f32, bits=0x3F800000}",
        P0,
        "Source{coord; axis=1}",
        "Source{extent; axis=0}",
        "Source{reduced; stage=0}",
        "Source{reduced_count; }",
        "Bind(0)",
        "Op{add; ; [0, 1]}",
        "Op{add; ; [2, 3]}",
        "Op{add; ; [4, 5]}",
        "Op{add; ; [7, 8]}",
        "Op{add; ; [9, 6]}",
        "Op{add; ; [10, 11]}",
    ]);
    let fusion = dag(&["Op{matmul; ; []}", P0, "Op{add; ; [0, 1]}"]);
    vec![
        v(
            "shape-decided-class-asks-nothing-of-the-op",
            doc(IR, &one("add"), "add", EW2),
        ),
        v(
            "fusion-fact-only-on-a-non-root-op",
            doc(IR, &fusion, "add", CTR),
        ),
        v(
            "every-source-tag-and-a-bind-node",
            doc(IR, &sources, "add", EW2),
        ),
        v(
            "declared-op-tag-with-no-node-is-exempt",
            doc(TAG, "[]", "matmul", CTR),
        ),
    ]
}

/// One fault each. `section` is where the reader declines; `expect` is its stable tag.
pub fn decline_vectors() -> Vec<SectionDeclineVector> {
    let mut out = Declines(Vec::new());
    decline_edges(&mut out);
    decline_sources(&mut out);
    decline_names(&mut out);
    decline_identity(&mut out);
    decline_class(&mut out);
    out.0
}

struct Declines(Vec<SectionDeclineVector>);

impl Declines {
    fn add(&mut self, name: &str, d: Vec<u8>, section: &'static str, expect: &'static str) {
        self.0.push(SectionDeclineVector {
            name: name.to_string(),
            doc: d,
            section,
            expect,
        });
    }
}

fn decline_edges(out: &mut Declines) {
    // §6.11-0007: children strictly earlier, root last.
    out.add(
        "root first: the root's child edges point forward",
        doc(IR, &dag(&["Op{add; ; [1, 2]}", P0, P1]), "add", EW2),
        "semantics",
        "op-dag-child-not-earlier",
    );
    out.add(
        "a node is its own child",
        doc(IR, &dag(&[&neg("0")]), "neg", EW2),
        "semantics",
        "op-dag-child-not-earlier",
    );
    out.add(
        "a child index past the end of the array",
        doc(IR, &dag(&[&neg("5")]), "neg", EW2),
        "semantics",
        "op-dag-child-not-earlier",
    );
    out.add(
        "root not last: an earlier node nothing uses",
        doc(IR, &dag(&[P0, P1, &neg("0")]), "neg", EW2),
        "semantics",
        "op-dag-multiple-roots",
    );
    out.add(
        "not an array",
        doc(IR, "Op{add; ; []}", "add", EW2),
        "semantics",
        "op-dag-malformed",
    );
    out.add(
        "a node of no kind",
        doc(IR, &dag(&["Tap{add; ; []}"]), "add", EW2),
        "semantics",
        "op-dag-malformed",
    );
    out.add(
        "child edges not in canonical spelling",
        doc(IR, &dag(&[P0, "Op{neg; ; [00]}"]), "neg", EW2),
        "semantics",
        "op-dag-malformed",
    );
}

fn decline_sources(out: &mut Declines) {
    // §6.11-0007 Source nodes.
    for (what, node) in [
        ("an unknown Source tag", "Source{iota; index=0}"),
        (
            "const bits in lower case",
            "Source{const; dtype=f32, bits=0x3f800000}",
        ),
        (
            "const bits not zero-padded to the dtype width",
            "Source{const; dtype=f32, bits=0x3F80}",
        ),
        (
            "const with an unknown dtype token",
            "Source{const; dtype=f99, bits=0x3F800000}",
        ),
        ("param index negative", "Source{param; index=-1}"),
        ("coord axis past u8", "Source{coord; axis=256}"),
        ("extent without its axis", "Source{extent; }"),
        ("reduced stage as a float", "Source{reduced; stage=1.5}"),
        (
            "reduced_count with attributes",
            "Source{reduced_count; stage=0}",
        ),
    ] {
        out.add(
            &format!("malformed Source: {what}"),
            doc(IR, &dag(&[node, &neg("0")]), "neg", EW2),
            "semantics",
            "op-dag-bad-source",
        );
    }
}

fn decline_names(out: &mut Declines) {
    // §6.4-0002: only KISS-Ops op names.
    out.add(
        "an op name outside KISS-Ops",
        doc(IR, &dag(&["Op{frobnicate; ; []}"]), "frobnicate", EW2),
        "semantics",
        "op-dag-unknown-op",
    );
    out.add(
        "a Grammar tag spelled inside op_name",
        doc(IR, &dag(&["Op{add{axis=0}; ; []}"]), "add", EW2),
        "semantics",
        "op-dag-unknown-op",
    );
    // §6.4-0001/-0007: the kind and the empty DAG.
    out.add(
        "a machine-checkable-IR Semantics with no node",
        doc(IR, "[]", "add", EW2),
        "semantics",
        "op-dag-empty",
    );
    out.add(
        "a semantics_kind outside the two values",
        doc("lifted-IR", &dag(&["Op{add; ; []}"]), "add", EW2),
        "semantics",
        "bad-semantics-kind",
    );
}

fn decline_identity(out: &mut Declines) {
    // §6.4-0002: root vs op_identity.
    out.add(
        "DAG root is mul, op_identity names add",
        doc(IR, &dag(&["Op{mul; ; []}"]), "add", EW2),
        "record",
        "op-identity-root-mismatch",
    );
    out.add(
        "op_identity names another op than the root",
        doc(IR, &dag(&[P0, &neg("0")]), "abs", EW2),
        "record",
        "op-identity-root-mismatch",
    );
    out.add(
        "the DAG root is a Source node",
        doc(IR, &dag(&[P0]), "add", EW2),
        "record",
        "op-identity-root-mismatch",
    );
}

fn decline_class(out: &mut Declines) {
    // §6.3-0006: cell class vs the op's facts.
    out.add(
        "ctr class on an op that is not a contraction",
        doc(IR, &one("add"), "add", CTR),
        "record",
        "cell-class-unsupported-by-op",
    );
    out.add(
        "ndx class on an op without the index role",
        doc(IR, &one("add"), "add", NDX),
        "record",
        "cell-class-unsupported-by-op",
    );
    out.add(
        "rdc class on an op that declares no reduce axis",
        doc(IR, &one("add"), "add", RDC),
        "record",
        "cell-class-unsupported-by-op",
    );
    out.add(
        "ndx class on a contraction (the facts are per class, not shared)",
        doc(IR, &one("matmul"), "matmul", NDX),
        "record",
        "cell-class-unsupported-by-op",
    );
    out.add(
        "a fusion in which no op has the fact",
        doc(IR, &dag(&[P0, P1, "Op{add; ; [0, 1]}"]), "add", CTR),
        "record",
        "cell-class-unsupported-by-op",
    );
    out.add(
        "declared-op-tag exempt only when empty: a node without the fact declines",
        doc(TAG, &one("add"), "add", CTR),
        "record",
        "cell-class-unsupported-by-op",
    );
}

/// The `op_dag_vectors` key of `contract_vectors.json`, rendered here so the generator is one function.
pub fn emit_op_dag_vectors_json() -> String {
    let jstr = crate::json::escape_string;
    let hex = crate::hex;
    let mut s = String::new();
    s.push_str("  \"op_dag_vectors\": {\n");
    s.push_str(&format!(
        "    \"spec_reference\": {},\n",
        jstr("spec/contract.md 6.3-0006, 6.4-0001, 6.4-0002, 6.4-0007, 6.11-0007, 6.11-0009: whole seven-section documents (header v4), ONE op_dag or identity fault each. Decided from conformance/corpus/op_facts.json (generated from spec/ops.md), never from a per-op table.")
    ));
    s.push_str(&format!(
        "    \"encoding_note\": {},\n",
        jstr("accept_vectors[].doc_hex must read cleanly. decline_vectors[].section is where the reader declines (`semantics` for the op_dag value, `record` for a cross-field assertion) and expect is its stable tag. The Capabilities accept_predicate always equals the Identity one, so no decline here is the 6.7-0002 equality.")
    ));
    s.push_str(&format!(
        "    \"rulings_pinned\": {},\n",
        jstr("(1) A machine-checkable-IR Semantics MUST have at least one node: it is a complete IR (6.4-0007) whose root equals the op_identity (6.4-0002), and an empty array has no root. (2) A declared-op-tag Semantics MAY carry an empty op_dag and is then not rejected on the 6.3-0006 assertion; with any node it is checked like any other DAG. (3) For a DAG with several Op nodes the cell-class requirement is met if any Op node has the fact; a Source or Bind node never does. (4) The op_identity is compared to the root by its bare op name; identity-bearing attributes and the operand-role tuple are not checked by these vectors.")
    ));
    s.push_str("    \"accept_vectors\": [\n");
    let accepts = accept_vectors();
    for (i, v) in accepts.iter().enumerate() {
        let comma = if i + 1 < accepts.len() { "," } else { "" };
        s.push_str(&format!(
            "      {{\"name\": {}, \"doc_hex\": {}}}{comma}\n",
            jstr(&v.name),
            jstr(&hex(&v.doc))
        ));
    }
    s.push_str("    ],\n");
    s.push_str("    \"decline_vectors\": [\n");
    let declines = decline_vectors();
    for (i, v) in declines.iter().enumerate() {
        let comma = if i + 1 < declines.len() { "," } else { "" };
        s.push_str(&format!(
            "      {{\"name\": {}, \"doc_hex\": {}, \"section\": {}, \"expect\": {}}}{comma}\n",
            jstr(&v.name),
            jstr(&hex(&v.doc)),
            jstr(v.section),
            jstr(v.expect)
        ));
    }
    s.push_str("    ]\n");
    s.push_str("  },\n");
    s
}
