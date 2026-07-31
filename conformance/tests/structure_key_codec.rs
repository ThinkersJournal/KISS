//! KISS-Conform teeth for the KISS-Classify `structure_key` **codec** surface
//! (`conformance/src/structure_key.rs`). Sibling of `structure_key_golden.rs`:
//! the golden file pins Appendix-A byte vectors; this file binds a set of
//! previously-unbacked KISS-CLASSIFY clauses to *behavioral* properties of the
//! real `to_token` / `from_token` / `derive_*` functions.
//!
//! Every test is named EXACTLY per the spec §9.1 traceability row for its clause
//! and cites its full `KISS-CLASSIFY-<sec>-<nnnn>` id in the leading comment and
//! in an assert message, so `kiss_trace` binds it by both the forward name-match
//! and the reverse citation. CITATION DISCIPLINE: a test body cites ONLY its own
//! clause id — no other clause id (of any sub-standard) appears in a full
//! `KISS-…-nnnn` form — so `--update-ledger` drops exactly these clause ids.
//!
//! TEETH: each test asserts a property that FAILS on a concrete, plausible codec
//! drift (removed bound-check, unsigned-stride reinterpret, case-folding parser,
//! uppercase/variable-width hex, fixed-width sub-key padding, order-insensitive
//! field parsing, prefix/case target match, discriminant-keyed codec, left-axis
//! alignment). No self-comparisons, no local codec re-implementation, no
//! substring greps of spec prose.

use kiss_conformance::structure_key::*;

// ---- shared builders (this file's own; a sibling test file is a separate crate) ----

fn op(contig: Contig, mask: u8, vec: VecWidth, div: DivBucket, flipped: bool) -> OperandSubKey {
    OperandSubKey { contig, bcast_mask: mask, vec, div, flipped }
}

/// A plain contiguous v4/d16 operand — the reusable filler sub-key.
fn co() -> OperandSubKey { op(Contig::Contiguous, 0x00, VecWidth::V4, DivBucket::D16, false) }

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
    }
}

/// A canonical, well-formed non-contraction token used as the mutation base for
/// the `from_token` teeth. Guarded valid by an `is_ok` assert in each test that
/// mutates it.
const BASE: &str =
    "sk3|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-";

// ============================================================================
// KISS-CLASSIFY-6.3-0001 — operand rank bounds
// ============================================================================

