//! Plan B Slice-0 Task T1 — `hp-core`: a dependency-free, from-scratch,
//! spec-pinned binary big-float `BigFloat<N>` (N in {4,8,16} => 256/512/1024
//! bit working precision P = 64*N). This is the numeric substrate every later
//! Slice-0 kernel (round-ziv, constants, reduction, exp/log/sin atoms) builds on.
//!
//! Non-normative reference code (KISS-Conform §0). Stdlib only: a conformance
//! harness shares no lowering code with any implementation under test, and this
//! crate ships dependency-free.
//!
//! # Representation (round-ziv §0 substrate — normative for this suite)
//!
//! ```text
//! value = (-1)^sign * (mant as P-bit integer) * 2^exp        (P = 64*N)
//! ```
//!
//! * `sign: bool` — true = negative (carried by Normal and Zero).
//! * `exp: i32` — the **LSB weight**: the power of two of the mantissa's least
//!   significant bit. (NOT the MSB binary exponent; see [`BigFloat::ebin`].)
//! * `mant: [u64; N]` — big-endian limbs, `mant[0]` MOST significant. For a
//!   Normal value the mantissa is NORMALIZED: bit `P-1` is set, i.e.
//!   `mant[0] & 0x8000_0000_0000_0000 != 0`, so `mant` in `[2^(P-1), 2^P)`.
//! * `class: FpClass` — Normal / Zero (signed) / Inf / NaN.
//!
//! The atom -> round-ziv error contract is `|val - v_true| <= err_ulps * 2^exp`,
//! valid only in this LSB-weight convention. Arithmetic ops return the LOCAL
//! rounding error they introduce (0 if the result is exact, else 1 as an integer
//! bound on the true <= 0.5-ULP round); an atom sums input errors (rescaled to
//! the result ULP) plus these across a chain — see the module tests and §9 of the
//! design spec.
//!
//! The MSB-exponent sibling specs (reduction/atan) map mechanically onto this
//! form via [`BigFloat::ebin`] (= `exp + P - 1`), [`BigFloat::from_limbs_ebin`],
//! and [`BigFloat::sign_i8`].

use std::cmp::Ordering;

pub mod atoms;
pub mod consts;
pub mod reduction;

pub use atoms::Exp;

/// Floating-point class. Zero carries a sign; Inf/NaN only arise from the atom /
/// semantics layer (they are canonicalized to an all-zero mantissa here).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FpClass {
    Normal,
    Zero,
    Inf,
    NaN,
}

/// A binary big-float in the round-ziv §0 substrate convention. `N` limbs =>
/// working precision `P = 64*N` bits. See the module docs for the exact value
/// semantics and invariants.
#[derive(Clone, Debug)]
pub struct BigFloat<const N: usize> {
    /// true = negative (meaningful for Normal and Zero).
    pub sign: bool,
    /// LSB weight: `value = (-1)^sign * (mant as P-bit int) * 2^exp`.
    pub exp: i32,
    /// Big-endian limbs, `mant[0]` most significant; normalized (bit P-1 set)
    /// for Normal. Doc-alias: reduction's `limbs`, atan's `sig`.
    pub mant: [u64; N],
    pub class: FpClass,
}

/// The atom -> round-ziv hand-off: a value plus its accumulated error, in units
/// of the value's ULP (`2^val.exp`). Contract: `|val - v_true| <= err_ulps * 2^val.exp`.
#[derive(Clone, Debug)]
pub struct Evaluated<const N: usize> {
    pub val: BigFloat<N>,
    pub err_ulps: u128,
}

// ===========================================================================
// LSW-first big-integer primitives.
//
// The stored `mant` is MSW-first (spec-pinned), but every arithmetic kernel
// converts to a little-endian ("LSW-first", index 0 = least significant limb)
// magnitude vector, because the standard shift/add/sub/mul/divmod algorithms are
// naturally written that way. Conversions happen only at the boundaries.
// ===========================================================================

/// mant (MSW-first) -> LSW-first magnitude vector of exactly N limbs.
fn to_lsw<const N: usize>(mant: &[u64; N]) -> Vec<u64> {
    let mut v = mant.to_vec();
    v.reverse();
    v
}

/// Low N limbs of an LSW-first buffer -> MSW-first `[u64; N]` (`out[0]` = most
/// significant = `buf[N-1]`). Higher limbs of `buf` are ignored by the caller.
fn lsw_low_to_msw<const N: usize>(buf: &[u64]) -> [u64; N] {
    let mut out = [0u64; N];
    for (i, o) in out.iter_mut().enumerate() {
        let idx = N - 1 - i;
        *o = if idx < buf.len() { buf[idx] } else { 0 };
    }
    out
}

fn lsw_is_zero(a: &[u64]) -> bool {
    a.iter().all(|&x| x == 0)
}

