//! Narrow-float MOVE bit-exactness vectors (§6.16-0009 / KISS-OPS-6.16-0011), increment 1:
//! the sign-edit ops neg/abs/copysign. A narrow-float sign edit MOVES the operand's bits — a
//! moved NaN's payload and (post-edit) sign survive EXACTLY. There is NO decline member for this
//! class (a conforming impl spells the op raw-bit; it does not refuse), so the VECTOR is the
//! enforcement. Two independent lanes reached this the same night: Unpopped's published fp8
//! neg/abs/copysign promoted through f32 and destroyed a NaN's sign; Baracuda measured sm_89
//! natively destroying payloads (`-bf16(0x7F81)`=`0x7FFF`). The failure mode is real, not
//! hypothetical.
//!
//! THE DISCRIMINATION (what a reviewer checks): the set must RED against an impl that promotes
//! through f32 and rounds back, and GREEN against one that spells the op raw-bit. Demonstrated
//! two ways: (1) promote-through-f32 QUIETS a signaling NaN → reds the signaling cells on the
//! dtypes fp can convert (bf16, e5m2); (2) a CANONICALIZING impl (fixed qNaN payload) → reds the
//! RICH-payload cells on every dtype (no conversion needed — this is why the payload is varied,
//! per §6.16-0011's note that a one-bit payload cannot tell canonicalizing from truncating apart).
//!
//! POPULATION / LIMITS, stated as results not omissions:
//! - f16: authored (raw-bit sign edit needs no conversion), but fp has NO f16<->f32, so the
//!   promote-through-f32 demo is not run for it; the canonicalizing demo still is.
//! - f8e4m3fn: single NaN encoding (§6.16-0004) → no signaling NaN and no payload → the MOVE is
//!   non-discriminating (promote == raw-bit); authored for completeness, flagged vacuous.

use kiss_conformance::corpus;
use kiss_conformance::fp::{
    bf16_to_f32, e4m3_to_f32, e5m2_to_f32, f32_to_bf16, f32_to_e4m3, f32_to_e5m2,
};
use kiss_conformance::nan_provenance::{admits_snan, compare_nan_output, is_nan, NanProvenance::Moved};

/// Width in bytes of a narrow-float dtype.
fn width(dtype: &str) -> usize {
    match dtype {
        "bf16" | "f16" => 2,
        "f8e5m2" | "f8e4m3fn" => 1,
        _ => panic!("not a narrow float: {dtype}"),
    }
}

/// The raw-bit MOVE oracle (§6.16-0009): the sign bit is the MSB of the big-endian byte 0. neg
/// flips it, abs clears it, copysign takes a's magnitude and b's sign. Every OTHER bit is
/// preserved, so a moved NaN's payload survives. Re-derived here independently of the minter.
fn raw_bit_move(dtype: &str, op: &str, a: &[u8], b: Option<&[u8]>) -> Vec<u8> {
    let mut out = a.to_vec();
    assert_eq!(out.len(), width(dtype));
    match op {
        "neg" => out[0] ^= 0x80,
        "abs" => out[0] &= 0x7F,
        "copysign" => {
            out[0] &= 0x7F;
            out[0] |= b.expect("copysign needs b")[0] & 0x80;
        }
        _ => panic!("unknown sign-edit op {op}"),
    }
    out
}

/// The promote-through-f32-and-round model — the WRONG impl the vectors must red. `None` for a
/// dtype fp cannot convert (f16).
fn promote_through_f32(dtype: &str, op: &str, a: &[u8], b: Option<&[u8]>) -> Option<Vec<u8>> {
    let to_f32 = |bytes: &[u8]| -> Option<f32> {
        Some(match dtype {
            "bf16" => bf16_to_f32(u16::from_be_bytes(bytes.try_into().ok()?)),
            "f8e5m2" => e5m2_to_f32(bytes[0]),
            "f8e4m3fn" => e4m3_to_f32(bytes[0]),
            _ => return None,
        })
    };
    let from_f32 = |x: f32| -> Vec<u8> {
        match dtype {
            "bf16" => f32_to_bf16(x).to_be_bytes().to_vec(),
            "f8e5m2" => vec![f32_to_e5m2(x)],
            "f8e4m3fn" => vec![f32_to_e4m3(x)],
            _ => unreachable!(),
        }
    };
    let fa = to_f32(a)?;
    let r = match op {
        "neg" => -fa,
        "abs" => fa.abs(),
        "copysign" => fa.copysign(to_f32(b?)?),
        _ => panic!("unknown op {op}"),
    };
    Some(from_f32(r))
}

