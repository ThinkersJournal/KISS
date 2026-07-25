//! KISS-Conform vocabulary-rebasing & upstream-binding vectors for the
//! KISS-Grammar region codec (Grammar §6.6 token spelling, §6.1 tag identity,
//! §6.2 Ops-owned OOB set), plus the two cross-cutting invariants they rest on:
//! the §6.0 exact-byte determinism class and the §7.1 typed-decline totality of
//! the codec. Every test drives the EXISTING reference codec in
//! `kiss_conformance::grammar` (encode / decode / subtree_key / MAGIC /
//! SCHEMA_VERSION / the typed `Decline`/`DecodeDecline`s) through the region wire
//! form alone — no lifter, matcher, tag-serialization fn, or op-name oracle is
//! invented here.
//!
//! Two tests carry a **real cross-standard payload**: the KISS-Ops OpAttrs blob
//! produced/validated by `kiss_conformance::opattrs`. That module is used purely
//! as a byte payload and an Ops-side validator, referenced **by content** (its
//! `encode_gather`/`decode_gather` functions and `OobPolicy`/`IndexDtype` types)
//! and never by clause id — so the Grammar codec's uninterpreted carriage of an
//! Ops-owned sub-vocabulary is attested without reverse-binding any KISS-Ops
//! clause (the established batch-2 precedent of `grammar_canonical.rs`).
//!
//! To isolate the SPELLING / identity property from commutative
//! canonicalization, the op-name and dtype tests use single-operand
//! non-commutative ops (relu-style) so only the token bytes differ. Every test is
//! named exactly per the spec *Test:* tag, opens with its single KISS-Grammar
//! clause id, and proves that id by MUTATION — a plausible codec/spec drift makes
//! the test fail. No substring greps, no `assert!(true)`, no self-compares, no
//! set-vs-hardcoded-copy, no binding to a derived `PartialEq`, no local codec
//! re-implementation.

use kiss_conformance::grammar::*;
use kiss_conformance::opattrs;

// ---------------------------------------------------------------------------
// Shared logical-region builders (author-facing input to the encoder).
// ---------------------------------------------------------------------------

/// A single-operand region over one op node `op(Bind(0))`, at the given upstream
/// version bindings. relu-style ops are non-commutative, so only the op-name /
/// dtype token bytes vary between two such regions — never operand order.
fn unary(op_name: &str, ops_v: &str, classify_v: &str) -> Region {
    Region::new(1, ops_v, classify_v, Node::op(op_name, vec![Node::Bind(0)]))
}

/// A positional `gather` region: root `gather` over `Bind(0)` (data) and
/// `Bind(1)` (index), carrying the given OpAttrs blob and per-operand roles.
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

/// The default `(data, *)` operand-role entry.
fn data_star() -> RoleEntry {
    RoleEntry::default_entry()
}
/// A `(role, dtype)` operand-role entry.
fn entry(role: &str, dtype: &str) -> RoleEntry {
    RoleEntry { role: role.into(), dtype: dtype.into() }
}

/// The in-band frozen-shape schema version stamped at wire bytes[4..6] (§6.8-0013),
/// read straight off the encoded region — the axis that MUST stay put while the
/// upstream version tokens vary.
fn wire_schema_version(wire: &[u8]) -> u16 {
    u16::from_le_bytes([wire[4], wire[5]])
}

/// Byte offset of a length-prefixed token (`u16-LE len` + UTF-8 bytes) on the wire.
fn token_at(wire: &[u8], s: &str) -> usize {
    let mut needle = (s.len() as u16).to_le_bytes().to_vec();
    needle.extend_from_slice(s.as_bytes());
    wire
        .windows(needle.len())
        .position(|w| w == needle.as_slice())
        .unwrap_or_else(|| panic!("token {s:?} not found on wire"))
}

