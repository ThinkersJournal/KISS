# Harness Increment 2 — Decomposition-Resolver + Reduction Differential Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Difference a non-primitive op (`reduce_mean`) by resolving its reference decomposition `div(reduce(sum,x), count)` to the primitive floor and comparing under the reassociation-band tolerance — backing `KISS-CONFORM-6.5-0004` and `6.13-0006a`.

**Architecture:** Adds a `harness::resolver` module (decomposition-to-floor oracle + tolerance) and a reduction marshaller to `harness::abi`, reusing increment 1's §6.5 FFI harness and `structural::{reduce_f32, reassoc_bound_f32, compare_reduced_f32}` (the #92 tolerance machinery). Two dissimilar C `reduce_mean` kernels agree with the resolved oracle within the band; a wrong divisor is caught.

**Tech Stack:** Rust (edition 2021, stdlib only), raw Win32 FFI (from inc 1), MSVC `cl.exe`, C fixtures.

## Global Constraints

- **Stdlib only** — no crate deps; no `Cargo.toml` change; no `build.rs`.
- **All `unsafe`** confined to `loader` + the single call-through-fn-pointer site in `abi`, each with a `// SAFETY:` note.
- **`#[cfg(windows)]`** gating: any module/test that pulls in `loader` (the C-fixture path) is Windows-gated so the ubuntu CI leg still compiles. The `resolver` unit test is pure Rust (cross-platform, runs on both legs).
- Edition 2021; crate name `kiss_conformance`. Verified reuse signatures: `structural::reduce_f32(xs: &[f32], monoid: Monoid) -> f32`; `structural::Monoid::{Sum,Prod,Max,Min}`; `structural::reassoc_bound_f32(n_addends: usize, abs_sum: f32) -> f32` (returns `(n-1)·(EPSILON/2)·|abs_sum|`, 0 for n<3); `structural::compare_reduced_f32(class: DeterminismClass, actual, expected, abs_tol, rel_tol) -> Result<(),String>`; `DeterminismClass::OrderInvariant` (at crate root — `kiss_conformance::DeterminismClass`, confirm import path).
- **Commit trailers** on every commit:
  ```
  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
  ```
- **Worktree:** all work in `C:/Projects/kiss-harness2` on `feat/harness-reduce-mean`; run `cargo` from `C:/Projects/kiss-harness2/conformance`.

---

## File Structure

| File | Responsibility |
|---|---|
| `conformance/src/harness/resolver.rs` | Create: `reduce_mean_oracle` (decomposition→floor) + `reduce_mean_abs_tol` (reassoc band ÷ n). |
| `conformance/src/harness/mod.rs` | Modify: add `pub mod resolver;`. |
| `conformance/src/harness/abi.rs` | Modify: add `ReduceKernel` type + `invoke_reduce`. |
| `conformance/tests/harness_fixtures/mean_a.c` / `mean_b.c` / `mean_wrong.c` | Create: two correct (different summation orders) + one wrong (÷ n−1). |
| `conformance/tests/common/mod.rs` | Modify: add `compile_and_load_reduce(name) -> Option<ReduceKernel>`. |
| `conformance/tests/harness_reduce_differential.rs` | Create: the 6.13-0006a freeze-gate test (`#![cfg(windows)]`). |

---

### Task 1: `resolver` — decomposition-to-floor oracle + tolerance (+ backs 6.5-0004)

**Files:**
- Create: `conformance/src/harness/resolver.rs`
- Modify: `conformance/src/harness/mod.rs` (add `pub mod resolver;`)
- Test: inline `#[cfg(test)] mod tests` in `resolver.rs`

**Interfaces:**
- Consumes: `crate::structural::{reduce_f32, reassoc_bound_f32, Monoid}`.
- Produces: `harness::resolver::reduce_mean_oracle(xs: &[f32]) -> f32`; `harness::resolver::reduce_mean_abs_tol(xs: &[f32]) -> f32`.

