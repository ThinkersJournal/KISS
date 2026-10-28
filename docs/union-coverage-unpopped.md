# Union coverage: can KISS describe everything Unpopped can build? (R5.7)

**Informative. No normative text; no spec change proposed here.** For CireSnave's R5.7 decision (board 304).
His standing requirement (2026-10-28, via the PM): *"I don't want KISS to be unable to describe something Unpopped can build."*

Read at KISS `origin/main` `bafbe07` (conformance 0.6.19). Unpopped's side is **not read by me**: the family list is the PM's
task text; every statement about Unpopped's IR below is from the Unpopped lane's own message of 2026-10-28 (its `origin/main`
`41cd411`), marked **[U]**. A row that has no **[U]** has not been checked against Unpopped's IR and is waiting for that lane.

## How to read a row

| Verdict | Meaning |
|---|---|
| **EXPRESSED** | KISS says it today; the clause is named. |
| **BY COMPOSITION** | KISS says it as a DAG of other ops; there is no op token for the whole, so no fusion identity for it. |
| **NEEDS** | KISS lacks one named field, enum value, op or limit. The cell says which. |
| **NO WAY** | KISS has no construct and the gap is not one field. |

Scheduling choices (split-K, stream-K, tiling) are not "what is computed"; KISS does not describe them (Ops §1, Emit). Where they
change a float result, the only KISS fact is the determinism class (§6.0-0004: float `sum`/`prod`, scan, contraction are
order-invariant/nondeterministic), so they are EXPRESSED as *a permitted reordering*, not as a feature.

## Part 1. Families

