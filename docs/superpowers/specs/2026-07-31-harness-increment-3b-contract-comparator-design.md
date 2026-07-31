# Harness increment 3b — Contract-sourced comparator selection + advertisement honesty

**Date:** 2026-07-31
**Branch:** `feat/harness-comparator-select` (worktree `C:/Projects/kiss-3b`), stacked on `feat/harness-axis-reduce` (3a).
**Backs:** KISS-CONFORM-6.13-0006b, KISS-OPS-7.4-0001.
**Prerequisite folded in:** #64 (`determinism.rs::atom_determinism_class` panic → `Option`).

## Problem

3a's differential harness selects the reduction comparator **structurally**: `compare_monoid_reduced_f32(monoid, …)` hardcodes `Sum/Prod → order-invariant band`, `Max/Min → exact-byte`. KISS-CONFORM-6.13-0006b forbids exactly that:

> KISS-Conform MUST select the differential comparator from the op's **advertised per-op determinism/fidelity class** (KISS-OPS §7.4-0001), **never a hardcoded comparator**, so the comparator matches the class the implementation advertises.

The class an implementation advertises travels with the artifact in its **Contract Guarantees** block (SYNTH §6.5-0004; contract §6.8-0003, imported from the canonical KISS-Ops §6.0 enum). 3b makes the harness read that advertised class from a **real Contract**, select the comparator from it, and reject an advertisement that is dishonestly permissive.

## Reuse, not rebuild

`structural::compare_reduced_f32(class, actual, expected, abs_tol, rel_tol)` already dispatches the comparator **from a `DeterminismClass`** (cited to §6.8-0006, already backed by `test_conform_comparator_selection_rule`). 3b does **not** re-implement comparison — it *sources the class correctly* and feeds this function. The delta from 3a is where the class comes from: parsed advertisement, not the monoid.

## Design — six units

### Unit 1 — #64 fix (prerequisite)
`atom_determinism_class(op) -> Option<DeterminismClass>` (`None` for a fold/conditional or unknown token, instead of `panic!`). Update `determinism.rs` tests. Moves no clause; unblocks iterating a mixed op set without a loud misuse panic. *Own commit.*

### Unit 2 — fold-aware class oracle (the honesty truth-source)
New `op_true_class(op: &str, monoid: Option<Monoid>) -> DeterminismClass` computing an op's **true** class per KISS-OPS §6.0:
- unconditional scalar atom → `atom_determinism_class` (now `Option`); 
- float `Sum`/`Prod` fold → `OrderInvariant` (§6.0-0004 — result depends on a float sum);
- `Max`/`Min` reduce → `ExactByte`;
- transcendental atom → `UlpTolerance` (§6.0-0003);
- multi-class resolved by the §6.0-0005 **most-permissive** order `exact-byte < ULP < order-invariant`.

This is the truth the honesty check (unit 5) compares the advertisement against. Lives in `determinism.rs` (extends the atom classifier with the fold arm). Scope: the ops the harness actually differences (elementwise + reduce monoids), not a complete §6.0 classifier for every op — the covered set is `log`-noted, no silent cap.

### Unit 3 — Contract Guarantees parser
Extend the contract codec (`contract.rs`) to read the **Guarantees block's `determinism_class` field** (contract §6.8-0003) from a real `Document`, mapping the wire token to the canonical crate-root `DeterminismClass`. The test constructs a **real** contract with a Guarantees block (existing `render_block`/`encode`), round-trips the bytes, and parses the class back — real codec, real bytes, no external fixture files. Backs **KISS-OPS-7.4-0001** via `test_ops_determinism_class_advertised`: the advertisement is drawn from the single canonical §6.0 enum and is **not re-forked** (a parse of any token outside the canonical enum is a typed decline, never a parallel class).

### Unit 4 — comparator-selection wiring (6.13-0006b crux)
The differential reads the **parsed advertised** class and dispatches via `compare_reduced_f32`, replacing 3a's structural path for this test. Backs **KISS-CONFORM-6.13-0006b** via `test_conform_ops_class_comparator_selection`.

**Crux property:** two candidate ops with the **same monoid but different advertised classes select different comparators.** Concretely — a `Sum` reduce advertised `order-invariant` gets the band comparator (a legitimately reassociated result is accepted); the *same* `Sum` reduce advertised `exact-byte` gets the byte comparator (the reassociated result is now *rejected*). Same op, same monoid, opposite verdict — driven only by the advertisement. That is what proves selection is advertisement-sourced, not hardcoded by monoid.

### Unit 5 — advertisement-honesty check
Reject an advertisement **strictly more permissive than** `op_true_class` (the §6.0-0005 order): that is the direction that selects a comparator too loose to catch a real error (e.g. a `Max` reduce advertised `order-invariant` to buy tolerance slack a genuinely-wrong Max could hide behind). This is a hard reject — a typed decline of the contract, not a silent pass.

The **over**-claim direction (advertising `exact-byte` for a fold that is truly `order-invariant`) is **not** separately linted here (per ruling): it is caught by the differential itself — an honest impl cannot meet a byte-exact claim across two independently-rounded orders — and is separately forbidden by SYNTH §6.5-0004b (a provider MUST NOT assert byte identity for a non-exact-byte op). 3b asserts both directions behave correctly (too-permissive → rejected by the honesty check; too-strict → caught by the differential), but only the too-permissive lint is new machinery.

### Unit 6 — tests + honest ledger
Bind **6.13-0006b** + **7.4-0001** by reverse-citation (a `//` comment inside each test fn scope citing the clause ID, per the §6.1 binding the trace tool reads — the same honest-accounting discipline as 3a). Do **not** claim the SYNTH §6.5-0004* clauses or §6.8-0006 unless a test genuinely exercises each (§6.8-0006 is already backed). Trace stays CLEAN; the genuinely-untested count drops by exactly what is earned (target **534 → 532**).

## Boundaries (what 3b does *not* do)
- No full contract *validation* pipeline — only the Guarantees `determinism_class` field parse.
- `op_true_class` covers the differenced op set (elementwise + reduce monoids), not every §6.0 op; the covered set is logged.
- Softmax / transcendental split comparator stays deferred to #67.

## Mechanics
- Stacks on `feat/harness-axis-reduce`; rebase onto `main` once 3a (PR #118) merges.
- `#![cfg(windows)]` on any loader/`cl.exe`-dependent test (ubuntu leg compiles them out; windows-latest is the sole evidence for those clauses).
- stdlib-only, no `build.rs`, no crate deps.
- Commit trailers exactly as the branch uses: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>` + `Claude-Session:` line.
- Subagent-driven TDD: sonnet implementers per task, opus whole-branch review before finish. Push/PR/merge held for explicit authorization.

## Clause map
| Clause | Test | State after 3b |
|---|---|---|
| KISS-CONFORM-6.13-0006b | `test_conform_ops_class_comparator_selection` | backed (unit 4) |
| KISS-OPS-7.4-0001 | `test_ops_determinism_class_advertised` | backed (unit 3) |
| #64 (determinism.rs) | its own tests | refactor, no clause |
| SYNTH §6.5-0004* | — | exercised incidentally; **not claimed** unless a test earns it |
| KISS-CONFORM-6.8-0006 | `test_conform_comparator_selection_rule` | already backed; reused, not re-claimed |
