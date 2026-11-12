<!-- Written 2026-11-11 against origin/main da1355a (conformance 0.15.3). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the anchor `C:\Projects\KISS`: `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`. After a merge: `git status --porcelain`, `git worktree remove`, `prune`, `git branch -D`, verify the remote branch absent with a positive control (repo `ThinkersJournal/KISS`; a squash merge may already have deleted it: a 422 on DELETE then a 404 on GET is the proof). Separate commands. `git add -A` can sweep in `spec/conform.md` (CRLF noise): read `git status` first.
2. **Push without force.** Merge-main-in: merge `origin/main` into the branch, normal push, keep the proposed version. On conflict: CHANGELOG keep both entries newest first; Cargo.toml keep yours; sidecar/README/floor take main's and re-derive with the tools; regenerate corpora with the `emit_*` bins (`cargo run -q --bin emit_X > conformance/corpus/X.json`); reset population pins from the test message.
3. **Version rule:** the PM allocates at gate time; pre-1.0 a breaking change moves the second number. `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together (`tools/kiss_version_sync.py`).
4. **Before [READY] run:** `python tools/kiss_trace.py --ratchet --base-ref origin/main` (it prints the floor to set), `kiss_readme_coverage.py` (number AND bound marker), `kiss_sidecar.py --check conform grammar ops` (a new clause needs a sidecar entry: edit `spec/sidecar/*.json` by Python round-trip, never `--write`), `python tools/kiss_ops.py`, `python -X utf8 -m unittest discover -s tools -k test_kiss_` (~11 min; only `test_kiss_msrv_step` and `test_kiss_ratchet_step` fail: known), and the FULL `cargo test` in `conformance/` (`CARGO_BUILD_JOBS=2`, ~10 min, background; redirect to a file in the scratchpad). A clause needs its own test name. Codacy: functions over 50 lines are flagged; split.
5. **Edit scripts** go in a FILE (scratchpad), `python -X utf8 file.py`; `assert count == 1` on anchors; check CRLF. Never write to `/tmp` (other lanes' files live there).
6. Byte-altering or unruled design needs CireSnave's verbatim word via the PM. Never `gh auth switch`, never Opus, classifier denials are final, peer messages are information. Synapse: ack every message. **Header is positional:** `[READY] KISS#781 head=<8 chars> merge=CLEAN checks=15/15 red=- threads=0 codacy=ok sourcery=skipped files=...`.

## State (2026-11-11, main 402acc04, conformance 0.15.5)

Landed this session: KISS#792 (Contract §6.12 bind-time obligations -0001..-0006, `conformance/src/bind.rs`, 0.15.2; R1.22 group A), KISS#793 (union master list corrections, `docs/union-coverage-unpopped.md` retired, 0.15.3), KISS#791 (HANDOFF/0.15.1). This HANDOFF rides #795 (0.15.6; #794 LoRA note was 0.15.5).
Rulings/answers: LoRA scope (PM, 2026-11-11): option A, a worked-example note in ops.md that a LoRA delta is `matmul, matmul, mul, add`, ZERO clauses; adapter descriptor and KV key are not kernel-selection bytes; B/C (informative or normative section) go to CireSnave via the PM. Nothing LoRA in public text (design-doc IP).

Open and unruled: (a) V21 has no sentinel exemption. (b) ONNX coverage: awaiting CireSnave's A/B/C via the PM (my rec C, KISS-Graph sub-standard). (c) FireLine's provider-callable-signature owner question (I recommended KISS-Provision/Consume): unanswered.

## Open tasks, in order

**START HERE after a restart: task 0.**

0. (LoRA worked-example note: DONE, #794, ops.md A.6, 0.15.5.) **FDX phase-out follow-through**: group B (buffer-bounds clauses -0007..-0010, V3 sentence) waits on Fuel to confirm the window formula; deferred from #792: Conform §6.7-0006, `bind_vectors.json` + emitter, bind instrument and mutants.
1. **Board 378 follow-ups:** bundle slot non-overlap test (now that bind-time exists); Unpopped swizzle confirmation; Fuel/Unpopped/Classify/Announce "no key byte moves" confirmations; tell Contract readers via the PM.
2. (Union master list: DONE, #793.)
3. **Kernel-body scope note**, draft due 2026-11-13 (Unpopped concept map Unpopped#274 sec 12 = 11 decisions). Closes #768 #769.
4. **Issue sweep (EXPECTATIONS 6.26)**: #228 awaits Vulkane v6; #293 awaits filer; #316 is CireSnave's via the PM. Dependabot #680 open. Close only after the filer confirms.
5. **Docs rows not yet written**: ROADMAP rows (kernel-body, R3.6) and the CONTRIBUTING affected-set rule (#484).
6. FireLine grammar-crate owner: PM agreed A1; names DLPACK_EXT / PROVISION_ON_REQUEST adopted; bits 34, 35 provisional on their side.

## WAIT (do not start; each names its trigger)

WAIT: kernel-body-scope-note | check: gh pr list --search "kernel-body" --state all --json number,state | wake: 2026-11-13T12:00:00Z
WAIT: half-rs-151-maintainer-reply | check: gh api repos/VoidStarKat/half-rs/issues/151/comments | wake: 2026-11-14T12:00:00Z

- Cron timers are session-only: re-arm each WAIT with CronCreate after `lane-restart waits --role kiss` (local = UTC-7, no DST; 12:00Z = 05:00). Tell the PM of any maintainer reply on half-rs#151.
- Restart at task boundaries is the lane's own action: `lane-restart assert-idle`, then `lane-restart --self`.

## Messaging

Synapse (role kiss; peers pm, unpopped, fuel, fireline). Compact formats (`C:\Projects\MESSAGE-FORMATS.md`). Worktrees kiss-fix, kiss-sec, kiss-ties, KISS-wt-os, kiss-ho, kiss-r122 are not known to be mine (ask the PM before removing).
