// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bind-time obligations (KISS-Contract §6.12-0001 to -0006, R1.22, board 379). Every arm drives the
//! SHIPPED `bind_expr`, and the marking test reads the shipped spec text.

use kiss_conformance::bind::{bind_expr, BindDecline};
use kiss_conformance::dispatch_expr::LaunchScalars;

const MAX: &str = "9223372036854775807";

fn env(n: i64) -> LaunchScalars {
    LaunchScalars {
        rank: 2,
        n,
        extents: vec![vec![3, 4]],
        strides: vec![vec![4, 1]],
        idx_extents: vec![],
        off: vec![0],
        ws_bytes: None,
        param: vec![],
    }
}

// Backs[enforced]: KISS-CONTRACT-6.12-0001 — one expression binds to a different value under each bound environment; the value is computed at bind time, not fixed at build time
#[test]
fn test_contract_bind_environment_definition() {
    for n in [1, 12, 4096] {
        assert_eq!(bind_expr("ceil_div(n, 4) * 4", &env(n), None), Ok((n + 3) / 4 * 4));
    }
    assert_eq!(bind_expr("extents0[1] * 2", &env(1), None), Ok(8));
}

// Backs[enforced]: KISS-CONTRACT-6.12-0002 — every §6.12 bind-time clause says "at bind time" in its first sentence and names its code; a malformed expression is a build-time fault, never one of the bind-time codes
#[test]
fn test_contract_bind_time_marking() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/contract.md"))
        .expect("read spec/contract.md");
    // the spec hard-wraps its lines: collapse every whitespace run so a phrase split by a wrap matches.
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for (id, code) in [
        ("0003", "unbound_symbol"),
        ("0004", "extent_out_of_range"),
        ("0005", "affine_overflow"),
        ("0007", "division_by_zero"),
        ("0008", "ceil_div_divisor"),
    ] {
        let head = format!("- **KISS-CONTRACT-6.12-{id}** — ");
        let at = text.find(&head).unwrap_or_else(|| panic!("clause 6.12-{id} missing"));
        let rest = &text[at + head.len()..];
        let first = rest.split(". ").next().unwrap();
        assert!(first.contains("at bind time"), "6.12-{id}: first sentence lacks \"at bind time\"");
        let body = &rest[..rest.find(" - **KISS-CONTRACT-").unwrap_or(rest.len())];
        assert!(body.contains(&format!("`{code}`")), "6.12-{id} does not name `{code}`");
    }
    let e = env(1);
    for bad in ["n +", "wobble(", "n[0]"] {
        let r = bind_expr(bad, &e, None);
        assert!(matches!(r, Err(BindDecline::BuildTime(_))), "{bad}: {r:?}");
        assert_eq!(r.unwrap_err().code(), "build_time");
    }
}

// Backs[enforced]: KISS-CONTRACT-6.12-0003 — a symbol the environment does not assign declines `unbound_symbol`; no zero or default is substituted
#[test]
fn test_contract_bind_unbound_symbol() {
    let e = env(12);
    // ws_bytes is in the vocabulary but this environment assigns none.
    for ex in ["ws_bytes", "n + ws_bytes", "param0"] {
        let r = bind_expr(ex, &e, None);
        assert!(matches!(r, Err(BindDecline::UnboundSymbol(_))), "{ex}: {r:?}");
        assert_eq!(r.unwrap_err().code(), "unbound_symbol");
    }
}

// Backs[enforced]: KISS-CONTRACT-6.12-0004 — a negative final value, or one outside a declared range, declines `extent_out_of_range`
#[test]
fn test_contract_bind_extent_range() {
    let e = env(5);
    assert_eq!(bind_expr("n - 6", &e, None), Err(BindDecline::ExtentOutOfRange(-1)));
    assert_eq!(bind_expr("n", &e, Some((1, 5))), Ok(5));
    assert_eq!(bind_expr("n", &e, Some((1, 4))), Err(BindDecline::ExtentOutOfRange(5)));
    assert_eq!(bind_expr("n - 5", &e, Some((1, 4))), Err(BindDecline::ExtentOutOfRange(0)));
    // the gate is on the FINAL value: a negative intermediate is legal.
    assert_eq!(bind_expr("(n - 9) * (0 - 1)", &e, None), Ok(4));
    assert_eq!(BindDecline::ExtentOutOfRange(0).code(), "extent_out_of_range");
}

