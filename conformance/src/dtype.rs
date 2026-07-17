//! From-scratch storage codec for the pinned **sub-byte / FP8 dtype layouts**
//! (KISS-Classify §6.1-0008 `s4`/`u4` nibble packing, §6.1-0009 `b1` bit packing,
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

/// Sign-extend a 4-bit nibble (`0..=15`) to `i8` as an `s4` value: nibbles
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

/// Pack a run of `s4` logical values into packed-pair bytes: even index → low
/// nibble, odd index → high nibble (§6.1-0008). A trailing lone even value
/// occupies the low nibble with the high nibble zero-filled. Each value is
/// truncated to its low 4 bits (its two's-complement `s4` pattern).
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

/// Unpack `count` `s4` logical values from packed-pair bytes: index `k` reads
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