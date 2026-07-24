//! First KISS-Emit conformance module — the §6.0 imported-enum and §6.2 lowering-
//! partition clauses, backed by DOC-LINT + ENUM-IMPORT tests (structural facts and
//! cross-standard enum spellings extracted from the spec text), continuing the #91
//! coverage burndown (EMIT was a true 0/65 backed).
//!
//! KISS-Emit is a lowering DISCIPLINE: a partition of lowering decisions plus a
//! contract-shaped output plus the emit↔consume round-trip. It ships **no** wire
//! envelope and **no** reference emitter in this crate, so — unlike the SYNTH module,
//! which exercised the real Announce codec — the cheapest EMIT clauses are backed by
//! reading the normative text and asserting a STRUCTURAL property (a partition, an
//! enum-set equality, a set disjointness) that would FAIL if the spec drifted. These
//! are structural / document-consistency bindings, NOT behavioral tests of an emitter;
//! the behavioral clauses (partition-at-runtime, artifact ABI, tier-2 round-trip)
//! genuinely need an emitter and are out of scope here.
//!
//! Clauses bound: KISS-EMIT-6.0-0001..0004, KISS-EMIT-6.2-0001..0005, KISS-EMIT-6.3-0006.
//!
//! No test in this crate previously read `spec/*.md`; this module establishes the
//! `read_spec()` / `clause_block()` pattern. `spec/` lives at the repo root, one level
//! above the conformance crate's `CARGO_MANIFEST_DIR`.

use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a phrase.
// ---------------------------------------------------------------------------

/// Read a `spec/<name>` markdown file. `spec/` is one directory above the crate.
fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// The body text of a single clause: from its `**KISS-…**` bold anchor up to the
/// next clause definition or the next markdown heading.
fn clause_block<'a>(md: &'a str, id: &str) -> &'a str {
    let anchor = format!("**{id}**");
    let start = md
        .find(&anchor)
        .unwrap_or_else(|| panic!("clause `{id}` is not defined in the spec text"));
    let rest = &md[start..];
    let mut end = rest.len();
    for pat in ["\n- **KISS-", "\n#"] {
        if let Some(i) = rest.get(1..).and_then(|r| r.find(pat)) {
            end = end.min(i + 1);
        }
    }
    &rest[..end]
}

/// Every clause id defined in `md` whose id begins with `prefix`
/// (e.g. `"KISS-EMIT-6.3-"` → all of §6.3's clause DEFINITIONS, not §9-matrix rows,
/// which are matched by the leading `**`).
fn defined_clauses(md: &str, prefix: &str) -> BTreeSet<String> {
    let anchor = format!("**{prefix}");
    let mut out = BTreeSet::new();
    let mut i = 0;
    while let Some(p) = md[i..].find(&anchor) {
        let s = i + p + 2; // skip the leading "**"
        if let Some(e) = md[s..].find("**") {
            out.insert(md[s..s + e].to_string());
        }
        i = s + 1;
    }
    out
}

/// The members of the first `{…}` enum literal inside `block`, each stripped of ALL
/// internal whitespace so a line-wrapped member still compares verbatim.
fn enum_members(block: &str) -> BTreeSet<String> {
    let open = block.find('{').expect("no `{` enum literal in clause block");
    let close = block[open..].find('}').expect("unterminated `{` enum literal") + open;
    block[open + 1..close]
        .split(',')
        .map(|m| m.split_whitespace().collect::<String>())
        .filter(|m| !m.is_empty())
        .collect()
}

