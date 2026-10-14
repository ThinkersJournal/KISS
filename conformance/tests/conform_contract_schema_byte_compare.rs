// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.13-0012 — the KISS-Contract tier byte-compares the **seven-section
//! schema/framing** under the exact-byte comparator.
//!
//! The subject is the Appendix C golden document: the codec's rendering of it
//! (`appendix_c_golden_document`) is compared byte-for-byte with the committed foreign-reader
//! artifact `corpus/contract_vectors.json`, then its framing is read back and its seven
//! `[section:N:name]` headings are checked as present, in ascending order, and exactly seven.
//!
//! Scope, stated so the clause is not over-credited: the free-text blurb is excluded from the
//! exact-byte comparison by a sibling clause (§6.13-0020), and the Dispatch block carries the
//! geometry-agnostic sentinel, so the five-field geometry alternative is not exercised here
//! (the artifact's own `coverage_note_496` says so).

use kiss_conformance::contract::{appendix_c_golden_document, read_document};
use kiss_conformance::json::{self, Json};
use kiss_conformance::{compare, parse_hex, DeterminismClass};

const VECTORS: &str = include_str!("../corpus/contract_vectors.json");

/// The `[section:N:name]` heading ids found in `doc`, in byte order.
fn section_ids(doc: &[u8]) -> Vec<u32> {
    let text = String::from_utf8_lossy(doc);
    let mut out = Vec::new();
    let mut rest: &str = &text;
    while let Some(i) = rest.find("[section:") {
        let tail = &rest[i + "[section:".len()..];
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = digits.parse::<u32>() {
            out.push(n);
        }
        rest = tail;
    }
    out
}

/// KISS-CONFORM-6.13-0012 — the seven-section schema/framing is byte-compared, exact-byte.
#[test]
fn test_conform_contract_schema_byte_compare() {
    let v = json::parse(VECTORS).expect("contract vectors parse");
    let g = v.get("golden_document").expect("golden_document");
    let hex = g.get("bytes_hex").and_then(Json::as_str).expect("bytes_hex");
    let committed = parse_hex(hex);
    assert!(
        !committed.is_empty(),
        "KISS-CONFORM-6.13-0012: the committed golden document is empty"
    );

    // COMPARATOR: the codec's rendering equals the committed bytes, exact-byte.
    let rendered = appendix_c_golden_document();
    if let Err(e) = compare(DeterminismClass::ExactByte, &rendered, &committed) {
        panic!("KISS-CONFORM-6.13-0012: golden document differs from the committed bytes: {e}");
    }

    // FRAMING: the header reads back and its self-declared length matches the body.
    let h = read_document(&committed).expect("framing reads");
    assert_eq!(h.contract_kind, "kiss-contract", "KISS-CONFORM-6.13-0012: kind token");
    assert!(h.body_len > 0, "KISS-CONFORM-6.13-0012: empty body");

    // SEVEN SECTIONS: exactly seven headings, ids 1..=7, in order.
    let ids = section_ids(&committed);
    assert_eq!(
        ids,
        (1..=7).collect::<Vec<u32>>(),
        "KISS-CONFORM-6.13-0012: the seven-section framing is not ids 1..=7 in order"
    );

    // TEETH. One flipped byte in the body MUST fail the byte comparison ...
    let mut drift = committed.clone();
    let mid = drift.len() / 2;
    drift[mid] ^= 0x01;
    assert!(
        compare(DeterminismClass::ExactByte, &drift, &committed).is_err(),
        "KISS-CONFORM-6.13-0012: the exact-byte comparator accepted a one-byte drift"
    );
    // ... and a document with a section heading removed MUST fail the seven-section check.
    let text = String::from_utf8_lossy(&committed).replacen("[section:4:", "[xsection:4:", 1);
    assert_ne!(
        section_ids(text.as_bytes()),
        (1..=7).collect::<Vec<u32>>(),
        "KISS-CONFORM-6.13-0012: the section-id check cannot see a missing heading"
    );
}
