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

**Candidate 2, NOT listed: a `c` role hint on the contraction (RU.2 Group 4 A).** CireSnave's 6.35 says "as KISS's open question 771 Q3 is ruled". "771 Q3" is KISS's own phrase for open question 3 of `docs/options-771-contraction-precision-and-accumulate-into-c.md` (RU.2 note lines 48 and 68; EXPECTATIONS 6.35). **READ 2026-11-26:** KISS issue #771 is CLOSED (2026-11-15, "DECIDED and shipped in 0.18.0 (sk8, #822). Board 440 Q1 A and board 443 ... accumulate-into-C stays ordinary ops, no new spelling, no recipe list"), and Unpopped's lane confirmed "settled, no accumulate-into-C input held". That reads as Group 4 option B (nothing in the key), so there is **no byte-altering item** here; this lane has not confirmed that the 11-15 closure is the ruling 6.35 means. The compute-dtype half stays "no new coordinate" (KISS-CLASSIFY-6.1-0005). **Pending pm-kernels' confirmation; until then it is not in this batch.**

## Not in this batch, and why

- **Slang precision and NaN-mode:** RULED by the PM (A now, D later, no NaN-mode coordinate): no bytes. Nothing to list.
- **A finite-only E3M4** (Mojo's docstring): a separate token only if a real device or language produces that reading; no consumer has said so. Not listed.
- **Derived major-ness pair, scale-factor output coordinate:** 6.35 says no and later.
- **Mojo 49, CUTLASS 15, CUTLASS 35:** not KISS bytes (an Unpopped label, an op-vocabulary question, a Fuel planner matter).

## Consumer asks since batch 1: what was and was not read

- **Unpopped (their message 2026-11-26, from `docs/ir-union-asks.md` 1a/1b):** open asks are (1) CUTLASS 3 more than one batch role and nested modes: batch axes are already a caller-declared set whose product gives the class (KISS-CLASSIFY-6.6-0023); nested shape is not keyed (RU.2 measured 2x3, 3x2 and 1x6 batches derive the same token), so nothing is needed; (2) CUTLASS 31 source C and compute dtype: see Candidate 2 (no byte item); (3) CUTLASS 23 scale-factor output: later (6.35), option B first; (4) UE5M3: Candidate 1; (5) CUTLASS 5 major-ness: nothing until a collision is named; (6) CUTLASS 15: op vocabulary, asks nothing. One UNVERIFIED flag from them: `f8e4m3fnuz` and `f8e5m2fnuz` are RESERVED in 6.1 (no computation semantics); a HIP emitter for CDNA3 would need them usable; **no emitter has asked, so nothing is listed**.
- **fishstringer (their message 2026-11-26):** no new token, coordinate or key for batch 2; Slang closed as ruled.
- **vulkane: no reply yet.** A regex over its `ROADMAP.md` found only its own `vulkan:` vocabulary rows; that regex also missed Unpopped's asks, so it is a weak control: "not found", not "none".

## Veto window

List posted: 2026-11-26 (this PR, draft). The 24 hours count from the PM's relay of this list to CireSnave; the relay time is the PM's to record. No shipping PR opens before it. Meanwhile consumers decline unknown tokens by name (0.24.0 note).

— lane: kiss
