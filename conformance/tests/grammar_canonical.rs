//! KISS-Conform canonicalization / wire-embedding vectors for the KISS-Grammar
//! region codec (Grammar §6.4, §6.6, §6.8), exercising the existing reference
//! codec in `kiss_conformance::grammar` (encode/decode round-trip, commutative
//! canonicalization, operand-role tuple embedding, extract-record ordering, and
//! the OpAttrs blob-deferral channel).
//!
//! Two tests carry a **real cross-standard payload**: the KISS-Ops OpAttrs blob
//! produced/validated by `kiss_conformance::opattrs`. That module is used purely
//! as a byte payload and an Ops-side validator, referenced **by content** (its
//! functions, types, and constants) and never by clause id — so the Grammar
//! codec's uninterpreted carriage of an Ops-owned sub-vocabulary can be attested
//! without reverse-binding any KISS-Ops clause.
//!
//! Every test asserts a structural/behavioural property through the codec that
//! fails on a plausible drift; none re-implements the codec, greps for a phrase,
//! or self-compares. Each `#[test]` is named exactly per the spec *Test:* tag and
//! cites its single KISS-Grammar clause id in the doc-comment and body.

use kiss_conformance::grammar::*;
use kiss_conformance::opattrs;

// ---------------------------------------------------------------------------
// Shared logical-region builders (author-facing input to the encoder).
// ---------------------------------------------------------------------------

/// Appendix A.1 vector G1: `out = (a*b) + c`, n_inputs = 3, node_count = 5.
fn g1_region() -> Region {
    Region::new(
        3,
        "1",
        "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]), Node::Bind(2)]),
    )
}

/// The default `(data, *)` operand-role entry.
fn data_star() -> RoleEntry {
    RoleEntry::default_entry()
}
/// A non-default `(index, u32)` operand-role entry — the KISS-Ops index-operand
/// example (`gather`'s `u32` index operand).
fn index_u32() -> RoleEntry {
    RoleEntry { role: "index".into(), dtype: "u32".into() }
}

/// A positional `gather` region: root `gather` over `Bind(0)` (data) and
/// `Bind(1)` (index), with the given per-operand roles and OpAttrs blob.
fn gather_region(opattrs: Vec<u8>, roles: Vec<RoleEntry>) -> Region {
    Region::new(
        2,
        "1",
        "1",
        Node::Op {
            op_name: "gather".into(),
            opattrs,
            roles,
            operands: vec![Node::Bind(0), Node::Bind(1)],
        },
    )
}

/// Locate the byte offset of a length-prefixed op_name token on the wire.
fn op_name_at(wire: &[u8], name: &str) -> usize {
    let mut needle = (name.len() as u16).to_le_bytes().to_vec();
    needle.extend_from_slice(name.as_bytes());
    wire
        .windows(needle.len())
        .position(|w| w == needle.as_slice())
        .unwrap_or_else(|| panic!("op_name token {name:?} not found on wire"))
}

fn label(n: &ParsedNode) -> String {
    match n {
        ParsedNode::Bind(i) => format!("bind{i}"),
        ParsedNode::Op { op_name, .. } => op_name.clone(),
    }
}

