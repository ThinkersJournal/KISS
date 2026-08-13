//! Reference codec for the KISS-Contract structured/text **document** framing
//! (Contract §6.11) and the transport hard-reject reader discipline (Contract
//! §6.1). A contract is a UTF-8 text document — a pinned header line followed by
//! seven section blocks under pinned heading lines — self-delimited by the inner
//! body length its header line declares and integrity-checked by that line's
//! CRC-32 (§6.11-0002/-0003). It is **not** a length-prefixed binary envelope.
//!
//! Every field of the framing is determinism-class exact-byte (§6.0-0001) **with
//! one exception**: the optional, non-normative Semantics `human_annotation`
//! (§6.4-0001) sits **outside** that scope and MUST NOT be byte-compared. So the
//! exact-byte comparator (Conform §6.8) applies to the framing throughout, but to
//! a contract only after the projection in [`exact_byte_scoped`] — which removes
//! that one field, and must be applied to the BODY before the header's `len=` /
//! `crc32=` are derived, or the excluded field leaks back in through the
//! checksum. The golden document
//! bytes are transcribed from Contract Appendix C (the §2.5 strided `add`
//! contract: header line + Identity block + Semantics block).
//!
//! `std` has no CRC, so [`crc32_ieee`] is implemented from scratch: the IEEE
//! 802.3 polynomial (reflected `0xEDB88320`), initial `0xFFFFFFFF`, final XOR
//! `0xFFFFFFFF` (§6.11-0003).

// ---------------------------------------------------------------------------
// CRC-32 (IEEE 802.3), from scratch — §6.11-0003.
// ---------------------------------------------------------------------------

/// CRC-32 over `data` per §6.11-0003: IEEE 802.3 polynomial, **reflected**
/// (`0xEDB88320`), initial value `0xFFFFFFFF`, final XOR `0xFFFFFFFF`. This is the
/// canonical CRC-32/ISO-HDLC whose check value over ASCII `"123456789"` is
/// `0xCBF43926`.
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            // branchless conditional XOR of the reflected polynomial.
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

// ---------------------------------------------------------------------------
// §6.11-0001 — per-type field-value encoding.
// ---------------------------------------------------------------------------

/// A typed field value under the §6.11-0001 text encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A string / enumerated token: verbatim UTF-8 to the line's LF (MUST NOT
    /// contain an LF).
    Str(String),
    /// A decimal-ASCII integer (a negative value carries a leading `-`).
    Int(i64),
    /// An array: `[` then elements separated by `, ` then `]`, in pinned order.
    Array(Vec<String>),
    /// An opaque byte blob: `<n>:<hex>` where `<n>` is the decimal byte count and
    /// `<hex>` is exactly `2·n` **lowercase** hex digits (empty blob is `0:`).
    Blob(Vec<u8>),
    /// A real (floating-point) value pinned by its IEEE-754 bit pattern (§6.11-0010).
    Real(Real),
}

/// A real value carried by its IEEE 754-2019 bit pattern in a KISS-Classify float
/// dtype (§6.11-0010). Pinned by bits, never a decimal spelling (umbrella §3.5), so
/// ±0, ±∞, and NaN payloads round-trip exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Real {
    /// `f32` (IEEE-754 binary32) bit pattern.
    F32(u32),
    /// `f64` (IEEE-754 binary64) bit pattern.
    F64(u64),
}

impl Real {
    /// The `f32` real with the given value's bit pattern.
    pub fn f32(v: f32) -> Real {
        Real::F32(v.to_bits())
    }
    /// The `f64` real with the given value's bit pattern.
    pub fn f64(v: f64) -> Real {
        Real::F64(v.to_bits())
    }

    /// Render as `<dtype>:<hex>` — the dtype token, then the bit pattern as
    /// most-significant-byte-first (big-endian) lowercase hex, zero-padded to the
    /// dtype width (8 digits for `f32`, 16 for `f64`).
    pub fn render(&self) -> String {
        match self {
            Real::F32(bits) => format!("f32:{bits:08x}"),
            Real::F64(bits) => format!("f64:{bits:016x}"),
        }
    }
}

