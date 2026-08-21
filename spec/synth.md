# KISS-Synth / Provision — The Kernel-Provision Protocol

**Sub-standard ID:** KISS-SYNTH
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification* (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ is normative. Normative
> clauses use RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-SYNTH-<section>-<nnnn>`, and each MUST/SHALL maps 1:1 to at least one
> named KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Synth / Provision |
| Sub-standard ID | KISS-SYNTH |
| Tier | **Protocol** (the kernel-provision protocol; sits above the KISS-Announce discovery seam it generalizes, above the KISS-Contract format it returns, and above the KISS-Ops computation vocabulary it imports) |
| Maturity stage | **Draft** (first-draft proposal; the provision protocol is NOT yet frozen — the freeze gate of §8 is unmet) |
| Editor of record | **Proposed, pending ratification** — a provider/JIT reference-impl project holds the pen and requests comment from interested cosignatories; the ratified governance record does not yet finalize an editor for KISS-Synth. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | a kernel-provision/JIT reference crate (`baracuda-kernelgen` provision path, project/crate name given in Appendix A as non-normative provenance); this crate is *a* conformant implementation with no privilege. |
| DAG position | **Protocol tier.** Depends (structurally) on KISS-Announce, KISS-Contract, and KISS-Ops; tested downstream by KISS-Conform. Depends on **none** of KISS-Consume or KISS-Emit. Not a root. |
| Upstream edges | KISS-Announce (**STRUCTURAL** — the provision request reuses the KISS-Announce contract-query request frame `CYRQ` verbatim and the typed decline reuses the Announce decline frame `CDEC` and the pinned decline-code enum verbatim; the `{artifact, contract}` provision success is carried in KISS-Synth's own provision-success frame `PRSP` (the artifact's wire home), which encloses a length-delimited contract payload byte-identical to what the Announce contract-response frame `CRSP` carries; provision IS the KISS-Announce contract-query generalized so the response may include a freshly built artifact); KISS-Contract (**STRUCTURAL** — provision returns `{artifact, contract}`, and the provided kernel's fidelity is exactly the contract Guarantees); KISS-Ops (**STRUCTURAL** — the determinism/fidelity enum and the MathPrecision attribute are imported verbatim, and the resolved Semantics op DAG down to the primitive floor is the verification oracle) |
| Downstream edges | KISS-Conform (test dependency — depends on and tests KISS-Synth; owns the fuzz/negative-vector modality that exercises the never-panic decline taxonomy) |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |
| Maturity | Draft proposal |

> **Edge-label note (informative).** All three KISS-Synth upstream edges are
> **STRUCTURAL**, consistent with the umbrella §2.2 edge table, which lists
> **KISS-Announce → KISS-Synth/Provision**, **KISS-Contract → KISS-Synth/Provision**,
> and **KISS-Ops → KISS-Synth/Provision** each as STRUCTURAL. KISS-Synth parses and
> reuses the internal structure of the KISS-Announce contract-query request/decline
> frames (it does not merely carry an opaque Announce token — it IS the Announce
> contract-query, generalized), imports the KISS-Ops determinism/fidelity enum and
> MathPrecision attribute by their exact spelling, and returns a KISS-Contract document
> as the second half of its response. The one wire artifact KISS-Synth contributes of
> its own is the provision-success frame `PRSP` — the artifact's wire home, which has
> no upstream owner — and even that encloses a contract payload byte-identical to the
> KISS-Announce `CRSP` payload. The KISS-Announce §0 front-matter labels its own
> **downstream** view of this edge OPAQUE (Announce carries the provision hand-off
> without depending on Synth's internals); the umbrella §2.2 edge table is
> authoritative for the **dependent-side** label used here, and from KISS-Synth's
> side the dependency on the Announce contract-query wire structure is STRUCTURAL.
> KISS-Synth has **no** edge to or from KISS-Consume or KISS-Emit: those two are DAG
> siblings of one another and of Synth, all three prerequisite-closed over the same
> structural parents (Ops, Classify, Contract) and none depending on the others.

---

## 1. Purpose & Scope

KISS-Synth / Provision owns the **kernel-provision protocol** of the suite: the
protocol-tier node across which a **consumer** asks a **provider** for a kernel **by
identity** and receives `{artifact, contract}` — the built (or cached) callable
kernel plus its full KISS-Contract — or a **typed decline**. Provision is the
general form of just-in-time synthesis: on a cache miss the provider **builds** the
kernel, and the already-exists case and the build-on-miss case are the **same**
request and the **same** response. KISS-Synth defines four things and nothing else:

1. **Provision by identity** — the request names the wanted kernel with a
   `structure_key` (the KISS-Classify specialization-cell key), optionally pinned to
   a specific `revision_hash`. It reuses the KISS-Announce contract-query request
   frame verbatim and adds no new request shape; provision is that request,
   generalized so the response may include a freshly built artifact.

2. **The `{artifact, contract}` response** — the built (or cached) callable kernel
   plus its full, self-delimiting KISS-Contract document, transported in the
   KISS-Synth provision-success frame, which carries the artifact and encloses the
   length-delimited contract bytes (byte-identical to what the KISS-Announce contract
   response frame would carry) — or a typed decline. **Every returned kernel carries
   its contract**; per-kernel capability lives in the contract, never in the
   availability announce.

3. **Build-on-miss = JIT** — on a miss for a known, provisionable cell, a provider
   that advertises build-on-miss builds the artifact and returns it on the same
   request/response; just-in-time synthesis is the build-on-miss branch of the one
   protocol, not a separate protocol.

4. **The never-panic obligation on the provision path** — every failure (unknown
   cell, build failure, unsupported version, missing revision, malformed request,
   query-not-supported) is a **typed decline**, never a panic, abort, or crash, and
   is produced within a bounded response, over a complete, testable decline taxonomy.

The three staged levels are **provider handshake** (deferred to KISS-Announce),
**kernel availability by identity only** (deferred to KISS-Announce), and
**contract-query / provision-on-miss** (this sub-standard's own act, level 3).

**KISS-Synth / Provision is NOT:** the discovery/handshake envelope, the
version-negotiation algorithm, the capability bitset, or the identity-only
availability list (those are KISS-Announce, whose frames and decline enum this
sub-standard reuses); the per-kernel *contract* format (that is KISS-Contract, which
provision returns and never re-defines); the *data* vocabulary (`structure_key`,
operand descriptors, `target_capability` are KISS-Classify, carried opaquely through
the Announce contract-query); the *computation* vocabulary, per-op semantics, the
determinism/fidelity enum, or the MathPrecision attribute (those are KISS-Ops,
imported verbatim, never re-forked); the recognition/lift direction (KISS-Consume) or
the generation/lower direction (KISS-Emit) — a JIT builder is a KISS-Emit emitter
*reached through* provision, and a provider's own kernel-decomposer is a KISS-Consume
lifter, but neither is defined here; a kernel implementation, a source language, a
compiler IR's internals, or a transport/session layer (TCP, IPC, shared memory, and
delivery ordering/reliability are out of scope — Synth specifies *bytes and message
framing*, not the pipe). Anything not enumerated as in-scope above is out of scope
for KISS-Synth (scope creep by silence is a named trap; silence is not inclusion).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — one request, hit and miss unified

A consumer that needs a specialized kernel has exactly one question: *"give me the
kernel for this cell."* It asks the provider by **identity** — a `structure_key`
naming the layout/dtype/target specialization cell, optionally pinned to a
`revision_hash` naming which build it wants — and gets back one of two things: the
pair `{artifact, contract}`, or a typed decline saying why not.

The key simplification KISS-Synth makes is that **the already-exists case and the
build-on-miss case are the same request and the same response**. Whether the provider
had the kernel sitting in a cache or had to compile it on the spot, the consumer sends
the same bytes and receives the same shape. Just-in-time synthesis is not a second
protocol bolted on; it is the *build-on-miss branch* of the one provision
request/response. A provider that builds on miss is doing JIT; a provider that only
serves what it already holds is the degenerate case of the same protocol.

This is why KISS-Synth is the **generalization of KISS-Announce's contract-query**.
The KISS-Announce discovery seam already defines a contract-query: a consumer asks for
a `structure_key`'s contract, and the provider returns it (the already-exists branch)
or a typed decline. KISS-Synth reuses that request verbatim and generalizes the
response so that, on a miss, the provider may **build** the artifact before answering.
Provision adds no new request shape — it *is* the Announce contract-query, with
build-on-miss added and an artifact returned alongside the contract.

### 2.2 The three levels — handshake, availability, provision

Provision rests on three staged acts, from coarse to fine. The first two are owned by
KISS-Announce and reused here; only the third is KISS-Synth's own:

- **Level 1 — Provider handshake (KISS-Announce SeamHello envelope).**
  Provider-level capability negotiation *only*: which KISS profiles/versions each
  side speaks, whether the provider answers contract-query (`CONTRACT_QUERY`, FEAT
  bit 33), and whether it builds on miss (`PROVISION_ON_REQUEST`, FEAT bit 32 — the
  provider does JIT). No per-kernel capability appears here. An empty profile
  intersection is an Announce-owned handshake failure, never a panic (§6.7-0002); it
  is not a KISS-Synth provision decline.

- **Level 2 — Availability by identity.** The provider names the kernels it can serve
  by **identity only** — the `(structure_key, revision_hash)` pair — so a consumer
  distinguishes a cache **hit** (both match byte-for-byte) from a **miss** without
  transferring anything heavy. The per-kernel announce carries **no** capability;
  that is the contract's single-source-of-truth job.

- **Level 3 — Contract-query / provision-on-miss.** On a miss (or to fetch a known
  cell's contract) the consumer requests by `structure_key`, optionally
  revision-pinned; the provider returns `{artifact, contract}`, **building it on miss**
  if it does not yet exist. Build-on-miss is exactly just-in-time synthesis.

### 2.3 Every returned kernel carries its contract

A kernel a consumer cannot learn *how to call* or *what it computes* is unusable. So
the provision response is never a bare `dlsym` symbol: it is the pair
`{artifact, contract}`. The **artifact** is the built, callable kernel binary/object;
the **contract** is the full seven-section KISS-Contract document that describes the
artifact's ABI (Interface + Dispatch), its op identity and Semantics, and its numeric
Guarantees. The contract is fetched or produced **on the same request** — it is never
pushed eagerly, per-kernel, into the availability announce (that would duplicate facts
that already have a home in the contract and would drift). Per-kernel capability is the
contract's job; the availability list is a de-dup index of identities only.

On the wire a provision success is the KISS-Synth provision-success frame (tag
`PRSP`): the echoed `(structure_key, revision_hash)` identity block (always carrying
the assigned `revision_hash`), a 4-byte `artifact_format_tag`, a `u64` artifact
byte-length, the artifact bytes, a `u32` contract byte-length, then that many bytes of
the self-delimiting contract document. The contract bytes are byte-identical to the
payload the KISS-Announce contract response frame (`CRSP`) would carry, and the
enclosing frame is opaque to the contract's internals, so the contract document's own
inner framing (its magic, header line, inner length/checksum, and section headings)
must **fail loudly on its own terms**, independently of the outer length prefix. A
bare `CRSP` frame — a contract with no enclosing `PRSP` and no artifact — is the
KISS-Announce already-exists contract-query result, **not** a KISS-Synth provision
success; the leading four bytes (`PRSP` vs `CRSP`) are the byte condition that tells
the two apart, and a provision success is therefore observable on the wire by the
presence of the artifact block.

### 2.4 The never-panic obligation is Synth's to own

KISS-Synth **owns the never-panic obligation on the provision path** as a
fuzz-testable, identified clause. Every failure — an unknown cell, a build failure, an
unsupported version, a missing revision, a malformed request, or a request to a
provider that does not answer contract-query — is a **typed decline** per
KISS-SYNTH-6.6-0001, never a panic, abort, or crash, produced within an
implementation-declared latency bound and without reading past the request's declared
framing. The decline codes are the pinned KISS-Announce decline enum, reused verbatim:

| Value | Name | When |
|---|---|---|
| `0x00000001` | `UNKNOWN_STRUCTURE_KEY` | no such cell, and none can be provisioned |
| `0x00000002` | `CANNOT_PROVISION` | cell known but build-on-miss unavailable or failed |
| `0x00000003` | `MALFORMED_REQUEST` | request framing invalid |
| `0x00000004` | `QUERY_NOT_SUPPORTED` | provider does not advertise contract-query (FEAT bit 33) |
| `0x00000005` | `VERSION_UNSUPPORTED` | negotiated version retired below floor on the provision path |
| `0x00000006` | `UNKNOWN_REVISION` | `structure_key` known but requested `revision_hash` not held |

plus the experimental range `[0x40000000, 0x80000000)` and the vendor range
`[0x80000000, 2^32)`. A consumer receiving an **unrecognized** decline_code treats it
as a generic decline and still never panics.

### 2.5 Fidelity is the contract Guarantees — imported, never re-forked

The fidelity of a provided kernel is **not** a separate statement in the provision
frame: it is exactly the accompanying contract's **Guarantees** section. The single
canonical **determinism/fidelity enum** `{exact-byte, ULP/tolerance,
order-invariant/nondeterministic}` (that literal spelling, verbatim) is **owned by
KISS-Ops** (`KISS-OPS-6.0-0001`) and imported by KISS-Synth, never re-forked. So is the
orthogonal **MathPrecision attribute** `{bit-stable, reduced-mantissa-permitted}`
(KISS-Ops §6.17). The determinism class of the provided kernel selects which
KISS-Conform comparator applies when a consumer verifies it: `exact-byte` → a byte
comparator; `ULP/tolerance` → the declared-ULP comparator; `order-invariant/
nondeterministic` → a comparator at the tolerance the contract Guarantees declare for
that class (the float sum / prod / matmul / atomic-add family). KISS-Synth asserts no
numeric behavior of its own and re-defines no op.

### 2.6 A worked provision — a strided binary `add` on `f32`

A consumer wants a binary elementwise `add` over `f32`, target `cuda:sm89`, on a
strided cell.

1. **Handshake (level 1).** The two sides exchange the KISS-Announce SeamHello
   envelope; the consumer sees the provider advertises `PROVISION_ON_REQUEST` (FEAT
   bit 32) and `CONTRACT_QUERY` (FEAT bit 33). Profile `1` is mutual.
2. **Availability (level 2).** The provider's availability list does not contain the
   consumer's `structure_key` (op-category `bin`, three `f32` operands, strided,
   `cuda:sm89`) — a **miss**.
3. **Provision (level 3).** The consumer sends the contract-query request (tag
   `CYRQ`): the `u32` `structure_key` length, the `structure_key` bytes, and a
   `revision_present = 0` flag. On the miss, the provider **builds** the kernel (JIT),
   assigns it a 32-byte `revision_hash`, and answers with `{artifact, contract}` in a
   `PRSP` frame: the echoed `(structure_key, revision_hash)` identity block (with
   `revision_present = 1` and the freshly assigned `revision_hash`), a 4-byte
   `artifact_format_tag`, the `u64`-framed artifact bytes, then the `u32`-framed opaque
   contract document whose Semantics is a one-node `add` DAG, whose Interface is the
   strided positional signature, and whose Guarantees declare determinism class
   `exact-byte`, MathPrecision `bit-stable`.

Had the provider already held the kernel, the consumer would have sent the identical
`CYRQ` request and received the identical `PRSP` response (echoing the same held
`revision_hash`) — the already-exists and build-on-miss cases are the same
request/response.

### 2.7 A worked decline — an unbuildable cell

The consumer requests a `structure_key` for a target the provider cannot compile for
(no toolchain for that `target_capability`). The provider does not panic and does not
return a mismatched or empty contract; it returns the `CDEC` decline frame with the
echoed identity block and `decline_code = CANNOT_PROVISION` (`0x00000002`) — the cell
was recognized but build-on-miss failed. Had the cell been entirely unknown and not
provisionable, the code would have been `UNKNOWN_STRUCTURE_KEY` (`0x00000001`); had the
consumer pinned a `revision_hash` the provider does not hold and cannot build, it would
have been `UNKNOWN_REVISION` (`0x00000006`).

### 2.8 How the three protocol seams meet at the contract (informative)

All three protocol-tier standards — KISS-Synth, KISS-Consume, KISS-Emit — are
prerequisite-closed over the **same** structural parents (KISS-Ops, KISS-Classify,
KISS-Contract) and re-define none of them. KISS-Synth additionally depends
structurally on KISS-Announce and **is its downstream generalization**: the
contract-query (handshake → availability-by-identity → query) is exactly the provision
protocol, with build-on-miss added. KISS-Consume and KISS-Emit do **not** depend on
KISS-Announce, and KISS-Synth depends on neither of them.

KISS-Emit lowers `(OpDef + structure_key) → artifact-described-by-a-contract`;
KISS-Consume lifts a kernel/source region → the contract's Semantics op DAG (as far as
it goes) + residue. They are **inverse directions** that share a two-tier round-trip but
**neither depends on the other** — they are DAG siblings. A JIT builder reached through
KISS-Synth provision is a KISS-Emit emitter; a provider's own kernel-decomposer is a
KISS-Consume lifter. The three seams meet at the contract: Synth returns it, Consume
produces its Semantics field + residue, Emit's output is described by it.

The emit/consume round-trip itself is owned normatively by KISS-Emit and KISS-Consume
(KISS-Synth states no round-trip clause of its own); it is reproduced here **verbatim
and informatively** so the reader of the provision path sees the guarantee that keeps a
JIT-built artifact honest. This two-tier statement is normatively identical in KISS-Emit
and KISS-Consume:

> **TIER 1 — structural round-trip:** emit-then-lift (or lift-then-emit) reproduces the
> SAME KISS-Ops op DAG under STRUCTURAL / op-DAG EQUALITY, checked over a DECLARED
> SUBSET of ops (the subset each side declares it round-trips, not the whole op set).
> This is the always-claimable tier and the one interop actually rests on: two parties
> agree on what an OpDef MEANS structurally. It is language-independent because it
> compares KISS-Ops op-DAG structure, not bytes.

> **TIER 2 — numeric round-trip:** bit-identity of the computed result is claimed ONLY
> SAME-LANGUAGE, ON-DEVICE — same source language, same target device — and only for the
> exact-byte determinism class; ULP/tolerance and order-invariant/nondeterministic ops
> are never claimed bit-identical. This tier is a strict, narrowly-scoped add-on to
> tier 1, never a substitute for it.
>
> How such an op's result is *evaluated* is KISS-Conform's, not this sub-standard's
> and not KISS-Emit's — see KISS-EMIT §6.7-0002 on the verification mandate excised
> from that clause, and on what its removal leaves unnamed.

> Numeric identity is NEVER claimed across languages. Cross-language round-trip is
> TIER 1 (structural) ONLY — Slang tanh is not bit-identical to CUDA tanh, and
> overclaiming cross-language numeric identity is a named trap. Across languages the
> guarantee stops at structural op-DAG equality over the declared subset. This two-tier
> statement is NORMATIVELY IDENTICAL in KISS-Emit and KISS-Consume (same wording, same
> clause intent) so the two directions cannot drift; both import the KISS-Ops
> determinism/fidelity enum to decide which tier applies per op.

### 2.9 Terms are joined, not restated

KISS-Synth references the KISS-Classify `structure_key` and `target_capability`, the
KISS-Contract seven-section document and its `revision_hash` Identity field, the
KISS-Announce handshake envelope / availability list / contract-query frames / decline
enum, and the KISS-Ops determinism/fidelity enum and MathPrecision attribute — all by
name/structure. It re-defines none of them and defines no op meaning, no data noun, no
contract section, and no wire envelope of its own **except the provision-success frame
that is the artifact's wire home** (the artifact has no owner upstream): Synth carries
the identities and the built artifact, the upstream sub-standards mean them.

---

## 3. Terms & Definitions (glossary)

- **Provision** — the act of a consumer asking a provider for a kernel by identity and
  receiving `{artifact, contract}` or a typed decline; the generalization of just-in-time
  synthesis, in which build-on-miss is one branch of the single request/response.
- **Provider** — a party that holds or can build kernels and answers handshake,
  availability, and contract-query / provision requests. A provider that builds-on-miss
  is doing JIT. (Umbrella-generic role; never a project name in normative text.)
- **Consumer** — a graph, runtime, or compiler that wants a kernel: it handshakes,
  distinguishes a cache hit from a miss by identity, and requests `{artifact, contract}`
  by `structure_key`. A consumer verifies a received kernel against its contract where
  practical (the KISS-Conform SHOULD of §6.4-0004). (Umbrella-generic role.)
- **artifact** — the built, callable kernel binary/object returned as the **first**
  element of the provision response `{artifact, contract}` and framed in the
  provision-success frame's artifact block. Its ABI is described by the accompanying
  contract's Interface + Dispatch sections; its `revision_hash` identifies which build it
  is. Owned by KISS-Synth/Provision (its wire home).
- **provision-success frame (`PRSP`)** — KISS-Synth's own wire frame (tag `PRSP`, wire
  bytes `50 52 53 50`) carrying a provision success: the echoed
  `(structure_key, revision_hash)` identity block, a 4-byte `artifact_format_tag`, a
  `u64` artifact byte-length + artifact bytes, then a `u32` contract byte-length +
  contract bytes. The contract bytes are byte-identical to the KISS-Announce `CRSP`
  payload. Owned by KISS-Synth/Provision.
- **artifact_format_tag** — a 4-byte tag naming the artifact's binary format/target
  family (e.g. a PTX/cubin/SPIR-V/object family), matched byte-for-byte and otherwise
  opaque; registry-assigned. Owned by KISS-Synth/Provision.
- **contract** — the KISS-Contract seven-section, self-delimiting document
  `{Identity, Semantics, Interface, Dispatch, Capabilities, Guarantees, Provenance}`
  that travels with every provided kernel. Owned by KISS-Contract; Synth **returns** it,
  never re-defines it. Every provided kernel carries one (KISS-SYNTH-6.2-0002).
- **structure_key / specialization cell** — the KISS-Classify admissibility predicate
  over one layout/dtype/target specialization cell: a coarse op-category tag +
  canonically-ordered operand descriptors + `target_capability` + role hints,
  **extent-free** (keyed by size classes, not literal extents), matched byte-for-byte.
  Owned by KISS-Classify; used here as the provision request key, carried opaquely
  through the Announce contract-query.
- **revision_hash** — an opaque, provider-assigned build identifier of one revision
  behind a `structure_key`; **32 bytes** on the KISS-Announce wire; compared only for
  byte-for-byte equality (no hash algorithm implied). `(structure_key, revision_hash)`
  is the full availability identity; a full byte match on both is a cache **hit**,
  anything else a **miss**. Owned by KISS-Announce (wire) / KISS-Contract (Identity).
- **op_identity** — the identity of the op the Semantics DAG root computes: **(a)** a
  full KISS-Grammar advertisable-op tag re-based on a KISS-Ops op name, OR **(b)** the
  bare KISS-Ops op name of the root (no Grammar tag). Owned by KISS-Contract Identity;
  distinct from `structure_key`. KISS-Grammar is not required for every kernel — form
  (b) is the hand-written common case.
- **target_capability** — the KISS-Classify namespaced `<namespace>:<capability-set>`
  all-hardware descriptor of the compilation target, matched byte-exact on the full
  string. Owned by KISS-Classify; carried in the contract Identity.
- **op DAG** — a hierarchical, mixed-abstraction-level directed acyclic graph over
  KISS-Ops op names, node schema `Op{op_name, op_attrs, child_edges} | Bind(positional_index)`,
  recursively resolvable to the primitive floor. Owned by KISS-Ops; carried by the
  KISS-Contract Semantics field. Synth's **resolved** DAG down to the floor is the
  verification oracle.
- **primitive floor** — the mandatory-core KISS-Ops op set every conforming consumer
  understands, at which every decomposition chain terminates (acyclic +
  strictly-decreasing level = the termination guarantee). Owned by KISS-Ops (§6.3).
  Synth resolves the Semantics DAG down to it to produce the oracle.
- **OpAttrs channel** — the per-op compile-time attribute record (reduce monoid/axes,
  gather `oob_policy`, pool geometry, permutation, …) with a canonical
  default-resolved little-endian byte encoding. Owned by KISS-Ops (§6.19); carried per
  op node and embedded by Contract/Grammar as opaque, byte-comparable bytes.
- **determinism/fidelity enum** — the single canonical enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}`, that literal spelling verbatim.
  **Owned by KISS-Ops** (`KISS-OPS-6.0-0001`), **imported** verbatim by KISS-Synth,
  never re-forked. Selects the KISS-Conform comparator.
- **MathPrecision attribute** — the compute-fidelity enum `{bit-stable,
  reduced-mantissa-permitted}`. Owned by KISS-Ops (§6.17), imported verbatim.
  Orthogonal to the determinism class and NOT a dtype. Surfaced in the contract
  Guarantees.
- **lift fraction / residue** — the lift fraction is the portion of a kernel lifted
  into the op DAG; the un-liftable complement is the recorded residue (a KISS-Consume
  refusal category). Contract COMPLETENESS tracks the lift fraction. Shared by
  KISS-Consume (produces it) and KISS-Contract (records it); Synth carries whatever the
  returned contract records, never fakes it.
- **contract-query** — the KISS-Announce request/response (tags `CYRQ` / `CRSP` /
  `CDEC`) by which a consumer fetches a contract for a `structure_key`; the
  already-exists branch of provision, reused here verbatim and generalized to
  build-on-miss.
- **build-on-miss (JIT)** — the branch of provision in which the requested kernel does
  not yet exist and the provider builds it before answering; just-in-time synthesis is
  exactly this branch, not a separate protocol.
- **typed decline** — a structured refusal returned in lieu of a result (a
  distinguished error value/enumerant, or an equivalent out-of-band error return),
  produced within a bounded response; never a panic, abort, crash, hang, or
  out-of-bounds read. The unifying failure currency of the provision path, drawn from
  the pinned KISS-Announce decline enum.
- **PROVISION_ON_REQUEST / CONTRACT_QUERY** — the two provider-level KISS-Announce FEAT
  capability bits (bit 32: builds a kernel on miss; bit 33: answers contract-query)
  that gate whether a provider participates in provision; owned by KISS-Announce §7.2,
  used here by number, never re-numbered.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point semantics; referenced transitively through
  KISS-Ops and KISS-Contract (KISS-Synth defines no numeric behavior of its own).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword
  convention, the normative/informative split, the clause-ID scheme and 1:1 test
  mapping, value pinning as bits/IEEE-754 in wire order, the ban on unquantified
  adjectives, the two version axes, the ≥2-dissimilar-implementations-plus-foreign-
  reader freeze gate, the capability/profile/extension model, governance, licensing,
  and patent posture. **Stated once in the umbrella; referenced here; never restated.**
  This sub-standard's §5 points at umbrella §3 for conventions.
- **KISS-Announce** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: KISS-Synth **is** the KISS-Announce contract-query, generalized so the
  response may include a freshly built artifact. The provision request reuses the
  Announce contract-query request frame verbatim (tag `CYRQ`, `KISS-ANNOUNCE-6.4-0001`);
  the `{artifact, contract}` provision success is carried in the KISS-Synth
  provision-success frame (tag `PRSP`), which is KISS-Synth's own wire contribution —
  the artifact's wire home — and encloses a length-delimited contract payload
  byte-identical to what the Announce contract response frame carries (`CRSP`,
  `KISS-ANNOUNCE-6.4-0004`); a bare `CRSP` frame with no enclosing `PRSP` is the
  Announce contract-only result, not a provision success. A typed decline reuses the
  Announce decline frame (tag `CDEC`, `KISS-ANNOUNCE-6.4-0007`) and the pinned
  decline-code enum (`KISS-ANNOUNCE-6.4-0009`); the handshake envelope,
  version-negotiation algorithm, capability bitset (FEAT bits 32 `PROVISION_ON_REQUEST`
  and 33 `CONTRACT_QUERY`, §7.2), and identity-only availability list
  (`KISS-ANNOUNCE-6.3`) are deferred to KISS-Announce and reused, never re-defined. The
  `structure_key` and the contract payload travel through Announce opaquely; from
  Synth's dependent side the reuse of the Announce contract-query wire structure is
  STRUCTURAL (umbrella §2.2). Re-defined nowhere here.
- **KISS-Contract** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: provision returns `{artifact, contract}`; the contract is the
  seven-section KISS-Contract document (`KISS-CONTRACT-6.2-0001`); every returned
  kernel carries one (`KISS-CONTRACT-6.2-0001` requires it of every kernel, and this
  sub-standard requires it of every **provided** kernel). The provided kernel's fidelity
  is exactly the contract Guarantees section (`KISS-CONTRACT-6.8`), which surfaces the
  imported determinism class and MathPrecision attribute; the returned artifact's ABI is
  described by the contract Interface + Dispatch sections (`KISS-CONTRACT-6.5` / `6.6`);
  the contract's self-delimiting inner framing fails loudly independently of the outer
  length prefix (`KISS-CONTRACT-6.1-0005`); the required-core content-validity clauses —
  a non-empty Semantics op DAG (`KISS-CONTRACT-6.4`) and a non-empty Guarantees section
  (`KISS-CONTRACT-6.8`), plus the required-core sections (`KISS-CONTRACT-6.2-0001`) —
  define what a **valid** returned contract is. Used here by structure; re-defined
  nowhere.
- **KISS-Ops** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: the single canonical determinism/fidelity enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` (`KISS-OPS-6.0-0001`) and the
  MathPrecision attribute `{bit-stable, reduced-mantissa-permitted}` (KISS-Ops §6.17)
  are **imported verbatim** and never re-forked; the resolved Semantics op DAG down to
  the KISS-Ops primitive floor (§6.3) is the verification oracle under the op's declared
  determinism class. KISS-Synth defines no op meaning and no determinism vocabulary of
  its own.
- **KISS-Classify** (by version) — **not a direct dependency edge of KISS-Synth.** The
  `structure_key` and `target_capability` are KISS-Classify vocabulary, but KISS-Synth
  carries them **opaquely** through the KISS-Announce contract-query (Announce's own
  Classify edge is OPAQUE); Synth never parses their internals and claiming Synth does
  not force a co-claim of KISS-Classify beyond agreement on the meaning of the token.
- **KISS-Consume / KISS-Emit** (by version) — **NOT dependencies.** KISS-Synth depends
  on neither. A provider's kernel-decomposer is a KISS-Consume lifter and a JIT builder
  reached through provision is a KISS-Emit emitter, but those are separate sub-standards
  reached *through* the contract, not edges of this one. The emit/consume two-tier
  round-trip is owned normatively by KISS-Emit and KISS-Consume and is reproduced in §2.8
  informatively only.
- **KISS-Conform** (by version) — depends on and tests KISS-Synth; owns the
  fuzz/negative-vector modality that exercises the never-panic decline taxonomy (§6.6),
  the oracle-differential harness that resolves a provided kernel's Semantics DAG to the
  primitive floor and compares under the op's declared determinism class, the
  content-validity check of a returned contract (§6.2-0008a), and the
  consumer-verification SHOULD (§6.4-0004).

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase
keywords `MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance and
consumer-behavior guidance and never state a structural or wire requirement. Every
atomic requirement carries a stable, append-only ID `KISS-SYNTH-<section>-<nnnn>` (a
lettered suffix such as `-0002a` denotes a later-allocated atomic split of an existing
clause and is itself append-only), allocated by the editor of record, never reused
after retirement, and mapped 1:1 to ≥1 named KISS-Conform test. Each clause states
exactly one `MUST` / `MUST NOT` / `SHALL`; compound requirements are split into atomic
clauses (umbrella §3.3). Values are pinned as tokens/frames/bytes spelled exactly as the
upstream KISS-Announce frames, KISS-Contract document, and KISS-Ops enums pin them, never
as one source language's surface spelling. Unquantified adjectives ("well-formed",
"well-framed", "reasonable", "neutral", "valid") are banned from normative text; where a
normative clause needs such a notion it pins it to an observable framing fact or a named
clause. Every clause declares its determinism/fidelity class so KISS-Conform selects the
correct comparator. See umbrella §3 for the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

