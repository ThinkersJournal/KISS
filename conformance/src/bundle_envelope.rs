//! Reference witness for the KISS-Contract provider-bundle **envelope** and its
//! shared provenance vocabulary (Contract §6.9-0009).
//!
//! A provider publishing a *catalog* of co-published kernels MAY carry an
//! OPTIONAL bundle-level `bundle_envelope`
//! `{provider_id, revision_base, derivation_lineage, contained}`. The envelope's
//! `revision_base` and `derivation_lineage` are a **bundle-level projection** of
//! the Provenance vocabulary (§6.9-0001/-0003) and MUST NOT fork from the per-cell
//! provenance that rides each contained contract — they are the **same** field
//! shape.
//!
//! This module makes the no-fork constraint mechanical: the [`RevisionBase`] and
//! [`DerivationLineage`] types are the single definitions, and they appear
//! **verbatim** as fields of both [`CellProvenance`] (per-cell) and
//! [`BundleEnvelope`] (bundle-level). There is no second struct for either level,
//! so a fork is unrepresentable.
//!
//! Every field here is determinism-class exact-byte, so the exact-byte comparator
//! (Conform §6.8) applies; the tokens serialize to fixed ASCII spellings.

// ---------------------------------------------------------------------------
// The single shared Provenance-vocabulary field definitions (§6.9-0009).
// ---------------------------------------------------------------------------

/// The shared `revision_base` field (§6.9-0001/-0003): the versioning **triple**
/// `(kiss_ops_spec_version, recipe_grammar_version, kiss_ref_semver)` plus the
/// `generator_commit`. This is the ONE definition; the per-cell provenance
/// ([`CellProvenance`]) and the bundle-level envelope ([`BundleEnvelope`]) both
/// carry this exact type — a bundle-local `revision_base` is unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionBase {
    /// The generator commit the kernel was built from.
    pub generator_commit: String,
    /// KISS-Ops spec version — element 1 of the versioning triple.
    pub kiss_ops_spec_version: String,
    /// Recipe-grammar version — element 2 of the versioning triple.
    pub recipe_grammar_version: String,
    /// kiss-ref SemVer — element 3 of the versioning triple.
    pub kiss_ref_semver: String,
}

impl RevisionBase {
    /// A `revision_base` is well-formed only when every one of the four fields —
    /// `generator_commit` plus the three-element versioning triple — is present
    /// (non-empty). A missing element is a malformed field, not a silent default.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        !self.generator_commit.is_empty()
            && !self.kiss_ops_spec_version.is_empty()
            && !self.recipe_grammar_version.is_empty()
            && !self.kiss_ref_semver.is_empty()
    }
}

/// The shared `derivation_lineage` lineage tag (§6.9-0009), drawn from a
/// **closed** three-token set. The same token domain classifies both the per-cell
/// provenance and the bundle-level envelope — an implementation MUST NOT fork it
/// between the two levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivationLineage {
    /// A generator-produced kernel.
    Generated,
    /// A kernel derived from the KISS-Ops §6.13 reference-decomposition table
    /// (KISS-OPS-6.13-0001).
    Spec613Table,
    /// A kernel produced by an external cold reader of the frozen spec.
    ExternalColdReader,
}

impl DerivationLineage {
    /// The pinned ASCII spelling of the lineage tag.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            DerivationLineage::Generated => "generated",
            DerivationLineage::Spec613Table => "spec-6.13-table",
            DerivationLineage::ExternalColdReader => "external-cold-reader",
        }
    }

    /// Parse a lineage tag, rejecting any token outside the closed set — a reader
    /// MUST NOT silently accept an "unknown" lineage.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "generated" => Some(DerivationLineage::Generated),
            "spec-6.13-table" => Some(DerivationLineage::Spec613Table),
            "external-cold-reader" => Some(DerivationLineage::ExternalColdReader),
            _ => None,
        }
    }

    /// The closed lineage set — exactly the three lineage tags (§6.9-0009).
    #[must_use]
    pub fn closed_set() -> [DerivationLineage; 3] {
        [
            DerivationLineage::Generated,
            DerivationLineage::Spec613Table,
            DerivationLineage::ExternalColdReader,
        ]
    }
}

