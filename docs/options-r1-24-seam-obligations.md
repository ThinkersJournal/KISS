# Options note: R1.24, seven obligations the seam still lacks

**Informative. No normative text. For CireSnave's decision; the PM turns each question into a board item.**
Read at `origin/main` `b9e08e7f`, 2026-10-26. Paths are `file:line` at that commit. Issues: KISS#10, #24, #25, #26,
#124, #289, #317 (Fuel, Baracuda, Unpopped). Roadmap row: R1.24. Nothing here freezes. Two proposals below are
**partly already answered** by text that landed after the issue was filed (Q5 and Q4), and one proposal as worded
would be **wrong** (Q3); each says so.

---

## Q1. Do two parties agree on an op vocabulary during the handshake? (#10)

**Example.** A provider has just added a new fused op `rms_norm_residual`. A consumer has never heard of it. Today the
consumer meets the op name inside a contract, does not know it, and resolves it through the reference decomposition
down to the primitive ops it does know. Nothing in the handshake told either side this would happen.

**What the specs say.** There is no op-level handshake: the handshake advertises that a side speaks Ops at all
(`SPEAKS_OPS`, `spec/announce.md:594`) and a profile number. An unknown op resolves by its decomposition and adding a
high-level op never forces a consumer change (`spec/ops.md:2608`, KISS-OPS-7.3-0001); registering a new op name
advances the op-vocabulary schema version but not the primitive-floor version (`spec/ops.md:2643`, KISS-OPS-8-0002).
So agreement is, in effect, "artifact-pinned and resolved on miss". The gap is that no sentence says this is
intended, so an implementer may wait for a handshake exchange that never comes.

**Options.**
- **A. State it once.** A short Announce clause: op-vocabulary agreement is intentionally not negotiated in the
  handshake; it is pinned per artifact and resolved on a miss by decomposition. Add the second small sentence the
  issue asks for: a provider-private spelling of an op (an importer name) is an adapter outside the KISS op
  vocabulary, not a second namespace. No bytes change.
- **B. Add an op-set channel to the handshake** (each side advertises a registry snapshot). New bytes and a new
  structure to keep in step with the registry; saves one decomposition round trip.
- **C. As A, plus give the vendor tier its own version** so adding a vendor op does not advance the core schema
  version. Changes the Ops versioning rule (8-0002), which today applies to every registered op.

**Recommendation: A.** It records what the specs already do and costs two sentences. B pays wire bytes to save a
round trip that works. **C is a separate, later question:** no vendor-tier op exists yet, so there is nothing for it
to protect.

---

## Q2. Must a consumer survive a bad artifact without crashing? (#24)

**Example.** A consumer loads a provided kernel and binding it throws (a malformed header, an unsupported object kind,
a failed verification). The consumer should return an error, not abort.

