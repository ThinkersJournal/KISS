# Options: which zero a float `reduce(max|min)` returns (vulkane#189 follow-up, pm-kernels TASK reduce-signed-zero-note)

Status: **options with a recommendation; nothing here is a clause and no byte moves.** 2026-11-27, against `origin/main` 8828b194 (conformance 0.24.3). Written by the KISS lane for the PM to put to CireSnave.

## 1. The gap (two sentences)

KISS-OPS-6.0-0002 (`spec/ops.md:575-581`) files the `max`/`min` `reduce` and `prefix_scan` monoids as **exact-byte** ("order-invariant in value"), and KISS-OPS-6.11-0002 (`:1027`) pins only NaN propagation and the empty-axis identity; no clause says which zero a fold returns when the operands are `-0.0` and `+0.0`. KISS-OPS-6.15-0001 (`:1670-1680`) pins operand `a` on a +-0 tie for the four **binary** minmax ops only, and a reduction over lanes or tiles has no fixed `a`.

## 2. What the consumers do today

- **KISS oracle (read):** `conformance/src/structural.rs:60-72` folds `max_prop` / `min_prop` from the identity -inf / +inf; `max_prop` returns the accumulator on a tie (`conformance/src/semantics.rs:20`), so the oracle returns the **first-seen zero in ascending-index order** (`[-0.0, +0.0]` gives `-0.0`; `[+0.0, -0.0]` gives `+0.0`). But the **comparator canonicalises +-0 before the byte compare** (`structural.rs:494-535`, `canon_signed_zero`; also `harness/advertised.rs:42`), and `structural.rs:286-289` states the reason in code: order "can flip the sign of a zero result". So the harness already behaves as option B below while the spec text reads as byte-exact: the two disagree, and only the code carries the exception.
- **Unpopped:** its CPU-C collective emits `a > b ? a : b` / `a < b ? a : b` (`crates/unpopped-cpu-c/src/collective.rs:326-327`, Unpopped `origin/main`); a tie yields the second operand, so the zero returned depends on which operand is the accumulator. I did not trace which; **UNVERIFIED** whether it is first- or last-seen. (The earlier "Unpopped's oracle: first-seen zero" is the KISS oracle above, not this file.)
- **torch `amax`/`max`:** **UNVERIFIED, not read.** No claim is made.
- **MSL `simd_max`/`simd_min`:** measured by vulkane (vulkane#189 run 45972534688, hosted Apple paravirtual device, safe math, 32 lanes): ties went to the lowest lane's zero. Consistent, not proof, and also NaN-ignoring (so already non-conforming under 6.11-0002).

## 3. Options

- **A. Pin first-seen in ascending-index order** (the lowest index's zero wins). Matches the KISS oracle fold and the MSL measurement; keeps 6.0-0002's "exact-byte" wording true with no exception. Cost: a tree or lane-parallel implementation must carry the index of the zero (or order its combine), i.e. it constrains the reduction schedule for a corner that is almost never observed, which the rest of the class (6.0-0004) deliberately leaves free.
- **B. Unspecified-but-must-be-a-zero.** The result MUST be a zero of the correct value (`-0.0` or `+0.0`) when the folded maximum is zero; the sign is the implementation's. 6.0-0002 gets an exception sentence (exact-byte except the sign of a zero result of `max`/`min` reduce and scan). Matches what the harness does today (`canon_signed_zero`) and what an order-free lowering can promise.
- **C. IEEE-maxNum style: +0 wins for max, -0 wins for min.** Deterministic and schedule-free (a commutative, associative total order on zeros), so every fold order gives the same bytes. Cost: it contradicts 6.15-0001's a-biased tie for the binary ops (`:478-481`, `:1674`, whose text calls IEEE `maximumNumber`/`minimumNumber` ordering exactly what KISS does not do), so the reduce and the binary op would disagree on the same two operands.

## 4. Recommendation: B

It states what the harness already enforces, needs no schedule constraint (6.0-0004 already refuses to pin one), and does not collide with 6.15-0001. C is the only option that makes the bytes schedule-independent, but it buys that by splitting the reduce from the binary op; A costs implementations a constraint nobody has asked for. If CireSnave wants bit-reproducible plans, C over A.

## 5. Who it affects, and bytes

Plans that claim bit-stable results over a float `max`/`min` reduce or scan (a zero result is the only observable case), and any lowering that is lane- or tile-parallel (vulkane's MSL collective, GPU tree reductions); NaN is out of scope here (6.11-0002 already propagates). **No stored byte changes under any option:** this is a comparator and wording question; no codec field, token, corpus vector or schema version moves, because no committed vector exercises a zero tie in a reduce. A later clause under B or C would add vectors (`[-0.0, +0.0]` both orders) but changes no existing one.

## 6. Open

Which of A/B/C is a CireSnave decision (via the PM). `scatter` `atomic-max`/`atomic-min` carry the same exception in code (`structural.rs:286`) and would follow the same answer. Torch behaviour and Unpopped's collective tie order are unread (above).
