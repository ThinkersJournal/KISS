# Design note: how a reader of KISS-Ops finds the kernel-body standard (board 450, option A)

**Informative. No normative text. For the PM, then the build PRs.** Measured at `origin/main` `8c14914`, 2026-11-16, by
reading `spec/umbrella.md`, `spec/ops.md`, `spec/consume.md`, `spec/emit.md`, `conformance/corpus/op_facts.json`, the
`tools/` generators and `.github/workflows/traceability.yml` at that ref.

CireSnave ruled board 450, verbatim: *"450: A...but only as long as the necessary references are kept in place so readers
know when recommendations are available."* Option A is a new sub-standard (working name **KISS-Kernel**) for the kernel-body
rules (scope as in `docs/kernel-body-scope-note.md`), on the condition that four references exist and cannot rot. This note
names each reference exactly, says what it does to bytes, and states the check.

## How a reader reaches a neighbouring standard today

Three places, and only the second is binding.

1. **The umbrella** (`spec/umbrella.md`) is *informative throughout* ("introduces no independent normative clauses",
   line 6). Section 2.1 has one paragraph per sub-standard, section 2.2 is the edge table, and section 6.3 makes a claim
   prerequisite-closed over incoming STRUCTURAL edges (line 297).
2. **The depending sub-standard's section 4 (Normative References)** carries the edge and its label. `consume.md:463`
   onward reads "KISS-Ops (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency"; `emit.md:506` and
   `contract.md:417` do the same. The umbrella says the label lives *there* and the umbrella table mirrors it (line 69).
3. **The depended-on sub-standard's "Downstream edges" row** (`ops.md:49`) lists, with labels, who builds on KISS-Ops.
   That is the only place an agent reading KISS-Ops learns that Grammar, Contract, Synth, Consume and Emit exist.

So the PM's reading of how the umbrella works is right in outline, with one correction: umbrella registration is
informative; the *binding* edge is in the new standard's own section 4 and in the Ops row. Registering also touches more
than one table (below).

## Reference 1: registration (text only, no bytes)

- **`spec/umbrella.md`:** a new paragraph in 2.1; rows in the 2.2 table; the counts "nine" (lines 47, 260) and "the other
  eight" (lines 91, 133) become ten and nine.
- **`spec/ops.md:49`, Downstream edges:** add *KISS-Kernel (**STRUCTURAL** — the kernel-body rules fix the thread-level
  order, scope and race behaviour of the Ops atomic combines and take the Ops monoid set)*. **Recommended label: STRUCTURAL,**
  because KISS-Kernel cannot be implemented without reading Ops clauses (6.11-0006, -0010, -0024), which is the umbrella's
  own test for STRUCTURAL; OPAQUE would let a claimant skip Ops. The reverse edge (Kernel into Emit and Consume) is not
  added in v1, because v1 does not change what those lower or lift.
- **`spec/kernel.md` section 4** carries the matching upstream edges to Ops and Classify, labeled.
- **Tool lists that must learn the new document:** `tools/kiss_trace.py:310`, `tools/kiss_freeze_gate.py:49`,
  `tools/kiss_scoped_cites.py:64`, `tools/kiss_tables.py:56`, the sidecar set (`spec/sidecar/kernel.json`, and
  `tools/kiss_sidecar.py`), and the counts in `conformance/README.md` ("0 of 9 sub-standards", a bound figure).
  `tools/kiss_bundle.py` already derives the suite shape from the umbrella table and **fails hard when the table, the
  trace SPECS list and the `spec/` directory disagree**, so a half-registered standard is caught by an existing tool.

## Reference 2: a binding pointer at every governed op (text only)

