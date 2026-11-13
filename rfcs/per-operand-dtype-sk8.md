# RFC: per-operand dtype in the `structure_key` (the next schema event, `sk7 -> sk8`)

| | |
|---|---|
| **Status** | **Draft, informative. Nothing is frozen.** The direction is ruled; the grammar, the ride-along set and the dates are not. |
| **Ruling** | CireSnave, 2026-11-13, board 430 Part 1 question 2, choice 2a, relayed verbatim by the PM: *"Yes.  Of course operations should be able to accept a different type per input."* |
| **Affects** | KISS-Classify (§6.1, §6.5, §6.6-0005/-0007/-0015/-0022, §6.7-0004/-0014), consumers of the key (Fuel, kiss-ref, Unpopped `unpopped-vocab`, Baracuda emit corpus) |
| **Follows** | `sk4-schema-event.md` §3.5 (the deferral this RFC takes up; same format) |
| **Name** | The PM's task says "sk5". The tree is already at `sk7` (`STRUCTURE_KEY_VERSION` = 7, Classify §6.7-0002 area), and `sk5`, `sk6` have shipped. The next event is **`sk8`**; this RFC uses that name. (The spec's own "currently `5`" in the §8 versioning paragraph is stale and is a separate wording fix, not byte-visible.) |

> **Informative until adopted.** Every token below is a **schematic**. The authoritative bytes come from
> the reference codec once the grammar is fixed, never from this document (the sk4 freeze-gate discipline).

## 1. Summary

Today a non-contraction `structure_key` names **one** dtype: operand 0's (§6.6-0005). §6.6-0015 says so on
purpose, and it forbids a per-operand dtype in a sub-key. Two cells that differ only in a non-primary
operand's dtype derive **byte-identical keys**. The ruling reverses that: an operation accepts a different
dtype per input, so the key must be able to say so.

Proposal, one sentence: **every per-operand sub-key carries that operand's dtype**, field 2 stays operand
0's dtype, and §6.6-0015 is replaced by a rule that a key is distinct for every distinct per-operand dtype
list. This is byte-visible for **every** key (a new mandatory component), so it is a schema event and rides
one coordinated re-derivation.

## 2. Why the collision hurts (evidence, all at `origin/main`)

- **Indexed ops.** `gather`, `scatter`, `index_select`, `embedding`, `scatter_add` carry an index operand
  (`index_operand` + `index_dtype` in OpAttrs, Ops §6.11-0012) in `{u32, i32, i64}` (§6.11-0009). Data is
  float, index is integer. Today a `u32`-index and an `i64`-index cell with the same data dtype and shape
  derive the **same key**; the index width the kernel loads is not in it. The sk4 RFC called this a
  describability gap and said the numeric axes were already keyed. That held while the key only had to
  separate numerics; it does not hold for a cache that must separate *kernels*.
- **`cast` and `bitcast`.** The destination dtype is an **output** operand's dtype (outputs are operands,
  §6.6-0014), so `f32 -> f16` and `f32 -> bf16` casts also share a key today. `bitcast` (staged) and `iota`
  are the same shape of problem.
- **Mixed-dtype elementwise and block-scaled operands.** `ctr` cells already escape the rule (§6.7-0006
  weight/accumulator/output coordinates; §6.6-0022 scale dtype, from `sk6`). The rule is therefore
  already special-cased for exactly the cells where the collision was found to be wrong. This RFC makes
  the exception the rule.

## 3. The design

### 3.1 Where the dtype goes (the real choice)

The sub-key today (§6.7-0004): `<contig>/<bcasthex>/<vec>/<div>/<flip>` plus an optional sixth `/<dq>` iff
the operand carries `quant`.

| Option | What it is | Consequence |
|---|---|---|
| **A. Mandatory dtype in every sub-key** | New fixed component `/<dtype>` before the optional `/<dq>`: five parts become six, six become seven. | One shape, no conditional presence. Parts count still distinguishes `dq` (`dl`/`dc`) from a dtype token. Longer tokens; every key changes. |
| **B. Elide when uniform** | Emit the dtype list only when some operand differs from field 2. | Uniform keys keep their old bytes (minus the version). But conditional presence is the hazard §6.6-0022 and `dq` already fight ("iff"), and a reader cannot tell "uniform" from "forgot". |
| **C. Key only role-designated operands** | Add a dtype only for the index operand (like §6.6-0022 does for the scale). | Smallest diff, solves the indexed cases. Does **not** satisfy the ruling's general sentence (a different type per input), and leaves cast/elementwise colliding. |

**Recommendation: A.** The ruling is general; C would need a second event the first time someone mixes
dtypes outside a role. B saves bytes nobody was paying for.

### 3.2 Rules the event pins (schematic wording)

