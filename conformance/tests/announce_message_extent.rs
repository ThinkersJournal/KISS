// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-ANNOUNCE §6.2-0014..-0017 (message extent) and §6.4-0014 (contract payload cap).
//!
//! The extent of each of the eight messages is computed from its own bytes, so the tests never frame a
//! message by anything outside it: they cut, extend and corrupt the message itself.

use kiss_conformance::announce::*;

fn messages() -> Vec<(MessageKind, Vec<u8>)> {
    vec![
        (MessageKind::Envelope, reference_envelope().encode()),
        (MessageKind::AvailabilityList, reference_availability_list().encode()),
        (MessageKind::QueryRequest, reference_cyrq().encode()),
        (MessageKind::ContractResponse, reference_crsp().encode()),
        (MessageKind::DeclineResponse, reference_cdec().encode()),
        (MessageKind::DeviceRequest, kiss_conformance::device_query::reference_dvrq().encode()),
        (MessageKind::DeviceResponse, kiss_conformance::device_query::reference_dvrs().encode()),
        (MessageKind::DeviceDecline, kiss_conformance::device_query::reference_dvdc().encode()),
    ]
}

// Backs[enforced]: KISS-ANNOUNCE-6.2-0014 — the extent of each of the eight messages is determined from the message's
// own bytes and equals the length of what the reference encoder wrote, whatever follows it.
#[test]
fn test_announce_message_extent_from_own_bytes() {
    for (kind, bytes) in messages() {
        assert_eq!(message_extent(&bytes), Ok((kind, bytes.len())), "{kind:?} alone");
        // appended bytes belong to whatever follows: the extent does not move (§6.2-0017)
        for tail in [&[0u8; 7][..], &[0xFF; 300][..], &messages()[0].1[..]] {
            let mut longer = bytes.clone();
            longer.extend_from_slice(tail);
            assert_eq!(message_extent(&longer), Ok((kind, bytes.len())), "{kind:?} + {} bytes", tail.len());
        }
    }
    // eight distinct leading tags: the control that kinds are told apart by the message, not by position
    let kinds: Vec<_> = messages().iter().map(|(k, _)| *k).collect();
    assert_eq!(kinds.len(), 8);
}

// Backs[enforced]: KISS-ANNOUNCE-6.2-0015 — a message cut at ANY length shorter than its extent is a typed decline
// (never a panic or an out-of-bounds read), whatever delivered the prefix.
#[test]
fn test_announce_truncated_message_rejected() {
    for (kind, bytes) in messages() {
        for cut in 0..bytes.len() {
            assert!(message_extent(&bytes[..cut]).is_err(), "{kind:?} cut to {cut} of {} must decline", bytes.len());
        }
        // positive control: the whole message is accepted, so the loop above is not rejecting everything
        assert!(message_extent(&bytes).is_ok(), "{kind:?} whole");
    }
    // an envelope claiming more than was supplied: 55 of 56 bytes
    let env = reference_envelope().encode();
    assert_eq!(message_extent(&env[..55]), Err(AnnounceDecline::Truncated { region: "envelope" }));
    // a list whose record_count promises records that are not there
    let mut savl = reference_availability_list().encode();
    savl[8..12].copy_from_slice(&1000u32.to_le_bytes());
    assert!(matches!(message_extent(&savl), Err(AnnounceDecline::Truncated { .. })));
}

// Backs[enforced]: KISS-ANNOUNCE-6.2-0016 — a leading tag that names none of the eight messages is a typed decline,
// and the reader does not look for a later tag.
#[test]
fn test_announce_unknown_leading_tag() {
    let mut b = reference_cyrq().encode();
    b[0] = b'X';
    assert!(matches!(message_extent(&b), Err(AnnounceDecline::BadTag { region: "leading_tag", .. })));
    // a valid message placed AFTER a bad tag is not found: no resynchronisation
    let mut shifted = vec![0xAA, 0xBB, 0xCC, 0xDD];
    shifted.extend(reference_cyrq().encode());
    assert!(matches!(message_extent(&shifted), Err(AnnounceDecline::BadTag { .. })));
    // positive control: the same message without the junk prefix is accepted
    assert!(message_extent(&reference_cyrq().encode()).is_ok());
}

// Backs[enforced]: KISS-ANNOUNCE-6.2-0017 — bytes after the extent are not read: corrupting them changes neither the
// extent nor the verdict.
#[test]
fn test_announce_bytes_after_extent_unread() {
    for (kind, bytes) in messages() {
        let mut a = bytes.clone();
        a.extend_from_slice(&[0x00; 64]);
        let mut b = bytes.clone();
        b.extend_from_slice(&[0xFF; 64]);
        assert_eq!(message_extent(&a), message_extent(&b), "{kind:?}: the tail must not matter");
    }
}

// Backs[enforced]: KISS-ANNOUNCE-6.4-0014 — a contract response declaring a payload longer than 2^24 bytes is a typed
// decline without the payload being present, and exactly 2^24 is not declined for its length.
#[test]
fn test_announce_contract_payload_cap() {
    assert_eq!(MAX_CONTRACT_PAYLOAD_LEN, 1 << 24);
    let mut crsp = reference_crsp().encode();
    let at = crsp.len() - reference_crsp().payload.len() - 4;
    crsp[at..at + 4].copy_from_slice(&(MAX_CONTRACT_PAYLOAD_LEN + 1).to_le_bytes());
    assert_eq!(
        message_extent(&crsp),
        Err(AnnounceDecline::PayloadLenOutOfRange { got: MAX_CONTRACT_PAYLOAD_LEN + 1 })
    );
    // at the cap the length itself is admissible: the decline is then the truncation, not the cap
    crsp[at..at + 4].copy_from_slice(&MAX_CONTRACT_PAYLOAD_LEN.to_le_bytes());
    assert!(matches!(message_extent(&crsp), Err(AnnounceDecline::Truncated { region: "contract_payload" })));
}
