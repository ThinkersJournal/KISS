# KISS ⇄ Baracuda ⇄ Fuel — Convergence Reconciliation (single shared list)

**Maintained by:** KISS (ThinkersJournal editors-of-record) · **Reconciles:** the KISS `RECONCILIATION.md` D-list, Baracuda's `kiss-convergence-agenda-2026-07-20.md`, and the open KISS RFC/defect issues into **one view all three sides work from**.
**As of:** 2026-07-20/21 · **Outcome:** convergent — **zero divergent rows** between KISS's tracking and Baracuda's agenda (Baracuda-confirmed).

## Frame (the two guardrails)

1. **No loss of either side's shipping functionality or roadmap** — Baracuda accepts 8/8 D-asks; the work is *sequencing and layering*, not "whether."
2. **KISS stays project- and language-agnostic (Eric, 2026-07-20)** — KISS changes to serve Baracuda/Fuel *only if* the change stays adoptable by any ML ecosystem in any language. Every provider-specific value (ULP tables, accumulator lattices) is carried **informative-only**; the normative clause gates on the *declared* per-target fact, never a provider's specific choice.

## Unified status (D-list ⇄ agenda ⇄ issues)

| # | Item | Issue / vehicle | Owner | Status |
|---|------|-----------------|-------|--------|
| **D2** | Retire §6.8 transcendental ULP ceiling → advisory floor; declared per-target tier `{max_ulp\|max_relative\|max_absolute}` is the sole gate; arg-dependent form reserved post-v1 | **#63 MERGED** | KISS | ✅ **MERGED to main** (both cosignatories approved) — **closed #39 + #42.** |
| **D1** | Operand + accumulator + output dtypes in the `gem` identity key; formalize `batch` | sk3 RFC §4.1.2 | KISS-Classify + Baracuda | 🔶 In the sk3 bundle (g6uuwo0p). |
| **D4** | dtype vocab + retire the `f32s` *token* (atomic with the `<mp>` MathPrecision coordinate); keep `s16`; prune `u16`/`u64`; MX additive | sk3 RFC §4.1.2 / §6.17 | KISS + Baracuda | 🔶 In the sk3 bundle; `f32s`-retire ordered strictly after `<mp>` exists (C-1). |
| **D5** | Surface Contract `accumulation_type` from `PrecisionGuarantee.accumulator` (one-field emit) | sk3 RFC §4.2 (**folded**) | Baracuda emit; KISS text | ✅ **Folded into sk3 §4.2** (not a standalone issue — would fragment the single bump). Informative lattice + same-token-spelling invariant + deferred exact-reduction note. |
| **batch** | `/b<class>` suffix, conditionally-present `<batch>` | sk3 RFC §4.1.2/§7 | KISS + Fuel | 🔶 In the sk3 bundle (g6uuwo0p flipping §4.1.2/§7). |
| **sk3 bundle** | D1+D4+D5+batch = **one** `sk2→sk3` bump (grow the key once, C-2) | sk3 RFC | KISS (g6uuwo0p) + Fuel/Baracuda ack | 🔶 **Eric-approved, gated on the §6.17 `<mp>` input-rounding pin** (with kiss-ref for sign-off). NOT "open." |
| **D3** | §6.6 Dispatch optional + a geometry-agnostic kernel class (grid-stride + host `Dim3`) | **#43** | KISS | ⏳ Open spec-edit accept — queued to author. |
| **D6** | Reproducibility-scope as a distinct axis orthogonal to the fidelity/comparator class; **keep** determinism-class → comparator selection | — | KISS | ⏳ Open spec-edit accept. |
| **D7** | DLPack = interchange; bless a **neutralized FDX-successor sidecar** (DLPack + MX stay OUT of the identity key; finite key + open sidecar, C-3) | (RFC to file) | KISS + Fuel + Baracuda | ⏳ Open — three-way co-design queued. |
| **D8** | sk2 codec: `sk1→sk2`, `sm*→cuda:sm*`, `i32/i64→ix32/ix64` | landed `aca0aa85` | Baracuda | ✅ Landed. #60 provenance-regen green-lit for g6uuwo0p to commit + close. |

