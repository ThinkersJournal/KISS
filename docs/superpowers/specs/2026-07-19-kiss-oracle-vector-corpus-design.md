# KISS-Conform oracle-vector corpus + differential harness — design

**Date:** 2026-07-19
**Status:** Design — approved in brainstorming; pending written-spec review before implementation.
**Owner:** KISS maintainer (final say on all normative text).
**Clauses this makes real:** KISS-CONFORM-6.5-0008, 6.5-0009 (newly landed, currently `untested`), and the pre-existing but unbuilt 6.3-0003, 6.4-0001/0002/0003, 6.5-0001, 6.13-0007.

---

## 1. Context & motivation

KISS-Conform's "crown the corpus" architecture rests on a **frozen, language-neutral, wide-precision oracle-vector corpus** — the single authoritative numeric truth a foreign reader and every consumer test against. That corpus does not exist yet. Traced against the crate (2026-07-18):

- The "corpus" is inline Rust `assert_eq!`s in `conformance/tests/*_golden.rs` — not serialized, not language-neutral, un-consumable by a foreign reader.
- `conformance/src/semantics.rs` is an **f32-only** oracle; its transcendental references are inconsistent (`pow` widens to f64, `tanh_refined` uses hardware `f32::tanh`), and there is no wide-precision path.
- `conformance/UNBACKED.tsv` admits the corpus spine is `untested`: §6.4-0001/0002/0003 (golden byte-vectors), §6.5-0001 (oracle-differential harness), §6.3-0003 (self-contained bundle), §6.13-0007 (transcendental+split), plus the two clauses just added, §6.5-0008/0009.

Phase (b) builds that corpus and the harness that mints and verifies against it, satisfying §6.5-0008 (coverage completeness) and §6.5-0009 (inline wide-precision stored values) rather than leaving them as prose.

Two decisions were settled in brainstorming and are load-bearing:

- **Minter = dependency-free Rust with a hand-written extended-precision core** (not a shipped MPFR/mpmath dependency). Research established that **double-double is insufficient** — the hardest-to-round binary64 arguments sit within 2⁻¹¹³…2⁻¹²⁶ of a rounding midpoint, ~20 bits below double-double's ~106-bit floor — so the core is a **256-bit big-float** (escalating to 512/1024).
- **Truth is certified against three independent sources** at dev time (never shipped): Python **mpmath**, **MPFR** driven by CORE-MATH's hard-to-round inputs, and pinned **Lefèvre–Muller** published worst cases (the only source independent of both mpmath and MPFR).

## 2. Goals / non-goals

**Goals**

1. A serialized, versioned, language-neutral **JSON corpus** a foreign reader reproduces byte-for-byte with no inferred convention (§6.4-0002, §6.3-0003).
2. A dependency-free **Rust extended-precision oracle** that is ≤0.5 ULP of f64 truth on the frozen input set — the §6.5-0009 "wider than the compute dtype, round once" floor for f64 (§6.5-0007).
3. A **Rust minter** that emits the corpus from that oracle + the existing `semantics.rs` domain logic, each cell self-certifying (hardness margin + stabilizing precision recorded).
4. A **Rust reader / differential harness** that loads the corpus, drives the existing `lib.rs` comparators against an implementation-under-test, and enforces §6.5-0008 (coverage) and §6.5-0009 (inline value).
5. A **dev-time-only three-source validation gate** that certifies the Rust oracle before freeze. Ordinary Rust devs never run it; they consume a certified frozen corpus with zero Python and zero MPFR in their loop.
6. Full op/dtype/atom coverage as the destination, reached incrementally with the format frozen after the first slice.

**Non-goals**

- No shipped runtime dependency on Python, mpmath, MPFR, or any crate for the consuming harness (the conformance crate stays stdlib-only).
- No exhaustive (millions-of-cells) sweep stored in JSON — those stay in the compute-not-store differential loop (`differential.rs`).
- No new comparators — the existing `DeterminismClass`, `compare_f32`, `compare_c32_transcendental`, and exact-byte comparators are reused unchanged.
- No fused ops, optimizer, or scheduling (consumer concerns, out of scope for kiss-conform).

