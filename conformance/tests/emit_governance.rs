//! Third and final KISS-Emit conformance module — the spec-self-consistency /
//! meta-conformance clauses of §6.1 neutrality, §7 capability, §8 versioning, and
//! §9 conformance, continuing the #91 coverage burndown. Companion to
//! `emit_partition.rs` and `emit_membership.rs`, reusing the same
//! `read_spec()` / `clause_block()` / `defined_clauses()` doc-lint pattern.
//!
//! KISS-Emit ships **no** reference emitter and **no** wire surface in this crate
//! (`conformance/src/*` has no `emit.rs`), so the behavioral EMIT clauses (emitter
//! input/output, artifact ABI, round-trip, typed decline, freeze artifacts) genuinely
//! need an emitter/consumer that does not exist and are out of scope — they stay in
//! the ledger. This is the LAST honestly-backable EMIT doc-lint module: after it, the
//! remaining untested EMIT clauses are all behavioral, so the next probe hits the true
//! floor.
//!
//! What gives these six tests TEETH (not a phrase grep): each cross-checks TWO
//! INDEPENDENT representations of the same fact inside emit.md, so a one-sided edit
//! fails —
//!   * §9.2-0001: every clause body's `*Test:*` tag  vs  the §9 traceability MATRIX;
//!   * §9.1-0001: the §9.1 prerequisite set  vs  the §0 front-matter STRUCTURAL
//!     upstream-edge set (compared as sub-standard NAMES, never clause-ids);
//!   * §8.1-0001 / §8.1-0002: the §8 bump-rule TABLE (its two axis halves, plus each
//!     row's change-vs-preserve semantics)  vs  each clause's own enumeration;
//!   * §6.1-0005: a spec-grounded vendor/C-ism token set confined to informative
//!     regions and ABSENT from every normative clause body (two-sided);
//!   * §7.1-0001: the §7.1 mandatory-core enumeration  vs  the tier-2-negotiable
//!     declaration (located in §7 BY CONTENT) + §6.x section existence.
//!
//! Each test cites ONLY its own EMIT clause-id as a source literal (in its
//! doc-comment / assert messages / its own `clause_block(...)` call). Every
//! sibling or referenced clause is reached through runtime-enumerated
//! `defined_clauses()` (runtime strings, invisible to the traceability scanner) or
//! located BY CONTENT / by short `§6.x-nnnn` ref (which the `KISS-<SUB>-…-\d{4}`
//! citation regex does not match), and cross-standard references are compared as
//! sub-standard NAMES — so kiss_trace reverse-binds/over-credits nothing.
//!
//! Clauses bound: KISS-EMIT-6.1-0005, KISS-EMIT-7.1-0001, KISS-EMIT-8.1-0001,
//! KISS-EMIT-8.1-0002, KISS-EMIT-9.1-0001, KISS-EMIT-9.2-0001 (6 clauses).

use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a phrase.
// (Same shapes as emit_partition.rs / emit_membership.rs; integration-test files
// are separate crates, so the pattern is restated here rather than imported.)
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

/// Every clause id defined in `md` whose id begins with `prefix`. `prefix` is a bare
/// section stem such as `"KISS-EMIT-"` or `"KISS-EMIT-6.7-"`; because it carries no
/// 4-digit tail it is NOT itself a clause-id (the traceability scanner's
/// `KISS-<SUB>-…-\d{4}` pattern does not match it), so using it here reverse-cites
/// nothing. Matrix rows are excluded (they are not `**`-bold).
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

/// Collapse whitespace (healing line wraps) and rejoin hyphen-wrapped compounds so
/// `"round-\n  trip"` matches `"round-trip"`.
fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").replace("- ", "-")
}

/// Everything in a clause block before its `*Test:*` tag — so a token scan is never
/// contaminated by the backtick-quoted test-fn name in the tag itself.
fn without_test(block: &str) -> &str {
    match block.find("*Test:*") {
        Some(i) => &block[..i],
        None => block,
    }
}

