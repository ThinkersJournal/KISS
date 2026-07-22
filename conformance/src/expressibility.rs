//! Reference encoder for the KISS-Conform **Appendix-F expressible-signature-set**
//! byte format. Backs **KISS-CONFORM-6.10-0006**
//! (`test_conform_expressible_signature_set_schema`,
//! `conformance/tests/expressibility_golden.rs`).
//!
//! A signature's `bytes` is the **KISS-Grammar §6.4-0010 canonical subtree
//! serialization** of its root, computed as a **structure-only projection**
//! (Appendix F "Signature bytes"): every operand's dtype-role collapses to the
//! wildcarded default `(data, *)`, so the operand-role tuple (§6.8-0008) is
//! always empty (`k = 0`) for every op exercised by this reference slice (the
//! primitive-floor `add`/`sub` goldens have no non-`data`-role operand — only
//! an op like `select`'s `cond` would emit a role-tuple entry). Recursively,
//! per §6.4-0010 / §6.8:
//!   * a `Bind(i)` leaf -> the byte `0x00`, then `i` as `u32` little-endian
//!     (no `consumers` byte — §6.8-0005: "A Bind leaf carries no consumers
//!     field");
//!   * an `Op` node -> the byte `0x01`, then: (1) `op_name` as a `u16`-LE
//!     byte-length-prefixed UTF-8 string (§6.8-0003); (2) `consumers`, a `u8`
//!     — `0x01` (ROOT) iff this node is the region root (the LAST node in
//!     canonical node order), else `0x00` (INTERIOR) (§6.8-0005); (3) the
//!     OpAttrs blob as a `u16`-LE byte-length-prefixed byte string
//!     (§6.8-0007); (4) the operand-role tuple as a `u16`-LE entry count
//!     (§6.8-0008, always `0` in this slice); (5) the operand subtrees,
//!     concatenated, in **canonical operand order** (§6.4-0010): for a
//!     KISS-Ops-declared **commutative** op, ascending unsigned-byte-lex order
//!     of each operand's own canonical subtree serialization; for a
//!     **positional** op, the authored order unchanged.
//!
//! Zero-dependency (stdlib only), mirroring `contract.rs`'s from-scratch
//! CRC-32: `signature_hash` uses a from-scratch FNV-1a (64-bit), the
//! algorithm Appendix F pins directly for it — see [`fnv1a64`] for the
//! resolved-spec-gap history and the KISS issue #67 cross-ecosystem tracking
//! note.

use crate::json::Json;

// ---------------------------------------------------------------------------
// The signature op-DAG, as authored (KISS-Grammar canonical node order: every
// child strictly earlier than its parent; root = the last node).
// ---------------------------------------------------------------------------

/// One node of a signature's op-DAG. `Bind(i)` is an input leaf bound to the
/// region's `i`-th external input; `Op` is an interior-or-root primitive-floor
/// node. `opattrs` is that node's raw §6.19 OpAttrs wire bytes (empty for the
/// empty-attrs form).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Op { name: String, opattrs: Vec<u8> },
    Bind(u32),
}

/// A signature: a topologically-ordered node list with the **root last**
/// (Grammar canonical node order), and `edges[i]` = the operand node-indices
/// of node `i`, in **authored** (pre-canonicalization) order (`edges[i]` is
/// empty for a `Bind` leaf, which has no operands).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub nodes: Vec<Node>,
    pub edges: Vec<Vec<u32>>,
}

/// A whole enumerated expressible-signature set (Appendix F top-level fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureSet {
    pub owner: String,
    pub ops_op_set_version: String,
    pub opattrs_wire_version: String,
    pub generator: String,
    pub signatures: Vec<Signature>,
}

/// KISS-Ops-declared **commutative** ops at the primitive floor. A real
/// implementation MUST read commutativity from KISS-Ops by name
/// (KISS-GRAMMAR-6.4-0005 forbids a Grammar-owned hardcoded list); this
/// reference slice only exercises the primitive-floor `add`/`sub` goldens, so
/// the small set is hardcoded here per the task brief.
// TODO: read KISS-Ops commutativity (§6.2-0005) instead of hardcoding.
const COMMUTATIVE_OPS: [&str; 2] = ["add", "mul"];

fn is_commutative(op_name: &str) -> bool {
    COMMUTATIVE_OPS.contains(&op_name)
}

