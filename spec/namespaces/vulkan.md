# The `vulkan:` capability-set vocabulary

**Namespace:** `vulkan` · **Maintainer:** [vulkane](https://github.com/ciresnave/vulkane)
· **Vocabulary version:** 4 · **Status:** draft

**This is a maintainer-owned annex, not a KISS clause.** KISS-CLASSIFY-6.8-0004
assigns each namespace's capability-set vocabulary to that namespace's
maintainer and requires that *KISS clauses pin only the token grammar and the
byte-exact match rule, never a specific namespace's capability-set vocabulary*.
This document is therefore normative **for the `vulkan:` namespace** and is
referenced from the namespace registry
([`conformance/registry/namespaces.json`](../../conformance/registry/namespaces.json)),
but it is not part of KISS-Classify's clause set and it freezes on its own axis
(§8). Nothing here may contradict §6.8-0001, -0002, -0005, or the general
encoding rules; where it appears to, the KISS clause wins.

Reference implementation:
[`kiss-vulkan-vocab`](https://github.com/ciresnave/vulkane/tree/main/kiss-vulkan-vocab)
(dependency-free, no Vulkan linkage, per §6.9-0003). Deriving a token from a
live device is a separate concern and lives in `vulkane::kiss`.

---

## 1. What the token names

A `vulkan:` token names **the specialization a kernel was built for**, not the
capability envelope of a device that could run it.

This follows from what a `target_capability` is for: it sits inside a
`structure_key`, which identifies a specialization *cell* — a kernel artifact.
A device advertising a pinnable subgroup range of 32..=64 can host a
wave32-pinned kernel and a wave64-pinned kernel, and those are different
binaries with different performance. A token naming the envelope would give
both the same bytes and collide two cells that are not interchangeable. This is
the same convention `cuda:sm89` follows: it names what a kernel was compiled
*for*, not the maximum capability of the part executing it.

Two consequences:

- **A device admits a set of tokens rather than having one.** A deriver's
  honest shape is `(device, choice) -> token`, not `device -> token`.
- **The consumer chooses first, then matches.** Because §6.8-0002 forbids
  subset, prefix, and feature-implication logic, a consumer holding a
  32..=64-capable device may **not** look up a `sg32` kernel by reasoning that
  its envelope contains 32. It decides it is building a wave32 cell, spells
  that token, and matches byte-exactly. Choice policy lives in the consumer;
  the vocabulary is a pure identity.

## 2. Grammar

```text
vulkan:<subgroup>.<ops>.<arith>.<coop>.<coopvec>
```

- **V-1.** A `vulkan:` capability-set MUST consist of exactly five fields
  separated by `.`, in the order above. No field may contain a `.`. Every field
  is always present; there are no optional or omissible fields, because an
  omission would be a second spelling of some target.
- **V-2.** Parts *within* a field are separated by `-`. Where a field carries a
  list, list items are separated by `,`.
- **V-3.** Every set is emitted in the canonical order given below.
  Legal-but-non-canonical input — an unsorted set, a duplicate member, a
  leading zero, uppercase hex — MUST be **rejected with a typed decline**, not
  normalized. Under §6.8-0002 two accepted spellings of one target would fail
  to match each other, which is worse than accepting neither.

### 2.1 `<subgroup>` — the width the kernel is built for

| Spelling | Meaning |
|---|---|
| `sg<N>` | pinned to width `N`, a power of two, no leading zeros |
| `sgdyn` | width-agnostic |

- **V-4.** `sgdyn` names a kernel that reads the subgroup width at runtime
  (`gl_SubgroupSize` / `WaveGetLaneCount()`) and is correct at any width. It is
  a distinct cell from any `sg<N>`: a width-agnostic binary and a
  width-pinned one are different artifacts with different performance. Without
  this spelling every width-agnostic kernel would have to be labelled with an
  arbitrary concrete width and would collide with the pinned variant of itself.

### 2.2 `<ops>` — subgroup operation classes required

`ops-<letters>`, or `ops-none`. Letters are single ASCII characters, emitted in
ascending order, each at most once:

| Letter | Class | | Letter | Class |
|---|---|---|---|---|
| `a` | arithmetic | | `q` | quad |
| `b` | basic | | `r` | shuffle-relative |
| `c` | clustered | | `s` | shuffle |
| `l` | ballot | | `t` | rotate-clustered |
| `p` | partitioned (NV) | | `v` | vote |
| | | | `w` | rotate |

- **V-5.** This field juxtaposes without separators. That is permitted because
  its alphabet is **fixed-width** — one ASCII character per member — and a
  fixed-width alphabet is uniquely decodable by construction, for every future
  member, without anyone having to check. See the general encoding rule in
  KISS-Classify §6.8.

### 2.3 `<arith>` — arithmetic capabilities required

`arith-<name>[-<name>…]`, or `arith-none`. Names are emitted in ascending
lexicographic order, each at most once:

| Name | Meaning |
|---|---|
| `dot8` | any accelerated 8-bit integer dot product (`VK_KHR_shader_integer_dot_product`) |
| `f16` | `shaderFloat16` — half-precision *arithmetic* |
| `i8` | `shaderInt8` — 8-bit integer *arithmetic* |
| `st16` | `storageBuffer16BitAccess` |
| `st8` | `storageBuffer8BitAccess` |

- **V-6.** This field uses explicit `-` separators and MUST NOT juxtapose. Its
  names are variable-length, so juxtaposition would be safe only while the set
  remained uniquely decodable as it grows — a property no one checks by eye and
  which nothing currently guarantees. Compute precision and *storage* precision
  are separate members because they are separate capabilities: a device may
  accept 16-bit data in a storage buffer while performing the arithmetic in
  f32.

### 2.4 `<coop>` — cooperative-matrix shapes used

One of:

| Spelling | Meaning |
|---|---|
| `cm-none` | the kernel uses no cooperative-matrix operations |
| `cm-<tuple>[,<tuple>…]` | the exact shapes it uses, canonically ordered |
| `cm-fnv1a64-<hex16>` | a digest, when the enumeration is too long (V-9) |

A tuple is `<M>-<N>-<K>-<A>-<B>-<C>-<R>` optionally suffixed `-sat` for
saturating accumulation, where `M`/`N`/`K` are decimal with no leading zeros
and each component type is one of `f16`, `f32`, `f64`, `bf16`, `i8`, `i16`,
`i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f8e4m3fn`, `f8e5m2`, `i8packed`,
`u8packed`, or `x<n>` for a `VkComponentTypeKHR` this vocabulary version does
not name.

> **Signed integers are `i`-prefixed as of vocabulary version 2** (was `s8`,
> `s16`, `s32`, `s64`). See §4. Note that the `i8` in the **arith** field is a
> different thing that has always been spelled `i8`: it names the `shaderInt8`
> *capability*, not a component type.

> **FP8 is named as of vocabulary version 3.** `f8e4m3fn` spells
> `VK_COMPONENT_TYPE_FLOAT8_E4M3_EXT` and `f8e5m2` spells
> `VK_COMPONENT_TYPE_FLOAT8_E5M2_EXT`. Both previously derived `x1000491002` /
> `x1000491003` under V-7, which is why naming them bumps the version — see §4,
> where this is the worked example.

- **V-7.** `x<n>` exists so that a driver exposing a component type newer than
  this vocabulary yields an honest, round-trippable token rather than a decline
  or a silent mis-spelling. New Vulkan component types appear faster than a
  vocabulary revision can track them.
- **V-8.** Tuples are ordered by `(M, N, K, A, B, C, R, saturating)` ascending
  and deduplicated. Driver-reported order is **not** stable across drivers or
  even across calls on one driver, so a producer that emitted them as reported
  would emit a token that varies run to run — and under byte-exact matching
  every cache lookup would then miss. Ordering is imposed by the producer, not
  trusted from the source.
- **V-9.** The **canonical enumeration string** is the comma-joined tuple list
  *excluding* the `cm-` prefix. If its length is `<= 512` bytes it is emitted
  inline; if `> 512` it is replaced by `cm-fnv1a64-<hex16>`, the digest of
  **that same string**. The switch is a hard, deterministic function of that
  byte count and MUST NOT be an implementation preference: two honest producers
  that disagreed about which form to emit would produce different tokens for
  one target. Hashing the same string that is measured means a producer can
  disagree about *whether* to hash but never about *what* is hashed.

  *Rationale (informative).* 512 is `2^9`, an eighth of `MAX_STRUCTURE_KEY_LEN`,
  reserving the rest for the op-family, dtype, and operand-descriptor fields so
  a target can never crowd out the operand data that makes a `structure_key`
  useful. At roughly 22 bytes per tuple it admits about 23 shapes inline; a
  measured AMD RDNA part reports 11, encoding to 281 bytes. The specific number
  is a policy choice — what matters is that it is pinned and identical
  everywhere.

- **V-10.** The `fn` suffix on `f8e4m3fn` is **mandatory and load-bearing**,
  not decoration. `e4m3` alone does not identify a format: the OCP OFP8 variant
  is finite-only (no infinities, one NaN encoding, max magnitude 448), while the
  `fnuz` variant differs in both NaN handling and exponent bias. The layouts
  are pinned normatively by KISS-OPS-6.16-0004 and -0005; this vocabulary only
  spells them.

  A deriver **MUST NOT** emit `f8e4m3fnuz` or `f8e5m2fnuz` for any
  `VkComponentTypeKHR`. KISS reserves those two spellings with *no computation
  semantics at all* (KISS-CLASSIFY-6.1-0001: recognized on parse, answered with
  a typed decline), and Vulkan exposes no enumerant for either — so a token
  carrying one would claim a device computes in a format the spec says has no
  semantics at this schema version.

  This is what makes the mapping determinable rather than a guess. `vk.xml`
  itself says nothing about which FP8 variant its enumerants denote; the
  determination is that the two reserved spellings cannot name a type a device
  actually computes with, which leaves exactly one coherent target for each of
  Vulkan's two FP8 values.

- **V-11.** `VK_COMPONENT_TYPE_FLOAT_E4M3_NV` and `..._FLOAT_E5M2_NV` are
  registry **aliases** of the two EXT enumerants, not distinct values, so they
  spell identically and need no separate names. Recorded because a future
  registry that split them would silently make an NV-only driver derive `x<n>`
  again.

### 2.5 `<coopvec>` — cooperative-vector combinations used

One of:

| Spelling | Meaning |
|---|---|
| `cv-none` | the kernel uses no cooperative-vector operations |
| `cv-<tuple>[,<tuple>…]` | the exact combinations it uses, canonically ordered |
| `cv-fnv1a64-<hex16>` | a digest, when the enumeration is too long |

A tuple is `<input>-<inputInterp>-<matrixInterp>-<biasInterp>-<result>`,
optionally suffixed `-t` when the combination transposes the matrix operand.
Each of the five positions is a component type from the same set §2.4 lists,
including the `x<n>` escape.

- **V-12.** Cooperative vector is a **separate capability from cooperative
  matrix**, and this is a separate field rather than a corner of `<coop>` for a
  structural reason, not a stylistic one. A cooperative-matrix tuple is
  `M-N-K` plus four component types; a cooperative-vector tuple is five
  component types and a flag, with no dimensions at all. A field whose parts are
  separated by `-` cannot carry both arities without becoming ambiguous, and an
  ambiguous field is one two producers can spell differently.

  They are also reported by different queries —
  `vkGetPhysicalDeviceCooperativeMatrixPropertiesKHR` and
  `vkGetPhysicalDeviceCooperativeVectorPropertiesNV` — so a deriver that read
  only the first could never observe the second at all.

- **V-13.** Tuples are ordered by
  `(input, inputInterp, matrixInterp, biasInterp, result, transpose)` ascending
  and deduplicated, and the length-triggered digest rule of V-9 applies
  unchanged: the threshold is measured over this field's own canonical
  enumeration string, and the digest runs over that same string. The two
  cooperative fields switch independently — each is measured on its own bytes —
  because a shared threshold would make one field's length change the other
  field's spelling.

- **V-14.** `i8packed` and `u8packed` spell
  `VK_COMPONENT_TYPE_SINT8_PACKED_NV` and `VK_COMPONENT_TYPE_UINT8_PACKED_NV`.
  They **MUST NOT** be folded onto `i8` / `u8`: the packed layout is a
  different shader-side contract, and collapsing them would let a packed-only
  device satisfy a target asking for the unpacked type.

  These are reachable **only** through the cooperative-vector query. The
  enumerants are defined by `VK_NV_cooperative_vector` with no dependency on
  `VK_KHR_cooperative_matrix`, which is why naming them had to wait for a
  deriver that reads it — before that they were spellable and underivable, and
  a name nothing can emit is worse than an honest `x<n>`.

  Verified rather than assumed: an NVIDIA RTX 4070 reports 16 cooperative-vector
  combinations whose component values include `1000491000` and `1000491001`,
  while the same device's cooperative-matrix properties contain neither.

## 3. Worked example

An AMD Radeon 610M (RDNA, default width 64, pinnable 32..=64, 11
cooperative-matrix shapes) admits three specializations, and therefore three
tokens. At `sg64`, with every capability the device offers:

```text
vulkan:sg64.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f16-f16,16-16-16-f16-f16-f16-f16-sat,16-16-16-f16-f16-f32-f32,16-16-16-i8-i8-i32-i32,16-16-16-i8-i8-i32-i32-sat,16-16-16-i8-u8-i32-i32,16-16-16-i8-u8-i32-i32-sat,16-16-16-u8-i8-i32-i32,16-16-16-u8-i8-i32-i32-sat,16-16-16-u8-u8-i32-i32,16-16-16-u8-u8-i32-i32-sat.cv-none
```

343 bytes, 8.4% of `MAX_STRUCTURE_KEY_LEN`. The `sg32` and
`sgdyn` tokens differ only in the first field. A kernel using less than all of
this spells its own narrower token — the token names what the **kernel**
requires, and over-claiming fragments cells that could otherwise be shared.

This part has **no** cooperative-vector support, which is why its fifth field is
`cv-none` — measured, not assumed: the same machine's RTX 4070 reports 16
combinations while this 610M reports zero. The eight bytes `cv-none` adds to a
device that gains nothing from the field are the cost of V-1 being a fixed arity
with no omissible fields, and are why version 4 invalidates every cached token
rather than only those of cooperative-vector devices.

A device that *does* support it spells the field out. The 4070's 16 combinations,
canonically ordered, are 331 bytes — comfortably inline, and the packed types
appear exactly where V-14 says they can:

```text
cv-f16-f16-f16-f16-f16-t,f16-f8e4m3fn-f8e4m3fn-f16-f16,f16-f8e5m2-f8e5m2-f16-f16,f32-i8-i8-i32-i32,i32-i8-i8-i32-i32,i32-i8-u8-i32-i32,i32-u8-i8-i32-i32,i32-u8-u8-i32-i32,i8-i8-i8-i32-i32,i8-i8-u8-i32-i32,u32-i8packed-i8-i32-i32,u32-i8packed-u8-i32-i32,u32-u8packed-i8-i32-i32,u32-u8packed-u8-i32-i32,u8-u8-i8-i32-i32,u8-u8-u8-i32-i32
```

Worth reading closely, because it is not what one would guess: the packed types
occur in **`inputInterpretation`** against a `u32` *input* — packed data arrives
as 32-bit words and is *interpreted* as four 8-bit values — never in the input
or result position. A vocabulary that had assumed packed types behave like
element types would have put them where they never appear.

## 4. Versioning

This vocabulary versions independently of `STRUCTURE_KEY_VERSION` and of
KISS-Classify's crate semver (§8).

**The rule is one sentence: any change that alters the bytes of a
previously-derivable token bumps the version.** A field reorder, a respelling,
a canonical-order change, or a change to the 512 threshold all qualify.

Adding a *name* — a component type, an op-class letter, an arith name — is
additive **only when no prior token could have been affected by its absence.**
That is narrower than it sounds, and the two ways it fails are the reason this
clause is spelled out rather than left to judgment:

- **Component types have the `x<n>` escape (V-7).** A device exposing a
  component type this vocabulary does not name derives `x<n>` for it *today*.
  Naming it later changes that device's token from `x1000491002` to
  `f8e4m3fn` — different bytes, **unchanged hardware, unchanged kernel**.
- **Op-class letters and arith names have no escape at all.** An unrecognized
  capability is simply absent from the field, so the token under-claims what
  the device offers. Adding the name later *adds a character* to the token of
  every device that already had the capability. Same byte change, and with no
  `x<n>` to hint that something was elided.

### The additive test

Compare against the **registry baseline** recorded for the previous vocabulary
version (below).

> If the underlying Vulkan enumerant or feature bit was **already assigned in
> the Vulkan registry at that baseline**, some conformant device could have
> reported it, so some derivable token could already have been affected by its
> absence → **naming it bumps the version.**
>
> If it was assigned only **after** that baseline, no prior token could have
> contained or omitted it → **additive, no bump.**

Mechanically checkable, and it preserves V-7's forward-compatibility intent: a
driver reporting a component type this vocabulary does not name still derives a
token for it, spelled `x<n>`, rather than the deriver refusing to produce one.
What the test closes is the separate case where *naming* such a type later
silently invalidates tokens already in the wild.

Over-bumping is the safe direction: a version bump on an addition nobody was
affected by costs a cache flush, whereas a missed bump costs silent
non-matching that no consumer can attribute.

### Registry baselines

Each vocabulary version records the `VK_HEADER_VERSION` it was authored
against, so the test above has a fixed thing to compare with rather than
"whatever the registry looked like at the time".

| Vocabulary version | `VK_HEADER_VERSION` baseline |
|---|---|
| 1 | 348 — reconstructed, see below |
| 2 | 348 |
| 3 | 348 |
| 4 | 348 |

**Version 4 is a grammar change, not only an addition**, and it is the most
expensive kind this vocabulary can make: adding the fifth field changes the
bytes of **every** token, including those of devices with no cooperative-vector
support — they gain `cv-none`. That is a full cache invalidation for every
consumer, not the partial one FP8 caused.

It is bundled here deliberately. Vocabulary versions 2 and 3 are **unpublished**
at the time of writing, so their invalidation has not been paid yet; folding the
grammar change in makes it one event rather than a second one after v3 ships.
The packed component types are included for the same reason — their enumerants
were already assigned at baseline 348, so naming them at any later point would
force a further bump of its own.

**Worked example — how version 3 was decided.** Version 3 names `f8e4m3fn` and
`f8e5m2`. Applying the additive test to them is the first time it has been run
on a real change, so the result is recorded here rather than left to be
re-derived:

> v2's recorded baseline is `VK_HEADER_VERSION` **348**. In the registry at 348,
> `VK_COMPONENT_TYPE_FLOAT8_E4M3_EXT` is already assigned — value `1000491002`,
> allocated from extension 492's block, required by `VK_KHR_cooperative_matrix`.
> A conformant device could therefore already have reported it, and any token
> derived from such a device already spelled `x1000491002`. **Already assigned
> at the baseline → naming it bumps the version.**

Note that this is the same case §4 uses to *illustrate* why naming is not
automatically additive, which is a coincidence worth flagging rather than
hiding: the illustration was written before the change it describes was made,
and the test then returned the answer the illustration predicted. That is
weak evidence the test is well-formed and no evidence at all that it is
correctly applied — the number `1000491002` was checked against a vendored
`vk.xml` at header 348, not taken from the prose.

Version 1 pinned no baseline, so its entry is a reconstruction rather than a
record: 348 is the header version the reference implementation shipped against,
which is **at or later than** v1's true baseline. That errs toward *over*-bumping
— a value assigned between v1's real baseline and 348 is classified "already
assigned", so naming it bumps when it might strictly have been additive. That
is the safe direction, and the cost is a cache flush nobody needed rather than a
silent non-match nobody can attribute. From v2 onward the baseline is recorded
at authoring time and no reconstruction is required.

### Version 2 — signed-integer component types are `i`-prefixed

`s8`, `s16`, `s32`, `s64` become `i8`, `i16`, `i32`, `i64`. This is a
**respelling**, so it bumps the vocabulary version by the rule above, and every
previously-derivable token naming a signed-integer component type changes
bytes. Under §6.8-0002 matching is byte-exact, so a version-1 token and its
version-2 equivalent **do not match**: any cache keyed on a version-1 token is
stale and must be invalidated, not migrated. Token *length* is unchanged, since
each name keeps its width.

Adopted to align with the KISS-Classify §6.1 `structure_key` dtype set, which
uses the `i` prefix, as part of the coordinated `sk4` schema event
(maintainer-ratified, 2026-08-08). §6.1's set and this vocabulary remain
**distinct axes** — a decoder never resolves a `target_capability` token
against the dtype set — so this is an alignment of convention, not a merge of
vocabularies.

Two things this change deliberately does **not** touch, recorded because both
look like oversights and are not:

- **The `arith` field keeps `i8`.** It has always been `i8`, and it names the
  `shaderInt8` *capability*, not a component type. It is unrelated to this
  rename and must not be "made consistent" with anything.
- **Unsigned types keep `u`.** `u8`/`u16`/`u32`/`u64` already match §6.1.

For the record, since it will otherwise read as an inconsistency worth
"fixing": version 1 spelled component types `s8`/`s32` because they transcribe
Vulkan's own `VK_COMPONENT_TYPE_SINT8_KHR`/`SINT32`, exactly as `arith`'s `i8`
transcribes `shaderInt8`. Each name tracked its Vulkan source. Version 2 trades
that correspondence for one prefix convention across both KISS vocabularies.
That was a deliberate maintainer decision, not a correction — do not revert it
on the grounds that `SINT8` says "s".
