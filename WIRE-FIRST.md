# Wire-first: what blocks one real handshake + provision

**Informative. A work order, not a specification.**

This document answers one question: *what stops Baracuda (provider) and Fuel (consumer) from
completing one end-to-end flow — handshake → availability → contract-query → provision →
bind & launch — over a real process boundary, today?*

It is the output of a clause-by-clause trace of that path, in which each candidate blocker was
then independently checked against the primary text by a reader whose job was to **refute** it.
46 candidates were traced; **38 survived**: 11 hard blocks, 24 soft blocks, 3 annoyances. What
follows is only what survived.

**The headline: the blockers are not missing code. They are missing decisions and missing
bytes.** Every item below is answerable from the desk. None needs hardware.

> **Dated snapshot — read this as of 2026-07-15; several items below are now closed
> (update 2026-07-23).** (1) **§1.4 Dispatch is resolved.** The §6.6-0006 grammar gained `/`,
> `ceil_div`, tuple-valued results and an element subscript, so `ceil(n/block)` and
> multi-dimensional grids *are* expressible; and because every reference implementation
> declines to emit a Dispatch section at all (a grid-stride kernel declares no geometry and the
> host picks the launch), **§6.6 Dispatch is now optional** with a geometry-agnostic kernel
> class. The question this document poses in §1.4 has been answered — it is not still open.
> (2) **The "no byte has crossed a process boundary" framing is overtaken.** Independent
> `structure_key` derivations now byte-match three ways (KISS, Fuel, Baracuda) — first at `sk2`
> for the `relu_add` cell, then re-verified at `sk3` after the schema bump. (3) Several §1.5
> `structure_key` derivation gaps are closed in the current spec. For the current
> cross-implementation state, see
> [`docs/kiss-convergence-reconciliation.md`](docs/kiss-convergence-reconciliation.md); for
> live coverage, run `python tools/kiss_trace.py --report`.

---

## 0. What is already green — start here

Do not re-litigate these. They are implementable from the document alone, today:

- **The 56-byte handshake envelope.** The §6.1 offset table (announce.md:274-283), the §2.5
  worked example (announce.md:150-163), and the one executed golden
  (`conformance/tests/announce_golden.rs`) are mutually consistent and byte-exact. Verified
  including the capabilities arithmetic: EXT bits 0–5 = `0x3F`, FEAT 32|33 = `0x3_0000_0000`,
  total `0x0000_0003_0000_003F`, wire `3F 00 00 00 03 00 00 00`. **Baracuda and Fuel can each
  produce a byte-identical envelope right now.**
- **Profile selection.** `max(L ∩ R)` (§7.1-0001) is unambiguous, and §7.3-0001 makes it
  symmetric and order-independent — both sides compute it locally, no ACK round needed.
- **Envelope hard-reject.** Everything in §6.2 except the ordering clause is implemented and
  tested (8 decline vectors).
- **Reading the provider's FEAT bits.** §7.2-0003 pins bit 32 `PROVISION_ON_REQUEST` and bit 33
  `CONTRACT_QUERY`.

**Issue #25 (the negotiated-capability AND) does not block this flow.** The consumer needs only
bits 32 and 33, both provider-only bits that §7.3-0002 forbids a consumer from setting — so
`local & remote` and `remote` are the same value for every bit this flow reads. Lifting the AND
is cheap and correct, but it is a divergence risk, not a wall. Don't spend a decision on it now.

---

## 1. The hard blocks

Ordered by what stops the flow earliest.

### 1.1 There is no message boundary — two processes cannot connect

**The single most important item in this document.**

KISS-Announce §1 (announce.md:64-66) explicitly claims Announce owns *"the framing of every
message."* §1 (announce.md:72-76) then explicitly disclaims the transport. **§6 delivers
neither an outer frame header nor any rule by which a stream reader finds a message boundary.**

> Fuel opens a socket to Baracuda. Baracuda writes 56 bytes. Fuel calls `read()` and gets 40.
> Is that a short read on one envelope, or a truncated message it must hard-reject under
> §6.2-0001? **The document never says.**

