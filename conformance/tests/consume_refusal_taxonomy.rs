//! KISS-CONSUME refusal-taxonomy & version-axis structural conformance module — the
//! THIRD CONSUME doc-lint module of the #91 coverage burndown, continuing the discipline
//! established in `tests/consume_taxonomy.rs` (and the EMIT `emit_partition` /
//! `emit_membership` siblings). KISS-Consume ships **no** reference lifter and **no**
//! input-structure-graph codec in this crate; its only crate codec
//! (`kiss_conformance::decline`) already backs §6.4-0009/-0010 in `tests/decline.rs`. So
//! every remaining CHEAP clause here is backed by extracting a STRUCTURAL SET (a byte-exact
//! backtick token set, an ordered step procedure, a decline/residue partition, a fixed
//! schema triple, a version-axis surface set) from the normative text and set-comparing it
//! against a canonical reference constant or a cross-clause set read BY CONTENT, so the
//! test FAILS on a plausible drift (a renamed/dropped category, a category crossing the
//! decline↔residue partition or its phase, a residue-schema field lost, lift-coverage
//! wrongly promoted onto the schema-version surface, a membership-polarity flip).
//!
//! The BEHAVIORAL heart of each refusal step (the actual classifier: zero-op-node
//! detection, op-family compatibility, expressibility-oracle membership, never-panic on
//! malformed graphs) genuinely needs a lifter + the KISS-Conform expressibility oracle and
//! is left `untested` in UNBACKED.tsv. This module backs only the structural/partition
//! SHAPE, not the classifier.
//!
//! Clauses bound by THIS module (9):
//!   * KISS-CONSUME-6.1-0004  input-structure-graph node record (3 prose fields + serialization)
//!   * KISS-CONSUME-6.3-0008  residue-entry fixed triple + sub-field authority cross-refs
//!   * KISS-CONSUME-6.4-0003  not-a-kernel  @ (step 1, whole-input) ∈ decline half
//!   * KISS-CONSUME-6.4-0004  wrong-op-class @ (step 2, whole-input) ∈ decline half, no inlined table
//!   * KISS-CONSUME-6.4-0005  unrecognized-but-expressible @ (step 3, per-region) ∈ residue half, +membership
//!   * KISS-CONSUME-6.4-0006  inexpressible-residue @ (step 4, per-region) ∈ residue half, −membership (complement)
//!   * KISS-CONSUME-6.4-0007  anti-sniffing category list = the four-category set + §6.1 anchor
//!   * KISS-CONSUME-8.1-0001  schema-version surface = {§6.1,§6.3,§6.4,§6.6}, excludes §6.2 lift-coverage
//!   * KISS-CONSUME-8.1-0002  residue op-set-relativity boundary pair = §6.3-0004 subset
//!
//! CITATION DISCIPLINE (kiss_trace reverse-binds a clause when its full `KISS-CONSUME-…`
//! id literal appears anywhere in a test's scope): every `#[test]` writes ONLY its own
//! KISS-CONSUME-* id in its doc-comment and body. Sibling facts — the four-category set
//! (§6.4-0001, still `lint`-backed), the §6.4-0008 decline/residue partition, the
//! §6.3-0004 two-token residue subset, the §8.1 intro Schema-version surface, the ordered
//! step headings — are read via `four_category_set` / `decline_residue_partition` /
//! `residue_subset` / `step_headings` / `step_clause` / `schema_version_surface` by CONTENT
//! ANCHOR, never by a sibling's full clause-id literal (that would over-credit a clause
//! this module does not back). The spec clause bodies reference siblings only in §-ref form
//! ("(§6.4-0008)"), never as the "KISS-CONSUME-…" literal, so those runtime-loaded strings
//! never leak an id into a test's source scope. Helper fns live outside every `#[test]`
//! scope, so their text is attributed to no clause.

