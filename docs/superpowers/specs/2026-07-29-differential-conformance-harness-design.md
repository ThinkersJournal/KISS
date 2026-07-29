# Differential-conformance harness — design (increment 1)

- **Date:** 2026-07-29
- **Status:** approved-shape, pending written-spec review
- **Branch:** `feat/differential-harness`
- **Tracking:** KISS #91 follow-on (post lint-backable burndown floor, 708→535). Targets KISS-CONFORM §6.5 / §6.13-0006.

## 1. Purpose

Build a **runnable differential-conformance tool**: it differences a KISS-contracted op **kernel** — delivered as a foreign compiled artifact and invoked live through the KISS-Contract §6.5 C-ABI — against an independent from-scratch CPU oracle. The tool realizes **KISS-CONFORM-6.13-0006**: *KISS-Conform owns the independent CPU-oracle differential harness that resolves an op's decomposition to the primitive floor and compares under its declared class, sharing no lowering module with any reference impl; ≥2 dissimilar impls agree on floor semantics.*

This is **capability-first**: the deliverable is the tool a genuine external implementor plugs into (the §5.3 external-implementor surface), not a count of backed clauses. Clause-backing is a byproduct, honestly scoped (§7).

Increment 1 proves the whole spine end-to-end on **one elementwise binary op (`add`)** with **two independent C artifacts** plus a deliberately-wrong third, exercising the ≥2-dissimilar-impls freeze-gate.

## 2. Locked decisions (with rationale)

| Decision | Choice | Why |
|---|---|---|
| Primary driver | Capability-first (runnable external-implementor tool) | Serves the real freeze path; a bespoke coverage lint would not |
| Candidate interface | **KISS-Contract §6.5 positional C-ABI**, reused wholesale | Dogfoods the standard (harness = a KISS consumer); a foreign artifact exposing §6.5 gives the "dissimilar impl / foreign reader" property for free |
| Execution model | **Foreign compiled artifact, invoked live** via raw platform FFI | Highest real-external-impl fidelity; §6.5-0002 guarantees "bind and launch with no out-of-band information" |
| Dependency posture | **Dependency-free runtime** (raw `LoadLibraryW`/`dlopen`, no loader crate) | KISS-Conform §6.5 harness stays dependency-free (see `differential.rs` note) |
| First-increment op | **One elementwise binary op (`add`)** | Narrowest vertical slice that still exercises buffer marshalling + launch scalars + diff |
| Increment shape | **Two-artifact freeze-gate slice** | Directly realizes 6.13-0006 (≥2 dissimilar impls agree), not just a spine |
| Artifact language | **Two C artifacts** (+ one deliberately-wrong C artifact) | Strongest foreign-reader fidelity, fully outside the reference language (Rust) |

**Feasibility (verified 2026-07-29):** MSVC `cl.exe` is installed (VS 18 Community, MSVC 14.51/14.52) and `vswhere.exe` is present, so the fixture C can be compiled at build time. No toolchain install required.

## 3. The §6.5 ABI — what is reused vs built

**Reused (exists in `conformance/src`):** the launch-scalar data model (`LaunchScalars` in `dispatch_expr.rs` already mirrors the §6.5-0004a 8 classes: extents, strides, `n`, offsets, gather/index, ws_ptr, ws_bytes, params); the dispatch/derivation evaluator (`dispatch_expr::eval`); the Interface schema; the scalar oracle (`semantics.rs`, `add` present); the deterministic corpus + `agree` comparator (`differential.rs`).

**Built (the crate has the *description* but no *invocation runtime*):**

1. **`abi`** — the §6.5 Interface descriptor as a Rust value (entry symbol, positional signature = operand pointers in KISS-Classify canonical order then output(s), then launch scalars in the pinned 8-class order) + the **marshaller** that lays a concrete invocation's buffers and scalars out in that exact pinned order.
2. **`loader`** — raw-platform-FFI artifact load + entry-symbol resolve (`#[cfg(windows)] LoadLibraryW`/`GetProcAddress`, `#[cfg(unix)] dlopen`/`dlsym`). **All `unsafe` is confined here** behind a safe wrapper returning a resolved entry function-pointer or a typed error.
3. **`differ`** — class-aware comparison: for `add` (a bit-stable elementwise float op) the NaN-relaxed exact-byte `agree`; the determinism class is sourced from the op's declared class (KISS-OPS §7.4-0001), not hardcoded.
4. **`runner` + verdict** — drives the corpus, invokes each artifact via `abi`+`loader`, differences vs the oracle, cross-checks the two correct artifacts agree, and emits a structured verdict (per-artifact pass/fail; each divergence reproducible by (seed, index)).

## 4. Components (each one purpose, testable in isolation)

