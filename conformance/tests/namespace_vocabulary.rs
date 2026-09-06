//! KISS-Conform tests for the namespace capability-vocabulary manifest ENVELOPE
//! (KISS-CLASSIFY §6.8-0008 … -0013, RFC #171).
//!
//! These validate the ENVELOPE only, against synthetic `example` manifests — never `cuda`'s or
//! `vulkan`'s content, which are the maintainers' (§6.8-0004). Each test pairs an accept with a
//! flip that must decline, so no guard passes vacuously.

use kiss_conformance::namespace_vocabulary::*;

// ---- synthetic manifest builders (envelope demonstration, not real namespaces) -------------

/// Render a manifest from (key, raw-json-value) pairs, in order.
fn build_from(fields: &[(&str, &str)]) -> String {
    let body = fields
        .iter()
        .map(|(k, v)| format!("  \"{k}\": {v}"))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n{body}\n}}")
}

fn drop_key<'a>(fields: Vec<(&'a str, &'a str)>, key: &str) -> Vec<(&'a str, &'a str)> {
    fields.into_iter().filter(|(k, _)| *k != key).collect()
}
fn set_key<'a>(fields: Vec<(&'a str, &'a str)>, key: &str, val: &'a str) -> Vec<(&'a str, &'a str)> {
    fields
        .into_iter()
        .map(|(k, v)| if k == key { (k, val) } else { (k, v) })
        .collect()
}

fn enum_fields() -> Vec<(&'static str, &'static str)> {
    vec![
        ("schema", "\"kiss-namespace-vocabulary-v1\""),
        ("namespace", "\"cuda\""),
        ("vocabulary_version", "3"),
        ("generated_from", "\"spec/namespaces/cuda.md\""),
        ("kind", "\"enumerated\""),
        ("grammar", "\"cuda:sm<N>[<letter>]\""),
        ("coverage_note", "\"closed list; recognition is the whole contract.\""),
        ("members", "[{\"token\": \"cuda:sm80\", \"notes\": \"synthetic row; content is the maintainer's\"}]"),
    ]
}

fn gen_fields() -> Vec<(&'static str, &'static str)> {
    vec![
        ("schema", "\"kiss-namespace-vocabulary-v1\""),
        ("namespace", "\"vulkan\""),
        ("vocabulary_version", "3"),
        ("generated_from", "\"spec/namespaces/vulkan.md\""),
        ("kind", "\"generated\""),
        ("grammar", "\"vulkan:<sg>.<ops>.<arith>.<coop>\""),
        ("coverage_note", "\"enumeration is impossible; the vectors are the contract.\""),
        ("field_spec", "{\"fields\": 1, \"separator\": \",\"}"),
        (
            "vectors",
            "[{\"pins\": \"order\", \"input\": \"b,a\", \"output\": \"a,b\"}, \
              {\"pins\": \"dedup\", \"input\": \"a,a\", \"output\": \"a\"}, \
              {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"}, \
              {\"pins\": \"digest_input\", \"input\": \"a,b,c\", \"output\": \"a,b,c\"}]",
        ),
    ]
}