use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Spec-reading helpers — parse structure, never grep a bare phrase for its own sake.
// (Copied from the sibling doc-lint module; each integration test compiles as its own
//  crate, so there is no cross-file symbol collision.)
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
/// of the §6.4 section, never by their clause ids). `(` delimits the phase; a leading
/// em-dash and spaces are trimmed by ascii class so no em-dash literal is relied on.
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
            .trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .to_string();
        let rest = &after[lp + 1..];
        let rp = rest.find(')').expect("unterminated `(<phase>)` in step heading");
        out.push((n, token, rest[..rp].trim().to_string()));
    }
    out
}

/// The plain-text body of §6.4 **Step `n`** (§6.4-0002+`n`), read BY CONTENT from the
/// "Step n" heading up to that step clause's `*Test:*` tag (its own end marker) — never
/// by the sibling clause id. Used to read the expressibility-oracle membership polarity
/// of steps 3 and 4 without citing 6.4-0005/-0006.
fn step_clause(consume: &str, n: u32) -> String {
    let sec = plain(section_slice(consume, "### 6.4", "### 6.5"));
    let key = format!("Step {n}");
    let i = sec
        .find(&key)
        .unwrap_or_else(|| panic!("§6.4: `{key}` clause not found"));
    let after = &sec[i..];
    let end = after.find("Test:").unwrap_or(after.len());
    after[..end].to_string()
}

/// Whether a per-region step clause keys on POSITIVE oracle membership (region signature
/// IS "a member of" the decomposable set) vs NEGATIVE ("not a member of"). The single
/// decidable membership test both steps share; `true` = expressible side, `false` =
/// inexpressible side. Panics if the membership discriminator is absent (a drift off the
/// decidable test entirely fails the caller).
fn membership_polarity(step_clause_plain: &str) -> bool {
    assert!(
        step_clause_plain.contains("member of"),
        "step clause dropped the `member of` oracle-membership discriminator"
    );
    !step_clause_plain.contains("not a member")
}

/// The §6.4-0008 decline/residue partition, RE-DERIVED BY CONTENT (anchor "The categories
/// partition into"), never by the 6.4-0008 clause id, which would reverse-bind an
/// already-backed sibling. Each of the four categories (read by content) is classified
/// into the whole-input-decline half (before the ` while ` connector) or the residue-tag
/// half (after ` while `, before the trailing ` Because `). Returns `(declines, residues)`.
fn decline_residue_partition(consume: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let sec = plain(section_slice(consume, "### 6.4", "### 6.5"));
    let i = sec
        .find("The categories partition into")
        .expect("§6.4 decline/residue partition sentence not found");
    let clause = &sec[i..];
    let clause = &clause[..clause.find("Test:").unwrap_or(clause.len())];
    let widx = clause
        .find(" while ")
        .expect("§6.4 partition ` while ` connector not found");
    let bidx = clause.find(" Because ").unwrap_or(clause.len());
    let before = &clause[..widx];
    let after = &clause[widx..bidx];
    let four = four_category_set(consume);
    let declines = four.iter().filter(|t| before.contains(t.as_str())).cloned().collect();
    let residues = four.iter().filter(|t| after.contains(t.as_str())).cloned().collect();
    (declines, residues)
}

/// The §6.3-0004 residue-bearing two-token subset, read BY CONTENT (anchor "residue-bearing
/// subset" within §6.3, ending at "spelled byte-exact"), never by the 6.3-0004 clause id.
/// Expected: {`unrecognized-but-expressible`, `inexpressible-residue`}.
fn residue_subset(consume: &str) -> BTreeSet<String> {
    let sec = section_slice(consume, "### 6.3", "### 6.4");
    let flat: String = sec.split_whitespace().collect::<Vec<_>>().join(" ");
    let a = flat
        .find("residue-bearing subset")
        .expect("§6.3-0004 `residue-bearing subset` phrase not found");
    let after = &flat[a..];
    let end = after.find("spelled byte-exact").unwrap_or(after.len());
    backtick_tokens(&after[..end]).into_iter().collect()
}

