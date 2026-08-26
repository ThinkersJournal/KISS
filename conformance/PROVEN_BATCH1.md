# PROVEN batch 1 — the isolation matrix behind the first nine `// Proven:` markers (#278)

**Ten clauses were demonstrated; nine became markers in the founding batch.** The tenth,
`KISS-CONTRACT-6.1-0008`, was demonstrated cleanly (its mutation reddens exactly its own test)
but was **deferred** as NAMED-not-CITED — see "The NAMED-not-CITED finding" below. **It is now
migrated and marked in the #278 follow-up (this PR): CITED via `// Backs:`, PROVEN via `// Proven:`,
floor proven 9 → 10. All ten demonstrations now carry markers.**

Each `// Proven: KISS-X (subject: impl; ref: PROVEN_BATCH1.md)` marker in the harness is
**testimony that a re-runnable demonstration exists here** — a seeded mutation of the clause's
implementation obligation, shown to redden its proving test. These are **fresh re-runs**, not
transcriptions of #270/#188's prose: the architect ruled (#278) that a past demonstration is
transcribable only when the proving test is byte-identical to its ref *and* the demonstration
itself is re-runnable; the earlier demonstrations were reverted and lived only in issue/PR
prose (a memory of a proof), so they were re-run against current `main`.

## How to re-run

`tools/proven_batch1_matrix.py` seeds each mutation below in `conformance/src/`, runs the four test
binaries **unfiltered** (`cargo test --test structure_key_golden --test integer_semantics
--test structural_access --test contract_framing`), records which tests fail, and restores the
source byte-for-byte. Each seed is asserted applied (convention 9) and asserted restored.

## The mutations (subject = impl for all ten; each is a one-site edit)

| clause | proving test | file:site | mutation (obligation attacked) |
|---|---|---|---|
| KISS-CLASSIFY-6.1-0001 | `reject_unknown_dtype` | structure_key.rs (dtype guard) | `if !DTYPES.contains(&f[2])` → `if false && …` (accept an unknown dtype) |
| KISS-CLASSIFY-6.5-0006 | `reject_unknown_op_family` | structure_key.rs (op-family guard) | `if !OP_FAMILIES.contains(&f[1])` → `if false && …` |
| KISS-CLASSIFY-6.7-0009 | `reject_wrong_field_count` | structure_key.rs (field-count guard) | `if f.len() != 9 && f.len() != 10` → `if false && …` |
| KISS-CLASSIFY-6.5-0007 | `test_classify_work_class_element_count` | structure_key.rs (`derive_work_class`) | `c <= 1024 => Block` → `=> Grid` (wrong work-class boundary) |
| KISS-OPS-6.4-0001 | `test_ops_add_sub_mul_wrapping` | integer.rs (`IntDtype::add`) | `wrapping_add` → `saturating_add` (saturate, not wrap) |
| KISS-OPS-6.4-0003 | `test_ops_neg_integer` | integer.rs (`IntDtype::neg`) | `wrapping_neg` → `!self` (one's- not two's-complement) |
| KISS-OPS-6.11-0003 | `scan_is_length_preserving` | structural.rs (`prefix_scan_f32`) | drop the last output (`out` → `out.pop(); out`) |
| KISS-OPS-6.11-0005 | `scatter_oob_writes_are_skipped` | structural.rs (`scatter_f32`) | remove the OOB-skip `continue` (OOB write no longer skipped) |
| KISS-OPS-6.11-0010 | `scatter_atomic_max_min_nan_propagating` | structural.rs (`scatter_f32`) | `max_prop` → `f32::max` (NaN-suppressing, not -propagating) |
| KISS-CONTRACT-6.1-0008 | `test_contract_version_value_pinned` | contract.rs (`read_document`) | `if version != "1"` → `!= "2"` (reject the pinned version) |

## Isolation matrix — every mutation reddens EXACTLY ONE batch test (its own)

Expected kill count per mutation: **1**. Result: **ALL EXACTLY-ONE**. Each mutation's own
proving test is the only one of the ten it reddens, so the ten clauses are **independently
demonstrated** — no property is one property credited ten times (convention 9's reason, which
applies even though its trigger — a shared test — does not fire on ten clauses with ten tests).

```
reject_unknown_dtype                     kills [reject_unknown_dtype]                     OK
reject_unknown_op_family                 kills [reject_unknown_op_family]                 OK
reject_wrong_field_count                 kills [reject_wrong_field_count]                 OK
test_classify_work_class_element_count   kills [test_classify_work_class_element_count]   OK
test_ops_add_sub_mul_wrapping            kills [test_ops_add_sub_mul_wrapping]            OK
test_ops_neg_integer                     kills [test_ops_neg_integer]                     OK
scan_is_length_preserving                kills [scan_is_length_preserving]                OK
scatter_oob_writes_are_skipped           kills [scatter_oob_writes_are_skipped]           OK
scatter_atomic_max_min_nan_propagating   kills [scatter_atomic_max_min_nan_propagating]   OK
test_contract_version_value_pinned       kills [test_contract_version_value_pinned]       OK
```