/// Enforces KISS-CLASSIFY-6.8-0013 — the `digest_input` CONTENT obligation: "the SAME
/// byte string measured against the threshold, so a producer may disagree about WHETHER
/// to digest but never about WHAT is digested".
///
/// ⚠️ THIS WAS UNIMPLEMENTED, NOT MERELY UNTESTED (#415/#432). `check_generated_vector_
/// coverage` compared `pins` TAG STRINGS against a required set and never opened a vector,
/// so a vector tagged `digest_input` whose two byte strings DIFFERED satisfied the clause.
/// A better test could not have closed that — there was nothing to discriminate.
///
/// ⚠️ AND THE COMPARISON IS ON BYTES, NEVER ON DIGESTS. Two different byte strings whose
/// digests collide would pass a digest comparison, and the clause deliberately says "the
/// same byte string" rather than "the same digest".
#[test]
fn test_namespace_vocabulary_digest_input_is_the_same_byte_string() {
    // the well-formed manifest declares the same string on both sides -> covered.
    let ok = validate_envelope(&build_from(&gen_fields())).unwrap();
    assert_eq!(check_generated_vector_coverage(&ok), Ok(()));

    // a `digest_input` vector whose measured and declared strings DIFFER -> typed decline.
    let differing = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"order\", \"input\": \"b,a\", \"output\": \"a,b\"},           {\"pins\": \"dedup\", \"input\": \"a,a\", \"output\": \"a\"},           {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"},           {\"pins\": \"digest_input\", \"input\": \"a,b,c\", \"output\": \"a,b,d\"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&differing)).unwrap()),
        Err(ManifestDecline::DigestInputNotIdentical),
        "a digest_input vector whose two byte strings differ MUST decline"
    );

    // ⚠️ AND A ONE-BYTE DIFFERENCE MUST DECLINE TOO -- the check is byte identity, not
    // similarity, and a near-miss is exactly what a digest comparison would let through.
    let near_miss = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"order\", \"input\": \"b,a\", \"output\": \"a,b\"},           {\"pins\": \"dedup\", \"input\": \"a,a\", \"output\": \"a\"},           {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"},           {\"pins\": \"digest_input\", \"input\": \"a,b,c\", \"output\": \"a,b,c \"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&near_miss)).unwrap()),
        Err(ManifestDecline::DigestInputNotIdentical),
        "a trailing-space difference is still a different byte string"
    );
}

/// Enforces KISS-CLASSIFY-6.8-0013 — a MALFORMED digest_input vector and a MISMATCHED one
/// are different conditions and must not share a decline code.
///
/// ⚠️ A review flagged the two `DigestInputNotIdentical` arms as redundant and prescribed
/// collapsing them into a wildcard. The redundancy was real; the remedy would have made
/// the conflation PERMANENT. A vector missing `input` was being told "your digest_input is
/// not identical" -- false, and it points the author at a comparison rather than at the
/// absent field. Same axis as #420's `UnsupportedDtype` collapsing spec-illegal with
/// not-yet-implemented, one level down.
#[test]
fn test_namespace_vocabulary_digest_input_malformed_is_not_mismatched() {
    // a `digest_input` vector with NO `input` field: MALFORMED, not mismatched.
    let no_input = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"order\", \"input\": \"b,a\", \"output\": \"a,b\"},           {\"pins\": \"dedup\", \"input\": \"a,a\", \"output\": \"a\"},           {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"},           {\"pins\": \"digest_input\", \"output\": \"a,b,c\"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&no_input)).unwrap()),
        Err(ManifestDecline::MissingField("input")),
        "a digest_input vector with no `input` is MALFORMED -- it must not be reported as a          byte mismatch, which would send the author to a comparison instead of the field"
    );

    // ... and with no `output`, naming the OTHER field rather than a generic decline.
    let no_output = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"order\", \"input\": \"b,a\", \"output\": \"a,b\"},           {\"pins\": \"dedup\", \"input\": \"a,a\", \"output\": \"a\"},           {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"},           {\"pins\": \"digest_input\", \"input\": \"a,b,c\"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&no_output)).unwrap()),
        Err(ManifestDecline::MissingField("output")),
        "the absent field must be NAMED, not folded into a shared code"
    );
}

// ---- §6.8-0008: envelope shape --------------------------------------------------------------

