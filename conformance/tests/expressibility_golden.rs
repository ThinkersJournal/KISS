//! KISS-Conform golden + decline vectors for the Appendix-F expressible-
//! signature-set byte format. Backs **KISS-CONFORM-6.10-0006**: the enumerated
//! expressible-signature set's byte form is pinned by Appendix F, and a reader
//! MUST reject a non-conforming set with a typed decline. The byte goldens
//! below are hand-derived from KISS-GRAMMAR §6.4-0010 (canonical subtree
//! serialization) + §6.8 field encodings (op_name §6.8-0003, consumers
//! §6.8-0005, OpAttrs blob §6.8-0007, operand-role tuple §6.8-0008), then
//! cross-checked against the reference encoder's actual output before being
//! transcribed into Appendix F's `<hex>` placeholders (round-trip discipline,
//! matching the structure_key goldens).

use kiss_conformance::expressibility::*;

fn bind(i: u32) -> Node {
    Node::Bind(i)
}

fn op(name: &str) -> Node {
    Node::Op { name: name.to_string(), opattrs: Vec::new() }
}

fn op_attrs(name: &str, opattrs: Vec<u8>) -> Node {
    Node::Op { name: name.to_string(), opattrs }
}

/// A 3-node signature `op_name(Bind ., Bind .)`: node 0 = `Bind{0}`, node 1 =
/// `Bind{1}`, node 2 (the root, since root = last node) = the op node, whose
/// authored operand order is `operand_edges` (indices into `nodes`).
fn binary_sig(op_name: &str, operand_edges: [u32; 2]) -> Signature {
    Signature {
        nodes: vec![bind(0), bind(1), op(op_name)],
        edges: vec![vec![], vec![], operand_edges.to_vec()],
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn valid_set() -> SignatureSet {
    SignatureSet {
        owner: "KISS-OPS".to_string(),
        ops_op_set_version: "1".to_string(),
        opattrs_wire_version: "1".to_string(),
        generator: "canonical-regen".to_string(),
        signatures: vec![binary_sig("add", [0, 1])],
    }
}

/// The `add(Bind 0, Bind 1)` bytes derived by hand from KISS-GRAMMAR §6.4-0010 +
/// §6.8: `01`(Op tag) `0300`(name u16-LE len = 3) `616464`(`"add"`)
/// `01`(consumers = ROOT, this IS the root/last node) `0000`(OpAttrs u16-LE
/// len = 0) `0000`(operand-role-tuple u16-LE count = 0, structure-only
/// wildcard projection) `00 00000000`(Bind 0: tag `0x00` + u32-LE `0`)
/// `00 01000000`(Bind 1: tag `0x00` + u32-LE `1`) = 21 bytes total.
const ADD_BYTES_HEX: &str = "010300616464010000000000000000000001000000";
/// Identical shape; `"sub"` (`737562`) replaces `"add"` (`616464`).
const SUB_BYTES_HEX: &str = "010300737562010000000000000000000001000000";

/// KISS-CONFORM-6.10-0006 (byte golden, forward name-match backing the
/// clause): the reference encoder reproduces the hand-derived `add`/`sub`
/// bytes exactly. This is the hard cross-check — if this fails, the encoder
/// (or the byte-format derivation) has a bug and must NOT be "adjusted" to
/// match a wrong output.
#[test]
fn test_conform_expressible_signature_set_schema() {
    let add = binary_sig("add", [0, 1]);
    assert_eq!(
        hex_lower(&serialize_signature_bytes(&add)),
        ADD_BYTES_HEX,
        "add(Bind0,Bind1) byte golden (KISS-GRAMMAR §6.4-0010 + §6.8)"
    );

    let sub = binary_sig("sub", [0, 1]);
    assert_eq!(
        hex_lower(&serialize_signature_bytes(&sub)),
        SUB_BYTES_HEX,
        "sub(Bind0,Bind1) byte golden"
    );
}

/// Commutative canonicalization (Grammar §6.4-0010 / Conform Appendix F):
/// `add` is KISS-Ops-declared commutative, so `add(Bind1,Bind0)` (authored
/// reversed) canonicalizes to the SAME bytes as `add(Bind0,Bind1)` — they are
/// the same signature. `sub` is positional, so the two authored orders MUST
/// be DISTINCT signatures.
// Backs: KISS-GRAMMAR-6.4-0010
#[test]
fn commutative_add_canonicalizes_positional_sub_does_not() {
    let add_01 = binary_sig("add", [0, 1]);
    let add_10 = binary_sig("add", [1, 0]);
    assert_eq!(
        serialize_signature_bytes(&add_01),
        serialize_signature_bytes(&add_10),
        "commutative add must canonicalize to identical bytes regardless of authored operand order"
    );

    let sub_01 = binary_sig("sub", [0, 1]);
    let sub_10 = binary_sig("sub", [1, 0]);
    assert_ne!(
        serialize_signature_bytes(&sub_01),
        serialize_signature_bytes(&sub_10),
        "positional sub must NOT canonicalize -- authored operand order is load-bearing"
    );
}

/// A 5-node signature exercising the general (non-3-node) case: `add(mul(Bind
/// 0, Bind 1), Bind 2)`, where the interior `mul` node carries a NON-EMPTY
/// OpAttrs blob. Node order (KISS-Grammar canonical: every child strictly
/// earlier than its parent, root last): 0 = `Bind{0}`, 1 = `Bind{1}`, 2 =
/// `Op{mul}` (interior, operands `[0,1]`), 3 = `Bind{2}`, 4 = `Op{add}` (root,
/// whose authored operand order is `root_operand_order` — either `[2,3]`
/// (mul then Bind2) or `[3,2]` (Bind2 then mul, the "wrong"/reversed order).
fn nested_add_mul_sig(root_operand_order: [u32; 2]) -> Signature {
    Signature {
        nodes: vec![
            bind(0),                            // 0
            bind(1),                            // 1
            op_attrs("mul", vec![0xAB, 0xCD]),  // 2 (interior; non-empty OpAttrs)
            bind(2),                             // 3
            op("add"),                           // 4 (root)
        ],
        edges: vec![
            vec![],                  // 0: Bind
            vec![],                  // 1: Bind
            vec![0, 1],              // 2: mul(Bind0, Bind1)
            vec![],                  // 3: Bind
            root_operand_order.to_vec(), // 4: add(., .)
        ],
    }
}

/// `add(mul(Bind0,Bind1; opattrs=ab cd), Bind2)`, round-tripped from the
/// encoder's own output (§6.4-0010 + §6.8, general/nested case).
const NESTED_ADD_MUL_BYTES_HEX: &str =
    "010300616464010000000000020000000103006d756c000200abcd000000000000000001000000";

/// The general (nested-Op, non-empty-OpAttrs) case, previously entirely
/// untested (every other test here uses a flat 3-node DAG). Exercises: (a) no
/// panic serializing a deeper DAG; (b) the interior `mul` node's `consumers`
/// byte is INTERIOR (`0x00`), not ROOT; (c) its non-empty OpAttrs length
/// prefix + bytes (`02 00 ab cd`) are correctly emitted; (d) commutative
/// canonicalization reorders an Op-subtree/Bind-leaf sibling pair by
/// byte-lex, independent of authored order (the root `add`'s two operands are
/// the `mul` subtree and `Bind{2}` — a Bind leaf's 5-byte serialization is
/// always byte-lex-less than any Op node's, whose first byte is `0x01`, so
/// the canonical order is always `[Bind2, mul]` regardless of which was
/// authored first).
#[test]
fn nested_op_interior_consumers_and_opattrs_with_commutative_sibling_reordering() {
    let authored_mul_first = nested_add_mul_sig([2, 3]);
    let authored_bind_first = nested_add_mul_sig([3, 2]); // "wrong" authored order

    // (a) serializes without panicking for both authored orders.
    let bytes_a = serialize_signature_bytes(&authored_mul_first);
    let bytes_b = serialize_signature_bytes(&authored_bind_first);

    // (d) commutative canonicalization: an Op-subtree/Bind-leaf sibling pair
    // under a commutative root canonicalizes identically regardless of
    // authored order.
    assert_eq!(
        bytes_a, bytes_b,
        "commutative add root must canonicalize its (mul-subtree, Bind leaf) operands to the same byte-lex order regardless of authored order"
    );

    // pinned golden bytes (round-tripped from this same encoder's output).
    assert_eq!(hex_lower(&bytes_a), NESTED_ADD_MUL_BYTES_HEX);

    // (b) + (c): the interior `mul` node's tag+name-len+"mul"+consumers must
    // show INTERIOR (0x00), immediately followed by its non-empty OpAttrs
    // length prefix + bytes (02 00 ab cd) -- checked as one contiguous
    // window, so both assertions are meaningful even if the exact full
    // golden bytes above ever change.
    let mul_interior_with_opattrs: [u8; 11] =
        [0x01, 0x03, 0x00, 0x6d, 0x75, 0x6c, 0x00, 0x02, 0x00, 0xab, 0xcd];
    assert!(
        bytes_a.windows(mul_interior_with_opattrs.len()).any(|w| w == mul_interior_with_opattrs),
        "expected the interior `mul` node's consumers=INTERIOR (0x00) + non-empty OpAttrs (02 00 ab cd) window"
    );

    // the root `add` node's consumers byte must show ROOT (0x01).
    let add_root: [u8; 7] = [0x01, 0x03, 0x00, 0x61, 0x64, 0x64, 0x01];
    assert!(
        bytes_a.windows(add_root.len()).any(|w| w == add_root),
        "expected the root `add` node's consumers=ROOT (0x01) window"
    );
}

/// The Appendix F golden `signature_hash` values (FNV-1a-64 of the pinned
/// `bytes` goldens above), pinned as literals so a change to the hash
/// algorithm/constants (e.g. the FNV prime) shows up as a FAILING test here
/// instead of silently staying green — `full_set_json_matches_appendix_f_golden`
/// below builds its "expected" string by calling `signature_hash_hex` itself,
/// which is self-referential and would drift silently without this pin.
const ADD_SIGNATURE_HASH_HEX: &str = "23ed69eedebe6a50";
const SUB_SIGNATURE_HASH_HEX: &str = "3cda77a1e825ea1f";

/// Appendix F full-set JSON golden: a one-`add`-signature set at op-set
/// version `1` / OpAttrs wire version `1`. Object keys ascending code-point
/// order, no insignificant whitespace, LF-only (there is none here — the
/// whole document is one line).
#[test]
fn full_set_json_matches_appendix_f_golden() {
    let add_bytes = kiss_conformance::parse_hex(ADD_BYTES_HEX);
    let sub_bytes = kiss_conformance::parse_hex(SUB_BYTES_HEX);
    // Pin the hash values themselves (see the const doc comment above) before
    // using `signature_hash_hex` to build the "expected" JSON string below —
    // that later use only checks JSON *shape* (key order/escaping/array
    // formatting), not the hash algorithm, so it must not be the only guard.
    assert_eq!(signature_hash_hex(&add_bytes), ADD_SIGNATURE_HASH_HEX, "pinned Appendix F add signature_hash (FNV-1a-64)");
    assert_eq!(signature_hash_hex(&sub_bytes), SUB_SIGNATURE_HASH_HEX, "pinned Appendix F sub signature_hash (FNV-1a-64)");

    let set = valid_set();
    let got = String::from_utf8(serialize_set(&set)).unwrap();
    let want = format!(
        "{{\"generator\":\"canonical-regen\",\"opattrs_wire_version\":\"1\",\"ops_op_set_version\":\"1\",\"owner\":\"KISS-OPS\",\"signatures\":[{{\"bytes\":\"{}\",\"edges\":[[],[],[0,1]],\"nodes\":[\"Bind{{0}}\",\"Bind{{1}}\",\"Op{{add;}}\"],\"signature_hash\":\"{}\"}}]}}",
        ADD_BYTES_HEX,
        ADD_SIGNATURE_HASH_HEX,
    );
    assert_eq!(got, want);
}

fn valid_sub_set() -> SignatureSet {
    SignatureSet {
        owner: "KISS-OPS".to_string(),
        ops_op_set_version: "1".to_string(),
        opattrs_wire_version: "1".to_string(),
        generator: "canonical-regen".to_string(),
        signatures: vec![binary_sig("sub", [0, 1])],
    }
}

/// Appendix F full-set JSON golden — the `sub` set (the SECOND Appendix-F
/// golden; only the `add` set was previously asserted end-to-end through
/// `serialize_set`, leaving the `sub` golden JSON doc unexercised by anything
/// but the raw `bytes` check). Mirrors `full_set_json_matches_appendix_f_golden`.
#[test]
fn full_set_json_matches_appendix_f_golden_for_sub() {
    let sub_bytes = kiss_conformance::parse_hex(SUB_BYTES_HEX);
    assert_eq!(
        signature_hash_hex(&sub_bytes),
        SUB_SIGNATURE_HASH_HEX,
        "pinned Appendix F sub signature_hash (FNV-1a-64)"
    );

    let set = valid_sub_set();
    let got = String::from_utf8(serialize_set(&set)).unwrap();
    let want = format!(
        "{{\"generator\":\"canonical-regen\",\"opattrs_wire_version\":\"1\",\"ops_op_set_version\":\"1\",\"owner\":\"KISS-OPS\",\"signatures\":[{{\"bytes\":\"{}\",\"edges\":[[],[],[0,1]],\"nodes\":[\"Bind{{0}}\",\"Bind{{1}}\",\"Op{{sub;}}\"],\"signature_hash\":\"{}\"}}]}}",
        SUB_BYTES_HEX,
        SUB_SIGNATURE_HASH_HEX,
    );
    assert_eq!(got, want);
}

/// `serialize_set` MUST order the `signatures` array ascending by each
/// signature's `bytes` (unsigned-byte-lex, Appendix F "Top-level fields"):
/// `add`'s bytes (`0103006164...`) < `sub`'s bytes (`0103007375...`) since
/// `0x61` (`'a'`) < `0x73` (`'s'`) at the first differing byte (the op_name).
/// The set is authored with `sub` FIRST to prove the sort actually reorders,
/// rather than merely preserving an input that happened to already be sorted
/// (the previously-only-tested one-signature sets never exercised this sort
/// at all).
#[test]
fn serialize_set_orders_signatures_ascending_by_bytes() {
    let set = SignatureSet {
        owner: "KISS-OPS".to_string(),
        ops_op_set_version: "1".to_string(),
        opattrs_wire_version: "1".to_string(),
        generator: "canonical-regen".to_string(),
        signatures: vec![binary_sig("sub", [0, 1]), binary_sig("add", [0, 1])], // authored sub-then-add
    };
    let got = String::from_utf8(serialize_set(&set)).unwrap();
    let sig_array_start = got.find("\"signatures\":[").expect("signatures array present");
    let add_pos = got.find(ADD_BYTES_HEX).expect("add bytes present");
    let sub_pos = got.find(SUB_BYTES_HEX).expect("sub bytes present");
    assert!(
        sig_array_start < add_pos && add_pos < sub_pos,
        "expected `add`'s signature object to precede `sub`'s in the ascending-by-bytes `signatures` array despite being authored sub-then-add; got: {got}"
    );
}

// ---- declines (§6.10-0006: reject a set missing a REQUIRED field, carrying
// an unknown field, or violating an enumerant) --------------------------------

// Backs: KISS-CONFORM-6.10-0006
#[test]
fn reject_set_declines_missing_owner() {
    let valid = String::from_utf8(serialize_set(&valid_set())).unwrap();
    let without_owner = valid.replacen("\"owner\":\"KISS-OPS\",", "", 1);
    assert_ne!(without_owner, valid, "sanity: the mutation actually changed the document");
    assert_eq!(
        reject_set(without_owner.as_bytes()),
        Err(Decline::MissingField("owner".to_string()))
    );
    // the unmutated document is accepted.
    assert!(reject_set(valid.as_bytes()).is_ok());
}

#[test]
fn reject_set_declines_unknown_field() {
    let valid = String::from_utf8(serialize_set(&valid_set())).unwrap();
    let with_extra = valid.replacen(
        "\"owner\":\"KISS-OPS\",",
        "\"owner\":\"KISS-OPS\",\"extra_field\":\"x\",",
        1,
    );
    assert_eq!(
        reject_set(with_extra.as_bytes()),
        Err(Decline::UnknownField("extra_field".to_string()))
    );
}

#[test]
fn reject_set_declines_bad_owner_enumerant() {
    let valid = String::from_utf8(serialize_set(&valid_set())).unwrap();
    let bad_owner = valid.replacen("\"owner\":\"KISS-OPS\"", "\"owner\":\"NOT-KISS-OPS\"", 1);
    assert_eq!(
        reject_set(bad_owner.as_bytes()),
        Err(Decline::BadEnumerant("owner=NOT-KISS-OPS".to_string()))
    );
}

/// A `signatures` value that is present (so the REQUIRED-field check alone
/// cannot catch it) but is NOT an array (a bare number here) must still be
/// declined -- Appendix F requires `signatures` to be an array, and
/// present-but-wrong-shaped is as much a schema violation as absent.
#[test]
fn reject_set_declines_non_array_signatures() {
    let valid = String::from_utf8(serialize_set(&valid_set())).unwrap();
    let sig_value_start = valid.find("\"signatures\":").unwrap() + "\"signatures\":".len();
    let malformed = format!("{}5}}", &valid[..sig_value_start]);
    assert_eq!(
        reject_set(malformed.as_bytes()),
        Err(Decline::MissingField("signatures[array]".to_string()))
    );
}
