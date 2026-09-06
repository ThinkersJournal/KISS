//! KISS-Conform tests for the Contract §6.6-0006 Dispatch expression grammar.
//! Backs KISS-CONTRACT-6.6-0006 (the pinned machine-evaluable expression grammar,
//! incl. the element-subscript operator resolving issue #43).

use kiss_conformance::dispatch_expr::*;

fn scalars() -> LaunchScalars {
    // rank-2 kernel: extents0 = [6, 4], strides0 = [4, 1], off0 = 0, n = 24
    LaunchScalars {
        rank: 2,
        n: 24,
        extents: vec![vec![6, 4]],
        strides: vec![vec![4, 1]],
        idx_extents: vec![],
        off: vec![0],
        ws_bytes: 0,
        param: vec![],
    }
}

#[test]
fn test_contract_dispatch_expressions_machine_evaluable() {
    let s = scalars();
    // scalar arithmetic + ceil_div (the pre-existing grammar) still evaluates.
    assert_eq!(eval("ceil_div(n, 256)", &s, EvalMode::Domain).unwrap(), 1);
    // NEW: element subscript pulls one axis out of a rank-length array symbol.
    assert_eq!(eval("extents0[0]", &s, EvalMode::Domain).unwrap(), 6);
    assert_eq!(eval("extents0[0] * extents0[1]", &s, EvalMode::Domain).unwrap(), 24);
    // subscript binds tighter than '*'.
    assert_eq!(eval("2 * strides0[0]", &s, EvalMode::Domain).unwrap(), 8);
    // precedence: '*' binds tighter than '+'  ->  2 + 3*4 == 14, not 20.
    assert_eq!(eval("2 + 3 * 4", &s, EvalMode::Domain).unwrap(), 14);
    // structural addressing may reference strides element-wise and go negative.
    let mut neg = scalars();
    neg.strides[0][0] = -4;
    assert_eq!(eval("off0 + strides0[0]", &neg, EvalMode::Structural).unwrap(), -4);
}

#[test]
fn dispatch_subscript_declines() {
    let s = scalars();
    assert!(matches!(eval("n[0]", &s, EvalMode::Domain), Err(Decline::SubscriptOfScalar(_))));
    assert!(matches!(eval("strides0", &s, EvalMode::Domain), Err(Decline::BareArraySymbol(_))));
    assert!(matches!(eval("extents0[2]", &s, EvalMode::Domain),
                     Err(Decline::SubscriptOutOfBounds { k: 2, rank: 2, .. })));
    assert!(matches!(eval("wobble", &s, EvalMode::Domain), Err(Decline::UnknownSymbol(_))));
    // a negative result IS a decline in Domain mode, but allowed in Structural mode.
    let mut neg = scalars();
    neg.strides[0][0] = -4;
    assert!(matches!(eval("strides0[0]", &neg, EvalMode::Domain), Err(Decline::NegativeInDomain(-4))));
    // integer overflow (+ - * and the a+b-1 inside ceil_div) MUST decline, never panic.
    assert!(matches!(
        eval("9223372036854775807 + 1", &s, EvalMode::Domain),
        Err(Decline::ParseError(_))
    ));
    assert!(matches!(
        eval("9223372036854775807 * 2", &s, EvalMode::Domain),
        Err(Decline::ParseError(_))
    ));
    assert!(matches!(
        eval("ceil_div(9223372036854775807, 2)", &s, EvalMode::Domain),
        Err(Decline::ParseError(_))
    ));
    // i64::MIN / -1 overflow MUST decline, never panic.
    assert!(matches!(
        eval("(0 - 9223372036854775807 - 1) / (0 - 1)", &s, EvalMode::Structural),
        Err(Decline::ParseError(_))
    ));
    // an oversized subscript literal (> u32::MAX) MUST NOT be silently
    // truncated by narrowing into a valid axis — it must decline out-of-bounds.
    assert!(matches!(
        eval("extents0[4294967296]", &s, EvalMode::Domain),
        Err(Decline::SubscriptOutOfBounds { .. })
    ));
}

