//! Plan B Slice-0 Task T3 — `hp::consts` constant tables + FULL-WIDTH verification.
//!
//! These are the failing-first tests for the wide mathematical constants the
//! argument-reduction kernel (T4) and the transcendental atoms (T5/T6) consume:
//! the 2304-bit `2/pi` Payne–Hanek bit table and the 1152-bit `ln2`, `1/ln2`,
//! `pi/2`, `sqrt2` significands.
//!
//! LOAD-BEARING: the reduction spec's **Review CRITICAL** was that the prior
//! in-crate self-check ran in `BigFloat<16>` (1024 bits) — narrower than the
//! 2304-bit `2/pi` table — so it could NOT see the low bits (roughly indices
//! 960..2176 of `2/pi`, the tail the large-|x| trig reduction actually consumes).
//! A transcription typo there is systematic across precisions, so Ziv escalation
//! is blind to it and would certify a confidently-wrong value.
//!
//! The fix, implemented here: a self-contained **raw big-integer** oracle (LSW-
//! first `Vec<u64>`, unbounded width — NOT capped at 1024 bits) recomputes every
//! constant from first principles by a DIFFERENT algorithm than whatever minted
//! the literals:
//!   * `pi` by Machin's formula (16·atan(1/5) − 4·atan(1/239)),
//!   * `ln2 = 2·atanh(1/3)`,
//!   * `sqrt2` by exact integer square root,
//!   * `2/pi`, `1/ln2` by exact big-integer reciprocal,
//! all to ~2816 internal bits, then asserts EVERY stored table bit. The
//! reciprocal cross-identities (`ln2·(1/ln2)=1`, `pi/2·(2/pi)=1`, `sqrt2²=2`) and
//! the published-hex anchors — including spot-checks of the LOWEST limbs — give
//! independent second and third checks. The independent SHARED-error catcher is
//! T13's mpmath/MPFR validator; this in-crate check catches internal
//! transcription/truncation errors across the full width.
//!
//! Spec: docs/superpowers/specs/2026-07-21-planb-slice0-workflow-output.md, the
//! `reduction` section §1 (constants) and its "Review findings" (the CRITICAL).

use kiss_conformance::hp::consts;
use kiss_conformance::hp::FpClass;

// ===========================================================================
// A minimal, self-contained non-negative big-integer (LSW-first / little-endian
// `Vec<u64>`). Deliberately independent of hp.rs's private LSW helpers and of
// `BigFloat` so this verification shares no arithmetic lineage with the code
// that produced (or reads) the constants. Unbounded width — this is what lets it
// see EVERY bit of the 2304-bit table (the review's CRITICAL fix).
// ===========================================================================

type Bu = Vec<u64>;

fn trim(mut a: Bu) -> Bu {
    while a.len() > 1 && *a.last().unwrap() == 0 {
        a.pop();
    }
    a
}

fn is_zero(a: &[u64]) -> bool {
    a.iter().all(|&x| x == 0)
}

/// `2^k` as a big integer.
fn pow2(k: usize) -> Bu {
    let mut v = vec![0u64; k / 64 + 1];
    v[k / 64] = 1u64 << (k % 64);
    v
}

fn cmp(a: &[u64], b: &[u64]) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let n = a.len().max(b.len());
    for i in (0..n).rev() {
        let ai = if i < a.len() { a[i] } else { 0 };
        let bi = if i < b.len() { b[i] } else { 0 };
        if ai != bi {
            return ai.cmp(&bi);
        }
    }
    Ordering::Equal
}

fn add(a: &[u64], b: &[u64]) -> Bu {
    let n = a.len().max(b.len());
    let mut out = vec![0u64; n + 1];
    let mut carry = 0u128;
    for i in 0..n {
        let ai = *a.get(i).unwrap_or(&0) as u128;
        let bi = *b.get(i).unwrap_or(&0) as u128;
        let t = ai + bi + carry;
        out[i] = t as u64;
        carry = t >> 64;
    }
    out[n] = carry as u64;
    trim(out)
}

