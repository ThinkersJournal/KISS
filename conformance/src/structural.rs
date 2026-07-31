//! From-scratch CPU oracles for the **structural (multi-element) access atoms**
//! (KISS-Ops §6.11) and their determinism-class-aware comparators (KISS-Conform
//! §6.8).
//!
//! This is the multi-element companion to [`crate::semantics`] (the scalar float
//! oracle). Each oracle is the atom's normative rule transcribed directly from
//! KISS-Ops §6.11, sharing no lowering code with any generator (Conform §6.5-0002).
//! Covered: `reduce` (monoid fold, §6.11-0002), `prefix_scan` (inclusive/exclusive,
//! §6.11-0003), `gather` (indexed read + OOB policy, §6.11-0004), and `scatter`
//! (indexed write + combine, §6.11-0005/-0006/-0010).
//!
//! **The one nondeterministic atom.** Every atom here is determinism-class
//! **exact-byte** (§6.0-0002) *except* the floating-point `sum`/`prod` `reduce` and
//! `prefix_scan` monoids and the `scatter` floating-point `atomic-add` combine,
//! which are **order-invariant/nondeterministic** (§6.0-0004): floating-point
//! summation is not associative and KISS-Ops pins neither a canonical reduction
//! order nor an accumulator width. For those, an implementation's result is
//! compared under the **order-invariant comparator** (Conform §6.8-0004), which
//! accepts reassociation-induced differences within a *contract-declared* tolerance
//! rather than byte-for-byte. Those comparators are the [`order_invariant_agree`] /
//! [`compare_order_invariant`] helpers below.
//!
//! Scope: the scalar oracles operate over a flat `&[f32]` (one iteration axis). The
//! `keepdim` extent-1/stride-0 result view (§6.11-0008) and the multi-axis
//! `reduce_axes` descriptor (§6.11-0011) are OpAttrs/shape concerns handled
//! elsewhere; this slice pins the per-axis numeric fold, OOB, and combine algebra.

use crate::semantics::{add, max_prop, min_prop, mul};
use crate::DeterminismClass;

// =============================================================================
// reduce — associative-monoid fold over an axis (KISS-Ops §6.11-0002)
// =============================================================================

/// The `reduce` / `prefix_scan` fold monoid, one of `{sum, prod, max, min}`
/// (KISS-Ops §6.11-0002, §3 "Monoid"). Each carries a pinned identity element and
/// combine operator; `Max`/`Min` are NaN-propagating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Monoid {
    Sum,
    Prod,
    Max,
    Min,
}

impl Monoid {
    /// The monoid identity element for the `f32` compute dtype (KISS-Ops
    /// §6.11-0002): `sum → +0.0`, `prod → 1`, `max → −∞` (the dtype minimum),
    /// `min → +∞` (the dtype maximum). A reduction over an **empty** axis yields
    /// exactly this identity (§6.11-0002), and it is the seed of every fold.
    pub fn identity_f32(self) -> f32 {
        match self {
            Monoid::Sum => 0.0,
            Monoid::Prod => 1.0,
            Monoid::Max => f32::NEG_INFINITY,
            Monoid::Min => f32::INFINITY,
        }
    }

    /// The monoid combine on `f32`. `Sum`/`Prod` are the IEEE-754 `add`/`mul` atoms
    /// (`crate::semantics`); `Max`/`Min` are the **NaN-propagating** `max_prop` /
    /// `min_prop` atoms (§6.11-0002: "the `max` and `min` monoids MUST be
    /// NaN-propagating"), *not* the IEEE NaN-suppressing `fmax_ieee`/`fmin_ieee`.
    pub fn combine_f32(self, acc: f32, x: f32) -> f32 {
        match self {
            Monoid::Sum => add(acc, x),
            Monoid::Prod => mul(acc, x),
            Monoid::Max => max_prop(acc, x),
            Monoid::Min => min_prop(acc, x),
        }
    }

    /// The determinism/fidelity class this monoid's `reduce`/`prefix_scan` carries
    /// over an `f32` dtype (KISS-Ops §6.0). `Sum`/`Prod` fold with non-associative
    /// float `add`/`mul`, so they are **order-invariant/nondeterministic**
    /// (§6.0-0004); `Max`/`Min` are order-invariant *in value* and reproduce
    /// byte-for-byte **except for the sign of a zero result**, so they are classed
    /// **exact-byte** (§6.0-0002) but MUST be compared through
    /// [`compare_monoid_reduced_f32`], which canonicalizes ±0 before the byte
    /// compare. The exception is real and load-bearing: `max_prop`/`min_prop` return
    /// the *first* operand on a tie (`max_prop(+0.0,-0.0)=+0.0` but
    /// `max_prop(-0.0,+0.0)=-0.0`), so the SAME multiset folded in two valid orders
    /// (e.g. a parallel/tree vs. a sequential max-reduction) can differ in that one
    /// sign bit while agreeing in value — a naive byte-exact comparator would reject
    /// two conforming implementations. Integer `Sum`/`Prod` (wrapping, hence
    /// associative) would be exact-byte with no such caveat, but this f32 oracle only
    /// exercises the float path.
    pub fn class_f32(self) -> DeterminismClass {
        match self {
            Monoid::Sum | Monoid::Prod => DeterminismClass::OrderInvariant,
            Monoid::Max | Monoid::Min => DeterminismClass::ExactByte,
        }
    }
}

