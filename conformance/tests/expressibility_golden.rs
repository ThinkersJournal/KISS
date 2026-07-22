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

/// Appendix F full-set JSON golden: a one-`add`-signature set at op-set
/// version `1` / OpAttrs wire version `1`. Object keys ascending code-point
/// order, no insignificant whitespace, LF-only (there is none here — the
/// whole document is one line). The expected `signature_hash` is computed via
/// the same public `signature_hash_hex` the encoder itself uses over the
/// independently byte-golden-checked `ADD_BYTES_HEX`; this test's job is the
/// JSON *shape* (key order/escaping/array formatting), not re-deriving the
/// hash algorithm.
#[test]
fn full_set_json_matches_appendix_f_golden() {
    let set = valid_set();
    let got = String::from_utf8(serialize_set(&set)).unwrap();
    let add_bytes = kiss_conformance::parse_hex(ADD_BYTES_HEX);
    let want = format!(
        "{{\"generator\":\"canonical-regen\",\"opattrs_wire_version\":\"1\",\"ops_op_set_version\":\"1\",\"owner\":\"KISS-OPS\",\"signatures\":[{{\"bytes\":\"{}\",\"edges\":[[],[],[0,1]],\"nodes\":[\"Bind{{0}}\",\"Bind{{1}}\",\"Op{{add;}}\"],\"signature_hash\":\"{}\"}}]}}",
        ADD_BYTES_HEX,
        signature_hash_hex(&add_bytes),
    );
    assert_eq!(got, want);
}

// ---- declines (§6.10-0006: reject a set missing a REQUIRED field, carrying
// an unknown field, or violating an enumerant) --------------------------------

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
