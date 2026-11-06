# KISS glossary

An **index**, not a second set of definitions. Each term is defined once, in the §3 *Terms & Definitions*
of the sub-standard that owns it (or in its Appendix B glossary where one exists). This page gives the
one-line sense and says where to read the normative text, so a reader can find the owner without
opening ten documents. If a gloss here disagrees with the owning §3, the owning §3 wins.

The same vocabulary serves the sibling repository `kiss-ref` (the differential reference implementation;
it is a conformance reference, not an authority over the spec).

Sub-standards: **Ops** `spec/ops.md`, **Classify** `spec/classify.md`, **Grammar** `spec/grammar.md`,
**Contract** `spec/contract.md`, **Announce** `spec/announce.md`, **Synth** `spec/synth.md`,
**Consume** `spec/consume.md`, **Emit** `spec/emit.md`, **Conform** `spec/conform.md`,
**umbrella** `spec/umbrella.md`.

| Term | Sense | Owner |
|---|---|---|
| **accept_predicate** | The Contract Identity field that holds the kernel's `structure_key`: the admissibility predicate a consumer matches against. | Contract §3 |
| **admissibility predicate** | A total function over an invocation's derivation inputs that decides whether a kernel specialization applies to it. The `structure_key` is its serialization. | Classify §3 |
| **advertisable op** | A KISS-Ops op *name* decorated with pattern attributes. It is not a second op vocabulary: every advertisable op re-bases onto an Ops name. | Grammar §3 |
| **availability record** | The identity triple `(structure_key, op_identity, revision_hash)` a provider announces for a kernel. | Announce §3 |
| **canonical suite artifact** | The single steward-published, versioned bundle of matrix, vectors and signature set that a conformance claim is made against. | Conform §3 |
| **cell class** | The coarse structural class of a cell, one of seven: `ctr`, `ndx`, `rdc`, `ew1`, `ew2`, `ew3`, `oth`. A closed set; an eighth is a schema-version event. | Classify §6.5-0006 |
| **clause ID** | The stable identifier of one normative clause, `KISS-<STANDARD>-<section>-<nnnn>`. Retired IDs stay burned and are never reused. | umbrella §3.3, Conform §3 |
| **comparator** | The equality predicate a result is checked under; selected by the clause's declared determinism class, never chosen by the test author. | Conform §3, §6.8 |
| **contract** | The per-kernel, self-delimiting, seven-section document: Identity, Semantics, Interface, Dispatch, Capabilities, Guarantees, Provenance. | Contract §3 |
| **declared ULP** | A per-target accuracy bound, in units in the last place, that a transcendental atom's contract declares. | Ops §3 |
| **determinism class** | The canonical Ops enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` that selects a comparator. Owned by Ops; every other sub-standard imports it verbatim. | Ops §3 |
| **dtype** | A scalar element type from the pinned set of Classify §6.1. | Classify §3 |
| **golden byte-vector** | A pinned `input -> exact output hex` row, compared with the exact-byte comparator. | Conform §3 |
| **lift** | Recognition: mapping a kernel or source region onto a KISS-Ops op DAG. The inverse of emit. | Consume §3 |
| **ledger (`UNBACKED.tsv`)** | The file recording every normative clause with no harness test, with a category and a note. It only shrinks (a ratchet). | `conformance/UNBACKED.tsv` header |
| **MathFidelity** | The Ops compute-fidelity attribute (a two-member enum, `bit-stable` among them). | Ops §3 |
| **neutrality audit** | The blocking pre-freeze audit proving an emitter's lowering decisions partition cleanly between the neutral driver and the emitter. | Emit §3 |
| **op DAG** | A directed acyclic graph over KISS-Ops op names, possibly mixed abstraction level; resolvable to the primitive floor. | Ops §3, Contract §3 |
| **OpAttrs** | The per-op compile-time attribute record (reduce monoid and axes, out-of-bounds policy, and so on), carried as a canonical little-endian blob. | Ops §6.19 |
| **oracle (CPU oracle)** | The reference computation for an op, derived solely from the Ops op-semantics tables and reference decompositions. | Conform §3 |
| **primitive floor** | The mandatory-core Ops op set at which every reference decomposition terminates. | Ops §3 |
| **profile** | An integer `>= 1` naming a mutually-versioned set of seam features, negotiated in the Announce handshake. | Announce §3 |
| **provision** | A consumer asking a provider for a kernel by identity. Build-on-miss is the branch where the kernel does not yet exist. | Synth §3 |
| **re-basing** | Declaring an advertisable op by naming an existing or newly added Ops name and decorating it; additive, no shape bump. | Grammar §3 |
| **residue** | The recorded un-liftable remainder of a partial lift: a connected group of nodes the lifter could not map. | Consume §3 |
| **revision_hash** | An opaque 32-byte, provider-assigned identifier of one build of a kernel. | Announce §3, Contract §3 |
| **`sk7`** | The `structure_key` token prefix at schema version 7; the version is part of the token. | Classify §6.4-0003 |
| **sidecar** | A sub-standard's machine-readable clause file; the sole authoritative source the traceability matrix is built from. | Conform §3 |
| **specialization cell** | One layout/dtype/target class a kernel is built for; the thing a `structure_key` names. | Classify §3 |
| **structure_key** | The admissibility predicate identifying one specialization cell, serialized as a pipe-delimited token. A cache hit needs it and the `revision_hash`. | Classify §3 |
| **target_capability** | A namespaced `<namespace>:<capability-set>` compilation target, for example `cuda:sm89`. Matched byte-exact. | Classify §3 |
| **traceability matrix** | The join between every normative clause and the named test that exercises it, in both directions. Built by `tools/kiss_trace.py`. | Conform §3, §6.1 |
| **typed decline** | A structured refusal returned instead of a result, drawn from an enumerable code set; never a panic. | every sub-standard §3 |
| **untested MUST** | A normative clause that no test cites. The count that must reach 0 is the genuinely-untested figure in the ledger. | Conform §3 |
| **wire/ABI schema version** | The version axis conformance is keyed on, distinct from the reference crate's semver. | umbrella §5.1, Conform §3 |