/// `reduce` — fold `xs` under `monoid` in the canonical left-to-right sequential
/// order, seeded with the monoid identity (KISS-Ops §6.11-0002). An **empty** input
/// yields the identity (the empty-reduction rule). This fixed order is *the oracle's*
/// ordering; for the `Sum`/`Prod` monoids an implementation-under-test MAY visit in
/// any order (§6.0-0004) and is compared under [`compare_order_invariant`], not
/// byte-exact — but the oracle itself is deterministic and reproducible.
///
/// NaN/±0 follow the atoms: `Max`/`Min` propagate any NaN (via `max_prop`/`min_prop`
/// seeded at ∓∞), and preserve signed zero (`reduce(Max, [-0.0]) = max_prop(−∞,−0.0)
/// = −0.0`); `Sum` of `[-0.0]` is `add(+0.0, −0.0) = +0.0` per IEEE-754.
pub fn reduce_f32(xs: &[f32], monoid: Monoid) -> f32 {
    xs.iter().fold(monoid.identity_f32(), |acc, &x| monoid.combine_f32(acc, x))
}

/// Rank-2 axis reduction: fold `axis` (0 or 1) of a row-major `[rows, cols]`
/// tensor with `monoid`, one output cell per surviving coordinate. Composes the
/// existing floor `reduce_f32` per slice — no new fold algorithm (§6.11-0002).
pub fn reduce_axis2_f32(data: &[f32], extents: [usize; 2], axis: usize, monoid: Monoid) -> Vec<f32> {
    let [rows, cols] = extents;
    assert_eq!(data.len(), rows * cols, "data length must equal rows*cols");
    match axis {
        1 => (0..rows).map(|r| reduce_f32(&data[r * cols..(r + 1) * cols], monoid)).collect(),
        0 => (0..cols)
            .map(|c| {
                let col: Vec<f32> = (0..rows).map(|r| data[r * cols + c]).collect();
                reduce_f32(&col, monoid)
            })
            .collect(),
        _ => panic!("rank-2 axis must be 0 or 1"),
    }
}

// =============================================================================
// prefix_scan — inclusive / exclusive running fold (KISS-Ops §6.11-0003)
// =============================================================================

/// Whether a `prefix_scan` element sees itself (`Inclusive`) or only its strict
/// predecessors (`Exclusive`). KISS-Ops §6.11-0003.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanKind {
    Inclusive,
    Exclusive,
}

/// `prefix_scan` — an inclusive or exclusive running monoid fold along one axis,
/// **length-preserving** (exactly one output per input position, §6.11-0003), and
/// distinct from `reduce` (§6.11-0002 collapses the axis; scan retains it,
/// §6.11-0008). `cumsum`/`cumprod`/`cummax` are `prefix_scan(sum|prod|max,
/// Inclusive)` (§6.13 table).
///
/// * `Inclusive`: `out[i] = fold(xs[0..=i])`.
/// * `Exclusive`: `out[i] = fold(xs[0..i])`, so `out[0]` is the monoid identity and
///   the final element `fold(xs[0..n-1])` excludes the last input.
///
/// An empty input yields an empty output (length preserved). Determinism class is
/// [`Monoid::class_f32`]: `Sum`/`Prod` scans are order-invariant/nondeterministic
/// (§6.0-0004 lists `prefix_scan(sum)`/`prefix_scan(prod)` explicitly), `Max`/`Min`
/// scans are exact-byte.
pub fn prefix_scan_f32(xs: &[f32], monoid: Monoid, kind: ScanKind) -> Vec<f32> {
    let mut out = Vec::with_capacity(xs.len());
    let mut acc = monoid.identity_f32();
    for &x in xs {
        match kind {
            ScanKind::Inclusive => {
                acc = monoid.combine_f32(acc, x);
                out.push(acc);
            }
            ScanKind::Exclusive => {
                out.push(acc);
                acc = monoid.combine_f32(acc, x);
            }
        }
    }
    out
}

// =============================================================================
// gather — data-dependent indexed read + OOB policy (KISS-Ops §6.11-0004)
// =============================================================================

/// The out-of-bounds policy for a `gather` **read** (KISS-Ops §6.11-0004):
/// `{skip, clamp, zero-fill}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OobRead {
    /// The addressed output position is **not written** (modeled as `None`).
    Skip,
    /// The index is clamped into `[0, len-1]` and that element is read.
    Clamp,
    /// The read yields `+0.0` (the dtype zero) regardless of the index.
    ZeroFill,
}

