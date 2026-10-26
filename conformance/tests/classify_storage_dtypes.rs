// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY §6.1-0005 (dtypes are pure storage: no precision-bearing token, no precision field) and
//! §6.1-0006 (`u32` is an ordinary storage dtype, legal on the same terms as any other).
//!
//! Subject: the reference deriver ([`kiss_conformance::derive`]) and the vendored `dtype_manifest.json`.
//! Scope, stated: these show the REFERENCE keyer treats the dtypes as the clauses require. They say nothing
//! about another implementation's dispatch, which is what a second deriver (R3.3) would add.

use kiss_conformance::derive::{derive_structure_key, CellOperand, DeriveDecline, DerivationCell};
use kiss_conformance::json::{parse, Json};
use kiss_conformance::structure_key::{MathFidelity, OperandUse::{Input, Output}, DTYPES};

const MANIFEST: &str = include_str!("../corpus/dtype_manifest.json");
const CUDA: &str = "cuda:sm89";

fn op(usage: kiss_conformance::structure_key::OperandUse, dtype: &'static str, e: &[i64], s: &[i64]) -> CellOperand {
    CellOperand { usage, dtype, extents: e.to_vec(), strides: s.to_vec(), alignment: 256, dequant: None }
}

fn cell(operands: Vec<CellOperand>) -> DerivationCell {
    DerivationCell {
        target: CUDA,
        operands,
        index_operand: None,
        reduced_axes: vec![],
        accumulator: None,
        math_fidelity: MathFidelity::Stable,
        contraction: None,
        vec_cap_bytes: None,
    }
}

fn token(c: &DerivationCell) -> String {
    derive_structure_key(c).unwrap_or_else(|e| panic!("declined: {e:?}")).to_token()
}

fn add(dtype: &'static str) -> DerivationCell {
    cell(vec![
        op(Input, dtype, &[128, 256], &[256, 1]),
        op(Input, dtype, &[128, 256], &[256, 1]),
        op(Output, dtype, &[128, 256], &[256, 1]),
    ])
}

fn reduce(dtype: &'static str, fidelity: MathFidelity) -> DerivationCell {
    let mut c = cell(vec![op(Input, dtype, &[4, 8], &[8, 1]), op(Output, dtype, &[4, 1], &[1, 1])]);
    c.reduced_axes = vec![1];
    c.math_fidelity = fidelity;
    c
}

/// The dtype component of a token (`sk6|<class>|<dtype>|...`).
fn dtype_of(token: &str) -> &str {
    token.split('|').nth(2).expect("dtype component")
}

// Backs: KISS-CLASSIFY-6.1-0005 — a dtype pins byte layout only: no strict-precision float token exists
// (the closed set of 27 contains none; an `f32s`-style token declines as unknown), the dtype record carries
// no precision field, and compute precision rides the separate `<mp>` attribute, never the dtype token.
#[test]
fn test_classify_dtypes_are_pure_storage() {
    // the record: exactly {token, kind, storage_bits, reserved}
    let doc = parse(MANIFEST).expect("manifest parses");
    let rows = doc.get("dtypes").and_then(Json::as_arr).expect("dtypes");
    assert_eq!(rows.len(), 27);
    for r in rows {
        let Json::Obj(fields) = r else { panic!("row is not an object") };
        let mut keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["kind", "reserved", "storage_bits", "token"], "a dtype row carries no precision field");
    }
    // the closed set has no precision-bearing spelling, and one is refused as unknown (not reserved)
    assert_eq!(DTYPES.len(), 27);
    for bad in ["f32s", "f32_strict", "f32-strict", "f32st", "f32x", "tf32"] {
        assert!(!DTYPES.contains(&bad), "`{bad}` must not be a dtype token");
        assert_eq!(derive_structure_key(&add(bad)).unwrap_err(), DeriveDecline::UnknownDtype, "{bad}");
    }
    // precision is the `<mp>` attribute: it changes the reduction token and leaves the dtype token alone
    let stable = token(&reduce("f32", MathFidelity::Stable));
    let reduced = token(&reduce("f32", MathFidelity::ReducedMantissa));
    assert_ne!(stable, reduced, "a reduced-mantissa request must be visible in the key");
    assert_eq!(dtype_of(&stable), "f32");
    assert_eq!(dtype_of(&reduced), "f32", "the dtype token is the same storage dtype either way");
}