/// The dtype's canonical quiet NaN (positive) — the output of a CANONICALIZING impl that ignores
/// the operand's payload. Reds any rich-payload cell without needing an f32 conversion.
fn canonical_qnan(dtype: &str) -> Vec<u8> {
    match dtype {
        "bf16" => 0x7FC0u16.to_be_bytes().to_vec(),
        "f16" => 0x7E00u16.to_be_bytes().to_vec(),
        "f8e5m2" => vec![0x7E],
        "f8e4m3fn" => vec![0x7F],
        _ => panic!("{dtype}"),
    }
}

#[test]
fn narrow_move_nan_bundle_round_trips_and_discriminates() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/ops-narrow-move-nan.json"
    ))
    .expect("bundle committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert!(!c.vectors.is_empty());

    let (mut saw_neg, mut saw_abs, mut saw_copysign) = (false, false, false);
    let (mut saw_signaling, mut saw_finite_control) = (false, false);
    let mut dtypes = std::collections::BTreeSet::new();
    let mut promoted_reds = 0; // signaling-quieting discriminations actually run
    let mut canon_reds = 0; // rich-payload canonicalizing discriminations actually run

    for v in &c.vectors {
        let where_ = format!("{} {} tc {}", v.op, v.dtype, v.tc_id);
        dtypes.insert(v.dtype.clone());
        match v.op.as_str() {
            "neg" => saw_neg = true,
            "abs" => saw_abs = true,
            "copysign" => saw_copysign = true,
            other => panic!("{where_}: unexpected op {other}"),
        }
        let b = v.inputs.get(1).map(|x| x.as_slice());

        // 1. RAW-BIT oracle: the emitted expected is exactly the sign-edited input.
        let oracle = raw_bit_move(&v.dtype, &v.op, &v.inputs[0], b);
        assert_eq!(v.expected, oracle, "{where_}: expected must be the raw-bit sign edit");

        if is_nan(&v.dtype, &v.expected) {
            // 2a. A moved NaN carries MOVED provenance; the comparator is exact-byte.
            assert_eq!(v.nan_provenance, Some(Moved), "{where_}: NaN output must be MOVED");
            assert!(
                compare_nan_output(&v.dtype, &v.expected, &oracle, Moved).is_ok(),
                "{where_}: a raw-bit impl (bits == expected) GREENs"
            );

            // 3a. DISCRIMINATION — promote-through-f32 quiets a signaling NaN → reds.
            if let Some(promoted) = promote_through_f32(&v.dtype, &v.op, &v.inputs[0], b) {
                if promoted != v.expected {
                    assert!(
                        compare_nan_output(&v.dtype, &promoted, &v.expected, Moved).is_err(),
                        "{where_}: a promote-through-f32 impl differs, so it must RED"
                    );
                    promoted_reds += 1;
                }
            }
            // 3b. DISCRIMINATION — a canonicalizing impl reds a rich payload (no conversion needed).
            let canon = canonical_qnan(&v.dtype);
            if canon != v.expected {
                assert!(
                    compare_nan_output(&v.dtype, &canon, &v.expected, Moved).is_err(),
                    "{where_}: a canonicalizing impl (fixed payload) must RED a rich payload"
                );
                canon_reds += 1;
            }
            // signaling input present? (quiet bit is the MSB of the trailing significand)
            if admits_snan(&v.dtype) && is_nan(&v.dtype, &v.inputs[0]) {
                let bits = v.inputs[0].iter().fold(0u32, |a, &x| (a << 8) | x as u32);
                let mant_bits = match v.dtype.as_str() {
                    "bf16" => 7,
                    "f16" => 10,
                    "f8e5m2" => 2,
                    _ => 0,
                };
                if mant_bits > 0 && (bits >> (mant_bits - 1)) & 1 == 0 {
                    saw_signaling = true;
                }
            }
        } else {
            // 2b. A finite MOVE output carries NO provenance (biconditional), and a
            // promote-through-f32 impl passes it — the NEGATIVE CONTROL: the set does not red
            // on everything.
            assert_eq!(v.nan_provenance, None, "{where_}: finite output must carry no provenance");
            saw_finite_control = true;
            if let Some(promoted) = promote_through_f32(&v.dtype, &v.op, &v.inputs[0], b) {
                assert_eq!(promoted, v.expected, "{where_}: a finite value passes promote-through-f32 too");
            }
        }
    }

    assert!(saw_neg && saw_abs && saw_copysign, "all three sign-edit ops present");
    for d in ["bf16", "f16", "f8e5m2", "f8e4m3fn"] {
        assert!(dtypes.contains(d), "dtype {d} must be covered");
    }
    assert!(saw_signaling, "at least one signaling-NaN input present");
    assert!(saw_finite_control, "at least one finite negative-control cell present");
    assert!(promoted_reds > 0, "the promote-through-f32 discrimination must actually fire (signaling cells)");
    assert!(canon_reds > 0, "the canonicalizing discrimination must actually fire (rich-payload cells)");
}
