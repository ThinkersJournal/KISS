# KISS-Conform — Conformance, Traceability & the Freeze Gate

**Sub-standard ID:** KISS-CONFORM
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification* (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ (and the normative
> Appendix E and Appendix F schemas it references) is normative. Normative clauses use
> RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-CONFORM-<section>-<nnnn>`, and each MUST/MUST NOT/SHALL maps 1:1 to at
> least one named KISS-Conform test `test_conform_<slug>`. The KISS-Conform suite
> build FAILS on any normative MUST without a mapped citing test — the mechanism
> this sub-standard itself defines and, uniquely in the suite, applies to its own
> clauses.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Conform |
| Sub-standard ID | KISS-CONFORM |
| Tier | **Cross-cutting.** The ninth and cross-cutting sub-standard: it owns conformance and depends on and TESTS all eight other sub-standards; it gates every freeze. It is what makes the other eight provable. |
| Maturity stage | **Draft** (first-draft proposal; the traceability mechanism, the four modalities, the comparator model, and the freeze gate are NOT yet frozen — the freeze gate of §8 is unmet, and KISS-Conform freezes only after it self-applies its own gate). |
| Editor of record | **Proposed, pending ratification** — a conformance-suite reference-impl project holds the pen and requests comment from every sub-standard editor (Conform tests them all); the ratified governance record does not yet finalize an editor for KISS-Conform. |
| Reference language | The implementation language of the reference conformance-harness crate, named in the ratified governance record; "foreign reader" (§8-0006) means an implementation written **outside** this named reference language. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | a conformance-harness reference crate (the traceability lint, golden-vector harness, oracle-differential engine, IR-DAG fuzzer, and negative-vector battery), named in Appendix A as non-normative provenance; this crate is *a* conformant implementation with no privilege and no exemption. |
| DAG position | **Cross-cutting root of the test relation.** Depends (as a **test dependency**) on all eight other sub-standards; is depended on by **none** (nothing imports Conform's structure). It signs every maturity transition. Not a data/compute/protocol node. |
| Upstream edges | KISS-Announce, KISS-Classify, KISS-Ops, KISS-Grammar, KISS-Contract, KISS-Synth/Provision, KISS-Consume, KISS-Emit — **each a test dependency** (Conform tests every sub-standard's normative surface; it harvests each sub-standard's machine-readable clause sidecar and imports the determinism/fidelity enum from KISS-Ops). No upstream edge is OPAQUE or STRUCTURAL in the data/protocol sense; every one is the **test** relation "Conform tests this sub-standard," matching the umbrella §2.2 edge-table row *each of the other eight → KISS-Conform (test dependency)*. |
| Downstream edges | **None.** No sub-standard depends on KISS-Conform's internal structure; Conform is the cross-cutting root and is imported by nobody. |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |
| Maturity | Draft proposal |

> **Edge-label note (informative).** KISS-Conform's eight upstream edges are all
> the **test** relation, not the OPAQUE/STRUCTURAL data-flow labels the other
> sub-standards carry: Conform does not consume any sub-standard's wire as a
> runtime peer, it *tests* it. It nonetheless **imports** one artifact
> structurally — the single canonical determinism/fidelity enum owned by
> KISS-Ops (KISS-OPS §6.0-0001) — which it re-uses verbatim as the comparator
> selector (§6.0). Because nothing depends on Conform, the suite's own
> Draft→Frozen transition is the last gate to close and is signed by an AUDIT
> role independent of Conform's authoring editor (§8, umbrella §5.3, §7.3).

---

## 1. Purpose & Scope

KISS-Conform owns the **conformance machinery of the entire suite**: the single,
machine-readable, **bidirectional clause↔test traceability matrix** that joins every
normative clause across all eight other sub-standards to a named conformance test;
the **build-fail gate** that makes an uncovered normative surface un-buildable; the
**four test modalities** (golden byte-vectors, an independent CPU-oracle differential
harness, an IR-DAG fuzzer emitting to every backend, and negative/decline vectors);
the **determinism-class-aware comparators** that decide what a passing run means;
**per-sub-standard-per-version keying** so a frozen version's suite is immutable; and
the **freeze gate** — the adversarial-outsider checklist that every sub-standard must
pass, signed by the KISS-Conform AUDIT role, before Draft→Frozen. KISS-Conform is what
turns "the authors declared it stable" into "has a passing conformance gate and a
demonstrated second dissimilar implementation from the document alone."

It owns, and closes, **every conformance obligation the other eight sub-standards
forward-reference to it**: the build-fails-on-untested-MUST mechanism each one cites in
its §0 header and §8 freeze gate; the exact-byte comparator mandate for POD wire fields
(KISS-Announce, KISS-Classify, KISS-Grammar); the oracle-differential harness and the
transcendental ULP ceiling and the complex-arithmetic split comparator (KISS-Ops); the
OpAttrs golden-vector freeze gate (KISS-Ops §6.19); the never-panic decline fuzzing
(KISS-Synth §6.6, KISS-Consume, KISS-Grammar, KISS-Announce); the four-category refusal
taxonomy and the expressibility oracle and the mislabeled-kernel structural lift
(KISS-Consume); the structural op-DAG equality comparator, the two-tier round-trip, and
the Emit↔Consume cross-standard document lint (KISS-Emit); the Semantics-DAG-resolves-to-
floor oracle and the malformed-contract typed decline (KISS-Contract); the foreign-reader
freeze gate and golden token vectors (KISS-Grammar, KISS-Classify); and the one operative
consumer-verify **SHOULD**, which Conform ships as an executable, non-build-gating check.

**KISS-Conform is NOT:** any data vocabulary (the dtype set, operand descriptors,
`structure_key`, `target_capability` are KISS-Classify); any computation vocabulary or
op semantics (the op set, NaN / signed-zero / wrapping / ULP behavior, reference
decompositions, the primitive floor, the determinism/fidelity enum, the OpAttrs channel
are KISS-Ops, imported and tested, never re-defined); any wire, ABI, or protocol format
(those are Announce, Classify, Grammar, Contract, Synth, Consume, Emit — Conform tests
their bytes, it defines none); a kernel implementation, a source language, or a compiler
IR's internals; nor a policing authority over who calls themselves conformant (that is
the umbrella's registry model, §8.4). It defines **tests, comparators, and gates**, and
nothing a sub-standard already owns. Anything not enumerated as in-scope above is out of
scope for KISS-Conform (scope creep by silence is a named trap; silence is not
inclusion).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — the suite is only as real as its gate

Eight sub-standards can each write "MUST" a thousand times, and none of it is worth
anything until a machine can prove that every MUST is enforced by a test and that every
test enforces a live MUST. KISS-Conform is that proof. It is the one sub-standard whose
job is not to describe a wire but to **make the other eight provable** — to turn prose
requirements into a build that refuses to compile when a requirement is unenforced, and
to turn "Frozen" into a signed fact rather than an authorial claim.

Every other sub-standard forward-references Conform in three fixed places: its §0 header
("the KISS-Conform suite build FAILS on any normative MUST without a mapped test"), its
§8 freeze gate ("the sub-standard's KISS-Conform suite exists and passes"), and its §9
traceability matrix. This document is where each of those deferrals is honored with an
actual, named, checkable clause.

### 2.2 Bidirectional traceability — the single join

The heart of KISS-Conform is one machine-readable artifact: the **traceability matrix**.
It is the single join between (a) every atomic normative clause across all eight
sub-standards — each keyed by its umbrella-§3.3 ID `KISS-<SUB>-<section>-<nnnn>`, harvested
from that sub-standard's machine-readable sidecar (the schema pinned normatively in
Appendix E), which is the **sole** authoritative clause source — and (b) every named
conformance test in the suite. The relation is many-to-many but **total in both
directions** over the suite's declared coverage set:

- **Direction 1 (clause → test):** every normative clause resolves to at least one named
  test that **cites** it. This proves *no MUST is unenforced*.
- **Direction 2 (test → clause):** every test cites at least one live clause ID. This
  proves *no test is orphaned or circular*, and *no test cites a retired or non-existent
  ID*.

The matrix is **derived, not hand-written**. A lint takes each sub-standard's sidecar as
the authoritative clause set — and for the plain-old-data tiers (KISS-Classify, KISS-Ops)
the sidecar is itself generated from the canonical schema, so clauses cannot drift from
prose — then cross-references it against the `#[cite(...)]`-style clause annotations each
test carries. The two directions together are the whole invariant: coverage forward,
non-circularity back.

### 2.3 The build fails — not the run

The distinction that makes this real: the **build** of the KISS-Conform suite fails, not
merely its run. A suite that does not fully cover the normative surface of its declared
coverage set cannot even be *produced*, let alone pass. The generate-time gate is purely
**structural** — it decides mapping existence, not test outcome — and fails hard and
non-zero on any of: a normative MUST that no test cites; a test citing a clause ID that
exists in no current sidecar (a dangling cite); a test citing a retired/burned clause ID
(an append-only violation — retired IDs are permanently burned and never re-mapped); or
a clause-ID token present in a sub-standard's prose but absent from its sidecar, or a
POD-tier sidecar that does not match its canonical-schema regeneration (drift). Whether a
mapped test *passes* is a separate **run-time** conformance property (§6.2-0006), kept
distinct from the generate-time structural gate so two independently-built suites never
disagree on whether a sub-standard "builds." This is the concrete realization of the
obligation every one of the eight sub-standards forward-references verbatim.

### 2.4 Four modalities — what a passing run means

Two independent conformance-suite implementations must agree on what "passing" means, so
each modality is pinned tightly enough to leave no room for interpretation:

- **Golden byte-vectors** — pinned input → exact output bytes, compared byte-exact. Each
  vector fixes a concrete input and the wire artifact it must produce or parse, pinned in
  wire order with endianness, field offsets, sizes, padding, magic, tags, and hashes
  explicit, so a foreign reader reproduces the bytes without inferring a convention. This
  is the primary evidence for structural (POD/wire/ABI) clauses whose declared class is
  exact-byte; exact-byte *ops* are additionally evidenced through the oracle-differential
  memcmp path (§6.5).
- **The independent CPU-oracle differential harness** — a reference CPU oracle computes
  each op's result independently and the harness compares an implementation's output under
  the op's declared determinism-class comparator. The oracle is derived *solely* from the
  §6 op-semantics tables and reference decompositions of KISS-Ops and shares **no lowering
  code** with any reference implementation — an objective set-intersection over the
  declared lowering-module manifests (§6.5-0002). A vector whose expected output was
  *derived from* a reference impl's run is rejected as circular by its provenance tag. This
  independence is not a hope; it is a manifest-checkable and provenance-checkable property.
- **The IR-DAG fuzzer emitting to every backend** — a structure-directed fuzzer generates
  random-but-valid KISS-Ops IR DAGs and drives them through every fuzz-target backend the
  suite registers (the suite's own emitter set) and through the recognition (lift)
  direction, asserting cross-backend agreement under the determinism-class comparators and
  exercising the emit↔consume round-trip as a join. Its job is to find the
  incidental-impl-choice leaks that fixed golden vectors miss; any concrete disagreement it
  finds is minimized and **promoted** into a pinned golden/oracle/negative vector, so the
  nondeterministic fuzz run never itself decides pass/fail (§6.6-0007).
- **Negative / decline vectors** — malformed, out-of-claim, truncated, mislabeled,
  unknown-op, or empty-intersection inputs → the correct **typed decline** (or, where
  KISS-Consume's four-category taxonomy makes residue the correct answer, the
  pinned-signature residue classification), never a crash or panic and never a silent empty
  result. Each vector pins the exact decline code — or the exact residue structural
  signature — the implementation must emit; the never-panic obligation is delivered both as
  fixed vectors and as a fuzz campaign with pinned time/memory bounds.

### 2.5 The comparator is selected, never chosen

A test author never *picks* how two results are compared. The comparator is **selected by
the clause's declared determinism/fidelity class**, drawn from the single canonical enum
`{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` owned by KISS-Ops
(KISS-OPS §6.0-0001) and imported here verbatim, never re-forked. Every POD/wire clause is
exact-byte by construction; every numeric clause declares its class; at runtime the class
travels with the artifact (a provided kernel's contract Guarantees carries the determinism
class, and each op advertises its class), so a consumer and Conform pick the *same*
comparator. Two refinements — the complex-arithmetic **split comparator** and the tier-1
**structural op-DAG equality** comparator — are applied where the owning clause names them
and, for the named ops, **override** the class-based default under the total precedence
ordering of §6.8-0008; they do not add a member to the canonical enum. The illustrative op
enumerations in the comparator clauses name which ops *carry* which declared class; the
declared class is always the authoritative selector where an op appears to match two
enumerations.

### 2.6 Per-sub-standard, per-version — a frozen suite is immutable

Conformance is keyed **per-sub-standard per wire/ABI schema version**, never per crate
semver, because the wire/ABI schema version is the axis on which observable bytes change.
A pure code refactor that preserves the bytes bumps only the crate semver and mints no new
suite. The traceability matrix, golden vectors, oracle vectors, and negative vectors are
all stamped `(sub_standard_token, schema_version)` and bundled into a versioned suite
artifact. A Frozen version's suite is **immutable and archived**: its clause set, tests,
and vectors are frozen with it, and retired clause IDs stay burned forever. "Conforms"
means passing the single steward-published **canonical suite artifact** for a specific
`(sub-standard, schema version)`, unmodified; a foreign suite must reproduce that canonical
vector set byte-for-byte before it can issue a verdict (§8-0010).

### 2.7 The freeze gate — adversarial by construction

A sub-standard cannot freeze on its authors' say-so. It freezes only when a **foreign
reader built from the document alone** — an implementation written outside the reference
language (the language of the reference implementation, named in the governance record) —
reproduces or parses the exact wire bytes, checks endianness, pointer width,
structure padding, field offsets, magic, and token spellings byte-for-byte against the
golden vectors, and reports every ambiguity that let it drift; and only when **≥2
structurally dissimilar implementations** (distinct codebases, disjoint lowering-module
manifests) interoperate on the golden vectors. The KISS-Conform **AUDIT role signs** the
Draft→Frozen transition — not the authoring editor — and the **reference implementation
runs the same public, unmodified suite with no exemption**. Two process guards make this
real: oracle-independence (vectors derive from the §6 semantics tables, carry an
`oracle`-derivation provenance tag, and the vector author must not read reference-impl
lowering code) and the second-dissimilar-implementation gate. The `const_lit` C-ism is the
cautionary proof that incidental impl choices leak unless a foreign reader and a
non-C-family emitter exercise the wire.

### 2.8 Terms are joined, not restated

KISS-Conform imports the determinism/fidelity enum, the primitive floor, the reference
decompositions, the OpAttrs channel, the transcendental ULP ceiling, and the complex-arith
family from KISS-Ops by name; the `structure_key`, dtype tokens, and `target_capability`
from KISS-Classify; the region grammar and wire form from KISS-Grammar; the seven-section
contract, the Semantics DAG, and the determinism class it carries from KISS-Contract; the
provision/decline frames from KISS-Synth; the four-category refusal taxonomy and its
residue classification from KISS-Consume; and the two-tier round-trip and neutrality
manifest from KISS-Emit. It re-defines none of them: Conform tests them.

---

## 3. Terms & Definitions

- **Conformance suite (KISS-Conform)** — the versioned, machine-readable artifact bundle
  for a `(sub-standard, wire/ABI schema version)` pair: the traceability matrix, golden
  byte-vectors, oracle vectors, negative/decline vectors, the IR-DAG fuzzer corpus, and the
  bundled vocabulary snapshots. Passing the **unmodified canonical** suite for a pair is the
  factual definition of conforming to that sub-standard at that version (§8-0010, umbrella §8.1).
- **Canonical suite artifact** — the single steward-published, versioned suite (the matrix
  plus the exact golden/oracle/negative vector set) per `(sub-standard, schema version)` that
  is authoritative for a conformance verdict. A foreign suite MUST reproduce its vector set
  byte-for-byte before issuing a verdict (§8-0010).
- **Declared coverage set** — the set of `(sub-standard, wire/ABI schema version)` pairs a
  given suite claims to cover. The traceability totality of §6.1-0002 / §6.1-0003 and the
  build-fail gate of §6.2 are scoped to this set; the canonical full-suite's declared
  coverage set is all eight sub-standards (§6.1-0008).
- **Traceability matrix** — the single machine-readable join keyed by
  `(sub_standard_token, wire/ABI_schema_version, clause_id) → set(named_test_id)`, plus its
  inverse index `named_test_id → set(clause_id)`; the total-in-both-directions relation
  between the normative clause set and the named test set over the declared coverage set (§6.1).
- **Sidecar** — a sub-standard's machine-readable clause file, the **sole** authoritative
  clause set from which the matrix is derived, whose schema (fields, types, encoding) is
  pinned normatively in Appendix E; for the POD tiers (Classify, Ops) it is generated from
  the canonical schema and cannot drift from prose.
- **Normative clause** — an atomic uppercase MUST / MUST NOT / SHALL in a sub-standard's
  §6–§8 carrying a clause ID (umbrella §3.3). Coverage is measured against normative
  clauses only; SHOULD/MAY governance clauses are catalogued separately and are not
  build-gating (§6.1-0005), with the single named exception of the consumer-verify SHOULD
  (§6.12).
- **Named conformance test** — a test in the suite carrying one or more `#[cite(...)]`
  clause annotations, named `test_conform_<slug>` (for a Conform-owned clause) or
  `test_<sub>_<slug>` (for a sub-standard clause the suite enforces).
- **Untested-MUST** — a normative clause in a covered sidecar that **no test cites** (zero
  mapped tests); a generate-time build-fail condition (§6.2-0001). Whether a mapped test
  *passes* is a distinct run-time verdict property (§6.2-0006), not the build gate.
- **Dangling cite** — a test citing a clause ID present in no current sidecar; a build-fail
  condition (§6.2-0002).
- **Retired / burned clause ID** — an append-only clause ID that was allocated and later
  retired; it is permanently burned and never reused or re-mapped (umbrella §3.3). A test
  citing one is a build-fail (§6.2-0003).
- **Drift** — a clause-ID token present in a sub-standard's prose (as
  `KISS-<SUB>-<sec>-<nnnn>`) but absent from its sidecar, or vice-versa, or a POD-tier
  sidecar that does not match its canonical-schema regeneration; a build-fail (§6.2-0004).
- **Golden byte-vector** — a pinned `input → exact output hex` row, compared with the
  exact-byte comparator, with endianness/offsets/sizes/padding/magic/tags/hashes explicit
  (§6.4).
- **CPU oracle** — the reference computation for an op, derived solely from the KISS-Ops
  §6 op-semantics tables and reference decompositions, sharing no lowering code with any
  reference implementation (no lowering module in common per the manifest partition of
  §6.5-0002) (§6.5).
- **Lowering module** — an identified source-file or crate unit that performs op
  lowering / code generation. Each party declares its lowering-module manifest (a set of
  declared module identifiers the suite ingests) so that "shares no lowering code" reduces
  to a set-intersection check on declared module identifiers (§6.5-0002).
- **Oracle-differential harness** — the modality that resolves an op to the primitive
  floor via its KISS-Ops reference decomposition and compares an implementation's output
  against the CPU oracle under the op's declared determinism-class comparator (§6.5).
- **Circular vector** — a conformance vector whose expected output was **derived from** a
  reference implementation's run rather than computed by the CPU oracle, evidenced by its
  derivation-provenance tag; rejected (§6.5-0003).
- **IR-DAG fuzzer** — the structure-directed generator of random-but-valid KISS-Ops IR
  DAGs that drives every fuzz-target backend/emitter the suite registers and the lift
  direction, asserting cross-backend agreement and feeding the negative modality (§6.6).
- **Negative / decline vector** — a malformed, out-of-claim, truncated, mislabeled,
  unknown-op, or empty-intersection input paired with the exact typed decline code — or the
  exact residue structural signature — the implementation must emit; asserts never-panic
  (§6.7).
- **Typed decline** — a structured refusal returned in lieu of a result (a distinguished
  error value/enumerant, or an equivalent out-of-band error return); never a panic, abort,
  crash, hang, or out-of-bounds read (imported usage, aligned with each sub-standard's
  typed-decline definition).
- **Residue** — the typed classification of an un-liftable remainder defined by
  KISS-Consume's four-category refusal taxonomy (not-a-kernel / wrong-op-class /
  unrecognized-but-expressible / inexpressible-residue). A residue carries a pinned
  structural signature and is never a panic or a silent-empty result; where the taxonomy
  makes residue (not a decline) the correct answer, the negative vector pins the residue's
  structural signature the way it otherwise pins a decline code (§6.7-0002, §6.7-0005).