#[test]
fn test_namespace_vocabulary_envelope_shape() {
    // both kinds, well-formed, are accepted as valid envelopes.
    assert_eq!(validate_envelope(&build_from(&enum_fields())).unwrap().kind, Kind::Enumerated);
    assert_eq!(validate_envelope(&build_from(&gen_fields())).unwrap().kind, Kind::Generated);

    // an unrecognized schema declines (naming it), so a v2 manifest cannot be read as v1.
    let bad_schema = set_key(enum_fields(), "schema", "\"kiss-namespace-vocabulary-v2\"");
    assert!(matches!(
        validate_envelope(&build_from(&bad_schema)),
        Err(ManifestDecline::UnknownSchema { .. })
    ));

    // a missing required envelope field declines, naming the field.
    assert_eq!(
        validate_envelope(&build_from(&drop_key(enum_fields(), "coverage_note"))),
        Err(ManifestDecline::MissingField("coverage_note"))
    );

    // per-kind shape: an enumerated manifest without `members` declines.
    assert_eq!(
        validate_envelope(&build_from(&drop_key(enum_fields(), "members"))),
        Err(ManifestDecline::EnumeratedMissingMembers)
    );
    // a generated manifest without the declarative `field_spec` declines.
    assert_eq!(
        validate_envelope(&build_from(&drop_key(gen_fields(), "field_spec"))),
        Err(ManifestDecline::MissingField("field_spec"))
    );
}

// ---- §6.8-0008: the envelope constraints that SHAPE alone does not enforce -------------------

// Backs: KISS-CLASSIFY-6.8-0003, KISS-CLASSIFY-6.8-0008, KISS-CLASSIFY-6.8-0009, KISS-CLASSIFY-6.8-0012
#[test]
fn test_namespace_vocabulary_envelope_constraints() {
    // (a) §6.8-0008 requires a namespace whose registry status is `registered` (§6.8-0003).
    // A synthetic name is NOT registered, so an envelope naming one declines — otherwise
    // validation passes while violating the clause it implements.
    let unreg = set_key(enum_fields(), "namespace", "\"example\"");
    assert!(matches!(
        validate_envelope(&build_from(&unreg)),
        Err(ManifestDecline::UnregisteredNamespace { .. })
    ));

    // `reserved` is not `registered`: 6.8-0003 reserves the name and forbids producing under it.
    let reserved = set_key(enum_fields(), "namespace", "\"rocm\"");
    assert!(matches!(
        validate_envelope(&build_from(&reserved)),
        Err(ManifestDecline::UnregisteredNamespace { .. })
    ));

    // (b) §6.8-0009 is a GATE, so the version must be an integer. Truncating 3.9 -> 3 would
    // admit a version nobody published.
    let frac = set_key(enum_fields(), "vocabulary_version", "3.9");
    assert!(matches!(
        validate_envelope(&build_from(&frac)),
        Err(ManifestDecline::NonIntegerVersion { .. })
    ));

    // (c) §6.8-0012 puts `grammar` in the declarative half, which MUST serve a parse-only
    // consumer — and a parser without a grammar has nothing.
    let no_grammar: Vec<_> =
        enum_fields().into_iter().filter(|(k, _)| *k != "grammar").collect();
    assert!(matches!(
        validate_envelope(&build_from(&no_grammar)),
        Err(ManifestDecline::MissingField("grammar"))
    ));

    // (d) PRESENCE IS NOT CONTENT: `"field_spec": null` is present and useless.
    let null_spec = set_key(gen_fields(), "field_spec", "null");
    assert!(matches!(
        validate_envelope(&build_from(&null_spec)),
        Err(ManifestDecline::MissingField("field_spec"))
    ));
}

// ---- §6.8-0009: vocabulary_version is a gate, not a field -----------------------------------

#[test]
fn test_namespace_vocabulary_version_is_asserted() {
    let m = validate_envelope(&build_from(&enum_fields())).unwrap();
    // reading is not asserting: validate_envelope succeeds whatever the version is.
    assert_eq!(m.vocabulary_version, 3);
    // the GATE: the matching version passes; a skew is a typed decline, not a graceful proceed.
    assert_eq!(assert_vocabulary_version(&m, 3), Ok(()));
    assert_eq!(
        assert_vocabulary_version(&m, 2),
        Err(ManifestDecline::VocabularyVersionMismatch { got: 3, built_for: 2 })
    );
}

// ---- §6.8-0010: kind is an open set ---------------------------------------------------------

