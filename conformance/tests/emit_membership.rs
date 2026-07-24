//! Second KISS-Emit conformance module — the lowering-partition MEMBERSHIP (§6.3
//! driver-may-spell, §6.4 emitter-must-supply) and the neutrality-audit SCOPE (§6.5),
//! continuing the #91 coverage burndown. Companion to `emit_partition.rs`, reusing the
//! same `read_spec()` / `clause_block()` doc-lint pattern.
//!
//! KISS-Emit ships **no** reference emitter in this crate (`conformance/src/*` has no
//! `emit.rs` / emitter surface), so — as in `emit_partition.rs` — the behavioral EMIT
//! clauses (partition-at-runtime, artifact ABI, tier-2 round-trip, typed decline) are
//! out of scope; there is no codec to drive. These are structural / document-consistency
//! bindings.
//!
//! What gives them TEETH (not a phrase grep): each normative §6.3/§6.4 clause's own
//! bucket + surface self-declaration is cross-checked against an INDEPENDENT second
//! statement of the same partition —
//!   * §6.3-0001..0005, §6.4-0004: the Appendix-A.2 worked-example partition-trace
//!     table (rows carry Set | Surface? | Clause), parsed here into a
//!     clause -> (bucket, surface) map;
//!   * §6.4-0003: the §3 `Declared-ULP atom` term-definition atom set;
//!   * §6.4-0004 (additionally): the KISS-Ops §6.16 self-contained dtype table,
//!     read by its `### 6.16` heading (never by an Ops clause-id, so no cross-standard
//!     clause is reverse-bound in kiss_trace);
//!   * §6.5-0001..0003: the surface-bearing §6.3 driver set, computed with the SAME
//!     surface classifier the §6.3-0006 test uses, plus the emitter-side homes
//!     (§6.4-0001/0005) located BY CONTENT (a runtime scan over §6.4 definitions, so
//!     their clause-ids never appear as source literals and are not reverse-cited).
//! Two independently-authored statements agreeing is a real invariant that fails on
//! one-sided drift.
//!
//! Each test cites ONLY its own EMIT clause-id (in its doc-comment and via a single
//! `clause_block(&emit, "KISS-EMIT-…")` literal), so it forward-binds exactly the clause
//! whose `*Test:*` fn name it bears and over-credits no still-unbacked sibling. Sibling
//! clauses are reached through helpers whose clause-id literals live in non-`#[test]`
//! code (invisible to the traceability scanner) or through runtime-constructed / short
//! `§6.x-nnnn` ids (which the `KISS-<SUB>-…` citation regex does not match).
//!
//! Clauses bound: KISS-EMIT-6.3-0001..0005, KISS-EMIT-6.4-0003, KISS-EMIT-6.4-0004,
//! KISS-EMIT-6.5-0001..0003 (10 clauses).

use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a phrase.
// (Same shapes as emit_partition.rs; integration-test files are separate crates,
// so the pattern is restated here rather than imported.)
// ---------------------------------------------------------------------------

/// Read a `spec/<name>` markdown file. `spec/` is one directory above the crate.
fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// The body text of a single clause: from its `**KISS-…**` bold anchor up to the next
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

/// Every clause id defined in `md` whose id begins with `prefix`. `prefix` is a bare
/// section stem such as `"KISS-EMIT-6.3-"`; because it carries no 4-digit tail it is
/// NOT itself a clause-id (the traceability scanner's `KISS-<SUB>-…-\d{4}` pattern does
/// not match it), so using it here reverse-cites nothing.
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

/// A §3 term definition block: from its `**<term>**` bold anchor up to the next term
/// bullet (`\n- **`) or the end of the slice. Term definitions carry no clause-id, so
/// this reads the §3 statement purely by content.
fn term_def<'a>(md: &'a str, term: &str) -> &'a str {
    let anchor = format!("**{term}**");
    let start = md
        .find(&anchor)
        .unwrap_or_else(|| panic!("term `{term}` is not defined in the spec text"));
    let rest = &md[start..];
    let end = rest
        .get(1..)
        .and_then(|r| r.find("\n- **"))
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

/// The dtype token set from the KISS-Ops §6.16 self-contained dtype-layout table (the
/// first backtick column of each `| \`token\` | … |` row). Read by the `### 6.16`
/// heading, not by any Ops clause-id.
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

