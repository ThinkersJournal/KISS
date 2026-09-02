# KISS ↔ kiss-ref: reference-implementation & conformance architecture

**Status:** non-normative (informative). The normative sources are `spec/{ops,classify,conform,umbrella}.md`; where this doc and a spec clause disagree, the clause wins.
**Version:** v0.1 · 2026-07-21 · **stable citation anchor** for outreach/consumer docs.
**Supersedes:** the 2026-07-16 handoff prose ("…kiss-ref is the oracle every KISS consumer tests against"). That framing was wrong and is corrected here; do not cite it.

**Citation convention:** KISS sub-standards share the `§6.x-NNNN` numbering space, so every clause
reference below is **prefixed with its home sub-standard** (`KISS-OPS-…`, `KISS-CONFORM-…`,
`KISS-CLASSIFY-…`) to prevent cross-sub-standard mis-resolution. A bare `§N` (no `-NNNN`) is a
structural section of the named sub-standard.

This document records how the KISS standard establishes conformance, where the reference
implementation (`kiss-ref`) sits relative to the conformance oracle, and the independence
properties that keep the standard's conformance credible to a third party. It was reconciled
against `spec/conform.md` and ratified across the KISS-ecosystem peers (Baracuda, Fuel, KISS
editors) on 2026-07-21.

---

## 1. The authority model: crown the corpus, not the implementation

Conformance is defined by a **versioned corpus of pinned vectors**, not by agreement with any
running implementation. This mirrors how mature standards do it — Unicode `NormalizationTest.txt`,
the WebAssembly spec tests, Berkeley TestFloat, NIST CAVP: the artifact of record is a frozen
test corpus that every implementation is measured against.

Two instruments produce and consume that corpus, and they must not be confused:

- The **KISS-Conform §6.5 oracle** (`conformance/src/semantics.rs`) is a **wide-precision authoring
  instrument** — the tool that *mints* corpus vectors. It is not "the live truth" and it is not
  run on a consumer's hot path; it authors vectors that then stand on their own. It is independent
  by mandate (`KISS-CONFORM-6.5-0002`/`-0003`): derived solely from the KISS-Ops §6 semantics
  tables, sharing no lowering code with any reference implementation, and it evaluates
  transcendentals at a precision **wider than the compute dtype, rounding once**
  (`KISS-CONFORM-6.5-0007`).
- **`kiss-ref`** (repo `kiss-ref`) is a **corpus-conformant proxy** — a spec-exact reference
  implementation that is *tested against* the corpus. It is **not** spec-truth and **not** the
  oracle. Per `KISS-CONFORM-6.5-0002`/`-0003` a vector derived from a kiss-ref run is **circular**
  and is rejected; kiss-ref never mints the corpus.

---

## 2. Roles and separation

| Instrument | Role | Independence property |
|---|---|---|
| KISS-Conform §6.5 oracle (`semantics.rs`) | Mints the corpus | Wide-precision; shares no lowering code with any reference impl (`KISS-CONFORM-6.5-0002`); per-vector provenance tag rejects reference-derived vectors (`KISS-CONFORM-6.5-0003`) |
| kiss-ref | Differential *target* + "it always works" correctness floor; tested against the corpus | Hardware-precision; code-disjoint from the oracle, but see §3 |
| Baracuda `oracle.rs` | First-party device test asset | Shares no lowering code with the emitter, but shares upstream `build_plan`/IR (agenda C-4) — never its own freeze certifier |
| Consumers (Fuel, Vulkane, Baracuda kernels) | Implementations under test | Measured against the corpus |

kiss-ref serves four consumers with none privileged: KISS-Conform (a spec-exact executable
reference to differential against, distinct from the independent KISS-Conform §6.5 oracle), Fuel (a
verify reference **and** a correctness-floor execution route), Baracuda (a reference to reconcile
kernels against), and Vulkane (same reference role).

---

## 3. The comprehension-correlation caveat (load-bearing)

Code-disjoint is not comprehension-independent. kiss-ref and the KISS-Conform §6.5 oracle **both
transcribe the same `KISS-OPS-6.13` decomposition table**. So on the **non-primitive** ops they are
code-disjoint but **comprehension-correlated**: a kiss-ref↔oracle agreement on a non-primitive is
one reading of the spec agreeing with itself, not independent evidence.

