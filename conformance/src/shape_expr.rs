//! Reference evaluator + canonical serializer for the KISS-Ops **shape-expression
//! vocabulary** and the shape-side oracle (Ops §6.20).
//!
//! A shape expression describes an op's output shape as a function of its
//! operands' concrete shapes (plus OpAttrs and declared params) — the shape-side
//! companion to the KISS-Contract §6.4-0006 value oracle. The closed *core*
//! vocabulary is two constructors:
//!
//!   * `ShapeExpr := SameAs(operand)`            — the operand's whole shape
//!   * `DimExpr   := Extent(operand, axis) | Const | Param | DimExpr BinOp DimExpr`
//!
//! Two further whole-shape constructors — `WithDim(operand, axis, DimExpr)` and
//! `Dims([DimExpr, …])` — are **experimental-range** extensions of this vocabulary,
//! entered through the umbrella §6.4 extension registry (issue #80, owner Fuel;
//! Ops §6.20-0009/-0010). Their functional spellings are `with_dim(...)` and
//! `dims(...)`. They are NOT core: a core encoder never emits them, and until a
//! reader opts into the registered extension it declines their tags typed.
//!
//! Operand references are **positional** (KISS-Classify canonical operand order),
//! because an op_dag interior node carries no operand-role tuple (Contract
//! §6.4-0009). `axis` is a **non-negative** operand-axis index or the reserved
//! `last` sentinel ([`LAST`]) — the KISS §6.19 axis convention is non-negative
//! (§6.19-0007 `u8`; §6.13-0008 `norm_axis`), so a signed axis never appears on the
//! wire; `last` is resolved against the operand's rank at evaluation time. `÷` is
//! floor division.
//!
//! `Reduce` (`0x09`) remains **reserved** (tag allocated, never emitted by a core
//! encoder, rejected by a reader) pending an extension-registry entry (umbrella
//! §6.4) — it has no consumer today. `WithDim` (`0x0A`) / `Dims` (`0x0B`) are the
//! now-**activated** experimental extension (§6.20-0009/-0010).
//!
//! Serialization follows the §6.19 canonical discipline: a one-byte tag (`0`
//! reserved, §6.19-0006), fixed-width little-endian fields (§6.19-0007), and
//! definite `u16`-LE length-prefixed children (§6.19-0010), so a shape-bearing
//! blob is byte-deterministic and hashable.

// ---- §6.20-0005 tag space (one byte; 0 reserved per §6.19-0006) ---------------

pub const TAG_SAME_AS: u8 = 0x01; // ShapeExpr
pub const TAG_EXTENT: u8 = 0x02; // DimExpr
pub const TAG_CONST: u8 = 0x03;
pub const TAG_PARAM: u8 = 0x04;
pub const TAG_ADD: u8 = 0x05;
pub const TAG_SUB: u8 = 0x06;
pub const TAG_MUL: u8 = 0x07;
pub const TAG_DIV: u8 = 0x08;
pub const TAG_REDUCE: u8 = 0x09; // reserved (no consumer; extension-registry, umbrella §6.4)
pub const TAG_WITH_DIM: u8 = 0x0A; // ShapeExpr, experimental extension (§6.20-0009, umbrella §6.4)
pub const TAG_DIMS: u8 = 0x0B; // ShapeExpr, experimental extension (§6.20-0010, umbrella §6.4)

/// The sentinel extent for a symbolic / data-dependent axis length. An expression
/// over it resolves to a surfaced [`DimValue::Gap`] / [`ShapeValue::Gap`] — never a
/// decline, never a panic (§6.20-0004).
pub const SYMBOLIC: i64 = i64::MIN;

/// The reserved `axis` sentinel denoting the **last (trailing)** axis, resolved to
/// `rank − 1` at evaluation time (§6.20-0002/-0003). Concrete axes are
/// `0..MAX_RANK-1` (MAX_RANK = 8), so `0xFF` is unambiguously the sentinel. It is a
/// **distinct** single-axis (`u8`) sentinel, chosen high in the spirit of the
/// §6.19-0020 `reduce_axes` trailing-axis sentinel but **not** byte-identical to it
/// — that `0xFFFE` is a `u16` axis-*set* mask, a different field.
pub const LAST: u8 = 0xFF;

