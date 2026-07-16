//! KISS-Conform vectors for the sub-byte / FP8 storage layouts (KISS-Classify §6.1).

use kiss_conformance::dtype::*;

// KISS-CLASSIFY-6.1-0008 — `test_classify_sub_byte_nibble_packing`:
// packed-pair byte, low nibble = EVEN logical index, high nibble = ODD; s4
// sign-extends on read, u4 zero-extends. Catches a high-nibble-first unpacker
// and an s4/u4 extend swap.
#[test]
fn test_classify_sub_byte_nibble_packing() {
    // Pair (even=-1, odd=+1): low nibble = (-1 & 0xF) = 0xF, high = 0x1 -> 0x1F.
    // A high-nibble-first packer would emit 0xF1 instead.
    assert_eq!(s4_pack(&[-1, 1]), vec![0x1F]);
    // Boundary pair (even=-8, odd=+7): low = (-8 & 0xF)=0x8, high=0x7 -> 0x78.
    assert_eq!(s4_pack(&[-8, 7]), vec![0x78]);

    // Read back: index 0 (even) MUST come from the LOW nibble. A high-first
    // unpacker would yield [1, -1] / [7, -8] here.
    assert_eq!(s4_unpack(&[0x1F], 2), vec![-1, 1]);
    assert_eq!(s4_unpack(&[0x78], 2), vec![-8, 7]);

    // The SAME byte 0x1F under u4 is [15, 1], NOT [-1, 1]: u4 zero-extends the
    // 0xF low nibble to 15 (catches u4 wrongly sign-extending).
    assert_eq!(u4_unpack(&[0x1F], 2), vec![15, 1]);
    assert_eq!(u4_pack(&[15, 1]), vec![0x1F]);
    // And s4 on that low nibble MUST be -1, not 15 (catches s4 zero-extending).
    assert_eq!(s4_sign_extend(0xF), -1);
    assert_eq!(u4_zero_extend(0xF), 15);
    assert_eq!(s4_sign_extend(0x8), -8);
    assert_eq!(s4_sign_extend(0x7), 7);

    // Roundtrip a multi-byte s4 run over the full [-8,7] domain.
    let vals: Vec<i8> = (-8..=7).collect(); // 16 values -> 8 bytes
    let packed = s4_pack(&vals);
    assert_eq!(packed.len(), 8);
    assert_eq!(s4_unpack(&packed, vals.len()), vals);

    // Roundtrip a u4 run over [0,15].
    let uvals: Vec<u8> = (0..=15).collect();
    let upacked = u4_pack(&uvals);
    assert_eq!(u4_unpack(&upacked, uvals.len()), uvals);
}

// KISS-CLASSIFY-6.1-0009 — `test_classify_b1_bit_packing`:
// packed-byte, 8 bits/byte, LSB = LOWEST logical index. Catches an MSB-first
// packer (e.g. numpy.packbits default).
#[test]
fn test_classify_b1_bit_packing() {
    // Only logical index 0 set -> LSB set -> 0x01. MSB-first would give 0x80.
    assert_eq!(b1_pack(&[1, 0, 0, 0, 0, 0, 0, 0]), vec![0x01]);
    // Only logical index 7 set -> MSB set -> 0x80. MSB-first would give 0x01.
    assert_eq!(b1_pack(&[0, 0, 0, 0, 0, 0, 0, 1]), vec![0x80]);
    // Indices 0 and 2 set -> bits 0 and 2 -> 0b0000_0101 = 0x05 (MSB-first: 0xA0).
    assert_eq!(b1_pack(&[1, 0, 1, 0, 0, 0, 0, 0]), vec![0x05]);

    // Unpack is the inverse: bit j of byte -> logical index j.
    assert_eq!(b1_unpack(&[0x01], 8), vec![1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(b1_unpack(&[0x80], 8), vec![0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(b1_unpack(&[0x05], 3), vec![1, 0, 1]);

    // Multi-byte: index 8 lands in the LSB of the SECOND byte (LSB-first per
    // byte, bytes in ascending logical order).
    let mut bits = vec![0u8; 12];
    bits[0] = 1; // byte0 bit0
    bits[8] = 1; // byte1 bit0
    assert_eq!(b1_pack(&bits), vec![0x01, 0x01]);
    assert_eq!(b1_unpack(&[0x01, 0x01], 12), bits);
}

// KISS-CLASSIFY-6.1-0010 — `test_classify_e4m3_format`:
// FP8 E4M3 (1s/4e/3m, bias 7), max finite ±448, NO infinity encodings, single
// NaN. Catches an IEEE-style mini-float reader that treats exp=1111 as inf/NaN
// (which would report max finite 240 and spurious infinities).
#[test]
fn test_classify_e4m3_format() {
    // Bias 7: S.0111.000 (0x38) = 2^(7-7) * 1.0 = 1.0.
    assert_eq!(e4m3_decode(0x38).value(), Some(1.0));

    // Max finite: S.1111.110 (0x7E) = 2^8 * 1.75 = 448 — a NORMAL finite number,
    // NOT NaN. An IEEE reader (exp=1111 => special) would call this NaN.
    assert_eq!(e4m3_decode(0x7E), E4M3::Normal(448.0));
    assert_eq!(e4m3_decode(0xFE), E4M3::Normal(-448.0));

    // exp=1111, mant=0 (0x78) is finite 2^8 = 256, NOT +inf. This is the exact
    // pattern an IEEE mini-float reader would decode as infinity.
    assert_eq!(e4m3_decode(0x78), E4M3::Normal(256.0));
    assert!(e4m3_decode(0x78).value().unwrap().is_finite());

    // The single NaN encoding is S.1111.111 (0x7F / 0xFF) — and ONLY that.
    assert!(e4m3_decode(0x7F).is_nan());
    assert!(e4m3_decode(0xFF).is_nan());
    assert_eq!(e4m3_decode(0x7F).value(), None);

    // Signed zero distinguished by bit pattern.
    assert_eq!(e4m3_decode(0x00), E4M3::Zero { negative: false });
    assert_eq!(e4m3_decode(0x80), E4M3::Zero { negative: true });

    // Smallest positive subnormal S.0000.001 (0x01) = (1/8) * 2^-6 = 2^-9.
    assert_eq!(e4m3_decode(0x01), E4M3::Subnormal(2.0f32.powi(-9)));
    // Smallest positive normal S.0001.000 (0x08) = 2^(1-7) = 2^-6 = 0.015625.
    assert_eq!(e4m3_decode(0x08), E4M3::Normal(0.015625));

    // Exhaustive format facts over all 256 encodings, computed from scratch:
    //  - max finite magnitude is EXACTLY 448 (an IEEE reader would find 240);
    //  - the NaN encodings are EXACTLY {0x7F, 0xFF} (single NaN mantissa);
    //  - NO byte decodes to an infinity (there is no Infinite variant, and every
    //    non-NaN value is finite).
    let mut max_finite = 0.0f32;
    let mut nan_bytes = Vec::new();
    for b in 0u16..=255 {
        let b = b as u8;
        match e4m3_decode(b).finite_magnitude() {
            Some(mag) => {
                assert!(mag.is_finite(), "byte {b:#04X} decoded to a non-finite value");
                if mag > max_finite {
                    max_finite = mag;
                }
            }
            None => nan_bytes.push(b),
        }
    }
    assert_eq!(max_finite, E4M3_MAX_FINITE);
    assert_eq!(max_finite, 448.0);
    assert_eq!(nan_bytes, vec![0x7F, 0xFF]);
}
