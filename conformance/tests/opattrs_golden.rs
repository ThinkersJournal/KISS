//! KISS-Conform golden-vector suite for the KISS-Ops OpAttrs encoding (Ops §6.19).
//!
//! Every expected hex string is transcribed verbatim from Ops Appendix E ("bytes
//! on the wire, left to right"); the reference encoder in `kiss_conformance::opattrs`
//! must reproduce it byte-for-byte under the exact-byte comparator (Conform §6.4,
//! §6.8). Each test cites its pinning clause.

use kiss_conformance::assert_golden;
use kiss_conformance::opattrs::*;
use kiss_conformance::structure_key::{Reduce, StructureKey, WorkClass};

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

// ---- §6.19.1 general encoding invariants ------------------------------------
//
// The §6.19.3 per-op golden vectors above pin each schema's concrete bytes; the
// tests below pin the cross-cutting wire invariants that every schema obeys —
// field-order ABI, reserve-0 ordinals, fixed width, little-endianness, explicit
// slots, definite lengths, opaque byte-compare, boolean 0/1, the reserved
// permutation sub-vocabulary — plus the §6.19.4 Classify reconciliation. Each is
// mutation-tight: a plausible codec drift moves a byte / a length / a category.

/// KISS-OPS-6.19-0004 (`test_ops_opattrs_field_order_abi`): a carrier op's blob is
/// exactly its schema fields concatenated in the frozen canonical field order, that
/// per-op order IS the ABI, and attribute names never appear on the wire (no
/// name-sorted dictionary, no self-describing tag stream). Encoded with per-field
/// distinct values so a swapped/reordered stream lands a different byte at the
/// affected offsets, and the exact length rules out any name/tag framing. MUTATION:
/// emit `oob_policy` before `axis`, or a tagged self-describing stream -> the
/// positional bytes move or the length grows -> fails.
#[test]
fn test_ops_opattrs_field_order_abi() {
    // gather schema order = axis | oob_policy | index_operand | index_dtype
    let g = encode_gather(3, OobPolicy::Clamp, 5, IndexDtype::I64);
    assert_golden("KISS-OPS-6.19-0004", "gather_field_order", &g, "03 02 05 03");
    assert_eq!(g.len(), 4, "gather blob is exactly its four schema fields, no name/tag framing");
    assert_eq!(g[0], 0x03, "byte 0 is axis");
    assert_eq!(g[1], 0x02, "byte 1 is oob_policy=clamp");
    assert_eq!(g[2], 0x05, "byte 2 is index_operand");
    assert_eq!(g[3], 0x03, "byte 3 is index_dtype=i64");
    // scatter schema order = axis | combine | oob_policy | index_operand | index_dtype
    let s = encode_scatter(2, ScatterCombine::AtomicMax, OobPolicy::Skip, 4, IndexDtype::I32);
    assert_golden("KISS-OPS-6.19-0004", "scatter_field_order", &s, "02 03 01 04 02");
    assert_eq!(s.len(), 5, "scatter blob is exactly its five schema fields");
    assert_eq!(s[1], 0x03, "byte 1 is combine=atomic-max (distinct from the axis byte)");
    assert_eq!(s[2], 0x01, "byte 2 is oob_policy=skip");
}

