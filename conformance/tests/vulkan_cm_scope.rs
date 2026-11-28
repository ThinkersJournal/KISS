// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS vulkan namespace V-18 (Draft; PM ruling 2026-11-27, Q1 = C, Q2 = B; no stored byte moves): a
//! `cm-` tuple means a SUBGROUP-scope cooperative-matrix shape; a shape of any other scope is omitted
//! whole and reported in `unnamed_types` as `vulkan:scope:<n>`; the subgroup range is a kernel internal
//! and adds nothing to the key.
//!
//! RED FIRST: the V-18 assertions fail until the rule is in `spec/namespaces/vulkan.md`. The BYTES
//! assertions (`test_vulkan_cm_tokens_bytes_unchanged`) hold both before and after: they pin every
//! `vulkan:` token in the corpus and the Classify annex, and the tuple and subgroup grammar lines, to
//! the bytes at origin/main e2d7ef95. If one fails, the change moved a byte and is Breaking (vocab 9).

use std::fs::read_to_string;

fn read(rel: &str) -> String {
    read_to_string(format!("{}/../{rel}", env!("CARGO_MANIFEST_DIR"))).unwrap_or_else(|e| panic!("read {rel}: {e}")).replace("\r\n", "\n")
}

/// Every `vulkan:sg...` token spelled in `s` (a token ends at whitespace, backtick, `|`, quote, `;`).
fn tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find("vulkan:sg") {
        let tail = &rest[i..];
        let end = tail.find(|c: char| c.is_whitespace() || "`|\";".contains(c)).unwrap_or(tail.len());
        let t = &tail[..end];
        if t.contains('.') && !t.contains("...") {
            out.push(t.to_string());
        }
        rest = &tail[end.max(1)..];
    }
    out.sort();
    out.dedup();
    out
}

const GOLDEN: &[&str] = &[
    "vulkan:sg32.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32",
    "vulkan:sg64.ops-ab.arith-f16.cm-none.cv-none",
    "vulkan:sg64.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32",
    "vulkan:sg64.ops-abr.arith-f16.cm-none",
    "vulkan:sg64.ops-abr.arith-f16.cm-none.cv-none",
    "vulkan:sg64.ops-bar.arith-f16.cm-none.cv-none",
    "vulkan:sgdyn.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32",
    "vulkan:sgdyn.ops-none.arith-none.cm-none",
];

// harness-internal: golden of the existing vulkan tokens and grammar lines, a namespace-document rule (V-18) with no KISS clause id, no clause asserted
#[test]
fn test_vulkan_cm_tokens_bytes_unchanged() {
    let mut all = tokens(&read("conformance/corpus/structure_key_vectors.json"));
    all.extend(tokens(&read("spec/classify.md")));
    all.sort();
    all.dedup();
    let golden: Vec<String> = GOLDEN.iter().map(|s| s.to_string()).collect();
    assert_eq!(all, golden, "an existing vulkan token changed: Q1-C moves no byte (else Breaking, vocab 9)");
    let v = read("spec/namespaces/vulkan.md");
    // The tuple grammar and the subgroup table are the bytes of e2d7ef95: no scope coordinate, no range.
    assert!(v.contains("A tuple is `<M>-<N>-<K>-<A>-<B>-<C>-<R>` optionally suffixed `-sat` for"), "cm tuple grammar unchanged");
    assert!(v.contains("| `sg<N>` | pinned to width `N`, a power of two, no leading zeros |\n| `sgdyn` | width-agnostic |"), "subgroup table has exactly sg<N> and sgdyn (Q2-B)");
    assert!(!v.contains("sgmin") && !v.contains("sgrange"), "Q2-B: the subgroup range adds nothing to the key");
}

// harness-internal: asserts the V-18 rule of spec/namespaces/vulkan.md, a namespace-document rule with no KISS clause id
#[test]
fn test_vulkan_cm_subgroup_scope() {
    let v = read("spec/namespaces/vulkan.md");
    let start = v.find("- **V-18.**").expect("V-18 (cm- tuples are Subgroup scope) is in spec/namespaces/vulkan.md");
    let rest = &v[start + 1..];
    let end = rest.find("\n- **V-").map_or(v.len(), |i| start + 1 + i);
    let c = &v[start..end];
    for needle in [
        "Subgroup scope",
        "VK_SCOPE_SUBGROUP_KHR",
        "omitted whole",
        "unnamed_types",
        "vulkan:scope:",
        "no stored byte moves",
        "no vocabulary version bump",
        "no stored kernel key changes",
        "subgroup range",
        "kernel internal",
        "Draft",
    ] {
        assert!(c.contains(needle), "V-18 must state {needle}");
    }
}
