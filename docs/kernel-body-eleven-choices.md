# The eleven choices for the kernel-body standard (decision draft for the board)

**Informative. No normative text. Nothing here is built.** Written 2026-11-17 for the PM, who files one board item per
choice that needs CireSnave and rules the rest from the spec's own clauses. Board 450 (RULED 2026-11-16, option A, "but
only as long as the necessary references are kept in place so readers know when recommendations are available") created
the new standard, working name **KISS-Kernel**, and the scope note's process step 2 says each of these eleven choices is
ruled before any clause text about it is written. Source: the Unpopped lane's concept map
(`docs/kiss-kernel-body-concept-map.md` in the Unpopped repository, read at its `origin/main`, section 12) and the scope
note `docs/kernel-body-scope-note.md`. Spec quotations below were read at KISS `origin/main` `c955ec1`.

## What this is about, in plain words

A GPU program runs one small piece of code, called a **kernel body**, on thousands of threads at the same moment. KISS-Ops
already says *what* a whole operation computes (add these two arrays, sum this axis). It says nothing about what **one
thread** may assume about the others. The eleven choices below are the places where Unpopped, which has built and tested a
model of this, picked one answer where the standard could pick another. A few words used throughout:

- A **barrier** is a point where every thread of a group must stop and wait until all of them have arrived.
- A **memory order** says when one thread's writes become visible to another thread. **Acquire/release** is the common
  pairing: a write marked "release" followed by a read marked "acquire" of the same location guarantees the reader sees
  everything the writer did before the release.
- A **data race** is two threads touching the same memory location, at least one writing, with nothing forcing an order.
- An **atomic** operation is a read-modify-write that no other thread can interleave with.
- A **target** is a specific kind of hardware or language the program is compiled for.

**Nothing Unpopped has done has been run on a device.** Its evidence is an interpreter, an independent reference per
feature, a compiled-C comparison and deliberately broken copies that must fail (map section 10). That shows its rules are
consistent with themselves, not that any GPU behaves that way. Every recommendation below inherits that limit.

**Bytes.** KISS-Kernel is a new document. For every choice below, *no byte of any existing standard moves*: the sk8 keys,
the Contract golden document, the operation facts and every existing corpus file are untouched (scope note section 2,
"All additions are additive under the capability model"). What each choice moves is the text and the test programs of the
**new** standard, and I say below whether it would also require a change to an existing clause. The only existing bytes the
whole effort touches are the three references board 450 asked for (`docs/kernel-standard-discoverability.md`).

## Choices that existing clauses already decide, or nearly do

These are the ones I suggest the PM may rule by quotation; the PM decides whether the quotation is enough.

**9. Float atomic add is a band, not exact bytes.** *Already ruled.* CireSnave, board 417 "A" (2026-11-12): adopt in the
first version, the bound defined as KISS-CONTRACT-6.8-0012 defines it. It landed as KISS-OPS-6.11-0024, whose Draft-change
note quotes the board. Nothing left to rule; KISS-Kernel only cites 6.11-0024. Bytes: none.

**11. Named barriers, async copies, tensor memory and matrix-multiply atoms are left out of the first version.** *Board
450's own option A text says so:* "Left out of version 1: named barriers, the asynchronous barrier and copy features,
tensor memory, matrix-multiply units, and total ordering of every operation" (the board item,
`CIRESNAVE-DECIDED-ARCHIVE.md` section 450, Options paragraph), and the scope note section 2 lists the same exclusions.
Unpopped agrees (map section 8: designed, not built). Leaving them out costs nothing now and adding them later is
additive. Bytes: none. Recommendation: rule by quotation.