- **KISS-SYNTH-6.0-0001** — Every structural obligation in §6–§8 (the provision request
  and response framing, the reuse of the KISS-Announce `CYRQ` / `CDEC` frames and the
  KISS-Synth `PRSP` frame, the decline-code values, every field spelling, and every
  token spelling) is determinism-class **exact byte compare**; KISS-Conform MUST evaluate
  each such clause with a byte-exact comparator. *Test:*
  `test_synth_determinism_class_exact_byte`.
- **KISS-SYNTH-6.0-0001a** — KISS-Conform MUST NOT apply a tolerance or order-invariant
  comparison to any structural obligation in §6–§8. *Test:*
  `test_synth_determinism_class_no_tolerance`.
- **KISS-SYNTH-6.0-0001b** — The numeric determinism class of any op a provided kernel
  computes, and of the numeric guarantees the returned contract carries, is **owned by
  KISS-Ops** (the single canonical enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}`, `KISS-OPS-6.0-0001`) and MUST NOT be re-forked in
  KISS-Synth. *Test:* `test_synth_determinism_class_not_reforked`.

### 6.1 Provision request — by identity, reusing the Announce contract-query

- **KISS-SYNTH-6.1-0001** — A provision request MUST identify the wanted kernel by
  **identity only**: a `structure_key` (the KISS-Classify specialization-cell key),
  optionally pinned to a specific `revision_hash`. *Test:*
  `test_synth_request_is_identity_only`.
- **KISS-SYNTH-6.1-0001a** — A provision request MUST NOT carry per-kernel capability,
  usage/ABI, dispatch, guarantee, or semantics fields (those are the returned contract's
  job). *Test:* `test_synth_request_no_capability_fields`.
- **KISS-SYNTH-6.1-0002** — A provision request MUST reuse the KISS-Announce
  contract-query request frame verbatim (tag `CYRQ`, wire bytes `43 59 52 51`,
  `KISS-ANNOUNCE-6.4-0001`): the 4-byte tag; a little-endian `u32` `structure_key`
  byte-length in `[1, 4096]`; the `structure_key` bytes; a 1-byte `revision_present` flag
  (`0` or `1`); and, only when `revision_present == 1`, the 32-byte `revision_hash`.
  *Test:* `test_synth_request_reuses_cyrq`.
- **KISS-SYNTH-6.1-0002a** — A provision request MUST NOT introduce a new request shape;
  provision IS the Announce contract-query, generalized so the response MAY include a
  freshly built artifact. *Test:* `test_synth_request_no_new_shape`.
- **KISS-SYNTH-6.1-0003** — A provision request MUST carry the `structure_key` as the
  opaque, length-delimited KISS-Classify token. *Test:*
  `test_synth_request_structure_key_opaque`.
- **KISS-SYNTH-6.1-0003a** — A provider MUST NOT reinterpret, truncate, or re-encode the
  `structure_key` bytes. *Test:* `test_synth_request_structure_key_not_reencoded`.
- **KISS-SYNTH-6.1-0004** — A provider reading a provision request MUST reject, with a
  `MALFORMED_REQUEST` typed decline (§6.6), a request whose `revision_present` is a value
  other than `0` or `1`, whose `structure_key` length is outside `[1, 4096]`, or whose
  declared length exceeds the remaining input. *Test:*
  `test_synth_request_malformed_declines`.
- **KISS-SYNTH-6.1-0004a** — A provider MUST NOT allocate on an unchecked declared
  length before validating it against the remaining input. *Test:*
  `test_synth_request_no_alloc_on_unchecked_length`.
- **KISS-SYNTH-6.1-0005** — The pair `(structure_key, revision_hash)` MUST be the full
  availability identity: when `revision_present == 1` the request pins one specific build,
  and when `revision_present == 0` the request names the cell without pinning a build
  (the provider selects or builds the default revision per §6.3-0005). *Test:*
  `test_synth_request_full_identity`.
- **KISS-SYNTH-6.1-0005a** — A provider MUST NOT treat a `structure_key` alone (with
  `revision_present == 0`) as pinning a specific build. *Test:*
  `test_synth_request_key_alone_not_pinned`.

### 6.2 Provision response — `{artifact, contract}` or a typed decline

- **KISS-SYNTH-6.2-0001** — A provider MUST answer a provision request with either
  **(a)** a provision success carrying the pair `{artifact, contract}` — the built (or
  cached) callable kernel plus its full KISS-Contract document — or **(b)** a typed
  decline (§6.6). *Test:* `test_synth_response_artifact_contract_or_decline`.
- **KISS-SYNTH-6.2-0001a** — A provider MUST NOT answer a provision request with any
  third outcome (neither a provision success nor a typed decline). *Test:*
  `test_synth_response_no_third_outcome`.
- **KISS-SYNTH-6.2-0002** — Every returned kernel MUST carry its KISS-Contract. *Test:*
  `test_synth_every_kernel_carries_contract`.
- **KISS-SYNTH-6.2-0002a** — A provider MUST NOT return an artifact without an
  accompanying contract. *Test:* `test_synth_no_artifact_without_contract`.
- **KISS-SYNTH-6.2-0002b** — A provider MUST NOT push the contract eagerly per-kernel
  into the availability announce (the contract is produced or fetched on the provision
  request, §6.7-0005). *Test:* `test_synth_contract_not_pushed_to_announce`.
- **KISS-SYNTH-6.2-0003** — The contract half of a provision success MUST be carried
  inside the `PRSP` frame (§6.4-0001b) as a little-endian `u32` contract byte-length
  followed by that many bytes of the self-delimiting KISS-Contract document, and those
  bytes MUST be byte-identical to the payload the KISS-Announce contract response frame
  (`CRSP`, `KISS-ANNOUNCE-6.4-0004`) would carry for the same contract. *Test:*
  `test_synth_contract_length_delimited`.
- **KISS-SYNTH-6.2-0003a** — The contract document MUST be carried **opaquely**; the
  provision path MUST NOT reinterpret or re-encode its bytes. *Test:*
  `test_synth_contract_opaque`.
- **KISS-SYNTH-6.2-0003b** — The provision path MUST NOT require the enclosing transport
  or the KISS-Announce layer to parse the contract's internals. *Test:*
  `test_synth_contract_not_parsed_by_host`.
- **KISS-SYNTH-6.2-0004** — The returned contract's own inner framing (its magic, header
  line, inner length/checksum, and section headings, `KISS-CONTRACT-6.1`) MUST fail loudly
  on malformation **independently** of the `PRSP` contract-length prefix. *Test:*
  `test_synth_contract_inner_framing_independent`.
- **KISS-SYNTH-6.2-0004a** — A consumer that cannot validate the returned contract's
  inner framing MUST NOT bind or launch the artifact. *Test:*
  `test_synth_invalid_contract_no_launch`.
- **KISS-SYNTH-6.2-0004b** — A consumer that cannot validate the returned contract's
  inner framing MUST surface a typed error distinct from a provision success (a
  consumer-internal error enumerant, not a wire `CDEC`). *Test:*
  `test_synth_invalid_contract_typed_error`.
- **KISS-SYNTH-6.2-0004c** — A consumer MUST NOT import a headingless or magic-less
  contract as an empty or no-op contract. *Test:* `test_synth_no_empty_contract_import`.
- **KISS-SYNTH-6.2-0005** — The returned artifact's ABI MUST be the one described by the
  accompanying contract's Interface + Dispatch sections (`KISS-CONTRACT-6.5` / `6.6`).
  *Test:* `test_synth_artifact_abi_described_by_contract`.
- **KISS-SYNTH-6.2-0005a** — A provider MUST NOT describe the artifact's calling
  convention, argument signature, or launch geometry out of band (outside the contract).
  *Test:* `test_synth_no_out_of_band_abi`.
- **KISS-SYNTH-6.2-0005b** — A consumer MUST bind and launch the artifact purely from the
  contract. *Test:* `test_synth_consumer_binds_from_contract`.
- **KISS-SYNTH-6.2-0006** — The returned contract's Identity `structure_key` and
  `revision_hash` MUST equal the echoed `(structure_key, revision_hash)` identity block
  byte-for-byte (the echo always carries `revision_present == 1` and the assigned
  `revision_hash` per §6.3-0006 / §6.3-0006a). *Test:*
  `test_synth_response_identity_consistent`.
- **KISS-SYNTH-6.2-0006a** — A consumer MUST NOT bind or launch the artifact, and MUST
  surface a typed error distinct from a provision success, when the returned contract's
  Identity disagrees with the echoed identity block. *Test:*
  `test_synth_identity_mismatch_rejected`.
- **KISS-SYNTH-6.2-0007** — Per-kernel capability MUST live in the returned contract (its
  single source of truth). *Test:* `test_synth_capability_only_in_contract`.
- **KISS-SYNTH-6.2-0007a** — A provider MUST NOT carry per-kernel capability, usage,
  guarantee, or semantics in the level-2 availability record (`KISS-ANNOUNCE-6.3-0002`),
  only the `(structure_key, revision_hash)` identity. *Test:*
  `test_synth_availability_no_capability`.
- **KISS-SYNTH-6.2-0008** — The returned contract MUST satisfy the KISS-Contract
  required-core content-validity clauses: a non-empty Semantics op DAG
  (`KISS-CONTRACT-6.4`), a non-empty Guarantees section (`KISS-CONTRACT-6.8`), and the
  required-core sections (`KISS-CONTRACT-6.2-0001`). *Test:*
  `test_synth_returned_contract_content_valid`.
- **KISS-SYNTH-6.2-0008a** — A consumer or the KISS-Conform harness MUST check the
  returned contract's required-core content-validity (§6.2-0008) and MUST NOT treat a
  framing-valid but content-degenerate contract as a valid provision. *Test:*
  `test_synth_content_validity_checked`.

### 6.3 Build-on-miss (just-in-time synthesis) — the generalization

- **KISS-SYNTH-6.3-0001** — The already-exists (cache **hit**) case and the
  build-on-miss case MUST be the **same** provision request (§6.1) and the **same**
  provision response shape (§6.2 / §6.4). *Test:*
  `test_synth_hit_and_miss_same_protocol`.
- **KISS-SYNTH-6.3-0001a** — A provider MUST NOT define a separate protocol, request tag,
  or response tag for just-in-time synthesis. *Test:* `test_synth_no_separate_jit_tag`.
- **KISS-SYNTH-6.3-0002** — On a cache **miss** for a cell it can provision, a provider
  that advertises `PROVISION_ON_REQUEST` (KISS-Announce FEAT bit 32) MUST **build** the
  artifact and return `{artifact, contract}`, or return a typed decline (§6.6); building
  on miss IS just-in-time synthesis, the build-on-miss branch of the one request/response.
  *Test:* `test_synth_build_on_miss_is_jit`.
- **KISS-SYNTH-6.3-0003** — For a cell it does not already hold, a provider that does
  **not** advertise `PROVISION_ON_REQUEST` MUST return a typed decline
  (`UNKNOWN_STRUCTURE_KEY` when no such cell exists and none can be provisioned, or
  `CANNOT_PROVISION` when the cell is known but build-on-miss is unavailable). *Test:*
  `test_synth_no_provision_bit_declines_miss`.
- **KISS-SYNTH-6.3-0003a** — Such a provider MUST NOT silently omit a response to a
  provision request. *Test:* `test_synth_miss_no_silent_omit`.
- **KISS-SYNTH-6.3-0003b** — Such a provider MUST NOT block past its declared maximum
  latency bound (§6.6-0001c); on exceeding the bound it MUST surface a `CANNOT_PROVISION`
  decline. *Test:* `test_synth_miss_no_indefinite_block`.
- **KISS-SYNTH-6.3-0003c** — Such a provider MUST NOT return an empty artifact in lieu of
  a decline. *Test:* `test_synth_miss_no_empty_artifact`.
- **KISS-SYNTH-6.3-0004** — When `revision_present == 1`, a provider that returns a
  provision success (whether from cache or freshly built) MUST return the kernel whose
  `(structure_key, revision_hash)` identity matches the request exactly; if it holds no
  such revision and cannot build one, it MUST return a typed decline (`UNKNOWN_REVISION`
  or `CANNOT_PROVISION`) rather than a mismatched kernel (reusing
  `KISS-ANNOUNCE-6.4-0003`). *Test:* `test_synth_revision_pinned_build`.
- **KISS-SYNTH-6.3-0004a** — When `revision_present == 1`, a provider MUST NOT return a
  kernel whose `revision_hash` differs from the pinned request value. *Test:*
  `test_synth_revision_pinned_no_mismatch`.
- **KISS-SYNTH-6.3-0005** — When `revision_present == 0`, a provider that returns a
  provision success MUST return the kernel for the highest-ordered `revision_hash` it
  holds for that `structure_key` (ordering = byte-for-byte lexicographic descending over
  the 32-byte value, reusing `KISS-ANNOUNCE-6.4-0012`), or, if it holds none, MUST build a
  revision (assigning it a `revision_hash`) and return that, or — if it holds none and
  cannot provision one — MUST return a typed decline (`UNKNOWN_STRUCTURE_KEY` or
  `CANNOT_PROVISION`). *Test:* `test_synth_default_revision_provision`.
- **KISS-SYNTH-6.3-0006** — On any provision success (from cache or build-on-miss), a
  provider MUST set `revision_present == 1` in the echoed identity block, regardless of
  the request's `revision_present`. *Test:* `test_synth_success_echo_revision_present`.
- **KISS-SYNTH-6.3-0006a** — On any provision success, the echoed identity block MUST
  include the selected or assigned 32-byte `revision_hash` of the returned kernel,
  regardless of the request's `revision_present`. *Test:*
  `test_synth_success_echo_revision_hash`.

### 6.4 The artifact (its wire home)

- **KISS-SYNTH-6.4-0001** — The artifact MUST be the **first** payload element of the
  `{artifact, contract}` provision success (it precedes the contract in wire order): the
  built (or cached) callable kernel binary/object. *Test:*
  `test_synth_artifact_is_first_element`.
- **KISS-SYNTH-6.4-0001a** — A provider MUST NOT return the contract without the artifact
  on a provision success. *Test:* `test_synth_success_requires_artifact`.
- **KISS-SYNTH-6.4-0001b** — A provision success MUST be framed as the KISS-Synth
  provision-success frame, tag `PRSP` (wire bytes `50 52 53 50`), whose fields in wire
  order left-to-right are: the 4-byte tag; the echoed `(structure_key, revision_hash)`
  identity block (`KISS-ANNOUNCE-6.4-0011`) with `revision_present == 1`; a 4-byte
  `artifact_format_tag`; a little-endian `u64` artifact byte-length; that many artifact
  bytes; a little-endian `u32` contract byte-length; then that many bytes of the
  self-delimiting KISS-Contract document. *Test:*
  `test_synth_provision_success_prsp_frame`.
- **KISS-SYNTH-6.4-0001c** — A provider MUST NOT return a bare KISS-Announce `CRSP`
  contract-response frame (a contract with no enclosing `PRSP` and no artifact block) as
  a provision success; the byte condition that distinguishes a provision success from a
  KISS-Announce contract-only response is the leading 4-byte frame tag — `PRSP`
  (`50 52 53 50`) for a provision success, `CRSP` (`43 52 53 50`) for a contract-only
  response. *Test:* `test_synth_success_distinct_from_contract_only`.
- **KISS-SYNTH-6.4-0001d** — The `artifact_format_tag` MUST be a 4-byte tag naming the
  artifact's binary format/target family, matched byte-for-byte and otherwise opaque, and
  MUST NOT be the reserved all-zero value `00 00 00 00`. *Test:*
  `test_synth_artifact_format_tag`.
- **KISS-SYNTH-6.4-0002** — The artifact's `revision_hash` MUST identify which build the
  artifact is: an opaque, provider-assigned identifier compared only for byte-for-byte
  equality. *Test:* `test_synth_artifact_revision_hash_opaque`.
- **KISS-SYNTH-6.4-0002a** — A provider MUST NOT assume, and a consumer MUST NOT infer,
  any particular hash algorithm or recomputable input domain for the `revision_hash` from
  the provision response. *Test:* `test_synth_artifact_revision_hash_no_algo`.
- **KISS-SYNTH-6.4-0003** — The validity of a provision response MUST NOT depend on the
  consumer verifying the artifact bytes against the `revision_hash` or against the
  contract's declared Guarantees. *Test:* `test_synth_no_verify_precondition`.
- **KISS-SYNTH-6.4-0003a** — Artifact-byte verification MUST NOT be a wire precondition of
  the provision response. *Test:* `test_synth_no_verify_wire_precondition`.
- **KISS-SYNTH-6.4-0003b** — A provider MUST NOT withhold a §6.2 provision-success
  response (a `PRSP` frame satisfying §6.4-0001b) pending a consumer verification
  handshake. *Test:* `test_synth_no_withhold_pending_verify`.
- **KISS-SYNTH-6.4-0004** — A consumer SHOULD verify a received artifact against its
  contract's declared Guarantees — the determinism class, the ULP/tolerance bound, the
  MathPrecision attribute, and the accept-predicate (`structure_key`) — before trusting
  it, reusing the KISS-Conform oracle-differential harness and determinism-class
  comparators; this is a consumer-behavior obligation owned by KISS-Conform (a SHOULD),
  and this sub-standard SHALL NOT restate it as a wire MUST. *Test:*
  `test_synth_consumer_verify_should`.

### 6.5 Fidelity — the contract Guarantees, imported from KISS-Ops

- **KISS-SYNTH-6.5-0001** — The fidelity of a provided kernel MUST be exactly the
  accompanying contract's **Guarantees** section (`KISS-CONTRACT-6.8`). *Test:*
  `test_synth_fidelity_is_contract_guarantees`.
- **KISS-SYNTH-6.5-0001a** — A provider MUST NOT carry a separate or parallel fidelity,
  determinism, or precision statement in the provision frame that could disagree with the
  contract Guarantees. *Test:* `test_synth_no_parallel_fidelity`.
- **KISS-SYNTH-6.5-0002** — The determinism/fidelity enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}` MUST be imported **verbatim** from KISS-Ops
  (`KISS-OPS-6.0-0001`), spelled exactly that way. *Test:*
  `test_synth_determinism_enum_imported`.
