//! Plan B Slice-0 Task T3 — `hp::consts`: the wide, hard-coded mathematical
//! constant tables consumed by the argument-reduction kernel (T4) and the
//! transcendental atoms (T5/T6).
//!
//! # What is stored
//!
//! | Constant     | Form                                  | Words | Bits | ebin |
//! |--------------|---------------------------------------|-------|------|------|
//! | `TWO_OVER_PI`| raw fractional bit-string of `2/π`    | 36    | 2304 | n/a  |
//! | `LN2`        | normalized significand of `ln2`       | 18    | 1152 | −1   |
//! | `INV_LN2`    | normalized significand of `1/ln2`     | 18    | 1152 |  0   |
//! | `PI_OVER_2`  | normalized significand of `π/2`       | 18    | 1152 |  0   |
//! | `SQRT2`      | normalized significand of `√2`        | 18    | 1152 |  0   |
//!
//! All are stored MSW-first (`word[0]` most significant) and **floor-truncated**
//! (stored value ≤ true value) — the reduction error analysis (spec §2.8/§3)
//! depends on the discarded tail being one-sided and `< 1` ULP-at-full-width.
//!
//! # Sizing — the 1024-bit escalation ceiling (OVERRIDES the design doc)
//!
//! The design doc's "~1280-bit 2/π / 256-bit ln2" figures are sized for the
//! 256-bit level ONLY. Because a Ziv straddle re-evaluates the reduction at
//! 512/1024 bits, the tables MUST be provisioned for the maximum escalation
//! (`T_max = 1024`). Per the reduction spec's §1.2 formulas:
//!   * `2/π` window worst case: `i1_max = E_max(971) + (T_max(1024)+G_C(128)) +
//!     p(53) = 2176 ≤ 2304` (36 words, 128-bit margin).
//!   * `ln2` needs `≥ T_max + 11 + guard ≈ 1099` bits ⇒ 18 words (1152 bits).
//!
//! A working-width-`N` computation reads the top `N` limbs; the extra low limbs
//! are the guard the escalation consumes.
//!
//! # Provenance & verification
//!
//! Literals minted by `tools/gen_constants.py` (dev-time, NOT shipped): computed
//! two mutually-independent ways — mpmath at 3200-bit precision AND a pure-Python
//! library-free big-integer Machin(π)/atanh(ln2) — required to agree bit-for-bit,
//! with the published fdlibm `ipio2` / libm high-words asserted as anchors.
//!
//! The SHIPPED crate re-verifies every stored bit from scratch: `tests/hp_consts.rs`
//! recomputes each constant in a raw unbounded big-integer (Machin / atanh / integer
//! `isqrt`) — a DIFFERENT algorithm and codebase than both the generator and
//! `BigFloat` — and asserts the full 2304-/1152-bit width, closing the reduction
//! review's CRITICAL (the prior self-check ran in `BigFloat<16>` = 1024 bits, blind
//! below ~960). The independent SHARED-error catcher is T13's mpmath/MPFR validator.

use super::{BigFloat, FpClass};

// ---- width / guard scalars -------------------------------------------------

/// Total stored bits of the `2/π` Payne–Hanek table (`== TWO_OVER_PI.len()*64`).
/// The reduction's hard table-length guard is `assert!(i1 <= TWO_OVER_PI_BITS)`.
pub const TWO_OVER_PI_BITS: usize = 2304;
/// Stored bits of the `ln2` significand.
pub const LN2_BITS: usize = 1152;
/// Stored bits of the `1/ln2` significand.
pub const INV_LN2_BITS: usize = 1152;
/// Stored bits of the `π/2` significand.
pub const PI_OVER_2_BITS: usize = 1152;
/// Stored bits of the `√2` significand.
pub const SQRT2_BITS: usize = 1152;
/// Maximum escalation width in limbs (1024-bit ceiling); the tables are sized
/// for this level.
pub const MAX_L: usize = 16;
/// Trig cancellation + rounding guard (bits), used by the reduction window
/// `F = 64·L + G_C`. Covers the documented ~61–67-bit f64 trig worst-case.
pub const G_C: usize = 128;

// ---- MSB binary exponent (`ebin`) of each significand table ----------------