// ---- AST ---------------------------------------------------------------------

/// A single-dimension expression (`DimExpr`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dim {
    /// The size of `operand`'s `axis` (non-negative index, or [`LAST`] = trailing).
    Extent { operand: u8, axis: u8 },
    /// A pinned integer constant.
    Const(i64),
    /// The value of the op's `field`-th declared param.
    Param(u8),
    Add(Box<Dim>, Box<Dim>),
    Sub(Box<Dim>, Box<Dim>),
    Mul(Box<Dim>, Box<Dim>),
    /// Floor division (rounds toward −∞).
    Div(Box<Dim>, Box<Dim>),
}

/// A whole-shape expression (`ShapeExpr`). The closed core is `SameAs`; `WithDim`
/// and `Dims` are the experimental-range extension constructors registered via
/// umbrella §6.4 (Ops §6.20-0009/-0010), activating the §6.20-0005 tags 0x0A/0x0B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeExpr {
    /// The operand's whole shape.
    SameAs { operand: u8 },
    /// EXPERIMENTAL (umbrella §6.4; tag `0x0A`). The `operand`'s shape with the
    /// resolved `axis` (non-negative, or [`LAST`] = trailing, §6.20-0003) replaced
    /// by `dim`. Functional spelling `with_dim(operand, axis, dim)` (§6.20-0009).
    WithDim { operand: u8, axis: u8, dim: Box<Dim> },
    /// EXPERIMENTAL (umbrella §6.4; tag `0x0B`). A whole shape built from `N >= 0`
    /// ordered `DimExpr`s (`N = 0` = the rank-0 scalar shape). Functional spelling
    /// `dims([dim, …])` (§6.20-0010).
    Dims(Vec<Dim>),
}

// ---- evaluation outcomes -----------------------------------------------------

/// The result of evaluating a `DimExpr`: a concrete dimension, or a surfaced gap
/// (a symbolic/data-dependent extent was reached, §6.20-0004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DimValue {
    Concrete(i64),
    Gap,
}

/// The result of evaluating a `ShapeExpr`: a concrete shape, or a surfaced gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeValue {
    Concrete(Vec<i64>),
    Gap,
}

/// A typed decline. A reader/evaluator MUST refuse a malformed input with one of
/// these, never a panic (KISS-Conform §6.7; §6.20-0003/0006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeExprError {
    /// The reserved `0` tag was read (§6.19-0006).
    ZeroTag,
    /// A tag not valid at the grammar level being decoded: for a `DimExpr` reader,
    /// any non-`DimExpr` tag (the still-reserved `Reduce` 0x09, or the whole-shape
    /// `SameAs`/`WithDim`/`Dims` tags); for a `ShapeExpr` reader, the still-reserved
    /// `Reduce` 0x09 or any unregistered/`DimExpr`-only tag (§6.20-0006/-0009/-0010).
    ReservedTag { tag: u8 },
    /// The blob was shorter than the tag's schema requires.
    TruncatedBlob { need: usize, got: usize },
    /// Bytes remained after a complete expression was decoded.
    TrailingBytes { extra: usize },
    /// A concrete axis was `>= rank` (or `last` on a rank-0 operand), §6.20-0003.
    AxisOutOfRange { axis: u8, rank: usize },
    /// An operand index named no operand.
    OperandOutOfRange { operand: u8, operands: usize },
    /// A param index named no declared param.
    ParamOutOfRange { field: u8, params: usize },
    /// A `÷` by zero (§6.20-0003).
    DivideByZero,
}

// ---- serialization (§6.20-0005) ----------------------------------------------

