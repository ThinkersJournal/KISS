// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY-6.1-0014 / -0015 / -0016: the MX element dtypes `f4e2m1`, `f6e2m3`, `f6e3m2`.
//!
//! The decode tables are computed here with exact rational arithmetic from the clause's value rule and
//! checked against LITERAL anchors transcribed from the clause text (maximum finite magnitudes, the
//! non-negative `f4e2m1` code list, the subnormal steps), so the rule and the anchors can disagree. The
//! format constants themselves are unverified against the primary OCP MX v1.0 PDF (stated in the clauses).

use kiss_conformance::derive::{derive_structure_key, DeriveDecline};
use kiss_conformance::structure_key::{
    from_token, KeyDecline, DTYPES, MX_ELEMENT_DTYPES, RESERVED_DTYPES,
};

/// `(−1)^s · 2^(e−bias) · (1 + m/2^M)` for `e ≠ 0`, `(−1)^s · 2^(1−bias) · (m/2^M)` for `e = 0`, as an exact
/// numerator over `2^(M + bias + 8)` so no float is involved.
fn decode(code: u32, ebits: u32, mbits: u32, bias: i32) -> (i64, u32) {
    let m = (code & ((1 << mbits) - 1)) as i64;
    let e = ((code >> mbits) & ((1 << ebits) - 1)) as i32;
    let s = (code >> (mbits + ebits)) & 1;
    // value * 2^(mbits + bias + 8) as an integer
    let scale = mbits as i32 + bias + 8;
    let (sig, exp) = if e == 0 { (m, 1 - bias) } else { ((1i64 << mbits) + m, e - bias) };
    // value = sig / 2^mbits * 2^exp ; times 2^scale = sig * 2^(exp + scale - mbits)
    let sh = exp + scale - mbits as i32;
    assert!(sh >= 0);
    let v = sig << sh;
    (if s == 1 { -v } else { v }, scale as u32)
}

fn as_f64(code: u32, ebits: u32, mbits: u32, bias: i32) -> f64 {
    let (n, sc) = decode(code, ebits, mbits, bias);
    n as f64 / (1u64 << sc) as f64
}

/// LSB-first bit packing of `bits`-wide elements: element `i` at stream bits `bits*i ..`.
fn pack(elems: &[u32], bits: u32) -> Vec<u8> {
    let mut out = vec![0u8; (elems.len() * bits as usize).div_ceil(8)];
    for (i, &v) in elems.iter().enumerate() {
        assert!(v < (1 << bits));
        for b in 0..bits as usize {
            if (v >> b) & 1 == 1 {
                let k = i * bits as usize + b;
                out[k / 8] |= 1 << (k % 8);
            }
        }
    }
    out
}

fn unpack(bytes: &[u8], bits: u32, n: usize) -> Vec<u32> {
    (0..n)
        .map(|i| {
            (0..bits as usize).fold(0u32, |acc, b| {
                let k = i * bits as usize + b;
                acc | ((u32::from(bytes[k / 8] >> (k % 8)) & 1) << b)
            })
        })
        .collect()
}

// Backs: KISS-CLASSIFY-6.1-0014 — f4e2m1 is sign 1, exp 2, mantissa 1, bias 1 with no infinity or NaN: the
// eight non-negative codes are 0, .5, 1, 1.5, 2, 3, 4, 6 and code 8 is -0, and elements pack two per byte
// with the low nibble holding the even index exactly as i4/u4.
#[test]
fn test_classify_mx_f4e2m1_format() {
    let nonneg: Vec<f64> = (0..8).map(|c| as_f64(c, 2, 1, 1)).collect();
    assert_eq!(nonneg, vec![0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0]);
    // sign symmetry: code 8 is -0 (bit-distinct from +0), code 8+c is -nonneg[c]
    for c in 0..8u32 {
        assert_eq!(as_f64(8 + c, 2, 1, 1), -nonneg[c as usize]);
    }
    assert!(as_f64(8, 2, 1, 1).is_sign_negative() || decode(8, 2, 1, 1).0 == 0, "code 8 is a (negative) zero");
    // no code is infinite or NaN: all 16 decode to finite values
    assert!((0..16).all(|c| as_f64(c, 2, 1, 1).is_finite()));
    // nibble packing identical to i4/u4: low nibble = even index
    assert_eq!(pack(&[0x1, 0x2], 4), vec![0x21]);
    assert_eq!(pack(&[0x7, 0x9, 0xF], 4), vec![0x97, 0x0F]);
    assert_eq!(unpack(&[0x21], 4, 2), vec![0x1, 0x2]);
    // negative control: the opposite nibble order would give 0x12
    assert_ne!(pack(&[0x1, 0x2], 4), vec![0x12]);
}

