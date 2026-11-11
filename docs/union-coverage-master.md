# KISS verdict column for the Unpopped union master list (R5.7)

**Informative. No normative text; no spec change is made here.** Requested by CireSnave 2026-11-07 via the PM:
*"Please do. We need the full list before we have a well documented plan for how to move forward in Unpopped's IR or KISS."*
His standing requirement (2026-10-28, via the PM): *"I don't want KISS to be unable to describe something Unpopped can build."*

**Keying.** Rows use Unpopped's grid ids (`A1`..`H6`, `docs/ir-union-capability-grid.md`, 46 rows, plus the new rows C4b, F9 and I1; the Unpopped lane owns the
master list `docs/ir-union-master-capabilities.md` and says new rows get new ids and none is renumbered). This file is the KISS
column only, so there is no second table to drift. It **replaces `docs/union-coverage-unpopped.md` as the view** (that file's
row numbers 1-38 map onto these ids in Part 3; **the Unpopped lane confirmed that mapping on 2026-11-11 at KISS `3fa8d11` (five corrections applied below), and the old file is retired**: its last text is `git show 3fa8d11:docs/union-coverage-unpopped.md`).

**Read at** KISS `origin/main` `d47b99e` (conformance 0.8.1: `cast` and `concat` landed, `MAX_OPERANDS` 16, `sk7`). Spec text
quoted from that ref. **Unpopped's side is not read by me**; its statements come from its grid and messages and are marked **[U]**.
The Unpopped lane has promised verified cells for the R5.7 rows; where its cell disagrees with a verdict here, its cell is the
input and this column is re-read, not the reverse.

| Verdict | Meaning |
|---|---|
| **EXPRESSED** | KISS says it today; the clause is named. |
| **BY COMPOSITION** | KISS says it as a DAG of other ops; there is no op token for the whole, so no fusion identity for it. |
| **NEEDS** | KISS lacks one named field, enum value, op or limit (named in the row). Options follow in Part 2. |
| **NO WAY** | KISS has no construct and the gap is not one field. |
| **SCHEDULE-ONLY** | Not "what is computed": a placement, ordering or instruction choice. KISS does not describe it (KISS-OPS "is NOT a kernel implementation or a lowering pipeline", Ops Purpose; Umbrella §2 "silence is not an implicit inclusion"). Where it changes a float result, the only KISS fact is the determinism class (6.0-0004, 6.17-0008). |

Rows whose verdict depends on the **kernel-body scope note** (draft due 2026-11-13: how much of a kernel body KISS describes) are marked
**[scope]**; their verdict is provisional until that note is ruled.

## Part 1. All 49 rows (the 46 grid rows plus C4b, F9 and I1)

