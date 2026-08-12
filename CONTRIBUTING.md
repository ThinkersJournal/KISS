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

## Normative writing conventions (summary of umbrella §3–§4)

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
