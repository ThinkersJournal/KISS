// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform tests for the `op_identity` text spelling: Contract 6.11-0009 (the spelling), 6.3-0003 (the two
//! forms, no private op enum) and 6.3-0004 (distinguishing attributes are carried, never reduced to the bare name).
//! CireSnave ruled the spelling on board 451 (2026-11-16); the vectors are `corpus/op_identity_vectors.json`.

use kiss_conformance::op_identity::{check_op_identity, IdentityDecline};
use kiss_conformance::op_identity_vectors::{emit_op_identity_vectors_json, vectors, verdict};

const SPEC: &str = include_str!("../../spec/contract.md");
const COMMITTED: &str = include_str!("../corpus/op_identity_vectors.json");

/// Every `op_identity = <line>` the Contract spec shows, golden document and Appendix C alike.
fn spec_op_identity_lines() -> Vec<&'static str> {
    SPEC.lines().filter_map(|l| l.strip_prefix("op_identity = ")).collect()
}

/// Enforces[enforced]: KISS-CONTRACT-6.11-0009 — the `op_identity` value is `<op_name>[{<op_attrs>}][[<roles>]]`;
/// `{...}` spells every field of the op's OpAttrs record in the KISS-Ops canonical order with each value spelled by
/// its field's type; a reader MUST reject, with a typed decline, a value not matching this spelling.
///
/// TEETH: the published vectors (13 accepts, 30 declines over all eight decline tags) agree with the reader line by
/// line; the committed file is the generator's output byte for byte; and every `op_identity` line the Contract spec
/// itself shows is accepted, so a spec example that drifts from KISS-Ops (the `oob=` / `scan` defect) fails here.
// Proven: KISS-CONTRACT-6.11-0009 (subject: impl; ref: contract_op_identity.rs)
#[test]
fn test_contract_text_op_identity() {
    assert_eq!(COMMITTED, emit_op_identity_vectors_json(), "op_identity_vectors.json is stale: regenerate with the emit bin");
    let all = vectors();
    let accepts = all.iter().filter(|v| v.expect.is_none()).count();
    assert_eq!((accepts, all.len() - accepts), (13, 30));
    let mut tags = std::collections::BTreeSet::new();
    for v in &all {
        assert_eq!(verdict(v.line), v.expect, "vector {:?}", v.name);
        tags.extend(v.expect.map(IdentityDecline::wire_tag));
    }
    assert_eq!(tags.len(), 8, "every decline tag has a vector: {tags:?}");
    let shown = spec_op_identity_lines();
    assert!(shown.len() >= 3, "control: the spec shows op_identity lines, found {}", shown.len());
    for line in shown {
        assert_eq!(check_op_identity(line), Ok(()), "a line the Contract spec shows: {line:?}");
    }
}

/// Enforces[enforced]: KISS-CONTRACT-6.3-0003 — `op_identity` is exactly one of two forms and never a private op
/// enum: form (a) the full advertisable-op tag (op name, attributes, optional role tuple), form (b) the bare KISS-Ops
/// op name plus its attributes with no role-tuple suffix.
///
/// TEETH: form (a) with a role tuple and form (b) with none are both read; a name outside KISS-Ops (`scan`) and a
/// private spelling (`Add`) decline.
#[test]
fn test_contract_op_identity_full_tag_or_bare_ops_dag() {
    let a = "gather{axis=0, oob_policy=clamp, index_operand=1, index_dtype=i32}[data:f32, index:i32]";
    let b = "prefix_scan{monoid=sum, reduce_axes=[0], exclusivity=inclusive, accumulator=compute-dtype, math_fidelity=bit-stable}";
    assert_eq!(check_op_identity(a), Ok(()));
    assert_eq!(check_op_identity(b), Ok(()));
    assert_eq!(check_op_identity("scan{op=add}"), Err(IdentityDecline::UnknownOp));
    assert_eq!(check_op_identity("Add"), Err(IdentityDecline::Malformed));
}

/// Enforces[enforced]: KISS-CONTRACT-6.3-0004 — an `op_identity` over an op with distinguishing attribute values
/// carries the full tag and is not reduced to the bare op name. A reader decides only what the line shows: two
/// gathers that differ only by `oob_policy` are two accepted lines, and a carrier written bare or missing a field is
/// declined as incomplete.
///
/// TEETH: both gathers read, the bare name and the one-field-short line decline `op-identity-incomplete`.
#[test]
fn test_contract_op_identity_carries_distinguishing_attrs() {
    let skip = "gather{axis=0, oob_policy=skip, index_operand=1, index_dtype=i32}";
    let clamp = "gather{axis=0, oob_policy=clamp, index_operand=1, index_dtype=i32}";
    assert_eq!((check_op_identity(skip), check_op_identity(clamp)), (Ok(()), Ok(())));
    assert_ne!(skip, clamp);
    assert_eq!(check_op_identity("gather"), Err(IdentityDecline::Incomplete));
    assert_eq!(check_op_identity("gather{axis=0, index_operand=1, index_dtype=i32}"), Err(IdentityDecline::Incomplete));
}
