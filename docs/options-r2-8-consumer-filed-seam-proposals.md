# Options note: R2.8, six seam proposals filed by the reference implementations

**Informative. No normative text. For CireSnave's decision; the PM turns each question into a board item.**
Read at `origin/main` `b9e08e7f`, 2026-10-26. Paths are `file:line` at that commit. Issues: KISS#11, #12, #18, #19,
#21, #27, all filed by Fuel or Baracuda while aligning their code to the specs. Roadmap row: R2.8. Nothing here
freezes; every option below is additive unless it says otherwise.

Two things changed since the issues were filed and they matter below: you ruled (decision 297) that the new
interface project holds both halves of each interface in code, generated from one definition; and KISS-Synth now
owns the table of `artifact_format_tag` values.

---

## Q1. Can a consumer send the *definition* of a kernel it wants built? (#11)

**Example.** Fuel's planner notices at run time that `add` then `relu` then `mul` can be one fused kernel. That fused
kernel has no catalog entry, so there is no identity to ask for. Today Fuel sends Baracuda the op graph itself (per the
issue, `jit.rs:40-45` in Fuel; I did not re-read that file). The spec does not allow that: a provision request
carries a `structure_key` and an op name and nothing else (`spec/synth.md:544`, KISS-SYNTH-6.1-0001a, "MUST NOT carry
... semantics fields"; `spec/synth.md:66` lists what Synth defines, "four things and nothing else"). So the reference
implementation's real build-on-miss path is not a conforming provision request.

**What already exists.** The emitter's whole input is already "an op definition plus a `structure_key`"
(`spec/emit.md:61`, `:126`). What is missing is only a way to carry that definition across the provision seam.

**Options.**
- **A. Keep requests identity-only.** Fused regions must be registered with the provider some other way first.
  Changes nothing; the dominant JIT case (fusing compilers) stays outside the standard.
- **B. Add an optional "provision by definition" branch.** A request may carry a KISS-Ops op graph, guarded by a
  capability bit so identity-only providers never see it. Changes 6.1-0001a (an exception) and adds one capability
  bit (FEAT bit 32 is `PROVISION_ON_REQUEST`, `spec/announce.md:590`; the experimental range is bits 40-43).
- **C. Make it an Emit-side profile instead.** Leave Synth alone and let the definition travel in a separate,
  Emit-owned request that returns the same `{artifact, contract}` frame. Same capability, different home; more
  new text.

**Recommendation: B, starting in the experimental bit range.** It is what the reference implementation already
does, the bit keeps every identity-only provider unaffected, and the experimental range lets us change our mind
before it becomes core. **What waits:** the op-graph encoding to carry (reuse the Grammar tag form, which already
exists, rather than invent one).

---

## Q2. Should there be a "missing kernel" feedback message? (#12)

**Example.** A consumer asks for 40 different softmax cells on `cuda:sm89` and a provider answers
`CANNOT_PROVISION` forty times. The issue wants a separate message the provider can aggregate to learn which kernels
to build next.

**What already exists.** A provider sees every request it receives, because the decline is its own answer
(`spec/synth.md:205`, the decline table). It can count misses by `structure_key` today, locally, with no message
from anyone. What it cannot see is demand the consumer never sent (the consumer saw a miss in an availability list
and did not ask). That is a small slice of the issue.

**Options.**
- **A. Add nothing.** Demand counting is an implementation matter. Say so in one informative sentence so the silence
  is not read as an oversight (the issue's "scope creep by silence" point).
- **B. Add a feedback message** `{structure_key | op graph, demand_count, first_seen, requested_target}`. A sixth
  message type: new tag, new vectors, new decline paths, and every provider must decide what to do with it.
- **C. Let a decline carry an optional demand field.** Smaller than B, but changes the decline frame bytes.

**Recommendation: A.** A provider already has the signal; the standard does not need to carry it. **This is a
prioritisation question, not a correctness one.** If a provider says it needs consumer-side demand, revisit with B.

---

## Q3. Is an in-process (no bytes) binding part of the standard? (#18)

**Example.** Fuel links its codegen into the same process, so it passes Rust structs and never serialises. It cannot
claim to implement a standard that is only defined as bytes without building an adapter it otherwise does not need.

**What already exists.** The suite is wire-first and says the transport is out of scope (`spec/umbrella.md:398`,
section 10.3). Your decision 297 settles most of this question: the interface project defines both halves of each
interface **in code from one definition**, with tests that prove the round trip (`docs/interface-project-plan.md`).
That is the in-process shape, and it is checked against the KISS vectors.

**Options.**
- **A. No new profile.** An in-process implementation conforms by producing, on the KISS vectors, the same
  observable results as a byte reader would. State that sentence once; the interface project supplies the trait shape.
- **B. A normative in-process profile in the specs** mapping each message to a trait or vtable. The spec would then
  describe a language-shaped ABI; the specs have so far avoided naming any language.
- **C. Leave it entirely to the interface project** and say nothing in the specs.

**Recommendation: A.** One sentence in the umbrella's conformance text, the code shape owned by the interface project.
It gives Fuel a conformance route without putting a language into the standard. **What waits:** the interface project
plan itself (`docs/interface-project-plan.md`); this answer depends on it existing.

---

## Q4. How does a consumer learn how to *call* a kernel that is not a C function? (#19)

**Example.** A Vulkan provider's kernel is launched with descriptor sets, not typed pointers. The Interface section
is a positional list of pointers and scalars (`spec/contract.md:854-872`, KISS-CONTRACT-6.5-0001 and -0002), so it
cannot describe that kernel. The issue proposes an `abi_model` field with values such as `pointer-abi`,
`descriptor-set`, `buffer-index`, `host-closure`.

**What already exists.** KISS-Consume never produces an artifact (`spec/consume.md:615`, KISS-CONSUME-6.2-0004), so
the binding step is outside every sub-standard. But the provision frame now carries an `artifact_format_tag` that
names the artifact's binary format and target family, and Synth owns the table of tags (KISS-SYNTH-6.4-0001g, once
merged). The only tag today is `PTX `, a pointer ABI.

**Options.**
- **A. Declare the Interface a pointer ABI** and say plainly that descriptor-set and buffer-index providers are
  outside this version.
- **B. Add an `abi_model` field to the Interface section.** A new field changes the contract bytes: Breaking
  (`contract_version` bump) and every contract regenerates.
- **C. Put the ABI model in the tag table.** Each seeded `artifact_format_tag` row names its ABI model (PTX:
  pointer; a future SPIR-V row: descriptor set). No new contract field, no byte change, and a consumer that does not
  support a tag already declines it (6.4-0001g).

**Recommendation: C, and only when the second tag is seeded.** A tag already names format and target, so the ABI
model belongs with it; this costs nothing now and avoids a Breaking contract change. **Not a decision yet:** with one
tag in the table there is nothing to decide. Record the rule ("a new tag row must name its ABI model") so it is not
forgotten.

---

## Q5. What does a reader do with an enum value it does not know? (#21)

**Example.** A newer provider sets a layout code `paged`. A consumer that ignores unknown values treats the operand
as dense and computes wrong results.

**What already exists, which answers most of this.**
- Unknown **capability bit**: ignored (`spec/announce.md:622`, KISS-ANNOUNCE-7.2-0007). Right, because bits only advertise.
- Unknown **field** or unknown **version** in a contract: typed decline, never ignored (`spec/contract.md:561-570`,
  KISS-CONTRACT-6.1-0006, and 6.1-0003). Forward compatibility is a version bump.
- Unknown **dtype token**: typed decline (KISS-CLASSIFY-6.7-0009).
- Unknown **determinism class** value inside a contract: typed decline (reference reader,
  `conformance/src/contract.rs`, decline tag `unknown-determinism-class`).
- Unknown **decline code**: treated as a generic decline (`spec/announce.md:497`, KISS-ANNOUNCE-6.4-0010).

So the Fuel concern is already covered wherever a value that changes correctness sits in a closed field. What is
missing is one stated rule, so a new field does not have to rediscover it.

**Options.**
- **A. State the rule once** (a short clause in Contract and Consume): an unknown value of a closed enum that affects
  correctness or admissibility is a typed decline and makes the artifact unavailable; only an advisory field may be
  ignored. Records what the specs already do per field.
- **B. Leave it per-field.** No new text; each new field must say its own rule.

**Recommendation: A.** It is one clause, no byte changes, and it turns a convention into something a new field
inherits. **Mostly not a decision:** it records existing behaviour.

---

## Q6. Should a provider be told to understate, never overstate, its guarantees? (#27)

**Example.** A provider is unsure whether its floored-remainder kernel is within 2 ULP or 4. If it declares 2 and the
truth is 4, a consumer binds the kernel where it is wrong. If it declares 4, the cost is a missed opportunity.

**What already exists.** Part 2 of the issue (an honest "no finite bound declared") is **already answered**: the
closed precision-class set includes `unbounded`, "maps to no declared tier" (`spec/ops.md:915-949`, KISS-OPS-6.8-0007
and -0009). What is still unclear is how that sits with `spec/contract.md:1136-1145` (KISS-CONTRACT-6.8-0002), which
says the tiers "MUST declare ... at least one of max_ulp, max_relative, max_absolute"; a provider with no finite bound
reads it as forced to invent one. KISS-CONTRACT-6.8-0010 (`spec/contract.md:1224`) yields `unaudited` when no bound is declared, which
suggests absence is allowed.

**Options.**
- **A. Add the SHOULD** (understate under uncertainty, never overstate) and fix 6.8-0002 to say "at least one ... or
  the class is `unbounded`".
- **B. Add only the 6.8-0002 fix.** The direction advice is good practice but unenforceable, so leave it out.
- **C. Neither.**

**Recommendation: A.** The SHOULD costs one sentence and names the safe direction; the 6.8-0002 wording gap is a real
inconsistency worth closing either way. **The 6.8-0002 fix is not a judgement call**; it is a defect to correct.

---

## Summary table

| Q | Issue | Recommend | New bytes? | Already partly answered? |
|---|---|---|---|---|
| 1 | #11 provision by definition | B (experimental bit) | no (new optional frame body) | Emit already takes the definition |
| 2 | #12 feedback message | A (nothing) | no | provider already sees the misses |
| 3 | #18 in-process profile | A (one sentence) | no | decision 297 / interface project |
| 4 | #19 `abi_model` | C when a 2nd tag lands | no | tag table |
| 5 | #21 unknown enum value | A (state the rule) | no | per-field rules exist |
| 6 | #27 understatement | A | no | `unbounded` class exists |
