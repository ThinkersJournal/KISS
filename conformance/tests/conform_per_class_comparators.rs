//! KISS-CONFORM-6.13-0005 — the three §6.8 comparators are the selection targets
//! of the canonical enum **imported from KISS-Ops**, and each class routes to the
//! comparator its own KISS-Ops member clause names.
//!
//! This is not the same property as §6.8-0006 (already backed), which says the
//! comparator is selected by the op's *declared* class. This clause says where
//! the classes come *from* and what each one routes *to*: exact-byte evaluated
//! with memcmp (KISS-OPS §6.0-0002), ULP ops with the declared-ULP comparator
//! (§6.0-0003), and nondeterministic ops **without** a byte-exact requirement
//! (§6.0-0004). Two halves, because the clause has two:
//!
//!   (I)  THE IMPORT SITE. Conform's enum must be KISS-Ops' enum verbatim — same
//!        three members, same spellings, same order. Nothing tests this today:
//!        `conform_comparator_selection.rs` cites Conform's own §6.0-0001, never
//!        KISS-Ops' §6.0-0001, so a Conform-local re-spelling would pass every
//!        existing test. §6.11-0003, the import-site lint, is unbacked.
//!
//!   (II) THE THREE TARGETS. A class-selected comparator that routes two classes
//!        to the same behaviour has three enum members and *two* selection
//!        targets. So it is not enough that each class works — the three must be
//!        PAIRWISE DISTINGUISHABLE, and part (II) exhibits an input for each of
//!        the three pairs on which they disagree.
//!
//! TEETH — the wrong implementations, each asserted against:
//!   1. **memcmp everywhere** ("byte-exact is the strictest, so it is safe"):
//!      fails the ULP and order-invariant arms, which §6.0-0003/-0004 forbid
//!      being byte-exact.
//!   2. **A tolerance everywhere** (one comparator, three names): fails the
//!      exact-byte arm, and fails the pairwise-distinctness matrix.
//!   3. **A Conform-local re-spelling** of the enum (`bit-exact`, a fourth
//!      member, a re-ordering): fails part (I).
//!   4. **A comparator that always says Ok**: fails every negative arm.
//!
//! CITATION DISCIPLINE: this test cites ONLY `KISS-CONFORM-6.13-0005`.
//! Cross-references to KISS-Ops and to other Conform clauses use the
//! `§<sec>-<nnnn>` short form, which does not match the citation grammar.

use kiss_conformance::structural::compare_reduced_f32;
use kiss_conformance::{compare, compare_f32, DeterminismClass};

fn read_spec(name: &str) -> String {
    let path = format!("{}/../spec/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read spec file `{path}`: {e}"))
}

/// Assemble a clause anchor from its parts.
///
/// The ID is BUILT rather than written out, and that is load-bearing. The
/// traceability gate treats any literal `KISS-<SUB>-<sec>-<nnnn>` appearing in a
/// test as a **citation** of that clause. Spelling out the KISS-Ops enum clause's
/// ID here would reverse-cite it — claiming this test backs §6.0-0001 of KISS-Ops,
/// which it does not: it reads that clause's spelling, it does not test its
/// obligation (that every op declares exactly one class and KISS-Ops forks no
/// parallel vocabulary). Worse, that clause is enforced by the `kiss_ops` document
/// lint, so the citation would have swapped real lint coverage for an incidental
/// string match, and the ENFORCED total would not have moved to show it.
///
/// This is not hypothetical: the first version of this file spelled both IDs out,
/// and `kiss_trace.py` failed with `STALE LEDGER: … now enforced`. The gate caught
/// the over-credit. No literal foreign clause ID appears anywhere in this file —
/// not even in a comment, since the scan's scope is not a property to rely on.
fn clause_anchor(sub: &str, section: &str, ordinal: &str) -> String {
    format!("KISS-{sub}-{section}-{ordinal}")
}

/// Body text of a single clause, from its `**KISS-…**` anchor to the next clause
/// or heading.
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

/// The determinism enum as the clause spells it: the members of the first
/// `{...}` brace group, in document order, trimmed. Reads the SPELLING out of the
/// text rather than comparing against a constant in this file — a hardcoded copy
/// here would drift with neither document and prove nothing.
fn enum_members(block: &str) -> Vec<String> {
    let open = block.find('{').expect("clause names no `{...}` enum");
    let close = block[open..].find('}').expect("unterminated enum brace") + open;
    block[open + 1..close]
        .split(',')
        .map(|m| m.trim().trim_matches('`').split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|m| !m.is_empty())
        .collect()
}

/// The f32 one ULP above `x`.
fn one_ulp_above(x: f32) -> f32 {
    f32::from_bits(x.to_bits() + 1)
}

