# KISS-Grammar — The Advertisable-Op Surface & Region Grammar

**Sub-standard ID:** KISS-GRAMMAR
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification* (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ is normative. Normative
> clauses use RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-GRAMMAR-<section>-<nnnn>`, and each normative requirement maps 1:1 to at
> least one named KISS-Conform test. The KISS-Conform suite build fails on any
> normative requirement without a mapped test. (Uppercase RFC-2119 keywords are used
> only in §6+ normative clauses, or when a keyword is named as a term; informative
> §0–§5 prose uses lowercase requirement language.)

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Grammar |
| Sub-standard ID | KISS-GRAMMAR |
| Tier | **Middle** (advertisable-op surface + region grammar; sits above the two foundational vocabularies and below the kernel-contract format) |
| Maturity stage | **Draft** (first-draft proposal; the frozen-shape schema is NOT yet frozen — the freeze gate of §8 is unmet) |
| Editor of record | **Proposed, pending ratification** — an advertisable-surface / provider-side project holds the pen and requests comment from interested cosignatories; the ratified governance record does not yet finalize an editor for KISS-Grammar. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | a pattern-derivation reference crate (`baracuda-kernelgen`, project/crate name given in Appendix A as non-normative provenance); this crate is *a* conformant implementation with no privilege. |
| DAG position | **Middle tier.** Depends (structurally) on KISS-Ops and KISS-Classify; consumed downstream (structurally) by KISS-Contract, and referenced by KISS-Consume and KISS-Emit (naming references) as the advertisable-op surface. Not a root. |
| Upstream edges | KISS-Ops (**STRUCTURAL** — every advertisable op re-bases onto a KISS-Ops op name; the OpAttrs channel, reference decompositions, the per-op commutativity/positionality property, and the operand-role vocabulary are owned there); KISS-Classify (**STRUCTURAL** — dtype-role tokens of the operand-role tuple and the cell-level op-category tag are KISS-Classify vocabulary) |
| Downstream edges | KISS-Contract (**STRUCTURAL** — the contract Identity `op_identity` field carries a KISS-Grammar advertisable-op tag **when the kernel advertises a named op** — a hand-written kernel may instead carry a bare KISS-Ops op DAG with no advertisable-op tag, §6.7-0005 — and the contract Semantics op DAG's nodes are advertisable-op tags and KISS-Ops op names); KISS-Consume (**naming reference**, informative — lift targets are named as advertisable-op tags over KISS-Ops names; **not** a prerequisite-closure edge in the umbrella §2.2 edge table); KISS-Emit (**naming reference**, informative — the advertisable surface names the ops an emitter lowers; **not** a prerequisite-closure edge in the umbrella §2.2 edge table); KISS-Conform (test dependency) |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |
| Maturity | Draft proposal |

> **Edge-label note (informative).** Both KISS-Grammar upstream edges are
> **STRUCTURAL**: KISS-Grammar parses the internal structure of a KISS-Ops op
> definition (its name, its OpAttrs channel, its reference decomposition, its
> per-op commutativity/positionality property, its operand-role vocabulary, and its
> primitive-floor membership and level) and of the KISS-Classify data vocabulary
> (dtype tokens, the cell-level op-category tag). This is the opposite of an
> OPAQUE edge (where a token is carried uninterpreted). The labels reconcile with
> the umbrella §2.2 edge table, which lists **KISS-Ops → KISS-Grammar** and
> **KISS-Classify → KISS-Grammar** each as STRUCTURAL. On the downstream side, the
> umbrella §2.2 edge table names **KISS-Grammar → KISS-Contract** as the direct
> STRUCTURAL prerequisite-closure edge. KISS-Consume and KISS-Emit **reference** the
> advertisable-op surface as a *naming* dependency — they lift into, and emit,
> KISS-Ops op names decorated as advertisable ops. These Consume/Emit references are
> **informative naming references only**: they are **not** STRUCTURAL prerequisite-closure
> edges and **do not appear** in the umbrella §2.2 edge table (in that table Consume and
> Emit depend on Ops/Classify/Contract, not on Grammar). They impose no
> prerequisite-closure obligation beyond the edges the umbrella edge table pins.

---

## 1. Purpose & Scope

KISS-Grammar owns the **advertisable-op surface** and the **region grammar** of the
suite: the middle-tier vocabulary that names *which computations a provider can
advertise* and *how those computations compose into recognizable regions
(fusions)*. It sits between the two foundational vocabularies — KISS-Classify (the
**data** nouns) and KISS-Ops (the **computation** verbs) — and the KISS-Contract
document that carries a kernel's meaning. It defines three things and nothing else:

1. **The advertisable-op surface.** An **advertisable op** is a KISS-Ops op **name**
   decorated with **pattern attributes** (the recognition/match channel), **synthesis
   attributes** (the generation/lowering channel), and an **operand-role tuple**
   (per-operand role + expected dtype role). The op name is the *sole identity
   anchor*: an advertisable op is a KISS-Ops op name plus attributes, **never** a
   parallel token that could drift from KISS-Ops. There is **no** independent,
   forkable advertisable-op enum.

2. **The region grammar.** How advertisable ops compose into **advertisable regions**
   — a single-rooted DAG of advertisable-op nodes over positional-input bind leaves,
   carried in a pinned, byte-exact **wire form**. A single advertisable op is a
   one-node region; a **fusion** is a multi-node region whose **root** carries the
   region's advertised identity. The grammar pins the composition rules: positional
   binds and the repeated-bind node-identity guard, the no-external-consumer
   fusion-safety guard, commutative-operand canonicalization (against a pinned total
   order and the KISS-Ops-declared per-op commutativity property) for reproducible
   emission, the scalar-param extract channel, the expressibility condition on the
   bound-input set relative to the declared region arity, and the serialization of
   all of the above into interoperable bytes.

3. **The growth rule.** How a **frozen** grammar admits a **still-growing** op set:
   the grammar freezes *shape* (the advertisable-op field schema, the region
   composition rules, the attribute-channel structure, and the region wire form),
   while the op **name** set stays growing and is deferred entirely to KISS-Ops. A
   newly-added KISS-Ops op becomes advertisable by **re-basing** onto its name, with
   no change to the frozen structural schema and no parallel op list to fork. Per-op
   properties a new op may introduce (its commutativity/positionality, its operand
   roles) are read from KISS-Ops by name, so the growing op set never forces an edit
   to any frozen literal.

KISS-Grammar is the **advertisable/named-op** surface and is **not required for every
kernel**. Every kernel carries a KISS-Contract, but a contract's semantic identity may be
a bare KISS-Ops op DAG (primitive or mixed-level) with **no** KISS-Grammar advertisable-op
entry — a hand-written kernel is not obliged to advertise a named op (§6.7-0005). The
advertisable-op surface and the region grammar exist for the ops a provider *advertises*
and *composes*; they sit off the mandatory contract path, not on it.

KISS-Grammar **maps** the advertisable surface onto KISS-Ops; it does **not** define
op *meaning*. Edge-case behavior (NaN propagation, signed zero, IEEE-`fmax` versus
NaN-propagating `max`), per-transcendental declared-ULP ceilings, per-op
commutativity/positionality, operand roles, and reference decompositions are resolved
**from** KISS-Ops. KISS-Grammar supplies the advertisable surface, the attribute
channels, the composition rules, and the wire form; KISS-Ops supplies the meaning,
the per-op properties, and the primitive floor.

**KISS-Grammar is NOT:** the computation vocabulary or per-op semantics (the op set,
NaN/signed-zero/wrapping/ULP behavior, per-op commutativity/positionality, operand
roles, and reference decompositions are KISS-Ops); the data vocabulary (dtype set,
operand descriptors, `structure_key`, `target_capability` — those are KISS-Classify);
the kernel-contract document format (that is KISS-Contract, which *carries* an
advertisable-op tag in its Identity section and an op DAG in its Semantics section);
the discovery/handshake protocol (KISS-Announce); a source language or grammar
(recognition **direction** is KISS-Consume; source-language parsing is out of scope
suite-wide); and it does **NOT** own any op's numeric meaning. Anything not enumerated
as in-scope above is out of scope for KISS-Grammar (scope creep by silence is a named
trap; silence is not inclusion).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — a name plus two channels, not a new op list

A provider does not advertise raw KISS-Ops atoms only; it advertises the
*computations it recognizes and can build* — `gelu`, a `matmul`+`gelu` epilogue
fusion, a `softmax`, a strided `gather`. The temptation is to give each of these a
fresh "advertisable-op" token in a list the provider owns. That list would
immediately **fork** from KISS-Ops: a `gelu` in the advertisable list and a `gelu`
in KISS-Ops could drift in spelling, in flavor (exact-erf vs tanh), or in edge-case
meaning, and a consumer would have two vocabularies to reconcile.

KISS-Grammar refuses the fork. An **advertisable op** is defined as *a KISS-Ops op
name decorated with attributes* — nothing more. The op name is the sole identity
anchor. Everything else the provider needs to say about the op splits into exactly
two channels:

- **Pattern attributes** — the **recognition/match** channel. Everything a consumer
  needs to match this advertised op *structurally* (never by sniffing its spelling):
  the KISS-Ops **OpAttrs** channel (a `reduce`/`gather`/`scan` axis, an out-of-bounds
  policy, a `transpose`/`permute` `perm`, a reduce-axis mask/keepdim), the
  operand-role tuple (a `u8` `cond`, a `u32` index), the **KISS-Ops-declared
  commutativity property** (for canonicalization), the consumer-count guard, and the
  node-identity binds. These are the attributes whose *values* are load-bearing for
  correctness — a transpose's `perm`, a gather's axis — and they must be matched with
  an explicit guard, not skipped through. The OpAttrs sub-vocabularies themselves — the
  out-of-bounds policy set, the permutation encoding, and the reduce-axis mask/keepdim
  encoding — are **KISS-Ops-owned op semantics** (§6.2-0006); KISS-Grammar cites KISS-Ops
  as their single owner and re-defines none of them. They are **distinct** from
  KISS-Grammar's own matching hints (the node-identity binds, the consumer-count guard,
  and the commutativity canonicalization key), which the grammar does own.

- **Synthesis attributes** — the **generation/lowering** channel. Everything needed to
  *synthesize* the op, parameterizing how it is lowered rather than how it is matched:
  the KISS-Ops reference-decomposition pointer (the resolution oracle), the
  exact-vs-approx flavor selection (expressed as *distinct KISS-Ops names* — `gelu`
  = exact-erf vs `gelu_tanh` = approximation; `relu` vs `fmax_ieee`; `rem_floor` vs
  `rem_trunc` — never a free-form flag), the scalar-param extract routing, and emit
  hints (`count_unit` / vector width, in-place eligibility, awkward-layout strategy).
  Synthesis attributes are what a provider consumes on the build-on-miss (provision)
  path. They are **not identity-bearing** and are **not** part of the region's
  byte-exact wire form (§2.8, §6.8-0009).

The **operand-role tuple** rides alongside: per-operand role + expected dtype role,
mirroring the KISS-Ops index/address operand role, so a consumer matches the
*mixed-dtype operand tuple* (the `u8` `cond` operand of `select`, the `u32` index
operand of `gather`/`index_select`), not just operand-0's dtype. The operand-role
token set (`data`, `cond`, `index`, `address`, …) is a KISS-Ops-owned closed set,
used here by name (§6.6-0005). A polymorphic (any-dtype) operand carries the pinned
dtype-role **wildcard** token rather than a concrete dtype (§6.6-0006). The op's
**level** (primitive-floor membership and decomposition level) is *inherited* from
KISS-Ops and never re-declared by Grammar.

### 2.2 The region grammar — a single-rooted DAG over binds

A single advertisable op is the simplest region: a **one-node region**. A fusion is a
**multi-node region** — a single-rooted DAG whose interior nodes are advertisable ops
and whose leaves are **binds** onto the region's positional inputs. So that a genuine
DAG (interior sharing, not only a tree) can be represented and serialized
unambiguously, the normative form is a **flat, indexed node table**: operands are
*indices* into that table, so a node referenced by two or more in-region parents is a
single shared node. The reference pattern grammar expresses this shape:

```
Region = {
  n_inputs         : declared region arity (how many positional inputs the region binds)
  ops_version      : the in-force KISS-Ops version these op names/attrs are read against
  classify_version : the in-force KISS-Classify version these dtype tokens are read against
  nodes            : ordered table nodes[0..k); operands reference strictly-earlier entries
  extract          : list of (path, param_slot) — scalar-param routing anchored on the root
}

Node = Op { op_name, pattern_attrs, operand_role_tuple, operands: [node_index …], consumers }
     | Bind(input_index)
```

**Scope of `pattern_attrs` here (narrow, per-node).** In the `Op { … }` record above,
`pattern_attrs` is the **per-node wire field** that carries **only** the KISS-Ops **OpAttrs
blob** (axis / OOB policy / permutation / reduce-axis mask/keepdim) — it is *not* the whole
recognition channel. The remaining members of the conceptual *pattern attributes* channel of
§6.2-0001 travel elsewhere: `operand_role_tuple` and `consumers` are the **sibling** node
fields shown; the KISS-Ops-declared commutativity property is read from KISS-Ops by name; and
the node-identity binds live on the `Bind` leaves. This narrow, single-scope reading is the
one KISS-Contract's op-graph node projection (`op_attrs ← pattern_attrs`, Contract §6.4-0009)
cites; the broader superset sense of "pattern attributes" is used only conceptually
(§6.2-0001).

The **root** is the single node that no other in-region node references (equivalently,
the last entry in the canonical node table). A one-node region's root *is* the op; a
fusion's root op is what the region advertises itself as. Internal sharing — a node
index appearing in more than one parent's `operands` list — makes the region a genuine
**DAG**, not merely a tree. The grammar pins these composition rules:

1. **Bind leaves reference positional inputs.** `Bind(i)` binds the producer at that
   leaf to the region's input `i`. A **repeated** `Bind` of the same index is the
   **node-identity guard**: a shared operand is *literally the same input*, so the
   guard is free — two leaves that bind index `i` assert the two subtrees read one and
   the same value, and canonicalize to one shared node-table entry (§6.4-0011).
   (Interior sharing of a *computed* node is expressed by two parents naming the same
   node index, not by repeating a leaf.)

2. **Interior nodes carry a no-external-consumer fusion-safety guard.** `consumers` is
   an enum counting **edges that leave the region** (consumers of the node's result in
   the surrounding candidate graph that are *not themselves inside the region*). An
   **interior** (non-root) node must carry `consumers = INTERIOR`, requiring its
   external-consumer count to be **0**: fusing a node whose result escapes the region
   would be unsafe. The **root** carries `consumers = ROOT` (its result *is* the
   region's output; it may be consumed freely). In-region sharing — two interior
   parents consuming one node — is **not** an external consumer and does **not**
   violate the guard; `consumers` never counts in-region edges. (This is why the model
   is a DAG and not a tree: `consumers` is external-only.)

3. **Commutative operands canonicalize; positional operands do not.** Whether a node's
   operands may be reordered is the **KISS-Ops-declared commutativity/positionality
   property** of that op, read from KISS-Ops by name — not a Grammar-owned literal
   list. For a KISS-Ops-declared **commutative** op (e.g. `add`, `mul`), operands are
   canonicalized to the pinned **total canonical operand order** (§6.4-0010) so two
   authorings of one expression (`a*b + c` vs `c + a*b`) emit byte-identically — for
   **reproducible emission**, *not* for match correctness (both the imported pattern
   and the candidate graph canonicalize before matching, so any single emitted order
   matches). A KISS-Ops-declared **positional** op — comparisons and `select`, whose
   operand order is load-bearing (`select` is `(cond, a, b)` per KISS-Ops) — must not
   be commutatively reordered.

4. **Scalar params ride an extract channel.** Scalar runtime parameters ride an
   **extract** channel anchored on the **root**. An extract path is a **sequence of
   canonical operand indices from the root** down to the target node, evaluated over
   the **already-canonicalized** tree (canonicalization happens first, §6.4-0007), and
   each path maps to a 0-based **param slot**, so a consumer can pull a runtime scalar
   out of the region without perturbing the op-graph shape.

5. **Exactly one output.** A region has exactly one root (§6.4-0009); a multi-output
   forest is not an expressible region.

A region is **expressible** only when its bound-input set equals exactly
`[0, n_inputs)`, where `n_inputs` is the region's **declared arity** field — every
positional input is bound, and no index outside the range is bound. Because `n_inputs`
is declared explicitly (not inferred as `max(bound index)+1`), a region that binds a
strict subset of its inputs *is* detectable and yields a typed decline (the reference
impl's `BindSetMismatch`); so does an out-of-range index.

### 2.3 A worked region — an elementwise fusion

Consider a provider that recognizes the epilogue `out = (a * b) + c` as a fusible
region over three positional inputs. Its region (rendered as the informative nested
view; the normative form is the flat node table of §6.8, and its exact bytes are in
Appendix A.1) is:

```
Op { op_name: "add", consumers: ROOT, operands: [
  Op { op_name: "mul", consumers: INTERIOR, operands: [ Bind(0), Bind(1) ] },
  Bind(2)
] }
```

The **root** is `add` — the region advertises itself as an `add`-rooted fusion.
`mul` is an interior node with `consumers = INTERIOR` (no external consumer permitted);
`add` and `mul` are KISS-Ops-declared commutative ops, so their operands are
canonicalized to the pinned total order for reproducible emission — the alternative
authoring `c + a*b` canonicalizes to the identical bytes. The bound-input set is
`{0, 1, 2}` = `[0, 3)` and `n_inputs = 3`, so the region is expressible. Every op name
(`add`, `mul`) is a KISS-Ops op name; the region adds *no* new op token.

### 2.4 A worked region — a load-bearing attribute

Now consider a `gather` with a non-default axis and a `clamp` out-of-bounds policy,
over a data operand and a `u32` index operand. The advertisable op is:

- **op_name:** `gather` (a KISS-Ops op name)
- **pattern_attrs:** the KISS-Ops OpAttrs channel carrying `axis = k`, OOB policy
  `clamp`; the **operand-role tuple** `[data: *, index: u32]` — where `data` and
  `index` are KISS-Ops operand-role tokens (§6.6-0005), `*` is the dtype-role wildcard
  meaning "any dtype" (§6.6-0006), and `u32` is a KISS-Classify dtype token
- **synthesis_attrs:** the reference-decomposition pointer, plus emit hints

The axis and OOB policy are **load-bearing values**: a consumer must match them with
an explicit guard. A region that dropped the axis would match *any* gather regardless
of axis — a wrong bind. The `u32` index operand is matched as part of the operand-role
tuple, not by assuming operand-0's dtype; the polymorphic data operand carries the
wildcard so "match a gather over any-dtype data" is expressible with a *pinned* token
rather than an ad-hoc placeholder. This is exactly the case the earlier flat grammar
could not express (it carried no attrs channel and no per-operand role tuple), which is
why those kernels formerly produced no contract; under KISS-Grammar the axis, OOB
policy, and index role live on the pattern-attribute channel, so the region is fully
expressible.

### 2.5 The growth rule — freeze the shape, grow the names

The grammar **freezes shape, not vocabulary.** What freezes: the advertisable-op field
schema (`op_name` + `pattern_attrs` + `synthesis_attrs` + `operand_role_tuple`), the
region-composition rules of §2.2, the attribute-channel structure (the OpAttrs channel,
the extract channel, the guards), and the region wire form of §6.8. What stays
**growing** — deferred entirely to KISS-Ops — is the op **name** set *and every per-op
property that travels with a name*: its commutativity/positionality, its operand roles.

Because an advertisable op *is* "a KISS-Ops op name + attributes," and because Grammar
reads per-op commutativity, positionality, and operand roles from KISS-Ops **by name**
(never from a frozen literal list), a newly-added KISS-Ops op becomes advertisable
simply by **re-basing** — naming it — with no change to the grammar's frozen structural
schema and **no parallel op list to fork**. When KISS-Ops adds a commutative op (say
`hypot` or complex `mul` over `c32`), Grammar canonicalizes it because KISS-Ops
declares it commutative — the commutativity is discovered by name, so no frozen set is
edited and no frozen-shape version bumps. KISS-Grammar and KISS-Ops therefore version
on **independent cadences**: KISS-Ops can add `mish`, `cbrt`, or a new fusion-worthy op
without touching the frozen grammar.

Both the op name *and the KISS-Ops version it is read against* are pinned on the region
(`ops_version`, §6.6-0007): an op's validity and primitiveness are always evaluated
against a definite KISS-Ops version, so two implementations built against different
KISS-Ops versions do not silently disagree on whether a name is valid.

And a non-primitive advertised op stays fully resolvable: a consumer that does not know
it **queries that op's contract** and resolves its KISS-Ops reference decomposition
recursively to the primitive floor. That recursive-resolution termination — inherited
from KISS-Ops — is exactly what lets a frozen grammar admit a still-growing op set.

### 2.6 The seam onto KISS-Contract

KISS-Grammar's output is consumed by KISS-Contract in two places, both STRUCTURAL:

- **Identity.** When the kernel advertises a named op, the contract's `op_identity` field
  is a KISS-Grammar advertisable-op tag re-based onto a KISS-Ops op name (the semantics DAG
  root's op); a hand-written kernel may instead carry a bare KISS-Ops op DAG as its
  `op_identity` with no advertisable-op tag (§6.7-0005). The advertisable tag has a
  pinned **canonical normal form** (§6.1-0007): its identity-bearing attributes (the
  pattern-channel OpAttrs values and the operand-role tuple) are carried at their
  resolved values with defaults made explicit, so two impls agree byte-for-byte on
  whether two tags are the same op identity — a cache-hit-vs-miss decision that must
  not diverge. Contract carries the tag/name, never a private op enum. (`op_identity`
  is one of the *two* orthogonal identities a contract's Identity section joins — the
  other being the `accept_predicate = structure_key`, a KISS-Classify admissibility
  predicate over a cell. A consumer matches both; neither alone is kernel identity.)

- **Semantics.** The op DAG's nodes *are* KISS-Ops op names and KISS-Grammar
  advertisable-op tags at mixed abstraction levels, each carrying the KISS-Ops OpAttrs
  channel. A one-node region maps to a one-node DAG; a fusion maps to its DAG.

Because KISS-Grammar re-bases every advertisable op onto a KISS-Ops name, KISS-Contract
never has to reconcile two op vocabularies: Grammar supplies the advertisable surface +
attribute channels, KISS-Ops supplies the meaning and the floor, and Contract references
both by name/tag over STRUCTURAL edges.

### 2.7 Terms are joined, not restated

KISS-Grammar references the KISS-Ops op **names**, the OpAttrs channel, reference
decompositions, the per-op commutativity/positionality property, the operand-role
vocabulary, and the primitive floor/level by name; and the KISS-Classify dtype tokens
and cell-level op-category tag by name. It re-defines none of them, and it disclaims op
*meaning* entirely — Grammar maps the advertisable surface, KISS-Ops means it.

### 2.8 The wire form — why bytes, not just term syntax

The determinism class of every structural obligation here is **exact byte compare**
(§6.0-0001), and the freeze gate requires a **foreign reader written outside the
reference language** to consume the region-grammar wire form and reproduce the golden
region vectors (§8-0005). Abstract term syntax alone cannot satisfy either: two
conforming implementations could represent the same region with different in-memory
shapes and never agree on bytes. KISS-Grammar therefore pins a normative **region wire
form** (§6.8): a flat, indexed node table prefixed by a fixed **magic** and an in-band
**frozen-shape schema version** (§6.8-0013), with fixed integer widths, a fixed
little-endian byte order, a fixed field order (both region-level, §6.8-0001, and
per-node-record, §6.8-0010), a fixed token encoding, a canonical node-table order with
maximal structural sharing (§6.4-0011, §6.8-0004), a canonical commutative-operand
order (§6.4-0010), and a canonical extract-record order (§6.8-0011). Every golden region
vector in Appendix A carries its exact left-to-right bytes. Only the
recognition-relevant, identity-bearing content is serialized; synthesis attributes
(lowering hints) are not part of the wire form (§6.8-0009), consistent with their being
non-identity-bearing.

---

## 3. Terms & Definitions

- **Advertisable op** — a KISS-Ops op **name** decorated with pattern attributes,
  synthesis attributes, and an operand-role tuple. The op name is the sole identity
  anchor; an advertisable op is never a parallel token independent of KISS-Ops.
- **Advertisable-op tag** — the identity of an advertisable op: its `op_name` together
  with its **identity-bearing** attribute values (the pattern-channel OpAttrs values and
  the operand-role tuple), in the pinned canonical normal form (§6.1-0007) and serialized
  in the dedicated tag canonical serialization (§6.8-0012). Carried by KISS-Contract's
  `op_identity` field. **Several** advertisable tags may share one `op_name`,
  distinguished by their identity-bearing attribute values (§6.1-0003); the tag is the
  FULL identity (name + distinguishing attrs), never the bare name where attributes
  distinguish it. The degenerate case is a bare KISS-Ops op name (no distinguishing
  attributes). Synthesis attributes are **not** part of the tag.
- **op_name** — the KISS-Ops op token an advertisable op re-bases onto; spelled exactly
  as the KISS-Ops op token (§6.6-0001), and valid only against the pinned KISS-Ops
  version (§6.6-0007). The SOLE identity anchor.
- **Pattern attributes** — the recognition/match **channel** (the conceptual sense): the
  KISS-Ops OpAttrs channel (axis / OOB policy / permutation / reduce-axis mask/keepdim), the
  operand-role tuple, the KISS-Ops-declared commutativity property, the consumer-count
  guard, and the node-identity binds. The attributes whose *values* are load-bearing for
  correctness. This channel is a **superset** (§6.2-0001) and is distinct from the narrower
  per-node wire field of the next entry.
- **`pattern_attrs` (per-node wire field)** — the narrow, single-scope field of that name
  carried on an `Op` node of the region node grammar (§2.2, §6.4-0001) and serialized as the
  node-record OpAttrs sub-block (§6.8-0007, §6.8-0010): it carries **only** the KISS-Ops
  **OpAttrs blob** (axis / OOB policy / permutation / reduce-axis mask/keepdim). The other
  members of the conceptual *pattern attributes* channel are carried as **sibling** node
  fields (`operand_role_tuple`, `consumers`) or are read from KISS-Ops by name (the
  commutativity property) or from the bind leaves (the node-identity binds) — they are **not**
  stored inside this `pattern_attrs` field. KISS-Contract's `op_attrs` projection
  (`op_attrs ← pattern_attrs`, Contract §6.4-0009) cites **this** narrow per-node field.
- **Synthesis attributes (`synthesis_attrs`)** — the generation/lowering channel: the
  KISS-Ops reference-decomposition pointer, the exact-vs-approx flavor selection (as
  distinct KISS-Ops names), the scalar-param extract routing, and emit hints
  (`count_unit` / vector width, in-place eligibility, awkward-layout strategy). Not
  identity-bearing; not part of the region wire form.
- **Operand-role tuple (`operand_role_tuple`)** — a per-operand role plus expected dtype
  role (e.g. the `u8` `cond` operand of `select`, the `u32` index operand of
  `gather`/`index_select`), mirroring the KISS-Ops index/address operand role. Role
  tokens are drawn from the KISS-Ops-owned closed operand-role set (§6.6-0005); the
  dtype-role position is a KISS-Classify dtype token or the wildcard token `*`
  (§6.6-0006). The **default** entry is `(data, *)`; its canonical form omits trailing
  default entries (§6.1-0008). Lets a consumer match the mixed-dtype operand tuple, not
  just operand-0's dtype.
- **Dtype-role wildcard (`*`)** — the pinned token occupying the dtype-role position of an
  operand-role entry when the operand's dtype is polymorphic (any dtype); a Grammar-owned
  structural sentinel that is part of the frozen shape and is never a KISS-Classify dtype
  token (§6.6-0006).
- **Level (inherited)** — the primitive-floor membership and decomposition level of an
  op, **owned by KISS-Ops** and inherited by an advertisable op; KISS-Grammar never
  re-asserts it.
- **OpAttrs channel** — the per-node attribute vocabulary (axis, out-of-bounds policy,
  permutation, reduce-axis mask/keepdim) **owned by KISS-Ops** as op semantics; its
  sub-vocabularies are cited, never re-defined, by KISS-Grammar (§6.2-0006), which surfaces
  them as pattern attributes and carries them per node in a region uninterpreted at the
  framing level. Its wire bytes are the KISS-Ops OpAttrs encoding — default-resolved and
  canonical at the KISS-Ops layer (§8-0008) — embedded length-prefixed (§6.8-0007).
  Distinct from KISS-Grammar's own matching hints (node-identity binds, consumer-count
  guard, commutativity canonicalization key).
- **Advertisable region (region)** — a single-rooted DAG of advertisable-op nodes over
  bind leaves, carried as the flat indexed node table of §6.8 whose operands are node
  indices (so interior sharing is a shared index). A one-node region is a single
  advertisable op; a fusion is a multi-node region whose root carries the region's
  advertised identity.
- **Root** — the single node that no other in-region node references (equivalently, the
  last entry in the canonical node table); carries the region's advertised identity and
  `consumers = ROOT`.
- **Bind leaf** — a leaf `Bind(i)` binding the producer at that leaf to the region's
  positional input `i`. Carries no `consumers` field.
- **Node-identity guard** — a repeated bind of the same input index, asserting that two
  subtrees read literally the same input (a shared operand); canonicalizes to one shared
  node-table entry (§6.4-0011).
- **Fusion-safety guard (no-external-consumer guard)** — the `consumers` enum on an `Op`
  node counting **only** edges leaving the region: an interior node must carry
  `consumers = INTERIOR` (external-consumer count 0); the root carries `consumers = ROOT`.
  In-region sharing is never counted.
- **Commutativity property (KISS-Ops-owned)** — the per-op property, declared by KISS-Ops
  and read here by name, stating whether an op's operands may be canonicalized
  (commutative) or must stay positional. Canonicalization is for reproducible emission,
  not for match correctness. There is **no** Grammar-owned literal commutative-op list.
- **Canonical operand order** — the pinned total order (§6.4-0010) over the operands of a
  commutative node: ascending unsigned-byte-lexicographic comparison of each operand's
  canonical subtree serialization, made total by mandatory structural dedup (§6.4-0011)
  so equal-key operands are the same node index. Makes commutative reorderings emit
  byte-identically.
- **Structural dedup (common-subexpression sharing)** — the canonical-form requirement
  (§6.4-0011) that any two nodes with byte-identical canonical subtree serialization are
  one shared node-table entry; guarantees the node table and the operand-order tie-break
  are byte-exact.
- **Extract channel** — the scalar-runtime-param routing anchored on the region root: a
  list of `(path, param_slot)` where a **path** is a sequence of canonical operand
  indices from the root over the **canonicalized** tree (the lexicographically-least such
  sequence for a shared target, §6.4-0007) and `param_slot` is a 0-based integer; records
  emitted in ascending `param_slot` order (§6.8-0011).
- **Declared region arity (`n_inputs`)** — the region-level field giving the number of
  positional inputs the region binds; the expressibility check is against this declared
  value, never an inferred `max(index)+1`.
- **Expressible region** — a region whose bound-input set equals exactly `[0, n_inputs)`
  and that has exactly one root.
- **Region wire form** — the pinned byte serialization of a region (§6.8): a fixed magic
  and in-band frozen-shape schema version prefix (§6.8-0013), then fixed field order,
  fixed integer widths and little-endian byte order, length-prefixed UTF-8 tokens, a fixed
  per-node-record layout (§6.8-0010), canonical node-table order, canonical
  commutative-operand order, and canonical extract-record order. The artifact against
  which byte-exact determinism and the foreign-reader freeze gate are evaluated.
- **Tag canonical serialization** — the dedicated identity-bearing-only byte form of an
  advertisable-op tag (§6.8-0012): `op_name` + OpAttrs blob + canonical operand-role
  tuple, carrying **none** of the region framing (no kind tag, consumers, operand_count,
  operand indices, version tokens, or magic/version header). Tag equality is byte-exact
  comparison of this form.
- **Frozen-shape schema version** — the KISS-Grammar wire/shape version, stamped in-band
  in the region wire form (§6.8-0013) and distinct from the upstream `ops_version` /
  `classify_version` bindings; the axis KISS-Conform keys on for frozen-shape conformance.
- **Upstream version binding** — the `ops_version` / `classify_version` fields pinned on a
  region (§6.6-0007) carrying the **upstream wire/ABI schema-version** of KISS-Ops /
  KISS-Classify (umbrella §5.1) that fix which version its op names and dtype tokens are
  read against; compared byte-exact on the token string.
- **Re-basing** — declaring an advertisable op by naming an existing or newly-added
  KISS-Ops op, adding attributes, and changing nothing in the frozen structural schema.
- **Frozen shape** — the advertisable-op field schema, the region-composition rules, the
  attribute-channel structure, and the region wire form; frozen at the freeze gate,
  versioned independently of the KISS-Ops op-name set.
- **op-category (cell-level op-family tag)** — a coarse KISS-Classify category (a component
  of `structure_key`); distinct from a KISS-Ops op name and used by name only here.
- **Typed decline** — a structured refusal returned in lieu of a result (a distinguished
  error value/enumerant, or an equivalent out-of-band error return); never a panic,
  abort, crash, hang, or out-of-bounds read.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point semantics; referenced transitively through KISS-Ops
  (KISS-Grammar defines no numeric behavior of its own).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword
  convention, the normative/informative split, the clause-ID scheme and 1:1 test mapping,
  value pinning as bits/IEEE-754 in wire order, the "bytes on the wire, left to right"
  golden-vector requirement (umbrella §4.3), the ban on unquantified adjectives, the two
  version axes, the ≥2-dissimilar-implementations-plus-foreign-reader freeze gate, the
  capability/profile/extension model, governance, licensing, and patent posture. **Stated
  once in the umbrella; referenced here; never restated.** This sub-standard's §5 points at
  umbrella §3 for conventions.
- **KISS-Ops** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency:
  every advertisable op re-bases onto a KISS-Ops op **name**; the OpAttrs channel
  (axis / OOB policy / permutation / reduce-axis) and its **default-resolved, canonical
  wire encoding** (§8-0008), the reference decompositions (the resolution oracle), the
  exact-vs-approx flavor spellings (`gelu` vs `gelu_tanh`, `relu` vs `fmax_ieee`,
  `rem_floor` vs `rem_trunc`), the **per-op commutativity / positionality property**, the
  **operand-role token vocabulary** (index/address/cond/data …), and the op level /
  primitive-floor membership are all **owned by KISS-Ops** and used here by name/tag
  against a pinned KISS-Ops version. KISS-Grammar re-defines none of them and defines no
  op meaning.
- **KISS-Classify** (by version) — DAG edge labeled **STRUCTURAL**, **upstream**
  dependency: the dtype **tokens** naming an operand-role tuple's expected dtype roles
  (`u8`, `u32`, …) and the cell-level **op-category** tag are KISS-Classify vocabulary,
  used here by name only and against a pinned KISS-Classify version.
- **KISS-Contract** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: the contract Identity `op_identity` field carries a KISS-Grammar
  advertisable-op tag (in its canonical normal form) re-based onto a KISS-Ops op name; the
  contract Semantics op DAG's nodes are advertisable-op tags and KISS-Ops op names at mixed
  levels, each carrying the KISS-Ops OpAttrs channel. This is the direct STRUCTURAL
  prerequisite-closure edge the umbrella §2.2 edge table pins from KISS-Grammar.
- **KISS-Consume** (by version) — **naming reference** (informative), **downstream**:
  recognition lift targets are named as advertisable-op tags over KISS-Ops names. This is a
  naming reference to the advertisable surface, **not** a prerequisite-closure edge in the
  umbrella §2.2 edge table.
- **KISS-Emit** (by version) — **naming reference** (informative), **downstream**: the
  advertisable surface names the ops an emitter lowers. This is a naming reference to the
  advertisable surface, **not** a prerequisite-closure edge in the umbrella §2.2 edge table.
- **KISS-Conform** (by version) — depends on and tests KISS-Grammar; owns the fuzzer and
  differential harness that exercise the region grammar, the wire form, and the growth
  rule.

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase
keywords `MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance and
consumer-behavior guidance and never state a structural requirement. Informative §0–§5
prose uses lowercase requirement language; uppercase RFC-2119 keywords appear only in §6+
or where a keyword is named as a term. Every atomic requirement carries a stable,
append-only ID `KISS-GRAMMAR-<section>-<nnnn>`, allocated by the editor of record, never
reused after retirement, and mapped 1:1 to ≥1 named KISS-Conform test. Values are pinned
as tokens/schema spelled exactly as the upstream foundational vocabularies pin them, never
as one source language's surface spelling, and serialized in the pinned wire form (§6.8).
Unquantified adjectives ("well-formed", "reasonable", "neutral", "valid") are banned from
normative text. Every clause declares its determinism/fidelity class so KISS-Conform
selects the correct comparator. A small number of clauses are explicitly marked
**aggregate** wire-encoding/normal-form obligations (§6.1-0007, §6.4-0004, §6.8-0002):
they pin a set of jointly-attested field encodings under one ID and one mapped test, a
deliberate exception to strict atomicity noted at each such clause. See umbrella §3 for
the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

- **KISS-GRAMMAR-6.0-0001** — Every structural obligation in §6–§8 (the advertisable-op
  field schema, the region node grammar, the composition rules, the attribute-channel
  structure, the region wire form of §6.8, and every token spelling) is
  determinism-class **exact byte compare**, evaluated over the **region wire form**
  (§6.8); KISS-Conform MUST compare the pinned wire bytes with a byte-exact comparator and
  MUST NOT apply tolerance or order-invariant comparison. KISS-Grammar defines no numeric
  result of its own; the numeric determinism class of any op it names is **owned by
  KISS-Ops** (the single canonical enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}`, KISS-OPS §6.0-0001) and MUST NOT be re-forked here.
  *Test:* `test_grammar_determinism_class_exact_byte`.

### 6.1 The advertisable-op surface — field schema and the no-fork rule

- **KISS-GRAMMAR-6.1-0001** — An advertisable op MUST consist of exactly the field schema
  `{op_name, pattern_attrs, synthesis_attrs, operand_role_tuple}`, where `op_name` is a
  KISS-Ops op name, `pattern_attrs` is the recognition/match channel (§6.2),
  `synthesis_attrs` is the generation/lowering channel (§6.3), and `operand_role_tuple`
  is the per-operand role tuple (§6.1-0005); an implementation MUST NOT add **any** fifth
  field to this schema at this schema version (any added field is a frozen-shape change per
  §8-0002 and is excluded from the wire form of §6.8). *Test:*
  `test_grammar_advertisable_op_field_schema`.
- **KISS-GRAMMAR-6.1-0002** — `op_name` MUST be the **sole identity anchor** of an
  advertisable op: an advertisable op is a KISS-Ops op name decorated with attributes,
  and an implementation MUST NOT define an independent or forkable advertisable-op token
  set — every advertised op MUST re-base onto a KISS-Ops op name, and no advertisable-op
  token that is not a KISS-Ops op name may exist. *Test:*
  `test_grammar_no_parallel_op_list`.
- **KISS-GRAMMAR-6.1-0003** — A single KISS-Ops op name MAY back **several** KISS-Grammar
  advertisable tags: KISS-Grammar MUST permit **many** advertisable tags per KISS-Ops op
  name, distinguished by their **identity-bearing** attribute values (the pattern-channel
  OpAttrs values and the operand-role tuple, §6.1-0007). An advertisable-op **tag** — the
  `op_identity` a KISS-Contract carries — MUST be the **full** tag: its `op_name`
  **together with** the identity-bearing attribute values that distinguish it, and an
  implementation MUST NOT reduce it to the bare `op_name` where those attributes
  distinguish it from another advertisable tag over the same name. The many-to-one
  relation is first-class, not an exception: KISS-Grammar MUST NOT constrain a KISS-Ops op
  name to at most one advertisable tag. *Test:* `test_grammar_tag_is_name_plus_attrs`.
- **KISS-GRAMMAR-6.1-0004** — An advertisable op's **level** — its primitive-floor
  membership and decomposition level — MUST be inherited from KISS-Ops and MUST NOT be
  re-declared or overridden by KISS-Grammar. *Test:* `test_grammar_level_inherited_not_redeclared`.
- **KISS-GRAMMAR-6.1-0005** — The `operand_role_tuple` MUST carry, per operand, the
  operand's **role token** (drawn from the KISS-Ops-owned operand-role set, §6.6-0005) and
  its **expected dtype role** (a KISS-Classify dtype token, or the wildcard `*` of
  §6.6-0006 for a polymorphic operand) — mirroring the KISS-Ops index/address operand role
  (e.g. the `u8` `cond` operand of `select`, the `u32` index operand of
  `gather`/`index_select`) — so that a consumer matches the mixed-dtype operand tuple and
  MUST NOT infer every operand's dtype from operand-0's dtype. *Test:*
  `test_grammar_operand_role_tuple`.
- **KISS-GRAMMAR-6.1-0006** — KISS-Grammar MUST NOT define, restate, or override any op's
  **semantics** (NaN propagation, signed-zero behavior, IEEE-`fmax`-vs-NaN-propagating-max,
  wrapping-integer behavior, raw-bit `select`, rounding, declared-ULP ceilings, per-op
  commutativity/positionality, operand roles, or a reference decomposition); those are
  resolved **from** KISS-Ops. KISS-Grammar maps the advertisable surface onto KISS-Ops
  names and attribute channels only. *Test:* `test_grammar_defines_no_op_semantics`.
- **KISS-GRAMMAR-6.1-0007** — An advertisable-op **tag** MUST have a single **canonical
  normal form** used for tag equality (an intentional **aggregate** normal-form clause:
  sub-requirements (a)–(c) are jointly attested by the single mapped test): (a) the
  **identity-bearing** attributes are exactly the pattern-channel OpAttrs values
  (§6.2-0001) and the `operand_role_tuple` (in its canonical form, §6.1-0008), together
  with `op_name`, and nothing else — `synthesis_attrs` are **not** identity-bearing and
  MUST NOT appear in the tag; (b) every identity-bearing attribute MUST be carried at its
  **resolved** value with defaults made **explicit**, and this default resolution for the
  OpAttrs channel is owned **upstream**: the embedded KISS-Ops OpAttrs wire encoding is
  itself default-resolved and canonical (KISS-Ops emits every defaulted attribute
  explicitly; §6.2-0006, §8-0008), so a defaulted attribute and an explicitly-stated equal
  value produce the **same** OpAttrs bytes and KISS-Grammar canonicalizes **without**
  interpreting the opaque OpAttrs sub-vocabulary (defaults MUST NOT be elided); (c) the tag
  MUST be serialized in the dedicated **tag canonical serialization** of §6.8-0012 (which
  carries only the identity-bearing fields of (a) and none of the region framing), and two
  tags MUST be compared for identity by **byte-exact** comparison of that serialization. An
  implementation MUST NOT treat two tags that differ only by an elided-vs-explicit default
  as distinct, nor two tags with different identity-bearing values as equal. *Test:*
  `test_grammar_canonical_tag_normal_form`.
- **KISS-GRAMMAR-6.1-0008** — The `operand_role_tuple` MUST have a single **canonical
  form** for tag identity and wire emission: the **default** operand-role entry is exactly
  `(data, *)` (role token `data`, dtype-role wildcard `*`), and an operand carrying that
  entry is **unconstrained**. The canonical tuple MUST encode exactly `k` entries, where
  `k` is `0` if every operand is unconstrained, else `1 +` the greatest operand index whose
  entry differs from `(data, *)`; every operand with index `≥ k` MUST be taken to be
  `(data, *)`, and trailing default entries MUST NOT be emitted. Two operand-role tuples
  MUST compare equal iff their default-filled forms (each padded to the node's
  `operand_count` with `(data, *)`) are equal; an implementation MUST NOT treat a `0`-entry
  tuple as distinct from an all-`(data, *)` tuple over the same operand count. *Test:*
  `test_grammar_operand_role_tuple_canonical_form`.

### 6.2 Pattern attributes — the recognition / match channel

- **KISS-GRAMMAR-6.2-0001** — `pattern_attrs` MUST carry the KISS-Ops **OpAttrs** channel
  (the `reduce`/`gather`/`scan` axis, the out-of-bounds policy drawn from the KISS-Ops set
  `{skip, clamp, zero-fill}`, the permutation/`perm` guard for layout ops, and the
  reduce-axis mask/keepdim), **owned by KISS-Ops** and referenced here — never re-defined —
  plus the operand-role tuple (§6.1-0005), the KISS-Ops-declared commutativity property
  (§6.4-0005), the consumer-count guard (§6.4-0004), and the node-identity binds
  (§6.4-0003). *Test:* `test_grammar_pattern_attrs_carry_opattrs`.

  > **Scope note (informative) — two senses of "pattern attributes."** This clause defines
  > *pattern attributes* as the whole **recognition/match channel** — a **superset**: the
  > OpAttrs channel **plus** the operand-role tuple, the commutativity property, the
  > consumer-count guard, and the node-identity binds. That conceptual channel is **distinct
  > from** the per-node **wire field** named `pattern_attrs` in the region node grammar
  > (§2.2, §6.4-0001), which carries **only** the KISS-Ops **OpAttrs blob** and is serialized
  > as the node-record OpAttrs sub-block (§6.8-0007, §6.8-0010). In the node record the other
  > members of this channel are separate **sibling** fields (`operand_role_tuple`,
  > `consumers`) or are not stored per node at all (the commutativity property is read from
  > KISS-Ops by name; the node-identity binds live on the `Bind` leaves). KISS-Contract's
  > op-graph node projection (`op_attrs ← pattern_attrs`, Contract §6.4-0009) cites the
  > **narrow, single-scope** per-node field, not this superset.
- **KISS-GRAMMAR-6.2-0002** — A pattern attribute whose **value** is load-bearing for
  correctness (a `transpose`/`permute` `perm`, a `gather`'s axis, an out-of-bounds policy,
  a reduce-axis mask) MUST be matched with an **explicit guard**; an implementation MUST
  NOT skip such an attribute (no see-through of a load-bearing attribute), because a
  region that dropped the value would match a structurally-similar op with a different
  value — a wrong bind. *Test:* `test_grammar_load_bearing_attr_guarded`.
- **KISS-GRAMMAR-6.2-0003** — Recognition of an advertisable op or region MUST be
  **structural** — over the region DAG, the per-node OpAttrs values, and the operand-role
  tuple — and MUST NOT rely on op-name substring matching, keyword sniffing, or any
  spelling-based heuristic. *Test:* `test_grammar_recognition_is_structural`.
- **KISS-GRAMMAR-6.2-0004** — The out-of-bounds policy surfaced on `pattern_attrs` MUST be
  exactly one of the KISS-Ops-owned set `{skip, clamp, zero-fill}` for reads (and `skip`
  for writes), spelled as KISS-Ops spells it; KISS-Grammar MUST NOT define an alternative
  OOB policy vocabulary. *Test:* `test_grammar_oob_policy_from_ops`.
- **KISS-GRAMMAR-6.2-0005** — The KISS-Ops-declared commutativity property surfaced on
  `pattern_attrs` MUST be used for canonicalization only (§6.4-0005) and MUST NOT alter
  which op a node matches; it is a canonicalization key, not an identity field, and MUST
  be read from KISS-Ops by name rather than from any Grammar-owned literal. *Test:*
  `test_grammar_commutativity_is_canonicalization_only`.
- **KISS-GRAMMAR-6.2-0006** — The OpAttrs **sub-vocabularies** surfaced on `pattern_attrs`
  — the **out-of-bounds policy** set (`{skip, clamp, zero-fill}` for reads, `skip` for
  writes), the **permutation** encoding, and the **reduce-axis mask / keepdim** encoding —
  are **owned by KISS-Ops** as op semantics; KISS-Grammar MUST cite KISS-Ops as the single
  owner of each — KISS-OPS **§6.19-0001** pins KISS-Ops as the **single normative owner** of
  the OpAttrs channel, with the individual sub-vocabularies frozen at KISS-OPS **§6.19-0015**
  (oob_policy), **§6.19-0020** (reduce_axes mask/keepdim multiplex), and **§6.19-0024**
  (permutation, RESERVED this version) — and MUST NOT re-define, restate, or fork any of
  them, and MUST carry each uninterpreted at the framing level (embedded as the KISS-Ops
  OpAttrs wire bytes, §6.8-0007), inventing no encoding of its own for any of them. The
  embedded KISS-Ops OpAttrs wire bytes are **default-resolved and canonical at the KISS-Ops
  layer** (KISS-Ops emits every defaulted attribute explicitly; KISS-OPS §6.19-0005, §8-0008);
  KISS-Grammar therefore attains
  tag/region byte-equality over these bytes **without** interpreting the sub-vocabulary,
  and MUST NOT perform default normalization of its own on the OpAttrs blob. KISS-Grammar's
  own `pattern_attrs` **matching hints** — the node-identity binds (§6.4-0003), the
  consumer-count fusion-safety guard (§6.4-0004), and the commutativity canonicalization
  key (§6.2-0005) — are **DISTINCT** from these KISS-Ops-owned sub-vocabularies and MUST
  NOT be conflated with them. Where KISS-Ops does not yet normatively pin one of these
  sub-vocabularies, KISS-Grammar MUST NOT supply a substitute definition; it carries only
  what KISS-Ops defines. *Test:* `test_grammar_opattrs_subvocab_owned_by_ops`.

### 6.3 Synthesis attributes — the generation / lowering channel

- **KISS-GRAMMAR-6.3-0001** — For a **non-primitive** advertisable op, `synthesis_attrs`
  MUST carry the KISS-Ops **reference-decomposition pointer** (the resolution oracle by
  which the op resolves recursively to the KISS-Ops primitive floor); the decomposition
  itself is owned by KISS-Ops and MUST NOT be restated here. *Test:*
  `test_grammar_synthesis_carries_decomposition_pointer`.
- **KISS-GRAMMAR-6.3-0002** — An exact-vs-approximate flavor distinction MUST be expressed
  as **distinct KISS-Ops op names** (e.g. `gelu` = exact-erf vs `gelu_tanh` =
  approximation; `relu` vs `fmax_ieee`; `rem_floor` vs `rem_trunc`) and MUST NOT be
  expressed as a free-form approximation/precision flag on `synthesis_attrs`. *Test:*
  `test_grammar_flavor_is_distinct_op_name`.
- **KISS-GRAMMAR-6.3-0003** — Scalar-runtime-param routing carried on `synthesis_attrs`
  MUST use the extract channel anchored on the region root (§6.4-0007) — a
  `path-to-node → param slot` mapping whose path and slot encodings are pinned in
  §6.4-0007 / §6.8-0006 — and MUST NOT be encoded as an additional op-graph node. *Test:*
  `test_grammar_synthesis_scalar_param_extract`.
- **KISS-GRAMMAR-6.3-0004** — Emit hints carried on `synthesis_attrs` (`count_unit` /
  vector width, in-place eligibility, awkward-layout strategy) MUST parameterize **how the
  op is lowered** and MUST NOT be treated as match-channel attributes; they are consumed on
  the build-on-miss (provision) path. *Test:* `test_grammar_synthesis_emit_hints`.
- **KISS-GRAMMAR-6.3-0005** — An attribute value that is load-bearing for the **correctness
  of a match** (§6.2-0002) MUST reside on `pattern_attrs`, and an attribute that only
  parameterizes lowering MUST reside on `synthesis_attrs`; an implementation MUST NOT place
  a match-load-bearing value on the synthesis channel where it would be invisible to
  recognition and to the identity-bearing tag (§6.1-0007). *Test:*
  `test_grammar_channel_separation`.

### 6.4 The region grammar — composition

- **KISS-GRAMMAR-6.4-0001** — An advertisable region MUST be a **single-rooted DAG** of
  advertisable-op nodes over bind leaves, represented as a flat, ordered node table whose
  node grammar is
  `Op { op_name, pattern_attrs, operand_role_tuple, operands, consumers } | Bind(input_index)`,
  where the per-node `pattern_attrs` field is the **narrow** field carrying only the KISS-Ops
  OpAttrs blob (§6.8-0007, §6.8-0010) — the `operand_role_tuple` and `consumers` are the
  sibling fields shown, not sub-parts of `pattern_attrs`; the broader superset sense of
  "pattern attributes" (§6.2-0001) is conceptual only — and
  where each element of a node's `operands` is an **index** referencing a strictly-earlier
  entry in the table (so a node index appearing in more than one parent's `operands`
  denotes a single shared interior node — a genuine DAG, not only a tree), and the region
  additionally carries the region-level fields `n_inputs` (§6.4-0008), `ops_version` /
  `classify_version` (§6.6-0007), and the root-anchored `extract` list (§6.4-0007). An
  implementation MUST NOT admit a region node that is neither an `Op` node nor a `Bind`
  leaf, and MUST NOT admit an operand index that does not reference a strictly-earlier
  table entry, at this schema version. *Test:* `test_grammar_region_is_single_rooted_dag`.
- **KISS-GRAMMAR-6.4-0002** — A **one-node** region MUST denote a single advertisable op,
  and a **multi-node** region (a fusion) MUST carry its advertised identity on its **root**
  node — the single node referenced by no other in-region node (the last entry in the
  canonical node table, §6.8-0004); a consumer MUST take the region's advertised
  `op_identity` from the root op, not from an interior node. *Test:*
  `test_grammar_root_carries_identity`.
- **KISS-GRAMMAR-6.4-0003** — A `Bind(i)` leaf MUST reference the region's positional
  input `i`, and a **repeated** `Bind` of the same index MUST be treated as the
  node-identity guard (the two leaves assert one and the same input); an implementation
  MUST NOT treat two `Bind(i)` leaves of the same index as two distinct inputs. All `Bind`
  leaves of the same `input_index` MUST canonicalize to exactly **one** node-table entry,
  referenced by index wherever that input is bound (structural dedup, §6.4-0011); an
  implementation MUST NOT emit two separate table entries for the same input index. *Test:*
  `test_grammar_repeated_bind_is_node_identity`.
- **KISS-GRAMMAR-6.4-0004** — The `consumers` field of an `Op` node MUST count **only edges
  that leave the region** (consumers of the node's result in the surrounding candidate
  graph that are not themselves inside the region); it MUST NOT count in-region edges. An
  **interior** (non-root) node MUST carry `consumers = INTERIOR`, which requires its
  external-consumer count to be **0**, and an implementation MUST NOT fuse an interior node
  whose result is consumed outside the region. The **root** node MUST carry
  `consumers = ROOT`. A `Bind` leaf MUST carry no `consumers` field. In-region sharing of
  an interior node by two or more in-region parents MUST NOT be treated as an external
  consumer and MUST NOT violate the guard. The `consumers` field is an enum with the wire
  encoding pinned in §6.8-0005. (This clause is an intentional **aggregate** fusion-safety
  obligation; its sub-requirements are jointly attested by the single mapped test.) *Test:*
  `test_grammar_interior_no_external_consumer_guard`.
- **KISS-GRAMMAR-6.4-0005** — For an op that KISS-Ops declares **commutative** (the
  property read from KISS-Ops by name per §6.2-0005, **not** from any Grammar-owned literal
  list), that node's operands MUST be canonicalized to the total **canonical operand order**
  of §6.4-0010 for **reproducible emission**; this canonicalization MUST NOT be relied upon
  for **match correctness** (both the imported region and the candidate graph MUST
  canonicalize before matching, so any single emitted operand order matches). An
  implementation MUST NOT hardcode a commutative-op set: any op KISS-Ops declares commutative
  MUST be canonicalized, and no op KISS-Ops declares positional MUST be. *Test:*
  `test_grammar_commutative_canonicalization`.
- **KISS-GRAMMAR-6.4-0006** — For an op that KISS-Ops declares **positional** (comparisons
  and `select`, whose operand order is load-bearing — `select` is `(cond, a, b)` per
  KISS-Ops), the node MUST be treated as strictly positional, and an implementation MUST NOT
  commutatively reorder its operands. Positionality, like commutativity (§6.4-0005), MUST be
  read from KISS-Ops by name and MUST NOT be a Grammar-owned literal list. *Test:*
  `test_grammar_comparison_select_positional`.
- **KISS-GRAMMAR-6.4-0007** — Scalar runtime parameters MUST ride the **extract** channel
  anchored on the region **root** as a list of `(path, param_slot)`, where a **path** is a
  sequence of **canonical operand indices** from the root down to the target node evaluated
  over the **already-canonicalized** tree, and `param_slot` is a 0-based unsigned integer.
  Canonicalization (§6.4-0005) MUST occur **before** extract paths are formed, so an extract
  path always addresses the node it names under the canonical order. When a target node is
  shared (reachable from the root by more than one sequence of canonical operand indices —
  a genuine DAG), the **canonical** extract path MUST be the **lexicographically-least** such
  sequence under unsigned comparison of the step indices, and an implementation MUST NOT
  emit a non-least path for a shared target. An implementation MUST anchor extract paths on
  the root and MUST NOT anchor them on an interior node, and MUST NOT form an extract path
  over a pre-canonical operand order. The path/slot wire encoding is pinned in §6.8-0006 and
  the record order in §6.8-0011. *Test:* `test_grammar_extract_channel_on_root`.
- **KISS-GRAMMAR-6.4-0008** — A region MUST carry an explicit **declared arity** field
  `n_inputs` (its wire encoding pinned in §6.8-0001/§6.8-0002); a region is **expressible**
  only when its bound-input set equals exactly `[0, n_inputs)`. A region that binds a strict
  subset of `[0, n_inputs)`, or an index `≥ n_inputs`, MUST yield a **typed decline** (never
  a panic) and MUST NOT be emitted as an expressible region. `n_inputs` MUST NOT be inferred
  as `max(bound index) + 1` (which would make a strict-subset bind undetectable); it MUST be
  the region's declared field. *Test:* `test_grammar_bind_set_covers_inputs`.
- **KISS-GRAMMAR-6.4-0009** — A region MUST have **exactly one** root (be single-output);
  a multi-output computation (a forest of distinct output roots) is not an expressible
  region and MUST yield a typed decline rather than deriving a region from one output alone.
  *Test:* `test_grammar_single_output_region`.
- **KISS-GRAMMAR-6.4-0010** — The **canonical operand order** of a commutative node's
  operands MUST be the ascending order under **unsigned-byte lexicographic comparison** of
  each operand's **canonical subtree serialization**, where the canonical subtree
  serialization of a node is: for a `Bind(i)` leaf, the byte `0x00` followed by `input_index`
  as a `u32` little-endian; for an `Op` node, the byte `0x01` followed by its length-prefixed
  `op_name`, its `consumers` byte, its length-prefixed OpAttrs blob, its operand-role tuple
  block, and the concatenated canonical subtree serializations of its operands **in this same
  canonical order** (recursively); a shared node contributes its subtree serialization once
  per referencing position. Comparison MUST be over these bytes. Because the canonical region
  form requires structural common-subexpression dedup (§6.4-0011), two **distinct**
  node-table entries cannot share a byte-identical subtree serialization, so a tie under this
  comparison arises only between references to the **same** node index, whose emitted operand
  indices are equal; as a defensive total order, equal keys MUST break ties by ascending
  referenced node-table index. Every conforming implementation MUST produce the identical
  operand permutation. *Test:* `test_grammar_canonical_operand_order_key`.
- **KISS-GRAMMAR-6.4-0011** — The canonical region form MUST apply **maximal structural
  sharing (common-subexpression dedup)**: any two nodes whose canonical subtree
  serialization (§6.4-0010) is byte-identical MUST be represented by a **single** node-table
  entry, referenced by index at every occurrence; in particular all `Bind` leaves of the
  same `input_index` (§6.4-0003) coalesce to exactly one entry, and two structurally-identical
  `Op` subtrees coalesce to one entry. An implementation MUST NOT emit two distinct table
  entries with byte-identical canonical subtree serializations. This makes the operand-order
  tie-break of §6.4-0010 total and the node table byte-exact. *Test:*
  `test_grammar_structural_dedup_and_repeated_bind`.

### 6.5 The growth rule — frozen shape, growing op-name set

- **KISS-GRAMMAR-6.5-0001** — Adding an advertisable op MUST be done by **re-basing** onto
  a KISS-Ops op name (existing or newly added in KISS-Ops), decorating it with attributes,
  and changing **nothing** in the frozen advertisable-op field schema (§6.1), the
  region-composition rules (§6.4), or the region wire form (§6.8); an implementation MUST NOT
  add an advertisable op by any other mechanism. *Test:* `test_grammar_add_op_by_rebasing`.
- **KISS-GRAMMAR-6.5-0002** — Adding an advertisable op by re-basing MUST NOT introduce a
  parallel or forkable op token and MUST NOT require a KISS-Grammar frozen-shape schema
  version bump (it is additive under the growth rule). Because per-op commutativity,
  positionality, and operand roles are read from KISS-Ops by name (§6.2-0005, §6.4-0005,
  §6.4-0006, §6.6-0005), a KISS-Ops op that introduces such a property MUST be honored
  without editing any Grammar literal and without a frozen-shape bump. *Test:*
  `test_grammar_rebasing_is_additive_no_bump`.
- **KISS-GRAMMAR-6.5-0003** — The KISS-Grammar frozen-shape schema version and the KISS-Ops
  op-name set MUST version on **independent** axes; a KISS-Ops op-set addition (including one
  that carries a new commutativity/positionality or operand-role property) MUST NOT bump the
  KISS-Grammar frozen-shape schema version, and a KISS-Grammar frozen-shape change MUST NOT
  be taken to imply a KISS-Ops change. *Test:* `test_grammar_independent_cadence`.
- **KISS-GRAMMAR-6.5-0004** — A **non-primitive** advertisable op MUST remain fully
  resolvable: a consumer that does not natively know the op MUST be able to query that op's
  contract and resolve its KISS-Ops reference decomposition **recursively** to the KISS-Ops
  primitive floor; the termination guarantee is inherited from KISS-Ops (acyclic,
  strictly-decreasing level) and MUST NOT be weakened by KISS-Grammar. *Test:*
  `test_grammar_nonprimitive_resolvable_to_floor`.
- **KISS-GRAMMAR-6.5-0005** — The **frozen shape** MUST be exactly the union of (a) the
  advertisable-op field schema (§6.1-0001), (b) the region-composition rules (§6.4),
  (c) the attribute-channel structure (the OpAttrs channel, the extract channel, and the
  guards of §6.2–§6.3), and (d) the region wire form (§6.8); a change to any member of this
  union — but **not** a change to the KISS-Ops op-name set or to any KISS-Ops-owned per-op
  property (commutativity, positionality, operand roles) — MUST bump the KISS-Grammar
  frozen-shape schema version (§8-0002). *Test:* `test_grammar_frozen_shape_membership`.

### 6.6 Vocabulary re-basing — upstream token spelling

- **KISS-GRAMMAR-6.6-0001** — Every `op_name` MUST be spelled **exactly** as the KISS-Ops
  op token (case-sensitive, underscore-delimited); an implementation MUST NOT accept a
  synonym, alias, or alternate spelling as a KISS-Ops op name. *Test:*
  `test_grammar_op_name_spelling`.
- **KISS-GRAMMAR-6.6-0002** — Every dtype role in an `operand_role_tuple` MUST be spelled
  **exactly** as a KISS-Classify dtype token (e.g. `u8`, `u32`, `f32`) **or** as the
  dtype-role wildcard token `*` (§6.6-0006); an implementation MUST NOT substitute a synonym
  or alternate casing for a dtype token, and MUST NOT spell a polymorphic dtype role as
  anything other than the pinned wildcard. *Test:* `test_grammar_operand_dtype_token_spelling`.
- **KISS-GRAMMAR-6.6-0003** — Where an advertisable op references a cell-level
  **op-category**, that category MUST be a KISS-Classify op-family tag (a coarse Classify
  category, a distinct closed set from the KISS-Ops op name), spelled as KISS-Classify
  spells it; an implementation MUST NOT conflate the cell-level op-category with the
  `op_name`. *Test:* `test_grammar_op_category_is_classify_tag`.
- **KISS-GRAMMAR-6.6-0004** — A referenced `op_name` MUST correspond to an op present in
  the KISS-Ops op set of the **KISS-Ops version pinned on the region** (`ops_version`,
  §6.6-0007); op-name validity **and** primitiveness MUST be evaluated against that pinned
  version. An implementation MUST NOT advertise an `op_name` that names no KISS-Ops op in
  the pinned version (there is no Grammar-local op vocabulary to fall back on); an
  unrecognized name MUST yield a typed decline. *Test:* `test_grammar_op_name_exists_in_ops`.
- **KISS-GRAMMAR-6.6-0005** — Every operand **role token** in an `operand_role_tuple` MUST
  be drawn from the **KISS-Ops-owned closed operand-role set** (e.g. `data`, `cond`,
  `index`, `address` — spelled exactly as KISS-Ops spells them, owner = KISS-Ops); an
  implementation MUST NOT define a Grammar-local operand-role vocabulary, MUST NOT accept a
  role synonym or alternate casing, and MUST NOT use a role token absent from the KISS-Ops
  operand-role set of the pinned KISS-Ops version. *Test:*
  `test_grammar_operand_role_token_vocabulary`.
- **KISS-GRAMMAR-6.6-0006** — The dtype-role position of an operand-role entry MUST be
  either a KISS-Classify dtype token (§6.6-0002) or the **dtype-role wildcard** token `*`,
  which denotes a polymorphic (any-dtype) operand. The wildcard `*` is a Grammar-owned
  structural sentinel that is part of the frozen shape (§6.5-0005), is spelled exactly as
  the single byte `0x2A` (`*`) in the wire form, and MUST NOT be treated as, or collide
  with, any KISS-Classify dtype token. An implementation MUST spell a polymorphic dtype role
  as this wildcard and MUST NOT invent an ad-hoc placeholder. *Test:*
  `test_grammar_dtype_role_wildcard_token`.
- **KISS-GRAMMAR-6.6-0007** — A region MUST carry the **upstream version binding** fields
  `ops_version` (the in-force KISS-Ops version) and `classify_version` (the in-force
  KISS-Classify version); every `op_name`, per-op property, and operand-role token MUST be
  resolved against the pinned `ops_version`, and every dtype token against the pinned
  `classify_version`. These fields MUST carry the **upstream wire/ABI schema-version** axis
  of KISS-Ops and KISS-Classify respectively (umbrella §5.1), **not** a crate semver; they
  are distinct from the KISS-Grammar frozen-shape schema version stamped in-band per
  §6.8-0013. Version identity and comparison MUST be **byte-exact on the token string** (an
  implementation MUST NOT treat `"1"` and `"1.0"` as equal, nor apply numeric
  normalization). An implementation MUST NOT resolve any upstream token against an unpinned
  or ambient version, and every golden region vector MUST cite the `ops_version` and
  `classify_version` it was authored against. Their wire encoding is pinned in
  §6.8-0001/§6.8-0003. *Test:* `test_grammar_upstream_version_binding`.

### 6.7 The seam onto KISS-Contract (downstream)

- **KISS-GRAMMAR-6.7-0001** — **When** a KISS-Contract Identity `op_identity` is a
  KISS-Grammar advertisable-op tag, it MUST be that tag in its canonical normal form
  (§6.1-0007), re-based onto a KISS-Ops op name (the Semantics DAG root's op); KISS-Contract
  MUST NOT carry a private op enum in `op_identity`, and where KISS-Grammar supplies an
  `op_identity` it MUST supply nothing other than an advertisable-op tag over a KISS-Ops
  name. (A contract MAY instead carry a bare KISS-Ops op DAG as its `op_identity` with no
  KISS-Grammar advertisable-op entry, per §6.7-0005; this clause governs only the
  advertisable-tag case.) *Test:* `test_grammar_op_identity_is_advertisable_tag`.
- **KISS-GRAMMAR-6.7-0002** — A KISS-Grammar advertisable **region** MUST map onto a
  KISS-Contract **Semantics op DAG**: a one-node region MUST map to a one-node DAG, a fusion
  MUST map to its DAG, and each node MUST carry the KISS-Ops OpAttrs channel; the mapping
  MUST NOT introduce an op vocabulary other than KISS-Ops op names and KISS-Grammar
  advertisable-op tags. *Test:* `test_grammar_region_maps_to_semantics_dag`.
- **KISS-GRAMMAR-6.7-0003** — The `op_identity` KISS-Grammar supplies is the op's
  **semantic identity** and MUST be kept distinct from the `accept_predicate =
  structure_key` (the KISS-Classify admissibility predicate over a specialization cell): a
  consumer matches **both**, and KISS-Grammar MUST NOT present its advertisable-op tag as a
  cell-admissibility predicate nor absorb the `structure_key` role. Because tag identity is
  pinned (§6.1-0007), two consumers MUST agree byte-for-byte on whether two `op_identity`
  tags match. *Test:* `test_grammar_op_identity_distinct_from_structure_key`.
- **KISS-GRAMMAR-6.7-0004** — Because every advertisable op re-bases onto a KISS-Ops name,
  a downstream consumer MUST be able to reconcile the advertisable surface and the op
  meaning **without a second op vocabulary**: KISS-Grammar supplies the advertisable
  surface and attribute channels, KISS-Ops supplies the meaning and the primitive floor,
  and an implementation MUST NOT require a consumer to reconcile two independent op
  vocabularies. *Test:* `test_grammar_single_op_vocabulary_seam`.
- **KISS-GRAMMAR-6.7-0005** — KISS-Grammar is the **advertisable/named-op** surface and is
  **NOT required for every kernel**: an implementation MUST NOT emit, synthesize, or require
  a KISS-Grammar advertisable-op tag as a **precondition** to producing any KISS-Grammar
  output (a region or its wire form). Producing a region whose nodes are a bare KISS-Ops op
  DAG (primitive or mixed-level) MUST NOT require an advertisable-op tag, and the absence of
  an advertisable-op tag MUST NOT be a KISS-Grammar conformance failure. (Informative:
  whether a KISS-Contract whose Semantics is a bare KISS-Ops op DAG and whose Identity
  carries no advertisable-op tag is an accepted contract is governed by **KISS-Contract**,
  not KISS-Grammar; that such a contract is a valid state is a KISS-Contract requirement,
  referenced here informatively — a pure KISS-Grammar implementation need implement no
  contract-acceptance machinery to satisfy this clause. Every kernel still carries a
  KISS-Contract; only the KISS-Grammar advertisable-op tag is optional.) *Test:*
  `test_grammar_bare_ops_dag_needs_no_tag`.

### 6.8 The region wire form — pinned serialization

- **KISS-GRAMMAR-6.8-0001** — A region MUST serialize to the **region wire form**: a byte
  sequence in exactly this field order — (0) the fixed **magic** and in-band **frozen-shape
  schema version** (§6.8-0013); (1) `n_inputs`; (2) `ops_version` token; (3)
  `classify_version` token; (4) `node_count`; (5) `node_count` node records in the canonical
  node-table order of §6.8-0004, each encoded per §6.8-0010; (6) `extract_count`; (7)
  `extract_count` extract records (§6.8-0006) in the canonical order of §6.8-0011. An
  implementation MUST emit and parse exactly this field order and MUST NOT insert, omit, or
  reorder fields. *Test:* `test_grammar_region_wire_form_field_order`.
- **KISS-GRAMMAR-6.8-0002** — Every integer field in the wire form MUST be encoded
  **little-endian** with a pinned fixed width: `n_inputs` `u32`; `node_count` `u32`; a node
  `kind` tag `u8`; a `Bind` `input_index` `u32`; an `Op` `operand_count` `u16`; each operand
  node-index `u32`; a `consumers` enum `u8` (§6.8-0005); an `operand_role` count `u16`;
  `extract_count` `u16`; an extract `path_len` `u16`; each extract path step `u16`; an
  extract `param_slot` `u32`; the frozen-shape schema version `u16` (§6.8-0013). An
  implementation MUST NOT vary width or endianness. (This clause is an intentional
  **aggregate** wire-encoding obligation pinning all fixed-width integer fields; the mapped
  test attests each width jointly.) *Test:* `test_grammar_wire_integer_encoding`.
- **KISS-GRAMMAR-6.8-0003** — Every token/string in the wire form (`ops_version`,
  `classify_version`, `op_name`, operand role token, dtype token, the wildcard `*`) MUST be
  encoded as a `u16` little-endian **byte-length** followed by that many **UTF-8** bytes,
  spelled exactly as the owning upstream vocabulary spells it (§6.6). An implementation MUST
  NOT null-terminate, pad, re-case, or otherwise transform a token. *Test:*
  `test_grammar_wire_token_encoding`.
- **KISS-GRAMMAR-6.8-0004** — The node table MUST be emitted in **canonical node-table
  order**: a post-order traversal from the root visiting each node's operands in the
  canonical operand order of §6.4-0010, assigning each node its table index at first finish,
  with a shared node (including one made shared by the structural dedup of §6.4-0011)
  assigned exactly once (at its first finish); the root therefore is the **last** entry.
  Every operand index MUST reference a strictly-earlier entry. An implementation MUST NOT
  emit a node table in any other order. *Test:* `test_grammar_wire_node_table_canonical_order`.
- **KISS-GRAMMAR-6.8-0005** — The `consumers` field MUST be encoded as a `u8` enum with the
  pinned values `0x00 = INTERIOR` (external-consumer count required to be 0) and
  `0x01 = ROOT` (region output); no other value is defined at this schema version. A `Bind`
  leaf record MUST NOT include a `consumers` byte. *Test:* `test_grammar_wire_consumers_enum`.
- **KISS-GRAMMAR-6.8-0006** — Each extract record MUST be encoded as: `path_len` (`u16`),
  then `path_len` path steps (each a `u16` canonical operand index from the root over the
  canonicalized tree, §6.4-0007), then `param_slot` (`u32`). An empty extract list MUST be
  encoded as `extract_count = 0` with no records. *Test:* `test_grammar_wire_extract_encoding`.
- **KISS-GRAMMAR-6.8-0007** — The per-node OpAttrs sub-block MUST be embedded as a `u16`
  little-endian byte-length followed by that many bytes of the **KISS-Ops OpAttrs wire
  encoding** (owned by KISS-Ops and used here uninterpreted at the framing level); an empty
  OpAttrs block MUST be encoded as length `0x0000`. The embedded KISS-Ops OpAttrs wire bytes
  MUST be reproduced verbatim and are default-resolved and canonical at the KISS-Ops layer
  (§6.2-0006, §8-0008); a region carrying a non-empty OpAttrs block is a valid golden vector
  only against a KISS-Ops version whose OpAttrs wire encoding is byte-frozen (§8-0008).
  KISS-Grammar MUST NOT define an alternative OpAttrs byte layout. *Test:*
  `test_grammar_wire_opattrs_blob_deferral`.
- **KISS-GRAMMAR-6.8-0008** — The per-node operand-role tuple MUST be encoded in its
  canonical form (§6.1-0008) as a `u16` little-endian **entry count** `k` followed by `k`
  entries, each a role token (§6.8-0003) immediately followed by a dtype token or the
  wildcard `*` (§6.8-0003), in operand order (matching the node's operand order after
  canonicalization, §6.4-0005). `k` MUST be the canonical entry count of §6.1-0008 (trailing
  `(data, *)` defaults omitted; `k = 0` when every operand is unconstrained), and every
  operand with index `≥ k` MUST be reconstructed as `(data, *)`. An implementation MUST NOT
  reorder operand-role entries relative to the node's operands, MUST NOT emit trailing
  default entries, and MUST NOT emit more than `operand_count` entries. *Test:*
  `test_grammar_wire_operand_role_tuple`.
- **KISS-GRAMMAR-6.8-0009** — `synthesis_attrs` MUST NOT appear in the region wire form
  (they are not identity-bearing, §6.1-0007, and parameterize lowering, not matching); two
  regions that differ only in `synthesis_attrs` MUST serialize to identical bytes. An
  implementation MUST NOT include any synthesis-channel content in the byte-exact region
  serialization. *Test:* `test_grammar_wire_excludes_synthesis_attrs`.
- **KISS-GRAMMAR-6.8-0010** — Each node record MUST be encoded with a leading `kind` tag
  (`u8`, §6.8-0002) whose pinned enumerant values are `0x00 = Bind` and `0x01 = Op` (the
  **same** encoding used by the §6.4-0010 canonical subtree serialization). A **Bind** node
  record MUST be, in this exact order: `kind = 0x00`, then `input_index` (`u32`). An **Op**
  node record MUST be, in this exact order: `kind = 0x01`; then `op_name` (length-prefixed
  token, §6.8-0003); then `consumers` (`u8` enum, §6.8-0005); then the OpAttrs sub-block
  (length-prefixed, §6.8-0007) — which is the wire realization of the node grammar's narrow
  per-node `pattern_attrs` field (§2.2, §6.4-0001), carrying **only** the KISS-Ops OpAttrs
  blob; then the operand-role tuple (§6.8-0008); then `operand_count`
  (`u16`); then `operand_count` operand node-indices (each `u32`, §6.8-0002). An
  implementation MUST emit and parse exactly this per-record field order for each kind and
  MUST NOT insert, omit, or reorder fields within a node record. *Test:*
  `test_grammar_wire_node_record_layout`.
- **KISS-GRAMMAR-6.8-0011** — The `extract_count` extract records (§6.8-0006) MUST be
  emitted in a canonical order: **ascending `param_slot`**; where two records share a
  `param_slot` (which an implementation MUST NOT normally produce), ties MUST break by the
  **lexicographically-least** path-step sequence under unsigned comparison. An
  implementation MUST NOT emit extract records in any other order. *Test:*
  `test_grammar_wire_extract_record_order`.
- **KISS-GRAMMAR-6.8-0012** — An advertisable-op **tag** (§6.1-0007) MUST have a dedicated
  **tag canonical serialization**, distinct from the region wire form and carrying **only**
  the identity-bearing fields, in exactly this order: (1) `op_name` (length-prefixed token,
  §6.8-0003); (2) the OpAttrs sub-block (length-prefixed, §6.8-0007, in its KISS-Ops-canonical
  default-resolved bytes); (3) the operand-role tuple in canonical form (§6.8-0008 /
  §6.1-0008). It MUST NOT include the `kind` tag, `consumers`, `operand_count`, operand
  indices, `n_inputs`, the `ops_version` / `classify_version` tokens, the magic/schema-version
  header, or any `extract` record. Two tags MUST be compared for identity (§6.1-0007(c)) by
  byte-exact comparison of this serialization. *Test:* `test_grammar_tag_canonical_serialization`.
- **KISS-GRAMMAR-6.8-0013** — The region wire form MUST begin with field (0): a fixed 4-byte
  **magic** `0x4B 0x47 0x52 0x4D` (ASCII `KGRM`) followed by the KISS-Grammar **frozen-shape
  schema version** as a `u16` little-endian. This stamps the frozen-shape schema version
  **in-band** (distinct from the upstream `ops_version` / `classify_version` bindings of
  §6.6-0007), so a foreign reader parsing raw region bytes detects both a non-KISS-Grammar
  stream (magic mismatch) and a frozen-shape-version it cannot decode. An implementation MUST
  reject (typed decline, never a panic) a region whose leading magic is not `KGRM` or whose
  stamped frozen-shape schema version it does not implement. *Test:*
  `test_grammar_wire_magic_and_schema_version`.

---

## 7. Capability, Profile & Extension model

### 7.1 Mandatory core and the growth extension

- **KISS-GRAMMAR-7.1-0001** — The KISS-Grammar **mandatory core** MUST be the frozen shape
  (§6.5-0005): the advertisable-op field schema, the region-composition rules, the
  attribute-channel structure, and the region wire form. An implementation that cannot honor
  the frozen shape does not conform to KISS-Grammar at all. *Test:*
  `test_grammar_mandatory_core_is_frozen_shape`.
- **KISS-GRAMMAR-7.1-0002** — The set of advertisable **op names** MUST be a growable
  vocabulary deferred entirely to KISS-Ops; a KISS-Grammar implementation MUST accept any
  `op_name` present in the pinned KISS-Ops op set (§6.6-0007) as advertisable-by-re-basing,
  without a KISS-Grammar schema change, and MUST NOT maintain a KISS-Grammar-local
  allow-list that could withhold an otherwise-valid KISS-Ops op from the advertisable
  surface. *Test:* `test_grammar_op_name_set_is_open_via_ops`.
- **KISS-GRAMMAR-7.1-0003** — An input that is not an expressible region (§6.4-0008,
  §6.4-0009) or that names an op absent from the pinned KISS-Ops version (§6.6-0004) MUST
  produce a **typed decline, never a panic**; KISS-Conform verifies both that expressible
  regions round-trip through the wire form and that non-expressible inputs decline cleanly.
  *Test:* `test_grammar_typed_decline_never_panic`.

### 7.2 Extension and promotion

- **KISS-GRAMMAR-7.2-0001** — A new advertisable op enters the surface by the KISS-Ops
  op-set extension path (a new KISS-Ops op name, possibly carrying a new
  commutativity/positionality or operand-role property) plus re-basing (§6.5-0001);
  KISS-Grammar MUST NOT define a separate extension registry for op tokens or op properties,
  because it owns no op vocabulary. Any KISS-Grammar-owned extension (a new
  attribute-channel field, a new composition rule, or a wire-form change) is a
  **frozen-shape** change governed by §8 and the umbrella extension registry (umbrella §6.4).
  *Test:* `test_grammar_extension_via_ops_or_frozen_shape`.

---

## 8. Versioning & Lifecycle

KISS-Grammar tracks the umbrella's **two version axes**: the **frozen-shape schema
version** (the field schema + composition rules + attribute-channel structure + region wire
form; stamped in-band per §6.8-0013) and the published reference-crate **semver**. They move
independently, and both move independently of the KISS-Ops op-name set and of the
KISS-Ops-owned per-op properties.

- **KISS-GRAMMAR-8-0001** — The frozen-shape schema version and the reference-crate semver
  MUST be tracked as independent axes; a crate semver change MUST NOT be taken to imply a
  frozen-shape change. *Test:* `test_grammar_two_version_axes_independent`.
- **KISS-GRAMMAR-8-0002** — Any change to a **frozen-shape** member — the advertisable-op
  field schema (§6.1-0001), a region-composition rule (§6.4), the attribute-channel
  structure (§6.2–§6.3), or the region wire form (§6.8) — MUST bump the frozen-shape schema
  version (and therefore the in-band stamp of §6.8-0013). *Test:*
  `test_grammar_shape_change_bumps_version`.
- **KISS-GRAMMAR-8-0003** — Adding an advertisable op by re-basing onto a KISS-Ops op name
  (§6.5-0001) MUST NOT bump the frozen-shape schema version (it is additive under the growth
  rule); a KISS-Ops op-name-set version change, or a KISS-Ops-owned per-op property carried
  by a new op, MUST NOT bump the KISS-Grammar frozen-shape schema version. *Test:*
  `test_grammar_op_addition_no_shape_bump`.
- **KISS-GRAMMAR-8-0004** — KISS-Grammar MUST NOT be promoted from Draft to Frozen until ≥2
  structurally dissimilar implementations have interoperated on the golden region vectors of
  Appendix A — emitting and parsing the **region wire form** (§6.8) byte-for-byte (umbrella
  §5.3). *Test:* `test_grammar_freeze_gate_two_impls` (checklist gate; signed by the AUDIT
  role, not DESIGN).
- **KISS-GRAMMAR-8-0005** — KISS-Grammar MUST NOT be promoted from Draft to Frozen until a
  foreign reader written outside the reference language has consumed the region-grammar
  **wire form** (§6.8) and reproduced or parsed the golden region vectors of Appendix A —
  including their pinned left-to-right bytes — (the adversarial-outsider checklist, umbrella
  §5.3). *Test:* `test_grammar_freeze_gate_foreign_reader` (checklist gate; AUDIT-signed).
- **KISS-GRAMMAR-8-0006** — KISS-Grammar MUST NOT be promoted from Draft to Frozen until this
  sub-standard's KISS-Conform suite exists and passes, with complete bidirectional
  clause-to-test traceability (umbrella §5.3). *Test:*
  `test_grammar_freeze_gate_conform_suite_passes` (checklist gate; AUDIT-signed).
- **KISS-GRAMMAR-8-0007** — Retire-by-floor deprecation MUST apply to the frozen-shape
  schema version only (the axis stamped in-band per §6.8-0013); an implementation MUST NOT
  advertise a frozen-shape schema version below its declared retirement floor. *Test:*
  `test_grammar_retire_by_floor`.
- **KISS-GRAMMAR-8-0008** — A KISS-Grammar region carrying a **non-empty** OpAttrs block
  (§6.8-0007) MUST NOT be admitted as a **golden region vector** (§8-0004, §8-0005) unless
  the pinned KISS-Ops version's **OpAttrs wire encoding is byte-frozen and default-resolved**
  (defaults emitted explicitly) — the upstream guarantee KISS-Grammar's opaque-blob
  byte-equality (§6.1-0007(b), §6.2-0006) depends on. That upstream guarantee is pinned by
  KISS-OPS **§6.19-0005** (every defaulted OpAttrs attribute emitted explicitly, so a
  defaulted attribute and an explicitly-equal one produce identical bytes) under the
  golden-vector conformance-and-freeze mechanism of KISS-OPS **§6.19-0013**. Golden vector G2
  (and any other OpAttrs-bearing vector) is **gated** on that upstream KISS-Ops freeze and
  MUST cite the KISS-Ops clause that pins OpAttrs byte-determinism — namely KISS-OPS
  §6.19-0005 (explicit default-resolution) and §6.19-0013 (golden-vector conformance +
  freeze); this citation discharges the forward-reference and makes the gate liftable. *Test:*
  `test_grammar_opattrs_freeze_precondition` (checklist gate; AUDIT-signed).

---

## 9. Conformance

An implementation conforms to KISS-Grammar at a given frozen-shape schema version if it
(a) represents the advertisable-op surface, the region grammar, and the region wire form
exactly per §6–§8 for that version, (b) passes the KISS-Conform suite for KISS-Grammar at
that version, and (c) satisfies the DAG prerequisite closure. Because the KISS-Ops →
KISS-Grammar and KISS-Classify → KISS-Grammar edges are **STRUCTURAL** (§4), claiming
KISS-Grammar requires claiming KISS-Ops and KISS-Classify (prerequisite closure, umbrella
§6.3). The KISS-Consume and KISS-Emit references are informative naming references and add
no prerequisite-closure obligation (§4). Un-claimed, non-expressible, or unknown-op inputs
yield typed declines, never panics, per §6.4-0008, §6.4-0009, §6.6-0004, §6.8-0013, and
§7.1-0003 (verified by the negative-vector modality). The modified-suite prohibition of the
mark policy is the umbrella's rule (umbrella §9.3), enforced via registry listing, and is not
restated as a free-standing KISS-Grammar clause.

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-GRAMMAR-6.0-0001 | `test_grammar_determinism_class_exact_byte` |
| KISS-GRAMMAR-6.1-0001 | `test_grammar_advertisable_op_field_schema` |
| KISS-GRAMMAR-6.1-0002 | `test_grammar_no_parallel_op_list` |
| KISS-GRAMMAR-6.1-0003 | `test_grammar_tag_is_name_plus_attrs` |
| KISS-GRAMMAR-6.1-0004 | `test_grammar_level_inherited_not_redeclared` |
| KISS-GRAMMAR-6.1-0005 | `test_grammar_operand_role_tuple` |
| KISS-GRAMMAR-6.1-0006 | `test_grammar_defines_no_op_semantics` |
| KISS-GRAMMAR-6.1-0007 | `test_grammar_canonical_tag_normal_form` |
| KISS-GRAMMAR-6.1-0008 | `test_grammar_operand_role_tuple_canonical_form` |
| KISS-GRAMMAR-6.2-0001 | `test_grammar_pattern_attrs_carry_opattrs` |
| KISS-GRAMMAR-6.2-0002 | `test_grammar_load_bearing_attr_guarded` |
| KISS-GRAMMAR-6.2-0003 | `test_grammar_recognition_is_structural` |
| KISS-GRAMMAR-6.2-0004 | `test_grammar_oob_policy_from_ops` |
| KISS-GRAMMAR-6.2-0005 | `test_grammar_commutativity_is_canonicalization_only` |
| KISS-GRAMMAR-6.2-0006 | `test_grammar_opattrs_subvocab_owned_by_ops` |
| KISS-GRAMMAR-6.3-0001 | `test_grammar_synthesis_carries_decomposition_pointer` |
| KISS-GRAMMAR-6.3-0002 | `test_grammar_flavor_is_distinct_op_name` |
| KISS-GRAMMAR-6.3-0003 | `test_grammar_synthesis_scalar_param_extract` |
| KISS-GRAMMAR-6.3-0004 | `test_grammar_synthesis_emit_hints` |
| KISS-GRAMMAR-6.3-0005 | `test_grammar_channel_separation` |
| KISS-GRAMMAR-6.4-0001 | `test_grammar_region_is_single_rooted_dag` |
| KISS-GRAMMAR-6.4-0002 | `test_grammar_root_carries_identity` |
| KISS-GRAMMAR-6.4-0003 | `test_grammar_repeated_bind_is_node_identity` |
| KISS-GRAMMAR-6.4-0004 | `test_grammar_interior_no_external_consumer_guard` |
| KISS-GRAMMAR-6.4-0005 | `test_grammar_commutative_canonicalization` |
| KISS-GRAMMAR-6.4-0006 | `test_grammar_comparison_select_positional` |
| KISS-GRAMMAR-6.4-0007 | `test_grammar_extract_channel_on_root` |
| KISS-GRAMMAR-6.4-0008 | `test_grammar_bind_set_covers_inputs` |
| KISS-GRAMMAR-6.4-0009 | `test_grammar_single_output_region` |
| KISS-GRAMMAR-6.4-0010 | `test_grammar_canonical_operand_order_key` |
| KISS-GRAMMAR-6.4-0011 | `test_grammar_structural_dedup_and_repeated_bind` |
| KISS-GRAMMAR-6.5-0001 | `test_grammar_add_op_by_rebasing` |
| KISS-GRAMMAR-6.5-0002 | `test_grammar_rebasing_is_additive_no_bump` |
| KISS-GRAMMAR-6.5-0003 | `test_grammar_independent_cadence` |
| KISS-GRAMMAR-6.5-0004 | `test_grammar_nonprimitive_resolvable_to_floor` |
| KISS-GRAMMAR-6.5-0005 | `test_grammar_frozen_shape_membership` |
| KISS-GRAMMAR-6.6-0001 | `test_grammar_op_name_spelling` |
| KISS-GRAMMAR-6.6-0002 | `test_grammar_operand_dtype_token_spelling` |
| KISS-GRAMMAR-6.6-0003 | `test_grammar_op_category_is_classify_tag` |
| KISS-GRAMMAR-6.6-0004 | `test_grammar_op_name_exists_in_ops` |
| KISS-GRAMMAR-6.6-0005 | `test_grammar_operand_role_token_vocabulary` |
| KISS-GRAMMAR-6.6-0006 | `test_grammar_dtype_role_wildcard_token` |
| KISS-GRAMMAR-6.6-0007 | `test_grammar_upstream_version_binding` |
| KISS-GRAMMAR-6.7-0001 | `test_grammar_op_identity_is_advertisable_tag` |
| KISS-GRAMMAR-6.7-0002 | `test_grammar_region_maps_to_semantics_dag` |
| KISS-GRAMMAR-6.7-0003 | `test_grammar_op_identity_distinct_from_structure_key` |
| KISS-GRAMMAR-6.7-0004 | `test_grammar_single_op_vocabulary_seam` |
| KISS-GRAMMAR-6.7-0005 | `test_grammar_bare_ops_dag_needs_no_tag` |
| KISS-GRAMMAR-6.8-0001 | `test_grammar_region_wire_form_field_order` |
| KISS-GRAMMAR-6.8-0002 | `test_grammar_wire_integer_encoding` |
| KISS-GRAMMAR-6.8-0003 | `test_grammar_wire_token_encoding` |
| KISS-GRAMMAR-6.8-0004 | `test_grammar_wire_node_table_canonical_order` |
| KISS-GRAMMAR-6.8-0005 | `test_grammar_wire_consumers_enum` |
| KISS-GRAMMAR-6.8-0006 | `test_grammar_wire_extract_encoding` |
| KISS-GRAMMAR-6.8-0007 | `test_grammar_wire_opattrs_blob_deferral` |
| KISS-GRAMMAR-6.8-0008 | `test_grammar_wire_operand_role_tuple` |
| KISS-GRAMMAR-6.8-0009 | `test_grammar_wire_excludes_synthesis_attrs` |
| KISS-GRAMMAR-6.8-0010 | `test_grammar_wire_node_record_layout` |
| KISS-GRAMMAR-6.8-0011 | `test_grammar_wire_extract_record_order` |
| KISS-GRAMMAR-6.8-0012 | `test_grammar_tag_canonical_serialization` |
| KISS-GRAMMAR-6.8-0013 | `test_grammar_wire_magic_and_schema_version` |
| KISS-GRAMMAR-7.1-0001 | `test_grammar_mandatory_core_is_frozen_shape` |
| KISS-GRAMMAR-7.1-0002 | `test_grammar_op_name_set_is_open_via_ops` |
| KISS-GRAMMAR-7.1-0003 | `test_grammar_typed_decline_never_panic` |
| KISS-GRAMMAR-7.2-0001 | `test_grammar_extension_via_ops_or_frozen_shape` |
| KISS-GRAMMAR-8-0001 | `test_grammar_two_version_axes_independent` |
| KISS-GRAMMAR-8-0002 | `test_grammar_shape_change_bumps_version` |
| KISS-GRAMMAR-8-0003 | `test_grammar_op_addition_no_shape_bump` |
| KISS-GRAMMAR-8-0004 | `test_grammar_freeze_gate_two_impls` |
| KISS-GRAMMAR-8-0005 | `test_grammar_freeze_gate_foreign_reader` |
| KISS-GRAMMAR-8-0006 | `test_grammar_freeze_gate_conform_suite_passes` |
| KISS-GRAMMAR-8-0007 | `test_grammar_retire_by_floor` |
| KISS-GRAMMAR-8-0008 | `test_grammar_opattrs_freeze_precondition` |

Every normative clause above appears in this matrix exactly once; the KISS-Conform build
MUST fail if any clause ID lacks a passing mapped test (bidirectional traceability). Clause
IDs are mirrored in the machine-readable sidecar (`kiss-grammar.validusage.json` analog)
kept in sync by the traceability lint.

---

## 10. Governance

- **Editor of record:** the KISS-Grammar editor assignment is **proposed, pending
  ratification** in the umbrella governance record. The editor holds the pen, allocates
  clause IDs (append-only, never reused after retirement), and solicits comment from
  interested cosignatories — any project building a provider or consumer that advertises,
  recognizes, or lowers ops over the advertisable surface — before deciding a
  cross-party-visible change.
- **Steward:** ThinkersJournal hosts the spec, the extension registry (PR-gated; note that
  the op-**name** vocabulary and the KISS-Ops-owned per-op properties are owned by KISS-Ops,
  not by a KISS-Grammar registry), and the conformance registry; it free-certifies
  self-certified implementations on request as resources permit.
- **Ratifier / maturity transitions:** the AUDIT role (not DESIGN) signs each maturity
  transition; the Draft→Frozen transition requires the freeze gate of §8-0004 / §8-0005 /
  §8-0006 / §8-0008 (umbrella §5.3).
- **License:** this specification is dedicated to the public domain under CC0 1.0 Universal;
  reference crates are MIT-OR-Apache-2.0; the KISS-Conform suite is permissive-to-run. Per
  the umbrella mark policy (umbrella §9.3), a modified conformance suite does not back a
  conformance claim; that policy is enforced via steward-registry listing, not restated as a
  normative KISS-Grammar clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC
  contribution, with defensive termination, per the umbrella.
- **Conformance posture:** self-certification with published results plus the
  steward-maintained registry is the authoritative record of verified implementations.

---

## Appendix A — Worked region vectors & provenance (informative)

**A.1 Golden region vectors.** Each golden vector below cites the upstream versions it was
authored against and carries its exact left-to-right region-wire-form bytes (umbrella §4.3),
so a foreign reader can emit or parse it independently (§8-0005). The region-level field
order is normative in §6.8-0001; the per-node-record layout is normative in §6.8-0010; the
magic and in-band frozen-shape schema version prefix is normative in §6.8-0013 (these bytes
are not merely illustrative). The illustrative frozen-shape schema version stamped below is
`1`.

*Vector G1 — the three-input elementwise fusion of §2.3 (`out = (a * b) + c`).* Authored
against `ops_version = "1"`, `classify_version = "1"` (illustrative version tokens; a real
vector cites the concrete KISS-Ops / KISS-Classify version strings). This is the first
golden vector for `test_grammar_region_is_single_rooted_dag`,
`test_grammar_root_carries_identity`, `test_grammar_interior_no_external_consumer_guard`,
`test_grammar_commutative_canonicalization`, `test_grammar_canonical_operand_order_key`,
`test_grammar_bind_set_covers_inputs`, `test_grammar_wire_node_record_layout`,
`test_grammar_wire_magic_and_schema_version`, and the other wire-form tests of §6.8. Its
root is `add` (`consumers = ROOT`); its interior `mul` carries `consumers = INTERIOR`; the
bound-input set is `{0, 1, 2}` = `[0, 3)` with `n_inputs = 3`. Under the canonical operand
order (§6.4-0010) a `Bind` subtree (`0x00…`) sorts before an `Op` subtree (`0x01…`), so
`add`'s operands canonicalize to `[Bind(2), mul]` and `mul`'s to `[Bind(0), Bind(1)]`; the
canonical node-table order (§6.8-0004, post-order) is therefore:

| index | node | operands (indices) | consumers |
|---|---|---|---|
| 0 | `Bind(2)` | — | (leaf) |
| 1 | `Bind(0)` | — | (leaf) |
| 2 | `Bind(1)` | — | (leaf) |
| 3 | `Op "mul"` | `[1, 2]` | `INTERIOR` |
| 4 | `Op "add"` (root) | `[0, 3]` | `ROOT` |

Its exact wire bytes (hex, left to right; `u16`/`u32` little-endian; each `Op` here has an
empty OpAttrs blob `00 00` and an empty operand-role tuple `00 00` — the canonical
omit-trailing-defaults form of §6.1-0008/§6.8-0008, because both operands of each node are
unconstrained `(data, *)` in this illustrative vector; `extract_count = 0`):

```
4B 47 52 4D                                  magic = "KGRM"
01 00                                        frozen_shape_schema_version = 1
03 00 00 00                                  n_inputs = 3
01 00 31                                     ops_version = "1"
01 00 31                                     classify_version = "1"
05 00 00 00                                  node_count = 5
00 02 00 00 00                               node0: Bind(2)  (kind 0x00, input_index 2)
00 00 00 00 00                               node1: Bind(0)
00 01 00 00 00                               node2: Bind(1)
01 03 00 6D 75 6C 00 00 00 00 00 02 00 01 00 00 00 02 00 00 00
                                             node3: Op "mul", consumers INTERIOR,
                                             opattrs len0, roles count0,
                                             operand_count 2, operands [1, 2]
01 03 00 61 64 64 01 00 00 00 00 02 00 00 00 00 00 03 00 00 00
                                             node4: Op "add" (root), consumers ROOT,
                                             opattrs len0, roles count0,
                                             operand_count 2, operands [0, 3]
00 00                                        extract_count = 0
```

Each `Op` record above follows the §6.8-0010 order (kind, op_name, consumers, opattrs,
operand-role tuple, operand_count, operand indices); each `Bind` record follows the
§6.8-0010 order (kind, input_index). The alternative authoring `c + a*b` canonicalizes to
the **identical** byte sequence, demonstrating the reproducible-emission guarantee of
§6.4-0005 / §6.4-0010 / §6.4-0011.

*Vector G2 — the load-bearing `gather` of §2.4.* Authored against `ops_version = "1"`,
`classify_version = "1"`, and **gated** on the pinned KISS-Ops version's OpAttrs wire
encoding being byte-frozen and default-resolved (§8-0008); its OpAttrs bytes are reproduced
verbatim from, and cite, that KISS-Ops OpAttrs byte-determinism clause — concretely KISS-OPS
**§6.19-0005** (explicit default-resolution) and **§6.19-0013** (golden-vector conformance +
freeze), with the oob_policy sub-vocabulary frozen at KISS-OPS **§6.19-0015**. A one-node region
(`n_inputs = 2`: a data input and an index input); its single node is `Op "gather"` with
`consumers = ROOT`, a non-empty OpAttrs blob carrying `axis = k` and OOB policy `clamp`
(whose bytes are the **KISS-Ops OpAttrs wire encoding**, embedded length-prefixed per
§6.8-0007 — cited from KISS-Ops, not defined here), and the operand-role tuple
`[(data, *), (index, u32)]` encoded per §6.8-0008 (role tokens `data`/`index` from the
KISS-Ops operand-role set, dtype `*` = the wildcard byte `0x2A`, dtype `u32` a KISS-Classify
token). Because operand 1 (`(index, u32)`) differs from the default `(data, *)`, the
canonical entry count is `k = 2` (§6.1-0008), so the tuple encodes both entries. Its operands
are `Bind(0)` and `Bind(1)`. This vector drives
`test_grammar_pattern_attrs_carry_opattrs`, `test_grammar_load_bearing_attr_guarded`,
`test_grammar_oob_policy_from_ops`, `test_grammar_operand_role_tuple`,
`test_grammar_operand_role_tuple_canonical_form`, `test_grammar_dtype_role_wildcard_token`,
`test_grammar_wire_opattrs_blob_deferral`, and `test_grammar_opattrs_freeze_precondition`;
because its full bytes include the KISS-Ops-owned OpAttrs sub-block, its OpAttrs bytes are
reproduced verbatim from the cited KISS-Ops version's OpAttrs vector (and are valid only once
that upstream encoding is frozen, §8-0008).

*Vector G3 — the tag-equality (default-attribute) vector.* Two authorings of a `gather` tag
— one stating `axis = 0` explicitly, one omitting `axis` and relying on its KISS-Ops default
of `0` — MUST canonicalize (§6.1-0007: defaults made explicit) to the **same** tag bytes and
MUST compare equal; a third authoring with `axis = 1` MUST differ. Default resolution is
owned upstream: the KISS-Ops OpAttrs wire encoding emits the defaulted `axis = 0` explicitly
(§6.2-0006, §8-0008; KISS-OPS §6.19-0005), so both authorings produce **identical** OpAttrs bytes and KISS-Grammar
compares the opaque blob byte-exactly without interpreting the sub-vocabulary. The tags are
compared in the dedicated **tag canonical serialization** of §6.8-0012 (op_name + OpAttrs
blob + canonical operand-role tuple, carrying no region framing). Drives
`test_grammar_canonical_tag_normal_form` and `test_grammar_tag_canonical_serialization`.

*Vector G4 — the repeated-bind / structural-dedup vector (`out = a + a`, `n_inputs = 1`).*
Authored against `ops_version = "1"`, `classify_version = "1"`. Its root `add`
(`consumers = ROOT`) has both operands bound to input `0`; the two `Bind(0)` leaves
canonicalize (§6.4-0003, §6.4-0011) to exactly **one** node-table entry, referenced twice, so
the node table is index0 `Bind(0)`, index1 `Op "add"` (root) with operands `[0, 0]`. Its exact
wire bytes:

```
4B 47 52 4D                                  magic = "KGRM"
01 00                                        frozen_shape_schema_version = 1
01 00 00 00                                  n_inputs = 1
01 00 31                                     ops_version = "1"
01 00 31                                     classify_version = "1"
02 00 00 00                                  node_count = 2
00 00 00 00 00                               node0: Bind(0)  (single shared entry)
01 03 00 61 64 64 01 00 00 00 00 02 00 00 00 00 00 00 00 00 00
                                             node1: Op "add" (root), consumers ROOT,
                                             opattrs len0, roles count0,
                                             operand_count 2, operands [0, 0]
00 00                                        extract_count = 0
```

This vector drives `test_grammar_repeated_bind_is_node_identity` and
`test_grammar_structural_dedup_and_repeated_bind` (that two authored `Bind(0)` leaves
produce one table entry, `node_count = 2`, and a deterministic operand permutation).

*Vector G5 — the DAG-shared-target extract vector.* A fusion in which a single scalar-bearing
node is reachable from the root by more than one sequence of canonical operand indices; the
extract record for that scalar param uses the **lexicographically-least** such path
(§6.4-0007), and the region's extract records are emitted in ascending `param_slot` order
(§6.8-0011). Drives `test_grammar_extract_channel_on_root`,
`test_grammar_wire_extract_encoding`, and `test_grammar_wire_extract_record_order`.

*Negative vectors.* A region binding a strict subset of `[0, n_inputs)` (`BindSetMismatch`,
detectable only because `n_inputs` is declared, §6.4-0008), a region binding an index
`≥ n_inputs`, a multi-output forest (§6.4-0009), an `op_name` naming no op in the pinned
KISS-Ops version (§6.6-0004), an operand role token absent from the KISS-Ops operand-role set
(§6.6-0005), a load-bearing attribute dropped (a `gather` with no axis guard), a flavor
expressed as a free-form flag rather than a distinct op name (§6.3-0002), and a byte stream
whose leading magic is not `KGRM` or whose stamped frozen-shape schema version is unsupported
(§6.8-0013) — drive the §6.4 / §6.6 / §6.8 / §7.1 decline tests and form the
adversarial-outsider battery for the foreign-reader freeze gate (§8-0005).

**A.2 Provenance / acknowledgments.** The region grammar (a single-rooted DAG of
advertisable-op nodes over `Bind(input_index)` leaves, carried as the flat indexed node
table of §6.8), the repeated-bind node-identity guard, the no-external-consumer interior
guard, the commutative-operand canonicalization for reproducible emission, the root-anchored
extract channel, and the `[0, n_inputs)` expressibility condition derive from a
pattern-derivation reference crate (the Baracuda `baracuda-kernelgen` `pattern` module),
which derives a fusion pattern mechanically from an op's scalar-expression body. The flat
indexed node table (operands as indices, enabling interior DAG sharing) and the pinned wire
form (§6.8) are the interoperability layer that makes the earlier nested term syntax
byte-exact and foreign-readable. Project and crate names in this appendix and in §0/§2 are
non-normative provenance and examples only; no normative clause names any project. The
neutralization of the earlier vendor vocabulary — a provider-local `OpTag`/fused-op list
folded into "a KISS-Ops op name + pattern/synthesis attributes," a vendor kernel-contract
acronym rendered as "KISS-Contract," and the honest-miss no-contract gates removed by the
attribute channels that carry the axis/OOB/index-dtype the flat grammar could not express —
is recorded here as design provenance, not as a normative requirement.

**A.3 Growth-rule migration note.** Because the op-name set and the per-op properties
(commutativity, positionality, operand roles) are deferred to KISS-Ops and read by name, a
KISS-Ops op-set addition reaches the advertisable surface by re-basing with no KISS-Grammar
frozen-shape schema bump (§6.5-0002, §8-0003) — even when the new op is commutative or
introduces a new operand role. The frozen-shape schema version bumps only on a change to the
field schema, a composition rule, the attribute-channel structure, or the region wire form
(§8-0002), and any such bump changes the in-band stamp of §6.8-0013. This is the mechanism by
which a frozen grammar admits a still-growing op set.

---

*End of KISS-Grammar (Draft proposal). This sub-standard is informative in §0–§5 and
normative in §6+; every binding requirement carries an identified clause with a mapped
KISS-Conform test. KISS-Grammar owns the advertisable-op surface, the region grammar, and
the region wire form, re-bases every advertisable op onto a KISS-Ops op name, reads every
per-op property from KISS-Ops by name, and defines no op semantics — those are resolved from
KISS-Ops. Project and product names appear only in non-normative examples, provenance, and
the reference-implementation pointer; normative clauses use only the generic roles provider,
consumer, implementation, kernel, contract, and target.*
