# Design note — adopting the D7 FDX-successor sidecar into Classify/Announce

**Status: design note, for the KISS editors. No clause is proposed as final here; the spec PR follows once the
open questions in §6 are answered.** Measured at KISS `origin/main` `904a4b4`, 2026-10-09. Input:
[`rfcs/d7-fdx-successor-sidecar.md`](../rfcs/d7-fdx-successor-sidecar.md) (cosigned by Fuel and Baracuda
2026-07-21, never adopted).

## 1. What adoption has to do

Publish the RFC's neutral field union as the shape of KISS-Classify §4's *external quantization-token registry*, so
Fuel's private FDX descriptor and Baracuda's private `QuantFacts` mirror can retire. **No `structure_key` byte
changes.** That is the RFC's claim, and §2 shows it needs a correction.

## 2. The RFC is stale against the spec in three places (read before adopting)

1. **`dequant_form` is absent from the RFC and present in the spec.** KISS-CLASSIFY-6.3-0009/-0009a (already on main)
   put `dequant_form ∈ {linear, codebook}` in the `quant` record and require it to be **folded into the
   `structure_key` at the next schema version** — a *narrowing* of carried-not-keyed. The RFC says the sidecar
   "MUST NOT be folded into `structure_key`" and lists no `dequant_form`. Taken literally, adopting the RFC would
   contradict a clause that is in force. The sidecar needs a `dequant_form` field, and the "never keyed" sentence
   must read "never keyed *except* `dequant_form`, per 6.3-0009a".
2. **The RFC's mapping table is incomplete.** It maps `family↔encoding`, `scale_placement↔scale.placement`,
   `sub_byte_bits/block_elems↔byte_layout/quant_block.block_size`. The sidecar's `logical_dtype`,
   `quant_block.scale_dtype`, `scale.dtype`, `scale.granularity` and `alignment` have **no counterpart in the
   `quant` record** (6.3-0009: `{family, sub_byte_bits, block_elems, scale_placement, dequant_form}`), so the
   sidecar is a strict superset, not a rename. Where the scale dtype lives matters: sk4 already models an MX scale
   as a **sibling operand** with its own dtype (`f8e8m0`, 6.1-0013), so `scale.dtype` for `separate_buffer` would
   duplicate that operand's dtype.
3. **The RFC's source link is dead.** It cites `../RECONCILIATION.md`; no such file exists on main. Use
   `docs/kiss-convergence-reconciliation.md`.

## 3. The decision the editors actually have (RFC §7 left it open): the wire form

| Option | What changes in the spec | Cost |
|---|---|---|
| **A. Extend the §6.3 `quant` record in place** | 6.3-0009 gains the five new fields (optional); `contract`/descriptor schema bytes change | A wire/descriptor change: needs a version allocation and rides with #477/#520; Fuel and Baracuda must emit it |
| **B. Standalone sidecar blob referenced by the operand** | New clause family + a wire spelling; operand gains a reference field | Largest: new wire, new golden vectors, new decline codes |
| **C. Informative annex plus one mapping clause (recommended)** | Classify §4 names the sidecar shape as the registry's shape; **one** new clause: a producer that exchanges quant facts as a sidecar MUST project to/from the `quant` record by the table, and the fields with no `quant` counterpart are carried registry-side, opaque to KISS (the 6.3-0009 opacity rule, generalized). `quant` record unchanged | Smallest; no descriptor bytes, no version allocation, no new vectors. Does not by itself make the sidecar *interoperable bytes* — it fixes the vocabulary, as the D2 informative-table treatment did |

**Recommendation: C**, because the RFC's own §6 says "no version bump, no golden regeneration" and its scope table
puts the byte form out of scope; C is the only option that honours that. A and B are the way to get byte-level
interop later, and nothing in C blocks them.

## 4. Spec edits under C

- `spec/classify.md` §4 (the external-registry dependency): name the neutral field set (RFC §4.1) and the open
  `encoding` seed registry.
- `spec/classify.md` §6.3, **one new clause** after 6.3-0011 (next free id to be confirmed at PR time): the
  projection table of §2.2 above, `dequant_form` included, and the sentence that every field except `dequant_form`
  stays out of the key at every version (consistent with 6.3-0009a).
- `spec/announce.md` §7.2: no change to bit assignments (`DLPACK_EXT_MX` bit 1, `_GGML` bit 2, `_AFFINE` bit 3 already
  reserve the seeds). Add one sentence that EXT meanings for those bits are the sidecar's `encoding` seeds.
- `rfcs/d7-fdx-successor-sidecar.md`: status line → "Adopted as of <PR>", fix the dead link, record §2's corrections.

## 5. Backing and the ratchet

A new MUST needs a test or it raises the untested-MUST count (479 per the `strict` job). Proposed test: a table lint
in `tools/` that parses the projection table out of the clause and checks every `quant`-record field appears exactly
once on the sidecar side and vice versa (the same shape as the existing correspondence lints). `COVERAGE_FLOOR.tsv`
and the README bounds are re-derived by the tools, not edited by hand. Size: M (two spec files, one RFC status edit,
one lint with controls).

## 6. Questions for the Fuel and Baracuda lanes (to be routed through the PM, objections only)

1. Does either side carry anything in its private descriptor that the RFC's field union plus `dequant_form` does not
   cover? (Fuel: the `AFFINE_BLOCK` / NF4 case, which `ROADMAP.md` calls out as out of the oracle's reach until it lands.)
2. Is C acceptable as "adoption", given that it fixes the vocabulary but not the bytes? If either side needs byte
   interop now, that is Option A or B and a version allocation.
3. `byte_layout` has no incumbent registry (RFC §7): does either side already have packing codes to seed it?
4. Baracuda: is `QuantFacts` still in code? Its repo mentions it only in `docs/fdx-successor-baracuda-position-2026-07-20.md`
   at `62c4cde2`; I did not look further.

## 7. Not verified

Whether any Fuel or Baracuda code currently emits quant facts in a shape that differs from the RFC's union; the next
free clause id; whether `dequant_form` is in fact slotted at `sk5` (the `sk5` collection, #222, is unscheduled).
