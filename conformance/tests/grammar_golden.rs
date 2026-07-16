//! KISS-Conform golden + rule-mutation vectors for the KISS-Grammar region wire
//! form (Grammar §6.4, §6.8), tested against Appendix A.1 vectors G1
//! (`out=(a*b)+c`, n_inputs=3) and G4 (`out=a+a`, n_inputs=1).

use kiss_conformance::grammar::*;
use kiss_conformance::{assert_golden, parse_hex};

const G1_GOLDEN: &str = concat!(
    "4B 47 52 4D ", "01 00 ", "03 00 00 00 ", "01 00 31 ", "01 00 31 ", "05 00 00 00 ",
    "00 02 00 00 00 ", "00 00 00 00 00 ", "00 01 00 00 00 ",
    "01 03 00 6D 75 6C 00 00 00 00 00 02 00 01 00 00 00 02 00 00 00 ",
    "01 03 00 61 64 64 01 00 00 00 00 02 00 00 00 00 00 03 00 00 00 ",
    "00 00",
);
const G4_GOLDEN: &str = concat!(
    "4B 47 52 4D ", "01 00 ", "01 00 00 00 ", "01 00 31 ", "01 00 31 ", "02 00 00 00 ",
    "00 00 00 00 00 ",
    "01 03 00 61 64 64 01 00 00 00 00 02 00 00 00 00 00 00 00 00 00 ",
    "00 00",
);
fn g1_region() -> Region {
    Region::new(3, "1", "1", Node::op("add", vec![
        Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]), Node::Bind(2)]))
}
fn g4_region() -> Region {
    Region::new(1, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Bind(0)]))
}

/// KISS-GRAMMAR-6.8-0001 — region serialization field order, incl. the trailing
/// empty `extract_count`. Teeth: an impl that omits the trailing `00 00`
/// extract_count for an empty extract list (2 bytes short) fails the golden.
#[test]
fn test_grammar_region_wire_form_field_order() {
    let b = encode(&g1_region()).unwrap();
    assert_golden("KISS-GRAMMAR-6.8-0001", "G1", &b, G1_GOLDEN);
    // The region MUST end with extract_count = 0x0000 even when empty (§6.8-0006).
    assert_eq!(&b[b.len() - 2..], &[0x00, 0x00], "trailing empty extract_count required");
    // Dropping the trailing extract_count would no longer match the golden.
    assert_ne!(&b[..b.len() - 2], parse_hex(G1_GOLDEN).as_slice());
}

/// KISS-GRAMMAR-6.8-0002 — fixed little-endian integer widths. Teeth: `n_inputs`
/// or an operand node-index encoded as `u16` (or `operand_count` as `u32`) shifts
/// every following byte and fails these spans.
#[test]
fn test_grammar_wire_integer_encoding() {
    let b = encode(&g1_region()).unwrap();
    // n_inputs is a u32 at offset 6..10 (after 4-byte magic + 2-byte schema ver).
    assert_eq!(u32::from_le_bytes(b[6..10].try_into().unwrap()), 3);
    // node_count is a u32 at offset 16..20 (after the two 3-byte version tokens).
    assert_eq!(u32::from_le_bytes(b[16..20].try_into().unwrap()), 5);
    // The root "add" record's operand_count is a u16 and each operand index a u32.
    // add record = last 21 bytes before the 2-byte extract_count.
    let add = &b[b.len() - 2 - 21..b.len() - 2];
    // layout: kind(1) namelen(2) name(3) consumers(1) oalen(2) roles(2) opcount(2) ops(4+4)
    assert_eq!(u16::from_le_bytes(add[11..13].try_into().unwrap()), 2, "operand_count u16");
    assert_eq!(u32::from_le_bytes(add[13..17].try_into().unwrap()), 0, "operand[0] u32");
    assert_eq!(u32::from_le_bytes(add[17..21].try_into().unwrap()), 3, "operand[1] u32");
}

