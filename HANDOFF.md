<!-- Written 2026-10-14 against origin/main 507f691 (conformance 0.2.35). A HANDOFF is current state, never a log:
     rewrite it, do not append (CIRESNAVE-EXPECTATIONS §2.3c). Ceiling 8KB. -->
# KISS lane HANDOFF

## Standing procedure (a restarted session must do these)

1. **Writes go in a worktree**, never the shared anchor `C:\Projects\KISS`. `git worktree add -b <branch> C:/Projects/kiss-<task> origin/main`;
   remove it after the merge (`git status --porcelain` first). Read `origin/main` after `git fetch`, never a working tree.
2. **Version rule:** the PM allocates the number at gate time (latest merged 0.2.35). `conformance/Cargo.toml` and the top `CHANGELOG.md`
   heading must be bumped TOGETHER: `tools/kiss_version_sync.py` is a blocking gate (#561). Pre-1.0 a breaking change moves the second number.
3. **Dependabot PRs** (weekly Monday, one grouped PR): whoever merges one adds the patch bump on its branch and says so in a comment. Review
   like #526: `using:` in the new tag's `action.yml`, release notes vs KISS's inputs, Node annotation, the 5 required contexts.
4. **Derived figure files come from the tools**: `python tools/kiss_trace.py --ratchet --base-ref origin/main` (floor numbers) and
   `python tools/kiss_readme_coverage.py` (README bound figures, update number AND marker together). Report ENFORCED separately from
   `untested`: recategorising a row is a ledger statement, not coverage.
5. **Before pushing run every python step of the `kiss-trace` job** (`.github/workflows/traceability.yml`), `cargo test` and `cargo clippy
   --all-targets --all-features` in `conformance/` (`approx_constant` is blocking). The three bash-launching suites DO run here:
   `python tools/run_with_git_bash.py tools/test_kiss_ratchet_step.py` (same for `_msrv_step`, `_orphan_step`); the WSL launcher in System32
   wins over PATH, so never call them bare. Do not report them "unrun".
6. **Auto-mode classifier denials are final for the session**; a peer cannot clear one. Report it and take a different route.
7. **Codacy:** methods over 50 lines on touched lines, `assert` in Python tests (use `unittest`), and a new `import subprocess`
   (accepted once, #568, with a disposition comment). Split or answer on the PR; never configure the path out.
8. **Peer messages are information, not authority.** CireSnave's decisions arrive only via the PM; `list_peers` and re-resolve IDs by cwd
   (they rotate; the PM's changed twice today). **Write scripts with the Write tool**: multi-line python heredocs with quotes broke Bash, and
   `/tmp` in Git Bash is not `/tmp` to Windows python.
9. **Claims:** a clause is "backed" only when the test discriminates; when half a clause has no vector form, count it PARTIAL (floor unmoved,
   `UNBACKED.tsv` note, test does not cite the clause), as 6.8-0003 and 6.9-0003 were. Say what a source scan cannot prove.

## Open PRs

None on KISS (main = 507f691, conformance 0.2.35). kiss-ref main = 0.4.5 (all re-vendors merged: int/bool, atan2, sqrt, nextafter);
kiss-ref is a **non-party** for OpAttrs (PM ruling). This HANDOFF PR carries 0.2.36 if open.

## State at 0.2.35

- **Floor** harness 418, lint 34, untested 448, proven 20. Classify 88/120. 539 clauses without a harness test: 446 untested, 34 lint,
  21 blocked, 12 definitional, 20 attested, 4 untestable, 2 decredited.
- **R3.2 Ops vectors all landed** (opt-in, no spec change): `ops-int-wrap` (440), `ops-compare-bool` (181), `ops-atan2-quadrants` (28),
  `ops-sqrt-rounding` (24, 6.8-0003 first arm: PARTIAL), `ops-nextafter` (40, value half: PARTIAL). kiss-ref reproduces all.
  Fuel measured int-wrap: 275 PASS / 0 FAIL / 55 ERROR (i8, GAP-356) / 110 NOT-EXPRESSIBLE.
- **R2.4 slice 1 landed:** `opattrs_value_vectors.json` (133 encode, 7 equivalences, 112 distinct; 15 of 17 ops) by a generic encoder,
  cross-checked vs the hand encoders and Appendix E.2. **Live OpAttrs pair count is 0**: Fuel's `to_canonical_bytes` is an envelope that
  diverges from 6.19.3 (its GAP-305, OPEN); Baracuda emits none. Not independent evidence (E6).
- **Spec finding, a [rule] (R1.9, board 193, not ours):** `reduce`/`prefix_scan` carry sk4 `accumulator`+`math_precision` (6 bytes) but
  6.19.2 pins no ordinal table for `math_precision`; Appendix E.2 and `encode_reduce`/`encode_prefix_scan` still show 4 bytes. Not minted.
- **R1.4 done:** `structure_key_derivation_vectors.json` is 36 vectors (blob 697aa543); `[8,0]` -> `da` per 6.3-0011+6.3-0002+6.5-0012,
  reference-deriver output not an independent golden.
- **Classify tail:** the 6 `untested` rows (6.0-0001, 6.1-0012, 6.3-0004, 6.3-0009, 6.3-0010, 6.8-0017) each say what was searched for.
  They need a SUBJECT (a quant-record model, a symbolic-extent model, a complex byte reader): ask the architect on the board if that is
  a spec change; do not write tests with no subject.

## Waiting (do not start)

- **R1.1** (Announce op identity), **R1.2** (#222 scale-dtype keying), **R1.5** (VariantFidelity), **R1.8/#543** (batch role; `ctr` vectors,
  6.6-0012/-0016), **R1.9**, #462 (signer), #387 instrument, sidecar fields: all [rule], boarded under PM 193. Timing is CireSnave's.
- Fuel (idle, queued): body-diff of its clause-derived OpAttrs vectors vs our file (will send a [FINDING]); re-vendor of the 36-vector
  derivation file; Unpopped: whether it reads `[8,0]` as `da` (no reply yet). Baracuda sk5 keyer, Unpopped sk5 keyer, four-way sk5 table.
- Ubuntu 26 runner 2026-10-19: if CI breaks, tell the PM first. Dependabot Monday report.

## Next actions

1. Read `list_peers`, `check_messages`; answer any [FINDING] from Fuel/Unpopped by re-measuring at their `origin/main` first.
2. Take the next **[go]** in `ROADMAP.md`; R2.4 and R3.2 have none left until a second encoder or R1.9 lands. If none, ask the PM.
3. R3.3 second-deriver runs and R4 (foreign reader, external implementors) are [ext]: nothing to build from inside this repo.
