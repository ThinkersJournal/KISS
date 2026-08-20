# KISS-Consume — The Kernel Recognition (Lift) Protocol

> **The name misleads, and this note is load-bearing.** "KISS-Consume" reads as
> *"the kernel consumer's sub-standard."* **It is not.** This is the **recognition
> (lift)** protocol, and its central role is **orthogonal to the provider/consumer
> axis** — as §3 says of the Lifter: *"a provider's own kernel-decomposer is a
> lifter; a consumer's kernel-decomposer is a lifter."* A provider implements this
> document as often as a consumer does.
>
> **Why the name was not changed** (#237, decided 2026-08-19). Clause IDs here are
> append-only and **never reused after retirement** (§ below). Renaming the
> sub-standard would retire all 46 `KISS-CONSUME-*` IDs, burn them permanently,
> mint 46 replacements, and break every cross-reference in the suite — including
> KISS-Emit's §6.7-0008 correspondence table. **Burning a sub-standard's entire ID
> space to fix a name that misleads on first contact is not proportionate.** The
> remedy is this note, plus leading with the real name wherever the pair is listed.
>
> **The evidence that it misleads is readers, not opinion:** in a single day the
> suite maintainer proposed an editorship on the strength of the wrong reading, and
> a portfolio coordinator reported they would have made the same inference from the
> README one-liner. A third reader declined to guess the document's content from its
> name at all — which is the same signal from the inside. **That is a surface
> defect, not a reader defect.**

**Sub-standard ID:** KISS-CONSUME
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification* (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ is normative. Normative
> clauses use RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-CONSUME-<section>-<nnnn>`, and each MUST/SHALL maps 1:1 to at least one
> named KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test.

---

## Abstract

KISS-Consume is the **recognition** protocol of the suite: it governs how an
implementation **lifts a kernel or a source region into the KISS-Ops op DAG as far
as it goes**, producing a KISS-Contract whose Semantics field is the lifted DAG, and
recording the un-liftable remainder as an honest **residue** under a normative
**refusal taxonomy**. The lifter consumes a syntax-free **input structure graph** — a
data-flow representation of candidate operations and their operand edges — and
recognition is **structure-based**: it is a function of that structure, never of
substring, keyword, symbol, or comment sniffing, which this sub-standard declares
**non-conforming**. The refusal taxonomy partitions every recognition failure into
exactly one of four mutually-exclusive and exhaustive categories — **not-a-kernel**,
**wrong-op-class**, **unrecognized-but-expressible**, and **inexpressible-residue** —
each pinned to an observable test, the first two as whole-input typed declines and the
last two as per-region residue tags. Contract **completeness** tracks the
residue-empty vs. residue-non-empty distinction: a fully-lifted or generated kernel
yields a machine-checkable-IR Semantics; a partially-lifted hand-written kernel yields a
declared-op-tag Semantics plus recorded residue — still honest, always carrying the
universal required core. KISS-Consume is the **inverse direction** of KISS-Emit and
shares a two-tier round-trip with it, but neither depends on the other: both depend only
on KISS-Ops, KISS-Classify, and KISS-Contract. Source languages and their grammars are
out of scope; KISS-Consume consumes **structure, not syntax**.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Consume |
| Sub-standard ID | KISS-CONSUME |
| Tier | **Protocol / behavior** (the recognition/lift direction; sits above the two foundational vocabularies and the universal kernel-contract format that it produces) |
| Maturity stage | **Draft** (first-draft proposal; the refusal taxonomy and the lift-contract behavior are NOT yet frozen — the freeze gate of umbrella §5.3 is unmet) |
| Editor of record | **Unpopped** — the neutral kernel-generator reference project — **ratified 2026-08-15**, holding the pen for **both** KISS-Consume and its inverse KISS-Emit. The identical round-trip wording the two share (§6.6) now has a **single pen** across both inverse standards; the KISS-Emit §6.7-0008 correspondence-table lint remains in force as the mechanical check. **Editor–implementer identity is deliberate and bounded** — see KISS-Emit §0 and §10 for the two constraints, which apply to both documents. **Capability caveat, stated rather than implied:** the editor's **Consume-side implementation is materially less mature than its Emit-side one** — a hand-written pilot lifter, with a recognizer that hardcodes a target-specific marker and grammar dependencies behind an off-by-default feature. Holding this pen with a pilot is a different proposition from holding KISS-Emit's with a mature emitter, and the appointment does not imply parity it does not have. The caveat was volunteered by the editor before ratification. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | a recognition/lift reference crate — **`unpopped`** (crate names given in Appendix C as non-normative provenance); this crate is *a* reference implementation with no privilege. **Its lift side is a pilot**, not a mature exemplar — see the Editor-of-record row. |
| DAG position | **Protocol/behavior tier.** Depends (structurally) on KISS-Classify, KISS-Ops, and KISS-Contract. Does **not** depend on KISS-Announce, KISS-Grammar (directly), KISS-Synth/Provision, or KISS-Emit. Consumed by KISS-Conform (test dependency). Not a root. |
| Upstream edges | KISS-Classify (**STRUCTURAL** — the produced contract's Identity `accept_predicate` is a KISS-Classify `structure_key` / specialization cell, and the operand descriptors / `target_capability` recorded on the lifted contract are Classify vocabulary); KISS-Ops (**STRUCTURAL** — the lift **target** is the KISS-Ops op DAG, every emitted node's `op_name` is a KISS-Ops op name, every lift bottoms out at the KISS-Ops **primitive floor**, and the round-trip tier selection imports the KISS-Ops determinism/fidelity enum verbatim); KISS-Contract (**STRUCTURAL** — a successful lift produces a KISS-Contract's Semantics field and its recorded `lift_residue`; the seven-section contract format itself is owned by KISS-Contract and is not re-defined here) |
| Downstream edges | KISS-Conform (**test dependency** — the oracle-differential harness resolves a lifted Semantics DAG to the primitive floor and compares under the op's determinism class; the negative/decline-vector modality exercises the four refusal categories and the never-panic obligation) |
| Sibling (inverse, non-dependency) | KISS-Emit — the **inverse** generation direction. KISS-Consume and KISS-Emit share the two-tier round-trip (§6.6, stated normatively identically in both) but **neither depends on the other**; they are DAG siblings, both depending only on KISS-Ops + KISS-Classify + KISS-Contract. There is **no** DAG edge between them. |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |
| Maturity | Draft proposal |

> **Edge-label note (informative).** All three KISS-Consume upstream edges are
> **STRUCTURAL**: KISS-Consume parses and targets the internal structure of a KISS-Ops
> op DAG (op names, the OpAttrs channel, the primitive floor, the determinism/fidelity
> enum), of a KISS-Classify `structure_key` / operand descriptor / `target_capability`,
> and of the KISS-Contract seven-section document (it produces the Semantics field and
> the recorded residue). The labels reconcile with the umbrella §2.2 edge table, which
> lists **KISS-Classify → KISS-Consume**, **KISS-Ops → KISS-Consume**, and
> **KISS-Contract → KISS-Consume** each as STRUCTURAL, and lists no edge between
> KISS-Consume and KISS-Emit. KISS-Grammar is referenced only **transitively through
> KISS-Contract** (an advertisable-op tag on a produced contract's Identity is a
> KISS-Contract-carried tag); KISS-Consume has **no** direct KISS-Grammar edge and does
> not require KISS-Grammar for any kernel.

---

## 1. Purpose & Scope

KISS-Consume owns the **recognition (lift) direction** of the suite: the mapping from a
recognized computational **structure** — a kernel, or a source region already parsed
into a data-flow / op structure — **into** the KISS-Ops op DAG, carried **as far as the
lifter can take it**, together with the honest accounting of everything it could not
lift. It defines four things and nothing else:

1. **The lift operation and its target** — recognition maps a syntax-free **input
   structure graph** (§6.1-0004) onto the KISS-Ops op DAG (node schema
   `Op{op_name, op_attrs, child_edges} | Bind(positional_index)`), every emitted node
   bottoming out, recursively, at the KISS-Ops **primitive floor**; a successful lift
   produces a **KISS-Contract** whose Semantics field is the lifted DAG. KISS-Consume
   produces the Semantics field and the recorded residue; it never re-defines the
   contract format (that is KISS-Contract) and it never produces an **artifact** (that is
   KISS-Emit).

2. **Structure-based recognition** — recognition is a function of the input structure
   graph. Deciding op identity by substring / keyword / symbol / identifier / comment
   sniffing is declared **non-conforming**; the same recognized structure lifts
   identically regardless of its surface spelling.

3. **The refusal taxonomy** — a normative, MECE (mutually-exclusive, collectively
   exhaustive) partition of every recognition failure into exactly one of
   **not-a-kernel**, **wrong-op-class**, **unrecognized-but-expressible**, and
   **inexpressible-residue**, each pinned to an observable test and assigned by an
   ordered decision procedure over two classification units (whole-input, then
   per-region).

4. **The residue contract** — the un-liftable complement of the lift, recorded honestly
   on the produced contract as `declared-op-tag` Semantics plus a `lift_residue`, so a
   hand-written kernel acquires the richest contract its decomposition affords and never
   a faked one. Contract completeness **tracks** the residue-empty vs. residue-non-empty
   distinction; residue never withholds a contract from a request that was not
   whole-input declined.

**KISS-Consume is NOT:** the data vocabulary (the dtype set, operand descriptor,
`structure_key`, and `target_capability` are KISS-Classify, used here by name/structure);
the computation vocabulary, per-op semantics, primitive floor, OpAttrs channel, or
determinism/fidelity enum (those are KISS-Ops, targeted and imported here, never
restated); the kernel-contract format (that is KISS-Contract, whose Semantics field and
`lift_residue` this direction *produces*); the discovery/handshake or provision protocol
(KISS-Announce / KISS-Synth); the generation/lower direction (KISS-Emit, the inverse,
which KISS-Consume does not depend on); a kernel implementation; and — stated as this
sub-standard's one-line self-policing exclusion drawn from umbrella §1.2 — **a source
language or its grammar**: KISS-Consume consumes **structure, not syntax**, and how a
source region is parsed into an input structure graph is out of scope. Anything not
enumerated as in-scope above is out of scope for KISS-Consume (scope creep by silence is
a named trap; silence is not inclusion).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — lift as far as it goes, then account honestly

A kernel that arrives with no description of *what it computes* is only half-usable: a
consumer can call it (if it has an ABI) but cannot reason about it, verify it, match it,
or fuse around it. KISS-Consume closes that gap for kernels that were **not** born from a
generator — the hand-written, escape-hatch, vendored-binary, or foreign-source kernel —
by **lifting** them into the neutral op vocabulary. Lifting is recognition: the lifter
walks the kernel's input structure graph and, wherever it recognizes a KISS-Ops idiom,
emits the corresponding op node.

The governing principle is **honesty over coverage**. A lifter is not required to lift
everything; it is required to lift what it recognizes and to **account precisely** for
what it did not. That accounting is the refusal taxonomy (§2.4) and the residue (§2.5).
A partially-lifted kernel is not a failure — it is a contract that carries the richest
semantics its decomposition affords, plus an honest gap. The alternative — faking a
machine-checkable semantics the lifter did not derive — is the one thing this
sub-standard forbids outright.

### 2.2 Recognition is structure-based — the anti-sniffing rule

The single most important discipline in KISS-Consume is that recognition is decided by
**structure**, not by **names**. A region that multiplies two operands and adds a third
is a `mul` feeding an `add` *whatever the identifiers are called* — whether the source
spells it `fma`, `mac`, `y = a*b + c`, or `def wibble(...)`. A lifter that decides
"this is a softmax because the function is named `softmax`" is **non-conforming**: it
would mislabel a mis-named kernel and miss a correctly-structured one. Substring,
keyword, symbol-name, identifier, and comment sniffing are all non-conforming as a
recognition signal; a lifter that reads a name may use it only as a non-dispositive hint
that never overrides structure, so that a deliberately mislabeled kernel (its name
suggests one op, its structure computes another) still lifts to the op its structure
computes. This is why **source languages and their grammars are out of scope**: the
standard governs the map from an *input structure graph* to the op DAG; how a particular
language's surface text is parsed into that graph is a separate, out-of-scope concern.
Two lifters that recognize the same structure must produce the same lift, even from two
different source spellings — recognition is renaming-invariant.

### 2.3 The lift target — the KISS-Ops op DAG, bottoming out at the primitive floor

The lift target is not a private IR; it is the **KISS-Ops op DAG** — the same
mixed-abstraction-level DAG the KISS-Contract Semantics field carries and the KISS-Emit
lowering sources read. Every node the lifter emits names a KISS-Ops op; a fusion is a
multi-node DAG, a primitive is a one-node DAG. Because the target is KISS-Ops, every
lifted node is recursively resolvable to the KISS-Ops **primitive floor** — the mandatory
op set every conforming consumer understands — which is the termination guarantee that
makes the lifted semantics *checkable* (KISS-Conform resolves the DAG to the floor and
compares under the op's determinism class). Lifting a high-level idiom directly (a
recognized `gelu` → the high-level `gelu` op) is both cheaper and more robust than
lifting its primitive expansion, so the hierarchical op model **raises the lift rate**:
"broaden the lifted portion" and "broaden contract coverage for hand-written kernels" are
the same work with two payoffs.

### 2.4 The refusal taxonomy — four categories, MECE

When recognition does not fully succeed, the failure is classified into **exactly one**
of four categories by an ordered decision procedure (§6.4) that runs in two units — a
whole-input pre-check (steps 1–2), then per-region classification of each un-lifted
region (steps 3–4):

- **not-a-kernel** — the input structure graph contains **no operation node** at all
  (zero op-bearing structure): there is nothing kernel-shaped to lift, decided from the
  input alone and *before* any op-class question. This is distinct from an op-bearing
  kernel this lifter simply failed to recognize (which leaves whole-body residue under
  steps 3–4, not not-a-kernel). (Testable: zero operation nodes in the input structure
  graph.)

- **wrong-op-class** — the input **is** an op-bearing kernel and yields an op, but its
  coarse op category is incompatible with the **requested** specialization **cell** — the
  recognized root op's KISS-Ops op-family does not match the request cell's Classify
  op-category, per the KISS-Contract Identity consistency relation. It is declined as
  *not-fitting*, not *not-understood*. (Testable: recognition yields a root op whose
  family is absent from the compatible set for the request cell.)

- **unrecognized-but-expressible** — a region that genuinely **is** expressible as a
  KISS-Ops op DAG, but *this* lifter does not yet recognize the idiom. A lifter-coverage
  gap, not a fundamental limit: a more complete lifter would succeed, and broadening the
  lifter converts it into a successful lift. (Testable: the KISS-Conform expressibility
  oracle at this op-set version judges the region expressible — its signature is a member
  of the enumerated set of KISS-Ops-decomposable region signatures for that op-set
  version — though this lifter emitted no node.)

- **inexpressible-residue** — the truly un-liftable remainder: a region the same
  expressibility oracle judges **not** a member of the decomposable-signature set at the
  current op set — a genuine atom with no in-vocabulary meaning. Recorded honestly as
  residue, never faked into a machine-checkable IR it does not have. (Testable: the
  region signature is not in the decomposable set at this op-set version.)

The first two reject the **request** (a typed decline: there is no useful contract to
produce for *this* asking). The last two leave **residue** on a partially-lifted
contract (the kernel still lifted as far as it went). The four are mutually exclusive
because the decision procedure is ordered over its two units, and exhaustive because the
last two cover the complementary cases "the oracle expresses it" versus "it does not."

### 2.5 The residue contract — completeness tracks residue emptiness

The **lift fraction** — informally, how much of a kernel was lifted into the op DAG — is
an informative descriptor with **no pinned denominator**; the cited numbers (1.0, ~0.5)
are illustrative only. The **normative** distinction is binary: whether the recorded
residue is **empty** or **non-empty**. Contract **completeness tracks** that distinction,
exactly as KISS-Contract requires:

- A **fully lifted** (or generated) kernel — empty residue — gets
  `semantics_kind = machine-checkable-IR`: a Semantics field that is a complete,
  resolvable op DAG.
- A **partially lifted** hand-written kernel — non-empty residue — gets
  `semantics_kind = declared-op-tag`: the recognized op-identity tag for the parts it did
  lift, **plus** a recorded `lift_residue` naming the un-liftable remainder and its
  refusal category.

Residue is recorded **honestly** and **never faked**: a lifter that cannot derive a
machine-checkable IR must not fabricate one. And residue **never withholds a contract**
from a request that was not whole-input declined (§2.4 steps 1–2): every such kernel still
carries the universal required core (the seven KISS-Contract sections) regardless of how
much was lifted — the residue is recorded *on* the contract, it does not suppress it. A
hand-written kernel therefore acquires its contract **by lifting**; a provider's own
kernel-decomposer *is* a KISS-Consume lifter.

### 2.6 A worked lift — a strided binary `add` recognized in full

A hand-written strided binary elementwise `add` over `f32`, target `cuda:sm89`. The
lifter recognizes a single elementwise binary structure: two operand reads feeding one
`add`, one store. It emits a **one-node DAG** `{ op: add }` (a primitive, so a 1-node
DAG). Residue = ∅ (lift fraction, informally, ≈ 1.0). It produces a KISS-Contract whose:

- **Identity** records `accept_predicate` = the cell's `structure_key` (cell op-category
  code `bin`, three `f32` operands, strided), `op_identity` = the **bare** KISS-Ops op
  name `add` (form (b) — the canonical form a lift produces), and
  `target_capability = cuda:sm89`;
- **Semantics** = `{ op: add }`, `semantics_kind = machine-checkable-IR`, edge-case
  policy resolved from KISS-Ops (`add` is IEEE-754, NaN-propagating).

The remaining five sections are filled by KISS-Contract's rules. The lift produced the
Semantics field; the contract format did the rest.

### 2.7 A worked lift — a partial lift with honest residue

A hand-written fused kernel computes a `matmul` followed by a bespoke,
provider-proprietary activation with no closed KISS-Ops form. The lifter recognizes the
contraction structure and emits `{ op: matmul }`, but the activation region's signature
is not in the KISS-Ops decomposable set at this op-set version. Residue is non-empty
(lift fraction, informally, ≈ 0.5). The contract:

- **Semantics** carries `semantics_kind = declared-op-tag`, the lifted `matmul` node, and
  a `lift_residue` entry over the activation region (one entry per maximal connected
  un-lifted component) tagged **inexpressible-residue** at the recorded op-set version;
- still carries all seven sections — the residue did not withhold the contract.

Had the activation instead been an ordinary `erf`-based `gelu` that *this* lifter simply
failed to recognize, the residue entry would be tagged **unrecognized-but-expressible**
(the oracle expresses it) — and broadening the lifter would later convert it into a
lifted node.

### 2.8 The four refusals, worked

- **not-a-kernel:** the input structure graph is a header, a comment block, a data table,
  or an empty region — zero operation nodes. Typed decline `not-a-kernel`.
- **wrong-op-class:** the consumer asks, by request `structure_key`, for a kernel in the
  reduction cell (cell op-category code `red`), but the lifted root op is `matmul`, whose
  KISS-Ops op-family is `contraction` — a family the KISS-Contract Identity consistency
  relation lists as incompatible with the `red` cell op-category. (Note the cell code and
  the op-family are distinct closed sets: `gem` is the *cell* op-category code, whereas
  `contraction` is the *KISS-Ops op-family*; they are never conflated.) Recognition
  succeeded, the op is understood, but its family is incompatible with the requested cell;
  typed decline `wrong-op-class`.
- **unrecognized-but-expressible:** a region computes a `softmax`, expressible over
  `reduce` + `exp` + `div`, but this early lifter has no softmax idiom; residue tagged
  `unrecognized-but-expressible`.
- **inexpressible-residue:** a region invokes a hardware transcendental whose signature is
  not in the KISS-Ops decomposable set at this op-set version; residue tagged
  `inexpressible-residue`.

### 2.9 The round-trip — how Consume and Emit stay honest without an edge

KISS-Emit lowers `(OpDef + structure_key) → artifact-described-by-a-contract`;
KISS-Consume lifts `kernel/source region → the contract's Semantics op DAG (as far as it
goes) + residue`. They are **inverse directions** that **share** a two-tier round-trip
but **neither depends on the other** — they are DAG siblings, both depending only on
Ops + Classify + Contract. The round-trip is the join that keeps them honest without an
edge: Emit's output, lifted by Consume, must reproduce the same op DAG (**tier 1,
structural**), and only same-language on-device may the numerics match bit-for-bit
(**tier 2, numeric**). Numeric identity is never claimed across languages — for example,
a `tanh` implemented in one GPU shader language (say, Slang) is not bit-identical to the
same `tanh` implemented in another (say, CUDA), even though both lift to the same
structural op-DAG node. The two-tier statement is worded **identically** in KISS-Emit and
KISS-Consume (§6.6) precisely so the two directions cannot drift, and both import the
KISS-Ops determinism/fidelity enum to decide which tier applies per op. A whole-kernel
tier-2 claim is admissible only when *every* op in the resolved DAG is exact-byte; a
single ULP or nondeterministic op makes the whole-kernel round-trip tier-1 only
(§6.6-0006).

### 2.10 Terms are joined, not restated

KISS-Consume references the KISS-Ops op names, op DAG node schema, OpAttrs channel,
primitive floor, and determinism/fidelity enum by name; the KISS-Classify `structure_key`,
operand descriptors, and `target_capability` by name; and the KISS-Contract seven-section
format, `semantics_kind`, and `lift_residue` by name. It re-defines none of them and
defines no op meaning: Consume performs recognition and produces a Semantics field, the
foundational vocabularies mean the ops and the contract format frames the document.

---

## 3. Terms & Definitions

- **Lift / recognition** — the operation this sub-standard governs: mapping a recognized
  **input structure graph** onto the KISS-Ops op DAG, as far as the lifter recognizes it,
  producing a KISS-Contract's Semantics field and its recorded residue (§6.2).
- **Lifter** — an implementation of KISS-Consume: the party performing recognition. A
  provider's own kernel-decomposer is a lifter; a consumer's kernel-decomposer is a
  lifter.
- **Computational structure** — the data-flow / op structure of a kernel or source region
  (the operations performed and their operand edges), as distinct from its **surface
  syntax** (the identifiers, keywords, and comments of a source language). Recognition is
  a function of the former only (§6.1).
- **input structure graph** — the syntax-free data-flow representation a lifter consumes
  for conformance (§6.1-0004): a directed graph whose nodes are candidate **operation
  records** — each an operation-kind field, a canonically-ordered operand-edge list, and
  an opaque attribute blob — and whose edges are operand data-flow, carrying no surface
  syntax (identifiers, keywords, comments). It is the representation in which golden
  lift/refusal input vectors are authored and fed, byte-for-byte, to independent lifters;
  its canonical serialization follows the suite little-endian discipline. How a source
  language's surface text is parsed into this graph is out of scope.
- **operation node** — a node of the input structure graph that records an arithmetic or
  memory operation (as opposed to a header, comment, data, or empty region). Its presence
  is decided from the input alone, independent of what any particular lifter recognized.
- **op-bearing computational structure** — an input structure graph that contains at least
  one operation node. Its negation (zero operation nodes) is the not-a-kernel condition
  (§6.4-0003).
- **op DAG / op node** — the KISS-Ops hierarchical, mixed-abstraction-level directed
  acyclic graph, node schema `Op{op_name, op_attrs, child_edges} | Bind(positional_index)`,
  recursively resolvable to the primitive floor; **owned by KISS-Ops**, carried by the
  KISS-Contract Semantics field, and the lift **target** here. A node's `op_name` is a bare
  KISS-Ops op name; a node **corresponds to** an advertisable op only when its `op_name` +
  identity-bearing `op_attrs` + operand-role tuple reconstruct a KISS-Grammar tag (a
  correspondence a lift does not itself record — a lift emits the bare op name, §6.2-0006).
- **primitive floor** — the mandatory-core KISS-Ops op set at which every decomposition
  chain terminates (acyclic + strictly-decreasing level = the termination guarantee);
  **owned by KISS-Ops (§6.3)**. Lift targets bottom out here.
- **OpAttrs channel** — the per-op compile-time attribute record (reduce monoid/axes,
  gather OOB policy, pool geometry, permutation, …) with a canonical default-resolved
  little-endian byte encoding; **owned by KISS-Ops (§6.19)**, carried per lifted node.
- **structure_key / specialization cell** — the KISS-Classify admissibility predicate over
  one layout/dtype/target specialization cell (a coarse op-category tag + canonically-ordered
  operand descriptors + `target_capability` + role hints, **extent-free**, matched
  byte-for-byte); **owned by KISS-Classify**. The lift records it on the produced contract's
  Identity `accept_predicate`; it is the accept-predicate, **not** the op's semantic identity.
- **request specialization cell** — an **optional input** to the lift: a KISS-Classify
  `structure_key` naming the specialization cell the caller requests. When supplied it is
  the subject of the wrong-op-class check (§6.4-0004) and becomes the produced contract's
  Identity `accept_predicate` (§6.2-0005); when absent, the lifter derives the
  `accept_predicate` `structure_key` from the kernel's operand structure and wrong-op-class
  cannot fire (§6.2-0008).
- **target_capability** — the KISS-Classify namespaced `<namespace>:<capability-set>`
  all-hardware descriptor of the compilation target, matched byte-exact on the full string;
  **owned by KISS-Classify**, recorded on the produced contract's Identity.
- **contract** — the KISS-Contract seven-section, self-delimiting document {Identity,
  Semantics, Interface, Dispatch, Capabilities, Guarantees, Provenance} that travels with
  every provided kernel; **owned by KISS-Contract**. KISS-Consume **produces its Semantics
  field** (and the recorded residue), never re-defines it.
- **semantics_kind** — the KISS-Contract Semantics-field flavor: `machine-checkable-IR` (a
  generated/lifted kernel, empty residue) or `declared-op-tag` (a hand-written kernel with
  non-empty residue). Set by the lift outcome (§6.3).
- **op_identity** — the identity of the op the produced Semantics DAG root computes,
  recorded on the contract Identity per KISS-Contract: **(a)** a full KISS-Grammar
  advertisable-op tag re-based on a KISS-Ops op name, or **(b)** the **bare KISS-Ops op
  name** of the root (no Grammar tag). A **lift always produces form (b)** (§6.2-0006); a
  downstream Grammar-aware step, out of scope here, may re-base it to form (a). **Owned by
  KISS-Contract Identity**; distinct from `structure_key`.
- **lift fraction** — an **informative** descriptor of how much of a kernel was lifted into
  the op DAG; it has **no pinned denominator** and is not a normative quantity. Only the
  **residue-empty vs. residue-non-empty** distinction is normative (§6.3-0001): residue-empty
  ⇔ `machine-checkable-IR`, residue-non-empty ⇔ `declared-op-tag`.
- **residue / lift_residue** — the recorded un-liftable remainder on a partial contract: the
  set of `unrecognized-but-expressible` and `inexpressible-residue` regions actually left on
  a given run, one entry per maximal connected component of un-lifted operation nodes
  (§6.3-0008), recorded on the produced contract per the refusal taxonomy (§6.3, §6.4).
- **residue region** — a maximal connected component of un-lifted operation nodes in the
  input structure graph; the canonical unit each `lift_residue` entry identifies (§6.3-0008),
  so two dissimilar lifters that leave the same operation nodes unlifted record the same set
  of residue regions.
- **recognition failure** — a condition in which the lift does not fully succeed, evaluated
  in two units: a **whole-input** unit (the input as a whole may be declined not-a-kernel or
  wrong-op-class, §6.4-0003/0004) and a **per-region** unit (each un-lifted residue region of
  an otherwise-lifted kernel is classified unrecognized-but-expressible or
  inexpressible-residue, §6.4-0005/0006). See §6.4-0002.
- **refusal taxonomy** — the normative, MECE partition of every recognition failure into
  exactly one of **not-a-kernel**, **wrong-op-class**, **unrecognized-but-expressible**,
  **inexpressible-residue** (§6.4).
- **expressibility oracle** — the KISS-Conform expressibility judgment at a referenced
  KISS-Ops op-set version: a region is **expressible** iff its region signature is a member
  of the enumerated set of KISS-Ops-decomposable region signatures published for that op-set
  version. This decidable membership test is the authority the
  unrecognized-but-expressible ↔ inexpressible-residue boundary is defined against
  (§6.4-0005/0006); it is not an open existential search.
- **determinism/fidelity enum** — the single canonical KISS-Ops enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` (KISS-OPS §6.0-0001), imported verbatim;
  selects which round-trip tier is claimable per op (§6.6).
- **MathPrecision attribute** — the KISS-Ops compute-fidelity enum `{bit-stable,
  reduced-mantissa-permitted}` (KISS-OPS §6.17), imported verbatim; surfaced in the produced
  contract's Guarantees section, never re-forked.
- **artifact** — the built, callable kernel binary/object (the first element of a provision
  response `{artifact, contract}`); **owned by KISS-Synth/Provision / produced by KISS-Emit**.
  KISS-Consume **never produces an artifact** — it produces semantics (§6.2).
- **structural round-trip (tier 1)** — emit-then-lift (or lift-then-emit) reproduces the same
  KISS-Ops op DAG under structural / op-DAG equality over a declared subset (§6.6-0001).
- **numeric round-trip (tier 2)** — bit-identity of the computed result claimed only
  same-language, on-device, and only for the exact-byte determinism class (§6.6-0002).
- **Typed decline** — a structured refusal returned in lieu of a result (a distinguished
  error value/enumerant, or an equivalent out-of-band error return); never a panic, abort,
  crash, hang, or out-of-bounds read. The unifying failure currency: not-a-kernel and
  wrong-op-class are returned as typed declines (§6.5).

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point semantics; referenced transitively through KISS-Ops
  (KISS-Consume defines no numeric behavior of its own).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword convention,
  the normative/informative split, the clause-ID scheme and 1:1 test mapping, value pinning
  as bits/IEEE-754 in wire order, the ban on unquantified adjectives, the two version axes,
  the ≥2-dissimilar-implementations-plus-foreign-reader freeze gate (umbrella §5.3), the
  capability/profile/extension model, governance, licensing, and patent posture. **Stated
  once in the umbrella; referenced here; never restated.** This sub-standard's §5 points at
  umbrella §3 for conventions.
- **KISS-Ops** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency: the
  lift **target** is the KISS-Ops **op DAG** (node schema
  `Op{op_name, op_attrs, child_edges} | Bind(positional_index)`); every lifted node's
  `op_name` is a KISS-Ops op name (KISS-OPS §6.1) carrying the KISS-Ops **OpAttrs** channel
  (KISS-OPS §6.19); every lift bottoms out, recursively, at the KISS-Ops **primitive floor**
  (KISS-OPS §6.3, the termination guarantee); the per-op semantics, the op-name→**op-family**
  classification used by the wrong-op-class category (KISS-OPS §6.1-0003 / §2.7), the single
  canonical **determinism/fidelity enum** `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}` (KISS-OPS §6.0-0001), and the **MathPrecision** attribute
  `{bit-stable, reduced-mantissa-permitted}` (KISS-OPS §6.17) are **imported verbatim, never
  re-forked**. KISS-Consume re-defines no op and no op meaning.
- **KISS-Classify** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency:
  the produced contract's Identity `accept_predicate` is a KISS-Classify **`structure_key`**
  (the admissibility predicate over a specialization cell); the operand descriptors and the
  namespaced **`target_capability`** recorded on the lifted contract are Classify vocabulary;
  the coarse cell-level **op-category** enum used by the wrong-op-class category (the
  `op_family_tag` component of a `structure_key`, KISS-CLASSIFY §6.5) is Classify vocabulary,
  a **distinct** closed set from the KISS-Ops op-family, spelled with the Classify code (e.g.
  the 3-letter code) verbatim and never conflated with a KISS-Ops op-family name. Used here by
  name/structure; re-defined nowhere.
- **KISS-Contract** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency
  (the contract this direction produces): a successful lift produces a KISS-Contract's
  **Semantics** field (`semantics_kind ∈ {machine-checkable-IR, declared-op-tag}`) and its
  recorded **`lift_residue`**; the seven-section universal required core, the Identity
  `accept_predicate` / `op_identity` fields, the `semantics_kind` enum, and the
  **Identity consistency relation** (the cell-op-category ↔ KISS-Ops-op-family compatibility
  table of KISS-CONTRACT §6.3-0006) that the **wrong-op-class** category leans on are all
  **owned by KISS-Contract** and referenced from there, **never re-forked** here. The
  residue-bearing category set carried by KISS-Contract's `lift_residue` field is the two-token
  subset defined once in KISS-Consume §6.3-0004 (`unrecognized-but-expressible`,
  `inexpressible-residue`); KISS-Contract references that subset and must not enumerate a wider
  set (Appendix D). KISS-Consume produces the contract; it does not re-define the contract
  format, the transport, or the consistency table.
- **KISS-Emit** (by version) — the **inverse** generation direction and a **DAG sibling**, not
  a dependency: KISS-Consume has **no** DAG edge to or from KISS-Emit. The two share the
  **two-tier round-trip** of §6.6, whose text is **semantically identical** in both documents (same
  clause intent, same normative effect, not a byte-for-byte diff) so the inverse directions cannot
  drift (the KISS-Emit §6.7-0008 clause-correspondence-table lint enforces the correspondence, §6.6);
  each imports the KISS-Ops determinism/fidelity enum to decide which tier applies per op.
  Neither may be claimed as a prerequisite of the other.
- **KISS-Grammar** (by version) — referenced **only transitively through KISS-Contract**: when
  a produced contract's Identity `op_identity` is later re-based to a KISS-Grammar
  advertisable-op tag, that tag is a KISS-Contract-carried field. KISS-Consume has **no** direct
  KISS-Grammar edge and a conforming lifter must not require a KISS-Grammar advertisable-op entry
  for any kernel (a lift produces a bare KISS-Ops op name, §6.2-0006).
- **KISS-Conform** (by version) — depends on and tests KISS-Consume; owns the
  oracle-differential harness that resolves a lifted Semantics DAG to the primitive floor and
  compares under the op's declared determinism class, the **expressibility oracle** that decides
  the unrecognized-but-expressible ↔ inexpressible-residue boundary (§6.4-0005/0006), and the
  negative/decline-vector modality that exercises the four refusal categories and the never-panic
  obligation.

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase
keywords `MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance and
consumer-behavior guidance and never state a structural or behavioral requirement. Every
atomic requirement carries a stable, append-only ID `KISS-CONSUME-<section>-<nnnn>`,
allocated by the editor of record, never reused after retirement, and mapped 1:1 to ≥1
named KISS-Conform test. Values are pinned as tokens/schema spelled exactly as the
upstream foundational vocabularies and KISS-Contract pin them, never as one source
language's surface spelling. Unquantified adjectives ("well-formed", "reasonable",
"neutral", "valid") are banned from normative text. Every clause declares its
determinism/fidelity class so KISS-Conform selects the correct comparator. See umbrella
§3 for the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

- **KISS-CONSUME-6.0-0001** — Every structural obligation in §6–§8 (the lift-target op-DAG
  discipline, the input-structure-graph representation, the refusal-taxonomy category
  spellings and decision procedure, the residue recording schema, the round-trip tier
  statements, and every token spelling) is determinism-class **exact byte compare**;
  KISS-Conform MUST evaluate each such clause with a byte-exact comparator and MUST NOT apply
  tolerance or order-invariant comparison. KISS-Consume defines no numeric result of its own;
  the numeric determinism class of any op a lifted DAG names, and the tier a round-trip claim
  may assert, are **owned by KISS-Ops** (the single canonical enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}`, KISS-OPS §6.0-0001) and MUST NOT be re-forked here.
  *Test:* `test_consume_determinism_class_exact_byte`.