/// The single root Op node of a decoded region (the last, ROOT-marked entry).
fn root_op(pr: &ParsedRegion) -> (&str, &[u8], &[RoleEntry], &[u32]) {
    match pr.nodes.last().expect("region has a root") {
        ParsedNode::Op { op_name, opattrs, roles, operands, .. } => {
            (op_name.as_str(), opattrs.as_slice(), roles.as_slice(), operands.as_slice())
        }
        _ => panic!("root must be an Op"),
    }
}

// ---------------------------------------------------------------------------
// 6.6-0007 — upstream version binding: byte-exact token axis, distinct from
//            the in-band frozen-shape schema version.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.6-0007 — a region carries `ops_version` and `classify_version`
/// as byte-exact UTF-8 tokens (§6.8-0003), compared byte-exact on the string; an
/// impl MUST NOT treat `"1"` and `"1.0"` as equal nor numerically normalize, and
/// these upstream tokens are a DISTINCT axis from the in-band frozen-shape schema
/// version stamped at bytes[4..6] (§6.8-0013). Teeth: (a) `ops_version="1"` vs
/// `"1.0"` produce different wire bytes and decode to distinct strings; (b) the
/// two version fields round-trip INDEPENDENTLY — swapping `(ops,classify)` from
/// `(2,1)` to `(1,2)` changes the bytes and each field survives verbatim; (c) the
/// frozen-shape slot stays `== SCHEMA_VERSION` regardless of the upstream tokens
/// (an impl that stamped the upstream version into that slot, or normalized the
/// tokens, diverges). Honesty: this backs the byte-exactness of the carried
/// tokens; resolving a token against the pinned version (the op-name oracle) is
/// the codec-absent limb left to §6.6-0004 (skipped).
#[test]
fn test_grammar_upstream_version_binding() {
    let _clause = "KISS-GRAMMAR-6.6-0007";

    // (a) "1" vs "1.0" are NOT numerically normalized: different bytes, distinct
    // decoded tokens.
    let w1 = encode(&unary("relu", "1", "1")).unwrap();
    let w10 = encode(&unary("relu", "1.0", "1")).unwrap();
    assert_ne!(w1, w10, "{_clause}: ops_version \"1\" and \"1.0\" produce different wire bytes");
    assert_eq!(decode(&w1).unwrap().ops_version, "1", "\"1\" round-trips verbatim");
    assert_eq!(decode(&w10).unwrap().ops_version, "1.0", "\"1.0\" round-trips verbatim");
    assert_ne!(
        decode(&w1).unwrap().ops_version,
        decode(&w10).unwrap().ops_version,
        "{_clause}: \"1\" != \"1.0\" as strings (no numeric normalization)"
    );

    // (b) ops_version and classify_version are independent fields: swapping them
    // changes the bytes, and each survives verbatim through a round-trip.
    let w_a = encode(&unary("relu", "2", "1")).unwrap();
    let w_b = encode(&unary("relu", "1", "2")).unwrap();
    assert_ne!(w_a, w_b, "{_clause}: swapping (ops,classify) changes the wire (fields not conflated)");
    let pa = decode(&w_a).unwrap();
    let pb = decode(&w_b).unwrap();
    assert_eq!((pa.ops_version.as_str(), pa.classify_version.as_str()), ("2", "1"),
        "ops=2/classify=1 round-trips independently");
    assert_eq!((pb.ops_version.as_str(), pb.classify_version.as_str()), ("1", "2"),
        "ops=1/classify=2 round-trips independently");

    // (c) the frozen-shape schema version (bytes[4..6]) is a SEPARATE axis: it
    // equals SCHEMA_VERSION for every upstream-token variant and never carries the
    // upstream version. With ops_version="2" the slot is still SCHEMA_VERSION (=1),
    // NOT 2 — an impl that stamped the upstream token there would read back 2.
    for w in [&w1, &w10, &w_a, &w_b] {
        assert_eq!(wire_schema_version(w), SCHEMA_VERSION,
            "{_clause}: in-band frozen-shape version stays SCHEMA_VERSION across upstream tokens");
    }
    assert_eq!(wire_schema_version(&w1), wire_schema_version(&w_a),
        "the frozen-shape slot is invariant to the ops_version token");
}

