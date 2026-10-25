// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference **emit-derivation stage** for the KISS-Classify `structure_key` (§6.6-0011/-0012):
//! a raw cell — operands `(use, dtype, extents, strides, alignment, quant)`, a target, and the
//! caller-declared hints — to a [`StructureKey`], by composing the per-fact `derive_*` functions of
//! [`crate::structure_key`]. Until this module the reference codec round-tripped tokens and derived
//! single facts; nothing derived a whole token from a cell (WIRE-FIRST §"No test in the repo derives a
//! structure_key"; §6.6-0019's "emit-derivation stage … deferred").
//!
//! **Scope:** every cell class, `ctr` included (R3.3). A dense-contraction cell is one whose caller declares
//! [`ContractionRoles`] (§6.6-0016/-0019/-0022/-0023): the M/N/K and batch axes and the weight and scale
//! operand slots are role hints, never inferred from extents, position or dtype.

use crate::structure_key::*;
use std::num::NonZeroU32;

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
    /// The caller's dense-contraction role hints (§6.6-0012); `Some` makes the cell class `ctr`.
    pub contraction: Option<ContractionRoles>,
    /// The vector-access byte cap the cell's target declares (§6.5-0009(c), §6.8-0018); `None` = the
    /// target declares nothing, treated as [`DEFAULT_VEC_ACCESS_CAP_BYTES`].
    pub vec_cap_bytes: Option<NonZeroU32>,
}

/// One axis of one operand, as a role hint names it: `operand` indexes `DerivationCell::operands` in CALL
/// order, `axis` counts from 0 = outermost of THAT operand's own extents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisRef {
    pub operand: usize,
    pub axis: usize,
}

/// The caller-declared roles of a dense-contraction cell. M, N and K are operand axes (`lhs [M,K]`,
/// `rhs [K,N]` share frame axes under right-alignment, so a frame axis cannot name them, §6.6-0016); the
/// batch axes are iteration-frame axes (0 = outermost, §6.6-0013, §6.6-0023); the weight and scale are
/// call-order operand slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractionRoles {
    pub m: AxisRef,
    pub n: AxisRef,
    pub k: AxisRef,
    /// Batch axes (§6.6-0023): possibly empty; batched iff non-empty.
    pub batch: Vec<usize>,
    /// The weight operand (§6.6-0019): the dtype source of `<wdt>`.
    pub weight: usize,
    /// The scale operand of a quantized weight (§6.6-0022); `None` = no `q` coordinate.
    pub scale: Option<usize>,
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
    /// M, N or K does not name an existing operand axis, or two of them name the same one (§6.6-0016).
    ContractionAxisInvalid,
    /// The weight or scale role names a slot that is not an operand (§6.6-0019, §6.6-0022).
    RoleSlotOutOfRange,
    /// A `ctr` cell has no output operand to supply `<out>` (§6.7-0006).
    ContractionNoOutput,
    /// A malformed batch-axis hint (§6.6-0023), with its own typed spelling.
    Batch(BatchRoleDecline),
    /// The declared index slot does not name an operand.
    IndexSlotOutOfRange,
    /// The accumulator coordinate the key would spell is an MX element dtype (§6.1-0016).
    ElementDtypeAsAccumulator,
}

