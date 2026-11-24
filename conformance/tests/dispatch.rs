// SPDX-License-Identifier: MIT OR Apache-2.0
//! §6.6 Dispatch tests (D3 / #43): Dispatch is optional (geometry-agnostic class), and a declared
//! launch geometry is exactly the three fields `{invocation_domain, workgroup_sizing, count_to_grid}`.
//!
//! Oracle: [`kiss_conformance::dispatch`].
//!   * §6.6-0001 — exactly three geometry fields, or the sentinel; the retired `thread_mapping` and
//!     `addressing_rule` are unknown fields now (board 476)
//!   * §6.6-0007 — the Dispatch section is always present; a geometry-agnostic kernel carries the
//!     `dispatch_model = geometry-agnostic` sentinel (`test_contract_dispatch_optional`)

use kiss_conformance::dispatch::{
    parse_dispatch_block, parse_dispatch_from_document, DispatchDecline, DispatchFields, DispatchModel,
    GEOMETRY_FIELDS,
};

/// A standard declared Dispatch: the three launch fields.
fn declared_fields() -> DispatchFields {
    DispatchFields {
        invocation_domain: "extents[0]".into(),
        workgroup_sizing: "256".into(),
        count_to_grid: "ceil_div(n, 256)".into(),
    }
}

/// §6.6-0007 — the Dispatch section is ALWAYS present (§6.11-0004); a geometry-agnostic kernel
/// carries the sentinel line `dispatch_model = geometry-agnostic` in place of the three geometry
/// fields (#456/#480). This exercises the obligation AT THE BYTE LEVEL — the four #456 arms — where
/// the prior `Ok(()) == Ok(())` assertion was vacuous: it returned `Ok` unconditionally and never
/// touched a serialized document, while every word of §6.6-0007 is about bytes.
#[test]
fn test_contract_dispatch_optional() {
    // (1) ROUND-TRIP — emit the geometry-agnostic block, parse it back to GeometryAgnostic; no
    // geometry field is required or present.
    let sentinel = DispatchModel::GeometryAgnostic.render_block();
    assert_eq!(parse_dispatch_block(&sentinel), Ok(DispatchModel::GeometryAgnostic));

    // (2) BYTE-PIN — the emitted block is exactly the pinned bytes (§6.11-0004 heading + the one
    // sentinel field line). A wrong emitter (absent block, or a different key/value) fails here.
    assert_eq!(
        sentinel,
        b"[section:4:dispatch]\ndispatch_model = geometry-agnostic\n".to_vec(),
        "§6.6-0007: the carried geometry-agnostic block is not the pinned bytes"
    );

    // The declared three-field block also round-trips byte-for-byte (the accept control, made real).
    let declared = DispatchModel::Declared(declared_fields());
    assert_eq!(parse_dispatch_block(&declared.render_block()), Ok(declared.clone()));

    // (3) DECLINE — the either/or schema (§6.6-0001): a proper subset of the three, and
    // `dispatch_model` alongside a geometry field, each decline (never a panic).
    let partial = b"[section:4:dispatch]\ninvocation_domain = extents[0]\nworkgroup_sizing = 256\n";
    assert_eq!(parse_dispatch_block(partial), Err(DispatchDecline::PartialGeometry));
    let both =
        b"[section:4:dispatch]\ndispatch_model = geometry-agnostic\ninvocation_domain = extents[0]\n";
    assert_eq!(parse_dispatch_block(both), Err(DispatchDecline::SentinelWithGeometry));
    // the retired fields are not in the schema: a `thread_mapping` or `addressing_rule` line beside the
    // three launch fields is an unknown field, never accepted and never ignored (board 476).
    for retired in ["thread_mapping = n", "addressing_rule = strides[0]"] {
        let with_retired = format!(
            "[section:4:dispatch]
invocation_domain = extents[0]
workgroup_sizing = 256
count_to_grid = ceil_div(n, 256)
{retired}
"
        );
        assert!(
            matches!(parse_dispatch_block(with_retired.as_bytes()), Err(DispatchDecline::UnknownField(_))),
            "§6.6-0001: the retired field line `{retired}` was not declined as unknown"
        );
    }
    let bad_sentinel = b"[section:4:dispatch]\ndispatch_model = provider-internal\n";
    assert!(matches!(
        parse_dispatch_block(bad_sentinel),
        Err(DispatchDecline::BadSentinelValue(_))
    ));

    // (4) DISCRIMINATE — the carried-vs-absent distinction. A document that OMITS the Dispatch block
    // MUST decline (§6.11-0004 requires all seven blocks); the wrong impl (the old "absent sentinel"
    // reading) would ACCEPT it. A document that CARRIES the sentinel block parses to GeometryAgnostic.
    let doc_without = b"[section:1:identity]\nname = k\n[section:3:interface]\nabi = c\n";
    assert_eq!(
        parse_dispatch_from_document(doc_without),
        Err(DispatchDecline::MissingSection),
        "an omitted Dispatch block must decline — the carried resolution forbids the absent form"
    );
    let doc_with = b"[section:1:identity]\nname = k\n[section:4:dispatch]\ndispatch_model = geometry-agnostic\n[section:5:capabilities]\ncost = 1\n";
    assert_eq!(parse_dispatch_from_document(doc_with), Ok(DispatchModel::GeometryAgnostic));
    // the heading search is LINE-ANCHORED: `[section:4:dispatch]` occurring INSIDE a field value is
    // not a section heading. An unanchored `find` would match it here and then mis-parse; the
    // anchored search correctly reports the section absent.
    let doc_in_value =
        b"[section:1:identity]\nname = see [section:4:dispatch] in the docs\n[section:5:capabilities]\ncost = 1\n";
    assert_eq!(
        parse_dispatch_from_document(doc_in_value),
        Err(DispatchDecline::MissingSection),
        "an unanchored heading search would falsely match `[section:4:dispatch]` inside a field value"
    );

    // The two pre-existing non-vacuous enum-level assertions stay: a full declaration validates,
    // and a partial one (an empty field) is rejected (§6.6-0007 no-partial).
    assert!(DispatchModel::Declared(declared_fields()).validate().is_ok());
    // KISS#380: "all three fields" is demonstrated for EACH of the three, not one. The list is the
    // validator's own (`GEOMETRY_FIELDS`), and the exhaustive destructure below fails to compile if
    // the struct gains a field the list does not name, so a fourth field cannot be added unnoticed.
    let DispatchFields { invocation_domain: _, workgroup_sizing: _, count_to_grid: _ } = declared_fields();
    assert_eq!(
        GEOMETRY_FIELDS,
        ["invocation_domain", "workgroup_sizing", "count_to_grid"],
        "§6.6-0001: the three geometry fields in wire order"
    );
    for blank in GEOMETRY_FIELDS {
        for empty in ["", "   "] {
            let mut partial = declared_fields();
            partial.set_field(blank, empty);
            assert!(
                DispatchModel::Declared(partial).validate().is_err(),
                "§6.6-0007: a Dispatch section with `{blank}` = {empty:?} was accepted"
            );
        }
    }
}
