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