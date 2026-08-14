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
        ("namespace", "\"example\""),
        ("vocabulary_version", "3"),
        ("generated_from", "\"spec/namespaces/example.md\""),
        ("kind", "\"enumerated\""),
        ("grammar", "\"example:e<N>\""),
        ("coverage_note", "\"closed list; recognition is the whole contract.\""),
        ("members", "[{\"token\": \"example:e1\", \"notes\": \"the first\"}]"),
    ]
}

fn gen_fields() -> Vec<(&'static str, &'static str)> {
    vec![
        ("schema", "\"kiss-namespace-vocabulary-v1\""),
        ("namespace", "\"example\""),
        ("vocabulary_version", "3"),
        ("generated_from", "\"spec/namespaces/example.md\""),
        ("kind", "\"generated\""),
        ("grammar", "\"example:<set>\""),
        ("coverage_note", "\"enumeration is impossible; the vectors are the contract.\""),
        ("field_spec", "{\"fields\": 1, \"separator\": \",\"}"),
        (
            "vectors",
            "[{\"pins\": \"order\", \"input\": \"b,a\", \"output\": \"a,b\"}, \
              {\"pins\": \"dedup\", \"input\": \"a,a\", \"output\": \"a\"}, \
              {\"pins\": \"threshold\", \"input\": \"at-512\", \"output\": \"inline\"}, \
              {\"pins\": \"digest_input\", \"input\": \"a,b,c\", \"output\": \"a,b,c\"}]",
        ),
    ]
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

    // a namespace with NO length-conditional field exempts threshold+digest_input and covers with
    // order+dedup alone.
    let mut exempt = set_key(gen_fields(), "vectors", "[{\"pins\": \"order\"}, {\"pins\": \"dedup\"}]");
    exempt.push(("pins_exempt", "[\"threshold\", \"digest_input\"]"));
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
    bad.push(("pins_exempt", "[\"order\"]"));
    assert_eq!(
        check_generated_vector_coverage(&validate_envelope(&build_from(&bad)).unwrap()),
        Err(ManifestDecline::GeneratedVectorsMissingPin("order"))
    );
}
