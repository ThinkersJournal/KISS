// SPDX-License-Identifier: MIT OR Apache-2.0
//! The device-query frames and the per-device record of KISS-Announce §6.6 (ROADMAP R1.33 / R1.36,
//! CireSnave board 355 "D").
//!
//! A backend is a kernel runner; the **device** behind it is what limits what can be computed. So a
//! provider that enumerates devices describes each one with the normalised record below: the dtypes it
//! can store and move, the dtypes it can compute on, and the accuracy it guarantees per
//! `(op, math_fidelity, dtype)`. A kernel keeps declaring its own dtype set (KISS-Contract); the
//! effective support for a `(device, kernel)` pair is the intersection ([`effective_dtypes`]). KISS pins
//! the shape and the meaning, never a device's contents.
//!
//! ```text
//!   DVRQ  request   tag(4) correlation
//!   DVRS  response  tag(4) device_count(u32) device_record* correlation
//!   DVDC  decline   tag(4) decline_code(u32) correlation
//!   device_record   device_id_len(u32) device_id  target(token)  store_dtypes  compute_dtypes
//!                   accuracy_count(u32) accuracy_record*
//!   token           len(u8, 1..=255) ASCII bytes          dtype_set  count(u8) token*
//!   accuracy_record op(token) math_fidelity(u8) dtype(token) max_ulp(u32, u32::MAX = no bound)
//! ```

use crate::announce::{read_correlation, AnnounceDecline, CorrelationId, CORRELATION_ID_LEN};
use crate::structure_key::{DTYPES, RESERVED_DTYPES};

/// `DVRQ` as a u32-LE (wire bytes `44 56 52 51`, §2.4).
pub const DVRQ: u32 = 0x5152_5644;
/// `DVRS` as a u32-LE (wire bytes `44 56 52 53`).
pub const DVRS: u32 = 0x5352_5644;
/// `DVDC` as a u32-LE (wire bytes `44 56 44 43`).
pub const DVDC: u32 = 0x4344_5644;
/// The FEAT-axis capability bit `DEVICE_QUERY` (§7.2).
pub const FEAT_DEVICE_QUERY_BIT: u32 = 35;
/// The most devices one response may carry (§6.6-0004).
pub const MAX_DEVICES: u32 = 4096;
/// The longest opaque `device_id` (§6.6-0006).
pub const MAX_DEVICE_ID_LEN: u32 = 4096;
/// The most dtypes in one set (§6.6-0006); the Classify closed table has fewer.
pub const MAX_DTYPES_PER_SET: u8 = 64;
/// The most accuracy records in one device record (§6.6-0008).
pub const MAX_ACCURACY_RECORDS: u32 = 1 << 16;
/// `max_ulp` value meaning the device declares no bound for the key (§6.6-0008).
pub const NO_ULP_BOUND: u32 = u32::MAX;
/// `math_fidelity` bytes: the `MathFidelity` ordinals of the OpAttrs table (`bit-stable`, `reduced-mantissa-permitted`).
pub const FIDELITY_BIT_STABLE: u8 = 1;
pub const FIDELITY_REDUCED_MANTISSA: u8 = 2;

/// One accuracy guarantee of a device, keyed by `(op, math_fidelity, dtype)` (§6.6-0008).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccuracyRecord {
    pub op: String,
    pub math_fidelity: u8,
    pub dtype: String,
    pub max_ulp: u32,
}

/// The normalised per-device record (§6.6-0006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    pub device_id: Vec<u8>,
    pub target: String,
    pub store_dtypes: Vec<String>,
    pub compute_dtypes: Vec<String>,
    pub accuracy: Vec<AccuracyRecord>,
}

/// A device-query request (§6.6-0003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRequest {
    pub correlation: Option<CorrelationId>,
}

/// A device-query response (§6.6-0004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceResponse {
    pub devices: Vec<DeviceRecord>,
    pub correlation: Option<CorrelationId>,
}

/// A device-query decline (§6.6-0005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDecline {
    pub decline_code: u32,
    pub correlation: Option<CorrelationId>,
}

fn push_correlation(b: &mut Vec<u8>, c: &Option<CorrelationId>) {
    match c {
        None => b.push(0),
        Some(id) => {
            b.push(CORRELATION_ID_LEN);
            b.extend_from_slice(id);
        }
    }
}

fn push_token(b: &mut Vec<u8>, t: &str) {
    b.push(t.len() as u8);
    b.extend_from_slice(t.as_bytes());
}

fn push_set(b: &mut Vec<u8>, set: &[String]) {
    b.push(set.len() as u8);
    for t in set {
        push_token(b, t);
    }
}