/// KISS-OPS-6.19-0006 (`test_ops_opattrs_enum_ordinal_reserve_zero`): every §6.19.2
/// enumerated sub-vocabulary reserves ordinal 0 as the invalid/unspecified sentinel
/// a conforming encoder never emits, and a reader rejects a 0 ordinal with a typed
/// decline. MUTATION: assign ordinal 0 to any declared variant -> the `!= 0` loop
/// fails; make the reader accept a 0 ordinal -> the decline legs fail.
#[test]
fn test_ops_opattrs_enum_ordinal_reserve_zero() {
    // Every declared variant of every §6.19.2 enum carries a non-zero ordinal (0 is
    // the reserved sentinel). Asserting the reserve property, not the concrete
    // values, so this is not a hardcoded copy of the codec.
    for o in [Monoid::Sum, Monoid::Prod, Monoid::Max, Monoid::Min].map(Monoid::ordinal) {
        assert_ne!(o, 0, "Monoid reserves ordinal 0");
    }
    for o in [OobPolicy::Skip, OobPolicy::Clamp, OobPolicy::ZeroFill].map(OobPolicy::ordinal) {
        assert_ne!(o, 0, "OobPolicy reserves ordinal 0");
    }
    for o in [
        ScatterCombine::Assign,
        ScatterCombine::AtomicAdd,
        ScatterCombine::AtomicMax,
        ScatterCombine::AtomicMin,
    ]
    .map(ScatterCombine::ordinal)
    {
        assert_ne!(o, 0, "ScatterCombine reserves ordinal 0");
    }
    for o in [IndexDtype::U32, IndexDtype::I32, IndexDtype::I64].map(IndexDtype::ordinal) {
        assert_ne!(o, 0, "IndexDtype reserves ordinal 0");
    }
    for o in [SortDirection::Ascending, SortDirection::Descending].map(SortDirection::ordinal) {
        assert_ne!(o, 0, "SortDirection reserves ordinal 0");
    }
    for o in [ScanExclusivity::Inclusive, ScanExclusivity::Exclusive].map(ScanExclusivity::ordinal) {
        assert_ne!(o, 0, "ScanExclusivity reserves ordinal 0");
    }
    for o in [ColumnOrdering::ChannelMajor, ColumnOrdering::TapMajor].map(ColumnOrdering::ordinal) {
        assert_ne!(o, 0, "ColumnOrdering reserves ordinal 0");
    }
    // The reader rejects a 0 in an enum slot with a typed, field-named decline.
    assert_eq!(
        decode_gather(&[0x00, 0x00, 0x01, 0x02]),
        Err(Decline::ReservedZeroOrdinal { field: "oob_policy" }),
        "a 0 oob_policy ordinal is rejected"
    );
    assert_eq!(
        decode_gather(&[0x00, 0x01, 0x01, 0x00]),
        Err(Decline::ReservedZeroOrdinal { field: "index_dtype" }),
        "a 0 index_dtype ordinal is rejected"
    );
}

/// KISS-OPS-6.19-0007 (`test_ops_opattrs_int_fixed_width_le`): every integer OpAttrs
/// field is encoded at its schema-pinned fixed width (not chosen by magnitude) —
/// `reduce_axes` as `u16`, window elements as `u32` — low-byte-first. MUTATION: a
/// magnitude/varint-width encoder emits the small value `0x0005` in one byte or a
/// window element in two -> the total length and the fixed-width byte spans shift.
#[test]
fn test_ops_opattrs_int_fixed_width_le() {
    // reduce(sum, subset{0,2} of rank 3, keepdim): reduce_axes is a fixed u16 that
    // occupies exactly bytes[1..3], even though its value 0x0005 fits in one byte.
    let b = encode_reduce(Monoid::Sum, &[0, 2], 3, true);
    assert_eq!(b.len(), 4, "monoid(1) + reduce_axes(2) + keepdim(1) = 4 fixed bytes");
    assert_eq!(&b[1..3], &[0x05, 0x00], "reduce_axes is a fixed-width u16, not a 1-byte varint");
    // a window element is a fixed u32 (4 bytes), even for the value 2.
    let w = window_param_vector(&[2]);
    assert_eq!(w.len(), 5, "count(1) + one u32 element(4) = 5");
    assert_eq!(&w[1..5], &[0x02, 0x00, 0x00, 0x00], "the element is a fixed-width u32, not 2 bytes");
}

/// KISS-OPS-6.19-0008 (`test_ops_opattrs_little_endian`): the canonical wire form is
/// little-endian throughout. Pinned with values whose LE and BE byte orders differ,
/// so this is distinct from the WIDTH pin of §6.19-0007. MUTATION: a big-endian
/// encoder reverses the multi-byte fields -> the exact byte sequences fail.
#[test]
fn test_ops_opattrs_little_endian() {
    // a u32 window element 0x04030201 is byte-reversed under LE.
    let w = window_param_vector(&[0x0403_0201]);
    assert_eq!(w, vec![0x01, 0x01, 0x02, 0x03, 0x04], "u32 element is little-endian");
    // the trailing-axis reduce_axes sentinel 0xFFFE serializes low-byte-first.
    let ps = encode_prefix_scan(Monoid::Sum, 1, 2, ScanExclusivity::Inclusive);
    assert_eq!(&ps[1..3], &[0xFE, 0xFF], "reduce_axes 0xFFFE is little-endian (not FF FE)");
}

