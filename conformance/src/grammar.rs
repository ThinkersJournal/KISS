//! Reference encoder/decoder for the KISS-Grammar **region wire form**
//! (Grammar §6.8), plus the canonicalization rules of §6.4 it depends on.
//!
//! A region is a single-rooted DAG of advertisable-op nodes over positional
//! `Bind(input_index)` leaves (§6.4-0001), serialized to a flat, indexed node
//! table prefixed by the magic `KGRM` and an in-band frozen-shape schema version
//! (§6.8-0013). The reference codec reproduces the Appendix A.1 golden vectors G1
//! (`out = (a*b)+c`) and G4 (`out = a+a`) byte-for-byte, and rejects malformed
//! streams with a typed decline (never a panic).
//!
//! Region-level field order (§6.8-0001): magic+schema_version, `n_inputs`,
//! `ops_version`, `classify_version`, `node_count`, node records, `extract_count`,
//! extract records. Integer widths (§6.8-0002) and token encoding (§6.8-0003) are
//! fixed. The node table is emitted in canonical post-order from the root
//! (§6.8-0004) after commutative-operand canonicalization (§6.4-0005/§6.4-0010)
//! and maximal structural sharing (§6.4-0011).
//!
//! Determinism class: **exact byte compare** over the region wire form
//! (§6.0-0001).

use std::collections::{BTreeSet, HashMap};

/// The region wire-form magic (§6.8-0013): ASCII `KGRM`, wire bytes 4B 47 52 4D.
pub const MAGIC: [u8; 4] = *b"KGRM";
/// The illustrative frozen-shape schema version of the Appendix A.1 vectors
/// (§6.8-0013): `1`, encoded as a `u16` little-endian.
pub const SCHEMA_VERSION: u16 = 1;

/// The `consumers` enum wire values (§6.8-0005).
pub const CONSUMERS_INTERIOR: u8 = 0x00;
pub const CONSUMERS_ROOT: u8 = 0x01;
/// The node-record `kind` tag values (§6.8-0010), shared with the §6.4-0010
/// canonical subtree serialization.
pub const KIND_BIND: u8 = 0x00;
pub const KIND_OP: u8 = 0x01;
/// The dtype-role wildcard token `*` (§6.6-0006), the single byte 0x2A.
pub const WILDCARD: &str = "*";
/// The default operand-role entry (§6.1-0008): role `data`, dtype-role `*`.
pub const DEFAULT_ROLE: &str = "data";

// ---------------------------------------------------------------------------
// Logical region model (author-facing input to the encoder).
// ---------------------------------------------------------------------------

/// One operand-role entry: a role token from the KISS-Ops operand-role set
/// (§6.6-0005) and a dtype role (a KISS-Classify dtype token or the wildcard `*`,
/// §6.6-0006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleEntry {
    pub role: String,
    pub dtype: String,
}

impl RoleEntry {
    /// The default (unconstrained) entry `(data, *)` (§6.1-0008).
    pub fn default_entry() -> Self {
        RoleEntry { role: DEFAULT_ROLE.into(), dtype: WILDCARD.into() }
    }
    fn is_default(&self) -> bool {
        self.role == DEFAULT_ROLE && self.dtype == WILDCARD
    }
}

/// A logical region node (§6.4-0001): either a positional-input bind leaf or an
/// advertisable-op node. Interior sharing is expressed by the encoder's
/// structural dedup (§6.4-0011); the author writes the natural tree and the
/// codec canonicalizes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// `Bind(input_index)` — binds this leaf to the region's positional input.
    Bind(u32),
    /// An advertisable-op node. `opattrs` is the narrow per-node `pattern_attrs`
    /// field: the KISS-Ops OpAttrs blob, carried uninterpreted (§6.8-0007).
    Op {
        op_name: String,
        opattrs: Vec<u8>,
        roles: Vec<RoleEntry>,
        operands: Vec<Node>,
    },
}

impl Node {
    /// Convenience constructor for an op node with empty OpAttrs and all-default
    /// operand roles.
    pub fn op(op_name: &str, operands: Vec<Node>) -> Node {
        Node::Op { op_name: op_name.into(), opattrs: Vec::new(), roles: Vec::new(), operands }
    }
}

/// A root-anchored scalar-param extract record (§6.4-0007, §6.8-0006). `path` is a
/// sequence of canonical operand indices from the root; `param_slot` is a 0-based
/// slot. (This reference takes the path as given and encodes it per §6.8-0006;
/// canonical-path derivation for shared targets, §6.4-0007, is out of this codec's
/// scope.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extract {
    pub path: Vec<u16>,
    pub param_slot: u32,
}