/// The §8.1 intro **Schema version** bullet's behavior-surface §-ref set, read BY CONTENT
/// (the bullet between "Schema version" and "Published-crate semver" inside §8.1..§8.2).
/// This is the observable behavior surface the schema version stamps.
fn schema_version_surface(consume: &str) -> BTreeSet<String> {
    let s81 = section_slice(consume, "### 8.1", "### 8.2");
    let bullet = section_slice(s81, "Schema version", "Published-crate semver");
    section_refs(bullet)
}

/// Strip a member of a prose/enum field list down to its bare name: drop markdown
/// emphasis, `(§…)` parentheticals and any non `[A-Za-z0-9- ]` char (em-dash, colon),
/// heal whitespace, and remove a leading conjunction `and `.
fn clean_field(s: &str) -> String {
    let kept: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == ' ' { c } else { ' ' })
        .collect();
    let t = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    t.trim().trim_start_matches("and ").trim().to_string()
}

/// The three fields of the §6.3-0008 residue-entry `{…}` triple literal, each cleaned of
/// markdown emphasis and its `(§…)` authority parenthetical. Expected 3-set
/// {region identifier, category token, determining op-set version}.
fn triple_fields(block: &str) -> BTreeSet<String> {
    let open = block.find('{').expect("§6.3-0008: no `{` triple literal in clause block");
    let close = block[open..].find('}').expect("§6.3-0008: unterminated `{` triple") + open;
    block[open + 1..close]
        .split(',')
        .map(|member| {
            // Drop any `(…)` authority parenthetical (`(§6.3-0004)` etc.) before cleaning,
            // so only the bare field name survives.
            let mut bare = String::new();
            let mut depth = 0i32;
            for c in member.chars() {
                match c {
                    '(' => depth += 1,
                    ')' => depth = (depth - 1).max(0),
                    _ if depth == 0 => bare.push(c),
                    _ => {}
                }
            }
            clean_field(&bare)
        })
        .filter(|f| !f.is_empty())
        .collect()
}

