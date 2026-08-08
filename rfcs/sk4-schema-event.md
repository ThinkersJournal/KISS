# RFC: `sk4` — one coordinated `structure_key` schema event (dtype spelling · MX · non-contraction precision)

| | |
|---|---|
| **Status** | **Accepted — maintainer-ratified (Eric, 2026-08-08).** Ratified inputs: the **#2 canonical dtype spelling scheme** is maintainer-ratified (Eric, 2026-08-06) and four-way-converged (Fuel/kiss-ref/Baracuda/Vulkane inputs folded, Unpopped cosigning); a **single coordinated schema event** is maintainer-authorized. **Six parties cosign. Four independently derive `structure_key` and byte-match** — KISS, Fuel, kiss-ref, and Unpopped's `unpopped-vocab`; **Baracuda's leg is the physical emit corpus** (its CUDA `.cu` + emitted contract, keyed by the `sk4` key), because `baracuda-kernels-types` re-exports `unpopped-vocab`'s derivation (`pub use unpopped_vocab::*`, no own `structure_key`) — a Baracuda↔Unpopped token match would be code-identity, not convergence (Baracuda's own catch, folded here). **Vulkane cosigns the §3.1/§3.2 vocabulary and clause D and derives no `structure_key`** (a vocabulary cosigner). All six cosigns are recorded and verified (§7); **maintainer-ratified (Eric, 2026-08-08)** — regeneration is unblocked: parties regenerate and byte-match per §6. |
| **Date** | 2026-08-06 |
| **Affects** | KISS-Classify (§6.1, §6.5, §6.6, §6.7), KISS-Ops (§6.17, §6.19 reduce OpAttrs), KISS-Contract (§6.8) |
| **Filing** | umbrella §7.2, to the KISS-Classify / KISS-Ops / KISS-Contract editors-of-record |
| **Source** | Fuel↔KISS vocabulary-alignment program (Eric-initiated, KISS = RFC hub, single cross-project standard, no adapters). Three-sided collision evidence (below). |
| **Related** | `sk3-gemm-precision-coordinates.md` (the immediately-prior bump and the format this follows); `accumulator-tolerance-cells-2026-07-24.md` (§6.17, the machinery the accumulator coordinate rides); KISS-CLASSIFY-6.7-0012 (the parked accumulator forward-requirement this realizes); Fuel #9 (MX dtypes, deferred from sk3) |
| **Cosigners** | KISS (hub/steward), Fuel, kiss-ref, Baracuda, Unpopped, Vulkane |

> **Informative until adopted.** This RFC pins the *grammar, coordinate set, and migration
> rules*; every illustrative token below is a **schematic**. The authoritative bytes come from
> the reference codec once the grammar is fixed — **never** from a hand-authored example. That
> is the freeze-gate discipline, and it is exactly why the three-way (now four-way token) byte-match
> re-derivation is the acceptance gate, not this document's examples.

---

## 1. Summary