## 3. Architecture overview

Six components and the data flow between them:

```
 ops.md ──(extract)──► op_manifest.json ─────────────┐  (coverage source, §6.5-0008)
                                                      │
 semantics.rs (domain/non-transcendental) ─┐          ▼
                                            ├─► kiss_mint (Rust bin) ─► corpus/*.json ─► corpus reader ─► differential
 hp.rs 256-bit core (transcendentals) ──────┘          │  (frozen, §6.4/§6.5-0009)       (Rust, §6.5-0008/0009)   │
                                                       │                                                         ▼
                                                       │                                          implementation-under-test
                                                       ▼
                                     validate_corpus.py  (DEV-TIME ONLY, never shipped)
                                     mpmath + MPFR(gmpy2, CORE-MATH .wc) + Lefèvre–Muller anchors
                                     ── bit-for-bit agreement gate ⇒ corpus may freeze ──
```

- **(1) Bundle format** — the JSON schema the corpus is written in.
- **(2) Extended-precision core** (`conformance/src/hp.rs`) — the 256-bit big-float + transcendental atoms.
- **(3) Minter** (`conformance/src/bin/kiss_mint.rs`) — emits the corpus.
- **(4) Reader / differential harness** (`conformance/src/corpus.rs` + `conformance/tests/corpus_*.rs`) — consumes the corpus, drives comparators, wires §6.5-0008/0009.
- **(5) Validation gate** (`tools/validate_corpus.py`) — dev-time three-source certification.
- **(6) Coverage source** (`conformance/corpus/op_manifest.json`, generated from ops.md) — the enumeration §6.5-0008 checks against.

## 4. Component 1 — the bundle format

A **Wycheproof-shaped JSON** corpus under `conformance/corpus/`, one file per (sub-standard, op-family) slice, versioned by a `schema` field. Every float — input **and** expected — is pinned as its **raw IEEE-754 bit-pattern in KISS uppercase-hex** (space/`·`-separated, most-significant byte first, left to right), reusing `conformance/src/lib.rs::parse_hex`. Never decimal, never C99 `%a` hex-float — raw bits are the only encoding under which NaN payload, sNaN-vs-qNaN, and signed zero survive with no convention inference.

Envelope + cell schema (concrete):

```json
{
  "schema": "kiss-oracle-vectors-v1.json",
  "kiss_substandard": "OPS",
  "schema_version": 1,
  "spec_clause": "KISS-CONFORM-6.4-0002",
  "generator": "kiss_mint <version>",
  "number_of_vectors": 4,
  "byte_order": "hex is the value's bytes most-significant first, left to right (KISS-Ops Appendix E, §6.4-0002)",
  "hex_encoding": "uppercase hex bytes; ' ' and '·' are grouping marks; any non-hex char is ignored (lib.rs::parse_hex)",
  "ulp_metric": "integer totalOrder distance |totalOrder(a)-totalOrder(b)| (lib.rs::ulp_distance_f32); NOT a fractional ULP from the exact real",
  "rounding_default": "none — every cell states its own rounding",
  "dtypes": {
    "f32":  {"bits": 32, "bytes": 4, "encoding": "IEEE-754 binary32"},
    "f64":  {"bits": 64, "bytes": 8, "encoding": "IEEE-754 binary64"},
    "c32":  {"bits": 64, "bytes": 8, "encoding": "two binary32 lanes [re, im], re first"}
  },
  "notes": {
    "signed-zero": "sign of a zero result is pinned; +0 and -0 are distinct expected bytes; MUST NOT be compared under plain ULP",
    "branch-pi":   "±π component sign is pinned even under the ULP class (split comparator, §6.18-0017)"
  },
  "vectors": [
    {
      "tcId": 3, "op": "exp", "dtype": "f32", "rounding": "roundTiesToEven",
      "inputs":  [ {"role": "x", "dtype": "f32", "bits": "3F 80 00 00"} ],
      "expected": {"dtype": "f32", "bits": "40 2D F8 54"},
      "class": "ULP", "ulp_bound": 2, "provenance": "oracle",
      "tags": ["transcendental"],
      "certificate": {"hardness_margin_bits": 41, "stabilized_precision_bits": 256},
      "comment": "exp(1.0f); any result within 2 ULP is conformant"
    }
  ]
}
```