/// The set of sub-standard NAMES (`KISS-Ops`, `KISS-Classify`, …) mentioned in
/// `text` — a `KISS-` followed by exactly one uppercase letter and a run of
/// lowercase. The all-caps clause-id anchor form (`KISS-EMIT-9.1-0001`) does NOT
/// match (its second letter is uppercase), so this extracts NAMES, never clause-ids.
fn kiss_names(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut i = 0;
    while let Some(p) = text[i..].find("KISS-") {
        let s = i + p + 5;
        let rest = text[s..].as_bytes();
        if !rest.is_empty() && rest[0].is_ascii_uppercase() {
            let mut end = 1;
            while end < rest.len() && rest[end].is_ascii_lowercase() {
                end += 1;
            }
            if end >= 2 {
                out.insert(format!("KISS-{}", &text[s..s + end]));
            }
        }
        i = s;
    }
    out
}

/// The STRUCTURAL upstream-dependency NAME set from the §0 front-matter
/// `| Upstream edges | … |` row: each `;`-separated entry carrying the exact
/// `**STRUCTURAL**` label contributes its sub-standard name. Read as NAMES from a
/// front-matter row, so no upstream sub-standard's clause is reverse-bound.
fn upstream_structural(md: &str) -> BTreeSet<String> {
    let line = md
        .lines()
        .find(|l| l.trim_start().starts_with("| Upstream edges |"))
        .expect("front-matter has no `Upstream edges` row");
    let cell = line.splitn(3, '|').nth(2).unwrap_or("");
    let mut out = BTreeSet::new();
    for seg in cell.split(';') {
        if seg.contains("**STRUCTURAL**") {
            out.extend(kiss_names(seg));
        }
    }
    out
}

