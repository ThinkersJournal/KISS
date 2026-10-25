// SPDX-License-Identifier: MIT OR Apache-2.0
//! **Cell-carrying `structure_key` derivation vectors** — the second surface §6.6-0019 says is missing:
//! each vector is a RAW cell (operands `(use, dtype, extents, strides, alignment, quant)`, target,
//! declared hints) and the token a conforming deriver MUST produce, or the typed decline it MUST return.
//! `structure_key_vectors.json` round-trips tokens only; a deriver that mis-reads the spec (which axis
//! is innermost, whose dtype is primary, what "pointwise" means) is invisible to it. These vectors make
//! the derivation itself byte-comparable across parties — the instrument freeze condition 1 (umbrella
//! §5.3: each field *independently derived*) asks for.
//!
//! **Scope:** every cell class, `ctr` included (R3.3): a dense-contraction cell carries its caller-declared
//! role hints (M/N/K axes, batch axes, weight and scale slots; KISS-CLASSIFY-6.6-0016/-0019/-0022/-0023).
//!
//! **Provenance of `expected`:** emitted by [`crate::derive::derive_structure_key`] at generation time.
//! Where the vector reproduces a KISS-CLASSIFY Appendix A.1 golden the spec token is carried as
//! `spec_golden` and asserted equal — those are the only vectors whose expected bytes are independent of
//! this reference (hand-derived in the spec text); the rest are reference output, to be cross-checked by
//! a second deriver, not trusted.

use crate::derive::*;
use crate::structure_key::*;

pub const SCHEMA: &str = "kiss-structure-key-derivation-vectors-v1";

pub enum Expect {
    Token(String),
    Decline(DeriveDecline),
}

pub struct DerivationVector {
    pub name: &'static str,
    pub clause: &'static str,
    pub note: &'static str,
    pub cell: DerivationCell,
    pub spec_golden: Option<&'static str>,
}

fn op(usage: OperandUse, dtype: &'static str, extents: &[i64], strides: &[i64], alignment: u32) -> CellOperand {
    CellOperand { usage, dtype, extents: extents.to_vec(), strides: strides.to_vec(), alignment, dequant: None }
}

fn cell(target: &'static str, operands: Vec<CellOperand>) -> DerivationCell {
    DerivationCell {
        target,
        operands,
        index_operand: None,
        reduced_axes: vec![],
        accumulator: None,
        math_fidelity: MathFidelity::Stable,
        contraction: None,
        vec_cap_bytes: None,
    }
}

const CUDA: &str = "cuda:sm89";
use OperandUse::{InPlace, Input, Output};

fn add_cell(in1_strides: &[i64], in1_extents: &[i64]) -> DerivationCell {
    cell(
        CUDA,
        vec![
            op(Input, "f32", &[128, 256], &[256, 1], 256),
            op(Input, "f32", in1_extents, in1_strides, 256),
            op(Output, "f32", &[128, 256], &[256, 1], 256),
        ],
    )
}

fn reduce(dtype: &'static str, in_e: &[i64], in_s: &[i64], out_e: &[i64], out_s: &[i64], axes: &[usize]) -> DerivationCell {
    let mut c = cell(CUDA, vec![op(Input, dtype, in_e, in_s, 256), op(Output, dtype, out_e, out_s, 256)]);
    c.reduced_axes = axes.to_vec();
    c
}

fn v(name: &'static str, clause: &'static str, note: &'static str, cell: DerivationCell) -> DerivationVector {
    DerivationVector { name, clause, note, cell, spec_golden: None }
}

fn golden(mut d: DerivationVector, token: &'static str) -> DerivationVector {
    d.spec_golden = Some(token);
    d
}

fn a1_elementwise(out: &mut Vec<DerivationVector>) {
    // ---- Appendix A.1 goldens, derived from raw cells (spec-independent expected bytes) ----
    out.push(golden(
        v("a1_binary_add", "KISS-CLASSIFY-6.6-0011", "the canonical derivation golden: two inputs + output, contiguous f32 [128,256]", add_cell(&[256, 1], &[128, 256])),
        "sk6|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-",
    ));
    out.push(golden(
        v("a1_binary_add_operand1_broadcast_axis0", "KISS-CLASSIFY-6.6-0008", "operand 1 stride [0,1]: layout br, mask 01, v1", add_cell(&[0, 1], &[128, 256])),
        "sk6|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;br/01/v1/d16/f;co/00/v4/d16/f|-",
    ));
    out.push(golden(
        v(
            "a1_unary_f16",
            "KISS-CLASSIFY-6.5-0009",
            "f16 [64,128]: v8 (8*2 = 16 bytes), d16, ix32, grid",
            cell(CUDA, vec![op(Input, "f16", &[64, 128], &[128, 1], 256), op(Output, "f16", &[64, 128], &[128, 1], 256)]),
        ),
        "sk6|ew1|f16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f;co/00/v8/d16/f|-",
    ));
}

