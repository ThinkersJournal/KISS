//! KISS-Conform golden + decline vectors for the KISS-Classify `structure_key`
//! token codec (Classify §6.7). The golden tokens are transcribed verbatim from
//! Classify Appendix A.1; each is checked in both directions: `to_token` on a
//! constructed key reproduces the token, and `from_token` round-trips it.

use kiss_conformance::structure_key::*;

fn op(contig: Contig, mask: u8, vec: VecWidth, div: DivBucket, flipped: bool) -> OperandSubKey {
    OperandSubKey { contig, bcast_mask: mask, vec, div, flipped }
}

fn key(
    op_family: &str,
    dtype: &str,
    target: &str,
    work_class: WorkClass,
    rank: u32,
    operands: Vec<OperandSubKey>,
    reduce: Reduce,
    contraction: Option<Contraction>,
) -> StructureKey {
    StructureKey {
        op_family: op_family.to_string(),
        dtype: dtype.to_string(),
        target: target.to_string(),
        index_width: "ix32".to_string(),
        work_class,
        rank,
        operands,
        reduce,
        contraction,
        acc_mp: None,
    }
}

/// Build a non-`gem` key carrying the §6.7-0013 non-contraction `(acc+mp)` field.
fn key_acc_mp(
    op_family: &str,
    dtype: &str,
    work_class: WorkClass,
    rank: u32,
    operands: Vec<OperandSubKey>,
    reduce: Reduce,
    acc_mp: Option<AccMp>,
) -> StructureKey {
    StructureKey {
        op_family: op_family.to_string(),
        dtype: dtype.to_string(),
        target: "cuda:sm89".to_string(),
        index_width: "ix32".to_string(),
        work_class,
        rank,
        operands,
        reduce,
        contraction: None,
        acc_mp,
    }
}

/// Assert both directions of the codec for one Appendix A golden token.
#[track_caller]
fn assert_token(clause: &str, k: &StructureKey, golden: &str) {
    // forward: to_token reproduces the golden token exactly (§6.7).
    assert_eq!(k.to_token(), golden, "[{clause}] to_token mismatch");
    // reverse: from_token parses it, and re-serializing is byte-identical (§6.7-0008).
    let parsed = from_token(golden).unwrap_or_else(|e| panic!("[{clause}] from_token declined a golden token: {e:?}"));
    assert_eq!(&parsed, k, "[{clause}] from_token produced a different key");
    assert_eq!(parsed.to_token(), golden, "[{clause}] round-trip is not byte-identical");
}

// convenient reusable operand sub-keys
fn co4() -> OperandSubKey { op(Contig::Contiguous, 0x00, VecWidth::V4, DivBucket::D16, false) }
fn co8() -> OperandSubKey { op(Contig::Contiguous, 0x00, VecWidth::V8, DivBucket::D16, false) }
fn br1() -> OperandSubKey { op(Contig::Broadcast, 0x01, VecWidth::V1, DivBucket::D16, false) }
fn co1_d8() -> OperandSubKey { op(Contig::Contiguous, 0x00, VecWidth::V1, DivBucket::D8, false) }
fn co1_da() -> OperandSubKey { op(Contig::Contiguous, 0x00, VecWidth::V1, DivBucket::Da, false) }

// ---- Appendix A.1 golden token vectors --------------------------------------

#[test]
fn a1_elementwise_binary_canonical() {
    let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0001", &k, "sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-");
}

#[test]
fn a1_elementwise_with_broadcast_operand() {
    let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), br1(), co4()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0004", &k, "sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;br/01/v1/d16/f;co/00/v4/d16/f|-");
}

#[test]
fn a1_unary_f16_v8() {
    let k = key("une", "f16", "cuda:sm89", WorkClass::Grid, 2, vec![co8(), co8()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0003", &k, "sk4|une|f16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f;co/00/v8/d16/f|-");
}

#[test]
fn a1_reduction_trailing_axis() {
    let k = key("red", "f32", "cuda:sm89", WorkClass::Warp, 2, vec![co1_d8(), co1_da()], Reduce::Trailing, None);
    assert_token("KISS-CLASSIFY-6.7-0005", &k, "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rlast");
}

#[test]
fn a1_reduction_all_axes() {
    let k = key("red", "f32", "cuda:sm89", WorkClass::Warp, 2, vec![co1_d8(), co1_da()], Reduce::All, None);
    assert_token("KISS-CLASSIFY-6.7-0005", &k, "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall");
}

#[test]
fn a1_reduction_rank1_all_axes() {
    let k = key("red", "f32", "cuda:sm89", WorkClass::Warp, 1, vec![co1_d8(), co1_da()], Reduce::All, None);
    assert_token("KISS-CLASSIFY-6.6-0009", &k, "sk4|red|f32|cuda:sm89|ix32|warp|r1|co/00/v1/d8/f;co/00/v1/da/f|rall");
}

#[test]
fn a1_reduction_subset_mask() {
    // rank-4 reduction over a non-trivial subset -> explicit x<hh> keepdim mask 0x0a
    let k = key("red", "f32", "cuda:sm89", WorkClass::Block, 4, vec![co1_da(), co1_da()], Reduce::Subset(0x0a), None);
    assert_token("KISS-CLASSIFY-6.7-0005", &k, "sk4|red|f32|cuda:sm89|ix32|block|r4|co/00/v1/da/f;co/00/v1/da/f|x0a");
}

#[test]
fn a1_binary_two_operands() {
    let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), co4()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0001", &k, "sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f|-");
}

/// The reference-generator (`baracuda-kernelgen`) `relu_add` cell — the freeze-gate
/// head-to-head token (KISS #60). This `bin` f32 cell is **rank-1**: the generator
/// emits a flat grid-stride kernel over the flattened tensor (three contiguous
/// vectorized operands, no reduce). Its token is pinned in
/// `cuda/generated/PROVENANCE.md` and the emitted `.cu` header comment; this golden
/// makes the match **machine-checked** rather than comment-deep. Independently
/// reproduced by Fuel's Baracuda-free deriver and Baracuda's codec —
/// same-namespace three-way reproduction (records condition-1; NOT the §6.4-0004
/// cross-namespace gate, and NOT a KISS-Classify freeze). The `.cu`/`PROVENANCE`
/// tokens re-prefix to sk4 in the coordinated regen.
#[test]
fn relu_add_generated_r1_cell() {
    let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 1, vec![co4(), co4(), co4()], Reduce::None, None);
    assert_token(
        "KISS-CLASSIFY-6.7-0001",
        &k,
        "sk4|bin|f32|cuda:sm89|ix32|grid|r1|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-",
    );
}

#[test]
fn a1_dense_contraction_cuda() {
    // sk4: the gem contraction field carries the precision group. This f32 gem is
    // non-batched (no `b<class>`), f32 weight/accumulator/output, bit-stable (`st`).
    let c = Contraction {
        m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
        batch: None, wdt: "f32".to_string(), acc: "f32".to_string(), out: "f32".to_string(),
        mp: MathPrecision::Stable,
    };
    let k = key("gem", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(c));
    assert_token("KISS-CLASSIFY-6.7-0006", &k, "sk4|gem|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st");
}

#[test]
fn a1_dense_contraction_vulkan_target() {
    let c = Contraction {
        m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
        batch: None, wdt: "f32".to_string(), acc: "f32".to_string(), out: "f32".to_string(),
        mp: MathPrecision::Stable,
    };
    let k = key("gem", "f32", "vulkan:sg64.ops-abr.arith-f16.cm-none", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(c));
    assert_token("KISS-CLASSIFY-6.8", &k, "sk4|gem|f32|vulkan:sg64.ops-abr.arith-f16.cm-none|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st");
}

// ---- sk4 gem precision-coordinate goldens (D1/D4/D5, sk3-origin) -------------

