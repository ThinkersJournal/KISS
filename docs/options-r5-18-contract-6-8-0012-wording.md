# Options note: R5.18, the stale "deferred" wording in KISS-CONTRACT-6.8-0012

**Informative. No normative text. For CireSnave's decision (board 279, pair 5 of 5).**
Evidence read at `origin/main`: `spec/contract.md` KISS-CONTRACT-6.8-0012; `spec/classify.md`
KISS-CLASSIFY-6.7-0012 and 6.7-0013; `spec/ops.md` near line 2262; the ROADMAP R5.18 row. The rest of
R5.18 is `[go]` (one docs PR per file group); this line is `[rule]` because it is normative text.

## The question

KISS-CONTRACT-6.8-0012 says the key carries the accumulator class "via the accumulator coordinate (the
contraction `<acc>` of KISS-CLASSIFY §6.7-0006, **or the deferred non-contraction coordinate of
§6.7-0012**)". The coordinate is no longer deferred: Classify 6.7-0013 ("realizing §6.7-0012") now
defines the non-contraction `<acc>/<mp>` field, and Ops line 2262 says the field "realizes"
6.7-0012. **May the Contract clause be reworded to say so, and to which citation?** Because the clause is
normative, the choice is yours and not an editor's.

## Options

**A. Reword to cite the realized field.** Replace "the deferred non-contraction coordinate of
§6.7-0012" with "the non-contraction `<acc>/<mp>` field of §6.7-0013 (realizing §6.7-0012)". No
obligation changes: the clause already says the key carries only the `(compute, accumulator)` class and
that `N` MUST NOT enter the key. It corrects which clause defines the coordinate.

**B. Reword to "§6.7-0012 or §6.7-0013" without saying "deferred".** Safer against a reader who knows
only 6.7-0012, at the cost of a vaguer citation.

**C. Leave it.** The clause stays accurate in obligation and wrong in one word, and the stale word keeps
suggesting a missing feature.

## Recommendation

**A.** It is a citation correction with no change to what an implementation must do. I have not checked
whether any test or the sidecar keys on this clause's exact text, and would run `kiss_trace.py` and the
Contract tests before it ships. If you would rather treat any normative edit as needing a version bump
of the clause set, say so and I will follow that process.

## What freezes, and what to decide

Contract is not frozen, so nothing is irreversible. Decisions needed: **A, B or C**, and whether the
remaining R5.18 doc fixes (the `[go]` file groups) may proceed as docs-only PRs without further sign-off,
which is what the roadmap marks.