// Backs[enforced]: KISS-CONTRACT-6.12-0005 — a `+`, `-` or `*` leaving the signed 64-bit range declines `affine_overflow`; i64::MIN / -1 likewise; no wrap, no saturation
#[test]
fn test_contract_bind_affine_overflow() {
    let e = env(2);
    for ex in [
        format!("{MAX} + n"),
        format!("{MAX} * n"),
        format!("(0 - {MAX} - 1) - n"),
        format!("(0 - {MAX} - 1) / (0 - 1)"),
    ] {
        assert_eq!(bind_expr(&ex, &e, None), Err(BindDecline::AffineOverflow), "{ex}");
    }
    // the last representable value is not an overflow.
    assert_eq!(bind_expr(&format!("{MAX} - 2 + n"), &e, None), Ok(i64::MAX));
    assert_eq!(BindDecline::AffineOverflow.code(), "affine_overflow");
}

// Backs[enforced]: KISS-CONTRACT-6.12-0006 — left to right, symbol lookup before the arithmetic step, the first failing step names the decline, the range gate runs last
#[test]
fn test_contract_bind_evaluation_order() {
    let e = env(2);
    // overflow in term 1 beats an unbound symbol in term 2.
    assert_eq!(
        bind_expr(&format!("({MAX} + n) + ws_bytes"), &e, None),
        Err(BindDecline::AffineOverflow)
    );
    // an unbound symbol in term 1 beats an overflow in term 2.
    assert!(matches!(
        bind_expr(&format!("ws_bytes + ({MAX} + n)"), &e, None),
        Err(BindDecline::UnboundSymbol(_))
    ));
    // a step failure beats the range gate: the value would also have been negative.
    assert_eq!(
        bind_expr(&format!("(0 - 1) + (0 - {MAX} - {MAX})"), &e, None),
        Err(BindDecline::AffineOverflow)
    );
    // the range gate runs last, only after every step succeeded.
    assert_eq!(bind_expr("n - 3", &e, None), Err(BindDecline::ExtentOutOfRange(-1)));
}

// Backs[enforced]: KISS-CONTRACT-6.5-0019 — two bundle slots whose byte hulls intersect under the bound environment decline `slot_overlap`; touching hulls, empty slots and an unbound symbol are told apart
#[test]
fn test_contract_bind_slot_overlap() {
    use kiss_conformance::layout::{parse_layout, Layout, LayoutContext, OutputLayout};
    use kiss_conformance::bind::check_slot_overlap;
    let slots = |t: &str, e: &LaunchScalars| {
        let ctx = LayoutContext { n_inputs: 1, n_outputs: 1, scalars: e };
        match parse_layout(t, &ctx) {
            Ok(OutputLayout::Guarantee(Layout::Bundle(s))) => s,
            other => panic!("not a bundle: {other:?}"),
        }
    };
    let e = env(12);
    let run = |t: &str| check_slot_overlap(&slots(t, &e), &e);
    // a: 4 f32 = bytes [0,16); b starts at 16 (touching: legal) or 15 (overlap).
    assert_eq!(run("bundle(a:f32[4]=dense@0,b:f32[4]=dense@16)"), Ok(()));
    let err = run("bundle(a:f32[4]=dense@0,b:f32[4]=dense@15)").unwrap_err();
    assert_eq!((err.code(), err), ("slot_overlap", BindDecline::SlotOverlap("a".into(), "b".into())));
    // the verdict depends on the environment: the same text, a different n.
    let two = "bundle(a:f32[n]=dense@0,b:f32[4]=dense@32)";
    assert_eq!(check_slot_overlap(&slots(two, &env(8)), &env(8)), Ok(()));
    assert_eq!(check_slot_overlap(&slots(two, &env(9)), &env(9)), Err(BindDecline::SlotOverlap("a".into(), "b".into())));
    // an empty slot has no bytes; a sub-byte dtype rounds outward (7 u4 = 4 bytes).
    assert_eq!(run("bundle(a:f32[0]=dense@8,b:f32[4]=dense@0)"), Ok(()));
    assert_eq!(run("bundle(a:u4[7]=dense@0,b:u8[1]=dense@4)"), Ok(()));
    assert!(run("bundle(a:u4[7]=dense@0,b:u8[1]=dense@3)").is_err());
    // strided and dense_order hulls: strides (4,1) over 2x2 reach element 5; dense_order stays packed.
    assert!(run("bundle(a:f32[2,2]=strided(4,1)@0,b:f32[1]=dense@20)").is_err());
    assert_eq!(run("bundle(a:f32[2,2]=strided(4,1)@0,b:f32[1]=dense@24)"), Ok(()));
    assert_eq!(run("bundle(a:f32[2,2]=dense_order(1,0)@0,b:f32[1]=dense@16)"), Ok(()));
    // a nested layout's hull is its highest reachable element: (2,2):(1,4) reaches 5.
    assert!(run("bundle(a:f32[2,2]=(2,2):(1,4)@0,b:f32[1]=dense@20)").is_err());
    assert_eq!(run("bundle(a:f32[2,2]=(2,2):(1,4)@0,b:f32[1]=dense@24)"), Ok(()));
    // an unbound symbol or an out-of-range offset is a different decline, never an overlap.
    // (the symbol is in the vocabulary at read time, then the bound environment assigns none.)
    let declared = LaunchScalars { ws_bytes: Some(0), ..env(12) };
    let s = slots("bundle(a:f32[4]=dense@0,b:f32[4]=dense@ws_bytes)", &declared);
    assert_eq!(check_slot_overlap(&s, &e).unwrap_err().code(), "unbound_symbol");
}

