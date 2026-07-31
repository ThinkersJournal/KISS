# Harness Increment 3b — Contract-sourced comparator selection — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the differential harness select each reduction's comparator from the op's *advertised* determinism class (parsed from a real Contract Guarantees block) instead of 3a's hardcoded structural monoid map, and reject an advertisement more permissive than the op's true class. Backs KISS-CONFORM-6.13-0006b + KISS-OPS-7.4-0001.

**Architecture:** Reuse `structural::compare_reduced_f32(class, …)` (already §6.8-0006). Add: (a) a fold-aware `op_true_class` truth oracle; (b) a Guarantees-block `determinism_class` parser on the existing text-document codec; (c) a `harness::advertised` module that runs the honesty lint then selects the comparator from the advertised class. The 6.13-0006b crux is pure Rust (a within-band reassociated result advertised two ways yields opposite verdicts), so tests run on both CI legs.

**Tech Stack:** Rust, stdlib-only (no crates, no `build.rs`). The `kiss-conformance` crate at `conformance/`.

## Global Constraints

- stdlib-only; no new crate dependencies; no `build.rs`.
- `#![cfg(windows)]` ONLY on a test that compiles/loads a C kernel via `cl.exe`. 3b's tests are pure Rust → **not** gated (they must compile and run on the ubuntu leg).
- Run `cargo` from **PowerShell** (Git-Bash's `link.exe` shadows MSVC's).
- Commit trailer EXACTLY:
  ```
  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
  ```
- Never push / open a PR / merge without explicit user authorization.
- Honest accounting: a test backs a clause only by citing its `KISS-…` ID inside the test fn's body or the contiguous `//` comment directly above `#[test]` (the trace tool's reverse-citation). Do not over-claim; run `python tools/kiss_trace.py` after each clause-binding task and keep it CLEAN.
- Worktree `C:/Projects/kiss-3b`, branch `feat/harness-comparator-select`, stacked on `feat/harness-axis-reduce`.

## File Structure

- `conformance/src/determinism.rs` — **modify**: `atom_determinism_class` → `Option` (Task 1); add `op_true_class` + the class-permissiveness ordering + `check_advertisement` (Tasks 2, 4).
- `conformance/src/contract.rs` — **modify**: add `parse_guarantees_class` reading the Guarantees block's `determinism_class` field (Task 3).
- `conformance/src/harness/advertised.rs` — **create**: `select_and_compare_reduced` (Task 5); registered in `harness/mod.rs`.
- `conformance/tests/conform_class_comparator_selection.rs` — **create**: the 6.13-0006b crux integration test (Task 5).
- `conformance/tests/ops_determinism_class_advertised.rs` — **create**: the 7.4-0001 test (Task 3).
- `conformance/UNBACKED.tsv` — **modify**: drops 6.13-0006b + 7.4-0001 (Task 6, via `--update-ledger`, then verify).

---

### Task 1: #64 — `atom_determinism_class` returns `Option`

**Files:**
- Modify: `conformance/src/determinism.rs:49-60`
- Test: `conformance/src/determinism.rs` (its `#[cfg(test)] mod tests`)

**Interfaces:**
- Produces: `pub fn atom_determinism_class(op: &str) -> Option<DeterminismClass>` — `Some(ExactByte)` for an exact-byte atom, `Some(UlpTolerance)` for a transcendental, `None` for a fold/conditional op or unknown token (was: panic).

