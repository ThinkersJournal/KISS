// SPDX-License-Identifier: MIT OR Apache-2.0
//! §6.6 Dispatch tests (D3 / #43): Dispatch is optional (geometry-agnostic class), and
//! `thread_mapping`/`addressing_rule` are thread-index-free structural declarations with the
//! richer form reserved post-v1.
//!
//! Oracle: [`kiss_conformance::dispatch`].
//!   * §6.6-0004 — thread_mapping/addressing_rule are thread-index-free (`test_contract_thread_and_addressing`)
//!   * §6.6-0007 — the Dispatch section is always present; a geometry-agnostic kernel carries the
//!     `dispatch_model = geometry-agnostic` sentinel (`test_contract_dispatch_optional`)
//!   * §6.6-0008 — the richer tile-mapping / `gid` form is reserved post-v1 (`test_contract_reserved_tile_mapping`)

use kiss_conformance::dispatch::{
    is_thread_index_free, parse_dispatch_block, parse_dispatch_from_document, DispatchDecline,
    DispatchFields, DispatchModel, GEOMETRY_FIELDS,
};

/// A standard grid-stride Dispatch: thread-index-free coefficients only.
fn grid_stride_fields() -> DispatchFields {
    DispatchFields {
        invocation_domain: "extents[0]".into(),
        workgroup_sizing: "256".into(),
        count_to_grid: "ceil_div(n, 256)".into(),
        thread_mapping: "n".into(),           // grid-stride constant (total threads)
        addressing_rule: "strides[0]".into(), // per-operand signed-stride coefficient (§6.6-0006 sym[k])
    }
}

/// §6.6-0007 — the Dispatch section is ALWAYS present (§6.11-0004); a geometry-agnostic kernel
/// carries the sentinel line `dispatch_model = geometry-agnostic` in place of the five geometry
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

    // The declared five-field block also round-trips byte-for-byte (the accept control, made real).
    let declared = DispatchModel::Declared(grid_stride_fields());
    assert_eq!(parse_dispatch_block(&declared.render_block()), Ok(declared.clone()));

    // (3) DECLINE — the either/or schema (§6.6-0001): a proper subset of the five, and
    // `dispatch_model` alongside a geometry field, each decline (never a panic).
    let partial = b"[section:4:dispatch]\ninvocation_domain = extents[0]\nworkgroup_sizing = 256\n";
    assert_eq!(parse_dispatch_block(partial), Err(DispatchDecline::PartialGeometry));
    let both =
        b"[section:4:dispatch]\ndispatch_model = geometry-agnostic\ninvocation_domain = extents[0]\n";
    assert_eq!(parse_dispatch_block(both), Err(DispatchDecline::SentinelWithGeometry));
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
    assert!(DispatchModel::Declared(grid_stride_fields()).validate().is_ok());
    // KISS#380: "all five fields" is demonstrated for EACH of the five, not one. The list is the
    // validator's own (`GEOMETRY_FIELDS`), and the exhaustive destructure below fails to compile if
    // the struct gains a field the list does not name, so a sixth field cannot be added unnoticed.
    let DispatchFields { invocation_domain: _, workgroup_sizing: _, count_to_grid: _, thread_mapping: _, addressing_rule: _ } =
        grid_stride_fields();
    assert_eq!(
        GEOMETRY_FIELDS,
        ["invocation_domain", "workgroup_sizing", "count_to_grid", "thread_mapping", "addressing_rule"],
        "§6.6-0001: the five geometry fields in wire order"
    );
    for blank in GEOMETRY_FIELDS {
        for empty in ["", "   "] {
            let mut partial = grid_stride_fields();
            partial.set_field(blank, empty);
            assert!(
                DispatchModel::Declared(partial).validate().is_err(),
                "§6.6-0007: a Dispatch section with `{blank}` = {empty:?} was accepted"
            );
        }
    }
}

/// §6.6-0004 — thread_mapping/addressing_rule are thread-index-free structural declarations; a
/// per-thread index symbol is not spellable in v1.
#[test]
fn test_contract_thread_and_addressing() {
    // Thread-index-free coefficients (grid-stride constant + per-axis signed stride) accept.
    assert!(DispatchModel::Declared(grid_stride_fields()).validate().is_ok());

    // A thread_mapping that reaches for a per-thread index (`gid`) is rejected — the v1 grammar
    // has no such symbol (§6.6-0008 reserves it post-v1).
    let mut with_gid = grid_stride_fields();
    with_gid.thread_mapping = "gid + k".into();
    assert!(DispatchModel::Declared(with_gid).validate().is_err());

    // Likewise an addressing_rule using a lane/thread index is rejected.
    let mut with_lane = grid_stride_fields();
    with_lane.addressing_rule = "strides[0] * lane".into();
    assert!(DispatchModel::Declared(with_lane).validate().is_err());

    // A launch scalar that merely CONTAINS index-like text (`idx_extents`) is NOT a thread index.
    assert!(is_thread_index_free("idx_extents[0] * strides[1]"));
}

/// §6.6-0008 — the richer thread-mapping form (tile-mapping / bound `gid`) is reserved post-v1:
/// not required for v1 conformance, and not a valid v1 declaration.
#[test]
fn test_contract_reserved_tile_mapping() {
    // A v1 grid-stride kernel (and a geometry-agnostic kernel) is fully valid WITHOUT the reserved
    // form — it is not required in v1.
    assert!(DispatchModel::Declared(grid_stride_fields()).validate().is_ok());
    // The geometry-agnostic control, as a DOCUMENT claim (KISS#490): the old line here
    // (`GeometryAgnostic.validate().is_ok()`) was `Ok(()) == Ok(())` and could not fail. The
    // intended claim is that the sentinel block alone is a complete, valid v1 Dispatch section,
    // and that a reserved form beside it is not.
    let sentinel = kiss_conformance::contract::appendix_c_dispatch_block();
    assert_eq!(parse_dispatch_block(&sentinel), Ok(DispatchModel::GeometryAgnostic));
    let mut with_reserved = sentinel.clone();
    with_reserved.extend_from_slice(b"thread_mapping = tile_mapping
");
    assert!(
        parse_dispatch_block(&with_reserved).is_err(),
        "the sentinel plus a reserved form is not a valid v1 Dispatch section"
    );

    // The reserved forms (a named `tile_mapping`, a bound `gid`) are NOT valid v1 declarations.
    let mut tiled = grid_stride_fields();
    tiled.thread_mapping = "tile_mapping".into();
    assert!(DispatchModel::Declared(tiled).validate().is_err());

    let mut gid_map = grid_stride_fields();
    gid_map.thread_mapping = "gid".into();
    assert!(DispatchModel::Declared(gid_map).validate().is_err());

    // The reserved symbols/forms are recognized as non-thread-index-free.
    assert!(!is_thread_index_free("gid"));
    assert!(!is_thread_index_free("tile_mapping"));
}