/// KISS-OPS-6.19-0009 (`test_ops_opattrs_optional_explicit_slot`): a field at its
/// resolved default still occupies an explicit slot — presence is never signalled by
/// omission. Distinct from §6.19-0005 (which pins default==explicit EQUALITY); this
/// pins PRESENCE/length. MUTATION: an omit-if-equals-default encoder drops the
/// defaulted byte -> the length shrinks and the slot vanishes.
#[test]
fn test_ops_opattrs_optional_explicit_slot() {
    // gather with oob_policy resolved to its default (skip): the slot is still emitted.
    let g = encode_gather(0, OobPolicy::Skip, 1, IndexDtype::U32);
    assert_eq!(g.len(), 4, "the defaulted oob_policy is present, not elided");
    assert_eq!(g[1], 0x01, "byte 1 is the explicitly-emitted default skip ordinal");
    // reduce_var with bessel_correction at its population default (false): still emitted.
    let rv = encode_reduce_var(&[0, 1], 2, true, false);
    assert_eq!(rv.len(), 4, "reduce_axes(2) + keepdim(1) + bessel(1), all explicit");
    assert_eq!(rv[3], 0x00, "the population-default bessel byte is explicitly emitted");
}

/// KISS-OPS-6.19-0010 (`test_ops_opattrs_definite_length_prefix`): every ordered
/// vector carries a `u8` element-count prefix immediately before its elements, and
/// the whole blob has a definite length equal to the sum of its parts. MUTATION: an
/// encoder that drops the count prefix, or an indefinite/sentinel-terminated vector,
/// breaks the count==prefix identity and the summed-length identity.
#[test]
fn test_ops_opattrs_definite_length_prefix() {
    // the leading byte is the element COUNT, not the first element: use a vector
    // whose count (3) differs from its element value (7) so a dropped-prefix encoder
    // (which would surface 0x07) is distinguished.
    let v3 = window_param_vector(&[7, 7, 7]);
    assert_eq!(v3[0], 0x03, "the prefix is the element count, not the first element");
    assert_eq!(v3.len(), 1 + 3 * 4, "definite length = count byte + 3 * u32");
    let v2 = window_param_vector(&[2, 2]);
    assert_eq!(v2[0], 0x02, "count prefix precedes the elements");
    assert_eq!(v2.len(), 1 + 2 * 4, "definite length = 9");
    // the whole avg_pool blob's length is the sum of its four count-prefixed vectors
    // plus the one flag byte — a definite length, no indefinite terminator.
    let ap = encode_avg_pool(&[2, 2], &[2, 2], &[1, 1], &[0, 0], true);
    assert_eq!(ap.len(), 4 * (1 + 2 * 4) + 1, "definite length = 4 vectors + 1 flag = 37");
}

/// KISS-OPS-6.19-0012 (`test_ops_opattrs_opaque_embedding_byte_compare`): the
/// embedding layer distinguishes two ops by a bytewise blob compare alone. Two
/// producers resolving the SAME attributes emit byte-identical blobs, and two
/// differing in exactly one resolved field emit DIFFERENT blobs. The load-bearing
/// leg is the INEQUALITY. MUTATION: an encoder that drops/normalizes the oob field
/// makes the skip and clamp blobs equal -> a byte-comparing embedder can no longer
/// tell them apart -> the `!=` assertion fails.
#[test]
fn test_ops_opattrs_opaque_embedding_byte_compare() {
    // two independent encodings of the same resolved attributes are byte-identical.
    let a = encode_gather(0, OobPolicy::Skip, 1, IndexDtype::U32);
    let b = encode_gather(0, OobPolicy::Skip, 1, IndexDtype::U32);
    assert_eq!(a, b, "same resolved attributes -> byte-identical opaque blob");
    // changing exactly one resolved field (oob_policy) changes the blob — the teeth.
    let c = encode_gather(0, OobPolicy::Clamp, 1, IndexDtype::U32);
    assert_ne!(a, c, "a single differing resolved field must change the opaque blob");
}