#[test]
fn test_namespace_vocabulary_kind_open_set() {
    assert_eq!(validate_envelope(&build_from(&enum_fields())).unwrap().kind, Kind::Enumerated);
    assert_eq!(validate_envelope(&build_from(&gen_fields())).unwrap().kind, Kind::Generated);
    // an unrecognized `kind` is a TYPED DECLINE (naming the value), never a guess or a panic —
    // the third-kind sketch (RFC §5) is admitted additively, not by assuming the nearer known.
    assert_eq!(
        validate_envelope(&build_from(&set_key(enum_fields(), "kind", "\"probe\""))),
        Err(ManifestDecline::UnrecognizedKind { got: "probe".to_string() })
    );
}

// ---- §6.8-0011: freshness provenance --------------------------------------------------------

#[test]
fn test_namespace_vocabulary_freshness_provenance() {
    assert!(validate_envelope(&build_from(&enum_fields())).is_ok());
    // absent → MissingField (nothing names the annex the gate would regenerate from).
    assert_eq!(
        validate_envelope(&build_from(&drop_key(enum_fields(), "generated_from"))),
        Err(ManifestDecline::MissingField("generated_from"))
    );
    // present-but-empty → EmptyProvenance (a gate needs a fixed thing to compare against).
    assert_eq!(
        validate_envelope(&build_from(&set_key(enum_fields(), "generated_from", "\"\""))),
        Err(ManifestDecline::EmptyProvenance)
    );
}

// ---- §6.8-0012: the declarative / production split ------------------------------------------

#[test]
fn test_namespace_vocabulary_declarative_production_split() {
    // A generated manifest WITHOUT `vectors` (the production half) is still a valid ENVELOPE for
    // a parse-only consumer — parsers need only the declarative half.
    let m = validate_envelope(&build_from(&drop_key(gen_fields(), "vectors")))
        .expect("a parse-only consumer is served without the production half");
    // …but a PRODUCER, which runs the production check, catches the missing vector set. The two
    // audiences are structurally separate: the parser never calls the check below.
    assert_eq!(check_generated_vector_coverage(&m), Err(ManifestDecline::GeneratedMissingVectors));
    // for the enumerated kind the production check is vacuously satisfied (no canonicalization).
    let e = validate_envelope(&build_from(&enum_fields())).unwrap();
    assert_eq!(check_generated_vector_coverage(&e), Ok(()));
}

// ---- §6.8-0013: generated vectors cover canonicalization ------------------------------------