/// Everything in a clause block before its `*Test:*` tag — so a content assertion (or a
/// backtick-token scan) is never contaminated by the backtick-quoted test-fn name in
/// the tag itself.
fn without_test(block: &str) -> &str {
    match block.find("*Test:*") {
        Some(i) => &block[..i],
        None => block,
    }
}

/// Collapse whitespace (healing line wraps) and rejoin hyphen-wrapped compounds so
/// `"emitter-\n  must-supply"` matches `"emitter-must-supply"`. Operates on the
/// pre-`*Test:*` body.
fn nbody(block: &str) -> String {
    without_test(block)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("- ", "-")
}

// ---------------------------------------------------------------------------
// Partition classifiers — a clause's OWN self-declared bucket and surface.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bucket {
    Driver,
    Emitter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Surface {
    Structural, // renders no target-language surface
    Bearing,    // does render a target-language surface
}

/// The partition bucket a clause places its decision in, from the exact placement
/// tokens `"driver-may-spell decision"` / `"emitter-must-supply decision"`. `None` if
/// the clause makes no unambiguous single placement.
fn clause_bucket(block: &str) -> Option<Bucket> {
    let b = nbody(block);
    let d = b.contains("driver-may-spell decision");
    let e = b.contains("emitter-must-supply decision");
    match (d, e) {
        (true, false) => Some(Bucket::Driver),
        (false, true) => Some(Bucket::Emitter),
        _ => None,
    }
}

/// A clause's surface self-declaration, from the exact phrases the §6.3 clauses use.
/// `None` when the clause does not self-declare (e.g. §6.3-0006 speaks in the plural
/// and §6.4-0004 leaves its surface attribute to the A.2 trace).
fn clause_surface(block: &str) -> Option<Surface> {
    let b = nbody(block).to_lowercase();
    let no = b.contains("renders no target-language surface");
    let yes = b.contains("does render a target-language surface");
    match (no, yes) {
        (true, false) => Some(Surface::Structural),
        (false, true) => Some(Surface::Bearing),
        _ => None,
    }
}

/// The set of §6.3 driver-side decisions that are SURFACE-BEARING, computed only from
/// each clause's own bucket + surface self-declaration (the same classifier the
/// §6.3-0006 partition test uses). At this schema version this is exactly {§6.3-0004}.
/// Returned as short ids ("6.3-0004") so no full sibling clause-id is materialized.
fn surface_bearing_driver(emit: &str) -> BTreeSet<String> {
    defined_clauses(emit, "KISS-EMIT-6.3-")
        .into_iter()
        .filter(|c| clause_bucket(clause_block(emit, c)) == Some(Bucket::Driver))
        .filter(|c| clause_surface(clause_block(emit, c)) == Some(Surface::Bearing))
        .map(|c| short(&c))
        .collect()
}

/// Strip the `KISS-EMIT-` prefix to a short section id, e.g. `"6.3-0004"`. The bare
/// prefix `"KISS-EMIT-"` is not a clause-id and cites nothing.
fn short(full_id: &str) -> String {
    full_id.trim_start_matches("KISS-EMIT-").to_string()
}

