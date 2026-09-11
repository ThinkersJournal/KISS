//! The §6.19.3 OpAttrs schema, PARSED FROM `spec/ops.md` — the first non-relative oracle for
//! the OpAttrs wire channel (#504).
//!
//! ⚠️ WHY THIS EXISTS. #504 measured that NOTHING ANYWHERE compares one party's OpAttrs bytes
//! against another's. Fuel has ~60 assertions against Fuel's own goldens; KISS's opattrs tests
//! compare KISS to KISS; Baracuda and kiss-ref emit no blob at all (they are NON-PARTIES, not
//! neglectful ones). So umbrella §5.3 condition 1 — ≥2 implementations interoperating per-field —
//! has a live pair count of ZERO, and Fuel's diagnostic is the sharp form of it: *"if Fuel's
//! canonical encoding silently diverged from §6.19's positional shape tomorrow, which number
//! would move? None."* Relative oracles on both sides of a seam, guarding a claim about
//! agreement with a second party.
//!
//! ⚠️ THE SCHEMA IS PARSED, NEVER TRANSCRIBED, AND THAT IS THE WHOLE POINT. A transcribed table
//! is just another golden with a longer provenance — #466's finding about KISS-Grammar, whose
//! goldens are retyped into Rust. Retyping §6.19.3 here would reproduce the exact circularity
//! #504 is about: the "external reference" would be one more implementation's opinion. Because
//! this reads `spec/ops.md`, a change to the spec that this crate does not follow REDDENS.
//!
//! ⚠️ AND `include_str!`, NOT a runtime read: the generator must emit byte-identical output
//! wherever it runs, so it cannot depend on a file being findable at runtime. Same reasoning as
//! `grammar.rs`'s `GRAMMAR_SPEC`.
//!
//! ⚠️ TWO INDEPENDENT READINGS, DELIBERATELY. The TABLE is the schema source; the per-op CLAUSES
//! are a second reading of the same facts. Their disagreement is therefore DETECTABLE rather than
//! assumed away — the shape that made #485's `Backs:`-vs-§9 cross-check work. The architect
//! predicts they agree 17-of-17; that prediction is a DETECTOR, not a shortcut, so a mismatch is
//! surprising and surprise is what we are preserving.

use std::collections::BTreeMap;

/// `spec/ops.md`, baked in at COMPILE time.
const OPS_SPEC: &str = include_str!("../../spec/ops.md");

/// One field of an OpAttrs blob, in canonical (ABI) order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaField {
    /// The attribute name. ⚠️ Never appears on the wire (§6.19.3): `op_name` selects the schema
    /// and the reader replays it positionally. Carried here so a vector is readable.
    pub name: String,
    /// The encoding text exactly as the spec writes it, unnormalised.
    pub encoding: String,
    /// Width in bytes, when the encoding pins one.
    pub width: FieldWidth,
}

/// ⚠️ A THIRD ARM, because `u8`/`u16` is not a partition. The pooling ops carry `vec` fields whose
/// width is not fixed by the table, and collapsing them into a default width would silently
/// fabricate bytes for a vector claiming to be spec-derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldWidth {
    /// `u8` — one byte.
    U8,
    /// `u16` LE — two bytes, little-endian.
    U16Le,
    /// The table does not pin a fixed width here (`vec`).
    Variable,
}

impl FieldWidth {
    /// Bytes, when fixed. `None` for `Variable` — callers must handle it rather than defaulting.
    pub fn bytes(self) -> Option<usize> {
        match self {
            FieldWidth::U8 => Some(1),
            FieldWidth::U16Le => Some(2),
            FieldWidth::Variable => None,
        }
    }
}

/// One carrier op's OpAttrs schema, in canonical field order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpSchema {
    pub op: String,
    pub fields: Vec<SchemaField>,
}

impl OpSchema {
    /// Total blob width, when every field pins one. `None` if any field is `Variable`.
    pub fn fixed_width(&self) -> Option<usize> {
        self.fields.iter().map(|f| f.width.bytes()).sum()
    }
}

/// The §6.19.3 section text, bounded by its own heading and the next `####`.
///
/// ⚠️ BOUNDED BY THE HEADING, NEVER BY A LINE COUNT OR A CHARACTER WINDOW. Five separate
/// windowed extractions failed in this repo in one night — a `-B8`, a 200-char window, a
/// `git log -20` read as a population, and two of mine. A window is INVISIBLE IN ITS OWN OUTPUT:
/// every one produced a well-formed, plausible number that was simply short.
fn section_6_19_3() -> &'static str {
    let start = OPS_SPEC
        .find("#### 6.19.3")
        .expect("spec/ops.md must carry `#### 6.19.3` — the OpAttrs schema section");
    let rest = &OPS_SPEC[start..];
    // skip past this heading before looking for the next one
    let end = rest[1..]
        .find("\n#### ")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