/// KISS-CLASSIFY-6.7-0006 (sk4): a **batched** gem cell carries the conditionally-
/// present `b<class>` coordinate in the geometry group (right after `<kdiv>`). A
/// non-batched cell of the same shape omits it entirely, so the two are distinct
/// tokens — the batch coordinate is additive over the non-batched form, and `bm`
/// (batch-medium) never collides with the `<mp>` codes `{st,rm}` (which never begin
/// with `b`).
#[test]
fn sk4_gem_batched_cell() {
    let c = Contraction {
        m: SizeClass::Medium, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
        batch: Some(SizeClass::Medium),
        wdt: "f32".to_string(), acc: "f32".to_string(), out: "f32".to_string(),
        mp: MathPrecision::Stable,
    };
    let k = key("gem", "f32", "cuda:sm90", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(c));
    assert_token(
        "KISS-CLASSIFY-6.7-0006",
        &k,
        "sk4|gem|f32|cuda:sm90|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|cmll/d16/bm/f32/f32/f32/st",
    );
}

/// KISS-CLASSIFY-6.7-0006 (sk4, D4): SIMT-`f32` (bit-stable, `mp=st`) and TF32
/// (reduced-mantissa, `mp=rm`) are the SAME shape but numerically- and
/// determinism-distinct cells requiring different kernels — so they MUST hold
/// distinct tokens. The `<mp>` coordinate distinguishes them; the spec-forbidden
/// `f32s` dtype hack is retired (§6.1-0005).
#[test]
fn sk4_simt_f32_vs_tf32_distinct_by_mp() {
    let base = |mp| Contraction {
        m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
        batch: None, wdt: "f32".to_string(), acc: "f32".to_string(), out: "f32".to_string(), mp,
    };
    let simt = key("gem", "f32", "cuda:sm90", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(base(MathPrecision::Stable)));
    let tf32 = key("gem", "f32", "cuda:sm90", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(base(MathPrecision::ReducedMantissa)));
    assert_token(
        "KISS-CLASSIFY-6.7-0006",
        &simt,
        "sk4|gem|f32|cuda:sm90|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st",
    );
    assert_token(
        "KISS-CLASSIFY-6.7-0006",
        &tf32,
        "sk4|gem|f32|cuda:sm90|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/rm",
    );
    assert_ne!(simt.to_token(), tf32.to_token(), "SIMT-f32 and TF32 must not collide");
}

/// KISS-CLASSIFY-6.6-0018 (sk4, D1): the mixed-precision FP8 collision the precision
/// coordinates exist to resolve. Under sk2 these two GEMMs derived BYTE-IDENTICAL tokens
/// (operand-0 `e4m3`, no dtype in the contraction field), so a provider was forbidden
/// to register both (§6.6-0018). The `<wdt>/<acc>/<out>` coordinates make them
/// distinct — and the FP8 spelling is variant-explicit (`f8e4m3fn`, not bare `e4m3`).
#[test]
fn sk4_mixed_precision_fp8_disambiguated() {
    // E4M3 x E5M2 -> F32, f32 acc, bit-stable.
    let e4m3_f8e5m2_f32 = Contraction {
        m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
        batch: None, wdt: "f8e5m2".to_string(), acc: "f32".to_string(), out: "f32".to_string(),
        mp: MathPrecision::Stable,
    };
    // E4M3 x E4M3 -> F16, f32 acc, bit-stable.
    let e4m3_e4m3_f16 = Contraction {
        m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
        batch: None, wdt: "f8e4m3fn".to_string(), acc: "f32".to_string(), out: "f16".to_string(),
        mp: MathPrecision::Stable,
    };
    let a = key("gem", "f8e4m3fn", "cuda:sm90", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(e4m3_f8e5m2_f32));
    let b = key("gem", "f8e4m3fn", "cuda:sm90", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(e4m3_e4m3_f16));
    assert_token(
        "KISS-CLASSIFY-6.6-0018",
        &a,
        "sk4|gem|f8e4m3fn|cuda:sm90|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f8e5m2/f32/f32/st",
    );
    assert_token(
        "KISS-CLASSIFY-6.6-0018",
        &b,
        "sk4|gem|f8e4m3fn|cuda:sm90|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f8e4m3fn/f32/f16/st",
    );
    assert_ne!(a.to_token(), b.to_token(), "mixed-precision FP8 cells must not collide under sk4");
}

/// KISS-CLASSIFY-6.7-0009 (sk4): the grown contraction field declines malformed
/// precision groups with a typed decline — a dtype outside the closed set, a bad
/// `<mp>` code, and a wrong part-count (5 or 8 parts) are all rejected.
#[test]
fn sk4_contraction_precision_group_declines() {
    // valid sk4 gem stem for mutation.
    let ok = "sk4|gem|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st";
    assert!(from_token(ok).is_ok());
    // unknown dtype in the precision group (`f99` weight).
    assert_eq!(
        from_token(&ok.replace("/f32/f32/f32/st", "/f99/f32/f32/st")),
        Err(KeyDecline::UnknownDtype)
    );
    // bad `<mp>` code (`zz` — not st/rm).
    assert_eq!(
        from_token(&ok.replace("/f32/f32/f32/st", "/f32/f32/f32/zz")),
        Err(KeyDecline::BadContractionField)
    );
    // `<mp>` beginning with `b` is refused (reserved for the batch coordinate).
    assert_eq!(
        from_token(&ok.replace("/f32/f32/f32/st", "/f32/f32/f32/bs")),
        Err(KeyDecline::BadContractionField)
    );
    // wrong part-count: 5 parts (missing `<mp>`).
    assert_eq!(
        from_token(&ok.replace("/f32/f32/f32/st", "/f32/f32/f32")),
        Err(KeyDecline::BadContractionField)
    );
    // batched form parses (7 parts); a bad batch class `bx` is refused.
    assert!(from_token(&ok.replace("ctll/d16/", "ctll/d16/bm/")).is_ok());
    assert_eq!(
        from_token(&ok.replace("ctll/d16/", "ctll/d16/bx/")),
        Err(KeyDecline::BadContractionField)
    );
}

// ---- sk4 §6.7-0013 non-contraction (acc+mp) precision-field goldens ----------

/// KISS-CLASSIFY-6.7-0013 rule (a)/(b): a non-`gem` reduction whose accumulator
/// (`f32`) deviates from its compute dtype (`f16`) emits the optional-trailing
/// `<acc>/<mp>` field in the same slot the gem contraction group would occupy. The
/// math-precision here is the default `st`, but the field is still emitted because
/// the accumulator deviates — and rule (b) spells BOTH slots (`f32/st`), including
/// the default-valued mp slot. This is the non-contraction analogue of the gem
/// `<acc>`, realizing the §6.7-0012 forward requirement on the wire.
#[test]
fn test_classify_noncontraction_acc_mp_field() {
    let k = key_acc_mp(
        "red", "f16", WorkClass::Warp, 2, vec![co1_d8(), co1_da()], Reduce::Trailing,
        Some(AccMp { acc: "f32".to_string(), mp: MathPrecision::Stable }),
    );
    assert_token(
        "KISS-CLASSIFY-6.7-0013",
        &k,
        "sk4|red|f16|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rlast|f32/st",
    );
}

/// KISS-CLASSIFY-6.7-0013 rule (a)/(b): the accumulator equals the compute dtype
/// (`f32`) but the math-precision deviates (`rm`, reduced-mantissa), so the field is
/// still emitted — extending the strict-vs-TF32 axis (§6.7-0006) to the
/// non-contraction key. Rule (b) spells both slots (`f32/rm`), including the
/// accumulator slot equal to its default. A `scn` (scan) cell carries `-` in the
/// reduce field (non-`-` is gated to `red`, §6.6-0017); the precision field is the
/// separate optional-trailing slot.
#[test]
fn sk4_noncontraction_acc_mp_deviating_precision_only() {
    let k = key_acc_mp(
        "scn", "f32", WorkClass::Warp, 2, vec![co4(), co4()], Reduce::None,
        Some(AccMp { acc: "f32".to_string(), mp: MathPrecision::ReducedMantissa }),
    );
    assert_token(
        "KISS-CLASSIFY-6.7-0013",
        &k,
        "sk4|scn|f32|cuda:sm89|ix32|warp|r2|co/00/v4/d16/f;co/00/v4/d16/f|-|f32/rm",
    );
}

/// KISS-CLASSIFY-6.7-0013 rule (c)/(e): when NEITHER coordinate deviates (accumulator
/// == compute dtype AND mp == default `st`), the field is omitted **entirely** — no
/// trailing `|`, and specifically NOT the reduce field's `-` convention (rule e).
/// A non-deviating cell is byte-identical to the pre-sk4 codec modulo dtype renames.
#[test]
fn sk4_noncontraction_acc_mp_omitted_when_default() {
    let k = key_acc_mp("red", "f32", WorkClass::Warp, 2, vec![co1_d8(), co1_da()], Reduce::All, None);
    let token = k.to_token();
    assert_eq!(token, "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall");
    // the omitted field means a nine-field token: no trailing precision field at all.
    assert_eq!(token.split('|').count(), 9, "rule (c)/(e): no trailing precision field");
}