/// KISS-GRAMMAR-6.8-0003 — every token is a `u16`-LE length + UTF-8 bytes. Teeth:
/// a `u8`-length token (`03 6D 75 6C`) or a NUL-terminated token
/// (`6D 75 6C 00`) both diverge from the pinned `03 00 6D 75 6C`.
#[test]
fn test_grammar_wire_token_encoding() {
    let b = encode(&g1_region()).unwrap();
    // ops_version token at offset 10: length u16 = 0x0001, then "1" = 0x31.
    assert_eq!(&b[10..13], &[0x01, 0x00, 0x31]);
    // classify_version token at offset 13: same u16-length form.
    assert_eq!(&b[13..16], &[0x01, 0x00, 0x31]);
    // op_name "mul" appears with a u16 length prefix: 03 00 6D 75 6C. A u8-length
    // token would be `03 6D 75 6C` and a NUL-terminated one `6D 75 6C 00`; neither
    // contains this 5-byte window, so its presence is the discriminator.
    assert!(
        b.windows(5).any(|w| w == [0x03, 0x00, 0x6D, 0x75, 0x6C]),
        "op_name must be u16-length-prefixed UTF-8"
    );
    // Round-trip preserves the exact spelling.
    let pr = decode(&b).unwrap();
    assert_eq!(pr.ops_version, "1");
    assert_eq!(pr.classify_version, "1");
}

/// KISS-GRAMMAR-6.8-0004 — node table is post-order from the root, index assigned
/// at first visit; the root is last. Teeth: a pre-order table (root first) or any
/// non-canonical order.
#[test]
fn test_grammar_wire_node_table_canonical_order() {
    let pr = decode(&encode(&g1_region()).unwrap()).unwrap();
    // Canonical post-order (Appendix A.1 table): Bind(2),Bind(0),Bind(1),mul,add.
    assert_eq!(pr.nodes[0], ParsedNode::Bind(2));
    assert_eq!(pr.nodes[1], ParsedNode::Bind(0));
    assert_eq!(pr.nodes[2], ParsedNode::Bind(1));
    match &pr.nodes[3] {
        ParsedNode::Op { op_name, operands, .. } => {
            assert_eq!(op_name, "mul");
            assert_eq!(operands, &vec![1, 2]);
        }
        _ => panic!("node3 must be mul"),
    }
    match &pr.nodes[4] {
        ParsedNode::Op { op_name, consumers, operands, .. } => {
            assert_eq!(op_name, "add");
            assert_eq!(*consumers, CONSUMERS_ROOT, "root is the LAST entry");
            assert_eq!(operands, &vec![0, 3]);
        }
        _ => panic!("node4 (last) must be the add root"),
    }
}

/// KISS-GRAMMAR-6.8-0005 — `consumers` u8 enum: 0x00 = INTERIOR, 0x01 = ROOT.
/// Teeth: swapping the two enum values (interior=0x01/root=0x00) flips both bytes.
#[test]
fn test_grammar_wire_consumers_enum() {
    assert_eq!(CONSUMERS_INTERIOR, 0x00);
    assert_eq!(CONSUMERS_ROOT, 0x01);
    let b = encode(&g1_region()).unwrap();
    // mul's consumers byte follows its 03 00 6D 75 6C name.
    let mul_at = b.windows(5).position(|w| w == [0x03, 0x00, 0x6D, 0x75, 0x6C]).unwrap();
    assert_eq!(b[mul_at + 5], CONSUMERS_INTERIOR, "interior mul = 0x00");
    let add_at = b.windows(5).position(|w| w == [0x03, 0x00, 0x61, 0x64, 0x64]).unwrap();
    assert_eq!(b[add_at + 5], CONSUMERS_ROOT, "root add = 0x01");
}

