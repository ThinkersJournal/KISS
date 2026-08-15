//! §6.11-0011 bundle / catalog member isolation (#29 item 2): a malformed member in a bundle
//! of contract documents is typed-declined in isolation; its well-formed siblings still parse,
//! and reading never panics (§6.1-0004). Isolation rides the §6.11-0003 self-delimiting frame.
//!
//! Oracle: [`kiss_conformance::bundle_isolation`] over the real `kiss_conformance::contract`
//! transport codec.

use kiss_conformance::bundle_isolation::{read_bundle, FrameError, MemberOutcome};
use kiss_conformance::contract::{ContractDecline, Document};

/// Build a well-formed contract-document member: a body under the first pinned section heading
/// (`[section:1:identity]`, §6.11-0004) plus `tail`, wrapped in the §6.11-0002/-0003 frame.
fn member(tail: &str) -> Vec<u8> {
    let mut body = b"[section:1:identity]\n".to_vec();
    body.extend_from_slice(tail.as_bytes());
    Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "1".into(),
        body,
    }
    .encode()
}

/// §6.11-0011 — a bundle whose middle member has a corrupted body still yields both valid
/// siblings; the malformed member is a typed decline (bad CRC), not a panic, and not
/// bundle-fatal.
#[test]
fn test_contract_bundle_member_isolation() {
    let a = member("op_identity = add\n");
    let mut b = member("op_identity = mul\n");
    let c = member("op_identity = exp\n");

    // Corrupt one BODY byte of the middle member AFTER its header line: the frame `len=<N>` is
    // unchanged (so the member is still delimited), but the body CRC-32 now mismatches.
    let last = b.len() - 1;
    b[last] ^= 0xFF;

    let mut stream = Vec::new();
    stream.extend_from_slice(&a);
    stream.extend_from_slice(&b);
    stream.extend_from_slice(&c);

    let (outcomes, frame_err) = read_bundle(&stream);

    // The corrupt member is well-framed, so reading completes with no terminal frame error.
    assert_eq!(frame_err, None, "a body-corrupt member must not break framing");
    assert_eq!(outcomes.len(), 3, "every member is delimited and reported");

    // Both well-formed siblings parse; the middle member is an ISOLATED typed decline.
    assert!(
        matches!(outcomes[0], MemberOutcome::Accepted(_)),
        "well-formed sibling before the bad member must still parse"
    );
    assert!(
        matches!(outcomes[1], MemberOutcome::Declined(ContractDecline::BadChecksum { .. })),
        "the malformed member is a typed decline (bad CRC), never a panic or poison"
    );
    assert!(
        matches!(outcomes[2], MemberOutcome::Accepted(_)),
        "well-formed sibling after the bad member must still parse — isolation, not bundle-fatal"
    );
}

/// §6.11-0011 — a headingless (magic-less body) member is likewise isolated: its siblings still
/// parse and it declines rather than importing as an empty/no-op contract (§6.1-0002).
// Backs: KISS-CONTRACT-6.1-0002, KISS-CONTRACT-6.11-0011
#[test]
fn test_contract_bundle_headingless_member_isolated() {
    let a = member("op_identity = add\n");
    // A well-framed member whose body does NOT start with the first pinned heading.
    let headingless = Document {
        contract_kind: "kiss-contract".into(),
        contract_version: "1".into(),
        body: b"garbage-with-no-heading\n".to_vec(),
    }
    .encode();
    let c = member("op_identity = exp\n");

    let mut stream = Vec::new();
    stream.extend_from_slice(&a);
    stream.extend_from_slice(&headingless);
    stream.extend_from_slice(&c);

    let (outcomes, frame_err) = read_bundle(&stream);
    assert_eq!(frame_err, None);
    assert_eq!(outcomes.len(), 3);
    assert!(matches!(outcomes[0], MemberOutcome::Accepted(_)));
    assert!(matches!(outcomes[1], MemberOutcome::Declined(ContractDecline::Headingless)));
    assert!(matches!(outcomes[2], MemberOutcome::Accepted(_)));
}

/// §6.11-0011 — a member whose FRAME is unrecoverable ends reading with a terminal typed
/// decline (never a panic), yet every well-formed member already read is preserved: a frame
/// break cannot un-read an earlier sibling.
// Backs: KISS-CONTRACT-6.11-0011
#[test]
fn test_contract_bundle_frame_break_preserves_prefix() {
    let a = member("op_identity = add\n");
    let b = member("op_identity = mul\n");

    let mut stream = Vec::new();
    stream.extend_from_slice(&a);
    stream.extend_from_slice(&b);
    // A trailing member with the magic but no header LF — its frame cannot be delimited.
    stream.extend_from_slice(b"KISC kiss-contract 1 len=");

    let (outcomes, frame_err) = read_bundle(&stream);
    assert_eq!(frame_err, Some(FrameError::UnterminatedHeader));
    assert_eq!(outcomes.len(), 2, "the two well-formed siblings are preserved");
    assert!(matches!(outcomes[0], MemberOutcome::Accepted(_)));
    assert!(matches!(outcomes[1], MemberOutcome::Accepted(_)));
}

/// §6.1-0004 — an empty bundle is not a panic: it reads as zero members, no frame error.
// Backs: KISS-CONTRACT-6.1-0004
#[test]
fn test_contract_bundle_empty_is_not_a_panic() {
    let (outcomes, frame_err) = read_bundle(&[]);
    assert!(outcomes.is_empty());
    assert_eq!(frame_err, None);
}