1. Each per-operand sub-key MUST carry its operand's dtype token (§6.1 spelling, §6.1-0004) as a mandatory
   component. Operand order is the canonical order of §6.6-0014 (inputs, then outputs).
2. Field 2 stays **operand 0's** dtype (§6.6-0005, unchanged), and the first sub-key's dtype MUST equal it;
   a reader MUST decline a key where they differ. This keeps field 2 usable as a cheap filter.
3. **Replaces §6.6-0015.** A `structure_key` MUST be distinct for two cells that differ in any operand's
   dtype. The "MUST NOT add a per-operand dtype to a sub-key" sentence and the `ctr`-only carve-out are
   deleted.
4. **Dtype positions** (§6.1 reserved/unknown handling) now include every operand's sub-key. A reserved
   dtype (the fnuz floats) in any operand MUST decline with the reserved-dtype typed decline. This
   *reverses* the current statement that a non-primary reserved dtype derives a key without declining; it
   is a consequence to expect, and the migration note must say it.
5. **`ctr` coordinates.** `<wdt>`, `<acc>`, `<out>` (§6.7-0006) and `q<dtype>` (§6.6-0022) now restate facts
   the sub-keys carry (weight, output and scale operand dtypes). Two spellings of one fact is a hazard.
   Options: (i) keep them and add a MUST that they agree with the sub-keys; (ii) drop `wdt`/`out`/`q`
   and keep only `acc`, which is not an operand dtype. **Recommend (i)**: fewer bytes move, the four-party
   re-derivation stays mechanical, and the agreement rule is a one-line check. Revisit (ii) at the next
   event once nothing reads the old coordinates.
6. **What the key does not decide.** Which dtype combinations an op *accepts* is the op's contract
   (Ops §6.11-0009, §6.17 and the per-op rules); the key still says only what was asked for
   (admissibility, not freshness). The caller-precondition that §6.6-0015 was becomes: a caller MUST
   present every operand's true dtype, and a key built from a dtype the buffer does not hold is a caller
   error, exactly as for the primary dtype today.

### 3.3 The `ix32` / `ix64` index-width key

Orthogonal, **unchanged**. `ix32`/`ix64` (field 4, §6.5-0005) says whether the cell's maximum touched
**element offset** is below `2^31`. It is a property of address arithmetic over extents and strides.
The index operand's dtype (`u32`, `i32`, `i64`) is the dtype of **values read from memory** and used as
subscripts. Neither implies the other: an `i64` index tensor into a small table is `ix32`; a `u32` index
tensor into a table over `2^31` elements is `ix64`. For a gather, the offset bound comes from the data
operand's extent along the gathered axis, so it is computable from shapes without reading indices
(the index values are unknown at classification time and bounded by the caller's `oob` behaviour,
§6.11). One residual question for the parties: whether the *value range* of a narrow index dtype ever needs
to constrain `ix` (an `i32` index into a `> 2^31`-element axis cannot address it). Recommend: that is an
**op-level decline** (the index dtype cannot span the axis), not a key rule. Raised, not decided.

### 3.4 Worked examples (schematic, `ndx` cell class; not authoritative bytes)

Sub-key shown as `<contig>/<bcasthex>/<vec>/<div>/<flip>/<dtype>`.

- **gather**, `f16` data `[V, D]`, `i32` index `[N]`, `f16` out `[N, D]`:
  `sk8|ndx|f16|T|ix32|grid|r2|co/00/v8/d16/f/f16;co/00/v1/d1/f/i32;co/00/v8/d16/f/f16|-`
  The same cell with a `u32` index differs in the middle sub-key, so no collision.
- **scatter / scatter_add**, `f32` destination (read-modify-write, listed once as an input, §6.6-0014),
  `i32` index, `f32` source: three sub-keys `f32 / i32 / f32`; the in-place destination appears once.
- **embedding**, `f32` table `[V, D]`, `u32` ids `[N]`, `f16` out: **mixed float widths** as well as an
  integer index (`f32 / u32 / f16`); only a per-operand dtype can describe the table and the output
  being different floats.
- **Block table (paged KV cache)**, `f16` KV pool, `i32` block table `[B, MAXB]`, `i32` sequence lengths
  `[B]`, `f16` out: an indexed region indirected through two integer operands; both index operands carry
  `i32` and their roles stay in OpAttrs. Whether a two-level indirection stays one `ndx` cell or needs its
  own class is **not** decided here (a cell-class question; see §5).
- **CSR SpMV**, `f32` values, `i32` col_indices, `i32` row_ptr, `f32` x, `f32` y: three integer-or-float
  groups, five sub-keys. Here the data-dependent extent (nnz per row) is the real obstacle, and it is
  independent of this RFC; the per-operand dtype only fixes the *types*.
- **`cast`** `f32 -> bf16`: `f32 / bf16` (two sub-keys): the destination finally appears in the key.

