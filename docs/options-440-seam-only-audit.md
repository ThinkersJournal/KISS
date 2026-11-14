# Options note: board 440, KISS states what crosses the seam, never how a kernel does it (audit and change list)

**Informative. No normative text, no spec or vector change. For CireSnave's decision (via the PM).** Evidence read at
`origin/main` `71d678a` (conformance 0.17.4) in `spec/ops.md`, `spec/contract.md`, `spec/announce.md`,
`spec/classify.md`, `conformance/src/{opattrs,structural}.rs`, `conformance/tests/matmul_attrs.rs` and the two
OpAttrs corpus files. Predecessor: `docs/options-771-contraction-precision-and-accumulate-into-c.md`.

## The rule being applied (CireSnave, 2026-11-14, via the PM)

KISS specifies what crosses the seam: what accuracy and fidelity a requester accepts, and what a kernel declares it
delivers. It never specifies how a kernel achieves it. A compound operation (his word "recipe", a sequence of
operations that describes a fused kernel) is described by that fused kernel's own contract when it is imported, and
is never listed in advance by KISS.

## 1. Audit: what names a kernel's internal strategy

Each item is marked **seam** (a statement about inputs, outputs, declared guarantees or caller obligations),
**internal** (a statement about how a kernel computes), or **mixed**. The audit covers what I read; a clause I did
not open is not listed, and the last row says so.

| Item (where) | Mark | Why |
|---|---|---|
| `precision_tag` vocabulary `none`/`3xtf32`/`fp8-fast-accumulate` (ops.md 6.17-0011, the 6.19.2 table, 6.19-0049; `opattrs.rs` `PrecisionTag`; 11 `precision_tag` entries in `opattrs_value_vectors.json`, 1 in `opattrs_vectors.json`; `matmul_attrs.rs`) | **internal** | The names are recipes: `3xtf32` is a three-pass split of an f32 product, `fp8-fast-accumulate` is a hardware accumulator mode. The clause itself says "KISS never defines by recipe", yet the vocabulary is the recipe's name. |
| The rules tying a tag to a compute dtype and to `math_fidelity` (6.17-0011; `structural.rs` `TagNeedsReducedMantissa`, `TagComputeDtype`) | **internal** | They encode which strategy exists on which hardware (`3xtf32` only for f32). That is knowledge about kernels, not about the seam. |
| "A kernel advertising a tagged cell MUST declare a bound; a tagged cell with no bound is refused" (6.17-0011; `structural.rs` `TagWithoutBound`) | **seam** | It says what a kernel must declare. The rule survives the tag: it can be restated over `reduced-mantissa-permitted` cells. |
| `math_fidelity` `bit-stable` / `reduced-mantissa-permitted` as a requester permission (6.17-0001, -0003; Classify `<mp>` `st`/`rm`; Announce accuracy record 6.6-0008) | **seam** | "A reduced-mantissa strategy is acceptable" is what a requester accepts. 6.17-0003 already says it is a permission, never an obligation. |
| Declared width `n` in `reduced-mantissa-permitted:<n>` (Contract 6.8-0004a) | **seam** | A kernel declares the fewest input-mantissa bits it retains. |
| The pinned input rounding, TF32 as canonical, 10 bits, RNE (6.17-0006) | **mixed** | It is there so a reference can derive a bound and reproduce rounded operands, which is a measuring stick. But it fixes how a kernel rounds. See question 3. |
| `accumulator` for an **integer** compute dtype (6.17-0010, 6.19-0049) | **seam** | The wrap on overflow decides the result bits, so the requester states it. |
| `accumulator` for a **float** compute dtype (6.17-0008, -0009; Contract 6.8-0011, 6.8-0012; Classify `<acc>`) | **mixed** | 6.17-0008 already reads it as a floor ("at least as wide as the compute dtype"), and the declared tolerance is stated per `(compute, accumulator)` cell. Read as a floor it is an accuracy request; read as an exact internal width it is strategy. See question 2. |
| Reduction monoids `sum`, `prod`, `max`, `min` (6.11-0002) | **seam** | They define the result. |
| Tuple-state monoids `welford`, `online-softmax`, `argmax` (6.11-0026 to -0029) | **seam** | The packed state `(n, mean, M2)`, `(m, l)`, `(value, index)`, its merge and finalise are the output a consumer receives and combines. The names are algorithm names, but the clauses define data that crosses the seam. |
| The per-cell reference value, ascending-index evaluation (6.17-0009) and the `k(S,A) * N * eps_A * |max partial sum|` band (Contract 6.8-0012) | **seam** | An oracle and a form for declaring a bound. It measures kernels; it does not prescribe one. |
| `determinism_class`, `reference_function`, `per_backend_ulp_tiers`, `cost` expressions (Contract 6.8-0001, -0002; 6.7) | **seam** | Declared guarantees. `cost` is already where a kernel declares speed. |
| `in_place` and `in_place_eligible_variants` (Contract 6.5-0009, 6.7-0003) | **seam** | Which operands may alias is a caller obligation. |
| `tie_rule` of `argmax`/`argmin` (6.11-0021) | **seam** | It decides which index is returned. |
| `awkward_layout_strategy` (Contract 6.7-0001, -0003) | **not marked** | The name suggests internal strategy; I read only that it "describes the kernel's declared layout/aliasing family". Not audited further. |
| Everything else in the three specs | **not audited** | Time-boxed to the items the PM named plus the fields found on the way. |

