// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference **emit-derivation stage** for the KISS-Classify `structure_key` (§6.6-0011/-0012):
//! a raw cell — operands `(use, dtype, extents, strides, alignment, quant)`, a target, and the
//! caller-declared hints — to a [`StructureKey`], by composing the per-fact `derive_*` functions of
//! [`crate::structure_key`]. Until this module the reference codec round-tripped tokens and derived
//! single facts; nothing derived a whole token from a cell (WIRE-FIRST §"No test in the repo derives a
//! structure_key"; §6.6-0019's "emit-derivation stage … deferred").
//!
//! **Scope, stated:** every cell class **except `ctr`**. A dense-contraction cell needs the batch role,
//! which no clause defines (KISS#543); a cell that declares contraction roles is out of scope here and
//! a [`DeriveDecline::ContractionOutOfScope`] marks the boundary rather than guessing a batch rule.

use crate::structure_key::*;

/// One operand of a cell as the caller presents it (call order; see [`derive_structure_key`]).
#[derive(Debug, Clone)]
pub struct CellOperand {
    pub usage: OperandUse,
    pub dtype: &'static str,
    pub extents: Vec<i64>,
    pub strides: Vec<i64>,
    pub alignment: u32,
    /// The operand's `quant.dequant_form` iff it carries a `quant` record (§6.3-0009a).
    pub dequant: Option<DequantForm>,
}

/// A cell and the caller-declared facts (§6.6-0012).
#[derive(Debug, Clone)]
pub struct DerivationCell {
    pub target: &'static str,
    /// Operands in the caller's CALL order; the canonical order (§6.6-0014) is derived.
    pub operands: Vec<CellOperand>,
    /// Slot (into `operands`, call order) the caller declares as the index operand, if any.
    pub index_operand: Option<usize>,
    /// Frame axes (0 = outermost) the caller declares reduced; empty = not a reduction.
    pub reduced_axes: Vec<usize>,
    /// Accumulator dtype if it differs from the compute dtype; `None` = the compute dtype.
    pub accumulator: Option<&'static str>,
    pub math_fidelity: MathFidelity,
    /// The caller declares M/N/K roles (a dense contraction) — out of scope, see the module doc.
    pub contraction_declared: bool,
}

/// Why a cell does not derive a key. Distinct causes, distinct spellings (published in the artifact).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveDecline {
    /// Operand 0 (canonical order) is a scale-type dtype (§6.6-0021).
    ScaleDtypeAtOperand0,
    /// A dtype in a dtype position is a reserved token (§6.1-0001).
    ReservedDtype,
    /// A dtype outside the closed §6.1 set.
    UnknownDtype,
    /// More operands than `MAX_OPERANDS` (§6.4-0002) or none at all.
    OperandCount,
    /// Rank beyond `MAX_RANK`, or `extents`/`strides` of different length (§6.4-0001, §6.3-0001).
    BadShape,
    /// A reduction output is not in keepdim form: its rank differs from the frame's or a reduced axis
    /// does not have extent 1 (§6.6-0009: a collapsed output is rejected, never keyed).
    CollapsedReduction,
    /// A declared reduced axis is not an axis of the frame (§6.6-0009).
    ReducedAxisOutOfRange,
    /// The cell declares contraction roles; the batch role is undefined (KISS#543).
    ContractionOutOfScope,
    /// The declared index slot does not name an operand.
    IndexSlotOutOfRange,
}

impl DeriveDecline {
    /// The published spelling (decoupled from `Debug`, like [`KeyDecline`]'s wire kinds).
    pub fn wire(&self) -> &'static str {
        match self {
            Self::ScaleDtypeAtOperand0 => "ScaleDtypeAtOperand0",
            Self::ReservedDtype => "ReservedDtype",
            Self::UnknownDtype => "UnknownDtype",
            Self::OperandCount => "OperandCount",
            Self::BadShape => "BadShape",
            Self::CollapsedReduction => "CollapsedReduction",
            Self::ReducedAxisOutOfRange => "ReducedAxisOutOfRange",
            Self::ContractionOutOfScope => "ContractionOutOfScope",
            Self::IndexSlotOutOfRange => "IndexSlotOutOfRange",
        }
    }
}

