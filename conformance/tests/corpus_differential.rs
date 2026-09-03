//! KISS-Conform §6.5-0001 differential: run an implementation-under-test against
//! the frozen corpus and compare under each cell's declared class. Proves teeth —
//! a correct add passes; a normalize-to-+0 add fails the (-0)+(-0) cell.
use kiss_conformance::corpus::{self, Cell, Corpus};
use kiss_conformance::fp::{bf16_to_f32, f32_to_bf16};
use kiss_conformance::semantics::{
    fmax_ieee, fmax_ieee_f64, fmin_ieee, fmin_ieee_f64, max_prop, max_prop_f64, min_prop,
    min_prop_f64,
};
use kiss_conformance::{compare, comparator_for, Comparator, DeterminismClass};

fn bundle() -> Corpus {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-arith.json")).unwrap();
    corpus::load(&text).unwrap()
}

// Apply an f32 binary implementation to a cell and return the result bytes (big-endian).
fn eval_add(cell: &Cell, f: impl Fn(f32, f32) -> f32) -> Vec<u8> {
    let a = f32::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
    let b = f32::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
    f(a, b).to_bits().to_be_bytes().to_vec()
}

fn run_against(c: &Corpus, f: impl Fn(f32, f32) -> f32) -> Result<(), String> {
    for cell in &c.vectors {
        let actual = eval_add(cell, &f);
        // §6.8-0008 precedence: selection via comparator_for, never cell.class directly (#339(a)).
        // Exact-byte is Plan A; compare_under_precedence errors for a not-yet-supported class OR a
        // §6.8-0005 refinement op — the "not in Plan A" guard reached THROUGH the precedence.
        compare_under_precedence(cell, &actual)
            .map_err(|e| format!("tcId {}: {e}", cell.tc_id))?;
    }
    Ok(())
}

#[test]
fn reference_add_passes_every_cell() {
    let c = bundle();
    run_against(&c, kiss_conformance::semantics::add).expect("reference add is conformant");
}

#[test]
fn a_normalize_to_plus_zero_add_is_caught() {
    let c = bundle();
    // A subtly-wrong add that scrubs the sign of a zero result: (-0)+(-0) -> +0.
    let wrong = |a: f32, b: f32| {
        let r = a + b;
        if r == 0.0 { 0.0 } else { r } // normalizes -0.0 to +0.0
    };
    let err = run_against(&c, wrong).expect_err("the harness MUST catch the signed-zero bug");
    assert!(err.contains("tcId 2"), "the (-0)+(-0) cell is the one with teeth: {err}");
}

// ---- §6.13 minmax signed-zero tie cells (issue #74) --------------------------
//
// HARNESS RULE: every cell is class exact-byte, i.e. compared on RAW BITS — a
// float value-compare of `0.0 == -0.0` passes vacuously; the expected values
// are bit patterns. The bf16 cells run promote→compute→round (exact widen to
// f32, f32 compute, RNE round back per §6.16-0003), proving the sign bit
// survives the round-trip.

fn minmax_bundle() -> Corpus {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/ops-minmax-signed-zero.json"
    ))
    .unwrap();
    corpus::load(&text).unwrap()
}