impl Value {
    /// Render the value bytes per §6.11-0001 (the text right of `<key> = `).
    pub fn render(&self) -> String {
        match self {
            Value::Str(s) => {
                debug_assert!(!s.contains('\n'), "§6.11-0001: a string value MUST NOT contain LF");
                s.clone()
            }
            Value::Int(n) => n.to_string(),
            Value::Array(elems) => format!("[{}]", elems.join(", ")),
            Value::Blob(bytes) => {
                let mut s = format!("{}:", bytes.len());
                for b in bytes {
                    s.push_str(&format!("{b:02x}"));
                }
                s
            }
            Value::Real(r) => r.render(),
        }
    }
}

/// One field line: `<key> = <value>` — the key, one space, `=` (`0x3D`), one
/// space, the value, then a single LF (§6.11-0001).
pub fn field_line(key: &str, value: &Value) -> String {
    format!("{} = {}\n", key, value.render())
}

// ---------------------------------------------------------------------------
// §6.11-0007 — op_dag serialization in KISS-Grammar canonical node order.
// ---------------------------------------------------------------------------

/// A Semantics op node as a tree, for the canonical serializer. `op_attrs` is the
/// per-node `key=value` OpAttrs list (empty when the op carries none).
#[derive(Debug, Clone)]
pub struct OpTree {
    pub op_name: String,
    pub op_attrs: Vec<(String, String)>,
    pub children: Vec<OpTree>,
}

impl OpTree {
    /// A leaf op node with no attrs (e.g. the §2.5 `add` primitive).
    pub fn leaf(op_name: &str) -> OpTree {
        OpTree { op_name: op_name.into(), op_attrs: vec![], children: vec![] }
    }
}

/// Serialize an op DAG as the §6.11-0007 array of nodes in **KISS-Grammar
/// canonical node order**: every child strictly earlier than its parent, so each
/// node's `child_edges` reference **strictly-earlier** indices and the **root
/// appears last**. This is a post-order walk that assigns each node its array
/// index only after its children, so a child's index is always `<` its parent's.
pub fn serialize_op_dag(root: &OpTree) -> String {
    let mut flat: Vec<String> = Vec::new();
    fn visit(node: &OpTree, flat: &mut Vec<String>) -> usize {
        // children FIRST, so each child index is strictly earlier than this node.
        let mut edges: Vec<usize> = Vec::with_capacity(node.children.len());
        for c in &node.children {
            edges.push(visit(c, flat));
        }
        let attrs = node
            .op_attrs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");
        let edges_s = edges.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let idx = flat.len();
        flat.push(format!("Op{{{}; {}; [{}]}}", node.op_name, attrs, edges_s));
        idx
    }
    let _root_idx = visit(root, &mut flat);
    format!("[{}]", flat.join(", "))
}

// ---------------------------------------------------------------------------
// Section blocks and the whole document — §6.11-0002/-0004/-0005.
// ---------------------------------------------------------------------------

/// Render a section block: the pinned heading line `[section:<id>:<name>]` and a
/// LF, then one field line per `(key, value)` in the given (schema) order
/// (§6.11-0004/-0005). Fields are emitted in the exact order supplied — this is
/// the ordered spine that a nondeterministic (HashMap-backed) emitter fails.
pub fn render_block(id: u8, name: &str, fields: &[(&str, Value)]) -> Vec<u8> {
    let mut s = format!("[section:{id}:{name}]\n");
    for (k, v) in fields {
        s.push_str(&field_line(k, v));
    }
    s.into_bytes()
}

/// A whole contract document = pinned header line + body (the section blocks).
#[derive(Debug, Clone)]
pub struct Document {
    pub contract_kind: String,
    pub contract_version: String,
    pub body: Vec<u8>,
}