/// Compare two LSW-first magnitudes (differing lengths allowed; missing high
/// limbs read as zero).
fn lsw_cmp(a: &[u64], b: &[u64]) -> Ordering {
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

/// `a += b` (LSW-first). `b` may be shorter than `a`. Returns the carry out.
fn lsw_add(a: &mut [u64], b: &[u64]) -> u64 {
    let mut carry = 0u64;
    for i in 0..a.len() {
        let bi = if i < b.len() { b[i] } else { 0 };
        let (s1, c1) = a[i].overflowing_add(bi);
        let (s2, c2) = s1.overflowing_add(carry);
        a[i] = s2;
        carry = (c1 as u64) + (c2 as u64);
    }
    carry
}

/// `a -= b` (LSW-first), assuming `a >= b`. `b` may be shorter. Returns the final
/// borrow (0 when `a >= b`).
fn lsw_sub(a: &mut [u64], b: &[u64]) -> u64 {
    let mut borrow = 0u64;
    for i in 0..a.len() {
        let bi = if i < b.len() { b[i] } else { 0 };
        let (d1, b1) = a[i].overflowing_sub(bi);
        let (d2, b2) = d1.overflowing_sub(borrow);
        a[i] = d2;
        borrow = (b1 as u64) + (b2 as u64);
    }
    borrow
}

/// Index of the most significant set bit, or None if the value is zero.
fn msb_index(a: &[u64]) -> Option<usize> {
    for i in (0..a.len()).rev() {
        if a[i] != 0 {
            return Some(i * 64 + (63 - a[i].leading_zeros() as usize));
        }
    }
    None
}

/// Bit `idx` (0 = LSB) of an LSW-first magnitude.
fn lsw_bit(a: &[u64], idx: usize) -> u64 {
    let w = idx / 64;
    if w < a.len() {
        (a[w] >> (idx % 64)) & 1
    } else {
        0
    }
}

/// Is any set bit strictly below position `k` present?
fn any_bits_below(a: &[u64], k: usize) -> bool {
    let words = k / 64;
    let bits = k % 64;
    if a[..words.min(a.len())].iter().any(|&x| x != 0) {
        return true;
    }
    if bits != 0 && words < a.len() && a[words] & ((1u64 << bits) - 1) != 0 {
        return true;
    }
    false
}

/// Zero every bit at position `>= k` (0 = LSB) in an LSW-first buffer, keeping
/// only the low `k` bits. `k == 64*len` keeps everything (no-op). Used by
/// round-ziv §4 to isolate the fractional field `F = m mod 2^shift`.
fn lsw_mask_low(a: &mut [u64], k: usize) {
    let words = k / 64;
    let bits = k % 64;
    for (i, limb) in a.iter_mut().enumerate() {
        if i < words {
            // fully retained
        } else if i == words {
            *limb &= if bits == 0 { 0 } else { (1u64 << bits) - 1 };
        } else {
            *limb = 0;
        }
    }
}

/// Set bit `k` (0 = LSB) in an LSW-first buffer (a no-op if `k` is out of range).
/// Used by round-ziv §4 to form `Fmid = 2^(shift-1)`.
fn lsw_set_bit(a: &mut [u64], k: usize) {
    let w = k / 64;
    if w < a.len() {
        a[w] |= 1u64 << (k % 64);
    }
}

/// `a <<= k` (LSW-first), dropping bits shifted out the top of the fixed-length
/// buffer. Value = `a * 2^k` when the buffer is wide enough.
fn lsw_shl(a: &mut [u64], k: u32) {
    if k == 0 {
        return;
    }
    let len = a.len();
    let words = (k / 64) as usize;
    let bits = k % 64;
    let src = a.to_vec();
    for i in (0..len).rev() {
        let mut val = 0u64;
        if i >= words {
            val = src[i - words] << bits;
        }
        if bits != 0 && i > words {
            val |= src[i - words - 1] >> (64 - bits);
        }
        a[i] = val;
    }
}

/// `a >>= k` (LSW-first), dropping the low bits.
fn lsw_shr(a: &mut [u64], k: u32) {
    if k == 0 {
        return;
    }
    let len = a.len();
    let words = (k / 64) as usize;
    let bits = k % 64;
    let src = a.to_vec();
    for i in 0..len {
        let mut val = 0u64;
        if i + words < len {
            val = src[i + words] >> bits;
        }
        if bits != 0 && i + words + 1 < len {
            val |= src[i + words + 1] << (64 - bits);
        }
        a[i] = val;
    }
}

/// Schoolbook product of two LSW-first magnitudes; result length `a.len()+b.len()`.
fn lsw_mul(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut out = vec![0u64; a.len() + b.len()];
    for i in 0..a.len() {
        if a[i] == 0 {
            continue;
        }
        let mut carry = 0u64;
        for j in 0..b.len() {
            let t = (a[i] as u128) * (b[j] as u128) + (out[i + j] as u128) + (carry as u128);
            out[i + j] = t as u64;
            carry = (t >> 64) as u64;
        }
        let mut k = i + b.len();
        while carry != 0 {
            let t = (out[k] as u128) + (carry as u128);
            out[k] = t as u64;
            carry = (t >> 64) as u64;
            k += 1;
        }
    }
    out
}

/// Bit-at-a-time **restoring long division** of two LSW-first magnitudes.
/// Returns `(quotient, remainder)` with `num = q*den + r`, `0 <= r < den`.
/// This is the normative airtight-error division kernel (design spec §6): the
/// final nonzero remainder is exactly the sticky signal for correct rounding.
fn lsw_divmod(num: &[u64], den: &[u64]) -> (Vec<u64>, Vec<u64>) {
    debug_assert!(!lsw_is_zero(den), "division by zero magnitude");
    let nbits = match msb_index(num) {
        Some(m) => m + 1,
        None => return (vec![0u64; 1], vec![0u64; den.len()]), // num == 0
    };
    // Remainder is always < den, so den.len()+1 limbs hold it through the <<1.
    let mut rem = vec![0u64; den.len() + 1];
    let mut q = vec![0u64; nbits / 64 + 1];
    for i in (0..nbits).rev() {
        lsw_shl(&mut rem, 1);
        rem[0] |= (num[i / 64] >> (i % 64)) & 1;
        if lsw_cmp(&rem, den) != Ordering::Less {
            lsw_sub(&mut rem, den);
            q[i / 64] |= 1u64 << (i % 64);
        }
    }
    (q, rem)
}

/// MSW-first increment by one; returns the carry out of the top limb.
fn msw_inc<const N: usize>(m: &mut [u64; N]) -> bool {
    for i in (0..N).rev() {
        let (v, c) = m[i].overflowing_add(1);
        m[i] = v;
        if !c {
            return false;
        }
    }
    true
}

/// Round a magnitude buffer to a P-bit normalized mantissa, RNE.
///
/// `mag` (LSW-first) has its bit 0 at weight `2^exp_lsb`; `extra_sticky` records
/// nonzero content strictly below bit 0 (e.g. a division remainder or bits lost
/// aligning an addend). Returns `(mant MSW-first, exp, inexact)`, or None when
/// `mag` is exactly zero (the caller makes a signed zero).
fn normalize_round<const N: usize>(
    mag: &[u64],
    exp_lsb: i32,
    extra_sticky: bool,
) -> Option<([u64; N], i32, bool)> {
    let p = (64 * N) as i32;
    let msb = msb_index(mag)? as i32;
    let drop = msb - (p - 1);

    if drop <= 0 {
        // Fewer than P significant bits: left-shift into place, exact except for
        // any content already flagged strictly below (extra_sticky).
        let sh = (-drop) as u32;
        let mut buf = mag.to_vec();
        if buf.len() < N + 1 {
            buf.resize(N + 1, 0);
        }
        lsw_shl(&mut buf, sh);
        let mant = lsw_low_to_msw::<N>(&buf);
        return Some((mant, exp_lsb + drop, extra_sticky));
    }

    let d = drop as u32;
    let round_bit = lsw_bit(mag, (d - 1) as usize) == 1;
    let sticky = extra_sticky || any_bits_below(mag, (d - 1) as usize);

    let mut buf = mag.to_vec();
    if buf.len() < N + 1 {
        buf.resize(N + 1, 0);
    }
    lsw_shr(&mut buf, d);
    let mut mant = lsw_low_to_msw::<N>(&buf);

    let lsb_odd = (mant[N - 1] & 1) == 1;
    let inexact = round_bit || sticky;
    let mut exp = exp_lsb + drop;
    if round_bit && (sticky || lsb_odd) && msw_inc(&mut mant) {
        // Carried out of bit P-1 (mantissa became 2^P): renormalize.
        mant = [0u64; N];
        mant[0] = 1u64 << 63;
        exp += 1;
    }
    Some((mant, exp, inexact))
}

// ===========================================================================
// BigFloat<N> API.
// ===========================================================================

impl<const N: usize> BigFloat<N> {
    /// Working precision in bits, `P = 64*N`.
    pub const P: usize = 64 * N;

    // ---- constructors ------------------------------------------------------

    fn make_zero(sign: bool) -> Self {
        BigFloat { sign, exp: 0, mant: [0u64; N], class: FpClass::Zero }
    }
    fn make_inf(sign: bool) -> Self {
        BigFloat { sign, exp: 0, mant: [0u64; N], class: FpClass::Inf }
    }
    fn make_nan() -> Self {
        BigFloat { sign: false, exp: 0, mant: [0u64; N], class: FpClass::NaN }
    }

    /// Build a normalized Normal from a raw significand `sig` (with its own bit
    /// length) whose LSB has weight `2^e2`. `sig` must be nonzero and fit in 64
    /// bits (all IEEE significands do). Exact.
    fn from_sig64(sign: bool, sig: u64, e2: i32) -> Self {
        debug_assert!(sig != 0);
        let l = 64 - sig.leading_zeros(); // bit length of sig
        let shift = Self::P as u32 - l; // P > 64 >= l, so positive
        let mut lsw = vec![0u64; N + 1];
        lsw[0] = sig;
        lsw_shl(&mut lsw, shift);
        BigFloat {
            sign,
            exp: e2 - shift as i32,
            mant: lsw_low_to_msw::<N>(&lsw),
            class: FpClass::Normal,
        }
    }

    /// Exact conversion from `f64` (53-bit significand fits in P). Handles
    /// normals, subnormals (clz-normalized), signed zero, ±Inf and NaN.
    pub fn from_f64(x: f64) -> Self {
        let bits = x.to_bits();
        let sign = (bits >> 63) != 0;
        let exp_field = ((bits >> 52) & 0x7ff) as i32;
        let frac = bits & 0x000f_ffff_ffff_ffff;
        if exp_field == 0x7ff {
            return if frac == 0 { Self::make_inf(sign) } else { Self::make_nan() };
        }
        if exp_field == 0 && frac == 0 {
            return Self::make_zero(sign);
        }
        let (sig, e2) = if exp_field == 0 {
            (frac, -1074) // subnormal: value = frac * 2^-1074
        } else {
            ((1u64 << 52) | frac, exp_field - 1075) // normal
        };
        Self::from_sig64(sign, sig, e2)
    }

    /// Exact conversion from `f32` (24-bit significand fits in P).
    pub fn from_f32(x: f32) -> Self {
        let bits = x.to_bits();
        let sign = (bits >> 31) != 0;
        let exp_field = ((bits >> 23) & 0xff) as i32;
        let frac = (bits & 0x7f_ffff) as u64;
        if exp_field == 0xff {
            return if frac == 0 { Self::make_inf(sign) } else { Self::make_nan() };
        }
        if exp_field == 0 && frac == 0 {
            return Self::make_zero(sign);
        }
        let (sig, e2) = if exp_field == 0 {
            (frac, -149) // subnormal
        } else {
            ((1u64 << 23) | frac, exp_field - 150) // normal
        };
        Self::from_sig64(sign, sig, e2)
    }

    /// Exact conversion from a signed 64-bit integer (`|k| < 2^63`).
    pub fn from_i64(k: i64) -> Self {
        if k == 0 {
            return Self::make_zero(false);
        }
        let sign = k < 0;
        let mag = (k as i128).unsigned_abs() as u64;
        Self::from_sig64(sign, mag, 0)
    }

    /// Exact conversion from an unsigned 64-bit integer.
    pub fn from_u64(k: u64) -> Self {
        if k == 0 {
            return Self::make_zero(false);
        }
        Self::from_sig64(false, k, 0)
    }

    /// The constant 1.0.
    pub fn one() -> Self {
        Self::from_u64(1)
    }

    /// A signed zero.
    pub fn zero(sign: bool) -> Self {
        Self::make_zero(sign)
    }

    /// Native (LSB-weight) constructor. Stores the fields verbatim — the caller
    /// is responsible for normalization when `class == Normal`. Used for
    /// constant tables and low-level construction.
    pub fn from_limbs(sign: bool, exp: i32, mant: [u64; N], class: FpClass) -> Self {
        BigFloat { sign, exp, mant, class }
    }

    /// MSB-exponent constructor (for the reduction/atan constant tables written
    /// in `[1,2)`-significand, MSB-binary-exponent form): stores
    /// `exp = ebin - (P-1)`.
    pub fn from_limbs_ebin(sign: bool, ebin: i32, mant: [u64; N], class: FpClass) -> Self {
        BigFloat { sign, exp: ebin - (Self::P as i32 - 1), mant, class }
    }

    // ---- cheap exact transforms -------------------------------------------

    /// Negate (exact). Flips the sign of Normal, Zero and Inf; NaN unchanged.
    pub fn neg(&self) -> Self {
        let mut r = self.clone();
        if r.class != FpClass::NaN {
            r.sign = !r.sign;
        }
        r
    }

    /// Absolute value (exact).
    pub fn abs(&self) -> Self {
        let mut r = self.clone();
        if r.class != FpClass::NaN {
            r.sign = false;
        }
        r
    }

    /// Multiply by `2^k` (exact): shifts the LSB weight.
    pub fn ldexp(&self, k: i32) -> Self {
        let mut r = self.clone();
        if r.class == FpClass::Normal {
            r.exp += k;
        }
        r
    }

    // ---- queries -----------------------------------------------------------

    /// True iff this is a (signed) zero.
    pub fn is_zero(&self) -> bool {
        self.class == FpClass::Zero
    }

    /// The stored LSB-weight exponent (the power of two of the mantissa's least
    /// significant bit). Width-dependent.
    pub fn exp2(&self) -> i32 {
        self.exp
    }

    /// The MSB binary exponent `Ebin = exp + (P-1)`: `|value|` in
    /// `[2^Ebin, 2^(Ebin+1))`. Width-invariant — this is what higher kernels
    /// compare across widths.
    pub fn ebin(&self) -> i32 {
        self.exp + (Self::P as i32 - 1)
    }

    /// Sign as `i8` (reduction/atan convention): -1 negative, +1 non-negative.
    /// A negative zero reports -1.
    pub fn sign_i8(&self) -> i8 {
        if self.sign {
            -1
        } else {
            1
        }
    }

    /// The top ~53 bits back to `f64` — a NON-exact seed helper (e.g. for a
    /// Newton reciprocal). Only meaningful for in-range magnitudes.
    pub fn leading_f64(&self) -> f64 {
        match self.class {
            FpClass::Zero => {
                if self.sign {
                    -0.0
                } else {
                    0.0
                }
            }
            FpClass::Inf => {
                if self.sign {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }
            FpClass::NaN => f64::NAN,
            FpClass::Normal => {
                // value ~= mant[0] * 2^(exp + 64*(N-1))
                let mut e = self.exp + 64 * (N as i32 - 1);
                let mut v = self.mant[0] as f64;
                // Scale by 2^e in bounded steps so huge/tiny e don't trip powi.
                while e > 1023 {
                    v *= 2f64.powi(1023);
                    e -= 1023;
                }
                while e < -1074 {
                    v *= 2f64.powi(-1074);
                    e += 1074;
                }
                v *= 2f64.powi(e);
                if self.sign {
                    -v
                } else {
                    v
                }
            }
        }
    }

    /// Round to the nearest `i64`, ties to even. Requires `|value| < 2^62`
    /// (large arguments are pre-clamped to ±Inf/±0 by the atom before reduction,
    /// so k-overflow never reaches here). Zero -> 0.
    pub fn round_to_i64(&self) -> i64 {
        match self.class {
            FpClass::Zero => 0,
            FpClass::Normal => {
                let lsw = to_lsw(&self.mant);
                let m: u64 = if self.exp >= 0 {
                    let mut buf = lsw.clone();
                    buf.resize(N + 2, 0);
                    lsw_shl(&mut buf, self.exp as u32);
                    buf[0]
                } else {
                    let shift = (-self.exp) as u32;
                    let round_bit = if shift >= 1 { lsw_bit(&lsw, (shift - 1) as usize) } else { 0 };
                    let sticky = shift >= 2 && any_bits_below(&lsw, (shift - 1) as usize);
                    let mut buf = lsw.clone();
                    buf.resize(N + 2, 0);
                    lsw_shr(&mut buf, shift);
                    let mut mm = buf[0];
                    if round_bit == 1 && (sticky || (mm & 1) == 1) {
                        mm += 1;
                    }
                    mm
                };
                if self.sign {
                    -(m as i64)
                } else {
                    m as i64
                }
            }
            // Pre-clamped away by the atom; be defensive rather than panic.
            FpClass::Inf | FpClass::NaN => 0,
        }
    }

    // ---- comparison + exact midpoint distance ------------------------------

    /// Sign-aware total order. `+0 == -0`. NaN sorts as the greatest value (so
    /// `cmp` stays total for sorting; NaN is out of round-ziv's domain anyway).
    ///
    /// Spec-pinned name (design spec §10); `BigFloat` deliberately does not
    /// implement `Ord` (NaN/signed-zero make the derive wrong).
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(&self, other: &Self) -> Ordering {
        if self.class == FpClass::NaN || other.class == FpClass::NaN {
            return match (self.class == FpClass::NaN, other.class == FpClass::NaN) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => unreachable!(),
            };
        }
        // Signed value ordering: value < 0 iff negative and not a zero.
        let a_sig = if self.class == FpClass::Zero {
            0
        } else if self.sign {
            -1
        } else {
            1
        };
        let b_sig = if other.class == FpClass::Zero {
            0
        } else if other.sign {
            -1
        } else {
            1
        };
        if a_sig != b_sig {
            return a_sig.cmp(&b_sig);
        }
        if a_sig == 0 {
            return Ordering::Equal; // both zero, sign-agnostic
        }
        let mag = self.cmp_mag(other);
        if a_sig > 0 {
            mag
        } else {
            mag.reverse()
        }
    }

    /// Magnitude compare of two non-zero, non-NaN values.
    fn cmp_mag(&self, other: &Self) -> Ordering {
        match (self.class, other.class) {
            (FpClass::Inf, FpClass::Inf) => Ordering::Equal,
            (FpClass::Inf, _) => Ordering::Greater,
            (_, FpClass::Inf) => Ordering::Less,
            (FpClass::Normal, FpClass::Normal) => {
                let (ea, eb) = (self.ebin(), other.ebin());
                if ea != eb {
                    return ea.cmp(&eb);
                }
                // Same ebin => same exp (both normalized) => compare mant MSW-first.
                for i in 0..N {
                    if self.mant[i] != other.mant[i] {
                        return self.mant[i].cmp(&other.mant[i]);
                    }
                }
                Ordering::Equal
            }
            _ => Ordering::Equal, // zeros handled by cmp()
        }
    }

    /// EXACT big-integer `|a - b|` of the two mantissas, aligned to a common LSB
    /// weight, returned as the low N limbs (MSW-first). This is round-ziv §4's
    /// distance-to-midpoint `D = abs_diff(F, Fmid)`; D is never truncated (F and
    /// Fmid share an exponent by construction, so the low N limbs are exact).
    pub fn abs_diff_limbs(&self, other: &Self) -> [u64; N] {
        // T1 hardening: the low-N return is only exact in the shared-exponent
        // regime — a nonzero exp difference shifts operand bits up and the
        // high-limb overflow is truncated by `lsw_low_to_msw`, silently
        // corrupting `D`. round-ziv §4 always builds `F` and `Fmid` at the same
        // exponent, so guard the "never truncate D" invariant here.
        debug_assert!(
            self.exp == other.exp,
            "abs_diff_limbs: operands must share an exponent (got {} vs {})",
            self.exp,
            other.exp
        );
        let de = (self.exp - other.exp).unsigned_abs();
        let extra = (de as usize) / 64 + 2;
        let w = N + extra;
        let mut a = vec![0u64; w];
        let mut b = vec![0u64; w];
        for (i, &v) in to_lsw(&self.mant).iter().enumerate() {
            a[i] = v;
        }
        for (i, &v) in to_lsw(&other.mant).iter().enumerate() {
            b[i] = v;
        }
        match self.exp.cmp(&other.exp) {
            Ordering::Greater => lsw_shl(&mut a, (self.exp - other.exp) as u32),
            Ordering::Less => lsw_shl(&mut b, (other.exp - self.exp) as u32),
            Ordering::Equal => {}
        }
        let diff = if lsw_cmp(&a, &b) == Ordering::Less {
            lsw_sub(&mut b, &a);
            b
        } else {
            lsw_sub(&mut a, &b);
            a
        };
        lsw_low_to_msw::<N>(&diff)
    }

    /// Compare the big-integer distance `d` (as produced by [`abs_diff_limbs`])
    /// against a `u128` error bound: `d > err_ulps` (strict). `d` can reach
    /// ~`2^(P-1)`, so any set bit above the low 128 exceeds any `u128`.
    ///
    /// [`abs_diff_limbs`]: Self::abs_diff_limbs
    pub fn dist_gt_err(d: &[u64; N], err_ulps: u128) -> bool {
        // MSW-first: limbs [0, N-2) are above the low 128 bits.
        if N >= 2 {
            for &limb in &d[..N - 2] {
                if limb != 0 {
                    return true;
                }
            }
            let lo = ((d[N - 2] as u128) << 64) | (d[N - 1] as u128);
            lo > err_ulps
        } else {
            (d[0] as u128) > err_ulps
        }
    }

    // ---- add / sub ---------------------------------------------------------

    /// `self - other`, correctly rounded RNE. Returns `(result, rounding_err)`
    /// where `rounding_err` is 0 (exact) or 1 (the local <= 0.5-ULP round).
    pub fn sub(&self, other: &Self) -> (Self, u128) {
        self.add(&other.neg())
    }

    /// `self + other`, correctly rounded RNE. Returns `(result, rounding_err)`
    /// (0 exact, else 1). Guard = 64 bits; content beyond the guard becomes a
    /// sticky bit. Input-error rescaling to the result ULP is the caller's job
    /// (design spec §4/§9).
    pub fn add(&self, other: &Self) -> (Self, u128) {
        use FpClass::*;
        // --- special classes ---
        match (self.class, other.class) {
            (NaN, _) | (_, NaN) => return (Self::make_nan(), 0),
            (Inf, Inf) => {
                return if self.sign == other.sign {
                    (Self::make_inf(self.sign), 0)
                } else {
                    (Self::make_nan(), 0) // Inf + (-Inf)
                };
            }
            (Inf, _) => return (Self::make_inf(self.sign), 0),
            (_, Inf) => return (Self::make_inf(other.sign), 0),
            (Zero, Zero) => {
                // IEEE round-to-nearest: -0 only when both are -0.
                return (Self::make_zero(self.sign && other.sign), 0);
            }
            (Zero, _) => return (other.clone(), 0),
            (_, Zero) => return (self.clone(), 0),
            (Normal, Normal) => {}
        }

        // T1 hardening: `big`/`small` are picked by `ebin()`, so a de-normalized
        // Normal operand (leading bit unset) misaligns silently. Mirror
        // `check_invariant`'s Normal clause on both operands. (`sub` routes here
        // via `add(other.neg())`, so this covers subtraction too.)
        debug_assert!(
            self.mant[0] & (1u64 << 63) != 0 && other.mant[0] & (1u64 << 63) != 0,
            "add/sub: Normal operands must be normalized (bit P-1 set): \
             self.mant[0]={:#018x}, other.mant[0]={:#018x}",
            self.mant[0],
            other.mant[0]
        );

        const GUARD: u32 = 64;
        // Pick the operand with the larger Ebin as "big"; on a tie either works.
        let (big, small) = if self.ebin() >= other.ebin() { (self, other) } else { (other, self) };
        let shift = (big.ebin() - small.ebin()) as u32; // >= 0

        let w = N + 3;
        // big placed with GUARD zero bits below its LSB.
        let mut big_buf = vec![0u64; w];
        for (i, &v) in to_lsw(&big.mant).iter().enumerate() {
            big_buf[i] = v;
        }
        lsw_shl(&mut big_buf, GUARD);

        // small aligned to the same LSB weight (big.exp - GUARD).
        let mut small_buf = vec![0u64; w];
        for (i, &v) in to_lsw(&small.mant).iter().enumerate() {
            small_buf[i] = v;
        }
        let mut align_sticky = false;
        if shift <= GUARD {
            lsw_shl(&mut small_buf, GUARD - shift);
        } else {
            let drop = shift - GUARD;
            align_sticky = any_bits_below(&small_buf, drop as usize);
            lsw_shr(&mut small_buf, drop);
        }

        let exp_lsb = big.exp - GUARD as i32;

        let (result_buf, result_sign, extra_sticky) = if big.sign == small.sign {
            // Same sign: add magnitudes.
            let mut r = big_buf;
            lsw_add(&mut r, &small_buf);
            (r, big.sign, align_sticky)
        } else {
            // Opposite sign: subtract the smaller magnitude.
            match lsw_cmp(&big_buf, &small_buf) {
                Ordering::Greater => {
                    let mut r = big_buf;
                    lsw_sub(&mut r, &small_buf);
                    if align_sticky {
                        // true |small| is slightly larger than small_buf (dropped
                        // positive bits): borrow one ULP; residual stays sticky.
                        lsw_sub(&mut r, &[1]);
                        (r, big.sign, true)
                    } else {
                        (r, big.sign, false)
                    }
                }
                Ordering::Less => {
                    // Only reachable when ebin is equal (shift == 0 => no align
                    // dropping), so align_sticky is false here.
                    let mut r = small_buf;
                    lsw_sub(&mut r, &big_buf);
                    (r, small.sign, false)
                }
                Ordering::Equal => {
                    if align_sticky {
                        // Degenerate sub-ULP residual; smallest representable step.
                        (vec![1u64; 1], small.sign, true)
                    } else {
                        return (Self::make_zero(false), 0); // exact cancellation -> +0
                    }
                }
            }
        };

        match normalize_round::<N>(&result_buf, exp_lsb, extra_sticky) {
            None => (Self::make_zero(false), 0),
            Some((mant, exp, inexact)) => (
                BigFloat { sign: result_sign, exp, mant, class: FpClass::Normal },
                inexact as u128,
            ),
        }
    }

    // ---- multiply ----------------------------------------------------------

    /// `self * other`, correctly rounded RNE. Returns `(result, rounding_err)`
    /// (0 exact, else 1). Exact 2N-limb schoolbook product, then round to P bits.
    pub fn mul(&self, other: &Self) -> (Self, u128) {
        use FpClass::*;
        let sign = self.sign ^ other.sign;
        match (self.class, other.class) {
            (NaN, _) | (_, NaN) => return (Self::make_nan(), 0),
            (Inf, Zero) | (Zero, Inf) => return (Self::make_nan(), 0),
            (Inf, _) | (_, Inf) => return (Self::make_inf(sign), 0),
            (Zero, _) | (_, Zero) => return (Self::make_zero(sign), 0),
            (Normal, Normal) => {}
        }
        let prod = lsw_mul(&to_lsw(&self.mant), &to_lsw(&other.mant));
        let exp_lsb = self.exp + other.exp;
        match normalize_round::<N>(&prod, exp_lsb, false) {
            None => (Self::make_zero(sign), 0),
            Some((mant, exp, inexact)) => {
                (BigFloat { sign, exp, mant, class: FpClass::Normal }, inexact as u128)
            }
        }
    }

    /// Multiply by a small signed integer (e.g. `k * ln2`), correctly rounded
    /// RNE. Returns `(result, err)` where err is 1 iff low bits were dropped.
    pub fn mul_small_int(&self, n: i64) -> (Self, u128) {
        use FpClass::*;
        if n == 0 {
            return (Self::make_zero(false), 0);
        }
        let sign = self.sign ^ (n < 0);
        let mag = (n as i128).unsigned_abs() as u64;
        match self.class {
            NaN => return (Self::make_nan(), 0),
            Inf => return (Self::make_inf(sign), 0),
            Zero => return (Self::make_zero(sign), 0),
            Normal => {}
        }
        let prod = lsw_mul(&to_lsw(&self.mant), &[mag]);
        match normalize_round::<N>(&prod, self.exp, false) {
            None => (Self::make_zero(sign), 0),
            Some((mant, exp, inexact)) => {
                (BigFloat { sign, exp, mant, class: FpClass::Normal }, inexact as u128)
            }
        }
    }

    // ---- divide ------------------------------------------------------------

    /// GENERAL `self / d`, correctly rounded RNE via **restoring long division**
    /// (design spec §6, normative). Returns `(result, rounding_err)` (0 exact,
    /// else 1). Divides two arbitrary ~P-bit values — required by `log`
    /// (`(m-1)/(m+1)`) and `atan2` (`|y|/|x|`, `1/w`), not just by a small int.
    pub fn div(&self, d: &Self) -> (Self, u128) {
        use FpClass::*;
        let sign = self.sign ^ d.sign;
        match (self.class, d.class) {
            (NaN, _) | (_, NaN) => return (Self::make_nan(), 0),
            (Inf, Inf) | (Zero, Zero) => return (Self::make_nan(), 0),
            (Inf, _) => return (Self::make_inf(sign), 0),
            (_, Inf) => return (Self::make_zero(sign), 0),
            (Zero, _) => return (Self::make_zero(sign), 0),
            (_, Zero) => return (Self::make_inf(sign), 0), // finite / 0
            (Normal, Normal) => {}
        }
        // numerator = mant_s << (P+1); Q0 = floor(numerator / mant_d) has P+1 or
        // P+2 bits (ratio in (0.5,2)); remainder != 0 is the sticky signal. The
        // numerator can reach ~2^(2P), so it needs 2N+2 limbs of headroom.
        let p1 = Self::P as u32 + 1;
        let mut num = vec![0u64; 2 * N + 2];
        for (i, &v) in to_lsw(&self.mant).iter().enumerate() {
            num[i] = v;
        }
        lsw_shl(&mut num, p1);
        let (q0, rem) = lsw_divmod(&num, &to_lsw(&d.mant));
        let exp_lsb = self.exp - d.exp - p1 as i32;
        let extra_sticky = !lsw_is_zero(&rem);
        match normalize_round::<N>(&q0, exp_lsb, extra_sticky) {
            None => (Self::make_zero(sign), 0),
            Some((mant, exp, inexact)) => {
                (BigFloat { sign, exp, mant, class: FpClass::Normal }, inexact as u128)
            }
        }
    }

    /// `self / n` for a small `u64` divisor, correctly rounded RNE via
    /// single-divisor long division. Returns `(result, err)` (0 exact, else 1).
    /// Agrees bit-for-bit with [`div`](Self::div) (both are correctly-rounded).
    pub fn div_small(&self, n: u64) -> (Self, u128) {
        use FpClass::*;
        match self.class {
            NaN => return (Self::make_nan(), 0),
            Inf => return (Self::make_inf(self.sign), 0),
            Zero => {
                return if n == 0 {
                    (Self::make_nan(), 0) // 0 / 0
                } else {
                    (Self::make_zero(self.sign), 0)
                };
            }
            Normal => {}
        }
        if n == 0 {
            return (Self::make_inf(self.sign), 0); // finite / 0
        }
        if n == 1 {
            return (self.clone(), 0);
        }
        // numerator = mant_s << 65 guarantees Q0 has >= P+1 bits for any u64 n.
        const B: u32 = 65;
        let mut num = vec![0u64; N + 2];
        for (i, &v) in to_lsw(&self.mant).iter().enumerate() {
            num[i] = v;
        }
        lsw_shl(&mut num, B);
        let (q0, rem) = lsw_divmod(&num, &[n]);
        let exp_lsb = self.exp - B as i32;
        let extra_sticky = !lsw_is_zero(&rem);
        match normalize_round::<N>(&q0, exp_lsb, extra_sticky) {
            None => (Self::make_zero(self.sign), 0),
            Some((mant, exp, inexact)) => {
                (BigFloat { sign: self.sign, exp, mant, class: FpClass::Normal }, inexact as u128)
            }
        }
    }

    // ---- escalation (const-generic width) ----------------------------------

    /// Widen to `M > N` limbs by exact zero-extension BELOW the low limb (the
    /// value is unchanged; `exp -= 64*(M-N)`). ONLY valid for provably-exact
    /// values — the accumulated `err_ulps` (tracked by the caller) is unchanged.
    pub fn widen<const M: usize>(&self) -> BigFloat<M> {
        assert!(M >= N, "widen requires M >= N");
        let mut mant = [0u64; M];
        mant[..N].copy_from_slice(&self.mant); // top N limbs, zeros below
        let exp = self.exp - 64 * ((M - N) as i32);
        BigFloat { sign: self.sign, exp, mant, class: self.class }
    }

    /// Narrow to `M < N` limbs by keeping the top M limbs (truncation), and
    /// RECORDING the dropped tail as error: `(narrowed, extra_err)` with
    /// `extra_err = 1` iff any dropped limb was nonzero (the dropped tail is
    /// strictly less than one result-ULP), else 0. `exp += 64*(N-M)`.
    pub fn narrow<const M: usize>(&self) -> (BigFloat<M>, u128) {
        assert!(M <= N, "narrow requires M <= N");
        let mut mant = [0u64; M];
        mant.copy_from_slice(&self.mant[..M]);
        let dropped = self.mant[M..].iter().any(|&x| x != 0);
        let exp = self.exp + 64 * ((N - M) as i32);
        let err = if self.class == FpClass::Normal && dropped { 1 } else { 0 };
        (BigFloat { sign: self.sign, exp, mant, class: self.class }, err)
    }

    // ---- debug -------------------------------------------------------------

    /// Debug invariant check (design spec §1). Panics on violation.
    pub fn check_invariant(&self) {
        match self.class {
            FpClass::Normal => {
                assert!(
                    self.mant[0] & (1u64 << 63) != 0,
                    "Normal must be normalized (bit P-1 set): mant[0]={:#018x}",
                    self.mant[0]
                );
            }
            FpClass::Zero => {
                assert!(self.mant.iter().all(|&x| x == 0), "Zero must have an all-zero mantissa");
            }
            FpClass::Inf | FpClass::NaN => {
                // mantissa is don't-care; we canonicalize to all-zero.
                assert!(
                    self.mant.iter().all(|&x| x == 0),
                    "Inf/NaN mantissa should be canonicalized to zero"
                );
            }
        }
    }
}

// ===========================================================================
// round-ziv kernel (Task T2)
//
// Single-rounding round-ties-to-even reducer from a wide `BigFloat<N>` to a
// correctly-rounded f64/f32 bit-pattern, wrapped in a Ziv decidability loop.
// Because the big-float and every target rounding-boundary (midpoint) are exact
// binary numbers, the "does the true value's rounding decide?" test is EXACT
// integer arithmetic on the low limbs: compare the integer distance-to-nearest-
// midpoint `D` against the accumulated error bound `err_ulps`. `D > err_ulps`
// => decided (round once, emit); `D <= err_ulps` => straddle => re-derive at
// 512 then 1024 bits (which shrinks `err_ulps`) and retest.
//
// See docs/superpowers/specs/2026-07-21-planb-slice0-workflow-output.md, the
// `round-ziv` section (its §1–§8) AND its "Review findings" — the CRITICAL fix
// is that `avail == 0` is a straddle boundary handled by the RNE machinery
// (exp(-745) rounds UP to 0x…01), NOT an unconditional round-to-zero.
// ===========================================================================

/// Target IEEE-754 binary format parameters (round-ziv §1). `t` counts the
/// implicit leading 1 (53 = 1+52 for f64, 24 = 1+23 for f32).
#[derive(Clone, Copy, Debug)]
pub struct Fmt {
    /// significand bits, INCLUDING the implicit leading 1.
    pub t: u32,
    /// minimum normal unbiased exponent.
    pub emin: i32,
    /// maximum normal unbiased exponent.
    pub emax: i32,
    /// dtype container width in bits (64 / 32).
    pub width: u32,
}

/// binary64 target parameters.
pub const F64: Fmt = Fmt { t: 53, emin: -1022, emax: 1023, width: 64 };
/// binary32 target parameters.
pub const F32: Fmt = Fmt { t: 24, emin: -126, emax: 127, width: 32 };

/// The authoritative evaluation-precision enum (round-ziv §8). The hp core
/// ALWAYS computes wider than the compute dtype, so this is always
/// `WiderThanCompute`; the numeric detail is `Certificate::stabilized_precision_bits`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvalPrecision {
    ComputeDtype,
    WiderThanCompute,
}

