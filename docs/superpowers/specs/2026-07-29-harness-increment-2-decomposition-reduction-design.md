# Differential harness increment 2 — decomposition-resolver + reduction differential (design)

- **Date:** 2026-07-29
- **Status:** approved
- **Branch:** `feat/harness-reduce-mean` (off `main` @ a87e7a5, which has increment 1)
- **Backs:** `KISS-CONFORM-6.5-0004` (resolve-to-floor) + `KISS-CONFORM-6.13-0006a` (≥2 dissimilar impls agree on decompositions)

## 1. Purpose

Extend the differential-conformance harness (increment 1) to difference a **non-primitive** op — `reduce_mean` — by **resolving its KISS-Ops reference decomposition down to the primitive floor** and comparing the fully-lowered result under the **reduction (order-invariant) tolerance**. This turns the harness from "one primitive elementwise op" into something that exercises non-primitive semantics and the reassociation-tolerance model, and it is a concrete testing surface for the #90 accumulator-divergence class.

## 2. Locked decisions

| Decision | Choice | Why |
|---|---|---|
| Target op | **`reduce_mean`** = `div(reduce(sum, x), reduced_count)` (ops.md:1178) | Minimal non-primitive with a clean floor decomposition (reduce + div); reuses the existing reduction oracle + tolerance |
| Resolver scope | **Focused** on `reduce_mean` | A general op-DAG recipe-evaluator is #67 territory / a later increment |
| Reduction shape | **Full 1-D reduction → scalar** | Minimal §6.5 ABI extension; keeps focus on the novel parts (resolver + reassociation-band differ) |
| Comparator | **reassociation-band** (`compare_reduced_f32` / `reassoc_bound_f32`), not exact-byte | The mean output is order-invariant/nondeterministic (§6.0-0004); reassociation is legitimate |

## 3. Reused vs new

**Reused (already in the crate):** increment 1's `harness::{msvc, loader, abi, corpus}` + the §6.5 C-ABI + raw-FFI; `structural::reduce_f32` (the sum oracle); `structural::reassoc_bound_f32` + `structural::compare_reduced_f32` (the reassociation-band comparator from #92); `semantics` div.

**New — three focused pieces:**
1. **`harness::resolver`** — evaluates `reduce_mean`'s reference decomposition `div(reduce(sum, x), count)` **to the floor**: it composes the floor ops (`reduce_f32(x, Monoid::Sum)` then divide by `count`), NOT a monolithic mean. This is the §6.5-0004 "resolve down to the floor, never terminate above it" obligation made executable. Focused on `reduce_mean`.
2. **`harness::abi` reduction signature** — `pub type ReduceKernel = unsafe extern "C" fn(*const f32, *mut f32, i64)` (input ptr, scalar-output ptr, `n`) + `invoke_reduce(kernel, xs: &[f32]) -> f32`. Minimal launch-scalar surface: `n` only (a rank-1 fully-packed reduction has no strides/axis/workspace).
3. **reduction differ path** — `run_reduce(candidate: f32, oracle: f32, n, abs_sum) -> Option<Divergence>` (or reuse `compare_reduced_f32`): the candidate agrees iff within `reassoc_bound_f32(n, abs_sum)` of the resolved oracle. NOT exact-byte.

## 4. Components + data flow

```
corpus (deterministic f32 vectors)
   │
   ├─► resolver.reduce_mean_oracle(xs)  =  div(reduce_f32(xs, Sum), xs.len())   ── the resolved floor value + |max partial sum| for the band
   │
   └─► for each candidate kernel: loader.load + abi.invoke_reduce(xs)  ── candidate mean
                                                                              │
       reassoc_bound_f32(n, abs_sum) ──► compare_reduced_f32(oracle, candidate) ─► agree? / Divergence
```

- **corpus** — reuse increment 1's deterministic edge + seeded-random f32 vectors, taken as reduction *inputs* (each test uses a vector, not a pair).
- **resolver** — the decomposition-to-floor evaluator (the new §6.5-0004 core).
- **abi** — the reduction marshaller (`invoke_reduce`).
- **differ** — the reassociation-band comparison.
- **fixtures** — three C `reduce_mean` kernels (§5).

