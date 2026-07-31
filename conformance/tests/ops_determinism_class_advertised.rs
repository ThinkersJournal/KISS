//! KISS-OPS-7.4-0001 via the contract codec: an implementation advertises, per op,
//! its determinism/fidelity class drawn from the single canonical §6.0 enum. Here
//! the advertisement rides a real Contract Guarantees block, round-tripped through
//! the codec; a token outside the canonical enum is a typed decline, never a
//! re-forked parallel class.

use kiss_conformance::contract::{
    field_line, parse_guarantees_class, ContractDecline, Value,
};
use kiss_conformance::DeterminismClass;

/// Build a minimal contract body: the pinned first heading (so it is not
/// Headingless) plus a Guarantees block carrying `determinism_class = <token>`.
fn body_with_class(token: &str) -> Vec<u8> {
    let mut b = b"[section:1:identity]\n".to_vec();
    b.extend_from_slice(b"[section:6:guarantees]\n");
    b.extend_from_slice(field_line("determinism_class", &Value::Str(token.into())).as_bytes());
    b
}

// Enforces KISS-OPS-7.4-0001: the advertised class is read from the canonical
// enum and an off-enum token is rejected, never re-forked into a new class.
#[test]
fn test_ops_determinism_class_advertised() {
    assert_eq!(
        parse_guarantees_class(&body_with_class("exact-byte")),
        Ok(DeterminismClass::ExactByte)
    );
    // NOTE: the canonical KISS-Ops §6.0-0001 wire token is `ULP/tolerance`
    // (capital ULP) — confirmed against spec/ops.md §6.0 and spec/contract.md
    // §6.8-0003, both of which spell it verbatim this way (also matched by the
    // existing reference constant in conformance/tests/contract_framing.rs and
    // conformance/tests/contract_schema.rs). NOT `ulp/tolerance`.
    assert_eq!(
        parse_guarantees_class(&body_with_class("ULP/tolerance")),
        Ok(DeterminismClass::UlpTolerance)
    );
    assert_eq!(
        parse_guarantees_class(&body_with_class("order-invariant/nondeterministic")),
        Ok(DeterminismClass::OrderInvariant)
    );
    // Off-enum token → typed decline, NOT a parallel class.
    assert!(matches!(
        parse_guarantees_class(&body_with_class("bit-exact-ish")),
        Err(ContractDecline::UnknownDeterminismClass { .. })
    ));
    // A lowercase re-spelling of the ULP member is ALSO off-enum — the codec
    // MUST NOT silently re-spell/re-fork the canonical token (KISS-OPS §7.4-0001).
    assert!(matches!(
        parse_guarantees_class(&body_with_class("ulp/tolerance")),
        Err(ContractDecline::UnknownDeterminismClass { .. })
    ));
    // No Guarantees class field → typed decline.
    assert_eq!(
        parse_guarantees_class(b"[section:1:identity]\n"),
        Err(ContractDecline::MissingGuaranteesClass)
    );
}

// The Capabilities block (§6.7-0004) also carries a `determinism_class` field;
// parse_guarantees_class must read the GUARANTEES value, not the earlier
// Capabilities one. Regression-guards the block-scoping against a refactor back
// to an unscoped whole-body scan.
#[test]
fn guarantees_class_not_confused_with_capabilities() {
    let mut body = b"[section:1:identity]\n".to_vec();
    body.extend_from_slice(b"[section:5:capabilities]\n");
    body.extend_from_slice(b"determinism_class = exact-byte\n");
    body.extend_from_slice(b"[section:6:guarantees]\n");
    body.extend_from_slice(b"determinism_class = order-invariant/nondeterministic\n");
    assert_eq!(
        parse_guarantees_class(&body),
        Ok(DeterminismClass::OrderInvariant)
    );
}