/// Parse the §6.19.3 TABLE — the canonical field order, which the section titles "= ABI".
///
/// Row grammar, as the spec writes it:
///
/// ```text
/// | `reduce` | `monoid`:enum `u8`:mandatory (...) — `reduce_axes`:`u16` LE:mandatory (...) — ... |
/// ```
///
/// Fields are separated by ` — ` (em dash), and each begins with a backticked name.
pub fn parse_table() -> Vec<OpSchema> {
    let mut out = Vec::new();
    for line in section_6_19_3().lines() {
        let line = line.trim();
        if !line.starts_with("| `") || !line.ends_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split(" | ").collect();
        if cells.len() != 2 {
            continue;
        }
        let op = cells[0].trim().trim_matches('`').to_string();
        if op.is_empty() {
            continue;
        }
        let mut fields = Vec::new();
        for part in cells[1].split(" — ") {
            let part = part.trim();
            // `name`:rest
            let Some(rest) = part.strip_prefix('`') else { continue };
            let Some((name, tail)) = rest.split_once('`') else { continue };
            let encoding = tail.trim_start_matches(':').trim().to_string();
            let width = if encoding.contains("`u16`") {
                FieldWidth::U16Le
            } else if encoding.contains("`u8`") {
                FieldWidth::U8
            } else {
                // ⚠️ NOT a default. `vec` and anything else unpinned lands here EXPLICITLY, so a
                // caller cannot silently treat an unknown encoding as one byte.
                FieldWidth::Variable
            };
            fields.push(SchemaField { name: name.to_string(), encoding, width });
        }
        if !fields.is_empty() {
            out.push(OpSchema { op, fields });
        }
    }
    // ⚠️ REFUSE ON AN EMPTY PARSE. A recognizer that silently matches nothing reports a clean
    // tree, and "0 schemas, no disagreements" is byte-identical to "the table moved and I did
    // not notice".
    assert!(
        !out.is_empty(),
        "§6.19.3 table parsed to ZERO schemas — the row grammar changed. This is a broken \
         extractor, NOT an empty table; reporting no findings here would be a FALSE CLEAN."
    );
    out
}

/// The SECOND, INDEPENDENT reading: which `KISS-OPS-6.19-00NN` clause names each op.
///
/// ⚠️ CLAUSE BODIES ARE BOUNDED BY THE NEXT CLAUSE ID, not by a character window — see the note
/// on [`section_6_19_3`]. The architect's first attempt at this used a 200-character window and
/// reported `max_pool` uncovered; it sits in -0032 beside `avg_pool`, past the window.
///
/// ⚠️ AND THE OP LIST IS PLURAL-AWARE. A multi-op clause writes "OpAttrs blob**s**", and a
/// pattern requiring the singular drops exactly the clauses that cover more than one op — which
/// is the error that produced my own "17 table ops vs 9 clause ops" non-finding.
pub fn parse_clause_coverage() -> BTreeMap<String, Vec<String>> {
    let sec = section_6_19_3();
    let mut ids: Vec<(usize, String)> = Vec::new();
    let mut cursor = 0usize;
    while let Some(i) = sec[cursor..].find("KISS-OPS-6.19-") {
        let at = cursor + i;
        let id: String = sec[at..].chars().take("KISS-OPS-6.19-0000".len()).collect();
        if id.len() == "KISS-OPS-6.19-0000".len()
            && id[id.len() - 4..].chars().all(|c| c.is_ascii_digit())
        {
            ids.push((at, id));
        }
        cursor = at + 1;
    }

    let mut cov: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (n, (at, id)) in ids.iter().enumerate() {
        let end = ids.get(n + 1).map(|(a, _)| *a).unwrap_or(sec.len());
        // ⚠️ WHITESPACE-NORMALISE BEFORE MATCHING. -0034 wraps as `... `scatter_add` OpAttrs`
        // NEWLINE `blobs MUST each be ...`, so a literal " OpAttrs blob" cannot match it. A phrase
        // query against hard-wrapped prose silently measures the AUTHOR'S WRAP WIDTH -- it found
        // avg_pool and missed max_pool, index_select, embedding and scatter_add, and the miss
        // looked exactly like a spec gap.
        let body: String = sec[*at..end].split_whitespace().collect::<Vec<_>>().join(" ");

        // ⚠️ EVERY OCCURRENCE, NOT THE FIRST. -0032 carries TWO: "The `avg_pool` OpAttrs blob
        // MUST be ...; the `max_pool` OpAttrs blob MUST be ...". Taking only the first drops the
        // second op while reporting a clean parse of the clause.
        let mut from = 0usize;
        while let Some(k) = body[from..].find(" OpAttrs blob") {
            let at_phrase = from + k;
            let head = &body[..at_phrase];
            // back up to the nearest article, EITHER CASE -- the second phrase in -0032 begins
            // with a lowercase `the` because it is mid-sentence.
            let anchor = head
                .rfind("The ")
                .into_iter()
                .chain(head.rfind("the "))
                .max()
                .unwrap_or(0);
            for name in head[anchor..].split('`').skip(1).step_by(2) {
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                {
                    let e = cov.entry(name.to_string()).or_insert_with(Vec::new);
                    if !e.contains(id) {
                        e.push(id.clone());
                    }
                }
            }
            from = at_phrase + " OpAttrs blob".len();
        }
    }

    // ⚠️ REFUSE ON AN EMPTY READING, same reasoning as the table parser: an empty map makes
    // every op read as uncovered, which is a loud wrong answer rather than a quiet one -- but it
    // is still the extractor talking about itself, not about the spec.
    assert!(
        !cov.is_empty(),
        "the §6.19.3 clause reading found ZERO ops -- broken extractor, not an empty section"
    );
    cov
}

