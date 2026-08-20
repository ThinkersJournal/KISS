# RFC — Can the emit/consume round-trip guarantee be made structural?

**Status:** Draft for cosignature. **Raised by:** Unpopped, editor of record for
KISS-Emit and KISS-Consume. **Against:** #232.

**Cosigners sought:** the round-trip parties. The author holds the pen on *both*
documents this concerns, which is the reason the RFC exists rather than a reason
to trust it.

---

## The question

Not *"shared section or correspondence table."* That framing presumes a shared
section is available, and it is not (see Obstacle).

> **Can the round-trip guarantee be made structural, given that clause IDs are
> per-document and KISS-Emit and KISS-Consume are declared DAG siblings?**

If the answer is no, the honest outcome is a table **plus an explicit statement
that its correctness depends on maintenance.** A worse guarantee stated
accurately beats a better one assumed.

## Motivation

KISS-Emit §6.7-0008 pins its round-trip statement semantically identical to
KISS-Consume's via an enumerated correspondence table, verified by a
cross-standard lint. That mechanism has a measured hole and a maintenance
dependency.

**One editor now holds both pens.** That makes divergence less likely and is
precisely why this is worth asking now:

> **One pen makes drift less likely; a shared section makes the asymmetry
> impossible. Those are different guarantees, and the appointment only bought the
> weaker one.**

The failure mode stops being *"two editors diverge"* — which the lint was built
for — and becomes *"one editor forgets,"* which nothing in the suite detects.
**The mechanism that would catch drift got weaker at the moment the person it
depends on got more powerful.**

## The measured drift (three findings, all current)

1. **§6.7-0006 defines the tier-2 determinants** — the language-identity token and
   byte-equal `target_capability` — and opens *"For the Emit direction."* It sits
   **outside** the correspondence table. So the lint compares §6.7-0002 ↔
   §6.6-0002, the clauses that *use* those terms, while excluding the clause that
   *defines* them. **Two implementations can disagree about tier-2 eligibility and
   both pass.** Consume states the same words with no determinant pinned.
2. **§6.7-0009 ↔ §6.6-0006 already differ in text.** Consume's whole-kernel
   admissibility carries *"(and the same-language, on-device restriction of
   §6.6-0002 holds)"*; Emit's states only the exact-byte condition. The pair is
   outside the table, so nothing asks.
3. **Consume carries no table.** Its only tether is three prose references into a
   document someone else may be editing.

Plus a defect in the guard itself: `consume.md:805` cites
`test_conform_emit_consume_roundtrip_correspondence`, **which exists nowhere**,
while `emit.md` cites `test_conform_emit_consume_correspondence_lint`, which
exists. It survives because it sits in **prose inside a blockquote**, not a
`*Test:*` binding — outside what `kiss_trace` can structurally see. *This needs no
ruling and should be fixed independently of this RFC.*

## The obstacle (measured, at `origin/main`)

**A shared normative section has nowhere to live.** `spec/umbrella.md` is
informative-only and that is **enforced**, `tools/kiss_trace.py`:

```python
if res.stem == "umbrella":
    for cid in sorted(body_ids | matrix_ids):
        res.add(f"umbrella is informative-only but defines a clause: {cid}")
    return
```

Verified: zero `KISS-UMBRELLA-*` clauses exist. Clause IDs are append-only and
per-document, so **there is no ID space both documents can draw from.**

## The two viable shapes, with their real costs

**A. A new sub-standard for the round-trip.** Its own prefix, §6, §8 freeze gate,
and DAG position. Single-sourced and structural. **Cost:** a tenth document in a
suite whose proposition is that the nine are minimal — and its own freeze gate to
satisfy.

**B. Consume carries no round-trip clauses and cites Emit's normatively.** The
true single-source fix; dissolves the asymmetry completely.

**Cost, as it is usually stated:** it creates the dependency edge the DAG denies
— `emit.md` §0: *"a **sibling** of KISS-Consume … neither depends on the
other."*

