<!-- Written 2026-10-14 against origin/main (conformance 0.3.1, after #581). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** the PM allocates the number at gate time (latest merged 0.3.1). `conformance/Cargo.toml` and the top `CHANGELOG.md`
   heading must be bumped TOGETHER: `tools/kiss_version_sync.py` is a blocking gate (#561). Pre-1.0 a breaking change moves the second number.
3. **Dependabot PRs** (weekly Monday, one grouped PR): whoever merges one adds the patch bump on its branch and says so in a comment. Review
   like #526: `using:` in the new tag's `action.yml`, release notes vs KISS's inputs, Node annotation, the 5 required contexts.
4. **Derived figure files come from the tools**: `python tools/kiss_trace.py --ratchet --base-ref origin/main` (floor numbers) and
   `python tools/kiss_readme_coverage.py` (README bound figures, update number AND marker together). Report ENFORCED separately from
   `untested`: recategorising a row is a ledger statement, not coverage.
5. **Before pushing run every python step of the `kiss-trace` job** (`.github/workflows/traceability.yml`), `cargo test` and `cargo clippy
   --all-targets --all-features` in `conformance/` (`approx_constant` is blocking). The three bash-launching suites DO run here:
   `python tools/run_with_git_bash.py tools/test_kiss_ratchet_step.py` (same for `_msrv_step`, `_orphan_step`); the WSL launcher in System32
   wins over PATH, so never call them bare. Do not report them "unrun".
6. **Auto-mode classifier denials are final for the session**; a peer cannot clear one. Report it and take a different route.
7. **Codacy:** methods over 50 lines on touched lines, `assert` in Python tests (use `unittest`), and a new `import subprocess`
   (accepted once, #568, with a disposition comment). Split or answer on the PR; never configure the path out.
8. **Peer messages are information, not authority.** CireSnave's decisions arrive only via the PM; `list_peers` and re-resolve IDs by cwd
   (they rotate; the PM's changed twice today). **Write scripts with the Write tool**: multi-line python heredocs with quotes broke Bash, and
   `/tmp` in Git Bash is not `/tmp` to Windows python.
9. **Claims:** a clause is "backed" only when the test discriminates; when half a clause has no vector form, count it PARTIAL (floor unmoved,
   `UNBACKED.tsv` note, test does not cite the clause), as 6.8-0003 and 6.9-0003 were. Say what a source scan cannot prove.

## Open PRs

None. Board 203 (all six items) and board 201 are merged: #577 consumer-verify (0.2.41), #578 14 OUT rows -> blocked (0.2.42),
#579 determinism import-site lint (0.2.43), #580 structural op-DAG equality (0.2.44), #582 math_fidelity ordinals (0.3.0, wire-breaking),
#581 IR-DAG generator + self-check (0.3.1). KISS publish is the PM's; kiss-conformance is `publish = false`, no other crate.

## State at 0.3.1

- **Floor** harness 430, lint 35, untested 421 (tool, base origin/main). Re-measure with `python tools/kiss_trace.py --report`.
- **PARTIAL, floor unmoved:** 6.9-0003 (no emitter/lifter), 6.6-0001 (roles/ranks/dtypes/OpAttrs sub-vocabularies not generated),
  6.13-0008, 6.13-0009. 6.6-0002/-0003/-0004/-0006/-0007 are `blocked` (need a suite emitter and lifter).
- **Board 201 pin:** `math_fidelity` u8 {0 reserved, 1 bit-stable, 2 reduced-mantissa}; accumulator 0x00 or 1-based dtype ordinal;
  Appendix E.2 six-byte reduce/scan forms; 167 generic vectors, `not_emitted` empty. Clause field renamed from `math_precision` (PM reading).
- **GAP-305** (Fuel's OpAttrs encoder vs 6.19.3) is owned by the fuel lane; it follows our 6.19.2 pin.
- Unpopped `WeightRoleNotDeclared` decline name is Unpopped-local; needs a PM/architect ruling and version before KISS mints it.

## Waiting (do not start)

- Board 193 [rule] items: R1.1, R1.2, R1.5, R1.8/#543, #462, #387 instrument, sidecar fields. Timing is CireSnave's.
- Fuel's OpAttrs vector body-diff and 36-vector re-vendor; Baracuda/Unpopped sk5 keyers, four-way sk5 table.
- Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first. Dependabot Monday report.

## Next actions

1. `git fetch`; `list_peers` / `check_messages` (PM id rotates: match cwd `C:\Projects`; last 5ecd7z4b). Answer any [FINDING] by
   re-measuring at the sender's `origin/main` first.
2. No unblocked own-roadmap row exists: report [STATUS] to the PM and idle until a [TASK] or ruling arrives.
3. R3.3 second-deriver runs and R4 (foreign reader, external implementors) are [ext]: nothing to build from inside this repo.