| ID | Capability | KISS verdict | Clause / basis | Gap or note |
|---|---|---|---|---|
| A1 | Elementwise scalar expressions | EXPRESSED | Ops §2.7 catalog, §6.8 accuracy tiers, `select` raw-bit move; leaves 6.12-0001 | none |
| A2 | Locals, mutation, structured control flow | BY COMPOSITION for `if` (`select` over `cmp_*`) and for a bounded loop whose body is a fold (`reduce`, `prefix_scan`); **NO WAY** for data-dependent `while`/`break`/`return` **[scope]** | 6.11-0002, 6.11-0003; an op body is a pure DAG (Contract 6.4-0009) | no mutation, no early exit; a value-dependent trip count has no construct |
| A3 | Device function calls | BY COMPOSITION (a call graph without recursion is a DAG with shared subtrees, Grammar 6.4-0011) **[scope]**; recursion **NO WAY** | Grammar 6.4-0011 | no named, reusable function identity inside a body |
| A4 | Generics, comptime specialization | SCHEDULE-ONLY | specialization is the Classify cell (dtype, layout, target); Classify 6.6 | a template is a way to produce kernels, not a thing a kernel computes |
| B1 | Thread, lane, warp, block, grid ids | SCHEDULE-ONLY; the launch geometry a kernel *needs* is EXPRESSED as Dispatch derivation expressions | Contract 6.11-0008 (`invocation_domain`, `workgroup_sizing`, `count_to_grid`, `thread_mapping`, `addressing_rule`) | a kernel that reads its own lane id to change its result is not described, only its mapping |
| B2 | Block-level SPMD program ids | SCHEDULE-ONLY / EXPRESSED as `thread_mapping` | Contract 6.11-0008 | none |
| B3 | Cluster scope, distributed shared memory | SCHEDULE-ONLY | not a computed value | **unverified:** whether the Dispatch launch-scalar vocabulary can name a cluster dimension; if a consumer needs it, that is NEEDS (one symbol) |
| B4 | Persistent kernels, tile schedulers | SCHEDULE-ONLY (a permitted reordering) | determinism class 6.0-0004, 6.17-0008 | none |
| C1 | Shared / workgroup memory | SCHEDULE-ONLY | not a computed value | none |
| C2 | Register fragments | SCHEDULE-ONLY | not a computed value | none |
| C3 | Typed load/store, byte-offset, bitcast | EXPRESSED (access = signed strides and base offset); **NEEDS** `bitcast` | Contract 6.5-0005; Ops Appendix D item 12 lists `bitcast` as an open value | `bitcast` has no op or semantics (Part 2, G3) |
| C4 | Masked / predicated load-store with fill value | EXPRESSED for fill `{skip, clamp, zero}` on a gather; BY COMPOSITION for any other fill | `oob_policy` `{1=skip, 2=clamp, 3=zero-fill}` 6.19-0015, `gather` 6.11-0004; arbitrary fill: `select(cmp(coord…), x, const)` over `coord`/`const` 6.12-0001 | **R5.7 row.** No non-zero fill value on the access itself; see Part 2 |
| C4b | Padding (new row, Unpopped) | **NEEDS** separate low and high; fill value is C4 | one symmetric `u32` per axis 6.19-0023/6.19-0032/6.19-0033; Appendix D.11 (d) | causal and SAME padding not sayable; see Part 2 |
| C5 | Gather / scatter | EXPRESSED | 6.11-0004/-0005/-0006/-0010, 6.19-0027/-0028 | none |
| C6 | Textures, samplers, images | NO WAY | no construct in any sub-standard | filtering, wrap and normalised coordinates are a computed behaviour with no op |
| C7 | Structs, arrays, vectors, matrices as values | NO WAY as a value type; vector width is a derived layout fact | Classify 6.5-0009 | **[scope]** same question as G2 |
| D1 | Hierarchical layout algebra | EXPRESSED as an **output's** layout guarantee (the nested `shape:stride` form); an input's nested layout is not stated (flat strided access only); every stride leaf is an integer | Contract 6.5-0015, 6.5-0017 (0.12.0, board 378 "B"/"393: i"); flat access 6.5-0005 | a run-time stride is `strided(...)`, so nesting and a run-time leading dimension cannot be combined in one layout (CireSnave, 393: i); input-side nested layouts are not stated (re-read 2026-11-09 at `04083516`) |
| D2 | Swizzle | EXPRESSED as an **output's** layout guarantee, `swz(B,M,S)(shape:stride)`; shared-memory placement stays SCHEDULE-ONLY | Contract 6.5-0018 (0.12.0) | `BadSwizzle` validity (`\|S\| >= B`, `B+M+\|S\| <= 62`) is Unpopped's, not checked in KISS; Unpopped confirmed the text round trip against its `Layout` (Unpopped#318); an input's swizzle is not stated (re-read at `04083516`) |
| D3 | Reductions | EXPRESSED | `reduce` 6.11-0002, `reduce_*`/`argmax`/`argmin`/`any`/`all` §6.13 | none |
| D4 | Custom combine function or tuple state | **NEEDS** (closed monoid set); `argmax`/`argmin` BY COMPOSITION | monoid set `{sum, prod, max, min}` 6.11-0002 | **R5.7 row**; Appendix D.11 (f) held for this; see Part 2 |
| D5 | Scans | EXPRESSED | `prefix_scan` 6.11-0003, 6.19-0026 | none |
| D6 | Sort, top-k | EXPRESSED (sort, trailing axis, stable); top-k BY COMPOSITION | 6.11-0014, 6.11-0019, 6.19-0029 | sort along a non-trailing axis; no slice op |
| E1 | Block / named barriers | SCHEDULE-ONLY | not a computed value | none |
| E2 | mbarrier split arrive/wait | SCHEDULE-ONLY | not a computed value | none |
| E3 | Fences, scoped memory model | NO WAY (a memory model) | none; sidestepped for the four scatter combines by the order-invariant class | matters only if a result depends on ordering |
| E4 | Atomics | EXPRESSED (add, max, min, assign); **NO WAY** (CAS, exchange, returned old value) | 6.11-0005, 6.11-0006, 6.11-0010 | a returned old value is order-dependent: a new determinism class, not one more enum value |
| E5 | Warp / subgroup collectives | EXPRESSED (`any`/`all`, reduce, scan); BY COMPOSITION (shuffle, broadcast = `gather` over a lane axis); **NO WAY** (ballot mask, match, elect) | §6.13, 6.11-0004 | no lane concept: a lane would be an axis of the problem |
| E6 | Block-level collectives, cooperative groups | EXPRESSED where the effect is a reduce or scan; cooperative groups NO WAY | 6.11-0002, 6.11-0003 | a grid barrier is schedule |
| F1 | Tensor-core MMA atoms | `matmul` EXPRESSED; the instruction is SCHEDULE-ONLY | §6.13 `matmul`; Classify 6.6-0010 | none |
| F2 | Mixed-dtype, block-scaled, 2:4 sparse MMA | mixed dtype: BY COMPOSITION now (`cast`, then `matmul`) but the `matmul` node has no attrs (see F3); block-scaled EXPRESSED; 2:4 sparse **NEEDS** | `cast` 6.21-0001..0003; `quant` record Classify 6.3-0009a; MX types Ops 6.16 | sparsity metadata is an operand of a new kind: see F9 |
| F3 | Accumulator-precision selection | mixed: see Part 2 | 6.17-0001/-0003/-0006/-0008, 6.19-0025, Classify 6.7-0012 | **R5.7 row** |
| F4 | Async copy, TMA tile/bulk/multicast/reduce/im2col, ldmatrix | SCHEDULE-ONLY (a copy computes nothing); the im2col *gather pattern* EXPRESSED | `im2col` 6.19-0033 | `output_column_ordering` pin (Appendix D.11 (c)); TMA reduce-copy is a `scatter` combine |
| F5 | Software pipelining, warp specialization | SCHEDULE-ONLY | not a computed value | none |
| F6 | EVT epilogue trees, grouped/ptr-array GEMM, split-K / stream-K | EVT BY COMPOSITION (an epilogue is a body over the contraction output and extra inputs); split-K/stream-K SCHEDULE-ONLY; grouped / ptr-array **NEEDS** | 6.14-0006, 6.12-0001, 6.0-0004 | grouped GEMM has a per-group extent known only at run time: no construct |
| F7 | Implicit-GEMM convolution | BY COMPOSITION (`im2col` then `matmul`), with the C4b padding and operand-count gaps | §6.13 `im2col` row | **R5.7 row**; see Part 2 |
| F8 | Attention | BY COMPOSITION (`matmul`, scale, mask over `coord`, `softmax`, `matmul`); paged KV is a `gather`; backward: no op | §6.13, 6.11-0004; online softmax is a reordering inside 6.0-0004 | no attention op token, so no fused identity; backward waits on the `rfc/backward-op-category` branch (not merged) |
| F9 | MMA operand count and kinds (new row, Unpopped) | count EXPRESSED up to 16; operand *kinds* (scales, sparsity metadata) partly | `MAX_OPERANDS` 16 Classify 6.4-0002 (sk7, 0.8.0), Ops 6.19-0037; block scale operand Classify 6.6-0022 | **R5.7 row**; see Part 2 |
| G1 | Narrow float dtypes | EXPRESSED | Ops 6.16 table; Classify 6.1-0013 | none |
| G2 | Packed / vector value types | NO WAY as a value type | vector width is a layout fact, Classify 6.5-0009 | **[scope]**; the open question is whether KISS must name a lane-typed value or may treat it as layout |
| G3 | Cast with rounding and saturation | EXPRESSED, with open values | `cast` 6.21-0001..0004 (nearest-even and three directed modes, saturation `none`/`clamp`); 0.8.1 | **R5.7 row**; Appendix D item 12 values unpinned (6.21-0004); `bitcast` has no op |
| G4 | Fma, rounding mode, ftz, approx vs exact | `fma` **NEEDS**; approximate form EXPRESSED within declared-ULP tiers; FTZ/DAZ is a decision already taken (not advertisable, 6.17-0002) **but Unpopped builds it** (`Numerics{round, ftz, approx}` [U], Unpopped#314), so against its IR it is a NEEDS that needs a ruling to reverse; directed rounding on arithmetic NO WAY today | 6.17-0002 (bit-stable forbids contraction), 6.17-0003, 6.17-0004 (per-op refinement) | **R5.7 row**; see Part 2 |
| G5 | NaN/Inf-preserving semantics control | EXPRESSED (NaN-propagating `max`/`min`, IEEE arithmetic); a "no-NaN" fast-math assumption is **NO WAY** unless MathFidelity gains a value | 6.11-0002, 6.11-0010, 6.17-0002 | a decision, not an oversight; needs a ruling to reverse |
| H1 | Inline asm, extern libraries, escape hatch | EXPRESSED | a hand-written kernel degrades to a declared op-identity tag plus a conformance claim: `declared-op-tag` (contract.md lines 186 and 330) | no semantics inside the tag |
| H2 | Automatic differentiation | NO WAY | no backward op; the `rfc/backward-op-category` branch is unmerged | an op category decision, not a field |
| H3 | RNG, device assert, device print | RNG BY COMPOSITION (counter-based Philox/Threefry over integer `mul`/`xor`/`shl`/`shr`, `coord`, `param`); assert/print **NO WAY** | 6.2-0002, 6.12-0001 | no named, versioned generator identity; side-effecting ops have no construct. **[U]** Unpopped's `dropout_fw` takes the random tensor as an input, so parity needs no RNG |
| H4 | Autotuning configs as schedule hints | SCHEDULE-ONLY | not a computed value | none |
| H5 | Multi-GPU, peer pointers, multicast | NO WAY | a cross-device access has no construct | **[scope]** |
| H6 | Emits to or parses from multiple targets | EXPRESSED | the purpose of KISS-Emit and KISS-Consume (umbrella §2) | none |
| I1 | Normalisation family as an operation (new row, Unpopped): softmax, log_softmax, rms_norm, layer_norm over several trailing axes, affine gamma/beta | EXPRESSED | `softmax`/`log_softmax`/`rms_norm`/`layer_norm` §6.13 (6.13-0004, 6.13-0008); `norm_axes` is a `u16` axis mask, so several axes are one op (6.19-0038); `eps` is `param(0)`, `gamma`/`beta` are operands `input(1)`/`input(2)` | read at `04083516` (the rest of this file is read at `d47b99e`); not a primitive, a non-primitive op with a reference decomposition |

## Part 2. NEEDS rows: options

**Format** as in the R5.7 note: what is missing, the options, a recommendation, whether it changes bytes. Nothing here is a clause.
Options that change bytes are batched per the PM's 0.9.0 ruling and need CireSnave's word.

### F3. Accumulator precision

| Case | Verdict | Basis |
|---|---|---|
| TF32 | EXPRESSED | `reduced-mantissa-permitted:<n>` with the declared width, TF32 = 10: 6.17-0001, 6.17-0003, 6.17-0006 |
| f16 accumulate (f16 compute) | EXPRESSED on `reduce`/`prefix_scan`: it is the diagonal cell | 6.17-0008, 6.19-0025 (`accumulator` = `0x00`) |
| fp8 compute, fp16/f32 accumulate | EXPRESSED on `reduce` (a wider float accumulator is a distinct tolerance cell) | 6.17-0008 |
| fp8 **fast-accumulate** (an internal accumulator narrower than f32 that is not a named dtype) | NEEDS | a declared width via `reduced-mantissa-permitted`, but the *accumulator's own* rounding width is not a field: `accumulator` names a dtype only |
| **int32 accumulate** | **NEEDS** (banned) | "The accumulator dtype MUST be a **float** dtype at least as wide as the compute dtype": 6.17-0008; `reduce` `accumulator` "MUST be a float", 6.19-0025, Classify 6.7-0012. One integer precedent: `b1` xor+popcount to `s32` 6.16-0006 |
| 3xTF32 | NEEDS | three-term split structure (hi·hi + hi·lo + lo·hi, **[U]** CUTLASS definition, not an Unpopped fact), not a width `n` |
| `matmul` cannot request any of this | NEEDS | the `matmul` row of §6.13 has no OpAttrs (D.11 (e)); `reduce` has `accumulator` and `math_fidelity` |

Options:

- **A. Give `matmul` the `reduce` precision pair** (`accumulator`, `math_fidelity`, same order), and relax 6.17-0008/6.19-0025 for an
  **integer** accumulator only when the compute dtype is an integer type and the accumulator is `s32` (or wider signed), with the
  reference value defined as exact integer arithmetic (no rounding, so it is bit-exact and *not* a tolerance cell). Adds a determinism
  statement: an integer reduction is order-invariant and exact (wrapping `add` 6.2-0002 is associative), which is a *stronger* class
  than the float cell, not a weaker one.
  *Bytes:* adds two fields to `matmul` (ABI order; D.11 says field order is ABI), so Breaking; the accumulator ordinal domain widens.
- **B. Keep the float-only rule and describe an integer GEMM as `reduce(sum)` of `cast`-to-`s32` products.** `cast` (6.21) exists now,
  so this is expressible today for the reference value. Costs: no fusion identity, and the 6.17-0008 prohibition then reads as
  "not in a *contraction* tolerance cell" while the integer composition is accepted: an inconsistency a reader has to be told about.
  *Bytes:* none.
- **C. Do both:** B now (it needs no ruling beyond stating the integer fold exact), A with the 0.9.0 batch.

Recommendation: **C**. B is correct today and costs nothing; A is needed only if a registry must recognise an integer contraction as one
op. The 3xTF32 and fp8 fast-accumulate rows stay NEEDS-with-no-recommendation: whether they are a width, a structure, or a declared
tag is a decision, not a measurement. Say so to CireSnave; do not default it.

### G3. Cast with rounding and saturation (landed; the open values)

`cast` landed in 0.8.1 (6.21). What Unpopped can build and KISS does not pin: NaN into a format with no NaN, NaN into an integer,
an out-of-range float into an integer, overflow into an IEEE float, `bitcast`, `cast` into `bf16` with a non-nearest mode (Appendix D
item 12, 6.21-0004). Options: **A.** adopt Unpopped's implemented behaviour (its PR #144) as the pinned value for each; **B.** leave
unpinned and make the reference decline, which is what 6.21-0004 says today; **C.** pin the integer cases and leave `bitcast` as its own op.
Recommendation: **C**. A cast into an integer must not be undefined in a conformance reference. `bitcast` is a different operation (no
rounding, a reinterpretation) and should be its own atom or attribute, not a rounding mode. *Bytes:* A and C add result rules only,
no wire change; a `bitcast` atom changes the floor (6.3-0001).

### G4. Fma, rounding mode, approximate form

`fma` needs a primitive-floor atom: `mul` then `add` double-rounds, so it cannot be a decomposition. FTZ/DAZ: "No target-token field and
no MathFidelity value advertises flushing" (6.17-0002) is a decision already taken. Options: **A.** add `fma` to the floor (6.3-0001) with
single-rounding semantics; **B.** no `fma` atom, and a fusing kernel declares `reduced-mantissa-permitted` (6.17-0003) as today;
**C.** as A, plus a per-op rounding qualifier reusing `cast_rounding` (6.19-0041) for `fma` and the arithmetic atoms.
Recommendation: **A**, only `fma`: it is the one that cannot be composed. Directed rounding on every arithmetic op (C) is a larger
change with no kernel in the survey that needs it. *Bytes:* A adds an op token and an ordinal (a floor change).

### D4. Custom reduction combiner and tuple state

The closed monoid set is `{sum, prod, max, min}` (6.11-0002). `argmax`/`argmin` are BY COMPOSITION today. Not expressible: a reduction
whose combine is a user function, or whose state is a tuple (Welford mean/variance, online softmax's `(m, l)` pair, top-k with indices).
Held for this task: board 304 Q4, Q5, Q6, Q8 and D.11 (f) (`monoid` is an identity-bearing OpAttrs field, not folded into op tokens).
Options:

- **A. Closed set, extended by definition.** Add named monoids (argmax with a tie rule, Welford, online-softmax) as new `monoid`
  ordinals, each a KISS clause. Closed, testable, no arbitrary code. Cost: a new ordinal per registered combiner, and a registered
  combiner is not necessarily associative (Welford's merge is, floating-point online softmax is only in the order-invariant class).
- **B. A combine body.** `reduce` takes a sub-DAG as its combine function and a tuple-valued state. Most general; needs a node kind for a
  sub-DAG with its own leaves, tuple-typed values (no tuple dtype exists, see G2), and an associativity claim KISS cannot check.
- **C. Composition only.** A tuple state is parallel `reduce` ops over the same axis, a combiner is a decomposition into the four monoids
  (what `softmax` and `argmax` already are); no change.

Recommendation: **A for the tuple monoids the survey actually needs (argmax/argmin, Welford, online softmax), nothing for B.** B is
the escape hatch KISS already has under H1 (`declared-op-tag`), and it needs a tuple type that does not exist. *Bytes:* A adds
`monoid` ordinals (additive); B changes the node schema (Contract 6.4-0009), Breaking.

### C4 and C4b. Masked load fill, padding

Fill `{skip, clamp, zero}` is the frozen `oob_policy` (6.19-0015; pad-fill IS `oob_policy`, Grammar 6.2-0004). An arbitrary fill value is
`select(cmp(coord…), x, const)` today (a composition). Padding is one symmetric `u32` per spatial axis on `avg_pool`, `max_pool`,
`im2col` (6.19-0023/-0032/-0033); causal conv1d needs left-only and TensorFlow SAME needs unequal low/high. Options:
**A.** `padding_lo` and `padding_hi` vectors replace `padding` (Breaking, ABI order); **B.** keep `padding`, add an optional `padding_hi`;
**C.** express asymmetric padding as `concat` with a constant slab (0.8.1 `concat` + `const` leaf): no op change, an extra copy in the
reference. For a **non-zero fill on the access itself**: **D.** a fourth `oob_policy` value `4=fill` plus a `fill_bits` field (Breaking).
Recommendation: **A** for padding (Unpopped's `Window` already has `pad_lo`/`pad_hi` [U], and the pair also fixes pools); **no** change for fill
(composition suffices until a kernel shows the cost). *Bytes:* A and D are Breaking (field order is ABI); they ride the 0.9.0 batch.

### F4 and F7. Async copy, implicit-GEMM conv

Async copy is SCHEDULE-ONLY; the only KISS-visible fact is `im2col`'s `output_column_ordering` set `{1=channel-major, 2=tap-major}`, "pending normative
pinning" (Appendix D.11 (c)). **[U]** Unpopped's `Im2Col` is 2-D, groups 1, symmetric padding, row = `c*kh*kw + ki*kw + kj`, i.e. ordinal `1`.
Implicit-GEMM conv: BY COMPOSITION (`im2col` then `matmul`), correct as a reference, with the C4b padding gap and, for a conv with bias and
activation epilogue plus scales, the operand-count question (now 16, F9). Options: **A.** pin `{1, 2}` now (a two-value enum, additive,
Unpopped needs only `1`); **B.** add a `conv` op token whose decomposition is `im2col` then `matmul` (gives a fusion identity; a floor-neutral
non-primitive op). Recommendation: **A now; B only if a registry needs a conv identity** (extend-by-definition model, Ops 2.2). *Bytes:* A none
beyond a value pin; B adds a non-primitive op (floor unchanged).

### F9. MMA operand count and kinds

Count: `MAX_OPERANDS` is 16 since 0.8.0 (Classify 6.4-0002, Ops 6.19-0037); a 9-operand kernel is expressible. Kinds: a block scale is a designated
operand (Classify 6.6-0022); **2:4 sparsity metadata** (an index/bitmask operand that selects which elements of the compressed operand are kept)
has no operand role. Options: **A.** a `sparsity_meta` operand role with a named layout (`2:4`) in Classify (a key field, so a schema event);
**B.** describe the sparse operand as dense plus a mask (`select` over a metadata gather): correct reference, no fusion identity, and it loses the
fact that the kernel never reads the pruned half; **C.** defer. **Corrected 2026-11-07 after the Unpopped lane's verified cells (Unpopped#314):** Unpopped already has `MmaAtom` `sparse_24` and `block_scale` extra operands, so this is a construct Unpopped builds and KISS cannot name; **C is withdrawn**. Recommendation: **A**, a sparsity-metadata operand role (and its selector) with a named `2:4` layout. *Bytes:* A is a `structure_key` field, so a schema event for the 0.9.0 batch if CireSnave rules it in.

## Part 3. Mapping from the previous view

| Previous row (union-coverage-unpopped.md) | Master id |
|---|---|
| 1 contraction, plain | F1 |
| 2 mixed dtype | F2, F3 |
| 3 TF32/3xTF32; 4 int32; 5 f16/fp8 accumulate | F3 |
| 6 reduction | D3 |
| 7 RowReduce second input (gamma/beta on `rms_norm`/`layer_norm`) | D3, I1 |
| 8 multi-output | F9 (operand count; `sort_network`'s two outputs also touch D6). Not C5: gather/scatter has no multi-output |
| 9 view | D1, C3 |
| 10 gather; 11 scatter | C5 |
| 12 scan; 17 sort; 18 top-k | D5; D6 |
| 13 pooling | D3, C4b (padding). **Pooling has no master row of its own**; it is mapped to the reduction row plus the padding row |
| 14 causal conv1d; 15 implicit GEMM; 16 im2col | C4b, F7, F4 |
| 19 offset | A1 (`coord`) |
| 20 dropout | H3 |
| 21 block-dequant | F2, G1 |
| 22 attention; 23 EVT; 24 split-K | F8; F6; B4 |
| 26-28 atomics | E4, E3 (row 28, order/scope: E3, and H5 once Unpopped U12 lands System scope; S2 was in flight, not on its main at `cbf2e882`) |
| 29-32 cooperative groups, subgroup ops | E5, E6 |
| 33 cast; 34 fma; 35 qualifiers | G3; G4; G4 only. G5 (NaN/Inf control) is not a qualifier row and has no old row |
| 36 integer arithmetic; 37 packed lanes; 38 new dtypes | A1; G2; G1 |

## What this file does not claim

Every verdict is my reading of KISS at `d47b99e`; each EXPRESSED and BY COMPOSITION cell is backed by a clause named in the row, not by a test I
ran against Unpopped's IR. The rows marked **[scope]** change if the kernel-body scope note is ruled differently. No count above is a claim of
completeness: the 46 rows are Unpopped's grid, whose own limits (one shared brief, no cell checked against primary documentation) it states.
