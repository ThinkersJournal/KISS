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

/// Every cell of the "How `entry_point` resolves" column, one per registered row.
fn resolution_cells() -> Vec<String> {
    let at = SYNTH.find("| Tag (ASCII) | Wire bytes |").expect("the 6.4-0001g table header");
    let header: Vec<&str> = SYNTH[at..].lines().next().unwrap().trim().trim_matches('|').split('|').map(str::trim).collect();
    let col = header.iter().position(|h| *h == "How `entry_point` resolves").expect("the resolution column");
    SYNTH[at..]
        .lines()
        .skip(2)
        .take_while(|l| l.trim_start().starts_with('|'))
        .map(|l| l.trim().trim_matches('|').split('|').map(str::trim).nth(col).unwrap_or("").to_string())
        .collect()
}

/// Every cell of the "ABI model" column, one per registered row.
fn abi_cells() -> Vec<String> {
    let at = SYNTH.find("| Tag (ASCII) | Wire bytes |").expect("the 6.4-0001g table header");
    let header: Vec<&str> = SYNTH[at..].lines().next().unwrap().trim().trim_matches('|').split('|').map(str::trim).collect();
    let col = header.iter().position(|h| *h == "ABI model").expect("the ABI model column");
    SYNTH[at..]
        .lines()
        .skip(2)
        .take_while(|l| l.trim_start().starts_with('|'))
        .map(|l| l.trim().trim_matches('|').split('|').map(str::trim).nth(col).unwrap_or("").to_string())
        .collect()
}

// Backs: KISS-SYNTH-6.4-0001g — every registered row states an ABI model drawn from the named set, and the only name used is pointer-abi (a row without one is not registered)
#[test]
fn test_synth_artifact_format_tag_abi_model() {
    let cells = abi_cells();
    assert_eq!(cells.len(), spec_rows().len(), "one ABI-model cell per registered row");
    for c in &cells {
        let name = c.trim_matches('`');
        assert!(!name.is_empty(), "a row that states no ABI model is not registered");
        assert!(["pointer-abi", "descriptor-set", "buffer-index", "host-closure"].contains(&name), "ABI model outside the named set: {c:?}");
    }
    assert_eq!(cells[0].trim_matches('`'), "pointer-abi", "the PTX row binds the Interface list as typed pointers and scalars");
}

/// KISS-SYNTH-6.4-0001h: a registered row states how `entry_point` resolves inside its artifact.
#[test]
fn test_synth_entry_point_resolves_in_artifact() {
    // Clause: KISS-SYNTH-6.4-0001h.
    let cells = resolution_cells();
    assert_eq!(cells.len(), spec_rows().len(), "one resolution cell per registered row");
    for c in &cells {
        assert!(c.starts_with("inside the artifact"), "a row's rule resolves inside the artifact: {c:?}");
    }
    assert!(cells[0].contains("`.entry`"), "the PTX row names the directive that carries the symbol");
}