/// The three node-record fields of the §6.1-0004 input-structure-graph node, extracted from
/// the prose list "each an operation-kind field, a canonically-ordered operand-edge list,
/// and an opaque attribute blob" (between "operation records" and "and whose edges").
fn node_record_fields(consume: &str) -> BTreeSet<String> {
    let block = plain(clause_block(consume, "KISS-CONSUME-6.1-0004"));
    let i = block
        .find("operation records")
        .expect("§6.1-0004: `operation records` node-list anchor not found");
    let after = &block[i..];
    let end = after
        .find("and whose edges")
        .expect("§6.1-0004: node-record list terminator (`and whose edges`) not found");
    let list = &after[..end];
    let le = list
        .find("each ")
        .map(|p| p + "each ".len())
        .expect("§6.1-0004: `each` node-list intro not found");
    list[le..]
        .split(',')
        .map(clean_field)
        .filter(|f| !f.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// §6.1 — the input structure graph
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.1-0004 — the conformance input a lifter consumes is the syntax-free
/// input structure graph, whose node record has exactly three fields: an operation-kind
/// field, a canonically-ordered operand-edge list, and an opaque attribute blob; golden
/// vectors are authored with a byte-exact canonical little-endian serialization, and
/// recognition is a function of this graph ALONE (§6.1-0001). Teeth: the prose 3-field node
/// record extracted from the clause must equal the canonical 3-set (dropping/renaming a
/// node field fails), the clause must pin the byte-exact canonical little-endian
/// serialization discipline (the same discipline the residue-entry schema pins,
/// cross-consistency), and it must carry the §6.1 recognition-is-a-function-of-this-graph
/// anchor via `section_refs`. MODERATE CONFIDENCE: the 3-set is a prose-list split, not a
/// `{…}` literal — harden the split if the clause is reworded.
#[test]
fn test_consume_input_structure_graph() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.1-0004");

    let fields = node_record_fields(&consume);
    assert_eq!(
        fields,
        sset(&[
            "an operation-kind field",
            "a canonically-ordered operand-edge list",
            "an opaque attribute blob",
        ]),
        "KISS-CONSUME-6.1-0004: the input-structure-graph node record drifted from its three fields"
    );
    assert_eq!(fields.len(), 3, "KISS-CONSUME-6.1-0004: node record must have exactly 3 fields");

    let p = plain(block);
    assert!(
        p.contains("byte-exact canonical little-endian"),
        "KISS-CONSUME-6.1-0004: the clause does not pin a byte-exact canonical little-endian serialization"
    );
    assert!(
        section_refs(block).contains("§6.1"),
        "KISS-CONSUME-6.1-0004: the clause dropped the §6.1 `recognition is a function of it alone` anchor"
    );
}

// ---------------------------------------------------------------------------
// §6.3 — the residue-entry schema
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.3-0008 — each `lift_residue` entry is the FIXED TRIPLE {region identifier,
/// category token, determining op-set version} with a byte-exact canonical little-endian
/// serialization; the region identifier is one residue region per maximal connected
/// component of un-lifted operation nodes in the input structure graph. Teeth: the triple
/// parsed from the clause's `{…}` literal must equal the canonical 3-field set (dropping a
/// field, adding a 4th, or renaming one fails), AND the clause must cross-reference the
/// category-token authority + op-set-version authority (§6.3 sub-fields) and the
/// input-structure-graph the maximal-connected-component rule is defined over (§6.1), via
/// `section_refs` — severing a sub-field authority or the connected-component definition
/// fails.
#[test]
fn test_consume_residue_entry_schema() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.3-0008");

    let triple = triple_fields(block);
    assert_eq!(
        triple,
        sset(&["region identifier", "category token", "determining op-set version"]),
        "KISS-CONSUME-6.3-0008: the residue-entry fixed triple drifted from its three fields"
    );
    assert_eq!(triple.len(), 3, "KISS-CONSUME-6.3-0008: the residue entry must be a 3-field triple");

    let refs = section_refs(block);
    assert!(
        refs.contains("§6.3"),
        "KISS-CONSUME-6.3-0008: lost the §6.3 category-token / op-set-version sub-field authority cross-refs"
    );
    assert!(
        refs.contains("§6.1"),
        "KISS-CONSUME-6.3-0008: lost the §6.1 input-structure-graph anchor the connected-component rule is defined over"
    );
    assert!(
        plain(block).contains("maximal connected component"),
        "KISS-CONSUME-6.3-0008: the region-identifier's maximal-connected-component partitioning rule is missing"
    );
}

// ---------------------------------------------------------------------------
// §6.4 — the refusal taxonomy: per-step structure + partition binding
// ---------------------------------------------------------------------------

/// KISS-CONSUME-6.4-0003 — Step 1, not-a-kernel, whole-input: the input has zero operation
/// nodes, decided from the input alone before any op-class question. Teeth: triangulate
/// three independent spec locations that must agree — the ordered `Step N` headings (read
/// by content) must place not-a-kernel at (step 1, phase whole-input), AND not-a-kernel
/// must land in the whole-input-DECLINE half of the §6.4-0008 partition (re-derived by
/// content) and be ABSENT from the residue-tag half. A drift that moved not-a-kernel to a
/// residue tag, to per-region phase, or off step 1 breaks the intersection and fails.
/// Distinct teeth from the 6.4-0002 ordered-sequence check and the 6.4-0008 partition check.
#[test]
fn test_consume_refusal_not_a_kernel() {
    let consume = read_spec("consume.md");
    let steps = step_headings(&consume);
    assert_eq!(
        steps[0],
        (1, "not-a-kernel".to_string(), "whole-input".to_string()),
        "KISS-CONSUME-6.4-0003: not-a-kernel is not at (step 1, whole-input)"
    );

    let (declines, residues) = decline_residue_partition(&consume);
    assert!(
        declines.contains("not-a-kernel"),
        "KISS-CONSUME-6.4-0003: not-a-kernel is not in the whole-input-decline half of the partition"
    );
    assert!(
        !residues.contains("not-a-kernel"),
        "KISS-CONSUME-6.4-0003: not-a-kernel wrongly appears in the residue-tag half of the partition"
    );
}

/// KISS-CONSUME-6.4-0004 — Step 2, wrong-op-class, whole-input: the input is op-bearing and
/// a request cell is supplied, but the lifted root op's KISS-Ops op-family is incompatible
/// with the request cell's Classify op-category per the KISS-Contract Identity consistency
/// relation, which MUST be referenced, not re-forked. Teeth: wrong-op-class must be at
/// (step 2, whole-input) per the step headings AND in the whole-input-DECLINE half of the
/// §6.4-0008 partition, absent from the residue half — so reclassifying it as a residue
/// entry fails. Additionally the clause must CROSS-REFERENCE the upstream compatibility
/// relation (a §6.3 ref) and MUST NOT inline the table: no `{…}` compatibility-table literal
/// may appear in the clause body — catching a re-fork of the table the clause forbids
/// restating.
#[test]
fn test_consume_refusal_wrong_op_class() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.4-0004");

    let steps = step_headings(&consume);
    assert_eq!(
        steps[1],
        (2, "wrong-op-class".to_string(), "whole-input".to_string()),
        "KISS-CONSUME-6.4-0004: wrong-op-class is not at (step 2, whole-input)"
    );

    let (declines, residues) = decline_residue_partition(&consume);
    assert!(
        declines.contains("wrong-op-class"),
        "KISS-CONSUME-6.4-0004: wrong-op-class is not in the whole-input-decline half of the partition"
    );
    assert!(
        !residues.contains("wrong-op-class"),
        "KISS-CONSUME-6.4-0004: wrong-op-class wrongly appears in the residue-tag half of the partition"
    );

    assert!(
        section_refs(block).contains("§6.3"),
        "KISS-CONSUME-6.4-0004: the clause does not cross-reference the upstream Contract compatibility relation"
    );
    assert!(
        !block.contains('{'),
        "KISS-CONSUME-6.4-0004: a `{{…}}` compatibility-table literal was inlined — the clause MUST reference, not re-fork, the table"
    );
}

