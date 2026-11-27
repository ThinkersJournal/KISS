# Decided: three gaps in out-of-range index behaviour (pm-kernels TASK oob-gaps-ruled; fuel GAP-413, baracuda E4b)

Status: **DECIDED as recommended** (PM ruling 2026-11-27, relayed by pm-kernels; CireSnave can overrule). 2026-11-27, against `origin/main` 8828b194 (conformance 0.24.3). Semantics only: **no stored byte moves** (section 5). Normative: yes (two existing clauses in `spec/ops.md` gain sentences). Byte-altering: no.

## 1. What KISS already stated (not a gap)

`gather` takes an `oob_policy` in `{skip, clamp, zero-fill, reject}`, a negative index is always out of bounds, and `reject` ends in a typed decline naming the first OOB index with no partial output as its result (KISS-OPS-6.11-0004). A `skip` gather with no `base` operand declines on an actual OOB read (6.11-0015); `index_select` is `gather(oob=skip)` with no base (6.13-0009), so its OOB declines. `scatter` MUST skip an OOB write (6.11-0005) and a skipped write retains `dest` (6.11-0016); `scatter_add` is `scatter(atomic-add, oob=skip)`. Rule of thumb: reads decline, writes skip. Clamping in `index_select` and unchecked `gather`/`scatter` violate these clauses today; a CUDA `scatter_add` that skips and returns Ok conforms.

## 2. The three rulings

- **(a) `index_add` and the `scatter_add` family skip an OOB write**, as `scatter` does (6.11-0005). KISS has no `index_add` op today (0 hits in `spec/`); the clause therefore binds every write op whose reference decomposition is a `scatter`, so it covers `index_add` the day it is defined, without defining it here (defining an op moves op-identity bytes and is not part of this ruling). **The clause makes no claim about torch `index_add_`** (its OOB behaviour is unverified and nothing relies on it).
- **(b) A read decline is pinned by its outcome:** delivered, naming the first OOB position, before the result can be consumed. A deferred out-of-range flag surfaced at completion (B1), a pre-validation pass over the index operand (B2), or not offering the op (B3) are each conformant; none is required.
- **(c) The value at a lane whose index declined is unspecified, and the evaluation never faults** (no undefined behaviour, trap or host crash).

## 3. Clause text (exact), `spec/ops.md`

Two existing clauses gain a sentence each; **no new clause id, so no sidecar entry** and no test row moves.

**KISS-OPS-6.11-0004** (`spec/ops.md` line 1049; after "...before the `gather`." and before its *Test:*), appended:

> **Delivery of a read decline:** the decline of this clause and of §6.11-0015 is pinned by its **outcome**, not its means — it MUST be delivered, naming the first out-of-bounds index position, before the result can be consumed. A device that cannot return a decline at launch MAY satisfy this by a deferred out-of-range flag surfaced as the typed decline when the launch completes, by a pre-validation pass over the index operand, or by not offering the op; none is required. Once a decline is delivered the output buffer is not a result: its value at a lane whose index declined is **unspecified**, and the evaluation MUST NOT fault (no undefined behaviour, trap or host crash, KISS-CONFORM §6.7-0004).

**KISS-OPS-6.11-0005** (`spec/ops.md` line ~1062; after "...and MUST skip an out-of-bounds write."), appended:

> Every write op whose reference decomposition is a `scatter` — `scatter_add` (§6.13-0009) and any accumulating index-write op KISS defines later, an `index_add` among them — inherits this rule: its out-of-bounds write is **skipped, never declined**.

## 4. Is it a spec change, and is it byte-altering?

Normative, yes: it edits `spec/ops.md`. Byte-altering, **no**: no codec field, structure-key token, corpus vector, contract or schema version moves, and no op is added; it fills gaps and states the outcome the existing clauses already require. The PR body cites the PM ruling. It changes no existing MUST: fuel's earlier "OOB = error everywhere" idea (which would have changed 6.11-0005) is not adopted.

## 5. Stored bytes

None. No committed vector exercises an OOB index on a device path, and no sentence above touches an encoding.

## 6. Who it affects

fuel GAP-413 (the contract row), baracuda E4b (CUDA `scatter_add` skip conforms; a gather path needs B1 or B2), vulkane/Metal/Vulkan (the `index_select` clamp violates 6.11-0015 today; Vulkan's unchecked `gather`/`scatter` violates 6.11-0004/-0005 and the no-OOB-access rule).

## 7. Options considered (record)

(a) skip (ruled) vs reject (would make `index_add` the only declining write) vs stay absent (needs a broadcast-index check on `scatter`, not done). (b) B1/B2/B3 all allowed vs mandating one. (c) unspecified (ruled) vs a pinned fill value (costs every device a pass for a value no consumer may read).
