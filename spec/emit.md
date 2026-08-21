# KISS-Emit — The Kernel Generation Direction & The Lowering Partition

**Sub-standard ID:** KISS-EMIT
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification* (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ is normative. Normative
> clauses use RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-EMIT-<section>-<nnnn>`, and each MUST/SHALL maps 1:1 to at least one
> named KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Emit |
| Sub-standard ID | KISS-EMIT |
| Tier | **Protocol** (the generation direction: it owns how an op definition plus a specialization-cell identity is lowered into a callable kernel described by a contract; it sits above the two foundational vocabularies and the middle-tier contract format it produces, and it is the inverse of the recognition direction KISS-Consume). |
| Maturity stage | **Draft** (first-draft proposal; the lowering partition and the emit/consume round-trip are NOT frozen — the freeze gate of §8 is unmet, and the blocking pre-freeze neutrality audit of §6.5 has not yet been performed). |
| Editor of record | **Unpopped** — the neutral generator reference-impl project — **ratified 2026-08-15**, holding the pen for **both** KISS-Emit and its inverse KISS-Consume. The two inverse standards now share a **single pen**, which is the primary guard on the round-trip statement of §6.7; §6.7-0008 and the §10 pen-drift guard remain in force as the mechanical check, since one pen makes drift less likely rather than impossible. **Editor–implementer identity here is deliberate and bounded.** The editor is also an implementer of this specification — which is a fact about who holds the pen, not a conformance claim — so: **(a)** the editor's own conformance status against the clauses it edits is recorded in Appendix A.1 and kept current; and **(b)** a change to any clause enumerated in the §6.7-0008 correspondence table — or to any clause whose alteration is motivated by the reference implementation's own difficulty — **requires a cosignatory who is not the editor.** Constraint (b) was proposed by the editor about itself. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | a kernel-generation reference crate — **`unpopped`**, with its vocabulary crate `unpopped-vocab` and its IR→Slang emitter `unpopped-slang`, which demonstrates an emitter whose surface spellings differ from a C-family emitter's (crate names given in Appendix A as non-normative provenance). This crate is *a* reference implementation with no privilege. **Two limits on reading it as an exemplar.** It **does not today emit a KISS-Contract** — it emits a consumer-specific contract format — so it seeds the lowering partition of §6.2–§6.5 and is **not** an exemplar of the §6.6 contract-pairing obligation. And its `Backend` trait is a **lowering** interface, narrower than this document's **emitter**, which per the Tier row above owns contract production; mapping the one onto the other silently omits obligations §6 places on the emitter. |
| DAG position | **Protocol tier.** Depends **STRUCTURALLY** on KISS-Ops (the lowering source is a KISS-Ops op definition), KISS-Classify (the specialization-cell identity is a `structure_key`), and KISS-Contract (the emitter's output is described by a contract). It is a **sibling** of KISS-Consume (the inverse recognition direction): the two share the round-trip of §6.7 but neither depends on the other. Not a root; nothing in the suite depends on KISS-Emit except KISS-Conform (test dependency). |
| Upstream edges | KISS-Ops (**STRUCTURAL** — the normative lowering source is a KISS-Ops **OpDef**, resolvable to the KISS-Ops primitive floor; the determinism/fidelity enum and the MathPrecision attribute the emitted kernel declares are imported verbatim from KISS-Ops, never re-forked); KISS-Classify (**STRUCTURAL** — the second half of the normative input is a KISS-Classify `structure_key` specialization-cell identity, carried verbatim; the emitted kernel's operand descriptors and `target_capability` are Classify vocabulary); KISS-Contract (**STRUCTURAL** — the emitter's output is described by a seven-section KISS-Contract, whose Interface + Dispatch pin the emitted ABI and whose Guarantees declare the emitted kernel's fidelity) |
| Downstream edges | KISS-Conform (test dependency — Conform tests this sub-standard, resolves the emitted kernel's Semantics DAG to the primitive floor as the oracle, and runs the round-trip of §6.7 under the determinism-class comparators). KISS-Emit has **no** other downstream edge. |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |

> **Edge-label note (informative).** All three KISS-Emit upstream edges are
> **STRUCTURAL**: KISS-Emit parses the internal structure of a KISS-Ops op
> definition (its name, OpAttrs channel, reference decomposition, determinism class,
> and MathPrecision attribute), of a KISS-Classify `structure_key` / operand
> descriptor / `target_capability`, and of the KISS-Contract seven-section document
> its output is described by. The labels reconcile with the umbrella §2.2 edge
> table, which lists **KISS-Ops → KISS-Emit**, **KISS-Classify → KISS-Emit**, and
> **KISS-Contract → KISS-Emit** each as STRUCTURAL. KISS-Emit has **no** edge to or
> from KISS-Consume: the two inverse directions are DAG **siblings**, both depending
> only on KISS-Ops + KISS-Classify + KISS-Contract, joined not by a dependency edge
> but by the shared round-trip of §6.7. KISS-Emit likewise does **not** depend on
> KISS-Announce or KISS-Synth/Provision; a just-in-time builder is a KISS-Emit
> emitter *reached through* KISS-Synth provision, but the reachability is a
> KISS-Synth concern and creates no edge into KISS-Emit.

---

## 1. Purpose & Scope

KISS-Emit owns the **generation direction** of the suite: given a computation to
build and the specialization cell to build it for, produce a callable kernel. Its
one job is to make lowering **auditable and vendor-neutral** by pinning, exhaustively,
*who spells what*. It defines three things and nothing else:

1. **The normative lowering input** — an **op definition** (a KISS-Ops `OpDef`)
   **paired with** a **specialization-cell identity** (a KISS-Classify
   `structure_key`): the pair `(OpDef + structure_key)`. This is the whole input a
   conforming emitter is entitled to, and explicitly **not** any implementation's
   schedule-resolved plan.

2. **The complete lowering partition** — a total, disjoint partition of *every*
   lowering decision into exactly two sets: **the neutral driver MAY spell it**
   (target-independent structure a neutral driver emits) versus **the emitter MUST
   supply it** (per-target surface an emitter is the only party allowed to render).
   Constant spelling and special-float-value spelling are **emitter-supplied by
   construction**, never driver-assumed.

3. **The emit/consume round-trip** — a two-tier fidelity statement, shared word-for-
   word with the inverse sub-standard KISS-Consume: **tier 1** is structural op-DAG
   equality over a declared subset (always claimable, language-independent); **tier
   2** is numeric bit-identity, claimable **only** same-language and on-device, and
   **only** for the exact-byte determinism class. Cross-language numeric identity is
   never claimed.

**KISS-Emit is NOT:** a source language or a source grammar (the languages a kernel
is emitted *into* are out of scope; KISS-Emit neither defines nor blesses any target
syntax — it partitions *who* renders it, never *how the syntax reads*); a **scheduler**
(the normative input is `(OpDef + structure_key)`, never a schedule-resolved plan, so
no tiling/fusion/loop-order scheduler is dragged into the ABI); the recognition/lift
direction (that is KISS-Consume, the inverse sub-standard KISS-Emit shares a round-trip
with but does not depend on); the op set or per-op semantics (KISS-Ops, resolved from
there, never restated); the data vocabulary or the `structure_key` machinery
(KISS-Classify, used by name/structure); the contract document format (KISS-Contract,
which *describes* the emitter's output); the provision protocol (KISS-Synth/Provision,
which *reaches* an emitter on a build-on-miss); a kernel implementation; or the
internals of any implementation's intermediate representation. Anything not enumerated
as in-scope above is out of scope for KISS-Emit (scope creep by silence is a named
trap; silence is not inclusion — and here the trap has a specific bite: a third,
"implicit" lowering-decision bucket is forbidden by the closure rule §6.2-0004).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — partition the pen, not the syntax

An emitter turns a computation into a kernel written in some target language. The
danger is not that different emitters produce different bytes — that is expected and
fine — but that a supposedly neutral driver silently bakes in **one language's
incidental spelling** and then everyone downstream inherits it. The C-family
happy-path is the classic leak: a decimal literal `0.5f`, an infix `a + b`, a
`sqrtf(x)` call all *look* neutral, so they get spelled in "the driver" and the
standard quietly becomes C-shaped.

KISS-Emit's whole discipline is to **partition the pen**. Every lowering decision is
placed in exactly one of two sets:

- **the neutral driver MAY spell it** — the decision is either **structural** (it
  renders no target-language surface at all — it fixes what is computed and in what
  order, not the concrete tokens) or a spelling the neutrality audit has **proven**
  target-independent, so a single neutral driver can emit it for all targets; or
- **the emitter MUST supply it** — the decision has a per-target surface, so only the
  emitter (which knows the target language) may render it.

The partition is **complete** (every decision lands somewhere — anything not
explicitly enumerated on the driver side is emitter-supplied by default, §6.2-0004)
and **disjoint** (no decision lands in both, §6.2-0001), with **no third "implicit"
bucket** — a decision nobody owns is exactly how a C-ism leaks in. What KISS-Emit
standardizes is this boundary, audited to be real, not the target syntax on either
side of it.

### 2.2 The normative input is `(OpDef + structure_key)`, never a plan

A conforming emitter is handed two things and only two things:

- an **op definition** (a KISS-Ops `OpDef`) — *what to compute*, resolvable through
  its reference decomposition to the KISS-Ops primitive floor; and
- a **specialization-cell identity** (a KISS-Classify `structure_key`) — *which
  layout/dtype/target cell to build for*.

It is **not** handed a schedule-resolved plan — no tile sizes, no fusion order, no
chosen loop nest. Admitting a plan into the input would drag a **scheduler** into the
ABI, and a scheduler is an implementation's private business, not a suite-wide seam.
The pair `(OpDef + structure_key)` is the exact input for which the driver/emitter
partition is defined: the driver walks the op DAG and the cell's declared strides;
the emitter supplies the per-target surface. Everything a scheduler would decide is
outside the standard.

### 2.3 The complete partition — the two sets

**The neutral driver MAY spell it:**

- the **op-DAG structure / topology** — walking the Semantics DAG node to node is
  **structural**: it renders no target-language surface, so it is neutral by
  construction (audit-exempt);
- **operand binding** in KISS-Classify canonical operand order and the positional
  argument-signature layout the contract Interface pins — **structural**, audit-exempt;
- **index / address arithmetic** over the declared signed strides and base offsets,
  and the grid-stride thread→element mapping from the contract Dispatch model —
  **structural**, audit-exempt;
- **control-flow / loop scaffolding** derived mechanically from the Dispatch launch
  model (count-unit → grid derivation, workgroup sizing) — **structural**, audit-exempt;
- the strictly-**universal infix arithmetic operator spellings** (`+ - * /`) for the
  ordinary arithmetic atoms — this is the **one** driver-side decision that *does*
  render a target-language surface, and it is placed on the driver side **only**
  subject to the pre-freeze neutrality audit (§2.5) that *proves* universality per
  target, never assumes it.

The first four bullets are **structural**: they determine the shape of the
computation, not the concrete tokens rendered into the target language. Because they
render no target-language surface, the neutrality audit does not apply to them — they
cannot leak a C-ism, since they emit no language-specific syntax. Only the last bullet
(the infix operators) renders a surface and is therefore audit-gated.

**The emitter MUST supply it:**

- **CONSTANT spelling** — a constant is pinned by its bit pattern per dtype, and the
  emitter renders the target-language surface for it. `const_lit` is **emitter-
  supplied by construction**: a decimal literal spelling is a per-language incidental
  (the C-ism that leaked through happy-path golden vectors), so it can never live on
  the driver side.
- **SPECIAL-FLOAT-VALUE spelling** — `+inf`, `-inf`, quiet NaN, signaling NaN (the
  non-finite values) together with `+0`, `-0`, and subnormals (which are *finite*
  edge values grouped here because they, too, have no portable decimal spelling) —
  each pinned by bit pattern per dtype and round-tripped exactly; how each is spelled
  in the target language is emitter-supplied.
- **Transcendental / declared-ULP atom and hardware-intrinsic spelling** (`exp`,
  `log`, `sin`, `erf`, `sqrt`, …) — each is a per-target atom to a declared ULP,
  never a mandated polynomial the driver could spell.
- **Target-specific type spelling** and any dtype whose surface form differs by
  language (`f16`/`bf16`/FP8, complex components).
- **Any operator or construct the neutrality audit did not prove universal** across
  targets, and **anything not explicitly enumerated on the driver side** (the closure
  rule, §6.2-0004).

The two sets are exhaustive and disjoint; §6.2 pins the completeness (by closure) and
disjointness as normative obligations, and §6.3 / §6.4 pin the membership.

### 2.4 Why constants and special float values are emitter-supplied

It is tempting to let the driver spell a constant — after all, `0.5` looks the same
everywhere. It is not the same everywhere: a half-precision literal, an FP8 literal, a
signaling-NaN literal, a `-0.0` literal, and a subnormal literal each have a *language-
specific surface* and several have no portable decimal spelling at all. The value that
is stable across targets is the **bit pattern per dtype**; the *spelling* of that bit
pattern is a per-language incidental. So KISS-Emit pins the value as bits (exactly as
KISS-Ops §6.2-0008 pins a `const(bits)` leaf and every special float value) and
assigns the *rendering* to the emitter. This is the **constants-are-emitter-supplied
rule**, and the pre-freeze neutrality audit's first job is to hunt every `const_lit`
sibling that snuck onto the driver side through a happy-path golden vector. (±0 and
subnormals are *finite* values; they are grouped with the non-finite values only
because, like them, they carry no portable decimal spelling.)

### 2.5 The pre-freeze neutrality audit is a blocking gate

The claim "infix `+ - * /` is universal across targets" is *asserted* in §2.3 but
**not yet proven**. The audit's scope is precisely the driver-side decisions that
render a **target-language surface** — at this schema version, that is exactly the
infix operators of §6.3-0004. The **structural** driver-side decisions (topology,
operand binding, index arithmetic, control-flow scaffolding) render no surface and are
**outside** the audit's scope: they are neutral by construction, so there is nothing
for the audit to prove about them. Until the audit proves the infix operators per
target, the driver/emitter boundary for arithmetic operators is **provisional**: any
operator the audit cannot prove identical across every target it claims drops to the
emitter side. The audit is a **blocking pre-freeze gate** — KISS-Emit cannot advance
Draft → Frozen until it is performed and recorded (§8). Its two named jobs are (a)
hunt every `const_lit` sibling on the driver side and (b) test every "this operator is
universal" assumption, moving each unproven operator to the emitter side. Scope creep
by silence — an operator that nobody proved but nobody moved — is the exact failure
the audit exists to catch.

### 2.6 The emit/consume round-trip in two tiers

KISS-Emit lowers `(OpDef + structure_key)` → an artifact described by a contract;
KISS-Consume lifts a kernel/source region → the contract's Semantics op DAG (as far as
it goes) + residue. They are **inverse directions**. The join that keeps them
honest — without a dependency edge between them — is a two-tier round-trip, whose
statement is shared with KISS-Consume (so the two directions cannot drift). The
following block is the shared verbatim illustration; the normative statement (§6.7)
carries the same intent with the vendor-language illustration neutralized to generic
roles:

> **TIER 1 — structural round-trip:** emit-then-lift (or lift-then-emit) reproduces
> the SAME KISS-Ops op DAG under STRUCTURAL / op-DAG EQUALITY, checked over a DECLARED
> SUBSET of ops (the subset each side declares it round-trips, not the whole op set).
> This is the always-claimable tier and the one interop actually rests on: two parties
> agree on what an OpDef MEANS structurally. It is language-independent because it
> compares KISS-Ops op-DAG structure, not bytes.
>
> **TIER 2 — numeric round-trip:** bit-identity of the computed result is claimed
> ONLY SAME-LANGUAGE, ON-DEVICE — same source language, same target device — and only
> for the exact-byte determinism class; ULP/tolerance and order-invariant/
> nondeterministic ops are never claimed bit-identical. This tier is a strict,
> narrowly-scoped add-on to tier 1, never a substitute for it.
>
> How a ULP/tolerance op's result is *evaluated* is KISS-Conform's, not this
> sub-standard's — see §6.7-0002 on the verification mandate excised from that
> clause, and on what its removal leaves unnamed.
>
> Numeric identity is NEVER claimed across languages. Cross-language round-trip is
> TIER 1 (structural) ONLY — Slang `tanh` is not bit-identical to CUDA `tanh`, and
> overclaiming cross-language numeric identity is a named trap. Across languages the
> guarantee stops at structural op-DAG equality over the declared subset. This
> two-tier statement is NORMATIVELY IDENTICAL in KISS-Emit and KISS-Consume (same
> wording, same clause intent) so the two directions cannot drift; both import the
> KISS-Ops determinism/fidelity enum to decide which tier applies per op.

Which tier is even *claimable* for a given op is decided by the KISS-Ops
determinism/fidelity enum imported into the emitted kernel's contract Guarantees:
`exact-byte` unlocks tier 2 (same-language, on-device); `ULP/tolerance` and
`order-invariant/nondeterministic` stop at tier 1's structural equality plus their own
declared comparators. The vendor names above (`Slang`, `CUDA`) appear only in this
informative aside; the normative clauses (§6.7) carry the same statement with the
illustration neutralized to generic roles, and §6.7-0008 pins the two directions'
statements semantically identical via an enumerated clause-correspondence table.

### 2.7 A worked emit — a strided binary `add` on `f32`, target `cuda:sm89`

Input: the KISS-Ops `OpDef` for `add` (a primitive-floor arithmetic atom, determinism
class `exact-byte`, MathPrecision `bit-stable`) paired with the KISS-Classify
`structure_key` for the cell `sk4|bin|f32|cuda:sm89|ix32|grid|r2|…` (three `f32`
operands, strided). The emitter produces:

- **Driver-may-spell** — the one-node op DAG `{ op: add }`; operand binding
  `(const f32* in0, const f32* in1, f32* out, …)` in canonical order; the signed-
  stride index arithmetic and the grid-stride mapping from Dispatch (all structural,
  audit-exempt); and — *pending the §2.5 audit* — the infix `+` for the `add` atom
  (the one surface-bearing driver-side decision).
- **Emitter-must-supply** — the `f32` type spelling in the target language, and (if
  the body carried any) the spelling of every constant and special float value by its
  bit pattern.
- **Output** — an artifact plus the KISS-Contract of §2.5/§2.7 of the KISS-Contract
  sub-standard, whose Guarantees declare determinism `exact-byte` and MathPrecision
  `bit-stable`. Because `add` is `exact-byte`, tier-2 numeric round-trip is claimable
  **same-language on-device**; across languages only tier-1 structural equality holds.

### 2.8 A worked emit — a `softmax`, target `cuda:sm89`

Input: the KISS-Ops `OpDef` for `softmax` (a `normalization`-family op whose
decomposition contains a transcendental `exp` and a floating-point `sum` reduction, so
its determinism class is `order-invariant/nondeterministic`, KISS-OPS §6.0-0004/-0005)
paired with the cell's `structure_key`.

- **Driver-may-spell** — the op DAG topology of the decomposition (`reduce(max)` →
  `sub` → `exp` → `reduce(sum)` → `div`); operand binding and the Dispatch loop
  scaffolding (all structural, audit-exempt).
- **Emitter-must-supply** — the `exp` transcendental atom spelling to a **declared
  ULP** (never a mandated polynomial); the constant spellings, if any.
- **Round-trip** — because `softmax` is `order-invariant/nondeterministic`, tier 2 is
  **not** claimable even same-language; the guarantee is tier-1 structural op-DAG
  equality plus the declared tolerance comparator. Overclaiming a bit-identical
  `softmax` — same language or not — is precisely the named trap.

### 2.9 A typed decline, never a panic

An emitter that is asked for a dtype it does not support, or an op it cannot resolve
to the KISS-Ops primitive floor, returns a **typed decline** — a structured refusal —
never a panic, abort, crash, hang, or out-of-bounds read. This is the same failure
currency KISS-Synth uses on the provision path and KISS-Consume uses in its refusal
taxonomy; on the just-in-time build path an emitter reached through KISS-Synth
provision answers a build failure as a typed decline that KISS-Synth surfaces as its
`CANNOT_PROVISION` provision decline.

### 2.10 Terms are joined, not restated

KISS-Emit references the KISS-Ops op names, OpAttrs channel, reference decompositions,
primitive floor, the determinism/fidelity enum, and the MathPrecision attribute by
name; the KISS-Classify `structure_key`, operand descriptors, canonical operand order,
and `target_capability` by name/structure; and the KISS-Contract seven-section document
(Identity, Semantics, Interface, Dispatch, Capabilities, Guarantees, Provenance) by
name/structure. It re-defines none of them and defines no op meaning, no data noun, and
no contract field: KISS-Emit owns only the lowering partition and the round-trip.

### 2.11 Exercising the partition outside one language family (informative)

The freeze gate (§8.2) requires, among its preconditions, an emitter whose constant
and operator surface spellings **differ** from the reference emitter's, as established
by the §6.5 audit. The purpose is to exercise the partition outside the C-family
happy path, where a leaked incidental spelling (`0.5f`, `sqrtf`) is easiest to miss:
an emitter for a target with a different literal grammar and a different intrinsic
surface will surface any driver-side C-ism the audit did not already catch. The
normative precondition (§8.2-0002) is phrased as a neutrality criterion, not as a
named language family; this note records only *why* the criterion exists.

---

## 3. Terms & Definitions

- **Emitter** — a party or component that, given the normative input `(OpDef +
  structure_key)`, produces the **ordered pair `{artifact, contract}`** (§6.6-0003):
  a callable kernel, **and** the KISS-Contract that describes it. It renders the
  emitter-must-supply portion of the lowering partition (§6.4) in a target language.
  **Producing the artifact alone does not make a component an emitter.**

  **An emitter is strictly larger than a target-language rendering component.** A
  component that renders §6.4's emitter-must-supply set into a target language
  performs *part* of emitting; it is an emitter only if it also produces the
  contract of §6.6 and honours the typed-decline obligation of §6.8. An
  §6's obligations on "the emitter" therefore do **not** attach to an
  implementation's per-target rendering interface merely because that interface is
  the part which varies by target; they attach to whatever component — or
  composition of components — produces the §6.6-0003 pair.

  This is stated because the narrower reading is the natural one and has been made
  in practice. An implementation mapping its per-target lowering interface onto the
  word "emitter" inherits every §6 obligation the larger object carries but the
  smaller one does not — observed twice, on §6.6-0001/-0003 (the pair) and
  §6.8-0004 (never panic), in the same implementation, for this reason both times.
- **Neutral driver** — the target-independent portion of the lowering machinery that
  spells the driver-may-spell portion of the partition (§6.3) without knowing the
  target language. It is the same code across all targets; **the target-language
  spelling is what differs per target.**

  *"The part that differs per target" is not a synonym for "the emitter."* An
  emitter is strictly larger than that part — see **Emitter** above. Reading the two
  as equivalent is precisely the mapping that drops the contract and typed-decline
  obligations, because those attach to the emitter and not to the rendering
  component.
- **OpDef (op definition)** — a KISS-Ops op definition: an op name, its OpAttrs
  channel, its per-op numeric semantics, and (for a non-primitive op) its reference
  decomposition into strictly-lower-level ops, resolvable to the KISS-Ops primitive
  floor. Owned by KISS-Ops; used here as the *what-to-compute* half of the normative
  input, by name/structure, never re-defined.
- **structure_key / specialization cell** — the KISS-Classify admissibility predicate
  over one layout/dtype/target specialization cell (a coarse op-category tag +
  canonically-ordered operand descriptors + `target_capability` + role hints,
  extent-free, matched byte-for-byte). Owned by KISS-Classify; used here as the
  *which-cell* half of the normative input, carried verbatim, never re-encoded.
- **Normative input** — the pair `(OpDef + structure_key)`: the exact and only input a
  conforming emitter is entitled to (§6.1). Explicitly **not** a schedule-resolved plan.
- **Schedule-resolved plan** — an implementation-private artifact that has already
  fixed scheduling decisions (tiling, fusion order, loop nest, workgroup choice). It
  is **out of scope** as an emitter input; admitting it would drag a scheduler into
  the ABI (§6.1-0002).
- **Lowering decision** — any single choice made while turning `(OpDef +
  structure_key)` into a kernel: a topology walk step, an operand binding, an index-
  arithmetic expression, an operator spelling, a constant spelling, a type spelling, a
  control-flow scaffold, an intrinsic spelling, and so on. Every lowering decision is
  partitioned by §6.2.
- **Target-language surface** — the concrete syntax a lowering decision renders into
  the emitted kernel's target language: an operator token, a constant or special-value
  literal, a type name, or an intrinsic call. A **structural decision** (op-DAG
  topology walk, operand binding, index/address arithmetic, control-flow scaffolding)
  renders **no** target-language surface — it fixes *what* is computed and *in what
  order*, not the concrete tokens — and is therefore neutral by construction. The
  neutrality audit (§6.5) and the audit-default rule (§6.2-0003) apply **only** to
  decisions that HAVE a target-language surface.
- **Storage-capable** — a target is *storage-capable* for a dtype when it can hold
  values of that dtype in memory and the emitter can render a target-language
  spelling for the **carrier** (§6.4-0004). This is a fact about **surface and
  representation**.
- **Compute-capable** — a target is *compute-capable* for a dtype when it can perform
  the op's arithmetic **in that dtype**. This is a fact about **the target's
  arithmetic**, and it is **strictly stronger** than storage-capable: a target may be
  storage-capable and not compute-capable for the same dtype, and that combination is
  ordinary rather than exotic. Two registered namespaces exhibit it — a `vulkan:`
  device may advertise `storageBuffer16BitAccess` (a **storage** capability) and
  perform the arithmetic in `f32`; a `cuda:` device holds fp8 in memory on any
  architecture while the arithmetic is architecture-gated. **The pair is a property of
  dtypes, not of one vendor's spelling.**
- **Structural decision** — a lowering decision that renders no target-language
  surface (§3, *Target-language surface*): topology, operand binding, index/address
  arithmetic, and control-flow scaffolding. Audit-exempt (§6.2-0005, §6.3-0006).
- **Lowering partition** — the total, disjoint partition of every lowering decision
  into **driver-may-spell** (§6.3) and **emitter-must-supply** (§6.4), with no third
  bucket (the closure rule §6.2-0004).
- **driver-may-spell** — the partition set of lowering decisions a neutral driver MAY
  spell because they are either structural (render no target-language surface) or
  spellings proven target-independent by the audit (§6.3).
- **emitter-must-supply** — the partition set of lowering decisions only the emitter
  may render because they have a per-target surface (§6.4); constant spelling and
  special-float-value spelling are members by construction, and by the closure rule
  (§6.2-0004) so is any decision not explicitly enumerated on the driver side.
- **const_lit / constant spelling** — the target-language surface form of a constant
  value. The *value* is pinned by its bit pattern per dtype (KISS-OPS §6.2-0008); the
  *spelling* is emitter-supplied by construction (§6.4-0001).
- **Special float value** — the non-default float values whose target-language
  spelling is emitter-supplied (§6.4-0002): the **non-finite** values `+inf`, `-inf`,
  quiet NaN, and signaling NaN, together with the **finite** edge values `+0`, `-0`,
  and subnormals. Each is pinned by bit pattern per dtype. `±0` and subnormals are
  *finite*; the term groups them with the non-finite values because, like them, they
  carry no portable decimal spelling.
- **Declared-ULP atom** — a KISS-Ops transcendental / special-function atom (`exp`,
  `log`, `sin`, `cos`, `sqrt`, `erf`, …) implemented per-target to a **declared
  per-target accuracy tier**, which is the sole accuracy gate (KISS-OPS §6.8-0001); its
  spelling is emitter-supplied
  (§6.4-0003), never a mandated polynomial on the driver side.
- **Neutrality audit** — the blocking pre-freeze audit (§6.5) that proves, per target,
  the universality of every **surface-bearing** operator placed on the driver side,
  hunts every `const_lit` sibling, and moves any unproven operator to the emitter
  side. Structural decisions are outside its scope (§6.2-0005). Its completion and
  recorded manifest are a freeze precondition (§8).
- **Artifact** — the built, callable kernel binary/object; the first element of the
  emit result `{artifact, contract}` (§6.6-0003). Owned by KISS-Synth/Provision as its
  wire home; KISS-Emit's job is to *produce* it, and its ABI is described by the
  accompanying contract's Interface + Dispatch.
- **Fully generates** — an emitter *fully generates* a kernel when **every** node of
  the emitted Semantics DAG is derived from the `OpDef` with **no opaque residue** —
  no wrapped target intrinsic, pre-existing region, or hand-supplied body the emitter
  did not derive from the `OpDef`. A kernel that incorporates an underived region is
  **not** fully generated (§6.6-0004, §6.6-0007).
- **determinism / fidelity enum** — the single canonical KISS-Ops enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` (KISS-OPS §6.0-0001), imported
  verbatim (§6.0-0003). Selects which round-trip tier is claimable per op (§6.7).
- **MathPrecision attribute** — the KISS-Ops compute-fidelity enum `{bit-stable,
  reduced-mantissa-permitted}` (KISS-OPS §6.17), imported verbatim (§6.0-0002),
  orthogonal to the determinism class and **not** a dtype (§6.0-0004); declared in the
  emitted kernel's contract Guarantees.
- **Structural / op-DAG equality** — the tier-1 round-trip comparator (§6.7-0001,
  pinned by §6.7-0007): two KISS-Ops op DAGs are **equal** iff, after (i) resolving
  every non-primitive node through its KISS-Ops reference decomposition to the
  KISS-Ops **primitive floor**, (ii) placing nodes and edges in KISS-Ops **canonical
  order**, and (iii) **normalizing** the operands of commutative/associative ops per
  the KISS-Ops canonicalization, their node sets, edge sets, and per-node **OpAttrs
  byte channels** are identical. It is a *structural* comparator: it compares neutral
  op-DAG structure, not emitted bytes. Owned as a comparator by KISS-Conform; pinned
  here by reference to KISS-Ops ordering/canonicalization so the two inverse directions
  compare identically.
- **Emit/consume round-trip** — the two-tier fidelity statement of §6.7, shared with
  KISS-Consume: **tier 1** structural op-DAG equality over a declared subset; **tier
  2** numeric bit-identity, same-language on-device only, for the exact-byte class only.
- **Declared subset** — the subset of KISS-Ops ops a party declares it round-trips
  structurally (tier 1). It is not the whole op set; how the subset is advertised, and
  whether an emitter's and a consumer's subsets must be equal or merely overlap for a
  *joint* claim, is an open question (§8) not yet pinned.
- **Language-identity token** — a neutral descriptor of the target language an emitter
  emits into, recorded in the emitted kernel's contract Provenance, used to determine
  "same language" for tier 2 (§6.7-0002, §6.7-0006). KISS-Emit defines no target
  language (§6.1-0005); the token *names* which one an artifact was emitted into
  without blessing it.
- **Tier 1 (structural round-trip)** — the always-claimable, language-independent tier:
  emit-then-lift (or lift-then-emit) reproduces the same KISS-Ops op DAG under op-DAG
  equality over the declared subset.
- **Tier 2 (numeric round-trip)** — the strict, narrowly-scoped add-on: bit-identity of
  the computed result, claimed **only** same-source-language, same-target-device, and
  **only** for the exact-byte determinism class.
- **Typed decline** — a structured refusal returned in lieu of a result (a
  distinguished error value/enumerant or out-of-band error return); never a panic,
  abort, crash, hang, or out-of-bounds read. KISS-Emit's dtype/op decline (§6.8) is a
  typed decline.
- **Contract** — the KISS-Contract seven-section, self-delimiting document {Identity,
  Semantics, Interface, Dispatch, Capabilities, Guarantees, Provenance} that describes
  the emitter's output. Owned by KISS-Contract; carried here by name/structure.
- **KISS-Consume** — the inverse sub-standard (the recognition/lift direction).
  KISS-Emit shares the §6.7 round-trip statement with it (semantically identical via
  the §6.7-0008 correspondence table) but does **not** depend on it; the two are DAG
  siblings.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point semantics; referenced transitively through
  KISS-Ops (KISS-Emit defines no numeric behavior of its own — it partitions who
  spells a value, and pins the value as bits by reference to KISS-Ops).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword
  convention, the normative/informative split, the clause-ID scheme and 1:1 test
  mapping, value pinning as bits/IEEE-754 in wire order, the ban on unquantified
  adjectives, the two version axes, the ≥2-dissimilar-implementations-plus-foreign-
  reader freeze gate, the capability/profile/extension model, governance, licensing,
  and patent posture. **Stated once in the umbrella; referenced here; never restated.**
  This sub-standard's §5 points at umbrella §3 for conventions.
- **KISS-Ops** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: the normative lowering source is a KISS-Ops **OpDef** (op name, OpAttrs
  channel, per-op semantics, reference decomposition), resolvable to the KISS-Ops
  **primitive floor** (the termination guarantee); the emitted kernel's declared-ULP
  transcendental atoms are gated by their own **declared per-target accuracy tier**, which
  KISS-Ops makes the sole accuracy gate (KISS-OPS §6.8-0001). The KISS-Ops §6.8 table is an
  informative advisory floor, not a cap on that tier.
  The constant and
  special-float-value bit pinning is KISS-OPS §6.2-0008; the KISS-Ops **canonical
  op-DAG ordering and commutativity/associativity canonicalization** used by the
  tier-1 comparator (§6.7-0007) are owned by KISS-Ops; and the single canonical
  **determinism/fidelity enum** `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}` (KISS-OPS §6.0-0001) and the **MathPrecision**
  attribute `{bit-stable, reduced-mantissa-permitted}` (KISS-OPS §6.17) are imported
  **verbatim**. KISS-Emit re-defines none of them and defines no op meaning.
- **KISS-Classify** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: the second half of the normative input is a KISS-Classify `structure_key`
  specialization-cell identity, carried verbatim; the emitted kernel's operand pointers
  follow the KISS-Classify **canonical operand order**, carry KISS-Classify dtype
  tokens, and its `target_capability` is a Classify namespaced descriptor (matched
  byte-exact, and the determinant of "same target device" for tier 2, §6.7-0006). Used
  here by name/structure; re-defined nowhere.
- **KISS-Contract** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: the emitter's output is **described by** a KISS-Contract; the emitted
  ABI is pinned by the contract's Interface (§6.5 of KISS-Contract) + Dispatch (§6.6
  of KISS-Contract), the emitted kernel's fidelity is declared in the contract's
  Guarantees (§6.8 of KISS-Contract), the emitter's language-identity token is recorded
  in the contract's Provenance (§6.7-0006), and a fully-generated kernel's Semantics
  field is machine-checkable IR (`semantics_kind = machine-checkable-IR`), while a
  kernel that incorporates an underived region uses the KISS-Contract
  `declared-op-tag` + `lift_residue` path (§6.6-0007). KISS-Emit carries these by
  name/structure and authors no contract field meaning.
