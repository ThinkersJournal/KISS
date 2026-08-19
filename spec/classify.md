# KISS-Classify — Data Vocabulary: Dtypes, Operand Descriptors, Specialization-Cell Identity & Target Capability

**Sub-standard ID:** KISS-CLASSIFY
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen. Deliberately **UNFROZEN** (the per-namespace target-capability vocabulary freeze waits on real non-CUDA usage — see §8).

> This document follows the KISS dual-doc template defined in the umbrella
> specification (umbrella §4): an **informative Overview** (§0–§5) and a
> **normative Conformance specification** (§6+). Only §6+ is normative. Normative
> clauses use RFC-2119 / RFC-8174 uppercase keywords, carry an append-only clause
> ID `KISS-CLASSIFY-<section>-<nnnn>`, and each MUST/SHALL maps 1:1 to at least one
> named KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test. KISS-Classify is a **plain-old-data vocabulary tier**: its
> §6 tables and its clause sidecar are generated from one canonical schema so they
> cannot drift.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Classify |
| Sub-standard ID | KISS-CLASSIFY |
| Tier | **Foundational** (data vocabulary) |
| Maturity stage | **Draft** (first-draft proposal; explicitly UNFROZEN pending a second dissimilar, non-CUDA target implementation) |
| Editor of record | **Proposed, pending ratification** — the data-vocabulary reference-impl project holds the pen (the ratified governance record assigns Classify to that project; the editor requests comment from interested cosignatories before deciding a cross-party-visible change). |
| Steward | ThinkersJournal |
| Reference seed crate(s) | the driver-free kernel-vocabulary crate (`baracuda-kernel-vocab`, project name given in Appendix A as non-normative provenance); this crate is *a* conformant implementation with no privilege. |
| DAG position | **Foundational tier — a root.** Depends on **nothing**. Sits at the bottom of the suite beside the computation vocabulary (KISS-Ops); every other sub-standard that talks about operands or specialization cells references it. |
| Upstream edges | **NONE.** KISS-Classify is foundational; it has no incoming dependency edge. It does not depend on KISS-Ops; the normative statement of that independence is §6.9-0001. |
| Downstream edges | KISS-Grammar (**STRUCTURAL** — consumes the dtype set and operand descriptors); KISS-Contract (**STRUCTURAL** — the contract identity/interface sections use operand descriptors and the `structure_key` accept-predicate); KISS-Announce (**OPAQUE** — carries `structure_key` as an uninterpreted, length-delimited token); KISS-Consume (**STRUCTURAL** — lift targets describe operands with this vocabulary); KISS-Emit (**STRUCTURAL** — the emitter's normative input pairs an op definition with a `structure_key`); KISS-Conform (test dependency — Conform tests this sub-standard). |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |

> **Edge-label note (informative).** The Classify↔Announce edge is labeled
> **OPAQUE** on both sides: KISS-Announce carries `structure_key` as an
> uninterpreted, length-delimited token and never parses its internals (announce
> §4). The Classify↔Grammar / Classify↔Contract / Classify↔Consume /
> Classify↔Emit edges are labeled **STRUCTURAL**: those sub-standards read the
> dtype tokens, operand-descriptor field names, and `structure_key` field layout
> defined here. All labels are consistent with the umbrella §2.2 edge table.

---

## 1. Purpose & Scope

KISS-Classify owns the **data noun set** of the suite: the vocabulary every other
KISS sub-standard uses to *name* the data a kernel accepts and produces, and to
*name* the specialization cell a kernel is built for. It defines five things and
nothing else:

1. **The pinned scalar dtype set** — every element type a kernel operand may
   carry, each with an exact storage bit width, a numeric kind, a stable spelling,
   and (for sub-byte and complex types) an exact packing convention.
2. **The operand descriptor** — the minimal, canonical description of one tensor
   operand: rank, per-axis extents, per-axis **signed** strides, dtype, base-pointer
   alignment, plus the derived layout tag and the cell-level op-family tag, and the
   optional quantization and symbolic-extent facts carried for binding.
3. **The specialization-cell identity `structure_key`** — an **admissibility
   predicate** over a layout/dtype/target specialization *cell*: a coarse,
   extent-free structural class computed canonically from the operand descriptors
   (including the cell-level op category), the target, and the op category's role
   hints, so a provider's build matrix and a consumer's runtime lookup join on the
   same token. It is **not** the op's semantic identity.
4. **The all-hardware target-capability descriptor** — the namespaced
   `<namespace>:<capability-set>` token that names the compilation target a
   specialized kernel is built for, matched byte-exact on the full string.
5. **The pinned structural constants** — `MAX_RANK`, `MAX_OPERANDS`, the
   `structure_key` schema version, and the `structure_key` maximum token length —
   that bound every array and buffer above.

**KISS-Classify is NOT:** the computation vocabulary (the op set and per-op
semantics — that is KISS-Ops); the op's semantic identity (a `structure_key` says
*which cell fits*, never *which op this is*); the kernel-contract format (that is
KISS-Contract); the discovery/handshake protocol (that is KISS-Announce); a kernel
implementation, a source language, a compiler IR, or an in-ecosystem loader.
Anything not enumerated as in-scope above is out of scope for KISS-Classify (scope
creep by silence is a named trap; silence is not inclusion).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — nouns, not verbs

A provider may hold **thousands** of specialized kernel *cells*, one per
layout/dtype/target specialization of some computation. KISS-Classify supplies the
**nouns** those cells and their operands are described with; the sibling
foundational vocabulary KISS-Ops supplies the **verbs** (what a kernel computes).
Keeping the two apart is deliberate and load-bearing: a `structure_key` answers
"which *cell* does this invocation fall into?" while an op name (KISS-Ops) answers
"which *computation* is this?". A consumer that wants a kernel must match **both** —
the two together are kernel identity, and neither alone is sufficient (umbrella
§2.1; charter §4a).

Because the data vocabulary is foundational, it depends on nothing. It must not
reach up into the op vocabulary: `structure_key` carries a coarse **op-family tag**
(an elementwise/reduction/contraction *category*), never a KISS-Ops op *name*. This
keeps Classify a true root and lets it freeze on its own cadence.

### 2.2 Why dtypes are pinned by bits, not by spelling

A foreign implementor — a C, SPIR-V, or plain-CPU reader who has never seen the
reference language — must reproduce the exact storage width and packing of every
dtype. So each dtype is pinned by its **storage bit width**, its **numeric kind**,
its **stable lowercase token** (the spelling that appears in a `structure_key`
token and in an operand descriptor), and, for sub-byte and complex types, its
**exact packing convention**. Two subtleties the table nails:

- **Dtypes are pure storage.** A dtype pins **byte layout only** — width, kind,
  spelling, and (for sub-byte and complex types) packing. **Compute precision is
  not a dtype.** For example, whether a `f32` multiply-add must be bit-stable
  full-precision or may use a reduced-mantissa reduction (e.g. TF32-style tensor
  math) is a *numeric-fidelity* fact, not a byte-layout fact; it is owned by
  KISS-Ops as a `MathPrecision`-style fidelity attribute (alongside the Ops-owned
  determinism/fidelity enum) and surfaced in a kernel's KISS-Contract guarantees.
  Classify therefore carries **one** binary32 storage dtype, `f32`, and defines no
  strict-precision variant; a kernel that needs bit-stable compute states that
  through the KISS-Ops fidelity attribute, not through a distinct dtype token.
- **`u32` is an ordinary storage dtype.** `u32` is a plain unsigned-32-bit storage
  type with the same container width as `i32` (4 bytes); it is legal on any operand
  path. Index-only-ness is **not** a dtype class: whether an operand is used as a
  gather/scatter/embedding index or address is an **operand role**, carried by
  KISS-Ops on the gather/scatter operand (the `index_operand` designation plus its
  `index_dtype`), never encoded as a Classify dtype.

### 2.3 Signed strides are the whole point of the descriptor

The operand descriptor carries **signed 64-bit** strides in *element* units, not
bytes. A stride of `0` means the axis broadcasts (the load is hoisted out of the
loop); a **negative** stride means the axis is reversed (a flipped view). This one
choice lets a single descriptor express contiguous, transposed, broadcast, and
reversed views without a separate flag soup. The layout tag (contiguous /
inner-contiguous / strided / broadcast) is a *derived projection* of extents and
strides, not an independently stored field — so it can never disagree with the raw
shape. Axes are ordered **outermost first** (axis index `0` outermost, axis index
`rank−1` innermost); every "innermost" derivation uses the highest active axis
index (§6.3-0011).

### 2.4 `structure_key` is an admissibility predicate over a cell

A `structure_key` is the canonical identity of a specialization **cell**. A kernel
accepts an invocation **if and only if** the `structure_key` derived from that
invocation's derivation inputs — the canonically-ordered operand descriptors
(including the cell-level op category), the target, and the op category's role hints
(§6.6-0012) — **byte-matches** the kernel's key. It is deliberately **extent-free**:
it keys size *classes* (one-warp / one-block / grid-stride; tiny / small / mid /
large), never literal extents, so that the same kernel serves a whole family of
shapes and the build matrix stays finite. Because the derivation is canonical and
deterministic, a provider building `add`-on-`f32`-on-`cuda:sm89`-contiguous and a
consumer looking one up compute the *same bytes* and join with no shared code beyond
this one function.

The key is **carried opaquely** across the discovery/announce seam: KISS-Announce
frames it as a length-delimited token (maximum 4096 bytes) and never parses it. The
token codec is **spelling-keyed, not discriminant-keyed** — adding a new dtype code
or op-family code shifts no existing token's bytes — so the vocabulary can grow
without a schema-version bump. The string token is the **only** normative wire form
(§6.7-0011); the binary field types quoted in the schema tables describe an
in-memory representation only.

### 2.5 The target is namespaced and matched byte-exact

The old reference impl carried a CUDA-only `ArchSku` enum (`Sm80`/`Sm89`/`Sm90a`).
That contradicts a suite meant to be shared across all hardware. KISS-Classify
replaces it with a **namespaced, all-hardware** descriptor
`<namespace>:<capability-set>` — `cuda:sm89`, `rocm:gfx942`,
`metal:apple9`, and so on. The steward registers **namespaces**; each namespace's
maintainer owns that namespace's **capability-set** vocabulary. Matching is
**byte-exact on the full string**: no ordering rules, no subset logic, no
feature-implication. `cuda:sm89` matches `cuda:sm89` and nothing else. A kernel
built for one target is never silently reused on another.

### 2.6 Readable catalog — the dtype set

The complete pinned scalar dtype set (normative table in §6.1):

| Token | Kind | Bits | Notes |
|---|---|---|---|
| `f16` | float | 16 | IEEE-754 binary16 (1s+5e+10m), half-precision storage |
| `bf16` | float | 16 | bfloat16 (1s+8e+7m); f32 exponent range, reduced mantissa |
| `f32` | float | 32 | IEEE-754 binary32 storage (compute precision is a KISS-Ops fidelity attribute, not a dtype) |
| `f64` | float | 64 | IEEE-754 binary64 |
| `i8` | int | 8 | signed 8-bit two's-complement (sk3 `s8`) |
| `i16` | int | 16 | signed 16-bit two's-complement (sk3 `s16`) |
| `u8` | uint | 8 | unsigned 8-bit; also the physical storage of `bool` |
| `u16` | uint | 16 | unsigned 16-bit |
| `i32` | int | 32 | signed 32-bit two's-complement |
| `i64` | int | 64 | signed 64-bit two's-complement |
| `u32` | uint | 32 | ordinary unsigned 32-bit storage; container width matches `i32`; the index/address *role* is carried by KISS-Ops on the gather/scatter operand, not a dtype class |
| `u64` | uint | 64 | unsigned 64-bit |
| `bool` | bool | 8 | 1-byte truth value; `0` = false, any non-zero byte = true; ops normalize to 0/1; storage width equals `u8` |
| `f8e4m3fn` | float | 8 | FP8 E4M3 OCP (1s+4e+3m, bias 7); max finite ±448, no infinities, single NaN (sk3 `e4m3fn`) |
| `f8e4m3fnuz` | float | 8 | FP8 E4M3 AMD `fnuz` variant (bias 8, no −0, no infinities); byte-incompatible with `f8e4m3fn`; reserved (recognized on parse; use typed-declines at this schema version) |
| `f8e5m2` | float | 8 | FP8 E5M2 (1s+5e+2m, bias 15); max finite ±57344, IEEE-style inf/NaN (sk3 `e5m2`) |
| `f8e5m2fnuz` | float | 8 | FP8 E5M2 AMD `fnuz` variant (bias 16, no −0, no infinities); byte-incompatible with `f8e5m2`; reserved (recognized on parse; use typed-declines at this schema version) |
| `f8e8m0` | float | 8 | MX shared-exponent scale (unsigned; 8 exp, 0 mantissa); OCP Microscaling; a sibling-operand scale type, not an element value dtype (new at sk4) |
| `f8e6m2` | float | 8 | MX scale (unsigned; 6 exp, 2 mantissa); finer-granularity sibling of `f8e8m0`; a scale type, not an element value dtype (new at sk4) |
| `i4` | int | 4 | signed 4-bit `[-8,+7]`; packed-pair storage (low nibble = even index, high nibble = odd index); sign-extended on read (sk3 `s4`) |
| `u4` | uint | 4 | unsigned 4-bit `[0,15]`; packed-pair storage identical to `i4`; zero-extended on read |
| `b1` | uint | 1 | 1-bit binary-GEMM operand; packed-byte storage (8 bits/byte, LSB = lowest logical index); xor+popcount accumulation, raw `i32` output (reference name `Bin`) |
| `c64` | complex | 64 | complex: interleaved (re,im) pair of `f32`, 64 bits total (named by total width; sk3 `c32`); complex arithmetic semantics owned by KISS-Ops (Classify pins storage only) |
| `c128` | complex | 128 | complex: interleaved (re,im) pair of `f64`, 128 bits total (named by total width; sk3 `c64`) |

Twenty-four dtypes, five numeric kinds (`float`, `int`, `uint`, `bool`, `complex`),
no "etc.". (sk4: the two 8-bit MX scales `f8e8m0`/`f8e6m2` are additive; the FP8/complex
respellings and the `s`→`i` integer renames are covered by §3.1.)

### 2.7 Readable catalog — the operand descriptor

| Field | Type | Meaning |
|---|---|---|
| `rank` | u8, `0..=MAX_RANK` | tensor rank; only `extents[0..rank]` / `strides[0..rank]` are meaningful |
| `extents` | `i64[MAX_RANK]` | per-axis logical extents (capacity for a symbolic axis, not live length) |
| `strides` | `i64[MAX_RANK]` | per-axis **signed** element strides (element units, not bytes); `0` = broadcast, `< 0` = reversed |
| `dtype` | dtype token (§6.1) | the operand's element dtype; the cell's primary dtype is operand-0's dtype |
| `alignment` | u32 | base-pointer alignment in bytes; drives achievable vector width |
| `layout_tag` | derived enum | per-operand memory-layout class (contiguous / inner-contiguous / strided / broadcast); a projection of extents+strides, part of the per-operand sub-key, not a stored raw field |
| `op_family_tag` | enum (cell-level) | the coarse op category the cell participates in; drives canonicalization legality; carried at the cell level, not per raw operand |
| `quant` | optional | quantization facts (family, sub-byte bit width, block extent, scale placement, dequant form); carried for binding, **not** folded into the admissibility key at this schema version. `dequant_form` **becomes keyed at the next schema version** (§6.3-0009a) — linear and codebook are not inter-substitutable; every other field stays out at every version |
| `symbolic_extent` | optional | live-vs-capacity dynamic-extent fact (axis + kind: scalar bound / range / affine); flags a dynamic live length even though capacity keys the strides |

### 2.8 Worked examples (informative)

**(a) Binary elementwise add, `[128,256]` row-major `f32`, target `cuda:sm89`.**
Three operands in canonical order (`in`, `in`, `out`), each `extents=[128,256]`,
`strides=[256,1]`, `alignment=256`. Each operand is contiguous (`co`), no broadcast
(`00`), vectorizes to V4 (inner extent 256, 256-byte aligned, f32 caps at float4 =
16 bytes), inner extent divisible by 16 (`d16`), not flipped (`f`). Max touched
offset `256·127 + 1·255 = 32767 < 2³¹` ⇒ `ix32`; iteration-frame element count
`128·256 = 32768 > 1024` ⇒ `grid`; iteration rank 2. The token:

```
sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-
```

**(b) A broadcast operand.** If operand 1 is `strides=[0,1]` (axis 0 broadcasts
over iteration extent 128), its sub-key becomes `br/01/v1/…` — layout tag
`broadcast`, broadcast-axis mask bit 0 set (`01`), scalar vector width (a broadcast
operand derives `v1`, §6.5-0009). A different cell, a different token, an honest miss
against the all-contiguous build.

**(c) A reduction, keepdim form.** The reduce field has three reserved,
distinctly-encoded values plus a general bitmask (§6.6-0009, §6.7-0005): `-` =
none / not-a-reduction; `rall` = all-axes reduction; `rlast` = trailing-axis
(innermost) reduction; `x<hh>` = an explicit keepdim bitmask for any other axis
set. Input `[4,8]` reducing the last (innermost) axis to keepdim output `[4,1]` (op
category `reduction`, caller-supplied) is a *trailing-axis* reduction ⇒ token field
`rlast` (the reduce-**field encoding** `rlast` is rank-independent — the same field
value is used for any rank whose innermost axis alone is reduced — though the overall
key still varies with the `rank` field, so a rank-2 `rlast` token and a rank-3
`rlast` token differ by bytes and do not match). Reducing **every** axis of `[4,8]`
to `[1,1]` ⇒ `rall`. Reducing only
axis 0 of `[4,8]` to keepdim `[1,8]` is neither all-axes nor trailing, so it uses
the explicit bitmask with bit 0 set ⇒ `x01`. These are distinct cells and distinct
tokens. A *collapsed* (rank-reduced) output `[4]` is **not keyable** — axis 0 and
axis 1 both collapse to `[4]`, so the cell would be indistinguishable; §6.6-0009
requires reductions to be presented in keepdim form and a collapsed reduction
yields a typed decline. The reduced operand's innermost axis is a reduced axis
(under `rall`, `rlast`, or an `x<hh>` mask whose innermost bit is set), so it
derives scalar vector width (`v1`, §6.5-0009).

**(d) A dense GEMM cell.** `lhs [8,4096] · rhs [4096,4096] → out [8,4096]`,
row-major, op category `contraction`; the caller supplies M/N/K axis roles (lhs
`[M,K]`, rhs `[K,N]`, out `[M,N]`, §6.6-0016). M is tiny (≤8), N and K are large
(>2048), K is divisible by 16 ⇒ optional trailing contraction field `|ctll/d16`.
Non-contraction tokens omit that field entirely and stay byte-identical to the base
codec.

