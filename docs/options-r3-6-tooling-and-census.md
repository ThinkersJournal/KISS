# Options note: R3.6, three tooling and process items (#458, #400, #484)

**Informative. No normative text. For CireSnave's decision (or the PM's, where it says so); the PM turns each question into a board item.**
Read at `origin/main` `2e65634`, 2026-10-26. Paths are `file:line` at that commit. Issues: KISS#458, #400, #484 (all filed
by the lane that runs the traceability tool). Roadmap row: R3.6. Nothing here freezes. These are tooling and process, so
none of the questions touches the wire.

---

## Q1. Where is a test recorded that checks the document and not an implementation? (#458)

**Example.** `test_consume_taxonomy_structure_based` checks the **text** of KISS-CONSUME-6.4-0007 (category assignment is
structure-based). No consumer is in the repo, so nothing observes whether an implementation obeys it. Filed as `harness`, it
reads as behavioural coverage it does not give, and `harness` is one of the three figures `COVERAGE_FLOOR.tsv` ratchets.
The `lint:` category is the right idea, but a row there must name a `tools/kiss_*.py` that declares its coverage and is
executed, and a Rust test cannot do that. KISS-CONSUME-6.4-0007 is the case R3.14 already carries.

**Options (the issue's three, none chosen).**
- **A. A `document:` category** in the ledger, parallel to `lint:`, naming the test and what it observes, without the
  discovery step. *Cost:* the cheapest; the category's truth is checked by nothing, which is the objection `lint:`'s
  discovery was built to answer.
- **B. Extend the `--emit-coverage` discovery to Rust tests,** so a test declares what it binds and is run to prove it.
  Keeps "the tool asserts and is executed". *Cost:* considerably more machinery.
- **C. Leave it and rely on the per-clause mutation audit** to catch a subject mismatch as it is found. *Cost:* nothing,
  and it scales with attention, not with the corpus.

**Recommendation: C now, and mark the one known instance in the ledger note.** One instance is measured; the issue says it
did not extrapolate and neither do I. If the count of document-binding tests filed as `harness` turns out to be more than a
handful, do B, not A: A adds a label nothing verifies, which is the failure the ledger exists to prevent.

## Q2. Should a clause be able to name more than one test? (#400)

**Example.** A clause with two independent obligations (a float sign flip and an integer two's-complement rule) can name
one test. Writing two names after one `*Test:*` tag used to bind only the first, silently. **That silent half is fixed:** a
`*Test:*` line with more than one name is now a violation (0.4.54, as the row records). What is left is the model itself:
one clause, one named test.

**Options.** **A.** Accept several `*Test:*` tags per clause, changing what a row of the clause-to-test table means. **B.** Accept a
list after one tag (`` `a`, `b` ``), which needs the same change in the matrix row. **C. Keep one named test** and let a
second conjunct be covered by citation (`Backs:`), which is what the traceability tool does today.

**Recommendation: C, and take it together with R3.8 (#498).** That note's recommendation is to fix the report's label for a
clause tested at two levels rather than add syntax, and measures 35 clauses where a second test already adds a level. The two
issues are the same assumption seen from two sides, so choosing differently for them would be inconsistent. If you
choose the level-syntax option (B) for #498, take B here.

## Q3. How is the set of affected parties constructed? (#484)

**Example, from the issue.** A normative change lands only when every affected implementor agrees, so the set of
implementors asked is load-bearing. On one change, four parties were mis-identified because the question searched a name
and inferred a role: one was routed as the implementer of a different sub-standard whose section numbers collide with
this one, one as an emitter when it only calls one, and so on.

**What is already written.** `CONTRIBUTING.md:95-104` says affectedness is usually **testable rather than judged** and
shows two per-clause questions, and `:111` says to ask directly when no input distinguishes a compliant implementation. It does
not say the census must be keyed on **what a party's code does** (emits the artifact, parses it, frames an opaque body
around it, or only calls a library that does one of those), that only the first two make a party affected by a content
clause, or how to avoid a **homonym** (a clause's word that is also an unrelated word in the target's code, so a bare grep
gives a false yes). This is process, not code: there is nothing to build.

**Options.** **A.** Add a short paragraph to `CONTRIBUTING.md` stating the four categories, that framing and calling are
recorded as downstream evidence and never as sign-offs, and that a search is anchored on a token the clause owns with a
negative control on the homonym. **B.** Leave it in practice and in this issue. **C.** Put it in the portfolio instructions
(not recommended: that file loads in every session and its owner asks for additions only if every lane needs them).

**Recommendation: A.** It is the one place a future affected-set is built from, it is project-specific, and it costs a
paragraph. The PM can approve it without you.

---

## Summary

| Q | Issue | Recommend | Bytes change? | Note |
|---|---|---|---|---|
| 1 | #458 category for a document-binding test | C (mutation audit); B if the count grows | no | one instance measured |
| 2 | #400 one clause, several tests | C (keep one), with R3.8 | no | silent half already fixed |
| 3 | #484 affected-set census | A (a paragraph in `CONTRIBUTING.md`) | no | process; PM can approve |
