// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Synth §6.6-0001d: a consumer turns every failure it observes at the seam into a typed
//! error. The reference consumer-side reader is `decode_provision_success`; this drives it with
//! every truncation of a valid frame, a corrupted tag, and a length over its cap, and requires a
//! typed `Err` each time. A reader that indexed, sliced or unwrapped past the end would panic and
//! fail the test.

use kiss_conformance::announce::Identity;
use kiss_conformance::synth::*;

fn valid() -> Vec<u8> {
    ProvisionSuccess {
        identity: Identity {
            op_identity: b"add".to_vec(),
            structure_key: vec![0xAA, 0xBB, 0xCC],
            revision_hash: Some([0x11; 32]),
            correlation: None,
        },
        artifact_format_tag: *b"PTX ",
        artifact: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
        contract: b"KISS-CONTRACT-STUB".to_vec(),
    }
    .encode()
    .expect("the control frame encodes")
}

/// KISS-SYNTH-6.6-0001d.
#[test]
fn test_synth_consumer_seam_failure_is_typed_error() {
    // Clause: KISS-SYNTH-6.6-0001d.
    let frame = valid();
    // Positive control: the whole frame decodes, so the declines below are about the damage.
    assert!(decode_provision_success(&frame).is_ok());

    // Every strict prefix is a typed decline, never a panic.
    for cut in 0..frame.len() {
        assert!(decode_provision_success(&frame[..cut]).is_err(), "prefix of {cut} bytes must decline");
    }
    // A corrupted leading tag declines.
    let mut bad_tag = frame.clone();
    bad_tag[0] ^= 0xFF;
    assert!(decode_provision_success(&bad_tag).is_err());
    // An artifact length over its cap declines before any byte is read for it (§6.4-0001e).
    let at = frame.len() - 18 - 4 - 5 - 8;
    let mut over = frame.clone();
    over[at..at + 8].copy_from_slice(&(MAX_ARTIFACT_LEN + 1).to_le_bytes());
    assert!(matches!(decode_provision_success(&over), Err(SynthDecline::LengthOutOfRange { .. })));
    // Garbage of every small length declines or decodes, and never panics.
    for n in 0..64usize {
        let junk: Vec<u8> = (0..n).map(|i| (i * 31 + 7) as u8).collect();
        let _ = decode_provision_success(&junk);
    }
}
