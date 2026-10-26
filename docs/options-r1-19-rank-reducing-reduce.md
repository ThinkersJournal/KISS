# Options note: R1.19, how does a rank-reducing reduction get a conformant contract? (#426)

**Informative. No normative text. For CireSnave's decision; the PM turns the question into a board item.**
Read at `origin/main` `2e65634`, 2026-10-26. Paths are `file:line` at that commit. Issue: KISS#426 (Fuel; it is also an
input of #67, which R1.25 covers). Roadmap row: R1.19. Nothing here freezes except through an added op.

---

## The question

**Example.** A kernel sums a rank-3 tensor over one axis and returns a rank-2 tensor, the axis gone. Fuel has such ops
(`SumDim`, `MeanDim`) next to ones that keep the axis as extent 1 (`ReduceSumTo`). KISS has only the second kind. Its
`reduce` always keeps each reduced axis as an extent-1 axis with stride 0 (KISS-OPS-6.11-0008, `spec/ops.md:1074`), so
that `sub(x, reduce(max, x))` reads the reduced value at every position of the original axis. The `keepdim` attribute is
pinned to `1` (KISS-OPS-6.19-0030, `spec/ops.md:2296`). A contract's declared output shape must equal the shape rule of its
Semantics op (KISS-CONTRACT-6.4-0011, `spec/contract.md:839`), so a rank-reducing kernel would declare rank 2 against a
rule that gives rank 3, and the contract is a typed decline. The kernel is not wrong; the vocabulary has no op that
describes it.

**Checked again, with controls.** `squeeze`, `reshape` and `rank-reducing` have no definition anywhere in `spec/`.
`unsqueeze` appears once, in a sentence listing ops whose shape comes from their operands (`spec/ops.md:2458`), and is
not defined either. The same search finds `keepdim` 65 times in `spec/`, so it does look in the right place. The reason
for the current rule is sound, so this may be a scope choice and not an oversight.

**The issue's warning, repeated:** do not fix this by making `keepdim` variable. Broadcast-back is what the
decompositions rely on, and the gap is a missing shape operation, not a missing flag.

## Options

- **A. Add a rank-changing op, `squeeze`,** with an axes attribute and a shape rule, so `reduce` followed by `squeeze`
  gives the rank-reducing form and its contract's Semantics is that two-node DAG, whose shape rule equals the declared
  output. Cheapest in expressiveness: `reduce` and its broadcast property are untouched. *Cost:* a new op needs a shape
  rule (KISS-OPS-6.20-0001), an OpAttrs blob, a decomposition (a view moves no data, so the value half is the
  identity), vectors, and support in Fuel, Unpopped and Baracuda. Adding a high-level op does not force a consumer
  change and advances the op-vocabulary schema version, not the primitive floor (KISS-OPS-7.3-0001 and 8-0002,
  `spec/ops.md:2610` and `:2645`); I did not check whether `squeeze` would count as high-level or as a primitive.
- **B. Say it is out of scope.** The op set is a decomposition currency, not a tensor-program language, so rank changes
  live above it. A rank-reducing kernel declares the kept shape in its contract, with the rank change outside the op.
  *Cost:* KISS-CONTRACT-6.4-0011 forbids that today, so it must gain an exception ("equal the rule, or equal it with
  the reduced extent-1 axes removed"), which is a special case in a clause that has none.
- **C. Add distinct rank-reducing reduce ops,** one per reduce, the way Fuel does. *Cost:* doubles the reduce family and
  repeats each decomposition; every future reduce needs two spellings.

**Recommendation: A.** It composes: any future reduction gets the rank-reducing form for free, the broadcast rule stays
as it is, and the shape rule of the two-node DAG is computed by the evaluator that already exists. B is honest but
makes a clause special; C costs the most. **A is the choice with a freeze consequence:** it adds a name the three
consumers must learn, so if you pick it, ask them for a measurement of which of their ops it replaces.

## What waits, and what to decide

Nothing waits on this except the Fuel rows that need a rank-reducing op. Decide: **A, B or C**, and for A whether
`squeeze` takes one axis or a set.