/// `ln2 ≈ 0.6931 ∈ [2^-1, 2^0)`.
pub const LN2_EBIN: i32 = -1;
/// `1/ln2 = log2(e) ≈ 1.4427 ∈ [2^0, 2^1)`.
pub const INV_LN2_EBIN: i32 = 0;
/// `π/2 ≈ 1.5708 ∈ [2^0, 2^1)`.
pub const PI_OVER_2_EBIN: i32 = 0;
/// `√2 ≈ 1.4142 ∈ [2^0, 2^1)`.
pub const SQRT2_EBIN: i32 = 0;

/// `1/√2 = √2/2` as the exact f64 the log reduction's octave split uses to
/// center `m ∈ [√2/2, √2)` (spec §4). Bit pattern `0x3FE6A09E667F3BCD`; the
/// stdlib constant is bit-identical (asserted in the T3 tests) and keeps this
/// MSRV-1.77-safe (`f64::from_bits` is only `const` since 1.83).
pub const SQRT1_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// Number of words in each significand table.
const SIG_WORDS: usize = 18;

// Compile-time sanity: the width scalars and array lengths cannot drift apart.
const _: () = assert!(TWO_OVER_PI.len() * 64 == TWO_OVER_PI_BITS);
const _: () = assert!(LN2.len() == SIG_WORDS && LN2.len() * 64 == LN2_BITS);
const _: () = assert!(INV_LN2.len() == SIG_WORDS);
const _: () = assert!(PI_OVER_2.len() == SIG_WORDS);
const _: () = assert!(SQRT2.len() == SIG_WORDS);

// ===========================================================================
// The tables.
//
// Minted by tools/gen_constants.py (mpmath 1.4.1 + library-free pure-int, in
// agreement; anchors verified against fdlibm ipio2 / libm high-words). Stored
// MSW-first, floor-truncated.
// ===========================================================================

/// `2/π` as a raw fractional bit-string: `p_i` (coefficient of `2^-i`) is bit
/// `63-((i-1) mod 64)` of `word[(i-1)/64]`; `p_1` (the `2^-1` bit) is the MSB of
/// `word[0]`. Equivalently the 2304-bit integer `floor((2/π)·2^2304)`. This is
/// the Payne–Hanek table (a long bit-string windowed by the exponent of `x`), NOT
/// a `BigFloat` significand.
///
/// Source: the fdlibm `__kernel_rem_pio2` `ipio2[]` 2/π table (24-bit words),
/// repacked into 64-bit words and extended to 2304 bits from the same computation
/// (Machin π ⇒ `2/π`). Words 0–2 are the pinned fdlibm anchors
/// `0xA2F9836E4E441529, 0xFC2757D1F534DDC0, 0xDB6295993C439041`.
pub const TWO_OVER_PI: [u64; 36] = [
    0xA2F9836E4E441529, 0xFC2757D1F534DDC0, 0xDB6295993C439041,
    0xFE5163ABDEBBC561, 0xB7246E3A424DD2E0, 0x06492EEA09D1921C,
    0xFE1DEB1CB129A73E, 0xE88235F52EBB4484, 0xE99C7026B45F7E41,
    0x3991D639835339F4, 0x9C845F8BBDF9283B, 0x1FF897FFDE05980F,
    0xEF2F118B5A0A6D1F, 0x6D367ECF27CB09B7, 0x4F463F669E5FEA2D,
    0x7527BAC7EBE5F17B, 0x3D0739F78A5292EA, 0x6BFB5FB11F8D5D08,
    0x56033046FC7B6BAB, 0xF0CFBC209AF4361D, 0xA9E391615EE61B08,
    0x6599855F14A06840, 0x8DFFD8804D732731, 0x06061556CA73A8C9,
    0x60E27BC08C6B47C4, 0x19C367CDDCE8092A, 0x8359C4768B961CA6,
    0xDDAF44D15719053E, 0xA5FF07053F7E33E8, 0x32C2DE4F98327DBB,
    0xC33D26EF6B1E5EF8, 0x9F3A1F35CAF27F1D, 0x87F121907C7C246A,
    0xFA6ED5772D30433B, 0x15C614B59D19C3C2, 0xC4AD414D2C5D000C,
];