- **KISS-Consume** (by version) — the **inverse** sub-standard (recognition/lift
  direction) and a DAG **sibling**: KISS-Emit shares the two-tier emit/consume
  round-trip of §6.7 with KISS-Consume — stated semantically identically, via the
  enumerated clause-correspondence table of §6.7-0008, so the two directions cannot
  drift — but **KISS-Emit does NOT depend on KISS-Consume** (there is no dependency
  edge in either direction; both depend only on KISS-Ops + KISS-Classify +
  KISS-Contract). The shared round-trip is the join that keeps the inverse directions
  honest without an edge between them.
- **KISS-Synth/Provision** (by version) — **not an upstream dependency of KISS-Emit.**
  A just-in-time builder is a KISS-Emit emitter *reached through* KISS-Synth provision
  (the build-on-miss branch returns `{artifact, contract}`); the artifact's wire home
  and the never-panic obligation on the provision path are owned by KISS-Synth. This
  reachability creates **no** edge into KISS-Emit; KISS-Emit's own dtype/op decline
  (§6.8) is the typed decline KISS-Synth surfaces as `CANNOT_PROVISION`.
- **KISS-Conform** (by version) — depends on and tests KISS-Emit; owns the oracle-
  differential harness that resolves the emitted kernel's Semantics DAG to the
  primitive floor and compares under the op's declared determinism class, the IR-DAG
  fuzzer that emits to every backend, the round-trip harness that runs tier 1
  (structural) and — same-language on-device, exact-byte only — tier 2 (numeric), the
  **op-DAG-equality comparator** of §6.7-0007, and the **cross-standard lint** that
  checks the §6.7-0008 emit/consume clause-correspondence.

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase
keywords `MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance
and consumer-behavior guidance and never state a structural or wire requirement. Every
atomic requirement carries a stable, append-only ID `KISS-EMIT-<section>-<nnnn>`,
allocated by the editor of record, never reused after retirement, and mapped 1:1 to ≥1
named KISS-Conform test; each clause states **exactly one** MUST / MUST NOT / SHALL,
and a compound requirement is split into atomic clauses (umbrella §3.3). Values are
pinned as bits and IEEE-754 semantics spelled exactly as the upstream foundational
vocabularies pin them, never as one source language's surface spelling — indeed, that a
per-language surface spelling is *not* normative is this sub-standard's central rule
(§6.4). Unquantified adjectives ("well-formed", "reasonable", "neutral", "universal",
"valid") are banned as the load-bearing requirement; where "universal across targets"
is used it is pinned to the observable neutrality-audit obligation of §6.5, not left as
an adjective. Every clause declares its determinism/fidelity class so KISS-Conform
selects the correct comparator. See umbrella §3 for the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class and imported enums

- **KISS-EMIT-6.0-0001** — Every structural obligation in §6–§9 is determinism-class
  **exact byte compare**; KISS-Conform MUST evaluate each such clause with a byte-exact
  comparator and MUST NOT apply tolerance or order-invariant comparison. *Test:*
  `test_emit_determinism_class_exact_byte`.
- **KISS-EMIT-6.0-0002** — The MathPrecision compute-fidelity attribute `{bit-stable,
  reduced-mantissa-permitted}` an emitted kernel declares MUST be imported **verbatim**
  from KISS-Ops (KISS-OPS §6.17); KISS-Emit MUST NOT re-spell or fork it. *Test:*
  `test_emit_mathprecision_imported_verbatim`.
- **KISS-EMIT-6.0-0003** — The single canonical determinism/fidelity enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` that governs the *numeric* result
  of an emitted kernel is **owned by KISS-Ops** (KISS-OPS §6.0-0001) and MUST be
  imported by KISS-Emit **verbatim**; KISS-Emit MUST NOT define, re-spell, or fork this
  enum. *Test:* `test_emit_determinism_enum_imported_verbatim`.