/// KISS-CONSUME-6.4-0005 — Step 3, unrecognized-but-expressible, per-region: an un-lifted
/// residue region the expressibility oracle judges expressible — its region signature IS a
/// member of the enumerated KISS-Ops-decomposable set — though this lifter emitted no node.
/// Teeth: it must be at (step 3, per-region) per the step headings, in the residue-TAG half
/// of the §6.4-0008 partition (absent from the decline half), a member of the §6.3-0004
/// residue-bearing two-token subset (read by content), and its discriminator must be
/// POSITIVE oracle membership (`member of`, not negated). A drift promoting it to a
/// whole-input decline, or flipping its membership polarity, fails.
#[test]
fn test_consume_refusal_unrecognized_but_expressible() {
    let consume = read_spec("consume.md");
    let tok = "unrecognized-but-expressible";

    let steps = step_headings(&consume);
    assert_eq!(
        steps[2],
        (3, tok.to_string(), "per-region".to_string()),
        "KISS-CONSUME-6.4-0005: unrecognized-but-expressible is not at (step 3, per-region)"
    );

    let (declines, residues) = decline_residue_partition(&consume);
    assert!(
        residues.contains(tok) && !declines.contains(tok),
        "KISS-CONSUME-6.4-0005: unrecognized-but-expressible is not exclusively in the residue-tag half"
    );

    let subset = residue_subset(&consume);
    assert_eq!(subset.len(), 2, "KISS-CONSUME-6.4-0005: residue-bearing subset is not the two-token set");
    assert!(
        subset.contains(tok),
        "KISS-CONSUME-6.4-0005: unrecognized-but-expressible is not in the §6.3-0004 residue-bearing subset"
    );

    let s3 = step_clause(&consume, 3);
    assert!(
        membership_polarity(&s3),
        "KISS-CONSUME-6.4-0005: step 3 does not key on POSITIVE expressibility-oracle membership"
    );
    assert!(
        s3.contains("decomposable"),
        "KISS-CONSUME-6.4-0005: step 3 does not name the KISS-Ops-decomposable oracle set"
    );
}

