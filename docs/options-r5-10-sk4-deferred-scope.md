# Options note: R5.10, the scope decisions the sk4 RFC deferred

**Informative. No normative text. For CireSnave's decision (board 279, pair 4 of 5).**
Evidence read at `origin/main`: `rfcs/sk4-schema-event.md` §3.5, §8, §10 and the ROADMAP R5.10 row.

## The question

The sk4 RFC deferred five scope items to "their own RFC". None has one. They are independent; each can be
ruled on its own. **Which are in scope for KISS, and which are not yet?**

## The five items

**1. Backward-op category convention.** CireSnave (2026-08-06, as quoted in the RFC): training space is a
first-class use case. The RFC leans toward non-primitive decompositions named for dispatch, which keeps
the forward-only primitive floor. The ROADMAP row says `git grep -i backward spec/ docs/ ROADMAP.md`
finds nothing, so no draft exists.

**2. Per-operand dtype and indexed-region synthesis.** Would let gather, scatter and embedding style
ops (float data, integer index) be described by one cell. It **reverses** KISS-CLASSIFY-6.6-0015's
deliberate caller precondition. The RFC says it is a describability gap, because the data dtype and the
index width are already keyed, and that it is not urgent.

**3. Catalog-vs-compiler capability bit.** A provider says whether it serves a fixed catalog or compiles
on demand. The RFC says it stands on trust-model merits, not urgent.

**4. A MUST that requests carry honest operand types.** Motivated by a near miss: substituting `i32` for
`u32` index operands would have made the region uniform and falsely passed the synthesizer gate. The RFC
treats this as a request-validity MUST, independently valuable.

**5. A migration note for the `Other(n)` reclassification hazard.** New variants that used to fall into
`Other(n)` silently reclassify earlier values, with no build break. A note flagging it beside the
version-prefix rule was accepted at cosign but is not in clause text.

## Options

**A. Rule each now; author the cheap ones.** Items 4 and 5 are small and independent (a request-validity
clause, a migration note). Item 1 gets a drafting task. Items 2 and 3 are explicitly ruled "not yet" with
a trigger.

**B. Rule only item 1 (it carries your stated priority) and leave the rest deferred.**

**C. Leave all five deferred until a consumer needs one.**

## Recommendation

**A.** Items 4 and 5 protect against silent wrong answers and cost a clause and a note. Item 1 is the one
you have already said matters. Items 2 and 3 should be recorded as "not yet", each with the trigger that
would reopen it (item 2: a consumer that must synthesize an indexed kernel; item 3: a second provider
model). I have not checked whether item 4 collides with an existing clause in Classify or Synth, and
would do so before drafting.

## What freezes, and what to decide

Item 2 is the only one that reverses an existing clause (6.6-0015); Classify is unfrozen, so it is
reversible on paper but would be a schema event. Decisions needed: **A, B or C**, and for item 1 whether to draft now.
