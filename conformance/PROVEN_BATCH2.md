# PROVEN batch 2 — the isolation matrix behind ten more `// Proven:` markers (#278)

**Ten already-CITED clauses migrated to PROVEN.** Unlike batch 1's founding set, every clause here
was **already CITED** (single-backed by a backing-form citation), so this batch adds no `// Backs:` —
only the `// Proven:` marker and its demonstration. Floor `proven 10 -> 20`; harness/lint/untested
unchanged. Run on green `main` @ `eae8c8b`.

Each `// Proven: KISS-X (subject: impl; ref: PROVEN_BATCH2.md)` marker is testimony that a seeded
mutation of the clause's implementation obligation was shown to redden its proving test — a **fresh
re-run** via `tools/proven_batch2_matrix.py`, not a transcription of prose.

## How to re-run

`tools/proven_batch2_matrix.py` seeds each mutation below in `conformance/src/`, runs the batch's
test set **unfiltered**, records which tests fail, and restores the source byte-for-byte. It asserts
each seed applied (convention 9) and each restore byte-exact. Two of the ten proving tests are lib
unit tests (`#[cfg(test)]` in `conformance/src/per_output.rs`), so the run passes `--lib` alongside
five integration binaries and matches failed tests by their last `::` component:

```
cargo test --no-fail-fast --manifest-path conformance/Cargo.toml --lib \
  --test contract_golden --test contract_audited_status --test shape_expr \
  --test synth_request_decline --test structure_key_golden
```

