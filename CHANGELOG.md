# Changelog

`kiss-conformance` (the reference crate and the spec it binds). Pre-1.0, a breaking change moves the
second number.

## 0.6.36 — 2026-10-30 (number proposed; the PM allocates at gate time; based on main)

- Docs only (no spec text, no bytes): ROADMAP R1.14 marked done (#618) and R3.10 marked half done, with what was and was not checked (Grammar G2/G3/G5 rendering unchecked).

## 0.6.35 — 2026-10-30 (number proposed; the PM allocates at gate time; based on main)

- Docs only (no spec text, no bytes): `docs/kiss-ref-v4-delta.md` sizes the kiss-ref move from `sk5`/contract v3 to `sk6`/v4 (one vendored file differs, three dtypes, 3-5 lane-hours, best done at the sk7 batch close); ROADMAP R3.17.

## 0.6.34 — 2026-10-30 (number proposed; the PM allocates at gate time; based on main)

- Ledger notes only (no spec text, no bytes, no status change, floor unmoved; ROADMAP R5.6): the eight `UNBACKED.tsv` rows that said `#144 no sidecar` now say what exists (the nine sidecars, `kiss_sidecar.py --check` in CI) and what is still missing.

## 0.6.33 — 2026-10-30 (number proposed; the PM allocates at gate time; based on main)

- Docs only (no spec text, no bytes): ROADMAP R1.28 row no longer lists the derivability witness as open; KISS-CLASSIFY-6.8-0014 already requires it.

## 0.6.32 — 2026-10-30 (number proposed; the PM allocates at gate time; based on main)

- Docs only (no spec text, no bytes): new `SECURITY.md` with the portfolio "Supported versions" text (CireSnave 2026-10-29: the latest line is fixed, an older line only when the backport is small, no support period promised) and which spec versions `kiss-ref` validates (the vendored corpus: KISS 0.2.14 plus bundles to 0.2.26; the current line is not yet covered; the spec itself is a pre-freeze draft with no support promise).

## 0.6.31 — 2026-10-29 (number proposed; the PM allocates at gate time; based on main)

- Tooling only (no spec text, no corpus bytes; CodeRipper OPS-011 `cli-basics`, ROADMAP R3.16): the seven `emit_*_vectors` binaries and `kiss_mint` share `kiss_conformance::cli::basics`. `--version`/`-V` and `--help`/`-h` print and exit 0 with no side effects; an undeclared argument is a usage error, exit 2 (the emitters used to ignore it and exit 0). **`kiss_mint` no longer writes the corpus on a bare run** (it used to rewrite all eleven bundles on every start, even `--version`): it writes only under `--write`. Regenerate with `cargo run -p kiss-conformance --bin kiss_mint -- --write`; `PROVENANCE.md` and the `mint_*` tests' hint messages say so.

## 0.6.30 — 2026-10-28 (number proposed; the PM allocates at gate time; based on main)

- Docs only (no spec text, no bytes): `HANDOFF.md` rewritten as current state (2026-10-29): #724/#725 held for the sk7 batch, board-309 Q1 wire work unbuilt, R1.31 held, R3.15 done (#727).

## 0.6.29 — 2026-10-28 (number proposed; the PM allocates at gate time; based on main)

- Tooling comments only (no spec text, no bytes, no tier moves; Part of KISS#498, ROADMAP R3.15, decision 308 B): 44 more unmarked `// Backs:`/`// Enforces:` citations in 24 files are relabelled `Backs[enforced]:`/`Enforces[enforced]:` because the citing test builds bytes or runs the shipped reader, encoder, decoder, comparator or reference op rather than reading spec text. Judged per test; the other 38 of the 82 unmarked sites stay `stated`: the 29 Emit document lints and the Synth/Classify tests that read spec tables, plus a weak-assertion test, a source-text scan, and two helper-fn citations. LEVELS re-derived: stated 75 to 38, enforced 205 to 226, both 22 to 6.

## 0.6.28 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on Q1)

- Docs only (no spec text, no bytes; union-table finding F1, ROADMAP R5.7; the PM's relay of the Unpopped U1 worker's Cast-to-MX proposal, 2026-10-28): `docs/options-cast-op-f1.md`, an options note on whether KISS adds a `cast` op. Lists what conversion rules are already pinned per destination dtype (bf16, the two fp8 formats, the three MX types) and six that are not (no op; NaN into a no-NaN format; overflow into IEEE-style floats; float to integer; non-RNE rounding; bitcast); sets the U1 proposal beside them (the MX part marked unread and unverified; the f16/bf16/integer part read from Unpopped#144's `cast_ref.rs`: four rounding modes, overflow follows the direction unless ClampFinite, NaN stays NaN float-to-float and goes to 0 float-to-integer, float-to-integer always saturates); gives three options and recommends a `cast` op with `{target_dtype, rounding, saturation}` after four open values are decided.

## 0.6.27 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on Q2)

- Docs only (no spec text, no bytes; ROADMAP R2.8 Q1, KISS#11; CireSnave 2026-10-28, board 309 Q1 = B, with his question "why does the spec forbid a consumer stating a kernel by its recipe"): `docs/options-r2-8-consumer-filed-seam-proposals.md` now records all six board-309 rulings and answers that question. The answer: identity-only requests were a design choice to keep `structure_key` the single source of cell identity (Classify 6.6-0001/-0002) and the op identity a bare root name (Consume 6.2-0006), not a view that a consumer must not ask; the cost is that (cell, bare root op name) does not establish what a fused kernel computes (Classify 6.6-0002, 6.6-0020), which is why carrying the recipe repairs a hole rather than widening one. Lists the five design items the wire work must settle (encoding, checking the answer against the recipe, size and trust bounds, the experimental capability bit, the request shape). The wire work itself is not started.

## 0.6.26 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on Q4)

- Docs only (no spec text, no bytes; ROADMAP R2.8 Q2, KISS#12; CireSnave 2026-10-28, board 309 Q2 = A: add nothing, and check whether the standard requires a provider claiming a backend to cover the basic ops): `docs/options-board309-q2-baseline-coverage.md`. The check found no such clause: KISS-OPS-7.1-0001 and 7.2-0002 bind evaluators and op-subset claims, KISS-SYNTH-7.1-0001 lets a provider answer every request with a decline, and the umbrella's claimable subsets are per sub-standard, not per cell. The note proposes an optional, claimable floor-complete-provider profile (experimental capability bit plus Conform vectors) as the recommended rule, with status quo and a mandatory baseline as the alternatives. Informative; the decision is CireSnave's.

## 0.6.25 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on Q5)

- Spec text and one test arm, no wire bytes (ROADMAP R2.8 Q4, KISS#19; CireSnave 2026-10-28, board 309 Q4 = C, "NOW (not when a second tag is seeded)"): the KISS-SYNTH-6.4-0001g tag table gains an `ABI model` column (the `PTX ` row is `pointer-abi`, the Interface list bound as typed pointers and scalars in order) and the rule that a row stating no ABI model is not a registered value; the named set is `pointer-abi`, `descriptor-set`, `buffer-index`, `host-closure`, of which only `pointer-abi` is defined because only it is used. An informative table lists every artifact format KISS recognises or needs with its target-family registry status (read from `conformance/registry/namespaces.json`: `cuda` and `vulkan` registered, `rocm` and `metal` reserved, no Intel or CPU entry) and a likely ABI model; PTX is NVIDIA-only, and the AMD, Intel, Metal and host rows are marked UNVERIFIED because none was read. The table stays one registered row (`test_synth_artifact_format_tag_registry` still pins that); `test_synth_artifact_format_tag_abi_model` reads the new column.

## 0.6.24 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on Q6)

- Spec text, no bytes (ROADMAP R2.8 Q5, KISS#21; CireSnave 2026-10-28, board 309 Q5 = A): the rule that is already true per field is stated once. KISS-CONTRACT-6.1-0006 gains arm (d), a value outside a closed value set of a field that affects correctness or admissibility is a typed decline, and a reader MAY ignore an unknown value only of a field its own clause marks advisory; KISS-CONSUME-6.5-0001 lists the same case among the refusals that MUST be typed declines. Records existing behaviour (unknown determinism class already declines as `unknown-determinism-class`; unknown field, version and dtype token already decline). No vector, no byte. The `UNBACKED.tsv` note for 6.1-0006 records which arm the test does not reach.

## 0.6.23 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on Q3)

- Spec text, no bytes (ROADMAP R2.8 Q6, KISS#27; CireSnave 2026-10-28, board 309 Q6 = A): KISS-CONTRACT-6.8-0002 now carries an explicit exception, a kernel with no finite bound on a backend declares no tier for it (its `precision_class` is `unbounded` per KISS-OPS-6.8-0008 and its `audited_status` is `unaudited` per 6.8-0010), which closes the inconsistency where "MUST declare at least one of max_ulp, max_relative, max_absolute" read as forcing a provider to invent a bound; and it adds the SHOULD: a provider unsure which bound a kernel meets SHOULD understate (declare the looser bound it can establish, or no tier) and SHOULD NOT declare a tighter bound than it has established. The reference reader already accepts a tier carrying none of the three quantities (`DeclaredAccuracyTier::is_bounded`), so no code or vector changes. 6.8-0002 stays `untestable` in `UNBACKED.tsv` (#44).

## 0.6.22 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on #715)

- Informative text only (no clause, no bytes; ROADMAP R2.8 Q3, KISS#18; CireSnave 2026-10-28, board 309 Q3 = A): umbrella §8.1 gains one informative sentence. The suite defines no in-process profile; an implementation that exchanges no bytes conforms by producing, on the KISS vectors, the same observable results as a byte reader would, and the in-process code shape belongs to the interface project (decision 297). Fuel's route to claim conformance with its linked codegen is therefore the vectors, not an adapter.

## 0.6.21 — 2026-10-28 (number proposed; the PM allocates at gate time; stacked on #714)

- Informative text only (no clause, no bytes; ROADMAP R5.8 SLA item; board 279 pair 3): umbrella §8.3 gains one informative sentence recording CireSnave's ruling of 2026-10-28, relayed by the PM, verbatim "#2: Promise nothing but we will get to it as quickly as possible." Timing option A (as resources permit; no date, no published queue rule) and interim status (i) (the registry lists verified implementations only; no "self-certified, pending" row), per the PM's reading "unless he objects". The other R5.8 rows are unchanged. **Also (docs, R5.7):** `docs/union-coverage-unpopped.md` folds in the Unpopped lane's row check of the table (its messages of 2026-10-28, `c9ab05b`): rows 1-19, 24 and 26-32 confirmed, Unpopped's corrections added, and six constructs it found missing added as Part 3 (cast with rounding and saturation, `fma`, per-op numerics qualifiers, exact integer arithmetic, packed lane values, the MX dtypes); 37 rows in all. This lands here because #714 merged before the fold-in commit.

## 0.6.20 — 2026-10-28 (number proposed; the PM allocates at gate time)

- Docs only (no spec text, no bytes; ROADMAP R5.7, board 304; CireSnave 2026-10-28 via the PM: "I don't want KISS to be unable to describe something Unpopped can build."): `docs/union-coverage-unpopped.md`, a union-coverage table at `bafbe07` of 37 Unpopped IR (31 asked for, 6 added from the Unpopped lane's row check)/Schedule families and kernel-language features against Ops/Contract/Synth/Classify, each marked EXPRESSED / BY COMPOSITION / NEEDS / NO WAY with the clause ids. Answers the PM's three flags: `MAX_OPERANDS = 8` is a hard key limit (Classify 6.4-0002), so a 9-operand causal conv1d needs a limit change or a composition; `matmul` with no OpAttrs says transposed and batched through strides and role hints but not mixed dtype or accumulator dtype (it lacks the `reduce` precision fields, and an integer accumulator is forbidden by 6.17-0008); padding needs separate low/high. Finding F1: the shape oracle and Conform name `cast`, `transpose`, `unsqueeze`, `concat`, but no op catalog entry or OpAttrs schema exists for them. Unpopped's IR facts are only those its lane sent (marked [U]); its lane is asked to check every row.

## 0.6.19 — 2026-10-28 (number proposed; the PM allocates at gate time; 0.6.18 is #712)

- Corpus bytes, provenance members only (CireSnave 2026-10-28, board 315 item 5, option A, verbatim "5: A"; KISS#218; ROADMAP R1.28): `structure_key_vectors.json` and `structure_key_derivation_vectors.json` now say `generated_from` = the emitting program (`conformance/src/reference_vectors.rs (cargo run --bin emit_structure_key_vectors)` and `conformance/src/derivation_vectors.rs (cargo run --bin emit_structure_key_derivation_vectors)`), not `spec/classify.md`; and `source_commit` (the never-updated literal `19c3ad7`) and the `SOURCE_COMMIT` constant are dropped from `structure_key_vectors.json` and its emitter. Both members are part (c) PROVENANCE under KISS-CONFORM-6.3-0010, so no vector, scoping member or `structure_key_schema_version` moves and a consumer may not rely on either. Regenerated with the two `emit_*` binaries; the delta is exactly those three lines. Vendors (Fuel, Unpopped) regenerate their copy and update the one pin (`source_commit`) in their byte-match test.

## 0.6.18 — 2026-10-28 (number proposed; the PM allocates at gate time)

- Tooling comments and ledger text only (no spec text, no bytes, no tier moves; Part of KISS#498, ROADMAP R3.15, decision 308 B): 197 unmarked `// Backs:`/`// Enforces:` citations in 40 files are relabelled `[enforced]` where the test builds real bytes or runs the shipped reader/oracle and asserts on its behaviour (a classification pass, spot-checked on `announce_frames.rs`, `semantics_golden.rs`, `emit_membership.rs`). LEVELS at this branch: 75 clauses carry a stated test, 205 an enforced test, 22 both (was 211 / 79 / 32 at 0.6.17). Not relabelled: the doc-lint tests (`emit_*`, `mint_*` corpus-content checks, registry/class-table checks) stay `stated`, and 39 judgement calls the classifier flagged unsure stay unmarked (the default) for a human read. R1.31 recorded as held (PM ruling 2026-10-28, option b): `KISS-SYNTH-6.4-0001f` stays `untested`, floor 392, no loader to be written in kiss-ref; ROADMAP R1.31 and the COVERAGE_FLOOR note say so.

## 0.6.17 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `HANDOFF.md` rewritten (not appended) at the throttle boundary: THROTTLED until ~2026-10-28T01:00Z (gates, fixes and security only), the remaining linear stack #702-#710 and how to handle a HOLD on it, the rulings of decisions 308/310/311, and what is owed (ROADMAP R3.15 relabel sweep, R1.31, R3.14, the stale R1.28 row text, the Fuel `negotiate()` check, the weekly sweep due 2026-11-01).

## 0.6.16 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Tooling (no spec text, no bytes; CireSnave decision 308 B, "add the level to the citation"; Part of KISS#498; stacked on #709): a citation now names its level. `// Backs[enforced]: KISS-X` tests at the **enforced** level (real bytes or the shipped reader); an **unmarked** `// Backs: KISS-X` is **stated** (reads the spec text); the assertion-argument form (`assert_golden`, `assert_token`) is enforced by construction; `Enforces` takes the same bracket; an unknown level word is malformed (the run fails, nothing is credited). `tools/kiss_trace.py` parses and counts both levels (new LEVELS lines in the report); `kiss_cite_form.py` accepts the bracket; `kiss_backs_matrix`, `kiss_uncited`, `kiss_cites` and `kiss_bundle` read the same parser and are exercised with a levelled citation by the new `tools/test_kiss_levels.py` (added to the kiss-trace job). **Migration:** the 26 clauses left in the "a DIFFERENT test carries it" group (35 when #684 measured it, 9 fixed by #709) now carry labelled citations on the named test and on every second test: 86 tests in 28 files, comments only. Result at this branch: the group is **0**, "the §9-named test carries it" is 258, and the report shows 211 clauses with a stated test, 79 with an enforced test, 32 with both. CONTRIBUTING convention 15 documents the form and withdraws the reserved `Supplements:`. **Not relabelled:** every other unmarked citation stays `stated` by the ruling's default, so the stated count still includes implementation-level tests until a wider sweep.

## 0.6.15 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Tests (no spec text, no bytes; CireSnave decision 308, "...and yes, review the 9"; Part of KISS#498; stacked on #708): the review of the nine clauses whose named test existed but did not cite them. All nine named tests were found to **discriminate**; none was a stale pointer. All nine now cite their clause in the test body (`// Backs: ...`, which the tracer reads as a backing; `// Clause:` is only a mention): KISS-CLASSIFY-6.8-0008/-0009/-0012/-0013, KISS-CONFORM-6.5-0017/6.8-0010, KISS-CONTRACT-6.6-0006/6.9-0009/6.11-0011. Two tests were **strengthened** because they exercised fewer conditions than their clause states: 6.8-0008 (every listed envelope field, an integer `vocabulary_version`, a registered `namespace`) and 6.11-0011 (an unsupported `contract_version` member is isolated like a bad CRC). 6.6-0006, 6.9-0009 and 6.11-0011 each have further citing tests (`Backs:`), so their coverage was wider than the named test showed. No clause moved tier.

## 0.6.14 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Emit, **informative only, no new MUST or SHOULD**; the PM approved this exact scope on #701; Part of KISS#234; stacked on #707): one note under KISS-EMIT-6.6-0005 saying a relation between two schedule variants of one cell is implementation-private and not a KISS field (ROADMAP R1.5, ruled 2026-10-15), with its only possible referent being the default lowering of the same cell, same backend, same version. Lets an implementer tell off-schema from unmentioned.

## 0.6.13 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: ROADMAP housekeeping. R1.26 rewritten to what is left after #675/#681/sk6 (the block length and group axis; points to `docs/options-r1-26-block-scaling.md`); R1.30 marked **done** with the OCP result (no value differs; packing is a convention; `e3m4` is not an OCP MX v1.0 type); new row **R2.9** for KISS#396 (the NaN kind of the rounding atoms, `[rule]`; I closed #396 by mistake once, reopened it and retracted my comment).

## 0.6.12 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r3-6-tooling-and-census.md`, the options note for R3.6 (KISS#458, #400, #484) for the board (the PM can decide #484 alone): where a test that binds the document is recorded (recommend leaving it to the mutation audit, and Rust-side discovery if the count grows), whether a clause may name several tests (recommend keeping one, together with R3.8), and how the affected-set census is built (recommend a paragraph in `CONTRIBUTING.md`: key on what a party's code emits or parses, anchor on a token the clause owns, negative control on the homonym). No normative text.

## 0.6.11 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-26-block-scaling.md`, the options note for R1.26 (KISS#32) for the board. States what has landed since the issue was filed (MX element formats #675, OCP verification #681, scale dtype keyed from `sk6`), so the row is stale, and asks the four questions that remain: key the block length (recommend at the next key-schema event), a closed `{16, 32}` set, an implied group axis, and who pins packing and conversion rounding (recommend waiting for a Baracuda hardware measurement). No normative text.

## 0.6.10 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-22-realize-time-and-reader-obligations.md`, the options note for R1.22 (KISS#424) for the board: whether KISS regulates the tensor seam at realize time and binds its readers (A: a new sub-standard, B: record it as out of scope by ruling, recommended; C: a narrow reader-honour clause on external token bits, marked untestable until a second reader exists). Re-searched at `origin/main` with controls: none of the other format's obligations are in `spec/`; its figures are the Fuel architect's and were not re-measured. No normative text.

## 0.6.9 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-19-rank-reducing-reduce.md`, the options note for R1.19 (KISS#426) for the board: how a rank-reducing reduction gets a conformant contract when `reduce` always keeps the axis (recommend a `squeeze` op so `reduce` + `squeeze` composes; alternatives: say it is out of scope with an exception in KISS-CONTRACT-6.4-0011, or add distinct rank-reducing reduce ops). Re-measured at `origin/main` with controls. No normative text.

## 0.6.8 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-13-write-side-layout.md`, the options note for R1.13 (KISS#316, with #236) for the board: whether a write-side layout guarantee goes in Guarantees (per output), in Interface, or waits (recommend Guarantees if #694's `contract_version` bump is still open, otherwise wait); and a statement that #236's first question was already ruled on 2026-09-05 and the issue is the tracker of the four-phase sequence. No normative text.

## 0.6.7 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-28-provenance-derivability-variants.md`, the options note for R1.28 (KISS#218, #228, #234) for the board: whether to rewrite `generated_from` in the two program-written vector files now (clause 6.8-0011 is already fixed; recommend with the next regeneration), #228 (derivable capability), which KISS-CLASSIFY-6.8-0014 already requires (#340; my first version of the note missed it and is corrected), and #234 (variant-to-variant bit relation), which is **already ruled** by R1.5 (2026-10-15, implementation-private) and needs at most one informative sentence in Emit. No normative text.

## 0.6.6 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `HANDOFF.md` rewritten (not appended) for the next session: the clean stack #693-#699, the rulings of decisions 310 and 311 (310 Q1 re-opened, Q4 open), the owed options notes (R1.28, R1.13, #426, #424, #32, R3.6, #396), ROADMAP housekeeping, owed witnesses, the issue-clock state, and the one-linear-stack procedure the PM now requires.

## 0.6.5 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Synth, additive; CireSnave decision 310, R1.24 Q7 B; Part of KISS#317; stacked on #698): new KISS-SYNTH-6.4-0001h: in a provision response the contract's `entry_point` resolves **inside the artifact** the same frame carries, by the rule in a new **"How `entry_point` resolves"** column of the `artifact_format_tag` table (6.4-0001g); a row that states no rule is not a registered value. The PTX row: `entry_point` names an `.entry` directive in the PTX text. The per-kernel link field in the Contract (option A, Breaking) is deferred until a provider with a non-self-contained artifact exists. The same column is where a format's calling convention (R2.8) goes. Backed by `test_synth_entry_point_resolves_in_artifact`; emptying the PTX cell fails it.

## 0.6.4 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Consume, additive; CireSnave decision 310, R1.24 Q6 A; Part of KISS#289; stacked on #697): new KISS-CONSUME-6.5-0004: a kernel that declines an invocation whose derived `structure_key` byte-matches its own MUST report a typed decline, and MUST NOT decline silently, abort, or return the result of a different computation. It restores the liveness property that dropping "if and only if" from KISS-CLASSIFY-6.6-0001 removed, without restoring the unsound "must admit". Stated in Consume, not Classify (one obligation, one sub-standard). Reference: new `key_match` module (a model of the three answers). Backed by `test_consume_key_matched_decline_is_typed`; making a match always admit fails it.

## 0.6.3 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Conform, additive; CireSnave decision 310, R1.24 Q5 A; Part of KISS#124; stacked on #696): new KISS-CONFORM-7.2-0005 extends the opt-in trust-but-verify profile: verification ends as **adopted**, **adopted-demoted** or **rejected**; only a value difference can be demoted (one demoted state, no lattice) and demotion is a local reclassification, not a claim that the provider's contract is wrong; the consumer keeps a durable record per (kernel, cell, target) with the `revision_hash` verified, and the record is invalid for any other revision. No wire bytes; consumes the oracle Conform owns. Reference: new `verify_record` module (`outcome_of`, `VerifyLedger`). Backed by `test_conform_verify_outcomes_and_record`; dropping the revision check fails it. A second implementation's witness is still owed (ROADMAP R1.31 is the same kind of row).

## 0.6.2 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Announce, additive; CireSnave decision 310, R1.24 Q3 A, "I lean toward A...role awareness", taken as ruled; Part of KISS#25; stacked on #695): new KISS-ANNOUNCE-7.3-0003 pins how the two capability words combine: `live = (local & remote & !FEAT) | (provider_word & FEAT)`. An EXT or SUB bit is live iff both sides set it; a FEAT bit is live iff the provider set it. The issue's plain `local & remote` would read a provider-only feature as off, because the consumer must write it as zero (7.3-0002). Masks by axis, not by recognition. One worked vector in the clause and as bytes in `announce_vectors.json` (`negotiation`, two role orderings, additive: `frames` is unchanged). Reference: `live_capabilities`, `negotiate_live`. Backed by `test_announce_live_capabilities_role_aware`; replacing the rule by a plain AND fails it. **Fuel to check:** a `negotiate()` that returns the plain AND reads provider-only features as off.

## 0.6.1 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Synth, additive; CireSnave decision 310, R1.24 Q2 A; Part of KISS#24; stacked on #694): new KISS-SYNTH-6.6-0001d, the consumer's mirror of the provider's never-panic obligation: every failure observed at the seam (malformed or truncated frame or contract, unsupported `artifact_format_tag`, length over its cap, refused load, failed verification) becomes a typed error and never an abort. The honest limit is in the clause: a fault inside the kernel's native code after launch cannot be caught by any seam clause, and isolation is the deployment's job (umbrella §10.6). Backed by `test_synth_consumer_seam_failure_is_typed_error` (every truncation of a valid PRSP, a corrupted tag, an over-cap length, and short junk; mutation of the cap check fails it).

## 0.6.0 — 2026-10-26 (number proposed; **Breaking**; the PM allocates at gate time)

- **Breaking** (Contract wire; CireSnave decision 311, Q1 option C; Part of KISS#1; stacked on #693): the 10-bit mantissa floor of KISS-OPS-6.17-0003 is retired and replaced by a **declared width** checked by the per-atom width probe. **Implemented by value, not by a new field**: the Guarantees `math_fidelity` is now `bit-stable` or `reduced-mantissa-permitted:<n>` (n = 1..52, mandatory; the bare spelling is declined), new KISS-CONTRACT-6.8-0004a. bfloat16-input rounding is `n = 7`, TF32 `n = 10`. A `bit-stable` declaration is byte-identical to before; the Classify `st`/`rm` key tokens and the OpAttrs ordinals (1/2) are unchanged. **`contract_version` 3 -> 4** (KISS-CONTRACT-6.1-0008, §8-0002): the header line of every contract changes, so `contract_vectors.json` is regenerated (version byte and CRC only). Ops 6.17-0003, -0005, -0006 and Appendix D.1 now speak of the declared width. Reference crate: new `fidelity` module (`parse_math_fidelity`, `round_mantissa_rne`, `probe_retained_bits`, `declaration_honoured`). Backs `test_contract_math_fidelity_declared_width` and `test_ops_math_precision_reduced` (KISS-OPS-6.17-0003, previously untested). Impact at `origin/main` of each consumer: no `reduced-mantissa-permitted` contract is shipped by Fuel, Baracuda or Unpopped (only the `rm` key token and comments); each must accept `contract_version` 4 in any contract it reads.

## 0.5.11 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Conform, additive; CireSnave decision 311, Q2 A; Part of KISS#4; stacked on #689): new KISS-CONFORM-6.8-0004a pins the step between a contract's declared tolerance and a comparator's threshold: a declared tier selects the ULP/tolerance comparator and admits within every bound; a declared accumulator band selects the order-invariant comparator with the absolute tolerance `k·N·eps_A·|max partial sum|`, evaluated per invocation; a contract that declares neither yields no threshold and no default. One worked vector (band 0.25). Reference wiring: new `tolerance` module (`comparator_threshold`, `threshold_admits_f32`). Backed by `test_conform_tolerance_threshold_from_declared`.

## 0.5.10 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Ops/Conform, additive; CireSnave decision 311, Q5 option A with C as its test; Part of KISS#293): a ULP result is measured against an **audited wide-precision reference**, never against another implementation. KISS-OPS-6.0-0003 now says so for an op (as 6.8-0001 already did for an atom), and new KISS-CONFORM-6.8-0002a forbids recording or claiming a ULP result from a two-implementation comparison. Backed by `test_conform_ulp_referent_is_wide_reference` over a shared-defect vector: two implementations of `exp(1)` that agree at 0 ULP and are 6 ULP from the reference. Reference crate gains `UlpReferent`, `UlpResult` and `ulp_result_is_recordable`.

## 0.5.9 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-25-region-and-recipe-identity.md`, the options note for R1.25 (KISS#28, #67, #125) for the board: whether a folded constant is part of a region (recommend a `Const(bits)` leaf), where the recipe grammar lives (recommend KISS-Ops 6.12 as the home, plus an index), and whether cost is one number or several (recommend an optional sibling with `{compute, bandwidth}`; `cost_provenance` already exists at Contract 6.8-0006). No normative text.

## 0.5.8 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-23-precision-floors-and-guarantees.md`, the options note for R1.23 (KISS#1, #4, #13, #56, #293): five questions with an example, what the specs say (`file:line`), options, a recommendation and what waits. Notes that half of #13 is already answered by `bit_stability` (Contract 6.8-0013) and records Baracuda's measured 13.9% / exactly 1 ULP divergence for #56. For the board. No normative text.

## 0.5.7 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r1-24-seam-obligations.md`, the options note for R1.24 (KISS#10, #24, #25, #26, #124, #289, #317): seven questions, each with an example, what the specs already say (`file:line`), options, a recommendation and what waits. Notes that #124 is partly answered (the opt-in trust-but-verify profile exists) and that the plain-AND capability rule proposed in #25 would zero provider-only bits. For the board. No normative text.

## 0.5.6 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r2-8-consumer-filed-seam-proposals.md`, the options note for R2.8 (KISS#11, #12, #18, #19, #21, #27): six questions with a concrete example, what the specs already say (`file:line`), options, a recommendation and what waits. For the board. No normative text.

## 0.5.5 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Docs only: `docs/options-r3-8-two-level-backing.md`, the options note for R3.8 / #498 (how the traceability report should show a clause tested at two levels), for the board. Measures the "different test carries it" group at `b9e08e7f`: 35 clauses, all with an existing named test, 26 of which mention the clause. No normative text.

## 0.5.4 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Synth, additive; CireSnave decision 297, R5.2 option A — overruling the note's recommendation of B): **KISS-Synth owns the `artifact_format_tag` values**, in the table of new **KISS-SYNTH-6.4-0001g**. A provider writes only listed values; a consumer does not load an artifact whose tag it does not support, and no unlisted tag is supported; a value is added by changing the table. The tag names the binary format and carries no architecture (the target is the request's `target_capability` and the artifact's own header). **Seeded with exactly one value**, Baracuda's reference provider's: `PTX ` (wire `50 54 58 20`, namespace `cuda`, emitted today); cubin is NOT seeded (one seam maps it but no provider emits it). Defaults stand: no "building, retry" answer, paging deferred; Appendix B updated. Reference: `REGISTERED_ARTIFACT_FORMAT_TAGS`, `is_registered_format_tag`; the test parses the spec table and compares it with the constant (mutation-checked).

## 0.5.3 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Emit/Consume, additive; CireSnave decision 297, R5.9 option C): the round-trip is declared **maintained, not structural**. Emit §6.7 and Consume §6.6 each state that their correspondence is kept true by the editor updating both documents together (and by the cross-standard lint), and point to the interface project (`docs/interface-project-plan.md`) as the eventual structural enforcement. **Two correspondence rows added:** Emit 6.7-0006 (the language-identity token and `target_capability` determinants of tier 2) now has a Consume twin, new **KISS-CONSUME-6.6-0007**, and Emit 6.7-0007 (the tier-1 comparator) has a twin, new **KISS-CONSUME-6.6-0008**; the 6.7-0008 table is 8 rows and the lint test pins the explicit sets (mutation-checked: removing the keyword from 6.6-0008 reddens it). The shared term **language-identity token** is cited from KISS-Emit §3 in a Consume §3 entry (a cross-reference, no dependency edge). **Ledger:** both new Consume clauses are `untested` (a restriction on what an implementation may claim; same standing and reason as their Emit twins), so the untested floor rises 391 to 393 deliberately. No wire byte, token or vector changes.

## 0.5.2 — 2026-10-26 (number proposed; the PM allocates at gate time)

- Spec (Classify/Ops, provenance only; CireSnave decision 301 follow-up, ROADMAP R1.30): the MX constants were read against the primary **OCP Microscaling Formats (MX) Specification v1.0 (Sep 2023)**. **No value differs**: `f4e2m1` (Table 5), `f6e2m3` and `f6e3m2` (Table 4) and the E8M0 scale (Table 7) match 6.1-0013/-0014/-0015 exactly. The clauses now say **verified-against-OCP** for the constants and **convention** for the packing (OCP fixes no byte or bit packing; the layout stays Unpopped's, unchecked against CUTLASS/Mojo/Triton). 6.1-0013's "zero is inferred" wording is replaced by the spec's `Zeros: N/A`. Ops dtype rows for the three MX types note the OCP-required conversion (round half to even, saturate). `e3m4` is not an OCP MX v1.0 type. No token, byte or vector changes.

## 0.5.1 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Spec (additive; CireSnave board 294, R5.3 option A; stacked on #678): the trust-boundary clauses of umbrella §10.5 items 1-5. **Caps:** KISS-SYNTH-6.4-0001e (`artifact` <= 2^28, `contract` <= 2^24, rejected before any read or allocation keyed on the length, supplied bytes or not) and KISS-CONTRACT-6.11-0003a (document body <= 2^24), joining the Announce caps already present; all placeholders, revisable before any freeze. **Statements:** KISS-SYNTH-6.4-0001f (a provided artifact is untrusted code unless the provider is in the consumer's trust domain), KISS-ANNOUNCE-6.3-0008a (`revision_hash` is not an integrity mechanism), KISS-CONTRACT-6.11-0003b (the CRC-32 detects corruption, not modification). **Reserved slot:** KISS-ANNOUNCE-7.2-0011 (FEAT bit 34 reserved by name as `ARTIFACT_ATTESTATION`; no payload defined; no wire byte moves). Umbrella 10.5 records the status. **Ledger:** 6.4-0001f is `untested` (a consumer-loader policy with no in-repo behaviour to witness), so the untested floor rises 391 to 392 deliberately; harness floor 465 to 470. Reference: `MAX_ARTIFACT_LEN`, `MAX_DOCUMENT_BODY_LEN`, `SynthDecline::LengthOutOfRange`. The contract-document cap reuses `BadLength` (no new decline category).

## 0.5.0 — 2026-10-25 (number proposed; **Breaking**; the PM allocates at gate time)

- **Breaking** (Announce wire; CireSnave decision 302, option B; stacked on #677): the correlation block of KISS-ANNOUNCE-6.4-0013 is now a **fixed-position, always-present length field**, the last component of every echoed identity block: `correlation_len` (`0` or `8`) then that many bytes. Where a synchronous seam could omit the old optional block entirely, every frame now carries at least the byte `00`, so `CYRQ`, `CRSP`, `CDEC` (and Synth's `PRSP`) are self-delimiting and the stated limitation in 6.2-0014 is removed. **Bytes change for those frames:** each is one byte longer when it carries no id; `announce_vectors.json` regenerated (exactly the three frames `cyrq_request`, `crsp_response`, `cdec_response` +1 byte; envelope and list unchanged). Reference: `Identity::correlation`, `AnnounceDecline::BadCorrelationLen`, `correlation_id::{encode_field, parse_field}` (the old `encode_tail`/`parse_tail` are gone). **Implementations checked (origin/main of each, `git grep` for CYRQ/CRSP/CDEC/PRSP/SAVL in .rs/.toml/.json): Fuel, Baracuda and Unpopped have no code for these frames (Fuel documents them in `docs/outreach/kiss-conformance-and-divergences.md` only); each speaks the 56-byte envelope, which is unchanged, so no implementation impact found.** Also adds ROADMAP R1.30 (verify the MX constants against the OCP PDF, decision 301). Numbered 0.5.0 (a breaking change pre-1.0 moves the second number).

## 0.4.90 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Spec (Announce, **normative, additive**; CireSnave board 294, R5.1 option A without any transport binding): defines **message extent** (§3) as the number of bytes a message occupies, computed from its own leading tag and length fields only. New 6.2-0014 (extent from the message alone, for all five messages), 6.2-0015 (a message cut short is truncated and rejected whatever carried it), 6.2-0016 (an unknown leading tag is a typed decline, no resynchronisation), 6.2-0017 (bytes after the extent are not read), and 6.4-0014 (contract-response payload length capped at 2^24; **a proposed value, no producer measurement exists**). The vague wording in 6.2-0001, 6.2-0007, 6.3-0009, 6.3-0010 and 6.4-0006 now uses the defined term. **No byte of any message changes** and no transport is named (umbrella §10.3). Reference: `announce::message_extent`, `MAX_CONTRACT_PAYLOAD_LEN`, decline `PayloadLenOutOfRange`. **Stated limitation recorded in 6.2-0014:** the optional correlation block of 6.4-0013 is not determinable from a frame's own bytes, so a frame carrying it is not self-delimiting; a decision is owed before any freeze. Numbered 0.4.90 because #675 proposes 0.4.89.

## 0.4.89 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Spec (Classify, **normative**, CireSnave board 290: "Making Unpoppped's IR a true union of the languages takes precendence over design decisions to keep things out I may have made in the past."): (Numbered 0.4.87: 0.4.85 was proposed by this change and skipped on purpose because 0.4.86 (#673) merged first.) three MX **element** dtype rows join the closed set, additive and usable: `f4e2m1` (FP4, 4 bits), `f6e2m3` and `f6e3m2` (FP6, 6 bits), appended after `c128` so no §6.19-0025 ordinal moves (27 recognized, 24 usable, 3 reserved). New clauses 6.1-0014 (f4e2m1 format, nibble packing as `i4`), 6.1-0015 (f6 formats, four-per-three-bytes LSB-first packing), 6.1-0016 (storage/operand dtypes only: never an accumulator, new typed decline `ElementDtypeAsAccumulator`). **Unverified:** the format constants were supplied from memory by the Unpopped lane, not read from the OCP MX v1.0 PDF, and the FP6 packing is Unpopped's convention, unchecked against CUTLASS/Mojo/Triton; both stated in the clauses. No existing token moves, no schema-version bump (6.7-0007). Corpus: `dtype_manifest.json` +3 rows, `structure_key_vectors.json` +3 positive and +2 decline vectors, `structure_key_derivation_vectors.json` +5 vectors.

## 0.4.88 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: `docs/interface-project-plan.md`, the one-page plan for the interface project (board 295; name left as `<name>`; A/B/C open). Numbered 0.4.88 because #675 proposes 0.4.87.

## 0.4.86 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: `docs/PATENT-SWEEP.md` second pass (PM follow-up on #672). All independent claims of the 8 patents read; a CPC/similar-documents walk found six more; one `needs-review` (US8458680B2, Google, a runtime that picks a processor type and resolves a precompiled kernel; an implementer-level question, not the documents). The record states what is still not done. Not legal advice.

## 0.4.84 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Spec (Classify, **normative**, CireSnave board 291 option B: "For the width, make it per-target."): the vector-access byte cap of KISS-CLASSIFY-6.5-0009(c) is no longer a fixed 16 but the cap the key's target namespace declares (new KISS-CLASSIFY-6.8-0018; bytes; a namespace declaring none, or with no vocabulary document, is 16). `cuda.md` and `vulkan.md` declare 16, so **no existing key moves**; the `vec` token set stays `{v1,v2,v4,v8}`. Derivation vectors 3 added (`vec_cap_undeclared_is_16_bytes`, `_32_bytes_widens_f32_to_v8`, `_8_bytes_narrows_f32_to_v2`; the last two carry an optional `target_vec_access_cap_bytes` key, absent from every other vector). `derive_vec_width_capped` and `DerivationCell::vec_cap_bytes` are new; `derive_vec_width` is unchanged. **No namespace vocabulary version bump:** the declaration alters no token byte, and both namespace documents' own rule bumps only for byte-altering changes (flagged to the PM).

## 0.4.82 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: `docs/PATENT-SWEEP.md`, the first patent sweep for the 0.4.x drafts (CireSnave board 286). A partial first pass: 11 ideas, 11 queries, 8 patents read, all `clear` on claim 1, none `needs-review`; ideas 1, 2 and 7 produced no relevant hit and are listed for a deeper search before any freeze. Not legal advice.

## 0.4.81 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Spec (Classify, **normative**, PM ruling C on the D7 sidecar extra-fields ask): KISS-CLASSIFY-6.3-0012 gains one MUST, that layouts differing in a fact the clause does not cover MUST get distinct `encoding` codes, plus an informative paragraph listing the four groups Fuel reported (scale of the scale, zero-point, nibble order/pack axis/permutation, block shape). No field, no table row, no `structure_key` byte changes. The MUST falls on the producer/registry and the Classify harness cannot test it; the informative list is unchecked against the external registry, which is not in this repository.

## 0.4.80 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: `docs/call-for-outside-readers.md` (a community-post draft asking for foreign readers, what they need, what to do, how to respond publicly) and the `Outside reader report` issue template; ROADMAP R3.11 records the `build_and_run.bat` MSVC 14.29 pin. Nothing is sent to anyone. Numbered 0.4.80 because #669 proposes 0.4.79.

## 0.4.79 — 2026-10-25 (number proposed; the PM allocates at gate time)

- R3.11 KISS#6, single-source the splitmix corpus generator. The CUDA-side copy lives once in `cuda/splitmix64.cuh`; `fmax_ieee.cu` and `generated_relu_add_diff.cu` include it, and `differential.rs` exposes the three constants. `tests/splitmix_single_source.rs` pins the header to the Rust constants and fails if a `.cu` re-declares the generator (mutation-checked). The `edge_f32` list is still hand-copied. No spec or wire change.

## 0.4.78 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: board 279 pair 5 options note, `docs/options-r5-18-contract-6-8-0012-wording.md` (R5.18, the stale "deferred non-contraction coordinate" wording in KISS-CONTRACT-6.8-0012). No normative text.

## 0.4.77 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: board 279 pair 4 options notes, `docs/options-r5-10-sk4-deferred-scope.md` (R5.10) and `docs/options-r5-16-positioning-governance.md` (R5.16); also corrects the R5.7 note on the D.5 ULP ceilings (it cites `PRIOR-ART.md` 5.2 and makes the model question first). No normative text.

## 0.4.76 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: board 279 pair 3 options notes, `docs/options-r5-7-ops-open-pins.md` (R5.7, which KISS-Ops Appendix D pins to close now) and `docs/options-r5-8-certification-sla.md` (R5.8 SLA item, what free steward certification promises); no normative text.

## 0.4.75 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: board 279 pair 2 options notes, `docs/options-r5-9-roundtrip-guarantee.md` (R5.9, can the emit/consume round-trip guarantee be structural) and `docs/options-r5-2-artifact-format-registry.md` (R5.2, who owns the `artifact_format_tag` registry), each with 2-3 options, what it touches and a recommendation; no normative text. Numbered 0.4.75 because #664 proposes 0.4.74.

## 0.4.74 — 2026-10-25 (number proposed; the PM allocates at gate time)

- R3.11 KISS#158, test gating only; no spec, corpus or wire change. `harness/msvc.rs`'s `mod tests` (`compiles_a_trivial_c_kernel_to_a_dll`, `the_override_compiler_is_the_one_that_runs`) is now `#[cfg(all(test, windows))]`. `find_msvc()` cannot find MSVC off Windows, so on every other leg the first test ran, declined through `runtime_gate_some!`, and reported `ok` having asserted nothing; the second returned early. They now do not exist there (visible to `--list` and the gate-only reporting, KISS-CONFORM-6.1-0009a). Verified: both pass on Windows; `cargo check --lib --tests --target aarch64-apple-darwin` builds with no unused-import warning. Not run: a Linux `cargo test`. Other tests of the same shape may exist: the issue names no detector, and none is built here. Numbered 0.4.74 because #663 (0.4.73) merged.

## 0.4.73 — 2026-10-25 (number proposed; the PM allocates at gate time)

- Docs only: two options notes for CireSnave's board 279 (pair 1 of 5): `docs/options-r5-1-message-boundary.md` (R5.1, no message boundary on a stream) and `docs/options-r5-3-trust-boundary.md` (R5.3, trust-boundary and integrity statements), each with 2-3 options, the clauses they touch and a recommendation; no normative text. ROADMAP R3.7 records CireSnave's acceptance of the 6.14 wording (board 278, 2026-10-25). HANDOFF refreshed (R3.10 done, issue sweep, board 279). Numbered 0.4.73 because #662 (0.4.72) merged.

## 0.4.72 — 2026-10-25 (number proposed; the PM allocates at gate time)

- KISS#240 / #239 item 3, vectors only, no normative text: three positive vectors that each differ from `binary_two_operands` in ONE token field, retiring three of the four `unauthored` rows of `UNDISCRIMINATED.tsv`: `binary_two_operands_i32` (dtype; i32 is the same width as f32, so `v4` holds), `binary_two_operands_ix64` (index_width) and `binary_two_operands_warp` (schedule). `structure_key_vectors.json` regenerated (28 to 31 positive vectors); pinned dtype coverage 4 to 5 in the dtype position and 7 to 8 anywhere. **Reduce stays `unauthored`, with a sharper reason**: the output operand's keepdim mask records the reduced axes (6.6-0009), so reduce may not vary independently of operands; either a legal pair is found or the row becomes `undiscriminated` with that proof. The three vectors are classify-legal by construction but have no second deriver yet. Numbered 0.4.72 because #661 (0.4.71) merged.

## 0.4.71 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Tool and ledger only, no normative text: `tools/kiss_pairs.py`, the mechanical pair checker behind KISS-CONFORM-6.14 (KISS#240, #239 item 3). Over `structure_key_vectors.json` (28 positive vectors) it finds, per token field, the vector pairs that differ in that field alone: cell_class 2, target 3, rank 1, operands 6, contraction 10. **Five fields have none**: schema (single-valued by construction, `undiscriminated`), and dtype, index_width (all 28 are `ix32`), schedule and reduce, recorded as `unauthored` debt in `conformance/UNDISCRIMINATED.tsv`. The check fails a field with no pair and no ledger row, and a row that outlives its gap, so the debt can be named once and cannot grow; authoring the four pairs retires it. Controls `tools/test_kiss_pairs.py`, wired into `traceability.yml`. The four 6.14 clauses stay `untested` in `UNBACKED.tsv` (the floor is unmoved): this tool covers one vector file and token fields only. Numbered 0.4.71 because #660 (0.4.70) merged.

## 0.4.70 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Comments only, KISS#147: 15 doc-comment lines in `conformance/src/{dtype,fp,integer}.rs` respelled from the retired `s4`/`e5m2` to `i4`/`f8e5m2`. No code, no behaviour. Numbered 0.4.70 because #659 (0.4.69) merged first.

## 0.4.69 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Spec text, KISS#379 (PM ruling A, 2026-10-24): KISS-Consume §8.1 now states the schema version's value, **1 by construction**. It records existing behaviour; no behaviour changes. Closes the uncomplyable half of KISS-CONSUME-8.1-0001(b). Numbered 0.4.69 because #658 proposes 0.4.68.

## 0.4.68 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Docs only: the portfolio PM's spec-to-roadmap audit (CireSnave's idea, 2026-10-24). `ROADMAP.md` gains R5.1-R5.18, the work that design docs, rfcs, July plans and the nine specs record but no row tracked, each with its source and state (not started). R3.7 and R3.8 are corrected for #646, #647, #653-#655 and #656. `docs/glossary.md` now states the availability record as the identity triple and the `sk6` prefix, as Announce §3 and Classify 6.4-0003 do. R5.11 records the Unpopped N2 NaN ruling (option (a), later, no text now).

## 0.4.67 — 2026-10-24 (number proposed; the PM allocates at gate time)

- KISS#387 option (a), board 271 ("Still (a)"), tool and ledger only, no normative text: a per-conjunct trace behind umbrella §5.3 condition 3. `conformance/CONJUNCTS.tsv` lists a multi-conjunct clause's conjuncts; a test discharges one with `// Backs: KISS-X-n-nnnn [label]`; `kiss_trace.py --freeze-ready` fails a listed clause while any label is cited by no test, and the default run fails a ledger row or citation naming no live clause or label. Seeded with KISS-CONFORM-6.8-0005 (4 conjuncts, all cited) and KISS-OPS-6.18-0017 (4; `zero-sign-re` is cited by no test, because `test_ops_complex_branch_sign_exact` exercises the imaginary zero only: the complementary pair #387 describes, now a named gap). The ledger proves each LISTED conjunct is cited, not that the list is complete nor that the citing test discriminates. New controls `tools/test_kiss_conjuncts.py`, wired into `traceability.yml`.

## 0.4.66 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Spec text, KISS#240 / board 271 item 1 (CireSnave: "1: A"): a fifth Conform modality, **discriminating vector pairs**, as new §6.14 with KISS-CONFORM-6.14-0001..0004. For each clause that pins a coordinate the vector set must hold a pair of otherwise identical legal inputs differing in that coordinate alone whose required outputs differ (-0001); where none can be built the row records why and the coordinate is reported undiscriminated (-0002); existence is decided mechanically over the vectors, never by count of positives or a real-world corpus (-0003); a missing pair fails the build at authoring time (-0004). Enumerations of "four modalities" now say five (front matter, §1, §2.4, §7.1-0001, §9, A.5, README, umbrella). Four clauses arrive with no test: `UNBACKED.tsv` +4 `untested`, floor untested 387 to 391 (an arrival, not a loss; harness 459 and lint 44 unmoved), README bound figures 961/502/384 to 965/506/388. Sidecar +4 ids. The pair checker (KISS#240, #239 item 3) is built on this text in a later PR. Numbered 0.4.66 because #655 proposes 0.4.65.

## 0.4.65 — 2026-10-24 (number proposed; the PM allocates at gate time)

- KISS#136 step 3 of 3, tool: `kiss_trace` now fails the build on an orphan test (KISS-CONFORM-6.1-0003): a test that cites no live clause, is not named by a §9 row and carries no `// harness-internal: <reason>` tag, or a tag with no reason, or a tag on a test that cites or is named by a clause. It prints each by name. The README-bound `uncited_tests` line is kept (225 tests, all 225 tagged, 0 orphans at `f52185b`). New controls `tools/test_kiss_trace_orphans.py` (9 cases, wired into `traceability.yml`); a planted untagged test turned the run red (exit 1, named) and removing it restored exit 0. No spec or wire change; floor unmoved (459/44/387). Closes KISS#136.

## 0.4.64 — 2026-10-24 (number proposed; the PM allocates at gate time)

- KISS#136 step 2 of 3, comments and README figures only: the other 185 tests that cited no clause (measured at `f286c6a`) are triaged one by one. 74 carry new `// Backs:` lines (89 lines in all) (classify `structure_key` token declines, OPS 6.11 structural ops, 6.15 min/max, OpAttrs golden/value vectors, Announce, Contract section schemas, CONFORM 6.8 comparator and NaN provenance) and 111 carry `// harness-internal: <reason>`. The harness-internal tag has been given a real reason in every case; claims the audit found weak were downgraded to a tag (a one-ordinal-per-enum spot check does not back OPS-6.19-0014..0021; CONFORM-6.5-0003 is ruled out of tests, board 203). With the gate armed in a trial run, 0 untagged orphans and 0 misused tags remain. README `named` 232 to 216 and `uncited_tests` 301 to 225 (bound figures). `kiss_trace` floor unmoved: harness 459, lint 44, untested 387. The gate itself (orphan fails the build) is step 3.

## 0.4.63 — 2026-10-24 (number proposed; the PM allocates at gate time)

- KISS#136 step 1 of 3, comments only: 116 of the 301 tests that cite no clause (measured at `f286c6a`) are tests of the suite's own machinery (`hp_*`/`round_ziv` oracle-library units, the JSON parser, `harness/*`) and now carry `// harness-internal: <reason>` per KISS-CONFORM-6.1-0003. 185 remain: most look like real conformance tests that cite nothing and need a clause (a `Backs:` claim) rather than a tag, so they are triaged by file in the next PRs before the gate is armed. No tool change; `kiss_trace` floor unmoved (459/44/387).

## 0.4.62 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Spec text only, KISS#136 option A (CireSnave, board 271): KISS-CONFORM-6.1-0003 gains a carve-out for a harness-internal test, tagged `// harness-internal: <reason>`; an untagged test citing no live clause stays an orphan, and a tagged test that also cites a clause is a violation. No tool change in this PR; the gate that enforces it follows separately. Numbered 0.4.62 because #651 proposes 0.4.61.

## 0.4.61 — 2026-10-24 (number proposed; the PM allocates at gate time)

- ROADMAP R3.13 marked done, docs only: `tools/run_with_git_bash.py` already makes the three bash-launching suites run on Windows with WSL; re-run green at `fb39fdf`.

## 0.4.60 — 2026-10-24 (number proposed; the PM allocates at gate time)

- HANDOFF rewritten to current state (R3.8 progress, #136 waiting on CireSnave). Docs only.

## 0.4.59 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.8 KISS#498, docs only: `docs/design-498-two-level-backing.md`. Reading `kiss_trace.py` at `3e994b0` shows citations are already many-to-one (`cited` is clause to a set of tests); only the §9 pointer is one-to-one, so the defect is the `tier_cited_other` report label (20 clauses) conflating a stale pointer with a second test adding a level. Recommends a report-only split after measuring those 20, and a level qualifier on `Backs:` only if a second instance appears. No tool, spec, test or wire change; floor unmoved. Numbered 0.4.59 because #647 proposes 0.4.58.

## 0.4.58 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.8 KISS#159, tooling only; no spec, corpus, wire or floor change. Re-read at `origin/main` 3e994b0: the 6 `if sm and hm` sites in `kiss_wire.py` and the 2 in `kiss_vocab.py` the issue names already have an `else` that appends a violation ("not found on both sides"), so those no longer skip silently. The two that did are the count-word comparisons in `kiss_tables.py` (`§6.17-0001 two-member`, `§6.4-0001 exactly the four categories`): a missing phrase or a word outside the lint's table (`words.get(..) not in (None, n)`) switched the check off with no output. Each is now a violation. `tools/test_kiss_tables.py` gains 6 controls; with the fix reverted 4 fail (the wrong-word and clean-spec controls pass either way, by design). Not done: a per-group minimum on the printed `kiss_wire` counts (the miss is already reported, so the count backstop would only repeat it).

## 0.4.57 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.8 KISS#137, tooling plus two test renames; no spec, corpus or wire change, floor unmoved. `kiss_trace.discover_tests` keys by bare fn name, so a same-named `#[test]` in two files silently replaced the first and its citations. It now reports each repeat and `kiss_trace.py` fails with `DUPLICATE TEST NAME`. Measured at `origin/main` 9b746f6 by an independent walk that records every occurrence: 811 `#[test]` fns, 809 names, 2 collisions (`oracle_is_self_consistent_under_fuzz`, `corpus_is_reproducible`, in `differential.rs` and `integer_differential.rs`); the integer copies are renamed `int_*`, no clause cites either name. Control: with the renames reverted the tool prints both and exits `VIOLATIONS FOUND`. `tools/test_kiss_trace_duplicates.py` (2 controls) is wired into the `kiss-trace` job. #136, #498 and #159 of R3.8 remain open.

## 0.4.56 — 2026-10-24 (number proposed; the PM allocates at gate time)

- ROADMAP R3.6 records #148 and #400 (loud half) done and #484 as process, not code; HANDOFF rewritten to current state (it was over its 8KB ceiling). Docs only; no spec, test, src or wire change.

## 0.4.55 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.6 KISS#148, tooling only, no spec, test, corpus or wire change; floor unmoved. New `tools/kiss_io_discard.py`: a lint for a filesystem read (`read_dir`, `read_to_string`, `fs::read(`, `File::open`) whose `Result` is discarded in the same statement by `.ok()`, `.flatten()`, `unwrap_or_default()` or `let Ok(..) =`, the short form KISS-CONFORM-6.5-0016 forbids. The line is drawn at the I/O call, not at a "discovery context" (that boundary is not textual); a deliberate probe carries `// io-discard-ok: <reason>`. Measured at `origin/main` 3ecc83b, enumerated from `git ls-files` (172 Rust files): 3 statements, all in `conformance/src/harness/msvc.rs` (a toolchain probe where `find_msvc()` returns `None` and the callers skip through a declared runtime gate); each now carries a reason comment, no behaviour change. 0 other instances in `conformance/` or `tools/`; the instances #148 was filed for were already fixed. It cannot see an error swallowed across a function boundary, `let _ =`, an ignored `Err` arm, or non-filesystem I/O; a clean run does not say the code propagates its I/O errors. `tools/test_kiss_io_discard.py` has 15 controls (each discard form flagged, each propagating form clean, the exemption honoured and refused without a reason); both wired into the `kiss-trace` job. Does not back a ledger row, so the floor is unmoved.

## 0.4.54 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.6 KISS#400 option 3, tooling only, no spec, src, corpus or wire change; floor unmoved. A `*Test:*` tag followed by a name list (`` `a`, `b` ``, also `;` `+` `&` `/` `and` `or`) bound only its first name and nothing reported it, so a clause read as naming two tests while the tracer recorded one. `kiss_trace` now reports it as a violation. Measured at `origin/main` 4c4d890: 961 `*Test:*` tags over the 12 spec files, 0 list continuations, so the check is clean on the corpus and fires on a fixture (`tools/test_kiss_trace_partial_bind.py`, born-red: 9 failures without the check, the two single-name controls pass either way). It does not give a clause two tests; options 1 and 2 of #400 (a per-conjunct model) are not taken. The new test is wired into the `kiss-trace` job.

## 0.4.53 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.7 KISS#239 items 1 and 2, tools tests only, no spec, src, corpus or wire change; floor unmoved. `tools/test_kiss_dtypes_manifest.py` pinned `storage_bits` for 4 of 24 dtype arms and `kind` for 2; it now pins `(kind, storage_bits, reserved)` for all 24 against a table transcribed from the spec, and the token count per width (the 8-bit cluster is nine). A control applies five same-width mutations and an `i16`/`u16` kind exchange to a copy of the real manifest and requires each be reported; a mutation of the normative §6.1 table row for `i16` was caught by both new tests. Item 3 (a discriminating swap vector in the corpus) is not done: it needs a vector home in KISS-Conform.

## 0.4.52 — 2026-10-24 (number proposed; the PM allocates at gate time)

- ROADMAP R3.14 recorded as ruled: CireSnave 2026-10-24 (board 269) chose (b) now, (a) later, when the first foreign reader exists. Also logs ROADMAP R3.5a (KISS#352 residue: no ULP-class moved-NaN vector; 0 of 78 moved rows). Docs only; no spec, test, src or wire change.

## 0.4.51 — 2026-10-24 (number proposed; the PM allocates at gate time)

- HANDOFF only: R3.2, R3.3 and R3.9 are done, so the next-actions list now names the unblocked rows (R3.5, R3.7, R3.6, OPS-6.18-0015) and what waits (R3.14 on CireSnave). No spec, test or wire change; floor unmoved at harness 459 / untested 387.

## 0.4.50 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 KISS#313, tests only, no spec, src or wire change; floor unmoved at harness 459 / untested 387. `test_conform_emit_consume_correspondence_lint` compares Emit against Consume, so a mandate re-added to BOTH twins (Emit §6.7-0004 and Consume §6.6-0004) stays green while each contradicts its own document's §-0002. New `test_conform_roundtrip_no_comparator_mandate_remains` is the issue's cheap first cut, stated as a TRIPWIRE for one known phrase and not a coherence check: across Emit §6.7-0001..0005 + -0009 and Consume §6.6-0001..0006, "comparator" may appear only as a quoted citation in a sentence that says it was excised. Re-adding the mandate to both twins keeps the correspondence lint green and reddens this one. The general within-document coherence check the issue also proposes is not built.

## 0.4.49 — 2026-10-24 (number proposed; the PM allocates at gate time)

- **De-crediting** (PM ruling 2026-10-24: every claim earned; harness 460 -> 459, untested 386 -> 387, lint unmoved): `KISS-CONSUME-6.4-0007` (category assignment is structure-based) no longer holds a credit. Its named test was a document lint with no consumer to observe, so the credit did not measure the clause's obligation. The `*Test:*` pointer in `spec/consume.md` (clause and §9 row; pointer text only, no normative sentence changes) now names the owed differential `test_consume_taxonomy_structure_based_differential`; the ledger row is `decredited` (a real test is still OWED, ROADMAP R3.14, needs CireSnave's ruling); the old test is renamed `test_consume_taxonomy_doc_lint_four_categories` and its scope comment says the credit was withdrawn. Reported by the ratchet as an honest de-crediting, not a regression; README bound figures follow.

## 0.4.48 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 KISS#378 option (b) (PM ruling 2026-10-24), comment and ROADMAP only; no spec, src, test-logic or wire change, floor unmoved at harness 460 / untested 386. `test_consume_taxonomy_structure_based` is a document lint credited with an implementational obligation (KISS-CONSUME-6.4-0007, no lifter in the repo). Its doc comment now says so and names the gap as an instance of KISS#458. New ROADMAP row R3.14 records option (a), a `consume_categorization_vectors.json` differential with a reference structural categorizer, as needing CireSnave's ruling because it mints a freeze surface and a source-neutral input format. Not done, and why: de-crediting the clause (precedent KISS#187) would mean changing the `*Test:*` pointer in the Consume spec so the name stops resolving, which moves the floor and edits spec text, not a ledger note; offered to the PM as the alternative.

## 0.4.47 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 KISS#385, tests only, no spec, src or wire change; floor unmoved at harness 460 / untested 386. `test_synth_decline_taxonomy_complete` (KISS-SYNTH-6.6-0003f) enumerated the crate side from a hard-coded six-entry array, so a seventh `pub const` in `announce::decline_code` passed the whole suite (the comment claimed otherwise). It now also reads the constants the SOURCE declares inside `pub mod decline_code` and requires them to be exactly the six; the issue's own mutation (`PROVISION_TIMEOUT = 0x7`) reddens it, and a control shows the reader counts a seventh on a synthetic module. Of the issue's options this takes (ii), a source-level check; (i), an enum with an exhaustive match, would change the public `decline_code` API and is not taken. It cites the clause (NAMED -> CITED, README `named` 234 -> 233).

## 0.4.46 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 KISS#376, tests only, no spec, src or wire change; floor unmoved at harness 460 / untested 386. `test_classify_target_fixed_width_juxtaposition` (KISS-CLASSIFY-6.8-0006) validated constants it declared itself, so no change to the spec or the code could redden it. It now reads the `vulkan:sg64.ops-….arith-…` fields from `classify.md` Appendix A and `spec/namespaces/vulkan.md` (which must agree) and runs the fixed-width and delimiter checks on those; controls show the checks reject a non-single-byte juxtaposed member, an unsorted field, an undelimited variable-width field and a same-width delimited one. The issue's own mutation (`ops-abclqrstvw` -> `ops-abclqrstvwXY9` in the spec) now reddens it. It cites the clause (NAMED -> CITED, README `named` 235 -> 234). Design call taken: parse the spec in the harness, rather than move the check to `kiss_vocab.py` or pin the transcription.

## 0.4.45 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 KISS#381, tests only (comments), no spec, src or wire change; floor unmoved at harness 460 / untested 386. KISS-EMIT-9.2-0001 puts bidirectional traceability in the conformance definition, and none of the 28 Emit named tests cited a clause. Of the issue's three readings this takes (ii): every one of the 28 gains `Enforces: KISS-EMIT-<id>` in its doc comment (`test_emit_spellable_is_not_computable` had no doc comment and gets one), so NAMED 264 -> 235 and CITED rises by the same. Readings (i) and (iii), which would soften the clause, are not taken. `test_emit_traceability_complete` now states its scope: it checks the document against itself, no instrument checks that each test cites its clause, and `kiss_cite_form` polices only the spelling.

## 0.4.44 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 batch 2, no spec or wire change; floor unmoved at harness 460 / untested 386. Five tests that could not fail now can: KISS#380 (CONTRACT-6.6-0007: `validate` and the test iterate one `GEOMETRY_FIELDS` list, every field is blanked and a destructure fails to compile if the struct grows; `DispatchFields::{field, set_field}` added), KISS#382 (`gelu` added to the anti-fork VOCAB so `gelu_fast` is a prefix near-miss), KISS#383 (f8e5m2 negative finite overflow saturates to 0xFB, never -inf), KISS#384 (new `test_ops_opattrs_framer_does_not_parse_inside`: blobs no schema accepts are framed verbatim), KISS#386 (ANNOUNCE-6.3-0012: all three MBZ offsets decline).

## 0.4.43 — 2026-10-24 (number proposed; the PM allocates at gate time)

- R3.9 KISS#419 (KISS-OPS-6.19-0037), no spec or wire change; floor unmoved at harness 460 / untested 386. Arm (4): new `opattrs::decode_window_param_vector`, the reader the clause requires (a count above `MAX_RANK` is a typed `FieldOutOfRange`, a short blob is `TruncatedBlob`); before, the only guard was an encoder `assert!(len <= 8)` that nothing exercised, so the bound could be raised to 99 with no test red. Arm (3): the existing `validate_reduce_axes` already declines a mask bit at position >= `MAX_RANK` (0x0100 and up); it is now asserted in the named test. The three hard-coded `8`s in `opattrs.rs` (rank, scan, window) now reference `MAX_RANK`, so the co-versioning obligation moves them together.

## 0.4.42 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Tests only, no spec, src or wire change; floor unmoved at harness 460 / untested 386 (R3.9, tests that cannot fail). KISS#490: `test_contract_reserved_tile_mapping` had a control `GeometryAgnostic.validate().is_ok()` that was `Ok(()) == Ok(())`; it is replaced by a document-level claim (the sentinel Dispatch block parses to `GeometryAgnostic`, and the same block with a reserved `thread_mapping = tile_mapping` beside it declines). KISS#377: new `test_conform_split_comparator_not_fourth_class` reads the `DeterminismClass` enum and requires exactly its three canonical members, so a deliberate fourth registration (with its match arms) reddens a test instead of passing on compiler exhaustiveness alone; it cites KISS-CONFORM-6.8-0005 (NAMED -> CITED).

## 0.4.41 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Complex-op oracle (R3.3 per the PM's order (c)), no spec or wire change. `semantics.rs` gains `cmake`/`cre`/`cim`, `cmul` (Annex G recovery on the spec's verbatim trigger), `carg`, `cexp`, `clog`, `csqrt`, `cpow`, with the Annex G rows that govern over the naive decomposition; `tests/semantics_complex.rs` backs nine KISS-OPS clauses (6.18-0003, -0005, -0007 to -0013): harness 451 -> 460, untested 395 -> 386, lint unmoved. Every expected value is transcribed from the clause rows, and each op is paired with its naive decomposition to show where Annex G governs. 6.18-0015 stays unbacked (c64/f32 only). Corrects the 0.4.35 ledger reasons: they said `conformance/` has no op evaluator, but `semantics.rs` already held `cadd`..`cdiv`; the 6.18-0015 row now says what the oracle does and does not cover.

## 0.4.40 — 2026-10-24 (number proposed; the PM allocates at gate time)

- **Correction** (PM ruling 2026-10-24: fix the example, not the clauses): the Appendix C worked contract violated KISS-CONTRACT-6.7-0002 and 6.9-0003, which `check_record_consistency` (0.4.39) made detectable. Appendix C and its golden document now conform, with no normative clause change: the Identity and Capabilities `accept_predicate` are both the real strided `ew2` cell token `sk6|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;st/00/v1/d16/f;co/00/v4/d16/f|-` (the old Identity value `ew2/f32,f32,f32/strided/cuda:sm89` and Capabilities value `sk6|ew2|f32` were neither the same nor a valid `structure_key`; the target component now equals `target_capability`), and the Provenance `revision_hash` is `4:deadbeef`, the Identity value. Regenerated `corpus/contract_vectors.json` (golden bytes, `len`/`crc32`, and the single-fault declines derived from it); `contract_section_schemas.rs` fixtures follow. A foreign reader that pinned the old golden bytes must re-pin; `contract_version` and the wire are unmoved. New regression test `appendix_c_golden_document_is_record_consistent`.

## 0.4.39 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Reference record-consistency validator `contract::check_record_consistency` (R3.2/R3.3 per the PM's order (b)), no spec or wire change. Backs four Contract clauses (6.3-0009, 6.7-0002, 6.9-0003, 6.9-0006): harness 447 -> 451, untested 399 -> 395, lint unmoved. New: `IDENTITY_FIELDS`, `GUARANTEES_FIELDS`, `parse_identity_block`, `parse_guarantees_block`, `RecordDecline`. Finding, not fixed here: the Appendix C golden document does NOT satisfy 6.7-0002 (Identity accept_predicate `ew2/f32,f32,f32/strided/cuda:sm89`, Capabilities `sk6|ew2|f32`) nor 6.9-0003 (Identity `4:deadbeef`, Provenance `abc`); changing the golden is a frozen-surface decision for the PM.

## 0.4.38 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Reference codec for the KISS-Synth provision-success frame `PRSP` (`conformance/src/synth.rs`, R3.2 per the PM's go), no spec or wire change. Backs eight Synth clauses (6.4-0001, -0001b, -0001c, -0001d, 6.2-0003, -0003a, -0003b, -0004): harness 439 -> 447, untested 407 -> 399, lint unmoved. `announce::Identity::{encode_into, decode_at}` become `pub(crate)` so the PRSP echo reuses Announce's identity block. Not bound (provider or contract-Identity halves have no subject): 6.4-0001a, 6.3-0006/-0006a, 6.2-0006.

## 0.4.37 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Ledger and HANDOFF only, no spec, test or wire change (R3.2, floor unmoved at harness 439 / untested 407): records a reason for all 96 bare-`untested` Synth rows. `conformance/` has the reused Announce CYRQ/CDEC codec but no PRSP codec, provider or consumer. Every sub-standard now has no empty-reason `untested` row.

## 0.4.36 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Ledger and HANDOFF only, no spec, test or wire change (R3.2, floor unmoved at harness 439 / untested 407): records a reason for all 69 bare-`untested` Contract rows. `conformance/` reads the document framing and the key set of three section blocks, but has no value reader, cross-field validator, resolver or contract producer. Contract now has no empty-reason row.

## 0.4.35 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Ledger and HANDOFF only, no spec, test or wire change (R3.2, floor unmoved at harness 439 / untested 407): records a reason for all 34 bare-`untested` Ops rows (complex-op semantics, op registry and version governance, bitwise decline, IEEE conformance and others). Most lack a subject: `conformance/` has no op evaluator. Ops now has no empty-reason row.

## 0.4.34 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Ledger and HANDOFF only, no spec, test or wire change (R3.2, floor unmoved at harness 439 / untested 407): records a reason for all 35 bare-`untested` Emit rows. Most lack a subject: `conformance/` has no emitter or lifter. Emit now has no empty-reason row.

## 0.4.33 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Ledger and HANDOFF only, no spec, test or wire change (R3.2, floor unmoved at harness 439 / untested 407): records a reason for all 28 bare-`untested` Consume rows. Most lack a subject: `conformance/` has no lifter or emitter. Consume now has no empty-reason row.

## 0.4.32 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Ledger only, no spec, test or wire change (R3.2, floor unmoved at harness 439 / untested 407): records a reason for the 20 remaining bare-`untested` Announce rows (6.0-0001, 6.2-0010, 6.3-0002/-0006/-0007, 6.4-0002/-0003/-0005/-0008/-0010/-0012, 7.1-0003/-0004, 7.2-0001/-0002/-0004/-0005/-0006/-0010, 7.3-0001/-0002; 6.1-0005 got its reason in 0.4.30). Announce now has no empty-reason row; most lack a subject because `announce.rs` is a byte codec with no provider, consumer, role or capability-bit table. Also keeps the `# 0.9.14` comment on the CodeRipper portfolio pin.

## 0.4.31 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Docs only, no spec, tooling or wire change: `conformance/corpus/PROVENANCE.md` records the generator, version and regeneration command for `ops-int-wrap.json` (and the other ten `kiss_mint` bundles); ROADMAP R3.13 records the Windows-only failure of `tools/test_kiss_msrv_step.py`.

- Note for consumers: 0.4.29 (#618) changed **normative text** — KISS-CONTRACT-6.5-0004a, 6.6-0006, 6.7-0006, 6.11-0008 (the launch-scalar symbol vocabulary of a contract); a re-vendor of `spec/` and `conformance/corpus/` is available. `kiss-conformance` is `publish = false`.

## 0.4.30 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Tests and ledger only, no spec or wire change (R3.2): `announce_frames.rs` gains producer-side tests for KISS-ANNOUNCE-6.1-0009, -0014 and -0015, read from the raw encoded bytes (harness 436 -> 439, untested 410 -> 407). 6.1-0005 stays untested with a recorded reason (the version a producer writes is a caller-supplied field, so a test would be circular). The advisory CodeRipper portfolio caller is repinned to coderipper d14a22b (CodeRipper 0.9.14, was 0.9.6; coderipper#142).


## 0.4.29 — 2026-10-24 (number proposed; the PM allocates at gate time)

- Spec text and reference-evaluator change, no wire change (KISS#410, R1.14): `spec/contract.md` §6.5-0004a defines **the launch-scalar symbol vocabulary of a contract** as the symbols its own Interface `positional_signature` declares (not the whole class table), and §6.6-0006, §6.7-0006 and §6.11-0008 now say "of this contract" and use one spelling. The reference `LaunchScalars.ws_bytes` becomes `Option<i64>` so an undeclared workspace declines `UnknownSymbol` instead of reading as `0` (**breaking for a caller constructing `LaunchScalars`**; the crate is pre-1.0). A new test pins that the same expression is accepted where its class is declared and declines where it is not.

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