- [ ] **Step 1: Write the failing test** (named per §6.5-0004's `*Test:*` tag so it binds that clause)

Create `conformance/src/harness/resolver.rs` with ONLY this test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // KISS-CONFORM-6.5-0004: the oracle for the non-primitive `reduce_mean` MUST be
    // its reference decomposition div(reduce(sum,x), count) evaluated at the floor —
    // NOT a monolithic mean. Teeth: on an input where a running-average mean
    // (m += (x-m)/(i+1)) diverges from sum/n, the resolver matches sum/n, and it is
    // built from the floor `reduce_f32(_, Sum)` + a divide (a wrong monoid or wrong
    // divisor would change the value on the checked input).
    #[test]
    fn test_conform_oracle_resolves_to_floor() {
        let xs = [1.0f32, 2.0, 3.0, 4.0];
        // decomposition value: sum then divide.
        let expected = crate::structural::reduce_f32(&xs, crate::structural::Monoid::Sum) / xs.len() as f32;
        assert_eq!(reduce_mean_oracle(&xs), expected);

        // A running-average mean diverges from sum/n in FP; the resolver must be the
        // sum/n decomposition, not that alternative.
        let mut running = 0.0f32;
        for (i, &x) in xs.iter().enumerate() {
            running += (x - running) / (i as f32 + 1.0);
        }
        let big = [1e7f32, 1.0, 1.0, 1.0, 1.0]; // sum/n and running-avg differ here
        let mut r2 = 0.0f32;
        for (i, &x) in big.iter().enumerate() { r2 += (x - r2) / (i as f32 + 1.0); }
        let dec = crate::structural::reduce_f32(&big, crate::structural::Monoid::Sum) / big.len() as f32;
        assert_eq!(reduce_mean_oracle(&big), dec);
        assert_ne!(reduce_mean_oracle(&big).to_bits(), r2.to_bits(),
            "the resolver must be the sum/n decomposition, distinguishable from a running-average mean");
        let _ = running;
    }

    #[test]
    fn abs_tol_is_the_reassociation_band_over_n() {
        let xs = [1.0f32, -2.0, 3.0, -4.0, 5.0];
        let sum_abs = xs.iter().map(|x| x.abs()).sum::<f32>();
        let expected = crate::structural::reassoc_bound_f32(xs.len(), sum_abs) / xs.len() as f32;
        assert_eq!(reduce_mean_abs_tol(&xs), expected);
    }
}
```

- [ ] **Step 2: Run it, verify it fails** — `cargo test -p kiss-conformance --lib harness::resolver 2>&1 | tail -15` → FAIL (`cannot find function reduce_mean_oracle`).

- [ ] **Step 3: Implement** — prepend to `resolver.rs` (and add `pub mod resolver;` to `mod.rs`, after `pub mod msvc;`):
```rust
//! Decomposition-resolver: evaluates a non-primitive op's KISS-Ops reference
//! decomposition down to the primitive floor (KISS-CONFORM-6.5-0004). Increment 2
//! is focused on `reduce_mean = div(reduce(sum, x), reduced_count)` (ops.md §6.13).

use crate::structural::{reassoc_bound_f32, reduce_f32, Monoid};

/// The `reduce_mean` oracle as its reference decomposition, evaluated at the floor:
/// `div(reduce(sum, x), count)`. It composes the floor `reduce_f32(_, Sum)` (a fold
/// of the `add` primitive) with a divide — it never terminates above the floor and
/// is not a monolithic mean.
pub fn reduce_mean_oracle(xs: &[f32]) -> f32 {
    reduce_f32(xs, Monoid::Sum) / xs.len() as f32
}