- **KISS-SYNTH-6.5-0002a** — KISS-Synth MUST NOT define, re-spell, or re-fork a parallel
  determinism vocabulary. *Test:* `test_synth_determinism_enum_not_reforked`.
- **KISS-SYNTH-6.5-0002b** — A downstream copy MUST NOT override the KISS-Ops definition
  of the determinism/fidelity enum. *Test:* `test_synth_determinism_enum_no_override`.
- **KISS-SYNTH-6.5-0003** — The MathPrecision attribute `{bit-stable,
  reduced-mantissa-permitted}` MUST be imported **verbatim** from KISS-Ops (§6.17) and
  surfaced in the returned contract's Guarantees. *Test:*
  `test_synth_mathprecision_imported`.
- **KISS-SYNTH-6.5-0003a** — KISS-Synth MUST NOT re-fork the MathPrecision attribute.
  *Test:* `test_synth_mathprecision_not_reforked`.
- **KISS-SYNTH-6.5-0003b** — KISS-Synth MUST NOT treat the MathPrecision attribute as a
  dtype. *Test:* `test_synth_mathprecision_not_dtype`.
- **KISS-SYNTH-6.5-0003c** — KISS-Synth MUST NOT conflate the MathPrecision attribute with
  the determinism class (they are orthogonal). *Test:*
  `test_synth_mathprecision_not_conflated`.
