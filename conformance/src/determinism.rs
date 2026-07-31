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
}