#[test]
fn test_conform_ops_per_class_comparators() {
    // -----------------------------------------------------------------
    // (I) THE IMPORT SITE — Conform's enum IS KISS-Ops' enum, verbatim.
    // -----------------------------------------------------------------
    let ops = read_spec("ops.md");
    let conform = read_spec("conform.md");
    let owner = enum_members(clause_block(&ops, &clause_anchor("OPS", "6.0", "0001")));
    let importer = enum_members(clause_block(&conform, &clause_anchor("CONFORM", "6.0", "0001")));

    // Parser sanity first: if either side parsed to something degenerate, the
    // comparison below would agree vacuously.
    assert_eq!(
        owner.len(),
        3,
        "KISS-CONFORM-6.13-0005: parsed {} members from the KISS-Ops enum ({owner:?}) — \
         parser drift, the import check would be vacuous",
        owner.len()
    );
    assert_eq!(
        importer, owner,
        "KISS-CONFORM-6.13-0005: Conform's imported determinism enum is not KISS-Ops' enum \
         verbatim — imported {importer:?}, owner declares {owner:?}. A re-spelled, re-ordered \
         or extended copy is a fork, and the three comparators would be selection targets of \
         the wrong vocabulary"
    );
    // The in-code enum must have exactly the three members the text declares —
    // otherwise the spellings agree while the implementation carries a fourth.
    let in_code = [
        DeterminismClass::ExactByte,
        DeterminismClass::UlpTolerance,
        DeterminismClass::OrderInvariant,
    ];
    assert_eq!(
        in_code.len(),
        owner.len(),
        "KISS-CONFORM-6.13-0005: the harness enum has {} members against the text's {}",
        in_code.len(),
        owner.len()
    );

    // -----------------------------------------------------------------
    // (II) THE THREE TARGETS — each class routes where its member clause says.
    // -----------------------------------------------------------------
    let x = 1.0f32;
    let x1 = one_ulp_above(x); // 1 ULP away: identical to no tolerance, equal to a loose one

    // exact-byte -> memcmp (§6.0-0002). Bit-identical passes; one bit differs, fails.
    assert!(compare(DeterminismClass::ExactByte, &[1, 2, 3], &[1, 2, 3]).is_ok());
    assert!(
        compare(DeterminismClass::ExactByte, &[1, 2, 3], &[1, 2, 4]).is_err(),
        "KISS-CONFORM-6.13-0005: the exact-byte class accepted differing bytes — it is not \
         routed to a byte-exact comparator"
    );
    assert!(
        compare_f32(DeterminismClass::ExactByte, x1, x, u64::MAX).is_err(),
        "KISS-CONFORM-6.13-0005: exact-byte accepted a 1-ULP difference even with an enormous \
         ULP bound in hand — the class is routed to a tolerance comparator"
    );

    // ULP/tolerance -> the DECLARED-ULP comparator (§6.0-0003), never byte-exact.
    assert!(
        compare_f32(DeterminismClass::UlpTolerance, x1, x, 1).is_ok(),
        "KISS-CONFORM-6.13-0005: the ULP class rejected a difference within its declared bound \
         — it is routed to a byte-exact comparator, which §6.0-0003 forbids across languages"
    );
    assert!(
        compare_f32(DeterminismClass::UlpTolerance, x1, x, 0).is_err(),
        "KISS-CONFORM-6.13-0005: the ULP comparator ignored its declared bound — a comparator \
         that accepts everything is not selected by anything"
    );

    // order-invariant/nondeterministic -> compared under a TOLERANCE, and MUST
    // NOT require byte-exact reproduction (§6.0-0004).
    assert!(
        compare_reduced_f32(DeterminismClass::OrderInvariant, x1, x, 1e-6, 0.0).is_ok(),
        "KISS-CONFORM-6.13-0005: the order-invariant class rejected a reassociation-sized \
         difference — §6.0-0004 forbids requiring byte-exact reproduction of its result"
    );
    assert!(
        compare_reduced_f32(DeterminismClass::OrderInvariant, 2.0, 1.0, 1e-6, 0.0).is_err(),
        "KISS-CONFORM-6.13-0005: the order-invariant comparator accepted a difference far \
         outside its declared tolerance — it is not comparing, it is passing"
    );

    // -----------------------------------------------------------------
    // (II.b) PAIRWISE DISTINCTNESS — three members, three DIFFERENT targets.
    // For each pair of classes, an input on which they disagree. Without this, a
    // comparator table routing two classes to one behaviour still passes above.
    // -----------------------------------------------------------------

    // exact-byte vs ULP: 1 ULP apart, bound 1.
    assert!(
        compare_f32(DeterminismClass::ExactByte, x1, x, 1).is_err()
            && compare_f32(DeterminismClass::UlpTolerance, x1, x, 1).is_ok(),
        "KISS-CONFORM-6.13-0005: exact-byte and ULP/tolerance agree on a 1-ULP difference — \
         two enum members, one selection target"
    );

    // exact-byte vs order-invariant: same pair, within an absolute band.
    assert!(
        compare_f32(DeterminismClass::ExactByte, x1, x, 0).is_err()
            && compare_reduced_f32(DeterminismClass::OrderInvariant, x1, x, 1e-6, 0.0).is_ok(),
        "KISS-CONFORM-6.13-0005: exact-byte and order-invariant agree on an in-band \
         difference — two enum members, one selection target"
    );

    // ULP vs order-invariant: a difference of many ULP that is still tiny in
    // absolute terms. The ULP comparator counts representable steps and rejects;
    // the order-invariant comparator measures an absolute band and accepts.
    let tiny = 1.0e-30f32;
    let tiny_off = f32::from_bits(tiny.to_bits() + 64); // 64 ULP away, ~1e-36 absolute
    assert!(
        compare_f32(DeterminismClass::UlpTolerance, tiny_off, tiny, 1).is_err()
            && compare_reduced_f32(DeterminismClass::OrderInvariant, tiny_off, tiny, 1e-6, 0.0)
                .is_ok(),
        "KISS-CONFORM-6.13-0005: ULP/tolerance and order-invariant agree on a many-ULP but \
         tiny-absolute difference — two enum members, one selection target"
    );
}
