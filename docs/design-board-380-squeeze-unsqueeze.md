# Design: `squeeze` and `unsqueeze` (board 380, R1.19, KISS#426)

**Informative. No normative text; the PR that adds the clauses follows this note.** Read at KISS `origin/main` `d47b99e` (conformance 0.8.1).
CireSnave, board 380, verbatim: "380: B plus C". As the PM summarized it: `squeeze` and `unsqueeze` primitive shape ops; dropping reduces (the `reduce_drop` family) as named non-primitive ops resolving to `squeeze(reduce(...))`; no `keepdim` flag; squeezing a non-size-1 axis is a typed refusal; existing ops' bytes do not move.
**Superseded in part, same day (CireSnave 2026-11-07, via the PM):** "Okay.. Q1: a." and "Make both real flags now." So `keepdim` **is** a flag (KISS#777), there are **no** `reduce_drop` ops, and `squeeze`/`unsqueeze` stay as primitive shape ops, with the identity `reduce(keepdim = 0) = squeeze(reduce(keepdim = 1))`. This note designs the two ops under that ruling.

## What they are

- `squeeze(x, axes)`: remove the axes in `axes`, each of which MUST have extent 1 (a non-extent-1 axis is a **typed refusal**, never a silent reshape). Output rank = input rank minus `|axes|`; surviving axes keep order and extents; removing every axis of a rank-`r` operand whose extents are all 1 gives rank 0.
- `unsqueeze(x, axes)`: insert an extent-1 axis at each position in `axes`, where the positions are **output** axes: output rank = input rank plus `|axes|`; the output axes in `axes` have extent 1 and the remaining output axes, in order, are `x`'s axes (NumPy `expand_dims` with a set).
- Both are **pure shape**: every element's bits (a NaN payload, a signed zero) are preserved and nothing is computed. `unsqueeze(squeeze(x, S), S) = x` when `S` is a legal squeeze set of `x`.
- Output shape rules (6.20): like `concat`, a semantic rule of the op, not a core `ShapeExpr` (the core vocabulary has no constructor that removes or inserts an axis).

## The axis encoding: a bitmask, a plain `u8`, never the `reduce_axes` sentinels

The question: bitmask or index list. **Recommendation: a bitmask**, because the operand is a *set* (order carries no meaning, an index list needs a sort and a duplicate rule), because it is one encoding across the spec (`reduce_axes`, and `norm_axes` per board 391 "B"), and because `MAX_RANK = 8` makes it one byte.

A refinement I recommend, and want CireSnave to see: **a plain `u8` subset mask (`0x01..0xFF`, bit `k` = axis `k`), not the `u16` `reduce_axes` word.** The `reduce_axes` word has the rank-aware sentinels `0xFFFF` (all axes) and `0xFFFE` (the lone trailing axis), which exist only so a reduction's category can be rank-independent. For `unsqueeze` they are **ambiguous**: the positions are over the *output* rank, and an all-axes `0xFFFF` over a rank-0 operand does not say how many axes to insert (`unsqueeze` of a scalar to rank 3 and to rank 5 would share the bytes). A plain mask has no such case and its bytes do not depend on the operand rank at all.

| Case | bytes (plain `u8` mask) | bytes if the `u16` `reduce_axes` word were used |
|---|---|---|
| `squeeze` axis 1 of `[2,1,4]` | `02` | `02 00` |
| `squeeze` axes `{0,2}` of `[1,5,1]` | `05` | `05 00` |
| `squeeze` every axis of `[1,1]` | `03` | `FF FF` |
| `unsqueeze` a `[3]` operand at output position 0 (output `[1,3]`) | `01` | `01 00` |
| `unsqueeze` a scalar to output rank 3 (positions `{0,1,2}`) | `07` | `FF FF` (rank of the output is **not recoverable**) |

An empty set (`0x00`) is a typed refusal for both (it would be an identity under another name: "one way to say one thing"). A set with a bit at or past the rank (`squeeze`: the input rank; `unsqueeze`: the output rank, which is the input rank plus the popcount) is a typed refusal.

## `#725` removed `unsqueeze`: reverse it, or mint fresh?

`#725` (0.8.1) added `cast` and `concat` and removed `transpose` and `unsqueeze` from the shape oracle, with 6.20-0011 stating "There is **no** `transpose`, `permute` or `unsqueeze` op in this version ... An implementation MUST NOT mint such an op token." The reason was "one way to say one thing": a transposed or axis-inserted operand is a *stride view* on an operand.
That reason does not reach a rank change on an **interior node's result** (a DAG node has no operand strides to put the view in), which is exactly where `keepdim = 0` results meet an op that wants them broadcast back. Options:

- **A. Reverse the removal for both ops.** 6.20-0011 keeps its `transpose`/`permute` prohibition, drops `unsqueeze` from it, and adds `squeeze` and `unsqueeze` as the two shape atoms. The names are the same tokens `#725` removed; any consumer that held a pre-`#725` `unsqueeze` token re-binds to a defined op.
- **B. Mint fresh tokens (`expand_dims`, `drop_dims`).** Avoids the appearance of a reversal. Costs a second spelling of a name every other IR already uses (NumPy `squeeze`/`unsqueeze` are PyTorch's; ONNX has `Squeeze`/`Unsqueeze`), against the one-spelling rule.

**Recommendation: A**, with the 6.20-0011 sentence rewritten to say what is true now: *transposition and axis permutation are views; a rank change of a node result is the `squeeze`/`unsqueeze` pair.* `transpose` and `permute` stay prohibited.

## Where they sit and what they cost

- **Primitive floor** (6.3-0001): `squeeze` and `unsqueeze` join the structural access atoms; the floor grows by two (45 to 47 at 0.8.1; the figure in the clause is the authority). No new arithmetic.
- **Decompositions:** none (primitives). The `keepdim = 0` result of every reduction-family op (KISS#777) resolves to `squeeze` of its `keepdim = 1` result.
- **OpAttrs:** two new carriers, each exactly one `axes` field (`u8` plain mask, mandatory, `0x00` refused); the carrier set grows by two on top of #777's seven. Existing ops' bytes do not move.
- **Typed refusals:** `squeeze` of an axis whose extent is not `1`; an empty or out-of-range set; `unsqueeze` of a set that leaves no room for the input's axes (`popcount` plus input rank above `MAX_RANK`).
- **Vectors:** the five rows above as OpAttrs goldens; a value vector per op (bit preservation of a signaling NaN and `-0`); the refusals; the identity `reduce(keepdim=0) = squeeze(reduce(keepdim=1))` checked over a grid of shapes and axis sets (this is the test that ties board 380 to #777).
- **Readers:** every OpAttrs reader gains two schemas; every shape-oracle reader two shape rules; `kiss-ref`'s `op_manifest.json` re-vendor **adds two ops** (124 to 126 once `cast`/`concat` land), and its coverage gate requires an evaluator for each (both are trivial data movement; this is the same shape as kiss-ref#61).
- **Unpopped:** its `Reshape` already covers rank change; `squeeze` is a rank-reducing `Reshape` there and `unsqueeze` the inverse (Unpopped lane's statement, 2026-11-04). The master-list rows D3 and I1 do not change.
- **Version:** Breaking for the op set (new ops in the floor); proposed for the 0.9.0 batch with #776 and #777. Not stamped.

## What this does not decide

Whether `squeeze` with **no** axes given ("squeeze every extent-1 axis", PyTorch's default) should exist. It makes the output rank depend on the operand's *extents*, not its rank, which the shape rule of every other op does not; I recommend against it and the empty mask is a refusal. CireSnave may overrule.