fn a1_reductions(out: &mut Vec<DerivationVector>) {
    out.push(golden(
        v("a1_reduction_trailing_axis", "KISS-CLASSIFY-6.6-0009", "keepdim [4,8]->[4,1]: rlast; output mask 02 (extent-1 axis, frame extent 8)", reduce("f32", &[4, 8], &[8, 1], &[4, 1], &[1, 1], &[1])),
        "sk6|rdc|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/02/v1/da/f|rlast",
    ));
    out.push(golden(
        v("a1_reduction_all_axes", "KISS-CLASSIFY-6.6-0009", "keepdim [4,8]->[1,1]: rall; output mask 03", reduce("f32", &[4, 8], &[8, 1], &[1, 1], &[1, 1], &[0, 1])),
        "sk6|rdc|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/03/v1/da/f|rall",
    ));
    out.push(golden(
        v("a1_reduction_rank1", "KISS-CLASSIFY-6.6-0009", "rank-1 [8]->[1]: all-axes AND trailing; rall wins the tie", reduce("f32", &[8], &[1], &[1], &[1], &[0])),
        "sk6|rdc|f32|cuda:sm89|ix32|warp|r1|co/00/v1/d8/f;co/01/v1/da/f|rall",
    ));
    out.push(golden(
        v(
            "a1_reduction_subset_rank4",
            "KISS-CLASSIFY-6.6-0009",
            "[2,4,3,5]->[2,1,3,1] reducing axes 1 and 3: bitmask x0a; output mask 0a",
            reduce("f32", &[2, 4, 3, 5], &[60, 15, 5, 1], &[2, 1, 3, 1], &[3, 3, 1, 1], &[1, 3]),
        ),
        "sk6|rdc|f32|cuda:sm89|ix32|block|r4|co/00/v1/da/f;co/0a/v1/da/f|x0a",
    ));
    {
        let mut c = reduce("f16", &[4, 8], &[8, 1], &[4, 1], &[1, 1], &[1]);
        c.accumulator = Some("f32");
        out.push(golden(
            v("a1_reduction_f16_accumulates_f32", "KISS-CLASSIFY-6.7-0013", "accumulator f32 != compute f16: |f32/st carried", c),
            "sk6|rdc|f16|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/02/v1/da/f|rlast|f32/st",
        ));
    }
    out.push(golden(
        v(
            "a1_inplace_accumulate",
            "KISS-CLASSIFY-6.6-0014",
            "an in-place operand appears once, as an input; n_operands = 2",
            cell(CUDA, vec![op(InPlace, "f32", &[128, 256], &[256, 1], 256), op(Input, "f32", &[128, 256], &[256, 1], 256)]),
        ),
        "sk6|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f|-",
    ));

    // ---- derivation cases the token vectors cannot express ----
}

fn frame_cases(out: &mut Vec<DerivationVector>) {
    out.push(golden(
        v(
            "call_order_is_not_canonical_order",
            "KISS-CLASSIFY-6.6-0014",
            "the output listed FIRST in call order: this reference canonicalizes (a convenience — the caller's job per §6.6-0012) and yields the a1_binary_add token; a deriver that takes canonical order reports NOT-EXPRESSIBLE here",
            cell(
                CUDA,
                vec![
                    op(Output, "f32", &[128, 256], &[256, 1], 256),
                    op(Input, "f32", &[128, 256], &[256, 1], 256),
                    op(Input, "f32", &[128, 256], &[256, 1], 256),
                ],
            ),
        ),
        "sk6|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-",
    ));
    out.push(v(
        "lower_rank_operand_in_frame",
        "KISS-CLASSIFY-6.5-0014",
        "operand 1 rank 1 [256] in frame [128,256]: mask over the padded view (01), layout/vec/div from its OWN axes (co, v4, d16)",
        add_cell(&[1], &[256]),
    ));
    out.push(v(
        "extent1_axis_with_nonzero_stride",
        "KISS-CLASSIFY-6.6-0008",
        "operand 1 [1,256] recording stride 256 on the unit axis: same tensor, same sub-key as the [0,1] broadcast view (#520)",
        add_cell(&[256, 1], &[1, 256]),
    ));
    out.push(v(
        "extent1_axis_with_zero_stride",
        "KISS-CLASSIFY-6.6-0008",
        "operand 1 [1,256] recording stride 0: byte-identical to extent1_axis_with_nonzero_stride",
        add_cell(&[0, 1], &[1, 256]),
    ));
}

fn boundary_cases(out: &mut Vec<DerivationVector>) {
    out.push(v(
        "flipped_inner_axis",
        "KISS-CLASSIFY-6.5-0013",
        "operand 1 inner stride -1: flipped flag r, v1 (not a forward-unit run); layout uses |stride| so it stays contiguous",
        add_cell(&[256, -1], &[128, 256]),
    ));
    out.push(v(
        "strided_inner_axis",
        "KISS-CLASSIFY-6.5-0002",
        "operand 1 strides [512,2]: layout st, v1",
        add_cell(&[512, 2], &[128, 256]),
    ));
    {
        let mut c = add_cell(&[256, 1], &[128, 256]);
        for o in &mut c.operands {
            o.alignment = 0;
        }
        out.push(v("alignment_zero_derives_v1", "KISS-CLASSIFY-6.5-0009", "alignment 0 cannot honor a packed load: v1 on every operand", c));
    }
}