/// KISS-GRAMMAR-6.8-0006 — extract record: `path_len` u16, then `path_len` steps
/// (each u16), then `param_slot` u32. Teeth: `param_slot` as u16, or `path_len`
/// as u8, changes the trailing byte layout.
#[test]
fn test_grammar_wire_extract_encoding() {
    // Empty extract list encodes as extract_count = 0x0000, no records.
    let empty = encode(&g1_region()).unwrap();
    assert_eq!(&empty[empty.len() - 2..], &[0x00, 0x00]);

    // One extract: path [1] (root's operand index 1 = mul), param_slot 0.
    let mut r = g1_region();
    r.extracts = vec![Extract { path: vec![1], param_slot: 0 }];
    let b = encode(&r).unwrap();
    // Tail: extract_count=1 | path_len=1 | step=1 | param_slot=0(u32).
    let tail = &b[b.len() - 10..];
    assert_eq!(tail, &[
        0x01, 0x00, // extract_count = 1 (u16)
        0x01, 0x00, // path_len = 1 (u16)
        0x01, 0x00, // step[0] = 1 (u16)
        0x00, 0x00, 0x00, 0x00, // param_slot = 0 (u32)
    ]);
    // Round-trip the extract.
    let pr = decode(&b).unwrap();
    assert_eq!(pr.extracts, vec![Extract { path: vec![1], param_slot: 0 }]);
}

/// KISS-GRAMMAR-6.8-0010 — Op node record field order: kind, op_name, consumers,
/// opattrs, operand-role tuple, operand_count, operand indices. Teeth: any field
/// reorder (e.g. operand_count before consumers) diverges from this exact slice.
#[test]
fn test_grammar_wire_node_record_layout() {
    let b = encode(&g1_region()).unwrap();
    // The root "add" record is the 21 bytes before the 2-byte extract_count.
    let add = &b[b.len() - 2 - 21..b.len() - 2];
    assert_eq!(add, &[
        0x01,                   // kind = Op
        0x03, 0x00, 0x61, 0x64, 0x64, // op_name = "add"
        0x01,                   // consumers = ROOT
        0x00, 0x00,             // opattrs length = 0
        0x00, 0x00,             // operand-role tuple count = 0
        0x02, 0x00,             // operand_count = 2
        0x00, 0x00, 0x00, 0x00, // operand[0] = 0
        0x03, 0x00, 0x00, 0x00, // operand[1] = 3
    ]);
}

/// KISS-GRAMMAR-6.4-0001 — every node is Op or Bind and every operand index
/// references a strictly-earlier table entry. Teeth: a reader that accepts an
/// operand index >= its own node index (a forward/self reference), or an
/// undefined `kind` byte.
#[test]
fn test_grammar_region_is_single_rooted_dag() {
    let b = encode(&g1_region()).unwrap();
    // A valid region round-trips.
    assert!(decode(&b).is_ok());

    // Mutate mul's operand[0] from 1 to 4 (references the later root) -> decline.
    let mut fwd = b.clone();
    let mul_at = fwd.windows(5).position(|w| w == [0x03, 0x00, 0x6D, 0x75, 0x6C]).unwrap();
    // after name(5) + consumers(1) + oalen(2) + roles(2) + opcount(2) = first operand.
    let op0 = mul_at + 5 + 1 + 2 + 2 + 2;
    fwd[op0] = 4;
    assert_eq!(
        decode(&fwd),
        Err(DecodeDecline::ForwardOperandReference { node: 3, operand: 4 })
    );

    // An undefined node kind byte (0x02) declines rather than being admitted.
    let mut badkind = b.clone();
    badkind[20] = 0x02; // node0's kind tag
    assert_eq!(decode(&badkind), Err(DecodeDecline::BadKind { got: 0x02 }));
}

/// KISS-GRAMMAR-6.4-0003 — repeated `Bind(i)` is the node-identity guard: both
/// `Bind(0)` leaves of `a+a` coalesce to ONE table entry. Teeth: an impl that
/// emits two separate `Bind(0)` entries yields `node_count = 3`, not 2.
#[test]
fn test_grammar_repeated_bind_is_node_identity() {
    let b = encode(&g4_region()).unwrap();
    assert_golden("KISS-GRAMMAR-6.4-0003", "G4", &b, G4_GOLDEN);
    let pr = decode(&b).unwrap();
    assert_eq!(pr.nodes.len(), 2, "two Bind(0) leaves coalesce to one entry");
    assert_eq!(pr.nodes[0], ParsedNode::Bind(0));
    match &pr.nodes[1] {
        ParsedNode::Op { operands, .. } => assert_eq!(operands, &vec![0, 0], "both operands reference the one Bind(0)"),
        _ => panic!("root must be add"),
    }
}

