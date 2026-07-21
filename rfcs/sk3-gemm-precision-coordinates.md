# RFC: `sk3` — GEMM precision/compute coordinates in the `structure_key`

| | |
|---|---|
| **Status** | **Accepted (2026-07-21).** All field-level review folded; the §6.17 input-rounding pin (the required change) signed off by Baracuda (cosignatory + provider) and **kiss-ref (reference-evaluator) firsthand, no caveats** (§4.3); Fuel cosignatory accepted the sk3 direction/grammar. Maintainer-authorized to adopt on kiss-ref's firsthand sign-off. The concrete clause edits below are approved for application to the spec/codec (a separate implementation step); RFC pending merge to main. |
| **Date** | 2026-07-19 |
| **Affects** | KISS-Classify (§6.1, §6.6, §6.7), KISS-Contract (§6.8), KISS-Ops (§6.1-0005, §6.17) |
| **Filing** | umbrella §7.2, to the KISS-Classify / KISS-Contract / KISS-Ops editors-of-record, cc Fuel + Baracuda |
| **Source** | [`RECONCILIATION.md`](../RECONCILIATION.md) decisions **D1, D4 (`f32s`), D5**; sequencing pin from Baracuda's reply |
| **Related** | Fuel #22 (output operand in key), #9 (MX dtypes); Baracuda's `sk2` landing (KISS #60) |

> **This RFC is informative until adopted.** It proposes concrete clause edits; the exact
> token grammar below is *one option* offered to force the decision — every field spelling
> and ordering is the editors' to pin. Illustrative tokens are marked as schematics; the
> authoritative bytes come from the reference codec once the grammar is fixed, **never** from
> a hand-authored example (that is the discipline the freeze-gate exists to enforce).

---

## 1. Summary

One coordinated, byte-visible schema bump — **`sk2 → sk3`** — that grows the **`gem`
(dense-contraction) cell identity** to carry its full precision/compute coordinate set:

- **D1** — the secondary (weight) dtype, folded into the contraction field;
- **D5 (key half)** — the accumulator/compute dtype;
- **D1** — the **output** dtype (Fuel #22);
- **D4** — a **MathPrecision** coordinate that replaces the `f32s` dtype hack;
- **D4** — **variant-explicit** FP8 spellings (`e4m3` → `e4m3fn`, reserving `e4m3fnuz` / the
  inf-carrying variant);
- **D1 D-note** — the **`batch`** size-class Baracuda already emits.

Each is a byte-visible change to `gem`/FP8 tokens, so — per Baracuda's sequencing pin — they
land in **one version bump, not four.** Non-`gem` cells are unchanged except the global
version prefix (`sk2|…` → `sk3|…`).

**This RFC deliberately excludes** the separable track (D2 ULP ceiling, D3 optional Dispatch,
D6 reproducibility axis, D7 FDX, plus the additive MX codes and the `u16`/`u64` prune). None
of those is byte-visible to an existing token, so none needs the version bump; each proceeds
independently.

## 2. Motivation

**The current key collides for mixed precision, and the collision is a hard block on the
provider.** Today `structure_key` keys only operand-0's dtype (KISS-CLASSIFY-6.6-0015) and the
contraction field carries no dtype (§6.6-0010). So a mixed-input FP8 GEMM and a homogeneous one
derive **byte-identical tokens**:

```
E4M3×E5M2→F32 :  sk2|gem|e4m3|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>
E4M3×E4M3→F16 :  sk2|gem|e4m3|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>   ← identical
```

Under §6.6-0018 a provider is **forbidden to register two cells with byte-identical tokens**,
so Baracuda cannot advertise the FP8 coverage matrix it actually ships; and a consumer cannot
look a mixed-precision cell up from the token alone (the defining requirement of a *join* key,
answered YES by Fuel from the consumer side and confirmed by Baracuda from the provider side —
RECONCILIATION D1). Out-of-band disambiguation (§6.6-0018) works for a provider that holds both
`token` + `winner_entry`, but not for cross-vendor consumer-side lookup.

**The `f32s` case is the same problem wearing a different hat.** SIMT-`f32` (full binary32,
bit-stable) and TF32-`f32` (10-bit mantissa, tensor-core, warp-reduction nondeterministic) are
**numerically and determinism-distinct cells that require different kernels** — so they must
hold distinct tokens. Baracuda currently forces that distinction with a spec-forbidden `f32s`
dtype token (§6.1-0005 says compute precision must be an attribute, not a dtype). The clean fix
is the same key growth: a MathPrecision *coordinate* in the key distinguishes them, and `f32s`
retires.

## 3. Scope boundary

| In scope (byte-visible → this `sk3` bump) | Out of scope (separable, no bump) |
|---|---|
| D1 weight + output dtype in the `gem` contraction field | Add MX element codes (additive, §6.7-0007) |
| D5 accumulator/compute dtype coordinate | Prune `u16` / `u64` from the closed set |
| D4 MathPrecision coordinate (retire `f32s` token) | Retire the §6.8 ULP ceiling (KISS-Ops) |
| D4 variant-explicit `e4m3` / `e5m2` spellings | Contract `accumulation_type` field¹ |
| D1 D-note `batch` size-class | D6 reproducibility-scope axis; D7 FDX blessing |

¹ The Contract `accumulation_type` field (D5's contract half) is a contract-format change that
does not gate on the token version, but it is *tightly coupled* — the key carries the
accumulator for lookup; the contract declares it as a guarantee — so it should land alongside.

## 4. Proposed changes

### 4.1 KISS-Classify — the codec (§6.6, §6.7, §6.1)

1. **Version.** `SCHEMA_VERSION 2 → 3`; token prefix `sk3` (§6.6-0004 / §6.7-0002).
2. **Grow the contraction field** (§6.6-0010 / §6.7-0006), present only for `gem` cells. One
   concrete grammar:

   ```
   current (sk2):       c<m><n><k>/<kdiv>
   sk3 (non-batched):   c<m><n><k>/<kdiv>/<wdt>/<acc>/<out>/<mp>
   sk3 (batched):       c<m><n><k>/<kdiv>/b<class>/<wdt>/<acc>/<out>/<mp>
   ```

   - `<batch>` = `b<class>`, `<class>` ∈ size-class `{t,s,m,l}`, **present iff the cell is
     batched** — a *conditionally-present* coordinate, not an always-emitted one. A non-batched
     `gem` cell omits it entirely. This is the general optional-coordinate rule the key already
     uses — the contraction field is `gem`-only, the reduce field is `reduce`-only — and it is
     **project-agnostic**: an optional coordinate appears iff the op class carries it, read the
     same way by any consumer in any language. It is **not** eliding a field at its default (what
     DESIGN §1.6 governs — that rule is about always-present fields); a structurally-optional
     coordinate satisfies §1.6 and additivity at once. *(Flipped from an always-present `nb`
     sentinel per Fuel + Baracuda cosignatory concurrence: an always-present `/nb` perturbs every
     non-batched token for zero information gain and breaks additivity. Driver: Baracuda's
     additivity constraint — a non-batched cell stays byte-identical to its pre-`sk3` token save
     the version prefix + the precision coordinates.)*
   - `<wdt>` = the **operand-1** dtype token (the **weight**, in ML terms);
   - `<acc>` = the **accumulator / compute** dtype token;
   - `<out>` = the **output** dtype token;
   - `<mp>` = the **math-precision** code, `{st}` bit-stable (strict) / `{rm}` reduced-mantissa
     (derived from the KISS-Ops MathPrecision attribute, §6.17), replacing `f32s`. `<mp>` codes
     **never begin with `b`** — that prefix is reserved for the batch coordinate — so the geometry
     and precision groups never collide in spelling (provider constraint, Baracuda review).
3. **Retire the `f32s` token** from the closed dtype set (§6.1). The `F32Strict` behavior is
   preserved by `<mp>=st` on an `f32`-primary cell; TF32 is `f32`-primary + `<mp>=rm`. This
   brings the codec into line with §6.1-0005, which already forbids the strict-precision dtype
   token.
4. **Variant-explicit FP8** (§6.1). Replace `e4m3` with `e4m3fn` (OCP, SATFINITE, no infinities)
   and reserve `e4m3fnuz` (AMD, distinct bias, no −0) and the inf-carrying `e4m3` variant;
   `e5m2` similarly. This changes the bytes of existing FP8 tokens — hence byte-visible.
5. **Relax §6.6-0015 for `gem` only.** The "keys only operand-0's dtype" rule is superseded for
   dense-contraction cells by the contraction field's explicit dtype coordinates. **Non-`gem`
   cells keep §6.6-0015 + §6.6-0018 unchanged** (Baracuda's out-of-band approach is correct
   there).

### 4.2 KISS-Contract — the guarantee (§6.8)

Add **`accumulation_type`** to the KISS-Contract Guarantees precision block (§6.8-0001 field
list): the dtype in which the kernel accumulates. Drafted by the D5 owner and folded here so the
key `<acc>` and the contract field stay pinned in one document. It slots into the §6.8 Guarantees
insertion point left clean by **D2** (merged to main @ `239ecba` as
[#63](https://github.com/ThinkersJournal/KISS/pull/63); closed #39 + #42): D2 retired §6.8-0002's
ULP ceiling — the declared per-target accuracy tier is now the sole gate — and touched §6.7-0005
only, leaving the §6.8 Guarantees field-list intact. `accumulation_type` adds to that intact
field-list.

**Normative — `KISS-CONTRACT-§6.8-00NN`** (number editor-assigned; append-only). A contract for a
contraction/reduction-bearing kernel (matmul, reduce-sum/prod, any op with a float fold) MUST
declare `accumulation_type`: the dtype in which the kernel accumulates, drawn from the **same
closed dtype set** as the KISS-Classify `<acc>` key coordinate (§4.1.2). For a given kernel the
contract's `accumulation_type` MUST **denote the same dtype** as the key's `<acc>`, using the
**same closed dtype-token spelling** — one dtype, two surfaces (key = identity/lookup, byte-visible
in the token; contract = declared guarantee). An implementation MUST NOT declare an
`accumulation_type` outside the closed set, and MUST NOT let `accumulation_type` and `<acc>`
disagree. *Test:* `test_contract_accumulation_type_matches_key_acc`.

> **Pin wording refined** from "same bytes" to "same dtype / same token spelling" (Baracuda's
> original consistency pin, sharpened by the D5 owner): the two surfaces serialize *differently* —
> the key token vs the contract field — so the invariant is **one dtype consistently spelled from
> the closed set**, not identical wire bytes.

**Informative (provider-example, not policy).** A real provider's accumulator lattice — Baracuda's
`PrecisionGuarantee.accumulator` — maps operand→accumulate as `int8/int4/bin → s32`;
`fp8/f16/bf16/f32 → f32`; `f64 → f64`. Shown for concreteness; KISS does **not** mandate it — any
backend (mobile GPU, CPU reference, Vulkane) declares its own, and the normative requirement is
only that the kernel *declares* one from the closed set. (Same treatment as the §6.8
advisory-floor ULP table — keeps §6.8 project-agnostic per the maintainer's guardrail.)

**Forward-reference (deferred, non-blocking).** An opt-in **exact-reduction** sub-class — pinning
reduction *order* so the accumulate is bit-reproducible, distinct from merely declaring the
accumulate dtype — is reserved for a future revision, sequenced behind a real consumer. Not
required for the sk3 bump; imposes nothing on v1. (This is the contract-side hook for the D6
reproducibility-scope axis, which stays out of the key per §7.)

### 4.3 KISS-Ops — the §6.17 input-rounding pin (required change; both signed off)

The `<mp>` key coordinate (§4.1.2) is *derived from* the KISS-Ops MathPrecision attribute (§6.17).
For a spec-derived reference to reproduce a reduced-mantissa result, §6.17 must pin the exact
input-rounding each MathPrecision value implies — otherwise `rm` names a *class* (TF32 10-bit /
bf16-mad 7-bit / mobile fp16) and no reference can reproduce bits from it. Per the maintainer's
ruling, **sk3 clears iff this pin is folded in**; the text below is final — signed off by both
Baracuda (provider) and kiss-ref (reference-evaluator).

**§6.17 MathPrecision — input-rounding pin (sk3 required change).** For each MathPrecision value,
§6.17 states the exact input-rounding applied to each operand before compute, as
`(retained_mantissa_bits, rounding_mode)` — precise enough for a spec-derived reference to
(a) derive the accuracy bound (u = 2^−(bits+1)) and (b) reproduce the rounded operand bit-for-bit.

- **`st` (full-precision):** no input rounding; operand at its storage-dtype mantissa.
- **`rm` (reduced-mantissa):** operand mantissa rounded to a pinned width before compute via the
  named mode; each `rm` value names `(retained_mantissa_bits, rounding_mode)`. **TF32** (f32-primary,
  `cuda:sm80+`): 10 mantissa bits, 8-bit exponent unchanged, RNE, u = 2⁻¹¹. *Rounding is a true
  round (RNE), not truncation, and MUST carry into the exponent (`1.11…1` → `10.0…0`, exp +1); a
  reference that truncates is non-conformant.*
- **Key ↔ semantics:** the `<mp>` KEY code resolves to a MathPrecision value per
  `(primary_dtype, target_capability)`; `rm` on f32-primary `cuda:sm80+` = TF32. A second
  reduced-mantissa mode for the same `(dtype, arch)` ⇒ an additive `<mp>` sub-code (`rm10` / `rm7`);
  the semantics layer names the mode regardless.
- **Reproducibility class — keyed on accumulation-schedule determinism, ORTHOGONAL to `<mp>`.**
  Float accumulation is non-associative, so input-rounding + `<acc>` width do **not** fix a
  contraction's bits — the reduction order/schedule does.
  1. **Bit-reproducible (golden exists)** only if BOTH input-rounding is pinned per §6.17 AND the
     accumulation schedule (order + per-step `<acc>` rounding) is deterministic AND specified to the
     reference.
  2. The trigger is **nondeterministic/unpinned accumulation**, correlating-with-but-not-`<mp>`: a
     full-precision `st` GEMM with warp-nondeterministic accumulation is a **tolerance** cell too.
  3. Dense contraction (GEMM) is **tolerance-class by default**; `<mp>` only shifts the tolerance
     *magnitude*, never flips tolerance ↔ golden. A GEMM golden exists only with an
     additionally-pinned, reference-specified schedule.
  4. A tolerance cell's declared tolerance MUST bound `(input-rounding ⊕ reduction-order)` against
     the wide-precision truth — the **`KISS-CONFORM-6.5-0007`** oracle evaluation (naming *which*
     "truth" the tolerance bounds against, per the bare-§ disambiguation discipline — kiss-ref
     editorial note).

This resolves the §7 "is `<mp>` sufficient?" sub-question in full: `<mp>` suffices for *key
discrimination* (which kernel), while *numeric reproducibility* is the orthogonal
accumulation-schedule axis above (the D6 reproducibility-scope facet, carried in the contract, not
the key). (§6.1-0005 already forbids the strict-precision dtype token, so the codec simply stops
violating it via `f32s` — no further Ops edit needed there.)

## 5. Worked example (schematic — bytes are illustrative, not authoritative)

The two collisions from §2, disambiguated under `sk3`:

```
E4M3×E5M2→F32, f32 acc, bit-stable, non-batched (no batch coordinate):
  sk3|gem|e4m3fn|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>/e5m2/f32/f32/st
E4M3×E4M3→F16, f32 acc, non-batched:
  sk3|gem|e4m3fn|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>/e4m3fn/f32/f16/st   ← now distinct

Same E4M3×E5M2→F32 but batched (class m) — batch coordinate present:
  sk3|gem|e4m3fn|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>/bm/e5m2/f32/f32/st  ← distinct from its non-batched form; note bm (batch) vs st (mp) — no collision

SIMT-f32 GEMM (full binary32):
  sk3|gem|f32|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>/f32/f32/f32/st
TF32 GEMM (tensor-core, reduced mantissa):
  sk3|gem|f32|cuda:sm90|ix32|grid|r2|…|-|c<mnk>/<kdiv>/f32/f32/f32/rm         ← distinct by mp, no f32s
```

## 6. Migration and sequencing

1. **Do the `sk2` `relu_add` freeze-gate byte-match *first*.** `relu_add` is a `bin` cell
   untouched by this RFC; landing the two-implementation byte-match at `sk2` before the bump
   means the freeze-gate is not chasing a moving version prefix. `sk3` follows.
2. **A version bump re-prefixes every token** (`sk2|…` → `sk3|…`); only `gem`/FP8 cells change
   *structurally*. Regenerate affected goldens / PROVENANCE from the landed codec (coordinate
   with the D8 `sk2` regen, KISS #60) — never by hand.
3. **Hold all in-scope changes together.** Per Baracuda, D1 + D4-`f32s` + D5 land as one bump;
   partial landing (e.g. retiring `f32s` before `<mp>` exists) reintroduces a §6.6-0018
   collision and a silent numeric/determinism ambiguity.

## 7. Open sub-questions for the editors

- **Field ordering and spelling** of the extended contraction tuple (§4.1.2) — the grammar
  above is one option.
- **Batch encoding — RESOLVED (RFC author + Fuel + Baracuda cosignatory concurrence).**
  `<batch>` is a *conditionally-present* coordinate spelled `b<class>`, present iff the cell is
  batched; a non-batched cell omits it (§4.1.2, the general optional-coordinate rule). The
  always-present `nb` sentinel is rejected as additivity-breaking. Absence canonically means
  **non-batched** — a size-1 *batched* cell still emits its class (`bt`), so absence is never
  ambiguous with a unit batch. Batch is an iteration-structure fact, so it sits with the geometry
  group (right after `<kdiv>`), not inside the precision group (Baracuda review).
- **`<mp>`/batch spelling collision — RESOLVED and pinned in the body (RFC author + Baracuda
  provider review both concur).** Bit-stable `<mp>` is spelled **`st`** (not `bs`), giving
  `<mp> ∈ {st, rm}` — collision-free against the batch classes `{bt,bs,bm,bl}` and echoing the
  **F32Strict** semantics this coordinate retires. The general rule recorded in §4.1.2 —
  **`<mp>` codes never begin with `b`**, that prefix reserved for the batch coordinate — is a
  clean codec-author constraint, not a Baracuda carve-out. Now pinned throughout §4.1.2/§5.
- **Is `<mp>` sufficient? — RESOLVED (Baracuda provider): yes, for every kernel-distinct cell
  today.** `<mp>` is a **compute/mantissa-precision** axis and must stay that — do **not** overload
  it to *mean* determinism. No shipping cell has mantissa-precision and determinism-scope diverge
  (TF32 couples them: reduced-mantissa ⟺ warp-nondeterministic), so one `<mp>` axis separates every
  cell that needs a different kernel. Reproducibility-scope is a **distinct axis (RECONCILIATION
  D6)** that lives in the **contract** (`bit_stable_on_same_hardware`) today, not the key. If a
  future backend ever needs an *independent* determinism-scope key coordinate, it is added then as
  a separate optional (additive) coordinate — not pre-baked now. Keeps the key minimal and
  forward-safe.
- **`gem`-only vs general.** This RFC scopes the growth to dense-contraction cells. If a
  non-`gem` reduced-mantissa distinction is later needed, it is a separate coordinate — out of
  scope here.

## 8. Non-goals

This RFC does **not** touch the separable track (§3) and does **not** alter non-`gem` cell
identity beyond the version prefix. It is exactly the byte-visible GEMM-precision coordinate
set, bundled once.
