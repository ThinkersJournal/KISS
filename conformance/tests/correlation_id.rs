// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform suite for the request-correlation FIELD on the contract-query / provision frames
//! (KISS-ANNOUNCE-6.4-0013): always present (so a frame is self-delimiting), spelled `00` when no id is
//! carried, echoed unchanged when one is.

use kiss_conformance::announce::*;
use kiss_conformance::correlation_id::*;

const ID: CorrelationId = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];

// Backs[enforced]: KISS-ANNOUNCE-6.4-0013
#[test]
fn test_announce_correlation_id_echoed_when_present() {
    // KISS-ANNOUNCE-6.4-0013: when a request carries a correlation id, the response answering it MUST echo the
    // identical 8 bytes unchanged.
    assert_eq!(echo(Some(ID)), Some(ID), "response MUST echo the request correlation id unchanged");
    let field = encode_field(Some(ID));
    assert_eq!(field[0], 8, "correlation_len MUST be 8");
    assert_eq!(field.len(), 9, "length byte + 8 id bytes");
    assert_eq!(parse_field(&field), Ok((Some(ID), 9)));
    assert_eq!(&field[1..], &ID, "bytes echoed unchanged");
    // in a real frame: a request that carries an id yields a response whose identity block carries the same one
    let mut req = reference_cyrq();
    req.identity.correlation = Some(ID);
    let mut resp = reference_crsp();
    resp.identity.correlation = echo(req.identity.correlation);
    assert_eq!(resp.identity.correlation, Some(ID));
    assert!(resp.encode().windows(9).any(|w| w == field.as_slice()), "the 9 field bytes appear in the response");
}

// Backs[enforced]: KISS-ANNOUNCE-6.4-0013 — the correlation field is always present: with no id it is the single byte 00, never nothing, and a frame cut before it is truncated
#[test]
fn test_announce_correlation_field_always_present() {
    assert_eq!(encode_field(None), vec![0u8], "no id is spelled as one zero byte, never as nothing");
    assert_eq!(parse_field(&[0]), Ok((None, 1)));
    // an empty slice is NOT an absent field: the field is mandatory, so a missing length byte is truncation
    assert_eq!(parse_field(&[]), Err(AnnounceDecline::Truncated { region: "correlation_len" }));
    // in a real frame: the identity block ends with the field
    let cyrq = reference_cyrq().encode();
    assert_eq!(*cyrq.last().unwrap(), 0, "a CYRQ with no id ends with the zero correlation byte");
    assert!(message_extent(&cyrq).is_ok());
    assert!(message_extent(&cyrq[..cyrq.len() - 1]).is_err(), "without its correlation byte the CYRQ is truncated");
    // a response to a request that carried none carries none
    assert_eq!(echo(None), None);
}

// Backs[enforced]: KISS-ANNOUNCE-6.4-0013 — a reader rejects a correlation length other than 0 or 8, and a field cut short, with a typed decline, never a panic
#[test]
fn test_announce_correlation_bad_length_declines() {
    assert_eq!(parse_field(&[2, 0, 0]), Err(AnnounceDecline::BadCorrelationLen { got: 2 }));
    assert_eq!(parse_field(&[9, 0, 0, 0, 0, 0, 0, 0, 0, 0]), Err(AnnounceDecline::BadCorrelationLen { got: 9 }));
    assert_eq!(parse_field(&[8, 0xAA, 0xBB]), Err(AnnounceDecline::Truncated { region: "correlation_id" }));
    // positive control: the same bytes with a valid length parse
    assert!(parse_field(&[8, 1, 2, 3, 4, 5, 6, 7, 8]).is_ok());
    // in a frame: a CYRQ whose correlation length byte is corrupted is a typed decline
    let mut cyrq = reference_cyrq().encode();
    let last = cyrq.len() - 1;
    cyrq[last] = 3;
    assert_eq!(message_extent(&cyrq), Err(AnnounceDecline::BadCorrelationLen { got: 3 }));
}
