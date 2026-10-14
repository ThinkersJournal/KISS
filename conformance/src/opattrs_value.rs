// SPDX-License-Identifier: MIT OR Apache-2.0
//! Value-bearing OpAttrs vectors (R2.4 slice 1, `docs/design-r24-opattrs-value-vectors.md`): attribute
//! tuple in, exact bytes out, for the carrier ops whose §6.19.3 schema is fully pinned.
//!
//! ⚠️ THE ENCODER HERE IS GENERIC, NOT PER-OP. It replays the schema PARSED from `spec/ops.md` §6.19.3
//! ([`crate::opattrs_schema::parse_table`]) and the sub-vocabulary ordinals PARSED from §6.19.2, so it
//! knows no op by name. It is deliberately a second derivation next to the hand encoders in
//! [`crate::opattrs`], which `tests/opattrs_value_vectors.rs` requires to agree with it.
//!
//! ⚠️ WHAT THAT IS WORTH. Both derivations share an author and a reading of the spec, so agreement shows
//! the code is consistent with the table, not that the table is read correctly (the E6 caveat). The file's
//! value is as a TARGET for a second party.
//!
//! ⚠️ TWO OPS ARE NOT EMITTED, AND THAT IS A FINDING, NOT AN OMISSION. `reduce` and `prefix_scan` carry
//! `accumulator` and `math_precision` since sk4 (§6.19-0025/-0026), but §6.19.2 pins NO ordinal table for
//! `math_precision` (the clause says "the `<mp>` enum ordinal of §6.7-0006", and §6.7-0006 spells the
//! values `st`/`rm` as TEXT), and Appendix E.2 and the hand encoders still show the pre-sk4 4-byte form.
//! Minting a byte for a field whose encoding the spec does not pin would be fabrication; the generator
//! refuses and records the reason in the artifact (`not_emitted`).

use crate::opattrs_schema::{parse_clause_coverage, parse_table, FieldWidth, OpSchema, SchemaField};
use std::collections::BTreeMap;

const OPS_SPEC: &str = include_str!("../../spec/ops.md");

/// `MAX_RANK` / `MAX_OPERANDS` (§6.19-0037). The test asserts the spec still says `= 8`.
pub const MAX_RANK: u32 = 8;
/// See [`MAX_RANK`].
pub const MAX_OPERANDS: u32 = 8;

/// One attribute value, spelled as a consumer would hold it (never an ordinal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A sub-vocabulary token, e.g. `sum`, `clamp`, `zero-fill`.
    Token(String),
    /// A boolean flag (`keepdim`, `bessel_correction`, ...).
    Flag(bool),
    /// A plain `u8` field: `axis`, `norm_axis`, `index_operand`.
    Byte(u8),
    /// A reduction's axis set and the operand rank (the rank is NOT in the blob; §6.19-0020).
    Axes { axes: Vec<usize>, rank: usize },
    /// A window-parameter vector (§6.19-0023), in ascending spatial-axis order.
    Vector(Vec<u32>),
}

/// Named attribute values in any order; the encoder replays the schema order.
pub type Attrs = Vec<(String, Value)>;

/// Ops this module declines to emit, with the reason recorded in the artifact.
pub fn not_emitted() -> [(&'static str, &'static str); 2] {
    const WHY: &str = "the sk4 fields `accumulator` and `math_precision` have no pinned wire ordinal table in \
        §6.19.2 (`<mp>` is spelled st/rm as text in §6.7-0006); Appendix E.2 and the hand encoders still show \
        the pre-sk4 4-byte form, which §6.19-0025/-0026 no longer says";
    [("reduce", WHY), ("prefix_scan", WHY)]
}

fn section_6_19_2() -> &'static str {
    let start = OPS_SPEC.find("#### 6.19.2").expect("spec/ops.md must carry `#### 6.19.2`");
    let rest = &OPS_SPEC[start + 1..];
    let end = rest.find("\n#### ").map(|i| i + 1).unwrap_or(rest.len());
    &rest[..end]
}

