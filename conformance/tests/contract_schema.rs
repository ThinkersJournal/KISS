//! Third KISS-Contract conformance module (sibling to `contract_golden.rs` and
//! `contract_framing.rs`), continuing the #91 coverage burndown. It mirrors the
//! already-accepted §6.8-0001 / §6.9-0001 doc-lint style — a frozen hardcoded
//! ordered list is only an ANCHOR; the load-bearing teeth are always a genuine
//! SECOND within-CONTRACT surface (a sibling clause's field/enum list, a
//! transcribed Appendix C golden, or a cross-clause single-home / disjointness /
//! cross-presence invariant), never a hardcoded self-copy.
//!
//!   (A) SECTION FIELD-SCHEMA + GUARANTEE CROSS-CLAUSE DOC-LINTS. Every lint stays
//!       WITHIN KISS-Contract, except the two enum lints, which compare against a
//!       reference CONSTANT token-set (never read from KISS-Ops / KISS-Classify BY
//!       CLAUSE-ID, so they neither reverse-bind nor over-credit an upstream
//!       owner) — identical discipline to the accepted §6.8-0003 determinism lint.
//!
//!   (B) ONE BEHAVIORAL back via the in-tree §6.6-0006 acceptor
//!       `dispatch_expr::eval` (the real crate codec, not a re-implementation):
//!       each of the five Dispatch field values is a §6.6-0006 expression string —
//!       a grammatical one is accepted, free prose declines.
//!
//! Clauses bound: KISS-CONTRACT-6.6-0001, -6.4-0001, -6.5-0001, -6.7-0001,
//! -6.7-0004, -6.8-0004, -6.8-0007, -6.8-0006, -6.9-0006, -6.9-0003, -6.11-0008.
//!
//! The only golden transcribed here (Appendix C Semantics block) is copied verbatim
//! from Contract Appendix C, identical to the copy in `contract_golden.rs`
//! (integration test files compile as separate crates and cannot share a `const`).

use kiss_conformance::dispatch_expr::{eval, Decline, EvalMode, LaunchScalars};
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Transcribed golden (Contract Appendix C; LF terminators).
// ---------------------------------------------------------------------------

/// Appendix C Semantics block (§6.11-0004/-0007; heading id 2): a one-node
/// `machine-checkable-IR` DAG. `human_annotation` is absent (its line is omitted),
/// so the golden carries exactly the two required Semantics fields.
const SEMANTICS_GOLDEN: &str = "\
[section:2:semantics]
semantics_kind = machine-checkable-IR
op_dag = [Op{add; ; []}]
";

/// The five Dispatch derivation fields, in the §6.11-0005 field-schema order. This
/// is only the frozen ANCHOR (exactly as the accepted §6.8-0001 lint hardcodes its
/// `expected`); the load-bearing teeth are the agreement of THREE independent spec
/// surfaces below.
const DISPATCH_FIELDS: [&str; 5] = [
    "invocation_domain",
    "workgroup_sizing",
    "count_to_grid",
    "thread_mapping",
    "addressing_rule",
];

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a phrase. (Re-declared per
// the established convention; each integration crate carries its own copy.)
// ---------------------------------------------------------------------------

/// Read a `spec/<name>` markdown file.
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

/// The ORDERED field list inside the first `{…}` literal of `block`, each field
/// stripped of all internal whitespace so a line-wrapped list still compares
/// verbatim. Order is preserved (field-schema order is normative, §6.11-0005). A
/// trailing `?` (optional marker) is preserved.
fn field_list_ordered(block: &str) -> Vec<String> {
    let open = block.find('{').expect("no `{…}` field list in clause block");
    let close = block[open..].find('}').expect("unterminated `{…}` field list") + open;
    block[open + 1..close]
        .split(',')
        .map(|m| m.split_whitespace().collect::<String>())
        .filter(|m| !m.is_empty())
        .collect()
}

/// The members of the first `{…}` enum literal as an unordered set (an enum has no
/// ordering obligation, unlike a field schema).
fn enum_set(block: &str) -> BTreeSet<String> {
    field_list_ordered(block).into_iter().collect()
}

/// Every all-ASCII-lowercase backtick token of `block`, in document order.
/// Underscored / hyphenated / mixed-case tokens (`cost_provenance`, the `*Test:*`
/// name) drop out, leaving only bare lowercase value tokens like `declared`.
fn backtick_lower_words(block: &str) -> Vec<String> {
    block
        .split('`')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, s)| s.to_string())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase()))
        .collect()
}