**3. Integer division and shifts never trap and always give an answer.** *Partly decided by existing clauses, and the
existing clauses point the other way.* Unpopped makes integer division by zero return the dividend, `MIN / -1` return `MIN`,
and shift counts wrap around the width (its PM ruling of 2026-10-31). But KISS-OPS-6.4-0002 says integer division is
**not an operation of KISS-Ops** and "division by zero MUST be treated as target-defined (target-UB), not pinned by
KISS-Ops", and KISS-OPS-6.10-0004 says a shift amount outside `[0, bitwidth)` "MUST inherit the target's behavior (KISS-Ops
does not pin it); the caller is responsible for clamping". So the standard already chose *target-defined* for the operations
it has. The open question is only whether the kernel body, which has its own expression language, may pin something the ops
layer deliberately did not.
Options. **A.** KISS-Kernel says nothing: integer division and shifts in a kernel body are target-defined, exactly as in
KISS-Ops, and a program that needs a defined answer clamps first. Consequence: no conflict, but two readers can disagree on
a program that divides by zero, and Unpopped's own total rule becomes a Unpopped-only behavior. **B.** KISS-Kernel pins
Unpopped's total rule for the kernel body only. Consequence: a program is now portable, but every target must emit extra
checks around each division, which costs speed, and it contradicts the spirit of 6.4-0002 (the two documents then give
different answers for "what does a divide do"). **C.** KISS-Kernel pins the total rule as a *reference* behavior that a
target may replace by stating its own. Consequence: a reader knows the default and a target can still choose speed.
*Recommendation: A for the first version,* because the existing clauses already decided it and a change belongs in a
separate, explicit ruling about KISS-Ops, not slipped in through a new document. *Bytes: no* under A; B and C add text to the
new standard only but would need a note in 6.4-0002 and 6.10-0004 to avoid a visible contradiction.

## Choices where the standard picks the rule every program can rely on (I think CireSnave should rule these)

Each of these fixes what a correct program may assume, which cannot be taken back once two implementations rely on it.

