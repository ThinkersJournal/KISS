# Options note: Slang precision and NaN-mode (union batch 2, Slang rows 50 and 52)

**Informative. No normative text. Docs only: nothing is minted and no byte moves.** Moved here from union batch 1 by pm-kernels (2026-11-26) because a guessed key coordinate is the byte-risk CireSnave's rule on stored bytes guards (board 501 "A", EXPECTATIONS 6.35). Any option below that moves bytes goes to CireSnave as a LISTED batch (exact tokens, what changes per consumer, a veto window of at least 24 hours) before a shipping PR opens.

## What is being asked

Unpopped union rows Slang 50 (per-function precision mode) and Slang 52 (NaN/Inf mode): express Slang's floating-point modes so a second implementation can state and match them. Unpopped's own reply (unpopped@5f822176, `ir.rs` lines cited by Unpopped, not read by this lane):
- Unpopped's numerics are **per-operation**: `Numerics { round, ftz, approx }` carried by `ScalarExpr::Qualified` over `Add Sub Mul Div Fma Sqrt Rsqrt Recip Exp2 Log2`, `f32` only; `round` has five values (`NearestEven`, `TowardZero`, `TowardNegInf`, `TowardPosInf`, `NearestAway`, the last a Cast only); `ftz` and `approx` are booleans.
- Unpopped has **no NaN-mode field**: NaN behaviour is chosen by the operation (`Max`/`Min` propagate NaN, `FmaxIeee`/`FminIeee` suppress it; `ReduceOp::Max`/`Min` propagate; `ArgMax`/`ArgMin` follow KISS-OPS-6.11-0007). Slang rows 50 and 52 are scored Partial and No for that reason.

## What Slang actually has (primary source, read 2026-11-26)

shader-slang/slang at `0e393b37`:
- `docs/command-line-slangc-reference.md` lines 513 to 519 and 1260 to 1268: `-fp-mode` / `-floating-point-mode` takes `precise` ("Disable optimization that could change the output of floating-point computations, including around infinities, NaNs, denormalized values, and negative zero. Prefer the most precise versions of special functions supported by the target."), `fast` ("Allow optimizations that may change results of floating-point computations. Prefer the fastest version of special functions supported by the target.") or `default`.
- Same file, lines 521 to 543 and 1269 to 1276: `-denorm-mode-fp16`, `-denorm-mode-fp32`, `-denorm-mode-fp64` take `any` (implementation defined, the default), `preserve` or `ftz`; the 16- and 64-bit flags are documented for SPIR-V, the 32-bit one for SPIR-V and DXIL.
- `include/slang.h` lines 833 to 848: `SlangFloatingPointMode` = `DEFAULT 0`, `FAST 1`, `PRECISE 2`; `SlangFpDenormalMode` = `ANY 0`, `PRESERVE 1`, `FTZ 2`; lines 4540 to 4547: the mode is a **target** option (`floatingPointMode` in the target description).
- **No rounding-mode control and no separate NaN switch were found** in these files: NaN, infinity, denormal and signed-zero behaviour appear only inside the text of `precise`. (Absence is from reading the reference page and the two enums, not from a full source search.) A `precise` modifier on individual declarations is reported by Unpopped's evidence docs and was **not** read here.

So the scopes differ: Unpopped's is per-operation and has five rounding values; Slang's is per-compilation-target, has two modes plus a three-valued denormal mode per width, and carries NaN/Inf inside `precise`.

## What the key says today

Compute precision is a KISS-Ops fidelity attribute, not a dtype (KISS-CLASSIFY-6.1-0005). The key carries it only as `<mp>`: `MathFidelity` is `st` (bit-stable) or `rm` (reduced-mantissa permitted) (`conformance/src/structure_key.rs`, `code_enum!(MathFidelity ...)`), spelled in the contraction field (6.7-0006) and, when it deviates, in the non-contraction `(acc+mp)` field (6.7-0013). Denormal handling, rounding mode and NaN policy are in no key coordinate.

## Options

- **A. No key change; carry the modes as a contract guarantee (recommended).** A kernel provider states its compiled modes (precise or fast; denormal preserve/ftz; as the facts it actually has) in its KISS-Contract Guarantees (6.17 family), and the request side maps Slang `precise` to `<mp>` = `st` and `fast` to `rm` only where the kernel already means that. No bytes move. *For:* the modes are properties of how a kernel was built, which is what a Contract guarantee states; no sk event. *Against:* a caller cannot select by denormal mode through the key; two kernels differing only in ftz share a key and must be told apart by their contract.
- **B. Widen `<mp>` with the denormal mode** (for example `st`, `rm` and denormal variants). Byte-altering: a new `<mp>` spelling set, an `sk` question. *For:* ftz versus preserve is admissibility, not only a guarantee. *Against:* it keys a compilation option of one language into every ordinary cell.
- **C. A new cross-op key coordinate `{fp-mode, denorm}`.** Byte-altering and the largest. *Against:* Unpopped's own model is per-op and has no equivalent; Slang's is per-target; the coordinate would be the union of two scopes that do not match.
- **D. Per-operation `{round, ftz, approx}` as OpAttrs (Unpopped's shape), not in the key.** An Ops matter (siblings: R1.33, R1.34), bytes only in the OpAttrs blob of the ops that take it, and only if a consumer asks. *For:* matches Unpopped's `Qualified`. *Against:* does not describe Slang's per-compilation switch.
- **NaN-mode specifically:** neither Slang nor Unpopped has a separate NaN switch; it follows from the operation (Unpopped) or from `precise` (Slang). **Recommendation: do not mint a NaN-mode coordinate.** KISS-Ops already pins `Max`/`Min` NaN behaviour per op (6.11 family).

## What each choice means per consumer

| Consumer | A | B | C | D |
|---|---|---|---|---|
| unpopped (`Qualified`, per-op) | none | new `<mp>` values to map | new coordinate that its model cannot fill per-op | maps one to one (it already has the fields) |
| fishstringer / slang-emit (per-compilation) | states modes in its contract | must spell the denormal mode in its key | must spell fp-mode and denorm | cannot express a per-target switch |
| other emitters | read the contract | exhaustive matches on `<mp>` break | key bytes change | OpAttrs decode extends |
(Consumer facts are from Unpopped's report and the Slang files above; the emitters' own models were not read.)

## RULED by the PM 2026-11-26 (relayed by pm-kernels; no stored byte moves, so the board 501 listing is not triggered)

- **Q1 = A now:** the modes are a contract guarantee and `<mp>` is unchanged (Slang `precise` maps to `st` and `fast` to `rm` only where the kernel already means that). **D** (per-op `{round, ftz, approx}` as OpAttrs, siblings R1.33 and R1.34) **later, only if a consumer asks.** **B and C are not taken.**
- **Q2 = no NaN-mode coordinate** (neither Slang nor Unpopped has a separate switch; KISS-Ops 6.11 pins `Max`/`Min` per op).
- **Open fact, no rush (only B or C would need it):** whether any Slang target honours `-fp-mode` or `-denorm-mode-*` per entry point rather than per target.

## Questions that were put to CireSnave's side (answered above)

1. A (recommended), B, C or D, or A now and D later if a consumer asks. Only B and C move stored bytes; A and D move none in the key.
2. Whether a NaN-mode coordinate is wanted at all (recommendation: no).

Open fact to settle first, if B or C is chosen: whether any Slang target honours `-fp-mode` or `-denorm-mode-*` per entry point rather than per target (not read here).
