// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference reader for the Interface `output_layouts` field (KISS-Contract §6.5-0014 to
//! §6.5-0020, board 378): the closed sum of seven output-layout forms.
//!
//! The canonical text carries **no spaces** (one way to say one thing). A reader declines a form
//! with a space, a leading `+`, a leading zero or any other non-canonical integer.
//!
//! ```text
//! output_layouts := "[" element { ", " element } "]"          (one element per output operand)
//! element        := "-" | layout                              ("-" = the layout the caller supplied)
//! layout         := dense | dense_order | same_as | strided | hier | bundle | none
//! hier           := tuple ":" tuple | "swz(" B "," M "," S ")(" tuple ":" tuple ")"
//! tuple          := integer | "(" tuple { "," tuple } ")"
//! ```
//!
//! A **known** form that is malformed is a typed [`LayoutDecline`] (the field is refused). A form
//! whose head is **not** one of the seven is [`OutputLayout::Refused`]: the guarantee is refused
//! (the consumer treats the output as `none` and records it), never the whole contract (§6.5-0015).

use crate::dispatch_expr::{eval, EvalMode, LaunchScalars};

/// Maximum nesting depth of a `hier` tuple (Unpopped `layout_algebra::MAX_DEPTH`).
pub const MAX_DEPTH: usize = 16;

/// A nested integer tuple: an integer leaf or a non-empty tuple.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntTuple {
    /// An integer leaf.
    Leaf(u64),
    /// A non-empty parenthesised tuple.
    Tup(Vec<IntTuple>),
}

/// A nested layout `shape:stride`, optionally composed with a swizzle `(B, M, S)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hier {
    /// CuTe `Swizzle<B,M,S>`: bits, base, shift (shift signed).
    pub swizzle: Option<(u32, u32, i32)>,
    /// Extents; every leaf is `>= 1`.
    pub shape: IntTuple,
    /// Strides, congruent to `shape`; every leaf is `>= 0` and an integer (§6.5-0017).
    pub stride: IntTuple,
}

/// One slot of a `bundle`: a name, a layout of forms 1 to 5, a byte-offset expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    /// Slot name, unique within the bundle.
    pub name: String,
    /// Never itself a `bundle` or `none`.
    pub layout: Layout,
    /// A §6.6-0006 expression over the launch scalars, spelled without spaces.
    pub offset: String,
}

/// The seven output-layout forms (§6.5-0015).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Densely packed, canonical row-major order.
    Dense,
    /// Densely packed in an explicit axis order, slowest axis first; a permutation of `0..rank`.
    DenseOrder(Vec<u32>),
    /// The strides of input operand `i`.
    SameAs(u32),
    /// One stride expression per axis (§6.6-0006 grammar, no spaces).
    Strided(Vec<String>),
    /// A CuTe-style nested layout, optionally swizzled.
    Hier(Hier),
    /// One buffer holding several named slots.
    Bundle(Vec<Slot>),
    /// No guarantee: read the layout from the buffer.
    None,
}

/// One element of `output_layouts`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputLayout {
    /// `-`: the layout the caller supplied (the meaning of an absent field).
    Supplied,
    /// A parsed guarantee.
    Guarantee(Layout),
    /// An unknown form: the guarantee is refused and treated as `none`; the spelling is recorded.
    Refused(String),
}

/// A typed decline of the `output_layouts` field. Never a panic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutDecline {
    /// The value is not `[...]`.
    NotAnArray,
    /// Element count differs from the number of output operands (§6.5-0014).
    CountMismatch { want: usize, got: usize },
    /// A space, a leading `+`, a leading zero, or another non-canonical spelling.
    NonCanonical(String),
    /// A known form that does not parse.
    Malformed(String),
    /// Shape and stride have different nesting profiles.
    NotCongruent,
    /// A shape leaf `0`.
    ZeroExtent,
    /// Nesting deeper than [`MAX_DEPTH`].
    TooDeep,
    /// `dense_order` is not a permutation of `0..rank`, or `strided` has not `rank` strides.
    BadAxisList,
    /// `same_as(i)` with `i` not an input operand.
    SameAsOutOfRange(u32),
    /// A bundle slot name repeated.
    DuplicateSlot(String),
    /// A `bundle` or `none` inside a bundle slot.
    NestedBundle,
    /// An expression the §6.6-0006 grammar or this contract's vocabulary rejects.
    BadExpression(String),
}

/// What the reader needs from the contract: operand counts and the launch-scalar vocabulary.
pub struct LayoutContext<'a> {
    /// Number of input operands (`same_as(i)` needs `i < n_inputs`).
    pub n_inputs: u32,
    /// Number of output operands (§6.5-0014: the element count).
    pub n_outputs: usize,
    /// A probe set of launch scalars carrying the contract's `rank` and vocabulary.
    pub scalars: &'a LaunchScalars,
}

const HEADS: [&str; 7] = ["dense", "dense_order", "same_as", "strided", "swz", "bundle", "none"];

