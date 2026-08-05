//! Determinism/fidelity class of the KISS-Ops **scalar atoms** (KISS-Ops §6.0).
//!
//! §6.0-0001 pins the canonical enum `{exact-byte, ULP/tolerance,
//! order-invariant/nondeterministic}` (the crate-root [`crate::DeterminismClass`]).
//! This module encodes the *atom-level* class assignment those clauses make:
//!
//!   * §6.0-0002 — the **exact-byte** atoms (arithmetic, comparisons, rounding, bitwise,
//!     `select`, `copysign`, `nextafter`, the algebraic complex ops, `element_map`,
//!     `gather`). Their result is byte-reproducible, so KISS-Conform uses a byte-exact
//!     comparator.
//!   * §6.0-0003 / §6.8 — the **ULP/tolerance** transcendental atoms (`exp`, `log`,
//!     `sin`, `cos`, `sqrt`, `erf`, `atan`, `lgamma`, and — per §6.8-0005 — `atan2`).
//!
//! Ops whose class is *conditional* on a monoid/combine/contraction (`reduce`,
//! `prefix_scan`, `scatter`, `matmul` and every op with a float fold — §6.0-0004/-0005)
//! are NOT modeled here; their class depends on parameters an atom name does not carry.
//! This function is the oracle for the unconditional scalar atoms, which is exactly the
//! set the `atan2` self-contradiction (#42) lived in.

use crate::DeterminismClass;

/// The §6.0-0002 exact-byte atoms whose class is unconditional (no monoid/combine).
pub const EXACT_BYTE_ATOMS: &[&str] = &[
    // arithmetic atoms
    "add", "sub", "mul", "div", "neg", "abs", //
    // ordered comparisons
    "cmp_eq", "cmp_ne", "cmp_lt", "cmp_le", "cmp_gt", "cmp_ge", //
    // rounding atoms
    "floor", "ceil", "trunc", "round_even", //
    // integer bitwise atoms
    "bit_and", "bit_or", "bit_xor", "bit_not", "shl", "shr", "popcount", "clz", "ctz", //
    // raw-bit / sign atoms
    "select", "copysign", "nextafter", //
    // algebraic complex ops (§6.18)
    "cadd", "csub", "cneg", "cconj", "cmul", "cdiv", "cmake", "cre", "cim", //
    // structural access atoms that are exact-byte with a deterministic combine
    "element_map", "gather",
];

/// The §6.8 declared-ULP transcendental atoms. `atan2` is included per §6.8-0005:
/// it is an op-family `binary_math` atom (§6.9) but a declared-ULP transcendental for
/// determinism-class purposes, so it is ULP/tolerance and never exact-byte.
pub const TRANSCENDENTAL_ATOMS: &[&str] =
    &["exp", "log", "sin", "cos", "sqrt", "erf", "atan", "lgamma", "atan2"];

/// The comparison op-family (KISS-OPS-6.0-0008): ops whose OUTPUT reports *which*
/// value(s) won a comparison. These are also §6.0-0002 exact-byte *atoms* — the
/// comparison arithmetic itself is byte-exact — but their output is a **selection**, so
/// over a non-exact producing sub-DAG it escalates by the §6.0-0007 selection rule
/// (order-invariant/nondeterministic, never ULP). Kept as an op-family so the selection
/// test is keyed on op **semantics**, not a hand-maintained allowlist of selection ops.
pub const COMPARISON_ATOMS: &[&str] =
    &["cmp_eq", "cmp_ne", "cmp_lt", "cmp_le", "cmp_gt", "cmp_ge"];

/// The determinism/fidelity class of a scalar atom, per §6.0-0002 (exact-byte) and
/// §6.0-0003 (ULP/tolerance transcendentals). Returns `None` for an op this
/// atom-level model does not cover (a conditional/fold op, or an unknown token) —
/// a typed "not an unconditional scalar atom", never a panic (#64).
pub fn atom_determinism_class(op: &str) -> Option<DeterminismClass> {
    if TRANSCENDENTAL_ATOMS.contains(&op) {
        Some(DeterminismClass::UlpTolerance)
    } else if EXACT_BYTE_ATOMS.contains(&op) {
        Some(DeterminismClass::ExactByte)
    } else {
        None
    }
}