/// The self-certifying rounding certificate (round-ziv §8). Serialized unchanged
/// into the frozen v1 `certificate` object `{hardness_margin_bits,
/// stabilized_precision_bits}`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Certificate {
    /// bit-distance by which the value misses the nearest target midpoint
    /// (leading-zero run of `D` in a `shift-1`-bit field). Larger = closer to a
    /// midpoint = harder. Capped at `shift-1` for an exact midpoint.
    pub hardness_margin_bits: u32,
    /// the working precision `P = 64*N` at which `D > err_ulps` FIRST held
    /// (256 | 512 | 1024). Strictly greater than the compute dtype width by
    /// construction (the §6.5-0009 carryover).
    pub stabilized_precision_bits: u32,
}

/// The result of rounding one `Evaluated` to a target format.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RoundResult {
    /// the target IEEE-754 bit pattern (f64 fills all 64 bits; f32 the low 32).
    pub bits: u64,
    /// true iff the rounding DECIDED at this width (`D > err_ulps`, or the input
    /// was exact / special / over-underflowed). When false the caller must
    /// escalate to a wider `N` (the [`round_atom_to_f64`] driver does so).
    pub decided: bool,
    /// always `WiderThanCompute` for the hp core.
    pub eval_precision: EvalPrecision,
    pub cert: Certificate,
}

