# RFC: Backward-op category — training as a first-class KISS use case

| | |
|---|---|
| **Status** | **Draft (2026-08-06).** Maintainer confirmed training space is a first-class KISS use case (2026-08-06); this RFC establishes the *category convention* for backward (gradient) ops. Not schema-affecting; independent of the `sk4` event. |
| **Date** | 2026-08-06 |
| **Affects** | KISS-Ops (§6.1 op-set registry, §6.2/§6.9 op-family tags, §6.13 decompositions, §6.0 determinism), KISS-Classify (§6.5 op-family tag) |
| **Source** | Fuel↔KISS vocabulary-alignment program, item #4b. Fuel: KISS-Ops is currently *forward-only* (a spec-wide search found zero backward/vjp/grad ops), yet training-capable KISS-speakers need them. |
| **Related** | Attention-family ops RFC (#4a); `selective_scan`/SSM (#4c); the non-primitive-decomposition pattern (`gelu`/`softmax`/`layer_norm` already in KISS) |
| **Cosigners** | KISS (hub/steward), Fuel, kiss-ref, Baracuda, Vulkane, Unpopped |

> **Informative until adopted.** This RFC pins the *category convention* — how backward ops enter
> KISS — not the individual backward decompositions, which are additive follow-ons once the
> convention is fixed.

---

## 1. Summary

KISS-Ops is forward-only. Training-capable speakers (Fuel today) need backward/gradient ops, so the
load-bearing decision is the **category convention**, not any single op. This RFC proposes:

**Backward ops are NON-PRIMITIVE decompositions, not new primitives.** A backward op is named for
dispatch and **decomposes to the existing forward primitive floor** — exactly the pattern KISS
already uses for `gelu`/`silu`/`softmax`/`layer_norm`. The primitive floor (§6.3) stays **forward-only
and unchanged**; a backward op is just another non-primitive whose reference decomposition is the
vector-Jacobian product (VJP) expressed in forward primitives.

This is the minimal, KISS-native answer: it adds a whole training category **without touching the
primitive floor**, gives every backward op a **derived** determinism class (§6.0, from its
decomposition) and a **decomposition-checked** conformance obligation (§6.5 oracle) for free, and
keeps the forward/backward pairing as ordinary nodes in one DAG (no special graph structure).

## 2. Motivation

Fuel's spec-wide search found **zero** backward ops in KISS-Ops. So `softmax_backward` /
`layer_norm_backward` / `rms_norm_backward` aren't individually missing — the entire gradient-op
category is absent, and every training-capable consumer must either invent its own backward names
(divergence, the thing this program exists to prevent) or cannot express training graphs at all.

The decision that unblocks all of them is the convention: **are backward ops primitives or
decompositions?** Get that wrong (backward as primitives) and the primitive floor bloats with one
new primitive per differentiable op, each needing its own conformance oracle. Get it right (backward
as decompositions) and the category is additive over the machinery KISS already has.

## 3. The convention

### 3.1 Backward = non-primitive decomposition (VJP)

A backward op computes the **vector-Jacobian product**: for a forward `y = f(x₁…xₙ)` and an upstream
cotangent `ȳ`, it produces `x̄ᵢ = (∂f/∂xᵢ)ᵀ · ȳ`. That product is expressible in forward primitives —
matmuls, reductions, and the elementwise derivatives — so a backward op **has a reference
decomposition to the §6.3 floor** and introduces **no new primitive**. (Examples, schematic:
`matmul` backward = two `matmul`s; `softmax` backward = an elementwise/`softmax`/reduce composition;
an elementwise activation backward = its elementwise derivative times `ȳ`.)

### 3.2 What comes for free

- **Determinism class** — derived from the decomposition via §6.0 (a backward that sum-reduces over a
  batch axis is order-invariant/nondeterministic; a purely elementwise backward is exact-byte or ULP
  per its atoms). No bespoke rule.
- **Conformance** — a backward op is checked against its decomposition by the §6.5 oracle, identical
  to every other non-primitive; no new oracle surface.
- **Graph structure** — a backward op is an ordinary node in the same DAG, consuming forward
  activations and upstream cotangents. No forward/backward graph split, no special pairing construct.

## 4. Open design questions (for cosign)

1. **Naming.** `<op>_backward`, `vjp(<op>)`, or `grad_<op>`? Proposed: a single systematic scheme
   (lean: `<op>_backward`, with the VJP semantics normative). Must be one convention, not per-op.
2. **Op-family tag (§6.5 classify).** Does a backward op get a distinct `grd` (gradient) family tag,
   or inherit the forward op's family / classify by its decomposition's dominant op? (Trade-off:
   a `grd` tag makes gradient kernels a coarse dispatch category; inheriting the forward family keeps
   the tag set smaller.)
3. **VJP contract precision.** Pin the cotangent convention (which operand is the upstream gradient,
   multi-output fan-in accumulation, the reduce-to-broadcast-source rule for broadcasted inputs).
4. **First backward set.** The training-critical ops to register with decompositions first:
   `softmax_backward`, `layer_norm_backward`, `rms_norm_backward` (Fuel's ask), the elementwise
   activation backwards (`gelu`/`silu`/`sigmoid`), `matmul` backward. Each is an additive follow-on.
5. **Out of scope (confirm).** Activation checkpointing / recomputation is an implementation strategy,
   not a semantic — not named here. Higher-order (grad-of-grad) rides the same decomposition rule
   recursively and needs no separate convention.

## 5. Relationship to the rest of the program

- **Not schema-affecting.** Backward ops are additive op-registry + decomposition entries; they do
  **not** touch `structure_key`, so this RFC is independent of the `sk4` event and can proceed in
  parallel.
- **Feeds #4a/#4c.** The attention family and `selective_scan` will each want a backward once this
  convention is fixed; naming/decomposition follow the same rule.

## 6. Cosign

Pending. Each cosigner confirms the **convention** (backward = non-primitive decomposition, §3), then
the four open questions (§4) are resolved before the first backward decompositions land.