- **KISS-SYNTH-6.5-0004** — The determinism class carried in the provided kernel's
  contract Guarantees MUST select the KISS-Conform comparator used to verify it:
  `exact-byte` selects a byte comparator; `ULP/tolerance` selects the declared-ULP
  comparator; `order-invariant/nondeterministic` selects a comparator at the tolerance the
  contract Guarantees declare for that class. *Test:*
  `test_synth_class_selects_comparator`.
- **KISS-SYNTH-6.5-0004a** — For the `order-invariant/nondeterministic` class, the
  tolerance used by the KISS-Conform comparator MUST be the one declared in the contract
  Guarantees (deferring to the KISS-Ops / KISS-Conform tolerance definition for that
  class), and MUST NOT be an implementation-chosen implicit default. *Test:*
  `test_synth_order_invariant_tolerance_declared`.
- **KISS-SYNTH-6.5-0004b** — A provider MUST NOT assert byte-for-byte identity of a
  provided kernel's floating-point result for an op whose determinism class is not
  `exact-byte`. *Test:* `test_synth_provider_no_byte_identity_claim`.
- **KISS-SYNTH-6.5-0004c** — A consumer MUST NOT expect byte-for-byte identity of a
  provided kernel's floating-point result for an op whose determinism class is not
  `exact-byte`. *Test:* `test_synth_consumer_no_byte_identity_expect`.