**Field contract.** `op` (KISS-Ops op name); `dtype` (keyed to the `dtypes` table — width/byte order never inferred); `inputs[]` (one object per operand, so multi-operand and packed ops need no lane convention); `expected` (single hex bit-pattern; complex/vector results are one contiguous `·`-grouped byte string); `class` ∈ {`exact-byte`, `ULP`, `order-invariant`} verbatim from the three `DeterminismClass` members; `ulp_bound` (integer; 0 for exact-byte, the declared ceiling otherwise); `provenance` ∈ {`oracle`, `promoted-differential`, `negative`} (never `reference-observed` — that is circular per §6.5-0003 and inadmissible); `rounding` (mandatory per cell — no global default); `tags[]` (edge labels drawn from `notes`); `certificate` (per §6.5-0009, the self-certifying hardness margin + stabilizing precision).

**Comparator selection (not a class).** The **split comparator** is NOT a fourth class (§6.8-0005 forbids a fourth enum member). A complex-transcendental cell (`carg`/`clog`/`csqrt`/`cexp`) carries its true `class` (`ULP`, per §6.18-0014); the reader then applies the §6.8-0008 precedence — an **op-named refinement** overrides the class default — to select `compare_c32_transcendental` for exactly those four ops, and the class-selected comparator for everything else. So the corpus stores the class; the comparator is derived by op name in the reader, never stored as a pseudo-class.

**Rules.**
- Frozen JSON holds the **curated edge-case corpus** (hundreds–thousands of cells). Exhaustive sweeps are *not* stored — they run in the compute-not-store differential loop (Component 4).
- Signed-zero and ±π-endpoint cells carry `exact-byte`, or `ULP` *with* the op-named split refinement (which pins the sign exactly), **never** plain `ULP` without that refinement (`ulp_distance(-0,+0)=1` would silently accept a wrong sign).
- The `ulp_metric` field pins KISS's integer-totalOrder metric so a foreign reader importing a glibc-style fractional-ULP bound cannot mis-judge.

## 5. Component 2 — the extended-precision core (`conformance/src/hp.rs`)

A hand-written, dependency-free **binary big-float**: fixed `[u64; 4]` (256-bit) significand + `i32` exponent + sign, with a compile-wired escalation to `[u64; 8]` (512) and `[u64; 16]` (1024). Binary, not decimal — rounding to binary64 makes the midpoint test exact.

**Why 256 bits.** Correct rounding to binary64 needs `53 + worst-run + guard ≈ 128` bits minimum (Lefèvre–Muller: worst hard-to-round runs are ~57–67 bits past the round bit). 256 clears that by ~130 bits, making the certificate airtight rather than empirical. Double-double (~106) physically cannot decide the hardest cases and is rejected.

**Evaluation discipline (per atom):** reduce → evaluate to ≥256 bits → **Ziv rounding test** (if the result interval straddles an f64 midpoint, recompute at 512, then 1024) → round **once** to f64 round-ties-to-even. Record the hardness margin (distance to nearest midpoint) and the precision at which rounding stabilized.

**Argument reduction (in-core):**
- Trig (`sin`/`cos`/`tan`): **Payne–Hanek** against a hard-coded **~1280-bit `2/π` table** (covers |x| up to 2¹⁰²³ + target + guard). Skipping this makes large-argument trig ~100% wrong, not 1 ULP.
- `exp`/`log`: range reduction against a **256-bit `ln2`** (k up to ~1024 ⇒ 11 bits + target + guard). A truncated `ln2` misrounds for large k.