/// `gather` — substitute each runtime integer `index` for one iteration-axis
/// coordinate, a data-dependent read of `data`, under an OOB policy (KISS-Ops
/// §6.11-0004). Indices are modeled as `i64` to carry the signed index dtypes
/// (`i32`/`i64`, §6.11-0009); a **negative** index is **always** out-of-bounds —
/// there is no from-end wrap (§6.11-0004).
///
/// Returns one cell per index, length-preserving over the index vector:
/// * in-bounds `0 <= idx < data.len()` → `Some(data[idx])`;
/// * OOB (`idx < 0` or `idx >= len`) under `Skip` → `None` (position unwritten);
/// * OOB under `Clamp` → `Some(data[clamp(idx, 0, len-1)])`;
/// * OOB under `ZeroFill` → `Some(0.0)`.
///
/// Edge cases: (a) a negative index of any magnitude is OOB, never a Python-style
/// tail index; (b) `Clamp` over **empty** `data` is **out of contract** — there is
/// no in-range element to clamp to, a case §6.11-0004 does not pin. Rather than
/// invent a value that could silently diverge from another equally-valid oracle,
/// this oracle makes one *deliberate, pinned* choice: the position is treated as
/// unwritten — exactly as under `Skip` — and yields `None`. A conforming harness
/// MUST NOT use empty-`data` clamp gather as a certifying vector; the pin exists
/// only to keep this oracle total and reproducible, and is fixed by the
/// `gather_clamp_empty_is_pinned_none` test below. (c) `gather` is
/// determinism-class **exact-byte**
/// (§6.0-0002), so its cells compare bit-for-bit including the ±0/NaN payload of the
/// read datum.
pub fn gather_f32(data: &[f32], indices: &[i64], policy: OobRead) -> Vec<Option<f32>> {
    let len = data.len();
    indices
        .iter()
        .map(|&idx| {
            let in_bounds = idx >= 0 && (idx as u64) < len as u64;
            if in_bounds {
                Some(data[idx as usize])
            } else {
                match policy {
                    OobRead::Skip => None,
                    OobRead::ZeroFill => Some(0.0),
                    OobRead::Clamp => {
                        if len == 0 {
                            None
                        } else if idx < 0 {
                            Some(data[0])
                        } else {
                            Some(data[len - 1])
                        }
                    }
                }
            }
        })
        .collect()
}

// =============================================================================
// scatter — data-dependent indexed write + combine algebra (KISS-Ops §6.11-0005)
// =============================================================================

/// The write-combining operator of `scatter` (KISS-Ops §6.11-0005, §3 "Combine
/// algebra"): `{assign, atomic-add, atomic-max, atomic-min}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combine {
    /// Overwrite. Deterministic under the pinned tie-break: when several sources hit
    /// the same destination, the **highest source (row-major) index wins**
    /// (last-writer-in-iteration-order, §6.11-0006).
    Assign,
    /// `dest += src`. On a **floating-point** dtype this is the ONE
    /// order-invariant/nondeterministic combine (§6.11-0006, §6.0-0004); on an
    /// integer dtype it is deterministic. This oracle is f32, so it is nondet.
    AtomicAdd,
    /// `dest = max_prop(dest, src)` — NaN-propagating (§6.11-0010). Deterministic.
    AtomicMax,
    /// `dest = min_prop(dest, src)` — NaN-propagating (§6.11-0010). Deterministic.
    AtomicMin,
}

impl Combine {
    /// The determinism/fidelity class of this combine over an `f32` dtype (KISS-Ops
    /// §6.11-0006). `AtomicAdd` is **order-invariant/nondeterministic** (float
    /// summation is non-associative); `Assign`/`AtomicMax`/`AtomicMin` are
    /// **exact-byte** (deterministic). `Assign` is bit-exact without qualification
    /// (the pinned last-writer tie-break, §6.11-0006, so a scattered `-0.0` stays
    /// `-0.0`). `AtomicMax`/`AtomicMin` are exact-byte *in value* but carry the same
    /// signed-zero exception as the [`Monoid::Max`]/[`Monoid::Min`] folds — order can
    /// flip the sign of a zero result — so their buffers MUST be compared through
    /// [`compare_scattered_f32`], which canonicalizes ±0 before the byte compare.
    pub fn class_f32(self) -> DeterminismClass {
        match self {
            Combine::AtomicAdd => DeterminismClass::OrderInvariant,
            Combine::Assign | Combine::AtomicMax | Combine::AtomicMin => DeterminismClass::ExactByte,
        }
    }
}

/// `scatter` — substitute each runtime integer `indices[k]` for one output-axis
/// coordinate, a data-dependent write of `src[k]` into `dest`, combined under
/// `combine` (KISS-Ops §6.11-0005). Sources are visited in **row-major `k` order**
/// (`0..indices.len()`), the canonical order that fixes the `Assign` tie-break
/// (§6.11-0006). An **out-of-bounds** write (`idx < 0` or `idx >= dest.len()`) is
/// **skipped** — the only OOB policy defined for writes (§6.11-0005); a negative
/// index is always OOB (no wrap, §6.11-0004/-0009).
///
/// Combine semantics (per contributing source, in `k` order):
/// * `Assign` → `dest[idx] = src[k]`; the highest `k` writing a given `idx` wins
///   (§6.11-0006, pinned last-writer-in-iteration-order tie-break).
/// * `AtomicAdd` → `dest[idx] = add(dest[idx], src[k])`. On f32 the *value* depends
///   on visit order only up to FP reassociation (§6.0-0004); this sequential order
///   is one valid ordering — compare under [`compare_order_invariant`].
/// * `AtomicMax` → `dest[idx] = max_prop(dest[idx], src[k])` (NaN-propagating,
///   §6.11-0010: a NaN scattered to, or already at, a destination yields NaN).
/// * `AtomicMin` → `dest[idx] = min_prop(dest[idx], src[k])` (NaN-propagating).
///
/// `dest` must be pre-initialized to the combine's starting contents (e.g. the
/// scatter destination's prior values, or the monoid identity for a fresh buffer);
/// `scatter` mutates it in place. `src.len()` must equal `indices.len()`.
pub fn scatter_f32(dest: &mut [f32], indices: &[i64], src: &[f32], combine: Combine) {
    assert_eq!(indices.len(), src.len(), "scatter: indices and src must be equal length");
    let len = dest.len();
    for (k, &idx) in indices.iter().enumerate() {
        let in_bounds = idx >= 0 && (idx as u64) < len as u64;
        if !in_bounds {
            continue; // §6.11-0005: OOB writes are skipped.
        }
        let i = idx as usize;
        let s = src[k];
        dest[i] = match combine {
            Combine::Assign => s,
            Combine::AtomicAdd => add(dest[i], s),
            Combine::AtomicMax => max_prop(dest[i], s),
            Combine::AtomicMin => min_prop(dest[i], s),
        };
    }
}