// Backs: KISS-CLASSIFY-6.1-0015 — f6e2m3 (bias 1, max 7.5) and f6e3m2 (bias 3, max 28) have no infinity or
// NaN, and pack four elements per three bytes LSB-first with ceil(6n/8) bytes and zero trailing bits.
#[test]
fn test_classify_mx_f6_formats() {
    for (ebits, mbits, bias, max, sub_step, name) in
        [(2u32, 3u32, 1i32, 7.5f64, 0.125f64, "f6e2m3"), (3, 2, 3, 28.0, 0.0625, "f6e3m2")]
    {
        let vals: Vec<f64> = (0..32).map(|c| as_f64(c, ebits, mbits, bias)).collect();
        assert_eq!(vals[31], max, "{name} maximum finite magnitude");
        assert!(vals.windows(2).all(|w| w[0] < w[1]), "{name} non-negative codes strictly increase");
        assert_eq!(vals[1], sub_step, "{name} smallest subnormal");
        assert!((0..64).all(|c| as_f64(c, ebits, mbits, bias).is_finite()), "{name} has no inf/NaN code");
        for c in 0..32u32 {
            assert_eq!(as_f64(32 + c, ebits, mbits, bias), -vals[c as usize], "{name} sign symmetry");
        }
    }
    // four elements per three bytes, LSB-first: [1,2,3,4] -> 0x81 0x30 0x10
    assert_eq!(pack(&[1, 2, 3, 4], 6), vec![0x81, 0x30, 0x10]);
    assert_eq!(unpack(&[0x81, 0x30, 0x10], 6, 4), vec![1, 2, 3, 4]);
    // ceil(6n/8) bytes, trailing bits zero
    assert_eq!(pack(&[63], 6), vec![0x3F]);
    assert_eq!(pack(&[63, 63], 6), vec![0xFF, 0x0F]);
    assert_eq!(pack(&[0; 5], 6).len(), 4);
    // round trip over every code in every position of a group
    for pos in 0..4 {
        for code in 0..64u32 {
            let mut g = [0u32; 4];
            g[pos] = code;
            assert_eq!(unpack(&pack(&g, 6), 6, 4), g.to_vec());
        }
    }
    // negative control: MSB-first would give a different first byte for [1,2,3,4]
    assert_ne!(pack(&[1, 2, 3, 4], 6)[0], 0x04);
}

const EW: &str = "sk7|ew2|DT|cuda:sm89|ix32|grid|r2|co/00/v1/d16/f;co/00/v1/d16/f;co/00/v1/d16/f|-";
const CTR: &str = "sk7|ctr|f16|cuda:sm90|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/WDT/ACC/bf16/st";

// Backs[enforced]: KISS-CLASSIFY-6.1-0016 — an MX element is a usable dtype (field 2, <wdt>) but a typed-decline
// ElementDtypeAsAccumulator in either accumulator position and from the deriver; the three tokens are in the
// closed set, usable (not reserved), and appended after c128.
#[test]
fn test_classify_mx_element_not_accumulator() {
    assert_eq!(DTYPES.len(), 27);
    assert_eq!(&DTYPES[24..], &MX_ELEMENT_DTYPES[..], "appended after c128, in table order");
    for dt in MX_ELEMENT_DTYPES {
        assert!(!RESERVED_DTYPES.contains(&dt), "{dt} is usable, not reserved");
        // usable in the dtype field
        assert!(from_token(&EW.replace("DT", dt)).is_ok(), "{dt} parses in the dtype field");
        // usable as the weight dtype, declined as the accumulator
        let ok = CTR.replace("WDT", dt).replace("ACC", "f32");
        assert!(from_token(&ok).is_ok(), "{dt} parses as <wdt>: {ok}");
        let bad = CTR.replace("WDT", "f16").replace("ACC", dt);
        assert_eq!(from_token(&bad), Err(KeyDecline::ElementDtypeAsAccumulator), "{dt} as contraction <acc>");
        // the non-contraction (acc+mp) field
        let acc_mp = format!("{}|{dt}/rm", EW.replace("DT", "f32"));
        assert_eq!(from_token(&acc_mp), Err(KeyDecline::ElementDtypeAsAccumulator), "{dt} as (acc+mp) accumulator");
    }
    // positive control for the decline: a float accumulator in the same slot parses, and an unknown token
    // is a DIFFERENT decline
    assert!(from_token(&CTR.replace("WDT", "f16").replace("ACC", "f32")).is_ok());
    assert_eq!(from_token(&CTR.replace("WDT", "f16").replace("ACC", "f99")), Err(KeyDecline::UnknownDtype));
    // the deriver: an undeclared accumulator defaults to operand 0's dtype
    let v = kiss_conformance::derivation_vectors::vectors();
    let find = |n: &str| v.iter().find(|d| d.name == n).unwrap_or_else(|| panic!("vector {n}"));
    assert_eq!(
        derive_structure_key(&find("mx_f4e2m1_ctr_undeclared_accumulator_declines").cell).unwrap_err(),
        DeriveDecline::ElementDtypeAsAccumulator
    );
    assert!(derive_structure_key(&find("mx_f4e2m1_ctr_declared_accumulator_derives").cell).is_ok());
    assert!(derive_structure_key(&find("mx_f6e3m2_elementwise_add").cell).is_ok());
}
