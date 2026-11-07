// SPDX-License-Identifier: MIT OR Apache-2.0
//! Interface `output_layouts` (KISS-Contract §6.5-0014 to §6.5-0019, board 378): the optional
//! trailing field, the seven closed forms, their canonical text, and the typed declines.
//!
//! Every arm drives the SHIPPED reader (`kiss_conformance::layout`, and the Interface block reader
//! in `kiss_conformance::contract`), never a test-side parser.

use kiss_conformance::contract::{parse_interface_block, render_block, SectionDecline, Value};
use kiss_conformance::dispatch_expr::LaunchScalars;
use kiss_conformance::layout::{
    parse_layout, parse_output_layouts, Hier, IntTuple, Layout, LayoutContext, LayoutDecline,
    OutputLayout,
};

fn scalars() -> LaunchScalars {
    LaunchScalars {
        rank: 2,
        n: 12,
        extents: vec![vec![3, 4], vec![3, 4]],
        strides: vec![vec![4, 1], vec![4, 1]],
        idx_extents: vec![],
        off: vec![0, 0],
        ws_bytes: None,
        param: vec![],
    }
}

fn ctx(s: &LaunchScalars, n_inputs: u32, n_outputs: usize) -> LayoutContext<'_> {
    LayoutContext { n_inputs, n_outputs, scalars: s }
}

fn one(text: &str) -> Result<OutputLayout, LayoutDecline> {
    let s = scalars();
    parse_layout(text, &ctx(&s, 2, 1))
}

fn leaf(v: u64) -> IntTuple {
    IntTuple::Leaf(v)
}

fn tup(v: Vec<IntTuple>) -> IntTuple {
    IntTuple::Tup(v)
}

fn interface(extra: &[(&str, &str)]) -> Vec<u8> {
    let mut f: Vec<(&str, &str)> = vec![
        ("entry_point", "k"),
        ("rank", "2"),
        ("positional_signature", "[]"),
        ("launch_scalars", "[]"),
        ("count_unit", "elements"),
        ("in_place", "none"),
        ("alignment_bytes", "16"),
    ];
    f.extend_from_slice(extra);
    let vals: Vec<(&str, Value)> = f.iter().map(|(k, v)| (*k, Value::Str((*v).to_string()))).collect();
    render_block(3, "interface", &vals)
}

// Backs[enforced]: KISS-CONTRACT-6.5-0014 — `output_layouts` is optional and pinned last in the Interface block (absent parses, present parses, any other position declines) and its element count must equal the output-operand count
#[test]
fn test_contract_output_layouts_field() {
    // absent: the seven required fields only — still parses (no existing contract moves).
    assert_eq!(parse_interface_block(&interface(&[])).map(|f| f.len()), Ok(7));
    // present and last: parses with eight fields.
    let with = interface(&[("output_layouts", "[dense, -]")]);
    assert_eq!(parse_interface_block(&with).map(|f| f.len()), Ok(8));
    // present but NOT last: the pinned position is part of the schema.
    let mut early = interface(&[]);
    let text = String::from_utf8(early.clone()).unwrap();
    early = text.replace("rank = 2\n", "rank = 2\noutput_layouts = [dense]\n").into_bytes();
    assert_eq!(parse_interface_block(&early), Err(SectionDecline::WrongOrder));
    // an unknown extra field still declines (the optional list is exactly one name).
    let unknown = interface(&[("output_layoutz", "[dense]")]);
    assert_eq!(
        parse_interface_block(&unknown),
        Err(SectionDecline::UnknownField("output_layoutz".into()))
    );
    // count: one element per output operand; a mismatch declines, never pads or truncates.
    let s = scalars();
    assert_eq!(
        parse_output_layouts("[dense, -]", &ctx(&s, 2, 2)),
        Ok(vec![OutputLayout::Guarantee(Layout::Dense), OutputLayout::Supplied])
    );
    assert_eq!(
        parse_output_layouts("[dense]", &ctx(&s, 2, 2)),
        Err(LayoutDecline::CountMismatch { want: 2, got: 1 })
    );
    assert_eq!(
        parse_output_layouts("[dense, dense, dense]", &ctx(&s, 2, 2)),
        Err(LayoutDecline::CountMismatch { want: 2, got: 3 })
    );
    assert_eq!(parse_output_layouts("dense", &ctx(&s, 2, 1)), Err(LayoutDecline::NotAnArray));
}

