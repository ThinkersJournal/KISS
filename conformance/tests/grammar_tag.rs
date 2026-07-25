//! KISS-Conform batch-3 vectors for the KISS-Grammar advertisable-op **tag**
//! canonical serialization (Grammar §6.8-0012 / §6.1-0007) — the third distinct
//! wire artifact after the region wire form (`grammar_golden.rs`) and the canonical
//! subtree key (`grammar_canonical.rs`) — plus the channel-separation and
//! single-output composition rules (§6.2-0002, §6.3-0002, §6.3-0003, §6.4-0009).
//!
//! Every test drives the EXISTING reference codec in `kiss_conformance::grammar`:
//! `encode` / `decode` / `encode_tag` / `encode_operand_roles` and the typed
//! `Decline` / `DecodeDecline`s. No lifter, matcher, or op-name oracle is invented
//! here. Two clauses rest on a small, spec-pinned addition to the reference codec:
//! `grammar::encode_tag` composes the EXISTING primitives (`push_token` +
//! `u16`-LE OpAttrs length + `encode_operand_roles`) into the §6.8-0012 tag byte
//! order, and `grammar::decode` now counts ROOT-marked nodes and declines a
//! multi-output forest (§6.4-0009, `DecodeDecline::MultipleRoots`) — both are the
//! real reference codec the spec pins, not test-local reimplementations.
//!
//! Where a KISS-Ops OpAttrs payload is needed it is produced by
//! `kiss_conformance::opattrs`, referenced **by content** (its `encode_gather`
//! function and `OobPolicy` / `IndexDtype` types) and **never by clause id**, so
//! the Grammar codec's uninterpreted carriage of an Ops-owned sub-vocabulary is
//! attested without reverse-binding any KISS-Ops clause (the batch-2 precedent).
//!
//! Each `#[test]` is named exactly per the spec *Test:* tag, cites its single
//! KISS-Grammar clause id, and proves that id by MUTATION — a plausible codec/spec
//! drift makes the test fail. No substring greps, no `assert!(true)`, no
//! self-compares, no set-vs-hardcoded-copy, no binding to a derived `PartialEq`.

use kiss_conformance::grammar::*;
use kiss_conformance::{assert_golden, opattrs};

// ---------------------------------------------------------------------------
// Shared builders (author-facing input to the encoder).
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
/// A non-default `(index, u32)` operand-role entry — the KISS-Ops index-operand
/// example (`gather`'s `u32` index operand).
fn index_u32() -> RoleEntry {
    RoleEntry { role: "index".into(), dtype: "u32".into() }
}

/// The single root Op node of a decoded region (the last, ROOT-marked entry).
fn root_op(pr: &ParsedRegion) -> (String, Vec<u8>, Vec<RoleEntry>, Vec<u32>, u8) {
    match pr.nodes.last().expect("region has a root") {
        ParsedNode::Op { op_name, opattrs, roles, operands, consumers } => {
            (op_name.clone(), opattrs.clone(), roles.clone(), operands.clone(), *consumers)
        }
        _ => panic!("root must be an Op"),
    }
}

// ---------------------------------------------------------------------------
// 6.8-0012 — the tag canonical serialization: identity-bearing fields only, in
//            the pinned order, with NONE of the region framing.
// ---------------------------------------------------------------------------

/// The pinned tag bytes for the load-bearing `gather` tag of §2.4 / Appendix A.1
/// G3: `op_name` "gather", OpAttrs blob `[axis=1, oob=clamp, idx_operand=1,
/// idx_dtype=u32]` = `01 02 01 01`, operand-role tuple `[(data,*),(index,u32)]`
/// (k=2). Left-to-right, exactly per §6.8-0012 order (op_name | OpAttrs sub-block |
/// operand-role tuple) and NOTHING else.
const GATHER_TAG_GOLDEN: &str = concat!(
    "06 00 67 61 74 68 65 72 ",             // (1) op_name = "gather"
    "04 00 01 02 01 01 ",                    // (2) OpAttrs len 4 + blob [1,2,1,1]
    "02 00 ",                                // (3) operand-role tuple: k = 2
    "04 00 64 61 74 61 01 00 2A ",           //     entry0 = (data, *)
    "05 00 69 6E 64 65 78 03 00 75 33 32",   //     entry1 = (index, u32)
);

