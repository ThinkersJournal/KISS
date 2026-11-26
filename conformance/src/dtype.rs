// SPDX-License-Identifier: MIT OR Apache-2.0
//! From-scratch storage codec for the pinned **sub-byte / FP8 dtype layouts**
//! (KISS-Classify §6.1-0008 `i4`/`u4` nibble packing, §6.1-0009 `b1` bit packing,
//! §6.1-0010 `e4m3` FP8 encoding).
//!
//! This is the Classify **storage** view: it reproduces the exact byte layout the
//! spec pins for the packed and FP8 dtypes and decodes a stored bit pattern back
//! to its logical value from scratch. It shares no code with any generator, so an
//! implementation under test can be differenced against it (Conform §6.5).
//!
//! Scope boundary — §6.1-0010 (`e4m3`) is pinned here as a **decoding/format**
//! obligation only: which bit patterns are finite, which is NaN, and that there is
//! **no** infinity encoding (max finite ±448). The saturating round-half-to-even
//! conversion *into* `e4m3` is a distinct obligation owned by KISS-Ops §6.16-0004 /
//! KISS-Emit and is deliberately NOT modeled here (§6.1-0010 last sentence).

// ---------------------------------------------------------------------------
// §6.1-0008 — s4 / u4 packed-pair byte layout
// ---------------------------------------------------------------------------
//
// Two logical 4-bit values share one byte: the **low nibble holds the EVEN
// logical index**, the **high nibble holds the ODD logical index**. `s4` is
// sign-extended on read (range [-8, +7]); `u4` is zero-extended on read
// (range [0, 15]). The pack/unpack below are keyed on `index & 1`, never on
// byte order, so a "high-nibble-first" unpacker (even index from the high
// nibble) diverges immediately.

/// Sign-extend a 4-bit nibble (`0..=15`) to `i8` as an `i4` value: nibbles
/// `8..=15` are the negatives `-8..=-1`, nibbles `0..=7` stay `0..=7`
/// (§6.1-0008, "sign-extended on read"). E.g. `0xF -> -1`, `0x8 -> -8`.
pub fn s4_sign_extend(nibble: u8) -> i8 {
    let n = (nibble & 0x0F) as i8;
    if n & 0x08 != 0 { n - 16 } else { n }
}

/// Zero-extend a 4-bit nibble (`0..=15`) to `u8` as a `u4` value (§6.1-0008,
/// "zero-extended on read"). E.g. `0xF -> 15` — never `-1`.
pub fn u4_zero_extend(nibble: u8) -> u8 {
    nibble & 0x0F
}

/// Pack a run of `i4` logical values into packed-pair bytes: even index → low
/// nibble, odd index → high nibble (§6.1-0008). A trailing lone even value
/// occupies the low nibble with the high nibble zero-filled. Each value is
/// truncated to its low 4 bits (its two's-complement `i4` pattern).
pub fn s4_pack(vals: &[i8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vals.len().div_ceil(2));
    let mut i = 0;
    while i < vals.len() {
        let lo = (vals[i] as u8) & 0x0F; // even index → low nibble
        let hi = if i + 1 < vals.len() { (vals[i + 1] as u8) & 0x0F } else { 0 };
        out.push(lo | (hi << 4)); // odd index → high nibble
        i += 2;
    }
    out
}

/// Pack a run of `u4` logical values into packed-pair bytes (identical byte
/// layout to [`s4_pack`], §6.1-0008: even → low nibble, odd → high nibble).
pub fn u4_pack(vals: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vals.len().div_ceil(2));
    let mut i = 0;
    while i < vals.len() {
        let lo = vals[i] & 0x0F;
        let hi = if i + 1 < vals.len() { vals[i + 1] & 0x0F } else { 0 };
        out.push(lo | (hi << 4));
        i += 2;
    }
    out
}

/// Unpack `count` `i4` logical values from packed-pair bytes: index `k` reads
/// the **low** nibble of byte `k/2` when `k` is even and the **high** nibble
/// when `k` is odd, then sign-extends (§6.1-0008). Panics if `bytes` is too
/// short for `count` values.
pub fn s4_unpack(bytes: &[u8], count: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(count);
    for k in 0..count {
        let byte = bytes[k / 2];
        let nib = if k % 2 == 0 { byte & 0x0F } else { (byte >> 4) & 0x0F };
        out.push(s4_sign_extend(nib));
    }
    out
}

