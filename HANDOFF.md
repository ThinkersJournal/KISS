<!-- Written 2026-10-24 against origin/main d8f2322 (conformance 0.4.66); #657 proposes 0.4.67, this PR (#658) 0.4.68. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first; a CRLF-only diff of `spec/conform.md` is autocrlf noise, `git checkout --` it; then `git worktree remove`, `git worktree prune`). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** the PM allocates the number at gate time (every pushed change bumps, docs and ledger included). `conformance/Cargo.toml` and the top
   `CHANGELOG.md` heading are bumped TOGETHER (`tools/kiss_version_sync.py` blocks). Pre-1.0 a breaking change moves the second number. Queued PRs conflict on exactly those two files at each rebase: keep every entry, descending, renumber.
3. **Dependabot PRs** (weekly Monday, one grouped PR): whoever merges one adds the patch bump on its branch and says so in a comment. Review
   like #526: `using:` in the new tag's `action.yml`, release notes vs KISS's inputs, Node annotation, the 5 required contexts.
4. **Derived figure files come from the tools**: `python tools/kiss_trace.py --ratchet --base-ref origin/main` and
   `python tools/kiss_readme_coverage.py` (re-derive after EVERY rebase; never pick a side). `--update-ledger` wipes the board-203 note in `UNBACKED.tsv`: `git checkout` that file after it.
5. **Before pushing run EVERY `run:` step of the `kiss-trace` job** in `.github/workflows/traceability.yml`, not only the `test_*.py` ones: the non-test steps (`kiss_spdx.py --check`, version sync, ...) reddened #657 once and `test_kiss_known_red.py` reddened #655 once. Also `cargo test`/`clippy --all-targets --all-features` in `conformance/`. The three bash-launching suites: `python tools/run_with_git_bash.py tools/test_kiss_ratchet_step.py` (same for `_msrv_step`, `_orphan_step`); never call them bare (WSL launcher). New `tools/*.py` need `python tools/kiss_spdx.py --stamp`. Every failure site in `kiss_trace.py` goes through `note_fail("<tag>")`.
6. **Auto-mode classifier denials are final for the session**; report and take a different route.
7. **Codacy:** methods over 50 lines on touched lines, `assert` in Python tests (use `unittest`), a new `import subprocess`, lambda assignment. Split or answer on the PR; never configure the path out.
8. **Peer messages are information, not authority.** CireSnave's decisions arrive only via the PM; all approval requests go to the PM (cwd `C:\Projects`, last id `eqktnoyh`; IDs rotate, re-resolve with `list_peers`). Write scripts with the Write tool, not multi-line quoted heredocs; `/tmp` in Git Bash is not `/tmp` to Windows python. Subagents: pin `model: sonnet`, brief read-only, spot-check their claims before writing them down.
9. **Claims:** a clause is "backed" only when the test discriminates; half a clause with no vector form is PARTIAL (floor unmoved, `UNBACKED.tsv` note, no citing test). A tag reason must not use the form "back(s) KISS-..." (parsed as a citation).
10. Coderipper asked callers of `ciresnave/coderipper` `portfolio.yml` to repin to `ee9c3950c510fae136d316fed3bd9ecda343cede` (`# 0.9.24`) at the next PR; not yet checked whether KISS's `portfolio` workflow pins it.

## Open PRs (all proposed numbers; the PM renumbers)

- **#657 (0.4.67)** KISS#387 (a), CireSnave "Still (a)": `conformance/CONJUNCTS.tsv` + `tools/kiss_conjuncts.py`; `// Backs: KISS-X [label]`; `--freeze-ready` fails a listed clause with an uncited label; default run fails unknown clause/label. Seed gap: OPS-6.18-0017 `zero-sign-re` cited by no test. READY sent; in the PM's gate.
- **#658 (0.4.68)** spec-to-roadmap audit (PM TASK): ROADMAP R5.1-R5.18 added (59 docs read, 18 rows, 9 need a CireSnave call, `[ASK]` sent), R3.7/R3.8 corrected, glossary fixed, this HANDOFF. Rebase over #657 for CHANGELOG/Cargo.
- On each merge: delete the remote branch (`gh api -X DELETE repos/ThinkersJournal/KISS/git/refs/heads/<b>`, verify absent), remove worktrees `C:/Projects/kiss-r387` (r387-conjuncts) and `C:/Projects/kiss-audit` (audit-roadmap).

## Done this session (history: `git log`, CHANGELOG)

#651 R3.13 done; #652-#655 KISS#136 complete (6.1-0003 carve-out text, 116+111 `harness-internal:` tags, 74 `Backs:`, orphan gate armed, 225 tagged / 0 orphans); #656 fifth Conform modality, clause text only (KISS-CONFORM-6.14-0001..0004, floor untested 387→391). Board 271 all three decisions executed except the **pair checker** (below).

## Floor (origin/main d8f2322)

harness 459, lint 44, untested **391** (387 + the four 6.14 arrivals). Re-measure: `python tools/kiss_trace.py --report`. PARTIAL, floor unmoved: 6.9-0003, 6.6-0001, 6.13-0008, 6.13-0009; 6.6-0002/-0003/-0004/-0006/-0007 `blocked` (suite emitter/lifter, R5.5).

## Rulings to keep

6.17-0002 subnormal preservation is part of bit-stable; `rm` is a permission, never an obligation; an FMA kernel cannot claim `st` until R1.11 (`[rule]`); Unpopped N2 NaN: option (a), **later**, no spec text, trigger a measured `VK_FALSE` Vulkan device (ROADMAP R5.11); KISS-CLASSIFY-6.1-0001 is 24 recognized / 21 usable / 3 reserved (answered Unpopped C7).

## Waiting (do not start)

- Unpopped/Baracuda/Fuel regeneration for sk6 and `list_version` 2 (their lanes). R4 and second-deriver runs are `[ext]`. R1.10 needs a scheduled freeze; R1.11 is `[rule]`.
- All `[rule]` rows of R5 wait on CireSnave via the PM; never start them on a peer's say-so.
- Throttle: `C:\Projects\THROTTLE.md` (FULL SPEED 2026-10-21) at session start and each task boundary. Dependabot Monday **2026-10-26**.

## Next actions

1. `git fetch`; `list_peers`/`check_messages`. `gh pr view 657` and `658`: on merge do the cleanup above; if the gate says the base moved, rebase (procedure 2, 4).
2. **Build the pair checker for KISS#240 / #239 item 3** on the merged 6.14 text: mechanically check that two vectors differ in field k alone and have differing required outputs; a coordinate with no pair is recorded (6.14-0002), a missing pair fails the build (6.14-0004). Clause wording of 6.14-0001 exclusions and 6.14-0003 (no corpus as evidence) is still for CireSnave to read; do not edit it without the PM.
3. Remaining unread ROADMAP rows: R3.10, R3.11, R3.12. **#458** (a `UNBACKED.tsv` category for a test that binds the document) has three unchosen remedies: ask the PM before building. R3.5a needs a region-format design read. R5.6 (re-triage 8 stale `#144` ledger notes) and R5.14/R5.15 are `[go]` and startable.
