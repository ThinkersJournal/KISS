// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform tests for Contract 6.4-0005 (every op resolves to the primitive floor) and 6.4-0010 (a typed
//! decline and an unresolved/deferred outcome are distinct), over `corpus/op_decomposition.json`.

use kiss_conformance::op_resolve::{op_names, outcome_vectors, recorded, resolve, walk, Resolution};

// KISS-CONTRACT-6.4-0005
#[test]
fn test_contract_semantics_resolves_to_floor() {
    let names = op_names();
    assert!(names.len() >= 100, "table has {} ops", names.len());
    let mut non_primitive = 0;
    for op in &names {
        let (level, floor) = walk(op).unwrap_or_else(|| panic!("{op}: a reference names an op outside the table"));
        assert!(!floor.is_empty(), "{op}: resolution reached no floor op");
        assert_eq!(Some((level, floor)), recorded(op), "{op}: the table's level/floor disagree with the walk");
        non_primitive += usize::from(level > 0);
    }
    // control: the walk is not trivially level 0, and a known chain is exactly the §6.13 body
    assert!(non_primitive > 50, "only {non_primitive} non-primitive ops");
    assert_eq!(walk("matmul").map(|(l, _)| l), Some(1));
    assert_eq!(walk("avg_pool").map(|(l, f)| (l, f.into_iter().collect::<Vec<_>>())), Some((2, vec!["div".to_string(), "reduce".to_string()])));
}

// KISS-CONTRACT-6.4-0010
#[test]
fn test_contract_semantics_resolution_outcomes() {
    let vectors = outcome_vectors();
    let mut seen = (false, false, false);
    for (op, unavailable, want) in &vectors {
        assert_eq!(&resolve(op, unavailable), want, "{op} with {unavailable:?} unavailable");
        match want {
            Resolution::Resolved => seen.0 = true,
            Resolution::Deferred(_) => seen.1 = true,
            Resolution::Decline => seen.2 = true,
        }
    }
    assert_eq!(seen, (true, true, true), "the vectors must carry all three outcomes");
    // the two failures are different values: an unavailable contract is not a decline
    assert_ne!(resolve("matmul", &["matmul".into()]), resolve("frobnicate", &[]));
    // a floor op needs no contract; an unavailable contract elsewhere in the chain defers, never declines
    assert_eq!(resolve("add", &["add".into()]), Resolution::Resolved);
    assert_eq!(resolve("avg_pool", &["reduce_mean".into()]), Resolution::Deferred("reduce_mean".into()));
}