### 6.6 The never-panic obligation and the provision decline taxonomy

- **KISS-SYNTH-6.6-0001** — Every failure on the provision path — an unknown cell, a
  build failure, an unsupported version, a missing revision, a malformed request, or a
  request to a provider that does not answer contract-query — MUST be answered with a
  **typed decline**. *Test:* `test_synth_failure_is_typed_decline`.
- **KISS-SYNTH-6.6-0001a** — A provider handling any provision request MUST NOT panic,
  abort, or crash. This obligation is fuzz-testable and is owned by this sub-standard.
  *Test:* `test_synth_never_panics`.
- **KISS-SYNTH-6.6-0001b** — A provider handling a provision request MUST NOT read beyond
  the bytes declared by the request framing (the fixed `CYRQ` fields, plus the declared
  `structure_key` length, plus the 32-byte `revision_hash` when `revision_present == 1`).
  *Test:* `test_synth_no_oob_read`.
- **KISS-SYNTH-6.6-0001c** — A provider MUST return a response (a provision success or a
  typed decline) within its implementation-declared maximum latency bound; exceeding that
  bound MUST be surfaced as a `CANNOT_PROVISION` typed decline. *Test:*
  `test_synth_bounded_latency`.
- **KISS-SYNTH-6.6-0002** — A provision decline MUST use the pinned KISS-Announce
  decline-code enum (`KISS-ANNOUNCE-6.4-0009`), reused verbatim as little-endian `u32`
  values:

  | Value | Name | Meaning |
  |---|---|---|
  | `0x00000000` | reserved | MUST NOT be emitted |
  | `0x00000001` | `UNKNOWN_STRUCTURE_KEY` | no such cell, and none can be provisioned |
  | `0x00000002` | `CANNOT_PROVISION` | cell known but build-on-miss unavailable or failed |
  | `0x00000003` | `MALFORMED_REQUEST` | request framing invalid |
  | `0x00000004` | `QUERY_NOT_SUPPORTED` | provider does not advertise contract-query (FEAT bit 33) |
  | `0x00000005` | `VERSION_UNSUPPORTED` | negotiated version retired below floor on the provision path (§6.6-0003d) |
  | `0x00000006` | `UNKNOWN_REVISION` | `structure_key` known but requested `revision_hash` not held |

  *Test:* `test_synth_decline_code_enum`.
- **KISS-SYNTH-6.6-0002a** — A provider MUST NOT emit a core code
  (`0x00000001`–`0x00000006`) with a meaning other than the one pinned in §6.6-0002.
  *Test:* `test_synth_decline_core_meaning_fixed`.
- **KISS-SYNTH-6.6-0003** — A provider MUST map an unknown, un-provisionable cell to
  `UNKNOWN_STRUCTURE_KEY`. *Test:* `test_synth_map_unknown_structure_key`.
- **KISS-SYNTH-6.6-0003a** — A provider MUST map a known cell whose build-on-miss is
  unavailable or fails to `CANNOT_PROVISION`. *Test:* `test_synth_map_cannot_provision`.
- **KISS-SYNTH-6.6-0003b** — A provider MUST map a malformed request frame to
  `MALFORMED_REQUEST`. *Test:* `test_synth_map_malformed_request`.
- **KISS-SYNTH-6.6-0003c** — A provider MUST map a provision request received when it does
  not advertise contract-query (FEAT bit 33) to `QUERY_NOT_SUPPORTED`. *Test:*
  `test_synth_map_query_not_supported`.
- **KISS-SYNTH-6.6-0003d** — A provider MUST map a provision request (`CYRQ`) received on
  a session whose negotiated profile has since been retired below the provider's declared
  floor (§8-0007) to `VERSION_UNSUPPORTED`; this is the concrete provision-path trigger
  for `VERSION_UNSUPPORTED` (a mismatch surfacing after a level-1 handshake, not during
  it). *Test:* `test_synth_map_version_unsupported`.
- **KISS-SYNTH-6.6-0003e** — A provider MUST map a known `structure_key` with an unheld,
  unbuildable requested `revision_hash` to `UNKNOWN_REVISION`. *Test:*
  `test_synth_map_unknown_revision`.
- **KISS-SYNTH-6.6-0003f** — The provision decline taxonomy MUST be complete over exactly
  the closed set of six failure kinds enumerated in §6.6-0003 through §6.6-0003e
  (`UNKNOWN_STRUCTURE_KEY`, `CANNOT_PROVISION`, `MALFORMED_REQUEST`, `QUERY_NOT_SUPPORTED`,
  `VERSION_UNSUPPORTED`, `UNKNOWN_REVISION`); the empty-profile-intersection handshake
  failure is owned by KISS-Announce (§6.7-0002) and is not a member of this set. *Test:*
  `test_synth_decline_taxonomy_complete`.
- **KISS-SYNTH-6.6-0003g** — A provider MUST NOT answer a recognized failure kind with a
  mismatched core code. *Test:* `test_synth_no_mismatched_code`.
- **KISS-SYNTH-6.6-0004** — A provision decline MUST be framed as the KISS-Announce
  decline response frame verbatim (tag `CDEC`, wire bytes `43 44 45 43`,
  `KISS-ANNOUNCE-6.4-0007`): the 4-byte tag; the echoed `(structure_key, revision_hash)`
  identity block (`KISS-ANNOUNCE-6.4-0011`); then a little-endian `u32` `decline_code`.
  *Test:* `test_synth_decline_framing`.
- **KISS-SYNTH-6.6-0004a** — When a `MALFORMED_REQUEST` decline is emitted for a request
  whose identity cannot be safely parsed, the provider MUST echo a canonical empty
  identity in the `CDEC` frame: a `u32` `structure_key` length of `0` and
  `revision_present == 0` (no `revision_hash` bytes). *Test:*
  `test_synth_malformed_echoes_empty_identity`.