Two properties are ENFORCED, not merely advised (architect review, #326):
- **`--no-fail-fast`** — without it cargo stops after the first failing TARGET, so a mutation whose
  victim is in an early target truncates the run and later targets never execute. The sharp case:
  `--lib` runs first, so a `per_output` victim would stop the run before any integration binary, and
  "kills exactly one" would be measured over a partial population missing eight of the ten batch tests.
  With it, every target runs every time, so the kill set is the COMPLETE population. Re-run under
  `--no-fail-fast` produced the IDENTICAL matrix and spill recorded below — the claim is EARNED, not a
  fail-fast artefact (the same check that retired this hazard on batch 1).
- **baseline gate** — the unmutated set must be all-green first, or the driver aborts (exit 2, nothing
  seeded). A pre-existing red would otherwise ride every mutation's kill set and could satisfy "kills
  exactly one" by a stale failure rather than by the mutation.

## The mutations (subject = impl for all ten; each is a one-site edit, `old` grep-verified unique)

| clause | proving test | file:site | mutation (obligation attacked) |
|---|---|---|---|
| KISS-CONTRACT-6.11-0001 | `test_contract_text_field_encoding` | contract.rs (`Value::Array` render) | `elems.join(", ")` → `elems.join(",")` (drop the mandated space after the comma) |
| KISS-CONTRACT-6.11-0002 | `test_contract_document_header_line` | contract.rs (header encode) | magic `"KISC …"` → `"XISC …"` (wrong 4-byte magic; doc[0]=0x58≠0x4B) |
| KISS-CONTRACT-6.11-0005 | `test_contract_document_field_order` | contract.rs (`render_block`) | `for (k, v) in fields` → `fields.iter().rev()` (reverse field order) |
| KISS-CONTRACT-6.8-0008 | `test_contract_audited_status_derived` | contract.rs (`derive_audited_status`) | `if g.declares_bounded_precision()` → `if true \|\| …` (constant, not DERIVED) |
| KISS-CONFORM-6.8-0011 | `test_conform_per_output_comparator_selection` | structural.rs (order-invariant arm) | `order_invariant_agree(…, abs_tol, rel_tol)` → `(…, 0.0, 0.0)` (collapse the band → one whole-op comparator) |
| KISS-OPS-6.20-0005 | `test_shape_expr_serialization_golden` | shape_expr.rs (`SameAs` encode) | `vec![TAG_SAME_AS, *operand]` → `vec![*operand, TAG_SAME_AS]` (tag/field byte swap) |
| KISS-OPS-6.0-0008 | `test_ops_comparison_mask_is_selection` | determinism.rs (`COMPARISON_ATOMS`) | drop `"cmp_lt"` from the set (mask classed value, not selection → ULP category error) |
| KISS-SYNTH-6.6-0004 | `test_synth_decline_framing` | announce.rs (`DeclineResponse::encode`) | `decline_code.to_le_bytes()` → `(… as u16).to_le_bytes()` (u32→u16, frame short 2 bytes) |
| KISS-SYNTH-6.1-0002 | `test_synth_request_reuses_cyrq` | announce.rs (`QueryRequest::encode`) | `CYRQ.to_le_bytes()` → `.to_be_bytes()` (big-endian tag: `51 52 59 43` not `43 59 52 51`) |
| KISS-CLASSIFY-6.7-0004 | `a1_elementwise_with_broadcast_operand` | structure_key.rs (`to_token` operands) | `.join(";")` → `.join(",")` (wrong per-operand separator in field 7) |

## Isolation matrix — every mutation reddens EXACTLY ONE batch test (its own)

Expected kill count per mutation: **1**. Result: **ALL EXACTLY-ONE**.

```
test_contract_text_field_encoding              kills [test_contract_text_field_encoding]              OK
test_contract_document_header_line             kills [test_contract_document_header_line]             OK
test_contract_document_field_order             kills [test_contract_document_field_order]             OK
test_contract_audited_status_derived           kills [test_contract_audited_status_derived]           OK
test_conform_per_output_comparator_selection   kills [test_conform_per_output_comparator_selection]   OK
test_shape_expr_serialization_golden           kills [test_shape_expr_serialization_golden]           OK
test_ops_comparison_mask_is_selection          kills [test_ops_comparison_mask_is_selection]          OK
test_synth_decline_framing                     kills [test_synth_decline_framing]                     OK
test_synth_request_reuses_cyrq                 kills [test_synth_request_reuses_cyrq]                  OK
a1_elementwise_with_broadcast_operand          kills [a1_elementwise_with_broadcast_operand]          OK
```

**Demonstration defect rate: 0/10** — every intended test reddened on the first matrix run. As in
batch 1 this is a *designed-then-verified* zero: each mutation was written against the read
implementation, and the seed-applied / seed-restored assertions confirm no mutation silently no-op'd.

## Co-reddened NON-batch tests (informative — all same-obligation, no different-obligation defect)

Each mutation reddens exactly one BATCH test; several also redden sibling NON-batch tests that share
the same implementation site. Every co-reddening is the **same obligation** (a shared encoder/decoder/
token-format chokepoint), which is why it cannot be a batch defect — it bounds what a single mutation
demonstrates, exactly as convention 9 anticipates:

- **6.11-0002** (KISC magic) → `test_contract_reject_malformed_header`, `test_contract_rejection_is_typed_decline` (the same magic on the decode path).
- **6.11-0005** (field order) → `test_contract_text_op_dag` (another render-order golden).
- **6.8-0008** (audited constant) → `test_conform_contract_audited_status`, `test_contract_unaudited_derivation_rule` (the same derivation rule).
- **6.6-0004** (decline u16) → `test_synth_malformed_echoes_empty_identity`, `test_synth_reader_accepts_empty_identity` (the same decline frame).
- **6.1-0002** (CYRQ big-endian) → the six `test_synth_request_*` siblings (all read the CYRQ request frame).
- **6.7-0004** (operand separator) → sixteen `structure_key` golden tokens (every ≥2-operand token — `a1_binary_two_operands`, `a1_reduction_*`, `sk4_*`, …).
- **6.11-0001, 6.8-0011, 6.20-0005, 6.0-0008** → no spill (fully surgical).

A consequence worth stating: a co-reddened sibling **cannot later be independently proven by the SAME
mutation** — it needs its own. E.g. `KISS-CLASSIFY-6.6-0020` (cell-mates-not-substitutable) was a
candidate for this batch and was **dropped** precisely because its only mutable site is the shared
`to_token` field set, so no single-site mutation reddens it *alone* (the collapse that would redden it
also reddens 6.7-0004's token and the whole golden suite). It is a structural/negative property, not an
isolated impl site — recorded here rather than forced.

## Scope of the spill measurement (no silent cap)

The isolation claim is COMPLETE, and **`--no-fail-fast` is what makes it so**: all ten batch proving
tests live in the run set (`--lib` + the five named binaries) AND every target runs every time, so
"each kills exactly one BATCH test" is measured against every batch test actually EXECUTED, not merely
declared. (Being in the run SET is not being in the run; `--no-fail-fast` closes that gap — architect
review, #326.) The spill lists above are measured within that run scope — a co-reddening in a binary
NOT in the scope (e.g. a different sub-standard's integration test) would not appear. Spill is informative, not the PROVEN gate;
the gate is isolation among batch tests, which is complete.

This scope is a deliberate **narrowing** from a full-suite run described earlier in review, not the
original plan quietly changed: isolation is complete in-scope (every batch test is in the run set), so
a full-suite pass would only widen the informative spill list, not the gate. Recorded as a reconsidered
decision so it reads as a narrowing, not a silent drift.

## Structurally-unprovable clauses (a category, not a miss)

A clause expressing a **structural or negative property** may have no single-site mutation at all — its
PROVEN tier is then **structurally unreachable**, not merely unearned. Such a clause cannot join a
PROVEN batch by construction, so the candidate pool is best understood as **provable minus
structurally-unprovable**, rather than a target that mysteriously never reaches 100%.

First member:
- **KISS-CLASSIFY-6.6-0020** (`test_classify_cell_mates_are_not_substitutable`) — its teeth are a
  negative property (`named.is_empty()`: no `|`-field of a derived token equals a KISS-Ops op-name)
  and a discrimination control. The only impl it exercises is `StructureKey::to_token`, whose sole
  distinguishing field between the "bin" and "une" keys is `op_family`; the single-site edit that
  would collapse them to redden this test simultaneously changes every golden token (it reddens
  6.7-0004's token and the whole golden suite). No mutation reddens it ALONE, so it has no isolable
  PROVEN demonstration — a property of the obligation, not a gap in effort.
