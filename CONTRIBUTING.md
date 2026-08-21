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

**Find the one you need by SYMPTOM, not by number.** This list exists because the conventions are
now numerous enough that people re-derive rules already written here — including their authors.
On 2026-08-20 the architect proposed three "new" conventions in one evening and **two were
already in this file**, both inside convention 9 — one of them added there five days earlier by
the very amendment being re-proposed (`8dfc56e`, #226). **The failure was never coverage — it was
retrieval.**

| the thing that just happened | convention |
|---|---|
| my check examined nothing / passed vacuously | 1–10 |
| my seed didn't apply, or applied and proved nothing | **9** |
| my seed applied and went RED — for the wrong reason | **9** (second half) |
| the check ran fine but on the wrong bytes / level / role / axis | **11** |
| I read an exit code, a return value, or a log line instead of the state | **11** (*level*) |
| the check is correct but I asked it the wrong question | **14** |
| my query returned NOTHING — is that an absence, or a broken query? | **14** (run a positive control) |
| I truncated with `head`/a cap/a size limit — may I conclude absence? | **14** (**no** — positive findings only) |
| my seed matched a DOCSTRING and left the code untouched | **9** (assert the file changed, not just that the pattern matched) |
| my fixture is green — but does the thing it names actually exist? | **9** (a vacuous fixture passes either way; pair it) |
| a "blocked" note that nobody has re-read since it was written | **12** |
| a fix added an outcome and I didn't check the old ones still happen | **13** |
| a clause id appears in a test — does it count? | **15** |
| the record is stale, unattributed, unenforced, or ambiguous | **16** |
| I am about to tidy something that looks redundant — is it? | **17** (the defence should have told you; if it did not, measure) |
| my comment says "don't simplify this" and gives a reason | **17** (a reason is believed; a measurement is checked) |
| I am adding a guard because another gate has one | **17** (which STRUCTURE needs it? analogy is cargo cult) |

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

Note the asymmetry, because it bounds how much older proofs are worth: **a green result proves
nothing until the application is checked**, while a red result proves *something* broke. **A red is
NOT self-validating** — an earlier draft of this convention said it was, and the statement later
in this same convention beginning *"A green mutation proves nothing and a red one proves only
that something broke"* already refuted it. (**Named rather than located on purpose:** the first
draft said *"the paragraph four below"*, which was already wrong when it was reviewed, because
editing the file moved the paragraph. A positional reference inside the convention titled
*Evidence has a LOCATION* is the rule failing on itself — see 16(a): name the thing, not where it
sits.) *Why the correction:* a seed wrote `return []` into a function returning
a **dict**; four controls reddened on the resulting type error, and "caught by 4 controls" was
nearly reported as evidence of discrimination. The seed applied, the anchor matched, `SEED APPLIED`
printed — **and the red was worthless.** Re-run type-correctly, exactly ONE control reddened.
**A seed must be type-correct and semantically plausible, not merely applied**, and an ill-typed
seed is more seductive than a false green because *a red reads as the control working.* What
caught it was noticing that tests named `ignores_*` failing under a detector that ignores
everything made no sense — a semantic smell, not a mechanical check.

And for a test credited to more than one clause, per-vector proofs are necessary but not sufficient:
publish the **isolation matrix** showing each mutation fails **exactly one** test. That no two fail
together is the only thing distinguishing N properties from one property credited N times.

**Asserting the PATTERN MATCHED is not asserting the FILE CHANGED**, and the two come apart
exactly where it hurts: a pattern that also occurs in a **docstring** or a comment matches there
first, `replace(..., 1)` edits prose, and the code is untouched. *Why:* a seed aimed at a
fail-closed guard hit the same string in the function's own docstring; the suite reported **18
passed** and the natural reading was *"the guard is not exercised."* Assert **both** — that the
anchor exists **and** that the resulting text differs (`assert new != old`).

