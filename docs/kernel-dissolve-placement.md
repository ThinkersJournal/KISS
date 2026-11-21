# Dissolving KISS-Kernel: placement of what survives, fate of every reference, and an audit for rules that live only inside a kernel (board 475)

Status: **informative placement note. No clause text. Nothing is deleted.** CireSnave ruled board 475 on 2026-11-20, verbatim:
*"I agree with your reasoning on what should be moved out of KISS-kernel, and that we should delete KISS-kernel after
that. Pass our ruling to pm-kernels and have them work with the KISS lane's agent on that, and to ensure that no other
part of KISS defines things that are purely *inside a single kernel*. I want kernel builders to have free reign on
anything in their kernsl that doesn't affect other kernels."* He reads this note before anything is removed. Read at
`origin/main` dd51c2c. Test per rule (pm-kernels): **does another kernel or the caller rely on it?** Companion to
`docs/kernel-scope-outside-visibility.md` (board 474).

## 1. How the audit was done, and what it cannot claim

- **Clause-by-clause pass, nine specs.** Every normative clause ID of Emit, Synth, Conform, Classify, Contract, Announce,
  Grammar, Consume and Ops was read in full by a reading agent (one per spec, Ops in two halves) and classified OUTSIDE,
  INSIDE-ONLY or BORDERLINE. I read Ops 6.0 (8 clauses) myself because neither Ops range covered it, and re-checked
  26 of the flagged quotes verbatim against `origin/main` (all 26 present). I did not re-read the other clauses; the
  agents' OUTSIDE calls stand on their reading.
- **Counts (clause IDs counted as `^- \*\*KISS-<SPEC>-` bullets, letter suffixes included):** Emit 66, Synth 136, Conform 136,
  Classify 131, Contract 131, Announce 98, Grammar 77, Consume 52, Ops 233 = **1,060 read, 0 unread**. The umbrella has
  no clause IDs (its 1.2 exclusion of in-ecosystem dispatch is quoted in the board 474 note). `spec/kernel.md` has 2 clauses, both about references.
- **Control, honestly stated.** pm-kernels asked that a known in-kernel rule be found by the same pass. **No audited spec
  contains one** (all agents report the barrier, integer-division and atomic-order rules are not in their spec; they live
  only in `kernel.md` and the Unpopped concept map). So the control could not be run inside the pass. What I can say: the same
  predicate classified those three rules as expected (barrier OUTSIDE, total integer division INSIDE-ONLY, atomic old
  value OUTSIDE) in every agent, and the pass did find real mechanism-style wording where it exists (Ops 6.10-0006, section 4
  group B). That shows the pass can see mechanism; it does not prove nothing was missed.
- **Limits.** The Conform agent said it judged about 130 clauses OUTSIDE by their role (suite, harness, wire format)
  rather than testing each one, and did not read Conform's appendices. Several agents treated governance and
  freeze-gate clauses as "meta" (they bind the standard, not a kernel) and counted them neither way. The classification
  predicate is mine and the agents'; a stricter or looser reading moves rows.

## 2. Placement of what survives KISS-Kernel

pm-kernels listed the items. Evidence is quoted from the specs; **every new field is a byte change and goes to CireSnave
verbatim.** Working field names below are mine, not proposals for a spelling.

