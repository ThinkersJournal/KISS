//! KISS-Conform teeth for the KISS-Classify `structure_key` **field-grammar /
//! admissibility** surface (`conformance/src/structure_key.rs`). Sibling of
//! `structure_key_codec.rs` and `structure_key_golden.rs`: the golden file pins
//! Appendix-A byte vectors, the codec file binds the round-trip/decline behaviour,
//! and this file binds a further set of previously-unbacked KISS-CLASSIFY clauses
//! to *behavioral* properties of the real `to_token` / `from_token` and the
//! reference derivations (`derive_vec_width`, `work_class_element_count`,
//! `derive_work_class`, `derive_div_bucket`).
//!
//! Every test is named EXACTLY per the spec §9.1 traceability row for its clause
//! and cites its full `KISS-CLASSIFY-<sec>-<nnnn>` id in the leading doc-comment
//! and in an assert message, so `kiss_trace` binds it by both the forward
//! name-match and the reverse citation. CITATION DISCIPLINE: a test body cites
//! ONLY its own clause id — every cross-reference to another clause uses the
//! `§<sec>-<nnnn>` short form, which does not match the `KISS-…-nnnn` citation
//! grammar — so `--update-ledger` drops exactly these clause ids and no other.
//!
//! TEETH: each test asserts a property that FAILS on a concrete, plausible codec
//! drift (a power-of-two alignment floor, an extent leak, a normalized-away
//! near-miss field, a reordered/dropped sub-key field, a per-operand op-family or
//! dtype, a missing target charset gate). No self-comparisons, no local codec
//! re-implementation, no substring greps of spec prose.

use kiss_conformance::structure_key::*;

// ---- shared builders (this file's own; a sibling test file is a separate crate) ----

fn op(contig: Contig, mask: u8, vec: VecWidth, div: DivBucket, flipped: bool) -> OperandSubKey {
    OperandSubKey { contig, bcast_mask: mask, vec, div, flipped }
}

