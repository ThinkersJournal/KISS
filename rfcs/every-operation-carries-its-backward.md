# RFC: every operation carries its backward

| | |
|---|---|
| **Status** | **Draft, informative. Nothing is frozen.** The principle is ruled; the clause shapes, the tie conventions and the gap list are proposals. |
| **Ruling** | CireSnave, 2026-11-13, board 430 Part 1 question 1, choice 1a strengthened, relayed verbatim by the PM: *"I agree that an operation is an operation and that it is neither a 'forward operation' or a 'backward operation'.  It is just an operation.  Each kernel viewed as a 'forward operation' should include in its contract the matching 'backward operation' defined as a sequence of operations.  That way all operations have both a forward and a backward defined for them."* |
| **Affects** | KISS-Ops (§6.13 reference-decomposition table, §6.14 levels, §6.11 gather/scatter, §6.17 accumulators), KISS-Contract (the per-kernel guarantees and output layouts, §6.5-0019) |
| **Evidence read** | `spec/ops.md` and `conformance/corpus/op_manifest.json` at `origin/main` (126 ops). `git grep -i -E 'backward|gradient|vjp' spec/ops.md` finds nothing today (ROADMAP R5.10 says the same), so this is new ground, not a revision. |
| **Related** | `per-operand-dtype-sk8.md` (the mixed-dtype key the index operands of the gather backward need), `docs/options-r5-10-sk4-deferred-scope.md` (the deferred backward-category item this supersedes) |

> **Informative until adopted.** Every formula below is a worked illustration of the *shape* of a backward
> section, written in catalog ops. The authoritative form is whatever the Ops editors pin and the reference
> evaluator checks; a hand-derived example here is not a vector.

## 1. Summary

