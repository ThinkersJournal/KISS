# The `vulkan:` capability-set vocabulary

**Namespace:** `vulkan` · **Maintainer:** [vulkane](https://github.com/ciresnave/vulkane)
· **Vocabulary version:** 2 · **Status:** draft

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
vulkan:<subgroup>.<ops>.<arith>.<coop>
```

- **V-1.** A `vulkan:` capability-set MUST consist of exactly four fields
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
`i32`, `i64`, `u8`, `u16`, `u32`, `u64`, or `x<n>` for a `VkComponentTypeKHR`
this vocabulary version does not name.

> **Signed integers are `i`-prefixed as of vocabulary version 2** (was `s8`,
> `s16`, `s32`, `s64`). See §4. Note that the `i8` in the **arith** field is a
> different thing that has always been spelled `i8`: it names the `shaderInt8`
> *capability*, not a component type.

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

## 3. Worked example

An AMD Radeon 610M (RDNA, default width 64, pinnable 32..=64, 11
cooperative-matrix shapes) admits three specializations, and therefore three
tokens. At `sg64`, with every capability the device offers:

```text
vulkan:sg64.ops-abclqrstvw.arith-dot8-f16-i8-st16-st8.cm-16-16-16-f16-f16-f16-f16,16-16-16-f16-f16-f16-f16-sat,16-16-16-f16-f16-f32-f32,16-16-16-i8-i8-i32-i32,16-16-16-i8-i8-i32-i32-sat,16-16-16-i8-u8-i32-i32,16-16-16-i8-u8-i32-i32-sat,16-16-16-u8-i8-i32-i32,16-16-16-u8-i8-i32-i32-sat,16-16-16-u8-u8-i32-i32,16-16-16-u8-u8-i32-i32-sat
```

335 bytes, 8.2% of `MAX_STRUCTURE_KEY_LEN`. The `sg32` and `sgdyn` tokens differ
only in the first field. A kernel using less than all of this spells its own
narrower token — the token names what the **kernel** requires, and over-claiming
fragments cells that could otherwise be shared.

## 4. Versioning

This vocabulary versions independently of `STRUCTURE_KEY_VERSION` and of
KISS-Classify's crate semver (§8). Adding a component-type name, an op-class
letter, or an arith name is **additive** and does not bump the vocabulary
version. Any change that alters the bytes of a previously-derivable token — a
field reorder, a respelling, a canonical-order change, or a change to the 512
threshold — bumps it.

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
