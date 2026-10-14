// SPDX-License-Identifier: MIT OR Apache-2.0
//! The structure-directed IR-DAG generator and its validity self-check (Conform §6.6).
//!
//! This is the IN half of the §6.6 split ruled 2026-10-14 (board 203, item 3): a seeded,
//! reproducible generator of KISS-Ops op DAGs, the self-check that tells a valid DAG from a
//! malformed one, and a malformed/near-boundary generator that feeds typed declines. The
//! cross-backend / emitter / lift rows (§6.6-0002/-0003/-0004/-0006/-0007) are OUT: they
//! need a suite-registered emitter and a lifter this crate does not have.
//!
//! **What is modelled, stated so it is not over-credited.** The model is the scalar-expression
//! DAG over ops whose arity the KISS-Ops §6.13 decomposition table itself fixes
//! ([`OpSigs::from_decompositions`]). Operand roles, ranks, dtypes and OpAttrs
//! sub-vocabularies are NOT generated; §6.6-0001 names them, so it is recorded PARTIAL. The
//! generator is deterministic: same seed, same DAG (§7.2-0002's reproducibility concern).

use crate::differential::SplitMix64;
use crate::structural_dag::{canonical_form, Dag, Decompositions, Node};
use std::collections::{BTreeMap, BTreeSet};

const LEAVES: &[&str] = &["input", "const", "param", "coord", "reduced", "extent", "sym"];

/// Op arities the §6.13 table fixes (an op is in only if every use agrees on one arity and
/// none of its uses takes a bare symbol, which marks a structural/attribute-bearing op).
#[derive(Debug, Clone, Default)]
pub struct OpSigs {
    pub arity: BTreeMap<String, usize>,
}

impl OpSigs {
    pub fn from_decompositions(d: &Decompositions) -> OpSigs {
        let mut seen: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        let mut tainted: BTreeSet<String> = BTreeSet::new();
        for (op, (arity, tpl)) in &d.templates {
            seen.entry(op.clone()).or_default().insert(*arity);
            for n in &tpl.nodes {
                if LEAVES.contains(&n.op.as_str()) {
                    continue;
                }
                seen.entry(n.op.clone()).or_default().insert(n.args.len());
                if n.args.iter().any(|&a| tpl.nodes[a].op == "sym") {
                    tainted.insert(n.op.clone());
                }
            }
        }
        // An op that resolves THROUGH an unreadable decomposition (log_softmax -> logsumexp) is not
        // resolvable to the floor by this harness, so the generator must not emit it.
        fn reaches_unreadable(op: &str, d: &Decompositions, depth: usize) -> bool {
            if d.unreadable.contains(op) {
                return true;
            }
            if depth > 64 {
                return true;
            }
            d.templates.get(op).is_some_and(|(_, tpl)| {
                tpl.nodes.iter().any(|n| {
                    !LEAVES.contains(&n.op.as_str()) && reaches_unreadable(&n.op, d, depth + 1)
                })
            })
        }
        let arity = seen
            .into_iter()
            .filter(|(op, a)| a.len() == 1 && !tainted.contains(op) && !reaches_unreadable(op, d, 0))
            .map(|(op, a)| (op, *a.iter().next().unwrap()))
            .filter(|(_, a)| *a >= 1)
            .collect();
        OpSigs { arity }
    }
}

/// Why a DAG is not valid (typed; never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    BadRoot,
    /// An argument index is not strictly earlier: a cycle or forward reference (acyclicity).
    ForwardEdge { node: usize, arg: usize },
    UnknownOp(String),
    Arity { op: String, expected: usize, got: usize },
    InputOutOfRange { index: usize, n_inputs: usize },
    /// Resolution to the primitive floor did not complete (level structure broken).
    Unresolvable(String),
}

fn input_index(n: &Node) -> Option<usize> {
    String::from_utf8_lossy(&n.attrs).parse().ok()
}

/// The validity self-check: a DAG is valid iff it is children-first and acyclic, every op is
/// known with its fixed arity, every input index is bound within `n_inputs`, and it resolves
/// to the primitive floor.
pub fn check_valid(dag: &Dag, sigs: &OpSigs, table: &Decompositions, n_inputs: usize) -> Result<(), Violation> {
    if dag.root >= dag.nodes.len() {
        return Err(Violation::BadRoot);
    }
    for (i, n) in dag.nodes.iter().enumerate() {
        for &a in &n.args {
            if a >= i {
                return Err(Violation::ForwardEdge { node: i, arg: a });
            }
        }
        match n.op.as_str() {
            "input" => match input_index(n) {
                Some(k) if k < n_inputs => {}
                Some(k) => return Err(Violation::InputOutOfRange { index: k, n_inputs }),
                None => return Err(Violation::UnknownOp("input".into())),
            },
            "const" => {}
            op => match sigs.arity.get(op) {
                None => return Err(Violation::UnknownOp(op.to_string())),
                Some(&e) if e != n.args.len() => {
                    return Err(Violation::Arity { op: op.to_string(), expected: e, got: n.args.len() })
                }
                Some(_) => {}
            },
        }
    }
    canonical_form(dag, table).map(|_| ()).map_err(|e| Violation::Unresolvable(format!("{e:?}")))
}