- **KISS-EMIT-6.0-0004** — KISS-Emit MUST treat the MathPrecision attribute as
  **orthogonal** to the determinism class and MUST NOT treat it as a dtype. *Test:*
  `test_emit_mathprecision_orthogonal_not_dtype`.

### 6.1 The normative lowering input — `(OpDef + structure_key)`

- **KISS-EMIT-6.1-0001** — The normative input to a conforming emitter MUST be exactly
  the pair `(OpDef, structure_key)`: a KISS-Ops op definition **paired with** a
  KISS-Classify specialization-cell identity. *Test:*
  `test_emit_normative_input_is_opdef_plus_key`.
- **KISS-EMIT-6.1-0002** — An emitter MUST NOT take a **schedule-resolved plan** (an
  input that has already fixed tiling, fusion order, loop nest, or workgroup choice) as
  its normative input. *Test:* `test_emit_rejects_schedule_plan_as_abi`.
- **KISS-EMIT-6.1-0003** — The `OpDef` half of the input MUST be a KISS-Ops op
  definition whose computation is resolvable, through its KISS-Ops reference
  decomposition, to the KISS-Ops **primitive floor** (the termination guarantee).
  *Test:* `test_emit_opdef_resolves_to_floor`.
- **KISS-EMIT-6.1-0004** — The `structure_key` half of the input MUST be a KISS-Classify
  `structure_key` token carried **verbatim**; an emitter MUST NOT re-encode, truncate,
  or reinterpret its bytes. *Test:* `test_emit_structure_key_carried_verbatim`.
