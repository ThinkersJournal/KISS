//! KISS-Conform suite for the OPTIONAL request-correlation id on the
//! contract-query / provision request+response frames (KISS-ANNOUNCE-6.4-0013).
//!
//! Two witnesses of the clause: (a) a response echoes the request's correlation
//! id unchanged when it is present; (b) an absent correlation id is valid — a
//! synchronous seam omits it and it is never required for v1 conformance. A third
//! test pins the never-panic typed-decline discipline on a malformed tail.
//! Each test cites its pinning clause.

use kiss_conformance::correlation_id::*;

#[test]
fn test_announce_correlation_id_echoed_when_present() {
    // KISS-ANNOUNCE-6.4-0013: when a request carries a correlation id, the
    // response answering it MUST echo the identical 8 bytes unchanged.
    let req_id: CorrelationId = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];

    let resp_id = echo(Some(req_id));
    assert_eq!(
        resp_id,
        Some(req_id),
        "response MUST echo the request correlation id unchanged"
    );

    // It survives a serialize -> parse round-trip on both request and response
    // tails, and the response advertises the same 8 bytes.
    let req_tail = encode_tail(Some(req_id));
    assert_eq!(req_tail[0], 1, "correlation_present flag MUST be set");
    assert_eq!(req_tail.len(), 9, "flag byte + 8 id bytes");
    assert_eq!(parse_tail(&req_tail).unwrap(), Some(req_id));

    let resp_tail = encode_tail(resp_id);
    assert_eq!(
        parse_tail(&resp_tail).unwrap(),
        Some(req_id),
        "the echoed response tail MUST parse back to the request's id"
    );
    assert_eq!(&resp_tail[1..9], &req_id, "bytes echoed unchanged");
}

#[test]
fn test_announce_correlation_id_absent_is_valid() {
    // KISS-ANNOUNCE-6.4-0013: the field is OPTIONAL and reserved — a synchronous
    // seam omits it, and it MUST NOT be required for v1 conformance.
    assert_eq!(
        encode_tail(None),
        Vec::<u8>::new(),
        "an omitted field adds no bytes (frame is byte-identical to §6.4-0001/0004/0007)"
    );
    assert_eq!(
        parse_tail(&[]).unwrap(),
        None,
        "an empty tail is a valid absent field — a reader MUST accept it"
    );
    assert_eq!(
        parse_tail(&[0]).unwrap(),
        None,
        "an explicit correlation_present == 0 is absent"
    );
    // A response to a request that carried none carries none.
    assert_eq!(echo(None), None);
}

#[test]
fn test_announce_correlation_bad_present_flag_declines() {
    // KISS-ANNOUNCE-6.4-0013: a correlation_present other than 0 or 1 is a typed
    // MALFORMED_REQUEST error (Announce §6.4-0009), never a panic.
    assert_eq!(
        parse_tail(&[2, 0, 0, 0, 0, 0, 0, 0, 0]),
        Err(CorrelationError::BadPresentFlag { got: 2 })
    );
    // A present flag of 1 with a truncated id is a typed error, not an
    // out-of-bounds panic.
    assert_eq!(parse_tail(&[1, 0xAA, 0xBB]), Err(CorrelationError::Truncated));
}