// ---------------------------------------------------------------------------
// 6.8-0007 — OpAttrs sub-block: u16-LE length + verbatim KISS-Ops bytes.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.8-0007 — the per-node OpAttrs sub-block is embedded as a `u16`
/// little-endian byte-length immediately after `op_name`+`consumers`, followed by
/// that many bytes of the KISS-Ops OpAttrs wire encoding, carried verbatim and
/// uninterpreted; an empty blob encodes as length `0x0000`. Teeth: a `u8` length
/// prefix, any re-encoding/normalization of the blob, or a non-zero empty
/// encoding all fail.
#[test]
fn test_grammar_wire_opattrs_blob_deferral() {
    let _clause = "KISS-GRAMMAR-6.8-0007";
    // A real KISS-Ops OpAttrs `gather` payload, referenced by content (not clause
    // id): [axis, oob_policy, index_operand, index_dtype].
    let blob = opattrs::encode_gather(1, opattrs::OobPolicy::Clamp, 1, opattrs::IndexDtype::U32);
    assert_eq!(blob, vec![1u8, 2, 1, 1], "sanity: the Ops gather blob is 4 bytes");

    let wire = encode(&gather_region(blob.clone(), Vec::new())).unwrap();

    // Root `gather` carries the blob back verbatim after a round-trip.
    let pr = decode(&wire).unwrap();
    match pr.nodes.last().unwrap() {
        ParsedNode::Op { consumers, opattrs, .. } => {
            assert_eq!(*consumers, CONSUMERS_ROOT, "gather is the region root");
            assert_eq!(opattrs, &blob, "{_clause}: OpAttrs blob carried byte-for-byte");
        }
        _ => panic!("root must be the gather Op"),
    }

    // On the wire: op_name, then consumers, then the u16-LE OpAttrs length.
    let at = op_name_at(&wire, "gather");
    let consumers_off = at + 2 + "gather".len();
    assert_eq!(wire[consumers_off], CONSUMERS_ROOT, "consumers precedes the OpAttrs sub-block");
    let len_off = consumers_off + 1;
    let declared = u16::from_le_bytes([wire[len_off], wire[len_off + 1]]) as usize;
    // A u8 length prefix would read as 4 + (blob[0] << 8) = 260, not 4.
    assert_eq!(declared, blob.len(), "{_clause}: OpAttrs length is a u16-LE == blob.len()");
    // The blob bytes follow the length verbatim (no normalization/re-encoding).
    assert_eq!(&wire[len_off + 2..len_off + 2 + blob.len()], blob.as_slice(),
        "OpAttrs payload embedded verbatim");

    // An empty OpAttrs block encodes as length 0x0000 (a 1-operand `relu` root).
    let empty = encode(&Region::new(1, "1", "1", Node::op("relu", vec![Node::Bind(0)]))).unwrap();
    let r_at = op_name_at(&empty, "relu");
    let r_len_off = r_at + 2 + "relu".len() + 1; // after name + consumers
    assert_eq!(&empty[r_len_off..r_len_off + 2], &[0x00, 0x00], "empty OpAttrs == 0x0000");
}

// ---------------------------------------------------------------------------
// 6.2-0006 — the OpAttrs sub-vocabulary is Ops-owned; Grammar carries it uninterpreted.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.2-0006 — the OpAttrs sub-vocabularies surfaced on
/// `pattern_attrs` are owned and validated by KISS-Ops; KISS-Grammar carries the
/// blob uninterpreted and invents no encoding or default-normalization of its
/// own. Teeth: a blob KISS-Ops REJECTS (a reserved-zero oob ordinal) still
/// round-trips byte-exact through the Grammar codec — proving Grammar never
/// validated/normalized the sub-vocabulary. (Distinct from 6.8-0007, which frames
/// a VALID blob; this frames an Ops-invalid one to isolate non-interpretation.)
#[test]
fn test_grammar_opattrs_subvocab_owned_by_ops() {
    let _clause = "KISS-GRAMMAR-6.2-0006";
    // [axis=0, oob=0x00 (the reserved-zero ordinal), index_operand=0, index_dtype=1].
    let blob = vec![0u8, 0, 0, 1];

    // KISS-Ops owns the sub-vocabulary and REJECTS the reserved-zero oob ordinal.
    match opattrs::decode_gather(&blob) {
        Err(opattrs::Decline::ReservedZeroOrdinal { field }) => {
            assert_eq!(field, "oob_policy", "Ops rejects the reserved-zero oob ordinal");
        }
        other => panic!("expected an Ops-side ReservedZeroOrdinal decline, got {other:?}"),
    }

    // Yet the Grammar codec carries the very same blob verbatim: it neither
    // validates nor normalizes the Ops-owned sub-vocabulary.
    let pr = decode(&encode(&gather_region(blob.clone(), Vec::new())).unwrap()).unwrap();
    match pr.nodes.last().unwrap() {
        ParsedNode::Op { opattrs, .. } => {
            assert_eq!(opattrs, &blob,
                "{_clause}: Grammar carries an Ops-invalid blob uninterpreted, unchanged");
        }
        _ => panic!("root must be the gather Op"),
    }
}