/// The §6.19.2 enum sub-vocabularies: name -> (ordinal, token), reserved `0` omitted.
/// Rows read `| \`monoid\` | enum, \`u8\` | \`0\`=RESERVED, \`1\`=sum, ... |`.
pub fn vocab() -> BTreeMap<String, Vec<(u8, String)>> {
    let mut out = BTreeMap::new();
    for line in section_6_19_2().lines() {
        let cells: Vec<&str> = line.trim().trim_matches('|').split(" | ").collect();
        if cells.len() != 3 || !cells[1].starts_with("enum") {
            continue;
        }
        let name = cells[0].trim().trim_matches('`').to_string();
        let mut entries = Vec::new();
        for item in cells[2].split(", ") {
            let Some((ord, tok)) = item.trim().split_once('=') else { continue };
            let Ok(ord) = ord.trim().trim_matches('`').parse::<u8>() else { continue };
            let tok = tok.split(" (").next().unwrap_or(tok).trim().to_string();
            if ord != 0 && tok != "RESERVED" {
                entries.push((ord, tok));
            }
        }
        out.insert(name, entries);
    }
    assert!(!out.is_empty(), "§6.19.2 parsed to ZERO enum sub-vocabularies: the table grammar changed");
    out
}

/// Which sub-vocabulary a schema field draws from. ⚠️ The ONE transcribed fact here: the §6.19.3 table
/// names the vocabulary only for some fields (`exclusivity`:`scan_exclusivity`), so the rest are mapped by
/// hand. A wrong entry fails the cross-check against the hand encoders.
fn vocab_of(field: &str) -> Option<&'static str> {
    Some(match field {
        "monoid" => "monoid",
        "oob_policy" => "oob_policy",
        "combine" => "scatter_combine",
        "index_dtype" => "index_dtype",
        "direction" => "sort_direction",
        "exclusivity" => "scan_exclusivity",
        "output_column_ordering" => "column_ordering",
        _ => return None,
    })
}

/// A field is a flag when the table says `bool`.
fn is_flag(f: &SchemaField) -> bool {
    f.encoding.starts_with("bool")
}

/// Whether the table marks the field fixed (a single legal value).
pub fn is_fixed(f: &SchemaField) -> bool {
    // ⚠️ `keepdim` is "fixed `1`" in the CLAUSE (§6.19-0030, §6.11-0008) but the §6.19.3 table row does
    // not say so; the test asserts the clause text so this exception cannot go stale unnoticed.
    f.encoding.contains("fixed") || f.name == "keepdim"
}

/// Whether the table marks the field mandatory (no default).
pub fn is_mandatory(f: &SchemaField) -> bool {
    f.encoding.contains("mandatory") || f.encoding.contains("trailing axis")
}

/// The resolved default byte the table gives, e.g. `enum \`u8\`:\`1\` (skip)` -> `1`.
pub fn default_byte(f: &SchemaField) -> Option<u8> {
    let after = f.encoding.split("`u8`:").nth(1)?;
    let digits = after.strip_prefix('`')?.split('`').next()?;
    digits.parse().ok()
}

fn token_of(vocab_name: &str, ordinal: u8) -> Option<String> {
    vocab().get(vocab_name)?.iter().find(|(o, _)| *o == ordinal).map(|(_, t)| t.clone())
}

fn schema_of(op: &str) -> Result<OpSchema, String> {
    parse_table().into_iter().find(|s| s.op == op).ok_or_else(|| format!("`{op}` is not a carrier op"))
}

/// `reduce_axes` word (§6.19-0020 total precedence), by bitmask arithmetic.
pub fn reduce_axes_word(axes: &[usize], rank: usize) -> Result<u16, String> {
    if !(1..=MAX_RANK as usize).contains(&rank) {
        return Err(format!("rank {rank} outside 1..=MAX_RANK"));
    }
    if axes.is_empty() || axes.iter().any(|&a| a >= rank) {
        return Err("reduced axes empty or out of range".into());
    }
    let mask = axes.iter().fold(0u32, |m, &a| m | (1 << a));
    let full = (1u32 << rank) - 1;
    Ok(if mask == full {
        0xFFFF
    } else if rank > 1 && mask == 1 << (rank - 1) {
        0xFFFE
    } else {
        mask as u16
    })
}