fn pick<'a, T>(rng: &mut SplitMix64, xs: &'a [T]) -> &'a T {
    &xs[(rng.next_u64() % xs.len() as u64) as usize]
}

fn input_node(k: usize) -> Node {
    Node { op: "input".into(), attrs: k.to_string().into_bytes(), args: vec![] }
}

/// A random valid DAG: `n_inputs` input leaves, then `n_ops` op nodes whose operands are
/// earlier nodes; the last node is the root, and every input is used (bound).
pub fn gen_valid(rng: &mut SplitMix64, sigs: &OpSigs, n_inputs: usize, n_ops: usize) -> Dag {
    assert!(n_inputs >= 1 && n_ops >= 1 && !sigs.arity.is_empty());
    let ops: Vec<(&String, &usize)> = sigs.arity.iter().collect();
    let mut nodes: Vec<Node> = (0..n_inputs).map(input_node).collect();
    for _ in 0..n_ops {
        let (op, &arity) = *pick(rng, &ops);
        let args = (0..arity).map(|_| (rng.next_u64() % nodes.len() as u64) as usize).collect();
        nodes.push(Node { op: op.clone(), attrs: vec![], args });
    }
    // Bind every input in the ROOT's reachable set: an input only a dead node uses would be
    // unbound once the tree is read from the root (the codec's expressibility rule, §6.4-0008).
    let root_op = ops.iter().find(|(_, a)| **a == 2).map(|(o, _)| (*o).clone());
    let mut root = nodes.len() - 1;
    if let Some(op2) = root_op {
        let mut reach = BTreeSet::new();
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            if reach.insert(i) {
                stack.extend(nodes[i].args.iter().copied());
            }
        }
        for k in 0..n_inputs {
            if !reach.contains(&k) {
                nodes.push(Node { op: op2.clone(), attrs: vec![], args: vec![root, k] });
                root = nodes.len() - 1;
            }
        }
    }
    Dag { nodes, root }
}

/// The malformation classes of the negative path (§6.6-0005), at and just past each boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Malformation {
    ForwardEdge,
    SelfLoop,
    UnknownOp,
    ArityPlusOne,
    ArityMinusOne,
    InputIndexAtBound,
    RootPastEnd,
}

pub const ALL_MALFORMATIONS: [Malformation; 7] = [
    Malformation::ForwardEdge,
    Malformation::SelfLoop,
    Malformation::UnknownOp,
    Malformation::ArityPlusOne,
    Malformation::ArityMinusOne,
    Malformation::InputIndexAtBound,
    Malformation::RootPastEnd,
];

/// Corrupt a valid DAG in exactly one way. Returns `None` when the class does not apply
/// (e.g. no op node has arity >= 2 for `ArityMinusOne` to stay non-zero).
pub fn corrupt(valid: &Dag, kind: Malformation, n_inputs: usize) -> Option<Dag> {
    let mut d = valid.clone();
    let first_op = d.nodes.iter().position(|n| !LEAVES.contains(&n.op.as_str()))?;
    match kind {
        Malformation::ForwardEdge => {
            let last = d.nodes.len() - 1;
            d.nodes[first_op].args[0] = last.max(first_op + 1);
        }
        Malformation::SelfLoop => d.nodes[first_op].args[0] = first_op,
        Malformation::UnknownOp => d.nodes[first_op].op = "not_a_kiss_op".into(),
        Malformation::ArityPlusOne => d.nodes[first_op].args.push(0),
        Malformation::ArityMinusOne => {
            d.nodes[first_op].args.pop()?;
        }
        Malformation::InputIndexAtBound => {
            let i = d.nodes.iter().position(|n| n.op == "input")?;
            d.nodes[i].attrs = n_inputs.to_string().into_bytes();
        }
        Malformation::RootPastEnd => d.root = d.nodes.len(),
    }
    Some(d)
}

/// Expand a DAG into the Grammar codec's tree form (inputs become `Bind`, a `const`'s literal
/// rides as its OpAttrs), so generated DAGs also exercise the real region encoder's typed
/// declines (`n_inputs` is declared, §6.4-0008).
pub fn to_region(dag: &Dag, n_inputs: u32) -> crate::grammar::Region {
    fn go(dag: &Dag, i: usize) -> crate::grammar::Node {
        let n = &dag.nodes[i];
        if n.op == "input" {
            return crate::grammar::Node::Bind(input_index(n).unwrap_or(0) as u32);
        }
        crate::grammar::Node::Op {
            op_name: n.op.clone(),
            opattrs: n.attrs.clone(),
            roles: Vec::new(),
            operands: n.args.iter().map(|&a| go(dag, a)).collect(),
        }
    }
    crate::grammar::Region::new(n_inputs, "ops-test", "classify-test", go(dag, dag.root))
}
