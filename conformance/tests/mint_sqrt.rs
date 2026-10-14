// SPDX-License-Identifier: MIT OR Apache-2.0
//! `sqrt` correct-rounding vectors (§6.8-0003, first arm; the clause is NOT counted as backed). Every cell's expected value is
//! recomputed from an INDEPENDENT exact integer square root with explicit round-to-nearest-even (not
//! `f32::sqrt`, which `kiss_mint` uses), and the cells are checked to include the inputs on which a
//! "within 1-2 ULP" implementation is wrong. The clause's second arm (a target that does not guarantee
//! correct rounding meets its declared tier) is a property of the contract's declared tier, NOT of
//! these cells, and is not backed here.

use kiss_conformance::{corpus, DeterminismClass};

/// Integer floor square root of a u128.
fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros()).div_ceil(2));
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// Correctly rounded (RNE) sqrt of a finite non-negative f32, from the exact integer significand.
fn model(x: f32) -> f32 {
    if x == 0.0 || x.is_infinite() {
        return x;
    }
    let bits = x.to_bits();
    let exp = ((bits >> 23) & 0xFF) as i32;
    let man = (bits & 0x7F_FFFF) as u128;
    let (mut m, mut e) = if exp == 0 { (man, -149) } else { (man | (1 << 23), exp - 150) };
    if e % 2 != 0 {
        m <<= 1;
        e -= 1;
    }
    // choose k so isqrt(m << 2k) has 26 bits (24 + guard + 1)
    let mut k = 0u32;
    while isqrt(m << (2 * k)) < (1 << 25) {
        k += 1;
    }
    let scaled = m << (2 * k);
    let s = isqrt(scaled);
    assert!((1 << 25..1 << 26).contains(&s));
    let sticky = s * s != scaled;
    let (mut q, low2) = (s >> 2, s & 3);
    if low2 > 2 || (low2 == 2 && (sticky || q & 1 == 1)) {
        q += 1;
    }
    // value = q * 2^(2 + e/2 - k); q < 2^25 so the f64 product is exact
    let p = 2 + e / 2 - k as i32;
    (q as f64 * 2f64.powi(p)) as f32
}

// Deliberately NOT a backing test for the clause (§6.8-0003 has two arms and this pins one): the
// vector instrument is checked here, and the ledger row stays `untested` with a note saying so.
#[test]
fn sqrt_first_arm_cells_match_the_integer_model() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-sqrt-rounding.json"))
        .expect("bundle committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses through load_cell");
    assert_eq!(c.vectors.len(), 24);
    let (mut exact_squares, mut irrational, mut subnormals) = (0, 0, 0);
    for v in &c.vectors {
        let w = format!("sqrt tc {}", v.tc_id);
        assert_eq!(v.op, "sqrt", "{w}");
        assert_eq!(v.dtype, "f32", "{w}");
        assert_eq!(v.class, DeterminismClass::UlpTolerance, "{w}");
        assert_eq!(v.ulp_bound, 0, "{w}: the correctly-rounded arm is bound 0");
        let xb: [u8; 4] = v.inputs[0].clone().try_into().expect("4 bytes");
        let x = f32::from_bits(u32::from_be_bytes(xb));
        let got: [u8; 4] = v.expected.clone().try_into().expect("4 bytes");
        let want = model(x);
        assert_eq!(u32::from_be_bytes(got), want.to_bits(), "{w}: sqrt({x:?}) differs from the integer model {want:?}");
        let r = f32::from_bits(u32::from_be_bytes(got));
        // an exact square must come back exact (r*r recovers x in f64)
        if x > 0.0 && x.is_finite() && f64::from(r) * f64::from(r) == f64::from(x) {
            exact_squares += 1;
        }
        // an irrational result is not representable: the neighbour one ulp away must differ
        if x.to_bits() == 0x4000_0000 || x.to_bits() == 0x4040_0000 || x.to_bits() == 0x4049_0FDB {
            assert!(f64::from(r) * f64::from(r) != f64::from(x), "{w}: irrational");
            irrational += 1;
        }
        if x > 0.0 && x < f32::MIN_POSITIVE {
            subnormals += 1;
        }
    }
    assert!(exact_squares >= 4, "exact squares present: {exact_squares}");
    assert!(irrational >= 3, "irrational cases present");
    assert!(subnormals >= 3, "subnormal operands present");
}
