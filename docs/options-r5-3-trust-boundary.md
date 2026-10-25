# Options note: R5.3, trust-boundary and integrity statements

**Informative. No normative text. For CireSnave's decision (board 279, pair 1 of 5).**
Evidence read at `origin/main` 9336a5b: `spec/umbrella.md` §10.5 items 1-5 (its own list of "normative
clauses that do not yet exist"); `git grep 'untrusted code' spec/synth.md` finds nothing. R1.24 already
covers the #26/#10/#25 part (revision_hash length, op-vocabulary agreement, capabilities AND); this note
is the rest.

## The question

A provider hands a consumer an artifact, which is executable code. The suite says nothing normative
about whether to trust it, what the checksums do and do not prove, how long an attacker-supplied
length may be, or where a signature would go. Umbrella §10 says so itself ("the honest statement of a
gap rather than a description of a defence"). Five items:

1. Cap every attacker-controlled length (`artifact_len` is a u64 from the less-trusted party; every
   never-panic obligation in Synth is written for the request direction, so the response direction,
   the one carrying code, binds nobody).
2. State the trust boundary: a consumer MUST treat a provided artifact as untrusted code unless the
   provider is in its own trust domain.
3. Say `revision_hash` is not an integrity mechanism (Announce 6.3).
4. Say the contract checksum detects corruption, not modification (Contract 6.2).
5. Reserve an optional signing/attestation extension point (umbrella §6.4 registry, Axis 2). The umbrella
   calls this "the item most worth doing before any sub-standard freezes", because adding it after a
   freeze is far more expensive.

## Options

**A. Do all five now.** Complete, but item 1 needs a number per field and so is a wire-visible limit,
and it overlaps R5.1's CRSP cap: two lanes deciding one number twice. About six new clauses, each
`untested` on arrival (floor untested rises by six; harness unmoved).
*Touches:* Synth §1 and §9, Announce 6.3 and 6.4, Contract 6.2, the umbrella §6.4 registry.

**B. Items 2, 3, 4 and 5 now; item 1 with R5.1.** The four are statements and a reserved slot, with no
byte on the wire and no number to choose. Item 1's caps land in the same change as the stream
binding, where the CRSP cap is already being set, so the lengths are decided once.
*Consequence:* the "two-line frame OOM-kills a consumer" hazard stays until R5.1 lands.

**C. Defer all to a post-freeze security profile.** Nothing to write now. The extension point would
then be added after a freeze, which the umbrella itself calls far more expensive, and the
untrusted-code statement stays missing from the normative text.

## Recommendation

**B.** It spends the cheap, irreversible-if-late decisions (the statements and the reserved
extension point) now and joins the one decision that carries a number to the work that already has to
pick one. Per the umbrella's own non-goals (§10.6), KISS leaves the extension point and names the
owners (TLS/Noise, Sigstore/in-toto); it does not specify signing.

## What freezes, and what to decide

Item 5 is the one with a deadline: it must exist before any sub-standard freezes. Decisions needed:
**A, B or C**; and whether the reserved extension point is a capability bit (Axis 2) or a registry
entry only (I do not know which fits the registry's current shape; the editor should say).