// =============================================================================
// The order-invariant comparator (KISS-Conform §6.8-0004) — the crux
// =============================================================================
//
// A correct scatter floating-point atomic-add (and a float sum/prod reduce/scan)
// result is invariant to index-visit order ONLY UP TO FP reassociation. Two
// implementations that visit the same multiset of addends in different orders
// produce sums that generally differ in the last bits. Conform §6.8-0004 forbids a
// byte-exact comparator for such a result and requires a tolerance — and that
// tolerance is "the one declared in the contract Guarantees, never an
// implementation-chosen implicit default." So the tolerance is a REQUIRED PARAMETER
// of every helper here, never a hidden constant.

/// The classic worst-case reassociation error bound for summing `n_addends`
/// floating-point values whose absolute values sum to `abs_sum`:
/// `(n-1) · u · abs_sum`, with `u = 2^-24` the `f32` unit roundoff
/// (`f32::EPSILON / 2`). Any single evaluation order of the sum lies within this of
/// the exact real sum, so two *different* orders differ by at most `2×` it. A
/// caller MAY declare `2.0 * reassoc_bound_f32(n, abs_sum)` as the absolute
/// tolerance of its order-invariant comparison; this function is provided so a
/// declared tolerance can be justified rather than guessed. `n < 3` ⇒ `0.0`: fewer
/// than three addends have no reassociation freedom (`a + b` is bit-identical to
/// `b + a`, and a single addend or empty sum is trivially fixed), so no tolerance is
/// owed — granting one there would loosen the bound for a genuinely deterministic
/// result.
pub fn reassoc_bound_f32(n_addends: usize, abs_sum: f32) -> f32 {
    if n_addends < 3 {
        return 0.0;
    }
    let u = f32::EPSILON / 2.0; // 2^-24
    (n_addends as f32 - 1.0) * u * abs_sum.abs()
}

/// Two `f32` results **agree under the order-invariant/nondeterministic comparator**
/// (KISS-Conform §6.8-0004) iff:
/// * both are NaN (the standard NaN-equivalence relaxation, matching
///   [`crate::differential::agree`]); or
/// * both are the same signed infinity; or
/// * `|a − b| <= abs_tol + rel_tol · max(|a|, |b|)`.
///
/// `abs_tol`/`rel_tol` are the **contract-declared** tolerance (§6.8-0004), passed
/// explicitly — this function invents no default. A pure absolute bound is `rel_tol
/// = 0.0`; a pure relative bound is `abs_tol = 0.0`. This is the comparator the
/// harness selects for `scatter` `AtomicAdd`, float `reduce(Sum|Prod)`, and float
/// `prefix_scan(Sum|Prod)` — never a byte compare (§6.8-0004: "MUST NOT be a byte
/// compare across implementations or runs").
///
/// **Contract-range caveat (infinity vs. finite).** No tolerance can bridge an
/// infinity to a finite value: if one evaluation order overflows to an infinity while
/// another stays finite (e.g. f32 `[1e38, 1e38, -1e38]`, whose exact sum `1e38` is
/// representable but whose `(1e38 + 1e38) + (-1e38)` order saturates to `+inf`), the
/// two never agree and **no implementation can be certified** on that input. This is
/// deliberate — an order-dependent overflow is a genuine non-result, not a rounding
/// difference — but it silently narrows the certifiable set, so a contract that
/// relies on this comparator SHOULD declare an output-range guarantee that precludes
/// order-dependent overflow (or accept that such inputs are out of scope).
pub fn order_invariant_agree(a: f32, b: f32, abs_tol: f32, rel_tol: f32) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    if a.is_infinite() || b.is_infinite() {
        // An infinity only agrees with the identical signed infinity; a tolerance
        // cannot bridge an ∞ to a finite value.
        return a.to_bits() == b.to_bits();
    }
    let tol = abs_tol + rel_tol * a.abs().max(b.abs());
    (a - b).abs() <= tol
}