| # | Family | Verdict | How KISS says it today (clause) | What it lacks |
|---|---|---|---|---|
| 1 | Contraction, plain | EXPRESSED | `matmul` (Ops §6.13 table, operands `input(0)=lhs`, `input(1)=rhs`); the `ctr` cell and its role hints: M/N/K by caller (Classify 6.6-0016), weight role (6.6-0019), batch axes (6.6-0023); output shape from the roles (Ops 6.20-0008) | none |
| 2 | Contraction, **mixed dtype** | **NEEDS** | Key side: the `ctr` contraction field carries weight, accumulator and output dtype (Classify 6.6-0010, 6.6-0015, grammar `c<m><n><k>/<kdiv>/<wdt>/<acc>/<out>/<mp>`). | Ops side: `matmul` is `reduce(sum) of element_map(mul(input(0), input(1)))` over **one compute dtype**, and the op catalog (§2.7, §6.1) has **no cast/convert op**, so nothing in Ops says what multiplying `f16 x f8e4m3fn` means or where the rounding to the output dtype happens. See Finding F1. |
| 3 | MMA / AccumSpec: **TF32**, **3xTF32** | EXPRESSED (TF32), **NEEDS** (3xTF32) | MathFidelity `reduced-mantissa-permitted:<n>` with the kernel's declared width (Ops 6.17-0001/-0003/-0006, Contract 6.8-0004a); TF32 is the canonical `n = 10`. | 3xTF32 recovers near-fp32 by three TF32 products; whether that is "a width" (`n` about 21) or a different thing (three-term split, a different rounding structure, so a different reference value) is not a KISS question I can answer from the spec. **[U] to say what 3xTF32 computes.** |
| 4 | MMA / AccumSpec: **int32** accumulate | **NEEDS** | Only float accumulators exist: the accumulator MUST be a float at least as wide as the compute dtype (Ops 6.17-0008; `reduce` field `accumulator` "MUST be a float", 6.19-0025). The one integer precedent is `b1` xor+popcount to `s32` (6.16-0006). | An integer accumulator for `i8 x i8` (and `u8`, `i4`): needs an integer-accumulator rule, plus the cast/mixed-dtype fix of row 2, because `mul` on `i8` wraps in `i8` (6.2-0002). |
| 5 | MMA / AccumSpec: **f16 / fp8-fast** accumulate | **NEEDS** | `reduce` and `prefix_scan` carry `accumulator` and `math_fidelity` (6.19-0025/-0026); the `ctr` key carries `<acc>` (Classify 6.6-0010). | `matmul` carries **no OpAttrs**, so an Ops node cannot *request* an accumulator or fidelity; only the key and the Contract declare one. The asymmetry is the gap (see Finding F2). |
| 6 | Reduction | EXPRESSED | `reduce` (6.11-0002, 6.19-0025) and the `reduce_*`/`logsumexp`/`argmax`/`argmin`/`any`/`all` family (§6.13) | none |
| 7 | RowReduce **second input** | EXPRESSED, **[U] to confirm** | A `reduce` pre-map or epilogue is an `element_map`-style body that reads `input(i)` for any `i` (Ops 6.12-0001, 6.14-0006), so a second input is a second leaf. `rms_norm`/`layer_norm` already read `gamma`/`beta` as extra inputs (6.19-0031). | none known; depends on what "second input" means in Unpopped's IR. |
| 8 | Multi-output | EXPRESSED | Outputs are operands, listed after inputs in canonical order (Classify 6.6-0014); `sort_network` already has two outputs (values and an `i64` index vector, 6.11-0019). **[U]** its cap is `n_inputs + n_outputs <= MAX_OPERANDS` (`ir.rs:2974`). | the cap, see Flag 1 |
| 9 | View (transpose, broadcast, reverse, slice, offset) | EXPRESSED as layout, **NEEDS** as an op | A view of an operand is its signed strides (`0` = broadcast, `< 0` = reversed) and a per-operand base offset in the Contract signature (Contract §6.5-0005 and the launch-scalar table); `reduce` keeps extent-1 views (6.11-0008). | The shape oracle names `transpose`, `unsqueeze`, `concat`, `cast` (Ops 6.20 preamble) but none is in the §2.7 catalog, the §6.1 registry or the §6.19.3 schema, and `permutation` is "frozen and reserved, attached to NO op" (6.19-0024). A view that is a *node in the DAG* (not a stride on an operand) has no op. See F3. |
| 10 | Gather | EXPRESSED | `gather`, `index_select`, `embedding` (6.11-0004, 6.19-0027/-0034) | none |
| 11 | Scatter | EXPRESSED | `scatter` with combine `{assign, atomic-add, atomic-max, atomic-min}`, `scatter_add` (6.11-0005/-0006/-0010, 6.19-0028) | order/scope/old-value: row 28 |
| 12 | Scan | EXPRESSED | `prefix_scan` inclusive/exclusive over one axis (6.11-0003, 6.19-0026), `cumsum`/`cumprod`/`cummax` | none |
| 13 | Window: pooling | EXPRESSED | `avg_pool`, `max_pool` (6.19-0032), vectors for window/stride/dilation/padding (6.19-0023) | asymmetric padding: Flag 3 |
| 14 | Window: **causal conv1d** | **NEEDS** | Windowed contraction = `im2col` then `matmul` (BY COMPOSITION; there is no `conv` op, and Classify has no windowed-contraction cell: a `ctr` cell's roles are M/N/K/batch only). | (a) **left-only padding**: Ops `padding` is one `u32` per axis, applied both sides (Flag 3); (b) **9 operands** against `MAX_OPERANDS = 8` (Flag 1). **[U]**: `causal_conv1d` itself is deferred in its IR (needs a weight operand = a windowed contraction, `ir.rs:2142`); its `Window` has `pad_lo`/`pad_hi` (`ir.rs:2160`). |
| 15 | Implicit-GEMM conv | BY COMPOSITION, with row 14's two gaps | `im2col` + `matmul` is the reference semantics (6.13 `im2col` row); strides and dilation are in the window vectors. | the same padding and operand-count gaps; no conv op token or cell. |
| 16 | `im2col` | EXPRESSED, one pin open | `im2col` (6.19-0033) | `output_column_ordering` set `{1, 2}` is "pending normative pinning" (App. D.11 c) |
| 17 | Sort | EXPRESSED | `sort_network`, stable, trailing axis, ascending or descending (6.11-0007/-0014/-0019, 6.19-0029) | sort along a non-trailing axis resolves to the trailing axis only (6.11-0014): **[U] to confirm Unpopped never sorts another axis** |
| 18 | Top-k | BY COMPOSITION | `sort_network` (descending, stable) then a leading-`k` slice (a view, row 9). | no `topk` op token; no slice op (row 9). The result is correct because the sort is stable; a *partial* selection is not distinguishable from it, which is fine for a conformance reference. |
| 19 | Offset (iota / base offset) | EXPRESSED | `coord(axis)` leaf (6.12-0001); the shape oracle's slice/iota `DimExpr` (6.20-0007); per-operand base offsets (Contract) | none |
| 20 | Dropout | BY COMPOSITION, naming gap | No random op or leaf exists (the only "random" in the specs is the Conform fuzzer). A counter-based generator (Philox, Threefry) is integer wrapping `mul`/`xor`/`shl`/`shr` over `u32`/`u64` with `coord(axis)` and a `param(i)` seed, all in the primitive floor, so it is expressible as an `element_map` body. | no *named* generator: two kernels using different generators look the same to a registry. A versioned RNG identity (a leaf or a registered body) is the missing piece. **[U] to say which generator and what determinism it promises.** |
| 21 | Block-dequant read-through | EXPRESSED, **[U] to confirm the forms** | The operand's `quant` record `{family, sub_byte_bits, block_elems, scale_placement, dequant_form}` (Classify §6.3-0009a, `quant` row), the designated scale operand and its dtype keyed from `sk6` (6.6-0022); MX element types `f4e2m1`/`f6e2m3`/`f6e3m2`, scale types `f8e8m0`/`f8e6m2` (6.1-0013, Ops 6.16). | the dequant arithmetic is whatever `dequant_form` names; a form Unpopped needs and Classify's registry lacks would be a NEEDS. Unpopped to list its forms. |
| 22 | Attention | BY COMPOSITION | `matmul`, scale, mask (`cmp_*`/`select` over `coord`), `softmax`, `matmul` (§6.13); online-softmax/flash is a reordering of the same sums, in the order-invariant class (6.0-0004). | no attention op token, hence no fused-attention identity. Accepted by the "extend by definition" model (Ops 2.2). |
| 23 | EVT epilogue trees | EXPRESSED | An epilogue is a body of ops over the contraction output and extra inputs (6.14-0006, 6.12-0001); aux outputs are multi-output (row 8). | operand count (Flag 1); a *reducing* epilogue (a row-reduce of the accumulator tile) is a `reduce` over the `matmul` output, fine; **[U] to say if any EVT node is neither elementwise nor reduce** (e.g. a broadcast-of-row-max). |
| 24 | Split-K / stream-K | EXPRESSED as permitted reordering | the determinism class (6.0-0004, 6.17-0008) | none: a schedule |

## Part 2. Kernel-language union features that change what is computed

| # | Feature | Verdict | Today | What it lacks |
|---|---|---|---|---|
| 26 | Atomics: the four combines | EXPRESSED | `scatter` combine `{atomic-add, atomic-max, atomic-min}` and `assign` (6.11-0005); float `atomic-add` is order-invariant class (6.11-0006), float `max`/`min` NaN-propagating (6.11-0010) | none |
| 27 | Atomics: **old-value** (fetch-and-op), compare-and-swap, exchange | **NO WAY** | `scatter` returns the destination after all updates; there is no per-update return value, no CAS. | a returned-old-value form changes what is computed (it is order-dependent), so it needs a different determinism class, not one more enum value. **[U]: its IR has none today** (`WriteCombine::{Assign, AtomicAdd, AtomicMax, AtomicMin}`, `ir.rs:2609`). |
| 28 | Atomics: memory **order** and **scope** | **NO WAY** | none. **[U]: its IR has none either**, so today this is a gap in neither; it is a future row. | a memory model. Only matters if a kernel's *result* depends on ordering, which the order-invariant class sidesteps for the four combines. |
| 29 | Cooperative groups | **NO WAY** as a construct; EXPRESSED where the effect is a reduce or scan | none; a grid-wide barrier is a schedule fact. | **[U] to name any cooperative-group use whose result is not a `reduce` or `prefix_scan`.** |
| 30 | Subgroup ops: reduce, scan | EXPRESSED | `reduce`, `prefix_scan` (their scope is a schedule choice) | none |
| 31 | Subgroup ops: shuffle, broadcast | BY COMPOSITION | a `gather` with a lane index | no lane concept: the "lane" would be an axis of the problem |
| 32 | Subgroup ops: vote, ballot | EXPRESSED (`any`/`all`), **NO WAY** (ballot mask) | `any`, `all` (§6.13) | a ballot's bitmask output has no op |

## The three flags

**Flag 1. `MAX_OPERANDS = 8` against causal conv1d's 9 operands.** Confirmed as a hard limit, not a convention. It is pinned by
Classify 6.4-0002 (`MAX_OPERANDS` MUST equal `8`; a reader MUST reject a key declaring more), it bounds `n_operands` (a `u8`,
Classify 6.7), and it bounds the OpAttrs `index_operand` range (Ops 6.19-0027). **[U]**: Unpopped's IR has no 8 cap of its own; the
only cap is on its key and plan (`n_inputs + n_outputs <= MAX_OPERANDS`, `ir.rs:2974`; `structure_key.rs:51`). Ways out, none
chosen here: (i) raise the pinned value (a `structure_key` schema-version event, Classify 8-0002, so a new `sk` number and a
vendor regeneration); (ii) keep 8 and require a kernel with more operands to be described as a composition (this is what row 14
already allows: `im2col` and `matmul` each have at most 3 operands), at the cost of no fusion identity for the whole; (iii) a
variable-length operand list in the key. (i) is the smallest change if more than one real kernel passes 8.

**Flag 2. Can `matmul` with NO free OpAttrs (D.11 e) say transposed, batched, mixed-dtype, accumulate-dtype?**

| Variant | Verdict | Why |
|---|---|---|
| transposed (either operand) | **yes** | a transposed operand is the same bytes with swapped signed strides (Contract §6.5-0005), and the caller's role hints say which operand axis is K (Classify 6.6-0016, "an axis of an operand, not an iteration-frame axis"). No OpAttrs needed. |
| batched | **yes** | batch axes are a caller role hint (Classify 6.6-0023); a shared operand is stride-0 on the batch axis. |
| mixed-dtype | **no, at the Ops layer** | the key can say it (row 2) but Ops has one compute dtype per `mul` and no cast op. Not fixable by an OpAttrs field on `matmul` alone. |
| accumulate-dtype / fidelity | **no, at the Ops layer** | `reduce` has `accumulator` and `math_fidelity`; `matmul` has neither, and an integer accumulator is forbidden outright (6.17-0008). A `matmul` node cannot request what its key and Contract can declare. |

So (e) as written ("`matmul` carries NO free OpAttrs") is true for layout and false for precision. Either `matmul` gets the
two `reduce` precision fields (additive, and the key already carries them), or `matmul` is declared sugar for `reduce(sum)` over
`mul` and the precision is read off the decomposition. That is CireSnave's to pick; I recommend giving `matmul` the same
`accumulator` and `math_fidelity` fields in the same order as `reduce`, because the key already has both and a field that exists
in one place and not the other invites a silent default.

**Flag 3. Does (d) padding need separate low and high?** **Yes.** Ops `padding` is a single `u32` per spatial axis applied to both
sides (6.19-0023, 6.19-0032/-0033; App. D.11 d), so causal (left-only) padding cannot be said. **[U]**: its `Window` already has
`pad_lo`/`pad_hi` as `u8` (`ir.rs:2160`), and a left-only pool is `pad_hi = 0`, so the IR needs the pair for pools too, not only
for conv1d. The same applies to TensorFlow-style `SAME` padding. A change to the `avg_pool`/`max_pool`/`im2col` schemas is an ABI
change (field order is ABI, App. D.11), so it moves with a version.

## Findings the survey turned up

- **F1. Dtype-conversion has no op.** `cast` appears in the shape oracle (Ops 6.20 preamble) and as the one dtype-bearing node in a
  Conform signature ("a `Cast`'s target dtype rides inside that node's OpAttrs blob", `spec/conform.md:2060`),
  but it is not in the §2.7 catalog, the §6.1 registry, or the §6.19.3 schema table, so it has no pinned semantics (rounding mode,
  saturation, NaN) and no OpAttrs layout. This is the root of rows 2, 4 and 9, and of any epilogue that rounds to a narrower output.
- **F2. Precision fields exist on `reduce`/`prefix_scan` and on the `ctr` key, not on `matmul`** (Flag 2).
- **F3. Four shape-oracle names (`transpose`, `unsqueeze`, `concat`, `cast`) and a reserved `permutation` field describe ops that
  the catalog does not contain.** Either they are intended ops (then the registry is missing them) or they are examples (then
  Ops 6.20 should not name them).

## What I need, and from whom

- **Unpopped lane (3t8sgtiw):** check every row against your IR and mark it. Specifically the **[U] to confirm** rows (7, 17, 21,
  23), what 3xTF32 computes (row 3), which RNG generator dropout uses (row 20), and any cooperative-group or EVT use that is not a
  reduce/scan/elementwise (rows 23, 29). Do not take my reading of your IR as given; your five findings of 2026-10-28 are the
  only Unpopped facts used above.
- **CireSnave (via the PM):** D.11 (d) and (e) are held until this table is read, as asked. The table's answer is: (d) needs
  low/high; (e) is right for layout and wrong for precision. F1 (the missing cast) is a larger question than either and is not on
  board 304 today.

No count above is a claim about completeness: the family list is the PM's, not a survey of Unpopped's source.