/// KISS-OPS-6.19-0022 (`test_ops_opattrs_boolean_flags`): every boolean OpAttrs field
/// encodes as a `u8` 0/1, with both values meaningful and emitted explicitly even
/// where this op-set version pins the flag to a constant. Each toggled pair has equal
/// length and differs in exactly the one flag byte, which is 0 or 1. MUTATION: encode
/// a bool as a 0xFF all-ones mask -> the `== 0`/`== 1` byte assertion fails; omit a
/// pinned-constant flag -> the length / one-byte-difference assertion fails.
#[test]
fn test_ops_opattrs_boolean_flags() {
    // keepdim (reduce): a 0/1 byte; toggling it flips exactly byte 3.
    let kt = encode_reduce(Monoid::Sum, &[0, 1], 2, true);
    let kf = encode_reduce(Monoid::Sum, &[0, 1], 2, false);
    assert_eq!((kt.len(), kf.len()), (4, 4));
    assert_eq!((kt[3], kf[3]), (0x01, 0x00), "keepdim is a u8 0/1, not a mask");
    assert_eq!(
        (0..4).filter(|&i| kt[i] != kf[i]).collect::<Vec<_>>(),
        vec![3],
        "toggling keepdim changes exactly the keepdim byte"
    );
    // bessel_correction (reduce_var): a 0/1 byte; toggling it flips exactly byte 3.
    let bt = encode_reduce_var(&[0, 1], 2, true, true);
    let bf = encode_reduce_var(&[0, 1], 2, true, false);
    assert_eq!((bt[3], bf[3]), (0x01, 0x00), "bessel_correction is a u8 0/1");
    assert_eq!(
        (0..4).filter(|&i| bt[i] != bf[i]).collect::<Vec<_>>(),
        vec![3],
        "toggling bessel_correction changes exactly one byte"
    );
    // stability (sort_network): a 0/1 byte; toggling it flips exactly byte 2.
    let st = encode_sort_network(1, SortDirection::Ascending, true);
    let sf = encode_sort_network(1, SortDirection::Ascending, false);
    assert_eq!((st[2], sf[2]), (0x01, 0x00), "stability is a u8 0/1");
    assert_eq!(
        (0..st.len()).filter(|&i| st[i] != sf[i]).collect::<Vec<_>>(),
        vec![2],
        "toggling stability changes exactly one byte"
    );
}

/// KISS-OPS-6.19-0024 (`test_ops_opattrs_permutation_reserved`): the `permutation`
/// sub-vocabulary is frozen and reserved, and NO op of this version carries a free
/// permutation field — `sort_network` exports its permutation as a runtime
/// index-lane output (§6.11-0007), not an OpAttrs field. The one op that could
/// plausibly carry it encodes to exactly three bytes (axis | direction | stability)
/// with no appended permutation vector. MUTATION: an encoder that appends a
/// length-prefixed permutation vector to sort_network -> len > 3 -> fails.
#[test]
fn test_ops_opattrs_permutation_reserved() {
    let sn = encode_sort_network(1, SortDirection::Ascending, true);
    assert_golden("KISS-OPS-6.19-0024", "sort_no_permutation", &sn, "01 01 01");
    assert_eq!(sn.len(), 3, "sort_network carries no permutation field (would add >=1 byte)");
}

/// Build a Classify `structure_key` and return just its serialized reduce field.
/// Reads the Classify reduce-field codec purely BY FUNCTION (the public `to_token`
/// serializer); for a non-`gem` family the reduce field is the last `|`-segment.
/// No Classify clause id is cited, so this stays a single-clause OpAttrs binding.
fn classify_reduce_field(r: Reduce) -> String {
    let key = StructureKey {
        op_family: "red".to_string(),
        dtype: "f32".to_string(),
        target: "cpu:baseline".to_string(),
        index_width: "u32".to_string(),
        work_class: WorkClass::Warp,
        rank: 3,
        operands: Vec::new(),
        reduce: r,
        contraction: None,
        acc_mp: None,
    };
    key.to_token().rsplit('|').next().unwrap().to_string()
}