/// Encode `(sign, exp_field, frac)` into an IEEE-754 bit pattern for `fmt`.
fn encode_bits(fmt: Fmt, sign: bool, exp_field: u64, frac: u64) -> u64 {
    let frac_width = fmt.t - 1; // stored fraction bits (implicit 1 dropped)
    ((sign as u64) << (fmt.width - 1)) | (exp_field << frac_width) | frac
}

/// The all-ones biased-exponent field for `fmt` (Inf/NaN exponent).
fn max_exp_field(fmt: Fmt) -> u64 {
    (1u64 << (fmt.width - fmt.t)) - 1
}

/// The target bit pattern for a non-finite / zero class.
fn special_bits(fmt: Fmt, class: FpClass, sign: bool) -> u64 {
    match class {
        FpClass::Zero => encode_bits(fmt, sign, 0, 0),
        FpClass::Inf => encode_bits(fmt, sign, max_exp_field(fmt), 0),
        // canonical quiet NaN: max exp field, top fraction bit set, sign 0.
        FpClass::NaN => encode_bits(fmt, false, max_exp_field(fmt), 1u64 << (fmt.t - 2)),
        FpClass::Normal => unreachable!("special_bits called on Normal"),
    }
}

/// Round one already-evaluated wide value to `fmt`, RNE, single rounding
/// (round-ziv §2–§5, §7). Measures the exact distance to the nearest midpoint
/// and decides via `D > err_ulps`. Does NOT escalate — the driver
/// ([`round_atom_to_f64`]/[`round_atom_to_f32`]) calls this once per width.
fn round_evaluated<const N: usize>(ev: &Evaluated<N>, fmt: Fmt) -> RoundResult {
    let val = &ev.val;
    let err_ulps = ev.err_ulps;
    let stab = (64 * N) as u32;
    // Kernel invariant (round-ziv §8): the working precision strictly exceeds the
    // compute dtype width, so evaluation_precision is always wider-than-compute.
    debug_assert!(stab > fmt.t, "stabilized precision must exceed compute dtype width");
    let cert_special = Certificate { hardness_margin_bits: 0, stabilized_precision_bits: stab };
    let decided_special = |bits: u64| RoundResult {
        bits,
        decided: true,
        eval_precision: EvalPrecision::WiderThanCompute,
        cert: cert_special,
    };

    // --- non-finite / zero from the atom: bypass rounding (round-ziv §7). ---
    match val.class {
        FpClass::Zero => return decided_special(special_bits(fmt, FpClass::Zero, val.sign)),
        FpClass::Inf => return decided_special(special_bits(fmt, FpClass::Inf, val.sign)),
        FpClass::NaN => return decided_special(special_bits(fmt, FpClass::NaN, false)),
        FpClass::Normal => {}
    }

    // --- locate the round position (round-ziv §2). ---
    let p = (64 * N) as i32;
    let e_orig = val.ebin(); // tentative unbiased result exponent E
    let avail_i: i32 = if e_orig >= fmt.emin {
        fmt.t as i32
    } else {
        fmt.t as i32 - (fmt.emin - e_orig)
    };

    // Underflow region (avail <= 0). NOTE (round-ziv review, CRITICAL): `avail == 0`
    // is a straddle BOUNDARY that flows through the RNE machinery below (q=0,
    // R=leading bit=1, round_up=sticky), so exp(-745) in (2^-1075, 2^-1074) rounds
    // correctly. The two boundaries strictly below `avail == 0` are (symmetric to
    // the overflow boundary handled during encoding):
    //
    //  * avail <= -2: the value is >= 2 binades below the underflow midpoint
    //    2^(emin-t); its distance is >= 2^P ULPs, which exceeds any u128 `err_ulps`
    //    (< 2^128 <= 2^P) by construction -> unconditional signed zero, decided.
    //  * avail == -1: the value sits in [2^(emin-t-1), 2^(emin-t)) — strictly below
    //    the underflow midpoint, so RNE gives signed zero, BUT its error interval
    //    can cross the midpoint into the smallest-subnormal cell. Run the Ziv D-test
    //    against the midpoint (in val-ULP units, D = 2^P - m): decided below ->
    //    signed zero; straddle -> escalate (re-derivation at a wider width may move
    //    the value up to avail == 0 and resolve it to the smallest subnormal).
    if avail_i <= -2 {
        return decided_special(special_bits(fmt, FpClass::Zero, val.sign));
    }
    if avail_i == -1 {
        let m_lsw = to_lsw(&val.mant);
        // D = 2^P - m = distance from the value UP to the underflow midpoint, in
        // the value's own ULP (2^exp). m in [2^(P-1), 2^P) => D in (0, 2^(P-1)],
        // which fits in N limbs (never truncated).
        let mut buf = vec![0u64; N + 1];
        lsw_set_bit(&mut buf, p as usize); // 2^P
        lsw_sub(&mut buf, &m_lsw); // 2^P - m (m < 2^P, so no final borrow)
        let d = lsw_low_to_msw::<N>(&buf);
        let d_lsw = to_lsw(&d);
        let hardness_margin_bits = match msb_index(&d_lsw) {
            None => p as u32, // exactly on the midpoint (unreachable: m < 2^P)
            Some(msb) => (p as u32).saturating_sub((msb + 1) as u32),
        };
        // The value is strictly below the midpoint => RNE is a signed zero; the
        // exact interval test governs whether that is decided or a straddle.
        let decided = err_ulps == 0 || BigFloat::<N>::dist_gt_err(&d, err_ulps);
        return RoundResult {
            bits: special_bits(fmt, FpClass::Zero, val.sign),
            decided,
            eval_precision: EvalPrecision::WiderThanCompute,
            cert: Certificate { hardness_margin_bits, stabilized_precision_bits: stab },
        };
    }
    let avail = avail_i as u32; // >= 0; avail <= t <= 53 so `q`/`sig` fit in a u64
    let shift = (p - avail_i) as u32; // low bits below the kept significand; in [P-t, P]

    // --- split the mantissa at the round bit (limb-wise; shift may reach P). ---
    let m_lsw = to_lsw(&val.mant);
    // q = m >> shift (the candidate significand, <= t bits => fits a u64; q == 0
    // when avail == 0). The `(1<<shift)-1` mask is formed limb-wise (see
    // `lsw_mask_low`), never as a single-word literal that would overflow at shift==P.
    let q: u64 = {
        let mut buf = m_lsw.clone();
        lsw_shr(&mut buf, shift);
        buf[0]
    };
    let round_bit = lsw_bit(&m_lsw, (shift - 1) as usize) == 1;
    let sticky = any_bits_below(&m_lsw, (shift - 1) as usize);

    // --- exact round-ties-to-even (round-ziv §3). ---
    let round_up = round_bit && (sticky || (q & 1) == 1);
    let mut sig = q + round_up as u64; // may carry to 2^avail

    // --- exact distance to the nearest midpoint (round-ziv §4). ---
    // F = m mod 2^shift; Fmid = 2^(shift-1); D = |F - Fmid|, exact big-int arith
    // on the low `shift` bits (both share `val.exp`, so `abs_diff_limbs` is exact).
    let f_bf = {
        let mut buf = m_lsw.clone();
        lsw_mask_low(&mut buf, shift as usize);
        BigFloat::<N> { sign: false, exp: val.exp, mant: lsw_low_to_msw::<N>(&buf), class: FpClass::Normal }
    };
    let fmid_bf = {
        let mut buf = vec![0u64; N];
        lsw_set_bit(&mut buf, (shift - 1) as usize);
        BigFloat::<N> { sign: false, exp: val.exp, mant: lsw_low_to_msw::<N>(&buf), class: FpClass::Normal }
    };
    let d = f_bf.abs_diff_limbs(&fmid_bf);

    // hardness_margin_bits = leading-zero run of D in a (shift-1)-bit field.
    let d_lsw = to_lsw(&d);
    let hardness_margin_bits = match msb_index(&d_lsw) {
        None => shift - 1, // D == 0: exact midpoint (cap)
        Some(msb) => (shift - 1).saturating_sub((msb + 1) as u32),
    };

    // --- encode (sign, E, sig) into the target (round-ziv §3, §7). ---
    let sign = val.sign;
    let bias = fmt.emax as i64; // exponent bias == emax (1023 / 127)
    let mut overflow = false;
    let bits: u64;
    if e_orig >= fmt.emin {
        // NORMAL target: avail == t, sig in [2^(t-1), 2^t] (2^t only on carry).
        let mut e_final = e_orig;
        if sig == (1u64 << avail) {
            sig >>= 1; // mantissa carry: renormalize (== 2^(t-1)), bump exponent
            e_final += 1;
        }
        if e_orig > fmt.emax {
            // A full binade above the overflow midpoint (distance >= 2^P ULPs >>
            // any u128 err): unconditionally +/-Inf, decided.
            overflow = true;
            bits = special_bits(fmt, FpClass::Inf, sign);
        } else if e_final > fmt.emax {
            // e_orig == emax and the mantissa carried out of the top finite cell.
            // The bits are +/-Inf, but decidedness is governed by the normal D-test
            // (D here measures the exact max-finite<->Inf midpoint), so a value
            // whose error interval straddles that midpoint escalates instead of
            // being mis-certified. `overflow` stays false.
            bits = special_bits(fmt, FpClass::Inf, sign);
        } else {
            let exp_field = (e_final as i64 + bias) as u64;
            let frac = sig - (1u64 << (fmt.t - 1)); // strip the implicit leading 1
            bits = encode_bits(fmt, sign, exp_field, frac);
        }
    } else {
        // SUBNORMAL target (includes the avail == 0 boundary): the fraction field
        // IS `sig` (no implicit 1), exp_field == 0. A round-up to 2^(t-1) promotes
        // cleanly to the smallest normal (exp_field = 1, frac = 0).
        if sig >= (1u64 << (fmt.t - 1)) {
            bits = encode_bits(fmt, sign, 1, 0);
        } else {
            bits = encode_bits(fmt, sign, 0, sig);
        }
    }

    // --- Ziv decidability (round-ziv §5). ---
    // Only the nearest (upper) midpoint is ever reachable: the guard `err_ulps <
    // 2^(shift-1)` is TYPE-GUARANTEED, not merely asserted — `err_ulps` is a u128
    // (< 2^128) and shift-1 >= P-t-1 >= 202, so 2^(shift-1) >= 2^202 > 2^128 for
    // every width/exponent. The debug_assert documents (and cheaply re-checks) it.
    debug_assert!(
        shift >= 129 || err_ulps < (1u128 << (shift - 1)),
        "err_ulps ({}) exceeds half a rounding cell (shift={}): atom error bound \
         catastrophically loose or precision far too low",
        err_ulps,
        shift
    );
    // An exact input (err_ulps == 0) always decides — RNE of an exact value,
    // including an exact midpoint via ties-to-even, is determined. A value a full
    // binade past emax (`overflow`) is decided-Inf. Otherwise — INCLUDING the emax
    // carry-to-Inf case (and the avail == -1 underflow boundary handled above) —
    // the exact interval test `D > err_ulps` governs.
    let decided = overflow || err_ulps == 0 || BigFloat::<N>::dist_gt_err(&d, err_ulps);

    RoundResult {
        bits,
        decided,
        eval_precision: EvalPrecision::WiderThanCompute,
        cert: Certificate { hardness_margin_bits, stabilized_precision_bits: stab },
    }
}

