//! KISS-CONSUME structural / document-consistency conformance module, continuing the
//! #91 coverage burndown. KISS-Consume is the recognition (lift) DISCIPLINE: it ships
//! **no** reference lifter and **no** input-structure-graph codec in this crate — its
//! only crate codec (`kiss_conformance::decline`) already backs §6.4-0009/-0010 in
//! `tests/decline.rs`. So, exactly like the EMIT partition module
//! (`tests/emit_partition.rs`, which established the `read_spec` / `clause_block` /
//! `enum_members` / `section_slice` pattern), the remaining CHEAP CONSUME clauses are
//! backed by DOC-LINT tests: each extracts a STRUCTURAL SET (an enum, a partition, an
//! ordered decision procedure, a dependency-edge set) from the normative text and
//! set-compares it against a canonical reference constant or a cross-document set, so
//! the test FAILS on a plausible spec drift (a renamed / dropped / re-forked category,
//! a swapped residue↔decline partition, a renumbered step, a new sibling edge).
//!
//! These are structural bindings, NOT behavioral tests of a lifter. The behavioral
//! clauses (structure-based recognition over an input-structure graph, the four step
//! classifiers, the never-panic obligation, the round-trip harness) genuinely need a
//! lifter + the KISS-Conform expressibility oracle and are out of scope here; they are
//! left `untested` in UNBACKED.tsv rather than backed with a hollow test.
//!
//! Clauses bound by THIS module (9):
//!   * KISS-CONSUME-6.0-0001  determinism-class exact-byte + imported determinism enum
//!   * KISS-CONSUME-6.3-0001  completeness tracks residue-empty ↔ semantics_kind
//!   * KISS-CONSUME-6.3-0004  residue-bearing 2-token subset (single authority)
//!   * KISS-CONSUME-6.4-0002  MECE ordered decision procedure (steps 1–4)
//!   * KISS-CONSUME-6.4-0008  whole-input-decline ⊎ residue-tag partition
//!   * KISS-CONSUME-6.6-0004  round-trip tier selected by imported determinism enum
//!   * KISS-CONSUME-6.6-0005  Consume/Emit are DAG siblings — no dependency edge
//!   * KISS-CONSUME-7.1-0001  mandatory core = §6.1–§6.5 (round-trip §6.6 excluded)
//!   * KISS-CONSUME-9.1-0001  prerequisite closure = {Classify, Ops, Contract}, no Emit
//!
//! CITATION DISCIPLINE (kiss_trace reverse-binds a clause when its full id literal
//! appears anywhere in a test's scope): every test writes ONLY its own KISS-CONSUME-*
//! id. Sibling clauses are read via `section_slice` + content anchors (never by a
//! sibling's full clause id — that would over-credit a clause this module does not
//! back, e.g. the still-`lint`-backed §6.4-0001 or the skipped §7.2-0001), and the
//! cross-document umbrella §2.2 edge table is read by content (the umbrella defines no
//! clause ids). Helper fns below live outside every `#[test]` scope, so their text is
//! attributed to no clause.

use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a bare phrase for its own sake.
// ---------------------------------------------------------------------------

/// Read a `spec/<name>` markdown file. `spec/` is one directory above the crate.
fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// Body text of a single clause: from its `**KISS-…**` bold anchor up to the next
/// clause definition or the next markdown heading.
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