/// The absolute tolerance for comparing a candidate `reduce_mean` against the oracle:
/// the reassociation band of the interior sum (`reassoc_bound_f32(n, Σ|x|)`) divided
/// by `count`, since the mean divides the sum by `n`. An absolute band (§6.8-0004 /
/// the #92 accumulator-tolerance model), not a ULP count.
pub fn reduce_mean_abs_tol(xs: &[f32]) -> f32 {
    let sum_abs = xs.iter().map(|x| x.abs()).sum::<f32>();
    reassoc_bound_f32(xs.len(), sum_abs) / xs.len() as f32
}
```

- [ ] **Step 4: Run tests, verify pass** — `cargo test -p kiss-conformance --lib harness::resolver 2>&1 | tail -8` → both PASS. (This binds `KISS-CONFORM-6.5-0004` via the `test_conform_oracle_resolves_to_floor` fn name; confirm with `python tools/kiss_trace.py --report 2>&1 | grep 6.5-0004` if desired.)

- [ ] **Step 5: Commit** — `git add conformance/src/harness/resolver.rs conformance/src/harness/mod.rs` then commit: `harness: reduce_mean decomposition-resolver + reassoc tolerance (backs 6.5-0004)`.

---

### Task 2: `abi` reduction marshaller

**Files:**
- Modify: `conformance/src/harness/abi.rs` (add below the existing `invoke_binary`)
- Test: extend `abi.rs`'s `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `harness::abi::ReduceKernel` (`unsafe extern "C" fn(*const f32, *mut f32, i64)`); `harness::abi::invoke_reduce(kernel: ReduceKernel, xs: &[f32]) -> f32`.

- [ ] **Step 1: Write the failing test** — add to `abi.rs`'s test module:
```rust
    unsafe extern "C" fn rust_reduce_mean(x: *const f32, o: *mut f32, n: i64) {
        let mut s = 0.0f32;
        for i in 0..n as isize {
            // SAFETY: the marshaller guarantees x has n readable f32.
            unsafe { s += *x.offset(i) };
        }
        // SAFETY: o points to one writable f32.
        unsafe { *o = s / n as f32 };
    }

    #[test]
    fn marshals_and_invokes_a_reduction() {
        let xs = [1.0f32, 2.0, 3.0, 4.0];
        assert_eq!(invoke_reduce(rust_reduce_mean, &xs), 2.5);
    }
```

- [ ] **Step 2: Run it, verify it fails** — `cargo test -p kiss-conformance --lib harness::abi 2>&1 | tail -15` → FAIL (`cannot find function invoke_reduce`).

- [ ] **Step 3: Implement** — append to `abi.rs` (after `invoke_binary`):
```rust
/// A §6.5 rank-1 full-reduction kernel entry point (C ABI): reads `n` inputs,
/// writes one scalar output.
pub type ReduceKernel = unsafe extern "C" fn(*const f32, *mut f32, i64);

/// Invoke a reduction `kernel` over `xs`; returns the scalar output. `xs` must be
/// non-empty (a mean over zero elements is undefined — the caller's contract).
pub fn invoke_reduce(kernel: ReduceKernel, xs: &[f32]) -> f32 {
    assert!(!xs.is_empty(), "reduce over an empty slice is undefined");
    let mut out = 0.0f32;
    // SAFETY: `xs` exposes `xs.len()` readable f32; `out` is one writable f32; the
    // pointers are non-overlapping (distinct allocations); len fits i64. The kernel
    // writes exactly `out`.
    unsafe { kernel(xs.as_ptr(), &mut out as *mut f32, xs.len() as i64) };
    out
}
```

- [ ] **Step 4: Run tests, verify pass** — `cargo test -p kiss-conformance --lib harness::abi 2>&1 | tail -8` → PASS.

- [ ] **Step 5: Commit** — `git add conformance/src/harness/abi.rs` then commit: `harness: §6.5 reduction marshaller (invoke_reduce)`.

---

### Task 3: C `reduce_mean` fixtures + reduce loader helper + smoke test

**Files:**
- Create: `conformance/tests/harness_fixtures/mean_a.c`, `mean_b.c`, `mean_wrong.c`
- Modify: `conformance/tests/common/mod.rs` (add `compile_and_load_reduce`)
- Create: `conformance/tests/harness_reduce_smoke.rs` (`#![cfg(windows)]`)

