# Changelog

`kiss-conformance` (the reference crate and the spec it binds). Pre-1.0, a breaking change moves the
second number.

## 0.4.28 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Docs only, no spec, tooling or wire change: retracts the premise of the merged `docs/design-298-operand-structure.md` and ROADMAP R1.12 (arity feeds `op_family`; #263 blocked on it) — `op_family` was dropped in a33282c and KISS-CLASSIFY-6.5-0015 derives the cell class without reading an op name.

## 0.4.27 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Docs and CI only, no spec, tooling or wire change: `docs/design-298-operand-structure.md` proposes the per-op operand-structure field (count form, closed role vocabulary, 122 rows; ROADMAP R1.12, KISS#298); the advisory CodeRipper `portfolio` job is re-pinned to `30e012b7`.

## 0.4.26 — 2026-10-23 (number proposed; the PM allocates at gate time)

- CI only, no spec, tooling or wire change: `.github/workflows/conformance.yml` gains an advisory, non-blocking `portfolio` job that calls CodeRipper's reusable portfolio workflow (pinned by SHA). It is not a required check.

## 0.4.25 — 2026-10-23 (number proposed; the PM allocates at gate time)

- Spec text, no wire change (closes KISS#3): `spec/emit.md` §7.2-0001, the Declared-subset definition and Appendix A.5 item 1 no longer say the declared subset is "not yet pinned"; they point at KISS-CONSUME-7.2-0001, which pins the advertisement (canonically-ordered, byte-exact op-name list in the capability channel) and the overlap-not-equality rule. The clause's obligations and its test are unchanged.

## 0.4.24 — 2026-10-23 (number proposed; the PM allocates at gate time)

- Docs only, no spec, tooling or wire change: `ROADMAP.md` puts every open issue on a row (R1.12–R1.29, R2.8, R3.5–R3.12). 23 issues verified resolved or obsolete at `a76aef0` were closed with evidence.

## 0.4.23 — 2026-10-22 (number proposed; the PM allocates at gate time)

- Ledger only, no spec, tooling or wire change: `conformance/UNBACKED.tsv` records a reason for KISS-ANNOUNCE-6.5-0001, -0002, 7.2-0008, 8-0001, -0002 and -0003 (no in-repo subject observes the property; they stay `untested`, floor unmoved at harness 436, lint 44, untested 410).

## 0.4.22 — 2026-10-22 (number proposed; the PM allocates at gate time)

- Ledger only, no spec, tooling or wire change: `conformance/UNBACKED.tsv` records a reason for KISS-OPS-6.11-0001, -0015, 6.14-0004, 6.16-0002, -0007 and -0011 (no executable subject in `conformance/`; they stay `untested`, floor unmoved at harness 436, lint 44, untested 410).

## 0.4.21 — 2026-10-22 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints KISS-OPS-6.13-0001 (every §6.13 row and every §6.18 complex table row carries a non-empty reference decomposition; the clause keeps saying it defines the op's pinned semantics) as a drift guard. Stays `untested`, PARTIAL, floor unmoved at harness 436, lint 44, untested 410.

## 0.4.20 — 2026-10-22 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints KISS-OPS-6.13-0008 (each of `softmax`, `log_softmax`, `rms_norm`, `layer_norm` has a mandatory `u8` `norm_axis` row in §6.19.3; the clause keeps its range and typed decline) and 6.15-0004 (`gelu` decomposes through `erf`, `gelu_tanh` through `tanh`, as distinct rows that never reach each other's op) as drift guards. Both stay `untested`, PARTIAL, floor unmoved at harness 436, lint 44, untested 410.

## 0.4.19 — 2026-10-22 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints the restated text of KISS-OPS-6.12-0001 (the seven scalar-source leaves), 6.12-0002 (`const` carried as exact bits), 6.11-0017 (scatter `updates` broadcast, rank-0 only the degenerate case) and 6.11-0018 (the four empty-axis results) as drift guards. All four stay `untested`, PARTIAL, floor unmoved at harness 436, lint 44, untested 410; no evaluator subject exists in `conformance/`.

## 0.4.18 — 2026-10-21 (number proposed; the PM allocates at gate time)

- Clarification, no wire or token change: KISS-OPS-6.17-0002 states that subnormal preservation is part of bit-stable, so a flush-to-zero kernel is non-conformant and declines `Exact`; no `denorm` field in the `vulkan:` token, no new MathFidelity value, `structure_key` stays `sk6` (PM ruling 2026-10-21, on Baracuda's recommendation and the Vulkane POC). Evidence recorded in `conformance/UNBACKED.tsv`.

## 0.4.17 — 2026-10-21 (number proposed; the PM allocates at gate time)

- Ledger and docs only, no spec, tooling or wire change: `conformance/UNBACKED.tsv` records a reason for KISS-OPS-6.19-0001, -0002, -0013 and KISS-OPS-6.12-0003 (no executable subject in `conformance/`; they stay `untested`, floor unmoved at harness 436, lint 44, untested 410). `HANDOFF.md` rewritten to the post-#604 state.

## 0.4.16 — 2026-10-21 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints the Ops index-ref wire clauses as drift guards: KISS-OPS-6.19-0039 (`index_ref := ref_tag:u8, ref_value:varint`, tag 0x00 slot / 0x01 node, tags >= 0x02 reserved and rejected with a typed decline, node-target bounds, canonical slot) and 6.19-0040 (the `index_outputs` root list, unconditional, after `outputs`, typed decline). Both stay `untested` (PARTIAL, floor unmoved: no decoder subject in `conformance/`).

## 0.4.15 — 2026-10-18 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints the Ops accumulator clauses as drift guards: KISS-OPS-6.17-0007 and -0009 carry one per-cell reference definition (ascending-index, each input rounded to S, each accumulate atom rounded to A, result rounded to S, widest-A cell is the wide reference), and 6.17-0008 still requires a float accumulator at least as wide as the compute dtype, names the diagonal cell and cites -0009. All three stay `untested` (PARTIAL, floor unmoved: no reduction evaluator in `conformance/`).

## 0.4.14 — 2026-10-17 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints two structural-atom clauses as drift guards: KISS-OPS-6.11-0008 (the `reduce` §6.19.3 row fixes `keepdim` at `1` citing the clause, the `prefix_scan` row has no `keepdim` field, the clause still says `prefix_scan` retains all axes) and 6.11-0016 (the clause and the §6.20-0008 enumeration both pin the `scatter` output to `SameAs(dest)` and cite each other). Both rows stay `untested` (PARTIAL, floor unmoved: no evaluator or `scatter` subject in `conformance/`).

## 0.4.13 — 2026-10-17 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints the Ops index-operand and axis clauses as drift guards: the legal index dtype set `{u32, i32, i64}` is one set across KISS-OPS-6.11-0009, 6.11-0019, the 6.19-0017 enum and its byte table; each of `gather`/`scatter`/`index_select`/`embedding`/`scatter_add` carries mandatory `axis`, `index_operand` and `index_dtype` fields in its 6.19.3 row; `sort_network` carries a `u8` trailing `axis` and no index or dtype field; 6.19-0037 pins a `MAX_RANK` a `u8` axis can hold. KISS-OPS-6.11-0009/-0012/-0013/-0014/-0019 stay `untested` with PARTIAL notes (floor unmoved: the typed declines and the producer-side `i64` have no subject in `conformance/`).

## 0.4.12 — 2026-10-17 (number proposed; the PM allocates at gate time)

- Docs only, no spec, tooling or wire change: `HANDOFF.md` rewritten to the post-#599 state.

## 0.4.11 — 2026-10-17 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints §6.17. KISS-OPS-6.17-0004 (no compute-precision spelling `f32s`/`st`/`rm`/`tf32` in the §6.16 storage dtype table, `f32` stays pure storage) moves from `untested` to `lint:kiss_ops`. KISS-OPS-6.17-0006 stays `untested` (PARTIAL, floor unmoved): the lint checks the TF32 pin is self-consistent (retained bits equal the §6.17-0003 floor, `u` = 2^-(bits+1), round-to-nearest-even named); a bit-for-bit rounding reference has no subject in `conformance/`. Floor: lint 44, untested 410.

## 0.4.10 — 2026-10-17 (number allocated by the PM in advance)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints the §6.19 carrier clauses as drift guards: the §6.19-0003 clause's carrier set equals the §6.19.3 schema table, and each §6.19-0036 non-carrier has no schema row and calls a `reduce`/`prefix_scan`/`sort_network` node in its §6.13 decomposition. Both rows stay `untested` (PARTIAL, floor unmoved: no per-op encoder and no consumer subject in `conformance/`). The README's unbound ledger prose (lint, blocked, definitional counts) is brought to the ledger's real counts.

## 0.4.9 — 2026-10-17 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: `tools/kiss_ops.py` also lints the §6.18 complex family. KISS-OPS-6.18-0002 (every decomposition stays inside the closure the clause names, acyclic) and 6.18-0014 (each op's determinism class is derived from its decomposition and compared with the clause lists and the table column) move from `untested` to `lint:kiss_ops`. Floor: lint 43, untested 411.

## 0.4.8 — 2026-10-16 (number proposed; the PM allocates at gate time)

- Tooling and ledger only, no spec or wire change: the §6.13 decomposition-graph lint also covers KISS-OPS-6.14-0006 (ops embedded in a structural atom's scalar body count as references); KISS-OPS-6.13-0002 and 6.14-0001 are recorded `definitional` (true by construction of the checked graph). Floor: lint 41, untested 413.

## 0.4.7 — 2026-10-16 (number allocated by the PM)

- Tooling only, no spec or wire change: `tools/kiss_ops.py` lints the §6.13 decomposition graph (no floor row, every call resolves, acyclic). KISS-OPS-6.3-0003, 6.3-0004, 6.14-0002, 6.14-0003 and 6.14-0005 move from `untested` to `lint:kiss_ops`. Floor: lint 40, untested 416.

## 0.4.6 — 2026-10-16 (number proposed; the PM allocates at gate time)

- HANDOFF rewritten for the post-#593 state (docs only, no behaviour change).

## 0.4.5 — 2026-10-16 (number proposed; the PM allocates at gate time)

- **Clarification, no wire change, no regeneration (ruled A by the PM 2026-10-16, from Baracuda's FINDING on a full-precision f32 FMA cell keyed `.../f32/f32/f32/rm`):** KISS-OPS-6.17-0003 states that `reduced-mantissa-permitted` (`rm`) is a permission, never an obligation: a kernel that does not round operands is admitted under `rm`, and a contracting (FMA) kernel cannot claim `bit-stable` (§6.17-0002). §6.17-0005's verification wording says a probe finding no narrowed input passes. Ledger: 6.17-0003 stays `untested`, with the reason recorded (no kernel subject in `conformance/`, so a vector would be vacuous). A third value for full-precision-with-FMA is a ROADMAP row, not this change.

## 0.4.4 — 2026-10-16 (number proposed; the PM allocates at gate time)

- HANDOFF rewritten for the post-#591 state.
- **Correction to 0.4.3 (§6.7-0006 warning):** the undeclared-accumulator default is **operand 0's dtype**, not "the weight's" (they coincide only when the weight is operand 0). The vector `ctr_weight_first_undeclared_accumulator_defaults_to_weight_dtype` is renamed `..._defaults_to_operand0_dtype`; its bytes are unchanged. New vector `ctr_weight_second_undeclared_accumulator_defaults_to_operand0_dtype` (weight at slot 1, accumulator undeclared) pins the case where the weight is NOT operand 0.

## 0.4.3 — 2026-10-16 (number proposed; the PM allocates at gate time)

- **Reference `ctr` derivation stage (ROADMAP R3.3).** `derive_structure_key` now derives dense-contraction cells: `DerivationCell.contraction: Option<ContractionRoles>` replaces the boolean `contraction_declared` and the `ContractionOutOfScope` decline. The roles are caller hints (§6.6-0012): M, N and K as **operand axes** `{operand, axis}`, batch axes as iteration-frame axes (§6.6-0023), the weight and scale as call-order operand slots.
  - **Spec clarification, no wire change (§6.6-0016):** M, N and K name an axis of an operand, not a frame axis. Under right-alignment `lhs [M,K]` and `rhs [K,N]` share frame axes, so a frame axis cannot name them (the §2.8(d) example `[8,4096] . [4096,4096]` has frame axis 0 = 4096, not M = 8). Batch axes stay frame axes. §6.6-0019's coverage note is rewritten: `<wdt>` is now derived from a cell, and `ctr_weight_role_not_operand_one` is a live detector.
  - **Artifact:** `structure_key_derivation_vectors.json` gains 15 `ctr` vectors (rank-2 to rank-4, batched and not, quantized weight/scale, accumulator + `rm`, and the weight-first undeclared-accumulator trap, warned about in §6.7-0006: `<acc>` defaults to operand 0's dtype, which is the weight's only when the weight is operand 0) and 12 role-hint declines (`ContractionAxisInvalid`, `RoleSlotOutOfRange`, `ContractionNoOutput`, the five batch declines, a reserved weight dtype). `decline_contraction_out_of_scope` is removed. Each cell gains a `contraction` object; `contraction_declared` is kept (true iff it is non-null). Expected bytes are reference output, not hand-derived goldens: a second deriver is what makes them count.
  - **Consumers:** a deriver reading the artifact now has `ctr` cells to reproduce; nothing about an existing token changes (`sk6`, no regeneration).
  - Floor: KISS-CLASSIFY-6.6-0012 and -0016 move from blocked to backed.

## 0.4.2 — 2026-10-16 (number proposed; the PM allocates at gate time)

- **Batch role defined (ROADMAP R1.8, KISS#543; ruled by the PM 2026-10-16 as option A, the editor's recommendation).** New KISS-CLASSIFY-6.6-0023: the batch axes of a `ctr` cell are a caller-declared role hint (never inferred from rank, position or extent, as for the weight, §6.6-0019); the cell is batched iff the declared set is non-empty (a declared axis of extent 1 is batched); the `b<class>` class is that of the product of the batch extents (saturating, §6.5-0008); a batch axis that is out of range, duplicated, also an M/N/K axis, absent from the output or of extent below 1 is a typed decline. §6.6-0012, §6.6-0016, §6.7-0006 and the cell-class table amended to point at it. Code: `derive_batch_class`, `size_class_of`, `BatchRoleDecline`; test ``test_classify_batch_role_derivation`.
  - **No wire change, no regeneration:** a single leading batch axis of a rank-3 cell derives the token the codec always carried; the clause pins what was unpinned (rank above 3, extent 1, several batch axes). `structure_key` stays `sk6`; every vector artifact is byte-identical except the text of the derivation corpus's `scope_note` and one vector `note` (the `ctr` stage is still not built).
  - **What each speaker does:** Unpopped declares the batch axes beside M/N/K (its blocked roles); Fuel passes the declared axes instead of a scalar extent (same number at rank 3); a third deriver reads the clause.
  - Floor harness 433 → 434 (one new clause, backed by a discriminating test); untested and lint unmoved.

## 0.4.1 — 2026-10-16 (number allocated by the PM)

- HANDOFF.md rewritten for the post-#588 state (docs only, no behaviour change).

## 0.4.0 — 2026-10-16 (number proposed; the PM allocates at gate time)

- **BREAKING (wire, pre-1.0 so the second number moves): the `structure_key` bump `sk5` → `sk6`, with the Announce `list_version` `1` → `2`** (board 193 R1.2 A + R1.1 B, one regeneration event; PM [TASK] "GO on the structure_key bump now"). Nothing is frozen. Per-vector old/new: `docs/sk6-vector-changes.md` (105 changed rows: 101 are the `sk5|` → `sk6|` prefix alone, the other 4 are the announce frames; 6 added; 0 removed).
  - **R1.2 A — scale dtype keyed for quantized `ctr` cells only** (§6.6-0022 new; §6.6-0015, §6.7-0006, §6.4-0003, §6.7-0014 amended). The contraction field gains an optional trailing `q<dtype>`: `ctll/d16/i4/f32/bf16/st/qf8e8m0` vs `…/qf16` are now two cells (at `sk5` both derived the unspelled form, so a lookup could return a kernel numerically wrong for the other scale). The scale operand is identified by a caller role hint (as the weight is, §6.6-0019), never by position or dtype; no hint → no component, so such a `ctr` token is its `sk5` spelling with the prefix changed. Non-`ctr` cells are untouched (§6.6-0015 stands). Code: `Contraction.sdt`, `derive_scale_dtype`.
  - **R1.1 B — `op_identity` in every Announce lookup** (§6.3-0013 new; §6.3-0001/-0005/-0006/-0012, §6.4-0001/-0003/-0011/-0012 amended; Synth §6.1-0002/-0004/-0005, §6.6-0004/-0004a/-0004b, §6.7-0003/-0004/-0004b new, mirrored). The availability record becomes `(structure_key, op_identity, revision_hash)` (`op_identity` = u32-LE length in `[1, 4096]` + bytes, right after the key); `list_version` is `2` and `1` is retired (declined, never read as `2`); `CYRQ`, `CRSP`, `CDEC` carry the same block in their identity; the empty identity echoed on a `MALFORMED_REQUEST` decline is now key length 0, op length 0, `revision_present` 0. A hit needs all three to match. Code: `Identity.op_identity`, `AvailabilityRecord.op_identity`, `LIST_VERSION`, `MAX_OP_IDENTITY_LEN`, `AnnounceDecline::OpIdentityLenOutOfRange`.
  - **Stated limitation (PM ruling 2026-10-16):** `CYRQ`/`CRSP`/`CDEC` have no version field, so no frame-version bump or invented field is made now (unfrozen; all speakers regenerate together); recorded in §6.3-0013 and ROADMAP R1.10 (owner: KISS lane as editor; detector: the freeze checklist).
  - **Not changed:** `envelope_version` stays `1` (the 56-byte envelope is untouched); `contract_version` stays `3` (no Contract document field or clause moved; only the embedded `accept_predicate` token prefix is `sk6`); cell-class derivation, the dtype closed set, and every non-`ctr` key except its prefix. The `sk4`/`sk5` decoder-arm policy (§6.7-0014): `sk4` is retired, a bounded `sk5` arm is a MAY through the `sk6` series.
  - **What each speaker must regenerate:** every `structure_key` deriver/keyer (Unpopped's `unpopped_vocab::structure_key`, Fuel's `structure_key_derive.rs`, Baracuda via re-export): emit `sk6`, add the `q<dtype>` component for a quantized `ctr` cell with a designated scale operand, and byte-match `structure_key_vectors.json`. Every Announce/Synth speaker (Baracuda provider, Fuel consumer, Synth targets): write/read `list_version` 2 records and the `op_identity` block in `CYRQ`/`CRSP`/`CDEC`/`PRSP` identities, and byte-match `announce_vectors.json`. Anything persisting an `sk5` token or a `list_version` 1 list must re-derive it (the scale-less `sk5` form does not say which `sk6` cell it was). `kiss-ref` 0.4.0 is bound to `sk4`/`904a4b4`, not to this.
  - Floor harness 430 → 433 (three new clauses, each backed by a discriminating test: `test_classify_quantized_scale_dtype_keyed`, `test_announce_op_identity_length_bounds`, `test_synth_hit_requires_op_identity`); untested and lint unmoved. Regenerated: `structure_key_vectors.json`, `structure_key_derivation_vectors.json`, `announce_vectors.json`, `contract_vectors.json` (prefix only), `dtype_manifest.json` (`token_prefix`); sidecars add the three clauses and classify's `wire_abi_schema_version` is `6`.

## 0.3.6 — 2026-10-16 (number proposed; the PM allocates at gate time)

- HANDOFF.md rewritten for the post-board-193 state (docs only, no behaviour change).

## 0.3.5 — 2026-10-15 (number proposed; the PM allocates at gate time)

- **Sidecars follow the #462 ruling (board 193):** `determinism_class` is dropped from every clause (OPTIONAL, absent = n/a; 768 `null`s removed) and `kiss_sidecar.py` no longer writes it and rejects a present non-enumerant. `wire_abi_schema_version` is set where the sub-standard states its own string: announce `1`, classify `5`, contract `3`. Recorded "none stated (editor: KISS lane, pending ratification)" (the KISS lane is editor of record for all nine until ratified; §8 read in each): conform, consume, emit, grammar (the `1` in its vectors is illustrative), ops, synth. No clause text, clause ID or freeze state changes.

## 0.3.4 — 2026-10-15 (number proposed; the PM allocates at gate time; stacked after #584's 0.3.3)

- **Board 193 #462 ruled (CireSnave verbatim: "Editor and auditor and me. Yes, make desterminism_class optional. wire_abi_schema_version = each sub-standard's own schema string.")**: a freeze needs three signatures (editor-of-record, AUDIT role, steward's signatory); umbrella §5.3/§7.1/§7.3 and the four restatements (Conform §3, §8-0006, §7 role list, Consume §10, Emit 8.2-0004) reconciled. Appendix E: `determinism_class` OPTIONAL (absent = n/a); `wire_abi_schema_version` = the sub-standard's own §8 schema string, UNSET plus one ask where none is stated. Text only; sidecars and `kiss_sidecar.py` follow in a later PR. No clause ID added or retired.

## 0.3.3 — 2026-10-15 (number proposed; the PM allocates at gate time)

- ROADMAP.md and HANDOFF.md record the board 193 rulings R1.1 B, R1.2 A, R1.5 B (docs only, no behaviour change; wire text waits for the next `structure_key` bump).

## 0.3.2 — 2026-10-14 (number proposed; the PM allocates at gate time)

- HANDOFF.md rewritten for the post-board-203/201 state (docs only, no behaviour change).

## 0.3.1 — 2026-10-14 (allocated by the PM)

- **IR-DAG generator and validity self-check (the IN half of board 203 item 3)** (floor harness 429 -> 430, untested 422 -> 421): new `fuzz` module and `tests/fuzz_gen.rs`. A seeded generator of valid scalar-expression DAGs over the ops whose arity the KISS-Ops §6.13 table fixes (ops that resolve through an unreadable decomposition are excluded, and the self-check caught that during development), a typed validity check, and a malformed/near-boundary generator (forward edge, self loop, unknown op, arity +/-1, input index at the bound, root past the end) plus the Grammar codec's `BindSetMismatch`/`BindIndexOutOfRange` declines back KISS-CONFORM-6.6-0005. 6.6-0001 stays `untested`, recorded PARTIAL: roles, ranks, dtypes and OpAttrs sub-vocabularies are not generated. The cross-backend rows stay `blocked`. No spec change.

## 0.3.0 — 2026-10-14 (allocated by the PM)

- **BREAKING (wire, pre-1.0 so the second number moves): the `reduce`/`prefix_scan` `math_fidelity` ordinals are pinned** (board 201, CireSnave verbatim: "Accept Fuel's table."). §6.19.2 gains `math_fidelity`: `0` reserved, `1`=bit-stable, `2`=reduced-mantissa-permitted (the §6.17-0001 member order); `accumulator` stays `0x00` (compute dtype) or the 1-based §6.1-table dtype ordinal. §6.19-0025/-0026 and the §6.19.3 rows spell the field `math_fidelity` (it was `math_precision`; the name is not on the wire, and the PM read the rename as following #478). Appendix E.2's `reduce`/`prefix_scan` blobs are now the 6-byte form (`01 FF FF 01 00 01`, `01 FE FF 01 00 01`, `03 FF FF 01 00 01`) plus a non-default worked blob (`01 FF FF 01 03 02`).
- Code: `opattrs::encode_reduce`/`encode_prefix_scan` take `accumulator` and `MathFidelityOrd` (a breaking Rust API change for any in-tree caller); the generic value encoder mints `reduce` and `prefix_scan` (`not_emitted` is empty; population 133 -> 167, 39 new vectors incl. non-default accumulator and fidelity); `opattrs_vectors.json` field `math_precision` -> `math_fidelity`; a prefix_scan folds exactly one axis (rank-1 scan is `01 00`, never `FF FF`). Floor unmoved: no clause changed status.

## 0.2.44 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **Structural op-DAG equality comparator shipped** (board 203 item 2; floor harness 427 -> 429, untested 424 -> 422): new `structural_dag` module and `tests/structural_dag.rs` back KISS-CONFORM-6.9-0001 and -0002. It resolves non-primitive nodes through the KISS-Ops §6.13 decomposition rows read from `spec/ops.md`, normalizes node/edge order and commutative operand order, and compares node sets, edge sets and OpAttrs bytes; an unreadable (let-bound) decomposition is a typed decline, never a silent primitive. 6.9-0003 stays `untested`, recorded PARTIAL: no emitter or lifter exists, so no round-trip runs. Nothing is re-bracketed (KISS-Ops declares no associativity property). No spec change.

## 0.2.43 — 2026-10-14 (allocated by the PM)

- **Determinism import-site lint** (board 203 item 6; floor lint 34 -> 35, untested 425 -> 424): `tools/kiss_tables.py` gains `check_determinism_import_sites`, so KISS-CONFORM-6.11-0003 is lint-enforced. Synth, Consume, Emit, Contract and Conform must each spell all three KISS-Ops determinism members verbatim and may not re-spell one in a backticked or bold span (`ULP-tolerance`, `Exact-byte`, a bare `order-invariant`). Mutation-checked: a re-spelled member in a copy of `synth.md` fails the tool. Prose and test-name identifiers are out of scope by construction. No spec change.

## 0.2.42 — 2026-10-14 (allocated by the PM)

- **Board 203 out-halves recorded** (ledger only; floor harness 427 unmoved, untested 439 -> 425): CireSnave ruled "I agree with the recommendations", so 14 rows move `untested` -> `blocked`, each note quoting the ruling: canonical-suite-artifact governance (6.3-0003/-0004, 8-0003/-0004/-0005/-0010; waits on #462), oracle-authoring provenance (6.5-0002/-0003/-0005; a record, not a test), and the cross-backend/emitter/lift/promotion fuzzer rows (6.6-0002/-0003/-0004/-0006/-0007; need a suite-registered emitter and a lifter outside this crate). Recategorising is a ledger statement, not coverage. No spec change.

## 0.2.41 — 2026-10-14 (allocated by the PM)

- **Consumer-verify check and opt-in trust-but-verify profile shipped** (board 203 item 1; floor harness 420 -> 427, untested 446 -> 439): new `consumer_verify` module and `tests/consumer_verify.rs` back KISS-CONFORM-6.12-0001/-0002/-0003, 6.13-0011, 7.2-0001/-0003/-0004. The check reuses `comparator_for`/`compare_f32` and the `structure_key` accept-predicate; the `trust-but-verify` cargo feature (off by default) is the profile switch and never alters the default clause verdict. Scalar-unary f32 only; the declared ULP is caller-supplied (no Guarantees-tier ULP reader yet). No spec change.

## 0.2.40 — 2026-10-14 (allocated by the PM)

- Portfolio hygiene (CodeRipper triage 2026-10-14), no behaviour change: `.github/CODEOWNERS` (`* @ciresnave`, WSP-001); top-level `permissions: contents: read` in `conformance.yml` and `traceability.yml` (SUP-008, 10 excessive-permissions findings; every `uses:` was already pinned to a full commit SHA); `docs/glossary.md`, an index of the vocabulary that points at each owning §3 (DOC-009; also serves kiss-ref). `HANDOFF.md` rewritten to the state at 0.2.39. No spec change; floor unmoved.

## 0.2.39 — 2026-10-14 (allocated by the PM)

- **Conform 6.13-0012 backed** (floor harness 419 -> 420, untested 447 -> 446): `test_conform_contract_schema_byte_compare` compares the codec's Appendix C golden document with the committed `corpus/contract_vectors.json` bytes under the exact-byte comparator and checks the seven `[section:N:...]` headings are ids 1..7 in order (flipping a committed byte reds it).
- The other 63 bare `untested` KISS-CONFORM rows each record what was searched for and a control. Findings: no consumer-verify check, structure-directed fuzzer, structural op-DAG equality, canonical suite artifact or profile switch exists in the crate, so none of those has a subject; 6.13-0008 and 6.13-0009 are recorded PARTIAL (floor unmoved). No category change, no spec change.

## 0.2.38 — 2026-10-14 (allocated by the PM)

- Ledger notes only: the 30 bare `untested` KISS-GRAMMAR rows in `conformance/UNBACKED.tsv` each record what was searched for and a control (no synthesis channel, recognizer, KISS-Ops lookup or `op_identity` mapping in the reference crate). No category change, no test added; floor unmoved (harness 419, untested 447). No spec change.

## 0.2.37 — 2026-10-14 (allocated by the PM)

- **Conform 6.13-0003 backed** (floor harness 418 -> 419, untested 448 -> 447): `test_conform_classify_exact_byte_and_registry_bundle` re-emits every bundled golden token under the exact-byte comparator and checks the bundled registry snapshot is versioned and has a row for every namespace the vectors name (renaming the `vulkan` row reds it).
- Grammar 6.6-0003/-0004/-0005 stay `untested`, now recording what was searched for: the reference region model has no cell-class field and `encode` validates neither op names nor role tokens. No reference behaviour invented.
- ROADMAP: R1.4 CLOSED (Unpopped 0.16.0 derives `[8,0]` as `da`/`v1`); R3.2 Classify re-measured 88 backed / 32 ledgered / 0 untraced. No spec change.

## 0.2.36 — 2026-10-14 (number proposed; the PM allocates at gate time)

- `HANDOFF.md` rewritten to the state at 0.2.35 (R3.2 Ops vectors, R2.4 slice 1, R1.4, the Classify tail and its dispositions, the bash-suite wrapper, what waits on rulings). Docs only; floor unmoved (418/34/448).

## 0.2.35 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **Classify untested tail: 5 clauses backed, 11 dispositioned** (ROADMAP R3.2; PM: each disposition names what was
  looked for and a positive control, and `untestable`/`definitional`/`blocked` are NOT counted backed).
  **Backed (floor harness 413 -> 418):** 6.9-0001 (no upstream dependency) and 6.9-0003 (standard library only), as a
  source-level lint over the Classify-tier modules with a control that proves the scan sees imports and driver loaders
  (`tests/classify_dependency_and_carriage.rs`); 6.9-0002 (opaque length-delimited carry: every derivation-vector token
  and arbitrary bytes round-trip an Announce availability list unchanged and the token still parses);
  6.1-0007 (`bool` is one byte, width equals `u8`; every logical cell of `ops-compare-bool.json` has the expected byte
  "non-zero is true" dictates, recomputed from the vector content; `tests/classify_bool_storage.rs`); 6.0-0002 (the §6.0
  statement of the special-value patterns, running the 6.1-0011/6.2-0002 tests). **Dispositioned (each with its basis in
  `UNBACKED.tsv`):** `blocked` on KISS#543 (6.6-0012, 6.6-0016); `definitional` (6.3-0002 and 6.3-0007, unrepresentable in
  the reference crate with a control; 6.7-0011, a statement of absence; 7.1-0001, 7.2-0001, 7.2-0002, 8-0001, 8-0002,
  8-0003, change-control and profile-scoping rules). **Left `untested`, notes now say why:** 6.0-0001, 6.1-0012, 6.3-0004,
  6.3-0009, 6.3-0010, 6.8-0017 (each names the symbol searched for and not found, and a control). The `untested`
  figure moves 464 -> 448: 5 by backing, 11 by recategorisation, which is a ledger statement and not coverage; the
  ENFORCED figure rises by 5 only. No spec or corpus change.

## 0.2.34 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **Classify storage-dtype clauses backed** (ROADMAP R3.2, KISS-CLASSIFY-6.1-0005 dtypes are pure storage, 6.1-0006
  `u32` is an ordinary storage dtype): `tests/classify_storage_dtypes.rs` holds the reference deriver and
  `dtype_manifest.json` to the clauses. 6.1-0005: a dtype row is exactly `{token, kind, storage_bits, reserved}` (no
  precision field), no strict-precision token exists (`f32s`, `f32_strict`, `tf32`, ... each declines as
  `UnknownDtype`, not reserved), and a reduced-mantissa request changes the reduction key through `<mp>` while the dtype
  component stays `f32`. 6.1-0006: `u32` is a 4-byte `uint` like `u8`/`u16`/`u64`, derives as the compute dtype of
  elementwise (differing from `f32` only in the dtype component) and reduction cells, as data beside an `i32` index
  operand and as the index operand, and the index role is the caller's declared slot (no slot, no `ndx`). A control
  would catch a precision field or a u32-specific path. **They agree today**: coverage, not a defect. Scope: the
  REFERENCE keyer only; another implementation's dispatch is what a second deriver (R3.3) adds. Floor harness 411 -> 413,
  untested 466 -> 464 (Classify 83/120); 2 `UNBACKED.tsv` rows removed.

## 0.2.33 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **Classify float formats backed** (ROADMAP R3.2, KISS-CLASSIFY-6.1-0011 the FP8 E5M2 format constants, 6.2-0002 the
  special values of every float dtype by pinned bit pattern): `tests/classify_float_formats.rs` decodes EVERY byte of
  `f8e5m2`, `f8e5m2fnuz`, `f8e4m3fn` and `f8e4m3fnuz` from the format definition (sign / exponent width / mantissa
  width / bias / NaN-and-infinity convention), never from `fp.rs`, and holds the crate's own decoders
  (`e5m2_to_f32`, `e4m3_to_f32`, `bf16_to_f32`, `e8m0_decode`) and the platform's `f32`/`f64` against it. Pinned: E5M2
  max finite 57344, IEEE-style infinities and exactly six NaN bytes, `-0` present; the fnuz variants have bias 16/8, no
  `-0`, no infinity, the single NaN `0x80`, and are byte-incompatible (the same byte `0x3C` is 1.0 vs 0.5); f16/bf16/f32/
  f64 infinities, `-0`, quiet vs signaling NaN (distinct encodings, quiet bit), subnormals; `f8e8m0` per 6.1-0013. A control
  proves the decode distinguishes bias and convention. **They agree today**: coverage, not a defect. Not tested, and said so
  in the file: the `f8e6m2` "defines no special values" statement of absence; no implementation's storage (R4.1). Floor
  harness 409 -> 411, untested 468 -> 466 (Classify 81/120); 2 `UNBACKED.tsv` rows removed.

## 0.2.32 — 2026-10-14

- **`tools/run_with_git_bash.py` — run the three bash-launching suites on a Windows box with WSL** (docs/tooling only):
  `CreateProcess` resolves a bare `bash` to the System32 WSL launcher before `PATH`, so `test_kiss_ratchet_step.py`,
  `test_kiss_msrv_step.py` and `test_kiss_orphan_step.py` failed with `execvpe(/bin/bash) failed` and were carried as
  "unrun (WSL)" through 0.2.22-0.2.31. The wrapper rewrites only a bare leading `bash` in `subprocess.run`/`Popen` to
  Git Bash (no-op off Windows) and runs the suite unmodified; `tools/test_run_with_git_bash.py` (7 controls) is a CI
  step. Measured green at `60c375c`: 8/8, 13/13, 7 controls. No spec, corpus or floor change.

## 0.2.31 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **Classify dtype-table clauses backed** (ROADMAP R3.2, KISS-CLASSIFY-6.1-0002 bit widths, 6.1-0003 numeric kinds,
  6.2-0001 the closed kind set): `tests/classify_dtype_table_clauses.rs` parses the clauses' own enumerations
  (`` `f16`/`bf16` = 16; ...``, `` `float`: `f16`, ...``) from `spec/classify.md` at compile time and holds them against
  `corpus/dtype_manifest.json` (the artifact Fuel, Baracuda and kiss-ref vendor) and `structure_key::DTYPES`, in both
  directions, over all 24 dtypes; independent anchors guard against a parser that returns a plausible wrong map; a control
  proves a changed width, a missing dtype, a changed kind and an extra dtype are each reported. **They agree today**, so this
  is coverage, not a defect. Scope: it pins the clauses' enumerated facts; it exercises no implementation's storage (a
  foreign reader, R4.1). Floor harness 406 -> 409, untested 471 -> 468 (Classify 79/120); 3 `UNBACKED.tsv` rows removed.
  Not touched: 6.1-0007 (bool: its normalization half is Ops'), 6.1-0011/-0012 (formats and layout need a byte-level subject).

## 0.2.30 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **`structure_key_derivation_vectors.json` 33 -> 36 vectors: the innermost axis is axis `rank-1` whatever its extent**
  (KISS-CLASSIFY-6.3-0011 with 6.3-0002 and 6.5-0012; ROADMAP R1.4). Three non-`ctr` cells: `[4,1]` (bucket `da`, `v1`)
  and `[1,8]` (`d8`) are the clause's own worked examples; `[8,0]` is the case Unpopped's CHANGELOG held open as
  "deliberately not changed" and **no earlier vector had an innermost extent of `0`**: bucket `da` from axis 1 (`E = 0`,
  6.5-0012 "covering ... `E = 0`"), not `d8` from the outer axis. The `[8,0]` expected token is reference-deriver output,
  not an independent golden. `derivation_vectors_innermost_axis_is_rank_minus_1_including_extents_one_and_zero` pins the
  three buckets as different. **Artifact change, no spec change; no existing vector's token moved.** New blob
  `697aa54370b51df0e26fe5fbd23ceca8419290c5`; consumers that vendored this file re-vendor (Fuel, Unpopped). Floor unmoved (406/34/471).

## 0.2.29 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **Correction to the R2.4 design note and ROADMAP** (docs only): the note called Fuel "the live pair"; Fuel's own
  GAP-305 divergence test says its `to_canonical_bytes` is an envelope that diverges from §6.19.3 on every carrier op,
  so the live OpAttrs pair count is still 0. The note's §4/slice 2 and ROADMAP R2.4 now say so, record the PM's ruling
  that kiss-ref stays a non-party, and R1.9 is added as a **[rule]** row (the `math_precision`/`accumulator` wire
  encodings for `reduce`/`prefix_scan`, and the Appendix E.2 refresh; boarded in PM item 193, not started).

## 0.2.28 — 2026-10-14 (number proposed; the PM allocates at gate time)

- **`opattrs_value_vectors.json` — the first value-bearing OpAttrs vectors** (R2.4 slice 1, #504;
  `docs/design-r24-opattrs-value-vectors.md`): 133 `encode` vectors (attribute tuple -> exact bytes), 7 default-resolution
  equivalences (§6.19-0005) and 112 distinct pairs, for the 15 carrier ops whose schema is fully pinned. Minted by a
  **generic** schema-driven encoder over the parsed §6.19.2 ordinals and §6.19.3 table (`src/opattrs_value.rs`, bin
  `emit_opattrs_value_vectors`); `tests/opattrs_value_vectors.rs` requires it to agree with the hand encoders in
  `opattrs.rs` on every vector they cover (110 of 133; `max_pool`, `rms_norm`, `layer_norm` are generic-only), reproduces
  Appendix E.2's gather/scatter/reduce_var/avg_pool blobs, and carries controls (swapped field order, unknown token,
  missing field, out-of-range axis). **Same author and same reading of the spec as the hand encoders: a target for a
  second party (Fuel), not independent evidence; umbrella 5.3 condition 1 is not discharged.**
- **FINDING recorded in the artifact (`not_emitted`): `reduce` and `prefix_scan` are NOT minted.** §6.19-0025/-0026 add
  `accumulator` and `math_precision` (sk4, 6 bytes), but §6.19.2 pins no ordinal table for `math_precision` (the clause
  says "the `<mp>` enum ordinal of §6.7-0006", which spells the values `st`/`rm` as text), and Appendix E.2 and the hand
  encoders (`encode_reduce`, `encode_prefix_scan`) still show the pre-sk4 4-byte form. A byte for an unpinned field would
  be fabricated, so the generator refuses; a test turns red if either side is fixed so the list is revisited. The spec
  needs a ruling on the `math_precision` wire ordinals (and a refresh of E.2); that is not made here.
- No spec change; no existing file changed; clause ledger and floor unmoved (406/34/471). Opt-in; not yet run by Fuel.

## 0.2.27 — 2026-10-14 (number proposed; the PM allocates at gate time)

- `docs/design-r24-opattrs-value-vectors.md` — R2.4 design note: a value-bearing `opattrs_value_vectors.json` (encode / equivalence / distinct kinds, ~150-250 vectors by estimate), two KISS-side derivations that must agree, Fuel as the live second party, kiss-ref's role left as a ruling. A proposal; no code, corpus or spec change. Docs only.

## 0.2.26 — 2026-10-14

- **`ops-nextafter.json` — the first `nextafter` vectors, value half** (KISS-OPS-6.9-0003, ROADMAP R3.2): 40 exact-byte
  cells, f32 and f64 over 14 shapes plus 6 raw-bit edge pairs: up/down, equal operands (return `b`, so
  `nextafter(+0,-0)` is `-0`), zero crossing to the smallest subnormal, `+-inf` both ways, max finite <-> inf, the
  subnormal/normal boundary, a power-of-two boundary, and a step onto zero that keeps `a`'s sign. A promote-through-f64
  `nextafter` returns the f64 neighbour of an f32 value (for `1.0` toward `2.0` that is `1.0` again). `mint_nextafter.rs`
  recomputes every cell from an independent monotone-key model. **6.9-0003 is counted as PARTIAL, not backed:** its
  typed-decline half (f16/bf16/f8e4m3fn/f8e5m2) has no vector form, so the floor does not move (harness 406, untested
  471) and the `UNBACKED.tsv` row stays `untested` with a note. Regenerating the other bundles is byte-identical.
  Opt-in file; not yet run through kiss-ref or Fuel. No spec change.

## 0.2.25 — 2026-10-14

- **`tools/kiss_version_sync.py` — the manifest version must equal the newest CHANGELOG heading** (blocking, in the
  `kiss-trace` job, with controls in `tools/test_kiss_version_sync.py`). The manifest had stayed at 0.2.21 while
  0.2.22 and 0.2.23 landed (#558, #559 each wrote an entry and neither bumped it) and nothing failed. The controls
  fail on drift (the real 0.2.21 vs 0.2.23 shape), fail on an absent reading rather than pass, and refuse to read a
  dependency's `version =` as the crate's. CI/tooling only; no spec or corpus change.

## 0.2.24 — 2026-10-14

- `HANDOFF.md` rewritten to the state at 0.2.23 (R3.2 atan2/sqrt vectors, kiss-ref 0.4.2-0.4.4 re-vendors, the PARTIAL
  treatment of 6.8-0003, next actions). Docs only. `conformance/Cargo.toml` is set to 0.2.24: it had stayed at 0.2.21
  while 0.2.22 and 0.2.23 landed (their entries are in this file; the crate manifest was not bumped).

## 0.2.23 — 2026-10-14

- **`ops-sqrt-rounding.json` — the first `sqrt` vectors** (KISS-OPS-6.8-0003, ROADMAP R3.2): 24 f32 cells, class
  **ULP with `ulp_bound` 0** (the correctly-rounded arm): exact squares, one ulp either side of 1 and 2, irrational
  results at odd and even exponents, the largest finite, the smallest normal, three subnormals, `+-0`, `+inf`. A
  `sqrt` built as `x * rsqrt(x)` or `exp(0.5 * log x)` passes a 2-ULP gate and is wrong by one ulp on these.
  **Scope:** only the FIRST arm of 6.8-0003 is pinned; a target that does not guarantee correct rounding is held to
  its declared tier (6.8-0001) by the contract, and is expected to fail these cells (a runner applies them where the
  guarantee is claimed). The sign of a zero result is not discriminated by the ULP comparator. `mint_sqrt.rs`
  recomputes every cell from an independent exact integer square root with explicit RNE (not `f32::sqrt`).
  **6.8-0003 is counted as PARTIAL, not backed:** the floor does not move (harness 406, untested 471) and its
  `UNBACKED.tsv` row stays `untested` with a note saying what is covered (the 6.2-0007 precedent). Regenerating the other bundles is byte-identical.
  Opt-in file; not yet run through kiss-ref or Fuel. No spec change.

## 0.2.22 — 2026-10-14

- **`ops-atan2-quadrants.json` — the first `atan2` vectors** (KISS-OPS-6.9-0001, ROADMAP R3.2): 28 f32 cells,
  class **ULP** (`ulp_bound` 4, never exact-byte, §6.8-0005): one ordinary cell per quadrant (two magnitudes
  each), the signed-zero matrix (`y,x` in `{+0,-0}`), `y = +-0` and `x = +-0` against finite operands of each sign,
  and the `+-inf` matrix (`+-pi/4`, `+-3pi/4`, `+-0`, `+-pi`, `+-pi/2`). `atan(y/x)` passes every `x > 0` cell and
  fails every `x < 0` and `x = +-0` cell. **Oracle caveat:** expected values are f64 libm `atan2` rounded once to
  f32, not correctly rounded (no `atan2` hp atom exists); the special cells are fixed by IEEE 754 Annex F.
  `mint_atan2.rs` recomputes every special cell from a hard-coded rule table (not `f64::atan2`) and asserts
  quadrant membership for the ordinary ones. Backs 6.9-0001: floor harness 405 -> 406, untested 472 -> 471.
  Regenerating the other bundles is byte-identical. Opt-in file; not yet run through kiss-ref or Fuel. No spec change.

## 0.2.21 — 2026-10-14

- `HANDOFF.md` rewritten to the state at 0.2.20 (R3.1 sidecars, integer/bool vectors, next actions). Docs only.

## 0.2.20 — 2026-10-14

- **`ops-compare-bool.json` — the first comparison and `bool` vectors** (KISS-OPS-6.2-0005 / -0006, ROADMAP R3.2): 181
  exact-byte cells with expected dtype `bool`, one byte. (a) `cmp_eq/ne/lt/le/gt/ge` over f32 (ordinary, equal,
  signed zeros, NaN on either side, inf), i32 and u32 (the u32 rows include the `0xFFFFFFFF` vs `1` pair on which a
  signed reading flips the answer); (b) `logical_and/or/not` on `bool` operands holding `0x02/0x7F/0x80/0xFF/0x40`,
  results strictly 0/1. An implementation returning a 4-byte `1.0f32` mask, or forwarding `0x02`, passed every
  earlier vector. Independent models in `mint_compare_bool.rs` (f64 widening, signed/unsigned readings, non-zero
  byte). Backs 6.2-0005 and 6.2-0006: floor harness 403 -> 405, untested 474 -> 472. **6.2-0007 stays unbacked** (its
  bitwise/reduction/scan arms have no cells; the ledger note says what is covered). Opt-in file; no existing file
  changed; not yet run through kiss-ref. No spec change.

## 0.2.19 — 2026-10-14

- **`ops-int-wrap.json` — the first integer vectors** (KISS-OPS-6.2-0002, ROADMAP R3.2): 440 exact-byte cells,
  `add`/`sub`/`mul` over i8/u8/i16/u16/i32/u32/i64/u64 at the five wrap-boundary patterns (0, 1, signed max, signed
  min, all-ones; every ordered pair for `sub`, unordered for `add`/`mul`). A saturating implementation passes every
  earlier vector and differs on at least one cell per (dtype, op) here. Minted by `kiss_mint`
  (regenerating the other five bundles is byte-identical); `mint_int_wrap.rs` recomputes every cell from an
  independent i128 model and asserts the saturating discrimination. Backs 6.2-0002: floor harness 402 -> 403, untested
  475 -> 474; OPS untraced 88 -> 87. **Consumers opt in by vendoring the file; no existing file changed.** Not run
  through kiss-ref yet (a re-vendor task). No spec change.

## 0.2.18 — 2026-10-14

- **Clause sidecars for the remaining six sub-standards** (#144 option A, completes the seed): `spec/sidecar/{announce,classify,conform,contract,ops,synth}.json`
  (768 clauses; all nine sub-standards now have one — per-sub counts in `kiss_sidecar.py --check`). Same
  rules as 0.2.17: seeded once, `--write` refused over an existing sidecar, drift is a `kiss_trace` violation and a CI
  step. `determinism_class` and `wire_abi_schema_version` remain `null`/UNSET pending the ruling (PM item 193). No clause
  text, freeze state or coverage figure changes.

## 0.2.17 — 2026-10-14

- **Clause sidecar, first three sub-standards** (#144 option A, PM-ruled; ROADMAP R3.1): `spec/sidecar/{consume,emit,grammar}.json`
  seeded ONCE from the prose (Appendix E form, `generator: hand-maintained`), `tools/kiss_sidecar.py --check` is the
  KISS-CONFORM-6.2-0004 prose<->sidecar token reconciliation (clause IDs and keyword; byte-deterministic serialization),
  wired into CI with seeded-drift controls (`test_kiss_sidecar.py`, 8) and into `kiss_trace` (drift is a document violation).
  `--write` over an existing sidecar is refused: regenerating from prose is option B and inverts 6.1-0007. **Two Appendix E
  fields are NOT set — `determinism_class` (per clause) and `wire_abi_schema_version` — they are `null` and reported as UNSET
  in every check: a ruling is asked; nothing is guessed, so these sidecars are not yet Appendix-E-valid.** No clause text, no
  freeze state, no coverage figure changes. Remaining sub-standards follow in later PRs.

## 0.2.16 — 2026-10-14

- **`--freeze-ready` stops saying `may freeze`** (#387, ROADMAP R3.1): an `[ OK ]` row now reads "every clause has a backing; whether each backing demonstrates its clause is NOT measured", and the summary carries a NOTE that `satisfy` is an existence reading (a sampled audit found about half of even well-evidenced pointers under-describe their clause). It also prints, per sub-standard, `proven / backed / clauses` — the existing `// Proven:` tier (#278) broken out by sub-standard (sums to the 20 the main report gives): **REPORTS, never gates** (#387 option B, PM-directed). Output only: the predicate, the exit codes and the parsed row/summary shapes are unchanged. Which condition-3 wording, or a per-conjunct obligation, to adopt is a ruling, asked separately.

## 0.2.15 — 2026-10-14

- `ROADMAP.md` (R2.7 count 13 of 13; header says which figures are 0.2.14-fresh) and `HANDOFF.md` refreshed to `9ea8c05d`; names kiss-ref#54. Docs only.

## 0.2.14 — 2026-10-14

- **`ops-transcendental-nan.json` gains one vector (12 → 13): f64 `log` of a signaling NaN** (tcId 13,
  `7FF0000000000001` → `7FF8000000000000`). Fuel's macOS CI leg measured Apple's f32 `logf` returning the
  sNaN UNCHANGED (expf/sinf, and every x86 libm, quiet it): a reference that computes the NaN through a platform
  libm rather than the §6.13 decomposition is not portable on this cell, and the f64 `log` had no cell to say
  whether Apple's `log` does the same. Appended, so tcIds 1–12 are unchanged. **Consumers that pin this file's
  vector count re-pin it** (kiss-ref's `kiss_corpus_vectors.rs` pins 12; vendoring this file without that edit
  fails its blob/count guard, by design). No spec change.

## 0.2.13 — 2026-10-14

- **GitHub Actions pinned to commit SHAs** (PM task, portfolio pin patch 2026-10-14): 15 mutable refs in
  `.github/workflows/conformance.yml` and `traceability.yml` (`actions/checkout`, `upload-artifact`,
  `download-artifact`, `setup-python`) now name the latest release's SHA with the tag in a trailing comment;
  node24 runtimes. CI/tooling only; no spec, harness or corpus change.

## 0.2.12 — 2026-10-14

- `ROADMAP.md` statuses (R0.1, R0.3, R0.4, R3.1, R3.4) and `HANDOFF.md` refreshed to current state. Docs only.

## 0.2.11 — 2026-10-14

- **Three clarifications from the first second-party derivation runs** (Fuel#358/GAP-354, Baracuda's harness):
  KISS-CLASSIFY-6.1-0001 now names the key's **dtype positions** (a non-primary operand's dtype is not one, so
  a reserved dtype there derives a key; an unknown dtype in any operand still declines); 6.6-0009 states the
  **producer side** (a deriver given a reduced axis at or beyond the rank MUST decline — the token it would emit
  is one a reader rejects); 6.6-0014 states that the canonical order is the **caller's** presentation (6.6-0012).
  No valid token changes. **Artifact change** (`structure_key_derivation_vectors.json`, 32 → 33 vectors):
  `decline_reserved_dtype` now puts the reserved dtype at operand 0 (a key position) — the first draft put it at
  operand 1 and **both Fuel and Baracuda correctly did not decline it**; new vector
  `reserved_dtype_at_non_primary_operand_derives` pins the derives-a-token reading; the `cell_note` and the
  `call_order_is_not_canonical_order` note no longer say canonicalization is the deriver's job. Reference
  deriver follows. Still owed by implementations (clause text unchanged, MUST already stated): collapsed
  reduction output must decline (6.6-0009), reduced axis out of range must decline.

## 0.2.10 — 2026-10-14

- **A second C toolchain for the differential slice** (#139, ROADMAP R3.4): `KISS_C_COMPILER` swaps the
  harness's `cl.exe` for a `cl`-compatible driver (`clang-cl`), keeping MSVC's include/lib paths; set-but-
  unusable panics instead of silently falling back to `cl.exe`. New CI job `windows-clang-cl` runs the whole
  suite that way and **fails, never skips, if LLVM is absent**; a test asserts the compiler that runs
  identifies as clang. Not a required status context. **Not demonstrated by this PR's author on this box**
  (no clang here): the job is the evidence, and a red there is a finding about the harness, not about this
  change. The loader stays Win32-only; portability is a separate question. No spec, corpus or artifact change.

## 0.2.9 — 2026-10-14

- **The freeze hard-fail the `strict` job promised now exists** (#524): `tools/kiss_freeze_gate.py` reads each
  spec's front-matter `Maturity stage`; any sub-standard that is **Frozen** in the tree MUST pass
  `kiss_trace.py --freeze-ready <SUB>` or the blocking `kiss-trace` job fails (a promotion in the PR and a
  regression of an already-Frozen sub-standard are both caught). Three outcomes kept apart — CLEAN 0, BLOCKED 1,
  DECLINED 2 (an unreadable stage row or base ref is inconclusive, not a pass). Because the whole tree is
  Draft, the controls (`tools/test_kiss_freeze_gate.py`, 10 tests, run in CI) seed the red and run the real
  `--freeze-ready` once to show the BLOCKED arm is not an artifact of an injected callable. **Not covered:**
  who signed (#462), traceability accuracy (#387), conditions 1 and 2. CI/tooling only; no spec, harness or
  corpus change. Also corrects the 0.2.8 entry's vector counts (32, ten A.1-anchored; first stated 30 and nine).

## 0.2.8 — 2026-10-14

- **New corpus artifact `conformance/corpus/structure_key_derivation_vectors.json`** (32 vectors; R3.3 — corrected in 0.2.9: first written as 30) and the
  reference **emit-derivation stage** it is generated from (`conformance/src/derive.rs`,
  `derive_structure_key`): a RAW cell — operands `(use, dtype, extents, strides, alignment, dequant_form)`,
  target, declared hints (index slot, reduced axes, accumulator, `<mp>`) — to the `structure_key`, or a typed
  decline. Until now the codec round-tripped tokens and derived single facts; nothing derived a whole token
  from a cell (§6.6-0019 names this stage as missing). Ten vectors (9 first stated) reproduce Appendix A.1 goldens **derived
  from raw operands** (their expected bytes are the spec's hand-derived tokens, asserted equal); the rest
  are reference output for a second deriver to cross-check. **Scope: every cell class except `ctr`** —
  the batch role is undefined (KISS#543), so a cell that declares contraction roles is a decline, not a
  guess. Fuel/Unpopped/Baracuda can byte-match their DERIVATION, not only their tokens.
  `KISS-CLASSIFY-6.6-0011` and `-6.6-0014` gain real tests; floor harness 400 → 402, untested 477 → 475.
  No spec text and no existing artifact changes.

## 0.2.7 — 2026-10-14

- **Two Classify clauses gain real tests** (`conformance/tests/classify_tail.rs`, R3.2): KISS-CLASSIFY-6.6-0005
  (`test_classify_structure_key_primary_dtype`: operand-0 by position, scale-at-0 is undefined, field 2 is the
  derived primary) and KISS-CLASSIFY-6.7-0012 (`test_classify_reduction_accumulator_coordinate`: absent = the
  compute-dtype diagonal, a deviating accumulator is carried from the closed dtype set and round-trips,
  unknown/reserved spellings decline, the redundant diagonal is not a second spelling of "absent"). Their
  `UNBACKED.tsv` rows are removed; the coverage floor moves harness 398 → 400, untested 479 → 477. No spec,
  corpus or published-artifact change.

## 0.2.6 — 2026-10-14

- **`ops-transcendental-nan.json` gains a `harness_rule` header key** (the key the two minmax files already carry)
  stating that its cells — class `ULP` with a NaN expected (`nan_provenance` computed) — are compared by NaN-ness
  and quietness per KISS-CONFORM-6.8-0010, never by totalOrder distance, sign or payload. The shared `ulp_metric`
  line read "totalOrder distance" and a first-time reader (Fuel) failed `log(-1)` against a negative qNaN. **One
  file changes (12 vectors untouched); no vector, no schema key a loader needs.** Consumers that vendor it
  re-vendor; nothing else moves.

## 0.2.5 — 2026-10-14

- **New corpus artifact `conformance/corpus/ops-minmax-nan.json`** (44 exact-byte vectors, minted by `kiss_mint`,
  `KISS-OPS-6.16-0009`; #329): f32 and f64 NaN cells for `max_prop`, `min_prop`, `fmax_ieee`, `fmin_ieee` — NaN in
  `a`, in `b`, signaling and quiet, a negative-signed NaN, and both-NaN with distinct payloads. Before it no f32/f64
  corpus cell carried a NaN, so the 72 existing minmax vectors passed for an implementation with the propagating and
  IEEE arms swapped. **Acceptance demonstrated:** `mint_minmax_nan.rs` computes each cell from the §6.13
  decomposition on native floats and asserts that a prop↔ieee swap differs from the published bytes on **every**
  cell. **Independent check:** kiss-ref 0.4.0's evaluator (`eval_op`) reproduces all 44 exact-byte (run in a scratch
  kiss-ref worktree with the file vendored; not committed there). Not covered: inf/subnormal operands (no NaN; the
  value arm is covered by ops-minmax-ordinary/-signed-zero) and narrow dtypes (ops-narrow-select-nan.json).
  `test_fns` figure 711 → 712. Consumers vendor the file; nothing existing changes.

## 0.2.4 — 2026-10-13

- **New corpus artifact `conformance/corpus/ops-rounding.json`** (72 f32 exact-byte vectors, minted by `kiss_mint`,
  `KISS-OPS-6.7-0001`): `floor`, `ceil`, `trunc`, `round_even` over ±0, ±0.5, the ties ±0.5/1.5/2.5,
  one-ulp-either-side-of-an-integer, 2^23, largest finite, ±inf and the smallest subnormals. The four are
  primitive-floor atoms with no decomposition, so a vector is their only cross-implementation pin; the corpus
  had none. **No NaN cell** (§6.7-0002 does not pin which NaN, #396). `test_fns` figure in the conformance
  README moves 710 → 711. Consumers (Fuel GAP-300 was this) vendor the file; nothing existing changes.

## 0.2.3 — 2026-10-13 — docs

- `HANDOFF.md` rewritten as current state. No spec, harness or corpus change.

## 0.2.2 — 2026-10-13 — docs

- `ROADMAP.md`: the ordered list of what each sub-standard must reach and what each consumer needs. No spec,
  harness or corpus change.

## 0.2.1 — 2026-10-11 — corrective

The spec rule is **unchanged** from 0.2.0 (Classify §6.6-0008 as amended by #520); the published goldens
now match it. 0.2.0 was never published (`publish = false`).

- **Seven `sk5` positive vectors changed** in `conformance/corpus/structure_key_vectors.json`: the
  keepdim reduction OUTPUT sub-key's broadcast mask is derived, not `00` — `02` (`[4,8]→[4,1]`), `03`
  (`[4,8]→[1,1]`), `01` (`[8]→[1]`), `0a` (`[2,4,3,5]→[2,1,3,1]`): `reduction_trailing_axis`,
  `reduction_all_axes`, `reduction_rank1_all_axes`, `reduction_subset_mask`,
  `noncontraction_acc_mp_field`, `noncontraction_acc_deviating_f64`,
  `noncontraction_reduction_mp_only`. The Classify Appendix A.1 goldens change the same way.
- **Downstream: re-vendor the corpus from the merged #536 commit, not from `a33282c`.** A deriver that
  follows §6.6-0008 already produces these masks.

## 0.2.0 — 2026-10-11 — **Breaking**

One coordinated schema event: KISS-Classify `structure_key` `sk4` → `sk5`, KISS-Contract
`contract_version` 2 → 3 (#263 ruling D, #520, #477, `dequant_form`). Design:
`docs/design-263-cell-class.md`. Every party that derives or parses a `structure_key`, or reads a
contract, regenerates.

### Breaking

- **`op_family` (24 codes) is replaced by the derived `cell_class`** (`ctr ndx rdc ew1 ew2 ew3 oth`,
  Classify §6.5-0006 / §6.5-0015). No op→class mapping exists; a lookup carries the op identity.
- **Every `structure_key` token re-prefixes `sk5`**; a persisted `sk4` token must be re-derived. The
  `sk3` decoder arm is retired (§6.7-0014).
- **`dequant_form` is keyed**: an operand carrying a `quant` record gains a sixth sub-key component
  `dl`/`dc` (§6.3-0009a).
- **The broadcast mask of an extent-1 own axis no longer depends on its recorded stride** (§6.6-0008,
  #520): such an axis sets its bit whenever the frame extent is > 1.
- **Contract `contract_version` is `3`** and the Guarantees field is **`math_fidelity`**
  (was `math_precision`, #477). The KISS-Ops OpAttrs field also named `math_precision` is a different
  channel and is unchanged.
- **The Contract op-category↔op-family table (§6.3-0006) is withdrawn**; the consistency requirement is
  decided from facts the op declares. Consume `wrong-op-class` uses the same requirement.
- **Published artifact spellings changed**: decline kinds `UnknownOpFamily` → `UnknownCellClass`,
  `ReduceNotGatedToRed` → `ReduceNotGatedToRdc` (new: `RdcWithoutReduce`, `BadDequantForm`); the
  decline vector `unknown_op_family` → `unknown_cell_class`.
- **Golden Contract document**: the Guarantees value was the dtype token `f32` (forbidden by §6.8-0004);
  it is now `bit-stable`.

### Not in this release

- Announce: op identity in every lookup (named follow-on, `docs/design-263-cell-class.md` §6).
- #222 candidate 1 (keying a quantized `gem` cell's scale dtype) — needs its own ruling.