// ---------------------------------------------------------------------------
// 7.1-0003 — a non-expressible input or malformed wire yields a typed decline,
//            never a panic; expressible regions round-trip.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-7.1-0003 — an input that is not an expressible region (§6.4-0008)
/// yields a typed encoder `Decline`, and a malformed wire stream yields a typed
/// `DecodeDecline` — NEVER a panic — while an expressible region round-trips
/// cleanly. Teeth: a battery of non-expressible regions (strict-subset bind, bind
/// index >= n_inputs) and malformed streams (bad magic, unsupported schema
/// version, mid-stream truncation, undefined kind byte, invalid consumers byte,
/// forward operand reference, root-not-ROOT, trailing bytes) each return `Err(..)`
/// of a specific typed variant; reaching the final round-trip assert proves no
/// call panicked — replacing any `.ok_or(Truncated)?` with `.unwrap()` would
/// abort on the truncation case. Honesty: the "op absent from the pinned KISS-Ops
/// version" decline limb of §6.6-0004 needs the op-name oracle the codec
/// deliberately lacks and is out of scope here (§6.6-0004, skipped).
#[test]
fn test_grammar_typed_decline_never_panic() {
    let _clause = "KISS-GRAMMAR-7.1-0003";

    // --- non-expressible regions -> typed encoder Decline (never a panic). ---
    // Strict subset: declares 2 inputs but binds only input 0.
    match encode(&Region::new(2, "1", "1", Node::op("relu", vec![Node::Bind(0)]))) {
        Err(Decline::BindSetMismatch { declared: 2, unbound: 1 }) => {}
        other => panic!("{_clause}: strict-subset bind must decline BindSetMismatch, got {other:?}"),
    }
    // Bind index out of range: names input 5 with only 1 declared.
    match encode(&Region::new(1, "1", "1", Node::op("relu", vec![Node::Bind(5)]))) {
        Err(Decline::BindIndexOutOfRange { index: 5, n_inputs: 1 }) => {}
        other => panic!("{_clause}: out-of-range bind must decline BindIndexOutOfRange, got {other:?}"),
    }

    // --- malformed wire streams -> typed DecodeDecline (never a panic). ---
    // Base: G4 `a + a`, a single-op region. node0 = Bind(0), node1 = add (root).
    let g4 = Region::new(1, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Bind(0)]));
    let w = encode(&g4).unwrap();
    // Header length (magic 4 + schema 2 + n_inputs 4 + "1" token 3 + "1" token 3 +
    // node_count 4) = the offset of the first node's kind byte.
    let hdr = 4 + 2 + 4 + (2 + 1) + (2 + 1) + 4;

    // bad magic
    let mut bad_magic = w.clone();
    bad_magic[0] ^= 0xFF;
    assert!(matches!(decode(&bad_magic), Err(DecodeDecline::BadMagic)),
        "{_clause}: corrupt magic declines BadMagic");

    // unsupported schema version (bytes[4..6] -> 0xFFFF)
    let mut bad_ver = w.clone();
    bad_ver[4] = 0xFF;
    bad_ver[5] = 0xFF;
    assert!(matches!(decode(&bad_ver), Err(DecodeDecline::UnsupportedSchemaVersion { got: 0xFFFF })),
        "{_clause}: unknown schema version declines UnsupportedSchemaVersion");

    // truncated mid-stream (magic+schema only -> the n_inputs u32 read fails)
    assert!(matches!(decode(&w[..6]), Err(DecodeDecline::Truncated)),
        "{_clause}: header truncation declines Truncated");
    // truncated by one byte at the tail (the extract_count u16 read fails)
    assert!(matches!(decode(&w[..w.len() - 1]), Err(DecodeDecline::Truncated)),
        "{_clause}: tail truncation declines Truncated");

    // undefined node kind byte (first node's kind 0x00 -> 0x7F)
    let mut bad_kind = w.clone();
    assert_eq!(bad_kind[hdr], KIND_BIND, "sanity: header end is the first node's kind byte");
    bad_kind[hdr] = 0x7F;
    assert!(matches!(decode(&bad_kind), Err(DecodeDecline::BadKind { got: 0x7F })),
        "{_clause}: undefined kind byte declines BadKind");

    // invalid consumers enum value (root's consumers 0x01 -> 0x05)
    let cons_off = token_at(&w, "add") + 2 + "add".len();
    assert_eq!(w[cons_off], CONSUMERS_ROOT, "sanity: consumers byte follows the op_name token");
    let mut bad_cons = w.clone();
    bad_cons[cons_off] = 0x05;
    assert!(matches!(decode(&bad_cons), Err(DecodeDecline::BadConsumers { got: 0x05 })),
        "{_clause}: out-of-domain consumers declines BadConsumers");

    // root not ROOT (root's consumers 0x01 -> 0x00 INTERIOR)
    let mut not_root = w.clone();
    not_root[cons_off] = CONSUMERS_INTERIOR;
    assert!(matches!(decode(&not_root), Err(DecodeDecline::RootNotRoot)),
        "{_clause}: last node not ROOT declines RootNotRoot");

    // forward operand reference (root operand1, at wire[len-6..len-2], -> index 5)
    let mut fwd = w.clone();
    let opd = fwd.len() - 6;
    fwd[opd..opd + 4].copy_from_slice(&5u32.to_le_bytes());
    assert!(matches!(decode(&fwd), Err(DecodeDecline::ForwardOperandReference { node: 1, operand: 5 })),
        "{_clause}: operand referencing a not-strictly-earlier node declines ForwardOperandReference");

    // trailing bytes after a complete region
    let mut trailing = w.clone();
    trailing.push(0xAB);
    assert!(matches!(decode(&trailing), Err(DecodeDecline::TrailingBytes)),
        "{_clause}: extra bytes decline TrailingBytes");

    // --- an expressible region round-trips cleanly (Ok, not a decline). ---
    // Reaching here without a panic proves the reader is total over the battery.
    let g1 = Region::new(3, "1", "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]), Node::Bind(2)]));
    let gw = encode(&g1).expect("G1 is expressible");
    let pr = decode(&gw).expect("{_clause}: an expressible region round-trips Ok");
    assert_eq!(root_op(&pr).0, "add", "the round-tripped G1 root is add");
    assert_eq!(pr.n_inputs, 3, "G1 declares 3 inputs");
}

