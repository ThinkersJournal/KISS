# Decision memo — #263: who decides which `op_family_tag` an op gets

**For CireSnave, via the PM. One decision needed; options and a recommendation below.** Measured at KISS
`origin/main` `904a4b4`, 2026-10-09. Evidence: `docs/rfc-op-family-mapping.md` (the 121-row proposal) and issue #263.

## The problem in one paragraph

`op_family_tag` is one of 24 codes (`une`, `bin`, `gem`, `red`, …) and it is a field **inside the `structure_key`**, the key
a kernel is admitted by. KISS-CLASSIFY-6.5-0006 says an implementation must fail rather than guess when it cannot map a
cell to a family, but **nothing in the suite says which family an op belongs to**. So two correct implementations can tag
the same computation differently, derive different keys, and neither's kernels are admissible to the other. This is
freeze-blocking. Today it works only because the implementations happen to agree on the 5 families the test vectors
exercise (`bin gem red scn une`); the other 19 families have no vector.

## What is already settled (so you are not asked again)

- Six ambiguous pairs resolve **most-specific-wins** (`softmax`→`sft`, `embedding`→`emb`, `silu`→`gat`, `matmul`→`gem`,
  `max_pool`→`pol`, `im2col`→`cnv`); **ruled by the architect on #263**. A code may be added freely; existing ops are never
  reassigned without a schema bump. `shp` being empty is ruled intended.
- The mapping is **per op**; no table between the two vocabularies' *family names* can exist (they partition on different axes).
- Of 121 ops: 96 are derived, 12 are covered by that ruling, **12 are judgement** (arity is stated nowhere in `ops.md`),
  and **`element_map` is left unassigned** (it has no fixed family: it depends on the body it carries).

## The decision

Who is the authority for the op→family table, and when does it become binding?

| Option | What it means | Cost / risk |
|---|---|---|
| **A. Pin the per-op table now as normative** (as an `op_manifest.json` column) | Every op gets a code; `element_map` gets an explicit rule | Changes keys wherever an implementation already assigns differently. 19 of 24 families have no vector, so disagreement is currently **undetectable**. Fuel and Unpopped have not confirmed their assignments (the proposal's own gate) |
| **B. Pin it after the derivers confirm, and fold any token change into the next schema bump** (recommended) | A first, but gated: each deriver reports, per op, what it emits today; agreed rows are pinned; disagreeing rows are decided and ride the `sk5` event (see `docs/proposal-wire-change-520-477.md`) | Slower by one measurement round; no kernel silently stops being admissible. The `sk5` event is already being collected (#222) |
| **C. Leave the tag caller-chosen and say so** | Document that `op_family_tag` is producer-declared; cross-implementation admissibility is not promised | Cheapest. Concedes that two conforming implementations may be mutually inadmissible, and the freeze gate stays blocked on #263 |

**Recommendation: B.** It is the only option that is both correct and safe: A pins a mapping nobody can falsify for 19
of 24 families, and C gives up the property the key exists to provide. The cost of B is one round of measurement by two
other lanes, which I cannot do for them.

## What I need from you

1. **B, A, or C.** (If B: nothing else from you; the lanes do the confirmation.)
2. Whether the architect's most-specific-wins ruling stands as a ruling you accept, or should be re-confirmed by you.
   It is recorded as the architect's, not yours.

## What I have *not* done

I have not compared Fuel's or Unpopped's actual op→family tables to the proposal. Both derive from **their own op
vocabularies** (Fuel's op kinds, Baracuda's `op_class` strings), not KISS op names, so the comparison needs each
lane's translation table and is that lane's work (M each). The proposal's 12 judgement rows have been reviewed once
(it already corrected `popcount`); none has been confirmed by a deriver. #298 (operand structure, ruled) is a
dependency for deriving arity from the spec rather than from prose, and is still open.
