// SPDX-License-Identifier: MIT OR Apache-2.0
//! **Cell-carrying `structure_key` derivation vectors** — the second surface §6.6-0019 says is missing:
//! each vector is a RAW cell (operands `(use, dtype, extents, strides, alignment, quant)`, target,
//! declared hints) and the token a conforming deriver MUST produce, or the typed decline it MUST return.
//! `structure_key_vectors.json` round-trips tokens only; a deriver that mis-reads the spec (which axis
//! is innermost, whose dtype is primary, what "pointwise" means) is invisible to it. These vectors make
//! the derivation itself byte-comparable across parties — the instrument freeze condition 1 (umbrella
//! §5.3: each field *independently derived*) asks for.
//!
//! **Scope:** every cell class except `ctr`; the batch role is undefined (KISS#543), so contraction
//! cells are declared out of scope in the artifact rather than guessed.
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
        contraction_declared: false,
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
        "sk5|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-",
    ));
    out.push(golden(
        v("a1_binary_add_operand1_broadcast_axis0", "KISS-CLASSIFY-6.6-0008", "operand 1 stride [0,1]: layout br, mask 01, v1", add_cell(&[0, 1], &[128, 256])),
        "sk5|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;br/01/v1/d16/f;co/00/v4/d16/f|-",
    ));
    out.push(golden(
        v(
            "a1_unary_f16",
            "KISS-CLASSIFY-6.5-0009",
            "f16 [64,128]: v8 (8*2 = 16 bytes), d16, ix32, grid",
            cell(CUDA, vec![op(Input, "f16", &[64, 128], &[128, 1], 256), op(Output, "f16", &[64, 128], &[128, 1], 256)]),
        ),
        "sk5|ew1|f16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f;co/00/v8/d16/f|-",
    ));
}

fn a1_reductions(out: &mut Vec<DerivationVector>) {
    out.push(golden(
        v("a1_reduction_trailing_axis", "KISS-CLASSIFY-6.6-0009", "keepdim [4,8]->[4,1]: rlast; output mask 02 (extent-1 axis, frame extent 8)", reduce("f32", &[4, 8], &[8, 1], &[4, 1], &[1, 1], &[1])),
        "sk5|rdc|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/02/v1/da/f|rlast",
    ));
    out.push(golden(
        v("a1_reduction_all_axes", "KISS-CLASSIFY-6.6-0009", "keepdim [4,8]->[1,1]: rall; output mask 03", reduce("f32", &[4, 8], &[8, 1], &[1, 1], &[1, 1], &[0, 1])),
        "sk5|rdc|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/03/v1/da/f|rall",
    ));
    out.push(golden(
        v("a1_reduction_rank1", "KISS-CLASSIFY-6.6-0009", "rank-1 [8]->[1]: all-axes AND trailing; rall wins the tie", reduce("f32", &[8], &[1], &[1], &[1], &[0])),
        "sk5|rdc|f32|cuda:sm89|ix32|warp|r1|co/00/v1/d8/f;co/01/v1/da/f|rall",
    ));
    out.push(golden(
        v(
            "a1_reduction_subset_rank4",
            "KISS-CLASSIFY-6.6-0009",
            "[2,4,3,5]->[2,1,3,1] reducing axes 1 and 3: bitmask x0a; output mask 0a",
            reduce("f32", &[2, 4, 3, 5], &[60, 15, 5, 1], &[2, 1, 3, 1], &[3, 3, 1, 1], &[1, 3]),
        ),
        "sk5|rdc|f32|cuda:sm89|ix32|block|r4|co/00/v1/da/f;co/0a/v1/da/f|x0a",
    ));
    {
        let mut c = reduce("f16", &[4, 8], &[8, 1], &[4, 1], &[1, 1], &[1]);
        c.accumulator = Some("f32");
        out.push(golden(
            v("a1_reduction_f16_accumulates_f32", "KISS-CLASSIFY-6.7-0013", "accumulator f32 != compute f16: |f32/st carried", c),
            "sk5|rdc|f16|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/02/v1/da/f|rlast|f32/st",
        ));
    }
    out.push(golden(
        v(
            "a1_inplace_accumulate",
            "KISS-CLASSIFY-6.6-0014",
            "an in-place operand appears once, as an input; n_operands = 2",
            cell(CUDA, vec![op(InPlace, "f32", &[128, 256], &[256, 1], 256), op(Input, "f32", &[128, 256], &[256, 1], 256)]),
        ),
        "sk5|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f|-",
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
        "sk5|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-",
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
    {
        let mut c = add_cell(&[256, 1], &[128, 256]);
        c.contraction_declared = true;
        out.push(v("decline_contraction_out_of_scope", "KISS-CLASSIFY-6.6-0016", "M/N/K roles declared: out of this corpus's scope until the batch role is ruled (KISS#543)", c));
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
    decline_cases(&mut out);
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

/// The artifact, exactly as committed (`corpus/structure_key_derivation_vectors.json`).
pub fn emit_json() -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"schema\": {},\n", js(SCHEMA)));
    s.push_str("  \"generated_from\": \"spec/classify.md\",\n");
    s.push_str("  \"clause\": \"KISS-CLASSIFY-6.6-0011\",\n");
    s.push_str(&format!("  \"structure_key_schema_version\": {},\n", SCHEMA_VERSION));
    s.push_str("  \"scope_note\": \"Every cell class except ctr. A dense-contraction cell needs the batch role, which no clause defines (KISS#543); a cell that declares contraction roles is a decline here, not a guess. `expected` is reference output; only vectors carrying `spec_golden` reproduce an Appendix A.1 token whose bytes were hand-derived in the spec text. A second deriver cross-checks the rest.\",\n");
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
            "    {{\"name\":{},\"clause\":{},\"note\":{},\"target\":{},\"operands\":[{}],\"index_operand\":{},\"reduced_axes\":{},\"accumulator\":{},\"math_fidelity\":{},\"contraction_declared\":{},\"expect\":{},\"value\":{},\"spec_golden\":{}}}{}\n",
            js(d.name),
            js(d.clause),
            js(d.note),
            js(c.target),
            c.operands.iter().map(operand_json).collect::<Vec<_>>().join(","),
            idx,
            ints(&c.reduced_axes.iter().map(|&a| a as i64).collect::<Vec<_>>()),
            acc,
            js(c.math_fidelity.code()),
            c.contraction_declared,
            js(kind),
            js(&val),
            gold,
            if i + 1 == vs.len() { "" } else { "," }
        ));
    }
    s.push_str("  ]\n}\n");
    s
}