/// The clause that makes the `offset` column STATED rather than COMPUTED, checked to still say so.
///
/// ⚠️ THE BOUNDARY THIS ENFORCES, kiss-ref's, and it is the sharpest correction this artifact
/// received: *"An external reference that resolves non-primitives from §6.13 is STILL
/// comprehension-correlated. External-ness is not the property you want — INDEPENDENCE OF
/// DERIVATION is."* Two parties reading one decomposition table are module-disjoint and
/// semantically correlated, and no set-intersection of module names can see it.
///
/// So this artifact pins STATED LAYOUT and must never compute an expected VALUE. Field order,
/// widths and ordinals are read straight off §6.19.3. **`offset` is the one column that is
/// arithmetic**, and it is legitimate only because §6.19-0004 states the composition rule:
///
/// > "Each carrier op's OpAttrs blob MUST be exactly its schema fields (§6.19.3) **concatenated**
/// > in the canonical, frozen field order shown"
///
/// Concatenated — no padding, no alignment, no tag stream. Under that sentence `offset` is a
/// restatement of the stated layout, not a walk through a shared intermediate.
///
/// ⚠️ AND IF THAT SENTENCE EVER CHANGES, THE OFFSETS BECOME UNFOUNDED WHILE STILL LOOKING
/// CORRECT — a padding or alignment rule would make every offset after the first field silently
/// wrong, and nothing else in this crate would notice. That is precisely the "compute past a gap
/// in a stated layout" failure kiss-ref told me to refuse rather than fill, so it is a guard
/// rather than a comment.
fn assert_concatenation_is_stated() {
    let c = OPS_SPEC
        .find("KISS-OPS-6.19-0004")
        .map(|i| &OPS_SPEC[i..i + 600])
        .expect("spec/ops.md must carry KISS-OPS-6.19-0004 — the blob composition rule");
    let flat: String = c.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains("concatenated in the canonical"),
        "KISS-OPS-6.19-0004 no longer states that the blob is its schema fields CONCATENATED in \
         canonical order. The `offset` column of this artifact is arithmetic over stated widths \
         and is sound ONLY under that sentence: a padding or alignment rule would make every \
         offset after field 0 silently wrong. Re-derive the offset model against the new text \
         rather than letting this artifact keep asserting the old one."
    );
}

