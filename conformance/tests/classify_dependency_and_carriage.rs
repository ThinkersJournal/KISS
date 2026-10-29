// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY §6.9 (the dependency and seam rules): 6.9-0001 no upstream dependency, 6.9-0002 the
//! `structure_key` token is carried as an opaque length-delimited byte string, 6.9-0003 producing and
//! parsing the vocabulary needs only the language's standard library.
//!
//! Subject and its limit, stated: 6.9-0001 and 6.9-0003 are checked as a SOURCE-LEVEL lint over the modules
//! the Classify clauses are exercised through (`CLASSIFY_TIER`). A source scan cannot prove an arbitrary
//! build graph; it proves these modules import nothing from the rest of the harness and load no driver, and
//! the controls prove the scan can fail. 6.9-0002 is a behavioural round trip through the Announce
//! availability list, which is the seam the clause names.

use kiss_conformance::announce::{decode_availability_list, AvailabilityList, AvailabilityRecord, LIST_VERSION};
use kiss_conformance::derivation_vectors::{expect_of, vectors, Expect};
use kiss_conformance::structure_key::from_token;

/// The modules through which the Classify clauses are tested, with their source baked in at compile time.
const CLASSIFY_TIER: [(&str, &str); 6] = [
    ("structure_key", include_str!("../src/structure_key.rs")),
    ("derive", include_str!("../src/derive.rs")),
    ("derivation_vectors", include_str!("../src/derivation_vectors.rs")),
    ("dtype", include_str!("../src/dtype.rs")),
    ("fp", include_str!("../src/fp.rs")),
    ("reference_vectors", include_str!("../src/reference_vectors.rs")),
];
const CARGO: &str = include_str!("../Cargo.toml");

/// Every `crate::<module>` a source file names, outside comments.
fn crate_imports(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        let code = line.split("//").next().unwrap_or("");
        let mut rest = code;
        while let Some(i) = rest.find("crate::") {
            let tail = &rest[i + "crate::".len()..];
            let name: String = tail.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                out.push(name);
            }
            rest = tail;
        }
    }
    out
}

/// Strings a driver/backend load would need, outside comments.
const DRIVER_LOADERS: [&str; 5] = ["extern \"C\"", "libloading", "dlopen", "LoadLibrary", "std::process::Command"];

fn driver_loads(src: &str) -> Vec<&'static str> {
    let code: String = src.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n");
    DRIVER_LOADERS.iter().copied().filter(|p| code.contains(p)).collect()
}

/// The non-comment, non-blank lines of the manifest's `[dependencies]` table.
fn dependencies(manifest: &str) -> Vec<String> {
    let mut in_deps = false;
    let mut out = Vec::new();
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_deps = t == "[dependencies]";
            continue;
        }
        if in_deps && !t.is_empty() && !t.starts_with('#') {
            out.push(t.to_string());
        }
    }
    out
}

// Backs[enforced]: KISS-CLASSIFY-6.9-0001 — the Classify modules depend on no KISS-Ops (or other sub-standard) module:
// every crate-internal import is within the Classify tier, so they build and test without any Ops artifact.
#[test]
fn test_classify_no_upstream_dependency() {
    let tier: Vec<&str> = CLASSIFY_TIER.iter().map(|(n, _)| *n).collect();
    let mut seen_any = 0;
    for (name, src) in CLASSIFY_TIER {
        for imp in crate_imports(src) {
            seen_any += 1;
            assert!(tier.contains(&imp.as_str()), "{name}.rs imports crate::{imp}, outside the Classify tier {tier:?}");
        }
    }
    assert!(seen_any >= 3, "the scan saw {seen_any} imports: a scanner that sees none proves nothing");
    assert!(dependencies(CARGO).is_empty(), "the harness crate declares external dependencies: {:?}", dependencies(CARGO));
}