- kiss-ref is a **strong** independent check on the **primitive floor** (`KISS-OPS-6.3`), a
  **weak** one on the `KISS-OPS-6.13` **decompositions**.
- **General rule (state it wherever a diff is counted as independent):** *any differential check
  whose reference is a shared decomposition table is not decomposition-independent.* This covers
  two cases: (1) kiss-ref ↔ KISS-Conform §6.5 oracle on a non-primitive (both read
  `KISS-OPS-6.13`); (2) a fused consumer kernel diffed against kiss-ref's `KISS-OPS-6.13`-resolved
  reference — decomposition-independent **only until** the consumer's recipe grammar unifies with
  `KISS-OPS-6.13` (the recipe = pattern = Semantics unification thread), after which it
  recategorizes to *kernel-vs-shared-decomposition*. Both remain valuable (they catch "kernel
  doesn't implement the decomposition" bugs); neither counts toward the external diversity the
  freeze gate needs.

Where kiss-ref and Baracuda's `oracle.rs` **differ** in the structural region, that is a **spec
ambiguity to file as a KISS issue** — not an implementation to converge. Two honest readings
diverging is the signal (live example: the u32-gather `same_as` bug, `f7578df1`). Reconciling the
two implementations *against each other* would only deepen the correlation and hide the spec bug.

---

## 4. The freeze gate is interop, not numerical

The KISS-Umbrella §5.3 / KISS-Conform §8 freeze gate certifies **interop / wire / foreign-reader**:
≥2 structurally dissimilar implementations interoperate on the golden vectors, and a foreign reader
parses the wire.

**Condition 1 is counted PER FIELD, not per implementation** — and this is the half most easily lost
when the gate is summarized. For each field of a golden vector, at least two parties must
*independently derive* that field's value. A party that receives the value and reproduces it
byte-exactly demonstrates **faithful passthrough** — real and testable, and *not* evidence of
independent derivation. So a second implementation that derived most fields itself but took a handful
from the reference **fails on exactly those fields**, and no passing differential would reveal which:
byte-agreement is what interop *requires*, so agreement can never establish provenance. Note this is
the case a shared-lowering-code check cannot catch, because **a passed-through field has no lowering
code to share.** A conformance report quoting a whole-vector count must state, per field, whether each
party **derived** or **copied** it. (Normative text: umbrella §5.3 condition 1.) It is **interop / implementable / unambiguous** only — there is deliberately **no
numerical oracle-cross-check item at the gate.** Numerical truth lives entirely at the corpus + the
KISS-Conform §6.5 oracle; it is not re-litigated at the freeze gate.

kiss-ref can serve as one *dissimilar implementation* for the interop/wire/foreign-reader axis. It
must **not** be counted as an independent numerical check even though it looks like one (§3).

**Maintainer ruling (E6, 2026-07-21): the KISS-Conform §8 gate STAYS OPEN** pending genuine external
diversity — other human minds, other ML-framework backgrounds, other-language implementors
reviewing/implementing KISS. Rationale: Baracuda, kiss-ref, Fuel, and Vulkane all trace to a
single reading of the spec, so a Vulkane seat + per-clause abstention on `oracle.rs`-lineage
clauses is *partial* mitigation, not gate-closing. The spec may already be right, but that cannot
be *assumed* without outside validation.

---

## 5. Precision posture: hardware vs wide, and the reproducibility axis

kiss-ref evaluates **in the compute dtype** (libm-grade transcendentals) — it is a
**hardware-precision** reference by construction. It does **not** anchor the wide-precision corpus;
that is the `KISS-CONFORM-6.5-0007` path (KISS's independent 256-bit vendored-precision core). Two
independent reasons forbid kiss-ref from anchoring it: (a) the `KISS-CONFORM-6.5-0002`/`-0003`
independence mandate — a reference impl minting corpus vectors is the circularity the architecture
exists to prevent; and (b) method — `KISS-CONFORM-6.5-0007` evaluates *wider than compute, rounds
once*, a different numeric method than kiss-ref's compute-dtype evaluation.

The real comparison axis is **bit-reproducible vs within-declared-tolerance**, with three region
sources feeding one classifier:

1. **Exact region** (integer arith, comparisons, `select`, rounding, shape/cast, bitwise) —
   kiss-ref is **bitwise** to the KISS-Ops §6 pin, so a verify seam tightens to **bit-exact**
   against it, no band.
2. **Transcendentals** (`KISS-OPS-6.8`) — kiss-ref is only *within-ceiling* of the wide truth, so
   two within-ceiling impls can differ ~2×. Keep a widened band against kiss-ref; tight
   transcendental verification points at the `KISS-CONFORM-6.5-0007` wide corpus, **not** kiss-ref.
3. **Nondeterministic-reduction GEMM** (`<mp>=rm` TF32 / warp-reduction, per the sk3 work) —
   behaves like the transcendental region: tolerance, not bit-exact.

A pending sk3 requirement follows (kiss-ref's reference-impl-role stake): `KISS-OPS-6.17`
MathPrecision must pin the exact input-rounding each `<mp>` value implies, and each `(acc, mp)` cell
must be classified **bit-reproducible → golden** vs **nondeterministic → declared-tolerance**,
precisely enough for a spec-derived reference to reproduce mixed-precision GEMM. This is the agenda
D6 reproducibility-scope axis; the codec/key-spelling half is out of the reference-impl lane.

---

## 6. Live-run discipline (`KISS-CONFORM-6.6-0007`)

Raw fuzz/live-run outcomes **must not gate a conformance verdict** — in *either* direction. A
beyond-corpus kiss-ref discrepancy does not Reject and a kiss-ref agreement does not Adopt; both
are signals. The path is: kiss-ref diff **flags** → minimize → **escalate** to the KISS-Conform §6.5
oracle to mint a pinned corpus vector → the (now-extended) corpus produces the verdict. A consumer's
Adopt/Reject therefore rests on corpus coverage or post-escalation; kiss-ref is a
discrepancy-*detector feeding escalation*, wired distinct from the corpus verdict path.

---

## 7. Where things live

- **This doc** (`KISS/docs/conformance-architecture.md`) — the authority model, the independence
  reconciliation, the oracle. Non-normative; interprets the normative clauses.
- **`kiss-ref/DESIGN.md`** — the kiss-ref-specific slice: scope, the provenance rule, the crate
  DAG, the build sequencing, distribution. Links here for the authority model.
- **Normative** — `spec/ops.md` (KISS-Ops), `spec/classify.md` (KISS-Classify), `spec/conform.md`
  (KISS-Conform), `spec/umbrella.md` (KISS-Umbrella).

---

## 8. Coverage anchor

`kiss-ref@ce4c047 · coverage_ledger → 78/106 ops evaluable` (machine-checked by the
`coverage_ledger` test in `kiss-ref-conformance`). Composition: **106 = 43 primitive floor + 63
non-primitive** (complex `KISS-OPS-6.18` deferred; the count is the vocab enumeration, which matches
KISS's own `op_manifest.json all_ops` — a no-drift cross-check, not independence). Done = float
floor + elementwise non-primitives over `f16`/`bf16`/`f32`/`f64` (minus `nextafter` on the narrow
floats, `KISS-OPS-6.9-0003`) + integer floor atoms over all 11 integer dtypes incl. packed
`s4`/`u4`/`b1`. Pending = the structural/tensor layer + the reductions/scans/norms/matmul/pooling
through it, and the FP8/`bool`/complex dtype breadth. (Anchor commit `ce4c047` is the 2026-07-21
doc-correction commit; re-pin if kiss-ref advances before this doc lands.)

---

## 9. Open coordination items (tracked elsewhere, listed for context)

- **Provenance / lineage vocab** — a shared, project-agnostic attribution set, incl.
  `evaluation_precision {compute-dtype | wider-than-compute}` and `derivation_lineage
  {spec-6.13-table | external-cold-reader}` (the latter operationalizes E6 so tooling can't
  double-count shared-lineage sources). Under CireSnave review via Baracuda.
- **sk3 numeric-semantics** — `KISS-OPS-6.17` `<mp>` rounding + `(acc,mp)` reproducibility
  classification (§5). Routed via Baracuda → KISS-Ops / RFC editor.
- **Consumer repoint** — a consumer retiring its self-realized primitive-floor reference to
  corpus(verdict) + kiss-ref(diff target); the consumer's cost-oracle and recipe-identity are
  untouched. Consumer-ratification item (ratified by CireSnave for Fuel, 2026-07-21).