fn encode_field(f: &SchemaField, v: &Value, vocab: &BTreeMap<String, Vec<(u8, String)>>) -> Result<Vec<u8>, String> {
    if matches!(f.name.as_str(), "accumulator" | "math_precision") {
        return Err(format!("field `{}` has no pinned wire ordinal table (§6.19.2)", f.name));
    }
    match (f.width, v) {
        (FieldWidth::U16Le, Value::Axes { axes, rank }) => {
            Ok(reduce_axes_word(axes, *rank)?.to_le_bytes().to_vec())
        }
        (FieldWidth::Variable, Value::Vector(elems)) => {
            if elems.len() > MAX_RANK as usize {
                return Err(format!("`{}` vector longer than MAX_RANK", f.name));
            }
            let mut out = vec![elems.len() as u8];
            for e in elems {
                out.extend_from_slice(&e.to_le_bytes());
            }
            Ok(out)
        }
        (FieldWidth::U8, Value::Flag(b)) if is_flag(f) => Ok(vec![u8::from(*b)]),
        (FieldWidth::U8, Value::Byte(n)) if vocab_of(&f.name).is_none() && !is_flag(f) => {
            if u32::from(*n) >= MAX_RANK.max(MAX_OPERANDS) {
                return Err(format!("`{}` = {n} is out of range", f.name));
            }
            Ok(vec![*n])
        }
        (FieldWidth::U8, Value::Token(t)) => {
            let name = vocab_of(&f.name).ok_or_else(|| format!("`{}` is not an enum field", f.name))?;
            let ord = vocab
                .get(name)
                .and_then(|es| es.iter().find(|(_, tok)| tok == t))
                .ok_or_else(|| format!("`{t}` is not a `{name}` token"))?;
            Ok(vec![ord.0])
        }
        _ => Err(format!("value {v:?} does not fit field `{}`", f.name)),
    }
}

/// Encode with an explicit schema (the controls pass a mutated one).
pub fn encode_with_schema(schema: &OpSchema, attrs: &Attrs) -> Result<Vec<u8>, String> {
    let vocab = vocab();
    for (name, _) in attrs {
        if !schema.fields.iter().any(|f| &f.name == name) {
            return Err(format!("`{name}` is not a field of `{}`", schema.op));
        }
    }
    let mut out = Vec::new();
    for f in &schema.fields {
        let v = attrs
            .iter()
            .find(|(n, _)| n == &f.name)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("`{}` has no value for `{}`", schema.op, f.name))?;
        out.extend(encode_field(f, v, &vocab)?);
    }
    Ok(out)
}

/// Encode `op` from fully resolved attributes.
pub fn encode(op: &str, attrs: &Attrs) -> Result<Vec<u8>, String> {
    encode_with_schema(&schema_of(op)?, attrs)
}

/// Fill every field `partial` omits from the table's resolved default (§6.19-0005); a mandatory or
/// default-less field that is omitted is an error, so the equivalence it supports cannot be vacuous.
pub fn resolve_defaults(op: &str, partial: &Attrs) -> Result<Attrs, String> {
    let schema = schema_of(op)?;
    let mut out = Attrs::new();
    for f in &schema.fields {
        if let Some((_, v)) = partial.iter().find(|(n, _)| n == &f.name) {
            out.push((f.name.clone(), v.clone()));
            continue;
        }
        let byte = default_byte(f).filter(|_| !is_mandatory(f)).ok_or_else(|| {
            format!("`{}` omits `{}`, which has no resolved default", op, f.name)
        })?;
        let v = if is_flag(f) {
            Value::Flag(byte != 0)
        } else {
            let name = vocab_of(&f.name).ok_or_else(|| format!("default of `{}` is not an enum", f.name))?;
            Value::Token(token_of(name, byte).ok_or_else(|| format!("no token for `{name}`={byte}"))?)
        };
        out.push((f.name.clone(), v));
    }
    Ok(out)
}

/// The base attribute tuple of an op: table defaults where the table gives one, a fixed legal value
/// where it is mandatory. `None` for an op in [`not_emitted`].
pub fn base_attrs(op: &str) -> Option<Attrs> {
    if not_emitted().iter().any(|(o, _)| *o == op) {
        return None;
    }
    let schema = schema_of(op).ok()?;
    let vocab = vocab();
    let mut out = Attrs::new();
    for f in &schema.fields {
        let v = if f.width == FieldWidth::U16Le {
            Value::Axes { axes: vec![0, 1], rank: 2 }
        } else if f.width == FieldWidth::Variable {
            Value::Vector(match f.name.as_str() {
                "padding" => vec![0, 0],
                "window_size" => vec![3, 3],
                _ => vec![1, 1],
            })
        } else if is_flag(f) {
            Value::Flag(default_byte(f).unwrap_or(1) != 0)
        } else if let Some(name) = vocab_of(&f.name) {
            let first = vocab.get(name)?.first()?.0;
            let ord = default_byte(f).filter(|_| !is_mandatory(f)).unwrap_or(first);
            Value::Token(token_of(name, ord)?)
        } else {
            Value::Byte(if f.name == "index_operand" { 1 } else { 0 })
        };
        out.push((f.name.clone(), v));
    }
    Some(out)
}

