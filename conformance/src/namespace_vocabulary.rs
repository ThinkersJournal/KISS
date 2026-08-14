//! Envelope reader for the machine-readable namespace capability-vocabulary manifest
//! (KISS-CLASSIFY §6.8-0008 … -0013, RFC #171).
//!
//! KISS pins the **envelope**; the vocabulary **content** is the namespace maintainer's
//! (§6.8-0004) and is NOT validated here. This reader checks only that a manifest is a
//! well-formed envelope of a **recognized** `kind`, exposes the `vocabulary_version` **gate**
//! (§6.8-0009), and — for a producer (§6.8-0012) — that a `generated` manifest's vector set
//! **covers** the four canonicalization concerns (§6.8-0013). It never inspects `cuda`'s
//! member list or `vulkan`'s field values; those live in the maintainers' annexes.
//!
//! The declarative/production split (§6.8-0012) is structural here: [`validate_envelope`] is
//! the declarative half a parse-only consumer runs (it does not require `vectors`), and
//! [`check_generated_vector_coverage`] is the production half only a producer runs.

use crate::json::{self, Json};
use std::collections::HashSet;

/// The schema id every manifest carries (§6.8-0008).
pub const MANIFEST_SCHEMA: &str = "kiss-namespace-vocabulary-v1";

/// A typed decline from the envelope reader (§6.8-0009 model: recognized-vs-unknown). Never a
/// panic — an unrecognized `kind` or a version skew is a decline a consumer can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestDecline {
    BadJson,
    NotAnObject,
    /// `schema` is not [`MANIFEST_SCHEMA`] (§6.8-0008).
    UnknownSchema { got: String },
    /// A required envelope field is absent or the wrong type (§6.8-0008).
    MissingField(&'static str),
    /// `generated_from` is present but empty — no annex to gate against (§6.8-0011).
    EmptyProvenance,
    /// `kind` is a value this build does not recognize — declined, NOT guessed (§6.8-0010).
    UnrecognizedKind { got: String },
    /// `kind: enumerated` without a `members` array (§6.8-0008).
    EnumeratedMissingMembers,
    /// `kind: generated` producer check with no `vectors` array (§6.8-0013).
    GeneratedMissingVectors,
    /// `kind: generated` vector set does not cover this canonicalization concern (§6.8-0013).
    GeneratedVectorsMissingPin(&'static str),
    /// The consumer gate: `vocabulary_version` is not the one this consumer was built for
    /// (§6.8-0009). Reading the field without reaching this is not a gate.
    VocabularyVersionMismatch { got: u64, built_for: u64 },
}

/// The manifest kinds recognized at this schema version. `kind` is an OPEN set (§6.8-0010): an
/// unrecognized value is [`ManifestDecline::UnrecognizedKind`], and a future kind is admitted by
/// adding a variant here — not by guessing the nearer of the two known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Enumerated,
    Generated,
}

/// A validated manifest ENVELOPE. Carries the gated/dispatch fields plus the raw document for
/// the production-half check; it does not decode the maintainer's vocabulary content.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub namespace: String,
    pub vocabulary_version: u64,
    pub generated_from: String,
    pub kind: Kind,
    raw: Json,
}

/// The four canonicalization concerns a `generated` vector set covers (§6.8-0013). `order` and
/// `dedup` always apply; `threshold`/`digest_input` may be exempted by a namespace with no
/// length-conditional field (declared via `pins_exempt`, and stated in its `coverage_note`).
const GENERATED_PINS: [&str; 4] = ["order", "dedup", "threshold", "digest_input"];
const NON_EXEMPTIBLE_PINS: [&str; 2] = ["order", "dedup"];