// ---------------------------------------------------------------------------
// Per-cell provenance (rides each contained contract, §6.9-0001).
// ---------------------------------------------------------------------------

/// The per-cell Provenance projection that rides one contained contract. It reuses
/// the shared [`RevisionBase`] and [`DerivationLineage`] types verbatim — this is
/// the anchor the bundle-level envelope MUST NOT fork from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellProvenance {
    pub revision_base: RevisionBase,
    pub derivation_lineage: DerivationLineage,
}

// ---------------------------------------------------------------------------
// The bundle envelope (bundle-level projection, §6.9-0009).
// ---------------------------------------------------------------------------

/// One `contained` entry: the pair `{contract_ref, structure_key_token}` naming a
/// co-published cell. The `structure_key_token` is carried **opaquely** as the
/// KISS-Classify token and is never reinterpreted here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainedCell {
    pub contract_ref: String,
    /// Opaque KISS-Classify `structure_key` token; carried, not parsed.
    pub structure_key_token: String,
}

/// The OPTIONAL provider-bundle envelope (§6.9-0009). Its `revision_base` and
/// `derivation_lineage` are the **same** shared types the per-cell provenance
/// carries, so the bundle-level projection cannot fork from the per-cell one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEnvelope {
    pub provider_id: String,
    pub revision_base: RevisionBase,
    pub derivation_lineage: DerivationLineage,
    pub contained: Vec<ContainedCell>,
}

/// A typed decline from envelope validation (§6.9-0009): a malformed envelope is
/// rejected, never silently repaired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeDecline {
    EmptyProviderId,
    MalformedRevisionBase,
    EmptyContained,
    EmptyContractRef,
    EmptyStructureKeyToken,
    /// A contained cell's per-cell provenance forks from the bundle-level
    /// projection (different `revision_base` or `derivation_lineage`).
    ProvenanceFork { cell: usize },
}

impl BundleEnvelope {
    /// Validate a bundle envelope. A well-formed envelope has a non-empty
    /// `provider_id`, a well-formed shared `revision_base`, and at least one
    /// `contained` cell each naming a `contract_ref` and an opaque
    /// `structure_key_token`.
    pub fn validate(&self) -> Result<(), EnvelopeDecline> {
        if self.provider_id.is_empty() {
            return Err(EnvelopeDecline::EmptyProviderId);
        }
        if !self.revision_base.is_well_formed() {
            return Err(EnvelopeDecline::MalformedRevisionBase);
        }
        if self.contained.is_empty() {
            return Err(EnvelopeDecline::EmptyContained);
        }
        for c in &self.contained {
            if c.contract_ref.is_empty() {
                return Err(EnvelopeDecline::EmptyContractRef);
            }
            if c.structure_key_token.is_empty() {
                return Err(EnvelopeDecline::EmptyStructureKeyToken);
            }
        }
        Ok(())
    }

    /// The no-fork witness (§6.9-0009): given the per-cell provenance of each
    /// contained cell, the envelope's bundle-level `revision_base` /
    /// `derivation_lineage` MUST equal every cell's per-cell projection. Any cell
    /// whose provenance disagrees is a fork and a typed decline.
    ///
    /// Because both levels carry the identical [`RevisionBase`] /
    /// [`DerivationLineage`] types, the comparison is a plain `==` — there is no
    /// second field shape to reconcile.
    pub fn check_no_fork(&self, per_cell: &[CellProvenance]) -> Result<(), EnvelopeDecline> {
        for (i, cell) in per_cell.iter().enumerate() {
            if cell.revision_base != self.revision_base
                || cell.derivation_lineage != self.derivation_lineage
            {
                return Err(EnvelopeDecline::ProvenanceFork { cell: i });
            }
        }
        Ok(())
    }
}