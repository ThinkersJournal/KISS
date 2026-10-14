# Changelog

`kiss-conformance` (the reference crate and the spec it binds). Pre-1.0, a breaking change moves the
second number.

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
