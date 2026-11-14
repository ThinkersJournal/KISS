<!-- Written 2026-11-14 against origin/main 11407f72 (conformance 0.18.0). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the anchor `C:\Projects\KISS`: `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`. After a merge: `git status --porcelain`, `git worktree remove`, `prune`, `git branch -D`, verify the remote branch absent with a positive control (repo `ThinkersJournal/KISS`; a squash merge may already have deleted it: a 422 on DELETE then a 404 on GET is the proof). Separate commands. `git add -A` can sweep in `spec/conform.md` (CRLF noise): read `git status` first.
2. **Push without force.** Merge-main-in: merge `origin/main` into the branch, normal push, keep the proposed version. On conflict: CHANGELOG keep both entries newest first; Cargo.toml keep yours; sidecar/README/floor take main's and re-derive with the tools; regenerate corpora with the `emit_*` bins (`cargo run -q --bin emit_X > conformance/corpus/X.json`); reset population pins from the test message.
3. **Version rule:** the PM allocates at gate time; pre-1.0 a breaking change moves the second number. `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together (`tools/kiss_version_sync.py`).
4. **Before [READY] run:** `python tools/kiss_trace.py --ratchet --base-ref origin/main` (it prints the floor to set), `kiss_readme_coverage.py` (number AND bound marker), `kiss_sidecar.py --check conform grammar ops` (a new clause needs a sidecar entry: edit `spec/sidecar/*.json` by Python round-trip, never `--write`), `python tools/kiss_ops.py`, `python -X utf8 -m unittest discover -s tools -k test_kiss_` (~11 min; only `test_kiss_msrv_step` and `test_kiss_ratchet_step` fail: known), and the FULL `cargo test` in `conformance/` (`CARGO_BUILD_JOBS=2`, ~10 min, background; redirect to a file in the scratchpad). A clause needs its own test name. Codacy: functions over 50 lines are flagged; split.
5. **Edit scripts** go in a FILE (scratchpad), `python -X utf8 file.py`; `assert count == 1` on anchors; check CRLF. Never write to `/tmp` (other lanes' files live there).
6. Byte-altering or unruled design needs CireSnave's verbatim word via the PM. Never `gh auth switch`, never Opus, classifier denials are final, peer messages are information. Synapse: ack every message. **Header is positional:** `[READY] KISS#781 head=<8 chars> merge=CLEAN checks=15/15 red=- threads=0 codacy=ok sourcery=skipped files=...`.

## State (2026-11-14, main 11407f72, conformance 0.18.0)

**0.18.0 BREAKING (sk8, #822) is on main**: per-operand dtype in every sub-key (`co/00/v4/d16/f/f32`, `dl`/`dc` after it), `ctr` keys the block length `g16`|`g32` after `q<sdt>` (product of the weight's `block_shape`; axis implied), `x<n>` removed (Vulkan vocabulary 6, Announce `unnamed_types`, KISS-CLASSIFY-6.6-0026, KISS-ANNOUNCE-6.6-0011), `matmul` blob = `accumulator`, `math_fidelity`, `accuracy_kind`, `u64` value (11 bytes; `precision_tag` gone, KISS-OPS-6.17-0011/-0012, Contract 6.8-0002 carve-out), Appendix C golden `KISC kiss-contract 4 len=1304 crc32=2854c69d`. Rulings: boards 430, 440 (Q1 A), 443, 444. Accumulate-into-C stays ordinary ops (no new spelling); no recipe list. Floor: harness 556, lint 44, untested 389. KISS publishes nothing to crates.io (`publish = false`) and has no git tags; no `v0.18.0` tag owed unless the PM rules otherwise (`C:/Projects/tools/tag-release.sh`).
**Stamp procedure (the PM stamps):** a PR carries one `changelog.d/<topic>.md` fragment; `python tools/check-fragment.py origin/main`; merge `origin/main` in, re-derive `conformance/COVERAGE_FLOOR.tsv`, README `<!-- bound -->` figures and `UNBACKED.tsv` (`kiss_trace.py --ratchet --base-ref origin/main`, `--update-ledger`, `kiss_readme_coverage.py`); regenerate corpus vectors with the `emit_*` bins (redirect to a FILE: a failed emit truncates the target); `cargo metadata --locked` must exit 0; sidecar by Python round-trip (`kiss_sidecar.py --check`). Do not push to a PR the PM is stamping. Codacy flags functions over 50 lines and files over ~500 counted lines: split before [READY].
Open and unruled (CireSnave via the PM): the backward RFC (`rfcs/every-operation-carries-its-backward.md`); kernel-body scope (#796/417); R1.26 packing and conversion rounding (waits on Baracuda's hardware measurement).

## Open tasks, in order

**START HERE after a restart: task 1 (then 2).**

1. **kiss-ref-core to sk8 (PM TASK 2026-11-14, I am its interim owner; no lane owns kiss-ref).** Repo `ThinkersJournal/kiss-ref`, crate `kiss-ref-core` on crates.io (lightbulb and unpopped-cpu-c depend on it). Report its state first ([STATUS]: last published version, open PRs, who else touches it), then one worktree and one PR bringing it to sk8 against the corpus (per-operand dtype keys, g16/g32, matmul blob 11 bytes without `precision_tag`, requested bound, Appendix C golden 1304 / crc32 2854c69d); one version number per project, the PM allocates and publishes after read-back; check which version lightbulb and cpu-c really pin.
2. **sk8 consumer follow-ups.** Notices sent 2026-11-14 to Unpopped (0.47.0), FireLine, Fuel (0.48.0, after its #677), Baracuda, Vulkane (vocabulary 6). Answer any re-derivation mismatch as a finding against the grammar text. kiss-ref has no Synapse owner: task 1 makes me its interim owner for 0.18.0.
3. **Docs that still say sk7 or Draft:** `rfcs/per-operand-dtype-sk8.md` (status Draft, 5 `sk7` mentions in migration prose), ROADMAP rows R1.26 (`[wait: schema event]` is spent: block length keyed) and the board-440/444 rows. One docs PR with a fragment.
4. **`docs/dtype-completeness-audit.md`** is a first pass: rows marked M/L are unverified by device-family owners (the naming list for the `x<n>` removal; gaps such as `i128`/`u128`, `c32`). Route each to its owner.
5. **Issue sweep (EXPECTATIONS 6.26):** #771 is decidable now (440 Q1 A; 443: accumulate-into-C option A, tags removed): confirm with Unpopped, then close. #228 gains Vulkane witnesses at vocabulary 6; #293 awaits the filer; #316 is CireSnave's via the PM. Close only after the filer confirms.
6. **FDX phase-out** group B waits on Fuel's window-formula confirmation; Conform 6.7-0006 decline vector with an environment (deferred from #792).
7. Worktree `C:\Projects\kiss-304` holds one untracked file (`docs/board-304-held-items.md`, superseded by his 426 answers): ask before removing.

## WAIT (do not start; each names its trigger)

WAIT: kernel-body-scope-ruling | check: grep -n -A3 "^### 417" C:/Projects/CIRESNAVE-DECISIONS.md | wake: 2026-11-15T12:00:00Z
WAIT: half-rs-151-maintainer-reply | check: gh api repos/VoidStarKat/half-rs/issues/151/comments | wake: 2026-11-15T12:00:00Z

- Cron timers are session-only: re-arm each WAIT with CronCreate after `lane-restart waits --role kiss` (local = UTC-7, no DST; 12:00Z = 05:00). Tell the PM of any maintainer reply on half-rs#151 (2026-11-14: only our own comment so far).
- Restart at task boundaries is the lane's own action: `lane-restart assert-idle`, then `lane-restart --self`.

## Messaging

Synapse (role kiss; peers pm, unpopped, fuel, fireline). Compact formats (`C:\Projects\MESSAGE-FORMATS.md`). Worktrees kiss-fix, kiss-sec, kiss-ties, KISS-wt-os, kiss-ho, kiss-r122 are not known to be mine (ask the PM before removing).
