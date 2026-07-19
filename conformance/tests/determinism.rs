//! Determinism/fidelity class of the KISS-Ops scalar atoms (KISS-Ops §6.0, §6.8).
//!
//! Backs the class-assignment clauses whose oracle is
//! [`kiss_conformance::determinism::atom_determinism_class`]:
//!   * §6.0-0002 — the exact-byte atom set (`test_ops_exact_byte_ops`)
//!   * §6.0-0003 — transcendental atoms are ULP/tolerance (`test_ops_ulp_class_ops`)
//!   * §6.8-0005 — `atan2` is ULP/tolerance, never exact-byte (`test_ops_atan2_class_is_ulp`)

use kiss_conformance::determinism::{
    atom_determinism_class, EXACT_BYTE_ATOMS, TRANSCENDENTAL_ATOMS,
};
use kiss_conformance::DeterminismClass;

/// KISS-OPS-6.8-0005: `atan2` MUST be class ULP/tolerance, never exact-byte.
///
/// Teeth for the #42 self-contradiction: before `atan2` was added to the §6.8
/// transcendental enumeration, §6.0-0002's "if and only if" read it as exact-byte
/// (it is an op-family `binary_math` atom, and §6.0-0002 lists its siblings `copysign`
/// / `nextafter` as exact-byte), while §6.8's table lists it at 4 ULP and `carg` — derived
/// from `atan2` (§6.18-0008) — is class ULP/tolerance (§6.18-0014). An exact-byte comparator
/// would reject every real `atan2`. This asserts the resolved class.
#[test]
fn test_ops_atan2_class_is_ulp() {
    assert_eq!(
        atom_determinism_class("atan2"),
        DeterminismClass::UlpTolerance,
        "atan2 is a declared-ULP transcendental atom (§6.8-0005), so its determinism \
         class is ULP/tolerance"
    );
    assert_ne!(
        atom_determinism_class("atan2"),
        DeterminismClass::ExactByte,
        "atan2 MUST NOT be assigned the exact-byte class — no atan2 is byte-identical \
         across targets"
    );
    // Its exact-byte binary-math siblings are unaffected (they carry no ULP ceiling).
    assert_eq!(atom_determinism_class("copysign"), DeterminismClass::ExactByte);
    assert_eq!(atom_determinism_class("nextafter"), DeterminismClass::ExactByte);
}

/// KISS-OPS-6.0-0002: every atom in the exact-byte set is class exact-byte, and no
/// transcendental atom is (condition (a): "contains no transcendental atom (§6.8)").
#[test]
fn test_ops_exact_byte_ops() {
    for op in EXACT_BYTE_ATOMS {
        assert_eq!(
            atom_determinism_class(op),
            DeterminismClass::ExactByte,
            "§6.0-0002 lists `{op}` in the exact-byte class"
        );
    }
    // No transcendental atom qualifies for exact-byte (the iff, condition (a)).
    for op in TRANSCENDENTAL_ATOMS {
        assert_ne!(
            atom_determinism_class(op),
            DeterminismClass::ExactByte,
            "`{op}` is a §6.8 transcendental atom and MUST NOT be exact-byte"
        );
    }
}

/// KISS-OPS-6.0-0003: every transcendental atom (§6.8) is class ULP/tolerance.
#[test]
fn test_ops_ulp_class_ops() {
    for op in TRANSCENDENTAL_ATOMS {
        assert_eq!(
            atom_determinism_class(op),
            DeterminismClass::UlpTolerance,
            "§6.0-0003: transcendental atom `{op}` is class ULP/tolerance"
        );
    }
}