/// KISS-GRAMMAR-6.8-0012 — an advertisable-op tag has a dedicated canonical
/// serialization: `op_name` (length-prefixed) + OpAttrs sub-block (length-prefixed,
/// verbatim KISS-Ops bytes) + operand-role tuple (canonical form), and it MUST NOT
/// include the `kind` tag, `consumers`, `operand_count`, operand indices,
/// `n_inputs`, the `ops_version`/`classify_version` tokens, the magic/schema-version
/// header, or any `extract` record. Teeth: the tag bytes for a known `gather` tag
/// equal EXACTLY the three pinned fields and nothing else (a byte-exact golden); the
/// tag carries NONE of the `KGRM` region framing; and the byte immediately after the
/// `op_name` token is the OpAttrs length prefix (0x04), NOT a `consumers` byte —
/// a node record inserts `consumers` there (§6.8-0010), the tag does not. Mutation:
/// an `encode_tag` that leaked any region-framing field (the consumers byte, the
/// KGRM header, operand_count/indices, version tokens) or reordered the three fields
/// diverges from the pinned slice and fails.
#[test]
fn test_grammar_tag_canonical_serialization() {
    let _clause = "KISS-GRAMMAR-6.8-0012";
    // Real KISS-Ops OpAttrs `gather` payload (referenced by content, not clause id).
    let opattrs = opattrs::encode_gather(1, opattrs::OobPolicy::Clamp, 1, opattrs::IndexDtype::U32);
    assert_eq!(opattrs, vec![1u8, 2, 1, 1], "sanity: the Ops gather blob is [axis,oob,idx_operand,idx_dtype]");

    let tag = encode_tag("gather", &opattrs, &[data_star(), index_u32()], 2);

    // (1) exact golden: the three identity-bearing fields, in order, and nothing else.
    assert_golden(_clause, "gather-tag", &tag, GATHER_TAG_GOLDEN);
    assert_eq!(tag.len(), 37, "the tag is exactly the three fields (8 + 6 + 23 bytes)");

    // (2) NONE of the region framing: no KGRM magic anywhere in the tag.
    assert!(!tag.windows(4).any(|w| w == MAGIC),
        "{_clause}: the tag carries no region magic/schema header");

    // (3) the byte after the 8-byte op_name token is the OpAttrs length prefix, NOT
    // a consumers byte — a node record (§6.8-0010) inserts consumers between op_name
    // and the OpAttrs sub-block; the tag omits it.
    let after_name = 2 + "gather".len(); // u16 len + name = 8
    assert_eq!(tag[after_name], 0x04,
        "{_clause}: op_name is immediately followed by the OpAttrs length (no consumers leak)");
    assert_ne!(tag[after_name], CONSUMERS_ROOT, "no ROOT consumers byte leaked into the tag");
    assert_ne!(tag[after_name], CONSUMERS_INTERIOR, "no INTERIOR consumers byte leaked into the tag");

    // Contrast: the REGION wire of the same gather node DOES carry the framing the
    // tag omits — the KGRM magic and the version tokens — proving the tag is the
    // stripped, identity-only form and not just the region bytes.
    let wire = encode(&gather_region(opattrs.clone(), vec![data_star(), index_u32()])).unwrap();
    assert!(wire.windows(4).any(|w| w == MAGIC), "the region wire carries KGRM magic");
    // The version token run "01 00 31" (ops_version="1") is in the region but not the tag.
    assert!(wire.windows(3).any(|w| w == [0x01, 0x00, 0x31]), "the region wire carries a version token");
    assert!(!tag.windows(3).any(|w| w == [0x01, 0x00, 0x31]),
        "{_clause}: the tag carries no ops_version/classify_version token");
    assert!(tag.len() < wire.len(), "the tag is the stripped identity form, shorter than the framed region");
}

