# RFC: `d7` — a neutral FDX-successor sidecar for quant/layout facts DLPack cannot carry

| | |
|---|---|
| **Status** | **Cosigned; ready for editor adoption (2026-07-21).** The neutral field union below is signed off by both cosignatories — Fuel (as the FDX-schema originator) and Baracuda (provider), which drops its private `QuantFacts` mirror in favor of this shared shape. No maintainer objection outstanding; RFC pending editor adoption + merge to main. The concrete clause touch-points (§4) are approved in principle for a later application step; the exact token spellings and clause numbers are the editors' to pin. |
| **Date** | 2026-07-21 |
| **Affects** | KISS-Classify (§6.3 `quant` facts, §4 external-registry dependency), KISS-Announce (§7.2 EXT axis, the DLPack/FDX interchange bits) — **informative/registry only; no `structure_key` bytes change** |
| **Filing** | umbrella §7.2, to the KISS-Classify / KISS-Announce editors-of-record, cc Fuel + Baracuda |
| **Source** | [`RECONCILIATION.md`](../RECONCILIATION.md) decision **D7**; the `sk3` scope-boundary carve-out (§3, "D7 FDX blessing" — separable, no version bump); the `#17` open-seeded-registry pattern; the D2 informative-table treatment |
| **Related** | `sk3` (GEMM precision coordinates — establishes that MX/DLPack element facts stay **out** of the identity key); KISS-CLASSIFY-6.3-0009 (`quant` carried-not-keyed); KISS-ANNOUNCE §7.2 EXT registry (DLPack/FDX bits); PRIOR-ART.md §4/§6 (DLPack owns the dtype/interchange boundary) |

> **This RFC is informative until adopted.** It proposes a shared *field union* (a sidecar
> schema) and its registry home; it does **not** propose any change to `structure_key` bytes.
> Every field spelling and every seed-registry code below is the editors' to pin — the seed
> codes are recommended spellings, not a closed set (§3). The one binding position is the
> **boundary**: DLPack owns interchange; the sidecar carries only what DLPack cannot; and none
> of it enters the KISS-Classify identity key (consistent with `sk3` and D4).

---

## 1. Summary

A single **neutral, project- and language-agnostic sidecar schema** — the successor to Fuel's
private FDX descriptor and Baracuda's private `QuantFacts` mirror — that carries the
**dtype / quantization / layout facts DLPack structurally cannot express**: sub-byte and block
element formats, quant-block structure, and scale placement.

Three things are settled here:

1. **It is a neutralized shared shape, not a Fuel-struct lift.** The field union below is the
   cosigned intersection-plus-union of both cosignatories' private descriptors, with every
   vendor spelling and every language-specific type erased. Baracuda **drops its private
   `QuantFacts` mirror** and consumes this shape instead; Fuel contributes the FDX field set
   neutralized.
2. **It sits alongside DLPack, never over it.** DLPack owns the interchange boundary (dtype
   code/bits/lanes, shape, strides, device). The sidecar is an **overlay** that carries *only*
   the encoding facts DLPack has no field for. Where DLPack already carries a fact (e.g. a
   whole-byte element dtype), the sidecar does not restate it.
3. **It stays out of the KISS-Classify identity key.** Exactly as `sk3` keeps MX element codes
   and DLPack interchange tokens out of `structure_key`, and exactly as KISS-CLASSIFY-6.3-0009
   already mandates for the `quant` record — these facts are **carried for binding, never folded
   into the admissibility key.** The sidecar is the fuller, neutral realization of that same
   `quant` record.

## 2. Motivation

**DLPack is the interchange boundary and should stay it — but it cannot describe a quantized
tensor's encoding.** PRIOR-ART.md §6 already concludes KISS should ride DLPack for the dtype
boundary rather than reinvent it: DLPack's `DLDataType` (code, bits, lanes) is the de-facto
standard and already absorbs the sub-byte and MX *element* formats (`kDLFloat4_e2m1fn`,
`kDLFloat8_e8m0fnu`, the FP6 pair). What DLPack does **not** carry is the *structure around* a
quantized element: the block size a scale applies to, where the scale lives (embedded vs a
sibling buffer vs a per-axis broadcast vector), the scale's own dtype and granularity, and the
byte packing of sub-byte elements. Those are precisely the facts a consumer needs to *bind* a
quantized operand to a kernel, and precisely the facts two providers today each carry in a
**private, mutually-incompatible descriptor** (Fuel's FDX struct; Baracuda's `QuantFacts`).

**Two private mirrors is a fork waiting to happen.** KISS-CLASSIFY §4 already names an *external
quantization-token registry* (the "FDX/DLPack quant family") as the source of truth for the
`family` / `scale_placement` codes the `quant` record mirrors (§6.3-0009) — but that registry's
*shape* was never neutralized. Each cosignatory filled the gap privately, so the two descriptors
diverge in field names, in what is embedded vs referenced, and in which quant families exist.
This RFC closes the gap by publishing the neutral shape both already project into, so the
external registry KISS depends on has one shared vocabulary instead of two private ones.