/// KISS-OPS-6.19-0035 (`test_ops_opattrs_reduce_axes_classify_reconciliation`): the
/// `reduce_axes` OpAttrs `u16` reconciles 1:1 with the Classify `structure_key`
/// reduce-field on the four shared categories. Both codecs are checked independently:
/// the Ops binary codec (opattrs) AND the Classify token codec (structure_key), read
/// by function only. MUTATION: swap the Ops rall/rlast sentinels (0xFFFF<->0xFFFE),
/// rewiden the subset mask, or change a Classify token -> one leg of a pinned pair
/// diverges -> fails.
#[test]
fn test_ops_opattrs_reduce_axes_classify_reconciliation() {
    // all-axes: Ops 0xFFFF  <->  Classify "rall"
    assert_eq!(reduce_axes_reduction(&[0, 1, 2], 3), 0xFFFF);
    assert_eq!(classify_reduce_field(Reduce::All), "rall");
    // trailing-axis: Ops 0xFFFE  <->  Classify "rlast"
    assert_eq!(reduce_axes_reduction(&[2], 3), 0xFFFE);
    assert_eq!(classify_reduce_field(Reduce::Trailing), "rlast");
    // subset {0,2}/rank3: Ops low-byte u8 mask 0x05  <->  Classify "x05"
    let subset = reduce_axes_reduction(&[0, 2], 3);
    assert_eq!(subset, 0x0005);
    assert_eq!(subset & 0x00FF, 0x05, "the subset mask lives in the low byte");
    assert_eq!(classify_reduce_field(Reduce::Subset(0x05)), "x05");
    // none/not-a-reduction: Ops rejects the 0x0000 sentinel  <->  Classify "-"
    assert!(validate_reduce_axes(0x0000).is_err(), "0x0000 is unreachable on the Ops channel");
    assert_eq!(classify_reduce_field(Reduce::None), "-");
}

/// KISS-OPS-6.19-0011 (`test_ops_opattrs_version_binding`): the pinned `MAX_RANK` and
/// `MAX_OPERANDS` constants (§6.19-0037) are SHARED anchors — defined once for the
/// OpAttrs channel (`opattrs`) and once for the Classify `structure_key` schema — and
/// §6.19-0011 requires them to be co-versioned, i.e. the two independently-authored
/// constants MUST stay equal. A non-additive one-sided change (raise the Classify rank
/// anchor to 16 without co-bumping the OpAttrs one, or vice versa) is a silent
/// desync the clause forbids. Both constants are read live from their own module, so
/// the equality is not a self-comparison: `opattrs::MAX_RANK` is `= 8` and
/// `structure_key::MAX_RANK` is a separate `= 8`, and a drift in either alone fails.
///
/// SCOPE (reported honestly): this binds ONLY the concretely-testable co-versioned-
/// CONSTANT sub-obligation of 0011. The broader "any non-additive layout change (field
/// reorder / re-widen / ordinal reuse / reduce_axes remultiplex) MUST be a new byte
/// form under a bumped schema version, never a silent in-place change" needs a
/// cross-version fixture and is not harness-testable here. The concrete VALUE `== 8`
/// is owned by §6.19-0037 (`test_ops_opattrs_max_rank_operands_pinned`), deliberately
/// NOT re-asserted here so a legitimate coordinated co-bump would not spuriously fail
/// this equality-of-anchors test.
///
/// MUTATION that fails: set `structure_key::MAX_RANK` to 16 (or `opattrs::MAX_RANK`
/// alone) -> the cross-module equality diverges -> the assertion fires.
#[test]
fn test_ops_opattrs_version_binding() {
    // §6.19-0011 co-versioning: the OpAttrs axis/subset-mask bound and the Classify
    // `structure_key` rank anchor are ONE shared constant, defined independently in two
    // modules; a conforming version keeps them equal. `MAX_RANK`/`MAX_OPERANDS` here
    // resolve to the OpAttrs-side constants via the `opattrs::*` glob import; the
    // Classify-side anchors are named by fully-qualified path so nothing is imported
    // twice.
    assert_eq!(
        MAX_RANK as u32,
        kiss_conformance::structure_key::MAX_RANK,
        "OpAttrs MAX_RANK must be co-versioned with the Classify structure_key rank anchor"
    );
    assert_eq!(
        MAX_OPERANDS as usize,
        kiss_conformance::structure_key::MAX_OPERANDS,
        "OpAttrs MAX_OPERANDS must be co-versioned with the Classify structure_key operand anchor"
    );
}
