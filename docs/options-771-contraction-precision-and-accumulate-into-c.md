# Options note: KISS#771, contraction precision policies and accumulate-into-C

**Informative. No normative text. For CireSnave's decision (via the PM).** Evidence read at `origin/main`
plus the stamped #812 head (`spec/ops.md` KISS-OPS-6.17-0010, -0011, KISS-OPS-6.19-0049); Unpopped's RFC is
KISS#771. This is the options note promised in the 2026-11-04 triage.

## What is already decided

CireSnave ruled on 2026-11-12 (board 427 "1: A") that 3xTF32 and fp8 fast-accumulate are a **tag** from a
**maintainer-owned vocabulary** plus a **declared bound** (R1.36). #812 implements it: `precision_tag`
(`none`, `3xtf32`, `fp8-fast-accumulate`) is the third byte of the `matmul` OpAttrs blob
(KISS-OPS-6.19-0049); a non-`none` tag needs `math_fidelity = reduced-mantissa-permitted`, names its compute
dtype, and a kernel advertising a tagged cell MUST declare a bound in its Guarantees (KISS-OPS-6.17-0011).
KISS pins neither the arithmetic nor the bound's value. So half (1) of the RFC is answered in kind: Unpopped's
`Tf32x3` is the `3xtf32` tag. What is **not** decided is (a) whether `Tf32`, `Bf16x3` and `F16Accum` join the
vocabulary, (b) whether the policy belongs in the `structure_key`, and (c) accumulate-into-C.

## Question 1: is the precision policy part of the key?

The triage lean was "yes, it is asked for, so it keys". Reading the key shows why that is not free:
a `ctr` cell already carries `<acc>` (the accumulator dtype) and `<mp>` (`st` strict, `rm`
reduced-mantissa permitted), `c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp>` (Classify 6.7-0006).
A consumer that sets `rm` already says "a reduced-mantissa strategy is acceptable". The tag says **which
one**.

- **A. Not in the key (what #812 does).** The tag rides in the OpAttrs blob, in op identity beside the key
  (KISS-CLASSIFY-6.6-0020). Admissibility is then two steps: the key admits every `rm` kernel, and the
  declared bound in the Guarantees decides whether a given tagged kernel is good enough for the consumer.
  *For:* no byte of any key moves, so the sk7 batch closes on #812; a new tag never changes keys, so
  vocabulary growth is free. *Against:* two kernels with different bounds (plain TF32 and `3xtf32`) share a
  key, so a cache or registry that looks up by key alone cannot tell them apart; the consumer must also
  compare bounds. Today that is already true of the `revision_hash` (the key says what was asked, the
  revision what was produced).
- **B. A new key field carrying the tag.** A consumer that must have `3xtf32` keys differently from one
  content with `rm`. *For:* the lean the triage stated, and it keys what the consumer asked for. *Against:*
  a key change (a schema event), so it rides with **sk8** (per-operand dtype, which `Fp8FastAccum` needs
  anyway) rather than reopening sk7; the key grows by a field every contraction carries; and every new tag
  becomes a registered token with the key's closed-domain rules.
- **C. Fold a coarse bound class into `<mp>`** (a third and fourth value, e.g. "near-f32" and "plain"). *For:*
  a consumer expresses the accuracy it needs, not a recipe, which matches the declared-bound philosophy.
  *Against:* it needs a defined, KISS-owned bound partition, which the 2026-11-12 ruling declined to pin.

**Result of the check the triage promised:** under A, two contractions differing only in policy key
**identically**, by design; the test the triage proposed ("they key differently") would fail, and that is
the point of A, not a defect. The triage lean was a hypothesis and the check does not support it for sk7.

**Recommendation: A for sk7, and carry B (not C) as an open item in the sk8 RFC.** A closes the sk7 batch on
#812 with nothing moved; B is evaluated once, next to per-operand dtype, so persisted keys break once, not
twice. If CireSnave rules B, nothing in #812 is wasted (the tag stays in the blob).

## Question 2: which policies join the vocabulary

#812 admits `3xtf32` and `fp8-fast-accumulate`. The RFC also lists `Tf32`, `Bf16x3`, `F16Accum` and
Unpopped's `k_step` (the atom's K) and bound `(gamma_K + gamma_s)*S + d(P)*S`.

**Recommendation:** the maintainer adds `tf32`, `bf16x3` and `f16-accum` to the `precision_tag` vocabulary
under the ordinary growth rule **when an emitter ships a tagged cell with a declared bound**, not before
(R1.36: a tag without a bound is refused). `k_step` and the bound formula stay **inside the declared bound**,
not KISS fields: they describe one strategy's arithmetic, which KISS does not pin. Unpopped's
`oracle/accum_policy.rs` and `tests/contraction_accum_policy.rs` are the independent reference the bounds
can be checked against; Unpopped should send the expected arrays when it files the first tagged cell.
`Int32`/`Int32Sat` are already answered by KISS-OPS-6.17-0010 (signed 32/64-bit integer accumulator,
exactly summed).

## Question 3: accumulate-into-C (`D = alpha*(A.B) + beta*C`, full `[M,N]` C)

- **A. Ordinary ops, no new matmul form.** `D = add(mul(alpha, matmul(A,B)), mul(beta, C))`; a backend that
  fuses it advertises a fused cell. *For:* nothing new in KISS-Ops. *Against:* the fusion has no spelling a
  provider can *advertise* as one kernel with a C operand, which is the RFC's problem.
- **B. Admit a dense row-major `[M,N]` extra input to the contraction**, read at the output element after
  the K-sum in the wide type, and leave `alpha` and `beta` as ordinary scalar ops. A rank-1 `[N]` extra
  input stays the per-column bias; the two already key differently (`OperandKey::bcast`). *For:* additive
  under the capability model, no key byte moves (the operand shape already keys), and it is what Unpopped
  implements (0.42.24). *Against:* it extends the contraction's extra-input rule, which pins only the bias
  today.
- **C. As B plus `alpha`/`beta` as OpAttrs fields.** *For:* the whole epilogue is one op. *Against:* more
  fields in a blob that changed this batch, and scalar ops cover them.

**Recommendation: B**, with `alpha` and `beta` left as ordinary ops. It is additive, moves no key bytes, and
matches the one implementation. It is a Draft change to KISS-Ops's contraction extra-input clause (not
Breaking: an existing `matmul` without C is unchanged).

## What I need from CireSnave

1. Question 1: A for sk7, B on the sk8 list, or B now (which would delay the sk7 close and ride sk8)?
2. Question 2: confirm tags are added as emitters ship bounded cells (recommended), not up front.
3. Question 3: B (the `[M,N]` extra input), or A (nothing).

— lane: kiss