**Interfaces:**
- Consumes: `msvc`, `Artifact`, `harness::abi::{ReduceKernel, invoke_reduce}`.
- Produces: `common::compile_and_load_reduce(name: &str) -> Option<ReduceKernel>`.

- [ ] **Step 1: Write the three fixtures**

`mean_a.c` (forward-order accumulation):
```c
/* reduce_mean over n elements, forward-order sum ÷ n. Entry per the §6.5 reduction ABI. */
__declspec(dllexport) void kiss_reduce_mean(const float* in, float* out, long long n) {
    float s = 0.0f;
    for (long long i = 0; i < n; ++i) s += in[i];
    out[0] = s / (float)n;
}
```
`mean_b.c` (pairwise/tree sum — a different reassociation, same value up to the band):
```c
/* reduce_mean via pairwise (tree) summation, then ÷ n. A DIFFERENT reassociation
   than mean_a — bit-different partial sums, within the reassociation band. */
static float pairwise(const float* a, long long n) {
    if (n == 1) return a[0];
    long long h = n / 2;
    return pairwise(a, h) + pairwise(a + h, n - h);
}
__declspec(dllexport) void kiss_reduce_mean(const float* in, float* out, long long n) {
    out[0] = pairwise(in, n) / (float)n;
}
```
`mean_wrong.c` (wrong divisor — a real error the band must not absorb):
```c
/* WRONG on purpose: divides by (n-1). The harness must CATCH this. */
__declspec(dllexport) void kiss_reduce_mean(const float* in, float* out, long long n) {
    float s = 0.0f;
    for (long long i = 0; i < n; ++i) s += in[i];
    out[0] = s / (float)(n - 1);
}
```

- [ ] **Step 2: Add the reduce loader helper** — append to `conformance/tests/common/mod.rs`:
```rust
use kiss_conformance::harness::abi::ReduceKernel;

/// Compile `tests/harness_fixtures/<name>.c` and resolve `kiss_reduce_mean` as a
/// `ReduceKernel`. `None` (skip) if no toolchain; leaks the `Artifact` so the fn
/// pointer stays valid.
pub fn compile_and_load_reduce(name: &str) -> Option<ReduceKernel> {
    let m = msvc::find_msvc()?;
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness_fixtures")
        .join(format!("{name}.c"));
    let out = std::env::temp_dir().join(format!("kiss_harness_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let dll = msvc::compile_c_to_dll(&m, &src, &out).expect("compile fixture");
    let art = Box::leak(Box::new(Artifact::load(&dll).expect("load fixture")));
    let sym = art.symbol("kiss_reduce_mean").expect("kiss_reduce_mean export");
    // SAFETY: every reduce fixture exports exactly the ReduceKernel C signature.
    Some(unsafe { std::mem::transmute::<*const core::ffi::c_void, ReduceKernel>(sym) })
}
```
*(Note: `compile_and_load` (the binary helper) already imports `BinaryKernel`/`Artifact`/`msvc`/`PathBuf`; keep those. Add only the `ReduceKernel` import + the new fn.)*

- [ ] **Step 3: Write the smoke test** — create `conformance/tests/harness_reduce_smoke.rs`:
```rust
#![cfg(windows)]
//! Each reduce fixture compiles, loads, and computes on a trivial input.
mod common;
use common::compile_and_load_reduce;
use kiss_conformance::harness::abi::invoke_reduce;

#[test]
fn reduce_fixtures_compile_load_and_run() {
    let Some(a) = compile_and_load_reduce("mean_a") else { eprintln!("SKIP: no MSVC"); return; };
    let b = compile_and_load_reduce("mean_b").unwrap();
    let w = compile_and_load_reduce("mean_wrong").unwrap();
    let xs = [2.0f32, 4.0, 6.0, 8.0]; // mean 5.0; wrong (÷3) = 6.666...
    assert_eq!(invoke_reduce(a, &xs), 5.0);
    assert_eq!(invoke_reduce(b, &xs), 5.0);
    assert!((invoke_reduce(w, &xs) - 20.0 / 3.0).abs() < 1e-4);
}
```