/// KISS-CLASSIFY-6.7-0013 rule (d): the all-default spelling (accumulator == compute
/// dtype AND mp == default `st`) is a forbidden redundant emission and MUST be
/// rejected — a canonical producer omits the field entirely (rule c), so its
/// presence at default is invalid. Distinct from the malformed-field decline.
#[test]
fn sk4_noncontraction_acc_mp_rejects_redundant_default() {
    // f32 compute, f32 accumulator, default `st` — all-default → RedundantAccMpField.
    let redundant = "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall|f32/st";
    assert_eq!(from_token(redundant), Err(KeyDecline::RedundantAccMpField));
    // but a deviating spelling on the same stem parses (accumulator f64 ≠ compute f32).
    let deviating = "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall|f64/st";
    assert!(from_token(deviating).is_ok(), "a genuinely-deviating acc+mp field must parse");
    // and a default accumulator with a deviating mp parses (mp ≠ default).
    let mp_only = "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall|f32/rm";
    assert!(from_token(mp_only).is_ok(), "a deviating-mp-only acc+mp field must parse");
}

/// KISS-CLASSIFY-6.7-0009/6.7-0013: the non-contraction `(acc+mp)` field declines
/// malformed spellings with typed declines — an accumulator outside the closed §6.1
/// set, a reserved accumulator, a bad `<mp>` code, and a wrong part-count (1 or 3
/// parts, incl. a contraction-shaped payload wrongly placed on a non-gem cell).
#[test]
fn sk4_noncontraction_acc_mp_declines() {
    let stem = "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall";
    // unknown accumulator dtype.
    assert_eq!(from_token(&format!("{stem}|f99/rm")), Err(KeyDecline::UnknownDtype));
    // reserved accumulator dtype (recognized-but-reserved, distinct from unknown).
    assert_eq!(from_token(&format!("{stem}|f8e4m3fnuz/rm")), Err(KeyDecline::ReservedDtype));
    // bad `<mp>` code (`zz` — not st/rm).
    assert_eq!(from_token(&format!("{stem}|f64/zz")), Err(KeyDecline::BadAccMpField));
    // wrong part-count: 1 part (no `<mp>`).
    assert_eq!(from_token(&format!("{stem}|f64")), Err(KeyDecline::BadAccMpField));
    // wrong part-count: 3 parts.
    assert_eq!(from_token(&format!("{stem}|f64/st/x")), Err(KeyDecline::BadAccMpField));
    // a contraction-shaped payload on a non-gem cell is simply a malformed acc+mp
    // field (6 parts, not 2) — under sk4 the 10th slot IS the acc+mp field here.
    assert_eq!(
        from_token(&format!("{stem}|ctll/d16/f32/f32/f32/st")),
        Err(KeyDecline::BadAccMpField)
    );
}

// ---- §6.5-0009 vector-width derivation ---------------------------------------

/// KISS-CLASSIFY-6.5-0009: the vector-access width derivation — the byte cap, the
/// exact-modulo alignment gate, extent divisibility, the `alignment = 0` and
/// sub-byte-dtype `v1` cases, and the **zero-extent** rule (KISS #82 finding 4,
/// ruled 2026-07-23): at `E = 0` the divisibility test is *vacuously* satisfied,
/// so a naive derivation elects the largest admissible `L`; the ruled reading is
/// `v1`, consistent with §6.5-0012 bucketing `E = 0` as `da`.
#[test]
fn test_classify_vec_width_derivation() {
    let f32 = Some(4u32); // 4 storage bytes
    let f64 = Some(8u32);

    // (c) positive control: 4*4 = 16 <= cap, 256 % 16 == 0, 256 % 4 == 0 -> v4.
    assert_eq!(derive_vec_width(1, 256, f32, 256, false), VecWidth::V4);
    // byte cap: f64 v8 would be 64 bytes > 16; v2 = 16 bytes fits -> v2.
    assert_eq!(derive_vec_width(1, 256, f64, 256, false), VecWidth::V2);
    // extent divisibility: 6 is divisible by 2 but not 4 -> v2.
    assert_eq!(derive_vec_width(1, 6, f32, 256, false), VecWidth::V2);
    // odd extent divides by nothing -> v1.
    assert_eq!(derive_vec_width(1, 7, f32, 256, false), VecWidth::V1);
    // alignment = 0 (unspecified) cannot honor a packed load -> v1.
    assert_eq!(derive_vec_width(1, 256, f32, 0, false), VecWidth::V1);
    // exact-modulo alignment gate, the discriminating case (§6.5-0009): f16,
    // alignment 24. 24 % (4*2)=0 -> v4 passes; 24 % (8*2)=8 != 0 -> v8 fails, even
    // though v8 is within the f16 byte cap (8*2=16<=16). So the divisor test yields
    // v4, NOT the naive floor(24/2)=12->8 shortcut's v8 -- this is the case the
    // clause's (now-corrected) example must demonstrate, which the old f32/48 one
    // could not (for f32 the 16-byte load cap already limits the result to v4, so the
    // shortcut is capped back to v4 and never diverges).
    assert_eq!(derive_vec_width(1, 256, Some(2u32), 24, false), VecWidth::V4);
    // sub-byte dtype (storage under one byte, encoded None) -> v1.
    assert_eq!(derive_vec_width(1, 256, None, 256, false), VecWidth::V1);

    // ---- the #82 finding-4 ruling: zero-extent innermost axis -> v1 ----------
    // E = 0 satisfies "L divides E" for EVERY L, and 256 % 16 == 0, so the naive
    // reading would return v4 here (the vacuous-truth trap Fuel surfaced). The
    // ruled reading is v1: a zero-length run has nothing to load.
    assert_eq!(derive_vec_width(1, 0, f32, 256, false), VecWidth::V1, "E=0 must not vectorize");
    // ...and it holds however permissive the dtype and alignment are.
    assert_eq!(derive_vec_width(1, 0, f64, 256, false), VecWidth::V1);
    assert_eq!(derive_vec_width(1, 0, f32, 4096, false), VecWidth::V1);
    // cross-check the §6.5-0012 companion: E=0 buckets `da`, so both derivations
    // now agree that the empty case is the degenerate one, not the maximal one.
    assert_eq!(derive_div_bucket(0), DivBucket::Da);
}

// ---- §6.5-0013 vector-width derivation: forward-unit-stride precondition ------

/// KISS-CLASSIFY-6.5-0013 (`test_classify_vec_width_unit_stride`): a `vL` with
/// `L > 1` requires the innermost active axis to be forward-unit-stride; a
/// flipped (`stride = -1`) or strided (`|stride| > 1`) innermost axis derives
/// `v1` even when the byte-cap / extent / alignment tests of §6.5-0009(c) would
/// otherwise pass, and a non-power-of-two alignment gates by exact modulo.
#[test]
fn test_classify_vec_width_unit_stride() {
    // f32: 4 storage bytes; inner extent 256 (divisible by 8/4/2); alignment 256.
    let f32 = Some(4u32);

    // forward-unit-stride, no broadcast -> vectorizes (v4: 4*4=16 <= 16 byte cap,
    // 256 % 16 == 0, 256 % 4 == 0). This is the positive control.
    assert_eq!(derive_vec_width(1, 256, f32, 256, false), VecWidth::V4);

    // flipped innermost axis (stride -1): |stride| == 1 but not a forward run ->
    // v1. This is the transposed/reversed-operand trap §6.5-0013 closes.
    assert_eq!(derive_vec_width(-1, 256, f32, 256, false), VecWidth::V1);

    // NOTE on the §6.6-0007 flipped flag: it is deliberately NOT a parameter here.
    // The flag is set iff *any* axis of the operand has a negative stride, whereas
    // §6.5-0013 tests the INNERMOST axis's signed stride. An operand whose outer
    // axis is reversed but whose innermost run is forward-contiguous has the flag
    // set and still derives v4 -- gating on the flag would wrongly derive v1.
    // This derivation cannot make that mistake because it never sees the flag;
    // the hazard is for a foreign implementer reading the clause, which no test
    // here can catch. Hence the wording of §6.5-0013 rather than a case below.

    // strided innermost axis (|stride| == 2, e.g. a transpose): -> v1.
    assert_eq!(derive_vec_width(2, 256, f32, 256, false), VecWidth::V1);

    // forward-unit-stride but an axis broadcasts -> v1.
    assert_eq!(derive_vec_width(1, 256, f32, 256, true), VecWidth::V1);

    // §6.5-0009(c) exact-modulo alignment gate: alignment 48 (non-power-of-two)
    // for f32 admits v4 (48 % 16 == 0) but NOT v8 (48 % 32 != 0); a power-of-two
    // floor (A = 32) would have wrongly admitted v8.
    assert_eq!(derive_vec_width(1, 256, f32, 48, false), VecWidth::V4);

    // alignment 0 (unspecified) cannot honor a packed load -> v1.
    assert_eq!(derive_vec_width(1, 256, f32, 0, false), VecWidth::V1);

    // sub-byte dtype (storage < 1 byte) -> v1 regardless of stride/alignment.
    assert_eq!(derive_vec_width(1, 256, None, 256, false), VecWidth::V1);
}

