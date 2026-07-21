# KISS-Contract — The Universal Kernel-Contract Format

**Sub-standard ID:** KISS-CONTRACT
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification* (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ is normative. Normative
> clauses use RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-CONTRACT-<section>-<nnnn>`, and each MUST/SHALL maps 1:1 to at least one
> named KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Contract |
| Sub-standard ID | KISS-CONTRACT |
| Tier | **Middle** (the universal kernel-contract format; sits above the two foundational vocabularies and the advertisable-op surface, and below the protocol tier that transports it) |
| Maturity stage | **Draft** (first-draft proposal; the seven-section contract schema is NOT yet frozen — the freeze gate of §8 is unmet) |
| Editor of record | **Proposed, pending ratification** — a kernel-contract reference-impl project holds the pen and requests comment from interested cosignatories; the ratified governance record does not yet finalize an editor for KISS-Contract. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | a contract-emission reference crate (`baracuda-kernelgen`, project/crate name given in Appendix A as non-normative provenance); this crate is *a* conformant implementation with no privilege. |
| DAG position | **Middle tier.** Depends (structurally) on KISS-Classify, KISS-Ops, and KISS-Grammar; consumed opaquely by KISS-Announce (the contract-query payload), and structurally by KISS-Synth/Provision, KISS-Consume, and KISS-Emit. Not a root. |
| Upstream edges | KISS-Classify (**STRUCTURAL** — the Identity accept-predicate is a `structure_key` and the Identity `target_capability` is a Classify namespaced descriptor (referenced, not re-pinned, by the Interface for entry-point selection); the Interface operand descriptors are Classify vocabulary); KISS-Ops (**STRUCTURAL** — the Semantics op DAG's nodes are KISS-Ops op names carrying the KISS-Ops OpAttrs channel, and the Guarantees determinism class and MathPrecision attribute are imported from KISS-Ops); KISS-Grammar (**STRUCTURAL**, present only when a contract names an advertisable op — the Identity `op_identity` field is then a full KISS-Grammar advertisable-op tag and the Semantics DAG's non-primitive nodes MAY correspond to advertisable ops (reconstructed as tags over each node's bare op name); a contract with no advertisable-op name carries a bare KISS-Ops op DAG as its Semantics and the bare op name of that DAG's root as its `op_identity` instead, and KISS-Grammar is not required for every kernel) |
| Downstream edges | KISS-Announce (**OPAQUE** — carries the contract document as an uninterpreted, length-delimited contract-query payload); KISS-Synth/Provision (**STRUCTURAL** — provision returns `{artifact, contract}`); KISS-Consume (**STRUCTURAL** — lifting produces a contract's Semantics field and residue); KISS-Emit (**STRUCTURAL** — the emitter's output is described by a contract); KISS-Conform (test dependency) |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |
| Maturity | Draft proposal |

> **Edge-label note (informative).** All three KISS-Contract upstream edges are
> **STRUCTURAL**: KISS-Contract parses the internal structure of a KISS-Classify
> `structure_key` / operand descriptor / `target_capability`, of a KISS-Ops op
> definition (its name, OpAttrs channel, determinism class, MathPrecision
> attribute, and reference decomposition), and of a KISS-Grammar advertisable-op
> tag and region. The labels reconcile with the umbrella §2.2 edge table, which
> lists **KISS-Classify → KISS-Contract**, **KISS-Ops → KISS-Contract**, and
> **KISS-Grammar → KISS-Contract** each as STRUCTURAL. On the downstream side, the
> **KISS-Contract → KISS-Announce** edge is **OPAQUE** (announce carries the
> contract document without parsing it), while **KISS-Contract → KISS-Synth**,
> **KISS-Contract → KISS-Consume**, and **KISS-Contract → KISS-Emit** are
> STRUCTURAL. All labels are consistent with the umbrella §2.2 edge table.

---

## 1. Purpose & Scope

KISS-Contract owns the **universal, vendor-neutral kernel-contract format** of the
suite: the single self-delimiting document that travels with every provided kernel
and tells a consumer, in one place, **what a kernel computes** and **exactly how to
call it**. It is the middle-tier document that binds the two foundational
vocabularies (KISS-Classify data nouns, KISS-Ops computation verbs) and the
advertisable-op surface (KISS-Grammar) into a per-kernel source of truth. It defines
seven things and nothing else:

1. **Identity** — which kernel this is, joining two orthogonal identities into a
   single match: which specialization **cell** fits (the `structure_key`
   admissibility predicate, KISS-Classify) **and** which **op** this computes (the
   `op_identity` — either a full KISS-Grammar advertisable-op tag re-based onto a
   KISS-Ops op name, or, for a kernel with no advertisable-op name, the bare KISS-Ops op
   name of the Semantics DAG root's op). Neither alone is kernel identity; a consumer
   matches both.

2. **Semantics** — the single mandatory spine: a hierarchical op DAG at **mixed**
   abstraction levels over KISS-Ops op names and KISS-Grammar advertisable ops,
   recursively resolvable to the KISS-Ops primitive floor (the termination
   guarantee), each node carrying the KISS-Ops OpAttrs channel and edge-case policy.

3. **Interface (ABI)** — everything needed to **mechanically call** the kernel: the
   full positional argument signature (operand pointers in KISS-Classify canonical
   operand order, then the runtime launch scalars) with declared types and a
   normative order.

4. **Dispatch** — the normative launch/geometry model: the invocation/index domain,
   workgroup sizing, the count-unit→grid derivation, thread→element mapping, and the
   signed-stride / base-offset addressing rule.

5. **Capabilities** — the declared per-kernel envelope (supported dtype set,
   awkward-layout strategy, in-place-eligible variants, index width, determinism
   class, precision/ULP class, cost class) whose accept-predicate is the
   `structure_key`.

6. **Guarantees** — the unified numeric-guarantee block: the precision reference
   function **named**, per-backend ULP tiers, the determinism class and MathPrecision
   attribute (imported from KISS-Ops), bit-stability, and the **audited status
   derived** from those declared guarantees (its single home), not authored as a constant.

7. **Provenance** — origin and trust in one place: kernel source, revision base and
   hash, cost provenance, and negotiation metadata.

**Every kernel — generated, lifted, or hand-written — must carry a contract with the
same universal required core.** Contract existence is decided by the contract's own
Semantics field, never by whether some downstream named-op vocabulary happens to have
a matching name. Completeness tracks the **lift fraction**: the Semantics field is
machine-checkable IR for a fully-lifted/generated kernel and degrades to a declared
op-identity tag plus a recorded residue for the un-liftable remainder of a
hand-written kernel — still honest, always carrying the universal core.

**KISS-Contract is NOT:** the data vocabulary (the dtype set, operand descriptor,
`structure_key`, and `target_capability` are KISS-Classify, used here by
name/structure); the computation vocabulary or per-op semantics (the op set, NaN /
signed-zero / wrapping / ULP behavior, the OpAttrs sub-vocabularies, and reference
decompositions are KISS-Ops, resolved from there, never restated); the advertisable-op
surface or region grammar (that is KISS-Grammar, whose advertisable-op tag this
contract *carries* in Identity and whose region maps onto the Semantics DAG); the
discovery/handshake protocol (that is KISS-Announce, which transports the contract as
an opaque token); the provision protocol (that is KISS-Synth/Provision, which returns
`{artifact, contract}`); the recognition/lift direction (KISS-Consume) or the
generation/lower direction (KISS-Emit); a kernel implementation, a source language, or
a compiler IR's internals. Anything not enumerated as in-scope above is out of scope
for KISS-Contract (scope creep by silence is a named trap; silence is not inclusion).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — one document, every fact one home

A consumer that receives a kernel and cannot learn *how to call it* or *what it
computes* has received something unusable. Today a consumer typically gets a `dlsym`
symbol and nothing else: no argument signature, none of the strided launch scalars,
no gather extent, and a launch geometry declared "provider-internal." The earlier
reference contract was *a good provider feed but a poor source of truth* — a flat
field list where "how to call it" was smeared across five places and the meaning was a
trailing, fusion-only afterthought.

KISS-Contract gives every fact **exactly one home** in a seven-section document:
Identity, Semantics, Interface, Dispatch, Capabilities, Guarantees, Provenance. The
biggest gap it closes is **Interface + Dispatch** — the "how to call it" the contract
exists FOR. A consumer binds and launches purely from the Interface's positional
signature and the Dispatch launch model; it never reaches out-of-band for an ABI
shared tacitly between two projects.

### 2.2 Two orthogonal identities, matched jointly

The Identity section joins two identities that are easy to conflate:

- **Which CELL fits** — the `accept_predicate = structure_key` (KISS-Classify): an
  **admissibility predicate** over a layout/dtype/target specialization cell — a
  coarse op-**category** tag + canonically-ordered operand descriptors + the target +
  op-category role hints, **extent-free** (keyed by size classes, not literal
  extents). It answers "does this invocation fall into this kernel's cell?" by
  byte-matching the derived key. It is explicitly **not** the op's semantic identity:
  its cell-level op-category is a coarse Classify category, a different closed set from
  a KISS-Ops op name.

- **Which OP is this** — the `op_identity`: the op the Semantics DAG root computes,
  carried either as a full KISS-Grammar advertisable-op tag re-based onto a KISS-Ops op
  name, or — for a hand-written kernel with no advertisable-op name — as the bare KISS-Ops
  op name of that root op (with its identity-bearing attributes). It answers "which
  computation is this?". KISS-Grammar is the advertisable/named-op surface only and is
  **not** required for every kernel; a contract whose `op_identity` is a bare KISS-Ops op
  name is accepted (the full op DAG lives only in the Semantics section).

A consumer must match **both**. Two kernels can share a cell (same `structure_key`)
yet compute different ops, and one op can span many cells. So Identity carries the
`structure_key` and the `op_identity` as two distinct fields, requires them jointly
for a match, and asserts their **consistency**: the `structure_key`'s coarse
op-category must be compatible with the `op_identity`'s KISS-Ops family, per the
normative compatibility table of §6.3, so the two identities cannot silently disagree.

### 2.3 Semantics is a hierarchical, resolvable op DAG — the mandatory spine

A kernel's Semantics is a DAG of ops at **mixed** abstraction levels: a fused
`matmul` + `gelu` epilogue is two nodes, NOT the primitive soup of multiply-adds,
`exp`, and polynomial atoms it would expand to. Every non-primitive node is
**resolvable**: a consumer that does not know an op acquires that op's contract (via
the KISS-Announce contract-query) and resolves its KISS-Ops reference decomposition
recursively to the KISS-Ops primitive floor. The termination guarantee is inherited
from KISS-Ops (acyclic, strictly-decreasing level); the fully-lowered primitive form
is produced on demand as the **verification oracle** under the op's determinism class.

Making Semantics **mandatory and decomposition-backed** is what fixes the old
"honest-miss" bug where a reduction, a gather, or a scatter produced *no* contract.
The op DAG can describe anything KISS-Ops can name — a reduction as
`reduce, op=sum, body=input(0)`, a gather with its axis and OOB policy on the OpAttrs
channel, a scatter, an offset slice — so a kernel is describable even to a consumer
with no native op for it.

For a generated or lifted kernel the Semantics field is machine-checkable IR; for a
hand-written escape-hatch kernel it degrades to a declared op-identity tag plus a
recorded **residue** (the KISS-Consume refusal taxonomy) for the un-liftable
remainder — still honest, never faking a named semantics it lacks.

### 2.4 The Interface — the full ABI, in one place

The Interface section is the concrete, positional ABI signature: the operand pointers
in KISS-Classify canonical operand order (`in0..in{k}`, then the output(s)), one typed
pointer per operand at its dtype (a gather's index operand at its own dtype, e.g. an
unsigned-32 index pointer), **followed by** the runtime launch scalars in a single
pinned, contract-declared order. The launch-scalar list — the how-to-call-it that has
no home today — is enumerated: per-operand extents, per-operand signed strides, the
iteration element count `n`, per-operand base offsets, gather/index extents, a
workspace pointer and its byte size, and scalar op params. Every element carries a
declared type and a declared position; the ordering is normative, so a consumer binds
and launches purely from the signature. The operand `rank` is a declared Interface
field, so a consumer sizes the per-operand `extents[rank]` / `strides[rank]` arrays
from the contract alone. `count_unit`, `in_place`, and `alignment_bytes` move OUT of
the old capabilities grab-bag into this section.

### 2.5 A worked contract — a strided binary `add` on `f32`

Consider a generated binary elementwise `add` over `f32`, target `cuda:sm89`, on a
strided cell. Its contract carries:

- **Identity:** `contract_kind = kiss-contract`; `op_identity = add` (a bare
  advertisable-op tag over the KISS-Ops name `add`); `accept_predicate` = the cell's
  `structure_key` token (op-category `bin`, three `f32` operands, strided);
  `target_capability = cuda:sm89`; a `revision_hash`.
- **Semantics:** a one-node DAG `{ op: add }` — a primitive, so a 1-node DAG; edge-case
  policy resolved from KISS-Ops (`add` is IEEE-754, NaN-propagating).
- **Interface:** `rank = 1`; signature `(const f32* in0, const f32* in1, f32* out, i64
  extents0[rank], i64 extents1[rank], i64 extentsOut[rank], i64 strides0[rank], i64
  strides1[rank], i64 stridesOut[rank], i64 n)`; `count_unit = elements`; `in_place =
  none`; `alignment_bytes` per the cell.
- **Dispatch:** grid-stride mapping; `n` elements → grid; per-thread addressing via the
  signed strides.
- **Capabilities:** `accept_predicate = structure_key`; determinism class `exact-byte`;
  precision class `correctly-rounded`; cost class `elementwise`, cost `1 * n`.
- **Guarantees:** reference function `add` (IEEE-754); ULP tier 0 (correctly rounded);
  determinism `exact-byte`; MathPrecision `bit-stable`; bit-stable on same hardware;
  `audited_status` **derived** here from those guarantees.
- **Provenance:** kernel source (generator), revision base + `revision_hash`,
  `cost_provenance = declared`, negotiation metadata (empty).

### 2.6 A worked contract — a `gather` with load-bearing attributes

A `gather` over a data operand and an unsigned-32 index operand, non-default axis `k`,
OOB policy `clamp`. Its Semantics node is `{ op: gather, op_attrs: { axis: k, oob:
clamp } }`; its `op_identity` is the advertisable-op tag `gather` distinguished by
those attribute values (per KISS-Grammar, more than one advertisable op may re-base
onto the same op name when distinguished by attribute values, so `op_identity` carries
the distinguishing attributes, not the bare name; it renders on the contract document's
`op_identity` line as `gather{axis=k, oob=clamp}` per §6.11-0009). Its Interface signature carries the
index operand's own dtype pointer plus the gather/index extents launch scalars; the
axis and OOB policy are matched with an explicit guard. This is exactly the case the
old flat grammar could not express (no attrs channel, no per-operand dtype tuple), so
those kernels formerly produced no contract; under the neutral hub they carry a full
contract.

### 2.7 Every kernel has a contract — the honest-miss gates are removed

The earlier reference emitter withheld a contract (returned "no contract") in many
cases: a primitive with no matching foreign `OpKind`, a fusion outside a foreign
fused-op whitelist, any gather/scatter/offset/addressing-view op, and comparison/select
bodies. **Every one of those withholds existed only because the old contract was tied
to a foreign named-op vocabulary.** Under the neutral hub the Semantics field plus the
OpAttrs channel plus the per-operand dtype tuple carry the axis, OOB policy, and index
dtype the old grammar could not, so those cells become **full contracts** — or, where a
hand-written kernel genuinely cannot be fully lifted, **honestly-partial** contracts —
never absent. Contract existence is decided by the contract's own Semantics, not by a
foreign name. This document therefore states **no** honest-miss gate and **no** op-kind
existence gate: a kernel that generates and runs always gets a contract.

### 2.8 Transport — self-delimiting structured/text document that fails loudly

A KISS-Contract is a self-delimiting, strict-schema **structured/text document** — **not** a
length-prefixed binary envelope. It is framed by a pinned magic, a version field, and a declared
**header line**, with an **inner length and checksum** that make its extent and integrity
self-describing (§6.11): the header line begins with the 4-byte magic `KISC`, then the
`kiss-contract` kind, the version, a `len=<N>` inner body byte-length, and a `crc32=<…>`
checksum, and the body is seven section blocks under pinned heading lines. Two independent
implementations serialize and parse the identical bytes. This replaces the earlier
markdown-fenced transport, whose parser **silently dropped** a block that did not sit under a
heading — so a bundle that merely concatenated blocks imported "OK but empty," a no-op that
looked like success. The neutral transport makes a malformed, magic-less, headingless, or
unknown-version contract a **hard, typed decline**: the reader hard-rejects, never repairs, never
silently ignores — exactly the KISS-Announce POD reader discipline. Across the KISS-Announce
contract-query seam the contract travels as an **opaque, length-delimited payload** (the Announce
contract response frames a u32 payload byte-length then that many bytes); Announce never parses
the internals, so the contract document's own inner framing (its magic, header line, inner
length/checksum, and section headings) must fail loudly on its own terms, independent of the
enclosing frame.

### 2.9 Terms are joined, not restated

KISS-Contract references the KISS-Classify dtype tokens, operand descriptors,
`structure_key`, and `target_capability` by name/structure; the KISS-Ops op names,
OpAttrs channel, reference decompositions, primitive floor, determinism class,
precision-class token set, and MathPrecision attribute by name; and the KISS-Grammar
advertisable-op tag and region by name. It re-defines none of them and defines no op
meaning: Contract carries the identities and the ABI, the foundational vocabularies
mean them.

---

## 3. Terms & Definitions

- **Contract (KISS-Contract)** — the self-delimiting, strict-schema per-kernel document
  defined by this sub-standard, carrying the seven sections Identity, Semantics,
  Interface, Dispatch, Capabilities, Guarantees, Provenance. The neutral successor to a
  vendor "kernel contract"; spelled out, no new opaque acronym.
- **Universal required core** — the seven sections every conforming contract MUST carry,
  regardless of the kernel's origin or lift fraction (§6.2).
- **Identity section** — the section joining the `accept_predicate = structure_key`
  (which cell fits) and the `op_identity` (which op this is) into one joint match
  (§6.3).
- **accept_predicate / structure_key** — the KISS-Classify admissibility predicate over
  a specialization cell (a coarse op-category + canonically-ordered operand descriptors
  + target + op-category role hints, extent-free); a kernel admits an invocation iff the
  derived `structure_key` byte-matches the kernel's key. **Not** the op's semantic
  identity.
- **op_identity** — the identity of the op the kernel computes, naming the Semantics DAG root's
  op, in one of two forms (§6.3-0003), encoded on the `op_identity` line per §6.11-0009:
  **(a)** a KISS-Grammar advertisable-op **tag** in full — the bare KISS-Ops op name **plus** the
  distinguishing identity-bearing attribute values that the tag carries (the pattern-channel
  OpAttrs values and, where present, the operand-role tuple, per §6.3-0004 / KISS-GRAMMAR
  §6.1-0007), a bare KISS-Ops op name being the special case whose tag carries no distinguishing
  attribute values (neither pattern-channel OpAttrs values nor an identity-bearing operand-role
  tuple); or **(b)** the **bare KISS-Ops op name** of
  the Semantics DAG root's op (plus that root's identity-bearing OpAttrs values), with no
  KISS-Grammar advertisable-op entry, for a kernel with no advertisable-op name — the full op
  DAG lives only in the Semantics section and is never duplicated into `op_identity`. Never a
  private op enum.
- **kernel_name** — the Identity section's stable identity name of the kernel. It is
  the contract's name for the kernel and MAY differ from the Interface `entry_point`
  (the callable symbol name); the two are not assumed equal (§6.3-0010).
- **target_capability** — the KISS-Classify namespaced `<namespace>:<capability-set>`
  all-hardware descriptor of the compilation target the kernel is built for, matched
  byte-exact on the full string.
- **Semantics section** — the mandatory hierarchical op DAG over KISS-Ops op names and
  KISS-Grammar advertisable-op tags at mixed abstraction levels, recursively resolvable
  to the KISS-Ops primitive floor (§6.4).
- **semantics_kind** — the Semantics field's flavor: `machine-checkable-IR` (a
  generated/lifted kernel) or `declared-op-tag` (a hand-written escape-hatch kernel).
- **human_annotation** — an optional, non-normative, free-text field of the Semantics
  section for human readers; it is outside the exact-byte schema scope (§6.0-0001) and
  is never byte-compared by KISS-Conform (§6.4-0001).
- **op DAG / op node** — a node of the Semantics DAG under the node schema
  `Op{op_name, op_attrs, child_edges} | Bind(positional_index)` (§6.4-0009); each node's
  `op_name` is a bare KISS-Ops op name (identity-bearing attributes ride the `op_attrs`
  channel, and a node **corresponds to** an advertisable op when its `op_name` + identity-bearing
  `op_attrs` + operand-role tuple reconstruct a KISS-Grammar advertisable-op tag, never a tag
  spelled inside `op_name`); a primitive is a 1-node DAG, a fusion is its DAG.
- **OpAttrs channel** — the per-node attribute vocabulary (axis / out-of-bounds policy /
  permutation / reduce-axis mask/keepdim) **owned by KISS-Ops**, surfaced by KISS-Grammar
  as pattern attributes and carried per node in the Semantics DAG.
- **decomposition_resolution** — the rule that every non-primitive op node is resolved by
  acquiring that op's contract (via the KISS-Announce contract-query) for its KISS-Ops
  reference decomposition, recursively, until every chain bottoms out at the KISS-Ops
  primitive floor.
- **lift fraction / lift_residue** — the fraction of a hand-written kernel that has been
  lifted into the op DAG; the un-liftable remainder is the recorded residue (the
  KISS-Consume refusal taxonomy). Contract **completeness** tracks the lift fraction.
- **Interface section (ABI)** — the full positional argument signature (operand pointers
  in KISS-Classify canonical operand order, then launch scalars in a pinned order) plus
  `rank`, `count_unit`, `in_place`, and `alignment_bytes` (§6.5).
- **positional_signature** — the ordered, typed argument list a consumer binds and
  launches from; the authoritative argument order.
- **launch_scalars** — the enumerated runtime arguments (per-operand extents, per-operand
  signed strides, the iteration element count `n`, per-operand base offsets, gather/index
  extents, workspace pointer, workspace byte size, and scalar op params); carried as a
  typed sub-view equal to the launch-scalar tail of `positional_signature` (§6.5-0011).
- **rank** — the fixed, per-contract compile-time-constant operand logical rank; every
  per-operand `extents` / `strides` array has length `rank`, while the base offset `off{i}`
  is a single scalar per operand, not a rank-length array (§6.5-0012).
- **fully packed** — a cell whose KISS-Classify contiguous-layout predicate marks every
  operand contiguous in canonical order with unit innermost stride; a non-packed cell
  (strided / broadcast / reversed) carries the class-2 signed strides (§6.5-0005).
- **count_unit** — the unit `n` is expressed in: `elements` or `vectors_x<w>` where the
  exact spelling is the literal `vectors_x` immediately followed by the decimal vector
  width with no delimiter (for example `vectors_x4`); load-bearing for the Dispatch
  count→grid derivation.
- **Dispatch section** — the normative launch/geometry model, each derivation field a
  machine-evaluable expression over the launch-scalar symbol vocabulary (§6.6).
- **Capabilities section** — the declared per-kernel envelope (not a per-call signature):
  supported dtype set, awkward-layout strategy, in-place-eligible variants, index width,
  declared determinism class, declared precision/ULP class, cost class and cost
  expressions (§6.7).
- **Guarantees section** — the unified numeric-guarantee block: named reference function,
  per-backend ULP tiers, determinism class, MathPrecision attribute, bit-stability, the
  **derived** `audited_status` (its single home, §6.8-0008/-0009/-0010), and cost provenance
  (§6.8).
- **reference_function** — the NAMED KISS-Ops function precision is measured against.
- **determinism class** — the single canonical KISS-Ops enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` (KISS-OPS §6.0-0001), imported
  verbatim, never re-forked.
- **precision_class** — the compute-precision class drawn from the closed precision-class
  token set imported verbatim from KISS-Ops, mapped to a per-backend ULP tier by the
  KISS-Ops precision-class↔ULP-tier correspondence (KISS-OPS §6.8); `correctly-rounded`
  and `bit-reproducible` map to ULP tier 0 (§6.7-0005).
- **MathPrecision attribute** — the KISS-Ops compute-fidelity attribute `{bit-stable,
  reduced-mantissa-permitted}` (KISS-OPS §6.17), imported from KISS-Ops, orthogonal to the
  determinism class, and **not** a dtype.
- **cost_provenance** — the provenance of the Capabilities cost model, `declared` or
  `measured`; authored in Guarantees and mirrored in Provenance (§6.8-0006, §6.9-0006).
- **Provenance section** — origin and trust: kernel source, revision base, `revision_hash`,
  cost provenance, and `negotiation_metadata` (§6.9). The `audited_status` is **not** here — it
  is a derived Guarantees field (§6.8-0008).
- **negotiation_metadata** — an opaque, length-delimited byte blob of provider-to-consumer
  negotiation hints in the Provenance section; MAY be empty, never interpreted as contract
  semantics (§6.9-0008).
- **revision_hash** — an opaque, provider-assigned build identifier of a specific
  revision of the kernel behind a `structure_key`, compared only for byte-for-byte
  equality (no hash algorithm implied).
- **entry_point** — the callable symbol name of the kernel (Interface section).
- **audited_status** — a **derived** trust field of the **Guarantees** section, derived from the
  Guarantees determinism class and precision per the normative derivation rule of §6.8-0009 /
  §6.8-0010, never an authored constant; its single home is the Guarantees section (§6.8-0008).
- **Transport / self-delimiting document** — the contract's framing as a self-delimiting
  **structured/text document** (not a length-prefixed binary envelope): a pinned magic + a
  declared header line carrying the kind, version, an inner body length, and a checksum, over
  seven section blocks under pinned heading lines (§6.11), hard-rejected on a malformed or
  headingless document (§6.1).
- **Typed decline** — a structured refusal returned in lieu of a result (a distinguished
  error value/enumerant, or an equivalent out-of-band error return); never a panic, abort,
  crash, hang, or out-of-bounds read.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point semantics; referenced transitively through KISS-Ops
  (KISS-Contract defines no numeric behavior of its own).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword
  convention, the normative/informative split, the clause-ID scheme and 1:1 test mapping,
  value pinning as bits/IEEE-754 in wire order, the ban on unquantified adjectives, the
  two version axes, the ≥2-dissimilar-implementations-plus-foreign-reader freeze gate, the
  capability/profile/extension model, governance, licensing, and patent posture. **Stated
  once in the umbrella; referenced here; never restated.** This sub-standard's §5 points at
  umbrella §3 for conventions.
- **KISS-Classify** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: the Identity `accept_predicate` is a `structure_key` (the admissibility
  predicate over a specialization cell); the `target_capability` namespaced descriptor is
  Classify vocabulary; the Interface operand pointers follow the KISS-Classify **canonical
  operand order**, carry KISS-Classify dtype tokens, and the launch scalars (extents,
  signed strides, index width `idx32`/`idx64`) are described with the Classify operand
  descriptor; the closed op-category set and the contiguous-layout ("fully packed")
  predicate are Classify vocabulary. The cell-level op-category of a `structure_key` is a
  Classify category, distinct from a KISS-Ops op name. Used here by name/structure;
  re-defined nowhere.
- **KISS-Ops** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency:
  the Semantics op DAG's nodes are KISS-Ops op names carrying the KISS-Ops **OpAttrs**
  channel (axis / OOB policy / permutation / reduce-axis); per-node NaN / signed-zero /
  edge-case behavior and the per-transcendental declared-ULP ceiling are resolved **from**
  the KISS-Ops op semantics, never restated; each non-primitive node's reference
  decomposition (the resolution oracle) and the **primitive floor** (the termination
  guarantee) are owned by KISS-Ops; the Guarantees **determinism class** (the single
  canonical enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`,
  KISS-OPS §6.0-0001), the closed **precision-class** token set and its ULP-tier
  correspondence (KISS-OPS §6.8), and the **MathPrecision** attribute `{bit-stable,
  reduced-mantissa-permitted}` (KISS-OPS §6.17) are imported verbatim. KISS-Contract
  re-defines none of them and defines no op meaning.
- **KISS-Grammar** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency (present only when a contract names an advertisable op — KISS-Grammar is the
  advertisable/named-op surface and is **not** required for every kernel, §6.2-0006): when
  present, the Identity `op_identity` field is a full KISS-Grammar advertisable-op tag
  re-based onto a KISS-Ops op name (the Semantics DAG root's op), and the Semantics DAG's
  non-primitive nodes MAY correspond to advertisable ops (reconstructed as tags over the
  node's bare op name, §6.4-0009); a contract with no advertisable-op name carries the bare
  KISS-Ops op name of its Semantics DAG root as `op_identity` instead (§6.3-0003). The
  Semantics op-DAG node schema is a
  **projection** of the KISS-Grammar region node grammar (`Op { op_name, pattern_attrs,
  operand_role_tuple, operands, consumers } | Bind(input_index)`) onto the Contract-local
  fields carried per node, plus the per-node OpAttrs channel (§6.4-0009); `extract` is a
  KISS-Grammar region-level field (a root-anchored list of (path, param_slot),
  KISS-GRAMMAR-6.4-0007), **not** a node field. A KISS-Grammar advertisable region maps
  onto the Semantics op DAG (a one-node region → a one-node DAG, a fusion → its DAG). Used
  here by tag/name only; KISS-Contract carries no private op enum.
- **KISS-Announce** (by version) — DAG edge labeled **OPAQUE**, **downstream** consumer:
  carries the contract document as an uninterpreted, length-delimited contract-query
  payload (the Announce contract response frames a u32 payload byte-length then that many
  bytes) and never parses its internals; a resolver acquires a referenced op's contract
  through the KISS-Announce contract-query (§6.4-0005).
- **KISS-Synth/Provision** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: provision returns `{artifact, contract}`; every provided kernel carries its
  contract.
- **KISS-Consume** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: lifting produces a contract's Semantics field as far as it goes, with the
  un-liftable remainder recorded as `lift_residue` per the KISS-Consume refusal taxonomy.
- **KISS-Emit** (by version) — DAG edge labeled **STRUCTURAL**, **downstream** consumer:
  the emitter's output is described by a contract (the Interface + Dispatch of the emitted
  kernel).
- **KISS-Conform** (by version) — depends on and tests KISS-Contract; owns the
  oracle-differential harness that resolves a Semantics DAG to the primitive floor and
  compares under the op's declared determinism class, and the negative-vector modality that
  exercises the hard-reject transport discipline.

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase
keywords `MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance and
consumer-behavior guidance and never state a structural or wire requirement. Every atomic
requirement carries a stable, append-only ID `KISS-CONTRACT-<section>-<nnnn>`, allocated
by the editor of record, never reused after retirement, and mapped 1:1 to ≥1 named
KISS-Conform test. Values are pinned as tokens/schema/bytes spelled exactly as the
upstream foundational vocabularies pin them and as the §6.11 structured/text document framing
renders them, never as one source language's surface spelling. Unquantified adjectives ("well-formed",
"reasonable", "neutral", "valid") are banned from normative text. Every clause declares
its determinism/fidelity class so KISS-Conform selects the correct comparator. See
umbrella §3 for the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

- **KISS-CONTRACT-6.0-0001** — Every structural obligation in §6–§8 (the document
  framing, the structured/text document format of §6.11, the seven-section schema, the field
  schema of each section, every field spelling, and every token spelling) is determinism-class
  **exact byte compare**; KISS-Conform MUST evaluate each such clause with a byte-exact comparator
  and MUST NOT apply tolerance or order-invariant comparison. The optional, non-normative
  `human_annotation` field (§6.4-0001) is **outside** this exact-byte scope and MUST NOT
  be byte-compared. KISS-Contract defines no numeric result of its own; the numeric
  determinism class of any op a contract names, and of the numeric guarantees it carries,
  is **owned by KISS-Ops** (the single canonical enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}`, KISS-OPS §6.0-0001) and MUST NOT be re-forked here.
  *Test:* `test_contract_determinism_class_exact_byte`.

### 6.1 Transport — self-delimiting structured/text document, hard-reject discipline

A contract is a self-delimiting **structured/text document**, **not** a length-prefixed binary
envelope: it is framed by a pinned magic, a version field, and a declared header line, with an
inner length and checksum, and it **fails loudly** on a malformed or headingless document. The
exact bytes of the header line, the inner length/checksum, and the section headings are pinned
in §6.11. This inner framing is self-sufficient and is **independent** of the KISS-Announce
outer length-prefix (§6.1-0005).

- **KISS-CONTRACT-6.1-0001** — A contract MUST be a **self-delimiting structured/text
  document** whose pinned **header line** (§6.11-0002) declares the magic, the `contract_kind`,
  and the `contract_version`, and whose header line declares its own inner body byte length and
  a checksum (§6.11-0003); a reader MUST be able to determine the document's exact byte extent
  from the document itself (the header line plus its declared body length), independent of any
  enclosing transport frame. A contract MUST NOT be framed as a length-prefixed binary
  envelope. *Test:* `test_contract_self_delimiting_document`.
- **KISS-CONTRACT-6.1-0002** — A reader MUST **fail loudly** — reject with a **typed decline** —
  a contract that does not begin with the pinned magic (§6.11-0002), whose header line is absent
  or malformed, that does not declare a recognized `contract_kind`, whose body is not organized
  under the pinned section heading lines (§6.11-0004), or whose declared inner length or
  checksum does not validate (§6.11-0003); the reader MUST NOT repair, MUST NOT silently ignore,
  and MUST NOT import a **headingless** or magic-less block as an empty or no-op contract. *Test:*
  `test_contract_reject_malformed_header`.
- **KISS-CONTRACT-6.1-0003** — A reader MUST reject, with a typed decline, a contract whose
  declared `contract_version` it does not support, and MUST NOT partially import it. *Test:*
  `test_contract_reject_unknown_version`.
- **KISS-CONTRACT-6.1-0004** — On any rejection under §6.1-0002 / §6.1-0003, a reader MUST
  return a typed decline and MUST NOT panic, abort, crash, hang, or read outside the input
  buffer, and MUST NOT allocate on an unchecked declared length. *Test:*
  `test_contract_rejection_is_typed_decline`.
- **KISS-CONTRACT-6.1-0005** — When transported across the KISS-Announce contract-query
  seam, a contract MUST be carried as an **opaque, length-delimited payload** (a `u32` payload
  byte-length followed by that many bytes of the self-delimiting structured/text document, per
  the KISS-Announce contract response); the contract's **inner** framing (the magic, header
  line, inner length/checksum, and section headings of §6.11) MUST fail loudly on malformation
  **independently** of that outer length prefix, and KISS-Announce MUST NOT be required to parse
  the contract's internals. *Test:* `test_contract_opaque_length_delimited_over_announce`.
- **KISS-CONTRACT-6.1-0006** — A contract document MUST use a **strict, closed** schema: a
  reader MUST reject, with a typed decline, a contract that (a) omits a **required** field of
  a present section — a field the section's field-schema clause marks **optional** (such as
  the Semantics `human_annotation`, §6.4-0001) MAY be absent, and its absence is **not** a
  decline — (b) carries a field whose declared type does not match this schema, or
  (c) carries a field this schema does not define at the document's declared
  `contract_version` (an unknown or extra field). The reader MUST NOT silently drop,
  reserve, or ignore an unknown field; forward compatibility is obtained only by a
  `contract_version` bump (§8), never by tolerating unknown fields. *Test:*
  `test_contract_strict_schema`.
- **KISS-CONTRACT-6.1-0007** — The only recognized `contract_kind` at this schema version
  MUST be the exact token `kiss-contract` (UTF-8, byte-exact); a reader MUST reject, with a
  typed decline, any contract whose `contract_kind` is not exactly that token. *Test:*
  `test_contract_kind_recognized_token`.
- **KISS-CONTRACT-6.1-0008** — The `contract_version` of **this** schema is the exact decimal
  token `1`; a reader conforming to this schema version MUST accept a contract whose
  `contract_version` is exactly `1` (UTF-8, byte-exact) and MUST reject, with a typed decline,
  any other `contract_version` value (§6.1-0003). *Test:* `test_contract_version_value_pinned`.

### 6.2 The universal required core and contract existence

- **KISS-CONTRACT-6.2-0001** — Every contract MUST carry the **universal required core** of
  seven sections: Identity (§6.3), Semantics (§6.4), Interface (§6.5), Dispatch (§6.6),
  Capabilities (§6.7), Guarantees (§6.8), and Provenance (§6.9). An implementation MUST NOT
  emit a contract that omits any of the seven sections at this schema version. *Test:*
  `test_contract_seven_section_core`.
- **KISS-CONTRACT-6.2-0002** — Contract **existence** MUST be decided by the contract's own
  Semantics field (§6.4) and MUST NOT be gated on whether a downstream named-op vocabulary
  has a matching op name, fused-op name, backend name, or dtype spelling; an implementation
  MUST NOT withhold a contract for a kernel that generates and runs on the ground that its
  op has no matching foreign name. *Test:* `test_contract_existence_is_intrinsic`.
- **KISS-CONTRACT-6.2-0003** — A kernel whose computation is expressible as a KISS-Ops op
  DAG — including a reduction, a scan, a gather, a scatter, an offset/sliced read, a
  comparison, or a `select` — MUST receive a contract; an implementation MUST NOT treat any
  of these op classes as a per-se contract-withholding condition. The axis, out-of-bounds
  policy, permutation, reduce-axis, and per-operand index dtype that such a kernel needs MUST
  be carried on the Semantics OpAttrs channel (§6.4) and the Interface per-operand dtype
  tuple (§6.5), never used as a reason to omit the contract. *Test:*
  `test_contract_no_op_class_withhold`.
- **KISS-CONTRACT-6.2-0004** — Contract **completeness** MUST track the **lift fraction**:
  the Semantics field MUST be machine-checkable IR (`semantics_kind = machine-checkable-IR`)
  for a fully-lifted or generated kernel, and MUST degrade to a declared op-identity tag
  (`semantics_kind = declared-op-tag`) plus a `lift_residue` recording the un-liftable
  remainder per the KISS-Consume refusal taxonomy for a hand-written kernel; in neither case
  MAY the universal required core (§6.2-0001) be omitted, and an implementation MUST NOT
  fabricate a machine-checkable IR Semantics it did not derive. *Test:*
  `test_contract_completeness_tracks_lift_fraction`.
- **KISS-CONTRACT-6.2-0005** — An input that is malformed per §6.1, carries an inconsistent
  Identity per §6.3-0006, or a Semantics that does not resolve to the primitive floor per
  §6.4-0005, MUST produce a **typed decline, never a panic**; an implementation MUST NOT
  emit a partial or internally-inconsistent contract in place of a decline. *Test:*
  `test_contract_typed_decline_never_panic`.
- **KISS-CONTRACT-6.2-0006** — A contract whose Semantics (§6.4) is a **bare KISS-Ops op DAG**
  — a primitive or mixed-level DAG over KISS-Ops op names with **no** KISS-Grammar
  advertisable-op tag — and whose Identity `op_identity` is therefore the **bare KISS-Ops op
  name** of that DAG's root (§6.3-0003 form (b)) MUST be accepted; an implementation MUST NOT
  reject, withhold, or downgrade such a contract on the ground that it carries no KISS-Grammar
  advertisable tag, and MUST NOT require a KISS-Grammar advertisable-op entry for any kernel. KISS-Grammar is the advertisable/named-op surface only
  (a contract MAY name advertisable ops via §6.3-0003 form (a) and §6.4-0002) and is **not**
  on the mandatory path of every contract; every kernel still MUST carry a contract with the
  universal required core (§6.2-0001), whether or not any of its ops is advertisable. *Test:*
  `test_contract_valid_without_grammar_tag`.

### 6.3 Identity section

- **KISS-CONTRACT-6.3-0001** — The Identity section MUST carry exactly the fields
  `{contract_kind, contract_version, kernel_name, revision_hash, accept_predicate,
  op_identity, target_capability}`, serialized in that order (§6.11-0005); an implementation
  MUST NOT omit any of these fields. *Test:* `test_contract_identity_field_schema`.
- **KISS-CONTRACT-6.3-0002** — The `accept_predicate` MUST be a KISS-Classify
  `structure_key` token — the admissibility predicate over a layout/dtype/target
  specialization cell (a coarse op-category + canonically-ordered operand descriptors +
  target + op-category role hints, extent-free) — carried verbatim as the KISS-Classify
  token; an implementation MUST NOT re-encode, truncate, or reinterpret its bytes, and MUST
  NOT present it as the op's semantic identity. The `structure_key` MUST be a **printable,
  LF-free UTF-8 token** so that it carries verbatim as a single string value (§6.11-0001)
  without re-encoding. *Test:* `test_contract_accept_is_structure_key`.
- **KISS-CONTRACT-6.3-0003** — The `op_identity` MUST be exactly one of two forms, encoded on
  the `op_identity` line per §6.11-0009, and an implementation MUST NOT carry a private op enum
  in `op_identity`: **(a)** a KISS-Grammar advertisable-op tag in its **full** canonical form —
  the bare KISS-Ops op name of the Semantics DAG root's op **plus** the distinguishing
  identity-bearing attribute values that the tag carries (the pattern-channel OpAttrs values and,
  where present, the operand-role tuple — the operand-role tuple is one of the identity-bearing
  attribute values, not a separately-appended element; per §6.3-0004, KISS-GRAMMAR §6.1-0007); or
  **(b)** the **bare KISS-Ops op
  name** of the Semantics DAG root's op (plus that root's identity-bearing OpAttrs values),
  carrying **no** KISS-Grammar advertisable-op entry, for a kernel that has no advertisable-op
  name — the full op DAG lives only in the Semantics section (§6.4) and MUST NOT be duplicated
  into `op_identity`. In form (a) the `op_identity` is the **full tag**, never merely the bare
  op name where distinguishing attributes exist; a bare op name with no distinguishing
  attributes is the special case of form (a) whose tag has none. An implementation MUST NOT
  require a KISS-Grammar advertisable-op entry for form (b) (KISS-Grammar is not required for
  every kernel, §6.2-0006). *Test:* `test_contract_op_identity_full_tag_or_bare_ops_dag`.
- **KISS-CONTRACT-6.3-0004** — When `op_identity` is a KISS-Grammar advertisable-op tag
  (§6.3-0003 form (a)) and more than one advertisable op re-bases onto the same KISS-Ops op
  name distinguished only by identity-bearing attribute values (the pattern-channel OpAttrs
  values and the operand-role tuple, per KISS-GRAMMAR §6.1-0007), the `op_identity` MUST carry
  the **full** advertisable-op tag (the op name **plus** the distinguishing attribute values)
  and MUST NOT be reduced to the bare op name. *Test:*
  `test_contract_op_identity_carries_distinguishing_attrs`.
- **KISS-CONTRACT-6.3-0005** — The `accept_predicate` and the `op_identity` MUST be carried
  as **two distinct fields**, and a consumer MUST match **both** to identify the kernel: the
  `accept_predicate` answers "which cell fits" and the `op_identity` answers "which op is
  this"; an implementation MUST NOT treat either alone as full kernel identity, and MUST NOT
  collapse the two into a single field. *Test:* `test_contract_identity_requires_both`.
- **KISS-CONTRACT-6.3-0006** — The Identity section MUST assert the **consistency** of its
  two identities: the coarse cell-level op-category of the `accept_predicate`
  `structure_key` (a KISS-Classify category) MUST be compatible with the KISS-Ops op family
  of the **op the `op_identity` names** — for form (a) the advertisable-op tag's op name, for
  form (b) the bare root op name (§6.3-0003), which in both forms is the Semantics DAG root's
  op (§6.4-0002); for a **fusion**, this is the family of the fusion **root** op (which carries
  the region's advertised identity), not of an interior node — per the **normative
  compatibility table of §6.3** (below). The op-name→op-family classification is **owned by
  KISS-Ops** (the KISS-Ops op-family classification) and referenced from there, never restated
  in this document; this clause and its table pin only the category↔family compatibility
  **relation**. A reader that detects a pair absent from that table's compatible set MUST
  reject the contract with a typed decline (never silently accept two disagreeing identities).
  *Test:* `test_contract_identity_consistency`.
- **KISS-CONTRACT-6.3-0007** — The `target_capability` MUST be a KISS-Classify namespaced
  `<namespace>:<capability-set>` descriptor, matched **byte-exact** on the full string; an
  implementation MUST NOT apply ordering, subset, or feature-implication logic to it, and
  MUST NOT substitute a hardware-family enumerant for the namespaced token. *Test:*
  `test_contract_target_capability_byte_exact`.
- **KISS-CONTRACT-6.3-0008** — The `revision_hash` MUST be an opaque, provider-assigned
  build identifier compared only for byte-for-byte equality; an implementation MUST NOT
  assume any particular hash algorithm or recomputable input domain from the contract. *Test:*
  `test_contract_revision_hash_opaque`.
- **KISS-CONTRACT-6.3-0009** — The Identity `contract_kind` and `contract_version` MUST
  equal the transport header `contract_kind` and `contract_version` (§6.11-0002)
  byte-for-byte; the header values are authoritative, and a reader MUST reject, with a typed
  decline, a contract whose Identity copies disagree with the header. *Test:*
  `test_contract_identity_header_match`.
- **KISS-CONTRACT-6.3-0010** — The Identity `kernel_name` MUST be the kernel's stable
  identity name; it MAY differ from the Interface `entry_point` (the callable symbol name,
  §6.5), and a reader MUST NOT assume `kernel_name` equals `entry_point`. *Test:*
  `test_contract_kernel_name_distinct`.
- **KISS-CONTRACT-6.3-0011** — The Identity `target_capability` is the **single normative
  authority** for the compilation target. When the `accept_predicate` `structure_key` embeds a
  target component (§6.3-0002), that component MUST equal the `target_capability` byte-for-byte;
  a reader that detects a `structure_key` target component disagreeing with `target_capability`
  MUST reject the contract with a typed decline, and MUST source the target solely from
  `target_capability` (§6.3-0007, §6.5-0003), never from a second target value that could
  disagree. *Test:* `test_contract_structure_key_target_matches_capability`.

#### Normative compatibility table (§6.3-0006)

The Identity consistency assertion (§6.3-0006) is decided **entirely from this table**, which
is the sole point in the suite where the two closed sets meet. The two domains of this relation
are **owned upstream and referenced, not defined here**: the cell op-category tokens are the
**closed KISS-Classify cell op-category set** and the op-family tokens are the **closed KISS-Ops
op-family set**, each spelled verbatim exactly as its owning vocabulary pins it and versioned
there (KISS-Classify / KISS-Ops respectively); on any divergence the upstream set governs, and
a member added upstream reaches this table only through a contract-schema bump that adds a row
(§8-0002). For reference, at this schema version the KISS-Classify cell op-category set is
`{elementwise-unary, elementwise-binary, elementwise-ternary, reduction, scan, contraction,
normalization, softmax, indexing, embedding, shape/layout}` and the KISS-Ops op-family set is
`{arithmetic, rounding, transcendental, binary_math, bitwise, logical, activation, minmax,
select, comparison, reduction, scan, contraction, normalization, gather_scatter, shape}`. The
family checked is the KISS-Ops op family of the op the `op_identity` names (the Semantics DAG
root's op, §6.3-0003 / §6.4-0002); for a **fusion**, that is the family of the fusion **root**
op, not of an interior node — which is why the `contraction`, `normalization`, and `softmax`
cells below admit elementwise **epilogue** families (a matmul+gelu epilogue, §2.3/§2.6, is a
`contraction` cell with an `activation` root). A contract is consistent iff that root family
appears in the compatible set of its `accept_predicate`'s cell op-category below; any other
pair is inconsistent and MUST be rejected with a typed decline.

| KISS-Classify cell op-category | Compatible KISS-Ops op family(ies) |
|---|---|
| `elementwise-unary` | `arithmetic`, `rounding`, `transcendental`, `binary_math`, `bitwise`, `logical`, `activation`, `minmax`, `comparison` |
| `elementwise-binary` | `arithmetic`, `rounding`, `transcendental`, `binary_math`, `bitwise`, `logical`, `activation`, `minmax`, `comparison`, `select` |
| `elementwise-ternary` | `arithmetic`, `binary_math`, `logical`, `activation`, `select`, `comparison` |
| `reduction` | `reduction` |
| `scan` | `scan` |
| `contraction` | `contraction`, `activation`, `arithmetic`, `binary_math`, `minmax`, `rounding` |
| `normalization` | `normalization`, `activation`, `arithmetic`, `binary_math`, `minmax`, `rounding` |
| `softmax` | `normalization`, `activation`, `arithmetic` |
| `indexing` | `gather_scatter` |
| `embedding` | `gather_scatter` |
| `shape/layout` | `shape` |

The elementwise families in the `contraction`, `normalization`, and `softmax` rows admit
fusion **epilogue roots** (the fusion root op that carries the region's advertised identity,
per §6.3-0006); adding a KISS-Ops op family, a KISS-Classify op-category, or a compatible
epilogue family to a row is a contract-schema change (§8) that adds rows and/or set members
without renumbering existing ones.

### 6.4 Semantics section

- **KISS-CONTRACT-6.4-0001** — The Semantics section MUST be **present and mandatory** in
  every contract and MUST carry the ordered field schema `{semantics_kind, op_dag,
  human_annotation?}`, serialized in that order (§6.11-0005): a `semantics_kind` of exactly
  `machine-checkable-IR` or `declared-op-tag`, then an `op_dag`, then an **optional**
  `human_annotation` field pinned **last**. When present, `human_annotation` is a
  non-normative, free-text field outside the exact-byte schema scope of §6.0-0001, which
  KISS-Conform MUST NOT byte-compare; its **absence** is permitted (its line is then omitted)
  and MUST NOT be treated as a missing-required-field decline (§6.1-0006, §6.11-0005). An
  implementation MUST NOT emit a contract without a Semantics section, MUST NOT treat
  `semantics_kind` or `op_dag` as optional, and MUST NOT treat Semantics as a trailing field.
  *Test:* `test_contract_semantics_mandatory`.
- **KISS-CONTRACT-6.4-0002** — The `op_dag` MUST be a hierarchical op DAG whose nodes each
  carry a bare KISS-Ops op name (§6.4-0009) — a node **corresponds to** a KISS-Grammar
  advertisable op when its op name plus identity-bearing `op_attrs` plus operand-role tuple
  reconstruct that advertisable-op tag — at **mixed** abstraction levels; a primitive MUST be a
  one-node DAG and a fusion MUST be its DAG. The DAG **root** MUST equal the op the
  `op_identity` (§6.3-0003) names: for form (a) the full advertisable-op tag (root op name plus
  its identity-bearing attributes and, where the tag carries one, its operand-role tuple), for
  form (b) the bare root op name. A `op_dag` that names **no** KISS-Grammar advertisable op (a
  bare KISS-Ops op DAG, §6.3-0003 form (b)) MUST be permitted — KISS-Grammar is not required.
  An implementation MUST NOT introduce an op vocabulary other than KISS-Ops op names (an
  advertisable op is reconstructed as a tag over those names, never a separate vocabulary).
  *Test:* `test_contract_op_dag_over_ops_and_grammar`.
- **KISS-CONTRACT-6.4-0003** — Each op node MUST carry the KISS-Ops **OpAttrs** channel
  applicable to its op — the axis (for `reduce` / `gather` / `scan`), the out-of-bounds policy
  (the KISS-Ops-owned OOB-policy set, KISS-OPS §6.11-0004, canonically encoded per KISS-OPS
  §6.19-0015), any permutation/`perm` a KISS-Ops layout op defines (KISS-OPS §6.19-0024), and
  the reduce-axis mask/keepdim (the KISS-Ops-owned `reduce_axes` encoding, KISS-OPS
  §6.11-0008/§6.11-0011, canonically encoded per KISS-OPS §6.19-0020) — spelled exactly as the
  node's KISS-Ops op pins it. The OpAttrs sub-vocabularies are **owned by KISS-Ops** as op
  semantics — KISS-Ops is their single normative owner via the KISS-Ops OpAttrs channel
  (KISS-OPS §6.19-0001, §6.10-0004);
  KISS-Contract carries them by reference and MUST NOT define, restate, or re-pin an
  alternative OpAttrs sub-vocabulary, and MUST NOT drop any OpAttrs value the node's KISS-Ops
  op defines as applicable. *Test:* `test_contract_node_carries_opattrs`.
- **KISS-CONTRACT-6.4-0004** — Each op node's per-node edge-case policy (NaN propagation,
  signed-zero behavior, raw-bit move) MUST be **resolved from** the KISS-Ops semantics of
  that op and MUST NOT be restated or overridden by KISS-Contract; where a contract records
  an edge-case policy field, it MUST equal the policy the node's KISS-Ops op pins. *Test:*
  `test_contract_edge_case_from_ops`.
- **KISS-CONTRACT-6.4-0005** — Every non-primitive op node in the `op_dag` MUST be
  **resolvable** by acquiring that op's contract (via the KISS-Announce contract-query, §4)
  and reading its KISS-Ops reference decomposition into strictly-lower-level ops,
  recursively, until every chain bottoms out at the KISS-Ops primitive floor; the
  termination guarantee (acyclic, strictly-decreasing level) is inherited from KISS-Ops and
  MUST NOT be weakened. *Test:* `test_contract_semantics_resolves_to_floor`.
- **KISS-CONTRACT-6.4-0006** — The fully-lowered primitive form produced on demand by
  resolving the `op_dag` (§6.4-0005) MUST be the **verification oracle** for the kernel under
  the op's determinism class (§6.8); an implementation MUST NOT claim a Semantics whose
  resolved primitive form disagrees with the kernel's computed result beyond the declared
  determinism class. *Test:* `test_contract_lowered_form_is_oracle`.
- **KISS-CONTRACT-6.4-0007** — When `semantics_kind = machine-checkable-IR`, the `op_dag`
  MUST be a complete machine-checkable IR of the kernel's computation (a generated or fully
  lifted kernel); when `semantics_kind = declared-op-tag`, the Semantics MUST carry the
  declared op-identity (a KISS-Grammar advertisable-op tag or a bare KISS-Ops op name,
  §6.3-0003) plus a `lift_residue` recording the un-liftable remainder per the
  KISS-Consume refusal taxonomy (`not-a-kernel` / `wrong-op-class` /
  `unrecognized-but-expressible` / `inexpressible-residue`); an implementation MUST NOT
  declare `machine-checkable-IR` for a kernel with un-lifted residue. *Test:*
  `test_contract_semantics_kind_matches_lift`.
- **KISS-CONTRACT-6.4-0008** — A KISS-Grammar advertisable **region** MUST map onto the
  Semantics `op_dag` structure-for-structure: a one-node region maps to a one-node DAG, a
  fusion maps to its DAG, and each region node's OpAttrs MUST appear on the corresponding DAG
  node; a region node's per-node **operand-role tuple** is **not** carried on the DAG node (only
  the Semantics DAG root op's operand-role tuple is rendered, on the `op_identity` line, §6.4-0009
  / §6.11-0009), and an interior node's operand roles are reconstructed per §6.4-0009. An
  implementation MUST NOT reshape a region's DAG structure when carrying it as
  Semantics. *Test:* `test_contract_region_maps_to_semantics`.
- **KISS-CONTRACT-6.4-0009** — Each `op_dag` node MUST use the node schema
  `Op{op_name, op_attrs, child_edges} | Bind(positional_index)`: an `Op` node carries a **bare
  KISS-Ops op name** (`op_name` — always the bare op name, never a KISS-Grammar tag spelled
  inside `op_name`; identity-bearing attributes ride `op_attrs`), its per-node OpAttrs
  (`op_attrs`, §6.4-0003), and ordered child edges to its operand sub-DAGs (`child_edges`, in
  operand order); a `Bind` node carries the positional operand index it binds. A node
  **corresponds to** a KISS-Grammar advertisable op when its (`op_name` + identity-bearing
  `op_attrs` + operand-role tuple) reconstructs that advertisable-op tag; the tag is a
  reconstruction over these fields, not a separate spelling. This node schema is a
  **projection** of the KISS-Grammar region node grammar (`Op { op_name, pattern_attrs,
  operand_role_tuple, operands, consumers } | Bind(input_index)`) — the Contract-local
  `op_attrs` carries the Grammar `pattern_attrs` (the per-node OpAttrs channel) and
  `child_edges` carries the Grammar `operands`. The `op_dag` node does **not** carry a per-node
  `operand_role_tuple`: only the Semantics DAG **root** op's operand-role tuple is rendered — on
  the `op_identity` line (§6.11-0009, form (a)) — and an **interior** node's operand roles are
  reconstructed from that node's KISS-Ops op definition (the per-op operand roles owned by
  KISS-Ops, §6.6-0005) together with the Interface per-operand dtypes (§6.5); that reconstruction
  is the defined source when §6.4-0002 reconstructs an interior node's advertisable-op tag from
  its `op_name` + identity-bearing `op_attrs` + operand-role tuple. Grammar's per-node
  `consumers` is likewise **not** carried as a node field; it is **subsumed** by the fixed
  root-last node ordering of the `op_dag` array (§6.11-0007) — from which each node's consumers
  are recoverable — rather than dropped silently. Grammar's `extract` is a region-level
  field (KISS-GRAMMAR-6.4-0007), **not** a node field, and so appears in neither schema; an
  implementation MUST NOT introduce a node kind or node field outside this schema. *Test:*
  `test_contract_op_dag_node_schema`.
- **KISS-CONTRACT-6.4-0010** — A resolver MUST distinguish two outcomes for a non-primitive
  node: (a) a node whose op has **no** reference decomposition reaching the KISS-Ops
  primitive floor MUST yield a **typed decline** (the Semantics does not resolve, §6.2-0005);
  (b) a node whose op's contract is merely **unavailable** to the resolver MUST yield a
  distinct typed **unresolved/deferred** outcome, neither a decline nor an accept. An
  implementation MUST NOT conflate an unavailable decomposition with a genuinely
  non-resolvable op. *Test:* `test_contract_semantics_resolution_outcomes`.
- **KISS-CONTRACT-6.4-0011** — The Interface (§6.5) declared output shape — its `rank`
  and per-output extents — MUST equal the Semantics op's KISS-Ops **shape rule** (KISS-OPS
  §6.20) evaluated over the operand shapes; a contract whose declared output shape
  disagrees with its op's shape rule MUST be a **typed decline** (§6.2-0005). This is the
  **shape-side companion** to the §6.4-0006 value oracle: §6.4-0006 makes the fully-lowered
  primitive form the oracle for the kernel's *values*, and this clause makes the op's shape
  rule the oracle for the kernel's *output shape* — closing the return-contract shape check
  that had no evaluator before. Where the shape rule resolves to a **surfaced gap** (a
  symbolic / data-dependent output, KISS-OPS §6.20-0004), a consumer MUST NOT treat the gap
  as an inconsistency. An implementation MUST NOT accept a contract whose Interface output
  shape is inconsistent with its Semantics op's shape rule. *Test:*
  `test_contract_output_shape_consistency`.

### 6.5 Interface section (ABI)

The Interface section is the positional ABI. The concrete signature is the operand pointers
in KISS-Classify canonical operand order (`in0..in{k}`, then the output(s)); one typed
pointer per operand at its dtype (a gather index operand at its own dtype), **followed by**
the runtime launch scalars in the single pinned order of §6.5-0004a.

- **KISS-CONTRACT-6.5-0001** — The Interface section MUST carry exactly the fields
  `{entry_point, rank, positional_signature, launch_scalars, count_unit, in_place,
  alignment_bytes}`, serialized in that order (§6.11-0005); an implementation MUST NOT omit
  any of these fields, and MUST NOT carry `count_unit`, `in_place`, or `alignment_bytes`
  outside the Interface section (they MUST NOT live in the Capabilities section). The Interface
  section MUST NOT carry an independent `target` field; the compilation target is the Identity
  `target_capability` (§6.3-0007, §6.5-0003). *Test:* `test_contract_interface_field_schema`.
- **KISS-CONTRACT-6.5-0002** — The `positional_signature` MUST be an ordered, typed argument
  list from which a consumer can bind and launch the kernel with no out-of-band information:
  the operand pointers MUST appear first, in the KISS-Classify **canonical operand order**
  (`in0..in{k}`, then output(s)), each with a declared pointer type at the operand's dtype;
  the runtime launch scalars MUST follow, in the pinned order of §6.5-0004a. An
  implementation MUST NOT require a consumer to source any argument from outside the
  contract. *Test:* `test_contract_positional_signature_complete`.
- **KISS-CONTRACT-6.5-0003** — The Interface section MUST select the kernel's entry point by
  **referencing** the Identity `target_capability` (§6.3-0007) — the **single normative
  source** of the compilation target — and MUST NOT re-pin, copy, or carry an independent
  target value that could disagree with Identity. An implementation MUST source the target for
  entry-point selection from the Identity `target_capability`, never from a duplicate Interface
  field. *Test:* `test_contract_interface_references_identity_target`.
- **KISS-CONTRACT-6.5-0004a** — The canonical **order** of the runtime launch scalars is
  pinned by **this normative KISS-Contract Interface clause** (dispatch is a Contract section,
  §6.6, not a separate sub-standard): the runtime launch scalars MUST be declared in the
  single, **pinned, normative order** of the launch-scalar class table below (classes 1–8) —
  per-operand logical extents, then per-operand signed strides, then the iteration count `n`,
  then per-operand base offsets, then gather/index extents (with the index-operand
  descriptor), then the workspace pointer, then the workspace byte size, then the scalar op
  params — each class present exactly when its "present when" predicate holds; an
  implementation MUST NOT introduce a launch-scalar class outside this table and MUST NOT
  reorder the classes. *Test:* `test_contract_launch_scalar_pinned_order`.

  | # | Class | Symbol(s) | Type | Present when |
  |---|---|---|---|---|
  | 1 | logical extents | `extents{i}[rank]` | i64, element units | always (per operand) |
  | 2 | signed strides | `strides{i}[rank]` | i64, element units (`0` = broadcast, `< 0` = reversed) | cell not fully packed |
  | 3 | iteration count | `n` | `i32`/`i64` per the cell's index width `idx32`/`idx64` | always |
  | 4 | base offsets | `off{i}` | i64 | sliced/offset kernels |
  | 5 | gather/index extents + index-operand descriptor | index descriptor, then `idx_extents{i}` | descriptor / i64 | data-dependent reads |
  | 6 | workspace pointer | `ws_ptr` | pointer | kernels needing scratch |
  | 7 | workspace byte size | `ws_bytes` | i64 | present with the workspace pointer |
  | 8 | scalar op params | `param{i}` | the op's scalar compute dtype | op has scalar params |

- **KISS-CONTRACT-6.5-0004b** — Each present launch scalar MUST carry a declared type and a
  declared position in the `positional_signature`; an implementation MUST NOT emit a launch
  scalar without a declared type or without a declared position. *Test:*
  `test_contract_launch_scalar_typed_positioned`.
- **KISS-CONTRACT-6.5-0004c** — An implementation MUST NOT reorder the launch-scalar classes
  of §6.5-0004a or place a launch scalar outside its pinned class position. This pinned
  ordering is owned by this Interface clause; the Dispatch section (§6.6) consumes it and
  does not re-pin it. *Test:* `test_contract_launch_scalar_no_reorder`.
- **KISS-CONTRACT-6.5-0004d** — Within each launch-scalar class, the per-operand entries
  MUST appear **grouped** (all of a class's per-operand entries together — e.g. all extents
  as class 1, then all strides as class 2), in KISS-Classify **canonical operand order**; an
  implementation MUST NOT interleave per-operand entries across classes. **Within class 5**
  (gather/index extents + index-operand descriptor), for each index operand in canonical
  operand order the **index-operand descriptor** MUST precede that operand's `idx_extents{i}`
  scalars; an implementation MUST NOT emit the `idx_extents{i}` before the index operand's
  descriptor. *Test:* `test_contract_launch_scalar_within_class_order`.
- **KISS-CONTRACT-6.5-0005** — A cell that is **not fully packed** — where "fully packed"
  means the KISS-Classify contiguous-layout predicate marks every operand contiguous in
  canonical order with unit innermost stride — MUST carry the per-operand signed
  `strides{i}[rank]` launch scalars (§6.5-0004a class 2); an implementation MUST NOT omit the
  strides for a non-packed cell, and MUST NOT emit an Interface whose signature cannot
  address the cell's layout. *Test:* `test_contract_strided_cell_carries_strides`.
- **KISS-CONTRACT-6.5-0006** — A sliced/offset kernel MUST carry the per-operand base-offset
  launch scalars `off{i}` (§6.5-0004a class 4); an implementation MUST NOT advertise an entry
  point whose ABI requires base offsets while omitting the corresponding launch scalars from
  the signature. *Test:* `test_contract_offset_kernel_carries_offsets`.
- **KISS-CONTRACT-6.5-0007** — A data-dependent gather/scatter/index kernel MUST carry, in
  the `positional_signature`, the index operand's own typed pointer at its KISS-Classify dtype
  (e.g. an unsigned-32 index pointer), and MUST carry the gather/index extents and
  index-operand descriptor launch scalars (§6.5-0004a class 5); an implementation MUST NOT
  infer the index operand's dtype from operand-0's dtype. *Test:* `test_contract_index_operand_typed`.
- **KISS-CONTRACT-6.5-0008** — The `count_unit` MUST be exactly one of `{elements,
  vectors_x<w>}`, where the `vectors_x<w>` spelling is the literal `vectors_x` immediately
  followed by the decimal vector width with no delimiter (for example `vectors_x4`); the
  count `n` in the signature MUST be interpreted in this `count_unit`, and an implementation
  MUST NOT leave `count_unit` unspecified where a vectorized/packed cell counts `w`-element
  vectors. *Test:* `test_contract_count_unit_enum`.
- **KISS-CONTRACT-6.5-0009** — The `in_place` field MUST state either that the output aliases
  a named input operand (identifying which input) or that it aliases none; an implementation
  MUST NOT declare `in_place` aliasing an input for a kernel whose output pointer does not in
  fact alias that input. *Test:* `test_contract_in_place_declared`.
- **KISS-CONTRACT-6.5-0010** — The `alignment_bytes` field MUST state the base-pointer
  alignment (in bytes) the kernel's ABI requires; an implementation MUST NOT omit it. *Test:*
  `test_contract_alignment_declared`.
- **KISS-CONTRACT-6.5-0011** — The `launch_scalars` field MUST be a typed sub-view equal,
  entry-for-entry, to the launch-scalar tail of the `positional_signature` (the entries
  following the operand pointers); the `positional_signature` is the authoritative ordered
  argument list, and an implementation MUST NOT carry a `launch_scalars` list that disagrees
  with that tail. *Test:* `test_contract_launch_scalars_match_signature_tail`.
- **KISS-CONTRACT-6.5-0012** — The Interface `rank` field MUST declare the operand logical
  rank as a fixed, per-contract compile-time constant (a non-negative integer); every
  per-operand `extents` / `strides` array length MUST equal `rank`, and a consumer MUST size
  those arrays from `rank` alone. The per-operand base offset `off{i}` (§6.5-0004a class 4)
  is a **single scalar** value per operand — one linear base offset, not a rank-length array —
  and a consumer MUST NOT size it from `rank`. An implementation MUST NOT require a consumer to
  recover `rank` by parsing the `accept_predicate` bytes (§6.3-0002 forbids reinterpreting
  them). *Test:* `test_contract_rank_declared`.
- **KISS-CONTRACT-6.5-0013** — The Interface MUST carry an explicit scalar-op-param **count**
  and a per-param **dtype list** fixing the number and order of the `param{i}` launch scalars
  (§6.5-0004a class 8), and MUST carry, for a data-dependent read, an **index-operand
  descriptor** fixing the index operand's dtype, its rank, and its gather/index-extent
  binding (§6.5-0004a class 5); an implementation MUST NOT leave scalar-param count/order or
  the index-descriptor fields to out-of-band knowledge. *Test:*
  `test_contract_param_count_and_index_descriptor`.

### 6.6 Dispatch section

- **KISS-CONTRACT-6.6-0001** — The Dispatch section is **optional** (§6.6-0007). A kernel that
  **declares** launch geometry MUST carry the launch model as exactly the fields
  `{invocation_domain, workgroup_sizing, count_to_grid, thread_mapping, addressing_rule}`,
  serialized in that order (§6.11-0005), and MUST NOT declare that geometry "provider-internal"
  or omit any of these fields, since a consumer launches the kernel from this section; but a
  **geometry-agnostic** kernel (§6.6-0007) MAY instead declare **no** Dispatch section (an
  explicit absent sentinel), and a consumer MUST NOT require the geometry fields for it. *Test:*
  `test_contract_dispatch_field_schema`.
- **KISS-CONTRACT-6.6-0002** — The `invocation_domain` MUST declare the iteration/index frame
  derived from the operand extents (the widest-rank iteration frame) as a machine-evaluable
  expression (§6.6-0006) over the launch-scalar symbols; an implementation MUST NOT declare an
  index domain a consumer cannot reconstruct from the Interface extents. *Test:*
  `test_contract_invocation_domain`.
- **KISS-CONTRACT-6.6-0003** — The `count_to_grid` derivation MUST be a machine-evaluable
  expression (§6.6-0006) stating how the count `n`, interpreted in its `count_unit`
  (§6.5-0008), maps to grid size; the expression MUST divide the element count by `w` for a
  `vectors_xw` count before deriving the grid, and an implementation MUST NOT state a grid
  derivation that ignores the `count_unit`. *Test:* `test_contract_count_to_grid`.
- **KISS-CONTRACT-6.6-0004** — The `thread_mapping` and `addressing_rule` are **thread-index-free
  structural model declarations** (§6.6-0006): the §6.6-0006 grammar carries no thread/lane index
  symbol (by design — §6.6-0008), so these fields declare only the parameters expressible without
  one. `thread_mapping` MUST declare the **grid-stride constant** — the stride, in elements,
  between successive elements one thread handles (for a standard grid-stride launch, the total
  thread count) — and `addressing_rule` MUST declare the per-operand class-2 **signed-stride** and
  class-4 **base-offset** coefficients (§6.5-0004a, subscripted per axis via the §6.6-0006 `sym[k]`
  operator) that scale a thread's element index into an address; an implementation MUST NOT omit
  the signed-stride / base-offset coefficients for a non-packed or offset cell. The **per-thread
  element index itself** is the normative **grid-stride semantic** — the element handled by thread
  `t` on iteration `k` is `t + k · thread_mapping`, applied by the executor/driver — and is **not**
  a declared expression (there is no per-thread index symbol to spell it; that surface is reserved
  to §6.6-0008, post-v1). *Test:* `test_contract_thread_and_addressing`.
- **KISS-CONTRACT-6.6-0005** — The Dispatch section MUST be consistent with the Interface
  section: the launch scalars the addressing and grid derivations reference MUST be exactly
  those declared in the Interface `positional_signature` (§6.5), and an implementation MUST
  NOT reference in Dispatch a launch scalar absent from the Interface. *Test:*
  `test_contract_dispatch_interface_consistent`.
- **KISS-CONTRACT-6.6-0006** — Each Dispatch derivation field (`invocation_domain`,
  `workgroup_sizing`, `count_to_grid`, `thread_mapping`, `addressing_rule`) MUST be a
  machine-evaluable expression, not free prose, in the pinned expression grammar: an
  expression is an ASCII string over the launch-scalar symbol vocabulary (§6.5-0004a /
  §6.7-0006), non-negative decimal integer literals, the binary operators `+ - * /`, the
  function `ceil_div(a, b)`, parentheses, and the **element-subscript operator** `sym[k]`,
  where `sym` is one of the rank-length array launch-scalar symbols (§6.5-0004a — exactly the
  symbols whose `array_len_kind` is `rank`, §6.11-0006: `extents{i}`, `strides{i}`,
  `idx_extents{i}`) and `k` is a non-negative decimal integer literal naming an axis; `sym[k]`
  is valid iff `0 <= k < rank`, where `rank` is the Interface `rank` for `extents{i}`/`strides{i}`
  and the index-operand rank for `idx_extents{i}`. Subscript binds tighter than `*` and `/`,
  which in turn bind tighter than `+` and `-`; operators of equal precedence are
  left-associative. Evaluating an `invocation_domain`,
  `workgroup_sizing`, or `count_to_grid` expression against concrete launch-scalar values MUST
  yield a non-negative integer (or a fixed-length tuple of such, for a multi-dimensional
  workgroup/grid); `thread_mapping` and `addressing_rule` are structural model declarations
  (per KISS-Emit §6.3-0003 the concrete signed-stride index arithmetic renders no target
  surface and is driver-owned) and are NOT subject to the non-negative-integer post-condition,
  since class-2 strides may be negative. An implementation MUST NOT carry a Dispatch derivation
  field the grammar does not accept, MUST NOT reference a symbol absent from the launch-scalar
  vocabulary, MUST NOT subscript a scalar symbol or use a rank-length array symbol without a
  subscript, and MUST NOT use an out-of-bounds subscript; each such case is a typed decline.
  *Test:* `test_contract_dispatch_expressions_machine_evaluable`.
- **KISS-CONTRACT-6.6-0007** — A **geometry-agnostic** kernel — one launched grid-stride from a
  host-computed launch dimension (`Dim3`) whose element→thread mapping is not consumer-visible —
  MAY declare **no** Dispatch section: the section is carried as an explicit **absent sentinel**,
  and KISS-Conform MUST accept a contract with an absent Dispatch section and MUST NOT require any
  of the five geometry fields for it. Launch geometry is then the **executor's**, not the
  contract's — the consumer launches the kernel with its own `Dim3` under the grid-stride semantic
  (§6.6-0004), needing no declared geometry (this is KISS's answer to the "a kernel contract should
  not declare launch geometry" objection, PRIOR-ART §5.1). A contract MUST NOT carry a **partial**
  Dispatch section: it declares either all five fields (§6.6-0001) or the absent sentinel, never a
  subset. *Test:* `test_contract_dispatch_optional`.
- **KISS-CONTRACT-6.6-0008** — KISS-Contract **reserves** — as a **named post-v1 extension**, NOT
  required for v1 conformance — a **richer thread-mapping form** for launches whose element→thread
  mapping is neither grid-stride nor thread-index-free (e.g. a pinned-tile / warp-lane→output-tile
  schedule, as in a tiled tensor-core GEMM): either a named **tile-mapping** rule or a bound
  per-thread index symbol (`gid`) added to the §6.6-0006 grammar. A v1 contract MUST express its
  launch as either a geometry-agnostic kernel (§6.6-0007) or a grid-stride Dispatch section with
  thread-index-free coefficients (§6.6-0004); KISS-Conform MUST NOT require the reserved form in v1,
  MUST NOT reject a v1 kernel for not carrying it, and the §6.6-0006 grammar MUST remain
  **thread-index-free** in v1 (no per-thread index symbol). *Test:*
  `test_contract_reserved_tile_mapping`.

### 6.7 Capabilities section

- **KISS-CONTRACT-6.7-0001** — The Capabilities section MUST carry exactly the fields
  `{accept_predicate, supported_dtype_set, awkward_layout_strategy, in_place_eligible_variants,
  index_width, determinism_class, precision_class, cost}`, serialized in that order
  (§6.11-0005); an implementation MUST NOT omit any of these fields. *Test:*
  `test_contract_capabilities_field_schema`.
- **KISS-CONTRACT-6.7-0002** — The Capabilities `accept_predicate` MUST be the same
  KISS-Classify `structure_key` the Identity section carries (§6.3-0002) — the per-kernel
  accept-predicate — carried byte-for-byte identically; an implementation MUST NOT carry a
  Capabilities accept-predicate that disagrees with the Identity accept-predicate. *Test:*
  `test_contract_capabilities_accept_matches_identity`.
- **KISS-CONTRACT-6.7-0003** — The Capabilities section MUST declare the per-kernel
  **envelope** (the family of invocations the kernel can serve) and MUST NOT be a per-call
  signature: `supported_dtype_set` MUST use KISS-Classify dtype tokens, `index_width` MUST be
  one of the KISS-Classify `{idx32, idx64}` codes, and `awkward_layout_strategy` and
  `in_place_eligible_variants` MUST describe the kernel's declared layout/aliasing family; an
  implementation MUST NOT place a per-call argument (an extent, a stride, an offset, `n`) in
  the Capabilities section. *Test:* `test_contract_capabilities_is_envelope`.
- **KISS-CONTRACT-6.7-0004** — The Capabilities `determinism_class` MUST be drawn from the
  single canonical KISS-Ops enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}` (KISS-OPS §6.0-0001), spelled verbatim, and MUST equal
  the Guarantees `determinism_class` (§6.8-0003); an implementation MUST NOT define a parallel
  determinism vocabulary. *Test:* `test_contract_capabilities_determinism_from_ops`.
- **KISS-CONTRACT-6.7-0005** — The Capabilities `precision_class` MUST be drawn from the
  **closed precision-class token set imported verbatim from KISS-Ops** and MUST be consistent
  with the Guarantees per-backend accuracy tiers (§6.8-0002) per the KISS-Ops
  precision-class↔tier correspondence (KISS-OPS §6.8): the `correctly-rounded` and
  `bit-reproducible` classes MUST map to tier 0 (a definitional floor, not a retired
  ceiling), and every looser class MUST carry a declared tier consistent with the
  precision-class ordering (a looser class MUST NOT declare a tighter tier than a stricter
  class for the same reference). An implementation MUST NOT define a
  KISS-Contract-local precision-class vocabulary and MUST NOT declare a precision class the
  Guarantees ULP tiers contradict. *Test:* `test_contract_precision_class_consistent`.
- **KISS-CONTRACT-6.7-0006** — The Capabilities `cost` MUST carry a cost **class** plus cost
  **expressions** over the launch-scalar symbol vocabulary (for example a coefficient times
  `n`), spelled as expressions over the Interface launch-scalar symbols (§6.5) in the grammar
  of §6.6-0006; the Capabilities `cost` is the **single authoritative home** for cost
  magnitude (§6.8-0007). An implementation MUST NOT carry a bare per-element scalar in place
  of a shape-parameterized cost expression, and MUST NOT reference a symbol absent from the
  launch-scalar vocabulary. *Test:* `test_contract_cost_expressions`.

### 6.8 Guarantees section

- **KISS-CONTRACT-6.8-0001** — The Guarantees section MUST carry exactly the fields
  `{reference_function, per_backend_ulp_tiers, determinism_class, math_precision,
  bit_stability, audited_status, cost_provenance}`, serialized in that order (§6.11-0005); an
  implementation MUST NOT omit any of these fields, and MUST NOT scatter these facts across
  other sections. The `audited_status` field is a **derived** trust field whose single
  normative home is this Guarantees section (§6.8-0008/-0009/-0010); it MUST NOT be carried in
  the Provenance section (§6.9-0001). *Test:* `test_contract_guarantees_field_schema`.
- **KISS-CONTRACT-6.8-0002** — Precision MUST **name its reference function**: the
  `reference_function` MUST be the NAMED KISS-Ops function the kernel's precision is measured
  against, and the `per_backend_ulp_tiers` MUST declare, per target backend, the kernel's
  **accuracy tier** — a tagged quantity carrying at least one of `{max_ulp, max_relative,
  max_absolute}` against that reference. The declared per-target tier is the **sole** accuracy
  gate (KISS-OPS §6.8-0001): an implementation MUST NOT declare a precision bound without naming
  its reference, but KISS-Contract MUST NOT constrain a declared tier against a fixed KISS-Ops
  ceiling — the §6.8 table is an *informative advisory floor*, not a normative cap, so a
  truthful tier looser than a table value MUST NOT be rejected. *Test:*
  `test_contract_precision_names_reference`.
- **KISS-CONTRACT-6.8-0003** — The `determinism_class` MUST be the single canonical KISS-Ops
  enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` (KISS-OPS §6.0-0001),
  spelled verbatim and imported from KISS-Ops; an implementation MUST NOT re-fork or re-spell
  the enum. *Test:* `test_contract_determinism_class_imported`.
- **KISS-CONTRACT-6.8-0004** — The `math_precision` MUST be the KISS-Ops MathPrecision
  attribute `{bit-stable, reduced-mantissa-permitted}` (KISS-OPS §6.17), imported from
  KISS-Ops and orthogonal to the determinism class; an implementation MUST NOT re-fork it,
  MUST NOT model it as a dtype, and MUST NOT infer it from a KISS-Classify dtype token. *Test:*
  `test_contract_math_precision_imported`.
- **KISS-CONTRACT-6.8-0005** — The `bit_stability` field MUST state whether the kernel is
  bit-stable on the same hardware and is the **single authoritative home** of that fact; it
  MUST be consistent with the `determinism_class`, and in particular a kernel of class
  `order-invariant/nondeterministic` MUST carry `bit_stability = false`. The `audited_status`
  derivation (§6.8-0009 / §6.8-0010) reads this field and MUST NOT set it. *Test:*
  `test_contract_bit_stability_consistent`.
- **KISS-CONTRACT-6.8-0006** — The `cost_provenance` MUST be `declared` or `measured`, stating
  the provenance of the Capabilities cost model (§6.7-0006); an implementation MUST NOT omit it
  where a cost is stated, and MUST NOT restate the cost class or cost expressions in the
  Guarantees section (those live solely in Capabilities, §6.8-0007). *Test:*
  `test_contract_guarantees_cost_provenance`.
- **KISS-CONTRACT-6.8-0007** — The Capabilities cost model (cost class + cost expressions,
  §6.7-0006) MUST be the single authoritative home for cost magnitude; the Guarantees section
  MUST carry only the cost provenance (`cost_provenance`, §6.8-0006), and the Provenance
  `cost_provenance` (§6.9-0006) MUST equal it. An implementation MUST NOT duplicate the cost
  class or cost expressions in the Guarantees or Provenance sections. *Test:*
  `test_contract_cost_single_home`.
- **KISS-CONTRACT-6.8-0008** — The `audited_status` MUST be **derived** from the Guarantees
  `determinism_class` and precision (the `reference_function` and `per_backend_ulp_tiers`) per
  the normative derivation rule of §6.8-0009 and §6.8-0010, and MUST NOT be an authored
  constant; an implementation MUST NOT hardcode `audited_status` independently of the
  Guarantees fields it is derived from. The Guarantees section is the **single home** of both
  the `audited_status` value and its derivation rule; KISS-Conform verifies the derivation.
  *Test:* `test_contract_audited_status_derived`.
- **KISS-CONTRACT-6.8-0009** — The `audited_status` derivation MUST yield `audited` for a
  kernel whose Guarantees declare a bounded precision against a named `reference_function`
  under its `determinism_class` — a per-backend ULP tier (or bit-reproducible /
  correctly-rounded precision), **including** an `order-invariant/nondeterministic` kernel
  whose nondeterminism is declared against a named reference under a stated tolerance. An
  implementation MUST derive this value by applying the rule to the Guarantees section, MUST
  NOT author `audited` where the rule does not yield it, and MUST NOT set `bit_stability` from
  this rule (that field is owned by §6.8-0005). *Test:* `test_contract_audited_derivation_rule`.
- **KISS-CONTRACT-6.8-0010** — The `audited_status` derivation MUST yield `unaudited` for a
  kernel whose Guarantees do **not** declare a bounded precision against a named
  `reference_function`; an implementation MUST NOT author `unaudited` where the Guarantees do
  declare such a bound, and MUST NOT produce an `audited_status` value that neither §6.8-0009
  nor this clause yields. *Test:* `test_contract_unaudited_derivation_rule`.

### 6.9 Provenance section

- **KISS-CONTRACT-6.9-0001** — The Provenance section MUST carry exactly the fields
  `{kernel_source, revision_base, revision_hash, cost_provenance, negotiation_metadata}`,
  serialized in that order (§6.11-0005); an implementation MUST NOT omit any of these fields,
  and MUST NOT carry `audited_status` in the Provenance section (it is a derived Guarantees
  field, §6.8-0001/-0008). The `negotiation_metadata` field is an opaque length-delimited blob
  (§6.9-0008). *Test:* `test_contract_provenance_field_schema`.
- **KISS-CONTRACT-6.9-0002** — The `kernel_source` MUST record how the kernel was produced
  (for example generated, lifted, or hand-written) and its origin; an implementation MUST NOT
  fabricate an origin it cannot substantiate. *Test:* `test_contract_provenance_source`.
- **KISS-CONTRACT-6.9-0003** — The Provenance `revision_hash` MUST equal the Identity
  `revision_hash` (§6.3-0008) byte-for-byte, and the `revision_base` MUST record the source
  revision the kernel was built from; an implementation MUST NOT carry a Provenance
  `revision_hash` that disagrees with the Identity `revision_hash`. *Test:*
  `test_contract_provenance_revision_matches_identity`.
- **KISS-CONTRACT-6.9-0006** — The `cost_provenance` MUST be `declared` or `measured` and MUST
  equal the Guarantees `cost_provenance` (§6.8-0006); an implementation MUST NOT carry a
  Provenance cost provenance that disagrees with the Guarantees. *Test:*
  `test_contract_cost_provenance_consistent`.
- **KISS-CONTRACT-6.9-0008** — The `negotiation_metadata` field MUST be an opaque,
  length-delimited byte blob (a declared byte-length followed by that many bytes in the
  §6.11-0001 opaque-blob text encoding) carrying provider-to-consumer negotiation hints; it MAY
  be empty (zero length). A reader MUST NOT interpret its bytes as contract semantics and MUST
  NOT reject a contract on its content (only a missing field or a malformed length is a decline,
  §6.1-0006). *Test:* `test_contract_negotiation_metadata_opaque`.

### 6.10 The structural seams onto the foundational vocabularies

- **KISS-CONTRACT-6.10-0001** — KISS-Contract MUST confine any citation of KISS-Grammar and
  KISS-Ops to exactly the two sections Identity and Semantics, both STRUCTURAL edges: the
  Identity `op_identity` is either a KISS-Grammar advertisable-op tag re-based onto a KISS-Ops
  op name **or** the bare KISS-Ops op name of the Semantics DAG root (§6.3-0003), and the
  Semantics `op_dag` nodes carry bare KISS-Ops op names (an advertisable op is reconstructed as
  a tag over the node's `op_name` + `op_attrs` + operand-role tuple, §6.4-0009), carrying the
  KISS-Ops OpAttrs channel (§6.4-0002, §6.4-0003). KISS-Ops is on the mandatory
  path; KISS-Grammar citation is present only when a contract names an advertisable op and MUST
  NOT be required otherwise (§6.2-0006). An implementation MUST NOT introduce a private op
  vocabulary in any section. *Test:* `test_contract_grammar_ops_seam_two_sections`.
- **KISS-CONTRACT-6.10-0002** — Per-node NaN / signed-zero / edge-case behavior and the
  per-transcendental declared-ULP ceiling MUST be resolved **from** the KISS-Ops op semantics
  and MUST NOT be restated in the contract; where a contract records such a value, it MUST
  equal the value the node's KISS-Ops op pins (§6.4-0004, §6.8-0002). *Test:*
  `test_contract_edge_case_and_ulp_from_ops`.
- **KISS-CONTRACT-6.10-0003** — Because KISS-Grammar re-bases every advertisable op onto a
  KISS-Ops op name, a consumer MUST be able to reconcile the advertisable surface and the op
  meaning **without a second op vocabulary**: KISS-Grammar supplies the advertisable surface
  and attribute channels, KISS-Ops supplies the meaning and the primitive floor, and an
  implementation MUST NOT require a consumer to reconcile two independent op vocabularies to
  read a contract. *Test:* `test_contract_single_op_vocabulary`.
- **KISS-CONTRACT-6.10-0004** — The OpAttrs sub-vocabularies — the out-of-bounds policy set
  (KISS-OPS §6.19-0015), the permutation/`perm` encoding (KISS-OPS §6.19-0024), and the
  reduce-axis mask/keepdim encoding (KISS-OPS §6.19-0020) — are **owned by KISS-Ops** as op
  semantics, whose **single normative owner** is the KISS-Ops OpAttrs channel (KISS-OPS
  §6.19-0001); KISS-Contract's Semantics OpAttrs channel (§6.4-0003) MUST cite KISS-OPS
  §6.19-0001 as their single owner and MUST NOT define, restate, or re-pin a
  KISS-Contract-local OpAttrs sub-vocabulary, and where KISS-Grammar surfaces these values
  through its pattern-attribute channel it likewise cites KISS-Ops (the KISS-Grammar
  `pattern_attrs` matching hints are a **distinct** channel from the KISS-Ops-owned OpAttrs
  semantics). An implementation MUST carry an OpAttrs value only as the node's KISS-Ops op pins
  it. The KISS-Ops canonical little-endian OpAttrs **wire encoding** (KISS-OPS §6.19-0010) and
  the requirement to embed that encoded blob **opaquely** and byte-compare it without parsing
  inside (KISS-OPS §6.19-0012) bind **KISS-Grammar**, whose binary region carries the blob;
  KISS-Contract is a structured/text document (§6.1-0001) and does **not** embed that binary
  blob — it carries the **same** OpAttrs values by their KISS-Ops-pinned **spelling** in its
  text document (§6.4-0003, §6.11-0007), so that Grammar embeds the encoded blob opaquely while
  Contract carries the values by KISS-Ops spelling. *Test:*
  `test_contract_opattrs_cited_from_ops`.

### 6.11 Document framing — pinned structured/text serialization

This section pins the bytes of a contract **structured/text document** so two structurally
dissimilar implementations serialize and parse the identical byte string. A contract document is
a UTF-8 text document — a pinned header line followed by seven section blocks under pinned
heading lines — **not** a length-prefixed binary envelope; it is self-delimited by the inner
length its header line declares and integrity-checked by the header line's checksum, and it
**fails loudly** (a typed decline) on a malformed or headingless document (§6.1-0002). Appendix C
renders the §2.5 `add` contract to its document bytes as the first golden document vector.

- **KISS-CONTRACT-6.11-0001** — A contract document MUST be encoded as **UTF-8 text** with **LF**
  (`0x0A`) line terminators and no other line-ending convention. Within a section block each
  field MUST occupy exactly one line of the form `<key> = <value>` (the key spelled exactly as
  its field-schema clause names it, then one space, the ASCII `=` `0x3D`, then one space, then
  the value), and MUST be encoded by its type as follows: a **string**/**enumerated token**
  value MUST be its verbatim UTF-8 bytes spanning to the line's LF (carrying the exact byte
  spelling its owning vocabulary pins, and MUST NOT contain an LF); an **integer** MUST be
  decimal ASCII, a negative value carrying a leading `-` (two's-complement value); an **array**
  MUST be `[` then the elements separated by `, ` (comma then one space) then `]`, in the pinned
  element order; an **opaque byte blob** MUST be `<n>:<hex>` where `<n>` is the decimal byte
  count and `<hex>` is exactly `2·n` lowercase hex digits (an empty blob is `0:`); and a
  **real** (floating-point) value MUST be encoded per §6.11-0010. An
  implementation MUST NOT vary these encodings by platform. *Test:*
  `test_contract_text_field_encoding`.
- **KISS-CONTRACT-6.11-0002** — A contract document MUST begin with the pinned **header line**:
  the 4-byte magic `0x4B 0x49 0x53 0x43` (ASCII `KISC`), one space, the `contract_kind` token
  (exactly `kiss-contract`, §6.1-0007), one space, the `contract_version` (decimal ASCII), one
  space, `len=<N>`, one space, `crc32=<HHHHHHHH>`, then a single LF (`0x0A`) — for example
  `KISC kiss-contract 1 len=<N> crc32=<HHHHHHHH>\n`. A reader MUST reject, with a typed decline,
  any document not beginning with the magic `KISC` or whose header line does not match this
  pinned form. *Test:* `test_contract_document_header_line`.
- **KISS-CONTRACT-6.11-0003** — The header line MUST declare the document's inner framing: `<N>`
  in `len=<N>` MUST be the decimal byte count of the document **body** (every byte after the
  header line's terminating LF, through the end of the document), and `<HHHHHHHH>` in
  `crc32=<HHHHHHHH>` MUST be the CRC-32 (IEEE 802.3 polynomial, reflected, initial `0xFFFFFFFF`,
  final XOR `0xFFFFFFFF`) of exactly those `N` body bytes, as 8 lowercase hex digits. A reader
  MUST determine the document's exact byte extent as the header line (through its LF) plus `N`
  body bytes, MUST NOT read beyond it, and MUST reject, with a typed decline, a document whose
  body is not exactly `N` bytes or whose CRC-32 does not match. This is the self-delimiting
  extent and loud-failure integrity required by §6.1-0001 / §6.1-0002, independent of any outer
  transport frame. *Test:* `test_contract_document_inner_length_and_crc`.
- **KISS-CONTRACT-6.11-0004** — The body MUST consist of exactly the seven section blocks in the
  fixed order Identity(1), Semantics(2), Interface(3), Dispatch(4), Capabilities(5),
  Guarantees(6), Provenance(7); each block MUST begin with the pinned **heading line**
  `[section:<id>:<name>]` followed by a single LF, where `<id>` is the decimal section number
  `1`–`7` and `<name>` is the lowercase section name matching that id (`identity`, `semantics`,
  `interface`, `dispatch`, `capabilities`, `guarantees`, `provenance`), and MUST be followed by
  that section's field lines (§6.11-0005). A reader MUST reject, with a typed decline, a document
  whose section headings are absent, misspelled, out of order, or duplicated, or that carries any
  body content before the first heading line (a **headingless** block MUST NOT be silently
  imported). *Test:* `test_contract_document_section_headings`.
- **KISS-CONTRACT-6.11-0005** — Within each section block, the field lines MUST appear in the
  exact order the section's field-schema clause lists them (§6.3-0001, §6.4-0001, §6.5-0001,
  §6.6-0001, §6.7-0001, §6.8-0001, §6.9-0001) and MUST be encoded per §6.11-0001; a reader MUST
  reject, with a typed decline, a block whose field order, field keys, or value encoding does
  not match the schema (a missing **required** field, an unknown field, or an extra field is a
  decline per §6.1-0006). A field the field-schema clause marks **optional** (such as the
  Semantics `human_annotation`, pinned last in §6.4-0001) MAY be absent; when absent its line is
  omitted, and when present it MUST occupy its pinned position — its absence is **not** a
  field-order or missing-field decline. *Test:* `test_contract_document_field_order`.
- **KISS-CONTRACT-6.11-0006** — The `positional_signature` value MUST be encoded as an array
  (§6.11-0001) of argument descriptors in the pinned order of §6.5-0004a / §6.5-0004d, each
  descriptor being the parenthesized tuple `(<arg_kind>, <launch_class>, <arg_name>,
  <type_token>, <dtype>, <array_len_kind>)` where `arg_kind` is `ptr` (operand pointer), `ls`
  (launch scalar), or `idesc` (the class-5 **index-operand descriptor**, §6.5-0013);
  `launch_class` is `0` for an operand pointer else the launch-scalar class `1`–`8` of
  §6.5-0004a, `arg_name` and `type_token` are tokens, `dtype` is the KISS-Classify dtype token
  (or `-` when not a typed pointer), and `array_len_kind` is `scalar` or `rank` (a rank-length
  array). For an `idesc` argument (`arg_kind = idesc`, `launch_class = 5`) the tuple slots are
  fixed as `<type_token> = idx` (the literal marker), `<dtype>` = the index operand's
  KISS-Classify dtype token, and `<array_len_kind>` = the decimal index **rank** followed by the
  ASCII `@` and the extent-binding token naming the class-5 `idx_extents{i}` it binds (for
  example `2@idx_extents0`), fixing the index operand's dtype, rank, and gather/index-extent
  binding (§6.5-0013). An implementation MUST NOT encode an argument descriptor outside this
  layout. *Test:* `test_contract_text_positional_signature`.
- **KISS-CONTRACT-6.11-0007** — The `op_dag` value MUST be encoded as an array (§6.11-0001) of
  nodes in a fixed order **identical to the KISS-Grammar canonical node table** (KISS-GRAMMAR
  §6.4-0001 / §6.8-0004): a strict order in which every child node appears **strictly earlier**
  than its parent, so each node's `child_edges` reference **strictly-earlier** indices and the
  **root node appears last**. Each node MUST be encoded as `Op{<op_name>; <op_attrs>;
  <child_edges>}` for an `Op` node — where `op_name` is a **bare KISS-Ops op name** (never a
  KISS-Grammar tag spelled inside `op_name`; identity-bearing attributes ride `op_attrs`,
  §6.4-0009), `op_attrs` is a `key=value` list separated by `, ` with keys and values spelled
  as KISS-Ops spells them (empty when the op carries none), and `child_edges` is a
  `[i0, i1, …]` array of strictly-earlier child node indices in operand order — or
  `Bind(<bind_index>)` for a `Bind` node binding that positional operand. This renders the
  §6.4-0009 node schema to text in the Grammar node order, so two dissimilar implementations
  serialize the identical bytes; an implementation MUST NOT encode a node kind or field outside
  it, MUST NOT place the root first, and MUST NOT reference a later index in `child_edges`.
  *Test:* `test_contract_text_op_dag`.
- **KISS-CONTRACT-6.11-0008** — Each Dispatch derivation field (`invocation_domain`,
  `workgroup_sizing`, `count_to_grid`, `thread_mapping`, `addressing_rule`) MUST be encoded as a
  string value (§6.11-0001) carrying a machine-evaluable expression in the pinned grammar of
  §6.6-0006; a reader MUST reject, with a typed decline, a Dispatch field that is not an
  expression the §6.6-0006 grammar accepts over the launch-scalar symbol vocabulary. *Test:*
  `test_contract_text_dispatch`.
- **KISS-CONTRACT-6.11-0009** — The `op_identity` value MUST be encoded as a single string
  value (§6.11-0001, LF-free) in the pinned canonical spelling
  `<op_name>[{<op_attrs>}][[<operand_role_tuple>]]`, where `<op_name>` is the bare KISS-Ops op
  name of the Semantics DAG root's op; `{<op_attrs>}` is present **iff** the root carries a
  non-empty identity-bearing OpAttrs set and is then the `key=value` list separated by `, `
  spelled exactly as KISS-Ops spells them (the same spelling as §6.11-0007); and
  `[<operand_role_tuple>]` is a `[role:dtype, …]` list of the root op's operand roles in
  KISS-Classify canonical operand order, present **iff** the KISS-Grammar advertisable-op tag
  carries an identity-bearing operand-role tuple (KISS-GRAMMAR §6.1-0007) — form (a) only. For
  **form (a)** (§6.3-0003 form (a)) this full spelling is the canonical text rendering of the
  KISS-Grammar advertisable-op tag: KISS-Grammar owns the tag's semantic identity (KISS-GRAMMAR
  §6.1-0007), and this clause pins only its byte rendering on the contract document's
  `op_identity` line. For **form (b)** (§6.3-0003 form (b)) the encoding is
  `<op_name>[{<op_attrs>}]` with **no** `[<operand_role_tuple>]` suffix (no advertisable-op
  entry). A root op with no identity-bearing attributes and no identity-bearing operand-role
  tuple therefore renders as the bare `<op_name>` (for example `add`). Two dissimilar
  implementations MUST render the identical `op_identity` bytes for the same root op, and a
  reader MUST reject, with a typed decline, an `op_identity` value not matching this spelling.
  **Default-resolution (§6.19-0005 alignment).** Op-identity *equality* is authoritatively
  defined by the KISS-Grammar binary tag canonical serialization (KISS-GRAMMAR §6.8-0012),
  under which every identity-bearing attribute is resolved to its explicit default so that a
  defaulted attribute and an explicitly-equal one produce identical bytes (KISS-OPS §6.19-0005);
  this text `op_identity` line is a **non-authoritative render** of that canonical form. A
  reader comparing two op identities MUST compare via the KISS-Grammar canonical binary tag
  serialization (§6.8-0012), and MUST NOT decide op-identity equality by byte-comparing this
  text line, so that a defaulted-vs-explicit attribute difference surfacing in the text render
  (this clause's default-eliding `{<op_attrs>}` presence rule) can never split, or falsely
  merge, a single op identity. *Test:* `test_contract_text_op_identity`.
- **KISS-CONTRACT-6.11-0010** — A **real** (floating-point) field value MUST be encoded as
  `<dtype>:<hex>` where `<dtype>` is the KISS-Classify float dtype token (`f32` or `f64`) and
  `<hex>` is the value's IEEE 754-2019 bit pattern in that dtype written **most-significant
  byte first** (big-endian, matching the header line's `crc32=<HHHHHHHH>` hex convention) as
  exactly `2·width_bytes` lowercase hex digits (`8` for `f32`, `16` for `f64`); the value MUST
  be pinned by this bit pattern and MUST NOT be encoded as a decimal spelling (umbrella §3.5),
  so each non-finite value (±∞, quiet NaN, signaling NaN, ±0, subnormal) is carried by its bit
  pattern and round-trips exactly. A reader MUST reject, with a typed decline, a real value
  whose `<dtype>` is neither `f32` nor `f64` or whose `<hex>` is not exactly the dtype width in
  lowercase hex. This clause defines only the real-number **encoding primitive**; the field
  schemas that carry real values (for example the Guarantees accuracy tiers and the cost
  expression coefficients) reference it once their layouts are pinned. *Test:*
  `test_contract_text_real_encoding`.

---

## 7. Capability, Profile & Extension model

### 7.1 Mandatory core and the lift-fraction degrade

- **KISS-CONTRACT-7.1-0001** — The KISS-Contract **mandatory core** MUST be the universal
  required core of seven sections (§6.2-0001) plus the self-delimiting hard-reject transport
  (§6.1) over the pinned structured/text document framing (§6.11). An implementation that cannot
  emit or read the seven-section core over the self-delimiting structured/text document does not
  conform to KISS-Contract at all. *Test:* `test_contract_mandatory_core`.
- **KISS-CONTRACT-7.1-0002** — The Semantics `semantics_kind` MUST be the negotiable degrade
  axis: `machine-checkable-IR` for a fully-lifted/generated kernel and `declared-op-tag` plus
  `lift_residue` for a hand-written kernel (§6.2-0004, §6.4-0007). An implementation MUST NOT
  omit the Semantics section on the ground that a kernel is only partially liftable, and MUST
  NOT fake `machine-checkable-IR` where residue remains. *Test:* `test_contract_semantics_degrade_axis`.
- **KISS-CONTRACT-7.1-0003** — An input that is malformed per §6.1, carries an inconsistent
  Identity per §6.3-0006, or a Semantics that does not resolve to the primitive floor per
  §6.4-0005, MUST produce a **typed decline, never a panic** (§6.2-0005); KISS-Conform verifies
  both that conforming contracts round-trip and that malformed or inconsistent inputs decline
  cleanly. *Test:* `test_contract_typed_decline_core`.

### 7.2 Extension and promotion

- **KISS-CONTRACT-7.2-0001** — A new op that a contract's Semantics or Identity references
  MUST enter through the KISS-Ops op-set extension path (a new KISS-Ops op name) plus
  KISS-Grammar re-basing; KISS-Contract MUST NOT define a separate op-token extension registry,
  because it owns no op vocabulary. Any KISS-Contract-owned extension (a new section field or a
  new schema field) is a schema-version change governed by §8 and the umbrella extension
  registry (umbrella §6.4). *Test:* `test_contract_extension_via_ops_or_schema`.
- **KISS-CONTRACT-7.2-0002** — A new external interchange token referenced by a contract's
  Interface or Provenance (for example a quantization-family token) MUST pin only its axis and
  defer its meaning to the external-token registry (umbrella §6.4); an implementation MUST NOT
  fold an unregistered external token's meaning into the contract schema. *Test:*
  `test_contract_external_token_deferred`.

---

## 8. Versioning & Lifecycle

KISS-Contract tracks the umbrella's **two version axes**: the **contract schema version**
(the seven-section schema, each section's field schema, and the transport/wire format) stamped
in the contract header as `contract_version`, and the published reference-crate **semver**.
They move independently, and both move independently of the KISS-Ops op-name set and the
KISS-Grammar frozen-shape schema.

- **KISS-CONTRACT-8-0001** — The contract schema version and the reference-crate semver MUST be
  tracked as independent axes; a crate semver change MUST NOT be taken to imply a contract schema
  change. *Test:* `test_contract_two_version_axes_independent`.
- **KISS-CONTRACT-8-0002** — Any change to the contract **schema** — a section, a section's
  field schema, the launch-scalar pinned order (§6.5-0004a), the transport framing or
  structured/text document format (§6.1, §6.11), the identity compatibility table (§6.3), or the
  `audited_status` derivation rule (§6.8-0009 / §6.8-0010) — MUST bump the `contract_version`.
  *Test:* `test_contract_schema_change_bumps_version`.
- **KISS-CONTRACT-8-0003** — A KISS-Ops op-name-set addition or a KISS-Grammar frozen-shape
  change MUST NOT, by itself, bump the KISS-Contract schema version (a contract references those
  vocabularies by name/tag under the growth rule); an implementation MUST NOT take an upstream
  vocabulary version change as a contract schema change. *Test:* `test_contract_upstream_growth_no_bump`.
- **KISS-CONTRACT-8-0004** — KISS-Contract MUST NOT be promoted from Draft to Frozen until ≥2
  structurally dissimilar implementations can **bind and launch and reason** about a kernel from
  the contract text alone, interoperating on the golden contract vectors of Appendix A and the
  golden document vector of Appendix C (umbrella §5.3). *Test:* `test_contract_freeze_gate_two_impls`
  (checklist gate; signed by the AUDIT role, not DESIGN).
- **KISS-CONTRACT-8-0005** — KISS-Contract MUST NOT be promoted from Draft to Frozen until a
  foreign reader written outside the reference language has consumed the self-delimiting contract
  document and reproduced or parsed the golden contract vectors (Appendix A) and the golden
  document vector (Appendix C) from the §6.11 structured/text document framing alone, with the
  hard-reject transport discipline exercised (the adversarial-outsider checklist, umbrella §5.3).
  *Test:* `test_contract_freeze_gate_foreign_reader` (checklist gate; AUDIT-signed).
- **KISS-CONTRACT-8-0006** — KISS-Contract MUST NOT be promoted from Draft to Frozen until this
  sub-standard's KISS-Conform suite exists and passes, with complete bidirectional
  clause-to-test traceability (umbrella §5.3). *Test:*
  `test_contract_freeze_gate_conform_suite_passes` (checklist gate; AUDIT-signed).
- **KISS-CONTRACT-8-0007** — Retire-by-floor deprecation MUST apply to the contract schema
  version only; a reader MUST reject a contract whose declared schema version is below its
  declared retirement floor with a typed decline (§6.1-0003). *Test:* `test_contract_retire_by_floor`.

---

## 9. Conformance

An implementation conforms to KISS-Contract at a given contract schema version if it (a) emits
and reads the seven-section contract exactly per §6–§8 for that version over the self-delimiting
hard-reject transport and structured/text document framing (§6.11), (b) passes the KISS-Conform suite for
KISS-Contract at that version, and (c) satisfies the DAG prerequisite closure. Because the
KISS-Classify → KISS-Contract, KISS-Ops → KISS-Contract, and KISS-Grammar → KISS-Contract edges
are **STRUCTURAL** (§4), claiming KISS-Contract requires claiming KISS-Classify, KISS-Ops, and
KISS-Grammar (prerequisite closure, umbrella §6.3). The downstream KISS-Contract → KISS-Announce
edge is **OPAQUE**, so a claim of KISS-Announce requires only agreement on the meaning of the
contract payload token, not a co-claim of KISS-Contract. Malformed, headingless, unknown-version,
or internally-inconsistent inputs yield typed declines, never panics, per §6.1-0004, §6.2-0005,
and §7.1-0003 (verified by the negative-vector modality). The modified-suite prohibition of the
mark policy is the umbrella's rule (umbrella §9.3), enforced via registry listing, and is not
restated as a free-standing KISS-Contract clause.

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-CONTRACT-6.0-0001 | `test_contract_determinism_class_exact_byte` |
| KISS-CONTRACT-6.1-0001 | `test_contract_self_delimiting_document` |
| KISS-CONTRACT-6.1-0002 | `test_contract_reject_malformed_header` |
| KISS-CONTRACT-6.1-0003 | `test_contract_reject_unknown_version` |
| KISS-CONTRACT-6.1-0004 | `test_contract_rejection_is_typed_decline` |
| KISS-CONTRACT-6.1-0005 | `test_contract_opaque_length_delimited_over_announce` |
| KISS-CONTRACT-6.1-0006 | `test_contract_strict_schema` |
| KISS-CONTRACT-6.1-0007 | `test_contract_kind_recognized_token` |
| KISS-CONTRACT-6.1-0008 | `test_contract_version_value_pinned` |
| KISS-CONTRACT-6.2-0001 | `test_contract_seven_section_core` |
| KISS-CONTRACT-6.2-0002 | `test_contract_existence_is_intrinsic` |
| KISS-CONTRACT-6.2-0003 | `test_contract_no_op_class_withhold` |
| KISS-CONTRACT-6.2-0004 | `test_contract_completeness_tracks_lift_fraction` |
| KISS-CONTRACT-6.2-0005 | `test_contract_typed_decline_never_panic` |
| KISS-CONTRACT-6.2-0006 | `test_contract_valid_without_grammar_tag` |
| KISS-CONTRACT-6.3-0001 | `test_contract_identity_field_schema` |
| KISS-CONTRACT-6.3-0002 | `test_contract_accept_is_structure_key` |
| KISS-CONTRACT-6.3-0003 | `test_contract_op_identity_full_tag_or_bare_ops_dag` |
| KISS-CONTRACT-6.3-0004 | `test_contract_op_identity_carries_distinguishing_attrs` |
| KISS-CONTRACT-6.3-0005 | `test_contract_identity_requires_both` |
| KISS-CONTRACT-6.3-0006 | `test_contract_identity_consistency` |
| KISS-CONTRACT-6.3-0007 | `test_contract_target_capability_byte_exact` |
| KISS-CONTRACT-6.3-0008 | `test_contract_revision_hash_opaque` |
| KISS-CONTRACT-6.3-0009 | `test_contract_identity_header_match` |
| KISS-CONTRACT-6.3-0010 | `test_contract_kernel_name_distinct` |
| KISS-CONTRACT-6.3-0011 | `test_contract_structure_key_target_matches_capability` |
| KISS-CONTRACT-6.4-0001 | `test_contract_semantics_mandatory` |
| KISS-CONTRACT-6.4-0002 | `test_contract_op_dag_over_ops_and_grammar` |
| KISS-CONTRACT-6.4-0003 | `test_contract_node_carries_opattrs` |
| KISS-CONTRACT-6.4-0004 | `test_contract_edge_case_from_ops` |
| KISS-CONTRACT-6.4-0005 | `test_contract_semantics_resolves_to_floor` |
| KISS-CONTRACT-6.4-0006 | `test_contract_lowered_form_is_oracle` |
| KISS-CONTRACT-6.4-0007 | `test_contract_semantics_kind_matches_lift` |
| KISS-CONTRACT-6.4-0008 | `test_contract_region_maps_to_semantics` |
| KISS-CONTRACT-6.4-0009 | `test_contract_op_dag_node_schema` |
| KISS-CONTRACT-6.4-0010 | `test_contract_semantics_resolution_outcomes` |
| KISS-CONTRACT-6.4-0011 | `test_contract_output_shape_consistency` |
| KISS-CONTRACT-6.5-0001 | `test_contract_interface_field_schema` |
| KISS-CONTRACT-6.5-0002 | `test_contract_positional_signature_complete` |
| KISS-CONTRACT-6.5-0003 | `test_contract_interface_references_identity_target` |
| KISS-CONTRACT-6.5-0004a | `test_contract_launch_scalar_pinned_order` |
| KISS-CONTRACT-6.5-0004b | `test_contract_launch_scalar_typed_positioned` |
| KISS-CONTRACT-6.5-0004c | `test_contract_launch_scalar_no_reorder` |
| KISS-CONTRACT-6.5-0004d | `test_contract_launch_scalar_within_class_order` |
| KISS-CONTRACT-6.5-0005 | `test_contract_strided_cell_carries_strides` |
| KISS-CONTRACT-6.5-0006 | `test_contract_offset_kernel_carries_offsets` |
| KISS-CONTRACT-6.5-0007 | `test_contract_index_operand_typed` |
| KISS-CONTRACT-6.5-0008 | `test_contract_count_unit_enum` |
| KISS-CONTRACT-6.5-0009 | `test_contract_in_place_declared` |
| KISS-CONTRACT-6.5-0010 | `test_contract_alignment_declared` |
| KISS-CONTRACT-6.5-0011 | `test_contract_launch_scalars_match_signature_tail` |
| KISS-CONTRACT-6.5-0012 | `test_contract_rank_declared` |
| KISS-CONTRACT-6.5-0013 | `test_contract_param_count_and_index_descriptor` |
| KISS-CONTRACT-6.6-0001 | `test_contract_dispatch_field_schema` |
| KISS-CONTRACT-6.6-0002 | `test_contract_invocation_domain` |
| KISS-CONTRACT-6.6-0003 | `test_contract_count_to_grid` |
| KISS-CONTRACT-6.6-0004 | `test_contract_thread_and_addressing` |
| KISS-CONTRACT-6.6-0005 | `test_contract_dispatch_interface_consistent` |
| KISS-CONTRACT-6.6-0006 | `test_contract_dispatch_expressions_machine_evaluable` |
| KISS-CONTRACT-6.6-0007 | `test_contract_dispatch_optional` |
| KISS-CONTRACT-6.6-0008 | `test_contract_reserved_tile_mapping` |
| KISS-CONTRACT-6.7-0001 | `test_contract_capabilities_field_schema` |
| KISS-CONTRACT-6.7-0002 | `test_contract_capabilities_accept_matches_identity` |
| KISS-CONTRACT-6.7-0003 | `test_contract_capabilities_is_envelope` |
| KISS-CONTRACT-6.7-0004 | `test_contract_capabilities_determinism_from_ops` |
| KISS-CONTRACT-6.7-0005 | `test_contract_precision_class_consistent` |
| KISS-CONTRACT-6.7-0006 | `test_contract_cost_expressions` |
| KISS-CONTRACT-6.8-0001 | `test_contract_guarantees_field_schema` |
| KISS-CONTRACT-6.8-0002 | `test_contract_precision_names_reference` |
| KISS-CONTRACT-6.8-0003 | `test_contract_determinism_class_imported` |
| KISS-CONTRACT-6.8-0004 | `test_contract_math_precision_imported` |
| KISS-CONTRACT-6.8-0005 | `test_contract_bit_stability_consistent` |
| KISS-CONTRACT-6.8-0006 | `test_contract_guarantees_cost_provenance` |
| KISS-CONTRACT-6.8-0007 | `test_contract_cost_single_home` |
| KISS-CONTRACT-6.8-0008 | `test_contract_audited_status_derived` |
| KISS-CONTRACT-6.8-0009 | `test_contract_audited_derivation_rule` |
| KISS-CONTRACT-6.8-0010 | `test_contract_unaudited_derivation_rule` |
| KISS-CONTRACT-6.9-0001 | `test_contract_provenance_field_schema` |
| KISS-CONTRACT-6.9-0002 | `test_contract_provenance_source` |
| KISS-CONTRACT-6.9-0003 | `test_contract_provenance_revision_matches_identity` |
| KISS-CONTRACT-6.9-0006 | `test_contract_cost_provenance_consistent` |
| KISS-CONTRACT-6.9-0008 | `test_contract_negotiation_metadata_opaque` |
| KISS-CONTRACT-6.10-0001 | `test_contract_grammar_ops_seam_two_sections` |
| KISS-CONTRACT-6.10-0002 | `test_contract_edge_case_and_ulp_from_ops` |
| KISS-CONTRACT-6.10-0003 | `test_contract_single_op_vocabulary` |
| KISS-CONTRACT-6.10-0004 | `test_contract_opattrs_cited_from_ops` |
| KISS-CONTRACT-6.11-0001 | `test_contract_text_field_encoding` |
| KISS-CONTRACT-6.11-0002 | `test_contract_document_header_line` |
| KISS-CONTRACT-6.11-0003 | `test_contract_document_inner_length_and_crc` |
| KISS-CONTRACT-6.11-0004 | `test_contract_document_section_headings` |
| KISS-CONTRACT-6.11-0005 | `test_contract_document_field_order` |
| KISS-CONTRACT-6.11-0006 | `test_contract_text_positional_signature` |
| KISS-CONTRACT-6.11-0007 | `test_contract_text_op_dag` |
| KISS-CONTRACT-6.11-0008 | `test_contract_text_dispatch` |
| KISS-CONTRACT-6.11-0009 | `test_contract_text_op_identity` |
| KISS-CONTRACT-6.11-0010 | `test_contract_text_real_encoding` |
| KISS-CONTRACT-7.1-0001 | `test_contract_mandatory_core` |
| KISS-CONTRACT-7.1-0002 | `test_contract_semantics_degrade_axis` |
| KISS-CONTRACT-7.1-0003 | `test_contract_typed_decline_core` |
| KISS-CONTRACT-7.2-0001 | `test_contract_extension_via_ops_or_schema` |
| KISS-CONTRACT-7.2-0002 | `test_contract_external_token_deferred` |
| KISS-CONTRACT-8-0001 | `test_contract_two_version_axes_independent` |
| KISS-CONTRACT-8-0002 | `test_contract_schema_change_bumps_version` |
| KISS-CONTRACT-8-0003 | `test_contract_upstream_growth_no_bump` |
| KISS-CONTRACT-8-0004 | `test_contract_freeze_gate_two_impls` |
| KISS-CONTRACT-8-0005 | `test_contract_freeze_gate_foreign_reader` |
| KISS-CONTRACT-8-0006 | `test_contract_freeze_gate_conform_suite_passes` |
| KISS-CONTRACT-8-0007 | `test_contract_retire_by_floor` |

Every normative clause above appears in this matrix exactly once; the KISS-Conform build fails
if any clause ID lacks a passing mapped test (bidirectional traceability, owned by KISS-Conform
per umbrella §3.3). Clause IDs are mirrored in the machine-readable sidecar
(`kiss-contract.validusage.json` analog) kept in sync by the traceability lint.

---

## 10. Governance

- **Editor of record:** the KISS-Contract editor assignment is **proposed, pending
  ratification** in the umbrella governance record. The editor holds the pen, allocates clause
  IDs (append-only, never reused after retirement), and solicits comment from interested
  cosignatories — any project building a provider, consumer, synthesizer, lifter, or emitter that
  emits or reads a contract — before deciding a cross-party-visible change. A cross-party-visible
  launch-scalar-ordering, wire-format, or transport-framing change is coordinated across affected
  parties as an RFC before it is wired.
- **Steward:** ThinkersJournal hosts the spec, the extension registry (PR-gated; note that the
  op-**name** vocabulary is owned by KISS-Ops and the dtype/`target_capability` vocabulary by
  KISS-Classify, not by a KISS-Contract registry), and the conformance registry; it free-certifies
  self-certified implementations on request as resources permit.
- **Ratifier / maturity transitions:** the AUDIT role (not DESIGN) signs each maturity transition;
  the Draft→Frozen transition requires the freeze gate of §8-0004 / §8-0005 / §8-0006 (umbrella
  §5.3).
- **License:** this specification is dedicated to the public domain under CC0 1.0 Universal;
  reference crates are MIT-OR-Apache-2.0; the KISS-Conform suite is permissive-to-run. Per the
  umbrella mark policy (umbrella §9.3), a modified conformance suite does not back a conformance
  claim; that policy is enforced via steward-registry listing, not restated as a normative
  KISS-Contract clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC contribution,
  with defensive termination, per the umbrella.
- **Conformance posture:** self-certification with published results plus the steward-maintained
  registry is the authoritative record of verified implementations.

---

## Appendix A — Worked contract vectors & provenance (informative)

**A.1 Golden contract vectors.** The strided binary `add` contract of §2.5 is the first golden
contract vector for `test_contract_seven_section_core`, `test_contract_identity_requires_both`,
`test_contract_positional_signature_complete`, `test_contract_launch_scalar_pinned_order`, and
`test_contract_determinism_class_imported` — a `machine-checkable-IR` Semantics (a one-node `add`
DAG), a strided Interface carrying per-operand extents and signed strides plus `n`, `rank = 1`,
`count_unit = elements`, `in_place = none`, determinism class `exact-byte`, MathPrecision
`bit-stable`. Its rendering as a structured/text document under the §6.11 framing is Appendix C. The `gather` contract
of §2.6 is the golden vector for `test_contract_node_carries_opattrs`,
`test_contract_op_identity_carries_distinguishing_attrs`, `test_contract_index_operand_typed`, and
`test_contract_no_op_class_withhold` — carrying `axis = k` and OOB policy `clamp` on the Semantics
OpAttrs channel, the index operand's own dtype pointer, and the gather/index-extent launch scalars;
its `op_identity` renders as `gather{axis=k, oob=clamp}` (§6.11-0009), shown with a form-(b)
bare-KISS-Ops `op_identity` vector in Appendix C for `test_contract_text_op_identity` and
`test_contract_op_identity_full_tag_or_bare_ops_dag`.
Additional negative vectors — a headingless block (silently-droppable under the old markdown
transport, now a hard `test_contract_reject_malformed_header` decline), an unknown
`contract_version`, a contract missing a required section, a contract carrying an unknown/extra
field (`test_contract_strict_schema`), an Identity whose `structure_key` op-category disagrees with
its `op_identity` family (`test_contract_identity_consistency`), a strided cell whose Interface
omits its strides (`test_contract_strided_cell_carries_strides`), an offset kernel omitting its base
offsets, a `declared-op-tag` Semantics falsely marked `machine-checkable-IR`, and an
authored-constant `audited_status` contradicting the derivation rule (`test_contract_audited_status_derived`,
now a §6.8 Guarantees vector) — drive the §6.1 / §6.2 / §6.3 / §6.5 / §6.8 decline tests and form the
adversarial-outsider battery for the foreign-reader freeze gate.

**A.2 Provenance / acknowledgments.** The seven-section decomposition (Identity / Semantics /
Interface / Dispatch / Capabilities / Guarantees / Provenance), the accept-predicate-is-the-
`structure_key` honesty invariant, the launch-scalar enumeration (extents / signed strides / `n` /
base offsets / gather extents / workspace pointer+size / scalar params), the `count_unit` load-bearing
distinction, the derived `audited_status`, and the removal of the honest-miss gates derive from a
contract-emission reference crate (the Baracuda `baracuda-kernelgen` `contract` module), which
assembled a per-kernel contract from an op definition and its specialization cell. The neutralization
of the earlier vendor vocabulary — a vendor "kernel contract" acronym rendered as "KISS-Contract"; a
foreign named-op / fused-op vocabulary re-based onto KISS-Ops names + KISS-Grammar advertisable tags;
the honest-miss no-contract gates (the primitive-op-kind None-withhold, the fused-op whitelist filter,
the gather/scatter/offset/addressing-view returns-None guards, and the comparison/`select` wholesale
withholds) removed by the OpAttrs channel and per-operand dtype tuple that carry the axis/OOB/index-dtype
the flat grammar could not express; the markdown-fenced transport whose headingless-block drop was a
silent-empty-import footgun, replaced by a self-delimiting hard-reject document; the derived
`audited_status` generalizing the earlier determinism→bit-stability→audited coupling — is recorded here
as design provenance, not as a normative requirement. Project and crate names in this appendix and in
§0/§2 are non-normative provenance and examples only; no normative clause names any project.

**A.3 Growth-rule migration note.** Because a contract references KISS-Ops op names and KISS-Grammar
advertisable tags by name/tag under the growth rule, a KISS-Ops op-set addition or a KISS-Grammar
frozen-shape change reaches a contract's Identity and Semantics with **no** KISS-Contract schema bump
(§8-0003); the contract schema version bumps only on a change to a section, a section's field schema,
the launch-scalar pinned order, the transport framing or structured/text document format, the identity
compatibility table, or the `audited_status` derivation rule (§8-0002). This is the mechanism by which a frozen contract
schema admits a still-growing op set.

---

## Appendix B — Identity consistency compatibility mapping (informative rendering of the §6.3 normative table)

The **normative** compatibility mapping is the table under §6.3 (below clause §6.3-0006); it is the
authoritative artifact the Identity consistency assertion (§6.3-0006) is decided from, and it
enumerates the closed KISS-Classify cell op-category set and the closed KISS-Ops op-family set in full.
This appendix is an informative restatement only; on any discrepancy the §6.3 table governs.

A contract whose Identity pairs a `structure_key` op-category with an `op_identity` whose KISS-Ops op
family is absent from that category's compatible set is inconsistent and is rejected with a typed
decline (§6.3-0006). The full mapping is versioned with the contract schema (§8-0002); adding a
KISS-Ops op family or a KISS-Classify op-category adds rows without renumbering existing ones.

---

## Appendix C — Golden document vector: the §2.5 strided `add` contract (informative)

This appendix renders the §2.5 `add` contract as a concrete golden **document** vector under the
§6.11 structured/text document framing: a UTF-8 text document with LF line terminators, a pinned
header line, and seven section blocks under pinned heading lines (**not** a length-prefixed binary
envelope). The header line, the full Identity block, and the Semantics block are shown here to seed a
foreign reader; the complete document (all seven blocks) is carried in the machine-readable
golden-vector file. `<N>` is the decimal byte count of the document body (every byte after the header
line's LF), and `<HHHHHHHH>` is the CRC-32 of exactly those body bytes (§6.11-0003) — both computed
over the fully assembled document.

**Header line (§6.11-0002/-0003)** — begins with the 4-byte magic `KISC` (`0x4B 0x49 0x53 0x43`):

```
KISC kiss-contract 1 len=<N> crc32=<HHHHHHHH>
```

**Identity block (§6.11-0004/-0005; heading id 1)** — heading line then one `key = value` line per
field, in field-schema order (§6.3-0001); `revision_hash` is the opaque-blob text form `<n>:<hex>`
(§6.11-0001):

```
[section:1:identity]
contract_kind = kiss-contract
contract_version = 1
kernel_name = add_f32_strided_sm89
revision_hash = 4:deadbeef
accept_predicate = bin/f32,f32,f32/strided/cuda:sm89
op_identity = add
target_capability = cuda:sm89
```

**Semantics block (§6.11-0004/-0007; heading id 2)** — a one-node `machine-checkable-IR` DAG; the
`op_dag` is an array of one `Op` node with no attrs and no children (`Op{add; ; []}`, §6.11-0007):

```
[section:2:semantics]
semantics_kind = machine-checkable-IR
op_dag = [Op{add; ; []}]
```

The `op_dag` array is in the §6.11-0007 node order (children strictly earlier, root last); the
one-node `add` DAG has the root as its only (last) element with empty `child_edges`.

**`op_identity` renderings (§6.11-0009).** The §2.5 `add` root carries no identity-bearing
attributes and no identity-bearing operand-role tuple, so its `op_identity` line renders as the
bare `op_identity = add` (form (a) special case; the golden vector shown in the Identity block
above). Two further golden `op_identity` vectors seed a foreign reader:

```
op_identity = gather{axis=k, oob=clamp}
op_identity = scan{op=add}
```

The first is the §2.6 `gather` (form (a) with distinguishing attribute values), the golden vector
for `test_contract_op_identity_carries_distinguishing_attrs` and `test_contract_text_op_identity`.
The second is a hand-written bare-KISS-Ops kernel with **no** KISS-Grammar advertisable-op entry
whose Semantics DAG root is a `scan` with `op=add` (form (b) — bare op name plus identity-bearing
OpAttrs, **no** `[<operand_role_tuple>]` suffix), the golden vector for
`test_contract_op_identity_full_tag_or_bare_ops_dag` (form (b) case).

The Interface (note: no `target` field, §6.5-0001), Dispatch, Capabilities, Guarantees (carrying
the derived `audited_status`, §6.8-0008), and Provenance blocks follow under their heading lines with
the same field encoding (§6.11-0004/-0005/-0006/-0008). Every byte is UTF-8 with LF terminators; the
header line's `len` and `crc32` are computed over the fully assembled body so a reader determines the
exact extent and integrity from the document itself (§6.11-0003).

---

*End of KISS-Contract (Draft proposal). This sub-standard is informative in §0–§5 and normative in
§6+; every binding requirement carries an identified clause with a mapped KISS-Conform test.
KISS-Contract owns the universal, vendor-neutral seven-section kernel-contract format; it carries the
KISS-Classify `structure_key` accept-predicate and `target_capability`, the KISS-Grammar advertisable-op
tag as `op_identity`, and the KISS-Ops op DAG as its Semantics — referencing all three foundational and
middle-tier vocabularies by name/tag/structure over STRUCTURAL edges, and defining no data noun, no op
meaning, and no advertisable surface of its own. Every kernel carries a contract; completeness tracks
the lift fraction; contract existence is decided by the contract's own Semantics. Project and product
names appear only in non-normative examples, provenance, and the reference-implementation pointer;
normative clauses use only the generic roles provider, consumer, implementation, kernel, contract, and
target.*
