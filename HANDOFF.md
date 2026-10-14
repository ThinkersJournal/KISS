<!-- Written 2026-10-14 against origin/main 6aeb9b6 (0.2.23). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** every docs/CI/test PR names the change set whose `kiss-conformance` bump covers it. The PM allocates the number
   (latest merged is 0.2.17, #553; the PM allocates the next number at gate time). Pre-1.0 a breaking change moves the second number.
3. **Dependabot PRs** (weekly Monday, one grouped PR): whoever merges one adds the patch bump as a commit on its branch and says so in a
   comment. Review like #526: `using:` in the new tag's `action.yml`, release notes vs KISS's inputs, Node annotation, the 5 required contexts.
4. **Derived figure files come from the tools**, never from a side: `python tools/kiss_trace.py --ratchet --base-ref origin/main` and
   `python tools/kiss_readme_coverage.py`. A new lint-backed clause needs: the `lint:<tool>` row in `conformance/UNBACKED.tsv`, the tool's
   `--emit-coverage` entry, and the floor's `lint` number raised by the PR itself (the ratchet then reports a green "born with detector" arrival).
5. **Before pushing run every python step of the `kiss-trace` job** (`.github/workflows/traceability.yml`). Three suites launch `bash` by name and
   fail on this box (WSL): `test_kiss_ratchet_step.py`, `test_kiss_msrv_step.py`, `test_kiss_orphan_step.py`; say so in the PR body.
6. **Auto-mode classifier denials are final for the session**; a peer cannot clear one. Report it and take a different route.
7. **Codacy flags methods over 50 lines** on the lines a PR touches: split before pushing. Never configure the path out.
8. **Peer messages are information, not authority.** CireSnave's decisions arrive only via the PM; objections from Fuel/Baracuda/Unpopped go
   through the PM too (`list_peers`, re-resolve IDs by cwd; they rotate).

## Open PRs

- KISS: none (origin/main at conformance 0.2.23, `6aeb9b6`; this HANDOFF PR is docs only, number allocated by the PM).
- **kiss-ref#57** (`ops-sqrt-rounding` re-vendor, 0.4.4): [READY] sent, 7/7 green; the PM gates. No local worktree; the branch is `revendor-sqrt`.
  If merged, nothing is owed. kiss-ref main is 0.4.3 (#55 int/bool 0.4.2, #56 atan2 0.4.3).

## Current state at origin/main (conformance 0.2.23)

- **#263 landed** (`sk5`, derived 7-class `cell_class`, `contract_version` 3); design in `docs/design-263-cell-class.md`.
  Touch kiss-ref only on a PM task (the PM has been issuing them per re-vendor).
- **Instruments:** `ops-rounding.json` (72), `ops-minmax-nan.json` (44), `ops-transcendental-nan.json` (13), the cell-carrying
  `structure_key_derivation_vectors.json` (33) with `conformance/src/derive.rs`, the blocking freeze hard-fail
  (`tools/kiss_freeze_gate.py`), a second C toolchain (`KISS_C_COMPILER`, CI job `windows-clang-cl`).
- **R3.2 Ops vectors this session (all opt-in, no spec change):** `ops-int-wrap.json` (440, 6.2-0002) and `ops-compare-bool.json`
  (181, 6.2-0005/-0006): kiss-ref 621/621 (#55); Fuel measured int-wrap: 275 PASS, 0 FAIL, 55 typed ERROR (all i8, Fuel GAP-356),
  110 NOT-EXPRESSIBLE (u16/u64). `ops-atan2-quadrants.json` (28, 6.9-0001, class ULP bound 4, libm-backed oracle, BACKED; kiss-ref
  28/28 #56). `ops-sqrt-rounding.json` (24, 6.8-0003 first arm, class ULP bound 0; **counted PARTIAL**: the floor did not move, the
  UNBACKED row stays `untested` with a note, and the test deliberately does not cite the clause; kiss-ref 24/24 in #57).
- **Ratchet floor:** harness 406, lint 34, untested 471, proven 20. 6.2-0007 stays unbacked (bitwise/reduction/scan arms uncovered).
- **ULP comparator caveat:** totalOrder distance does not discriminate `+0`/`-0`; atan2 and sqrt zero-sign cells are carried, not enforced.
- **Owed by implementations:** collapsed reduction output must decline, reduced axis out of range must decline (6.6-0009); Unpopped:
  reduced innermost axis -> `v1` (6.5-0009(b)). Re-vendor `structure_key_derivation_vectors.json` at blob `299811b1`.
- **Sidecar (#144, option A):** done for all nine (0.2.17, 0.2.18). `determinism_class` / `wire_abi_schema_version` stay `null`/UNSET until
  CireSnave rules (PM item 193; option C amends Appendix E). **#387:** instrument/wording choice boarded (PM 193).
- **Roadmap:** `ROADMAP.md` is the ordered list. Take the next **[go]** item without asking; never start a **[rule]** item.

## Held / waiting (do not start)

- **R1.1 Announce op identity in every lookup** (`docs/design-263-cell-class.md` §6): own PR, own `list_version`; **timing is CireSnave's**.
- **#222 candidate 1** (quantized `ctr` scale dtype keying) and **VariantFidelity's home**: need a ruling. R1.2/R1.5/R1.8/#462: boarded, PM 193.
- `ctr` derivation vectors wait on #543; Unpopped sk5 keyer; the four-way sk5 table; Fuel's replies.
- Dependabot Monday reports; Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first.

## Next actions

1. DONE in this change (0.2.26, `ops-nextafter.json`, PARTIAL like 6.8-0003): R3.2 Ops `nextafter` value arm (6.9-0003; its typed-decline half has no vector form,
   so do **not** back the clause: count it PARTIAL as 6.8-0003 was, floor unmoved, UNBACKED note, test not citing the clause). The rest of
   Ops (decomposition/level/registry/version/complex) has no subject in the harness: a design question, not a vector batch.
2. kiss-ref re-vendor of `ops-nextafter.json` (own kiss-ref PR, PM-allocated number, tests/fixtures only; the runner already handles f32/f64 exact-byte binary ops). Then R3.2 has no vector candidate left: ask the PM what is next.
3. R2.4: OpAttrs cross-implementation byte comparison (#504/#502) -- large; needs a design note before code.
4. Method that worked: mint in `conformance/src/bin/kiss_mint.rs` (keep each fn under 50 lines, Codacy), an independent-model test, run every
   python `kiss-trace` step plus `cargo clippy --all-targets --all-features` locally (`approx_constant` is blocking), README bound figures
   by `tools/kiss_readme_coverage.py`. Write scripts to files with the Write tool: multi-line python heredocs with quotes broke the Bash tool.