// ---------------------------------------------------------------------------
// 6.8-0008 — operand-role tuple: canonical entry count k, in operand order.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.8-0008 — the per-node operand-role tuple is a `u16` entry count
/// `k` followed by `k` entries in operand order, with trailing `(data, *)`
/// defaults omitted; `k` is the §6.1-0008 canonical count (never
/// `operand_count`-padded) and entries are never reordered. `gather` is
/// positional, so operand order after canonicalization equals author order.
/// Teeth: emitting trailing default entries, `k = operand_count`, or reordering
/// role entries all fail.
#[test]
fn test_grammar_wire_operand_role_tuple() {
    let _clause = "KISS-GRAMMAR-6.8-0008";

    // operand0 = (data, *), operand1 = (index, u32) -> k = 2, both entries emitted.
    let wire = encode(&gather_region(Vec::new(), vec![data_star(), index_u32()])).unwrap();
    // Wire k: after op_name + consumers + empty-OpAttrs(00 00) comes the u16 count.
    let at = op_name_at(&wire, "gather");
    let k_off = at + 2 + "gather".len() + 1 + 2; // name + consumers + opattrs len(0)
    assert_eq!(u16::from_le_bytes([wire[k_off], wire[k_off + 1]]), 2, "wire entry count k == 2");

    let pr = decode(&wire).unwrap();
    let (roles, operands) = match pr.nodes.last().unwrap() {
        ParsedNode::Op { roles, operands, .. } => (roles.clone(), operands.clone()),
        _ => panic!("root must be the gather Op"),
    };
    // (b) entries reconstruct in operand order.
    assert_eq!(roles, vec![data_star(), index_u32()], "{_clause}: entries in operand order");
    // (d) k never exceeds operand_count.
    assert!(roles.len() <= operands.len(), "k <= operand_count");

    // (c) only operand0 non-default over a 2-operand node -> k = 1 (trailing
    // default omitted); operand1 reconstructs as the (data, *) default.
    let wire_c = encode(&gather_region(Vec::new(), vec![index_u32(), data_star()])).unwrap();
    let at_c = op_name_at(&wire_c, "gather");
    let k_off_c = at_c + 2 + "gather".len() + 1 + 2;
    assert_eq!(u16::from_le_bytes([wire_c[k_off_c], wire_c[k_off_c + 1]]), 1,
        "trailing default omitted -> k == 1, not operand_count");
    let pr_c = decode(&wire_c).unwrap();
    let roles_c = match pr_c.nodes.last().unwrap() {
        ParsedNode::Op { roles, .. } => roles.clone(),
        _ => panic!("root must be the gather Op"),
    };
    assert_eq!(roles_c, vec![index_u32()], "only the leading non-default entry survives");
    // Every operand with index >= k reconstructs as (data, *) (§6.1-0008 fill rule).
    let op1 = roles_c.get(1).cloned().unwrap_or_else(RoleEntry::default_entry);
    assert_eq!(op1, data_star(), "operand1 (index >= k) reconstructs as (data, *)");
}

// ---------------------------------------------------------------------------
// 6.1-0005 — per-operand (role, dtype) carried independently, no operand-0 inference.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.1-0005 — the operand-role tuple carries, per operand, BOTH a
/// role token and its expected dtype role, independently. Teeth: an impl that
/// inferred every operand's dtype from operand-0's dtype would decode operand1 as
/// `(index, *)` (operand-0 carries `*`); an impl carrying only role or only dtype
/// would drop one token. (Shares the encoder with 6.8-0008 but asserts a DIFFERENT
/// drift: operand-0 dtype inference vs trailing-default/k-count.)
#[test]
fn test_grammar_operand_role_tuple() {
    let _clause = "KISS-GRAMMAR-6.1-0005";
    // operand0 = (data, *) with dtype `*`; operand1 = (index, u32) with dtype `u32`.
    let pr = decode(&encode(&gather_region(Vec::new(), vec![data_star(), index_u32()])).unwrap())
        .unwrap();
    let roles = match pr.nodes.last().unwrap() {
        ParsedNode::Op { roles, .. } => roles.clone(),
        _ => panic!("root must be the gather Op"),
    };
    assert_eq!(roles.len(), 2, "both operands carry an explicit entry");
    // operand0 dtype is the wildcard `*`.
    assert_eq!(roles[0].dtype, "*", "operand0 dtype role");
    // operand1's dtype is `u32`, NOT inferred from operand0's `*`.
    assert_eq!(roles[1].role, "index", "{_clause}: operand1 role token survives");
    assert_eq!(roles[1].dtype, "u32", "{_clause}: operand1 dtype is u32, not inferred from op0");
    assert_ne!(roles[1].dtype, roles[0].dtype, "per-operand dtypes are independent");
}

