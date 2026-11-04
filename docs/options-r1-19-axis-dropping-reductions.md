# Options: axis-dropping reductions (R1.19, KISS#426, board 380)

Status: **options with a recommendation; nothing here is a clause.** 2026-11-04, `origin/main` 5988cf1 (conformance 0.7.10). CireSnave's instruction on board 380 (via the PM, 2026-11-04): not picked; he wants "maximum width of expressibility"; keep-the-axis, a squeeze op, **and** dropping reduce ops may all be needed. Deliverable asked: what each language does (cited), what we need to express it, a recommendation.

## 1. What KISS has today (measured at 5988cf1)

`reduce` is "associative-monoid fold over an axis set (keepdim result)" (`spec/ops.md` line 277); the result keeps each reduced axis at extent 1 with stride 0 so it broadcasts back (line 1085; test `test_ops_reduce_keepdim_broadcast`); `reduce_axes` is a per-axis keepdim bitmask (line 2166). `squeeze`: 0 hits in `spec/ops.md` and `spec/contract.md` (control: `keepdim` 30 and 3 hits in the same files). `unsqueeze`: one mention, no op entry (finding F1 of `docs/union-coverage-unpopped.md`). `reshape`: 1 hit in contract.md, no op. §6.4-0011 requires a declared shape to equal the op rule's shape.

## 2. What other languages do

Only rows I read on the page itself are marked **verified**; the rest are stated as what could not be read, not guessed. Survey run 2026-11-04 by a delegated read-only agent; I did not re-fetch each page.

| Language | Mechanism | Default | Source | Status |
|---|---|---|---|---|
| NumPy `sum` | `keepdims`; true leaves reduced axes "as dimensions with size one" | drop | numpy.org/doc/stable/reference/generated/numpy.sum.html | verified |
| PyTorch `torch.sum` | `keepdim` flag | drop (False) | docs.pytorch.org/docs/2.15/generated/torch.sum.html | verified |
| ONNX `ReduceSum` | `keepdims` attribute | **keep** (1) | onnx.ai/onnx/operators/onnx__ReduceSum.html | verified |
| Triton `tl.sum` | `keep_dims` | drop | triton-lang.org/main/python-api/generated/triton.language.sum.html | verified |
| TVM `topi.sum` | `keepdims=False` | drop | tvm.apache.org/docs/reference/api/python/topi.html | verified |
| XLA/HLO `Reduce` | drops reduced dims (rank = operand rank minus reduced); restore with `Reshape` or `BroadcastInDim` | drop only | openxla.org/xla/operation_semantics | verified |
| MLIR `linalg.reduce` | example `tensor<16x32x64xf32>`, `dimensions=[1]` gives `tensor<16x64xf32>` | drop only | mlir.llvm.org/docs/Dialects/Linalg/ | partial (from the example, not prose) |
| MLIR `linalg.generic` | iterator types `parallel`/`reduction`; the output indexing map sets the shape, so both are expressible | n/a | same | partial (the keep-via-map point is inference) |
| MLIR `tensor.collapse_shape`/`expand_shape` | separate reshape ops by reassociation | n/a | mlir.llvm.org/docs/Dialects/TensorOps/ | verified |
| Halide `RDom`, inline `sum` | no flag; the author chooses the output Func's pure dims | n/a | halide-lang.org/docs/api/generated/RDom.html | snippet only, not fetched |
| CUB `WarpReduce` | per-thread scalar, valid in lane 0; no tensor shape | n/a | nvidia.github.io/cccl/cub/api/classcub_1_1WarpReduce.html | snippet only (fetch 404) |
| TVM relay/relax `keepdims`/`exclude`, `te.sum` | not read | ? | n/a | **unverified** |
| PTX `redux.sync` | not read (fetch truncated) | ? | n/a | **unverified** |
| SPIR-V `OpGroupNonUniform*` reduce | not read | ? | n/a | **unverified** |
| OpenCL `work_group_reduce_*` | not read | ? | n/a | **unverified** |
| WGSL `subgroupAdd` | not read | ? | n/a | **unverified** |

Reading, limited to what was verified: five tensor-level APIs (NumPy, PyTorch, ONNX, Triton, topi) offer **both** modes behind one flag and **disagree on the default** (ONNX keeps, the others drop). XLA and `linalg.reduce` offer only the dropping form and restore rank with a separate reshape. Loop-nest IRs (Halide, `linalg.generic`) have no flag because the output indexing decides. The lane/warp primitives return a scalar per lane, with no tensor axis; they say nothing about KISS's shape question and need an axis-to-lane mapping, which is Consume's concern. The five unverified rows (PTX, SPIR-V, OpenCL, WGSL, relay/relax) should be read before the board entry cites them; the PM's list is the Unpopped IR's union, and the Unpopped lane reports its IR already models both keep-axis and drop-axis reduce, with squeeze a `Reshape`, not a reduce mode (PM relay 2026-11-04).

## 3. What KISS needs to express the union

1. **keep-axis reduce** (exists).
2. **drop-axis reduce** (the NumPy/PyTorch/Triton/XLA default; Fuel `SumDim`/`MeanDim`).
3. **squeeze / unsqueeze** over an explicit axis list, as pure shape ops (zero arithmetic; XLA `Reshape`, MLIR `collapse_shape`/`expand_shape`, Unpopped `Reshape`).
4. A stated **identity**: drop-axis reduce over axes S equals keep-axis reduce over S followed by squeeze of exactly S. (My inference; no surveyed source states it.) With it, #67's requirement holds: the two forms are distinct cells (different output shapes) yet share one semantics.

## 4. Options

- **A. Say it is deliberate** (board option 1): the rank change is outside the op. Rejected by the instruction for width.
- **B. `squeeze` and `unsqueeze` primitives only** (board option 2). Reduce plus squeeze composes to the dropping form. Smallest; but a contract for `SumDim` then has two ops, so a kernel that fuses them has no single op name to declare, and the structure key sees a composition.
- **C. Drop-axis reduce ops as named non-primitives with a floor decomposition** (board option 3, built on B): e.g. `reduce_drop` with the same `reduce_axes` attribute, defined as `squeeze(reduce(...))` at the §6.13 floor. A kernel declares the one op; a reader resolves it to the floor and meets only `reduce` and `squeeze`. The floor gains one primitive (`squeeze`) and no new arithmetic.
- **D. A `keepdim` flag on `reduce`.** Rejected: it makes the broadcast contract of line 1085 conditional (the issue's warning), and ONNX/NumPy disagreeing on its default shows how readers diverge.

## 5. Recommendation: B plus C (primitives `squeeze`/`unsqueeze` at the floor, `reduce_drop` as a non-primitive resolving to them)

It covers items 1 to 4, keeps `reduce` as it is, adds no arithmetic primitive, and gives a fused kernel one declarable name. **`unsqueeze` goes with `squeeze`** since XLA/MLIR/Unpopped treat the pair as one reshape family and a union IR needs the inverse (F1 already found the oracle naming it). Open points: whether the axis list is an attribute bitmask (as `reduce_axes`) or an index list; whether `squeeze` of a non-extent-1 axis is a typed decline (recommend yes); the §6.4-0011 carve-out is **not** needed under C because each op states its own output shape; the Unpopped lane's own `Reshape` shape to be read before the clause.

Byte consequence: new op names and ordinals and a new OpAttrs schema row; an existing op's bytes do not move. No clause until CireSnave picks (or accepts B plus C as "maximum width").
