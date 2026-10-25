# Options note: R5.8 (SLA item only), what does free steward certification promise?

**Informative. No normative text. For CireSnave's decision (board 279, pair 3 of 5).**
Evidence read at `origin/main`: `spec/conform.md` Appendix D item 5; `spec/umbrella.md` §8.2 to §8.4.
The other R5.8 items (Conform D.1-4, 6-7, Consume D.2/D.3/D.5, Emit A.5) are marked `[go]` and are not
in this note.

## The question

Umbrella §8.3 says ThinkersJournal runs the unmodified suite "as resources permit, free of charge" and,
on a pass, lists the implementation. Conform Appendix D item 5 asks for two things the text does not
give: **the SLA or queue policy**, and **whether a self-certified-but-not-yet-verified result carries an
interim registry status.**

## Options

**A. Keep "as resources permit"; promise nothing.** Matches §8.3 as written. A requester gets no
date, and cannot plan around the listing.

**B. Publish a queue rule with no time promise.** For example: requests are handled in arrival order,
the steward posts the queue position and the date it last ran a verification. A requester sees
progress without the steward owing a deadline.

**C. Publish a numeric target (a number of days).** Strongest for a requester. The steward owes it, and
there is currently no staffing for it that I can see; a missed target is a visible breach of something
the steward chose to say.

**Interim status (independent of A to C).**
**(i) None.** The registry stays "the authoritative record of verified implementations" (§8.3); a
self-certifier publishes results under §8.2 and is simply not listed yet.
**(ii) A distinct "self-certified, pending" row.** Visible, but it puts unverified claims on the list
whose whole value is verification, and §8.4 says value accrues to being listed.

## Recommendation

**B with (i).** A queue rule costs the steward almost nothing and answers the requester's real question
(where am I). No interim row: §8.2 already gives self-certification its own path, and a pending row
would blur the one list meant to be authoritative. Revisit C only if requests arrive faster than one
steward can clear them. I have not checked how many requests have been received; I believe none, but
that is unverified.

## What freezes, and what to decide

Umbrella is informative, so no clause freezes. Decisions needed: **A, B or C**, and **(i) or (ii)**. This
is a steward-policy item, which is why the ROADMAP marks it `[rule]` while the rest of R5.8 is `[go]`.