**Per-atom notes (informing effort + the stress corpus):**
- `sqrt` — **use `f64::sqrt`** (already IEEE correctly-rounded); confirm with the core, don't rebuild.
- `atan` — benign (monotone, well-conditioned).
- `atan2` — a sign/branch problem; quadrant edges and signed-zero cases (`atan2(+0,-0)=π`, etc.) are **exact-match**, not tolerance; needs π to full precision.
- `pow(a,b)=exp(b·log a)` — **hardest** (composed error amplification); most likely to force 512/1024 escalation. Domain/special-values (`0^0=1`, negative-base integer-parity sign, ±0 rules) stay **exact-match table logic in `semantics.rs`**, outside the precision core; the core computes only the `a>0` magnitude.
- `tan` — Payne–Hanek + pole at odd·π/2 (needs enormous relative accuracy of the reduced argument).
- `sin` near k·π — catastrophic cancellation (true value ~ r, tiny) ⇒ full relative precision of the reduced argument.
- `lgamma` — reflection formula for x<0.5 (reintroduces `sin(πx)` cancellation near negative-integer poles) + Stirling asymptotic for large x (needs Bernoulli B₂…B₂₀ and `log(√2π)` to full 256 bits); separate exact `signgam`.
- `erf` — compute via `erfc=1-erf` in the tail (decays like `exp(-x²)/x`); needs `2/√π` to 256 bits; Maclaurin↔asymptotic transition is the nastiest region.
- Complex (`carg`/`clog`/`csqrt`/`cexp`) — metric is **≤0.5 ULP per component**; branch cuts and signed zeros are **exact-match** (C99 Annex G), matching `compare_c32_transcendental`. `carg=atan2(im,re)`; `clog=(0.5·log(re²+im²), atan2(im,re))` via a hypot/`log1p` form; `cexp=exp(re)·(cos im, sin im)` inheriting exp reduction + Payne–Hanek for large `im`; `csqrt` from a sign-stable hypot+sqrt.

**Truncation error must be bounded**, not just rounding: `lgamma`/`erf` have series/asymptotic truncation error that the Ziv test won't catch unless it is bounded and folded into the interval. Carry Bernoulli numbers, `2/√π`, `log(√2π)` to the full width.

Precedent to follow: `conformance/src/fp.rs` is the existing from-scratch, dependency-free, spec-pinned RNE reference pattern.

## 6. Component 3 — the minter (`conformance/src/bin/kiss_mint.rs`)

A Rust binary that, for each op in the op-manifest and each declared input cell:

1. Computes the expected value using **`semantics.rs`** (non-transcendental / domain / exact-byte ops and all special-value logic) or **`hp.rs`** (transcendental magnitude), selected by the op's family.
2. Emits a cell in the Component-1 format, with `class`/`ulp_bound`/`provenance:"oracle"`/`rounding`/`tags` set from the op's determinism class and the input's edge tags.
3. Records the `certificate` (hardness margin + stabilizing precision) so the frozen artifact certifies itself.

The minter owns the **declared input set** per atom — deliberately seeded with the load-bearing edges §6.5-0008 requires: NaN propagation, signed zero, each atom's domain boundaries and overflow arguments, plus the stressors the research flagged (near-midpoint constructions, large-|x| trig up to ~2¹⁰²³, `tan` just off odd·π/2, `sin` just off k·π, `lgamma` near negative integers and near 1/2, `erf` deep tail, `pow` with large |b|).

## 7. Component 4 — the reader / differential harness

**Reader** (`conformance/src/corpus.rs`): parses the JSON bundle into typed cells (reusing `parse_hex`), validating the envelope (`schema`, `ulp_metric`, `dtypes`).