**(e) Byte-exact target matching.** A kernel with target `cuda:sm89` does **not**
serve an invocation classified for `cuda:sm90a`, nor for `rocm:gfx942`. The
full string differs by at least one byte, so the keys do not match; the consumer
sees a miss and asks the provider to provision the right cell.

### 2.9 Terms are joined, not restated

KISS-Classify references the umbrella conventions (keywords, clause-ID scheme,
value pinning) rather than restating them, and it references the op-semantics
currency (KISS-Ops) only to *disclaim* it: a `structure_key` is not an op name, and
Classify defines no op meaning.

---

## 3. Terms & Definitions

- **dtype** — a scalar element type from the pinned set of §6.1, identified by its
  stable lowercase token (e.g. `f16`, `f32`, `b1`).
- **numeric kind** — the coarse classification of a dtype: one of `float`, `int`,
  `uint`, `bool`, `complex`.
- **operand descriptor** — the minimal per-operand description of §6.3: rank,
  extents, strides, dtype, alignment, plus derived layout tag, cell-level op-family
  tag, and optional quant / symbolic-extent facts.
- **extent** — the logical length of one axis; for a symbolic axis it is the
  *capacity*, not the live length.
- **stride** — a per-axis signed element offset step; `0` = broadcast, `< 0` =
  reversed/flipped.
- **alignment** — base-pointer alignment in bytes (unsigned 32-bit).
- **layout_tag** — the derived per-operand memory-layout class: `contiguous`,
  `inner-contiguous`, `strided`, or `broadcast`.
- **op_family_tag (op category)** — the coarse category a cell participates in
  (elementwise-unary/binary/ternary, reduction, scan, contraction, normalization,
  gather/scatter, …). A component of `structure_key`; **distinct from the KISS-Ops
  op name**. Supplied by the caller as part of the derivation input (§6.6-0012).
- **structure_key** — the admissibility predicate identifying one specialization
  cell (§6.6). Carried opaquely across the announce seam as a length-delimited
  token. **Not** the op's semantic identity.
- **specialization cell** — one layout/dtype/target class a kernel is built for; a
  `structure_key` names exactly one.
- **admissibility predicate** — a total function from an invocation's derivation
  inputs (§6.6-0012) — the canonically-ordered operand descriptors including the
  cell-level op category, the target, and the op category's role hints — to a
  `structure_key`; a kernel admits the invocation iff the derived key byte-matches
  the kernel's key.
- **iteration frame** — the common axis frame of an invocation: `rank` = the widest
  operand rank, with lower-rank operands right-aligned into it (§6.6-0013).
- **role hint** — a caller-supplied fact an op category needs that is not derivable
  from bare operand extents: the M/N/K axis roles of a contraction cell (§6.6-0016)
  and the index-operand slot of a gather/scatter/embedding cell (mirroring the
  KISS-Ops index/address operand role, §6.1-0006; the index is not identified by
  dtype).
- **target_capability** — the namespaced, all-hardware compilation-target
  descriptor `<namespace>:<capability-set>` (§6.8), matched byte-exact.
- **namespace** — the registered left component of a `target_capability` token
  (e.g. `cuda`, `vulkan`, `rocm`, `metal`); registered by the steward.
- **capability-set** — the right component of a `target_capability` token (e.g.
  `sm89`, `gfx942`, `apple9`); owned by the namespace's maintainer.
- **MAX_RANK / MAX_OPERANDS** — the pinned constants bounding operand rank and the
  per-cell operand count (reference values `8` and `8`).
- **token** — the stable string serialization of a `structure_key` (§6.7); the sole
  normative wire form (§6.7-0011), carried opaquely by KISS-Announce.
- **typed decline** — a caller-observable error indication an implementation returns
  in place of a result (a distinguished error value or enumerant where the language
  has one, or an equivalent out-of-band error return elsewhere) that leaves no
  partial output and does **not** panic, abort, crash, hang, or read out of bounds.
  The language-neutral obligation is to return an error indication rather than
  terminating.
- **Implementation** — any software that computes, serializes, or parses a
  `structure_key`, an operand descriptor, or a target_capability token per this
  sub-standard.
- **MBZ** — Must Be Zero (a reserved field a producer zeroes and a reader rejects
  if nonzero). At this schema version no fixed-width binary wire encoding of this
  vocabulary is defined (§6.7-0011), so MBZ is currently inapplicable to interchange.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point storage formats and special values pinned by
  §6.1 / §6.2 (binary16, binary32, binary64; the FP8 E4M3 / E5M2 conventions are
  pinned here directly).
- **KISS Umbrella Specification** — the shared conventions: the RFC-2119 keyword
  convention, the normative/informative split, the clause-ID scheme and 1:1 test
  mapping, value pinning (bits / IEEE-754, endianness fixed), the two version axes,
  the freeze gate (umbrella §5.3), the capability/profile/extension model,
  governance, licensing, and patent posture. **Stated once in the umbrella;
  referenced here; never restated.**
- **KISS-Ops** — the computation vocabulary. **KISS-Classify does NOT depend on
  KISS-Ops, and KISS-Ops does NOT depend on KISS-Classify** — the two are
  foundational sibling roots with **no dependency edge in either direction** (§6.9).
  The dtype tokens, the operand-descriptor field names, `structure_key` /
  `target_capability`, and the pinned constants (`MAX_RANK`, `MAX_OPERANDS`, the
  `structure_key` schema version and token bound) are a **shared naming convention
  spelled identically in both sibling vocabularies** and used **by name only**;
  neither foundational vocabulary consumes or re-derives the other's definitions.
  (Classify's op-category enum and the KISS-Ops per-op family taxonomy are separate,
  Classify-owned and Ops-owned respectively; neither imports the other.) The
  value-conversion and result-normalization obligations that Classify factors out
  (e.g. bool 0/1 normalization, e4m3fn/e5m2 saturating conversion) are owned by
  KISS-Ops / KISS-Emit.
- **KISS-Grammar** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**:
  re-bases advertisable ops onto the dtype set and operand vocabulary defined here.
- **KISS-Contract** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**:
  the contract identity/interface sections describe operands with this vocabulary
  and use `structure_key` as the accept-predicate.
- **KISS-Announce** (by version) — DAG edge labeled **OPAQUE**, **downstream**:
  carries `structure_key` as an uninterpreted, length-delimited token (max 4096
  bytes) and never parses its internals.
- **KISS-Consume** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**:
  lift targets describe their operands with this vocabulary.
- **KISS-Emit** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**: the
  emitter's normative input is an op definition paired with a `structure_key`.
- **External quantization-token registry** (FDX/DLPack quant family, by version) —
  the external source of truth for the quant-family and scale-placement codes
  mirrored by the optional `quant` facts (§6.3). KISS pins only the projection, not
  the external vocabulary's semantics; `family` and `scale_placement` are opaque to
  Classify (§6.3-0009).

---

## 5. Conventions

