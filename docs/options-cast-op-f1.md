# Options note: should KISS have a `cast` op, and what does it pin? (union-table finding F1, board 304)

**Informative. No normative text. For CireSnave's decision; no clause is proposed here, only the shape of one.**
Read at the board-309 stack tip (conformance 0.6.27) on `origin/main` `4257822`. Starting point for the semantics: the Unpopped
U1 worker's proposed Cast-to-MX rule, relayed by the PM on 2026-10-28. **That proposal is a message, not a document: its PR does
not exist yet, I have not read it, and its hardware-NaN claims are, in its own words, unverified.** Everything marked **[U1]**
below is from that relay only.

## The question

The union-coverage table (`docs/union-coverage-unpopped.md`, finding F1, row 33) found that `cast` is **named** by the shape
oracle (Ops 6.20 preamble) and by Conform's signature rule ("a `Cast`'s target dtype rides inside that node's OpAttrs blob",
`spec/conform.md:2060`), but is **not in the op catalog** (§2.7), the registry (§6.1) or the OpAttrs schema table (§6.19.3).
Unpopped's IR has `Cast(x, {to, round, sat})` and `Bitcast`. So a converting kernel cannot be described, and mixed-dtype
contraction, dequantisation and a narrowing epilogue all depend on it. **Does KISS add a `cast` op, and what does it fix?**

## What conversion rules are pinned today (all attached to the *destination* dtype)

| Destination | What is pinned | Clause |
|---|---|---|
| `bf16` | any op producing a `bf16` result rounds to nearest, ties to even | KISS-OPS-6.16-0003 |
| `f8e4m3fn` | conversion into it saturates to the max finite (448), round-half-to-even; one NaN encoding, no infinity | 6.16-0004 |
| `f8e5m2` | conversion into it saturates to the max finite (57344), round-half-to-even | 6.16-0005 |
| `f4e2m1`, `f6e2m3`, `f6e3m2` | "rounds half to even and saturates to max-finite" (OCP MX v1.0 §5.3.2 / 5.3.3); maxima 6, 7.5, 28; no infinity, no NaN | 6.16 dtype table (normative home Classify 6.1-0014/-0015) |
| `f16`, `f32`, `f64` | the IEEE 754-2019 encodings; arithmetic follows IEEE | 6.16-0002, 6.2-0001 |
| integers | wrapping `add`/`sub`/`mul` modulo 2^width; `bool` normalises to 0/1 | 6.2-0002, 6.2-0006 |

## What is not pinned

1. **There is no op.** A conversion happens only as "an op producing a result of dtype T", so a conversion with nothing else in
   it, requested on purpose, has no name and no OpAttrs layout.
2. **NaN into a format with no NaN** (the three MX types): nothing says what NaN becomes. **[U1]** proposes NaN to +0.
3. **Overflow into an IEEE-style format** (`f16`, `bf16`, `f32` from wider): saturate or go to infinity? The fp8 and MX rows
   saturate; `bf16` says only "round to nearest, ties to even"; `f16` is silent.
4. **Float to integer** (and integer to float, and integer narrowing): no rule at all. Out-of-range float to integer is the
   classic divergence between targets.
5. **A rounding mode other than nearest-even.** Only RNE exists anywhere in Ops (6.7 rounding atoms are `floor`/`ceil`/
   `trunc`/`round_even` on values, not conversion modes).
6. **Bitcast** (reinterpret the bits as another dtype of the same width): no op. `select`, `copysign`, `neg`, `abs` are raw-bit
   moves but cannot change the dtype.

## The Unpopped U1 proposal, as relayed **[U1]**

Cast to an MX element type: **saturating always** (`ClampFinite`); `Wrap` is a **typed reject**; **NaN becomes +0**; round to
nearest-even plus **four other rounding modes** computed from the exact value (the relay does not name them); MX maxima e2m1 = 6,
e2m3 = 7.5, e3m2 = 28 (these three agree with the KISS dtype table above). Hardware NaN behaviour is **unverified**, and the
relay says emitters must decline rather than approximate where it matters.

What agrees with KISS already: the saturation, RNE and maxima. What is new: NaN to +0, the typed reject of `Wrap`, and the
extra rounding modes.

## Options

**A. A `cast` op with three explicit attributes, defaults equal to today's pinned rules.** OpAttrs
`{ target_dtype, rounding, saturation }` with `rounding` default RNE and `saturation` default what the destination already pins
(saturate for fp8/MX, per 6.16), and the unpinned cases (rows 2 to 4 above) each decided in the clause. It is a primitive-floor
atom if it cannot be decomposed to existing atoms without double rounding; a conversion into a narrow float *can* be written
with bit ops, but not readably, so I would add it as an atom (a floor change, Ops 6.3, so a version event). Pairs with a
separate `bitcast`.

**B. Keep conversion as a property of the destination dtype, and name the op only in the shape oracle.** Add the missing rules
(NaN into no-NaN formats, overflow, float to integer) to 6.16 per destination, and delete `cast` from the shape-oracle and
Conform prose or define it there as a non-op. Smallest change; leaves "a deliberate narrowing" unnameable, so a mixed-dtype
contraction still has nowhere to say where it rounds.

**C. A `cast` op that takes only `target_dtype`**, with rounding and saturation always the destination's pinned rule. Simplest
OpAttrs; cannot express a round-toward-zero conversion that Unpopped already builds (**[U1]**).

## Recommendation

**A**, because the Unpopped IR already needs the three attributes, the missing-rule list above is a defect whichever option is
chosen (rows 2 to 4 should be pinned even under B), and the ABI cost of adding fields later to a per-op layout is higher than
adding them now (App. D.11: field order is ABI). Decide the four open values first, because they set the defaults:

1. NaN into a format with no NaN: **+0** (the U1 proposal), a typed decline, or a canonical value of the format.
2. Overflow into an IEEE-style float narrower than the source: saturate (consistent with fp8/MX) or infinity (consistent with
   IEEE).
3. Float to integer when out of range: saturate, wrap, or typed decline (I would pick saturate, with NaN to 0).
4. Which rounding modes beyond RNE, and whether they are an enum in the `scatter_combine` style (frozen ordinals, `0` reserved).

## What I have not done

I have not read the U1 worker's code or PR (none exists yet), checked any hardware's NaN behaviour, or counted how many existing
kernels (Baracuda's `.cu` files in `conformance/cuda/`, Fuel's) depend on a conversion rule. The list of unpinned cases is from
reading `spec/ops.md` only, so a rule pinned in another sub-standard would not appear here.

## What freezes, and what to decide

Nothing freezes on this note. Decisions: **A, B or C**; the four values above; and whether `bitcast` rides with it. Adding an
atom is a floor change, so it needs a KISS-Ops version event whichever of A or C is chosen.