### 6.1 Recognition is structure-based

- **KISS-CONSUME-6.1-0001** — Recognition MUST be a function of the region's **computational
  structure** (its operations and their operand data-flow edges) and MUST NOT depend on a
  substring, keyword, symbol name, identifier, type name, or comment of the source: a kernel
  whose name or surface tokens suggest one op while its computational structure computes
  another MUST lift to the op its structure computes, not the op its name suggests. The
  KISS-Conform vector set for this clause MUST include a deliberately mislabeled kernel (name
  and tokens disagree with structure) and require the structurally-correct lift. *Test:*
  `test_consume_recognition_structure_based`.
- **KISS-CONSUME-6.1-0002** — Recognition MUST be **renaming-invariant**: two inputs with the
  same computational structure but different surface spellings (identifiers, keywords,
  comments, whitespace) MUST produce the **same** lifted op DAG and the same refusal
  classification. An implementation MUST NOT produce a different lift for the same structure
  on the basis of surface spelling alone. *Test:* `test_consume_recognition_renaming_invariant`.
- **KISS-CONSUME-6.1-0003** — A conforming lifter MUST NOT require, assume, or bless any
  particular source language or grammar: how a source region is parsed into an input structure
  graph is **out of scope** for KISS-Consume, and a conforming lifter's recognition MUST be
  expressible over the input structure graph alone. An implementation MUST NOT make a
  source-syntax match a **necessary** condition for a lift that the structure otherwise
  supports. *Test:* `test_consume_source_syntax_out_of_scope`.