This sub-standard adopts the umbrella's keyword convention and clause-ID rules
verbatim (umbrella §3). Per the umbrella: normative §6+ uses **only** uppercase
`MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance and
consumer-behavior guidance and never state a byte-level fact. Every atomic
requirement carries a stable, append-only ID `KISS-CLASSIFY-<section>-<nnnn>`,
allocated by the editor of record, never reused after retirement, and mapped 1:1 to
≥1 named KISS-Conform test. Values are pinned as bits / IEEE-754 semantics with
endianness fixed, never as one source language's surface spelling. Unquantified
adjectives ("well-formed", "reasonable", "neutral", "valid", "useful") are banned
from normative text. See the umbrella for the full statement; it is not restated
here.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

- **KISS-CLASSIFY-6.0-0001** — Every dtype token, bit-width, packing convention,
  operand-descriptor field, `structure_key` field, `structure_key` token byte, and
  `target_capability` token in §6–§8 is determinism-class **exact byte compare**;
  KISS-Conform MUST evaluate each such clause with a byte-exact comparator and MUST
  NOT apply tolerance or order-invariant comparison. *Test:*
  `test_classify_determinism_class_exact_byte`.
- **KISS-CLASSIFY-6.0-0002** — Each dtype's IEEE-754 special-value bit patterns —
  positive and negative zero, subnormals, quiet NaN and (where the format defines
  it) signaling NaN, and, for the formats that define them, positive and negative
  infinity — MUST be pinned format constants that an implementation reproduces
  byte-for-byte; a clause bearing such a bit pattern is determinism-class **exact
  byte compare**. This is a format-constant obligation, not a value-conversion
  obligation (Classify encodes dtype tokens, not dtype values). *Test:*
  `test_classify_special_value_bit_patterns`.

### 6.1 The pinned scalar dtype set

The scalar dtype set is the following closed table. Each row pins a dtype's stable
token, numeric kind, and storage bit width; the packing/semantic note is normative
where it fixes storage bytes.

| Token | Kind | Storage bits | Pinned storage / packing |
|---|---|---|---|
| `f16` | float | 16 | IEEE-754 binary16 (sign 1, exp 5, mantissa 10) |
| `bf16` | float | 16 | bfloat16 (sign 1, exp 8, mantissa 7) |
| `f32` | float | 32 | IEEE-754 binary32 storage; compute precision is **not** pinned here — it is a KISS-Ops fidelity attribute (see §6.1-0005) |
| `f64` | float | 64 | IEEE-754 binary64 |
| `i8` | int | 8 | signed 8-bit two's-complement (sk3 `s8`) |
| `i16` | int | 16 | signed 16-bit two's-complement (sk3 `s16`) |
| `u8` | uint | 8 | unsigned 8-bit; physical storage of `bool` |
| `u16` | uint | 16 | unsigned 16-bit |
| `i32` | int | 32 | signed 32-bit two's-complement |
| `i64` | int | 64 | signed 64-bit two's-complement |
| `u32` | uint | 32 | ordinary unsigned 32-bit storage; container width 4 bytes (matches `i32`); index/address is an operand role owned by KISS-Ops, not a dtype class |
| `u64` | uint | 64 | unsigned 64-bit |
| `bool` | bool | 8 | 1-byte truth value; storage width equals `u8` |
| `f8e4m3fn` | float | 8 | FP8 E4M3 OCP finite (sign 1, exp 4, mantissa 3, bias 7); max finite ±448; no infinities; single NaN encoding (OCP OFP8, §6.1-0010). sk3 `e4m3fn` + the `f8` width prefix (§3.1.2) |
| `f8e4m3fnuz` | float | 8 | FP8 E4M3 AMD `fnuz` variant (bias 8, no −0, no infinities); byte-incompatible with `f8e4m3fn`; **reserved** (recognized on parse; use typed-declines at this schema version). sk3 `e4m3fnuz` + `f8` prefix |
| `f8e5m2` | float | 8 | FP8 E5M2 IEEE-style (sign 1, exp 5, mantissa 2, bias 15); max finite ±57344; IEEE-style inf/NaN (OCP OFP8, §6.1-0011). sk3 `e5m2` + `f8` prefix; carries no variant suffix (only `fnuz` deviates from IEEE E5M2, §3.1.5) |
| `f8e5m2fnuz` | float | 8 | FP8 E5M2 AMD `fnuz` variant (bias 16, no −0, no infinities); byte-incompatible with `f8e5m2`; **reserved** (recognized on parse; use typed-declines at this schema version). sk3 `e5m2fnuz` + `f8` prefix |
| `f8e8m0` | float | 8 | MX shared-exponent **scale** (unsigned: 0 sign, 8 exp, 0 mantissa); all-exponent, no mantissa; OCP Microscaling (MX), §6.1-0013. A scale type — the per-block shared scale of an MX-encoded operand, carried as a **sibling operand**, not an element value dtype (§3.2). New at sk4 (additive) |
| `f8e6m2` | float | 8 | MX **scale** (unsigned: 0 sign, 6 exp, 2 mantissa); finer-granularity sibling of `f8e8m0` (+2 mantissa, −2 exponent, less range); OCP Microscaling (MX), §6.1-0013. A scale type, not an element value dtype (§3.2). New at sk4 (additive) |
| `i4` | int | 4 | signed 4-bit `[-8,+7]`; packed-pair byte (low nibble = even index, high nibble = odd index); sign-extended on read (sk3 `s4`) |
| `u4` | uint | 4 | unsigned 4-bit `[0,15]`; packed-pair byte identical to `i4`; zero-extended on read |
| `b1` | uint | 1 | 1-bit; packed-byte (8 bits/byte, LSB = lowest logical index) |
| `c64` | complex | 64 | interleaved (re,im) pair of `f32`; 64 bits **total** (named by total width, §3.1.4). **sk4 meaning-flip:** this is the sk3 `c32`; the token `c64` denoted pair-of-`f64` at sk3 — the version prefix (§3.4) makes the reinterpretation loud, never silent |
| `c128` | complex | 128 | interleaved (re,im) pair of `f64`; 128 bits **total** (named by total width, §3.1.4). This is the sk3 `c64` |

- **KISS-CLASSIFY-6.1-0001** — The scalar dtype set MUST be **exactly** the
  twenty-four tokens in the table above (`f16`, `bf16`, `f32`, `f64`, `i8`, `i16`, `u8`,
  `u16`, `i32`, `i64`, `u32`, `u64`, `bool`, `f8e4m3fn`, `f8e4m3fnuz`, `f8e5m2`, `f8e5m2fnuz`,
  `f8e8m0`, `f8e6m2`, `i4`, `u4`, `b1`, `c64`, `c128`); an implementation MUST NOT recognize a
  twenty-fifth dtype token at this schema version and MUST NOT omit any of the twenty-four. The FP8
  spellings are **width-prefixed and variant-explicit** (sk4): the `f8` width prefix (§3.1.2) plus,
  where a layout admits multiple variants, a mandatory variant suffix (§3.1.5) — `f8e4m3fn` (OCP)
  with `f8e4m3fnuz` reserved, and `f8e5m2` (IEEE, no suffix) with `f8e5m2fnuz` reserved
  — byte-incompatible hardware variants MUST NOT share a token. The `f8e8m0`/`f8e6m2` **scale**
  types are new at sk4 (additive; MX shared-exponent scales, §3.2), carried as sibling operands,
  not element value dtypes. The complex tokens are named by **total** width (§3.1.4): `c64` =
  pair-of-`f32` (the sk3 `c32`), `c128` = pair-of-`f64` (the sk3 `c64`); the version prefix (§3.4)
  makes this reinterpretation loud, never silent. In particular, no
  strict-precision float variant is a dtype: the dtype set is **pure storage**
  (§6.1-0005). A **reserved** spelling (`f8e4m3fnuz`, `f8e5m2fnuz`) is part of the
  closed vocabulary — a reader MUST recognize it and distinguish it from an
  unknown token — but has **no computation semantics at this schema version**: a
  `structure_key` using a reserved dtype in **any** dtype position MUST be
  answered with a typed decline (§6.7-0009) distinct from the unknown-token
  decline (decline vectors: `reserved_fnuz_dtypes_typed_decline`). Activating a
  reserved spelling is a future additive schema event.
  *Test:* `test_classify_dtype_set_is_closed`.
- **KISS-CLASSIFY-6.1-0002** — Each dtype MUST have the exact storage bit width in
  the table above (`f16`/`bf16` = 16; `f32` = 32; `f64` = 64; `i8` = 8;
  `i16` = 16; `u8` = 8; `u16` = 16; `i32` = 32; `i64` = 64; `u32` = 32;
  `u64` = 64; `bool` = 8; `f8e4m3fn`/`f8e4m3fnuz`/`f8e5m2`/`f8e5m2fnuz` = 8;
  `f8e8m0`/`f8e6m2` = 8; `i4`/`u4` = 4; `b1` = 1; `c64` = 64; `c128` = 128). *Test:*
  `test_classify_dtype_bit_widths`.
- **KISS-CLASSIFY-6.1-0003** — Each dtype MUST have the exact numeric kind in the
  table above (`float`: `f16`, `bf16`, `f32`, `f64`, `f8e4m3fn`, `f8e4m3fnuz`, `f8e5m2`,
  `f8e5m2fnuz`, `f8e8m0`, `f8e6m2`; `int`: `i8`, `i16`, `i32`, `i64`, `i4`; `uint`: `u8`, `u16`, `u32`, `u64`, `u4`, `b1`;
  `bool`: `bool`; `complex`: `c64`, `c128`). The MX scales `f8e8m0`/`f8e6m2` are kind `float`
  (unsigned exponent-scales; the unsigned property is a packing fact of §6.1-0013, not a
  distinct kind). *Test:* `test_classify_dtype_numeric_kinds`.
- **KISS-CLASSIFY-6.1-0004** — Each dtype MUST be spelled by exactly its stable
  lowercase token in the table above wherever it appears in a `structure_key` token
  or an operand descriptor; an implementation MUST NOT substitute a synonym or an
  alternate casing. *Test:* `test_classify_dtype_token_spelling`.
- **KISS-CLASSIFY-6.1-0005** — The dtype set MUST be **pure storage**: a dtype pins
  byte layout only (width, numeric kind, spelling, packing) and MUST NOT encode any
  compute-precision or numeric-fidelity guarantee. In particular, `f32` MUST be a
  single IEEE-754 binary32 **storage** dtype, and a strict-precision (bit-stable,
  full-precision multiply-add) float variant MUST NOT exist as a distinct dtype
  token; equivalently, the closed twenty-two-token set (§6.1-0001) contains no such
  token and the dtype record carries no precision field. Compute precision — whether
  a computation must be bit-stable full-precision or may use a reduced-mantissa
  reduction — is a **KISS-Ops fidelity attribute** (a `MathPrecision`-style attribute
  alongside the KISS-Ops-owned determinism/fidelity enum), surfaced in a kernel's
  KISS-Contract guarantees, and MUST NOT be inferred from or attached to a Classify
  dtype token. *Test:* `test_classify_dtypes_are_pure_storage`.
- **KISS-CLASSIFY-6.1-0006** — `u32` MUST be an **ordinary storage dtype**: an
  unsigned 32-bit integer whose container width MUST be 4 bytes, legal in any
  operand slot on the same terms as any other storage dtype. An implementation MUST
  NOT define an index-only dtype class and MUST NOT reject `u32` on any arithmetic,
  reduction, or vectorization path on the ground that it is an index type. The
  index/address **operand role** — which operand of a gather/scatter/embedding cell
  is the index and what its `index_dtype` is — is a KISS-Ops operand-role fact
  carried on the gather/scatter operand, **not** a Classify dtype class. *Test:*
  `test_classify_u32_is_ordinary_storage`.
- **KISS-CLASSIFY-6.1-0007** — `bool` MUST be stored in exactly one byte with the
  pinned truth encoding `0x00` = false and any non-zero byte read as true; its
  canonical (normalized) true value is the byte `0x01`, and its storage width MUST
  equal `u8` (8 bits). (The obligation that an operation normalizes its produced
  `bool` byte to `0x00`/`0x01` is owned by KISS-Ops, not tested here.) *Test:*
  `test_classify_bool_encoding`.
- **KISS-CLASSIFY-6.1-0008** — `i4` and `u4` MUST use the packed-pair byte layout:
  the low nibble holds the even logical index and the high nibble holds the odd
  logical index; `i4` MUST be sign-extended on read and `u4` MUST be zero-extended
  on read. *Test:* `test_classify_sub_byte_nibble_packing`.
- **KISS-CLASSIFY-6.1-0009** — `b1` MUST use the packed-byte layout of 8 bits per
  byte with the least-significant bit holding the lowest logical index. *Test:*
  `test_classify_b1_bit_packing`.
- **KISS-CLASSIFY-6.1-0010** — `f8e4m3fn` (sk3 `e4m3fn`) MUST use the FP8 E4M3 OCP encoding (sign 1,
  exp 4, mantissa 3, bias 7) with maximum finite magnitude 448, no infinity encodings,
  and a single NaN encoding; the reserved `f8e4m3fnuz` variant uses bias 8, defines no
  `−0`, and defines no infinities (byte-incompatible with `f8e4m3fn`) and MUST NOT be
  conflated with it. These are pinned format constants; the saturating
  round-half-to-even conversion *into* `f8e4m3fn` is owned by KISS-Ops / KISS-Emit and
  is not a Classify obligation. A platform component type naming OCP E4M3 (e.g. Vulkan
  `VK_COMPONENT_TYPE_FLOAT8_E4M3_EXT`) denotes `f8e4m3fn`, **not** the reserved
  `f8e4m3fnuz`: `f8e4m3fnuz` carries no computation semantics at this schema version and
  MUST receive a typed decline (§6.1-0001), so a live device reporting the OCP-E4M3
  component type could not denote it without becoming undispatchable — OCP-finite
  `f8e4m3fn` is the only coherent denotation. *Test:* `test_classify_e4m3_format`.
- **KISS-CLASSIFY-6.1-0011** — `f8e5m2` (sk3 `e5m2`) MUST use the FP8 E5M2 IEEE-style encoding (sign
  1, exp 5, mantissa 2, bias 15) with maximum finite magnitude 57344 and IEEE-style
  infinity/NaN encodings; the reserved `f8e5m2fnuz` variant uses bias 16, defines no `−0`,
  and defines no infinities (byte-incompatible with `f8e5m2`) and MUST NOT be conflated
  with it. These are pinned format constants; the saturating round-half-to-even
  conversion *into* `f8e5m2` is owned by KISS-Ops / KISS-Emit and is not a Classify
  obligation. *Test:* `test_classify_e5m2_format`.
- **KISS-CLASSIFY-6.1-0012** — `c64` (sk3 `c32`) MUST be stored as an interleaved
  `(real, imag)` pair of `f32` occupying 64 storage bits, and `c128` (sk3 `c64`) as an
  interleaved `(real, imag)` pair of `f64` occupying 128 storage bits; the real
  component MUST occupy the lower-addressed half. Complex tokens are named by **total**
  width at sk4 (§3.1.4): the token `c64` denotes the 64-bit pair-of-`f32` (the sk3 `c32`),
  and `c128` the 128-bit pair-of-`f64` (the sk3 `c64`); the version prefix (§3.4) makes
  this reinterpretation loud. *Test:* `test_classify_complex_interleaved_layout`.
- **KISS-CLASSIFY-6.1-0013** — The MX scale dtypes `f8e8m0` and `f8e6m2` (both new at
  sk4, additive) MUST use the OCP Microscaling (MX) scale encodings: `f8e8m0` is an
  **unsigned** 8-bit all-exponent scale (0 sign, 8 exp, 0 mantissa); `f8e6m2` is an
  **unsigned** 8-bit scale (0 sign, 6 exp, 2 mantissa), a finer-granularity sibling of
  `f8e8m0`. A scale carries **no sign bit**, so the width self-check is `exp + mantissa`
  (8+0 and 6+2, both 8). Both are **scale types** — the per-block shared scale of an
  MX-encoded value operand, carried as a **sibling operand** (§3.2), never an element
  value dtype. These are pinned format constants citing OCP-MX; their special values
  follow the OCP-MX definitions (not restated here), and the MX **block** structure
  (block size, scale placement) is an encoding-axis concern **outside** §6.1.
  *Sibling placement (informative):* this clause pins no **position** for the scale's
  sibling operand in the canonical operand order (§6.6-0014); placement is left
  unconstrained. §6.6-0019's prohibition on reading the weight role from operand
  position depends on that — if a later revision pins sibling placement, revisit
  §6.6-0019. *Test:* `test_classify_mx_scale_format`.

### 6.2 Numeric-kind and special-value pinning

- **KISS-CLASSIFY-6.2-0001** — The numeric-kind set MUST be exactly `{float, int,
  uint, bool, complex}`; every dtype MUST map to exactly one kind per §6.1-0003, and
  an implementation MUST NOT introduce a sixth kind at this schema version. *Test:*
  `test_classify_numeric_kind_set_closed`.
- **KISS-CLASSIFY-6.2-0002** — For each float dtype (`f16`, `bf16`, `f32`, `f64`,
  `f8e4m3fn`, `f8e4m3fnuz`, `f8e5m2`, `f8e5m2fnuz`) the special values the format defines MUST be
  identified by their pinned bit patterns: `±0` for every float dtype **except** the
  `fnuz` variants (which define no `−0`), and subnormals for every float dtype; positive
  and negative infinity for `f16`/`bf16`/`f32`/`f64`/`f8e5m2` (but **not** `f8e4m3fn`,
  `f8e4m3fnuz`, or `f8e5m2fnuz`, which define none); quiet and signaling NaN for every float
  dtype that defines both (all except the FP8 variants); and the single NaN encoding for
  the FP8 variants (§6.1-0010/-0011). The MX scale floats `f8e8m0`/`f8e6m2` follow the
  OCP-MX special-value definitions (§6.1-0013) and are not restated here. An implementation MUST distinguish `-0` from `+0`
  by bit pattern where the format defines `−0`, and MUST NOT conflate distinct NaN
  encodings when identifying a dtype's special values. *Test:*
  `test_classify_float_special_values_pinned`.

### 6.3 Operand descriptor fields

The operand descriptor is the following fixed field set. `rank`, `extents`,
`strides`, `dtype`, and `alignment` are stored raw; `layout_tag` and `op_family_tag`
are derived/cell-level; `quant` and `symbolic_extent` are optional. The binary types
below (`u8`, `i64`, `u16`) describe a permitted in-memory representation only; the
token (§6.7) is the sole normative wire form (§6.7-0011).

| Field | Type | Range / domain |
|---|---|---|
| `rank` | u8 | `0 ..= MAX_RANK` (§6.4) |
| `extents` | `i64[MAX_RANK]` | any i64; only `extents[0..rank]` meaningful; symbolic-axis entry is the capacity |
| `strides` | `i64[MAX_RANK]` | any signed i64 (`0` = broadcast, `< 0` = reversed); element units; only `strides[0..rank]` meaningful |
| `dtype` | dtype token | one of the twenty-two (§6.1) |
| `alignment` | u32 | any unsigned 32-bit byte count (`0` and non-power-of-two permitted; §6.5-0009 pins the gating) |
| `layout_tag` | enum | `{contiguous, inner-contiguous, strided, broadcast}` (§6.5-0001) |
| `op_family_tag` | enum | one op category (§6.5-0006); cell-level |
| `quant` | optional record | `{family, sub_byte_bits: u8, block_elems: u16, scale_placement, dequant_form}` |
| `symbolic_extent` | optional record | `{axis: u8 (< rank), kind ∈ {scalar, range, affine}}`; `kind` is an uninterpreted tag at this schema version (no bounds payload) |

- **KISS-CLASSIFY-6.3-0001** — An operand descriptor's `rank` MUST be in the
  inclusive range `0 ..= MAX_RANK`; a producer MUST NOT emit and a reader MUST
  reject (with a typed decline, never a panic) a `rank` greater than `MAX_RANK`.
  *Test:* `test_classify_operand_rank_bounds`.
- **KISS-CLASSIFY-6.3-0002** — Only `extents[0..rank]` and `strides[0..rank]` MUST
  be treated as meaningful; a reader MUST NOT read `extents[i]` or `strides[i]` for
  `i >= rank`. *Test:* `test_classify_operand_active_axes_only`.
- **KISS-CLASSIFY-6.3-0003** — `strides` MUST be **signed** 64-bit element strides
  (not byte strides), with `0` denoting a broadcast axis and a negative value
  denoting a reversed/flipped axis; an implementation MUST NOT reinterpret a stride
  as unsigned. *Test:* `test_classify_strides_are_signed_elements`.
- **KISS-CLASSIFY-6.3-0004** — `extents` MUST be 64-bit values in *element* units,
  and for a symbolic axis the stored `extents[axis]` MUST be the axis **capacity**
  (which keys strides and index width), not the live length. *Test:*
  `test_classify_extent_is_capacity`.
- **KISS-CLASSIFY-6.3-0005** — `alignment` MUST be the base-pointer alignment in
  **bytes** as an unsigned 32-bit value; the value `0` and non-power-of-two values
  are permitted, and §6.5-0009 pins how they gate vector width via an
  **exact-modulo** alignment gate (a divisor test, not a power-of-two floor), with
  `alignment = 0` (unspecified base-pointer alignment) forcing `v1`. *Test:*
  `test_classify_alignment_is_bytes`.
- **KISS-CLASSIFY-6.3-0006** — `dtype` MUST be exactly one of the twenty-two tokens
  of §6.1. *Test:* `test_classify_operand_dtype_in_set`.
- **KISS-CLASSIFY-6.3-0007** — `layout_tag` MUST be derived as a projection of
  `extents` and `strides` (§6.5-0002) and MUST NOT be an independently stored raw
  field that can disagree with them. *Test:* `test_classify_layout_tag_is_derived`.
- **KISS-CLASSIFY-6.3-0008** — `op_family_tag` MUST be carried at the **cell**
  level (one per `structure_key`), not per raw operand, and MUST be supplied by the
  caller (it is part of the derivation input, §6.6-0012). *Test:*
  `test_classify_op_family_is_cell_level`.
- **KISS-CLASSIFY-6.3-0009** — When present, the `quant` record MUST carry
  `{family, sub_byte_bits (u8), block_elems (u16), scale_placement, dequant_form}`;
  `sub_byte_bits` and `block_elems` are pinned integer fields, `dequant_form` is the
  closed Classify-owned enumerant of §6.3-0009a, while `family` and `scale_placement`
  are opaque tokens **uninterpreted by Classify** at this schema version (their
  vocabularies are owned by the external quantization-token registry, §4). Every field
  of the `quant` record MUST be carried for binding and MUST NOT
  be folded into the `structure_key` admissibility key at this schema version. *Test:*
  `test_classify_quant_carried_not_keyed`.
- **KISS-CLASSIFY-6.3-0009a** — `dequant_form` MUST be one of the closed set
  `{linear, codebook}`, owned and interpreted by KISS-Classify. `linear` means the stored
  value is recovered by an affine map of the stored integer and its block scale (and
  zero-point, if any); `codebook` means it is recovered by **indexing a table of values
  the stored bits select**. An implementation MUST NOT infer `dequant_form` from
  `family`, which Classify does not interpret, and MUST NOT omit it from a `quant`
  record.
  **Effective at the next `structure_key` schema version**, `dequant_form` — and, of the
  `quant` record, only `dequant_form` — MUST be folded into the `structure_key`
  admissibility key. **At the current schema version the §6.7 token grammar defines no
  slot for it**, so an implementation MUST NOT invent one, and MUST NOT emit a key
  claiming to distinguish dequant form at this version. The exclusion of §6.3-0009 is
  thereby **narrowed, not reversed**: every other `quant` field stays out of the key at
  every version.
  *Rationale — this is the one quant distinction that is not a performance choice.*
  Two symmetric 4-bit schemes over the same storage dtype (e.g. a GPTQ-derived linear
  int4 and a 16-entry non-uniform codebook) agree on shapes, dtypes, layout and
  `sub_byte_bits`, so without this field they produce **one key for two kernels** — and
  **no scale reproduces a codebook with a linear dequant**, so a cache serving one for
  the other returns a **numerically wrong** result rather than a slower one. Quant
  distinctions that ARE inter-substitutable stay out of the key, per §6.3-0009: this
  clause prices only the distinction where a wrong choice is wrong rather than slow.
  *Test:* `test_classify_dequant_form_keyed`.
- **KISS-CLASSIFY-6.3-0010** — When present, the `symbolic_extent` record MUST
  carry `{axis (u8, `< rank`), kind ∈ {scalar, range, affine}}`. At this schema
  version `kind` is an **uninterpreted tag** carrying no bounds payload: the record
  stores neither a scalar bound, nor a `[min, max]` interval, nor affine
  coefficients/parameter references. The informative distinction — `scalar` denoting
  a single dynamic length bound, `range` a `[min, max]` length interval, and `affine`
  a length given by an affine form over symbolic parameters — describes intended
  future semantics only and MUST NOT be relied upon to carry bounds at this version.
  The record MUST flag that the named axis's *live* length is dynamic while its
  *capacity* (`extents[axis]`) keys the strides, and the symbolic fact MUST NOT
  change how strides or index width are keyed. *Test:*
  `test_classify_symbolic_extent_flags_live_length`.
- **KISS-CLASSIFY-6.3-0011** — Axes MUST be ordered **outermost first**: axis index
  `0` is the outermost axis and axis index `rank−1` is the innermost axis. Every
  derivation that references an "innermost" axis (§6.5-0002 layout, §6.5-0009 vector
  width, §6.5-0012 divisibility) MUST use the highest active axis index as the
  innermost axis. *Test:* `test_classify_axis_ordering_convention`.

### 6.4 Pinned structural constants

- **KISS-CLASSIFY-6.4-0001** — `MAX_RANK` MUST equal `8`. Every per-axis array
  (`extents`, `strides`) MUST be bounded by `MAX_RANK`. *Test:*
  `test_classify_max_rank_is_8`.
- **KISS-CLASSIFY-6.4-0002** — `MAX_OPERANDS` MUST equal `8`. A `structure_key`
  MUST carry no more than `MAX_OPERANDS` per-operand sub-keys; a producer MUST NOT
  emit and a reader MUST reject a key declaring more than `MAX_OPERANDS`. *Test:*
  `test_classify_max_operands_is_8`.
- **KISS-CLASSIFY-6.4-0003** — The `structure_key` schema version
  (`STRUCTURE_KEY_VERSION`) MUST be the integer `4` at this maturity, encoded as the
  token prefix `sk4` (§6.7-0002); a bump of this integer is required only when a
  predicate axis is added or altered in a non-additive way. (Version `4` supersedes
  version `3`: the sk4 dtype respellings — integer `i`-prefix, FP8 `f8`-prefix + mandatory
  variant suffix, and the complex total-width meaning-flip (§6.1, §3.1) — together with the
  new non-contraction precision coordinate `(acc + mp)` (§6.7-0012, §3.3), forced this
  non-additive bump while the sub-standard is UNFROZEN. Version `3` had superseded version
  `2` for the `gem` contraction field's non-additive growth to carry the precision/compute
  coordinate set — weight/accumulator/output dtypes, the conditionally-present batch
  size-class, and the math-precision code `<mp>` (§6.7-0006); version `2` had superseded
  version `1` for the reduce field's non-additive split, §6.6-0009.) *Test:*
  `test_classify_structure_key_version_matches_codec`.
- **KISS-CLASSIFY-6.4-0004** — The `structure_key` token maximum length MUST be
  `4096` bytes (`MAX_STRUCTURE_KEY_LEN = 4096`); a producer MUST NOT emit a token
  longer than 4096 bytes and a reader MUST reject a token whose length is `0` or
  greater than `4096` with a typed decline, without allocating on the unchecked
  length. *Test:* `test_classify_structure_key_token_length_bound`.

### 6.5 Per-operand sub-key and cell-level enumerations

The `structure_key` per-operand sub-key and its cell-level axes draw from these
closed enumerations.

**Layout tag (contiguity).** `{contiguous, inner-contiguous, strided, broadcast}`,
token codes `co` / `ic` / `st` / `br`.
**Vector-access width.** `{scalar, v2, v4, v8}`, token codes `v1` / `v2` / `v4` /
`v8`.
**Inner-extent divisibility bucket.** `{div16, div8, div4, div2, any}`, token codes
`d16` / `d8` / `d4` / `d2` / `da`.
**Index width.** `{idx32, idx64}`, token codes `ix32` / `ix64` (deliberately
distinct from the `i32` / `i64` **dtype** tokens of §6.1 so no token field shares a
spelling across the orthogonal dtype and index-width axes); the boundary is `2³¹`
elements — a maximum touched element offset `< 2³¹` is `idx32`, otherwise `idx64`.
**Work class.** `{one-warp, one-block, grid-stride}`, token codes `warp` / `block`
/ `grid`; `one-warp` iff total element count `≤ 32`, `one-block` iff `≤ 1024`,
`grid-stride` otherwise.
**Contraction size class.** `{tiny, small, mid, large}`, token codes `t` / `s` /
`m` / `l`; `tiny` iff extent `≤ 8`, `small` iff `9..=128`, `mid` iff `129..=2048`,
`large` iff `> 2048`.
**Op-family tag (op category).** The closed set of coarse categories, each with a
3-letter token code:

| Category | Code | Category | Code |
|---|---|---|---|
| contraction (dense GEMM) | `gem` | indexing (gather/scatter) | `idx` |
| elementwise-unary | `une` | embedding | `emb` |
| elementwise-binary | `bin` | shape/layout | `shp` |
| elementwise-ternary | `ter` | sorting | `srt` |
| gated-activation | `gat` | quantization | `qnt` |
| reduction | `red` | random | `rnd` |
| scan | `scn` | loss | `los` |
| normalization | `nrm` | segment-ops | `seg` |
| softmax | `sft` | image | `img` |
| convolution | `cnv` | fft | `fft` |
| pooling | `pol` | linalg | `lin` |
| attention | `att` | mixture-of-experts | `moe` |

- **KISS-CLASSIFY-6.5-0001** — The `layout_tag` domain MUST be exactly
  `{contiguous, inner-contiguous, strided, broadcast}` with token codes `co`, `ic`,
  `st`, `br`. *Test:* `test_classify_layout_tag_enum`.
- **KISS-CLASSIFY-6.5-0002** — `layout_tag` MUST be derived from `extents` and
  `strides` using `|stride|` (absolute value — so a fully reversed view is
  `contiguous` with the reversal captured only by the flipped flag, §6.6-0007) by the
  following pinned algorithm. Consider only the **active non-unit axes** (active axes
  §6.3-0002 whose extent is neither `0` nor `1`), visited **innermost (§6.3-0011)
  first**. **(1)** The operand is `broadcast` if any axis of extent > 1 has stride
  `0`. **(2)** Else it is `contiguous` if, maintaining a running product `P`
  initialized to `1` and visiting the active non-unit axes innermost→outermost,
  `|stride|` for each such axis equals `P` immediately before `P` is multiplied by
  that axis's extent (unit axes and zero-stride are excluded from the iteration, so
  the product ranges over the active non-unit axes only). **(3)** Else it is
  `inner-contiguous` if the innermost active non-unit axis has `|stride| == 1`.
  **(4)** Else it is `strided`. An operand with no active axis of extent > 1 (every
  active axis of extent `0` or `1`) MUST be classified `contiguous` (the product is
  empty). An active axis of extent `0` is not a non-unit axis and MUST be excluded
  from the product exactly as a unit axis is. *Test:*
  `test_classify_layout_tag_derivation`.
- **KISS-CLASSIFY-6.5-0003** — The vector-access-width domain MUST be exactly
  `{scalar, v2, v4, v8}` with token codes `v1`, `v2`, `v4`, `v8`. *Test:*
  `test_classify_vec_width_enum`.
- **KISS-CLASSIFY-6.5-0004** — The inner-extent divisibility-bucket domain MUST be
  exactly `{div16, div8, div4, div2, any}` with token codes `d16`, `d8`, `d4`,
  `d2`, `da`, where `d16` selects extents divisible by 16, `d8` extents divisible by
  8 but not 16, `d4` extents divisible by 4 but not 8, `d2` extents divisible by 2
  but not 4, and `da` the remaining (odd, unit, or zero) extents; the derivation is
  §6.5-0012. *Test:* `test_classify_div_bucket_enum`.
- **KISS-CLASSIFY-6.5-0005** — The index-width domain MUST be exactly `{idx32,
  idx64}` with token codes `ix32`, `ix64`, and the classification boundary MUST be
  `2³¹` elements of maximum touched element offset (offsets `< 2³¹` ⇒ `idx32`); the
  offset is computed per §6.5-0011. The index-width token codes `ix32` / `ix64` are
  deliberately distinct from the integer **dtype** tokens `i32` / `i64` (§6.1): the
  index-width field (token field 4, §6.7-0003) and the dtype field (token field 2)
  are orthogonal axes, and the §6.1-0004 dtype-spelling rule applies **only** to the
  dtype field and the per-operand dtype positions, never to the index-width field.
  *Test:* `test_classify_index_width_boundary`.
- **KISS-CLASSIFY-6.5-0006** — The op-family-tag domain MUST be exactly the
  twenty-four categories of the table above, each spelled by its 3-letter token
  code; an implementation MUST NOT invent a twenty-fifth code at this schema
  version, and MUST fail (not silently encode as an "unknown" code) when it cannot
  map a cell to one of the twenty-four. *Test:* `test_classify_op_family_enum`.
- **KISS-CLASSIFY-6.5-0007** — The work-class domain MUST be exactly `{one-warp,
  one-block, grid-stride}` with token codes `warp`, `block`, `grid`, and the
  boundaries MUST be total element count `≤ 32` (`one-warp`), `≤ 1024`
  (`one-block`), otherwise `grid-stride`; the count is computed per §6.5-0010.
  *Test:* `test_classify_work_class_enum`.
- **KISS-CLASSIFY-6.5-0008** — The contraction size-class domain MUST be exactly
  `{tiny, small, mid, large}` with token codes `t`, `s`, `m`, `l` and boundaries
  `≤ 8`, `9..=128`, `129..=2048`, `> 2048`; a `structure_key` MUST key size
  **classes**, never literal extents. *Test:* `test_classify_size_class_enum`.
- **KISS-CLASSIFY-6.5-0009** — The vector-access width of an operand MUST be
  derived as: **(a)** `v1` if the operand's `layout_tag` is `broadcast`; **(b)**
  `v1` if the operand's innermost active axis (§6.3-0011) is a reduced axis of a
  reduction cell — i.e. the cell's reduce field (§6.6-0009) is `rall`, or is
  `rlast`, or is an `x<hh>` bitmask whose innermost-axis bit is set — or the cell's
  op category is scan (`scn`); **(c)** otherwise — provided the innermost active
  axis is **forward-unit-stride** per §6.5-0013, else `v1` — the token `vL` for the
  largest `L ∈ {8, 4, 2, 1}` such that `L · (dtype storage bytes) ≤ 16` (the
  vector-access byte cap), `L` divides the innermost active axis extent, and
  `alignment mod (L · dtype storage bytes) = 0` — an **exact-modulo** alignment
  gate that tests whether the base address is genuinely `L · bytes`-aligned, **not** a
  `floor(alignment / dtype storage bytes)`-to-a-power-of-two shortcut (which over-selects,
  because it ignores whether the address actually admits the wider load). The two disagree
  when the address is not `L · bytes`-aligned: e.g. `alignment = 24`, `f16` derives `v4`
  (`24 mod (4·2) = 0`, but `24 mod (8·2) = 8 ≠ 0`, so an 8-wide load would be misaligned),
  whereas the shortcut `floor(24 / 2) = 12 → 8` would wrongly pick `v8`. (The earlier
  `alignment = 48`, `f32` example could not show this: for `f32` the 16-byte load cap
  already limits the result to `v4`, so the over-selecting shortcut is capped back to `v4`
  and never diverged from the exact-modulo gate there.)
  An operand with `alignment = 0` (unspecified base-pointer
  alignment) cannot honor a packed load and MUST derive `v1`. A sub-byte dtype
  (`i4`, `u4`, `b1`), whose storage is under one byte, MUST derive `v1`. An
  innermost active axis of extent `E = 0` MUST derive `v1`: the divisibility test
  is **vacuously** satisfied at `E = 0` (every `L` divides `0`), which would
  otherwise elect the largest `L` the byte cap and alignment allow — but a
  zero-length run has nothing to load, and the empty case MUST NOT be read as
  maximally vectorizable. This is the reading consistent with §6.5-0012, which
  buckets `E = 0` as `da`. *Test:* `test_classify_vec_width_derivation`.
- **KISS-CLASSIFY-6.5-0010** — The work-class **total element count** MUST be the
  product, over the active axes of the iteration frame (§6.6-0006, §6.6-0013), of
  each axis's iteration-frame extent (the maximum extent across operands at that
  axis); the boundaries of §6.5-0007 MUST be applied to this count. *Test:*
  `test_classify_work_class_element_count`.
- **KISS-CLASSIFY-6.5-0011** — The **maximum touched element offset** MUST be the
  maximum, over all operands, of `Σ |strides[a]| · (extents[a] − 1)` taken over that
  operand's active non-broadcast axes (a broadcast axis, stride `0`, contributes
  `0`), in element units; the boundary of §6.5-0005 (`2³¹`) MUST be applied to this
  maximum. *Test:* `test_classify_index_width_offset`.
- **KISS-CLASSIFY-6.5-0012** — The inner-extent divisibility bucket MUST be computed
  from the innermost active axis (§6.3-0011) extent `E`, measured in **elements**:
  `d16` iff `E ≥ 16` and `E mod 16 = 0`; else `d8` iff `E ≥ 8` and `E mod 8 = 0`;
  else `d4` iff `E ≥ 4` and `E mod 4 = 0`; else `d2` iff `E ≥ 2` and `E mod 2 = 0`;
  else `da` (covering odd `E`, `E = 1`, and `E = 0`). *Test:*
  `test_classify_div_bucket_derivation`.
- **KISS-CLASSIFY-6.5-0013** — A vector-access width `vL` with `L > 1`
  (§6.5-0009(c)) MUST additionally require that the operand's **innermost active
  axis** (§6.3-0011) is **forward-unit-stride** — its signed stride (§6.3-0003) is
  exactly `+1` — and that **no** axis of the operand broadcasts. An innermost axis
  whose stride is `-1` (a **reversed** run), or whose `|stride| > 1` (a transposed /
  non-inner-contiguous axis), MUST derive `v1`, because its elements are not a
  contiguous forward run in memory and no conformant packed / vector load (`ld.128`,
  a SPIR-V vector load, a CPU SIMD load) can honor `L > 1` for them. Only a
  forward-unit stride qualifies; a reversed run derives `v1` even though
  `|stride| = 1`. The test is the **innermost** axis's signed stride, and **not**
  the §6.6-0007 flipped flag: that flag is set iff *any* axis of the operand has a
  negative stride, so an operand whose outer axis is reversed but whose innermost
  active axis is forward-unit-stride derives `vL` with the flag set. This
  precondition gates §6.5-0009(c) before its byte-cap, extent-divisibility, and
  alignment tests are applied. *Test:* `test_classify_vec_width_unit_stride`.

### 6.6 The `structure_key` admissibility predicate

`structure_key` is an admissibility predicate over a specialization cell. Its field
layout is the following ordered set. The binary field types (`u16`, `u8`) describe a
permitted in-memory representation only; the token (§6.7) is the sole normative wire
form (§6.7-0011).

| Field | Type | Meaning |
|---|---|---|
| `version` | u16 (`= 4`) | schema version; extra fields append so old tokens stay byte-identical |
| `op_family` | op category (§6.5-0006) | the coarse op category — **NOT** the semantic op name |
| `dtype` | dtype token | operand-0 / primary element dtype |
| `target` | target_capability (§6.8) | the namespaced compilation-target descriptor |
| `index_width` | `{idx32, idx64}` | offset-arithmetic width (boundary `2³¹`) |
| `work_class` | `{one-warp, one-block, grid-stride}` | total-work size class |
| `rank` | u8 | iteration rank = the widest operand rank |
| `n_operands` | u8 (`≤ MAX_OPERANDS`) | count of populated per-operand sub-keys |
| `operands[]` | sub-key array | per-operand: `{layout_tag, broadcast-axis mask, vector-access width, inner-extent divisibility bucket, flipped flag}` |
| `reduce_axes` | tagged reduce spec | one of three distinct values — none/not-a-reduction, all-axes, trailing-axis — or an explicit keepdim axis bitmask (u8) for any other set (§6.6-0009) |
| `contraction` | optional | `{M, N, K size classes, K divisibility bucket, conditionally-present batch size class, weight dtype, accumulator dtype, output dtype, math-precision}` (sk3, §6.7-0006); absent for non-contraction cells |

- **KISS-CLASSIFY-6.6-0001** — A `structure_key` MUST be an **admissibility
  predicate**: a kernel MUST admit an invocation **if and only if** the
  `structure_key` derived from that invocation's derivation inputs (§6.6-0012)
  byte-matches the kernel's key. An implementation MUST NOT admit an invocation
  whose derived key differs by any byte. *Test:*
  `test_classify_structure_key_is_admissibility_predicate`.
- **KISS-CLASSIFY-6.6-0002** — A `structure_key` MUST NOT encode the op's semantic
  identity; its `op_family` field MUST be a coarse op **category**, never a KISS-Ops
  op name. Op-semantic identity is owned by KISS-Ops / KISS-Contract, and a
  consumer MUST match **both** the cell (this key) and the op identity (KISS-Ops) to
  establish kernel identity. *Test:*
  `test_classify_structure_key_is_not_op_identity`.
- **KISS-CLASSIFY-6.6-0003** — A `structure_key` MUST be **extent-free**: it MUST
  key size *classes* (`work_class`, contraction size classes) and structural facts,
  and MUST NOT carry any literal extent. *Test:*
  `test_classify_structure_key_extent_free`.
- **KISS-CLASSIFY-6.6-0004** — The `structure_key` fields MUST appear in exactly
  the order and with exactly the types of the table above; `version` MUST be the
  first field and MUST equal `4` (§6.4-0003, §6.7-0002). *Test:*
  `test_classify_structure_key_field_layout`.
- **KISS-CLASSIFY-6.6-0005** — `structure_key.dtype` MUST be operand-0's (the
  primary operand's) dtype, where operand-0 is fixed by the canonical operand
  ordering of §6.6-0014. *Test:* `test_classify_structure_key_primary_dtype`.
- **KISS-CLASSIFY-6.6-0006** — `structure_key.rank` MUST be the widest operand rank
  (the iteration rank, §6.6-0013), and `n_operands` MUST be the count of populated
  per-operand sub-keys, `≤ MAX_OPERANDS`. The codec (§6.7-0004) MUST serialize
  exactly `n_operands` sub-keys, so no unused sub-key slot is observable. *Test:*
  `test_classify_structure_key_rank_and_operand_count`.
- **KISS-CLASSIFY-6.6-0007** — Each per-operand sub-key MUST carry exactly
  `{layout_tag, broadcast-axis mask, vector-access width, inner-extent divisibility
  bucket, flipped flag}`, where the flipped flag MUST be set iff any axis of that
  operand has a negative stride. *Test:* `test_classify_operand_sub_key_fields`.
- **KISS-CLASSIFY-6.6-0008** — The broadcast-axis mask MUST be a bitmask over
  iteration-frame axes (§6.6-0013), bit `i` denoting iteration-frame axis `i`,
  bounded by `MAX_RANK` so a single byte suffices, with bit `i` set iff
  iteration-frame axis `i` has extent > 1 and that operand's stride along it is `0`
  (the operand broadcasts along that axis). *Test:*
  `test_classify_broadcast_axis_mask`.
- **KISS-CLASSIFY-6.6-0009** — The reduce spec MUST be exactly one of four
  **distinctly-encoded** values, and an implementation MUST NOT overload a single
  sentinel across two of them: **(1) none / not-a-reduction** (token field `-`,
  §6.7-0005), meaning the cell is not a reduction; **(2) all-axes reduction** (token
  field `rall`), meaning every iteration-frame axis (§6.6-0013) is reduced; **(3)
  trailing-axis reduction** (token field `rlast`), meaning exactly the single
  innermost active axis (§6.3-0011) is reduced; **(4) an explicit keepdim axis
  bitmask** over iteration-frame axes (token field `x<hh>`, §6.7-0005) for any
  reduced-axis set that is neither all-axes nor the lone trailing axis, with bit `d`
  set iff iteration-frame axis `d` is reduced. The `rall` and `rlast` **field
  encodings** are rank-independent (the same field value is used for any rank whose
  reduced set matches), but the overall key still varies with the `rank` field, so a
  reduction key at one rank never byte-matches one at another rank. A reduction cell
  MUST be presented in **keepdim** form: each reduced axis `d` is folded to the
  corresponding size-1 output axis. To keep the derivation canonical (§6.6-0011),
  when the reduced set is exactly all iteration-frame axes the reduce spec MUST be
  encoded as `rall` (never the equivalent bitmask), and when it is exactly the
  innermost axis alone it MUST be encoded as `rlast` (never the equivalent bitmask);
  the `x<hh>` form MUST NOT be used for those two cases. **When both antecedents hold
  — the reduced set is simultaneously all iteration-frame axes and the lone innermost
  axis, which occurs for every rank-1 reduction (reducing the single axis of a 1-D
  iteration frame) — the reduce spec MUST be encoded as `rall`; `rall` takes
  precedence over `rlast` whenever the reduced set is both, so two conforming
  implementations never disagree on the rank-1 encoding.** **The empty reduced set is
  the not-a-reduction case and MUST be spelled `-`, never the all-zero bitmask `x00`;
  consequently a rank-0 (scalar) cell, having no iteration-frame axis to reduce, admits
  only `-`. The canonicalization is thus a total function of `(rank, reduced-set)`:
  the empty set → `-`; the full set `(1<<rank)-1` → `rall`; the lone innermost axis
  `1<<(rank-1)` → `rlast` (with `rall` winning where they coincide, at rank 1); and
  every other in-range set → its `x<hh>` bitmask. A reduced-axis bit at index ≥ `rank`
  names a non-existent axis and is out of range (rejected under §6.7-0005 / §6.7-0009),
  not a reduced-set spelling at all — so the `x<hh>` sentinel-overload decline and the
  out-of-range decline are kept distinct. At rank 0 the same axis splits the two
  **sentinels**: `rall` names the full set of zero axes, which *is* the empty set — a
  real set spelled wrong, declined as a sentinel overload exactly as `x00` is; `rlast`
  names a lone innermost axis that does not exist at rank 0 — a set that cannot exist
  for the rank, declined as out-of-range exactly as a bit at index ≥ `rank` is. The two
  MUST decline distinctly, so a consumer can tell "wrong spelling of the empty set" from
  "a reduction over an axis the rank does not have."** A collapsed (rank-reduced)
  reduction output MUST be rejected with a typed decline (§7.1-0002) rather than
  keyed, so that reductions over different axis sets never collide. *Test:*
  `test_classify_reduce_axes_encoding`.
- **KISS-CLASSIFY-6.6-0010** — A **dense-contraction cell** is defined as a cell
  whose `op_family` is `gem` (§6.5-0006). The `contraction` field MUST be present
  **if and only if** the cell is a dense-contraction cell (`op_family == gem`), and
  MUST then carry `{M, N, K size classes, K divisibility bucket, the
  conditionally-present batch size class, and the precision group: weight dtype,
  accumulator dtype, output dtype, math-precision}` (sk3 grammar
  `c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp>`, §6.7-0006); for every cell
  whose `op_family` is not `gem` the `contraction` field MUST be absent, so the token
  is byte-identical to the base (non-contraction) codec. A reader MUST reject, with a
  typed decline (§7.1-0002), a token that carries the contraction field on a non-`gem`
  cell or omits it on a `gem` cell. *Test:*
  `test_classify_contraction_field_optional`.
- **KISS-CLASSIFY-6.6-0011** — The derivation MUST be **canonical and
  deterministic**: two invocations whose derivation inputs (§6.6-0012) are equal
  MUST derive byte-identical keys, so a provider's build matrix and a consumer's
  runtime lookup join on the same token without shared code beyond the derivation
  function. *Test:* `test_classify_structure_key_derivation_canonical`.
- **KISS-CLASSIFY-6.6-0012** — The `structure_key` derivation function's input MUST
  be: the canonically-ordered operand descriptors (§6.6-0014), including the
  cell-level `op_family_tag` (op category, §6.3-0008); the `target`; and the
  caller-supplied **role hints** required by that op category — for a
  dense-contraction cell the M/N/K axis-role assignment (§6.6-0016) and which operand
  slot carries the **weight role** (§6.6-0019; the weight is not identified by operand
  position or dtype), and for a gather/scatter/embedding cell which operand slot
  carries the index/address role
  (the KISS-Ops operand-role fact of §6.1-0006, supplied to the derivation as a
  caller role hint; the index is **not** identified by dtype). An implementation
  MUST NOT infer the op category or these role hints from bare
  operand extents; they MUST be supplied by the caller. *Test:*
  `test_classify_derivation_input_tuple`.
- **KISS-CLASSIFY-6.6-0013** — When operands differ in rank, each operand's axes
  MUST be **right-aligned** to the iteration rank (§6.6-0006): an operand of rank
  `r` occupies iteration-frame axes `[iteration_rank − r, iteration_rank − 1]`, and
  every iteration-frame axis below `iteration_rank − r` MUST be treated as broadcast
  (stride `0`) for that operand. All per-operand masks (broadcast-axis mask
  §6.6-0008, `reduce_axes` §6.6-0009) MUST be computed in the iteration frame.
  *Test:* `test_classify_mixed_rank_axis_alignment`.
- **KISS-CLASSIFY-6.6-0014** — Operands MUST be presented in canonical order: all
  **input** operands in call order, followed by all **output** operands in call
  order. Operand-0 (the primary operand) MUST be the first operand in this canonical
  order, and the `;`-joined sub-key order (§6.7-0004) MUST follow it. An operand that
  is **both read and written** (an in-place / read-modify-write operand, e.g. an
  accumulator) MUST appear **exactly once** in the canonical order — classified as an
  **input** and placed at its position in the input call order — and MUST NOT also be
  listed among the outputs; it therefore contributes exactly one per-operand sub-key
  and is counted exactly once in `n_operands`. *Test:*
  `test_classify_operand_canonical_order`.
- **KISS-CLASSIFY-6.6-0015** — Outside a dense-contraction (`gem`) cell, a
  `structure_key` keys only the primary (operand-0) dtype (§6.6-0005); per-operand
  sub-keys (§6.6-0007) carry **no** dtype, so two non-`gem` cells that differ only in a
  non-primary operand's dtype derive byte-identical keys. This collision is
  **deliberate** for non-`gem` cells at this schema version: an implementation MUST NOT
  vary a non-`gem` derived key with any non-primary operand's dtype and MUST NOT add a
  per-operand dtype to a sub-key. **For `gem` cells this is superseded** — the sk3
  contraction field (§6.7-0006) carries the weight, accumulator, and output dtype
  coordinates explicitly, so mixed-precision `gem` cells are distinguished in-key and no
  longer collide. (Agreeing secondary-operand dtypes for non-`gem` cells remains the
  caller's responsibility outside the admissibility key; see §8.2 and the registration
  obligation §6.6-0018.) *Test:* `test_classify_secondary_dtype_unkeyed`.
- **KISS-CLASSIFY-6.6-0016** — The M, N, and K axis roles of a dense-contraction
  cell MUST be supplied by the caller as role hints (§6.6-0012); an implementation
  MUST NOT infer M/N/K from bare operand extents. The `contraction` field's M, N, K
  size classes (§6.5-0008) MUST be computed from the caller-assigned M/N/K axis
  extents, and its K-divisibility bucket from the K axis extent (§6.5-0012). *Test:*
  `test_classify_contraction_axis_roles`.
- **KISS-CLASSIFY-6.6-0017** — The reduce field (§6.6-0009) MUST carry a non-`-`
  value **only** for a **reduction cell**, defined as a cell whose `op_family` is
  `red` (§6.5-0006); for every cell whose `op_family` is not `red` the reduce field
  MUST be `-`. This pins the "reduction cell" referent used in §6.5-0009(b): the
  vector-width `v1` rule of §6.5-0009(b) applies to a reduced innermost axis of a
  `red` cell (scan cells derive `v1` via their own `scn` clause in §6.5-0009(b)). At
  this schema version an op family that reduces along an axis without being `red` —
  softmax (`sft`), normalization (`nrm`), attention (`att`), or loss (`los`) — MUST
  carry the reduce field `-` and does not key its reduction axis (a disclosed
  limitation, analogous to §6.6-0015). A reader MUST reject, with a typed decline
  (§7.1-0002), a token whose `op_family` is not `red` yet whose reduce field is not
  `-`. *Test:* `test_classify_reduce_field_op_family_gated`.
- **KISS-CLASSIFY-6.6-0018** — Because a `structure_key` omits several binding-only
  facts (and, for non-`gem` cells, keys only the primary dtype, §6.6-0015), a provider
  MUST NOT register two distinct specialization cells whose derived `structure_key`
  tokens are byte-identical; such cells MUST be disambiguated out-of-band (outside the
  admissibility key) so that a consumer's byte-exact lookup (§6.6-0001) resolves to
  exactly one cell and neither implementation silently overwrites the other. (This is
  the provider-side enforcement of the §8.2 caller-responsibility rule.) The
  mixed-precision `gem` collision that formerly forced this out-of-band step — two GEMMs
  differing only in weight / accumulator / output dtype — is resolved in-key by the sk3
  contraction coordinates (§6.7-0006), so such `gem` cells are now distinct by their
  tokens. *Test:* `test_classify_no_colliding_cell_registration`.
- **KISS-CLASSIFY-6.6-0019** — The **weight role** of a dense-contraction (`gem`)
  cell — the operand whose dtype the `contraction` field's `<wdt>` coordinate
  (§6.7-0006) carries — MUST be supplied by the caller as a role hint (§6.6-0012). An
  implementation MUST NOT infer the weight operand from operand **position**, and MUST
  NOT infer it from any operand's **dtype**: §6.6-0014's canonical order is the
  caller's call order, not a semantic role assignment, so no fixed operand position
  identifies the weight. The `<wdt>` token MUST be the §6.1 dtype of the
  caller-designated weight operand. This closes for the weight role the inference gap
  §6.6-0016 closes for the M/N/K axis roles. *Worked hazard (informative):* a
  quantized-GEMM operand order `[weight, weight_scale, activation]` places a **scale**
  at operand position 1, so an implementation reading a fixed "operand-1" position
  emits the scale's dtype (e.g. `f8e8m0`) where the weight's (e.g. `i4`) is meant — a
  well-formed, byte-stable token naming the wrong type, which nothing downstream would
  question. *Coverage (informative):* this rule is **not corpus-detectable at this
  schema version**. The `structure_key` corpus round-trips tokens
  (`to_token`/`from_token`); no stage derives `<wdt>` from operands, so no published
  token can disagree with a positional emitter today. A conformance detector needs two
  surfaces deferred to a future schema version: (1) an **emit-derivation stage** that
  computes a token from a cell and its role hints, and (2) **cell-carrying corpus
  vectors** the harness derives and compares. Until then the rule is an emit-side
  obligation the reference **states in code but no live path yet applies**:
  `derive_weight_dtype` defines the derivation and is applied only to construct the
  `gem_weight_role_discriminator` corpus vector (and exercised by its test); no live
  cell→token path derives `<wdt>` at this schema version — the function is the seed of
  the deferred emit-derivation stage, not a production path. So no golden token
  distinguishes a hint-following emitter from a positional one today: that vector
  (operands `[i4, f8e8m0, bf16]`, `<wdt>` = the hinted weight `i4`, where a fixed
  operand-1 read would name the scale `f8e8m0`) pins the correct bytes now but is a
  **latent** detector — inert until an emit-derivation stage lets a foreign emitter
  diverge from it. The
  "no fixed operand position identifies the weight" conclusion additionally depends on
  §6.1-0013 leaving the MX-scale **sibling** slot unpinned; if a later revision pins
  sibling placement, this clause's positional prohibition MUST be revisited. *Test:*
  `test_classify_weight_role_hint`.

### 6.7 The `structure_key` token codec

The token is the stable string serialization of a `structure_key` — the wire form
KISS-Announce carries opaquely. Its grammar is `|`-separated fields:

```
sk<version> | <op_family> | <dtype> | <target> | <index_width> | <work_class>
            | r<rank> | <operand0>;<operand1>;… | <reduce>
            [ | c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp> ]