/// Whether an op's OUTPUT is a **selection** (KISS-OPS-6.0-0008) — it reports *which*
/// value(s) won a comparison — determined by op **semantics** (the comparison op-family,
/// [`COMPARISON_ATOMS`]), not by a hand-maintained allowlist of selection ops. This is
/// orthogonal to the op's own atom class: a comparison is an exact-byte *atom*
/// ([`atom_determinism_class`] returns `ExactByte`) whose *output* is nonetheless a
/// selection. A selection output MUST be classified by the §6.0-0007 selection rule
/// (exact-byte iff its producing sub-DAG is entirely exact-byte, else
/// order-invariant/nondeterministic) — NOT by the value-lane most-permissive join of its
/// operands, which would carry ULP/tolerance up from a non-exact operand. So a comparison
/// mask over a non-exact value is order-invariant/nondeterministic, never ULP, regardless
/// of which internal lane an implementation computes the mask in.
pub fn is_selection_producer(op: &str) -> bool {
    COMPARISON_ATOMS.contains(&op)
}

/// The **true** determinism/fidelity class of an op per KISS-OPS §6.0, extending
/// the unconditional-atom oracle with the conditional fold arm the atom model
/// cannot carry (§6.0-0004/-0005). This is the truth an advertised class is
/// checked against (the honesty lint, `check_advertisement`).
///
/// Coverage is the op set the harness differences — the unconditional atoms plus
/// `reduce`/`prefix_scan` over the four monoids — not every §6.0 op. `None` means
/// "this model does not determine the class" (an unknown op, or a fold op with no
/// monoid supplied); callers treat `None` as "cannot honesty-check", never as a class.
pub fn op_true_class(
    op: &str,
    monoid: Option<crate::structural::Monoid>,
) -> Option<DeterminismClass> {
    use crate::structural::Monoid;
    // A float fold's class depends on the monoid (§6.0-0004): Sum/Prod accumulate
    // rounding order-dependently → order-invariant/nondeterministic; Max/Min are
    // selection, order-independent and exact-byte.
    if matches!(op, "reduce" | "prefix_scan") {
        return match monoid {
            Some(Monoid::Sum) | Some(Monoid::Prod) => Some(DeterminismClass::OrderInvariant),
            Some(Monoid::Max) | Some(Monoid::Min) => Some(DeterminismClass::ExactByte),
            None => None,
        };
    }
    // Otherwise it is (or is not) an unconditional scalar atom.
    atom_determinism_class(op)
}

/// The §6.0-0005 permissiveness order: `exact-byte < ULP/tolerance <
/// order-invariant/nondeterministic`. A larger value admits a wider set of results.
pub fn class_permissiveness(c: DeterminismClass) -> u8 {
    match c {
        DeterminismClass::ExactByte => 0,
        DeterminismClass::UlpTolerance => 1,
        DeterminismClass::OrderInvariant => 2,
    }
}