// ---------------------------------------------------------------------------
// 6.8-0011 — extract records emitted ascending param_slot, ties by lex-least path.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.8-0011 — the `extract_count` records are emitted in canonical
/// order: ascending `param_slot`, ties broken by lexicographically-least path.
/// Teeth: preserving author order or sorting descending. (Distinct from backed
/// 6.8-0006, which pins a single-record byte layout; this attests the multi-record
/// ordering sort.)
#[test]
fn test_grammar_wire_extract_record_order() {
    let _clause = "KISS-GRAMMAR-6.8-0011";
    // Author three extracts OUT OF ORDER by param_slot: 2, 0, 1.
    let mut r = g1_region();
    r.extracts = vec![
        Extract { path: vec![1], param_slot: 2 },
        Extract { path: vec![0], param_slot: 0 },
        Extract { path: vec![2], param_slot: 1 },
    ];
    let pr = decode(&encode(&r).unwrap()).unwrap();
    let slots: Vec<u32> = pr.extracts.iter().map(|e| e.param_slot).collect();
    assert_eq!(slots, vec![0, 1, 2], "{_clause}: records emitted ascending param_slot");
    // The paths ride along with their slots (0->[0], 1->[2], 2->[1]).
    let paths: Vec<Vec<u16>> = pr.extracts.iter().map(|e| e.path.clone()).collect();
    assert_eq!(paths, vec![vec![0u16], vec![2u16], vec![1u16]], "path travels with its slot");

    // Tie case: two records share param_slot 5 with paths [2] and [1] -> lex-least
    // path [1] first.
    let mut t = g1_region();
    t.extracts = vec![
        Extract { path: vec![2], param_slot: 5 },
        Extract { path: vec![1], param_slot: 5 },
    ];
    let pt = decode(&encode(&t).unwrap()).unwrap();
    let tie_paths: Vec<Vec<u16>> = pt.extracts.iter().map(|e| e.path.clone()).collect();
    assert_eq!(tie_paths, vec![vec![1u16], vec![2u16]], "ties break by lex-least path");
}

// ---------------------------------------------------------------------------
// 6.4-0007 — scalar runtime param rides the extract channel, not an op-graph node.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.4-0007 — a scalar runtime parameter rides the root-anchored
/// extract channel and MUST NOT be materialized as an additional op-graph node.
/// Teeth: an impl that lowered the scalar into a Bind/Op node would bump
/// `node_count`; here adding an extract leaves `node_count` invariant while
/// `extract_count` goes 0 -> 1. (Caveat: the shared-target lex-least-path
/// derivation of §6.4-0007 is out of this codec's scope — see grammar.rs — and is
/// not exercised here; this backs the headline no-extra-node obligation.)
#[test]
fn test_grammar_extract_channel_on_root() {
    let _clause = "KISS-GRAMMAR-6.4-0007";
    // G1 with zero extracts: 5 nodes, 0 extracts.
    let base = decode(&encode(&g1_region()).unwrap()).unwrap();
    assert_eq!(base.nodes.len(), 5, "G1 node_count is 5");
    assert_eq!(base.extracts.len(), 0, "G1 has no extracts");

    // Add ONE scalar extract, root-anchored (path is operand indices from the root).
    let mut r = g1_region();
    r.extracts = vec![Extract { path: vec![0], param_slot: 0 }];
    let pr = decode(&encode(&r).unwrap()).unwrap();
    assert_eq!(pr.nodes.len(), 5, "{_clause}: scalar rides extract channel, node_count unchanged");
    assert_eq!(pr.extracts.len(), 1, "the scalar added a tail extract record (0 -> 1)");
    // The extract round-trips root-anchored.
    assert_eq!(pr.extracts[0], Extract { path: vec![0], param_slot: 0 }, "extract root-anchored");
}