- **KISS-CONSUME-6.1-0004** — The conformance input a lifter consumes MUST be the syntax-free
  **input structure graph**: a directed graph whose nodes are candidate operation records —
  each an operation-kind field, a canonically-ordered operand-edge list, and an opaque
  attribute blob — and whose edges are operand data-flow, carrying no surface syntax. A
  conforming lifter MUST accept this representation, and its recognition and refusal
  classification MUST be a function of it alone (§6.1-0001). Golden lift/refusal input vectors
  MUST be authored in this representation with a byte-exact canonical little-endian
  serialization, so a single input vector is feedable to two dissimilar lifters. Other
  in-memory input forms are out of scope; an implementation that additionally accepts them
  MUST produce the identical lift and refusal classification for the canonical input structure
  graph of the same computational structure. *Test:* `test_consume_input_structure_graph`.

### 6.2 The lift operation and its target

- **KISS-CONSUME-6.2-0001** — The lift **target** MUST be the KISS-Ops **op DAG** under the
  node schema `Op{op_name, op_attrs, child_edges} | Bind(positional_index)`; every op node an
  implementation emits MUST carry an `op_name` that is a KISS-Ops op name of the referenced
  KISS-Ops version (KISS-OPS §6.1). Emitting a node whose `op_name` is outside the KISS-Ops op
  set, or lifting into a private or forked op vocabulary, is the negation of this requirement
  and is non-conforming. *Test:* `test_consume_lift_target_is_ops_dag`.
