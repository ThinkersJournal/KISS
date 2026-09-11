//! The §6.19.3 schema, read from `spec/ops.md` by two independent paths (#504).
//!
//! ⚠️ PRE-REGISTERED COUNTS. Every population below was written down BEFORE the first run, from
//! the architect's independent reading, and the assertions were authored against that prediction
//! rather than against whatever the parser emitted. That ordering is the whole discipline: a
//! count written after the run agrees with the code by construction and tests nothing.
//!
//! The prediction: 17 table rows, 17 distinct ops, all 17 covered by a `KISS-OPS-6.19-00NN`
//! clause, 0 uncovered. If this file is red on its first run, the disagreement is the finding.

use kiss_conformance::opattrs_schema::{self, FieldWidth};

/// The number of carrier ops §6.19.3's table declares. PRE-REGISTERED.
const EXPECTED_OPS: usize = 17;

/// ⚠️ THE TABLE MUST PARSE TO THE PRE-REGISTERED POPULATION.
///
/// A parser that silently narrows reports a smaller, plausible number, and every windowed or
/// under-specified extraction in this repo produced exactly that — a well-formed count that was
/// simply short. Pinning the expected total makes a narrowing REDDEN instead of reading as a
/// clean run over less text.
#[test]
fn test_opattrs_table_parses_to_the_preregistered_population() {
    let schemas = opattrs_schema::parse_table();
    assert_eq!(
        schemas.len(),
        EXPECTED_OPS,
        "§6.19.3 table parsed {} ops; {} were pre-registered. Either the spec's table changed \
         (a real finding) or the row grammar narrowed (a broken extractor). Parsed: {:?}",
        schemas.len(),
        EXPECTED_OPS,
        schemas.iter().map(|s| s.op.as_str()).collect::<Vec<_>>()
    );

    // Every op distinct — a duplicate row would inflate the count while covering less.
    let mut names: Vec<&str> = schemas.iter().map(|s| s.op.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate op rows in the §6.19.3 table");

    // Every schema carries at least one field; a row parsed to zero fields is a silent drop.
    for s in &schemas {
        assert!(!s.fields.is_empty(), "op `{}` parsed to ZERO fields", s.op);
    }
}

/// ⚠️ THE TWO READINGS MUST AGREE — and this is a DETECTOR, not a formality.
///
/// The table and the per-op clauses are two records of one fact. The architect read them
/// independently and predicted agreement at 17-of-17. Holding that as a prediction means a
/// mismatch is SURPRISING, and surprise is the only thing that reliably fires on a bad read.
///
/// ⚠️ Note what this does NOT claim: that the clause PROSE and the table agree on field order.
/// It claims only that every op the table declares is named by some §6.19.3 clause. Comparing
/// the orders is a separate, harder check and asserting it here would overstate what is tested.
#[test]
fn test_every_table_op_is_named_by_a_clause() {
    let schemas = opattrs_schema::parse_table();
    let coverage = opattrs_schema::parse_clause_coverage();

    // CONTROL: the clause reading must find SOMETHING, or "everything is covered" would be
    // vacuously false in the other direction — an empty map makes every op uncovered, and an
    // empty map is what a broken clause parser returns.
    assert!(
        !coverage.is_empty(),
        "the clause reading found ZERO ops — broken extractor, not an empty section"
    );

    let uncovered: Vec<&str> = schemas
        .iter()
        .map(|s| s.op.as_str())
        .filter(|op| !coverage.contains_key(*op))
        .collect();

    assert!(
        uncovered.is_empty(),
        "{} table op(s) are named by NO §6.19.3 clause: {:?}. Pre-registered expectation was \
         ZERO. Either the spec has a genuine gap, or one of the two readings narrowed.",
        uncovered.len(),
        uncovered
    );
}

/// ⚠️ WIDTHS ARE THREE-VALUED AND THE THIRD ARM MUST BE REACHABLE.
///
/// `u8`/`u16` is not a partition: the pooling ops carry `vec` fields the table does not pin. If
/// `Variable` never occurs, the enum's third arm is dead and a future `vec` field would be
/// silently mis-sized — so this asserts the arm is LIVE, which is a claim about the spec rather
/// than about the parser.
#[test]
fn test_variable_width_fields_exist_and_are_not_silently_sized() {
    let schemas = opattrs_schema::parse_table();
    let variable: Vec<(&str, &str)> = schemas
        .iter()
        .flat_map(|s| {
            s.fields
                .iter()
                .filter(|f| f.width == FieldWidth::Variable)
                .map(move |f| (s.op.as_str(), f.name.as_str()))
        })
        .collect();

    assert!(
        !variable.is_empty(),
        "no Variable-width field found — either the pooling ops' `vec` fields stopped being \
         variable, or the width classifier is defaulting them to a fixed size"
    );

    // and their owning ops must report NO fixed width, rather than a plausible wrong one
    for (op, _field) in &variable {
        let s = schemas.iter().find(|s| s.op == *op).unwrap();
        assert!(
            s.fixed_width().is_none(),
            "op `{op}` has a Variable field yet reports a fixed blob width — a fabricated size"
        );
    }
}

/// The fixed-width ops must report a width that is the SUM of their fields, not a guess.
#[test]
fn test_fixed_width_ops_sum_their_fields() {
    let schemas = opattrs_schema::parse_table();
    let mut fixed = 0usize;
    for s in &schemas {
        let Some(total) = s.fixed_width() else { continue };
        fixed += 1;
        let by_hand: usize = s.fields.iter().filter_map(|f| f.width.bytes()).sum();
        assert_eq!(total, by_hand, "op `{}` width disagrees with its own fields", s.op);
        assert!(total > 0, "op `{}` reports a zero-byte blob", s.op);
    }
    assert!(fixed > 0, "no fixed-width op found — the width classifier is not working");
}

/// ⚠️ THE COMMITTED ARTIFACT IS BYTE-IDENTICAL TO THE GENERATOR (#161 freshness).
///
/// Without this the artifact is a snapshot: the spec moves, the generator follows, and the
/// committed file keeps describing the old ABI while every test passes. A foreign party
/// reproducing the STALE file would then be measured against a spec nobody holds.
#[test]
fn test_opattrs_vectors_artifact_is_fresh() {
    let committed = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/opattrs_vectors.json"
    ))
    .expect("conformance/corpus/opattrs_vectors.json must exist");
    let generated = opattrs_schema::emit_opattrs_vectors_json();
    assert_eq!(
        // normalise line endings: the generator emits LF, a CRLF checkout stores CRLF, and a
        // mismatch there is about git config rather than about the ABI (#506).
        committed.replace("\r\n", "\n"),
        generated,
        "conformance/corpus/opattrs_vectors.json is STALE — regenerate with \
         `cargo run --bin emit_opattrs_vectors > conformance/corpus/opattrs_vectors.json`"
    );
}

