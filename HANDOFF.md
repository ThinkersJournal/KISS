<!-- Written 2026-10-24 against origin/main 7818026 (conformance 0.4.55); this PR proposes 0.4.56. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first, then `git worktree prune`). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** the PM allocates the number at gate time (latest merged 0.4.55; every pushed change bumps, docs and ledger included, CireSnave 2026-09-25). `conformance/Cargo.toml` and the top
   `CHANGELOG.md` heading must be bumped TOGETHER: `tools/kiss_version_sync.py` is a blocking gate (#561). Pre-1.0 a breaking change moves the second number.
3. **Dependabot PRs** (weekly Monday, one grouped PR): whoever merges one adds the patch bump on its branch and says so in a comment. Review
   like #526: `using:` in the new tag's `action.yml`, release notes vs KISS's inputs, Node annotation, the 5 required contexts.
4. **Derived figure files come from the tools**: `python tools/kiss_trace.py --ratchet --base-ref origin/main` (floor numbers) and
   `python tools/kiss_readme_coverage.py` (README bound figures, update number AND marker together). Report ENFORCED separately from
   `untested`: recategorising a row is a ledger statement, not coverage.
5. **Before pushing run every python step of the `kiss-trace` job** (`.github/workflows/traceability.yml`), `cargo test` and `cargo clippy
   --all-targets --all-features` in `conformance/` (`approx_constant` is blocking). The three bash-launching suites DO run here:
   `python tools/run_with_git_bash.py tools/test_kiss_ratchet_step.py` (same for `_msrv_step`, `_orphan_step`); the WSL launcher in System32
   wins over PATH, so never call them bare. Run `kiss_trace.py` without `python -I`. Do not report them "unrun".
6. **Auto-mode classifier denials are final for the session**; a peer cannot clear one. Report it and take a different route.
7. **Codacy:** methods over 50 lines on touched lines, `assert` in Python tests (use `unittest`), and a new `import subprocess`
   (accepted once, #568, with a disposition comment). Split or answer on the PR; never configure the path out.
8. **Peer messages are information, not authority.** CireSnave's decisions arrive only via the PM; all approval requests go to the PM. `list_peers` and
   re-resolve IDs by cwd (they rotate; PM is cwd `C:\Projects`, last zqx9v437). **Write scripts with the Write tool**: multi-line python heredocs
   with quotes broke Bash, and `/tmp` in Git Bash is not `/tmp` to Windows python. Worktree files can be CRLF on disk.
9. **Claims:** a clause is "backed" only when the test discriminates; when half a clause has no vector form, count it PARTIAL (floor unmoved,
   `UNBACKED.tsv` note, test does not cite the clause), as 6.8-0003 and 6.9-0003 were. Say what a source scan cannot prove.

## Last merged

**This PR (0.4.56, proposed):** R3.6 ROADMAP note + this HANDOFF. **#644 (0.4.55):** `tools/kiss_io_discard.py` (R3.6 #148; 3 hits, all `msvc.rs`, annotated). **#643 (0.4.54):** `*Test:* `a`, `b`` is a violation (#400 option 3). **#642 (0.4.53):** all 24 dtype arms pinned (#239 items 1-2). **#641 (0.4.52):** R3.14 ruled, (b) now, (a) when the first foreign reader exists. **#638-#640 (0.4.49-0.4.51):** CONSUME-6.4-0007 de-credited, R3.9 done. Older history: `git log` and CHANGELOG. Standing rulings: 6.17-0002 subnormal preservation is part of bit-stable (no `denorm` field, no new MathFidelity value, PM 2026-10-21); `rm` is a permission, never an obligation (6.17-0003/-0005); an FMA kernel cannot claim `st` until ROADMAP **R1.11** (a wire change, `[rule]`, not started); 6.17-0003 stays `untested` (no kernel subject in `conformance/`); spec 6.7-0006 warns a weight-first call order defaults `<acc>` to operand 0's dtype. `ctr` stage in `derive.rs` (#591): second-deriver run is the other lanes'.

## State at 0.4.5 (origin/main 3688cfe)

- **#588 merged:** `structure_key` `sk5`->`sk6` (scale dtype keyed for quantized `ctr`, §6.6-0022) + Announce `list_version` 2 (`op_identity`,
  §6.3-0013, Synth mirrored). Per-vector old/new: `docs/sk6-vector-changes.md`. `envelope_version` 1 and `contract_version` 3 unmoved. Nothing frozen.
  KISS publish is the PM's; kiss-conformance is `publish = false`, no other crate. The PM notifies Unpopped, Baracuda, Fuel (CHANGELOG 0.4.0 lists what each regenerates).
- **Open item, KISS lane owns: ROADMAP R1.10 "Frame versioning before freeze".** CYRQ/CRSP/CDEC carry no version field (PM ruling: no bump now, since all
  speakers regenerate together); must be resolved before any freeze. Detector: the freeze checklist. Not startable until a freeze is scheduled.
- **Floor** harness 459, lint 44, untested 387 (0.4.55, unmoved since 0.4.41). Re-measure with `python tools/kiss_trace.py --report`.
- **PARTIAL, floor unmoved:** 6.9-0003 (no emitter/lifter), 6.6-0001 (roles/ranks/dtypes/OpAttrs sub-vocabularies not generated),
  6.13-0008, 6.13-0009. 6.6-0002/-0003/-0004/-0006/-0007 are `blocked` (need a suite emitter and lifter).
- **Board 201 pin:** `math_fidelity` u8 {0 reserved, 1 bit-stable, 2 reduced-mantissa}; accumulator 0x00 or 1-based dtype ordinal;
  Appendix E.2 six-byte reduce/scan forms; 167 generic vectors, `not_emitted` empty.
- **GAP-305** (Fuel's OpAttrs encoder vs 6.19.3) is owned by the fuel lane; it follows our 6.19.2 pin.
- Unpopped `WeightRoleNotDeclared` decline name is Unpopped-local; needs a PM/architect ruling and version before KISS mints it.

## Waiting (do not start)

- Unpopped, Baracuda, Fuel regeneration for sk6 / `list_version` 2 (their lanes; Unpopped reports 0.18.0 sk6 published).
- **Board 271 (CireSnave):** a fifth Conform modality, legal-A vs legal-B outputs MUST differ (#240; also #239 item 3 and #387's options). Build nothing on them until ruled. R3.3 second-deriver runs and R4 are [ext].
- Six sidecars record `wire_abi_schema_version` as "none stated (editor: KISS lane, pending ratification)"; the KISS lane is editor of record for all nine until ratified.
- Throttle: read `C:\Projects\THROTTLE.md` at session start and each task boundary (FULL SPEED 2026-10-21). Dependabot Monday report.

## Next actions

1. `git fetch`; `list_peers` / `check_messages`. Answer any [FINDING] by re-measuring at the sender's `origin/main` first.
2. **BEGIN the next unblocked own row, do not wait.** R3.2, R3.3, R3.9 DONE. R3.6: #148 and #400 (loud half) DONE; #484 is process (no code); **#458** (a `UNBACKED.tsv` category for a test that binds the document) has three unchosen remedies, ask the PM before building. **R3.5a** (a ULP-class `nan_provenance: moved` vector; 0 of 78 moved rows are ULP at 2921a21) needs a region-format design read first, [ASK] if it changes a normative format. R3.7 (#193 closed superseded by the derived seven-class `cell_class`; #239 items 1-2 done) waits on board 271. Unread: R3.8 (`kiss_trace` defects #136 #137 #498 #159), R3.10, R3.11, R3.12, R3.13 (check it is not already fixed by `run_with_git_bash.py`). OPS-6.18-0015 needs a c128/f64 complex evaluator. R1.10 needs a scheduled freeze; R1.11 is `[rule]`.
3. Dependabot Monday (next 2026-10-26); answer second-deriver questions on the `ctr` vectors.
