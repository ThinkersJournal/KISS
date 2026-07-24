# RFC: Float-reduction accumulator width as a tolerance-cell key (KISS #90, direction b)

**Status:** draft · 2026-07-24 · authored by kiss-ref (the reference implementation)
**Direction:** (b) — signaled by Eric 2026-07-24; supersedes the open (a)/(b) choice on KISS #90.
**Clauses AMENDED (existing text changes):** §6.17-0007 (the "bound against wide-precision truth" sentence — C3 rewrites it), §6.0-0004 / §6.17-0005 (C1 adds the (compute, acc) classification).
**Clauses BOUND/REUSED (no text change):** KISS-Classify §6.7-0006 (`<acc>` key coordinate, C2), §6.6-0003 (extent-free key, C4), §6.8-0004 (tolerance lives in the Contract Guarantees, C4).
**Clause CROSS-REFERENCED (independent):** §6.11-0002 (no-inf monoid identity, C5).
**Author role:** reference-implementation feedback (C-4). kiss-ref surfaced the gap by executing the tensor layer on the narrow lanes (commit `e8ae0b5`, `narrow_tensor_lane.rs`) and pinned the divergence as a golden rather than hiding it.

> **Filing note (KISS-side, 2026-07-24).** Filed on kiss-ref's behalf into the KISS #57 RFC
> process (KISS text is `c413a26` on kiss-ref, reproduced verbatim below), for the KISS-Ops
> editor's **clause ratification**. This RFC document is the proposal; the actual spec-text
> edits (the §6.17-0007 amendment + the C1–C5 normative clauses + ledger) are a follow-up
> realization PR on ratification — the same two-step path the §6.11 structural-atom RFC (#75)
> took to its realization (#83). Direction (b) is ruled; the specific clause wording below is
> what awaits ratification. Tracks KISS #90.

## Summary
Float sum/prod reductions, scans, and contractions have an unpinned accumulator width (§6.0-0004; open-question 6). §6.17-0005 pins a bit-stable reference profile — ascending index order, accumulator at the storage dtype's precision — which kiss-ref implements, so kiss-ref is conformant, not wrong. But §6.17-0007 requires a declared tolerance to classify a candidate pass-or-flag, and no clause supplies one for a narrow compute dtype. Two provably-conformant implementations diverge with nothing to bound the gap.

Measured (pinned goldens, `narrow_tensor_lane.rs`): reduce-sum of `[128, 8×8]` (wide-truth 192, representable in every lane) is 128 under the storage-precision profile (e4m3/e5m2: `128+8` rounds back to 128, RNE stagnation) and 192 under an f32 accumulator (every real FP8 tensor core). 33% divergence, both conformant, no tolerance. (Also: softmax rows don't sum to 1 — e4m3 +3.1%, e5m2 −6.3%.)

Direction (b), refined: treat these ops as tolerance-cells keyed on (compute-dtype, accumulator-dtype), reusing the existing `<acc>` key coordinate — no new key field. Accumulator width becomes a determinism-class axis orthogonal to `<mp>`. Neither reverses §6.17-0005 nor constrains silicon: any accumulator dtype stays conformant, compared against its own reference cell.

## The core move: an accumulator-parameterized reference
The 128-vs-192 gap is a "failure" only if both compare against ONE reference. Under (b) they are DIFFERENT CELLS: `<acc>=e4m3` compares against the e4m3-accumulator reduction (128, the §6.17-0005 diagonal); `<acc>=f32` compares against the f32-accumulator reduction (192); the cross-`<acc>` divergence is expected/conformant, never compared. Within a cell only reduction reassociation remains → declared tolerance is TIGHT (small ULP at the accumulator's precision). Requires one reference change: kiss-ref's reduction must be parameterized by accumulator dtype to emit the reference for any declared `<acc>`.

## Proposed normative clauses

**C1 — classification (amends §6.0-0004 / §6.17-0005):** a float sum/prod reduction, scan, or contraction is a tolerance-cell whose class + declared tolerance are a function of (compute-dtype, accumulator-dtype). Accumulator dtype MUST be a float dtype ≥ the compute dtype width (a narrower accumulator can't improve on storage precision, has no use). §6.17-0005 storage-precision profile is the diagonal cell accumulator==compute.

**C2 — key coordinate (binds §6.7-0006):** keyed on the existing `<acc>`; NO new key field. Absent `<acc>` ⇒ accumulator==compute (the §6.17-0005 default), preserving today's behavior for every kernel that never opted in.

**C3 — reference value (AMENDS §6.17-0007):** §6.17-0007 currently ends: "A tolerance cell's declared tolerance MUST bound the combined input-rounding-plus-reduction-order error against the wide-precision truth — the KISS-Conform §6.5-0007 oracle evaluation." For an accumulator-keyed reduction cell this is unsatisfiable in any useful form (narrow-accumulator reference sits ~33% from wide truth → a 33% tolerance that flags nothing) and contradicts the per-cell reference. REPLACE it, for the (compute, acc) reduction/scan/contraction cells, with:

> The reference value of a (compute-dtype S, accumulator-dtype A) reduction/scan/contraction cell is the §6.17-0005-ordered (ascending-index) evaluation with each input rounded to S, each accumulate atom rounded to A, and the result rounded to S. The cell's declared tolerance MUST bound a candidate's reduction-order (reassociation) error against that per-cell reference. The cell whose A is the widest permitted accumulator dtype IS the §6.5-0007 wide-precision oracle; every narrower-A cell's reference is a defined, representable point whose distance from that oracle is the cell's characteristic (the semantics of using that accumulator), not an error to be toleranced away.

This keeps §6.17-0007's intent (bound the error a candidate may introduce) while removing the self-contradiction: widest-A cell binds against wide truth (§6.5-0007), narrower cells bind against their own fully-specified deterministic reference. Wide-truth conformance preserved transitively, not discarded.

**C4 — tolerance form (lives in the Contract Guarantees, not the key):** declared tolerance for cell (S,A) scales with reduction length: `k(S,A) · N · eps_A · |max partial sum|`, as a ULP-of-A band. NOT a fixed constant, NOT a bound against wide truth; tight within-cell. Keying discipline (§6.6-0003 / §6.8-0004), load-bearing: N (reduction extent) is RUNTIME and the key is EXTENT-FREE (§6.6-0003), so N MUST NOT enter the key. Key carries only the (compute, acc) class via `<acc>` (C2). The numeric FORMULA lives in the Contract Guarantees, evaluated per-invocation with the real N (§6.8-0004). `k(S,A)` is the per-cell constant declared there — the one open calibration number. A reader must not conclude N is keyed: the class is keyed, the magnitude computed per call.

**C5 — no-inf monoid identity (independent; already on #90):** when a monoid identity is ±∞ (max=−∞, min=+∞, §6.11-0002) and the compute dtype has no infinity encoding (e4m3fn), the identity MUST materialize as the dtype's finite extremal magnitude of the correct sign (±448 for e4m3fn). Both impls already do this; lands separately.

## Reference-implementation obligations (kiss-ref)
1. Parameterize the reduction reference by accumulator dtype (`kernels::reduce`/`prefix_scan`/`tensor_ops::matmul`): accumulate in A via the existing promote-compute-round machinery while rounding inputs to S; current storage-precision path = the A==S case, unchanged.
2. Expose the per-cell reference on `diff.rs` for Baracuda's 3b + Fuel.
3. Add the A==f32 companion goldens beside the existing A==S diagonal.
4. Hold affected cells Provisional until ruled, then conform.

## Open calibration point
`k(S,A)` is declared per (compute, acc) cell (the summation-bound constant differs by accumulator width), calibrated empirically and adversarially: for each (S,A), sweep the reassociation orders a conformant kernel may use (pairwise tree, blocked, sequential, worst-case adversarial) against the pinned ascending reference over an adversarial value set; set k to the observed worst case plus margin. kiss-ref produces the per-cell calibration table as a follow-up once the clause shape is ratified.

## Why (b) over (a)
(a) — pinning one canonical accumulator per dtype — makes narrow reductions byte-comparable but reverses §6.17-0005 for narrow lanes and renders a genuinely-narrow-accumulator device non-conformant (constrains silicon). (b) keeps every accumulator width conformant, matches hardware reality (FP8 tensor cores accumulate f32, some accelerators accumulate narrow), reuses `<acc>` at zero new key cost, turns the divergence from unclassifiable failure into a keyed/bounded/executable tolerance cell.