**A PAIRED CONTROL CATCHES A FIXTURE THAT IS TESTING NOTHING, not only an over-eager fix.** A
control naming a subject that does not exist — a clause absent from the ledger, a symbol absent
from the tree — **passes whichever way its assertion points**, so a whole set of them can ship
green against nothing. *Why:* a fixture was vacuous through more than one revision
(`KISS-OPS-6.99-0003`, then `-0044`) before the tool's own output revealed the real id; **the first version was green**, and
had the assertion been written the natural way, four controls would have shipped against a clause
that was never in the ledger. **What caught it was one control pointing the other way and
failing** — because **a vacuous fixture cannot satisfy both directions at once.** So: before
trusting a new fixture, confirm its subject actually exists in the thing it is fixturing.

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
- **Level, again — a return value is not a state.** A threaded reply was posted by API; the
  second call failed because the body contained a **backslash followed by a backtick**, and JSON
  permits only a fixed escape set (`\"` `\` `\/` `\b` `\f` `\n` `\r` `\t` `\uXXXX`) — that pair is
  not among them, so the payload was malformed. The only signal was a bare non-zero exit. **"The command returned" and "the thing happened" are different
  levels**, and reporting the first as the second would have claimed two dispositions with one
  posted. Caught by counting the replies afterwards. **After any action whose success you intend
  to report, measure the state rather than read the return.**
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
- **AN EMPTY RESULT IS A CLAIM ABOUT YOUR QUERY, NOT ABOUT THE WORLD — three instances in one
  evening, none of them careless.** A grep over four phrasings of an obligation returned nothing
  and was reported as *"nowhere stated"*; the clause existed and said it in different words, and
  a new clause was nearly written to duplicate a normative obligation. An API call failed on an
  invalid JSON escape and the only signal was a bare exit code — "both dispositioned" was nearly
  reported with one posted. A subprocess died on a `cp1252` decode error and its empty output was
  read as *"no threaded replies"* on two PRs at once. **In all three the instrument returned
  successfully-looking emptiness**, and in none of them had anyone asked whether it could have
  returned anything at all.
  **THE RULE THAT NEEDS NO ONE TO NOTICE ANYTHING — a constraint on what an output LICENSES:**
  **a truncated or crashed query can support a POSITIVE finding and never a NEGATIVE one.**
  `head -N`, a size cap, a dead subprocess, a silently-cut heredoc — any of them can show you that
  something *exists*; **none of them can show you that nothing does.** In every instance below the
  error was the same: **a bounded output read as an exhaustive one.** So: **if you truncated, you
  may not conclude absence** — re-run unbounded, or run a positive control.
  *The instances below — three agents, one evening:* a `head -6` concluding a string was absent from a
  list that was cut before reaching it (**it was there, eleven lines further down**); a `head -2`
  on a merge gate, which is how a PR was merged and then reported as prevented; a `cp1252` decode
  crash whose empty output printed as a clean *"no threaded replies"* **on two PRs at once**; a
  heredoc that truncated at an apostrophe and wrote a 26KB file as 7.6KB, caught only by checking
  the byte count; and a grep over four phrasings of an obligation reported as *"nowhere stated"*
  when the clause existed in different words.
  **And one sub-case where checking the return code would NOT have helped: the failure can be in
  the PAYLOAD rather than the call.** A PR body passed inline to a shell had its backticked spans
  evaluated as command substitution — one span ran as a command, `not found` went to stderr, and
  **the span was replaced with nothing.** The tool created the PR and reported success; the only
  signal was stderr scrolling past a successful call. **A document arguing that a truncated output
  cannot license a conclusion was itself about to ship truncated.** Caught by reading the state
  back, which is the only check that covers this: **the call succeeded, so neither an exit code
  nor a positive control would have fired.**
  **And it runs in BOTH directions — the payload you SEND and the payload you RECEIVE.** A merge
  gate queried a PR's review comments, **the API returned an incomplete list and reported
  success**, and the gate cleared a PR that had two undispositioned findings on it. **Exit 0, no
  truncation, no crash, nothing malformed** — the tool simply answered with less than it had.
  Reading the state back does not cover this one either, because *this was* reading the state
  back. **What covers it is asking a second time and comparing**, which is what that gate now
  does. **A single successful read is a sample, not a census.**
  **The remedy is one extra call: a POSITIVE CONTROL.** Run the same query against something you
  know is there; if it finds that, the empty result is evidence. A consumer answering a
  reliance question the same day did exactly this unprompted — *"the same pattern DOES find a
  `link.structure_key` read, so the near-empty result is absence, not a broken query"* — and that
  is the sentence to copy. **A null result reported without a positive control is a failure to
  find, described as a finding.**

Neither is a careless grep. Both are validators built in a hurry against a measurement built
carefully, which is the asymmetry.

**Anchor the pattern when you verify a measurement** — `fn <name>\s*\(`, not `fn <name>` —
and when a spot-check contradicts a measurement, **suspect the spot-check first.** It is the
instrument you spent less on.

**15. A citation's FORM settles deliberateness; only a MUTATION settles aboutness — and the
mutation must target the SUBJECT of the obligation, not the text that states it.** A clause ID
in a test earns coverage credit only in a **backing form**: the ID as the first argument of a
backing assertion (`assert_golden("KISS-X", …)`, `assert_token(…)`), or after a `Backs:` /
`Enforces` keyword in a comment. Every other occurrence — a fixture literal, a `panic!`/`assert!`
message, a bare comment id, a lookup key — is a **mention** and earns nothing (`kiss_trace`'s
scanner enforces this; `// Backs: KISS-X` is the one-line migration for a genuine backing written
in a bare form). But form is only *deliberateness*; whether the test **asserts the clause's
obligation** is settled by mutation — seed a violation and confirm the named test reddens — and
you must mutate the **subject** of the obligation: the **implementation** for an implementation
clause, the **spec text** only for a document-consistency clause. **A test that reddens only under
a spec-text edit backs a document obligation, not the implementation clause.** *Why:* the #187
backing-vs-mention scanner dropped 13 reverse-cited clauses; by their comments, ten looked like
backings and three like mentions, and mutation confirmed exactly that — but the raw over-flag was
**4.3× (13 vs 3), all toward apparent rigour.** De-crediting more *looks* like more honesty, so an
over-flagging scanner reads as virtue while it destroys real backings, and a scanner false-negative
fed to the `decrediting_recorded` ratchet verdict (#261) would launder ten live backings into
recorded gaps — floor bumped, ledger marked, `RESULT: CLEAN`. So: **never de-credit on a
recognizer's silence.** Migrate a genuine backing; de-credit only a mutation-confirmed-dead one,
and record the mutation's *subject* in the ledger note. See `KISS-EMIT-6.4-0001/-0002/-0005` (the
three whose only reddening mutation is a spec-text edit).

**16. Evidence has a LOCATION, and five ways of losing track of it.** A finding is only as
durable as someone's ability to go back to what it rested on. These are one convention because
they are one failure — the evidence moved, or was never named, or was argued instead of shown.

**(a) When a finding rests on what another party said or did, name BOTH the PARTY and the
ARTIFACT the evidence lives in** — a commit SHA, a comment id, a re-runnable command.
**The artifact makes the attribution checkable; the name makes it routable**, and they fail
differently: a name with no artifact is an assertion about who did something, corroborated by
nothing, while an artifact with no name tells you what happened but not whom to tell. **In this
repo a name alone is especially weak** — the last 60 commits on `origin/main` are 60 by one
account, and PR comments land under that same account, so **the record cannot corroborate a
name.** Attribution is part of the measurement, not decoration on it. *Why:* #238 recorded that *"two
independent projects"* cited the wrong clause and that *"both were verified against the clause
text by the architect before this filing"* — **verified but never attributed.** When the
correction finally needed routing, the attribution was recoverable from nothing: not the issue,
not its comments (one shared account), not any sibling working copy (the citation lived in
correspondence). The finding itself was undamaged, because it never rested on identity — but
**the ability to act on it was gone**, and the only honest answer was that a plausible pair
would be an inference wearing a measurement's clothes. Same failure shape as a summary that
caches a floor number: **a record that drifts from its source does not fail, it ages.**

**(b) When a change argues that some usual evidence DOES NOT APPLY to it, that argument is the
load-bearing claim and is the first thing to audit.** A volunteered limitation is a claim about
where the evidence lives. *Why:* #279 stated plainly that GitHub runs PR checks on a merge
commit where the base is always an ancestor, **so its new stale-base detector could never fire
in CI, and the controls were therefore the evidence.** Read as candour that is commendable and
was meant as such; read as a premise it says *the controls are the ENTIRE evidence*, which
leaves exactly one question — do they cover everything? They covered the detector and not its
wiring: seeding `inconclusive = True` → `pass` left **every control green** while the feature
silently stopped firing and the tool returned to `RESULT: CLEAN` on a stale tree. The gap was
found by taking the warning as a premise rather than as a courtesy.
**Author side, and it is load-bearing: stating what your evidence cannot show is a TECHNIQUE,
not a confession.** A volunteered limitation is the reviewer's hardest work done for them, and
it is what made the #279 gap findable at all. **A convention that rewards candour by auditing it
teaches authors to stop volunteering** — and that failure is invisible, because an absence of
warnings looks exactly like a run of PRs with no limitations worth stating. Read (b) as raising
what a PR is worth, never as lowering what its author is trusted with.

**(c) Do not delete a check because a stronger one covers it until you have DEMONSTRATED the
stronger one fires on the case the old one was catching — and say which you are doing.**
*Removal by subsumption* and *relaxation* can produce the identical diff and license opposite
futures: the first is a precedent for proving subsumption, the second for relaxing the next
inconvenient check. **Prove it with a seeded case, not with an argument** — and per convention
9, the seed must assert that it applied. *Why:* #247's `test_`-prefix requirement is genuinely
subsumed (forward-existence catches a bogus name *including* one that starts with `test_`,
which the prefix check never could) — but that is a claim about a tool's behaviour, and the two
seeds that settle it (a matrix entry naming a nonexistent test; one naming a non-test symbol
that exists elsewhere) cost minutes. **If the second seed does not redden, the subsumption is
incomplete and the old check was doing something after all.**

**(d) A BLOCKED record hides the blockers it does not list — and a PARTIAL blocker list is
more misleading than none, because it looks like someone checked.** Record a known future
obstacle **while the item is blocked**, not at the moment someone tries to unblock it. *Why:*
`KISS-CONFORM-6.13-0002` carried three blocking reasons; one had been resolved and a fourth was
never listed. **Two of three true is exactly what made it read as accurate.** Whoever cleared
the listed reasons would have believed they were finished and then hit an unrecorded obstacle —
**a blocked item is precisely where a known future blocker gets lost, because nobody re-reads
the note until they think they are done.** The re-read never happens while it is blocked, and by
then the evidence for the unlisted obstacle may have expired.

**(e) A LOCALLY-SCOPED IDENTIFIER does not locate anything once it crosses a document boundary
— carry the scope with it.** The commonest case is a bare `§X.Y-NNNN`: clause numbering is per
sub-standard, so the short form is ambiguous by construction and **the ambiguity is invisible,
because both readings resolve to a real clause.** *Why:* three
live collisions, and **two of them produced a wrong citation by a careful reader.** `§6.8-0013` is
a namespace-vocabulary clause in KISS-Classify and an exhibition clause in KISS-Conform — two
projects cited the wrong one, which is what made #238 necessary. `§6.6-0002` is an op-identity
clause in KISS-Classify and a tier-2 numeric round-trip clause in KISS-Consume — **a finding about
the first was relayed against the second, and the correction of that relay drew a wrong conclusion
about a correct finding.** Two errors, one collision, both by careful readers, inside twenty
minutes. Unlike (a)–(d) this one is **mechanically checkable**: an unprefixed `§\d+\.\d+-\d+` in
cross-document prose is a grep, so it can have a detector rather than only a rule.

**The same mechanism reaches any identifier scoped to one document, and clause IDs are only its
commonest form.** A decision document's **option letters** are per-document, so *"A"* becomes
ambiguous the moment a second document renumbers them — **and both readings resolve to a real
option.** *Why:* a four-option ruling was carried to the maintainer as a paraphrase that
**re-lettered the options**, and the maintainer answered *"A"*. **In the original, `A` was the
no-op — zero cost, zero effect. In the paraphrase, `A` was the most expensive option available: a
normative tightening that breaks one sub-standard against another and rebuilds 34 call sites in a
consumer.** The two `A`s sat at opposite ends of the cost range and **one letter answered both.**

**So a paraphrase MAY compress the content and MUST preserve the identifiers.** The identifier is
what a decision attaches to: **compression is lossy and recoverable; relabelling is a collision and
is not.** Where renumbering is unavoidable, say so **at the point of use** — *"my letters differ
from the source's"* travels with the question, and a note in the paraphrase does not.

**It was caught only because the receiver asked which document the decision had been made against
before acting on it**, and that question is worth asking because of the direction the error runs:
**a no-op ruling is cheap to get wrong in the way that looks like agreement.** A wrong tightening
produces broken builds and gets found; **a wrong no-op produces nothing at all, which is exactly
what a correct no-op produces.**

> **On convention 9, from the same week it was needed twice.** A mutation seeded during the
> #280 review failed to apply and produced three green results that read exactly like evidence;
> only the seeder's own `substring not found` gave it away. **A convention that exists is not a
> convention that ran.** Print the seed's confirmation, and treat any green obtained from an
> unconfirmed seed as no result at all.

**17. A comment defending a non-obvious choice must carry the MEASUREMENT, not the reason.**
Conventions 1–16 all catch a **claim that outran its evidence**. This one fails in the opposite
direction: **a defence that never captured its evidence**, and so is asserted too weakly to
survive contact with a competent editor. *"Keep these distinct"* is a **preference** and loses an
argument with someone tidying. *"Tidy this and the sabotage that reddens two tests reddens none"*
is a **fact**, and it tells the reader how to re-derive it before touching the line.
**The failure mode of the weak version is that it reads as superstition — and superstition is the
first thing a competent person removes.** A defence that survives only while the next reader
believes an unsupported assertion has an expiry date set by someone else's confidence.

*Why, and the instance is cited as a RECIPE rather than a number on purpose — see below.*
**MLMF** (`https://github.com/ciresnave/mlmf`, branch `design/backend-agnostic-mlmf`, ref `8c1fbb5`)
carries an array fixture at `crates/mlmf-gguf/src/metadata.rs:728` and `:776` whose values are
deliberately **distinct** rather than zeros. Reproduce, in a worktree at that ref: replace
`(i * 11)` with `0u32` at both sites; set the `array_get("nums", 3)` expectation to `U32(0)`;
apply the off-by-one `index.checked_mul(w)` → `index.saturating_sub(1).checked_mul(w)`; run
`cargo test -p mlmf-gguf --lib`. **The off-by-one alone reddens two tests. With the fixture
tidied to zeros it reddens NONE** — every assertion stays true because two elements become the
same bytes. **The distinctness was load-bearing and nothing said so.**

> **The citation is a recipe because the convention applied to itself demands one.** MLMF's own
> reason for refusing the short form: *"I would rather they wrote that than '2 red vs 47 green,
> measured by mlmf', because **the second is a number someone has to believe and the first is a
> thing someone can run**."* Note also that **the all-zeros variant does not exist in the
> repository** — it was built in a throwaway worktree, measured, and removed. A citation naming a
> stored artifact would have been wrong.
>
> **And a hazard inside the strong form, which is theirs:** a measurement can carry a number that
> is **incidental to the point**. The passing total in that run will grow with every test they
> add; **the load-bearing figure is the contrast — two reds became zero — not the total.** An
> entry hanging on the incidental number goes stale and then looks wrong for a reason that does
> not matter. **Cite the contrast, not the magnitude.**

**A second instance, and a different failure of the weak form. Baracuda** (`baracuda`, PR #27,
commit `e4188096`, `scripts/check-test-crate-locality.sh`, fn `names_crate`) runs `sed` to
completion into a variable rather than piping it to `grep -q`. **It looks like a pipe somebody
forgot to simplify.** Simplifying it reintroduces a SIGPIPE-under-`pipefail` bug: `grep -q` exits
at first match and closes the pipe, `sed` — still writing a file larger than the pipe buffer —
takes SIGPIPE, `pipefail` promotes 141 to the pipeline status although grep *succeeded*, and `if !`
inverts a pass into a violation. **Measured on the guard's first live CI run: 8 files reddened on
`ubuntu-latest`, 0 on `windows-latest`, 0 on the local Git-Bash box** — because MSYS masks the
signal Linux delivers.

> **Their failure mode is the opposite of MLMF's and that is why both are cited.** MLMF's weak
> comment would be removed as **superstition**. Baracuda's would be removed as **"it does not even
> reproduce"** — a maintainer on Windows sees zero and concludes the defence is imaginary. **The
> strong form tells that reader why their platform is the wrong instrument before they draw the
> conclusion.** A measurement that is a *split* carries more than a contrast: it predicts what a
> skeptic will see, including when they will see nothing.

> **THIS ENTRY FAILED ITS OWN RULE THREE TIMES WHILE BEING DRAFTED, each caught by a different
> party, and that is recorded here rather than tidied away.** Written as *"2 red vs 47 green,
> measured by mlmf"*, the founding citation would have been a number to believe rather than a
> recipe to run — **caught by MLMF.** Citing Baracuda before their consent landed would have
> carried a **weak-form** comment as the example of the **strong** form — **caught by waiting.**
> And the worked example above originally said *"this file's own guard"* when the guard is in a
> Rust test file, **so the thirty-second verification pointed at something the reader could not
> locate** — **caught by review.**
>
> **A convention about verifiability that failed verifiability three times in its own drafting is
> not embarrassing; it is the strongest available evidence that the failure is DEFAULT rather than
> careless.** Nobody was being sloppy. **The weak form is what writing produces unless something
> external checks it**, which is precisely why the rule is worth having and precisely why it will
> keep needing enforcement rather than agreement.

**THE COUNTERWEIGHT, AND IT IS THIS CONVENTION'S OWN FAILURE MODE ARRIVING AS A FALSE POSITIVE.**
A rule that says *defend your choices with measurements* invites defences everywhere. **A defence
added by ANALOGY rather than by MECHANISM is cargo cult, and it is indistinguishable from a real
one to the next reader** — which is strictly worse than the weak comment this convention replaces,
because it will be believed. *Why:* an empty-set assertion was proposed for a gate whose set is a
**cargo-validated literal list** — an unknown or removed package makes cargo hard-error, so that
step **cannot** silently check an empty set. **The empty-set defence belongs to a gate that
DISCOVERS its set**, and that gate already carried it. **Ask which structure needs which defence;
do not apply the shape everywhere it rhymes.**

**The strongest instance is this repository's own, it predates the convention, and it defends an
ABSENCE rather than a choice** — which is the harder case, because there is no code to point at and
the next reader's default assumption is that somebody forgot. In
`conformance/tests/structure_key_vectors.rs`, where the obvious assertion would be to compare the
emitted artifact's `source_commit` against `SOURCE_COMMIT`:

> *"**NOT asserted here:** `source_commit`. `doc` is the emitter's own output and the emitter writes
> `SOURCE_COMMIT` into that field, so comparing the two compares a value to itself and cannot fail —
> **verified by mutating the constant to a bogus value, after which this test still passed.** The
> committed-vs-fresh byte equality in `test_structure_key_vectors_artifact_is_fresh` catches that
> same mutation (it fails STALE), so **the obligation is covered non-vacuously there, not here.**"*

**It does all three things the strong form requires, and one more.** It states the **measurement**
(the constant was mutated and the test still passed), it names **the mutation that established
it**, and it says **where the obligation IS covered instead** — so a reader who thinks the check is
missing is told both why it is absent and what to inspect in its place. **Without that last part, a
defended absence still reads as a gap.**

**A convention whose best instance was already in the file before anyone wrote the convention is a
much stronger claim that the rule describes something real** than any number of instances written
to comply with it. Nobody was following a rule here; **this is what the strong form looks like when
it is produced by the work rather than by the process.**

**This raises the bar here rather than describing what we already do.** A single grep over
`conformance/`, `tools/` and `.github/` finds this category **everywhere** — `"DO NOT SIMPLIFY THE
BRANCH BELOW"`, `` "`|| rc=$?` is LOAD-BEARING, not defensive" ``, `"deliberately narrow in two
ways, both load-bearing"`, `"Scan by MACRO INVOCATION, not by line"` — and **nearly every one is
the weak form.** They carry a mechanism, which is a reason, not a measurement.

**The worked example is this repository's own, and it is named precisely so a skeptic can reach
it:** `conformance/tests/structure_key_vectors.rs`, the guard backing
`KISS-CONFORM-6.3-0011`. Its comment says a line-based scan
*"is disarmed by a newline — and rustfmt wraps a long `assert_eq!` across lines as a matter of
course."* **True, and unfalsifiable as written.** The strong form seeds the multi-line form and
records that **the guard goes green while the defect stands** — which a skeptic checks in thirty
seconds instead of taking on trust.

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
