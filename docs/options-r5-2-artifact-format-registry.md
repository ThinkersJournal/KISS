# Options note: R5.2, who owns the `artifact_format_tag` registry?

**Informative. No normative text. For CireSnave's decision (board 279, pair 2 of 5).**
Evidence read at `origin/main`: `spec/synth.md` §6.4-0001d and Appendix B; `WIRE-FIRST.md` §1.3;
`spec/umbrella.md` §6.4 (the extension registry, PR-gated, under ThinkersJournal).

## The question

A provision response carries a 4-byte `artifact_format_tag` naming the artifact's binary format and
target family. KISS-SYNTH-6.4-0001d pins the field (4 bytes, byte-matched, opaque, never all zero) and
`synth.md` calls it "registry-assigned", but no registry of values exists (Appendix B says so). Because
6.2-0005b forbids a consumer from using out-of-band information to interpret the artifact, a provider
cannot lawfully say "this is cubin" and a consumer cannot lawfully guess. **Who owns the values, and
where do they live?** Appendix B adds two smaller questions: may a provider answer "building, retry"
instead of blocking within the bound or declining; and what is the paging and cache-invalidation story
for a provider offering thousands of cells.

## Options

**A. KISS-Synth owns it; the values live in a Synth appendix.** Simplest, fastest, versioned with the
sub-standard. *Consequence:* a new value is a Synth text change (a PR, and a version), and the registry cannot
be shared with Announce (Appendix B leaves that sharing question open).

**B. The umbrella extension registry holds it, PR-gated under ThinkersJournal.** The registry already
exists, has a lifecycle (experimental, arbitrated, core) and names the owning clause for each entry.
Values can be added without editing Synth, and Announce can reuse them. *Consequence:* the umbrella is
informative-only, so each entry would point back to a normative Synth clause; one more thing the steward
maintains. This is the pattern the namespace registries (`cuda`, `vulkan`) already follow.

**C. Vendor-namespaced tags with no central list.** A provider mints its own tag under a prefix it owns
(the 4 bytes leave little room; a collision is possible). *Consequence:* no coordination cost, but two
providers can collide and the consumer's byte-match then decides wrongly with no way to tell.

## Recommendation

**B**, seeded with the values the reference provider needs today (WIRE-FIRST's example is cubin; the
provider names the rest, and nothing else is assigned). It matches how namespaces are already
handled and keeps Synth text stable. The smaller questions need separate answers: I recommend that a
provider **MAY NOT** answer "building, retry" until a consumer needs it (it adds a response state to
every reader), and that paging waits for a provider that actually exceeds the `2^20` record bound.

## What freezes, and what to decide

Synth cannot freeze with the registry open (Appendix B says "before any KISS-Synth freeze"), and until a
value is assigned the provision flow is not encodable end to end (WIRE-FIRST §1.3). Decisions needed:
**A, B or C**; the seed values and who proposes them (Baracuda, as the provider); and the two
"retry" and "paging" defaults above.