// ---------------------------------------------------------------------------
// 6.1-0003 — one op_name backs many tags, distinguished by identity-bearing
//            OpAttrs values and the operand-role tuple.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.1-0003 — a single KISS-Ops op name MAY back several
/// advertisable tags, distinguished by their identity-bearing attribute values
/// (the pattern-channel OpAttrs and the operand-role tuple); the tag is the FULL
/// `op_name` + those values, and an impl MUST NOT reduce it to the bare `op_name`.
/// Teeth: two `gather` nodes identical in name+structure but differing ONLY in the
/// OpAttrs blob (axis 0 vs 1) produce different `subtree_key`s and different wire
/// bytes; two differing ONLY in an operand-role dtype (`(index,u32)` vs
/// `(index,i32)`) also differ; two identical in name+opattrs+roles produce
/// identical keys and bytes. An impl that keyed tag identity on the bare `op_name`
/// would collapse the first two pairs — the `assert_ne!`s fail.
#[test]
fn test_grammar_tag_is_name_plus_attrs() {
    let _clause = "KISS-GRAMMAR-6.1-0003";

    // Real KISS-Ops OpAttrs `gather` payloads (referenced by content), differing
    // only in the axis field.
    let attrs_ax0 = opattrs::encode_gather(0, opattrs::OobPolicy::ZeroFill, 1, opattrs::IndexDtype::U32);
    let attrs_ax1 = opattrs::encode_gather(1, opattrs::OobPolicy::ZeroFill, 1, opattrs::IndexDtype::U32);
    assert_ne!(attrs_ax0, attrs_ax1, "sanity: the two Ops blobs differ (axis 0 vs 1)");

    // Pair 1: same op_name+structure+roles, DIFFERENT identity-bearing OpAttrs.
    let node_ax0 = Node::Op { op_name: "gather".into(), opattrs: attrs_ax0.clone(), roles: vec![], operands: vec![Node::Bind(0), Node::Bind(1)] };
    let node_ax1 = Node::Op { op_name: "gather".into(), opattrs: attrs_ax1.clone(), roles: vec![], operands: vec![Node::Bind(0), Node::Bind(1)] };
    assert_ne!(subtree_key(&node_ax0), subtree_key(&node_ax1),
        "{_clause}: tags over one op_name differ by their OpAttrs values");
    assert_ne!(
        encode(&gather_region(attrs_ax0.clone(), vec![])).unwrap(),
        encode(&gather_region(attrs_ax1.clone(), vec![])).unwrap(),
        "the OpAttrs difference reaches the region wire form");

    // Pair 2: same op_name+opattrs+structure, DIFFERENT operand-role dtype token.
    let roles_u32 = vec![data_star(), entry("index", "u32")];
    let roles_i32 = vec![data_star(), entry("index", "i32")];
    let node_u32 = Node::Op { op_name: "gather".into(), opattrs: vec![], roles: roles_u32.clone(), operands: vec![Node::Bind(0), Node::Bind(1)] };
    let node_i32 = Node::Op { op_name: "gather".into(), opattrs: vec![], roles: roles_i32.clone(), operands: vec![Node::Bind(0), Node::Bind(1)] };
    assert_ne!(subtree_key(&node_u32), subtree_key(&node_i32),
        "{_clause}: tags over one op_name differ by their operand-role tuple");
    assert_ne!(
        encode(&gather_region(vec![], roles_u32.clone())).unwrap(),
        encode(&gather_region(vec![], roles_i32)).unwrap(),
        "the operand-role difference reaches the region wire form");

    // Control: two independently-built tags identical in name+opattrs+roles carry
    // the SAME identity (the tag is a pure function of those fields — nothing
    // else, e.g. authoring-object identity, distinguishes them).
    let node_u32_again = Node::Op { op_name: "gather".into(), opattrs: vec![], roles: roles_u32, operands: vec![Node::Bind(0), Node::Bind(1)] };
    assert_eq!(subtree_key(&node_u32), subtree_key(&node_u32_again),
        "identical name+opattrs+roles -> identical tag key");
}