/// Every `<sec_prefix><4 digits>` short id mentioned in `text` (e.g. `sec_prefix =
/// "6.3-"` yields `{"6.3-0004"}`). Short ids, so nothing is reverse-cited.
fn mentioned_ids(text: &str, sec_prefix: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(i) = rest.find(sec_prefix) {
        let after = &rest[i + sec_prefix.len()..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == 4 {
            out.insert(format!("{sec_prefix}{digits}"));
        }
        rest = &rest[i + sec_prefix.len()..];
    }
    out
}

/// The alphanumeric backtick-quoted tokens in a clause/term body (its atom or dtype
/// list): every `` `tok` `` where `tok` is non-empty and all ASCII-alphanumeric. Scans
/// the pre-`*Test:*` body, so the backticked test-fn name, `const(bits)` (parens), and
/// the `+ - * /` operators are all excluded.
fn backtick_atoms(block: &str) -> BTreeSet<String> {
    let b = without_test(block);
    let mut out = BTreeSet::new();
    let mut rest = b;
    while let Some(i) = rest.find('`') {
        let after = &rest[i + 1..];
        match after.find('`') {
            Some(j) => {
                let tok = &after[..j];
                if !tok.is_empty() && tok.chars().all(|c| c.is_ascii_alphanumeric()) {
                    out.insert(tok.to_string());
                }
                rest = &after[j + 1..];
            }
            None => break,
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The independent second statement: the Appendix-A.2 partition-trace table.
// ---------------------------------------------------------------------------

/// One parsed A.2 trace row: the informative worked example's placement of a lowering
/// decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct A2Row {
    bucket: Bucket,
    surface: Surface,
    conditional: bool, // Set cell conditions the driver placement on the §6.5 audit
    audit_gated: bool, // Surface cell marks the surface audit-gated
}

/// Parse the Appendix-A.2 "partition trace" table (columns
/// `Lowering decision | Set | Surface? | Clause`) into a map keyed by the SHORT clause
/// id each row cites (e.g. `"6.3-0001"`). A row citing several clauses maps each to that
/// row's cells. This is the independent second authored statement of the same partition;
/// keying by short id keeps every sibling clause-id out of the source literals.
fn a2_trace(md: &str) -> BTreeMap<String, A2Row> {
    let sec = section_slice(md, "### A.2", "### A.3");
    let mut out = BTreeMap::new();
    for line in sec.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(|c| c.trim()).collect();
        if cells.len() != 4 {
            continue;
        }
        if cells[0].eq_ignore_ascii_case("Lowering decision") || cells[0].starts_with("---") {
            continue; // header / separator
        }
        let (set_cell, surf_cell, clause_cell) = (cells[1], cells[2], cells[3]);
        let bucket = if set_cell.contains("driver-may-spell") {
            Bucket::Driver
        } else if set_cell.contains("emitter-must-supply") {
            Bucket::Emitter
        } else {
            continue;
        };
        let surf_lc = surf_cell.to_lowercase();
        let surface = if surf_lc.starts_with("structural") {
            Surface::Structural
        } else if surf_lc.starts_with("surface") {
            Surface::Bearing
        } else {
            continue;
        };
        let row = A2Row {
            bucket,
            surface,
            conditional: set_cell.contains("only after") || set_cell.contains("§6.5"),
            audit_gated: surf_lc.contains("audit-gated"),
        };
        for sid in mentioned_ids(clause_cell, "6.3-")
            .into_iter()
            .chain(mentioned_ids(clause_cell, "6.4-"))
        {
            out.insert(sid, row);
        }
    }
    out
}

/// The one §6.4 clause whose (content-scanned) body satisfies `pred`, located WITHOUT a
/// clause-id source literal — `defined_clauses`/`clause_block` here take runtime values,
/// so the matched clause is not reverse-cited. Panics unless exactly one matches (a
/// drift that hid or duplicated the emitter-side home is itself a failure).
fn find_64_clause<'a>(emit: &'a str, pred: impl Fn(&str) -> bool) -> &'a str {
    let ids: Vec<String> = defined_clauses(emit, "KISS-EMIT-6.4-")
        .into_iter()
        .filter(|c| pred(clause_block(emit, c)))
        .collect();
    assert_eq!(
        ids.len(),
        1,
        "expected exactly one §6.4 clause matching the predicate, found {ids:?}"
    );
    clause_block(emit, &ids[0])
}

// ---------------------------------------------------------------------------
// §6.3 — the driver-may-spell set, cross-checked against the A.2 trace.
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.3-0001 — the op-DAG structure / topology walk MUST be a driver-may-spell,
/// no-surface (structural, audit-exempt) decision. Teeth: the normative clause's own
/// bucket + surface self-declaration must AGREE with the independent Appendix-A.2 trace
/// row that places topology (driver-may-spell / structural), and both must carry the
/// §6.2-0005 structural tie. A one-sided move of topology to the emitter side, or a drop
/// of the structural attribute, fails.
#[test]
fn test_emit_driver_spells_dag_topology() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.3-0001");
    let a2 = a2_trace(&emit);
    let row = *a2
        .get("6.3-0001")
        .expect("Appendix-A.2 trace has no row citing §6.3-0001");
    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Driver),
        "§6.3-0001: normative clause does not place topology on the driver-may-spell side"
    );
    assert_eq!(
        clause_surface(block),
        Some(Surface::Structural),
        "§6.3-0001: normative clause does not self-declare topology structural (no surface)"
    );
    assert!(
        nbody(block).contains("§6.2-0005"),
        "§6.3-0001: normative clause drops the §6.2-0005 structural / audit-exempt tie"
    );
    assert_eq!(
        row.bucket,
        Bucket::Driver,
        "§6.3-0001: A.2 trace places topology on the emitter side — one-sided drift vs the clause"
    );
    assert_eq!(
        row.surface,
        Surface::Structural,
        "§6.3-0001: A.2 trace marks topology surface-bearing — one-sided drift vs the clause"
    );
}