/// A plain contiguous v4/d16 operand — the reusable filler sub-key.
fn co() -> OperandSubKey {
    op(Contig::Contiguous, 0x00, VecWidth::V4, DivBucket::D16, false)
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

// ============================================================================
// KISS-CLASSIFY-6.3-0005 — alignment is a byte count gating vec width by modulo
// ============================================================================

/// KISS-CLASSIFY-6.3-0005: `alignment` is the base-pointer alignment in **bytes**
/// (an unsigned 32-bit value); `0` and non-power-of-two values are permitted, and
/// they gate vector width via an **exact-modulo** (divisor) test — NOT a
/// power-of-two floor — with `alignment = 0` forcing `v1`.
#[test]
fn test_classify_alignment_is_bytes() {
    // 24 bytes with a 4-byte dtype: a power-of-two FLOOR gate (`align >= L*bytes`)
    // would elect v4 (24 >= 16); the exact-modulo gate rejects v4 (24 % 16 = 8) and
    // v8, and elects v2 (24 % 8 == 0). This is the load-bearing divisor-vs-floor case.
    assert_eq!(
        derive_vec_width(1, 16, Some(4), 24, false),
        VecWidth::V2,
        "KISS-CLASSIFY-6.3-0005: alignment gate is a power-of-two floor, not exact-modulo (24B/4B)"
    );
    // 48 is NON-power-of-two yet 48 % 16 == 0 -> v4: proves the gate is a plain
    // divisor test, not "alignment must be a power of two" (which would drop to v1).
    assert_eq!(
        derive_vec_width(1, 16, Some(4), 48, false),
        VecWidth::V4,
        "KISS-CLASSIFY-6.3-0005: non-power-of-two alignment 48 (48%16==0) not honored"
    );
    // alignment = 0 (unspecified base-pointer alignment) forces v1.
    assert_eq!(
        derive_vec_width(1, 16, Some(4), 0, false),
        VecWidth::V1,
        "KISS-CLASSIFY-6.3-0005: alignment 0 did not force v1"
    );
    // baseline: an exact 16-byte alignment with a 4-byte dtype packs v4.
    assert_eq!(
        derive_vec_width(1, 16, Some(4), 16, false),
        VecWidth::V4,
        "KISS-CLASSIFY-6.3-0005: 16B alignment / 4B dtype should pack v4"
    );
    // the gate is in BYTES relative to the dtype's storage width: the SAME 16-byte
    // alignment packs v8 for a 2-byte dtype (16 % (8*2) == 0) but only v4 for a
    // 4-byte dtype (16 % (8*4) != 0) — an element-count reading of alignment could
    // not produce this dtype-width dependence.
    assert_eq!(
        derive_vec_width(1, 16, Some(2), 16, false),
        VecWidth::V8,
        "KISS-CLASSIFY-6.3-0005: 16B alignment / 2B dtype should pack v8 (bytes, not elements)"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.6-0003 — structure_key is extent-free (size classes only)
// ============================================================================

/// KISS-CLASSIFY-6.6-0003: a `structure_key` is **extent-free** — it coarsens
/// literal extents to size *classes* (`work_class`, divisibility bucket) and
/// carries no literal extent, so two cells whose extents differ but fall in the
/// same classes derive byte-identical keys.
#[test]
fn test_classify_structure_key_extent_free() {
    // the work-class element count coarsens a shape to a scalar product: [4,8] and
    // [2,16] are different tuples but the same count (32) and the same class (warp).
    assert_eq!(
        work_class_element_count(&[&[4, 8]]),
        work_class_element_count(&[&[2, 16]]),
        "KISS-CLASSIFY-6.6-0003: element count did not coarsen distinct extent tuples"
    );
    assert_eq!(
        work_class_element_count(&[&[4, 8]]),
        32,
        "KISS-CLASSIFY-6.6-0003: element count off"
    );
    assert_eq!(
        derive_work_class(&[&[4, 8]]),
        derive_work_class(&[&[2, 16]]),
        "KISS-CLASSIFY-6.6-0003: same-class extents derived different work classes"
    );
    // the divisibility bucket coarsens: 16 and 32 both bucket d16.
    assert_eq!(
        derive_div_bucket(16),
        derive_div_bucket(32),
        "KISS-CLASSIFY-6.6-0003: divisibility bucket carried the literal extent"
    );
    assert_eq!(derive_div_bucket(16), DivBucket::D16, "KISS-CLASSIFY-6.6-0003: bucket off");

    // build two full keys from DIFFERENT literal extent tuples that share a class:
    // [2,16] and [1,32] both derive work_class=warp, inner div=d16, rank=2. The
    // token carries no extent, so the two are byte-identical.
    let key_from = |ext: &[i64]| {
        key(
            "bin",
            "f32",
            "cuda:sm89",
            derive_work_class(&[ext]),
            ext.len() as u32,
            vec![op(Contig::Contiguous, 0x00, VecWidth::V1, derive_div_bucket_of(ext), false)],
            Reduce::None,
            None,
        )
    };
    assert_eq!(
        key_from(&[2, 16]).to_token(),
        key_from(&[1, 32]).to_token(),
        "KISS-CLASSIFY-6.6-0003: distinct same-class extents produced different tokens (extent leak)"
    );
    // coarsening, not blanket-ignoring: a tuple in a DIFFERENT class ([2,32] = 64 =
    // block) crosses a work-class boundary and MUST produce a different token.
    assert_ne!(
        key_from(&[2, 16]).to_token(),
        key_from(&[2, 32]).to_token(),
        "KISS-CLASSIFY-6.6-0003: a cross-class extent change did not change the key"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.6-0001 — structure_key is an admissibility predicate
// ============================================================================

/// KISS-CLASSIFY-6.6-0001: a `structure_key` is an **admissibility predicate** — a
/// kernel admits an invocation IFF the derived key byte-matches, so a key that
/// differs by any single field MUST NOT match. This binds the "MUST NOT admit a
/// differing key" half via a single-field near-miss battery (each variant differs
/// from the base in exactly one field and MUST yield a different token). It does
/// not model the front-end derivation itself.
#[test]
fn test_classify_structure_key_is_admissibility_predicate() {
    let base = key(
        "bin",
        "f32",
        "cuda:sm89",
        WorkClass::Grid,
        2,
        vec![op(Contig::Contiguous, 0x00, VecWidth::V4, DivBucket::D16, false), co()],
        Reduce::None,
        None,
    );
    let base_tok = base.to_token();
    // the base is a real, parseable key.
    assert!(
        from_token(&base_tok).is_ok(),
        "KISS-CLASSIFY-6.6-0001: base token is not valid"
    );

    // each closure returns a one-field-different variant of the base.
    let variants: Vec<(&str, StructureKey)> = vec![
        ("op_family", { let mut k = base.clone(); k.op_family = "une".into(); k }),
        ("dtype", { let mut k = base.clone(); k.dtype = "f16".into(); k }),
        ("target", { let mut k = base.clone(); k.target = "cuda:sm90a".into(); k }),
        ("index_width", { let mut k = base.clone(); k.index_width = "ix64".into(); k }),
        ("work_class", { let mut k = base.clone(); k.work_class = WorkClass::Block; k }),
        ("rank", { let mut k = base.clone(); k.rank = 3; k }),
        ("op0.contig", { let mut k = base.clone(); k.operands[0].contig = Contig::Strided; k }),
        ("op0.vec", { let mut k = base.clone(); k.operands[0].vec = VecWidth::V2; k }),
        ("op0.div", { let mut k = base.clone(); k.operands[0].div = DivBucket::D8; k }),
        ("op0.bcast_mask", { let mut k = base.clone(); k.operands[0].bcast_mask = 0x01; k }),
        ("op0.flipped", { let mut k = base.clone(); k.operands[0].flipped = true; k }),
    ];
    for (field, v) in &variants {
        assert_ne!(
            v.to_token(),
            base_tok,
            "KISS-CLASSIFY-6.6-0001: near-miss on `{field}` byte-matched the base (field normalized away)"
        );
    }

    // the reduce field is gated to `red` cells, so its near-miss uses a `red` base.
    let red_base = key(
        "red",
        "f32",
        "cuda:sm89",
        WorkClass::Warp,
        3,
        vec![co(), co()],
        Reduce::None,
        None,
    );
    let mut red_variant = red_base.clone();
    red_variant.reduce = Reduce::All;
    assert_ne!(
        red_variant.to_token(),
        red_base.to_token(),
        "KISS-CLASSIFY-6.6-0001: near-miss on `reduce` byte-matched the base"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.6-0007 — per-operand sub-key field set / order / round-trip
// ============================================================================

/// KISS-CLASSIFY-6.6-0007: each per-operand sub-key carries exactly
/// `{layout_tag, broadcast-axis mask, vector width, divisibility bucket, flipped}`
/// as five `/`-joined fields in that fixed order, and round-trips. (The clause's
/// "flipped iff any negative stride" DERIVATION is not modeled in `src` — there is
/// no `derive_flipped` — so it is out of scope here; only the field set, order, and
/// round-trip of the sub-key are bound.)
#[test]
fn test_classify_operand_sub_key_fields() {
    // a rich sub-key with a distinct value in every position.
    let sk = op(Contig::Strided, 0x0a, VecWidth::V2, DivBucket::D8, true);
    let field7 = |o: OperandSubKey| -> String {
        key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![o], Reduce::None, None)
            .to_token()
            .split('|')
            .nth(7)
            .expect("operand field present")
            .to_string()
    };
    let f = field7(sk);
    // exactly five fields => exactly four `/` separators (no field dropped or padded).
    assert_eq!(
        f.matches('/').count(),
        4,
        "KISS-CLASSIFY-6.6-0007: sub-key does not have exactly 5 `/`-joined fields ({f})"
    );
    let p: Vec<&str> = f.split('/').collect();
    assert_eq!(p[0], "st", "KISS-CLASSIFY-6.6-0007: position 0 is not the layout_tag");
    assert_eq!(p[1], "0a", "KISS-CLASSIFY-6.6-0007: position 1 is not the 2-hex bcast mask");
    assert_eq!(p[2], "v2", "KISS-CLASSIFY-6.6-0007: position 2 is not the vector width");
    assert_eq!(p[3], "d8", "KISS-CLASSIFY-6.6-0007: position 3 is not the divisibility bucket");
    assert_eq!(p[4], "r", "KISS-CLASSIFY-6.6-0007: position 4 is not the flipped flag");

    // each field is independent and stays in its own position (a reorder of, e.g.,
    // vec and div would move the changed code to the wrong slot).
    assert_eq!(
        field7(op(Contig::Contiguous, 0x0a, VecWidth::V2, DivBucket::D8, true)).split('/').nth(0),
        Some("co"),
        "KISS-CLASSIFY-6.6-0007: layout_tag not at position 0"
    );
    assert_eq!(
        field7(op(Contig::Strided, 0x00, VecWidth::V2, DivBucket::D8, true)).split('/').nth(1),
        Some("00"),
        "KISS-CLASSIFY-6.6-0007: bcast mask not at position 1"
    );
    assert_eq!(
        field7(op(Contig::Strided, 0x0a, VecWidth::V4, DivBucket::D8, true)).split('/').nth(2),
        Some("v4"),
        "KISS-CLASSIFY-6.6-0007: vector width not at position 2 (vec/div swap?)"
    );
    assert_eq!(
        field7(op(Contig::Strided, 0x0a, VecWidth::V2, DivBucket::D16, true)).split('/').nth(3),
        Some("d16"),
        "KISS-CLASSIFY-6.6-0007: divisibility bucket not at position 3 (vec/div swap?)"
    );
    assert_eq!(
        field7(op(Contig::Strided, 0x0a, VecWidth::V2, DivBucket::D8, false)).split('/').nth(4),
        Some("f"),
        "KISS-CLASSIFY-6.6-0007: flipped flag not at position 4"
    );

    // the sub-key round-trips: parsing reproduces the exact struct.
    let tok =
        key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![sk], Reduce::None, None).to_token();
    assert_eq!(
        from_token(&tok).unwrap().operands[0],
        sk,
        "KISS-CLASSIFY-6.6-0007: sub-key did not round-trip"
    );

    // arity is exactly five: a 4-field and a 6-field sub-key are typed declines.
    let base = "sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f|-";
    assert!(from_token(base).is_ok(), "KISS-CLASSIFY-6.6-0007: base token invalid");
    assert_eq!(
        from_token(&base.replacen("co/00/v4/d16/f", "co/00/v4/d16", 1)),
        Err(KeyDecline::BadOperandSubKey),
        "KISS-CLASSIFY-6.6-0007: a 4-field sub-key was accepted"
    );
    assert_eq!(
        from_token(&base.replacen("co/00/v4/d16/f", "co/00/v4/d16/f/x", 1)),
        Err(KeyDecline::BadOperandSubKey),
        "KISS-CLASSIFY-6.6-0007: a 6-field sub-key was accepted"
    );
}

// ============================================================================
// KISS-CLASSIFY-6.3-0008 — op_family is carried once at cell level, not per operand
// ============================================================================

/// KISS-CLASSIFY-6.3-0008: `op_family` is carried at the **cell** level (token
/// field 1), exactly once, never per raw operand — so it appears in exactly one
/// `|`-field and never inside any per-operand sub-key.
#[test]
fn test_classify_op_family_is_cell_level() {
    let operands = vec![
        co(),
        op(Contig::Broadcast, 0x0a, VecWidth::V1, DivBucket::Da, true),
        op(Contig::Strided, 0x00, VecWidth::V2, DivBucket::D8, false),
    ];
    let mut base_field7: Option<String> = None;
    for fam in OP_FAMILIES {
        // gem adds the trailing contraction field (field 9); the op-family-presence
        // rule for gem is a separate clause, so this cell-level test uses non-gem
        // families where field 7 is the last operand block.
        if fam == "gem" {
            continue;
        }
        let tok = key(fam, "f32", "cuda:sm89", WorkClass::Grid, 2, operands.clone(), Reduce::None, None)
            .to_token();
        let fields: Vec<&str> = tok.split('|').collect();
        // exactly one whole `|`-field is an op-family code, and it is field 1.
        let fam_positions: Vec<usize> =
            (0..fields.len()).filter(|&i| OP_FAMILIES.contains(&fields[i])).collect();
        assert_eq!(
            fam_positions,
            vec![1],
            "KISS-CLASSIFY-6.3-0008: op-family `{fam}` is not carried uniquely at field 1"
        );
        // no per-operand sub-key component (field 7) is an op-family code — an
        // op-family emitted per operand would inject its code into every sub-key.
        for sub in fields[7].split(';') {
            for comp in sub.split('/') {
                assert!(
                    !OP_FAMILIES.contains(&comp),
                    "KISS-CLASSIFY-6.3-0008: op-family code `{comp}` leaked into a sub-key (`{fam}`)"
                );
            }
        }
        // the operand block is byte-invariant as the cell-level op-family varies.
        match &base_field7 {
            None => base_field7 = Some(fields[7].to_string()),
            Some(b) => assert_eq!(
                fields[7], b,
                "KISS-CLASSIFY-6.3-0008: operand block drifted when op-family changed to `{fam}`"
            ),
        }
    }
}

// ============================================================================
// KISS-CLASSIFY-6.6-0015 — secondary-operand dtype is unkeyed outside a gem cell
// ============================================================================

/// KISS-CLASSIFY-6.6-0015: outside a dense-contraction (`gem`) cell a
/// `structure_key` keys only the primary (operand-0) dtype; per-operand sub-keys
/// carry **no** dtype. For a `gem` cell this is superseded — the sk4 contraction
/// field carries the weight/accumulator/output dtypes explicitly (still not via a
/// sub-key).
#[test]
fn test_classify_secondary_dtype_unkeyed() {
    // a non-gem, multi-operand cell. One operand has a bcast mask 0xb1, which
    // serializes as "b1" — that collides with the `b1` DTYPE token, so the dtype
    // scan MUST exclude the mask position (position 1) to avoid a false positive.
    let ops = vec![
        co(),
        op(Contig::Broadcast, 0xb1, VecWidth::V1, DivBucket::Da, false),
        op(Contig::Strided, 0x00, VecWidth::V2, DivBucket::D8, true),
    ];
    let tok = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 3, ops, Reduce::None, None).to_token();
    let fields: Vec<&str> = tok.split('|').collect();

    // count dtype tokens across the whole token, EXCLUDING the sub-key mask positions.
    let mut dtype_hits = 0usize;
    for (fi, field) in fields.iter().enumerate() {
        if fi == 7 {
            for sub in field.split(';') {
                for (pos, comp) in sub.split('/').enumerate() {
                    if pos == 1 {
                        continue; // mask position: 0xb1 -> "b1" is a mask, not a dtype
                    }
                    if DTYPES.contains(&comp) {
                        dtype_hits += 1;
                    }
                }
            }
        } else if DTYPES.contains(field) {
            dtype_hits += 1;
        }
    }
    assert_eq!(
        dtype_hits, 1,
        "KISS-CLASSIFY-6.6-0015: a non-gem token carries a dtype outside field 2 (per-operand dtype leaked)"
    );
    assert!(
        DTYPES.contains(&fields[2]),
        "KISS-CLASSIFY-6.6-0015: the single dtype is not the primary (field 2)"
    );

    // contrast: a gem cell DOES carry the secondary dtypes — but only in the
    // contraction field (field 9), never in a sub-key.
    let gem = key(
        "gem",
        "f32",
        "cuda:sm90",
        WorkClass::Grid,
        2,
        vec![co(), co(), co()],
        Reduce::None,
        Some(Contraction {
            m: SizeClass::Tiny,
            n: SizeClass::Large,
            k: SizeClass::Large,
            k_div: DivBucket::D16,
            batch: None,
            wdt: "f8e5m2".to_string(),
            acc: "f32".to_string(),
            out: "f16".to_string(),
            mp: MathPrecision::Stable,
        }),
    );
    let gt = gem.to_token();
    let gf: Vec<&str> = gt.split('|').collect();
    assert_eq!(gf.len(), 10, "KISS-CLASSIFY-6.6-0015: gem token missing the contraction field");
    assert!(
        gf[9].split('/').any(|c| c == "f8e5m2"),
        "KISS-CLASSIFY-6.6-0015: gem weight dtype not carried in the contraction field"
    );
    assert!(
        gf[9].split('/').any(|c| c == "f16"),
        "KISS-CLASSIFY-6.6-0015: gem output dtype not carried in the contraction field"
    );
    for sub in gf[7].split(';') {
        for (pos, comp) in sub.split('/').enumerate() {
            if pos == 1 {
                continue;
            }
            assert!(
                !DTYPES.contains(&comp),
                "KISS-CLASSIFY-6.6-0015: a gem sub-key component `{comp}` is a dtype"
            );
        }
    }
}

// ============================================================================
// KISS-CLASSIFY-6.8-0005 — target_capability charset (single unambiguous field)
// ============================================================================

/// KISS-CLASSIFY-6.8-0005: a `target_capability` token is case-sensitive ASCII and
/// MUST NOT contain a `structure_key` field separator (`;` or `/`), any whitespace
/// or control byte (`0x00`–`0x20`, `0x7f`), or any non-ASCII byte, so it embeds as
/// one unambiguous field; a token whose target violates the charset is a typed
/// decline.
#[test]
fn test_classify_target_token_charset() {
    let base = "sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f|-";
    assert!(from_token(base).is_ok(), "KISS-CLASSIFY-6.8-0005: baseline token invalid");

    // legal targets still parse: `.` (e.g. spirv1.6) and uppercase letters (ASCII,
    // case-sensitive) are permitted.
    //
    // These are arbitrary WELL-FORMED tokens; capability semantics are irrelevant
    // here. §6.8 has two orthogonal layers — well-formedness (§6.8-0001/-0005,
    // grammar and charset) and capability validity (§6.8-0004, a vocabulary owned
    // by each namespace's maintainer). This fixture asserts only the first, so a
    // token may be well-formed and still a poor capability choice; `spirv1.6`
    // appears purely as a dot-bearing string. See the §6.8 informative note.
    for good in ["cuda:sm89", "vulkan:spirv1.6", "rocm:gfx942", "cuda:sm90a", "Cuda:sm89"] {
        let t = base.replacen("cuda:sm89", good, 1);
        assert!(
            from_token(&t).is_ok(),
            "KISS-CLASSIFY-6.8-0005: legal target `{good}` was rejected"
        );
    }

    // a target containing a field separator, whitespace/control byte, or non-ASCII
    // byte is a typed BadTargetCharset decline. The CURRENT codec without the field-3
    // charset gate ACCEPTS `cuda:sm/89` (the colon check alone passes), so each of
    // these asserts FAILS if the gate is removed.
    let del = "cuda:sm\u{7f}89"; // 0x7f control byte
    let non_ascii = "cuda:sm\u{e9}89"; // 'é' -> UTF-8 bytes 0xc3 0xa9 (both >= 0x80)
    for bad in ["cuda:sm/89", "cuda:sm;89", "cuda:sm 89", del, non_ascii] {
        let t = base.replacen("cuda:sm89", bad, 1);
        assert_eq!(
            from_token(&t),
            Err(KeyDecline::BadTargetCharset),
            "KISS-CLASSIFY-6.8-0005: target `{bad}` with a forbidden byte was accepted"
        );
    }
}