// Backs: KISS-CLASSIFY-6.1-0006 — u32 is an unsigned 32-bit ordinary storage dtype: 4-byte container, kind
// `uint` like u8/u16/u64, derivable as the compute dtype of elementwise and reduction cells, as a data
// operand beside an index operand of another dtype, and as an index operand.
#[test]
fn test_classify_u32_is_ordinary_storage() {
    let doc = parse(MANIFEST).expect("manifest parses");
    let rows = doc.get("dtypes").and_then(Json::as_arr).expect("dtypes");
    let row = |t: &str| rows.iter().find(|r| r.get("token").and_then(Json::as_str) == Some(t)).expect(t);
    assert_eq!(row("u32").get("storage_bits").and_then(Json::as_u64), Some(32), "4-byte container");
    for t in ["u8", "u16", "u64"] {
        assert_eq!(row("u32").get("kind"), row(t).get("kind"), "u32 is the same kind as {t}");
    }
    assert_eq!(row("u32").get("reserved").and_then(|j| if let Json::Bool(b) = j { Some(*b) } else { None }), Some(false));

    // elementwise: u32 derives, and differs from f32 only in the dtype component (same 4-byte width)
    let (tu, tf) = (token(&add("u32")), token(&add("f32")));
    assert_eq!(dtype_of(&tu), "u32");
    assert_eq!(tu.replacen("|u32|", "|f32|", 1).replace("/u32", "/f32"), tf, "no u32-specific path: only the dtype differs");
    // reduction on the arithmetic path: not rejected "because it is an index type"
    let red = token(&reduce("u32", MathFidelity::Stable));
    assert_eq!(dtype_of(&red), "u32");
    // u32 as DATA beside an i32 index operand, and u32 as the index operand itself
    let mut data = cell(vec![
        op(Input, "u32", &[1000, 64], &[64, 1]),
        op(Input, "i32", &[32], &[1]),
        op(Output, "u32", &[32, 64], &[64, 1]),
    ]);
    data.index_operand = Some(1);
    assert!(token(&data).contains("|u32|"), "u32 data with an i32 index");
    let mut idx = cell(vec![
        op(Input, "f32", &[1000, 64], &[64, 1]),
        op(Input, "u32", &[32], &[1]),
        op(Output, "f32", &[32, 64], &[64, 1]),
    ]);
    idx.index_operand = Some(1);
    assert!(token(&idx).contains("|ndx|"), "u32 as the index operand");
    // and it is the CALLER's declared slot that makes it an index, not the dtype: with no slot declared the
    // same u32 operand is not treated as one
    idx.index_operand = None;
    assert!(!token(&idx).contains("|ndx|"), "the index role is a caller hint, not a dtype class (6.6-0012)");
}

// ---- control: the comparisons above can fail ----------------------------------------------------

// harness-internal: negative control showing the sibling tests' manifest-shape and u8/u32 comparisons can fail; checks the test machinery only.
#[test]
fn control_a_precision_field_or_an_index_only_dtype_would_be_caught() {
    // a row with an extra field would not match the pinned four-key shape
    let with_extra = parse(r#"{"token":"f32","kind":"float","storage_bits":32,"reserved":false,"precision":"strict"}"#).unwrap();
    let Json::Obj(fields) = with_extra else { unreachable!() };
    let mut keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
    keys.sort_unstable();
    assert_ne!(keys, ["kind", "reserved", "storage_bits", "token"]);
    // u8 derives like u32 does; a deriver that special-cased u32 would split these
    assert_eq!(dtype_of(&token(&add("u8"))), "u8");
}