// ---- A.2 decline vectors: structural codec rejects (§6.7-0009) ---------------

const A_GOLDEN: &str = "sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-";

#[test]
fn reject_wrong_field_count() {
    assert_eq!(from_token("sk4|bin|f32"), Err(KeyDecline::WrongFieldCount { got: 3 }));
}

#[test]
fn reject_bad_version_prefix() {
    // the "wall": a MALFORMED version field (no `sk` prefix) is BadVersionPrefix,
    // distinct from a well-formed-but-other version (see reject_unsupported_schema_version).
    let t = A_GOLDEN.replacen("sk4", "vv4", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::BadVersionPrefix));
    // a non-numeric remainder after `sk` is likewise the wall.
    let t2 = A_GOLDEN.replacen("sk4", "skx", 1);
    assert_eq!(from_token(&t2), Err(KeyDecline::BadVersionPrefix));
}

/// KISS-CLASSIFY-6.7-0015: the version-field decline splits, on the §6.1-0001
/// recognized-vs-unknown model, into a SIGNPOST and a WALL. A well-formed CANONICAL
/// version that isn't this build's is UnsupportedSchemaVersion{got} — a recognized
/// token from another schema, so a consumer re-derives rather than treating it as
/// garbage. A malformed field (no `sk`, non-numeric, or a non-canonical leading zero)
/// is BadVersionPrefix — not a token. The split is what makes the §6.7-0014 MAY-arm
/// safe in both directions.
#[test]
fn test_classify_unsupported_schema_version() {
    // signpost: a well-formed CANONICAL other version → UnsupportedSchemaVersion{got}.
    assert_eq!(
        from_token(&A_GOLDEN.replacen("sk4", "sk9", 1)),
        Err(KeyDecline::UnsupportedSchemaVersion { got: 9 })
    );
    assert_eq!(
        from_token(&A_GOLDEN.replacen("sk4", "sk3", 1)),
        Err(KeyDecline::UnsupportedSchemaVersion { got: 3 })
    );
    // wall: a malformed version field stays BadVersionPrefix, NOT the signpost —
    // no `sk` prefix, a non-numeric remainder, or the non-canonical leading zero.
    assert_eq!(from_token(&A_GOLDEN.replacen("sk4", "vv4", 1)), Err(KeyDecline::BadVersionPrefix));
    assert_eq!(from_token(&A_GOLDEN.replacen("sk4", "skx", 1)), Err(KeyDecline::BadVersionPrefix));
    assert_eq!(from_token(&A_GOLDEN.replacen("sk4|", "sk04|", 1)), Err(KeyDecline::BadVersionPrefix));
}

/// KISS-CLASSIFY-6.7-0014: a build MAY retain a bounded `sk3` decoder arm through the
/// `sk4` series but MUST retire it by `sk5`; a consumer MUST NOT rely on
/// cross-implementation `sk3` acceptance. This REFERENCE build ships NO `sk3` arm, so
/// it declines a well-formed `sk3` token — the conformant no-arm choice — and does so
/// with the signpost (UnsupportedSchemaVersion), telling a consumer to re-derive.
#[test]
fn test_classify_no_sk3_decoder_arm() {
    let sk3 = A_GOLDEN.replacen("sk4", "sk3", 1);
    // ships no accept-both arm: the sk3 token is NOT decoded, it is declined …
    assert_eq!(from_token(&sk3), Err(KeyDecline::UnsupportedSchemaVersion { got: 3 }));
    // … and specifically NOT accepted as if it were sk4 (no silent cross-vocab decode).
    assert!(from_token(&sk3).is_err(), "reference build must not accept a persisted sk3 token");
}

#[test]
fn reject_bad_reduce_field() {
    let t = format!("{}zzz", A_GOLDEN.strip_suffix('-').unwrap());
    assert_eq!(from_token(&t), Err(KeyDecline::BadReduceField));
}

#[test]
fn reject_uppercase_hex_mask() {
    // §6.7-0010: masks must be lowercase, 2 digits — a `FF` broadcast mask is refused.
    let t = A_GOLDEN.replacen("co/00/", "co/FF/", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::UppercaseOrWidthHex));
}

#[test]
fn reject_bad_operand_subkey() {
    let t = A_GOLDEN.replacen("co/00/v4/d16/f", "zz/00/v4/d16/f", 1); // unknown contig code
    assert_eq!(from_token(&t), Err(KeyDecline::BadOperandSubKey));
}

#[test]
fn reject_bad_work_class() {
    let t = A_GOLDEN.replacen("|grid|", "|foo|", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::BadWorkClass));
}

#[test]
fn reject_bad_rank() {
    let t = A_GOLDEN.replacen("|r2|", "|rX|", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::BadRank));
}

#[test]
fn reject_unknown_op_family() {
    // A.2 decline vector: `sk4|zzz|f32|…` — op-family outside the closed §6.5-0006 set.
    let t = A_GOLDEN.replacen("|bin|", "|zzz|", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::UnknownOpFamily));
}

#[test]
fn reject_unknown_dtype() {
    // A.2 decline vector: `sk4|bin|f99|…` — dtype outside the closed §6.1 set.
    let t = A_GOLDEN.replacen("|f32|", "|f99|", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::UnknownDtype));
}

#[test]
fn accepts_every_closed_op_family_and_dtype() {
    // every one of the 24 op-family codes and 24 dtype tokens is recognized
    for fam in OP_FAMILIES {
        let t = A_GOLDEN.replacen("|bin|", &format!("|{fam}|"), 1);
        assert_ne!(from_token(&t), Err(KeyDecline::UnknownOpFamily), "op-family {fam} rejected");
    }
    for dt in DTYPES {
        let t = A_GOLDEN.replacen("|f32|", &format!("|{dt}|"), 1);
        if RESERVED_DTYPES.contains(&dt) {
            // in the closed vocabulary — recognized on parse — but reserved:
            // use typed-declines at this schema version (§6.1-0001).
            assert_eq!(from_token(&t), Err(KeyDecline::ReservedDtype), "reserved dtype {dt}");
        } else {
            assert_ne!(from_token(&t), Err(KeyDecline::UnknownDtype), "dtype {dt} rejected");
            assert_ne!(from_token(&t), Err(KeyDecline::ReservedDtype), "active dtype {dt} declined as reserved");
        }
    }
}