/// Storage bytes of a dtype for the §6.5-0009(c) byte cap: `None` for the sub-byte dtypes
/// (`i4`, `u4`, `b1`), which derive `v1`. Mirrors the §6.1 table's storage widths.
pub fn dtype_storage_bytes(dtype: &str) -> Option<Option<u32>> {
    Some(match dtype {
        "f16" | "bf16" | "i16" | "u16" => Some(2),
        "f32" | "i32" | "u32" => Some(4),
        "f64" | "i64" | "u64" | "c64" => Some(8),
        "c128" => Some(16),
        "i8" | "u8" | "bool" | "f8e4m3fn" | "f8e4m3fnuz" | "f8e5m2" | "f8e5m2fnuz" | "f8e8m0" | "f8e6m2" => Some(1),
        "i4" | "u4" | "b1" => None,
        _ => return Option::None,
    })
}

fn check_dtype(d: &str) -> Result<(), DeriveDecline> {
    if RESERVED_DTYPES.contains(&d) {
        return Err(DeriveDecline::ReservedDtype);
    }
    if !DTYPES.contains(&d) {
        return Err(DeriveDecline::UnknownDtype);
    }
    Ok(())
}

/// The canonical reduce spelling of a declared reduced-axis set over frame rank `rank` (§6.6-0009).
fn canonical_reduce(axes: &[usize], rank: usize) -> Result<Reduce, DeriveDecline> {
    if axes.is_empty() {
        return Ok(Reduce::None);
    }
    let mut mask: u32 = 0;
    for &a in axes {
        if a >= rank {
            return Err(DeriveDecline::ReducedAxisOutOfRange);
        }
        mask |= 1 << a;
    }
    let full = (1u32 << rank) - 1;
    Ok(if mask == full {
        Reduce::All // rall wins the rank-1 tie
    } else if mask == 1 << (rank - 1) {
        Reduce::Trailing
    } else {
        Reduce::Subset(mask as u8)
    })
}

/// Validate the cell's shape and dtypes before anything is derived from it. Only an UNKNOWN dtype is
/// refused here (its storage width is needed for the per-operand vector width); a RESERVED dtype is
/// refused only where it occupies a key position — the primary dtype and the accumulator (§6.1-0001).
fn validate(cell: &DerivationCell) -> Result<(), DeriveDecline> {
    if cell.contraction_declared {
        return Err(DeriveDecline::ContractionOutOfScope);
    }
    if cell.operands.is_empty() || cell.operands.len() > MAX_OPERANDS {
        return Err(DeriveDecline::OperandCount);
    }
    if cell.index_operand.is_some_and(|s| s >= cell.operands.len()) {
        return Err(DeriveDecline::IndexSlotOutOfRange);
    }
    for o in &cell.operands {
        if o.extents.len() != o.strides.len() || o.extents.len() > MAX_RANK as usize {
            return Err(DeriveDecline::BadShape);
        }
        if !DTYPES.contains(&o.dtype) {
            return Err(DeriveDecline::UnknownDtype);
        }
    }
    if let Some(a) = cell.accumulator {
        check_dtype(a)?;
    }
    Ok(())
}

/// Canonical operand order (§6.6-0014): inputs and in-place operands in call order, then outputs.
fn canonical_order(cell: &DerivationCell) -> Vec<&CellOperand> {
    let mut ordered: Vec<&CellOperand> = cell
        .operands
        .iter()
        .filter(|o| matches!(o.usage, OperandUse::Input | OperandUse::InPlace))
        .collect();
    ordered.extend(cell.operands.iter().filter(|o| o.usage == OperandUse::Output));
    ordered
}

/// Per-axis iteration-frame extents: the maximum across operands, right-aligned (§6.5-0010).
fn frame_extents(ordered: &[&CellOperand], frame_rank: usize) -> Vec<i64> {
    (0..frame_rank)
        .map(|j| {
            ordered
                .iter()
                .map(|o| {
                    let off = frame_rank - o.extents.len();
                    if j < off { 1 } else { o.extents[j - off] }
                })
                .max()
                .unwrap_or(1)
        })
        .collect()
}

/// The reduce spec: carried iff the class is `rdc` (§6.6-0017), and the output must be keepdim.
fn reduce_spec(
    cell: &DerivationCell,
    ordered: &[&CellOperand],
    class: &str,
    frame_rank: usize,
) -> Result<Reduce, DeriveDecline> {
    if class != "rdc" {
        return Ok(Reduce::None);
    }
    let r = canonical_reduce(&cell.reduced_axes, frame_rank)?;
    for o in ordered.iter().filter(|o| o.usage == OperandUse::Output) {
        let collapsed =
            o.extents.len() != frame_rank || cell.reduced_axes.iter().any(|&a| o.extents[a] != 1);
        if collapsed {
            return Err(DeriveDecline::CollapsedReduction);
        }
    }
    Ok(r)
}