**What the specs say.** Never-panic is stated for the provider (`spec/synth.md:796-805`, KISS-SYNTH-6.6-0001 and
-0001a) and for the lifter (`spec/consume.md:799-812`, KISS-CONSUME-6.5-0001 and -0002). Nothing binds the
**consumer of a provided artifact**, which is the party in the umbrella's own words that "binds nobody" in the response
direction (`spec/umbrella.md:430`, section 10.5 item 1). The just-merged trust text calls the artifact untrusted code
(KISS-SYNTH-6.4-0001f, in #679).

**An honest limit first.** A consumer can only promise what it controls. A fault inside the kernel's own native code
once it is launched (a bad memory access, a GPU fault) cannot be caught by any seam clause. What a consumer *can* do
is turn every failure it observes **at the seam** (a malformed frame or contract, an unsupported object kind, a
refused load, a failed verification) into a typed error.

**Options.**
- **A. Add a consumer-side clause scoped to the seam:** a failure observed while receiving, loading, binding or
  verifying a provided artifact MUST become a typed error and MUST NOT abort the consumer. Home: KISS-Synth, next to
  the provider's obligation, since the artifact is Synth's.
- **B. As A, but also require isolation** (run the kernel in a sandbox). KISS has said it will not be a sandboxing
  standard (`spec/umbrella.md:394` says KISS provides no sandboxing; `:437`, section 10.6, lists the non-goals).
- **C. Nothing.**

**Recommendation: A, with its limit written into the clause.** It is the mirror of an obligation the specs already
impose on every other party, and it costs no bytes. **Not B:** that is the deployment's job.

---

## Q3. Is the live capability set the AND of the two sides? (#25) **The proposal as worded is wrong.**

**Example.** A provider sets `CONTRACT_QUERY` (it can answer queries). A consumer cannot answer queries, so it must
write that bit as zero (`spec/announce.md:645`, KISS-ANNOUNCE-7.3-0002: a role that cannot provide a provider-only
feature MUST write it as zero). The issue proposes `capabilities = local & remote`. Applied to this pair that gives
`CONTRACT_QUERY = 0`: the one feature the provider actually offers is reported as not live.

**What the specs say.** Profile selection is pinned (`spec/announce.md:546`, KISS-ANNOUNCE-7.1-0001: the highest
common profile). How the two capability words combine is not stated anywhere, so two independent implementations may
combine them differently.

**Options.**
- **A. A role-aware rule.** Bits that describe what both sides *speak or support* (the sub-standard and external
  token axes) are the AND. Provider-only feature bits are read from the provider's word, and are live if the
  provider set them. Add one golden negotiation vector so two implementations can be compared byte for byte.
- **B. The issue's plain AND.** Simple, and wrong for provider-only bits as shown.
- **C. Say nothing** and let each consumer combine them.

**Recommendation: A.** It is the rule the existing zeroing requirement already implies, and the vector makes it
checkable. **Please note for Fuel:** if `negotiate()` returns the plain AND, provider-only features will read as off
(I did not read Fuel's code; this follows from the issue's wording and the clause above).

---

## Q4. May `revision_hash` be shorter than 32 bytes? (#26)

**Example.** A provider's native build identifier is a 64-bit counter. Today it must put it in a 32-byte field
(`spec/announce.md:381`, KISS-ANNOUNCE-6.3-0003, "exactly 32 bytes").

**What the specs say.** The value is an opaque label compared for equality only (`spec/announce.md:383`, 6.3-0007;
6.3-0008 and, in #679, 6.3-0008a: it is not a hash and not an integrity check). The issue says a fixed width forces a
"lossy re-encoding". Zero-padding a 64-bit counter into 32 bytes is **not lossy**: it is a one-to-one mapping.
The cost is space: 24 wasted bytes per record, and a list may carry up to 2^20 records (about 24 MiB).

**Options.**
- **A. Keep exactly 32 bytes** and add one informative sentence: a shorter native identifier is padded with zeros,
  which is one-to-one.
- **B. Make it a bounded, length-prefixed token** (8 to 64 bytes). Changes the availability record and the identity
  block bytes: **Breaking**, and every speaker regenerates.
- **C. Keep 32 now and revisit if the 24 MiB list cost is ever measured to matter.**

**Recommendation: A (which already contains C).** No provider is blocked, and a length prefix would add a second
Breaking wire change on top of the correlation-field change already in flight (#678). **If a provider reports the
list size as a real cost, B is the answer** and is cheaper to do together with an existing Breaking change.

---

## Q5. Must a consumer check a received kernel before trusting it? (#124) **Partly answered.**

**Example.** Fuel shipped, then reverted, a fused rotary-embedding op where the provider's op and its own shared a
name and computed different functions. Nothing at the contract layer caught it; a numeric differential did.

**What the specs say (newer than the issue).** KISS-Conform already ships a **consumer-verify check**
(`spec/conform.md:1216`, KISS-CONFORM-6.12-0002) that is a governance SHOULD, not build-gating by default
(`:1221`, 6.12-0003), and defines an **opt-in trust-but-verify profile** that makes it gating
(`:1464`, KISS-CONFORM-7.2-0003). So "no clause exists" is no longer true. What is **still missing** is the part of
the issue that goes beyond a pass/fail check: the **outcome vocabulary** (adopt as declared, adopt demoted and
recorded, reject), the **durable record** per (kernel, cell, target), and re-verification when `revision_hash`
changes. The issue itself calls the demotion a local reclassification, not a claim that the provider's contract is
wrong.

**Options.**
- **A. Extend the existing opt-in profile** with the three outcomes, the per-(kernel, cell, target) record and an
  invalidate-on-new-revision rule. No wire change; optional; consumes the oracle Conform already owns.
- **B. Leave the existing profile as is** and publish the pattern as informative guidance.
- **C. Make a richer lattice** (record how far the kernel missed). More useful, harder to pin.

**Recommendation: A, with a single demoted state.** One implementation (Fuel) runs it in production; a second
implementer is the right moment to add the lattice. **What waits:** a witness in a second implementation
(ROADMAP R1.31 is the same kind of owed witness for the untrusted-artifact clause).

---

## Q6. Must a refused byte-match be a visible decline? (#289)

**Example.** A dispatch table finds the one kernel whose key byte-matches an invocation, calls it, and the kernel
refuses (it does not compute what was asked). The caller has no row to fall back to and no signal to act on.

**What the specs say.** The key match is necessary but **not sufficient**: a kernel "MAY decline an invocation whose
derived key byte-matches its own" (`spec/classify.md:1149`, KISS-CLASSIFY-6.6-0001, weakened in #288). Removing the
"if and only if" also removed an implicit liveness guarantee. The obligation that closes it is a consumer-side one and
Classify does not own consumer behaviour, so the issue puts it in KISS-Consume (the #275 ruling it cites forbids
stating one obligation in two sub-standards).

**Options.**
- **A. Add a clause to KISS-Consume:** a kernel that declines an invocation whose key matches MUST report a typed
  decline; it MUST NOT decline silently, abort, or return a result from a different computation. At least one
  consumer already models this.
- **B. Put it in Classify.** Rejected by the #275 ruling (duplicated obligation).
- **C. Nothing.**

**Recommendation: A.** It restores the liveness property without restoring the unsound "must admit". No bytes change.

---

## Q7. How does a loader get from `entry_point` to a callable? (#317)

**Example.** A contract names `entry_point = "vec_add_f32"`. A loader is handed the artifact bytes. Which object, and
under which ABI, carries that symbol?

**What the specs say.** The Interface section names the symbol only (`spec/contract.md:854-872`, 6.5-0001 and
-0002). The one nearby field, `bundle_envelope`, is optional and catalog-level (`spec/contract.md:1276`,
KISS-CONTRACT-6.9-0009), so a lone kernel cannot rely on it.

**What changed since the issue.** In a provision response the artifact travels in the same frame as the contract
(KISS-SYNTH-6.4-0001b), and its `artifact_format_tag` names the format (KISS-SYNTH-6.4-0001g). For the one seeded
format (`PTX `) the symbol is resolved inside the artifact bytes themselves. Resolution only becomes an open question
for a provider whose artifact does **not** carry its own symbols (several objects linked at load).

**Options.**
- **A. Add a per-kernel link field** to the Contract (the issue's architect ruling): Breaking, every contract
  regenerates.
- **B. Say that, for a provision response, `entry_point` resolves inside the artifact**, and record "how symbols
  resolve" as a column of the `artifact_format_tag` table, the same place the calling-convention question
  of the R2.8 note (its Q4) goes. Additive.
- **C. Make the bundle roster mandatory** whenever the artifact is not self-contained.

**Recommendation: B now, A only when a provider with a non-self-contained artifact exists.** The table already
exists and a new tag row can carry the rule; nothing is lost by waiting. **Not a decision yet** because only one
format is seeded.

---

## Summary

| Q | Issue | Recommend | Bytes change? | Note |
|---|---|---|---|---|
| 1 | #10 op-vocabulary agreement | A (state it) | no | already how it works |
| 2 | #24 consumer never-panic | A (scoped to the seam) | no | limit stated |
| 3 | #25 capability AND | A (role-aware) | no | issue's plain AND is wrong |
| 4 | #26 `revision_hash` width | A (keep 32) | no | padding is one-to-one |
| 5 | #124 verify-before-adopt | A (extend opt-in profile) | no | profile exists since 7.2-0003 |
| 6 | #289 refused byte-match | A (Consume clause) | no | |
| 7 | #317 `entry_point` resolution | B (tag table) | no | only PTX seeded |