#[test]
fn test_classify_work_class_element_count() {
    // §6.5-0010 + §6.6-0013 (backs the clause; KISS #82 finding 2 ruling, 2026-07-23):
    // the work-class total element count is the FRAME-MAX — the product over the
    // iteration-frame axes of each axis's max extent across operands — NOT the output
    // operand's frame. Boundaries §6.5-0007: <=32 warp, <=1024 block, else grid.

    // The disambiguating cell the ruling asks for: a `gem` whose two readings land in
    // DIFFERENT buckets. lhs[M,K]=[8,4096], rhs[K,N]=[4096,8], out[M,N]=[8,8].
    //   frame-max (all rank 2, right-aligned): axis0 = max(8,4096,8) = 4096,
    //                                          axis1 = max(4096,8,8) = 4096
    //     => 4096 * 4096 = 16_777_216  (> 1024)            => grid    <- RULED reading
    //   output-frame (out only): 8 * 8 = 64  (in 33..=1024) => block  <- REJECTED reading
    // The two readings genuinely diverge, so this golden forces any implementation
    // (incl. Baracuda at the next regen) to reveal which one it computes.
    let lhs = [8i64, 4096];
    let rhs = [4096i64, 8];
    let out = [8i64, 8];
    assert_eq!(
        work_class_element_count(&[&lhs, &rhs, &out]),
        16_777_216,
        "frame-max element count for the skinny-output gem cell"
    );
    assert_eq!(derive_work_class(&[&lhs, &rhs, &out]), WorkClass::Grid, "frame-max => grid");
    // prove the rejected output-frame reading really would disagree:
    assert_eq!(derive_work_class(&[&out]), WorkClass::Block, "output-frame alone => block (the rejected reading)");

    // Second disambiguating cell — catches the OPERAND-0-NUMEL reading (distinct from
    // the output-frame reading above). A gem lhs[M,K]=[8,8], rhs[K,N]=[8,4096],
    // out[M,N]=[8,4096]:
    //   frame-max: axis0 = max(8,8,8) = 8, axis1 = max(8,4096,4096) = 4096
    //     => 8 * 4096 = 32768 (> 1024)          => grid   <- RULED reading
    //   operand-0 (lhs) numel: 8 * 8 = 64  (33..=1024)  => block  <- a REJECTED reading
    //   output-frame (out): 8 * 4096 = 32768            => grid   (agrees here — so this
    //     cell does NOT catch output-frame; the first cell does. The two cells together
    //     pin frame-max against BOTH the output-frame and operand-0-numel readings.)
    // Operand-0-numel diverges from frame-max whenever operand-0 is not the frame-
    // defining operand — here lhs is small on the N axis that rhs/out make large.
    let g2_lhs = [8i64, 8];
    let g2_rhs = [8i64, 4096];
    let g2_out = [8i64, 4096];
    assert_eq!(work_class_element_count(&[&g2_lhs, &g2_rhs, &g2_out]), 32_768, "frame-max mixes the large N axis in");
    assert_eq!(derive_work_class(&[&g2_lhs, &g2_rhs, &g2_out]), WorkClass::Grid, "frame-max => grid");
    assert_eq!(derive_work_class(&[&g2_lhs]), WorkClass::Block, "operand-0 (lhs) numel alone => block (a rejected reading)");

    // Appendix A.1 skinny-decode cell: here both readings bucket the same (grid), so it
    // does NOT disambiguate — pinned to lock the frame-max VALUE (4096*4096), not 8*4096.
    let a1_lhs = [8i64, 4096];
    let a1_rhs = [4096i64, 4096];
    let a1_out = [8i64, 4096];
    assert_eq!(work_class_element_count(&[&a1_lhs, &a1_rhs, &a1_out]), 16_777_216);
    assert_eq!(derive_work_class(&[&a1_lhs, &a1_rhs, &a1_out]), WorkClass::Grid);

    // §6.5-0007 boundary cells (single-operand frames): exact thresholds.
    assert_eq!(derive_work_class(&[&[32i64]]), WorkClass::Warp, "32 is the warp ceiling");
    assert_eq!(derive_work_class(&[&[33i64]]), WorkClass::Block, "33 crosses to block");
    assert_eq!(derive_work_class(&[&[1024i64]]), WorkClass::Block, "1024 is the block ceiling");
    assert_eq!(derive_work_class(&[&[1025i64]]), WorkClass::Grid, "1025 crosses to grid");

    // right-alignment: a lower-rank operand broadcasts on the leading frame axis
    // (contributes extent 1), so [4,8] vs [8] => axis0 max(4,1)=4, axis1 max(8,8)=8 => 32.
    assert_eq!(work_class_element_count(&[&[4i64, 8], &[8i64]]), 32);
    assert_eq!(derive_work_class(&[&[4i64, 8], &[8i64]]), WorkClass::Warp);
}

#[test]
fn test_classify_broadcast_axis_mask() {
    // KISS-CLASSIFY-6.6-0008: the broadcast-axis mask is a bitmask over iteration-frame
    // axes (§6.6-0013), bit `i` (LSB = axis 0, outermost-first §6.3-0011) set iff frame
    // axis `i` has extent > 1 AND this operand's stride there is 0. The load-bearing
    // guard is `extent > 1` — the mask is NOT "any zero stride sets the bit".

    // THE disambiguating cell the clause names. A size-1 leading axis carries stride
    // 0 legitimately (there is nothing to walk), so extents [1,256] strides [0,1]
    // MUST yield mask 0 — NOT 0b01. The naive "any zero stride" impl sets bit 0 here
    // and is caught: axis0 has extent 1 (not > 1), axis1 has stride 1 (not 0), so
    // neither axis broadcasts.
    assert_eq!(
        derive_bcast_mask(&[1, 256], &[0, 1]),
        0b00,
        "KISS-CLASSIFY-6.6-0008: size-1 axis with stride 0 is NOT a broadcast — the naive any-zero-stride impl wrongly returns 0b01"
    );

    // The genuine broadcast: same layout but the leading axis is REAL (extent 256)
    // and the operand refuses to walk it (stride 0) — bit 0 set.
    assert_eq!(
        derive_bcast_mask(&[256, 256], &[0, 1]),
        0b01,
        "a real (extent>1) axis with stride 0 IS a broadcast — bit 0 set"
    );

    // Bit position tracks the axis index (LSB = axis 0). A broadcast on the inner
    // axis (axis 1) sets bit 1, not bit 0 — catches a reversed bit order.
    assert_eq!(derive_bcast_mask(&[256, 256], &[1, 0]), 0b10, "inner-axis broadcast sets bit 1");
    // Both real axes broadcasting sets both bits.
    assert_eq!(derive_bcast_mask(&[256, 256], &[0, 0]), 0b11, "both real axes broadcast");
    // A fully-walked operand (no zero stride) has an empty mask.
    assert_eq!(derive_bcast_mask(&[256, 256], &[256, 1]), 0b00, "no broadcast axis => empty mask");
    // Every axis size-1: no bit is ever set even though every stride is 0.
    assert_eq!(derive_bcast_mask(&[1, 1], &[0, 0]), 0b00, "all size-1 axes never set a bit even at stride 0");

    // MAX_RANK bound (§6.6-0008: a single byte suffices). Axis 0 at full rank 8 sets
    // the LSB; axis 7 (the highest representable) sets the MSB — the whole mask still
    // fits one byte.
    assert_eq!(
        derive_bcast_mask(&[2, 2, 2, 2, 2, 2, 2, 2], &[0, 1, 2, 4, 8, 16, 32, 64]),
        0b0000_0001,
        "axis-0 broadcast at full rank 8 sets bit 0"
    );
    assert_eq!(
        derive_bcast_mask(&[2, 2, 2, 2, 2, 2, 2, 2], &[1, 2, 4, 8, 16, 32, 64, 0]),
        0b1000_0000,
        "axis-7 (highest representable) broadcast sets the MSB (bit 7)"
    );

    // Consistency with derive_layout_tag part (1): a non-empty mask iff the layout tag
    // is Broadcast — the two use the identical extent>1 && stride==0 predicate
    // (§6.5-0002 vs §6.6-0008), so they can never disagree on whether ANY axis
    // broadcasts.
    assert_ne!(derive_bcast_mask(&[256, 256], &[0, 1]), 0);
    assert_eq!(derive_layout_tag(&[256, 256], &[0, 1]), Contig::Broadcast);
    assert_eq!(derive_bcast_mask(&[1, 256], &[0, 1]), 0);
    assert_ne!(derive_layout_tag(&[1, 256], &[0, 1]), Contig::Broadcast);
}

