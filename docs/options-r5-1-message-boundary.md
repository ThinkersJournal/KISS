# Options note: R5.1, where does one message end on a stream?

**Informative. No normative text. For CireSnave's decision (board 279, pair 1 of 5).**
Evidence read at `origin/main` 9336a5b: `WIRE-FIRST.md` §1.1; `git grep -E 'message extent|short
read|stream reader|stream binding' spec/` finds nothing (control: `git grep -c hard-reject
spec/announce.md` finds 5, so the grep works). WIRE-FIRST is a dated snapshot (2026-07-15); §1.1 is
not among the items it says were closed.

## The question

Two processes connect over TCP or a pipe. The provider writes a 56-byte envelope, the consumer's
`read()` returns 40. Is that a short read, or a truncated message to hard-reject under
KISS-ANNOUNCE-6.2-0001? Today's text does not say, because Announce §1 (informative) disclaims the
transport and §6 gives no outer frame and no rule for finding a boundary. Five clauses bound-check
against a range nothing defines: 6.2-0001 ("any input"), 6.2-0007 ("the input buffer"), 6.3-0009
("the remaining input"), 6.3-0010, 6.4-0006 ("the request buffer"). 6.2-0001 is meaningful only on a
datagram transport, so the one clause that defines envelope rejection cannot be enforced on a stream.
Separately, 6.4-0004 carries a u32 payload length with no cap.

## Options

**A. Add a stream binding to Announce (WIRE-FIRST's recommendation).** A new §6.6: dispatch on all
four leading tag bytes (SEAM/SAVL share `0x53`, CYRQ/CRSP/CDEC share `0x43`); each tagged message is
self-delimiting; an unknown tag is a typed decline with no resynchronisation (consistent with
6.2-0008); the CRSP payload is capped; one defined term, "message extent", substituted in the five
clauses above. **No byte of an existing message changes.**
*Touches:* KISS-ANNOUNCE 6.2-0001/-0007, 6.3-0009/-0010, 6.4-0004/-0006, plus about four new clauses
(each arrives `untested` until a test lands) and two vectors: SEAM+SAVL concatenated, and a SAVL
whose `record_count` overruns into a following CRSP (typed decline). No regeneration for any speaker.
*Needs a number:* the CRSP cap.

**B. Add an outer length-prefixed frame to every message.** Robust to a corrupted tag, but a wire
change: all five messages gain a header, so `list_version`, the Announce vectors and every speaker
(Fuel, Baracuda, Unpopped) regenerate. This is my sketch, not in WIRE-FIRST; it solves a problem
(resync after corruption) that 6.2-0008 deliberately declines to solve.

**C. Stay transport-agnostic.** State that the message extent is supplied by the binding, never
by the message; conform the existing clauses to that wording; define the stream binding in a separate
profile later, together with the in-process profile (R2.8, issue #18).
*Consequence:* two readers conforming to the text can still behave oppositely over TCP, so a foreign
reader (R4.1) over a socket would have nothing to conform to.

## Recommendation

**A**, written as a named binding profile so that #18's in-process profile is the other one (WIRE-FIRST
asks for them to be decided jointly; both want the same missing concept, a binding profile under the
umbrella's capability model). It is the only option that fixes the unenforceable clause without a
wire change. If CireSnave prefers C, the five clauses still need the "message extent" wording
either way.

## What freezes, and what to decide

Announce cannot freeze with a foreign reader (R4.1) over a real socket until this is settled, so it
precedes that freeze; nothing is frozen today. Decisions needed: **A, B or C**, and if A, who picks the
CRSP cap number (I suggest the PM with Fuel and Baracuda, who must allocate it).
