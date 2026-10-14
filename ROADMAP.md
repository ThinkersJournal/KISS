# KISS roadmap

**Statuses refreshed at `origin/main` `9ea8c05` (conformance 0.2.14), 2026-10-14; the measured tables below were taken at `5b28bf5` (0.2.1) unless a row says otherwise.** Not a plan of record until the
PM gates it. Order is dependency order, not calendar: nothing here has a date, because the gates below are
not ours to date.

How to read it: **[go]** = unblocked and ours; **[rule]** = needs a ruling (CireSnave via the PM) before
anyone writes a line; **[wait: X]** = blocked on X; **[ext]** = needs people outside this repo's lanes.
Every figure carries the command that produces it. Claims about a consumer cite the file they came from
(read at that repo's `origin/main` by a research pass on 2026-10-13; not re-run).

## 1. What "reached" means

KISS is nine sub-standards (Announce, Classify, Consume, Contract, Conform, Emit, Grammar, Ops, Synth) plus an
informative umbrella. A sub-standard is **Frozen** only when umbrella §5.3's three conditions hold:

1. **two structurally dissimilar parties independently derive each golden field** (per field, not per
   implementation; passthrough does not count);
2. **a non-native foreign reader** consumes the wire;
3. **its KISS-Conform suite exists, passes, and traces every clause both ways** — and the AUDIT role signs.

CireSnave's ruling E6 stands: the siblings (Baracuda, kiss-ref, Fuel, Vulkane) trace to one interpretation,
so they are a partial mitigation, **not** a clearing; freeze needs genuine external implementors. Therefore
**no item in this file, completed in full, freezes anything by itself** — the work below makes freezing
*possible and cheap to demonstrate*; conditions 1(external) and 2 are `[ext]`.

## 2. Where it stands

| Measure | Value | Command / source |
|---|---|---|
| normative clauses | 957 | `python tools/kiss_trace.py` |
| backed by a harness test / lint-enforced | 398 / 34 | same; floor in `conformance/COVERAGE_FLOOR.tsv` |
| genuinely untested | 479 | same |
| §5.3 condition 3, per sub-standard (traced of total) | Classify 81/120 · Ops 114/202 · Announce 42/76 · Contract 40/114 · Consume 21/49 · Emit 28/66 · Grammar 37/74 · Conform 38/126 · Synth 31/130 — **0 of 9** | `python tools/kiss_trace.py --freeze-ready` |
| schema | `structure_key` `sk5`, Contract `contract_version` 3, Announce `list_version` 1 | KISS #535, #536 |
| reference implementation | kiss-ref 0.4.0 on crates.io, bound to KISS `a33282c` | kiss-ref #52 |
| open issues | 139 (`gh issue list --state open --limit 500 --json number --jq length`) — most are instrument/test-gap findings, not consumer asks | |

Each consumer's binding today:

| Consumer | Bound to | Source |
|---|---|---|
| **Fuel** | `sk5` deriver, `contract_version` 3, KISS `5b28bf51`; kiss-ref 0.4; `op_manifest.json` + `ops-*` corpus still vendored from `f4952b4c`; six newer corpus files not vendored | fuel `CHANGELOG.md` v0.16.0, `fuel-dispatch/fixtures/kiss-corpus/PROVENANCE.md` |
| **Baracuda** | still **`sk4`** keys (emitted and asserted); `cuda:` seed pinned at KISS `5b28bf51`; kiss-ref/unpopped followers 0.4.0/0.16.0 | baracuda `crates/baracuda-cuda-emit/tests/link.rs:24`, `kiss-cuda-seed.tsv` |
| **Unpopped** | 0.16.0 merged (kiss-ref 0.4.0); vendored corpus from `bc16715` (`sk4` manifest) — keyer re-derivation to `sk5` follows the PM's task | unpopped `crates/unpopped-vocab/kiss/README.md` |
| **kiss-ref** | KISS `a33282c`; models no `structure_key`, no OpAttrs bytes, no Contract, no `MathFidelity` type, no native Bool lane | kiss-ref `CHANGELOG.md` |
| **lightbulb** | dev-dependency `kiss-ref-core`/`kiss-ops-vocab` **0.2.0** (differential target only); consumes no other sub-standard | lightbulb `Cargo.toml`, `tests/kissref_differential.rs` |

## 3. The roadmap

### R0 — make the record true (days; ours)

| # | Item | Why / who needs it | State |
|---|---|---|---|
| R0.1 | ~~Refresh `HANDOFF.md`~~ **done (#538); refreshed again with this change.** | every restarted KISS session reads it first | **done** |
| R0.2 | **Close the issues the merged work answered** (#263, #520, #477 at least; confirm #298, #305, #219, #364 against `origin/main` before closing, one at a time). | the board reads open issues as open work; consumers cite them (#520 is quoted in Unpopped's CHANGELOG as "raised with the PM") | **ready list below**; closing is CireSnave's, not the lane's |
| R0.3 | ~~Answer Fuel GAP-349~~ **done**: answered from §6.1-0001/-0013; Fuel recorded it. | Fuel `docs/gaps.md` GAP-349 | **done** |
| R0.4 | **Re-measure the differentials.** Partly done as the *derivation* runs: Fuel (#358/GAP-354: 25 pass, 10 anchors byte-match) and Baracuda (19 pass / 8 fail at unpopped-vocab 0.16.0, sk4) reported against `structure_key_derivation_vectors.json`; the token-vector runs at current main are Fuel's. The four-way sk5 table needs Baracuda on the unpopped sk5 keyer (GAP-350). | Fuel, Unpopped, Baracuda | **[wait: Unpopped sk5 keyer]** |

### R1 — finish what the `sk5` event left open (spec-side, consumer-visible)

**R1.1, R1.2, R1.5 and the freeze-signer question (#462) are boarded for CireSnave as one item (PM board 193, 2026-10-14); none is started.**

| # | Item | Needed by | State |
|---|---|---|---|
| R1.1 | **Announce: op identity in every lookup** — availability record, contract query and echo carry `op_identity`; `list_version` 2; Synth + corpus follow. Design is written (`docs/design-263-cell-class.md` §6). Without it a coarser key makes key-only lookups ambiguous by construction (`silu`/`relu` now share `ew1`). | Fuel (names it in GAP-350 as NOT STARTED), Baracuda (provider), anything speaking Announce | **[rule]** — a second regeneration for every Announce speaker; PM ruled "own PR", **timing and go are CireSnave's** |
| R1.2 | **Scale-dtype keying for quantized `ctr` cells** (#222 candidate 1) — without reversing §6.6-0015 for non-`ctr` cells. An `f8e8m0` and an `f16` scale are the same cell today. | Unpopped (`structure_key.rs:717`, pinned as a deliberate equality), Baracuda quant path | **[rule]** — a ruling on a decided boundary; another `structure_key` bump |
| R1.3 | **Block-granularity collision above bucket saturation** (#222 candidate 2): block counts ≥ 16 derive identical cells. | Unpopped (same file, line 137) | recorded as a known limitation; **[rule]** only if someone wants finer buckets |
| R1.4 | **Rule the open reading Unpopped still waits on**: §6.3-0011 "active axis" for zero extents. The spec text (§6.3-0011, §6.5-0002) now pins it; confirm with Unpopped it is no longer open and tell them where. | Unpopped CHANGELOG 0.13.0 | **[go]** — a question, then a pointer |
| R1.5 | **Give `VariantFidelity` a home** (KISS-Emit category-(c) gap) or rule it stays out. | Unpopped `docs/deferred.md` ~448–462 | **[rule]** |
| R1.6 | ~~A vector for KISS-CLASSIFY-6.8-0002~~ — **already exists**: `target_match_vectors` (8 pairs, `expect_match` verdicts, vocabulary-blind) in `structure_key_vectors.json` since #392 (2026-09-05). Unpopped's note that none is vendorable is stale; tell them. | Unpopped `docs/deferred.md` ~1041 | **done** (answer sent) |
| R1.7 | **Namespace vocabulary decisions that are not ours**: widen the `cuda:` manifest from the 4-row dispatch set to the 13-token set is Baracuda's `vocabulary_version` call; KISS's part is only keeping `spec/namespaces/cuda.md` current. | Baracuda `CHANGELOG.md` alpha.91 | **[wait: PM/Baracuda]** |
| R1.8 | **Define the batch coordinate** (`b<class>`): which axes are batch axes, what extent the class is computed from, what "batched" means for extent 1 and rank > 3 (KISS#543). Fuel takes it as a caller-supplied extent; Unpopped = leading dim of a rank-3 cell. Blocks the `ctr` half of the derivation corpus (R3.3). | Fuel, Unpopped, any third deriver | **[rule]** — boarded in PM item 193; not started |

### R2 — consumer asks that need new spec

| # | Item | Needed by | State |
|---|---|---|---|
| R2.1 | **Attention observability** — a second output slot (per-key column-sum of post-softmax weights) via `output_views`. Open "four-way (Lightbulb / Fuel / Baracuda / KISS)". | lightbulb `docs/superpowers/notes/capability-regression-audit.md` L36 | **[rule]** — an Ops/Contract shape; needs a proposal first |
| R2.2 | ~~**Vectors for the four rounding atoms.**~~ **done (#539, 0.2.4).** Fuel GAP-300 reads "`trunc` has no spec decomposition"; it is a **primitive-floor atom** (§6.7: no decomposition by design), so what was missing is a *vector* — the oracle corpus carried none for `floor`/`ceil`/`trunc`/`round_even`. | Fuel GAP-300 | **done**: `ops-rounding.json`, 72 f32 cells, no NaN cell — #396 |
| R2.3 | ~~**NaN vectors for `max_prop` vs `fmax_ieee`**~~ **done (#540, 0.2.5)**: `ops-minmax-nan.json`, 44 f32/f64 cells; a prop↔ieee swap differs on 44/44; kiss-ref 0.4.0 reproduces 44/44. Fuel runs it as the cosign (its lazy-Tensor test). Still open: inf/subnormal operands (value arm) — not asked for. | Fuel `PROVENANCE.md`, #329 | **done** (closing #329 is CireSnave's) |
| R2.7 | ~~say in the vector header that a `class: ULP` cell with a NaN expected is compared per 6.8-0010~~ **done (0.2.6)**: `ops-transcendental-nan.json` carries a `harness_rule` key (the only file with such cells; 13 of 13 since 0.2.14, #550: +f64 `log` of an sNaN, from Fuel's macOS leg). kiss-ref re-pin: kiss-ref#54 (also vendors `ops-minmax-nan` 44 and `ops-rounding` 72). | Fuel finding 2026-10-14 | **done** |
| R2.4 | **Reference wire codec + per-op OpAttrs bytes** with a cross-implementation byte comparison (#504, #502, #23). OpAttrs has **no** cross-implementation oracle today. | kiss-ref (models none), every OpAttrs consumer | **[go]**, large |
| R2.5 | **`randombits` basis slot, shape-rule expression vocabulary** — filed by Fuel as outreach RFCs; `dims`/`with_dim` are registered *experimental* (umbrella §6.4.1) pending ≥2 dissimilar implementations. | Fuel `docs/outreach/kiss-rfc-*.md` | **[rule]** per RFC; promotion needs Baracuda as the second consumer |
| R2.6 | **SVD / linalg ops** — absent from the op vocabulary; needed only if low-rank KV compression is wanted. | lightbulb audit L243–245 | **not started; no consumer committed** |
| R2.7 | **FDX-successor sidecar** — D7 adopted as an annex + clause 6.3-0012 (#533). Baracuda's "bless a neutralized successor so we can drop the private mirror" thread is *not opened* per its own prep doc. | Baracuda `docs/fdx-successor-baracuda-position-2026-07-20.md` | **[wait: Baracuda opens the thread]** |

### R3 — make the freeze demonstrable (instruments; ours)

Ordered by what unblocks a freeze claim, not by count.

| # | Item | State |
|---|---|---|
| R3.1 | **The freeze-gate instruments tell the truth.** **Done (0.2.9, #545):** the blocking Draft→Frozen hard-fail (#524) with 10 seeded controls. **Open:** `--freeze-ready` measures existence, not accuracy (#387); who signs (#462, boarded); **#387:** wording + non-gating PROVEN-per-sub figure shipped (0.2.16, #552); the instrument/wording choice is boarded (PM 193). **#144:** option A ruled; consume/emit/grammar sidecars + the 6.2-0004 drift check shipped (0.2.17), the other six follow; `determinism_class` and `wire_abi_schema_version` need a ruling before they are set. | **[go]** for the remaining sidecars; **[rule]** for the two fields, #387's instrument and the signer |
| R3.2 | **Classify first** (closest: 81/120). 39 untraced clauses; start with the ones consumers cite. Then Ops (88), Announce (34), Consume (28), Emit (38), Grammar (37), Contract (74), Conform (88), Synth (99). | **[go]**, the long tail; each is a test, a lint row, or an honest ledger category |
| R3.3 | **Condition 1 per-field provenance — derivation corpus.** ~~the report~~ **non-`ctr` cells done (0.2.8)**: `structure_key_derivation_vectors.json` (32 vectors: raw operands + hints -> token or typed decline) and the reference emit-derivation stage `derive.rs`; ten vectors reproduce A.1 goldens derived from raw cells. Still open: `ctr` cells (blocked on R1.8 / #543 — the batch role), and a **second deriver** run over it (Fuel/Unpopped/Baracuda), which is what makes a field count as independently derived. | **[wait: #543 for ctr; consumers for the cross-run]** |
| R3.4 | ~~Single-toolchain differential~~ **done (0.2.10, #546):** `KISS_C_COMPILER` + the `windows-clang-cl` job runs the suite under clang-cl, fails (never skips) if LLVM is absent; passed on its first CI run. The loader is still Win32-only. | **done** |

### R4 — freeze (not ours to complete)

| # | Item | State |
|---|---|---|
| R4.1 | **A foreign-language reader** of the golden wire (Classify token, Announce frames, Contract document) with the adversarial-outsider checklist. | **[ext]** |
| R4.2 | **Genuine external implementors** (E6): other minds, other framework backgrounds, other languages. | **[ext]**; a recruiting/outreach question for CireSnave, not an engineering one |
| R4.3 | **AUDIT-role signature** per sub-standard, jointly with the editor-of-record. | **[ext]**, after R3 and R4.1–2 |

## 4. What each consumer needs, in the order it can use it

| Consumer | Needs from KISS, in order | Gaps it reports that are **not** KISS's to close |
|---|---|---|
| **Fuel** | R0.3 (answer) → R1.1 (Announce op identity) → R2.2 (rounding vectors, #539) → R2.3 (NaN corpus) → R2.5 (RFCs) | GAP-350 Baracuda's `sk5` keyer; its own Semantics op-DAG (KISS-Contract §6.4-0001) and `OpAttrs` canonical bytes; MX dtypes outside the closed set; SeamHello lacks availability list / contract-query frames; `revision_hash` is `u64` where KISS pins 32 bytes (`docs/outreach/kiss-conformance-and-divergences.md`, dated 2026-07-14 — may be stale) |
| **Baracuda** | R1.7 (decision is its own) → R2.7 (open the thread) → R1.1 | `sk4`→`sk5` keyer and `contract_version` 3 (PM task issued); parses versions permissively where §6.7-0002 says MUST-reject; `rall`/`rlast` read side (agenda B2/E3, July — verify) |
| **Unpopped** | R1.4 (pointer) → R1.2 (ruling) → R1.5 | re-vendor from `5b28bf5`. Its README holds re-vendoring of the dtype/op manifests "until the CRLF/LF generator fix lands": **measured here, both generators' `--stdout` paths already emit LF** (`python tools/kiss_dtypes.py --emit-manifest --stdout`, `…kiss_ops.py…`: 0 `` bytes at `5b28bf5`), so that hold is stale |
| **kiss-ref** | R2.4 (OpAttrs oracle) | models no `structure_key`/Contract by design; the `#[non_exhaustive]` reservation landed at 0.4.0; `op_manifest.json` now lists the §6.18 complex ops (`all_ops` = 122; #305's premise no longer holds — close it, R0.2) |
| **lightbulb** | R2.1 (attention observability) | pinned at kiss-ref **0.2.0** — two minors behind 0.4.0; bumping is lightbulb's/the PM's |

## 5. Standing rules for this file

- Items are added with the evidence (path or command) and the consumer that needs them; an item with no
  consumer and no clause is a preference and does not belong here.
- Finishing an item deletes it or moves it to `CHANGELOG.md`; a roadmap that accretes done work is a log.
- A **[rule]** item is never started on a peer's say-so; the PM brings CireSnave's answer verbatim.
- Numbers move with `origin/main`; **re-run the commands in §2 before quoting any of them**.

## 6. R0.2 — issues the merged work answered (ready list; closing is CireSnave's)

Each verified at `origin/main` `5b28bf5`.

| Issue | Evidence |
|---|---|
| #263 | ruled D, landed KISS#535; `docs/design-263-cell-class.md` |
| #520 | KISS-CLASSIFY-6.6-0008 amended in #535; goldens corrected in #536 |
| #477 | `math_fidelity`, `contract_version` 3 in #535 |
| #305 | `op_manifest.json` `all_ops` = 122 and contains all 15 KISS-OPS-6.18-0001 ops (set membership) |
| #219 | KISS-CLASSIFY-6.6-0019 defines the weight role as a caller hint (#245); discriminating vector #248 |
| #364 | Appendix C shows all seven blocks: `test_appendix_c_shows_every_block_the_codec_renders` passes (#496) |

**Not on the list:** #298 (ops.md still states no per-op operand structure) and #222 (the collection; candidate 1 is a ruling).