impl Document {
    /// Serialize to the §6.11-0002 document bytes: the header line
    /// `KISC <kind> <version> len=<N> crc32=<HHHHHHHH>\n` (magic `0x4B 0x49 0x53
    /// 0x43`, single-space separators, `<N>` = decimal body length, `<HHHHHHHH>` =
    /// 8 lowercase hex CRC-32 of the body, then a single LF) followed by the body.
    pub fn encode(&self) -> Vec<u8> {
        let n = self.body.len();
        let crc = crc32_ieee(&self.body);
        let header = format!(
            "KISC {} {} len={} crc32={:08x}\n",
            self.contract_kind, self.contract_version, n, crc
        );
        let mut out = header.into_bytes();
        out.extend_from_slice(&self.body);
        out
    }
}

// ---------------------------------------------------------------------------
// §6.0-0001 — the exact-byte-scoped projection (the `human_annotation` exclusion).
// ---------------------------------------------------------------------------

/// The pinned Semantics section heading line (§6.11-0004; section id 2).
const SEMANTICS_HEADING: &[u8] = b"[section:2:semantics]\n";

/// The pinned field-line prefix of the one field outside the exact-byte scope.
/// `<key>`, one space, `=`, one space (§6.11-0001) — matched at a line start, so
/// a *value* that merely mentions the key is never mistaken for the field.
const BLURB_LINE_PREFIX: &[u8] = b"human_annotation = ";

/// Project a contract **body** onto the bytes KISS-Conform may byte-compare: the
/// body verbatim, minus the Semantics `human_annotation` field line.
///
/// `human_annotation` is the sole optional Semantics field (§6.4-0001) and the
/// sole field the contract text places **outside** the exact-byte determinism
/// scope (§6.0-0001). A comparator that includes it calls two contracts with
/// identical machine-checkable content different because a human edited a
/// comment.
///
/// The projection is deliberately narrow in two ways, both load-bearing:
///
/// * It strips the field **only inside the Semantics block**. `human_annotation`
///   is defined only there; the same key in any other block is an *unknown
///   field*, which a reader MUST decline (§6.11-0005). Stripping it document-wide
///   would hide that decline behind this exclusion.
/// * It matches the pinned line form anchored at a line start, never a substring,
///   so an `op_dag` or blurb value containing the text `human_annotation` is
///   untouched.
///
/// Note this operates on the **body**, before the header line is derived. The
/// §6.11-0002 header carries `len=` and `crc32=` computed over the body, so a
/// blurb edit changes them; projecting first and deriving the header from the
/// projection is what keeps the excluded field from leaking back in through the
/// checksum.
pub fn exact_byte_scoped(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut in_semantics = false;
    for line in body.split_inclusive(|&b| b == b'\n') {
        if line.starts_with(b"[section:") {
            in_semantics = line == SEMANTICS_HEADING;
        } else if in_semantics && line.starts_with(BLURB_LINE_PREFIX) {
            continue; // outside the exact-byte scope (§6.0-0001, §6.4-0001)
        }
        out.extend_from_slice(line);
    }
    out
}

// ---------------------------------------------------------------------------
// §6.1 — transport reader, hard-reject discipline (typed decline, never panic).
// ---------------------------------------------------------------------------

/// A typed decline from the contract transport reader (§6.1 hard-reject). Never a
/// panic, abort, crash, hang, or out-of-bounds read (§6.1-0004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractDecline {
    /// The document does not begin with the pinned magic `KISC` (§6.11-0002).
    NoMagic,
    /// The header line is absent (no LF) or does not match the pinned form
    /// (§6.11-0002) — includes a CR (CRLF is not the LF-only convention).
    MalformedHeader,
    /// The `contract_kind` is not the recognized token `kiss-contract` (§6.1-0007).
    UnknownKind { got: String },
    /// The `contract_version` is not `1` (§6.1-0008).
    UnknownVersion { got: String },
    /// The declared body length does not equal the actual body byte count
    /// (§6.11-0003). Carries the declared length as a `u64` — it is never used to
    /// size an allocation before this check (§6.1-0004).
    BadLength { declared: u64, actual: usize },
    /// The declared CRC-32 does not match the body's computed CRC-32 (§6.11-0003).
    BadChecksum { declared: u32, computed: u32 },
    /// The body does not begin with the first pinned section heading
    /// `[section:1:identity]` — a headingless block MUST NOT be imported
    /// (§6.11-0004, §6.1-0002).
    Headingless,
    /// The Guarantees block has no `determinism_class` field (§6.8-0003).
    MissingGuaranteesClass,
    /// The `determinism_class` token is not a member of the canonical KISS-Ops
    /// §6.0-0001 enum — a re-forked/unknown class (§6.8-0003, KISS-OPS §7.4-0001).
    UnknownDeterminismClass { got: String },
}