/// One operand's sub-key (§6.6-0007): layout/vec/div from its OWN axes, the mask in the frame.
fn sub_key(o: &CellOperand, frame: &[i64], reduced_innermost: bool) -> OperandSubKey {
    let layout = derive_layout_tag(&o.extents, &o.strides);
    let any_own_broadcast = o.extents.iter().zip(&o.strides).any(|(&e, &s)| e > 1 && s == 0);
    let rank = o.extents.len();
    let bytes = dtype_storage_bytes(o.dtype).expect("closed dtype set checked by validate");
    let vec = if layout == Contig::Broadcast || reduced_innermost || rank == 0 {
        VecWidth::V1 // §6.5-0009 (a), (b); a rank-0 operand has no run to vectorize
    } else {
        derive_vec_width(o.strides[rank - 1], o.extents[rank - 1], bytes, o.alignment, any_own_broadcast)
    };
    OperandSubKey {
        contig: layout,
        bcast_mask: derive_bcast_mask_in_frame(&o.extents, &o.strides, frame),
        vec,
        div: derive_div_bucket_of(&o.extents),
        flipped: o.strides.iter().any(|&s| s < 0),
        dequant: o.dequant,
    }
}

/// The non-contraction `(acc + mp)` field (§6.7-0013): emitted iff the accumulator differs from the
/// compute dtype or `<mp>` is not the default; both slots are spelled when it is.
fn precision_field(cell: &DerivationCell, primary: &str) -> Option<AccMp> {
    let acc_deviates = cell.accumulator.is_some_and(|a| a != primary);
    if acc_deviates || cell.math_fidelity != ACC_MP_DEFAULT_MP {
        Some(AccMp { acc: cell.accumulator.unwrap_or(primary).to_string(), mp: cell.math_fidelity })
    } else {
        None
    }
}

/// Derive the `structure_key` of `cell` (§6.6-0011): the same inputs always give the same bytes.
pub fn derive_structure_key(cell: &DerivationCell) -> Result<StructureKey, DeriveDecline> {
    validate(cell)?;
    let ordered = canonical_order(cell);

    // primary dtype (§6.6-0005/-0021)
    let dtypes: Vec<&str> = ordered.iter().map(|o| o.dtype).collect();
    let primary = derive_primary_dtype(&dtypes).ok_or(DeriveDecline::ScaleDtypeAtOperand0)?;
    check_dtype(primary)?; // a reserved dtype in the key's dtype field (§6.1-0001)

    let frame_rank = ordered.iter().map(|o| o.extents.len()).max().unwrap_or(0);
    let frame = frame_extents(&ordered, frame_rank);

    // cell class (§6.5-0015) from the operands (use + own extents) and the declared facts.
    let class_ops: Vec<(OperandUse, &[i64])> = ordered.iter().map(|o| (o.usage, o.extents.as_slice())).collect();
    let facts = DeclaredFacts {
        contraction_roles: false,
        index_role: cell.index_operand.is_some(),
        reduced_axes_nonempty: !cell.reduced_axes.is_empty(),
    };
    let class = derive_cell_class(&class_ops, facts);

    let reduce = reduce_spec(cell, &ordered, class, frame_rank)?;
    let reduced_innermost = match reduce {
        Reduce::None => false,
        Reduce::All | Reduce::Trailing => true,
        Reduce::Subset(m) => frame_rank > 0 && m & (1 << (frame_rank - 1)) != 0,
    };
    let operands: Vec<OperandSubKey> = ordered.iter().map(|o| sub_key(o, &frame, reduced_innermost)).collect();

    let ext_refs: Vec<&[i64]> = ordered.iter().map(|o| o.extents.as_slice()).collect();
    let pairs: Vec<(&[i64], &[i64])> =
        ordered.iter().map(|o| (o.extents.as_slice(), o.strides.as_slice())).collect();

    Ok(StructureKey {
        cell_class: class.to_string(),
        dtype: primary.to_string(),
        target: cell.target.to_string(),
        index_width: derive_index_width(&pairs).to_string(),
        work_class: derive_work_class(&ext_refs),
        rank: frame_rank as u32,
        operands,
        reduce,
        contraction: None,
        acc_mp: precision_field(cell, primary),
    })
}