An operation is just an operation. There is no "backward op category". Instead **every op has a backward**:
a short sequence of ordinary catalog operations that, given the **output gradients** and some **saved
values**, computes the **input gradients**. It sits beside the op's reference decomposition (§6.13) and is
written in the same language. **No backward operation is named**: there is no `attention_backward`,
`layer_norm_backward` or `conv_backward` token (§3.1). A provider that fuses a backward **advertises the
region** (KISS-Grammar: the fusion's structure is its identity), exactly as it would for any fusion. Because the backward is a sequence of ordinary ops, **the
backward of a backward is defined for free** (each op in the sequence has its own backward).

What this RFC settles, in order: where the section lives (§3), the four points the PM asked for (§4 to
§7), the **closure check** (§8), and five worked examples (§9). §10 lists what CireSnave must still rule.

## 2. What "ordinary operation" buys, and what it costs

- **Buys:** one vocabulary, no second catalog; fused backward kernels (flash-attention style) are just
  providers of a sequence; the structure key and Contract machinery apply unchanged to a backward kernel,
  because a backward *is* an op sequence with its own cell.
- **Costs:** the catalog must be **closed under backward**. If a backward needs an operation that is not in
  the catalog, that is a hole, not a footnote (§8 is that list). It also makes every new op owe a backward
  section, which is a standing review item for the Ops editors.

## 3. Where the section lives

Two surfaces, with one owner each:

| Surface | Holds | Owner |
|---|---|---|
| **KISS-Ops** | the **semantic** backward of an op: its sequence, its saved values (as definitions), its tie rule, its precision statement, or the words **no backward** | Ops editors; one place per op, in or next to the §6.13 table (a new column) |
| **KISS-Contract** | what a **kernel** says about it: whether it is run as the plain sequence or **advertised as a fused region** (a Grammar region, §3.1), **and the layout of its saved values** (§5) | the kernel's Contract (the §6.5 guarantees surface) |

The Ops text defines *what* the gradient is; the Contract states *how this kernel hands it over*. A
consumer that does not recognize a provider's fused region runs the sequence; this is the same fallback
rule as §6.14-0004 for an unrecognized op.

### 3.1 No pre-named backward operations (CireSnave, board 430 Q1 follow-up)

*Relayed by the PM, 2026-11-13:* do not pre-name backward operations. A backward is **a sequence of
ordinary operations in the forward operation's contract** (KISS-Ops reference decomposition). A provider
that fuses it **advertises the region** (KISS-Grammar region grammar: the fusion's structure is its
identity). A name is added **later only by the ordinary growth rule** (add a high-level op *by definition*)
when a region **recurs** and consumers need to match it without raising the primitive DAG.

So this RFC adds **no op token** and **no tag**. Backward sections are sequences; the table below is a
*watch list*, not a list of names to add.

**Likely to recur (candidates for later promotion, in the order they are likely to be asked for):**

| Candidate region | Why it recurs | Where it appears |
|---|---|---|
| attention backward (§9.1) | every transformer trainer wants one fused kernel | the same seven-step DAG across heads, masks, dropouts |
| layer norm / RMS norm backward (§9.3) | every transformer block, twice | `dx`, `dgamma`, `dbeta` |
| softmax / log-softmax backward (§9.2) | the loss and the attention | a three-op region |
| convolution backward, input and weight (§9.5) | every CNN | `col2im`-style scatter plus two contractions |

Not listed, because they are elementwise fusions the ordinary elementwise-chain growth rule already
covers (activation gradients) or are one `scatter_add` (embedding). The promotion test is the existing one:
a region recurs **and** consumers need to match it by name **and** the primitive DAG would otherwise grow;
none of the three is satisfied by a draft.

### 3.2 Is a canonical form of a backward region needed? (the PM's question)

Short answer: **no new canonical form; one real gap; one decision.** What the Grammar already erases, and
what it does not, read at `origin/main` (`spec/grammar.md`):

| Difference between two equal-looking backward sequences | Matched today? | Why |
|---|---|---|
| **Operand swap** of a commutative op (`mul(a,b)` vs `mul(b,a)`) | **Yes** | canonical operand order, §6.4-0010 |
| **Independent steps written in a different order** (the `dV` step before or after the `dP` step) | **Yes** | a region is a **DAG**, not a sequence; the node table is emitted in canonical post-order (§6.8-0004), so step order is not part of the identity |
| **A repeated sub-expression written twice** | **Yes** | structural common-subexpression dedup, §6.4-0011 |
| **Algebraically equal but structurally different** (`add(mul(a,b), mul(a,c))` vs `mul(a, add(b,c))`; `sub(g,x)` vs `add(g, neg(x))`) | **No, deliberately** | these round differently, so they are different computations; deciding equality is not a canonicalization problem |
| **Regrouped associative sums** (`(a+b)+c` vs `a+(b+c)`) | **No, deliberately** | different reduction order, different bits (§6.17 reduction-order cells) |
| **A saved value that is stored versus recomputed** | **No** | an emitted `L` is a bind leaf; a recomputed `L` is a subtree; those are two regions |
| **Several gradients from one kernel** (`dQ`, `dK`, `dV`) | **Not expressible** | a region has **exactly one root**; a multi-output forest is a typed decline (§6.4-0009) |

So reordered-but-equal **steps** already match; reordered-but-**not**-equal ones should not. The two rows
that matter are the last two:

1. **Saved versus recomputed.** Recommend: the Ops reference backward binds each saved value as a **leaf**
   (it is a declared input, §5). A provider that recomputes it advertises a **different, larger region**,
   which is the truth: it is a different fusion. A consumer wanting either matches either. No rule needed
   beyond stating the reference's choice.
2. **Multi-output backward.** A fused attention or layer-norm backward emits several gradients, and the
   Grammar cannot advertise that as one region today. Options: (a) advertise **one region per gradient**
   (each has one root; shared sub-expressions repeat in each region, and the kernel's Contract bundles the
   outputs with `bundle(...)`, §6.5-0019; recommended, **no Grammar change**); (b) extend the Grammar to a
   **forest** of roots (a Grammar change, §6.4-0009 reversed, and a new canonical form for the forest);
   (c) forbid advertising multi-output regions (the fused backward stays unmatched). **Recommend (a).**

**Where the reference sequence is written** matters for matching: the Ops editors MUST write each backward
in the Grammar's canonical region form (post-order node table, dedup applied) so that matching a provider's
region against the reference is a byte comparison, not an equivalence proof.

**Primitives and non-primitives.** For a **primitive** (§6.3-0001) the backward is written by hand: the
floor is small. For a **non-primitive** the default is the **mechanical derivative of its reference
decomposition**, with the derivative of `select` defined as "the derivative of the branch taken" and of a
comparison as "no backward" (§4). A non-primitive MAY **override** the mechanical result when the mechanical
sequence is unstable or wasteful (softmax is the standard case, §9.2). Termination needs no new level rule:
a backward references forward ops only, and asking for a second-order gradient just applies the same table
again.

## 4. Point 1: operations with no gradient say "no backward"

Four reasons an op has none, which the section states as a reason tag so a reader knows why:

| Reason | Ops (catalog names) |
|---|---|
| **integer / bit / bool type** | `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr`, `popcount`, `clz`, `ctz`, `logical_*`, `any`, `all` |
| **comparison / predicate** | `cmp_eq`, `cmp_ne`, `cmp_lt`, `cmp_le`, `cmp_gt`, `cmp_ge` (their output is not a differentiable value; their **inputs** gradient is the sum of the other uses, not through the compare) |
| **index output** | `argmax`, `argmin`, the index vector of `sort_network` (§6.11-0007) |
| **piecewise-constant** (zero almost everywhere, undefined at the jumps) | `floor`, `ceil`, `trunc`, `round_even`, `step`, `sign`, `nextafter` |
| **not in the catalog** | random sampling / dropout: **there is no random op today**; if one is added its backward is *no backward* with respect to the random draw (and a saved mask, §5) |

Plus the indexed ops' **index operand** (`u32`/`i32`/`i64`): it is an integer input, so the gradient for
it is *no backward*, while the data operand's backward is defined. `cast` to an integer type has none;
`bitcast` has none.

**Choice: `no backward` or "gradient is zero" for the piecewise-constant group?**
- (a) **`no backward`** for all of them. A caller who wants a straight-through estimator writes it with
  ordinary ops. Recommended: a silent zero kills learning without telling anyone, whereas "no backward"
  is an error a trainer sees at once.
- (b) "zero gradient" for `floor`/`ceil`/`trunc`/`round_even`/`step`/`sign`. Matches the mathematics almost
  everywhere, but makes a misuse invisible.

## 5. Point 2: which values the forward must save, and how that shows in the contract

The backward needs some forward quantities that are not the forward's outputs: attention's per-row
`logsumexp`, layer norm's per-row mean and inverse standard deviation. Three ways to supply them:

| Option | What it is | Consequence |
|---|---|---|
| **A. Named saved values defined by expression** | The op's section lists **saved values**, each *defined* as an ordinary-op expression of the forward inputs (`L = logsumexp(S)`, `mu = reduce_mean(x)`, `rstd = rsqrt(add(reduce_var(x), eps))`). The kernel's Contract says, per saved value, **whether it is emitted** as an extra output (layout via the existing `bundle(slot,…)` output layout, §6.5-0019) **or recomputed** by the backward. | A provider that does not want to store a value recomputes it from its definition, so the semantics never depend on the save policy. The value's layout is stated, so a fused forward and a fused backward from different providers still meet. |
| **B. Recompute everything** | The backward takes only the forward inputs and outputs. | No contract change, but the attention backward then repeats the forward's softmax statistic pass: a real cost, and it removes the freedom to save. |
| **C. Provider's secret** | Saved values are private to a forward/backward pair. | Breaks "an operation is an operation": a backward from another provider cannot run. |

**Recommendation: A.** A saved value is a *named intermediate of the forward's own decomposition*, so no
new concept enters the catalog; and the existing `bundle` layout is already the multi-output mechanism, so
no new Contract field is needed for the layout, only a list of which saved names the kernel emits.
Saved values default to **`f32` or the forward's accumulator dtype**, whichever is wider (statistics in a
narrow float lose the gradient, §7). Note the tie to ruling 427: attention's `L` is exactly the finalised
online-softmax state `(m, l)` of the packed reduction, `L = m + log(l)`.