One coordinated, byte-visible schema bump — **`sk3 → sk4`** — batching every change that alters
an existing `structure_key` token, so the ecosystem pays **one** four-way token re-derivation +
byte-match (plus Baracuda's emit-corpus regen), not three. The committed scope:

- **A. Canonical dtype spelling (ratified).** A generative grammar `{class}{bits}[{variant}]`
  replacing the internally-inconsistent, partly vendor-leaked enum. De-vendored **`i`-prefix
  integers** (`s8/s16/s4` → `i8/i16/i4`); **width-prefixed floats** keeping the `fn`/`fnuz`
  variant suffix (`e4m3fn` → `f8e4m3fn`); **`bf` as a grammar class** so `bf16` = `{class bf}{bits 16}`
  keeps the plain `bf16` wire token with **no alias table** (§3.1.3, Option C, maintainer-ruled
  2026-08-06); **complex named by total width** (the `c32`/`c64` *meaning-flip*, §3.1.4); a single
  normative anchor at §6.1.
- **B. MX (OCP Microscaling) dtype additions.** `f6e2m3`/`f6e3m2` (MXFP6), the `f4` MXFP4
  (`e2m1`), and the `f8e8m0` / `f8e6m2` shared block-**scales** (both unsigned; `f8e6m2`
  maintainer-ruled a scale, not an element, §3.2) — introduced via the
  **native-vs-quantized-encoding synthesis** (§3.2): native narrow integers stay dtypes; MX is a
  physical **encoding + scale operand** under a logical dtype.
- **C. Non-contraction precision — `(acc + mp)` identity coordinate (maintainer-ruled KEEP,
  2026-08-06).** The accumulator DTYPE and mp MODE (strict-SIMT vs TF32) are **identity** coordinates:
  an f32-vs-f16-accumulate reduction, or strict-vs-TF32, is a different kernel with different bits —
  the evidenced §2 collision. Realizes the parked KISS-CLASSIFY-6.7-0012 accumulator forward-
  requirement and extends `<mp>` to the non-contraction key (§3.3). Absent ⇒ compute dtype (the
  §6.17-0005 diagonal), so ops with no accumulator don't carry it — it is not "contraction-shaped."
  (Vulkane's transcendental-ULP / reduction-order / denorm terms are the *guarantee* surface —
  deferred to the precision workstream; schedule-level reduction order, split-K/tree/sequential, is a
  generator/identity choice, only hardware wave-order is guarantee.)
- **D. Version-prefix inseparability (codec).** A foundational §6.7 codec clause: the schema
  version prefix is normative and **inseparable** — a dtype sub-token has no meaning detached from
  its version; implementations MUST NOT persist, index, or compare dtype sub-tokens independently
  of it. This is what makes A's renames migratable *and* the `c64` meaning-flip safe (§4).

**Deliberately deferred** (not in `sk4`): **per-operand dtype** — the indexed-region-synthesis
coordinate, a *describability* gap whose addition **reverses** KISS-CLASSIFY-6.6-0015's *deliberate*
caller-precondition, gated on a separate scope decision (§8), not urgent; and the reduction-order /
denorm **accuracy** axes (→ guarantee-surface / precision workstream).

The bump is **forced by A** (the rename is byte-visible to existing tokens); B (new MX codes) is
additive and folded in for one regen. **`sk4` = spelling + MX + `(acc + mp)`.**

## 2. Motivation

**Three independent collisions, each the same failure shape — the key projection discards a
distinction that changes the kernel — and none hypothetical:**

1. **Accumulator (numeric).** A mixed-precision non-contraction reduction (`f16` data, `f32`
   accumulate) and a same-shape `f16`-accumulate reduction derive **byte-identical** `structure_key`
   tokens: the non-contraction key carries no accumulator coordinate (KISS-CLASSIFY-6.7-0012 states
   this as a parked *forward requirement*; `<acc>` exists only in the `gem` contraction field). A
   build-on-miss cache keyed on `structure_key` cannot tell them apart, so the first build's
   accumulator silently serves every later requester on that key, with no way to request the other.
   Evidence: kiss-ref's 128-vs-192 divergence (reference side); Fuel's `BitStablePreferenceFilter`
   routing decode off-GPU (consumer side); Baracuda's emitter shipping `accumulation_type: f64` for
   an sk3-un-discriminable non-contraction `f32` reduction (provider side, Baracuda repo — `baracuda-cuda-emit/tests/contract.rs:171`).
2. **Math precision (numeric).** `<mp>` exists exactly once in the key — inside `ContractionKey`.
   For a non-contraction op there is no math-precision coordinate, and `canonical_dtype` folds the
   strict-vs-TF32 axis unconditionally, so a strict-SIMT `f32` reduction and a TF32 `f32` reduction
   **collide**. Same shape as (1), second axis. (KISS already models this correctly for `gem` via
   `<mp>`, §6.7-0006 — this extends the coordinate to the non-contraction key.)
3. **Spelling (de-vendoring + hazards).** The §6.1 vocabulary is internally inconsistent —
   signed 8/16 are `s8`/`s16` but signed 32/64 are `i32`/`i64` — and the `s`-prefix is a **vendor
   lineage leak** (NVIDIA PTX ISA `.s8`/`.s4`/`.s32`), while `i` is neutral. A vendor-neutral
   standard cannot spell half its signed integers in one vendor's ISA notation. Two further
   hazards ride along: bare FP8 tokens that don't pin `fn`/`fnuz` (identical bits, different
   NaN/Inf/saturation), and complex named by **component** width (`c32` = a pair of `f32` = 64 bits
   total = NumPy's `complex64`) — a token that reads unambiguous and is wrong by 2×.

**Why one event.** A, C are byte-visible to existing tokens, so each *forces* a version bump; B is
additive. Every structure_key schema bump costs a full re-derivation + byte-match reverification —
the four token derivations (KISS/Fuel/kiss-ref/Unpopped) plus Baracuda's emit-corpus regen. Doing
A/B/C separately is three such regens. **The publish-boundary window is open now** (Unpopped's `unpopped-vocab` 0.1.0 published
with ~no dependents; the rename is nearly free today and permanent after the ecosystem migrates),
which is why the *scheme* is ratified now even though realization batches here.

## 3. The changes

### 3.1 Canonical dtype spelling (A)

A **generative grammar**, not a fixed enum, so future dtype additions have a **determined** spelling
rather than a negotiated one (Vulkane): a token is `{class}{bits}[{variant}]`. (The grammar delivers
spelling *determinism*, not non-breaking-ness — non-breaking additions are the `#[non_exhaustive]` /
`Other(n)` axis, §10, not the grammar.)

**Grammar constructs; the §6.1 closed set is normative (Vulkane).** The grammar can spell the same
type more than one way — under Option C `f16` and `f16e5m10` are both grammar-well-formed and denote
IEEE half — so the **§6.1 closed set is the authority** on which spelling is canonical: a token the
grammar could generate but that is **not in the §6.1 set at its declared schema version MUST be
rejected**, exactly as clause D (§3.4) rejects a spelling invalid at its version. Without this, the
token-deriving implementations could each pick a different grammar-valid canonical spelling and fail
the byte-match *after* the expensive re-derivation. The grammar is the construction rule; the closed set is the law.

- **3.1.1 Integers — uniform `i`/`u` prefix.** `i8 i16 i32 i64`, `u8 u16 u32 u64`, and `i4`/`u4`
  for the sub-byte integers. KISS renames `s8`/`s16`/`s4` → `i8`/`i16`/`i4`; `b1` for the 1-bit
  integer (was `bin`/`Bin` in `unpopped-vocab`). `bool` stays **distinct** from `i1`/`b1` — it is a
  logical type, not a width.
- **3.1.2 Floats — width-prefixed.** `f16 bf16 f32 f64`; FP8 as `f8e4m3fn`, `f8e5m2`, … — the `f8`
  width prefix **added** to KISS's existing `fn`/`fnuz` spelling. Rationale (kiss-ref): the width
  prefix self-describes and extends cleanly to MX, where a bare `e2m3` is width-ambiguous.
- **3.1.3 `f16`/`bf16` — `bf` is a grammar class (maintainer-ruled Option C, 2026-08-06).** `bf16` =
  `{class bf}{bits 16}` and `f16` = `{class f}{bits 16}`, both grammar-generated with the IEEE-standard
  layout implied by class+width (`f16` ⇒ e5m10, `bf16` ⇒ e8m7); where a width has no single standard
  layout (FP8/MX), the layout is spelled explicitly (`f8e4m3fn`, `f6e2m3`). This keeps the conventional
  `bf16` on the wire with **no alias table** — removing the normalize-before-hash mismatch hazard a
  `f16`↔`f16e5m10` alias would introduce — and avoids the category error of treating `bf` as a
  special-value *variant* of `f16` (it is a distinct layout **class**). Decodability holds:
  `b1`/`bf16`/`bool` disambiguate at char 2 (Sardinas–Patterson, the KISS-Classify decodability lint,
  gates the class set). **Conditional on the six-way cosign confirming every project can emit/consume
  `bf16` under the class model** (all currently tokenize `bf16`, so this is expected unanimous).
- **3.1.4 Complex named by TOTAL width — the meaning-flip.** Canonical complex spelling matches
  the ecosystem (NumPy/PyTorch/C name by total width): a pair of `f32` is `complex64`, a pair of
  `f64` is `complex128`. KISS's current `c32`/`c64` are **component**-width (`c32` = pair of `f32`
  = 64 bits total). **This is the riskiest item**: the new `c64` means what the old `c64` did
  **not** (old `c64` = pair-of-`f64`; new `c64` = pair-of-`f32`). It is therefore *not* a
  mechanically-safe rename and MUST ride this version event with an explicit migration note and a
  token-collision/decodability check — never a silent rename. It is safe *because of* the version
  prefix (§4).