/// KISS-CLASSIFY-6.3-0011: the innermost ACTIVE axis is axis `rank-1`, including when its extent is `1`
/// (no non-unit exclusion) and, by §6.3-0002 + §6.5-0012, including when it is `0` (bucket `da`). The
/// first two are the clause's own worked examples (`[4,1]`, `[1,8]`); the third is the case a
/// consumer (Unpopped) held open: `[8,0]` derives its bucket from axis 1 (`E = 0`), NOT from the outer
/// axis. No earlier vector had an innermost extent of `0`.
fn innermost_axis_cases(out: &mut Vec<DerivationVector>) {
    let unary = |extents: &[i64], strides: &[i64]| {
        cell(CUDA, vec![op(Input, "f32", extents, strides, 256), op(Output, "f32", extents, strides, 256)])
    };
    out.push(v(
        "innermost_axis_extent_one_is_axis_rank_minus_1",
        "KISS-CLASSIFY-6.3-0011",
        "[4,1]: innermost active axis is axis 1 (extent 1: bucket da, width v1), not axis 0 (the clause's worked example)",
        unary(&[4, 1], &[1, 1]),
    ));
    out.push(v(
        "innermost_axis_extent_eight_after_unit_axis",
        "KISS-CLASSIFY-6.3-0011",
        "[1,8]: innermost active axis is axis 1 (extent 8: bucket d8) (the clause's worked example)",
        unary(&[1, 8], &[8, 1]),
    ));
    out.push(v(
        "innermost_axis_extent_zero_is_axis_rank_minus_1",
        "KISS-CLASSIFY-6.5-0012",
        "[8,0]: innermost active axis is axis 1 with E = 0: bucket da (6.5-0012 'covering ... E = 0'), NOT d8 from the outer axis",
        unary(&[8, 0], &[1, 1]),
    ));
}

fn class_cases(out: &mut Vec<DerivationVector>) {
    out.push(v(
        "sub_byte_dtype_i4",
        "KISS-CLASSIFY-6.5-0009",
        "i4 storage is under one byte: v1 whatever the alignment and extent",
        cell(CUDA, vec![op(Input, "i4", &[8, 16], &[16, 1], 256), op(Output, "i4", &[8, 16], &[16, 1], 256)]),
    ));
    out.push(v(
        "index_width_just_below_2_31",
        "KISS-CLASSIFY-6.5-0005",
        "max touched offset 32768*65535 + 32767 = 2^31 - 1: ix32",
        cell(CUDA, vec![op(Input, "f32", &[65536, 32768], &[32768, 1], 256), op(Output, "f32", &[65536, 32768], &[32768, 1], 256)]),
    ));
    out.push(v(
        "index_width_at_2_31",
        "KISS-CLASSIFY-6.5-0005",
        "max touched offset 32768*65536 + 32767 = 2^31 + 32767: ix64 (the boundary is exclusive)",
        cell(CUDA, vec![op(Input, "f32", &[65537, 32768], &[32768, 1], 256), op(Output, "f32", &[65537, 32768], &[32768, 1], 256)]),
    ));
    for (name, n) in [("work_class_32_is_warp", 32i64), ("work_class_1024_is_block", 1024), ("work_class_1025_is_grid", 1025)] {
        out.push(v(
            name,
            "KISS-CLASSIFY-6.5-0007",
            "frame element count on the work-class boundary: <=32 warp, <=1024 block, else grid",
            cell(CUDA, vec![op(Input, "f32", &[n], &[1], 256), op(Output, "f32", &[n], &[1], 256)]),
        ));
    }
}

fn quantized_and_precision_cases(out: &mut Vec<DerivationVector>) {
    {
        let mut c = cell(
            CUDA,
            vec![
                op(Input, "f32", &[1000, 64], &[64, 1], 256),
                op(Input, "u32", &[32], &[1], 256),
                op(Output, "f32", &[32, 64], &[64, 1], 256),
            ],
        );
        c.index_operand = Some(1);
        out.push(v("indexed_access_declared_index_slot", "KISS-CLASSIFY-6.5-0015", "index slot declared: class ndx whatever the shapes", c));
    }
    out.push(v(
        "not_pointwise_output_narrower_than_frame",
        "KISS-CLASSIFY-6.5-0015",
        "no declared fact, output [128,128] in frame [128,256]: oth",
        cell(CUDA, vec![op(Input, "f32", &[128, 256], &[256, 1], 256), op(Output, "f32", &[128, 128], &[128, 1], 256)]),
    ));
    {
        let mut w = op(Input, "i4", &[8, 16], &[16, 1], 256);
        w.dequant = Some(DequantForm::Linear);
        out.push(v(
            "quantized_operand_dequant_form",
            "KISS-CLASSIFY-6.3-0009a",
            "operand 0 carries a quant record (linear): sixth sub-key component dl; the scale sibling carries none",
            cell(CUDA, vec![w, op(Input, "f32", &[8, 1], &[1, 1], 256), op(Output, "f16", &[8, 16], &[16, 1], 256)]),
        ));
    }
    {
        let mut c = reduce("f32", &[4, 8], &[8, 1], &[1, 1], &[1, 1], &[0, 1]);
        c.accumulator = Some("f64");
        out.push(v("reduction_f64_accumulator", "KISS-CLASSIFY-6.7-0013", "accumulator f64 != compute f32: |f64/st", c));
        let mut c = reduce("f32", &[4, 8], &[8, 1], &[1, 1], &[1, 1], &[0, 1]);
        c.math_fidelity = MathFidelity::ReducedMantissa;
        out.push(v("reduction_reduced_mantissa_only", "KISS-CLASSIFY-6.7-0013", "accumulator == compute but <mp> rm: |f32/rm (both slots spelled)", c));
    }
}