- **KISS-SYNTH-6.6-0004b** — A reader MUST accept a zero-length echoed `structure_key`
  (with `revision_present == 0`) on a `MALFORMED_REQUEST` `CDEC` frame. *Test:*
  `test_synth_reader_accepts_empty_identity`.
- **KISS-SYNTH-6.6-0005** — A producer MUST confine any non-core decline code to the
  experimental range `[0x40000000, 0x80000000)` or the vendor range `[0x80000000, 2^32)`
  (namespaced, registry-registered). *Test:* `test_synth_decline_ranges`.
- **KISS-SYNTH-6.6-0005a** — A producer MUST NOT reuse a core value
  (`0x00000001`–`0x00000006`) for a different meaning. *Test:*
  `test_synth_decline_no_core_reuse`.
- **KISS-SYNTH-6.6-0006** — A consumer that receives a `decline_code` it does not
  recognize MUST treat it as a **generic decline**. *Test:*
  `test_synth_unknown_decline_generic`.
- **KISS-SYNTH-6.6-0006a** — A consumer that receives an unrecognized `decline_code` MUST
  NOT panic, abort, or crash. *Test:* `test_synth_unknown_decline_never_panics`.
- **KISS-SYNTH-6.6-0007** — A provider that does **not** advertise the `CONTRACT_QUERY`
  capability bit (KISS-Announce FEAT bit 33) MUST answer any provision request with a
  `QUERY_NOT_SUPPORTED` typed decline (reusing `KISS-ANNOUNCE-6.4-0008`). *Test:*
  `test_synth_no_query_bit_declines`.
- **KISS-SYNTH-6.6-0007a** — Such a provider MUST NOT silently drop a provision request.
  *Test:* `test_synth_no_query_bit_no_silent_drop`.
- **KISS-SYNTH-6.6-0008** — A build failure on the build-on-miss path (a compiler error, a
  resource exhaustion, an unsupported target toolchain) MUST be surfaced as a
  `CANNOT_PROVISION` typed decline. *Test:* `test_synth_build_failure_declines`.
- **KISS-SYNTH-6.6-0008a** — A build failure MUST NOT leak as a panic, abort, crash, or a
  partially-built or empty artifact. *Test:* `test_synth_build_failure_no_leak`.

### 6.7 The three levels — handshake and availability defer to KISS-Announce

- **KISS-SYNTH-6.7-0001** — Level 1 (provider handshake) MUST defer to the KISS-Announce
  SeamHello handshake envelope (`KISS-ANNOUNCE-6.1`). *Test:*
  `test_synth_handshake_defers_to_announce`.
- **KISS-SYNTH-6.7-0001a** — KISS-Synth MUST NOT define a parallel handshake envelope.
  *Test:* `test_synth_no_parallel_handshake`.
- **KISS-SYNTH-6.7-0001b** — Level 1 negotiates provider-level capability only — which
  KISS profiles/versions each side speaks, whether the provider answers contract-query
  (`CONTRACT_QUERY`, FEAT bit 33), and whether it builds on miss (`PROVISION_ON_REQUEST`,
  FEAT bit 32) — and MUST NOT carry per-kernel capability. *Test:*
  `test_synth_handshake_no_per_kernel_capability`.
- **KISS-SYNTH-6.7-0002** — An empty profile intersection at level 1 MUST be handled as
  the KISS-Announce handshake failure (`KISS-ANNOUNCE-7.1-0002`); it is NOT a
  `CDEC`-framed KISS-Synth provision decline and is not a member of the §6.6 provision
  decline taxonomy. *Test:* `test_synth_empty_profile_is_announce_owned`.
- **KISS-SYNTH-6.7-0002a** — A provider or consumer MUST NOT panic, abort, or crash on an
  empty profile intersection. *Test:* `test_synth_empty_profile_never_panics`.
- **KISS-SYNTH-6.7-0002b** — A provider or consumer MUST NOT select a profile on an empty
  intersection. *Test:* `test_synth_empty_profile_no_select`.
- **KISS-SYNTH-6.7-0003** — Level 2 (kernel availability) MUST name the kernels a provider
  can serve by **identity only** — the `(structure_key, revision_hash)` pair (reusing
  `KISS-ANNOUNCE-6.3-0001`). *Test:* `test_synth_availability_identity_only`.
- **KISS-SYNTH-6.7-0003a** — The level-2 availability record MUST NOT carry any per-kernel
  capability, usage/ABI, dispatch, guarantee, or semantics field; those facts are carried
  only by the queried contract. *Test:* `test_synth_availability_no_capability_fields`.
- **KISS-SYNTH-6.7-0004** — A consumer MUST distinguish a cache **hit** from a **miss** by
  full byte-for-byte identity: a hit is a match on **both** `structure_key` and the
  32-byte `revision_hash`, and any other case is a miss (reusing
  `KISS-ANNOUNCE-6.3-0006`). *Test:* `test_synth_hit_miss_by_identity`.
- **KISS-SYNTH-6.7-0004a** — A consumer MUST NOT treat a `structure_key` match alone as a
  hit. *Test:* `test_synth_hit_not_key_alone`.
