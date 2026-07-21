//! Reference-decomposition body grammar (KISS-Ops §6.13-0006) and structured-op
//! classification (§6.13-0009).
//!
//!   * `test_ops_decomposition_body_grammar` (§6.13-0006) — the scalar-expression bodies
//!     parse; the structured/prose bodies do NOT (the reason §6.13-0006 was narrowed).
//!   * `test_ops_structured_decomposition_reference` (§6.13-0009) — each structured op's
//!     decomposition references a §6.11 structural op.

use kiss_conformance::decomp_grammar::{
    parse_scalar_body, STRUCTURAL_OPS, STRUCTURED_OPS,
};

/// Representative scalar-expression bodies transcribed from the §6.13 table. Each is the
/// core expression a mechanical resolver parses (the informative "(refinement-permitted…)"
/// notes are annotations, not the body).
const SCALAR_BODIES: &[(&str, &str)] = &[
    ("sqr", "mul(x, x)"),
    ("recip", "div(const(1), x)"),
    (
        "sign",
        "select(cmp_gt(x,const(0)), const(1), select(cmp_lt(x,const(0)), const(-1), const(0)))",
    ),
    ("relu", "select(cmp_lt(x, const(0)), const(0), x)"),
    (
        "gelu",
        "mul(mul(const(0.5), x), add(const(1), erf(div(x, const(sqrt2)))))",
    ),
    ("reduce_mean", "div(reduce(sum, x), reduced_count)"),
    ("reduce_norm2", "sqrt(reduce(sum, sqr(x)))"),
    ("any", "reduce(max, cmp_ne(x, const(0)))"),
    ("cumsum", "prefix_scan(monoid=sum, inclusive)"),
    (
        "softmax",
        "m=reduce(max,x); e=exp(sub(x,m)); s=reduce(sum,e); out=div(e,s)",
    ),
    (
        "logsumexp",
        "m=reduce(max,x); out=add(m, log(reduce(sum, exp(sub(x,m)))))",
    ),
    (
        "rms_norm",
        "ms=reduce_mean(sqr(input(0))); out=mul(mul(input(0), rsqrt(add(ms, param(0)))), input(1))",
    ),
];

/// Structured/prose bodies transcribed from the §6.13 table. These describe an iteration
/// space / index mapping and are the ~9 rows §6.13-0006 could not parse as written — the
/// #41 defect. A mechanical scalar-grammar resolver MUST reject them (they are governed by
/// §6.13-0009 + the §6.11 oracle instead).
const STRUCTURED_PROSE_BODIES: &[(&str, &str)] = &[
    (
        "matmul",
        "reduce(sum, axis=K) of element_map(mul(input(0), input(1)))",
    ),
    (
        "argmax",
        "original-index at rank 0 of sort_network(desc, keys=x)",
    ),
    (
        "avg_pool",
        "reduce_mean over the window axis of the pooled view",
    ),
    (
        "im2col",
        "closed-form structured gather mapping each output element to a source index",
    ),
];

/// KISS-OPS-6.13-0006: a scalar-expression body parses as the tree/SSA grammar; a
/// structured op's prose body does not (and MUST NOT be required to).
#[test]
fn test_ops_decomposition_body_grammar() {
    for (op, body) in SCALAR_BODIES {
        assert!(
            parse_scalar_body(body).is_ok(),
            "scalar body of `{op}` MUST parse as the §6.13-0006 grammar: {body:?} -> {:?}",
            parse_scalar_body(body)
        );
    }
    // Teeth: the structured/prose rows are exactly what made the un-narrowed §6.13-0006
    // untestable. The scalar grammar rejects them; §6.13-0009 governs them instead.
    for (op, body) in STRUCTURED_PROSE_BODIES {
        assert!(
            parse_scalar_body(body).is_err(),
            "structured body of `{op}` is prose over an iteration space and MUST NOT be \
             required to parse as the §6.13-0006 scalar grammar: {body:?}"
        );
    }

    // §6.13-0006 SSA discipline: a name is bound exactly once and MUST NOT reference
    // itself or a not-yet-bound name.
    assert!(
        parse_scalar_body("m = mul(x, x); out = add(m, m)").is_ok(),
        "a well-formed SSA body (each name bound before use) MUST parse"
    );
    assert!(
        parse_scalar_body("a = a").is_err(),
        "a self-reference `a = a` MUST be rejected (§6.13-0006)"
    );
    assert!(
        parse_scalar_body("x = y; y = mul(x, x)").is_err(),
        "a forward reference (`x` uses `y` before `y` is bound) MUST be rejected (§6.13-0006)"
    );
    assert!(
        parse_scalar_body("m = mul(x, x); m = add(x, x)").is_err(),
        "binding a name twice MUST be rejected (SSA, §6.13-0006)"
    );
}

/// KISS-OPS-6.13-0009: every structured op's decomposition references a §6.11 structural
/// op (or the `matmul` contraction), and the named set is exactly the nine structured ops.
#[test]
fn test_ops_structured_decomposition_reference() {
    for (op, structural) in STRUCTURED_OPS {
        assert!(
            STRUCTURAL_OPS.contains(structural),
            "structured op `{op}` MUST reference a §6.11 structural op; `{structural}` is \
             not in {STRUCTURAL_OPS:?}"
        );
    }
    // Guard against drift: the §6.13-0009 structured-op set is exactly these nine.
    let ops: Vec<&str> = STRUCTURED_OPS.iter().map(|(op, _)| *op).collect();
    let expected = [
        "matmul", "argmax", "argmin", "avg_pool", "max_pool", "index_select", "embedding",
        "scatter_add", "im2col",
    ];
    assert_eq!(ops, expected, "the §6.13-0009 structured-op set drifted");
}