// Backs[enforced]: KISS-CLASSIFY-6.9-0003 — producing, serializing and parsing the vocabulary needs only the standard
// library: no driver/backend loader in the Classify modules and no external crate dependency.
#[test]
fn test_classify_zero_dependency() {
    for (name, src) in CLASSIFY_TIER {
        assert!(driver_loads(src).is_empty(), "{name}.rs references a driver loader: {:?}", driver_loads(src));
    }
    assert!(dependencies(CARGO).is_empty(), "external crates: {:?}", dependencies(CARGO));
    // and the vocabulary really is produced and parsed with nothing else: every derivation vector yields a
    // token that parses back to itself
    let mut n = 0;
    for v in vectors() {
        if let Expect::Token(t) = expect_of(&v) {
            assert_eq!(from_token(&t).expect("parses").to_token(), t, "{}", v.name);
            n += 1;
        }
    }
    assert!(n >= 30, "round-tripped {n} tokens");
}

// Backs[enforced]: KISS-CLASSIFY-6.9-0002 — a structure_key token is carried across the announce seam as an opaque,
// length-delimited byte string (the carrier never parses it) and, returned byte-for-byte, is still parseable
// per §6.7: every derivation-vector token survives an availability-list encode/decode unchanged.
#[test]
fn test_classify_structure_key_opaque_carry() {
    let tokens: Vec<String> = vectors()
        .iter()
        .filter_map(|v| match expect_of(v) {
            Expect::Token(t) => Some(t),
            Expect::Decline(_) => None,
        })
        .collect();
    assert!(tokens.len() >= 30);
    let list = AvailabilityList {
        list_version: LIST_VERSION,
        records: tokens
            .iter()
            .enumerate()
            .map(|(i, t)| AvailabilityRecord { op_identity: b"add".to_vec(), structure_key: t.clone().into_bytes(), revision_hash: [i as u8; 32] })
            .collect(),
    };
    let back = decode_availability_list(&list.encode()).expect("a well-formed list decodes");
    assert_eq!(back.records.len(), tokens.len());
    for (r, t) in back.records.iter().zip(&tokens) {
        assert_eq!(r.structure_key, t.as_bytes(), "carried byte-for-byte");
        let s = std::str::from_utf8(&r.structure_key).expect("token is ASCII");
        assert_eq!(from_token(s).expect("parseable per 6.7 after the round trip").to_token(), *t);
    }
    // opaque: the carrier does not require the bytes to be a token at all
    let junk = AvailabilityList {
        list_version: LIST_VERSION,
        records: vec![AvailabilityRecord { op_identity: b"add".to_vec(), structure_key: vec![0xFF, 0x00, 0x7F, b'|'], revision_hash: [9; 32] }],
    };
    let jb = decode_availability_list(&junk.encode()).expect("opaque bytes carry");
    assert_eq!(jb.records[0].structure_key, [0xFF, 0x00, 0x7F, b'|']);
}

// ---- controls: the scans must be able to go red ------------------------------------------------

// harness-internal: negative control showing the import, driver-load and dependency source scanners flag what they should; checks the test machinery only.
#[test]
fn control_the_scans_flag_what_they_are_meant_to_flag() {
    assert_eq!(crate_imports("use crate::opattrs::encode_reduce; // x"), ["opattrs"]);
    assert_eq!(crate_imports("// use crate::opattrs;\nuse crate::derive::*;"), ["derive"], "comments are ignored");
    assert_eq!(driver_loads("let l = unsafe { libloading::Library::new(\"libcuda.so\") };"), ["libloading"]);
    assert!(driver_loads("// dlopen is only mentioned in prose\nfn f() {}").is_empty());
    assert_eq!(dependencies("[package]\nname = \"x\"\n[dependencies]\nserde = \"1\"\n[features]\n"), ["serde = \"1\""]);
    assert!(dependencies("[dependencies]\n# stdlib only\n\n[features]\n").is_empty());
    // the scan is not vacuous on the real tree: derive.rs does import the tier's structure_key
    assert!(crate_imports(CLASSIFY_TIER[1].1).contains(&"structure_key".to_string()));
}
