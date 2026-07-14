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
    assert_token("KISS-CLASSIFY-6.7-0001", &k, "sk2|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-");
}

#[test]
fn a1_elementwise_with_broadcast_operand() {
    let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), br1(), co4()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0004", &k, "sk2|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;br/01/v1/d16/f;co/00/v4/d16/f|-");
}

#[test]
fn a1_unary_f16_v8() {
    let k = key("une", "f16", "cuda:sm89", WorkClass::Grid, 2, vec![co8(), co8()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0003", &k, "sk2|une|f16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f;co/00/v8/d16/f|-");
}

#[test]
fn a1_reduction_trailing_axis() {
    let k = key("red", "f32", "cuda:sm89", WorkClass::Warp, 2, vec![co1_d8(), co1_da()], Reduce::Trailing, None);
    assert_token("KISS-CLASSIFY-6.7-0005", &k, "sk2|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rlast");
}

#[test]
fn a1_reduction_all_axes() {
    let k = key("red", "f32", "cuda:sm89", WorkClass::Warp, 2, vec![co1_d8(), co1_da()], Reduce::All, None);
    assert_token("KISS-CLASSIFY-6.7-0005", &k, "sk2|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall");
}

#[test]
fn a1_reduction_rank1_all_axes() {
    let k = key("red", "f32", "cuda:sm89", WorkClass::Warp, 1, vec![co1_d8(), co1_da()], Reduce::All, None);
    assert_token("KISS-CLASSIFY-6.6-0009", &k, "sk2|red|f32|cuda:sm89|ix32|warp|r1|co/00/v1/d8/f;co/00/v1/da/f|rall");
}

#[test]
fn a1_reduction_subset_mask() {
    // rank-4 reduction over a non-trivial subset -> explicit x<hh> keepdim mask 0x0a
    let k = key("red", "f32", "cuda:sm89", WorkClass::Block, 4, vec![co1_da(), co1_da()], Reduce::Subset(0x0a), None);
    assert_token("KISS-CLASSIFY-6.7-0005", &k, "sk2|red|f32|cuda:sm89|ix32|block|r4|co/00/v1/da/f;co/00/v1/da/f|x0a");
}

#[test]
fn a1_binary_two_operands() {
    let k = key("bin", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), co4()], Reduce::None, None);
    assert_token("KISS-CLASSIFY-6.7-0001", &k, "sk2|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f|-");
}

#[test]
fn a1_dense_contraction_cuda() {
    let c = Contraction { m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16 };
    let k = key("gem", "f32", "cuda:sm89", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(c));
    assert_token("KISS-CLASSIFY-6.7-0006", &k, "sk2|gem|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16");
}

#[test]
fn a1_dense_contraction_vulkan_target() {
    let c = Contraction { m: SizeClass::Tiny, n: SizeClass::Large, k: SizeClass::Large, k_div: DivBucket::D16 };
    let k = key("gem", "f32", "vulkan:spirv1.6", WorkClass::Grid, 2, vec![co4(), co4(), co4()], Reduce::None, Some(c));
    assert_token("KISS-CLASSIFY-6.8", &k, "sk2|gem|f32|vulkan:spirv1.6|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16");
}

// ---- A.2 decline vectors: structural codec rejects (§6.7-0009) ---------------

const A_GOLDEN: &str = "sk2|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-";

#[test]
fn reject_wrong_field_count() {
    assert_eq!(from_token("sk2|bin|f32"), Err(KeyDecline::WrongFieldCount { got: 3 }));
}

#[test]
fn reject_bad_version_prefix() {
    let t = A_GOLDEN.replacen("sk2", "sk9", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::BadVersionPrefix));
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
    // A.2 decline vector: `sk2|zzz|f32|…` — op-family outside the closed §6.5-0006 set.
    let t = A_GOLDEN.replacen("|bin|", "|zzz|", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::UnknownOpFamily));
}

#[test]
fn reject_unknown_dtype() {
    // A.2 decline vector: `sk2|bin|f99|…` — dtype outside the closed §6.1 set.
    let t = A_GOLDEN.replacen("|f32|", "|f99|", 1);
    assert_eq!(from_token(&t), Err(KeyDecline::UnknownDtype));
}

#[test]
fn accepts_every_closed_op_family_and_dtype() {
    // every one of the 24 op-family codes and 23 dtype tokens is recognized
    for fam in OP_FAMILIES {
        let t = A_GOLDEN.replacen("|bin|", &format!("|{fam}|"), 1);
        assert_ne!(from_token(&t), Err(KeyDecline::UnknownOpFamily), "op-family {fam} rejected");
    }
    for dt in DTYPES {
        let t = A_GOLDEN.replacen("|f32|", &format!("|{dt}|"), 1);
        assert_ne!(from_token(&t), Err(KeyDecline::UnknownDtype), "dtype {dt} rejected");
    }
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
