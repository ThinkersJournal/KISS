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
/// bf16 minmax as a raw-bit SELECT (§6.13 decomposition, KISS-OPS-6.16-0009): the winning operand's
/// EXACT bf16 bits, never a promote→compute→round value. Routing a MOVED operand through
/// `f32_to_bf16` quiets a signaling NaN (it forces the quiet bit, fp.rs), violating §6.8-0010(a)
/// (#354). The f32 promotion is used ONLY to order the operands; the RESULT bits come from the
/// source. Correct for non-NaN too — the winner is already an exact bf16 value, nothing to round.
fn bf16_minmax_select(op: &str, a: u16, b: u16) -> u16 {
    let (af, bf) = (bf16_to_f32(a), bf16_to_f32(b));
    let (a_nan, b_nan) = (af.is_nan(), bf.is_nan());
    match op {
        // NaN-propagating: a moved NaN wins; else the ordered pick, operand a on ties (§6.13).
        "max_prop" => if a_nan { a } else if b_nan { b } else if af >= bf { a } else { b },
        "min_prop" => if a_nan { a } else if b_nan { b } else if af <= bf { a } else { b },
        // IEEE NaN-suppressing: a NaN yields the OTHER operand; else the ordered pick.
        "fmax_ieee" => if a_nan { b } else if b_nan { a } else if af >= bf { a } else { b },
        "fmin_ieee" => if a_nan { b } else if b_nan { a } else if af <= bf { a } else { b },
        other => panic!("unknown minmax op `{other}`"),
    }
}

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
            // §6.13 minmax is a raw-bit SELECT (KISS-OPS-6.16-0009): the winner's EXACT bf16 bits,
            // NOT promote→compute→round, which quiets a moved sNaN via f32_to_bf16 (#354).
            let a = u16::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
            let b = u16::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
            bf16_minmax_select(&cell.op, a, b).to_be_bytes().to_vec()
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

// ---- KISS-OPS-6.16-0009: bf16 minmax MOVES, it does not round (#354) ----------

/// KISS-OPS-6.16-0009 born-red. A NaN-propagating bf16 minmax MOVES the operand's exact bits; a
/// promote→compute→round path QUIETS a signaling NaN (`f32_to_bf16` forces the quiet bit). These 6
/// cases red before the fix and green after; inline (not #329's corpus) so the fix does not depend
/// on the corpus it unblocks. `a=7F81` is a bf16 sNaN — the move must preserve it, not quiet it to
/// `7FC1`. Values are kiss-ref's CUDA-measured (sm_89) expectations.
#[test]
fn test_ops_bf16_minmax_moves_not_rounds() {
    // Backs: KISS-OPS-6.16-0009 — reverse backing form. The clause is also forward-bound by the §9
    // traceability row; this makes the test→clause link explicit for the backing-form scanner
    // (convention 15: a MENTION in a doc comment/assert message is not a backing).
    let cases: &[(&str, u16, u16, u16)] = &[
        ("max_prop", 0x7F81, 0x3F80, 0x7F81), // sNaN a, 1.0 b → propagate sNaN a
        ("min_prop", 0x7F81, 0x3F80, 0x7F81),
        ("max_prop", 0x3F80, 0x7F81, 0x7F81), // sNaN b → propagate sNaN b
        ("min_prop", 0x3F80, 0x7F81, 0x7F81),
        ("max_prop", 0x7F81, 0x7FD2, 0x7F81), // both NaN, a (sNaN) wins → moved exactly
        ("min_prop", 0x7F81, 0x7FD2, 0x7F81),
    ];
    for &(op, a, b, expected) in cases {
        let got = bf16_minmax_select(op, a, b);
        assert_eq!(
            got, expected,
            "{op}(bf16 {a:04X}, {b:04X}) must MOVE the sNaN exactly (got {got:04X}); a promote→round \
             path quiets it to 7FC1, violating §6.8-0010(a) / KISS-OPS-6.16-0009"
        );
    }
}