**Why a sidecar and not more key coordinates.** `sk3` settled the analogous question for GEMM
precision: facts that distinguish *which kernel* go in the key; facts carried *for binding a
kernel already chosen* do not. Quant encoding is binding-facts — a consumer resolves the cell
first (geometry, op-family, primary dtype), then checks the sidecar to confirm the encoding it
holds is one the kernel accepts. Folding encoding into the key would perturb every quantized
token for zero discrimination gain and would re-import into KISS an external vocabulary whose
semantics KISS deliberately keeps opaque (§6.3-0009). The sidecar keeps the key minimal
(`sk3`'s guardrail) and the boundary clean (PRIOR-ART's recommendation).

## 3. Scope boundary

| In scope (the neutral sidecar) | Out of scope (owned elsewhere) |
|---|---|
| Encoding facts DLPack has no field for: sub-byte packing, quant-block structure, scale placement/dtype/granularity | The interchange dtype boundary itself (DLPack: code/bits/lanes, shape, strides, device) |
| A neutral, open-seeded `encoding` registry both cosignatories project into | The KISS-Classify identity key — `structure_key` bytes are **unchanged** (§1, `sk3`/D4) |
| Retiring the two private descriptors (Fuel FDX struct, Baracuda `QuantFacts`) in favor of this shape | Element-format *values* (E8M0, E2M1, …) — those are DLPack/OCP definitions, cited not restated |
| The registry home + mirror (KISS-Announce EXT axis; KISS-Classify §4 dependency) | MX + DLPack in the key — stays **out** (`sk3` §3; this RFC does not reopen it) |

**Boundary, pinned.** DLPack owns the interchange boundary. The FDX-successor sidecar is an
*overlay* that carries **only** the encoding facts DLPack cannot: sub-byte element formats and
quant-block structure. MX and DLPack interchange tokens stay **out of the identity key** —
this RFC neither adds them to `structure_key` nor removes the `sk3`/D4 position that keeps them
out.

## 4. The neutral sidecar schema

The sidecar is a record attached to an operand (the neutral successor to the KISS-CLASSIFY §6.3
`quant` record, and the realization of the external-registry shape KISS-CLASSIFY §4 depends on).
All fields are **optional at the record level** — a record present at all implies at least
`logical_dtype` + `encoding`; the block/scale sub-records are present iff the encoding is a block
scheme. Field spellings below are the cosigned neutral names; the editors pin the wire form.

### 4.1 Fields

- **`logical_dtype`** — the stored **logical element format** (what one element *is*, before any
  block/scale interpretation). Drawn from the DLPack dtype boundary where DLPack has a code for
  it; the sidecar restates it only so the record is self-describing when read apart from the
  DLPack header. This is the anchor the other fields modify.

- **`encoding`** — the **encoding scheme**, drawn from an **OPEN registry**. Seed set (recommended
  spellings, **not** a closed enum):

  | `encoding` | Meaning |
  |---|---|
  | `ggml` | GGML block-quant family (llama.cpp lineage) |
  | `mx` | OCP Microscaling (shared per-block scale) |
  | `affine_int` | affine (scale+zero-point) quant, integer scale domain |
  | `affine_float` | affine quant, floating-point scale domain |
  | `affine_block` | affine quant applied per block (block-affine) |

  The registry is **OPEN** so any ecosystem adds its own encoding without a spec revision — the
  **same open-seeded-registry pattern** used by the `#17` `blocker_reason` codes and the D2
  informative tables: KISS pins the *seed spellings* as recommended, mirrors external additions
  in the KISS registry (KISS-ANNOUNCE §7.2 EXT axis), and treats the code as **opaque** — KISS
  never re-derives an encoding's semantics (KISS-CLASSIFY-6.3-0009's opacity rule, generalized).

- **`quant_block { block_size, scale_dtype }`** — **block schemes only** (present iff `encoding`
  is a block scheme, e.g. `mx`, `ggml`, `affine_block`). `block_size` is the element count one
  scale governs; `scale_dtype` is the dtype of the per-block scale. Absent for non-block
  encodings (e.g. a plain per-tensor `affine_int`).

- **`scale { placement, dtype, granularity }`** — how the dequantizing scale is stored and
  applied:
  - `placement ∈ {inline, separate_buffer, broadcast_per_axis}` — `inline` = scale embedded with
    the elements; `separate_buffer` = scale is a **referenced sibling operand, never embedded**
    (this is Fuel's model-B "scale as a referenced sibling operand" — it maps exactly to
    `placement = separate_buffer`); `broadcast_per_axis` = a per-axis scale vector broadcast over
    the tensor.
  - `dtype` — the scale's own element dtype.
  - `granularity ∈ {per_tensor, per_axis, per_block}` — the extent one scale value covers.

