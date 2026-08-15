# KISS — Kernel Interface Standards Suite

**Umbrella Specification**

**Status:** Draft proposal (pre-freeze; nothing in this document is Frozen)
**Document class:** Suite front-door — **informative throughout**. This umbrella introduces no independent normative clauses. Every binding requirement lives in a sub-standard's normative section as an identified clause (§3.3) mapped one-to-one to a KISS-Conform test; where this umbrella describes such a requirement, it is restating a rule the named sub-standard owns and enforces, and the sub-standard's clause is authoritative.
**Version:** Umbrella v0.1
**Steward:** ThinkersJournal — a non-profit publisher of free, open public standards — is the steward of record for the KISS suite. ThinkersJournal holds the specification text, the extension and namespace registries, and the free-certification registry; it does not author sub-standards, does not police claims, and holds custody in trust for the interested-cosignatory community.

## Abstract

KISS is a suite of interrelated, independently-conformable sub-standards that define the interface across which machine-learning libraries, compute libraries, and kernel providers exchange kernels: how a provider announces which kernels it has, how the two parties share capabilities, how they negotiate the data a kernel accepts and produces, how a consumer learns what a kernel computes and how to call it, and how a missing kernel is provisioned on request. KISS standardizes the seam *between* software, expressed as wire formats, ABIs, protocols, and a shared vocabulary. It does not standardize kernel implementations, source languages, in-ecosystem kernel loading and dispatch, or the internals of any implementation's compiler intermediate representation. Every sub-standard follows one dual-document template, uses one normative-keyword convention, and carries append-only clause identifiers that map one-to-one to executable conformance tests.

---

## 1. Purpose & scope of the suite

### 1.1 What KISS is

KISS defines the **interface between machine-learning libraries, compute libraries, and kernel providers**. Concretely, it standardizes the seam across which a graph or runtime (the **consumer**) and a kernel source (the **provider**) agree on:

- **Availability announcement** — a provider declares, at handshake time, which KISS sub-standards and versions it speaks and, separately, which kernels it can supply (by identity, so a consumer can distinguish a cache hit from a miss).
- **Capability sharing** — the two parties negotiate a mutually-supported profile: which sub-standards and versions, which optional features, and which external data tokens each side understands.
- **I/O negotiation** — the operands a kernel accepts and produces are described in a shared vocabulary: count, rank, extents, strides, dtype, alignment, and layout.
- **Missing-kernel provision** — when a consumer needs a kernel the provider has not yet built, the consumer requests it by identity and the provider returns it, building it on demand if necessary. Provision is the general form of just-in-time synthesis.
- **The shared vocabulary** — a common, vendor-neutral description of *data* (operand shapes and types), *computation* (the op set and per-op semantics), and *contracts* (what a kernel computes, how to call it, and what it guarantees), so that two independently-written implementations agree not merely on syntax but on meaning.

KISS is a **wire/ABI + protocol + vocabulary** standard. Its currency is bytes on a wire and the documented meaning of those bytes, not any one language's surface spelling.

### 1.2 What KISS is NOT

KISS explicitly does **not** standardize, and every sub-standard restates its own one-line exclusion drawn from this list:

