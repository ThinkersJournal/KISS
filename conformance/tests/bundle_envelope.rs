//! KISS-Conform witness for the KISS-Contract provider-bundle envelope and its
//! shared provenance vocabulary (Contract §6.9-0009):
//!
//!   * the `revision_base` field is the versioning triple + `generator_commit`,
//!     carried by the **same** shared type at both the per-cell and bundle levels;
//!   * `derivation_lineage` is the closed three-token set;
//!   * a well-formed bundle envelope validates, and its bundle-level provenance
//!     projection does not fork from the per-cell provenance of its contained cells;
//!   * the envelope is OPTIONAL — a single-kernel provider carries none.

use kiss_conformance::bundle_envelope::*;

/// A shared `revision_base` — the triple (`kiss_ops_spec_version`,
/// `recipe_grammar_version`, `kiss_ref_semver`) plus `generator_commit`.
fn a_revision_base() -> RevisionBase {
    RevisionBase {
        generator_commit: "9f3a1c2".into(),
        kiss_ops_spec_version: "6.19.0".into(),
        recipe_grammar_version: "3.2.0".into(),
        kiss_ref_semver: "1.4.0".into(),
    }
}

fn an_envelope() -> BundleEnvelope {
    BundleEnvelope {
        provider_id: "acme-kernels".into(),
        revision_base: a_revision_base(),
        derivation_lineage: DerivationLineage::Generated,
        contained: vec![
            ContainedCell {
                contract_ref: "contracts/add_f32.kisc".into(),
                structure_key_token: "sk2|bin|f32|cuda:sm89|ix32|warp|r1|co/00/v4/d16/f|-".into(),
            },
            ContainedCell {
                contract_ref: "contracts/mul_f32.kisc".into(),
                structure_key_token: "sk2|bin|f32|cuda:sm89|ix32|warp|r1|co/00/v4/d16/f|-".into(),
            },
        ],
    }
}

/// §6.9-0009: `revision_base` is the versioning triple + `generator_commit`, all
/// four present; a missing element is malformed.
#[test]
fn test_revision_base_is_triple_plus_commit() {
    let rb = a_revision_base();
    assert!(rb.is_well_formed());
    // each of the four fields is load-bearing.
    for missing in 0..4 {
        let mut r = a_revision_base();
        match missing {
            0 => r.generator_commit.clear(),
            1 => r.kiss_ops_spec_version.clear(),
            2 => r.recipe_grammar_version.clear(),
            _ => r.kiss_ref_semver.clear(),
        }
        assert!(!r.is_well_formed(), "field {missing} must be required");
    }
}

/// §6.9-0009: `derivation_lineage` is the closed three-token set; an unknown
/// token is rejected and every tag round-trips its pinned ASCII spelling.
#[test]
fn test_derivation_lineage_closed_set() {
    assert_eq!(DerivationLineage::closed_set().len(), 3);
    assert_eq!(DerivationLineage::Generated.token(), "generated");
    assert_eq!(DerivationLineage::Spec613Table.token(), "spec-6.13-table");
    assert_eq!(DerivationLineage::ExternalColdReader.token(), "external-cold-reader");
    for tag in DerivationLineage::closed_set() {
        assert_eq!(DerivationLineage::parse(tag.token()), Some(tag));
    }
    assert_eq!(DerivationLineage::parse("lifted"), None);
    assert_eq!(DerivationLineage::parse("spec-6.14-table"), None);
    assert_eq!(DerivationLineage::parse(""), None);
}

/// §6.9-0009: a well-formed bundle envelope validates.
#[test]
fn test_bundle_envelope_validates() {
    assert_eq!(an_envelope().validate(), Ok(()));
}

/// §6.9-0009: the envelope is a *catalog* feature — a single-kernel provider
/// carries none, and its absence is never a decline.
#[test]
fn test_bundle_envelope_is_optional() {
    let single_kernel_provider: Option<BundleEnvelope> = None;
    assert!(single_kernel_provider.is_none());
    // per-cell provenance still rides the single contract with no envelope needed.
    let per_cell = CellProvenance {
        revision_base: a_revision_base(),
        derivation_lineage: DerivationLineage::Generated,
    };
    assert!(per_cell.revision_base.is_well_formed());
}

/// §6.9-0009: a malformed envelope is a typed decline, never a silent repair.
#[test]
fn test_bundle_envelope_declines_malformed() {
    let mut e = an_envelope();
    e.provider_id.clear();
    assert_eq!(e.validate(), Err(EnvelopeDecline::EmptyProviderId));

    let mut e = an_envelope();
    e.contained.clear();
    assert_eq!(e.validate(), Err(EnvelopeDecline::EmptyContained));

    let mut e = an_envelope();
    e.revision_base.kiss_ref_semver.clear();
    assert_eq!(e.validate(), Err(EnvelopeDecline::MalformedRevisionBase));

    let mut e = an_envelope();
    e.contained[0].structure_key_token.clear();
    assert_eq!(e.validate(), Err(EnvelopeDecline::EmptyStructureKeyToken));
}

/// §6.9-0009: the no-fork constraint. The bundle-level `revision_base` /
/// `derivation_lineage` are the SAME shared types the per-cell provenance carries,
/// so an aligned catalog validates and any per-cell disagreement is a typed fork.
#[test]
fn test_contract_bundle_envelope_shared_provenance() {
    let env = an_envelope();

    // Per-cell provenance riding each contained contract, using the identical
    // shared field shape — no bundle-local re-definition.
    let aligned = vec![
        CellProvenance {
            revision_base: a_revision_base(),
            derivation_lineage: DerivationLineage::Generated,
        },
        CellProvenance {
            revision_base: a_revision_base(),
            derivation_lineage: DerivationLineage::Generated,
        },
    ];
    assert_eq!(env.check_no_fork(&aligned), Ok(()));

    // A cell whose per-cell revision_base forks from the bundle projection.
    let mut forked = aligned.clone();
    forked[1].revision_base.generator_commit = "deadbee".into();
    assert_eq!(env.check_no_fork(&forked), Err(EnvelopeDecline::ProvenanceFork { cell: 1 }));

    // A cell whose per-cell lineage tag forks from the bundle projection.
    let mut forked = aligned.clone();
    forked[0].derivation_lineage = DerivationLineage::ExternalColdReader;
    assert_eq!(env.check_no_fork(&forked), Err(EnvelopeDecline::ProvenanceFork { cell: 0 }));
}

/// The shared types are *literally* the same across levels: a per-cell
/// provenance's fields drop into the bundle envelope unchanged (type-identity of
/// `RevisionBase` / `DerivationLineage`), which is the mechanical no-fork witness.
#[test]
fn test_shared_types_are_type_identical() {
    let per_cell = CellProvenance {
        revision_base: a_revision_base(),
        derivation_lineage: DerivationLineage::Spec613Table,
    };
    // These moves compile only because both levels use the one definition.
    let env = BundleEnvelope {
        provider_id: "p".into(),
        revision_base: per_cell.revision_base.clone(),
        derivation_lineage: per_cell.derivation_lineage,
        contained: vec![ContainedCell {
            contract_ref: "c".into(),
            structure_key_token: "sk2|bin|f32|cuda:sm89|ix32|warp|r1|co/00/v1/da/f|-".into(),
        }],
    };
    assert_eq!(env.revision_base, per_cell.revision_base);
    assert_eq!(env.derivation_lineage, per_cell.derivation_lineage);
    assert_eq!(env.validate(), Ok(()));
}