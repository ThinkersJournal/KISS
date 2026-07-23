# KISS — design rationale

This document is **informative**. It records *why* KISS is shaped the way it is —
the cross-cutting design commitments, the alternatives that were weighed and set
aside, and the process by which the suite is authored and advanced. The normative
source is the specification text under [`spec/`](spec); where this document and a
clause disagree, the clause governs.

It exists so that a future editor, implementer, or adopter can understand the
constraints behind a decision without re-litigating settled ground — and so that
the reasoning survives independently of any one contributor.

---

## 1. Design commitments

### 1.1 A suite, not a monolith

KISS is nine interrelated sub-standards over a strict dependency DAG, not one
document. An implementation conforms to the subset it needs: a provider that only
announces pre-built kernels implements far less than a just-in-time generator, and
neither is forced to adopt a facet it does not use. The cost — nine documents to
keep mutually consistent — is paid down by the shared dual-document template, one
clause-ID scheme, and a cross-cutting conformance sub-standard that tests every
edge of the graph.

### 1.2 The IR is an opaque hub

KISS standardizes the *meaning* of each operation — the op-semantics currency — but
**not** the in-memory representation, type layout, or lowering pipeline of any
implementation's intermediate representation. The alternative — standardizing a
concrete IR — was rejected: it would freeze one implementation's internal shape
into the interface, ossify a design space that is still moving, and force every
party onto a single data model. An opaque hub lets implementations differ
internally while agreeing on what a kernel computes.

### 1.3 Bytes and documented meaning, never surface spelling

The currency is wire formats, ABIs, and the documented meaning of the bytes — not
any one language's syntax. Values are pinned as bits / IEEE-754 in wire order, not
as a language's spelling of a constant. This is what lets two independently-written
implementations agree not merely on syntax but on meaning, and what makes a
conformance claim checkable byte-for-byte.

### 1.4 Every clause is testable, or the build fails

Each sub-standard follows one dual-document template: an informative overview
(§0–§5) and a tight normative conformance spec (§6+). Every normative requirement
carries an append-only clause identifier `KISS-<SUB>-<section>-<nnnn>` that maps
**one-to-one to a named conformance test**. Unquantified adjectives ("fast",
"small", "reasonable") are banned from normative text. The intended effect: the
specification cannot drift from its tests, and "conformant" is a factual,
checkable property rather than a marketing claim.

**This is the goal, and the suite does not meet it yet.** As of 2026-07-19, of 861
normative clauses **114 (13.2%) are backed by executable code**, a further 38 are
enforced by document lints (**17.7% enforced** in total), and 704 remain genuinely
untested (5 untestable). The gap is recorded clause by clause in
[`conformance/UNBACKED.tsv`](conformance/UNBACKED.tsv); `tools/kiss_trace.py --report`
prints the live numbers (the source of truth, which move as coverage lands) and the
freeze gate fails on any untested MUST. The ledger is a ratchet: a new untested MUST
fails the gate, and a clause that gains a test is struck from the ledger. (Two earlier
claims in this section have since been fixed: it quoted 3.4% / 29 clauses — long
superseded — and it described a checker that compared markdown to markdown and never
read `conformance/`; the checker now reads the harness, which is how the
846-of-852-missing gap was surfaced.) Closing the remainder is the pre-1.0 priority.

### 1.5 Determinism and fidelity are first-class

Numeric reproducibility is a *property of an op's semantics*, so KISS-Ops owns the
single canonical determinism/fidelity enum (`exact-byte` / `ULP-tolerance` /
`order-invariant`) and the compute-fidelity (MathPrecision) attribute; the protocol
tier imports them. Ownership sits in a foundational vocabulary deliberately: if a
protocol-tier sub-standard owned the enum, a foundational vocabulary would have to
import *upward* from it, inverting the dependency DAG. Each numeric clause declares
its class, and conformance selects the matching comparator — exact-byte where a
result is bit-reproducible, ULP-tolerance for transcendentals (the named function
to within a declared ULP, never cross-language bit identity), order-invariant for
the one nondeterministic case (floating-point atomic combine, invariant to visit
order only up to reassociation).

### 1.6 Canonical encodings resolve defaults explicitly