/// KISS-EMIT-6.3-0002 — operand binding in KISS-Classify canonical order MUST be a
/// driver-may-spell, no-surface (structural) decision. Teeth: the normative clause's
/// bucket + surface self-declaration must AGREE with the Appendix-A.2 'Bind (…) in
/// canonical order' row (driver-may-spell / structural). One-sided bucket/surface drift
/// fails.
#[test]
fn test_emit_driver_spells_operand_binding() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.3-0002");
    let a2 = a2_trace(&emit);
    let row = *a2
        .get("6.3-0002")
        .expect("Appendix-A.2 trace has no row citing §6.3-0002");
    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Driver),
        "§6.3-0002: normative clause does not place operand binding driver-may-spell"
    );
    assert_eq!(
        clause_surface(block),
        Some(Surface::Structural),
        "§6.3-0002: normative clause does not self-declare operand binding structural"
    );
    assert!(
        nbody(block).contains("§6.2-0005"),
        "§6.3-0002: normative clause drops the §6.2-0005 structural / audit-exempt tie"
    );
    assert_eq!(
        row.bucket,
        Bucket::Driver,
        "§6.3-0002: A.2 trace places operand binding on the emitter side — one-sided drift"
    );
    assert_eq!(
        row.surface,
        Surface::Structural,
        "§6.3-0002: A.2 trace marks operand binding surface-bearing — one-sided drift"
    );
}

/// KISS-EMIT-6.3-0003 — signed-stride index / address arithmetic (with the grid-stride
/// thread→element mapping) MUST be a driver-may-spell, no-surface (structural) decision.
/// Teeth: the normative clause's bucket + surface self-declaration must AGREE with the
/// Appendix-A.2 index-arithmetic row (which cites §6.3-0003 and §6.3-0005 together;
/// driver-may-spell / structural). One-sided bucket/surface drift fails.
#[test]
fn test_emit_driver_spells_index_arithmetic() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.3-0003");
    let a2 = a2_trace(&emit);
    let row = *a2
        .get("6.3-0003")
        .expect("Appendix-A.2 trace has no row citing §6.3-0003");
    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Driver),
        "§6.3-0003: normative clause does not place index arithmetic driver-may-spell"
    );
    assert_eq!(
        clause_surface(block),
        Some(Surface::Structural),
        "§6.3-0003: normative clause does not self-declare index arithmetic structural"
    );
    assert!(
        nbody(block).contains("§6.2-0005"),
        "§6.3-0003: normative clause drops the §6.2-0005 structural / audit-exempt tie"
    );
    assert_eq!(
        row.bucket,
        Bucket::Driver,
        "§6.3-0003: A.2 trace places index arithmetic on the emitter side — one-sided drift"
    );
    assert_eq!(
        row.surface,
        Surface::Structural,
        "§6.3-0003: A.2 trace marks index arithmetic surface-bearing — one-sided drift"
    );
}