/// The operand-index/subtree-bytes pairs of node `idx`, in **canonical**
/// operand order (§6.4-0010): for a commutative op, sorted ascending by
/// unsigned-byte-lex of each operand's recursively-canonical subtree
/// serialization; for a positional op (or a `Bind` leaf, which has none), the
/// authored `edges[idx]` order unchanged.
fn canonical_operands(nodes: &[Node], edges: &[Vec<u32>], idx: usize) -> Vec<(u32, Vec<u8>)> {
    let mut pairs: Vec<(u32, Vec<u8>)> = edges[idx]
        .iter()
        .map(|&oi| (oi, node_bytes(nodes, edges, oi as usize)))
        .collect();
    if let Node::Op { name, .. } = &nodes[idx] {
        if is_commutative(name) {
            // ascending unsigned-byte-lex: Vec<u8>'s derived Ord is exactly
            // this (u8 has no sign; slice comparison is lexicographic).
            pairs.sort_by(|a, b| a.1.cmp(&b.1));
        }
    }
    pairs
}

/// The §6.4-0010 canonical subtree serialization of node `idx` of `nodes`
/// (root = `nodes.len() - 1`).
fn node_bytes(nodes: &[Node], edges: &[Vec<u32>], idx: usize) -> Vec<u8> {
    let root_idx = nodes.len() - 1;
    match &nodes[idx] {
        // Bind leaf: 0x00, then input_index as u32 LE. No consumers byte
        // (§6.8-0005).
        Node::Bind(i) => {
            let mut b = Vec::with_capacity(5);
            b.push(0x00);
            b.extend_from_slice(&i.to_le_bytes());
            b
        }
        Node::Op { name, opattrs } => {
            let mut b = Vec::new();
            b.push(0x01);
            // 1. op_name: u16 LE byte-length, then UTF-8 bytes (§6.8-0003). A
            //    silent `as u16` truncation for a length >= 65536 would emit a
            //    WRONG length prefix while still appending all the bytes --
            //    corrupt-but-non-panicking output, the worst failure mode for
            //    a byte-format reference. Loudly refuse instead.
            let name_bytes = name.as_bytes();
            let name_len = u16::try_from(name_bytes.len())
                .expect("op_name length exceeds u16 -- not a valid region");
            b.extend_from_slice(&name_len.to_le_bytes());
            b.extend_from_slice(name_bytes);
            // 2. consumers: u8 -- ROOT iff this is the region root (§6.8-0005).
            b.push(if idx == root_idx { 0x01 } else { 0x00 });
            // 3. OpAttrs blob: u16 LE byte-length, then that many bytes
            //    (§6.8-0007). Same silent-truncation guard as op_name above.
            let opattrs_len = u16::try_from(opattrs.len())
                .expect("OpAttrs blob length exceeds u16 -- not a valid region");
            b.extend_from_slice(&opattrs_len.to_le_bytes());
            b.extend_from_slice(opattrs);
            // 4. operand-role tuple: u16 LE entry count (§6.8-0008). Always 0
            //    in this reference slice -- see the module doc comment.
            b.extend_from_slice(&0u16.to_le_bytes());
            // 5. operand subtrees, concatenated, in canonical operand order.
            for (_, ob) in canonical_operands(nodes, edges, idx) {
                b.extend_from_slice(&ob);
            }
            b
        }
    }
}

/// Validate the structural precondition the encoder relies on (§6.8-0004: an
/// operand MUST reference a strictly-earlier node than its parent, KISS-Grammar
/// canonical node order): `edges` has exactly one entry per node, and every
/// operand index in `edges[i]` is `< i`. Without this check, a malformed
/// `Signature` (an out-of-range operand index, an `edges`/`nodes` length
/// mismatch, or a self/cyclic edge) would cause an out-of-bounds index panic
/// or unbounded recursion deeper inside the encoder -- UB-adjacent failure
/// modes for a byte-format reference. This turns any of those into one
/// immediate, clearly-worded panic at the public entry point instead.
fn validate_signature_shape(sig: &Signature) {
    assert_eq!(
        sig.edges.len(),
        sig.nodes.len(),
        "Signature is malformed: edges.len() ({}) != nodes.len() ({}) -- exactly one operand-index list is required per node",
        sig.edges.len(),
        sig.nodes.len()
    );
    for (i, ops) in sig.edges.iter().enumerate() {
        for &oi in ops {
            assert!(
                (oi as usize) < i,
                "Signature is malformed: node {i}'s operand index {oi} does not reference a \
                 strictly-earlier node (§6.8-0004) -- an out-of-range, self-referential, or \
                 cyclic edge is not a valid region"
            );
        }
    }
}

/// The §6.4-0010 canonical subtree serialization of `sig`'s root (the last
/// node) -- the membership key of the expressibility oracle (Appendix F).
///
/// **Precondition** (validated up front via [`validate_signature_shape`],
/// which panics loudly on violation rather than risk out-of-bounds/UB deeper
/// in the recursion): `sig.edges.len() == sig.nodes.len()`, and every operand
/// index in `sig.edges[i]` MUST be strictly less than `i` (§6.8-0004).
pub fn serialize_signature_bytes(sig: &Signature) -> Vec<u8> {
    validate_signature_shape(sig);
    if sig.nodes.is_empty() {
        return Vec::new();
    }
    node_bytes(&sig.nodes, &sig.edges, sig.nodes.len() - 1)
}