impl ShapeExpr {
    /// The canonical wire bytes for this shape expression (§6.20-0005; the
    /// experimental extension tags 0x0A/0x0B per §6.20-0009/-0010).
    pub fn encode(&self) -> Vec<u8> {
        match self {
            ShapeExpr::SameAs { operand } => vec![TAG_SAME_AS, *operand],
            // 0x0A, u8 operand, u8 axis (0xFF = last), one u16-LE length-prefixed
            // child DimExpr blob (§6.20-0009).
            ShapeExpr::WithDim { operand, axis, dim } => {
                let child = dim.encode();
                let mut v = vec![TAG_WITH_DIM, *operand, *axis];
                v.extend_from_slice(&(child.len() as u16).to_le_bytes());
                v.extend_from_slice(&child);
                v
            }
            // 0x0B, u8 count, then count × (u16-LE length-prefixed child DimExpr
            // blob) (§6.20-0010).
            ShapeExpr::Dims(dims) => {
                let mut v = vec![TAG_DIMS, dims.len() as u8];
                for d in dims {
                    let child = d.encode();
                    v.extend_from_slice(&(child.len() as u16).to_le_bytes());
                    v.extend_from_slice(&child);
                }
                v
            }
        }
    }
}

impl Dim {
    /// The canonical wire bytes for this dim expression (§6.20-0005).
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Dim::Extent { operand, axis } => vec![TAG_EXTENT, *operand, *axis],
            Dim::Const(c) => {
                let mut v = vec![TAG_CONST];
                v.extend_from_slice(&c.to_le_bytes());
                v
            }
            Dim::Param(f) => vec![TAG_PARAM, *f],
            Dim::Add(a, b) => encode_binary(TAG_ADD, a, b),
            Dim::Sub(a, b) => encode_binary(TAG_SUB, a, b),
            Dim::Mul(a, b) => encode_binary(TAG_MUL, a, b),
            Dim::Div(a, b) => encode_binary(TAG_DIV, a, b),
        }
    }
}

fn encode_binary(tag: u8, a: &Dim, b: &Dim) -> Vec<u8> {
    let (ca, cb) = (a.encode(), b.encode());
    let mut v = vec![tag];
    v.extend_from_slice(&(ca.len() as u16).to_le_bytes());
    v.extend_from_slice(&ca);
    v.extend_from_slice(&(cb.len() as u16).to_le_bytes());
    v.extend_from_slice(&cb);
    v
}

// ---- deserialization (§6.20-0006: typed decline, never a panic) --------------

/// Decode one `DimExpr` blob, rejecting a malformed one with a typed decline. A
/// well-formed blob round-trips: `decode_dim(&d.encode()) == Ok(d)`.
pub fn decode_dim(blob: &[u8]) -> Result<Dim, ShapeExprError> {
    let (d, consumed) = decode_dim_at(blob, 0)?;
    if consumed != blob.len() {
        return Err(ShapeExprError::TrailingBytes { extra: blob.len() - consumed });
    }
    Ok(d)
}

fn decode_dim_at(blob: &[u8], pos: usize) -> Result<(Dim, usize), ShapeExprError> {
    let tag = *blob
        .get(pos)
        .ok_or(ShapeExprError::TruncatedBlob { need: pos + 1, got: blob.len() })?;
    match tag {
        0x00 => Err(ShapeExprError::ZeroTag),
        TAG_EXTENT => {
            need(blob, pos, 3)?;
            Ok((Dim::Extent { operand: blob[pos + 1], axis: blob[pos + 2] }, pos + 3))
        }
        TAG_CONST => {
            need(blob, pos, 9)?;
            let mut a = [0u8; 8];
            a.copy_from_slice(&blob[pos + 1..pos + 9]);
            Ok((Dim::Const(i64::from_le_bytes(a)), pos + 9))
        }
        TAG_PARAM => {
            need(blob, pos, 2)?;
            Ok((Dim::Param(blob[pos + 1]), pos + 2))
        }
        TAG_ADD | TAG_SUB | TAG_MUL | TAG_DIV => {
            let (c1, p1) = read_child(blob, pos + 1)?;
            let (c2, p2) = read_child(blob, p1)?;
            let (a, b) = (Box::new(c1), Box::new(c2));
            let d = match tag {
                TAG_ADD => Dim::Add(a, b),
                TAG_SUB => Dim::Sub(a, b),
                TAG_MUL => Dim::Mul(a, b),
                _ => Dim::Div(a, b),
            };
            Ok((d, p2))
        }
        other => Err(ShapeExprError::ReservedTag { tag: other }),
    }
}

