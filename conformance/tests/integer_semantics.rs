//! KISS-Conform golden/edge vectors for the pinned **integer** primitive
//! semantics (KISS-Ops §6.4 arithmetic atoms on the integer path, §6.10 bitwise
//! atoms). These transcribe the prose the spec spends pinning integer behavior —
//! two's-complement **wrapping** at the boundary (§6.2-0002, §6.4-0005), the
//! **arithmetic-vs-logical** `shr` split keyed on signedness (§6.10-0003), and
//! the out-of-range shift being **target-defined, not pinned** (§6.10-0004) —
//! now executable. Integer results are determinism-class exact-byte (§6.0-0002,
//! the exact-byte class enumerated in §6.0-0001 with precedence in §6.0-0005 —
//! NOT §6.8, which is the declared-ULP transcendental class); there is no
//! NaN/ULP class, so every check is exact equality.

use kiss_conformance::integer::*;

// KISS-OPS-6.4-0001 / §6.2-0002 — `test_ops_add_sub_mul` (integer path):
// wrapping two's-complement add/sub/mul, per width, never saturating, never UB.
#[test]
fn test_ops_add_sub_mul_wrapping() {
    // signed MAX+1 wraps to MIN (§6.2-0002); unsigned MAX+1 wraps to 0.
    assert_eq!(add(i8::MAX, 1i8), i8::MIN);
    assert_eq!(add(u8::MAX, 1u8), 0u8);
    assert_eq!(add(i16::MAX, 1i16), i16::MIN); // s16 wraps at 16 bits
    assert_eq!(add(u16::MAX, 1u16), 0u16); // u16 wraps at 16 bits
    assert_eq!(add(i32::MAX, 1i32), i32::MIN);
    assert_eq!(add(u32::MAX, 1u32), 0u32);
    assert_eq!(add(i64::MAX, 1i64), i64::MIN);
    assert_eq!(add(u64::MAX, 1u64), 0u64); // u64 wraps at 64 bits

    // unsigned 0-1 = unsigned MAX (all ones); signed MIN-1 = MAX.
    assert_eq!(sub(0u8, 1u8), u8::MAX);
    assert_eq!(sub(0u32, 1u32), u32::MAX);
    assert_eq!(sub(i8::MIN, 1i8), i8::MAX);
    assert_eq!(sub(i32::MIN, 1i32), i32::MAX);

    // mul keeps only the low BITS (high bits discarded); signed MIN * -1 = MIN
    // (overflow WRAPS, does not saturate).
    assert_eq!(mul(i8::MIN, -1i8), i8::MIN);
    assert_eq!(mul(i32::MIN, -1i32), i32::MIN);
    assert_eq!(mul(i64::MIN, -1i64), i64::MIN);
    assert_eq!(mul(16u8, 16u8), 0u8); // 256 mod 256 = 0
    assert_eq!(mul(0xFFFF_FFFFu32, 0xFFFF_FFFFu32), 1u32); // (-1)^2 low32 = 1
    // ordinary values agree with plain arithmetic
    assert_eq!(add(3i32, 5i32), 8i32);
    assert_eq!(sub(5i32, 8i32), -3i32);
    assert_eq!(mul(6i32, 7i32), 42i32);
}

// KISS-OPS-6.4-0003 — `test_ops_neg` (integer path): two's-complement wrapping
// negate. The distinct integer neg, NOT the float sign-bit flip.
#[test]
fn test_ops_neg_integer() {
    assert_eq!(neg(3i32), -3i32);
    assert_eq!(neg(-3i32), 3i32);
    // unsigned neg(0)=0, neg(1)=unsigned MAX (2^BITS - 1).
    assert_eq!(neg(0u8), 0u8);
    assert_eq!(neg(1u8), u8::MAX);
    assert_eq!(neg(1u32), u32::MAX);
    assert_eq!(neg(0u32), 0u32);
}

