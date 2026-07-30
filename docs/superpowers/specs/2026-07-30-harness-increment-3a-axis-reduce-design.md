# Harness increment 3a — axis reduction (design)

- **Date:** 2026-07-30
- **Status:** approved (design-panel-grounded)
- **Branch:** `feat/harness-axis-reduce` (off `origin/main` @ 62dfe70 — has increments 1+2)
- **Backs:** `KISS-OPS-6.11-0002` (axis form) + `KISS-OPS-6.11-0011` (3 of 4 reduce-axes categories) + `KISS-OPS-6.11-0008` (**partial** — see §8) ; materially strengthens 6.13-0006/0006a (rank-1 → axis).

## 0. Where 3a sits (sequenced increment 3)

Increment 3 has three sub-increments, each its own branch/PR:
- **3a (this doc): axis reduction** over a rank-2 tensor, `reduce(monoid, axis)` — the KISS-Ops §6.11 *primitive* (no decomposition-resolver needed). Buildable now, no spec/cosign dependency. **The multi-monoid (Sum + Max) foundation the others build on.**
- **3b: Contract-sourced comparator selection (0006b, Option B — Eric's ruling).** The harness reads a per-candidate KISS-Contract `determinism_class` (§6.7-0004/§6.8-0003) and *selects* the §6.8 comparator from it — the literal §7.4-0001 "advertised by an implementation" reading. Builds on 3a's Sum(nondet)+Max(exact-byte) ops. Backs 6.13-0006b.
- **3c: per-output determinism class.** Land the append-only `KISS-OPS-6.0-0007` + a Conform §6.8 mirror (kiss-ref/Baracuda cosign in flight) + a 2-root-DAG unit test.

**Honest 3a scope:** 3a selects its comparators **structurally, per monoid** (Sum→order-invariant band, Max→±0-canon exact-byte). That is *not yet* Contract-sourced, so **3a does NOT claim 6.13-0006b** — it builds the multi-monoid ops that 3b's Contract-sourced selection then binds. Stated in the test doc-comment, matching increments 1/2's self-scoping.

## 1. Purpose

Generalize the harness's reduction differential from increment 2's rank-1 full reduction to a **rank-2 axis reduction** over two monoids. Two dissimilar C kernels (sequential vs pairwise per-row) agree within the per-row reassociation band; a **wrong-axis** kernel (reduces axis 0 instead of 1) and a wrong-value kernel are caught — a failure mode a rank-1-only harness *structurally cannot express*.

## 2. Locked decisions (panel-grounded)

| Decision | Choice | Why |
|---|---|---|
| Op | `reduce(monoid, axis)` — §6.11 **primitive** | No decomposition-resolver needed (unlike reduce_mean); pure flat→axis generalization |
| Monoids | **Sum** (order-invariant/nondet) **and Max** (exact-byte, ±0-canon) | Two *different* determinism classes → the multi-class foundation for 3b's 0006b |
| Shape | **rank-2, one fixed reduced axis per fixture, fully packed** | Minimal; exercises §6.5-0004a class 1 (extents) + class 3 (n); no strides/offsets/workspace (§6.5-0005) |
| Axis carrier | **baked into the compiled kernel, NOT a launch scalar** | Matches the norm_axis precedent (§6.19-0031: axis is a compile-time OpAttrs `u8`, never a runtime scalar) |
| Comparator | per-output-cell, monoid-dispatched via the existing `compare_monoid_reduced_f32` | Reuses #92 machinery; 3a selects structurally, 3b makes it Contract-sourced |

## 3. Reused vs new

**Reused:** inc 1/2's `harness::{msvc, loader, abi, corpus, differ}` + the §6.5 FFI; `structural::{reduce_f32, reassoc_bound_f32, compare_reduced_f32, compare_monoid_reduced_f32}` (Sum/Prod→order-invariant, Max/Min→±0-canon exact-byte — already dispatches by monoid). No new comparator code.

**New:**
1. `structural::reduce_axis2_f32(data: &[f32], extents: [usize;2], axis: usize, monoid: Monoid) -> Vec<f32>` — a row/column iterator that slices `data` and calls the *existing* `reduce_f32` per surviving coordinate. No new fold algorithm.
2. `harness::abi`: `pub type AxisReduceKernel = unsafe extern "C" fn(*const f32, *mut f32, *const i64, *const i64, i64)` + `invoke_axis_reduce(kernel, data, extents_in:[i64;2], extents_out:[i64;2]) -> Vec<f32>`. Marshals class-1 extents (in + out) + class-3 `n` (finally exercises §6.5-0004a class 1, which inc 1/2 never did).
3. `harness::resolver`: `reduce_axis_oracle(data, extents, axis, monoid)` (calls #1) + `reduce_axis_abs_tol(data, extents, axis)` — **per-row** `2 * reassoc_bound_f32(row_len, row_abs_sum)` (a row's own length/abs-sum bounds its own reassociation, not the whole tensor's; the 2× per inc-2's two-order rule).
4. `harness::differ`: `run_axis_reduce(actual: &[f32], expected: &[f32], tol: &[f32]) -> Vec<Divergence>` — array-valued (reuses the existing `Divergence` shape); worth a real fn since ≥2 monoid sub-cases share the loop (inc 2 inlined its scalar compare — 3a corrects that).
5. `harness::corpus`: `tagged_axis_corpus(seed, monoid) -> Vec<AxisVector>` — provenance-tagged per §6.5-0003 (following **increment 1's** discipline, not increment 2's inline-Vec shortcut). Fixed rank-2 shapes: 1×n, n×1, small square, large-magnitude-spread row, 64-wide mixed-sign.
6. Six C fixtures: `reduce_axis1_{sum,max}_{a,b,wrong}.c` (`kiss_reduce_axis1` entry). `_a` sequential per-row, `_b` pairwise per-row (dissimilar reassociation), `_wrong` reduces the **wrong axis** (axis 0). `common::compile_and_load_axis_reduce`.
7. `tests/harness_reduce_axis_differential.rs` (`#![cfg(windows)]`) — the freeze-gate test.

## 4. Data flow

```
tagged_axis_corpus(monoid) ──► for each AxisVector (data, extents, axis):
   resolver.reduce_axis_oracle(...) ──► expected: Vec<f32> (per-output cells)
   resolver.reduce_axis_abs_tol(...) ──► tol: Vec<f32> (per-row band; 0 for Max)
   loader.load(fixture) + abi.invoke_axis_reduce(...) ──► candidate: Vec<f32>
   differ.run_axis_reduce(candidate, expected, tol) [monoid-dispatched] ──► Vec<Divergence>
```

## 5. Testing — the honest teeth

For **each** monoid (Sum, Max):
1. **Dissimilar-order agreement:** `_a` and `_b` (sequential vs pairwise per row) agree with the oracle and each other — within the per-row band for Sum; bit-exact (±0-canon) for Max.
2. **Wrong-axis caught:** `_wrong` (reduces axis 0) produces output cells that diverge — the *new* failure mode (a rank-1 harness can't express a wrong axis).
3. **Wrong-value caught:** a perturbed cell is caught (Sum: outside the band; Max: any bit difference after ±0-canon).
4. **Comparator differs by monoid:** the Max case uses exact-byte where the Sum case uses the band — proven by showing a legitimate ±0/reassociation that Sum accepts would be *rejected* under Max's comparator and vice-versa (the multi-class foundation; 3b turns this into the Contract-sourced 0006b binding).

## 6. Error handling / cross-platform

Reuse inc 1/2's typed `HarnessError`; divergence is data; `unsafe` confined to `loader` + the `invoke_axis_reduce` call site (each `// SAFETY:`). The differential + smoke tests are `#![cfg(windows)]` (ubuntu leg compiles them out; the **windows-latest CI leg is the evidence**). `structural::reduce_axis2_f32` is pure Rust (cross-platform; runs on both legs).

## 7. Backs (honest)

- **6.11-0002** (axis form) — backed (previously only flat/rank-1 evidence).
- **6.11-0011** (reduce-axes categories) — **3 of 4**: all-axes (a free rank-2 fold-both vector = the existing flat path), trailing-axis (axis 1), subset (axis 0). The 4th ("none/not-a-reduction", 0x0000) is a decline case already DONE (`KISS-OPS-6.19-0038`) — correctly out of scope.
- **6.11-0008** — **PARTIAL** (see §8).
- Materially strengthens 6.13-0006/0006a (generalizes rank-1 evidence to the axis case). Does **NOT** claim 6.13-0006b (that's 3b, Contract-sourced).

## 8. Normative bookkeeping call for Eric (one)

Does a value-only differential test *back* `KISS-OPS-6.11-0008`'s keepdim/stride-0 **broadcast-view** claim, or only its retained-axis/correct-value consequence? 3a proves a candidate folds the correct axis into the correct output cell (via `extents_out`), but 6.11-0008's literal text is a stride-0 view consumed by a *downstream* op (its example is `sub(x, reduce(max,x))`), which 3a in isolation can't exercise. **Recommend marking 6.11-0008 `partial` in UNBACKED.tsv with this caveat**, not fully-backed. (Deferred to the ledger-reconciliation task; flagged here.)

## 9. Out of scope

- reduce_mean-over-axis (`div(reduce(sum,axis), extents[axis])`) — a ~3-line reuse; **increment 3b-adjacent** (exercises §6.19-0036 noncarrier axis resolution), kept out of 3a to keep the diff reviewable.
- Contract-sourced 0006b (3b); the per-output clause (3c); softmax (deferred pending #67's multi-node-DAG evaluator); the §6.5-0004a Contract-Interface launch-scalar schema (a separate, currently-zero-code gap the panel flagged — 3a's raw-fn-ptr signature informally follows the class order but does not validate a serialized Contract document).
