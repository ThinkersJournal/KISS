# Design: phasing FDX out in favour of KISS clauses (V1..V22 mapping and draft clauses for the NEEDS rows)

Status: **design with draft clause text; nothing here is a clause yet.** Written 2026-11-09 against `origin/main` 2727c22 (conformance 0.13.0).
Ruling: CireSnave 2026-11-09, verbatim, relayed by the PM: *"FDX should have been phased out in favor of a KISS standard."*
Also his 395 ruling: *"KISS owns the standards.  FireLine implements the seam standards' format information.  KISS-ref implements the whole of KISS."*
Source read: Fuel `docs/specs/dlpack-extension.md` section 8 at `de12d8ef` (it has **22** validators, V1..V22). Read, not run.
Fuel's answer on the DROPs (relayed by the PM, 2026-11-09): confirm V2, V15, V22; V3 conditional; **object to dropping V8 and V9** (they are the no-OOB guard that V20 and V21a lean on).

## 1. The mapping, as it stands

| Class | Validators | Where it goes |
|---|---|---|
| Mapped to an existing clause | V1 (text framing, CONTRACT-6.1-0002/-0003/-0008, 6.11-0002), V4 (CLASSIFY-6.1-0014..-0016), V10 (CONTRACT-6.5-0019, now with slot dtype and shape), V11 (CONTRACT-6.5-0005) | none needed |
| By generalisation | V16, V17 (Dispatch expressions are a general arithmetic language, CONTRACT-6.6-0006/-0009; the fixed term list of an FDX affine extent has no counterpart, so its duplicate/degenerate rules are not carried) | group A |
| NEEDS a clause | V7, V14 (extent range, unbound symbol); V12, V13, V8, V9, V3 (buffer bounds); V5, V6 (quant); V18..V21 (paged gather) | groups A to D |
| Dropped | V2 (flag word), V15 (no pointers: a KISS contract is text), V22 (flag implication) | confirmed by Fuel |

## 2. Group A: bind-time symbols (V7, V14, V17)

Already designed: `docs/design-r1-22-bind-time-obligations.md` section 3.1 (Contract section 6.12, -0001..-0006: bound environment, unbound symbol, range, overflow, evaluation order; CireSnave board 379, "1"). Nothing new here except: **land it**. It is the carrier for V7/V14/V17 and for every bind-time check below.

## 3. Group B: buffer bounds (V3, V8, V9, V12, V13)

Fuel's objection stands: these are the no-OOB guard. KISS has the *inputs* (signed strides CONTRACT-6.5-0005, base offsets 6.5-0006, extents, `alignment` and dtype in the Classify operand description) and **no obligation that uses them**. Proposal: extend the R1.22 bound environment with **one byte size per pointer operand**, and add bind-time clauses (Contract 6.12, each naming its typed decline, per R1.22 -0002):

