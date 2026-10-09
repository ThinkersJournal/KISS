# Implementation differentials against KISS — results, 2026-10-09

**Non-normative. Measured, not forecast.** KISS `origin/main` `904a4b4`. Implementations at `origin/main` of
their own repos: Fuel `36302fe5`, Unpopped `a025fc7`, Baracuda `62c4cde2` (each in a fresh detached worktree;
sccache on; nothing in those repos was committed or edited except the substitution named in §2).

## 0. Read this first: none of these instruments reads KISS `main`

The request was to run each implementation's differential "against current KISS main". They cannot be, as built.
Each reads a **frozen copy** of KISS, and the copies are of different ages. What each one passes is therefore a
statement about *its pinned copy*, not about the spec as it stands.

| Implementation | What it reads | Pinned copy vs KISS main (git blob compare) |
|---|---|---|
| Fuel | `fuel-dispatch/fixtures/kiss-corpus/` vendored from KISS `f4952b4c` (2026-09-02) | `ops-arith`, `ops-minmax-ordinary`, `ops-minmax-signed-zero`: **same**. `structure_key_vectors`, `dtype_manifest`, `op_manifest`: **stale** |
| Unpopped | `crates/unpopped-vocab/kiss/` re-vendored 2026-10-02 from KISS `bc16715` | `dtype_manifest`, `structure_key_vectors`: **same** as main |
| Unpopped / Baracuda reference evaluator | `kiss-ref-core`, `kiss-ops-vocab`, `kiss-classify-vocab` **0.3.4** from crates.io (published 2026-09-05; the kiss-ref repo has no commit after 2026-09-16) | not blob-comparable. It predates KISS #516–#519 (2026-10-01/02) by date; **not diffed** |
| Baracuda `baracuda-cuda-vocab` | `kiss-cuda-seed.tsv` pinned at KISS `becf90fc` (2026-08-26): **4 rows** (`Sm80 Sm89 Sm90 Sm90a`) | `spec/namespaces/cuda.md` now carries a **13-row token set** and a 4-row dispatch set (#521, #522): the pinned seed is **stale**, the 4-member manifest is **not** a gap (see §3) |

## 1. Results as run (each against its own pinned copy)

| Instrument | Command (in the implementation's worktree) | Result |
|---|---|---|
| Fuel structure_key byte-match, `KISS-CLASSIFY-6.7` token grammar | `cargo test -p fuel-dispatch --features telemetry --test kiss_structure_key_byte_match` | **7 passed, 0 failed** |
| Fuel KISS clause ledger | `… --test kiss_clause_ledger` | **9 passed** — a ledger of *Fuel's declared* obligation per cited clause; it does not read KISS `main` |
| Fuel signed-zero/tie conformance | `… --test kiss_signed_zero_tie_conformance` | **4 passed** |
| Unpopped structure_key byte-match + decline vectors | `cargo test -p unpopped-vocab --test kiss_byte_match` | **12 passed** |
| Unpopped dtype manifest | `… --test kiss_dtype_manifest` | **6 passed** |
| Unpopped in-tree kiss-ref differential (CPU) | `cargo test -p unpopped --lib kiss_ref_diff` | **31 passed** (of 449 in the crate; 418 filtered out by name) |
| Baracuda `tools/kiss-ref-diff` incl. **on-device legs** | `cargo run --manifest-path tools/kiss-ref-diff/Cargo.toml` | **exit 0, every step OK.** Device legs ran on an RTX 4070 Laptop, `compute_89`: elementwise (3) and block-tree folds (`sum`, `max`) §6.8-exact vs kiss-ref; CPU legs: elementwise, folds, `matmul`, `prefix_scan`, `gather`, `scatter`, `bincount` bit-identical |
| Baracuda `baracuda-cuda-vocab` | `cargo test -p baracuda-cuda-vocab` | **16 passed** (against the 4-row seed) |

The tool prints "kiss-ref 0.1.0" in a banner; the dependency actually resolved is `0.3.4` (its `Cargo.toml`).
The tool has no lockfile (`Locking 36 packages to latest compatible versions`), so a rerun can resolve differently.

## 2. The one run that does touch current KISS: Fuel's byte-match with main's vectors swapped in

In a throwaway Fuel worktree only, KISS main's `structure_key_vectors.json`, `dtype_manifest.json` and
`op_manifest.json` replaced Fuel's vendored copies, then `kiss_structure_key_byte_match` was rerun.

| Test | Result |
|---|---|
| `positive_vectors_byte_match` | **pass** — Fuel's deriver reproduces main's 21 positive vectors byte for byte |
| `redundant_acc_mp_is_unrepresentable_on_the_emit_path`, `fuel_never_emits_a_published_decline_token`, `constructed_and_excluded_partition_the_positive_vectors`, `the_i4_exclusion_still_has_its_reason` | pass |
| `corpus_is_the_artifact_this_leg_was_bound_to` | **fail** — expects `usable_count` 22; main has 21. This is the test doing its job (the artifact it was bound to moved); not itself a conformance failure |
| `fuel_emits_only_recognized_sk4_dtype_spellings` | **fail — a real divergence.** Fuel's `fuel_ir::sk4_token(DType::F8E6M2)` emits `f8e6m2`; KISS #517 made `f8e6m2` **reserved** (KISS-CLASSIFY-6.1-0013: recognized on parse, typed-declines, never emitted). |

Not rerun against main's artifacts: the other Fuel tests, and Unpopped (its copies already equal main's, so §1 *is* the
main result for those two tests).

## 3. What this cannot say

- **Not a per-clause conformance verdict for any implementation.** Only the Fuel and Unpopped byte-match legs are
  clause-keyed (to `KISS-CLASSIFY-6.7`/`-6.1`); the kiss-ref differentials are keyed to ops and comparators
  (§6.8, §6.11-0005), not to clause ids. KISS's own matrix counts 479 untested MUSTs; none of this changes it.
- **Nothing here exercises #516 (contract_version 2), #518 (precision-class tokens) or the Contract/Ops changes**: the
  evaluator is 0.3.4, and no implementation instrument reads Contract documents.
- **Retracted 2026-10-09 (the original bullet here called the 4-member vs 13-row difference an unverified possible
  gap).** Baracuda answered: its `cuda:` manifest is the closed `ArchSku` set of CUTLASS-dispatch SKUs
  (`sm80 sm89 sm90 sm90a`) by design; the nine NVRTC-only tokens live in KISS's own `spec/namespaces/cuda.md`
  (#522) and were deliberately moved onto the open `TargetId` (baracuda#151). The subset is correct and **is not a gap**.
  What remains true is only that its vendored seed (`becf90fc`, 4 rows) predates the two-block annex, so its
  agreement gate cannot see later annex edits. Basis: the Baracuda lane's reply, relayed by the PM; I did not read
  their code for it.
- The device legs cover 3 elementwise kernels and 2 folds on one GPU architecture (`sm_89`).
- Fuel's vendored `ops-*` corpus is current for the three files it uses, but main has seven further corpus files
  (`announce`, `contract`, `grammar`, `opattrs`, `ops-narrow-*`, `ops-transcendental-nan`) that none of these
  instruments read.

## 4. Reproduce

```sh
git worktree add --detach C:/Projects/kiss-run-<repo> origin/main     # in the implementation repo
RUSTC_WRAPPER=sccache CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=<dir> cargo test …   # commands in §1
# §2: copy conformance/corpus/{structure_key_vectors,dtype_manifest,op_manifest}.json from KISS origin/main
#     over fuel-dispatch/fixtures/kiss-corpus/ (worktree only), rerun the byte-match test.
```
