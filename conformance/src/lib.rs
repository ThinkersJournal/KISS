//! KISS reference conformance harness — core.
//!
//! Non-normative. This crate is *a* conformant implementation of the KISS
//! specification with no privilege and no exemption (KISS-Conform §0). The
//! normative source is the specification text under `spec/`; where this code and
//! a spec clause disagree, the clause governs.
//!
//! This first slice implements two of the four KISS-Conform test modalities for
//! the KISS-Ops OpAttrs canonical wire encoding (Ops §6.19):
//!
//!   * **Golden byte-vectors** (Conform §6.4): a reference encoder produces the
//!     exact little-endian bytes pinned in Ops Appendix E, compared byte-for-byte.
//!   * **Negative / decline vectors** (Conform §6.7): a reader rejects a
//!     malformed blob with a typed decline, never a panic.
//!
//! selected by the **exact-byte** determinism-class comparator (Conform §6.8),
//! because every field of a POD-tier encoding is determinism-class exact-byte.

pub mod opattrs;

/// A determinism/fidelity class (KISS-Ops §6.0-0001) that selects a comparator
/// (KISS-Conform §6.8). Only `ExactByte` is exercised in this slice; the POD
/// tiers (Classify, Ops encodings) are exact-byte throughout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismClass {
    ExactByte,
    UlpTolerance,
    OrderInvariant,
}

/// Compare `actual` against `expected` under a determinism class (Conform §6.8).
pub fn compare(class: DeterminismClass, actual: &[u8], expected: &[u8]) -> Result<(), String> {
    match class {
        DeterminismClass::ExactByte => {
            if actual == expected {
                Ok(())
            } else {
                Err(format!(
                    "byte mismatch under exact-byte comparator\n    expected: {}\n    actual:   {}",
                    hex(expected),
                    hex(actual)
                ))
            }
        }
        other => Err(format!("comparator for {other:?} is not part of this slice")),
    }
}

/// Render bytes as space-separated uppercase hex — the spec's "bytes on the wire,
/// left to right" convention (Ops Appendix E).
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse a spec golden-vector hex string into bytes. Any non-hex separator
/// (spaces, and the `·` field-grouping mark used in Ops Appendix E) is ignored,
/// so a vector may be transcribed exactly as printed.
pub fn parse_hex(s: &str) -> Vec<u8> {
    let digits: Vec<char> = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    assert!(digits.len() % 2 == 0, "hex string has an odd number of digits: {s:?}");
    digits
        .chunks(2)
        .map(|pair| u8::from_str_radix(&pair.iter().collect::<String>(), 16).unwrap())
        .collect()
}

/// Assert a reference-encoder output matches a spec golden vector byte-for-byte,
/// citing the pinning clause in any failure. This is one golden-vector check of
/// KISS-Conform modality 1 (§6.4) under the exact-byte comparator (§6.8).
#[track_caller]
pub fn assert_golden(clause: &str, name: &str, actual: &[u8], golden_hex: &str) {
    let expected = parse_hex(golden_hex);
    if let Err(e) = compare(DeterminismClass::ExactByte, actual, &expected) {
        panic!("[{clause}] golden vector `{name}` FAILED\n    {e}");
    }
}

#[cfg(test)]
mod core_tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        assert_eq!(parse_hex("01 FF FF 01"), vec![0x01, 0xFF, 0xFF, 0x01]);
        assert_eq!(hex(&[0x00, 0x02, 0x01, 0x02]), "00 02 01 02");
        // the `··` grouping marks used in Appendix E are ignored
        assert_eq!(parse_hex("02·03 00 00 00·03 00 00 00").len(), 9);
    }

    #[test]
    fn exact_byte_comparator() {
        assert!(compare(DeterminismClass::ExactByte, &[1, 2], &[1, 2]).is_ok());
        assert!(compare(DeterminismClass::ExactByte, &[1, 2], &[1, 3]).is_err());
    }
}
