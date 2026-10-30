<!-- Written 2026-10-30 against origin/main a68a7c5f (conformance 0.6.45). A HANDOFF is current state, never a log:
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
10. Coderipper 0.10.0 advisory rule `patent-sweep-record`: keep `docs/PATENT-SWEEP.md` current (a docs PR with a version bump) when it lands.
11. **Stacks: build ONE LINEAR stack**, each PR based on its parent branch, each ONE commit made by `git cherry-pick --no-commit` onto the parent branch (never merge main into a branch: it re-adds squashed content, the PM HOLDs it). Never force-push (the classifier denies it). Helper scripts were in the session scratchpad and are gone: CHANGELOG entry + `conformance/Cargo.toml` version are bumped together; README figures and the floor come from `kiss_readme_coverage.py` / `kiss_trace.py --ratchet`. Run ALL of: version_sync, readme_coverage, clause_position, tables, scoped_cites, ratchet.

## State (2026-10-30)

**Throttle:** read `C:/Projects/THROTTLE.md` at session start (FULL SPEED since 2026-10-28T04:30Z at last read). `CARGO_BUILD_JOBS=2`, one cargo build at a time.

**Merged 2026-10-30:** #731-#736, then #737 (HANDOFF), #738 (Appendix A.1 G3 prose fixed, `tag_vectors` in `grammar_vectors.json`), #739 (G5 options note), #740 (ROADMAP R3.18 define scalar-bearing node, blocked by R1.25), #741 (ROADMAP R5.19 `lint:kiss_sidecar` ledger category), #742 (`ops-ulp-edge.json`, 0.6.44), #743 (`ops-move-wide-nan.json`, 0.6.45, main `a68a7c5f`). Floor: harness 484, lint 44, untested 393. Both corpus files are informative, NO consumer obligation, not Breaking; #20 stays open (Fuel: f32-only comparator, pins +-0 at distance 1, not scheduled, will comment on #20 itself). Fuel was told both files exist.

**HELD for the sk7 batch (PM says when closed; do not merge alone):** **#724** (`c-maxop`, worktree `C:/Projects/kiss-maxop`, `MAX_OPERANDS` 8->16, sk6->sk7; conflicts with main) and **#725** (`c-f1ops`, worktree `C:/Projects/kiss-f1`, stacked on #724: `cast`/`concat` added, `transpose`/`unsqueeze` removed; on restack: the ROADMAP F1 sentence "2026-10-29 (PM, option B) ... open values Ops App. D item 12"; replay the commit on a fresh branch, no force-push). Padding low/high and matmul attrs wait on CireSnave (304 R5.7). **R1.31** HELD. **R3.17: do NOT vendor sk6/v4 into kiss-ref now; one vendoring at the sk7 close.**

**Rulings to keep:** R3.10 remainder: G5 blocked by R3.18/R1.25 (option A accepted as direction only); G2 waits on the OpAttrs byte freeze (grammar 8-0008). Unpopped normative seams Q1-Q5: stay "derived per implementation", no new MUSTs; whether KISS STATES any is CireSnave's (board 338); take no KISS row for Q2-Q5 until he answers. **Target tokens (PM 2026-10-30):** no row now; each emitter lane (bonefish sycl, rockfish hip, openclownfish opencl, vulkane spirv/glsl/hlsl/wgsl) sends a concrete token list + derivability witness (Classify 6.8-0014); then add `spec/namespaces/<ns>.md`, one PR per namespace, informative about status. KISS owns the `<ns>:<capability>` token vocabulary; the `TargetId` enum is Unpopped's. R1.31 held. 311 Q1 C, Q2 A, Q3 A, Q4 A, Q5 A+C. 310 Q1 (#10) re-opened, Q4 (#26) open: build neither. 308 B levels; 6.17-0002 subnormals are bit-stable; `rm` is a permission.

## Owed

1. R1.29: a SHOULD clause on the edge corpus only after Fuel, Baracuda and Unpopped adopt it; a MUST is CireSnave's call, not proposed. Close #20 only when a consumer confirms (or the PM says).
2. R5.6 remainder: retired-ID convention (6.2-0003), POD regeneration (6.1-0006), declared coverage set (6.1-0008). R5.19 (`lint:kiss_sidecar` category) is a row awaiting a decision.
3. Board 309 Q1 wire work (provision-by-definition, experimental capability bit 40-43, recipe encoding): unbuilt. R3.14 (Emit/Consume twin witnesses), a second implementation's witness for Conform 7.2-0005. 6.16-0011 vectors need a composite cell shape (not carried).
4. Tell Unpopped lane 1 (cwd `C:/Projects/Unpopped`; `list_peers`) when #725 nears the gate.
5. Issue clock (EXPECTATIONS 6.23): `python /c/Projects/tools/issue-age.py --repo ThinkersJournal/KISS` (30 open, none overdue at 2026-10-30); weekly sweep next due **2026-11-01**.
6. **Before every push run ALL single-line `python tools/...` kiss-trace steps (47) plus the three bash suites via `run_with_git_bash.py`** (a clause containing "comparator" needs a `Normalizes:` declaration or a `SELECTS_ONLY` entry in `tools/kiss_comparators.py`; Codacy flags methods >50 lines). A PM-gated stack renumbers one PR at a time: after each `[MERGED]`, merge main in (never force), renumber the CHANGELOG heading and `conformance/Cargo.toml`, re-derive README/floor, `[READY]`.


## Waiting (do not start)

All `[rule]` rows wait on CireSnave via the PM. Unpopped/Baracuda/Fuel regeneration is their lanes. R4 and second-deriver runs are `[ext]`. R1.10 needs a scheduled freeze; R1.11 is `[rule]`. Board 295 interface-project plan, outside-readers call publication, and the patent sweep leftovers wait on CireSnave or the lanes.
