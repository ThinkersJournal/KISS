// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-ANNOUNCE §6.6 — the device-query frames and the per-device record (ROADMAP R1.33 / R1.36,
//! CireSnave board 355 "D").
//!
//! Negative controls: a record claiming a dtype its device does not compute, a compute set that
//! outruns the store set, an unknown or reserved dtype token and a non-canonical set are each declined
//! by the reader; a kernel is never admitted for a dtype outside the intersection.

use kiss_conformance::announce::{decline_code, message_extent, AnnounceDecline, MessageKind};
use kiss_conformance::device_query::*;

fn dtypes(v: &[&str]) -> Vec<String> {
    v.iter().map(|t| t.to_string()).collect()
}

fn with(f: impl FnOnce(&mut DeviceRecord)) -> Vec<u8> {
    let mut d = reference_device();
    f(&mut d);
    DeviceResponse { devices: vec![d], correlation: None }.encode()
}

// Backs[enforced]: KISS-ANNOUNCE-7.2-0012
#[test]
fn test_announce_device_query_bit_assignment() {
    assert_eq!(FEAT_DEVICE_QUERY_BIT, 35);
    // FEAT core sub-range [24, 40), distinct from the contract-query (33) and the reserved attestation bit (34)
    assert!((24..40).contains(&FEAT_DEVICE_QUERY_BIT));
    assert!(![32, 33, 34].contains(&FEAT_DEVICE_QUERY_BIT));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0003
#[test]
fn test_announce_device_request_framing() {
    // wire-order tag (a big-endian tag write is the defect this catches)
    assert_eq!(&reference_dvrq().encode()[..4], b"DVRQ");
    assert_eq!(reference_dvrq().encode(), [0x44, 0x56, 0x52, 0x51, 0x00]);
    let rq = DeviceRequest { correlation: Some([7; 8]) };
    assert_eq!(decode_request(&rq.encode()), Ok(rq));
    // a wrong tag is a typed decline, not a misread
    assert!(matches!(decode_request(&reference_dvrs().encode()), Err(AnnounceDecline::BadTag { .. })));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0004
#[test]
fn test_announce_device_response_framing() {
    assert_eq!(&reference_dvrs().encode()[..4], b"DVRS");
    assert_eq!(decode_response(&reference_dvrs().encode()), Ok(reference_dvrs()));
    // an empty enumeration is a valid answer: a provider with no device says so
    let none = DeviceResponse { devices: vec![], correlation: None };
    assert_eq!(decode_response(&none.encode()), Ok(none));
    assert!(matches!(decode_response(&reference_dvrq().encode()), Err(AnnounceDecline::BadTag { .. })));
    let mut count = reference_dvrs().encode();
    count[4..8].copy_from_slice(&(MAX_DEVICES + 1).to_le_bytes());
    assert_eq!(decode_response(&count), Err(AnnounceDecline::DeviceCountOverflow { got: MAX_DEVICES + 1 }));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0005
#[test]
fn test_announce_device_decline_framing() {
    assert_eq!(&reference_dvdc().encode()[..4], b"DVDC");
    let dec = DeviceDecline { decline_code: decline_code::QUERY_NOT_SUPPORTED, correlation: Some([1; 8]) };
    assert_eq!(decode_decline(&dec.encode()), Ok(dec));
    // an unrecognised code still frames and reads: the consumer treats it as a generic decline (§6.4-0010)
    let odd = DeviceDecline { decline_code: 0x4000_0001, correlation: None };
    assert_eq!(decode_decline(&odd.encode()), Ok(odd));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0006
#[test]
fn test_announce_device_record_bounds() {
    // device_id length 0 and above the cap, with no allocation on the unchecked length
    for bad in [0u32, MAX_DEVICE_ID_LEN + 1, u32::MAX] {
        let mut b = reference_dvrs().encode();
        b[8..12].copy_from_slice(&bad.to_le_bytes());
        assert_eq!(decode_response(&b), Err(AnnounceDecline::DeviceIdLenOutOfRange { got: bad }), "{bad}");
    }
    // a truncated response at every length is a typed decline, never a panic
    let whole = reference_dvrs().encode();
    for cut in 0..whole.len() {
        assert!(decode_response(&whole[..cut]).is_err(), "cut {cut}");
    }
    assert!(decode_response(&whole).is_ok());
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0007
#[test]
fn test_announce_device_record_claims_are_consistent() {
    // NEGATIVE CONTROL: the device claims accuracy for f64, which it does not compute on
    let r = with(|d| {
        d.store_dtypes = dtypes(&["bf16", "f16", "f32", "f64", "i8"]);
        d.accuracy.push(AccuracyRecord { op: "exp".into(), math_fidelity: 1, dtype: "f64".into(), max_ulp: 1 });
    });
    assert_eq!(decode_response(&r), Err(AnnounceDecline::AccuracyDtypeNotComputable { dtype: "f64".into() }));
    // NEGATIVE CONTROL: it computes on f64 and stores no f64
    let r = with(|d| d.compute_dtypes = dtypes(&["bf16", "f16", "f32", "f64"]));
    assert_eq!(decode_response(&r), Err(AnnounceDecline::ComputeNotInStore { dtype: "f64".into() }));
    // a spelling outside the Classify closed table, and a reserved one
    let r = with(|d| {
        d.store_dtypes = dtypes(&["bf16", "f16", "f32", "i8", "quad"]);
    });
    assert_eq!(decode_response(&r), Err(AnnounceDecline::DtypeUnknown { region: "store_dtypes" }));
    let r = with(|d| d.store_dtypes = dtypes(&["bf16", "f16", "f32", "f8e4m3fnuz"]));
    assert_eq!(decode_response(&r), Err(AnnounceDecline::DtypeReserved { region: "store_dtypes" }));
    // two spellings of one set: a non-ascending or repeated set
    let r = with(|d| d.store_dtypes = dtypes(&["f16", "bf16", "f32", "i8"]));
    assert_eq!(decode_response(&r), Err(AnnounceDecline::SetNotCanonical { region: "store_dtypes" }));
    let r = with(|d| d.compute_dtypes = dtypes(&["bf16", "bf16"]));
    assert_eq!(decode_response(&r), Err(AnnounceDecline::SetNotCanonical { region: "compute_dtypes" }));
    // a target that is not <ns>:<cap>
    for bad in ["cuda", ":sm89", "cuda:"] {
        let r = with(|d| d.target = bad.to_string());
        assert_eq!(decode_response(&r), Err(AnnounceDecline::TokenMalformed { region: "target" }), "{bad}");
    }
    // POSITIVE CONTROL: the reference record, which stores more than it computes, is accepted
    assert!(decode_response(&reference_dvrs().encode()).is_ok());
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0008
#[test]
fn test_announce_device_accuracy_record() {
    let r = with(|d| d.accuracy[0].math_fidelity = 3);
    assert_eq!(decode_response(&r), Err(AnnounceDecline::BadMathFidelity { got: 3 }));
    // the key (op, math_fidelity, dtype) is unique and ascending: a repeated key is a second spelling
    let r = with(|d| d.accuracy.push(d.accuracy[1].clone()));
    assert_eq!(decode_response(&r), Err(AnnounceDecline::AccuracyNotCanonical));
    let r = with(|d| d.accuracy.reverse());
    assert_eq!(decode_response(&r), Err(AnnounceDecline::AccuracyNotCanonical));
    // with no accuracy records the count is the 4 bytes before the 1-byte correlation field
    let mut huge = with(|d| d.accuracy.clear());
    let at = huge.len() - 5;
    huge[at..at + 4].copy_from_slice(&(MAX_ACCURACY_RECORDS + 1).to_le_bytes());
    assert_eq!(decode_response(&huge), Err(AnnounceDecline::AccuracyCountOverflow { got: MAX_ACCURACY_RECORDS + 1 }));
    // max_ulp u32::MAX means "no bound", and a ulp of 0 is a real (exact) bound, not "none"
    let r = with(|d| d.accuracy[0].max_ulp = NO_ULP_BOUND);
    assert_eq!(decode_response(&r).unwrap().devices[0].accuracy[0].max_ulp, NO_ULP_BOUND);
    let r = with(|d| d.accuracy[0].max_ulp = 0);
    assert_eq!(decode_response(&r).unwrap().devices[0].accuracy[0].max_ulp, 0);
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0009
#[test]
fn test_announce_effective_support_is_intersection() {
    let dev = reference_device(); // computes bf16, f16, f32; stores i8 too
    // the kernel declares f32, f64, i8: the device computes only f32 of them
    assert_eq!(effective_dtypes(&["f32", "f64", "i8"], &dev), dtypes(&["f32"]));
    assert_eq!(admit_dtype(&["f32", "f64", "i8"], "f32", &dev), Ok(()));
    // NEGATIVE CONTROL: the kernel offers f64 and the device does not compute on it
    assert_eq!(admit_dtype(&["f32", "f64"], "f64", &dev), Err(PairingDecline::NotComputedByDevice { dtype: "f64".into() }));
    // stored but not computed: i8 is in the store set, so the device can hold it and still not run a kernel on it
    assert_eq!(admit_dtype(&["i8"], "i8", &dev), Err(PairingDecline::NotComputedByDevice { dtype: "i8".into() }));
    // the device computes bf16 and the kernel does not offer it
    assert_eq!(admit_dtype(&["f32"], "bf16", &dev), Err(PairingDecline::NotOfferedByKernel { dtype: "bf16".into() }));
    // two devices of one backend answer differently: there is no backend-wide dtype list
    let mut other = reference_device();
    other.device_id = b"gpu1".to_vec();
    other.compute_dtypes = dtypes(&["f32"]);
    assert_ne!(effective_dtypes(&["bf16", "f32"], &dev), effective_dtypes(&["bf16", "f32"], &other));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0010
#[test]
fn test_announce_kernel_accuracy_not_looser_than_device_floor() {
    let dev = reference_device(); // exp/f32: 1 ulp bit-stable, 4 ulp reduced-mantissa
    assert_eq!(effective_max_ulp(&dev, "exp", FIDELITY_BIT_STABLE, "f32", Some(1)), Ok(Some(1)));
    // a tighter kernel keeps its own bound
    assert_eq!(effective_max_ulp(&dev, "exp", FIDELITY_BIT_STABLE, "f32", Some(0)), Ok(Some(0)));
    // NEGATIVE CONTROL: a kernel looser than the device floor under the same key is declined
    assert_eq!(
        effective_max_ulp(&dev, "exp", FIDELITY_BIT_STABLE, "f32", Some(2)),
        Err(PairingDecline::KernelLooserThanDeviceFloor { kernel: 2, device: 1 })
    );
    // the same 2-ulp kernel is within the reduced-mantissa key: the mode is part of the key
    assert_eq!(effective_max_ulp(&dev, "exp", FIDELITY_REDUCED_MANTISSA, "f32", Some(2)), Ok(Some(2)));
    // a kernel that states nothing runs at the device floor; no device claim and no kernel claim is no bound
    assert_eq!(effective_max_ulp(&dev, "exp", FIDELITY_BIT_STABLE, "f32", None), Ok(Some(1)));
    assert_eq!(effective_max_ulp(&dev, "log", FIDELITY_BIT_STABLE, "f32", None), Ok(None));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0002
#[test]
fn test_announce_device_query_no_bit_declines() {
    // a provider without the bit answers with the pinned QUERY_NOT_SUPPORTED decline, framed as a DVDC
    let dec = reference_dvdc();
    assert_eq!(dec.decline_code, decline_code::QUERY_NOT_SUPPORTED);
    assert_eq!(message_extent(&dec.encode()), Ok((MessageKind::DeviceDecline, dec.encode().len())));
}

// Backs[enforced]: KISS-ANNOUNCE-6.6-0001
#[test]
fn test_announce_device_query_bit_implies_endpoint() {
    // the three frames are told apart by message extent: the endpoint a provider with the bit implements
    assert_eq!(message_extent(&reference_dvrq().encode()), Ok((MessageKind::DeviceRequest, 5)));
    let rs = reference_dvrs().encode();
    assert_eq!(message_extent(&rs), Ok((MessageKind::DeviceResponse, rs.len())));
}