Worse, five clauses bound-check against a byte range nothing defines — §6.2-0001 ("any input"),
§6.2-0007 ("the input buffer"), §6.3-0009 ("the remaining input"), §6.3-0010, §6.4-0006 ("the
request buffer"). §6.2-0001 is only meaningful on a *datagram* transport where a boundary is
supplied from outside, and §1 refuses to name any transport. **The one clause defining envelope
rejection is unenforceable on TCP or a pipe** — i.e. on what Baracuda and Fuel would actually use.

**Resolution (recommended):** add §6.6 "Stream binding". No wire change to existing messages.
1. **§6.6-0001** — a leading 4-byte tag. Dispatch on **all four bytes**: SEAM/SAVL share `0x53`
   and CYRQ/CRSP/CDEC share `0x43`, so a 1-byte dispatch is wrong. Source the tag namespace from
   normative §6.1's table, §6.3-0011, §6.4-0001/0004/0007 — **not** from §2.4, which is
   informative and cannot carry a normative clause.
2. **§6.6-0002** — each message is self-delimiting once tagged (verified true for all five today).
3. **§6.6-0003** — unrecognized tag → typed decline, no resynchronization (consistent with §6.2-0008).
4. **§6.6-0004** — cap the CRSP payload. §6.4-0004 has a u32 length with **no cap and no
   allocation guard**, unlike §6.3-0009/0010 which both say "without allocating on the unchecked
   length." Without this, §6.6 turns an unreachable bug into a reachable 4 GiB allocation from
   four attacker-controlled bytes.
5. **Define one term — "message extent"** — and amend all five referents above to use it.
   *Amending only §6.2-0001 leaves the §6.3-0009/0010 bounds-check evaporation intact.*

**Do not fix this by editing §1.** §1 is informative and its transport/framing split is correct.
The bug is in §6.

**Resolve jointly with issue #18** (in-process binding profile). #18 wants *no* framing for linked
providers; this wants *complete* framing for cross-process ones. Both are asking for the same
missing concept: a named **binding profile** under the umbrella's capability/profile model.
§6.6 is the wire profile; #18 is the in-process profile.

**Vectors to ship (both writable before any transport exists):**
- SEAM+SAVL concatenated on one stream → proves dispatch.
- **SAVL whose `record_count` overruns its own extent into a following CRSP → typed decline.**
  This is the adversarial one: it is the exact case where two readers conforming to today's text
  behave *oppositely*, and it is the vector that would have caught this.

### 1.2 The same request bytes mandate two different responses

FEAT bit 33 alone obligates a provider to answer **one identical CYRQ byte sequence** with two
different mandated frames: **CRSP** per KISS-Announce, **PRSP** per KISS-Synth. A conforming
provider cannot satisfy both. This is a direct cross-document contradiction on the flow's
critical path — an owner decision, and a cheap one.

### 1.3 The PRSP provision response is unencodable

- **`artifact_format_tag` has no value registry.** synth.md:351 already calls it
  "registry-assigned"; synth.md:1283 admits the registry does not exist. §6.2-0005b then forbids
  the consumer from using out-of-band information to interpret the artifact. So Baracuda cannot
  legally say "this is cubin," and Fuel cannot legally guess.
- **No golden contract document exists.** Appendix C is *not a document*: it renders only
  Identity+Semantics, leaves the header's `len`/`crc32` as unresolved placeholders, and its own
  text says the Interface/Dispatch/Capabilities/Guarantees/Provenance blocks "follow" — they do
  not; the appendix ends first.
- **Seven Contract field values have no pinned encoding**, including `per_backend_ulp_tiers` and
  `cost`: §6.11-0001 pins no map type, no record type, and no float encoding.
- **`revision_hash` has neither a type nor a width in KISS-Contract**, so the wire's 32 raw bytes
  and §6.2-0006's byte-for-byte equality requirement are uncheckable. (Pairs with issue #26.)

### 1.4 Dispatch is unwritable in its own grammar

> **RESOLVED (2026-07-23) — see the snapshot banner above.** The §6.6-0006 grammar has since
> gained `/`, `ceil_div`, tuple-valued results and an element subscript, so the specifics below
> are overtaken; and **§6.6 Dispatch is now optional** (geometry-agnostic kernel class), because
> every reference implementation declines to emit the section. The item is retained for the
> reasoning that produced the fix, not as an open blocker.

**This is the flagship contribution, and it cannot express its own mandatory fields.**

§6.6-0001 mandates five Dispatch derivation fields. The §6.6-0006 expression grammar has:
- no array subscript — so nothing can reach the rank-length `extent`/`stride`/`offset` arrays;
- no thread-index symbol — so the grid-stride `thread_mapping` is unwritable;
- no modulo, and no tuple constructor — so a multi-dimensional launch is unexpressible despite
  §6.6-0006 promising tuple-valued results;
- a non-negative-integer value rule that **contradicts** the signed strides §6.6-0004 requires.

All five fields are unwritable **at every rank, including the spec's own §2.5 rank-1 `add`
example**. Additionally: `workgroup_sizing` is mandated but **has no defining clause anywhere**;
§6.6-0003's "grid size" is undefined and backend-divergent; and there is **no dynamic
shared-memory field**, which three independent closed lists each forbid adding.

**For Baracuda this is the whole ballgame** — a CUDA kernel's grid/block geometry is exactly what
Dispatch exists to carry. Note also the fusion-barrier objection in [`PRIOR-ART.md`](PRIOR-ART.md)
§5.1: XLA deliberately rejected declaring launch geometry. KISS should decide whether it has an
answer before investing more here.

### 1.5 `structure_key` cannot be derived — the wedge is the least-tested thing in the repo

This is the one claim [`PRIOR-ART.md`](PRIOR-ART.md) concludes is genuinely novel, and:

- **No test in the repo derives a `structure_key`.** All 90 Classify clauses name tests that do
  not exist; the shipped code is a *codec* (token ↔ struct), not the §6.6-0011 derivation.
  §6.6-0011 requires the derivation be "canonical and deterministic" — nothing checks it.
- **No document owns the `op-name → cell-op-category` mapping**, so token field 1 is undetermined.
- **Broadcast has three legal spellings and no canonicalization** — the same `add` derives
  `co/00/v4`, `co/01/v4`, or `br/01/v1`. Appendix A itself uses two conflicting conventions.
- **The keepdim reduce contradiction is real.** OPS-6.11-0008 requires a reduce's retained axis be
  stride-0; CLASSIFY-6.6-0009's broadcast mask is a function of exactly that stride; Classify pins
  no such requirement and §6.9-0001 forbids it from reading Ops. **The same reduction derives two
  different `structure_key`s.**
- §6.5-0009(c) has no stride precondition (issue #14): a transposed operand derives `v4`/`v8` with
  no contiguous run to load.
- §6.5-0010's max-extent frame conflates M with K on a contraction, diverging from Classify's own
  GEMM golden vector.

**Fuel's "single-classifier division" (its comment on issue #11) is the way out and should be
ratified.** If the *provider* classifies raw operands and produces the `structure_key`, two
classifiers cannot drift — the entire class of bugs above stops being a wire problem. This costs
one clause and deletes a category of divergence.

---

## 2. What each project can do now

The ordering matters: **1 is worth more than 2–4 combined**, because it is the only claim that
survives prior art.

### Everyone — first
**Ratify single-classifier division** (§1.5). It is one decision, it deletes a bug class, and
Fuel has already built it.

### Baracuda (provider)
1. **Publish a build matrix as `structure_key` tokens.** This is the wedge
   ([`PRIOR-ART.md`](PRIOR-ART.md) §3): a cache key cannot express a build matrix, and Baracuda's
   `GPU_TARGETS × DTYPES × tile-config` registry is exactly the thing no incumbent can publish.
   **Emitting that list as tokens is the single most valuable artifact in this whole effort.**
2. **Derive a `structure_key` from a real kernel** and contribute it as a golden vector. First
   real test of Classify §6.6.
3. Do **not** invest in Dispatch until §1.4 is resolved — you would be implementing an ungrammar.

### Fuel (consumer)
1. **Derive a `structure_key` independently from the same kernel** and check it byte-matches
   Baracuda's. **That single comparison is the freeze gate's condition 1** — two dissimilar
   implementations interoperating on a golden vector — for the one clause that matters. If they
   diverge, that divergence is worth more than any spec text.
2. Report every ambiguity hit while doing it, as the adversarial-outsider record (umbrella §5.3).
3. Bring the `JitRequest` op-DAG payload to issue #11.

### Unpopped (fresh eyes)
**Be the foreign reader.** Umbrella §5.3 condition 2 requires a reader written *outside* the
reference language reproducing the exact bytes from the document alone. Unpopped has never seen
this spec — that is an asset that decays fast. Spend it on: parse the 56-byte envelope from
`announce.md` alone, without reading `conformance/`. Log every question. **The questions are the
deliverable**, not the parser.

### The repo
1. Fix the §6.8 ULP ceilings ([`PRIOR-ART.md`](PRIOR-ART.md) §5.2) — five rows reject conformant
   hardware.
2. Decide §1.2 (CRSP vs PRSP). One sentence.
3. Add §6.6 stream binding (§1.1) + its two vectors.
4. Stand up the `artifact_format_tag` registry (§1.3) — a file with four rows unblocks PRSP.
5. Cite clause IDs in the 95 harness tests that don't — cheapest coverage in the repo
   (`conformance/README.md` Phase 5).

---

## 3. The honest summary

Everything above is a decision or a byte sequence. **None of it requires hardware, a second
vendor, or a foundation.** The protocol tier has never had a byte cross a process boundary, and
that — not the spec's length — is why 38 blockers sat undetected: *the only way to find them was
to try, and no one had tried.*

Three implementations trying at once is exactly the missing ingredient. The first byte-identical
`structure_key` derived independently by Baracuda and Fuel is worth more to this standard than
the next 100KB of clauses.
