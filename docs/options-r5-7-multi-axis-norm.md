# Options note: normalization over several axes (Ops Appendix D.11 (i), `norm_axis`)

**Informative. No normative text.** For CireSnave's decision through the PM. Read at KISS `origin/main` `d47b99e` (conformance 0.8.1).
His word, 2026-11-07, via the PM, verbatim: *"PyTorch's normalized_shape allows LayerNorm over several trailing axes.  As such, Unpopped's IR must as well.  If Unpopped's IR can express that then KISS must have the ability to decribe it at the seams where it applies."*
The PM's reading: a single-axis `norm_axis` is not the end state, and composition from `reduce` is not enough, because the op must name several axes as one op so that a fused kernel keeps its identity. It rides the 0.9.0 Breaking batch with the `keepdim`/`stability` flags (one layout break for readers).

## What is there today

- `softmax`, `log_softmax`, `rms_norm`, `layer_norm` each carry exactly one OpAttrs field, `norm_axis` (`u8`, mandatory, `0 <= norm_axis < MAX_RANK`): 6.13-0004, 6.13-0008, 6.19-0031.
- `gamma` (`input(1)`), `beta` (`input(2)`) and `eps` (`param(0)`) are operands and a param, not OpAttrs (6.19-0031, §6.13 operand ordering).
- The decompositions use `reduce`, `reduce_mean`, `reduce_var` (§6.13), which already take a **set** of axes (`reduce_axes`, 6.19-0020) and a divisor that is the product over all selected axes (`reduced_count`, 6.12-0001). So the arithmetic for several axes exists; only the four ops' attribute is single-axis.
- `reduce_axes` is one `u16` LE with a rank-aware total precedence (6.19-0020): `0xFFFF` all axes, `0xFFFE` the single trailing axis of a rank > 1 operand, otherwise the subset mask in the low byte (`0x0001..0x00FF`); `0x0000` never emitted for a reduction.

## The two ways to name several axes

**B. A `u16` axis mask, the `reduce_axes` encoding.** Replace `norm_axis` (`u8`) by `norm_axes` (`u16` LE) with exactly the `reduce_axes` categories and precedence, except that `0x0000` (none) is a typed decline. Any subset is expressible; PyTorch's trailing range and ONNX's "axis to the end" are particular masks.
**C. A count field.** Keep `norm_axis` (`u8`, the first normalized axis) and add `norm_count` (`u8`, default `1`, `>= 1`): the axes `norm_axis .. norm_axis + norm_count - 1`. Contiguous trailing or interior ranges only; this is exactly PyTorch's `normalized_shape` and ONNX's `axis`.

## Rules common to both

- **`gamma` and `beta`.** Their element at a coordinate `c` of the iteration frame is read at `c` restricted to the normalized axes, with extent equal to `x`'s extent on every normalized axis and stride `0` on every other axis (the §6.11-0001 broadcast rule, the same presentation any broadcast operand has). So the parameter is the normalized sub-shape, as in PyTorch (`weight.shape == normalized_shape`), whether or not the axes are contiguous. A `gamma` or `beta` whose normalized-axis extents disagree with `x` is a typed decline, as an operand-shape disagreement already is.
- **Statistics.** The mean, the mean square and the variance are formed over all selected axes together (the divisor is the product of the selected extents, `reduced_count`), never axis by axis. `layer_norm`'s two-pass variance (mean first, then the mean of squared deviations) is unchanged; its `eps` is added to the variance.
- **`softmax`, `log_softmax`.** The max, the sum and `logsumexp` are taken over the selected axes together.
- **Keepdim.** The decompositions name `keepdim = 1` explicitly (the broadcast-back of 6.11-0008) once the `keepdim` flag is real.
- **Default and byte consequence.** An existing single-axis blob is one byte today; both options make it longer (B: two bytes; C: two bytes), so the four ops' bytes move. This is Breaking either way, and the explicit-emission rule (6.19-0005) means the default is written out, not omitted.

## Byte layouts and golden vectors

| Case | B (`norm_axes`, `u16` LE) | C (`norm_axis`, `norm_count`) |
|---|---|---|
| `softmax`, rank 3, last axis | `FE FF` (`0xFFFE`, trailing) | `02 01` |
| `layer_norm`, rank 3, axes `{1,2}` (PyTorch `normalized_shape=(d1,d2)`) | `06 00` (`0x0006`) | `01 02` |
| `layer_norm`, rank 3, all axes | `FF FF` (`0xFFFF`) | `00 03` |
| `rms_norm`, rank 4, axes `{0,2}` (non-contiguous) | `05 00` | not expressible |
| `softmax`, rank 1 | `FF FF` | `00 01` |

(The existing precedence makes B's bytes a function of rank as well as the set, exactly as for `reduce`; C's bytes are rank-free but a count that runs past the rank is a decline.) Vectors to add either way: the five rows above, one decline per illegal value (B: `0x0000`, a reserved `0x0100..0xFFFD`, a bit at or above the rank; C: `norm_count = 0`, `norm_axis + norm_count > rank`), and a numerical vector per op showing that two axes together differ from one axis then the other.

## Readers affected

Anything that decodes the four ops' OpAttrs (`opattrs.rs`, `opattrs_value.rs`, the schema table in `opattrs_schema.rs`), the shape and decomposition evaluators for the four ops, and every foreign reader (Fuel, Baracuda, Unpopped, kiss-ref's `op_manifest`/`kiss-ops-vocab`) that builds or recognizes these blobs. The Classify key is **not** affected: a norm is not a `reduce` cell, and the structure key carries no norm field.

## Comparison

| | B mask | C count |
|---|---|---|
| Expresses PyTorch `normalized_shape`, ONNX `axis` | yes | yes |
| Non-contiguous axes | yes | no |
| One axis encoding across the spec (`reduce`, `prefix_scan`, `reduce_var`, the norms) | yes | no (a second convention) |
| Bytes independent of operand rank | no | yes |
| Reader code | reuse the `reduce_axes` decoder | a new two-field check |
| Risk | a mask bit beyond the rank must be declined | a range past the rank must be declined |

## Recommendation: B

It reuses the one encoding the spec already has for "a set of axes", so a reader has one decoder and the decompositions (`reduce_mean`, `reduce_var` with the same mask) become literal restatements. It is strictly more general, so nothing C expresses is lost. Its one cost, the rank-aware precedence, is already paid for `reduce`. C is simpler to read for the single case PyTorch and ONNX define, and has rank-free bytes, but it adds a second way to name axes and cannot say `{0,2}`.

## What the Unpopped lane reported (2026-11-07, Unpopped `b3e44429`, `crates/unpopped-vocab/src/ops.rs:573-720`)

Its kernel IR has no norm op (softmax and rms_norm are composed from `reduce` and elementwise ops). Its SKU vocabulary has `NormalizationKind {RMSNorm, LayerNorm, GroupNorm, BatchNorm (train only), InstanceNorm}` and `SoftmaxKind {Softmax, LogSoftmax}`, forward and backward. Its doc says RMSNorm and LayerNorm take a **bitmask of normalized axes that must be a suffix of the input shape** (PyTorch `normalized_shape`). So its encoding is a mask, which is option B's shape, restricted to a suffix. Its `gamma`/`beta` shape per axis is **not stated in the source it read**; it will check and report. It is adding master row `I1` (normalization family, multi-axis, affine shape). Not read by me: `GroupNorm`, `BatchNorm` and `InstanceNorm` have no KISS op today and are not in this note's scope (new rows if CireSnave wants them).