Where an encoding must be byte-exact so that one party can compare it without
interpreting it, KISS emits **every field explicitly at its resolved value** —
never eliding a field equal to its default. This is the opposite of the mature
ML-IR systems (which elide defaults and reconstruct them from an out-of-band
schema), and it is deliberate: those systems therefore guarantee only *semantic*
stability, never byte-equality, because an absent field's meaning depends on a
versioned default table. Explicit resolution decouples the wire bytes from the
defaulting rules — any producer that resolves to the same value emits the same
bytes — which is exactly what a party that treats the blob as opaque needs.

### 1.7 Identity and the hand-written path

A kernel's op-identity is the *full* advertisable tag (the op name plus its
distinguishing attributes), so specialized variants that share a decomposition
remain distinguishable. The advertisable-op surface is **not** on the mandatory
path: a hand-written kernel may carry a bare op DAG with no advertisable-op entry,
so a one-off kernel still gets a contract without anyone minting a named op for it.
Every kernel carries a contract; the contract is the single source of truth for
how to call it and what it guarantees.

### 1.8 The contract is self-delimiting and fails loudly

The kernel contract is framed by a pinned magic + version + header with an inner
length/checksum, and it fails loudly on a malformed or truncated document, on its
own, independent of any outer transport framing. This answers a concrete failure
mode: a contract carried in a silently-droppable envelope can be received as
"empty-but-OK", so a consumer does nothing with a kernel it was handed and no error
is raised.

---

## 2. Legal shape

- **Specification text: CC0 1.0** (public-domain dedication). CC-BY was weighed and
  set aside: a spec's value is universal adoption, its text is copied into
  implementations, other standards, and docs constantly, and under attribution each
  such reproduction must carry correct credit or infringe. CC0 removes that
  friction. The lever for controlling who may claim *"KISS-conformant"* was never
  copyright anyway — it is the certification mark and the registry.
- **Patents:** a separate royalty-free grant to essential claims with defensive
  termination, bound at contribution time. CC0 waives copyright but not patents, so
  the grant is made explicitly and separately.
- **Reference implementations:** MIT OR Apache-2.0 (the Apache option carries the
  patent grant MIT lacks). A reference implementation is *a* conformant
  implementation with no privilege and no exemption — it runs the same public
  conformance suite.

---

## 3. Conformance is factual, not policed

Conformance is a self-certified claim backed by published results from the
**unmodified** conformance suite, recorded in a steward-maintained free registry.
KISS does not police "KISS-conformant" claims made off-registry: a false claim
self-reveals (the software will not interoperate, and it is not on the registry),
so value accrues to being *listed*, not to the assertion. A registered
certification mark is an optional future lever, not a v1 requirement — which keeps
the standard simple.

---

## 4. How the suite is authored and advanced

- **Authoring.** Each sub-standard is drafted, then put through an *adversarial
  foreign-implementer audit*: a reader who did not write the text attempts to
  implement it from the document alone and reports every ambiguity, every
  under-specified value, and every place two conforming implementations could
  diverge. That pass, not the drafting, is what turns a plausible-looking spec into
  an implementable one — it repeatedly surfaced defects (a dependency-graph
  inversion, an inverted magic-byte convention, an encoding whose defaults left two
  implementations free to disagree) that a solo author does not see.
- **The freeze gate.** A sub-standard advances Draft → Frozen only after **at least
  two dissimilar, independently-developed implementations interoperate on golden
  vectors**, a **foreign reader implements it from the document alone**, and the
  conformance suite for it exists and passes. Growth after a freeze is additive
  (new versions, an extension registry), never a silent redefinition of frozen
  text.
- **Executable conformance.** The reference conformance harness makes the spec's
  claims runnable rather than asserted: golden byte-vectors transcribed from the
  spec's own appendices, an independent CPU oracle that shares no lowering code with
  any implementation, randomized differential loops whose value is that they *catch*
  a wrong implementation, and — for the numeric claims — on-device runs of real
  kernels against the oracle. The point throughout is teeth: a harness that only
  ever passed correct code would prove nothing.

---

## 5. Status

KISS is **pre-1.0 draft**. Nothing is frozen; clause numbers, wire formats, and
vocabularies will change before 1.0. Open design questions and pending decisions
are tracked as issues on the repository. Comment and proposals are welcome now,
while the drafts are still soft — see [`CONTRIBUTING.md`](CONTRIBUTING.md).
