// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform tests for the cross-section equalities of a live KISS-Contract record, driven
//! through `contract::check_record_consistency` (the reference record-consistency validator).
//!
//! `contract_schema.rs` records why these four clauses stayed unbacked: each is a runtime
//! value-equality between two fields of one record, a schema-presence lint proves only that the
//! equality is well-formed, and no record-consistency validator existed to drive the disagreement
//! path. The validator now exists; every test below pairs a CONTROL that must pass with a
//! one-field MUTATION that must decline, so a validator that declined everything, or nothing,
//! fails.
//!
//! Clauses bound: KISS-CONTRACT-6.3-0009, 6.7-0002, 6.9-0003, 6.9-0006.

use kiss_conformance::contract::*;

/// The knobs a test turns. `Default` is the consistent control record.
#[derive(Clone)]
struct Knobs {
    identity_kind: &'static str,
    identity_version: &'static str,
    identity_revision_hash: &'static str,
    identity_accept_predicate: &'static str,
    capabilities_accept_predicate: &'static str,
    provenance_revision_hash: &'static str,
    guarantees_cost_provenance: &'static str,
    provenance_cost_provenance: &'static str,
}

impl Default for Knobs {
    fn default() -> Self {
        Knobs {
            identity_kind: "kiss-contract",
            identity_version: "4",
            identity_revision_hash: "abc",
            identity_accept_predicate: "sk6|ew2|f32",
            capabilities_accept_predicate: "sk6|ew2|f32",
            provenance_revision_hash: "abc",
            guarantees_cost_provenance: "measured",
            provenance_cost_provenance: "measured",
        }
    }
}

fn s(v: &str) -> Value {
    Value::Str(v.into())
}

/// A full seven-section document, header `len`/`crc32` recomputed so the framing is valid for
/// every variant: only the cross-section values differ between a control and its mutation.
fn doc(k: &Knobs) -> Vec<u8> {
    let identity = render_block(
        1,
        "identity",
        &[
            ("contract_kind", s(k.identity_kind)),
            ("contract_version", s(k.identity_version)),
            ("kernel_name", s("add_f32_strided_sm89")),
            ("revision_hash", s(k.identity_revision_hash)),
            ("accept_predicate", s(k.identity_accept_predicate)),
            ("op_identity", s("add")),
            ("target_capability", s("cuda:sm89")),
        ],
    );
    // Capabilities / Guarantees / Provenance are the Appendix C blocks with the knobbed field
    // replaced, so everything else stays the schema-valid reference content.
    let swap = |block: Vec<u8>, key: &str, value: &str| -> Vec<u8> {
        let text = String::from_utf8(block).unwrap();
        let mut out = String::new();
        let mut hit = 0;
        for line in text.split_inclusive('\n') {
            if line.starts_with(&format!("{key} = ")) {
                out.push_str(&format!("{key} = {value}\n"));
                hit += 1;
            } else {
                out.push_str(line);
            }
        }
        assert_eq!(hit, 1, "exactly one `{key}` line in the block");
        out.into_bytes()
    };
    let capabilities = swap(appendix_c_capabilities_block(), "accept_predicate", k.capabilities_accept_predicate);
    let guarantees = swap(appendix_c_guarantees_block(), "cost_provenance", k.guarantees_cost_provenance);
    let provenance = swap(
        swap(appendix_c_provenance_block(), "revision_hash", k.provenance_revision_hash),
        "cost_provenance",
        k.provenance_cost_provenance,
    );
    let mut body = Vec::new();
    for block in [
        identity,
        appendix_c_semantics_block(),
        appendix_c_interface_block(),
        appendix_c_dispatch_block(),
        capabilities,
        guarantees,
        provenance,
    ] {
        body.extend_from_slice(&block);
    }
    Document { contract_kind: "kiss-contract".into(), contract_version: "4".into(), body }.encode()
}

fn control() -> Vec<u8> {
    doc(&Knobs::default())
}

// harness-internal: control case of the reference record-consistency validator (a validator self-check, no clause asserted on its own)
#[test]
fn control_record_is_consistent_and_framing_valid() {
    assert!(read_document(&control()).is_ok(), "the control reads cleanly at the framing layer");
    assert_eq!(check_record_consistency(&control()), Ok(()));
}

/// KISS-CONTRACT-6.3-0009 — the Identity `contract_kind` and `contract_version` MUST equal the
/// transport header's, byte-for-byte; a reader MUST reject a contract whose Identity copies
/// disagree with the header.
/// TEETH: the control passes; changing only the Identity version, or only the Identity kind,
/// declines with the field named and both values reported; the document framing stays valid
/// throughout, so the decline is the record check and not the transport reader.
#[test]
fn test_contract_identity_header_match() {
    assert_eq!(check_record_consistency(&control()), Ok(()));

    let v = doc(&Knobs { identity_version: "2", ..Knobs::default() });
    assert!(read_document(&v).is_ok(), "framing is still valid");
    assert_eq!(
        check_record_consistency(&v),
        Err(RecordDecline::IdentityHeaderMismatch {
            field: "contract_version",
            identity: "2".into(),
            header: "4".into()
        })
    );

    let k = doc(&Knobs { identity_kind: "kiss-contrac", ..Knobs::default() });
    assert_eq!(
        check_record_consistency(&k),
        Err(RecordDecline::IdentityHeaderMismatch {
            field: "contract_kind",
            identity: "kiss-contrac".into(),
            header: "kiss-contract".into()
        })
    );
}