- **KISS-CONSUME-6.2-0002** — Every lifted op DAG MUST be recursively resolvable to the
  KISS-Ops **primitive floor** (KISS-OPS §6.3): every non-primitive node an implementation
  emits MUST be one whose KISS-Ops reference decomposition terminates at the floor
  (acyclic, strictly-decreasing level). An implementation MUST NOT emit a lifted DAG that does
  not bottom out at the primitive floor. *Test:* `test_consume_lift_resolves_to_floor`.
- **KISS-CONSUME-6.2-0003** — A kernel lifted with **empty residue** MUST produce a
  KISS-Contract whose Semantics field is the lifted op DAG with
  `semantics_kind = machine-checkable-IR` (KISS-CONTRACT §6.4). KISS-Consume produces the
  Semantics field and the recorded residue; an implementation MUST NOT re-define, re-frame, or
  re-order the KISS-Contract seven-section format, which is owned by KISS-Contract. *Test:*
  `test_consume_full_lift_produces_machine_checkable_ir`.
- **KISS-CONSUME-6.2-0004** — A KISS-Consume lift MUST NOT produce an **artifact** (a built,
  callable kernel binary/object): it produces **semantics** (a Semantics field and recorded
  residue) only. Producing an artifact is the KISS-Emit / KISS-Synth direction; an
  implementation MUST NOT present a lift result as a provision `{artifact, contract}` response.
  *Test:* `test_consume_produces_no_artifact`.