- **Reference language** — the implementation language of the reference implementation,
  named in the ratified governance record / §0 front-matter. A **foreign reader** is an
  implementation written outside the reference language (§8-0006, Appendix B item 2).
- **Determinism / fidelity class** — the single canonical KISS-Ops enum `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` (KISS-OPS §6.0-0001), imported verbatim
  and used as the comparator selector (§6.0, §6.8).
- **Comparator** — the equality/acceptance predicate applied to a result, one of:
  **exact-byte** (memcmp), **ULP/tolerance** (within the declared per-target ULP),
  **order-invariant/nondeterministic** (declared tolerance, no byte-exact requirement), the
  **split comparator** (a complex-transcendental hybrid), or **structural op-DAG equality**
  (the tier-1 round-trip comparator) (§6.8, §6.9).
- **Structural op-DAG equality** — two KISS-Ops op DAGs are equal iff, after resolving
  every non-primitive node to the primitive floor, placing nodes/edges in KISS-Ops
  canonical order, and normalizing commutative/associative operands, their node sets, edge
  sets, and per-node OpAttrs byte channels are identical (§6.9).
- **Expressibility oracle** — the judgment that a residue region is "expressible" iff its
  region signature is a member of the enumerated expressible-signature set at the referenced
  KISS-Ops op-set version. That set is a concrete, versioned artifact stamped by the KISS-Ops
  op-set version, owned by the KISS-Ops editor, regenerated on each op-set-version bump, and
  bundled WITH the suite (§6.10, §6.10-0005). It drives the KISS-Consume
  unrecognized-but-expressible vs inexpressible-residue split (§6.10).
- **Cross-standard document lint** — the lint that checks two paired texts (e.g. the
  KISS-Emit and KISS-Consume two-tier round-trip clauses) are semantically equivalent per
  an enumerated correspondence table, and that shared imports (e.g. the determinism enum)
  do not textually fork; it reads the documents, not an implementation's behavior (§6.11).
- **Consumer-verify check** — the executable, non-build-gating check implementing the one
  operative consumer-verify SHOULD: running a received kernel against its contract's
  declared precision, determinism class, and accept-predicate, reusing the oracle-
  differential harness and the determinism-class comparators (§6.12).
- **Freeze gate** — the adversarial-outsider checklist (Appendix B) — ≥2 dissimilar
  implementations interoperating plus a foreign reader from the document alone — as
  objective, checkable items that gate a sub-standard's Draft→Frozen transition; signed by
  the AUDIT role, with the reference impl running the unmodified public suite with no
  exemption (§8, umbrella §5.3).
- **AUDIT role** — the KISS-Conform role that signs a maturity transition (not the
  authoring/design editor); attempts a second dissimilar implementation from the document
  alone and reports every ambiguity (umbrella §5.3, §7.3).
- **Wire/ABI schema version** — the version axis Conform keys conformance on (umbrella
  §5.1): the KISS-Announce envelope version, the KISS-Classify `structure_key` version, the
  KISS-Ops op-vocabulary schema version, the KISS-Grammar `grammar_version`, the KISS-Contract
  `contract_version`, the KISS-Emit `EMIT_ABI_VERSION`, and the KISS-Synth PRSP version.
- **Reference implementation** — a conformant implementation of one or more sub-standards
  with no privilege and no exemption; it runs the same public unmodified canonical suite
  every other implementation runs (§8-0007, umbrella §8.1).

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point semantics; referenced transitively through KISS-Ops
  (KISS-Conform defines no numeric behavior of its own; it evaluates numeric results under
  comparators selected by the KISS-Ops determinism class).
- **ISO/IEC 9899 Annex G** — complex arithmetic semantics; referenced transitively through
  KISS-Ops for the complex-arith family whose split comparator KISS-Conform implements
  (§6.8-0005).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword convention,
  the normative/informative split, the clause-ID scheme and 1:1 test mapping (umbrella
  §3.3), value pinning as bits/IEEE-754 in wire order, the ban on unquantified adjectives,
  the two version axes (§5.1), the freeze gate (§5.3), the capability/profile/extension
  model (§6), the conformance model (§8), governance (§7), licensing and patent posture
  (§9). **Stated once in the umbrella; referenced here; never restated.** This
  sub-standard's §5 points at umbrella §3 for conventions. Where umbrella §5.3's
  foreign-reader field list is narrower than the authoritative superset of §8-0006 /
  Appendix B item 2, KISS-CONFORM §8-0006 governs.
- **KISS-Ops** (by version) — **test dependency**, and the one **structural import**:
  KISS-Conform imports the single canonical **determinism/fidelity enum** `{exact-byte,
  ULP/tolerance, order-invariant/nondeterministic}` (KISS-OPS §6.0-0001) verbatim as the
  comparator selector (§6.0), and evaluates numeric clauses against the KISS-Ops §6
  op-semantics tables, reference decompositions, the **primitive floor** (the termination
  guarantee for the oracle-differential harness), the **transcendental ULP ceiling**
  (KISS-OPS §6.8-0001), the **per-op class advertisement** (KISS-OPS §7.4-0001), the
  **OpAttrs canonical little-endian encoding** (KISS-OPS §6.19-0013), the **complex-arith
  family** (carg/clog/csqrt/cexp), and the **enumerated expressible-signature set** (owned
  by the KISS-Ops editor, stamped by op-set version, §6.10). Conform re-forks none of these;
  it tests them and imports the enum.
- **KISS-Classify** (by version) — **test dependency**: KISS-Conform applies the exact-byte
  comparator to the `structure_key` token codec and the `target_capability` grammar, bundles
  the versioned, machine-readable namespace-registry snapshot and target-capability
  vocabulary WITH the suite so an offline foreign reader holds a complete copy (§6.3-0003,
  §6.13-0003), and supplies golden token vectors. Classify stays UNFROZEN until the
  namespace vocabulary is exercised by usage on a target outside the initial
  reference-hardware namespace (§6.13-0004).
- **KISS-Grammar** (by version) — **test dependency**: KISS-Conform owns the fuzzer and
  differential harness that exercise the region grammar, wire form (`grammar_version`-keyed),
  and still-growing-op-set growth model, verifies expressible regions round-trip and
  non-expressible inputs decline cleanly (never panic), and supplies the foreign-reader
  golden token vectors (§6.13-0009, §6.13-0010).
- **KISS-Contract** (by version) — **test dependency**: KISS-Conform byte-compares the
  seven-section schema/framing (never the free-text blurb), verifies the `audited_status`
  derivation, resolves the Semantics DAG to the primitive floor via the oracle, supplies
  negative vectors so malformed/inconsistent contracts fail LOUDLY as a typed decline over
  the hard-reject transport, and ships the executable check for the consumer-verify SHOULD
  (§6.12, §6.13-0011, §6.13-0012 through §6.13-0024).
- **KISS-Announce** (by version) — **test dependency**: KISS-Conform applies the byte-exact
  comparator to every Announce structural clause, supplies golden byte-vectors for the
  56-byte envelope and the version-negotiation frames, proves the two seam-hello seeds are
  byte-identical via golden hex, runs a foreign reader over the envelope, and supplies
  negative vectors for the unknown-bit reserved-and-ignore and empty-profile
  hard-fail-never-panic obligations (§6.13-0001, §6.13-0002, §6.13-0019).
- **KISS-Synth/Provision** (by version) — **test dependency**: KISS-Conform owns the
  fuzz/negative modality that drives the provision/JIT path to prove never-panic across the
  §6.6 decline taxonomy, selects the comparator from the provided kernel's contract
  determinism class, checks returned-contract content-validity, and reuses the
  oracle-differential harness for the consumer-verify SHOULD (§6.13-0013).
- **KISS-Consume** (by version) — **test dependency**: KISS-Conform resolves a lifted
  Semantics DAG to the primitive floor under the op's determinism class, exercises the four
  refusal categories and the never-panic obligation with negative vectors, defines the
  expressibility oracle, requires a mislabeled-kernel structural-lift vector, and verifies
  the emit↔consume round-trip as a JOIN, not a DAG edge (§6.10, §6.13-0014, §6.13-0015).
- **KISS-Emit** (by version) — **test dependency**: KISS-Conform owns the structural op-DAG
  equality comparator, admits tier-2 numeric bit-identity only same-language on-device and
  only when every resolved-to-floor op is exact-byte class, uses the emitted kernel's
  Semantics DAG as the numeric oracle, runs the IR-DAG fuzzer to every backend including a
  non-C-family emitter, owns the Emit↔Consume cross-standard document lint, verifies the
  neutrality-audit manifest, and AUDIT-signs the Draft→Frozen transition (§6.9, §6.11,
  §6.13-0016, §6.13-0017).
- **KISS-Conform is depended on by none.** No sub-standard imports Conform's structure;
  the eight forward-reference obligations to Conform (each sub-standard's §0/§8/§9) are
  **test** relations, closed here in §6.13.

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase keywords
`MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance and
consumer-behavior guidance and never state a structural or wire requirement — with the one
suite-wide exception that KISS-Conform *owns* the consumer-verify SHOULD as an executable
check (§6.12), the SHOULD being catalogued separately from the build-gating normative set.
Every atomic requirement carries a stable, append-only ID `KISS-CONFORM-<section>-<nnnn>`,
allocated by the editor of record, never reused after retirement, and mapped 1:1 to ≥1
named KISS-Conform test `test_conform_<slug>`. Each clause states exactly one obligation
(umbrella §3.3, atomicity); compound requirements are split into separate clauses with
their own append-only IDs and dedicated tests. Values are pinned as the upstream
vocabularies pin them (bits/IEEE-754/tokens in wire order), never as one source language's
surface spelling. Unquantified adjectives ("well-formed", "reasonable", "neutral", "valid",
"dissimilar") are not the load-bearing requirement: where a term of art (for example
"structurally dissimilar", "foreign reader", "lowering module", or "residue") is used, §3
or an objective §8/Appendix-B checklist item pins it. Every clause that evaluates a numeric
result names the determinism class that selects its comparator, drawn from the KISS-Ops
enum (§6.0). See umbrella §3 for the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class — imported, never re-forked

- **KISS-CONFORM-6.0-0001** — KISS-Conform MUST import the single canonical
  determinism/fidelity enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`
  from KISS-Ops (KISS-OPS §6.0-0001) **verbatim** and MUST NOT re-fork, re-spell, extend, or
  re-order it; the three comparators of §6.8 are the selection targets of exactly these three
  members. *Test:* `test_conform_determinism_enum_imported`.
- **KISS-CONFORM-6.0-0002** — For every clause KISS-Conform evaluates, the comparator MUST be
  **selected** by the clause's declared determinism/fidelity class and MUST NOT be chosen by
  the test author. *Test:* `test_conform_comparator_selected_by_class`.
- **KISS-CONFORM-6.0-0003** — Every **machine-comparable structural artifact** of this
  sub-standard's §6–§8 (the matrix serialization, the modality vector encodings, and the
  keying tuples) is determinism-class **exact-byte** and MUST be evaluated with a byte-exact
  comparator; the **human/process freeze-gate items** (AUDIT-role sign-off, foreign-reader
  ambiguity reports, the "≥2 dissimilar implementations interoperate" demonstration) are NOT
  byte sequences and MUST NOT be subjected to a byte-exact comparator — they are gated by
  AUDIT signature and checklist satisfaction (§8-0006). *Test:*
  `test_conform_structural_artifacts_byte_exact`.

