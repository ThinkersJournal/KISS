<!-- Written 2026-10-08 against origin/main e4b866d. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS` (its HEAD is not `origin/main`).
   `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`; remove it after the merge
   (`git status --porcelain` first). Read `origin/main` (after `git fetch`), never a working tree.
2. **Every docs/CI/test PR carries its own `kiss-conformance` patch bump** (`conformance/Cargo.toml`, currently
   0.1.3) **or names in its body the change set whose bump lands with it.** The PM allocates the number.
   Pre-1.0 a breaking change moves the second number. The crate number is bookkeeping: conformance keys on the
   per-sub-standard schema versions (KISS-CONFORM-8-0001), which a PR bumps only if its own bytes/semantics change.
3. **Dependabot PRs (`.github/dependabot.yml`, github-actions, weekly Monday, one grouped PR, no auto-merge):
   whoever merges one adds the `kiss-conformance` patch bump as a commit on that PR's branch before merging,
   and says so in a PR comment** (PM ruling 2026-10-08, option 1; no standing quarterly change set). Review each
   like #526: read `using:` in the new tag's `action.yml` (node20 -> node24 is the control), read the release
   notes against KISS's inputs, and check the Node annotation and the 5 required contexts on the PR's own run.
4. **Derived figure files are re-derived by the tool, never kept from a side.** `conformance/COVERAGE_FLOOR.tsv` and
   the README bound markers come from `python tools/kiss_trace.py --ratchet --base-ref origin/main` and
   `python tools/kiss_readme_coverage.py`. If a rebase conflicts only in them: `git rebase --abort`, rename the old
   branch `...-v1-backup`, `git checkout -b <name> origin/main`, cherry-pick the non-figure commits, then re-run
   the tools and commit the numbers separately.
5. **Before pushing run every python step of the `kiss-trace` job** (list in `.github/workflows/traceability.yml`),
   not just the tests you touched; a new `tools/kiss_*.py` must answer `--emit-coverage` at once (#266). Three
   suites launch `bash` by name and fail on this box (WSL launcher): `test_kiss_ratchet_step.py`,
   `test_kiss_msrv_step.py`, `test_kiss_orphan_step.py`; say so in the PR body, CI is their first run.
6. **Auto-mode classifier denials are final for the session**: a peer (including the PM) cannot clear one, and
   re-issuing the same outcome through another tool is the same outcome. Report it and take a different route.
7. `strict` (non-required) now reports 479 untested MUSTs / "0 of 9 freeze-ready" in its job summary and is green
   while that is the known red; it goes red only if the red changes or an instrument fails. Do not "ack" it by name.

## State (as of origin/main e4b866d, 2026-10-08)

- Merged this reconciliation: #516 (Ops/Contract, `contract_version` 2), #517 (Classify), #518 (precision-class
  token set, Ops 6.8-0007..0009), #519 (layout_tag from own axes, mask over the frame-padded view), #521/#522
  (`cuda:` annex: token set / dispatch set, nine NVRTC-only rows), #523 (0.1.0), #525 (`strict` report),
  #526 (Actions to Node 24 majors), #527 (Dependabot). No open KISS PRs.
- Open issues owned here: **#520** (mask keys on the stride of an extent-1 axis: two tokens for one broadcast
  meaning; PM prefers normalizing, not urgent), **#524** (the hard-fail on a freeze or a conformance claim that
  the old `strict` comment promised is NOT implemented).
- Other lanes' pending work that touches KISS output: Fuel `structure_key_derive.rs` own-axes layout (fuel#285,
  held for bundling), Unpopped 0.14.0 (mask over the frame). Nothing is owed by KISS to them.

## Next actions

- **Monday 2026-10-12, after Dependabot's first scheduled run:** read the Dependabot tab (Insights -> Dependency
  graph -> Dependabot) and tell the PM what it shows. This is an observation, not a prediction; expected: nothing.
- **2026-10-19:** `ubuntu-latest` becomes Ubuntu 26. No pin added; if CI breaks that day tell the PM first.
- Otherwise idle until the PM assigns work.