/// Unpack `count` `u4` logical values from packed-pair bytes (same nibble
/// selection as [`s4_unpack`] but **zero**-extended, §6.1-0008).
pub fn u4_unpack(bytes: &[u8], count: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(count);
    for k in 0..count {
        let byte = bytes[k / 2];
        let nib = if k % 2 == 0 { byte & 0x0F } else { (byte >> 4) & 0x0F };
        out.push(u4_zero_extend(nib));
    }
    out
}

// ---------------------------------------------------------------------------
// §6.1-0009 — b1 packed-byte bit layout
// ---------------------------------------------------------------------------
//
// 8 bits per byte, the **least-significant bit holds the LOWEST logical index**.
// Bit `j` (value `1 << j`) of byte `k` carries logical index `8*k + j`. This is
// LSB-first; the opposite (MSB-first, `numpy.packbits` default) diverges on any
// non-symmetric pattern.

/// Pack `bits` (each `0` or `1`) into `b1` packed bytes, LSB = lowest logical
/// index (§6.1-0009). Logical index `i` sets bit `i % 8` of byte `i / 8`.
/// A trailing partial byte is zero-filled in its unused high bits.
pub fn b1_pack(bits: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; bits.len().div_ceil(8)];
    for (i, &b) in bits.iter().enumerate() {
        if b & 1 != 0 {
            out[i / 8] |= 1 << (i % 8);
        }
    }
    out
}

/// Unpack `count` `b1` logical bits from packed bytes, LSB-first (§6.1-0009):
/// logical index `i` is bit `i % 8` of byte `i / 8`. Returns `0`/`1` values.
pub fn b1_unpack(bytes: &[u8], count: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        out.push((bytes[i / 8] >> (i % 8)) & 1);
    }
    out
}

// ---------------------------------------------------------------------------
// §6.1-0010 — e4m3 FP8 storage/decoding
// ---------------------------------------------------------------------------
//
// 1 sign bit, 4 exponent bits (bias 7), 3 mantissa bits. Max finite magnitude
// ±448 (= S.1111.110). **No infinity encoding.** A single NaN encoding:
// S.1111.111 (bytes 0x7F / 0xFF). Exponent field 1111 is therefore an ordinary
// finite exponent for every mantissa except 111 — the load-bearing difference
// from an IEEE-style mini-float that would treat 1111 as inf/NaN wholesale (and
// so report a max finite of 240, and spurious infinities).

/// Largest finite `e4m3` magnitude (§6.1-0010): `S.1111.110` =
/// `2^(15-7) * (1 + 6/8)` = `256 * 1.75` = `448`.
pub const E4M3_MAX_FINITE: f32 = 448.0;

/// The decoded class of an `e4m3` byte. There is deliberately **no `Infinite`
/// variant** — `e4m3` defines no infinity encoding (§6.1-0010).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum E4M3 {
    /// Signed zero (`e=0`, `m=0`); `negative` distinguishes `-0` from `+0`.
    Zero { negative: bool },
    /// Subnormal (`e=0`, `m!=0`): value = `sign * (m/8) * 2^(1-7)`.
    Subnormal(f32),
    /// Normal (`e` in `1..=15`, excluding the NaN pattern): value =
    /// `sign * (1 + m/8) * 2^(e-7)`.
    Normal(f32),
    /// The single NaN encoding `S.1111.111` (`e=15`, `m=7`).
    Nan,
}

impl E4M3 {
    /// True only for the single NaN encoding (§6.1-0010).
    pub fn is_nan(self) -> bool {
        matches!(self, E4M3::Nan)
    }

    /// The finite numeric value, or `None` for NaN. `e4m3` has no infinity, so a
    /// finite (non-NaN) byte always yields `Some`.
    pub fn value(self) -> Option<f32> {
        match self {
            E4M3::Zero { negative } => Some(if negative { -0.0 } else { 0.0 }),
            E4M3::Subnormal(v) | E4M3::Normal(v) => Some(v),
            E4M3::Nan => None,
        }
    }

    /// The finite magnitude `|value|`, or `None` for NaN — used to find the
    /// maximum finite magnitude across all 256 encodings.
    pub fn finite_magnitude(self) -> Option<f32> {
        self.value().map(f32::abs)
    }
}