fn canonical_uint(s: &str) -> Result<u64, LayoutDecline> {
    let ok = !s.is_empty()
        && s.bytes().all(|b| b.is_ascii_digit())
        && (s == "0" || !s.starts_with('0'));
    if !ok {
        return Err(LayoutDecline::NonCanonical(s.to_string()));
    }
    s.parse::<u64>().map_err(|_| LayoutDecline::NonCanonical(s.to_string()))
}

fn canonical_int(s: &str) -> Result<i32, LayoutDecline> {
    let (neg, digits) = s.strip_prefix('-').map_or((false, s), |d| (true, d));
    let v = canonical_uint(digits)?;
    if neg && v == 0 {
        return Err(LayoutDecline::NonCanonical(s.to_string()));
    }
    let v = i32::try_from(v).map_err(|_| LayoutDecline::NonCanonical(s.to_string()))?;
    Ok(if neg { -v } else { v })
}

/// Split `s` on top-level commas (not inside parentheses).
fn split_top(s: &str) -> Result<Vec<&str>, LayoutDecline> {
    let (mut depth, mut start, mut out) = (0i32, 0usize, Vec::new());
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        if depth < 0 {
            return Err(LayoutDecline::Malformed(s.to_string()));
        }
    }
    if depth != 0 {
        return Err(LayoutDecline::Malformed(s.to_string()));
    }
    out.push(&s[start..]);
    Ok(out)
}

fn parse_tuple(s: &str) -> Result<IntTuple, LayoutDecline> {
    match s.strip_prefix('(') {
        None => Ok(IntTuple::Leaf(canonical_uint(s)?)),
        Some(rest) => {
            let inner = rest.strip_suffix(')').ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
            let items = split_top(inner)?;
            items.into_iter().map(parse_tuple).collect::<Result<Vec<_>, _>>().map(IntTuple::Tup)
        }
    }
}

fn depth(t: &IntTuple) -> usize {
    match t {
        IntTuple::Leaf(_) => 0,
        IntTuple::Tup(v) => 1 + v.iter().map(depth).max().unwrap_or(0),
    }
}

fn congruent(a: &IntTuple, b: &IntTuple) -> bool {
    match (a, b) {
        (IntTuple::Leaf(_), IntTuple::Leaf(_)) => true,
        (IntTuple::Tup(x), IntTuple::Tup(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| congruent(p, q))
        }
        _ => false,
    }
}

fn any_leaf(t: &IntTuple, f: &dyn Fn(u64) -> bool) -> bool {
    match t {
        IntTuple::Leaf(v) => f(*v),
        IntTuple::Tup(v) => v.iter().any(|c| any_leaf(c, f)),
    }
}

/// Split `shape:stride` at the single top-level colon.
fn split_colon(s: &str) -> Result<(&str, &str), LayoutDecline> {
    let mut depth = 0i32;
    let mut at = None;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ':' if depth == 0 && at.is_none() => at = Some(i),
            ':' if depth == 0 => return Err(LayoutDecline::Malformed(s.into())),
            _ => {}
        }
    }
    let i = at.ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
    Ok((&s[..i], &s[i + 1..]))
}

fn parse_shape_stride(s: &str) -> Result<(IntTuple, IntTuple), LayoutDecline> {
    let (a, b) = split_colon(s)?;
    let (shape, stride) = (parse_tuple(a)?, parse_tuple(b)?);
    if depth(&shape) > MAX_DEPTH || depth(&stride) > MAX_DEPTH {
        return Err(LayoutDecline::TooDeep);
    }
    if !congruent(&shape, &stride) {
        return Err(LayoutDecline::NotCongruent);
    }
    if any_leaf(&shape, &|v| v == 0) {
        return Err(LayoutDecline::ZeroExtent);
    }
    Ok((shape, stride))
}

fn parse_swizzled(s: &str) -> Result<Hier, LayoutDecline> {
    let rest = s.strip_prefix("swz(").ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
    let close = rest.find(")(").ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
    let args: Vec<&str> = rest[..close].split(',').collect();
    let body = rest[close + 2..].strip_suffix(')').ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
    if args.len() != 3 {
        return Err(LayoutDecline::Malformed(s.into()));
    }
    let (b, m) = (canonical_uint(args[0])?, canonical_uint(args[1])?);
    let (b, m) = (
        u32::try_from(b).map_err(|_| LayoutDecline::NonCanonical(args[0].into()))?,
        u32::try_from(m).map_err(|_| LayoutDecline::NonCanonical(args[1].into()))?,
    );
    let (shape, stride) = parse_shape_stride(body)?;
    Ok(Hier { swizzle: Some((b, m, canonical_int(args[2])?)), shape, stride })
}

// `eval` is this crate's own §6.6-0006 recursive-descent arithmetic evaluator (dispatch_expr),
// not dynamic code execution.
fn check_expr(e: &str, ctx: &LayoutContext) -> Result<(), LayoutDecline> {
    eval(e, ctx.scalars, EvalMode::Structural)
        .map(|_| ())
        .map_err(|d| LayoutDecline::BadExpression(format!("{e}: {d:?}")))
}