// ---------------------------------------------------------------------------
// 6.6-0001 — op_name spelled exactly as the KISS-Ops token; no synonym/case fold.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.6-0001 — every `op_name` is spelled EXACTLY as the KISS-Ops op
/// token (case-sensitive); an impl MUST NOT accept a synonym, alias, or alternate
/// casing. Using a single-operand non-commutative op so only the name token
/// varies. Teeth: `encode(relu)` and `encode("Relu")` differ in bytes and each
/// decodes to its exact case; a synonym (`activate`) is carried as its own token,
/// never folded onto `relu`; the round-trip reproduces the name byte-for-byte. A
/// case-normalizing or synonym-aliasing encoder collapses `relu`/`Relu` (the
/// `assert_ne!` fails) or returns a normalized spelling on decode.
#[test]
fn test_grammar_op_name_spelling() {
    let _clause = "KISS-GRAMMAR-6.6-0001";

    let w_lower = encode(&unary("relu", "1", "1")).unwrap();
    let w_upper = encode(&unary("Relu", "1", "1")).unwrap();
    let w_syn = encode(&unary("activate", "1", "1")).unwrap();

    // Case is load-bearing: `relu` and `Relu` are distinct on the wire.
    assert_ne!(w_lower, w_upper, "{_clause}: op_name is case-sensitive on the wire");
    // A synonym is a different token, never aliased to `relu`.
    assert_ne!(w_lower, w_syn, "{_clause}: a synonym is carried as a distinct token, not aliased");

    // Each decodes to its EXACT spelling (no normalization on the read path).
    assert_eq!(root_op(&decode(&w_lower).unwrap()).0, "relu", "relu round-trips byte-for-byte");
    assert_eq!(root_op(&decode(&w_upper).unwrap()).0, "Relu", "{_clause}: Relu preserves its case");
    assert_eq!(root_op(&decode(&w_syn).unwrap()).0, "activate", "the synonym token survives verbatim");
    assert_ne!(
        root_op(&decode(&w_lower).unwrap()).0,
        root_op(&decode(&w_upper).unwrap()).0,
        "decoded relu != decoded Relu");
}