/// Emit the SPEC-DERIVED OpAttrs vector artifact (#504).
///
/// ⚠️ THIS IS THE NON-RELATIVE ORACLE. Every byte below is computed from `spec/ops.md` §6.19.3 —
/// the field NAMES, their ORDER, and their WIDTHS all come from the table the section titles
/// "canonical field order = ABI". No implementation's output was consulted, which is the single
/// property that distinguishes this from every existing OpAttrs assertion in the portfolio:
/// Fuel's ~60 compare Fuel to Fuel, KISS's compare KISS to KISS.
///
/// ⚠️ WHAT A FOREIGN PARTY DOES WITH IT. A second emitter reproduces these offsets and widths
/// from their own encoder and byte-diffs. Disagreement is then about the SPEC, not about whose
/// golden is authoritative — which is what umbrella §5.3 condition 1 asks for and what the live
/// pair count of ZERO currently prevents.
///
/// ⚠️ WHAT IT DOES NOT DO: it does not close the interop gap. That needs a second emitter and is
/// not buildable from inside this repo. This makes the KISS side externally CHECKABLE, which is
/// the half that can be built alone.
pub fn emit_opattrs_vectors_json() -> String {
    // the offset column is sound only while §6.19-0004 says "concatenated" — check, do not assume
    assert_concatenation_is_stated();
    let schemas = parse_table();
    let coverage = parse_clause_coverage();

    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"schema\": \"kiss-opattrs-spec-vectors/1\",\n");
    s.push_str("  \"generated_from\": \"spec/ops.md §6.19.3 (canonical field order = ABI)\",\n");
    s.push_str("  \"derivation\": \"PARSED from the spec table at compile time, never transcribed. \
No implementation output was consulted: this is an EXTERNAL reference for the OpAttrs channel, \
not another party's golden (#504).\",\n");
    // ⚠️ THE POPULATION IS PART OF THE ARTIFACT. A narrowed parser emits a smaller, well-formed
    // file, and a reader diffing two such files sees a clean diff of a shorter document. Stating
    // the count makes a narrowing visible IN the artifact rather than only in a test run.
    s.push_str("  \"layout_authority\": \"KISS-OPS-6.19-0004 — the blob is its §6.19.3 schema fields CONCATENATED in canonical order (no padding, no alignment, no tag stream). The offset column is a restatement of that sentence, not a computed value; if the sentence changes the offsets are unfounded and the generator refuses.\",\n");
    s.push_str("  \"scope\": \"STATED LAYOUT ONLY. This artifact pins field order, widths, ordinals and offsets. It deliberately states NO expected VALUE for any op: a value would have to be COMPUTED, and two parties computing through a shared intermediate are comprehension-correlated even when code-disjoint (kiss-ref, DESIGN.md:17-20). Independence of DERIVATION is the property, not external-ness.\",\n");
    s.push_str(&format!("  \"carrier_ops\": {},\n", schemas.len()));

    s.push_str("  \"ops\": [\n");
    for (i, sc) in schemas.iter().enumerate() {
        s.push_str("    {\n");
        s.push_str(&format!("      \"op\": {},\n", crate::json::escape_string(&sc.op)));
        let clauses = coverage.get(&sc.op).cloned().unwrap_or_default();
        s.push_str(&format!(
            "      \"clauses\": [{}],\n",
            clauses
                .iter()
                .map(|c| crate::json::escape_string(c))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        match sc.fixed_width() {
            Some(w) => s.push_str(&format!("      \"blob_bytes\": {w},\n")),
            // ⚠️ `null`, never 0. A pooling op's `vec` fields are not pinned by the table, and a
            // 0 would read as an empty blob rather than as an unpinned width.
            None => s.push_str("      \"blob_bytes\": null,\n"),
        }
        s.push_str("      \"fields\": [\n");
        let mut offset: Option<usize> = Some(0);
        for (j, f) in sc.fields.iter().enumerate() {
            let width = match f.width {
                FieldWidth::U8 => "\"u8\"",
                FieldWidth::U16Le => "\"u16le\"",
                FieldWidth::Variable => "\"variable\"",
            };
            let off = match offset {
                Some(o) => format!("{o}"),
                None => "null".to_string(),
            };
            s.push_str(&format!(
                "        {{\"ordinal\": {}, \"name\": {}, \"width\": {}, \"offset\": {}}}{}\n",
                j,
                crate::json::escape_string(&f.name),
                width,
                off,
                if j + 1 == sc.fields.len() { "" } else { "," }
            ));
            // once a variable-width field is passed, no later offset is knowable
            offset = match (offset, f.width.bytes()) {
                (Some(o), Some(w)) => Some(o + w),
                _ => None,
            };
        }
        s.push_str("      ]\n");
        s.push_str(if i + 1 == schemas.len() { "    }\n" } else { "    },\n" });
    }
    s.push_str("  ]\n");
    s.push_str("}\n");
    s
}