fn decline_cases(out: &mut Vec<DerivationVector>) {
    // ---- declines ----
    out.push(v(
        "decline_scale_dtype_at_operand0",
        "KISS-CLASSIFY-6.6-0021",
        "operand 0 is an MX scale dtype: no primary dtype, never 'the next operand'",
        cell(CUDA, vec![op(Input, "f8e8m0", &[8], &[1], 256), op(Input, "f16", &[8], &[1], 256), op(Output, "f16", &[8], &[1], 256)]),
    ));
    out.push(v(
        "decline_reserved_dtype",
        "KISS-CLASSIFY-6.1-0001",
        "a reserved dtype (f8e4m3fnuz) at operand 0 — the key's dtype field (a KEY position): typed decline",
        cell(CUDA, vec![op(Input, "f8e4m3fnuz", &[8], &[1], 256), op(Input, "f8e4m3fnuz", &[8], &[1], 256), op(Output, "f8e4m3fnuz", &[8], &[1], 256)]),
    ));
    out.push(v(
        "reserved_dtype_at_non_primary_operand_derives",
        "KISS-CLASSIFY-6.1-0001",
        "a reserved dtype (f8e6m2) at operand 1 is NOT a key position (§6.6-0015): the cell derives a token, no decline — two independent derivers (Fuel, Baracuda) agreed against the first draft of this corpus on exactly this",
        cell(CUDA, vec![op(Input, "f16", &[8], &[1], 256), op(Input, "f8e6m2", &[8], &[1], 256), op(Output, "f16", &[8], &[1], 256)]),
    ));
    out.push(v(
        "decline_collapsed_reduction_output",
        "KISS-CLASSIFY-6.6-0009",
        "reduction output of rank 1 for a rank-2 frame: a collapsed output is rejected, not keyed",
        reduce("f32", &[4, 8], &[8, 1], &[4], &[1], &[1]),
    ));
    out.push(v(
        "decline_reduced_axis_out_of_range",
        "KISS-CLASSIFY-6.6-0009",
        "declared reduced axis 2 in a rank-2 frame",
        reduce("f32", &[4, 8], &[8, 1], &[4, 1], &[1, 1], &[2]),
    ));
}

fn strides_of(extents: &[i64]) -> Vec<i64> {
    let mut st = vec![1i64; extents.len()];
    for i in (0..extents.len().saturating_sub(1)).rev() {
        st[i] = st[i + 1] * extents[i + 1].max(1);
    }
    st
}

fn dense(usage: OperandUse, dtype: &'static str, extents: &[i64]) -> CellOperand {
    op(usage, dtype, extents, &strides_of(extents), 256)
}

fn axis(operand: usize, axis: usize) -> AxisRef {
    AxisRef { operand, axis }
}

/// `lhs [..B, M, K] · rhs [..B, K, N] -> out [..B, M, N]`, row-major; roles M=(lhs, r-2), K=(lhs, r-1),
/// N=(rhs, r-1), weight = rhs (slot 1), no scale, no batch axes declared (callers add them).
fn gemm(dtype: &'static str, batch: &[i64], m: i64, n: i64, k: i64) -> DerivationCell {
    let (mut l, mut r, mut o) = (batch.to_vec(), batch.to_vec(), batch.to_vec());
    l.extend([m, k]);
    r.extend([k, n]);
    o.extend([m, n]);
    let rank = l.len();
    let mut c = cell(CUDA, vec![dense(Input, dtype, &l), dense(Input, dtype, &r), dense(Output, dtype, &o)]);
    c.contraction = Some(ContractionRoles {
        m: axis(0, rank - 2),
        n: axis(1, rank - 1),
        k: axis(0, rank - 1),
        batch: vec![],
        weight: 1,
        scale: None,
    });
    c
}

fn with_batch(mut c: DerivationCell, axes: &[usize]) -> DerivationCell {
    c.contraction.as_mut().expect("ctr cell").batch = axes.to_vec();
    c
}

fn with_roles(mut c: DerivationCell, f: impl FnOnce(&mut ContractionRoles)) -> DerivationCell {
    f(c.contraction.as_mut().expect("ctr cell"));
    c
}

/// A quantized GEMM `[activation, weight i4, scale] -> out`, the operands listed in `order` (indices into
/// that list), with the declared weight and scale slots.
fn quant(scale_dtype: &'static str, weight: usize, scale: Option<usize>, order: [usize; 3]) -> DerivationCell {
    let mut w = dense(Input, "i4", &[4096, 4096]);
    w.dequant = Some(DequantForm::Linear);
    let base = [dense(Input, "bf16", &[8, 4096]), w, dense(Input, scale_dtype, &[4096, 128])];
    let mut ops: Vec<CellOperand> = order.iter().map(|&i| base[i].clone()).collect();
    ops.push(dense(Output, "bf16", &[8, 4096]));
    // M = activation axis 0, K = activation axis 1, N = weight axis 1 (slots follow `order`)
    let at = |orig: usize| order.iter().position(|&i| i == orig).expect("operand present");
    let mut c = cell(CUDA, ops);
    c.contraction = Some(ContractionRoles {
        m: axis(at(0), 0),
        n: axis(at(1), 1),
        k: axis(at(0), 1),
        batch: vec![],
        weight,
        scale,
    });
    c
}