## Related defect/coverage issues (KISS-internal)

| Issue | Status |
|-------|--------|
| **#42** atan2 self-contradictory determinism class | ✅ **Closed** — folded into #63 (§6.8-0005); #58 closed as superseded. |
| **#41** decomposition grammar can't parse structured-op rows | ✅ **#59 MERGED** (closed #41) — scopes grammar to scalar bodies + routes structured ops via §6.11/§6.13-0004. Both resolvers confirmed **from code** (Baracuda kernelgen + Fuel fuel-graph) that structured ops go through attribute records, never scalar trees. |
| **#44** no byte layout for `cost`/`per_backend_ulp_tiers`; no float encoding | 🔶 **PR #62** adds the real-number encoding (§6.11-0010, the load-bearing half) — merging now. `per_backend_ulp_tiers` layout now rides the D2 tier shape (post-#63); `cost`/`cost_class` half still open. Stays open. |
| **#57** governance / RFC-as-GitHub-issue | ✅ **MERGED** — umbrella §7.2 + CONTRIBUTING + issue templates. |
| **#22** namespaced target_capability + output operand in key | ✅ **Closed** — both halves landed on origin/main + provider-confirmed. |
| **#5** build-on-miss latency | 🟡 Partial: bounded-latency floor landed (§6.6-0001c); two-phase light-offer/heavy-fetch handover shape still open. |
| **#9** broaden dtype table | 🟡 Partial: integer half (s16/u16/u64) landed (#30); sub-byte floats → #32; u16/u64 pruning is an open editors' call. |
| **#10** op-vocab agreement + vendor op tier | 🟡 Partial: vendor-namespaced op tier landed (§7.3); op-vocab-agreement pin + importer-spelling clause open. |
| **#64** determinism.rs typed-decline (Option vs panic) | ⏳ Filed (Fuel review follow-up; non-blocking). |

## Baracuda's five firm positions — KISS concurrence

- **C-1 — never retire `f32s` before the `<mp>` key coordinate exists.** ✅ KISS agrees; the sk3 RFC makes `f32s`-retire atomic-with-`<mp>` (E2 confirmed). Portable rule: a compute-precision attribute must be keyable before its dtype-token proxy is retired.
- **C-2 — grow the `gem` key exactly once (D1+D4+D5+batch = one `sk2→sk3`).** ✅ KISS agrees (E1); D5 filed *into* sk3, not standalone.
- **C-3 — MX/quant + DLPack stay OUT of the key, in the FDX-successor sidecar.** ✅ KISS agrees; D7 co-design honors finite-key + open-sidecar.
- **C-4 — Baracuda's `oracle.rs` is a first-party test asset, never its own freeze certifier.** ✅ KISS agrees; matches KISS's own "oracle independence is only code-deep" caveat.
- **C-5 — hold KISS-Classify UNFROZEN until the key bump lands + a non-CUDA reader exercises the wire.** ✅ KISS agrees; = charter D9.

## Freeze gate — E6 maintainer ruling (Eric)

The §8 freeze gate **stays open.** Current sibling implementors (Baracuda, kiss-ref, Fuel, Vulkane) all trace to a single interpretation of the spec, so Vulkane + Baracuda's per-clause `oracle.rs`-lineage abstention are a **partial mitigation, not a clearing.** Freeze requires **genuine external diversity** — other human minds, other ML-framework backgrounds, other-language implementors reviewing/implementing KISS first. structure_key condition-1 is **met** (Baracuda emits byte-exact from real derivation; Fuel's independent deriver byte-reproduces); the recorded head-to-head runs on #60 + the r1 golden. But KISS-Classify is recorded **draft pending genuine external implementors**, NOT "condition-2 met."

## Net

8/8 D-asks accepted; the convergence reduces to the five firm positions + one sequencing spine. Hold those, grow the key once, and both Baracuda and Fuel reach full KISS conformance without losing a shipping asset or roadmap item — while every rule stays general enough for any ML ecosystem to adopt.