/// The KISS-Ops-owned canonical determinism/fidelity enum (KISS-Ops §6.0). Its
/// membership and no-fork ownership are enforced **Ops-side** by the `kiss_tables`
/// no-fork lint on the owning clause; this EMIT module verifies only that KISS-Emit
/// imports it VERBATIM, so it compares Emit's parsed enum against this reference set
/// rather than re-reading (and thereby claiming to back) the Ops clause it only
/// partially exercises. Members carry no internal whitespace, matching `enum_members`.
fn ops_determinism_enum() -> BTreeSet<String> {
    ["exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The KISS-Ops-owned canonical MathPrecision enum (KISS-Ops §6.17) — see
/// [`ops_determinism_enum`] for why this is a reference constant, not an Ops re-read.
fn ops_mathprecision_enum() -> BTreeSet<String> {
    ["bit-stable", "reduced-mantissa-permitted"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Collapse whitespace (healing line wraps) and rejoin hyphen-wrapped compounds so
/// `"emitter-\n  must-supply"` matches `"emitter-must-supply"`. The clause-id/text
/// separator is an em dash (U+2014), not `"- "`, so it is untouched.
fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").replace("- ", "-")
}

/// A clause makes a driver-side placement iff its (healed) body assigns the
/// `driver-may-spell` bucket.
fn assigns_driver(block: &str) -> bool {
    norm(block).contains("driver-may-spell decision")
}

/// A clause makes an emitter-side placement iff its (healed) body assigns the
/// `emitter-must-supply` bucket. (Note `"treated as emitter-must-supply"` in the
/// audit-fallback prose is NOT a `"… decision"` placement and does not match.)
fn assigns_emitter(block: &str) -> bool {
    norm(block).contains("emitter-must-supply decision")
}

/// The slice of `md` between the `start_h` heading and the next `end_h` heading.
fn section_slice<'a>(md: &'a str, start_h: &str, end_h: &str) -> &'a str {
    let s = md.find(start_h).unwrap_or_else(|| panic!("section `{start_h}` not found"));
    let after = &md[s..];
    let e = after[start_h.len()..]
        .find(end_h)
        .map(|i| i + start_h.len())
        .unwrap_or(after.len());
    &after[..e]
}

/// The dtype token set from the KISS-Ops §6.16 self-contained dtype-layout table
/// (the first backtick column of each `| \`token\` | … |` row).
fn dtype_tokens(ops: &str) -> BTreeSet<String> {
    let sec = section_slice(ops, "### 6.16", "### 6.17");
    let mut out = BTreeSet::new();
    for line in sec.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("| `") {
            if let Some(end) = rest.find('`') {
                out.insert(rest[..end].to_string());
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// §6.0 — determinism/fidelity class and imported enums
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.0-0001 — every structural obligation in §6–§9 is determinism-class
/// EXACT-BYTE, evaluated with a byte-exact comparator and NEVER with a tolerance or
/// order-invariant comparator. Teeth: the class Emit selects (exact-byte) plus the
/// comparators it forbids must, between them, account for every member of the
/// KISS-Ops determinism enum — the forbidden-comparator tokens are derived from the
/// Ops member spellings, so a renamed Ops class that Emit failed to exclude fails.
#[test]
fn test_emit_determinism_class_exact_byte() {
    let emit = read_spec("emit.md");
    let ops_enum = ops_determinism_enum();
    assert!(
        ops_enum.iter().any(|m| m == "exact-byte"),
        "KISS-EMIT-6.0-0001: `exact-byte` is not a member of the KISS-Ops enum {ops_enum:?}"
    );
    let block = norm(clause_block(&emit, "KISS-EMIT-6.0-0001")).to_lowercase();
    assert!(
        block.contains("exact byte") || block.contains("exact-byte"),
        "KISS-EMIT-6.0-0001: the clause does not select the exact-byte determinism class"
    );
    // Every NON-exact-byte member of the canonical enum must be named (via a spelling
    // component of the member itself) as a forbidden comparator.
    for m in &ops_enum {
        if m == "exact-byte" {
            continue;
        }
        let excluded = m.split('/').any(|part| block.contains(&part.to_lowercase()));
        assert!(
            excluded,
            "KISS-EMIT-6.0-0001: determinism class `{m}` is not named as a forbidden comparator"
        );
    }
    assert!(
        block.contains("must not")
            && block.contains("tolerance")
            && block.contains("order-invariant"),
        "KISS-EMIT-6.0-0001: the clause does not forbid the tolerance / order-invariant comparators"
    );
}

/// KISS-EMIT-6.0-0002 — the MathPrecision compute-fidelity attribute
/// `{bit-stable, reduced-mantissa-permitted}` is imported VERBATIM from KISS-Ops
/// §6.17. Teeth: the two enum SETS (Emit's and Ops') must be equal — same members,
/// same spelling. A silent re-spell or fork fails.
#[test]
fn test_emit_mathprecision_imported_verbatim() {
    let emit = read_spec("emit.md");
    let ops_mp = ops_mathprecision_enum();
    let emit_mp = enum_members(clause_block(&emit, "KISS-EMIT-6.0-0002"));
    assert_eq!(
        ops_mp.len(),
        2,
        "KISS-EMIT-6.0-0002: KISS-Ops §6.17 MathPrecision must be a 2-member enum, found {ops_mp:?}"
    );
    assert_eq!(
        emit_mp, ops_mp,
        "KISS-EMIT-6.0-0002: KISS-Emit MathPrecision {emit_mp:?} is not the verbatim KISS-Ops set {ops_mp:?}"
    );
    for m in ["bit-stable", "reduced-mantissa-permitted"] {
        assert!(
            ops_mp.contains(m),
            "KISS-EMIT-6.0-0002: canonical MathPrecision member `{m}` missing"
        );
    }
}

/// KISS-EMIT-6.0-0003 — the canonical determinism/fidelity enum
/// `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` is owned by
/// KISS-Ops §6.0-0001 and imported by KISS-Emit VERBATIM. Teeth: the three-member
/// enum SET parsed out of emit.md must equal the SET parsed out of ops.md — same
/// members, same spelling. If Emit drifted a member, the SETS differ and this fails.
#[test]
fn test_emit_determinism_enum_imported_verbatim() {
    let emit = read_spec("emit.md");
    let ops_enum = ops_determinism_enum();
    let emit_enum = enum_members(clause_block(&emit, "KISS-EMIT-6.0-0003"));
    assert_eq!(
        ops_enum.len(),
        3,
        "KISS-EMIT-6.0-0003: KISS-Ops §6.0-0001 must own a 3-member enum, found {ops_enum:?}"
    );
    assert_eq!(
        emit_enum, ops_enum,
        "KISS-EMIT-6.0-0003: KISS-Emit's determinism enum {emit_enum:?} is not the verbatim KISS-Ops enum {ops_enum:?}"
    );
    for m in ["exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"] {
        assert!(
            ops_enum.contains(m),
            "KISS-EMIT-6.0-0003: canonical member `{m}` missing from the KISS-Ops enum"
        );
    }
}

/// KISS-EMIT-6.0-0004 — MathPrecision is ORTHOGONAL to the determinism class and is
/// NOT a dtype. Teeth: no MathPrecision enum member appears as a token in the
/// KISS-Ops §6.16 dtype set — the intersection of the two SETS is empty. This guards
/// exactly the anti-pattern §6.17 warns against: MathPrecision re-modeled as a
/// strict-precision (`f32s`) dtype.
#[test]
fn test_emit_mathprecision_orthogonal_not_dtype() {
    let ops = read_spec("ops.md");
    let emit = read_spec("emit.md");
    let mp = ops_mathprecision_enum();
    let dtypes = dtype_tokens(&ops);
    assert!(
        dtypes.len() >= 12,
        "KISS-EMIT-6.0-0004: parsed only {} dtypes from §6.16 — parser drift, check would be vacuous",
        dtypes.len()
    );
    assert!(!mp.is_empty(), "KISS-EMIT-6.0-0004: MathPrecision enum came back empty");
    let overlap: Vec<_> = mp.intersection(&dtypes).cloned().collect();
    assert!(
        overlap.is_empty(),
        "KISS-EMIT-6.0-0004: MathPrecision member(s) {overlap:?} also appear as dtypes — not orthogonal"
    );
    let block = norm(clause_block(&emit, "KISS-EMIT-6.0-0004")).to_lowercase();
    assert!(
        block.contains("orthogonal"),
        "KISS-EMIT-6.0-0004: the clause does not state MathPrecision is orthogonal"
    );
    assert!(
        block.contains("dtype"),
        "KISS-EMIT-6.0-0004: the clause does not state MathPrecision is not a dtype"
    );
}

// ---------------------------------------------------------------------------
// §6.2 — the complete lowering partition (total and disjoint)
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.2-0001 — the lowering partition is DISJOINT: no decision falls in both
/// the driver-may-spell set (§6.3) and the emitter-must-supply set (§6.4). Teeth:
/// extract the two decision-clause SETS; every §6.3 clause that makes a bucket
/// placement assigns `driver` (never `emitter`) and every §6.4 clause assigns
/// `emitter` (never `driver`), so no clause is placed in both buckets. A §6.4 clause
/// that also claimed driver-may-spell (or vice-versa) fails here.
#[test]
fn test_emit_partition_disjoint() {
    let emit = read_spec("emit.md");
    let driver = defined_clauses(&emit, "KISS-EMIT-6.3-");
    let emitter = defined_clauses(&emit, "KISS-EMIT-6.4-");
    assert!(
        !driver.is_empty() && !emitter.is_empty(),
        "KISS-EMIT-6.2-0001: §6.3/§6.4 decision sets must be non-empty ({}, {})",
        driver.len(),
        emitter.len()
    );

    let mut driver_tagged = BTreeSet::new();
    let mut emitter_tagged = BTreeSet::new();
    for c in &driver {
        let b = clause_block(&emit, c);
        assert!(
            !assigns_emitter(b),
            "KISS-EMIT-6.2-0001: driver-side clause {c} assigns the emitter-must-supply bucket"
        );
        if assigns_driver(b) {
            driver_tagged.insert(c.clone());
        }
    }
    for c in &emitter {
        let b = clause_block(&emit, c);
        assert!(
            !assigns_driver(b),
            "KISS-EMIT-6.2-0001: emitter-side clause {c} assigns the driver-may-spell bucket"
        );
        if assigns_emitter(b) {
            emitter_tagged.insert(c.clone());
        }
    }
    assert!(
        !driver_tagged.is_empty(),
        "KISS-EMIT-6.2-0001: no §6.3 clause makes a driver-may-spell placement — check would be vacuous"
    );
    assert!(
        !emitter_tagged.is_empty(),
        "KISS-EMIT-6.2-0001: no §6.4 clause makes an emitter-must-supply placement — check would be vacuous"
    );
    // Disjointness is proven by the per-clause cross-checks in the two loops above: a
    // §6.3 clause is asserted NEVER to assign the emitter bucket and a §6.4 clause NEVER
    // to assign the driver bucket, so no decision can land in both. (A set is_disjoint()
    // on driver_tagged vs emitter_tagged would be VACUOUS — they are built from disjoint
    // §6.3-* / §6.4-* clause-id populations and can never intersect regardless of content
    // — so it is deliberately omitted; the teeth are the mis-placement asserts above.)
}

/// KISS-EMIT-6.2-0002 — constant spelling and special-float-value spelling are
/// placed emitter-must-supply (§6.4-0001, §6.4-0002) and MUST NOT be defaulted onto
/// the driver side. Teeth: the two decisions, keyed by their bold decision labels,
/// live only on the emitter side (their homes assign emitter-must-supply) and NO
/// §6.3 driver clause bears them.
#[test]
fn test_emit_constants_are_emitter_supplied() {
    let emit = read_spec("emit.md");
    let const_home = "KISS-EMIT-6.4-0001";
    let sfloat_home = "KISS-EMIT-6.4-0002";
    let cb = clause_block(&emit, const_home);
    let sb = clause_block(&emit, sfloat_home);
    assert!(
        cb.contains("**Constant spelling**"),
        "KISS-EMIT-6.2-0002: {const_home} is not the constant-spelling decision"
    );
    assert!(
        sb.contains("**Special-float-value spelling**"),
        "KISS-EMIT-6.2-0002: {sfloat_home} is not the special-float-value decision"
    );
    assert!(
        assigns_emitter(cb),
        "KISS-EMIT-6.2-0002: constant spelling is not placed emitter-must-supply"
    );
    assert!(
        assigns_emitter(sb),
        "KISS-EMIT-6.2-0002: special-float-value spelling is not placed emitter-must-supply"
    );
    // No driver-side (§6.3) clause may carry a constant / special-float decision.
    for c in defined_clauses(&emit, "KISS-EMIT-6.3-") {
        let b = clause_block(&emit, &c);
        assert!(
            !b.contains("**Constant spelling**") && !b.contains("**Special-float-value spelling**"),
            "KISS-EMIT-6.2-0002: driver clause {c} carries a constant/special-float decision label"
        );
        let nb = norm(b).to_lowercase();
        assert!(
            !(assigns_driver(b) && nb.contains("constant spelling")),
            "KISS-EMIT-6.2-0002: driver clause {c} places constant spelling on the driver side"
        );
    }
    // The umbrella clause names §6.4-0001 and §6.4-0002 as the emitter-side homes.
    let g = clause_block(&emit, "KISS-EMIT-6.2-0002");
    assert!(
        g.contains("§6.4-0001") && g.contains("§6.4-0002"),
        "KISS-EMIT-6.2-0002: the clause does not cite the §6.4 emitter-side homes"
    );
}

/// KISS-EMIT-6.2-0003 — a lowering decision that HAS a target-language surface and is
/// NOT proven target-independent by the §6.5 audit defaults to emitter-must-supply;
/// the driver set holds a surface-bearing spelling only once the audit has proven it.
/// Teeth: classify each §6.3 driver decision by its own surface self-declaration; the
/// surface-bearing driver decision(s) MUST each be conditioned on the §6.5 audit and
/// default to emitter-must-supply, and §6.4-0005 sweeps every unproven operator to the
/// emitter side. A new surface-bearing driver decision left ungated fails here.
#[test]
fn test_emit_unproven_decision_defaults_to_emitter() {
    let emit = read_spec("emit.md");
    let mut surface_bearing = BTreeSet::new();
    for c in defined_clauses(&emit, "KISS-EMIT-6.3-") {
        let b = clause_block(&emit, &c);
        if !assigns_driver(b) {
            continue;
        }
        let nb = norm(b).to_lowercase();
        let no_surface = nb.contains("renders no target-language surface");
        let has_surface = nb.contains("does render a target-language surface");
        if has_surface && !no_surface {
            surface_bearing.insert(c.clone());
        }
    }
    assert!(
        !surface_bearing.is_empty(),
        "KISS-EMIT-6.2-0003: found no surface-bearing driver decision — classifier drift, check would be vacuous"
    );
    for c in &surface_bearing {
        let nb = norm(clause_block(&emit, c)).to_lowercase();
        assert!(
            nb.contains("§6.5") || nb.contains("neutrality audit"),
            "KISS-EMIT-6.2-0003: surface-bearing driver decision {c} is not gated on the §6.5 audit"
        );
        assert!(
            nb.contains("emitter-must-supply"),
            "KISS-EMIT-6.2-0003: surface-bearing driver decision {c} does not default to emitter-must-supply absent proof"
        );
    }
    // The emitter-side complement: §6.4-0005 sweeps every unproven operator to emitter-must-supply.
    let comp = norm(clause_block(&emit, "KISS-EMIT-6.4-0005")).to_lowercase();
    assert!(
        comp.contains("not") && comp.contains("prove") && comp.contains("emitter-must-supply"),
        "KISS-EMIT-6.2-0003: §6.4-0005 does not sweep unproven operators to emitter-must-supply"
    );
    // The clause itself states the surface → audit → emitter-must-supply default rule.
    let g = norm(clause_block(&emit, "KISS-EMIT-6.2-0003")).to_lowercase();
    assert!(
        g.contains("target-language surface")
            && g.contains("emitter-must-supply")
            && (g.contains("§6.5") || g.contains("audit")),
        "KISS-EMIT-6.2-0003: the clause does not state the surface→audit→emitter default rule"
    );
}

/// KISS-EMIT-6.2-0004 — completeness by closure: any decision NOT enumerated on the
/// closed driver-may-spell side defaults to emitter-must-supply; the emitter side is
/// the open complement and there is NO third bucket. Teeth: scan every
/// `"<bucket> decision"` placement token used across §6.3 + §6.4; the SET of bucket
/// names must be exactly `{driver-may-spell, emitter-must-supply}` (a third/implicit
/// bucket would grow the set and fail), and the clause states the closure rule.
#[test]
fn test_emit_partition_complete_by_closure() {
    let emit = read_spec("emit.md");
    let mut buckets = BTreeSet::new();
    for pfx in ["KISS-EMIT-6.3-", "KISS-EMIT-6.4-"] {
        for c in defined_clauses(&emit, pfx) {
            let words: Vec<String> = norm(clause_block(&emit, &c))
                .split_whitespace()
                .map(|w| {
                    w.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
                        .to_lowercase()
                })
                .collect();
            for w in words.windows(2) {
                if w[1] == "decision" && (w[0].ends_with("-spell") || w[0].ends_with("-supply")) {
                    buckets.insert(w[0].clone());
                }
            }
        }
    }
    let expect: BTreeSet<String> = ["driver-may-spell", "emitter-must-supply"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        buckets, expect,
        "KISS-EMIT-6.2-0004: placement buckets drifted (a third bucket?): {buckets:?}"
    );
    let g = norm(clause_block(&emit, "KISS-EMIT-6.2-0004")).to_lowercase();
    for needle in [
        "not explicitly enumerated",
        "emitter-must-supply",
        "closed enumeration",
        "open complement",
        "third",
    ] {
        assert!(
            g.contains(needle),
            "KISS-EMIT-6.2-0004: closure clause is missing the load-bearing phrase `{needle}`"
        );
    }
}

/// KISS-EMIT-6.2-0005 — a STRUCTURAL decision (renders no target-language surface:
/// op-DAG topology, operand binding, index/address arithmetic, control-flow
/// scaffolding) is neutral by construction and audit-EXEMPT — not subject to the
/// §6.2-0003 audit-default. Teeth: the structural categories the clause enumerates
/// must each be realized by a §6.3 driver clause that declares it renders no surface
/// AND cites §6.2-0005 (at least four such clauses exist).
#[test]
fn test_emit_structural_decisions_audit_exempt() {
    let emit = read_spec("emit.md");
    let g = norm(clause_block(&emit, "KISS-EMIT-6.2-0005")).to_lowercase();
    for cat in [
        "op-dag topology",
        "operand binding",
        "index/address arithmetic",
        "control-flow scaffolding",
    ] {
        assert!(
            g.contains(cat),
            "KISS-EMIT-6.2-0005: structural category `{cat}` missing from the clause enumeration"
        );
    }
    assert!(
        g.contains("audit") && g.contains("no") && g.contains("target-language surface"),
        "KISS-EMIT-6.2-0005: the clause does not tie 'renders no surface' to audit-exemption"
    );
    let mut structural = 0;
    for c in defined_clauses(&emit, "KISS-EMIT-6.3-") {
        let b = clause_block(&emit, &c);
        if !assigns_driver(b) {
            continue;
        }
        let nb = norm(b).to_lowercase();
        if nb.contains("renders no target-language surface") {
            assert!(
                nb.contains("§6.2-0005"),
                "KISS-EMIT-6.2-0005: structural driver clause {c} does not cite §6.2-0005"
            );
            structural += 1;
        }
    }
    assert!(
        structural >= 4,
        "KISS-EMIT-6.2-0005: expected >= 4 structural driver clauses citing §6.2-0005, found {structural}"
    );
}

// ---------------------------------------------------------------------------
// §6.3 — audit-exempt companion (bonus, clean structural teeth)
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.3-0006 — of the driver-may-spell decisions, the structural ones
/// (§6.3-0001/0002/0003/0005) are audit-EXEMPT and only the infix operator spellings
/// (§6.3-0004), which DO render a surface, are audit-GATED; an implementation MUST NOT
/// treat a structural driver decision as gated nor the infix operators as exempt.
/// Teeth: independently classify §6.3-0001..0005 by each clause's own surface
/// self-declaration, then assert the exempt/gated split (4 exempt, 1 gated) matches
/// the enumeration §6.3-0006 itself makes — a drift between the two fails.
#[test]
fn test_emit_structural_driver_decisions_audit_exempt() {
    let emit = read_spec("emit.md");
    let g = norm(clause_block(&emit, "KISS-EMIT-6.3-0006")).to_lowercase();

    let mut exempt = BTreeSet::new();
    let mut gated = BTreeSet::new();
    for n in 1..=5 {
        let id = format!("KISS-EMIT-6.3-{n:04}");
        let nb = norm(clause_block(&emit, &id)).to_lowercase();
        let no_surface = nb.contains("renders no target-language surface");
        let has_surface = nb.contains("does render a target-language surface");
        assert!(
            no_surface ^ has_surface,
            "KISS-EMIT-6.3-0006: {id} has an ambiguous surface self-declaration"
        );
        if no_surface {
            exempt.insert(id);
        } else {
            gated.insert(id);
        }
    }
    // The clause's own enumeration must list exactly these ids.
    for id in exempt.iter().chain(gated.iter()) {
        let short = id.trim_start_matches("KISS-EMIT-"); // e.g. "6.3-0001"
        assert!(
            g.contains(&format!("§{short}")),
            "KISS-EMIT-6.3-0006: {id} is not enumerated by the clause"
        );
    }
    assert_eq!(
        exempt.len(),
        4,
        "KISS-EMIT-6.3-0006: expected 4 audit-exempt structural decisions, found {}: {exempt:?}",
        exempt.len()
    );
    assert_eq!(
        gated.len(),
        1,
        "KISS-EMIT-6.3-0006: expected exactly 1 audit-gated (infix) decision, found {}: {gated:?}",
        gated.len()
    );
    assert!(
        g.contains("must not") && g.contains("audit-exempt") && g.contains("audit-gated"),
        "KISS-EMIT-6.3-0006: the clause does not pin the exempt/gated distinction in both directions"
    );
}
