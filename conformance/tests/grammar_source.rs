// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Grammar `Source` node vectors (§6.4-0013, §6.8-0014): the one node kind that carries the
//! six KISS-Ops scalar-source leaves that are not input binds (`const`, `param`, `coord`,
//! `reduced`, `extent`, `reduced_count`), exercised through `kiss_conformance::grammar`.

use kiss_conformance::grammar::*;

fn c32(bits: u32) -> Node {
    Node::Source(SourceLeaf::Const { dtype: "f32".into(), bits: bits.to_le_bytes().to_vec() })
}

/// `add(Bind(0), leaf)`: a one-op region whose second operand is a `Source`.
fn with_leaf(leaf: SourceLeaf) -> Region {
    Region::new(1, "1", "1", Node::op("add", vec![Node::Bind(0), Node::Source(leaf)]))
}

fn leaf_of(parsed: &ParsedRegion) -> SourceLeaf {
    parsed
        .nodes
        .iter()
        .find_map(|n| match n {
            ParsedNode::Source(s) => Some(s.clone()),
            _ => None,
        })
        .expect("region carries a Source node")
}

/// Enforces[enforced]: KISS-GRAMMAR-6.4-0013 — the six leaves are the closed tag set `{1..6}`;
/// each round-trips through the wire form; a `const` carries its exact bits (a NaN payload, a
/// signaling NaN, a subnormal, a signed zero and an infinity are never quieted or re-spelled);
/// two identical sources coalesce; a `Source` is never the root; a reader declines an unknown tag
/// (`0` and the next ordinal) rather than skipping or aliasing it. TEETH: a codec that quiets NaNs
/// (`0x7F800001` would read back `0x7FC00001`), reuses a tag, or skips an unknown node.
#[test]
fn test_grammar_source_leaf_kinds() {
    tags_and_round_trip();
    exact_bits();
    narrow_widths();
    sharing_and_root();
    unknown_tags();
}

