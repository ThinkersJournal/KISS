# Union batch 2: NO BATCH (the list is empty; no veto window is needed)

**Informative. No normative text, no stored byte moves, nothing minted.** Prepared 2026-11-26 by the kiss lane on pm-kernels' TASK (batch-2) under EXPECTATIONS 6.35 (board 501 "A"). Result: **no candidate qualifies, so batch 2 is empty and there is no shipping PR.** Evidence is marked READ (what, where, when) or UNVERIFIED. Origin/main at e628c96 (0.24.2).

## Why empty

The rule for a candidate (pm-kernels, 2026-11-26): a UNION row, a token/role/coordinate with a byte effect, that an Unpopped union row NEEDS, with a primary source. An emitter's decline row is not a union need.

| Item | Union row (byte effect)? | Source read | Consumer row that needs it | Verdict |
|---|---|---|---|---|
| `f8ue5m3` (UE5M3 scale) | Yes: a token (closed set 34 to 35) | PTX ISA 9.5 §5.2.3, read 2026-11-26 (summarising fetch, sentence verbatim): "8-bit unsigned floating-point format with 5 bits for exponent and 3 bits for mantissa ... does not support infinity. `NaN` value is limited to `0xff`" ; Table 9 `.ue5m3x2`. **No bias stated**; Unpopped read the same text locally and holds no bias source either. CUTLASS (`5c179ede`, `main`) has no UE5M3. | Unpopped's tcgen05 `mxf4nvf4` scale row (ROADMAP U47) names it but is not built or scheduled and "no kernel of ours uses it today"; vulkane: no need, no objection; fishstringer: none | **Not listed**: no primary bias, no consumer. Revisit when both exist. |
| `c` role hint (RU.2 Group 4 A) | Yes: a presence coordinate | RU.2 note lines 45-52: "Group 4. Source C and a compute dtype ... **A. A `c` role hint ... Byte-altering. B. Nothing in the key ...** Compute dtype: ... **Recommendation: no new coordinate**" and "This is the open question 3 of `docs/options-771-...`: **decide it there, once**" | Unpopped CUTLASS 31 | **Not in batch 2.** PM confirmed 2026-11-26 that 771 Q3 is UNRULED (board item 521). Separately, KISS#771 itself closed 2026-11-15: accumulate-into-C stays ordinary ops, no new spelling (board 440 Q1 A, 443). |
| CUTLASS 3, more than one batch role, nested modes | No | KISS-CLASSIFY-6.6-0023: batch axes are a caller-declared SET whose product gives the class; RU.2 measured 2x3, 3x2, 1x6 batches derive the same token | Unpopped | **Already covered** (control: `test_classify_batch_role_derivation` is in spec/classify.md; `k_more` is in `derive.rs`, 0.24.0's K change). Nested shape is not keyed: nothing to add. |
| CUTLASS 23, scale-factor output (SFD) | Option A yes, option B no | RU.2 Group 5: "B first (no bytes), A only if a consumer shows the key admitting a kernel that writes the wrong scale dtype" | Unpopped | **Later** (6.35). Grep for `out_scale`/`o<sdt>` in spec and `conformance/src` at e628c96: 1 hit, in `reference_vectors.rs` only; no spec text. |
| Slang precision and NaN-mode | No | KISS#849 options note, RULED by the PM (A now, no NaN-mode) | Unpopped Slang 50/52; fishstringer | **No bytes.** Closed. |

## Considered, not proposed

- **`i2` / `u2`** (signed [-2,1] / unsigned [0,3] 2-bit). Asked by vulkane. Source READ by vulkane: Metal Shading Language Specification 4.1 (2026-06-04) §2.21 Table 2.14 p.79 `int2b_format`, `uint2b_format`, used as `packed_numeric_type` elements. Consumer: vulkane-msl-emit's decline row (an emitter that lowers none of it) and possibly an Unpopped B1 tile type that has not asked. Not a union need today; no packing layout is chosen. Grep for `i2`/`u2`/`int2b` in spec: 0 hits (control: the same grep finds the vulkan.md line).
- **`mxint8`** (OCP MX INT8, scale `f8e8m0`). Asked by vulkane. Source: SPIR-V unified1 3.2.55 `MXInt8EXT` 4227; GLSL `GL_EXT_ocp_microscaling_types` (`floatmxint8_t`); names and values only, element layout UNREAD by that lane. Already recorded in `docs/dtype-completeness-audit.md` as a GAP, "DECLINED at vocabulary v8", and a naming-list candidate (block-scaled `i8` versus a token). Decline-row consumers only.
- **`f8e4m3fnuz` / `f8e5m2fnuz` made usable** (they are RESERVED in 6.1). Flagged by Unpopped for a CDNA3 HIP emitter (Mojo 71, Baracuda). No emitter has asked; UNVERIFIED.
- **A finite-only E3M4** (Mojo's docstring): a separate token only if a real device or language produces that reading; none named.
- **Vulkane's MSL/SPIR-V mapping confirmations** (MSL fp8 e4m3 = `f8e4m3fn`, e5m2 = `f8e5m2`, ue8m0 = `f8e8m0`, fp4 e2m1 = `f4e2m1`): no new token; their NaN/infinity columns were not cross-checked here (MSL's e5m2 NaN column is flagged by them as an extraction artefact).
- **Derived major-ness pair, CUTLASS 15 (trmm/symm/hemm), CUTLASS 35, Mojo 49:** 6.35 says no, or not KISS bytes.

## Asks received (batch-2 request, 2026-11-26)

fishstringer: none. Unpopped: CUTLASS 3, 31, 23, UE5M3, 5, 15 (all rows above) plus the fnuz flag. vulkane: `i2`/`u2`, `mxint8`, mapping confirmations; no coordinates and no collectives.

## What happens next

Nothing ships. If CireSnave rules board 521, or a consumer later has a union row needing `f8ue5m3` (with a primary bias), that opens batch 2 under 6.35 with its own LIST and window.

— lane: kiss
