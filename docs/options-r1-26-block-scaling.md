# Options note: R1.26, what is left of block-scaled sub-byte floats after the element formats and the scale key landed? (#32)

**Informative. No normative text. For CireSnave's decision; the PM turns each question into a board item.**
Read at `origin/main` `2e65634`, 2026-10-26. Paths are `file:line` at that commit. Issue: KISS#32 (Baracuda, Fuel, the
architect). Roadmap row: R1.26, gated `[wait: schema event]`. Nothing here freezes. **Much of #32 has landed since it was
filed, so the row is stale and the questions are fewer than the issue lists.**

---

## What has landed, so it is not asked again

- The MX **element** formats `f4e2m1`, `f6e2m3`, `f6e3m2` (KISS-CLASSIFY-6.1-0014 to -0016, `spec/classify.md:612-648`, PR #675)
  and the MX **scale** dtype `f8e8m0` (6.1-0013, `:585`), with their constants checked against OCP Microscaling v1.0 (#681).
- The `quant` record carries `{family, sub_byte_bits, block_elems, scale_placement, dequant_form}` for binding, **not** for the
  key (`spec/classify.md:698`, `:732`); only `dequant_form` is keyed, from `sk5` (6.3-0009a).
- From `sk6` the **scale dtype** of a quantized dense-contraction cell is keyed, as a trailing `q<dtype>` coordinate
  (KISS-CLASSIFY-6.6-0022, `spec/classify.md:1436`), described there as "the correction of a numerically wrong cache hit".
- Open searches with controls: `block_length`, `scale_group_axis` and `mxfp` appear **nowhere** in `spec/`, while `e8m0` appears
  30 times, so the search reaches the right files.

So what is left of the issue's keyed attribute `{block length, scale dtype, group axis}` is the **block length** and the
**group axis**; the scale dtype is done.

## Q1. Should the block length be part of the structure key?

**Example.** An `e2m1` weight with an `e8m0` scale in blocks of 32 elements (MXFP4) and one with the same scale dtype
in blocks of 16 are two different dequant kernels. With only the scale dtype keyed, they derive **one key**, so a cache serves
a kernel that dequantizes with the wrong block size: the same wrong-cache-hit that `sk6` corrected for the scale dtype. (Today the
two real formats differ in scale dtype as well, `e8m0` against `e4m3`, so the collision is not visible yet.)

**Options.**
- **A. Key it now,** as its own key-schema event. *Cost:* every consumer regenerates its keys once more.
- **B. Key it at the next scheduled key-schema event.** *Cost:* the gap stays until then, hidden while MX and NVFP4 are
  the only formats.
- **C. Do not key it:** the producer owns the collision, as it does for cell-mates (KISS-CLASSIFY-6.6-0020). *Cost:* the
  unrecognised half of the same hazard `sk6` just closed.

**Recommendation: B.** It is a real, if latent, hole, and the precedent of `sk6` says to key it. A key event is expensive
for three consumers, and nothing is wrong on the wire until a third format appears. **Do not choose C.**

## Q2. Is the set of block lengths closed?

If it is keyed, the key spells the integer. A **closed set** `{16, 32}` (NVFP4 and OCP-MX) keeps membership checkable
and the token space bounded; a new length is a deliberate, additive spec change. An **open integer** accepts any length with no spec change.
**Recommendation: closed `{16, 32}`.** Two formats exist and a third length can be added as one value.

## Q3. Is the group axis keyed, and spelled as an index?

For a dense contraction the scale typically groups along the K axis (the issue's words), and the caller already designates K by role
hint (KISS-CLASSIFY-6.6-0016 and the role hints of 6.6-0012). So "the K axis" is already unambiguous. **A. Spell an explicit axis index** (the issue's
proposal; it keeps a transposed or tiled operand honest). **B. Imply it** (K for a contraction, innermost otherwise) and add an
explicit index the day a format groups along another axis. **Recommendation: B.** No format in the issue groups any other
way, and every degree of freedom keyed now is one more byte in every key.

## Q4. Who pins packing inside a block and the conversion rounding and saturation?

The element formats have no inf or NaN to overflow into, so a conversion needs a pinned rule, and six-bit elements need a
bit order inside the block. OCP v1.0 fixes **no packing** (#681), and Unpopped reports conventions that I have **not**
checked against CUTLASS, Mojo or Triton. I did not read the OCP conversion text for a rounding mode.
**Options.** **A.** KISS-Ops pins both as conventions now, with vectors. **B.** Wait until Baracuda measures what the hardware
conversion does (it has the devices) and pin that. **Recommendation: B.** Pinning a convention that a device then contradicts is
worse than waiting; ask Baracuda for the measurement first.

---

## Summary

| Q | Question | Recommend | Bytes change? | Note |
|---|---|---|---|---|
| 1 | key the block length | B (next key event) | yes (key token) | `sk6` precedent for the scale dtype |
| 2 | closed `{16, 32}` | closed | with Q1 | new length is additive |
| 3 | explicit group axis | B (imply K / innermost) | no | add an index when needed |
| 4 | packing, rounding, saturation | B (measure first) | no | OCP fixes no packing |
