// SPDX-License-Identifier: MIT OR Apache-2.0
//! Per-field TYPE table of every op's OpAttrs record (KISS-OPS-6.19-0049 fields, §6.19.2 vocabulary, §6.19.3 layout),
//! so a second reader can decide a bad `op_identity` value (KISS-CONTRACT-6.11-0009, tag `op-identity-bad-value`)
//! without keeping its own table. Everything is read from `spec/ops.md` by [`parse_table`] and [`vocab`]; the one
//! transcribed fact is `vocab_of` (field name -> sub-vocabulary), already pinned by the hand-encoder cross-check.
//!
//! The build-time agreement discipline: [`emit_opattrs_field_types_json`] states each field's kind, vocabulary and
//! accepted tokens, and then asks the reference reader ([`field_value`], the `op_identity` value check) about every
//! one of them. A stated fact the reader disagrees with fails the build instead of being emitted.

use crate::op_identity::field_value;
use crate::opattrs_schema::{parse_table, FieldWidth, SchemaField};
use crate::opattrs_value::{accumulator_ordinal, is_flag, target_dtype_ordinal, vocab, vocab_of, COMPUTE_DTYPE_TOKEN, MAX_RANK};
use crate::structure_key::DTYPES;

const SCHEMA: &str = "kiss-opattrs-field-types-v1";

/// The kind of a field's text value. `u8` is not a type on its own: it is a flag, an enum, a count or a dtype.
fn kind(f: &SchemaField) -> &'static str {
    match (f.width, f.name.as_str()) {
        (FieldWidth::U16Le, _) | (FieldWidth::U8, "axes") => "axis-set",
        (FieldWidth::Variable, _) => "vector",
        (FieldWidth::U64Le, _) => "count",
        (FieldWidth::U8, "accumulator") => "accumulator-dtype",
        (FieldWidth::U8, "target_dtype") => "dtype",
        (FieldWidth::U8, _) if is_flag(f) => "flag",
        (FieldWidth::U8, n) if vocab_of(n).is_some() => "enum",
        (FieldWidth::U8, _) => "count",
    }
}

fn q(s: &str) -> String {
    format!("\"{s}\"")
}

fn list(xs: &[String]) -> String {
    format!("[{}]", xs.iter().map(|x| q(x)).collect::<Vec<_>>().join(", "))
}

/// The rule text of a kind, for a reader that wants the range and not only the kind.
fn rule(op: &str, f: &SchemaField, kind: &str) -> String {
    match kind {
        "axis-set" => {
            let one = if op == "prefix_scan" { "; exactly one axis for prefix_scan" } else { "" };
            format!("`[a,b,..]` canonical decimals, strictly ascending, non-empty, each < {MAX_RANK}{one}")
        }
        "vector" => format!("`[a,b,..]` canonical decimals, 1 to {MAX_RANK} elements, each <= {}", u32::MAX),
        "count" if f.width == FieldWidth::U64Le => format!("canonical decimal (no sign, no leading zero), <= {}", u64::MAX),
        "count" => match count_values(op, f) {
            v if v.len() == 1 => format!("canonical decimal; exactly {} (a fixed field)", v[0]),
            v if v.windows(2).all(|w| w[1] == w[0] + 1) => format!("canonical decimal (no sign, no leading zero), {} to {}", v[0], v[v.len() - 1]),
            v => format!("canonical decimal, one of {v:?}"),
        },
        "flag" => "`true` or `false`".to_string(),
        "enum" => "one of `vocab`".to_string(),
        "accumulator-dtype" => "`compute-dtype` or one of `accepts` (a non-reserved float dtype, KISS-OPS-6.7-0012)".to_string(),
        _ => "one of `accepts` (a non-reserved dtype)".to_string(),
    }
}

/// Which `u8` decimals the reference reader accepts for a count field (0..=255 asked one by one).
fn count_values(op: &str, f: &SchemaField) -> Vec<u32> {
    (0u32..=255).filter(|n| field_value(op, f, &n.to_string()).is_ok()).collect()
}