// ---------------------------------------------------------------------------
// 6.4-0005 — commutative-operand canonicalization; the set is not hardcoded to add.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.4-0005 — an op KISS-Ops declares commutative has its operands
/// canonicalized to the total canonical order for reproducible emission, and the
/// commutative set is NOT hardcoded to just `add`. Teeth: an impl that only
/// canonicalized `add` (a `mul` operand swap would diverge) or canonicalized
/// nothing fails; here two authorings differing only in `mul`'s operand order
/// converge to identical bytes, and an `add`-level swap also converges.
#[test]
fn test_grammar_commutative_canonicalization() {
    let _clause = "KISS-GRAMMAR-6.4-0005";
    // A and B differ ONLY in MUL's operand order (add operands authored identically),
    // so convergence proves `mul` — not only `add` — canonicalizes its operands.
    let a = Region::new(3, "1", "1",
        Node::op("add", vec![Node::Bind(2), Node::op("mul", vec![Node::Bind(0), Node::Bind(1)])]));
    let b = Region::new(3, "1", "1",
        Node::op("add", vec![Node::Bind(2), Node::op("mul", vec![Node::Bind(1), Node::Bind(0)])]));
    assert_eq!(encode(&a).unwrap(), encode(&b).unwrap(),
        "{_clause}: mul operand swap canonicalizes to identical bytes");

    // An add-level operand swap also converges (add is commutative).
    let c = Region::new(3, "1", "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]), Node::Bind(2)]));
    let d = Region::new(3, "1", "1",
        Node::op("add", vec![Node::Bind(2), Node::op("mul", vec![Node::Bind(0), Node::Bind(1)])]));
    assert_eq!(encode(&c).unwrap(), encode(&d).unwrap(), "add operand swap converges");

    // The convergence is driven by the commutativity oracle read by name: `mul`'s
    // canonical operand order is order-independent, so both operand keys sort the
    // same way regardless of authored order.
    let key_ab = subtree_key(&Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]));
    let key_ba = subtree_key(&Node::op("mul", vec![Node::Bind(1), Node::Bind(0)]));
    assert_eq!(key_ab, key_ba, "mul subtree key is operand-order-independent (commutative)");
}

// ---------------------------------------------------------------------------
// 6.4-0006 — positional op operand order is load-bearing and not reordered.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.4-0006 — an op KISS-Ops declares positional (`select` is
/// `(cond, a, b)`) keeps its operand order; the codec MUST NOT commutatively
/// reorder it. Teeth: an impl that sorted `select`'s operands would collapse
/// `select(cond, a, b)` and `select(cond, b, a)` to identical bytes, destroying
/// the `a`/`b` positions; here they stay distinct and the decoded operand->target
/// order is preserved.
#[test]
fn test_grammar_comparison_select_positional() {
    let _clause = "KISS-GRAMMAR-6.4-0006";
    // X = mul(b1,b2), Y = add(b1,b2): distinct Op subtrees at operand slots 1 and 2.
    let a = Region::new(3, "1", "1", Node::op("select", vec![
        Node::Bind(0),
        Node::op("mul", vec![Node::Bind(1), Node::Bind(2)]),
        Node::op("add", vec![Node::Bind(1), Node::Bind(2)]),
    ]));
    let b = Region::new(3, "1", "1", Node::op("select", vec![
        Node::Bind(0),
        Node::op("add", vec![Node::Bind(1), Node::Bind(2)]),
        Node::op("mul", vec![Node::Bind(1), Node::Bind(2)]),
    ]));
    let (ba, bb) = (encode(&a).unwrap(), encode(&b).unwrap());
    assert_ne!(ba, bb, "{_clause}: positional select operand swap changes the bytes");

    // Decoded root operand -> target order is preserved (cond, a, b vs cond, b, a).
    let resolve = |wire: &[u8]| -> (String, String, String) {
        let pr = decode(wire).unwrap();
        let ops = match pr.nodes.last().unwrap() {
            ParsedNode::Op { operands, .. } => operands.clone(),
            _ => panic!("root must be select"),
        };
        (
            label(&pr.nodes[ops[0] as usize]),
            label(&pr.nodes[ops[1] as usize]),
            label(&pr.nodes[ops[2] as usize]),
        )
    };
    assert_eq!(resolve(&ba), ("bind0".into(), "mul".into(), "add".into()), "A = select(cond, mul, add)");
    assert_eq!(resolve(&bb), ("bind0".into(), "add".into(), "mul".into()), "B = select(cond, add, mul)");
    // select is positional in the KISS-Ops-declared commutativity oracle.
    assert!(!is_commutative("select"), "select is positional (not commutative)");
}

