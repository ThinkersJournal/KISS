# RFC: KISS-Ops §6.11 structural-atom under-specifications

**Status:** draft · 2026-07-21 · surfaced by kiss-ref (the reference implementation)
**Affects:** KISS-Ops §6.11 (structural atoms), §6.20 (shape oracle)
**Author role:** reference-implementation feedback — per the C-4 agreement, where a
spec-derived implementation must invent a convention the spec does not pin, that is
a spec-ambiguity to file as a KISS issue, not an implementation detail to bury.

## Summary

Building kiss-ref's tensor-evaluation layer (the six §6.11 structural atoms,
transcribed from the KISS-Ops spec) surfaced **three points where a spec-derived
reference cannot compute a defined result without inventing a convention the spec
leaves open.** kiss-ref has pinned each by local convention to stay evaluable, and
marks the affected `(op × dtype)` cells **Provisional** in its coverage ledger
pending a KISS ruling. This RFC states the three gaps and kiss-ref's provisional
pins, and proposes normative resolutions.

None are blocking — each is narrow and the reference runs today — but for a
citation-anchored conformance standard they should be pinned so two independent
implementations cannot diverge silently on a legal input.

---

## Gap 1 — `gather` OOB policy `skip` has no defined read-value

**Clauses:** `KISS-OPS-6.11-0004` (gather OOB policy ∈ {skip, clamp, zero-fill}),
`KISS-OPS-6.19-0015`/`-0027` (the `oob_policy` field permits `skip` on a gather),
`KISS-OPS-6.20-0008` (gather output shape = `data[..axis] ++ index.shape ++
data[axis+1..]`).

**The gap.** `clamp` (read the clamped in-range index) and `zero-fill` (output `0`)
are well-defined read results. **`skip` is not**: gather is a pure READ with a
*pinned* output shape and no self/base operand, so when an index is out of bounds
under `skip`, the spec does not say what value the corresponding output element
holds — undefined vs. prior vs. `0` are all consistent with the current text. A
reference cannot produce a defined result for that cell.

**kiss-ref's provisional pin.** `skip` reads a **caller-provided base value** at that
output position (gather takes an optional `base` view); `base = None` under `skip`
on an OOB index is a typed error, not a silent value.

**Proposed resolution: option (1) — give `skip`-gather a base/self operand** (the
skipped output element takes the base operand's value at that position; makes
`skip` a read-modify-passthrough). `base = None` under `skip` on an OOB index is a
typed error (the caller must supply the base).

**Baracuda (provider) cosigns (1) and counters the alternatives.** Baracuda *emits*
skip-gather — its bespoke `gather`/`index_select`/`embedding` use `skip`, defined
provider-side as "leave the output element **unwritten**", i.e. the element retains
the **output buffer's prior value**. So:
- **Option (2) — forbid `skip` on a READ — is rejected:** it would break Baracuda's
  shipped gather set.
- **Option (3) — `skip ≡ zero-fill` — is rejected:** it loses the distinction
  Baracuda's emit relies on.
- Baracuda's skip-gather **IS option (1)** with the base realized as the output
  buffer (in-place read-modify-write) — identical value-semantics to kiss-ref's
  explicit base operand.

---

## Gap 2 — `scatter` has no pinned output shape or destination/base state

**Clauses:** `KISS-OPS-6.11-0005`/`-0006` (scatter semantics + duplicate/atomic-add
combine), `KISS-OPS-6.20-0008` (enumerates gather/index_select/embedding/matmul
output shapes but **omits scatter**).

**The gap.** Two linked omissions:
- **Output shape** — `KISS-OPS-6.20-0008` does not give scatter's output shape rule.
- **Destination / base state** — no clause pins the scatter destination's initial
  content, nor whether a self/base operand exists. This is **load-bearing** for
  `atomic-add`/`scatter_add` (what value is accumulated *onto*?) and for output
  positions that are never written or are OOB-skipped (what do they hold?).

Without a pinned base, `scatter_add` has no defined result: the sum is
`base[j] + Σ updates`, and `base` is unspecified.

**kiss-ref's provisional pin.** scatter takes an **explicit owned `dest` operand**;
the output equals `dest` with the combined writes applied; OOB-skipped and
never-written positions **retain their `dest` value**. Output shape = `dest.shape`.

**Proposed resolution:** pin a scatter **self/base (destination) operand** in §6.11
and add scatter's **output-shape rule = destination shape** to `KISS-OPS-6.20-0008`.
This closes both omissions with one operand, and matches how consumers already model
`scatter_add` (base + updates).

---

## Gap 3 — empty-axis behavior is pinned only for `reduce`

**Clauses:** `KISS-OPS-6.11-0002` (reduce over an empty axis → the monoid identity),
vs. `KISS-OPS-6.11-0003` (`prefix_scan`), `-0004` (`gather`), `-0005` (`scatter`),
`-0007` (`sort_network`) — which have no empty-input clause.

**The gap.** Only `reduce` pins the empty-axis result (monoid identity). For the
other four atoms the empty-input behavior is *implied* by the length-preserving /
shape rules but never stated, so an implementation must infer it:
- `prefix_scan` over an empty axis → zero output positions along that axis;
- `gather` with an empty index operand → an empty gathered axis;
- `scatter` with an empty index → no writes (destination returned unchanged);
- `sort_network` over an empty row → an empty permutation.

**kiss-ref's provisional pin.** Exactly the length-preserving / shape-implied
behaviors above (each produces the shape its rule dictates, with no elements to
process).

**Proposed resolution:** add a one-line normative empty-input clause to §6.11 for
the four non-`reduce` atoms, confirming the shape-implied behavior above. Purely a
clarification; no behavior change expected — it just removes the inference.

---

## Recipe-grammar implication (Gaps 1 & 2) — per Baracuda provider review

Baracuda cosigns Gap 1 (option 1), Gap 2, and Gap 3, and adds the unifying fact:
Baracuda realizes **both** the skip-gather base and the scatter destination as the
**output buffer** (in-place read-modify-write), so the value-semantics are identical
to the explicit-operand pins here — *in-place vs. functional is a backend
realization detail, not a semantics difference*. Surfacing that implicit base/dest
as an explicit operand is therefore a **recipe-grammar refinement** these clauses
imply: the `gather` node gains an **optional `base` child-edge (skip only)** and the
`scatter` node gains a **`dest` child-edge**. Routed to the recipe-grammar
consolidation (mlgheozs) alongside these clauses; Baracuda backs the operand-surface
change on the provider side.

## Coverage-ledger status

Until KISS rules, kiss-ref holds the affected cells **Provisional** (evaluable under
the pins above, but flagged as not-yet-authoritative): the `gather`/`index_select`
skip-read path (Gap 1), the `scatter`/`scatter_add` base-state (Gap 2), and the
empty-axis path for `prefix_scan`/`gather`/`scatter`/`sort_network` (Gap 3). When a
resolution lands, kiss-ref conforms to it and un-flags the cells.

## Routing

Reference-impl-role item → the KISS-Ops editor + the conformance-architecture lane.
These are the divergence signals the C-4 "file, don't reconcile" rule exists to
surface; resolving them tightens the standard for every implementer, not just
kiss-ref.
