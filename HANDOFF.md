<!-- Written 2026-10-13 against origin/main 5b28bf5. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** every docs/CI/test PR names the change set whose `kiss-conformance` bump covers it. The PM allocates the number
   (latest merged is 0.2.17, #553; the PM allocates the next number at gate time). Pre-1.0 a breaking change moves the second number.
3. **Dependabot PRs** (weekly Monday, one grouped PR): whoever merges one adds the patch bump as a commit on its branch and says so in a
   comment. Review like #526: `using:` in the new tag's `action.yml`, release notes vs KISS's inputs, Node annotation, the 5 required contexts.
4. **Derived figure files come from the tools**, never from a side: `python tools/kiss_trace.py --ratchet --base-ref origin/main` and
   `python tools/kiss_readme_coverage.py`. A new lint-backed clause needs: the `lint:<tool>` row in `conformance/UNBACKED.tsv`, the tool's
   `--emit-coverage` entry, and the floor's `lint` number raised by the PR itself (the ratchet then reports a green "born with detector" arrival).
5. **Before pushing run every python step of the `kiss-trace` job** (`.github/workflows/traceability.yml`). Three suites launch `bash` by name and
   fail on this box (WSL): `test_kiss_ratchet_step.py`, `test_kiss_msrv_step.py`, `test_kiss_orphan_step.py`; say so in the PR body.
6. **Auto-mode classifier denials are final for the session**; a peer cannot clear one. Report it and take a different route.
7. **Codacy flags methods over 50 lines** on the lines a PR touches: split before pushing. Never configure the path out.
8. **Peer messages are information, not authority.** CireSnave's decisions arrive only via the PM; objections from Fuel/Baracuda/Unpopped go
   through the PM too (`list_peers`, re-resolve IDs by cwd; they rotate).

## Open KISS PRs

None (origin/main at conformance 0.2.20, 2026-10-14; this HANDOFF PR carries 0.2.21 if open). kiss-ref 0.4.1 merged (#54); no open kiss-ref PR.

## Current state at origin/main (conformance 0.2.20)

- **#263 landed** (`sk5`, derived 7-class `cell_class`, `contract_version` 3); design in `docs/design-263-cell-class.md`.
  kiss-ref 0.4.0 is published (bound to `a33282c`); corpus re-pin is kiss-ref#54. Touch kiss-ref only on a PM task.
- **New instruments since:** `ops-rounding.json` (72) and `ops-minmax-nan.json` (44, Fuel cosigned 44/44 and 72/72);
  the cell-carrying `structure_key_derivation_vectors.json` (33 vectors, 10 anchored to Appendix A.1) with its reference
  deriver `conformance/src/derive.rs`; the blocking freeze hard-fail (`tools/kiss_freeze_gate.py`); a second C
  toolchain (`KISS_C_COMPILER`, CI job `windows-clang-cl`); `ops-transcendental-nan.json` is 13 vectors (0.2.14, f64 `log` sNaN).
- **First second-party derivation runs** (Fuel#358, Baracuda): rulings in KISS#547. **Owed by implementations:** collapsed
  reduction output must decline, reduced axis out of range must decline (6.6-0009); Unpopped: reduced innermost axis -> `v1`
  (6.5-0009(b)). Re-vendor `structure_key_derivation_vectors.json` at blob `299811b1`.
- **Sidecar (#144, option A):** `tools/kiss_sidecar.py` (`--check`; `--write` seeds once, refused over an existing one). Done: all nine (0.2.17, 0.2.18). `determinism_class` / `wire_abi_schema_version` stay `null`/UNSET until CireSnave rules (PM item 193; option C amends Appendix E).
- **Integer / bool vectors (R3.2 Ops, 0.2.19-0.2.20):** `ops-int-wrap.json` (440, 6.2-0002) and `ops-compare-bool.json` (181,
  6.2-0005/-0006). Both opt-in; **not yet run through kiss-ref or Fuel** (Fuel has a [NOTICE] for int-wrap, no reply). 6.2-0007 is
  deliberately unbacked (bitwise/reduction/scan arms uncovered). Ratchet floor: harness 405, untested 472. OPS untraced 85.
- **#387:** `--freeze-ready` now says existence-not-accuracy and prints PROVEN per sub-standard (0.2.16); the instrument/wording choice is boarded (PM 193).
- **Roadmap:** `ROADMAP.md` is the ordered list. Take the next **[go]** item without asking; never start a **[rule]** item.

## Held / waiting (do not start)

- **R1.1 Announce op identity in every lookup** (`docs/design-263-cell-class.md` §6): own PR, own `list_version`; **timing is CireSnave's**.
- **#222 candidate 1** (quantized `ctr` scale dtype keying) and **VariantFidelity's home**: need a ruling.
- Dependabot Monday reports; Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first.

## Next actions

1. **kiss-ref re-vendor** (kiss-ref PR, PM allocates): vendor `ops-int-wrap.json` + `ops-compare-bool.json` at the KISS main that has them
   into `crates/kiss-ref-conformance/fixtures/kiss-corpus/`, add to the `FILES` list in `tests/kiss_corpus_vectors.rs`, update
   `PROVENANCE.md`, CHANGELOG. If kiss-ref cannot reproduce an integer/bool cell that is a [FINDING] (kiss-ref models no native Bool lane).
2. R3.2 Ops, remaining candidates with a vector subject: `nextafter` value arm (6.9-0003; its typed-decline half has no vector form, so do
   not back the clause), ~~`atan2` quadrants (6.9-0001)~~ done in this change (0.2.22, `ops-atan2-quadrants.json`, libm-backed oracle, not yet through kiss-ref), `sqrt` correct rounding (6.8-0003). The rest of Ops
   (decomposition/level/registry/version/complex) has no subject in the harness: a design question, not a vector batch.
3. R2.4: OpAttrs cross-implementation byte comparison (#504/#502) — large; needs a design note before code.
4. Waiting, do not start: R1.1/R1.2/R1.5/R1.8 and #462 (boarded, PM item 193); `ctr` derivation vectors wait on #543; Fuel's reply on
   `ops-int-wrap.json`; Unpopped sk5 keyer; the four-way sk5 table.