/// KISS-EMIT-6.3-0004 — the strictly-universal infix operator spellings (`+ - * /`) are
/// the ONE driver-side decision that DOES render a target-language surface; the placement
/// is driver-may-spell only AFTER the §6.5 audit proves it universal, and defaults to
/// emitter-must-supply absent that proof. Teeth: independently compute the surface-bearing
/// §6.3 driver set (same classifier as §6.3-0006) — it must be exactly {§6.3-0004}; the
/// clause must gate the placement on §6.5 and default to emitter-must-supply; and the
/// Appendix-A.2 row must AGREE (driver, surface, conditional-on-§6.5, audit-gated). Making
/// the operator unconditionally driver-side, or reclassifying it no-surface, in either
/// statement fails.
#[test]
fn test_emit_infix_operators_gated_on_audit() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.3-0004");
    let b = nbody(block).to_lowercase();

    // §6.3-0004 is the UNIQUE surface-bearing driver decision.
    let sb = surface_bearing_driver(&emit);
    let mut expect = BTreeSet::new();
    expect.insert("6.3-0004".to_string());
    assert_eq!(
        sb, expect,
        "§6.3-0004: the surface-bearing driver set is not exactly {{§6.3-0004}} (found {sb:?})"
    );

    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Driver),
        "§6.3-0004: normative clause does not place the infix operators driver-may-spell"
    );
    assert_eq!(
        clause_surface(block),
        Some(Surface::Bearing),
        "§6.3-0004: normative clause does not self-declare the infix operators surface-bearing"
    );
    assert!(
        b.contains("§6.5") || b.contains("neutrality audit"),
        "§6.3-0004: the driver placement is not conditioned on the §6.5 audit"
    );
    assert!(
        b.contains("emitter-must-supply"),
        "§6.3-0004: absent the §6.5 proof the clause does not default to emitter-must-supply"
    );

    let row = *a2_trace(&emit)
        .get("6.3-0004")
        .expect("Appendix-A.2 trace has no row citing §6.3-0004");
    assert_eq!(
        row.bucket,
        Bucket::Driver,
        "§6.3-0004: A.2 trace does not place the infix operator driver-may-spell"
    );
    assert_eq!(
        row.surface,
        Surface::Bearing,
        "§6.3-0004: A.2 trace marks the infix operator structural — a surface token IS a surface"
    );
    assert!(
        row.conditional,
        "§6.3-0004: A.2 trace does not condition the driver placement on the §6.5 audit"
    );
    assert!(
        row.audit_gated,
        "§6.3-0004: A.2 trace does not mark the infix operator surface audit-gated"
    );
}

/// KISS-EMIT-6.3-0005 — control-flow / loop scaffolding derived from the Dispatch launch
/// model MUST be a driver-may-spell, no-surface (structural) decision. Teeth: the
/// normative clause's bucket + surface self-declaration must AGREE with the Appendix-A.2
/// index/grid row that also cites §6.3-0005 (driver-may-spell / structural). One-sided
/// bucket/surface drift fails.
#[test]
fn test_emit_driver_spells_control_flow() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.3-0005");
    let a2 = a2_trace(&emit);
    let row = *a2
        .get("6.3-0005")
        .expect("Appendix-A.2 trace has no row citing §6.3-0005");
    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Driver),
        "§6.3-0005: normative clause does not place control-flow scaffolding driver-may-spell"
    );
    assert_eq!(
        clause_surface(block),
        Some(Surface::Structural),
        "§6.3-0005: normative clause does not self-declare control-flow scaffolding structural"
    );
    assert!(
        nbody(block).contains("§6.2-0005"),
        "§6.3-0005: normative clause drops the §6.2-0005 structural / audit-exempt tie"
    );
    assert_eq!(
        row.bucket,
        Bucket::Driver,
        "§6.3-0005: A.2 trace places control-flow scaffolding on the emitter side — one-sided drift"
    );
    assert_eq!(
        row.surface,
        Surface::Structural,
        "§6.3-0005: A.2 trace marks control-flow scaffolding surface-bearing — one-sided drift"
    );
}

// ---------------------------------------------------------------------------
// §6.4 — the emitter-must-supply set, cross-checked against §3 and KISS-Ops §6.16.
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.4-0003 — transcendental / declared-ULP atom spelling (`exp`, `log`,
/// `sin`, `cos`, `sqrt`, `erf`, …) MUST be an emitter-must-supply decision, and the
/// neutral driver MUST NOT spell it as a mandated polynomial. Teeth: the atom set the
/// clause names must be non-trivial and a SUBSET of the INDEPENDENT §3 `Declared-ULP
/// atom` term-definition atom set (if the clause invents an atom §3 does not recognize,
/// the subset fails), the decision must be placed emitter-must-supply, and the mandated-
/// polynomial prohibition must be present.
#[test]
fn test_emit_transcendental_spelling_emitter_supplied() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.4-0003");

    let clause_atoms = backtick_atoms(block);
    let def_atoms = backtick_atoms(term_def(&emit, "Declared-ULP atom"));
    assert!(
        clause_atoms.len() >= 4,
        "§6.4-0003: parsed only {} atoms from the clause — parser drift, check would be vacuous",
        clause_atoms.len()
    );
    assert!(
        def_atoms.len() >= 4,
        "§6.4-0003: parsed only {} atoms from the §3 Declared-ULP definition — parser drift",
        def_atoms.len()
    );
    assert!(
        clause_atoms.is_subset(&def_atoms),
        "§6.4-0003: clause names atom(s) the §3 Declared-ULP definition does not recognize: {:?}",
        clause_atoms.difference(&def_atoms).collect::<Vec<_>>()
    );
    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Emitter),
        "§6.4-0003: transcendental atom spelling is not placed emitter-must-supply"
    );
    let b = nbody(block).to_lowercase();
    assert!(
        b.contains("must not") && b.contains("mandated polynomial"),
        "§6.4-0003: the clause does not forbid the driver from spelling a mandated polynomial"
    );
}

