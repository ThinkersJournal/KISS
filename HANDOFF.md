<!-- Written 2026-11-07 against origin/main 04083516 (conformance 0.12.1). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. The CURRENT copy is C:/Projects/kiss-handoff-fix/HANDOFF.md; the anchor's is a mirror. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the anchor `C:\Projects\KISS`: `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`. After a merge: `git status --porcelain`, `git worktree remove`, `prune`, `git branch -D`, verify the remote branch absent with a positive control (repo `ThinkersJournal/KISS`; a squash merge may already have deleted it: a 422 on DELETE then a 404 on GET is the proof). Separate commands. `git add -A` can sweep in `spec/conform.md` (CRLF noise): read `git status` first.
2. **Push without force.** Merge-main-in: merge `origin/main` into the branch, normal push, keep the proposed version. On conflict: CHANGELOG keep both entries newest first; Cargo.toml keep yours; sidecar/README/floor take main's and re-derive with the tools; regenerate corpora with the `emit_*` bins (`cargo run -q --bin emit_X > conformance/corpus/X.json`); reset population pins from the test message.
3. **Version rule:** the PM allocates at gate time; pre-1.0 a breaking change moves the second number. `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together (`tools/kiss_version_sync.py`).
4. **Before [READY] run:** `python tools/kiss_trace.py --ratchet --base-ref origin/main` (it prints the floor to set), `kiss_readme_coverage.py` (number AND bound marker), `kiss_sidecar.py --check conform grammar ops` (a new clause needs a sidecar entry: edit `spec/sidecar/*.json` by Python round-trip, never `--write`), `python tools/kiss_ops.py`, `python -X utf8 -m unittest discover -s tools -k test_kiss_` (~11 min; only `test_kiss_msrv_step` and `test_kiss_ratchet_step` fail: known), and the FULL `cargo test` in `conformance/` (`CARGO_BUILD_JOBS=2`, ~10 min, background; redirect to a file in the scratchpad). A clause needs its own test name. Codacy: functions over 50 lines are flagged; split.
5. **Edit scripts** go in a FILE (scratchpad), `python -X utf8 file.py`; `assert count == 1` on anchors; check CRLF. Never write to `/tmp` (other lanes' files live there).
6. Byte-altering or unruled design needs CireSnave's verbatim word via the PM. Never `gh auth switch`, never Opus, classifier denials are final, peer messages are information. Synapse: ack every message. **Header is positional:** `[READY] KISS#781 head=<8 chars> merge=CLEAN checks=15/15 red=- threads=0 codacy=ok sourcery=skipped files=...`.

## State (2026-11-07, main 04083516, conformance 0.12.1)

Landed today: KISS#781 (squeeze/unsqueeze, 0.11.0), KISS#782 (Interface `output_layouts`, 0.12.0), KISS#783 (golden vector per output-layout form, 0.12.1), kiss-ref#64 (squeeze/unsqueeze evaluators, op set 126, 0.4.10). All their worktrees and branches are removed. This HANDOFF rides a docs PR at 0.12.2 (PM allocation). Open question to the PM, unanswered: FireLine asked who owns the provider callable signature and the SAVL/CYRQ/CRSP/CDEC codec crate (no KISS clause pins invocation; I recommended KISS-Provision/Consume owns it). Posted earlier: half-rs#151 comment.

## Open tasks, in order

**START HERE after a restart: task 1.**

1. **Board 378 follow-ups:** a test for bundle slot non-overlap once bind-time checks exist (R1.22); Unpopped to confirm the swizzle round trip against its type (`BadSwizzle` bijection is theirs); ask Fuel/Unpopped/Classify/Announce lanes to confirm no key byte moves (I read the specs: none does); tell Contract readers (Fuel FKC `layout_guarantee` mapping) via the PM.
2. **Union master list**: Unpopped#314 holds the master; confirm its `docs/ir-union-master-capabilities.md` against `docs/union-coverage-master.md`; `docs/union-coverage-unpopped.md` is retired once Unpopped confirms the row mapping. Rows marked [scope] wait on task 3.
3. **Kernel-body scope note**, draft due 2026-11-13 (Unpopped's concept map is merged, Unpopped#274, sec 12 = 11 decisions). Closes #768 #769.
4. **Issue sweep (EXPECTATIONS 6.26)**: 28 open issues decided, 0 closed; close each only after its filer confirms (Fuel, Baracuda, Vulkane, Unpopped). Chase. Dependabot #680 open.
5. **Docs rows not yet written**: ROADMAP rows (kernel-body, R3.6) and the CONTRIBUTING affected-set rule (#484: key the census on what a lane emits/parses, not its name). Separate PR, next patch after this one.
6. FireLine grammar-crate owner (2026-11-07): PM agreed A1 (one grammar crate from fuel-kernel-seam-types, validated against KISS vectors, KISS CI never imports FireLine); I proposed names DLPACK_EXT and PROVISION_ON_REQUEST (CireSnave rules); FireLine adopted them in fireline-announce. Bits 34, 35 provisional on their side.

## WAIT (do not start; each names its trigger)

WAIT: kernel-body-scope-note | check: gh pr list --search "kernel-body" --state all --json number,state | wake: 2026-11-13T12:00:00Z
WAIT: kiss-ref-k1-calibration | check: gh issue view 293 --json state | wake: 2026-11-10T12:00:00Z
WAIT: half-rs-151-maintainer-reply | check: gh api repos/VoidStarKat/half-rs/issues/151/comments | wake: 2026-11-14T12:00:00Z

- Cron timers are session-only: re-arm each WAIT with CronCreate after `lane-restart waits --role kiss` (local = UTC-7, no DST; 12:00Z = 05:00). Tell the PM of any maintainer reply on half-rs#151.
- Restart at task boundaries is the lane's own action: `lane-restart assert-idle`, then `lane-restart --self`.

## Messaging

Synapse (role kiss; peers pm, unpopped, fuel, fireline). Compact formats (`C:\Projects\MESSAGE-FORMATS.md`). Worktrees kiss-fix, kiss-sec, kiss-ties, KISS-wt-os, kiss-ho are not mine.