/// Evaluate one §6.13 minmax tie cell with the reference oracle at the cell's
/// dtype, returning big-endian result bytes.
fn eval_minmax(cell: &Cell) -> Vec<u8> {
    let op32: fn(f32, f32) -> f32 = match cell.op.as_str() {
        "max_prop" => max_prop,
        "min_prop" => min_prop,
        "fmax_ieee" => fmax_ieee,
        "fmin_ieee" => fmin_ieee,
        other => panic!("unknown minmax op `{other}`"),
    };
    match cell.dtype.as_str() {
        "f32" => {
            let a = f32::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
            let b = f32::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
            op32(a, b).to_bits().to_be_bytes().to_vec()
        }
        "f64" => {
            let op64: fn(f64, f64) -> f64 = match cell.op.as_str() {
                "max_prop" => max_prop_f64,
                "min_prop" => min_prop_f64,
                "fmax_ieee" => fmax_ieee_f64,
                "fmin_ieee" => fmin_ieee_f64,
                _ => unreachable!(),
            };
            let a = f64::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
            let b = f64::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
            op64(a, b).to_bits().to_be_bytes().to_vec()
        }
        "bf16" => {
            // bf16 minmax is a SELECT (moves an exact input value), NOT promote→compute→round:
            // f32_to_bf16 forces the quiet bit on any NaN (fp.rs), so routing a MOVED sNaN through it
            // QUIETS it, violating §6.8-0010(a) — a moved NaN's payload+sign are exact (#354). Pick the
            // winning operand by the op's §6.15 decomposition on the f32-promoted values (comparison
            // only), and return its EXACT bf16 bits. Correct for non-NaN too: the winner is already an
            // exact bf16 value, so there is nothing to round.
            let a = u16::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
            let b = u16::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
            let (af, bf) = (bf16_to_f32(a), bf16_to_f32(b));
            let (a_nan, b_nan) = (af.is_nan(), bf.is_nan());
            let winner = match cell.op.as_str() {
                "max_prop" => if a_nan { a } else if b_nan { b } else if af >= bf { a } else { b },
                "min_prop" => if a_nan { a } else if b_nan { b } else if af <= bf { a } else { b },
                "fmax_ieee" => if a_nan { b } else if b_nan { a } else if af >= bf { a } else { b },
                "fmin_ieee" => if a_nan { b } else if b_nan { a } else if af <= bf { a } else { b },
                other => panic!("unknown minmax op `{other}`"),
            };
            winner.to_be_bytes().to_vec()
        }
        other => panic!("unknown dtype `{other}`"),
    }
}

/// The 48 signed-zero tie cells (16 rows × f32/f64/bf16) all pass against the
/// reference oracle under the exact-byte (raw-bits) comparator.
#[test]
fn reference_minmax_passes_every_signed_zero_tie_cell() {
    let c = minmax_bundle();
    assert_eq!(c.vectors.len(), 48, "16 rows x 3 dtypes");
    for cell in &c.vectors {
        let actual = eval_minmax(cell);
        assert_eq!(cell.class, DeterminismClass::ExactByte, "tcId {}: tie cells are exact-byte", cell.tc_id);
        compare_under_precedence(cell, &actual)
            .unwrap_or_else(|e| panic!("tcId {} ({} {}): {e}", cell.tc_id, cell.op, cell.dtype));
    }
}

/// Teeth: a `>`-spelled max tie (`a > b ? a : b` — b on ties) is the exact
/// divergence the kiss-ref↔Baracuda step-2 recipe differential caught (fixed by
/// Baracuda@7297f17d). It fails precisely the two mixed-sign seam cells, and
/// ONLY a raw-bits compare can see it — the float value-compare is vacuous.
#[test]
fn a_b_biased_tie_max_is_caught() {
    let c = minmax_bundle();
    let wrong = |a: f32, b: f32| if a > b { a } else { b }; // b on ties
    // The harness rule, executable: the wrong result +0.0 compares EQUAL to the
    // pinned -0.0 by value (vacuous pass), and differs on raw bits (the catch).
    let seam = wrong(-0.0f32, 0.0f32);
    assert!(seam == -0.0, "float == across ±0 is vacuously true — why the rule pins raw bits");
    assert_ne!(seam.to_bits(), (-0.0f32).to_bits(), "raw bits see the lost sign");
    let mut caught = Vec::new();
    for cell in c.vectors.iter().filter(|v| v.op == "max_prop" && v.dtype == "f32") {
        let a = f32::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
        let b = f32::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
        let actual = wrong(a, b).to_bits().to_be_bytes().to_vec();
        if compare_under_precedence(cell, &actual).is_err() {
            caught.push((a.to_bits(), b.to_bits()));
        }
    }
    assert_eq!(
        caught,
        vec![(0x0000_0000, 0x8000_0000), (0x8000_0000, 0x0000_0000)],
        "the b-biased tie must fail exactly the (+0,-0) and (-0,+0) seam cells"
    );
}