- **KISS-CONSUME-6.2-0005** — The produced contract's Identity `accept_predicate` MUST be a
  KISS-Classify **`structure_key`** and its `target_capability` MUST be the KISS-Classify
  namespaced descriptor, both recorded verbatim from Classify vocabulary: the `structure_key`
  is the **request specialization cell's** when one is supplied and the structure-derived cell's
  otherwise (§6.2-0008). An implementation MUST NOT invent, re-encode, or reinterpret these
  tokens, and MUST NOT present the `structure_key` as the op's semantic identity. *Test:*
  `test_consume_records_structure_key_on_identity`.
- **KISS-CONSUME-6.2-0006** — A lift's produced contract Identity `op_identity` MUST be **form
  (b)**: the **bare KISS-Ops op name** of the lifted Semantics DAG root, spelled exactly as
  KISS-Ops pins it. A lift MUST NOT emit **form (a)** (a full KISS-Grammar advertisable-op tag)
  and MUST NOT require a KISS-Grammar advertisable-op entry for any kernel; where a downstream
  Grammar-aware step re-bases a bare op name onto a Grammar tag, that step is outside
  KISS-Consume. An implementation MUST NOT duplicate the full op DAG into `op_identity`.
  *Test:* `test_consume_op_identity_from_dag_root`.
- **KISS-CONSUME-6.2-0007** — Every op node an implementation emits MUST carry its
  identity-bearing attributes on the KISS-Ops **OpAttrs** channel (KISS-OPS §6.19),
  default-resolved and encoded exactly as KISS-Ops pins it. An implementation MUST NOT omit the
  OpAttrs channel from an emitted node and MUST NOT carry a node's op attributes in a private
  side-channel. *Test:* `test_consume_lift_node_carries_opattrs`.
- **KISS-CONSUME-6.2-0008** — The lift's input signature MUST admit an **optional request
  specialization cell** (a KISS-Classify `structure_key`). When a request cell is supplied, the
  produced contract's Identity `accept_predicate` MUST be that request cell's `structure_key`
  (recorded verbatim, §6.2-0005) and the wrong-op-class check (§6.4-0004) MUST be evaluated
  against it. When no request cell is supplied, the implementation MUST derive the
  `accept_predicate` `structure_key` from the kernel's operand structure, and the wrong-op-class
  category MUST NOT fire (there is no requested cell to conflict with). *Test:*
  `test_consume_lift_input_signature`.

### 6.3 Lift outcome and the residue contract

- **KISS-CONSUME-6.3-0001** — Contract **completeness** MUST track the **residue-empty vs.
  residue-non-empty** distinction (the only normative reading of the lift fraction; the
  continuous fraction is informative, §3): a kernel lifted with **empty** residue MUST get
  `semantics_kind = machine-checkable-IR` (§6.2-0003), and a kernel lifted with **non-empty**
  residue MUST get `semantics_kind = declared-op-tag` plus a recorded `lift_residue` naming the
  un-liftable remainder. An implementation MUST NOT emit `machine-checkable-IR` for a kernel
  whose lift left residue. *Test:* `test_consume_completeness_tracks_lift_fraction`.
- **KISS-CONSUME-6.3-0002** — Recorded residue MUST be **honest** and MUST NOT be **faked**: an
  implementation MUST NOT fabricate a machine-checkable-IR Semantics it did not derive, and MUST
  NOT emit a lifted node for a region it did not recognize (see also §6.5-0003). *Test:*
  `test_consume_residue_honest_never_faked`.
- **KISS-CONSUME-6.3-0003** — For a request that is **not** whole-input declined under §6.4
  steps 1–2 (not-a-kernel, wrong-op-class), residue MUST **never withhold a contract**: a
  partially-lifted or fully-un-lifted-but-op-bearing kernel MUST still carry the KISS-Contract
  **universal required core** (the seven sections, KISS-CONTRACT §6.2-0001); an implementation
  MUST NOT return "no contract" for such a request on the ground that it lifted only partially
  or not at all — the residue is recorded **on** the contract, it does not suppress it. A
  whole-input typed decline under §6.4 steps 1–2 produces no contract for that request
  (§6.4-0008) and is the sole exception. *Test:* `test_consume_residue_never_withholds`.
- **KISS-CONSUME-6.3-0004** — Each recorded `lift_residue` entry MUST tag its un-liftable region
  with exactly one refusal category from the **residue-bearing subset** —
  `unrecognized-but-expressible` or `inexpressible-residue` — spelled byte-exact. This
  two-token subset is the **single authoritative definition** of the residue-bearing category
  set; `not-a-kernel` and `wrong-op-class` are whole-input typed declines (§6.4-0008), never
  residue tags. KISS-Contract's `lift_residue` field references this subset and MUST NOT
  enumerate a wider set; an implementation MUST NOT record a residue entry with any other
  category token. *Test:* `test_consume_residue_entry_tagged`.
- **KISS-CONSUME-6.3-0005** — Each recorded `lift_residue` entry MUST carry the **KISS-Ops
  op-set version** under which its `unrecognized-but-expressible` / `inexpressible-residue`
  determination was made, so the determination stays honest across op-set version bumps (a later
  op set can move an entry from inexpressible to liftable; the recorded version is what pins the
  determination — §8.1-0002). An implementation MUST NOT record a residue entry without its
  determining op-set version. *Test:* `test_consume_residue_records_opset_version`.
- **KISS-CONSUME-6.3-0006** — A hand-written kernel's contract Semantics MUST be **produced by
  lifting** the kernel as far as recognition takes it (§6.2), not authored as a bare
  op-identity tag with no lift attempt; where recognition lifts nothing over an op-bearing input,
  the residue records the whole body and `semantics_kind = declared-op-tag` (§6.3-0001). An
  implementation MUST NOT emit a `declared-op-tag` Semantics that discards a lift the structure
  supported. *Test:* `test_consume_handwritten_semantics_via_lifting`.
- **KISS-CONSUME-6.3-0007** — An implementation MUST NOT **silently drop** an un-liftable region:
  every un-lifted residue region (§6.3-0008) MUST be recorded as a `lift_residue` entry rather
  than discarded. *Test:* `test_consume_no_silent_drop_of_residue`.
- **KISS-CONSUME-6.3-0008** — Each `lift_residue` entry MUST be the fixed triple {**region
  identifier**, **category token** (§6.3-0004), **determining op-set version** (§6.3-0005)} with
  a byte-exact canonical little-endian serialization. The region identifier MUST identify a
  **residue region** under the canonical partitioning rule: **one residue entry per maximal
  connected component of un-lifted operation nodes** in the input structure graph (§6.1-0004),
  so two dissimilar lifters that leave the same operation nodes unlifted record the same set of
  residue regions. An implementation MUST NOT split or merge residue regions in violation of the
  maximal-connected-component rule, and MUST NOT emit a residue entry omitting any of the three
  fields. *Test:* `test_consume_residue_entry_schema`.

### 6.4 The refusal taxonomy (MECE)

- **KISS-CONSUME-6.4-0001** — The refusal taxonomy MUST be exactly the four categories, spelled
  byte-exact: **`not-a-kernel`**, **`wrong-op-class`**, **`unrecognized-but-expressible`**,
  **`inexpressible-residue`**. An implementation MUST classify every recognition failure into
  exactly one of these four and MUST NOT introduce, alias, or omit a category at this schema
  version. *Test:* `test_consume_taxonomy_four_categories`.
- **KISS-CONSUME-6.4-0002** — The four categories MUST be **mutually exclusive and collectively
  exhaustive**, applied in two classification units. **Phase A — whole-input pre-check:** steps
  1–2 (§6.4-0003 not-a-kernel, §6.4-0004 wrong-op-class) are evaluated over the input as a whole
  and yield **at most one whole-input typed decline**; if either fires, classification stops with
  that single decline. **Phase B — per-region:** if neither step 1 nor step 2 fires, the lift
  proceeds and each un-lifted residue region (a maximal connected component of un-lifted operation
  nodes, §6.3-0008) MUST be classified into exactly one of steps 3–4 (§6.4-0005
  unrecognized-but-expressible, §6.4-0006 inexpressible-residue) by taking the **first** whose
  condition holds. An implementation MUST NOT assign two categories to one unit and MUST NOT leave
  a recognition failure unclassified. *Test:* `test_consume_taxonomy_mece_ordered`.
- **KISS-CONSUME-6.4-0003** — **Step 1 — not-a-kernel (whole-input).** A recognition failure MUST
  be classified **`not-a-kernel`** when the input structure graph contains **no op-bearing
  computational structure** — **zero operation nodes** (an arithmetic or memory operation record)
  — decided from the **input alone**, independent of what this lifter recognized. This is a
  rejection of the input's **kind**, decided **before** any op-class question; an implementation
  MUST NOT reach steps 2–4 when the input has zero operation nodes, and MUST NOT classify an
  op-bearing input (≥1 operation node) `not-a-kernel` merely because this lifter recognized none
  of its operations (that is whole-body residue under steps 3–4, §6.3-0006). *Test:*
  `test_consume_refusal_not_a_kernel`.
- **KISS-CONSUME-6.4-0004** — **Step 2 — wrong-op-class (whole-input).** When the input is
  op-bearing (step 1 did not fire) **and** a **request specialization cell is supplied**
  (§6.2-0008), a failure MUST be classified **`wrong-op-class`** when the lifted **root** op's
  KISS-Ops op-family is **incompatible** with the request cell's Classify cell op-category **per
  the KISS-Contract Identity consistency relation** (the cell-op-category ↔ KISS-Ops-op-family
  compatibility table of KISS-CONTRACT §6.3-0006, over the op-name→op-family classification owned
  by KISS-Ops). Wrong-op-class MUST fire **only** when that relation has a row for the request
  cell's op-category **and** the root op's family is absent from that row's compatible set; if no
  request cell is supplied, or the relation has no row for the request cell's op-category, an
  implementation MUST NOT decline `wrong-op-class` on that basis. An implementation MUST reference
  that upstream compatibility relation and MUST NOT re-fork, restate, or redefine the table here.
  *Test:* `test_consume_refusal_wrong_op_class`.
