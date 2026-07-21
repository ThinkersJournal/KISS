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
}