// ---- §6.13 minmax ORDINARY (strict-inequality) cells (#333) ------------------
//
// These SEPARATE max-family from min-family — the discrimination the two files
// above cannot give: the tie set returns operand `a` under both cmp_ge and cmp_le
// (so a max<->min swap is invisible), and the NaN set short-circuits before the
// distinguishing branch. Every cell here is a STRICT inequality between two
// non-NaN operands, so max returns the larger and min the smaller; swapping the
// max/min family reddens every cell. Scope: separates max-from-min, NOT
// prop-from-ieee (those agree on non-NaN; #329's NaN rows tell them apart).
//
// Two independent derivations agree on all 24 (including orientation — which
// family returns the larger operand): these hand-derived (decomposition-traced)
// cells and kiss-ref's blind eval_op (kiss-ref main @ ccf294b4, test minmax_ordinary_derive).

fn ordinary_minmax_bundle() -> Corpus {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/ops-minmax-ordinary.json"
    ))
    .unwrap();
    corpus::load(&text).unwrap()
}

/// Compare under §6.8-0008 PRECEDENCE — selection routed through `comparator_for`,
/// never the declared class directly. For every minmax op this resolves to
/// `ClassDefault(exact-byte)`, so it is behaviour-preserving today. #339(a) migrated
/// every `compare(cell.class, ..)` site in this file onto it, so no site selects by the
/// declared class directly; it is correct the moment a §6.8-0005 refinement op is added.
fn compare_under_precedence(cell: &Cell, actual: &[u8]) -> Result<(), String> {
    match comparator_for(&cell.op, cell.class) {
        Comparator::ClassDefault(class) => compare(class, actual, &cell.expected),
        Comparator::OpNamedRefinement(name) => {
            Err(format!("op-named refinement `{name}` is not part of this slice"))
        }
    }
}

/// All 24 strict-inequality cells pass against the reference oracle (same
/// `eval_minmax` as the tie set — the difference is the DATA, which is where the
/// max-from-min discrimination lives).
#[test]
fn reference_minmax_passes_every_ordinary_cell() {
    let c = ordinary_minmax_bundle();
    assert_eq!(c.vectors.len(), 24, "4 ops x 3 dtypes x 2 directions");
    for cell in &c.vectors {
        let actual = eval_minmax(cell);
        compare_under_precedence(cell, &actual)
            .unwrap_or_else(|e| panic!("tcId {} ({} {}): {e}", cell.tc_id, cell.op, cell.dtype));
    }
}

/// TEETH — the discrimination this file exists to provide. An implementation that
/// SWAPS the max/min family (min where the op says max, and vice versa) reddens on
/// EVERY cell, because each is a strict inequality with distinct operands. The
/// paired control proves the teeth come from the DATA, not the mutation: the SAME
/// swap over the tie set reddens NOTHING (operand `a` wins under both comparisons),
/// which is exactly why the tie file cannot separate max from min. The prop<->ieee
/// axis is deliberately NOT tested here — those agree on non-NaN and are #329's.
#[test]
fn a_max_min_family_swap_reddens_every_cell() {
    fn swap_family(op: &str) -> &str {
        match op {
            "max_prop" => "min_prop",
            "min_prop" => "max_prop",
            "fmax_ieee" => "fmin_ieee",
            "fmin_ieee" => "fmax_ieee",
            other => other,
        }
    }
    fn swapped_eval(cell: &Cell) -> Vec<u8> {
        let mut c2 = cell.clone();
        c2.op = swap_family(cell.op.as_str()).to_string();
        eval_minmax(&c2)
    }

    let ord = ordinary_minmax_bundle();
    let reddened = ord
        .vectors
        .iter()
        .filter(|&cell| compare_under_precedence(cell, &swapped_eval(cell)).is_err())
        .count();
    assert_eq!(
        reddened,
        ord.vectors.len(),
        "every strict-inequality cell MUST red under a max<->min family swap — the \
         discrimination the tie set (all `a`) and NaN set (short-circuit) cannot give"
    );

    // Paired control: the SAME swap over the TIE set reddens NOTHING, proving the
    // teeth are in the strict-inequality DATA rather than the mutation.
    let tie_reddened = minmax_bundle()
        .vectors
        .iter()
        .filter(|&cell| compare_under_precedence(cell, &swapped_eval(cell)).is_err())
        .count();
    assert_eq!(
        tie_reddened, 0,
        "a max<->min swap over the TIE set must red NOTHING — operand `a` wins under both \
         cmp_ge and cmp_le on a tie, which is why the tie file cannot separate max from min"
    );
}

