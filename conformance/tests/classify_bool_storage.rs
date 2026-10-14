// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY §6.1-0007 (`bool` storage: one byte, `0x00` false, any non-zero byte reads as true,
//! canonical true `0x01`, width equal to `u8`), plus the length-mismatch decline that bounds §6.3-0002's
//! "only `extents[0..rank]` is meaningful" in the reference deriver.
//!
//! The truth-encoding half is exercised through the Ops comparison/bool corpus (`ops-compare-bool.json`,
//! KISS-OPS-6.2-0005/-0006): for every `logical_*` cell the EXPECTED byte is recomputed here from "any
//! non-zero byte is true" and compared, so the pinned encoding is checked against vector content and not
//! against the minter's own logic. The normalization obligation itself is Ops' and is not claimed here.

use kiss_conformance::derive::{derive_structure_key, CellOperand, DeriveDecline, DerivationCell};
use kiss_conformance::json::{parse, Json};
use kiss_conformance::structure_key::{MathFidelity, OperandUse::{Input, Output}};

const MANIFEST: &str = include_str!("../corpus/dtype_manifest.json");
const COMPARE_BOOL: &str = include_str!("../corpus/ops-compare-bool.json");

fn bits(row: &Json, key: &str) -> u64 {
    row.get(key).and_then(Json::as_u64).expect(key)
}

fn hex_byte(s: &str) -> u8 {
    u8::from_str_radix(s.trim(), 16).expect("one hex byte")
}

// Backs: KISS-CLASSIFY-6.1-0007 — bool is exactly one byte, its width equals u8's, and the truth encoding
// is `0x00` false / any non-zero true with canonical true `0x01`: every logical cell of the bool corpus
// (operands include 0x02, 0x7F, 0x80, 0xFF) has the expected byte the encoding dictates.
#[test]
fn test_classify_bool_encoding() {
    let doc = parse(MANIFEST).expect("manifest parses");
    let rows = doc.get("dtypes").and_then(Json::as_arr).expect("dtypes");
    let row = |t: &str| rows.iter().find(|r| r.get("token").and_then(Json::as_str) == Some(t)).expect(t);
    assert_eq!(bits(row("bool"), "storage_bits"), 8, "exactly one byte");
    assert_eq!(bits(row("bool"), "storage_bits"), bits(row("u8"), "storage_bits"), "width equals u8");

    let corpus = parse(COMPARE_BOOL).expect("corpus parses");
    let vs = corpus.get("vectors").and_then(Json::as_arr).expect("vectors");
    let (mut checked, mut non_canonical_true) = (0, 0);
    for v in vs {
        let op = v.get("op").and_then(Json::as_str).expect("op");
        if !op.starts_with("logical_") {
            continue;
        }
        let ins: Vec<u8> = v
            .get("inputs")
            .and_then(Json::as_arr)
            .expect("inputs")
            .iter()
            .map(|i| hex_byte(i.get("bits").and_then(Json::as_str).expect("bits")))
            .collect();
        let want = match (op, ins.as_slice()) {
            ("logical_and", [a, b]) => u8::from(*a != 0 && *b != 0),
            ("logical_or", [a, b]) => u8::from(*a != 0 || *b != 0),
            ("logical_not", [a]) => u8::from(*a == 0),
            other => panic!("unexpected cell shape {other:?}"),
        };
        let got = hex_byte(v.get("expected").and_then(|e| e.get("bits")).and_then(Json::as_str).expect("expected"));
        assert_eq!(got, want, "{op} {ins:02X?}: a produced bool is 0x00 or the canonical 0x01");
        if ins.iter().any(|&b| b > 1) {
            non_canonical_true += 1;
        }
        checked += 1;
    }
    assert!(checked >= 20, "logical cells checked: {checked}");
    assert!(non_canonical_true >= 4, "cells with a non-canonical true operand: {non_canonical_true}");
}

// The reference deriver refuses an operand whose extents and strides disagree in length, so nothing past the
// declared rank is ever read (the §6.3-0002 clause is otherwise unrepresentable here: Vec, not a padded array).
#[test]
fn derive_declines_an_extents_strides_length_mismatch() {
    let operand = |e: &[i64], s: &[i64]| CellOperand {
        usage: Input,
        dtype: "f32",
        extents: e.to_vec(),
        strides: s.to_vec(),
        alignment: 256,
        dequant: None,
    };
    let mut out = operand(&[4, 8], &[8, 1]);
    out.usage = Output;
    let ok = DerivationCell {
        target: "cuda:sm89",
        operands: vec![operand(&[4, 8], &[8, 1]), out.clone()],
        index_operand: None,
        reduced_axes: vec![],
        accumulator: None,
        math_fidelity: MathFidelity::Stable,
        contraction_declared: false,
    };
    assert!(derive_structure_key(&ok).is_ok(), "control: the well-formed cell derives");
    let mut bad = ok.clone();
    bad.operands[0] = operand(&[4, 8], &[8, 1, 1]);
    assert_eq!(derive_structure_key(&bad).unwrap_err(), DeriveDecline::BadShape);
}