impl DeriveDecline {
    /// The published spelling (decoupled from `Debug`, like [`KeyDecline`]'s wire kinds).
    pub fn wire(&self) -> &'static str {
        match self {
            Self::ScaleDtypeAtOperand0 => "ScaleDtypeAtOperand0",
            Self::ReservedDtype => "ReservedDtype",
            Self::ElementDtypeAsAccumulator => "ElementDtypeAsAccumulator",
            Self::UnknownDtype => "UnknownDtype",
            Self::OperandCount => "OperandCount",
            Self::BadShape => "BadShape",
            Self::CollapsedReduction => "CollapsedReduction",
            Self::ReducedAxisOutOfRange => "ReducedAxisOutOfRange",
            Self::ContractionAxisInvalid => "ContractionAxisInvalid",
            Self::RoleSlotOutOfRange => "RoleSlotOutOfRange",
            Self::ContractionNoOutput => "ContractionNoOutput",
            Self::Batch(b) => b.wire(),
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
        "i4" | "u4" | "b1" | "f4e2m1" | "f6e2m3" | "f6e3m2" => None,
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
fn sub_key(o: &CellOperand, frame: &[i64], reduced_innermost: bool, vec_cap: u32) -> OperandSubKey {
    let layout = derive_layout_tag(&o.extents, &o.strides);
    let any_own_broadcast = o.extents.iter().zip(&o.strides).any(|(&e, &s)| e > 1 && s == 0);
    let rank = o.extents.len();
    let bytes = dtype_storage_bytes(o.dtype).expect("closed dtype set checked by validate");
    let vec = if layout == Contig::Broadcast || reduced_innermost || rank == 0 {
        VecWidth::V1 // §6.5-0009 (a), (b); a rank-0 operand has no run to vectorize
    } else {
        derive_vec_width_capped(o.strides[rank - 1], o.extents[rank - 1], bytes, o.alignment, any_own_broadcast, vec_cap)
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
fn precision_field(cell: &DerivationCell, primary: &str) -> Result<Option<AccMp>, DeriveDecline> {
    let acc_deviates = cell.accumulator.is_some_and(|a| a != primary);
    if acc_deviates || cell.math_fidelity != ACC_MP_DEFAULT_MP {
        let acc = cell.accumulator.unwrap_or(primary);
        // §6.1-0016: an MX element is never an accumulator coordinate that gets spelled
        if MX_ELEMENT_DTYPES.contains(&acc) {
            return Err(DeriveDecline::ElementDtypeAsAccumulator);
        }
        Ok(Some(AccMp { acc: acc.to_string(), mp: cell.math_fidelity }))
    } else {
        Ok(None)
    }
}

/// The `contraction` field of a `ctr` cell (§6.7-0006) from the declared roles: M/N/K size classes from
/// the caller-assigned operand axes (§6.6-0016), K-divisibility from the K extent, the
/// batch class (§6.6-0023), and `<wdt>`/`<sdt>` from the hinted operands' own dtypes (§6.6-0019/-0022).
fn contraction_field(
    cell: &DerivationCell,
    roles: &ContractionRoles,
    ordered: &[&CellOperand],
    frame: &[i64],
    primary: &str,
) -> Result<Contraction, DeriveDecline> {
    let rank = frame.len();
    let (m, n, k) = (roles.m, roles.n, roles.k);
    if m == n || m == k || n == k {
        return Err(DeriveDecline::ContractionAxisInvalid);
    }
    // the extent of a role axis and the iteration-frame axis it lands on (right-aligned, §6.5-0010)
    let role = |r: AxisRef| -> Result<(i64, usize), DeriveDecline> {
        let o = cell.operands.get(r.operand).ok_or(DeriveDecline::ContractionAxisInvalid)?;
        let extent = *o.extents.get(r.axis).ok_or(DeriveDecline::ContractionAxisInvalid)?;
        Ok((extent, rank - o.extents.len() + r.axis))
    };
    let ((me, ma), (ne, na), (ke, ka)) = (role(m)?, role(n)?, role(k)?);
    let slot = |s: usize| cell.operands.get(s).ok_or(DeriveDecline::RoleSlotOutOfRange);
    let weight = slot(roles.weight)?;
    let scale = roles.scale.map(slot).transpose()?;
    let out = ordered
        .iter()
        .find(|o| o.usage == OperandUse::Output)
        .ok_or(DeriveDecline::ContractionNoOutput)?;
    // frame axes the output covers (right-aligned, §6.5-0010)
    let out_axes: Vec<usize> = (rank - out.extents.len()..rank).collect();
    let batch = derive_batch_class(frame, &roles.batch, &[ma, na, ka], &out_axes).map_err(DeriveDecline::Batch)?;
    for d in [weight.dtype, out.dtype].into_iter().chain(scale.map(|o| o.dtype)) {
        check_dtype(d)?; // wdt, out and sdt are key positions (§6.1-0001)
    }
    // §6.1-0016: `<acc>` is always spelled in a contraction field and is never an MX element, so an
    // MX-element operand 0 with an undeclared accumulator (which defaults to it) declines.
    if MX_ELEMENT_DTYPES.contains(&cell.accumulator.unwrap_or(primary)) {
        return Err(DeriveDecline::ElementDtypeAsAccumulator);
    }
    Ok(Contraction {
        m: size_class_of(me),
        n: size_class_of(ne),
        k: size_class_of(ke),
        k_div: derive_div_bucket(ke),
        batch,
        wdt: weight.dtype.to_string(),
        acc: cell.accumulator.unwrap_or(primary).to_string(),
        out: out.dtype.to_string(),
        mp: cell.math_fidelity,
        sdt: scale.map(|o| o.dtype.to_string()),
    })
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
        contraction_roles: cell.contraction.is_some(),
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
    let operands: Vec<OperandSubKey> = ordered
        .iter()
        .map(|o| sub_key(o, &frame, reduced_innermost, cell.vec_cap_bytes.map_or(DEFAULT_VEC_ACCESS_CAP_BYTES, NonZeroU32::get)))
        .collect();

    let ext_refs: Vec<&[i64]> = ordered.iter().map(|o| o.extents.as_slice()).collect();
    let pairs: Vec<(&[i64], &[i64])> =
        ordered.iter().map(|o| (o.extents.as_slice(), o.strides.as_slice())).collect();

    let contraction = cell
        .contraction
        .as_ref()
        .map(|roles| contraction_field(cell, roles, &ordered, &frame, primary))
        .transpose()?;

    Ok(StructureKey {
        cell_class: class.to_string(),
        dtype: primary.to_string(),
        target: cell.target.to_string(),
        index_width: derive_index_width(&pairs).to_string(),
        work_class: derive_work_class(&ext_refs),
        rank: frame_rank as u32,
        operands,
        reduce,
        contraction,
        acc_mp: if cell.contraction.is_some() { None } else { precision_field(cell, primary)? },
    })
}