/// Alternatives to the base value of one field: the values a vector varies it over.
pub fn alternatives(f: &SchemaField, base: &Value) -> Vec<Value> {
    if is_fixed(f) {
        return Vec::new();
    }
    let vocab = vocab();
    let mut alts: Vec<Value> = match (f.width, base) {
        (FieldWidth::U16Le, _) => [(vec![0], 1), (vec![0], 2), (vec![1], 2), (vec![0, 1], 2), (vec![0, 2], 3),
            (vec![0, 1, 2], 3), (vec![2], 3), (vec![1], 3), (vec![0, 1, 2, 3, 4, 5, 6, 7], 8), (vec![7], 8),
            (vec![0, 7], 8)]
            .into_iter()
            .map(|(axes, rank)| Value::Axes { axes, rank })
            .collect(),
        (FieldWidth::Variable, _) => [vec![3, 5], vec![2], vec![2, 2, 2], vec![1; 8]]
            .into_iter()
            .map(Value::Vector)
            .collect(),
        (_, Value::Flag(b)) => vec![Value::Flag(!b)],
        (_, Value::Token(_)) => vocab_of(&f.name)
            .and_then(|n| vocab.get(n))
            .map(|es| es.iter().map(|(_, t)| Value::Token(t.clone())).collect())
            .unwrap_or_default(),
        (_, Value::Byte(_)) => [0u8, 1, 7].into_iter().map(Value::Byte).collect(),
        _ => Vec::new(),
    };
    alts.retain(|v| v != base);
    alts
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

fn json_value(v: &Value) -> String {
    match v {
        Value::Token(t) => format!("\"{t}\""),
        Value::Flag(b) => b.to_string(),
        Value::Byte(n) => n.to_string(),
        Value::Axes { axes, rank } => {
            let a: Vec<String> = axes.iter().map(ToString::to_string).collect();
            format!("{{\"axes\":[{}],\"rank\":{rank}}}", a.join(","))
        }
        Value::Vector(e) => {
            let a: Vec<String> = e.iter().map(ToString::to_string).collect();
            format!("[{}]", a.join(","))
        }
    }
}

fn json_attrs(attrs: &Attrs) -> String {
    let parts: Vec<String> = attrs.iter().map(|(n, v)| format!("\"{n}\":{}", json_value(v))).collect();
    format!("{{{}}}", parts.join(","))
}

/// One minted vector: attributes, the bytes the generic encoder gives, and why it exists.
#[derive(Debug, Clone)]
pub struct EncodeVector {
    pub tc_id: u32,
    pub op: String,
    pub attrs: Attrs,
    pub blob: Vec<u8>,
    pub tag: String,
}

/// The population: for each emitted op, the base tuple and every one-field variation of it.
pub fn encode_vectors() -> Vec<EncodeVector> {
    let mut out = Vec::new();
    let mut tc = 1u32;
    for schema in parse_table() {
        let Some(base) = base_attrs(&schema.op) else { continue };
        let mut push = |attrs: Attrs, tag: String| {
            let blob = encode(&schema.op, &attrs).unwrap_or_else(|e| panic!("{}: {e}", schema.op));
            out.push(EncodeVector { tc_id: tc, op: schema.op.clone(), attrs, blob, tag });
            tc += 1;
        };
        push(base.clone(), "base".into());
        for (i, f) in schema.fields.iter().enumerate() {
            for alt in alternatives(f, &base[i].1) {
                let mut attrs = base.clone();
                attrs[i].1 = alt;
                push(attrs, format!("vary:{}", f.name));
            }
        }
    }
    out
}

const HEADER: &str = r#"  "schema": "kiss-opattrs-value-vectors/1",
  "generated_from": "spec/ops.md §6.19.2 (ordinals) + §6.19.3 (schema, resolved defaults)",
  "derivation": "a GENERIC schema-driven encoder over the parsed spec tables (conformance/src/opattrs_value.rs); cross-checked against the hand encoders in opattrs.rs by tests/opattrs_value_vectors.rs. Same author and same reading of the spec as those encoders: a TARGET for a second party, not independent evidence (E6).",
  "reduce_axes_note": "`rank` is carried in the vector only; it is NOT in the blob (§6.19-0020). Two vectors with different (axes, rank) can share a blob by design (rank-1 sole axis and rank-2 full both encode FF FF).",
"#;

fn join_items(items: &[String]) -> String {
    items.join(",\n")
}

fn render_not_emitted() -> String {
    let items: Vec<String> = not_emitted()
        .iter()
        .map(|(op, why)| format!("    {{\"op\": \"{op}\", \"why\": \"{why}\"}}"))
        .collect();
    format!("  \"not_emitted\": [\n{}\n  ],\n", join_items(&items))
}

fn render_vectors(vectors: &[EncodeVector]) -> String {
    let coverage = parse_clause_coverage();
    let items: Vec<String> = vectors
        .iter()
        .map(|v| {
            let clauses: Vec<String> =
                coverage.get(&v.op).cloned().unwrap_or_default().iter().map(|c| format!("\"{c}\"")).collect();
            format!(
                "    {{\"tcId\": {}, \"kind\": \"encode\", \"op\": \"{}\", \"attrs\": {}, \"blob\": \"{}\", \"tag\": \"{}\", \"clauses\": [{}]}}",
                v.tc_id,
                v.op,
                json_attrs(&v.attrs),
                hex(&v.blob),
                v.tag,
                clauses.join(", ")
            )
        })
        .collect();
    format!("  \"vectors\": [\n{}\n  ],\n", join_items(&items))
}

fn render_equivalences() -> String {
    let items: Vec<String> = equivalences()
        .iter()
        .map(|e| {
            let omitted: Vec<String> = e.omitted.iter().map(|n| format!("\"{n}\"")).collect();
            format!(
                "    {{\"op\": \"{}\", \"explicit\": {}, \"omitted\": [{}], \"same_blob\": \"{}\"}}",
                e.op,
                json_attrs(&e.explicit),
                omitted.join(","),
                hex(&e.blob)
            )
        })
        .collect();
    format!("  \"equivalences\": [\n{}\n  ],\n", join_items(&items))
}

fn render_distinct(vectors: &[EncodeVector]) -> String {
    let items: Vec<String> = distinct_pairs(vectors)
        .iter()
        .map(|(a, b, op)| format!("    {{\"op\": \"{op}\", \"a\": {a}, \"b\": {b}}}"))
        .collect();
    format!("  \"distinct\": [\n{}\n  ]\n", join_items(&items))
}

/// Render the artifact `conformance/corpus/opattrs_value_vectors.json`.
pub fn emit_opattrs_value_vectors_json() -> String {
    let vectors = encode_vectors();
    let mut s = String::from("{\n");
    s.push_str(HEADER);
    s.push_str(&format!("  \"number_of_vectors\": {},\n", vectors.len()));
    s.push_str(&render_not_emitted());
    s.push_str(&render_vectors(&vectors));
    s.push_str(&render_equivalences());
    s.push_str(&render_distinct(&vectors));
    s.push_str("}\n");
    s
}

/// A default-resolution equivalence (§6.19-0005): the omitted fields are filled from the table.
#[derive(Debug, Clone)]
pub struct Equivalence {
    pub op: String,
    pub explicit: Attrs,
    pub omitted: Vec<String>,
    pub blob: Vec<u8>,
}

/// One equivalence per emitted op that has at least one defaulted, non-mandatory field.
pub fn equivalences() -> Vec<Equivalence> {
    let mut out = Vec::new();
    for schema in parse_table() {
        let Some(base) = base_attrs(&schema.op) else { continue };
        let omitted: Vec<String> = schema
            .fields
            .iter()
            .filter(|f| !is_mandatory(f) && default_byte(f).is_some())
            .map(|f| f.name.clone())
            .collect();
        if omitted.is_empty() {
            continue;
        }
        let partial: Attrs = base.iter().filter(|(n, _)| !omitted.contains(n)).cloned().collect();
        let resolved = resolve_defaults(&schema.op, &partial).expect("defaults resolve");
        let blob = encode(&schema.op, &resolved).expect("encodes");
        assert_eq!(blob, encode(&schema.op, &base).expect("encodes"), "{}: defaulted != explicit", schema.op);
        out.push(Equivalence { op: schema.op.clone(), explicit: base, omitted, blob });
    }
    out
}

/// Pairs (base tcId, varied tcId, op) whose blobs differ, so each is a real non-collision.
pub fn distinct_pairs(vectors: &[EncodeVector]) -> Vec<(u32, u32, String)> {
    let mut by_op: BTreeMap<&str, &EncodeVector> = BTreeMap::new();
    let mut out = Vec::new();
    for v in vectors {
        if v.tag == "base" {
            by_op.insert(&v.op, v);
        } else if let Some(b) = by_op.get(v.op.as_str()) {
            if b.blob != v.blob {
                out.push((b.tc_id, v.tc_id, v.op.clone()));
            }
        }
    }
    out
}