/// R3.3: the reference contraction stage. Every role is a caller hint (§6.6-0012): the vectors vary the
/// hint and hold the operands, or the reverse, so a deriver that infers a role from extents, position or
/// dtype gives a different token on at least one of them.
fn ctr_geometry_cases(out: &mut Vec<DerivationVector>) {
    out.push(v(
        "ctr_gemm_small_m_large_nk",
        "KISS-CLASSIFY-6.6-0016",
        "the §2.8(d) worked example: lhs [8,4096] . rhs [4096,4096] -> [8,4096]; M tiny, N and K large, K divisible by 16: ctll/d16, not batched. Roles are operand axes: frame axis 0 is max(8,4096,8), so the frame extent is NOT M",
        gemm("f16", &[], 8, 4096, 4096),
    ));
    out.push(v(
        "ctr_gemm_roles_swapped_m_and_n",
        "KISS-CLASSIFY-6.6-0016",
        "the same operands as ctr_gemm_small_m_large_nk with the caller declaring M and N the other way round: the M/N classes follow the DECLARED axes (cl.. vs ct..), never the extents",
        with_roles(gemm("f16", &[], 8, 4096, 4096), |r| std::mem::swap(&mut r.m, &mut r.n)),
    ));
    out.push(v(
        "ctr_k_divisibility_from_k_axis",
        "KISS-CLASSIFY-6.5-0012",
        "K = 4100 (divisible by 4, not 8): d4, from the K axis and not from the innermost axis of the output (N = 4096 would give d16)",
        gemm("f16", &[], 8, 4096, 4100),
    ));
    {
        let mut c = gemm("f16", &[], 8, 4096, 4096);
        c.accumulator = Some("f32");
        c.math_fidelity = MathFidelity::ReducedMantissa;
        out.push(v(
            "ctr_accumulator_and_reduced_mantissa",
            "KISS-CLASSIFY-6.7-0006",
            "accumulator f32 != compute f16, <mp> rm: /f16/f32/f16/rm inside the contraction field; no separate (acc+mp) field (the two never coexist)",
            c,
        ));
    }
}

/// The batch role (§6.6-0023): a declaration, classed by the product of the declared extents.
fn ctr_batch_cases(out: &mut Vec<DerivationVector>) {
    out.push(v(
        "ctr_rank3_batch_declared",
        "KISS-CLASSIFY-6.6-0023",
        "[64,8,512] . [64,512,4096] -> [64,8,4096], batch axis 0 declared: /bs (64 is small)",
        with_batch(gemm("f16", &[64], 8, 4096, 512), &[0]),
    ));
    out.push(v(
        "ctr_rank3_batch_not_declared",
        "KISS-CLASSIFY-6.6-0023",
        "the same rank-3 operands with NO batch hint: not batched, the b coordinate is absent although the leading extent is 64",
        gemm("f16", &[64], 8, 4096, 512),
    ));
    out.push(v(
        "ctr_batch_declared_extent_one",
        "KISS-CLASSIFY-6.6-0023",
        "a declared batch axis of extent 1 is batched (/bt): batched is a declaration, not an extent test",
        with_batch(gemm("f16", &[1], 8, 4096, 512), &[0]),
    ));
    out.push(v(
        "ctr_rank4_two_batch_axes_product",
        "KISS-CLASSIFY-6.6-0023",
        "[4,64,8,512] . [4,64,512,4096]: batch axes 0 and 1 declared; the class is that of the PRODUCT 4*64 = 256 (/bm), not of either extent",
        with_batch(gemm("f16", &[4, 64], 8, 4096, 512), &[0, 1]),
    ));
    out.push(v(
        "ctr_rank4_one_batch_axis_declared",
        "KISS-CLASSIFY-6.6-0023",
        "the same rank-4 operands, only axis 1 declared: class of 64 (/bs). The hint decides which axes count",
        with_batch(gemm("f16", &[4, 64], 8, 4096, 512), &[1]),
    ));
    out.push(v(
        "ctr_rank4_no_batch_declared",
        "KISS-CLASSIFY-6.6-0023",
        "the same rank-4 operands, no hint: a rank above 3 is not batched by rank alone",
        gemm("f16", &[4, 64], 8, 4096, 512),
    ));
}