#[test]
fn reserved_fnuz_dtypes_typed_decline() {
    // §6.1-0001: `f8e4m3fnuz`/`f8e5m2fnuz` are IN the closed vocabulary (their
    // spellings pinned now so a byte-incompatible variant can never squat on
    // them) but reserved — recognized on parse, distinct from an unknown token,
    // and their use typed-declines at this schema version. Cross-implementation
    // seam: Baracuda's reader declines them identically (PR #81 review note).
    for dt in RESERVED_DTYPES {
        let t = A_GOLDEN.replacen("|f32|", &format!("|{dt}|"), 1);
        assert_eq!(from_token(&t), Err(KeyDecline::ReservedDtype), "{dt} in the dtype field");
    }
    // the retired bare spelling stays an UNKNOWN token — the two declines differ.
    let bare = A_GOLDEN.replacen("|f32|", "|e4m3|", 1);
    assert_eq!(from_token(&bare), Err(KeyDecline::UnknownDtype));
    // a reserved spelling inside the gem precision group declines the same way.
    let gem = "sk4|gem|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f8e4m3fnuz/f32/st";
    assert_eq!(from_token(gem), Err(KeyDecline::ReservedDtype));
}

#[test]
fn producer_emits_rall_not_equivalent_mask() {
    // §6.7-0005: the all-axes and trailing cases MUST serialize as rall / rlast,
    // never the equivalent x<hh> bitmask.
    let all = key("red", "f32", "cuda:sm89", WorkClass::Warp, 2, vec![co4(), co4()], Reduce::All, None);
    assert!(all.to_token().ends_with("|rall"));
    let last = key("red", "f32", "cuda:sm89", WorkClass::Warp, 2, vec![co4(), co4()], Reduce::Trailing, None);
    assert!(last.to_token().ends_with("|rlast"));
}

// ---- §6.4 / §6.7 / §6.8 codec bounds: a reader MUST reject a malformed key -----
// Each of the four below caught a real hole in the shipped from_token: it parsed
// the malformed token as valid. The reference reader is now a validating reader.

/// KISS-CLASSIFY-6.4-0004 (`test_classify_structure_key_token_length_bound`): a
/// reader MUST reject a token of length 0 or > MAX_STRUCTURE_KEY_LEN (4096), and
/// on the O(1) byte length before allocating on it. Catches a reader that splits
/// first and parses an attacker-sized token (the previous one did).
#[test]
fn test_classify_structure_key_token_length_bound() {
    assert_eq!(MAX_STRUCTURE_KEY_LEN, 4096);
    assert_eq!(from_token(""), Err(KeyDecline::TokenLengthOutOfBound { len: 0 }));
    let too_long = "x".repeat(MAX_STRUCTURE_KEY_LEN + 1);
    assert_eq!(
        from_token(&too_long),
        Err(KeyDecline::TokenLengthOutOfBound { len: 4097 })
    );
    // the canonical golden is well within bound and still parses.
    assert!(from_token(A_GOLDEN).is_ok());
}

/// KISS-CLASSIFY-6.4-0002 (`test_classify_max_operands_is_8`): a reader MUST
/// reject a key declaring more than MAX_OPERANDS (8) per-operand sub-keys. Catches
/// a reader that `split(';').collect()`s an unbounded operand vector.
#[test]
fn test_classify_max_operands_is_8() {
    assert_eq!(MAX_OPERANDS, 8);
    let nine = std::iter::repeat("co/00/v4/d16/f").take(9).collect::<Vec<_>>().join(";");
    let t = A_GOLDEN.replacen("co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f", &nine, 1);
    assert_eq!(from_token(&t), Err(KeyDecline::TooManyOperands { got: 9 }));
    // exactly 8 is accepted (the boundary is not over-tight).
    let eight = std::iter::repeat("co/00/v4/d16/f").take(8).collect::<Vec<_>>().join(";");
    let t8 = A_GOLDEN.replacen("co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f", &eight, 1);
    assert!(from_token(&t8).is_ok());
}

/// KISS-CLASSIFY-6.8-0001 (`test_classify_target_token_grammar`): target_capability
/// MUST be `<namespace>:<capability-set>` — exactly one `:`, both parts non-empty.
/// Catches a reader that copies field 3 unchecked (the previous one did): a bare
/// `cuda` with no capability set would pass and later fail to select a kernel.
#[test]
fn test_classify_target_token_grammar() {
    for bad in ["cuda", "cuda:sm89:extra", ":sm89", "cuda:"] {
        let t = A_GOLDEN.replacen("cuda:sm89", bad, 1);
        assert_eq!(from_token(&t), Err(KeyDecline::BadTargetGrammar), "accepted `{bad}`");
    }
    assert!(from_token(A_GOLDEN).is_ok()); // `cuda:sm89` is well-formed
}

/// KISS-CLASSIFY-6.7-0002 (`test_classify_token_version_prefix`): field 0 is `sk`
/// followed by the canonical decimal version. Catches a reader that `parse::<u32>()`s
/// the version (accepting the non-canonical `sk04`, which parses to 4).
#[test]
fn test_classify_token_version_prefix() {
    // non-canonical leading-zero `sk04` is the WALL (BadVersionPrefix): a bare
    // parse::<u32>() would accept it as 4 — the load-bearing guard §6.7-0015 keeps.
    assert_eq!(
        from_token(&A_GOLDEN.replacen("sk4|", "sk04|", 1)),
        Err(KeyDecline::BadVersionPrefix)
    );
    // a well-formed CANONICAL other version `sk9` is the SIGNPOST
    // (UnsupportedSchemaVersion), not the wall — §6.7-0015 recognized-vs-malformed split.
    assert_eq!(
        from_token(&A_GOLDEN.replacen("sk4|", "sk9|", 1)),
        Err(KeyDecline::UnsupportedSchemaVersion { got: 9 })
    );
    assert!(from_token(A_GOLDEN).is_ok()); // `sk4` is canonical
}

/// KISS-CLASSIFY-6.6-0010 (`test_classify_contraction_field_optional`): the
/// contraction field is present IFF op_family == gem. A reader MUST reject a
/// 10-field non-gem token and a 9-field gem token with a typed decline. Catches a
/// reader that does not cross-check family vs contraction presence (the shipped
/// from_token parsed both as valid).
#[test]
fn test_classify_contraction_field_optional() {
    let gem = A_GOLDEN.replacen("|bin|", "|gem|", 1);
    // gem WITH contraction (10 fields) — the only valid gem form (sk4 precision group).
    let gem_10 = format!("{gem}|ctll/d16/f32/f32/f32/st");
    assert!(from_token(&gem_10).is_ok(), "valid gem+contraction rejected");
    // gem WITHOUT contraction (9 fields) — must be rejected (ContractionPresenceMismatch
    // now fires ONLY on this arm: a gem cell missing its mandatory precision field).
    assert_eq!(from_token(&gem), Err(KeyDecline::ContractionPresenceMismatch));
    // non-gem (bin) WITH a contraction-shaped 10th field — under sk4 the 10th slot of a
    // non-gem cell IS the `(acc+mp)` field (§6.7-0013), so a 6-part contraction payload
    // is simply a malformed acc+mp field (not 2 parts) → BadAccMpField. The pre-sk4
    // "non-gem carrying the contraction field" framing is obsolete: the op-family now
    // decides what the 10th field *is*, not merely whether it may be present.
    let bin_10 = format!("{A_GOLDEN}|ctll/d16/f32/f32/f32/st");
    assert_eq!(from_token(&bin_10), Err(KeyDecline::BadAccMpField));
    // non-gem (bin) WITHOUT a trailing field (9 fields) — the base codec, accepted.
    assert!(from_token(A_GOLDEN).is_ok());
}

/// KISS-CLASSIFY-6.6-0017 (`test_classify_reduce_field_op_family_gated`): the
/// reduce field is non-`-` ONLY for a reduction cell (op_family == red); every
/// other family MUST carry `-`. A reader MUST reject a non-`red` token whose
/// reduce field is not `-` with a typed decline. Catches a reader that parses the
/// reduce field without gating it on op_family (the shipped from_token did).
#[test]
fn test_classify_reduce_field_op_family_gated() {
    let stem = A_GOLDEN.strip_suffix('-').unwrap(); // "...|"
    // a non-red (bin) cell carrying any non-`-` reduce value is rejected.
    for r in ["rall", "rlast", "x0a"] {
        let t = format!("{stem}{r}");
        assert_eq!(
            from_token(&t),
            Err(KeyDecline::ReduceNotGatedToRed),
            "non-red cell accepted reduce `{r}`"
        );
    }
    // a non-red cell with `-` is the base form, accepted.
    assert!(from_token(A_GOLDEN).is_ok());
    // a red cell carries the same reduce values legitimately. `x01` (not the out-of-range
    // `x0a`) is a well-formed canonical subset at A_GOLDEN's rank 2: it reduces axis 0
    // only — neither all-axes (rall), the lone trailing axis (rlast), nor empty (`-`) —
    // so it is the one bitmask spelling that survives §6.6-0009 canonicalization here.
    let red_stem = stem.replacen("|bin|", "|red|", 1);
    for r in ["rall", "rlast", "x01", "-"] {
        let t = format!("{red_stem}{r}");
        assert!(from_token(&t).is_ok(), "red cell rejected reduce `{r}`");
    }
}

