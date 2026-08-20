//! Gates for KISS-CLASSIFY §6.6-0002 and §6.6-0020 — the key names a SHAPE, never a
//! computation.
//!
//! §6.6-0002 says `op_family` MUST be a coarse category and never a KISS-Ops op name.
//! §6.6-0020 says cell-mates are therefore NOT mutually substitutable. Both rest on one
//! measurable fact: **nothing in the token names what is computed.** These gates assert
//! that against the real vocabularies instead of restating the prose.
//!
//! EVERY COLLISION IS PAIRED WITH A DISCRIMINATION CONTROL (§6.8-0013's rule): an
//! assertion that two things collide is satisfied by a derivation that collapses
//! everything, which would "prove" semantic blindness by destroying the key.
//!
//! LIMIT, STATED SO IT IS NOT MISTAKEN FOR MORE: the sharpest exhibit of §6.6-0020 would
//! be two genuinely different computations deriving one token — `relu(a + b)` and `a + b`,
//! per the producer that reported the collision. This codec cannot build it: it derives a
//! key from operand descriptors and never takes a body, which is *why* the collision
//! exists but also why it cannot be constructed here. What is asserted below is the
//! structural fact underneath it — no field of a derived token can name a computation —
//! and that is strictly weaker than exhibiting the collision. See #275.

use kiss_conformance::{json, structure_key::*};

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{}/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// Every KISS-Ops op name, from the ops.md-derived manifest.
fn all_ops() -> Vec<String> {
    let m = json::parse(&read("corpus/op_manifest.json")).unwrap();
    m.get("all_ops").unwrap().as_arr().unwrap().iter()
        .filter_map(|j| j.as_str()).map(|s| s.to_string()).collect()
}

fn co() -> OperandSubKey {
    OperandSubKey {
        contig: Contig::Contiguous, bcast_mask: 0x00,
        vec: VecWidth::V4, div: DivBucket::D16, flipped: false,
    }
}

fn shape_key(op_family: &str) -> StructureKey {
    StructureKey {
        op_family: op_family.to_string(),
        dtype: "f32".to_string(),
        target: "cuda:sm89".to_string(),
        index_width: "ix32".to_string(),
        work_class: WorkClass::Grid,
        rank: 2,
        operands: vec![co(), co()],
        reduce: Reduce::None,
        contraction: None,
        acc_mp: None,
    }
}

/// Backs: KISS-CLASSIFY-6.6-0002 — `op_family` is a coarse category, never an op name.
#[test]
fn test_classify_structure_key_is_not_op_identity() {
    let ops = all_ops();

    // Vacuity guards: both vocabularies must be non-empty, or the disjointness below
    // holds trivially and asserts nothing.
    assert!(!ops.is_empty(), "KISS-CLASSIFY-6.6-0002: op manifest is empty — the check would be vacuous");
    assert!(!OP_FAMILIES.is_empty(), "KISS-CLASSIFY-6.6-0002: OP_FAMILIES is empty — the check would be vacuous");

    let collisions: Vec<&&str> = OP_FAMILIES.iter()
        .filter(|f| ops.iter().any(|o| o == **f)).collect();
    assert!(
        collisions.is_empty(),
        "KISS-CLASSIFY-6.6-0002: op_family code(s) {collisions:?} are also KISS-Ops op names. \
         A family code that IS an op name makes the key carry op identity in the one field \
         that is supposed to be a coarse category."
    );

    // DISCRIMINATION CONTROL: the predicate above must be able to fire. A disjointness
    // check that cannot detect a collision passes for a vocabulary that is riddled with
    // them. Probe with a real op name and require the same predicate to flag it.
    let probe = ops[0].as_str();
    let probe_families: Vec<&str> = vec!["gem", probe];
    let probe_hits: Vec<&&str> = probe_families.iter()
        .filter(|f| ops.iter().any(|o| o == **f)).collect();
    assert_eq!(
        probe_hits.len(), 1,
        "DISCRIMINATION: the disjointness predicate failed to flag the real op name `{probe}` — \
         it cannot detect the collision it exists to detect"
    );
}

/// Backs: KISS-CLASSIFY-6.6-0020 — cell-mates are not mutually substitutable.
#[test]
fn test_classify_cell_mates_are_not_substitutable() {
    let ops = all_ops();
    let tok = shape_key("bin").to_token();

    // Vacuity guards, matching this test's sibling: an empty op vocabulary or a token with
    // no fields would make the "no field names an op" check below pass by having nothing to
    // check.
    assert!(!ops.is_empty(), "KISS-CLASSIFY-6.6-0020: op manifest is empty — the check would be vacuous");
    assert!(tok.split('|').count() > 1, "KISS-CLASSIFY-6.6-0020: token has no fields to inspect — the check would be vacuous");

    // No FIELD of a derived token is a KISS-Ops op name — so the token cannot name what
    // it computes, and two computations of one shape land on one cell.
    let named: Vec<&str> = tok.split('|')
        .filter(|f| ops.iter().any(|o| o == *f)).collect();
    assert!(
        named.is_empty(),
        "KISS-CLASSIFY-6.6-0020: token field(s) {named:?} name a KISS-Ops op. If a field \
         named the computation, cell-mates WOULD be distinguishable and this clause would \
         not be needed — so this failing means the clause is wrong, not the code."
    );

    // REMOVED, and the removal is the point. This line used to read:
    //
    //     assert_eq!(shape_key("bin").to_token(), shape_key("bin").to_token(), ...)
    //
    // BOTH SIDES ARE THE SAME EXPRESSION, so it could only ever fail on nondeterminism —
    // it did not exhibit "two computations, one cell", which is what its message claimed.
    // That is the comparison-against-itself shape this suite exists to catch, written into
    // the file whose subject is instruments that cannot fail. What it was reaching for is
    // already asserted above, and asserted properly: no field of the token can name a
    // computation. Determinism of `to_token` is a real property but it is not THIS clause's
    // obligation, and pinning it under §6.6-0020's message would mislabel both.

    // DISCRIMINATION CONTROL: the key is blind to the COMPUTATION, not to everything. A
    // derivation returning a constant would satisfy every collision above and destroy the
    // key, so an axis the key DOES capture must still separate.
    assert_ne!(
        shape_key("bin").to_token(), shape_key("une").to_token(),
        "DISCRIMINATION: two different op_family codes must not collide — if they do, the \
         key is not semantics-blind, it is inert"
    );
}