### 6.1 The bidirectional traceability matrix

- **KISS-CONFORM-6.1-0001** — KISS-Conform MUST maintain a machine-readable **traceability
  matrix** that is the single join between (a) every atomic normative clause across the
  sub-standards in the suite's declared coverage set, each keyed by its umbrella-§3.3 ID and
  harvested from that sub-standard's machine-readable sidecar, and (b) every named
  conformance test in the suite. *Test:* `test_conform_traceability_matrix_exists`.
- **KISS-CONFORM-6.1-0002** — The matrix MUST be **total in direction 1 (clause → test)**:
  every normative clause in every sidecar of the suite's declared coverage set MUST resolve
  to at least one mapped named test that **cites** it. A normative clause that no test cites
  is an untested-MUST and is a generate-time build-fail (§6.2-0001); this direction-1 gate is
  **structural** (mapping existence), not a test-outcome check (§6.2-0006). *Test:*
  `test_conform_every_clause_has_test`.
- **KISS-CONFORM-6.1-0003** — The matrix MUST be **total in direction 2 (test → clause)**:
  every named conformance test MUST cite at least one clause ID that is **live** (present in a
  current sidecar and not retired). A test citing no live clause is an orphan and is a
  build-fail (§6.2-0002 / §6.2-0003). *Test:* `test_conform_every_test_cites_clause`.
- **KISS-CONFORM-6.1-0004** — The matrix MUST be **derived, not hand-authored**: a lint MUST
  parse each sub-standard's sidecar for the authoritative clause set and cross-reference it
  against the `#[cite(...)]`-style clause annotations each test carries; an implementation
  MUST NOT accept a hand-maintained matrix as authoritative, because a hand-maintained matrix
  can silently diverge from the sidecar and the tests. *Test:*
  `test_conform_matrix_derived_from_sidecar`.
- **KISS-CONFORM-6.1-0005** — Coverage MUST be measured against **normative clauses only**
  (uppercase MUST / MUST NOT / SHALL in a sub-standard's §6–§8); SHOULD / MAY governance
  clauses MUST be catalogued separately and MUST NOT be build-gating, with the **single named
  exception** of the consumer-verify SHOULD, which KISS-Conform owns and ships an executable
  check for (§6.12). *Test:* `test_conform_coverage_normative_only`.
- **KISS-CONFORM-6.1-0006** — For the plain-old-data tiers (KISS-Classify, KISS-Ops), the
  sidecar the matrix harvests MUST be **generated from the canonical schema** so its clause
  set cannot drift from the prose tables; KISS-Conform MUST reject a POD-tier sidecar that does
  not match its canonical-schema regeneration as drift (§6.2-0004). *Test:*
  `test_conform_pod_sidecar_generated`.
- **KISS-CONFORM-6.1-0007** — The machine-readable **sidecar MUST be the sole authoritative
  clause source** for every sub-standard in the suite's declared coverage set — POD and
  non-POD alike — and the matrix's clause set MUST be exactly the union of the covered
  sidecars' clause sets; the sidecar file schema (fields, types, encoding) MUST conform to the
  normative schema of **Appendix E**, and a sidecar that does not conform to Appendix E MUST be
  rejected. Prose MUST NOT be treated as an independent clause source. *Test:*
  `test_conform_sidecar_sole_authority`.
- **KISS-CONFORM-6.1-0008** — Each suite MUST declare its **coverage set** — the explicit set
  of `(sub-standard, wire/ABI schema version)` pairs it covers — and the totality gates of
  §6.1-0002 / §6.1-0003 and the build-fail gate of §6.2 MUST be scoped to that declared
  coverage set; clauses of a sub-standard NOT in the declared coverage set MUST be out of scope
  for that suite's build gate. The steward-published canonical full-suite MUST declare all
  eight sub-standards in its coverage set. *Test:* `test_conform_declared_coverage_set`.

### 6.2 The build-fail gate

- **KISS-CONFORM-6.2-0001** — The KISS-Conform suite **build** MUST fail hard and non-zero on
  any normative MUST / MUST NOT / SHALL clause in any covered sub-standard's sidecar that **no
  test cites** (an untested-MUST); this is a **structural mapping-existence** check and MUST
  occur at build/generate time, independent of whether any mapped test passes. *Test:*
  `test_conform_build_fails_untested_must`.
- **KISS-CONFORM-6.2-0002** — The build MUST fail on any test citing a **clause ID that exists
  in no current sidecar** (a dangling cite). *Test:* `test_conform_build_fails_dangling_cite`.
- **KISS-CONFORM-6.2-0003** — The build MUST fail on any test citing a **retired / burned
  clause ID**; retired IDs are permanently burned and MUST NOT be re-mapped in any version
  (umbrella §3.3, append-only). *Test:* `test_conform_build_fails_retired_cite`.
- **KISS-CONFORM-6.2-0004** — The build MUST fail on **drift**, detected by a deterministic
  clause-ID-token match (never a semantic prose parse): every clause-ID token that appears in a
  sub-standard's prose as `KISS-<SUB>-<sec>-<nnnn>` MUST appear in that sub-standard's sidecar
  and every sidecar clause ID MUST appear in prose, and a POD-tier sidecar MUST match its
  canonical-schema regeneration (§6.1-0006); any mismatch is drift. *Test:*
  `test_conform_build_fails_drift`.
- **KISS-CONFORM-6.2-0005** — The gate of §6.2-0001 through §6.2-0004 MUST be a
  **compile/generate-time** gate: a suite that does not fully cover the normative surface of
  its declared coverage set MUST NOT be producible, so it cannot be run or claimed. An
  implementation MUST NOT defer any of these four structural checks to run time. *Test:*
  `test_conform_gate_is_generate_time`.
- **KISS-CONFORM-6.2-0006** — For a conformance **verdict** (distinct from the generate-time
  structural gate of §6.2-0001 through §6.2-0005), **every** test mapped to a covered clause
  MUST **pass at run time**; a failing mapped test MUST fail the conformance run for that
  `(sub-standard, schema version)`. This run-time pass requirement MUST NOT be conflated with,
  nor evaluated at, the build/generate-time gate. *Test:*
  `test_conform_mapped_tests_pass_runtime`.

### 6.3 Matrix shape, keying, and self-contained bundling

- **KISS-CONFORM-6.3-0001** — The matrix MUST be keyed by the composite
  `(sub_standard_token, wire/ABI_schema_version, clause_id) → set(named_test_id)` and MUST
  carry the inverse index `named_test_id → set(clause_id)`; both indices MUST be present so
  each direction of §6.1 is machine-checkable. *Test:* `test_conform_matrix_composite_key`.
- **KISS-CONFORM-6.3-0002** — Each matrix entry MUST carry the per-clause metadata **copied
  from the source sidecar**: the clause's section, its keyword (MUST / MUST NOT / SHALL /
  SHOULD), its declared determinism/fidelity class (which selects the comparator, §6.8), and
  the maturity of the owning sub-standard. *Test:* `test_conform_matrix_carries_metadata`.
- **KISS-CONFORM-6.3-0003** — The suite MUST be **self-contained** for an offline foreign
  reader: the matrix, golden vectors, oracle vectors, and negative vectors MUST be bundled as
  versioned machine-readable artifacts WITH the suite for that version, and the versioned
  KISS-Classify namespace-registry snapshot, the target-capability vocabulary, and the
  KISS-Ops enumerated expressible-signature set (§6.10-0005, conforming to the schema of
  **Appendix F**) MUST be bundled alongside so a
  reader built from the document alone holds a complete, self-contained copy. *Test:*
  `test_conform_suite_self_contained_bundle`.
- **KISS-CONFORM-6.3-0004** — The matrix (and every bundled vector set) for a **Frozen**
  version MUST be immutable and archived; an implementation MUST NOT mutate a frozen version's
  matrix, and new or changed clauses MUST land only against a new schema version's matrix
  (§8-0004). *Test:* `test_conform_frozen_matrix_immutable`.

### 6.4 Modality 1 — golden byte-vectors

- **KISS-CONFORM-6.4-0001** — KISS-Conform MUST provide **golden byte-vectors** — pinned input
  → exact output bytes compared with the **byte-exact comparator** (§6.8-0001) — and this
  modality MUST be the sole evidence for every **structural (POD / wire / ABI)** clause whose
  declared determinism class is **exact-byte**; KISS-Conform MUST NOT accept a tolerance or
  order-invariant comparison for such a structural exact-byte clause. (Exact-byte *ops* are
  evidenced through the oracle-differential memcmp path of §6.5 / §6.13-0005, not by this
  sole-evidence rule.) *Test:* `test_conform_golden_byte_vectors`.
- **KISS-CONFORM-6.4-0002** — Each golden byte-vector MUST pin its bytes in **wire order (left
  to right)** with **endianness, field offsets, sizes, padding, magic, tags, and hashes
  explicit**, so a foreign reader reproduces the bytes without inferring a convention; a vector
  MUST NOT leave any of these implicit. *Test:* `test_conform_golden_vectors_fully_pinned`.
- **KISS-CONFORM-6.4-0003** — Golden byte-vectors for a `(sub-standard, schema version)` MUST
  be **exactly and only** the set of that sub-standard's Appendix-A "bytes on the wire, left to
  right" rows for that version (a bijection with those rows), stored as `input → hex` rows, so
  the suite's exact-byte evidence and the sub-standard's own golden rows are the same artifact;
  a golden-vector set that is a strict superset or subset of the Appendix-A rows MUST be
  rejected. *Test:* `test_conform_golden_vectors_hex_rows`.

### 6.5 Modality 2 — the independent CPU-oracle differential harness

- **KISS-CONFORM-6.5-0001** — KISS-Conform MUST provide an **oracle-differential harness** in
  which a reference CPU oracle computes each op's result independently and the harness compares
  an implementation's output against it under the op's declared determinism-class comparator
  (§6.8). *Test:* `test_conform_oracle_differential_harness`.
- **KISS-CONFORM-6.5-0002** — The CPU oracle MUST be derived **solely** from the KISS-Ops §6
  op-semantics tables and reference decompositions (NaN propagation, signed zero, IEEE-fmax vs
  NaN-propagating max, wrapping-int, raw-bit select, complex-arith Annex G, transcendental ULP
  ceilings) and MUST share **no lowering module** with any reference implementation, where
  "shares none" is a **machine-checkable set-intersection** of the oracle's declared
  lowering-module manifest and each reference implementation's declared lowering-module
  manifest that MUST be empty; a non-empty intersection MUST fail the independence check.
  *Test:* `test_conform_oracle_independent`.