/// A logical region: its declared arity, upstream version bindings, its root
/// node, and its extract list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub n_inputs: u32,
    pub ops_version: String,
    pub classify_version: String,
    pub root: Node,
    pub extracts: Vec<Extract>,
}

impl Region {
    /// A region with no extracts (the Appendix A.1 G1/G4 shape).
    pub fn new(n_inputs: u32, ops_version: &str, classify_version: &str, root: Node) -> Region {
        Region {
            n_inputs,
            ops_version: ops_version.into(),
            classify_version: classify_version.into(),
            root,
            extracts: Vec::new(),
        }
    }
}

/// A typed decline from the encoder (§6.4-0008 expressibility, never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decline {
    /// The bound-input set is a strict subset of `[0, n_inputs)` — an input is
    /// unbound. Detectable only because `n_inputs` is a **declared** field
    /// (§6.4-0008); an implementation that inferred `n_inputs = max(bound)+1`
    /// could not raise this.
    BindSetMismatch { declared: u32, unbound: u32 },
    /// A bind names an index `>= n_inputs` (§6.4-0008).
    BindIndexOutOfRange { index: u32, n_inputs: u32 },
}

// ---------------------------------------------------------------------------
// The commutativity oracle — read from KISS-Ops by name (§6.2-0005, §6.4-0005).
// ---------------------------------------------------------------------------

/// The reference commutativity oracle: mirrors the KISS-Ops-declared per-op
/// commutativity property, read by name (§6.4-0005), **not** a Grammar-owned
/// literal identity list. `add`/`mul` are commutative; comparisons and `select`
/// are positional (§6.4-0006). A conforming impl reads this from KISS-Ops; the
/// harness reproduces it.
pub fn is_commutative(op_name: &str) -> bool {
    matches!(op_name, "add" | "mul")
}

// ---------------------------------------------------------------------------
// Token / integer helpers (§6.8-0002, §6.8-0003).
// ---------------------------------------------------------------------------