/// Emphasis-free, single-spaced view of a span: strip markdown `*`/`` ` `` and heal
/// line wraps, so a phrase that is bolded and wrapped in the source compares as plain
/// prose. Case and punctuation (em-dash, parens, `§`) are preserved.
fn plain(s: &str) -> String {
    let cleaned: String = s.chars().filter(|&c| c != '*' && c != '`').collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every `` `token` `` delimited by backticks within `window`, internal whitespace
/// healed. Used to read a byte-exact token SET out of a normative sentence.
fn backtick_tokens(window: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = window;
    while let Some(a) = rest.find('`') {
        rest = &rest[a + 1..];
        match rest.find('`') {
            Some(b) => {
                let tok: String = rest[..b].split_whitespace().collect();
                if !tok.is_empty() {
                    out.push(tok);
                }
                rest = &rest[b + 1..];
            }
            None => break,
        }
    }
    out
}

/// Build a `BTreeSet<String>` from string literals (expected-set construction).
fn sset(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// The KISS-Ops-owned canonical determinism/fidelity enum (KISS-Ops §6.0-0001). Its
/// membership and no-fork ownership are enforced Ops-side by the `kiss_tables` no-fork
/// lint on the owning clause; the CONSUME clauses only IMPORT it, so a CONSUME test
/// compares its parsed enum against this reference constant rather than re-reading (and
/// thereby claiming to back) the KISS-Ops clause it does not exercise. Members carry no
/// internal whitespace, matching `enum_members`.
fn ops_determinism_enum() -> BTreeSet<String> {
    sset(&["exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"])
}

/// The four-category refusal-taxonomy SET, read by CONTENT out of the §6.4-0001 owner
/// sentence ("… the four categories, spelled byte-exact: `…`, `…`, …") — NOT via that
/// clause's id, which is still `lint`-backed by `kiss_tables` and must not be
/// reverse-bound here. Returns the byte-exact token set (expected size 4).
fn four_category_set(consume: &str) -> BTreeSet<String> {
    let sec = section_slice(consume, "### 6.4", "### 6.5");
    let flat: String = sec.split_whitespace().collect::<Vec<_>>().join(" ");
    let anchor = "four categories, spelled byte-exact";
    let i = flat
        .find(anchor)
        .expect("§6.4-0001 owner sentence (`four categories, spelled byte-exact`) not found");
    let after = &flat[i..];
    let end = after
        .find("An implementation")
        .unwrap_or_else(|| after.len().min(300));
    backtick_tokens(&after[..end]).into_iter().collect()
}

/// The ordered step decision procedure `(step_no, category_token, phase)` parsed from
/// the `Step N — <token> (<phase>).` headings of §6.4-0003..0006 (read by content out
/// of the §6.4 section, never by their clause ids). `(` delimits the phase so parsing
/// does not depend on the em-dash spelling.
fn step_headings(consume: &str) -> Vec<(u32, String, String)> {
    let sec = plain(section_slice(consume, "### 6.4", "### 6.5"));
    let mut out = Vec::new();
    for n in 1..=4u32 {
        let key = format!("Step {n}");
        let i = sec
            .find(&key)
            .unwrap_or_else(|| panic!("§6.4 decision procedure: `{key}` heading not found"));
        let after = &sec[i + key.len()..];
        let lp = after
            .find('(')
            .unwrap_or_else(|| panic!("`{key}` heading has no `(<phase>)`"));
        let token = after[..lp]
            .trim_start_matches(|c: char| c == '—' || c == ' ')
            .trim()
            .to_string();
        let rest = &after[lp + 1..];
        let rp = rest.find(')').expect("unterminated `(<phase>)` in step heading");
        out.push((n, token, rest[..rp].trim().to_string()));
    }
    out
}

/// Every `§6.N` section reference appearing in `block` (single-digit subsections).
fn section_refs(block: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let needle = "§6.";
    let mut start = 0;
    while let Some(rel) = block[start..].find(needle) {
        let i = start + rel;
        let after = &block[i + needle.len()..];
        if let Some(d) = after.chars().next() {
            if d.is_ascii_digit() {
                out.insert(format!("§6.{d}"));
            }
        }
        start = i + needle.len();
    }
    out
}

/// The umbrella §2.2 dependency edge table, parsed by CONTENT (the umbrella defines no
/// clause ids). Each `| A → B | LABEL |` row where both endpoints are sub-standards
/// becomes `(dependency, dependent, label)`. The `| each of the other eight → … |` row
/// is skipped (left endpoint is not a `KISS-` node).
fn umbrella_edges(umbrella: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in umbrella.lines() {
        let l = line.trim();
        if !l.starts_with("| KISS-") || !l.contains('→') {
            continue;
        }
        let cells: Vec<&str> = l.trim_matches('|').split('|').map(|c| c.trim()).collect();
        if cells.len() < 2 {
            continue;
        }
        let mut ab = cells[0].split('→');
        let a = ab.next().unwrap_or("").trim().to_string();
        let b = ab.next().unwrap_or("").trim().to_string();
        if a.starts_with("KISS-") && b.starts_with("KISS-") {
            out.push((a, b, cells[1].trim().to_string()));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// §6.0 — determinism / fidelity class
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.0-0001 — every structural obligation in §6–§8 is determinism-class
/// EXACT-BYTE, evaluated with a byte-exact comparator and never a tolerance /
/// order-invariant one; the numeric determinism enum is OWNED by KISS-Ops and imported,
/// not re-forked. Teeth: the `{…}` enum parsed out of the clause body must equal the
/// canonical KISS-Ops set `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`
/// (a re-fork / rename / drop here fails the set equality), AND the clause must select
/// exact-byte while forbidding the tolerance and order-invariant comparators.
#[test]
fn test_consume_determinism_class_exact_byte() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.0-0001");
    let ops = ops_determinism_enum();
    assert_eq!(ops.len(), 3, "canonical determinism enum must have 3 members: {ops:?}");
    assert_eq!(
        enum_members(block),
        ops,
        "KISS-CONSUME-6.0-0001: the imported determinism enum drifted from the canonical KISS-Ops set"
    );
    let p = plain(block).to_lowercase();
    assert!(
        p.contains("exact byte") || p.contains("exact-byte"),
        "KISS-CONSUME-6.0-0001: the clause does not select the exact-byte determinism class"
    );
    assert!(
        p.contains("must not") && p.contains("tolerance") && p.contains("order-invariant"),
        "KISS-CONSUME-6.0-0001: the clause does not forbid the tolerance / order-invariant comparators"
    );
    assert!(
        p.contains("owned by kiss-ops") && p.contains("re-forked"),
        "KISS-CONSUME-6.0-0001: the clause does not pin the enum as KISS-Ops-owned / not-re-forked"
    );
}

// ---------------------------------------------------------------------------
// §6.3 — lift outcome and the residue contract
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.3-0001 — contract completeness MUST track the residue-empty vs.
/// residue-non-empty distinction: empty residue ⇒ `semantics_kind = machine-checkable-IR`,
/// non-empty residue ⇒ `semantics_kind = declared-op-tag` + recorded `lift_residue`, and
/// machine-checkable-IR is forbidden for a kernel whose lift left residue. Teeth: the
/// `semantics_kind` axis parsed from the body must be exactly the 2-token set
/// {machine-checkable-IR, declared-op-tag}, the empty→machine-checkable-IR and
/// non-empty→declared-op-tag pairings must both be present, non-empty MUST NOT be paired
/// with machine-checkable-IR (swap guard), and the forbiddance sentence must be present.
/// MEDIUM CONFIDENCE: prose-adjacency extraction, weaker than a clean `{…}` literal.
#[test]
fn test_consume_completeness_tracks_lift_fraction() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.3-0001");

    // The semantics_kind axis: every `semantics_kind = <token>` occurrence.
    let flat: String = block.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut kinds = BTreeSet::new();
    let key = "semantics_kind = ";
    let mut start = 0;
    while let Some(rel) = flat[start..].find(key) {
        let i = start + rel + key.len();
        let tok: String = flat[i..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if !tok.is_empty() {
            kinds.insert(tok);
        }
        start = i;
    }
    assert_eq!(
        kinds,
        sset(&["machine-checkable-IR", "declared-op-tag"]),
        "KISS-CONSUME-6.3-0001: the semantics_kind axis drifted from {{machine-checkable-IR, declared-op-tag}}"
    );

    let p = plain(block);
    assert!(
        p.contains("empty residue MUST get semantics_kind = machine-checkable-IR"),
        "KISS-CONSUME-6.3-0001: empty residue is not paired with machine-checkable-IR"
    );
    assert!(
        p.contains("non-empty residue MUST get semantics_kind = declared-op-tag"),
        "KISS-CONSUME-6.3-0001: non-empty residue is not paired with declared-op-tag"
    );
    // Swap guard: non-empty must never be the machine-checkable-IR branch.
    assert!(
        !p.contains("non-empty residue MUST get semantics_kind = machine-checkable-IR"),
        "KISS-CONSUME-6.3-0001: non-empty residue is (wrongly) paired with machine-checkable-IR"
    );
    assert!(
        p.contains("MUST NOT emit machine-checkable-IR for a kernel whose lift left residue"),
        "KISS-CONSUME-6.3-0001: the clause does not forbid machine-checkable-IR when residue remains"
    );
}

/// KISS-CONSUME-6.3-0004 — each `lift_residue` entry MUST tag its region with exactly one
/// category from the residue-bearing SUBSET `{unrecognized-but-expressible,
/// inexpressible-residue}`; this two-token subset is the single authoritative definition,
/// and `not-a-kernel` / `wrong-op-class` are whole-input typed declines, never residue
/// tags. Teeth: the subset parsed from the clause must equal exactly those two tokens, be
/// a strict 2-of-4 subset of the §6.4-0001 four-category set (read by content), exclude
/// the two decline tokens, and the clause must name them as "typed declines, never
/// residue tags".
#[test]
fn test_consume_residue_entry_tagged() {
    let consume = read_spec("consume.md");
    let raw = clause_block(&consume, "KISS-CONSUME-6.3-0004");
    let flat: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let a = flat
        .find("residue-bearing subset")
        .expect("KISS-CONSUME-6.3-0004: `residue-bearing subset` phrase not found");
    let after = &flat[a..];
    let end = after.find("spelled byte-exact").unwrap_or(after.len());
    let subset: BTreeSet<String> = backtick_tokens(&after[..end]).into_iter().collect();

    let expected = sset(&["unrecognized-but-expressible", "inexpressible-residue"]);
    assert_eq!(
        subset, expected,
        "KISS-CONSUME-6.3-0004: residue-bearing subset drifted from the two authoritative tokens"
    );

    let four = four_category_set(&consume);
    assert_eq!(four.len(), 4, "expected 4 refusal categories, got {four:?}");
    assert!(
        subset.is_subset(&four) && subset.len() == 2,
        "KISS-CONSUME-6.3-0004: residue subset is not a strict 2-of-4 subset of the taxonomy"
    );
    assert!(
        !subset.contains("not-a-kernel") && !subset.contains("wrong-op-class"),
        "KISS-CONSUME-6.3-0004: a whole-input decline token leaked into the residue-bearing subset"
    );

    let p = plain(raw);
    assert!(
        p.contains("not-a-kernel")
            && p.contains("wrong-op-class")
            && p.contains("typed declines")
            && p.contains("never residue tags"),
        "KISS-CONSUME-6.3-0004: the clause does not exclude the two declines as `never residue tags`"
    );
}

// ---------------------------------------------------------------------------
// §6.4 — the refusal taxonomy (MECE)
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.4-0002 — the four categories are MECE, applied in two ordered units:
/// Phase A whole-input pre-check (steps 1–2), Phase B per-region (steps 3–4, first match
/// wins). Teeth: the `Step N — <token> (<phase>)` headings of §6.4-0003..0006 must be
/// exactly the ordered mapping [1 not-a-kernel whole-input, 2 wrong-op-class whole-input,
/// 3 unrecognized-but-expressible per-region, 4 inexpressible-residue per-region]
/// (renumber / rename / re-phase fails), the step-token SET must equal the §6.4-0001
/// four-category set read by content (MECE completeness), and the clause body must state
/// the MECE / Phase-A / Phase-B structure.
#[test]
fn test_consume_taxonomy_mece_ordered() {
    let consume = read_spec("consume.md");

    let steps = step_headings(&consume);
    let expected = vec![
        (1u32, "not-a-kernel".to_string(), "whole-input".to_string()),
        (2, "wrong-op-class".to_string(), "whole-input".to_string()),
        (3, "unrecognized-but-expressible".to_string(), "per-region".to_string()),
        (4, "inexpressible-residue".to_string(), "per-region".to_string()),
    ];
    assert_eq!(
        steps, expected,
        "KISS-CONSUME-6.4-0002: the ordered step decision procedure drifted"
    );

    let step_tokens: BTreeSet<String> = steps.iter().map(|(_, t, _)| t.clone()).collect();
    let four = four_category_set(&consume);
    assert_eq!(four.len(), 4, "expected 4 refusal categories, got {four:?}");
    assert_eq!(
        step_tokens, four,
        "KISS-CONSUME-6.4-0002: the step tokens and the §6.4-0001 taxonomy disagree (not MECE-complete)"
    );

    let p = plain(clause_block(&consume, "KISS-CONSUME-6.4-0002"));
    assert!(
        p.contains("mutually exclusive and collectively exhaustive"),
        "KISS-CONSUME-6.4-0002: the clause does not state the MECE property"
    );
    assert!(
        p.contains("Phase A") && p.contains("Phase B"),
        "KISS-CONSUME-6.4-0002: the clause does not state the two-phase (A/B) decision units"
    );
}

/// KISS-CONSUME-6.4-0008 — the taxonomy partitions into whole-input TYPED DECLINES
/// (`not-a-kernel`, `wrong-op-class`) and residue TAGS (`unrecognized-but-expressible`,
/// `inexpressible-residue`); the declines produce no contract (Phase A), the tags are
/// `lift_residue` entries (Phase B). Teeth: split the clause at its `while` connector and
/// classify each of the four taxonomy tokens (read by content) into the decline half or
/// the residue half; the two halves must be exactly {not-a-kernel, wrong-op-class} and
/// {unrecognized-but-expressible, inexpressible-residue}, disjoint, unioning to the four
/// categories — so a token crossing the partition fails — and the clause must bind
/// declines to a typed decline / no-contract and tags to `lift_residue`.
#[test]
fn test_consume_decline_vs_residue_partition() {
    let consume = read_spec("consume.md");
    let bp = plain(clause_block(&consume, "KISS-CONSUME-6.4-0008"));
    let four = four_category_set(&consume);
    assert_eq!(four.len(), 4, "expected 4 refusal categories, got {four:?}");

    // Phase-A declines are stated before the `while` connector; Phase-B residue tags
    // after it and before the trailing `Because …` sentence (which re-mentions all four).
    let widx = bp
        .find(" while ")
        .expect("KISS-CONSUME-6.4-0008: `while` partition connector not found");
    let bidx = bp.find(" Because ").unwrap_or(bp.len());
    let before = &bp[..widx];
    let after = &bp[widx..bidx];

    let declines: BTreeSet<String> =
        four.iter().filter(|t| before.contains(t.as_str())).cloned().collect();
    let residues: BTreeSet<String> =
        four.iter().filter(|t| after.contains(t.as_str())).cloned().collect();

    assert_eq!(
        declines,
        sset(&["not-a-kernel", "wrong-op-class"]),
        "KISS-CONSUME-6.4-0008: the whole-input decline half is not exactly {{not-a-kernel, wrong-op-class}}"
    );
    assert_eq!(
        residues,
        sset(&["unrecognized-but-expressible", "inexpressible-residue"]),
        "KISS-CONSUME-6.4-0008: the residue-tag half is not exactly the two residue tokens"
    );
    assert!(
        declines.is_disjoint(&residues),
        "KISS-CONSUME-6.4-0008: a token appears on both sides of the decline/residue partition"
    );
    let mut union = declines.clone();
    union.extend(residues.iter().cloned());
    assert_eq!(
        union, four,
        "KISS-CONSUME-6.4-0008: the decline/residue partition does not cover the four categories"
    );

    assert!(
        before.contains("typed decline") && bp.contains("no contract is produced"),
        "KISS-CONSUME-6.4-0008: the declines are not bound to a typed decline / no contract"
    );
    assert!(
        after.contains("lift_residue"),
        "KISS-CONSUME-6.4-0008: the residue tags are not bound to `lift_residue` entries"
    );
}

// ---------------------------------------------------------------------------
// §6.6 — the emit/consume round-trip (two tiers)
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.6-0004 — which round-trip tier applies to an op MUST be selected by the
/// imported KISS-Ops determinism/fidelity enum: an `exact-byte` op is tier-2 bit-identity
/// (under the same-language on-device restriction), while `ULP/tolerance` and
/// `order-invariant/nondeterministic` ops are compared under their declared comparator and
/// MUST NOT be compared for bit identity. Teeth: the `{…}` enum parsed here must equal the
/// canonical KISS-Ops set (a re-fork fails), exact-byte must be tied to tier-2 bit
/// identity, and the two non-exact classes must be tied to "declared comparator" +
/// "MUST NOT be compared for bit identity". Distinct clause body from §6.0-0001.
#[test]
fn test_consume_roundtrip_tier_selected_by_determinism_enum() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.6-0004");
    assert_eq!(
        enum_members(block),
        ops_determinism_enum(),
        "KISS-CONSUME-6.6-0004: the round-trip tier-selector enum drifted from the canonical KISS-Ops set"
    );
    let p = plain(block).to_lowercase();
    assert!(
        p.contains("exact-byte") && p.contains("tier-2 bit identity"),
        "KISS-CONSUME-6.6-0004: exact-byte is not tied to a tier-2 bit-identity claim"
    );
    assert!(
        p.contains("declared comparator"),
        "KISS-CONSUME-6.6-0004: non-exact ops are not tied to their declared comparator"
    );
    assert!(
        p.contains("must not be compared for bit identity"),
        "KISS-CONSUME-6.6-0004: the clause does not forbid bit-identity for non-exact-byte ops"
    );
    assert!(
        p.contains("must not re-fork this enum"),
        "KISS-CONSUME-6.6-0004: the clause does not forbid re-forking the imported enum"
    );
}

/// KISS-CONSUME-6.6-0005 — KISS-Consume and KISS-Emit are DAG SIBLINGS with NO dependency
/// edge in either direction; the round-trip is a join verified by KISS-Conform, not an
/// edge. Teeth: the umbrella §2.2 edge table (parsed by content) must contain NO edge
/// Consume→Emit or Emit→Consume — while both nodes DO appear as endpoints elsewhere (so
/// the negative check is not vacuous) — and the clause body must state the sibling / no
/// dependency edge / join-verified-by-KISS-Conform invariant.
#[test]
fn test_consume_emit_are_siblings_no_edge() {
    let umbrella = read_spec("umbrella.md");
    let consume = read_spec("consume.md");
    let edges = umbrella_edges(&umbrella);
    assert!(
        edges.len() >= 15,
        "KISS-CONSUME-6.6-0005: parsed only {} umbrella edges — parser drift, check would be vacuous",
        edges.len()
    );
    let has_consume = edges.iter().any(|(a, b, _)| a == "KISS-Consume" || b == "KISS-Consume");
    let has_emit = edges.iter().any(|(a, b, _)| a == "KISS-Emit" || b == "KISS-Emit");
    assert!(
        has_consume && has_emit,
        "KISS-CONSUME-6.6-0005: Consume/Emit are not endpoints of any parsed edge — check would be vacuous"
    );
    assert!(
        !edges.iter().any(|(a, b, _)| {
            (a == "KISS-Consume" && b == "KISS-Emit") || (a == "KISS-Emit" && b == "KISS-Consume")
        }),
        "KISS-CONSUME-6.6-0005: the umbrella edge table contains a Consume↔Emit dependency edge"
    );

    let p = plain(clause_block(&consume, "KISS-CONSUME-6.6-0005"));
    assert!(
        p.contains("siblings") && p.contains("no") && p.contains("dependency edge"),
        "KISS-CONSUME-6.6-0005: the clause does not state the sibling / no-dependency-edge invariant"
    );
    assert!(
        p.contains("join") && p.contains("KISS-Conform"),
        "KISS-CONSUME-6.6-0005: the clause does not state the round-trip is a KISS-Conform-verified join"
    );
}

// ---------------------------------------------------------------------------
// §7.1 — mandatory core
// ---------------------------------------------------------------------------

/// KISS-CONSUME-7.1-0001 — the mandatory core is structure-based recognition (§6.1), the
/// lift-into-the-op-DAG discipline (§6.2), the residue contract (§6.3), the four-category
/// MECE taxonomy (§6.4), and typed-decline-never-panic (§6.5); the round-trip (§6.6) is
/// NOT core. Teeth: the §-refs naming the core pillars must be exactly
/// {§6.1,§6.2,§6.3,§6.4,§6.5} (a dropped pillar fails), §6.6 must NOT be named, the §7.2
/// negotiable-feature section (read by content) must mark the round-trip subset "not part
/// of the mandatory core", and each named §6.x section heading must exist.
/// MEDIUM CONFIDENCE: references pillars by section number, so a pillar RENAME (vs a drop)
/// is not caught.
#[test]
fn test_consume_mandatory_core() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-7.1-0001");

    let refs = section_refs(block);
    assert_eq!(
        refs,
        sset(&["§6.1", "§6.2", "§6.3", "§6.4", "§6.5"]),
        "KISS-CONSUME-7.1-0001: the mandatory-core pillar set drifted"
    );
    assert!(
        !refs.contains("§6.6"),
        "KISS-CONSUME-7.1-0001: the negotiable round-trip (§6.6) was pulled into the mandatory core"
    );

    // §7.2 (read by content, never by its clause id) keeps the round-trip subset out.
    let s72 = plain(section_slice(&consume, "### 7.2", "## 8"));
    assert!(
        s72.contains("not part of the mandatory core"),
        "KISS-CONSUME-7.1-0001: §7.2 does not mark the declared round-trip subset out of the core"
    );
    assert!(
        s72.contains("declared round-trip subset"),
        "KISS-CONSUME-7.1-0001: §7.2 is not the declared-round-trip-subset feature section"
    );

    for h in ["### 6.1", "### 6.2", "### 6.3", "### 6.4", "### 6.5"] {
        assert!(
            consume.contains(h),
            "KISS-CONSUME-7.1-0001: named core section heading `{h}` is missing"
        );
    }
    assert!(
        consume.contains("### 6.6"),
        "KISS-CONSUME-7.1-0001: the round-trip section §6.6 is missing (its exclusion would be vacuous)"
    );
}

// ---------------------------------------------------------------------------
// §9.1 — claim format & prerequisite closure
// ---------------------------------------------------------------------------

/// KISS-CONSUME-9.1-0001 — a KISS-Consume claim MUST be prerequisite-closed over its
/// incoming STRUCTURAL edges: it MUST also claim KISS-Classify, KISS-Ops, KISS-Contract,
/// and MUST NOT be required to claim KISS-Emit (a sibling). Teeth: the STRUCTURAL edges
/// into KISS-Consume in the umbrella §2.2 table (parsed by content) must be exactly
/// {Classify, Ops, Contract} with NO Emit→Consume edge, and the clause body must name the
/// same three prerequisites and state Emit is not required.
#[test]
fn test_consume_claim_prerequisite_closed() {
    let umbrella = read_spec("umbrella.md");
    let consume = read_spec("consume.md");
    let edges = umbrella_edges(&umbrella);

    let prereqs: BTreeSet<String> = edges
        .iter()
        .filter(|(_, b, label)| b == "KISS-Consume" && label.contains("STRUCTURAL"))
        .map(|(a, _, _)| a.clone())
        .collect();
    assert_eq!(
        prereqs,
        sset(&["KISS-Classify", "KISS-Ops", "KISS-Contract"]),
        "KISS-CONSUME-9.1-0001: the STRUCTURAL prerequisite closure for KISS-Consume drifted"
    );
    assert!(
        !edges.iter().any(|(a, b, _)| a == "KISS-Emit" && b == "KISS-Consume"),
        "KISS-CONSUME-9.1-0001: the umbrella table has an Emit→Consume edge (Emit is a sibling, not a prerequisite)"
    );

    let p = plain(clause_block(&consume, "KISS-CONSUME-9.1-0001"));
    assert!(
        p.contains("KISS-Classify") && p.contains("KISS-Ops") && p.contains("KISS-Contract"),
        "KISS-CONSUME-9.1-0001: the clause does not name the three structural prerequisites"
    );
    assert!(
        p.contains("MUST NOT be required to claim KISS-Emit"),
        "KISS-CONSUME-9.1-0001: the clause does not exempt the sibling KISS-Emit from the closure"
    );
}