- **Kernel implementations.** KISS describes how a kernel is announced, described, called, and provisioned; it never dictates how a kernel computes its result.
- **Source languages.** The languages kernels are written in are out of scope. KISS neither defines nor blesses any source syntax.
- **In-ecosystem kernel loading and dispatch.** Loading a compiled kernel and dispatching it *within* an execution ecosystem is already standardized by that ecosystem (for example, the platform's own device-side loader and command submission model). KISS governs kernel exchange *between* software and buys nothing by reinventing intra-ecosystem load/dispatch.
- **The internals of any implementation's intermediate representation.** KISS treats the neutral IR as an **opaque hub**: the *meaning* of each operation (the op-semantics currency) is normative and lives in KISS-Ops, but the in-memory representation, type layout, and lowering pipeline of any implementation's IR are not standardized and cannot be relied upon across the wire (KISS-Ops and KISS-Emit own this exclusion as identified clauses).

Scope creep by silence is a named trap. Silence is not an implicit inclusion; anything not enumerated as in-scope in a sub-standard's Purpose section is out of scope for that sub-standard.

---

## 2. The suite

KISS is a **set of interrelated sub-standards with a strict dependency DAG**, not one monolithic standard. An implementor conforms to the subset it needs: a provider that only announces pre-built kernels implements far less than a just-in-time generator, and neither is forced to adopt contracts it does not use. There are **nine** sub-standards.

### 2.1 The nine sub-standards

**KISS-Classify (foundational — data vocabulary).** Owns the vocabulary that describes *data*: the dtype set, operand descriptors (rank, extents, signed strides, alignment), layout and op-family tags, the specialization-cell identity (`structure_key`), and the namespaced, all-hardware target-capability descriptor. The dtype set is **pure storage** — a dtype pins byte layout only; compute precision (bit-stable versus reduced-mantissa) is deliberately *not* a dtype but a KISS-Ops fidelity attribute, and index/address use is an operand role rather than a dtype class. It is the shared noun set every other sub-standard uses to talk about operands and specialization cells.

**KISS-Ops (foundational — computation vocabulary).** Owns the *computation* vocabulary: the op set, each op's pinned semantics (NaN propagation, signed zero, IEEE-fmax versus NaN-propagating-max conventions, wrapping-integer behavior, raw-bit select, and similar edge cases), each non-primitive op's reference decomposition into strictly-lower-level ops, and the mandatory **primitive floor** every consumer must understand. It is the concrete home of the opaque-hub op-semantics currency and the termination guarantee for recursive contract resolution. Because an op's determinism is a property of its semantics, KISS-Ops also owns the single canonical **determinism/fidelity enum** `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` (that spelling, verbatim, is the canonical form owned by KISS-Ops §6.0-0001); KISS-Synth, KISS-Consume, KISS-Emit, KISS-Contract, and KISS-Conform import it rather than re-fork it. Ownership sits here, in a foundational vocabulary, so that no lower-tier vocabulary must import upward from a protocol-tier sub-standard. It likewise owns the orthogonal **compute-fidelity (MathPrecision) attribute** (`bit-stable` versus `reduced-mantissa-permitted`) — the home of the compute-precision distinction that is deliberately not a dtype — and the **complex-arithmetic op family** over the `c64`/`c128` storage dtypes, every member of which is non-primitive over the real primitive floor (introducing no new axiom) and pinned to ISO C Annex G. Finally it owns the **OpAttrs channel**: the per-op compile-time attribute record (out-of-bounds policy, reduce-axis mask, scan/pool/gather parameters, …), its frozen sub-vocabularies, and a **canonical, default-resolved, little-endian wire encoding** in which every field is emitted explicitly at its resolved value (no elision) so the blob is byte-comparable by a party that treats it as opaque — KISS-Grammar and KISS-Contract embed it without parsing it.

**KISS-Grammar (structural on Ops + Classify).** Owns the advertisable-op surface and region grammar: the mapping from an op tag to a KISS-Ops op name plus pattern attributes, synthesis attributes, and an operand-role tuple; the pinned region wire form; and how a frozen grammar admits a still-growing op set. It re-bases each advertisable op onto a KISS-Ops op name so there is no parallel, forkable op list; one KISS-Ops op name may back several advertisable tags distinguished by their attributes. The advertisable surface is **not on the mandatory contract path**: a valid kernel contract may instead carry a bare KISS-Ops op DAG with no advertisable-op tag (a hand-written kernel is the common case), so KISS-Grammar is not required for every kernel. Grammar's *pattern attributes* are matching hints and are distinct from the KISS-Ops-owned **OpAttrs channel**, which Grammar carries as opaque bytes and never interprets.

**KISS-Contract (structural on Ops, Classify, Grammar).** Owns the universal, vendor-neutral kernel-contract format: the seven-section document (identity, semantics, interface/ABI, dispatch, capabilities, guarantees, provenance) that tells a consumer what a kernel computes and exactly how to call it. Every provided kernel carries a contract; the contract is the single source of truth for per-kernel capability.

**KISS-Announce (opaque on Classify; opaque on Contract).** Owns the provider handshake and availability protocol: the fixed-layout, little-endian, language-independent handshake envelope; the version-negotiation procedure (highest mutual profile, hard-fail-never-panic on empty intersection); the split capability bitset; the identity-only kernel-availability list; and the contract-query request/response. Announce carries the `structure_key` (from KISS-Classify) and the contract-query payload (defined by KISS-Contract) as **opaque, length-delimited tokens** — it never parses their internals — so both upstream edges are OPAQUE. The per-kernel announce carries identity only — never per-kernel capability, which is the contract's job.

**KISS-Synth / Provision (structural on Announce, Contract, Ops).** Owns the kernel-provision protocol: a consumer asks a provider for a kernel by identity and receives `{artifact, contract}`, the provider building it on a cache miss. Just-in-time synthesis is the build-on-miss case of the same request/response. This sub-standard owns the never-panic obligation on the provision path; the canonical determinism/fidelity enum it uses is **imported from KISS-Ops** (a foundational vocabulary), not owned here, which keeps the dependency DAG acyclic.

**KISS-Consume (structural on Ops, Classify, Contract).** Owns the recognition direction: lifting a kernel or source region into the op DAG as far as it goes, with a normative refusal taxonomy for the un-liftable remainder (not-a-kernel / wrong-op-class / unrecognized-but-expressible / inexpressible-residue). Recognition is structure-based; substring or keyword sniffing is declared non-conforming. Source languages and grammars are out of scope.

**KISS-Emit (structural on Ops, Classify, Contract).** Owns the generation direction: a complete partition of every lowering decision into "the neutral driver may spell it" versus "the emitter must supply it," with non-finite and constant spelling emitter-supplied. Its normative input is an op definition plus a specialization-cell identity, not any implementation's schedule-resolved plan. It defines the emit/consume round-trip in two tiers: structural op-DAG equality over a declared subset, and numeric bit-identity only same-language on-device (cross-language numeric identity is never claimed).

**KISS-Conform (cross-cutting — depends on all).** Owns conformance: the bidirectional clause-to-test traceability matrix (the suite build fails on any untested normative MUST), the four test modalities (golden byte-vectors; an independent CPU-oracle differential harness that shares no lowering code with any reference impl; an IR-DAG fuzzer emitting to every backend; and negative/decline vectors), determinism-class-aware comparators, per-sub-standard-per-version keying, and the adversarial-outsider checklist that gates any freeze. The reference implementation runs the same public suite with no exemption.

### 2.2 Dependency DAG

Edges point from a dependency to its dependent (an edge `A → B` means "B depends on A"). Every edge is labeled **OPAQUE** or **STRUCTURAL** in the depending sub-standard's Normative References; the two foundational vocabularies sit at the bottom. The complete edge set is the **edge table** below — it is authoritative for prerequisite-closure. The diagram that follows it is a reading aid that draws only nearest parents; where the diagram and the table disagree, the table governs.

**Complete edge table (dependency → dependent, with label):**

| Dependency → Dependent | Label |
|---|---|
| KISS-Classify → KISS-Grammar | STRUCTURAL |
| KISS-Ops → KISS-Grammar | STRUCTURAL |
| KISS-Classify → KISS-Contract | STRUCTURAL |
| KISS-Ops → KISS-Contract | STRUCTURAL |
| KISS-Grammar → KISS-Contract | STRUCTURAL |
| KISS-Classify → KISS-Announce | OPAQUE |
| KISS-Contract → KISS-Announce | OPAQUE |
| KISS-Announce → KISS-Synth/Provision | STRUCTURAL |
| KISS-Contract → KISS-Synth/Provision | STRUCTURAL |
| KISS-Ops → KISS-Synth/Provision | STRUCTURAL |
| KISS-Classify → KISS-Consume | STRUCTURAL |
| KISS-Ops → KISS-Consume | STRUCTURAL |
| KISS-Contract → KISS-Consume | STRUCTURAL |
| KISS-Classify → KISS-Emit | STRUCTURAL |
| KISS-Ops → KISS-Emit | STRUCTURAL |
| KISS-Contract → KISS-Emit | STRUCTURAL |
| each of the other eight → KISS-Conform | test dependency (Conform tests every sub-standard) |

Reading aid (nearest structural parents only; consult the table for the full set):

```
                 FOUNDATIONAL VOCABULARIES
        ┌─────────────────┐     ┌─────────────────┐
        │  KISS-Classify  │     │    KISS-Ops     │
        │  (data vocab)   │     │ (compute vocab) │
        └──┬───┬───┬───┬──┘     └─┬───┬───┬───┬───┘
           │   │   │   │          │   │   │   │
           │   │   │   └───┐  ┌───┘   │   │   │
           │   │   │       ▼  ▼       │   │   │
           │   │   │   ┌──────────────┐│   │   │
           │   │   │   │ KISS-Grammar ││   │   │
           │   │   │   └──────┬───────┘│   │   │
           │   │   │          │        │   │   │
           │   │   └────┐  ┌──▼────────▼┐  │   │
           │   │        ▼  ▼             │  │   │
           │   │   ┌───────────────────┐ │  │   │
           │   │   │    KISS-Contract   │ │  │   │
           │   │   │ (identity+sem+iface│ │  │   │
           │   │   │  +dispatch+caps+   │ │  │   │
           │   │   │  guarantees+prov)  │ │  │   │
           │   │   └─┬────┬────┬────┬───┘ │  │   │
           │   │     │    │    │    │     │  │   │
           │(opaque) │    │    │    │     │  │   │
           ▼   ▼     ▼    │    │    ▼     ▼  ▼   ▼
       ┌───────────────┐  │    │  ┌─────────────┐ ┌────────────┐
       │ KISS-Announce │  │    │  │ KISS-Consume│ │ KISS-Emit  │
       │ (handshake +  │  │    │  │ (lift/      │ │ (generate/ │
       │  availability │  │    │  │  recognize) │ │  lower)    │
       │  + contract-  │  │    │  └─────────────┘ └────────────┘
       │  query)       │  │    │
       └──────┬────────┘  ▼    ▼
              │        ┌────────────┐
              │        │ KISS-Synth │
              └───────▶│ /Provision │
       (Announce +     │ (build-on- │
        Contract + Ops)│  miss)     │
                       └─────┬──────┘
                             │
        (Conform depends on and tests the OTHER EIGHT)
                             ▼
       ╔══════════════════════════════════════╗
       ║        KISS-Conform (cross-cutting)   ║
       ║  depends on the 8; gates every freeze   ║
       ╚══════════════════════════════════════╝
```

Foundational tier: **KISS-Classify** (data) and **KISS-Ops** (computation). Middle tier: **KISS-Grammar** and **KISS-Contract**. Protocol/behavior tier: **KISS-Announce**, **KISS-Synth/Provision**, **KISS-Consume**, **KISS-Emit**. Cross-cutting: **KISS-Conform**, which depends on and tests every sub-standard and signs every maturity transition. The graph is acyclic: the foundational vocabularies depend on nothing, and no upstream node depends on a downstream one.

---

## 3. Conventions

These conventions are stated once here. Every sub-standard's Conventions section (§5) points to this section and does not restate the rules.

### 3.1 Normative keywords

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in KISS documents are to be interpreted as described in RFC 2119 and RFC 8174 when, and only when, they appear in all capitals.

### 3.2 The hard normative/informative split

Each sub-standard document has an **informative** half (Overview, §0–§5) and a **normative** half (Conformance spec, §6–§10 plus appendices). The split is hard, and each sub-standard's own normative clauses enforce it:

- Informative sections may use lowercase "must", "should", and "may" freely as ordinary prose; lowercase keywords carry no normative force.
- Normative sections (§6 onward) express byte-level and behavioral requirements using **only the uppercase keywords MUST, MUST NOT, and SHALL**. Wire facts and ABI facts are never expressed with SHOULD or MAY.
- SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL are reserved for governance obligations and consumer-behavior guidance (for example, "a consumer SHOULD verify a received kernel against its contract"). They are not used to state a wire or ABI requirement.
- Informative content does not introduce a requirement, and normative content does not rely on informative prose for its meaning. Informative-text-leaking-into-normative is a defect the per-sub-standard validation pass rejects.

This umbrella is entirely informative and therefore uses lowercase requirement-language throughout; the uppercase keywords appear here only where a sub-standard's clause is being named or quoted.

### 3.3 Clause-ID scheme

Every atomic normative requirement in a sub-standard carries a stable identifier of the form:

```
KISS-<SUB>-<section>-<nnnn>
```

where `<SUB>` is the sub-standard token (`ANNOUNCE`, `CLASSIFY`, `OPS`, `GRAMMAR`, `CONTRACT`, `SYNTH`, `CONSUME`, `EMIT`, `CONFORM`), `<section>` is the normative section number the clause lives under (for example `6.1`), and `<nnnn>` is a zero-padded ordinal allocated by that sub-standard's editor. Example: `KISS-ANNOUNCE-6.1-0004`.

Rules (each enforced by the sub-standards and by the KISS-Conform build):

- **Atomic.** Each clause states exactly one MUST / MUST NOT / SHALL. Compound requirements are split.
- **Append-only.** IDs are allocated in order and never reused after a clause is retired. A retired ID is permanently burned.
- **1:1 to a test.** Each clause ID maps to at least one named KISS-Conform test, and each such test cites the clause ID(s) it enforces. Traceability is bidirectional.
- **Build fails on an untested MUST.** The KISS-Conform suite build fails if any normative clause has no mapped test, and fails if any test cites a non-existent or retired clause ID.
- **Machine-readable sidecar.** Clause IDs and their metadata (section, keyword, determinism/fidelity class, mapped tests) live in a machine-readable sidecar kept in sync by a lint. For the plain-old-data vocabularies (Classify, Ops), both the prose tables and the sidecar are generated from the canonical schema so they cannot drift.
- **Determinism class declared.** Every numeric clause declares its determinism/fidelity class so KISS-Conform selects the correct comparator, drawn from the single canonical enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` owned by KISS-Ops (§6.0-0001), spelled verbatim.

### 3.4 Ban on unquantified adjectives

Normative text does not rely on unquantified qualitative adjectives such as "well-formed", "reasonable", "neutral", "appropriate", "efficient", or "valid" as the load-bearing requirement. Every normative requirement is pinned to observable facts: bits, IEEE-754 semantics, byte order, field offsets, sizes, alignment, counts, or an enumerated set. Where a term of art is needed it is defined in the sub-standard's Terms & Definitions and the normative clause references the definition.

### 3.5 Value pinning

Values in normative text are pinned as **bits and IEEE-754 semantics with endianness fixed**, never as one source language's surface spelling. Constant and non-finite values (positive and negative infinity, quiet and signaling NaN, positive and negative zero, subnormals) are specified by their bit patterns per dtype and round-trip exactly. Where a normative artifact is a wire byte sequence, its bytes are pinned in wire order (left to right) so a foreign reader reproduces them without inferring an endianness convention.

---

## 4. The dual-document template

Every KISS sub-standard is a single document in two halves with a fixed section structure. No sub-standard reorders, renumbers, or omits these sections.

### 4.1 Overview (informative) — §0–§5

- **§0 Front-matter.** Title; sub-standard ID; maturity stage (Draft / Frozen(date) / Deprecated / Retired); editor-of-record (marked "proposed, pending ratification" until recorded in the governance record, §7); steward (ThinkersJournal); reference seed crate(s) and reference-implementation pointer; DAG position (upstream and downstream edges, each labeled OPAQUE or STRUCTURAL, matching the umbrella §2.2 edge table). Project and product names may appear here as reference-impl pointers and provenance.
- **§1 Purpose & Scope.** What the sub-standard owns, plus a one-line "KISS-⟨X⟩ is **NOT**: …" exclusion drawn from §1.2 of this umbrella (self-policing scope).
- **§2 Overview / Rationale.** The human mental model, worked examples, and the reasoning behind the design choices. May use lowercase keywords freely. Project names may appear in examples.
- **§3 Terms & Definitions.** Every term of art the normative half relies on.
- **§4 Normative References.** External references (the language's `repr(C)`-equivalent layout guarantee, IEEE-754, byte order) and every upstream KISS sub-standard **by version**, each DAG edge labeled **OPAQUE** or **STRUCTURAL** consistently with the umbrella §2.2 edge table.
- **§5 Conventions.** A pointer to §3 of this umbrella (keywords + clause-ID scheme). States nothing new.

### 4.2 Conformance spec (normative) — §6–§10

- **§6 Specification.** Numbered atomic clauses, each a single MUST / MUST NOT / SHALL with a stable clause ID, values pinned as bits/IEEE-754 in wire order, and a declared determinism/fidelity class per numeric clause. For the plain-old-data vocabularies, this section is generated from the canonical schema.
- **§7 Capability, Profile & Extension model.** The mandatory core for this sub-standard; the negotiable options; the reserved ranges pinned to explicit integer bit indices; the version-negotiation algorithm; and, per field, whether an unknown value is a hard-gate reject or a reserved-and-ignored skip.
- **§8 Versioning & Lifecycle.** The two version axes (wire/ABI schema version versus published-crate semver) with a bump-versus-no-bump rule table; maturity entry and exit criteria; the freeze gate (§5.3 of this umbrella); and retire-by-floor deprecation.
- **§9 Conformance.** The claim format, the DAG prerequisite-closure rule, and the clause-ID-to-KISS-Conform-test traceability matrix (or its stub).
- **§10 Governance.** The editor-of-record, the ratifier, the license, the patent grant, and the mark-use tie — all by reference to this umbrella's §7 and §9.

### 4.3 Appendices (informative)

- **Appendix A onward.** Golden-vector references (each with a "bytes on the wire, left to right" row), migration recipes, and worked examples. Informative only.

---

## 5. Versioning & lifecycle

### 5.1 Two version axes

Every sub-standard carries **two independent version numbers** and does not conflate them:

- **Wire/ABI schema version** — an integer stamped in the wire format or ABI (for example a handshake envelope version or a structure-key version). It changes only when the observable bytes or the ABI change. It is the axis KISS-Conform keys conformance on.
- **Published-crate semver** — the ordinary semantic version of the reference crate(s). It changes on any code release, including changes that do not touch the wire (documentation, performance, additional helper APIs).

A bump-versus-no-bump rule table in each sub-standard's §8 states, per kind of change, which axis moves. A pure code refactor that preserves the bytes bumps only the crate semver; a layout change bumps the wire/ABI schema version (and, consequently, the crate semver).

### 5.2 Maturity stages

Each sub-standard is in exactly one maturity stage, recorded in §0 front-matter:

- **Draft** — under active design; the wire shape may change; no interop guarantee. This umbrella and all nine sub-standards are Draft.
- **Frozen(date)** — the wire/ABI schema version at the freeze date is stable; subsequent changes are additive under the capability model or require a new schema version. Entry requires passing the freeze gate (§5.3).
- **Deprecated** — superseded; still supported for a stated window (retire-by-floor: a minimum version implementors may still rely on).
- **Retired** — no longer supported; clause IDs remain burned and are never reused.

### 5.3 The freeze gate

A sub-standard advances Draft → Frozen only when all three conditions are met and demonstrated:

1. **At least two structurally dissimilar implementations interoperate on the golden vectors — counted per field, not per implementation.** For each field of a golden vector, at least two parties must **independently derive** that field's value. A party that receives the value as an input and reproduces it byte-exactly demonstrates **faithful passthrough** — a real and testable property, and not evidence of independent derivation of that field. Two implementations that share lowering code do not count as dissimilar; neither do two that share the *value*. The second case is the one a shared-lowering-code test cannot detect, because **a passed-through field has no lowering code to share**. A conformance report quoting a whole-vector count states, per field, whether each party **derived** or **copied** it.
2. **A non-native foreign reader consumes the wire.** A reader written outside the reference language reproduces or parses the exact bytes, with endianness, pointer width, and structure padding checked (the adversarial-outsider checklist).
3. **The sub-standard's KISS-Conform suite exists and passes**, with complete bidirectional clause-to-test traceability.

The **KISS-Conform AUDIT role signs the freeze transition** — not the authoring/design role. "Frozen" means "has a passing conformance gate and a demonstrated second dissimilar implementation from the document alone," never merely "the authors declared it stable." A sub-standard citing this gate refers to it as "umbrella §5.3" so the reference resolves from the public documents alone.

---

## 6. Capability, profile & extension model

### 6.1 Mandatory core per sub-standard

Each sub-standard defines a **mandatory core**: the clauses every conforming implementation of that sub-standard must satisfy regardless of which options it claims. An implementation that cannot satisfy the mandatory core does not conform to that sub-standard at all. Un-claimed inputs produce a **typed decline, never a panic** — each sub-standard owns this obligation as an identified clause with a mapped KISS-Conform test.

### 6.2 The capability bitset

Capability negotiation uses a **u64 bitset split into three axes**. Reserving the ranges now — while single-ownership makes reservation free — prevents the flat-bitset bit-sprawl trap:

- **Axis 1 — which sub-standards I speak.** Bits identifying the KISS sub-standards an implementation supports. The wire/ABI schema version a side speaks for each sub-standard is resolved through profile negotiation (below), not encoded one bit per version.
- **Axis 2 — optional features.** Bits for negotiable, non-core features within the sub-standards claimed on axis 1 (for example "supports contract-query", "does just-in-time build-on-miss").
- **Axis 3 — external data tokens.** Bits identifying external interchange tokens an implementation understands (for cross-ecosystem data-descriptor interop). Each such bit pins only the axis and range; the *meaning* of the external token is defined by the external-token registry (§6.4), not owned by KISS.

Within each axis, ranges are partitioned into **core** (assigned by the sub-standard editor, stable), **experimental** (for in-development features), and **vendor** (for a single namespace's private use). Each sub-standard pins these sub-ranges to explicit integer bit indices per axis; the ranges are never expressed as computed fractions.

**Forward-compatibility rule.** Capability bits are **advertisement only**: they announce what a side supports and do not by themselves hard-gate a session. A receiver treats an unknown bit in any axis as reserved-and-ignored — that is, as absent — and never panics on one. Hard-gating on an unsupported peer happens exactly once, at **profile/version negotiation**: the parties compute the highest mutually-supported profile, and an empty intersection is a hard-fail (never a panic). This keeps the rule unambiguous — unknown advertisement bits are always ignored, and rejection is expressed at the negotiation step, not per bit.

### 6.3 Claimable DAG-aligned subsets with prerequisite closure

An implementation claims a **DAG-aligned subset** of the suite. A claim is **prerequisite-closed**: claiming a sub-standard requires claiming every sub-standard on its incoming **STRUCTURAL** edges (an incoming **OPAQUE** edge requires agreement on the meaning of the exchanged token, not on the upstream sub-standard's internal structure, and so does not force a claim). The umbrella §2.2 edge table is authoritative for which edges are structural. KISS-Conform tests both that the claim's own clauses pass **and** that inputs outside the claim produce typed declines rather than panics.

### 6.4 Extension registry and promotion path

Optional features and external tokens are recorded in an **extension registry under ThinkersJournal**, PR-gated. The lifecycle is **experimental → arbitrated → core**: a feature enters as an experimental-range extension owned by its proposer, may be arbitrated into a shared/interoperable extension, and — once at least two dissimilar implementations use it and it has a conformance test — may be promoted into the core range of the owning sub-standard by that sub-standard's editor. Promotion is additive and never renumbers existing core bits. External-token (Axis 3) bits are governed here: the registry is the single source of truth for which external interchange token a bit denotes, is versioned, and each KISS clause pins only the bit range, deferring the token's semantics to its external standard. The all-hardware target-capability descriptor uses a parallel **namespace registry**: ThinkersJournal registers namespaces; each namespace's maintainer owns that namespace's capability-set vocabulary; tokens match byte-exact on the full string.

#### 6.4.1 Extension-registry entries (record)

The registry of record is this table. Each entry is PR-gated (§7.2) and names the owning sub-standard's clause(s) that pin the token's wire/semantics; the registry itself pins only the lifecycle stage, the tag/bit range, the owner, and the promotion path. An entry is **additive** and never renumbers an existing core token.

| Token | Owning sub-standard | Tag / range | Lifecycle stage | Owner | Consumers / promotion path | Reference |
| --- | --- | --- | --- | --- | --- | --- |
| `dims(...)` / `Dims` | KISS-Ops §6.20 shape-expression vocabulary | `ShapeExpr` tag `0x0B` (experimental range of the §6.20-0005 tag space) | **experimental** | Fuel | Promotion to §6.20 core requires ≥2 dissimilar implementations + a conformance test: **Fuel** (byte-matches the minted goldens) + the **KISS conformance reference** (`conformance/src/shape_expr.rs`, goldens in `conformance/tests/shape_expr.rs`). Declared future consumer: Baracuda (pooling/conv output shapes). | KISS-Ops §6.20-0010; RFC #80 |
| `with_dim(...)` / `WithDim` | KISS-Ops §6.20 shape-expression vocabulary | `ShapeExpr` tag `0x0A` (experimental range of the §6.20-0005 tag space) | **experimental** | Fuel | Promotion to §6.20 core requires ≥2 dissimilar implementations + a conformance test: **Fuel** (byte-matches the minted goldens) + the **KISS conformance reference**. Declared future consumer: Baracuda (rank-polymorphic conv/qmatmul output shapes). | KISS-Ops §6.20-0009; RFC #80 |

Both entries activate the two §6.20-0005 tags that were allocated-reserved before RFC #80; the still-reserved `Reduce` (`0x09`) is **not** registered (no consumer). The canonical functional spellings are `dims(...)` and `with_dim(...)` (Baracuda cosign pin, RFC #80 comment 5060433119), pinned alongside the wire tags in the owning clauses so text-emitters and byte-emitters converge on one spelling.

---

## 7. Governance

### 7.1 Roles

- **Steward — ThinkersJournal.** Hosts the specification text, the extension and namespace registries, the free-certification registry, and the conformance-suite distribution. The steward does not author sub-standards, does not arbitrate technical disputes within a sub-standard, and does not police conformance claims. Custody is held in trust for the interested-cosignatory community; when external parties join, the non-profit is formalized and org/crate ownership transfers to it.
- **Editor-of-record — one per sub-standard.** Holds the pen for that sub-standard: allocates clause IDs, integrates ratified changes, and signs maturity-stage advances jointly with the KISS-Conform AUDIT role. There is a single editor per sub-standard, not design-by-committee. Editor assignments are recorded in the governance record; a sub-standard whose editor is not yet ratified marks its §0 editor field "proposed, pending ratification."
- **Interested cosignatories.** Projects and parties that build consumers, providers, emitters, or otherwise depend on a sub-standard. (A *cosignatory* is a co-signer of the suite's governance — a stakeholder with standing to comment and vote; the term is not "consignatory," which concerns consignment of goods.) They receive comment and vote on proposed changes to the sub-standards that affect them. An editor requests comment from affected cosignatories before deciding a cross-party-visible change.

### 7.2 The RFC process

Every surfaced ambiguity or proposed change becomes a **numbered RFC** — a GitHub issue labeled `rfc` in the KISS repository issue tracker, which is the ThinkersJournal RFC directory of record. The flow is propose-first: a change is floated to the affected cosignatories on the issue before it is wired; an accepted change is authored as a pull request that cites the issue and adds the clause text together with its mapped KISS-Conform test; cross-party-visible version bumps (any change to a wire/ABI schema version) are coordinated across affected parties. The issue is public and remains the durable account of why each decision was made — whether the RFC is accepted or declined. The contribution guide's *RFC lifecycle* section states the per-state mechanics; this clause owns the propose-first governance obligation.

### 7.3 Advancing a maturity stage

A sub-standard advances a maturity stage when its stage-transition criteria (§5.2, and for freeze §5.3) are met and **the transition is signed by the KISS-Conform AUDIT role**, not by the authoring editor alone. The audit confirms traceability is complete, that a second dissimilar implementation was built from the document alone, that the adversarial-outsider checklist passed, and — for a freeze — that the full freeze gate is met. The signed transition is recorded in §0 front-matter and in the RFC record.

---

## 8. Conformance model

### 8.1 Conformance is factual

An implementation **conforms** to a sub-standard at a given wire/ABI schema version if and only if it passes the **unmodified** KISS-Conform suite for that sub-standard at that version. Conformance is a factual, testable property, not a matter of assertion or endorsement. There is no partial credit for a claimed subset beyond what the suite verifies: the claim's own clauses pass and out-of-claim inputs decline cleanly.

### 8.2 Self-certification with published results

The default path is **self-certification**: an implementor runs the unmodified suite and publishes the results. Publishing results is the norm; the results are the evidence.

### 8.3 The steward-maintained free-certification registry

ThinkersJournal maintains a **registry of verified implementations**. An implementor may request certification; the steward runs the unmodified suite (as resources permit, free of charge) and, on a pass, lists the implementation on a "steward-certified implementations" list. The registry is the authoritative record of verified implementations.

### 8.4 No claim-policing

KISS does **not** police who calls themselves "KISS-conformant." A false claim self-reveals: non-conforming software fails to interoperate and does not appear on the registry, so value accrues to *being listed*, not to the assertion. A registered certification mark is an optional future lever, not a v1 requirement. The one operative rule — the mark policy (§9.3) that results from a **modified** conformance suite do not back a conformance claim — is enforced not by policing but through its **only enforcement surface in v1**: such results are ineligible for the steward's registry. The certification mark, if and when registered, becomes a second lever; until then, registry ineligibility is the mechanism.

---

## 9. Legal

### 9.1 Specification text — CC0 1.0 Universal (public-domain dedication)

The KISS specification text (this umbrella and every sub-standard) is dedicated to the public domain under **Creative Commons CC0 1.0 Universal**. Anyone may copy, modify, distribute, and implement the specification — including for any commercial purpose — without permission or attribution. CC0 waives copyright and related rights only; it does **not** grant patent rights, which are addressed separately in §9.4. The zero-friction dedication is deliberate: a standard whose text anyone may reproduce and re-host is harder to fork through enclosure and cheaper to adopt.

### 9.2 Reference crates — MIT OR Apache-2.0

The KISS reference implementation crates are licensed **MIT OR Apache-2.0** (dual-licensed, implementor's choice). The Apache-2.0 option carries the patent grant that the MIT license lacks.

### 9.3 Conformance suite — permissive to run, with a mark policy

The KISS-Conform suite is licensed permissively so that anyone may run it. A **mark policy** applies: a conformance claim is backed only by results from the **unmodified** suite. Results from a modified suite do not back a conformance claim and are not eligible for listing on the steward's registry. As §8.4 notes, this policy is enforced through registry eligibility (and the eventual certification mark), not through claim-policing.

### 9.4 Patent — royalty-free grant with defensive termination

Each contributor grants, on contribution to a KISS RFC, a **royalty-free license to its essential patent claims** necessary to implement the contributed specification text. The grant is subject to **defensive termination**: the license to a party terminates if that party initiates patent litigation asserting that a conforming implementation of KISS infringes its patents. The grant is bound at contribution time, while rights are still consolidated, because it cannot be retrofitted onto external signatories later.

---

## 10. Security considerations

This section is informative, like the rest of this umbrella: it states the suite's trust model and scope. The binding requirements it calls for live, as always, in sub-standard clauses — several of which **do not exist yet** and are enumerated in §10.5 as work to be ratified. Until they do, this section is the honest statement of a gap rather than a description of a defence.

### 10.1 KISS is a code-distribution protocol

This is the fact from which everything else in this section follows, and the suite did not state it before Umbrella v0.1.

KISS-Synth/Provision moves a **kernel artifact** — native executable code (for example PTX, cubin, SPIR-V, or an object file) — from a provider to a consumer, which then **loads and launches it on its own hardware, inside its own address space**. The consumer runs code it did not write, did not compile, and cannot generally inspect, obtained from a party on the other side of a vendor boundary.

Every other seam in the suite is data; this one is control. A protocol that hands over executable code inherits the whole trust model of a software supply chain, whether or not it says so. KISS previously said nothing, and silence is not a scope exclusion (§1.2: "scope creep by silence is a named trap" — the trap runs in both directions).

### 10.2 What KISS does not provide

KISS provides **none** of the following, and an implementor must not infer otherwise from any field name:

- **No authentication.** Nothing in the handshake establishes who the peer is. A conforming provider is any process that emits a well-formed envelope.
- **No integrity.** The contract's inner checksum (KISS-Contract §6.2) detects *accidental* corruption and truncation. **A checksum is not a signature.** It is computed by whoever wrote the bytes, so it is recomputed by anyone who alters them. It stops a truncated pipe, not an adversary.
- **No content binding of identity.** `revision_hash` (KISS-Announce §6.3) is a 32-byte, provider-assigned, opaque label. Despite its name and its SHA-256-shaped width, no clause requires it to be a hash *of* anything, and §6.3 forbids a consumer from assuming it is. **Revision-pinning therefore does not detect artifact substitution:** a provider — or anything positioned as one — may return different bytes under an unchanged `revision_hash` and remain conformant. Pinning a revision expresses a cache-coherence intent, not a security guarantee.
- **No confidentiality.** Operand descriptors, `structure_key`s, and contracts describe a consumer's model architecture in detail and cross the wire in the clear.
- **No sandboxing or isolation.** KISS does not constrain what a received artifact may do once launched. It runs with the consumer's privileges.
- **No provenance or supply-chain attestation.** The Contract's Provenance section (§6.10) is a self-asserted record by the party that produced the kernel. It is a claim, not evidence.
- **No transport security.** See §10.3.

### 10.3 The transport is out of scope — and so, therefore, is transport security

KISS pins the **bytes** of each message, not what carries them. It says nothing about sockets, pipes, shared memory, or RPC. This is deliberate and consistent with §1.2's exclusion of in-ecosystem mechanism.

The consequence must be stated rather than left implicit: **authentication, confidentiality, and integrity of the channel are the deployment's responsibility, not KISS's.** A deployment that carries KISS frames across a trust boundary is responsible for placing them inside a channel that provides those properties (a mutually-authenticated transport, a signed package format, a verified OCI layer, an OS IPC channel with peer credentials — the choice is the deployment's). KISS's frames are designed to be carried inside such a channel; they are not designed to survive without one.

**This delegation is only honest if it is loud.** Two of the three deployment shapes the suite anticipates put the parties in different trust domains:

| Deployment | Trust relationship | Who secures the channel |
|---|---|---|
| Provider linked in-process | same trust domain as the consumer | nobody needs to; there is no channel |
| Provider as a local subprocess | same host, possibly different vendor | the OS (peer credentials, process isolation) |
| Provider over a network | **different trust domain** | **the deployment — KISS provides nothing** |

A consumer that speaks KISS to a remote provider over an unauthenticated channel is accepting arbitrary code execution from anyone who can reach the socket. That is a true statement about the protocol as specified, and it belongs in this document rather than in an incident report.

### 10.4 What a consumer is actually trusting

A consumer that launches a provided artifact is trusting, with no protocol-level recourse:

1. that the peer is the provider it believes it is (unauthenticated — §10.2);
2. that the artifact is the code the provider built, unmodified in transit (no integrity — §10.2);
3. that the artifact computes what its contract says (unverified — the contract is the provider's own claim; KISS-Conform's differential is a *voluntary* check the consumer may run, and §10.5 item 5 proposes making that recourse explicit);
4. that the artifact does nothing besides compute (unconstrained — no sandboxing);
5. that the provider is not hostile.

Item 5 is doing all the work. **KISS's security model today is "trust the provider."** That is a legitimate model for a provider linked into your own process. It is not one for a provider reached over a network, and the suite currently draws no distinction between the two.

### 10.5 Required work — normative clauses that do not yet exist

The following are gaps, not features. Each needs a clause in the named sub-standard, a mapped test, and removal from `conformance/UNBACKED.tsv` once real. They are ordered by how cheaply they remove a real hazard:

1. **Bound every attacker-controlled length (KISS-Synth, KISS-Contract, KISS-Announce).** `artifact_len` is a u64 supplied by the *less-trusted* party and no clause caps it or forbids allocating on it before the bytes arrive. Every never-panic, no-out-of-bounds, and no-unbounded-allocation obligation in KISS-Synth (§1, §2.4, §9) is written against the **request** direction — the provider's inputs. The **response** direction, the one carrying executable code, binds nobody. A conforming consumer can be OOM-killed by a two-line frame. Fixing this is a per-field cap plus one clause per length; it is the cheapest hazard removal in the suite. Related: issue #24, which asks for a consumer-side never-panic obligation — necessary, but it addresses *crashing on a malformed artifact*, not *executing a hostile one*.
2. **State the trust boundary normatively (KISS-Synth §1).** A one-line scope clause: a conforming consumer MUST treat a provided artifact as untrusted code unless the provider is in its own trust domain, and the deployment MUST provide channel authentication and integrity where it is not.
3. **Say what `revision_hash` is not (KISS-Announce §6.3).** A clause stating that `revision_hash` is not an integrity mechanism and MUST NOT be relied on to detect substitution, so its name stops implying a guarantee it does not carry. Pairs with issue #26, which reopens the field's encoding — the two should land together.
4. **Say what the contract checksum is not (KISS-Contract §6.2).** As above: detects corruption, not modification.
5. **Define an optional signing/attestation extension (registry, umbrella §6.4).** The capability model's Axis 2 (optional features) is the natural home, and the extension registry exists precisely for this. This does not belong in the mandatory core — a linked in-process provider should not pay for it — but the *extension point* must exist, and adding one after a freeze is far more expensive than reserving it now. This is the item most worth doing before any sub-standard freezes.
6. **Give the consumer's verification recourse a name (KISS-Conform).** A consumer that differentials a received kernel against the oracle before trusting it (Fuel's ledger-with-downgrade-to-UNAUDITED, per its comment on issue #16) is doing the only thing in this list that catches a *wrong* kernel rather than a *malformed* one. That pattern should be a named, optional profile rather than folklore.

### 10.6 Non-goals

KISS does not intend to become a code-signing standard, a package format, or a transport. Where an existing standard owns a problem — Sigstore/in-toto for provenance, TLS/Noise for channels, OCI for distribution, the platform loader for isolation — KISS's job is to leave a **clean extension point** and name the owner, not to re-solve it. §10.5 item 5 is that extension point; the rest of this section names the owners.

---

*End of KISS Umbrella Specification (Draft proposal, Umbrella v0.1). This umbrella is informative throughout; every binding requirement lives in a sub-standard's identified clause with a mapped KISS-Conform test. Project and product names appearing in any sub-standard are confined to non-normative examples, provenance and acknowledgments, reference-implementation pointers, and the governance/signatory record; normative clauses use only the generic roles provider, consumer, implementation, kernel, contract, and target.*