/// KISS-CLASSIFY-6.7-0005 / -6.6-0009 (#160): field 8 MUST be one of the four canonical
/// spellings `-` / `rall` / `rlast` / `x<hh>`, and an `x<hh>` bitmask is legal ONLY when its
/// reduced-axis set has no shorter canonical spelling for the cell's rank. A reader MUST
/// reject an `x<hh>` that re-encodes the empty set (`-`), the all-axes set (`rall`), or the
/// lone innermost axis (`rlast`) with a typed decline (`NonCanonicalReduceField`) — kept
/// DISTINCT from an out-of-range bit (index ≥ rank), which is `BadReduceField` (§6.7-0009).
///
/// These are the paired flips for `validate_reduce_mask`: delete that guard and every `Err`
/// below becomes `Ok` (a sentinel-overloading token would round-trip silently). The five
/// declining tokens are the exact literals published as decline vectors in
/// `structure_key_vectors.json`, so this test and the artifact move together.
#[test]
fn test_classify_noncanonical_reduce_mask_declines() {
    // all-axes at rank 2 (bits 0,1) — MUST be rall, never x03.
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|x03"),
        Err(KeyDecline::NonCanonicalReduceField)
    );
    // lone innermost axis at rank 2 (bit 1) — MUST be rlast, never x02.
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|x02"),
        Err(KeyDecline::NonCanonicalReduceField)
    );
    // empty set — that is not a reduction; MUST be `-`, never x00.
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|x00"),
        Err(KeyDecline::NonCanonicalReduceField)
    );
    // rank 1: all-axes and the lone trailing axis coincide (both are bit 0). §6.6-0009 ties
    // that to `rall`, so `x01` at rank 1 is non-canonical — the tie has one winner, not x<hh>.
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r1|co/00/v1/d8/f;co/00/v1/da/f|x01"),
        Err(KeyDecline::NonCanonicalReduceField)
    );
    // out-of-range: bit 2 set at rank 2 (only axes 0,1 exist). A DIFFERENT producer bug from a
    // sentinel overload — kept discriminable so a consumer can tell "wrong spelling of a real
    // set" from "a set that cannot exist for this rank." §6.7-0009.
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|x04"),
        Err(KeyDecline::BadReduceField)
    );
    // rank 0 (scalar): no axis to reduce, so ONLY `-` is valid. The `rall`/`rlast` SENTINELS
    // are not `Reduce::Subset`, so they bypass `validate_reduce_mask` and are caught by a
    // separate rank-0 guard as the same sentinel overload x00 gets. THESE ARE THAT GUARD'S OWN
    // FLIPS — every Subset-arm assertion above stays green if the rank-0 guard is deleted, so
    // the guard would be untested without them. (`rall` reduces all of zero axes = the empty
    // reduction; `rlast` names a trailing axis that does not exist — both are what `-` owns.)
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r0|co/00/v1/d8/f;co/00/v1/da/f|rall"),
        Err(KeyDecline::NonCanonicalReduceField)
    );
    assert_eq!(
        from_token("sk4|red|f32|cuda:sm89|ix32|warp|r0|co/00/v1/d8/f;co/00/v1/da/f|rlast"),
        Err(KeyDecline::NonCanonicalReduceField)
    );
    // boundary: `-` is the ONE valid rank-0 reduce (built at runtime, not a source literal).
    let ok_r0 =
        "sk4|red|f32|cuda:sm89|ix32|warp|r0|co/00/v1/d8/f;co/00/v1/da/f|rall".replace("|rall", "|-");
    assert!(from_token(&ok_r0).is_ok(), "rank-0 `-` (no reduction) must parse");
    // the two declines are genuinely distinct enum values (not one Err reached two ways).
    assert_ne!(KeyDecline::NonCanonicalReduceField, KeyDecline::BadReduceField);
    // BOUNDARY (guard discriminates, does not over-reject): a genuine partial reduction that
    // has NO shorter canonical spelling survives. At rank 2, reducing axis 0 only (bit 0) is
    // neither all-axes (x03), the lone trailing axis (x02), nor empty — so `x01` is the correct
    // accepted x<hh> spelling. Built at runtime so it is not a source literal the artifact-
    // superset scanner would require as a vector.
    let survives =
        "sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|x03".replace("|x03", "|x01");
    assert!(
        from_token(&survives).is_ok(),
        "a partial reduction with no shorter canonical spelling must parse as x<hh>"
    );
}

/// KISS-CLASSIFY-7.1-0002 (`test_classify_unclaimed_input_typed_decline`): every
/// unrecognized or out-of-range input yields a typed decline (an Err), never a
/// panic / OOB read. Each arm catches a reader that would silently ACCEPT (return
/// Ok for) that malformed input. (The "collapsed reduction" arm of §7.1-0002 is a
/// derivation-side concern, not a from_token token defect, and is out of scope for
/// the token reader tested here.)
#[test]
fn test_classify_unclaimed_input_typed_decline() {
    // unknown dtype
    assert_eq!(
        from_token(&A_GOLDEN.replacen("|f32|", "|f99|", 1)),
        Err(KeyDecline::UnknownDtype)
    );
    // unknown op-family
    assert_eq!(
        from_token(&A_GOLDEN.replacen("|bin|", "|zzz|", 1)),
        Err(KeyDecline::UnknownOpFamily)
    );
    // over-MAX_RANK rank: r9 with MAX_RANK == 8
    assert_eq!(MAX_RANK, 8);
    assert_eq!(
        from_token(&A_GOLDEN.replacen("|r2|", "|r9|", 1)),
        Err(KeyDecline::BadRank)
    );
    // boundary: r8 == MAX_RANK is accepted (the bound is not off-by-one).
    assert!(from_token(&A_GOLDEN.replacen("|r2|", "|r8|", 1)).is_ok());
    // over-MAX_OPERANDS count: 9 sub-keys
    let nine = std::iter::repeat("co/00/v4/d16/f").take(9).collect::<Vec<_>>().join(";");
    let over_ops = A_GOLDEN.replacen(
        "co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f",
        &nine,
        1,
    );
    assert_eq!(from_token(&over_ops), Err(KeyDecline::TooManyOperands { got: 9 }));
    // malformed operand sub-key (unknown contig code)
    assert_eq!(
        from_token(&A_GOLDEN.replacen("co/00/v4/d16/f", "zz/00/v4/d16/f", 1)),
        Err(KeyDecline::BadOperandSubKey)
    );
    // over-length token (> MAX_STRUCTURE_KEY_LEN), rejected without OOB
    let too_long = "x".repeat(MAX_STRUCTURE_KEY_LEN + 1);
    assert_eq!(
        from_token(&too_long),
        Err(KeyDecline::TokenLengthOutOfBound { len: 4097 })
    );
    // malformed target (missing capability set)
    assert_eq!(
        from_token(&A_GOLDEN.replacen("cuda:sm89", "cuda", 1)),
        Err(KeyDecline::BadTargetGrammar)
    );
    // wholly malformed / garbage token — a typed decline, not a panic.
    assert!(from_token("not-a-structure-key").is_err());
    // the canonical golden still parses (the battery is not over-broad).
    assert!(from_token(A_GOLDEN).is_ok());
}