KISS-Ops today has **no** barrier, fence or collective op (searched `ops.md` for `barrier`, `fence`, `collective` and `subgroup`, any case: zero lines; control: the same
search finds `atomic` in 36 lines). The governed surface is therefore the `scatter` atomic combines: clauses
`KISS-OPS-6.11-0006`, `-0010`, `-0024` and the `scatter_add` row of the 6.13 table (`ops.md:1497`). Any op added later
(for example the collective monoids of KISS#768) takes the same pointer when it lands.

The pointer is one normative sentence at each clause, in a fixed form so a tool can find it:
`Kernel-governed: KISS-KERNEL-<id>[, KISS-KERNEL-<id>]` (the thread-level order, scope and race rule of this op's
concurrent behaviour are those clauses). Because it carries ids, it is **binding**: an implementation of the op also meets
them. To keep one source of truth the ids are not scattered: `ops.md` gains one table, "Kernel-governed ops" (op | ids), and
each op clause cites the table row. The table is what the next reference is generated from.

## Reference 3: a per-op field in `op_facts.json` (the only byte change)

**Name: `kernel_rules`.** Value: the array of KISS-Kernel clause ids that govern the op, `[]` for an op no rule governs
(so an absent pointer and "ungoverned" are told apart by the generator's cross-check, not by a missing key). It is generated
by `tools/kiss_op_facts.py` from the table in Reference 2; the generator refuses to emit if an id is not in
`spec/sidecar/kernel.json`. `fact_clauses` gains `"kernel_rules": ["KISS-OPS-<table clause>"]`.

What it does to bytes, stated completely:

- `conformance/corpus/op_facts.json` gains one line per op (`"kernel_rules": []` is three lines under the generator's
  `indent=2` layout, so +3 per op over 128 ops, about +384 lines) and one `fact_clauses` entry. The file is regenerated by CI
  (`traceability.yml:301`, `git diff --exit-code`); the committed bytes change once.
- **No digest or manifest is affected.** `op_manifest.json` is a separate file from a separate tool
  (`kiss_ops.py --emit-manifest`, `traceability.yml:295`) and holds only the op set and atomic list, so it does not move.
  No hash or checksum of `op_facts.json` exists in the repo (searched `tools/` and `conformance/corpus/PROVENANCE.md` for
  `digest`: none beyond the freshness comparison above).
- **No wire or golden byte moves:** `op_facts.json` is not part of any Contract document, so the Appendix C golden
  (`KISC kiss-contract 4 len=1304 crc32=2854c69d`), every `structure_key` and the OpAttrs blobs are unchanged.
- **Readers:** the Rust reader (`conformance/src/op_dag.rs`) looks fields up by key and ignores others, and the Python tests
  index named keys, so an added key breaks none (checked by regenerating and running both). The `schema` string stays
  `kiss-op-facts-v1` because the change is additive; a reader that wants the new field reads it by name.
- The crate embeds the file with `include_str!`, so the conformance crate's own build output changes with it (no version
  rule beyond the usual stamp).

## Reference 4: a check so the condition cannot rot (CireSnave's own condition)

A new generator-side check, `tools/kiss_kernel_refs.py`, run beside `kiss_ops.py` in `traceability.yml`, plus a harness row
under KISS-Conform so it is a counted clause, fails when any of these holds:

1. an op in the Kernel-governed table has an empty `kernel_rules` in `op_facts.json`, or an id there is not a live clause of
   `spec/sidecar/kernel.json`;
2. a KISS-OPS clause or table row that says `Kernel-governed:` names an id the table does not list, or an op the table lists
   has no such sentence at its clause;
3. `umbrella.md` 2.1 or 2.2 has no KISS-Kernel entry, the Ops Downstream edges row has none, or `kernel.md` section 4 has no
   Ops edge (the existing `kiss_bundle.py` reconciliation already covers the third disagreement among table, SPECS and
   `spec/`);
4. a clause of KISS-Kernel names an op that is not a KISS-Ops op.

Each rule gets a mutation test (delete the pointer, delete the umbrella row, add a bogus id) that must turn the check red,
so a check that cannot fail is not counted.

## Order of the build PRs

1. Skeleton `spec/kernel.md` with sections 5 and 6 of the scope note (barriers, atomics), the sidecar, the tool lists and the
   umbrella and Ops rows (References 1 and 2). Text only.
2. `kernel_rules` in `op_facts.json` and `kiss_kernel_refs.py` with its mutation tests (References 3 and 4), landing with
   PR 1 or immediately after so the condition holds from the first commit that mentions the standard.
3. KISS#769 content, then KISS#768.

**Decisions this note needs:** none that change bytes beyond Reference 3, which CireSnave's ruling already covers ("a
per-op pointer field"). Two small choices are mine to make unless the PM objects: the edge label STRUCTURAL, and the field
name `kernel_rules`.

— lane: kiss
