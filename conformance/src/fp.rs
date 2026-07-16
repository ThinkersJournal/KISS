//! From-scratch f32 <-> reduced-float converters for the three non-IEEE storage
//! dtypes KISS-Ops pins directly in §6.16: `bf16` (§6.16-0003), `e4m3`
//! (§6.16-0004, OCP OFP8), and `e5m2` (§6.16-0005, OCP OFP8).
//!
//! This is a validating reference oracle: every encoder reproduces the exact
//! byte a conformant op must emit, computed from the spec-pinned bit layout and
//! the pinned rounding rule (round-to-nearest, ties-to-even) with no dependency
//! on Rust's `f16`/fp8 support (there is none in std). The three formats differ
//! in ways the spec makes load-bearing, and this module encodes each difference:
//!
//!   * **bf16** shares binary32's 8-bit exponent range, so it is *not* saturating
//!     — an f32 that overflows the 7-bit mantissa at the top binade rounds up to
//!     **infinity**, exactly like binary32. The pinned rule is RNE, so the naive
//!     truncation `(bits >> 16)` is wrong (§6.16-0003).
//!   * **e4m3** (OCP OFP8) has **no infinity encoding** and a **single NaN**
//!     (`S.1111.111`); its max finite is ±448 (`S.1111.110`), *not* ±480, because
//!     the all-ones-mantissa slot at the top exponent is the NaN. Conversion
//!     **saturates** overflow to ±448 (§6.16-0004).
//!   * **e5m2** (OCP OFP8) *does* have IEEE-style inf/NaN at exponent field 31,
//!     yet f32->e5m2 conversion of an overflowing **finite** value still
//!     **saturates** to ±57344 (`S.11110.11`) rather than emitting infinity
//!     (§6.16-0005).
//!
//! All encoders round-half-to-even and preserve the sign of zero.

// ---------------------------------------------------------------------------
// bf16 : 1 sign / 8 exp / 7 mantissa, bias 127 — truncated binary32 with RNE.
// §6.16-0003. Same exponent range as f32 (NOT saturating): mantissa overflow at
// the top binade carries to infinity, exactly as binary32 does.
// ---------------------------------------------------------------------------

/// Encode an `f32` as the 16-bit `bf16` pattern, round-to-nearest-ties-to-even
/// (§6.16-0003). `bf16` is the high 16 bits of the binary32 encoding with the
/// low 16 bits *rounded* in, never truncated.
pub fn f32_to_bf16(x: f32) -> u16 {
    let bits = x.to_bits();
    if x.is_nan() {
        // Not "rounding": propagate as a quiet bf16 NaN (sign kept, mantissa MSB
        // forced set so the result cannot degenerate to an infinity).
        return ((bits >> 16) as u16) | 0x0040;
    }
    // Round-to-nearest-ties-to-even by adding half-an-ULP-minus-one plus the
    // result's own lsb, then truncating: a tie (low 16 bits == 0x8000) carries
    // iff the retained bit is odd, which is exactly ties-to-even. A carry out of
    // the mantissa propagates naturally into the (f32-range) exponent, so the
    // top binade overflows to infinity — bf16 is NOT saturating.
    let lsb = (bits >> 16) & 1;
    let rounding_bias = 0x7fff + lsb;
    (bits.wrapping_add(rounding_bias) >> 16) as u16
}

/// Decode a `bf16` pattern to `f32` — an exact widening (zero-extend the low 16
/// mantissa bits); `bf16`'s exponent range is binary32's, so no value is lost.
pub fn bf16_to_f32(b: u16) -> f32 {
    f32::from_bits((b as u32) << 16)
}

// ---------------------------------------------------------------------------
// Shared FP8 magnitude rounder (RNE, saturating). Used by both e4m3 and e5m2.
// ---------------------------------------------------------------------------

/// Right-shift `value` by `drop` bits, rounding to nearest with ties to even.
fn rne_shift(value: u32, drop: u32) -> u32 {
    if drop == 0 {
        return value;
    }
    let shifted = value >> drop;
    let rem = value & ((1u32 << drop) - 1);
    let half = 1u32 << (drop - 1);
    if rem > half || (rem == half && (shifted & 1) == 1) {
        shifted + 1
    } else {
        shifted
    }
}

/// Round the magnitude of a **finite, non-negative** `mag` to the 7-bit
/// magnitude field of an OCP OFP8 format with `mbits` mantissa bits and exponent
/// `bias`, saturating to `max_finite` (the encoded magnitude byte of the largest
/// finite value). Round-half-to-even throughout (§6.16-0004 / §6.16-0005).
fn fp8_magnitude(mag: f32, mbits: u32, bias: i32, max_finite: u32) -> u8 {
    if mag == 0.0 {
        return 0;
    }
    let bits = mag.to_bits();
    let exp32 = ((bits >> 23) & 0xff) as i32;
    let man32 = bits & 0x7f_ffff;
    // Normalize to  value = sig * 2^(e - 23),  sig in [2^23, 2^24).
    let (e, sig): (i32, u32) = if exp32 == 0 {
        // f32 subnormal (man32 != 0 here since mag != 0): shift MSB to bit 23.
        let lz = man32.leading_zeros();
        (-118 - lz as i32, man32 << (lz - 8))
    } else {
        (exp32 - 127, (1u32 << 23) | man32)
    };
    let emin_n = 1 - bias; // unbiased exponent of the smallest normal
    // Bits to drop: 23 - mbits for a normal target; extra (emin_n - e) more when
    // the value lands in the target's subnormal range.
    let drop = if e >= emin_n {
        23 - mbits as i32
    } else {
        (23 - mbits as i32) + (emin_n - e)
    };
    // drop <= 24 whenever the value can reach the smallest subnormal; anything
    // needing a larger shift is below half the smallest subnormal -> rounds to 0.
    let rounded = if drop >= 25 { 0 } else { rne_shift(sig, drop as u32) };
    let magnitude: u32 = if e >= emin_n {
        // Normal: rounded in [2^mbits, 2^(mbits+1)]; a mantissa carry bumps the
        // exponent field, which (on overflow) exceeds max_finite and saturates.
        (((e + bias) as u32) << mbits) + (rounded - (1u32 << mbits))
    } else {
        // Subnormal: the magnitude byte IS the rounded value; a round-up to
        // 2^mbits promotes cleanly to the smallest normal (exp field 1, mant 0).
        rounded
    };
    magnitude.min(max_finite) as u8
}