```
corpus ──► runner ──► loader.load(artifact) ──► abi.marshal(inputs, launch_scalars) ──► invoke ──► candidate outputs
             │                                                                                          │
             └─► oracle(op) ──► expected outputs ───────────────────────────────► differ(class) ◄──────┘
                                                                                       │
                                                                              divergences ──► verdict
```

- **`oracle`** — from-scratch reference value for the op (reuse `semantics::add`). Increment 1 needs no decomposition (add is primitive-floor).
- **`corpus`** — deterministic edge + seeded-random f32 vectors, each carrying a **provenance tag** naming the source of its expected value (§6.5-0003 = "the oracle"). Extends `differential.rs`.
- **`abi`** — §6.5 Interface descriptor + pinned-order marshaller (§3.1).
- **`loader`** — raw-FFI artifact load/resolve; `unsafe` isolated (§3.2).
- **`differ`** — class-aware comparator (§3.3).
- **`runner`** — the driver + verdict (§3.4).
- **`fixtures`** (test material, not shipped in the harness lib) — two independent correct C `add` kernels + one deliberately-wrong C `add` kernel, each exposing the §6.5 entry-point ABI, compiled to shared libs at build time.

## 5. Error handling

- Typed `HarnessError` (`LoadFailed`, `SymbolMissing`, `AbiMismatch`, `OutputSizeMismatch`) — a bad artifact is a typed error, **never a panic**.
- A **divergence is data** (a `Divergence { seed, index, a, b, oracle, candidate }`), not a harness failure. The harness reports; the *test* asserts the correct artifacts have none and the wrong one has ≥1.
- Output buffers are sized from `extents`/`n` before the call; the marshaller refuses an invocation whose signature does not match the descriptor (no OOB write).
- `unsafe` exists only in `loader` and the single call-through-fn-pointer site, each with a documented safety contract.

## 6. Testing — the honest teeth

The harness's own tests are the proof it works:

1. **Agreement:** two *independent* correct C `add` artifacts → **0 divergences** vs the oracle and vs each other over the full corpus (edge cases + N seeded-random pairs).
2. **Catch:** a deliberately-wrong C `add` (`a−b`, or one that mishandles `−0.0`/NaN) → **caught** — ≥1 divergence, reproducible by (seed, index). *If the harness cannot catch the wrong kernel it is worthless; this is the mutation check.*
3. **Robustness:** a missing entry symbol / unloadable artifact → typed `HarnessError`, not a panic.

## 7. What increment 1 honestly backs (no over-claiming)

- **KISS-CONFORM-6.13-0006** — the differential harness + ≥2-dissimilar-impls-agree gate. **Core deliverable, real teeth** (catches a wrong impl; the two artifacts share no code with the oracle or each other).
- **KISS-CONFORM-6.5-0003** — every conformance vector carries a derivation-provenance tag. **Yes** (added to the corpus).
- **KISS-CONFORM-6.5-0002 / -0005** — oracle derived solely from Ops semantics, sharing no lowering with any reference impl; independence process-enforced. **Partial/structural** (the oracle is separate Rust; the artifacts are separate C).

**Explicitly NOT backed by increment 1** (later increments): 6.5-0004 (resolve a *non-primitive* op to the floor — needs a composite op like a reduction), 6.5-0010 (wide-precision transcendental oracle floor), and the foreign-*reader* freeze-gates of other sub-standards (6.13-0002/0004).

## 8. Out of scope / future increments

- **Decomposition resolver** (non-primitive op → primitive floor) — the next increment; unlocks 6.5-0004 and a reduction op.
- **Reductions / structured ops** — workspace (ws_ptr/ws_bytes), the accumulator determinism class, non-elementwise output shapes.
- **Transcendentals** — the wide-precision oracle floor (§6.5-0007/0010), ULP/split comparators.
- **Contract-document-driven invocation** — increment 1 constructs the §6.5 Interface descriptor in-tree for `add`; a later increment parses it from a real KISS Contract document.
- **Corpus provenance/independence process record** (§6.5-0005 full) and the golden-vector-supply §6.13 clauses.

## 9. Open implementation decisions (for the plan, not blockers)

- **Fixture compilation:** invoke `cl.exe` directly from `build.rs` (located via `vswhere`, env from the MSVC dir) to honor dependency-free, **or** accept the `cc` build-dependency for portable compiler detection. Lean: direct invocation (zero crate deps), with a documented `vswhere` probe.
- **Cross-platform FFI:** `#[cfg(windows)]` `LoadLibraryW`/`GetProcAddress` vs `#[cfg(unix)]` `dlopen`/`dlsym`, behind one `loader` trait. Increment 1 targets Windows (the dev/CI host); the unix arm is a thin parallel.
- **Where the code lives:** a `harness` module tree under `conformance/src/` reusing `semantics`/`differential`/`dispatch_expr`, vs a sibling crate. Lean: in-crate module (reuse is direct, no new crate boundary).