impl DeviceRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = DVRQ.to_le_bytes().to_vec();
        push_correlation(&mut b, &self.correlation);
        b
    }
}

impl DeviceDecline {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = DVDC.to_le_bytes().to_vec();
        b.extend_from_slice(&self.decline_code.to_le_bytes());
        push_correlation(&mut b, &self.correlation);
        b
    }
}

impl DeviceRecord {
    fn encode_into(&self, b: &mut Vec<u8>) {
        b.extend_from_slice(&(self.device_id.len() as u32).to_le_bytes());
        b.extend_from_slice(&self.device_id);
        push_token(b, &self.target);
        push_set(b, &self.store_dtypes);
        push_set(b, &self.compute_dtypes);
        b.extend_from_slice(&(self.accuracy.len() as u32).to_le_bytes());
        for a in &self.accuracy {
            push_token(b, &a.op);
            b.push(a.math_fidelity);
            push_token(b, &a.dtype);
            b.extend_from_slice(&a.max_ulp.to_le_bytes());
        }
    }
}

impl DeviceResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = DVRS.to_le_bytes().to_vec();
        b.extend_from_slice(&(self.devices.len() as u32).to_le_bytes());
        for d in &self.devices {
            d.encode_into(&mut b);
        }
        push_correlation(&mut b, &self.correlation);
        b
    }
}

// ---- reading -------------------------------------------------------------

/// A bounds-checked cursor: every read is checked against the bytes supplied (§6.2-0015).
struct Cursor<'a> {
    b: &'a [u8],
    off: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize, region: &'static str) -> Result<&'a [u8], AnnounceDecline> {
        let end = self.off.checked_add(n).filter(|&e| e <= self.b.len());
        let end = end.ok_or(AnnounceDecline::Truncated { region })?;
        let s = &self.b[self.off..end];
        self.off = end;
        Ok(s)
    }
    fn u8(&mut self, region: &'static str) -> Result<u8, AnnounceDecline> {
        Ok(self.take(1, region)?[0])
    }
    fn u32(&mut self, region: &'static str) -> Result<u32, AnnounceDecline> {
        Ok(u32::from_le_bytes(self.take(4, region)?.try_into().expect("4 bytes")))
    }
    fn tag(&mut self, want: u32, region: &'static str) -> Result<(), AnnounceDecline> {
        let got = self.u32(region)?;
        if got == want { Ok(()) } else { Err(AnnounceDecline::BadTag { region, got }) }
    }
    fn correlation(&mut self) -> Result<Option<CorrelationId>, AnnounceDecline> {
        let (c, end) = read_correlation(self.b, self.off)?;
        self.off = end;
        Ok(c)
    }
    /// A token: a length byte in `[1, 255]` then that many bytes. In `semantic` mode the bytes must be
    /// printable ASCII with no space.
    fn token(&mut self, region: &'static str, semantic: bool) -> Result<String, AnnounceDecline> {
        let len = self.u8(region)?;
        if len == 0 {
            return Err(AnnounceDecline::TokenLenOutOfRange { region, got: 0 });
        }
        let bytes = self.take(len as usize, region)?;
        if semantic && !bytes.iter().all(|c| c.is_ascii_graphic()) {
            return Err(AnnounceDecline::TokenMalformed { region });
        }
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
    fn set(&mut self, region: &'static str, semantic: bool) -> Result<Vec<String>, AnnounceDecline> {
        let n = self.u8(region)?;
        if n > MAX_DTYPES_PER_SET {
            return Err(AnnounceDecline::SetCountOverflow { region, got: n });
        }
        (0..n).map(|_| self.token(region, semantic)).collect()
    }
}

fn check_dtype_token(t: &str, region: &'static str) -> Result<(), AnnounceDecline> {
    if RESERVED_DTYPES.contains(&t) {
        return Err(AnnounceDecline::DtypeReserved { region });
    }
    if !DTYPES.contains(&t) {
        return Err(AnnounceDecline::DtypeUnknown { region });
    }
    Ok(())
}

fn check_set(set: &[String], region: &'static str) -> Result<(), AnnounceDecline> {
    for t in set {
        check_dtype_token(t, region)?;
    }
    if !set.windows(2).all(|w| w[0].as_bytes() < w[1].as_bytes()) {
        return Err(AnnounceDecline::SetNotCanonical { region });
    }
    Ok(())
}

fn check_target(t: &str) -> Result<(), AnnounceDecline> {
    match t.split_once(':') {
        Some((ns, cap)) if !ns.is_empty() && !cap.is_empty() => Ok(()),
        _ => Err(AnnounceDecline::TokenMalformed { region: "target" }),
    }
}