## 5. Fixtures (three C kernels, `tests/harness_fixtures/`)

- `mean_a.c` — forward-order accumulation: `s=0; for i: s+=in[i]; out[0]=s/n;`
- `mean_b.c` — a **different reassociation**: pairwise/tree sum, then `/n`. Bit-different partial sums from `mean_a`, but within the reassociation band.
- `mean_wrong.c` — a real error the band does NOT absorb: divides by `n-1` (or drops the last element). Must be **caught**.

Entry symbol `kiss_reduce_mean`, signature `(const float* in, float* out, long long n)` — `out` is one f32.

## 6. Error handling

Reuse increment 1's typed `HarnessError` (Load/Compile/Symbol). A divergence is data (a `Divergence`), not an error. `unsafe` stays confined to `loader` + the single call-through-fn-pointer site (`invoke_reduce`), each with a `// SAFETY:` note. `#[cfg(windows)]` gating as in increment 1 (loader + the integration test).

## 7. Testing — the honest teeth

1. **Legitimate reassociation accepted:** `mean_a` and `mean_b` (different summation orders) both agree with the resolved oracle *and each other* within `reassoc_bound_f32(n, abs_sum)` — the differ does not false-positive on reassociation.
2. **Real error caught:** `mean_wrong` (÷ n−1) produces a divergence outside the band — the band does not swallow a wrong divisor. This is the load-bearing tooth; fixtures are chosen so the wrong result straddles *outside* the band while `mean_b` sits *inside* it.
3. **Resolves to the floor:** a unit test asserting the resolver's value equals `div(reduce_f32(xs, Sum), n)` composed from floor ops (not a monolithic mean) — backs 6.5-0004's "never terminate above the floor".

## 8. What increment 2 honestly backs

- **KISS-CONFORM-6.5-0004** — resolve a non-primitive op to the floor + compare the lowered result. **Backed** (the resolver + the decomposition test).
- **KISS-CONFORM-6.13-0006a** — ≥2 dissimilar impls agree on decompositions. **Backed** (the two dissimilar C kernels agree within the band; the wrong one caught).

Does **not** back: 6.13-0006b (the comparator is still chosen structurally by op, not from an advertised per-op-class lookup); axis/multi-output reductions; softmax/transcendental decompositions.

## 9. Determinism-class note

The mean output is classified **order-invariant/nondeterministic (§6.0-0004), per output**, bounded by the reassociation band — not a ULP count. This increment exercises that class concretely and is the same per-output classification the Lightbulb secondary-reduction thread surfaced; a future 6.13-0006b increment would drive the comparator *from* the advertised class rather than choosing it structurally.

## 10. Out of scope / future increments

- **General op-DAG recipe-evaluator** (resolve ANY non-primitive) — #67 recipe-grammar territory.
- **Axis / multi-axis reductions** (exercise the §6.5 extents/axis/reduced_count/workspace launch scalars).
- **`softmax`** (multi-node recipe + transcendental `exp` → ULP tolerance + the split path).
- **6.13-0006b** — comparator selection from the advertised per-op class (§7.4-0001); connects to #64 (`determinism.rs` typed-decline).

## 11. Open implementation decisions (for the plan)

- The reassociation-band constant: reuse `reassoc_bound_f32`'s existing `(n-1)·eps·|Σ|`-style bound verbatim; the plan pins the exact `abs_sum` fed to it (the running |partial sum| max, or `|Σ|xs|`, per what `reassoc_bound_f32` already expects — read its signature).
- `mean_b`'s reassociation must be genuinely different from `mean_a` yet provably within the band on the corpus; the plan picks a pairwise sum and a corpus that keeps it inside while `mean_wrong` lands outside.
- Whether the resolver lives in `harness::resolver` (new module) or extends `harness::abi` — lean: a new `resolver` module (one clear responsibility).