- **KISS-CONFORM-6.5-0003** — Every conformance vector MUST carry a **derivation-provenance
  tag** naming the source of its expected output; a vector whose provenance tag is anything
  other than `oracle` (e.g. a value **derived from** a reference implementation's run) MUST be
  rejected as **circular**, and KISS-Conform MUST NOT admit it into the suite. Circularity is a
  provenance property, not an output-comparison property. *Test:*
  `test_conform_reject_circular_vector`.
- **KISS-CONFORM-6.5-0004** — The harness MUST resolve any non-primitive op through its
  KISS-Ops reference decomposition **down to the primitive floor** (the floor is the
  termination guarantee) and compare the fully-lowered result; an implementation MUST NOT
  terminate resolution above the floor for a non-primitive op. *Test:*
  `test_conform_oracle_resolves_to_floor`.
- **KISS-CONFORM-6.5-0005** — Oracle-independence MUST be **process-enforced and recorded**:
  the party authoring oracle vectors MUST NOT read any reference-implementation lowering code,
  the authoring provenance MUST record this and MUST declare the oracle's lowering-module
  manifest for the §6.5-0002 set-intersection check, and each vector MUST carry the `oracle`
  derivation-provenance tag of §6.5-0003; a vector authored in violation MUST be treated as
  circular. *Test:* `test_conform_oracle_authoring_independence`.
- **KISS-CONFORM-6.5-0006** — For a **discontinuous op** — a comparison (`cmp_eq` / `cmp_ne` /
  `cmp_lt` / `cmp_le` / `cmp_gt` / `cmp_ge`), a `select` **condition**, `sign`, or `step` — the
  CPU oracle MUST make the boundary decision on each operand **rounded to the op's declared
  compute dtype BEFORE the comparison**, so the oracle resolves the boundary in the same domain
  the kernel does. An oracle that decides on an un-narrowed higher-precision differential value
  (e.g. an f64 operand never rounded to the op's compute dtype) MUST NOT be admitted, because it
  flips spuriously at a boundary the kernel resolves in the compute dtype: two operands that are
  distinct in the differential's wide precision but round to the **same** compute-dtype value are
  equal at the op, and the oracle MUST decide them equal. The narrowing MUST be the same
  round-to-nearest the kernel's own store/compute path applies (a single rounding to the compute
  dtype), evidenced by the pinned boundary golden vector (Appendix A.2). *Test:*
  `test_conform_boundary_decision_compute_dtype`.
- **KISS-CONFORM-6.5-0007** — The CPU oracle's reference value for a **transcendental atom** MUST
  be computed **strictly tighter than the declared per-target ULP tolerance** it is then compared
  under (§6.8-0002 / §6.8-0003): the oracle MUST evaluate the atom at a precision **wider than the
  op's compute dtype** and round once to that dtype, bounding the oracle's own error at ≤ 0.5 ULP
  of the compute dtype while any admissible declared tolerance is ≥ 1 ULP. This is an
  oracle-accuracy **floor** the oracle MUST meet, distinct from and complementary to the ULP
  **ceiling** §6.8-0003 caps on the *declared* tolerance; an oracle whose own transcendental error
  is not strictly tighter than the tolerance it enforces (e.g. a reference computed at the same
  compute-dtype precision as the implementation under test) MUST NOT be admitted, because it
  measures the oracle's error rather than the implementation's and yields false passes. *Test:*
  `test_conform_oracle_tighter_than_declared_ulp`.
- **KISS-CONFORM-6.5-0008** — The **frozen oracle-vector set MUST cover every op carrying an
  oracle-differential obligation** (§6.5-0001) in the enumerated op-set version the suite bundles,
  and for **every transcendental atom** (KISS-OPS §6.8) the covering vectors MUST include the
  **load-bearing edges** the §6 op-semantics tables pin — the §6.15 non-mergeable distinctions,
  signed zero, NaN propagation, and each atom's declared domain boundaries and overflow arguments —
  each minted per §6.5-0007; a bundled oracle-vector set that omits an obligated op, or that covers
  a transcendental atom only at ordinary interior points while skipping its pinned edges, MUST be
  rejected. This is the oracle-modality analogue of the golden-byte-vector coverage bijection
  (§6.4-0003): once the oracle is a minting instrument rather than a live service (§6.5-0007
  requires a precision wider than the compute dtype), the frozen oracle vectors are the **sole tight
  evidence** for the ULP-class atoms, so a partial set silently leaves those atoms in the
  widened-band regime with no tight check anywhere. *Test:*
  `test_conform_oracle_vector_coverage_complete`.
- **KISS-CONFORM-6.5-0009** — Each frozen oracle vector MUST **store its oracle-computed expected
  value inline** (a pinned `input → expected output → declared tolerance`), so the corpus is
  verifiable **offline without re-running the oracle**; a vector that carries only an input and
  defers its expected value to a live oracle invocation MUST be rejected, because the §6.5-0007
  oracle — evaluating wider than the compute dtype, which at the widest supported compute dtype is
  not a native machine float — is a minting instrument, not a hot-path service a consumer or foreign
  reader can call. For a **transcendental atom** the stored value MUST be the §6.5-0007
  wide-precision reference, and at the **widest supported floating compute dtype**, where no wider
  native float exists, it MUST be produced with an **extended- or arbitrary-precision facility** (a
  correctly-rounded or not-less-than-double-mantissa reference) and rounded once to the compute
  dtype; a stored transcendental reference computed at that dtype's own native precision MUST be
  rejected under §6.5-0007. KISS-Conform MUST NOT mandate a specific such facility. *Test:*
  `test_conform_oracle_vector_stores_wide_precision_value`.

### 6.6 Modality 3 — the IR-DAG fuzzer emitting to every backend

- **KISS-CONFORM-6.6-0001** — KISS-Conform MUST provide a **structure-directed fuzzer** that
  generates random-but-valid KISS-Ops IR DAGs respecting operand roles, ranks, dtypes, OpAttrs
  sub-vocabularies, and the acyclic strictly-decreasing-level structure; a generated DAG that
  violates any of these MUST NOT be emitted as a valid-path input (it is a negative-path input,
  §6.6-0005). *Test:* `test_conform_fuzzer_generates_valid_dags`.
- **KISS-CONFORM-6.6-0002** — The fuzzer MUST drive each generated DAG through **every
  backend/emitter the conformance SUITE registers as a fuzz target** (the suite's own emitter
  set, not the implementation-under-test's shipped backends) and through the **recognition
  (lift) direction**; an implementation MUST NOT exempt a suite-registered fuzz-target backend
  or the lift direction from the fuzz campaign. *Test:* `test_conform_fuzzer_every_backend`.
- **KISS-CONFORM-6.6-0003** — The fuzzer MUST assert **cross-backend agreement** under the
  determinism-class comparators: structural op-DAG equality (§6.9) for the tier-1 round-trip,
  and numeric bit-identity claimed only same-language on-device (§6.8-0007, §6.13-0017); it
  MUST NOT assert cross-language numeric bit-identity. *Test:*
  `test_conform_fuzzer_cross_backend_agreement`.
- **KISS-CONFORM-6.6-0004** — The fuzzer MUST exercise the **emit↔consume round-trip as a
  join** (a generated DAG emitted then lifted, compared structurally, §6.10-0004); it MUST NOT
  model the round-trip as a directed dependency edge between KISS-Emit and KISS-Consume.
  *Test:* `test_conform_fuzzer_roundtrip_join`.
- **KISS-CONFORM-6.6-0005** — The fuzzer MUST also generate **malformed / near-boundary DAGs**
  and feed them to the negative modality (§6.7); an implementation MUST NOT restrict the fuzzer
  to valid-path inputs. *Test:* `test_conform_fuzzer_feeds_negative`.
- **KISS-CONFORM-6.6-0006** — At least one emitter the SUITE registers as a fuzz target MUST be
  a **non-C-family emitter**, so an incidental C-family lowering choice cannot pass unseen; a
  suite whose only fuzz-target emitters are C-family does not satisfy this clause. *Test:*
  `test_conform_fuzzer_includes_non_c_emitter`.
- **KISS-CONFORM-6.6-0007** — A concrete cross-backend disagreement the fuzzer finds MUST be
  **minimized and promoted** into a pinned golden, oracle, or negative vector (which then
  gates), so a nondeterministic fuzz run never itself decides a pass/fail conformance verdict;
  the fuzzer's structural obligations of §6.6-0002 through §6.6-0006 gate as suite properties,
  but raw fuzz-run outcomes MUST NOT gate a conformance verdict (§7.2-0002). *Test:*
  `test_conform_fuzz_finding_promoted`.

### 6.7 Modality 4 — negative / decline vectors and never-panic

- **KISS-CONFORM-6.7-0001** — KISS-Conform MUST provide **negative/decline vectors** in which a
  malformed, out-of-claim, truncated, mislabeled, unknown-op, or empty-intersection input
  yields the correct **typed decline** — or, where KISS-Consume's four-category taxonomy makes
  residue the correct answer, the correct pinned-signature **residue** classification (§3
  "Residue") — and MUST NOT yield a crash, panic, or a silent empty result; an implementation
  MUST NOT pass a negative vector by producing a silent empty result in place of the required
  typed decline or residue. *Test:* `test_conform_negative_typed_decline`.
- **KISS-CONFORM-6.7-0002** — Each negative/decline vector MUST pin **exactly one** required
  output — either the **exact decline code/token** the implementation must emit, or, for a
  residue case, the **exact residue structural signature** — and MUST NOT accept either
  interchangeably; KISS-Conform MUST NOT accept an arbitrary or unpinned error, nor a residue
  where a decline code is pinned, nor a decline where a residue signature is pinned. *Test:*
  `test_conform_decline_code_pinned`.
- **KISS-CONFORM-6.7-0003** — The **never-panic** obligation MUST be delivered both as fixed
  negative vectors and as a **fuzz campaign** (never-panic is a fuzz-testable property); an
  implementation MUST NOT rely on fixed vectors alone for the never-panic obligation. *Test:*
  `test_conform_never_panic_fuzz`.
- **KISS-CONFORM-6.7-0004** — Under every negative vector and every fuzz input, the
  implementation **process MUST survive** within the suite's pinned bounds: no panic, abort,
  crash, or out-of-bounds read, and no exceeding of the **pinned maximum wall-clock per input**
  (the "hang" bound) or the **pinned maximum resident-memory bound** (the "unbounded
  allocation" bound); those two bounds MUST be bundled and versioned with the fuzzer corpus as
  suite parameters, so "hang" and "unbounded allocation" are decided against a fixed threshold.
  *Test:* `test_conform_process_survives`.
- **KISS-CONFORM-6.7-0005** — For each negative vector, whether the correct output is a
  **typed decline** or a **residue** MUST be determined by the KISS-Consume four-category
  refusal taxonomy (KISS-CONSUME §6.4): a vector in a decline category MUST pin a decline code
  (§6.7-0002) and a vector in a residue category MUST pin the residue's structural signature; a
  vector MUST NOT be authored to accept whichever the implementation happens to emit. *Test:*
  `test_conform_decline_vs_residue_taxonomy`.

### 6.8 Determinism-class-aware comparators

- **KISS-CONFORM-6.8-0001** — The **exact-byte** comparator MUST be a bit/byte-identical
  compare (memcmp), and KISS-Conform MUST NOT relax a clause whose declared class is exact-byte
  to a tolerance or order-invariant comparator. *Test:* `test_conform_exact_byte_comparator`.
- **KISS-CONFORM-6.8-0002** — The **ULP/tolerance** comparator MUST compare within the op's
  **declared per-target ULP** bound and MUST NOT be a byte compare across implementations or
  languages; it MUST apply to any op whose decomposition transitively contains a transcendental
  atom, and KISS-Conform MUST NOT claim cross-language numeric identity for such an op. *Test:*
  `test_conform_ulp_comparator`.
- **KISS-CONFORM-6.8-0003** — KISS-Conform MUST evaluate a transcendental atom under the ULP
  the contract declares, and MUST **reject** a declared ULP that exceeds the KISS-Ops maximum
  ULP ceiling (KISS-OPS §6.8-0001); a declared ULP looser than the ceiling MUST NOT be accepted.
  *Test:* `test_conform_ulp_ceiling_enforced`.
- **KISS-CONFORM-6.8-0004** — The **order-invariant/nondeterministic** comparator MUST NOT
  require byte-exact reproduction across implementations or runs; it applies to ops whose
  **declared class** is order-invariant/nondeterministic — floating-point atomic-combine
  reductions/scatter (e.g. scatter atomic-add, scatter_add) are **illustrative** of that
  declared class, not an independent selector — and the tolerance used MUST be the one declared
  in the contract Guarantees, never an implementation-chosen implicit default. *Test:*
  `test_conform_nondeterministic_comparator`.
- **KISS-CONFORM-6.8-0005** — For the complex-transcendental ops `carg`, `clog`, `csqrt`,
  `cexp`, KISS-Conform MUST apply the **split comparator**: an exact-bit comparator on the sign
  bit of every zero-valued result component combined with a ULP/tolerance comparator on the
  magnitude. The split comparator is a hybrid of the three canonical classes and MUST NOT be
  registered as a fourth enum member (§6.0-0001). *Test:* `test_conform_split_comparator`.
- **KISS-CONFORM-6.8-0006** — The comparator MUST be **selected** by the clause's declared
  determinism/fidelity class travelling with the artifact — a provided kernel's contract
  Guarantees carries the class (KISS-SYNTH §6.5-0004) and each op advertises its class (KISS-OPS
  §7.4-0001) — so a consumer and KISS-Conform pick the same comparator; KISS-Conform MUST NOT
  let a test author override the class-selected comparator. *Test:*
  `test_conform_comparator_selection_rule`.
- **KISS-CONFORM-6.8-0007** — Whole-kernel tier-2 numeric bit-identity MUST be admitted only
  when **every** op in the resolved-to-floor DAG is exact-byte class; any ULP/tolerance or
  order-invariant/nondeterministic op in the DAG MUST downgrade the whole-kernel claim off
  numeric bit-identity (KISS-EMIT §6.7-0009). *Test:* `test_conform_whole_kernel_downgrade`.
- **KISS-CONFORM-6.8-0008** — Comparator selection MUST be **total and unambiguous** under a
  fixed precedence ordering: an **op-named refinement** (the split comparator §6.8-0005 for
  `carg`/`clog`/`csqrt`/`cexp`; structural op-DAG equality §6.9 for the tier-1 round-trip) MUST
  **override** the declared-class default selection of §6.8-0006 for the named ops; where no
  op-named refinement applies, the declared determinism/fidelity class (§6.8-0006) MUST select
  the comparator. No op may be left with two admissible comparators. *Test:*
  `test_conform_comparator_precedence`.
- **KISS-CONFORM-6.8-0009** — The exact-byte comparator MUST be the only admissible comparator
  for POD/wire/ABI structural clauses; the op families cited alongside it
  (integer/bitwise/select/exact-byte float moves and the order-invariant-in-value monoid
  reductions and prefix scans) are **illustrative** of ops that CARRY the exact-byte declared
  class, and the declared class (§6.8-0006) is authoritative when an op appears to match two
  comparator-clause enumerations. *Test:* `test_conform_exact_byte_admissibility`.

### 6.9 The structural op-DAG equality comparator (tier-1 round-trip)

- **KISS-CONFORM-6.9-0001** — KISS-Conform MUST implement **structural op-DAG equality** as:
  two KISS-Ops op DAGs are equal iff, after resolving every non-primitive node to the primitive
  floor, placing nodes and edges in KISS-Ops canonical order, and normalizing
  commutative/associative operands, their **node sets, edge sets, and per-node OpAttrs byte
  channels are identical**. *Test:* `test_conform_structural_dag_equality`.
- **KISS-CONFORM-6.9-0002** — The tier-1 round-trip comparator MUST be this **structural**
  comparator (owned by KISS-Conform), NOT a byte-compare of emitted source; a predicate tighter
  than structural op-DAG equality (for example byte-identity of emitted source across
  languages) MUST NOT be required for the tier-1 round-trip and is non-conforming. *Test:*
  `test_conform_structural_not_source_bytes`.
- **KISS-CONFORM-6.9-0003** — KISS-Conform MUST use structural op-DAG equality as the
  comparator for the KISS-Emit↔KISS-Consume tier-1 round-trip over the declared subset
  (KISS-EMIT §6.7-0007); it MUST NOT substitute the tier-2 numeric comparator for the tier-1
  structural round-trip. *Test:* `test_conform_roundtrip_tier1`.

### 6.10 The expressibility oracle

- **KISS-CONFORM-6.10-0001** — KISS-Conform MUST define the **expressibility oracle**: a residue
  region is judged **expressible** iff its region signature is a member of the enumerated
  expressible-signature set at the **referenced KISS-Ops op-set version**; the oracle MUST be
  evaluated at the referenced op-set version, not an implementation's current op set. The byte
  form of a region signature and the membership test are pinned by **Appendix F**
  (§6.10-0006). *Test:*
  `test_conform_expressibility_oracle`.
- **KISS-CONFORM-6.10-0002** — The expressibility oracle MUST drive the KISS-Consume
  **unrecognized-but-expressible vs inexpressible-residue** split (KISS-CONSUME §6.4): a residue
  whose signature is in the set MUST be classified unrecognized-but-expressible, and one whose
  signature is not MUST be classified inexpressible-residue. *Test:*
  `test_conform_expressibility_split`.
- **KISS-CONFORM-6.10-0003** — KISS-Conform MUST include a **deliberately mislabeled-kernel
  vector** — one whose name/tokens disagree with its structure — and MUST demand the
  **structurally-correct** lift (recognition is structure-based; substring/keyword sniffing is
  non-conforming, KISS-CONSUME); an implementation MUST NOT pass by honoring the label over the
  structure. *Test:* `test_conform_mislabeled_kernel_structural`.
- **KISS-CONFORM-6.10-0004** — KISS-Conform MUST verify the emit↔consume relation as a **join**
  (a shared structural comparison of a DAG emitted then lifted), NOT as a directed dependency
  edge between the two sub-standards; the corresponding suite check is named
  `test_consume_emit_are_siblings_no_edge`. *Test:* `test_conform_consume_emit_siblings`.
- **KISS-CONFORM-6.10-0005** — The enumerated **expressible-signature set** MUST be a concrete,
  versioned artifact **stamped by the KISS-Ops op-set version**, **owned by the KISS-Ops
  editor**, regenerated on each op-set-version bump, **serialized in the byte format pinned by
  Appendix F (KISS-CONFORM-6.10-0006)**, and **bundled WITH the suite** for that
  version (like the namespace-registry snapshot, §6.3-0003), so an offline foreign reader holds
  the exact set the oracle evaluates against; an oracle evaluated against an unbundled,
  differently-versioned, **or Appendix-F-nonconforming** set MUST be rejected. *Test:*
  `test_conform_expressibility_set_bundled`.
- **KISS-CONFORM-6.10-0006** — The enumerated expressible-signature set MUST be serialized in
  the byte format pinned by **Appendix F**; a reader MUST reject with a typed decline a set that
  omits a REQUIRED field, carries an unknown field, or violates an enumerant, and MUST reject an
  oracle evaluated against a set that does not conform to Appendix F. Two regenerations of the
  set at the same `ops_op_set_version` MUST be byte-identical. *Test:*
  `test_conform_expressible_signature_set_schema`.

### 6.11 The cross-standard document lint

- **KISS-CONFORM-6.11-0001** — KISS-Conform MUST own a **cross-standard document lint** that
  checks the two-tier round-trip clauses of KISS-Emit are semantically equivalent to
  KISS-Consume's per the **enumerated correspondence table** (KISS-EMIT §6.7-0008); the lint
  MUST read both texts (their clause sidecars), NOT an emitter's runtime behavior, and MUST fail
  on a correspondence-table row whose paired clauses are not equivalent. *Test:*
  `test_conform_emit_consume_correspondence_lint`.
- **KISS-CONFORM-6.11-0002** — KISS-Conform MUST verify the KISS-Emit **neutrality-audit**
  governance preconditions against the AUDIT role's **recorded manifest** (the recorded
  lowering-decision partition and the passed neutrality audit, KISS-EMIT §6.5 / §8.2-0004)
  before the Emit freeze transition is signed. *Test:* `test_conform_neutrality_audit_manifest`.
- **KISS-CONFORM-6.11-0003** — The cross-standard lint MUST also cover the **determinism-enum
  import sites**: it MUST verify that every sub-standard importing the KISS-Ops determinism/
  fidelity enum (KISS-Synth, KISS-Consume, KISS-Emit, KISS-Contract, and KISS-Conform §6.0)
  spells it verbatim and does not textually fork it; a forked spelling MUST fail the lint.
  *Test:* `test_conform_determinism_import_site_lint`.

### 6.12 The consumer-verify SHOULD — an executable, non-gating check

- **KISS-CONFORM-6.12-0001** — KISS-Conform MUST **ship an executable check** implementing the
  one operative consumer-verify **SHOULD** (a consumer SHOULD verify a received kernel against
  its contract, per KISS-SYNTH §6.4-0004 and KISS-CONTRACT): the MUST is on KISS-Conform to
  provide the check; the check's own force on a consumer remains SHOULD. *Test:*
  `test_conform_consumer_verify_check`.
- **KISS-CONFORM-6.12-0002** — The consumer-verify check MUST run a received kernel against its
  contract's **declared precision** (under the ULP/tolerance comparator), its **determinism
  class**, and its **accept-predicate** (`= structure_key`), reusing the oracle-differential
  harness (§6.5) and the determinism-class comparators (§6.8); it MUST NOT introduce a separate
  comparator or a separate oracle. *Test:* `test_conform_consumer_verify_reuses_oracle`.
- **KISS-CONFORM-6.12-0003** — The consumer-verify check MUST be catalogued as a governance
  SHOULD and MUST NOT be build-gating in the default profile (§6.1-0005, §7.2-0001); a failing
  consumer-verify check MUST NOT fail the suite build in the default profile. *Test:*
  `test_conform_consumer_verify_not_gating`.

### 6.13 Per-sub-standard forward-reference closure

Each clause below **closes a specific conformance obligation** a sub-standard defers to
KISS-Conform, and cites the deferring sub-standard clause where useful. The mechanism a clause
relies on is defined in §6.1–§6.12; this section is the checkable proof that every deferral is
honored. Where a deferral bundles multiple obligations, each obligation is stated as its own
atomic clause with its own append-only ID and dedicated test.

**KISS-Announce**

- **KISS-CONFORM-6.13-0001** — KISS-Conform MUST apply the **byte-exact comparator** (§6.8-0001)
  to every KISS-Announce POD structural clause (sizes, offsets, magic, tags, decline codes,
  hashes, bounds, bitset positions) and MUST forbid tolerance or order-invariant comparison on
  them. *Test:* `test_conform_announce_exact_byte_pod`.
- **KISS-CONFORM-6.13-0019** — KISS-Conform MUST supply golden byte-vectors (§6.4) for the
  KISS-Announce 56-byte handshake envelope and the version-negotiation frames. *Test:*
  `test_conform_announce_golden_vectors`.
- **KISS-CONFORM-6.13-0002** — KISS-Conform MUST provide the KISS-Announce **freeze-gate** test
  (the Appendix-B checklist, AUDIT-signed, KISS-ANNOUNCE §8), MUST prove the two seam-hello seeds
  are **byte-identical via golden hex** (not struct equality), MUST run a foreign reader
  (written outside the reference language) over the envelope, and MUST supply negative vectors
  for the unknown-bit **reserved-and-ignore** and the empty-profile **hard-fail-never-panic**
  obligations. *Test:* `test_conform_announce_freeze_gate`.

**KISS-Classify**

- **KISS-CONFORM-6.13-0003** — KISS-Conform MUST apply the byte-exact comparator (§6.8-0001) to
  the KISS-Classify `structure_key` token codec and the `target_capability` token grammar, MUST
  **bundle** the versioned, machine-readable namespace-registry snapshot WITH the suite so an
  offline foreign reader holds a complete copy (§6.3-0003), and MUST supply golden token vectors.
  *Test:* `test_conform_classify_exact_byte_and_registry_bundle`.
- **KISS-CONFORM-6.13-0004** — KISS-Conform MUST provide the KISS-Classify **freeze-gate** test
  (suite exists and passes with complete bidirectional traceability plus the foreign-reader wire
  check, KISS-CLASSIFY §8) and MUST keep KISS-Classify's freeze-gate check UNFROZEN-eligible only
  once the namespace vocabulary has been exercised by usage on a target **outside the initial
  reference-hardware namespace**; it MUST NOT sign a Classify freeze on evidence drawn solely
  from the initial reference-hardware namespace. *Test:* `test_conform_classify_freeze_gate`.

**KISS-Ops**

- **KISS-CONFORM-6.13-0005** — KISS-Conform MUST implement the three comparators of §6.8 as the
  selection targets of the canonical enum imported from KISS-OPS §6.0-0001 (§6.0-0001):
  exact-byte ops evaluated with memcmp (KISS-OPS §6.0-0002), ULP ops with the declared-ULP
  comparator (KISS-OPS §6.0-0003), and nondeterministic ops without a byte-exact requirement
  (KISS-OPS §6.0-0004). *Test:* `test_conform_ops_per_class_comparators`.
- **KISS-CONFORM-6.13-0006** — KISS-Conform MUST **own** the independent CPU-oracle differential
  harness (§6.5) that resolves an op's decomposition to the primitive floor and compares under
  its declared class sharing no lowering module with any reference impl, MUST provide the KISS-Ops
  freeze-gate tests (≥2 dissimilar impls agree on floor semantics and decompositions, KISS-OPS
  §8-0005 / §8-0006), and MUST use the advertised per-op class (KISS-OPS §7.4-0001) to select
  the comparator. *Test:* `test_conform_ops_oracle_and_freeze_gate`.
- **KISS-CONFORM-6.13-0007** — KISS-Conform MUST evaluate each transcendental atom under the
  contract's declared per-target ULP and **reject** any declared ULP exceeding the KISS-Ops
  ceiling (§6.8-0003, KISS-OPS §6.8-0001), MUST NOT claim cross-language numeric identity for a
  transcendental (KISS-OPS §6.8), and MUST implement the **split comparator** (§6.8-0005) for
  `carg` / `clog` / `csqrt` / `cexp`. *Test:* `test_conform_ops_transcendental_and_split`.
- **KISS-CONFORM-6.13-0008** — KISS-Conform MUST supply **golden byte-vectors** mapping each
  KISS-Ops OpAttrs record to its exact canonical little-endian hex, covering **every field at its
  resolved default value with no elision** (KISS-OPS §6.19-0013), and MUST verify that KISS-Grammar
  and KISS-Contract embed the OpAttrs blob as **opaque bytes** compared byte-exact (the
  `test_ops_opattrs_opaque_embedding_byte_compare` obligation); this is the OpAttrs golden-vector
  freeze gate the suite MUST carry. *Test:* `test_conform_ops_opattrs_golden_hex`.

**KISS-Grammar**

- **KISS-CONFORM-6.13-0009** — KISS-Conform MUST compare the pinned KISS-Grammar region **wire
  bytes byte-exact** (no tolerance/order-invariant), MUST key frozen-shape conformance on
  `grammar_version`, and MUST supply golden token/region vectors at the pinned shape. *Test:*
  `test_conform_grammar_region_exact_byte`.
- **KISS-CONFORM-6.13-0010** — KISS-Conform MUST **own** the fuzzer and differential harness
  (§6.6) that exercise the KISS-Grammar region grammar, wire form, and still-growing-op-set
  growth model, MUST verify that expressible regions round-trip through the wire form AND that
  non-expressible or unknown/absent-from-pinned-Ops-version ops **decline cleanly (never panic)**,
  and MUST provide the KISS-Grammar freeze-gate test plus foreign-reader golden token vectors
  (retro-fitting the gate Grammar lacked, KISS-GRAMMAR §8). *Test:*
  `test_conform_grammar_fuzz_and_freeze_gate`.

**KISS-Contract**

- **KISS-CONFORM-6.13-0011** — KISS-Conform MUST **own and ship** the executable consumer-verify
  check (§6.12) for the consumer-verify SHOULD (KISS-SYNTH §6.4-0004, KISS-CONTRACT): running a
  received kernel against its contract's declared precision (ULP/tolerance), determinism class,
  and accept-predicate (`= structure_key`), reusing the oracle-differential harness and the
  determinism-class comparators, non-build-gating in the default profile. *Test:*
  `test_conform_contract_consumer_verify`.
- **KISS-CONFORM-6.13-0012** — KISS-Conform MUST byte-compare the KISS-Contract seven-section
  schema/framing (KISS-CONTRACT §6.0-0001). *Test:* `test_conform_contract_schema_byte_compare`.
- **KISS-CONFORM-6.13-0020** — KISS-Conform MUST NOT byte-compare the optional free-text blurb /
  `human_annotation` (KISS-CONTRACT §6.4-0001). *Test:* `test_conform_contract_blurb_excluded`.
- **KISS-CONFORM-6.13-0021** — KISS-Conform MUST verify the `audited_status` **derivation**
  (`test_contract_audited_status_derived`, KISS-CONTRACT §6.4). *Test:*
  `test_conform_contract_audited_status`.
- **KISS-CONFORM-6.13-0022** — KISS-Conform MUST resolve the KISS-Contract Semantics DAG to the
  primitive floor via the oracle (§6.5, KISS-CONTRACT §6.4-0005). *Test:*
  `test_conform_contract_semantics_to_floor`.
- **KISS-CONFORM-6.13-0023** — KISS-Conform MUST supply negative vectors so
  malformed/inconsistent contracts fail **LOUDLY** as a typed decline over the hard-reject
  transport (never panic, never silent-empty, KISS-CONTRACT §6.1-0004 / §6.2-0005). *Test:*
  `test_conform_contract_malformed_decline`.
- **KISS-CONFORM-6.13-0024** — KISS-Conform MUST gate the KISS-Contract freeze on ≥2 dissimilar
  impls binding+launching+reasoning from the text alone (KISS-CONTRACT §8-0004). *Test:*
  `test_conform_contract_freeze_gate`.

**KISS-Synth / Provision**

- **KISS-CONFORM-6.13-0013** — KISS-Conform MUST **own** the fuzz/negative modality (§6.6, §6.7)
  that drives the KISS-Synth provision/JIT path to prove **never-panic** across the whole §6.6
  decline taxonomy (KISS-SYNTH §8-0006), MUST select the comparator from the provided kernel's
  contract determinism class (exact-byte→byte, ULP→declared-ULP, order-invariant→declared
  tolerance, no implicit default; KISS-SYNTH §6.5-0004 / §6.5-0004a), MUST check returned-contract
  required-core content-validity in the harness (KISS-SYNTH §6.2-0008a), and MUST reuse the
  oracle-differential harness and comparators for the consumer-verify SHOULD (KISS-SYNTH
  §6.4-0004). *Test:* `test_conform_synth_fuzz_decline_and_comparator`.

**KISS-Consume**

- **KISS-CONFORM-6.13-0014** — KISS-Conform MUST resolve a **lifted** Semantics DAG to the
  primitive floor and compare under the op's determinism class (§6.5), and MUST exercise the
  **four refusal categories** (not-a-kernel / wrong-op-class / unrecognized-but-expressible /
  inexpressible-residue) and the never-panic obligation with negative vectors, testing both that
  a claim's clauses pass AND that out-of-claim inputs **decline-or-residue** cleanly (KISS-CONSUME
  §9.1-0002). *Test:* `test_conform_consume_lift_oracle_and_refusals`.
- **KISS-CONFORM-6.13-0015** — KISS-Conform MUST define the **expressibility oracle** (§6.10) at
  the referenced KISS-Ops op-set version (KISS-CONSUME §6.4 / region-signature membership) driving
  the unrecognized-but-expressible vs inexpressible-residue split against the bundled, op-set-
  version-stamped expressible-signature set (§6.10-0005) — the byte form of a region signature
  and the membership test are pinned by **Appendix F** (§6.10-0006) — MUST require a deliberately
  **mislabeled-kernel** vector demanding the structurally-correct lift (§6.10-0003), and MUST
  verify the emit↔consume round-trip as a **join** (`test_consume_emit_are_siblings_no_edge`,
  §6.10-0004), not a DAG edge. *Test:* `test_conform_consume_expressibility_and_join`.

**KISS-Emit**

- **KISS-CONFORM-6.13-0016** — KISS-Conform MUST **own** the cross-standard document lint (§6.11)
  checking the KISS-Emit two-tier round-trip clauses are semantically equivalent to KISS-Consume's
  per the enumerated correspondence table (KISS-EMIT §6.7-0008, reading both texts, not an
  emitter-behavior test), MUST verify the neutrality-audit governance preconditions against the
  AUDIT role's recorded manifest (§6.11-0002), and MUST sign the Emit Draft→Frozen transition via
  the **AUDIT role only** after a non-C-family emitter and a second consumer certify (KISS-EMIT
  §8.2-0004). *Test:* `test_conform_emit_cross_standard_lint_and_freeze`.
- **KISS-CONFORM-6.13-0017** — KISS-Conform MUST **own** the structural op-DAG equality comparator
  (§6.9; resolve to floor + canonical order + normalize commutative/associative + compare OpAttrs
  bytes, KISS-EMIT §6.7-0007), MUST admit tier-2 numeric bit-identity **only same-language**
  (language-identity token) on-device (byte-equal `target_capability`) and **only** when every
  resolved-to-floor op is exact-byte class (§6.8-0007, KISS-EMIT §6.7-0006 / §6.7-0009), MUST use
  the emitted kernel's Semantics DAG resolved to the floor as the numeric oracle, and MUST run the
  IR-DAG fuzzer to every backend including the non-C-family emitter (§6.6-0006). *Test:*
  `test_conform_emit_structural_and_tier2`.

**ALL (cross-cutting)**

- **KISS-CONFORM-6.13-0018** — KISS-Conform MUST provide the single **build-fails-on-untested-MUST**
  traceability gate (§6.1, §6.2) that fails on any normative MUST with no mapped citing test and on
  any test citing a non-existent or retired clause ID, across **every sidecar in the suite's
  declared coverage set** (the steward-published canonical full-suite covering all eight
  sidecars, §6.1-0008) — the mechanism each sub-standard forward-references verbatim in its §0
  header, §8 freeze gate, and §9 traceability matrix — MUST key every suite artifact on
  `(sub-standard, wire/ABI schema version)` and freeze it immutable (§6.3, §8-0003 / §8-0004),
  MUST run the reference implementation through the unmodified public canonical suite with **no
  exemption** (§8-0007), and MUST sign every maturity transition via the AUDIT role (§8-0006).
  *Test:* `test_conform_all_build_gate_and_keying`.

---

## 7. Capability, Profile & Extension model

### 7.1 Mandatory core

- **KISS-CONFORM-7.1-0001** — The KISS-Conform **mandatory core** MUST be: the bidirectional
  traceability matrix (§6.1) and its build-fail gate (§6.2); the four test modalities (golden
  byte-vectors §6.4, the independent CPU-oracle differential harness §6.5, the IR-DAG fuzzer
  §6.6, and negative/decline vectors §6.7); and the determinism-class-aware comparators (§6.8),
  applied to whatever DAG-aligned subset of sub-standards the suite's **declared coverage set**
  (§6.1-0008) names. An implementation that cannot produce the traceability gate and the four
  modalities does not conform to KISS-Conform at all. *Test:* `test_conform_mandatory_core`.
- **KISS-CONFORM-7.1-0002** — An input to the conformance harness itself that is out of the
  harness's claimed coverage MUST produce a **typed decline, never a panic** (the same
  typed-decline discipline the harness enforces on the sub-standards it tests); the harness MUST
  NOT crash on an out-of-coverage input. *Test:* `test_conform_harness_typed_decline`.