/// The weight/scale roles of a quantized cell (§6.6-0019, §6.6-0022).
fn ctr_quant_cases(out: &mut Vec<DerivationVector>) {
    // ---- quantized: weight and scale are role hints (§6.6-0019, §6.6-0022) ----
    out.push(v(
        "ctr_quantized_scale_e8m0",
        "KISS-CLASSIFY-6.6-0022",
        "[activation, weight i4, scale f8e8m0]: weight slot 1, scale slot 2: /i4/bf16/bf16/st/qf8e8m0",
        quant("f8e8m0", 1, Some(2), [0, 1, 2]),
    ));
    out.push(v(
        "ctr_quantized_scale_f16",
        "KISS-CLASSIFY-6.6-0022",
        "the same cell with an f16 scale: a different key (/qf16), because the two scales decode the same stored values to different numbers",
        quant("f16", 1, Some(2), [0, 1, 2]),
    ));
    out.push(v(
        "ctr_weight_second_undeclared_accumulator_defaults_to_operand0_dtype",
        "KISS-CLASSIFY-6.7-0006",
        "[activation bf16, weight i4 (slot 1), scale], NO accumulator declared: <acc> is operand 0's dtype bf16 (/i4/bf16/bf16), NOT the weight's i4. Pairs with the weight-first vector, where operand 0 IS the weight (/i4/i4/bf16): the default follows operand 0, never the weight role",
        quant("f8e8m0", 1, Some(2), [0, 1, 2]),
    ));
    out.push(v(
        "ctr_quantized_no_scale_designated",
        "KISS-CLASSIFY-6.6-0022",
        "the same operands, no scale designated: the q coordinate is omitted, never empty",
        quant("f8e8m0", 1, None, [0, 1, 2]),
    ));
    {
        let mut c = quant("f8e8m0", 0, Some(1), [1, 2, 0]);
        c.accumulator = Some("bf16"); // weight-first call order makes the primary dtype i4, so the accumulator is declared
        out.push(v(
            "ctr_weight_role_not_operand_one",
            "KISS-CLASSIFY-6.6-0019",
            "call order [weight i4, scale f8e8m0, activation]: the weight is operand 0 and a scale sits at position 1, so a positional operand-1 read names f8e8m0 where i4 is meant. Declared weight 0, scale 1, accumulator bf16",
            c,
        ));
    }
    {
        let mut c = quant("f8e8m0", 0, Some(1), [1, 2, 0]);
        c.accumulator = None; // WARNING (trap): the weight is operand 0, so the primary and compute dtype is i4 and <acc> defaults to it
        out.push(v(
            "ctr_weight_first_undeclared_accumulator_defaults_to_operand0_dtype",
            "KISS-CLASSIFY-6.7-0006",
            "the ctr_weight_role_not_operand_one operands with NO accumulator declared: <acc> defaults to the compute dtype = operand 0's dtype (here the weight, i4: /i4/i4/bf16). With the weight NOT at operand 0 (ctr_quantized_scale_e8m0, accumulator also undeclared) it is the operand-0 dtype bf16, not the weight's i4. Differs from the declared-bf16 vector in <acc> only",
            c,
        ));
    }
}

/// Malformed M/N/K, weight and scale hints (§6.6-0016, -0019, -0022): each a typed decline, never ignored.
fn ctr_role_decline_cases(out: &mut Vec<DerivationVector>) {
    out.push(v(
        "decline_ctr_axis_roles_not_distinct",
        "KISS-CLASSIFY-6.6-0016",
        "K names the same operand axis as M",
        with_roles(gemm("f16", &[], 8, 4096, 4096), |r| r.k = r.m),
    ));
    out.push(v(
        "decline_ctr_axis_out_of_range",
        "KISS-CLASSIFY-6.6-0016",
        "N names axis 5 of a rank-2 operand",
        with_roles(gemm("f16", &[], 8, 4096, 4096), |r| r.n = axis(1, 5)),
    ));
    out.push(v(
        "decline_ctr_axis_operand_out_of_range",
        "KISS-CLASSIFY-6.6-0016",
        "M names operand slot 9 of a 3-operand cell",
        with_roles(gemm("f16", &[], 8, 4096, 4096), |r| r.m = axis(9, 0)),
    ));
    out.push(v(
        "decline_ctr_weight_slot_out_of_range",
        "KISS-CLASSIFY-6.6-0019",
        "the weight role names slot 7 of a 3-operand cell",
        with_roles(gemm("f16", &[], 8, 4096, 4096), |r| r.weight = 7),
    ));
    out.push(v(
        "decline_ctr_scale_slot_out_of_range",
        "KISS-CLASSIFY-6.6-0022",
        "the scale role names slot 7 of a 3-operand cell: not silently ignored",
        with_roles(gemm("f16", &[], 8, 4096, 4096), |r| r.scale = Some(7)),
    ));
    {
        let mut c = gemm("f16", &[], 8, 4096, 4096);
        c.operands[1].dtype = "f8e4m3fnuz";
        out.push(v(
            "decline_ctr_reserved_weight_dtype",
            "KISS-CLASSIFY-6.1-0001",
            "the designated weight is a reserved dtype (f8e4m3fnuz): <wdt> is a key position, so unlike a sibling operand it declines",
            c,
        ));
    }
    {
        let mut c = gemm("f16", &[], 8, 4096, 4096);
        c.operands.pop();
        out.push(v("decline_ctr_no_output_operand", "KISS-CLASSIFY-6.7-0006", "no output operand: <out> has no source", c));
    }
}

