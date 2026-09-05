//! Plan B / #352 — the per-row NaN-provenance comparator arms (Conform §6.8-0010(a)
//! MOVED / §6.16-0010 COMPUTED), the gap that made both clauses unenforceable in the
//! harness (`compare_f32` carried no provenance). Three born-red / pinning tests; the
//! comparator change does not land with any unproven (architect ruling).
//!
//! bf16: sNaN 0x7F81 (quiet bit 6 CLEAR, mant 0000001), qNaN 0x7FC1 (quiet SET, same
//! payload). f8e4m3fn: single NaN encoding 0x7F / 0xFF (either sign), no sNaN.

use kiss_conformance::nan_provenance::{
    admits_snan, compare_nan_output, is_nan,
    NanProvenance::{Computed, Moved},
};

fn bf16(bits: u16) -> Vec<u8> {
    bits.to_be_bytes().to_vec()
}
fn f8(bits: u8) -> Vec<u8> {
    vec![bits]
}

/// COMPUTED (§6.16-0010): a computed NaN must be QUIET where the format admits sNaN,
/// and payload+sign are NOT compared (no unique correct payload across conformant
/// implementations). Born-red: a promote-and-round impl that fails to set the quiet
/// bit ships an sNaN where a qNaN is expected → reds. Waivers: a different quiet
/// payload, or the opposite sign, MUST pass.
#[test]
fn computed_nan_born_red_and_payload_sign_waived_bf16() {
    // BORN-RED: sNaN actual vs qNaN expected → quietness mismatch.
    assert!(
        compare_nan_output("bf16", &bf16(0x7F81), &bf16(0x7FC1), Computed).is_err(),
        "an impl that drops the quiet bit on a computed NaN MUST red (§6.16-0010)"
    );
    // Payload NOT compared: a different QUIET payload passes.
    assert!(
        compare_nan_output("bf16", &bf16(0x7FC5), &bf16(0x7FC1), Computed).is_ok(),
        "computed NaN payload MUST NOT be compared"
    );
    // Sign NOT compared: opposite-sign quiet NaN passes.
    assert!(
        compare_nan_output("bf16", &bf16(0xFFC1), &bf16(0x7FC1), Computed).is_ok(),
        "computed NaN sign MUST NOT be compared"
    );
    // Conformant: same quietness passes regardless of payload.
    assert!(compare_nan_output("bf16", &bf16(0x7FC0), &bf16(0x7FC1), Computed).is_ok());
}

/// MOVED (§6.8-0010(a)): a moved NaN's bytes ARE the contract — payload AND sign
/// preserved. The EXPECTED cell carries the OUTPUT bytes (the moved sNaN 0x7F81), so
/// this is a byte-exact compare of actual-vs-EXPECTED, NOT "result == input" (a
/// sign-flipping neg/abs would carry the post-edit bits in expected).
#[test]
fn moved_nan_born_red_bf16() {
    // Conformant: the moved sNaN is preserved byte-for-byte.
    assert!(
        compare_nan_output("bf16", &bf16(0x7F81), &bf16(0x7F81), Moved).is_ok(),
        "a preserved moved sNaN MUST pass"
    );
    // BORN-RED: an impl that quiets the moved sNaN (0x7F81 → 0x7FC1) reds.
    assert!(
        compare_nan_output("bf16", &bf16(0x7FC1), &bf16(0x7F81), Moved).is_err(),
        "quieting a moved sNaN MUST red (§6.8-0010(a): payload+sign are the contract)"
    );
    // Moved DOES pin sign (unlike computed): a sign flip on a moved NaN reds.
    assert!(
        compare_nan_output("bf16", &bf16(0xFF81), &bf16(0x7F81), Moved).is_err(),
        "a moved NaN's sign is part of the exact-byte contract"
    );
}

/// f8e4m3fn OVER-ENFORCEMENT PIN: f8e4m3fn has a SINGLE NaN encoding (§6.16-0004),
/// admits no sNaN, so the COMPUTED quietness obligation is VACUOUS — is-NaN is the
/// whole test and any two NaNs (either sign) match. This is the ONLY test that reds
/// if someone "tightens" the COMPUTED arm to compare quietness/sign/bytes for
/// f8e4m3fn — the COMPUTED born-red above is on bf16 (admits sNaN) and stays GREEN
/// through that regression. Deliberate vacuity, PINNED (the "NARROW OVERRIDE — do
/// not simplify" shape, as a test rather than a comment).
#[test]
fn f8e4m3fn_computed_arm_accepts_any_nan_over_enforcement_pin() {
    assert!(!admits_snan("f8e4m3fn"), "f8e4m3fn admits no sNaN (§6.16-0004)");
    // 0x7F and 0xFF are both NaN (S.1111.111, either sign); every pairing must match.
    for (a, e) in [(0x7Fu8, 0x7Fu8), (0xFF, 0x7F), (0x7F, 0xFF), (0xFF, 0xFF)] {
        assert!(is_nan("f8e4m3fn", &f8(a)) && is_nan("f8e4m3fn", &f8(e)), "both NaN");
        assert!(
            compare_nan_output("f8e4m3fn", &f8(a), &f8(e), Computed).is_ok(),
            "f8e4m3fn COMPUTED must accept any NaN (0x{a:02X} vs 0x{e:02X}) — no synthesized distinction"
        );
    }
}