fn push_token(out: &mut Vec<u8>, s: &str) {
    // §6.8-0003: u16-LE byte length, then that many UTF-8 bytes. No NUL, no pad.
    out.extend_from_slice(&(s.len() as u16).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// Encode a node's operand-role tuple in canonical form (§6.1-0008, §6.8-0008):
/// a `u16` entry count `k` (0 if every operand is the default `(data, *)`, else
/// `1 +` the greatest operand index differing from the default), then `k`
/// entries — each `(role token, dtype token)` — with trailing defaults omitted.
/// Every operand index `>= k` reconstructs as `(data, *)`.
pub fn encode_operand_roles(roles: &[RoleEntry], operand_count: usize) -> Vec<u8> {
    let entry_at = |i: usize| -> RoleEntry {
        roles.get(i).cloned().unwrap_or_else(RoleEntry::default_entry)
    };
    // k = 1 + greatest index whose entry differs from the default; else 0.
    let mut k = 0usize;
    for i in 0..operand_count {
        if !entry_at(i).is_default() {
            k = i + 1;
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(&(k as u16).to_le_bytes());
    for i in 0..k {
        let e = entry_at(i);
        push_token(&mut out, &e.role);
        push_token(&mut out, &e.dtype);
    }
    out
}

// ---------------------------------------------------------------------------
// Tag canonical serialization (§6.8-0012).
// ---------------------------------------------------------------------------

/// Serialize an advertisable-op **tag** to its dedicated canonical serialization
/// (§6.8-0012 / §6.1-0007): (1) `op_name` as a length-prefixed token (§6.8-0003),
/// (2) the OpAttrs sub-block as a `u16`-LE byte length + the verbatim KISS-Ops
/// OpAttrs bytes (§6.8-0007), (3) the operand-role tuple in canonical form
/// (§6.1-0008 / §6.8-0008). It composes the **same** primitives the region node
/// record uses (`push_token`, the length-prefixed OpAttrs blob, and
/// `encode_operand_roles`), but carries **only** these identity-bearing fields and
/// **none** of the region framing — no magic/schema-version header, no `kind`
/// tag, no `consumers`, no `operand_count`, no operand indices, no
/// `ops_version`/`classify_version` tokens, and no `extract` records. Tag equality
/// (§6.1-0007(c)) is byte-exact comparison of this serialization. `synthesis_attrs`
/// are not identity-bearing (§6.1-0007(a)) and are not an input to this function.
pub fn encode_tag(op_name: &str, opattrs: &[u8], roles: &[RoleEntry], operand_count: usize) -> Vec<u8> {
    let mut out = Vec::new();
    // (1) op_name token (§6.8-0003).
    push_token(&mut out, op_name);
    // (2) OpAttrs sub-block: u16-LE length + verbatim KISS-Ops bytes (§6.8-0007).
    out.extend_from_slice(&(opattrs.len() as u16).to_le_bytes());
    out.extend_from_slice(opattrs);
    // (3) operand-role tuple in canonical form (§6.1-0008 / §6.8-0008).
    out.extend_from_slice(&encode_operand_roles(roles, operand_count));
    out
}

// ---------------------------------------------------------------------------
// Canonical subtree serialization + operand order (§6.4-0010).
// ---------------------------------------------------------------------------

/// The canonical subtree serialization of a node (§6.4-0010): for `Bind(i)`,
/// byte 0x00 then `input_index` as `u32` LE; for `Op`, byte 0x01, the
/// length-prefixed `op_name`, the `consumers` byte (INTERIOR — operands are never
/// the root), the length-prefixed OpAttrs blob, the operand-role tuple block, and
/// the operand subtree serializations in canonical order (recursively).
pub fn subtree_key(node: &Node) -> Vec<u8> {
    let mut out = Vec::new();
    match node {
        Node::Bind(i) => {
            out.push(KIND_BIND);
            out.extend_from_slice(&i.to_le_bytes());
        }
        Node::Op { op_name, opattrs, roles, operands } => {
            out.push(KIND_OP);
            push_token(&mut out, op_name);
            out.push(CONSUMERS_INTERIOR);
            out.extend_from_slice(&(opattrs.len() as u16).to_le_bytes());
            out.extend_from_slice(opattrs);
            out.extend_from_slice(&encode_operand_roles(roles, operands.len()));
            for child in canonical_operands(op_name, operands) {
                out.extend_from_slice(&subtree_key(child));
            }
        }
    }
    out
}

/// A node's operands in canonical order (§6.4-0010): for a KISS-Ops-declared
/// commutative op, ascending unsigned-byte-lexicographic over each operand's
/// canonical subtree key; a positional op keeps author order (§6.4-0006).
fn canonical_operands<'a>(op_name: &str, operands: &'a [Node]) -> Vec<&'a Node> {
    let mut v: Vec<&Node> = operands.iter().collect();
    if is_commutative(op_name) {
        // Stable sort; ties (identical keys) resolve to the same deduped index.
        v.sort_by(|a, b| subtree_key(a).cmp(&subtree_key(b)));
    }
    v
}

fn collect_binds(node: &Node, set: &mut BTreeSet<u32>) {
    match node {
        Node::Bind(i) => {
            set.insert(*i);
        }
        Node::Op { operands, .. } => {
            for c in operands {
                collect_binds(c, set);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Encoder: logical region -> canonical wire bytes (§6.8).
// ---------------------------------------------------------------------------

struct Builder {
    records: Vec<Vec<u8>>,
    index_of: HashMap<Vec<u8>, u32>,
}

impl Builder {
    /// Post-order visit assigning each distinct canonical subtree its table index
    /// at first finish (§6.8-0004), with maximal structural sharing (§6.4-0011):
    /// a subtree whose canonical key was already emitted reuses that index.
    fn visit(&mut self, node: &Node, is_root: bool) -> u32 {
        let key = subtree_key(node);
        if !is_root {
            if let Some(&idx) = self.index_of.get(&key) {
                return idx;
            }
        }
        let record = match node {
            Node::Bind(i) => {
                let mut r = Vec::new();
                r.push(KIND_BIND);
                r.extend_from_slice(&i.to_le_bytes());
                r
            }
            Node::Op { op_name, opattrs, roles, operands } => {
                // Children first (post-order): their indices are strictly lower.
                let ordered = canonical_operands(op_name, operands);
                let child_indices: Vec<u32> =
                    ordered.iter().map(|c| self.visit(c, false)).collect();
                let mut r = Vec::new();
                r.push(KIND_OP);
                push_token(&mut r, op_name);
                r.push(if is_root { CONSUMERS_ROOT } else { CONSUMERS_INTERIOR });
                r.extend_from_slice(&(opattrs.len() as u16).to_le_bytes());
                r.extend_from_slice(opattrs);
                r.extend_from_slice(&encode_operand_roles(roles, operands.len()));
                r.extend_from_slice(&(child_indices.len() as u16).to_le_bytes());
                for ci in child_indices {
                    r.extend_from_slice(&ci.to_le_bytes());
                }
                r
            }
        };
        let idx = self.records.len() as u32;
        self.records.push(record);
        self.index_of.insert(key, idx);
        idx
    }
}

/// Serialize a region to the canonical region wire form (§6.8), or return a typed
/// decline if the region is not expressible (§6.4-0008).
pub fn encode(region: &Region) -> Result<Vec<u8>, Decline> {
    // Expressibility (§6.4-0008): bound set must equal exactly [0, n_inputs).
    let mut bound = BTreeSet::new();
    collect_binds(&region.root, &mut bound);
    for &b in &bound {
        if b >= region.n_inputs {
            return Err(Decline::BindIndexOutOfRange { index: b, n_inputs: region.n_inputs });
        }
    }
    for i in 0..region.n_inputs {
        if !bound.contains(&i) {
            return Err(Decline::BindSetMismatch { declared: region.n_inputs, unbound: i });
        }
    }

    let mut b = Builder { records: Vec::new(), index_of: HashMap::new() };
    b.visit(&region.root, true);

    let mut out = Vec::new();
    // (0) magic + in-band frozen-shape schema version (§6.8-0013).
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&SCHEMA_VERSION.to_le_bytes());
    // (1) n_inputs u32 (§6.8-0002).
    out.extend_from_slice(&region.n_inputs.to_le_bytes());
    // (2)(3) version tokens (§6.8-0003).
    push_token(&mut out, &region.ops_version);
    push_token(&mut out, &region.classify_version);
    // (4) node_count u32; (5) node records in canonical order.
    out.extend_from_slice(&(b.records.len() as u32).to_le_bytes());
    for r in &b.records {
        out.extend_from_slice(r);
    }
    // (6) extract_count u16; (7) extract records, ascending param_slot (§6.8-0011).
    let mut ex = region.extracts.clone();
    ex.sort_by(|a, b| (a.param_slot, &a.path).cmp(&(b.param_slot, &b.path)));
    out.extend_from_slice(&(ex.len() as u16).to_le_bytes());
    for e in &ex {
        out.extend_from_slice(&(e.path.len() as u16).to_le_bytes());
        for step in &e.path {
            out.extend_from_slice(&step.to_le_bytes());
        }
        out.extend_from_slice(&e.param_slot.to_le_bytes());
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Decoder / validating reader (§6.8 round-trip; §6.4-0001 strictly-earlier).
// ---------------------------------------------------------------------------

/// A parsed node from the wire form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedNode {
    Bind(u32),
    Op {
        op_name: String,
        consumers: u8,
        opattrs: Vec<u8>,
        roles: Vec<RoleEntry>,
        operands: Vec<u32>,
    },
}

/// A parsed region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRegion {
    pub n_inputs: u32,
    pub ops_version: String,
    pub classify_version: String,
    pub nodes: Vec<ParsedNode>,
    pub extracts: Vec<Extract>,
}

/// A typed decline from the validating reader (never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeDecline {
    BadMagic,
    UnsupportedSchemaVersion { got: u16 },
    Truncated,
    BadUtf8,
    BadKind { got: u8 },
    BadConsumers { got: u8 },
    /// An operand index that does not reference a strictly-earlier table entry
    /// (§6.4-0001, §6.8-0004).
    ForwardOperandReference { node: u32, operand: u32 },
    /// The last (root) node does not carry `consumers = ROOT` (§6.4-0002/0004).
    RootNotRoot,
    /// More than one node carries `consumers = ROOT` — a multi-output forest,
    /// which is not an expressible single-rooted region (§6.4-0009). A region has
    /// exactly one root; a second ROOT-marked node means a second output. The
    /// last-node-is-ROOT check alone (below) would silently admit such a forest, so
    /// this counts the ROOT-marked nodes and declines when there is more than one.
    MultipleRoots { count: usize },
    TrailingBytes,
}

struct Cur<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Cur<'a> {
    fn u8(&mut self) -> Result<u8, DecodeDecline> {
        let v = *self.b.get(self.p).ok_or(DecodeDecline::Truncated)?;
        self.p += 1;
        Ok(v)
    }
    fn u16(&mut self) -> Result<u16, DecodeDecline> {
        if self.p + 2 > self.b.len() {
            return Err(DecodeDecline::Truncated);
        }
        let v = u16::from_le_bytes([self.b[self.p], self.b[self.p + 1]]);
        self.p += 2;
        Ok(v)
    }
    fn u32(&mut self) -> Result<u32, DecodeDecline> {
        if self.p + 4 > self.b.len() {
            return Err(DecodeDecline::Truncated);
        }
        let v = u32::from_le_bytes(self.b[self.p..self.p + 4].try_into().unwrap());
        self.p += 4;
        Ok(v)
    }
    fn take(&mut self, n: usize) -> Result<Vec<u8>, DecodeDecline> {
        if self.p + n > self.b.len() {
            return Err(DecodeDecline::Truncated);
        }
        let v = self.b[self.p..self.p + n].to_vec();
        self.p += n;
        Ok(v)
    }
    fn token(&mut self) -> Result<String, DecodeDecline> {
        let n = self.u16()? as usize;
        let raw = self.take(n)?;
        String::from_utf8(raw).map_err(|_| DecodeDecline::BadUtf8)
    }
}

/// Parse and validate a region wire-form byte stream (§6.8). Enforces the magic
/// and in-band schema version (§6.8-0013), the `consumers` enum domain
/// (§6.8-0005), that every operand index references a strictly-earlier entry
/// (§6.4-0001), and that the last node is the root (§6.4-0002).
pub fn decode(bytes: &[u8]) -> Result<ParsedRegion, DecodeDecline> {
    let mut c = Cur { b: bytes, p: 0 };
    if c.take(4)? != MAGIC {
        return Err(DecodeDecline::BadMagic);
    }
    let sv = c.u16()?;
    if sv != SCHEMA_VERSION {
        return Err(DecodeDecline::UnsupportedSchemaVersion { got: sv });
    }
    let n_inputs = c.u32()?;
    let ops_version = c.token()?;
    let classify_version = c.token()?;
    let node_count = c.u32()?;
    let mut nodes = Vec::new();
    for idx in 0..node_count {
        let kind = c.u8()?;
        match kind {
            KIND_BIND => {
                let i = c.u32()?;
                nodes.push(ParsedNode::Bind(i));
            }
            KIND_OP => {
                let op_name = c.token()?;
                let consumers = c.u8()?;
                if consumers != CONSUMERS_INTERIOR && consumers != CONSUMERS_ROOT {
                    return Err(DecodeDecline::BadConsumers { got: consumers });
                }
                let oalen = c.u16()? as usize;
                let opattrs = c.take(oalen)?;
                let k = c.u16()? as usize;
                let mut roles = Vec::new();
                for _ in 0..k {
                    let role = c.token()?;
                    let dtype = c.token()?;
                    roles.push(RoleEntry { role, dtype });
                }
                let opc = c.u16()? as usize;
                let mut operands = Vec::new();
                for _ in 0..opc {
                    let oi = c.u32()?;
                    if oi >= idx {
                        return Err(DecodeDecline::ForwardOperandReference { node: idx, operand: oi });
                    }
                    operands.push(oi);
                }
                nodes.push(ParsedNode::Op { op_name, consumers, opattrs, roles, operands });
            }
            other => return Err(DecodeDecline::BadKind { got: other }),
        }
    }
    // §6.4-0009: a region has exactly one root. Count the ROOT-marked nodes and
    // decline a multi-output forest before the last-node-is-ROOT check (which alone
    // would admit a second, interior ROOT-marked node — a 2-output forest).
    let root_count = nodes
        .iter()
        .filter(|n| matches!(n, ParsedNode::Op { consumers, .. } if *consumers == CONSUMERS_ROOT))
        .count();
    if root_count > 1 {
        return Err(DecodeDecline::MultipleRoots { count: root_count });
    }
    if let Some(ParsedNode::Op { consumers, .. }) = nodes.last() {
        if *consumers != CONSUMERS_ROOT {
            return Err(DecodeDecline::RootNotRoot);
        }
    }
    let ecount = c.u16()? as usize;
    let mut extracts = Vec::new();
    for _ in 0..ecount {
        let plen = c.u16()? as usize;
        let mut path = Vec::new();
        for _ in 0..plen {
            path.push(c.u16()?);
        }
        let param_slot = c.u32()?;
        extracts.push(Extract { path, param_slot });
    }
    if c.p != bytes.len() {
        return Err(DecodeDecline::TrailingBytes);
    }
    Ok(ParsedRegion { n_inputs, ops_version, classify_version, nodes, extracts })
}

// ---- Appendix A.1 golden region vectors: the CANONICAL builders ---------------------------
//
// ⚠️ THESE LIVE IN THE LIBRARY, NOT IN A TEST, AND THAT IS THE POINT. Before this, `g1_region()`
// was defined THREE times -- in `grammar_golden.rs`, `grammar_canonical.rs` and `grammar_tag.rs`.
// Measured at the time of consolidation: all three were identical in substance (the only textual
// difference was a trailing comma), so this is a LIVE HAZARD being closed, not a live defect being
// repaired. Three copies of a golden vector drift silently and nothing compares them; one copy
// cannot.
//
// The artifact generator below and every grammar test now read the SAME region from here, so a
// change to the golden shape reaches the emitted bytes and every assertion at once.

/// Appendix A.1 vector **G1** — the three-input elementwise fusion of §2.3, `out = (a * b) + c`.
/// `n_inputs = 3`, five canonical nodes, root `add` with interior `mul`.
pub fn g1_region() -> Region {
    Region::new(
        3,
        "1",
        "1",
        Node::op(
            "add",
            vec![
                Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]),
                Node::Bind(2),
            ],
        ),
    )
}

/// Appendix A.1 vector **G4** — the repeated-bind / structural-dedup vector, `out = a + a`,
/// `n_inputs = 1`. The same input is bound twice, so the node table carries one `Bind(0)`.
pub fn g4_region() -> Region {
    Region::new(1, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Bind(0)]))
}

