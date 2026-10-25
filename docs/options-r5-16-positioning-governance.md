# Options note: R5.16, positioning and governance the docs promise

**Informative. No normative text. For CireSnave's decision (board 279, pair 4 of 5).**
Evidence read at `origin/main`: `README.md` (Conformance section), `DESIGN.md`, `PRIOR-ART.md` §3 and §4,
`docs/kiss-convergence-reconciliation.md` (D4, #9), and the ROADMAP R5.16 row. These are your
positioning calls; the note lays out choices and does not make them.

## The question

Four things the docs say or omit need an owner's ruling.

**1. A free results registry and a certification mark.** README says the steward "maintains a free
registry of results" and that "an eventual certification mark" will make claims checkable. DESIGN calls
the mark optional and not a v1 requirement. Is the mark a promise or a hope? (The registry's service
level is the R5.8 note.)

**2. The README does not state the cache-key vs join-key distinction.** `PRIOR-ART.md` §3 makes it the
central argument ("a cache key is not a join key"). The README leads with "the build fails on any untested
MUST" in its sub-standard table, and the README itself says the figures drift. That sentence describes the ratchet's design; it does not say that 391 clauses are still genuinely
untested (`kiss_trace.py`, `origin/main` at 0.4.76).

**3. Preempted material and open dtype sets.** `PRIOR-ART.md` §4 says the Classify dtype table is
preempted by DLPack and that a closed twenty-token set forecloses the "frozen container, extensible
leaves" mechanism. R1.26 (waits on a schema event) covers sub-byte floats only. Should the suite demote
what DLPack already does and say so?

**4. `u16` and `u64`.** The convergence reconciliation records "prune `u16`/`u64`" as an editors' call
(#9: "open"); D4 says it is not re-verified. Do they stay among the 24 tokens?

## Options

**A. Rule all four; edit the docs to match.** One docs PR per file group. README gets a lead paragraph on cache key vs join key and a corrected conformance sentence; the dtype and
`u16`/`u64` rulings are recorded where D4 lives.

**B. Rule 2 and 4 now (they are factual, cheap) and defer 1 and 3.** Positioning that depends on an
outside reader (the mark, DLPack bridge) waits for the first foreign reader (R4.1).

**C. Change nothing.** The README keeps promising more than it states.

## Recommendation

**B.** The README's lead sentence and the cache-key point are corrections of existing statements, and
the `u16`/`u64` ruling closes a recorded open item. The mark and the DLPack stance are strategy, and
committing to them before a second reader exists spends optionality for nothing. I would word the mark as
"optional, not committed" wherever it appears, which matches DESIGN.

## What freezes, and what to decide

Item 3 touches the closed set of twenty-four tokens (Classify §6.1-0001), so a change there is a
`structure_key` schema event, not a docs edit; Classify itself is still unfrozen. Note that `PRIOR-ART.md`
§4 says "twenty-token set", which no longer matches §6.1-0001's twenty-four. Decisions needed: **A, B or C**; the `u16`/`u64` ruling; and whether the README may
lead with the cache-key distinction.
