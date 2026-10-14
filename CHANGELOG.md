# Changelog

`kiss-conformance` (the reference crate and the spec it binds). Pre-1.0, a breaking change moves the
second number.

## 0.2.3 — 2026-10-13 — docs

- `HANDOFF.md` rewritten as current state. No spec, harness or corpus change.

## 0.2.2 — 2026-10-13 — docs

- `ROADMAP.md`: the ordered list of what each sub-standard must reach and what each consumer needs. No spec,
  harness or corpus change.

## 0.2.1 — 2026-10-11 — corrective

The spec rule is **unchanged** from 0.2.0 (Classify §6.6-0008 as amended by #520); the published goldens
now match it. 0.2.0 was never published (`publish = false`).

- **Seven `sk5` positive vectors changed** in `conformance/corpus/structure_key_vectors.json`: the
  keepdim reduction OUTPUT sub-key's broadcast mask is derived, not `00` — `02` (`[4,8]→[4,1]`), `03`
  (`[4,8]→[1,1]`), `01` (`[8]→[1]`), `0a` (`[2,4,3,5]→[2,1,3,1]`): `reduction_trailing_axis`,
  `reduction_all_axes`, `reduction_rank1_all_axes`, `reduction_subset_mask`,
  `noncontraction_acc_mp_field`, `noncontraction_acc_deviating_f64`,
  `noncontraction_reduction_mp_only`. The Classify Appendix A.1 goldens change the same way.
- **Downstream: re-vendor the corpus from the merged #536 commit, not from `a33282c`.** A deriver that
  follows §6.6-0008 already produces these masks.

## 0.2.0 — 2026-10-11 — **Breaking**

One coordinated schema event: KISS-Classify `structure_key` `sk4` → `sk5`, KISS-Contract
`contract_version` 2 → 3 (#263 ruling D, #520, #477, `dequant_form`). Design:
`docs/design-263-cell-class.md`. Every party that derives or parses a `structure_key`, or reads a
contract, regenerates.

### Breaking

- **`op_family` (24 codes) is replaced by the derived `cell_class`** (`ctr ndx rdc ew1 ew2 ew3 oth`,
  Classify §6.5-0006 / §6.5-0015). No op→class mapping exists; a lookup carries the op identity.
- **Every `structure_key` token re-prefixes `sk5`**; a persisted `sk4` token must be re-derived. The
  `sk3` decoder arm is retired (§6.7-0014).
- **`dequant_form` is keyed**: an operand carrying a `quant` record gains a sixth sub-key component
  `dl`/`dc` (§6.3-0009a).
- **The broadcast mask of an extent-1 own axis no longer depends on its recorded stride** (§6.6-0008,
  #520): such an axis sets its bit whenever the frame extent is > 1.
- **Contract `contract_version` is `3`** and the Guarantees field is **`math_fidelity`**
  (was `math_precision`, #477). The KISS-Ops OpAttrs field also named `math_precision` is a different
  channel and is unchanged.
- **The Contract op-category↔op-family table (§6.3-0006) is withdrawn**; the consistency requirement is
  decided from facts the op declares. Consume `wrong-op-class` uses the same requirement.
- **Published artifact spellings changed**: decline kinds `UnknownOpFamily` → `UnknownCellClass`,
  `ReduceNotGatedToRed` → `ReduceNotGatedToRdc` (new: `RdcWithoutReduce`, `BadDequantForm`); the
  decline vector `unknown_op_family` → `unknown_cell_class`.
- **Golden Contract document**: the Guarantees value was the dtype token `f32` (forbidden by §6.8-0004);
  it is now `bit-stable`.

### Not in this release

- Announce: op identity in every lookup (named follow-on, `docs/design-263-cell-class.md` §6).
- #222 candidate 1 (keying a quantized `gem` cell's scale dtype) — needs its own ruling.