**1. The strongest memory ordering ("sequentially consistent", `SeqCst`) is accepted but treated as acquire/release.**
What Unpopped does today: a program may ask for the strongest ordering, which on C++ means "all threads agree on one global
order of these operations", but Unpopped models only the weaker acquire/release pairing. Why it is a choice: a program that
truly needs the global order (the classic example is two threads each setting a flag and then reading the other's) would
behave differently under the stronger rule. Affected: anyone writing a lock-free algorithm, and every future implementer
who must decide what the word means. *Board 450's option A text already leaves "total ordering of every operation" out of version 1 (see choice 11), so option C
below is already excluded; what remains is whether the word is accepted or refused.*
Options. **A.** The first version accepts the word and states plainly that it means
acquire/release, with the global order left to a later version. Consequence: honest and small; a program needing the global
order is not portable, and the standard says so. **B.** Refuse the word in the first version. Consequence: nothing can be
misread, but programs that use it cannot be expressed. **C.** Define the global order now (excluded by board 450's text). Consequence: correct for all
programs, but a large amount of new rules and tests that Unpopped has no model for, and no implementer has said they can
check it. *Recommendation: A;* the PM may rule it by quoting board 450 for the exclusion and CireSnave need only confirm
accept-versus-refuse. Bytes: no.

**2. A thread asking a neighbour for a value that is not there keeps its own value.** In a "shuffle", each thread in a group
of, say, 32 reads a value held by another thread (the one `n` places up, say). For the threads at the edge there is no such
neighbour. Unpopped returns the thread's own value, which is what NVIDIA hardware does; other targets leave it undefined or
wrap around. Options. **A.** Adopt "keep your own value" as the rule, and let a target differ only by stating that it does.
Consequence: portable programs, a little extra work for targets that do otherwise. **B.** Say the result is undefined and
require programs not to depend on it. Consequence: no target pays anything, but no program can use the edge case, and
"undefined" is exactly what choice 4 below argues against. **C.** Leave it to each target to state. Consequence: no common
rule, so no test program can have a single expected answer. *Recommendation: A.* Bytes: no.

**4. A data race is classified and named, not "anything can happen".** In C and C++ a data race makes the whole program
undefined. Unpopped instead classifies a racy program as racy and stops with a named error, never simulating weak hardware.
Options. **A.** KISS-Kernel adopts the classification: a conforming reader either refuses the program or marks it racy,
and never gives a quiet answer. Consequence: every racy program has a checkable verdict and test programs can say "this one
is racy"; the price is that every reader must implement the race check. **B.** Undefined behavior as in C++.
Consequence: simple to write, nothing to check, but a bug can hide and two readers cannot be compared on a racy program.
**C.** Race freedom is a promise the caller makes, readers need not check. Consequence: cheapest, and gives no conformance
evidence at all for the most error-prone feature. *Recommendation: A,* because the standard's whole conformance method is a
program plus an expected verdict. Bytes: no.

**5. Reading memory nobody wrote is an error.** Shared (per-block) and private (per-thread) memory is not cleared when a
kernel starts; real devices hand back leftover garbage. Unpopped treats a read of a never-written element as an error, so a
missing barrier cannot hide behind a lucky zero. Options. **A.** Adopt in the reference behavior, and let a target say it
returns something else. Consequence: bugs are found early, targets are not forced to clear memory. **B.** Require
zero-filling. Consequence: portable, but a real speed cost on every launch and it hides missing-barrier bugs.
**C.** Undefined. Consequence: no checkable verdict for the most common synchronization mistake. *Recommendation: A.* Bytes:
no.

**6. The "did every thread reach this barrier?" check is deliberately cautious.** A barrier is only meaningful if every
thread of its group arrives. Unpopped decides this with a conservative analysis: if any condition leading to the barrier
*could* differ between threads, it rejects the program, even when it would actually have been fine. It never accepts a
program that could hang. Options. **A.** Adopt the cautious analysis as a *sufficient* rule: a program this rule accepts is
guaranteed safe, and a reader may be less cautious than the rule (accept more programs) only if it still never accepts a
program that really diverges. Consequence: portable verdicts for the programs the rule rejects, freedom for smarter checkers.
**B.** Say a non-uniform barrier is undefined and leave detection to tools. Consequence: simplest text, no test program can
say "this is rejected". **C.** Require an exact decision. Consequence: impossible in general (it is as hard as deciding
whether a loop ends). *Recommendation: A.* Bytes: no.

**8. What an atomic returns as the "old value".** When many threads atomically add to one location, each gets back the value
just before its own step. Unpopped says it is the value in *some* ordering of the steps that agrees with each thread's own
program order; which ordering is not stated. Options. **A.** Adopt that wording. Consequence: every real device satisfies
it and a test can check the set of possible answers. **B.** Pin one ordering (for example by thread number).
Consequence: tests are exact, but real devices do not behave that way and every implementer must slow down to match.
**C.** Say nothing. Consequence: two readers cannot be told apart. *Recommendation: A.* Bytes: no.

## Choices that are small and low-risk (the PM may rule them)

**7. A whole-grid barrier assumes every block is running at once.** A barrier that waits for the entire launch can only work
if all blocks fit on the device at the same time (on NVIDIA this needs a special "cooperative" launch). Unpopped assumes this.
Options. **A.** Adopt, and make "does the target support it" a fact the target states (the Announce device record already
carries per-device capabilities). Consequence: a program using it either runs correctly or the target says it cannot.
**B.** Drop the whole-grid barrier from the first version. Consequence: nothing can mis-state it, but grid-wide sums cannot
be expressed. *Recommendation: A.* Bytes: no.

**10. "Constant" memory has a size but no initial contents.** Unpopped lets a program declare constant memory but gives no
way to fill it, so it refuses to read one. Options. **A.** Same in the first version. Consequence: nothing to specify now;
programs that need constants pass them as ordinary inputs. **B.** Add initializers now. Consequence: more text and tests for a
feature no consumer has asked for. *Recommendation: A.* Bytes: no.

## One finding for the board

Choice 3 is the only one where an existing clause points away from Unpopped, and it was ruled by the PM for Unpopped's own
use on 2026-10-31, not for KISS. If KISS-Kernel silently adopted Unpopped's rule, KISS would contain two answers to "what is
integer division by zero". That is why I recommend A, and the same reasoning applies to any later choice that touches an
operation KISS-Ops already defines.

## What the PM asked for, and where I stopped

I did not write any clause text for these eleven choices. My recommendations are A for choices 1 through 8 and 10, with 9
and 11 closed by existing text (board 417 and board 450) and the exclusion half of 1 closed by board 450. In my reading choices 1, 2, 4, 5, 6 and 8 fix what a correct program may rely
on and should go to CireSnave; 3 needs the PM to say whether it is a KISS-Ops matter; 7 and 10 are low risk. That split is a
recommendation, not a ruling.

— lane: kiss
