# Design: output layout forms in the Interface section (board 378, R1.13, KISS#316)

**Informative. No normative text; the PR that adds the clauses follows this note.** Read at KISS `origin/main` `d47b99e` (conformance 0.8.1); Unpopped at `origin/main` (`layout_algebra.rs`, fetched 2026-11-07).
CireSnave, board 378, verbatim: "378: B". As the PM summarized it: a **closed sum of seven layout forms in Interface 6.5, beside each output operand, not in Guarantees 6.8; absent means "the layout supplied"; an unknown form is a typed refusal of the guarantee.** This note settles the four open points the options note left (`docs/options-r1-13-output-layout-guarantee.md`) and gives the byte consequences.

## The seven forms

Each output operand's Interface entry MAY carry one `layout` value. **Absent = the layout the caller supplied** (today's meaning; no existing contract moves).

| # | Form | Spelling | Meaning |
|---|---|---|---|
| 1 | `dense` | `dense` | densely packed, canonical (row-major) order |
| 2 | `dense_order` | `dense_order(p0,p1,…)` | densely packed in an explicit axis order: `p` is a permutation of `0..rank-1`, slowest axis first (column-major of rank 2 is `dense_order(1,0)`) |
| 3 | `same_as` | `same_as(i)` | the strides of input operand `i` (FKC `same_as`) |
| 4 | `strided` | `strided(e0,e1,…)` | one stride per axis, each an expression in the §6.6-0006 grammar over the launch scalars (padded rows: `strided(ldc,1)`) |
| 5 | `hier` | `shape:stride` or `swz(B,M,S)(shape:stride)` | a CuTe-style nested layout, optionally composed with a swizzle (Unpopped `Layout`, `Swizzle`) |
| 6 | `bundle` | `bundle(slot,…)` | one buffer holding several named slots, each with its own layout (FKC `bundle`) |
| 7 | `none` | `none` | no guarantee: the reader must read the layout from the buffer |

Orthogonal and **not** part of the sum: aliasing (`in_place`, `accumulate`, Interface 6.5-0009 already) and who allocates (`preallocated`).
An **unknown form** (a spelling outside this table) is a typed refusal of the *guarantee* (the consumer treats the layout as `none` and records the refusal), never a refusal of the whole contract and never silent acceptance. The set is closed: a new form is a new clause, never a free-form string.

## Open point 1: the nested-layout text grammar

Start from Unpopped's `Display` (`layout_algebra.rs:37-43,205-220`), which emits no spaces: a `Layout` is `shape:stride`, an `IntTuple` is an integer or `(` item `,` item … `)`. So `((2,3),4):((1,2),6)` is the shape `((2,3),4)` with stride `((1,2),6)`.

```
layout   := tuple ":" tuple
tuple    := leaf | "(" tuple { "," tuple } ")"
leaf     := integer | expr        (expr only where "run-time symbol", below, allows it)
swizzled := "swz(" B "," M "," S ")" "(" layout ")"
```

Rules carried over from Unpopped (they are its `Layout::new` checks, which is what the round trip needs to agree on): shape and stride are **congruent** (same nesting profile, `NotCongruent` otherwise); a shape leaf is an integer `>= 1`; a stride leaf is `>= 0` (no negative strides, `NegativeStride`); a tuple is never empty; nesting depth at most `16` (`MAX_DEPTH`). The canonical text has **no spaces**, so two writers emit the same bytes; a reader MUST decline a form with a space, a leading `+`, a leading zero, or a non-canonical integer (one way to say one thing).
**Swizzle:** Unpopped's `Swizzle { bits, base, shift }` has **no `Display`**, so the `swz(B,M,S)(…)` spelling above is **my proposal, not Unpopped's**; it needs the Unpopped lane's confirmation and a round-trip test against its type before the clause is written. `B`, `M` are non-negative integers and `S` a signed integer (CuTe's `Swizzle<B,M,S>`).
**Round trip:** the vector set is `parse(print(x)) = x` over Unpopped's own layouts (a vector per shape category: flat, nested, broadcast `0`-stride, swizzled) and the declines above. Unpopped's `Display` already round-trips its `IntTuple`; the layout vectors would be produced from its type, not transcribed.