// KISS-OPS-6.4-0005 — `test_ops_int_neg_abs_wrap`: the pinned boundary wrap.
// neg(INT_MIN)=INT_MIN and abs(INT_MIN)=INT_MIN for EVERY signed width; MUST
// NOT saturate or trap.
#[test]
fn test_ops_int_neg_abs_wrap() {
    assert_eq!(neg(i8::MIN), i8::MIN);
    assert_eq!(neg(i16::MIN), i16::MIN); // s16 boundary wrap
    assert_eq!(neg(i32::MIN), i32::MIN);
    assert_eq!(neg(i64::MIN), i64::MIN);
    assert_eq!(abs(i8::MIN), i8::MIN);
    assert_eq!(abs(i16::MIN), i16::MIN); // s16 boundary wrap
    assert_eq!(abs(i32::MIN), i32::MIN);
    assert_eq!(abs(i64::MIN), i64::MIN);
    // ordinary abs, and the unsigned identity.
    assert_eq!(abs(-1i32), 1i32);
    assert_eq!(abs(-128i32 + 1), 127i32);
    assert_eq!(abs(5u8), 5u8);
    assert_eq!(abs(u32::MAX), u32::MAX); // unsigned abs is identity
}

// KISS-OPS-6.4-0002 — `test_ops_div_float_only`: integer div/rem are EXCLUDED
// from the op set (division is float-only; integer div-by-zero is target-UB).
// The POSITIVE half of the pin — "division exists, and it is FLOAT" — is checked
// here by exercising the ONLY division oracle in the harness, `semantics::div`
// over f32. The NEGATIVE half — that `integer::div`/`integer::rem` do NOT resolve
// — is a compile-fail (trybuild) vector in the negative-vector slice: it is not
// expressible as a runtime assertion here because naming a nonexistent path would
// fail to COMPILE this test rather than fail it. The integer module exposes no
// div/rem symbol; a later implementer must not add one. A load-bearing NON-op.
#[test]
fn test_ops_div_float_only() {
    use kiss_conformance::semantics;
    // Division is pinned FLOAT-ONLY: the float oracle divides, with the IEEE-754
    // results integer division could never produce (no target-UB at 0).
    assert_eq!(semantics::div(6.0f32, 2.0f32), 3.0f32);
    assert_eq!(semantics::div(-7.5f32, 2.5f32), -3.0f32);
    // float div-by-zero is a pinned IEEE result (±inf / NaN), NOT the target-UB
    // that made integer division-by-zero unpinnable and excluded from the op set.
    assert_eq!(semantics::div(1.0f32, 0.0f32), f32::INFINITY);
    assert_eq!(semantics::div(-1.0f32, 0.0f32), f32::NEG_INFINITY);
    assert!(semantics::div(0.0f32, 0.0f32).is_nan());
    // There is intentionally no `integer::div`/`integer::rem`; the compile-fail
    // guard against re-adding one lives in the trybuild negative-vector slice
    // (§6.4-0002).
}

// KISS-OPS-6.10-0002 — `test_ops_bitwise_logic`: bit_and/or/xor/not over the
// integer bit pattern, with the algebraic identities per width.
#[test]
fn test_ops_bitwise_logic() {
    // AND: annihilator 0, identity all-ones.
    assert_eq!(bit_and(0xF0u8, 0x00u8), 0x00u8);
    assert_eq!(bit_and(0xF0u8, 0xFFu8), 0xF0u8);
    // OR: identity 0, saturator all-ones.
    assert_eq!(bit_or(0xF0u8, 0x00u8), 0xF0u8);
    assert_eq!(bit_or(0x0Fu8, 0xF0u8), 0xFFu8);
    // XOR: x^x=0, x^0=x, x^all-ones = bit_not(x).
    assert_eq!(bit_xor(0xABu8, 0xABu8), 0x00u8);
    assert_eq!(bit_xor(0xABu8, 0x00u8), 0xABu8);
    assert_eq!(bit_xor(0xABu8, 0xFFu8), bit_not(0xABu8));
    // NOT: ~0 = all-ones; signed ~x = -x-1; ~MIN = MAX (signed).
    assert_eq!(bit_not(0u8), u8::MAX);
    assert_eq!(bit_not(0u32), u32::MAX);
    assert_eq!(bit_not(0i32), -1i32); // ~0 = -1 signed
    assert_eq!(bit_not(5i32), -6i32); // ~x = -x-1
    assert_eq!(bit_not(i32::MIN), i32::MAX);
}

