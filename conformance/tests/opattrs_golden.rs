//! KISS-Conform golden-vector suite for the KISS-Ops OpAttrs encoding (Ops §6.19).
//!
//! Every expected hex string is transcribed verbatim from Ops Appendix E ("bytes
//! on the wire, left to right"); the reference encoder in `kiss_conformance::opattrs`
//! must reproduce it byte-for-byte under the exact-byte comparator (Conform §6.4,
//! §6.8). Each test cites its pinning clause.

use kiss_conformance::assert_golden;
use kiss_conformance::opattrs::*;

// ---- E.2 full per-op OpAttrs blobs ------------------------------------------

#[test]
fn e2_reduce_sum_all_axes_keepdim() {
    // reduce(sum, all-axes, keepdim) over a rank-2 operand -> 01 FF FF 01
    let b = encode_reduce(Monoid::Sum, &[0, 1], 2, true);
    assert_golden("KISS-OPS-6.19-0025", "reduce_sum_all_keepdim", &b, "01 FF FF 01");
}

#[test]
fn e2_gather_clamp_nondefault() {
    // gather(axis=0, oob=clamp, index_operand=1, index_dtype=i32) -> 00 02 01 02
    let b = encode_gather(0, OobPolicy::Clamp, 1, IndexDtype::I32);
    assert_golden("KISS-OPS-6.19-0027", "gather_clamp", &b, "00 02 01 02");
}

#[test]
fn e2_prefix_scan_trailing_inclusive() {
    // prefix_scan(sum, trailing-axis, inclusive) over rank>1 -> 01 FE FF 01
    let b = encode_prefix_scan(Monoid::Sum, 1, 2, ScanExclusivity::Inclusive);
    assert_golden("KISS-OPS-6.19-0026", "prefix_scan_trailing_incl", &b, "01 FE FF 01");
}

#[test]
fn e2_scatter_atomic_add() {
    // scatter(axis=0, atomic-add, skip, index_operand=1, i64) -> 00 02 01 01 03
    let b = encode_scatter(0, ScatterCombine::AtomicAdd, OobPolicy::Skip, 1, IndexDtype::I64);
    assert_golden("KISS-OPS-6.19-0028", "scatter_atomic_add", &b, "00 02 01 01 03");
}

#[test]
fn e2_reduce_var_population() {
    // reduce_var(all-axes, keepdim, population) -> FF FF 01 00
    let b = encode_reduce_var(&[0, 1], 2, true, false);
    assert_golden("KISS-OPS-6.19-0030", "reduce_var_population", &b, "FF FF 01 00");
}

#[test]
fn e2_reduce_rank1_sole_axis_is_all_axes() {
    // reduce(sum) over a rank-1 sole axis MUST encode FF FF (all-axes precedence),
    // never the single-bit subset mask 01 00 -> 01 FF FF 01
    let b = encode_reduce(Monoid::Sum, &[0], 1, true);
    assert_golden("KISS-OPS-6.19-0020", "reduce_rank1_all", &b, "01 FF FF 01");
}

#[test]
fn e2_reduce_rank3_full_is_all_axes() {
    // reduce(max) over a rank-3 operand's {0,1,2} MUST encode FF FF, never 07 00
    let b = encode_reduce(Monoid::Max, &[0, 1, 2], 3, true);
    assert_golden("KISS-OPS-6.19-0020", "reduce_rank3_all", &b, "03 FF FF 01");
}

#[test]
fn e2_sort_network_ascending_stable() {
    // sort_network(ascending, stable) over rank-2: axis=trailing(1) -> 01 01 01
    let b = encode_sort_network(1, SortDirection::Ascending, true);
    assert_golden("KISS-OPS-6.19-0029", "sort_asc_stable", &b, "01 01 01");
}

#[test]
fn e2_softmax_norm_axis() {
    let b = encode_softmax(1);
    assert_golden("KISS-OPS-6.19-0031", "softmax_axis1", &b, "01");
}

#[test]
fn e2_index_select_u32() {
    let b = encode_index_select(0, 1, IndexDtype::U32);
    assert_golden("KISS-OPS-6.19-0034", "index_select_u32", &b, "00 01 01");
}

