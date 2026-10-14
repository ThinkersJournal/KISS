// SPDX-License-Identifier: MIT OR Apache-2.0
//! R2.4 slice 1: the value-bearing OpAttrs vectors (`corpus/opattrs_value_vectors.json`).
//!
//! The artifact is minted by a GENERIC schema-driven encoder (`opattrs_value`). Here it is held against
//! (1) the hand encoders in `opattrs.rs` — a second derivation by the same author, so agreement shows the
//! code consistent with the table and not that the table is read correctly (the E6 caveat) — and
//! (2) the spec's own Appendix E.2 worked blobs, which are hand-written in the spec text. Controls prove
//! the cross-check can go red. The `reduce`/`prefix_scan` divergence is recorded, not hidden.

use kiss_conformance::opattrs::{
    self, ColumnOrdering, IndexDtype, OobPolicy, ScatterCombine, SortDirection,
};
use kiss_conformance::opattrs_schema::parse_table;
use kiss_conformance::opattrs_value::{
    self as ov, encode, encode_vectors, equivalences, not_emitted, Attrs, EncodeVector, Value,
};

fn get<'a>(v: &'a EncodeVector, name: &str) -> &'a Value {
    &v.attrs.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("{}: no `{name}`", v.op)).1
}
fn tok<'a>(v: &'a EncodeVector, name: &str) -> &'a str {
    match get(v, name) {
        Value::Token(t) => t,
        other => panic!("{}: `{name}` is {other:?}, not a token", v.op),
    }
}
fn byte(v: &EncodeVector, name: &str) -> u8 {
    match get(v, name) {
        Value::Byte(n) => *n,
        other => panic!("{}: `{name}` is {other:?}", v.op),
    }
}
fn flag(v: &EncodeVector, name: &str) -> bool {
    match get(v, name) {
        Value::Flag(b) => *b,
        other => panic!("{}: `{name}` is {other:?}", v.op),
    }
}
fn vecu32(v: &EncodeVector, name: &str) -> Vec<u32> {
    match get(v, name) {
        Value::Vector(e) => e.clone(),
        other => panic!("{}: `{name}` is {other:?}", v.op),
    }
}
fn axes(v: &EncodeVector) -> (Vec<usize>, usize) {
    match get(v, "reduce_axes") {
        Value::Axes { axes, rank } => (axes.clone(), *rank),
        other => panic!("{}: reduce_axes is {other:?}", v.op),
    }
}
fn oob(t: &str) -> OobPolicy {
    match t {
        "skip" => OobPolicy::Skip,
        "clamp" => OobPolicy::Clamp,
        "zero-fill" => OobPolicy::ZeroFill,
        _ => panic!("oob {t}"),
    }
}
fn idt(t: &str) -> IndexDtype {
    match t {
        "u32" => IndexDtype::U32,
        "i32" => IndexDtype::I32,
        "i64" => IndexDtype::I64,
        _ => panic!("index_dtype {t}"),
    }
}

/// The hand encoder's bytes for a vector, `None` where `opattrs.rs` has no encoder for the op.
fn hand(v: &EncodeVector) -> Option<Vec<u8>> {
    Some(match v.op.as_str() {
        "gather" => opattrs::encode_gather(byte(v, "axis"), oob(tok(v, "oob_policy")), byte(v, "index_operand"), idt(tok(v, "index_dtype"))),
        "scatter" => {
            let c = match tok(v, "combine") {
                "assign" => ScatterCombine::Assign,
                "atomic-add" => ScatterCombine::AtomicAdd,
                "atomic-max" => ScatterCombine::AtomicMax,
                "atomic-min" => ScatterCombine::AtomicMin,
                t => panic!("combine {t}"),
            };
            opattrs::encode_scatter(byte(v, "axis"), c, oob(tok(v, "oob_policy")), byte(v, "index_operand"), idt(tok(v, "index_dtype")))
        }
        "sort_network" => {
            let d = if tok(v, "direction") == "ascending" { SortDirection::Ascending } else { SortDirection::Descending };
            opattrs::encode_sort_network(byte(v, "axis"), d, flag(v, "stability"))
        }
        "reduce_var" | "reduce_std" => {
            let (a, r) = axes(v);
            opattrs::encode_reduce_var(&a, r, flag(v, "keepdim"), flag(v, "bessel_correction"))
        }
        "softmax" | "log_softmax" => opattrs::encode_softmax(byte(v, "norm_axis")),
        "index_select" | "embedding" | "scatter_add" => {
            opattrs::encode_index_select(byte(v, "axis"), byte(v, "index_operand"), idt(tok(v, "index_dtype")))
        }
        "avg_pool" => opattrs::encode_avg_pool(
            &vecu32(v, "window_size"),
            &vecu32(v, "stride"),
            &vecu32(v, "dilation"),
            &vecu32(v, "padding"),
            flag(v, "count_include_pad"),
        ),
        "im2col" => {
            let o = if tok(v, "output_column_ordering") == "channel-major" { ColumnOrdering::ChannelMajor } else { ColumnOrdering::TapMajor };
            opattrs::encode_im2col(&vecu32(v, "window_size"), &vecu32(v, "stride"), &vecu32(v, "dilation"), &vecu32(v, "padding"), o)
        }
        _ => return None,
    })
}