// ---------------------------------------------------------------------------
// 6.1-0007 — the tag's canonical normal form used for tag equality.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.1-0007 — an advertisable-op tag has a single canonical normal
/// form for equality (aggregate a–c): (a) the identity-bearing attributes are
/// exactly the OpAttrs values + the operand-role tuple + `op_name`, and
/// `synthesis_attrs` are NOT in the tag (they are not an input to `encode_tag` at
/// all); (b) every identity-bearing attribute is carried at its resolved value with
/// defaults made EXPLICIT (never elided); (c) the tuple is in canonical form
/// (trailing `(data,*)` defaults omitted) and tags compare by byte-exact comparison.
/// Teeth: (b) a tag whose axis resolves to the default 0 still carries the 4-byte
/// OpAttrs blob — it is NOT collapsed to an empty-OpAttrs tag (an eliding impl makes
/// them equal); axis 0 vs axis 1 tags DIFFER (identity-bearing values distinguish);
/// (c) `tag[(index,u32)] == tag[(index,u32),(data,*)]` and `tag[] ==
/// tag[(data,*),(data,*)]` (trailing default omission), while a NON-trailing role
/// difference is NOT omitted and stays distinct. Mutation: treating an
/// elided-vs-explicit trailing default as distinct, eliding the explicit default
/// attr, or calling two different identity-bearing values equal, flips an assert.
#[test]
fn test_grammar_canonical_tag_normal_form() {
    let _clause = "KISS-GRAMMAR-6.1-0007";
    let attrs0 = opattrs::encode_gather(0, opattrs::OobPolicy::Clamp, 1, opattrs::IndexDtype::U32);
    let attrs1 = opattrs::encode_gather(1, opattrs::OobPolicy::Clamp, 1, opattrs::IndexDtype::U32);
    assert_ne!(attrs0, attrs1, "sanity: the two Ops blobs differ (axis 0 vs 1)");
    let roles = [data_star(), index_u32()];

    // (b) defaults made EXPLICIT, not elided: a tag whose axis resolves to the
    // default 0 still carries the full 4-byte OpAttrs blob (the upstream KISS-Ops
    // encoding emits the defaulted axis explicitly). It is NOT the same as a bare
    // tag with an empty OpAttrs block — an impl that elided the default would
    // collapse these two.
    let tag_default0 = encode_tag("gather", &attrs0, &roles, 2);
    let tag_no_attrs = encode_tag("gather", &[], &roles, 2);
    assert_ne!(tag_default0, tag_no_attrs,
        "{_clause}: the default axis-0 attr is carried explicit, not elided to empty");

    // (a)/(b) identity-bearing values distinguish: axis 0 and axis 1 are distinct
    // tag identities (a cache-hit-vs-miss decision that must not diverge).
    let tag_axis1 = encode_tag("gather", &attrs1, &roles, 2);
    assert_ne!(tag_default0, tag_axis1, "{_clause}: axis 0 and axis 1 are distinct tag identities");

    // (c) trailing-default omission: an explicit trailing (data,*) is omitted, so
    // these authorings canonicalize to the SAME tag bytes.
    let t_k1_omitted = encode_tag("gather", &[], &[index_u32()], 2);
    let t_k1_explicit = encode_tag("gather", &[], &[index_u32(), data_star()], 2);
    assert_eq!(t_k1_omitted, t_k1_explicit,
        "{_clause}: a trailing (data,*) default is omitted -> identical tag");
    // all-default roles collapse to k=0 regardless of how they were authored.
    let t_empty = encode_tag("gather", &[], &[], 2);
    let t_all_default = encode_tag("gather", &[], &[data_star(), data_star()], 2);
    assert_eq!(t_empty, t_all_default,
        "{_clause}: all-default operand roles omit to k=0 -> identical tag");

    // (c) position matters: a NON-trailing (leading) non-default is NOT omitted, so a
    // role at operand 0 vs operand 1 stays distinct (the omission is trailing-only,
    // never a wholesale default drop).
    let t_lead = encode_tag("gather", &[], &[data_star(), index_u32()], 2);
    assert_ne!(t_k1_omitted, t_lead,
        "{_clause}: a non-trailing role difference is not omitted -> distinct tag");
    assert_ne!(t_empty, t_lead, "a present non-default role differs from the all-default tag");
}

