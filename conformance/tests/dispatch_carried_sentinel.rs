//! #456 / #480 spec-consistency guard: the geometry-agnostic Dispatch clause (§6.6-0007) and the
//! seven-block body invariant (§6.11-0004) MUST stay mutually satisfiable.
//!
//! They were not. §6.6-0007 said a geometry-agnostic kernel MAY declare *no* Dispatch section and
//! KISS-Conform MUST *accept* an absent one, while §6.11-0004 requires *exactly the seven section
//! blocks* and MUST *reject* a document with an absent section heading. On the case "Dispatch
//! absent" one said accept and the other said reject — a contradiction no reader could satisfy
//! (#480). #456 resolved it toward CARRIED: the Dispatch block is always present, and a
//! geometry-agnostic kernel carries the sentinel line `dispatch_model = geometry-agnostic`.
//!
//! This is a spec-text guard, not a byte-behaviour backing — it reads the prose and reds if the
//! absent-Dispatch license returns to §6.6-0007. The byte-level acceptance/decline of the sentinel
//! block is the document-layer work in #481 (the §6.6-0007 test stays vacuous until then).

fn contract() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/contract.md");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// The clause block for `id`, from its bold definition line to the next clause bullet.
fn clause<'a>(doc: &'a str, id: &str) -> &'a str {
    let marker = format!("**{id}**");
    let start = doc.find(&marker).unwrap_or_else(|| panic!("clause {id} not found in contract.md"));
    let rest = &doc[start..];
    // the next "- **KISS-CONTRACT-…**" bullet ends this block (skip the current marker first).
    let end = rest[marker.len()..]
        .find("\n- **KISS-CONTRACT-")
        .map(|i| i + marker.len())
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn test_dispatch_sentinel_is_carried_not_absent() {
    let doc = contract();

    // §6.11-0004 requires all seven blocks and rejects an absent heading, so NEITHER the field
    // schema (§6.6-0001) NOR the geometry-agnostic clause (§6.6-0007) may license omitting the
    // Dispatch section or calling it optional. Case-INSENSITIVE, so a re-cased prose edit
    // ("No Dispatch Section") cannot slip a banned phrase past the guard.
    for id in ["KISS-CONTRACT-6.6-0001", "KISS-CONTRACT-6.6-0007"] {
        let block = clause(&doc, id).to_lowercase();
        for banned in ["no dispatch section", "absent dispatch section", "dispatch section is optional"]
        {
            assert!(
                !block.contains(banned),
                "{id} licenses an absent/optional Dispatch section ({banned:?}) — reopens the #480 \
                 contradiction with 6.11-0004's seven-block invariant"
            );
        }
    }
    // §6.6-0007 MUST name the carried sentinel line in place of the geometry fields.
    assert!(
        clause(&doc, "KISS-CONTRACT-6.6-0007").contains("dispatch_model = geometry-agnostic"),
        "6.6-0007 no longer names the carried sentinel `dispatch_model = geometry-agnostic`"
    );

    // Control — the invariant the carried resolution rests on is still stated: §6.11-0004 requires
    // exactly seven blocks including dispatch. If it moved, this resolution needs revisiting.
    let body = clause(&doc, "KISS-CONTRACT-6.11-0004");
    assert!(
        body.contains("seven section blocks"),
        "6.11-0004 no longer requires exactly seven section blocks (the carried resolution's premise)"
    );
    assert!(
        body.contains("dispatch"),
        "6.11-0004 no longer names the dispatch block among the seven"
    );
}