/// `spec/grammar.md`, baked in at COMPILE time.
///
/// ⚠️ `include_str!`, NOT a runtime read, and the distinction is load-bearing. The generator must
/// emit byte-identical output wherever it runs, so it cannot depend on a file being findable at
/// runtime; and it must not depend on a HAND-COPIED list, which is the defect this replaces. A
/// compile-time include is both: derived from the document, and fixed at build time.
/// (`namespace_vocabulary.rs` includes the registry the same way.)
const GRAMMAR_SPEC: &str = include_str!("../../spec/grammar.md");

/// Every golden region vector Appendix A.1 DECLARES, derived from the appendix itself.
///
/// ⚠️ THIS WAS A HAND-TYPED LIST AND THAT WAS THE DEFECT. The first version of this artifact wrote
/// `["G1","G2","G3","G4","G5"]` as a literal and computed nothing from it, so **if Appendix A.1
/// gained a G6, nothing would notice** — a hand-maintained list guarding against a
/// hand-maintenance failure, which has exactly one failure mode and it is the one it guards
/// against. The generator could only ever report what it RENDERED; the "declared" set was a
/// qualitative claim with no failure state (baracuda's accumulate-on-execution shape).
///
/// Now the declared set is read out of the appendix and `unrendered` is the DIFFERENCE, so a new
/// appendix vector appears in `unrendered_vectors` on the next generation without anyone editing
/// this file.
pub fn declared_appendix_vectors() -> Vec<String> {
    let start = match GRAMMAR_SPEC.find("## Appendix A") {
        Some(i) => i,
        None => panic!(
            "spec/grammar.md has no `## Appendix A` — the declared-vector derivation cannot run, \
             and an EMPTY declared set would silently read as 'nothing is unrendered'"
        ),
    };
    let rest = &GRAMMAR_SPEC[start..];
    // ⚠️ NO `rest[3..]` OFFSET. A byte-index skip panics on a short slice and on a
    // non-char boundary, and it bought nothing: `rest` begins at "## Appendix A" with NO
    // leading newline, so the search below cannot match the current heading anyway.
    let end = rest.find("
## ").unwrap_or(rest.len());
    let appendix = &rest[..end];

    let mut out: Vec<String> = Vec::new();
    let mut rem = appendix;
    while let Some(i) = rem.find("*Vector ") {
        let tail = &rem[i + "*Vector ".len()..];
        let id: String = tail
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if !id.is_empty() && !out.contains(&id) {
            out.push(id);
        }
        rem = tail;
    }
    // ⚠️ A DERIVATION THAT FINDS NOTHING MUST NOT READ AS "NOTHING IS DECLARED". If the appendix's
    // vector-heading form changes, an empty result would make `unrendered_vectors` empty too and
    // the artifact would claim complete coverage of a set it never found.
    assert!(
        !out.is_empty(),
        "no `*Vector <id>` headings found in spec/grammar.md Appendix A — the derivation is \
         broken, and an empty declared set would report the artifact as covering everything"
    );
    out
}

/// The golden region vectors this codec can render, as `(id, description, region)`.
///
/// ⚠️ TWO OF THE FIVE, AND THE ARTIFACT SAYS SO RATHER THAN LEAVING IT TO BE DISCOVERED.
/// `spec/grammar.md` Appendix A.1 declares G1..G5. G2 (the load-bearing `gather` of §2.4), G3
/// (tag-equality / default-attribute) and G5 (DAG-shared-target extract) have **no builder
/// anywhere in this crate** -- measured: zero references to `g2_region`/`g3_region`/`g5_region`
/// and no `G2_/G3_/G5_GOLDEN` constant. Rendering them is a separate piece of work; claiming
/// them here would be the defect this artifact exists to remove.
pub fn golden_regions() -> Vec<(&'static str, &'static str, Region)> {
    vec![
        (
            "G1",
            "three-input elementwise fusion (§2.3), out = (a * b) + c",
            g1_region(),
        ),
        (
            "G4",
            "repeated-bind / structural dedup, out = a + a",
            g4_region(),
        ),
    ]
}

// ---- artifact-rendering helpers (module level, not nested in the generator) ----------------

/// Contiguous LOWERCASE hex — the artifact's wire-form spelling.
///
/// ⚠️ DELIBERATELY NOT `crate::hex`, WHICH IS A DIFFERENT FORMAT. `crate::hex` renders
/// space-separated UPPERCASE (the spec's Appendix-E display convention, "bytes on the wire, left
/// to right"). This is the machine-readable form, and the suite's own byte conventions pin
/// lowercase (KISS-CONTRACT §6.11-0003 "8 lowercase hex digits", §6.11-0010 "lowercase hex
/// digits"). Two formats, two functions, both named — never one function with a flag.
fn compact_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The node's wire CATEGORY.
///
/// ⚠️ AN EXPLICIT VOCABULARY, NEVER RUST'S `Debug`. `{:?}` renders a variant name that is an
/// implementation detail: renaming the enum would silently rewrite the artifact, and a foreign
/// reader would be parsing Rust's formatting rather than a declared vocabulary.
fn node_kind(n: &Node) -> &'static str {
    match n {
        Node::Bind(_) => "bind",
        Node::Op { .. } => "op",
    }
}