// KISS-OPS-6.10-0003 — `test_ops_shift_arithmetic_vs_logical`: the load-bearing
// signed/unsigned split. shl is logical for both; shr is ARITHMETIC (sign
// replicated) on signed and LOGICAL (zero filled) on unsigned.
#[test]
fn test_ops_shift_arithmetic_vs_logical() {
    // shift-by-0 — the identity at the LOW boundary of the domain [0, BITS). A
    // distinct edge from BITS-1: amt==0 is in range, so it MUST return Some(x)
    // unchanged (never None, never off-by-one to the empty domain), for both
    // shift directions and both signednesses.
    assert_eq!(shl(0x3Ci8, 0), Some(0x3Ci8)); // signed shl, unchanged
    assert_eq!(shr(-1i8, 0), Some(-1i8)); // signed arithmetic shr, unchanged
    assert_eq!(shr(i32::MIN, 0), Some(i32::MIN)); // signed, sign bit preserved
    assert_eq!(shl(0xC3u8, 0), Some(0xC3u8)); // unsigned shl, unchanged
    assert_eq!(shr(0xC3u8, 0), Some(0xC3u8)); // unsigned logical shr, unchanged
    assert_eq!(shr(u32::MAX, 0), Some(u32::MAX));

    // shl — logical left, same for signed and unsigned; bits past the top drop.
    assert_eq!(shl(1u8, 1), Some(2u8));
    assert_eq!(shl(0x40u8, 1), Some(0x80u8)); // into the top bit
    assert_eq!(shl(0x80u8, 1), Some(0x00u8)); // shifted off the top -> 0
    assert_eq!(shl(1i8, 7), Some(i8::MIN)); // 1<<7 = 0x80 = -128 (into sign bit)
    assert_eq!(shl(3i32, 4), Some(48i32));

    // shr on SIGNED = arithmetic (sign fill): shr(-1,k) = -1 for all in-range k.
    assert_eq!(shr(-1i8, 1), Some(-1i8));
    assert_eq!(shr(-1i32, 31), Some(-1i32));
    assert_eq!(shr(-8i32, 1), Some(-4i32));
    assert_eq!(shr(i32::MIN, 1), Some(i32::MIN / 2)); // sign preserved
    assert_eq!(shr(i8::MIN, 1), Some(-64i8));

    // shr on UNSIGNED = logical (zero fill): a high-bit-set value zero-fills top.
    assert_eq!(shr(0x80u8, 1), Some(0x40u8)); // NOT 0xC0 — top zero-filled
    assert_eq!(shr(u8::MAX, 1), Some(0x7Fu8));
    assert_eq!(shr(0x8000_0000u32, 1), Some(0x4000_0000u32));
    assert_eq!(shr(u32::MAX, 1), Some(0x7FFF_FFFFu32));
}

// KISS-OPS-6.10-0004 — `test_ops_shift_out_of_range_target_defined`: any shift
// amount <0 or >=BITS is target-defined (NOT pinned). The oracle represents this
// as None — a first-class "declines to pin" value, never a Rust shift panic.
#[test]
fn test_ops_shift_out_of_range_target_defined() {
    // amount >= BITS -> None (per width).
    assert_eq!(shl(1u8, 8), None);
    assert_eq!(shr(1u8, 8), None);
    assert_eq!(shl(1i8, 8), None);
    assert_eq!(shl(1i32, 32), None);
    assert_eq!(shr(1i32, 32), None);
    assert_eq!(shl(1i64, 64), None);
    assert_eq!(shr(1i64, 64), None);
    // negative amount -> None (collapsed with too-large into one "not pinned").
    assert_eq!(shl(1i32, -1), None);
    assert_eq!(shr(1i32, -1), None);
    // the largest in-range amount BITS-1 is still Some (domain is [0, BITS)).
    assert_eq!(shl(1u8, 7), Some(0x80u8));
    assert_eq!(shr(u8::MAX, 7), Some(1u8));
    assert_eq!(shl(1i32, 31), Some(i32::MIN));
}