/// A `u16`-LE length-prefixed child expression at `pos`.
fn read_child(blob: &[u8], pos: usize) -> Result<(Dim, usize), ShapeExprError> {
    if blob.len() < pos + 2 {
        return Err(ShapeExprError::TruncatedBlob { need: 2, got: blob.len().saturating_sub(pos) });
    }
    let len = u16::from_le_bytes([blob[pos], blob[pos + 1]]) as usize;
    let start = pos + 2;
    if blob.len() < start + len {
        return Err(ShapeExprError::TruncatedBlob { need: len, got: blob.len().saturating_sub(start) });
    }
    // A child must consume its declared length exactly (decode_dim rejects slack).
    let child = decode_dim(&blob[start..start + len])?;
    Ok((child, start + len))
}

/// The blob must hold at least `n` bytes starting at `pos`.
fn need(blob: &[u8], pos: usize, n: usize) -> Result<(), ShapeExprError> {
    if blob.len() < pos + n {
        Err(ShapeExprError::TruncatedBlob { need: n, got: blob.len().saturating_sub(pos) })
    } else {
        Ok(())
    }
}

/// Decode one whole-shape `ShapeExpr` blob, rejecting a malformed one with a typed
/// decline (§6.20-0006). Reads `SameAs` (core) plus the registered experimental
/// extension constructors `WithDim`/`Dims` (§6.20-0009/-0010); the still-reserved
/// `Reduce` (0x09) and `DimExpr`-only tags decline typed. A well-formed blob
/// round-trips: `decode_shape(&s.encode()) == Ok(s)`.
pub fn decode_shape(blob: &[u8]) -> Result<ShapeExpr, ShapeExprError> {
    let (s, consumed) = decode_shape_at(blob, 0)?;
    if consumed != blob.len() {
        return Err(ShapeExprError::TrailingBytes { extra: blob.len() - consumed });
    }
    Ok(s)
}

fn decode_shape_at(blob: &[u8], pos: usize) -> Result<(ShapeExpr, usize), ShapeExprError> {
    let tag = *blob
        .get(pos)
        .ok_or(ShapeExprError::TruncatedBlob { need: pos + 1, got: blob.len() })?;
    match tag {
        0x00 => Err(ShapeExprError::ZeroTag),
        TAG_SAME_AS => {
            need(blob, pos, 2)?;
            Ok((ShapeExpr::SameAs { operand: blob[pos + 1] }, pos + 2))
        }
        // 0x0A, u8 operand, u8 axis, one u16-LE length-prefixed child (§6.20-0009).
        TAG_WITH_DIM => {
            need(blob, pos, 3)?;
            let operand = blob[pos + 1];
            let axis = blob[pos + 2];
            let (child, next) = read_child(blob, pos + 3)?;
            Ok((ShapeExpr::WithDim { operand, axis, dim: Box::new(child) }, next))
        }
        // 0x0B, u8 count, then count × u16-LE length-prefixed children (§6.20-0010).
        TAG_DIMS => {
            need(blob, pos, 2)?;
            let count = blob[pos + 1] as usize;
            let mut dims = Vec::with_capacity(count);
            let mut p = pos + 2;
            for _ in 0..count {
                let (child, next) = read_child(blob, p)?;
                dims.push(child);
                p = next;
            }
            Ok((ShapeExpr::Dims(dims), p))
        }
        other => Err(ShapeExprError::ReservedTag { tag: other }),
    }
}

// ---- evaluation (§6.20-0002/0003/0004) ---------------------------------------

/// Resolve a non-negative `axis` (or the [`LAST`] sentinel) against `rank`
/// (§6.20-0003): `last` is `rank − 1`; a concrete axis `>= rank`, or `last` on a
/// rank-0 operand, is a typed decline.
fn resolve_axis(axis: u8, rank: usize) -> Result<usize, ShapeExprError> {
    if axis == LAST {
        return rank.checked_sub(1).ok_or(ShapeExprError::AxisOutOfRange { axis, rank });
    }
    let a = axis as usize;
    if a >= rank {
        return Err(ShapeExprError::AxisOutOfRange { axis, rank });
    }
    Ok(a)
}

