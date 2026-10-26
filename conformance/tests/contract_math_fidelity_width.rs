// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CONTRACT §6.8-0004a (the declared width of `reduced-mantissa-permitted`) and the
//! KISS-OPS §6.17-0003 / -0005 check that a declaration is honoured. bfloat16-input rounding is
//! expressible (`n = 7`); a declaration the probe contradicts is caught.

use kiss_conformance::fidelity::{
    declaration_honoured, parse_math_fidelity, probe_retained_bits, round_mantissa_rne,
    DeclaredMathFidelity, FidelityDecline,
};

fn full(a: f32, b: f32) -> f32 {
    a * b
}
fn tf32(a: f32, b: f32) -> f32 {
    round_mantissa_rne(a, 10) * round_mantissa_rne(b, 10)
}
fn bf16_inputs(a: f32, b: f32) -> f32 {
    round_mantissa_rne(a, 7) * round_mantissa_rne(b, 7)
}

/// KISS-CONTRACT-6.8-0004a: `bit-stable` or `reduced-mantissa-permitted:<n>`; the width is mandatory.
#[test]
fn test_contract_math_fidelity_declared_width() {
    // Clause: KISS-CONTRACT-6.8-0004a.
    assert_eq!(parse_math_fidelity("bit-stable"), Ok(DeclaredMathFidelity::BitStable));
    for (text, n) in [("reduced-mantissa-permitted:7", 7), ("reduced-mantissa-permitted:10", 10)] {
        assert_eq!(parse_math_fidelity(text), Ok(DeclaredMathFidelity::ReducedMantissa { width: n }));
    }
    assert_eq!(parse_math_fidelity("reduced-mantissa-permitted"), Err(FidelityDecline::BareReducedMantissa));
    for bad in ["0", "53", "07", "+7", "7.0", "", "-1", " 7"] {
        let v = format!("reduced-mantissa-permitted:{bad}");
        assert_eq!(parse_math_fidelity(&v), Err(FidelityDecline::BadWidth(bad.to_string())), "{v:?}");
    }
    for unknown in ["tf32", "st", "rm", "bit-stable:7", "Reduced-Mantissa-Permitted:7"] {
        assert!(matches!(parse_math_fidelity(unknown), Err(FidelityDecline::Unknown(_))), "{unknown}");
    }
}

/// KISS-OPS-6.17-0003 and -0005: the declared width, not a pinned floor, is what the per-atom width
/// probe checks, so a bfloat16-input kernel is admitted at `n = 7` and rejected at `n = 10`.
#[test]
fn test_ops_math_precision_reduced() {
    // Clause: KISS-OPS-6.17-0003.
    let (p_full, p_tf32, p_bf16) = (probe_retained_bits(full), probe_retained_bits(tf32), probe_retained_bits(bf16_inputs));
    assert_eq!((p_full, p_tf32, p_bf16), (23, 10, 7), "the probe reads each target's input width");

    let rm = |n| DeclaredMathFidelity::ReducedMantissa { width: n };
    // bfloat16-input rounding is expressible: declared at 7 it is honoured; the old floor of 10 is gone.
    assert!(declaration_honoured(rm(7), p_bf16));
    assert!(!declaration_honoured(rm(10), p_bf16), "declaring 10 over a 7-bit kernel is caught");
    // TF32 honours 10 and 7; a full-precision kernel honours any width (a permission, never an obligation).
    assert!(declaration_honoured(rm(10), p_tf32) && declaration_honoured(rm(7), p_tf32));
    assert!(!declaration_honoured(rm(11), p_tf32));
    assert!(declaration_honoured(rm(23), p_full));
    // bit-stable requires full storage precision: a TF32 or bf16 kernel cannot claim it.
    assert!(declaration_honoured(DeclaredMathFidelity::BitStable, p_full));
    assert!(!declaration_honoured(DeclaredMathFidelity::BitStable, p_tf32));
    assert!(!declaration_honoured(DeclaredMathFidelity::BitStable, p_bf16));
}
