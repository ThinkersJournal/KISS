# Options note: R3.8 (#498), how should the traceability report show a clause that is tested at two levels?

**Informative. No normative text. For CireSnave's decision (the choice between options A and B of
`docs/design-498-two-level-backing.md`, which row R3.8 says is the PM's call; the PM asked for this note so the call can go to the board).**
Measured at `origin/main` `b9e08e7f`, 2026-10-26, with `python tools/kiss_trace.py --report` plus a scratch
copy of the tool that printed the composition (the scratch copy is not committed).

## The question

Some clauses are checked twice, in different ways. One test reads the spec text and checks the clause lists
what it should (the "stated" check). Another builds the real bytes and parses them with the shipped reader
(the "enforced" check). Both are real and they catch different mistakes. The traceability tool knows one test
per clause as the pointer in the spec's test table, so when the second test cites the clause the tool files
the clause under "a different test carries it, the table points elsewhere". That wording reads like a
mistake even when the table is right. **Do we fix the wording, or add a notion of levels to the tool?**

## What is true today

There are 35 clauses in that "different test carries it" group. For all 35 the test the spec table names
exists, so none is a stale pointer. For 26 of the 35 the named test also mentions the clause, which is the
healthy case: a second test adds a level. For the other 9 (`KISS-CLASSIFY-6.8-0008`, `-0009`, `-0012`, `-0013`,
`KISS-CONFORM-6.5-0017`, `-6.8-0010`, `KISS-CONTRACT-6.11-0011`, `-6.6-0006`, `-6.9-0009`) the named test
exists but says nothing about the clause, which is a weaker tie and worth a look, though it is not the
"stale pointer" the label implies. When the issue was filed the group had 20 clauses; it has grown.

## Options

**A. Fix the label only.** Split the report line in two: "the table's test is present and mentions the clause,
and a second test adds a level" (26 today) and "the table's test is present but does not mention the clause"
(9 today). No new syntax and nothing gates. *Cost:* nearly nothing. *What it leaves:* a second test that does not
cite is still invisible, and the tool still cannot say which level a test is for.

**B. Add a level to the citation.** A test says which level it covers, for example `Backs[enforced]: ID`, and an
unmarked citation means "stated". The report then counts clauses per level. *Cost:* a new piece of citation
vocabulary; every tool that reads the citations (`kiss_cites`, `kiss_uncited`, `kiss_backs_matrix`,
`kiss_bundle`) has to be checked against it. *What it gives:* both levels visible and correctly attributed.

**C. Add a "supplementary" marker for a second citing test** (`Also-backs: ID`), counted apart from the "points
elsewhere" group. *Cost:* two citation forms to keep honest, and "supplementary" is a claim no check verifies.

## Recommendation

**A now, B only if a second instance of the problem appears.** The evidence is one case (three tests from #497)
and a group of 35 whose composition is now measured: nearly three quarters are the healthy case, so the label is
the main defect. B would grow the citation vocabulary to fix a label. If you would rather have the structure
now, B is the one to pick, not C.

## What freezes, and what to decide

Nothing freezes: this is tooling. Decisions needed: **A, B or C**; and whether the 9 "named test does not
mention the clause" clauses should be reviewed as a separate small task (I suggest yes, under A).
