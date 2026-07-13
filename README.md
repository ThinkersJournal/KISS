# KISS — Kernel Interface Standards Suite

**Free, open, vendor-neutral standards for the interface between machine-learning
libraries, compute libraries, and kernel providers.**

Stewarded by [ThinkersJournal](https://github.com/ThinkersJournal).

> ## ⚠️ Status: pre-1.0 DRAFT
>
> Nothing in this repository is frozen. Every document here is a **draft proposal**
> open to comment and revision. Clause numbers, wire formats, and vocabularies
> **will** change before 1.0. Do not build a production integration against these
> drafts expecting stability — build against them to *inform* the drafts.

---

## What KISS is

Today, every ML framework, compute library, and kernel generator re-invents private
glue for every pairing it supports. KISS standardizes the *seams* between three kinds
of software so that glue is written once, in the open:

- **Kernel providers** — libraries and just-in-time generators that build or ship
  compute kernels.
- **Kernel consumers** — ML and compute frameworks that need a kernel for a given
  operation on given data.
- **Tooling** — that recognizes, generates, or verifies kernels.

KISS is an *interface* standard. It does **not** dictate how a kernel is compiled,
scheduled, or executed, nor does it define a source language, an ABI for any specific
hardware, or a runtime. It standardizes how kernels are **described, discovered,
requested, and verified** across a vendor boundary.

Rather than one monolithic document, KISS is a **set of interrelated sub-standards
with a strict dependency DAG**. An implementation adopts only the subset it needs: a
provider that merely announces pre-built kernels implements far less than a
just-in-time generator, and neither is forced to adopt the parts it does not use.

## The nine sub-standards

| Sub-standard | Tier | Owns | Status |
|---|---|---|---|
| **KISS-Classify** | foundational | The vocabulary that describes *data*: dtypes, operand descriptors, layout/op-family tags, the specialization-cell identity (`structure_key`), and the all-hardware target-capability descriptor. | **draft — [`spec/classify.md`](spec/classify.md)** |
| **KISS-Ops** | foundational | The vocabulary that describes *computation*: the op set, each op's pinned numeric semantics, each non-primitive op's reference decomposition, and the mandatory **primitive floor**. | **draft — [`spec/ops.md`](spec/ops.md)** |
| **KISS-Grammar** | middle | The advertisable-op surface: how an op tag maps to a KISS-Ops op name plus pattern/synthesis attributes and an operand-role tuple, the region wire form, and how a frozen grammar admits a still-growing op set. | **draft — [`spec/grammar.md`](spec/grammar.md)** |
| **KISS-Contract** | middle | The universal, vendor-neutral **kernel-contract** format — the seven-section document (identity, semantics, interface/ABI, dispatch, capabilities, guarantees, provenance) that tells a consumer what a kernel computes and exactly how to call it. Every provided kernel carries one. | **draft — [`spec/contract.md`](spec/contract.md)** |
| **KISS-Announce** | protocol | The provider handshake and availability protocol: the fixed-layout, little-endian handshake envelope; version negotiation; the split capability bitset; the identity-only availability list; and the contract-query. | **draft — [`spec/announce.md`](spec/announce.md)** |
| **KISS-Synth / Provision** | protocol | The kernel-provision protocol: a consumer asks for a kernel by identity and receives `{artifact, contract}`, the provider building it on a cache miss. Just-in-time synthesis is the build-on-miss branch of the same request. | planned |
| **KISS-Consume** | protocol | The recognition direction: lifting a kernel or source region into the op DAG as far as it goes, with a normative refusal taxonomy for the un-liftable remainder. | planned |
| **KISS-Emit** | protocol | The generation direction: a complete partition of every lowering decision into "the neutral driver may spell it" versus "the emitter must supply it," plus the emit/consume round-trip tiers. | planned |
| **KISS-Conform** | cross-cutting | Conformance: the bidirectional clause-to-test traceability matrix, the four test modalities, determinism-class-aware comparators, and the adversarial-outsider checklist that gates every freeze. | planned |

The two **foundational vocabularies** (Classify = data, Ops = computation) sit at the
bottom; everything else references them. See
[`spec/umbrella.md` §2.2](spec/umbrella.md) for the authoritative dependency DAG and
edge-label table.

```
        FOUNDATIONAL          Classify (data)      Ops (computation)
                                    │                    │
        MIDDLE                      └──► Grammar ──► Contract ◄──┘
                                                 │
        PROTOCOL      Announce ◄─(opaque)────────┤
                      Synth/Provision ◄──────────┤
                      Consume ◄──────────────────┤
                      Emit ◄─────────────────────┘
                                                 │
        CROSS-CUTTING             Conform (depends on & tests ALL nine)
```

An **opaque** edge (e.g. Classify → Announce, Contract → Announce) means the depending
sub-standard carries the other's payload as an uninterpreted, length-delimited token —
it never parses the internals. A **structural** edge means the depending sub-standard
reads and reasons about the referenced structure. This opaque/structural split is what
keeps the wire protocols stable while the vocabularies grow.

## Repository layout

| Path | Contents |
|---|---|
| [`spec/umbrella.md`](spec/umbrella.md) | The suite umbrella: purpose/scope, the nine sub-standards + DAG, conventions, the dual-document template, versioning/lifecycle + freeze gate, the capability/profile/extension model, governance, the conformance model, and legal. Informative throughout — every binding requirement lives in a sub-standard clause. |
| [`spec/announce.md`](spec/announce.md) | **KISS-Announce**: the handshake, availability, and contract-query protocol, with numbered normative clauses each mapped to a conformance test. |
| [`spec/classify.md`](spec/classify.md) | **KISS-Classify**, the foundational *data* vocabulary: the dtype set, operand descriptors, the `structure_key` specialization-cell identity, and the all-hardware `<namespace>:<capability-set>` target descriptor. |
| [`spec/ops.md`](spec/ops.md) | **KISS-Ops**, the foundational *computation* vocabulary: the op set, pinned per-op numeric semantics, reference decompositions, the mandatory primitive floor, and the canonical determinism/fidelity enum. |
| | `LICENSE` | CC0 1.0 Universal (see [License](#license)). |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to comment and contribute; governance and RFC process; contributor licensing terms. |

## How to read a KISS sub-standard

Every sub-standard follows one **dual-document template** (umbrella §4):

- **§0–§5 are informative** — overview, motivation, model, and examples you can read
  front-to-back to understand the sub-standard.
- **§6 onward is normative** — the tight, testable conformance spec. Each requirement
  carries a clause ID of the form `KISS-<SUB>-<section>-<nnnn>` and maps 1:1 to a named
  conformance test. If a MUST has no test, the suite build fails.

Read the informative half for understanding; implement against the normative half.

## Conformance

Conformance is a **factual, self-certified** claim backed by published results from the
**unmodified** KISS-Conform suite. The steward maintains a free registry of results.
KISS does not police claims made off-registry; the registry (and an eventual
certification mark) is where a claim is made checkable. See umbrella §8.

## Governance & contributing

KISS is developed in the open under a lightweight RFC process, stewarded by
ThinkersJournal. Each sub-standard has an **editor of record**; substantive changes go
through an RFC that interested parties may co-sign and comment on; advancing a maturity
stage (Draft → Frozen) requires passing the **freeze gate** — at least two dissimilar
independent implementations plus an adversarial-outsider review. See umbrella §5.3 and
§7, and [`CONTRIBUTING.md`](CONTRIBUTING.md).

Comment and proposals are welcome now, while the drafts are still soft. Open an issue or
a pull request.

## License

- **Specification text** (this repository) — dedicated to the public domain under
  **[CC0 1.0 Universal](LICENSE)**. Anyone may copy, modify, distribute, re-host, and
  implement it, for any purpose, without permission or attribution. CC0 waives copyright
  and related rights only; patent rights are addressed separately (umbrella §9.4).
- **Reference implementation crates** (separate repositories) — MIT OR Apache-2.0.
- **KISS-Conform suite** — permissive to run; a conformance claim is backed only by
  results from the unmodified suite.

---

*KISS is a work in progress. It is being drafted with a design → validate → adversarial-audit
process, and the first-draft documents in this repository have already survived one such
pass — but they are first drafts. Expect them to change.*