#[test]
fn e2_avg_pool_square_37_bytes() {
    // avg_pool window=[2,2] stride=[2,2] dilation=[1,1] padding=[0,0] cip=true
    let b = encode_avg_pool(&[2, 2], &[2, 2], &[1, 1], &[0, 0], true);
    assert_golden(
        "KISS-OPS-6.19-0032",
        "avg_pool_2x2",
        &b,
        "02 02 00 00 00 02 00 00 00 02 02 00 00 00 02 00 00 00 \
         02 01 00 00 00 01 00 00 00 02 00 00 00 00 00 00 00 00 01",
    );
    assert_eq!(b.len(), 37);
}

#[test]
fn e2_avg_pool_nonsquare_preserves_axis_order() {
    // window=[3,5] stride=[1,2] — a reversed encoder emitting [5,3]/[2,1] diverges
    let b = encode_avg_pool(&[3, 5], &[1, 2], &[1, 1], &[0, 0], true);
    assert_golden(
        "KISS-OPS-6.19-0023",
        "avg_pool_3x5",
        &b,
        "02 03 00 00 00 05 00 00 00 02 01 00 00 00 02 00 00 00 \
         02 01 00 00 00 01 00 00 00 02 00 00 00 00 00 00 00 00 01",
    );
}

#[test]
fn e2_im2col_channel_major() {
    // im2col window=[3,3] stride=[1,1] dilation=[1,1] padding=[1,1] channel-major
    let b = encode_im2col(&[3, 3], &[1, 1], &[1, 1], &[1, 1], ColumnOrdering::ChannelMajor);
    assert_golden(
        "KISS-OPS-6.19-0033",
        "im2col_3x3_pad1",
        &b,
        "02 03 00 00 00 03 00 00 00 02 01 00 00 00 01 00 00 00 \
         02 01 00 00 00 01 00 00 00 02 01 00 00 00 01 00 00 00 01",
    );
}

// ---- E.1 sub-vocabulary ordinals + reduce_axes multiplex --------------------

#[test]
fn e1_sub_vocabulary_ordinals() {
    assert_eq!(Monoid::Sum.ordinal(), 0x01);
    assert_eq!(Monoid::Min.ordinal(), 0x04);
    assert_eq!(OobPolicy::ZeroFill.ordinal(), 0x03);
    assert_eq!(ScatterCombine::AtomicMin.ordinal(), 0x04);
    assert_eq!(IndexDtype::I64.ordinal(), 0x03);
    assert_eq!(SortDirection::Descending.ordinal(), 0x02);
    assert_eq!(ScanExclusivity::Exclusive.ordinal(), 0x02);
    assert_eq!(ColumnOrdering::TapMajor.ordinal(), 0x02);
}

#[test]
fn e1_reduce_axes_multiplex() {
    // E.1: all-axes / trailing(rank>1) / subset{axis0} / subset{axis0,axis2}
    assert_eq!(reduce_axes_reduction(&[0, 1], 2), 0xFFFF); // all-axes
    assert_eq!(reduce_axes_reduction(&[1], 2), 0xFFFE); // trailing, rank>1
    assert_eq!(reduce_axes_reduction(&[0], 2), 0x0001); // subset{axis0} -> 01 00 LE
    assert_eq!(reduce_axes_reduction(&[0, 2], 3), 0x0005); // subset{axis0,axis2} -> 05 00 LE
    // precedence: a full set is all-axes, never a subset mask
    assert_eq!(reduce_axes_reduction(&[0], 1), 0xFFFF); // rank-1 sole axis
    assert_eq!(reduce_axes_reduction(&[0, 1, 2], 3), 0xFFFF); // never 0x0007
    // scan: trailing rank>1 -> FFFE; non-trailing -> single-bit mask
    assert_eq!(reduce_axes_scan(1, 2), 0xFFFE);
    assert_eq!(reduce_axes_scan(0, 2), 0x0001);
    assert_eq!(reduce_axes_scan(0, 1), 0x0001); // rank-1 sole axis (scan never all-axes)
}

// ---- E.3 default-resolution equality (§6.19-0005) ---------------------------

#[test]
fn e3_default_resolution_byte_equality() {
    // A gather written with the default oob_policy (skip) and one written
    // explicitly skip — all other fields equal — emit identical bytes. This is
    // what lets KISS-Grammar byte-compare the blob without interpreting it.
    // The encoder takes *resolved* values, so "defaulted" resolves to Skip first.
    let resolved_default: OobPolicy = OobPolicy::Skip; // resolve(None) -> skip
    let explicit: OobPolicy = OobPolicy::Skip;
    let a = encode_gather(0, resolved_default, 1, IndexDtype::U32);
    let b = encode_gather(0, explicit, 1, IndexDtype::U32);
    assert_eq!(a, b, "default-resolved and explicit-equal blobs must be byte-identical");
    assert_golden("KISS-OPS-6.19-0005", "gather_default_skip", &a, "00 01 01 01");
}

