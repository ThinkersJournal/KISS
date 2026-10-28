# Options note: should a provider that claims a backend have to cover the basic ops? (board 309 Q2, KISS#12)

**Informative. No normative text. For CireSnave's decision.**
Evidence read at the head of the board-309 stack (conformance 0.6.25) on `origin/main` `bafbe07`; clause ids are quoted from
`spec/` as they stand there. Nothing here freezes.

## The question

You ruled Q2 = A on #12: no "missing kernel" feedback message, because a provider already sees every miss. You added a check:
**does the standard require every provider that claims a backend to cover the basic ops, so that a consumer who is declined
for a fused kernel can always fall back to the pieces?** If not, this note proposes a rule.

## The answer: no, nothing requires it

What the standard does say, and what each clause actually binds:

| Clause | What it requires | Who it binds | Does it make a provider cover the basic ops? |
|---|---|---|---|
| KISS-OPS-7.1-0001 | an implementation of KISS-Ops MUST evaluate every primitive-floor op | an *evaluator of the vocabulary* (a reference evaluator, a lifter, a checker) | No. It is about evaluating op semantics, not about holding compiled kernels. |
| KISS-OPS-7.2-0002 | any op-subset claim is prerequisite-closed under decomposition and MUST include the floor | an implementation *claiming an op subset* | No, same reason: a claim about vocabulary support. |
| KISS-SYNTH-7.1-0001 | answer every provision request with `{artifact, contract}` or a typed decline | a provider | No. A provider that answers every request with `CANNOT_PROVISION` conforms. |
| KISS-SYNTH-6.3-0003 | a provider without `PROVISION_ON_REQUEST` returns `UNKNOWN_STRUCTURE_KEY` or `CANNOT_PROVISION` for a cell it does not hold | a provider | No. It says how to decline a miss, not which cells must be held. |
| Umbrella 6.3 | a claim is prerequisite-closed over sub-standards, not over cells | any claimant | No: the unit is a sub-standard, and a cell is finer than that. |

So the fallback the question assumes exists on the *consumer's* side only: a consumer can resolve a non-primitive op through its
reference decomposition (KISS-OPS-6.14) down to the floor. The standard never promises that a provider holds a kernel for any
floor cell on any backend. A consumer declined for `add` then `relu` then `mul` can decompose, ask for the three pieces, and be
declined three more times.

## Why a blanket rule is the wrong shape

A provider's unit of coverage is a **cell**: a `structure_key` that fixes dtype, rank class, layout, work class, target
capability and more. "The basic ops" is therefore not a short list; it is the floor ops crossed with the dtypes and layouts a
consumer might ask for, per backend. And providers are legitimately narrow: a tensor-core GEMM provider, a quantised-matmul
provider and a sort-only provider are all useful and none of them can honestly cover the floor. A MUST on every provider would
either exclude them from conformance or be met by a token catalogue of trivial kernels. The suite's claimable-subset model
(umbrella 6.3) exists to avoid exactly that.

## Options

**A. Nothing.** Status quo. A consumer that needs the floor on a backend finds a provider that covers it, or runs its own
evaluator on the host. Cheapest; the question stays a deployment matter.

**B. An optional, claimable "floor-complete provider" profile.** A provider MAY advertise (capability bit in the experimental
range, so we can change our mind) that, for a named target namespace and a named dtype set, it can provision **every
primitive-floor op** for that pair. Backing: a KISS-Conform vector set that asks the provider for each floor cell in the
claimed set and expects a provision success (or build-on-miss success), never `CANNOT_PROVISION`. A consumer that sees the bit
knows decomposition always bottoms out at that provider; one that does not see it knows it may not. Narrow providers are
unaffected.

**C. A mandatory baseline for providers that announce a backend namespace.** The strongest guarantee and the one the question
suggests. It needs a definition of the baseline cell set per namespace, and the namespace vocabularies belong to their
maintainers (Classify 6.8-0004), so KISS would be pinning a set for `cuda` and `vulkan` that Baracuda and Vulkane own. It also
excludes narrow providers.

## Recommendation

**B.** It gives a consumer the thing it actually wants (a fallback that is known to terminate) without forcing a narrow provider
to pretend, and without KISS pinning a per-namespace cell list it does not own. I would write it only after Q1 (the optional
provision-by-definition branch) is built, because both use the same experimental capability range and a consumer that can send a
definition also wants to know which providers can build the floor. **If you prefer the status quo, A costs nothing**, and the
only loss is that the fallback is a convention rather than a checkable claim.

I have not checked how Fuel or Baracuda handle a declined fused kernel today; the example above is constructed from the
clauses, not observed.

## What freezes, and what to decide

Nothing freezes. Decision needed: **A, B or C**, and if B, whether the profile is wanted before or after Q1.