/// Decode one stored `e4m3` byte to its class/value from scratch (§6.1-0010):
/// bit 7 = sign, bits 6..=3 = biased exponent (bias 7), bits 2..=0 = mantissa.
/// `S.1111.111` is the single NaN; every other exponent-1111 mantissa is a
/// normal finite number; there is no infinity encoding.
pub fn e4m3_decode(byte: u8) -> E4M3 {
    let sign = (byte >> 7) & 0x01;
    let exp = (byte >> 3) & 0x0F;
    let mant = byte & 0x07;
    let s = if sign == 1 { -1.0f32 } else { 1.0f32 };

    if exp == 0x0F && mant == 0x07 {
        return E4M3::Nan; // the single NaN encoding; NO inf path exists
    }
    if exp == 0 {
        if mant == 0 {
            return E4M3::Zero { negative: sign == 1 };
        }
        // subnormal: 2^(1-bias) * (m / 8), bias = 7
        let v = s * (mant as f32 / 8.0) * 2.0f32.powi(1 - 7);
        return E4M3::Subnormal(v);
    }
    // normal: 2^(e-bias) * (1 + m/8), bias = 7
    let v = s * (1.0 + mant as f32 / 8.0) * 2.0f32.powi(exp as i32 - 7);
    E4M3::Normal(v)
}

// ---------------------------------------------------------------------------
// §6.1-0013 — E8M0 MX scale storage/decoding (`f8e8m0`)
// ---------------------------------------------------------------------------
//
// Unsigned 8-bit, 0 sign / 8 exponent / 0 mantissa, bias 127. A stored byte `e`
// in `0x00..=0xFE` denotes the power of two `2^(e - 127)`; `0xFF` is the single
// NaN; there is no infinity and no zero encoding. SECONDARY-SOURCED (Rouhani et
// al., arXiv 2310.10537 §2.2; OCP MX v1.0) pending verification against the
// primary OCP PDF; "no zero" is an inference from the format having no mantissa
// and a biased exponent field covering `0x00..=0xFE` (see the clause text).
// `f8e6m2` is RESERVED (no layout pinned), so there is deliberately no decoder
// for it here.

/// The exponent bias of E8M0 (§6.1-0013).
pub const E8M0_BIAS: i32 = 127;

/// The decoded class of an `f8e8m0` byte (§6.1-0013). There is deliberately no
/// `Infinite` and no `Zero` variant: the format defines neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum E8M0 {
    /// The power of two `2^k`, carried as the unbiased exponent `k = e - 127`.
    Pow2(i32),
    /// The single NaN encoding `0xFF`.
    Nan,
}

/// Decode one stored `f8e8m0` byte from scratch (§6.1-0013): `0xFF` is NaN, every
/// other byte `e` is `2^(e - 127)`.
pub fn e8m0_decode(byte: u8) -> E8M0 {
    if byte == 0xFF {
        E8M0::Nan
    } else {
        E8M0::Pow2(i32::from(byte) - E8M0_BIAS)
    }
}

// ---------------------------------------------------------------------------
// §6.1-0014 / §6.1-0015 — MX element storage (f4e2m1, f6e2m3, f6e3m2)
// ---------------------------------------------------------------------------
//
// Library codec for the three MX element dtypes, so an implementation under test can be
// differenced against it (before this, the packing existed only as helpers inside one test
// file). `f4e2m1` packs exactly like `u4` (low nibble = EVEN logical index); the two FP6
// dtypes pack four elements per three bytes, LSB-first (element `i` at stream bits `6i..6i+5`).
//
// SOURCES for the packing (read 2026-11-25; the packing is a CONVENTION, OCP MX v1.0 fixes none):
//   * NVIDIA/cutlass `include/cutlass/subbyte_reference.h` `SubbyteReference::get()`:
//     `(*byte_ptr >> (byte_offset * sizeof_bits<Element>)) & kMask` with
//     `byte_offset = offset_ % elements_per_byte`: element 0 is the LOW nibble. AGREES.
//   * the same file's non-divisible specialization (8 % 6 != 0): `start_bit_idx_ =
//     (offset_ * 6) % 8`, `low_storage_unit_idx_ = (offset_ * 6) / 8`, element read as
//     `full_item >> start_bit_idx_`: contiguous LSB-first, four per three bytes. AGREES.
//   * Triton `python/triton/language/core.py` dot_scaled docstring: "Fp4 elements are packed
//     into uint8 inputs with the first element in lower bits" — read through a summarising
//     fetch, NOT verbatim, so weaker evidence. AGREES.
// NOT read: CUTLASS's shared-memory / TMA forms (`float_e2m1_unpacksmem_t` etc. are selector
// tags in `float_subbyte.h` with no layout code there), the Mojo layout, and the text of the OCP
// MX v1.0 PDF (the fetch returned compressed bytes). So this is a second source for the
// global-memory reference layout only.