/// The header-line essentials a successful read yields (§6.11-0002/-0003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractHeader {
    pub contract_kind: String,
    pub contract_version: String,
    pub body_len: usize,
    pub crc32: u32,
}

/// Read the transport framing of a contract document with the §6.1 hard-reject
/// discipline: validate magic, header line, kind, version, the self-declared
/// inner length, the CRC-32, and the first section heading — returning a typed
/// [`ContractDecline`] on any malformation and **never** panicking, reading out
/// of bounds, or allocating on an unchecked declared length (§6.1-0002/-0004).
///
/// This validates the *transport* framing only (the header + inner
/// length/checksum + presence of the first heading). Full seven-section schema
/// validation is §6.11-0004 / §6.2-0001, out of this function's scope.
pub fn read_document(doc: &[u8]) -> Result<ContractHeader, ContractDecline> {
    // (1) Magic. Bounds-checked: a short buffer never reads out of range.
    if doc.len() < 4 || &doc[0..4] != b"KISC" {
        return Err(ContractDecline::NoMagic);
    }
    // (2) Header line = bytes up to the first LF. Absent LF => malformed.
    let lf = match doc.iter().position(|&b| b == b'\n') {
        Some(i) => i,
        None => return Err(ContractDecline::MalformedHeader),
    };
    let header = &doc[..lf];
    // LF-only: a CR anywhere in the header (a CRLF emitter) is a malformed header.
    if header.contains(&b'\r') {
        return Err(ContractDecline::MalformedHeader);
    }
    let header_str = match std::str::from_utf8(header) {
        Ok(s) => s,
        Err(_) => return Err(ContractDecline::MalformedHeader),
    };
    // Pinned form: `KISC <kind> <version> len=<N> crc32=<H>` — five space-joined
    // fields, single-space separators.
    let parts: Vec<&str> = header_str.split(' ').collect();
    if parts.len() != 5 || parts[0] != "KISC" {
        return Err(ContractDecline::MalformedHeader);
    }
    // (3) Recognized kind (§6.1-0007).
    let kind = parts[1];
    if kind != "kiss-contract" {
        return Err(ContractDecline::UnknownKind { got: kind.to_string() });
    }
    // (4) Supported version (§6.1-0008).
    let version = parts[2];
    if version != "1" {
        return Err(ContractDecline::UnknownVersion { got: version.to_string() });
    }
    // (5) `len=<N>` — decimal body byte count.
    let declared_len: u64 = match parts[3].strip_prefix("len=").and_then(|s| s.parse().ok()) {
        Some(n) => n,
        None => return Err(ContractDecline::MalformedHeader),
    };
    // (6) `crc32=<HHHHHHHH>` — exactly 8 lowercase hex digits.
    let declared_crc: u32 = match parts[4].strip_prefix("crc32=").and_then(parse_crc_hex) {
        Some(c) => c,
        None => return Err(ContractDecline::MalformedHeader),
    };
    // Body = bytes after the header LF. This is a slice of the *actual* input; no
    // allocation is keyed on `declared_len`.
    let body = &doc[lf + 1..];
    // §6.1-0004: validate the declared length against the ACTUAL buffer BEFORE
    // any work keyed on it. A huge declared_len against a short buffer declines
    // here — it is never handed to `Vec::with_capacity`.
    if declared_len != body.len() as u64 {
        return Err(ContractDecline::BadLength { declared: declared_len, actual: body.len() });
    }
    // §6.11-0003: integrity. CRC is computed over the actual (now length-checked)
    // body only.
    let computed = crc32_ieee(body);
    if computed != declared_crc {
        return Err(ContractDecline::BadChecksum { declared: declared_crc, computed });
    }
    // §6.11-0004 / §6.1-0002: the body MUST begin under the first pinned heading;
    // a headingless block MUST NOT be silently imported.
    if !body.starts_with(b"[section:1:identity]\n") {
        return Err(ContractDecline::Headingless);
    }
    Ok(ContractHeader {
        contract_kind: kind.to_string(),
        contract_version: version.to_string(),
        body_len: body.len(),
        crc32: declared_crc,
    })
}