// ---------------------------------------------------------------------------
// 6.6-0002 — operand dtype role spelled exactly as a Classify token or wildcard.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.6-0002 — every dtype role in an `operand_role_tuple` is spelled
/// EXACTLY as a KISS-Classify dtype token (`u8`, `u32`, `f32`, …) or as the
/// dtype-role wildcard `*` (§6.6-0006); an impl MUST NOT substitute a synonym or
/// alternate casing, nor spell a polymorphic role as anything but the pinned
/// wildcard. Teeth: a `gather` role with dtype `u32` vs `U32` produces different
/// wire bytes and decodes to distinct `RoleEntry.dtype` (no case fold); a real
/// dtype `f32` is distinct from the wildcard `*` on the wire and after decode
/// (they are not conflated). A dtype-token case-normalizer collapses `u32`/`U32`;
/// an impl that treated a dtype token as the wildcard collapses `f32`/`*`.
#[test]
fn test_grammar_operand_dtype_token_spelling() {
    let _clause = "KISS-GRAMMAR-6.6-0002";
    assert_eq!(WILDCARD, "*", "the dtype-role wildcard is spelled '*'");
    assert_ne!("f32", WILDCARD, "sanity: a real dtype token is not the wildcard spelling");

    // Casing: operand1 dtype `u32` vs `U32` (role fixed = index, so both entries
    // are non-default and the dtype token is the only varying byte run).
    let w_u32 = encode(&gather_region(vec![], vec![data_star(), entry("index", "u32")])).unwrap();
    let w_upper_dt = encode(&gather_region(vec![], vec![data_star(), entry("index", "U32")])).unwrap();
    assert_ne!(w_u32, w_upper_dt, "{_clause}: dtype token is case-sensitive on the wire");
    assert_eq!(root_op(&decode(&w_u32).unwrap()).2[1].dtype, "u32", "u32 round-trips verbatim");
    assert_eq!(root_op(&decode(&w_upper_dt).unwrap()).2[1].dtype, "U32", "{_clause}: U32 preserves its case");

    // A real dtype token vs the wildcard: `f32` and `*` are distinct.
    let w_f32 = encode(&gather_region(vec![], vec![data_star(), entry("index", "f32")])).unwrap();
    let w_star = encode(&gather_region(vec![], vec![data_star(), entry("index", "*")])).unwrap();
    assert_ne!(w_f32, w_star, "{_clause}: a dtype token is not conflated with the wildcard");
    let d_f32 = root_op(&decode(&w_f32).unwrap()).2[1].dtype.clone();
    let d_star = root_op(&decode(&w_star).unwrap()).2[1].dtype.clone();
    assert_eq!(d_f32, "f32", "f32 dtype token round-trips verbatim");
    assert_eq!(d_star, "*", "the wildcard round-trips as '*'");
    assert_ne!(d_f32, d_star, "decoded f32 != decoded wildcard");
}

