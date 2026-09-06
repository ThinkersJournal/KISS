//! #414 rename guard — the KISS-Ops compute-fidelity attribute was renamed `MathPrecision` →
//! `MathFidelity` (spec + this crate) to remove the collision with unpopped-vocab's dtype-shaped
//! `MathPrecision`: an implementer following KISS-CONTRACT-6.8-0004's "imported from KISS-Ops"
//! would otherwise `use ..::MathPrecision` and land silently on the decoy.
//!
//! ⚠️ The rename is a TYPE-NAME change ONLY. The WIRE ENCODING — the structure_key `<mp>` code,
//! `st` / `rm` — MUST NOT move: those bytes are in every `sk4|…` token and the KISS-Contract
//! Guarantees `math_precision` field, so re-spelling them would break the wire and churn every
//! golden. This guard pins the codes so a future edit that "tidies" the enum values fails loudly,
//! independently of the Appendix-A golden tokens that also carry them.

use kiss_conformance::structure_key::MathFidelity;

#[test]
fn test_math_fidelity_wire_codes_survive_the_414_rename() {
    // bit-stable -> "st", reduced-mantissa-permitted -> "rm": the two-member attribute's codes.
    assert_eq!(MathFidelity::Stable.code(), "st", "#414: the bit-stable `<mp>` code must stay `st`");
    assert_eq!(
        MathFidelity::ReducedMantissa.code(), "rm",
        "#414: the reduced-mantissa `<mp>` code must stay `rm`"
    );
    // `parse` is the exact inverse — the `<mp>` field round-trips through the renamed type.
    assert_eq!(MathFidelity::parse("st"), Some(MathFidelity::Stable));
    assert_eq!(MathFidelity::parse("rm"), Some(MathFidelity::ReducedMantissa));
    // The code set is EXACTLY {st, rm} — the rename widened nothing, and the NEW type name is not
    // itself a wire code (a codec keyed on the type name rather than the value would parse it).
    for bad in ["mp", "MathFidelity", "MathPrecision", "bit-stable", "reduced-mantissa-permitted", "ST", "RM", "s", ""] {
        assert_eq!(
            MathFidelity::parse(bad), None,
            "#414: `{bad}` must not parse as a MathFidelity `<mp>` code (the set is exactly {{st, rm}})"
        );
    }
}
