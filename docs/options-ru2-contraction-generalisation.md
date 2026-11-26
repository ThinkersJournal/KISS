# Options note: RU.2, generalising the Contraction schema and `StructureKey` for the IR union

**Informative. No normative text. For CireSnave's word on every byte-altering option (via the PM, board 501 covers the union-token side); nothing is minted by this note.**
Measured at `origin/main` `89a4ed81` (conformance 0.23.1; `conformance/src/derive.rs` is untouched on the branch), 2026-11-25, with the reference deriver
(`derive_structure_key`) on hand-built cells: every `RU2 ...` token below is its output, not a reading of the clauses. Source of the asks: Unpopped
`docs/ir-union-asks.md` section 1b (CuTe 39; CUTLASS 3, 4, 5, 15, 23, 31; CUDA 44, 55). Roadmap row: RU.2. Ownership proposed by Unpopped and accepted by the PM: KISS owns
the schema and the key, Fuel owns the planner and dispatch half.

## What the key already says (measured)

M=128, N=256, K=512, `f16`, `cuda:sm89`, M/N/K role hints on the row-major operands:

| Case | Result (contraction field and the operand sub-keys that changed) |
|---|---|
| A row-major, B row-major | `csmm/d16/f16/f16/f16/st`; sub-keys `co/00/v8` x3 |
| B column-major (strides `[1, K]`) | same contraction field; B's sub-key is `st/00/v1`, A and out stay `co/00/v8` |
| A column-major (strides `[1, M]`) | same contraction field; A's sub-key is `st/00/v1` |
| a 4th input, an `[M,N]` `f32` operand | derives (no decline); a 4th sub-key `co/00/v4/d16/f/f32`; contraction field unchanged |
| rank 3, one batch axis of 6 | `csmm/d16/bt/...`, `r3` |
| rank 4, batch axes 2x3, 3x2 and 1x6 | **the same token for all three**: `.../r4/...|-|csmm/d16/bt/...` |

The roles are `ContractionRoles { m: AxisRef, n: AxisRef, k: AxisRef, batch: Vec<usize>, weight, scale }`: **one** K axis, one M, one N, and a batch SET that
KISS-CLASSIFY-6.6-0023 collapses to the product of its extents. The weight dtype, accumulator, output and (sk6/sk8) scale dtype and block length are keyed (6.6-0022, 6.6-0022a, 6.7-0006).

## Group 1. Rank above 2, batched (CuTe 39): nothing to build

Rank 3 and rank 4 batched cells already derive and key (rows 5 and 6 above; `derivation_vectors.rs` has a rank-4 cell; 6.6-0023 pins "every cell of rank above 3").
"`Access::Contraction` is rank-2" is Unpopped's plan-layer type, not KISS's. **Recommendation: close CuTe 39 on KISS's side with one sentence in the ask's answer; Unpopped adds a rank-3 round-trip of its own.**

## Group 2. More than one batch role, nested modes (CUTLASS 3), more than one K axis (CUTLASS 4)

Measured: the key cannot tell 2x3, 3x2 and 1x6 batches apart, and a second K axis has no role (`k` is one `AxisRef`).
- **A. Collapse to a product, additively (recommended).** `k` becomes a role SET like `batch`, and the K size class and divisibility bucket are computed from the **product** of the declared K extents (as 6.6-0023 does for batch). No existing cell's bytes move (a single-K cell derives the same token), so by the R1.8 precedent no `sk` bump and nothing to regenerate; cells that could not be declared now derive. *Against:* a kernel that handles 2x3 but not 3x2 (a stride-regularity matter) is not separable by the key; that is a **binding** fact, and `st` sub-keys already flag irregular strides.
- **B. Key the structure** (a mode count or the nesting as a coordinate). *For:* a consumer can see "two K modes". *Against:* a new coordinate, an `sk` event, and it keys the shape of the iteration rather than the admissibility of a kernel.
- **C. Do nothing.** Unpopped keeps nested modes in its plan layer and presents one batch product. *For:* zero cost. *Against:* CUTLASS 3/4 stay unexpressible to a second deriver.

