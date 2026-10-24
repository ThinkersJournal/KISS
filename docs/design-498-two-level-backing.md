# Design note — #498: a clause covered at two levels (R3.8)

**Status: design note for the PM's gate; no code in this PR.** Measured at KISS `origin/main` `3e994b0`,
2026-10-24 (`python tools/kiss_trace.py --report`). Docs only; the version number is the PM's.

## 1. What the tool does today (read, not recalled)

The issue says the model is "1:1 by construction". Reading `kiss_trace.py` at `3e994b0`, that is only half true:

- **Citations are already many-to-one.** `cited` maps clause → *set* of tests; `backed` accepts any test in the set, not only the §9-named one.
- **The §9 pointer is one-to-one**: `clause_test` holds a single name per clause.
- **The tiers are defined from the pair.** `tier_cited_by_name` = clause whose §9-named test is among its citers (207 at `3e994b0`); `tier_cited_other` = cited, but the §9-named test is *not* a citer (20). `NAMED` = backed with no backing-form citation (232). `NAMED + CITED = 459`.

So the failure #497 hit is narrower than "the model cannot hold two tests": a second test that *cites* is held fine. What goes wrong is **how the report reads that state**.

## 2. The three real gaps

1. **Label.** `tier_cited_other` is worded "a DIFFERENT test carries it — backed, but the §9 row points elsewhere". It is one bucket for two different facts: (a) the §9 row is stale or wrong, (b) the §9 row is right and a *second* test adds a level. Only (a) is a defect; (b) reads as one.
2. **Tier jump.** In #497 the stated-level test is the §9 name and does not cite; the new byte test cites. The clause moves NAMED → CITED(other) and `named` drops 237 → 234. The move itself is *correct* by the tool's own rule (the strongest evidence is a deliberate citation), but nothing says the §9 test is still doing its separate job.
3. **Invisible subject.** A byte test that does not cite (#497's option b) is protected against deletion by the bound `uncited_tests` figure and against nothing else; no record says what it is *for*. Related to #458 (the *kind* of coverage a test provides), which this is the cardinality side of.

## 3. Options

| | Change | Fixes | Cost / risk |
|---|---|---|---|
| **A** | Split the `tier_cited_other` report line in two by a cheap test: does the §9-named test *exist and carry a mention* (`cited_raw`) of the clause? Yes → "§9 test present, second test adds a level"; no → "§9 pointer stale". Report only, no new syntax, no gate. | gap 1 | smallest; does not touch gap 2 or 3 |
| **B** | A level qualifier on the existing backing form: `// Backs[enforced]: ID` / `assert_golden[...]`; unqualified = `stated`. `cited` becomes clause → {level → tests}. Report prints per-level counts; the tier rules stay. | gaps 1, 2, 3 | touches the citation grammar and `_backing_clauses`, so every reader of `info["clauses"]` (`kiss_cites`, `kiss_uncited`, `kiss_backs_matrix`, `kiss_bundle`) must be re-checked; a new vocabulary |
| **C** | A `supplementary` tier: a second citer is marked `// Also-backs: ID` and counted separately from `tier_cited_other`. | gaps 1, 3 | two citation forms to keep honest; "supplementary" is a claim no mutation checks |

## 4. Recommendation

**A first, B only if a second instance appears.** Evidence is one instance (#497, three tests) plus 20 `tier_cited_other` clauses of unknown composition. Step zero for A is to *measure* those 20: how many have a present, mentioning §9 test (case b) versus a stale pointer (case a). If most are (b), the label is the whole defect and A closes it. If the 20 are mostly (a), the issue's framing was wrong and the 20 are a pointer-cleanup list instead. Either answer is cheap and decides whether B is warranted.

Not recommended now: B before that measurement, because it would grow the citation grammar to fix a report label.

## 5. What this note does not decide

- Whether per-level counts belong in the README bound figures (they would add a ratcheted-adjacent number; `named` is already bound at README L39).
- Whether a level should ever gate. Today nothing about tiers gates except through the floor; this note keeps it that way.

## 6. Ask

PM: approve A with the 20-clause measurement as its first step (a `[FINDING]` before any code), or rule B directly.