- **KISS-CONSUME-6.4-0005** — **Step 3 — unrecognized-but-expressible (per-region).** An un-lifted
  residue region (steps 1–2 did not fire) MUST be classified **`unrecognized-but-expressible`**
  when the **KISS-Conform expressibility oracle** at the referenced KISS-Ops op-set version judges
  the region **expressible** — its region signature is a **member of the enumerated set of
  KISS-Ops-decomposable region signatures** published for that op-set version — although **this**
  lifter emitted no node for it (a lifter-coverage gap, not a fundamental limit). An implementation
  MUST distinguish this category from `inexpressible-residue` on exactly this **decidable
  membership test**, not on an open existential search (downstream: the byte form of a region
  signature and the set is pinned by KISS-Conform Appendix F). *Test:*
  `test_consume_refusal_unrecognized_but_expressible`.
- **KISS-CONSUME-6.4-0006** — **Step 4 — inexpressible-residue (per-region).** An un-lifted residue
  region whose region signature is **not** a member of the enumerated KISS-Ops-decomposable
  region-signature set at the referenced op-set version (per the same expressibility oracle; steps
  1–3 did not fire) MUST be classified **`inexpressible-residue`** — the truly un-liftable remainder
  — and MUST be recorded honestly as residue (`semantics_kind = declared-op-tag` plus the recorded
  entry, §6.3), never faked into a machine-checkable-IR Semantics it does not have (downstream: the
  byte form of a region signature and the set is pinned by KISS-Conform Appendix F). *Test:*
  `test_consume_refusal_inexpressible_residue`.
- **KISS-CONSUME-6.4-0007** — Category assignment MUST itself be **structure-based** (§6.1): an
  implementation MUST NOT decide `not-a-kernel`, `wrong-op-class`,
  `unrecognized-but-expressible`, or `inexpressible-residue` by substring, keyword, symbol, or
  comment sniffing. *Test:* `test_consume_taxonomy_structure_based`.
- **KISS-CONSUME-6.4-0008** — The categories partition into **whole-input typed declines** and
  **residue tags**: `not-a-kernel` and `wrong-op-class` MUST be returned as **typed declines** for
  the request under §6.4 **Phase A** — no contract is produced for that request (the sole exception
  to §6.3-0003) — while `unrecognized-but-expressible` and `inexpressible-residue` MUST be recorded
  as **`lift_residue` entries** on a produced partial contract under **Phase B** (§6.3). Because the
  two declines are evaluated only as the whole-input pre-check (steps 1–2), an implementation MUST
  NOT return a per-region expressible/inexpressible remainder as a whole-input decline, and MUST NOT
  record a whole-input not-a-kernel or wrong-op-class result as a residue entry. *Test:*
  `test_consume_decline_vs_residue_partition`.
- **KISS-CONSUME-6.4-0009** — Every whole-input typed decline (§6.4-0008 Phase A) and every
  `lift_residue` entry (§6.4-0008 Phase B) MUST carry a **`decline_code`** drawn from the **closed**
  normative set `{not-a-region, multi-output, bind-set-mismatch, unknown-op,
  operand-tuple-inexpressible, attrs-can't-carry, out-of-range-index}` — a finer, orthogonal
  refinement of the four-category classification (§6.4-0001) that names the *specific* structural
  failure or lift-miss. The `decline_code` is the part a consumer's next-action decision **binds on**
  — it distinguishes, for example, "retry with a different operand dtype" (`operand-tuple-inexpressible`)
  from "this op has no KISS-Ops name" (`unknown-op`) — so it MUST be a closed, agreed set: an
  implementation MUST NOT emit a `decline_code` outside it. The four categories (§6.4-0001) remain the
  MECE partition; the `decline_code` refines within a frame and MUST be consistent with its category
  (a Phase-A decline carries a structural code; a Phase-B residue carries the lift-miss code its region
  exhibits). *Test:* `test_consume_decline_code_closed_set`.
- **KISS-CONSUME-6.4-0010** — A `lift_residue` entry (§6.4-0008 Phase B) and a missing-kernel
  honest-miss frame MAY additionally carry an informative **`blocker_reason`** — a **set** (a single
  lift-miss can couple multiple blockers, e.g. a `gather` blocked by both the OpAttrs channel and the
  operand keying) drawn from an **open registry** seeded with `{vocabulary, attrs-channel, keying,
  determinism, shape-layout-inexpressible, dispatch-envelope}`. `blocker_reason` is purely
  **informative** — a diagnostic that lets a consumer route its next action (fallback-decompose /
  request-op-addition / try-another-provider / retry) or emit telemetry — and MUST NEVER be a binding
  gate: no conformance or interop decision keys on it (that is the `decline_code`'s role, §6.4-0009).
  The registry is **open** so any ecosystem may name a novel blocker without a spec change, but a party
  whose blocker matches a seed entry **MUST use the seed's spelling** (the recommended-spelling rule —
  no `attrs-channel` vs `attr_gap` vs `no-attrs` for one blocker); a genuinely new reason is additive.
  The principle: **close the normative gate (`decline_code`), open the informative diagnostic
  (`blocker_reason`).** *Test:* `test_consume_blocker_reason_open_seeded_set`.

### 6.5 Typed decline, never panic

- **KISS-CONSUME-6.5-0001** — Every refusal — `not-a-kernel`, `wrong-op-class`, a malformed or
  empty input, an unsupported referenced version, or an internal recognition failure — MUST be a
  **typed decline** (a distinguished error value/enumerant or out-of-band error return) and MUST
  NOT be a panic, abort, crash, hang, or out-of-bounds read. *Test:*
  `test_consume_refusal_is_typed_decline`.
- **KISS-CONSUME-6.5-0002** — A lifter MUST NOT panic, abort, crash, hang, read outside the
  serialized input, or allocate on an unchecked declared length on **any** input, including a
  malformed input structure graph (cyclic operand edges, dangling or out-of-range operand indices,
  unbounded fan-out, a node with a missing operand), a truncated serialization, an empty input, or
  otherwise adversarial input; it MUST return either a typed decline (§6.5-0001) or an honest
  zero-lift / partial-lift residue contract (§6.3). *Test:*
  `test_consume_never_panic_on_adversarial_input`.
- **KISS-CONSUME-6.5-0003** — An unrecognized region MUST NOT cause a lifter to **guess** or
  **hallucinate** an op node: an implementation MUST NOT emit a lifted node it did not derive
  from the region's structure, and MUST instead record the region as residue (§6.3-0002) or,
  where nothing was op-bearing, decline `not-a-kernel` (§6.4-0003). *Test:*
  `test_consume_no_hallucinated_node`.

### 6.6 The emit/consume round-trip (two tiers)

> The clauses §6.6-0001 through §6.6-0005 state the two-tier round-trip in text that is
> **semantically identical** (same clause intent, same normative effect) to the corresponding
> clauses §6.7-0001 through §6.7-0005 of KISS-Emit so the two inverse directions cannot drift. The
> wording is deliberately **not** byte-identical: each side neutralizes the other's
> vendor/source-language illustrations, so the two directions can never be a byte-for-byte diff.
> That cross-document semantic correspondence is a **governance/CI invariant** enforced by the
> KISS-Emit **§6.7-0008 clause-correspondence-table lint** (`test_conform_emit_consume_roundtrip_correspondence`),
> which pairs each Consume round-trip clause with its Emit counterpart and checks that the pair
> states one invariant — it is **not** an implementation-conformance obligation and is not
> exercisable by any per-implementation KISS-Conform behavior test; KISS-Consume does not depend on
> KISS-Emit. Any edit to these clauses (including the fixes applied in this revision — the removal
> of "neutral" in §6.6-0001 and of the source-language example from the §6.6-0003 clause body) must
> be mirrored in KISS-Emit's corresponding clauses in lockstep to preserve the invariant. (Consume
> §6.6-0006, the whole-kernel tier aggregation rule, has no §6.7-0001..0005 counterpart; its
> mirroring is tracked separately against KISS-Emit.)

- **KISS-CONSUME-6.6-0001** — **TIER 1 — structural round-trip:** emit-then-lift (or
  lift-then-emit) reproduces the SAME KISS-Ops op DAG under STRUCTURAL / op-DAG EQUALITY, checked
  over a DECLARED SUBSET of ops (the subset each side declares it round-trips, not the whole op
  set). This is the always-claimable tier and the one interop actually rests on: two parties agree
  on what an OpDef MEANS structurally. It is language-independent because it compares KISS-Ops
  op-DAG structure, not bytes. An implementation MUST support this tier over its declared subset
  as the always-claimable round-trip. *Test:* `test_consume_roundtrip_tier1_structural`.
- **KISS-CONSUME-6.6-0002** — **TIER 2 — numeric round-trip:** bit-identity of the computed result
  is claimed ONLY SAME-LANGUAGE, ON-DEVICE — same source language, same target device — and only
  for the exact-byte determinism class; ULP/tolerance and order-invariant/nondeterministic ops are
  compared under their declared comparator, never for bit identity. This tier is a strict,
  narrowly-scoped add-on to tier 1, never a substitute for it. An implementation MUST NOT claim
  tier-2 bit identity except same-language, on-device, and for the exact-byte determinism class.
  *Test:* `test_consume_roundtrip_tier2_numeric_same_language_ondevice`.
- **KISS-CONSUME-6.6-0003** — Numeric identity is NEVER claimed across languages. Cross-language
  round-trip is TIER 1 (structural) ONLY — an op's implementation in one source language is not
  guaranteed bit-identical to its implementation in another source language, and overclaiming
  cross-language numeric identity is a named trap. Across languages the guarantee stops at
  structural op-DAG equality over the declared subset. An implementation MUST NOT claim
  cross-language numeric identity. *Test:* `test_consume_roundtrip_no_cross_language_numeric`.