/// KISS-CONTRACT-6.7-0002 — the Capabilities `accept_predicate` MUST be the same `structure_key`
/// the Identity section carries, byte-for-byte; a Capabilities accept-predicate that disagrees
/// with the Identity one is non-conforming.
/// TEETH: the control passes; a one-byte difference in the Capabilities copy declines; a
/// difference in case or a trailing space is a difference (byte-for-byte, no normalization).
#[test]
fn test_contract_capabilities_accept_matches_identity() {
    for other in ["sk6|ew2|f33", "SK6|ew2|f32", "sk6|ew2|f32 "] {
        let d = doc(&Knobs { capabilities_accept_predicate: other, ..Knobs::default() });
        assert_eq!(
            check_record_consistency(&d),
            Err(RecordDecline::AcceptPredicateMismatch {
                identity: "sk6|ew2|f32".into(),
                capabilities: other.into()
            }),
            "capabilities copy `{other}`"
        );
    }
    // Both copies changed together stay consistent: the check is equality, not a fixed value.
    let both = doc(&Knobs {
        identity_accept_predicate: "sk6|ew2|f16",
        capabilities_accept_predicate: "sk6|ew2|f16",
        ..Knobs::default()
    });
    assert_eq!(check_record_consistency(&both), Ok(()));
}

/// KISS-CONTRACT-6.9-0003 — the Provenance `revision_hash` MUST equal the Identity
/// `revision_hash` byte-for-byte.
/// TEETH: the control passes; a disagreeing Provenance value declines naming both; equality is
/// of the serialized value, so the same bytes spelled in the opaque-blob form versus a bare string
/// are a disagreement; both fields changed together stay consistent.
#[test]
fn test_contract_provenance_revision_matches_identity() {
    let d = doc(&Knobs { provenance_revision_hash: "abd", ..Knobs::default() });
    assert_eq!(
        check_record_consistency(&d),
        Err(RecordDecline::RevisionHashMismatch { identity: "abc".into(), provenance: "abd".into() })
    );

    let spelled = doc(&Knobs { provenance_revision_hash: "3:616263", ..Knobs::default() });
    assert!(matches!(check_record_consistency(&spelled), Err(RecordDecline::RevisionHashMismatch { .. })));

    let both = doc(&Knobs {
        identity_revision_hash: "4:deadbeef",
        provenance_revision_hash: "4:deadbeef",
        ..Knobs::default()
    });
    assert_eq!(check_record_consistency(&both), Ok(()));
}

/// KISS-CONTRACT-6.9-0006 — the Provenance `cost_provenance` MUST equal the Guarantees
/// `cost_provenance`.
/// TEETH: the control passes (both `measured`); changing only one side, in either direction,
/// declines; changing both stays consistent.
#[test]
fn test_contract_cost_provenance_consistent() {
    let g = doc(&Knobs { guarantees_cost_provenance: "declared", ..Knobs::default() });
    assert_eq!(
        check_record_consistency(&g),
        Err(RecordDecline::CostProvenanceMismatch { guarantees: "declared".into(), provenance: "measured".into() })
    );
    let p = doc(&Knobs { provenance_cost_provenance: "declared", ..Knobs::default() });
    assert_eq!(
        check_record_consistency(&p),
        Err(RecordDecline::CostProvenanceMismatch { guarantees: "measured".into(), provenance: "declared".into() })
    );
    let both = doc(&Knobs {
        guarantees_cost_provenance: "declared",
        provenance_cost_provenance: "declared",
        ..Knobs::default()
    });
    assert_eq!(check_record_consistency(&both), Ok(()));
}

/// Not a bound clause: the validator is a typed decline, never a panic, on malformed input. A
/// document with a missing section, a framing error, or a block that violates its field schema
/// declines with the matching variant.
// harness-internal: the reference validator must be a typed decline, not a panic, on malformed input (validator robustness)
#[test]
fn record_check_declines_malformed_input_without_panicking() {
    assert!(matches!(check_record_consistency(b""), Err(RecordDecline::Framing(_))));
    assert!(matches!(
        check_record_consistency(&well_formed_document()),
        Err(RecordDecline::Section { section: "identity", .. })
    ));
    let identity_only = Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "4".into(),
        body: appendix_c_identity_block(),
    }
    .encode();
    assert_eq!(check_record_consistency(&identity_only), Err(RecordDecline::MissingSection("guarantees")));
    let c = control();
    for n in 0..c.len() {
        let _ = check_record_consistency(&c[..n]); // must not panic on any prefix
    }
}

/// Not a bound clause: the Appendix C golden document (the foreign-reader target of 8-0004 and
/// 8-0005) is itself a consistent record. It was not, before 0.4.40: its Identity and Capabilities
/// `accept_predicate` disagreed (and neither was a valid `structure_key`), and its Identity and
/// Provenance `revision_hash` were spelled differently. Appendix C and this golden were corrected
/// together, so a regression in either shows here.
// harness-internal: fixture self-consistency of the Appendix C golden document against the suite's own validator
#[test]
fn appendix_c_golden_document_is_record_consistent() {
    assert_eq!(check_record_consistency(&appendix_c_golden_document()), Ok(()));
}