#[test]
fn test_namespace_vocabulary_generated_vectors_cover_canonicalization() {
    // all four pins present → covered.
    let full = validate_envelope(&build_from(&gen_fields())).unwrap();
    assert_eq!(check_generated_vector_coverage(&full), Ok(()));

    // missing `order` → decline naming it.
    let no_order = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"dedup\"}, {\"pins\": \"threshold\"}, {\"pins\": \"digest_input\"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&no_order)).unwrap()),
        Err(ManifestDecline::GeneratedVectorsMissingPin("order"))
    );

    // ⚠️ MISSING `dedup` MUST DECLINE TOO, and this was unexercised (#415). The test
    // presented only an `order`-less set, so `dedup` could be DROPPED from GENERATED_PINS
    // entirely -- or made exemptible -- and nothing reddened. Measured: both mutations were
    // NO-OPs. `order` and `dedup` are the two NON-EXEMPTIBLE pins, so each needs its own
    // case; a single case for one of them leaves the other free.
    let no_dedup = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"order\"}, {\"pins\": \"threshold\"}, {\"pins\": \"digest_input\"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&no_dedup)).unwrap()),
        Err(ManifestDecline::GeneratedVectorsMissingPin("dedup"))
    );

    // ... and `dedup` is NOT exemptible either, mirroring the `order` case below.
    // reuses `no_dedup`'s vector set rather than restating it (review suggestion):
    // the two cases differ ONLY in the `omits` field below.
    let mut bad_dedup = no_dedup.clone();
    bad_dedup.push(("omits", "[\"dedup\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&bad_dedup)).unwrap()),
        Err(ManifestDecline::GeneratedVectorsMissingPin("dedup"))
    );

    // a namespace with NO length-conditional field exempts threshold+digest_input and covers with
    // order+dedup alone.
    let mut exempt = set_key(gen_fields(), "vectors", "[{\"pins\": \"order\"}, {\"pins\": \"dedup\"}]");
    exempt.push(("omits", "[\"threshold\", \"digest_input\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&exempt)).unwrap()),
        Ok(())
    );

    // but `order` is NOT exemptible — exempting it does not make an order-less set pass.
    let mut bad = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"dedup\"}, {\"pins\": \"threshold\"}, {\"pins\": \"digest_input\"}]",
    );
    bad.push(("omits", "[\"order\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&bad)).unwrap()),
        Err(ManifestDecline::GeneratedVectorsMissingPin("order"))
    );
}

// ---- §6.8-0014: the derivability witness ----------------------------------------------------

/// Backs: KISS-CLASSIFY-6.8-0014 — every entry carries a derivability witness, so a
/// capability NAMED but UNREACHABLE is unrepresentable rather than merely discouraged.
///
/// KISS checks the ENVELOPE only: witness present, a non-empty LIST, references resolvable,
/// a gate NAMED. Whether the witness produces the entry, and whether the named gate runs, is
/// the maintainer's — KISS has no device, no toolchain, and §6.8-0004 puts the content out
/// of reach. Every assertion below is on the envelope half.
#[test]
fn test_namespace_vocabulary_derivability_witness() {
    // A witnessed entry, with a named gate. The reference names its HOME, which is what
    // makes it resolvable: `unpopped-vocab::ArchSku::Sm90` resolves, a bare `ArchSku::Sm90`
    // does not — both maintainers hit that from opposite sides.
    let witnessed = vec![
        ("witness_gate", "\"unpopped-vocab::tests::arch_sku_roundtrip\""),
        ("members", "[{\"token\": \"cuda:sm80\", \
                       \"derivability_witness\": [\"unpopped-vocab::ArchSku::Sm80\"]}]"),
    ];
    let ok = set_key(set_key(enum_fields(), "members", witnessed[1].1),
                     "coverage_note", "\"synthetic\"");
    let mut ok = ok;
    ok.push(witnessed[0]);
    let m = validate_envelope(&build_from(&ok)).unwrap();
    assert_eq!(check_derivability_witnesses(&m), Ok(()));

    // A LIST, and a single reference is a list of one. Vulkane's `dot8` derives from SIX
    // distinct struct members; a single-string field would have forced it to name one of
    // six arbitrarily — true of a construct, false of the entry.
    let six = "[{\"token\": \"vulkan:dot8\", \"derivability_witness\": [\
        \"v::F::a\",\"v::F::b\",\"v::F::c\",\"v::F::d\",\"v::F::e\",\"v::F::f\"]}]";
    let mut multi = set_key(enum_fields(), "members", six);
    multi.push(witnessed[0]);
    assert_eq!(check_derivability_witnesses(&validate_envelope(&build_from(&multi)).unwrap()),
               Ok(()));

    // NO witness -> typed decline naming the entry.
    let mut bare = set_key(enum_fields(), "members",
                           "[{\"token\": \"cuda:sm80\", \"notes\": \"none\"}]");
    bare.push(witnessed[0]);
    assert_eq!(
        check_derivability_witnesses(&validate_envelope(&build_from(&bare)).unwrap()),
        Err(ManifestDecline::EntryMissingWitness { token: "cuda:sm80".to_string() })
    );

    // PRESENT-BUT-EMPTY is the same defect and the likelier one: a generator emitting `[]`
    // for an entry it could not trace looks compliant to a presence check.
    let mut empty = set_key(enum_fields(), "members",
                            "[{\"token\": \"cuda:sm80\", \"derivability_witness\": []}]");
    empty.push(witnessed[0]);
    assert_eq!(
        check_derivability_witnesses(&validate_envelope(&build_from(&empty)).unwrap()),
        Err(ManifestDecline::EntryMissingWitness { token: "cuda:sm80".to_string() })
    );

    // THE LIMIT, ASSERTED RATHER THAN LEFT IMPLICIT (#340 review): the envelope check
    // cannot tell an artifact-rooted path from a type-rooted one. Both pass. A comment
    // here once claimed otherwise, naming `ArchSku::Sm90` as the counterexample -- it
    // contains `::` and passes. Recorded as a control so the claim cannot drift back.
    let mut typed = set_key(enum_fields(), "members",
        "[{\"token\": \"cuda:sm80\", \"derivability_witness\": [\"ArchSku::Sm80\"]}]");
    typed.push(witnessed[0]);
    assert_eq!(
        check_derivability_witnesses(&validate_envelope(&build_from(&typed)).unwrap()),
        Ok(()),
        "the envelope check does not distinguish a type-rooted path -- if this now declines,          the check gained a discrimination and the clause's limit note must be updated"
    );

    // A reference that does not name its home is NOT resolvable.
    let mut unres = set_key(enum_fields(), "members",
                            "[{\"token\": \"cuda:sm80\", \"derivability_witness\": [\"Sm80\"]}]");
    unres.push(witnessed[0]);
    assert_eq!(
        check_derivability_witnesses(&validate_envelope(&build_from(&unres)).unwrap()),
        Err(ManifestDecline::WitnessNotResolvable {
            token: "cuda:sm80".to_string(),
            reference: "Sm80".to_string()
        })
    );

    // NO NAMED GATE -> decline. A witness nobody evaluates is a claim in the shape of a
    // proof, and this clause would have bought nothing.
    let nogate = set_key(enum_fields(), "members", witnessed[1].1);
    assert_eq!(
        check_derivability_witnesses(&validate_envelope(&build_from(&nogate)).unwrap()),
        Err(ManifestDecline::NoWitnessGate)
    );

    // RETROACTIVITY (ruling 10): an existing v1 manifest still PARSES unchanged. The witness
    // obligation is a separate checker precisely so an existing manifest is not re-issued
    // solely to satisfy this clause — folding it into `parse` would do exactly that.
    assert!(validate_envelope(&build_from(&enum_fields())).is_ok(),
            "an existing witness-less manifest must still parse");
}

// ---- §6.8-0015: an exemption is DECLARED, not narrated --------------------------------------

/// Backs: KISS-CLASSIFY-6.8-0015 — the `omits` list must name "exactly the required pins it
/// does not supply", and a reader must decline any manifest whose declared set differs from
/// the absent set "in either direction".
///
/// ⚠️ ONE OF THE TWO DIRECTIONS WAS ALREADY ENFORCED, AND THIS SAYS WHICH. A pin that is
/// absent and UNDECLARED was already an uncovered required pin, so `GeneratedVectorsMissingPin`
/// fired for it before this clause existed — under a different code, for a different reason.
/// Only the converse, declaring a pin omitted while SUPPLYING it, is new. Recording that stops
/// the under-declaration case below from reading as evidence for code it does not exercise.
///
/// ⚠️ AND THE FIELD IS `omits`, NOT `pins_exempt`. This reader invented `pins_exempt` before
/// the exemption was made structural; that name appears nowhere in the spec (measured at
/// origin/main: 0 hits across `spec/`, control `coverage_note` 4 hits). Honouring both names
/// would leave a bypass of this very check, so the old one is gone rather than aliased.
#[test]
fn test_namespace_vocabulary_omits_matches_absent_pins() {
    let order_dedup = "[{\"pins\": \"order\"}, {\"pins\": \"dedup\"}]";

    // declared == absent -> accepted.
    let mut exact = set_key(gen_fields(), "vectors", order_dedup);
    exact.push(("omits", "[\"threshold\", \"digest_input\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&exact)).unwrap()),
        Ok(()),
        "a manifest whose `omits` equals its absent set must be accepted"
    );

    // ⚠️ OVER-DECLARATION -- the direction nothing checked. All four pins are SUPPLIED and
    // the manifest declares `threshold` omitted anyway. The clause names this explicitly:
    // an over-declaration conceals a pin that is present-but-unclaimed.
    let mut over = gen_fields();
    over.push(("omits", "[\"threshold\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&over)).unwrap()),
        Err(ManifestDecline::OmitsDeclaresASuppliedPin("threshold")),
        "declaring a pin omitted while supplying it must decline"
    );

    // UNDER-DECLARATION -- absent and undeclared. Kept because the clause requires both
    // directions to hold, but it is the PRE-EXISTING arm and this says so.
    let under = set_key(
        gen_fields(),
        "vectors",
        "[{\"pins\": \"order\"}, {\"pins\": \"dedup\"}, \
          {\"pins\": \"digest_input\", \"input\": \"a\", \"output\": \"a\"}]",
    );
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&under)).unwrap()),
        Err(ManifestDecline::GeneratedVectorsMissingPin("threshold")),
        "a pin absent from vectors and undeclared must still decline"
    );

    // "EXACTLY the required pins" -- a set containing anything else is not that set. Without
    // this, a typo'd entry exempts nothing and is silently accepted as a declaration.
    let mut bogus = gen_fields();
    bogus.push(("omits", "[\"banana\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&bogus)).unwrap()),
        Err(ManifestDecline::OmitsNamesANonRequiredPin { got: "banana".to_string() }),
        "`omits` naming a non-required pin must decline"
    );
}