/// Compare two buffers element-wise under the order-invariant comparator
/// ([`order_invariant_agree`]). Intended for differencing two *orderings* of a
/// nondeterministic FP accumulation (e.g. a scatter atomic-add computed by the
/// oracle vs. by an implementation, or two permutations of the same scatter). Lengths
/// must match. Returns the first divergence, citing the position and both values.
pub fn compare_order_invariant(
    actual: &[f32],
    expected: &[f32],
    abs_tol: f32,
    rel_tol: f32,
) -> Result<(), String> {
    if actual.len() != expected.len() {
        return Err(format!(
            "order-invariant compare: length mismatch (actual {}, expected {})",
            actual.len(),
            expected.len()
        ));
    }
    for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
        if !order_invariant_agree(a, e, abs_tol, rel_tol) {
            let tol = abs_tol + rel_tol * a.abs().max(e.abs());
            return Err(format!(
                "order-invariant mismatch at [{i}]: actual {a}, expected {e} (|Δ|={} > tol {tol})",
                (a - e).abs()
            ));
        }
    }
    Ok(())
}

/// A determinism-class-aware scalar comparator for a reduced/combined `f32` result
/// (KISS-Conform §6.8-0006: "the comparator MUST be selected by the clause's
/// declared determinism/fidelity class"). Dispatches on `class`:
/// * `ExactByte` → the crate's byte-exact `f32` comparator ([`crate::compare_f32`]),
///   so `Max`/`Min` reductions and deterministic combines compare bit-for-bit;
/// * `OrderInvariant` → [`order_invariant_agree`] with the declared tolerance, so a
///   float `Sum`/`Prod` reduction or a `scatter` atomic-add is accepted up to
///   reassociation;
/// * `UlpTolerance` → delegates to [`crate::compare_f32`] treating `abs_tol`'s
///   integer part as a ULP bound (no transcendental atom appears in this slice, so
///   this arm is present only for completeness and total dispatch).
///
/// The tolerance is a parameter, honoring the "no implementation-chosen implicit
/// default" rule (§6.8-0004).
pub fn compare_reduced_f32(
    class: DeterminismClass,
    actual: f32,
    expected: f32,
    abs_tol: f32,
    rel_tol: f32,
) -> Result<(), String> {
    match class {
        DeterminismClass::ExactByte => {
            crate::compare_f32(DeterminismClass::ExactByte, actual, expected, 0)
        }
        DeterminismClass::OrderInvariant => {
            if order_invariant_agree(actual, expected, abs_tol, rel_tol) {
                Ok(())
            } else {
                let tol = abs_tol + rel_tol * actual.abs().max(expected.abs());
                Err(format!(
                    "order-invariant mismatch: actual {actual}, expected {expected} (|Δ|={} > tol {tol})",
                    (actual - expected).abs()
                ))
            }
        }
        DeterminismClass::UlpTolerance => {
            crate::compare_f32(DeterminismClass::UlpTolerance, actual, expected, abs_tol as u64)
        }
    }
}

/// Canonicalize signed zero: map both `-0.0` and `+0.0` to `+0.0`, and pass
/// everything else through unchanged (finite non-zero, the infinities, and NaN with
/// its exact payload — `x == 0.0` is `false` for all of them). This is the ±0
/// normalization applied before a byte compare of a float `max`/`min` result, where
/// the sign of a zero is the *only* bit a change of fold order can flip (see
/// [`Monoid::class_f32`]); it is applied nowhere else, so no other bit difference is
/// masked.
pub(crate) fn canon_signed_zero(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else {
        x
    }
}

/// Compare a float `reduce`/`prefix_scan` monoid result under the monoid's
/// determinism class — the entry the harness MUST select for a monoid fold (in
/// preference to the raw class dispatch of [`compare_reduced_f32`], which cannot see
/// the monoid). `Sum`/`Prod` route to the order-invariant comparator
/// ([`order_invariant_agree`]) within the contract-declared `abs_tol`/`rel_tol`.
/// `Max`/`Min` route to a byte-exact compare that FIRST canonicalizes ±0 on both
/// sides ([`canon_signed_zero`]): they are bit-exact for every valid fold order
/// *except* the sign of a zero result (§6.0-0002 note; see [`Monoid::class_f32`]), so
/// this accepts exactly that signed-zero reordering — two conforming tree/sequential
/// max-reductions agree — and rejects every other bit difference. `abs_tol`/`rel_tol`
/// are ignored for `Max`/`Min` (no tolerance beyond ±0 is admitted).
pub fn compare_monoid_reduced_f32(
    monoid: Monoid,
    actual: f32,
    expected: f32,
    abs_tol: f32,
    rel_tol: f32,
) -> Result<(), String> {
    match monoid {
        Monoid::Sum | Monoid::Prod => {
            compare_reduced_f32(DeterminismClass::OrderInvariant, actual, expected, abs_tol, rel_tol)
        }
        Monoid::Max | Monoid::Min => crate::compare_f32(
            DeterminismClass::ExactByte,
            canon_signed_zero(actual),
            canon_signed_zero(expected),
            0,
        ),
    }
}

