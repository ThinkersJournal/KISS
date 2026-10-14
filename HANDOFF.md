<!-- Written 2026-10-14 against origin/main ff12265 (conformance 0.2.39; #576 carries 0.2.40). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** the PM allocates the number at gate time (latest merged 0.2.39; #576 = 0.2.40). `conformance/Cargo.toml` and the top `CHANGELOG.md`
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

**KISS#576** (hygiene, 0.2.40, PM-allocated): `.github/CODEOWNERS`, workflow `permissions: contents: read`, `docs/glossary.md` index,
plus this HANDOFF. Waiting on checks and the PM's gate; on merge: `git worktree remove C:/Projects/kiss-hyg`, delete branch `kiss-hygiene`.
CodeRipper rules read the COMMITTED tree: an uncommitted fix still shows as a finding. Build it from a worktree of coderipper `origin/main`
with `CARGO_TARGET_DIR` in the scratchpad; the binary under `C:/Projects/coderipper/target/debug` is stale.

## State at 0.2.39 (main ff12265)

- **Floor** harness 420, lint 34, untested 446, proven 20. Every KISS-GRAMMAR and KISS-CONFORM `untested` row now carries a reason
  (#574, #575); none is bare. Re-measure with `python tools/kiss_trace.py --report` at the ref you work from.
- **Backed this session:** CONFORM-6.13-0003 (#573), 6.13-0012 (#575). PARTIAL, floor unmoved: 6.13-0008 (no Contract-side opaque embed),
  6.13-0009 (Grammar vectors 2 of 5; G2/G3/G5 unrendered). GRAMMAR-6.6-0003/-0004/-0005 stay untested: the crate has no cell-class field
  and validates neither op names nor role tokens; a test needs new reference behaviour.
- **R1.4 CLOSED:** Unpopped 0.16.0 derives `[8,0]` as `da/v1`, matching our 36-vector file. **Classify:** 88 backed / 32 ledgered /
  0 untraced of 120; the 6 `untested` rows have no subject and wait on an architect ruling.
- **GAP-305** (Fuel's OpAttrs encoder diverges from 6.19.3) is OPEN, owned by the fuel lane, queued after nn .13. Live OpAttrs pair count 0.
- Earlier state unchanged: R3.2 Ops vectors landed (int-wrap, compare-bool, atan2, sqrt PARTIAL, nextafter PARTIAL); R2.4 slice 1 landed;
  R1.9 (`reduce`/`prefix_scan` `math_precision` ordinal table) is a [rule] not minted.

## Waiting (do not start)

- **Board 203 (CireSnave): the ~40 Conform rows with no subject.** My [ASK] recommended: consumer-verify check IN (with a profile
  feature), structural op-DAG equality IN, IR-DAG fuzzer SPLIT (generator in; cross-backend/emitter rows out -> `blocked`), canonical
  suite artifact OUT (needs a signer, #462), oracle provenance record OUT, determinism-enum import-site lint IN. **No recategorisation
  and no tests until ruled.**
- Board 193/201 [rule] items: R1.1, R1.2, R1.5, R1.8/#543, R1.9, #462, #387 instrument, sidecar fields. Timing is CireSnave's.
- Fuel: body-diff of its clause-derived OpAttrs vectors (will send a [FINDING]); re-vendor of the 36-vector file. Baracuda/Unpopped sk5
  keyers, four-way sk5 table.
- Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first. Dependabot Monday report.

## Next actions

1. `git fetch`; read `gh pr view 576`; if merged, clean up (above). Read `list_peers` / `check_messages` (the PM's id rotates: match cwd
   `C:\Projects`); answer any [FINDING] by re-measuring at the sender's `origin/main` first.
2. If board 203 is ruled, the rulings define the next slices; otherwise no [go] item exists: report [STATUS] to the PM and idle.
3. R3.3 second-deriver runs and R4 (foreign reader, external implementors) are [ext]: nothing to build from inside this repo.
