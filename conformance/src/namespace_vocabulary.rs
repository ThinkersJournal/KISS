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
    /// An entry carries no derivability witness, or an empty one (§6.8-0014).
    /// Present-but-empty is the likelier defect: a generator emitting `[]` for an entry it
    /// could not trace looks compliant to a presence check.
    EntryMissingWitness { token: String },
    /// A witness reference does not name its home, so it cannot be resolved (§6.8-0014).
    WitnessNotResolvable { token: String, reference: String },
    /// The manifest names no gate that evaluates its witnesses (§6.8-0014). A witness
    /// nobody evaluates is a claim in the shape of a proof.
    NoWitnessGate,
    /// `kind: generated` vector set does not cover this canonicalization concern (§6.8-0013).
    GeneratedVectorsMissingPin(&'static str),
    /// `namespace` is not a namespace whose registry status is `registered` (§6.8-0003, cited
    /// by §6.8-0008). An envelope naming an unregistered or reserved namespace is declined:
    /// otherwise validation passes while violating the clause it implements.
    UnregisteredNamespace { got: String },
    /// `vocabulary_version` is a JSON number with a fractional part (§6.8-0009). A gate that
    /// truncates `3.9` to `3` is not a gate — it silently admits a version nobody declared.
    NonIntegerVersion { got: String },
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
/// The bundled §6.8-0003 registry. `validate_envelope` reads only the `namespace` and `status`
/// columns — never a maintainer's vocabulary — so the envelope check stays content-blind.
const REGISTRY: &str = include_str!("../registry/namespaces.json");

/// True iff `ns` appears in the bundled registry with `status == "registered"`. A `reserved`
/// row is NOT registered: §6.8-0003 reserves the name and forbids producing under it.
fn is_registered(ns: &str) -> bool {
    let Ok(reg) = json::parse(REGISTRY) else { return false };
    let Some(rows) = reg.get("namespaces").and_then(|j| j.as_arr()) else { return false };
    rows.iter().any(|row| {
        row.get("namespace").and_then(|j| j.as_str()) == Some(ns)
            && row.get("status").and_then(|j| j.as_str()) == Some("registered")
    })
}

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
    // §6.8-0008 requires a namespace whose registry status is `registered` (§6.8-0003).
    // Shape alone is not enough: an envelope that validates while naming an unregistered
    // namespace passes the check and violates the clause the check exists to enforce.
    let namespace = str_field(&doc, "namespace")?.to_string();
    if !is_registered(&namespace) {
        return Err(ManifestDecline::UnregisteredNamespace { got: namespace });
    }
    // §6.8-0009: the version is a GATE, so it MUST be an integer. `as_u64` alone truncates —
    // a manifest declaring 3.9 would be admitted as 3, a version nobody published.
    let vocabulary_version = match doc.get("vocabulary_version") {
        Some(Json::Num(n)) if n.fract() == 0.0 && *n >= 0.0 => *n as u64,
        Some(Json::Num(n)) => {
            return Err(ManifestDecline::NonIntegerVersion { got: n.to_string() })
        }
        _ => return Err(ManifestDecline::MissingField("vocabulary_version")),
    };
    let generated_from = str_field(&doc, "generated_from")?.to_string();
    if generated_from.is_empty() {
        return Err(ManifestDecline::EmptyProvenance); // §6.8-0011
    }
    // `coverage_note` is required (§6.8-0008 / RFC §4.4).
    // `grammar` is required for BOTH kinds: §6.8-0012 places it in the declarative half, which
    // MUST be sufficient for a parse-only consumer — and a parser without a grammar has nothing.
    str_field(&doc, "grammar")?;
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
            // Presence is not content: `"field_spec": null` satisfies `is_none()` and leaves a
            // parse-only consumer with nothing (§6.8-0012).
            if !matches!(doc.get("field_spec"), Some(Json::Obj(_))) {
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


/// KISS-CLASSIFY-6.8-0014 — the derivability-witness ENVELOPE.
///
/// A separate checker rather than part of `parse`, for two reasons that happen to agree.
/// It matches `check_generated_vector_coverage` (the production half is not owed by a
/// parse-only consumer, §6.8-0012); and §6.8-0014 applies **from its schema version**, so
/// an existing `kiss-namespace-vocabulary-v1` manifest must keep parsing unchanged and
/// complies at its next `vocabulary_version` revision. Folding this into `parse` would
/// re-issue both published manifests on the spot, which the clause forbids.
///
/// KISS checks the ENVELOPE ONLY: a witness is present, is a non-empty LIST, its entries
/// are resolvable references, and a gate is named. Whether the witness actually produces
/// the entry — and whether the named gate exists and runs — is the maintainer's to
/// discharge. KISS has no device and no toolchain, and the vocabulary content is out of
/// reach by §6.8-0004.
pub fn check_derivability_witnesses(m: &Manifest) -> Result<(), ManifestDecline> {
    // The gate is NAMED here, never contained: requiring co-location would assume one
    // namespace's architecture, which §6.8-0008's own note forbids.
    let gate = m.raw.get("witness_gate").and_then(|j| j.as_str());
    if gate.map_or(true, |g| g.trim().is_empty()) {
        return Err(ManifestDecline::NoWitnessGate);
    }
    let entries = match m.raw.get("members").and_then(|j| j.as_arr()) {
        Some(e) => e,
        // NO `members` -> `kind: generated`, whose entries are an open product space rather
        // than a list. THIS CHECK SKIPS THEM ENTIRELY, and says so rather than claiming a
        // coverage it does not have: an earlier comment here read "witnessed per vector",
        // which describes a validation this function does not perform -- nothing inspects a
        // witness field on `vectors`. That is the same defect as the resolvability comment
        // below, and the same defect §6.8-0014 is about: a justification claiming more than
        // its mechanism. Whether a generated vocabulary witnesses per vector, per field, or
        // some other way is a real question and it is NOT settled here (#340 review).
        None => return Ok(()),
    };
    for entry in entries {
        // The entry is identified as the MANIFEST identifies it, not by token alone: a
        // class-qualified vocabulary distinguishes `cuda:sm90` (admits under `<=`) from
        // `cuda:sm90a` (`==`), and a witness bound to a flat token cannot say which it proves.
        let token = entry.get("token").and_then(|j| j.as_str()).unwrap_or("<unnamed>");
        let witness = entry.get("derivability_witness").and_then(|j| j.as_arr());
        let refs = match witness {
            Some(r) if !r.is_empty() => r,
            // Present-but-empty is the same defect as absent, and is the likelier one: a
            // generator that emits `[]` for an entry it could not trace looks compliant.
            _ => return Err(ManifestDecline::EntryMissingWitness { token: token.to_string() }),
        };
        for r in refs {
            // ENVELOPE ONLY: the reference must be non-empty and PATH-STRUCTURED. It does
            // NOT verify that the leading segment names an artifact rather than a type --
            // `ArchSku::Sm90` passes here and `unpopped-vocab::ArchSku::Sm90` passes here,
            // and this check cannot tell them apart. An earlier comment claimed it could,
            // naming the first as the counterexample: FALSE, it contains `::` and passes.
            // That was a justification overstating its mechanism, which is the defect this
            // clause is about, committed inside the clause's own enforcement.
            //
            // Distinguishing a crate from a type is not decidable from the envelope --
            // `MyCrate::Thing` and `ArchSku::Sm90` are the same shape -- so whether the
            // HOME is genuinely nameable is the maintainer's half, like the rest of the
            // content obligations. §6.8-0014 states the obligation; this checks the part
            // KISS can see.
            let s = r.as_str().unwrap_or("");
            if s.trim().is_empty() || !s.contains("::") {
                return Err(ManifestDecline::WitnessNotResolvable {
                    token: token.to_string(),
                    reference: s.to_string(),
                });
            }
        }
    }
    Ok(())
}
