//! Narrow-float MOVE bit-exactness vectors, increment 2: the SELECTION ops min/max. §6.16-0011
//! traces the fold to the observable output, and for a selection the output is a MOVED operand —
//! but which operand, and whether it is the NaN, depends on the arm (corrected after Baracuda's
//! sm_89 probe over-broadened by my first framing):
//!
//! - PROPAGATING (`max_prop`/`min_prop`): a NaN operand is SELECTED and moved → its payload and
//!   sign survive EXACTLY (§6.16-0009). This is where the native spelling breaks (promote-through-
//!   f32 quiets the moved NaN). Full payload-preservation vectors.
//! - IEEE (`fmax_ieee`/`fmin_ieee`): NaN-SUPPRESSING — one NaN operand in returns the OTHER
//!   (non-NaN) operand's bits. The NaN is DISCARDED, not moved, so a payload-preservation vector
//!   here would OVER-CONSTRAIN a conforming impl (Baracuda's max_ieee cells all AGREE natively).
//!   These cells pin the SUPPRESSION: the survivor's bits out. Their teeth are against an impl
//!   that wrongly PROPAGATES instead of suppressing.
//! - both-NaN under an IEEE arm is the one case a NaN survives: `f{max,min}_ieee(a,b)` returns
//!   `b` (§6.15 decomposition). Its own cell; the oracle authors it — NOT assumed equal to prop.
//!
//! Limits (stated): f16 authored but no promote-through-f32 demo (fp has no f16<->f32);
//! f8e4m3fn single NaN encoding → non-discriminating for the prop payload, authored anyway.

use kiss_conformance::corpus;
use kiss_conformance::fp::{
    bf16_to_f32, e4m3_to_f32, e5m2_to_f32, f32_to_bf16, f32_to_e4m3, f32_to_e5m2,
};
use kiss_conformance::nan_provenance::{compare_nan_output, is_nan, NanProvenance::Moved};
use kiss_conformance::semantics;

fn to_f32(dtype: &str, b: &[u8]) -> Option<f32> {
    Some(match dtype {
        "bf16" => bf16_to_f32(u16::from_be_bytes(b.try_into().ok()?)),
        "f8e5m2" => e5m2_to_f32(b[0]),
        "f8e4m3fn" => e4m3_to_f32(b[0]),
        _ => return None, // f16: no fp conversion
    })
}
fn from_f32(dtype: &str, x: f32) -> Vec<u8> {
    match dtype {
        "bf16" => f32_to_bf16(x).to_be_bytes().to_vec(),
        "f8e5m2" => vec![f32_to_e5m2(x)],
        "f8e4m3fn" => vec![f32_to_e4m3(x)],
        _ => unreachable!(),
    }
}

/// The raw-bit SELECTION oracle (§6.15 arms, applied to the moved bits): every cell here has at
/// least one NaN operand, so the NaN arm decides and no value comparison is needed.
fn raw_bit_select(op: &str, a: &[u8], b: &[u8], dtype: &str) -> Vec<u8> {
    let (an, bn) = (is_nan(dtype, a), is_nan(dtype, b));
    let pick = match op {
        // propagating: the NaN is selected (first NaN wins the decomposition).
        "max_prop" | "min_prop" => {
            if an {
                a
            } else if bn {
                b
            } else {
                panic!("select cell must have a NaN operand")
            }
        }
        // IEEE: the NaN is suppressed, the OTHER operand survives; both-NaN returns b.
        "fmax_ieee" | "fmin_ieee" => {
            if an {
                b
            } else if bn {
                a
            } else {
                panic!("select cell must have a NaN operand")
            }
        }
        _ => panic!("unknown select op {op}"),
    };
    pick.to_vec()
}

/// The promote-through-f32 model via the f32 selection semantics — the impl the prop vectors red.
fn promote_select(op: &str, a: &[u8], b: &[u8], dtype: &str) -> Option<Vec<u8>> {
    let (fa, fb) = (to_f32(dtype, a)?, to_f32(dtype, b)?);
    let r = match op {
        "max_prop" => semantics::max_prop(fa, fb),
        "min_prop" => semantics::min_prop(fa, fb),
        "fmax_ieee" => semantics::fmax_ieee(fa, fb),
        "fmin_ieee" => semantics::fmin_ieee(fa, fb),
        _ => panic!("unknown op {op}"),
    };
    Some(from_f32(dtype, r))
}