/// KISS-CLASSIFY-6.3-0011 (`test_classify_axis_ordering_convention`): the
/// innermost axis is the HIGHEST active index (axis `rank-1`), not axis 0. Teeth:
/// a column-major reader that treats axis 0 as innermost picks the wrong extent
/// for the divisibility bucket. Verified against Appendix A `[4,1]` -> `da`
/// (inner axis 1 extent 1), which a column-major reader would mis-bucket `d4`
/// (axis 0 extent 4).
#[test]
fn test_classify_axis_ordering_convention() {
    // the innermost active axis index is rank-1 (row-major / outermost-first).
    assert_eq!(innermost_active_axis(0), None); // rank-0 scalar: no axis
    assert_eq!(innermost_active_axis(1), Some(0));
    assert_eq!(innermost_active_axis(2), Some(1)); // NOT Some(0): catches column-major
    assert_eq!(innermost_active_axis(4), Some(3));

    // bound to a real derivation: the div bucket reads extents[rank-1], the LAST
    // extent. For [8,5] the inner axis is extent 5 (odd) -> `da`; a column-major
    // reader taking extents[0]=8 would wrongly bucket `d8`.
    assert_eq!(derive_div_bucket_of(&[8, 5]), DivBucket::Da);
    assert_eq!(derive_div_bucket_of(&[5, 8]), DivBucket::D8);
    // Appendix A `[4,1] -> [4,1]` output: inner axis 1 extent 1 -> `da`
    // (not axis 0 extent 4 -> `d4`).
    assert_eq!(derive_div_bucket_of(&[4, 1]), DivBucket::Da);
}

/// KISS-CLASSIFY-6.5-0002 (`test_classify_layout_tag_derivation`): the 4-step
/// layout algorithm over `|stride|`, innermost-first. Teeth: (a) uses `|stride|`
/// so a reversed view is `co` (a signed-stride impl would call it `ic`/`st`); (b)
/// broadcast requires extent > 1 with stride 0 (an "any stride 0" impl mis-flags a
/// unit axis); (c) the product is innermost-first (a column-major impl mis-classes
/// a transpose). Verified against Appendix A: `[128,256]/[256,1]` -> `co`,
/// `[4,1]/[1,1]` -> `co`, `[1,1]/[1,1]` -> `co`.
#[test]
fn test_classify_layout_tag_derivation() {
    // contiguous multi-axis (Appendix A binary-add operand): P=1 at inner (|1|==1),
    // then |256|==256. -> co.
    assert_eq!(derive_layout_tag(&[128, 256], &[256, 1]), Contig::Contiguous);
    // empty product: every active axis extent 0 or 1 -> contiguous (Appendix A [1,1]).
    assert_eq!(derive_layout_tag(&[1, 1], &[1, 1]), Contig::Contiguous);
    // Appendix A output [4,1]/[1,1]: axis 1 (extent 1) dropped, axis 0 (4,|1|==P=1) -> co.
    assert_eq!(derive_layout_tag(&[4, 1], &[1, 1]), Contig::Contiguous);

    // real broadcast: axis 0 extent 128 (>1) has stride 0 -> br.
    assert_eq!(derive_layout_tag(&[128, 256], &[0, 1]), Contig::Broadcast);
    // NOT broadcast: the stride-0 axis has extent 1 (<=1); it is excluded, the
    // remaining axis is contiguous -> co. Catches an "any stride == 0" impl.
    assert_eq!(derive_layout_tag(&[1, 256], &[0, 1]), Contig::Contiguous);

    // reversed inner axis (stride -1): |stride| is used, so still contiguous (the
    // reversal lives only in the flipped flag). Catches a signed-stride impl.
    assert_eq!(derive_layout_tag(&[128, 256], &[256, -1]), Contig::Contiguous);

    // inner-contiguous: padded rows (outer |16| != product 8) but inner |1|==1.
    assert_eq!(derive_layout_tag(&[4, 8], &[16, 1]), Contig::InnerContiguous);

    // strided / transposed: inner axis |stride|=4 != 1 and product fails -> st.
    assert_eq!(derive_layout_tag(&[4, 8], &[1, 4]), Contig::Strided);
    // column-major-contiguous [2,3]/[1,2]: under the innermost-first product the
    // inner axis has |2| != P=1 and |stride|!=1 -> strided. A column-major
    // (outermost-first) product impl would wrongly call it contiguous.
    assert_eq!(derive_layout_tag(&[2, 3], &[1, 2]), Contig::Strided);
}

/// KISS-CLASSIFY-6.5-0011 (`test_classify_index_width_offset`): max touched offset
/// = max over operands of sum of `|stride|*(extent-1)` over non-broadcast axes;
/// `< 2^31` is `ix32`. Teeth: the `(extent-1)` off-by-one, and the exclusive 2^31
/// boundary. Verified against Appendix A binary-add operand
/// `256*127 + 1*255 = 32767`.
#[test]
fn test_classify_index_width_offset() {
    // Appendix A: [128,256]/[256,1] -> 256*127 + 1*255 = 32512 + 255 = 32767.
    // An (extent) impl gives 256*128 + 1*256 = 33024 -> caught by the exact value.
    assert_eq!(operand_touched_offset(&[128, 256], &[256, 1]), 32767);

    // a broadcast axis (stride 0) contributes 0: only the inner axis counts.
    assert_eq!(operand_touched_offset(&[128, 256], &[0, 1]), 255);

    // |stride| is used: a reversed (stride -1) axis contributes +255, not -255.
    assert_eq!(operand_touched_offset(&[256], &[-1]), 255);

    // boundary teeth: extent 2^31, stride 1 -> offset 2^31 - 1 = 2147483647 < 2^31
    // -> ix32. An (extent) off-by-one gives exactly 2^31 -> ix64.
    let big = 1i64 << 31; // 2147483648
    assert_eq!(operand_touched_offset(&[big], &[1]), big - 1);
    assert_eq!(derive_index_width(&[(&[big], &[1])]), "ix32");
    // one element further: extent 2^31 + 1 -> offset exactly 2^31 -> ix64
    // (the boundary is exclusive).
    assert_eq!(derive_index_width(&[(&[big + 1], &[1])]), "ix64");

    // max is taken over all operands: the larger rhs offset (16777215) decides,
    // still < 2^31 -> ix32.
    let e0: &[i64] = &[128, 256];
    let s0: &[i64] = &[256, 1];
    let e1: &[i64] = &[4096, 4096];
    let s1: &[i64] = &[4096, 1];
    assert_eq!(operand_touched_offset(e1, s1), 16_777_215);
    assert_eq!(derive_index_width(&[(e0, s0), (e1, s1)]), "ix32");
}

/// KISS-CLASSIFY-6.5-0012 (`test_classify_div_bucket_derivation`): the div bucket
/// from the innermost extent E, with the `E >= N` guard. Teeth: the E=0 trap —
/// `0 % 16 == 0`, so an impl that drops the `E >= 16` guard mis-buckets a
/// zero-extent axis as `d16` instead of `da`. Verified against Appendix A:
/// inner extent 256 -> d16, 8 -> d8, 5 (odd) -> da, 1 -> da.
#[test]
fn test_classify_div_bucket_derivation() {
    // the E=0 trap: guard makes it `da`, a guardless `E % 16 == 0` impl says d16.
    assert_eq!(derive_div_bucket(0), DivBucket::Da);

    // d16: divisible by 16 (Appendix A inner 256).
    assert_eq!(derive_div_bucket(16), DivBucket::D16);
    assert_eq!(derive_div_bucket(256), DivBucket::D16);
    assert_eq!(derive_div_bucket(48), DivBucket::D16); // 48 % 16 == 0
    // d8: divisible by 8 not 16 (Appendix A inner 8).
    assert_eq!(derive_div_bucket(8), DivBucket::D8);
    assert_eq!(derive_div_bucket(24), DivBucket::D8); // 24 % 16 == 8, 24 % 8 == 0
    // d4: divisible by 4 not 8.
    assert_eq!(derive_div_bucket(4), DivBucket::D4);
    assert_eq!(derive_div_bucket(12), DivBucket::D4); // 12 % 8 == 4, 12 % 4 == 0
    // d2: divisible by 2 not 4.
    assert_eq!(derive_div_bucket(2), DivBucket::D2);
    assert_eq!(derive_div_bucket(6), DivBucket::D2); // 6 % 4 == 2, 6 % 2 == 0
    // da: odd, unit, zero (Appendix A inner 5 odd, output inner 1).
    assert_eq!(derive_div_bucket(5), DivBucket::Da);
    assert_eq!(derive_div_bucket(3), DivBucket::Da);
    assert_eq!(derive_div_bucket(1), DivBucket::Da);
}