// ---------------------------------------------------------------------------
// 6.4-0009 — a region has exactly one root (single-output); a multi-output
//            forest yields a typed decline.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.4-0009 — a region MUST have exactly one root (be single-output);
/// a multi-output forest is not an expressible region and MUST yield a typed
/// decline, never a panic. Teeth: encode side — a `Region` holds exactly one
/// `root: Node`, so a normal region round-trips with exactly one ROOT-marked node.
/// Decode side — take a valid G1 wire and flip the interior `mul` consumers byte
/// 0x00 -> 0x01 (a SECOND ROOT-marked node = a 2-output forest); decode MUST return
/// `DecodeDecline::MultipleRoots`, never a panic, EVEN THOUGH the last node is still
/// ROOT (so the last-node-is-ROOT guard is satisfied). Mutation: a decoder that
/// only checks `last == ROOT` would ADMIT the two-ROOT forest — the added
/// count-ROOT check is exactly what declines it.
#[test]
fn test_grammar_single_output_region() {
    let _clause = "KISS-GRAMMAR-6.4-0009";

    // Encode side: a normal region is structurally single-output — exactly one node
    // carries consumers = ROOT after a round-trip.
    let wire = encode(&g1_region()).unwrap();
    let pr = decode(&wire).unwrap();
    let roots = pr.nodes.iter()
        .filter(|n| matches!(n, ParsedNode::Op { consumers, .. } if *consumers == CONSUMERS_ROOT))
        .count();
    assert_eq!(roots, 1, "{_clause}: an expressible region has exactly one root");
    assert_eq!(root_op(&pr).4, CONSUMERS_ROOT, "the single root is the last node");

    // Decode side: forge a second root. Flip the interior `mul` consumers 0x00->0x01.
    let mut forest = wire.clone();
    let mul_at = forest.windows(5).position(|w| w == [0x03, 0x00, 0x6D, 0x75, 0x6C]).unwrap();
    assert_eq!(forest[mul_at + 5], CONSUMERS_INTERIOR, "sanity: mul is INTERIOR before the mutation");
    forest[mul_at + 5] = CONSUMERS_ROOT; // now mul AND add both claim ROOT

    // The last node (add) is STILL ROOT, so the last-node-is-ROOT guard passes; only
    // the multi-root count catches this. It MUST decline, never panic or admit.
    let add_at = forest.windows(5).position(|w| w == [0x03, 0x00, 0x61, 0x64, 0x64]).unwrap();
    assert_eq!(forest[add_at + 5], CONSUMERS_ROOT, "sanity: the last node (add) is still ROOT");
    assert_eq!(decode(&forest), Err(DecodeDecline::MultipleRoots { count: 2 }),
        "{_clause}: a two-ROOT forest declines MultipleRoots (not RootNotRoot, not admitted)");

    // A region with ZERO roots is a different, pre-existing decline — this test's
    // count check is specifically the >1 case.
    let mut headless = wire.clone();
    headless[add_at + 5] = CONSUMERS_INTERIOR;
    assert_eq!(decode(&headless), Err(DecodeDecline::RootNotRoot),
        "the zero-root case stays RootNotRoot (distinct from the multi-root decline)");
}