// ---- Conform modality 4: negative / decline vectors (§6.7) ------------------

#[test]
fn negative_reserved_zero_ordinal_declines() {
    // A gather blob carrying the reserved oob_policy ordinal 0 must be refused
    // with a typed decline, never a panic.
    let malformed = [0x00u8, 0x00, 0x01, 0x02]; // oob_policy = 0 (reserved)
    assert_eq!(
        decode_gather(&malformed),
        Err(Decline::ReservedZeroOrdinal { field: "oob_policy" })
    );
}

#[test]
fn negative_truncated_blob_declines() {
    assert_eq!(decode_gather(&[0x00, 0x01, 0x01]), Err(Decline::TruncatedBlob { need: 4, got: 3 }));
}

#[test]
fn negative_reserved_reduce_axes_band_declines() {
    assert_eq!(validate_reduce_axes(0x0100), Err(Decline::ReservedReduceAxes { value: 0x0100 }));
    assert_eq!(validate_reduce_axes(0xFFFD), Err(Decline::ReservedReduceAxes { value: 0xFFFD }));
    assert!(validate_reduce_axes(0xFFFE).is_ok()); // trailing-axis is valid
    assert!(validate_reduce_axes(0x00FF).is_ok()); // subset mask is valid
}

/// KISS-OPS-6.19-0038 (`test_ops_opattrs_reduce_axes_zero_unreachable`): the
/// none/not-a-reduction sentinel 0x0000 is unreachable on the OpAttrs channel —
/// every carrier op that owns `reduce_axes` is a reduction or a scan — so a
/// reader MUST reject it as malformed. Catches a reader that accepts 0x0000
/// because it only guards the reserved band (0x0100..=0xFFFD) and lets the
/// sentinel through: such a reader treats a scan-over-nothing blob as valid.
#[test]
fn test_ops_opattrs_reduce_axes_zero_unreachable() {
    assert_eq!(
        validate_reduce_axes(0x0000),
        Err(Decline::ReservedReduceAxes { value: 0x0000 })
    );
    // the neighbouring live values remain valid, so the guard is not over-broad:
    assert!(validate_reduce_axes(0x0001).is_ok()); // single-axis subset mask
    assert!(validate_reduce_axes(0xFFFF).is_ok()); // all-axes sentinel
}

#[test]
fn positive_decode_roundtrip() {
    let (axis, oob, idx_op, idt) = decode_gather(&[0x00, 0x02, 0x01, 0x02]).unwrap();
    assert_eq!((axis, oob, idx_op, idt), (0, OobPolicy::Clamp, 1, IndexDtype::I32));
}

/// KISS-OPS-6.19-0037 (`test_ops_opattrs_max_rank_operands_pinned`): the OpAttrs
/// channel pins MAX_RANK = 8 and MAX_OPERANDS = 8, and §6.19-0027 bounds a
/// gather's `axis` to `0..MAX_RANK-1` and `index_operand` to `0..MAX_OPERANDS-1`.
/// Catches a decoder that returns these fields unchecked (the previous one did):
/// axis=8 or index_operand=8 would index a nonexistent axis/operand in a
/// downstream consumer, so a reader MUST reject the blob rather than pass it on.
#[test]
fn test_ops_opattrs_max_rank_operands_pinned() {
    // the constants are the pinned concrete values (§6.19-0037).
    assert_eq!(MAX_RANK, 8);
    assert_eq!(MAX_OPERANDS, 8);
    // the last in-range values are accepted ...
    assert!(decode_gather(&[7, 0x01, 7, 0x01]).is_ok());
    // ... and the first out-of-range value of each bounded field is rejected.
    assert_eq!(
        decode_gather(&[8, 0x01, 0, 0x01]),
        Err(Decline::FieldOutOfRange { field: "axis", value: 8, max: 8 })
    );
    assert_eq!(
        decode_gather(&[0, 0x01, 8, 0x01]),
        Err(Decline::FieldOutOfRange { field: "index_operand", value: 8, max: 8 })
    );
}