### 7.2 Negotiable options

- **KISS-CONFORM-7.2-0001** — KISS-Conform MUST NOT build-gate the **consumer-verify** check in
  the **default profile** (§6.12-0003); a failing consumer-verify check MUST NOT fail the suite
  build in the default profile. *Test:* `test_conform_consumer_verify_profile`.
- **KISS-CONFORM-7.2-0003** — KISS-Conform MUST define an **opt-in trust-but-verify profile**
  that promotes the consumer-verify check to a build-gating check; an implementation that does
  not advertise the opt-in profile MUST NOT gate on it, and the opt-in gate MUST take effect only
  for an implementation that advertises trust-but-verify. *Test:*
  `test_conform_optin_profile_defined`.
- **KISS-CONFORM-7.2-0004** — The opt-in trust-but-verify profile MUST NOT alter the
  default-profile clause set, so a default-profile result and an opt-in-profile result remain
  distinguishable. *Test:* `test_conform_optin_profile_isolated`.
- **KISS-CONFORM-7.2-0002** — The IR-DAG fuzzer's seeds/corpora MUST be **versioned and shared
  with the suite** so a foreign implementer reproduces a failing case deterministically; the
  corpus MUST NOT be treated as a normative surface (a clause set), so that it cannot ossify an
  incidental impl choice — corpus membership MUST NOT gate conformance, only the pinned golden,
  oracle, and negative vectors (including fuzz findings promoted per §6.6-0007) and the
  traceability gate do. *Test:* `test_conform_fuzzer_corpus_versioned`.