- **`byte_layout` / `alignment`** — how the quantized elements are **packed**: the sub-byte
  packing order / nibble arrangement (`byte_layout`) and the base packing `alignment`. This is
  the fact DLPack most conspicuously lacks — DLPack's `bits` names the element width but not how
  sub-byte elements tile a byte.

### 4.2 Relationship to the existing `quant` record and registry

- The neutral sidecar is the **superset shape** of KISS-CLASSIFY-6.3-0009's `quant` record. That
  clause's `family` ↔ this RFC's `encoding`; its `scale_placement` ↔ this RFC's `scale.placement`;
  its `sub_byte_bits` / `block_elems` ↔ `byte_layout` / `quant_block.block_size`. The
  carried-not-keyed rule (§6.3-0009) applies unchanged: **the sidecar MUST NOT be folded into
  `structure_key`.**
- The `encoding` and `scale.placement` vocabularies remain **owned by the external
  quantization-token registry** (KISS-CLASSIFY §4) and **mirrored** in the KISS-Announce EXT axis
  (§7.2; bit 1 `DLPACK_EXT_MX`, bit 3 `DLPACK_EXT_AFFINE`, bit 2 `DLPACK_EXT_GGML` already
  reserve the seed encodings). KISS pins the projection and the mirror, never the external
  vocabulary's semantics.

## 5. Worked example — the full MX advert

A Microscaling (OCP MX) block-quantized weight, advertised through the sidecar:

```
sidecar {
  logical_dtype = <element format from the DLPack boundary>   // e.g. e2m1 / e4m3fn element
  encoding      = mx                                          // OPEN registry seed
  quant_block {
    block_size  = 32                                          // 32 elements per shared scale
    scale_dtype = f8e8m0                                      // OCP E8M0 shared exponent
  }
  scale {
    placement   = separate_buffer                             // scale is a referenced sibling operand
    dtype       = f8e8m0                                      // OCP E8M0
    granularity = per_block                                   // one scale per 32-element block
  }
  // byte_layout / alignment as the element packing requires
}
```

Read as: `{ encoding = mx, quant_block.block_size = 32, scale.dtype = f8e8m0 (per-block, OCP
E8M0), scale.placement = separate_buffer }`. None of these bytes enter `structure_key`; a
consumer resolves the cell by geometry/op-family/primary dtype first, then reads this sidecar to
confirm the kernel it selected accepts an MX-32/E8M0/sibling-scale operand.

## 6. Migration

1. **Publish the neutral shape (this RFC).** The field union in §4 becomes the shared external
   registry shape KISS-CLASSIFY §4 already references by name.
2. **Both cosignatories retire their private descriptors.** Fuel's FDX struct and Baracuda's
   `QuantFacts` mirror are superseded by this shape; each projects to/from it. This is the whole
   point of neutralizing — one shared shape replaces two private mirrors, closing the drift.
3. **No `structure_key` regen.** Because nothing here touches key bytes, there is **no version
   bump and no golden/PROVENANCE regeneration** — unlike `sk3`. The only registry motion is
   mirroring any new `encoding` seed spellings into the KISS registry (EXT axis).

## 7. Open sub-questions for the editors

- **Wire form.** §4 pins the neutral *field set and names*; the concrete serialization (the
  KISS-CLASSIFY §6.3 record extension vs a standalone sidecar blob referenced by the operand) is
  the editors' to pin.
- **Seed spellings.** The five `encoding` seeds are recommended, not closed. Editors confirm the
  exact recommended spellings and the EXT-bit mirror for each (`mx`/`ggml`/`affine*`).
- **`logical_dtype` restatement.** Whether the sidecar restates the DLPack element dtype (for
  self-description) or references the DLPack header by pointer — a redundancy-vs-locality call.
- **`byte_layout` vocabulary.** The packing-order codes (nibble hi/lo, block interleave) are the
  one field with no incumbent registry; they may seed their own open sub-registry.

## 8. Non-goals

This RFC does **not** change `structure_key` bytes, does **not** add any encoding to the identity
key (MX/DLPack stay out per `sk3`/D4), does **not** redefine any DLPack or OCP element-format
*value* (it cites them), and does **not** close the `encoding` registry. It is exactly the
neutral quant/layout sidecar — the successor to two private descriptors — and its registry home.

## 9. Cosignatory status

- **Fuel (cosignatory; FDX-schema originator)** — signs off on the neutralized field union as the
  successor to its private FDX descriptor; contributes the FDX field set with vendor/language
  spellings erased. Concurs that `scale.placement = separate_buffer` is the neutral spelling of
  its model-B "scale as a referenced sibling operand."
- **Baracuda (cosignatory; provider)** — signs off and **drops its private `QuantFacts` mirror**
  in favor of this shape; concurs the sidecar stays carried-not-keyed (§6.3-0009) and out of the
  identity key (consistent with its `sk3` provider position).
- **Maintainer** — no objection outstanding; the boundary (DLPack owns interchange; sidecar
  carries the remainder; nothing enters the key) matches the PRIOR-ART recommendation and the
  `sk3`/D4 line. RFC ready for editor adoption.