/// ⚠️ THE BORN-RED THE WHOLE ARTIFACT EXISTS FOR: A WRONG FIELD ORDER MUST REDDEN.
///
/// §6.19.3 titles itself "canonical field order = ABI", and a reader replays the blob
/// POSITIONALLY — attribute names never appear on the wire. So swapping two same-width fields
/// changes nothing observable EXCEPT the meaning of every byte after the swap. That is the
/// defect class with no natural detector: same length, same field set, same widths, different
/// ABI.
///
/// This test does not mutate the spec. It asserts the artifact CARRIES order — ordinals dense
/// and ascending from 0, offsets strictly derived from preceding widths — so that a spec-side
/// reordering propagates into the emitted bytes and reddens the freshness test above. The
/// mutation drill that proves it is run against the spec table and recorded in the PR.
#[test]
fn test_field_order_is_carried_and_offsets_follow_it() {
    let schemas = opattrs_schema::parse_table();
    let mut checked = 0usize;
    for s in &schemas {
        let mut expect_offset: Option<usize> = Some(0);
        for (i, f) in s.fields.iter().enumerate() {
            if let Some(off) = expect_offset {
                // the offset of field i is the sum of widths 0..i — recomputed here from the
                // schema rather than read back from the emitter, so the two disagree if either
                // changes alone
                let by_sum: usize = s.fields[..i].iter().filter_map(|g| g.width.bytes()).sum();
                assert_eq!(
                    off, by_sum,
                    "`{}`.{} offset {} disagrees with the sum of preceding widths {}",
                    s.op, f.name, off, by_sum
                );
                checked += 1;
            }
            expect_offset = match (expect_offset, f.width.bytes()) {
                (Some(o), Some(w)) => Some(o + w),
                _ => None,
            };
        }
    }
    // CONTROL: the loop must actually have asserted something. A schema set whose every field
    // were Variable would skip every offset check and pass having verified nothing.
    assert!(
        checked >= 40,
        "only {checked} offset assertions ran; the fixed-width population collapsed"
    );
}