### 7.3 Extension

- **KISS-CONFORM-7.3-0001** — KISS-Conform defines no data or op vocabulary; a new comparator MUST
  NOT be added as a member of the canonical determinism/fidelity enum (which is owned by KISS-Ops,
  §6.0-0001). A comparator **refinement** (the split comparator §6.8-0005, the structural op-DAG
  equality comparator §6.9) MUST enter only where an owning sub-standard clause names it, and MUST
  NOT be introduced by KISS-Conform as a free-standing new class. *Test:*
  `test_conform_no_new_comparator_class`.

---

## 8. Versioning & Lifecycle

KISS-Conform tracks the umbrella's **two version axes** and keys conformance on exactly one of
them. Each **suite** is a versioned artifact stamped `(sub_standard_token, wire/ABI schema
version)`; the reference **crate** carries an ordinary semver that moves on any code release.

- **KISS-CONFORM-8-0001** — KISS-Conform MUST key conformance **per-sub-standard per wire/ABI
  schema version** (the KISS-Announce envelope version, the KISS-Classify `structure_key` version,
  the KISS-Ops op-vocabulary schema version, the `grammar_version`, the `contract_version`, the
  `EMIT_ABI_VERSION`, the KISS-Synth PRSP version) and MUST NOT key conformance on a crate semver.
  *Test:* `test_conform_keys_on_schema_version`.
- **KISS-CONFORM-8-0002** — A pure code refactor that **preserves the observable bytes/behavior**
  MUST bump only the reference-crate semver and MUST NOT mint a new suite version; KISS-Conform
  MUST NOT treat a byte-preserving refactor as a new schema version. *Test:*
  `test_conform_refactor_no_new_suite`.
- **KISS-CONFORM-8-0003** — The traceability matrix, golden vectors, oracle vectors, negative
  vectors, and bundled vocabulary snapshots MUST all be **stamped** `(sub_standard_token, schema
  version)` and bundled into the single versioned suite artifact for that pair. *Test:*
  `test_conform_suite_stamped_per_version`.
- **KISS-CONFORM-8-0004** — A **Frozen** version's suite MUST be **immutable and archived**: its
  clause set, tests, and vectors are frozen with it; retired clause IDs stay burned and MUST NOT
  be reused or re-mapped in any later version; new or changed clauses MUST land only against a new
  schema version's suite. *Test:* `test_conform_frozen_suite_immutable`.
- **KISS-CONFORM-8-0005** — "**Conforms**" MUST mean passing the **unmodified canonical** suite
  artifact (§8-0010) for a specific `(sub-standard, schema version)`; a result from a **modified**
  suite MUST NOT back a conformance claim and MUST be ineligible for the steward registry (the v1
  enforcement surface, umbrella §8.4 / §9.3). *Test:* `test_conform_unmodified_suite_only`.
- **KISS-CONFORM-8-0006** — A sub-standard MUST NOT be promoted Draft→Frozen until the
  **freeze-gate checklist** (Appendix B, umbrella §5.3) is met as **objective, checkable items**:
  ≥2 structurally dissimilar implementations (distinct codebases, disjoint lowering-module
  manifests per §6.5-0002) interoperate on the golden vectors; a **foreign reader built from the
  document alone** (written outside the reference language) reproduces or parses the exact wire
  bytes — the authoritative field list being **endianness, pointer width, structure padding,
  field offsets, magic, and token spellings, all byte-exact** (this superset governs where
  umbrella §5.3 enumerates fewer) — and reports every ambiguity (each resolved in text or filed as
  a numbered RFC before freeze); the sub-standard's suite exists and passes with complete
  bidirectional traceability; the CPU oracle reproduces the pinned results (**circularity check
  clean** by provenance tag, §6.5-0003); and negative/decline coverage passes (never-panic fuzz
  clean within the pinned time/memory bounds, §6.7-0004). The transition MUST be **signed by the
  KISS-Conform AUDIT role**, not the authoring editor. *Test:*
  `test_conform_freeze_gate_checklist` (checklist gate; AUDIT-signed).
- **KISS-CONFORM-8-0007** — The **reference implementation** MUST run the **same public,
  unmodified** canonical KISS-Conform suite every other implementation runs, with **no exemption,
  no golden-vector authoring privilege, no comparator relaxation, and no traceability waiver**;
  its pass is evidence on exactly the same terms as any other implementation's. *Test:*
  `test_conform_reference_impl_no_exemption`.
- **KISS-CONFORM-8-0008** — Deprecation MUST itself be tested: the suite MUST assert a
  **retire-by-floor** minimum schema version implementors may still rely on for the tested
  sub-standard, and the immutable frozen suite for a superseded version MUST remain runnable and
  archived for the stated support window. *Test:* `test_conform_retire_by_floor`.
- **KISS-CONFORM-8-0009** — KISS-Conform MUST NOT be promoted Draft→Frozen until it **self-applies**
  its own freeze gate: its own suite exists and passes with complete bidirectional traceability,
  and an AUDIT role **independent of KISS-Conform's authoring editor** signs the transition (the
  cross-cutting root is audited like any other sub-standard). *Test:* `test_conform_self_freeze_gate`
  (checklist gate; AUDIT-signed).
- **KISS-CONFORM-8-0010** — There MUST be, per `(sub-standard, schema version)`, a single
  steward-published, versioned **canonical suite artifact** (the matrix plus the exact
  golden/oracle/negative vector set and the Appendix-F-conforming expressible-signature set) that
  is authoritative for a conformance verdict; a foreign or
  independently-built suite MUST reproduce that canonical vector set **byte-for-byte** before it
  may issue a conformance verdict, and a verdict from a suite whose vector set differs from the
  canonical artifact MUST NOT back a conformance claim. *Test:*
  `test_conform_canonical_suite_authoritative`.

---

## 9. Conformance

An implementation conforms to KISS-Conform at a given suite schema version if it (a) produces the
bidirectional traceability matrix and the build-fail gate exactly per §6.1–§6.2, implements the
four modalities per §6.4–§6.7, and selects comparators per §6.0 / §6.8 for that version; (b)
passes the unmodified canonical KISS-Conform suite (§8-0010) for KISS-Conform at that version;
and (c) satisfies the DAG prerequisite closure. KISS-Conform's upstream edges are **test
dependencies** on all eight other sub-standards: a KISS-Conform suite that tests a sub-standard
MUST harvest that sub-standard's sidecar and evaluate its clauses (Conform cannot test a
sub-standard it has not imported the sidecar of). KISS-Conform is depended on by **none**, so no
downstream claim forces a co-claim of Conform. Out-of-coverage inputs to the harness yield typed
declines, never panics, per §7.1-0002. The modified-suite prohibition (§8-0005) is the umbrella's
mark policy (umbrella §9.3), enforced via registry ineligibility, and is not restated as a
free-standing extra clause.

### 9.1 Clause → KISS-Conform test traceability matrix

Uniquely, KISS-Conform's own clauses are enforced by the very gate they define: the suite build
FAILS if any KISS-CONFORM clause below lacks a mapped citing test (generate-time, §6.2-0001) or
any test cites a non-existent/retired KISS-CONFORM clause ID (§6.2-0002/0003), and a conformance
run fails if any mapped test fails at run time (§6.2-0006), applied reflexively. Clause IDs are
mirrored in the machine-readable sidecar (conforming to the Appendix E schema) kept in sync by
the traceability lint.