```

where each `<operandI>` is `<contig>/<bcasthex>/<vec>/<div>/<flip>` and:
`<contig>` ∈ `{co, ic, st, br}`; `<bcasthex>` is the 2-lowercase-hex-digit broadcast
mask (§6.7-0010); `<vec>` ∈ `{v1, v2, v4, v8}`; `<div>` ∈ `{d16, d8, d4, d2, da}`;
`<flip>` ∈ `{f, r}` (`r` = flipped). `<reduce>` is exactly one of `-`
(none / not-a-reduction), `rall` (all-axes reduction), `rlast` (trailing-axis
reduction), or `x<hex>` (an explicit keepdim bitmask, 2 lowercase hex digits, for
any other reduced-axis set) — the four distinctly-encoded values of §6.6-0009. The
optional final field (present only for a dense-contraction `gem` cell) is
`c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp>`: a geometry group (the M/N/K size
codes `{t, s, m, l}`, the K-divisibility code, and a conditionally-present batch
size-class `b<class>`) followed by a precision group (the weight / accumulator / output
dtype tokens and the math-precision code `<mp>` ∈ `{st, rm}`).

- **KISS-CLASSIFY-6.7-0001** — A `structure_key` token MUST consist of exactly nine
  `|`-separated fields for a non-contraction cell, or exactly ten fields (the tenth
  being the contraction field) for a dense-contraction cell; a reader MUST reject a
  token with any other field count with a typed decline. *Test:*
  `test_classify_token_field_count`.
- **KISS-CLASSIFY-6.7-0002** — Field 0 MUST be `sk` immediately followed by the
  canonical decimal schema version (`sk4` at this maturity); a reader MUST reject a
  token whose field 0 is not `sk` followed by a supported version, including a
  non-canonical leading-zero spelling (e.g. `sk04`) — the malformed-field wall. The
  canonical-form check MUST precede any numeric parse (a bare `parse::<u32>()` accepts
  `sk04` as `4`); the recognized-but-unaccepted-version case is the distinct
  §6.7-0015 signpost. *Test:* `test_classify_token_version_prefix`.
- **KISS-CLASSIFY-6.7-0003** — Fields 1–6 MUST be, in order, the op-family code
  (§6.5-0006), the dtype token (§6.1), the target_capability string (§6.8), the
  index-width code (`ix32`/`ix64`, §6.5-0005 — distinct from the `i32`/`i64` dtype
  tokens), the work-class code (`warp`/`block`/`grid`), and `r` immediately followed
  by the decimal iteration rank. *Test:* `test_classify_token_scalar_fields`.
- **KISS-CLASSIFY-6.7-0004** — Field 7 MUST be the per-operand sub-keys joined by
  `;`, each formatted `<contig>/<bcasthex>/<vec>/<div>/<flip>` with the codes of
  §6.5 and §6.6-0007, in the canonical operand order of §6.6-0014; the number of
  `;`-separated entries MUST equal `n_operands` and MUST NOT exceed `MAX_OPERANDS`.
  *Test:* `test_classify_token_operand_field`.
- **KISS-CLASSIFY-6.7-0005** — Field 8 MUST be exactly one of the four
  distinctly-encoded reduce values of §6.6-0009: `-` (none / not-a-reduction),
  `rall` (all-axes reduction), `rlast` (trailing-axis reduction), or `x` followed by
  the 2-lowercase-hex-digit reduced-axis keepdim mask (§6.7-0010) for any other
  reduced-axis set. A reader MUST reject any other field-8 spelling with a typed
  decline; a producer MUST emit `rall` / `rlast` (never the equivalent `x<hh>`
  bitmask) for the all-axes and trailing-axis cases. The reader MUST additionally
  enforce the §6.6-0009 canonicalization **on the `x<hh>` form itself**: an `x<hh>`
  whose reduced-axis set has a shorter canonical spelling for the cell's `rank` — the
  empty set (spelled `-`), the all-axes set (`rall`), or the lone innermost axis
  (`rlast`), including the rank-1 case where all-axes and trailing coincide and
  §6.6-0009 ties to `rall` — is a **sentinel overload** and MUST be rejected with a
  typed decline (the reference reader names it `NonCanonicalReduceField`), so a given
  reduced set has exactly one accepted spelling and two conforming parties never key it
  two ways. This is **distinct** from an `x<hh>` bit set for a non-existent axis (index
  ≥ `rank`), which is an **out-of-range mask** rejected under §6.7-0009 (the reference
  reader names it `BadReduceField`): a wrong spelling of a real set versus a set that
  cannot exist for the rank are separable producer bugs, and a reader MUST keep the two
  declines discriminable (the `noncanonical_reduce_*` and `out_of_range_reduce_mask_r2`
  decline vectors). *Test:* `test_classify_token_reduce_field`.
- **KISS-CLASSIFY-6.7-0006** — When present (dense-contraction `gem` cells only,
  §6.7-0001), field 9 MUST be `c` followed by the three M/N/K size-class codes (each ∈
  `{t, s, m, l}`), then `/`-separated: the K-divisibility code (∈ `{d16, d8, d4, d2,
  da}`); a **conditionally-present** batch coordinate `b<class>` (`<class>` ∈ `{t, s, m,
  l}`) emitted **iff the cell is batched** — a non-batched cell omits it entirely; the
  weight, accumulator, and output dtype tokens `<wdt>`/`<acc>`/`<out>`, each from the
  closed §6.1 set (`<wdt>` is the dtype of the caller-designated **weight operand**,
  §6.6-0019, not the operand at any fixed position); and the math-precision code `<mp>` ∈ `{st, rm}` (`st` = bit-stable,
  `rm` = reduced-mantissa-permitted, resolving to the KISS-Ops MathPrecision value of
  KISS-OPS §6.17 per `(primary_dtype, target)` — on an `f32` primary at `cuda:sm80+`,
  `rm` is TF32, §6.17-0006). `<mp>` codes MUST NOT begin with `b` (reserved for the
  batch coordinate), so the geometry and precision groups never collide in spelling. The
  full form is `c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp>` — six `/`-parts
  non-batched, seven batched. A reader MUST reject a malformed precision group (a dtype
  outside the closed set, an `<mp>` not in `{st, rm}`, or a wrong part-count) with a
  typed decline (§6.7-0009). The `<acc>` coordinate is the identity/lookup surface of the
  same accumulator dtype the contract declares as `accumulation_type` (KISS-CONTRACT
  §6.8-0011). *Test:* `test_classify_token_contraction_field`.
- **KISS-CLASSIFY-6.7-0007** — The token codec MUST be **spelling-keyed, not
  discriminant-keyed**: adding a new dtype code or op-family code MUST NOT change
  the bytes of any pre-existing token and MUST NOT bump the schema version. *Test:*
  `test_classify_token_codec_additive`.
- **KISS-CLASSIFY-6.7-0008** — `to_token` and `from_token` MUST round-trip: for
  every `structure_key` whose fields each satisfy the domains and derivations of
  §6.5–§6.6, parsing its serialized token MUST reproduce a byte-identical key, and
  re-serializing MUST reproduce a byte-identical token. *Test:*
  `test_classify_token_roundtrip`.
- **KISS-CLASSIFY-6.7-0009** — A reader MUST reject, with a typed decline and
  without a panic, abort, crash, hang, or out-of-bounds read, any token with a
  malformed field, an unknown op-family or dtype code, an out-of-range mask, or a
  length outside `[1, MAX_STRUCTURE_KEY_LEN]`. *Test:*
  `test_classify_token_reject_malformed`.
- **KISS-CLASSIFY-6.7-0010** — Every hexadecimal mask in a token — the per-operand
  broadcast mask `<bcasthex>` (§6.7-0004) and the reduce field's `x<hex>` keepdim
  bitmask form (§6.7-0005; the `-`, `rall`, and `rlast` reduce values are not hex
  masks) — MUST be **lowercase** hexadecimal, zero-padded to exactly two digits
  (`00`..`ff`); an implementation MUST NOT emit uppercase or variable-width hex, and
  a reader MUST reject such a token with a typed decline. *Test:*
  `test_classify_mask_hex_lowercase`.
- **KISS-CLASSIFY-6.7-0011** — The §6.7 string token MUST be the **only** normative
  wire/interchange form of a `structure_key`. The `u16` / `u8` / `i64` field types in
  the §6.3 and §6.6 tables describe a permitted in-memory representation only; no
  binary or fixed-width struct wire encoding is defined at this schema version, so
  byte offsets, endianness, and padding of any in-memory form are
  implementation-internal and MUST NOT be relied upon across parties. Two parties
  exchanging a `structure_key` MUST exchange the token. *Test:*
  `test_classify_token_is_only_wire_form`.
- **KISS-CLASSIFY-6.7-0012** — A float **reduction or scan** tolerance-cell (a `reduce` or
  `prefix_scan` over a float `sum`/`prod`, KISS-OPS §6.17-0008) is keyed on its
  accumulator dtype, but the current key grammar carries an accumulator coordinate ONLY for
  dense-contraction `gem` cells (the field-9 `<acc>` of §6.7-0006); the non-contraction key
  (§6.7-0001) has **no** accumulator coordinate. This clause states the **forward
  requirement**: a non-contraction reduction/scan cell whose accumulator dtype differs from
  its compute dtype MUST carry an accumulator-dtype coordinate drawn from the closed §6.1
  dtype set — the non-contraction analogue of the contraction `<acc>`. When the coordinate
  is **absent**, the accumulator dtype MUST default to the compute dtype
  (accumulator-dtype == compute-dtype, the §6.17-0005 diagonal), so every existing token is
  unchanged in meaning and every kernel that never opts in behaves exactly as at this
  version. The **wire/codec realization** — the non-contraction optional-trailing precision
  field `<acc>/<mp>` and its byte-exact spelling — is **realized at sk4** by §6.7-0013 (the
  coordinated schema-version bump this clause anticipated). *Test:*
  `test_classify_reduction_accumulator_coordinate`.
- **KISS-CLASSIFY-6.7-0013** — The non-contraction precision field `(acc + mp)` (sk4,
  realizing §6.7-0012) is the **optional-trailing** precision field of a non-contraction cell,
  spelled **gem-symmetrically** as `<acc>/<mp>` — a §6.1 accumulator dtype and the
  math-precision code `<mp>` (the same codes as the `gem` contraction group's `<mp>`,
  §6.7-0006), `/`-delimited. It occupies the **same optional-trailing token slot** the
  contraction group occupies for `gem`; a cell carries **at most one** precision field — a
  `gem` cell the contraction group, a non-contraction cell the `(acc + mp)` field — and the two
  **never coexist**, so `from_token`'s nine-or-ten-field dispatch is resolved unambiguously by
  the op-family code. The field:
  (a) is **emitted iff at least one** of {accumulator dtype ≠ compute dtype, `<mp>` ≠ the
  cell's default math-precision} holds — that default being the KISS-Ops-owned default of
  KISS-OPS-6.17-0001 (a computation is bit-stable unless a reduced-mantissa variant is
  explicitly requested), spelled `st` here (§6.7-0006);
  (b) when emitted, spells **both** slots explicitly (`<acc>/<mp>`), including a slot equal to
  its default;
  (c) when **neither** coordinate deviates, is **omitted entirely** — not `-`, not empty;
  (d) MUST NOT be emitted in the all-default form (redundant emission is forbidden — a token
  carrying the field with both slots at default is invalid and MUST be rejected);
  (e) is **omitted-when-absent**, in deliberate contrast to the **mandatory** reduce field
  (§6.6-0009) which emits `-` when inapplicable: applying the reduce field's `-` convention to
  `(acc + mp)` would emit a spurious trailing `|-` and fail the byte-match, and is forbidden.
  The `<mp>` coordinate extends the strict-vs-TF32 math-precision axis (§6.7-0006) to the
  non-contraction key, so it stops collapsing for reductions/scans. This field **declares** the
  accumulator/precision coordinate for identity; it does **not** pin bit-level determinism
  (§6.17-0007 — float accumulation may remain order-invariant-nondeterministic). **Byte-stability:**
  every non-contraction token whose accumulator equals its compute dtype and whose `<mp>` is
  default is **byte-identical to the pre-sk4 codec** modulo the §6.1 dtype renames, so the sk4
  regen diff is exactly the cells whose accumulator/precision actually deviates. The rule is
  assented byte-for-byte by the four token-deriving parties (KISS, Fuel, kiss-ref,
  `unpopped-vocab`). *Test:* `test_classify_noncontraction_acc_mp_field`.
- **KISS-CLASSIFY-6.7-0014** — **Old-version decoder-arm retention is bounded, not
  indefinite.** An implementation **MAY** retain a bounded `sk3` decoder arm through
  the `sk4` series — decoding a persisted `sk3`-prefixed token under the sk3
  vocabulary — but it **MUST NOT** accept an `sk3`-prefixed token at `sk5` or later:
  the arm's retirement is **scheduled** to the `sk5` event, an event every party
  already tracks, not merely permitted at some unfixed future point. A consumer
  **MUST NOT** rely on cross-implementation `sk3` acceptance — because the arm is a
  MAY, whether any given build decodes a persisted `sk3` token is
  implementation-dependent, so a persisted `sk3` token **MUST** be re-derived, not
  assumed decodable. This clause fixes the migration-window policy the sk4 schema
  event named but left open (an unbounded `MUST be bounded` is a permanent arm by
  default — the outcome the boundedness requirement exists to prevent). The arm's
  soundness **depends on** the exact version-prefix matching of §6.7-0002/§6.7-0015:
  `c64` decodes as pair-`f64` under sk3 and pair-`f32` under sk4 (§6.1-0012), so a
  reader that loosened the version match would silently decode a cross-vocabulary
  token under the wrong dtype semantics. The KISS **reference** codec ships **no**
  `sk3` arm; it therefore declines every non-`sk4` version, naming the cause per
  §6.7-0015. *Test:* `test_classify_no_sk3_decoder_arm`.
- **KISS-CLASSIFY-6.7-0015** — **A well-formed token at an unaccepted schema version
  is a distinct, recognized decline — not a malformed-token decline.** On the
  §6.1-0001 recognized-vs-unknown model (a reserved dtype is *recognized but not
  usable here*, distinct from an *unknown* token), a reader MUST distinguish two
  version-field verdicts: (a) a **canonical** decimal version (`0` or `[1-9][0-9]*`)
  that is not one this build accepts is a **recognized token from another schema** —
  it MUST decline with a typed *unsupported-schema-version* verdict that **names the
  version**, the "re-derive" signpost; (b) a **malformed** version field — no `sk`
  prefix, a non-numeric remainder, or a **non-canonical leading-zero** spelling
  (`sk04`) — MUST decline with the distinct *bad-version-prefix* verdict, the "not a
  token" wall. The canonical-form check (the leading-zero guard of §6.7-0002) MUST
  run **before** any numeric parse: a bare `parse::<u32>()` would accept `sk04` as
  `4` and, one loosening later, a cross-vocabulary token under the wrong dtype
  meaning (§6.1-0012, §6.7-0014). This split is what makes the §6.7-0014 MAY-arm safe
  in both directions — whichever an implementation chooses, a consumer that hits a
  persisted older-schema token gets a diagnosable answer rather than an opaque
  refusal. *Test:* `test_classify_unsupported_schema_version`.

### 6.8 The target-capability descriptor

The target-capability descriptor names the compilation target a specialized kernel
is built for. Its grammar is `<namespace>:<capability-set>` — a single ASCII colon
separating a registered namespace from that namespace's capability-set token.

- **KISS-CLASSIFY-6.8-0001** — A `target_capability` token MUST have the form
  `<namespace>:<capability-set>`: a non-empty namespace, exactly one `:` separator,
  and a non-empty capability-set; a reader MUST reject a token with zero or more
  than one `:`, an empty namespace, or an empty capability-set with a typed decline.
  *Test:* `test_classify_target_token_grammar`.
- **KISS-CLASSIFY-6.8-0002** — Matching of two `target_capability` tokens MUST be
  **byte-exact on the full string**; an implementation MUST NOT apply ordering,
  subset, prefix, or feature-implication logic. Two tokens that differ by any byte
  MUST NOT match. *Test:* `test_classify_target_byte_exact_match`.
- **KISS-CLASSIFY-6.8-0003** — The `<namespace>` component MUST be a namespace
  registered with the steward; a KISS-owned namespace assignment MUST originate from
  a merged change to the PR-gated ThinkersJournal namespace registry, published as a
  **versioned, machine-readable registry file bundled with the KISS-Conform suite**
  so an offline reader holds a complete copy. Registration MUST be enforced only when
  a party first **produces** (announces) a kernel under a new namespace; byte-exact
  matching (§6.8-0002) MUST NOT consult the registry. An implementation MUST NOT
  produce a kernel under an unregistered namespace. *Test:*
  `test_classify_target_namespace_registered`.
- **KISS-CLASSIFY-6.8-0004** — The `<capability-set>` vocabulary for a namespace
  MUST be owned by that namespace's maintainer; KISS clauses MUST pin only the token
  grammar and the byte-exact match rule, never a specific namespace's capability-set
  vocabulary (which freezes independently — §8). *Test:*
  `test_classify_target_capability_set_owned_by_namespace`.
- **KISS-CLASSIFY-6.8-0005** — A `target_capability` token MUST be case-sensitive
  ASCII and MUST NOT contain the `structure_key` token field separators (`|`, `;`,
  `/`) or any whitespace or control byte (`0x00`–`0x20`, `0x7f`), so it embeds in a
  `structure_key` token as a single unambiguous field; matching (§6.8-0002) is
  byte-exact and therefore case-sensitive. *Test:*
  `test_classify_target_token_charset`.
- **KISS-CLASSIFY-6.8-0006** — Where a namespace's capability-set encodes a set
  of members by **concatenating their names without a separator**, that
  namespace's member alphabet MUST be **fixed-width**. A namespace whose member
  names vary in length MUST separate adjacent members with an explicit
  delimiter. This binds every namespace, not only those registered today.
  *Test:* `test_classify_target_fixed_width_juxtaposition`.

  > *Informative.* Juxtaposition is safe exactly when the alphabet is uniquely
  > decodable, and a fixed-width alphabet is uniquely decodable **by
  > construction** — for every member the namespace will ever add, without
  > anyone re-checking. A variable-length alphabet is uniquely decodable only
  > *contingently*: the property holds until some future member breaks it, and
  > it cannot be verified by eye, since distinctness is strictly weaker than
  > unique decodability (`{a, ab, b}` are distinct, yet `ab` parses two ways).
  > The rule is therefore not "never juxtapose" — it is that juxtaposition
  > requires an alphabet that cannot stop being decodable. §6.1's own dtype
  > tokens are the cautionary case: they are uniquely decodable today but not
  > prefix-free (`f8e4m3fn` prefixes `f8e4m3fnuz`; `f8e5m2` prefixes `f8e5m2fnuz`), so
  > the property holds by a margin no reviewer is checking. That is what the
  > §6.1 decodability lint exists to hold.
- **KISS-CLASSIFY-6.8-0007** — Where a namespace's capability-set replaces a
  long enumeration with a **digest**, it MUST use FNV-1a 64-bit with offset
  basis `0xcbf29ce484222325` and prime `0x100000001b3`, emitted as exactly 16
  lowercase hexadecimal digits, computed over the UTF-8 bytes of the canonical
  enumeration string it replaces; the digest MUST carry an algorithm marker
  (`fnv1a64`) delimited from the hex by a byte other than `:`; and the choice
  between enumerating and digesting MUST be a deterministic function of that
  canonical string's byte length against a threshold the namespace pins, never
  an implementation preference. *Test:*
  `test_classify_target_digest_pinned`.

  > *Informative.* Pinned once here rather than per namespace so a conformance
  > implementation carries one digest primitive rather than N, and so the
  > byte-order and width questions where determinism bugs hide are settled in a
  > single place. FNV-1a is chosen because §6.9-0003 requires a token be
  > producible with only a standard library, which rules out reaching for a
  > cryptographic hash; producers here are not adversarial, so accidental
  > collision resistance is the only requirement. Hashing *the same string that
  > is measured* means two producers can disagree about whether to digest but
  > never about what was digested. The algorithm marker keeps a future hash
  > migration distinguishable rather than silently colliding, and it excludes
  > `:` because §6.8-0001 permits exactly one colon in a token — the namespace
  > separator.
- **KISS-CLASSIFY-6.8-0008** — A namespace's capability-set vocabulary (§6.8-0004)
  MAY be published as a machine-readable **vocabulary manifest** whose `schema` is
  `kiss-namespace-vocabulary-v1`. Such a manifest MUST carry `schema`, `namespace` (a
  namespace whose registry status is `registered`, §6.8-0003), `vocabulary_version` (an
  **integer** — a gate that truncates a fractional value is not a gate), `generated_from`,
  `kind`, `grammar`, and `coverage_note`. `grammar` is required because §6.8-0012 places it in
  the declarative half, which MUST be sufficient for a parse-only consumer. A reader MUST
  reject with a typed decline a manifest missing any of these or bearing an unrecognized
  `schema`. KISS pins this **envelope** only — the manifest's vocabulary content remains
  owned by the namespace maintainer (§6.8-0004) and is never pinned here. *Test:*
  `test_namespace_vocabulary_envelope_shape`.

  > *Informative.* The manifest is the machine-readable form of the annex the
  > namespace registry (§6.8-0003) already points at, standing to a capability
  > vocabulary as `dtype_manifest.json` stands to §6.1's dtype set: one artifact a
  > consumer binds against instead of hand-transcribing an annex or hand-parsing its
  > prose. The two registered namespaces differ **in kind** — `cuda` is a closed
  > enumeration, `vulkan` a generated grammar over an open product space — so a single
  > format that assumed either would exclude the other; the shared thing is the
  > envelope, not the shape.
- **KISS-CLASSIFY-6.8-0009** — A consumer that reads a vocabulary manifest MUST
  **assert** that its `vocabulary_version` equals the version the consumer was built
  against, and MUST reject a mismatch with a typed decline; it MUST NOT proceed against
  a `vocabulary_version` it does not recognize. Reading the field without gating on it
  does not satisfy this clause — a vocabulary freezes independently of the schema
  version (§6.8-0004, §8), so a consumer that handles a version skew *gracefully* has
  defeated the freeze rather than honored it. *Test:*
  `test_namespace_vocabulary_version_is_asserted`.
- **KISS-CLASSIFY-6.8-0010** — `kind` discriminates a manifest's vocabulary shape and
  is an **open set**. A reader encountering a `kind` it does not recognize MUST reject
  with a typed decline and MUST NOT guess a shape. The kinds defined at this schema
  version are `enumerated` (a closed `members` list) and `generated` (an open product
  space validated by vectors, §6.8-0013); a further kind is admitted additively. *Test:*
  `test_namespace_vocabulary_kind_open_set`.

  > *Informative.* A discriminated union whose cases were read off exactly two examples
  > is a closed list built from `n = 2`. The unrecognized-`kind` decline is the
  > abstention rule: a reader that cannot recognize a shape refuses rather than assuming
  > the nearer of the two it knows.
- **KISS-CLASSIFY-6.8-0011** — A vocabulary manifest MUST name **what emitted its bytes** in
  `generated_from`, and MUST **agree** with its prose annex under an
  emit-and-`git diff --exit-code` freshness gate. These are two obligations, and this clause
  previously fused them into an annex-as-source model that compelled a false field: it
  required `generated_from` to name the **annex**, so a manifest generated from a reference
  crate — with the annex a peer output rather than the source — could satisfy the clause only
  by naming a document its bytes did not come from. **Provenance names the producer;
  agreement is a relation between two artifacts, and neither settles which is the source.**
  An implementation MAY generate from the annex, from a reference crate, or from another
  authority, MUST name whichever it used, and owes the agreement gate in every case. The
  commit such an artifact records alongside this field is a **separate** obligation from
  either of the two above, and is not pinned at this version. *Test:*
  `test_namespace_vocabulary_freshness_provenance`.
- **KISS-CLASSIFY-6.8-0012** — A vocabulary manifest has a **declarative** half
  (grammar, alphabets, orderings, and — for `enumerated` — `members`) and a
  **production** half (canonicalization algorithms and, for `generated`, `vectors`).
  The declarative half MUST be sufficient for a consumer that only **parses** foreign
  tokens; the production half is required only for a consumer that also **produces**
  tokens. A conformance profile for a parse-only consumer MUST NOT require the
  production half. *Test:* `test_namespace_vocabulary_declarative_production_split`.
- **KISS-CLASSIFY-6.8-0013** — For `kind: generated`, the `field_spec` is documentation
  — canonicalization cannot be validated from a grammar — and the normative contract is
  the `vectors` array, whose entries a conformant producer's output MUST reproduce
  byte-exact. The vector set MUST cover, each vector tagged by what it `pins`: **`order`**
  (a non-canonically-ordered input and its canonical output), **`dedup`** (a
  duplicate-bearing input and its deduped output), each length-conditional **`threshold`**
  (presented *at* and *immediately across* its boundary, so both forms are pinned at the
  exact byte count that flips them), and the **`digest_input`** fed to any §6.8-0007
  digest (the *same* byte string measured against the threshold, so a producer may
  disagree about *whether* to digest but never about *what* is digested). A namespace with
  no length-conditional field omits `threshold`/`digest_input` and states so in its
  `coverage_note`. *Test:*
  `test_namespace_vocabulary_generated_vectors_cover_canonicalization`.

> **Informative examples.** Well-formed `target_capability` tokens include
> `cuda:sm80`, `cuda:sm89`, `cuda:sm90a`, `cuda:sm100a`, `rocm:gfx942`,
> `rocm:gfx1100`, and `metal:apple9`. These are illustrative; the per-namespace
> capability-set vocabulary is owned by each namespace's maintainer and is not
> pinned normatively here.
>
> Note that each of these keys on a **hardware capability**, not on an encoding
> or IR version — `sm89` names what a kernel is compiled *for*, not the PTX ISA
> version it happens to be expressed in. A capability-set that named an encoding
> envelope instead would be non-injective over the cells it must separate: two
> devices consuming the same IR version routinely require different kernels, and
> because §6.8-0002 matching is byte-exact with subset and implication logic
> forbidden, such a token would silently merge those cells rather than degrade.
> The `vulkan:` capability-set, for instance, must separate cells by subgroup
> width and by cooperative-matrix support — a single Vulkan device commonly
> admits more than one subgroup width, so a token that cannot distinguish them
> would collide two kernels that are not interchangeable. Its concrete grammar is
> defined by that namespace's maintainer under §6.8-0004 and is deliberately not
> spelled here — see
> [`spec/namespaces/vulkan.md`](namespaces/vulkan.md), which the registry points
> at. A derived example, from a device whose pinnable subgroup range is 32..=64:
> `vulkan:sg64.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32`.
> Its `sg32` sibling is a **different** token and therefore a different cell,
> which is exactly the separation `spirv1.6` could not express.
>
> **Registered namespaces** live in
> [`conformance/registry/namespaces.json`](../conformance/registry/namespaces.json)
> (§6.8-0003). A namespace listed there as `reserved` rather than `registered`
> has no maintainer and no vocabulary: the name is held so it cannot be claimed
> by an unrelated party, but a producer MUST NOT emit tokens under it. Note that
> a namespace appearing in an *informative example* above is not thereby
> registered — the registry is the authority, and the examples are prose.

### 6.9 Foundational independence and opaque carry

- **KISS-CLASSIFY-6.9-0001** — KISS-Classify MUST NOT depend on KISS-Ops or on any
  other KISS sub-standard; it has no upstream edge. An implementation of
  KISS-Classify MUST be buildable and testable without any KISS-Ops artifact.
  *Test:* `test_classify_no_upstream_dependency`.
- **KISS-CLASSIFY-6.9-0002** — A `structure_key` token MUST be carriable as an
  **opaque, length-delimited** byte token across the discovery/announce seam; the
  vocabulary MUST NOT require the carrying party to parse the token's internals, and
  the token MUST remain byte-identical and parseable per §6.7 when carried and
  returned byte-for-byte. *Test:* `test_classify_structure_key_opaque_carry`.
- **KISS-CLASSIFY-6.9-0003** — Producing, serializing, or parsing a dtype token, an
  operand descriptor, a `structure_key`, or a `target_capability` token MUST NOT
  require loading a compute driver, kernel runtime, GPU library, or any backend
  dynamic library; an implementation MUST be able to do so using only its language's
  standard library. This obligation binds every implementation uniformly; the
  reference implementation holds no exemption. *Test:* `test_classify_zero_dependency`.

---

## 7. Capability, Profile & Extension model

### 7.1 Mandatory core

- **KISS-CLASSIFY-7.1-0001** — The KISS-Classify **mandatory core** — which every
  conforming implementation MUST satisfy regardless of claimed options — MUST be:
  the full twenty-two-dtype set (§6.1), the operand-descriptor field set (§6.3), the
  pinned constants (§6.4), the enumerations and derivations (§6.5), the
  `structure_key` field layout and admissibility semantics (§6.6), the token codec
  (§6.7), and the target-capability grammar and byte-exact match (§6.8). An
  implementation that cannot satisfy the mandatory core does not conform to
  KISS-Classify. *Test:* `test_classify_mandatory_core`.
- **KISS-CLASSIFY-7.1-0002** — An implementation MUST answer an unrecognized or
  out-of-range input (an unknown dtype token, an over-`MAX_RANK` rank, an
  over-`MAX_OPERANDS` count, a malformed token, an over-length token, a collapsed
  reduction, or a malformed target) with a **typed decline** (§3, Terms) and MUST
  NOT panic, abort, crash, hang, or read out of bounds. *Test:*
  `test_classify_unclaimed_input_typed_decline`.

### 7.2 Extension model

- **KISS-CLASSIFY-7.2-0001** — Adding a new dtype code, op-family code, or
  `target_capability` namespace MUST be an **additive** extension that leaves every
  pre-existing `structure_key` token byte-identical and does not bump the schema
  version (§6.7-0007); such an addition MUST originate from a merged change to the
  PR-gated ThinkersJournal registry. *Test:* `test_classify_extension_is_additive`.
- **KISS-CLASSIFY-7.2-0002** — A predicate-axis change that is **not** additive
  (removing, reordering, or altering the meaning of an existing `structure_key`
  field or enumeration) MUST bump the `structure_key` schema version (§8). *Test:*
  `test_classify_non_additive_bumps_version`.

---

## 8. Versioning & Lifecycle

KISS-Classify tracks the umbrella's **two version axes**: the wire/ABI *structure-key
schema version* (`STRUCTURE_KEY_VERSION`, currently `3`) and the published
reference-crate *semver*. They move independently. A third, Classify-local handle —
`DTYPE_LAYOUT_VERSION` (§8-0007) — separately tracks the pinned dtype bit **layouts** of
§6.1, on its own axis independent of both.

- **KISS-CLASSIFY-8-0001** — The `structure_key` schema version and the
  reference-crate semver MUST be tracked as independent axes; a crate semver change
  MUST NOT be taken to imply a `structure_key` wire change. *Test:*
  `test_classify_two_version_axes_independent`.
- **KISS-CLASSIFY-8-0002** — Any change that alters the bytes of a pre-existing
  `structure_key` token (a field reorder, an enumeration renumber, a codec change,
  or a constant change to `MAX_RANK` / `MAX_OPERANDS` / `MAX_STRUCTURE_KEY_LEN`)
  MUST bump the `structure_key` schema version. *Test:*
  `test_classify_wire_change_bumps_version`.
- **KISS-CLASSIFY-8-0003** — Assigning a previously-unused dtype code, op-family
  code, or target namespace MUST NOT bump the schema version (additive,
  spelling-keyed; §6.7-0007). *Test:* `test_classify_additive_no_version_bump`.
- **KISS-CLASSIFY-8-0004** — KISS-Classify MUST NOT be promoted from Draft to
  Frozen until at least two structurally dissimilar implementations — **including at
  least one whose `target_capability` namespace differs from the reference
  implementation's namespace** — have interoperated on the golden `structure_key`
  token vectors of Appendix A. *Test:*
  `test_classify_freeze_gate_two_dissimilar_impls` (checklist gate; AUDIT-signed).

  > *Informative.* The reference implementation's namespace is `cuda`; the
  > per-namespace capability-set vocabulary freeze accordingly waits on real
  > non-CUDA usage (see §8.4 resolved-decision note).
- **KISS-CLASSIFY-8-0005** — KISS-Classify MUST NOT be promoted from Draft to
  Frozen until a foreign reader written outside the reference language has parsed and
  reproduced the golden `structure_key` tokens and dtype table byte-for-byte
  (umbrella §5.3 freeze gate). *Test:* `test_classify_freeze_gate_foreign_reader`
  (checklist gate; AUDIT-signed).
- **KISS-CLASSIFY-8-0006** — KISS-Classify MUST NOT be promoted from Draft to
  Frozen until this sub-standard's KISS-Conform suite exists and passes with
  complete bidirectional clause-to-test traceability. *Test:*
  `test_classify_freeze_gate_conform_suite_passes` (checklist gate; AUDIT-signed).
- **KISS-CLASSIFY-8-0007** — The dtype bit **layouts** pinned in §6.1 (each dtype's storage
  width, and for the float dtypes the sign/exponent/mantissa split and exponent bias) are
  tracked by a dedicated schema handle, `DTYPE_LAYOUT_VERSION` (initial value `1`), on an
  axis independent of `STRUCTURE_KEY_VERSION` and the crate semver. Any non-additive change
  to a pinned layout fact MUST bump `DTYPE_LAYOUT_VERSION`; assigning a previously-unused
  dtype code MUST NOT (additive, §8-0003). *Test:* `test_classify_dtype_layout_version_handle`.

> **Resolved decisions (informative; ratified 2026-07-12, tracked as RFCs).**
> **(8.1 — RESOLVED)** Compute precision is **not** a dtype. The dtype set is pure
> storage; there is no strict-precision float variant. Whether a computation must be
> bit-stable full-precision or may use a reduced-mantissa reduction is a **KISS-Ops
> fidelity attribute** (a `MathPrecision`-style attribute alongside the KISS-Ops
> determinism/fidelity enum), surfaced in a kernel's KISS-Contract guarantees
> (§6.1-0005). **(8.2 — RESOLVED)** Index-only-ness is an **operand role**, not a
> dtype class. `u32` is an ordinary storage dtype (§6.1-0006); the index/address
> role (`index_operand` + `index_dtype`) is carried by KISS-Ops on the
> gather/scatter operand. Non-primary operand dtypes still do **not** enter the
> admissibility key at this schema version (§6.6-0015 keys only the primary dtype).
> **(8.3 — CONFIRMED)** The sub-byte/packed packing conventions (`i4`/`u4` nibble
> order, `b1` LSB-first) are owned **here** in Classify as byte layout
> (§6.1-0008/0009); KISS-Ops references them for popcount/MMA semantics. **(8.4)**
> Whether the `target_capability` namespace axis is keyed on ecosystem/compilation
> target (recommended) or on manufacturer — still open. **(8.5 — RESOLVED)** The
> reduce spec is now **three distinctly-encoded values plus a general bitmask**:
> `-` (none / not-a-reduction), `rall` (all-axes), `rlast` (trailing-axis), and
> `x<hh>` (explicit keepdim bitmask for any other set); no single sentinel is
> overloaded (§6.6-0009). This non-additive split bumped `STRUCTURE_KEY_VERSION` to
> `2` (§6.4-0003). KISS-Classify stays **explicitly UNFROZEN** until the remaining
> open item (8.4) is resolved and the §8-0004/0005/0006 gates pass.

---

## 9. Conformance

An implementation conforms to KISS-Classify at a given `structure_key` schema
version if it (a) recognizes exactly the dtype set, operand-descriptor fields,
constants, enumerations and derivations, `structure_key` layout, token codec, and
target-capability grammar of §6–§8 for that version, (b) passes the KISS-Conform
suite for KISS-Classify at that version, and (c) satisfies the DAG prerequisite
closure. KISS-Classify is **foundational** and has no upstream STRUCTURAL edge, so
claiming it forces no co-claim of any other sub-standard (§6.9-0001). Downstream
sub-standards that claim KISS-Classify on a STRUCTURAL edge (Grammar, Contract,
Consume, Emit) co-claim it; KISS-Announce depends on it only on an **OPAQUE** edge
and needs agreement on the meaning of the `structure_key` token, not a co-claim.
Un-claimed or malformed inputs yield typed declines, never panics (§7.1-0002, the
owning clause; verified by the negative-vector modality). The modified-suite
prohibition of the mark policy is the umbrella's rule (umbrella §9.3), enforced via
registry listing, and is not restated as a free-standing Classify clause.

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-CLASSIFY-6.0-0001 | `test_classify_determinism_class_exact_byte` |
| KISS-CLASSIFY-6.0-0002 | `test_classify_special_value_bit_patterns` |
| KISS-CLASSIFY-6.1-0001 | `test_classify_dtype_set_is_closed` |
| KISS-CLASSIFY-6.1-0002 | `test_classify_dtype_bit_widths` |
| KISS-CLASSIFY-6.1-0003 | `test_classify_dtype_numeric_kinds` |
| KISS-CLASSIFY-6.1-0004 | `test_classify_dtype_token_spelling` |
| KISS-CLASSIFY-6.1-0005 | `test_classify_dtypes_are_pure_storage` |
| KISS-CLASSIFY-6.1-0006 | `test_classify_u32_is_ordinary_storage` |
| KISS-CLASSIFY-6.1-0007 | `test_classify_bool_encoding` |
| KISS-CLASSIFY-6.1-0008 | `test_classify_sub_byte_nibble_packing` |
| KISS-CLASSIFY-6.1-0009 | `test_classify_b1_bit_packing` |
| KISS-CLASSIFY-6.1-0010 | `test_classify_e4m3_format` |
| KISS-CLASSIFY-6.1-0011 | `test_classify_e5m2_format` |
| KISS-CLASSIFY-6.1-0012 | `test_classify_complex_interleaved_layout` |
| KISS-CLASSIFY-6.1-0013 | `test_classify_mx_scale_format` |
| KISS-CLASSIFY-6.2-0001 | `test_classify_numeric_kind_set_closed` |
| KISS-CLASSIFY-6.2-0002 | `test_classify_float_special_values_pinned` |
| KISS-CLASSIFY-6.3-0001 | `test_classify_operand_rank_bounds` |
| KISS-CLASSIFY-6.3-0002 | `test_classify_operand_active_axes_only` |
| KISS-CLASSIFY-6.3-0003 | `test_classify_strides_are_signed_elements` |
| KISS-CLASSIFY-6.3-0004 | `test_classify_extent_is_capacity` |
| KISS-CLASSIFY-6.3-0005 | `test_classify_alignment_is_bytes` |
| KISS-CLASSIFY-6.3-0006 | `test_classify_operand_dtype_in_set` |
| KISS-CLASSIFY-6.3-0007 | `test_classify_layout_tag_is_derived` |
| KISS-CLASSIFY-6.3-0008 | `test_classify_op_family_is_cell_level` |
| KISS-CLASSIFY-6.3-0009 | `test_classify_quant_carried_not_keyed` |
| KISS-CLASSIFY-6.3-0009a | `test_classify_dequant_form_keyed` |
| KISS-CLASSIFY-6.3-0010 | `test_classify_symbolic_extent_flags_live_length` |
| KISS-CLASSIFY-6.3-0011 | `test_classify_axis_ordering_convention` |
| KISS-CLASSIFY-6.4-0001 | `test_classify_max_rank_is_8` |
| KISS-CLASSIFY-6.4-0002 | `test_classify_max_operands_is_8` |
| KISS-CLASSIFY-6.4-0003 | `test_classify_structure_key_version_matches_codec` |
| KISS-CLASSIFY-6.4-0004 | `test_classify_structure_key_token_length_bound` |
| KISS-CLASSIFY-6.5-0001 | `test_classify_layout_tag_enum` |
| KISS-CLASSIFY-6.5-0002 | `test_classify_layout_tag_derivation` |
| KISS-CLASSIFY-6.5-0003 | `test_classify_vec_width_enum` |
| KISS-CLASSIFY-6.5-0004 | `test_classify_div_bucket_enum` |
| KISS-CLASSIFY-6.5-0005 | `test_classify_index_width_boundary` |
| KISS-CLASSIFY-6.5-0006 | `test_classify_op_family_enum` |
| KISS-CLASSIFY-6.5-0007 | `test_classify_work_class_enum` |
| KISS-CLASSIFY-6.5-0008 | `test_classify_size_class_enum` |
| KISS-CLASSIFY-6.5-0009 | `test_classify_vec_width_derivation` |
| KISS-CLASSIFY-6.5-0010 | `test_classify_work_class_element_count` |
| KISS-CLASSIFY-6.5-0011 | `test_classify_index_width_offset` |
| KISS-CLASSIFY-6.5-0012 | `test_classify_div_bucket_derivation` |
| KISS-CLASSIFY-6.5-0013 | `test_classify_vec_width_unit_stride` |
| KISS-CLASSIFY-6.6-0001 | `test_classify_structure_key_is_admissibility_predicate` |
| KISS-CLASSIFY-6.6-0002 | `test_classify_structure_key_is_not_op_identity` |
| KISS-CLASSIFY-6.6-0003 | `test_classify_structure_key_extent_free` |
| KISS-CLASSIFY-6.6-0004 | `test_classify_structure_key_field_layout` |
| KISS-CLASSIFY-6.6-0005 | `test_classify_structure_key_primary_dtype` |
| KISS-CLASSIFY-6.6-0006 | `test_classify_structure_key_rank_and_operand_count` |
| KISS-CLASSIFY-6.6-0007 | `test_classify_operand_sub_key_fields` |
| KISS-CLASSIFY-6.6-0008 | `test_classify_broadcast_axis_mask` |
| KISS-CLASSIFY-6.6-0009 | `test_classify_reduce_axes_encoding` |
| KISS-CLASSIFY-6.6-0010 | `test_classify_contraction_field_optional` |
| KISS-CLASSIFY-6.6-0011 | `test_classify_structure_key_derivation_canonical` |
| KISS-CLASSIFY-6.6-0012 | `test_classify_derivation_input_tuple` |
| KISS-CLASSIFY-6.6-0013 | `test_classify_mixed_rank_axis_alignment` |
| KISS-CLASSIFY-6.6-0014 | `test_classify_operand_canonical_order` |
| KISS-CLASSIFY-6.6-0015 | `test_classify_secondary_dtype_unkeyed` |
| KISS-CLASSIFY-6.6-0016 | `test_classify_contraction_axis_roles` |
| KISS-CLASSIFY-6.6-0017 | `test_classify_reduce_field_op_family_gated` |
| KISS-CLASSIFY-6.6-0018 | `test_classify_no_colliding_cell_registration` |
| KISS-CLASSIFY-6.6-0019 | `test_classify_weight_role_hint` |
| KISS-CLASSIFY-6.7-0001 | `test_classify_token_field_count` |
| KISS-CLASSIFY-6.7-0002 | `test_classify_token_version_prefix` |
| KISS-CLASSIFY-6.7-0003 | `test_classify_token_scalar_fields` |
| KISS-CLASSIFY-6.7-0004 | `test_classify_token_operand_field` |
| KISS-CLASSIFY-6.7-0005 | `test_classify_token_reduce_field` |
| KISS-CLASSIFY-6.7-0006 | `test_classify_token_contraction_field` |
| KISS-CLASSIFY-6.7-0007 | `test_classify_token_codec_additive` |
| KISS-CLASSIFY-6.7-0008 | `test_classify_token_roundtrip` |
| KISS-CLASSIFY-6.7-0009 | `test_classify_token_reject_malformed` |
| KISS-CLASSIFY-6.7-0010 | `test_classify_mask_hex_lowercase` |
| KISS-CLASSIFY-6.7-0011 | `test_classify_token_is_only_wire_form` |
| KISS-CLASSIFY-6.7-0012 | `test_classify_reduction_accumulator_coordinate` |
| KISS-CLASSIFY-6.7-0013 | `test_classify_noncontraction_acc_mp_field` |
| KISS-CLASSIFY-6.7-0014 | `test_classify_no_sk3_decoder_arm` |
| KISS-CLASSIFY-6.7-0015 | `test_classify_unsupported_schema_version` |
| KISS-CLASSIFY-6.8-0001 | `test_classify_target_token_grammar` |
| KISS-CLASSIFY-6.8-0002 | `test_classify_target_byte_exact_match` |
| KISS-CLASSIFY-6.8-0003 | `test_classify_target_namespace_registered` |
| KISS-CLASSIFY-6.8-0004 | `test_classify_target_capability_set_owned_by_namespace` |
| KISS-CLASSIFY-6.8-0005 | `test_classify_target_token_charset` |
| KISS-CLASSIFY-6.8-0006 | `test_classify_target_fixed_width_juxtaposition` |
| KISS-CLASSIFY-6.8-0007 | `test_classify_target_digest_pinned` |
| KISS-CLASSIFY-6.8-0008 | `test_namespace_vocabulary_envelope_shape` |
| KISS-CLASSIFY-6.8-0009 | `test_namespace_vocabulary_version_is_asserted` |
| KISS-CLASSIFY-6.8-0010 | `test_namespace_vocabulary_kind_open_set` |
| KISS-CLASSIFY-6.8-0011 | `test_namespace_vocabulary_freshness_provenance` |
| KISS-CLASSIFY-6.8-0012 | `test_namespace_vocabulary_declarative_production_split` |
| KISS-CLASSIFY-6.8-0013 | `test_namespace_vocabulary_generated_vectors_cover_canonicalization` |
| KISS-CLASSIFY-6.9-0001 | `test_classify_no_upstream_dependency` |
| KISS-CLASSIFY-6.9-0002 | `test_classify_structure_key_opaque_carry` |
| KISS-CLASSIFY-6.9-0003 | `test_classify_zero_dependency` |
| KISS-CLASSIFY-7.1-0001 | `test_classify_mandatory_core` |
| KISS-CLASSIFY-7.1-0002 | `test_classify_unclaimed_input_typed_decline` |
| KISS-CLASSIFY-7.2-0001 | `test_classify_extension_is_additive` |
| KISS-CLASSIFY-7.2-0002 | `test_classify_non_additive_bumps_version` |
| KISS-CLASSIFY-8-0001 | `test_classify_two_version_axes_independent` |
| KISS-CLASSIFY-8-0002 | `test_classify_wire_change_bumps_version` |
| KISS-CLASSIFY-8-0003 | `test_classify_additive_no_version_bump` |
| KISS-CLASSIFY-8-0004 | `test_classify_freeze_gate_two_dissimilar_impls` |
| KISS-CLASSIFY-8-0005 | `test_classify_freeze_gate_foreign_reader` |
| KISS-CLASSIFY-8-0006 | `test_classify_freeze_gate_conform_suite_passes` |
| KISS-CLASSIFY-8-0007 | `test_classify_dtype_layout_version_handle` |

Every normative clause above appears in this matrix exactly once; the KISS-Conform
build fails if any clause ID lacks a passing mapped test (bidirectional
traceability, a restatement of umbrella §3.3). Clause IDs are mirrored in the
machine-readable sidecar (`kiss-classify.validusage.json` analog) kept in sync by
the traceability lint; for this plain-old-data vocabulary tier, both the prose
tables of §6 and the sidecar are generated from the one canonical schema so they
cannot drift.

---

## 10. Governance

- **Editor of record:** the KISS-Classify editor assignment is recorded in the
  umbrella governance record (Classify is assigned to the data-vocabulary
  reference-impl project). The editor holds the pen, allocates clause IDs
  (append-only, never reused after retirement), and solicits comment from interested
  cosignatories — any project building a consumer, provider, emitter, or lifter that
  reads the dtype set, operand descriptors, or `structure_key` — before deciding a
  cross-party-visible change.
- **Steward:** ThinkersJournal hosts the spec, the dtype/op-family/namespace
  registries (PR-gated, published as versioned machine-readable files bundled with
  the KISS-Conform suite), and the conformance registry; it free-certifies
  self-certified implementations on request as resources permit.
- **Ratifier / maturity transitions:** the KISS-Conform AUDIT role (not DESIGN)
  signs each maturity transition; the Draft→Frozen transition requires the freeze
  gate of §8-0004 / §8-0005 / §8-0006 (umbrella §5.3), and Classify stays
  **explicitly UNFROZEN** until a second dissimilar, non-CUDA-namespace
  implementation certifies the target-capability vocabulary.
- **License:** this specification is dedicated to the public domain under CC0 1.0
  Universal; reference crates are MIT-OR-Apache-2.0; the KISS-Conform suite is
  permissive-to-run. Per the umbrella mark policy (umbrella §9.3), a modified
  conformance suite does not back a conformance claim; that policy is enforced via
  steward-registry listing, not restated as a normative Classify clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC
  contribution, with defensive termination, per the umbrella.
- **Conformance posture:** self-certification with published results plus the
  steward-maintained registry is the authoritative record of verified
  implementations.

---

## Appendix A — Golden vectors & provenance (informative)

**A.1 Golden `structure_key` token vectors.** The following tokens are the first
golden vectors for `test_classify_token_roundtrip`,
`test_classify_token_scalar_fields`, `test_classify_token_operand_field`, and the
byte-exact-match / additivity tests. Each is shown as the exact bytes on the wire,
left to right. Each vector **pins its complete derivation input** so a foreign
implementer can reproduce the token deterministically per §6.6-0011: every operand is
given as `(extents; strides; dtype; alignment)` in canonical order (§6.6-0014), plus
the cell's `op_family` and any role hints. (Recall: index-width token codes are
`ix32` / `ix64`, distinct from the `i32` / `i64` dtype tokens, §6.5-0005.)

- **Binary elementwise add** — `op_family = bin`; three operands (`in`, `in`, `out`),
  each `([128,256]; [256,1]; f32; 256)`; no role hints. Each operand is contiguous
  (`co`), no broadcast (`00`), vectorizes to `v4` (inner 256, `4·4 = 16 ≤ 16` byte
  cap, `16 ≤ A = 256`), inner extent 256 divisible by 16 (`d16`), unflipped (`f`).
  Max touched offset `256·127 + 1·255 = 32767 < 2³¹` ⇒ `ix32`; frame element count
  `128·256 = 32768 > 1024` ⇒ `grid`; rank 2; reduce field `-` (not a reduction):
  `sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-`
  This vector is the **canonical derivation golden vector** for
  `test_classify_structure_key_derivation_canonical` — the full input tuple above maps
  to exactly these bytes.
- **The same cell with operand 1 broadcasting axis 0** — operand 1 is
  `([128,256]; [0,1]; f32; 256)` (stride 0 on axis 0), all else unchanged. Operand 1
  becomes layout `broadcast` (`br`), broadcast mask bit 0 set (`01`), scalar width
  (`v1`, §6.5-0009(a)); its inner extent 256 still buckets `d16`:
  `sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;br/01/v1/d16/f;co/00/v4/d16/f|-`
- **Unary elementwise** — `op_family = une`; two operands (`in`, `out`), each
  `([64,128]; [128,1]; f16; 256)`; no role hints. `v8` (`8·2 = 16 ≤ 16` byte cap,
  `16 ≤ A = 256`, 8 divides inner 128); inner 128 buckets `d16`. Max offset
  `128·63 + 1·127 = 8191 < 2³¹` ⇒ `ix32`; `64·128 = 8192 > 1024` ⇒ `grid`; rank 2:
  `sk4|une|f16|cuda:sm89|ix32|grid|r2|co/00/v8/d16/f;co/00/v8/d16/f|-`
- **Reduction, keepdim, `[4,8] → [4,1]`** (trailing-axis reduce ⇒ reserved `rlast`,
  not a bitmask) — `op_family = red`, caller op category `reduction`; two operands,
  `in = ([4,8]; [8,1]; f32; 256)` and `out = ([4,1]; [1,1]; f32; 256)`. The input's
  innermost axis (extent 8) is reduced ⇒ `v1` (§6.5-0009(b)) while its own inner
  extent 8 still buckets `d8`; the output's size-1 inner axis buckets `da`. Max offset
  `31 < 2³¹` ⇒ `ix32`; frame `4·8 = 32 ≤ 32` ⇒ `warp`; rank 2:
  `sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rlast`
- **Reduction, keepdim, `[4,8] → [1,1]`** (all-axes reduce ⇒ reserved `rall`) — same
  inputs as above but `out = ([1,1]; [1,1]; f32; 256)` and **every** axis reduced.
  Operand-0 (the input) keeps its own inner extent 8 ⇒ bucket `d8` (reduction changes
  only vector width, §6.5-0009, never the divisibility bucket); the `[1,1]` output
  buckets `da`:
  `sk4|red|f32|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rall`
- **Rank-1 reduction, keepdim, `[8] → [1]`** (the single axis is simultaneously
  all-axes and the trailing axis; by the §6.6-0009 tiebreak `rall` takes precedence) —
  `op_family = red`; `in = ([8]; [1]; f32; 256)`, `out = ([1]; [1]; f32; 256)`. Inner
  extent 8 ⇒ `d8`; reduced innermost ⇒ `v1`; frame `8 ≤ 32` ⇒ `warp`; rank 1;
  reduce field `rall` (never `rlast`):
  `sk4|red|f32|cuda:sm89|ix32|warp|r1|co/00/v1/d8/f;co/00/v1/da/f|rall`
- **Reduction, keepdim, rank-4 reducing axes 1 and 3** (neither all-axes nor
  trailing ⇒ explicit keepdim bitmask `0x0a`, exercising a two-digit lowercase hex ≥
  `0x0a`) — `op_family = red`; `in = ([2,4,3,5]; [60,15,5,1]; f32; 256)`,
  `out = ([2,1,3,1]; [3,3,1,1]; f32; 256)`. Innermost axis (extent 5, odd) buckets
  `da` and is reduced ⇒ `v1`. Max offset `60·1 + 15·3 + 5·2 + 1·4 = 119 < 2³¹` ⇒
  `ix32`; frame `2·4·3·5 = 120` (`32 < 120 ≤ 1024`) ⇒ `block`; rank 4; reduce mask
  `x0a` (bits 1 and 3):
  `sk4|red|f32|cuda:sm89|ix32|block|r4|co/00/v1/da/f;co/00/v1/da/f|x0a`
- **Reduction with a deviating accumulator (sk4, §6.7-0013)** — a trailing-axis `f16`
  reduction `[4,8] → [4,1]` that accumulates in `f32`. `op_family = red`, dtype `f16`;
  `in = ([4,8]; [8,1]; f16; 256)`, `out = ([4,1]; [1,1]; f16; 256)`. Innermost axis
  reduced ⇒ `v1`; input inner extent 8 ⇒ `d8`, output inner extent 1 ⇒ `da`; frame
  `4·8 = 32` ⇒ `warp`; rank 2; reduce `rlast`. The accumulator dtype (`f32`) **differs**
  from the compute dtype (`f16`), so the optional-trailing non-contraction precision
  field is emitted — `<acc>/<mp>` = `f32/st` (mp at its `st` default, but rule (b)
  spells both slots). The same cell accumulating in `f16` (accumulator == compute) omits
  the field entirely (rule c), yielding the nine-field token without the trailing
  `|f32/st`; a redundant `|f16/st` on that cell is rejected (rule d):
  `sk4|red|f16|cuda:sm89|ix32|warp|r2|co/00/v1/d8/f;co/00/v1/da/f|rlast|f32/st`
- **In-place binary accumulate** (`op_family = bin`) — an operand that is both read
  and written appears **exactly once**, classified as an input (§6.6-0014). The
  accumulator `acc = ([128,256]; [256,1]; f32; 256)` (in-place) and addend
  `b = ([128,256]; [256,1]; f32; 256)` yield two operands (`acc` is **not** repeated
  as an output), `n_operands = 2`, operand-0 = `acc`:
  `sk4|bin|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f|-`
- **Dense GEMM skinny-decode cell** `[8,4096]·[4096,4096]→[8,4096]` — `op_family =
  gem`; three operands `lhs = ([8,4096]; [4096,1]; f32; 256)`,
  `rhs = ([4096,4096]; [4096,1]; f32; 256)`, `out = ([8,4096]; [4096,1]; f32; 256)`;
  role hints `lhs = [M,K]`, `rhs = [K,N]`, `out = [M,N]` (§6.6-0016). M = 8 (tiny
  `t`), N = K = 4096 (large `l`), K divisible by 16 (`d16`). This f32 GEMM is
  non-batched with f32 weight/accumulator/output and bit-stable compute, so the sk4
  precision group is `/f32/f32/f32/st` and the contraction field is
  `ctll/d16/f32/f32/f32/st`. Max offset (rhs) `4096·4095 + 1·4095 = 16777215 < 2³¹` ⇒
  `ix32`; work-class **frame-max** (§6.5-0010, the §6.6-0013 per-axis maximum across
  operands — **not** the output frame) `max(8,4096,8)·max(4096,4096,4096) = 4096·4096
  = 16777216 > 1024` ⇒ `grid`; rank 2:
  `sk4|gem|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st`
- **The same GEMM cell built for a Vulkan target** — a **different** cell that does
  not match the CUDA one (byte-exact target rule, §6.8-0002); inputs identical except
  `target = vulkan:sg64.ops-abr.arith-f16.cm-none`:
  `sk4|gem|f32|vulkan:sg64.ops-abr.arith-f16.cm-none|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st`

  > This is the re-mint anticipated when the vector was moved off
  > `vulkan:spirv1.6`: it now carries a **registered** namespace with a published
  > vocabulary, which the interim `rocm:gfx942` could not, since `rocm` is
  > reserved rather than registered and §6.8-0003 forbids producing under an
  > unregistered namespace. §A.1.1 expands the cross-namespace case.

**A.1.1 `vulkan` namespace vectors.** These satisfy the §8-0004 requirement for
interoperation with an implementation whose `target_capability` namespace differs
from the reference implementation's `cuda`. Each is labelled by what it tests,
because only one class is a shared obligation:

- **codec-neutral** — the envelope, field order, separators, escaping, dedup and
  sort order, and length discipline. **Both** implementations must encode these
  byte-identically *regardless of namespace*; this is the actual interop surface.
- **capability-specific** — the payload a namespace's vocabulary owns. Each
  implementation owns its own and they are never cross-emitted.

> **(a) `vulkan` GEMM cell, wave64-capable device — capability-specific.** The
> same GEMM inputs as A.1, targeted at a device whose default subgroup width is
> 64:
> `sk4|gem|f32|vulkan:sg64.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st`
>
> **(b) The same cell on a wave32-only device — capability-specific.** Differs
> from (a) in exactly one field of the capability-set, and is therefore a
> different cell:
> `sk4|gem|f32|vulkan:sg32.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st`
>
> (a) and (b) together are the point: they are two devices *and* two cells, and
> no single encoding-envelope token could separate them. A measured consumer
> laptop carries exactly this pair — an integrated AMD part at wave64 beside a
> discrete NVIDIA part at wave32, differing additionally in ICD — so the
> divergence appears on ordinary hardware rather than a constructed rig.
>
> **(c) Nonzero subgroup width — codec-neutral, and a deliberate loud failure.**
> Every `vulkan` vector above pins a **nonzero** subgroup width. A deriver that
> reads an untouched Vulkan 1.1+ `pNext` property struct — which happens when it
> gates on the *device's* API version instead of `min(instance, device)` — emits
> `sg0` and fails these vectors immediately, rather than shipping a
> plausible-but-wrong token that no test catches. The trap is silent by nature;
> the vector is what makes it loud.
>
> **(d) Width-agnostic cell — capability-specific.** A kernel that reads the
> subgroup width at runtime is a distinct artifact from either pinned variant:
> `sk4|gem|f32|vulkan:sgdyn.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f32-f32|ix32|grid|r2|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f|-|ctll/d16/f32/f32/f32/st`
>
> **(e) Minimal `vulkan` cell — codec-neutral.** Every field at its empty
> spelling, exercising the "no omissible fields" rule:
> `vulkan:sgdyn.ops-none.arith-none.cm-none`

**A.2 Adversarial / negative vectors.** The negative battery for the §6.7 / §6.8
reject tests and the foreign-reader freeze gate includes: a token with 8 fields
(too few) and one with 11 (too many); a token whose field 0 is a well-formed but
unaccepted schema version — `sk9`, or a real persisted `sk3` — which is the
**recognized-other-schema signpost** (`UnsupportedSchemaVersion`, naming the version,
§6.7-0015), *distinct* from the non-canonical leading-zero `sk04`, which is the
**malformed-field wall** (`BadVersionPrefix`); `sk4` is the current supported
version; an unknown dtype code
(`sk4|bin|f99|cuda:sm89|…`); an unknown op-family code (`sk4|zzz|f32|…`); an
over-`MAX_OPERANDS` operand field (9 sub-keys); a token exceeding
`MAX_STRUCTURE_KEY_LEN` (4096 bytes); an uppercase-hex mask (`…|x0A`, forbidden by
§6.7-0010); an unrecognized reduce-field spelling (`…|rmid`, `…|x` with no digits,
or an all-axes set spelled as a bitmask instead of the required `rall`, forbidden by
§6.6-0009 / §6.7-0005); a rank-1 reduction spelled `rlast` instead of the required
`rall` (the tiebreak of §6.6-0009); a non-`red` cell carrying a non-`-` reduce field
(forbidden by §6.6-0017); a `gem` cell omitting its mandatory contraction field
(forbidden by §6.6-0010) — under sk4 a **non**-`gem` cell's optional-trailing field is
the `(acc+mp)` field (§6.7-0013), so a contraction-shaped payload there is simply a
malformed `(acc+mp)` field, not a mis-placed contraction; a non-contraction `(acc+mp)`
field spelled in the forbidden **all-default** form (accumulator == compute dtype AND
`<mp>` default — redundant, §6.7-0013(d)) or malformed (not two `/`-parts, a bad
`<mp>`, or an out-of-set / reserved accumulator dtype); a collapsed (rank-reduced) reduction cell
(forbidden by §6.6-0009); and a `target_capability` with no colon (`cudasm89`), with
two colons
(`cuda:sm:89`), with an empty namespace (`:sm89`), and with an embedded field
separator (`cuda:sm|89`). Each yields a typed decline, never a panic (§6.7-0009,
§6.8-0001, §7.1-0002).

**A.3 Golden dtype table vector.** The twenty-four-row dtype table of §6.1 (token,
kind, bit width, packing) is itself a golden vector: per the §8-0005 freeze gate a
foreign reader reproduces every token spelling, bit width, and numeric kind
byte-for-byte, and reproduces the `i4`/`u4` nibble order, the `b1` LSB-first bit
order, and the `c64`/`c128` interleaved (re,im) layout with the real component in the
lower-addressed half.

**A.4 Provenance / acknowledgments.** The dtype set, operand descriptor, and
`structure_key` derive from the driver-free kernel-vocabulary seed crate
`baracuda-kernel-vocab` (an Evans Laboratories project), whose `ElementKind`,
`OperandDesc`, `StructureKey`, `structure_key()` derivation, and `to_token` /
`from_token` codec this sub-standard neutralizes: the CUDA-only `ArchSku` enum
(`Sm80`/`Sm89`/`Sm90a`) is replaced by the namespaced all-hardware
`target_capability` descriptor (§6.8), and every project/vendor name is confined to
this appendix and §0. Project names in this appendix and in §0 are non-normative
provenance and examples only; no normative clause names any project.

---

## Appendix B — Glossary (informative)

- **admissibility predicate** — a total function from the derivation inputs
  (operand descriptors + op category + target + role hints, §6.6-0012) to a
  `structure_key`; a kernel admits an invocation iff the derived key byte-matches
  (§6.6-0001).
- **capability-set** — the right component of `<namespace>:<capability-set>`; owned
  by the namespace maintainer (e.g. `sm89`, `gfx942`, `apple9`).
- **cell (specialization cell)** — one layout/dtype/target class a kernel is built
  for; named by exactly one `structure_key`.
- **dtype** — a scalar element type from the twenty-two-token set of §6.1; pure
  storage (byte layout only), never a compute-precision guarantee.
- **extent** — an axis's logical length (capacity for a symbolic axis).
- **inner-contiguous** — a layout tag: the innermost non-unit axis has `|stride| ==
  1` but outer axes are strided.
- **index width** — the offset-arithmetic width class (`idx32` / `idx64`); boundary
  `2³¹` elements (§6.5-0011).
- **iteration frame** — the widest-operand axis frame into which lower-rank operands
  are right-aligned (§6.6-0013).
- **layout_tag** — the derived per-operand memory-layout class (`contiguous`,
  `inner-contiguous`, `strided`, `broadcast`).
- **namespace** — the registered left component of a `target_capability`; registered
  by the steward.
- **numeric kind** — one of `float`, `int`, `uint`, `bool`, `complex`.
- **op_family_tag (op category)** — the coarse op category component of a
  `structure_key`; distinct from a KISS-Ops op name.
- **op name (KISS-Ops)** — the *semantic* identity of a computation; owned by
  KISS-Ops, **not** by KISS-Classify.
- **primary dtype** — operand-0's dtype; the `structure_key.dtype` field.
- **role hint** — a caller-supplied fact (M/N/K axis roles, index-operand slot) not
  derivable from bare extents (§6.6-0012).
- **stride** — a signed per-axis element step; `0` = broadcast, `< 0` = reversed.
- **structure_key** — the admissibility-predicate identity of a cell; carried
  opaquely as a token (max 4096 bytes).
- **target_capability** — the namespaced compilation-target descriptor
  `<namespace>:<capability-set>`, matched byte-exact.
- **token** — the stable string serialization of a `structure_key` (§6.7); the sole
  normative wire form.
- **typed decline** — a caller-observable error indication that leaves no partial
  output and never panics/aborts/crashes/hangs/reads out of bounds (§3, Terms).
- **work class** — the total-work size class (`one-warp` / `one-block` /
  `grid-stride`).

---

*End of KISS-Classify (Draft proposal). This sub-standard is foundational: it
depends on nothing, is referenced OPAQUELY by KISS-Announce and STRUCTURALLY by
KISS-Grammar / KISS-Contract / KISS-Consume / KISS-Emit, and MUST NOT depend on
KISS-Ops. Every binding requirement lives in an identified §6+ clause with a mapped
KISS-Conform test. Project and product names appear only in non-normative examples,
provenance, and reference-implementation pointers; normative clauses use only the
generic roles provider, consumer, implementation, kernel, contract, and target.*




