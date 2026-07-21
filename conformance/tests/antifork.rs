//! §6.4-0012 anti-fork op-tag test (#29 item 1): the op name is the sole identity anchor;
//! never fabricate a tag — an operation with no KISS-Ops name typed-declines.
//!
//! Oracle: [`kiss_conformance::antifork`].

use kiss_conformance::antifork::{emit_op_tag, no_fabricated_tag, UnknownOp};

const VOCAB: &[&str] = &["add", "mul", "exp", "matmul", "gather", "reduce"];

/// §6.4-0012 — every emitted op tag is a confirmed KISS-Ops name; an unrecognized op
/// typed-declines rather than fabricating a tag.
#[test]
fn test_grammar_no_fabricated_op_tag() {
    // A confirmed KISS-Ops op emits its own name (the identity anchor), unchanged.
    assert_eq!(emit_op_tag("matmul", VOCAB), Ok("matmul"));
    assert_eq!(emit_op_tag("exp", VOCAB), Ok("exp"));

    // An op with NO KISS-Ops name typed-declines — it is NOT re-based onto a near-miss tag
    // (a provider's private `gelu_fast` is never fabricated into `gelu`, nor emitted raw).
    assert_eq!(emit_op_tag("gelu_fast", VOCAB), Err(UnknownOp { candidate: "gelu_fast".into() }));
    assert!(emit_op_tag("my_fused_thing", VOCAB).is_err());

    // The positive invariant: every emitted tag is a member of the confirmed vocabulary —
    // no fabricated tag ever escapes (the anti-silent-fork witness).
    assert!(no_fabricated_tag(&["add", "mul", "matmul"], VOCAB));

    // A recipe that carried a fabricated tag would be caught — such an op MUST instead be a
    // typed decline, never emitted.
    assert!(!no_fabricated_tag(&["add", "gelu_fabricated"], VOCAB));
}
