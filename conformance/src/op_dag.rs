// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Semantics `op_dag` reader and the Identity consistency assertion (KISS-CONTRACT-6.11-0007,
//! 6.4-0002, 6.4-0007, 6.3-0006).
//!
//! [`parse_op_dag`] reads the §6.11-0007 text (`[node, node, ...]`, each node `Op{name; attrs; [edges]}`,
//! `Bind(i)` or `Source{tag; attrs}`) and [`check_op_dag`] enforces the structure: every child strictly
//! earlier than its parent, the root last (so every other node is somebody's child), `Source` nodes spelled
//! exactly, and only KISS-Ops op names. [`check_identity_consistency`] is the §6.3-0006 assertion: the DAG
//! root is the op the `op_identity` names (§6.4-0002), and the cell class of the `accept_predicate` is
//! supported by the structural facts that op, or for a fusion any op of the DAG, declares.
//!
//! The facts come from `conformance/corpus/op_facts.json`, which `tools/kiss_op_facts.py` generates from
//! `spec/ops.md`; nothing is a per-op table written here. `include_str!`, not a runtime read, so the
//! reader and the vectors built from it cannot disagree about which facts file they used.
//!
//! Every decline is a typed value with a stable kebab-case [`wire_tag`](OpDagDecline::wire_tag); nothing
//! here panics on input.

use crate::json::{self, Json};
use std::sync::OnceLock;

const OP_FACTS: &str = include_str!("../corpus/op_facts.json");
const DTYPE_MANIFEST: &str = include_str!("../corpus/dtype_manifest.json");

/// The structural facts an op declares that §6.3-0006 reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpFacts {
    /// The op is a contraction (declares an M/N/K iteration space, §6.13-0009).
    pub is_contraction: bool,
    /// The op carries the index-operand role (§6.11-0012).
    pub index_operand_role: bool,
    /// The op declares a reduce axis, as an OpAttrs field or via its inner carrier (§6.19-0036).
    pub declares_reduce_axis: bool,
}

fn flag(entry: &Json, key: &str) -> bool {
    matches!(entry.get(key), Some(Json::Bool(true)))
}

fn facts_table() -> &'static Vec<(String, OpFacts)> {
    static TABLE: OnceLock<Vec<(String, OpFacts)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let doc = json::parse(OP_FACTS).expect("op_facts.json is valid JSON");
        let Some(Json::Obj(ops)) = doc.get("ops") else {
            panic!("op_facts.json has an `ops` object")
        };
        ops.iter()
            .map(|(name, e)| {
                let facts = OpFacts {
                    is_contraction: flag(e, "is_contraction"),
                    index_operand_role: flag(e, "index_operand_role"),
                    declares_reduce_axis: flag(e, "declares_reduce_axis"),
                };
                (name.clone(), facts)
            })
            .collect()
    })
}

/// The facts of the KISS-Ops op `name`, or `None` when it is not a KISS-Ops op name.
pub fn op_facts(name: &str) -> Option<OpFacts> {
    facts_table()
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, f)| *f)
}

/// Storage width in bits of a dtype token, from the dtype manifest (the §6.1 SSOT).
fn dtype_bits(token: &str) -> Option<u64> {
    let doc = json::parse(DTYPE_MANIFEST).ok()?;
    let list = doc.get("dtypes")?.as_arr()?;
    let entry = list
        .iter()
        .find(|d| d.get("token").and_then(Json::as_str) == Some(token))?;
    entry.get("storage_bits")?.as_u64()
}

/// One node of the `op_dag` array (§6.11-0007).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DagNode {
    /// `Op{<op_name>; <op_attrs>; [<child_edges>]}`.
    Op {
        name: String,
        attrs: String,
        edges: Vec<usize>,
    },
    /// `Bind(<bind_index>)`.
    Bind(u32),
    /// `Source{<tag>; <source_attrs>}`.
    Source { tag: String, attrs: String },
}

/// A typed decline of the `op_dag` value. Never a panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpDagDecline {
    /// Not a §6.11-0007 array of nodes, or a node is not spelled `Op{..}` / `Bind(..)` / `Source{..}`.
    Malformed,
    /// A `Source` node outside the §6.11-0007 tags and attribute spellings.
    BadSource,
    /// A `child_edges` entry that is not a strictly earlier index (a later or equal index, out of range).
    ChildNotEarlier,
    /// A node other than the last that no later node uses: a second root, so the root is not last.
    MultipleRoots,
    /// An `Op` whose name is not a bare KISS-Ops op name (§6.4-0002).
    UnknownOp,
    /// A `machine-checkable-IR` Semantics with no node: there is no root to equal the `op_identity`.
    Empty,
}

