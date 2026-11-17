// SPDX-License-Identifier: MIT OR Apache-2.0
//! The `op_identity` text spelling (KISS-CONTRACT-6.11-0009, with 6.3-0003 / 6.3-0004) and the vectors of it.
//!
//! CireSnave ruled board 451 (2026-11-16, verbatim: "451: 1: A.  2: A.  3: A.  4: A."): the `{...}` list spells
//! EVERY field of the op's OpAttrs record, in the KISS-Ops canonical field order of its §6.19.3 row; a key is the
//! field name, an enum value its §6.19.2 name, a number a canonical decimal, a dtype its token (`compute-dtype`
//! for an accumulator at the compute dtype), a flag `true`/`false`, an axis set or window vector `[0,1]`. A line
//! that omits, repeats, mis-orders or invents a field is declined; whether a line was "reduced to the bare name"
//! is a producer rule (the document does not carry the advertisable-op list), so a reader decides only what the
//! line itself shows.
//!
//! No per-op table is written here: the schema is PARSED from `spec/ops.md` §6.19.3 ([`parse_table`]), the enum
//! names from §6.19.2, and the set of ops from `op_facts.json`. Value ranges are those of the OpAttrs encoder
//! (`opattrs_value`), reused so the text and the blob cannot disagree about what a legal value is.

use crate::opattrs_schema::{parse_table, FieldWidth, OpSchema, SchemaField};
use crate::opattrs_value::{
    accumulator_ordinal, encode_with_schema, is_flag, target_dtype_ordinal, vocab, vocab_of, Value, COMPUTE_DTYPE_TOKEN, MAX_RANK,
};
use std::sync::OnceLock;

/// A typed decline: never a panic. The wire tag is stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityDecline {
    /// Not `<op_name>[{<attrs>}][[<roles>]]`: bad name characters, spacing, an unclosed brace or bracket.
    Malformed,
    /// The op name is not a KISS-Ops op.
    UnknownOp,
    /// A key the op's OpAttrs record does not have (also any attribute on an op with no record).
    UnknownAttr,
    /// A key written twice.
    DuplicateAttr,
    /// A field of the op's OpAttrs record is missing (including a carrier op written bare).
    Incomplete,
    /// Every field present but not in the §6.19.3 canonical order.
    AttrOrder,
    /// A value not spelled as its field's type requires, or outside the field's legal range.
    BadValue,
    /// The `[role:dtype, ...]` suffix is empty or has a malformed entry.
    BadRoleTuple,
}

impl IdentityDecline {
    pub fn wire_tag(self) -> &'static str {
        match self {
            Self::Malformed => "op-identity-malformed",
            Self::UnknownOp => "op-identity-unknown-op",
            Self::UnknownAttr => "op-identity-unknown-attr",
            Self::DuplicateAttr => "op-identity-duplicate-attr",
            Self::Incomplete => "op-identity-incomplete",
            Self::AttrOrder => "op-identity-attr-order",
            Self::BadValue => "op-identity-bad-value",
            Self::BadRoleTuple => "op-identity-bad-role-tuple",
        }
    }
}

fn schemas() -> &'static [OpSchema] {
    static S: OnceLock<Vec<OpSchema>> = OnceLock::new();
    S.get_or_init(parse_table)
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some('a'..='z')) && c.all(|ch| matches!(ch, 'a'..='z' | '0'..='9' | '_'))
}

/// A canonical decimal: digits only, no sign, no leading zero (but `0` itself), no space.
fn canonical_u64(s: &str) -> Option<u64> {
    let ok = !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s == "0" || !s.starts_with('0'));
    if ok { s.parse().ok() } else { None }
}

/// `[a,b,c]` of canonical decimals, no spaces; `None` when it is not that shape.
fn int_list(s: &str) -> Option<Vec<u64>> {
    let inner = s.strip_prefix('[')?.strip_suffix(']')?;
    if inner.is_empty() {
        return Some(Vec::new());
    }
    inner.split(',').map(canonical_u64).collect()
}

fn axis_set(op: &str, v: &str) -> Result<Vec<usize>, IdentityDecline> {
    let xs = int_list(v).ok_or(IdentityDecline::BadValue)?;
    let ascending = xs.windows(2).all(|w| w[0] < w[1]);
    let one_only = op != "prefix_scan" || xs.len() == 1;
    if xs.is_empty() || !ascending || xs.iter().any(|&a| a >= u64::from(MAX_RANK)) || !one_only {
        return Err(IdentityDecline::BadValue);
    }
    Ok(xs.iter().map(|&a| a as usize).collect())
}

/// A one-byte field: the accumulator and target-dtype spellings, a flag, a vocabulary token or a plain byte.
fn byte_value(f: &SchemaField, v: &str, op: &str) -> Result<Value, IdentityDecline> {
    let bad = IdentityDecline::BadValue;
    match f.name.as_str() {
        "axes" => Ok(Value::Byte(axis_set(op, v)?.iter().fold(0u8, |m, &a| m | (1 << a)))),
        "accumulator" if v == COMPUTE_DTYPE_TOKEN || accumulator_ordinal(v, op).is_ok() => Ok(Value::Token(v.to_string())),
        "accumulator" => Err(bad),
        "target_dtype" => target_dtype_ordinal(v).map(|_| Value::Token(v.to_string())).map_err(|_| bad),
        _ if is_flag(f) => match v {
            "true" => Ok(Value::Flag(true)),
            "false" => Ok(Value::Flag(false)),
            _ => Err(bad),
        },
        name if vocab_of(name).is_some() => {
            let known = vocab_of(name).and_then(|n| vocab().get(n).map(|es| es.iter().any(|(_, t)| t == v)));
            if known == Some(true) { Ok(Value::Token(v.to_string())) } else { Err(bad) }
        }
        _ => {
            let n = canonical_u64(v).filter(|&n| n <= u64::from(u8::MAX)).ok_or(bad)?;
            Ok(Value::Byte(n as u8))
        }
    }
}