/// KISS-GRAMMAR-6.4-0004 — `consumers` counts only edges leaving the region: an
/// interior node is INTERIOR, the root is ROOT. Teeth: an impl that marks the
/// root INTERIOR, or an interior node ROOT.
#[test]
fn test_grammar_interior_no_external_consumer_guard() {
    let pr = decode(&encode(&g1_region()).unwrap()).unwrap();
    // Interior mul carries INTERIOR; root add carries ROOT.
    match &pr.nodes[3] {
        ParsedNode::Op { op_name, consumers, .. } => {
            assert_eq!(op_name, "mul");
            assert_eq!(*consumers, CONSUMERS_INTERIOR);
        }
        _ => panic!(),
    }
    match &pr.nodes[4] {
        ParsedNode::Op { op_name, consumers, .. } => {
            assert_eq!(op_name, "add");
            assert_eq!(*consumers, CONSUMERS_ROOT);
        }
        _ => panic!(),
    }
    // A decoder MUST reject a stream whose last (root) node is not ROOT.
    let mut b = encode(&g1_region()).unwrap();
    let add_at = b.windows(5).position(|w| w == [0x03, 0x00, 0x61, 0x64, 0x64]).unwrap();
    b[add_at + 5] = CONSUMERS_INTERIOR; // downgrade the root to interior
    assert_eq!(decode(&b), Err(DecodeDecline::RootNotRoot));
}

/// KISS-GRAMMAR-6.4-0008 — `n_inputs` is a DECLARED field; a region is
/// expressible only when its bound set equals exactly `[0, n_inputs)`. Teeth: an
/// impl that infers `n_inputs = max(bound)+1` cannot detect the strict subset
/// `{0,1}` under a declared `n_inputs = 3` and would wrongly accept it.
#[test]
fn test_grammar_bind_set_covers_inputs() {
    // Expressible: G1 binds exactly {0,1,2} = [0,3).
    assert!(encode(&g1_region()).is_ok());

    // Strict subset: declared n_inputs = 3 but only inputs {0,1} are bound.
    let subset = Region::new(3, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Bind(1)]));
    assert_eq!(
        encode(&subset),
        Err(Decline::BindSetMismatch { declared: 3, unbound: 2 })
    );

    // Out of range: an index >= n_inputs.
    let oor = Region::new(2, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Bind(5)]));
    assert_eq!(
        encode(&oor),
        Err(Decline::BindIndexOutOfRange { index: 5, n_inputs: 2 })
    );
}

/// KISS-GRAMMAR-6.4-0010 — canonical operand order is ascending unsigned-byte
/// lexicographic over each operand's canonical subtree key (a `Bind` key `0x00…`
/// sorts before an `Op` key `0x01…`). Teeth: an impl that preserves author order,
/// or sorts Op-before-Bind — the alternative authoring `c + b*a` MUST still emit
/// the identical G1 bytes.
#[test]
fn test_grammar_canonical_operand_order_key() {
    // Author add's operands reversed (c first) and mul's operands reversed (b,a).
    let reversed = Region::new(
        3,
        "1",
        "1",
        Node::op("add", vec![
            Node::Bind(2),
            Node::op("mul", vec![Node::Bind(1), Node::Bind(0)]),
        ]),
    );
    let b = encode(&reversed).unwrap();
    assert_golden("KISS-GRAMMAR-6.4-0010", "G1-reversed", &b, G1_GOLDEN);

    // The key ordering itself: Bind subtree (0x00…) < Op subtree (0x01…).
    let bind_key = subtree_key(&Node::Bind(2));
    let op_key = subtree_key(&Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]));
    assert_eq!(bind_key[0], 0x00);
    assert_eq!(op_key[0], 0x01);
    assert!(bind_key < op_key);
}