impl OpDagDecline {
    /// The stable kebab-case category tag (payload omitted).
    pub fn wire_tag(self) -> &'static str {
        match self {
            OpDagDecline::Malformed => "op-dag-malformed",
            OpDagDecline::BadSource => "op-dag-bad-source",
            OpDagDecline::ChildNotEarlier => "op-dag-child-not-earlier",
            OpDagDecline::MultipleRoots => "op-dag-multiple-roots",
            OpDagDecline::UnknownOp => "op-dag-unknown-op",
            OpDagDecline::Empty => "op-dag-empty",
        }
    }
}

/// Split `s` at every `", "` that sits outside `{}`, `[]` and `()`.
fn split_top(s: &str) -> Vec<&str> {
    let (mut depth, mut start, mut out) = (0i32, 0usize, Vec::new());
    let b = s.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 && b.get(i + 1) == Some(&b' ') => {
                out.push(&s[start..i]);
                start = i + 2;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// A canonical decimal: ASCII digits only, no sign, no leading zero (except `0` itself).
fn canonical_uint(s: &str) -> Option<u64> {
    let ok =
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s == "0" || !s.starts_with('0'));
    if ok {
        s.parse().ok()
    } else {
        None
    }
}

fn parse_edges(list: &str) -> Result<Vec<usize>, OpDagDecline> {
    if list.is_empty() {
        return Ok(Vec::new());
    }
    split_top(list)
        .into_iter()
        .map(|e| {
            canonical_uint(e)
                .map(|v| v as usize)
                .ok_or(OpDagDecline::Malformed)
        })
        .collect()
}

fn parse_op_node(body: &str) -> Result<DagNode, OpDagDecline> {
    let (name, rest) = body.split_once("; ").ok_or(OpDagDecline::Malformed)?;
    let at = rest.rfind("; [").ok_or(OpDagDecline::Malformed)?;
    let edges = rest[at + 3..]
        .strip_suffix(']')
        .ok_or(OpDagDecline::Malformed)?;
    Ok(DagNode::Op {
        name: name.to_string(),
        attrs: rest[..at].to_string(),
        edges: parse_edges(edges)?,
    })
}

/// `key=<canonical decimal>` with the value inside `0..=max`.
fn keyed_uint(attrs: &str, key: &str, max: u64) -> bool {
    attrs
        .strip_prefix(key)
        .and_then(|v| v.strip_prefix('='))
        .and_then(canonical_uint)
        .is_some_and(|v| v <= max)
}

/// `dtype=<token>, bits=0x<HEX>`: upper-case hex, zero-padded to `2 * ceil(width_bits / 8)` digits.
fn const_source_ok(attrs: &str) -> bool {
    let Some((dtype, hex)) = attrs.split_once(", bits=0x") else {
        return false;
    };
    let Some(bits) = dtype.strip_prefix("dtype=").and_then(dtype_bits) else {
        return false;
    };
    let digits = 2 * bits.div_ceil(8) as usize;
    hex.len() == digits
        && hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
}

fn source_attrs_ok(tag: &str, attrs: &str) -> bool {
    match tag {
        "const" => const_source_ok(attrs),
        "param" => keyed_uint(attrs, "index", u64::from(u32::MAX)),
        "coord" | "extent" => keyed_uint(attrs, "axis", u64::from(u8::MAX)),
        "reduced" => keyed_uint(attrs, "stage", u64::from(u32::MAX)),
        "reduced_count" => attrs.is_empty(),
        _ => false,
    }
}

fn parse_node(s: &str) -> Result<DagNode, OpDagDecline> {
    if let Some(body) = s.strip_prefix("Op{") {
        return parse_op_node(body.strip_suffix('}').ok_or(OpDagDecline::Malformed)?);
    }
    if let Some(body) = s.strip_prefix("Source{") {
        let body = body.strip_suffix('}').ok_or(OpDagDecline::Malformed)?;
        let (tag, attrs) = body.split_once("; ").ok_or(OpDagDecline::BadSource)?;
        return if source_attrs_ok(tag, attrs) {
            Ok(DagNode::Source {
                tag: tag.to_string(),
                attrs: attrs.to_string(),
            })
        } else {
            Err(OpDagDecline::BadSource)
        };
    }
    let index = s
        .strip_prefix("Bind(")
        .and_then(|b| b.strip_suffix(')'))
        .and_then(canonical_uint);
    match index.and_then(|v| u32::try_from(v).ok()) {
        Some(i) => Ok(DagNode::Bind(i)),
        None => Err(OpDagDecline::Malformed),
    }
}

/// Read the §6.11-0007 text into nodes. Syntax only: ordering and names are [`check_op_dag`].
pub fn parse_op_dag(text: &str) -> Result<Vec<DagNode>, OpDagDecline> {
    let inner = text
        .strip_prefix('[')
        .and_then(|t| t.strip_suffix(']'))
        .ok_or(OpDagDecline::Malformed)?;
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    split_top(inner).into_iter().map(parse_node).collect()
}

/// The structural rules of §6.11-0007 and §6.4-0002 over parsed nodes: a `machine-checkable-IR` DAG is
/// non-empty; every `child_edges` entry is a strictly earlier index; every node but the last is some later
/// node's child (the root is last); every `Op` name is a KISS-Ops op name.
pub fn check_op_dag(nodes: &[DagNode]) -> Result<(), OpDagDecline> {
    if nodes.is_empty() {
        return Err(OpDagDecline::Empty);
    }
    let mut used = vec![false; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        let DagNode::Op { name, edges, .. } = node else {
            continue;
        };
        if op_facts(name).is_none() {
            return Err(OpDagDecline::UnknownOp);
        }
        for &e in edges {
            if e >= i {
                return Err(OpDagDecline::ChildNotEarlier);
            }
            used[e] = true;
        }
    }
    let last = nodes.len() - 1;
    if used[..last].iter().any(|u| !u) {
        return Err(OpDagDecline::MultipleRoots);
    }
    Ok(())
}

/// A typed decline of the §6.3-0006 / §6.4-0002 cross-field assertion. Never a panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityDecline {
    /// The DAG root is not the op the `op_identity` names (§6.4-0002).
    RootMismatch,
    /// The `accept_predicate` cell class needs a fact no op of the DAG declares (§6.3-0006).
    ClassUnsupported,
}