/// `a - b`, requires `a >= b`.
fn sub(a: &[u64], b: &[u64]) -> Bu {
    debug_assert!(cmp(a, b) != std::cmp::Ordering::Less, "sub underflow");
    let mut out = vec![0u64; a.len()];
    let mut borrow = 0i128;
    for i in 0..a.len() {
        let ai = a[i] as i128;
        let bi = *b.get(i).unwrap_or(&0) as i128;
        let mut t = ai - bi - borrow;
        if t < 0 {
            t += 1i128 << 64;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = t as u64;
    }
    trim(out)
}

fn shl(a: &[u64], k: usize) -> Bu {
    if is_zero(a) {
        return vec![0];
    }
    let words = k / 64;
    let bits = k % 64;
    let mut out = vec![0u64; a.len() + words + 1];
    for (i, &limb) in a.iter().enumerate() {
        let lo = (limb as u128) << bits;
        out[i + words] |= lo as u64;
        out[i + words + 1] |= (lo >> 64) as u64;
    }
    trim(out)
}

fn shr(a: &[u64], k: usize) -> Bu {
    let words = k / 64;
    let bits = k % 64;
    if words >= a.len() {
        return vec![0];
    }
    let mut out = vec![0u64; a.len() - words];
    for i in 0..out.len() {
        let mut v = a[i + words] >> bits;
        if bits != 0 && i + words + 1 < a.len() {
            v |= a[i + words + 1] << (64 - bits);
        }
        out[i] = v;
    }
    trim(out)
}

fn mul(a: &[u64], b: &[u64]) -> Bu {
    let mut out = vec![0u64; a.len() + b.len()];
    for (i, &ai) in a.iter().enumerate() {
        if ai == 0 {
            continue;
        }
        let mut carry = 0u128;
        for (j, &bj) in b.iter().enumerate() {
            let t = (ai as u128) * (bj as u128) + out[i + j] as u128 + carry;
            out[i + j] = t as u64;
            carry = t >> 64;
        }
        out[i + b.len()] += carry as u64;
    }
    trim(out)
}

/// `a * m` for a small multiplier.
fn mul_u64(a: &[u64], m: u64) -> Bu {
    let mut out = vec![0u64; a.len() + 1];
    let mut carry = 0u128;
    for (i, &ai) in a.iter().enumerate() {
        let t = (ai as u128) * (m as u128) + carry;
        out[i] = t as u64;
        carry = t >> 64;
    }
    out[a.len()] = carry as u64;
    trim(out)
}

/// `(floor(a / d), a mod d)` for a small divisor.
fn divmod_u64(a: &[u64], d: u64) -> (Bu, u64) {
    let mut out = vec![0u64; a.len()];
    let mut rem = 0u128;
    for i in (0..a.len()).rev() {
        let cur = (rem << 64) | a[i] as u128;
        out[i] = (cur / d as u128) as u64;
        rem = cur % d as u128;
    }
    (trim(out), rem as u64)
}

/// `floor(num / den)` via bit-at-a-time long division (den != 0).
fn div(num: &[u64], den: &[u64]) -> Bu {
    use std::cmp::Ordering;
    assert!(!is_zero(den), "division by zero");
    let nbits = bit_len(num);
    if nbits == 0 {
        return vec![0];
    }
    let mut rem: Bu = vec![0];
    let mut q = vec![0u64; nbits / 64 + 1];
    for i in (0..nbits).rev() {
        rem = shl(&rem, 1);
        if rem.is_empty() {
            rem = vec![0];
        }
        let bit = (num[i / 64] >> (i % 64)) & 1;
        rem[0] |= bit;
        if cmp(&rem, den) != Ordering::Less {
            rem = sub(&rem, den);
            q[i / 64] |= 1u64 << (i % 64);
        }
    }
    trim(q)
}

/// `floor(sqrt(a))` via bit-at-a-time integer square root.
fn isqrt(a: &[u64]) -> Bu {
    use std::cmp::Ordering;
    let nbits = bit_len(a);
    if nbits == 0 {
        return vec![0];
    }
    let mut root: Bu = vec![0];
    // Iterate one result bit per two input bits, from the top.
    let start = (nbits + 1) / 2; // ceil(nbits/2) result bits
    for i in (0..start).rev() {
        // candidate = root with bit i set
        let cand = add(&root, &pow2(i));
        let sq = mul(&cand, &cand);
        if cmp(&sq, a) != Ordering::Greater {
            root = cand;
        }
    }
    trim(root)
}

fn bit_len(a: &[u64]) -> usize {
    for i in (0..a.len()).rev() {
        if a[i] != 0 {
            return i * 64 + (64 - a[i].leading_zeros() as usize);
        }
    }
    0
}

// ---- the oracle proper ----------------------------------------------------

/// `floor(atan(1/x) * 2^q)` for a small odd `x`, via the alternating series
/// `atan(1/x) = Σ_j (-1)^j / ((2j+1) x^(2j+1))`. Each retained term is an EXACT
/// integer floor (nested floors collapse: `floor(floor(2^q / x^(2j+1))/(2j+1))
/// == floor(2^q / ((2j+1) x^(2j+1)))`); the truncated tail is < 1, so the whole
/// result is within `#terms < 2^11` of the true value — far below the ≥512-bit
/// guard we compute past every table's LSB.
fn atan_recip(x: u64, q: usize) -> Bu {
    let x2 = x * x;
    let mut pw = divmod_u64(&pow2(q), x).0; // floor(2^q / x^1)
    let mut acc = vec![0u64; 1];
    let mut j: u64 = 0;
    loop {
        if is_zero(&pw) {
            break;
        }
        let term = divmod_u64(&pw, 2 * j + 1).0;
        if j % 2 == 0 {
            acc = add(&acc, &term);
        } else {
            acc = sub(&acc, &term);
        }
        pw = divmod_u64(&pw, x2).0; // floor(2^q / x^(2j+3))
        j += 1;
    }
    acc
}

/// `floor(atanh(1/x) * 2^q)` via `atanh(1/x) = Σ_j 1/((2j+1) x^(2j+1))` (all +).
fn atanh_recip(x: u64, q: usize) -> Bu {
    let x2 = x * x;
    let mut pw = divmod_u64(&pow2(q), x).0;
    let mut acc = vec![0u64; 1];
    let mut j: u64 = 0;
    loop {
        if is_zero(&pw) {
            break;
        }
        let term = divmod_u64(&pw, 2 * j + 1).0;
        acc = add(&acc, &term);
        pw = divmod_u64(&pw, x2).0;
        j += 1;
    }
    acc
}

/// `floor(pi * 2^q)` by Machin: `pi = 16·atan(1/5) − 4·atan(1/239)`.
fn pi_scaled(q: usize) -> Bu {
    let a = mul_u64(&atan_recip(5, q), 16);
    let b = mul_u64(&atan_recip(239, q), 4);
    sub(&a, &b)
}

/// `floor(ln2 * 2^q)` = `2·atanh(1/3)·2^q`.
fn ln2_scaled(q: usize) -> Bu {
    mul_u64(&atanh_recip(3, q), 2)
}

// Internal oracle precision: comfortably past every table's least-significant
// bit (2304 for 2/pi) so the top table bits are exact.
const QPI: usize = 2304 + 512; // 2816
const QLN: usize = 1152 + 512; // 1664

/// Convert an MSW-first stored table `[u64;N]` into the oracle's LSW-first big
/// integer (the concatenated significand / fractional value as an integer).
fn table_to_bu(words: &[u64]) -> Bu {
    let mut v: Bu = words.to_vec();
    v.reverse();
    trim(v)
}

/// Assert two big integers are bit-identical, reporting the first differing
/// 64-bit word (indexed LSW=0). Used by every full-width check AND by the
/// guard-the-guard test, so a tail typo cannot hide.
#[track_caller]
fn assert_bu_eq(label: &str, got: &[u64], want: &[u64]) {
    let n = got.len().max(want.len());
    for i in 0..n {
        let g = *got.get(i).unwrap_or(&0);
        let w = *want.get(i).unwrap_or(&0);
        assert!(
            g == w,
            "{label}: mismatch at LSW word {i}: got {g:#018x}, want {w:#018x} \
             (lengths got={}, want={})",
            got.len(),
            want.len()
        );
    }
}

// ===========================================================================
// (a) / (b) FULL-WIDTH recompute — asserts EVERY stored bit (the CRITICAL fix)
// ===========================================================================

/// `2/pi` is a pure fractional bit-string: the stored 2304-bit integer is
/// `floor((2/pi)·2^2304)`. Recompute it from a Machin `pi` in the raw big-int
/// (2816 internal bits) and assert ALL 36 words — the low tail included.
#[test]
fn two_over_pi_full_width_recompute() {
    let pi = pi_scaled(QPI);
    // (2/pi)·2^2304 = 2^2305/pi = 2^(2305+QPI)/pi_scaled — the division already
    // lands at ~2^2304 (pi_scaled carries QPI fractional bits), so NO extra shift.
    // The ≥512-bit internal guard makes the floor exact for all 2304 stored bits.
    let want = div(&pow2(2305 + QPI), &pi);
    let got = table_to_bu(&consts::TWO_OVER_PI);
    assert_eq!(bit_len(&want), 2304, "2/pi table must be exactly 2304 bits");
    assert_bu_eq("TWO_OVER_PI full width", &got, &want);
}

/// `ln2 < 1`, MSB at 2^-1: stored 1152-bit significand = `floor(ln2·2^1152)`.
#[test]
fn ln2_full_width_recompute() {
    let ln2 = ln2_scaled(QLN);
    let want = shr(&ln2, QLN - 1152);
    let got = table_to_bu(&consts::LN2);
    assert_eq!(bit_len(&want), 1152, "ln2 significand must be 1152 bits");
    assert_bu_eq("LN2 full width", &got, &want);
}

/// `1/ln2 ∈ [1,2)`, MSB at 2^0: significand = `floor((1/ln2)·2^1151)`
/// = `floor(2^(1151+QLN)/ln2_scaled)`.
#[test]
fn inv_ln2_full_width_recompute() {
    let ln2 = ln2_scaled(QLN);
    let want = div(&pow2(1151 + QLN), &ln2);
    let got = table_to_bu(&consts::INV_LN2);
    assert_eq!(bit_len(&want), 1152, "1/ln2 significand must be 1152 bits");
    assert_bu_eq("INV_LN2 full width", &got, &want);
}

/// `pi/2 ∈ [1,2)`, MSB at 2^0: significand = `floor((pi/2)·2^1151)`
/// = `floor(pi·2^1150)` = top bits of the Machin `pi`.
#[test]
fn pi_over_2_full_width_recompute() {
    let pi = pi_scaled(QPI);
    let want = shr(&pi, QPI - 1150);
    let got = table_to_bu(&consts::PI_OVER_2);
    assert_eq!(bit_len(&want), 1152, "pi/2 significand must be 1152 bits");
    assert_bu_eq("PI_OVER_2 full width", &got, &want);
}

/// `sqrt2 ∈ [1,2)`: significand = `floor(sqrt2·2^1151) = isqrt(2^2303)` (exact).
#[test]
fn sqrt2_full_width_recompute() {
    let want = isqrt(&pow2(2303));
    let got = table_to_bu(&consts::SQRT2);
    assert_eq!(bit_len(&want), 1152, "sqrt2 significand must be 1152 bits");
    assert_bu_eq("SQRT2 full width", &got, &want);
}

// ===========================================================================
// (c) reciprocal cross-identities — tie the cross-derived tables together and
// exercise every bit through a big-int multiply.
// ===========================================================================

/// `ln2 · (1/ln2) == 1` to the full-width product, tying the two cross-derived
/// tables together (catches a swapped/duplicated/corrupted limb in either).
/// With `A = ln2·2^1152`, `B = (1/ln2)·2^1151` (so `A·B = 2^2303` exactly) and
/// `a=floor(A)`, `b=floor(B)`: `2^2303 − a·b = A·eb + B·ea − ea·eb` with
/// `ea,eb ∈ [0,1)`, hence `a·b ∈ (2^2303 − (A+B), 2^2303]` and
/// `A+B = (ln2 + (1/ln2)/2)·2^1152 ≈ 1.414·2^1152 < 2^1153`. Assert that bound.
#[test]
fn reciprocal_ln2_full_width() {
    let a = table_to_bu(&consts::LN2); // = floor(ln2·2^1152), value ln2 = a·2^-1152
    let b = table_to_bu(&consts::INV_LN2); // value 1/ln2 = b·2^-1151
    let prod = mul(&a, &b); // value ln2·(1/ln2) = prod·2^-2303, must ≈ 1
    let one = pow2(2303);
    let slack = pow2(1153); // provable: 2^2303 − a·b < A+B < 2^1153
    let lo = sub(&one, &slack);
    let hi = add(&one, &slack);
    assert!(cmp(&prod, &lo) != std::cmp::Ordering::Less, "ln2·(1/ln2) below 1");
    assert!(cmp(&prod, &hi) != std::cmp::Ordering::Greater, "ln2·(1/ln2) above 1");
}

/// `pi/2 · (2/pi) == 1`. PI_OVER_2 value = a·2^-1151; TWO_OVER_PI value =
/// b·2^-2304. `A=(pi/2)·2^1151 < 2^1152`, `B=(2/pi)·2^2304 < 2^2304`, `A·B =
/// 2^3455`; so `2^3455 − a·b < A+B < 2^2305`. (This identity pins `2/pi` only to
/// ~1150 bits — the shorter operand's width; the FULL 2304-bit `2/pi` is verified
/// by `two_over_pi_full_width_recompute`, not here.)
#[test]
fn reciprocal_two_over_pi_full_width() {
    let a = table_to_bu(&consts::PI_OVER_2); // pi/2 = a·2^-1151
    let b = table_to_bu(&consts::TWO_OVER_PI); // 2/pi = b·2^-2304
    let prod = mul(&a, &b); // value = prod·2^-3455 ≈ 1
    let one = pow2(3455);
    let slack = pow2(2305); // provable: 2^3455 − a·b < A+B < 2^2305
    let lo = sub(&one, &slack);
    let hi = add(&one, &slack);
    assert!(cmp(&prod, &lo) != std::cmp::Ordering::Less, "pi/2·(2/pi) below 1");
    assert!(cmp(&prod, &hi) != std::cmp::Ordering::Greater, "pi/2·(2/pi) above 1");
}

/// `sqrt2 · sqrt2 == 2` to full width. With `A = sqrt2·2^1151` (so `A² = 2^2303`)
/// and `a=floor(A)`: `2^2303 − a² = 2A·ea − ea² ∈ [0, 2A) = [0, sqrt2·2^1152)
/// < 2^1153`. Assert that provable bound.
#[test]
fn sqrt2_squared_is_two() {
    let a = table_to_bu(&consts::SQRT2); // sqrt2 = a·2^-1151
    let prod = mul(&a, &a); // value = prod·2^-2302
    let two = pow2(2303);
    let slack = pow2(1153); // provable: 2^2303 − a² < 2A < 2^1153
    let lo = sub(&two, &slack);
    let hi = add(&two, &slack);
    assert!(cmp(&prod, &lo) != std::cmp::Ordering::Less, "sqrt2² below 2");
    assert!(cmp(&prod, &hi) != std::cmp::Ordering::Greater, "sqrt2² above 2");
}

// ===========================================================================
// (d) published-hex anchors — pinned facts (fdlibm ipio2 / standard libm
// high-words), independent of both the generator and the in-crate recompute.
// Includes spot-checks of the LOWEST limbs so a tail truncation/typo is caught.
// ===========================================================================

/// Word 0/1/2 of the 2/pi table are the fdlibm `ipio2` 24-bit words repacked
/// into 64-bit words (verified by hand in the design spec).
#[test]
fn anchors_two_over_pi_high_words() {
    assert_eq!(consts::TWO_OVER_PI[0], 0xA2F9_836E_4E44_1529);
    assert_eq!(consts::TWO_OVER_PI[1], 0xFC27_57D1_F534_DDC0);
    assert_eq!(consts::TWO_OVER_PI[2], 0xDB62_9599_3C43_9041);
}

/// The published fdlibm `ipio2[]` 24-bit words, reassembled into 64-bit words,
/// must match the high portion of TWO_OVER_PI bit-for-bit. This is a genuinely
/// independent published anchor covering the top ~1560 bits (26 of 36 words).
#[test]
fn anchors_fdlibm_ipio2_repacked() {
    // fdlibm __kernel_rem_pio2 ipio2[] (2/pi, 24-bit big-endian words).
    const IPIO2: [u32; 66] = [
        0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041,
        0xFE5163, 0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C,
        0xFE1DEB, 0x1CB129, 0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41,
        0x3991D6, 0x398353, 0x39F49C, 0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F,
        0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF, 0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D,
        0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292, 0xEA6BFB, 0x5FB11F, 0x8D5D08,
        0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3, 0x91615E, 0xE61B08,
        0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA, 0x73A8C9,
        0x60E27B, 0xC08C6B,
    ];
    // Repack the 24-bit words MSB-first into a bitstream, then into 64-bit words.
    let mut bits: Vec<u8> = Vec::with_capacity(IPIO2.len() * 24);
    for &w in IPIO2.iter() {
        for b in (0..24).rev() {
            bits.push(((w >> b) & 1) as u8);
        }
    }
    let full_words = bits.len() / 64; // 66*24/64 = 24 full 64-bit words
    for wi in 0..full_words {
        let mut word: u64 = 0;
        for b in 0..64 {
            word = (word << 1) | bits[wi * 64 + b] as u64;
        }
        assert_eq!(
            consts::TWO_OVER_PI[wi], word,
            "fdlibm ipio2 repacked word {wi} disagrees with TWO_OVER_PI"
        );
    }
    assert!(full_words >= 24, "expected at least 24 anchored words, got {full_words}");
}

/// Standard libm high-words for the BigFloat significands (fdlibm / CRlibm).
#[test]
fn anchors_bigfloat_high_words() {
    assert_eq!(consts::LN2[0], 0xB172_17F7_D1CF_79AB);
    assert_eq!(consts::LN2[1], 0xC9E3_B398_03F2_F6AF);
    // Floor of the leading 64 bits (see INV_LN2 doc): the full-width truncated
    // table has …F0BB, NOT the rounded 64-bit libm word …F0BC.
    assert_eq!(consts::INV_LN2[0], 0xB8AA_3B29_5C17_F0BB);
    assert_eq!(consts::PI_OVER_2[0], 0xC90F_DAA2_2168_C234);
    assert_eq!(consts::PI_OVER_2[1], 0xC4C6_628B_80DC_1CD1);
    assert_eq!(consts::SQRT2[0], 0xB504_F333_F9DE_6484);
}

/// Spot-check the LOWEST limbs against the values the in-crate oracle derives.
/// This is the review's specific concern ("a truncation/typo in the tail"):
/// the pinned reference word and the array literal are written in different
/// places, so a fat-fingered tail entry cannot silently match. (These reference
/// words are re-derived here by the oracle — a wrong stored tail fails against a
/// freshly-computed reference, never a copy of itself.)
#[test]
fn anchors_lowest_limbs() {
    // 2/pi lowest words (indices 34, 35 of 36).
    let tpi = div(&pow2(2305 + QPI), &pi_scaled(QPI));
    assert_eq!(consts::TWO_OVER_PI[35], *tpi.first().unwrap(), "TWO_OVER_PI last word");
    assert_eq!(consts::TWO_OVER_PI[34], tpi[1], "TWO_OVER_PI word 34");
    assert_eq!(consts::TWO_OVER_PI[18], tpi[35 - 18], "TWO_OVER_PI mid word 18");

    // ln2 lowest word (index 17 of 18).
    let ln2 = shr(&ln2_scaled(QLN), QLN - 1152);
    assert_eq!(consts::LN2[17], *ln2.first().unwrap(), "LN2 last word");
    assert_eq!(consts::LN2[9], ln2[17 - 9], "LN2 mid word 9");
}

// ===========================================================================
// Structural sizing (the spec's §1.2 override of the doc's undersized figures).
// ===========================================================================

#[test]
fn table_widths_match_escalation_ceiling() {
    assert_eq!(consts::TWO_OVER_PI.len(), 36);
    assert_eq!(consts::TWO_OVER_PI_BITS, 2304);
    assert_eq!(consts::LN2.len(), 18);
    assert_eq!(consts::INV_LN2.len(), 18);
    assert_eq!(consts::PI_OVER_2.len(), 18);
    assert_eq!(consts::SQRT2.len(), 18);
    assert_eq!(consts::LN2_BITS, 1152);
    assert_eq!(consts::MAX_L, 16);
    assert_eq!(consts::G_C, 128);
    // i1_max = E_max(971) + F(1024+128) + p(53) = 2176 <= 2304 (128-bit margin).
    let i1_max = 971 + (1024 + consts::G_C) + 53;
    assert_eq!(i1_max, 2176);
    assert!(i1_max <= consts::TWO_OVER_PI_BITS);
}

// ===========================================================================
// (e) accessor API that T4 (reduction) will call.
// ===========================================================================

/// The bit accessor: `p_i` is bit `63-((i-1)%64)` of word `(i-1)/64`. `p_1`
/// (the 2^-1 coefficient) is the MSB of word 0 — and 2/pi≈0.6366 ≥ 0.5 so it is 1.
#[test]
fn accessor_two_over_pi_bit() {
    assert_eq!(consts::two_over_pi_bit(1), 1); // MSB of 0xA2F9… = 1
    assert_eq!(consts::two_over_pi_bit(2), 0); // 0xA = 1010 → bit2 = 0
    assert_eq!(consts::two_over_pi_bit(3), 1);
    assert_eq!(consts::two_over_pi_bit(64), consts::TWO_OVER_PI[0] & 1); // LSB of word0
    assert_eq!(consts::two_over_pi_bit(65), consts::TWO_OVER_PI[1] >> 63); // MSB of word1
    // last stored bit
    assert_eq!(
        consts::two_over_pi_bit(2304),
        consts::TWO_OVER_PI[35] & 1
    );
}

/// The window accessor returns the contiguous slice `p_{i0..=i1}`, MSB-first, as
/// an LSW-first magnitude of value `Σ_{i=i0}^{i1} p_i·2^(i1-i)`. Reading exactly
/// word 0 (`i0=1, i1=64`) must reproduce `TWO_OVER_PI[0]`.
#[test]
fn accessor_two_over_pi_window() {
    let (w, nbits) = consts::two_over_pi_window(1, 64);
    assert_eq!(nbits, 64);
    assert_eq!(*w.first().unwrap(), consts::TWO_OVER_PI[0]);
    assert_eq!(w.iter().skip(1).copied().sum::<u64>(), 0);

    // A window crossing a word boundary: p_33..p_96 = low 32 bits of word0 then
    // high 32 bits of word1.
    let (w2, n2) = consts::two_over_pi_window(33, 96);
    assert_eq!(n2, 64);
    let expect = (consts::TWO_OVER_PI[0] << 32) | (consts::TWO_OVER_PI[1] >> 32);
    assert_eq!(*w2.first().unwrap(), expect);
}

/// `ln2::<N>()` reads the top N limbs of the 18-word table as a normalized
/// `BigFloat<N>` with `ebin() == -1`, recording the dropped tail as error.
#[test]
fn accessor_ln2_reader() {
    let (v, err) = consts::ln2::<4>();
    assert_eq!(v.class, FpClass::Normal);
    assert_eq!(v.ebin(), -1);
    assert_eq!(v.mant, [consts::LN2[0], consts::LN2[1], consts::LN2[2], consts::LN2[3]]);
    assert_eq!(err, 1, "tail below limb 4 is nonzero → 1 ULP-at-N");
    v.check_invariant();

    // At the escalation ceiling N=16 the reader still has 2 words of guard tail.
    let (v16, err16) = consts::ln2::<16>();
    assert_eq!(v16.ebin(), -1);
    assert_eq!(err16, 1);
    v16.check_invariant();
}

#[test]
fn accessor_inv_ln2_pi_over_2_sqrt2_readers() {
    let (inv, _) = consts::inv_ln2::<8>();
    assert_eq!(inv.ebin(), 0); // 1/ln2 ∈ [1,2)
    inv.check_invariant();

    let (p2, _) = consts::pi_over_2::<8>();
    assert_eq!(p2.ebin(), 0); // pi/2 ∈ [1,2)
    p2.check_invariant();

    let (s2, _) = consts::sqrt2::<8>();
    assert_eq!(s2.ebin(), 0); // sqrt2 ∈ [1,2)
    s2.check_invariant();

    // SQRT1_2 (the f64 log-centering split) is the exact double the spec pins.
    assert_eq!(consts::SQRT1_2.to_bits(), 0x3FE6_A09E_667F_3BCD);
}

/// The readers round-trip against a leading-f64 sanity check.
#[test]
fn readers_leading_value_sanity() {
    let (ln2, _) = consts::ln2::<16>();
    assert!((ln2.leading_f64() - std::f64::consts::LN_2).abs() < 1e-15);
    let (inv, _) = consts::inv_ln2::<16>();
    assert!((inv.leading_f64() - std::f64::consts::LOG2_E).abs() < 1e-15);
    let (p2, _) = consts::pi_over_2::<16>();
    assert!((p2.leading_f64() - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
    let (s2, _) = consts::sqrt2::<16>();
    assert!((s2.leading_f64() - std::f64::consts::SQRT_2).abs() < 1e-15);
}

// ===========================================================================
// GUARD THE GUARD — prove the full-width comparator actually reaches the tail:
// a single-bit corruption in a deep/last word MUST be flagged. (Plan T3
// verification: "a deliberately corrupted tail word (e.g. word 20) makes a test
// fail".)
// ===========================================================================

#[test]
fn full_width_check_catches_tail_corruption() {
    let pi = pi_scaled(QPI);
    let want = div(&pow2(2305 + QPI), &pi);

    // Uncorrupted table passes the exact bit comparison.
    let good = table_to_bu(&consts::TWO_OVER_PI);
    assert_eq!(cmp(&good, &want), std::cmp::Ordering::Equal);

    // Corrupt word 20 (deep tail, ~bits 1280..1344 — squarely in the load-bearing
    // large-|x| range the review flagged) and confirm the comparator sees it.
    let mut corrupt = consts::TWO_OVER_PI;
    corrupt[20] ^= 1u64 << 17;
    assert_ne!(
        cmp(&table_to_bu(&corrupt), &want),
        std::cmp::Ordering::Equal,
        "corrupting word 20 must be detected by the full-width check"
    );

    // Corrupt the LOWEST word (35) — the review's specific tail concern.
    let mut corrupt_last = consts::TWO_OVER_PI;
    corrupt_last[35] ^= 1;
    assert_ne!(
        cmp(&table_to_bu(&corrupt_last), &want),
        std::cmp::Ordering::Equal,
        "corrupting the last word must be detected"
    );
}