// ---------------------------------------------------------------------------
// 6.2-0002 — a load-bearing pattern-attribute value is carried (guarded), never
//            see-through.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.2-0002 — a pattern attribute whose VALUE is load-bearing for
/// correctness (a `gather`'s axis) MUST be matched with an explicit guard; an impl
/// MUST NOT skip it (no see-through), because a region that dropped the value would
/// match a structurally-similar op with a different value — a wrong bind. Teeth:
/// two `gather` regions identical except the load-bearing OpAttrs axis
/// (`encode_gather(0,…)` vs `encode_gather(1,…)`) produce DIFFERENT region wire
/// bytes AND different tag bytes, and each axis survives a round-trip verbatim — the
/// value is carried, byte-distinguishable, so a guard can read it. Mutation: a codec
/// that dropped or zeroed the per-node OpAttrs blob (see-through) would collapse the
/// two axes to identical bytes and every `assert_ne!` here fails (the wrong bind:
/// matching any gather regardless of axis).
#[test]
fn test_grammar_load_bearing_attr_guarded() {
    let _clause = "KISS-GRAMMAR-6.2-0002";
    let attrs0 = opattrs::encode_gather(0, opattrs::OobPolicy::Clamp, 1, opattrs::IndexDtype::U32);
    let attrs1 = opattrs::encode_gather(1, opattrs::OobPolicy::Clamp, 1, opattrs::IndexDtype::U32);
    let roles = vec![data_star(), index_u32()];

    // Region wire: the axis is load-bearing on the wire — two axes -> different bytes.
    let w0 = encode(&gather_region(attrs0.clone(), roles.clone())).unwrap();
    let w1 = encode(&gather_region(attrs1.clone(), roles.clone())).unwrap();
    assert_ne!(w0, w1, "{_clause}: the load-bearing axis reaches the region wire (not see-through)");

    // Tag: the axis is identity-bearing — two axes -> different tag identity.
    let t0 = encode_tag("gather", &attrs0, &roles, 2);
    let t1 = encode_tag("gather", &attrs1, &roles, 2);
    assert_ne!(t0, t1, "{_clause}: the load-bearing axis reaches the tag identity");

    // Each axis survives a round-trip verbatim — the value is CARRIED, so a consumer
    // can guard on it rather than see through it.
    let carried0 = root_op(&decode(&w0).unwrap()).1;
    let carried1 = root_op(&decode(&w1).unwrap()).1;
    assert_eq!(carried0, attrs0, "axis-0 blob carried byte-for-byte");
    assert_eq!(carried1, attrs1, "axis-1 blob carried byte-for-byte");
    assert_ne!(carried0, carried1, "{_clause}: the two axes stay distinguishable after decode");
    // The differing byte is the axis field itself (blob[0]): 0 vs 1.
    assert_eq!((carried0[0], carried1[0]), (0, 1), "the axis value itself is on the wire, guardable");
}

// ---------------------------------------------------------------------------
// 6.3-0003 — a scalar runtime param rides the extract channel, not an op-graph
//            node (synthesis-channel routing, distinct from a graph node).
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.3-0003 — scalar-runtime-param routing carried on
/// `synthesis_attrs` MUST use the root-anchored extract channel and MUST NOT be
/// encoded as an additional op-graph node. Teeth: routing a scalar via an Extract
/// record keeps the op-graph node table BYTE-INVARIANT (the whole region prefix up
/// to `extract_count` is unchanged; node_count stays 5) while `extract_count` goes
/// 0 -> 1, and the extract is root-anchored (path is canonical operand indices from
/// the root). To show node_count is genuinely a signal (not always 5), a
/// materialized-as-node variant reaches node_count 6. Mutation: an impl that lowered
/// the scalar into a Bind/Op node bumps node_count 5 -> 6 and perturbs the node-table
/// prefix — the invariance asserts fail. (Overlaps the backed 6.4-0007 headline
/// no-extra-node property; distinct emphasis: the scalar is a synthesis-channel
/// param that materializes NOTHING in the op graph — proven here by byte-invariance
/// of the node-table prefix, not just the decoded node count. The shared-target
/// lex-least-path derivation of §6.4-0007 is out of this codec's scope.)
#[test]
fn test_grammar_synthesis_scalar_param_extract() {
    let _clause = "KISS-GRAMMAR-6.3-0003";

    // Base G1: 5 nodes, 0 extracts.
    let base = encode(&g1_region()).unwrap();
    let pb = decode(&base).unwrap();
    assert_eq!(pb.nodes.len(), 5, "G1 node_count is 5");
    assert_eq!(pb.extracts.len(), 0, "G1 has no extracts");

    // Route ONE scalar via a root-anchored extract (path [1] = root's operand-1 = mul).
    let mut r = g1_region();
    r.extracts = vec![Extract { path: vec![1], param_slot: 0 }];
    let withx = encode(&r).unwrap();
    let pw = decode(&withx).unwrap();

    // The op-graph node table is BYTE-INVARIANT: the scalar materialized no node.
    // (base = <node table> + extract_count 0x0000; withx = the SAME <node table> +
    // extract_count 1 + the record. The whole node-table prefix must be identical.)
    let node_table_prefix = &base[..base.len() - 2]; // strip the trailing 00 00 extract_count
    assert_eq!(&withx[..node_table_prefix.len()], node_table_prefix,
        "{_clause}: routing a scalar adds NO node — the op-graph byte prefix is unchanged");
    assert_eq!(pw.nodes.len(), 5, "{_clause}: node_count stays 5 (scalar rides the extract channel)");
    assert_eq!(pw.nodes, pb.nodes, "the decoded op-graph node table is invariant to the scalar param");
    assert_eq!(pw.extracts.len(), 1, "the scalar added exactly one tail extract record (0 -> 1)");
    // Root-anchored: the path is operand indices from the root.
    assert_eq!(pw.extracts[0], Extract { path: vec![1], param_slot: 0 }, "the extract is root-anchored");

    // node_count IS a signal: a region that materializes an extra op node reaches 6,
    // so the invariance above is meaningful (not a constant).
    let materialized = Region::new(3, "1", "1", Node::op("add", vec![
        Node::op("mul", vec![Node::Bind(0), Node::op("add", vec![Node::Bind(1), Node::Bind(2)])]),
        Node::Bind(0),
    ]));
    let pm = decode(&encode(&materialized).unwrap()).unwrap();
    assert_eq!(pm.nodes.len(), 6, "a materialized extra op node bumps node_count to 6 (the drift the extract avoids)");
}

