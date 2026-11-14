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

## Why the key is a join key, not a cache key

Almost every compute library already has a *cache key*: cuBLASLt's algo, cuDNN's engine config, oneDNN's
`primitive_desc`, Composable Kernel's template tuple, Triton's hash of source plus compiler plus environment,
Inductor's graph-cache key. KISS's `structure_key` is a different kind of object, and the difference is the
reason it exists (the full argument is [`PRIOR-ART.md`](PRIOR-ART.md) §3):

| | Cache key | Join key (KISS) |
|---|---|---|
| Computed | **from the artifact**, after the build | **from the need**, before any artifact exists |
| By whom | the party that owns the artifact | a party that may never see the build |
| Wrong answer costs | a false hit is a silent wrong result, a false miss only a rebuild, so it is **over-specified on purpose** | a false miss means two parties that should have met did not, so it is **under-specified on purpose** |
| Goal | never reuse the wrong build | two parties who share no code, no version and no vendor derive the **same bytes** |

**Why KISS chose a join key.** A cache key cannot be derived without the artifact, so it can never be agreed
between a consumer asking for a kernel and a provider that has not built it yet, and it cannot list a build
matrix ahead of time. A join key is a predicate over the *need* (dtype, layout class, target, iteration
rank, reduced axes), carrying no extents, so the set of cells a provider can build is **finite and
publishable**, and a consumer can derive the same token at run time from an invocation the provider never sees.
The first buyer is a provider who wants to publish *what* it built without publishing *how*. **What it costs:**
the key is deliberately silent on the computation, so a byte-match is **necessary, not sufficient** for
admission (KISS-CLASSIFY-6.6-0001), and two kernels in one cell are not interchangeable
(KISS-CLASSIFY-6.6-0020); the op identity travels beside the key.

## The nine sub-standards

