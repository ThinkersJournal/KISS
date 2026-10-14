// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY §6.1-0002 / §6.1-0003 / §6.2-0001 — the dtype table's bit widths and numeric kinds.
//!
//! Three readings are held against each other, so a disagreement is DETECTABLE rather than assumed away:
//!   * the clause text's own enumerations (`f16`/`bf16` = 16; ...; `float`: `f16`, ...), parsed here;
//!   * `corpus/dtype_manifest.json`, the artifact consumers vendor (Fuel, Baracuda, kiss-ref), which
//!     `tools/kiss_dtypes.py` derives from the §6.1 table;
//!   * the reserved-token and kind constants in the reference crate (`structure_key::DTYPES`).
//! The spec is read at COMPILE time (`include_str!`), never at run time and never transcribed.
//!
//! Scope, stated: these tests pin the clauses' enumerated facts. They do not exercise any
//! implementation's storage; that is what a foreign reader would add (R4.1).

use kiss_conformance::json::{parse, Json};
use kiss_conformance::structure_key::DTYPES;
use std::collections::{BTreeMap, BTreeSet};

const SPEC: &str = include_str!("../../spec/classify.md");
const MANIFEST: &str = include_str!("../corpus/dtype_manifest.json");

/// The body of one clause, bounded by the next clause id (never a character window).
fn clause_body(id: &str) -> String {
    let spec = SPEC.replace("\r\n", "\n");
    let head = format!("**{id}**");
    let start = spec.find(&head).unwrap_or_else(|| panic!("spec lost clause {id}"));
    let rest = &spec[start + head.len()..];
    let end = rest.find("- **KISS-CLASSIFY-").unwrap_or(rest.len());
    rest[..end].split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every backticked word in `s`.
fn ticks(s: &str) -> Vec<String> {
    s.split('`').skip(1).step_by(2).map(str::to_string).collect()
}

/// `(f16/bf16 = 16; f32 = 32; ...)` -> token -> bits.
fn clause_bit_widths() -> BTreeMap<String, u64> {
    let body = clause_body("KISS-CLASSIFY-6.1-0002");
    let list = &body[body.find('(').expect("list opens")..body.find("*Test:*").expect("test marker")];
    let list = list.trim().trim_start_matches('(').trim_end_matches(|c: char| c == ')' || c == '.' || c == ' ');
    let mut out = BTreeMap::new();
    for seg in list.split(';') {
        let (l, r) = seg.split_once('=').unwrap_or_else(|| panic!("segment without '=': {seg}"));
        let bits: u64 = r.trim().parse().unwrap_or_else(|_| panic!("not a width: {r}"));
        for t in ticks(l) {
            assert!(out.insert(t.clone(), bits).is_none(), "`{t}` listed twice in 6.1-0002");
        }
    }
    out
}

/// `(`float`: `f16`, ...; `int`: ...)` -> token -> kind.
fn clause_kinds() -> BTreeMap<String, String> {
    let body = clause_body("KISS-CLASSIFY-6.1-0003");
    let from = body.find("(`float`").expect("kind list opens");
    let to = body[from..].find("). ").map(|i| i + from).expect("kind list closes");
    let mut out = BTreeMap::new();
    for seg in body[from + 1..to].split(';') {
        let (k, toks) = seg.split_once(':').unwrap_or_else(|| panic!("segment without ':': {seg}"));
        let kind = ticks(k).into_iter().next().expect("kind word");
        for t in ticks(toks) {
            assert!(out.insert(t.clone(), kind.clone()).is_none(), "`{t}` listed under two kinds");
        }
    }
    out
}

struct Row {
    kind: String,
    bits: u64,
}

fn manifest() -> BTreeMap<String, Row> {
    let doc = parse(MANIFEST).expect("manifest parses");
    let rows = doc.get("dtypes").and_then(Json::as_arr).expect("dtypes");
    rows.iter()
        .map(|r| {
            let tok = r.get("token").and_then(Json::as_str).expect("token").to_string();
            let kind = r.get("kind").and_then(Json::as_str).expect("kind").to_string();
            let bits = r.get("storage_bits").and_then(Json::as_u64).expect("storage_bits");
            (tok, Row { kind, bits })
        })
        .collect()
}

/// Disagreements between a clause's token->value map and the manifest's, both directions.
fn diff<V: PartialEq + std::fmt::Debug>(
    what: &str,
    clause: &BTreeMap<String, V>,
    manifest: &BTreeMap<String, V>,
) -> Vec<String> {
    let mut out = Vec::new();
    for (t, v) in clause {
        match manifest.get(t) {
            None => out.push(format!("{what}: clause names `{t}`, manifest has no such dtype")),
            Some(m) if m != v => out.push(format!("{what}: `{t}` clause {v:?} != manifest {m:?}")),
            Some(_) => {}
        }
    }
    for t in manifest.keys() {
        if !clause.contains_key(t) {
            out.push(format!("{what}: manifest dtype `{t}` is absent from the clause's list"));
        }
    }
    out
}

fn manifest_bits() -> BTreeMap<String, u64> {
    manifest().into_iter().map(|(t, r)| (t, r.bits)).collect()
}
fn manifest_kinds() -> BTreeMap<String, String> {
    manifest().into_iter().map(|(t, r)| (t, r.kind)).collect()
}

#[test]
fn test_classify_dtype_bit_widths() {
    let clause = clause_bit_widths();
    assert_eq!(clause.len(), 24, "the clause enumerates all 24 dtypes");
    let d = diff("bit width", &clause, &manifest_bits());
    assert!(d.is_empty(), "{d:#?}");
    // the reference crate's token list is the same 24
    let crate_set: BTreeSet<&str> = DTYPES.iter().copied().collect();
    let clause_set: BTreeSet<&str> = clause.keys().map(String::as_str).collect();
    assert_eq!(crate_set, clause_set, "structure_key::DTYPES vs the clause's enumeration");
    // a few anchors stated independently of the parse, so a parser that returns a plausible wrong map fails
    assert_eq!(clause["f16"], 16);
    assert_eq!(clause["c128"], 128);
    assert_eq!(clause["b1"], 1);
    assert_eq!(clause["bool"], clause["u8"], "bool is stored as a byte (6.1-0007)");
}

#[test]
fn test_classify_dtype_numeric_kinds() {
    let clause = clause_kinds();
    assert_eq!(clause.len(), 24, "the clause assigns a kind to all 24 dtypes");
    let d = diff("numeric kind", &clause, &manifest_kinds());
    assert!(d.is_empty(), "{d:#?}");
    // the two statements the clause makes in prose, checked against the parse
    assert_eq!(clause["f8e8m0"], "float", "the MX scale is kind float");
    assert_eq!(clause["f8e6m2"], "float", "the reserved f8e6m2 is kind float");
    assert_eq!(clause["b1"], "uint");
}

#[test]
fn test_classify_numeric_kind_set_closed() {
    let kinds: BTreeSet<String> = manifest_kinds().into_values().collect();
    let want: BTreeSet<String> = ["float", "int", "uint", "bool", "complex"].iter().map(|s| s.to_string()).collect();
    assert_eq!(kinds, want, "the kinds actually used are exactly the five");
    // the manifest's own declared kind list agrees, and the clause's kinds are a subset of it
    let doc = parse(MANIFEST).expect("manifest parses");
    let declared: BTreeSet<String> = doc
        .get("kinds")
        .and_then(Json::as_arr)
        .expect("kinds")
        .iter()
        .map(|k| k.as_str().expect("kind string").to_string())
        .collect();
    assert_eq!(declared, want, "manifest `kinds` is exactly the closed set");
    let used: BTreeSet<String> = clause_kinds().into_values().collect();
    assert_eq!(used, want, "the clause introduces no sixth kind and uses all five");
    // every dtype maps to exactly one kind: the clause parse refuses a duplicate, and covers all 24
    assert_eq!(clause_kinds().len(), DTYPES.len());
}

// ---- controls: the comparison must be able to go red ------------------------------------------

#[test]
fn control_a_wrong_width_a_missing_dtype_and_a_wrong_kind_are_all_caught() {
    let mut bits = clause_bit_widths();
    bits.insert("f32".into(), 64);
    assert!(!diff("w", &bits, &manifest_bits()).is_empty(), "a changed width must be reported");
    let mut bits = clause_bit_widths();
    bits.remove("b1");
    assert!(!diff("w", &bits, &manifest_bits()).is_empty(), "a dtype missing from the clause must be reported");
    let mut kinds = clause_kinds();
    kinds.insert("bool".into(), "uint".into());
    assert!(!diff("k", &kinds, &manifest_kinds()).is_empty(), "a changed kind must be reported");
    let mut kinds = clause_kinds();
    kinds.insert("zz9".into(), "float".into());
    assert!(!diff("k", &kinds, &manifest_kinds()).is_empty(), "an extra dtype must be reported");
}