/// Floor division (rounds the quotient toward −∞), unlike Rust's truncating `/`.
fn floordiv(a: i64, b: i64) -> i64 {
    let (q, r) = (a / b, a % b);
    if r != 0 && ((r < 0) != (b < 0)) { q - 1 } else { q }
}

fn eval_binary(
    a: DimValue,
    b: DimValue,
    f: impl Fn(i64, i64) -> Result<i64, ShapeExprError>,
) -> Result<DimValue, ShapeExprError> {
    match (a, b) {
        (DimValue::Concrete(x), DimValue::Concrete(y)) => Ok(DimValue::Concrete(f(x, y)?)),
        // A gap in either operand propagates (§6.20-0004).
        _ => Ok(DimValue::Gap),
    }
}

/// Evaluate a `DimExpr` against the operands' concrete shapes and param values.
pub fn eval_dim(d: &Dim, operands: &[Vec<i64>], params: &[i64]) -> Result<DimValue, ShapeExprError> {
    match d {
        Dim::Extent { operand, axis } => {
            let op = *operand as usize;
            let shape = operands.get(op).ok_or(ShapeExprError::OperandOutOfRange {
                operand: *operand,
                operands: operands.len(),
            })?;
            let idx = resolve_axis(*axis, shape.len())?;
            let ext = shape[idx];
            Ok(if ext == SYMBOLIC { DimValue::Gap } else { DimValue::Concrete(ext) })
        }
        Dim::Const(c) => Ok(DimValue::Concrete(*c)),
        Dim::Param(f) => {
            let fi = *f as usize;
            let v = params.get(fi).ok_or(ShapeExprError::ParamOutOfRange {
                field: *f,
                params: params.len(),
            })?;
            Ok(DimValue::Concrete(*v))
        }
        Dim::Add(a, b) => eval_binary(eval_dim(a, operands, params)?, eval_dim(b, operands, params)?, |x, y| Ok(x + y)),
        Dim::Sub(a, b) => eval_binary(eval_dim(a, operands, params)?, eval_dim(b, operands, params)?, |x, y| Ok(x - y)),
        Dim::Mul(a, b) => eval_binary(eval_dim(a, operands, params)?, eval_dim(b, operands, params)?, |x, y| Ok(x * y)),
        Dim::Div(a, b) => eval_binary(eval_dim(a, operands, params)?, eval_dim(b, operands, params)?, |x, y| {
            if y == 0 { Err(ShapeExprError::DivideByZero) } else { Ok(floordiv(x, y)) }
        }),
    }
}

/// Evaluate a `ShapeExpr` to a concrete shape (or a surfaced gap).
pub fn eval_shape(s: &ShapeExpr, operands: &[Vec<i64>], params: &[i64]) -> Result<ShapeValue, ShapeExprError> {
    match s {
        ShapeExpr::SameAs { operand } => {
            let op = *operand as usize;
            let shape = operands.get(op).ok_or(ShapeExprError::OperandOutOfRange {
                operand: *operand,
                operands: operands.len(),
            })?;
            if shape.iter().any(|&e| e == SYMBOLIC) {
                Ok(ShapeValue::Gap)
            } else {
                Ok(ShapeValue::Concrete(shape.clone()))
            }
        }
        // §6.20-0009 `with_dim`: the operand's shape with the resolved axis replaced
        // by `dim`. A Gap replacement, or a symbolic extent in any *kept* axis,
        // surfaces the whole shape as a Gap (§6.20-0004); an out-of-range axis is a
        // typed decline (§6.20-0003), never a panic.
        ShapeExpr::WithDim { operand, axis, dim } => {
            let op = *operand as usize;
            let shape = operands.get(op).ok_or(ShapeExprError::OperandOutOfRange {
                operand: *operand,
                operands: operands.len(),
            })?;
            let idx = resolve_axis(*axis, shape.len())?;
            let replacement = eval_dim(dim, operands, params)?;
            let mut out = shape.clone();
            let mut gap = false;
            for (i, e) in out.iter_mut().enumerate() {
                if i == idx {
                    match &replacement {
                        DimValue::Concrete(v) => *e = *v,
                        DimValue::Gap => gap = true,
                    }
                } else if *e == SYMBOLIC {
                    gap = true;
                }
            }
            if gap { Ok(ShapeValue::Gap) } else { Ok(ShapeValue::Concrete(out)) }
        }
        // §6.20-0010 `dims`: build the whole shape from the ordered DimExprs. A Gap
        // in any element surfaces the whole shape as a Gap (§6.20-0004); an empty
        // list is the rank-0 scalar shape.
        ShapeExpr::Dims(dims) => {
            let mut out = Vec::with_capacity(dims.len());
            let mut gap = false;
            for d in dims {
                match eval_dim(d, operands, params)? {
                    DimValue::Concrete(v) => out.push(v),
                    DimValue::Gap => gap = true,
                }
            }
            if gap { Ok(ShapeValue::Gap) } else { Ok(ShapeValue::Concrete(out)) }
        }
    }
}