## Open point 2: a run-time symbol in a stride

Options: **(i)** integers only inside `hier`; a run-time stride is form 4 only (loses the combination of nesting and a run-time leading dimension, which a GEMM with a run-time `lda` wants). **(ii)** a `hier` **stride** leaf may be an integer or a §6.6-0006 expression over the launch scalars (`(M,N):(1,lda)`); a **shape** leaf stays an integer in the form and its extent is checked against the operand's declared extent at bind time. **(iii)** both stride and shape leaves may be expressions.
**Recommendation: (ii).** A run-time stride is what real kernels have; a run-time *shape* inside a nested layout is a second source of the same fact the Interface already states (the extents). The expression leaves make congruence and `>= 0` **bind-time** checks (`docs/design-r1-22-bind-time-obligations.md`): the contract is well formed, and a contract whose symbol resolves to a negative stride fails at bind, not at read. Writing the leaf as an expression in the text grammar costs one production (`expr`).

## Open point 3: bundles against several outputs

They are different things. **Several outputs** (Classify 6.6-0014; `sort_network` has two): several output operands, each its own buffer, each with its own `layout`. **A bundle** is **one** operand (one buffer) whose bytes are partitioned into slots: `bundle(a=dense@0, b=dense_order(1,0)@ceil_div(n*4,16)*16)` is a slot name, a layout of forms 1 to 5 (a slot is never itself a bundle), and a byte-offset expression (§6.6-0009 style, launch scalars). Slots MUST NOT overlap (a bind-time check when an offset has a symbol); the slot names are the bundle's identity and are unique within it.
**Recommendation:** keep both, and say in the clause that a kernel with several independent buffers uses several outputs and a kernel that hands back one packed buffer uses a bundle (the FKC section 5.5 case). I did not find a reason to unify them, and unifying would lose the single-allocation fact a bundle states.

## Open point 4: where it sits against `structure_key` and `revision_hash`

The guarantee is something the kernel **produces**, so by the admissibility rule (the key says what was asked for, the revision says what was produced) it is **not** a `structure_key` field: no key byte moves, Classify is unchanged, and a cache lookup is unaffected. It is Interface content (a contract field). `revision_hash` is, per Contract section 4, "an opaque, provider-assigned build identifier of a specific build": the Contract does not derive it from the contract bytes, so it cannot *carry* the guarantee; what the rule gives is that **a build whose guarantee differs is a different revision**, which is the provider's to assign. Stated for the Classify and Announce lanes to confirm; I do not read any clause that makes the guarantee part of a key, and I propose none.

## Byte consequence, readers, version

- **New:** an optional `layout` field on each output operand's Interface entry, with the seven spellings and the grammar above; a typed refusal for an unknown form. **No existing contract moves** (absent = the supplied layout).
- **Readers:** every Contract reader parses the optional field and the grammar; a reader that only wants the old behavior ignores an absent field and **refuses** (does not ignore) a present one it does not know. Fuel's FKC `layout_guarantee` maps `contiguous` to `dense`, `preallocated` to absent (always true), `same_as(x)` to `same_as(i)`, `bundle` to `bundle(…)`. Unpopped states the full `Layout` it writes through form 5.
- **Vectors:** a golden Contract per form; the hierarchical round trip; the declines (unknown form, a space, a non-congruent pair, a negative stride, depth 17, an overlapping bundle).
- **Version:** Breaking for Contract readers (a new field they must refuse when unknown); proposed for the 0.9.0 batch; not stamped. Sized as the largest of the batch: new Interface clauses, a text grammar, and a Unpopped-confirmed swizzle spelling. If it is not ready when the others are, it ships in 0.10.0 under the PM's cap.

## What I need before clauses

(1) The swizzle spelling confirmed by the Unpopped lane (and a `Display` for `Swizzle` there, or my spelling adopted); (2) CireSnave on open point 2 (recommend (ii)); (3) the Classify/Announce lanes (both mine) confirming open point 4; (4) the date to cut it, which I will propose to the PM after sizing.