**Differential** (`conformance/tests/corpus_differential.rs`): for each cell, runs the implementation-under-test on the pinned inputs and compares against `expected` under the cell's `class`, dispatching to the existing comparators — `compare` (exact-byte), `compare_f32` (ULP), `compare_c32_transcendental` (split). This *replaces* the inline `assert_eq!` golden tests incrementally: the JSON corpus becomes the source of truth, the Rust test a thin reader.

**Clause tests wired here:**
- `test_conform_oracle_vector_coverage_complete` (**§6.5-0008**): loads `op_manifest.json` and asserts every oracle-differential op — and every transcendental atom's required edge tags — is present in the frozen corpus; a missing op or a transcendental covered only at interior points (no edge tags) fails.
- `test_conform_oracle_vector_stores_wide_precision_value` (**§6.5-0009**): asserts every cell carries an inline `expected` value and a `certificate` (no cell defers its value to a live run), and that transcendental cells' `stabilized_precision_bits` exceed the compute dtype.

The existing compute-not-store loop (`differential.rs`) stays for exhaustive fuzzing beyond the frozen set (this is the "live oracle" mode; per §6.6-0007 its findings gate only once promoted into a frozen cell with `provenance:"promoted-differential"`).

## 8. Component 5 — the validation gate (`tools/validate_corpus.py`, dev-time only)

**Never shipped, never required by ordinary devs.** Reads the frozen JSON bundle (language-neutral — the validator does not call Rust) and certifies every `provenance:"oracle"` cell against **three independent sources**; the corpus may freeze only when all agree bit-for-bit on the rounded value:

1. **mpmath** (pure Python) — its own Ziv loop (raise `mp.prec`, recompute until the rounded f64 is stable across two precisions; round via exact-midpoint sign comparison in high precision, never naive `float()`).
2. **MPFR via `gmpy2`** — an algorithmically independent codebase from mpmath, driven by **CORE-MATH's MIT-licensed hard-to-round `.wc` inputs** (vendored under the validator's **dev-only** tree `tools/corpus-validation/hard-cases/`, NOT the shipped `conformance/corpus/` — deterministically subsampled with a recorded seed since `exp.wc` alone is >10 MB, so consumers never pull them). This is the adversarial input set where a shared bug would hide.
3. **Lefèvre–Muller published worst cases** — ~dozens of pinned anchor points (facts, cited from Arith-15 2001 / the Handbook), the only source independent of **both** mpmath and MPFR; catches a bug those two might share. Covers exp/log/sin/cos/atan; **not erf** — erf agreement carries lower confidence unless Arb/FLINT (`arb_hypgeom_erf`, ball arithmetic, non-MPFR) is added as an optional fourth leg.

**Protocol & guards** (from research):
- Independence is only real if the engines share no constants: Rust hard-codes its own π/ln2/(2/π); mpmath and MPFR compute their own; a few constants are checked against a third source before trust.
- The validator pins its own `mpmath`/`gmpy2`/MPFR/GMP versions in a provenance header so "validated once" is reproducible.
- `gmpy2` needs a GMP/MPFR toolchain (dev-only). On platforms where that is painful, `python-flint` (Arb) is the documented alternative MPFR-independent engine.
- Agreement certifies **rounding**, not **completeness** — the gate is meaningful only because the minter (Component 6/3) deliberately includes the hard stressors.

## 9. Component 6 — the coverage source (`conformance/corpus/op_manifest.json`)

The op/atom enumeration §6.5-0008 checks against, **derived from ops.md** (the `tools/kiss_ops.py` extraction pattern: primitive floor, non-primitives + families, complex ops, transcendental atoms by family), emitted as a checked-in JSON manifest so the Rust reader needs no Python. Regenerated by maintainers when ops.md changes and gated by a `kiss_ops`-style drift lint, keeping it **derived, not hand-written** — consistent with how `kiss_trace.py`/`kiss_ops.py` already derive their tables from the spec.

## 10. Scope & sequencing

Full corpus is the destination; the path is a proven vertical slice first, then mechanical scale-out.

