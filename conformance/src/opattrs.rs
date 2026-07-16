//! Reference encoder for the KISS-Ops OpAttrs canonical wire encoding (Ops §6.19).
//!
//! Design rules, verbatim from §6.19 (each field of every per-op schema is
//! emitted **explicitly at its resolved value** — no elision, R1):
//!   * enums are frozen little-endian `u8` ordinals, `0` reserved (§6.19.2);
//!   * `reduce_axes` is a `u16` LE multiplex with a rank-aware total precedence
//!     (§6.19-0020) — the operand rank is NOT in the blob, so the category is
//!     derived from the reduced-axis set and the rank;
//!   * window-parameter vectors are a `u8` count then that many `u32` LE
//!     elements in ascending spatial-axis order (§6.19-0023);
//!   * fields appear in the canonical per-op order (§6.19.3), names never on the
//!     wire — the embedding layer's `op_name` selects the schema.

use std::collections::BTreeSet;

// ---- §6.19.2 sub-vocabularies: frozen u8 ordinals, 0 reserved ----------------

macro_rules! ordinal_enum {
    ($(#[$m:meta])* $name:ident { $($variant:ident = $ord:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            /// The frozen wire ordinal (§6.19.2).
            pub fn ordinal(self) -> u8 { match self { $($name::$variant => $ord),+ } }
        }
    };
}

ordinal_enum!(/// §6.19-0014
    Monoid { Sum = 1, Prod = 2, Max = 3, Min = 4 });
ordinal_enum!(/// §6.19-0015
    OobPolicy { Skip = 1, Clamp = 2, ZeroFill = 3 });
ordinal_enum!(/// §6.19-0016
    ScatterCombine { Assign = 1, AtomicAdd = 2, AtomicMax = 3, AtomicMin = 4 });
ordinal_enum!(/// §6.19-0017
    IndexDtype { U32 = 1, I32 = 2, I64 = 3 });
ordinal_enum!(/// §6.19-0018
    SortDirection { Ascending = 1, Descending = 2 });
ordinal_enum!(/// §6.19-0019
    ScanExclusivity { Inclusive = 1, Exclusive = 2 });
ordinal_enum!(/// §6.19-0021 (schema field `output_column_ordering`)
    ColumnOrdering { ChannelMajor = 1, TapMajor = 2 });

// ---- §6.19-0020 reduce_axes multiplex ---------------------------------------

/// `reduce_axes` u16 for a reduction (`reduce`/`reduce_var`/`reduce_std`) over
/// `reduced` axes of an operand of rank `rank`, per the §6.19-0020 total
/// precedence: all-axes (0xFFFF) beats trailing-only (0xFFFE) beats subset mask.
/// A reduction never emits 0x0000 (§6.19-0038).
pub fn reduce_axes_reduction(reduced: &[usize], rank: usize) -> u16 {
    assert!((1..=8).contains(&rank), "rank out of range 1..=MAX_RANK(8)");
    let set: BTreeSet<usize> = reduced.iter().copied().collect();
    assert!(!set.is_empty() && set.iter().all(|&k| k < rank), "reduced axes out of range");
    let all: BTreeSet<usize> = (0..rank).collect();
    if set == all {
        0xFFFF
    } else if rank > 1 && set.len() == 1 && set.contains(&(rank - 1)) {
        0xFFFE
    } else {
        set.iter().fold(0u16, |m, &k| m | (1u16 << k)) // low-byte subset mask
    }
}

/// `reduce_axes` u16 for a `prefix_scan` folding over the single axis `axis`
/// (§6.19-0020 scan precedence): trailing axis of a rank>1 operand -> 0xFFFE,
/// otherwise the single-bit subset mask. Never 0xFFFF, never 0x0000.
pub fn reduce_axes_scan(axis: usize, rank: usize) -> u16 {
    assert!((1..=8).contains(&rank) && axis < rank, "scan axis/rank out of range");
    if rank > 1 && axis == rank - 1 {
        0xFFFE
    } else {
        1u16 << axis
    }
}