// ---- §6.20-0007 primitive-floor shape rules that ride op semantics ------------

/// The shape rule of a `reduce` family op: the input shape with the `reduce_axes`
/// removed (`keepdim = false`) or set to `1` (`keepdim = true`). This is derived
/// from the op's semantics (the `reduce_axes`/`keepdim` OpAttrs, §6.19-0020/0025),
/// not from a free shape attr — the "already-polymorphic primitive" path.
pub fn reduce_shape(input: &[i64], reduce_axes: &[usize], keepdim: bool) -> Vec<i64> {
    let set: std::collections::BTreeSet<usize> = reduce_axes.iter().copied().collect();
    let mut out = Vec::new();
    for (i, &e) in input.iter().enumerate() {
        if set.contains(&i) {
            if keepdim {
                out.push(1);
            }
        } else {
            out.push(e);
        }
    }
    out
}

// ---- §6.20-0008 the output-shape ≠ operand-shape class -----------------------

/// The shape rule of a `gather` / `index_select` / `embedding`: the data operand's
/// shape with the gathered `axis` replaced by the index operand's shape
/// (`data[..axis] ++ index ++ data[axis+1..]`). In general the output equals no
/// operand's shape — which is exactly why advertising `SameAs(data)` for a gather is
/// a bug the shape oracle catches (§6.20-0008).
pub fn gather_shape(data: &[i64], index: &[i64], axis: usize) -> Vec<i64> {
    let mut out = Vec::with_capacity(data.len() - 1 + index.len());
    out.extend_from_slice(&data[..axis]);
    out.extend_from_slice(index);
    out.extend_from_slice(&data[axis + 1..]);
    out
}

/// The shape rule of a `matmul` (contraction): its role-vector-derived output
/// (KISS-Classify §6.6-0016 M/N/K axis roles — carried as axis roles, not a
/// `ShapeExpr`). For the canonical same-rank ≥ 2 cell,
/// `lhs [..batch, M, K] · rhs [..batch, K, N] -> [..batch, M, N]`; the output
/// equals neither operand (§6.20-0008).
pub fn matmul_shape(lhs: &[i64], rhs: &[i64]) -> Vec<i64> {
    let r = lhs.len();
    let mut out = lhs[..r - 2].to_vec(); // aligned leading batch dims
    out.push(lhs[r - 2]); // M (lhs second-last)
    out.push(rhs[r - 1]); // N (rhs last)
    out
}

// ---- KISS-CONTRACT-6.4-0011 Interface output shape ⟷ op shape rule -----------

/// The Interface (§6.5) `declared` output shape is **consistent** iff it equals
/// the op's shape rule `computed` over the operand shapes. A surfaced [`ShapeValue::Gap`]
/// (symbolic/data-dependent output) is never a hard inconsistency — a consumer
/// cannot assert a mismatch it cannot compute. This is the shape-side companion to
/// the §6.4-0006 value oracle.
pub fn shape_consistent(declared: &[i64], computed: &ShapeValue) -> bool {
    match computed {
        ShapeValue::Concrete(c) => declared == c.as_slice(),
        ShapeValue::Gap => true,
    }
}