/// Behaviour-preservation: for every NON-NaN bf16 pair the select is IDENTICAL to the old
/// promote→compute→round path — the winner is an exact bf16 value, so `f32_to_bf16` is the identity
/// on it and there is nothing to round. Exercised on subnormals and the ties-to-even boundary
/// (architect): the two places a rounding difference could hide if the winner were ever a COMPUTED
/// value (it is not). Doubles as a check that the select's ordering matches semantics::{max,min}_*.
#[test]
fn test_bf16_minmax_select_is_behaviour_preserving_for_non_nan() {
    // Backs: KISS-OPS-6.16-0009 — the ORDINARY-finite arm. The clause's "no arithmetic → nothing to
    // round" covers a non-NaN winner too: it is already an exact bf16 value. This demonstrates that
    // arm (select == round-trip ⇒ the round was a no-op), which the NaN born-red does not cover.
    let samples: &[u16] = &[
        0x0000, 0x8000, // ±0
        0x0001, 0x8001, // ± smallest subnormal
        0x007F, 0x807F, // ± largest subnormal
        0x0080, 0x8080, // ± smallest normal
        0x3F7F, 0x3F80, 0x3F81, 0x4000, // 1.0-eps, 1.0, 1.0+eps, 2.0 (tie-boundary neighbours)
        0x7F7F, 0xFF7F, // ± largest finite
        0x7F80, 0xFF80, // ±inf (non-NaN)
    ];
    let old_round_trip = |op: &str, a: u16, b: u16| -> u16 {
        let op32: fn(f32, f32) -> f32 = match op {
            "max_prop" => max_prop,
            "min_prop" => min_prop,
            "fmax_ieee" => fmax_ieee,
            "fmin_ieee" => fmin_ieee,
            _ => unreachable!(),
        };
        f32_to_bf16(op32(bf16_to_f32(a), bf16_to_f32(b)))
    };
    for op in ["max_prop", "min_prop", "fmax_ieee", "fmin_ieee"] {
        for &a in samples {
            for &b in samples {
                assert_eq!(
                    bf16_minmax_select(op, a, b),
                    old_round_trip(op, a, b),
                    "non-NaN {op}(bf16 {a:04X}, {b:04X}): select must equal the old round-trip"
                );
            }
        }
    }
}

// ---- §6.16-0009 reaches every NO-ARITHMETIC op, not only minmax (#390) -------
//
// The clause's TRIGGER is "an op whose §6.13 reference decomposition contains no
// arithmetic"; `max_prop`/`min_prop` are its EXAMPLE, not its extent. `neg`, `abs` and
// `copysign` are defined in §6.13 as raw-bit sign-bit operations ("clear the sign bit
// (raw-bit); NaN payload preserved"), so the trigger reaches them and nothing asserted
// it — a wrong-direction bf16 `neg` or `abs` was conforming by omission.
//
// ⚠️ The two most likely to be missed are `neg` and `abs`, and the reason is a label:
// the §6.13 op table files both under family `arithmetic` while their SEMANTICS are
// raw-bit. Reading the family column to find the clause's domain finds neither.

/// bf16 `neg`/`abs` as the raw-bit moves §6.13 defines: the operand's bits with the sign
/// bit flipped or cleared. Nothing is computed, so there is nothing to round.
fn bf16_move_unary(op: &str, x: u16) -> u16 {
    match op {
        "neg" => x ^ 0x8000,
        "abs" => x & 0x7FFF,
        other => panic!("unknown move unary `{other}`"),
    }
}

/// bf16 `copysign` (§6.9-0002): magnitude of `a`, sign bit of `b`, raw-bit — specified
/// that way precisely so a moved NaN's sign survives.
fn bf16_copysign(a: u16, b: u16) -> u16 {
    (a & 0x7FFF) | (b & 0x8000)
}

/// The HAZARD path the clause names non-conforming: promote to `f32`, apply, round back.
/// `f32_to_bf16` forces the quiet bit on a NaN (fp.rs), so a moved signaling NaN is
/// quieted. This is fuel's `chassis/unary.rs` shape and it is here to be MEASURED, not
/// used — the test asserts the two paths DISAGREE.
fn bf16_promote_round_unary(op: &str, x: u16) -> u16 {
    let v = bf16_to_f32(x);
    f32_to_bf16(match op {
        "neg" => -v,
        "abs" => v.abs(),
        other => panic!("unknown move unary `{other}`"),
    })
}

fn bf16_promote_round_copysign(a: u16, b: u16) -> u16 {
    f32_to_bf16(bf16_to_f32(a).copysign(bf16_to_f32(b)))
}

/// True iff `x` is a bf16 signaling NaN: exponent all ones, mantissa non-zero, quiet
/// bit (mantissa MSB, 0x0040) CLEAR.
fn bf16_is_snan(x: u16) -> bool {
    (x & 0x7F80) == 0x7F80 && (x & 0x007F) != 0 && (x & 0x0040) == 0
}