- **KISS-CONSUME-6.6-0004** — Which round-trip tier applies to a given op MUST be selected by the
  KISS-Ops determinism/fidelity enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`
  (KISS-OPS §6.0-0001), imported verbatim: an `exact-byte` op is compared for tier-2 bit identity
  only under §6.6-0002's same-language on-device restriction, while `ULP/tolerance` and
  `order-invariant/nondeterministic` ops MUST be compared under their declared comparator and MUST
  NOT be compared for bit identity. An implementation MUST NOT re-fork this enum. *Test:*
  `test_consume_roundtrip_tier_selected_by_determinism_enum`.
- **KISS-CONSUME-6.6-0005** — KISS-Consume and KISS-Emit MUST be treated as **DAG siblings** with
  **no** dependency edge between them: an implementation MUST NOT require KISS-Emit to perform a
  KISS-Consume lift, and MUST NOT make a round-trip claim (§6.6-0001–§6.6-0003) a prerequisite of
  producing a contract's Semantics field (§6.2). The round-trip is a **join** verified by
  KISS-Conform, not an edge in the dependency DAG (umbrella §2.2). *Test:*
  `test_consume_emit_are_siblings_no_edge`.
- **KISS-CONSUME-6.6-0006** — A **whole-kernel** round-trip claim MUST aggregate per-op tiers: a
  whole-kernel **tier-2** bit-identity claim is admissible **only** when **every** op in the
  resolved op DAG is `exact-byte` (and the same-language, on-device restriction of §6.6-0002 holds).
  If any op in the resolved DAG is `ULP/tolerance` or `order-invariant/nondeterministic`, the
  whole-kernel round-trip MUST be claimed as **tier-1 (structural) only**. An implementation MUST
  NOT assert whole-kernel tier-2 bit identity for a DAG containing a non-`exact-byte` op. *Test:*
  `test_consume_roundtrip_whole_kernel_aggregation`.

---

## 7. Capability, Profile & Extension model

KISS-Consume adopts the umbrella capability/profile/extension model (umbrella §6) and adds
nothing to the negotiation machinery; it pins only the mandatory core and the one negotiable
feature below. Reserved bit ranges follow the umbrella u64 three-axis split (umbrella §6.2);
KISS-Consume reserves no new axis and expresses no range as a computed fraction.

### 7.1 Mandatory core

- **KISS-CONSUME-7.1-0001** — The **mandatory core** of KISS-Consume, which every conforming
  implementation MUST satisfy regardless of claimed options, is: structure-based recognition over
  the input structure graph (§6.1), the lift-into-the-op-DAG discipline bottoming out at the
  primitive floor (§6.2), the residue contract (§6.3), the four-category MECE refusal taxonomy
  (§6.4), and the typed-decline-never-panic obligation (§6.5). An implementation that cannot
  satisfy this core does not conform to KISS-Consume at all, and an out-of-core or unrecognized
  input MUST produce a typed decline or an honest residue contract, never a panic (§6.5). *Test:*
  `test_consume_mandatory_core`.

### 7.2 Negotiable feature — the declared round-trip subset

- **KISS-CONSUME-7.2-0001** — The **declared round-trip subset** (the set of ops an implementation
  claims it round-trips structurally, §6.6-0001) is a **negotiable, advertised** capability, not
  part of the mandatory core. An implementation MUST advertise its declared subset, when it makes a
  tier-1 round-trip claim, as a **canonically-ordered, byte-exact list of KISS-Ops op names** carried
  in the capability channel (umbrella §6.2), and MUST NOT claim a structural round-trip for an op
  outside its advertised subset. A tier-1 round-trip claim between a Consume declared subset and an
  Emit declared subset holds over the **intersection (overlap)** of the two subsets, not over their
  equality; an implementation MUST NOT extend the claim to an op outside that intersection. *Test:*
  `test_consume_declared_subset_advertised`.

---

## 8. Versioning & Lifecycle

### 8.1 Two version axes

KISS-Consume carries the two umbrella version axes (umbrella §5.1) and does not conflate them:

- **Schema version** — the integer stamping the observable KISS-Consume behavior surface: the
  input-structure-graph representation (§6.1-0004), the refusal-taxonomy category set and spellings
  (§6.4), the residue-recording schema (§6.3), and the round-trip tier statements (§6.6). It changes
  only when that surface changes. It is the axis KISS-Conform keys conformance on.
- **Published-crate semver** — the ordinary semantic version of the reference crate(s); it changes
  on any code release, including recognition-coverage improvements that broaden the lifted portion
  without changing the observable behavior surface.

- **KISS-CONSUME-8.1-0001** — A change that broadens the recognition coverage (lifts an idiom
  previously left as `unrecognized-but-expressible` residue) without changing the refusal-taxonomy
  category set, the residue schema, the input-structure-graph representation, or the round-trip tier
  statements MUST bump **only** the published-crate semver and MUST NOT bump the KISS-Consume schema
  version; a change to the category set/spellings, the residue schema, the input-structure-graph
  representation, or a round-trip tier statement MUST bump the schema version (and, consequently, the
  crate semver). *Test:* `test_consume_version_bump_rule`.
- **KISS-CONSUME-8.1-0002** — Because the `unrecognized-but-expressible` ↔ `inexpressible-residue`
  boundary is **op-set-version-relative** (adding a high-level KISS-Ops op can move a region from
  inexpressible to expressible), a recorded residue determination MUST be interpreted against the
  KISS-Ops op-set version it recorded (§6.3-0005); an implementation MUST NOT treat a residue
  determination as valid under a different op-set version without re-evaluating it. *Test:*
  `test_consume_residue_opset_relative`.

### 8.2 Maturity & freeze gate

KISS-Consume is at maturity stage **Draft**. It advances Draft → Frozen only through the umbrella
freeze gate (umbrella §5.3): at least two structurally dissimilar lifters interoperate on the golden
lift/refusal vectors — authored in the input structure graph of §6.1-0004 so a single vector is
feedable to both — a non-native foreign reader reproduces or parses the exact recorded
residue/refusal tokens under the residue-entry schema of §6.3-0008, and the KISS-Consume KISS-Conform
suite exists and passes with complete bidirectional clause-to-test traceability. The **KISS-Conform
AUDIT role signs the freeze transition**, not the authoring editor. The remaining open questions of
Appendix D — in particular the shared-pen risk on the identical round-trip wording (§6.6) and the
residue re-evaluation workflow across op-set bumps — should be resolved before freeze; the declared
round-trip subset advertisement (§7.2-0001), previously an open question, was pinned in this revision.

---

## 9. Conformance

### 9.1 Claim format & prerequisite closure

- **KISS-CONSUME-9.1-0001** — A KISS-Consume conformance claim MUST state the KISS-Consume schema
  version claimed and MUST be **prerequisite-closed** over the incoming **STRUCTURAL** edges: a
  claim to KISS-Consume MUST also claim KISS-Classify, KISS-Ops, and KISS-Contract (umbrella §2.2
  edge table, §6.3). An implementation MUST NOT claim KISS-Consume without claiming those three
  structural prerequisites, and MUST NOT be required to claim KISS-Emit (a sibling, not a
  prerequisite, §6.6-0005). *Test:* `test_consume_claim_prerequisite_closed`.
- **KISS-CONSUME-9.1-0002** — An input outside a claimed subset MUST produce a **typed decline or
  an honest residue contract**, never a panic (§6.5); KISS-Conform tests both that the claim's own
  clauses pass **and** that out-of-claim inputs decline or residue cleanly. *Test:*
  `test_consume_out_of_claim_declines_cleanly`.

### 9.2 Clause → KISS-Conform test traceability matrix

Every normative clause maps 1:1 to at least one named KISS-Conform test; the suite build FAILS on
any normative MUST without a mapped test and on any test citing a retired or non-existent clause ID
(umbrella §3.3).

| Clause ID | Named KISS-Conform test |
|---|---|
| KISS-CONSUME-6.0-0001 | `test_consume_determinism_class_exact_byte` |
| KISS-CONSUME-6.1-0001 | `test_consume_recognition_structure_based` |
| KISS-CONSUME-6.1-0002 | `test_consume_recognition_renaming_invariant` |
| KISS-CONSUME-6.1-0003 | `test_consume_source_syntax_out_of_scope` |
| KISS-CONSUME-6.1-0004 | `test_consume_input_structure_graph` |
| KISS-CONSUME-6.2-0001 | `test_consume_lift_target_is_ops_dag` |
| KISS-CONSUME-6.2-0002 | `test_consume_lift_resolves_to_floor` |
| KISS-CONSUME-6.2-0003 | `test_consume_full_lift_produces_machine_checkable_ir` |
| KISS-CONSUME-6.2-0004 | `test_consume_produces_no_artifact` |
| KISS-CONSUME-6.2-0005 | `test_consume_records_structure_key_on_identity` |
| KISS-CONSUME-6.2-0006 | `test_consume_op_identity_from_dag_root` |
| KISS-CONSUME-6.2-0007 | `test_consume_lift_node_carries_opattrs` |
| KISS-CONSUME-6.2-0008 | `test_consume_lift_input_signature` |
| KISS-CONSUME-6.3-0001 | `test_consume_completeness_tracks_lift_fraction` |
| KISS-CONSUME-6.3-0002 | `test_consume_residue_honest_never_faked` |
| KISS-CONSUME-6.3-0003 | `test_consume_residue_never_withholds` |
| KISS-CONSUME-6.3-0004 | `test_consume_residue_entry_tagged` |
| KISS-CONSUME-6.3-0005 | `test_consume_residue_records_opset_version` |
| KISS-CONSUME-6.3-0006 | `test_consume_handwritten_semantics_via_lifting` |
| KISS-CONSUME-6.3-0007 | `test_consume_no_silent_drop_of_residue` |
| KISS-CONSUME-6.3-0008 | `test_consume_residue_entry_schema` |
| KISS-CONSUME-6.4-0001 | `test_consume_taxonomy_four_categories` |
| KISS-CONSUME-6.4-0002 | `test_consume_taxonomy_mece_ordered` |
| KISS-CONSUME-6.4-0003 | `test_consume_refusal_not_a_kernel` |
| KISS-CONSUME-6.4-0004 | `test_consume_refusal_wrong_op_class` |
| KISS-CONSUME-6.4-0005 | `test_consume_refusal_unrecognized_but_expressible` |
| KISS-CONSUME-6.4-0006 | `test_consume_refusal_inexpressible_residue` |
| KISS-CONSUME-6.4-0007 | `test_consume_taxonomy_structure_based` |
| KISS-CONSUME-6.4-0008 | `test_consume_decline_vs_residue_partition` |
| KISS-CONSUME-6.4-0009 | `test_consume_decline_code_closed_set` |
| KISS-CONSUME-6.4-0010 | `test_consume_blocker_reason_open_seeded_set` |
| KISS-CONSUME-6.5-0001 | `test_consume_refusal_is_typed_decline` |
| KISS-CONSUME-6.5-0002 | `test_consume_never_panic_on_adversarial_input` |
| KISS-CONSUME-6.5-0003 | `test_consume_no_hallucinated_node` |
| KISS-CONSUME-6.6-0001 | `test_consume_roundtrip_tier1_structural` |
| KISS-CONSUME-6.6-0002 | `test_consume_roundtrip_tier2_numeric_same_language_ondevice` |
| KISS-CONSUME-6.6-0003 | `test_consume_roundtrip_no_cross_language_numeric` |
| KISS-CONSUME-6.6-0004 | `test_consume_roundtrip_tier_selected_by_determinism_enum` |
| KISS-CONSUME-6.6-0005 | `test_consume_emit_are_siblings_no_edge` |
| KISS-CONSUME-6.6-0006 | `test_consume_roundtrip_whole_kernel_aggregation` |
| KISS-CONSUME-7.1-0001 | `test_consume_mandatory_core` |
| KISS-CONSUME-7.2-0001 | `test_consume_declared_subset_advertised` |
| KISS-CONSUME-8.1-0001 | `test_consume_version_bump_rule` |
| KISS-CONSUME-8.1-0002 | `test_consume_residue_opset_relative` |
| KISS-CONSUME-9.1-0001 | `test_consume_claim_prerequisite_closed` |
| KISS-CONSUME-9.1-0002 | `test_consume_out_of_claim_declines_cleanly` |

---

## 10. Governance

KISS-Consume adopts the umbrella governance model (umbrella §7) and legal/IP posture (umbrella §9)
by reference and restates none of it. The **editor of record** is **Unpopped**, the neutral
kernel-generator reference project, **ratified 2026-08-15** for both KISS-Consume and its inverse
KISS-Emit; the identical round-trip wording of §6.6 therefore has a **single pen** across the two,
and the KISS-Emit §6.7-0008 correspondence-table lint remains the mechanical check (Appendix D).
The editor is also an implementer of both directions — a fact about who holds the pen, not a
conformance claim; conformance is self-certified with published results like anyone else's. The
two constraints that bound
that identity — a recorded, current conformance status against the clauses it edits, and a
non-editor cosignatory for round-trip-table clauses and for any clause changed because the
reference implementation found it difficult — are stated in KISS-Emit §10 and apply here
identically. Maturity-stage advances are signed by the
**KISS-Conform AUDIT role** jointly with the editor (umbrella §7.3), not by the authoring editor
alone. The **specification text** is CC0 1.0 Universal (umbrella §9.1); the **reference crates** are
MIT-OR-Apache-2.0 (umbrella §9.2); the **conformance suite** carries the mark policy of umbrella
§9.3; the **patent grant** is the royalty-free grant with defensive termination of umbrella §9.4.
Project and product names in this document are confined to non-normative examples, provenance, and
the reference-implementation pointer; the normative clauses use only the generic roles provider,
consumer, implementation, kernel, contract, and target.

---

## Appendix A — Worked lift & refusal vectors (informative)

- **A.1 Full lift (strided binary `add`).** Structure: two `f32` operand reads → one `add` → one
  store, strided cell, `target = cuda:sm89`. Lift → one-node DAG `{ op: add }`;
  `semantics_kind = machine-checkable-IR`; Identity `op_identity = add` (bare KISS-Ops name, form
  (b)), `accept_predicate = <structure_key sk4|bin|f32|…>` (cell op-category code `bin`); residue =
  ∅ (lift fraction, informally, ≈ 1.0).
- **A.2 Partial lift with inexpressible residue.** Structure: `matmul` → proprietary activation with
  no KISS-Ops decomposable-signature entry. Lift → `{ op: matmul }` node; `semantics_kind =
  declared-op-tag`; one `lift_residue` entry (one maximal connected un-lifted component) over the
  activation region tagged `inexpressible-residue` at op-set version `<v>`; all seven contract
  sections present.
- **A.3 Partial lift with expressible residue.** Same structure but the second region is an
  `erf`-based `gelu` this lifter does not yet recognize; the `lift_residue` entry is tagged
  `unrecognized-but-expressible` at op-set version `<v>` (the expressibility oracle's decomposable
  set includes its signature, over `mul`/`add`/`erf`); broadening the lifter later converts it to a
  lifted node.
- **A.4 not-a-kernel.** Input structure graph is a comment block / data table / empty region → zero
  operation nodes → typed decline `not-a-kernel` (decided from the input alone).
- **A.5 wrong-op-class.** Request cell op-category code `red`; lifted root op `matmul` (KISS-Ops
  op-family `contraction`, absent from the compatible set for the `red` cell op-category per the
  KISS-Contract Identity consistency relation) → typed decline `wrong-op-class`. (The cell code
  `gem` and the op-family `contraction` are distinct closed sets and are never conflated.)
- **A.6 Round-trip.** Emit `(OpDef add + structure_key)` → artifact-described-by-a-contract; lift
  that kernel → the same one-node `{ op: add }` DAG (**tier 1**, structural equality over the
  declared subset); same-language on-device, `add` being `exact-byte` and the only op in the DAG,
  the whole-kernel computed result matches bit-for-bit (**tier 2**, admissible under §6.6-0006);
  cross-language, only tier 1 is claimed.

## Appendix B — Glossary (informative)

- **Lift fraction** — informal, unquantified descriptor of how much of a kernel was lifted into the
  op DAG; only the residue-empty vs. residue-non-empty distinction is normative (§6.3-0001).
- **MECE** — mutually exclusive, collectively exhaustive; the property the four refusal categories
  hold via the two-unit ordered decision procedure (§6.4-0002).
- **Residue** — the recorded un-liftable remainder on a partial contract (the
  `unrecognized-but-expressible` ∪ `inexpressible-residue` regions left on a run, one entry per
  maximal connected un-lifted component, §6.3-0008).
- **Input structure graph** — the syntax-free data-flow representation a lifter consumes; the
  representation golden vectors are authored in (§6.1-0004).
- **Declared subset** — the set of ops an implementation claims it round-trips structurally
  (§6.6-0001, §7.2-0001).
- **Sibling (inverse)** — KISS-Emit; the generation direction, sharing the round-trip but with no
  dependency edge (§6.6-0005).
- **Structure-based recognition** — recognition decided by computational structure, not surface
  syntax (§6.1); the anti-sniffing rule.

## Appendix C — Provenance / acknowledgments (informative)

The KISS-Consume recognition/lift direction generalizes an existing kernel-decomposition capability
in the reference generator project (a source→IR lift) and an existing runtime kernel-decomposer, both
of which are KISS-Consume lifters under this sub-standard: standardizing the neutral op vocabulary
makes them target the same KISS-Ops op DAG, so either can produce a contract's Semantics field. The
reference seed crate is *a* reference implementation with no privilege; its project/crate name is
recorded here as non-normative provenance only.

## Appendix D — Open questions (informative)

1. **Declared round-trip subset — RESOLVED in this revision.** The advertisement of the declared
   subset and the equal-vs-overlap rule are now pinned in §7.2-0001: the subset is advertised as a
   canonically-ordered, byte-exact list of KISS-Ops op names in the capability channel, and a tier-1
   round-trip claim holds over the **intersection (overlap)** of the two sides' declared subsets, not
   over their equality. No longer freeze-blocking.
2. **Shared pen on the identical round-trip wording — RESOLVED (2026-08-15).** One editor now
   holds both KISS-Consume and KISS-Emit, so the no-single-pen risk this item recorded no longer
   applies. The semantic correspondence remains enforced by the KISS-Emit §6.7-0008
   clause-correspondence-table lint, not by any per-implementation conformance test (§6.6); the two
   blocker/minor fixes applied to §6.6-0001 and §6.6-0003 in this revision must still be mirrored in
   KISS-Emit in lockstep. **What remains open is a design question, not a governance one:** whether
   the round-trip statement should live in a shared section both documents cite. This document
   carries no correspondence table of its own and defers to KISS-Emit's in three places, and the
   lint binds only §6.7-0001..-0005 — so §6.7-0006, which defines the tier-2 determinants both
   documents use, is outside it. A single pen makes drift less likely; a shared section would make
   the asymmetry impossible.
3. **Residue honesty across op-set version bumps.** The `unrecognized-but-expressible` ↔
   `inexpressible-residue` boundary is op-set-version-relative; §6.3-0005 / §8.1-0002 stamp each
   residue entry with its determining op-set version, but the re-evaluation/migration workflow when
   an op set adds a high-level op is not yet fully specified.
4. **wrong-op-class vs. the Contract Identity consistency relation (cross-standard coordination).**
   The wrong-op-class category leans on the `structure_key` op-category ↔ KISS-Ops op-family
   compatibility table owned upstream by KISS-Contract/KISS-Ops (§6.4-0004); the exact table
   membership is defined there and must not be re-forked here — the cross-reference must remain the
   single source. Two coordination items are tracked against the upstream table before freeze: (a) it
   must be made **total** over the full KISS-Classify cell op-category domain, since a cell
   op-category with no row would otherwise force every kernel in it to wrong-op-class — §6.4-0004
   already forbids declining wrong-op-class when the relation has no row for the request cell's
   op-category, so Consume does not inherit that defect, but the upstream table should be completed;
   and (b) the cell op-category MUST be spelled with its Classify code (e.g. the 3-letter code)
   verbatim in the Contract table and in every Consume reference, never conflated with a KISS-Ops
   op-family name (e.g. cell code `gem` vs. op-family `contraction`).
5. **Residue-bearing tag set — cross-standard reconciliation.** §6.3-0004 defines the residue-bearing
   category set once as the two-token subset {`unrecognized-but-expressible`, `inexpressible-residue`}.
   KISS-Contract's `lift_residue` clause must reference that subset and must not enumerate the wider
   four-category set (which would let a provider emit a `not-a-kernel`/`wrong-op-class` residue tag a
   conforming consumer would reject); this is an upstream reconciliation item tracked before freeze.

---

*End of KISS-Consume (Draft proposal). This sub-standard is informative through §5 and normative in
§6+; every binding requirement is an identified clause `KISS-CONSUME-<section>-<nnnn>` mapped 1:1 to
a named KISS-Conform test. KISS-Consume owns recognition (lift), depends structurally only on
KISS-Ops, KISS-Classify, and KISS-Contract, and is the inverse of KISS-Emit with which it shares —
but on which it does not depend — the two-tier round-trip of §6.6.*