/// Round an already-evaluated wide value to f64, RNE, single rounding. The
/// returned [`RoundResult::decided`] reports whether the Ziv test settled at this
/// width; if false, escalate (or use [`round_atom_to_f64`], which does so).
pub fn round_to_f64<const N: usize>(ev: &Evaluated<N>) -> RoundResult {
    round_evaluated(ev, F64)
}

/// Round an already-evaluated wide value DIRECTLY to f32, RNE, single rounding
/// (never f64-then-f32 — round-ziv §3 double-rounding hazard).
pub fn round_to_f32<const N: usize>(ev: &Evaluated<N>) -> RoundResult {
    round_evaluated(ev, F32)
}

/// A transcendental atom (exp/log/sin/clog component) re-evaluable at any working
/// width `N ∈ {4,8,16}`. The escalation driver calls `eval::<N>()` at successively
/// wider `N` until the rounding DECIDES.
///
/// **Contract for T5/T6 atom implementers:** each `eval::<N>()` MUST re-derive
/// the value from the ORIGINAL exact f64/f32 input at width `N` (`from_f64(x)` is
/// exact at any `N`), running the reduction against the wider constant view — this
/// adds ~256 genuinely-correct bits per step, shrinking `err_ulps` while the true
/// value's distance-to-midpoint is fixed. Widening an INEXACT 256-bit result
/// preserves its error and shrinks nothing (the spin-to-1024-panic failure;
/// hp-core §8). `err_ulps` must be a RIGOROUS bound, in units of the returned
/// value's ULP (`2^val.exp`), that INCLUDES folded series/reduction truncation.
pub trait Atom {
    fn eval<const N: usize>(&self) -> Evaluated<N>;
}

