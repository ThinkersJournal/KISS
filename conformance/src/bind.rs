// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bind-time evaluation of a Dispatch expression (KISS-Contract §6.12).
//!
//! A **bound environment** is a [`LaunchScalars`]; **bind time** is the moment a consumer evaluates
//! an expression against it. [`bind_expr`] is the reference for the three bind-time declines
//! (§6.12-0003..-0005) and for the order in which they are found (§6.12-0006). It evaluates in
//! [`EvalMode::Structural`] (no per-node sign gate), then applies the range gate **last**, so a
//! negative intermediate is legal and only the final value is gated.
//!
//! An expression that is malformed is a **build-time** fault (§6.12-0002), carried as
//! [`BindDecline::BuildTime`]; a reader MUST NOT defer it to bind time. `UnknownSymbol` is mapped to
//! `unbound_symbol` because the evaluator cannot tell "a vocabulary symbol the environment does not
//! assign" from "an identifier outside the vocabulary"; either way no zero or default is substituted.

// `eval` is this crate's own Dispatch-expression parser/evaluator (a bounded arithmetic grammar over
// launch-scalar symbols), not dynamic code execution.
use crate::dispatch_expr::{eval, Decline, EvalMode, LaunchScalars};

/// A typed bind-time decline (§6.12). Never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindDecline {
    /// `unbound_symbol` (§6.12-0003): the expression names a symbol the environment does not assign.
    UnboundSymbol(String),
    /// `extent_out_of_range` (§6.12-0004): the final value is negative or outside a declared range.
    ExtentOutOfRange(i64),
    /// `affine_overflow` (§6.12-0005): a `+`, `-` or `*` left the signed 64-bit range.
    AffineOverflow,
    /// A build-time fault (malformed expression, bad subscript): not a bind-time decline.
    BuildTime(Decline),
}

impl BindDecline {
    /// The typed decline code the spec names, or `build_time` for a build-time fault.
    pub fn code(&self) -> &'static str {
        match self {
            BindDecline::UnboundSymbol(_) => "unbound_symbol",
            BindDecline::ExtentOutOfRange(_) => "extent_out_of_range",
            BindDecline::AffineOverflow => "affine_overflow",
            BindDecline::BuildTime(_) => "build_time",
        }
    }
}

/// Evaluate `expr` against the bound environment `env` (§6.12). `range` is the inclusive range the
/// field declares, if any; the value must also be non-negative (§6.12-0004).
pub fn bind_expr(
    expr: &str,
    env: &LaunchScalars,
    range: Option<(i64, i64)>,
) -> Result<i64, BindDecline> {
    let v = eval(expr, env, EvalMode::Structural).map_err(|d| match d {
        Decline::UnknownSymbol(s) => BindDecline::UnboundSymbol(s),
        Decline::Overflow => BindDecline::AffineOverflow,
        other => BindDecline::BuildTime(other),
    })?;
    let in_range = range.map_or(true, |(lo, hi)| (lo..=hi).contains(&v));
    if v < 0 || !in_range {
        return Err(BindDecline::ExtentOutOfRange(v));
    }
    Ok(v)
}