// ---- §6.8-0016: a threshold vector carries the boundary it pins -----------------------------

/// Backs: KISS-CLASSIFY-6.8-0016 — a `threshold` vector carries `threshold_of` and `bytes`,
/// and for every `threshold_of` the vectors must include an N / N+1 pair whose two `output`
/// values DIFFER.
///
/// ⚠️ THE FLIP IS THE WHOLE CLAUSE; ADJACENCY ALONE IS THE THING IT REJECTS. The clause's own
/// counter-example is the case below at 3 and 4 bytes: adjacent, and both far beneath a
/// 512-byte boundary. A check satisfied by adjacency reports the requirement met while
/// asserting something strictly weaker than the clause says, which is why that case must
/// decline rather than pass.
#[test]
fn test_namespace_vocabulary_threshold_pair_straddles_its_boundary() {
    fn vectors(threshold: &str) -> String {
        format!(
            "[{{\"pins\": \"order\"}}, {{\"pins\": \"dedup\"}}, \
              {{\"pins\": \"digest_input\", \"input\": \"a\", \"output\": \"a\"}}, {threshold}]"
        )
    }
    fn check(v: &str) -> Result<(), ManifestDecline> {
        let vs = vectors(v);
        let fields = set_key(gen_fields(), "vectors", &vs);
        check_generated_vector_coverage(&validate_envelope(&build_from(&fields)).unwrap())
    }

    // N and N+1 with DIFFERENT outputs -> the pair sits on the boundary. Accepted.
    assert_eq!(
        check(
            "{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"}"
        ),
        Ok(())
    );

    // ⚠️ THE CLAUSE'S OWN COUNTER-EXAMPLE. 3 and 4 bytes are adjacent, and nothing flips.
    // An adjacency-only check accepts this; the clause does not.
    assert_eq!(
        check(
            "{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 3, \"token\": \"inline\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 4, \"token\": \"inline\"}"
        ),
        Err(ManifestDecline::ThresholdPairDoesNotFlip { field: "blob".to_string() }),
        "an adjacent pair that flips nothing declares a boundary that is not there"
    );

    // a pair that flips but is NOT adjacent establishes nothing about WHERE the boundary is.
    assert_eq!(
        check(
            "{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 100, \"token\": \"inline\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"digest\"}"
        ),
        Err(ManifestDecline::ThresholdPairNotAdjacent { field: "blob".to_string() })
    );

    // the two carried fields are each required, and named individually.
    assert_eq!(
        check("{\"pins\": \"threshold\", \"enumeration_bytes\": 512, \"token\": \"inline\"}"),
        Err(ManifestDecline::ThresholdVectorMissingField("threshold_of"))
    );
    assert_eq!(
        check("{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"token\": \"inline\"}"),
        Err(ManifestDecline::ThresholdVectorMissingField("enumeration_bytes"))
    );
    assert_eq!(
        check("{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512}"),
        Err(ManifestDecline::ThresholdVectorMissingField("token"))
    );

    // ⚠️ PER-FIELD, because §6.8-0013 says EACH length-conditional field and a namespace may
    // have more than one. `blob` straddles correctly; `name` does not. A checker that pooled
    // every threshold vector into one set would find an adjacent flipping pair and accept.
    assert_eq!(
        check(
            "{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"inline\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"name\", \"enumeration_bytes\": 8, \"token\": \"short\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"name\", \"enumeration_bytes\": 64, \"token\": \"long\"}"
        ),
        Err(ManifestDecline::ThresholdPairNotAdjacent { field: "name".to_string() }),
        "each threshold_of must straddle its own boundary; one good field does not cover another"
    );

    // ⚠️ A SHARED BYTE COUNT MUST NOT HIDE THE PARTNER THAT FLIPS, and the ORDER of the
    // three rows below is the whole point of the case.
    //
    // The clause is existential over pairs: the vectors must INCLUDE an N / N+1 pair whose
    // outputs differ. Here 512/"digest" and 513/"inline" are exactly that pair, so the
    // manifest satisfies it and must be accepted. A checker that sorts and then compares only
    // CONSECUTIVE entries sees 512/digest, 513/digest, 513/inline -- the adjacent neighbours
    // agree, the two that differ are not neighbours -- and declines a conformant manifest.
    //
    // ⚠️ AN EARLIER VERSION OF THIS CASE USED inline/inline/digest AND DISCRIMINATED NOTHING.
    // The sort is by (bytes, output), so "digest" sorted ahead of "inline" and handed the
    // neighbour scan the flipping pair anyway; the mutation SURVIVED and the comment claiming
    // otherwise was false. The outputs are chosen so the shared-count sibling sorts BETWEEN
    // the pair, which is the only arrangement that separates the two implementations.
    assert_eq!(
        check(
            "{\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 512, \"token\": \"digest\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"digest\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"blob\", \"enumeration_bytes\": 513, \"token\": \"inline\"}"
        ),
        Ok(()),
        "a shared byte count must not hide the adjacent partner that flips"
    );

    // ⚠️ THE REAL EXTERNAL MANIFEST, AND THE REASON THE KEY NAMES ARE WHAT THEY ARE (#447).
    // vulkane published `kiss-vulkan-vocab` 0.4.1 with `kind: generated` and both threshold
    // straddle pairs -- correct in substance, and the FIRST version of this clause would have
    // DECLINED it, because that version invented `bytes`/`output` without checking the one
    // implementation that exists. A reader is not conformant because it enforces a clause; it
    // is conformant when it accepts the manifests the clause is about.
    //
    // Two `threshold_of` values, each straddling its own boundary, tokens differing across it.
    assert_eq!(
        check(
            "{\"pins\": \"threshold\", \"threshold_of\": \"coop\", \"enumeration_bytes\": 512, \"token\": \"enumerated\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"coop\", \"enumeration_bytes\": 513, \"token\": \"digested\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"coopvec\", \"enumeration_bytes\": 512, \"token\": \"enumerated\"}, \
             {\"pins\": \"threshold\", \"threshold_of\": \"coopvec\", \"enumeration_bytes\": 513, \"token\": \"digested\"}"
        ),
        Ok(()),
        "the published kiss-vulkan-vocab 0.4.1 straddle pairs must be ACCEPTED"
    );
}