- **Draft -0007 (window).** At bind time, for each pointer operand with element byte width `w`, base offset `off` (elements) and signed strides `s_i` over extents `e_i`: let `lo = off + sum over s_i<0 of (e_i-1)*s_i`, `hi = off + sum over s_i>0 of (e_i-1)*s_i`. The byte window `[lo*w, (hi+1)*w)` MUST lie within `[0, size_bytes)`; otherwise decline `buffer_out_of_bounds`. An axis with `e_i == 0` is **skipped** in the two sums (it contributes nothing), and the window over the remaining axes and the offset are **still checked**, so a descriptor whose offset or other extents run past the buffer is rejected even when the tensor is empty (PM's pick of Fuel's stricter reading, 2026-11-09; validation only, no wire byte moves). (V13 and V8: *extent times stride must not exceed the buffer.* The sum over axes, not the maximum over axes: for shape `[4,8]` strides `[8,1]` the window needs 32 elements, and a max-over-axes bound says 25.)
  **Sub-byte and quant operands (Fuel's third difference).** For an operand whose dtype is sub-byte (`i4`, `u4`, `b1`, the MX element dtypes) or that carries a `quant` record, `w` is **not** a whole number of bytes, so the window is counted in **bits**: with `b` the bits per stored element (the storage dtype, per the V3 sentence below), `[lo*b, (hi+1)*b)` MUST lie within `[0, 8*size_bytes)`, and extents, strides and `off` are counted in stored elements. Without the V3 sentence `w` is undefined for a quantized operand.
- **Draft -0008 (offset).** `off*w <= size_bytes` (in bits: `off*b <= 8*size_bytes`), else `buffer_out_of_bounds` (V9's offset rule). It applies to **every** pointer operand, an empty one included. This matches Fuel's `check_v13`, which skips only the empty *axis*; an earlier draft of -0007 exempted a whole empty operand and was changed by the PM's pick above.
- **Draft -0009 (alignment).** The operand's base address plus `off*w` MUST be a multiple of the operand's stated `alignment` (Classify operand description; Contract `alignment_bytes`), else `misaligned` (V12; FDX's 256 is a policy value a producer states, not a KISS constant).
- **Draft -0010 (no aliasing between pointers).** Two pointer operands' byte windows MUST NOT overlap unless the contract says `in_place` (Interface `in_place`) for that pair or they are slots of one bundle (CONTRACT-6.5-0019), else `buffer_overlap` (V9's overlap rule). **Open:** a read-only pair may legitimately alias; scope the rule to pairs where at least one is written?
- **V3 (honest base dtype).** KISS has no DLPack base struct, so the *mechanism* is dropped; the *property* is "when an operand's meaning rides outside its dtype, the dtype is the storage dtype". CLASSIFY-6.3-0009 keeps the `quant` record separate from `dtype`, but no sentence says the dtype is the storage one. **Draft:** one sentence in 6.3-0009: *"An operand carrying a `quant` record states its **storage** dtype; the dequantized type is never the operand's dtype."* Not yet checked against Unpopped's and Baracuda's emitters.

## 4. Group C: quantization (V5, V6)

What KISS has: the `quant` record `{family, sub_byte_bits, block_elems, scale_placement, dequant_form}`; `family` and `scale_placement` are **opaque tokens owned by the external quantization-token registry** (CLASSIFY-6.3-0009), `dequant_form` is keyed (6.3-0009a), the MX scale operand is role-hinted (6.6-0022). What it lacks: any rule that couples `family` to `scale_placement`/granularity (V5), and any N-dimensional block geometry (`block_elems` is a scalar; V6's per-axis `block_shape` has no carrier).

- **Options.** (i) Regime coherence (V5: GGML scales inline, MX per-block separate, AFFINE never per-block) is a rule **of the registry**, not of Classify, because Classify does not interpret the tokens; KISS adds one clause that a registry entry MUST state the legal `scale_placement` set for its `family`, and a reader MUST decline a record outside it. (ii) KISS owns the regimes as a closed table. **Recommend (i)**: (ii) would make Classify interpret what it deliberately does not.
- **V6 (scale shape).** Needs block geometry. **Open (CireSnave or Unpopped):** is `block_elems` (a scalar, along the contraction axis) enough for every family KISS cares about, or is an N-d `block_shape` wanted? FDX's `AFFINE_BLOCK` uses `block_shape`/`block_axes`. Until that is answered a V6 clause would be a guess. No draft text.

## 5. Group D: paged KV (V18..V21), as ops over block_table operands

PM direction (2026-11-09): express paged KV as ops over `block_table` operands, no new descriptor. Then V18..V20 (pool geometry coherence, base honesty, pool backing) are descriptor checks with nothing to check: **dropped with the struct**, and what remains is the data-dependent part.

- **V21c/d (block id range).** `gather` already reads through a runtime index with `oob_policy` in `{skip, clamp, zero-fill}` and always treats a negative index as out of bounds (OPS-6.11-0004, 6.19-0015). What KISS lacks is a **typed-reject mode**: FDX's verdict is `BlockIdOutOfRange`, not a silent skip. **Open ruling:** add `oob_policy = 4 (reject)` (a new frozen ordinal, additive on the wire since `0` is reserved and `1..3` are taken), or declare that a consumer wanting reject validates the index operand itself at bind time. A data-dependent reject is a **runtime** verdict, not an environment-bound one (R1.22 section 4 already marks data-dependent output sizes out of scope; OPS-6.20-0004), so an `oob_policy` reject would be a kernel behaviour a conformance vector must pin.
- **The unmapped sentinel stays exempt (Fuel, `check_v21c`).** A paged block table marks an unmapped slot with a sentinel id that is `>=` the number of blocks; Fuel skips `id == unmapped_sentinel` and rejects every other id outside `0 <= id < num_blocks`. A reject policy that also rejected the sentinel would break ragged tables (sequences shorter than the table width). So the reject clause MUST state: a stated sentinel value is **not** an out-of-bounds index for `reject`. **Open:** where the sentinel is stated (an OpAttrs field, or a launch scalar); an extra field is a further wire change and goes in the same HOLD PR.
- **V21d lengths.** `0 <= L <= max_seq_capacity` and `ceil(L/block_size) <= max_blocks_per_seq`, with `L == 0` legal: expressible as bound-environment range checks on a launch scalar `context_len` (group A, -0004), no new clause.
- **Byte impact.** A new `oob_policy` ordinal moves OPS-6.19-0015 and the OpAttrs byte for gather; everything else here is additive bind-time text.

## 6. Decisions needed, in order

1. Land R1.22 Contract section 6.12 (group A): already ruled by CireSnave (board 379).
2. Group B clauses -0007..-0010 and the V3 sentence: design for the PM and Fuel to read; Fuel to confirm the window formula against `dlpack_view.rs`.
3. Group C: option (i) vs (ii), and scalar vs N-d block geometry: **CireSnave** (touches Classify).
4. Group D: `oob_policy = 4` vs consumer-side validation: **CireSnave** (touches a frozen enum).

## 7. Staging and what this costs to be wrong

Docs only until step 1; the group B/C/D clause PRs each take their own version from the PM. No existing vector or key moves from groups A and B. Group D with `oob_policy = 4` moves the gather OpAttrs vocabulary (additive). Not covered: FDX's Rust ABI crate (FireLine's, 395) and the FDX struct layouts, which KISS does not adopt.