/// The advertisement-honesty lint. Rejects an `advertised` class **strictly more
/// permissive** than the op's `true_class` — that direction selects a comparator
/// too loose to catch a real error (e.g. advertising a Max reduce as
/// order-invariant to buy tolerance a wrong Max could hide behind). An advertisement
/// no more permissive than the truth (equal, or an over-strict over-claim) passes:
/// an over-claim is caught by the differential itself and forbidden by SYNTH
/// §6.5-0004b, so it is not this lint's job (per the design ruling). Returns the
/// advertised class on success so the caller feeds it straight to the comparator.
pub fn check_advertisement(
    advertised: DeterminismClass,
    true_class: DeterminismClass,
) -> Result<DeterminismClass, String> {
    if class_permissiveness(advertised) > class_permissiveness(true_class) {
        return Err(format!(
            "dishonest advertisement: {advertised:?} is more permissive than the true \
             class {true_class:?} (§6.0-0005) — its comparator could not catch a real error"
        ));
    }
    Ok(advertised)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_class_is_some_for_atoms_none_for_folds() {
        assert_eq!(atom_determinism_class("add"), Some(DeterminismClass::ExactByte));
        assert_eq!(atom_determinism_class("exp"), Some(DeterminismClass::UlpTolerance));
        // a fold/conditional op is NOT an unconditional atom → None, never a panic (#64)
        assert_eq!(atom_determinism_class("reduce"), None);
        assert_eq!(atom_determinism_class("not_an_op"), None);
    }

    #[test]
    fn op_true_class_covers_atoms_and_folds() {
        use crate::structural::Monoid;
        // unconditional atom → atom class
        assert_eq!(op_true_class("add", None), Some(DeterminismClass::ExactByte));
        // float Sum/Prod fold → order-invariant (§6.0-0004)
        assert_eq!(op_true_class("reduce", Some(Monoid::Sum)), Some(DeterminismClass::OrderInvariant));
        assert_eq!(op_true_class("reduce", Some(Monoid::Prod)), Some(DeterminismClass::OrderInvariant));
        // Max/Min reduce → exact-byte (order-independent, no float fold error)
        assert_eq!(op_true_class("reduce", Some(Monoid::Max)), Some(DeterminismClass::ExactByte));
        assert_eq!(op_true_class("reduce", Some(Monoid::Min)), Some(DeterminismClass::ExactByte));
        // a fold op with no monoid is underspecified → None (not a guess)
        assert_eq!(op_true_class("reduce", None), None);
    }

    #[test]
    fn honesty_rejects_only_too_permissive() {
        use DeterminismClass::*;
        // ordering
        assert!(class_permissiveness(ExactByte) < class_permissiveness(UlpTolerance));
        assert!(class_permissiveness(UlpTolerance) < class_permissiveness(OrderInvariant));
        // honest (advertised == true) → Ok
        assert_eq!(check_advertisement(OrderInvariant, OrderInvariant), Ok(OrderInvariant));
        // too permissive: advertise order-invariant for an exact-byte (Max) op → Err
        assert!(check_advertisement(OrderInvariant, ExactByte).is_err());
        // over-claim (stricter than true): advertise exact-byte for a Sum fold → Ok here
        // (caught by the differential + SYNTH §6.5-0004b, not this lint)
        assert_eq!(check_advertisement(ExactByte, OrderInvariant), Ok(ExactByte));
    }

    #[test]
    fn is_selection_producer_keys_on_comparison_family() {
        // §6.0-0008: "selection" is keyed on the comparison op-family (semantics), not a
        // hand-maintained list. Check the WHOLE family (not just one member), and guard the
        // two lists against drift: every comparison atom must be a selection producer AND an
        // exact-byte atom — its OWN class stays exact-byte (it is the OUTPUT that is a
        // selection, escalated by §6.0-0007 over non-exact operands). Adding a cmp token to
        // one list but not the other would silently desync selection-detection from classing.
        for &op in COMPARISON_ATOMS {
            assert!(is_selection_producer(op), "{op} must be a selection producer");
            assert_eq!(
                atom_determinism_class(op),
                Some(DeterminismClass::ExactByte),
                "{op} must be an exact-byte atom (COMPARISON_ATOMS ⊆ EXACT_BYTE_ATOMS)"
            );
        }
        // Non-comparison ops are not selections — including a bitwise combinator that could
        // form a COMPOUND predicate mask (bit_and(cmp, cmp)); such a mask inherits the
        // selection class by propagation from its comparison inputs, not by tagging bit_and.
        assert!(!is_selection_producer("add"));
        assert!(!is_selection_producer("exp"));
        assert!(!is_selection_producer("bit_and"));
    }
}