// ---------------------------------------------------------------------------
// §6.8-0003 / KISS-OPS §7.4-0001 — Guarantees `determinism_class` advertisement.
// ---------------------------------------------------------------------------

/// Read the Guarantees block's `determinism_class` from a contract `body` and map
/// the canonical KISS-Ops §6.0-0001 token to [`crate::DeterminismClass`]. Locates
/// the `[section:6:guarantees]` heading (§6.11-0004) and reads *that block's*
/// `determinism_class = <token>` field line (§6.8-0003) — scoped to the
/// Guarantees block specifically, since a full seven-section document also
/// carries a same-named field in Capabilities (§6.7-0004) earlier in the body.
/// An absent block/field or an off-enum token is a typed [`ContractDecline`] —
/// never a panic, never a re-forked class (KISS-OPS §7.4-0001).
pub fn parse_guarantees_class(body: &[u8]) -> Result<crate::DeterminismClass, ContractDecline> {
    let text = std::str::from_utf8(body).map_err(|_| ContractDecline::MalformedHeader)?;
    // Line-anchored scan of a line-structured format: the heading is matched as a
    // FULL LINE (`line == HEADING`), never a substring — so a heading string that
    // appeared inside a field value cannot false-open the block. Read this block's
    // field lines from the heading line up to the next `[section:` heading line
    // (or EOF); a same-named field in an earlier section (e.g. Capabilities
    // §6.7-0004) is outside the block and never seen.
    const HEADING: &str = "[section:6:guarantees]";
    let mut in_block = false;
    for line in text.lines() {
        if line == HEADING {
            in_block = true;
            continue;
        }
        if line.starts_with("[section:") {
            if in_block {
                break; // reached the next section; the Guarantees block ended
            }
            continue;
        }
        if in_block {
            if let Some(token) = line.strip_prefix("determinism_class = ") {
                // The canonical KISS-Ops §6.0-0001 enum, spelled verbatim
                // (KISS-CONTRACT §6.8-0003): `exact-byte`, `ULP/tolerance`,
                // `order-invariant/nondeterministic`.
                return match token {
                    "exact-byte" => Ok(crate::DeterminismClass::ExactByte),
                    "ULP/tolerance" => Ok(crate::DeterminismClass::UlpTolerance),
                    "order-invariant/nondeterministic" => Ok(crate::DeterminismClass::OrderInvariant),
                    other => Err(ContractDecline::UnknownDeterminismClass { got: other.to_string() }),
                };
            }
        }
    }
    Err(ContractDecline::MissingGuaranteesClass)
}

/// Parse exactly 8 lowercase-hex digits into a `u32` (the `crc32=` field form).
fn parse_crc_hex(s: &str) -> Option<u32> {
    if s.len() == 8 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        u32::from_str_radix(s, 16).ok()
    } else {
        None
    }
}
// ---------------------------------------------------------------------------
// §6.8-0008/-0009/-0010 — the `audited_status` DERIVATION.
// ---------------------------------------------------------------------------