/// KISS-EMIT-6.4-0004 — target-specific type spelling (`f16`, `bf16`, `e4m3fn`, `e5m2`,
/// `c32`, `c64`) MUST be an emitter-must-supply decision. Teeth: the normative clause's
/// bucket must AGREE with the Appendix-A.2 'f32 type surface spelling' row (emitter-must-
/// supply / surface), AND every dtype the clause enumerates must be a REAL KISS-Ops §6.16
/// registry token (the registry read by its `### 6.16` heading, never by an Ops clause-id).
/// A phantom/renamed dtype no longer in the Ops registry, or a bucket drift from the A.2
/// placement, fails.
#[test]
fn test_emit_type_spelling_emitter_supplied() {
    let emit = read_spec("emit.md");
    let ops = read_spec("ops.md");
    let block = clause_block(&emit, "KISS-EMIT-6.4-0004");

    let named = backtick_atoms(block);
    let registry = dtype_tokens(&ops);
    assert!(
        registry.len() >= 12,
        "§6.4-0004: parsed only {} dtypes from KISS-Ops §6.16 — parser drift, check would be vacuous",
        registry.len()
    );
    assert!(
        named.len() >= 4,
        "§6.4-0004: parsed only {} dtypes from the clause — parser drift",
        named.len()
    );
    assert!(
        named.is_subset(&registry),
        "§6.4-0004: clause names dtype(s) not in the KISS-Ops §6.16 registry: {:?}",
        named.difference(&registry).collect::<Vec<_>>()
    );

    assert_eq!(
        clause_bucket(block),
        Some(Bucket::Emitter),
        "§6.4-0004: type spelling is not placed emitter-must-supply — drift vs the A.2 trace"
    );
    let row = *a2_trace(&emit)
        .get("6.4-0004")
        .expect("Appendix-A.2 trace has no row citing §6.4-0004");
    assert_eq!(
        row.bucket,
        Bucket::Emitter,
        "§6.4-0004: A.2 trace places type spelling driver-side — one-sided drift vs the clause"
    );
    assert_eq!(
        row.surface,
        Surface::Bearing,
        "§6.4-0004: A.2 trace marks type spelling structural — a type name IS a surface"
    );
}

// ---------------------------------------------------------------------------
// §6.5 — the pre-freeze neutrality-audit scope (freeze-gate governance).
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.5-0001 — the recorded neutrality audit MUST cover EVERY surface-bearing
/// driver-side decision (at this schema version, the §6.3-0004 infix operators), MUST
/// exclude the structural (§6.2-0005) decisions, and its recorded manifest MUST be a
/// freeze precondition. Teeth: the set of §6.3 ids the audit clause names is set-compared
/// against the INDEPENDENTLY-computed surface-bearing §6.3 driver set (same classifier as
/// §6.3-0006) — a new surface-bearing driver clause the audit scope failed to name, or a
/// spurious id it named, fails; the structural exclusion and the freeze-precondition
/// wording must also be present.
#[test]
fn test_emit_neutrality_audit_manifest_is_freeze_precondition() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.5-0001");
    let b = nbody(block).to_lowercase();

    // The audit's named §6.3 boundary must equal the surface-bearing driver set exactly.
    let named = mentioned_ids(&b, "6.3-");
    let sb = surface_bearing_driver(&emit);
    assert_eq!(
        named, sb,
        "§6.5-0001: audit scope names §6.3 ids {named:?} but the surface-bearing driver set is {sb:?}"
    );
    assert!(
        !sb.is_empty(),
        "§6.5-0001: surface-bearing driver set is empty — classifier drift, check would be vacuous"
    );

    // Structural decisions are explicitly outside the audit's scope.
    assert!(
        b.contains("§6.2-0005") && b.contains("structural") && b.contains("scope"),
        "§6.5-0001: the clause does not exclude structural (§6.2-0005) decisions from audit scope"
    );
    // The recorded manifest is a freeze precondition, and a freeze without it fails the gate.
    assert!(
        b.contains("recorded") && b.contains("freeze precondition"),
        "§6.5-0001: the clause does not pin the recorded manifest as a freeze precondition"
    );
    assert!(
        b.contains("fail") && b.contains("gate"),
        "§6.5-0001: the clause does not require a freeze without the recorded audit to fail the gate"
    );
}