| Surviving item | Proposed home | What moves (exact bytes/fields) | Evidence for the home |
|---|---|---|---|
| Co-residency / cooperative launch: **the device offers it** | **Announce 6.6 device record** | One new trailing field in each device record (a u8 flag, working name `cooperative_launch`, default 0 = not offered). Changes every `DVRS` body: **BREAKING**, same shape as `unnamed_types` (KISS-ANNOUNCE-6.6-0011) | 6.6 states that a device "limits what can be computed" and already carries per-device `store_dtypes`/`compute_dtypes`; 6.6-0009 already dispatches on the **intersection** of a kernel's declared set and the device's set. The capability bitset (7.2) is **provider-level** (FEAT bits like `DEVICE_QUERY`), the wrong grain for a per-device fact. Classify namespace tokens name what a kernel **uses** (`cm-`, `arith`), not what a device guarantees |
| Co-residency: **the kernel requires it** | **Contract 6.7 Capabilities** | One new optional field (working name `requires_co_resident_launch`, default false). Today 6.7-0001 lists exactly: accept_predicate, supported_dtype_set, awkward_layout_strategy, in_place_eligible_variants, index_width, determinism_class, precision_class, cost | Contract has no co-residency, cooperative or visibility text (reader's grep: `barrier\|co-resid\|cooperative\|visib\|memory model` hits only "consumer-visible" and "cross-party-visible"). Dispatch (6.6) is not the home: 6.6-0001 requires exactly one of two launch forms |
| Barrier must not span launches unless co-resident | Contract (rule tied to the field above) | No new bytes beyond the field; a rule that a kernel which waits on another launch's progress declares the requirement | CireSnave's case (a); the draft's Device barrier spans blocks of one launch only (see the board 474 note) |
| Kernel entry/exit guarantees (what a kernel assumes at entry, guarantees at exit) | **Contract 6.8 Guarantees** | New optional field(s), working name `entry_exit_visibility`, default "caller-ordered". 6.8-0001 lists: reference_function, per_backend_ulp_tiers, determinism_class, math_fidelity, accumulation_type, bit_stability, audited_status, cost_provenance | Guarantees is where a kernel's promises to callers live. Umbrella 1.2 leaves launch ordering to the ecosystem, so the field states a boundary promise, never a mechanism |
| Device-memory visibility table (which memory is visible to whom) | **Contract 6.5 Interface** (a short table plus one clause) | Informative table; at most one new per-operand field if a clause needs it | Interface already carries per-operand ABI facts, including `in_place` (6.5-0009) |
| Within-launch barrier, fence, race, atomic-order rules; the 5 drops of the board 474 note; `thread`/`subgroup` terms | **Back to Unpopped as source model** | None (they were never published as clauses) | CireSnave: *"free reign on anything in their kernels that doesn't affect other kernels"* |

**Checked: a new Contract field breaks older readers even when optional** (pm-kernels TASK, 2026-11-20). KISS-CONTRACT-6.1-0006(c) makes a
reader reject a field the schema does not define at the declared `contract_version`, and says forward compatibility comes "only by a
`contract_version` bump, never by tolerating unknown fields"; 8-0002 requires that bump for any field-schema change and 6.1-0008 requires a
document on the new schema to carry the new token, so an older reader rejects even a document that omits the optional field (6.1-0003).
Repo precedent: the optional `output_layouts` field (6.5-0014, "absent = no existing contract moves") was recorded as BREAKING in
the CHANGELOG. Controls, both run: an unknown field is rejected (`contract_output_layouts`: `output_layoutz` gives `UnknownField`; this is the
path an older reader takes for the new field) and versions 1/99/0 give `UnknownVersion` (`contract_framing`); an Interface block with only the
seven required fields parses, and one with `output_layouts` last parses. Limit: the older reader is modelled as "its schema lacks the name",
not built as a separate binary. **So every Contract row above, and group B of section 4, is BREAKING (a `contract_version` bump).**

**Consequence to name for him.** Removing KISS-Kernel leaves the **order and reach** of the scatter atomic combines
(KISS-OPS-6.11-0006, -0010, -0024) stated nowhere in KISS: Ops keeps only the pinned **results** (the band of 0024).
That follows from his ruling (within-launch atomic order is "inside" one kernel's launch and returns to Unpopped), and
he should see it stated.

## 3. Fate of every reference to KISS-Kernel (census at dd51c2c)

| Where | What | Proposed fate |
|---|---|---|
| `spec/kernel.md` (149 lines) | the skeleton; clauses KISS-KERNEL-6.1-0001, -8-0001 | delete after the moves; the two clause IDs are **retired, never reused** (append-only IDs) |
| `spec/sidecar/kernel.json` | sidecar for those two clauses | delete |
| `spec/ops.md` (13 mentions) | "Kernel-governed:" tags on 6.11-0006/-0010/-0024; the 6.22 "Kernel-governed ops" table; the downstream-edge row; front-matter | remove the tags and table; the three clauses keep their pinned result text. The 6.22 sentence "KISS-Ops has no barrier, fence, collective or subgroup op today" and its pointer need rewording, since it points at KISS-Kernel |
| `conformance/corpus/op_facts.json` (134 lines) | a `kernel_rules` field on every op (`[]` on 126) and a `fact_clauses` entry | remove the field: **a byte change to the corpus, BREAKING** (the 0.20.0 CHANGELOG added it as additive under schema `kiss-op-facts-v1`; removing it is not additive, so a schema bump is for his word). Also `tools/kiss_op_facts.py` (4 mentions) |
| `tools/kiss_kernel_refs.py`, `tools/test_kiss_kernel_refs.py`, `conformance/tests/kernel_refs.rs` | the reference lint and its tests | delete |
| `conformance/UNBACKED.tsv`, `conformance/COVERAGE_FLOOR.tsv` | rows for the two clauses | remove; the ratchet floor figures move and must be re-derived |
| `spec/umbrella.md` (3 mentions) | the KISS-Kernel row of the suite; edge text | remove the row and the edges; **KISS-Kernel was never added as a SUB capability bit** (7.2 lists `SPEAKS_*` for the nine others only), so no Announce byte moves |
| `README.md` (2), `ROADMAP.md` (1), `docs/union-coverage-master.md` (2), `tools/kiss_bundle.py` (1), `conformance/cuda/generated/PROVENANCE.md` (1) | mentions | reword or remove |
| `docs/kernel-standard-discoverability.md`, `docs/kernel-body-eleven-choices.md`, `docs/kernel-body-scope-note.md` | the board 450 and 459 working documents | keep as history, marked superseded by board 475, or move to Unpopped (his call) |
| `CHANGELOG.md` (12) | history | stays |
| **Board 450 condition** | option A was ruled *"only as long as the necessary references are kept in place so readers know when recommendations are available"* | board 475 supersedes it by ruling deletion; I flag it so he confirms he means to overtake his own condition |
| **Board 459** | the six choices | superseded (pm-kernels) |

## 4. Audit: rules that live only inside a kernel (or inside one implementation)

Result: **outside `kernel.md`, no whole clause in the nine specs governs only the inside of a running kernel's computation.**
What the pass found is (a) **mechanism sentences laid over a result that is already pinned**, (b) **a launch-model
mandate in Contract**, (c) **generator-architecture rules in Emit**, and (d) **rules about one implementation's own
process**. Calls below are keep, reword (keep the outcome, drop the mechanism) or remove; he rules.

**A. Mechanism over a pinned result (Ops).** The caller sees the result; the method is free. Call: **reword to the outcome.**

| Clause | Sentence | Who relies | Call |
|---|---|---|---|
| KISS-OPS-6.5-0004 | `select` MUST NOT be rewritten to or from a mask-multiply form | nobody; 6.5-0002 already pins the raw bits | remove (redundant) |
| KISS-OPS-6.10-0006 | "MUST either (a) truncate every narrow operand ... or (b) require every operand ... to be a direct load" | callers rely on the truncation result; (b) also shows as a decline | keep the value pin, reword the menu |
| KISS-OPS-6.13-0004 | `reduce_var` MUST be computed from deviations about the mean, not `E[x²] − E[x]²` | callers rely on "result ≥ +0" and "std not NaN through cancellation" | keep the two outcomes, drop the algorithm |
| KISS-OPS-6.18-0006 (3rd sentence) | `cdiv` MUST NOT evaluate the zero-denominator or infinite cases through the real `div` atom | callers see exact-byte results, which are pinned | remove the sentence |
| KISS-OPS-6.18-0012 (2nd half) | `cpow` non-finite/zero inputs MUST be evaluated by applying the Annex-G values of `clog`, `cmul`, `cexp` in that composition | the enumerated results are pinned | reword to the result rows |
| KISS-OPS-6.18-0015 | a `c64` op MUST use `f32` component lanes, no promotion across atoms | exact-byte ops: bytes pin it (OUTSIDE); ULP-class ops: only the tolerance shows | keep for exact-byte, relax for ULP-class |
| KISS-OPS-6.21-0002 | a conversion through an intermediate dtype that can round twice is non-conforming | callers see the double-rounding bits | keep the single-rounding outcome, reword |
| KISS-OPS-6.11-0027 | welford merge with counts and per-op rounding in the state dtype | **other kernels merging partial states rely on the merge formula** | **keep** (cross-kernel monoid) |

No clause in Ops 6.1-6.11 or 6.12-6.21 fixes an accumulation order, an internal intermediate precision or an algorithm for a
float result (Ops 6.11-0024: "any accumulation order a candidate used is admitted"); the reading agents found those
absences with a mechanism-wording control (6.10-0006). Ops 6.0 (my read) pins classes and results, so it stays.

**B. Contract's launch model (the strongest candidates).** KISS-CONTRACT-6.6-0001 requires the launch model to carry
exactly `{invocation_domain, workgroup_sizing, count_to_grid, thread_mapping, addressing_rule}`; 6.6-0004 mandates the
**grid-stride** mapping (`t + k·thread_mapping`) and an `addressing_rule` scaling a thread index into an address;
6.6-0008 requires a v1 contract to be geometry-agnostic or grid-stride with thread-index-free coefficients.
**Who relies:** the caller needs `invocation_domain`, `workgroup_sizing` and `count_to_grid` to launch; it passes the
strides and offsets itself (6.5-0005/-0006). **`thread_mapping` and `addressing_rule` describe how the kernel indexes
internally**, and 6.6-0008 forbids a tiled or warp-lane schedule. Call: **keep the three launch fields, REMOVE the two
internal ones, drop the grid-stride-only restriction.** **RULED (CireSnave 2026-11-21, verbatim via pm): "Rather than making the two internal fields optional, remove them.  KISS should provide no way to enforce anything on the internals of a kernel."** Call B is REMOVE `thread_mapping`, `addressing_rule` and the grid-stride-only rule (6.6-0001/-0004/-0008); KEEP `invocation_domain`, `workgroup_sizing`, `count_to_grid`. Still BREAKING (contract_version bump). Readers at origin/main: FireLine only (`crates/contract/src/fields.rs:147-148` plus its tests; control: `invocation_domain` found in the same files); Baracuda mentions them in prose only. No clause text is written here. This is the one place a kernel builder is
bound today, and it changes Contract bytes (BREAKING for the 6.6 field set). Also KISS-CONTRACT-6.8-0012: the
"calibration record" an implementation must be able to produce is "not a contract field and adds no bytes"; only an
auditor could ask for it. Call: remove the sentence, keep the declared band.

**C. Emit's lowering partition (borderline: generator architecture, not a running kernel).** KISS-EMIT-6.2-0001 to -0005,
6.3-0001 to -0006, 6.4-0004/-0005 and the audit clauses 6.5-0001 to -0003, 8.2-0002 split lowering decisions between a
"neutral driver" and the emitter. **Who relies:** only an author who shares a neutral driver across emitters, and the freeze
audit built on the split. The caller of a kernel sees emitted output bytes, not who spelled what. Per pm-kernels I do not
pre-decide: my call is **it is not "inside a kernel" at all**, it is how a generator is organized, so the free-reign
ruling does not reach it unless he decides the generator counts. Evidence of dependency: removing it needs edits to 7.1-0001,
8.1-0001, 8.2-0001, 8.2-0002 and informative §1-§3, A.2-A.4. KISS-EMIT-6.1-0008 (no scheduler in the conformance path)
protects the input schema: **keep.** Mixed clauses 6.4-0001/-0002/-0003: keep the bit-pattern and declared-tier halves
(callers see the values), drop the "neutral driver MUST NOT spell it" halves.

**D. One implementation's own process (not a kernel; he may want a separate ruling).**
- Synth, consumer-internal: 6.2-0004a/b/c, 6.2-0006a (2nd half), 6.2-0008a, 6.4-0001f, 6.4-0004, 6.5-0004c, 6.6-0001d,
  6.6-0006a (a consumer MUST NOT panic or MUST surface a typed error); provider build internals 6.1-0004a, 6.6-0001b
  and the word "build" in 6.3-0002; the allocation-ordering half of 6.4-0001e. Synth is the provision protocol, not a
  kernel-semantics spec.
- Announce/Grammar/Consume: retirement-floor rules (ANNOUNCE-7.1-0004, GRAMMAR-8-0007); standard-library-only
  (ANNOUNCE-6.5-0002); Grammar's frozen in-memory schema and an off-wire channel (GRAMMAR-6.1-0001, 6.3-0001, 6.3-0004);
  "read by name, no hardcoded list" (GRAMMAR-6.2-0005, 6.4-0005, 6.4-0006); method bans where the outcome is already
  pinned (GRAMMAR-6.2-0003, CONSUME-6.1-0001, 6.4-0007); CONSUME-6.6-0005 (no Emit dependency) and 6.4-0010 (seed spelling,
  already "never a binding gate").
- Classify fragments, none a whole clause: 6.1-0005 ("no precision field" in the in-memory record), 6.3-0007, 6.3-0008,
  the "without allocating" tail of 6.4-0004, the ordering sentences of 6.7-0002 and 6.7-0015, the "owes the gate" half
  of 6.8-0011 and the "demonstrably failing" duty of 6.8-0014.
- Conform: 6.7-0003 (do not rely on fixed vectors alone) and 7.2-0005 (consumer durable record).
- **Keep, not a kernel rule at all:** parser-safety riders ("without allocating on an unchecked declared length", about 15
  clauses across Announce, Contract, Consume, Synth, Classify): the crash or out-of-memory is observable to a caller and
  these are security properties.
Call for D: **remove or reword the ones whose outcome is already pinned elsewhere; leave the rest**, and ask him whether
"free reign" extends to an implementation's own process or only to the inside of a kernel.

## 5. Case (c): a Contract read/write footprint (NOT ruled; cost only)

Today Contract states one footprint fact, `in_place` (KISS-CONTRACT-6.5-0009: the output aliases a named input or none).
It states no per-operand read set or write set, so a caller cannot tell from the Contract which buffers a kernel writes.
The race rule between two concurrent kernels (case (c) of board 474) is only checkable by a host if it has one.

Cost, my estimate, not measured: (1) a new per-operand Contract Interface field (read, write, read-write) with a default,
so a Contract-document byte change; (2) Contract golden vectors and `contract_golden` regenerate; (3) Consume must
**infer** the access mode when it lifts a hand-written kernel (a new lift obligation, with no way to tell write from
read-write for an opaque body); (4) Emit must emit it; (5) Conform needs a vector family, and per the freeze gate two
dissimilar implementers; (6) no consumer has asked for it. Left **open for CireSnave.** Without it, the boundary rule
for races stays conditional on what the caller knows.

## 6. Decisions for CireSnave

1. The placements of section 2, including the two **BREAKING** byte changes (Announce device-record field; removal of
   `kernel_rules` from the op-facts corpus) (the Contract field additions are checked: they break older readers, section 2).
2. That removing KISS-Kernel leaves scatter's atomic order and reach unstated in KISS (section 2, consequence).
3. That board 475 overtakes the board 450 condition about keeping references (section 3).
4. Section 4: group A (reword to outcome), group B (loosen Contract's launch model: BREAKING), group C (is a generator
   "inside a kernel"?), group D (does free reign cover an implementation's own process?).
5. Case (c), the footprint field: open.

## 7. Defects found on the way (not part of this ruling)

- **KISS-CLASSIFY-6.6-0004 said `version` "MUST equal `5`"** (also its field table and the §8 introduction) while KISS-CLASSIFY-6.4-0003 says `8` (sk8).
  The test `test_classify_structure_key_field_layout` already asserts `sk8` (only its comment said 4), so the suite never disagreed. **Fixed in the vocab-v8 PR**, not here, so this PR stays docs-only; no token byte moves.
- `spec/namespaces/vulkan.md` section 4 carries a stray line, `## 6. Sample tokens / vectors: not yet checked...`, left
  over from vocabulary 7. Not touched here.

— lane: kiss
