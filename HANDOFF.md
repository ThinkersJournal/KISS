<!-- Written 2026-10-16 against origin/main 151e748 (conformance 0.4.1); the batch-role PR proposes 0.4.2. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first, then `git worktree prune`). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** the PM allocates the number at gate time (latest merged 0.4.1; the batch-role PR proposes 0.4.2; no docs exception). `conformance/Cargo.toml` and the top
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

## Open PR

**Batch role (R1.8 / KISS#543)**, branch `kiss-batch-role`, worktree `C:/Projects/kiss-batch`. PM ruled option A 2026-10-16 ("A ... no sk bump so no
regeneration; nothing is frozen so reversible"; pins in the #543 comment 6218238118). KISS-CLASSIFY-6.6-0023: batch axes are a declared role hint, batched
iff non-empty, `b<class>` from the product of the batch extents, malformed hints decline. Code `derive_batch_class`; test `test_classify_batch_role_derivation`;
floor harness 434. **After it merges:** tell Unpopped (its M/N/K roles were blocked on it; it declares B too) and Fuel (axes instead of an extent), remove
the worktree (`git status --porcelain` first), delete the branch and verify ABSENT with a positive control (read the gate's branch line; the PM once
reported "deleted" without it), then restart.

## State at 0.4.1 (origin/main 151e748)

- **#588 merged:** `structure_key` `sk5`->`sk6` (scale dtype keyed for quantized `ctr`, §6.6-0022) + Announce `list_version` 2 (`op_identity`,
  §6.3-0013, Synth mirrored). Per-vector old/new: `docs/sk6-vector-changes.md`. `envelope_version` 1 and `contract_version` 3 unmoved. Nothing frozen.
  KISS publish is the PM's; kiss-conformance is `publish = false`, no other crate. The PM notifies Unpopped, Baracuda, Fuel (CHANGELOG 0.4.0 lists what each regenerates).
- **Next own row after the batch PR: build the reference `ctr` derivation stage (R3.3)** in `derive.rs` and its vectors, including a rank-4 batched cell;
  it replaces `decline_contraction_out_of_scope`. Not started.
- **Open item, KISS lane owns: ROADMAP R1.10 "Frame versioning before freeze".** CYRQ/CRSP/CDEC carry no version field (PM ruling: no bump now, since all
  speakers regenerate together); must be resolved before any freeze. Detector: the freeze checklist. Not startable until a freeze is scheduled.
- **Floor** harness 433, lint 35, untested 421 (tool, base origin/main). Re-measure with `python tools/kiss_trace.py --report`.
- **PARTIAL, floor unmoved:** 6.9-0003 (no emitter/lifter), 6.6-0001 (roles/ranks/dtypes/OpAttrs sub-vocabularies not generated),
  6.13-0008, 6.13-0009. 6.6-0002/-0003/-0004/-0006/-0007 are `blocked` (need a suite emitter and lifter).
- **Board 201 pin:** `math_fidelity` u8 {0 reserved, 1 bit-stable, 2 reduced-mantissa}; accumulator 0x00 or 1-based dtype ordinal;
  Appendix E.2 six-byte reduce/scan forms; 167 generic vectors, `not_emitted` empty.
- **GAP-305** (Fuel's OpAttrs encoder vs 6.19.3) is owned by the fuel lane; it follows our 6.19.2 pin.
- Unpopped `WeightRoleNotDeclared` decline name is Unpopped-local; needs a PM/architect ruling and version before KISS mints it.

## Waiting (do not start)

- Unpopped, Baracuda, Fuel regeneration for sk6 / `list_version` 2 (their lanes; Unpopped reports 0.18.0 sk6 published).
- Still [rule]: #387 instrument. R3.3 second-deriver runs and R4 (foreign reader, external implementors) are [ext].
- Six sidecars record `wire_abi_schema_version` as "none stated (editor: KISS lane, pending ratification)"; the KISS lane is editor of record for all nine until ratified.
- Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first. Dependabot Monday report.

## Next actions

1. `git fetch`; `list_peers` / `check_messages`. Answer any [FINDING] by re-measuring at the sender's `origin/main` first.
2. Drive the batch-role PR (checks, threads, `[READY]` to the PM). Once merged, the `ctr` reference stage is the next unblocked own row; start it and send
   `[READY]` (do not wait for a [TASK]).