// ---------------------------------------------------------------------------
// signature_hash -- the FNV-1a-64 membership INDEX over `bytes` (Appendix F);
// `bytes` itself remains the decidable membership key (§6.9).
// ---------------------------------------------------------------------------

/// FNV-1a (64-bit), from scratch (zero-dependency): offset basis
/// `0xcbf29ce484222325`, prime `0x100000001b3`, one XOR-then-multiply step per
/// byte.
///
/// **RESOLVED spec gap** (was flagged as an open ASSUMPTION during initial
/// development; resolved in-branch by commit `6481633`). Appendix F originally
/// cited a bare "§6.4-0005" for `signature_hash`, but no clause anywhere in
/// the KISS spec suite named `§6.4-0005` ever defined a hash *algorithm* (it
/// was a copy/collision artifact — every sub-standard's own `§6.4-0005` is an
/// unrelated clause, e.g. KISS-GRAMMAR-6.4-0005 is commutative-operand
/// canonicalization). Appendix F now pins FNV-1a-64 **directly**, self-contained,
/// with no cross-reference: this function is that pinned algorithm, not a
/// standing guess. `signature_hash` is a fixed 64-bit **membership index**
/// only — the *decidable* membership key is `bytes` itself, compared for
/// byte-identity under the §6.9 structural-equality comparator (Appendix F,
/// `signature_hash` bullet); where the two disagree, `bytes` governs.
/// Cross-ecosystem reconciliation of the hash *algorithm* choice against
/// Fuel's `base_map_hash` is tracked in KISS issue #67 — if `base_map_hash`
/// turns out to differ from FNV-1a-64, the two Appendix-F goldens re-mint
/// (bytes-wise they are unaffected; the algorithm choice does not change
/// `bytes`, only `signature_hash`).
#[must_use]
pub fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h: u64 = OFFSET_BASIS;
    for &byte in data {
        h ^= u64::from(byte);
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// `signature_hash`: the FNV-1a-64 hash of `bytes`, rendered as 16 lowercase
/// hex digits (big-endian textual rendering of the `u64`, mirroring
/// `contract.rs`'s `Real::render` convention). See [`fnv1a64`] for the
/// algorithm-choice ASSUMPTION.
#[must_use]
pub fn signature_hash_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ---------------------------------------------------------------------------
// The set (JSON) serialization -- Appendix F "Encoding".
// ---------------------------------------------------------------------------

/// Minimal JSON string-literal escaper (quote + surrounding `"`s): `"`, `\`,
/// and the control characters. None of this reference slice's actual string
/// values (op names, version stamps, `KISS-OPS`) need it, but a from-scratch
/// encoder must not silently emit invalid JSON if a caller ever does supply
/// one that does.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The Appendix F node display string: `Bind{index}` or `Op{name;opattrs_hex}`
/// (empty `opattrs_hex` for the empty-attrs form).
fn node_display(n: &Node) -> String {
    match n {
        Node::Bind(i) => format!("Bind{{{i}}}"),
        Node::Op { name, opattrs } => format!("Op{{{name};{}}}", hex_lower(opattrs)),
    }
}

/// Serialize one [`Signature`] to its Appendix F JSON object, returning both
/// its `bytes` (needed by the caller to order `signatures` ascending) and the
/// object's rendered text.
fn signature_json(sig: &Signature) -> (Vec<u8>, String) {
    let bytes = serialize_signature_bytes(sig);
    let hash = signature_hash_hex(&bytes);
    let nodes_json: Vec<String> = sig.nodes.iter().map(|n| json_string(&node_display(n))).collect();
    let edges_json: Vec<String> = (0..sig.nodes.len())
        .map(|i| {
            let idxs: Vec<String> = canonical_operands(&sig.nodes, &sig.edges, i)
                .into_iter()
                .map(|(oi, _)| oi.to_string())
                .collect();
            format!("[{}]", idxs.join(","))
        })
        .collect();
    let obj = format!(
        "{{\"bytes\":{},\"edges\":[{}],\"nodes\":[{}],\"signature_hash\":{}}}",
        json_string(&hex_lower(&bytes)),
        edges_json.join(","),
        nodes_json.join(","),
        json_string(&hash),
    );
    (bytes, obj)
}

/// The Appendix-F JSON document for `set`: object keys in ascending
/// code-point order, no insignificant whitespace, `signatures` ordered
/// ascending by each signature's `bytes` (unsigned-byte-lex).
pub fn serialize_set(set: &SignatureSet) -> Vec<u8> {
    let mut sigs: Vec<(Vec<u8>, String)> = set.signatures.iter().map(signature_json).collect();
    sigs.sort_by(|a, b| a.0.cmp(&b.0));
    let sig_strs: Vec<String> = sigs.into_iter().map(|(_, s)| s).collect();
    let doc = format!(
        "{{\"generator\":{},\"opattrs_wire_version\":{},\"ops_op_set_version\":{},\"owner\":{},\"signatures\":[{}]}}",
        json_string(&set.generator),
        json_string(&set.opattrs_wire_version),
        json_string(&set.ops_op_set_version),
        json_string(&set.owner),
        sig_strs.join(","),
    );
    doc.into_bytes()
}

// ---------------------------------------------------------------------------
// reject_set -- the reader: typed decline on a missing/unknown field or a
// violated enumerant (§6.10-0006).
// ---------------------------------------------------------------------------

/// A typed decline from [`reject_set`] (§6.10-0006): never a panic. Only these
/// three kinds are needed at the primitive floor this reference slice
/// exercises (a REQUIRED top-level or per-signature field absent, an
/// unrecognized field present, or the `owner` enumerant violated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decline {
    MissingField(String),
    UnknownField(String),
    BadEnumerant(String),
}