- [ ] **Step 4: Run the smoke test, verify pass** — `cargo test -p kiss-conformance --test harness_reduce_smoke -- --nocapture 2>&1 | tail -10` → PASS (must NOT print SKIP on this host; it actually compiles the fixtures).

- [ ] **Step 5: Commit** — `git add conformance/tests/harness_fixtures/mean_*.c conformance/tests/common/mod.rs conformance/tests/harness_reduce_smoke.rs` then commit: `harness: reduce_mean C fixtures (2 orders + wrong) + reduce loader + smoke test`.

---

### Task 4: The reduction freeze-gate test (backs 6.13-0006a) + ledger reconciliation

**Files:**
- Create: `conformance/tests/harness_reduce_differential.rs` (`#![cfg(windows)]`)
- Modify: `conformance/UNBACKED.tsv` (regenerated)

**Interfaces:**
- Consumes: `common::compile_and_load_reduce`, `harness::abi::invoke_reduce`, `harness::resolver::{reduce_mean_oracle, reduce_mean_abs_tol}`, `harness::corpus` (edge/random f32 via `crate::differential`), `kiss_conformance::{DeterminismClass, structural::compare_reduced_f32}`.

- [ ] **Step 1: Write the freeze-gate test** — create `conformance/tests/harness_reduce_differential.rs`:
```rust
#![cfg(windows)]
//! KISS-CONFORM-6.13-0006a (increment 2): the differential harness resolves the
//! non-primitive `reduce_mean` to its floor decomposition and proves ≥2 dissimilar
//! implementations agree ON THE DECOMPOSITION within the reassociation band, while a
//! wrong divisor is caught. The oracle is the resolved floor value (resolver.rs), the
//! comparator is the order-invariant/reassociation-band comparator (§6.0-0004), NOT
//! exact-byte — legitimate reassociation between the two summation orders is accepted.

mod common;
use common::compile_and_load_reduce;
use kiss_conformance::harness::abi::invoke_reduce;
use kiss_conformance::harness::resolver::{reduce_mean_abs_tol, reduce_mean_oracle};
use kiss_conformance::structural::compare_reduced_f32;
use kiss_conformance::DeterminismClass;

// A deterministic corpus of reduction inputs: a few fixed vectors that keep the two
// correct orders inside the band and the wrong divisor outside it.
fn inputs() -> Vec<Vec<f32>> {
    vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        vec![1e6, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],   // large-magnitude spread
        (0..64).map(|i| (i as f32 * 0.1) - 3.0).collect(), // 64 mixed-sign
        vec![0.5; 100],
    ]
}

#[test]
fn test_conform_ops_decomposition_agreement() {
    let Some(mean_a) = compile_and_load_reduce("mean_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C reduction slice needs cl.exe");
        return;
    };
    let mean_b = compile_and_load_reduce("mean_b").unwrap();
    let mean_wrong = compile_and_load_reduce("mean_wrong").unwrap();

    for xs in inputs() {
        let oracle = reduce_mean_oracle(&xs);
        let tol = reduce_mean_abs_tol(&xs);
        let a = invoke_reduce(mean_a, &xs);
        let b = invoke_reduce(mean_b, &xs);
        let w = invoke_reduce(mean_wrong, &xs);

        // (1) Both dissimilar orders agree with the resolved decomposition within the band.
        compare_reduced_f32(DeterminismClass::OrderInvariant, a, oracle, tol, 0.0)
            .expect("KISS-CONFORM-6.13-0006a: mean_a diverged beyond the reassociation band");
        compare_reduced_f32(DeterminismClass::OrderInvariant, b, oracle, tol, 0.0)
            .expect("KISS-CONFORM-6.13-0006a: mean_b diverged beyond the reassociation band");
        // (2) ...and with each other.
        compare_reduced_f32(DeterminismClass::OrderInvariant, a, b, tol, 0.0)
            .expect("KISS-CONFORM-6.13-0006a: the two dissimilar orders disagree beyond the band");

        // (3) TEETH: the wrong divisor is CAUGHT — outside the band. (Skip degenerate
        // n where ÷n and ÷(n-1) coincide within tol; all `inputs()` have n>=8 and a
        // nonzero mean, so the wrong result is well outside the band.)
        assert!(
            compare_reduced_f32(DeterminismClass::OrderInvariant, w, oracle, tol, 0.0).is_err(),
            "KISS-CONFORM-6.13-0006a: the band wrongly ABSORBED a ÷(n-1) error on {xs:?}"
        );
    }
}
```