- **KISS-EMIT-6.1-0005** — KISS-Emit MUST NOT define, mandate, or bless a source
  language or source grammar; the target language a kernel is emitted into is out of
  scope, and no clause of this sub-standard MUST be read as pinning a target syntax.
  *Test:* `test_emit_defines_no_source_language`.
- **KISS-EMIT-6.1-0006** — An implementation MUST NOT require, as a condition of
  conformance, any input beyond the pair `(OpDef, structure_key)` of §6.1-0001. *Test:*
  `test_emit_no_input_beyond_pair`.
- **KISS-EMIT-6.1-0007** — An implementation MUST NOT treat a kernel produced from any
  input other than the pair `(OpDef, structure_key)` as a KISS-Emit conforming emission.
  *Test:* `test_emit_other_input_not_conforming`.
- **KISS-EMIT-6.1-0008** — An emitter MUST NOT require a scheduler in the conformance
  path; scheduling MUST NOT be part of the KISS-Emit ABI or appear in the normative
  input schema of §6.1-0001. (An emitter MAY make scheduling choices internally, but
  those choices are not a required input.) *Test:* `test_emit_no_scheduler_in_abi`.
- **KISS-EMIT-6.1-0009** — An emitter MUST NOT accept as an `OpDef` a token outside the
  KISS-Ops op set of the declared KISS-Ops version, and MUST return a typed decline
  (§6.8) for an op it cannot resolve to the primitive floor. *Test:*
  `test_emit_opdef_outside_opset_declines`.
- **KISS-EMIT-6.1-0010** — The emitted kernel's contract Identity `accept_predicate`
  MUST be the input `structure_key` byte-for-byte. *Test:*
  `test_emit_accept_predicate_is_structure_key`.

### 6.2 The complete lowering partition — total and disjoint

- **KISS-EMIT-6.2-0001** — The lowering partition MUST be **disjoint**: no lowering
  decision falls in both the **driver-may-spell** set (§6.3) and the
  **emitter-must-supply** set (§6.4); no decision explicitly enumerated on the driver
  side (§6.3) also appears on the emitter side (§6.4). *Test:*
  `test_emit_partition_disjoint`.
- **KISS-EMIT-6.2-0002** — Constant spelling and special-float-value spelling MUST be
  placed in the **emitter-must-supply** set (§6.4-0001, §6.4-0002) and MUST NOT be
  placed in, or defaulted onto, the driver-may-spell set. *Test:*
  `test_emit_constants_are_emitter_supplied`.
- **KISS-EMIT-6.2-0003** — Any lowering decision that **has a target-language surface**
  (§3) and is **not proven** target-independent by the neutrality audit (§6.5) MUST
  default to the **emitter-must-supply** set; the driver-may-spell set holds a
  surface-bearing spelling only when the audit has proven it. *Test:*
  `test_emit_unproven_decision_defaults_to_emitter`.
- **KISS-EMIT-6.2-0004** — Any lowering decision **not explicitly enumerated** on the
  driver-may-spell side (§6.3) MUST be **emitter-must-supply by default**; the driver
  side is a **closed enumeration** and the emitter side is its open complement, and an
  implementation and this specification MUST NOT introduce a third, "implicit", or
  unowned bucket. This closure rule is what makes the partition **complete**. *Test:*
  `test_emit_partition_complete_by_closure`.
- **KISS-EMIT-6.2-0005** — A **structural decision** — one that renders **no**
  target-language surface (§3): op-DAG topology, operand binding, index/address
  arithmetic, or control-flow scaffolding — is neutral by construction and MUST NOT be
  subject to the audit-default rule of §6.2-0003, because it renders no syntax for the
  audit to prove. *Test:* `test_emit_structural_decisions_audit_exempt`.