/// Envelope-validate a manifest (§6.8-0008, -0010, -0011) — the DECLARATIVE half, sufficient for
/// a consumer that only parses foreign tokens (§6.8-0012). Recognition only: no vocabulary
/// content is inspected, and `vectors` are NOT required (they are the production half). An
/// unrecognized `kind` is a typed decline (§6.8-0010), never a guess.
pub fn validate_envelope(text: &str) -> Result<Manifest, ManifestDecline> {
    let doc = json::parse(text).map_err(|_| ManifestDecline::BadJson)?;
    if !matches!(doc, Json::Obj(_)) {
        return Err(ManifestDecline::NotAnObject);
    }

    // §6.8-0008: the schema id and the required envelope fields.
    let schema = str_field(&doc, "schema")?;
    if schema != MANIFEST_SCHEMA {
        return Err(ManifestDecline::UnknownSchema { got: schema.to_string() });
    }
    let namespace = str_field(&doc, "namespace")?.to_string();
    let vocabulary_version = doc
        .get("vocabulary_version")
        .and_then(|j| j.as_u64())
        .ok_or(ManifestDecline::MissingField("vocabulary_version"))?;
    let generated_from = str_field(&doc, "generated_from")?.to_string();
    if generated_from.is_empty() {
        return Err(ManifestDecline::EmptyProvenance); // §6.8-0011
    }
    // `coverage_note` is required (§6.8-0008 / RFC §4.4).
    str_field(&doc, "coverage_note")?;

    // §6.8-0010: kind is an OPEN set — an unrecognized value declines, never guesses.
    let kind = match str_field(&doc, "kind")? {
        "enumerated" => Kind::Enumerated,
        "generated" => Kind::Generated,
        other => return Err(ManifestDecline::UnrecognizedKind { got: other.to_string() }),
    };

    // §6.8-0008 per-kind DECLARATIVE shape. Note `generated` requires `field_spec` (declarative,
    // for a parser) but NOT `vectors` (production, §6.8-0012 — a parse-only consumer never needs
    // them; a producer checks them via `check_generated_vector_coverage`).
    match kind {
        Kind::Enumerated => {
            if doc.get("members").and_then(|j| j.as_arr()).is_none() {
                return Err(ManifestDecline::EnumeratedMissingMembers);
            }
        }
        Kind::Generated => {
            if doc.get("field_spec").is_none() {
                return Err(ManifestDecline::MissingField("field_spec"));
            }
        }
    }

    Ok(Manifest { namespace, vocabulary_version, generated_from, kind, raw: doc })
}

/// §6.8-0009: the consumer GATE. A consumer that merely reads `vocabulary_version` has not gated
/// on it — this is the assert, and it fails loud on a version the consumer was not built for.
pub fn assert_vocabulary_version(m: &Manifest, built_for: u64) -> Result<(), ManifestDecline> {
    if m.vocabulary_version != built_for {
        return Err(ManifestDecline::VocabularyVersionMismatch {
            got: m.vocabulary_version,
            built_for,
        });
    }
    Ok(())
}

/// §6.8-0013: the PRODUCTION half — only a producer runs this. A `generated` manifest's vector
/// set MUST cover the canonicalization concerns: `order` and `dedup` always; `threshold` and
/// `digest_input` unless exempted via `pins_exempt` (a namespace with no length-conditional
/// field). Returns the first missing pin. A parse-only consumer never calls this (§6.8-0012).
pub fn check_generated_vector_coverage(m: &Manifest) -> Result<(), ManifestDecline> {
    if m.kind != Kind::Generated {
        return Ok(()); // vacuously covered: an enumerated vocabulary has no canonicalization
    }
    let vectors = m
        .raw
        .get("vectors")
        .and_then(|j| j.as_arr())
        .ok_or(ManifestDecline::GeneratedMissingVectors)?;
    let present: HashSet<&str> = vectors
        .iter()
        .filter_map(|v| v.get("pins").and_then(|j| j.as_str()))
        .collect();
    let exempt: HashSet<&str> = m
        .raw
        .get("pins_exempt")
        .and_then(|j| j.as_arr())
        .map(|a| a.iter().filter_map(|j| j.as_str()).collect())
        .unwrap_or_default();
    for pin in GENERATED_PINS {
        let exemptible = !NON_EXEMPTIBLE_PINS.contains(&pin);
        if !present.contains(pin) && !(exemptible && exempt.contains(pin)) {
            return Err(ManifestDecline::GeneratedVectorsMissingPin(pin));
        }
    }
    Ok(())
}

fn str_field<'a>(doc: &'a Json, key: &'static str) -> Result<&'a str, ManifestDecline> {
    doc.get(key)
        .and_then(|j| j.as_str())
        .ok_or(ManifestDecline::MissingField(key))
}