- **KISS-SYNTH-6.7-0005** — Level 3 (contract-query / provision-on-miss) MUST be the act
  this sub-standard owns: on a miss (or to fetch a known cell's contract) the consumer
  requests by `structure_key`, optionally revision-pinned (§6.1), and the provider returns
  `{artifact, contract}`, **building the artifact on miss** if it does not yet exist
  (§6.3), or a typed decline (§6.6). *Test:* `test_synth_level3_provision`.
- **KISS-SYNTH-6.7-0005a** — A provider MUST NOT require a separate transaction for the
  already-exists branch versus the build-on-miss branch. *Test:*
  `test_synth_no_separate_transaction`.

---

## 7. Capability, Profile & Extension model

### 7.1 Mandatory core, options, and negotiation

- **KISS-SYNTH-7.1-0001** — An implementation MUST satisfy the **mandatory core** of
  KISS-Synth to conform: answer every provision request with either a
  `{artifact, contract}` provision success or a typed decline (§6.2-0001), carry a
  contract with every returned kernel (§6.2-0002), and never panic on the provision path
  (§6.6-0001a). *Test:* `test_synth_mandatory_core`.
- **KISS-SYNTH-7.1-0001a** — An un-claimed or malformed input MUST produce a typed
  decline, never a panic. *Test:* `test_synth_core_unclaimed_declines`.
- **KISS-SYNTH-7.1-0002** — The two provider-level options that gate provision MUST be the
  KISS-Announce FEAT capability bits `PROVISION_ON_REQUEST` (bit 32 — the provider builds
  on miss / does JIT) and `CONTRACT_QUERY` (bit 33 — the provider answers contract-query /
  provision requests), pinned by `KISS-ANNOUNCE-7.2-0003`. *Test:*
  `test_synth_feature_bits`.
- **KISS-SYNTH-7.1-0002a** — KISS-Synth MUST NOT re-number, re-locate, or fork these bits.
  *Test:* `test_synth_feature_bits_not_reforked`.
- **KISS-SYNTH-7.1-0002b** — A provider that advertises `CONTRACT_QUERY` MUST implement a
  conforming provision endpoint (§6.2). *Test:* `test_synth_query_bit_implies_endpoint`.
- **KISS-SYNTH-7.1-0002c** — A provider that advertises `PROVISION_ON_REQUEST` MUST build
  on miss per §6.3-0002. *Test:* `test_synth_provision_bit_builds_on_miss`.
- **KISS-SYNTH-7.1-0003** — Per-version negotiation MUST use the KISS-Announce profile
  mechanism (`KISS-ANNOUNCE-7.1`). *Test:* `test_synth_version_via_profile`.
- **KISS-SYNTH-7.1-0003a** — KISS-Synth MUST NOT add a second "which version do we both
  speak" channel. *Test:* `test_synth_no_second_version_channel`.
- **KISS-SYNTH-7.1-0003b** — A negotiated-version mismatch surfaced on the provision path
  (per the concrete trigger of §6.6-0003d) MUST be a `VERSION_UNSUPPORTED` typed decline
  (§6.6). *Test:* `test_synth_version_mismatch_declines`.
- **KISS-SYNTH-7.1-0004** — Any non-core provision decline code MUST originate from the
  experimental or vendor ranges of the PR-gated KISS extension registry (§6.6-0005).
  *Test:* `test_synth_decline_registry`.
- **KISS-SYNTH-7.1-0004a** — An implementation MUST NOT rely on an unregistered core
  decline-code assignment. *Test:* `test_synth_no_unregistered_core_code`.
- **KISS-SYNTH-7.1-0004b** — An implementation MUST NOT hard-gate a session on a
  capability bit it does not recognize (unknown capability bits are ignored per
  `KISS-ANNOUNCE-7.2-0007`). *Test:* `test_synth_unknown_bit_not_hard_gated`.
- **KISS-SYNTH-7.1-0005** — `PROVISION_ON_REQUEST` (bit 32) MUST be treated as meaningful
  only when `CONTRACT_QUERY` (bit 33) is also advertised; a provider that advertises bit
  32 without bit 33 MUST be treated as not advertising bit 32 (the JIT capability is
  unreachable without a provision endpoint, which bit 33 gates). *Test:*
  `test_synth_provision_bit_requires_query_bit`.

---

## 8. Versioning & Lifecycle

KISS-Synth tracks the umbrella's **two version axes**: the wire/ABI provision-protocol
schema version (carried through the reused KISS-Announce frames, the KISS-Synth `PRSP`
frame, and their `envelope_version` / message framing) and the published reference-crate
semver. They move independently.

- **KISS-SYNTH-8-0001** — The provision-protocol wire/ABI schema version and the
  reference-crate semver MUST be tracked as independent axes. *Test:*
  `test_synth_two_version_axes`.
- **KISS-SYNTH-8-0001a** — A crate semver change MUST NOT be taken to imply a provision
  wire change. *Test:* `test_synth_semver_no_wire_implication`.
- **KISS-SYNTH-8-0002** — Any change to the **shape** of the provision request, success,
  or decline frame — a change to a reused KISS-Announce `CYRQ` / `CDEC` frame or to the
  KISS-Synth `PRSP` frame (field offset, size, count, or the identity/artifact/payload
  framing) — MUST bump the corresponding wire schema version. *Test:*
  `test_synth_shape_change_bumps_version`.
- **KISS-SYNTH-8-0002a** — Any such shape change MUST be coordinated across affected
  parties as a cross-party-visible RFC before it is wired. *Test:*
  `test_synth_shape_change_rfc_coordinated`.
- **KISS-SYNTH-8-0003** — Assigning a previously-reserved provision decline code (within
  the experimental/vendor ranges) or relying on a newly-assigned KISS-Announce FEAT/SUB
  capability bit MUST be additive (forward-compatible under `KISS-ANNOUNCE-7.2-0007`).
  *Test:* `test_synth_additive_no_bump`.
- **KISS-SYNTH-8-0003a** — Such an additive assignment MUST NOT bump the provision wire
  schema version. *Test:* `test_synth_additive_no_wire_version_bump`.
- **KISS-SYNTH-8-0004** — KISS-Synth MUST NOT be promoted from Draft to Frozen until ≥2
  structurally dissimilar implementations have interoperated on the golden provision
  vectors of Appendix A (a provision hit, a build-on-miss, and each decline code),
  with the already-exists and build-on-miss branches shown to be the same
  request/response. *Test:* `test_synth_freeze_gate_two_impls` (checklist gate; signed by
  the AUDIT role, not DESIGN).
- **KISS-SYNTH-8-0005** — KISS-Synth MUST NOT be promoted from Draft to Frozen until a
  foreign reader written outside the reference language has consumed the provision wire
  (the `CYRQ` / `PRSP` / `CDEC` frames, including the `PRSP`-enclosed contract payload
  that is byte-identical to `CRSP`), with endianness, pointer width, structure padding,
  the `u64` artifact-length and `u32` contract-length delimiting, and the opaque-contract
  framing explicitly checked (umbrella §5.3 freeze gate). *Test:*
  `test_synth_freeze_gate_foreign_reader` (checklist gate; AUDIT-signed).
- **KISS-SYNTH-8-0006** — KISS-Synth MUST NOT be promoted from Draft to Frozen until this
  sub-standard's KISS-Conform suite exists and passes, including the fuzz/negative-vector
  modality that exercises the §6.6 never-panic decline taxonomy, with complete
  bidirectional clause-to-test traceability. *Test:*
  `test_synth_freeze_gate_conform_suite_passes` (checklist gate; AUDIT-signed).
- **KISS-SYNTH-8-0007** — An implementation MUST NOT advertise or negotiate a provision
  profile below its declared retirement floor; profiles at or above the floor and at or
  below the current maximum define the live provision window (retire-by-floor deprecation,
  reusing `KISS-ANNOUNCE-7.1-0004`). *Test:* `test_synth_retire_by_floor`.

---

## 9. Conformance

An implementation conforms to KISS-Synth at a given provision-protocol wire/ABI schema
version if it (a) produces and parses exactly the provision request, `{artifact, contract}`
provision-success, and typed-decline wire artifacts of §6–§8 for that version, (b) passes
the KISS-Conform suite for KISS-Synth at that version, and (c) satisfies the DAG
prerequisite closure. Because the **KISS-Announce → KISS-Synth**, **KISS-Contract →
KISS-Synth**, and **KISS-Ops → KISS-Synth** edges are **STRUCTURAL** (§4), claiming
KISS-Synth requires claiming KISS-Announce, KISS-Contract, and KISS-Ops (prerequisite
closure, umbrella §6.3); KISS-Synth depends on **neither** KISS-Consume nor KISS-Emit, so
claiming it forces no claim of either. Malformed, unknown-cell, unbuildable,
missing-revision, version-retired, or query-not-supported inputs yield typed declines,
never panics, per §6.6-0001 / §6.6-0001a (the owning clauses; verified by the
fuzz/negative-vector modality). The modified-suite prohibition of the mark policy is the
umbrella's rule (umbrella §9.3), enforced via registry listing, and is not restated as a
free-standing KISS-Synth clause.

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-SYNTH-6.0-0001 | `test_synth_determinism_class_exact_byte` |
| KISS-SYNTH-6.0-0001a | `test_synth_determinism_class_no_tolerance` |
| KISS-SYNTH-6.0-0001b | `test_synth_determinism_class_not_reforked` |
| KISS-SYNTH-6.1-0001 | `test_synth_request_is_identity_only` |
| KISS-SYNTH-6.1-0001a | `test_synth_request_no_capability_fields` |
| KISS-SYNTH-6.1-0002 | `test_synth_request_reuses_cyrq` |
| KISS-SYNTH-6.1-0002a | `test_synth_request_no_new_shape` |
| KISS-SYNTH-6.1-0003 | `test_synth_request_structure_key_opaque` |
| KISS-SYNTH-6.1-0003a | `test_synth_request_structure_key_not_reencoded` |
| KISS-SYNTH-6.1-0004 | `test_synth_request_malformed_declines` |
| KISS-SYNTH-6.1-0004a | `test_synth_request_no_alloc_on_unchecked_length` |
| KISS-SYNTH-6.1-0005 | `test_synth_request_full_identity` |
| KISS-SYNTH-6.1-0005a | `test_synth_request_key_alone_not_pinned` |
| KISS-SYNTH-6.2-0001 | `test_synth_response_artifact_contract_or_decline` |
| KISS-SYNTH-6.2-0001a | `test_synth_response_no_third_outcome` |
| KISS-SYNTH-6.2-0002 | `test_synth_every_kernel_carries_contract` |
| KISS-SYNTH-6.2-0002a | `test_synth_no_artifact_without_contract` |
| KISS-SYNTH-6.2-0002b | `test_synth_contract_not_pushed_to_announce` |
| KISS-SYNTH-6.2-0003 | `test_synth_contract_length_delimited` |
| KISS-SYNTH-6.2-0003a | `test_synth_contract_opaque` |
| KISS-SYNTH-6.2-0003b | `test_synth_contract_not_parsed_by_host` |
| KISS-SYNTH-6.2-0004 | `test_synth_contract_inner_framing_independent` |
| KISS-SYNTH-6.2-0004a | `test_synth_invalid_contract_no_launch` |
| KISS-SYNTH-6.2-0004b | `test_synth_invalid_contract_typed_error` |
| KISS-SYNTH-6.2-0004c | `test_synth_no_empty_contract_import` |
| KISS-SYNTH-6.2-0005 | `test_synth_artifact_abi_described_by_contract` |
| KISS-SYNTH-6.2-0005a | `test_synth_no_out_of_band_abi` |
| KISS-SYNTH-6.2-0005b | `test_synth_consumer_binds_from_contract` |
| KISS-SYNTH-6.2-0006 | `test_synth_response_identity_consistent` |
| KISS-SYNTH-6.2-0006a | `test_synth_identity_mismatch_rejected` |
| KISS-SYNTH-6.2-0007 | `test_synth_capability_only_in_contract` |
| KISS-SYNTH-6.2-0007a | `test_synth_availability_no_capability` |
| KISS-SYNTH-6.2-0008 | `test_synth_returned_contract_content_valid` |
| KISS-SYNTH-6.2-0008a | `test_synth_content_validity_checked` |
| KISS-SYNTH-6.3-0001 | `test_synth_hit_and_miss_same_protocol` |
| KISS-SYNTH-6.3-0001a | `test_synth_no_separate_jit_tag` |
| KISS-SYNTH-6.3-0002 | `test_synth_build_on_miss_is_jit` |
| KISS-SYNTH-6.3-0003 | `test_synth_no_provision_bit_declines_miss` |
| KISS-SYNTH-6.3-0003a | `test_synth_miss_no_silent_omit` |
| KISS-SYNTH-6.3-0003b | `test_synth_miss_no_indefinite_block` |
| KISS-SYNTH-6.3-0003c | `test_synth_miss_no_empty_artifact` |
| KISS-SYNTH-6.3-0004 | `test_synth_revision_pinned_build` |
| KISS-SYNTH-6.3-0004a | `test_synth_revision_pinned_no_mismatch` |
| KISS-SYNTH-6.3-0005 | `test_synth_default_revision_provision` |
| KISS-SYNTH-6.3-0006 | `test_synth_success_echo_revision_present` |
| KISS-SYNTH-6.3-0006a | `test_synth_success_echo_revision_hash` |
| KISS-SYNTH-6.4-0001 | `test_synth_artifact_is_first_element` |
| KISS-SYNTH-6.4-0001a | `test_synth_success_requires_artifact` |
| KISS-SYNTH-6.4-0001b | `test_synth_provision_success_prsp_frame` |
| KISS-SYNTH-6.4-0001c | `test_synth_success_distinct_from_contract_only` |
| KISS-SYNTH-6.4-0001d | `test_synth_artifact_format_tag` |
| KISS-SYNTH-6.4-0002 | `test_synth_artifact_revision_hash_opaque` |
| KISS-SYNTH-6.4-0002a | `test_synth_artifact_revision_hash_no_algo` |
| KISS-SYNTH-6.4-0003 | `test_synth_no_verify_precondition` |
| KISS-SYNTH-6.4-0003a | `test_synth_no_verify_wire_precondition` |
| KISS-SYNTH-6.4-0003b | `test_synth_no_withhold_pending_verify` |
| KISS-SYNTH-6.4-0004 | `test_synth_consumer_verify_should` |
| KISS-SYNTH-6.5-0001 | `test_synth_fidelity_is_contract_guarantees` |
| KISS-SYNTH-6.5-0001a | `test_synth_no_parallel_fidelity` |
| KISS-SYNTH-6.5-0002 | `test_synth_determinism_enum_imported` |
| KISS-SYNTH-6.5-0002a | `test_synth_determinism_enum_not_reforked` |
| KISS-SYNTH-6.5-0002b | `test_synth_determinism_enum_no_override` |
| KISS-SYNTH-6.5-0003 | `test_synth_mathprecision_imported` |
| KISS-SYNTH-6.5-0003a | `test_synth_mathprecision_not_reforked` |
| KISS-SYNTH-6.5-0003b | `test_synth_mathprecision_not_dtype` |
| KISS-SYNTH-6.5-0003c | `test_synth_mathprecision_not_conflated` |
| KISS-SYNTH-6.5-0004 | `test_synth_class_selects_comparator` |
| KISS-SYNTH-6.5-0004a | `test_synth_order_invariant_tolerance_declared` |
| KISS-SYNTH-6.5-0004b | `test_synth_provider_no_byte_identity_claim` |
| KISS-SYNTH-6.5-0004c | `test_synth_consumer_no_byte_identity_expect` |
| KISS-SYNTH-6.6-0001 | `test_synth_failure_is_typed_decline` |
| KISS-SYNTH-6.6-0001a | `test_synth_never_panics` |
| KISS-SYNTH-6.6-0001b | `test_synth_no_oob_read` |
| KISS-SYNTH-6.6-0001c | `test_synth_bounded_latency` |
| KISS-SYNTH-6.6-0002 | `test_synth_decline_code_enum` |
| KISS-SYNTH-6.6-0002a | `test_synth_decline_core_meaning_fixed` |
| KISS-SYNTH-6.6-0003 | `test_synth_map_unknown_structure_key` |
| KISS-SYNTH-6.6-0003a | `test_synth_map_cannot_provision` |
| KISS-SYNTH-6.6-0003b | `test_synth_map_malformed_request` |
| KISS-SYNTH-6.6-0003c | `test_synth_map_query_not_supported` |
| KISS-SYNTH-6.6-0003d | `test_synth_map_version_unsupported` |
| KISS-SYNTH-6.6-0003e | `test_synth_map_unknown_revision` |
| KISS-SYNTH-6.6-0003f | `test_synth_decline_taxonomy_complete` |
| KISS-SYNTH-6.6-0003g | `test_synth_no_mismatched_code` |
| KISS-SYNTH-6.6-0004 | `test_synth_decline_framing` |
| KISS-SYNTH-6.6-0004a | `test_synth_malformed_echoes_empty_identity` |
| KISS-SYNTH-6.6-0004b | `test_synth_reader_accepts_empty_identity` |
| KISS-SYNTH-6.6-0005 | `test_synth_decline_ranges` |
| KISS-SYNTH-6.6-0005a | `test_synth_decline_no_core_reuse` |
| KISS-SYNTH-6.6-0006 | `test_synth_unknown_decline_generic` |
| KISS-SYNTH-6.6-0006a | `test_synth_unknown_decline_never_panics` |
| KISS-SYNTH-6.6-0007 | `test_synth_no_query_bit_declines` |
| KISS-SYNTH-6.6-0007a | `test_synth_no_query_bit_no_silent_drop` |
| KISS-SYNTH-6.6-0008 | `test_synth_build_failure_declines` |
| KISS-SYNTH-6.6-0008a | `test_synth_build_failure_no_leak` |
| KISS-SYNTH-6.7-0001 | `test_synth_handshake_defers_to_announce` |
| KISS-SYNTH-6.7-0001a | `test_synth_no_parallel_handshake` |
| KISS-SYNTH-6.7-0001b | `test_synth_handshake_no_per_kernel_capability` |
| KISS-SYNTH-6.7-0002 | `test_synth_empty_profile_is_announce_owned` |
| KISS-SYNTH-6.7-0002a | `test_synth_empty_profile_never_panics` |
| KISS-SYNTH-6.7-0002b | `test_synth_empty_profile_no_select` |
| KISS-SYNTH-6.7-0003 | `test_synth_availability_identity_only` |
| KISS-SYNTH-6.7-0003a | `test_synth_availability_no_capability_fields` |
| KISS-SYNTH-6.7-0004 | `test_synth_hit_miss_by_identity` |
| KISS-SYNTH-6.7-0004a | `test_synth_hit_not_key_alone` |
| KISS-SYNTH-6.7-0005 | `test_synth_level3_provision` |
| KISS-SYNTH-6.7-0005a | `test_synth_no_separate_transaction` |
| KISS-SYNTH-7.1-0001 | `test_synth_mandatory_core` |
| KISS-SYNTH-7.1-0001a | `test_synth_core_unclaimed_declines` |
| KISS-SYNTH-7.1-0002 | `test_synth_feature_bits` |
| KISS-SYNTH-7.1-0002a | `test_synth_feature_bits_not_reforked` |
| KISS-SYNTH-7.1-0002b | `test_synth_query_bit_implies_endpoint` |
| KISS-SYNTH-7.1-0002c | `test_synth_provision_bit_builds_on_miss` |
| KISS-SYNTH-7.1-0003 | `test_synth_version_via_profile` |
| KISS-SYNTH-7.1-0003a | `test_synth_no_second_version_channel` |
| KISS-SYNTH-7.1-0003b | `test_synth_version_mismatch_declines` |
| KISS-SYNTH-7.1-0004 | `test_synth_decline_registry` |
| KISS-SYNTH-7.1-0004a | `test_synth_no_unregistered_core_code` |
| KISS-SYNTH-7.1-0004b | `test_synth_unknown_bit_not_hard_gated` |
| KISS-SYNTH-7.1-0005 | `test_synth_provision_bit_requires_query_bit` |
| KISS-SYNTH-8-0001 | `test_synth_two_version_axes` |
| KISS-SYNTH-8-0001a | `test_synth_semver_no_wire_implication` |
| KISS-SYNTH-8-0002 | `test_synth_shape_change_bumps_version` |
| KISS-SYNTH-8-0002a | `test_synth_shape_change_rfc_coordinated` |
| KISS-SYNTH-8-0003 | `test_synth_additive_no_bump` |
| KISS-SYNTH-8-0003a | `test_synth_additive_no_wire_version_bump` |
| KISS-SYNTH-8-0004 | `test_synth_freeze_gate_two_impls` |
| KISS-SYNTH-8-0005 | `test_synth_freeze_gate_foreign_reader` |
| KISS-SYNTH-8-0006 | `test_synth_freeze_gate_conform_suite_passes` |
| KISS-SYNTH-8-0007 | `test_synth_retire_by_floor` |

Every normative clause above appears in this matrix exactly once; the KISS-Conform build
fails if any clause ID lacks a passing mapped test (bidirectional traceability, owned by
KISS-Conform per umbrella §3.3). Clause IDs are mirrored in the machine-readable sidecar
(`kiss-synth.validusage.json` analog) kept in sync by the traceability lint.

---

## 10. Governance

- **Editor of record:** the KISS-Synth editor assignment is **proposed, pending
  ratification** in the umbrella governance record (which does not yet finalize an editor
  for this sub-standard). The editor holds the pen, allocates clause IDs (append-only,
  never reused after retirement), and solicits comment from interested cosignatories — any
  project building a provider, a consumer, or a just-in-time builder that provisions or
  requests kernels — before deciding a cross-party-visible change. Because KISS-Synth
  reuses the KISS-Announce frames and decline enum, a cross-party-visible change to a
  provision request/decline frame or a core decline code is coordinated with the
  KISS-Announce editor as an RFC before it is wired.
- **Steward:** ThinkersJournal hosts the spec, the extension/decline-code registry
  (PR-gated; note that the handshake envelope, capability bitset, and decline-code enum are
  owned by KISS-Announce, the contract format by KISS-Contract, and the determinism/
  MathPrecision vocabulary by KISS-Ops — not by a KISS-Synth registry, though the `PRSP`
  provision-success frame and the `artifact_format_tag` registry are KISS-Synth's own),
  and the conformance registry; it free-certifies self-certified implementations on request
  as resources permit.
- **Ratifier / maturity transitions:** the AUDIT role (not DESIGN) signs each maturity
  transition; the Draft→Frozen transition requires the freeze gate of §8-0004 / §8-0005 /
  §8-0006 (umbrella §5.3).
- **License:** this specification is dedicated to the public domain under CC0 1.0
  Universal; reference crates are MIT-OR-Apache-2.0; the KISS-Conform suite is
  permissive-to-run. Per the umbrella mark policy (umbrella §9.3), a modified conformance
  suite does not back a conformance claim; that policy is enforced via steward-registry
  listing, not restated as a normative KISS-Synth clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC
  contribution, with defensive termination, per the umbrella.
- **Conformance posture:** self-certification with published results plus the
  steward-maintained registry is the authoritative record of verified implementations.

---

## Appendix A — Worked provision vectors & provenance (informative)

**A.1 Golden provision vectors.** The strided binary `add` provision of §2.6 is the first
golden vector for `test_synth_request_reuses_cyrq`,
`test_synth_response_artifact_contract_or_decline`,
`test_synth_every_kernel_carries_contract`, `test_synth_provision_success_prsp_frame`,
`test_synth_contract_length_delimited`, and `test_synth_build_on_miss_is_jit`: a `CYRQ`
request with a `revision_present = 0` flag over the `bin/f32,f32,f32/strided/cuda:sm89`
`structure_key`, answered — on a miss — by a `PRSP` provision-success frame carrying the
echoed `(structure_key, revision_hash)` identity (with `revision_present = 1` and the
freshly assigned 32-byte `revision_hash`, per §6.3-0006 / §6.3-0006a), a 4-byte
`artifact_format_tag` naming the artifact's format/target family, a `u64` artifact
byte-length + the freshly-built artifact bytes, then a `u32` contract byte-length + the
opaque KISS-Contract document (a one-node `add` Semantics DAG, determinism class
`exact-byte`, MathPrecision `bit-stable`). The **same** `CYRQ` request answered from cache
(a hit) yields the **same** `PRSP` response (echoing the same held `revision_hash`), the
golden vector for `test_synth_hit_and_miss_same_protocol`. A separate vector pairs this
`PRSP` frame against a bare `CRSP` contract-only response over the same identity to fix the
distinguishing byte condition (leading `PRSP` vs `CRSP` tag) for
`test_synth_success_distinct_from_contract_only`.

The unbuildable-cell decline of §2.7 is the golden vector for `test_synth_decline_framing`,
`test_synth_decline_code_enum`, `test_synth_map_cannot_provision`, and
`test_synth_build_failure_declines`: a `CDEC` frame carrying the echoed identity and
`decline_code = 0x00000002` (`CANNOT_PROVISION`). Additional negative vectors —
an out-of-range `revision_present` flag whose identity cannot be safely parsed
(`test_synth_request_malformed_declines` / `test_synth_malformed_echoes_empty_identity` →
`MALFORMED_REQUEST` with a canonical empty echoed identity, `u32` `structure_key`
length `0` and `revision_present = 0`), a request to a provider without FEAT bit 33
(`test_synth_no_query_bit_declines` / `test_synth_map_query_not_supported` →
`QUERY_NOT_SUPPORTED`), a revision-pinned request for an unheld build
(`test_synth_revision_pinned_build` / `test_synth_map_unknown_revision` →
`UNKNOWN_REVISION`), an unknown cell on a non-provisioning provider
(`test_synth_no_provision_bit_declines_miss` / `test_synth_map_unknown_structure_key` →
`UNKNOWN_STRUCTURE_KEY`), a `CYRQ` received on a session whose negotiated profile has been
retired below floor (`test_synth_map_version_unsupported` /
`test_synth_version_mismatch_declines` → `VERSION_UNSUPPORTED`), and an unrecognized
experimental-range decline code (`test_synth_unknown_decline_generic`) — drive the §6.6
decline tests and form the fuzz/adversarial-outsider battery for the foreign-reader freeze
gate. Each reused message tag (`CYRQ` / `CDEC`) and the KISS-Synth `PRSP` frame carries a
golden vector with an explicit "bytes on the wire, left to right" row: for `PRSP` the row
is tag `50 52 53 50` · echoed identity block · `artifact_format_tag` · `u64` artifact
length · artifact bytes · `u32` contract length · contract bytes; the `CYRQ` and `CDEC`
rows are identical to the KISS-Announce Appendix A vectors they reuse, and the
`PRSP`-enclosed contract payload is byte-identical to the KISS-Announce `CRSP` payload for
the same contract.

**A.2 Provenance / acknowledgments.** The provision-by-identity request key, the
`{artifact, contract}` response pair, the build-on-miss-equals-JIT unification, the
never-panic decline taxonomy, and the deferral of handshake and availability to the
discovery seam derive from a kernel-provision/JIT reference crate (the Baracuda
`baracuda-kernelgen` provision path), which assembled a per-kernel artifact and its contract
from an op definition and its specialization cell and answered a request-by-`structure_key`.
The neutralization of the earlier vendor vocabulary — a vendor "JIT request/response/synth
artifact" surface rendered as the generic provision request / `{artifact, contract}`
response / typed decline; the JIT-on-request trigger re-based onto the KISS-Announce
`PROVISION_ON_REQUEST` / `CONTRACT_QUERY` capability bits; the determinism vocabulary imported
from KISS-Ops rather than re-forked — is recorded here as design provenance, not as a
normative requirement. Project and crate names in this appendix and in §0/§2 are
non-normative provenance and examples only; no normative clause names any project.

## Appendix B — Open questions (informative)

These are surfaced ambiguities that do not block the Draft but must be resolved (each as a
numbered RFC) before any KISS-Synth freeze:

- **The `artifact_format_tag` value registry.** The provision-success frame now pins a
  4-byte `artifact_format_tag` field (§6.4-0001b / §6.4-0001d), resolving the earlier
  "is a format/target tag needed" question in the affirmative. What remains open is the
  registry of concrete tag values (the format/target families and their exact 4-byte
  spellings) and whether that registry is KISS-Synth-owned or shared with KISS-Announce.
- **Artifact-byte verification depth.** The response is `{artifact, contract}`, and the
  consumer-verify obligation is a KISS-Conform SHOULD (§6.4-0004), not a Synth wire MUST;
  how deeply a consumer verifies the artifact bytes against `revision_hash` or against the
  contract's declared Guarantees (beyond the framing and identity checks the wire pins) is
  left to KISS-Conform.
- **Build-on-miss latency / async.** Provision folds JIT into one request/response and
  §6.6-0001c now pins a bounded-latency obligation (exceeding the bound is
  `CANNOT_PROVISION`); whether a provider MAY instead return a "building, retry"
  intermediate response — versus blocking within the bound or declining — is not pinned.
- **Provision at scale.** A provider may offer thousands of cells; the availability-by-identity
  list is framed (`record_count <= 2^20`), but the churn/streaming/pagination story for large
  or dynamically-provisioned cell sets, and cache-invalidation on revision bumps, is not
  covered.

---

*End of KISS-Synth / Provision (Draft proposal). This sub-standard is informative in §0–§5
and normative in §6+; every binding requirement carries an identified clause with a mapped
KISS-Conform test. KISS-Synth owns the kernel-provision protocol: a consumer asks a provider
for a kernel by identity and receives `{artifact, contract}`, the provider building it on a
cache miss (JIT = the build-on-miss branch of the one request/response). It is the
generalization of the KISS-Announce contract-query, returns the KISS-Contract document, and
imports the KISS-Ops determinism/fidelity enum and MathPrecision attribute verbatim —
depending structurally on KISS-Announce, KISS-Contract, and KISS-Ops, and on neither
KISS-Consume nor KISS-Emit. Every returned kernel carries its contract; every failure is a
typed decline, never a panic. Project and product names appear only in non-normative
examples, provenance, and the reference-implementation pointer; normative clauses use only
the generic roles provider, consumer, implementation, kernel, contract, and target.*