#[test]
fn narrow_select_nan_bundle_round_trips_and_discriminates() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/ops-narrow-select-nan.json"
    ))
    .expect("bundle committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert!(!c.vectors.is_empty());

    let mut ops = std::collections::BTreeSet::new();
    let mut dtypes = std::collections::BTreeSet::new();
    let mut prop_promote_reds = 0; // prop cells where promote-through-f32 quiets the moved NaN
    let mut ieee_suppress_agrees = 0; // IEEE cells where promote agrees (NOT over-constrained)
    let mut ieee_both_nan_pinned = 0; // IEEE both-NaN: wrong survivor (a) reds

    for v in &c.vectors {
        let where_ = format!("{} {} tc {}", v.op, v.dtype, v.tc_id);
        assert_eq!(v.inputs.len(), 2, "{where_}: selection ops are binary");
        ops.insert(v.op.clone());
        dtypes.insert(v.dtype.clone());
        let (a, b) = (&v.inputs[0], &v.inputs[1]);

        // 1. The emitted expected is exactly the selected operand's bits.
        let oracle = raw_bit_select(&v.op, a, b, &v.dtype);
        assert_eq!(v.expected, oracle, "{where_}: expected must be the raw-bit selected operand");

        let out_is_nan = is_nan(&v.dtype, &v.expected);
        if out_is_nan {
            assert_eq!(v.nan_provenance, Some(Moved), "{where_}: a moved NaN output is MOVED");
        } else {
            assert_eq!(v.nan_provenance, None, "{where_}: a finite (suppressed) output carries no provenance");
        }

        let prop = v.op == "max_prop" || v.op == "min_prop";
        let both_nan = is_nan(&v.dtype, a) && is_nan(&v.dtype, b);

        if let Some(promoted) = promote_select(&v.op, a, b, &v.dtype) {
            if prop && out_is_nan {
                // PROP moves the NaN; promote-through-f32 quiets it on a signaling payload → reds.
                if promoted != v.expected {
                    assert!(
                        compare_nan_output(&v.dtype, &promoted, &v.expected, Moved).is_err(),
                        "{where_}: a promote-through-f32 impl differs on a moved NaN → must RED"
                    );
                    prop_promote_reds += 1;
                }
            } else if !prop && !both_nan {
                // IEEE single-NaN: the survivor is finite; promote SUPPRESSES too → must AGREE.
                // (A payload-preservation assertion here would over-constrain a conforming impl.)
                assert_eq!(promoted, v.expected, "{where_}: IEEE suppression must not over-constrain — promote agrees");
                ieee_suppress_agrees += 1;
            }
        }

        // IEEE both-NaN pins the SURVIVOR IDENTITY (returns b): an impl returning a must RED.
        if !prop && both_nan {
            assert_eq!(v.expected, b.clone(), "{where_}: IEEE both-NaN returns b (§6.15)");
            let wrong = compare_nan_output(&v.dtype, a, &v.expected, Moved);
            if a != b {
                assert!(wrong.is_err(), "{where_}: returning a instead of b must RED");
                ieee_both_nan_pinned += 1;
            }
        }
    }

    for op in ["max_prop", "min_prop", "fmax_ieee", "fmin_ieee"] {
        assert!(ops.contains(op), "op {op} must be covered");
    }
    for d in ["bf16", "f16", "f8e5m2", "f8e4m3fn"] {
        assert!(dtypes.contains(d), "dtype {d} must be covered");
    }
    assert!(prop_promote_reds > 0, "the propagating-arm discrimination must fire (moved NaN quieted by promote)");
    assert!(ieee_suppress_agrees > 0, "the IEEE-suppression non-over-constraint must be demonstrated (promote agrees)");
    assert!(ieee_both_nan_pinned > 0, "the IEEE both-NaN survivor identity (returns b) must be pinned");
}