/// KISS-EMIT-6.5-0002 — the recorded audit MUST move every surface-bearing driver-side
/// operator it could not prove target-independent to the emitter-must-supply sweep, and
/// the §6.3-0004 boundary is provisional until proven. Teeth: the set of §6.3 ids the
/// clause marks provisional is set-compared against the independently-computed surface-
/// bearing driver set, and the emitter-side sweep clause — located BY CONTENT (the §6.4
/// clause whose body speaks of the audit not proving an operator; never by clause-id) —
/// must actually assign emitter-must-supply. A provisional boundary that drifts from the
/// surface-bearing set, or a sweep target that stops assigning emitter-must-supply, fails.
#[test]
fn test_emit_audit_moves_unproven_spelling() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.5-0002");
    let b = nbody(block).to_lowercase();

    let named = mentioned_ids(&b, "6.3-");
    let sb = surface_bearing_driver(&emit);
    assert_eq!(
        named, sb,
        "§6.5-0002: provisional boundary names §6.3 ids {named:?} but the surface-bearing set is {sb:?}"
    );
    assert!(
        b.contains("provisional"),
        "§6.5-0002: the clause does not mark the §6.3-0004 boundary provisional until proven"
    );
    assert!(
        b.contains("emitter-must-supply"),
        "§6.5-0002: the clause does not move the unproven operator to emitter-must-supply"
    );

    // The emitter-side sweep home, by content: the §6.4 clause about the audit failing to
    // prove an operator universal. It must place its decision emitter-must-supply.
    let sweep = find_64_clause(&emit, |bl| {
        // strip markdown emphasis so a bolded `**not**` does not split `not prove`.
        let n = nbody(bl).to_lowercase().replace('*', "");
        n.contains("audit") && n.contains("not prove")
    });
    assert_eq!(
        clause_bucket(sweep),
        Some(Bucket::Emitter),
        "§6.5-0002: the unproven-operator sweep home does not assign emitter-must-supply — the routing target is incoherent"
    );
}

/// KISS-EMIT-6.5-0003 — the recorded audit MUST find every `const_lit` sibling on the
/// driver side and reclassify it to the emitter-must-supply constant home. Teeth: the
/// clause must require the const_lit sibling hunt and reclassification to emitter-must-
/// supply, and the constant-spelling home — located BY CONTENT (the §6.4 clause carrying
/// the **Constant spelling** decision label; never by clause-id) — must actually place
/// constant spelling emitter-side. If that home drifts driver-side the 'reclassify to
/// emitter' target is incoherent and this fails.
#[test]
fn test_emit_audit_hunts_const_lit_siblings() {
    let emit = read_spec("emit.md");
    let block = clause_block(&emit, "KISS-EMIT-6.5-0003");
    let b = nbody(block).to_lowercase();

    assert!(
        b.contains("const_lit"),
        "§6.5-0003: the clause does not name the const_lit sibling hunt"
    );
    assert!(
        b.contains("driver") && b.contains("emitter-must-supply"),
        "§6.5-0003: the clause does not reclassify driver-side const_lit siblings to emitter-must-supply"
    );

    // The constant-spelling home, by content: the §6.4 clause with the **Constant
    // spelling** decision label. It must place constant spelling emitter-side.
    let home = find_64_clause(&emit, |bl| bl.contains("**Constant spelling**"));
    assert_eq!(
        clause_bucket(home),
        Some(Bucket::Emitter),
        "§6.5-0003: the constant-spelling home does not assign emitter-must-supply — the reclassification target is incoherent"
    );
}