const TOP_LEVEL_FIELDS: [&str; 5] =
    ["generator", "opattrs_wire_version", "ops_op_set_version", "owner", "signatures"];
const SIGNATURE_FIELDS: [&str; 4] = ["bytes", "edges", "nodes", "signature_hash"];

/// Every field in `required` MUST be present in `obj` (else `MissingField`,
/// checked in `required`'s order so the failure is deterministic), and every
/// field actually in `obj` MUST be in `required` (else `UnknownField`).
fn check_fields(obj: &[(String, Json)], required: &[&str]) -> Result<(), Decline> {
    for &r in required {
        if !obj.iter().any(|(k, _)| k == r) {
            return Err(Decline::MissingField(r.to_string()));
        }
    }
    for (k, _) in obj {
        if !required.contains(&k.as_str()) {
            return Err(Decline::UnknownField(k.clone()));
        }
    }
    Ok(())
}

/// Read + validate an Appendix-F expressible-signature-set document,
/// rejecting a set that omits a REQUIRED field, carries an unknown field, or
/// violates an enumerant, with a typed [`Decline`] (§6.10-0006) -- never a
/// panic. Malformed JSON / a non-object root is reported as a missing
/// `generator` field (the first required top-level key): there is no
/// dedicated "not JSON" decline in the pinned three-variant [`Decline`] enum,
/// and an unparseable document is, at minimum, missing every required field.
pub fn reject_set(json: &[u8]) -> Result<(), Decline> {
    let not_json = || Decline::MissingField(TOP_LEVEL_FIELDS[0].to_string());
    let text = std::str::from_utf8(json).map_err(|_| not_json())?;
    let doc = crate::json::parse(text).map_err(|_| not_json())?;
    let Json::Obj(top) = &doc else {
        return Err(not_json());
    };
    check_fields(top, &TOP_LEVEL_FIELDS)?;

    // enumerant: `owner` MUST be exactly `KISS-OPS` (Appendix F top-level fields).
    let owner = doc.get("owner").and_then(Json::as_str).unwrap_or("");
    if owner != "KISS-OPS" {
        return Err(Decline::BadEnumerant(format!("owner={owner}")));
    }

    // `signatures` (already confirmed present by `check_fields` above) MUST be
    // an array (Appendix F); present-but-wrong-shaped (e.g. a bare number or
    // an object) is as much a schema violation as absent, and must not
    // silently pass. Each element gets the same missing/unknown-field check.
    match doc.get("signatures") {
        Some(Json::Arr(sigs)) => {
            for s in sigs {
                if let Json::Obj(m) = s {
                    check_fields(m, &SIGNATURE_FIELDS)?;
                } else {
                    return Err(Decline::MissingField(SIGNATURE_FIELDS[0].to_string()));
                }
            }
        }
        _ => return Err(Decline::MissingField("signatures[array]".to_string())),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // FNV-1a-64 offset basis is, by construction, the hash of the empty byte
    // string (no update steps run) -- this is the algorithm's own defining
    // constant, not an external oracle, but it does pin that `fnv1a64` does
    // not e.g. accidentally run one spurious update on empty input.
    #[test]
    fn fnv1a64_empty_is_the_offset_basis() {
        assert_eq!(fnv1a64(&[]), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn fnv1a64_is_deterministic_and_input_sensitive() {
        let a = fnv1a64(b"add-signature-bytes");
        let b = fnv1a64(b"add-signature-bytes");
        let c = fnv1a64(b"sub-signature-bytes");
        assert_eq!(a, b, "same input must hash identically across calls");
        assert_ne!(a, c, "different input should (overwhelmingly likely) hash differently");
    }

    #[test]
    fn signature_hash_hex_is_16_lowercase_hex_digits() {
        let h = signature_hash_hex(b"\x01\x02\x03");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