/// The record-level validity checks of §6.6-0007 and §6.6-0008, run only when a record is read for use.
fn check_record(d: &DeviceRecord) -> Result<(), AnnounceDecline> {
    check_target(&d.target)?;
    check_set(&d.store_dtypes, "store_dtypes")?;
    check_set(&d.compute_dtypes, "compute_dtypes")?;
    if let Some(t) = d.compute_dtypes.iter().find(|t| !d.store_dtypes.contains(t)) {
        return Err(AnnounceDecline::ComputeNotInStore { dtype: t.clone() });
    }
    for a in &d.accuracy {
        if ![FIDELITY_BIT_STABLE, FIDELITY_REDUCED_MANTISSA].contains(&a.math_fidelity) {
            return Err(AnnounceDecline::BadMathFidelity { got: a.math_fidelity });
        }
        check_dtype_token(&a.dtype, "accuracy_dtype")?;
        if !d.compute_dtypes.contains(&a.dtype) {
            return Err(AnnounceDecline::AccuracyDtypeNotComputable { dtype: a.dtype.clone() });
        }
    }
    let key = |a: &AccuracyRecord| (a.op.clone().into_bytes(), a.math_fidelity, a.dtype.clone().into_bytes());
    if !d.accuracy.windows(2).all(|w| key(&w[0]) < key(&w[1])) {
        return Err(AnnounceDecline::AccuracyNotCanonical);
    }
    Ok(())
}

fn read_device(c: &mut Cursor, semantic: bool) -> Result<DeviceRecord, AnnounceDecline> {
    let id_len = c.u32("device_id_len")?;
    if !(1..=MAX_DEVICE_ID_LEN).contains(&id_len) {
        return Err(AnnounceDecline::DeviceIdLenOutOfRange { got: id_len });
    }
    let device_id = c.take(id_len as usize, "device_id")?.to_vec();
    let target = c.token("target", semantic)?;
    let store_dtypes = c.set("store_dtypes", semantic)?;
    let compute_dtypes = c.set("compute_dtypes", semantic)?;
    let n = c.u32("accuracy_count")?;
    if n > MAX_ACCURACY_RECORDS {
        return Err(AnnounceDecline::AccuracyCountOverflow { got: n });
    }
    let mut accuracy = Vec::new();
    for _ in 0..n {
        let op = c.token("accuracy_op", semantic)?;
        let math_fidelity = c.u8("math_fidelity")?;
        let dtype = c.token("accuracy_dtype", semantic)?;
        let max_ulp = c.u32("max_ulp")?;
        accuracy.push(AccuracyRecord { op, math_fidelity, dtype, max_ulp });
    }
    let rec = DeviceRecord { device_id, target, store_dtypes, compute_dtypes, accuracy };
    if semantic {
        check_record(&rec)?;
    }
    Ok(rec)
}

fn read_response(bytes: &[u8], semantic: bool) -> Result<(DeviceResponse, usize), AnnounceDecline> {
    let mut c = Cursor { b: bytes, off: 0 };
    c.tag(DVRS, "leading_tag")?;
    let n = c.u32("device_count")?;
    if n > MAX_DEVICES {
        return Err(AnnounceDecline::DeviceCountOverflow { got: n });
    }
    let mut devices = Vec::new();
    for _ in 0..n {
        devices.push(read_device(&mut c, semantic)?);
    }
    let correlation = c.correlation()?;
    Ok((DeviceResponse { devices, correlation }, c.off))
}

/// Decode and validate a device-query response: every malformation, and every record that fails the
/// §6.6-0007 / §6.6-0008 checks, is a typed decline, never a panic (§6.2-0008).
pub fn decode_response(bytes: &[u8]) -> Result<DeviceResponse, AnnounceDecline> {
    read_response(bytes, true).map(|(r, _)| r)
}

/// Decode a device-query request.
pub fn decode_request(bytes: &[u8]) -> Result<DeviceRequest, AnnounceDecline> {
    let mut c = Cursor { b: bytes, off: 0 };
    c.tag(DVRQ, "leading_tag")?;
    Ok(DeviceRequest { correlation: c.correlation()? })
}

/// Decode a device-query decline.
pub fn decode_decline(bytes: &[u8]) -> Result<DeviceDecline, AnnounceDecline> {
    let mut c = Cursor { b: bytes, off: 0 };
    c.tag(DVDC, "leading_tag")?;
    let decline_code = c.u32("decline_code")?;
    Ok(DeviceDecline { decline_code, correlation: c.correlation()? })
}