**Result:** exactly one family is internal, the precision tag with its dtype and fidelity rules. Everything else I
read is seam-level, with two mixed items (the pinned TF32 rounding and the float accumulator).

## 2. What a requester states and a kernel declares, in place of the tag

Today a requester states one thing: `math_fidelity` (permission: `rm` admits reduced-mantissa strategies). A kernel
declares its numbers in the Guarantees (`per_backend_ulp_tiers`, a tagged `max_ulp`, `max_relative` or
`max_absolute`, plus the declared width `n`). Matching is two steps (the key admits every `rm` kernel, the declared
bound then decides), and that second step is currently done outside the op, by the consumer.

- **A. Remove the tag and add nothing.** The requester's accuracy need stays outside the op: it reads declared
  bounds at match time. *For:* smallest change, nothing new to keep consistent. *Against:* a requester cannot state
  "I need at most 4 ULP" in the op, so two consumers with different needs are not distinguishable by the op.
- **B. Remove the tag and add an optional requested bound to `matmul` (later `reduce`)**: one tagged quantity in the
  Contract's own tier shape (`max_ulp` | `max_relative` | `max_absolute`), default "no bound requested". Matching
  rule: a kernel is admitted when its **declared** tier for the target is **no looser** than the requested one,
  which is the rule Announce 6.6-0010 already applies between a kernel and a device floor. *For:* it states exactly
  what crosses the seam and reuses the existing tier format. *Against:* one new OpAttrs field, and whether a
  requested bound belongs in op identity or beside it is a design question (an identity field splits caches by
  bound).
- **C. Coarse fidelity classes** (more `math_fidelity` values). *Against:* needs a KISS-owned bound partition,
  which board 427 declined to pin.

**Speed stays out of the request.** A kernel already declares speed as `cost`; a requester asking for speed would
ask for a strategy. **Recommendation: B, with the field riding the sk8 event, and A for the interval before it.**
The 6.17-0011 "declare a bound" rule is kept, restated over `rm` cells: a kernel advertising an `rm` cell declares a
tier or declares none and is `unbounded` (Contract 6.8-0002 already allows that).

## 3. Can a fused kernel's contract describe a compound operation?

Mostly yes today. Contract 6.4-0002: an `op_dag` of KISS-Ops op nodes at mixed levels, "a fusion MUST be its DAG",
root equal to the op identity. 6.4-0003: each node carries the OpAttrs its op defines (so a `matmul` node carries
`accumulator` and `math_fidelity`). 6.4-0005/-0006: the DAG resolves to primitives, and the lowered form is the
verification oracle. 6.8-0002: `reference_function` plus a tier is the declared accuracy against that oracle. So a
`3xtf32` kernel could say: Semantics = split, three `matmul`, add; Guarantees = tier against the f32 reference. That
is the recipe the tag stood in for, in the place CireSnave describes.

What is missing, each only as far as I checked:
1. **No per-node accuracy.** Tiers are per kernel and backend, not per DAG node, so a contract cannot say "the
   inner product is `rm`, the final add is exact". Whether that is needed is a question for the first real fused
   kernel; I would not add it speculatively.
2. **The recipe wire home is unresolved.** ops.md 6.19.6 defers the recipe/FlatDag container wire to issue #67
   (R1.25, `docs/options-r1-25-region-and-recipe-identity.md`). I did not check what has been ruled there since.
3. **No op for a split or a hi/lo product.** A recipe made of those steps needs ops for them. I did not search the
   op table for them, so I cannot say whether they exist.

## 4. Consumers, breaking-ness, and the sk8 batch

- **Consumer use (asked, no answer yet).** I asked Unpopped on Synapse whether anything emits, reads or keys on a
  tag other than `none`. In this repo the only non-`none` values are in tests and corpus vectors; I found no kernel
  declaring a tagged cell. Unpopped's answer will be added to this note or sent to the PM.
- **Breaking.** Removing `precision_tag` shrinks the `matmul` blob from three bytes to two (`00 01 01` becomes
  `00 01`) and changes the corpus vectors and the golden for any document carrying a `matmul` node, so it is
  **Breaking** by the 0.17.0 precedent. The blob was empty before 0.17.0, so only blobs written at 0.17.x are
  affected. Option B adds a field and is a second blob change if it ships separately.
- **sk8.** Per the PM's recommendation, one breaking release: remove the tag and add the requested bound (B)
  together with the sk8 key event, so persisted blobs and keys break once. Removing the tag alone could ship
  earlier, but it would be a second breaking release for the blob.
  Cost to CireSnave: sk8 waits on this decision.

## 5. Open questions for CireSnave

1. Should the requester be able to state a numeric accuracy bound in the operation itself (option B), or should the
   requester's need stay outside the operation and be settled when a consumer reads a kernel's declared bounds
   (option A)?
2. For a float accumulator, is the requested accumulator width a floor ("accumulate in at least this width", which
   is how 6.17-0008 reads today) that stays in the operation, or is it a statement about how a kernel works that
   should give way to the accuracy bound?
3. Clause 6.17-0006 fixes how operands are rounded for `reduced-mantissa-permitted` (TF32, 10 bits, round to
   nearest even). Should KISS keep that as a measuring reference only, or drop it because it fixes how a kernel
   rounds?
4. Should the rule "a kernel advertising a reduced-precision cell declares a bound or is `unbounded`" stay, now
   attached to `rm` cells instead of to the tag?
5. May the tag be removed now as its own breaking release, or must it wait and ride the sk8 event together with the
   requested bound?

— lane: kiss
