//! §6.6 Dispatch tests (D3 / #43): Dispatch is optional (geometry-agnostic class), and
//! `thread_mapping`/`addressing_rule` are thread-index-free structural declarations with the
//! richer form reserved post-v1.
//!
//! Oracle: [`kiss_conformance::dispatch`].
//!   * §6.6-0004 — thread_mapping/addressing_rule are thread-index-free (`test_contract_thread_and_addressing`)
//!   * §6.6-0007 — Dispatch is optional; a geometry-agnostic kernel declares none (`test_contract_dispatch_optional`)
//!   * §6.6-0008 — the richer tile-mapping / `gid` form is reserved post-v1 (`test_contract_reserved_tile_mapping`)

use kiss_conformance::dispatch::{is_thread_index_free, DispatchFields, DispatchModel};

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

/// §6.6-0007 — Dispatch is optional: a geometry-agnostic kernel declares none; a declared section
/// MUST carry all five fields (no partial).
#[test]
fn test_contract_dispatch_optional() {
    // Geometry-agnostic kernel: absent Dispatch is accepted, no geometry required.
    assert!(DispatchModel::GeometryAgnostic.validate().is_ok());

    // A fully-declared grid-stride Dispatch is accepted.
    assert!(DispatchModel::Declared(grid_stride_fields()).validate().is_ok());

    // A PARTIAL Dispatch section (a missing field) is rejected — all five or the sentinel.
    let mut partial = grid_stride_fields();
    partial.addressing_rule = "".into();
    assert!(DispatchModel::Declared(partial).validate().is_err());
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
    assert!(DispatchModel::GeometryAgnostic.validate().is_ok());

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