fn push_u16_le(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// A window-parameter vector (§6.19-0023): `u8` count then count × `u32` LE, in
/// the given (ascending spatial-axis) order — never sorted.
pub fn window_param_vector(elems: &[u32]) -> Vec<u8> {
    assert!(elems.len() <= 8, "window vector longer than MAX_RANK(8)");
    let mut out = vec![elems.len() as u8];
    for &e in elems {
        out.extend_from_slice(&e.to_le_bytes());
    }
    out
}

// ---- §6.19.3 per-op schemas (canonical field order = ABI) -------------------
//
// Every field is emitted explicitly at its resolved value. Callers pass resolved
// values; a "defaulted" and an "explicit-equal" field therefore produce the same
// bytes (§6.19-0005, Appendix E.3).

/// `reduce`: monoid, reduce_axes, keepdim (§6.19-0025).
pub fn encode_reduce(monoid: Monoid, reduced: &[usize], rank: usize, keepdim: bool) -> Vec<u8> {
    let mut out = vec![monoid.ordinal()];
    push_u16_le(&mut out, reduce_axes_reduction(reduced, rank));
    out.push(keepdim as u8);
    out
}

/// `prefix_scan`: monoid, reduce_axes, exclusivity (§6.19-0026).
pub fn encode_prefix_scan(monoid: Monoid, axis: usize, rank: usize, excl: ScanExclusivity) -> Vec<u8> {
    let mut out = vec![monoid.ordinal()];
    push_u16_le(&mut out, reduce_axes_scan(axis, rank));
    out.push(excl.ordinal());
    out
}

/// `gather`: axis, oob_policy, index_operand, index_dtype (§6.19-0027).
pub fn encode_gather(axis: u8, oob: OobPolicy, index_operand: u8, index_dtype: IndexDtype) -> Vec<u8> {
    vec![axis, oob.ordinal(), index_operand, index_dtype.ordinal()]
}

/// `scatter`: axis, combine, oob_policy(=skip fixed), index_operand, index_dtype (§6.19-0028).
pub fn encode_scatter(
    axis: u8,
    combine: ScatterCombine,
    oob: OobPolicy,
    index_operand: u8,
    index_dtype: IndexDtype,
) -> Vec<u8> {
    vec![axis, combine.ordinal(), oob.ordinal(), index_operand, index_dtype.ordinal()]
}

/// `sort_network`: axis(=trailing r-1), direction, stability (§6.19-0029).
pub fn encode_sort_network(axis: u8, direction: SortDirection, stability: bool) -> Vec<u8> {
    vec![axis, direction.ordinal(), stability as u8]
}

/// `reduce_var` / `reduce_std`: reduce_axes, keepdim, bessel_correction (§6.19-0030).
pub fn encode_reduce_var(reduced: &[usize], rank: usize, keepdim: bool, bessel: bool) -> Vec<u8> {
    let mut out = Vec::new();
    push_u16_le(&mut out, reduce_axes_reduction(reduced, rank));
    out.push(keepdim as u8);
    out.push(bessel as u8);
    out
}

/// `softmax` / `log_softmax`: norm_axis (§6.19-0031).
pub fn encode_softmax(norm_axis: u8) -> Vec<u8> {
    vec![norm_axis]
}

/// `index_select`: axis, index_operand, index_dtype (§6.19-0034).
pub fn encode_index_select(axis: u8, index_operand: u8, index_dtype: IndexDtype) -> Vec<u8> {
    vec![axis, index_operand, index_dtype.ordinal()]
}

/// `avg_pool`: window_size, stride, dilation, padding, count_include_pad (§6.19-0032).
pub fn encode_avg_pool(
    window: &[u32],
    stride: &[u32],
    dilation: &[u32],
    padding: &[u32],
    count_include_pad: bool,
) -> Vec<u8> {
    let mut out = Vec::new();
    for v in [window, stride, dilation, padding] {
        out.extend(window_param_vector(v));
    }
    out.push(count_include_pad as u8);
    out
}

/// `im2col`: window_size, stride, dilation, padding, output_column_ordering (§6.19-0033).
pub fn encode_im2col(
    window: &[u32],
    stride: &[u32],
    dilation: &[u32],
    padding: &[u32],
    column_ordering: ColumnOrdering,
) -> Vec<u8> {
    let mut out = Vec::new();
    for v in [window, stride, dilation, padding] {
        out.extend(window_param_vector(v));
    }
    out.push(column_ordering.ordinal());
    out
}

// ---- Reader side: a typed decline, never a panic (Conform §6.7) --------------

/// A typed decline raised when a blob fails validation. A reader MUST refuse a
/// malformed OpAttrs blob with a decline of this shape, never a crash/panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decline {
    /// An enum field carried the reserved `0` ordinal (§6.19.2).
    ReservedZeroOrdinal { field: &'static str },
    /// `reduce_axes` fell in the reserved band 0x0100..=0xFFFD (§6.19-0020).
    ReservedReduceAxes { value: u16 },
    /// The blob was shorter than the op's schema requires.
    TruncatedBlob { need: usize, got: usize },
}

/// Validate a decoded `gather` blob (`[axis, oob, index_operand, index_dtype]`),
/// returning a typed decline for a malformed one. Demonstrates modality 4.
pub fn decode_gather(blob: &[u8]) -> Result<(u8, OobPolicy, u8, IndexDtype), Decline> {
    if blob.len() != 4 {
        return Err(Decline::TruncatedBlob { need: 4, got: blob.len() });
    }
    let oob = match blob[1] {
        0 => return Err(Decline::ReservedZeroOrdinal { field: "oob_policy" }),
        1 => OobPolicy::Skip,
        2 => OobPolicy::Clamp,
        3 => OobPolicy::ZeroFill,
        _ => return Err(Decline::ReservedZeroOrdinal { field: "oob_policy" }),
    };
    let idt = match blob[3] {
        0 => return Err(Decline::ReservedZeroOrdinal { field: "index_dtype" }),
        1 => IndexDtype::U32,
        2 => IndexDtype::I32,
        3 => IndexDtype::I64,
        _ => return Err(Decline::ReservedZeroOrdinal { field: "index_dtype" }),
    };
    Ok((blob[0], oob, blob[2], idt))
}

/// Validate a `reduce_axes` u16 on a carrier OpAttrs blob (§6.19-0020, §6.19-0038).
///
/// The reserved band `0x0100..=0xFFFD` is malformed (§6.19-0020). So is `0x0000`:
/// it is the none/not-a-reduction sentinel whose sole role is the Classify
/// `structure_key` `-` token reconciliation (§6.19-0035), and every carrier op
/// that owns this field is a reduction or a scan, so `0x0000` is unreachable on
/// the OpAttrs channel and a reader MUST reject it as malformed (§6.19-0038).
pub fn validate_reduce_axes(value: u16) -> Result<(), Decline> {
    match value {
        0x0000 => Err(Decline::ReservedReduceAxes { value }),
        0x0100..=0xFFFD => Err(Decline::ReservedReduceAxes { value }),
        _ => Ok(()),
    }
}
