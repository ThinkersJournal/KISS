//! KISS-CONFORM-6.13-0023 — malformed contracts fail LOUDLY, as a typed decline
//! over the §6.1 hard-reject transport: never a panic, never a silent empty.
//!
//! The clause's obligation is on KISS-Conform to **supply negative vectors**, so
//! the artifact under test is the vector SET, not one hand-written assertion. Two
//! things make such a set worth having, and both are asserted here because a
//! corpus can fail either way while looking complete:
//!
//!   * **Each vector pins exactly ONE required decline.** `is_err()` cannot tell a
//!     reader that classified the fault correctly from one that declined for an
//!     unrelated reason. Checked that way, a decline taxonomy is untested — the
//!     codes may drift into each other and every assertion still passes. Every
//!     vector here asserts the exact `ContractDecline`.
//!   * **A positive control.** A corpus in which everything declines proves
//!     nothing. `well_formed_document()` must READ, or the negatives are vacuous —
//!     the same discipline `trace_gate.rs` applies to the build gate.
//!
//! TEETH — four wrong readers, each asserted against below:
//!   1. **The wrong-code decliner.** Declines, but classifies the fault wrongly.
//!      Caught by (A), and by nothing weaker than (A).
//!   2. **The decline-everything reader.** Caught by (B), the positive control.
//!   3. **The silent-empty / partial reader** — returns `Ok`, or a partial
//!      contract, in place of a decline (§6.2-0005 names this exactly). Caught by
//!      (A) and restated by (D).
//!   4. **The one-vector corpus** that satisfies "supply negative vectors"
//!      trivially. Caught by (C), which pins both the vector count and the number
//!      of DISTINCT decline variants exercised.
//!
//! And (E) is the never-panic obligation as a campaign rather than a claim: every
//! truncation prefix of every vector is fed to the reader. A panic anywhere aborts
//! the test.
//!
//! CITATION DISCIPLINE: this test cites ONLY `KISS-CONFORM-6.13-0023`.
//! Cross-references to KISS-Contract use the `§<sec>-<nnnn>` short form, which
//! does not match the citation grammar.

use kiss_conformance::contract::{
    malformed_contract_vectors, read_document, well_formed_document, ContractDecline,
};
use std::collections::BTreeSet;

/// The variant name of a decline, ignoring its payload — for counting how many
/// DISTINCT faults the corpus actually distinguishes.
fn variant_of(d: &ContractDecline) -> &'static str {
    match d {
        ContractDecline::NoMagic => "NoMagic",
        ContractDecline::MalformedHeader => "MalformedHeader",
        ContractDecline::UnknownKind { .. } => "UnknownKind",
        ContractDecline::UnknownVersion { .. } => "UnknownVersion",
        ContractDecline::BadLength { .. } => "BadLength",
        ContractDecline::BadChecksum { .. } => "BadChecksum",
        ContractDecline::Headingless => "Headingless",
        ContractDecline::MissingGuaranteesClass => "MissingGuaranteesClass",
        ContractDecline::UnknownDeterminismClass { .. } => "UnknownDeterminismClass",
    }
}

#[test]
fn test_conform_contract_malformed_decline() {
    let vectors = malformed_contract_vectors();

    // (B) POSITIVE CONTROL, asserted FIRST. If the well-formed document does not
    // read, every negative below passes for the wrong reason.
    assert!(
        read_document(&well_formed_document()).is_ok(),
        "KISS-CONFORM-6.13-0023: the positive control does not read — the negative \
         vectors below would all pass vacuously against a decline-everything reader"
    );

    // (C) The corpus is non-trivial, in both size and discrimination. A single
    // vector, or ten vectors that all provoke the same code, would satisfy
    // "supply negative vectors" while distinguishing nothing.
    assert!(
        vectors.len() >= 13,
        "KISS-CONFORM-6.13-0023: the negative corpus holds only {} vectors",
        vectors.len()
    );
    let variants: BTreeSet<&str> = vectors.iter().map(|v| variant_of(&v.expect)).collect();
    assert!(
        variants.len() >= 7,
        "KISS-CONFORM-6.13-0023: the corpus provokes only {} distinct decline variants \
         ({:?}) — it is not exercising the taxonomy, only its most reachable arm",
        variants.len(),
        variants
    );

    for v in &vectors {
        // (A) EXACTLY the pinned decline. The whole value, payload included — a
        // `BadLength` reporting the wrong declared length is a wrong answer, not
        // a near miss.
        match read_document(&v.doc) {
            Ok(_) => panic!(
                "KISS-CONFORM-6.13-0023: vector `{}` was ACCEPTED — a malformed contract \
                 read as valid is the silent-empty failure the clause forbids",
                v.name
            ),
            Err(got) => assert_eq!(
                got, v.expect,
                "KISS-CONFORM-6.13-0023: vector `{}` declined with the wrong code — the \
                 reader noticed something was wrong but classified it as `{}` where `{}` \
                 is required",
                v.name,
                variant_of(&got),
                variant_of(&v.expect)
            ),
        }

        // (D) §6.2-0005 restated as its own assertion: a decline, never a partial
        // or internally-inconsistent contract handed back in its place.
        assert!(
            read_document(&v.doc).is_err(),
            "KISS-CONFORM-6.13-0023: vector `{}` yielded a contract in place of a decline",
            v.name
        );
    }

    // (E) Never-panic as a campaign, not a claim: every truncation prefix of every
    // vector, including the empty input. Truncation is the transport's most
    // ordinary fault, and it is where an unchecked index or a length-keyed
    // allocation shows up. A panic anywhere aborts this test.
    let mut prefixes = 0usize;
    for v in &vectors {
        for n in 0..=v.doc.len() {
            let _ = read_document(&v.doc[..n]);
            prefixes += 1;
        }
    }
    assert!(
        prefixes > 1_000,
        "KISS-CONFORM-6.13-0023: the never-panic campaign covered only {prefixes} inputs — \
         too few to have exercised the reader's length and index paths"
    );
    println!("never-panic campaign: {prefixes} truncation prefixes across {} vectors", vectors.len());
}
