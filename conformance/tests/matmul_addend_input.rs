// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-OPS-6.13-0012 (Draft; CireSnave 2026-11-27, board 521 "B", KISS#771 Q3): a `matmul`
//! MAY carry one ordinary `[M,N]` addend as its third input, `input(2)`; nothing is added to the
//! structure key, there is no `c` role hint, and `alpha` / `beta` stay ordinary scalar ops.
//!
//! Written RED FIRST: it fails until the clause is in `spec/ops.md`.

fn ops_md() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/ops.md")).expect("read spec/ops.md")
}

/// The clause's own paragraph: from its id to the next clause id.
fn clause_0012(spec: &str) -> &str {
    let start = spec
        .find("**KISS-OPS-6.13-0012**")
        .expect("KISS-OPS-6.13-0012 is in spec/ops.md (the addend clause)");
    let rest = &spec[start + 1..];
    let end = rest.find("- **KISS-OPS-").map_or(spec.len(), |i| start + 1 + i);
    &spec[start..end]
}

#[test]
fn test_ops_matmul_addend_input() {
    let spec = ops_md();
    let c = clause_0012(&spec);
    for needle in ["`input(2)`", "`[M,N]`", "ordinary", "no role hint", "contraction field", "unchanged", "`alpha`", "`beta`", "board 521"] {
        assert!(c.contains(needle), "KISS-OPS-6.13-0012 must state {needle}");
    }
    // alpha and beta are NOT fields: the clause defines no such field.
    assert!(c.contains("no `alpha` or `beta` field"), "alpha and beta stay ordinary scalar ops");
    // Draft, not Breaking.
    assert!(c.contains("not Breaking"), "a Draft change, not Breaking");
}