// ---------------------------------------------------------------------------
// e4m3 : 1 sign / 4 exp / 3 mantissa, bias 7. Max finite ±448 (S.1111.110).
// NO infinity encoding; single NaN (S.1111.111). Saturating. §6.16-0004.
// ---------------------------------------------------------------------------

/// Encoded 7-bit magnitude of the largest finite `e4m3` value, ±448 = `S.1111.110`.
pub const E4M3_MAX_FINITE_MAG: u8 = 0x7E;
/// The single `e4m3` NaN magnitude, `S.1111.111`.
pub const E4M3_NAN_MAG: u8 = 0x7F;

/// Encode an `f32` as an `e4m3` (OCP OFP8) byte, round-half-to-even, **saturating**
/// overflow to ±448 — never emitting an infinity, because e4m3 has none
/// (§6.16-0004). NaN maps to the single NaN encoding.
pub fn f32_to_e4m3(x: f32) -> u8 {
    let sign = if x.is_sign_negative() { 0x80 } else { 0 };
    if x.is_nan() {
        return sign | E4M3_NAN_MAG;
    }
    if x.is_infinite() {
        // e4m3 has no infinity encoding; saturate. (An infinite *input* is not a
        // case §6.16-0004 pins — it pins finite overflow — so this is a
        // totality choice, not a tested normative fact.)
        return sign | E4M3_MAX_FINITE_MAG;
    }
    sign | fp8_magnitude(x.abs(), 3, 7, E4M3_MAX_FINITE_MAG as u32)
}

/// Decode an `e4m3` byte to `f32`. `S.1111.111` is the single NaN; there is no
/// infinity, so `S.1111.110` (±448) is the largest magnitude and every other
/// top-exponent code is an ordinary finite value.
pub fn e4m3_to_f32(b: u8) -> f32 {
    let sign = if b & 0x80 != 0 { -1.0 } else { 1.0 };
    let e = ((b >> 3) & 0x0F) as i32;
    let m = (b & 0x07) as i32;
    if e == 15 && m == 7 {
        return f32::NAN; // single NaN encoding (both signs)
    }
    let val = if e == 0 {
        // subnormal: 2^(1-7) * (m/8) = 2^-6 * m/8
        (m as f32) * 2f32.powi(-6) / 8.0
    } else {
        (1.0 + m as f32 / 8.0) * 2f32.powi(e - 7)
    };
    sign * val
}

// ---------------------------------------------------------------------------
// e5m2 : 1 sign / 5 exp / 2 mantissa, bias 15. Max finite ±57344 (S.11110.11).
// HAS IEEE-style inf/NaN at exp field 31. f32->e5m2 still SATURATES finite
// overflow to ±57344 rather than emitting inf. §6.16-0005.
// ---------------------------------------------------------------------------

/// Encoded 7-bit magnitude of the largest finite `e5m2` value, ±57344 = `S.11110.11`.
pub const E5M2_MAX_FINITE_MAG: u8 = 0x7B;
/// The `e5m2` positive-infinity magnitude, `S.11111.00`.
pub const E5M2_INF_MAG: u8 = 0x7C;
/// A quiet `e5m2` NaN magnitude, `S.11111.10`.
pub const E5M2_NAN_MAG: u8 = 0x7E;

/// Encode an `f32` as an `e5m2` (OCP OFP8) byte, round-half-to-even. A finite
/// value that overflows the format **saturates** to ±57344 rather than emitting
/// an infinity (§6.16-0005), even though e5m2 *has* an infinity encoding. An
/// infinite input maps to the infinity encoding (the format has one).
pub fn f32_to_e5m2(x: f32) -> u8 {
    let sign = if x.is_sign_negative() { 0x80 } else { 0 };
    if x.is_nan() {
        return sign | E5M2_NAN_MAG;
    }
    if x.is_infinite() {
        return sign | E5M2_INF_MAG;
    }
    sign | fp8_magnitude(x.abs(), 2, 15, E5M2_MAX_FINITE_MAG as u32)
}

/// Decode an `e5m2` byte to `f32`. Exponent field 31 is IEEE-style: mantissa 0 is
/// infinity, any nonzero mantissa is NaN.
pub fn e5m2_to_f32(b: u8) -> f32 {
    let sign = if b & 0x80 != 0 { -1.0 } else { 1.0 };
    let e = ((b >> 2) & 0x1F) as i32;
    let m = (b & 0x03) as i32;
    if e == 31 {
        return if m == 0 { sign * f32::INFINITY } else { f32::NAN };
    }
    let val = if e == 0 {
        // subnormal: 2^(1-15) * (m/4) = 2^-14 * m/4
        (m as f32) * 2f32.powi(-14) / 4.0
    } else {
        (1.0 + m as f32 / 4.0) * 2f32.powi(e - 15)
    };
    sign * val
}