// ---------------------------------------------------------------------------
// 7.1-0002 — the advertisable op-name set is open, deferred to KISS-Ops: any
//            op_name re-bases without a Grammar schema change or allow-list.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-7.1-0002 — the advertisable op-name set is a growable vocabulary
/// deferred entirely to KISS-Ops; the codec MUST accept any `op_name`
/// (re-basing-by-name) with no Grammar-local allow-list, and op addition is
/// additive — it does NOT bump the frozen-shape schema version. Teeth: a NOVEL
/// op_name absent from any Grammar literal (`vendor_new_op`) encodes and
/// round-trips byte-exact, and its wire stamps the SAME frozen-shape
/// `SCHEMA_VERSION` as a known-op (`relu`) region. An impl with a hardcoded
/// allow-list would return `Err` on the novel name (this test's `unwrap` would
/// fail); one that bumped the schema on an unknown op would make the novel
/// region's bytes[4..6] differ from the known region's.
#[test]
fn test_grammar_op_name_set_is_open_via_ops() {
    let _clause = "KISS-GRAMMAR-7.1-0002";

    // A novel op absent from every Grammar literal encodes with no allow-list veto.
    let novel = encode(&unary("vendor_new_op", "1", "1"))
        .expect("{_clause}: a novel op_name must not be withheld by a Grammar-local allow-list");
    // It round-trips byte-exact: the name survives verbatim and re-encoding is stable.
    assert_eq!(root_op(&decode(&novel).unwrap()).0, "vendor_new_op",
        "{_clause}: the novel op_name round-trips byte-for-byte");
    assert_eq!(encode(&unary("vendor_new_op", "1", "1")).unwrap(), novel,
        "encoding the novel-op region is deterministic");

    // Op addition is additive: the novel region stamps the unchanged frozen-shape
    // version — identical to a known-op region's slot, and == SCHEMA_VERSION.
    let known = encode(&unary("relu", "1", "1")).unwrap();
    assert_eq!(wire_schema_version(&novel), SCHEMA_VERSION,
        "{_clause}: a novel op does not bump the frozen-shape schema version");
    assert_eq!(wire_schema_version(&novel), wire_schema_version(&known),
        "novel-op and known-op regions carry the same frozen-shape version");
}

// ---------------------------------------------------------------------------
// 6.2-0004 — the OOB read policy is the KISS-Ops-owned set {skip, clamp,
//            zero-fill}; Grammar carries the ordinal verbatim, invents nothing.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.2-0004 — the out-of-bounds read policy surfaced on
/// `pattern_attrs` is exactly the KISS-Ops-owned set `{skip, clamp, zero-fill}`,
/// spelled as KISS-Ops spells it; KISS-Grammar defines NO alternative OOB
/// vocabulary and carries the ordinal uninterpreted. Teeth: for each of the three
/// Ops `OobPolicy` ordinals, a `gather` OpAttrs blob carrying it round-trips
/// through the Grammar codec BYTE-EXACT (Grammar never remaps/normalizes the oob
/// byte) and the Ops validator `decode_gather` recovers the SAME policy —
/// establishing Ops ownership; the reserved ordinal `0` and an out-of-set ordinal
/// are carried verbatim by Grammar yet DECLINED by the Ops validator, so no
/// Grammar-local vocabulary admits them. A Grammar-side remap (e.g. `zero-fill`→
/// `clamp`) makes the round-trip blob differ from the input (assert fails) and the
/// Ops validator recover the wrong policy. (`opattrs` is referenced by content
/// only — never by clause id.)
#[test]
fn test_grammar_oob_policy_from_ops() {
    let _clause = "KISS-GRAMMAR-6.2-0004";

    for policy in [opattrs::OobPolicy::Skip, opattrs::OobPolicy::Clamp, opattrs::OobPolicy::ZeroFill] {
        let blob = opattrs::encode_gather(0, policy, 1, opattrs::IndexDtype::U32);
        // Grammar carries the Ops oob byte verbatim through a full round-trip.
        let pr = decode(&encode(&gather_region(blob.clone(), vec![])).unwrap()).unwrap();
        let carried = root_op(&pr).1.to_vec();
        assert_eq!(carried, blob,
            "{_clause}: Grammar carries the oob ordinal byte-exact (no remap/normalization)");
        // Ops — the owner — recovers the SAME policy from the carried blob.
        let (_, recovered, _, _) = opattrs::decode_gather(&carried)
            .expect("Ops validates its own oob vocabulary");
        assert_eq!(recovered, policy,
            "{_clause}: the Ops validator recovers the identical policy (Ops owns the vocabulary)");
    }

    // Reserved ordinal 0 and an out-of-set ordinal: Grammar admits the raw byte
    // (no Grammar-local set filters it), but the Ops validator declines — proving
    // the OOB vocabulary is Ops-owned, not Grammar-owned.
    for bad in [[0u8, 0, 1, 1], [0u8, 7, 1, 1]] {
        let pr = decode(&encode(&gather_region(bad.to_vec(), vec![])).unwrap()).unwrap();
        assert_eq!(root_op(&pr).1, &bad, "Grammar carries even an Ops-invalid oob byte verbatim");
        assert!(opattrs::decode_gather(&bad).is_err(),
            "{_clause}: the Ops validator declines an out-of-set oob ordinal");
    }
}