/// Malformed batch-axis hints (§6.6-0023): each a typed decline, never ignored.
fn ctr_batch_decline_cases(out: &mut Vec<DerivationVector>) {
    out.push(v(
        "decline_ctr_batch_axis_out_of_range",
        "KISS-CLASSIFY-6.6-0023",
        "batch axis 7 of a rank-3 frame",
        with_batch(gemm("f16", &[64], 8, 4096, 512), &[7]),
    ));
    out.push(v(
        "decline_ctr_batch_axis_duplicate",
        "KISS-CLASSIFY-6.6-0023",
        "batch axis 0 listed twice",
        with_batch(gemm("f16", &[64], 8, 4096, 512), &[0, 0]),
    ));
    out.push(v(
        "decline_ctr_batch_axis_is_an_mnk_axis",
        "KISS-CLASSIFY-6.6-0023",
        "batch axis 1 is frame axis 1, which M occupies",
        with_batch(gemm("f16", &[64], 8, 4096, 512), &[1]),
    ));
    {
        let mut c = with_batch(gemm("f16", &[64], 8, 4096, 512), &[0]);
        c.operands[2] = dense(Output, "f16", &[8, 4096]);
        out.push(v(
            "decline_ctr_batch_axis_not_in_output",
            "KISS-CLASSIFY-6.6-0023",
            "the output is rank 2 in a rank-3 frame: frame axis 0 is not an output axis",
            c,
        ));
    }
    out.push(v(
        "decline_ctr_batch_axis_zero_extent",
        "KISS-CLASSIFY-6.6-0023",
        "a declared batch axis of extent 0 (every operand agrees): extent below 1",
        with_batch(gemm("f16", &[0], 8, 4096, 512), &[0]),
    ));
}

fn ctr_cases(out: &mut Vec<DerivationVector>) {
    ctr_geometry_cases(out);
    ctr_batch_cases(out);
    ctr_quant_cases(out);
    ctr_role_decline_cases(out);
    ctr_batch_decline_cases(out);
}

/// §6.5-0009(c) / §6.8-0018: the vector-access byte cap is the target's declared cap, not a fixed 16.
/// `rocm` is reserved and declares nothing today (so it derives with 16); these two vectors SUPPLY the
/// declaration a maintainer would make, and show the same cell deriving a different width.
fn vec_cap_cases(out: &mut Vec<DerivationVector>) {
    let base = |cap: Option<u32>| {
        let mut c = cell(
            "rocm:gfx942",
            vec![op(Input, "f32", &[64, 128], &[128, 1], 256), op(Output, "f32", &[64, 128], &[128, 1], 256)],
        );
        c.vec_cap_bytes = cap.and_then(std::num::NonZeroU32::new);
        c
    };
    out.push(v(
        "vec_cap_undeclared_is_16_bytes",
        "KISS-CLASSIFY-6.8-0018",
        "f32 [64,128] on a target declaring no cap: 16 bytes, v4 (the control for the next two)",
        base(None),
    ));
    out.push(v(
        "vec_cap_32_bytes_widens_f32_to_v8",
        "KISS-CLASSIFY-6.8-0018",
        "same cell, target declares a 32-byte cap: 8*4 = 32 <= 32, v8 (differs from the 16-byte control in vec only)",
        base(Some(32)),
    ));
    out.push(v(
        "vec_cap_8_bytes_narrows_f32_to_v2",
        "KISS-CLASSIFY-6.8-0018",
        "same cell, target declares an 8-byte cap: 2*4 = 8 <= 8, v2",
        base(Some(8)),
    ));
}

/// §6.1-0014..-0016: the MX element dtypes are usable storage dtypes in the key's dtype field, and never
/// an accumulator. An elementwise cell on one derives (the accumulator coordinate is omitted when it equals
/// the compute dtype); a contraction on one declines until the caller declares the accumulator, because
/// `<acc>` is always spelled in a contraction field and defaults to operand 0's dtype.
fn mx_element_cases(out: &mut Vec<DerivationVector>) {
    for dt in ["f4e2m1", "f6e2m3", "f6e3m2"] {
        let name: &'static str = match dt {
            "f4e2m1" => "mx_f4e2m1_elementwise_add",
            "f6e2m3" => "mx_f6e2m3_elementwise_add",
            _ => "mx_f6e3m2_elementwise_add",
        };
        out.push(v(
            name,
            "KISS-CLASSIFY-6.1-0014",
            "an elementwise add on an MX element dtype: sub-byte storage never vectorizes, so every operand is v1; no accumulator coordinate is spelled",
            cell(
                CUDA,
                vec![
                    op(Input, dt, &[64, 128], &[128, 1], 256),
                    op(Input, dt, &[64, 128], &[128, 1], 256),
                    op(Output, dt, &[64, 128], &[128, 1], 256),
                ],
            ),
        ));
    }
    out.push(v(
        "mx_f4e2m1_ctr_undeclared_accumulator_declines",
        "KISS-CLASSIFY-6.1-0016",
        "a contraction whose operand 0 is f4e2m1 with no declared accumulator: <acc> would default to the compute dtype, an MX element, so the deriver declines",
        gemm("f4e2m1", &[], 8, 4096, 4096),
    ));
    {
        let mut c = gemm("f4e2m1", &[], 8, 4096, 4096);
        c.accumulator = Some("f32");
        out.push(v(
            "mx_f4e2m1_ctr_declared_accumulator_derives",
            "KISS-CLASSIFY-6.1-0016",
            "the same contraction with accumulator f32 declared: <wdt> and <out> may be the MX element, so it derives",
            c,
        ));
    }
}