/// Backs KISS-CONTRACT-6.6-0009 — the DIVISION SEMANTICS of the §6.6-0006 Dispatch expression
/// grammar. ⚠️ The CLAUSE is -0009; -0006 defines the grammar these operators live in and is
/// backed separately. Citing -0006 here would have credited a clause that was already backed
/// while leaving the one this test was written for resting on the §9 matrix row alone.
/// `/` truncates toward zero, `ceil_div` is the true ceiling over a positive divisor, and a zero
/// divisor and `i64::MIN / -1` are typed declines.
///
/// ⚠️ BEFORE THIS TEST, NOTHING IN THE CORPUS DISCRIMINATED THE TWO CANDIDATE SEMANTICS.
/// Measured across the whole crate — 113 files, 6106 string literals — exactly four literals are
/// Dispatch expressions containing a division: `ceil_div(n, 256)` twice,
/// `ceil_div(9223372036854775807, 2)`, and `(0 - 9223372036854775807 - 1) / (0 - 1)`. The first
/// three have non-negative dividends, and the fourth is `i64::MIN / -1`, which OVERFLOWS under
/// BOTH semantics and is declined either way. **So a foreign implementor could have implemented
/// Euclidean `/` and passed every existing test.** The clause without these vectors would have
/// been unfalsifiable prose.
///
/// ⚠️ AND THE DISCRIMINATING OPERAND IS THE DIVIDEND, NOT THE DIVISOR. Measured over 300 (a, b)
/// pairs: 86 diverge, **all of them with a negative dividend, none with a dividend >= 0**,
/// whatever the divisor's sign. A fixture built on a negative DIVISOR — `7 / (0 - 2)` — does not
/// discriminate, and would have occupied the slot while proving nothing.
#[test]
fn test_contract_division_semantics_are_pinned() {
    let s = scalars();
    let ev = |e: &str| eval(e, &s, EvalMode::Structural);

    // ---- `/` TRUNCATES TOWARD ZERO. Each of these differs under Euclidean division. ----------
    // -7 / 2 : truncating -3, Euclidean -4.
    assert_eq!(ev("(0 - 7) / 2").unwrap(), -3, "`/` must truncate toward zero, not floor");
    // -7 / -2 : truncating 3, Euclidean 4. Both operands negative — the divisor's sign changes
    // the DIRECTION of the difference, which is why one case of each sign is carried.
    assert_eq!(ev("(0 - 7) / (0 - 2)").unwrap(), 3, "`/` truncates for a negative divisor too");

    // ⚠️ CONTROL, NOT COVERAGE. An EXACT division agrees under both semantics, so it proves the
    // evaluator runs and proves NOTHING about which rounding is implemented. It is here so that a
    // later reader does not mistake it for a discriminating case and delete one that is.
    assert_eq!(ev("(0 - 8) / 2").unwrap(), -4, "control: exact division, both semantics agree");

    // ---- `ceil_div` IS THE TRUE CEILING, and its internals are NOT the pinned `/` ------------
    // ceil(-10/4) = -2. An implementation spelling ceil_div as the truncating `(a + b - 1) / b`
    // yields -1 here. This is the case that makes `div_euclid` load-bearing rather than stylistic.
    assert_eq!(ev("ceil_div(0 - 10, 4)").unwrap(), -2, "ceil_div must be the true ceiling");
    assert_eq!(ev("ceil_div(0 - 9, 3)").unwrap(), -3, "exact ceiling, negative dividend");
    assert_eq!(ev("ceil_div(7, 2)").unwrap(), 4, "ordinary ceiling, positive dividend");

    // ---- the two DECLINES, and they are DISTINCT declines ------------------------------------
    assert!(ev("7 / 0").is_err(), "a zero divisor MUST decline, never a panic or a value");
    assert!(
        ev("ceil_div(7, 0)").is_err(),
        "ceil_div's divisor precondition is POSITIVE — zero declines"
    );
    assert!(
        ev("ceil_div(7, 0 - 2)").is_err(),
        "ceil_div over a NEGATIVE divisor is outside its precondition and declines"
    );
    assert!(
        ev("(0 - 9223372036854775807 - 1) / (0 - 1)").is_err(),
        "i64::MIN / -1 overflows under BOTH semantics and MUST decline"
    );
}
