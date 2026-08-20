# Contributing to KISS

KISS (Kernel Interface Standards Suite) is developed in the open and stewarded by
[ThinkersJournal](https://github.com/ThinkersJournal). Comment and proposals are welcome
now, while every document is still a pre-1.0 draft and nothing is frozen.

This file is a short operational guide. The authoritative governance and legal terms live
in the umbrella specification: [`spec/umbrella.md`](spec/umbrella.md) §7 (Governance) and
§9 (Legal).

## How to participate

- **Comment or ask a question** — open an issue. Point at the document and clause ID
  (e.g. `KISS-ANNOUNCE-6-0003`) so the discussion is anchored to specific normative text.
- **Propose a change** — open an **RFC issue** (use the *RFC* issue template) for anything
  substantive: a new clause, a wire-format change, or a vocabulary addition. The RFC *is*
  the issue (see [The RFC lifecycle](#the-rfc-lifecycle) below). Explain the problem before
  the solution. Small, obvious fixes may skip straight to a pull request.
- **Report an interoperability failure** — if two implementations read the same clause
  and behaved differently, that is a defect in the *specification*, not just the code.
  File it against the clause; ambiguity that admits two readings is a bug we want.

## Roles and process (summary of umbrella §7)

- Each sub-standard has an **editor of record** who is responsible for its text and for
  integrating accepted RFCs.
- Substantive changes go through a lightweight **RFC**: a written proposal that interested
  parties may co-sign and comment on. The editor integrates or declines with a rationale.
- Anyone may implement, comment, and co-sign. Stewardship is about maintaining a coherent,
  testable, vendor-neutral specification — not about gatekeeping who may use it.

## The RFC lifecycle

A KISS RFC **is a GitHub issue** labeled `rfc`. The issue is the durable record of the
change: the problem, the discussion, who co-signed, and the final accept-or-decline
rationale all live in one place and stay visible after the issue is closed. Open one with
the **RFC** issue template (or **Spec defect / interoperability failure** for a clause that
is ambiguous or self-contradictory).

An RFC moves through these states:

1. **Proposed** — the issue is opened with the problem, the affected clause IDs, and the
   proposal. Interested cosignatories (the providers, consumers, and emitters it affects)
   comment and co-sign. Propose-first: float the RFC before it is wired.
2. **Discussion** — the **editor of record** for each affected sub-standard and the
   cosignatories converge on a decision in the thread. Disagreement is resolved here.
3. **Accepted** — the change is authored as a **pull request that cites the issue**
   (`Refs #NNN`), adds the clause text **and** its mapped KISS-Conform test, and — for a
   heavyweight design — an accompanying `rfcs/<slug>.md` document. Merging the PR and
   closing the issue records acceptance. A cross-party-visible wire/ABI schema-version bump
   is coordinated across affected parties before it lands.
4. **Declined** — the issue is closed with a comment stating why. The reasoning stays
   visible; a dropped idea is recorded, not silently forgotten.

This is the process the umbrella describes in §7.2: the GitHub issue tracker **is** the
ThinkersJournal RFC directory of record.

## Maturity and the freeze gate (summary of umbrella §5)

Each sub-standard is versioned independently and moves through maturity stages. Advancing
**Draft → Frozen** requires passing the **freeze gate**:

1. at least **two dissimilar, independently developed implementations** demonstrating the
   normative behavior, and
2. an **adversarial-outsider review** — a reader who did not write the text attempts to
   implement it from the document alone and reports every ambiguity, every under-specified
   value, and every place two conforming implementations could diverge.

A frozen clause does not change incompatibly; growth happens through additive versions and
the extension registry, never by silently redefining frozen text.

## Evidence conventions

These are process rules, not normative clauses. Each was adopted after a real failure, and the
motivating incident is named so the rule reads as a lesson rather than as ceremony. They share
one general form: **a state derived from an artifact's *existence* is not a state derived from
its *content*.**

**1. A claim that gates a decision states its method, and what that method did not examine.**
Coverage figures, audit counts, byte-match leg reports, "verified clean." *Why:* two separate
sweeps were reported as complete when each had searched one axis — one for dtype spellings but
not version prefixes, another for an uppercase `SKIP` marker but not the lowercase idiom the
project actually used. The second was wrong by 6.6×. **Composing the exclusion sentence is what
surfaces the gap**; a stated-scope 12 is worth more than an unscoped 0.
See `KISS-CONFORM-6.1-0011`.

**2. A verification report names the exact command and flags that produced it, and reads the
ran-count rather than the colour.** *Why:* `cargo`'s `--all-targets` does not imply
`--all-features`, and one project's wire codec sat behind a non-default feature — a six-crate
sweep would have reported green having compiled none of the code under test. It surfaced only
because someone saw `running 0 tests; 737 filtered out` and refused to read it as a pass. For
device- or toolchain-gated suites, `0 filtered out` is the discriminator.

**3. "It has a review" is not "the review was read."** Before reporting a PR merge-ready, fetch
its review state and address or explicitly disposition every finding — fixed, stale, out of
scope, or declined with a reason. *Why:* six PRs were reported merge-ready on the strength of
`reviews=1`; twelve inline findings sat underneath, eleven unaddressed, including a false claim
in a document that had already merged. **A green-looking review count and an actually-read
review are the same bytes.** Note also that a **draft** PR receives no automated review and
sends no notifications: work being finished and a PR being reviewable are different states.

**4. Verify against the merge tree with the full gate — not the branch with a subset.** CI tests
`refs/pull/<N>/merge`; fetch that ref. *Why:* a lint comparing two spec tables passes on `main`
(where both are one vocabulary), passes on the branch in isolation, and fails only on the
integrated tree. A four-of-seven-step local run reported "all gates green" while the three unrun
steps held real drift.

**5. A stated zero is a result; silence is not.** When asked for a count, report it — including
zero — with the structural reason. *Why:* two projects audited to zero and both were *findings*:
their components take no environment input, so the defect class has nowhere to live. **Immune is
not the same as lucky**, and only the reason distinguishes them.

**6. Independence of runs is not independence of instruments.** Two people running the same check
and agreeing have tested the check once. *Why:* a dtype lint reported CLEAN to its author and
CLEAN to an independent reviewer while parsing one table concatenated with another and comparing
the result against itself. This is the freeze gate's own problem at small scale — the gate counts
≥2 dissimilar implementations, but implementations sharing one comprehension lineage overstate
the weight of their agreement in exactly the same way.

**7. Name the enforcing instrument, then ask whether it can observe the property at all — before
asking whether it currently does.** *Why:* a clause requiring per-run crediting was recorded backed,
then corrected to unbacked, before anyone asked the prior question: its sole enforcing instrument is
a *static* reader that parses specification markdown and Rust source and never executes the harness,
so "did this test run in this run, or decline at the gate?" is outside what it can see in principle.
A declaration that a test *may* skip is not an observation that it *did*. "Which tool enforces this?"
gets asked routinely; **"can that tool, in principle, see this?" does not** — and a clause its only
enforcer structurally cannot observe will be recorded wrong repeatedly, each time by a different
reader acting in good faith. When the answer is no, the fix is a second instrument of the right
class, not a weaker clause fitted to the instrument on hand.

**8. A round-trip proves internal consistency, not external agreement.** Encode-then-decode, or
emit-then-parse, tells you an implementation agrees with *itself*. It cannot detect that the whole
vocabulary moved. *Why:* a crate-side-only rename of an emitted vocabulary passed **20 of 21**
existing tests, because every one of them round-tripped through the same renamed table; the
disagreement with the published document was invisible from inside. **The twenty-first is the part
that matters:** it failed for an unrelated reason — a sort-order expectation that broke only because
the old and new spellings sort differently — so a maintainer would have corrected the sort and never
learned the vocabulary had moved. Partial coverage that fails for the wrong reason is worse than
uniform silence, because it routes the reader confidently away from the defect. The same blindness
is why the suite cross-verifies tokens between independently derived implementations rather than
trusting any one implementation's self-consistency — **a systematic rename is exactly the error a
round-trip is constitutionally unable to see.** Compare the *token image*, not the variant count: an
implementation may legitimately hold more internal variants than there are tokens (one folding to
another's canonical spelling), so comparing counts manufactures a divergence that isn't there while
missing the one that is. Both halves collapse to one instruction: **compare what you emit against
something you did not write** — and if the thing you did not write is a document a human retypes
into a test, you have moved the transcription, not removed it.

**9. A seeded mutation must assert that it applied.**
An unapplied mutation is indistinguishable from a guard that does not fire. Proving a test can fail means breaking the thing it checks and
watching it go red. If the edit silently matched nothing, the "mutation" runs against unmodified
source and the test passes **because there was no defect to catch**, which reads exactly like a test
with a hole in it. *Why:* a mutation vector reported green twice, and the honest reading — *"my test
does not catch a silent accept"* — was wrong about the **instrument**, not the test: a `\n` in the
search string was mangled before it reached the replacer, so nothing matched. **The error points the
investigation at the code, which is fine, and away from the tooling, which is where the fault was.**
Assert the pattern matched exactly once before replacing.

Note the asymmetry, because it bounds how much older proofs are worth: **a red result is
self-validating** — the test could not have failed unless the patch applied — **while a green result
proves nothing until the application is checked.** Proofs run before this convention are trustworthy
exactly insofar as every vector came back red.

And for a test credited to more than one clause, per-vector proofs are necessary but not sufficient:
publish the **isolation matrix** showing each mutation fails **exactly one** test. That no two fail
together is the only thing distinguishing N properties from one property credited N times.

**A mutation that applied is not yet a mutation that tested what you meant.** Asserting the patch
landed proves the file changed; it does not prove the *installed* text has the property under test.
*Why:* a narrow pattern was to be justified by seeding the broad one in its place. The mutation
applied, behavior changed, and a test went red — and the evidence was worthless, because the
installed pattern did not match the line the whole argument was about, so it never exercised the
claim; the red came from an unrelated assertion. **A green mutation proves nothing and a red one
proves only that *something* broke** — which is the thing it was meant to establish only when the
mutation actually reaches the behavior in dispute. So the applied-check must assert the property,
not the diff: install the mutation, then — **before running the suite** — call the mutated predicate
on the exact input the claim names and confirm it answers wrongly. In the case above that is one
line, `RE_WORDING.search("MUST be NaN-propagating (not IEEE maxNum)")`, expected `True` from the
broad pattern and actually `False`; run after the suite instead, and the same fact arrives as a
confusing red in an unrelated test. **The difference between the two is when you learn it, and
whether you learn it at all** — a red that looks like the one you predicted invites no further
questions.

**Undo a seeded mutation by reversing that edit, never by discarding the file.** `git checkout --
<file>` restores the *committed* state, so when the file also holds the uncommitted work being
proven, the mutation and the fix go together — and the suite then fails for the honest reason that
the fix is gone. *Why:* exactly that, on the flip for a newly added guard: the tool reverted, the
test kept its new control, and the re-run went red. It was caught only because the failure
**contradicted a green verified minutes earlier** — the same contradiction-with-a-verified-result
signal that catches a stale read, and the only one that fires here. Reverse the specific edit, or
stash and pop.

**10. Close the class, not the instance.**
A class guard covers cases its author never imagined, including ones that did not exist yet. Fixing
the instance in front of you leaves the next one to
be found by whoever trips over it. *Why:* a reviewer found one `tools/test_*.py` that pytest
collected zero tests from. The instance fix was one file; the class fix was a CI guard failing **any**
`test_*.py` that collects nothing. Two days later a **new** tool — the citation audit, written to
check the integrity of every coverage figure the project quotes — shipped with the same defect in
**its own controls**, and the guard caught it. **The guard knew nothing about citation audits. It
knew that a `test_*.py` collecting nothing is a lie regardless of what it tests.**

That is the return a class guard pays and an instance fix cannot: **it catches a defect in the
controls of an instrument that did not exist when the guard was written** — the instrument that
proves an instrument. Note also what it cost to learn: the instance was found by a **reviewer**, not
by any check the project owned. Closing the class is what converts one reviewer's catch into a
standing property.

**11. A check can run correctly on the wrong representation.**
Conventions 1–10 catch a check that examined *nothing*. This one catches a check that examined
*something* — correctly, exiting clean — that was not the thing the claim is about. It yields a
**plausible wrong answer** rather than a vacuous pass, so every guard built for the earlier
conventions passes it through: the test ran, the ran-count is non-zero, the mutation applied, the
instrument was capable of seeing the property. It simply was not pointed at it. *Why:* four
instances in one day, on four different axes.

- **Representation.** A `diff` reported `1,186c1,186` — the entire file — and was read as content
  drift. It was the **line-ending signature** — CRLF on disk against an LF generation; re-measured
  with line endings normalized, all three artifacts were fresh. Reporting the first result would
  have triggered work on a defect that does not exist.
- **Level.** A freshness gate compared `json.loads(committed)` against `json.loads(fresh)` —
  **parsed content** — while the property consumers depend on is **the bytes they hash**. Both are
  "is the artifact fresh"; only one is the claim.
- **Role.** Two dtypes were reported unreachable after checking reachability *in principle*. They
  construct fine as operand 0; they are unreachable only in the **sibling-operand role** the
  specification assigns them. The clause answered a question one role over.
- **Axis.** *Spelled* (the implementation holds the dtype and emits its token) was read as
  *derivable* (it can construct a cell that places that token in a key). Different measurements,
  one label — and the wrong one made "add the missing dtypes" look like the remediation for a gap
  that was never about dtypes being absent.

**The discriminator, before running anything: name the representation, level, role, and axis the
claim is about, then check that the instrument's input is that one.** And note what does *not*
work — this class survives more care, and it survives a second run by a second person
(convention 6), because the instrument is behaving correctly. **The only thing that reliably
catches it is measuring twice by different means.** Three of the four above were caught that way;
the fourth was caught by another party measuring their own project after being told what it
contained.

**12. A "blocked on X" record is a claim with no expiry, and it ages into a confident lie.**
Conventions 1–11 are about instruments that measure the wrong thing. This one is about a claim
**no instrument is pointed at at all**: a line in a register, a tracker entry, a comment saying
*we are waiting on someone else*. It is written once, believed indefinitely, and **nothing in your
own repository changes at the moment it stops being true** — so no control you own can fire. It
then propagates, because a recorded blocker reads as a fact about the world rather than as a
prediction nobody has rechecked.

*Why:* four instances in one day, across four projects, in four distinct shapes. Two were found
only because an outsider happened to ask.

- **The trigger names an issue instead of the ruling.** A register said a decision *"cannot be
  Accepted without cosign"* — for a ruling that had merged the previous day, whose clauses that
  same project's code **already consumed**. Machine-sweepable: the trigger cites an artifact you
  can look up.
- **The trigger names a predicted coupling that never materialized.** *"Whether we import a table,
  a grammar, or both changes what the replacement has to hold."* It didn't — the replacement
  declined to hold vocabulary at all, so the coupling dissolved. **Not sweepable:** there is no
  artifact to look up, because the design it predicted was never made. Hence — **a dependency
  between two pieces of work is a claim about a design that hasn't been made yet, and it expires
  when the design does.**
- **The tracker asserts the opposite of the repository.** An **open**, `rfc`-labeled issue titled
  *"three unpinned decisions — the last is a freeze-blocker"*, filed **fifteen days after** all
  three were ruled and merged. Worse than a stale document: **a tracker is the instrument you
  consult specifically to avoid trusting your memory**, so it launders a stale belief into a cited
  fact. The merged clause set is the authority; the tracker is a claim about it.
- **The watch list is itself a cached claim.** A subscription naming the files you depend on drifts
  the moment you stop depending on one — and it fails toward **silence**, the direction nobody
  investigates.

**Carry both halves explicitly marked by which is detectable**, or someone will sweep for the
undetectable one and conclude they are clean.

**The discriminator: a "blocked on X" record must name the artifact that will exist when the block
lifts — a clause ID, a published version, a file — never the issue where the question was asked.**
An issue closes for many reasons; a clause either exists or it does not.

**Then make it a guard that can fail: assert the absence.** If your register says *blocked on
clause X*, have your own CI assert that clause X is **absent** from the specification you pin.
When the ruling lands and you bump that pin, the assertion goes red and tells you your blocker
resolved. Two properties earn it a place here: it is owned by the party that **holds the belief**,
so it cannot drift away from what it protects; and it **can fail**, which is what separates a guard
from a decoration.

Two honest limits, both learned by getting them wrong:

- An absence assertion against a **vendored pin** tests the pin, not upstream. It fires at bump
  time, not at merge time. For this failure mode that is the correct moment — you learn when you
  would act on it — but left unstated it becomes another guard that looks like it works.
- An absence assertion written against **guessed** names passes forever, asserting the absence of
  something that was never going to exist under that spelling. **Name the thing precisely, or the
  assertion is decorative.**

*What this cost:* in every instance the waiting party had already shipped releases carrying the
answer while advertising that it lacked it. In one, the ruling was three weeks old and **the only
thing between the project and it was a message** — the reconciliation shipped within the hour of
being told. **None of the four was caught by any check any project owned.**

**13. When a fix adds a new outcome, prove the OLD outcomes are still reachable.**
Conventions 1–12 point falsification at the *defect*: can this check catch the bug? This one
points it at the *remedy*: **can this fix now miss a different bug?** A control that verifies the
reported symptom is gone will happily certify a fix that broke something else — because the
symptom **is** gone, and the tool **does** say what was asked of it.

*Why:* the coverage ratchet reported `VIOLATIONS FOUND` for a *usage* error — a missing
`--base-ref` — which is a false alarm in the tool whose own header says *"a check that has always
been red teaches everyone to ignore it."* The obvious fix wrapped the refusal so a missing base
reported INCONCLUSIVE instead of failing. The symptom vanished. It also nested the **count**
comparison inside the same `else`, so **any genuine floor regression run without a base returned a
soft non-answer.**

**That is strictly worse than the false alarm it replaced.** The false alarm cried wolf; the fix
turned a missing flag into **a way to mask a regression** — and it would have shipped, because
everything asked of it passed. What caught it was a second control asking whether a **real breach
with `--base-ref` absent** still reported VIOLATIONS. It did not. `rc=2`.

**Use the mechanical form, not the general one.** *"Add a control for the failure mode the fix
could introduce"* is true and nearly useless when you are the author: you have just spent an hour
convincing yourself it introduces none. The askable version is:

> **The fix widened the result space. For each outcome that existed before, is it still
> reachable — and is there a control that says so?**

A fix that adds a third exit state must prove the first two still occur. A fix that adds a new
decline variant must prove the old ones still fire. **Widening a result space is the specific
shape that silently narrows another.**

**14. The natural check after an action often answers a different question than the one you asked.**
Distinct from convention 11 (a check run on the wrong *representation*): here the instrument is
correct and healthy, and the *question* is mis-posed — so the answer is both accurate and
misleading. **The error direction follows the direction of the imprecision, not the kind of
check.** A validator that is too **loose** accepts a superset and reassures you falsely; one that
is too **narrow**, or pointed the wrong way, misses what is there and **alarms** you falsely.
Both produce a confident wrong answer, and neither announces which it is. **A check that validates
a MEASUREMENT is the case that costs most** — it can discard correct work or manufacture absent
work — and it has its own treatment below.

- **After a squash merge, `git branch -r --contains <sha>` reports the branch commit as ABSENT
  from `main`.** The content landed; the commit identity did not. The obvious post-merge check
  reads as *"the wrong commit was merged."* Ask whether the **content** is on `main` — compare the
  file, or read the merge commit's message — not whether the SHA is an ancestor.
- **A PR that sat through another merge carries a stale floor BY CONSTRUCTION, not by mistake.**
  One PR's floor figure was re-derived three times and the arithmetic was right every time; the
  **base** was stale twice. The number was never wrong. **Re-derive at the actual merge base
  immediately before merging**, and treat any interleaved PR as stale-by-default rather than
  suspect-on-evidence.

**The discriminator: name what the check's answer is ABOUT, and confirm that is what you wanted to
know.** A green from the right instrument on the wrong question is indistinguishable, at a glance,
from a green on the right one.

**And the sharpest case is when the mis-posed question is the one VALIDATING a measurement.**
A sweep found 53 clauses naming a test that does not exist. Spot-checking it,
`grep -rq "fn test_ops_add_sub_mul"` returned **YES** — apparently falsifying the sweep. It was
prefix-matching `fn test_ops_add_sub_mul_wrapping`. **The spot-check was wrong, not the sweep**,
and a correct finding was one keystroke from being retracted.

> **A check used to validate a measurement must be at least as precise as the measurement — and
> the natural quick check is reliably less precise, not more.**

That asymmetry is what makes it a rule rather than an anecdote. **Nobody reaches for a *more*
rigorous instrument to spot-check something; the whole point of a spot-check is that it is
cheap.** So validation drifts systematically toward looseness — which yields false *reassurance*
when the measurement said "clean" and false *retraction* when it said "defect." The second is
rarer and more expensive: it discards work that was right.

**Two instances, two different loosenesses, one afternoon, found independently** — which is what
makes this a class rather than a grep anecdote:

- **A looser PATTERN.** `grep -rq "fn test_ops_add_sub_mul"` prefix-matched
  `fn test_ops_add_sub_mul_wrapping`. The validator accepted a superset of what it was asked
  about.
- **A wrong DIRECTION.** A `cited_in` heuristic searched **backward** from a citation for its
  enclosing `fn`, and so attributed every doc-comment citation to the **previous** test —
  because those citations *precede* the test they belong to. The validator scanned the wrong
  way past its target.
- **A pattern too NARROW — the opposite direction, and the one that manufactures work.** An
  issue's own evidence was `grep -rn "6\.5-0011" spec/*.md` *"returns nothing."* Re-run, it
  returns **seven** hits across two documents. The grep had matched nothing it should have, and
  the issue reported a clause as undefined that was defined twice over. **A validator can invent
  a defect as readily as it can hide one**, and this one sent a lane to fix something half absent.

Neither is a careless grep. Both are validators built in a hurry against a measurement built
carefully, which is the asymmetry.

**Anchor the pattern when you verify a measurement** — `fn <name>\s*\(`, not `fn <name>` —
and when a spot-check contradicts a measurement, **suspect the spot-check first.** It is the
instrument you spent less on.


If you propose normative text, follow the house style so it stays testable:

- Use RFC 2119 / RFC 8174 keywords (MUST, SHOULD, MAY) with their exact meanings.
- Keep §0–§5 informative and §6+ normative; do not smuggle requirements into the overview.
- Give every normative requirement a clause ID `KISS-<SUB>-<section>-<nnnn>` and a
  corresponding conformance test. **A MUST with no test fails the suite build.**
- Pin values (byte offsets, magic constants, enumerations) exactly. No unquantified
  adjectives ("fast", "small", "reasonable") in normative text.
- Keep project and product names out of normative clauses. They may appear only in
  non-normative examples, provenance, acknowledgments, and the signatory record. Normative
  text uses the generic roles: *provider*, *consumer*, *implementation*, *kernel*,
  *contract*, *target*.

## Contributor licensing terms (summary of umbrella §9)

By contributing to this repository you agree that:

- **Your contribution to the specification text is dedicated to the public domain under
  [CC0 1.0 Universal](LICENSE)**, the same terms as the rest of the specification.
- You grant a **royalty-free license to any of your essential patent claims** necessary to
  implement the contributed specification text, subject to **defensive termination** (the
  grant to a party ends if that party sues asserting that a conforming KISS implementation
  infringes its patents). CC0 waives copyright but not patents, so this grant is made
  separately and is bound at contribution time.

If you cannot make those commitments for a given contribution, say so in the PR and we will
work out how to proceed — do not merge text you cannot license this way.

---

*Thank you for helping make the kernel interface a commons instead of a thicket of private
glue.*
