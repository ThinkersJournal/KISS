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
