// SPDX-License-Identifier: MIT OR Apache-2.0
//! The declared MathFidelity of a contract and the per-atom width probe that checks it
//! (KISS-CONTRACT §6.8-0004a, KISS-OPS §6.17-0003 / -0005; CireSnave decision 311, Q1 C).
//!
//! A contract's `math_fidelity` is `bit-stable` or `reduced-mantissa-permitted:<n>`: the second
//! carries the **declared width**, the fewest explicit input-mantissa bits the kernel guarantees to
//! retain. There is no suite-wide floor; the declaration is checked against the kernel's behaviour
//! by [`probe_retained_bits`]. bfloat16-input rounding is `n = 7`, TF32 is `n = 10`.

/// The explicit mantissa bits of an `f32`.
pub const F32_MANTISSA_BITS: u32 = 23;

/// The widest explicit mantissa any KISS float dtype stores (`f64`), the upper bound of a declared width.
pub const MAX_DECLARED_WIDTH: u32 = 52;

/// A parsed `math_fidelity` value (§6.8-0004a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredMathFidelity {
    BitStable,
    /// `reduced-mantissa-permitted:<n>`: no fewer than `n` explicit mantissa bits are retained.
    ReducedMantissa { width: u32 },
}

/// A typed decline of a `math_fidelity` value. Never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FidelityDecline {
    /// `reduced-mantissa-permitted` without `:<n>`: the width is mandatory, never defaulted.
    BareReducedMantissa,
    /// The `:<n>` part is not a decimal integer in `1..=52` without a sign or a leading zero.
    BadWidth(String),
    /// Not a member of the value grammar.
    Unknown(String),
}

/// Read a `math_fidelity` value (KISS-CONTRACT §6.8-0004a).
pub fn parse_math_fidelity(value: &str) -> Result<DeclaredMathFidelity, FidelityDecline> {
    const RM: &str = "reduced-mantissa-permitted";
    if value == "bit-stable" {
        return Ok(DeclaredMathFidelity::BitStable);
    }
    if value == RM {
        return Err(FidelityDecline::BareReducedMantissa);
    }
    let Some(width) = value.strip_prefix(RM).and_then(|r| r.strip_prefix(':')) else {
        return Err(FidelityDecline::Unknown(value.to_string()));
    };
    let canonical = !width.is_empty()
        && width.bytes().all(|b| b.is_ascii_digit())
        && !width.starts_with('0');
    match width.parse::<u32>() {
        Ok(n) if canonical && (1..=MAX_DECLARED_WIDTH).contains(&n) => {
            Ok(DeclaredMathFidelity::ReducedMantissa { width: n })
        }
        _ => Err(FidelityDecline::BadWidth(width.to_string())),
    }
}

/// Round `x` to `n` explicit mantissa bits, to nearest, ties to even, carrying into the exponent
/// (KISS-OPS §6.17-0006: a true round, never a truncation). `x` must be finite.
pub fn round_mantissa_rne(x: f32, n: u32) -> f32 {
    if n >= F32_MANTISSA_BITS {
        return x;
    }
    let drop = F32_MANTISSA_BITS - n;
    let bits = x.to_bits();
    let half = (1u32 << (drop - 1)) - 1;
    let lsb = (bits >> drop) & 1;
    f32::from_bits((bits.wrapping_add(half + lsb)) >> drop << drop)
}

/// The per-atom width probe (KISS-OPS §6.17-0005): feed a multiply odd-mantissa inputs (all 23
/// explicit bits potentially live) times `1.0` and report the **widest** mantissa among the results,
/// as the count of explicit bits up to the last set one. A multiply that rounds its inputs to `n`
/// bits returns results of at most `n`; one at full precision returns 23. A probe that finds no
/// narrowing at all reports 23, which satisfies every declared width (the value is a permission).
pub fn probe_retained_bits(mul: impl Fn(f32, f32) -> f32) -> u32 {
    let mut widest = 0;
    for i in 0..256u32 {
        // 1.xxx with the lowest mantissa bit set, the other bits varying with `i`.
        let x = f32::from_bits(0x3F80_0000 | (i.wrapping_mul(0x9E37) << 1 & 0x007F_FFFF) | 1);
        let m = mul(x, 1.0).to_bits() & 0x007F_FFFF;
        let retained = if m == 0 { 0 } else { F32_MANTISSA_BITS - m.trailing_zeros() };
        widest = widest.max(retained);
    }
    widest
}

/// Whether a kernel's probed behaviour honours its declaration. `bit-stable` requires full
/// storage precision; `reduced-mantissa-permitted:<n>` requires no narrowing below `n`.
pub fn declaration_honoured(declared: DeclaredMathFidelity, probed_bits: u32) -> bool {
    match declared {
        DeclaredMathFidelity::BitStable => probed_bits >= F32_MANTISSA_BITS,
        DeclaredMathFidelity::ReducedMantissa { width } => probed_bits >= width,
    }
}