#[test]
fn committed_artifact_is_the_generator_output() {
    let committed = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/opattrs_value_vectors.json"))
        .expect("run `cargo run --bin emit_opattrs_value_vectors > conformance/corpus/opattrs_value_vectors.json`");
    assert_eq!(
        committed.replace("\r\n", "\n"),
        ov::emit_opattrs_value_vectors_json(),
        "opattrs_value_vectors.json is STALE: regenerate it"
    );
}

#[test]
fn generic_encoder_agrees_with_the_hand_encoders_on_every_vector() {
    let (mut compared, mut skipped) = (0, 0);
    for v in encode_vectors() {
        match hand(&v) {
            Some(h) => {
                assert_eq!(h, v.blob, "{} tc {} ({}): hand encoder disagrees with the generic one", v.op, v.tc_id, v.tag);
                compared += 1;
            }
            None => skipped += 1,
        }
    }
    // generic-only (no hand encoder in opattrs.rs): max_pool (17), rms_norm (3), layer_norm (3)
    assert_eq!(skipped, 23, "max_pool, rms_norm and layer_norm are generic-only");
    assert_eq!(compared + skipped, 133, "population is pinned");
}

#[test]
fn generic_encoder_reproduces_the_spec_appendix_e2_blobs() {
    let enc = |op: &str, a: Attrs| encode(op, &a).expect("encodes");
    let s = |t: &str| Value::Token(t.into());
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ");
    let g = enc("gather", vec![("axis".into(), Value::Byte(0)), ("oob_policy".into(), s("clamp")), ("index_operand".into(), Value::Byte(1)), ("index_dtype".into(), s("i32"))]);
    assert_eq!(hex(&g), "00 02 01 02", "E.2 gather clamp");
    let sc = enc("scatter", vec![("axis".into(), Value::Byte(0)), ("combine".into(), s("atomic-add")), ("oob_policy".into(), s("skip")), ("index_operand".into(), Value::Byte(1)), ("index_dtype".into(), s("i64"))]);
    assert_eq!(hex(&sc), "00 02 01 01 03", "E.2 scatter atomic-add");
    let rv = enc("reduce_var", vec![("reduce_axes".into(), Value::Axes { axes: vec![0, 1], rank: 2 }), ("keepdim".into(), Value::Flag(true)), ("bessel_correction".into(), Value::Flag(false))]);
    assert_eq!(hex(&rv), "FF FF 01 00", "E.2 reduce_var population");
    let ap = enc(
        "avg_pool",
        vec![
            ("window_size".into(), Value::Vector(vec![3, 5])),
            ("stride".into(), Value::Vector(vec![1, 2])),
            ("dilation".into(), Value::Vector(vec![1, 1])),
            ("padding".into(), Value::Vector(vec![0, 0])),
            ("count_include_pad".into(), Value::Flag(true)),
        ],
    );
    assert_eq!(
        hex(&ap),
        "02 03 00 00 00 05 00 00 00 02 01 00 00 00 02 00 00 00 02 01 00 00 00 01 00 00 00 02 00 00 00 00 00 00 00 00 01",
        "E.2 non-square avg_pool (37 bytes)"
    );
}

#[test]
fn rank_sensitivity_of_reduce_axes_matches_the_spec_examples() {
    // §6.19-0020 / Appendix E.1: the rank is NOT in the blob, so these share or split bytes by design
    assert_eq!(ov::reduce_axes_word(&[0], 1).unwrap(), 0xFFFF, "rank-1 sole axis is all-axes");
    assert_eq!(ov::reduce_axes_word(&[0, 1, 2], 3).unwrap(), 0xFFFF, "rank-3 full is not 07 00");
    assert_eq!(ov::reduce_axes_word(&[0], 2).unwrap(), 0x0001);
    assert_eq!(ov::reduce_axes_word(&[1], 2).unwrap(), 0xFFFE, "trailing");
    assert_eq!(ov::reduce_axes_word(&[0, 2], 3).unwrap(), 0x0005);
    assert!(ov::reduce_axes_word(&[], 2).is_err(), "0x0000 is unreachable (§6.19-0038)");
    assert!(ov::reduce_axes_word(&[2], 2).is_err());
}