// KISS-OPS-6.10-0005 — `test_ops_popcount_clz_ctz`: set-bit / leading-zero /
// trailing-zero counts over the bit pattern, INCLUDING the pinned 0 / -1 / MAX
// edges. clz(0)=BITS and ctz(0)=BITS are the pinned width answers (hardware may
// leave 0 undefined, but "over the bit pattern" forces the width).
#[test]
fn test_ops_popcount_clz_ctz() {
    // popcount
    assert_eq!(popcount(0u8), 0);
    assert_eq!(popcount(u8::MAX), 8);
    assert_eq!(popcount(-1i32), 32); // all ones
    assert_eq!(popcount(u32::MAX), 32);
    assert_eq!(popcount(0x8000_0000u32), 1); // single power of two
    assert_eq!(popcount(-1i64), 64);

    // clz — pinned zero edge = BITS.
    assert_eq!(clz(0u8), 8);
    assert_eq!(clz(0i32), 32);
    assert_eq!(clz(0u32), 32);
    assert_eq!(clz(0i64), 64);
    assert_eq!(clz(1u8), 7); // BITS-1
    assert_eq!(clz(1u32), 31);
    assert_eq!(clz(i8::MIN), 0); // sign bit set -> 0 leading zeros
    assert_eq!(clz(i32::MIN), 0);
    assert_eq!(clz(0x8000_0000u32), 0);
    assert_eq!(clz(i32::MAX), 1); // 0x7FFF_FFFF -> one leading zero

    // ctz — pinned zero edge = BITS.
    assert_eq!(ctz(0u8), 8);
    assert_eq!(ctz(0i32), 32);
    assert_eq!(ctz(0i64), 64);
    assert_eq!(ctz(1u32), 0);
    assert_eq!(ctz(2u32), 1);
    assert_eq!(ctz(i8::MIN), 7); // 0x80 -> 7 trailing zeros (BITS-1)
    assert_eq!(ctz(i32::MIN), 31); // high-bit-only -> BITS-1
    assert_eq!(ctz(0x8000_0000u32), 31);
}

// KISS-OPS-6.10-0006 — `test_ops_narrow_int_promote_truncate_composition`: for a
// sub-32-bit integer dtype (`s8`/`u8`/`s16`/`u16`/…) every §6.10 atom MUST behave as
// promote-to-width → apply → TRUNCATE-ON-STORE, and its result MUST be INDEPENDENT of
// DAG sharing/hoisting. The hazard: C-family lowering promotes a narrow operand to
// `int` (32-bit) before the atom, so a COMPOSED narrow operand is observed un-truncated
// (the promoted value) when its producer is inlined but truncated (an 8-/16-bit temp)
// when hoisted. The pin: the operand carries its TRUNCATED narrow value either way —
// equivalently, narrow §6.10 operands are direct loads (leaf inputs). The narrow-typed
// oracle embodies (a): every intermediate stores in its own width, so it is exactly the
// leaf/hoisted evaluation; the un-truncated promoted path is constructed here in `u32`
// as the OUTLAWED value, and shown to genuinely diverge (so the clause is load-bearing).
#[test]
fn test_ops_narrow_int_promote_truncate_composition() {
    // (1) truncate-on-store: a single narrow atom already wraps mod 2^bitwidth. The u8
    // oracle stores 8 bits, so `bit_not`/`shl` equal the native narrow op, NOT the
    // 32-bit promoted result (`~0u8` is 0xFF, not the promoted 0xFFFF_FFFF).
    assert_eq!(bit_not(0x00u8), 0xFFu8);
    assert_eq!(shl(0x01u8, 7), Some(0x80u8)); // 1<<7 stays in 8 bits (0x80)

    // (2) the composition hazard, made concrete on u8. Let the narrow operand be a
    // COMPOSED value: t = add(200, 100). Truncate-on-store gives the 8-bit temp 44
    // (0x2C); the un-truncated PROMOTED intermediate is 300 (0x12C). A downstream narrow
    // atom MUST observe the truncated 44, never the promoted 300.
    let (a, b) = (200u8, 100u8);
    let narrow_temp: u8 = add(a, b); // truncate-on-store => 44
    assert_eq!(narrow_temp, 44u8);
    let promoted_untruncated: u32 = a as u32 + b as u32; // 300 — the outlawed value
    assert_eq!(promoted_untruncated, 300u32);

    // `shr` by 4 makes the two evaluations DIVERGE — exactly the DAG-sharing ambiguity
    // the clause outlaws. The PINNED result follows truncate-on-store (leaf/hoisted).
    let pinned = shr(narrow_temp, 4).unwrap(); // 44 >> 4 = 2  (PINNED)
    let hazard = (promoted_untruncated >> 4) as u8; // 300 >> 4 = 18 (inlined-promoted)
    assert_ne!(pinned, hazard); // the two genuinely differ -> the hazard is real
    assert_eq!(pinned, 2u8); // pinned: the atom sees the 8-bit temp
    assert_eq!(hazard, 18u8); // the outlawed inlined-promoted result

    // A bit-count atom is an unambiguous witness of the same split: `popcount` over the
    // NARROW bit pattern of 44 (0b0010_1100) is 3; over the promoted 300 (0b1_0010_1100)
    // it is 4. The atom MUST count the 8-bit truncated pattern (§6.10-0005 "over the
    // integer bit pattern" = the operand dtype's own bits), never the promoted pattern.
    assert_eq!(popcount(narrow_temp), 3);
    assert_eq!(promoted_untruncated.count_ones(), 4);

    // (3) a 16-bit witness — "any width narrower than 32 bits". add(0xFFFF, 2) stores
    // 1 (0x1_0001 mod 2^16); the promoted intermediate is 0x1_0001. A narrow atom sees
    // the truncated 1, and the promoted value is a distinct 17-bit pattern.
    let t16: u16 = add(0xFFFFu16, 2u16); // 0x1_0001 mod 2^16 = 1
    assert_eq!(t16, 1u16);
    assert_eq!(shr(t16, 0).unwrap(), 1u16); // narrow atom observes 1, not 0x1_0001
    assert_ne!(0x1_0001u32, t16 as u32);
}