// ---------------------------------------------------------------------------
// 6.4-0002 — a fusion's advertised identity lives on the ROOT (last) node.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.4-0002 — a one-node region denotes a single advertisable op,
/// and a multi-node fusion carries its advertised identity on its ROOT — the last
/// canonical table entry (`consumers = ROOT`) — never on an interior node. Teeth:
/// an impl that read `op_identity` from the FIRST Op node in post-order would take
/// the interior `mul` of G1 instead of the root `add`.
#[test]
fn test_grammar_root_carries_identity() {
    let _clause = "KISS-GRAMMAR-6.4-0002";
    // (a) one-advertisable-op region (a + a): exactly one Op node, and it is the
    // ROOT-marked last entry — the identity carrier.
    let one = decode(&encode(&Region::new(1, "1", "1",
        Node::op("add", vec![Node::Bind(0), Node::Bind(0)]))).unwrap()).unwrap();
    let op_count = one.nodes.iter().filter(|n| matches!(n, ParsedNode::Op { .. })).count();
    assert_eq!(op_count, 1, "one-node region denotes a single advertisable op");
    match one.nodes.last().unwrap() {
        ParsedNode::Op { op_name, consumers, .. } => {
            assert_eq!(*consumers, CONSUMERS_ROOT, "identity carrier is the ROOT (last) entry");
            assert_eq!(op_name, "add", "the single advertised op");
        }
        _ => panic!("last entry must be the Op"),
    }

    // (b) multi-node G1: identity = root `add` (last, ROOT); the FIRST Op in
    // post-order is the interior `mul` (INTERIOR) and is NOT the identity.
    let g1 = decode(&encode(&g1_region()).unwrap()).unwrap();
    let first_op = g1.nodes.iter().find(|n| matches!(n, ParsedNode::Op { .. })).unwrap();
    let (first_name, first_consumers) = match first_op {
        ParsedNode::Op { op_name, consumers, .. } => (op_name.clone(), *consumers),
        _ => unreachable!(),
    };
    let (root_name, root_consumers) = match g1.nodes.last().unwrap() {
        ParsedNode::Op { op_name, consumers, .. } => (op_name.clone(), *consumers),
        _ => panic!("root must be an Op"),
    };
    assert_eq!(first_name, "mul", "first Op in post-order is the interior node");
    assert_eq!(first_consumers, CONSUMERS_INTERIOR, "the interior node is INTERIOR");
    assert_eq!(root_name, "add", "{_clause}: identity is the ROOT op");
    assert_eq!(root_consumers, CONSUMERS_ROOT, "root carries consumers ROOT");
    assert_ne!(root_name, first_name, "identity comes from the root, not the first/interior Op");
}

// ---------------------------------------------------------------------------
// 6.6-0006 — the dtype-role wildcard is spelled exactly '*' (single byte 0x2A).
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.6-0006 — the dtype-role wildcard is the pinned single-byte
/// sentinel `*` (0x2A); an impl MUST NOT substitute an ad-hoc placeholder
/// (`any`, `?`, empty). Teeth: the exported `WILDCARD` constant and the default
/// entry's dtype are `*`, and on the wire operand0's forced-explicit dtype token
/// is exactly `[0x01, 0x00, 0x2A]` (u16 length 1 + `*`) — a multi-char or empty
/// placeholder would change the length prefix and the byte.
#[test]
fn test_grammar_dtype_role_wildcard_token() {
    let _clause = "KISS-GRAMMAR-6.6-0006";
    // The pinned sentinel spelling.
    assert_eq!(WILDCARD, "*", "{_clause}: the dtype-role wildcard is spelled '*'");
    assert_eq!(RoleEntry::default_entry().dtype, "*", "the default entry's dtype role is the wildcard");

    // Force operand0's leading (data, *) to be emitted explicitly by making
    // operand1 non-default, then inspect operand0's dtype token on the wire.
    let wire = encode(&gather_region(Vec::new(), vec![data_star(), index_u32()])).unwrap();
    // operand0's role token is `data`; its dtype token immediately follows.
    let data_at = op_name_at(&wire, "data"); // the length-prefixed role token "data"
    let dtype_off = data_at + 2 + "data".len();
    // A single-byte token: u16 length == 1, then 0x2A.
    assert_eq!(u16::from_le_bytes([wire[dtype_off], wire[dtype_off + 1]]), 1,
        "wildcard dtype token has length 1 (rejects 'any'/'??'/empty)");
    assert_eq!(wire[dtype_off + 2], 0x2A, "{_clause}: the wildcard byte is 0x2A ('*')");
    assert_eq!(&wire[dtype_off..dtype_off + 3], &[0x01, 0x00, 0x2A], "operand0 dtype token = u16-len-1 + '*'");
}