impl IdentityDecline {
    /// The stable kebab-case category tag (payload omitted).
    pub fn wire_tag(self) -> &'static str {
        match self {
            IdentityDecline::RootMismatch => "op-identity-root-mismatch",
            IdentityDecline::ClassUnsupported => "cell-class-unsupported-by-op",
        }
    }
}

/// The bare op name of an `op_identity` in the §6.11-0009 spelling `<op_name>[{attrs}][[roles]]`.
pub fn identity_op_name(op_identity: &str) -> &str {
    op_identity.split(['{', '[']).next().unwrap_or("")
}

/// Whether `facts` supports the KISS-Classify cell `class` (§6.3-0006): `ctr` needs a contraction, `ndx` the
/// index-operand role, `rdc` a declared reduce axis; every other class is decided by shape and asks nothing.
fn supports(class: &str, facts: OpFacts) -> bool {
    match class {
        "ctr" => facts.is_contraction,
        "ndx" => facts.index_operand_role,
        "rdc" => facts.declares_reduce_axis,
        _ => true,
    }
}

/// §6.3-0006 over a non-empty DAG that [`check_op_dag`] accepted: the root op is the `op_identity`'s op, and
/// the cell class of the `accept_predicate` (field 2 of the `structure_key`) is supported by the root op or,
/// for a fusion, by any op of the DAG. An empty DAG (`declared-op-tag`, no node for the region) is not
/// rejected on this assertion and is not passed here.
pub fn check_identity_consistency(
    nodes: &[DagNode],
    op_identity: &str,
    accept_predicate: &str,
) -> Result<(), IdentityDecline> {
    let root = match nodes.last() {
        Some(DagNode::Op { name, .. }) => name.as_str(),
        _ => return Err(IdentityDecline::RootMismatch),
    };
    if root != identity_op_name(op_identity) {
        return Err(IdentityDecline::RootMismatch);
    }
    let class = accept_predicate.split('|').nth(1).unwrap_or("");
    let any = nodes.iter().any(|n| matches!(n, DagNode::Op { name, .. } if op_facts(name).is_some_and(|f| supports(class, f))));
    if any {
        Ok(())
    } else {
        Err(IdentityDecline::ClassUnsupported)
    }
}