// Backs[enforced]: KISS-CONTRACT-6.5-0019 — the committed slot_overlap vectors equal the emitter's output and replay through `check_slot_overlap`; the interleaved-but-valid case is the pinned v1 (byte-hull) refusal
#[test]
fn test_contract_slot_overlap_vectors() {
    use kiss_conformance::bind::{check_slot_overlap, emit_slot_overlap_vectors_json, slot_overlap_env, SLOT_OVERLAP_VECTORS};
    use kiss_conformance::layout::{parse_layout, Layout, LayoutContext, OutputLayout};
    let committed = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/slot_overlap_vectors.json")).expect("corpus file");
    assert_eq!(committed.replace("\r\n", "\n"), emit_slot_overlap_vectors_json(), "stale: cargo run --bin emit_slot_overlap_vectors > conformance/corpus/slot_overlap_vectors.json");
    for (name, text, n, expect, _) in SLOT_OVERLAP_VECTORS {
        let e = slot_overlap_env(n);
        let ctx = LayoutContext { n_inputs: 1, n_outputs: 1, scalars: &e };
        let slots = match parse_layout(text, &ctx) {
            Ok(OutputLayout::Guarantee(Layout::Bundle(s))) => s,
            other => panic!("{name}: not a bundle: {other:?}"),
        };
        let got = check_slot_overlap(&slots, &e).err().map_or("ok", |d| d.code());
        assert_eq!(got, expect, "{name}");
    }
    let pinned: Vec<&str> = SLOT_OVERLAP_VECTORS.iter().filter(|v| v.4).map(|v| v.0).collect();
    assert_eq!(pinned, ["interleaved_valid_refused_v1"]);
}

// Backs[enforced]: KISS-CONTRACT-6.12-0007 — a `/` whose divisor names a launch-scalar symbol and is zero declines `division_by_zero` at bind time; a constant zero divisor is a build-time fault; the decline follows the first-failing-step order
#[test]
fn test_contract_bind_division_by_zero() {
    let e = env(5);
    // the divisor comes from the environment: bind-time, whatever its spelling.
    for bad in ["8 / (n - 5)", "n / (n - n)", "8 / extents0[0] / (extents0[1] - 4)"] {
        let r = bind_expr(bad, &e, None);
        assert_eq!(r, Err(BindDecline::DivisionByZero), "{bad}");
        assert_eq!(r.unwrap_err().code(), "division_by_zero");
    }
    assert_eq!(bind_expr("8 / (n - 4)", &e, None), Ok(8), "a non-zero bound divisor binds");
    // constant text: the same fault, a build-time one.
    for bad in ["n / 0", "8 / (3 - 3)"] {
        let r = bind_expr(bad, &e, None);
        assert!(matches!(r, Err(BindDecline::BuildTime(_))), "{bad}: {r:?}");
        assert_eq!(r.unwrap_err().code(), "build_time");
    }
    // first failing step: an unbound symbol in the dividend is reported before the zero divisor.
    assert_eq!(bind_expr("ws_bytes / (n - 5)", &e, None).unwrap_err().code(), "unbound_symbol");
}

// Backs[enforced]: KISS-CONTRACT-6.12-0008 — a `ceil_div` whose divisor names a launch-scalar symbol and is zero or negative declines `ceil_div_divisor` at bind time, carrying the value; a constant non-positive divisor is a build-time fault
#[test]
fn test_contract_bind_ceil_div_divisor() {
    let e = env(5);
    assert_eq!(bind_expr("ceil_div(8, n - 5)", &e, None), Err(BindDecline::CeilDivDivisor(0)));
    assert_eq!(bind_expr("ceil_div(8, n - 7)", &e, None), Err(BindDecline::CeilDivDivisor(-2)));
    assert_eq!(bind_expr("ceil_div(8, n - 5)", &e, None).unwrap_err().code(), "ceil_div_divisor");
    assert_eq!(bind_expr("ceil_div(8, n - 1)", &e, None), Ok(2), "a positive bound divisor binds");
    for bad in ["ceil_div(8, 0)", "ceil_div(8, 2 - 3)"] {
        let r = bind_expr(bad, &e, None);
        assert!(matches!(r, Err(BindDecline::BuildTime(_))), "{bad}: {r:?}");
    }
}