/// The extents of the three frames, from their own bytes only (§6.2-0014). Structural: record validity is
/// not checked here, only the lengths and counts that locate the end.
pub(crate) fn request_extent(bytes: &[u8]) -> Result<usize, AnnounceDecline> {
    let mut c = Cursor { b: bytes, off: 4 };
    c.correlation()?;
    Ok(c.off)
}
pub(crate) fn response_extent(bytes: &[u8]) -> Result<usize, AnnounceDecline> {
    read_response(bytes, false).map(|(_, n)| n)
}
pub(crate) fn decline_extent(bytes: &[u8]) -> Result<usize, AnnounceDecline> {
    let mut c = Cursor { b: bytes, off: 4 };
    c.u32("decline_code")?;
    c.correlation()?;
    Ok(c.off)
}

// ---- consumer side: effective support and accuracy (§6.6-0009, §6.6-0010) ------------

/// Why a `(device, kernel)` pairing is declined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingDecline {
    /// The kernel does not declare the dtype.
    NotOfferedByKernel { dtype: String },
    /// The device's record does not list the dtype as computable.
    NotComputedByDevice { dtype: String },
    /// The kernel's bound is looser than the device's floor under the same key.
    KernelLooserThanDeviceFloor { kernel: u32, device: u32 },
}

/// The dtypes a kernel can run on a device: the kernel's declared set intersected with the device's
/// compute set, ascending (§6.6-0009).
pub fn effective_dtypes(kernel: &[&str], device: &DeviceRecord) -> Vec<String> {
    let mut v: Vec<String> = kernel.iter().filter(|k| device.compute_dtypes.iter().any(|d| d == *k)).map(|k| k.to_string()).collect();
    v.sort();
    v.dedup();
    v
}

/// Admit a kernel for `dtype` on a device, or say which side lacks it (§6.6-0009).
pub fn admit_dtype(kernel: &[&str], dtype: &str, device: &DeviceRecord) -> Result<(), PairingDecline> {
    if !kernel.contains(&dtype) {
        return Err(PairingDecline::NotOfferedByKernel { dtype: dtype.to_string() });
    }
    if !device.compute_dtypes.iter().any(|d| d == dtype) {
        return Err(PairingDecline::NotComputedByDevice { dtype: dtype.to_string() });
    }
    Ok(())
}

/// The ULP bound in force for `(op, math_fidelity, dtype)` on a device running a kernel that declares
/// `kernel_ulp` (`None` = no bound declared): the tighter of the two, and a kernel looser than the
/// device's floor under the same key is declined (§6.6-0010). `Ok(None)` = neither side declares one.
pub fn effective_max_ulp(
    device: &DeviceRecord,
    op: &str,
    math_fidelity: u8,
    dtype: &str,
    kernel_ulp: Option<u32>,
) -> Result<Option<u32>, PairingDecline> {
    let floor = device
        .accuracy
        .iter()
        .find(|a| a.op == op && a.math_fidelity == math_fidelity && a.dtype == dtype)
        .map(|a| a.max_ulp)
        .filter(|&u| u != NO_ULP_BOUND);
    match (kernel_ulp, floor) {
        (Some(k), Some(d)) if k > d => Err(PairingDecline::KernelLooserThanDeviceFloor { kernel: k, device: d }),
        (Some(k), _) => Ok(Some(k)),
        (None, f) => Ok(f),
    }
}

/// The reference device record of the golden vector: a device that stores more than it computes on.
pub fn reference_device() -> DeviceRecord {
    let s = |v: &[&str]| v.iter().map(|t| t.to_string()).collect::<Vec<_>>();
    DeviceRecord {
        device_id: b"gpu0".to_vec(),
        target: "cuda:sm89".to_string(),
        store_dtypes: s(&["bf16", "f16", "f32", "i8"]),
        compute_dtypes: s(&["bf16", "f16", "f32"]),
        accuracy: vec![
            AccuracyRecord { op: "exp".into(), math_fidelity: FIDELITY_BIT_STABLE, dtype: "f32".into(), max_ulp: 1 },
            AccuracyRecord { op: "exp".into(), math_fidelity: FIDELITY_REDUCED_MANTISSA, dtype: "f32".into(), max_ulp: 4 },
        ],
    }
}

pub fn reference_dvrq() -> DeviceRequest {
    DeviceRequest { correlation: None }
}
pub fn reference_dvrs() -> DeviceResponse {
    DeviceResponse { devices: vec![reference_device()], correlation: None }
}
pub fn reference_dvdc() -> DeviceDecline {
    DeviceDecline { decline_code: crate::announce::decline_code::QUERY_NOT_SUPPORTED, correlation: None }
}
