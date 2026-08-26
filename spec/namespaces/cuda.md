# The `cuda:` capability-set vocabulary

**Namespace:** `cuda` · **Maintainer:** [baracuda](https://github.com/ciresnave/baracuda) · **Vocabulary version:** 1 · **Status:** draft

> **This is a maintainer-owned annex, not a KISS clause.** Under
> [KISS-CLASSIFY-6.8-0004](../classify.md), the `cuda` namespace's capability-set
> vocabulary is owned by its maintainer (baracuda); KISS pins only the token
> *grammar* and the byte-exact match rule, never this vocabulary. This document is
> normative for the `cuda:` namespace only, is referenced from
> [`conformance/registry/namespaces.json`](../../conformance/registry/namespaces.json),
> is **not** part of KISS-Classify's clause set, and freezes on its own axis (§8 of
> `classify.md`). Where it appears to contradict §6.8-0001 / -0002 / -0005 or the
> general encoding rules, **the KISS clause wins.**

**Reference implementation:** [`unpopped-vocab`](https://github.com/EvansLaboratories/Unpopped)
— the dependency-free crate that derives the `structure_key` and emits `cuda:`
tokens through the real codec. Per KISS-CLASSIFY-6.9-0003 it links no backend; the
CUDA emitter that *consumes* these tokens lives separately (`unpopped-cuda`).

---

## 1. What the token names

A `cuda:` token names **the specialization a kernel was built for** — the CUDA
compute capability it was compiled against — not the maximum capability of the
part it will run on. An RTX 4070 (an `sm_89` part) that also runs `sm_80`-targeted
kernels as a forward-compatible fallback therefore *admits a set* of tokens:
`{ cuda:sm89, cuda:sm80 }`. The producer picks the token for the specialization it
actually built; the consumer matches it **byte-exact** (§6.8-0002 — no subset,
prefix, or implication). `cuda:sm80` and `cuda:sm89` are different cells, not a
range.

## 2. Grammar

```text
cuda:sm<N>[<letter>]
```

- **C-1** The token is exactly the namespace `cuda`, a single `:`, and one
  capability-set field. There are **no `.`-separated fields** (unlike the `vulkan:`
  grammar) — see §4.
- **C-2** The capability-set is `sm` followed by the CUDA **compute-capability
  integer** `<N>` — the full decimal value, **two or more digits**, no leading zeros
  (`80`, `89`, `90`, `100`) — optionally followed by a **single lowercase ASCII
  letter** `<letter>` denoting an architecture-specific / accelerated target (e.g.
  the `a` in `sm90a`, or the `100a` in `sm100a`). `<N>` is not fixed-width: three-
  digit capabilities such as `sm100` are admitted.
- **C-3** Every character is lowercase ASCII drawn from `[a-z0-9]` plus the one `:`
  separator. A token MUST NOT contain the `structure_key` field separators `|`,
  `;`, `/`, any whitespace, or any control byte (KISS-Classify §6.8-0005). Matching is byte-exact
  and therefore **case-sensitive**: `cuda:sm89`, `Cuda:sm89`, `cuda:SM89`, and
  `cuda:sm89x` are four different tokens.

Spellings currently in use (illustrative, not closed — the grammar above is the
authority):

| Token | Compute capability | Architecture |
|---|---|---|
| `cuda:sm80` | 8.0 | Ampere (also the forward-compatible fallback on Ada / Hopper) |
| `cuda:sm89` | 8.9 | Ada Lovelace (FP8 tensor cores) |
| `cuda:sm90` | 9.0 | Hopper |
| `cuda:sm90a` | 9.0 (arch-specific) | Hopper, accelerated features |
| `cuda:sm100a` | 10.0 (arch-specific) | Blackwell |

## 3. Worked example

A kernel compiled for Ada Lovelace with FP8 paths carries the target-capability
field `cuda:sm89`, embedded in the `structure_key` token as a single field
(schematic — the schema-version prefix is illustrative; the `cuda:` field is
independent of it):

```text
sk4|bin|f32|cuda:sm89|ix32|grid|r1|…|-
```

The `cuda:sm89` field is a handful of bytes — a negligible fraction of
`MAX_STRUCTURE_KEY_LEN`. A sibling built for Hopper differs from it in exactly one
field (`cuda:sm90a`) and is therefore a different cell.

## 4. The capability-set is a single scalar

A `cuda:` capability-set is a **single atomic scalar** — one `sm<N>[<letter>]`
token. It is **not** a `.`-separated multi-field grammar (as `vulkan:` is), and it
carries **no concatenated member list and no range**. Two consequences follow
directly:

- The **fixed-width juxtaposition** rule (KISS-Classify §6.8-0006) is **vacuous** here: nothing
  is juxtaposed, so there is no member alphabet whose width must be fixed.
- The **digest fallback** (KISS-Classify §6.8-0007) is **structurally unreachable**: a digest
  replaces a long enumeration, and a single scalar is never long enough — there is
  no enumeration to digest. A `cuda:` token is always its literal spelling.

This is what makes `cuda.md` a much shorter annex than `vulkan.md`, and it is
backed by a test rather than left to prose (`unpopped-vocab`'s
`cuda_tokens_are_single_scalar` cites this section).

## 5. Versioning

This vocabulary versions **independently** of `STRUCTURE_KEY_VERSION` and of the
`unpopped-vocab` crate's semver (§8 of `classify.md`). Adding a new SKU that §2's
grammar already admits (a new `sm<N>[<letter>]` spelling) is **additive** and does
not bump the vocabulary version — an existing token's bytes are unchanged. Changing
the grammar itself (the token shape, the charset) is **byte-altering** and bumps
the vocabulary version.

## Appendix: machine-readable capability set (SSOT seed)

The rows below are the single source the `cuda:` maintainer's generators consume
directly, so the SKU set is not hand-transcribed into downstream code. KISS's
conformance suite does **not** read this table — it treats `cuda:` tokens as opaque
bytes and matches them byte-exact (§6.8-0002); only the maintainer's generators
consume it. Format: tab-separated `sku` (the Rust `ArchSku` variant), `token`,
`arch`, `notes`.

```tsv
sku	token	arch	notes
Sm80	cuda:sm80	ampere	forward-compatible fallback on Ada/Hopper
Sm89	cuda:sm89	ada	FP8 tensor cores; requires the sm89 feature
Sm90	cuda:sm90	hopper	base Hopper; no arch-specific feature required (`cuda:sm90a` is a different cell, §2)
Sm90a	cuda:sm90a	hopper	accelerated features; requires the sm90a feature
```

> The rows above are the SKUs the reference emitter wires today (the `ArchSku`
> enum's variants). `cuda:sm100a` is valid under §2's grammar and appears in
> illustrative examples, but is not yet an `ArchSku` variant; a row is
> added here (with its variant) when the emitter wires that arch — deliberately a
> breaking-change event for the exhaustive-match kernel dispatchers, per the
> `ArchSku` design (it is intentionally not `#[non_exhaustive]`).
>
> `cuda:sm90`'s row was added 2026-08-26 under that rule, after the emitter had
> already wired `ArchSku::Sm90` (`unpopped-vocab` `layout.rs:59`, `target.rs:309`).
> **The rule fired correctly and nothing checked it:** the seed under-reported the
> vocabulary by one token for weeks, while §2's table above listed four and
> `ArchSku::Sm90`'s own doc comment says it was *"Added because KISS names `cuda:sm90` in
> its §6.7 reference vectors"* — the structure_key reference vectors of **KISS-Classify
> §6.7**, resolved here because a bare `§6.7` is ambiguous across seven sub-standards
> (the quoted identifier is preserved as the maintainer wrote it; the resolution is ours).
> So two artifacts each cited the other and disagreed about the same token. See #334.
