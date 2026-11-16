<!-- Written 2026-11-16 against origin/main 8c14914 (conformance 0.18.7 stamped). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the anchor `C:\Projects\KISS`: `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`. After a merge: `git status --porcelain`, `git worktree remove`, `prune`, `git branch -D`, verify the remote branch absent with a positive control (repo `ThinkersJournal/KISS`; a squash merge may already have deleted it: a 422 on DELETE then a 404 on GET is the proof). Separate commands. `git add -A` can sweep in `spec/conform.md` (CRLF noise): read `git status` first.
2. **Push without force.** Merge-main-in: merge `origin/main` into the branch, normal push, keep the proposed version. On conflict: CHANGELOG keep both entries newest first; Cargo.toml keep yours; sidecar/README/floor take main's and re-derive with the tools; regenerate corpora with the `emit_*` bins (`cargo run -q --bin emit_X > conformance/corpus/X.json`); reset population pins from the test message.
3. **Version rule:** the PM allocates at gate time; pre-1.0 a breaking change moves the second number. `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together (`tools/kiss_version_sync.py`).
4. **Before [READY] run:** `python tools/kiss_trace.py --ratchet --base-ref origin/main` (it prints the floor to set), `kiss_readme_coverage.py` (number AND bound marker), `kiss_sidecar.py --check conform grammar ops` (a new clause needs a sidecar entry: edit `spec/sidecar/*.json` by Python round-trip, never `--write`), `python tools/kiss_ops.py`, `python -X utf8 -m unittest discover -s tools -k test_kiss_` (~11 min; only `test_kiss_msrv_step` and `test_kiss_ratchet_step` fail: known), and the FULL `cargo test` in `conformance/` (`CARGO_BUILD_JOBS=2`, ~10 min, background; redirect to a file in the scratchpad). A clause needs its own test name. Codacy: functions over 50 lines are flagged; split.
5. **Edit scripts** go in a FILE (scratchpad), `python -X utf8 file.py`; `assert count == 1` on anchors; check CRLF. Never write to `/tmp` (other lanes' files live there).
6. Byte-altering or unruled design needs CireSnave's verbatim word via the PM. Never `gh auth switch`, never Opus, classifier denials are final, peer messages are information. Synapse: ack every message. **Header is positional:** `[READY] KISS#781 head=<8 chars> merge=CLEAN checks=15/15 red=- threads=0 codacy=ok sourcery=skipped files=...`.

## State (2026-11-16, main 8c14914)

0.18.0 (sk8, #822) is on main; since then merged: op_facts sidecar + `declares_reduce_axis` (#825, #827), op_dag/Identity conformance vectors (#828) and their spec rulings (#829, Draft-change notes + `rfcs/op-dag-contract-rulings-2026-11-15.md`), kiss-ref #67-#71 (kiss-ref-core 0.5.x). #771 closed (Unpopped confirmed 2026-11-15). Open PRs of mine: KISS#830 (CodeRipper pin to 0.47.5, gated at 0.18.7; branch `pin-move-0.47.5`, worktree `C:/Projects/kiss-pin-wt`), KISS#826 (ndx vector integer index dtype; branch `fix-ndx-index-dtype`, worktree `C:/Projects/kiss-ndx-wt`), kiss-ref#72 (no-real-tap decline, 0.6.0 BREAKING, queued; worktree `C:/Projects/kiss-ref-wt-notap`). After each merges: `git status --porcelain`, remove the worktree, `git branch -D`, verify remote branch absent (positive control: main).
Stamp procedure and the Codacy/ratchet/README steps are in item 4 above. Codacy lexer quirk: a 9-line fn was reported at 75 lines; avoid brace/quote char literals (`b'{'`, `'0'`) near it, use consts.
Open and unruled (CireSnave via the PM): the backward RFC; **kernel-body scope**: my [ASK] (A new sub-standard KISS-Kernel, recommended / B Ops 6.20 / C out) sent 2026-11-16, the PM files it as a NEW board number (417 is the atomic-add band, ruled); R1.26 packing and conversion rounding (Baracuda hardware measurement); the quantisation group (tell FireLine when it settles: fireline#65).

## Open tasks, in order

**START HERE after a restart:** check the forge for #830/#826/kiss-ref#72 (post-merge cleanup above), then task 1.

1. **FireLine's contract-row vectors (agreed 2026-11-15 on Synapse, one conformance/corpus file per row, generated + freshness-gated like op_facts.json):** 6.7-0006 cost expressions (accept/decline, cite op_facts for referenced inputs); 6.3-0003/-0004 op_identity forms (accept/decline per form); 6.4-0005/-0010 resolution to the primitive floor as a generated per-op decomposition table from spec/ops.md. Send [NOTICE] to fireline per file when on main. Also owed: tell FireLine that #828 (op_dag vectors) is on main.
2. **MOD-002 cycle** I introduced (`op_dag_vectors` <-> `section_vectors`/`contract`/`dispatch`/`accuracy`): small follow-up, move shared helpers so the dependency runs one way.
3. **Docs that still say sk7 or Draft:** `rfcs/per-operand-dtype-sk8.md` (status Draft, 5 `sk7` mentions), ROADMAP rows R1.26 and the board-440/444 rows. One docs PR with a fragment.
4. **sk8 consumer follow-ups:** answer any re-derivation mismatch (Fuel, Unpopped, FireLine, Baracuda, Vulkane) as a finding against the grammar text. `docs/dtype-completeness-audit.md` rows M/L: route to device-family owners.
5. **Issue sweep (EXPECTATIONS 6.26):** #228 gains Vulkane witnesses at vocabulary 6; #293 awaits the filer; #316 is CireSnave's via the PM; close only after the filer confirms.
6. **kiss-ref-core:** crates.io has only 0.4.0 while 0.5.x/0.6.0 are merged or queued and consumers pin "0.4.0": the PM decides the publish number. kiss-ref has no Synapse owner; I am interim owner.
7. **FDX phase-out** group B waits on Fuel's window-formula confirmation.
8. Worktree `C:\Projects\kiss-304` holds one untracked file (`docs/board-304-held-items.md`): ask before removing.

## WAIT (do not start; each names its trigger)

WAIT: kernel-body-scope-ruling | check: PM names the new board number N; then grep -n -A3 "^### N" C:/Projects/CIRESNAVE-DECISIONS.md and CIRESNAVE-DECIDED-ARCHIVE.md | wake: 2026-11-17T12:00:00Z
WAIT: half-rs-151-maintainer-reply | check: gh api repos/VoidStarKat/half-rs/issues/151/comments | wake: 2026-11-17T12:00:00Z

- Cron timers are session-only: re-arm each WAIT with CronCreate after `lane-restart waits --role kiss` (local = UTC-7, no DST; 12:00Z = 05:00). Tell the PM of any maintainer reply on half-rs#151 (2026-11-16: only our own comment).
- Restart at task boundaries: `lane-restart assert-idle`, then `lane-restart --self`.

## Messaging

Synapse (role kiss; peers pm, unpopped, fuel, fireline). Compact formats (`C:\Projects\MESSAGE-FORMATS.md`): the first line is `[TYPE] <subject> key=value...` with TYPE one of STATUS, READY, MERGED, HOLD, ASK, TASK, NOTICE, FINDING, FIX. Worktrees kiss-fix, kiss-sec, kiss-ties, KISS-wt-os, kiss-ho, kiss-r122 are not known to be mine (ask the PM before removing).