/// Compare two `scatter` result buffers under the combine's determinism class — the
/// buffer companion to [`compare_monoid_reduced_f32`]. `AtomicAdd` -> the
/// order-invariant comparator ([`compare_order_invariant`]) within the declared
/// tolerance. `AtomicMax`/`AtomicMin` -> element-wise byte-exact AFTER ±0
/// canonicalization (the same signed-zero exception as the max/min monoids).
/// `Assign` -> strict element-wise byte-exact: a scattered `-0.0` MUST stay `-0.0`
/// (the last-writer tie-break is fully bit-deterministic, §6.11-0006), so ±0 is
/// deliberately NOT canonicalized for `Assign`.
pub fn compare_scattered_f32(
    combine: Combine,
    actual: &[f32],
    expected: &[f32],
    abs_tol: f32,
    rel_tol: f32,
) -> Result<(), String> {
    if actual.len() != expected.len() {
        return Err(format!(
            "scatter compare: length mismatch (actual {}, expected {})",
            actual.len(),
            expected.len()
        ));
    }
    match combine {
        Combine::AtomicAdd => compare_order_invariant(actual, expected, abs_tol, rel_tol),
        Combine::Assign => {
            for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
                if a.to_bits() != e.to_bits() {
                    return Err(format!(
                        "scatter Assign exact-byte mismatch at [{i}]: actual {:#010X}, expected {:#010X}",
                        a.to_bits(),
                        e.to_bits()
                    ));
                }
            }
            Ok(())
        }
        Combine::AtomicMax | Combine::AtomicMin => {
            for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
                if canon_signed_zero(a).to_bits() != canon_signed_zero(e).to_bits() {
                    return Err(format!(
                        "scatter {combine:?} exact-byte(±0-canon) mismatch at [{i}]: actual {:#010X}, expected {:#010X}",
                        a.to_bits(),
                        e.to_bits()
                    ));
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod structural_tests {
    use super::*;

    #[test]
    fn empty_reduction_is_monoid_identity() {
        // KISS-OPS §6.11-0002: a reduction over an empty axis yields the identity.
        assert_eq!(reduce_f32(&[], Monoid::Sum).to_bits(), 0.0f32.to_bits());
        assert_eq!(reduce_f32(&[], Monoid::Prod), 1.0);
        assert_eq!(reduce_f32(&[], Monoid::Max), f32::NEG_INFINITY);
        assert_eq!(reduce_f32(&[], Monoid::Min), f32::INFINITY);
    }

    #[test]
    fn reduce_max_propagates_nan_and_preserves_signed_zero() {
        // §6.11-0002: max/min monoids are NaN-propagating.
        assert!(reduce_f32(&[1.0, f32::NAN, 2.0], Monoid::Max).is_nan());
        assert!(reduce_f32(&[1.0, f32::NAN, 2.0], Monoid::Min).is_nan());
        // signed zero preserved through the max_prop atom seeded at -inf.
        assert_eq!(reduce_f32(&[-0.0], Monoid::Max).to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn exclusive_scan_starts_at_identity_and_is_length_preserving() {
        // §6.11-0003: length-preserving; exclusive out[0] = identity.
        let inc = prefix_scan_f32(&[1.0, 2.0, 3.0], Monoid::Sum, ScanKind::Inclusive);
        let exc = prefix_scan_f32(&[1.0, 2.0, 3.0], Monoid::Sum, ScanKind::Exclusive);
        assert_eq!(inc, vec![1.0, 3.0, 6.0]);
        assert_eq!(exc, vec![0.0, 1.0, 3.0]);
        assert_eq!(inc.len(), 3);
        assert_eq!(exc.len(), 3);
    }

    #[test]
    fn gather_negative_index_is_oob_never_wraps() {
        // §6.11-0004: negative index always OOB, no from-end wrap.
        let d = [10.0, 20.0, 30.0];
        assert_eq!(gather_f32(&d, &[-1], OobRead::Skip), vec![None]);
        assert_eq!(gather_f32(&d, &[-1], OobRead::ZeroFill), vec![Some(0.0)]);
        assert_eq!(gather_f32(&d, &[-1], OobRead::Clamp), vec![Some(10.0)]); // clamp to 0
        assert_eq!(gather_f32(&d, &[5], OobRead::Clamp), vec![Some(30.0)]); // clamp to len-1
    }

    #[test]
    fn scatter_assign_highest_source_index_wins() {
        // §6.11-0006: last-writer-in-iteration-order tie-break.
        let mut dest = [0.0; 2];
        scatter_f32(&mut dest, &[0, 0, 0], &[1.0, 2.0, 3.0], Combine::Assign);
        assert_eq!(dest[0], 3.0);
    }

    #[test]
    fn scatter_oob_write_is_skipped() {
        // §6.11-0005: OOB writes skipped, dest unchanged there.
        let mut dest = [7.0, 8.0];
        scatter_f32(&mut dest, &[-1, 9], &[1.0, 2.0], Combine::AtomicAdd);
        assert_eq!(dest, [7.0, 8.0]);
    }

    #[test]
    fn scatter_atomic_max_propagates_nan() {
        // §6.11-0010: fp atomic-max/min are NaN-propagating.
        let mut dest = [1.0];
        scatter_f32(&mut dest, &[0], &[f32::NAN], Combine::AtomicMax);
        assert!(dest[0].is_nan());
    }

    #[test]
    fn order_invariant_accepts_reassociation_but_exact_byte_rejects() {
        // The crux (§6.11-0006 / §6.0-0004 / Conform §6.8-0004): two visit orders of
        // the SAME scatter atomic-add multiset produce genuinely different f32 sums,
        // because ±1e8 swamps the 1.0 depending on order.
        let mut a = [0.0]; // ((0 + 1e8) + -1e8) + 1.0 = 1.0
        scatter_f32(&mut a, &[0, 0, 0], &[1e8_f32, -1e8, 1.0], Combine::AtomicAdd);
        let mut b = [0.0]; // ((0 + 1e8) + 1.0) + -1e8 = 0.0
        scatter_f32(&mut b, &[0, 0, 0], &[1e8_f32, 1.0, -1e8], Combine::AtomicAdd);
        assert_eq!(a[0], 1.0);
        assert_eq!(b[0], 0.0);
        // Exact-byte would REJECT this real divergence (that is the wrong comparator
        // for a nondeterministic op) ...
        assert!(a[0].to_bits() != b[0].to_bits());
        assert!(crate::compare_f32(DeterminismClass::ExactByte, a[0], b[0], 0).is_err());
        // ... the order-invariant comparator ACCEPTS it within the reassociation
        // tolerance derived from the addend magnitudes.
        let abs_sum = 1e8_f32 + 1e8 + 1.0;
        let tol = 2.0 * reassoc_bound_f32(3, abs_sum);
        assert!(compare_order_invariant(&a, &b, tol, 0.0).is_ok());
    }

    #[test]
    fn class_dispatch_matches_spec() {
        // §6.0-0002 / §6.0-0004: max/min exact-byte, sum/prod order-invariant.
        assert_eq!(Monoid::Max.class_f32(), DeterminismClass::ExactByte);
        assert_eq!(Monoid::Min.class_f32(), DeterminismClass::ExactByte);
        assert_eq!(Monoid::Sum.class_f32(), DeterminismClass::OrderInvariant);
        // §6.0-0004 lists prod explicitly as order-invariant.
        assert_eq!(Monoid::Prod.class_f32(), DeterminismClass::OrderInvariant);
        // scatter combines: only fp atomic-add is order-invariant (§6.11-0006).
        assert_eq!(Combine::AtomicAdd.class_f32(), DeterminismClass::OrderInvariant);
        assert_eq!(Combine::Assign.class_f32(), DeterminismClass::ExactByte);
        assert_eq!(Combine::AtomicMax.class_f32(), DeterminismClass::ExactByte);
        assert_eq!(Combine::AtomicMin.class_f32(), DeterminismClass::ExactByte);
    }

    #[test]
    fn reduce_max_signed_zero_is_order_dependent_in_bits() {
        // The signed-zero hazard the ExactByte class must NOT be compared naively for:
        // the same multiset {+0.0, -0.0} folded in two valid orders gives max results
        // that agree in value but differ in the sign bit, because max_prop returns the
        // first operand on a tie.
        let pos_first = reduce_f32(&[0.0, -0.0], Monoid::Max);
        let neg_first = reduce_f32(&[-0.0, 0.0], Monoid::Max);
        assert_eq!(pos_first.to_bits(), 0x0000_0000); // +0.0
        assert_eq!(neg_first.to_bits(), 0x8000_0000); // -0.0
        assert_ne!(pos_first.to_bits(), neg_first.to_bits());
        // A raw byte-exact comparator would WRONGLY reject these two conforming folds.
        assert!(crate::compare_f32(DeterminismClass::ExactByte, pos_first, neg_first, 0).is_err());
        // The monoid-aware comparator canonicalizes ±0 and ACCEPTS them ...
        assert!(compare_monoid_reduced_f32(Monoid::Max, pos_first, neg_first, 0.0, 0.0).is_ok());
        assert!(compare_monoid_reduced_f32(Monoid::Min, -0.0, 0.0, 0.0, 0.0).is_ok());
        // ... while still rejecting a genuine (non-±0) bit difference.
        assert!(compare_monoid_reduced_f32(Monoid::Max, 1.0, 2.0, 0.0, 0.0).is_err());
    }

    #[test]
    fn empty_prefix_scan_is_empty() {
        // §6.11-0003: prefix_scan is length-preserving, so an empty input yields an
        // empty output — no spurious identity element is seeded into the result.
        assert_eq!(prefix_scan_f32(&[], Monoid::Sum, ScanKind::Exclusive), Vec::<f32>::new());
        assert_eq!(prefix_scan_f32(&[], Monoid::Sum, ScanKind::Inclusive), Vec::<f32>::new());
        assert_eq!(prefix_scan_f32(&[], Monoid::Max, ScanKind::Inclusive), Vec::<f32>::new());
    }

    #[test]
    fn gather_clamp_empty_is_pinned_none() {
        // Out-of-contract degenerate (§6.11-0004 does not pin): Clamp over empty data
        // has no in-range element. This oracle's PINNED choice is None (unwritten, as
        // under Skip) — asserted here so the behavior is deliberate, not incidental.
        let empty: [f32; 0] = [];
        assert_eq!(
            gather_f32(&empty, &[0, -1, 5], OobRead::Clamp),
            vec![None, None, None]
        );
    }

    #[test]
    fn reassoc_bound_is_zero_below_three_addends() {
        // Fewer than three addends have no reassociation freedom (a+b == b+a bit for
        // bit), so the bound must be exactly 0.0 — not a spuriously loosened tolerance.
        assert_eq!(reassoc_bound_f32(0, 1e9), 0.0);
        assert_eq!(reassoc_bound_f32(1, 1e9), 0.0);
        assert_eq!(reassoc_bound_f32(2, 1e9), 0.0);
        // three addends: the first case with genuine freedom, so the bound is > 0.
        assert!(reassoc_bound_f32(3, 3.0) > 0.0);
    }

    #[test]
    fn scatter_atomic_minmax_signed_zero_agrees_under_canon() {
        // Two valid visit orders of an atomic-max/min can differ only in the sign of a
        // zero result; the combine-aware buffer comparator canonicalizes ±0 and
        // accepts them, while Assign (fully bit-deterministic) does NOT.
        let a = [0.0f32]; // +0.0
        let b = [-0.0f32]; // -0.0, same value
        assert_ne!(a[0].to_bits(), b[0].to_bits());
        assert!(compare_scattered_f32(Combine::AtomicMax, &a, &b, 0.0, 0.0).is_ok());
        assert!(compare_scattered_f32(Combine::AtomicMin, &a, &b, 0.0, 0.0).is_ok());
        // Assign keeps ±0 distinct (last-writer is bit-exact, §6.11-0006).
        assert!(compare_scattered_f32(Combine::Assign, &a, &b, 0.0, 0.0).is_err());
    }

    #[test]
    fn reduce_axis2_folds_the_named_axis() {
        // row-major [2,3]: rows [1,2,3],[4,5,6]
        let d = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        // axis 1 (trailing) → per-row: [6, 15]
        assert_eq!(reduce_axis2_f32(&d, [2, 3], 1, Monoid::Sum), vec![6.0, 15.0]);
        // axis 0 → per-column: [5, 7, 9]
        assert_eq!(reduce_axis2_f32(&d, [2, 3], 0, Monoid::Sum), vec![5.0, 7.0, 9.0]);
        // Max axis 1 → [3, 6]; different monoid, different result (teeth vs a hardcoded Sum)
        assert_eq!(reduce_axis2_f32(&d, [2, 3], 1, Monoid::Max), vec![3.0, 6.0]);
    }
}

// =============================================================================
// sort_network — stable per-row permutation under a total order (KISS-Ops §6.11-0007)
// =============================================================================

use std::cmp::Ordering;

/// The `sort_network` sort direction (KISS-Ops §6.11-0007): one of
/// `{ascending, descending}`, default `ascending`. This is the *numeric-oracle* twin
/// of the wire-ordinal [`crate::opattrs::SortDirection`], exactly as [`Monoid`] here is
/// the twin of [`crate::opattrs::Monoid`]: this one drives the fold/permutation, that
/// one only encodes the OpAttrs byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// The **key** half of the `sort_network` total order (KISS-Ops §6.11-0007): a
/// comparison in which **NaN orders as the greatest value** and every non-NaN pair
/// follows the IEEE numeric order (so `-0.0` and `+0.0` compare *equal* — a tie that
/// the original-index rule resolves). Because NaN is forced greatest here, a NaN never
/// yields the non-transitive `false`-on-both-sides result a raw `a < b` / `partial_cmp`
/// gives against a NaN — the exact break §6.11-0007's total order forbids.
fn nan_greatest_cmp(a: f32, b: f32) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        // Neither is NaN: `partial_cmp` is total on the non-NaN f32 domain.
        (false, false) => a.partial_cmp(&b).expect("non-NaN f32 pair is totally ordered"),
    }
}

/// The full `sort_network` **total order** on `(key, original_index)` pairs (KISS-Ops
/// §6.11-0007). The primary comparison is [`nan_greatest_cmp`] (NaN greatest),
/// **reversed** for `Descending` so the greatest key — NaN — lands *first* (ascending →
/// NaN last, descending → NaN first). The secondary comparison is the **original index,
/// ascending in BOTH directions**: equal keys retain their input order (the pinned
/// stability rule, "ties break by the lower original index"), which does *not* flip with
/// the sort direction. Because original indices are unique, this is a strict total order
/// (never `Equal` for two distinct positions), so the permutation is fixed regardless of
/// the underlying sort's own stability.
fn sort_network_cmp(direction: SortDirection, a: (f32, usize), b: (f32, usize)) -> Ordering {
    let key = nan_greatest_cmp(a.0, b.0);
    let key = match direction {
        SortDirection::Ascending => key,
        SortDirection::Descending => key.reverse(),
    };
    // Stability: lower original index first, ascending in both directions.
    key.then_with(|| a.1.cmp(&b.1))
}

/// `sort_network` — a **stable per-row permutation** under the total order
/// [`sort_network_cmp`] (KISS-Ops §6.11-0007). Returns the op's **two** pinned outputs:
/// * `.0` (values) — the keys written back as a **raw-bit permutation** of the input
///   (a `-0.0` sign or a NaN payload rides along on its element, never normalized), and
/// * `.1` (index vector) — for each output rank, the **source position** that key came
///   from (the vector `argmax` reads rank 0 of under `Descending`, §6.13 table).
///
/// NaN sorts to one end (last under `Ascending`, first under `Descending`) and equal
/// keys — including `±0` and two NaNs — keep their input order (lower original index
/// first). Determinism-class **exact-byte** (§6.0-0002): a structural atom with no
/// monoid, its outputs compare bit-for-bit.
pub fn sort_network(row: &[f32], direction: SortDirection) -> (Vec<f32>, Vec<usize>) {
    let mut order: Vec<usize> = (0..row.len()).collect();
    order.sort_by(|&i, &j| sort_network_cmp(direction, (row[i], i), (row[j], j)));
    let values = order.iter().map(|&i| row[i]).collect();
    (values, order)
}
