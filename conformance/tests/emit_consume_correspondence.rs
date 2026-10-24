// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cross-standard document lint for the emit↔consume two-tier round-trip
//! correspondence, continuing the #91 burndown. This one Conform-owned test
//! DOUBLE-BACKS two clauses of different sub-standards:
//!   * KISS-EMIT-6.7-0008    — the enumerated clause-correspondence table, and
//!   * KISS-CONFORM-6.11-0001 — Conform MUST own the cross-standard lint that reads
//!     both texts and fails on a row whose paired clauses are not equivalent.
//!
//! The `test_conform_` prefix marks it as a Conform-owned cross-standard test, so
//! kiss_trace's suite gate treats the two-clause mapping as an allowed deferral
//! rather than a fan-out violation. It lives in its own CONFORM test file (not in
//! emit_partition.rs) for exactly that reason.
//!
//! This is a document-consistency binding, NOT a behavioral round-trip test: it
//! reads the two spec texts and asserts the table is internally consistent and that
//! each paired clause exists and shares the row's discriminating keyword. It does not
//! exercise any emitter or consumer.

use std::collections::BTreeSet;

fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// Body text of a single clause: from its `**KISS-…**` bold anchor to the next
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

/// Every clause id defined in `md` whose id begins with `prefix`.
fn defined_clauses(md: &str, prefix: &str) -> BTreeSet<String> {
    let anchor = format!("**{prefix}");
    let mut out = BTreeSet::new();
    let mut i = 0;
    while let Some(p) = md[i..].find(&anchor) {
        let s = i + p + 2;
        if let Some(e) = md[s..].find("**") {
            out.insert(md[s..s + e].to_string());
        }
        i = s + 1;
    }
    out
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").replace("- ", "-")
}

/// One parsed row of the emit↔consume correspondence table.
struct Row {
    emit: String,
    consume: String,
    statement: String,
}

/// Parse the 5-row `| KISS-Emit | KISS-Consume | Statement |` table from emit.md.
fn correspondence_rows(emit_md: &str) -> Vec<Row> {
    let header = "| KISS-Emit | KISS-Consume | Statement |";
    let h = emit_md
        .find(header)
        .expect("emit.md: correspondence-table header not found");
    let mut rows = Vec::new();
    for line in emit_md[h..].lines().skip(1) {
        let t = line.trim();
        if !t.starts_with('|') {
            break; // blank line / end of table
        }
        if t.starts_with("|---") || t.starts_with("| ---") {
            continue; // markdown separator row
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(|c| c.trim()).collect();
        if cells.len() != 3 || !cells[0].starts_with('§') {
            break;
        }
        rows.push(Row {
            emit: cells[0].to_string(),
            consume: cells[1].to_string(),
            statement: cells[2].to_string(),
        });
    }
    rows
}

/// Enforces: KISS-EMIT-6.7-0008 AND KISS-CONFORM-6.11-0001 — the cross-standard document lint.
/// The emit↔consume correspondence table in emit.md pairs each KISS-Emit §6.7
/// round-trip clause with its KISS-Consume §6.6 counterpart. Teeth: (a) exactly 6
/// rows covering §6.7-0001..0005 + §6.7-0009 ↔ §6.6-0001..0006 -- the emit side is
/// NOT contiguous, and pinning the explicit set rather than a range is what makes a
/// silently-dropped row fail; (b) every referenced clause is
/// actually DEFINED in its own document (a renumber/removal on either side fails);
/// (c) each row's discriminating keyword — the longest non-generic word of the
/// Statement — appears in BOTH paired clause bodies, so a mispaired row (Emit §6.7-0001
/// pointed at a Consume clause that is not the structural one) fails.
#[test]
fn test_conform_emit_consume_correspondence_lint() {
    let emit = read_spec("emit.md");
    let consume = read_spec("consume.md");
    let rows = correspondence_rows(&emit);
    assert_eq!(
        rows.len(),
        6,
        "KISS-EMIT-6.7-0008 / KISS-CONFORM-6.11-0001: expected 6 correspondence rows, found {}",
        rows.len()
    );

    let emit_defs = defined_clauses(&emit, "KISS-EMIT-6.7-");
    let consume_defs = defined_clauses(&consume, "KISS-CONSUME-6.6-");

    // Generic round-trip vocabulary that does not discriminate one row from another.
    let generic = [
        "tier",
        "1",
        "2",
        "round-trip",
        "round",
        "trip",
        "by",
        "no",
        "of",
        "the",
        "",
    ];

    let mut emit_refs = BTreeSet::new();
    let mut consume_refs = BTreeSet::new();
    for r in &rows {
        let emit_id = format!("KISS-EMIT-{}", r.emit.trim_start_matches('§'));
        let consume_id = format!("KISS-CONSUME-{}", r.consume.trim_start_matches('§'));
        assert!(
            emit_defs.contains(&emit_id),
            "correspondence: emit clause {emit_id} referenced by the table is not defined in emit.md"
        );
        assert!(
            consume_defs.contains(&consume_id),
            "correspondence: consume clause {consume_id} referenced by the table is not defined in consume.md"
        );
        emit_refs.insert(emit_id.clone());
        consume_refs.insert(consume_id.clone());

        let kw = r
            .statement
            .split(|ch: char| ch.is_whitespace() || ch == ',')
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                    .to_lowercase()
            })
            .filter(|w| !generic.contains(&w.as_str()))
            .max_by_key(|w| w.len())
            .unwrap_or_default();
        assert!(
            kw.len() >= 4,
            "correspondence: row `{}` has no discriminating keyword",
            r.statement
        );

        let eb = norm(clause_block(&emit, &emit_id)).to_lowercase();
        let cb = norm(clause_block(&consume, &consume_id)).to_lowercase();
        assert!(
            eb.contains(&kw),
            "correspondence: keyword `{kw}` from row `{}` is absent from {emit_id}",
            r.statement
        );
        assert!(
            cb.contains(&kw),
            "correspondence: keyword `{kw}` from row `{}` is absent from {consume_id} (paired with {emit_id})",
            r.statement
        );
    }

    // Explicit sets, not ranges: the emit side is NOT contiguous. §6.7-0009
    // (whole-kernel tier aggregation) pairs with §6.6-0006, while §6.7-0006 -- the
    // tier-2 determinants -- has no Consume twin to pair with and is deliberately
    // absent. A range would have quietly admitted the wrong six.
    let expect_emit: BTreeSet<String> = [1, 2, 3, 4, 5, 9]
        .iter()
        .map(|n| format!("KISS-EMIT-6.7-{n:04}"))
        .collect();
    let expect_consume: BTreeSet<String> = (1..=6)
        .map(|n| format!("KISS-CONSUME-6.6-{n:04}"))
        .collect();
    assert_eq!(
        emit_refs, expect_emit,
        "correspondence: emit side of the table does not cover exactly §6.7-0001..0005 + §6.7-0009"
    );
    assert_eq!(
        consume_refs, expect_consume,
        "correspondence: consume side of the table does not cover exactly §6.6-0001..0006"
    );
}