## 4. Migration

- **Every `sk7` token changes**: the new mandatory component alters each key, so there is **no
  "unchanged for uniform cells" tier** under option A. Every persisted key MUST be re-derived, not
  decoded. Under §6.7-0014 the schedule is unchanged: at `sk8` the **reference** codec declines every
  non-`sk8` version with the recognized-but-unaccepted decline (§6.7-0015); an `sk7` arm MAY be retained
  through the `sk8` series; `sk6` is not decodable at `sk8`.
- Version-prefix inseparability (§6.7, sk4 clause D) is what makes a dtype sub-token safe to add: it has no
  meaning detached from `sk8`.
- **No token that decodes under both versions to different meanings** is introduced (the sk4 `c64` check):
  every `sk7` token has five or six parts per sub-key and every `sk8` token six or seven, and the
  version prefix is exact-matched. Confirm in the four-way derivation.

## 5. Ride-along: every other byte-visible change to batch into the one re-derivation

Committed by a ruling:
- **Per-operand dtype (this RFC).**

Pending, undecided, would be byte-visible and should ride if decided in time:
- **The `u16` / `u64` prune** (board 430 Part 2 item 4, **undecided**). Removing two of the 24 dtype tokens
  (§6.1-0001, which `PRIOR-ART.md` still spells "twenty-token") changes the closed set; any persisted key
  using them must be re-derived (it declines at `sk8` as unknown). If kept, the 24-token set is unchanged.
- **R1.26 remainder** (ROADMAP, `[wait: schema event]`): keying the block length and the scale-group axis
  (a closed length set `{16, 32}`), and who pins in-block packing and conversion rounding. Block scaling
  adds components to the contraction field or sub-key; it should share the event.
- **Cell class for multi-level indirection** (the block-table question above): only if the parties find
  one `ndx` cell cannot distinguish single- from two-level indirection. Possibly not needed.

Not byte-visible, do **not** need the event: the packed tuple-state reductions (ruled 427, presenter rule:
key bytes unchanged), `padding_lo/padding_hi` and the matmul fields (OpAttrs, Ops layer), `bitcast`/`iota`
(Ops layer), the §8 "currently `5`" wording.

I know of no other pending non-additive Classify change; the parties are asked to list theirs before the
grammar freezes (§7).

## 6. Clause impact (indicative)

| Clause | Change |
|---|---|
| §6.6-0015 | replaced (§3.2 rule 3) |
| §6.6-0005 | stays; adds the operand-0 sub-key must agree |
| §6.6-0007 / §6.7-0004 | the sub-key gains the dtype component |
| §6.6-0022 / §6.7-0006 | `q<dtype>`, `wdt`, `out` agreement rule (or removal, §3.2 rule 5) |
| §6.1 dtype-position text | positions widen to every operand; reserved dtype declines |
| §6.7-0014 | arm schedule moves one step (`sk7` MAY be retained, `sk6` not) |
| `STRUCTURE_KEY_VERSION` | `7 -> 8`; corpus, decline vectors, `structure_key_vectors` regenerated |

## 7. Re-derivation plan (four parties) and acceptance

Same gate as sk4 §6: **four independent derivations byte-match** (KISS spec + reference codec, Fuel,
kiss-ref, Unpopped `unpopped-vocab`); Baracuda's leg is the **emit corpus** regenerated under `sk8`
(downstream of `unpopped-vocab` on spelling, independent on cell assignment and coverage, per sk4 §6).
Vulkane cosigns vocabulary and derives no key.

1. KISS publishes the grammar clauses and the **schematic** corpus plan, no bytes.
2. Each party lists the other byte-visible changes it wants to ride (§5) and its index-dtype cases
   (§3.4). **Freeze nothing until that list closes.**
3. The reference codec emits the corpus; **the codec, not this document, decides spelling**.
4. Three other derivations regenerate independently and byte-match; mismatches are findings against the
   grammar text, not against a party.
5. Baracuda regenerates the emit corpus; Unpopped confirms the `ndx`-class kernels' cell assignment.
6. Cut, scheduled retirement per §4.

New decline vectors required: a sub-key missing its dtype, an operand-0 dtype that disagrees with field 2,
a reserved dtype in a non-primary operand, and an `sk7` token presented to an `sk8` reader.

## 8. Decisions this note needs from CireSnave

Not asked, and not needed to start the work: the direction is ruled. Open items for the maintainer, in
order of consequence: **(1)** the `u16`/`u64` prune (board 430 Part 2 item 4) as a ride-along or not;
**(2)** option A (recommended) versus C for the sub-key; **(3)** whether a narrow-index range limit is a key
rule or an op decline (recommended: op decline). Everything else is a clause-authoring detail for the
editors once the parties have reported.

*— lane: kiss*