/// Pack `codes` (each `< 1 << bits`) LSB-first into a byte stream: element `i` occupies stream
/// bits `bits*i .. bits*(i+1)`, stream bit `k` being bit `k % 8` of byte `k / 8`. `ceil(bits*n/8)` bytes;
/// unused trailing bits are zero.
pub fn lsb_pack(codes: &[u8], bits: u32) -> Vec<u8> {
    let mut out = vec![0u8; (codes.len() * bits as usize).div_ceil(8)];
    for (i, &v) in codes.iter().enumerate() {
        assert!(u32::from(v) < (1u32 << bits), "code {v} does not fit {bits} bits");
        for b in 0..bits as usize {
            if (v >> b) & 1 == 1 {
                let k = i * bits as usize + b;
                out[k / 8] |= 1 << (k % 8);
            }
        }
    }
    out
}

/// Inverse of [`lsb_pack`]: read `count` `bits`-wide codes from `bytes`.
///
/// Panics if `bytes` is too short for `count` codes of `bits` bits (the same convention as
/// [`s4_unpack`]).
pub fn lsb_unpack(bytes: &[u8], bits: u32, count: usize) -> Vec<u8> {
    (0..count)
        .map(|i| {
            (0..bits as usize).fold(0u8, |acc, b| {
                let k = i * bits as usize + b;
                acc | (((bytes[k / 8] >> (k % 8)) & 1) << b)
            })
        })
        .collect()
}

/// Pack `f4e2m1` codes (`0..=15`) two per byte, low nibble = even index (§6.1-0014).
pub fn f4e2m1_pack(codes: &[u8]) -> Vec<u8> {
    lsb_pack(codes, 4)
}

/// Pack `f6e2m3` / `f6e3m2` codes (`0..=63`) four per three bytes, LSB-first (§6.1-0015).
pub fn f6_pack(codes: &[u8]) -> Vec<u8> {
    lsb_pack(codes, 6)
}

/// Decode an MX element code with `ebits` exponent bits, `mbits` mantissa bits and exponent bias
/// `bias` (sign in the next bit up). `code` must fit `1 + ebits + mbits` bits (checked by
/// `debug_assert!`; in release builds higher bits are ignored). For the three MX element formats
/// called below no code is infinite or NaN (§6.1-0014, §6.1-0015); the value
/// is `(-1)^s * 2^(e-bias) * (1 + m/2^mbits)` for `e != 0`, `(-1)^s * 2^(1-bias) * (m/2^mbits)`
/// for `e == 0`. Exact in `f64` for these widths.
pub fn mx_element_decode(code: u8, ebits: u32, mbits: u32, bias: i32) -> f64 {
    debug_assert!(u32::from(code) < (1u32 << (1 + ebits + mbits)), "code {code} does not fit {} bits", 1 + ebits + mbits);
    let m = f64::from(code & ((1u8 << mbits) - 1));
    let e = i32::from((code >> mbits) & ((1u8 << ebits) - 1));
    let s = (code >> (mbits + ebits)) & 1;
    let frac = m / f64::from(1u32 << mbits);
    let mag = if e == 0 { 2f64.powi(1 - bias) * frac } else { 2f64.powi(e - bias) * (1.0 + frac) };
    if s == 1 { -mag } else { mag }
}

/// `f4e2m1` code (`0..=15`) to value (§6.1-0014).
pub fn f4e2m1_decode(code: u8) -> f64 {
    mx_element_decode(code, 2, 1, 1)
}

/// `f6e2m3` code (`0..=63`) to value (§6.1-0015).
pub fn f6e2m3_decode(code: u8) -> f64 {
    mx_element_decode(code, 2, 3, 1)
}

/// `f6e3m2` code (`0..=63`) to value (§6.1-0015).
pub fn f6e3m2_decode(code: u8) -> f64 {
    mx_element_decode(code, 3, 2, 3)
}

// ---------------------------------------------------------------------------
// §6.1-0017 .. -0020 — union batch 1 (CireSnave board 501 "A"): wide integers, f128, f8e3m4, f8ue4m3
// ---------------------------------------------------------------------------