// Backs[enforced]: KISS-CONTRACT-6.5-0015 — exactly seven forms parse; an unknown form refuses the guarantee (not the contract); `-` is the supplied layout
#[test]
fn test_contract_output_layout_forms_closed() {
    let heads = [
        ("dense", Layout::Dense),
        ("dense_order(1,0)", Layout::DenseOrder(vec![1, 0])),
        ("same_as(1)", Layout::SameAs(1)),
        ("none", Layout::None),
    ];
    for (text, want) in heads {
        assert_eq!(one(text), Ok(OutputLayout::Guarantee(want)), "{text}");
    }
    assert!(matches!(one("strided(4,1)"), Ok(OutputLayout::Guarantee(Layout::Strided(_)))));
    assert!(matches!(one("(2,3):(1,2)"), Ok(OutputLayout::Guarantee(Layout::Hier(_)))));
    assert!(matches!(one("swz(2,3,-3)(8:1)"), Ok(OutputLayout::Guarantee(Layout::Hier(_)))));
    assert!(matches!(one("bundle(a=dense@0)"), Ok(OutputLayout::Guarantee(Layout::Bundle(_)))));
    assert_eq!(one("-"), Ok(OutputLayout::Supplied));
    // an unknown form is a refusal of the GUARANTEE, recorded, never a whole-field decline.
    assert_eq!(one("tiled(4)"), Ok(OutputLayout::Refused("tiled(4)".into())));
    assert_eq!(one("Dense"), Ok(OutputLayout::Refused("Dense".into())));
    // a KNOWN form that is malformed is a decline, not a refusal.
    assert!(matches!(one("dense_order(1"), Err(_)));
}

// Backs[enforced]: KISS-CONTRACT-6.5-0016 — canonical text (no space, no leading zero or plus), the permutation/rank/input-index checks, and §6.6-0006 expressions in `strided`
#[test]
fn test_contract_output_layout_simple_forms() {
    for bad in ["dense_order(1, 0)", "dense_order(01,0)", "dense_order(+1,0)", "same_as(+1)"] {
        assert!(matches!(one(bad), Err(LayoutDecline::NonCanonical(_))), "{bad}");
    }
    assert_eq!(one("dense_order(0,0)"), Err(LayoutDecline::BadAxisList));
    assert_eq!(one("dense_order(0)"), Err(LayoutDecline::BadAxisList));
    assert_eq!(one("dense_order(2,0)"), Err(LayoutDecline::BadAxisList));
    assert_eq!(one("same_as(2)"), Err(LayoutDecline::SameAsOutOfRange(2)));
    assert_eq!(
        one("strided(strides0[0],1)"),
        Ok(OutputLayout::Guarantee(Layout::Strided(vec!["strides0[0]".into(), "1".into()])))
    );
    assert_eq!(one("strided(4)"), Err(LayoutDecline::BadAxisList));
    assert!(matches!(one("strided(nosuch,1)"), Err(LayoutDecline::BadExpression(_))));
    assert!(matches!(one("strided(strides0,1)"), Err(LayoutDecline::BadExpression(_))));
}