/// KISS-GRAMMAR-6.4-0011 — maximal structural sharing: byte-identical canonical
/// subtrees coalesce to ONE node, for `Op` subtrees as well as `Bind` leaves.
/// Teeth: an impl that dedups `Bind` leaves but not identical `Op` subtrees emits
/// two `mul` entries (`node_count = 5`) instead of one (`node_count = 4`).
#[test]
fn test_grammar_structural_dedup_and_repeated_bind() {
    // out = (a*b) + (a*b), n_inputs = 2. The two mul subtrees are byte-identical.
    let r = Region::new(
        2,
        "1",
        "1",
        Node::op("add", vec![
            Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]),
            Node::op("mul", vec![Node::Bind(0), Node::Bind(1)]),
        ]),
    );
    let pr = decode(&encode(&r).unwrap()).unwrap();
    // Bind0, Bind1, mul, add  — the second mul shares the first's index.
    assert_eq!(pr.nodes.len(), 4, "identical Op subtrees coalesce to one entry");
    assert_eq!(pr.nodes[0], ParsedNode::Bind(0));
    assert_eq!(pr.nodes[1], ParsedNode::Bind(1));
    match &pr.nodes[2] {
        ParsedNode::Op { op_name, operands, .. } => {
            assert_eq!(op_name, "mul");
            assert_eq!(operands, &vec![0, 1]);
        }
        _ => panic!(),
    }
    match &pr.nodes[3] {
        ParsedNode::Op { op_name, operands, .. } => {
            assert_eq!(op_name, "add");
            assert_eq!(operands, &vec![2, 2], "both add operands reference the one shared mul");
        }
        _ => panic!(),
    }
}

/// KISS-GRAMMAR-6.1-0008 — the operand-role tuple's canonical entry count `k`:
/// `0` when every operand is `(data, *)`, else `1 +` the greatest operand index
/// differing from the default, with trailing defaults omitted and a leading
/// default emitted explicitly when a later operand is non-default. Teeth: an impl
/// that emits `k = operand_count` for the all-default case, or drops the leading
/// `(data, *)` when operand 1 is non-default, or emits trailing defaults.
#[test]
fn test_grammar_operand_role_tuple_canonical_form() {
    let data = || RoleEntry::default_entry();
    let index_u32 = || RoleEntry { role: "index".into(), dtype: "u32".into() };

    // (a) all-default over 2 operands -> k = 0, no entries.
    assert_eq!(encode_operand_roles(&[], 2), vec![0x00, 0x00]);
    assert_eq!(encode_operand_roles(&[data(), data()], 2), vec![0x00, 0x00]);

    // (b) operand 1 = (index, u32), operand 0 default -> k = 2; BOTH entries
    // emitted, the leading (data,*) made explicit.
    let b = encode_operand_roles(&[data(), index_u32()], 2);
    assert_eq!(b, [
        &[0x02, 0x00][..],                         // k = 2
        &[0x04, 0x00], b"data", &[0x01, 0x00], b"*", // entry0 = (data, *)
        &[0x05, 0x00], b"index", &[0x03, 0x00], b"u32", // entry1 = (index, u32)
    ].concat());

    // (c) operand 0 = (index, u32), operand 1 default -> k = 1; the trailing
    // default is omitted (only the first entry survives).
    let c = encode_operand_roles(&[index_u32(), data()], 2);
    assert_eq!(c, [
        &[0x01, 0x00][..],                          // k = 1
        &[0x05, 0x00], b"index", &[0x03, 0x00], b"u32",
    ].concat());
    // k=1 tuple over a 2-operand node reconstructs operand 1 as (data, *):
    // its byte length is strictly shorter than the k=2 form of case (b).
    assert!(c.len() < b.len());
}