### 6.3 The driver-may-spell set

- **KISS-EMIT-6.3-0001** — The op-DAG **structure / topology** — the node-to-node walk
  of the emitted kernel's Semantics DAG — MUST be a driver-may-spell decision: it is a
  structural decision that renders no target-language surface (§6.2-0005), and a
  neutral driver MAY emit it for all targets. *Test:*
  `test_emit_driver_spells_dag_topology`.
- **KISS-EMIT-6.3-0002** — **Operand binding** in KISS-Classify canonical operand
  order, and the positional argument-signature layout the contract Interface pins,
  MUST be a driver-may-spell decision: it is a structural decision that renders no
  target-language surface (§6.2-0005). *Test:*
  `test_emit_driver_spells_operand_binding`.
- **KISS-EMIT-6.3-0003** — **Index / address arithmetic** over the declared signed
  strides and base offsets, and the grid-stride thread→element mapping derived from the
  contract Dispatch model, MUST be a driver-may-spell decision: it is a structural
  decision that renders no target-language surface (§6.2-0005). *Test:*
  `test_emit_driver_spells_index_arithmetic`.
- **KISS-EMIT-6.3-0004** — The strictly-universal infix arithmetic operator spellings
  (`+`, `-`, `*`, `/`) for the ordinary arithmetic atoms — the one driver-side decision
  that DOES render a target-language surface — MAY be a driver-may-spell decision
  **only** after the neutrality audit (§6.5) has **proven** that operator identical
  across every target the emitter claims; absent that proof, the operator MUST be
  treated as emitter-must-supply (§6.2-0003). This clause MUST NOT be read as asserting
  universality — it conditions the placement on the §6.5 proof. *Test:*
  `test_emit_infix_operators_gated_on_audit`.
- **KISS-EMIT-6.3-0005** — **Control-flow / loop scaffolding** derived mechanically from
  the Dispatch launch model (the count-unit → grid derivation and workgroup sizing) MUST
  be a driver-may-spell decision: it is a structural decision that renders no
  target-language surface (§6.2-0005). *Test:*
  `test_emit_driver_spells_control_flow`.
- **KISS-EMIT-6.3-0006** — Of the driver-may-spell decisions of §6.3, the structural
  decisions §6.3-0001, §6.3-0002, §6.3-0003, and §6.3-0005 render no target-language
  surface and MUST be treated as **audit-exempt** (§6.2-0005); only the infix operator
  spellings of §6.3-0004, which do render a target-language surface, are audit-gated
  (§6.5). An implementation MUST NOT treat a structural driver-side decision as
  audit-gated, and MUST NOT treat the infix operators as audit-exempt. *Test:*
  `test_emit_structural_driver_decisions_audit_exempt`.

### 6.4 The emitter-must-supply set

- **KISS-EMIT-6.4-0001** — **Constant spelling** MUST be an emitter-must-supply
  decision: a constant MUST be pinned by its bit pattern per dtype (as KISS-OPS
  §6.2-0008 pins a `const(bits)` leaf), and the emitter MUST render the target-language
  surface for that bit pattern; the neutral driver MUST NOT spell a constant. A decimal
  literal spelling is a per-language incidental and MUST NOT be a normative or
  driver-side decision. *Test:* `test_emit_constant_spelling_emitter_supplied`.
- **KISS-EMIT-6.4-0002** — **Special-float-value spelling** — the non-finite values
  `+inf`, `-inf`, quiet NaN, and signaling NaN, together with the finite edge values
  `+0`, `-0`, and subnormals — MUST be an emitter-must-supply decision: each value MUST
  be pinned by its bit pattern per dtype and MUST round-trip exactly, and the emitter
  MUST render its target-language surface; the neutral driver MUST NOT spell a special
  float value, and an implementation MUST NOT assume a portable decimal spelling exists
  for one. (`±0` and subnormals are *finite*; they are grouped here because, like the
  non-finite values, they have no portable decimal spelling.) *Test:*
  `test_emit_special_float_value_spelling_emitter_supplied`.
- **KISS-EMIT-6.4-0003** — **Transcendental / declared-ULP atom and hardware-intrinsic
  spelling** (`exp`, `log`, `sin`, `cos`, `sqrt`, `erf`, and the other KISS-Ops
  transcendental / special-function atoms) MUST be an emitter-must-supply decision, and the
  neutral driver MUST NOT spell it as a mandated polynomial or a fixed language intrinsic.
  Each is a per-target atom implemented to a **declared per-target accuracy tier**, which
  KISS-Ops makes the sole accuracy gate (KISS-OPS §6.8-0001). The KISS-Ops §6.8 table is an
  informative advisory floor, not a cap: a truthful tier looser than a table value MUST NOT
  be rejected. *Test:*
  `test_emit_transcendental_spelling_emitter_supplied`.
- **KISS-EMIT-6.4-0004** — **Target-specific type spelling**, and the surface form of
  any dtype that differs by target language (`f16`, `bf16`, the FP8 formats `f8e4m3fn` /
  `f8e5m2`, and the components of the complex dtypes `c64` / `c128`), MUST be an emitter-
  must-supply decision; the neutral driver MUST NOT spell a dtype's target-language
  surface. *Test:* `test_emit_type_spelling_emitter_supplied`.
- **KISS-EMIT-6.4-0005** — Any operator or construct the neutrality audit (§6.5) did
  **not** prove universal across the emitter's claimed targets MUST be an emitter-must-
  supply decision; an implementation MUST NOT retain such an operator on the driver-may-
  spell side. *Test:* `test_emit_unproven_operator_is_emitter_supplied`.
- **KISS-EMIT-6.4-0006** — **Spellable is not computable.** That a dtype's
  target-language surface is emitter-must-supply (§6.4-0004) says the target is
  **storage-capable** (§3) for it and says **nothing** about whether the target is
  **compute-capable** (§3). An implementation **MUST NOT** infer computability from
  spellability, and **MUST NOT** treat the existence of a carrier spelling as a
  licence to emit that dtype's arithmetic. Where the two differ the emitter MUST
  return the typed decline of §6.8-0002 rather than emit arithmetic the target cannot
  perform. *Test:* `test_emit_spellable_is_not_computable`.

### 6.5 The pre-freeze neutrality audit (blocking freeze-gate governance)

> The clauses of this section are **freeze-gate governance preconditions**, verified by
> the KISS-Conform AUDIT role (§8.2) against the audit's **recorded manifest**, not
> per-run behavioral obligations on an arbitrary emitter. Their mapped tests assert on
> the recorded audit artifact/manifest.

- **KISS-EMIT-6.5-0001** — Before KISS-Emit advances Draft → Frozen (§8), a
  **neutrality audit** MUST be performed and **recorded** that, for **every** driver-
  side decision that has a **target-language surface** (§3) — at this schema version,
  the infix operator spellings of §6.3-0004 — **proves** that surface identical across
  every target the emitter claims; the recorded audit manifest MUST be a freeze
  precondition, and a freeze attempted without a completed recorded audit MUST fail the
  gate. Structural decisions (§6.2-0005) are outside the audit's scope. *Test:*
  `test_emit_neutrality_audit_manifest_is_freeze_precondition`.
- **KISS-EMIT-6.5-0002** — The recorded neutrality audit MUST show every surface-bearing
  driver-side operator or construct it could not prove target-independent moved to the
  emitter-must-supply side (§6.4-0005); the driver/emitter boundary for the §6.3-0004
  infix operators is **provisional** until the audit records them as proven. *Test:*
  `test_emit_audit_moves_unproven_spelling`.
- **KISS-EMIT-6.5-0003** — The recorded neutrality audit MUST show every constant-
  spelling (`const_lit`) sibling on the driver-may-spell side found and reclassified as
  emitter-must-supply (§6.4-0001); a constant or special-float-value spelling recorded
  on the driver side after the audit is a conformance failure. *Test:*
  `test_emit_audit_hunts_const_lit_siblings`.

### 6.6 The emitted kernel is described by a contract

- **KISS-EMIT-6.6-0001** — Every kernel a conforming emitter produces MUST be described
  by a KISS-Contract carrying the universal required core of seven sections
  (KISS-CONTRACT §6.2-0001); an emitter MUST NOT return a callable kernel without an
  accompanying contract. *Test:* `test_emit_output_has_contract`.
- **KISS-EMIT-6.6-0002** — The emitted kernel's **fidelity** MUST be declared in its
  contract **Guarantees** section: the determinism class (the imported KISS-Ops enum,
  §6.0-0003) and the MathPrecision attribute (§6.0-0002) MUST both be present there.
  *Test:* `test_emit_fidelity_declared_in_guarantees`.
- **KISS-EMIT-6.6-0003** — The emitter's output MUST be the ordered pair
  `{artifact, contract}` (the artifact first, the contract second), and this output
  shape MUST hold **independently of whether the emitter is reached through KISS-Synth
  provision**: the same ordered pair is the emit result on the standalone path and on
  the build-on-miss provision path. *Test:*
  `test_emit_output_is_artifact_contract_pair`.