/// KISS-CLASSIFY-6.3-0001: `rank` MUST be in `0 ..= MAX_RANK`; a reader rejects a
/// greater rank with a typed decline, never a panic.
#[test]
fn test_classify_operand_rank_bounds() {
    // A constant drift on MAX_RANK would silently move the boundary.
    assert_eq!(MAX_RANK, 8, "KISS-CLASSIFY-6.3-0001: MAX_RANK is not 8");
    assert!(from_token(BASE).is_ok(), "KISS-CLASSIFY-6.3-0001: base token invalid");

    // every in-range rank r0..=r8 parses (the bound is not over-tight).
    for r in 0..=MAX_RANK {
        let t = BASE.replacen("|r2|", &format!("|r{r}|"), 1);
        assert!(
            from_token(&t).is_ok(),
            "KISS-CLASSIFY-6.3-0001: in-range rank r{r} was declined"
        );
    }
    // r9 — one past MAX_RANK — is a typed BadRank decline. A reader that dropped the
    // `rank > MAX_RANK` guard would accept it; this assert then fails.
    assert_eq!(
        from_token(&BASE.replacen("|r2|", "|r9|", 1)),
        Err(KeyDecline::BadRank),
        "KISS-CLASSIFY-6.3-0001: r9 (> MAX_RANK) not declined"
    );
    // far out of range stays a decline (never a panic / OOB).
    assert_eq!(
        from_token(&BASE.replacen("|r2|", "|r100|", 1)),
        Err(KeyDecline::BadRank),
        "KISS-CLASSIFY-6.3-0001: r100 not declined"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.3-0003 — strides are signed element strides
// ============================================================================

/// KISS-CLASSIFY-6.3-0003: `strides` are SIGNED element strides — a negative value
/// is a reversed axis and MUST NOT be reinterpreted as a huge unsigned value. Both
/// the touched-offset and the layout derivation read `|stride|`.
#[test]
fn test_classify_strides_are_signed_elements() {
    // a reversed axis contributes +|stride|*(extent-1) = 1*255 = 255 — NOT ~2^64.
    // An unsigned reinterpret of stride -1 (0xffff…ffff) would give an astronomically
    // large offset here.
    assert_eq!(
        operand_touched_offset(&[256], &[-1]),
        255,
        "KISS-CLASSIFY-6.3-0003: reversed stride reinterpreted as unsigned"
    );
    // magnitude scales with |stride|: stride -2 over extent 256 -> 2*255 = 510.
    assert_eq!(operand_touched_offset(&[256], &[-2]), 510, "KISS-CLASSIFY-6.3-0003");
    // sign flips direction, not magnitude: forward and reversed axes of equal |stride|
    // yield equal offsets.
    assert_eq!(
        operand_touched_offset(&[256], &[1]),
        operand_touched_offset(&[256], &[-1]),
        "KISS-CLASSIFY-6.3-0003: |stride| not used for the offset"
    );
    // the layout derivation uses |stride|, so a fully reversed contiguous view stays
    // `contiguous` (the reversal lives only in the flipped flag, never the layout tag).
    // An unsigned reinterpret makes |stride| ~2^64 and mis-buckets it as `strided`.
    assert_eq!(
        derive_layout_tag(&[128, 256], &[256, -1]),
        Contig::Contiguous,
        "KISS-CLASSIFY-6.3-0003: reversed inner axis mis-bucketed"
    );
    // a reversed OUTER axis is likewise absorbed by |stride| -> contiguous.
    assert_eq!(
        derive_layout_tag(&[128, 256], &[-256, 1]),
        Contig::Contiguous,
        "KISS-CLASSIFY-6.3-0003: reversed outer axis mis-bucketed"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.1-0004 — dtype token spelling (exact, lowercase, no synonym)
// ============================================================================

/// KISS-CLASSIFY-6.1-0004: a dtype is keyed on its EXACT lowercase token — no
/// synonym, no alternate casing. The field-2 dtype parser rejects a miscased or
/// synonym spelling as `UnknownDtype`.
#[test]
fn test_classify_dtype_token_spelling() {
    assert!(from_token(BASE).is_ok(), "KISS-CLASSIFY-6.1-0004: base token invalid");
    // miscasings / synonyms a case-folding or alias-tolerant parser would wrongly
    // accept — each MUST decline as UnknownDtype.
    for bad in [
        "F32", "Float32", "float32", "fp32", "F16", "BF16", "Bf16", "BOOL", "Bool",
        "S8", "U8", "I32", "E4M3FN", "C32",
    ] {
        let t = BASE.replacen("|f32|", &format!("|{bad}|"), 1);
        assert_eq!(
            from_token(&t),
            Err(KeyDecline::UnknownDtype),
            "KISS-CLASSIFY-6.1-0004: parser accepted non-canonical dtype `{bad}`"
        );
    }
    // each of the exact lowercase spellings is recognized: active -> Ok, reserved ->
    // the distinct ReservedDtype (feeding each through the parser tests the parser's
    // behaviour, not a copy of the set).
    for dt in DTYPES {
        let t = BASE.replacen("|f32|", &format!("|{dt}|"), 1);
        if RESERVED_DTYPES.contains(&dt) {
            assert_eq!(
                from_token(&t),
                Err(KeyDecline::ReservedDtype),
                "KISS-CLASSIFY-6.1-0004: reserved spelling `{dt}` mis-declined"
            );
        } else {
            assert!(
                from_token(&t).is_ok(),
                "KISS-CLASSIFY-6.1-0004: exact spelling `{dt}` was rejected"
            );
        }
    }
}

// ============================================================================
// KISS-CLASSIFY-6.3-0006 — operand dtype in the closed set (contraction path)
// ============================================================================

/// KISS-CLASSIFY-6.3-0006: an operand's `dtype` MUST be exactly one of the closed
/// 22-token set. Exercised on the gem contraction precision group's operand-dtype
/// positions (weight / accumulator / output) — a code path distinct from the
/// field-2 primary dtype — each of which the reader validates against the closed set.
#[test]
fn test_classify_operand_dtype_in_set() {
    const GEM: &str = "sk3|gem|f32|cuda:sm89|ix32|grid|r2|\
co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st";
    assert!(from_token(GEM).is_ok(), "KISS-CLASSIFY-6.3-0006: all-active group must parse");

    // out-of-set token in the WEIGHT position — a reader skipping the closed-set check
    // would accept `f99`.
    assert_eq!(
        from_token(&GEM.replace("/f32/f32/f32/st", "/f99/f32/f32/st")),
        Err(KeyDecline::UnknownDtype),
        "KISS-CLASSIFY-6.3-0006: out-of-set weight dtype accepted"
    );
    // out-of-set in the ACCUMULATOR position.
    assert_eq!(
        from_token(&GEM.replace("/f32/f32/f32/st", "/f32/zz9/f32/st")),
        Err(KeyDecline::UnknownDtype),
        "KISS-CLASSIFY-6.3-0006: out-of-set accumulator dtype accepted"
    );
    // out-of-set in the OUTPUT position.
    assert_eq!(
        from_token(&GEM.replace("/f32/f32/f32/st", "/f32/f32/qqx/st")),
        Err(KeyDecline::UnknownDtype),
        "KISS-CLASSIFY-6.3-0006: out-of-set output dtype accepted"
    );
    // a spelling IN the closed vocabulary but reserved -> the distinct ReservedDtype,
    // proving the gate is against the vocabulary, not a looser "looks like a dtype".
    assert_eq!(
        from_token(&GEM.replace("/f32/f32/f32/st", "/f32/e4m3fnuz/f32/st")),
        Err(KeyDecline::ReservedDtype),
        "KISS-CLASSIFY-6.3-0006: reserved dtype not distinguished"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.6-0004 — structure_key field layout / order / version-first
// ============================================================================

/// KISS-CLASSIFY-6.6-0004: fields appear in exactly the pinned order; `version` is
/// field 0 and MUST equal 3. Position — not content-sniffing — decides each field's
/// role.
#[test]
fn test_classify_structure_key_field_layout() {
    assert!(from_token(BASE).is_ok(), "KISS-CLASSIFY-6.6-0004: base token invalid");
    // version is field 0 and is `sk3`, on both the wire and a freshly serialized key.
    assert_eq!(BASE.split('|').next(), Some("sk3"), "KISS-CLASSIFY-6.6-0004");
    let built = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co(), co(), co()], Reduce::None, None);
    assert!(
        built.to_token().starts_with("sk3|"),
        "KISS-CLASSIFY-6.6-0004: version is not the first field"
    );
    // version moved out of field 0 is rejected (must be first).
    assert_eq!(
        from_token("bin|sk3|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-"),
        Err(KeyDecline::BadVersionPrefix),
        "KISS-CLASSIFY-6.6-0004: version not required in field 0"
    );
    // swap op-family (field 1) and dtype (field 2): `f32` in the op-family slot is an
    // unknown op-family. An order-insensitive parser would instead recognize it as a
    // dtype and ACCEPT the swap — so this decline is the teeth.
    assert_eq!(
        from_token("sk3|f32|bin|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-"),
        Err(KeyDecline::UnknownOpFamily),
        "KISS-CLASSIFY-6.6-0004: op-family/dtype positions not enforced"
    );
    // swap index-width (field 4) and work-class (field 5): `ix32` in the work-class
    // slot fails WorkClass parsing.
    assert_eq!(
        from_token("sk3|bin|f32|cuda:sm89|grid|ix32|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-"),
        Err(KeyDecline::BadWorkClass),
        "KISS-CLASSIFY-6.6-0004: index-width/work-class positions not enforced"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.6-0006 — rank & operand count (exactly n_operands, no padding)
// ============================================================================

/// KISS-CLASSIFY-6.6-0006: the codec serializes EXACTLY `n_operands` `;`-joined
/// sub-keys — no padding to a fixed `MAX_OPERANDS` — so no unused slot is
/// observable; the count round-trips and is bounded by `MAX_OPERANDS`.
#[test]
fn test_classify_structure_key_rank_and_operand_count() {
    assert_eq!(MAX_OPERANDS, 8, "KISS-CLASSIFY-6.6-0006: MAX_OPERANDS is not 8");
    // n operands -> exactly (n-1) `;` separators in the operand field (field 7). A
    // fixed-width padding codec would emit MAX_OPERANDS slots regardless of n.
    for n in 1..=MAX_OPERANDS {
        let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co(); n], Reduce::None, None);
        let tok = k.to_token();
        let field7 = tok.split('|').nth(7).expect("operand field present");
        assert_eq!(
            field7.matches(';').count(),
            n - 1,
            "KISS-CLASSIFY-6.6-0006: {n} operands serialized the wrong sub-key count (padding?)"
        );
        // the count round-trips: no slot invented or dropped.
        assert_eq!(
            from_token(&tok).unwrap().operands.len(),
            n,
            "KISS-CLASSIFY-6.6-0006: operand count not preserved on round-trip"
        );
    }
    // > MAX_OPERANDS is a typed decline (9 sub-keys).
    let nine = std::iter::repeat("co/00/v4/d16/f").take(9).collect::<Vec<_>>().join(";");
    let t9 = BASE.replacen("co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f", &nine, 1);
    assert_eq!(
        from_token(&t9),
        Err(KeyDecline::TooManyOperands { got: 9 }),
        "KISS-CLASSIFY-6.6-0006: 9 operands not declined"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.6-0013 — mixed-rank right-alignment to the iteration frame
// ============================================================================

/// KISS-CLASSIFY-6.6-0013: lower-rank operands RIGHT-align to the iteration frame —
/// an operand of rank `r` occupies the innermost `r` frame axes and broadcasts
/// (extent 1) on every frame axis below that. The frame element count exposes the
/// alignment.
#[test]
fn test_classify_mixed_rank_axis_alignment() {
    // rank-2 [4,8] vs rank-1 [8]: [8] lands on the INNER axis (axis 1) ->
    //   axis0 = max(4,1) = 4, axis1 = max(8,8) = 8  =>  4*8 = 32.
    // A LEFT-aligning impl lands [8] on axis 0 -> max(8,1)*max(8,1)... = 64.
    assert_eq!(
        work_class_element_count(&[&[4, 8], &[8]]),
        32,
        "KISS-CLASSIFY-6.6-0013: rank-1 operand not right-aligned (32 vs left-align 64)"
    );
    // three-operand mixed-rank frame [2,3,4], [4], [3,4]:
    //   axis0 = max(2,1,1) = 2 ; axis1 = max(3,1,3) = 3 ; axis2 = max(4,4,4) = 4
    //   => 2*3*4 = 24.  A left-aligning impl gives 4*4*4 = 64.
    assert_eq!(
        work_class_element_count(&[&[2, 3, 4], &[4], &[3, 4]]),
        24,
        "KISS-CLASSIFY-6.6-0013: multi-operand right-alignment wrong (24 vs left-align 64)"
    );
    // a rank-0 (scalar) operand broadcasts on every frame axis (extent 1 everywhere).
    assert_eq!(
        work_class_element_count(&[&[4, 8], &[]]),
        32,
        "KISS-CLASSIFY-6.6-0013: scalar operand not broadcast across the frame"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.7-0007 — token codec is spelling-keyed, not discriminant-keyed
// ============================================================================

/// KISS-CLASSIFY-6.7-0007: the codec is spelling-keyed, not discriminant-keyed —
/// adding a dtype or op-family code MUST NOT shift the bytes of any pre-existing
/// field and MUST NOT bump the schema version.
#[test]
fn test_classify_token_codec_additive() {
    assert_eq!(SCHEMA_VERSION, 3, "KISS-CLASSIFY-6.7-0007: schema version is not 3");
    // the reference token whose field-2 dtype we vary; every OTHER field is invariant.
    let base = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co(), co(), co()], Reduce::None, None);
    let base_tok = base.to_token();
    let base_fields: Vec<&str> = base_tok.split('|').collect();

    for dt in DTYPES {
        let mut k = base.clone();
        k.dtype = dt.to_string();
        let tok = k.to_token();
        let fields: Vec<&str> = tok.split('|').collect();
        assert_eq!(fields.len(), base_fields.len(), "KISS-CLASSIFY-6.7-0007: dtype `{dt}` changed field count");
        // field 0 stays the un-bumped version `sk3` (a set-size-keyed version bump
        // would emit sk4/… here).
        assert_eq!(fields[0], "sk3", "KISS-CLASSIFY-6.7-0007: dtype `{dt}` bumped the version");
        // field 2 carries the dtype SPELLING verbatim (a discriminant codec would emit
        // a numeric index instead).
        assert_eq!(fields[2], dt, "KISS-CLASSIFY-6.7-0007: dtype `{dt}` not stored by spelling");
        // every OTHER field is byte-identical to the base — no positional drift.
        for i in 0..base_fields.len() {
            if i == 2 {
                continue;
            }
            assert_eq!(fields[i], base_fields[i], "KISS-CLASSIFY-6.7-0007: dtype `{dt}` shifted field {i}");
        }
    }
    // the same position-stability holds for op-family codes (non-gem, so the presence
    // of the contraction field is not in play here).
    for fam in OP_FAMILIES {
        if fam == "gem" {
            continue;
        }
        let mut k = base.clone();
        k.op_family = fam.to_string();
        let tok = k.to_token();
        let fields: Vec<&str> = tok.split('|').collect();
        assert_eq!(fields[0], "sk3", "KISS-CLASSIFY-6.7-0007: op-family `{fam}` bumped the version");
        assert_eq!(fields[1], fam, "KISS-CLASSIFY-6.7-0007: op-family `{fam}` not stored by spelling");
        for i in 0..base_fields.len() {
            if i == 1 {
                continue;
            }
            assert_eq!(fields[i], base_fields[i], "KISS-CLASSIFY-6.7-0007: op-family `{fam}` shifted field {i}");
        }
    }
}

// ============================================================================
// KISS-CLASSIFY-6.7-0008 — to_token / from_token round-trip byte-identically
// ============================================================================

/// KISS-CLASSIFY-6.7-0008: `to_token`/`from_token` round-trip byte-identically for
/// every well-formed key — parsing a serialized token reproduces the key, and
/// re-serializing reproduces the token, byte-for-byte. A single-field asymmetry (a
/// flip `f`/`r`, a present/absent batch coordinate, a reduce variant) breaks it.
#[test]
fn test_classify_token_roundtrip() {
    let mut battery: Vec<StructureKey> = Vec::new();
    // non-gem base, no reduce.
    battery.push(key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co(), co()], Reduce::None, None));
    // unary f16 v8 on a different target.
    battery.push(key(
        "une", "f16", "rocm:gfx942", WorkClass::Block, 2,
        vec![
            op(Contig::Contiguous, 0x00, VecWidth::V8, DivBucket::D16, false),
            op(Contig::Contiguous, 0x00, VecWidth::V8, DivBucket::D16, false),
        ],
        Reduce::None, None,
    ));
    // every reduce variant on a `red` cell (the None/All/Trailing/Subset spellings).
    for red in [Reduce::All, Reduce::Trailing, Reduce::Subset(0x0a), Reduce::None] {
        battery.push(key(
            "red", "f32", "cuda:sm89", WorkClass::Warp, 3,
            vec![
                op(Contig::Contiguous, 0x00, VecWidth::V1, DivBucket::D8, false),
                op(Contig::Contiguous, 0x00, VecWidth::V1, DivBucket::Da, false),
            ],
            red, None,
        ));
    }
    // varied operand sub-keys: ic/st/br contig, v1..v8, d16..da, a flipped operand, a
    // non-zero broadcast mask. The target is an arbitrary well-formed token —
    // capability semantics are irrelevant to a codec round-trip.
    battery.push(key(
        "bin", "s8", "vulkan:spirv1.6", WorkClass::Grid, 4,
        vec![
            op(Contig::Contiguous, 0x00, VecWidth::V4, DivBucket::D16, false),
            op(Contig::InnerContiguous, 0x00, VecWidth::V2, DivBucket::D8, true),
            op(Contig::Strided, 0x00, VecWidth::V1, DivBucket::D4, false),
            op(Contig::Broadcast, 0x0b, VecWidth::V1, DivBucket::Da, false),
        ],
        Reduce::None, None,
    ));
    // gem WITHOUT the conditional batch coordinate.
    battery.push(key(
        "gem", "f32", "cuda:sm90", WorkClass::Grid, 2, vec![co(), co(), co()], Reduce::None,
        Some(Contraction {
            m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16,
            batch: None, wdt: "f32".to_string(), acc: "f32".to_string(), out: "f32".to_string(),
            mp: MathPrecision::Stable,
        }),
    ));
    // gem WITH the batch coordinate + a mixed precision group + reduced-mantissa mp.
    battery.push(key(
        "gem", "e4m3fn", "cuda:sm90a", WorkClass::Grid, 2, vec![co(), co(), co()], Reduce::None,
        Some(Contraction {
            m: SizeClass::Medium, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D8,
            batch: Some(SizeClass::Medium), wdt: "e5m2".to_string(), acc: "f32".to_string(),
            out: "f16".to_string(), mp: MathPrecision::ReducedMantissa,
        }),
    ));

    for k in &battery {
        let tok = k.to_token();
        let parsed = from_token(&tok)
            .unwrap_or_else(|e| panic!("KISS-CLASSIFY-6.7-0008: a valid key was declined: {e:?} ({tok})"));
        // parsing reproduces the key ...
        assert_eq!(&parsed, k, "KISS-CLASSIFY-6.7-0008: parsed key differs ({tok})");
        // ... and re-serializing is byte-identical (the load-bearing wire equality).
        assert_eq!(parsed.to_token(), tok, "KISS-CLASSIFY-6.7-0008: re-serialization not byte-identical");
        // reverse direction over the raw wire form: to_token(from_token(tok)) == tok.
        assert_eq!(from_token(&tok).unwrap().to_token(), tok, "KISS-CLASSIFY-6.7-0008: wire round-trip drifted");
    }
}

// ============================================================================
// KISS-CLASSIFY-6.7-0010 — hex masks are lowercase, zero-padded to two digits
// ============================================================================

/// KISS-CLASSIFY-6.7-0010: every hex mask — the per-operand broadcast mask and the
/// reduce `x<hh>` keepdim mask — is LOWERCASE hex, zero-padded to EXACTLY two
/// digits; a reader rejects uppercase or variable-width hex in both positions.
#[test]
fn test_classify_mask_hex_lowercase() {
    // --- producer side: to_token emits `hh` (lowercase, 2 digits) in both positions ---
    // broadcast mask 0x0a -> `0a` (a `{:X}` producer -> `0A`; a width-less `{:x}` -> `a`).
    for (m, hex) in [(0x0au8, "0a"), (0x05u8, "05"), (0xffu8, "ff"), (0x00u8, "00")] {
        let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2,
            vec![op(Contig::Broadcast, m, VecWidth::V1, DivBucket::D16, false)], Reduce::None, None);
        let tok = k.to_token();
        let want = format!("br/{hex}/v1/d16/f");
        assert_eq!(
            tok.split('|').nth(7),
            Some(want.as_str()),
            "KISS-CLASSIFY-6.7-0010: broadcast mask 0x{m:02x} not serialized `{hex}`"
        );
    }
    // reduce keepdim mask 0x0a -> `x0a` on a red cell.
    let r = key("red", "f32", "cuda:sm89", WorkClass::Warp, 4,
        vec![op(Contig::Contiguous, 0x00, VecWidth::V1, DivBucket::Da, false)], Reduce::Subset(0x0a), None);
    assert_eq!(
        r.to_token().split('|').nth(8),
        Some("x0a"),
        "KISS-CLASSIFY-6.7-0010: reduce mask not serialized `x` + lowercase 2-digit"
    );

    // --- consumer side: reject uppercase / variable-width in the BROADCAST position ---
    let bcast_base = "sk3|bin|f32|cuda:sm89|ix32|grid|r2|co/0a/v4/d16/f|-";
    assert!(from_token(bcast_base).is_ok(), "KISS-CLASSIFY-6.7-0010: valid lowercase mask rejected");
    for bad in ["co/FF/v4/d16/f", "co/Aa/v4/d16/f", "co/0/v4/d16/f", "co/100/v4/d16/f", "co/a/v4/d16/f"] {
        let t = bcast_base.replacen("co/0a/v4/d16/f", bad, 1);
        assert_eq!(
            from_token(&t),
            Err(KeyDecline::UppercaseOrWidthHex),
            "KISS-CLASSIFY-6.7-0010: broadcast mask `{bad}` accepted"
        );
    }
    // --- consumer side: reject uppercase / variable-width in the REDUCE position ---
    let red_base = "sk3|red|f32|cuda:sm89|ix32|warp|r4|co/00/v1/da/f|x0a";
    assert!(from_token(red_base).is_ok(), "KISS-CLASSIFY-6.7-0010: valid reduce mask rejected");
    for bad in ["xFF", "xAa", "x0", "x100", "xa"] {
        let t = red_base.replacen("x0a", bad, 1);
        assert_eq!(
            from_token(&t),
            Err(KeyDecline::BadReduceField),
            "KISS-CLASSIFY-6.7-0010: reduce mask `{bad}` accepted"
        );
    }
}

// ============================================================================
// KISS-CLASSIFY-6.8-0002 — target_capability matching is byte-exact
// ============================================================================

/// KISS-CLASSIFY-6.8-0002: two `target_capability` tokens match iff byte-exact on
/// the full string — no ordering, subset, prefix, case, or feature-implication
/// logic. The codec carries the target VERBATIM into field 3, so byte-exact token
/// matching reduces to byte-exact target matching.
#[test]
fn test_classify_target_byte_exact_match() {
    let mk = |tgt: &str| {
        key("bin", "f32", tgt, WorkClass::Grid, 2, vec![co(), co()], Reduce::None, None).to_token()
    };
    // the target is embedded byte-for-byte (no lowercasing / truncation / normalization),
    // and survives parse unchanged.
    //
    // Arbitrary well-formed tokens; capability semantics are irrelevant here — this
    // asserts byte-exact carriage (§6.8-0002), not vocabulary quality (§6.8-0004).
    for tgt in ["cuda:sm89", "cuda:sm90a", "Cuda:sm89", "cuda:sm89x", "rocm:gfx942", "vulkan:spirv1.6"] {
        let tok = mk(tgt);
        assert_eq!(
            tok.split('|').nth(3),
            Some(tgt),
            "KISS-CLASSIFY-6.8-0002: target `{tgt}` not carried verbatim into field 3"
        );
        assert_eq!(
            from_token(&tok).unwrap().target,
            tgt,
            "KISS-CLASSIFY-6.8-0002: target `{tgt}` normalized on parse"
        );
    }
    // near-miss targets a subset / prefix / case / feature matcher would collide with
    // `cuda:sm89` MUST produce byte-UNEQUAL tokens (i.e. no match).
    let exact = mk("cuda:sm89");
    for other in ["cuda:sm90a", "cuda:sm8", "Cuda:sm89", "cuda:sm89x", "cuda:SM89"] {
        assert_ne!(
            mk(other),
            exact,
            "KISS-CLASSIFY-6.8-0002: `{other}` collided with `cuda:sm89` (non-byte-exact match)"
        );
    }
    // byte-equal targets DO match: the same target text yields byte-identical tokens.
    assert_eq!(mk("cuda:sm89"), exact, "KISS-CLASSIFY-6.8-0002: equal targets did not match");
}

// KISS-CLASSIFY-6.9-0002 — test dropped per adversarial review; clause skipped-pending-a-real-surface (restored to UNBACKED.tsv as `untested`): no announce-seam carrier exists in this crate, so the ex-test's "opaque carry" was a std into_bytes()/from_utf8() identity on its own bytes (tautological) plus a re-parse already fully covered by test_classify_token_roundtrip — it had no unique mutation-provable teeth.