1. **Slice 0 — format + core + minter + reader end-to-end.** Format v1 frozen; `hp.rs` core for exp/log/sin at f32 **and** f64; `semantics.rs`-backed exact-byte ops (`add`, a signed-zero case); one complex op (`clog`); minter emits a small bundle; reader + §6.5-0008/0009 tests green on the slice; validation gate green (all three sources) on the slice. **This proves the architecture and freezes the format.**
2. **Slice 1 — the rest of the transcendental atoms** (cos/tan/atan/atan2/erf/lgamma/pow) with their hard stressors; per-atom minting + validation fannable across workflow agents.
3. **Slice 2 — remaining dtypes** (f16/bf16/fp8 where transcendentals apply; complex family) and the exact-byte/integer/structural op coverage.
4. **Slice 3 — coverage closure**: op_manifest completeness so §6.5-0008 passes over the full op set; migrate remaining inline golden tests to the corpus.

Per Ultracode, the per-atom mint+validate work in slices 1–3 is fanned out with workflows (one agent per atom: mint → validate against three sources → report), synthesized back. The format never changes after slice 0, so scaling is additive.

## 11. Risks & mitigations

| Risk | Mitigation |
|---|---|
| Double-double ships a silent misround as "truth" | Rejected; 256-bit core with Ziv escalation to 512/1024. |
| 256 bits is a margin, not a theorem (pow, tan near pole) | Keep 512/1024 escalation wired; a straddle triggers it; an input reaching 1024 unresolved is a red flag to investigate. |
| Agreement certifies rounding, not completeness | Minter deliberately seeds the hard stressors (§6.5-0008 edges); the gate is only as good as the corpus's adversarial inputs. |
| mpmath + MPFR share a bug | Lefèvre–Muller anchors (independent of both); optional Arb/FLINT for erf. |
| Engines share constants ⇒ shared-constant bug passes silently | Independent constants per engine; spot-check a few against a third source. |
| JSON doesn't scale to exhaustive sweeps | Freeze only curated edge cases; exhaustive stays in the compute-not-store loop. |
| Signed-zero lost under plain ULP | Sign-of-zero cells are exact-byte/split, enforced by schema + a minter check. |
| `provenance:"oracle"` is an unenforceable integrity claim | Relies on the §6.5-0005 authoring-independence process; a mislabeled cell would launder a quirk — flagged as a process obligation, not a code check. |
| Rounding mode inferred | `rounding` mandatory per cell; no global default. |
| `gmpy2`/MPFR toolchain friction (Windows) | Dev-only; `python-flint` (Arb) documented as the alternative MPFR-independent engine. |

## 12. Clause traceability

| Clause | How this design satisfies it |
|---|---|
| §6.4-0002 | Raw-hex bit-pattern cells; envelope pins byte order/encoding explicitly. |
| §6.3-0003 | Versioned machine-readable JSON bundle + op_manifest, self-contained for a foreign reader. |
| §6.5-0001 | Reader/differential harness drives comparators against an implementation-under-test. |
| §6.5-0007 | 256-bit core is the "wider than compute dtype, round once" floor for f64. |
| §6.5-0008 | `op_manifest.json` + coverage test over every oracle-differential op and each transcendental atom's edges. |
| §6.5-0009 | Every cell stores an inline `expected` + `certificate`; no cell defers to a live run. |
| §6.13-0007 | Transcendental atoms under declared ULP + split comparator for complex-transcendentals. |

## 13. Open questions (for the writing-plans stage, not blockers)

- **Op-manifest extractor placement**: extend `tools/kiss_ops.py` to emit `op_manifest.json`, or a new small extractor? (Leaning: extend `kiss_ops.py` with an `--emit-manifest` mode, reusing its existing ops.md parsing.)
- **CORE-MATH `.wc` subsample size** per atom (balance repo weight vs adversarial coverage) — pick during slice 1, record the seed.
- **erf fourth leg**: adopt Arb/FLINT now or defer until erf lands in slice 1.