| Sub-standard | Tier | Owns | Status |
|---|---|---|---|
| **KISS-Classify** | foundational | The vocabulary that describes *data*: dtypes, operand descriptors, layout tags, the derived cell class, the specialization-cell identity (`structure_key`), and the all-hardware target-capability descriptor. | **draft — [`spec/classify.md`](spec/classify.md)** |
| **KISS-Ops** | foundational | The vocabulary that describes *computation*: the op set, each op's pinned numeric semantics, each non-primitive op's reference decomposition, and the mandatory **primitive floor**. | **draft — [`spec/ops.md`](spec/ops.md)** |
| **KISS-Grammar** | middle | The advertisable-op surface: how an op tag maps to a KISS-Ops op name plus pattern/synthesis attributes and an operand-role tuple, the region wire form, and how a frozen grammar admits a still-growing op set. | **draft — [`spec/grammar.md`](spec/grammar.md)** |
| **KISS-Contract** | middle | The universal, vendor-neutral **kernel-contract** format — the seven-section document (identity, semantics, interface/ABI, dispatch, capabilities, guarantees, provenance) that tells a consumer what a kernel computes and exactly how to call it. Every provided kernel carries one. | **draft — [`spec/contract.md`](spec/contract.md)** |
| **KISS-Announce** | protocol | The provider handshake and availability protocol: the fixed-layout, little-endian handshake envelope; version negotiation; the split capability bitset; the identity-only availability list; and the contract-query. | **draft — [`spec/announce.md`](spec/announce.md)** |
| **KISS-Synth / Provision** | protocol | The kernel-provision protocol: a consumer asks for a kernel by identity and receives `{artifact, contract}`, the provider building it on a cache miss. Just-in-time synthesis is the build-on-miss branch of the same request. | **draft — [`spec/synth.md`](spec/synth.md)** |
| **KISS-Consume** *(Recognition / Lift)* | protocol | **Despite the name, this is not "the consumer's sub-standard" — a provider's kernel-decomposer is a lifter too.** The recognition direction: lifting a kernel or source region into the op DAG as far as it goes, with a normative refusal taxonomy for the un-liftable remainder. | **draft — [`spec/consume.md`](spec/consume.md)** |
| **KISS-Emit** | protocol | The generation direction: a complete partition of every lowering decision into "the neutral driver may spell it" versus "the emitter must supply it," plus the emit/consume round-trip tiers. | **draft — [`spec/emit.md`](spec/emit.md)** |
| **KISS-Conform** | cross-cutting | Conformance: the bidirectional clause-to-test traceability matrix, the five test modalities, determinism-class-aware comparators, and the adversarial-outsider checklist that gates every freeze. | **draft — [`spec/conform.md`](spec/conform.md)** |

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
| [`spec/ops.md`](spec/ops.md) | **KISS-Ops**, the foundational *computation* vocabulary: the op set, pinned per-op numeric semantics, reference decompositions, the mandatory primitive floor, the canonical determinism/fidelity enum, and the OpAttrs channel + canonical default-resolved wire encoding. |
| [`spec/grammar.md`](spec/grammar.md) | **KISS-Grammar**, the advertisable-op surface + region grammar: op-tag → KISS-Ops name + attrs + operand-role tuple, the pinned region wire form, and the frozen-grammar/growing-op-set rule. |
| [`spec/contract.md`](spec/contract.md) | **KISS-Contract**, the universal 7-section kernel contract (identity, semantics, interface/ABI, dispatch, capabilities, guarantees, provenance) — what a kernel computes and exactly how to call it; self-delimiting fail-loud transport. |
| [`spec/synth.md`](spec/synth.md) | **KISS-Synth / Provision**, the kernel-provision protocol: ask by identity → `{artifact, contract}` (built on a cache miss); the PRSP provision-success frame; the never-panic typed-decline taxonomy. |
| [`spec/consume.md`](spec/consume.md) | **KISS-Consume** *(Recognition / Lift)*, the recognition direction — **not the consumer's sub-standard; the role is orthogonal to provider/consumer**: structure-based lifting into the KISS-Ops op DAG, the MECE refusal taxonomy, and the residue / lift-fraction. |
| [`spec/emit.md`](spec/emit.md) | **KISS-Emit**, the generation direction: the complete driver-may-spell vs emitter-must-supply lowering partition, and the two-tier emit/consume round-trip. |
| [`spec/conform.md`](spec/conform.md) | **KISS-Conform**, the cross-cutting conformance sub-standard: the bidirectional clause↔test traceability matrix, the five test modalities, determinism-class comparators, and the adversarial-outsider freeze gate. Tests all eight others. **What CI actually enforces of this is stated in [How coverage is enforced](#how-coverage-is-enforced); it is a ratchet, not "the build fails on any untested MUST".** |
| `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`, `REUSE.toml` | The licence texts and the machine-readable map of which applies where (see [License](#license)). |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to comment and contribute; governance and RFC process; contributor licensing terms. |
| [`DESIGN.md`](DESIGN.md) | Design rationale (informative): the cross-cutting commitments, the alternatives weighed and set aside, and how the suite is authored and advanced. |
| [`PRIOR-ART.md`](PRIOR-ART.md) | Prior art (informative): KISS measured against StableHLO/PJRT, ONNX, Triton/MLIR, PyTorch, DLPack, SPIR-V/IREE, the vendor kernel libraries, and the Khronos ULP/CTS precedent — what they already solve, what they do not, and the one claim that survives. **Read this before implementing.** |
| [`WIRE-FIRST.md`](WIRE-FIRST.md) | A work order (informative): the 38 confirmed blockers between here and one real handshake+provision across a process boundary, what is already green, and what each implementation can do now. |
| [`conformance/UNBACKED.tsv`](conformance/UNBACKED.tsv) | The normative clauses with no executable harness test (506 at `e84b36a`). The honest coverage record; its floor is a ratchet enforced by `tools/kiss_trace.py --ratchet` in CI (see [How coverage is enforced](#how-coverage-is-enforced)). |

## How to read a KISS sub-standard

Every sub-standard follows one **dual-document template** (umbrella §4):

- **§0–§5 are informative** — overview, motivation, model, and examples you can read
  front-to-back to understand the sub-standard.
- **§6 onward is normative** — the tight, testable conformance spec. Each requirement
  carries a clause ID of the form `KISS-<SUB>-<section>-<nnnn>` and maps 1:1 to a named
  conformance test.

Read the informative half for understanding; implement against the normative half.

## How coverage is enforced

> **Test coverage — a clause↔test *mapping*, not a passing rate.** Measured at `e84b36a`
> (`python tools/kiss_trace.py --report`): of **1043** normative clauses, **537** (51.5%) are backed by a named
> executable test and **44** more are enforced by a document lint, **581 (55.7%)** in all; **391 are
> genuinely untested**, and the rest of the **506** unbacked clauses (71) are recorded in
> [`conformance/UNBACKED.tsv`](conformance/UNBACKED.tsv) as blocked, untestable, definitional or attested.
> `tools/kiss_trace.py` reads spec markdown and Rust source and never builds or runs the crate, so a named
> test is known to **exist**, not to compile, run or pass (that is `cargo test`, run separately by the
> `conformance` workflow). Treat a clause as *proposed* unless it appears outside the ledger.

**What the tooling actually enforces, and what it does not** (each line names its tool or workflow, in
[`.github/workflows/traceability.yml`](.github/workflows/traceability.yml) unless stated):

- **The build does *not* fail on an untested MUST.** 391 of them exist today and CI is green. The strict
  reading (every MUST backed, `kiss_trace.py --strict`) is run **non-blocking**; the job instead
  **asserts the tolerated red is the known one** (`kiss_trace.py --assert-known-red`, `kiss_strict_report.py`).
- **What fails the build is a regression:** `kiss_trace.py --ratchet` (the *Coverage ratchet (blocking)*
  step) compares the live figures to [`conformance/COVERAGE_FLOOR.tsv`](conformance/COVERAGE_FLOOR.tsv)
  and to the base branch's ledger, and fails if backed clauses fall, untested clauses rise, or the lint set
  changes in a way that would launder a regression (#213). "Green" means *at* the floor, never under it.
- **The ledger is a record, not an exemption:** a stale `UNBACKED.tsv` or floor fails the same ratchet.
- **The figures quoted in `conformance/README.md` are checked** (`kiss_readme_coverage.py`, blocking). The
  figures in **this** README are a stamped snapshot and are **not** checked: a figure without its commit is not
  a measurement.
- **Lint-enforced clauses (44)** are credited only where a `tools/kiss_*.py` linter emits coverage for them
  (`--emit-coverage`, discovered by `kiss_trace.py`); such a credit says a document lint checks the clause,
  not that a test passed.
- **The freeze gate is only partly automated.** `tools/kiss_freeze_gate.py` (*Frozen sub-standard passes
  --freeze-ready*, blocking) fails the build if a sub-standard marked **Frozen** is not fully traced
  (umbrella §5.3 condition 3). Every sub-standard is Draft today, so this gate has never had to block a real
  promotion (its controls seed the red). It does **not** check the two independent implementations, the
  adversarial-outsider review or the three signatures (umbrella §5.3, tracked in #462): those remain human
  process.
- **Licence headers:** `tools/kiss_spdx.py --check` (*Every code file carries its SPDX header*, blocking).
- **Not checked by CI:** that any third party's conformance claim is true (see below), and the corpus files'
  sha256 pins recorded in [`conformance/corpus/PROVENANCE.md`](conformance/corpus/PROVENANCE.md); some
  corpus artifacts are checked for staleness by regeneration tests (for example
  `test_derivation_vectors_artifact_is_fresh`), which is a different check.

## Conformance

Conformance is a **factual, self-certified** claim: an implementor runs the **unmodified** KISS-Conform
suite and states the result. Nothing in this repository's CI verifies a third party's claim, and KISS does
not police claims (umbrella §8.4). What KISS **promises** (umbrella §8.3; CireSnave 2026-10-28) is a **free
registry of verified implementations**: on request the steward runs the unmodified suite and, on a pass,
lists the implementation. No date is promised, there is no interim entry for an unverified claim, and **no
such registry is published in this repository yet** (the only registry here, `conformance/registry/`, is the
namespace table). Results from a **modified** suite are ineligible for listing, which is the *only*
enforcement of the mark policy in v1. The **certification mark** is **promised** (CireSnave, 2026-11-13,
board 430 Part 2), not a v1 requirement; what it certifies and who issues it are open
([`ROADMAP.md`](ROADMAP.md) R5.20).

## Governance & contributing

KISS is developed in the open under a lightweight RFC process, stewarded by
ThinkersJournal. Each sub-standard has an **editor of record**; substantive changes go
through an RFC that interested parties may co-sign and comment on; advancing a maturity
stage (Draft → Frozen) requires passing the **freeze gate** — at least two dissimilar
independent implementations plus an adversarial-outsider review. **The rule is a process; only its
traceability condition is automated** (see [How coverage is enforced](#how-coverage-is-enforced)). See umbrella
§5.3 and §7, and [`CONTRIBUTING.md`](CONTRIBUTING.md).

Comment and proposals are welcome now, while the drafts are still soft. A substantive
change is an **RFC — a GitHub issue** labeled `rfc` (use the RFC issue template); the issue
is the durable record from proposal through accept-via-PR or decline-with-rationale. See
[`CONTRIBUTING.md`](CONTRIBUTING.md#the-rfc-lifecycle). Small, obvious fixes may go straight
to a pull request.

## License

KISS is split **by file kind**, because a standard and the code that tests it want different terms.

| What | Licence | Where it is recorded |
|---|---|---|
| **Specification text and data** — every `.md`, `.json`, `.tsv` and other non-code file, including `spec/`, `docs/`, `rfcs/` and the conformance corpus | **[CC0 1.0 Universal](LICENSE)** | [`REUSE.toml`](REUSE.toml), by path |
| **Code** — `.rs` `.c` `.cu` `.py` `.sh` `.bat` `.toml` `.yml`, in `conformance/`, `tools/` and `.github/` | **MIT OR Apache-2.0** ([`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE)) | an `SPDX-License-Identifier` header in each file |

- **The text is public domain.** Anyone may copy, modify, distribute, re-host and implement the
  standard, for any purpose, without permission or attribution — which is how a standard gets
  adopted. CC0 waives copyright and related rights only; patent rights are addressed separately
  (umbrella §9.4).
- **The code is dual-licensed** under the usual Rust terms, at your option.
- **Text is not stamped in-file.** An added first line would shift every specification line number,
  and those numbers are cited in issues and in other repositories; the corpus JSON's generators and
  sha256 pins are recorded in `conformance/corpus/PROVENANCE.md`. `REUSE.toml` records the licence by path instead, touching no byte of those files.
- **Third-party files keep their origin's terms.** `conformance/cuda/generated/` holds a kernel
  emitted verbatim by Baracuda's generator; it is annotated in `REUSE.toml` with its origin's
  licence and holder and is not edited.
- **The copyright holder is Thinker's Journal.**
- `tools/kiss_spdx.py --check` runs in CI and fails if any tracked code file lacks its header.
- `conformance/Cargo.toml` has `publish = false`, so no crate is published from this repository and
  there is no crates.io licence field to keep in step.
- **KISS-Conform suite** — permissive to run; a conformance claim is backed only by results from the
  unmodified suite (the mark policy, umbrella §9.3; enforced through registry eligibility, not by any tool).

---

*KISS is a work in progress. It is being drafted with a design → validate → adversarial-audit
process, and the first-draft documents in this repository have already survived one such
pass — but they are first drafts. Expect them to change.*