/// `ln2` normalized significand (value `= LN2 · 2^(-1 - 1151)`, i.e. `ebin = -1`).
/// Source: standard libm/CRlibm extended `ln2`; word0 anchor `0xB17217F7D1CF79AB`.
pub const LN2: [u64; 18] = [
    0xB17217F7D1CF79AB, 0xC9E3B39803F2F6AF, 0x40F343267298B62D,
    0x8A0D175B8BAAFA2B, 0xE7B876206DEBAC98, 0x559552FB4AFA1B10,
    0xED2EAE35C1382144, 0x27573B291169B825, 0x3E96CA16224AE8C5,
    0x1ACBDA11317C387E, 0xB9EA9BC3B136603B, 0x256FA0EC7657F74B,
    0x72CE87B19D6548CA, 0xF5DFA6BD38303248, 0x655FA1872F20E3A2,
    0xDA2D97C50F3FD5C6, 0x07F4CA11FB5BFB90, 0x610D30F88FE551A2,
];

/// `1/ln2 = log2(e)` normalized significand (`ebin = 0`).
/// Word0 = `0xB8AA3B295C17F0BB` — the FLOOR of the leading 64 bits. NOTE: the
/// widely-published 64-bit `log2(e)` word is `0x…F0BC` (round-to-nearest); a
/// full-width floor-truncated table has `…F0BB` (its 65th bit is 1). Floor keeps
/// the one-sided reduction-error bound and is consistent with LN2 / PI_OVER_2.
pub const INV_LN2: [u64; 18] = [
    0xB8AA3B295C17F0BB, 0xBE87FED0691D3E88, 0xEB577AA8DD695A58,
    0x8B25166CD1A13247, 0xDE1C43F755176CD6, 0x24D92F75C16BE0B3,
    0xEA90B9E60C4A909F, 0xC4BFAF0353DF39B3, 0x2FE294932617D9D5,
    0xB21B43D579D5A206, 0x0B5EBBBF3A828546, 0x8D1CF457AB63253C,
    0x199A94836F5B4967, 0x278CCF084679C940, 0xCE7E20358CD5DB8F,
    0x612F08FBAE30A173, 0x2650B6D1058EBA50, 0x9638C84C5A02065F,
];

/// `π/2` normalized significand (`ebin = 0`).
/// Word0/1 anchors `0xC90FDAA22168C234, 0xC4C6628B80DC1CD1` (the standard π hex).
pub const PI_OVER_2: [u64; 18] = [
    0xC90FDAA22168C234, 0xC4C6628B80DC1CD1, 0x29024E088A67CC74,
    0x020BBEA63B139B22, 0x514A08798E3404DD, 0xEF9519B3CD3A431B,
    0x302B0A6DF25F1437, 0x4FE1356D6D51C245, 0xE485B576625E7EC6,
    0xF44C42E9A637ED6B, 0x0BFF5CB6F406B7ED, 0xEE386BFB5A899FA5,
    0xAE9F24117C4B1FE6, 0x49286651ECE45B3D, 0xC2007CB8A163BF05,
    0x98DA48361C55D39A, 0x69163FA8FD24CF5F, 0x83655D23DCA3AD96,
];

/// `√2` normalized significand (`ebin = 0`). Word0 anchor `0xB504F333F9DE6484`.
pub const SQRT2: [u64; 18] = [
    0xB504F333F9DE6484, 0x597D89B3754ABE9F, 0x1D6F60BA893BA84C,
    0xED17AC8583339915, 0x4AFC83043AB8A2C3, 0xA8B1FE6FDC83DB39,
    0x0F74A85E439C7B4A, 0x780487363DFA2768, 0xD2202E8742AF1F4E,
    0x53059C6011BC337B, 0xCAB1BC911688458A, 0x460ABC722F7C4E33,
    0xC6D5A8A38BB7E9DC, 0xCB2A634331F3C84D, 0xF52F120F836E582E,
    0xEAA4A0899040CA4A, 0x81394AB6D8FD0EFD, 0xF4D3A02CEBC93E0C,
];

// ===========================================================================
// Readers — how a working width `N` obtains each constant. These are exactly
// the accessors the reduction kernel (T4) calls.
// ===========================================================================

