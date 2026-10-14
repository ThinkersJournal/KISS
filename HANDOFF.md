<!-- Written 2026-10-13 against origin/main 5b28bf5. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** every docs/CI/test PR names the change set whose `kiss-conformance` bump covers it. The PM allocates the number
   (latest merged is 0.2.16, #552; the PM allocates the next number at gate time). Pre-1.0 a breaking change moves the second number.
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

KISS#553-slot: the sidecar PR (0.2.17) if open; otherwise none (origin/main `055ed018`, 2026-10-14, conformance 0.2.16). kiss-ref 0.4.1 merged (#54).

## Current state at origin/main 055ed018 (conformance 0.2.16)

- **#263 landed** (`sk5`, derived 7-class `cell_class`, `contract_version` 3); design in `docs/design-263-cell-class.md`.
  kiss-ref 0.4.0 is published (bound to `a33282c`); corpus re-pin is kiss-ref#54. Touch kiss-ref only on a PM task.
- **New instruments since:** `ops-rounding.json` (72) and `ops-minmax-nan.json` (44, Fuel cosigned 44/44 and 72/72);
  the cell-carrying `structure_key_derivation_vectors.json` (33 vectors, 10 anchored to Appendix A.1) with its reference
  deriver `conformance/src/derive.rs`; the blocking freeze hard-fail (`tools/kiss_freeze_gate.py`); a second C
  toolchain (`KISS_C_COMPILER`, CI job `windows-clang-cl`); `ops-transcendental-nan.json` is 13 vectors (0.2.14, f64 `log` sNaN).
- **First second-party derivation runs** (Fuel#358, Baracuda): rulings in KISS#547. **Owed by implementations:** collapsed
  reduction output must decline, reduced axis out of range must decline (6.6-0009); Unpopped: reduced innermost axis -> `v1`
  (6.5-0009(b)). Re-vendor `structure_key_derivation_vectors.json` at blob `299811b1`.
- **Sidecar (#144, option A):** `tools/kiss_sidecar.py` (`--check`; `--write` seeds once, refused over an existing one). Done: consume, emit, grammar. TODO: announce, classify, conform, contract, ops, synth (+umbrella?). `determinism_class` / `wire_abi_schema_version` stay `null` until ruled (second ASK sent).
- **Roadmap:** `ROADMAP.md` is the ordered list. Take the next **[go]** item without asking; never start a **[rule]** item.

## Held / waiting (do not start)

- **R1.1 Announce op identity in every lookup** (`docs/design-263-cell-class.md` §6): own PR, own `list_version`; **timing is CireSnave's**.
- **#222 candidate 1** (quantized `ctr` scale dtype keying) and **VariantFidelity's home**: need a ruling.
- Dependabot Monday reports; Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first.

## Next actions

1. R3.1: traceability ACCURACY (#387) and the clause sidecar (#144) — design first; both are instruments, not text.
2. R3.2: the remaining untested Classify clauses are lint-bound or restatements (see KISS#542 body); look at Ops (88) next, starting with clauses consumers cite.
3. R2.4: OpAttrs cross-implementation byte comparison (#504/#502) — large; needs a design note before code.
4. Waiting, do not start: R1.1/R1.2/R1.5/R1.8 and #462 (boarded, PM item 193); `ctr` derivation vectors wait on #543.
