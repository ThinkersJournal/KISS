// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-SYNTH-6.4-0001g (R5.2 option A): KISS-Synth owns the `artifact_format_tag` values. The spec table is
//! read at COMPILE time (`include_str!`) and parsed here, so the reference constant is checked against the
//! document, not against itself.

use kiss_conformance::synth::{is_registered_format_tag, REGISTERED_ARTIFACT_FORMAT_TAGS};

const SYNTH: &str = include_str!("../../spec/synth.md");

/// The rows of the §6.4-0001g table: (ASCII cell, wire-bytes cell).
fn spec_rows() -> Vec<(String, String)> {
    let at = SYNTH.find("| Tag (ASCII) | Wire bytes |").expect("the 6.4-0001g table header");
    SYNTH[at..]
        .lines()
        .skip(2)
        .take_while(|l| l.trim_start().starts_with('|'))
        .map(|l| {
            let c: Vec<&str> = l.trim().trim_matches('|').split('|').map(str::trim).collect();
            (c[0].to_string(), c[1].to_string())
        })
        .collect()
}

fn parse_hex4(cell: &str) -> [u8; 4] {
    let bytes: Vec<u8> = cell
        .trim_matches('`')
        .split_whitespace()
        .map(|h| u8::from_str_radix(h, 16).expect("hex byte"))
        .collect();
    bytes.try_into().expect("exactly 4 wire bytes")
}

// Backs: KISS-SYNTH-6.4-0001g — the registered artifact_format_tag values are exactly the rows of the spec table (one seed value, PTX), each ASCII cell agrees with its wire bytes, none is all-zero, and an unlisted tag is not registered
#[test]
fn test_synth_artifact_format_tag_registry() {
    let rows = spec_rows();
    assert_eq!(rows.len(), 1, "seeded with exactly one value: {rows:?}");
    let spec: Vec<[u8; 4]> = rows.iter().map(|(_, hex)| parse_hex4(hex)).collect();
    assert_eq!(spec, REGISTERED_ARTIFACT_FORMAT_TAGS.to_vec(), "code and spec table list the same values");
    // the ASCII cell and the bytes cell describe the same four bytes (independent of the constant)
    let ascii = rows[0].0.trim_matches('`').split(' ').next().unwrap();
    assert_eq!(ascii, "PTX");
    assert_eq!(spec[0], [0x50, 0x54, 0x58, 0x20], "`PTX` plus one trailing space");
    for tag in REGISTERED_ARTIFACT_FORMAT_TAGS {
        assert_ne!(tag, [0; 4], "the reserved all-zero value is never registered (6.4-0001d)");
        assert!(is_registered_format_tag(tag));
    }
    // unlisted tags are not registered (cubin is mapped by one seam but emitted by no provider, so it is not assigned)
    for tag in [*b"CUBN", *b"SPRV", [0; 4], [0xFF; 4], *b"PTX\0", *b"ptx "] {
        assert!(!is_registered_format_tag(tag), "{tag:?} must not be registered");
    }
}