## Group 3. Operand major-ness (CUTLASS 5, CUDA 55)

Measured: a column-major B or A is **already distinct in the key** (`st/00/v1` against `co/00/v8`), so a kernel built for one cannot silently match the other. What is missing: `st` also covers a padded leading dimension and
any other irregular stride, and the key does not say **which of the role axes is unit-stride**.
- **A. Nothing (recommended unless Unpopped shows a collision).** The planner derives major-ness from strides and the role hints, which the caller already declares.
- **B. A derived pair in the contraction field** (which of K or M/N is contiguous, for A and for B), computed from the declared role axes' strides: deterministic, no new hint. A byte-altering `sk` event.
- Ask for Unpopped: one concrete pair of kernels that the current key admits wrongly; none has been named.

## Group 4. Source C and a compute dtype (CUTLASS 31, CUDA 44)

Measured: an extra `[M,N]` `f32` input derives, with its dtype in its own sub-key; **nothing names it as the source C** (any fourth input looks the same), and the compute dtype is operand 0's (6.6-0005, 6.17-0005).
This is the open question 3 of `docs/options-771-contraction-precision-and-accumulate-into-c.md` (accumulate into C: B the `[M,N]` extra input, or A nothing): **decide it there, once**.
- **A. A `c` role hint (a slot, like `weight`) plus the existing operand sub-key** carries C and its dtype; the key gains a presence coordinate. Byte-altering.
- **B. Nothing in the key**: C is an operand like any other and the op identity (`matmul` OpAttrs) says it is the addend. No bytes; weaker for admissibility.
- **Compute dtype:** KISS-CLASSIFY-6.1-0005 says compute precision is a KISS-Ops fidelity attribute, not a dtype, and `<acc>` plus `<mp>` already exist. **Recommendation: no new coordinate**; ask Unpopped what a distinct compute dtype adds beyond `<acc>` and `<mp>`.

## Group 5. Epilogue scale-factor output, SFD (CUTLASS 23)

Today the scale dtype keyed (`q<sdt>`, `g<len>`) is the **input weight's**; an output scale role does not exist. CUTLASS writes SFD as `float_ue8m0_t` per 16 elements (example 72b), which fits the existing `f8e8m0` token.
- **A. An `out_scale` role hint plus a keyed coordinate `o<sdt>` / `g<len>`** mirroring the input rule. Byte-altering.
- **B. A second Output operand named by its op** (`output_views`, R2.1), no new coordinate.
- Recommendation: B first (no bytes), A only if a consumer shows the key admitting a kernel that writes the wrong scale dtype.

## Group 6. Structured operands (CUTLASS 15)

Triangular, symmetric and Hermitian are different **ops** in BLAS (`trmm`, `symm`, `hemm`); KISS-Ops has none of them. **Recommendation: an op-vocabulary question (Ops, with a consumer: Fuel's planner), not a Classify key coordinate**; no bytes in Classify.

## Questions for CireSnave (each byte-altering option; the PM turns them into board items)

1. Group 2: A (collapse K and batch to products, additive) or B (key the structure) or C. **Recommend A.**
2. Group 3: A (nothing) or B (derived major-ness pair). **Recommend A until a collision is named.**
3. Group 4: with 771 Q3, a `c` role hint (A) or nothing (B). **Recommend whatever 771 Q3 is ruled; do not decide it twice.**
4. Group 5: B first.
5. One `sk` event for everything chosen above, listed to him beforehand and reported afterwards (the shape board 501 asks for the tokens).

**Sizing (this lane's estimate):** options note done; Group 2 option A about 1 lane-day (role set, product class, vectors, derivation corpus; no regeneration of existing bytes); each of Group 3 B, Group 4 A, Group 5 A about 1 to 2 lane-days
plus one `sk` event shared by all. Groups 1, 3 A, 4 B, 5 B and 6 cost no bytes.