/// The vectors, in a stable order (names are the join key for consumers).
pub fn vectors() -> Vec<DerivationVector> {
    let mut out = Vec::new();
    a1_elementwise(&mut out);
    a1_reductions(&mut out);
    frame_cases(&mut out);
    boundary_cases(&mut out);
    innermost_axis_cases(&mut out);
    class_cases(&mut out);
    quantized_and_precision_cases(&mut out);
    ctr_cases(&mut out);
    decline_cases(&mut out);
    vec_cap_cases(&mut out);

    mx_element_cases(&mut out);
    out
}

/// Evaluate a vector with the reference deriver.
pub fn expect_of(d: &DerivationVector) -> Expect {
    match derive_structure_key(&d.cell) {
        Ok(k) => Expect::Token(k.to_token()),
        Err(e) => Expect::Decline(e),
    }
}

fn js(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn ints(v: &[i64]) -> String {
    format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

fn operand_json(o: &CellOperand) -> String {
    let usage = match o.usage {
        Input => "in",
        Output => "out",
        InPlace => "inout",
    };
    let dq = match o.dequant {
        Some(d) => js(d.code()),
        None => "null".to_string(),
    };
    format!(
        "{{\"use\":{},\"dtype\":{},\"extents\":{},\"strides\":{},\"alignment\":{},\"dequant_form\":{}}}",
        js(usage),
        js(o.dtype),
        ints(&o.extents),
        ints(&o.strides),
        o.alignment,
        dq
    )
}

fn contraction_json(r: Option<&ContractionRoles>) -> String {
    let Some(r) = r else { return "null".to_string() };
    let a = |x: AxisRef| format!("{{\"operand\":{},\"axis\":{}}}", x.operand, x.axis);
    format!(
        "{{\"m\":{},\"n\":{},\"k\":{},\"batch_axes\":{},\"weight\":{},\"scale\":{}}}",
        a(r.m),
        a(r.n),
        a(r.k),
        ints(&r.batch.iter().map(|&x| x as i64).collect::<Vec<_>>()),
        r.weight,
        r.scale.map_or("null".to_string(), |x| x.to_string())
    )
}

/// The artifact, exactly as committed (`corpus/structure_key_derivation_vectors.json`).
pub fn emit_json() -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"schema\": {},\n", js(SCHEMA)));
    s.push_str("  \"generated_from\": \"spec/classify.md\",\n");
    s.push_str("  \"clause\": \"KISS-CLASSIFY-6.6-0011\",\n");
    s.push_str(&format!("  \"structure_key_schema_version\": {},\n", SCHEMA_VERSION));
    s.push_str("  \"scope_note\": \"Every cell class, ctr included. A dense-contraction cell carries its caller-declared role hints in `contraction` (M/N/K as operand axes {operand, axis}, batch axes as iteration-frame axes, weight and scale as call-order operand slots); contraction_declared is true iff `contraction` is non-null. `expected` is reference output; only vectors carrying `spec_golden` reproduce an Appendix A.1 token whose bytes were hand-derived in the spec text. A second deriver cross-checks the rest.\",\n");
    s.push_str("  \"cell_note\": \"operands are listed in CALL order. The canonical order (§6.6-0014: inputs and in-place operands in call order, then outputs) is the CALLER's presentation (§6.6-0012); this reference reorders as a convenience, and a deriver whose input contract is canonical order reports NOT-EXPRESSIBLE on vector call_order_is_not_canonical_order, which is not a defect. reduced_axes are frame axes, 0 = outermost; index_operand is a call-order slot. accumulator null = the compute dtype.\",\n");
    s.push_str("  \"vectors\": [\n");
    let vs = vectors();
    for (i, d) in vs.iter().enumerate() {
        let c = &d.cell;
        let idx = c.index_operand.map_or("null".to_string(), |x| x.to_string());
        let acc = c.accumulator.map_or("null".to_string(), js);
        let (kind, val) = match expect_of(d) {
            Expect::Token(t) => ("token", t),
            Expect::Decline(e) => ("decline", e.wire().to_string()),
        };
        let gold = d.spec_golden.map_or("null".to_string(), js);
        s.push_str(&format!(
            "    {{\"name\":{},\"clause\":{},\"note\":{},\"target\":{},\"operands\":[{}],\"index_operand\":{},\"reduced_axes\":{},\"accumulator\":{},\"math_fidelity\":{},\"contraction_declared\":{},\"contraction\":{},\"expect\":{},\"value\":{},\"spec_golden\":{}{}}}{}\n",
            js(d.name),
            js(d.clause),
            js(d.note),
            js(c.target),
            c.operands.iter().map(operand_json).collect::<Vec<_>>().join(","),
            idx,
            ints(&c.reduced_axes.iter().map(|&a| a as i64).collect::<Vec<_>>()),
            acc,
            js(c.math_fidelity.code()),
            c.contraction.is_some(),
            contraction_json(c.contraction.as_ref()),
            js(kind),
            js(&val),
            gold,
            c.vec_cap_bytes.map_or(String::new(), |n| format!(",\"target_vec_access_cap_bytes\":{n}")),
            if i + 1 == vs.len() { "" } else { "," }
        ));
    }
    s.push_str("  ]\n}\n");
    s
}