| Clause ID | Named conformance test |
|---|---|
| KISS-CONFORM-6.0-0001 | `test_conform_determinism_enum_imported` |
| KISS-CONFORM-6.0-0002 | `test_conform_comparator_selected_by_class` |
| KISS-CONFORM-6.0-0003 | `test_conform_structural_artifacts_byte_exact` |
| KISS-CONFORM-6.1-0001 | `test_conform_traceability_matrix_exists` |
| KISS-CONFORM-6.1-0002 | `test_conform_every_clause_has_test` |
| KISS-CONFORM-6.1-0003 | `test_conform_every_test_cites_clause` |
| KISS-CONFORM-6.1-0004 | `test_conform_matrix_derived_from_sidecar` |
| KISS-CONFORM-6.1-0005 | `test_conform_coverage_normative_only` |
| KISS-CONFORM-6.1-0006 | `test_conform_pod_sidecar_generated` |
| KISS-CONFORM-6.1-0007 | `test_conform_sidecar_sole_authority` |
| KISS-CONFORM-6.1-0008 | `test_conform_declared_coverage_set` |
| KISS-CONFORM-6.2-0001 | `test_conform_build_fails_untested_must` |
| KISS-CONFORM-6.2-0002 | `test_conform_build_fails_dangling_cite` |
| KISS-CONFORM-6.2-0003 | `test_conform_build_fails_retired_cite` |
| KISS-CONFORM-6.2-0004 | `test_conform_build_fails_drift` |
| KISS-CONFORM-6.2-0005 | `test_conform_gate_is_generate_time` |
| KISS-CONFORM-6.2-0006 | `test_conform_mapped_tests_pass_runtime` |
| KISS-CONFORM-6.3-0001 | `test_conform_matrix_composite_key` |
| KISS-CONFORM-6.3-0002 | `test_conform_matrix_carries_metadata` |
| KISS-CONFORM-6.3-0003 | `test_conform_suite_self_contained_bundle` |
| KISS-CONFORM-6.3-0004 | `test_conform_frozen_matrix_immutable` |
| KISS-CONFORM-6.4-0001 | `test_conform_golden_byte_vectors` |
| KISS-CONFORM-6.4-0002 | `test_conform_golden_vectors_fully_pinned` |
| KISS-CONFORM-6.4-0003 | `test_conform_golden_vectors_hex_rows` |
| KISS-CONFORM-6.5-0001 | `test_conform_oracle_differential_harness` |
| KISS-CONFORM-6.5-0002 | `test_conform_oracle_independent` |
| KISS-CONFORM-6.5-0003 | `test_conform_reject_circular_vector` |
| KISS-CONFORM-6.5-0004 | `test_conform_oracle_resolves_to_floor` |
| KISS-CONFORM-6.5-0005 | `test_conform_oracle_authoring_independence` |
| KISS-CONFORM-6.5-0006 | `test_conform_boundary_decision_compute_dtype` |
| KISS-CONFORM-6.5-0007 | `test_conform_oracle_tighter_than_declared_ulp` |
| KISS-CONFORM-6.5-0008 | `test_conform_oracle_vector_coverage_complete` |
| KISS-CONFORM-6.5-0009 | `test_conform_oracle_vector_stores_wide_precision_value` |
| KISS-CONFORM-6.6-0001 | `test_conform_fuzzer_generates_valid_dags` |
| KISS-CONFORM-6.6-0002 | `test_conform_fuzzer_every_backend` |
| KISS-CONFORM-6.6-0003 | `test_conform_fuzzer_cross_backend_agreement` |
| KISS-CONFORM-6.6-0004 | `test_conform_fuzzer_roundtrip_join` |
| KISS-CONFORM-6.6-0005 | `test_conform_fuzzer_feeds_negative` |
| KISS-CONFORM-6.6-0006 | `test_conform_fuzzer_includes_non_c_emitter` |
| KISS-CONFORM-6.6-0007 | `test_conform_fuzz_finding_promoted` |
| KISS-CONFORM-6.7-0001 | `test_conform_negative_typed_decline` |
| KISS-CONFORM-6.7-0002 | `test_conform_decline_code_pinned` |
| KISS-CONFORM-6.7-0003 | `test_conform_never_panic_fuzz` |
| KISS-CONFORM-6.7-0004 | `test_conform_process_survives` |
| KISS-CONFORM-6.7-0005 | `test_conform_decline_vs_residue_taxonomy` |
| KISS-CONFORM-6.8-0001 | `test_conform_exact_byte_comparator` |
| KISS-CONFORM-6.8-0002 | `test_conform_ulp_comparator` |
| KISS-CONFORM-6.8-0003 | `test_conform_ulp_ceiling_enforced` |
| KISS-CONFORM-6.8-0004 | `test_conform_nondeterministic_comparator` |
| KISS-CONFORM-6.8-0005 | `test_conform_split_comparator` |
| KISS-CONFORM-6.8-0006 | `test_conform_comparator_selection_rule` |
| KISS-CONFORM-6.8-0007 | `test_conform_whole_kernel_downgrade` |
| KISS-CONFORM-6.8-0008 | `test_conform_comparator_precedence` |
| KISS-CONFORM-6.8-0009 | `test_conform_exact_byte_admissibility` |
| KISS-CONFORM-6.9-0001 | `test_conform_structural_dag_equality` |
| KISS-CONFORM-6.9-0002 | `test_conform_structural_not_source_bytes` |
| KISS-CONFORM-6.9-0003 | `test_conform_roundtrip_tier1` |
| KISS-CONFORM-6.10-0001 | `test_conform_expressibility_oracle` |
| KISS-CONFORM-6.10-0002 | `test_conform_expressibility_split` |
| KISS-CONFORM-6.10-0003 | `test_conform_mislabeled_kernel_structural` |
| KISS-CONFORM-6.10-0004 | `test_conform_consume_emit_siblings` |
| KISS-CONFORM-6.10-0005 | `test_conform_expressibility_set_bundled` |
| KISS-CONFORM-6.10-0006 | `test_conform_expressible_signature_set_schema` |
| KISS-CONFORM-6.11-0001 | `test_conform_emit_consume_correspondence_lint` |
| KISS-CONFORM-6.11-0002 | `test_conform_neutrality_audit_manifest` |
| KISS-CONFORM-6.11-0003 | `test_conform_determinism_import_site_lint` |
| KISS-CONFORM-6.12-0001 | `test_conform_consumer_verify_check` |
| KISS-CONFORM-6.12-0002 | `test_conform_consumer_verify_reuses_oracle` |
| KISS-CONFORM-6.12-0003 | `test_conform_consumer_verify_not_gating` |
| KISS-CONFORM-6.13-0001 | `test_conform_announce_exact_byte_pod` |
| KISS-CONFORM-6.13-0002 | `test_conform_announce_freeze_gate` |
| KISS-CONFORM-6.13-0003 | `test_conform_classify_exact_byte_and_registry_bundle` |
| KISS-CONFORM-6.13-0004 | `test_conform_classify_freeze_gate` |
| KISS-CONFORM-6.13-0005 | `test_conform_ops_per_class_comparators` |
| KISS-CONFORM-6.13-0006 | `test_conform_ops_oracle_and_freeze_gate` |
| KISS-CONFORM-6.13-0007 | `test_conform_ops_transcendental_and_split` |
| KISS-CONFORM-6.13-0008 | `test_conform_ops_opattrs_golden_hex` |
| KISS-CONFORM-6.13-0009 | `test_conform_grammar_region_exact_byte` |
| KISS-CONFORM-6.13-0010 | `test_conform_grammar_fuzz_and_freeze_gate` |
| KISS-CONFORM-6.13-0011 | `test_conform_contract_consumer_verify` |
| KISS-CONFORM-6.13-0012 | `test_conform_contract_schema_byte_compare` |
| KISS-CONFORM-6.13-0013 | `test_conform_synth_fuzz_decline_and_comparator` |
| KISS-CONFORM-6.13-0014 | `test_conform_consume_lift_oracle_and_refusals` |
| KISS-CONFORM-6.13-0015 | `test_conform_consume_expressibility_and_join` |
| KISS-CONFORM-6.13-0016 | `test_conform_emit_cross_standard_lint_and_freeze` |
| KISS-CONFORM-6.13-0017 | `test_conform_emit_structural_and_tier2` |
| KISS-CONFORM-6.13-0018 | `test_conform_all_build_gate_and_keying` |
| KISS-CONFORM-6.13-0019 | `test_conform_announce_golden_vectors` |
| KISS-CONFORM-6.13-0020 | `test_conform_contract_blurb_excluded` |
| KISS-CONFORM-6.13-0021 | `test_conform_contract_audited_status` |
| KISS-CONFORM-6.13-0022 | `test_conform_contract_semantics_to_floor` |
| KISS-CONFORM-6.13-0023 | `test_conform_contract_malformed_decline` |
| KISS-CONFORM-6.13-0024 | `test_conform_contract_freeze_gate` |
| KISS-CONFORM-7.1-0001 | `test_conform_mandatory_core` |
| KISS-CONFORM-7.1-0002 | `test_conform_harness_typed_decline` |
| KISS-CONFORM-7.2-0001 | `test_conform_consumer_verify_profile` |
| KISS-CONFORM-7.2-0002 | `test_conform_fuzzer_corpus_versioned` |
| KISS-CONFORM-7.2-0003 | `test_conform_optin_profile_defined` |
| KISS-CONFORM-7.2-0004 | `test_conform_optin_profile_isolated` |
| KISS-CONFORM-7.3-0001 | `test_conform_no_new_comparator_class` |
| KISS-CONFORM-8-0001 | `test_conform_keys_on_schema_version` |
| KISS-CONFORM-8-0002 | `test_conform_refactor_no_new_suite` |
| KISS-CONFORM-8-0003 | `test_conform_suite_stamped_per_version` |
| KISS-CONFORM-8-0004 | `test_conform_frozen_suite_immutable` |
| KISS-CONFORM-8-0005 | `test_conform_unmodified_suite_only` |
| KISS-CONFORM-8-0006 | `test_conform_freeze_gate_checklist` |
| KISS-CONFORM-8-0007 | `test_conform_reference_impl_no_exemption` |
| KISS-CONFORM-8-0008 | `test_conform_retire_by_floor` |
| KISS-CONFORM-8-0009 | `test_conform_self_freeze_gate` |
| KISS-CONFORM-8-0010 | `test_conform_canonical_suite_authoritative` |

Every normative clause above appears in this matrix exactly once; the KISS-Conform build fails if
any clause ID lacks a mapped citing test, and a conformance run fails if any mapped test fails at
run time (bidirectional traceability, the mechanism this sub-standard defines in §6.1–§6.2 and
applies reflexively to itself, umbrella §3.3).

---

## 10. Governance

- **Editor of record:** the KISS-Conform editor assignment is **proposed, pending ratification**
  in the umbrella governance record. The editor holds the pen, allocates clause IDs (append-only,
  never reused after retirement), and — because KISS-Conform tests every sub-standard — solicits
  comment from **every** sub-standard editor before deciding a cross-party-visible change to a
  comparator, a modality, the traceability keying, or the freeze-gate checklist. A change to the
  freeze-gate checklist or the comparator-selection rule is coordinated across all affected parties
  as a numbered RFC before it is wired.
- **AUDIT role separation:** the KISS-Conform **AUDIT role** signs every maturity transition
  across the suite (not the authoring/design editor), attempts a second dissimilar implementation
  from the document alone, and reports every ambiguity as a numbered RFC (umbrella §5.3, §7.3;
  design charter §5, informative). For KISS-Conform's own freeze, the AUDIT role MUST be
  independent of KISS-Conform's authoring editor (§8-0009).
- **Steward:** ThinkersJournal hosts the spec, the conformance-suite distribution, and the
  free-certification registry; it publishes the canonical suite artifact per `(sub-standard, schema
  version)` (§8-0010), runs the unmodified suite on request as resources permit, and lists passing
  implementations. The op-name vocabulary and the expressible-signature set are owned by KISS-Ops
  and the dtype/`target_capability` vocabulary by KISS-Classify, not by a KISS-Conform registry.
- **Ratifier / maturity transitions:** the AUDIT role signs each transition; a Draft→Frozen
  transition requires the freeze gate of §8-0006 (and, for KISS-Conform itself, §8-0009), umbrella
  §5.3.
- **License:** this specification is dedicated to the public domain under CC0 1.0 Universal;
  reference crates are MIT-OR-Apache-2.0; the KISS-Conform suite is permissive-to-run. Per the
  umbrella mark policy (umbrella §9.3), a **modified** conformance suite does not back a conformance
  claim (§8-0005); that policy is enforced via steward-registry ineligibility, not restated as an
  extra normative clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC contribution,
  with defensive termination, per the umbrella.
- **Conformance posture:** self-certification with published results plus the steward-maintained
  registry is the authoritative record of verified implementations (umbrella §8); the reference
  implementation runs the same unmodified public canonical suite with no exemption (§8-0007).

---

## Appendix A — Vector families & provenance (informative)

**A.1 Golden byte-vector families.** The exact-byte evidence (§6.4) mirrors each sub-standard's
Appendix-A "bytes on the wire" rows (a bijection, §6.4-0003): the KISS-Announce 56-byte handshake
envelope and version-negotiation frames and the byte-identity of the two seam-hello seeds (golden
hex, not struct equality, §6.13-0002, §6.13-0019); the KISS-Classify `structure_key` token codec
and `target_capability` token grammar with the bundled namespace-registry snapshot (§6.13-0003);
the KISS-Ops OpAttrs canonical little-endian records, every field at its resolved default with no
elision, and their opaque-embedding byte-compare in KISS-Grammar and KISS-Contract (§6.13-0008);
the KISS-Grammar region wire form keyed on `grammar_version` (§6.13-0009); the KISS-Contract
seven-section schema/framing over the hard-reject transport (§6.13-0012); and the KISS-Synth PRSP
provision/decline frames and decline-code values.

**A.2 Oracle-differential families.** The CPU oracle (§6.5), derived solely from the KISS-Ops §6
semantics tables and reference decompositions and sharing no lowering module with any reference
impl (declared-manifest set-intersection, §6.5-0002), covers: KISS-Ops per-op pinned semantics,
the primitive floor, reference decompositions, transcendental atoms under the declared ULP ceiling,
and the complex-arith split comparator; KISS-Contract Semantics-DAG resolution to the floor and
`audited_status` derivation; KISS-Consume lifted-Semantics-DAG resolution and mislabeled-kernel
structural correctness; KISS-Emit emitted-kernel-Semantics-DAG-as-oracle and the tier-1 structural
round-trip; and the KISS-Synth consumer-verify of a provided kernel against its contract's declared
precision/determinism. Two oracle-hygiene disciplines the harness itself observes are pinned as
normative oracle clauses: the **boundary golden vector** for discontinuous ops (`cmp_*`, a
`select` condition, `sign`, `step`) whose operands are narrowed to the op's compute dtype before
the decision, so a differential value distinct in wide precision but equal after rounding does not
flip the oracle spuriously (§6.5-0006); and the **transcendental oracle-accuracy floor** — the
oracle evaluates each transcendental atom wider than the compute dtype and rounds once, keeping its
own error strictly under the declared ULP tolerance it enforces so the differential measures the
implementation and not the oracle (§6.5-0007). Every vector carries the `oracle`
derivation-provenance tag (§6.5-0003).

**A.3 Fuzzer families.** The IR-DAG fuzzer (§6.6) drives every suite-registered fuzz-target backend
— including at least one non-C-family emitter (§6.6-0006) — and the lift direction, asserting
cross-backend agreement under the determinism-class comparators and the emit↔consume round-trip
join, and feeds malformed/near-boundary DAGs to the negative modality. Its seeds/corpora are
versioned and shared (§7.2-0002) but are not a normative surface; any concrete disagreement it
finds is minimized and promoted into a pinned vector (§6.6-0007).

**A.4 Negative/decline families.** The never-panic battery (§6.7) covers every sub-standard's
mandatory-core typed-decline obligation within the pinned time/memory bounds (§6.7-0004):
KISS-Announce unknown-bit reserved-and-ignore and empty-profile hard-fail-never-panic;
KISS-Classify unknown token/layout reject; KISS-Grammar unknown/absent-op typed decline;
KISS-Contract malformed/inconsistent typed decline over the hard-reject transport; KISS-Synth §6.6
never-panic taxonomy on the provision/JIT path; KISS-Consume §9.1-0002 out-of-claim
decline-or-residue across the four-category refusal taxonomy (decline vs residue chosen per
§6.7-0005); and KISS-Emit dtype/op/schedule typed decline on the JIT path.

**A.5 Provenance / acknowledgments.** The bidirectional traceability matrix, the four modalities,
the determinism-class-aware comparators, the per-sub-standard-per-version keying, and the
adversarial-outsider freeze checklist derive from a conformance-harness reference crate (the
traceability lint, golden-vector harness, oracle-differential engine, IR-DAG fuzzer, and
negative-vector battery). The `const_lit` C-ism — a finite-const encoding that passed every
happy-path golden vector while leaking a C-family lowering choice — is recorded as the cautionary
proof that a non-C-family emitter (§6.6-0006) and a foreign reader from the document alone
(Appendix B) are load-bearing, not optional. This rationale is informative only; no normative
clause depends on it. Project and crate names in this appendix are non-normative provenance and
examples only; no normative clause names any project, and every normative role is the generic
provider / consumer / implementation / kernel / contract / target.

---

## Appendix B — The freeze-gate checklist (informative rendering of the §8-0006 objective items)

The **normative** freeze gate is §8-0006 (and, for KISS-Conform itself, §8-0009); this appendix
renders its objective, checkable items. A sub-standard advances Draft→Frozen only when every item
is met and demonstrated, and the KISS-Conform AUDIT role signs the transition (never the authoring
editor):

1. **≥2 structurally dissimilar implementations** interoperate on the sub-standard's golden
   vectors — distinct codebases, **disjoint lowering-module manifests** (implementations whose
   declared lowering-module manifests intersect do NOT count as dissimilar, §6.5-0002); verified
   by the differential harness passing between them.
2. A **non-native foreign reader built from the document alone** (an implementation written outside
   the reference language named in the governance record) reproduces or parses the exact wire
   bytes, with the authoritative field list — **endianness, pointer width, structure padding,
   field offsets, magic, and token spellings, all byte-exact** — checked against the golden
   vectors. This field list is the authoritative superset; where umbrella §5.3 enumerates fewer,
   KISS-CONFORM §8-0006 governs.
3. The foreign reader **reports every ambiguity** encountered, and each is either resolved in the
   text or filed as a **numbered RFC** before freeze (an unresolved ambiguity blocks freeze).
4. The sub-standard's KISS-Conform suite **exists and passes** at the candidate wire/ABI schema
   version, with **complete bidirectional clause-to-test traceability** (build fails on no untested
   MUST and no dangling/retired cite; every mapped test passes at run time, §6.2-0006).
5. Every normative MUST / MUST NOT / SHALL in §6–§8 maps to ≥1 named test that cites it and every
   test cites a live clause ID (**direction-2 orphan check clean**).
6. The **independent CPU oracle** (derived from the §6 semantics tables, disjoint lowering-module
   manifest) reproduces the pinned results — no vector carries a derivation-provenance tag other
   than `oracle` (**circularity check clean by provenance**, §6.5-0003).
7. **Negative/decline coverage** is present and passing: out-of-claim, malformed, unknown-op, and
   empty-intersection inputs produce the correct typed decline/residue, never a panic, within the
   pinned wall-clock and resident-memory bounds (**never-panic fuzz campaign clean**, §6.7-0004).
8. **Sub-standard-specific gate items** are met where the sub-standard names them: KISS-Ops (≥2
   dissimilar impls agree on floor semantics and decompositions via oracle-differential, §8-0005 /
   §8-0006; OpAttrs golden hex frozen, §6.19-0013); KISS-Grammar / KISS-Classify (golden token
   vectors + foreign reader, §8-0006); KISS-Contract (≥2 dissimilar impls bind+launch+reason from
   the text alone, §8-0006); KISS-Emit (neutrality audit passed against the recorded manifest + a
   non-C-family emitter + a second consumer certify, §8.2-0004, AUDIT-signed); KISS-Consume (foreign
   reader reproduces residue/refusal tokens); KISS-Synth (fuzz/negative modality over the §6.6
   never-panic taxonomy passes, §8-0006).
