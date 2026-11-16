// SPDX-License-Identifier: MIT OR Apache-2.0
//! `op_identity` line vectors (KISS-CONTRACT-6.11-0009, 6.3-0003, 6.3-0004): single lines a reader must ACCEPT, and
//! single-fault lines it must DECLINE with a stable tag. The generator states the tag it intends and
//! [`check_op_identity`] must return exactly that, so an intended tag the reader does not produce fails the
//! build instead of being emitted (the `op_dag_vectors` discipline).

use crate::op_identity::{check_op_identity, IdentityDecline};

/// A line and the verdict a reader must reach.
pub struct Vector {
    pub name: &'static str,
    pub line: &'static str,
    pub expect: Option<IdentityDecline>,
}

const GATHER: &str = "gather{axis=0, oob_policy=clamp, index_operand=1, index_dtype=i32}";

const fn ok(name: &'static str, line: &'static str) -> Vector {
    Vector { name, line, expect: None }
}

const fn no(name: &'static str, line: &'static str, why: IdentityDecline) -> Vector {
    Vector { name, line, expect: Some(why) }
}

fn accepts() -> Vec<Vector> {
    vec![
        ok("an op with no OpAttrs record renders bare", "add"),
        ok("gather, the Appendix C form (a) line", GATHER),
        ok("gather at its defaults, every field still written", "gather{axis=0, oob_policy=skip, index_operand=1, index_dtype=u32}"),
        ok("prefix_scan, the Appendix C form (b) line", "prefix_scan{monoid=sum, reduce_axes=[0], exclusivity=inclusive, accumulator=compute-dtype, math_fidelity=bit-stable}"),
        ok("reduce_mean with a one-axis set", "reduce_mean{reduce_axes=[1], keepdim=true}"),
        ok("reduce_mean with a two-axis set and a false flag", "reduce_mean{reduce_axes=[0,1], keepdim=false}"),
        ok("matmul: the u64 field is a canonical decimal", "matmul{accumulator=compute-dtype, math_fidelity=bit-stable, accuracy_kind=none, accuracy_value=0}"),
        ok("a dtype-typed accumulator spelled as its token", "reduce{monoid=sum, reduce_axes=[0], keepdim=true, accumulator=f32, math_fidelity=bit-stable}"),
        ok("cast: target_dtype is a dtype token", "cast{target_dtype=f16, rounding=nearest-even, saturation=none}"),
        ok("avg_pool: window vectors are bracketed lists", "avg_pool{window_size=[2,2], stride=[2,2], dilation=[1,1], padding_lo=[0,0], padding_hi=[0,0], count_include_pad=true}"),
        ok("squeeze: an axes mask is spelled as its axis list", "squeeze{axes=[0,2]}"),
        ok("form (a): a role-tuple suffix after the attributes", "gather{axis=0, oob_policy=clamp, index_operand=1, index_dtype=i32}[data:f32, index:i32]"),
        ok("form (a): a role-tuple suffix on an op with no record", "add[data:*, data:f32]"),
    ]
}

