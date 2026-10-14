<!-- Written 2026-10-13 against origin/main 5b28bf5. A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** every docs/CI/test PR names the change set whose `kiss-conformance` bump covers it. The PM allocates the number
   (0.2.3 is carried by #538; 0.2.2 was #537; 0.2.1 #536; 0.2.0 #535, Breaking; 0.1.6 was #534; the PM allocates the next number at gate time). Pre-1.0 a breaking change moves the second number.
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

## Open KISS PRs

- **#539** `conformance/corpus/ops-rounding.json` (0.2.4): the first vectors for floor/ceil/trunc/round_even. After it merges: remove worktree `C:/Projects/kiss-rounding` and the branch.
- **This HANDOFF PR** (`docs/handoff-2026-10-13`): remove `C:/Projects/kiss-handoff2` and the branch once merged.

## Current state at origin/main 5b28bf5 (conformance 0.2.1)

- **#263 ruled D and landed (#535, #536):** `structure_key` is `sk5`, field 1 is the **derived** 7-class `cell_class`
  (`ctr ndx rdc ew1 ew2 ew3 oth`, Classify §6.5-0015); no op→family table exists; Contract `contract_version` 3
  (`math_fidelity`); `dequant_form` keyed as the optional sixth sub-key component; extent-1 mask fix; keepdim reduction
  outputs derive masks `02/03/01/0a`. Design and the 24→7 accounting: `docs/design-263-cell-class.md`. `CHANGELOG.md` has 0.2.0/0.2.1.
- **kiss-ref 0.4.0 is published** (crates.io, from kiss-ref `2583d5ef`, bound to KISS `a33282c`). Do not touch kiss-ref without a PM task.
- **Followers** (PM-issued, theirs): Unpopped keyer, Fuel deriver (bound `sk5`), Baracuda (still `sk4` in production), Vulkane (no lane).
- **Roadmap:** `ROADMAP.md` (on main since #537) is the ordered list. Take the next **[go]** item without asking; never start a **[rule]** item.

## Held / waiting (do not start)

- **R1.1 Announce op identity in every lookup** (`docs/design-263-cell-class.md` §6): own PR, own `list_version`; **timing is CireSnave's**.
- **#222 candidate 1** (quantized `ctr` scale dtype keying) and **VariantFidelity's home**: need a ruling.
- Dependabot Monday reports; Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first.

## Next actions

1. R2.3: NaN vectors for the signed-zero corpus (#329) (needs a build; check C: free space first).
2. R3.1/R3.2: freeze-gate instrument fixes (#387, #524, #144) and the Classify traceability tail (39 untraced clauses).
3. R0.2 close-list is ready in ROADMAP §6; closing is CireSnave's. R1.1/R1.2/R1.5 and #462 are boarded (PM item 193): do not start.