/// KISS-CONSUME-6.4-0006 — Step 4, inexpressible-residue, per-region: the truly un-liftable
/// remainder — a region signature the SAME oracle judges NOT a member of the decomposable
/// set at the current op set. Teeth: it must be at (step 4, per-region) per the step
/// headings, in the residue-TAG half of the §6.4-0008 partition (absent from the decline
/// half), and a member of the §6.3-0004 residue-bearing subset. Its discriminator must be
/// the COMPLEMENT of step 3: both step-3 and step-4 clause bodies (read by content) name the
/// same decomposable oracle set and take OPPOSITE membership polarity — so a drift that made
/// them key on two different discriminators (no longer a single decidable membership test)
/// fails.
#[test]
fn test_consume_refusal_inexpressible_residue() {
    let consume = read_spec("consume.md");
    let tok = "inexpressible-residue";

    let steps = step_headings(&consume);
    assert_eq!(
        steps[3],
        (4, tok.to_string(), "per-region".to_string()),
        "KISS-CONSUME-6.4-0006: inexpressible-residue is not at (step 4, per-region)"
    );

    let (declines, residues) = decline_residue_partition(&consume);
    assert!(
        residues.contains(tok) && !declines.contains(tok),
        "KISS-CONSUME-6.4-0006: inexpressible-residue is not exclusively in the residue-tag half"
    );

    let subset = residue_subset(&consume);
    assert!(
        subset.contains(tok),
        "KISS-CONSUME-6.4-0006: inexpressible-residue is not in the §6.3-0004 residue-bearing subset"
    );

    // Complementarity: steps 3 and 4 key on the SAME decomposable oracle set, opposite polarity.
    let s3 = step_clause(&consume, 3);
    let s4 = step_clause(&consume, 4);
    assert!(
        s3.contains("decomposable") && s4.contains("decomposable"),
        "KISS-CONSUME-6.4-0006: steps 3 and 4 do not both name the KISS-Ops-decomposable oracle set"
    );
    assert!(
        !membership_polarity(&s4),
        "KISS-CONSUME-6.4-0006: step 4 does not key on NEGATIVE (not-a-member) oracle membership"
    );
    assert_ne!(
        membership_polarity(&s3),
        membership_polarity(&s4),
        "KISS-CONSUME-6.4-0006: steps 3 and 4 do not take OPPOSITE polarity on the single decidable membership test"
    );
}

/// KISS-CONSUME-6.4-0007 — category assignment MUST itself be structure-based (§6.1): an
/// implementation MUST NOT decide any of the four categories by substring, keyword, symbol,
/// or comment sniffing. Teeth: the anti-sniffing sentence must enumerate ALL FOUR category
/// tokens; the backtick set extracted from the clause body (before its `*Test:*` tag) must
/// equal the canonical four-category set read BY CONTENT from the §6.4-0001 owner sentence,
/// so dropping/renaming a category from the structure-based list fails. It must also
/// reference §6.1 (the structure-based recognition anchor it inherits) — if the anti-sniffing
/// rule is decoupled from §6.1 recognition, fails.
#[test]
fn test_consume_taxonomy_structure_based() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-6.4-0007");
    // Exclude the `*Test:*` tag's backticked fn name from the category token set.
    let body = &block[..block.find("*Test:*").unwrap_or(block.len())];

    let listed: BTreeSet<String> = backtick_tokens(body).into_iter().collect();
    let four = four_category_set(&consume);
    assert_eq!(four.len(), 4, "KISS-CONSUME-6.4-0007: expected 4 refusal categories, got {four:?}");
    assert_eq!(
        listed, four,
        "KISS-CONSUME-6.4-0007: the anti-sniffing category list does not enumerate the full four-category set"
    );

    assert!(
        section_refs(block).contains("§6.1"),
        "KISS-CONSUME-6.4-0007: category assignment is decoupled from the §6.1 structure-based recognition anchor"
    );
}