// ---------------------------------------------------------------------------
// KISS#313 — a within-document tripwire for the excised "declared comparator" mandate.
// ---------------------------------------------------------------------------

/// The round-trip clause family on each side: Emit §6.7-0001..0005 + -0009, Consume §6.6-0001..0006.
const EMIT_ROUNDTRIP: [&str; 6] = [
    "KISS-EMIT-6.7-0001",
    "KISS-EMIT-6.7-0002",
    "KISS-EMIT-6.7-0003",
    "KISS-EMIT-6.7-0004",
    "KISS-EMIT-6.7-0005",
    "KISS-EMIT-6.7-0009",
];
const CONSUME_ROUNDTRIP: [&str; 6] = [
    "KISS-CONSUME-6.6-0001",
    "KISS-CONSUME-6.6-0002",
    "KISS-CONSUME-6.6-0003",
    "KISS-CONSUME-6.6-0004",
    "KISS-CONSUME-6.6-0005",
    "KISS-CONSUME-6.6-0006",
];

/// Occurrences of "comparator" in `block` that are NOT a quoted citation of the excised mandate: an
/// occurrence is allowed only inside a double-quoted span AND in a sentence that says the phrase was
/// "excised" or belongs to a "previous revision". Anything else is a live mandate (or an unexplained
/// mention) and is returned with its context.
fn live_comparator_mentions(block: &str) -> Vec<String> {
    let text = norm(block);
    let lower = text.to_lowercase();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(p) = lower[from..].find("comparator") {
        let at = from + p;
        let quoted = text[..at].matches('"').count() % 2 == 1;
        let lo = at.saturating_sub(260);
        let hi = (at + 260).min(text.len());
        let window = &lower[floor_char(&lower, lo)..floor_char(&lower, hi)];
        let explained = window.contains("excised") || window.contains("previous revision");
        if !(quoted && explained) {
            out.push(text[floor_char(&text, lo)..floor_char(&text, hi)].to_string());
        }
        from = at + "comparator".len();
    }
    out
}

fn floor_char(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// KISS#313 — a TRIPWIRE, not a coherence check. `test_conform_emit_consume_correspondence_lint` compares
/// the two documents against each other, so a mandate re-added to BOTH twins (Emit §6.7-0004 and Consume
/// §6.6-0004, which correspond) leaves it green while each twin contradicts its own document's §-0002. This
/// asserts the one known phrase family: after the excision of the "declared comparator" verification mandate
/// (Emit §6.7-0002/-0004, Consume §6.6-0004), "comparator" appears in the round-trip family only as a
/// quoted citation of the excised wording. It does not generalise to other within-document contradictions.
/// TEETH: the real spec is clean; a live mandate re-added to either document, or to both, is reported; an
/// unquoted or unexplained mention is reported; the quoted-and-explained form is not.
#[test]
fn test_conform_roundtrip_no_comparator_mandate_remains() {
    let emit = read_spec("emit.md");
    let consume = read_spec("consume.md");
    for id in EMIT_ROUNDTRIP {
        let live = live_comparator_mentions(clause_block(&emit, id));
        assert!(live.is_empty(), "{id}: a comparator mandate stands in the round-trip block: {live:?}");
    }
    for id in CONSUME_ROUNDTRIP {
        let live = live_comparator_mentions(clause_block(&consume, id));
        assert!(live.is_empty(), "{id}: a comparator mandate stands in the round-trip block: {live:?}");
    }

    // Controls: the detector reports a live mandate and accepts the excision note.
    let mandate = "bit-identity MUST be claimed only for an exact-byte op, and MUST compare under that op's declared comparator.";
    assert_eq!(live_comparator_mentions(mandate).len(), 1, "an unquoted mandate is reported");
    let quoted_only = "the phrase \"plus its declared comparator\" stands here.";
    assert_eq!(live_comparator_mentions(quoted_only).len(), 1, "quoted but unexplained is reported");
    let excised = "the phrase \"plus its declared comparator\" was excised with the same mandate for the same reason.";
    assert!(live_comparator_mentions(excised).is_empty(), "a quoted citation of the excision is allowed");
}
