# Union batch 2: the LIST (for the veto window; nothing is minted by this document)

**Informative. No normative text, no stored byte moves.** Posted for CireSnave's veto window under EXPECTATIONS 6.35 (board 501 "A"): a batch is LISTED first, with the exact tokens and what changes per consumer; the shipping PR opens no sooner than 24 hours after the PM relays this list. Prepared 2026-11-26 by the kiss lane on pm-kernels' TASK (batch-2). Evidence is marked READ (what was read, where, when) or UNVERIFIED.

## The list

**Candidate 1, one token: `f8ue5m3`** (the unsigned UE5M3, a block-scale storage format).
- **What it would be:** one byte, no sign bit, 5 exponent bits, 3 mantissa bits, no infinity, a single NaN encoding `0xFF`. It would sit beside `f8e8m0` and `f8ue4m3` (§6.1-0013, §6.1-0020) as a scale type: not a valid operand-0 dtype (§6.6-0021), usable in the contraction `<sdt>` coordinate (§6.7-0006). Closed set 34 to 35 (32 usable, 3 reserved). Next clause number: 6.1-0021. No `sk` bump on the 0.24.0 precedent.
- **Evidence READ:** PTX ISA 9.5, §5.2.3 "Alternate Floating-Point Data Formats", read 2026-11-26 through a summarising web fetch that returned the sentence verbatim: "`ue5m3` data format: This data format is a 8-bit unsigned floating-point format with 5 bits for exponent and 3 bits for mantissa. The `ue5m3` encoding does not support infinity. `NaN` value is limited to `0xff`. A register variable containing single `ue5m3` value must be declared with `.b8` type." Table 9 (§5.2.5.1) lists a packed `.ue5m3x2` (register type `.b16` is inferred from flattened text).
- **UNVERIFIED, and the reason this is a candidate and not a recommendation:**
  1. **The exponent bias is not stated** for `ue5m3` (nor for `ue4m3` or `ue8m0`) in the text read. Without it the value function cannot be pinned; the bit layout and NaN can. `f8ue4m3`'s bias 7 came from CUTLASS, which has no UE5M3 (`float8.h` and `ScaleFormat` at `5c179ede` and `main` carry UE4M3 and UE8M0 only).
  2. **Single source.** No CUTLASS, CUDA header or cuBLAS text naming UE5M3 was read. The tcgen05 descriptor code `UE5M3 = 2` (PTX Table 53, `.kind::mxf4nvf4`) is this lane's earlier read (`docs/dtype-completeness-audit.md` §6); the table was not reachable in today's fetch (cut off before §9.7.18).
  3. **No consumer has been named** that stores or reads UE5M3 data. Unpopped's `docs/ir-union-asks.md` says "ue4m3/ue5m3 exist in PTX; the gap is real, board 501".
- **Recommendation:** list it, but ship it only if (a) the bias is found in a primary source (PTX PDF or a header) and (b) a consumer row needs it. If neither, drop it from batch 2; a declined candidate costs nothing.
- **What changes per consumer** (facts about KISS's own surface; consumers' own code is UNVERIFIED): unpopped `ElementKind` gains a variant (not `non_exhaustive`, so exhaustive matches break); fuel `element_kind_to_dtype` maps or declines by name; kiss-ref gains the token to keep byte-match; every emitter declines it by name until it lowers it (the 0.24.0 note).

**Candidate 2, NOT ready to list: a `c` role hint on the contraction (RU.2 Group 4 A).** CireSnave's 6.35 says "as KISS's open question 771 Q3 is ruled". I found no recorded ruling of 771 Q3 in `docs/options-771-contraction-precision-and-accumulate-into-c.md` or ROADMAP at `origin/main` e628c96 (only the 2026-11-12 board 427 ruling on the tag question). It is byte-altering (a presence coordinate). **Needs from pm-kernels: the Q3 ruling, or confirmation there is none.** Until then it is not in this batch.

## Not in this batch, and why

- **Slang precision and NaN-mode:** RULED by the PM (A now, D later, no NaN-mode coordinate): no bytes. Nothing to list.
- **A finite-only E3M4** (Mojo's docstring): a separate token only if a real device or language produces that reading; no consumer has said so. Not listed.
- **Derived major-ness pair, scale-factor output coordinate:** 6.35 says no and later.
- **Mojo 49, CUTLASS 15, CUTLASS 35:** not KISS bytes (an Unpopped label, an op-vocabulary question, a Fuel planner matter).

## Consumer asks since batch 1: what was and was not read

- **READ:** Unpopped `docs/ir-union-asks.md` and its 2026-11-25 update (origin/main): the open KISS items are UE5M3, the 128-bit rows (now shipped), Slang (ruled, no bytes) and the RU.2 groups above. Unpopped's 2026-11-26 STATUS (via Synapse) asks nothing new of KISS.
- **NOT READ in full:** fishstringer and vulkane. A regex for "KISS ... token/dtype/coordinate ask" over their `ROADMAP.md` at origin/main matched only vulkane's own vocabulary rows (`vulkan:` namespace, not KISS tokens) and nothing in fishstringer. That regex found no Unpopped ask either, so it is a weak control: absence here means "not found by this search", not "none". Those two lanes should say if they have an ask.

## Veto window

List posted: 2026-11-26 (this PR, draft). The 24 hours count from the PM's relay of this list to CireSnave; the relay time is the PM's to record. No shipping PR opens before it. Meanwhile consumers decline unknown tokens by name (0.24.0 note).

— lane: kiss
