# Decided: which zero a float `reduce(max|min)` returns (vulkane#189 follow-up, pm-kernels TASK signed-zero-B-ruled)

Status: **DECIDED, option B** (PM / cluster ruling 2026-11-27, relayed by pm-kernels; CireSnave can overrule; C stays open as a later tightening). 2026-11-27, against `origin/main` 8828b194 (conformance 0.24.3). **No stored byte moves** (section 5).

## 1. The gap

KISS-OPS-6.0-0002 (`spec/ops.md:573-581`) files the `max`/`min` `reduce` and `prefix_scan` monoids as **exact-byte** ("order-invariant in value"), and KISS-OPS-6.11-0002 (`:1027`) pins only NaN propagation and the empty-axis identity; no clause said which zero a fold returns over `-0.0` and `+0.0`. KISS-OPS-6.15-0001 (`:1670-1680`) pins operand `a` on a +-0 tie for the four **binary** minmax ops only; a reduction has no fixed `a`.

## 2. What the code already does (read at 8828b194)

- The harness canonicalises +-0 before the byte compare of a `max`/`min` reduce: `conformance/src/structural.rs:494-535` (`canon_signed_zero`), `conformance/src/harness/advertised.rs:42`; the comment at `structural.rs:286` gives the reason (a change of order can flip the sign of a zero result) and names `scatter` `atomic-max`/`atomic-min` as carrying the same exception. So the code already behaves as option B while the spec text read as byte-exact.
- As a fact about the oracle only, not a contract: the KISS oracle fold (`structural.rs:60-72`, `semantics.rs:20`) keeps its accumulator on a tie, so it returns the first-seen zero in ascending-index order. Nothing in the clause below promises this.
- vulkane#189 (run 45972534688, hosted Apple paravirtual device, safe math, 32 lanes): MSL `simd_max`/`simd_min` ties went to the lowest lane's zero (consistent, not proof); that measurement is a datum, not a rule.
- **Not claimed:** the behaviour of torch `amax`/`max`, and the tie order of Unpopped's CPU-C collective, are UNVERIFIED (unread / untraced); this note says nothing about either.

## 3. The ruling and its clause text (B)

A `max`/`min` reduction over cells containing both `+0.0` and `-0.0` returns **a zero, value-equal**: byte-equal is not promised. The clause is an **exception sentence added to KISS-OPS-6.0-0002** (`spec/ops.md:573`; no new clause id, so no sidecar entry), exact wording:

> KISS-Conform MUST evaluate an exact-byte op with a byte-exact comparator, **except for the sign of a zero**: when a floating-point `max` / `min` `reduce` or `prefix_scan` (or a `scatter` `atomic-max` / `atomic-min`, §6.11-0010) folds cells holding both `+0.0` and `-0.0` and the folded result is a zero, the result is **a zero, value-equal** — either `+0.0` or `-0.0` — and byte-equality of that zero is **not promised** across implementations or fold orders (a result that is not a zero, and NaN propagation, §6.11-0002, stay exact-byte). KISS-Conform MUST compare such a result with the sign of a zero canonicalised on both sides before the byte compare and MUST NOT canonicalise any other bit. This pins no stored byte: no corpus vector, token or schema version changes.

6.15-0001 is untouched: the binary minmax ops keep operand `a` on a tie.

## 4. Is it a normative spec change?

Yes: it edits `spec/ops.md` (a MUST in an existing clause), so it ships as a spec PR. It is **not** a byte-altering change, and the byte-altering rule (only CireSnave approves a change that alters stored bytes) is not engaged: no codec field, structure-key token, corpus vector, contract or schema version moves; the comparator behaviour the clause states is what `canon_signed_zero` already does. The PR body cites the PM ruling.

## 5. Stored bytes

None move. No committed vector exercises a +-0 tie in a reduce (checked: the only signed-zero reduce-adjacent vector, `packed_reduce.rs:290` `argmax_signed_zeros_equal`, is `argmax`, a different op, and is unchanged).

## 6. Options considered

- **A. First-seen zero in ascending-index order.** Keeps exact-byte true but constrains every lane- or tile-parallel lowering for a corner nobody observes.
- **B. A zero, value-equal (ruled).** States what the harness does; no schedule constraint (6.0-0004 already pins none); does not collide with 6.15-0001.
- **C. +0 wins for max, -0 wins for min (IEEE-maxNum style). Recorded as the OPEN TIGHTENING.** Schedule-independent bytes, so the only option giving bit-reproducible plans; costs a split from 6.15-0001's a-biased binary ops. Can be adopted later as a strict narrowing of B (every C result is a B result), moving no stored byte either.

## 7. Who it affects

Plans claiming bit-stable float `max`/`min` reduce or scan results (a zero result is the only case), and lane/tile-parallel lowerings such as vulkane's MSL collective. NaN is unaffected (6.11-0002).
