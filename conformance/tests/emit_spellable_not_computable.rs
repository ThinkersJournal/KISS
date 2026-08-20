//! **KISS-EMIT-6.4-0006 — spellable is not computable.**
//!
//! # Why this is a document lint rather than an emitter-behaviour test
//!
//! The clause forbids an *inference*: that a dtype having an emitter-supplied
//! target-language surface (§6.4-0004) implies the target can compute in it. No
//! emitter can be observed making or not making an inference, so what is testable
//! is that the document **carries the distinction the inference needs in order to
//! be refusable** — two defined terms, a clause that separates them, and a route
//! for the case where they differ.
//!
//! Weak lints of this shape assert that a sentence exists. This one asserts the
//! **structure**: that `storage-capable` and `compute-capable` are both defined in
//! §3, that §6.4-0006 references both and is a distinct clause from §6.4-0004, and
//! that it routes the divergent case to §6.8's typed decline. Deleting the clause,
//! deleting either term, or collapsing the two into one fails it.
//!
//! # The defect it guards against, which is not hypothetical
//!
//! A reference C emitter had its dtype-support predicate implemented as
//! `!matches!(F16 | Bf16) && scalar_ctype(dtype).is_some()`. `scalar_ctype` returns
//! the **carrier** — `unsigned char` for FP8 and the sub-byte dtypes — so a
//! **storage fact was answering a compute question**. It gave correct answers for a
//! reason it never stated, and the two-dtype exclusion list was a patch over the gap
//! rather than a model of it.
//!
//! Two registered namespaces exhibit the split, which is what makes it a property of
//! dtypes rather than one vendor's spelling: a `vulkan:` device may advertise
//! `storageBuffer16BitAccess` and do the arithmetic in `f32`; a `cuda:` device holds
//! fp8 in memory on any architecture while the arithmetic is architecture-gated.

fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// Body text of one clause: its bold anchor to the next clause or heading.
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

#[test]
fn test_emit_spellable_is_not_computable() {
    let emit = read_spec("emit.md");

    // (a) Both terms are DEFINED in §3 — as bold term entries, not merely used in
    //     prose somewhere. A term used but never defined is the gap this closes.
    for term in ["**Storage-capable**", "**Compute-capable**"] {
        assert!(
            emit.contains(&format!("- {term} —")),
            "§3 must DEFINE {term} as a term entry. Without both defined, §6.4-0006 \
             forbids an inference using vocabulary the document does not have."
        );
    }

    // (b) The clause exists and is DISTINCT from the spelling clause it qualifies.
    //     Folding the two together would lose exactly the separation being asserted.
    let c6 = clause_block(&emit, "KISS-EMIT-6.4-0006").to_lowercase();
    let c4 = clause_block(&emit, "KISS-EMIT-6.4-0004").to_lowercase();
    assert!(
        !c6.contains("kiss-emit-6.4-0004**"),
        "§6.4-0006 must be its own clause, not a continuation of §6.4-0004"
    );
    assert!(
        c4.contains("spelling"),
        "§6.4-0004 is the SPELLING obligation this clause qualifies; if it no longer \
         is, §6.4-0006's reference to it is stale"
    );

    // (c) The clause names both sides of the distinction. Naming only one would let a
    //     reader keep the conflation while the clause appears to address it.
    for needed in ["storage-capable", "compute-capable"] {
        assert!(
            c6.contains(needed),
            "§6.4-0006 must name `{needed}` — a clause forbidding the inference must \
             name both things being conflated:\n{c6}"
        );
    }

    // (d) It forbids rather than merely observes. An informative note would not stop
    //     the inference, which is the entire point of the clause.
    assert!(
        c6.contains("must not"),
        "§6.4-0006 must carry a MUST NOT. Observing that the two differ does not \
         prevent an implementation from inferring one from the other:\n{c6}"
    );

    // The check above was too loose in its first form: it asserted only that SOME
    // prohibition survives, while claiming to assert that THE INFERENCE is
    // forbidden. The clause carries two MUST NOTs, so a seeded mutation replacing
    // "MUST NOT infer" with "may infer" PASSED -- the other prohibition satisfied
    // it. A check measuring something adjacent to what it names, inside a test
    // written to catch exactly that. Pinned to the specific prohibition below.
    assert!(
        c6.contains("infer"),
        "6.4-0006 must address INFERRING computability, the act it exists to forbid"
    );
    assert!(
        !c6.contains("may infer"),
        "6.4-0006 PERMITS the inference it exists to forbid; weakening the prohibition to permissive language is the failure this guards"
    );

    // (e) It routes the divergent case somewhere real. Forbidding the inference
    //     without saying what to do instead leaves the implementer to invent it —
    //     and inventing it is how the original defect got written.
    assert!(
        c6.contains("6.8"),
        "§6.4-0006 must route the storage-capable-but-not-compute-capable case to \
         §6.8's typed decline, or it forbids an inference without supplying an \
         alternative:\n{c6}"
    );
}
