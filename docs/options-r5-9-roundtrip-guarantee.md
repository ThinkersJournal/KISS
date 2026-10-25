# Options note: R5.9, can the emit/consume round-trip guarantee be structural?

**Informative. No normative text. For CireSnave's decision (board 279, pair 2 of 5).**
Evidence read at `origin/main`: `rfcs/roundtrip-structural-guarantee.md` (Unpopped, 2026-08);
`spec/consume.md` Appendix D item 2 ("remains open"); `spec/emit.md` §0 and §6.7-0008. The RFC's
"option C" half is already merged (the §6.7-0009 to §6.6-0006 correspondence row); the Consume twin of
Emit §6.7-0006 is the part still unauthored.

## The question

KISS-Emit and KISS-Consume must say the same thing about what a round trip preserves. Today that holds
because Emit's §6.7-0008 lists which clauses correspond and a lint checks the list. Two problems: the
lint covers §6.7-0001..-0005 only, so §6.7-0006, which defines the tier-2 determinants (the language
identity token and the byte-equal `target_capability`), sits outside it; and Consume has no table of its
own, only three prose references into Emit. One editor now holds both pens (resolved 2026-08-15), which
makes drift less likely but not impossible. **Can the guarantee be made structural, or do we say it is
maintained, not structural?** Obstacle: clause IDs are per-document, `umbrella.md` is informative-only
(enforced in `kiss_trace.py`), and Emit and Consume are declared DAG siblings (`emit.md` §0), so a
shared normative section has nowhere to live.

## Options

**A. A tenth sub-standard for the round trip.** Own prefix, §6, §8 freeze gate, DAG position.
Single-sourced and truly structural. *Cost:* a tenth document in a suite whose proposition is nine, and a
new freeze gate (R4) to satisfy; Emit and Consume cite it, which edits both.

**B. Consume cites Emit's round-trip clauses normatively.** One source of truth, the asymmetry dissolves.
*Cost:* it creates the DAG edge `emit.md` §0 denies. The RFC argues that edge already exists in
substance (three prose deferrals, and the `language-identity token` is defined only in Emit). Needs a
ruling that the sibling relation is not load-bearing, or a documented directional edge. Would touch
Consume §3 and §6.6, the DAG statement in both documents and the umbrella, and the cross-standard lint.

**C. Accept "maintained, not structural" and say so in both documents.** Land the missing rows (the
§6.7-0006 twin needs one of three choices for the shared term: cite Emit's, duplicate the definition into
Consume §3, or move it to a neutral owner, and there is no neutral owner). Add one sentence to both
documents that correctness depends on the editor updating both sides. *Cost:* nothing structural; the
table remains the kind of artifact that rots.

## Recommendation

**C now, with the sentence, and decide A vs B only when a second editor or a foreign reader (R4.1)
exists.** The RFC's own author recommends the same ordering, and it is the only option that
spends nothing irreversible before the suite has outside readers. I would not pick B on the strength of
"the edge already exists": that argues for documenting it, not for deciding it now.
If CireSnave prefers a structural answer, **B** is cheaper than **A**.

## What freezes, and what to decide

Neither Emit nor Consume freezes before R4. Decisions needed: **A, B or C**; and for the shared term
`language-identity token` (needed even under C): cite, duplicate, or move. The editor of record
(Unpopped) holds the pen on both and has said it will implement whichever wins.