// ---------------------------------------------------------------------------
// 6.3-0002 — an exact-vs-approx flavor is a DISTINCT op_name, not a free-form flag.
// ---------------------------------------------------------------------------

/// KISS-GRAMMAR-6.3-0002 — an exact-vs-approximate flavor distinction MUST be
/// expressed as DISTINCT KISS-Ops op names (`gelu` exact-erf vs `gelu_tanh`;
/// `relu` vs `fmax_ieee`; `rem_floor` vs `rem_trunc`) and MUST NOT be a free-form
/// approximation/precision flag on `synthesis_attrs`. Teeth: each flavor pair has
/// DIFFERENT tag bytes, and the difference is localized ENTIRELY to the `op_name`
/// token — the two flavors carry byte-identical trailing (empty OpAttrs + empty
/// operand-role) fields, so the tag/node schema has no separate flag field a flavor
/// could hide in. Mutation: a codec that carried the flavor as a non-identity
/// free-form flag OFF the tag (or an identity-preserving impl that folded
/// `gelu`/`gelu_tanh` onto one name) would collapse a flavor pair to identical
/// identity bytes — here they differ, so the `assert_ne!` fails on that drift.
/// (Adjacent to the backed 6.6-0001 op-name-spelling; distinct: this is
/// flavor-as-name semantics over `encode_tag`, not case/synonym spelling.)
#[test]
fn test_grammar_flavor_is_distinct_op_name() {
    let _clause = "KISS-GRAMMAR-6.3-0002";
    // Unary flavors: one operand, empty OpAttrs, all-default roles — so the ONLY
    // possible carrier of the flavor is the op_name token itself.
    let tag = |name: &str| encode_tag(name, &[], &[], 1);

    // Each exact-vs-approx flavor pair is two DISTINCT op_names -> distinct tags.
    for (exact, approx) in [("gelu", "gelu_tanh"), ("relu", "fmax_ieee"), ("rem_floor", "rem_trunc")] {
        let te = tag(exact);
        let ta = tag(approx);
        assert_ne!(te, ta,
            "{_clause}: flavor {exact:?} vs {approx:?} is a distinct op_name -> distinct tag identity");
        // The flavor lives in the op_name ONLY: everything AFTER the op_name token is
        // byte-identical (empty OpAttrs 00 00 + k=0 roles 00 00). There is no separate
        // precision/approx flag field in the tag for a flavor to hide in.
        let tail = |t: &[u8], name: &str| t[2 + name.len()..].to_vec();
        assert_eq!(tail(&te, exact), tail(&ta, approx),
            "{_clause}: the flavor pair differs ONLY in the op_name token (no hidden flag field)");
        assert_eq!(tail(&te, exact), vec![0x00, 0x00, 0x00, 0x00],
            "the non-name fields are exactly empty OpAttrs + k=0 roles (no flag slot)");
    }

    // Cross-check: `gelu` is also distinct from an unrelated op, and equal-name tags
    // are equal — the tag is a pure function of the op_name (+ its empty attrs), so a
    // flavor CANNOT be a free-form flag riding alongside an identical name.
    assert_ne!(tag("gelu"), tag("relu"), "distinct op names are distinct tags");
}
