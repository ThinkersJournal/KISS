// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Capabilities `precision_class` is a label derived from the Guarantees
//! (KISS-CONTRACT-6.7-0005, over KISS-OPS-6.8-0007/-0008/-0009).

use kiss_conformance::accuracy::PrecisionClass;
use kiss_conformance::contract::{
    appendix_c_capabilities_block, appendix_c_guarantees_inputs, verify_precision_class,
    BitStability, DeclaredAccuracyTier, Guarantees, PrecisionClassDecline,
};

fn guarantees(tier: DeclaredAccuracyTier, bs: BitStability) -> Guarantees {
    Guarantees {
        reference_function: Some("add".into()),
        per_backend_ulp_tiers: vec![("cpu".into(), tier)],
        determinism_class: kiss_conformance::DeterminismClass::ExactByte,
        bit_stability: bs,
    }
}

/// Enforces: KISS-CONTRACT-6.7-0005 — the Appendix C `strict` is the derived label of its own
/// Guarantees, a contradicting label or one outside the closed set is refused, and the spec text
/// points at the Ops clauses that define the set.
#[test]
fn test_contract_precision_class_consistent() {
    // The shipped example: tier 0 + portable => strict, and the golden carries that spelling.
    let g = appendix_c_guarantees_inputs();
    assert_eq!(g.derived_precision_class(), PrecisionClass::Strict);
    assert_eq!(verify_precision_class(&g, "strict"), Ok(()));
    let caps = String::from_utf8(appendix_c_capabilities_block()).unwrap();
    assert!(caps.contains("precision_class = strict\n"), "{caps}");

    // Tier 0 without portable is correctly-rounded, and `strict` there contradicts it.
    let t0 = DeclaredAccuracyTier { max_ulp: Some(0), ..DeclaredAccuracyTier::default() };
    let g = guarantees(t0, BitStability::SameHardware);
    assert_eq!(verify_precision_class(&g, "correctly-rounded"), Ok(()));
    assert_eq!(
        verify_precision_class(&g, "strict"),
        Err(PrecisionClassDecline::ContradictsGuarantees {
            declared: PrecisionClass::Strict,
            derived: PrecisionClass::CorrectlyRounded
        })
    );
    // A looser tier cannot carry a tighter class.
    let t4 = DeclaredAccuracyTier { max_ulp: Some(4), ..DeclaredAccuracyTier::default() };
    let g = guarantees(t4, BitStability::Portable);
    assert_eq!(verify_precision_class(&g, "bounded-ulp"), Ok(()));
    assert!(matches!(verify_precision_class(&g, "strict"), Err(PrecisionClassDecline::ContradictsGuarantees { .. })));
    assert!(matches!(verify_precision_class(&g, "correctly-rounded"), Err(PrecisionClassDecline::ContradictsGuarantees { .. })));
    // No tier at all => unbounded.
    let g = Guarantees { per_backend_ulp_tiers: vec![], ..guarantees(t4, BitStability::Portable) };
    assert_eq!(verify_precision_class(&g, "unbounded"), Ok(()));
    // Outside the closed set, including the withdrawn spelling.
    assert_eq!(
        verify_precision_class(&g, "bit-reproducible"),
        Err(PrecisionClassDecline::NotInClosedSet("bit-reproducible".into()))
    );

    // Cross-document pointer: §6.7-0005 cites the three Ops clauses, and Ops defines them.
    let root = format!("{}/..", env!("CARGO_MANIFEST_DIR"));
    let contract = std::fs::read_to_string(format!("{root}/spec/contract.md")).unwrap();
    let ops = std::fs::read_to_string(format!("{root}/spec/ops.md")).unwrap();
    let i = contract.find("**KISS-CONTRACT-6.7-0005**").expect("6.7-0005 defined");
    let block = &contract[i..i + contract[i..].find("\n- **KISS-CONTRACT-6.7-0006**").unwrap()];
    for id in ["KISS-OPS-6.8-0007", "KISS-OPS-6.8-0008", "KISS-OPS-6.8-0009"] {
        assert!(block.contains(id), "6.7-0005 must cite {id}");
        assert!(ops.contains(&format!("**{id}**")), "{id} must be defined in ops.md");
    }
}