9. The transition is **signed by the KISS-Conform AUDIT role** and recorded in the sub-standard's §0
   front-matter and the RFC record; the **reference implementation ran the same unmodified public
   canonical suite with no exemption**.

---

## Appendix C — Forward-reference closure cross-map (informative)

Each row shows a conformance obligation an eighth sub-standard defers to KISS-Conform, and the
Conform clause that closes it. This is the informative rendering of §6.13; on any discrepancy the
§6.13 clauses (and the mechanism clauses they cite) govern.

| Deferring sub-standard | Obligation | Closed by |
|---|---|---|
| KISS-Announce | Exact-byte comparator for all POD wire fields | §6.13-0001 (via §6.4, §6.8-0001) |
| KISS-Announce | Envelope + version-negotiation golden vectors | §6.13-0019 (via §6.4) |
| KISS-Announce | Freeze gate + seam-hello byte-identity + foreign reader + reserved-and-ignore / empty-profile negatives | §6.13-0002 (via §8-0006, §6.7) |
| KISS-Classify | Exact-byte `structure_key`/token codec/`target_capability` + bundled namespace registry + golden tokens | §6.13-0003 (via §6.3-0003, §6.8-0001) |
| KISS-Classify | Freeze gate; stays UNFROZEN until usage exercises a target outside the initial reference-hardware namespace | §6.13-0004 (via §8-0006) |
| KISS-Ops | Canonical determinism enum + three per-class comparators | §6.13-0005 (via §6.0-0001, §6.8) |
| KISS-Ops | Oracle-differential harness + freeze gates §8-0005/0006 + per-op class advertisement | §6.13-0006 (via §6.5, §6.8-0006) |
| KISS-Ops | Transcendental ULP ceiling + no cross-language identity + complex split comparator | §6.13-0007 (via §6.8-0002/0003/0005) |
| KISS-Ops | OpAttrs golden hex (every field, no elision) + opaque-embedding byte-compare | §6.13-0008 (via §6.4) |
| KISS-Grammar | Exact-byte region wire form + `grammar_version` keying | §6.13-0009 (via §6.8-0001, §8-0001) |
| KISS-Grammar | Typed-decline/never-panic + fuzzer/differential + freeze gate + golden tokens | §6.13-0010 (via §6.6, §6.7, §8-0006) |
| KISS-Contract | Consumer-verify SHOULD as an executable check | §6.13-0011 (via §6.12) |
| KISS-Contract | Exact-byte schema/framing | §6.13-0012 (via §6.4) |
| KISS-Contract | Free-text blurb excluded from byte-compare | §6.13-0020 (via §6.4-0001) |
| KISS-Contract | `audited_status` derivation | §6.13-0021 (via §6.5) |
| KISS-Contract | Semantics DAG resolved to floor | §6.13-0022 (via §6.5) |
| KISS-Contract | Malformed/inconsistent typed decline over hard-reject transport | §6.13-0023 (via §6.7) |
| KISS-Contract | Freeze gate: ≥2 dissimilar impls from text alone | §6.13-0024 (via §8-0006) |
| KISS-Synth | Fuzz/negative over never-panic taxonomy + determinism-class comparator selection + returned-contract validity + consumer-verify reuse | §6.13-0013 (via §6.6, §6.7, §6.8-0006, §6.12) |
| KISS-Consume | Oracle-differential lift resolution + four-category refusal / never-panic negatives | §6.13-0014 (via §6.5, §6.7) |
| KISS-Consume | Expressibility oracle + mislabeled-kernel structural lift + emit↔consume sibling join | §6.13-0015 (via §6.10) |
| KISS-Emit | Emit↔Consume cross-standard document lint + neutrality-audit manifest + AUDIT-signed freeze | §6.13-0016 (via §6.11, §8-0006) |
| KISS-Emit | Structural op-DAG equality tier-1 + tier-2 same-language-only + whole-kernel aggregation + emitted-kernel-as-oracle | §6.13-0017 (via §6.9, §6.8-0007, §6.6-0006) |
| ALL (cross-cutting) | Build-fails-on-untested-MUST + per-sub-standard-per-version keying + reference-impl no exemption + AUDIT-signed transitions | §6.13-0018 (via §6.1, §6.2, §6.3, §8) |

---

## Appendix D — Open questions (informative)

These are recorded for the KISS-Conform RFC process and are **not** normative; none gates a
conformance run. They are surfaced so the design record is honest about what is not yet pinned:

1. **Artifact-byte verification depth** (from KISS-Synth): how deeply the consumer-verify check and
   the harness must verify returned artifact bytes against `revision_hash` or the contract's declared
   Guarantees, beyond the framing+identity checks the wire pins.
2. **Consumer-verify profile scope** (§7.2-0003): whether the opt-in trust-but-verify profile that
   promotes the consumer-verify SHOULD to a gating check needs any further policy so the one operative
   SHOULD does not sprawl.
3. **Fuzzer corpus governance** (§7.2-0002): the exact versioning/sharing policy so a foreign
   implementer reproduces a failing case deterministically without the corpus ossifying an incidental
   choice.
4. **Cross-standard lint scope** (§6.11): whether the lint should generalize beyond the Emit↔Consume
   correspondence and the determinism-enum import sites to other paired surfaces (e.g. the OpAttrs
   opaque-embedding claims in Grammar/Contract) to catch textual drift the traceability matrix cannot
   see.
5. **Steward-run certification throughput** (umbrella §8.3): the SLA/queue policy for free
   certification, and whether a self-certified-but-not-yet-steward-verified result carries an interim
   registry status.
6. **Deprecation / retire-by-floor detail** (§8-0008): exactly what the suite asserts for a
   Deprecated version's retire-by-floor minimum, and how long a superseded version's immutable frozen
   suite must remain runnable/archived.
7. **Nondeterministic-class reproducibility floor** (§6.8-0004): whether a declared tolerance is
   always sufficient for order-invariant/nondeterministic ops, or whether Conform needs a
   statistical/repeated-run acceptance criterion to avoid a flaky pass/fail on floating-point atomic
   combines.

> **Resolved (formerly open) — expressibility-oracle maintenance (§6.10).** The update cadence and
> ownership of the enumerated expressible-signature set are now pinned normatively: the set is owned
> by the **KISS-Ops editor**, regenerated on each **KISS-Ops op-set-version bump**, stamped by that
> op-set version, and bundled WITH the suite (§6.10-0005). The set's byte **format** is now pinned
> by **Appendix F** (§6.10-0006). This item is retained here for the design
> record and is no longer an open question.

---

## Appendix E — Sidecar schema (normative)

This appendix is **normative** and is referenced by §6.1-0007. It pins the machine-readable clause
sidecar so that two independently-built conformance suites enumerate the identical clause set for a
sub-standard. The sidecar — not the prose — is the sole authoritative clause source (§6.1-0007);
prose is reconciled to it only by the deterministic clause-ID-token match of §6.2-0004.

- **Encoding.** Each sidecar MUST be a single UTF-8, JSON-encoded document (the `validusage.json`
  analog) with byte-deterministic serialization: object member keys in ascending Unicode code-point
  order, no insignificant whitespace, and LF line endings, so the file is byte-comparable (§6.0-0003).
- **Top-level fields (all REQUIRED).**
  - `sub_standard_token` — string, the sub-standard identifier (e.g. `KISS-OPS`).
  - `wire_abi_schema_version` — string, the wire/ABI schema version this sidecar is stamped for
    (§8-0001).
  - `generator` — string enumerant, one of `canonical-schema` (POD tiers, §6.1-0006) or
    `hand-maintained` (non-POD tiers), recording how the sidecar was produced.
  - `clauses` — array of clause objects, in ascending clause-ID order.
- **Clause object fields (all REQUIRED).**
  - `clause_id` — string matching `KISS-<SUB>-<section>-<nnnn>` (umbrella §3.3).
  - `section` — string, the owning section (e.g. `6.8`).
  - `keyword` — string enumerant, one of `MUST` / `MUST NOT` / `SHALL` / `SHOULD` / `MAY`.
  - `normative` — boolean; `true` iff `keyword` ∈ {`MUST`, `MUST NOT`, `SHALL`}; coverage is
    measured against `normative: true` clauses only (§6.1-0005).
  - `determinism_class` — string enumerant drawn verbatim from the KISS-Ops enum
    `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`, or `n/a` for a non-numeric
    structural clause; selects the comparator (§6.8).
  - `status` — string enumerant, one of `live` or `retired`; a `retired` ID is permanently burned
    and MUST NOT be re-mapped (§6.2-0003).
- **Determinism.** Two regenerations of a POD-tier sidecar from the same canonical schema MUST be
  byte-identical; a mismatch is drift (§6.2-0004, §6.1-0006). A sidecar that omits a REQUIRED field,
  carries an unknown field, or violates an enumerant MUST be rejected (§6.1-0007).

---

## Appendix F — Expressible-signature-set schema (normative)

This appendix is **normative** and is referenced by §6.10-0005 and KISS-CONFORM-6.10-0006.
It pins the byte format of the enumerated expressible-signature set so that two
independently-built suites, and an offline foreign reader, enumerate the identical
membership set the expressibility oracle (§6.10-0001) evaluates against.

**Encoding.** The set MUST be a single UTF-8, JSON-encoded document with byte-deterministic
serialization: object member keys in ascending Unicode code-point order, no insignificant
whitespace, and LF line endings, so the file is byte-comparable (§6.0-0003, §8-0010).

**Top-level fields (all REQUIRED).**
- `owner` — the string `KISS-OPS` (the set is KISS-Ops-editor-owned, §6.10-0005).
- `ops_op_set_version` — the KISS-Ops op-set version stamp (§6.10-0005).
- `opattrs_wire_version` — the KISS-Ops OpAttrs wire-freeze version (per-node OpAttrs bytes
  are part of a signature; §6.8-0007 / KISS-OPS §8-0008).
- `generator` — an enumerant naming how the set was produced (e.g. `canonical-regen`).
- `signatures` — an array of signature objects, ordered ascending by the unsigned-byte
  lexicographic order of each signature's `bytes` (matching KISS-Grammar §6.4-0010).

**Signature bytes (the membership key).** A signature is the residue region resolved to the
primitive floor (§6.9). Its canonical byte form is the **KISS-Grammar §6.4-0010 canonical
subtree serialization** computed as a **structure-only projection**: every operand dtype-role
token is first normalized to the KISS-Grammar §6.6-0006 dtype-role **wildcard** token, so
op-DAGs that are structurally identical but differ only in operand dtypes collapse to a single
signature. The **sole** exception is a `Cast` node's target dtype, which rides inside that
node's §6.19 OpAttrs blob and is retained verbatim — it is the only dtype-bearing path in a
signature; no operand dtype-role token survives the wildcarding anywhere else. Membership is
KISS-Conform structural op-DAG equality (§6.9): two signatures are the same iff their
wildcarded §6.4-0010 serializations are byte-identical.

**Signature object fields (all REQUIRED).**
- `nodes` — the ordered node list in §6.9 canonical order, each `Op{name; opattrs_hex}` or
  `Bind{index}`; `name` is a KISS-Ops op name at the primitive floor; `opattrs_hex` is the
  per-node §6.19 OpAttrs wire bytes as lowercase hex (empty string for the empty-attr form; a
  `Cast`'s target dtype retained).
- `edges` — per node, the operand node-indices, in **KISS-Grammar §6.4-0010 canonical operand
  order**: for a node whose op KISS-Ops declares **commutative** (§6.2-0005), operands are
  ordered by the §6.4-0010 pinned total order — ascending unsigned-byte-lexicographic
  comparison of each operand's **wildcarded** canonical subtree serialization — so
  `add(Bind 0, Bind 1)` and `add(Bind 1, Bind 0)` are the *same* signature; for a **positional**
  (non-commutative) op, operand order is preserved as authored, so `sub(Bind 0, Bind 1)` and
  `sub(Bind 1, Bind 0)` are *distinct* signatures.
- `bytes` — the §6.4-0010 wildcarded canonical subtree serialization over which membership is
  decided, lowercase hex. Within `bytes`, a `Bind{index}` leaf serializes as the byte `0x00`
  followed by its `input_index` as a `u32` **little-endian** (§6.4-0010); the commutative
  byte-lex order above is computed over these same wildcarded subtree serializations, so the
  ordering, the projection, and the Bind encoding compose consistently.
- `signature_hash` — the hash over `bytes` (the decidable-membership key, §6.4-0005).

**Determinism.** Two regenerations of the set at the same `ops_op_set_version` MUST be
byte-identical. A set that omits a REQUIRED field, carries an unknown field, or violates an
enumerant MUST be rejected; an oracle evaluated against a set that does not conform to this
appendix MUST be rejected (§6.10-0005).

**Golden example (informative transcription target for the conformance suite).** A one-signature
set over the primitive `add(Bind 0, Bind 1)` at op-set version `1`, OpAttrs wire version `1`
(add is commutative; its edge list `[0,1]` is already the §6.4-0010 byte-lex-canonical order,
since `Bind 0` sorts before `Bind 1`):

    {"generator":"canonical-regen","opattrs_wire_version":"1","ops_op_set_version":"1","owner":"KISS-OPS","signatures":[{"bytes":"<hex>","edges":[[],[],[0,1]],"nodes":["Bind{0}","Bind{1}","Op{add;}"],"signature_hash":"<hex>"}]}

A second golden over the positional (non-commutative) primitive `sub(Bind 0, Bind 1)` — whose
edge list `[0,1]` is order-significant (distinct from `[1,0]`), exercising the Bind index
encoding and the positional-order rule — is pinned alongside the `add` golden by the
conformance suite:

    {"generator":"canonical-regen","opattrs_wire_version":"1","ops_op_set_version":"1","owner":"KISS-OPS","signatures":[{"bytes":"<hex>","edges":[[],[],[0,1]],"nodes":["Bind{0}","Bind{1}","Op{sub;}"],"signature_hash":"<hex>"}]}

---

*End of KISS-Conform (Draft proposal). This sub-standard is informative in §0–§5 and normative in
§6+ (and in the Appendix E sidecar schema §6.1-0007 references and the Appendix F
expressible-signature-set schema §6.10-0006 references); every binding requirement carries
an identified clause `KISS-CONFORM-<section>-<nnnn>` with a mapped `test_conform_<slug>` test.
KISS-Conform is the cross-cutting root of the suite's test relation: it depends on and TESTS all
eight other sub-standards, is depended on by none, imports the determinism/fidelity enum from
KISS-Ops, and closes every forward-referenced conformance obligation in §6.13. It defines the
bidirectional clause↔test traceability matrix and the build-fail gate, the four test modalities, the
determinism-class-aware comparators, per-sub-standard-per-version keying, and the adversarial-outsider
freeze gate that — signed by the AUDIT role, with the reference implementation running the same
unmodified public canonical suite with no exemption — is what makes the other eight provable. Project
and product names appear only in non-normative examples, provenance, and the reference-implementation
pointer; normative clauses use only the generic roles provider, consumer, implementation, kernel,
contract, and target.*