/// Ziv escalation driver: round `atom` to f64, re-deriving at 256 -> 512 -> 1024
/// bits on a straddle. Panics only if unresolved at 1024 (a genuine red flag: a
/// mis-routed exact/special value or an error-bound bug — round-ziv §5).
pub fn round_atom_to_f64<A: Atom>(atom: &A) -> RoundResult {
    round_atom(atom, F64)
}

/// As [`round_atom_to_f64`] but rounds DIRECTLY to f32 (single rounding, §3).
pub fn round_atom_to_f32<A: Atom>(atom: &A) -> RoundResult {
    round_atom(atom, F32)
}

fn round_atom<A: Atom>(atom: &A, fmt: Fmt) -> RoundResult {
    if let Some(r) = try_width::<4, A>(atom, fmt) {
        return r;
    }
    if let Some(r) = try_width::<8, A>(atom, fmt) {
        return r;
    }
    if let Some(r) = try_width::<16, A>(atom, fmt) {
        return r;
    }
    panic!(
        "hp::round: rounding unresolved at 1024 bits for {} — mis-routed \
         exact/special value or error-bound bug; investigate",
        if fmt.width == 64 { "f64" } else { "f32" }
    );
}

/// Re-derive the atom at width `N` and round; `Some` iff the Ziv test decided.
fn try_width<const N: usize, A: Atom>(atom: &A, fmt: Fmt) -> Option<RoundResult> {
    let ev = atom.eval::<N>();
    let r = round_evaluated(&ev, fmt);
    if r.decided {
        Some(r)
    } else {
        None
    }
}