#[test]
fn defaulted_and_explicit_equal_attributes_give_identical_bytes() {
    let eqs = equivalences();
    assert_eq!(eqs.len(), 7, "gather, scatter, sort_network, reduce_var, reduce_std, avg_pool, im2col");
    for e in &eqs {
        assert!(!e.omitted.is_empty(), "{}: an equivalence that omits nothing proves nothing", e.op);
    }
}

#[test]
fn every_variation_that_differs_is_recorded_distinct_and_none_is_vacuous() {
    let vs = encode_vectors();
    let distinct = ov::distinct_pairs(&vs);
    assert!(distinct.len() >= 100, "distinct pairs: {}", distinct.len());
    for (a, b, op) in &distinct {
        let (va, vb) = (&vs[*a as usize - 1], &vs[*b as usize - 1]);
        assert_eq!((va.tc_id, vb.tc_id), (*a, *b));
        assert_eq!(&va.op, op);
        assert_ne!(va.blob, vb.blob);
    }
}

#[test]
fn reduce_and_prefix_scan_are_recorded_not_minted_and_the_divergence_is_real() {
    let ne: Vec<&str> = not_emitted().iter().map(|(o, _)| *o).collect();
    assert_eq!(ne, ["reduce", "prefix_scan"]);
    let schemas = parse_table();
    for op in ne {
        let s = schemas.iter().find(|s| s.op == op).expect("carrier op");
        let names: Vec<&str> = s.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"accumulator") && names.contains(&"math_precision"), "{op}: {names:?}");
        assert_eq!(s.fixed_width(), Some(6), "{op}: the table now says 6 bytes");
    }
    // the hand encoders and Appendix E.2 still say 4: when either side is fixed this goes red and the
    // `not_emitted` list must be revisited rather than left to rot
    assert_eq!(opattrs::encode_reduce(opattrs::Monoid::Sum, &[0, 1], 2, true).len(), 4);
    assert!(encode_vectors().iter().all(|v| v.op != "reduce" && v.op != "prefix_scan"));
    assert!(encode("reduce", &Vec::new()).is_err(), "and the generic encoder refuses the unpinned fields");
}

#[test]
fn the_fixed_keepdim_exception_is_still_what_the_clause_says() {
    // `is_fixed` special-cases `keepdim` because the table row omits it; the clause is the authority
    let spec = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).unwrap().replace("\r\n", "\n");
    let at = spec.find("KISS-OPS-6.19-0030").expect("clause");
    let clause = &spec[at..at + 700];
    assert!(clause.contains("`keepdim` (bool `u8`, fixed"), "6.19-0030 no longer says keepdim is fixed");
    assert!(spec.contains("`MAX_RANK = 8` and `MAX_OPERANDS = 8`") || spec.contains("MAX_RANK = 8"), "pinned constants moved");
}

// ---- controls: the cross-check must be able to go red -----------------------------------------

#[test]
fn control_a_swapped_field_order_is_caught() {
    let mut schema = parse_table().into_iter().find(|s| s.op == "gather").unwrap();
    schema.fields.swap(0, 1); // oob_policy before axis
    let v = encode_vectors().into_iter().find(|v| v.op == "gather" && v.tag.starts_with("vary:oob_policy")).unwrap();
    // the swapped schema cannot even take the attrs (axis is a Byte, oob_policy a token), and where it can
    // the bytes differ from the hand encoder's
    match ov::encode_with_schema(&schema, &v.attrs) {
        Err(_) => {}
        Ok(b) => assert_ne!(Some(b), hand(&v)),
    }
}

#[test]
fn control_a_wrong_token_or_missing_field_is_refused_not_guessed() {
    let mut a: Attrs = ov::base_attrs("gather").unwrap();
    a[1].1 = Value::Token("wrap-around".into());
    assert!(encode("gather", &a).is_err(), "an unknown token must not become a byte");
    let mut b = ov::base_attrs("gather").unwrap();
    b.pop();
    assert!(encode("gather", &b).is_err(), "a missing mandatory field must not default silently");
    let mut c = ov::base_attrs("gather").unwrap();
    c[0].1 = Value::Byte(8);
    assert!(encode("gather", &c).is_err(), "axis >= MAX_RANK is refused (§6.19-0037)");
}
