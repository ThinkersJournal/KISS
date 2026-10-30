// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.8-0002b: the ULP distance is the difference of sign-magnitude total-order keys.
//! The failing vector is the one that shipped twice in Fuel: raw bit-pattern subtraction reports
//! 2^31 between `+0.0` and `-0.0`, which are adjacent.

use kiss_conformance::{ulp_distance_f32, ulp_key};

fn d(width: u32, a: u64, b: u64) -> u64 {
    ulp_key(width, a).abs_diff(ulp_key(width, b))
}

/// The wrong comparator §6.8-0002b rejects: subtract the raw sign-magnitude patterns.
fn raw_distance_f32(a: f32, b: f32) -> u64 {
    (a.to_bits() as u64).abs_diff(b.to_bits() as u64)
}

/// KISS-CONFORM-6.8-0002b. Sign-boundary goldens at f32, and the same keys at f16, bf16 and f64.
#[test]
fn test_conform_ulp_distance_function() {
    // Clause: KISS-CONFORM-6.8-0002b.
    // Zero: -0.0 and +0.0 are one apart, and the raw comparator is wrong by 2^31.
    assert_eq!(ulp_distance_f32(-0.0, 0.0), 1);
    assert_eq!(raw_distance_f32(-0.0, 0.0), 1 << 31, "the trap: raw subtraction, not the clause");

    // Subnormals straddling zero: -min, -0, +0, +min are three steps.
    let min = f32::from_bits(1);
    assert_eq!(ulp_distance_f32(-min, min), 3);
    // Subnormal -> normal, and largest finite -> +inf, are single steps.
    assert_eq!(ulp_distance_f32(f32::from_bits(0x007F_FFFF), f32::MIN_POSITIVE), 1);
    assert_eq!(ulp_distance_f32(f32::MAX, f32::INFINITY), 1);
    // Across zero at +-1.0: 0xBF800000 - 0x407FFFFF.
    assert_eq!(ulp_distance_f32(-1.0, 1.0), 0x7F00_0001);
    // The generic key agrees with the f32 comparator on every one of these patterns.
    for (a, b) in [(-0.0f32, 0.0f32), (-min, min), (-1.0, 1.0), (f32::MAX, f32::INFINITY), (1.0, 1.5)] {
        assert_eq!(d(32, a.to_bits() as u64, b.to_bits() as u64), ulp_distance_f32(a, b));
    }

    // Other widths, hand-computed: f16 1.0 = 0x3C00, -1.0 = 0xBC00; bf16 1.0 = 0x3F80, -1.0 = 0xBF80;
    // f64 +-0 and +-min subnormal.
    assert_eq!(d(16, 0x8000, 0x0000), 1);
    assert_eq!(d(16, 0xBC00, 0x3C00), 0x7801);
    assert_eq!(d(16, 0xBF80, 0x3F80), 0x7F01);
    assert_eq!(d(64, 0x8000_0000_0000_0000, 0), 1);
    assert_eq!(d(64, 0x8000_0000_0000_0001, 1), 3);
}
