# Options note: R1.25 (#28, #67, #125), what is a region's identity, where does the recipe grammar live, and is cost one number or several?

**Informative. No normative text. For CireSnave's decision (row R1.25 is gated `[rule]`).**
Measured at `origin/main` `93bf30a`, 2026-10-26, by reading `spec/grammar.md`, `spec/ops.md`,
`spec/contract.md` and `ROADMAP.md` at that ref.

Three issues share one row. They are independent, so each gets its own answer below.

## Question 1 (#28): when a kernel has a constant baked in, is the constant part of the region?

Take `x * 2.0 + 1.0`. Two correct tools can disagree on what the region is. One counts `2.0` and `1.0` as
part of it. Another leaves them out, so `x * 3.0 + 1.0` is the same region. If they disagree, the same fused
kernel is recognised as two different regions and the round trip between an emitter and a lifter breaks.

What the spec says today: the region grammar has exactly two kinds of node, `Op` and `Bind` (`grammar.md:198-199`).
There is no node for a constant. Separately, KISS-Ops already has a constant leaf, `const(bits)`, for op bodies
(`ops.md` 6.12-0001), pinned by its bit pattern (6.12-0002). The region grammar also has an `extract` list that
routes scalar parameters out of a region (`grammar.md:195`). So the pieces exist; nothing says which one a
folded constant uses.

**A. A constant is a leaf in the region.** Add a `Const(bits)` node next to `Bind`, using the Ops leaf.
`x*2.0+1.0` and `x*3.0+1.0` are different regions. *What it changes:* one new node kind in the grammar and its
wire form, plus one golden vector that folds and refolds a constant. *Price:* a JIT that folds many constants
sees many regions, so a cache keyed on region identity grows.

**B. A constant is an attribute on the node that uses it.** Same result for identity as A (different values,
different regions), but no new node kind. *Price:* the attribute blob (`pattern_attrs`) is today only the Ops
OpAttrs blob (`grammar.md:419-424`), so this widens a field that is deliberately narrow.

**C. A constant is not part of the region.** It is routed out through `extract` as a scalar parameter, so the
region is the shape of the computation only. *Price:* a constant that the kernel really did bake in then needs a
parameter slot the kernel does not have, so a lifter would have to invent one; the emitted kernel and the lifted
region stop agreeing about arity.

**Recommendation: A.** It reuses a leaf that is already pinned bit for bit, it is the answer the issue itself
recommends, and it keeps the region honest about what the kernel computes. The cache growth is a cost for
the consumer to manage, not an ambiguity. The issue also states that requiring a particular buffer name is an
implementation bug, not a spec gap; the spec agrees (Consume 6.1-0002, 6.1-0003), so there is nothing to decide there.

## Question 2 (#67): where does the "recipe grammar" live?

The request is for one authoritative, project-agnostic place that says what a fused-op recipe is. Today the
parts are in four places: the region grammar (`grammar.md`), the scalar-source leaves (`ops.md` 6.12), the
decompositions (`ops.md` 6.13) and the Semantics section of the contract (`contract.md` 2.3). Two names the issue
and the row use, `runtime_scalar` and `scan_placeholder`, have no home at all: a search of `spec/`, `docs/` and
`ROADMAP.md` finds them only in the R1.25 row itself (the same search finds `const(bits)`, so it does look at
the right places).

**A. A new standard, "KISS-Recipe".** One new file. *Price:* a new standard means a new sidecar, new rows in
every coverage table, and a decision about what the existing four places then say (point at it, or repeat it).
**B. Name an existing standard as the home and fill the gaps.** KISS-Ops 6.12 already owns the leaves, so extend
it with the two missing ones and put an index in `grammar.md` and `contract.md` that points there. *Price:*
Ops gets longer. **C. Leave it spread out and write only an index.** Cheapest, and it does not give anyone one
document to build against, which is the stated need.

**Recommendation: B first.** It answers the practical complaint (two names with no home) with the least
machinery, and it can be promoted to A later without changing any clause id. The issue's own sequencing says the
authoring waits for the pre-freeze gap fixes; I have not re-checked that those have all landed. Note that the
harness half of this (a composite-op evaluator) is already rowed as R5.12.

## Question 3 (#125): is cost one number or several?

Fuel asks that cost carry a small set of expressions (compute, bandwidth, overhead) so a consumer can pick a
device, and that it carry whether the numbers are `declared` or `measured`.

**The second half is already in the spec.** Contract 6.8-0001 lists `cost_provenance` as a Guarantees field,
6.8-0006 says it MUST be `declared` or `measured`, and 6.8-0007 says the Capabilities cost is the only home of
the magnitude. So "add provenance" is done; what is left is only whether the magnitude is a vector. (I read the
clauses; I did not check whether anything flips `declared` to `measured` automatically, which the issue
describes as Fuel's own behaviour.)

For the vector, 6.7-0006 says "a cost class plus cost expressions", plural, but they are all expressions of the
same cost, so a consumer cannot tell compute from bandwidth.

**A. Decline.** Keep one magnitude. *Price:* the placement problem in the issue (two kernels with equal totals,
opposite best devices) stays unsolvable from the contract. **B. Add an optional sibling field** with named
axes, leaving 6.7-0006 untouched. *Price:* two places to look for cost, and an optional field most providers
may leave empty; the issue itself names that as its strongest objection. **C. Change the pinned clause** so the
cost is the vector, a single expression read as the compute term. *Price:* it edits pinned text, which needs a
second implementer to freeze, and only Fuel has one today.

**Recommendation: B, with the axes limited to `compute` and `bandwidth`.** It does not touch pinned text, so it
is freeze-safe, and it can be backed by one implementation now. If most providers leave it empty, that is
information, and C is then not worth its cost. Ask Baracuda and Unpopped (the issue names Vulkane) whether they
can measure the two axes separately before the clause is written.

## What freezes, and what to decide

Question 1 touches the region wire form (A adds a node kind), so it should be decided before the freeze. Question
3 touches nothing pinned under B. Decisions needed: **Q1 A, B or C; Q2 A, B or C; Q3 A, B or C**, and whether
the axes in Q3 are `{compute, bandwidth}`.