// ---------------------------------------------------------------------------
// 6.0-0001 — the determinism class is exact-byte: deterministic output, order-
//            invariant input, no-tolerance discrimination.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.0-0001 — the region wire form is determinism-class EXACT BYTE
/// COMPARE, evaluated with a byte-exact comparator (no tolerance, no
/// order-invariance on the OUTPUT). Teeth: `encode(G1)` is byte-IDENTICAL across
/// repeated calls (output order is the post-order record `Vec`, not a HashMap
/// iteration — a HashMap-order encoder would be nondeterministic and this
/// repeatability assert would fail); two order-different authorings of the same
/// region converge to identical bytes (order-invariant INPUT → exact-byte
/// OUTPUT); and two genuinely different regions differ by ≥1 byte under the
/// byte-exact comparator (no tolerance would ever call them equal). Honesty: this
/// backs the exact-byte / repeatability limb; the definitional "numeric
/// determinism class owned by KISS-Ops, not re-forked" limb has no codec behaviour
/// to exercise and is out of scope. (The codec exposes no ParsedRegion→wire
/// re-encoder, so byte-stability is attested by repeated encode + a round-trip
/// decode rather than a literal decode-then-re-encode.)
#[test]
fn test_grammar_determinism_class_exact_byte() {
    let _clause = "KISS-GRAMMAR-6.0-0001";
    let g1 = || Region::new(3, "1", "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]), Node::Bind(2)]));

    // Deterministic output: repeated encodings are byte-identical (a HashMap-
    // iteration-order encoder would diverge across these independent calls).
    let first = encode(&g1()).unwrap();
    for _ in 0..8 {
        assert_eq!(encode(&g1()).unwrap(), first,
            "{_clause}: encode is deterministic — repeated calls are byte-identical");
    }
    // The wire round-trips (the determinism is over the real, decodable wire form).
    assert_eq!(root_op(&decode(&first).unwrap()).0, "add", "the deterministic wire decodes back to G1");

    // Order-invariant INPUT -> exact-byte OUTPUT: two authorings of G1 differing
    // only in commutative operand order converge to identical bytes.
    let auth_a = Region::new(3, "1", "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]), Node::Bind(2)]));
    let auth_b = Region::new(3, "1", "1",
        Node::op("add", vec![Node::Bind(2), Node::op("mul", vec![Node::Bind(1), Node::Bind(0)])]));
    assert_eq!(encode(&auth_a).unwrap(), encode(&auth_b).unwrap(),
        "{_clause}: order-different authorings converge to identical bytes");

    // No tolerance: two genuinely different regions differ by >=1 byte under the
    // byte-exact comparator.
    let other = encode(&Region::new(1, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Bind(0)]))).unwrap();
    assert_ne!(first, other, "{_clause}: distinct regions differ under a byte-exact comparator");
    let diffs = first.iter().zip(other.iter()).filter(|(a, b)| a != b).count()
        + first.len().abs_diff(other.len());
    assert!(diffs >= 1, "at least one byte distinguishes distinct regions (no tolerance)");
}