fn count_nodes(n: &Node) -> usize {
    match n {
        Node::Bind(_) => 1,
        Node::Op { operands, .. } => 1 + operands.iter().map(count_nodes).sum::<usize>(),
    }
}

/// One golden region vector as a JSON object, indented for the `vectors` array.
///
/// Extracted so the generator reads as "header, then the vectors" rather than as one long
/// sequence of pushes — a real unit (it renders exactly one vector) rather than a block moved out
/// to satisfy a line count.
fn render_vector(id: &str, desc: &str, region: &Region) -> String {
    let jstr = crate::json::escape_string;
    let bytes = encode(region).expect("a golden region vector must encode");
    let mut v = String::from("    {\n");
    v.push_str(&format!("      \"id\": {},\n", jstr(id)));
    v.push_str(&format!("      \"description\": {},\n", jstr(desc)));
    v.push_str(&format!("      \"n_inputs\": {},\n", region.n_inputs));
    v.push_str(&format!("      \"ops_version\": {},\n", jstr(&region.ops_version)));
    v.push_str(&format!(
        "      \"classify_version\": {},\n",
        jstr(&region.classify_version)
    ));
    v.push_str(&format!("      \"root_kind\": {},\n", jstr(node_kind(&region.root))));
    v.push_str(&format!("      \"node_count\": {},\n", count_nodes(&region.root)));
    v.push_str(&format!("      \"extract_count\": {},\n", region.extracts.len()));
    v.push_str(&format!("      \"wire_bytes\": {},\n", bytes.len()));
    v.push_str(&format!("      \"wire_hex\": {}\n", jstr(&compact_hex(&bytes))));
    v.push_str("    }");
    v
}