/// Every snake-case backtick token of `block` matching `^[a-z][a-z0-9_]*$`, in
/// document order. Unlike `backtick_lower_words` this KEEPS underscored field names
/// (`invocation_domain`, …) while still rejecting bracketed / operator tokens
/// (`sym[k]`, `+ - * /`, `ceil_div(a, b)`).
fn backtick_snake(block: &str) -> Vec<String> {
    block
        .split('`')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, s)| s.to_string())
        .filter(|s| {
            let mut cs = s.chars();
            matches!(cs.next(), Some(c) if c.is_ascii_lowercase())
                && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
        .collect()
}

/// The `<key>` of every `<key> = <value>` field line in a rendered block/golden,
/// in document order (the heading line, carrying no ` = `, is skipped).
fn field_keys(rendered: &str) -> Vec<String> {
    rendered
        .lines()
        .filter(|l| l.contains(" = "))
        .map(|l| l.split(" = ").next().unwrap().trim().to_string())
        .collect()
}

/// Collapse runs of whitespace to single spaces, healing markdown line wraps so a
/// parenthetical split across a wrapped line still parses as one group.
fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The snake-case backtick tokens inside the FIRST `(…)` group after the phrase
/// `derivation field` — the Dispatch field list as spelled in §6.6-0006 and
/// §6.11-0008 (`(`invocation_domain`, `workgroup_sizing`, …)`), one field per
/// backtick span. Distinct surface from §6.6-0001's `{…}` brace list.
fn dispatch_fields_in_parens(block: &str) -> Vec<String> {
    let b = norm(block);
    let a = b
        .find("derivation field")
        .expect("clause does not carry the `derivation field (…)` list");
    let rest = &b[a..];
    let open = rest.find('(').expect("no `(…)` field list after `derivation field`");
    let close = rest[open..].find(')').expect("unterminated `(…)` field list") + open;
    backtick_snake(&rest[open + 1..close])
}

/// The single canonical KISS-Ops determinism/fidelity enum, as a reference
/// CONSTANT — deliberately NOT a re-read of KISS-Ops by clause-id (that would
/// reverse-bind and over-credit the upstream owner clause). Identical to the copy
/// the accepted §6.8-0003 lint uses.
fn ops_determinism_enum() -> BTreeSet<String> {
    ["exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The KISS-Ops MathPrecision attribute set, as a reference CONSTANT — same
/// no-reverse-bind discipline as `ops_determinism_enum`.
fn ops_math_precision_enum() -> BTreeSet<String> {
    ["bit-stable", "reduced-mantissa-permitted"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

// ===========================================================================
// (A) Section field-schema + guarantee cross-clause doc-lints.
// ===========================================================================

/// KISS-CONTRACT-6.6-0001 — a kernel that declares launch geometry MUST carry the
/// five Dispatch fields `{invocation_domain, workgroup_sizing, count_to_grid,
/// thread_mapping, addressing_rule}` in that order. THREE independent within-CONTRACT
/// surfaces are ordered-compared: the §6.6-0001 `{…}` brace list, the §6.6-0006
/// `(…)` field list, and the §6.11-0008 `(…)` field list. The hardcoded
/// `DISPATCH_FIELDS` is only the anchor.
///
/// TEETH: rename/reorder/drop a field on ANY of the three surfaces but not the
/// others → the three ordered lists disagree and the test fails. No hardcoded
/// self-copy is load-bearing (each of the three is an independent spec surface).
#[test]
fn test_contract_dispatch_field_schema() {
    let contract = read_spec("contract.md");
    let s_6601 = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.6-0001"));
    let s_6606 = dispatch_fields_in_parens(clause_block(&contract, "KISS-CONTRACT-6.6-0006"));
    let s_1108 = dispatch_fields_in_parens(clause_block(&contract, "KISS-CONTRACT-6.11-0008"));

    // Each surface carries exactly the five fields (cardinality anchor).
    assert_eq!(s_6601.len(), 5, "KISS-CONTRACT-6.6-0001: §6.6-0001 must list exactly 5 Dispatch fields, got {s_6601:?}");
    assert_eq!(s_6606.len(), 5, "KISS-CONTRACT-6.6-0001: §6.6-0006 must list exactly 5 Dispatch fields, got {s_6606:?}");
    assert_eq!(s_1108.len(), 5, "KISS-CONTRACT-6.6-0001: §6.11-0008 must list exactly 5 Dispatch fields, got {s_1108:?}");

    // The load-bearing teeth: all three spec surfaces agree, in order.
    assert_eq!(s_6601, s_6606, "KISS-CONTRACT-6.6-0001: the §6.6-0001 and §6.6-0006 Dispatch field lists drifted (rename/reorder/drop)");
    assert_eq!(s_6601, s_1108, "KISS-CONTRACT-6.6-0001: the §6.6-0001 and §6.11-0008 Dispatch field lists drifted (rename/reorder/drop)");
    // The anchor pins the concrete order/spelling.
    assert_eq!(s_6601, DISPATCH_FIELDS, "KISS-CONTRACT-6.6-0001: the Dispatch field list drifted from the pinned 5-field ordered schema");
}

/// KISS-CONTRACT-6.4-0001 — the Semantics section carries the ordered schema
/// `{semantics_kind, op_dag, human_annotation?}`: the two REQUIRED fields
/// (`semantics_kind`, `op_dag`) serialized first, then an OPTIONAL
/// `human_annotation` pinned LAST. The §6.4-0001 spec field list (spec-prose
/// surface) is cross-checked against the transcribed Appendix C Semantics golden
/// keys (golden-byte surface), which carries exactly the two required fields.
///
/// TEETH: make `op_dag` optional (add `?`) → the required list drops it and no
/// longer equals the two golden keys; drop the `human_annotation` `?` → the
/// optional/last assertion fails; reorder the two required fields → the ordered
/// required list ≠ the golden keys.
#[test]
fn test_contract_semantics_mandatory() {
    let contract = read_spec("contract.md");
    let fields = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.4-0001"));
    assert_eq!(fields.len(), 3, "KISS-CONTRACT-6.4-0001: §6.4-0001 must list exactly 3 Semantics fields, got {fields:?}");

    let required: Vec<String> = fields.iter().filter(|f| !f.ends_with('?')).cloned().collect();
    let optional: Vec<String> = fields.iter().filter(|f| f.ends_with('?')).cloned().collect();
    let golden = field_keys(SEMANTICS_GOLDEN);

    // Required Semantics fields (the `?` stripped) equal the Appendix C golden keys,
    // in order — spec-prose surface vs golden-byte surface.
    assert_eq!(
        required, golden,
        "KISS-CONTRACT-6.4-0001: the §6.4-0001 required Semantics fields {required:?} ≠ the Appendix C golden keys {golden:?}"
    );
    assert_eq!(required, vec!["semantics_kind", "op_dag"], "KISS-CONTRACT-6.4-0001: the two required Semantics fields drifted");

    // `human_annotation` is the SOLE optional field and is pinned LAST.
    assert_eq!(optional, vec!["human_annotation?"], "KISS-CONTRACT-6.4-0001: `human_annotation?` must be the only optional Semantics field");
    assert!(fields.last().unwrap().ends_with('?'), "KISS-CONTRACT-6.4-0001: the optional field must be pinned LAST");

    // `op_dag` / `semantics_kind` are NOT optional (no trailing `?`).
    assert!(fields.iter().any(|f| f == "op_dag"), "KISS-CONTRACT-6.4-0001: `op_dag` must be a required (non-optional) field");
    assert!(fields.iter().any(|f| f == "semantics_kind"), "KISS-CONTRACT-6.4-0001: `semantics_kind` must be a required (non-optional) field");
}

/// KISS-CONTRACT-6.5-0001 — the Interface section carries exactly the seven fields
/// `{entry_point, rank, positional_signature, launch_scalars, count_unit, in_place,
/// alignment_bytes}` in order, and MUST NOT carry `count_unit` / `in_place` /
/// `alignment_bytes` in Capabilities nor an independent `target` field. LOAD-BEARING
/// cross-clause invariant against §6.7-0001: `{count_unit, in_place, alignment_bytes}`
/// ⊆ Interface AND ∩ Capabilities(§6.7-0001) = ∅, and `target` ∉ Interface.
///
/// TEETH: migrate `count_unit` into the §6.7-0001 Capabilities list → the
/// disjointness assert fails; add a bare `target` field to Interface → the exclusion
/// assert fails; a rename/reorder within §6.5-0001 fails the ordered anchor.
#[test]
fn test_contract_interface_field_schema() {
    let contract = read_spec("contract.md");
    let iface = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.5-0001"));
    let caps = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.7-0001"));

    let expected = [
        "entry_point",
        "rank",
        "positional_signature",
        "launch_scalars",
        "count_unit",
        "in_place",
        "alignment_bytes",
    ];
    assert_eq!(iface, expected, "KISS-CONTRACT-6.5-0001: the Interface field list drifted from the pinned 7-field ordered schema");

    // The three Interface-only fields live in Interface and are FORBIDDEN in Capabilities.
    for f in ["count_unit", "in_place", "alignment_bytes"] {
        assert!(iface.iter().any(|x| x == f), "KISS-CONTRACT-6.5-0001: `{f}` must be an Interface field");
        assert!(
            !caps.iter().any(|x| x == f),
            "KISS-CONTRACT-6.5-0001: `{f}` leaked into the Capabilities (§6.7-0001) field list — it MUST live only in Interface"
        );
    }
    // No independent `target` field in Interface (the target is Identity `target_capability`).
    assert!(!iface.iter().any(|f| f == "target"), "KISS-CONTRACT-6.5-0001: Interface MUST NOT carry an independent `target` field");
}

/// KISS-CONTRACT-6.7-0001 — the Capabilities section carries exactly the eight
/// fields `{accept_predicate, supported_dtype_set, awkward_layout_strategy,
/// in_place_eligible_variants, index_width, determinism_class, precision_class,
/// cost}` in order. LOAD-BEARING cross-clause invariants: `{count_unit, in_place,
/// alignment_bytes}` ∩ Capabilities = ∅ (§6.5-0001 prohibition), `accept_predicate`
/// is also an Identity field (§6.3-0001), and `determinism_class` is also a
/// Guarantees field (§6.8-0001, the §6.7-0004 equality precondition).
///
/// TEETH: drop `determinism_class` from Capabilities → the §6.8-0001 cross-presence
/// invariant becomes unsatisfiable; leak `in_place` into Capabilities → the
/// disjointness assert fails; a rename/reorder fails the ordered anchor.
#[test]
fn test_contract_capabilities_field_schema() {
    let contract = read_spec("contract.md");
    let caps = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.7-0001"));
    let expected = [
        "accept_predicate",
        "supported_dtype_set",
        "awkward_layout_strategy",
        "in_place_eligible_variants",
        "index_width",
        "determinism_class",
        "precision_class",
        "cost",
    ];
    assert_eq!(caps, expected, "KISS-CONTRACT-6.7-0001: the Capabilities field list drifted from the pinned 8-field ordered schema");

    // Disjoint from the three Interface-only fields (§6.5-0001).
    let iface = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.5-0001"));
    for f in ["count_unit", "in_place", "alignment_bytes"] {
        assert!(iface.iter().any(|x| x == f), "KISS-CONTRACT-6.7-0001: `{f}` must be an Interface field (§6.5-0001)");
        assert!(!caps.iter().any(|x| x == f), "KISS-CONTRACT-6.7-0001: `{f}` MUST NOT appear in Capabilities");
    }

    // Cross-presence preconditions of the §6.7-0002/§6.7-0004 equality obligations:
    // `accept_predicate` is also an Identity field, `determinism_class` a Guarantees field.
    let identity = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.3-0001"));
    let guarantees = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.8-0001"));
    assert!(
        caps.iter().any(|f| f == "accept_predicate") && identity.iter().any(|f| f == "accept_predicate"),
        "KISS-CONTRACT-6.7-0001: `accept_predicate` must be present in BOTH Capabilities and Identity (§6.3-0001) for the §6.7-0002 equality"
    );
    assert!(
        caps.iter().any(|f| f == "determinism_class") && guarantees.iter().any(|f| f == "determinism_class"),
        "KISS-CONTRACT-6.7-0001: `determinism_class` must be present in BOTH Capabilities and Guarantees (§6.8-0001) for the §6.7-0004 equality"
    );
}

/// KISS-CONTRACT-6.7-0004 — the Capabilities `determinism_class` is the single
/// canonical enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`
/// spelled verbatim AND identical to the Guarantees §6.8-0003 enum (§6.7-0004
/// requires equality). The §6.7-0004 enum is compared against a reference CONSTANT
/// and against the §6.8-0003 enum — both KISS-Contract's own surfaces, never a
/// KISS-Ops clause-read.
///
/// TEETH: re-spell a member in §6.7-0004 (`exact_byte` / `ulp-tolerance`) → the
/// parsed set differs from the canonical constant AND from the §6.8-0003 set.
#[test]
fn test_contract_capabilities_determinism_from_ops() {
    let contract = read_spec("contract.md");
    let caps_enum = enum_set(clause_block(&contract, "KISS-CONTRACT-6.7-0004"));
    let guarantees_enum = enum_set(clause_block(&contract, "KISS-CONTRACT-6.8-0003"));
    let canonical = ops_determinism_enum();

    assert_eq!(canonical.len(), 3, "KISS-CONTRACT-6.7-0004: the canonical determinism enum must have 3 members");
    assert_eq!(
        caps_enum, canonical,
        "KISS-CONTRACT-6.7-0004: §6.7-0004's determinism enum {caps_enum:?} is not the verbatim canonical set {canonical:?} (a member was re-forked/re-spelled)"
    );
    assert_eq!(
        caps_enum, guarantees_enum,
        "KISS-CONTRACT-6.7-0004: the Capabilities (§6.7-0004) and Guarantees (§6.8-0003) determinism enums disagree — the clause requires them equal"
    );
}

/// KISS-CONTRACT-6.8-0004 — the `math_precision` is the KISS-Ops MathPrecision
/// attribute `{bit-stable, reduced-mantissa-permitted}` spelled verbatim. The
/// §6.8-0004 enum is compared against a reference CONSTANT (same no-reverse-bind
/// discipline as the accepted §6.8-0003 determinism lint — the constant is NOT a
/// KISS-Ops §6.17 clause-read).
///
/// TEETH: re-fork/re-spell a member (`bit_stable`, `reduced_mantissa`) in §6.8-0004
/// → the parsed set ≠ the canonical constant.
#[test]
fn test_contract_math_precision_imported() {
    let contract = read_spec("contract.md");
    let parsed = enum_set(clause_block(&contract, "KISS-CONTRACT-6.8-0004"));
    let canonical = ops_math_precision_enum();
    assert_eq!(canonical.len(), 2, "KISS-CONTRACT-6.8-0004: the canonical math_precision enum must have 2 members");
    assert_eq!(
        parsed, canonical,
        "KISS-CONTRACT-6.8-0004: §6.8-0004's math_precision enum {parsed:?} is not the verbatim canonical set {canonical:?} (a member was re-forked/re-spelled)"
    );
}

/// KISS-CONTRACT-6.8-0007 — the Capabilities `cost` model is the single
/// authoritative home for cost magnitude; the bare `cost` field's single home is
/// Capabilities. Cross-clause single-home over the three section schemas: `cost` ∈
/// §6.7-0001, and `cost` ∉ §6.8-0001 (Guarantees carries only `cost_provenance`)
/// and ∉ §6.9-0001 (by exact element equality, so `cost_provenance` ≠ `cost`).
///
/// TEETH: duplicate the cost class into Guarantees or Provenance (add a `cost`
/// field there) → the `∉` assertion fails. Same single-home shape as the accepted
/// `audited_status` test.
#[test]
fn test_contract_cost_single_home() {
    let contract = read_spec("contract.md");
    let caps = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.7-0001"));
    let guarantees = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.8-0001"));
    let provenance = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.9-0001"));

    assert!(caps.iter().any(|f| f == "cost"), "KISS-CONTRACT-6.8-0007: the bare `cost` field's single home is Capabilities (§6.7-0001)");
    assert!(
        !guarantees.iter().any(|f| f == "cost"),
        "KISS-CONTRACT-6.8-0007: the bare `cost` field leaked into Guarantees (§6.8-0001) — cost magnitude MUST live only in Capabilities"
    );
    assert!(
        !provenance.iter().any(|f| f == "cost"),
        "KISS-CONTRACT-6.8-0007: the bare `cost` field leaked into Provenance (§6.9-0001) — `cost_provenance` is not the same field"
    );
}

/// KISS-CONTRACT-6.8-0006 — the Guarantees `cost_provenance` value domain is
/// `{declared, measured}`, present in the §6.8-0001 Guarantees schema; §6.8-0006 and
/// §6.9-0006 declare the identical 2-token domain (§6.9-0006 requires equality). The
/// backtick value-token domains of §6.8-0006 and §6.9-0006 are cross-compared (both
/// KISS-Contract's own).
///
/// TEETH: rename a token (`declared`→`stated`) in §6.8-0006 only → the two domains
/// diverge; drop `cost_provenance` from §6.8-0001 → the presence precondition fails.
#[test]
fn test_contract_guarantees_cost_provenance() {
    let contract = read_spec("contract.md");
    let dom_6806 = backtick_lower_words(clause_block(&contract, "KISS-CONTRACT-6.8-0006"));
    let dom_6906 = backtick_lower_words(clause_block(&contract, "KISS-CONTRACT-6.9-0006"));

    assert_eq!(dom_6806, vec!["declared", "measured"], "KISS-CONTRACT-6.8-0006: the §6.8-0006 cost_provenance domain drifted from `{{declared, measured}}`");
    assert_eq!(
        dom_6806, dom_6906,
        "KISS-CONTRACT-6.8-0006: the §6.8-0006 and §6.9-0006 cost_provenance value domains diverged (a token was renamed on one side only)"
    );

    // `cost_provenance` is present in the Guarantees schema.
    let guarantees = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.8-0001"));
    assert!(
        guarantees.iter().any(|f| f == "cost_provenance"),
        "KISS-CONTRACT-6.8-0006: `cost_provenance` must be present in the Guarantees (§6.8-0001) schema"
    );
}

/// KISS-CONTRACT-6.9-0006 — the Provenance `cost_provenance` MUST equal the
/// Guarantees `cost_provenance`; that equality is only satisfiable if
/// `cost_provenance` is present in BOTH the §6.8-0001 and §6.9-0001 schemas and both
/// clauses share the `{declared, measured}` domain (cross-presence + shared domain).
///
/// TEETH: drop `cost_provenance` from either schema → the cross-presence assert
/// fails; fork its token domain between §6.8-0006/§6.9-0006 → the shared-domain
/// assert fails.
#[test]
fn test_contract_cost_provenance_consistent() {
    let contract = read_spec("contract.md");
    let guarantees = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.8-0001"));
    let provenance = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.9-0001"));

    assert!(
        guarantees.iter().any(|f| f == "cost_provenance"),
        "KISS-CONTRACT-6.9-0006: `cost_provenance` must be present in the Guarantees (§6.8-0001) schema for the equality to hold"
    );
    assert!(
        provenance.iter().any(|f| f == "cost_provenance"),
        "KISS-CONTRACT-6.9-0006: `cost_provenance` must be present in the Provenance (§6.9-0001) schema for the equality to hold"
    );

    // Both clauses that pin the token domain agree on `{declared, measured}`.
    let dom_6806 = backtick_lower_words(clause_block(&contract, "KISS-CONTRACT-6.8-0006"));
    let dom_6906 = backtick_lower_words(clause_block(&contract, "KISS-CONTRACT-6.9-0006"));
    assert_eq!(
        dom_6906, dom_6806,
        "KISS-CONTRACT-6.9-0006: the Provenance (§6.9-0006) and Guarantees (§6.8-0006) cost_provenance domains diverged"
    );
    assert_eq!(dom_6906, vec!["declared", "measured"], "KISS-CONTRACT-6.9-0006: the cost_provenance domain drifted from `{{declared, measured}}`");
}

/// KISS-CONTRACT-6.9-0003 — the Provenance `revision_hash` MUST equal the Identity
/// `revision_hash` byte-for-byte; that obligation is only satisfiable if the
/// identically-spelled `revision_hash` field exists in BOTH the §6.3-0001 Identity
/// and §6.9-0001 Provenance schemas (cross-presence).
///
/// TEETH: rename `revision_hash`→`rev_hash` in Identity (§6.3-0001) but not
/// Provenance (§6.9-0001) → the two schemas no longer share the field name and the
/// cross-presence assert fails.
#[test]
fn test_contract_provenance_revision_matches_identity() {
    let contract = read_spec("contract.md");
    let identity = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.3-0001"));
    let provenance = field_list_ordered(clause_block(&contract, "KISS-CONTRACT-6.9-0001"));

    assert!(
        identity.iter().any(|f| f == "revision_hash"),
        "KISS-CONTRACT-6.9-0003: `revision_hash` must be present in the Identity (§6.3-0001) schema"
    );
    assert!(
        provenance.iter().any(|f| f == "revision_hash"),
        "KISS-CONTRACT-6.9-0003: `revision_hash` must be present in the Provenance (§6.9-0001) schema"
    );
    // The SAME spelling is the shared field the byte-for-byte equality is taken over.
    let shared = identity.iter().find(|f| *f == "revision_hash");
    assert_eq!(
        shared,
        provenance.iter().find(|f| *f == "revision_hash"),
        "KISS-CONTRACT-6.9-0003: Identity and Provenance must carry the identically-spelled `revision_hash` field"
    );
}

// ===========================================================================
// (B) Behavioral back via the in-tree §6.6-0006 acceptor.
// ===========================================================================

/// Rank-2 launch scalars: `extents0 = [6, 4]`, `strides0 = [4, 1]`, `off0 = 0`,
/// `n = 24`.
fn launch_scalars() -> LaunchScalars {
    LaunchScalars {
        rank: 2,
        n: 24,
        extents: vec![vec![6, 4]],
        strides: vec![vec![4, 1]],
        idx_extents: vec![],
        off: vec![0],
        ws_bytes: 0,
        param: vec![],
    }
}

/// KISS-CONTRACT-6.11-0008 — each Dispatch derivation field value is encoded as a
/// string carrying a machine-evaluable §6.6-0006 expression; a reader MUST reject,
/// with a typed decline, a field value the grammar does not accept. Driven through
/// the real in-tree acceptor `dispatch_expr::eval` (not a re-implementation): for
/// each of the five named fields a grammatical expression is ACCEPTED and evaluates
/// to the pinned integer, while free prose / malformed input DECLINES.
///
/// TEETH: a reader that stored a Dispatch field as opaque free text would (wrongly)
/// accept `provider-internal` (which §6.6-0001 explicitly forbids) — `eval` declines
/// it with a typed `Decline`, so this pins that the TEXT encoding of a Dispatch
/// field IS a §6.6-0006 expression, not opaque prose.
#[test]
fn test_contract_text_dispatch() {
    let s = launch_scalars();

    // (field, grammatical expression, expected value, evaluation mode)
    let cases: &[(&str, &str, i64, EvalMode)] = &[
        ("invocation_domain", "extents0[0] * extents0[1]", 24, EvalMode::Domain),
        ("workgroup_sizing", "ceil_div(n, 256)", 1, EvalMode::Domain),
        ("count_to_grid", "ceil_div(n, 128)", 1, EvalMode::Domain),
        ("thread_mapping", "n", 24, EvalMode::Structural),
        ("addressing_rule", "off0 + strides0[0]", 4, EvalMode::Structural),
    ];
    for (field, expr, want, mode) in cases {
        // Accepted, and eval actually RAN the grammar (value pinned, not a stub Ok).
        assert_eq!(
            eval(expr, &s, *mode),
            Ok(*want),
            "KISS-CONTRACT-6.11-0008: the {field} value `{expr}` is a grammatical §6.6-0006 expression and MUST evaluate to {want}"
        );
        // The SAME field carrying free prose MUST be a typed decline, not opaque text.
        assert!(
            matches!(eval("provider-internal", &s, *mode), Err(Decline::UnknownSymbol(_))),
            "KISS-CONTRACT-6.11-0008: a free-prose {field} value `provider-internal` MUST decline (never be stored as opaque free text)"
        );
    }

    // Further non-grammar values decline with a typed error — never a panic, never Ok.
    assert!(
        matches!(eval("n +", &s, EvalMode::Domain), Err(Decline::ParseError(_))),
        "KISS-CONTRACT-6.11-0008: a malformed Dispatch value `n +` MUST decline ParseError"
    );
    assert!(
        matches!(eval("foo", &s, EvalMode::Structural), Err(Decline::UnknownSymbol(_))),
        "KISS-CONTRACT-6.11-0008: an out-of-vocabulary Dispatch value `foo` MUST decline UnknownSymbol"
    );
}
