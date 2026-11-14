<!-- Written 2026-11-14 against origin/main 2c1e2cfa (conformance 0.17.4). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the anchor `C:\Projects\KISS`: `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`. After a merge: `git status --porcelain`, `git worktree remove`, `prune`, `git branch -D`, verify the remote branch absent with a positive control (repo `ThinkersJournal/KISS`; a squash merge may already have deleted it: a 422 on DELETE then a 404 on GET is the proof). Separate commands. `git add -A` can sweep in `spec/conform.md` (CRLF noise): read `git status` first.
2. **Push without force.** Merge-main-in: merge `origin/main` into the branch, normal push, keep the proposed version. On conflict: CHANGELOG keep both entries newest first; Cargo.toml keep yours; sidecar/README/floor take main's and re-derive with the tools; regenerate corpora with the `emit_*` bins (`cargo run -q --bin emit_X > conformance/corpus/X.json`); reset population pins from the test message.
3. **Version rule:** the PM allocates at gate time; pre-1.0 a breaking change moves the second number. `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together (`tools/kiss_version_sync.py`).
4. **Before [READY] run:** `python tools/kiss_trace.py --ratchet --base-ref origin/main` (it prints the floor to set), `kiss_readme_coverage.py` (number AND bound marker), `kiss_sidecar.py --check conform grammar ops` (a new clause needs a sidecar entry: edit `spec/sidecar/*.json` by Python round-trip, never `--write`), `python tools/kiss_ops.py`, `python -X utf8 -m unittest discover -s tools -k test_kiss_` (~11 min; only `test_kiss_msrv_step` and `test_kiss_ratchet_step` fail: known), and the FULL `cargo test` in `conformance/` (`CARGO_BUILD_JOBS=2`, ~10 min, background; redirect to a file in the scratchpad). A clause needs its own test name. Codacy: functions over 50 lines are flagged; split.
5. **Edit scripts** go in a FILE (scratchpad), `python -X utf8 file.py`; `assert count == 1` on anchors; check CRLF. Never write to `/tmp` (other lanes' files live there).
6. Byte-altering or unruled design needs CireSnave's verbatim word via the PM. Never `gh auth switch`, never Opus, classifier denials are final, peer messages are information. Synapse: ack every message. **Header is positional:** `[READY] KISS#781 head=<8 chars> merge=CLEAN checks=15/15 red=- threads=0 codacy=ok sourcery=skipped files=...`.

## State (2026-11-14, main 2c1e2cfa, conformance 0.17.4)

Landed since 0.15.8: R5.18 docs, 430/433 rulings, iota (422 A), cast odd cases (426), bitcast (427 B), Contract index_width spelling `ix32`/`ix64` (#815), `in_place = none | input:<k>` (#813; golden `KISC kiss-contract 4 len=1280 crc32=79f503e4`, 1325 bytes), BREAKING 0.16.0 padding_lo/hi (#811), BREAKING 0.17.0 matmul accumulator+math_fidelity+precision_tag and the packed reductions Welford/online-softmax/argmax with Classify presenter 6.6-0025 (#812), the KISS#771 options note (#816, board 440), FireLine's five contract-vector requests (#818, 0.17.4: `section_vectors` key, 6 accept + 49 declines, stricter section reader; FireLine byte-diffed it clean). The sk7 batch is CLOSED (PM, at #812). Floor at 0.17.1: harness 552, untested 389.
**Stamp procedure (the PM now stamps):** a PR carries one `changelog.d/<topic>.md` fragment; `python tools/check-fragment.py origin/main`; merge `origin/main` in, re-derive `conformance/COVERAGE_FLOOR.tsv`, README `<!-- bound -->` figures and `UNBACKED.tsv` (`kiss_trace.py --ratchet --base-ref origin/main`, `--update-ledger`, `kiss_readme_coverage.py`); regenerate corpus vectors with `cargo run --bin emit_opattrs_value_vectors` / `emit_opattrs_vectors`; `cargo metadata --locked` must exit 0; the sidecar clause list is plain string-sorted. Do not push to a PR the PM is stamping.
Open and unruled (CireSnave via the PM): board 440 (#771 note: tag out of key, tags added when an emitter ships a bounded cell, accumulate-into-C as a dense [M,N] extra input); the sk8 per-operand-dtype RFC (`rfcs/per-operand-dtype-sk8.md`) and the backward RFC (`rfcs/every-operation-carries-its-backward.md`) wait on his board items; kernel-body scope (#796).

## Open tasks, in order

**START HERE after a restart: task 3 (the issue sweep); tasks 1 and 2 wait on a ruling or on Fuel.**

1. **Contract/consumer follow-ups:** tell the consumers when board 440 or sk8 is ruled; Unpopped re-vendors sk7 as 0.46.0 (PM sequences it).
2. **FDX phase-out** group B waits on Fuel's window-formula confirmation; Conform 6.7-0006 decline vector with an environment (deferred from #792).
3. **Issue sweep (EXPECTATIONS 6.26):** #228 awaits Vulkane v6; #293 awaits the filer; #316 is CireSnave's via the PM; #771 stays open until board 440 is ruled. Close only after the filer confirms.
4. Worktree `C:\Projects\kiss-304` holds one untracked file (`docs/board-304-held-items.md`, superseded by his 426 answers): ask before removing.

## WAIT (do not start; each names its trigger)

WAIT: kernel-body-scope-ruling | check: grep -n -A3 "^### 417" C:/Projects/CIRESNAVE-DECISIONS.md | wake: 2026-11-15T12:00:00Z
WAIT: board-440-ruling | check: grep -n -A3 "^### 440" C:/Projects/CIRESNAVE-DECISIONS.md | wake: 2026-11-15T12:00:00Z
WAIT: half-rs-151-maintainer-reply | check: gh api repos/VoidStarKat/half-rs/issues/151/comments | wake: 2026-11-15T12:00:00Z

- Cron timers are session-only: re-arm each WAIT with CronCreate after `lane-restart waits --role kiss` (local = UTC-7, no DST; 12:00Z = 05:00). Tell the PM of any maintainer reply on half-rs#151 (2026-11-14: only our own comment so far).
- Restart at task boundaries is the lane's own action: `lane-restart assert-idle`, then `lane-restart --self`.

## Messaging

Synapse (role kiss; peers pm, unpopped, fuel, fireline). Compact formats (`C:\Projects\MESSAGE-FORMATS.md`). Worktrees kiss-fix, kiss-sec, kiss-ties, KISS-wt-os, kiss-ho, kiss-r122 are not known to be mine (ask the PM before removing).