fn declines() -> Vec<Vector> {
    use IdentityDecline::*;
    vec![
        no("a carrier written bare (the record is missing)", "gather", Incomplete),
        no("one field of the record omitted (the default left out)", "gather{axis=0, index_operand=1, index_dtype=i32}", Incomplete),
        no("the old golden line: `oob` is not a field", "gather{axis=0, oob=clamp}", UnknownAttr),
        no("an op KISS-Ops does not have", "scan{op=add}", UnknownOp),
        no("an attribute on an op with no record", "add{axis=0}", UnknownAttr),
        no("fields out of the canonical order", "gather{oob_policy=clamp, axis=0, index_operand=1, index_dtype=i32}", AttrOrder),
        no("alphabetical order is not the canonical order", "gather{axis=0, index_dtype=i32, index_operand=1, oob_policy=clamp}", AttrOrder),
        no("a key written twice", "gather{axis=0, axis=0, oob_policy=clamp, index_operand=1, index_dtype=i32}", DuplicateAttr),
        no("an enum value outside its sub-vocabulary", "gather{axis=0, oob_policy=wrap, index_operand=1, index_dtype=i32}", BadValue),
        no("a flag spelled as a number", "reduce_mean{reduce_axes=[1], keepdim=1}", BadValue),
        no("a number with a leading zero", "gather{axis=00, oob_policy=clamp, index_operand=1, index_dtype=i32}", BadValue),
        no("a negative number", "gather{axis=-1, oob_policy=clamp, index_operand=1, index_dtype=i32}", BadValue),
        no("an axis past MAX_RANK", "gather{axis=8, oob_policy=clamp, index_operand=1, index_dtype=i32}", BadValue),
        no("a placeholder where a number belongs", "gather{axis=k, oob_policy=clamp, index_operand=1, index_dtype=i32}", BadValue),
        no("an axis set out of ascending order", "reduce_mean{reduce_axes=[1,0], keepdim=true}", BadValue),
        no("an axis set repeating an axis", "reduce_mean{reduce_axes=[0,0], keepdim=true}", BadValue),
        no("an empty axis set", "reduce_mean{reduce_axes=[], keepdim=true}", BadValue),
        no("rall is not spelled in this line", "reduce_mean{reduce_axes=rall, keepdim=true}", BadValue),
        no("a scan over two axes", "prefix_scan{monoid=sum, reduce_axes=[0,1], exclusivity=inclusive, accumulator=compute-dtype, math_fidelity=bit-stable}", BadValue),
        no("an accumulator that is not a float dtype", "reduce{monoid=sum, reduce_axes=[0], keepdim=true, accumulator=i8, math_fidelity=bit-stable}", BadValue),
        no("a space inside a list", "reduce_mean{reduce_axes=[0, 1], keepdim=true}", Malformed),
        no("no space after the comma between pairs", "reduce_mean{reduce_axes=[0],keepdim=true}", Malformed),
        no("an upper-case op name", "Add", Malformed),
        no("an empty line", "", Malformed),
        no("trailing whitespace", "add ", Malformed),
        no("an unclosed brace", "gather{axis=0", Malformed),
        no("the role tuple before the attributes", "gather[data:f32]{axis=0, oob_policy=clamp, index_operand=1, index_dtype=i32}", Malformed),
        no("an empty role tuple", "add[]", BadRoleTuple),
        no("a role tuple naming no dtype token", "add[data:f99]", BadRoleTuple),
        no("a role entry with no dtype", "add[data]", BadRoleTuple),
    ]
}

/// Every vector, accepts first.
pub fn vectors() -> Vec<Vector> {
    let mut v = accepts();
    v.extend(declines());
    v
}

/// What the reader returns for `line`: the stated tag, or `None` for an accept.
pub fn verdict(line: &str) -> Option<IdentityDecline> {
    check_op_identity(line).err()
}

/// The `op_identity_vectors.json` artifact. Refuses to render a vector the reader does not agree with.
pub fn emit_op_identity_vectors_json() -> String {
    let jstr = crate::json::escape_string;
    let mut s = String::from("{\n");
    s.push_str("  \"spec_reference\": ");
    s.push_str(&jstr("spec/contract.md 6.11-0009, 6.3-0003, 6.3-0004: single op_identity lines. CireSnave ruled board 451 on 2026-11-16 (verbatim: \"451: 1: A.  2: A.  3: A.  4: A.\"). The field list and the field order are those of spec/ops.md 6.19.3 (also conformance/corpus/op_facts.json, `opattrs`); enum names are those of 6.19.2."));
    s.push_str(",\n  \"rulings_pinned\": ");
    s.push_str(&jstr("(1) The {...} list spells EVERY field of the op's OpAttrs record, defaults included, in the 6.19.3 canonical order; an op with no record is bare. (2) key = the field name; enum = its 6.19.2 name; number = canonical decimal (no sign, no leading zero); dtype = its KISS-Classify token, `compute-dtype` for an accumulator at the compute dtype; flag = true|false; axis set or window vector = [0,1], no spaces. An axis set is the explicit ascending list, never rall/rlast. (3) A reader decides only what the line shows: a missing field is `op-identity-incomplete`; whether a line was reduced to the bare name is a producer rule (6.3-0004). (4) A role-tuple suffix [role:dtype, ...] is read for shape only; the closed role set is KISS-Grammar's."));
    s.push_str(",\n  \"vectors\": [\n");
    let all = vectors();
    for (i, v) in all.iter().enumerate() {
        assert_eq!(verdict(v.line), v.expect, "vector {:?}: the reader disagrees with the stated verdict", v.name);
        let tail = match v.expect {
            Some(d) => format!(", \"verdict\": \"decline\", \"expect\": {}", jstr(d.wire_tag())),
            None => ", \"verdict\": \"accept\"".to_string(),
        };
        let comma = if i + 1 < all.len() { "," } else { "" };
        s.push_str(&format!("    {{\"name\": {}, \"op_identity\": {}{tail}}}{comma}\n", jstr(v.name), jstr(v.line)));
    }
    s.push_str("  ]\n}\n");
    s
}