## 6. Point 3: the convention at ties and kinks, per operation

A backward is evaluated at points where the forward is not smooth: ties in a max, zeros of `abs`, the
`relu` corner, a fully masked softmax row. Without a stated convention two providers return different
gradients for the same input, and no tolerance can bridge a difference that is a whole value.

**Rule proposed: every op section states its kink convention; there is no implicit default.** A table
form (the recommended default in the third column is the one the RFC suggests; each row is the maintainer's
to confirm):

| Op | Kink / tie | Recommended convention | Why |
|---|---|---|---|
| `abs`, `sign` | `x = 0` | gradient `0` | the sub-gradient of minimal norm; matches `sign(0)=0` (§6.13) |
| `relu` | `x = 0` | gradient `0` | the mechanical derivative of the decomposition (`select(cmp_lt(x,0),0,x)` takes the `x` branch at `0`) would give `1`; this is the case that shows the override rule is needed, and `0` matches `step(0)=0` |
| `max_prop`/`min_prop`, `fmax_ieee`/`fmin_ieee` (binary elementwise) | `a = b` | **split equally** (`0.5` each) | symmetric, keeps the sum of the input gradients equal to the output gradient |
| `reduce(max)`/`reduce(min)`, `max_pool`, `cummax`, `argmax`-driven | tie among extremal elements | **lowest index** gets the whole gradient | the same tie rule the forward `argmax`/`sort_network` already uses (stable, §6.11-0007); deterministic and cheap |
| `reduce_norm2`, `hypot`, `cabs` | input all zero | gradient `0` | the mechanical `x/out` is `0/0` |
| `sqrt`, `log`, `div`, `pow` | domain edge | the **IEEE** result of the stated formula (`inf`/`NaN`), no special case | already pinned by the forward's special-value rules (the §6.13 `pow` row cites IEEE 754-2019) |
| `softmax`/`log_softmax`/attention row | **all inputs `-inf`** (a fully masked row) | forward output `0`, backward `0` | otherwise `exp(-inf - -inf)` is `NaN` and poisons a whole batch |
| `select` | condition | gradient flows to the **taken** branch only; none to the condition | the definition (§3) |

