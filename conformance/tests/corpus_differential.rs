//! KISS-Conform §6.5-0001 differential: run an implementation-under-test against
//! the frozen corpus and compare under each cell's declared class. Proves teeth —
//! a correct add passes; a normalize-to-+0 add fails the (-0)+(-0) cell.
use kiss_conformance::corpus::{self, Cell, Corpus};
use kiss_conformance::{compare, DeterminismClass};

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
        // Plan A is exact-byte only; the class dispatch is here for when ULP/split arrive.
        match cell.class {
            DeterminismClass::ExactByte => compare(cell.class, &actual, &cell.expected)
                .map_err(|e| format!("tcId {}: {e}", cell.tc_id))?,
            other => return Err(format!("tcId {}: class {other:?} not in Plan A", cell.tc_id)),
        }
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
