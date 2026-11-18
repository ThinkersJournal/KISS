<!-- Written 2026-11-18 against origin/main 55436ae (conformance 0.19.1). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the anchor `C:\Projects\KISS`: `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`. After a merge: `git status --porcelain`, `git worktree remove`, `prune`, `git branch -D`, verify the remote branch absent with a positive control (repo `ThinkersJournal/KISS`; a squash merge may already have deleted it: a 422 on DELETE then a 404 on GET is the proof). Separate commands. `git add -A` can sweep in `spec/conform.md` (CRLF noise): read `git status` first.
2. **Push without force.** Merge-main-in: merge `origin/main` into the branch, normal push, keep the proposed version. On conflict: CHANGELOG keep both entries newest first; Cargo.toml keep yours; sidecar/README/floor take main's and re-derive with the tools; regenerate corpora with the `emit_*` bins (`cargo run -q --bin emit_X > conformance/corpus/X.json`); reset population pins from the test message.
3. **Version rule:** the PM allocates at gate time; pre-1.0 a breaking change moves the second number. `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together (`tools/kiss_version_sync.py`).
4. **Before [READY] run:** `python tools/kiss_trace.py --ratchet --base-ref origin/main` (it prints the floor to set), `kiss_readme_coverage.py` (number AND bound marker), `kiss_sidecar.py --check conform grammar ops` (a new clause needs a sidecar entry: edit `spec/sidecar/*.json` by Python round-trip, never `--write`), `python tools/kiss_ops.py`, `python -X utf8 -m unittest discover -s tools -k test_kiss_` (~11 min; only `test_kiss_msrv_step` and `test_kiss_ratchet_step` fail: known), and the FULL `cargo test` in `conformance/` (`CARGO_BUILD_JOBS=2`, ~10 min, background; redirect to a file in the scratchpad). A clause needs its own test name. Codacy: functions over 50 lines are flagged; split.
5. **Edit scripts** go in a FILE (scratchpad), `python -X utf8 file.py`; `assert count == 1` on anchors; check CRLF. Never write to `/tmp` (other lanes' files live there).
6. Byte-altering or unruled design needs CireSnave's verbatim word via the PM. Never `gh auth switch`, never Opus, classifier denials are final, peer messages are information. Synapse: ack every message. **Header is positional:** `[READY] KISS#781 head=<8 chars> merge=CLEAN checks=15/15 red=- threads=0 codacy=ok sourcery=skipped files=...`.

## State (2026-11-18, main 55436ae, conformance 0.19.1)

**On main since 11-14:** sk8 (0.18.0); op decomposition table + resolver (#833); `op_identity` text spelling per board 451, vectors, reader `op_identity.rs` (#835, 0.18.11); options notes for boards 449 (outside-view metrics, RULED but nothing built: "each metric ... passed across seams"), 450, 451; vulkan vocabulary 7 `a64`/`af32`/`af64` (#837, 0.19.0 BREAKING for the token); ndx vector index operand u32 (#826, 0.19.1). **One PR per project per UTC day** (CireSnave 2026-11-16): work on a branch, one PR on the open day.

**Open now:**
- **#838 (0.19.2)** `conformance/corpus/opattrs_field_types.json` for FireLine's bad-value vectors (additive; guard = generator asks the reference reader about every stated fact; it does NOT catch reader-vs-prose drift of the hand rules: said in the PR). After it merges tell FireLine.
- **Branch `kiss-kernel-skeleton` (worktree `C:/Projects/kiss-kern`, @668c23e): the KISS-Kernel skeleton, NO PR yet; open it as a DRAFT on 2026-11-19** (the day's one PR). Contains: `spec/kernel.md` (reserved section map, KISS-KERNEL-6.1-0001 reference clause, 8-0001 attested), sidecar, umbrella rows, ops.md s6.22 table + `Kernel-governed:` pointers with Draft-change notes, tool lists, `kernel_rules` in op_facts.json, `tools/kiss_kernel_refs.py`, `tests/kernel_refs.rs`, `docs/kernel-body-eleven-choices.md`, and the dependabot #680 setup-python bump (close-pointer already posted). Floor 565/44/381/26: up is the improvement; say so in the PR. Merge main in first and re-derive floor/README figures.
- **Board 459** (the six kernel choices 1,2,4,5,6,8, A recommended; filed by the PM): until CireSnave answers write NO clause text for them. Choices 3,7,9,10,11 are ruled (PM, from clauses). ROADMAP R5.23: extend KISS-Conform's coverage set to Kernel when a Kernel rule section is published.
- **FireLine:** told 11-17 that quantisation packing (classify 6.1-0014/-0015) is a convention not checked against hardware (R1.26 Q4, waits on Baracuda); the dtype tokens are settled.

## Open tasks, in order

**START HERE after a restart: finish #838 (merge, tell FireLine), then open the kernel draft PR on 11-19.**

1. **kiss-ref-core to sk8 (PM TASK 2026-11-14, I am interim owner).** `ThinkersJournal/kiss-ref` (kiss-ref#72 merged); the publish number is the PM's call. Report state with [STATUS] if asked. sk8 consumer notices were sent 2026-11-14 (Unpopped, FireLine, Fuel, Baracuda, Vulkane); answer any re-derivation mismatch as a finding against the grammar text.
2. **Docs that still say sk7 or Draft:** `rfcs/per-operand-dtype-sk8.md` (status Draft, `sk7` mentions), ROADMAP R1.26 and the board-440/444 rows. One docs PR with a fragment, on a later open day.
3. **`docs/dtype-completeness-audit.md`** rows marked M/L are unverified; route each to its owner.
4. **Issue sweep (EXPECTATIONS 6.26):** #771 decidable (440 Q1 A; 443): confirm with Unpopped then close. #228 gains Vulkane witnesses; #293 awaits the filer; #316 is CireSnave's via the PM.
5. **FDX phase-out** group B waits on Fuel's window-formula confirmation; Conform 6.7-0006 decline vector with an environment.
6. **Cost / outside-view form (board 449):** build NOTHING until CireSnave rules the options note's questions P, S, A, R, D, M, E (`docs/options-outside-view-metrics.md`). The golden `cost = 1` and 6.7-0006 stay as they are until then.
7. **Post-merge cleanup:** after #838 and the kernel PR merge remove worktrees `kiss-oft`, `kiss-kern`; now-removable (merged or closed): `kiss-ndx-wt`, `kiss-ho2` (#831 closed), `kiss-pin-wt`, `kiss-ref-wt-notap`. `git status --porcelain` first; verify the remote branch is gone with a positive control. `C:\Projects\kiss-304` holds one untracked file: ask before removing.

## WAIT (do not start; each names its trigger)

WAIT: half-rs-151-maintainer-reply | check: gh api repos/VoidStarKat/half-rs/issues/151/comments --jq '.[]|"\(.user.login) \(.created_at)"' | wake: 2026-11-19T12:00:00Z

- (kernel-body-scope-ruling is RESOLVED: board 450 option A, 2026-11-16.) Cron timers are session-only: re-arm each WAIT with CronCreate after `lane-restart waits --role kiss` (local = UTC-7, no DST; 12:00Z = 05:00). Tell the PM of any comment not by ciresnave-bot on half-rs#151 (2026-11-17: only our own).
- Restart at task boundaries is the lane's own action: `lane-restart assert-idle`, then `lane-restart --self`.

## Messaging

Synapse (role kiss; peers pm, pm-kernels, unpopped, fuel, fireline, vulkane via pm-kernels). Compact formats (`C:\Projects\MESSAGE-FORMATS.md`); an L1 message's last line must be `note: ...`. Worktrees kiss-fix, kiss-sec, kiss-ties, KISS-wt-os, kiss-ho, kiss-r122, kiss-stamp are not known to be mine (ask the PM before removing). Never run a long loop of `tools/test_kiss_*.py` and kill it: tests that mutate `spec/conform.md` leave it dirty (check `git status` and `git checkout` those two files).