// Backs[enforced]: KISS-CONTRACT-6.5-0017 — a nested layout is integer-only, congruent, shape >= 1, stride >= 0, depth <= 16; a run-time symbol in a leaf declines
#[test]
fn test_contract_output_layout_hier() {
    let want = Hier {
        swizzle: None,
        shape: tup(vec![tup(vec![leaf(2), leaf(3)]), leaf(4)]),
        stride: tup(vec![tup(vec![leaf(1), leaf(2)]), leaf(6)]),
    };
    assert_eq!(one("((2,3),4):((1,2),6)"), Ok(OutputLayout::Guarantee(Layout::Hier(want))));
    // a flat broadcast (0-stride) leaf is a valid stride.
    assert!(matches!(one("(4,8):(1,0)"), Ok(_)));
    assert_eq!(one("(2,3):(1,(2,3))"), Err(LayoutDecline::NotCongruent));
    assert_eq!(one("(0,3):(1,2)"), Err(LayoutDecline::ZeroExtent));
    assert!(matches!(one("(2,3):(1,-2)"), Err(LayoutDecline::NonCanonical(_))));
    assert!(matches!(one("(2,3):(1, 2)"), Err(LayoutDecline::NonCanonical(_))));
    assert!(matches!(one("(2,03):(1,2)"), Err(LayoutDecline::NonCanonical(_))));
    assert!(matches!(one("()):(1)"), Err(_)));
    assert!(matches!(one("(2,3):(1,lda)"), Err(LayoutDecline::NonCanonical(_))));
    // depth: 16 nested tuples pass, 17 decline (Unpopped MAX_DEPTH = 16).
    let nest = |d: usize| format!("{}1{}", "(".repeat(d), ")".repeat(d));
    assert!(matches!(one(&format!("{}:{}", nest(16), nest(16))), Ok(_)));
    assert_eq!(one(&format!("{}:{}", nest(17), nest(17))), Err(LayoutDecline::TooDeep));
}

// Backs[enforced]: KISS-CONTRACT-6.5-0018 — `swz(B,M,S)(shape:stride)` round-trips B, M and a signed S into the parsed layout
#[test]
fn test_contract_output_layout_swizzle() {
    let got = one("swz(2,3,-3)((8,8):(8,1))");
    let want = Hier {
        swizzle: Some((2, 3, -3)),
        shape: tup(vec![leaf(8), leaf(8)]),
        stride: tup(vec![leaf(8), leaf(1)]),
    };
    assert_eq!(got, Ok(OutputLayout::Guarantee(Layout::Hier(want))));
    assert!(matches!(one("swz(2,3,3)(8:1)"), Ok(OutputLayout::Guarantee(Layout::Hier(Hier { swizzle: Some((2, 3, 3)), .. })))));
    assert!(matches!(one("swz(2,3)(8:1)"), Err(LayoutDecline::Malformed(_))));
    assert!(matches!(one("swz(-2,3,3)(8:1)"), Err(LayoutDecline::NonCanonical(_))));
    assert!(matches!(one("swz(2,3,-0)(8:1)"), Err(LayoutDecline::NonCanonical(_))));
    // the inner layout obeys 6.5-0017.
    assert_eq!(one("swz(2,3,3)((8,8):(8,(1,2)))"), Err(LayoutDecline::NotCongruent));
}

// Backs[enforced]: KISS-CONTRACT-6.5-0019 — a bundle is named slots `name=layout@offset`, unique names, no nested bundle or none, offsets are §6.6-0006 expressions
#[test]
fn test_contract_output_layout_bundle() {
    let ok = one("bundle(a=dense@0,b=dense_order(1,0)@ceil_div(n*4,16)*16)");
    match ok {
        Ok(OutputLayout::Guarantee(Layout::Bundle(slots))) => {
            assert_eq!(slots.len(), 2);
            assert_eq!(slots[1].name, "b");
            assert_eq!(slots[1].offset, "ceil_div(n*4,16)*16");
            assert_eq!(slots[1].layout, Layout::DenseOrder(vec![1, 0]));
        }
        other => panic!("bundle did not parse: {other:?}"),
    }
    assert_eq!(one("bundle(a=dense@0,a=dense@8)"), Err(LayoutDecline::DuplicateSlot("a".into())));
    assert_eq!(one("bundle(a=bundle(b=dense@0)@0)"), Err(LayoutDecline::NestedBundle));
    assert_eq!(one("bundle(a=none@0)"), Err(LayoutDecline::NestedBundle));
    assert!(matches!(one("bundle(a=dense@nosuch)"), Err(LayoutDecline::BadExpression(_))));
    assert!(matches!(one("bundle(A=dense@0)"), Err(LayoutDecline::NonCanonical(_))));
    assert!(matches!(one("bundle(a=dense)"), Err(LayoutDecline::Malformed(_))));
}