- **KISS-EMIT-6.6-0004** — For a kernel the emitter **fully generates** (§3: every node
  of the emitted Semantics DAG is derived from the `OpDef` with no opaque residue), the
  contract's Semantics field MUST be machine-checkable IR (`semantics_kind =
  machine-checkable-IR`, KISS-CONTRACT §6.2-0004); an emitter MUST NOT emit a fully-
  generated kernel with a degraded `declared-op-tag` Semantics, and MUST NOT fabricate
  a machine-checkable IR Semantics it did not derive from the `OpDef`. *Test:*
  `test_emit_generated_semantics_is_machine_checkable`.
- **KISS-EMIT-6.6-0005** — An emitter MUST NOT declare the emitted kernel's fidelity
  (determinism class or MathPrecision) anywhere the KISS-Contract schema does not home
  it; the Guarantees section of §6.6-0002 is the only home. *Test:*
  `test_emit_fidelity_not_declared_off_schema`.
- **KISS-EMIT-6.6-0006** — The artifact's ABI MUST be the ABI the accompanying
  contract's Interface + Dispatch pin; an emitter MUST NOT produce an artifact whose ABI
  diverges from the contract that describes it. *Test:*
  `test_emit_artifact_abi_matches_contract`.
- **KISS-EMIT-6.6-0007** — An emitter that incorporates into the kernel a region it did
  **not** derive from the `OpDef` (a wrapped target intrinsic or a pre-existing region)
  MUST record that region on the KISS-Contract `declared-op-tag` + `lift_residue` path
  (KISS-CONTRACT) rather than presenting the kernel as fully generated, and MUST NOT
  fabricate machine-checkable IR for the underived region. *Test:*
  `test_emit_underived_region_recorded_as_residue`.

### 6.7 The emit/consume round-trip — two tiers (semantically identical to KISS-Consume)

The clauses §6.7-0001 through §6.7-0005 state the two-tier emit/consume round-trip.
Their intent is **semantically identical** to the corresponding clauses of KISS-Consume
so the two inverse directions cannot drift; §6.7-0008 pins that identity via an
enumerated clause-correspondence table. Where the informative §2.6 illustrates the
cross-language case with named target languages, this normative statement uses only
generic roles per the umbrella §3 keyword and value-pinning discipline (no vendor or
project names in normative text); the illustration is neutralized, the intent is
identical. These five clauses are held as **unified statements** — each corresponds
one-for-one to a KISS-Consume clause — and are, for that cross-standard-correspondence
reason, kept whole rather than atomically split (the split would break the 1:1
correspondence the sibling relies on).

- **KISS-EMIT-6.7-0001** — **Tier 1 (structural round-trip).** Emit-then-lift (or
  lift-then-emit) MUST reproduce the **same KISS-Ops op DAG** under **structural /
  op-DAG equality** (§6.7-0007), checked over a **declared subset** of ops (the subset
  each side declares it round-trips, not the whole op set). Tier 1 is the always-
  claimable tier; a conforming emitter MUST support the tier-1 structural round-trip
  over its declared subset. It is language-independent because it compares KISS-Ops
  op-DAG structure, not bytes. *Test:* `test_emit_roundtrip_tier1_structural`.
- **KISS-EMIT-6.7-0002** — **Tier 2 (numeric round-trip).** Bit-identity of the computed
  result MUST be claimed **only** same-source-language and on-device (same source
  language, same target device — determined per §6.7-0006) **and only** for the
  **exact-byte** determinism class; for an op of the `ULP/tolerance` or
  `order-invariant/nondeterministic` class an implementation MUST NOT claim
  bit-identity. Tier 2 is a strict, narrowly-scoped add-on to tier 1 and MUST NOT be
  treated as a substitute for it.

  This clause restricts a **claim**. It states no verification method and MUST NOT be
  read as sanctioning one: how a `ULP/tolerance` op's result is evaluated is
  KISS-Conform's, under §6.0-0003. A previous revision of this clause additionally
  required such an op to "compare under that op's declared comparator" — a
  **verification mandate inside a claim-restriction clause**, legislating outside this
  sub-standard's subject, and in direct conflict with KISS-Conform once §6.0-0003
  anchors the class to an audited wide-precision referent. It was excised rather than
  reworded: rewording would have left the same clause legislating outside its subject,
  merely agreeing with the new rule for as long as that rule held still.

  **Consequence, stated rather than left to be inferred.** With the mandate gone and
  §6.0-0003 not yet anchored, an **equivalence check** — two artifacts of the *same*
  computation compared against each other, which asks about no third thing and for
  which a wide-precision referent is meaningless rather than merely absent — has **no
  sanctioned form in this suite**. That is a known and accepted cost, not an oversight,
  and it is recorded here because a practice that is unnameable by silence is
  indistinguishable from one nobody considered. Implementations that perform such
  checks today are not thereby non-conforming to this clause; they are performing an
  activity this suite does not yet name.
  *Test:* `test_emit_roundtrip_tier2_numeric_same_language`.
- **KISS-EMIT-6.7-0003** — **No cross-language numeric identity.** An implementation
  MUST NOT claim numeric bit-identity across source languages; a cross-language round-
  trip MUST be tier 1 (structural) only — a transcendental atom lowered into one target
  language is not bit-identical to the same atom lowered into another, and overclaiming
  cross-language numeric identity is a non-conforming overclaim. Across languages the
  guarantee MUST stop at structural op-DAG equality over the declared subset. *Test:*
  `test_emit_no_cross_language_numeric_identity`.
- **KISS-EMIT-6.7-0004** — Which round-trip tier is claimable for a given op MUST be
  decided by the imported KISS-Ops determinism/fidelity enum (§6.0-0003): an
  `exact-byte` op MAY be claimed at tier 2 (same-language on-device); a `ULP/tolerance`
  or `order-invariant/nondeterministic` op MUST NOT be claimed at tier 2 and stops at
  tier 1. An implementation MUST NOT select a tier by any rule other than the op's
  imported determinism class. As in §6.7-0002, this clause decides WHICH CLAIM IS
  ADMISSIBLE and states no verification method; the phrase "plus its declared
  comparator" was excised with the same mandate for the same reason. *Test:*
  `test_emit_roundtrip_tier_selected_by_determinism_class`.
- **KISS-EMIT-6.7-0005** — KISS-Emit and KISS-Consume MUST be treated as **DAG
  siblings** with **no** dependency edge between them: an implementation MUST NOT
  require KISS-Consume to perform a KISS-Emit emission, and MUST NOT make a round-trip
  claim (§6.7-0001–§6.7-0004) a prerequisite of producing the artifact + contract of
  §6.6. The round-trip is a **join** verified by KISS-Conform, not an edge in the
  dependency DAG (umbrella §2.2). *Test:*
  `test_emit_emit_consume_are_siblings_no_edge`.
- **KISS-EMIT-6.7-0006** — For the Emit direction, "same source language" in §6.7-0002
  MUST be determined by the emitter's **language-identity token** (§3) recorded in the
  emitted kernel's contract Provenance, and "same target device" MUST be determined by
  **byte-equal** KISS-Classify `target_capability`. An implementation MUST NOT claim
  tier-2 bit-identity across differing language-identity tokens or differing
  `target_capability`. *Test:* `test_emit_tier2_language_and_device_determinant`.
- **KISS-EMIT-6.7-0007** — **Structural / op-DAG equality** (the tier-1 comparator,
  §6.7-0001) MUST be evaluated as: two KISS-Ops op DAGs are **equal** iff, after (i)
  resolving every non-primitive node through its KISS-Ops reference decomposition to the
  KISS-Ops primitive floor, (ii) placing nodes and edges in KISS-Ops canonical order,
  and (iii) normalizing the operands of commutative/associative ops per the KISS-Ops
  canonicalization, their node sets, edge sets, and per-node **OpAttrs byte channels**
  are identical; the comparison is performed **after** resolution to the primitive floor
  and **includes** the OpAttrs bytes. An implementation MUST NOT claim tier-1 equality
  under any looser predicate. *Test:* `test_emit_op_dag_equality_defined`.
- **KISS-EMIT-6.7-0008** — The round-trip statement of §6.7-0001 through §6.7-0005
  **and §6.7-0009** MUST be **semantically equivalent** — same clause intent, same
  normative effect — to the corresponding round-trip clauses of KISS-Consume under the
  following enumerated **clause-correspondence table**; the only permitted difference is that the
  vendor-language illustration of the informative §2.6 is neutralized to generic roles
  in this normative section. This obligation is verified by a KISS-Conform
  **cross-standard document lint** over both sub-standards' texts, not by an emitter-
  behavior test. *Correspondence:*

  | KISS-Emit | KISS-Consume | Statement |
  |---|---|---|
  | §6.7-0001 | §6.6-0001 | Tier 1 — structural round-trip |
  | §6.7-0002 | §6.6-0002 | Tier 2 — numeric round-trip |
  | §6.7-0003 | §6.6-0003 | No cross-language numeric identity |
  | §6.7-0004 | §6.6-0004 | Tier selected by determinism enum |
  | §6.7-0005 | §6.6-0005 | DAG siblings, no dependency edge |
  | §6.7-0009 | §6.6-0006 | Whole-kernel tier aggregation |

  *Test:* `test_conform_emit_consume_correspondence_lint` (KISS-Conform
  cross-standard lint).

- **KISS-EMIT-6.7-0009** — **Whole-kernel tier aggregation.** A whole-kernel tier-2
  (numeric bit-identity) round-trip claim MUST be admissible **only** when **every** op
  in the resolved DAG — resolved through its KISS-Ops reference decomposition to the
  KISS-Ops primitive floor — is of the **exact-byte** determinism class **and the
  same-language, on-device restriction of §6.7-0002 holds**; if **any** op
  in the resolved DAG is of the `ULP/tolerance` or `order-invariant/nondeterministic`
  class, the whole-kernel round-trip MUST stop at tier 1 (structural) only, and an
  implementation MUST NOT claim whole-kernel tier-2 bit-identity for it. This mirrors
  the identical whole-kernel aggregation rule of the inverse direction (KISS-Consume
  §6.6-0006) so the emit/consume round-trip aggregates symmetrically in both
  directions. *Test:* `test_emit_whole_kernel_tier2_requires_all_exact_byte`.

### 6.8 Typed decline, never a panic

- **KISS-EMIT-6.8-0001** — An emitter that cannot emit for its input — an unsupported
  dtype, an op it cannot resolve to the KISS-Ops primitive floor (§6.1-0003), an
  unsupported target, or a malformed `(OpDef + structure_key)` input — MUST return a
  **typed decline** (a distinguished error value/enumerant or out-of-band error
  return). *Test:* `test_emit_decline_is_typed`.
- **KISS-EMIT-6.8-0002** — On a dtype the emitter does not support, the emitter MUST
  return the typed decline of §6.8-0001 and MUST NOT emit a partial, silently-narrowed,
  or wrong-dtype kernel in its place. *Test:* `test_emit_unsupported_dtype_declines`.
- **KISS-EMIT-6.8-0003** — When an emitter is reached through KISS-Synth provision on a
  build-on-miss and cannot build, its typed decline (§6.8-0001) MUST be the failure
  KISS-Synth surfaces as a `CANNOT_PROVISION` provision decline. *Test:*
  `test_emit_build_on_miss_declines_cleanly`.
- **KISS-EMIT-6.8-0004** — On **any** input, including malformed, empty, truncated, or
  adversarial input, an emitter MUST NOT panic, abort, crash, hang, or read outside its
  input buffers; it MUST instead return the typed decline of §6.8-0001. *Test:*
  `test_emit_never_panic_on_adversarial_input`.
- **KISS-EMIT-6.8-0005** — On a build-on-miss the emitter MUST NOT satisfy the request
  with a null artifact, and MUST NOT return an artifact without the accompanying
  contract of §6.6-0001. *Test:* `test_emit_build_on_miss_no_null_artifact`.

---

## 7. Capability, Profile & Extension model

The KISS-Emit mandatory core, negotiable options, and reserved ranges follow the
umbrella capability/profile/extension model (umbrella §6), which is stated once there
and not restated here. This section pins only what is specific to KISS-Emit.

- **KISS-EMIT-7.1-0001** — The KISS-Emit **mandatory core** — the clauses every
  conforming emitter MUST satisfy regardless of which options it claims — MUST be: the
  normative-input schema (§6.1), the complete-and-disjoint partition (§6.2), the
  partition membership (§6.3 / §6.4), the emitted-contract obligations (§6.6), the
  tier-1 structural **self-round-trip** over the emitter's declared subset (§6.7-0001,
  the single-party emit-then-lift-then-emit round-trip that a party can evaluate on its
  own output), and the typed-decline-never-panic obligation (§6.8). A **joint**
  round-trip claim between two independent parties is **not** part of the mandatory
  core (see §7.2). An emitter that cannot satisfy the mandatory core does not conform to
  KISS-Emit at all. *Test:* `test_emit_mandatory_core`.
- **KISS-EMIT-7.2-0001** — An emitter MUST advertise the **declared subset** of ops it
  round-trips structurally at tier 1 (§6.7-0001), and a tier-1 round-trip claim MUST be
  evaluated only over the advertised declared subset; an emitter MUST NOT claim a tier-1
  round-trip over an op outside its advertised declared subset. The wire encoding of the
  declared-subset advertisement, and whether an emitter's and a consumer's declared
  subsets must be **equal** or merely **overlap** for a **joint** round-trip claim, are
  an open question (§8, Appendix A.5) **not yet pinned** and MUST be resolved before
  freeze (umbrella §5.3); this clause pins only that a claim is bounded by the
  advertised subset. *Test:* `test_emit_declared_subset_bounds_claim`.
- **KISS-EMIT-7.3-0001** — Tier-2 numeric round-trip (§6.7-0002) MUST be a **negotiable
  option**, not part of the mandatory core, and MUST be claimable by an emitter only for
  the same-language, on-device, exact-byte case; an emitter that does not claim the
  tier-2 option MUST still conform via the mandatory core (which requires only tier 1).
  *Test:* `test_emit_tier2_is_optional`.

---

## 8. Versioning & Lifecycle

KISS-Emit carries the two independent version axes of the umbrella (umbrella §5.1): the
**wire/ABI schema version** (`EMIT_ABI_VERSION`, the axis KISS-Conform keys conformance
on) and the **published-crate semver** of the reference crate(s). A §8 bump-versus-no-
bump rule table (below) states, per kind of change, which axis moves.

| Change | Axis that moves |
|---|---|
| The normative-input schema `(OpDef + structure_key)` changes | `EMIT_ABI_VERSION` (and crate semver) |
| A lowering decision moves between the two partition sets (e.g. an operator moves driver→emitter after the neutrality audit) | `EMIT_ABI_VERSION` (and crate semver) |
| The declared-subset advertisement mechanism changes | `EMIT_ABI_VERSION` (and crate semver) |
| The round-trip tier wording changes (must be mirrored in KISS-Consume per the §6.7-0008 correspondence and the §10 governance pen-drift guard) | `EMIT_ABI_VERSION` (and crate semver), coordinated cross-party |
| A pure code refactor that preserves the partition, input schema, and round-trip | crate semver only |
| A new emitter backend that changes no partition boundary | crate semver only |

- **KISS-EMIT-8.1-0001** — Any change that alters the normative-input schema (§6.1),
  moves a lowering decision between the two partition sets (§6.3 / §6.4), changes the
  declared-subset advertisement (§7.2), or alters the round-trip tier wording (§6.7)
  MUST bump `EMIT_ABI_VERSION`. *Test:* `test_emit_abi_version_bump_rule`.
- **KISS-EMIT-8.1-0002** — A change that preserves the normative-input schema, the
  partition boundary, the declared-subset advertisement, and the round-trip tier wording
  MUST NOT bump `EMIT_ABI_VERSION` and MUST bump only the crate semver. *Test:*
  `test_emit_no_abi_bump_on_compatible_change`.
- **KISS-EMIT-8.2-0001** — KISS-Emit MUST NOT advance Draft → Frozen unless, in addition
  to the umbrella §5.3 freeze gate, the blocking neutrality audit of §6.5 is completed
  and its manifest recorded. *Test:* `test_emit_freeze_precondition_audit`.
- **KISS-EMIT-8.2-0002** — KISS-Emit MUST NOT advance Draft → Frozen unless an emitter
  for a target whose **constant and operator surface spellings differ** from the
  reference emitter's — as established by the §6.5 audit — has been demonstrated (so the
  partition is exercised across dissimilar surface grammars). *Test:*
  `test_emit_freeze_precondition_differing_spelling_emitter`.
- **KISS-EMIT-8.2-0003** — KISS-Emit MUST NOT advance Draft → Frozen unless a **second
  consumer** has certified the round-trip of §6.7 against an emitter's output. *Test:*
  `test_emit_freeze_precondition_second_consumer_roundtrip`.
- **KISS-EMIT-8.2-0004** — The Draft → Frozen transition MUST be signed by the
  KISS-Conform AUDIT role, not the authoring editor alone. *Test:*
  `test_emit_freeze_signed_by_audit_role`.

---

## 9. Conformance

- **KISS-EMIT-9.1-0001** — A KISS-Emit conformance claim MUST be **prerequisite-closed**
  over KISS-Emit's incoming **STRUCTURAL** edges: a claim to conform to KISS-Emit MUST
  include a claim to conform to KISS-Ops, KISS-Classify, and KISS-Contract (each by
  version), per the umbrella §2.2 edge table and §6.3 prerequisite-closure rule. An
  implementation MUST NOT claim KISS-Emit conformance without the closed prerequisite
  claims. *Test:* `test_emit_claim_prerequisite_closed`.
- **KISS-EMIT-9.2-0001** — An implementation **conforms** to KISS-Emit at a given
  `EMIT_ABI_VERSION` if and only if it passes the unmodified KISS-Conform suite for
  KISS-Emit at that version, with each normative clause of §6–§8 mapped to at least one
  passing named test and each test citing the clause ID(s) it enforces (bidirectional
  traceability); the KISS-Conform build MUST fail on any normative MUST without a mapped
  test. *Test:* `test_emit_traceability_complete`.

**Clause-to-test traceability matrix (stub).** Each KISS-Emit clause maps 1:1 to at
least one named KISS-Conform test; the matrix below is the authoritative stub kept in
sync by the KISS-Conform lint.

| Clause | Test |
|---|---|
| KISS-EMIT-6.0-0001 | `test_emit_determinism_class_exact_byte` |
| KISS-EMIT-6.0-0002 | `test_emit_mathprecision_imported_verbatim` |
| KISS-EMIT-6.0-0003 | `test_emit_determinism_enum_imported_verbatim` |
| KISS-EMIT-6.0-0004 | `test_emit_mathprecision_orthogonal_not_dtype` |
| KISS-EMIT-6.1-0001 | `test_emit_normative_input_is_opdef_plus_key` |
| KISS-EMIT-6.1-0002 | `test_emit_rejects_schedule_plan_as_abi` |
| KISS-EMIT-6.1-0003 | `test_emit_opdef_resolves_to_floor` |
| KISS-EMIT-6.1-0004 | `test_emit_structure_key_carried_verbatim` |
| KISS-EMIT-6.1-0005 | `test_emit_defines_no_source_language` |
| KISS-EMIT-6.1-0006 | `test_emit_no_input_beyond_pair` |
| KISS-EMIT-6.1-0007 | `test_emit_other_input_not_conforming` |
| KISS-EMIT-6.1-0008 | `test_emit_no_scheduler_in_abi` |
| KISS-EMIT-6.1-0009 | `test_emit_opdef_outside_opset_declines` |
| KISS-EMIT-6.1-0010 | `test_emit_accept_predicate_is_structure_key` |
| KISS-EMIT-6.2-0001 | `test_emit_partition_disjoint` |
| KISS-EMIT-6.2-0002 | `test_emit_constants_are_emitter_supplied` |
| KISS-EMIT-6.2-0003 | `test_emit_unproven_decision_defaults_to_emitter` |
| KISS-EMIT-6.2-0004 | `test_emit_partition_complete_by_closure` |
| KISS-EMIT-6.2-0005 | `test_emit_structural_decisions_audit_exempt` |
| KISS-EMIT-6.3-0001 | `test_emit_driver_spells_dag_topology` |
| KISS-EMIT-6.3-0002 | `test_emit_driver_spells_operand_binding` |
| KISS-EMIT-6.3-0003 | `test_emit_driver_spells_index_arithmetic` |
| KISS-EMIT-6.3-0004 | `test_emit_infix_operators_gated_on_audit` |
| KISS-EMIT-6.3-0005 | `test_emit_driver_spells_control_flow` |
| KISS-EMIT-6.3-0006 | `test_emit_structural_driver_decisions_audit_exempt` |
| KISS-EMIT-6.4-0001 | `test_emit_constant_spelling_emitter_supplied` |
| KISS-EMIT-6.4-0002 | `test_emit_special_float_value_spelling_emitter_supplied` |
| KISS-EMIT-6.4-0003 | `test_emit_transcendental_spelling_emitter_supplied` |
| KISS-EMIT-6.4-0004 | `test_emit_type_spelling_emitter_supplied` |
| KISS-EMIT-6.4-0005 | `test_emit_unproven_operator_is_emitter_supplied` |
| KISS-EMIT-6.4-0006 | `test_emit_spellable_is_not_computable` |
| KISS-EMIT-6.5-0001 | `test_emit_neutrality_audit_manifest_is_freeze_precondition` |
| KISS-EMIT-6.5-0002 | `test_emit_audit_moves_unproven_spelling` |
| KISS-EMIT-6.5-0003 | `test_emit_audit_hunts_const_lit_siblings` |
| KISS-EMIT-6.6-0001 | `test_emit_output_has_contract` |
| KISS-EMIT-6.6-0002 | `test_emit_fidelity_declared_in_guarantees` |
| KISS-EMIT-6.6-0003 | `test_emit_output_is_artifact_contract_pair` |
| KISS-EMIT-6.6-0004 | `test_emit_generated_semantics_is_machine_checkable` |
| KISS-EMIT-6.6-0005 | `test_emit_fidelity_not_declared_off_schema` |
| KISS-EMIT-6.6-0006 | `test_emit_artifact_abi_matches_contract` |
| KISS-EMIT-6.6-0007 | `test_emit_underived_region_recorded_as_residue` |
| KISS-EMIT-6.7-0001 | `test_emit_roundtrip_tier1_structural` |
| KISS-EMIT-6.7-0002 | `test_emit_roundtrip_tier2_numeric_same_language` |
| KISS-EMIT-6.7-0003 | `test_emit_no_cross_language_numeric_identity` |
| KISS-EMIT-6.7-0004 | `test_emit_roundtrip_tier_selected_by_determinism_class` |
| KISS-EMIT-6.7-0005 | `test_emit_emit_consume_are_siblings_no_edge` |
| KISS-EMIT-6.7-0006 | `test_emit_tier2_language_and_device_determinant` |
| KISS-EMIT-6.7-0007 | `test_emit_op_dag_equality_defined` |
| KISS-EMIT-6.7-0008 | `test_conform_emit_consume_correspondence_lint` |
| KISS-EMIT-6.7-0009 | `test_emit_whole_kernel_tier2_requires_all_exact_byte` |
| KISS-EMIT-6.8-0001 | `test_emit_decline_is_typed` |
| KISS-EMIT-6.8-0002 | `test_emit_unsupported_dtype_declines` |
| KISS-EMIT-6.8-0003 | `test_emit_build_on_miss_declines_cleanly` |
| KISS-EMIT-6.8-0004 | `test_emit_never_panic_on_adversarial_input` |
| KISS-EMIT-6.8-0005 | `test_emit_build_on_miss_no_null_artifact` |
| KISS-EMIT-7.1-0001 | `test_emit_mandatory_core` |
| KISS-EMIT-7.2-0001 | `test_emit_declared_subset_bounds_claim` |
| KISS-EMIT-7.3-0001 | `test_emit_tier2_is_optional` |
| KISS-EMIT-8.1-0001 | `test_emit_abi_version_bump_rule` |
| KISS-EMIT-8.1-0002 | `test_emit_no_abi_bump_on_compatible_change` |
| KISS-EMIT-8.2-0001 | `test_emit_freeze_precondition_audit` |
| KISS-EMIT-8.2-0002 | `test_emit_freeze_precondition_differing_spelling_emitter` |
| KISS-EMIT-8.2-0003 | `test_emit_freeze_precondition_second_consumer_roundtrip` |
| KISS-EMIT-8.2-0004 | `test_emit_freeze_signed_by_audit_role` |
| KISS-EMIT-9.1-0001 | `test_emit_claim_prerequisite_closed` |
| KISS-EMIT-9.2-0001 | `test_emit_traceability_complete` |

---

## 10. Governance

KISS-Emit's governance — the editor-of-record role, the ratifier, the RFC process, the
maturity-transition signing by the KISS-Conform AUDIT role, the specification license
(CC0 1.0 Universal), the reference-crate license (MIT OR Apache-2.0), the patent grant
with defensive termination, and the mark-use tie — is defined once in the umbrella
(umbrella §7, §9) and referenced here, not restated.

Four governance facts are specific to KISS-Emit and recorded here by reference:

- **Editor of record.** **Unpopped**, the neutral generator reference-impl project,
  ratified **2026-08-15** for **both** KISS-Emit and its inverse KISS-Consume. The two
  inverse standards therefore share a **single pen**. That is the primary guard on the
  round-trip statement of §6.7 and it replaces the no-single-pen risk this section
  previously recorded — but it does not retire the mechanical guard below, because one
  pen makes drift less likely, not impossible.
- **Editor–implementer identity (bounded).** The editor is also an implementer of both
  directions — a fact about who holds the pen, not a conformance claim; conformance
  here is self-certified with published results, as for any other implementer
  (§8-0007). Two constraints hold in consequence: the editor's
  own conformance status against the clauses it edits is **recorded in Appendix A.1 and
  kept current**; and a change to any clause enumerated in the §6.7-0008 correspondence
  table, **or to any clause whose alteration is motivated by the reference
  implementation's own difficulty, requires a cosignatory who is not the editor.** The
  second constraint was proposed by the editor about itself, and is recorded here rather
  than left as an understanding.
- **Pen-drift guard (editorial obligation).** The editor of KISS-Emit SHALL NOT alter
  the wording or intent of any round-trip clause enumerated in the §6.7-0008
  correspondence table without the corresponding, semantically-equivalent alteration to
  its KISS-Consume counterpart, and vice versa; any such change is a cross-party-visible
  change coordinated per umbrella §7.2. This is an **editorial/governance** obligation,
  not a per-emitter conformance requirement; it is verified by the KISS-Conform
  cross-standard document lint (§6.7-0008), which reads both sub-standards' texts, not by
  a runtime emitter test.
- **Freeze signing.** The Draft → Frozen transition is signed by the KISS-Conform AUDIT
  role, not the authoring editor (§8.2-0004), and only after the §8.2 preconditions (the
  completed recorded neutrality audit, a demonstrated differing-surface-spelling emitter,
  and a second-consumer round-trip certification) are met on top of the umbrella §5.3
  gate.

---

## Appendix A — Worked examples & provenance (informative)

Informative only; where an example and a normative clause differ, the clause governs.

### A.1 Provenance

The reference seed crate for KISS-Emit is **`unpopped`**, a kernel-generation crate,
with its vocabulary crate `unpopped-vocab`; its IR→Slang emitter lives in
`unpopped-slang` and demonstrates an emitter whose surface spellings differ from a
C-family emitter's (the "differing-surface-spelling emitter" of the §8.2-0002 freeze
precondition). It is *a* reference implementation with no privilege, and **being the
seed confers no conformance claim**: per §8-0007 it runs the same public, unmodified
KISS-Conform suite with no exemption, and its conformance status is whatever that
suite reports — self-certified with published results, exactly as for any other
implementer. The open divergences recorded below are the current answer. Project and
crate names appear here as non-normative provenance only.

> **Lineage (informative).** This crate was extracted from an earlier CUDA
> kernel-generation crate; the CUDA emitter went to a separate project-specific crate at
> the same time. Earlier revisions of this document named that predecessor. The name is
> recorded here so a reader tracing the provenance is not left matching a crate that no
> longer holds the thing cited.

**Editor conformance status (§10, constraint (a)).** The editor of record is also this
reference implementation, and its conformance against the clauses it edits is recorded
here rather than assumed. As of 2026-08-15 the following divergences are **known and
open**, each with a tracking issue:

| clause | divergence |
|---|---|
| §6.6-0001 / §6.6-0003 | the emit API does not return the `{artifact, contract}` pair; contract construction is a separate call the caller must make |
| §6.8-0004 | the lowering trait documents that it MAY panic on a dtype it cannot spell, with the caller expected to gate it; §6.8-0004 admits no trusted-input exemption |
| §6.2-0001 | the emitted document is a consumer-specific contract format, not a seven-section KISS-Contract |
| §6.2-0002 / §6.2-0003 | contract existence is gated on a downstream vocabulary in three places, including the whole scatter family |

**A reference implementation that diverges from the specification it seeds is a fact the
document should carry, not one a reader should have to discover.** The divergences were
measured and reported by the implementation itself, before the editorship was ratified.

### A.2 Worked emit — strided binary `add` on `f32` (partition trace)

Input `(OpDef=add, structure_key=sk4|bin|f32|cuda:sm89|ix32|grid|r2|…)`. The complete
partition of this emission's lowering decisions:

| Lowering decision | Set | Surface? | Clause |
|---|---|---|---|
| Walk the 1-node op DAG `{ op: add }` | driver-may-spell | structural (none) | §6.3-0001 |
| Bind `(const f32* in0, const f32* in1, f32* out, …)` in canonical order | driver-may-spell | structural (none) | §6.3-0002 |
| Signed-stride index arithmetic + grid-stride mapping | driver-may-spell | structural (none) | §6.3-0003, §6.3-0005 |
| Infix `+` for the `add` atom | driver-may-spell **only after** §6.5 proves it | surface (audit-gated) | §6.3-0004 |
| `f32` type surface spelling | emitter-must-supply | surface | §6.4-0004 |
| Any constant / special float value in the body | emitter-must-supply | surface | §6.4-0001, §6.4-0002 |

Every decision lands in exactly one set: the four structural decisions and the (proven)
infix operator on the driver side, everything else emitter-supplied, and anything not
enumerated on the driver side emitter-supplied by the closure rule (§6.2-0004),
disjoint by §6.2-0001. The emitted contract's Guarantees declare determinism
`exact-byte`, MathPrecision `bit-stable`; tier-2 numeric round-trip is claimable
same-language on-device (§6.7-0002/-0006), tier-1 structural across languages
(§6.7-0001/-0003).

### A.3 Worked emit — `softmax` (round-trip tier trace)

Input `(OpDef=softmax, structure_key=…|sft|…)`. `softmax` decomposes through a
transcendental `exp` and a float `sum` reduction, so its imported determinism class is
`order-invariant/nondeterministic` (KISS-OPS §6.0-0004/-0005). Consequently:

- The `exp` atom spelling is emitter-must-supply to a declared ULP (§6.4-0003).
- Tier-2 numeric round-trip is **not** claimable, even same-language (§6.7-0002,
  §6.7-0004); the round-trip stops at tier-1 structural op-DAG equality (§6.7-0007) plus
  the declared tolerance comparator.
- Claiming a bit-identical `softmax` — same language or cross-language — is the named
  overclaim trap (§6.7-0003).

### A.4 The `const_lit` sibling hunt (neutrality-audit worked example)

The audit (§6.5) walks every **surface-bearing** driver-side decision looking for a
constant spelling that leaked in through a happy-path golden vector — the C-ism `0.5f`,
a `-0.0` literal, a half-precision or FP8 literal, a signaling-NaN literal. Each such
sibling is reclassified emitter-must-supply and re-pinned as a bit pattern per dtype
(§6.5-0003, §6.4-0001). The structural driver-side decisions (topology, binding, index
arithmetic, control flow) render no surface and are outside the walk (§6.2-0005,
§6.3-0006). Only after the walk records none remaining, and records each surviving
surface-bearing driver-side operator (the infix `+ - * /`) proven universal across the
claimed targets (§6.5-0001/-0002), may KISS-Emit approach the freeze gate (§8.2).

### A.5 Open questions (informative — tracked as RFCs)

These are unresolved at this draft and are tracked as numbered RFCs in the ThinkersJournal
RFC directory; they gate no clause above except where a clause explicitly defers to them:

1. **The declared subset for tier-1 round-trip is not yet pinned** — which ops each side
   must round-trip structurally, how the subset is advertised (a capability-bit range? a
   manifest?), and whether the emitter's and the consumer's declared subsets must be
   equal or merely overlap for a **joint** round-trip claim (§7.2 pins only that a claim
   is bounded by the advertised subset; the mandatory core requires only the single-party
   self-round-trip, §7.1-0001).
2. **RESOLVED (2026-08-15) — one editor now holds both pens.** KISS-Emit and
   KISS-Consume are both edited by the neutral generator reference-impl project, so the
   no-single-pen risk this item recorded no longer applies. **What remains open is a
   design question, not a governance one:** whether the §6.7 round-trip statement should
   be extracted into a shared section both documents cite, so the asymmetry is
   structurally impossible rather than merely unlikely. Today Consume carries no
   correspondence table of its own and defers to this document's in three places, and
   §6.7-0006 — which defines the tier-2 determinants — sits outside the §6.7-0008 lint's
   binding range. A single pen makes drift less likely; a shared section would make the
   asymmetry impossible.
3. **The pre-freeze neutrality audit has not been performed** — until it is, the
   driver/emitter boundary for the **surface-bearing** driver-side spellings (at this
   schema version, the infix `+ - * /` operators of §6.3-0004) is provisional (§6.5).
   The structural driver-side decisions are audit-exempt (§6.2-0005, §6.3-0006) and are
   not affected by this open question.
4. **Artifact verification is under-specified** — the emit result is `{artifact,
   contract}` (§6.6-0003) but no clause pins whether/how a consumer verifies the artifact
   bytes against `revision_hash` or the contract's declared guarantees (the
   consumer-verify obligation is a KISS-Conform SHOULD, not a KISS-Emit MUST); whether a
   format/target tag on the artifact is needed, beyond the language-identity token of
   §6.7-0006, is open.
5. **Build-on-miss latency/async** — whether an emitter reached through KISS-Synth
   provision MAY return a "building, retry" intermediate versus blocking until the
   artifact is ready or declining `CANNOT_PROVISION` is not pinned (§6.8-0003).

---

*End of KISS-Emit (Draft proposal). This sub-standard is normative only in §6–§9; every
binding requirement is an identified clause `KISS-EMIT-<section>-<nnnn>` mapped 1:1 to a
named KISS-Conform test. Project and product names appearing in this document are
confined to non-normative examples, provenance, and the reference-implementation
pointer; normative clauses use only the generic roles provider, consumer, implementation,
emitter, neutral driver, kernel, contract, and target.*
