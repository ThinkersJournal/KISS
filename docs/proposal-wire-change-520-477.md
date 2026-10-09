# Proposal — one coordinated wire change: #520 (`sk5`) + #477 (`contract_version` 3)

**Proposal for the PM and editors; no clause is changed by this document.** Measured at KISS `origin/main` `904a4b4`,
2026-10-09. **Version numbers are the PM's to allocate; the ones below are what each change requires, not a request
for specific numbers.**

## Why one event

A schema bump re-prefixes every token or breaks every reader of a format, and forces every party that byte-matches
the corpus to regenerate. #222 states the policy: candidates accumulate so the cost is paid **once**. #520 and #477
are both non-additive wire changes found after `sk4` / `contract_version` 2 closed, both small, and both found by
implementors. Landing them together means Fuel, Unpopped, Baracuda and Vulkane regenerate once.

## The two changes

**#520 — extent-1 axes and the broadcast mask (Classify, structure_key).** KISS-CLASSIFY-6.6-0008 sets mask bit `i` iff the
frame extent is > 1 and the operand's stride along that axis is 0. The stride of an extent-1 axis is never stepped, so
it is arbitrary — but the mask keys on it. Same tensor, same meaning, two tokens (`01` vs `00`). Safe-failing (cache
miss, not a wrong kernel). **Fix: an extent-1 own axis counts as stride 0 for the mask test.** One sentence in 6.6-0008,
a test, both derivers (`fuel-dispatch` `operand_sub_key`, `unpopped-vocab` `derive_operand_key`). It changes tokens for
exactly the operands whose extent-1 axis carries a non-zero stride, so it is **non-additive: a `structure_key` schema
bump** (KISS-CLASSIFY-6.4-0003). Cheap now, while the sub-standard is UNFROZEN.

**#477 — rename the Guarantees field `math_precision` → `math_fidelity` (Contract).** The type was renamed in #414/#478;
the serialized field in KISS-CONTRACT-6.8-0001 was deliberately left, because renaming a MUST-carry-exactly field in a
pinned serialization order is a contract-format break: a **`contract_version` bump** (§8-0002). The corpus does not
cover it (`contract_vectors.json` renders 3 of 7 blocks, and the Guarantees block is not among them), so a golden that
exercises the field is part of the change. Known affected: Baracuda (`PrecisionGuarantee.math_precision`) and Vulkane;
**a complete affected-set census has not been done** (#484: key on what a lane emits or parses, not on its name).

## What else is already waiting for the same event (so it is decided as a set, per #222)

- **`dequant_form` into the key.** KISS-CLASSIFY-6.3-0009a already mandates it "effective at the next `structure_key`
  schema version" and says the current grammar has no slot. This is an obligation, not a candidate: the next bump
  *must* carry it.
- **#222 candidate 1: key a quantized `gem` cell's scale dtype** (without reversing 6.6-0015 for non-`gem` cells). A
  request to revisit a ruled decision; needs your ruling, not a default.
- **#263 (op→family mapping)** if option B of `docs/decision-263-op-family-mapping.md` is chosen: any row where a
  deriver disagrees with the pinned table changes tokens, so it belongs in this event.

I recommend this event contain: #520, `dequant_form`, #477, and the #263 rows that need to move; and that scale-dtype
keying be decided before the PR opens rather than discovered in it.

## Proposed sequence and sizes

| Step | Owner | Size |
|---|---|---|
| 1. Decide the event's contents (above) and allocate the two numbers | PM / you | S |
| 2. Spec edits: 6.6-0008 (+ test), 6.3-0009a slot, 6.7 grammar, 6.4-0003 text, 6.8-0001 field name and 6.11-0005 order, §8 notes | KISS lane | M |
| 3. Regenerate the codec-generated artifacts (`structure_key_vectors.json`, new Guarantees golden) with the tool, never by hand; re-derive floor/README figures | KISS lane | M |
| 4. Publish the reference crates at the new pin (needs your per-crate yes) | kiss-ref lane | L (it is also the catch-up to the October reconciliation; see the release evidence) |
| 5. Derivers regenerate: Fuel, Unpopped (`sk5`); Baracuda, Vulkane (`math_fidelity`) | those lanes | M each |

Steps 1–3 are the KISS lane's and I can start on your word. Steps 4–5 are not mine to schedule.

## Risks and what is not known

- Unpopped#49 (0.15.0, HELD) and fuel's re-vendor of `f8e6m2` are in flight against the **current** `sk4` corpus. A
  second regeneration right behind them is the cost of not batching; sequencing against those is the PM's call.
- The affected set for #477 beyond Baracuda and Vulkane is unmeasured.
- Nothing here has been shown to the Fuel, Unpopped or Baracuda lanes. Objections go through the PM.