/// A per-backend **accuracy tier** (§6.8-0002): a tagged quantity carrying at
/// least one of `{max_ulp, max_relative, max_absolute}` against a named
/// reference. A tier carrying none of the three declares no bound.
///
/// The `correctly-rounded` and `bit-reproducible` precision classes are carried
/// here as `max_ulp = 0`, which is the §6.7-0007 precision-class↔tier
/// correspondence ("MUST map to tier 0"), not a separate representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccuracyTier {
    pub max_ulp: Option<u32>,
    /// Bit pattern of the declared relative bound, if any (§6.11-0010 real form).
    pub max_relative: Option<Real>,
    /// Bit pattern of the declared absolute bound, if any.
    pub max_absolute: Option<Real>,
}

impl AccuracyTier {
    /// A tier declares a bound iff it carries at least one of the three tagged
    /// quantities (§6.8-0002).
    pub fn is_bounded(&self) -> bool {
        self.max_ulp.is_some() || self.max_relative.is_some() || self.max_absolute.is_some()
    }
}

/// The `audited_status` value set. §6.8-0010 forbids producing a value neither
/// §6.8-0009 nor §6.8-0010 yields, so the derivation is TOTAL over exactly these
/// two — there is no third variant and no "unknown".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditedStatus {
    Audited,
    Unaudited,
}

/// The Guarantees fields the derivation reads, plus the one field it MUST NOT
/// read. `bit_stability` is present deliberately: §6.8-0009 forbids the rule
/// setting it (that field is owned by §6.8-0005), and a rule that cannot see the
/// field cannot be shown not to use it.
#[derive(Debug, Clone)]
pub struct Guarantees {
    /// The NAMED KISS-Ops function precision is measured against (§6.8-0002).
    /// `None` — or a name that is empty/whitespace — is "no reference named".
    pub reference_function: Option<String>,
    /// Per target backend, the declared accuracy tier (§6.8-0002). Empty means
    /// no tier is declared for any backend.
    pub per_backend_ulp_tiers: Vec<(String, AccuracyTier)>,
    pub determinism_class: crate::DeterminismClass,
    /// Read by NO branch of the derivation (§6.8-0009, §6.8-0005 owns it).
    pub bit_stability: bool,
}

impl Guarantees {
    /// Whether the Guarantees declare a **bounded precision against a named
    /// reference function** — the single predicate §6.8-0009 and §6.8-0010
    /// partition on.
    fn declares_bounded_precision(&self) -> bool {
        let named = self
            .reference_function
            .as_deref()
            .is_some_and(|r| !r.trim().is_empty());
        named && self.per_backend_ulp_tiers.iter().any(|(_, t)| t.is_bounded())
    }
}

/// Derive `audited_status` from the Guarantees (§6.8-0008): `audited` when the
/// Guarantees declare a bounded precision against a named `reference_function`
/// (§6.8-0009), `unaudited` when they do not (§6.8-0010).
///
/// The determinism class does **not** gate the result. §6.8-0009 says the
/// `audited` arm **includes** an `order-invariant/nondeterministic` kernel whose
/// nondeterminism is declared against a named reference under a stated tolerance
/// — so "nondeterministic therefore unaudited" is the wrong rule, not a
/// conservative one. `bit_stability` is likewise never read (§6.8-0005 owns it).
pub fn derive_audited_status(g: &Guarantees) -> AuditedStatus {
    if g.declares_bounded_precision() {
        AuditedStatus::Audited
    } else {
        AuditedStatus::Unaudited
    }
}

/// A declared `audited_status` that disagrees with the value the rule derives —
/// i.e. an authored constant (§6.8-0008).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditedStatusMismatch {
    pub declared: AuditedStatus,
    pub derived: AuditedStatus,
}

/// The KISS-Conform check (§6.13-0021): a contract's **declared**
/// `audited_status` MUST equal the value derived from its own Guarantees.
/// A mismatch is the observable signature of a hardcoded field.
pub fn verify_audited_status(
    declared: AuditedStatus,
    g: &Guarantees,
) -> Result<(), AuditedStatusMismatch> {
    let derived = derive_audited_status(g);
    if declared == derived {
        Ok(())
    } else {
        Err(AuditedStatusMismatch { declared, derived })
    }
}