/// Emit `conformance/corpus/grammar_vectors.json` — the machine-readable golden REGION vector
/// set, generated from this codec.
///
/// A LIBRARY generator, never a test helper (the `emit_contract_vectors_json` / `#365` pattern):
/// a generator that lives in a test cannot be run to refresh the artifact, and the committed file
/// then becomes the only copy of a value nothing derives.
///
/// ⚠️ WHY THIS ARTIFACT EXISTS AT ALL. `KISS-GRAMMAR-8-0005` requires a foreign reader **written
/// outside the reference language** to consume the region wire form. Before this, the golden bytes
/// existed only as prose in an informative appendix and as a Rust `const` -- **a foreign reader can
/// consume neither.** This file is what §8-0004/-0005 give such a reader to reproduce FROM.
pub fn emit_grammar_vectors_json() -> String {
    // ⚠️ ONE escaper, in `json`, not a private copy per generator — see json::escape_string for
    // why: the private copies had already diverged, and neither escaped the C0 controls RFC 8259
    // requires.
    let jstr = crate::json::escape_string;

    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"schema\": \"kiss-grammar-region-vectors-v1\",\n");
    s.push_str("  \"generated_from\": \"conformance/src/grammar.rs::emit_grammar_vectors_json (the region codec)\",\n");
    s.push_str("  \"spec_source\": \"spec/grammar.md Appendix A.1\",\n");
    s.push_str("  \"clause\": \"KISS-GRAMMAR-6.8-0001\",\n");
    s.push_str("  \"freeze_gate\": \"KISS-GRAMMAR-8-0004 / -0005 (umbrella §5.3)\",\n");
    s.push_str("  \"coverage_note\": ");
    s.push_str(&jstr(COVERAGE_NOTE));
    s.push_str(",\n");
    // ⚠️ DERIVED, and `unrendered` is the DIFFERENCE rather than a second hand-typed list.
    let declared = declared_appendix_vectors();
    let rendered: Vec<String> = golden_regions().iter().map(|(id, _, _)| id.to_string()).collect();
    // A rendered id the appendix does not declare is a RAISE, not a silent extra row: it means the
    // codec is emitting a vector the spec does not name, which no count would reveal.
    for r in &rendered {
        assert!(
            declared.contains(r),
            "golden_regions() renders `{r}`, which spec/grammar.md Appendix A does not declare — \
             the artifact must not carry a vector the specification does not name"
        );
    }
    let unrendered: Vec<String> = declared
        .iter()
        .filter(|d| !rendered.contains(d))
        .cloned()
        .collect();
    let json_list = |v: &[String]| {
        v.iter()
            // ⚠️ THE CRATE'S ESCAPER, not a hand-rolled quote. The ids are alphanumeric
            // today, so this changes no byte -- which is exactly when a hand-rolled quote is
            // easiest to leave in. Two private JSON escapers had already diverged here once.
            .map(|x| crate::json::escape_string(x))
            .collect::<Vec<_>>()
            .join(", ")
    };
    s.push_str(&format!(
        "  \"declared_appendix_vectors\": [{}],\n",
        json_list(&declared)
    ));
    s.push_str(&format!("  \"rendered_vectors\": [{}],\n", json_list(&rendered)));
    s.push_str(&format!("  \"unrendered_vectors\": [{}],\n", json_list(&unrendered)));
    s.push_str("  \"vectors\": [\n");
    let regions = golden_regions();
    let rendered: Vec<String> = regions
        .iter()
        .map(|(id, desc, region)| render_vector(id, desc, region))
        .collect();
    s.push_str(&rendered.join(",\n"));
    s.push('\n');
    s.push_str("  ]\n");
    s.push_str("}\n");
    s
}