- [ ] **Step 2: Run the freeze-gate test, verify pass** — `cargo test -p kiss-conformance --test harness_reduce_differential -- --nocapture 2>&1 | tail -12` → `test_conform_ops_decomposition_agreement ... ok`, NOT SKIP. If assert (3) fails on some input, the band absorbed the wrong divisor there — replace that input with a larger-`n`/larger-magnitude vector so the ÷(n−1) error clears the band (do NOT loosen the comparator).

- [ ] **Step 3: Whole-suite + Linux check** — `cargo test 2>&1 | grep -E 'test result|error|warning:' | tail`, then `cargo check -p kiss-conformance --target x86_64-unknown-linux-gnu --tests 2>&1 | tail -1` (must compile clean — the differential + smoke tests are `#[cfg(windows)]`, so on Linux only the `resolver`/`abi` unit tests compile).

- [ ] **Step 4: Ledger reconciliation** — `python tools/kiss_trace.py --update-ledger`; confirm `KISS-CONFORM-6.5-0004` (from Task 1's fn) and `KISS-CONFORM-6.13-0006a` (from this test's fn name = its `*Test:*` tag) are now BACKED (`git diff conformance/UNBACKED.tsv | grep '^-KISS-CONFORM'` shows both removed). Run all 5 lint tools (`kiss_trace kiss_tables kiss_vocab kiss_ops kiss_wire`) → all exit 0.

- [ ] **Step 5: Commit** — `git add conformance/tests/harness_reduce_differential.rs conformance/UNBACKED.tsv` then commit: `harness: reduce_mean freeze-gate differential — 2 orders agree in-band, wrong divisor caught (6.13-0006a)`.

---

## Self-Review

**1. Spec coverage** (against the design doc): resolver (Task 1 = §3 piece 1 + backs 6.5-0004), abi reduction (Task 2 = piece 2), differ via `compare_reduced_f32` (Task 4 = piece 3), fixtures (Task 3 = §5), teeth — in-band acceptance + wrong-divisor catch + resolve-to-floor (Tasks 1,4 = §7), clauses 6.5-0004 + 6.13-0006a (Tasks 1,4 = §8), determinism-class OrderInvariant (Task 4 = §9). ✓

**2. Placeholder scan:** no TBD/TODO; every code step has complete code; commands have expected output. ✓

**3. Type consistency:** `ReduceKernel = unsafe extern "C" fn(*const f32, *mut f32, i64)` defined in Task 2, consumed in Tasks 3+4. `reduce_mean_oracle`/`reduce_mean_abs_tol` defined Task 1, consumed Task 4. `compile_and_load_reduce` defined Task 3, consumed Tasks 3+4. `compare_reduced_f32(DeterminismClass::OrderInvariant, actual, expected, abs_tol, rel_tol)` matches the verified `structural.rs` signature. ✓

*Implementer note:* confirm the `DeterminismClass` import path (crate root `kiss_conformance::DeterminismClass` vs `kiss_conformance::determinism::DeterminismClass`) by reading `conformance/src/lib.rs`; the plan assumes the crate-root re-export.