// ---- §6.13 minmax MOVED-NaN cells (§6.8-0010(a), #329) ------------------------
//
// 96 vectors from kiss-ref's `minmax_nan_corpus` generator (kiss-ref main @ c74e472;
// baracuda CUDA sm_89, both arms, @ e31f47548af9): 4 ops × {f32,f64,bf16} × 8 NaN cases.
// A minmax NaN output is a MOVED input value pinned payload AND sign (§6.8-0010(a)), so
// class exact-byte compares every bit. The both-NaN rows carry DISTINCT payloads, so a
// payload-blind path (`agree()`) would silently pass a wrong payload — which is why these
// route through compare_under_precedence (§6.8-0008 precedence → exact-byte), never
// `agree()`; the differ.rs #339(a) guard refuses a moved-NaN op from `run_binary` anyway.

fn nan_minmax_bundle() -> Corpus {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/corpus/ops-minmax-nan.json"
    ))
    .unwrap();
    corpus::load(&text).unwrap()
}

/// The cross-implementation differential: KISS's reference semantics (`eval_minmax`) must
/// reproduce all 96 of kiss-ref's independently-derived, CUDA-measured moved-NaN values,
/// bit-for-bit under exact-byte — two implementations agreeing on payload AND sign.
#[test]
fn reference_minmax_passes_every_nan_cell() {
    let c = nan_minmax_bundle();
    assert_eq!(c.vectors.len(), 96, "4 ops x 3 dtypes x 8 NaN cases");
    let mut diverged = Vec::new();
    for cell in &c.vectors {
        let actual = eval_minmax(cell);
        if compare_under_precedence(cell, &actual).is_err() {
            let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ");
            eprintln!(
                "DIVERGE tcId {} {} {} tags={:?} expected={} actual={}",
                cell.tc_id, cell.op, cell.dtype, cell.tags, hex(&cell.expected), hex(&actual)
            );
            diverged.push(cell.dtype.clone());
        }
    }
    let by_dtype = |dt: &str| diverged.iter().filter(|d| *d == dt).count();
    assert!(
        diverged.is_empty(),
        "{}/{} diverge — f32:{} f64:{} bf16:{}",
        diverged.len(), c.vectors.len(), by_dtype("f32"), by_dtype("f64"), by_dtype("bf16")
    );
}

/// TEETH — a wrong NaN payload is caught. Flip the low payload byte of the reference result
/// (a still-NaN, wrong-payload output on the NaN-expected rows) — exact-byte MUST red EVERY
/// cell. The both-NaN distinct-payload rows are exactly where a NaN-blind comparator (`agree()`)
/// would silently pass; asserting they exist keeps this from proving a vacuous point.
#[test]
fn a_wrong_nan_payload_is_caught() {
    let c = nan_minmax_bundle();
    assert!(
        c.vectors.iter().any(|v| v.tags.iter().any(|t| t == "distinct-payload")),
        "there must be both-NaN distinct-payload rows — the payload-blind discrimination cases"
    );
    let caught = c
        .vectors
        .iter()
        .filter(|cell| {
            let mut wrong = eval_minmax(cell);
            let last = wrong.len() - 1;
            wrong[last] ^= 0x01;
            compare_under_precedence(cell, &wrong).is_err()
        })
        .count();
    assert_eq!(
        caught,
        c.vectors.len(),
        "exact-byte MUST red a wrong payload on EVERY cell — a NaN-blind comparator would pass the NaN rows"
    );
}