/// Parse the §8 `| Change | Axis that moves |` bump-rule table into `(change, axis)`
/// pairs. Same `'|'`-delimited shape as the correspondence-table parser in
/// emit_consume_correspondence.rs.
fn abi_table_rows(md: &str) -> Vec<(String, String)> {
    let h = md
        .find("| Change | Axis that moves |")
        .expect("§8 bump-rule table header not found");
    let mut rows = Vec::new();
    for line in md[h..].lines().skip(1) {
        let t = line.trim();
        if !t.starts_with('|') {
            break; // blank line / end of table
        }
        if t.starts_with("|---") || t.starts_with("| ---") {
            continue; // markdown separator row
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(|c| c.trim()).collect();
        if cells.len() != 2 {
            break;
        }
        rows.push((cells[0].to_string(), cells[1].to_string()));
    }
    rows
}

/// The single most distinctive concept word of a table `Change` cell: strip backtick
/// spans and parentheticals (incidental illustrations), then take the longest
/// non-generic token (len ≥ 6), tie-broken lexicographically for determinism. This
/// is derived FROM the table row, so a newly-added bump row contributes its own
/// concept dynamically (rather than being matched against a hardcoded list).
fn longest_concept(change_cell: &str) -> Option<String> {
    let mut plain = String::new();
    let mut in_tick = false;
    let mut depth = 0i32;
    for c in change_cell.chars() {
        match c {
            '`' => in_tick = !in_tick,
            '(' if !in_tick => depth += 1,
            ')' if !in_tick => depth = (depth - 1).max(0),
            _ if in_tick || depth > 0 => {}
            _ => plain.push(c),
        }
    }
    let stop = ["change", "changes", "between", "preserve", "preserves"];
    plain
        .to_lowercase()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .map(|w| w.trim_matches('-'))
        .filter(|w| w.len() >= 6 && !stop.contains(w))
        .map(|w| w.to_string())
        .max_by_key(|w| (w.len(), w.clone()))
}

/// A table `Change` cell that reads as a PRESERVATION (a compatible change) rather
/// than an alteration — the semantic that pairs with the `crate semver only` axis.
fn is_preserving(change_cell: &str) -> bool {
    let l = change_cell.to_lowercase();
    l.contains("preserve") || l.contains("changes no")
}

/// The backtick-quoted test name in a clause block's `*Test:*` tag.
fn test_tag(block: &str) -> Option<String> {
    let i = block.find("*Test:*")?;
    let after = &block[i + "*Test:*".len()..];
    let a = after.find('`')?;
    let rest = &after[a + 1..];
    let b = rest.find('`')?;
    Some(rest[..b].to_string())
}

/// Parse the §9 `| Clause | Test |` traceability matrix into a `clause_id -> test`
/// map. An independent second authored representation of the clause→test mapping.
fn matrix_rows(md: &str) -> BTreeMap<String, String> {
    let h = md.find("| Clause | Test |").expect("§9 matrix header not found");
    let mut out = BTreeMap::new();
    for line in md[h..].lines().skip(1) {
        let t = line.trim();
        if !t.starts_with('|') {
            break;
        }
        if t.starts_with("|---") || t.starts_with("| ---") {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(|c| c.trim()).collect();
        if cells.len() != 2 {
            break;
        }
        let id = cells[0].to_string();
        if !id.starts_with("KISS-EMIT-") {
            break;
        }
        out.insert(id, cells[1].trim_matches('`').to_string());
    }
    out
}

/// The `§<major><digits>` section stems referenced in `text` (e.g. `major = "6."`
/// pulls `"6.1"` from `§6.1` and `"6.7"` from `§6.7-0001`). Short refs, not
/// clause-ids, so nothing is reverse-cited.
fn section_refs(text: &str, major: &str) -> BTreeSet<String> {
    let needle = format!("§{major}");
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(i) = rest.find(&needle) {
        let after = &rest[i + needle.len()..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            out.insert(format!("{major}{digits}"));
        }
        rest = &rest[i + needle.len()..];
    }
    out
}

// ---------------------------------------------------------------------------
// §6.1 — vendor/source-language neutrality
// ---------------------------------------------------------------------------

/// KISS-EMIT-6.1-0005 — KISS-Emit MUST NOT define, mandate, or bless a source
/// language or grammar; no normative clause pins a target syntax. Teeth: a
/// spec-grounded blocklist of the document's OWN informative examples of leaked/
/// blessed target-language surface (`Slang`, `CUDA`, `cuda:`, `sqrtf`, `0.5f`,
/// `C-family`, `baracuda-kernelgen`). Two-sided: (presence guard, non-vacuous) each
/// token MUST occur in an informative region — the overview (§0–§5) or the Appendix
/// — so an all-neutralized spec cannot make the check pass trivially; and (teeth)
/// NONE may occur inside ANY defined §6–§9 normative clause body. The moment a
/// normative clause is edited to spell a target language / intrinsic
/// (e.g. "emit `sqrtf` on CUDA"), the absence side fails. Blocklist grounding: §5 and
/// the closing note confine product/vendor names to non-normative text.
#[test]
fn test_emit_defines_no_source_language() {
    let emit = read_spec("emit.md");
    let blocklist = [
        "baracuda-kernelgen",
        "Slang",
        "CUDA",
        "cuda:",
        "sqrtf",
        "0.5f",
        "C-family",
    ];

    // The informative region = everything before the normative marker plus the
    // Appendix onward (both are informative per the document's own structure).
    let norm_start = emit
        .find("# NORMATIVE CONFORMANCE SPECIFICATION")
        .expect("KISS-EMIT-6.1-0005: normative-boundary marker not found");
    let appendix = emit
        .find("## Appendix A")
        .expect("KISS-EMIT-6.1-0005: Appendix marker not found");
    let informative = format!("{}{}", &emit[..norm_start], &emit[appendix..]);

    // Presence guard: the blocklist is not stale — each token still lives in an
    // informative region (else the absence check below would be vacuous).
    for tok in blocklist {
        assert!(
            informative.contains(tok),
            "KISS-EMIT-6.1-0005: blocklist token `{tok}` no longer appears in any informative region — the guard has gone stale"
        );
    }

    // Teeth: no normative §6–§9 clause body spells a vendor/source-language surface.
    let clauses = defined_clauses(&emit, "KISS-EMIT-");
    assert!(
        clauses.len() >= 40,
        "KISS-EMIT-6.1-0005: parsed only {} clause bodies — parser drift",
        clauses.len()
    );
    for id in &clauses {
        let body = without_test(clause_block(&emit, id));
        for tok in blocklist {
            assert!(
                !body.contains(tok),
                "KISS-EMIT-6.1-0005: normative clause {id} spells the target-language/vendor surface `{tok}` — a leaked source-language decision"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// §7 — capability / mandatory-core partition
// ---------------------------------------------------------------------------

/// KISS-EMIT-7.1-0001 — the mandatory core every conforming emitter MUST satisfy.
/// Teeth: (1) the tier-1 structural SELF-round-trip (§6.7-0001) is IN the §7.1 core
/// enumeration while the tier-2 numeric round-trip (§6.7-0002) is NOT — and the
/// tier-2-is-a-negotiable-option declaration, located in §7 BY CONTENT
/// (`negotiable option` co-located with the tier-2 ref, never `clause_block` by id),
/// confirms tier-2 non-core; promoting tier-2 into the core, or dropping tier-1,
/// fails. (2) every §6.x section the core cites (§6.1,§6.2,§6.3,§6.4,§6.6,§6.8) must
/// resolve to an actually-DEFINED clause-section, so a dangling §6.9 core reference
/// fails.
#[test]
fn test_emit_mandatory_core() {
    let emit = read_spec("emit.md");
    let block = norm(clause_block(&emit, "KISS-EMIT-7.1-0001")).to_lowercase();

    // (1) tier-1 self-round-trip in core; tier-2 numeric round-trip NOT in core.
    assert!(
        block.contains("mandatory core") && block.contains("self-round-trip"),
        "KISS-EMIT-7.1-0001: the clause is not the mandatory-core / self-round-trip statement"
    );
    assert!(
        block.contains("§6.7-0001"),
        "KISS-EMIT-7.1-0001: the core enumeration omits the tier-1 structural self-round-trip (§6.7-0001)"
    );
    assert!(
        !block.contains("§6.7-0002"),
        "KISS-EMIT-7.1-0001: the core enumeration wrongly includes the tier-2 numeric round-trip (§6.7-0002)"
    );

    // The tier-2-is-a-negotiable-option declaration, located in §7 BY CONTENT: a
    // "negotiable option" mention co-located (within a clause's worth of bytes) with
    // the tier-2 / §6.7-0002 reference. Whitespace is normalized first (the phrase is
    // line-wrapped in the source), and proximity is measured position-to-position so
    // no byte-window slice can split the multi-byte `§`.
    let s7 = norm(section_slice(&emit, "## 7.", "## 8.")).to_lowercase();
    let neg: Vec<isize> = s7.match_indices("negotiable option").map(|(i, _)| i as isize).collect();
    let refs: Vec<isize> = s7
        .match_indices("§6.7-0002")
        .chain(s7.match_indices("tier-2"))
        .map(|(i, _)| i as isize)
        .collect();
    let tier2_negotiable = neg.iter().any(|&n| refs.iter().any(|&r| (n - r).abs() < 500));
    assert!(
        tier2_negotiable,
        "KISS-EMIT-7.1-0001: §7 does not declare the tier-2 numeric round-trip a negotiable option — the tier-2-out-of-core invariant is unconfirmed"
    );

    // (2) every §6.x section the core cites resolves to a defined clause-section.
    let stems = section_refs(&block, "6.");
    assert!(
        stems.len() >= 5,
        "KISS-EMIT-7.1-0001: parsed only {} §6.x refs from the core enumeration — parser drift",
        stems.len()
    );
    for stem in &stems {
        let prefix = format!("KISS-EMIT-{stem}-");
        assert!(
            !defined_clauses(&emit, &prefix).is_empty(),
            "KISS-EMIT-7.1-0001: the core cites §{stem} but no clause-section §{stem} is defined"
        );
    }
}

// ---------------------------------------------------------------------------
// §8 — the EMIT_ABI_VERSION bump-rule table, partitioned across two clauses
// ---------------------------------------------------------------------------

/// KISS-EMIT-8.1-0001 — a change altering the normative-input schema, the partition
/// boundary, the declared-subset advertisement, or the round-trip tier wording MUST
/// bump `EMIT_ABI_VERSION`. Teeth: parse the §8 table; the concept set of the rows
/// whose axis moves `EMIT_ABI_VERSION` must EQUAL the concept set §8.1-0001 actually
/// enumerates (concepts derived dynamically from the table, then intersected with the
/// clause body). A bump-trigger row added to the table but not the clause, or a
/// trigger the clause enumerates that no bump row carries, breaks the set equality.
/// Plus each ABI-bump row must read as an ALTERATION (not a preservation), so a row
/// whose Change prose contradicts its bump axis fails.
#[test]
fn test_emit_abi_version_bump_rule() {
    let emit = read_spec("emit.md");
    let rows = abi_table_rows(&emit);
    assert!(
        rows.len() >= 5,
        "KISS-EMIT-8.1-0001: parsed only {} §8 table rows — parser drift",
        rows.len()
    );

    let mut bump = Vec::new();
    for (change, axis) in &rows {
        let is_bump = axis.contains("EMIT_ABI_VERSION");
        let is_crate = axis.to_lowercase().contains("crate semver only");
        assert!(
            is_bump ^ is_crate,
            "KISS-EMIT-8.1-0001: table row axis `{axis}` is neither exclusively ABI-bump nor crate-semver-only"
        );
        if is_bump {
            bump.push(change.clone());
        }
    }
    assert!(
        bump.len() >= 3,
        "KISS-EMIT-8.1-0001: only {} ABI-bump rows — check would be under-powered",
        bump.len()
    );

    // Concept vocabulary is derived from ALL rows; the bump-half concepts must be
    // exactly the concepts the clause body enumerates.
    let vocab: BTreeSet<String> = rows.iter().filter_map(|(c, _)| longest_concept(c)).collect();
    let k_bump: BTreeSet<String> = bump.iter().filter_map(|c| longest_concept(c)).collect();
    assert!(
        !k_bump.is_empty(),
        "KISS-EMIT-8.1-0001: extracted no bump concepts — extractor drift, check would be vacuous"
    );
    let clause = norm(clause_block(&emit, "KISS-EMIT-8.1-0001")).to_lowercase();
    let k_clause: BTreeSet<String> =
        vocab.iter().filter(|c| clause.contains(c.as_str())).cloned().collect();
    assert_eq!(
        k_bump, k_clause,
        "KISS-EMIT-8.1-0001: the §8 ABI-bump table concepts and the §8.1-0001 enumeration disagree"
    );

    assert!(
        clause.contains("must bump") && clause.contains("emit_abi_version"),
        "KISS-EMIT-8.1-0001: the clause does not mandate bumping EMIT_ABI_VERSION"
    );
    for c in &bump {
        assert!(
            !is_preserving(c),
            "KISS-EMIT-8.1-0001: ABI-bump row `{c}` reads as a preservation — Change prose contradicts its bump axis"
        );
    }
}

/// KISS-EMIT-8.1-0002 — a change PRESERVING the input schema, partition boundary,
/// declared-subset advertisement, and round-trip tier wording MUST NOT bump
/// `EMIT_ABI_VERSION` and bumps only the crate semver. Complement of §8.1-0001:
/// collect the §8 rows whose axis is `crate semver only` and assert (a) §8.1-0002
/// declares the MUST-NOT-bump / crate-semver-only verdict, (b) each such row's concept
/// is acknowledged in the clause, and (c) each such row reads as a PRESERVATION —
/// matching its crate-semver axis. Together with §8.1-0001 this partitions the whole
/// table; moving a row between the two axis halves without its Change prose following
/// (a preserve-row on the bump side, or an alter-row on the crate-semver side) fails.
#[test]
fn test_emit_no_abi_bump_on_compatible_change() {
    let emit = read_spec("emit.md");
    let rows = abi_table_rows(&emit);

    let mut crate_only = Vec::new();
    for (change, axis) in &rows {
        let is_bump = axis.contains("EMIT_ABI_VERSION");
        let is_crate = axis.to_lowercase().contains("crate semver only");
        assert!(
            is_bump ^ is_crate,
            "KISS-EMIT-8.1-0002: table row axis `{axis}` is neither exclusively ABI-bump nor crate-semver-only"
        );
        if is_crate {
            crate_only.push(change.clone());
        }
    }
    assert!(
        crate_only.len() >= 2,
        "KISS-EMIT-8.1-0002: only {} crate-semver-only rows — check would be under-powered",
        crate_only.len()
    );

    let clause = norm(clause_block(&emit, "KISS-EMIT-8.1-0002")).to_lowercase();
    assert!(
        clause.contains("must not bump") && clause.contains("crate semver"),
        "KISS-EMIT-8.1-0002: the clause does not mandate the no-bump / crate-semver-only verdict"
    );
    for c in &crate_only {
        if let Some(k) = longest_concept(c) {
            assert!(
                clause.contains(&k),
                "KISS-EMIT-8.1-0002: the clause does not acknowledge the crate-semver-only concept `{k}`"
            );
        }
        assert!(
            is_preserving(c),
            "KISS-EMIT-8.1-0002: crate-semver-only row `{c}` does not read as a preservation — Change prose contradicts its axis"
        );
    }
}

// ---------------------------------------------------------------------------
// §9 — conformance: prerequisite closure and traceability
// ---------------------------------------------------------------------------

/// KISS-EMIT-9.1-0001 — a KISS-Emit conformance claim MUST be prerequisite-closed over
/// KISS-Emit's incoming STRUCTURAL edges: it MUST include a claim to KISS-Ops,
/// KISS-Classify, and KISS-Contract. Teeth: the prerequisite NAME set the §9.1-0001
/// clause lists (minus KISS-Emit itself) must EQUAL the STRUCTURAL upstream-edge NAME
/// set independently parsed from the §0 front-matter `Upstream edges` row. Adding or
/// removing a structural upstream dep on ONE side breaks the set equality; the sibling
/// KISS-Consume and the non-upstream KISS-Synth are correctly excluded (neither is a
/// STRUCTURAL front-matter upstream edge). Compared as sub-standard NAMES, so no
/// upstream standard's clause is read.
#[test]
fn test_emit_claim_prerequisite_closed() {
    let emit = read_spec("emit.md");

    let mut prereq = kiss_names(clause_block(&emit, "KISS-EMIT-9.1-0001"));
    prereq.remove("KISS-Emit"); // self is not its own prerequisite

    let upstream = upstream_structural(&emit);

    assert_eq!(
        prereq.len(),
        3,
        "KISS-EMIT-9.1-0001: expected exactly 3 prerequisite sub-standards, found {prereq:?}"
    );
    assert_eq!(
        prereq, upstream,
        "KISS-EMIT-9.1-0001: the §9.1 prerequisite set {prereq:?} and the front-matter STRUCTURAL upstream set {upstream:?} disagree"
    );
    // The DAG sibling and the non-upstream provisioner must NOT be prerequisites.
    assert!(
        !prereq.contains("KISS-Consume") && !prereq.contains("KISS-Synth"),
        "KISS-EMIT-9.1-0001: a non-STRUCTURAL-upstream sub-standard leaked into the prerequisite set {prereq:?}"
    );
}

/// KISS-EMIT-9.2-0001 — every normative clause maps to at least one named test and
/// each test cites its clause (bidirectional traceability), kept in sync by the
/// KISS-Conform lint. Teeth: build two INDEPENDENT clause→test maps from emit.md — one
/// from every defined clause body's own `*Test:*` tag, the other from the §9
/// traceability MATRIX table — and assert they are IDENTICAL. A clause added/renamed
/// without a matching matrix row, a matrix row for a clause that no longer exists, or
/// ANY test-name mismatch between a clause's tag and its matrix row, breaks the map
/// equality. Both maps guarded non-empty (≥40) so it cannot go vacuous.
#[test]
fn test_emit_traceability_complete() {
    let emit = read_spec("emit.md");

    // Representation 1: each defined clause body's own *Test:* tag.
    let mut by_body: BTreeMap<String, String> = BTreeMap::new();
    for id in defined_clauses(&emit, "KISS-EMIT-") {
        let tag = test_tag(clause_block(&emit, &id))
            .unwrap_or_else(|| panic!("KISS-EMIT-9.2-0001: clause {id} has no *Test:* tag"));
        by_body.insert(id, tag);
    }

    // Representation 2: the §9 clause→test traceability matrix.
    let by_matrix = matrix_rows(&emit);

    assert!(
        by_body.len() >= 40 && by_matrix.len() >= 40,
        "KISS-EMIT-9.2-0001: parsed {} clause bodies and {} matrix rows — parser drift, check would be under-powered",
        by_body.len(),
        by_matrix.len()
    );
    assert_eq!(
        by_body, by_matrix,
        "KISS-EMIT-9.2-0001: the clause-body *Test:* tags and the §9 traceability matrix disagree"
    );
}
