# Harness Increment 3a — Axis Reduction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Add a rank-2 **axis reduction** (`reduce(monoid, axis=1)`) differential over two monoids (Sum, Max) — two dissimilar C kernels agree per-row within the reassociation band (Sum) / bit-exact ±0-canon (Max), a **wrong-axis** kernel is caught. Backs `KISS-OPS-6.11-0002`; strengthens 6.13-0006/0006a to the axis case; the Sum(nondet)/Max(exact-byte) multi-class foundation for 3b.

**Architecture:** New `structural::reduce_axis2_f32` (axis oracle) + axis extensions to `harness::{abi,resolver,differ,corpus}`, reusing inc-1/2's §6.5 FFI + `structural::{reduce_f32, reassoc_bound_f32, compare_monoid_reduced_f32}`. Candidate C kernels reduce a **baked** axis (axis is OpAttrs, never a launch scalar, per §6.19-0031).

**Tech Stack:** Rust (edition 2021, stdlib only), raw Win32 FFI, MSVC `cl.exe`, C fixtures.

## Global Constraints
- Stdlib only — no crate deps; no `Cargo.toml` change; no `build.rs`.
- Every `unsafe` block preceded by a `// SAFETY:` comment; `unsafe` confined to `loader` + the `invoke_axis_reduce` call site.
- `#[cfg(windows)]` on any loader-dependent test (ubuntu leg compiles them out; the windows-latest CI leg is the evidence). `structural`/`resolver`/`abi` unit tests are pure Rust (both legs).
- Edition 2021; crate name `kiss_conformance`. Verified reuse: `structural::reduce_f32(xs,&[f32]→f32, monoid: Monoid)`, `Monoid::{Sum,Max}`, `reassoc_bound_f32(n: usize, abs_sum: f32) -> f32` (`(n-1)·(EPS/2)·|abs_sum|`, 0 for n<3), `compare_monoid_reduced_f32(monoid, actual, expected, abs_tol, rel_tol) -> Result<(),String>` (Sum→order-invariant band, Max→±0-canon exact-byte). Types to mirror: `differ::Divergence{index,expected,actual,...}`, `corpus::Vector{...,provenance}`, `abi::ReduceKernel`/`invoke_reduce`, `common::compile_and_load_reduce`.
- Commit trailers:
  ```
  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
  ```
- Worktree: all work in `C:/Projects/kiss-axisreduce` on `feat/harness-axis-reduce`; run `cargo` from its `conformance/`.

---

## File Structure
| File | Responsibility |
|---|---|
| `conformance/src/structural.rs` | Modify (ADD): `reduce_axis2_f32`. |
| `conformance/src/harness/resolver.rs` | Modify (ADD): `reduce_axis_oracle`, `reduce_axis_abs_tol`. |
| `conformance/src/harness/abi.rs` | Modify (ADD): `AxisReduceKernel`, `invoke_axis_reduce`. |
| `conformance/src/harness/differ.rs` | Modify (ADD): `AxisDivergence`, `run_axis_reduce`. |
| `conformance/src/harness/corpus.rs` | Modify (ADD): `AxisVector`, `tagged_axis_corpus`. |
| `conformance/tests/harness_fixtures/reduce_axis1_{sum,max}_{a,b,wrong}.c` | Create: 6 fixtures. |
| `conformance/tests/common/mod.rs` | Modify (ADD): `compile_and_load_axis_reduce`. |
| `conformance/tests/harness_reduce_axis_differential.rs` | Create: the freeze-gate test (`#![cfg(windows)]`). |

---

### Task 1: `structural::reduce_axis2_f32` — the axis oracle
**Files:** Modify `conformance/src/structural.rs` (add after `reduce_f32`); test in its `#[cfg(test)] mod tests`.
**Interfaces:** Produces `structural::reduce_axis2_f32(data: &[f32], extents: [usize;2], axis: usize, monoid: Monoid) -> Vec<f32>`.