// ---------------------------------------------------------------------------
// §8.1 — two version axes
// ---------------------------------------------------------------------------

/// KISS-CONSUME-8.1-0001 — a recognition-coverage broadening (lifting an idiom previously
/// left as residue) MUST bump ONLY the published-crate semver and MUST NOT bump the schema
/// version; a change to the category set/spellings, residue schema, input-structure-graph
/// representation, or a round-trip tier statement MUST bump the schema version. Teeth: the
/// §8.1 intro `Schema version` bullet's behavior-surface §-ref set (read by content) must
/// equal the canonical {§6.1, §6.3, §6.4, §6.6} (input-structure-graph, residue schema,
/// refusal taxonomy, round-trip tiers) AND must EXCLUDE §6.2 (recognition/lift coverage) —
/// because a coverage improvement is assigned to published-crate-semver only. A drift that
/// promoted §6.2 lift-coverage onto the schema-version surface, or dropped the residue
/// schema / taxonomy from it, breaks the partition and fails.
#[test]
fn test_consume_version_bump_rule() {
    let consume = read_spec("consume.md");
    let surface = schema_version_surface(&consume);
    assert_eq!(
        surface,
        sset(&["§6.1", "§6.3", "§6.4", "§6.6"]),
        "KISS-CONSUME-8.1-0001: the schema-version behavior surface drifted from {{§6.1,§6.3,§6.4,§6.6}}"
    );
    assert!(
        !surface.contains("§6.2"),
        "KISS-CONSUME-8.1-0001: §6.2 lift-coverage was promoted onto the schema-version surface (belongs to crate-semver only)"
    );

    let block = plain(clause_block(&consume, "KISS-CONSUME-8.1-0001"));
    assert!(
        block.contains("only the published-crate semver")
            && block.contains("MUST NOT bump the KISS-Consume schema version"),
        "KISS-CONSUME-8.1-0001: coverage broadening is not pinned to published-crate-semver-only"
    );
}

/// KISS-CONSUME-8.1-0002 — the unrecognized-but-expressible ↔ inexpressible-residue boundary
/// is op-set-version-relative; a recorded residue determination MUST be interpreted against
/// the op-set version it recorded (§6.3-0005) and MUST NOT be treated as valid under a
/// different op-set version without re-evaluation. Teeth: the two boundary tokens extracted
/// from the clause body (before its `*Test:*` tag) must equal the §6.3-0004 residue-bearing
/// two-token subset read BY CONTENT (a drift in either boundary token fails), and the clause
/// must back-reference §6.3 (the recorded determining op-set version that pins the
/// determination) — severing the version-relativity from the recorded-version field fails.
#[test]
fn test_consume_residue_opset_relative() {
    let consume = read_spec("consume.md");
    let block = clause_block(&consume, "KISS-CONSUME-8.1-0002");
    let body = &block[..block.find("*Test:*").unwrap_or(block.len())];

    let boundary: BTreeSet<String> = backtick_tokens(body).into_iter().collect();
    let subset = residue_subset(&consume);
    assert_eq!(subset.len(), 2, "KISS-CONSUME-8.1-0002: residue-bearing subset is not the two-token set");
    assert_eq!(
        boundary, subset,
        "KISS-CONSUME-8.1-0002: the op-set-relative boundary pair drifted from the §6.3-0004 residue subset"
    );

    assert!(
        section_refs(block).contains("§6.3"),
        "KISS-CONSUME-8.1-0002: the clause does not back-reference the recorded determining op-set version (§6.3-0005)"
    );
}