/// The artifact's stated population. ⚠️ PARTIAL AND STATED, NOT LEFT TO BE DISCOVERED --
/// the 16(d) principle, and the same discipline the `structure_key_vectors` coverage note uses.
const COVERAGE_NOTE: &str = "This artifact carries the region WIRE FORM of the Appendix A.1 \
golden vectors this codec can build: G1 and G4. It is generated from conformance/src/grammar.rs, \
and conformance/tests/grammar_vectors.rs asserts (a) the committed file is byte-identical to a \
fresh generation, so a stale artifact fails CI, and (b) the rendered bytes agree with the \
hand-maintained hex in spec/grammar.md Appendix A.1 -- which is the binding that did not exist \
before: the goldens were a Rust `const` with no tie to the spec, so the two could diverge \
silently. WHAT THIS DOES NOT CARRY, each with a different reason so a reader knows what to do \
next: (i) G2, G3 and G5 are DECLARED in Appendix A.1 and have no builder in this crate, so they \
are named in `unrendered_vectors` rather than omitted quietly -- rendering them needs the gather, \
default-attribute and extract shapes built, not a wider corpus. (ii) The op vocabulary is NOT \
exercised: G1 and G4 place `add`, `mul` and (via G2, absent) `gather`, so a divergence on any \
other KISS-Ops op name is invisible to a byte-match against this set -- op-name agreement is a \
different instrument's job. (iii) The declared KISS-Classify op-CATEGORY set has 24 members and \
no region vector here places any of them; a kernel class absent from every corpus in the suite \
is invisible to this artifact exactly as it is to the others, which is a coverage question owned \
upstream and not closed by generating this file. So 'the byte-match passed' means the region wire \
FORM agrees for the two shapes rendered -- not that the op vocabulary agrees, and not that the \
corpus spans the domain.";
