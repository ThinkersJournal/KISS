// SPDX-License-Identifier: MIT OR Apache-2.0
//! Section-level contract vectors (FireLine's requests): every accept document reads cleanly and
//! every decline document is declined at the section and with the tag the generator states.

use kiss_conformance::section_vectors::{
    accept_vectors, parse_positional_signature, read_full_document, section_decline_vectors, strided_signature,
    FullDecline,
};

// harness-internal: the accept documents of the section-vector set read cleanly under the reference full-document reader, no clause asserted
#[test]
fn test_section_vectors_accept_documents_read_cleanly() {
    let accepts = accept_vectors();
    assert_eq!(accepts.len(), 6, "the accept set is the strided, five-field, layouts, annotation and two in_place documents");
    for v in &accepts {
        assert_eq!(read_full_document(&v.doc), Ok(()), "accept vector {:?}", v.name);
    }
}

// harness-internal: each single-fault section-vector document declines at the stated section with the stated tag, no clause asserted
#[test]
fn test_section_vectors_decline_with_the_stated_tag() {
    let declines = section_decline_vectors();
    assert_eq!(declines.len(), 14 + 36, "14 feature declines and 6 faults in each of 6 schemas");
    for v in &declines {
        assert_eq!(
            read_full_document(&v.doc),
            Err(FullDecline { section: v.section, tag: v.expect }),
            "decline vector {:?}",
            v.name
        );
    }
}

// harness-internal: the strided signature renders to the §6.11-0006 layout and reads back, no clause asserted
#[test]
fn test_section_vectors_strided_signature_reads_back() {
    let (sig, tail) = strided_signature();
    let args = parse_positional_signature(&sig).expect("the strided signature reads");
    assert_eq!(args.len(), 10);
    assert_eq!(parse_positional_signature(&tail).expect("the tail reads").len(), 7);
    assert!(parse_positional_signature("[]").expect("empty reads").is_empty());
}