fn field_value(op: &str, f: &SchemaField, v: &str) -> Result<(), IdentityDecline> {
    let bad = IdentityDecline::BadValue;
    let value = match f.width {
        FieldWidth::U16Le => return axis_set(op, v).map(|_| ()),
        FieldWidth::Variable => {
            let xs = int_list(v).ok_or(bad)?;
            if xs.is_empty() || xs.len() > MAX_RANK as usize || xs.iter().any(|&x| x > u64::from(u32::MAX)) {
                return Err(bad);
            }
            Value::Vector(xs.iter().map(|&x| x as u32).collect())
        }
        FieldWidth::U64Le => Value::U64(canonical_u64(v).ok_or(bad)?),
        FieldWidth::U8 => match f.name.as_str() {
            // These two are checked against their own ordinal tables and need no one-field encode.
            "accumulator" | "target_dtype" => return byte_value(f, v, op).map(|_| ()),
            _ => byte_value(f, v, op)?,
        },
    };
    // The one-field schema reuses the OpAttrs encoder's range and fixed-value rules for this field.
    let one = OpSchema { op: op.to_string(), fields: vec![f.clone()] };
    encode_with_schema(&one, &vec![(f.name.clone(), value)]).map(|_| ()).map_err(|_| bad)
}

fn role_tuple(s: &str) -> Result<(), IdentityDecline> {
    let inner = s.strip_prefix('[').and_then(|r| r.strip_suffix(']')).ok_or(IdentityDecline::Malformed)?;
    let bad = IdentityDecline::BadRoleTuple;
    if inner.is_empty() {
        return Err(bad);
    }
    for entry in inner.split(", ") {
        let (role, dtype) = entry.split_once(':').ok_or(bad)?;
        let dtype_ok = dtype == "*" || crate::structure_key::DTYPES.contains(&dtype);
        if !is_ident(role) || !dtype_ok {
            return Err(bad);
        }
    }
    Ok(())
}

/// Split `<name>[{attrs}][[roles]]`: `(name, attrs, roles)`.
fn split_line(line: &str) -> Result<(&str, Option<&str>, Option<&str>), IdentityDecline> {
    let cut = line.find(['{', '[']).unwrap_or(line.len());
    let (name, mut rest) = line.split_at(cut);
    if !is_ident(name) {
        return Err(IdentityDecline::Malformed);
    }
    let mut attrs = None;
    if let Some(body) = rest.strip_prefix('{') {
        let close = body.find('}').ok_or(IdentityDecline::Malformed)?;
        attrs = Some(&body[..close]);
        rest = &body[close + 1..];
    }
    let roles = if rest.is_empty() { None } else { Some(rest) };
    if roles.is_some_and(|r| !r.starts_with('[') || !r.ends_with(']') || r.contains(['{', '}'])) {
        return Err(IdentityDecline::Malformed);
    }
    Ok((name, attrs, roles))
}

fn check_attrs(op: &str, schema: Option<&OpSchema>, attrs: Option<&str>) -> Result<(), IdentityDecline> {
    let (Some(schema), Some(attrs)) = (schema, attrs) else {
        // No record: no braces allowed. A carrier written bare is missing every field.
        return match (schema, attrs) {
            (None, Some(_)) => Err(IdentityDecline::UnknownAttr),
            (Some(_), None) => Err(IdentityDecline::Incomplete),
            _ => Ok(()),
        };
    };
    let mut pairs = Vec::new();
    for piece in if attrs.is_empty() { Vec::new() } else { attrs.split(", ").collect() } {
        let (k, v) = piece.split_once('=').ok_or(IdentityDecline::Malformed)?;
        if !is_ident(k) || v.is_empty() || v.contains(['=', ' ', '{', '}']) {
            return Err(IdentityDecline::Malformed);
        }
        pairs.push((k, v));
    }
    for (i, (k, _)) in pairs.iter().enumerate() {
        if !schema.fields.iter().any(|f| f.name == *k) {
            return Err(IdentityDecline::UnknownAttr);
        }
        if pairs[..i].iter().any(|(p, _)| p == k) {
            return Err(IdentityDecline::DuplicateAttr);
        }
    }
    if schema.fields.iter().any(|f| !pairs.iter().any(|(k, _)| *k == f.name)) {
        return Err(IdentityDecline::Incomplete);
    }
    if schema.fields.iter().zip(&pairs).any(|(f, (k, _))| f.name != *k) {
        return Err(IdentityDecline::AttrOrder);
    }
    for (f, (_, v)) in schema.fields.iter().zip(&pairs) {
        field_value(op, f, v)?;
    }
    Ok(())
}

/// Read an `op_identity` line in the §6.11-0009 spelling.
pub fn check_op_identity(line: &str) -> Result<(), IdentityDecline> {
    let (name, attrs, roles) = split_line(line)?;
    if crate::op_dag::op_facts(name).is_none() {
        return Err(IdentityDecline::UnknownOp);
    }
    check_attrs(name, schemas().iter().find(|s| s.op == name), attrs)?;
    roles.map_or(Ok(()), role_tuple)
}