/// `(min, max, bits, signed)` of the four wide integer tokens (§6.1-0017), each bound as its literal bit
/// pattern in big-endian 64-bit limbs, most-significant limb first, occupying the first `bits / 64` limbs
/// (the rest are zero). `None` for every other token.
pub fn wide_int_bounds(token: &str) -> Option<([u64; 4], [u64; 4], u32, bool)> {
    let m = u64::MAX;
    let s = 0x8000_0000_0000_0000u64;
    let h = 0x7FFF_FFFF_FFFF_FFFFu64;
    Some(match token {
        "i128" => ([s, 0, 0, 0], [h, m, 0, 0], 128, true),
        "u128" => ([0; 4], [m, m, 0, 0], 128, false),
        "i256" => ([s, 0, 0, 0], [h, m, m, m], 256, true),
        "u256" => ([0; 4], [m, m, m, m], 256, false),
        _ => return None,
    })
}

/// The class of an `f128` bit pattern (§6.1-0018): IEEE-754 binary128, sign 1 / exponent 15 / fraction 112.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F128Class {
    /// `±0`.
    Zero { negative: bool },
    /// Exponent field 0, fraction non-zero.
    Subnormal { negative: bool },
    /// Exponent field `1..=0x7FFE`.
    Normal { negative: bool },
    /// Exponent field `0x7FFF`, fraction 0.
    Infinity { negative: bool },
    /// Exponent field `0x7FFF`, fraction non-zero; quiet iff the top fraction bit is set (IEEE 754-2019 6.2.1).
    Nan { signaling: bool },
}

/// Classify one `f128` bit pattern from scratch (§6.1-0018).
pub fn f128_class(bits: u128) -> F128Class {
    let negative = bits >> 127 == 1;
    let exp = ((bits >> 112) & 0x7FFF) as u32;
    let frac = bits & ((1u128 << 112) - 1);
    match (exp, frac) {
        (0, 0) => F128Class::Zero { negative },
        (0, _) => F128Class::Subnormal { negative },
        (0x7FFF, 0) => F128Class::Infinity { negative },
        (0x7FFF, f) => F128Class::Nan { signaling: f >> 111 == 0 },
        _ => F128Class::Normal { negative },
    }
}

/// The decoded value of an `f8e3m4` byte (§6.1-0019).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum F8Value {
    /// A finite value, `-0.0` included (exact in `f64`).
    Finite(f64),
    /// `±infinity`: exponent field 7, mantissa 0.
    Infinity { negative: bool },
    /// Exponent field 7, mantissa non-zero (30 NaN encodings: 15 non-zero mantissas times 2 signs).
    Nan,
}

/// Decode one `f8e3m4` byte from scratch (§6.1-0019): sign 1, exponent 3, mantissa 4, bias 3, IEEE-style
/// infinity and NaN.
pub fn f8e3m4_decode(byte: u8) -> F8Value {
    let negative = byte >> 7 == 1;
    let exp = i32::from((byte >> 4) & 7);
    let man = f64::from(byte & 15);
    if exp == 7 {
        return if byte & 15 == 0 { F8Value::Infinity { negative } } else { F8Value::Nan };
    }
    let mag = if exp == 0 { 2f64.powi(1 - 3) * (man / 16.0) } else { 2f64.powi(exp - 3) * (1.0 + man / 16.0) };
    F8Value::Finite(if negative { -mag } else { mag })
}

/// The decoded value of an `f8ue4m3` byte (§6.1-0020).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ue4m3 {
    /// A finite non-negative value in `[0, 448]`.
    Value(f64),
    /// The single NaN encoding `0x7F`.
    Nan,
    /// Bit 7 set: the format has no sign bit, so no value is defined for the byte.
    OutsideFormat,
}

/// Decode one `f8ue4m3` byte from scratch (§6.1-0020): exponent 4, mantissa 3, bias 7, no sign, no infinity.
pub fn ue4m3_decode(byte: u8) -> Ue4m3 {
    if byte & 0x80 != 0 {
        return Ue4m3::OutsideFormat;
    }
    if byte == 0x7F {
        return Ue4m3::Nan;
    }
    let exp = i32::from(byte >> 3);
    let man = f64::from(byte & 7);
    Ue4m3::Value(if exp == 0 { 2f64.powi(1 - 7) * (man / 8.0) } else { 2f64.powi(exp - 7) * (1.0 + man / 8.0) })
}