// KISS-OPS-6.16-0006 — dtype tokens for the pinned ordinary integer set. The set
// now includes the 16-bit tokens `s16`/`u16` and 64-bit unsigned `u64` (the
// former open question is resolved the other way: they are IN, made
// compute-operable by the §6.16 bit layouts + §6.2-0002 wrapping arithmetic).
#[test]
fn test_ops_integer_dtype_tokens() {
    assert_eq!(<i8 as IntDtype>::dtype_token(), "s8");
    assert_eq!(<i16 as IntDtype>::dtype_token(), "s16");
    assert_eq!(<u8 as IntDtype>::dtype_token(), "u8");
    assert_eq!(<u16 as IntDtype>::dtype_token(), "u16");
    assert_eq!(<i32 as IntDtype>::dtype_token(), "i32");
    assert_eq!(<u32 as IntDtype>::dtype_token(), "u32");
    assert_eq!(<u64 as IntDtype>::dtype_token(), "u64");
    assert_eq!(<i64 as IntDtype>::dtype_token(), "i64");
    // §6.16 pinned bit widths for the three additions.
    assert_eq!(<i8 as IntDtype>::BITS, 8);
    assert_eq!(<i16 as IntDtype>::BITS, 16);
    assert_eq!(<u16 as IntDtype>::BITS, 16);
    assert_eq!(<u64 as IntDtype>::BITS, 64);
    assert_eq!(<i64 as IntDtype>::BITS, 64);
    // signedness selects the two's-complement (s16) vs plain-unsigned (u16/u64) path.
    assert!(<i16 as IntDtype>::is_signed());
    assert!(!<u16 as IntDtype>::is_signed());
    assert!(!<u64 as IntDtype>::is_signed());
    assert!(<i32 as IntDtype>::is_signed());
    assert!(!<u32 as IntDtype>::is_signed());
    // to_bits_u64 reinterprets (does NOT sign-extend) the two's-complement pattern.
    assert_eq!((-1i8).to_bits_u64(), 0xFF);
    assert_eq!((-1i16).to_bits_u64(), 0xFFFF);
    assert_eq!((-1i32).to_bits_u64(), 0xFFFF_FFFF);
    assert_eq!((-1i64).to_bits_u64(), u64::MAX);
    assert_eq!(u16::MAX.to_bits_u64(), 0xFFFF);
    assert_eq!(u64::MAX.to_bits_u64(), u64::MAX);
}