/// The six tag ordinals are pinned, and every leaf round-trips.
fn tags_and_round_trip() {
    // Tag ordinals are pinned, in order.
    assert_eq!(
        [TAG_CONST, TAG_PARAM, TAG_COORD, TAG_REDUCED, TAG_EXTENT, TAG_REDUCED_COUNT],
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(KIND_SOURCE, 0x02);

    // Every leaf round-trips.
    let leaves = [
        SourceLeaf::Const { dtype: "f32".into(), bits: 0x3F80_0000u32.to_le_bytes().to_vec() },
        SourceLeaf::Param(3),
        SourceLeaf::Coord(2),
        SourceLeaf::Reduced(1),
        SourceLeaf::Extent(7),
        SourceLeaf::ReducedCount,
    ];
    for l in leaves {
        let back = leaf_of(&decode(&encode(&with_leaf(l.clone())).unwrap()).unwrap());
        assert_eq!(back, l, "leaf must round-trip");
    }
}

/// A `const` keeps its exact bits: a NaN payload is never quieted.
fn exact_bits() {
    // Exact bits: a NaN is never canonicalized.
    for bits in [
        0x7FC0_0001u32, // quiet NaN with a payload
        0x7F80_0001,    // signaling NaN
        0xFFFF_FFFF,    // negative NaN, all payload
        0x0000_0001,    // smallest subnormal
        0x8000_0000,    // -0
        0x7F80_0000,    // +inf
        0xFF80_0000,    // -inf
    ] {
        let parsed = decode(&encode(&with_leaf(SourceLeaf::Const {
            dtype: "f32".into(),
            bits: bits.to_le_bytes().to_vec(),
        }))
        .unwrap())
        .unwrap();
        match leaf_of(&parsed) {
            SourceLeaf::Const { bits: b, .. } => {
                assert_eq!(u32::from_le_bytes(b.try_into().unwrap()), bits, "bits {bits:#010x} must survive")
            }
            other => panic!("expected a const, got {other:?}"),
        }
    }
}

/// Narrow dtypes: bits above the width, a wrong length, an unknown dtype and an axis past `MAX_RANK` decline.
fn narrow_widths() {
    // A narrow dtype: f4e2m1 is one byte, bits above the 4-bit width must be zero.
    let ok = SourceLeaf::Const { dtype: "f4e2m1".into(), bits: vec![0x0F] };
    assert_eq!(encode(&with_leaf(ok)).map(|_| ()), Ok(()));
    let high = SourceLeaf::Const { dtype: "f4e2m1".into(), bits: vec![0x1F] };
    assert_eq!(encode(&with_leaf(high)), Err(Decline::BadSource(SourceDecline::BitsAboveWidth)));
    let short = SourceLeaf::Const { dtype: "f32".into(), bits: vec![0, 0, 0] };
    assert_eq!(
        encode(&with_leaf(short)),
        Err(Decline::BadSource(SourceDecline::BitsLength { want: 4, got: 3 }))
    );
    let unknown = SourceLeaf::Const { dtype: "f5".into(), bits: vec![0] };
    assert_eq!(encode(&with_leaf(unknown)), Err(Decline::BadSource(SourceDecline::UnknownDtype)));
    assert_eq!(
        encode(&with_leaf(SourceLeaf::Coord(8))),
        Err(Decline::BadSource(SourceDecline::AxisOutOfRange { axis: 8 }))
    );
}

/// Identical sources coalesce; a `Source` is never the root.
fn sharing_and_root() {
    // Sharing: two identical sources are one table entry; two different ones are two.
    let shared = Region::new(
        1,
        "1",
        "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), c32(0x3F80_0000)]), c32(0x3F80_0000)]),
    );
    let sources = |r: &Region| {
        decode(&encode(r).unwrap()).unwrap().nodes.iter().filter(|n| matches!(n, ParsedNode::Source(_))).count()
    };
    assert_eq!(sources(&shared), 1, "identical sources coalesce (§6.4-0011)");
    let distinct = Region::new(
        1,
        "1",
        "1",
        Node::op("add", vec![Node::op("mul", vec![Node::Bind(0), c32(0x3F80_0000)]), c32(0x4000_0000)]),
    );
    assert_eq!(sources(&distinct), 2);

    // A Source is never the root.
    assert_eq!(encode(&Region::new(0, "1", "1", c32(0))), Err(Decline::SourceRoot));
}

/// An unknown tag is a typed decline on read, never skipped or aliased.
fn unknown_tags() {
    // An unknown tag is a typed decline on read: tag 0 and tag 7.
    let good = encode(&with_leaf(SourceLeaf::ReducedCount)).unwrap();
    let at = good.windows(2).position(|w| w == [KIND_SOURCE, TAG_REDUCED_COUNT]).unwrap();
    for bad in [0u8, 7] {
        let mut b = good.clone();
        b[at + 1] = bad;
        assert_eq!(decode(&b), Err(DecodeDecline::BadSourceTag { got: bad }));
    }
}

/// Enforces[enforced]: KISS-GRAMMAR-6.8-0014 — the `Source` record is `kind = 0x02`, `tag` (`u8`),
/// then the tag's payload in the stated order and widths; the stamp is schema version 2; a
/// truncated payload and a last-node `Source` are typed declines. TEETH: a record that put the
/// dtype after the bits, used a `u16` axis, or left the stamp at 1.
#[test]
fn test_grammar_wire_source_record_layout() {
    let region = with_leaf(SourceLeaf::Const { dtype: "f32".into(), bits: 0x3F80_0000u32.to_le_bytes().to_vec() });
    let bytes = encode(&region).unwrap();
    // Stamp: magic then schema version 2 (u16 LE).
    assert_eq!(&bytes[..6], &[0x4B, 0x47, 0x52, 0x4D, 0x02, 0x00]);
    // The const record, byte for byte: kind, tag, token (u16 len + "f32"), bits_len, 3F800000 LE.
    let want = [0x02, 0x01, 0x03, 0x00, b'f', b'3', b'2', 0x04, 0x00, 0x00, 0x80, 0x3F];
    assert!(bytes.windows(want.len()).any(|w| w == want), "const record layout");

    // The other records: param u32 LE, coord u8, reduced u32 LE, extent u8, reduced_count empty.
    let rec = |l: SourceLeaf| l.record();
    assert_eq!(rec(SourceLeaf::Param(0x0102_0304)), [0x02, 0x02, 0x04, 0x03, 0x02, 0x01]);
    assert_eq!(rec(SourceLeaf::Coord(5)), [0x02, 0x03, 0x05]);
    assert_eq!(rec(SourceLeaf::Reduced(2)), [0x02, 0x04, 0x02, 0x00, 0x00, 0x00]);
    assert_eq!(rec(SourceLeaf::Extent(1)), [0x02, 0x05, 0x01]);
    assert_eq!(rec(SourceLeaf::ReducedCount), [0x02, 0x06]);

    // The canonical subtree key of a Source is the same bytes (§6.4-0010).
    assert_eq!(subtree_key(&Node::Source(SourceLeaf::Coord(5))), [0x02, 0x03, 0x05]);

    // A truncated payload is a typed decline, never a panic.
    let p = encode(&with_leaf(SourceLeaf::Param(9))).unwrap();
    let at = p.windows(2).position(|w| w == [KIND_SOURCE, TAG_PARAM]).unwrap();
    assert!(decode(&p[..at + 4]).is_err());

    // A stamp of 1 (the pre-Source version) is refused by this reader.
    let mut old = bytes.clone();
    old[4] = 1;
    assert_eq!(decode(&old), Err(DecodeDecline::UnsupportedSchemaVersion { got: 1 }));
}

/// Enforces[enforced]: KISS-GRAMMAR-6.4-0014 — an extract record must end at a param leaf (the
/// `Source` node tagged `param`) and its `param_slot` must equal that leaf's index; a path that
/// resolves nowhere, a target that is not a param leaf, and a slot that differs are typed declines
/// on both write and read. TEETH: a codec that accepts a path to a `Bind`, to an `Op`, or to a
/// param leaf with another slot, and one that panics on an out-of-range step.
#[test]
fn test_grammar_extract_targets_param_leaf() {
    let region = |extracts: Vec<Extract>| {
        let mut r = Region::new(1, "1", "1", Node::op("mul", vec![
            Node::Bind(0),
            Node::Source(SourceLeaf::Param(7)),
        ]));
        r.extracts = extracts;
        r
    };
    // mul is commutative: the Bind subtree key (0x00) orders before the Source key (0x02), so the
    // canonical operands are [Bind, param leaf] and the leaf is at path [1].
    let good = Extract { path: vec![1], param_slot: 7 };
    assert_eq!(param_leaf_path(&region(vec![]), 7), Some(vec![1]));
    assert_eq!(param_leaf_path(&region(vec![]), 8), None);
    let bytes = encode(&region(vec![good.clone()])).unwrap();
    assert_eq!(decode(&bytes).unwrap().extracts, vec![good]);

    // write side
    let fault = |e: Extract| encode(&region(vec![e])).unwrap_err();
    assert_eq!(fault(Extract { path: vec![0], param_slot: 7 }), Decline::BadExtract(ExtractFault::NotParamLeaf), "a Bind");
    assert_eq!(fault(Extract { path: vec![], param_slot: 7 }), Decline::BadExtract(ExtractFault::NotParamLeaf), "the root op");
    assert_eq!(
        fault(Extract { path: vec![1], param_slot: 6 }),
        Decline::BadExtract(ExtractFault::SlotMismatch { leaf_index: 7 })
    );
    assert_eq!(fault(Extract { path: vec![2], param_slot: 7 }), Decline::BadExtract(ExtractFault::PathUnresolved), "step past the operands");
    assert_eq!(fault(Extract { path: vec![1, 0], param_slot: 7 }), Decline::BadExtract(ExtractFault::PathUnresolved), "step through a leaf");

    // read side: tamper with the slot of a valid record (the last 4 bytes) and the path step
    let mut slot = bytes.clone();
    let n = slot.len();
    slot[n - 4] = 6;
    assert_eq!(decode(&slot), Err(DecodeDecline::BadExtract(ExtractFault::SlotMismatch { leaf_index: 7 })));
    let mut step = bytes.clone();
    step[n - 6] = 0; // path step 1 -> 0: the Bind
    assert_eq!(decode(&step), Err(DecodeDecline::BadExtract(ExtractFault::NotParamLeaf)));
    step[n - 6] = 9;
    assert_eq!(decode(&step), Err(DecodeDecline::BadExtract(ExtractFault::PathUnresolved)));

    // a shared leaf has one canonical path: the lexicographically-least
    let shared = Region::new(0, "1", "1", Node::op("select", vec![
        Node::Source(SourceLeaf::Param(3)),
        Node::Source(SourceLeaf::Param(3)),
    ]));
    assert_eq!(param_leaf_path(&shared, 3), Some(vec![0]));
}