/// Backs: KISS-OPS-6.16-0009 — the clause reaches `neg`/`abs`/`copysign`, not only minmax.
///
/// Two assertions per case, and the SECOND is the one with teeth. Asserting only that the
/// raw-bit path preserves the bits would be near-tautological — `x ^ 0x8000` preserves a
/// payload by construction, and a test that can only pass proves nothing about the clause.
/// So each case also measures the promote→compute→round path and asserts it DIVERGES by
/// quieting. That is the clause's own claim ("a promote-to-f32-and-round-back
/// implementation of such an op is therefore non-conforming for a narrow float") made
/// executable: the hazard is demonstrated on the same input, not assumed.
#[test]
fn test_ops_bf16_move_ops_preserve_snan_bits() {
    // (op, a, b, required) — `b` is the sign donor for copysign, ignored by the unaries.
    // 0x7F81 is a minimum-payload bf16 sNaN; 0x7FBF is a MAXIMUM-payload one (mantissa
    // 0x3F, quiet bit still clear), present so the assertion covers payload bits beyond
    // the LSB — a path that preserved only the low bit would pass on 0x7F81 alone.
    let cases: &[(&str, u16, u16, u16)] = &[
        ("neg", 0x7F81, 0, 0xFF81),      // +sNaN -> -sNaN, payload intact
        ("neg", 0xFF81, 0, 0x7F81),      // and back
        ("neg", 0x7FBF, 0, 0xFFBF),      // max payload
        ("abs", 0x7F81, 0, 0x7F81),      // already positive: an exact identity move
        ("abs", 0xFF81, 0, 0x7F81),      // sign cleared, payload intact
        ("abs", 0xFFBF, 0, 0x7FBF),      // max payload
        ("copysign", 0x7F81, 0x8000, 0xFF81), // sign taken from b
        ("copysign", 0xFF81, 0x0000, 0x7F81),
        ("copysign", 0x7FBF, 0x8000, 0xFFBF),
    ];
    for &(op, a, b, required) in cases {
        assert!(bf16_is_snan(a), "fixture error: {a:04X} is not a bf16 sNaN");

        let moved = if op == "copysign" { bf16_copysign(a, b) } else { bf16_move_unary(op, a) };
        assert_eq!(
            moved, required,
            "{op}(bf16 {a:04X}, {b:04X}) must MOVE the operand's exact bits (got {moved:04X}, \
             want {required:04X}): §6.13 defines it raw-bit, so §6.16-0009 leaves nothing to round"
        );
        // the moved result is still SIGNALING — the property a promotion destroys.
        assert!(
            bf16_is_snan(moved),
            "{op}(bf16 {a:04X}) returned {moved:04X}, which is no longer a signaling NaN"
        );

        // ⚠️ TEETH: the promote→round path must DISAGREE, by quieting. If this ever stops
        // holding, the case above has become unable to distinguish a conforming
        // implementation from the one the clause names non-conforming.
        let promoted =
            if op == "copysign" { bf16_promote_round_copysign(a, b) } else { bf16_promote_round_unary(op, a) };
        assert_ne!(
            promoted, required,
            "{op}(bf16 {a:04X}, {b:04X}): the promote-and-round path agreed with the required \
             answer, so this case cannot tell the two implementations apart"
        );
        assert_eq!(
            promoted & 0x0040,
            0x0040,
            "{op}(bf16 {a:04X}, {b:04X}): promote-and-round gave {promoted:04X}; the divergence \
             this case pins is QUIETING, so the quiet bit must be the thing that differs"
        );
    }
}

/// Backs: KISS-OPS-6.16-0009 — the ORDINARY arm, mirroring the minmax behaviour-preservation
/// test. For every NON-NaN bf16 the raw-bit move and the promote→round path AGREE, which is
/// what makes the clause a scope clarification rather than a behaviour change: it bites
/// exactly on the NaN payload, and nowhere else. Without this, "the two paths differ" would
/// be an unbounded claim rather than one confined to signaling NaNs.
#[test]
fn test_bf16_move_ops_are_behaviour_preserving_for_non_nan() {
    let samples: &[u16] = &[
        0x0000, 0x8000, // ±0
        0x0001, 0x8001, // ± smallest subnormal
        0x007F, 0x807F, // ± largest subnormal
        0x0080, 0x8080, // ± smallest normal
        0x3F7F, 0x3F80, 0x3F81, 0x4000, // tie-boundary neighbours around 1.0
        0x7F7F, 0xFF7F, // ± largest finite
        0x7F80, 0xFF80, // ±inf
    ];
    for &x in samples {
        for op in ["neg", "abs"] {
            assert_eq!(
                bf16_move_unary(op, x),
                bf16_promote_round_unary(op, x),
                "non-NaN {op}(bf16 {x:04X}): the raw-bit move must equal the round-trip"
            );
        }
        for &b in samples {
            assert_eq!(
                bf16_copysign(x, b),
                bf16_promote_round_copysign(x, b),
                "non-NaN copysign(bf16 {x:04X}, {b:04X}): raw-bit must equal the round-trip"
            );
        }
    }
}
