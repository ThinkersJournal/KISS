//! Per-row NaN-output provenance comparison (Conform §6.8-0010 Layer-2 + §6.16-0010).
//!
//! A NaN *output* is compared by HOW it arose, per row, not by the op class:
//!
//! - MOVED (§6.8-0010(a)): a raw-bit permutation / sign-transform (select, gather,
//!   bitcast, neg, abs, copysign) whose NaN output is a moved input value — compare
//!   EXACT BYTES (payload AND sign preserved).
//! - COMPUTED (§6.16-0010): an arithmetic/transcendental minted NaN — the NaN-ness
//!   must match, and (where the dtype's encoding admits a signaling NaN) the
//!   quietness must match the expected. Payload and sign are NOT compared (no unique
//!   correct payload exists across conformant implementations; §6.5-0001).
//!
//! ⚠️ The vacuity guard is the load-bearing subtlety (kiss-ref caught it, §6.16-0004):
//! `f8e4m3fn` has a SINGLE NaN encoding and admits no signaling NaN, so for it the
//! quietness obligation is VACUOUS — a comparator MUST NOT synthesize a quiet/
//! signaling distinction the format cannot represent, or it over-enforces and fails
//! a conformant `f8e4m3fn` implementation. So the COMPUTED arm is
//! `is-NaN ∧ (admits_snan(dtype) ⟹ quiet(actual) == quiet(expected))`.
//!
//! Bytes are big-endian at the dtype's native width (as the corpus stores them).

/// Which side of the moved/computed split a NaN output falls on. Carried per row
/// (a Cell field); a NaN output with no provenance is rejected at load, so this is
/// never "unknown" by the time a comparison runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NanProvenance {
    Moved,
    Computed,
}

/// Bit layout of a float dtype: total width and the number of trailing-significand
/// (mantissa) bits. The exponent field is everything between the sign and the
/// mantissa, and is all-ones for Inf/NaN.
struct Layout {
    width: u32,
    mant_bits: u32,
    /// `f8e4m3fn` is the sole exception: a single NaN encoding (S.1111.111), no sNaN.
    single_nan_encoding: bool,
}

fn layout(dtype: &str) -> Option<Layout> {
    let l = match dtype {
        "f64" => Layout { width: 64, mant_bits: 52, single_nan_encoding: false },
        "f32" => Layout { width: 32, mant_bits: 23, single_nan_encoding: false },
        "f16" => Layout { width: 16, mant_bits: 10, single_nan_encoding: false },
        "bf16" => Layout { width: 16, mant_bits: 7, single_nan_encoding: false },
        "f8e5m2" => Layout { width: 8, mant_bits: 2, single_nan_encoding: false },
        "f8e4m3fn" => Layout { width: 8, mant_bits: 3, single_nan_encoding: true },
        _ => return None,
    };
    Some(l)
}

/// `true` iff this dtype's encoding admits a signaling NaN. All IEEE-style dtypes do
/// EXCEPT `f8e4m3fn` (single NaN encoding, §6.16-0004) — the guard the COMPUTED arm
/// keys on so it does not fabricate a distinction the format cannot represent.
pub fn admits_snan(dtype: &str) -> bool {
    layout(dtype).is_some_and(|l| !l.single_nan_encoding)
}

/// The dtype's raw bits, from big-endian bytes of exactly its native width.
fn bits_be(l: &Layout, be: &[u8]) -> Option<u64> {
    if be.len() as u32 != l.width / 8 {
        return None;
    }
    Some(be.iter().fold(0u64, |acc, &b| (acc << 8) | b as u64))
}

/// `true` iff `be` (big-endian, native width) encodes a NaN in `dtype`.
pub fn is_nan(dtype: &str, be: &[u8]) -> bool {
    let Some(l) = layout(dtype) else { return false };
    let Some(bits) = bits_be(&l, be) else { return false };
    if l.single_nan_encoding {
        // f8e4m3fn: the ONLY NaN is exp=1111 ∧ mant=111 (S.1111.111); bits & 0x7F == 0x7F.
        return bits & ((1 << (l.width - 1)) - 1) == (1 << (l.width - 1)) - 1;
    }
    let exp_bits = l.width - 1 - l.mant_bits;
    let exp_mask = (1u64 << exp_bits) - 1;
    let exp = (bits >> l.mant_bits) & exp_mask;
    let mant = bits & ((1u64 << l.mant_bits) - 1);
    exp == exp_mask && mant != 0
}

/// `true` iff the NaN's quiet bit (MSB of the trailing significand) is set. Only
/// meaningful when `be` is a NaN in a dtype that admits an sNaN.
fn quiet_set(l: &Layout, bits: u64) -> bool {
    let quiet_bit = l.mant_bits - 1; // MSB of the mantissa field
    (bits >> quiet_bit) & 1 == 1
}

/// Compare a NaN-output row by its provenance. `dtype` bytes are big-endian native.
/// Returns `Ok(())` on a conformant match, `Err(reason)` otherwise.
///
/// Precondition (enforced upstream by the load-time biconditional): this is called
/// only when the EXPECTED output is a NaN, with the row's carried provenance.
pub fn compare_nan_output(
    dtype: &str,
    actual: &[u8],
    expected: &[u8],
    prov: NanProvenance,
) -> Result<(), String> {
    match prov {
        // §6.8-0010(a): a moved NaN's bytes ARE the contract — payload and sign.
        // (actual vs EXPECTED, both carrying the post-sign-edit bits for neg/abs/
        // copysign, so this is byte-exact, NOT "result == input".)
        NanProvenance::Moved => {
            if actual == expected {
                Ok(())
            } else {
                Err(format!(
                    "moved-NaN mismatch ({dtype}): bytes differ — payload/sign not preserved \
                     (actual {actual:02X?}, expected {expected:02X?})"
                ))
            }
        }
        // §6.16-0010: NaN-ness always; quietness only where the format admits sNaN.
        NanProvenance::Computed => {
            let l = layout(dtype).ok_or_else(|| format!("unknown dtype `{dtype}`"))?;
            if !is_nan(dtype, actual) {
                return Err(format!("computed-NaN mismatch ({dtype}): actual is not a NaN"));
            }
            if !is_nan(dtype, expected) {
                return Err(format!("computed-NaN test error ({dtype}): expected is not a NaN"));
            }
            if l.single_nan_encoding {
                // Vacuous quietness: f8e4m3fn cannot represent the distinction. is-NaN
                // is the whole obligation — MUST NOT synthesize a quiet/signaling split.
                return Ok(());
            }
            let (Some(ab), Some(eb)) = (bits_be(&l, actual), bits_be(&l, expected)) else {
                return Err(format!("computed-NaN ({dtype}): byte width mismatch"));
            };
            if quiet_set(&l, ab) == quiet_set(&l, eb) {
                Ok(())
            } else {
                Err(format!(
                    "computed-NaN quietness mismatch ({dtype}): actual quiet={}, expected quiet={} \
                     (§6.16-0010 — quietness must agree where the format admits an sNaN)",
                    quiet_set(&l, ab),
                    quiet_set(&l, eb)
                ))
            }
        }
    }
}