**But that statement is already false in substance, which changes the
proposition.** Consume defers to Emit's correspondence table in **three places**
in prose, **and** its tier-2 clause relies on a determinant — `language-identity
token` — that **KISS-Emit defines and KISS-Consume does not** (see Addendum).
Two independent informal dependencies already exist.

**So B would be *documenting* an edge rather than creating one.** That is a
materially different proposition from the one §0 makes it sound like, and the
honest comparison is not "add a dependency vs keep independence" but "**make the
existing dependency explicit and directional, vs leave it implicit and
unenforced.**" The sibling relation may still be load-bearing for reasons beyond
this RFC — but it should be defended on those reasons, not on an independence the
documents do not actually have.

**C. Status quo plus rows** (being done now, independently): add §6.7-0009 ↔
§6.6-0006 and a Consume twin for §6.7-0006. **Closes the measured hole. Does not
make the table self-maintaining** — the table is itself the class of artifact that
rots.

## What the author recommends

**Do C now** — it is strictly an improvement and blocks nothing.

**Then decide A vs B vs "C is the ceiling."** If C is the ceiling, say so in the
text: *the round-trip correspondence is maintained, not structural, and its
correctness depends on the editor updating both sides.* That sentence costs
nothing and stops a future reader assuming a guarantee the mechanism does not
provide.

The author has no stake in A over B and will implement whichever wins.

## Note on the author's position

Unpopped holds the pen on both documents. This RFC proposes reducing that pen's
latitude, which is the direction that should invite less suspicion rather than
more — but it remains an editor proposing about his own authority, and it is
raised as an RFC precisely so it has cosigners by construction rather than by
exception.

## Addendum — the obstacle is about a shared TERM, not only a shared section

Found while implementing option C, and it is sharper than the section problem
because it is smaller and concrete.

**C's second row could not be authored.** A Consume twin for §6.7-0006 needs two
determinants:

| determinant | owner | in `consume.md`? |
|---|---|---|
| `target_capability` (device) | KISS-Classify | **yes** — 9 occurrences. Clean. |
| **language-identity token** (language) | **KISS-Emit §3** | **no** — zero occurrences |

Consume's §3 uses "source language" informally and defines **no determinant**. So
the language half can only be authored three ways:

- **cite Emit's term** — creates the dependency edge `emit.md` §0 denies. Shape B's
  cost, in miniature.
- **duplicate the definition into `consume.md` §3** — two definitions that can
  drift, which is precisely the problem this RFC exists to solve.
- **move the term to a neutral owner** — nowhere to put it; `umbrella.md` is
  informative-only and enforced.

**So the same obstacle recurs one level down.** The question is not only *where
does a shared normative section live* but **where does a shared normative term
live** — and `language-identity token` is the concrete instance. It is a better
test case than the section, because whatever shape resolves it for one term
plausibly resolves it for the statement.

**Consequence: option C is partially blocked.** The §6.7-0009 ↔ §6.6-0006 row is
clean and is landed. The §6.7-0006 twin is not authorable without choosing one of
the three above, which is a cosigner decision rather than an editorial one.

## If the answer is no, that is a finding — not a fallback

**Required of whatever this RFC concludes.** If the guarantee cannot be made
structural under the current ID and DAG constraints, the outcome must be recorded
as:

> **We asked whether the emit/consume round-trip guarantee could be made
> structural, established that it cannot under per-document clause IDs and the
> declared sibling relation, and therefore state the guarantee honestly instead.**

Not as *"we added a row."*

**Two reasons, and the second is the load-bearing one.**

It is a considerably stronger artifact: a reader learns what was asked, what was
established, and why the weaker guarantee is the honest one rather than the lazy
one.

And **an unrecorded negative result is expensive.** The next person to notice the
correspondence table is maintained rather than structural will re-open exactly
this question, re-derive the umbrella constraint, re-discover the borrowed term,
and arrive back here — because nothing in the documents says it was asked and
answered. **A settled question that does not look settled gets re-litigated**, and
the cost lands on whoever is next rather than on whoever failed to write it down.

The same then applies in the spec text, per §"What the author recommends": if C is
the ceiling, both documents say so. **A reader of `consume.md` should not have to
find an RFC to learn what the guarantee is worth.**