/// The tokens a dtype-typed field accepts, in `DTYPES` order (plus `compute-dtype` for an accumulator).
fn dtype_accepts(op: &str, kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    if kind == "accumulator-dtype" {
        out.push(COMPUTE_DTYPE_TOKEN.to_string());
    }
    for d in DTYPES {
        let ok = if kind == "accumulator-dtype" { accumulator_ordinal(d, op).is_ok() } else { target_dtype_ordinal(d).is_ok() };
        if ok {
            out.push(d.to_string());
        }
    }
    out
}

/// Ask the reference reader about every stated fact; panic (fail the build) on any disagreement.
fn agree(op: &str, f: &SchemaField, kind: &str, vocab_tokens: &[String], accepts: &[String]) {
    let at = |v: &str| field_value(op, f, v).is_ok();
    match kind {
        "enum" => {
            for t in vocab_tokens {
                assert!(at(t), "{op}.{}: stated vocab token `{t}` is declined by the reader", f.name);
            }
            assert!(!at("zz-not-a-token"), "{op}.{}: an off-vocab token is accepted", f.name);
        }
        "flag" => assert!(at("true") && at("false") && !at("1") && !at("True"), "{op}.{}: flag spelling", f.name),
        "accumulator-dtype" | "dtype" => {
            let all: Vec<String> = DTYPES.iter().map(|d| d.to_string()).chain([COMPUTE_DTYPE_TOKEN.to_string()]).collect();
            for t in &all {
                assert_eq!(at(t), accepts.contains(t), "{op}.{}: reader and stated `accepts` disagree on `{t}`", f.name);
            }
        }
        "count" if f.width == FieldWidth::U64Le => assert!(at("0") && at(&u64::MAX.to_string()) && !at("00") && !at("-1"), "{op}.{}: u64", f.name),
        "count" => {
            let v = count_values(op, f);
            assert!(!v.is_empty() && !at("256") && !at("00") && !at("-1") && !at("+1"), "{op}.{}: u8 count", f.name);
        }
        "axis-set" => assert!(at("[0]") && !at("[]") && !at("[1,0]") && !at(&format!("[{MAX_RANK}]")) && (op == "prefix_scan") != at("[0,1]"), "{op}.{}: axis set", f.name),
        _ => assert!(at("[1]") && !at("[]") && !at("[0, 1]"), "{op}.{}: vector", f.name),
    }
}

/// `conformance/corpus/opattrs_field_types.json`: LF, two-space indent, byte-stable.
pub fn emit_opattrs_field_types_json() -> String {
    let vocabs = vocab();
    let mut ops = Vec::new();
    for s in parse_table() {
        if s.fields.is_empty() {
            continue;
        }
        let mut fields = Vec::new();
        for f in &s.fields {
            let k = kind(f);
            let vt: Vec<String> = if k == "enum" { vocabs[vocab_of(&f.name).unwrap()].iter().map(|(_, t)| t.clone()).collect() } else { Vec::new() };
            let acc = if k == "accumulator-dtype" || k == "dtype" { dtype_accepts(&s.op, k) } else { Vec::new() };
            agree(&s.op, f, k, &vt, &acc);
            let mut parts = vec![format!("\"name\": {}", q(&f.name)), format!("\"kind\": {}", q(k))];
            if k == "enum" {
                parts.push(format!("\"vocab\": {}", list(&vt)));
            }
            if !acc.is_empty() {
                parts.push(format!("\"accepts\": {}", list(&acc)));
            }
            parts.push(format!("\"rule\": {}", q(&rule(&s.op, f, k).replace('`', "'"))));
            fields.push(format!("        {{{}}}", parts.join(", ")));
        }
        ops.push(format!("    {}: [\n{}\n    ]", q(&s.op), fields.join(",\n")));
    }
    format!(
        "{{\n  \"schema\": {},\n  \"generated_from\": \"conformance/src/opattrs_field_types.rs (reads spec/ops.md 6.19.2 and 6.19.3)\",\n  \"clauses\": [\"KISS-CONTRACT-6.11-0009\", \"KISS-OPS-6.19-0049\"],\n  \"ops\": {{\n{}\n  }}\n}}\n",
        q(SCHEMA),
        ops.join(",\n")
    )
}