fn parse_call<'a>(s: &'a str, head: &str) -> Option<&'a str> {
    s.strip_prefix(head)?.strip_prefix('(')?.strip_suffix(')')
}

fn parse_dense_order(args: &str, ctx: &LayoutContext) -> Result<Layout, LayoutDecline> {
    let p: Vec<u32> = split_top(args)?
        .into_iter()
        .map(|x| canonical_uint(x).and_then(|v| u32::try_from(v).map_err(|_| LayoutDecline::BadAxisList)))
        .collect::<Result<_, _>>()?;
    let mut sorted = p.clone();
    sorted.sort_unstable();
    if sorted != (0..ctx.scalars.rank).collect::<Vec<_>>() {
        return Err(LayoutDecline::BadAxisList);
    }
    Ok(Layout::DenseOrder(p))
}

fn parse_strided(args: &str, ctx: &LayoutContext) -> Result<Layout, LayoutDecline> {
    let es = split_top(args)?;
    if es.len() != ctx.scalars.rank as usize {
        return Err(LayoutDecline::BadAxisList);
    }
    for e in &es {
        check_expr(e, ctx)?;
    }
    Ok(Layout::Strided(es.into_iter().map(String::from).collect()))
}

fn parse_slot(s: &str, ctx: &LayoutContext) -> Result<Slot, LayoutDecline> {
    let (name, rest) = s.split_once('=').ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
    let (lay, off) = rest.rsplit_once('@').ok_or_else(|| LayoutDecline::Malformed(s.into()))?;
    let ok = !name.is_empty()
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !ok {
        return Err(LayoutDecline::NonCanonical(name.into()));
    }
    check_expr(off, ctx)?;
    match parse_known(lay, ctx)? {
        Layout::Bundle(_) | Layout::None => Err(LayoutDecline::NestedBundle),
        layout => Ok(Slot { name: name.into(), layout, offset: off.into() }),
    }
}

fn parse_bundle(args: &str, ctx: &LayoutContext) -> Result<Layout, LayoutDecline> {
    let mut slots: Vec<Slot> = Vec::new();
    for part in split_top(args)? {
        let slot = parse_slot(part, ctx)?;
        if slots.iter().any(|x| x.name == slot.name) {
            return Err(LayoutDecline::DuplicateSlot(slot.name));
        }
        slots.push(slot);
    }
    Ok(Layout::Bundle(slots))
}

fn parse_known(s: &str, ctx: &LayoutContext) -> Result<Layout, LayoutDecline> {
    if s == "dense" {
        return Ok(Layout::Dense);
    }
    if s == "none" {
        return Ok(Layout::None);
    }
    if let Some(a) = parse_call(s, "dense_order") {
        return parse_dense_order(a, ctx);
    }
    if let Some(a) = parse_call(s, "same_as") {
        let i = u32::try_from(canonical_uint(a)?).map_err(|_| LayoutDecline::SameAsOutOfRange(u32::MAX))?;
        return if i < ctx.n_inputs { Ok(Layout::SameAs(i)) } else { Err(LayoutDecline::SameAsOutOfRange(i)) };
    }
    if let Some(a) = parse_call(s, "strided") {
        return parse_strided(a, ctx);
    }
    if let Some(a) = parse_call(s, "bundle") {
        return parse_bundle(a, ctx);
    }
    if s.starts_with("swz(") {
        return parse_swizzled(s).map(Layout::Hier);
    }
    let (shape, stride) = parse_shape_stride(s)?;
    Ok(Layout::Hier(Hier { swizzle: None, shape, stride }))
}

fn head_of(s: &str) -> &str {
    s.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).next().unwrap_or("")
}

/// Parse one `layout` spelling (a single array element other than `-`).
pub fn parse_layout(s: &str, ctx: &LayoutContext) -> Result<OutputLayout, LayoutDecline> {
    if s.contains(' ') {
        return Err(LayoutDecline::NonCanonical(s.into()));
    }
    let head = head_of(s);
    let starts_alpha = head.bytes().next().is_some_and(|b| b.is_ascii_alphabetic());
    if s == "-" {
        return Ok(OutputLayout::Supplied);
    }
    if starts_alpha && !HEADS.contains(&head) {
        return Ok(OutputLayout::Refused(s.to_string()));
    }
    parse_known(s, ctx).map(OutputLayout::Guarantee)
}

/// Parse the Interface `output_layouts` value (§6.5-0014): `[e0, e1, …]`, one per output operand.
pub fn parse_output_layouts(value: &str, ctx: &LayoutContext) -> Result<Vec<OutputLayout>, LayoutDecline> {
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(LayoutDecline::NotAnArray)?;
    let parts: Vec<&str> = if inner.is_empty() { Vec::new() } else { inner.split(", ").collect() };
    if parts.len() != ctx.n_outputs {
        return Err(LayoutDecline::CountMismatch { want: ctx.n_outputs, got: parts.len() });
    }
    parts.into_iter().map(|p| parse_layout(p, ctx)).collect()
}