- [ ] **Step 1: Write the failing test** (append to `mod tests`; create the module if absent)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_class_is_some_for_atoms_none_for_folds() {
        assert_eq!(atom_determinism_class("add"), Some(DeterminismClass::ExactByte));
        assert_eq!(atom_determinism_class("exp"), Some(DeterminismClass::UlpTolerance));
        // a fold/conditional op is NOT an unconditional atom → None, never a panic (#64)
        assert_eq!(atom_determinism_class("reduce"), None);
        assert_eq!(atom_determinism_class("not_an_op"), None);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run (PowerShell): `Set-Location C:\Projects\kiss-3b\conformance; cargo test --lib determinism::tests::atom_class_is_some_for_atoms_none_for_folds`
Expected: FAIL — currently `atom_determinism_class("reduce")` panics (the test aborts), or the return type is `DeterminismClass` not `Option`.

- [ ] **Step 3: Rewrite the function to return `Option`**

Replace the body of `atom_determinism_class` (and its doc comment's "Panics…" sentence):

```rust
/// The determinism/fidelity class of a scalar atom, per §6.0-0002 (exact-byte) and
/// §6.0-0003 (ULP/tolerance transcendentals). Returns `None` for an op this
/// atom-level model does not cover (a conditional/fold op, or an unknown token) —
/// a typed "not an unconditional scalar atom", never a panic (#64).
pub fn atom_determinism_class(op: &str) -> Option<DeterminismClass> {
    if TRANSCENDENTAL_ATOMS.contains(&op) {
        Some(DeterminismClass::UlpTolerance)
    } else if EXACT_BYTE_ATOMS.contains(&op) {
        Some(DeterminismClass::ExactByte)
    } else {
        None
    }
}
```

- [ ] **Step 4: Fix any existing callers**

Run: `Set-Location C:\Projects\kiss-3b\conformance; cargo build --lib` and `cargo build --tests`. If any caller consumed the old `DeterminismClass` return directly, adapt it (e.g. `.expect("unconditional atom")` at a site that is known-atom, or propagate the `Option`). Search first: `grep -rn "atom_determinism_class" conformance/` (exclude `target`).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib determinism`
Expected: PASS (new test + any pre-existing determinism tests, updated to the `Option` return).

- [ ] **Step 6: Commit**

```bash
git -C C:/Projects/kiss-3b add conformance/src/determinism.rs
git -C C:/Projects/kiss-3b commit -F - <<'MSG'
conform: determinism.rs atom_determinism_class -> Option (#64)

Return None for a fold/conditional op or unknown token instead of panicking
(Fuel's #63 cosign follow-up). Test-oracle internal; typed-decline convention,
matching shape_expr.rs. No spec change; moves no clause.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
MSG
```

---

### Task 2: fold-aware `op_true_class` truth oracle

**Files:**
- Modify: `conformance/src/determinism.rs` (add below `atom_determinism_class`)
- Test: `conformance/src/determinism.rs` `mod tests`

**Interfaces:**
- Consumes: `atom_determinism_class(op) -> Option<DeterminismClass>` (Task 1); `crate::structural::Monoid` (`Sum|Prod|Max|Min`).
- Produces: `pub fn op_true_class(op: &str, monoid: Option<crate::structural::Monoid>) -> Option<DeterminismClass>` — the op's TRUE §6.0 class. For `"reduce"`/`"prefix_scan"` with `Some(Sum|Prod)` → `OrderInvariant` (§6.0-0004); with `Some(Max|Min)` → `ExactByte`; an unconditional atom → its `atom_determinism_class`; `None` if unknown/underspecified (a fold op with `monoid: None`).

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn op_true_class_covers_atoms_and_folds() {
        use crate::structural::Monoid;
        // unconditional atom → atom class
        assert_eq!(op_true_class("add", None), Some(DeterminismClass::ExactByte));
        // float Sum/Prod fold → order-invariant (§6.0-0004)
        assert_eq!(op_true_class("reduce", Some(Monoid::Sum)), Some(DeterminismClass::OrderInvariant));
        assert_eq!(op_true_class("reduce", Some(Monoid::Prod)), Some(DeterminismClass::OrderInvariant));
        // Max/Min reduce → exact-byte (order-independent, no float fold error)
        assert_eq!(op_true_class("reduce", Some(Monoid::Max)), Some(DeterminismClass::ExactByte));
        assert_eq!(op_true_class("reduce", Some(Monoid::Min)), Some(DeterminismClass::ExactByte));
        // a fold op with no monoid is underspecified → None (not a guess)
        assert_eq!(op_true_class("reduce", None), None);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib determinism::tests::op_true_class_covers_atoms_and_folds`
Expected: FAIL — `op_true_class` not defined.

- [ ] **Step 3: Implement `op_true_class`**

```rust
/// The **true** determinism/fidelity class of an op per KISS-OPS §6.0, extending
/// the unconditional-atom oracle with the conditional fold arm the atom model
/// cannot carry (§6.0-0004/-0005). This is the truth an advertised class is
/// checked against (the honesty lint, `check_advertisement`).
///
/// Coverage is the op set the harness differences — the unconditional atoms plus
/// `reduce`/`prefix_scan` over the four monoids — not every §6.0 op. `None` means
/// "this model does not determine the class" (an unknown op, or a fold op with no
/// monoid supplied); callers treat `None` as "cannot honesty-check", never as a class.
pub fn op_true_class(
    op: &str,
    monoid: Option<crate::structural::Monoid>,
) -> Option<DeterminismClass> {
    use crate::structural::Monoid;
    // A float fold's class depends on the monoid (§6.0-0004): Sum/Prod accumulate
    // rounding order-dependently → order-invariant/nondeterministic; Max/Min are
    // selection, order-independent and exact-byte.
    if matches!(op, "reduce" | "prefix_scan") {
        return match monoid {
            Some(Monoid::Sum) | Some(Monoid::Prod) => Some(DeterminismClass::OrderInvariant),
            Some(Monoid::Max) | Some(Monoid::Min) => Some(DeterminismClass::ExactByte),
            None => None,
        };
    }
    // Otherwise it is (or is not) an unconditional scalar atom.
    atom_determinism_class(op)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib determinism::tests::op_true_class_covers_atoms_and_folds`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git -C C:/Projects/kiss-3b add conformance/src/determinism.rs
git -C C:/Projects/kiss-3b commit -F - <<'MSG'
conform: op_true_class — fold-aware determinism-class oracle (§6.0-0004/-0005)

Extends the unconditional-atom oracle with the reduce/prefix_scan fold arm
(Sum/Prod -> order-invariant, Max/Min -> exact-byte). The truth an advertised
class is checked against in 3b's honesty lint. Covers the differenced op set;
None where the model cannot determine the class.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
MSG
```

---

### Task 3: Guarantees-block `determinism_class` parser + 7.4-0001 test

**Files:**
- Modify: `conformance/src/contract.rs` (add below `read_document`)
- Create: `conformance/tests/ops_determinism_class_advertised.rs`

**Interfaces:**
- Consumes: the text-block codec (`render_block`, `field_line`, `Value::Str`, `Document`).
- Produces: `pub fn parse_guarantees_class(body: &[u8]) -> Result<crate::DeterminismClass, ContractDecline>` — scans the contract `body` for the `[section:6:guarantees]` block, reads its `determinism_class = <token>` field line, and maps the canonical KISS-Ops token to `DeterminismClass`. An absent block/field or an unrecognized token is a typed `ContractDecline` (never a panic, never a re-forked class).
- Adds `ContractDecline` variants: `MissingGuaranteesClass` and `UnknownDeterminismClass { got: String }`.

**Pin first (Step 0, no code):** confirm the three exact wire tokens and the Guarantees section id/name from `spec/contract.md` §6.8-0003 (and the golden contract in the module doc). This plan assumes `exact-byte`, `ulp/tolerance`, `order-invariant/nondeterministic` and heading `[section:6:guarantees]`; if the spec's serialized spellings differ, use the spec's and adjust the test tokens to match. The canonical enum is §6.0-0001; 7.4-0001 forbids a re-fork, so any other token MUST decline.

- [ ] **Step 1: Add the two decline variants**

In `enum ContractDecline` add:
```rust
    /// The Guarantees block has no `determinism_class` field (§6.8-0003).
    MissingGuaranteesClass,
    /// The `determinism_class` token is not a member of the canonical KISS-Ops
    /// §6.0-0001 enum — a re-forked/unknown class (§6.8-0003, KISS-OPS §7.4-0001).
    UnknownDeterminismClass { got: String },
```

- [ ] **Step 2: Write the failing test** (`conformance/tests/ops_determinism_class_advertised.rs`)

```rust
//! KISS-OPS-7.4-0001 via the contract codec: an implementation advertises, per op,
//! its determinism/fidelity class drawn from the single canonical §6.0 enum. Here
//! the advertisement rides a real Contract Guarantees block, round-tripped through
//! the codec; a token outside the canonical enum is a typed decline, never a
//! re-forked parallel class.

use kiss_conformance::contract::{
    field_line, parse_guarantees_class, ContractDecline, Value,
};
use kiss_conformance::DeterminismClass;

/// Build a minimal contract body: the pinned first heading (so it is not
/// Headingless) plus a Guarantees block carrying `determinism_class = <token>`.
fn body_with_class(token: &str) -> Vec<u8> {
    let mut b = b"[section:1:identity]\n".to_vec();
    b.extend_from_slice(b"[section:6:guarantees]\n");
    b.extend_from_slice(field_line("determinism_class", &Value::Str(token.into())).as_bytes());
    b
}

// Enforces KISS-OPS-7.4-0001: the advertised class is read from the canonical
// enum and an off-enum token is rejected, never re-forked into a new class.
#[test]
fn test_ops_determinism_class_advertised() {
    assert_eq!(
        parse_guarantees_class(&body_with_class("exact-byte")),
        Ok(DeterminismClass::ExactByte)
    );
    assert_eq!(
        parse_guarantees_class(&body_with_class("order-invariant/nondeterministic")),
        Ok(DeterminismClass::OrderInvariant)
    );
    // Off-enum token → typed decline, NOT a parallel class.
    assert!(matches!(
        parse_guarantees_class(&body_with_class("bit-exact-ish")),
        Err(ContractDecline::UnknownDeterminismClass { .. })
    ));
    // No Guarantees class field → typed decline.
    assert_eq!(
        parse_guarantees_class(b"[section:1:identity]\n"),
        Err(ContractDecline::MissingGuaranteesClass)
    );
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --test ops_determinism_class_advertised`
Expected: FAIL — `parse_guarantees_class` not defined.

- [ ] **Step 4: Implement `parse_guarantees_class`**

```rust
/// Read the Guarantees block's `determinism_class` from a contract `body` and map
/// the canonical KISS-Ops §6.0-0001 token to [`crate::DeterminismClass`]. Scans
/// the text body for the `determinism_class = <token>` field line (the Guarantees
/// block, §6.8-0003). An absent field or an off-enum token is a typed
/// [`ContractDecline`] — never a panic, never a re-forked class (KISS-OPS §7.4-0001).
pub fn parse_guarantees_class(body: &[u8]) -> Result<crate::DeterminismClass, ContractDecline> {
    let text = std::str::from_utf8(body).map_err(|_| ContractDecline::MalformedHeader)?;
    // Find the field line; the Guarantees block owns `determinism_class` (§6.8-0003).
    let token = text
        .lines()
        .find_map(|l| l.strip_prefix("determinism_class = "))
        .ok_or(ContractDecline::MissingGuaranteesClass)?;
    match token {
        "exact-byte" => Ok(crate::DeterminismClass::ExactByte),
        "ulp/tolerance" => Ok(crate::DeterminismClass::UlpTolerance),
        "order-invariant/nondeterministic" => Ok(crate::DeterminismClass::OrderInvariant),
        other => Err(ContractDecline::UnknownDeterminismClass { got: other.to_string() }),
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test ops_determinism_class_advertised` and `cargo test --lib contract`
Expected: PASS. (If Step 0 found different token spellings, both the test and the `match` arms use the spec's.)

- [ ] **Step 6: Confirm the trace binding**

Run: `python tools/kiss_trace.py`
Expected: CLEAN, exit 0; `KISS-OPS-7.4-0001` is no longer in `conformance/UNBACKED.tsv`'s untested set (now reverse-cited by `test_ops_determinism_class_advertised`). Genuinely-untested drops by 1 (534 → 533).

- [ ] **Step 7: Commit**

```bash
git -C C:/Projects/kiss-3b add conformance/src/contract.rs conformance/tests/ops_determinism_class_advertised.rs
git -C C:/Projects/kiss-3b commit -F - <<'MSG'
conform: parse Guarantees determinism_class from a real contract (7.4-0001)

parse_guarantees_class reads the canonical §6.0-0001 determinism_class token from
a contract Guarantees block (§6.8-0003) via the existing text codec; an off-enum
token or a missing field is a typed ContractDecline, never a re-forked class.
Backs KISS-OPS-7.4-0001 (test_ops_determinism_class_advertised).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
MSG
```

---

### Task 4: honesty lint — reject a too-permissive advertisement

**Files:**
- Modify: `conformance/src/determinism.rs`
- Test: `conformance/src/determinism.rs` `mod tests`

**Interfaces:**
- Produces:
  - `pub fn class_permissiveness(c: DeterminismClass) -> u8` — the §6.0-0005 order `ExactByte(0) < UlpTolerance(1) < OrderInvariant(2)` (higher = more permissive).
  - `pub fn check_advertisement(advertised: DeterminismClass, true_class: DeterminismClass) -> Result<DeterminismClass, String>` — `Ok(advertised)` when `advertised` is no more permissive than `true_class`; `Err(..)` when strictly more permissive (would select a comparator too loose to catch a real error). Over-claims (stricter than true) pass here — the differential + SYNTH §6.5-0004b handle them.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn honesty_rejects_only_too_permissive() {
        use DeterminismClass::*;
        // ordering
        assert!(class_permissiveness(ExactByte) < class_permissiveness(UlpTolerance));
        assert!(class_permissiveness(UlpTolerance) < class_permissiveness(OrderInvariant));
        // honest (advertised == true) → Ok
        assert_eq!(check_advertisement(OrderInvariant, OrderInvariant), Ok(OrderInvariant));
        // too permissive: advertise order-invariant for an exact-byte (Max) op → Err
        assert!(check_advertisement(OrderInvariant, ExactByte).is_err());
        // over-claim (stricter than true): advertise exact-byte for a Sum fold → Ok here
        // (caught by the differential + SYNTH §6.5-0004b, not this lint)
        assert_eq!(check_advertisement(ExactByte, OrderInvariant), Ok(ExactByte));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib determinism::tests::honesty_rejects_only_too_permissive`
Expected: FAIL — `class_permissiveness` / `check_advertisement` not defined.

- [ ] **Step 3: Implement both functions**

```rust
/// The §6.0-0005 permissiveness order: `exact-byte < ULP/tolerance <
/// order-invariant/nondeterministic`. A larger value admits a wider set of results.
pub fn class_permissiveness(c: DeterminismClass) -> u8 {
    match c {
        DeterminismClass::ExactByte => 0,
        DeterminismClass::UlpTolerance => 1,
        DeterminismClass::OrderInvariant => 2,
    }
}

/// The advertisement-honesty lint. Rejects an `advertised` class **strictly more
/// permissive** than the op's `true_class` — that direction selects a comparator
/// too loose to catch a real error (e.g. advertising a Max reduce as
/// order-invariant to buy tolerance a wrong Max could hide behind). An advertisement
/// no more permissive than the truth (equal, or an over-strict over-claim) passes:
/// an over-claim is caught by the differential itself and forbidden by SYNTH
/// §6.5-0004b, so it is not this lint's job (per the design ruling). Returns the
/// advertised class on success so the caller feeds it straight to the comparator.
pub fn check_advertisement(
    advertised: DeterminismClass,
    true_class: DeterminismClass,
) -> Result<DeterminismClass, String> {
    if class_permissiveness(advertised) > class_permissiveness(true_class) {
        return Err(format!(
            "dishonest advertisement: {advertised:?} is more permissive than the true \
             class {true_class:?} (§6.0-0005) — its comparator could not catch a real error"
        ));
    }
    Ok(advertised)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib determinism::tests::honesty_rejects_only_too_permissive`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git -C C:/Projects/kiss-3b add conformance/src/determinism.rs
git -C C:/Projects/kiss-3b commit -F - <<'MSG'
conform: advertisement-honesty lint — reject a too-permissive class (§6.0-0005)

check_advertisement rejects an advertised class strictly more permissive than the
op's true class (the direction that hides a real error); equal and over-strict
over-claims pass (over-claims -> differential + SYNTH §6.5-0004b, per ruling).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
MSG
```

---

### Task 5: comparator-selection wiring + the 6.13-0006b crux test

**Files:**
- Create: `conformance/src/harness/advertised.rs`
- Modify: `conformance/src/harness/mod.rs` (add `pub mod advertised;`)
- Create: `conformance/tests/conform_class_comparator_selection.rs`

**Interfaces:**
- Consumes: `crate::structural::compare_reduced_f32(class, actual, expected, abs_tol, rel_tol) -> Result<(), String>`; `crate::determinism::{op_true_class, check_advertisement}`; `crate::structural::Monoid`; `crate::DeterminismClass`.
- Produces: `pub fn select_and_compare_reduced(op: &str, monoid: Option<Monoid>, advertised: DeterminismClass, actual: f32, expected: f32, abs_tol: f32, rel_tol: f32) -> Result<(), String>` — runs the honesty lint (`check_advertisement` against `op_true_class`), then compares via the comparator the **advertised** class selects (never a monoid-hardcoded map). A too-permissive advertisement, or an op whose true class is unknown, is `Err` before any compare.

- [ ] **Step 1: Write the failing test** (`conformance/tests/conform_class_comparator_selection.rs`)

```rust
//! KISS-CONFORM-6.13-0006b: the comparator is selected from the op's ADVERTISED
//! determinism class, never hardcoded. The crux: the SAME op and monoid, advertised
//! two different ways, yield OPPOSITE verdicts on the same result — proving selection
//! follows the advertisement, not 3a's structural monoid map. Pure Rust (a real
//! within-band reassociated result), so it runs on both CI legs.

use kiss_conformance::harness::advertised::select_and_compare_reduced;
use kiss_conformance::structural::{reassoc_bound_f32, Monoid};
use kiss_conformance::DeterminismClass;

// Enforces KISS-CONFORM-6.13-0006b: comparator selected from the advertised class.
#[test]
fn test_conform_ops_class_comparator_selection() {
    // A legitimately reassociated Sum result: true sum 1e8, one order lands 1e8+8.
    let expected = 1e8f32;
    let actual = 1e8f32 + 8.0; // within the reassociation band for ~16 addends @1e8
    let abs_tol = 2.0 * reassoc_bound_f32(16, 1e8); // ~179 » 8
    let rel_tol = 0.0;

    // Advertised order-invariant (the TRUE class of a Sum fold) → band comparator → ACCEPT.
    assert!(select_and_compare_reduced(
        "reduce", Some(Monoid::Sum), DeterminismClass::OrderInvariant,
        actual, expected, abs_tol, rel_tol,
    ).is_ok());

    // SAME op, SAME monoid, SAME result — advertised exact-byte → byte comparator → REJECT.
    // (An over-strict over-claim: the honesty lint permits it; the differential exposes it.)
    assert!(select_and_compare_reduced(
        "reduce", Some(Monoid::Sum), DeterminismClass::ExactByte,
        actual, expected, abs_tol, rel_tol,
    ).is_err());

    // Too-permissive advertisement is rejected before any compare: advertise a Max
    // reduce (true class exact-byte) as order-invariant → honesty lint Err.
    assert!(select_and_compare_reduced(
        "reduce", Some(Monoid::Max), DeterminismClass::OrderInvariant,
        1.0, 1.0, 0.0, 0.0,
    ).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test conform_class_comparator_selection`
Expected: FAIL — `harness::advertised::select_and_compare_reduced` not defined.

- [ ] **Step 3: Create the module + register it**

In `conformance/src/harness/mod.rs` add `pub mod advertised;` (follow the existing `pub mod …;` ordering).

Create `conformance/src/harness/advertised.rs`:
```rust
//! Contract-sourced comparator selection (KISS-CONFORM-6.13-0006b): select the
//! differential comparator from the op's advertised determinism class, never a
//! hardcoded structural map. The advertised class is honesty-checked against the
//! op's true class (§6.0-0005) before it drives the comparator.

use crate::determinism::{check_advertisement, op_true_class};
use crate::structural::{compare_reduced_f32, Monoid};
use crate::DeterminismClass;

/// Honesty-check the `advertised` class against the op's true class, then compare
/// `actual` vs `expected` with the comparator that **advertised** class selects
/// (via [`compare_reduced_f32`]). This is the 6.13-0006b path: the comparator is a
/// function of the advertisement, not of the monoid. Errors (before any compare) if
/// the op's true class is unknown, or the advertisement is too permissive (§6.0-0005).
pub fn select_and_compare_reduced(
    op: &str,
    monoid: Option<Monoid>,
    advertised: DeterminismClass,
    actual: f32,
    expected: f32,
    abs_tol: f32,
    rel_tol: f32,
) -> Result<(), String> {
    let true_class = op_true_class(op, monoid)
        .ok_or_else(|| format!("cannot honesty-check `{op}` (monoid {monoid:?}): true class unknown"))?;
    let selected = check_advertisement(advertised, true_class)?;
    compare_reduced_f32(selected, actual, expected, abs_tol, rel_tol)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test conform_class_comparator_selection`
Expected: PASS (all three assertions).

- [ ] **Step 5: Confirm the trace binding**

Run: `python tools/kiss_trace.py`
Expected: CLEAN, exit 0; `KISS-CONFORM-6.13-0006b` now reverse-cited by `test_conform_ops_class_comparator_selection`. Genuinely-untested drops by 1 (533 → 532).

- [ ] **Step 6: Commit**

```bash
git -C C:/Projects/kiss-3b add conformance/src/harness/advertised.rs conformance/src/harness/mod.rs conformance/tests/conform_class_comparator_selection.rs
git -C C:/Projects/kiss-3b commit -F - <<'MSG'
conform: select comparator from the advertised class (6.13-0006b)

harness::advertised::select_and_compare_reduced honesty-checks the advertised
class (§6.0-0005) then dispatches via compare_reduced_f32 — the comparator is a
function of the ADVERTISEMENT, not the monoid. Crux test: same reduce+Sum, same
result, advertised order-invariant -> accept vs exact-byte -> reject; a
too-permissive Max advertisement is rejected before compare. Pure Rust (both CI
legs). Backs KISS-CONFORM-6.13-0006b.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
MSG
```

---

### Task 6: full-suite gate, ledger reconciliation, honest accounting

**Files:**
- Modify: `conformance/UNBACKED.tsv` (via the trace tool)

- [ ] **Step 1: Full suite + Linux cross-check**

Run (PowerShell): `Set-Location C:\Projects\kiss-3b\conformance; cargo test`
Expected: all green, exit 0.
Run: `cargo check --target x86_64-unknown-linux-gnu --tests`
Expected: exit 0 — 3b's tests are pure Rust and MUST compile on the ubuntu leg (no `#![cfg(windows)]`).

- [ ] **Step 2: Reconcile the ledger**

Run: `python tools/kiss_trace.py` — confirm CLEAN and genuinely-untested is **532**.
If the tool reports STALE for 6.13-0006b or 7.4-0001 (now backed), run: `python tools/kiss_trace.py --update-ledger` and inspect: `git -C C:/Projects/kiss-3b diff conformance/UNBACKED.tsv` shows ONLY those two clauses removed. Do NOT let any SYNTH §6.5-0004* clause change state — if one did, a test over-claimed it; fix the citation.

- [ ] **Step 3: Run the lint tools**

Run: `python tools/kiss_ops.py --emit-coverage; python tools/kiss_tables.py; python tools/kiss_vocab.py; python tools/kiss_wire.py` (whichever expose a check). Expected: no regression from these Rust-only + contract-codec changes.

- [ ] **Step 4: Commit the ledger (if it changed)**

```bash
git -C C:/Projects/kiss-3b add conformance/UNBACKED.tsv
git -C C:/Projects/kiss-3b commit -F - <<'MSG'
conform: ledger — 6.13-0006b + 7.4-0001 now backed (534 -> 532)

Contract-sourced comparator selection (6.13-0006b) and the canonical-enum
advertisement (7.4-0001) are now reverse-cited by executable tests. No SYNTH
§6.5-0004* clause claimed — exercised incidentally only.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
MSG
```

- [ ] **Step 5: Update the SDD ledger** at `C:/Projects/KISS/.superpowers/sdd/progress.md` with per-task outcomes (backed clauses, 534→532, any Step-0 token corrections).

---

## After all tasks

- **Opus whole-branch review** of `feat/harness-comparator-select` (0765252..HEAD): adversarial, focused on — (a) the §6.0-0005 permissiveness ordering is correct and the honesty lint's direction is right (too-permissive rejected, over-claim allowed); (b) `op_true_class` matches §6.0-0004 (Sum/Prod nondeterministic, Max/Min exact-byte); (c) the Guarantees token spellings match the spec (Step 0); (d) the crux test genuinely flips verdict on the SAME result via the advertisement; (e) honest accounting — exactly 6.13-0006b + 7.4-0001 bound, no SYNTH over-claim, trace CLEAN; (f) ubuntu leg compiles (no stray `#![cfg(windows)]`).
- Fix any Critical/Important, re-run the gates.
- **Push / PR / merge is HELD for explicit user authorization.** When authorized: rebase onto `main` if 3a (#118) has merged, then push branch + open PR (footer `🤖 Generated with [Claude Code](https://claude.com/claude-code)` + session URL).

## Self-review notes (author)
- Spec coverage: unit 1→Task 1; unit 2→Task 2; unit 3→Task 3; unit 4→Task 4 + Task 5 (wiring); unit 5 honesty→Task 4; unit 6→Task 6. All six covered.
- Type consistency: `atom_determinism_class -> Option` (T1) consumed by `op_true_class` (T2) and `parse_guarantees_class` returns `DeterminismClass` (T3) consumed by `select_and_compare_reduced` (T5); `check_advertisement` (T4) consumed by T5. Names consistent across tasks.
- Placeholder scan: Step 0 of Task 3 is the one genuine unknown (exact wire-token spelling); it is a bounded "confirm against spec + adjust the two match arms and the test tokens", not an open placeholder — the test fails loudly if the spelling is wrong.