**Defect rate (KISS's first measured demonstration defect rate): 0 defects — and the denominator
is worth stating, because two are in play (Copilot #311).** Ten mutations were *demonstrated*
(the matrix above), so the demonstration defect rate is **0/10**; nine of the ten became
`// Proven:` markers and the tenth (`KISS-CONTRACT-6.1-0008`) was deferred as NAMED-not-CITED, so
across *marked clauses* the rate is **0/9**. Same zero, different construct: 0/10 counts
demonstrations, 0/9 counts markers. Every intended test reddened on the first matrix run. This is
a *designed-then-verified* zero: each mutation was written against the read implementation, not a
blind first guess, and the seed-applied / seed-restored assertions confirm no mutation silently
no-op'd. Reported even though it is zero, per #278; MLMF's 11-across-10 is a prior, not this
measurement.

## The NAMED-not-CITED finding (why 6.1-0008 is demonstrated but not marked)

The survey (#278) listed `KISS-CONTRACT-6.1-0008` among the byte-identical CITED candidates. It
is **not CITED** — `test_contract_version_value_pinned` names the clause only inside assert
*messages* (`.expect("KISS-CONTRACT-6.1-0008: …")`), a MENTION, not a backing form, so
`discover_tests` reports its `clauses` set as empty and the clause is **NAMED** (forward-backed
by its §9 name alone). This is exactly the distinction the tier exists to draw, and the batch's
own `collect_proven` proof-without-backing gate is what caught it: a `// Proven:` there would be
rejected. `PROVEN ⊆ CITED`, so proving it first requires an evidence-adding **migration** to
CITED (a genuine `// Backs:`, justified by the demonstration above), which is a distinct step
kept out of the founding batch to keep every founding member *already* CITED. Deferred to a
follow-up. The demonstration itself is valid and recorded above.

**Resolved (this PR, #278 follow-up).** `test_contract_version_value_pinned` now carries
`// Backs: KISS-CONTRACT-6.1-0008` (NAMED → CITED; harness unchanged — the move is NAMED 241 → 240 /
CITED 139 → 140, both inside the 380) and `// Proven: KISS-CONTRACT-6.1-0008 (subject: impl; ref:
PROVEN_BATCH1.md)`. The floor's `proven` dimension rises 9 → 10, tool-derived. No re-run was needed
and none was transcribed from prose: the demonstration in row 36 / the isolation matrix (row 55) is
batch 1's own **fresh matrix re-run**, which already satisfies the #278 transcribability rule — the
proving test is byte-identical to its ref and the demonstration is re-runnable. That byte-identity is
re-confirmed on green `main` (9c8543b): `git diff --quiet ee1d186 9c8543b -- conformance/src/contract.rs
conformance/tests/contract_framing.rs` exits 0, so the recorded mutation site and its target test are
unchanged and the demonstration applies bit-for-bit. (A fresh Edit-based re-mutation of the source on
this tree was refused by the sandbox classifier; it was not routed around, and file-invariance is the
stronger grounding regardless — the subject bytes cannot have drifted.)

## Cross-marker properties (checked over the whole set)

1. **No `(clause, test)` pair repeats with a different subject** — the ten pairs are distinct
   and every subject is `impl`.
2. **Every `ref` contains a demonstration** — the `ref` is this file, which records the mutation
   and its reddening for each of the ten.
3. **Mutation granularity is consistent** — each is a single-site edit to the exact function the
   clause obligates. One deliberate exception, recorded not hidden: `scatter_oob_writes_are_skipped`
   reddens by **panic** (removing the OOB skip lets an out-of-range index reach the write and the
   test aborts) rather than by a value assertion; the failure is still attributable to the
   OOB-skip obligation and to that test alone.

## Co-reddened NON-batch tests (informative, not a batch defect)

Each mutation also reddens sibling tests that share the same implementation — all OUTSIDE this
batch, so they do not affect the ten-clause isolation. Worth recording because it bounds what a
single mutation demonstrates: e.g. the `neg` mutation also reddens `test_ops_int_neg_abs_wrap`
(KISS-OPS-6.4-0005, which shares `neg`), so 6.4-0005 could not later be independently proven by
the *same* `neg` mutation — it needs its own. Likewise the contract-version mutation reddens the
four other tests that read the pinned version, and the dtype/op-family mutations redden the other
closed-set typed-decline tests.