- **3.1.5 FP8 variant suffix — mandatory where a layout admits multiple variants.** A variant suffix
  is mandatory on an FP8 layout that has more than one variant: a bare `e4m3` vs `e4m3fn` vs
  `e4m3fnuz` is identical bits, different NaN/Inf/saturation — a silent correctness hazard (Vulkane).
  It is **not** universal: `f8e5m2` carries **no** suffix because only `fnuz` deviates from the
  IEEE-conventional E5M2 (which is why §4's byte-match target is `f8e4m3fn`/`f8e5m2` — one suffixed,
  one not). The **§6.1 closed set is authoritative** on which spellings exist (§3.1.6), so a party
  implementing "mandatory suffix" literally does not emit a spurious `f8e5m2fn`-style token.
- **3.1.6 Single normative anchor.** §6.1 is the single normative source of truth; the reference
  crates (`kiss-classify-vocab`, `unpopped-vocab`) **reproduce** it, they do not compete. This is
  the "share the inert vocabulary, keep the derivations independent" ruling: sharing spelling tables
  costs nothing and kills drift; the `structure_key` *derivation* stays independently implemented,
  which is what the four-way token byte-match proves. The Sardinas–Patterson decodability lint (the queued
  KISS-Classify task) gates the resulting vocabulary.

  *Where decodability actually bites (Unpopped's codec question).* The top-level `structure_key` token
  is **pipe-delimited** (`sk4|…|f32|…`), so a reader splits on `|` before it ever inspects a dtype
  token — top-level dtype fields do **not** require prefix-freeness, and the lint is belt-and-braces
  there. Decodability becomes **load-bearing** only where dtype sub-tokens are **concatenated without
  a delimiter** inside a field. Codec-owner ruling (Unpopped): both new fields are **internally
  delimited on `/`**, not concatenated-and-prefix-free — the **non-contraction precision field**
  `(acc + mp)` is spelled gem-symmetrically (like the contraction field's `/`-separated
  `<wdt>/<acc>/<out>/<mp>`), and the **MX element+scale** pairing likewise. Rationale: a delimiter is a
  one-time decision, whereas prefix-freeness is a *standing* obligation every future dtype addition
  would have to re-prove in that context (silent-misdecode failure mode); and gem-symmetry means one
  parsing rule, not two, for what is literally the non-contraction analogue of gem's precision pair.
  With both delimited, the Sardinas–Patterson lint is **pure belt-and-braces everywhere** rather than
  load-bearing. The §6.7 codec clause (post-cosign) pins the `/`-delimited spelling.

### 3.2 MX dtype additions (B) — the native-vs-quantized synthesis

The line that reconciles KISS (sub-byte-as-dtype), Fuel (sub-byte-as-encoding-layer), and
Baracuda/Unpopped (Fuel's precise framing, adopted): the **element dtype** and the **MX block
structure** are two separate axes.

- **Element dtypes** — `f8e4m3fn`/`f8e5m2` (native FP8) *and* the sub-byte floats `f6e2m3`,
  `f6e3m2` (MXFP6), `f4` (MXFP4, `e2m1`) — are genuine **value dtypes**, alongside the native narrow
  integers `i4`/`u4`/`b1` (which also stay dtypes; packing is a separate layout coordinate). These all
  close §3.2's width self-check for a signed value float (`1 + e + m = bits`): `f8e4m3fn` = 1+4+3,
  `f8e5m2` = 1+5+2 (both 8); `f6e2m3`/`f6e3m2` = 1+2+3 / 1+3+2 (both 6); `f4` (`e2m1`) = 1+2+1 (4).
- **`f8e6m2` — RESOLVED as an unsigned 8-bit SCALE (maintainer, 2026-08-06; Vulkane's self-check
  catch).** Not a native FP8 element dtype: as a *signed* value float it would need `1 + 6 + 2 = 9`
  bits, but Fuel's E6M2 is an **unsigned scale** (`0 + 6 + 2 = 8`, which closes the self-check) — a
  finer-granularity sibling of `f8e8m0` (+2 mantissa, −2 exponent, so less dynamic range). It belongs
  to the **scale category** below, declared unsigned, and **rejoins the byte-match set** as a scale;
  Fuel's token is unchanged (already the `f8e8m0` posture: `is_float`, 1 byte, token-only).
- **MX block structure** — grouping elements into a block with **one shared `f8e8m0` scale** — is a
  separate **encoding / SType axis** with the scale as a **sibling operand**, not part of the value
  type. `f8e8m0` is the **scale type**, not an element dtype (E8M0 = 8-bit, all-exponent, no
  mantissa). The **scale types are `f8e8m0` and `f8e6m2`** (the latter maintainer-ruled 2026-08-06),
  both **unsigned** — a scale carries **no sign bit** — so the self-check for a scale is
  `exp + mantissa`: `f8e8m0` = `8 + 0 = 8`, `f8e6m2` = `6 + 2 = 8` (a finer-granularity scale, +2
  mantissa / −2 exponent vs `f8e8m0`, less range). The signed-float `1 + exp + mantissa` check does
  **not** apply to a scale — the implied-sign term is for signed value floats only (Fuel). So "native vs
  quantized" maps to **dtype(element) vs encoding(block-scale)** — the sub-byte-as-encoding-axis
  convergence, settled **here** rather than smuggled under the float class.

**MX float semantics MUST be pinned by citing OCP-MX, not by analogy** (kiss-ref's anti-invention
rule). `f4`=`e2m1` is a sub-byte **float** — new territory for KISS (whose sub-byte set was
integers-only) — so its bias, saturation behaviour, special-value encodings, and packing MUST be
fixed by explicit reference to the OCP Microscaling (MX) specification, exactly as the existing FP8
dtypes cite OCP OFP8 (§6.16). No new numeric semantics are invented in this RFC; the §6.1/§6.16
clauses cite OCP-MX for every added format. The new value codes are otherwise **additive** (new
tokens; no existing token changes) — they ride this event only to avoid a second regen.

### 3.3 Non-contraction precision coordinate `(acc + mp)` (C)

The non-contraction key grows a precision coordinate set that is the analogue of the `gem` cell's
`<acc>` + `<mp>`:

- **Accumulator dtype** — realizing KISS-CLASSIFY-6.7-0012 verbatim: a non-contraction
  reduction/scan cell whose accumulator dtype differs from its compute dtype carries an
  accumulator-dtype coordinate drawn from the closed §6.1 set. **Absent ⇒ accumulator == compute
  dtype** (the §6.17-0005 diagonal), so every existing token is unchanged in meaning.
- **Math precision** — extending `<mp>` (§6.7-0006) to the non-contraction key so the strict-vs-TF32
  axis stops collapsing for reductions.

**Ops-side dependency:** for the accumulator to be *requestable* (part of identity), the op must be
able to express it — today `reduce` OpAttrs are exactly `{monoid, reduce_axes, keepdim}`
(§6.19-0025), no accumulator. So sk4's C-leg spans **three** sub-standards: KISS-Ops (accumulator/mp
as reduce/scan OpAttrs), KISS-Classify (the key coordinate), KISS-Contract (§6.8-0012 numeric
formula, already parked). Baracuda is landing an `Access::Reduction` accumulator field pre-extraction
(defaulted, behaviour-preserving) — the impl-side seam this coordinate slots into.

**Scope boundary (Fuel's check).** sk4's C-leg is the **byte-visible `structure_key` coordinate**
`(acc + mp)` only — the identity/lookup surface. The in-memory **guarantee-type** unification
(`PrecisionGuarantee`/`DetClass`: `max_relative`/`max_absolute` tolerances, the same-hardware-bitwise
vs any-hardware-bitwise `exact-byte` split) is the **separate later precision workstream**, a
KISS-Contract surface change that is **not** schema-visible. They do not collide: one is what the key
carries for lookup, the other is what the contract declares as a guarantee.

### 3.4 Version-prefix inseparability (D) — foundational codec clause

Final wording (Unpopped, codec owner), for adoption verbatim:

> The schema version prefix is normative and inseparable. A dtype token has no meaning detached from
> the `structure_key` schema version that produced it. Implementations MUST NOT persist, index,
> cache, or compare dtype sub-tokens independently of their schema version, and MUST NOT compare
> tokens across schema versions for equality of meaning. A decoder MUST reject a token whose spelling
> is not valid at the declared schema version rather than interpreting it under a different version's
> vocabulary.

The final sentence does real work: it makes the **retired**-token cases fail *loudly* instead of
being silently reinterpreted under the wrong version's vocabulary — without it the clause covers
detachment but not misinterpretation. This protects **every** schema bump, not just this one; the
real hazard is a persisted cache key or an index that kept `"c64"` without its `"sk4"` (§4).

### 3.5 Deferred: per-operand dtype

Not in `sk4`. It unblocks the indexed op class (gather/scatter/embedding/block-tables/ragged/CSR —
all float-data + integer-index, hence inherently mixed-dtype and undescribable under KISS's
single-operand-0 dtype slot), but: it is a **describability** gap (the numeric-determining axes —
data dtype = operand-0, index width = `ix32`/`ix64` — are already keyed); adding it **reverses**
KISS-CLASSIFY-6.6-0015's *deliberate* caller-precondition; and it is not urgent (Fuel's motivating
consumer routes around the JIT seam). It is coupled to the catalog-vs-compiler / indexed-region-
synthesis scope decision and belongs to that RFC, not this one.

## 4. Migration & parseability

The token is version-prefixed (`sk4|…`, `STRUCTURE_KEY_VERSION: u16 = 4`), and that prefix is what
makes the whole migration tractable. **Wire-visibility is narrower than the Rust-variant renames
suggest** (Unpopped's codec read): `Bin`→`B1` already tokenizes as `b1` both versions (variant-only,
wire-invisible). Per-impl FP8 deltas to the uniform target **`f8e4m3fn`/`f8e5m2`** (the §6.1 spec
token is `e4m3fn`): a crate already at `e4m3fn` (`unpopped-vocab`) adds the `f8` **prefix only**;
`kiss-classify-vocab`, **drifted** to a bare `e4m3` (kiss-ref's catch — since §6.1 is `e4m3fn`, a
pre-existing spec-vs-crate drift folded into the regen), adds **prefix + suffix**; Fuel, at `f8e4m3`
(Fuel repo, `fuel-ir` crate — `src/dtype.rs:106`), adds the `fn` **suffix only**. The target is `f8e4m3fn`/`f8e5m2` regardless
of a party's starting spelling.
Schema-visible changes overall: `s8`→`i8`, `s4`→`i4`, the `f8` float prefix, the mandatory FP8 variant
suffix where a crate lacks it, and the complex meaning-flip.

**The token-collision check localizes the entire risk to one token.** Across the full `sk3` and `sk4`
sets: every retired spelling (`s8`, `s4`, `e4m3fn`, `e5m2`, `c32`) is **version-exclusive** — an
`sk4` decoder meeting `s8` or `c32`, or an `sk3` decoder meeting `c128`, rejects it as
unknown-at-this-version and fails loudly (clause D's final sentence). **`c64` is the sole token that
decodes successfully under *both* schemas while meaning different things** — a pair of `f64` at
`sk3`, a pair of `f32` at `sk4`. **This holds only because the `sk4` complex set is pinned exactly to
`{c64, c128}`** — `sk4` does **not** add complex-`f16` (Vulkane's catch). If it did, complex-`f16`
would spell `c32` (total width 32 bits) and `c32` would *also* dual-decode (`sk3` pair-of-`f32`, 64
bits, vs `sk4` pair-of-`f16`, 32 bits) — the same 2× hazard. Adding complex-`f16` is a **future**
decision that would carry its own `c32` migration note; at this event the set is `{c64, c128}`. `c64`
is safe *because of* the prefix (`sk3|…|c64` vs `sk4|…|c64` are never confused; cross-version
string-equality cannot false-match), but it is **not** presentable as a
mechanical rename.

- **Pure renames** stay parseable across the bump via a bounded `sk3` decoder arm mapping the old
  spelling to the same variant (meaning unchanged).
- **`c64` MUST ship an explicit migration note** naming the token: a persisted `c64` written under
  `sk3` denotes what `c128` denotes under `sk4`, and a consumer holding the new `Complex64` must know
  it now denotes what `Complex32` denoted. This is the one token that cannot be silently migrated.
- **The `sk3` decoder arm MUST be bounded**, not indefinite (Unpopped): a permanently-retained
  old-version arm is its own maintenance hazard. **Concrete pin (ratified value):** an
  implementation **MAY** retain a bounded `sk3` decoder arm **through the `sk4` series**, but it
  **MUST NOT** accept `sk3`-prefixed tokens at **`sk5`** or later — retirement is *scheduled* to the
  `sk5` event (which every party already tracks), not left to a calendar nobody shares. A consumer
  **MUST NOT** rely on cross-implementation `sk3` acceptance (the arm is a MAY, so acceptance is
  implementation-dependent): a persisted `sk3` token **MUST** be re-derived, not assumed decodable.
  Because the arm is a permission, not a mandate, the KISS **reference** codec ships **no** `sk3` arm
  and declines every non-`sk4` version with a typed *recognized-but-unaccepted* decline that names
  the version (the "re-derive" signpost), distinct from the malformed-field decline. Normalized to
  clause text as **KISS-CLASSIFY-6.7-0014** (retirement policy) and **KISS-CLASSIFY-6.7-0015** (the
  signpost-vs-wall decline split). The arm's soundness **depends on** exact version-prefix matching
  (§6.7-0002/-0015): `c64` decodes as pair-`f32` under `sk3` and pair-`f64` under `sk4`, so a reader
  that loosened the version match would silently decode a cross-vocabulary token under the wrong
  dtype semantics.
- **The one systemic hazard** is a consumer that detaches the sub-token from its prefix (§3.4) —
  exactly what clause D forbids.

## 5. Clause impact (indicative)

Authoritative clause text is authored post-cosign; this is the map.

- **KISS-Classify §6.1** — replace the enum with the §3.1 grammar + the closed token set (renamed
  ints, width-prefixed floats, complex-by-total-width, MX value codes + the `e8m0` scale category).
- **KISS-Classify §6.7** — `STRUCTURE_KEY_VERSION` 3→4, token prefix `sk3`→`sk4`; the §3.4
  version-prefix inseparability clause; the non-contraction precision field codec (§3.3).
- **KISS-Classify §6.6 / §6.7-0012** — realize the accumulator forward-requirement; extend `<mp>` to
  the non-contraction key.
- **KISS-Ops §6.19-0025 / §6.17** — accumulator + math-precision as reduce/scan OpAttrs; the
  `(compute, accumulator)` and strict-vs-TF32 tolerance cells already exist (§6.17-0008/-0009).
- **KISS-Contract §6.8** — the accumulation-type/precision guarantee surfaces already present
  (§6.8-0011/-0012); confirm alignment with the new key coordinates.

## 6. Regen & byte-match acceptance gate

Adoption requires **four independent `structure_key` token derivations to byte-match**: KISS (spec +
reference codec), Fuel, kiss-ref, and Unpopped's `unpopped-vocab` each independently regenerate the
corpus under `sk4` and the tokens byte-match across all four. Agreement is the conformance evidence
that the grammar is unambiguous; one shared crate would prove nothing — which is exactly why
**Baracuda is not counted a fifth token derivation**: `baracuda-kernels-types` re-exports
`unpopped-vocab`'s derivation wholesale (`pub use unpopped_vocab::*`; no own `fn structure_key` /
`STRUCTURE_KEY_VERSION`), so a Baracuda↔Unpopped match is code-identity, not convergence.
**Baracuda's independent leg is the physical emit corpus** — its CUDA `.cu` kernels + emitted
contract, keyed by the `sk4` structure_key, regenerated at the cut. This witnesses a **different
proposition** than the token byte-match, and the distinction is load-bearing (Unpopped's
source-verified sharpening, 2026-08-07): the corpus is **not** independent evidence of token
*spelling*, because the tokens stamped into the `.cu`/contract front matter are `unpopped-vocab`'s
`to_token()` output (Unpopped repo, `unpopped` crate — `src/contract.rs:746`, calling `unpopped-vocab`'s `to_token()` at `src/structure_key.rs:1150`) — so on spelling the corpus is **downstream of
`unpopped-vocab`**, not a fifth independent derivation. What it *does* witness is **semantic
assignment + coverage**: that each shipped physical CUDA kernel is classified into the cell it
actually implements, and that the `(acc + mp)` field flows key → `baracuda-cuda-emit` →
kernel + goldens faithfully — i.e. the LOGICAL→PHYSICAL lowering is correct, over the real corpus, a
surface the token byte-match does not reach. (Confirmed with Baracuda, 2026-08-07: `kiss-ref-diff`
adds **no** spelling independence either — it *constructs* keys via the shared
`unpopped_vocab::structure_key(…)` to drive generation and never re-derives the token from a
contract, so on spelling it is downstream of `unpopped-vocab` like the emit path. Its own independent
subject is a *different* surface — a three-way **numeric** bit-comparison of the semantics DAG
(kernelgen CPU oracle / kiss-ref `eval_recipe` / on-device generated CUDA kernel), which *reinforces*
this semantic-assignment/coverage leg rather than adding a spelling derivation.) The
reference crates reproduce §6.1 (§3.1.6), but the `structure_key` derivation stays independently
implemented per token party.

**This is a synchronized BREAKING release, not a patch.** It changes public enum variants, normative
tokens, and the conformance corpus, so every party cuts a coordinated **semver-major** release (e.g.
`0.3.0` across kiss-ref's three crates) in lockstep, and each invalidates any token-keyed caches
(Fuel's Judge profile caches, provision caches) as part of the cut. No party ships `sk4` tokens
before the byte-match passes; none retains `sk3` tokens after its migration window (§4) closes.

**Enum-hardening (`#[non_exhaustive]`) fixes RIDE this cut, they do not precede it.** Because `sk4`
already forces a semver-major on every affected vocab crate (the `s8`→`i8` rename alone does it), any
`#[non_exhaustive]` annotation a party still needs is folded into the *same* coordinated major — a
separate pre-`sk4` `0.2.0` would be **two** breaking releases for downstream instead of one. The
payoff of the annotation is forward: it lets a *future* additive bump (e.g. `sk5`'s MX-style codes)
land without forcing another major.

**Scope of the cut — token-deriving / schema-surface crates only** (maintainer-ratified, Eric 2026-08-08). The coordinated major binds the crates that carry the schema breaking change: those that derive `structure_key` tokens or hold the affected vocabulary (the `s8`→`i8` rename, the enum-hardening). A party's *downstream* crates whose breaking changes are unrelated to the schema — a generator or emitter that consumes the vocabulary but derives no tokens — are **not** gated on this cut and MAY release on their own schedule; gating them would add a break for their downstream with none of the one-coordinated-major benefit. (Where a party's several crates are all schema-surface — e.g. kiss-ref's three — they move together for *that* reason, not because co-ownership implies lockstep.)

**Per-impl regen deltas differ; the byte-match target is uniform.** Illustratively: Fuel is already
`i`-prefixed (no `s`→`i` change) but adds the FP8 `fn` suffix (it emits `f8e4m3` today); the
`c32`/`c64` complex meaning-flip is a **no-op** only for an impl without complex dtypes (Fuel =
none), which plans **no** codec work on the complex axis; kiss-ref implemented the §6.18
complex-arithmetic family after this RFC's initial draft (v0.2.4, op corpus 106→121), so the flip
**applies** to kiss-ref at the coordinated regen — its complex-axis leg is real, not a no-op; `bf16`
tokenization aligns to §3.1.3. Each party's delta is its own; only the emitted `sk4` tokens must
agree.

## 7. Cosign & sign-off

**Six cosigners; four token-derivation parties + one emit-corpus party.** The four independent
`structure_key` **token** derivations — KISS, Fuel, kiss-ref, and Unpopped's `unpopped-vocab` — each
confirm the full set (coordinate set + grammar A/B/C, migration §4, clause D) **and** own a token
re-derivation + byte-match leg (§6). **Baracuda cosigns the full set and owns the emit-corpus leg**,
not a fifth token derivation: `baracuda-kernels-types` re-exports `unpopped-vocab`
(`pub use unpopped_vocab::*`; no own `fn structure_key` / `STRUCTURE_KEY_VERSION`), so its
independent evidence is the physical CUDA `.cu` + contract emit keyed by the `sk4` key, regenerated
at the cut. That corpus witnesses **semantic assignment + coverage** (each shipped kernel is
classified into the cell it implements; the lowering is faithful over the real corpus) — **not**
independent token *spelling*, since the stamped tokens are `unpopped-vocab`'s `to_token()` output and
so are downstream of it on spelling (Baracuda's own party-model catch + Unpopped's spelling-scope
sharpening, 2026-08-07; see §6). **Vulkane cosigns the §3.1/§3.2 vocabulary and clause D and
derives no `structure_key`** — its obligation is `kiss-vulkan-vocab`'s dtype spellings and the §6.8
`target_capability` surface — so it is **not** a byte-match party either. Unpopped additionally
supplies the final codec wording (§3.4) and the rename delta (`docs/dtype-spelling-delta.md`).

Cosigns received (each verified against its own code, not the relay): **Unpopped, Fuel, kiss-ref**.
**Vulkane** — yes on substance, the §7/§9-2/§10 cleanups folded; bf16 spelling confirmation discharged and verified against its own code (`kiss-vulkan-vocab/src/lib.rs:536` and `kiss-vulkan-vocab/src/lib.rs:554` round-trip under the Option C class model, no spelling change — §3.1.3's one open cosign-confirmation, now closed; scope: token *spelling*, not end-to-end device derivation — a separate, non-blocking Vulkane-side item where `component()` has no BF16 arm so `VK_COMPONENT_TYPE_BFLOAT16_KHR` → `Other(n)`, the token unchanged under Option C). **Baracuda** — yes on substance
(verified against its own code), contributing the §6/§7 party-model refinement folded into this
revision: Baracuda re-exports `unpopped-vocab`, so its leg is the physical emit corpus, not a fifth
independent token derivation. All six cosigns are recorded and verified against each party's own code (Vulkane's via the bf16 spelling confirmation above; KISS as hub/steward); the token byte-match is four-way. **Event ratified 2026-08-08** (see Status).

## 8. Out of scope / separate workstreams

- **Backward-op category convention** — training space is a first-class KISS use case (maintainer,
  2026-08-06). Backward ops are in scope, drafted **separately** (lean: non-primitive decompositions
  named for dispatch, preserving the forward-only primitive floor). Not schema-affecting; not part of
  `sk4`.
- **Per-operand dtype / indexed-region-synthesis** (§3.5) — its own scope decision + RFC.
- **Catalog-vs-compiler capability bit** — stands on trust-model merits, not urgent; separate RFC.
- **Request-time operand fidelity** — a normative **MUST on the request itself** (not merely a
  conformance-test obligation): requests MUST carry honest operand types. Fuel's near-miss
  (substituting `i32` for `u32` index operands would have made the region uniform-dtype and *falsely
  passed* the synthesizer gate) shows an approximate request produces a confidently-wrong *result*,
  not merely an untested one — so it is a request-validity MUST. Independently valuable, tracked
  separately.

## 9. Resolved decisions (maintainer, 2026-08-06)

Both cosign-surfaced scope questions are **resolved**; §3.1.3 and §3.3 are final for cosign (subject
only to the one cosign-confirmation noted on bf16).

1. **`bf16` spelling → Option C (`bf` as a grammar class), 2026-08-06.** `fn`/`fnuz` vary
   special-values over a *fixed* layout, but `bf16`/`f16` vary the *layout* itself — so `bf`-as-an-
   `f16`-*variant* (the originally-ratified literal) was a category error. Option C makes `bf` a class
   (`bf16` = `{class bf}{bits 16}`), keeping the conventional `bf16` on the wire with **no alias
   table** (removing the normalize-vs-hash hazard that the layout-explicit-with-aliases alternative
   introduced) and no category error. Decodability holds (`b1`/`bf16`/`bool` split at char 2).
   **Ruled by the maintainer conditional on the six-way cosign confirming every project can
   emit/consume `bf16` under the class model** — all currently tokenize `bf16`, so this is expected
   unanimous; a project that cannot support C flags it at cosign.
2. **`(acc + mp)` identity coordinate → KEEP in `sk4`, 2026-08-06.** The earlier "pull" recommendation
   (steward relaying Vulkane; Fuel) rested on a **conflated premise** and was withdrawn: the
   accumulator DTYPE and mp MODE are **identity** coordinates — an f32-vs-f16-accumulate reduction, or
   strict-SIMT vs TF32, is a different kernel with different bits (the *evidenced* §2 collision) —
   whereas Vulkane's transcendental-ULP / reduction-order / denorm terms are **guarantee-surface**
   accuracy (deferred to the precision workstream), never structure_key coordinates. `(acc + mp)`
   stays; only the reduction-order/denorm accuracy axes move to the future guarantee work. (Nuance,
   Fuel: schedule-level reduction order — split-K / tree / sequential — is a GENERATOR choice =
   *identity*;
   only hardware wave-order is *guarantee*.) **Ruled KEEP by the maintainer, 2026-08-06** — the
   coordinate stays in `sk4`.

## 10. Cosign pins to fold at clause-authoring (accepted, not yet in clause text)

- **Layout ⇒ variant table (normative) — precondition, not hygiene, with a named blocked consumer.**
  Some sources name an FP8 *layout* with no variant (`VK_COMPONENT_TYPE_FLOAT8_E4M3_EXT`, Fuel
  `f8e4m3`), yet §3.1.5 makes the `fn`/`fnuz` suffix mandatory and nothing in those sources states
  which variant the layout denotes. **Vulkane therefore cannot write its FP8 arms at all until this
  table is ratified** — its only options otherwise are to guess a variant its source doesn't carry, or
  emit `Other(n)` and drop the type. So ratify **once** which variant each bare layout maps to (e.g.
  `VK_..._E4M3_EXT` / `f8e4m3` ⇒ `f8e4m3fn`), reasoning recorded, and record that Vulkane's FP8
  implementation is blocked on it — this is the **precondition** for implementing §3.1.5, not a nicety.
- **Terminology (unconditional — §9-2 ruled KEEP).** "Non-contraction" here means **not a
  tensor-contraction (GEMM) cell** (pointwise/reduction/elementwise), **not** "no FP contraction/FMA"
  — an elementwise `a*b+c` is a non-contraction *op* that may emit a contracted FMA. The clauses MUST
  say "pointwise/reduction vs tensor-contraction" to kill the collision. (This pin is now
  unconditional: the `(acc + mp)` coordinate stays, so its clauses ship and this terminology must be
  right in them.)
- **`Other(n)` reclassification hazard.** New `#[non_exhaustive]` vocab variants that were previously
  a catch-all `Other(n)` **silently reclassify** prior `Other` values — invisible, not a build break.
  A migration note must flag this alongside the version-prefix rule (§4).
- **Motivation strengthening.** The `kiss-classify-vocab` crate (kiss-ref repo), `src/lib.rs:42`, documents the `s8/s16`-vs-`i32/i64`
  mix as *intentional* ("reproduced verbatim"). `sk4` is therefore **fixing KISS's own inconsistency**,
  not merely aligning Fuel — and that comment must die with the rename.