- [ ] **Step 1: Write the failing test** (add to structural.rs's test module):
```rust
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
```
- [ ] **Step 2: Run it, verify it fails** — `cargo test -p kiss-conformance --lib structural::tests::reduce_axis2 2>&1 | tail` → FAIL (`cannot find function reduce_axis2_f32`).
- [ ] **Step 3: Implement** — add after `reduce_f32` in `structural.rs`:
```rust
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
```
- [ ] **Step 4: Run it, verify pass** — `cargo test -p kiss-conformance --lib structural::tests::reduce_axis2 2>&1 | tail` → PASS.
- [ ] **Step 5: Commit** — `git add conformance/src/structural.rs`; commit: `structural: rank-2 axis reduction oracle (reduce_axis2_f32, §6.11-0002)`.

---

### Task 2: `resolver` axis oracle + per-row tolerance
**Files:** Modify `conformance/src/harness/resolver.rs`; test in its test module.
**Interfaces:** Consumes `crate::structural::{reduce_axis2_f32, reassoc_bound_f32, Monoid}`. Produces `resolver::reduce_axis_oracle(data, extents, axis, monoid) -> Vec<f32>`; `resolver::reduce_axis_abs_tol(data, extents, axis) -> Vec<f32>`.

- [ ] **Step 1: Write the failing test** (add to resolver.rs's test module):
```rust
#[test]
fn axis_oracle_and_per_row_band() {
    let d = [1.0f32, -2.0, 3.0, 4.0, 5.0, 6.0]; // [2,3]
    assert_eq!(reduce_axis_oracle(&d, [2, 3], 1, crate::structural::Monoid::Sum), vec![2.0, 15.0]);
    // per-row band: row0 abs-sum=6 over 3 addends, row1 abs-sum=15 over 3.
    let t = reduce_axis_abs_tol(&d, [2, 3], 1);
    assert_eq!(t.len(), 2);
    assert_eq!(t[0], 2.0 * crate::structural::reassoc_bound_f32(3, 6.0));
    assert_eq!(t[1], 2.0 * crate::structural::reassoc_bound_f32(3, 15.0));
}
```
- [ ] **Step 2: Run it, verify it fails** — `cargo test -p kiss-conformance --lib harness::resolver::tests::axis 2>&1 | tail` → FAIL.
- [ ] **Step 3: Implement** — add to `resolver.rs`:
```rust
use crate::structural::reduce_axis2_f32;

/// The `reduce(monoid, axis)` oracle for a rank-2 tensor — the §6.11-0002
/// primitive, evaluated by the floor `reduce_axis2_f32`.
pub fn reduce_axis_oracle(data: &[f32], extents: [usize; 2], axis: usize, monoid: crate::structural::Monoid) -> Vec<f32> {
    reduce_axis2_f32(data, extents, axis, monoid)
}

/// Per-output-cell absolute tolerance for a Sum-monoid axis reduction: `2 ×`
/// the reassociation band of THAT cell's own reduced slice (its own length +
/// abs-sum bound its own reassociation, not the whole tensor's). The 2× is the
/// two-order rule (candidate order vs oracle order — see reduce_mean_abs_tol).
/// (Max/Min are exact-byte; this band is ignored on the exact-byte path.)
pub fn reduce_axis_abs_tol(data: &[f32], extents: [usize; 2], axis: usize) -> Vec<f32> {
    let [rows, cols] = extents;
    let band = |slice_abs_sum: f32, len: usize| 2.0 * reassoc_bound_f32(len, slice_abs_sum);
    match axis {
        1 => (0..rows)
            .map(|r| band(data[r * cols..(r + 1) * cols].iter().map(|x| x.abs()).sum(), cols))
            .collect(),
        0 => (0..cols)
            .map(|c| band((0..rows).map(|r| data[r * cols + c].abs()).sum(), rows))
            .collect(),
        _ => panic!("rank-2 axis must be 0 or 1"),
    }
}
```
*(Note: `reassoc_bound_f32` is already imported in resolver.rs from Task-2 of inc 2; do not double-import.)*
- [ ] **Step 4: Run it, verify pass.** → PASS.
- [ ] **Step 5: Commit** — `git add conformance/src/harness/resolver.rs`; commit: `harness: axis-reduce oracle + per-row reassociation band`.

---

### Task 3: `abi` axis-reduce marshaller
**Files:** Modify `conformance/src/harness/abi.rs`; test in its test module.
**Interfaces:** Produces `abi::AxisReduceKernel` (`unsafe extern "C" fn(*const f32, *mut f32, *const i64, *const i64, i64)`); `abi::invoke_axis_reduce(kernel, data: &[f32], extents_in: [i64;2], extents_out: [i64;2]) -> Vec<f32>`.

- [ ] **Step 1: Write the failing test** (add to abi.rs's test module):
```rust
unsafe extern "C" fn rust_axis1_sum(inp: *const f32, out: *mut f32, ein: *const i64, _eout: *const i64, _n: i64) {
    // SAFETY: ein points to [rows, cols]; inp has rows*cols readable; out has rows writable.
    let (rows, cols) = unsafe { (*ein.offset(0), *ein.offset(1)) };
    for r in 0..rows {
        let mut s = 0.0f32;
        for c in 0..cols { s += unsafe { *inp.offset((r * cols + c) as isize) }; }
        unsafe { *out.offset(r as isize) = s };
    }
}
#[test]
fn marshals_and_invokes_axis_reduce() {
    let d = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]; // [2,3], reduce axis 1 → [6,15]
    assert_eq!(invoke_axis_reduce(rust_axis1_sum, &d, [2, 3], [2, 1]), vec![6.0, 15.0]);
}
```
- [ ] **Step 2: Run it, verify it fails.** → FAIL.
- [ ] **Step 3: Implement** — append to `abi.rs`:
```rust
/// A §6.5 rank-2 axis-reduction kernel entry: input ptr, output ptr, `extents_in`
/// (class 1, length rank), `extents_out` (class 1), `n` (class 3 = total input
/// count). The reduced axis is baked into the kernel (OpAttrs, §6.19-0031), not a
/// launch scalar.
pub type AxisReduceKernel = unsafe extern "C" fn(*const f32, *mut f32, *const i64, *const i64, i64);

/// Invoke an axis-reduction `kernel`; returns the `∏extents_out`-length output.
pub fn invoke_axis_reduce(kernel: AxisReduceKernel, data: &[f32], extents_in: [i64; 2], extents_out: [i64; 2]) -> Vec<f32> {
    let n_in: i64 = extents_in.iter().product();
    assert_eq!(data.len() as i64, n_in, "data length must equal ∏extents_in");
    let n_out = extents_out.iter().product::<i64>() as usize;
    let mut out = vec![0.0f32; n_out];
    // SAFETY: `data` exposes n_in readable f32; `out` exposes n_out writable f32;
    // `extents_in`/`extents_out` are 2-elem i64 arrays; all pointers non-overlapping
    // (distinct allocations); n_in fits i64. The kernel writes exactly out[0..n_out].
    unsafe {
        kernel(data.as_ptr(), out.as_mut_ptr(), extents_in.as_ptr(), extents_out.as_ptr(), n_in);
    }
    out
}
```
- [ ] **Step 4: Run it, verify pass.** → PASS.
- [ ] **Step 5: Commit** — `git add conformance/src/harness/abi.rs`; commit: `harness: §6.5 axis-reduce marshaller (invoke_axis_reduce, class-1 extents)`.

---

### Task 4: `differ` array reducer + `corpus` tagged axis corpus
**Files:** Modify `conformance/src/harness/differ.rs` + `conformance/src/harness/corpus.rs`; tests in each.
**Interfaces:** Produces `differ::AxisDivergence{cell,expected,actual}`; `differ::run_axis_reduce(actual: &[f32], expected: &[f32], monoid: Monoid, tol: &[f32]) -> Vec<AxisDivergence>`; `corpus::AxisVector{data,extents,axis,provenance}`; `corpus::tagged_axis_corpus(seed: u64) -> Vec<AxisVector>`.

- [ ] **Step 1: Write the failing differ test** (add to differ.rs's test module):
```rust
#[test]
fn axis_reduce_catches_out_of_band_and_wrong_max() {
    // Sum: within band accepted, outside caught.
    let exp = [6.0f32, 15.0];
    let tol = [1e-3f32, 1e-3];
    assert!(run_axis_reduce(&[6.0, 15.0], &exp, crate::structural::Monoid::Sum, &tol).is_empty());
    let d = run_axis_reduce(&[6.0, 15.5], &exp, crate::structural::Monoid::Sum, &tol);
    assert_eq!(d.len(), 1); assert_eq!(d[0].cell, 1);
    // Max: exact-byte — a 1-ULP diff is caught even with a nonzero tol array.
    let d2 = run_axis_reduce(&[3.0, f32::from_bits(6.0f32.to_bits() + 1)], &[3.0, 6.0], crate::structural::Monoid::Max, &tol);
    assert_eq!(d2.len(), 1);
}
```
- [ ] **Step 2: Run it, verify it fails.** → FAIL.
- [ ] **Step 3: Implement differ** — add to `differ.rs`:
```rust
use crate::structural::{compare_monoid_reduced_f32, Monoid};

/// One caught divergence in an array-valued (axis) reduction, at output `cell`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisDivergence {
    pub cell: usize,
    pub expected: f32,
    pub actual: f32,
}

/// Difference a candidate axis-reduction's `actual` cells against the oracle
/// `expected`, dispatching the comparator BY MONOID (Sum → order-invariant band
/// using `tol[i]`; Max → ±0-canon exact-byte, ignoring `tol`). This per-monoid
/// dispatch is the structural comparator selection 3b makes Contract-sourced.
pub fn run_axis_reduce(actual: &[f32], expected: &[f32], monoid: Monoid, tol: &[f32]) -> Vec<AxisDivergence> {
    assert_eq!(actual.len(), expected.len(), "one actual per expected cell");
    assert_eq!(tol.len(), expected.len(), "one tolerance per cell");
    let mut out = Vec::new();
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        if compare_monoid_reduced_f32(monoid, a, e, tol[i], 0.0).is_err() {
            out.push(AxisDivergence { cell: i, expected: e, actual: a });
        }
    }
    out
}
```
- [ ] **Step 4: Run it, verify pass.** → PASS.
- [ ] **Step 5: Write the failing corpus test** (add to corpus.rs's test module):
```rust
#[test]
fn axis_corpus_is_tagged_and_shaped() {
    let c = tagged_axis_corpus(0xA715);
    assert!(c.len() >= 4);
    for v in &c {
        assert!(!v.provenance.is_empty(), "axis vector missing provenance (§6.5-0003)");
        assert_eq!(v.data.len(), v.extents[0] * v.extents[1]);
        assert_eq!(v.axis, 1, "3a corpus is trailing-axis");
    }
}
```
- [ ] **Step 6: Run it, verify it fails.** → FAIL.
- [ ] **Step 7: Implement corpus** — add to `corpus.rs`:
```rust
/// One rank-2 axis-reduction input, provenance-tagged (§6.5-0003).
#[derive(Debug, Clone)]
pub struct AxisVector {
    pub data: Vec<f32>,
    pub extents: [usize; 2],
    pub axis: usize,
    pub provenance: &'static str,
}

const AXIS_PROVENANCE: &str = "oracle:KISS-OPS-6.11-0002/structural::reduce_axis2_f32";

/// A deterministic trailing-axis (axis 1) corpus of rank-2 shapes that keep the
/// two summation orders inside the band and a wrong axis / wrong value outside it:
/// a wide row, a tall column, a small square, and a large-magnitude-spread row.
pub fn tagged_axis_corpus(_seed: u64) -> Vec<AxisVector> {
    let mk = |data: Vec<f32>, extents: [usize; 2]| AxisVector { data, extents, axis: 1, provenance: AXIS_PROVENANCE };
    vec![
        mk((1..=24).map(|i| i as f32).collect(), [4, 6]),
        mk((1..=8).map(|i| i as f32).collect(), [8, 1]),
        mk(vec![0.5; 100], [10, 10]),
        mk({ let mut v = vec![1e6f32]; v.extend([1.0; 7]); v }, [2, 4]), // large-spread rows
    ]
}
```
- [ ] **Step 8: Run it, verify pass.** → PASS.
- [ ] **Step 9: Commit** — `git add conformance/src/harness/differ.rs conformance/src/harness/corpus.rs`; commit: `harness: array-valued axis differ (monoid-dispatched) + provenance-tagged axis corpus`.

---

### Task 5: C fixtures + loader helper + the freeze-gate test + ledger
**Files:** Create 6 `conformance/tests/harness_fixtures/reduce_axis1_{sum,max}_{a,b,wrong}.c`; modify `conformance/tests/common/mod.rs`; create `conformance/tests/harness_reduce_axis_differential.rs`; modify `conformance/UNBACKED.tsv`.
**Interfaces:** Consumes all prior tasks. Produces `common::compile_and_load_axis_reduce(name) -> Option<AxisReduceKernel>`.

- [ ] **Step 1: Write the 6 C fixtures.** All export `kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n)`. `ein=[rows,cols]`.
  `reduce_axis1_sum_a.c` (correct, axis 1, sequential per row):
```c
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) { float s = 0.0f; for (long long c = 0; c < cols; ++c) s += in[r*cols+c]; out[r] = s; }
}
```
  `reduce_axis1_sum_b.c` (correct, axis 1, pairwise per row — different reassociation):
```c
static float pw(const float* a, long long n){ if(n==1) return a[0]; long long h=n/2; return pw(a,h)+pw(a+h,n-h); }
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) out[r] = pw(in + r*cols, cols);
}
```
  `reduce_axis1_sum_wrong.c` (WRONG — reduces axis 0 instead of axis 1):
```c
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long c = 0; c < cols; ++c) { float s=0.0f; for (long long r=0;r<rows;++r) s+=in[r*cols+c]; out[c]=s; }
}
```
  `reduce_axis1_max_a.c` (correct max, axis 1):
```c
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r=0;r<rows;++r){ float m=in[r*cols]; for(long long c=1;c<cols;++c){ float x=in[r*cols+c]; if(x>m) m=x; } out[r]=m; }
}
```
  `reduce_axis1_max_b.c` (correct max, axis 1, reverse scan — same result, different order):
```c
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r=0;r<rows;++r){ float m=in[r*cols+cols-1]; for(long long c=cols-2;c>=0;--c){ float x=in[r*cols+c]; if(x>m) m=x; } out[r]=m; }
}
```
  `reduce_axis1_max_wrong.c` (WRONG — computes the MIN):
```c
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r=0;r<rows;++r){ float m=in[r*cols]; for(long long c=1;c<cols;++c){ float x=in[r*cols+c]; if(x<m) m=x; } out[r]=m; }
}
```

- [ ] **Step 2: Add the loader helper** — append to `conformance/tests/common/mod.rs`:
```rust
use kiss_conformance::harness::abi::AxisReduceKernel;

/// Compile a fixture and resolve `kiss_reduce_axis1` as an `AxisReduceKernel`.
pub fn compile_and_load_axis_reduce(name: &str) -> Option<AxisReduceKernel> {
    let m = msvc::find_msvc()?;
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/harness_fixtures").join(format!("{name}.c"));
    let out = std::env::temp_dir().join(format!("kiss_harness_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let dll = msvc::compile_c_to_dll(&m, &src, &out).expect("compile fixture");
    let art = Box::leak(Box::new(Artifact::load(&dll).expect("load fixture")));
    let sym = art.symbol("kiss_reduce_axis1").expect("kiss_reduce_axis1 export");
    // SAFETY: every axis fixture exports exactly the AxisReduceKernel C signature.
    Some(unsafe { std::mem::transmute::<*const core::ffi::c_void, AxisReduceKernel>(sym) })
}
```

- [ ] **Step 3: Write the freeze-gate test** — create `conformance/tests/harness_reduce_axis_differential.rs`:
```rust
#![cfg(windows)]
//! KISS-OPS-6.11-0002 (axis reduction) via the differential harness: two dissimilar
//! C `reduce(monoid, axis=1)` kernels agree with the resolved oracle per output row
//! — within the reassociation band for Sum, bit-exact (±0-canon) for Max — while a
//! WRONG-AXIS kernel (reduces axis 0) and a wrong-value kernel are caught. The
//! comparator is dispatched BY MONOID (Sum→order-invariant band, Max→exact-byte);
//! this is the multi-class FOUNDATION for increment 3b's Contract-sourced selection,
//! which is what binds §6.13-0006b — 3a does NOT claim 0006b (the comparator is still
//! chosen structurally by monoid, not read from an advertised Contract class).

mod common;
use common::compile_and_load_axis_reduce;
use kiss_conformance::harness::abi::invoke_axis_reduce;
use kiss_conformance::harness::corpus::tagged_axis_corpus;
use kiss_conformance::harness::differ::run_axis_reduce;
use kiss_conformance::harness::resolver::{reduce_axis_abs_tol, reduce_axis_oracle};
use kiss_conformance::structural::Monoid;

#[test]
fn test_ops_reduce_axis_differential() {
    let Some(sum_a) = compile_and_load_axis_reduce("reduce_axis1_sum_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C axis-reduce slice needs cl.exe");
        return;
    };
    let sum_b = compile_and_load_axis_reduce("reduce_axis1_sum_b").unwrap();
    let sum_wrong = compile_and_load_axis_reduce("reduce_axis1_sum_wrong").unwrap();
    let max_a = compile_and_load_axis_reduce("reduce_axis1_max_a").unwrap();
    let max_b = compile_and_load_axis_reduce("reduce_axis1_max_b").unwrap();
    let max_wrong = compile_and_load_axis_reduce("reduce_axis1_max_wrong").unwrap();

    for v in tagged_axis_corpus(0x5EED) {
        let ein = [v.extents[0] as i64, v.extents[1] as i64];
        let eout = [v.extents[0] as i64, 1]; // axis-1 keepdim: out is rows×1
        let sum_oracle = reduce_axis_oracle(&v.data, v.extents, 1, Monoid::Sum);
        let sum_tol = reduce_axis_abs_tol(&v.data, v.extents, 1);
        let max_oracle = reduce_axis_oracle(&v.data, v.extents, 1, Monoid::Max);
        let zero_tol = vec![0.0f32; max_oracle.len()];

        let a = invoke_axis_reduce(sum_a, &v.data, ein, eout);
        let b = invoke_axis_reduce(sum_b, &v.data, ein, eout);
        assert!(run_axis_reduce(&a, &sum_oracle, Monoid::Sum, &sum_tol).is_empty(), "Sum mean_a diverged: {:?}", v.extents);
        assert!(run_axis_reduce(&b, &sum_oracle, Monoid::Sum, &sum_tol).is_empty(), "Sum mean_b diverged: {:?}", v.extents);
        // wrong-axis: reducing axis 0 gives a different-length / different-valued output → caught.
        let w = invoke_axis_reduce(sum_wrong, &v.data, ein, [1, v.extents[1] as i64]); // wrong kernel writes cols outputs
        // compare only the overlapping prefix; a length/shape mismatch is itself a divergence.
        assert!(w.len() != sum_oracle.len() || !run_axis_reduce(&w[..sum_oracle.len().min(w.len())], &sum_oracle[..sum_oracle.len().min(w.len())], Monoid::Sum, &sum_tol[..sum_oracle.len().min(w.len())]).is_empty(),
            "the band wrongly ABSORBED a wrong-axis reduction: {:?}", v.extents);

        let ma = invoke_axis_reduce(max_a, &v.data, ein, eout);
        let mb = invoke_axis_reduce(max_b, &v.data, ein, eout);
        assert!(run_axis_reduce(&ma, &max_oracle, Monoid::Max, &zero_tol).is_empty(), "Max a diverged: {:?}", v.extents);
        assert!(run_axis_reduce(&mb, &max_oracle, Monoid::Max, &zero_tol).is_empty(), "Max b diverged: {:?}", v.extents);
        let mw = invoke_axis_reduce(max_wrong, &v.data, ein, eout);
        assert!(!run_axis_reduce(&mw, &max_oracle, Monoid::Max, &zero_tol).is_empty(), "wrong Max (min) not caught: {:?}", v.extents);
    }
}
```
*Note:* if the wrong-axis assertion is awkward for a square shape (rows==cols, same output length), the `w.len() != oracle.len()` guard still catches value divergence via the prefix compare; the corpus includes non-square shapes so the length mismatch fires there.

- [ ] **Step 4: Run the freeze-gate test** — `cargo test -p kiss-conformance --test harness_reduce_axis_differential -- --nocapture 2>&1 | tail -15` → `test_ops_reduce_axis_differential ... ok`, NOT SKIP. If a wrong assertion fails to catch on some shape, adjust that corpus shape (do NOT loosen the comparator).
- [ ] **Step 5: Whole-suite + Linux** — `cargo test 2>&1 | grep -E 'test result|error|warning:' | tail`; `cargo check -p kiss-conformance --target x86_64-unknown-linux-gnu --tests 2>&1 | tail -1` (clean).
- [ ] **Step 6: Ledger** — `python tools/kiss_trace.py --update-ledger`. Backing: this test's fn `test_ops_reduce_axis_differential` is NOT a spec `*Test:*` tag (it's a harness capability test), so it may bind nothing automatically — check `git diff conformance/UNBACKED.tsv`. If 6.11-0002's `*Test:*` tag names this fn (verify in ops.md §9), it binds; otherwise cite `KISS-OPS-6.11-0002` in the test body to bind it, and mark 6.11-0008 partial per the design §8. Run all 5 lint tools → exit 0. **Honesty check:** only claim clauses whose obligation this test genuinely exercises (axis form = 6.11-0002); do NOT claim 6.13-0006b.
- [ ] **Step 7: Commit** — `git add conformance/tests/harness_fixtures/reduce_axis1_*.c conformance/tests/common/mod.rs conformance/tests/harness_reduce_axis_differential.rs conformance/UNBACKED.tsv`; commit: `harness: axis-reduce freeze-gate — Sum+Max, dissimilar orders agree, wrong axis/min caught (6.11-0002)`.

---

## Self-Review
**Spec coverage:** reduce_axis2_f32 (T1), resolver oracle+band (T2), abi marshaller (T3), differ+corpus (T4), fixtures+test+ledger (T5) — matches design §3. Teeth: dissimilar-order agreement + wrong-axis + wrong-value + monoid-dispatched comparator (T5 = design §5). Honest scope: backs 6.11-0002 (+ 6.11-0008 partial), NOT 0006b (design §0). ✓
**Placeholders:** none; every code step complete. ✓
**Type consistency:** `AxisReduceKernel` (T3) consumed T5; `AxisVector`/`tagged_axis_corpus` (T4) consumed T5; `AxisDivergence`/`run_axis_reduce(monoid-dispatched)` (T4) consumed T5; `reduce_axis_oracle`/`reduce_axis_abs_tol` (T2) consumed T5; `reduce_axis2_f32` (T1) consumed T2. `compare_monoid_reduced_f32(monoid, actual, expected, abs_tol, rel_tol)` matches structural.rs. ✓
*Implementer note:* the wrong-axis catch relies on a shape where reducing axis 0 yields a different-length or different-valued output than axis 1 — the corpus's non-square shapes ([4,6],[8,1],[2,4]) guarantee this; keep at least one strongly-non-square shape.
