// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference for the conversion atom `cast` (KISS-Ops §6.21): the rounding modes, the saturation attribute,
//! the destinations whose own clause pins them, and the cases this version leaves unpinned (Appendix D item
//! 12), which these functions return `None` for rather than guess. Float-to-float is shown for `bf16`
//! (the format whose rounding is a shift) and the two `fp8` formats come from [`crate::fp`].

use crate::opattrs::{CastRounding, CastSaturation};

/// A conversion the destination's own clause forbids (§6.21-0002, §6.21-0003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastDecline {
    /// The destination pins nearest-even and no other mode (fp8 and the MX element types, §6.21-0002).
    RoundingNotDefined,
    /// The destination pins saturation, so `saturation = none` is not defined for it (§6.21-0003).
    SaturationRequired,
    /// A complex source or destination (§6.21-0001).
    UndefinedPair,
}

const SATURATING_DESTINATIONS: [&str; 5] = ["f8e4m3fn", "f8e5m2", "f4e2m1", "f6e2m3", "f6e3m2"];

/// Whether `rounding`/`saturation` are legal for destination `dst` (and source `src`), per §6.21.
pub fn check_cast(src: &str, dst: &str, rounding: CastRounding, saturation: CastSaturation) -> Result<(), CastDecline> {
    let complex = |d: &str| d == "c64" || d == "c128";
    if complex(src) || complex(dst) {
        return Err(CastDecline::UndefinedPair);
    }
    if SATURATING_DESTINATIONS.contains(&dst) {
        if rounding != CastRounding::NearestEven {
            return Err(CastDecline::RoundingNotDefined);
        }
        if saturation != CastSaturation::Clamp {
            return Err(CastDecline::SaturationRequired);
        }
    }
    Ok(())
}

/// `f32` to `bf16` bits under `rounding` and `saturation` (§6.21-0002/-0003). A NaN stays a NaN with its sign
/// kept; an infinity clamps to the largest finite value under `Clamp`.
pub fn f32_to_bf16(x: f32, rounding: CastRounding, saturation: CastSaturation) -> u16 {
    let bits = x.to_bits();
    let sign = (bits >> 31) as u16;
    let upper = (bits >> 16) as u16;
    let low = bits & 0xFFFF;
    if x.is_nan() {
        return (sign << 15) | 0x7FC0 | (upper & 0x003F);
    }
    let max_finite = (sign << 15) | 0x7F7F;
    let inf = (sign << 15) | 0x7F80;
    if x.is_infinite() {
        return if saturation == CastSaturation::Clamp { max_finite } else { inf };
    }
    let mag = upper & 0x7FFF;
    let bump = match rounding {
        CastRounding::NearestEven => low > 0x8000 || (low == 0x8000 && mag & 1 == 1),
        CastRounding::TowardZero => false,
        CastRounding::TowardNegInf => low != 0 && sign == 1,
        CastRounding::TowardPosInf => low != 0 && sign == 0,
    };
    let mag = if bump { mag + 1 } else { mag };
    if mag >= 0x7F80 {
        // the finite source rounded past the largest finite bf16
        return if saturation == CastSaturation::Clamp { max_finite } else { inf };
    }
    (sign << 15) | mag
}

/// `f64` to `bf16` bits, nearest-even, **one** rounding of the exact source (§6.21-0002), for values in the
/// normal `bf16` range; `None` outside it. Staging through `f32` can round twice and gives a different
/// answer for some inputs, which is the non-conformance the clause names.
pub fn f64_to_bf16_nearest_even(x: f64) -> Option<u16> {
    let b = x.to_bits();
    let sign = (b >> 63) as u16;
    let e = ((b >> 52) & 0x7FF) as i32 - 1023 + 127;
    if !(1..=254).contains(&e) {
        return None;
    }
    let m = b & ((1u64 << 52) - 1);
    let keep = (m >> 45) as u16;
    let rem = m & ((1u64 << 45) - 1);
    let half = 1u64 << 44;
    let up = rem > half || (rem == half && keep & 1 == 1);
    let mag = ((e as u16) << 7 | keep) + u16::from(up);
    Some((sign << 15) | mag)
}

fn int_range(bits: u32, signed: bool) -> (i128, i128) {
    if signed {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    } else {
        (0, (1i128 << bits) - 1)
    }
}

/// Integer to integer (§6.21-0003): `Clamp` takes the nearest bound, `None` takes the value modulo 2^bits.
pub fn int_to_int(v: i64, bits: u32, signed: bool, saturation: CastSaturation) -> i64 {
    let (lo, hi) = int_range(bits, signed);
    let v = i128::from(v);
    match saturation {
        CastSaturation::Clamp => v.clamp(lo, hi) as i64,
        CastSaturation::None => {
            let m = 1i128 << bits;
            let w = v.rem_euclid(m);
            (if signed && w > hi { w - m } else { w }) as i64
        }
    }
}

/// Float to integer. `None` for the cases §6.21-0004 leaves unpinned: a NaN, and an out-of-range value under
/// `saturation = none`. The recommended values (NaN to 0, always clamp) are Appendix D item 12, not normative.
pub fn f32_to_int(x: f32, rounding: CastRounding, saturation: CastSaturation, bits: u32, signed: bool) -> Option<i64> {
    if x.is_nan() {
        return None;
    }
    let r = match rounding {
        CastRounding::NearestEven => x.round_ties_even(),
        CastRounding::TowardZero => x.trunc(),
        CastRounding::TowardNegInf => x.floor(),
        CastRounding::TowardPosInf => x.ceil(),
    };
    let (lo, hi) = int_range(bits, signed);
    if r >= lo as f32 && r <= hi as f32 && (r as f64) >= lo as f64 && (r as f64) <= hi as f64 {
        return Some(r as i64);
    }
    match saturation {
        CastSaturation::Clamp => Some(if r < 0.0 { lo } else { hi } as i64),
        CastSaturation::None => None,
    }
}