/// Read an 18-word significand table at working width `N` (`N ≤ 16`): keeps the
/// top `N` limbs, records the dropped tail as `err_ulps` (1 ULP-at-`N` since the
/// tail is always nonzero), preserves the width-invariant `ebin`. Reuses the
/// tested [`BigFloat::narrow`]. The caller (a reduction/atom) folds `err` into
/// its Ziv error budget; escalation to a wider `N` reads more of the guard tail,
/// shrinking the error.
fn read_sig<const N: usize>(table: &[u64; SIG_WORDS], ebin: i32) -> (BigFloat<N>, u128) {
    let wide = BigFloat::<SIG_WORDS>::from_limbs_ebin(false, ebin, *table, FpClass::Normal);
    wide.narrow::<N>()
}

/// `ln2` at working width `N` as `(BigFloat<N>, err_ulps)`, `ebin = -1`.
pub fn ln2<const N: usize>() -> (BigFloat<N>, u128) {
    read_sig::<N>(&LN2, LN2_EBIN)
}

/// `1/ln2 = log2(e)` at working width `N`, `ebin = 0`.
pub fn inv_ln2<const N: usize>() -> (BigFloat<N>, u128) {
    read_sig::<N>(&INV_LN2, INV_LN2_EBIN)
}

/// `π/2` at working width `N`, `ebin = 0`.
pub fn pi_over_2<const N: usize>() -> (BigFloat<N>, u128) {
    read_sig::<N>(&PI_OVER_2, PI_OVER_2_EBIN)
}

/// `√2` at working width `N`, `ebin = 0`.
pub fn sqrt2<const N: usize>() -> (BigFloat<N>, u128) {
    read_sig::<N>(&SQRT2, SQRT2_EBIN)
}

/// The `2/π` coefficient `p_i` (the bit weighting `2^-i`), `i ≥ 1`: bit
/// `63-((i-1) mod 64)` of `word[(i-1)/64]`. Returns 0 for `i` past the stored
/// table (the reduction guards against ever needing those via
/// `assert!(i1 <= TWO_OVER_PI_BITS)`).
pub fn two_over_pi_bit(i: usize) -> u64 {
    debug_assert!(i >= 1, "2/pi bit index is 1-based (p_1 is the 2^-1 bit)");
    let idx = i - 1;
    let word = idx / 64;
    if word >= TWO_OVER_PI.len() {
        return 0;
    }
    (TWO_OVER_PI[word] >> (63 - (idx % 64))) & 1
}

/// The contiguous `2/π` bit-window `p_{i0..=i1}` read MSB-first as an unsigned
/// big integer of value `Σ_{i=i0}^{i1} p_i · 2^(i1-i)`, returned LSW-first (index
/// 0 = least significant limb) together with the bit count `i1-i0+1`. This is the
/// Payne–Hanek `win` of reduction spec §2.4 — pure bit-plumbing, no arithmetic.
/// The reduction then multiplies it by the integer significand `M` of `x`.
pub fn two_over_pi_window(i0: usize, i1: usize) -> (Vec<u64>, usize) {
    assert!(i0 >= 1 && i1 >= i0, "invalid 2/pi window [{i0}, {i1}]");
    // Defense-in-depth (T3-review Minor): the reduction kernel already asserts
    // `i1 <= TWO_OVER_PI_BITS` before calling, but re-check it at the table's own
    // edge so a future index bug is a LOUD panic rather than a silently-wrong
    // large-|x| trig value (the accessor otherwise zero-fills past the table,
    // which is exactly the "systematic, Ziv-blind" failure §5 names as most
    // dangerous). See reduction spec §2.3 / §5.
    assert!(
        i1 <= TWO_OVER_PI_BITS,
        "2/pi window past the stored table: requested bit {i1}, table is {TWO_OVER_PI_BITS} bits"
    );
    let nbits = i1 - i0 + 1;
    let mut out = vec![0u64; nbits.div_ceil(64)];
    for i in i0..=i1 {
        if two_over_pi_bit(i) != 0 {
            let pos = i1 - i; // 0 = LSB
            out[pos / 64] |= 1u64 << (pos % 64);
        }
    }
    (out, nbits)
}