**Choice for the max-like ties:**
- (a) **lowest index takes all** (recommended: deterministic, consistent with the forward).
- (b) split equally among all ties: conserving, but costs a count and a division.
- (c) every tied element gets the full gradient: the sum of input gradients exceeds the output gradient,
  which is wrong for a conserving transpose.

**Complex ops.** The gradient convention for complex values (which of ∂f/∂z, ∂f/∂z̄ the "gradient" is) is a
separate ruling; the complex family (`cadd`…`cpow`) leaves its backward `deferred`, listed in §10.

## 7. Point 4: precision of gradient accumulation, and the `(acc + mp)` coordinates

A backward is full of **sums**: the bias and scale gradients sum over batch and sequence, a weight gradient
sums over the whole batch, `scatter_add` sums the duplicate-index rows. These are the same kind of sum the
forward's `reduce` and `matmul` already describe, and they are the **identity coordinates** `(accumulator,
math_fidelity)` of the sk4 event (reduce carries them today; matmul gains them in the next Breaking batch).

**Recommendation: no new coordinate.** The backward section states, **per summation node**, the accumulator
and fidelity, in the same fields the forward ops already use, so the structure key covers it through the
ordinary op cell. The convention chosen for the *defaults*:

- (a) **At least `f32`** for any floating backward whose compute dtype is narrower, whatever the forward
  used. Recommended: a sum of thousands of `f16` gradients in `f16` overflows or loses the small terms.
- (b) Inherit the forward op's accumulator. Simple, but a forward that runs an `f16` accumulate for speed
  would make its gradient wrong in the same way.
- (c) A new declared "gradient accumulator" coordinate. More freedom, one more byte-visible field.

Two catalog facts the precision rule runs into, both listed again in §8: (i) `scatter_add` has **no
accumulator field** (a dtype to accumulate in, separate from the destination's); and (ii) its float
`atomic-add` is **non-deterministic** (§6.11-0006, band §6.11-0024), so the gather/embedding backward is
non-deterministic by construction unless a deterministic combine (a sorted or segmented sum) is offered.

## 8. The closure check: what the backward sequences need that the catalog does not have

Method: for each op family in the 126-op catalog, write its backward in catalog ops and list what is
missing. Read at `origin/main`; "present" means in `op_manifest.json`'s `all_ops`.

**Correction to the task's premise.** `scatter_add` **is already in the catalog** (non-primitive,
`scatter(combine=atomic-add, oob=skip)`, §6.13-0009; `scatter` with the four combines is on the primitive
floor). Gather's backward therefore needs no new op. What is missing is its **determinism** and **accumulator
dtype** (items 4 and 5 below).

| # | Backward of | Needs | In catalog? | Proposal |
|---|---|---|---|---|
| 1 | `lgamma` | `digamma` (ψ) | **No** | add `digamma` as a transcendental (non-primitive if a decomposition exists, otherwise a primitive floor addition: a maintainer call) |
| 2 | `concat` | a **slice** of an axis range | **No** (no slice/narrow op; `squeeze`/`unsqueeze` only) | do not mint an op (§6.20-0011 forbids transpose/permute minting; same spirit): express as a read **view** with offset in the cell, or as `gather` of an `iota` range |
| 3 | `im2col` (so convolution input gradient) | `col2im` | **No** (`im2col` is a closed-form structured gather with no materialised index tensor) | `scatter_add` of the gradient at the index map built from `iota` and arithmetic (`iota` is staged, ruling 422); a fused provider never materialises it. Do not mint `col2im` unless providers find the `iota` form too costly |
| 4 | `gather`, `embedding`, `index_select` (data operand) | a **deterministic** `scatter_add` | partly: float `atomic-add` is non-deterministic (§6.11-0006) | add a deterministic combine (sorted or segmented sum) as a `scatter` combine option, or declare the backward tolerance-class (§6.11-0024) and say so |
| 5 | same, and every reduction in a backward | an **accumulator dtype** on `scatter_add` | **No** (the field exists on `reduce`, not on `scatter`) | add `accumulator` (+ `math_fidelity`) to `scatter` combine `atomic-add`, same pair and order as `reduce` |
| 6 | `matmul`, convolution weight gradient | reading an operand with **axes swapped** (`A^T B`, `A B^T`) | unclear | the `matmul` decomposition fixes `input(0)` at `[m,k]`; a transposed read is a **stride view** on the operand, which a strided cell already carries (Contract §6.5-0005); the gap is that **Ops has no op-level statement of such a view**. To confirm with the editors; not a new op |
| 7 | attention (a composition) | a per-row stat, `matmul`, `exp`, `reduce(sum)` | all present | none; attention is **not a catalog op**: its forward and backward are both sequences, and the fused flash kernel is a provider of each (§9.1) |
| 8 | `reduce(prod)` | exclusive product of the others | present: `prefix_scan` has `direction` and `exclusivity` (§6.19), so `prod`-of-others = forward-exclusive × reverse-exclusive scan | none |
| 9 | `cummax` | the position that set the running max | present, composed: `cummax` over `select(cmp_eq(x, cummax(x)), iota, -1)`, then `scatter_add` | needs `iota` (staged) |
| 10 | `max_pool` | argmax over a window | `argmax` over the window axis of the pooled view, then `scatter_add`; present | none once item 3's index map exists |
| 11 | `sort_network` | inverse permutation | `scatter` of the gradient by the index vector (§6.11-0007) | none |
| 12 | `cast` (float to float) | cast back | present | none |
| 13 | complex family | a convention (§6) | n/a | deferred ruling |
| 14 | random / dropout | a random op and its mask | **No random op exists** | out of this RFC; if added, `no backward` for the draw and the mask is a saved value (§5) |

Everything else in the catalog (the arithmetic, transcendental, activation, rounding, normalisation,
`softmax`, `layer_norm`, `rms_norm`, `reduce_*`, `cumsum`, `select`, `squeeze`, `unsqueeze`,
`element_map`) has a backward written entirely in present ops; see §9 for four of them in full.

## 9. Worked examples (illustrations of shape, in catalog ops)

### 9.1 Attention (a composition, not a catalog op)

Forward, one head: `S = add(mul(matmul(Q, K^T), scale), mask)`, `P = softmax(S)`, `O = matmul(P, V)`.
**Saved:** `L = logsumexp(S)` per row (`f32`), and the output `O`. **Backward** from `dO`:

```
D  = reduce(sum, mul(dO, O), axis=d)           # one value per row
P  = exp(sub(S, L))                            # recomputed, never stored
dV = matmul(P^T, dO)
dP = matmul(dO, V^T)
dS = mul(P, sub(dP, D))
dQ = mul(matmul(dS, K), scale)
dK = mul(matmul(dS^T, Q), scale)
```

All present ops; the transposes are operand views (item 6). A **fused kernel** (flash style) computes this in
tiles and never writes `P`; it advertises the region, under no new name (§3.1). **Kink:** a fully masked row has `L = -inf`; the convention (§6) pins `P = 0`
and the row's gradients `0`, not `NaN`. **Precision:** `D`, `dQ`, `dK`, `dV` sum over the sequence: `f32`
accumulate by default.

### 9.2 Softmax (an override of the mechanical derivative)

Forward `y = softmax(x)`; saved: **`y`** (the output, nothing extra). Backward:
`dx = mul(y, sub(g, reduce(sum, mul(g, y), axis)))`. The mechanical derivative of the decomposition
(`exp`, `reduce(max)`, `div`) is correct but longer and unstable; this is the declared override.
`log_softmax`: `dx = sub(g, mul(exp(y), reduce(sum, g, axis)))`.

### 9.3 Layer norm

Forward `y = (x - mu) * rstd * gamma + beta`. **Saved (named, defined by expression):**
`mu = reduce_mean(x)`, `rstd = rsqrt(add(reduce_var(x), eps))`, both `f32` per row. Backward from `g`:

```
xhat = mul(sub(x, mu), rstd)
gg   = mul(g, gamma)
dx   = mul(rstd, sub(sub(gg, reduce_mean(gg)), mul(xhat, reduce_mean(mul(gg, xhat)))))
dgamma = reduce(sum, mul(g, xhat), over batch axes)
dbeta  = reduce(sum, g, over batch axes)
```

All present ops. `dgamma` and `dbeta` are the batch-wide sums that need the wide accumulator (§7).
`rms_norm` is the same with no `mu` and no `beta`.

### 9.4 Gather and embedding

Forward `out[i] = x[idx[i]]` (`gather`, `oob_policy` as stated; `embedding` is `oob = zero-fill`).
**Saved:** nothing but the inputs `idx`, `x`'s shape. Backward: `dx = scatter_add(zeros_like(x), idx, g)`
with the **same out-of-bounds policy, transposed**: `skip` and `zero-fill` skip the write, `clamp` writes
at the clamped index, `reject` rejects. **`idx` has no backward** (integer). **Ties:** none, but
**duplicate indices accumulate**, which is the non-determinism of item 4. This is also the cell the
per-operand dtype RFC exists for: the `scatter_add` backward has an `i32` index operand beside `f16` data.

### 9.5 Convolution

Forward (catalog form): `Y = matmul(im2col(X), W)`. Backward from `g`:

```
dW = matmul(im2col(X)^T, g)                    # weight gradient (operand view, item 6)
dX = scatter_add(zeros_like(X), index_map, matmul(g, W^T))   # col2im via the im2col index map (item 3)
db = reduce(sum, g, over batch and spatial axes)
```

Padding taps are skipped in `dX`; stride and dilation are already in the index map. The `index_map` is the
closed-form `base(patch) * stride + tap * dilation - padding` of the `im2col` decomposition, built from
`iota` (ruling 422). **Gaps hit:** items 3, 5 and 6. The padding change in the next Breaking batch
(`padding_lo`/`padding_hi`) changes `index_map` only by its offset term.

## 10. Decisions this note needs from CireSnave

The principle is ruled. Each open point is a question with its choices (recommendation first).

1. **Where is a backward *defined* normatively?**
   - (a) KISS-Ops, beside the reference decomposition; Contract only says how a kernel provides it. **Recommended.**
   - (b) KISS-Contract per kernel only (each kernel then restates the math: many copies).
2. **How are saved values supplied?** (§5)
   - (A) Named, defined by expression, emitted or recomputed per the kernel's Contract. **Recommended.**
   - (B) Always recompute. (C) Provider-private.
3. **Piecewise-constant ops (floor, ceil, trunc, round_even, step, sign):** (§4)
   - (a) `no backward` for all of them. **Recommended.** (b) zero gradient.
4. **Tie convention for max-like ops:** (§6)
   - (a) lowest index takes the whole gradient. **Recommended.** (b) split equally. (c) every tied element gets it all.
5. **Default accumulator for backward sums:** (§7)
   - (a) at least `f32` whatever the forward used. **Recommended.** (b) inherit the forward's. (c) a new declared coordinate.
6. **How to close the catalog gaps** (§8):
   - (a) add `digamma`, give `scatter` an accumulator field and a deterministic combine, express the slice and `col2im` as views and `iota`+`scatter_add` rather than new ops. **Recommended.**
   - (b) mint `slice` and `col2im` ops as well (shorter backward sequences, a bigger catalog).
7. **Multi-output backward regions** (§3.2):
   - (a) one region per output gradient, outputs bundled by the Contract. **Recommended.**
   - (b) extend the Grammar to a forest of roots. (c) do not advertise them.
8. **Complex gradient convention:** which derivative is "the gradient" for complex values?
   - (a) defer until a complex training consumer exists. **Recommended.**
   - (b) rule it now with the conjugate-Wirtinger convention.

Order of work if approved: items 2 and 4 before any clause text (they shape every section); the §8 table
becomes a checked list in the conformance tooling (a closure test that fails on a missing op); no byte
of the `structure_key` moves for this RFC (a backward is an ordinary op sequence with its own cell).

*— lane: kiss*
