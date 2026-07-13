# KISS-Ops — The Op Vocabulary & Per-Op Numeric Semantics

**Sub-standard ID:** KISS-OPS
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Umbrella
> Specification*: an **informative Overview** (§0–§5) and a **normative Conformance
> specification** (§6+). Only §6+ is normative. Normative clauses use RFC-2119 /
> RFC-8174 uppercase keywords, carry an append-only clause ID
> `KISS-OPS-<section>-<nnnn>`, and each MUST/SHALL maps 1:1 to at least one named
> KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test.

---

## Abstract

KISS-Ops is the foundational sub-standard that owns the vocabulary describing
**computation**: the op set, each op's pinned numeric semantics (NaN propagation,
signed-zero behavior, IEEE-`fmax` versus NaN-propagating `max`, wrapping versus
saturating integer behavior, raw-bit `select`, rounding mode, and every other
edge case), each non-primitive op's **reference decomposition** into strictly-
lower-level ops, and the mandatory **primitive floor** every consumer must
understand. It is the concrete home of the opaque-hub op-semantics currency — the
meaning of each op — consumed structurally by KISS-Grammar, KISS-Contract,
KISS-Synth/Provision, KISS-Consume, and KISS-Emit, and the termination guarantee
for recursive contract resolution. KISS-Ops standardizes what an op *means*, pinned
to bits and IEEE-754 semantics, not any one language's surface spelling. So that it
is resolvable by a foreign reader holding only this document plus the umbrella, the
bit layout of every dtype to which KISS-Ops assigns numeric meaning is inlined
normatively in §6.16.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Ops |
| Sub-standard ID | KISS-OPS |
| Maturity stage | **Draft** (first-draft proposal; op set and pinned semantics NOT frozen) |
| Editor of record | **Proposed, pending ratification** — the governance record does not yet name an editor for KISS-Ops; the editor holds the pen and requests comment from interested cosignatories (projects building consumers, providers, emitters, or lifters over the op vocabulary). |
| Steward | ThinkersJournal |
| Reference seed crate(s) | `baracuda-kernel-vocab`, `baracuda-kernelgen` (project/crate names given in Appendix C as non-normative provenance) |
| DAG position | **Foundational tier (computation vocabulary).** Sits at the bottom of the suite DAG beside the data vocabulary. Depends on **NOTHING**. Not consumed opaquely — every downstream edge is STRUCTURAL. |
| Upstream edges | **None** (foundational root; KISS-Ops depends on no other sub-standard). |
| Downstream edges | KISS-Grammar (**STRUCTURAL** — advertisable ops re-base on KISS-Ops op names); KISS-Contract (**STRUCTURAL** — the contract semantics field carries the KISS-Ops op DAG); KISS-Synth/Provision (**STRUCTURAL** — the resolved decomposition is the oracle); KISS-Consume (**STRUCTURAL** — lift targets are KISS-Ops op names); KISS-Emit (**STRUCTURAL** — lowering sources are KISS-Ops op definitions). KISS-Conform depends on and tests this sub-standard. |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |
| Maturity | Draft proposal |

> **Edge-label note (informative).** Every KISS-Ops downstream edge is **STRUCTURAL**:
> a dependent parses the internal structure of an op definition (its name, family,
> decomposition, and pinned semantics). This is the opposite of an OPAQUE edge (where
> a token is carried without interpretation). The labels reconcile with the umbrella
> §2.2 edge table, which lists KISS-Ops → Grammar / Contract / Synth / Consume / Emit
> each as STRUCTURAL. KISS-Ops has **no** incoming edge and in particular does **not**
> depend on KISS-Classify; the two foundational vocabularies are siblings, neither
> importing the other. Because a foreign reader must nonetheless be able to compute a
> constant's bit pattern and evaluate every float clause, the bit layout of each dtype
> KISS-Ops assigns semantics to is inlined here (§6.16) rather than deferred.

---

## 1. Purpose & Scope

KISS-Ops is the **computation vocabulary** of the suite: it defines the set of ops a
kernel may compute, and — for each op — pins exactly what it *means* down to the bit
and the IEEE-754 edge case. It answers "which op is this, and what does it compute?"
so that two independently-written implementations agree not merely that a node is
named `gelu`, but on which of the two GELU formulas it denotes, whether its `relu`
scrubs NaN, and whether its `max` propagates or suppresses NaN.

KISS-Ops owns four things:

1. **The op set** — every op, spelled as an exact token, with its op-family tag and
   its primitive-floor membership flag.
2. **Pinned per-op numeric semantics** — for each op: NaN propagation, signed-zero
   behavior, IEEE versus NaN-propagating min/max, wrapping versus saturating integer
   behavior, raw-bit versus arithmetic moves, rounding mode, shift semantics, out-of-
   bounds index policy, combine algebra, the determinism/fidelity class, and the
   declared-ULP status of transcendental atoms.
3. **Reference decompositions** — for every non-primitive op, a decomposition into
   strictly-lower-level ops, forming an acyclic, strictly-decreasing hierarchy that
   terminates at the primitive floor. This is what makes a kernel labelled with a
   high-level op resolvable by a consumer that has never heard of that op.
4. **The mandatory primitive floor** — the minimal op set every conforming consumer
   MUST understand, at which all decomposition chains bottom out, with each floor op
   justified as not further decomposable in-standard.

**KISS-Ops is NOT:** a data vocabulary (the dtype *set membership*, operand
descriptors, `structure_key`, layout tags, and the `target_capability` descriptor are
owned by the sibling data-vocabulary sub-standard; KISS-Ops references those data
terms **by name only** and re-defines none of them — except that the *bit layout* of
each dtype it assigns numeric meaning to is inlined normatively in §6.16 so this
document is self-contained); a kernel implementation or a lowering pipeline (how an op
is spelled in any source or target language is out of scope — KISS-Emit owns
lowering); a source language or grammar (KISS-Consume owns recognition); the contract
document format (KISS-Contract owns the seven-section contract that *carries* an op DAG
in its semantics field); the advertisable-op surface (KISS-Grammar owns the
`OpTag`→op-name mapping); and it does **NOT** claim cross-language numeric bit
identity for any transcendental atom (that overclaim is explicitly disavowed — see
§6.8).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model — a resolvable hierarchy, not a flat primitive soup

A kernel's semantics is a **DAG of ops at mixed abstraction levels**. A fused
GELU-matmul is `matmul` + `gelu`, *not* the thousands of primitive multiply-add /
exp / polynomial ops it would expand to. Every op that is not primitive carries its
own **reference decomposition** into strictly-lower-level ops. A consumer that meets
an op it does not recognize resolves that op through its decomposition, recursively,
until it reaches ops it knows or the primitive floor.

This buys three things a flat always-lowered form cannot:

- **Compactness** — two nodes (`matmul`, `gelu`) instead of a primitive soup.
- **Meaningful matching** — a consumer with a native `gelu` matches the node directly
  instead of *raising* a primitive DAG back into a recognized shape (hard and lossy).
- **Full resolvability** — a consumer lacking the vocabulary still gets a defined,
  reproducible meaning by resolving the decomposition on demand. The fully-lowered
  primitive form is the oracle for verification under the op's determinism class.

The **primitive floor** is the termination guarantee: every chain bottoms out at a
small, stable set of atoms. Two invariants keep resolution finite: the graph is
**acyclic**, and each op is defined only via **strictly-lower-level** ops.

### 2.2 Extend by definition, not by primitive

Adding a **high-level op** is cheap and additive: it is a decomposition over the
existing floor, and no consumer is *required* to learn it (they resolve it). Adding a
**primitive** is a new *axiom* every consumer must eventually implement — a mandatory-
core change. So the rule is: add high-level ops freely; add a primitive only when a
function is genuinely inexpressible over the floor (a true atom — e.g. a transcendental
with no elementary closed form, or a hardware intrinsic). The floor stays small and its
growth is guarded.

### 2.3 The load-bearing edge cases (why the pins matter)

Four families of "obvious" ops split into non-mergeable pairs precisely because their
edge-case behavior differs and real workloads depend on the difference:

- **`max_prop` / `min_prop`** (NaN-*propagating*, matching `torch.maximum` /
  `torch.minimum`) versus **`fmax_ieee` / `fmin_ieee`** (IEEE-754 `maxNum` /
  `minNum`, NaN-*suppressing*, returning the non-NaN operand). Four distinct ops.
- **`relu`** is `x<0 ? 0 : x` — NaN-*propagating* and `-0.0`-preserving (matching
  `torch.relu`). It is **not** `max(x,0)`, which would scrub NaN and normalize `-0.0`
  to `+0.0`.
- **`rem_floor`** (floored remainder, sign of the *divisor*, `torch.remainder`) versus
  **`rem_trunc`** (truncated remainder / `fmod`, sign of the *dividend*, `torch.fmod`).
- **`gelu`** (exact erf-based, matching `nn.GELU()` default) versus **`gelu_tanh`**
  (the tanh approximation, differing by up to ~1e-4).

`select` is **raw-bit**: `cond != 0 ? a : b` moves the chosen operand's bits with no
arithmetic, so signed zero and NaN payloads (quiet *and* signaling) survive. `-0.0`
tests false; any NaN tests true. It is never rewritten to or from a mask-multiply,
which would perturb exactly these values.

### 2.4 Transcendentals are declared atoms, not mandated polynomials

`exp`, `log`, `sin`, `cos`, `sqrt`, `erf`, `atan`, `lgamma`, and the binary-math atoms
`atan2`, `copysign`, `nextafter` are **atoms**: each is implemented per-target to a
**declared ULP** bound, itself bounded above by a normative per-atom ULP ceiling
pinned in §6.8. KISS-Ops does **not** mandate a reference polynomial and does **not**
claim cross-language bit identity for them — `sin` under one target's math library
differs in the last bit from another's. Mandating a polynomial would over-specify while
still failing to deliver cross-language identity, so the semantics is "the named
function to within a declared ULP no looser than the §6.8 ceiling," and the determinism
class is ULP/tolerance, not exact-byte.

### 2.5 A worked resolution — `gelu`

A consumer receives a kernel whose semantics DAG contains a node `gelu` it does not
implement natively. It queries the decomposition:

```
gelu(x)  =  mul(mul(const(0.5), x), add(const(1), erf(div(x, const(sqrt2)))))
```

Every op on the right — `mul`, `add`, `erf`, `div`, and the `const` leaves — is either
a primitive-floor atom (`mul`, `add`, `div`, `erf`) or a scalar-source leaf (`const`).
Resolution terminates in one step: `gelu` is level 1 over a floor of level 0. A
consumer that *does* implement `gelu` natively matches the node directly and never
expands it. Contrast `gelu_tanh`, which resolves through `tanh` (itself level-lifted
over `exp`), a strictly deeper but still finite and acyclic chain.

### 2.6 A worked resolution — `softmax`

```
m   = reduce(max, x)              # row max (NaN-propagating monoid, keepdim)
e   = exp(sub(x, m))             # shifted exp
s   = reduce(sum, e)             # shifted-exp sum (keepdim)
out = div(e, s)                  # normalize
```

`softmax` is a `normalization`-family op whose decomposition uses `reduce` (a
structural access atom), `exp` (a transcendental atom), and `sub` / `div` (arithmetic
atoms). Output shape equals input shape; the two `reduce` results are keepdim views
(reduced axis retained as extent 1, stride 0, §6.11-0008) so they broadcast back over
the row. Because the decomposition depends on a floating-point `sum` reduction, its
determinism class is order-invariant/nondeterministic (§6.0-0004/-0005). `log_softmax`
decomposes to `sub(x, logsumexp(x))`, and `logsumexp` is itself a `reduction`-family op
with its own numerically-stable decomposition — again finite and acyclic.

### 2.7 Vocabulary catalog (readable — informative)

The complete op set, by family, with primitive-floor membership. This table is a
readable rendering of the normative registry in §6.1 and the semantics tables in §6.4–
§6.13; where this catalog and §6 differ, §6 governs.

**Primitive-floor ops (the mandatory core):**

| Op | Family | Meaning (informative) |
|---|---|---|
| `add` | arithmetic | `a+b`; IEEE-754 float / wrapping int |
| `sub` | arithmetic | `a-b`; IEEE-754 float / wrapping int |
| `mul` | arithmetic | `a*b`; IEEE-754 float / wrapping int |
| `div` | arithmetic | `a/b`; IEEE-754 **float only** |
| `neg` | arithmetic | `-x`; flips sign bit, `-(-0.0)=+0.0`, NaN propagates |
| `abs` | arithmetic | `|x|` by clearing the sign bit (raw-bit) |
| `select` | select | `cond!=0 ? a : b`; raw-bit move, order `(cond,a,b)` |
| `cmp_eq` | comparison | `a==b ? 1:0`; any-NaN → 0 |
| `cmp_ne` | comparison | `a!=b ? 1:0`; any-NaN → 1 (isnan via `cmp_ne(x,x)`) |
| `cmp_lt` | comparison | `a<b ? 1:0`; false on NaN |
| `cmp_le` | comparison | `a<=b ? 1:0`; false on NaN; `-0.0<=+0.0` true |
| `cmp_gt` | comparison | `a>b ? 1:0`; false on NaN |
| `cmp_ge` | comparison | `a>=b ? 1:0`; false on NaN |
| `floor` | rounding | round toward −∞ |
| `ceil` | rounding | round toward +∞ |
| `trunc` | rounding | round toward zero |
| `round_even` | rounding | round to nearest, ties to even |
| `exp` | transcendental | `e^x` (declared-ULP atom) |
| `log` | transcendental | `ln x` (declared-ULP atom) |
| `sin` | transcendental | `sin x` (declared-ULP atom) |
| `cos` | transcendental | `cos x` (declared-ULP atom) |
| `sqrt` | transcendental | √x (IEEE correctly-rounded where guaranteed, else declared-ULP) |
| `erf` | transcendental | Gauss error function (special-function atom) |
| `atan` | transcendental | arctangent (declared-ULP atom) |
| `lgamma` | transcendental | `ln|Γ(x)|` (special-function atom) |
| `atan2` | binary_math | four-quadrant arctangent, IEEE ±0 quadrants |
| `copysign` | binary_math | magnitude of `a`, sign bit of `b` (raw-bit) |
| `nextafter` | binary_math | next representable after `a` toward `b`, dtype's own lattice |
| `bit_and` | bitwise | `a & b` (integer only) |
| `bit_or` | bitwise | `a | b` (integer only) |
| `bit_xor` | bitwise | `a ^ b` (integer only) |
| `bit_not` | bitwise | `~x` (integer only) |
| `shl` | bitwise | `a << b` (integer only) |
| `shr` | bitwise | `a >> b`; arithmetic on signed, logical on unsigned |
| `popcount` | bitwise | set-bit count (integer only, atom) |
| `clz` | bitwise | count leading zeros (integer only, atom) |
| `ctz` | bitwise | count trailing zeros (integer only, atom) |
| `element_map` | access-primitive | per-element scalar body, broadcast reads via stride-0 |
| `reduce` | access-primitive | associative-monoid fold over an axis set (keepdim result) |
| `prefix_scan` | access-primitive | inclusive/exclusive running monoid fold, length-preserving |
| `gather` | access-primitive | data-dependent indexed read; OOB `{skip,clamp,zero-fill}` |
| `scatter` | access-primitive | data-dependent indexed write; combine `{assign,atomic-add,atomic-max,atomic-min}` |
| `sort_network` | access-primitive | stable per-row permutation; exposes permuted values and original-index vector |

**Non-primitive ops (each resolves to the floor via §6.13):**

| Op | Family | Op | Family |
|---|---|---|---|
| `sqr` | arithmetic | `pow` | binary_math |
| `recip` | arithmetic | `hypot` | binary_math |
| `rsqrt` | transcendental | `rem_floor` | binary_math |
| `frac` | rounding | `rem_trunc` | binary_math |
| `sign` | arithmetic | `ldexp` | binary_math |
| `step` | activation | `logical_and` | logical |
| `max_prop` | minmax | `logical_or` | logical |
| `min_prop` | minmax | `logical_not` | logical |
| `fmax_ieee` | minmax | `reduce_mean` | reduction |
| `fmin_ieee` | minmax | `reduce_norm2` | reduction |
| `exp2` | transcendental | `reduce_var` | reduction |
| `expm1` | transcendental | `reduce_std` | reduction |
| `log2` | transcendental | `logsumexp` | reduction |
| `log10` | transcendental | `argmax` | reduction |
| `log1p` | transcendental | `any` | reduction |
| `tan` | transcendental | `all` | reduction |
| `tanh` | transcendental | `matmul` | contraction |
| `sinh` | transcendental | `softmax` | normalization |
| `cosh` | transcendental | `log_softmax` | normalization |
| `asinh` | transcendental | `rms_norm` | normalization |
| `acosh` | transcendental | `layer_norm` | normalization |
| `atanh` | transcendental | `cumsum` | scan |
| `asin` | transcendental | `cumprod` | scan |
| `acos` | transcendental | `cummax` | scan |
| `cbrt` | transcendental | `avg_pool` | window |
| `erfc` | transcendental | `max_pool` | window |
| `sigmoid` | activation | `index_select` | gather_scatter |
| `relu` | activation | `embedding` | gather_scatter |
| `silu` | activation | `scatter_add` | gather_scatter |
| `softplus` | activation | `im2col` | shape |
| `mish` | activation | | |
| `gelu` | activation | | |
| `gelu_tanh` | activation | | |

### 2.8 Terms are joined, not restated

KISS-Ops references the **dtype** tokens (`f16 bf16 f32 f32s f64 s8 u8 i32 i64 u32
bool e4m3 e5m2 s4 u4 b1 c32 c64`), the **operand descriptor** field names (`rank`,
`extents`, `strides`, `dtype`, `alignment`, `layout_tag`, `op_family_tag`, `quant`,
`symbolic_extent`), `structure_key`, the
`target_capability` descriptor, and the pinned constants `MAX_RANK` / `MAX_OPERANDS`
by name only — spelled identically to the shared anchor. These are the shared data-
vocabulary nouns; KISS-Ops does not re-define the descriptor/`structure_key` machinery
and does not depend on the data-vocabulary sub-standard. (The data vocabulary's
`op_family_tag` operand-descriptor field — the cell-level op-category component of
`structure_key` — is the shared noun listed above; the per-op **op-family tag**
taxonomy of §2.7/§3 is a **separate, KISS-Ops-owned** concept, a different closed set,
and is *not* a shared data-vocabulary noun.) It does, however, inline the
**bit layout** of every dtype to which it assigns numeric meaning (§6.16), because the
per-op float semantics are layout-dependent (a `const` bit pattern, the `f16`/`bf16`
lattice, and IEEE encodings cannot be computed from a name alone); inlining the layouts
keeps the "no upstream edge" position true while making this document resolvable by a
reader holding only KISS-Ops plus the umbrella.

---

## 3. Terms & Definitions

- **Op** — a named unit of computation in the KISS-Ops vocabulary, spelled as an exact
  token (e.g. `add`, `gelu`, `reduce`).
- **Op-family tag** — the coarse per-op category an op belongs to (e.g. `arithmetic`,
  `comparison`, `reduction`, `normalization`, `access-primitive`), spanning the per-op
  families listed in §2.7. This per-op family taxonomy is **owned by KISS-Ops** and is a
  **distinct** closed set from the data vocabulary's cell-level `op_family_tag` /
  op-category enum (a separate, coarser closed set that is a component of `structure_key`);
  the two are different closed sets, not the same tag. An op's specialization cell MAY
  participate in a `structure_key` op-category, but that op-category is a separate,
  data-vocabulary-owned classification and is not the KISS-Ops op-family named by this
  term. Distinct from the op name.
- **Primitive op / primitive floor** — an op that has no in-standard decomposition; the
  primitive floor is the mandatory-core set of such ops at which every decomposition
  chain terminates.
- **Non-primitive op** — an op that carries a reference decomposition into strictly-
  lower-level ops.
- **Reference decomposition** — the canonical expression of a non-primitive op in terms
  of strictly-lower-level ops; it defines the op's meaning and is the oracle for
  verification. A conforming kernel MAY be more accurate than the reference for an op
  the §6.13 table marks **refinement-permitted**, but MUST agree on the semantics the
  reference pins.
- **Level** — an integer assigned to each op: `level(primitive) = 0`; for a non-
  primitive op, `level = 1 + max(level of every op referenced in its decomposition,
  including ops embedded in a structural atom's scalar body)`. Levels are strictly
  decreasing along a decomposition edge.
- **Scalar-source leaf** — a non-op source of a scalar value inside an op body:
  `input(i)`, `const(bits)`, `param(i)`, `coord(axis)`, `reduced(stage)`, and
  `extent(axis)`.
- **Compute dtype** — the dtype in which an op's arithmetic is performed, one of the
  pinned dtype set excluding the index/address-only `u32`.
- **Raw-bit move** — a value transfer that copies bits without any arithmetic, so
  signed zero and NaN payloads (quiet and signaling) are preserved.
- **NaN-propagating** — an op that returns a NaN when any operand contributing to the
  result is NaN.
- **NaN-suppressing** — an op (IEEE `maxNum`/`minNum` family) that returns the non-NaN
  operand when exactly one operand is NaN.
- **Declared ULP** — a per-target accuracy bound (in units in the last place) declared
  by a kernel's contract for a transcendental atom, no looser than the §6.8 ceiling;
  the semantics is "the named function to within the declared ULP," not bit identity.
- **Monoid** — an associative binary operation with an identity element; the `reduce` /
  `prefix_scan` fold operator, one of `{sum, prod, max, min}` (identities in §6.11-0002).
- **OOB (out-of-bounds) policy** — the behavior of a data-dependent index that falls
  outside the addressed extent: `skip`, `clamp`, or `zero-fill` for reads; `skip` for
  writes.
- **Combine algebra** — the write-combining operator of `scatter`: `assign`,
  `atomic-add`, `atomic-max`, or `atomic-min`.
- **Determinism class** — the comparator class a numeric result is checked under:
  **exact-byte**, **ULP/tolerance**, or **order-invariant/nondeterministic**. The single
  canonical enum is **owned by KISS-Ops** (the computation-vocabulary root) and imported
  downstream by KISS-Conform and KISS-Synth; it is defined once here (§6.0) and never
  re-forked.
- **dtype**, **operand descriptor**, **structure_key**, **target_capability**,
  **MAX_RANK**, **MAX_OPERANDS** — data-vocabulary terms owned by the sibling data-
  vocabulary sub-standard; used here by name only (§2.8), never re-defined. The bit
  *layout* of each dtype KISS-Ops assigns numeric meaning to is inlined normatively in
  §6.16.
- **Typed decline** — a structured refusal returned in lieu of a result; never a panic,
  abort, crash, hang, or out-of-bounds read.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point arithmetic, comparison predicates, `maxNum` /
  `minNum`, rounding-direction attributes, signed zero, quiet/signaling NaN, and
  subnormals. The per-op float semantics of §6 are pinned against this reference for the
  IEEE-754 dtypes (`f16`, `f32`, `f32s`, `f64`) only; `bf16` and the FP8 formats
  `e4m3` / `e5m2` are **not** IEEE-754 formats and are pinned explicitly in §6.16.
- **Open Compute Project (OCP) 8-bit Floating Point Specification (OFP8), FP8 formats
  E4M3 and E5M2** — the normative reference for the `e4m3` and `e5m2` encodings,
  saturation, and NaN/infinity conventions restated in §6.16. `bf16` (bfloat16) is
  pinned directly in §6.16 as a truncated binary32 layout with round-to-nearest-even.
- **Two's-complement integer representation** — the integer model for wrapping
  arithmetic, arithmetic/logical shift, and the bitwise atoms (§6.4, §6.10).
- **KISS Umbrella Specification** — the suite conventions: the RFC-2119 keyword
  convention, the normative/informative split, the clause-ID scheme and 1:1 test
  mapping, value pinning as bits/IEEE-754 in wire order, the ban on unquantified
  adjectives, the two version axes, the ≥2-dissimilar-implementations-plus-foreign-
  reader freeze gate, the capability/profile/extension model, governance, licensing,
  and patent posture. **Stated once in the umbrella; referenced here; never restated.**
  This sub-standard's §5 points at umbrella §3 for conventions.
- **The data-vocabulary sub-standard** (by version) — **NOT a dependency edge.** KISS-
  Ops has no upstream edge and does not import the data vocabulary. The dtype tokens,
  operand-descriptor field names, `structure_key`, `target_capability`, and the pinned
  constants `MAX_RANK` / `MAX_OPERANDS` are a **shared naming convention** spelled
  identically in both foundational vocabularies (the shared anchor); each is used here
  by name only and neither foundational vocabulary depends on the other. The dtype bit
  layouts KISS-Ops needs for its numeric clauses are **inlined** in §6.16, not imported,
  so no dependency edge is created by the layout-dependence of the float semantics.
- **KISS-Grammar** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: re-bases each advertisable `OpTag` onto a KISS-Ops op name.
- **KISS-Contract** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: the contract semantics field carries the KISS-Ops op DAG, and the per-target
  declared ULP of each transcendental atom (bounded by the §6.8 ceiling).
- **KISS-Synth/Provision** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: the fully-resolved decomposition is the oracle for verification under the
  op's determinism class. The single canonical determinism/fidelity enum is **owned by
  KISS-Ops** (§6.0) and imported by KISS-Synth downstream; no edge points from KISS-Ops
  up to KISS-Synth.
- **KISS-Consume** (by version) — DAG edge labeled **STRUCTURAL**, **downstream**
  consumer: lift targets are KISS-Ops op names.
- **KISS-Emit** (by version) — DAG edge labeled **STRUCTURAL**, **downstream** consumer:
  the normative lowering input is a KISS-Ops op definition plus a specialization-cell
  identity.
- **KISS-Conform** (by version) — depends on and tests KISS-Ops; owns the oracle-
  differential harness that resolves an op's decomposition to the floor and compares
  under the op's declared determinism class, importing the §6.0 enum downstream.

---

## 5. Conventions

This sub-standard adopts the KISS umbrella's conventions (umbrella §3) verbatim and
restates none of them. Per the umbrella: normative §6+ uses **only** the uppercase
keywords `MUST` / `MUST NOT` / `SHALL`; `SHOULD` / `MAY` are reserved for governance
and consumer-behavior guidance and never state a numeric or semantic requirement.
Every atomic requirement carries a stable, append-only ID `KISS-OPS-<section>-<nnnn>`,
allocated by the editor of record, never reused after retirement, and mapped 1:1 to ≥1
named KISS-Conform test. Values are pinned as bits and IEEE-754 semantics, never as one
source language's surface spelling; constant and non-finite values round-trip exactly
per dtype. Unquantified adjectives ("well-formed", "reasonable", "efficient") are
banned from normative text. Every numeric clause declares its determinism/fidelity
class so KISS-Conform selects the correct comparator. See umbrella §3 for the full
statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

The single canonical determinism/fidelity enum is **owned by KISS-Ops** (the
computation-vocabulary root) and imported downstream by KISS-Conform and KISS-Synth;
it is defined here and never re-forked. Its three members are **exact-byte**,
**ULP/tolerance**, and **order-invariant/nondeterministic** (that literal spelling,
verbatim, everywhere).

- **KISS-OPS-6.0-0001** — Every op MUST declare exactly one determinism/fidelity class
  drawn from the single canonical enum `{exact-byte, ULP/tolerance,
  order-invariant/nondeterministic}` owned by KISS-Ops and imported by KISS-Conform and
  KISS-Synth; KISS-Ops MUST NOT define a parallel or forked determinism vocabulary, and
  no downstream sub-standard's copy of the enum overrides this definition. *Test:*
  `test_ops_determinism_class_enum`.
- **KISS-OPS-6.0-0002** — An op MUST be class **exact-byte** if and only if it (a)
  contains no transcendental atom (§6.8), (b) does not depend on a floating-point `sum`
  or `prod` `reduce` or `prefix_scan`, or a `matmul`/contraction, over a float dtype,
  and (c) does not depend on a floating-point atomic combine; the exact-byte class
  therefore covers the arithmetic atoms `add`/`sub`/`mul`/`div`/`neg`/`abs`, the
  comparisons, the rounding atoms, the bitwise atoms, `select`, `copysign`,
  `nextafter`, `element_map`, `gather`, `scatter` with a deterministic combine, and the
  `max` / `min` `reduce` and `prefix_scan` monoids (which are order-invariant in value).
  KISS-Conform MUST evaluate an exact-byte op with a byte-exact comparator. *Test:*
  `test_ops_exact_byte_ops`.
- **KISS-OPS-6.0-0003** — Every transcendental atom (§6.8) and every op whose
  decomposition transitively contains a transcendental atom, and which does not also
  fall under §6.0-0004, MUST be class **ULP/tolerance**, and KISS-Conform MUST evaluate
  it with the op's declared-ULP comparator, never a byte-exact comparator across
  languages. *Test:* `test_ops_ulp_class_ops`.
- **KISS-OPS-6.0-0004** — An op whose result depends on a floating-point `sum` or
  `prod` reduction or scan, a `matmul`/contraction over a float dtype, or a
  floating-point atomic combine (the `scatter` `atomic-add` combine and `scatter_add`)
  MUST be class **order-invariant/nondeterministic**; KISS-Conform MUST NOT require
  byte-exact reproduction of its floating-point result across implementations or runs,
  and MUST compare it under a tolerance rather than a byte-exact comparator. This
  removes floating-point `sum`/`prod` reductions and scans (`reduce(sum)`,
  `reduce(prod)`, `prefix_scan(sum)`, `prefix_scan(prod)`, `cumsum`, `cumprod`),
  `matmul` and every op containing a float contraction (`reduce_mean`, `reduce_var`,
  `reduce_std`, `reduce_norm2`, `logsumexp`, `softmax`, `log_softmax`, `rms_norm`,
  `layer_norm`, `avg_pool`) from the exact-byte class, because floating-point summation
  is not associative and neither a canonical reduction order nor a canonical accumulator
  width is pinned by KISS-Ops. *Test:* `test_ops_nondeterministic_class_ops`.
- **KISS-OPS-6.0-0005** — Where an op qualifies for more than one class under §6.0-0002
  through §6.0-0004, it MUST be assigned the **most permissive** class in the order
  `order-invariant/nondeterministic` (most permissive) > `ULP/tolerance` >
  `exact-byte` (least permissive); e.g. `softmax` (contains both a transcendental `exp`
  and a float `sum` reduction) MUST be class order-invariant/nondeterministic. *Test:*
  `test_ops_determinism_class_precedence`.
- **KISS-OPS-6.0-0006** — For every **exact-byte** op over a float dtype, each
  arithmetic atom in the op's reference decomposition MUST round independently per
  IEEE 754-2019; an implementation MUST NOT apply a fused-multiply-add or any other
  multi-atom expression contraction that removes an intermediate rounding, because doing
  so perturbs the byte-exact result. Expression contraction and reassociation are
  permitted only for ops in the order-invariant/nondeterministic class (§6.0-0004),
  whose result is already not required to reproduce byte-for-byte. *Test:*
  `test_ops_no_fma_contraction_exact_byte`.

### 6.1 The op-set registry

- **KISS-OPS-6.1-0001** — The KISS-Ops op set for this version MUST be exactly the set
  enumerated in §2.7 / §6.4–§6.13 (the primitive-floor ops of §6.3 plus the non-
  primitive ops of §6.13); an implementation MUST NOT treat a token outside this set as
  a KISS-Ops op of this version. *Test:* `test_ops_op_set_closed`.
- **KISS-OPS-6.1-0002** — Each op MUST be spelled as the exact token given in this
  document (case-sensitive, underscore-delimited); an implementation MUST NOT accept a
  synonym, alias, or alternative spelling as the same op. *Test:*
  `test_ops_op_token_spelling`.
- **KISS-OPS-6.1-0003** — Each op MUST carry exactly the op-family tag assigned to it in
  §2.7; an implementation MUST NOT re-classify an op into a different family. *Test:*
  `test_ops_op_family_tags`.
- **KISS-OPS-6.1-0004** — Each op MUST carry exactly the primitive-floor membership flag
  assigned to it (primitive = in §6.3; non-primitive = in §6.13); an implementation MUST
  NOT treat a non-primitive op as primitive or a primitive op as non-primitive. *Test:*
  `test_ops_primitive_flags`.

### 6.2 Shared numeric conventions

- **KISS-OPS-6.2-0001** — For every op whose compute dtype is an IEEE-754 float dtype
  (`f16` binary16, `f32`, `f32s` binary32, `f64` binary64), the arithmetic MUST follow
  IEEE 754-2019 for that operation and dtype, except where a specific op clause below
  pins a departure (e.g. the NaN-suppressing min/max family, the declared-ULP
  transcendental atoms). For the non-IEEE-754 float dtypes `bf16`, `e4m3`, and `e5m2`,
  the arithmetic MUST follow the encodings, rounding, saturation, and NaN/infinity
  conventions pinned in §6.16 (these formats are **not** governed by IEEE 754-2019).
  *Test:* `test_ops_float_ieee754`.
- **KISS-OPS-6.2-0002** — For every op whose compute dtype is an integer dtype (`s8`,
  `u8`, `i32`, `i64`, `s4`, `u4`), integer `add`/`sub`/`mul` MUST be **wrapping** two's-
  complement modulo `2^bitwidth`, and MUST NOT be undefined-behavior on overflow and
  MUST NOT saturate. *Test:* `test_ops_int_wrapping`.
- **KISS-OPS-6.2-0003** — For every **primitive-floor** op not explicitly pinned as
  NaN-suppressing or raw-bit, a NaN in any operand that contributes to the scalar result
  MUST propagate a NaN to the result (default NaN propagation for atoms). *Test:*
  `test_ops_default_nan_propagation`.
- **KISS-OPS-6.2-0004** — Every op MUST preserve IEEE signed zero per IEEE 754-2019 for
  its dtype and MUST NOT normalize `-0.0` to `+0.0` except where a clause explicitly
  produces `+0.0` (e.g. `neg` of `-0.0`, `abs` of `-0.0`). *Test:*
  `test_ops_signed_zero_preserved`.
- **KISS-OPS-6.2-0005** — A comparison op (§6.6) MUST produce its result as the value `1`
  (true) or `0` (false) encoded in the op's compute dtype, and MUST NOT produce any other
  value. *Test:* `test_ops_compare_result_zero_one`.
- **KISS-OPS-6.2-0006** — For the `bool` dtype, any op that consumes a `bool` operand MUST
  treat a byte value of `0` as false and any non-zero byte as true, and any op that
  produces a `bool` result MUST normalize it to strictly `0` or `1`. *Test:*
  `test_ops_bool_normalization`.
- **KISS-OPS-6.2-0007** — The `u32` dtype MUST be accepted only in the index-operand role
  of `gather`, `scatter`, `index_select`, `embedding`, and `scatter_add` (per the legal
  index-dtype set of §6.11-0009); every arithmetic, comparison, rounding, bitwise,
  reduction, and scan path MUST reject a `u32` compute operand with a typed decline.
  *Test:* `test_ops_u32_index_only`.
- **KISS-OPS-6.2-0008** — A `const(bits)` leaf and every non-finite value (±∞, quiet NaN,
  signaling NaN, ±0, subnormal) MUST be pinned by its bit pattern in the operand's dtype
  and MUST round-trip exactly; a clause MUST NOT pin a constant by a source language's
  decimal spelling. A symbolic constant named in the §6.13 decomposition table is pinned
  per §6.12-0003. *Test:* `test_ops_const_bits_roundtrip`.
- **KISS-OPS-6.2-0009** — A non-primitive op's NaN, signed-zero, and edge-case behavior
  MUST be exactly the behavior obtained by evaluating its reference decomposition
  (§6.13); the default atom-propagation rule of §6.2-0003 does not independently
  constrain a non-primitive op where its decomposition produces a different result — for
  example `sign(NaN)=0`, `step(NaN)=0`, `logical_not(NaN)=0` (false), and `frac(-0.0)=
  +0.0` are the pinned results of those ops' decompositions and are conforming. *Test:*
  `test_ops_nonprimitive_semantics_from_decomposition`.

### 6.3 The mandatory primitive floor

- **KISS-OPS-6.3-0001** — The primitive floor for this version MUST be exactly the
  following op set: the arithmetic atoms `add`, `sub`, `mul`, `div`, `neg`, `abs`; the
  raw-bit ternary `select`; the ordered comparisons `cmp_eq`, `cmp_ne`, `cmp_lt`,
  `cmp_le`, `cmp_gt`, `cmp_ge`; the rounding atoms `floor`, `ceil`, `trunc`,
  `round_even`; the transcendental atoms `exp`, `log`, `sin`, `cos`, `sqrt`, `erf`,
  `atan`, `lgamma`; the binary-math atoms `atan2`, `copysign`, `nextafter`; the integer
  bitwise atoms `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr`, `popcount`,
  `clz`, `ctz`; and the structural access atoms `element_map`, `reduce`, `prefix_scan`,
  `gather`, `scatter`, `sort_network`. *Test:* `test_ops_primitive_floor_set`.
- **KISS-OPS-6.3-0002** — Every conforming consumer of KISS-Ops MUST understand (be able
  to evaluate the pinned semantics of) every op in the primitive floor; the floor is the
  mandatory core (umbrella §6.1) and a consumer that cannot evaluate a floor op does not
  conform. *Test:* `test_ops_floor_is_mandatory_core`.
- **KISS-OPS-6.3-0003** — A primitive-floor op MUST NOT carry an in-standard reference
  decomposition. *Test:* `test_ops_floor_ops_have_no_decomposition`.
- **KISS-OPS-6.3-0004** — Every decomposition chain of every non-primitive op (§6.13)
  MUST terminate at the primitive floor; a non-primitive op MUST NOT decompose to a token
  outside the union of the primitive floor, the non-primitive op set, and the scalar-
  source leaves. *Test:* `test_ops_decomposition_terminates_at_floor`.
- **KISS-OPS-6.3-0005** — Each primitive-floor op MUST be justified as a true atom —
  either an IEEE-754 / two's-complement scalar operation, a raw-bit move, a directed-
  rounding operation, a special function with no elementary closed form, an integer bit
  intrinsic, or a structural access pattern — with no expression over strictly-lower-
  level KISS-Ops ops. *Test:* `test_ops_floor_ops_justified_atoms`.

### 6.4 Arithmetic atoms

The arithmetic atoms compute as follows (float dtypes per §6.2-0001, integer dtypes
wrapping two's-complement per §6.2-0002):

| Op | Result | Float NaN/zero pin | Integer pin |
|---|---|---|---|
| `add` | `a + b` | IEEE-754 | wrapping |
| `sub` | `a - b` | IEEE-754 | wrapping |
| `mul` | `a * b` | IEEE-754 | wrapping |
| `div` | `a / b` | IEEE-754, **float only** | excluded |
| `neg` | `-x` | flip sign bit; `-(-0.0)=+0.0`; NaN propagates | two's-complement negate (wrapping) |
| `abs` | `|x|` | clear sign bit (raw-bit); `|-0.0|=+0.0`; NaN payload preserved | two's-complement absolute (wrapping) |

- **KISS-OPS-6.4-0001** — `add`, `sub`, and `mul` MUST compute `a+b`, `a-b`, `a*b`
  respectively, using IEEE-754 for float dtypes and wrapping two's-complement for integer
  dtypes. *Test:* `test_ops_add_sub_mul`.
- **KISS-OPS-6.4-0002** — `div` MUST compute `a/b` under IEEE-754 for float dtypes only;
  integer division and integer remainder are excluded from the op set, and division by
  zero MUST be treated as target-defined (target-UB), not pinned by KISS-Ops. *Test:*
  `test_ops_div_float_only`.
- **KISS-OPS-6.4-0003** — `neg` MUST compute `-x` by flipping the sign bit for float
  dtypes such that `neg(-0.0)` yields `+0.0` and a NaN operand propagates a NaN, and by
  two's-complement negation for integer dtypes. *Test:* `test_ops_neg`.
- **KISS-OPS-6.4-0004** — `abs` MUST compute `|x|` by clearing the sign bit as a raw-bit
  operation for float dtypes such that `abs(-0.0)` yields `+0.0` and a NaN operand's
  payload is preserved (the result is a NaN with the same payload, sign cleared), and by
  two's-complement absolute value for integer dtypes. *Test:* `test_ops_abs_raw_bit`.
- **KISS-OPS-6.4-0005** — Integer `neg` and `abs` MUST be **wrapping** two's-complement
  at the representable boundary: `neg(INT_MIN)` MUST yield `INT_MIN` and `abs(INT_MIN)`
  MUST yield `INT_MIN` for each signed integer dtype, and neither MUST saturate, trap, or
  invoke undefined behavior. *Test:* `test_ops_int_neg_abs_wrap`.

### 6.5 Raw-bit select

- **KISS-OPS-6.5-0001** — `select` MUST take operands in the order `(cond, a, b)` and MUST
  yield `a` when `cond != 0` and `b` when `cond == 0`. *Test:* `test_ops_select_order`.
- **KISS-OPS-6.5-0002** — `select` MUST move the chosen operand as a **raw-bit** copy with
  no arithmetic applied to either arm, so that the chosen operand's signed zero and NaN
  payload (quiet and signaling) are preserved bit-for-bit. *Test:*
  `test_ops_select_raw_bit`.
- **KISS-OPS-6.5-0003** — In evaluating the `cond` operand, `select` MUST treat `-0.0` as
  false (it compares equal to zero) and MUST treat any NaN as true (it is non-zero).
  *Test:* `test_ops_select_cond_zero_nan`.
- **KISS-OPS-6.5-0004** — `select` MUST NOT be rewritten to or from a mask-multiply form
  (e.g. `cond*a + (1-cond)*b`), because a mask-multiply perturbs signed zero and NaN
  payloads that the raw-bit move preserves. *Test:* `test_ops_select_no_mask_multiply`.

### 6.6 Comparison atoms

Each comparison yields `1` or `0` in the compute dtype (§6.2-0005):

| Op | True when | NaN operand |
|---|---|---|
| `cmp_eq` | `a == b` | → 0 (false) |
| `cmp_ne` | `a != b` | → 1 (true) |
| `cmp_lt` | `a < b` | → 0 (false) |
| `cmp_le` | `a <= b` | → 0 (false) |
| `cmp_gt` | `a > b` | → 0 (false) |
| `cmp_ge` | `a >= b` | → 0 (false) |

- **KISS-OPS-6.6-0001** — Each comparison op MUST compute the predicate in its row above
  and yield `1` (true) or `0` (false) in the compute dtype. *Test:*
  `test_ops_compare_predicates`.
- **KISS-OPS-6.6-0002** — `cmp_eq`, `cmp_lt`, `cmp_le`, `cmp_gt`, and `cmp_ge` MUST each
  yield `0` (false) whenever either operand is NaN. *Test:* `test_ops_compare_nan_false`.
- **KISS-OPS-6.6-0003** — `cmp_ne` MUST yield `1` (true) whenever either operand is NaN,
  and consequently `cmp_ne(x, x)` MUST serve as the `isnan` predicate for `x`. *Test:*
  `test_ops_cmp_ne_nan_true`.
- **KISS-OPS-6.6-0004** — Comparisons MUST honor IEEE signed-zero equality: `cmp_le(-0.0,
  +0.0)` and `cmp_ge(-0.0, +0.0)` MUST each yield `1` (true), and `cmp_eq(-0.0, +0.0)`
  MUST yield `1` (true). *Test:* `test_ops_compare_signed_zero`.

### 6.7 Rounding atoms

| Op | Rounding direction |
|---|---|
| `floor` | toward −∞ |
| `ceil` | toward +∞ |
| `trunc` | toward zero (exact on finite) |
| `round_even` | to nearest, ties to even |

- **KISS-OPS-6.7-0001** — `floor` MUST round toward −∞, `ceil` MUST round toward +∞,
  `trunc` MUST round toward zero, and `round_even` MUST round to nearest with ties to
  even, each per the corresponding IEEE 754-2019 rounding-direction attribute. *Test:*
  `test_ops_rounding_directions`.
- **KISS-OPS-6.7-0002** — Each rounding atom MUST propagate a NaN operand to a NaN result
  and MUST preserve the sign of a zero operand (`trunc(-0.0)=-0.0`). *Test:*
  `test_ops_rounding_nan_signed_zero`.

### 6.8 Transcendental atoms (declared-ULP)

The transcendental atoms are `exp` (`e^x`), `log` (`ln x`), `sin`, `cos`, `sqrt` (√x),
`erf` (Gauss error function), `atan` (arctangent), and `lgamma` (`ln|Γ(x)|`). Each
carries a normative **maximum ULP ceiling** below; a kernel's contract declares a
per-target ULP that MUST NOT exceed the ceiling, and KISS-Conform evaluates the atom
under that declared ULP:

| Atom | Maximum ULP ceiling (compute dtype ≥ 16-bit float) |
|---|---|
| `sqrt` | 0.5 ULP (correctly rounded) where the target guarantees it, else 2 ULP |
| `exp`, `log`, `sin`, `cos`, `atan`, `atan2` | 4 ULP |
| `erf` | 4 ULP |
| `lgamma` | 8 ULP |

- **KISS-OPS-6.8-0001** — Each transcendental atom MUST compute its named mathematical
  function to within its **maximum ULP ceiling** in the table above; a kernel's contract
  MAY declare a tighter per-target ULP but MUST NOT declare one looser than the ceiling,
  and KISS-Conform MUST reject a declared ULP exceeding the ceiling. KISS-Ops MUST NOT
  mandate a specific reference polynomial or table for a transcendental atom. *Test:*
  `test_ops_transcendental_declared_ulp`.
- **KISS-OPS-6.8-0002** — KISS-Ops MUST NOT claim cross-language or cross-target bit
  identity for any transcendental atom; conformance for these atoms MUST be evaluated
  under the ULP/tolerance determinism class (§6.0-0003), and byte-exact identity MUST be
  claimed only same-language on-device (deferred to KISS-Emit / KISS-Conform). *Test:*
  `test_ops_transcendental_no_cross_lang_identity`.
- **KISS-OPS-6.8-0003** — `sqrt` MUST be correctly rounded per IEEE 754-2019 on any target
  that guarantees correctly-rounded square root, and MUST otherwise meet its declared ULP
  bound (≤ the 2 ULP ceiling). *Test:* `test_ops_sqrt_correctly_rounded_or_ulp`.
- **KISS-OPS-6.8-0004** — `erf` and `lgamma` MUST be treated as special-function atoms
  with no elementary decomposition over other KISS-Ops ops; an implementation MUST NOT
  require them to be expressed via `exp`/`log`/etc. *Test:*
  `test_ops_special_function_atoms`.

### 6.9 Binary-math atoms

- **KISS-OPS-6.9-0001** — `atan2` MUST compute the four-quadrant arctangent of `(y=a,
  x=b)` with the IEEE 754-2019 ±0 quadrant conventions (the sign of a zero operand selects
  the quadrant). *Test:* `test_ops_atan2_quadrants`.
- **KISS-OPS-6.9-0002** — `copysign` MUST produce a value with the magnitude of `a` and
  the sign bit of `b` as a raw-bit operation, so that the signed zero of `b` and the sign
  of a NaN `b` are carried into the result. *Test:* `test_ops_copysign_raw_bit`.
- **KISS-OPS-6.9-0003** — `nextafter` MUST produce the next representable value after `a`
  toward `b` in the **dtype's own** representation lattice; `nextafter` MUST reject `f16`
  and `bf16` operands (because stepping in promoted `f32` yields the wrong neighbor) with
  a typed decline. *Test:* `test_ops_nextafter_own_lattice`.

### 6.10 Bitwise atoms

- **KISS-OPS-6.10-0001** — Every bitwise atom (`bit_and`, `bit_or`, `bit_xor`, `bit_not`,
  `shl`, `shr`, `popcount`, `clz`, `ctz`) MUST accept only integer dtypes and MUST reject
  a float, complex, or `bool`-arithmetic operand with a typed decline. *Test:*
  `test_ops_bitwise_integer_only`.
- **KISS-OPS-6.10-0002** — `bit_and`, `bit_or`, `bit_xor`, and `bit_not` MUST compute
  `a&b`, `a|b`, `a^b`, and `~x` respectively over the integer bit pattern. *Test:*
  `test_ops_bitwise_logic`.
- **KISS-OPS-6.10-0003** — `shl` MUST compute a logical left shift `a<<b`; `shr` MUST
  compute an **arithmetic** right shift (sign bit replicated) on signed integer dtypes and
  a **logical** right shift (zero-filled) on unsigned integer dtypes. *Test:*
  `test_ops_shift_arithmetic_vs_logical`.
- **KISS-OPS-6.10-0004** — For `shl` and `shr`, a shift amount outside `[0, bitwidth)`
  MUST inherit the target's behavior (KISS-Ops does not pin it); the caller is responsible
  for clamping the shift amount if a defined result is required. *Test:*
  `test_ops_shift_out_of_range_target_defined`.
- **KISS-OPS-6.10-0005** — `popcount`, `clz`, and `ctz` MUST compute the set-bit count,
  the leading-zero count, and the trailing-zero count respectively over the integer bit
  pattern, and MUST be treated as atoms with no decomposition. *Test:*
  `test_ops_popcount_clz_ctz`.

### 6.11 Structural access atoms

- **KISS-OPS-6.11-0001** — `element_map` MUST define the base access in which the output
  coordinate equals the input coordinate, with broadcast reads expressed by a stride of
  `0` along an axis, applying a per-element scalar-expression body; it MUST be treated as
  a structural atom. *Test:* `test_ops_element_map_base_access`.
- **KISS-OPS-6.11-0002** — `reduce` MUST fold over an axis set using an associative monoid
  drawn from `{sum (identity +0.0 / integer 0), prod (identity 1), max (identity the
  dtype minimum: −∞ for float, INT_MIN for signed int, 0 for unsigned int), min (identity
  the dtype maximum: +∞ for float, INT_MAX for signed int, the unsigned maximum for
  unsigned int)}`; the `max` and `min` monoids MUST be **NaN-propagating**, and a
  reduction over an **empty** axis MUST yield the monoid identity. *Test:*
  `test_ops_reduce_monoids`.
- **KISS-OPS-6.11-0003** — `prefix_scan` MUST compute an inclusive or exclusive running
  monoid fold along exactly one axis and MUST be length-preserving (one output element per
  input position), distinct from `reduce`. *Test:* `test_ops_prefix_scan_length_preserving`.
- **KISS-OPS-6.11-0004** — `gather` MUST substitute a runtime integer index value for one
  iteration-axis coordinate (a data-dependent read) with an out-of-bounds policy in
  `{skip, clamp, zero-fill}`; a negative index (representable only in a signed index
  dtype, §6.11-0009) MUST always be treated as out-of-bounds (no from-end wrap). *Test:*
  `test_ops_gather_oob_policy`.
- **KISS-OPS-6.11-0005** — `scatter` MUST substitute a runtime integer index for one
  output-axis coordinate (a data-dependent write) with a combine operator in `{assign,
  atomic-add, atomic-max, atomic-min}`, and MUST skip an out-of-bounds write. *Test:*
  `test_ops_scatter_combine_and_oob`.
- **KISS-OPS-6.11-0006** — The `scatter` `atomic-add` combine on a floating-point dtype
  MUST be declared nondeterministic (class order-invariant/nondeterministic, §6.0-0004).
  The `atomic-max`, `atomic-min`, and integer `atomic-add` combines MUST be
  deterministic. The `assign` combine MUST be deterministic: when two or more source
  elements scatter to the **same** destination index, the write from the **highest
  source (row-major iteration) index** MUST win (a pinned last-writer-in-iteration-order
  tie-break), so `assign` never depends on hardware race order. *Test:*
  `test_ops_scatter_fp_atomic_add_nondeterministic`.
- **KISS-OPS-6.11-0007** — `sort_network` MUST perform a **stable** per-row permutation
  under a total order on `(key, original-index)` pairs in which NaN orders as the greatest
  value (ascending → NaN last, descending → NaN first), and MUST expose **two** outputs:
  (a) the values written back as a raw-bit permutation, and (b) the **original-index
  vector** — for each output rank, the source position it came from. It MUST be treated as
  a structural atom with no monoid. *Test:* `test_ops_sort_network_total_order`.
- **KISS-OPS-6.11-0008** — `reduce` MUST retain each reduced axis as an extent-`1` axis
  with stride `0` (a keepdim result) so the reduced value broadcasts back over the
  original axis, making shifted-shape decompositions such as `sub(x, reduce(max, x))`
  well-formed; `prefix_scan` MUST retain all axes (length-preserving). *Test:*
  `test_ops_reduce_keepdim_broadcast`.
- **KISS-OPS-6.11-0009** — The legal index-operand dtype set for `gather`, `scatter`,
  `index_select`, `embedding`, and `scatter_add` MUST be exactly `{u32, i32, i64}`; an
  implementation MUST reject any other index dtype with a typed decline. The negative-
  index-is-out-of-bounds rule (§6.11-0004) applies to the signed index dtypes `i32` and
  `i64`; `u32` carries no negative value and the rule is vacuous for it. *Test:*
  `test_ops_index_dtype_set`.
- **KISS-OPS-6.11-0010** — The `scatter` floating-point `atomic-max` and `atomic-min`
  combines MUST be **NaN-propagating**, consistent with the `max` / `min` `reduce`
  monoids (§6.11-0002): if a NaN is scattered to a destination, or a destination already
  holds a NaN, the combined result MUST be NaN. *Test:*
  `test_ops_scatter_atomic_minmax_nan`.

### 6.12 Scalar-source leaves

- **KISS-OPS-6.12-0001** — The scalar-source leaves inside an op body MUST be exactly:
  `input(i)` (the `i`-th input operand's element), `const(bits)` (a dtype-typed constant
  pinned by its bit pattern), `param(i)` (the `i`-th scalar parameter), `coord(axis)` (the
  current iteration coordinate along `axis`), `reduced(stage)` (the accumulated result of
  a named reduction/scan stage), and `extent(axis)` (the runtime logical extent — length —
  of iteration `axis`, a non-negative integer-valued scalar source used where a
  decomposition needs a shape-derived quantity such as a mean divisor). These leaves are
  part of the op-semantics currency and are not themselves ops. *Test:*
  `test_ops_scalar_source_leaves`.
- **KISS-OPS-6.12-0002** — A `const(bits)` leaf MUST carry its value as the exact dtype bit
  pattern (round-tripping ±∞, quiet/signaling NaN, ±0, and subnormals per §6.2-0008); the
  spelling of a constant in any source or target language is emitter-supplied and MUST NOT
  be relied upon across the semantics currency. A `const` leaf MUST NOT denote a non-
  literal, shape-dependent, or runtime-derived quantity (such a quantity MUST be sourced
  from `extent(axis)`, `param(i)`, `coord(axis)`, or `reduced(stage)`). *Test:*
  `test_ops_const_leaf_bits`.
- **KISS-OPS-6.12-0003** — A symbolic constant named in the §6.13 reference-decomposition
  table (`0.5`, `1`, `-1`, `2`, `3`, `ln2`, `ln10`, `pi/2`, `sqrt2`, `sqrt(2/pi)`,
  `0.044715`) MUST denote a `const(bits)` leaf whose value is the **correctly-rounded,
  round-to-nearest-ties-to-even image of the exact real number** it names, in the
  operand's compute dtype. Each name's exact real value is fixed by its mathematical
  definition (`ln2 = logₑ2`, `ln10 = logₑ10`, `pi/2 = π/2`, `sqrt2 = √2`,
  `sqrt(2/pi) = √(2/π)`, and the exact decimals `0.5`, `1`, `-1`, `2`, `3`, `0.044715 =
  44715/1000000`); the round-to-nearest-ties-to-even mapping from that exact real into a
  given compute dtype is unique, so the bit pattern is fully determined per dtype and a
  clause MUST NOT rely on a decimal spelling as such. For the `f32` compute dtype the
  bit patterns include (informative examples, all determined by the rule above):
  `0.5 = 0x3F000000`, `1 = 0x3F800000`, `-1 = 0xBF800000`, `2 = 0x40000000`,
  `sqrt2 = 0x3FB504F3`, `ln2 = 0x3F317218`. *Test:* `test_ops_named_constant_bits`.

### 6.13 Reference decompositions of non-primitive ops

Every non-primitive op resolves to the primitive floor via the reference decomposition
in the table below (this resolution is the normative requirement of KISS-OPS-6.13-0001).
Each decomposition uses only ops of strictly lower level (§6.14) and the scalar-source
leaves of §6.12. Symbolic constants (`ln2`, `ln10`, `pi/2`, `sqrt2`, `sqrt(2/pi)`,
`0.044715`, etc.) are `const(bits)` leaves pinned per §6.12-0003. A decomposition body
follows the grammar of §6.13-0006 (a single expression tree, or single-assignment
`name = expr;` let-bindings followed by a final result). The **Refine** column marks the
ops for which §6.13-0003 permits an accuracy-refined implementation.

Operand-ordering conventions for parameterized ops (pinned as attributes per §6.13-0004):
`rms_norm` reads `input(0)=x`, `input(1)=gamma`, `param(0)=eps`; `layer_norm` reads
`input(0)=x`, `input(1)=gamma`, `input(2)=beta`, `param(0)=eps`; `matmul` reads
`input(0)=lhs`, `input(1)=rhs`.

| Op | Family | Refine | Reference decomposition |
|---|---|---|---|
| `sqr` | arithmetic | — | `mul(x, x)` |
| `recip` | arithmetic | — | `div(const(1), x)` |
| `rsqrt` | transcendental | — | `div(const(1), sqrt(x))` |
| `frac` | rounding | — | `sub(x, trunc(x))` |
| `sign` | arithmetic | — | `select(cmp_gt(x,const(0)), const(1), select(cmp_lt(x,const(0)), const(-1), const(0)))` |
| `step` | activation | — | `select(cmp_gt(x, const(0)), const(1), const(0))` |
| `max_prop` | minmax | — | `select(cmp_ne(a,a), a, select(cmp_ne(b,b), b, select(cmp_ge(a,b), a, b)))` |
| `min_prop` | minmax | — | `select(cmp_ne(a,a), a, select(cmp_ne(b,b), b, select(cmp_le(a,b), a, b)))` |
| `fmax_ieee` | minmax | — | `select(cmp_ne(a,a), b, select(cmp_ne(b,b), a, select(cmp_ge(a,b), a, b)))` |
| `fmin_ieee` | minmax | — | `select(cmp_ne(a,a), b, select(cmp_ne(b,b), a, select(cmp_le(a,b), a, b)))` |
| `exp2` | transcendental | — | `exp(mul(x, const(ln2)))` |
| `expm1` | transcendental | ✓ | `sub(exp(x), const(1))` |
| `log2` | transcendental | — | `div(log(x), const(ln2))` |
| `log10` | transcendental | — | `div(log(x), const(ln10))` |
| `log1p` | transcendental | ✓ | `log(add(const(1), x))` |
| `tan` | transcendental | — | `div(sin(x), cos(x))` |
| `tanh` | transcendental | — | `div(sub(exp(x), exp(neg(x))), add(exp(x), exp(neg(x))))` |
| `sinh` | transcendental | — | `div(sub(exp(x), exp(neg(x))), const(2))` |
| `cosh` | transcendental | — | `div(add(exp(x), exp(neg(x))), const(2))` |
| `asinh` | transcendental | — | `log(add(x, sqrt(add(sqr(x), const(1)))))` |
| `acosh` | transcendental | — | `log(add(x, sqrt(sub(sqr(x), const(1)))))` |
| `atanh` | transcendental | — | `mul(const(0.5), log(div(add(const(1),x), sub(const(1),x))))` |
| `asin` | transcendental | — | `atan(div(x, sqrt(sub(const(1), sqr(x)))))` |
| `acos` | transcendental | — | `sub(const(pi/2), asin(x))` |
| `cbrt` | transcendental | — | `mul(sign(x), exp(div(log(abs(x)), const(3))))` |
| `erfc` | transcendental | — | `sub(const(1), erf(x))` |
| `sigmoid` | activation | — | `recip(add(const(1), exp(neg(x))))` |
| `relu` | activation | — | `select(cmp_lt(x, const(0)), const(0), x)` |
| `silu` | activation | — | `mul(x, sigmoid(x))` |
| `softplus` | activation | — | `log(add(const(1), exp(x)))` |
| `mish` | activation | — | `mul(x, tanh(softplus(x)))` |
| `gelu` | activation | — | `mul(mul(const(0.5), x), add(const(1), erf(div(x, const(sqrt2)))))` |
| `gelu_tanh` | activation | — | `mul(mul(const(0.5), x), add(const(1), tanh(mul(const(sqrt(2/pi)), add(x, mul(const(0.044715), mul(x, sqr(x))))))))` |
| `pow` | binary_math | ✓ | `exp(mul(b, log(a)))` for `a>0`; full-domain behavior pinned by §6.13-0005 |
| `hypot` | binary_math | ✓ | `sqrt(add(sqr(a), sqr(b)))` |
| `rem_floor` | binary_math | — | `sub(a, mul(floor(div(a,b)), b))` |
| `rem_trunc` | binary_math | — | `sub(a, mul(trunc(div(a,b)), b))` |
| `ldexp` | binary_math | ✓ | `mul(a, exp2(b))` (reference; exact scaling `a·2^b` for integer `b`, see §6.13-0003) |
| `logical_and` | logical | — | `mul(cmp_ne(a, const(0)), cmp_ne(b, const(0)))` |
| `logical_or` | logical | — | `min_prop(const(1), add(cmp_ne(a,const(0)), cmp_ne(b,const(0))))` |
| `logical_not` | logical | — | `cmp_eq(x, const(0))` |
| `reduce_mean` | reduction | — | `div(reduce(sum, x), extent(reduced_axis))` |
| `reduce_norm2` | reduction | — | `sqrt(reduce(sum, sqr(x)))` |
| `reduce_var` | reduction | — | `sub(reduce_mean(sqr(x)), sqr(reduce_mean(x)))` |
| `reduce_std` | reduction | — | `sqrt(reduce_var(x))` |
| `logsumexp` | reduction | — | `m=reduce(max,x); out=add(m, log(reduce(sum, exp(sub(x,m)))))` |
| `argmax` | reduction | — | `original-index at rank 0 of sort_network(desc, keys=x)` (the §6.11-0007 index-vector output) |
| `any` | reduction | — | `reduce(max, cmp_ne(x, const(0)))` |
| `all` | reduction | — | `reduce(min, cmp_ne(x, const(0)))` |
| `matmul` | contraction | — | `reduce(sum, axis=K) of element_map(mul(input(0), input(1)))`, where over iteration space `(m,n,k)` `input(0)` is read at `[m,k]` broadcast over N (stride 0 on N) and `input(1)` at `[k,n]` broadcast over M (stride 0 on M) per §6.11-0001 |
| `softmax` | normalization | — | `m=reduce(max,x); e=exp(sub(x,m)); s=reduce(sum,e); out=div(e,s)` |
| `log_softmax` | normalization | — | `sub(x, logsumexp(x))` |
| `rms_norm` | normalization | — | `ms=reduce_mean(sqr(input(0))); out=mul(mul(input(0), rsqrt(add(ms, param(0)))), input(1))` |
| `layer_norm` | normalization | — | `mu=reduce_mean(input(0)); v=reduce_var(input(0)); out=add(mul(mul(sub(input(0),mu), rsqrt(add(v,param(0)))), input(1)), input(2))` |
| `cumsum` | scan | — | `prefix_scan(monoid=sum, inclusive)` |
| `cumprod` | scan | — | `prefix_scan(monoid=prod, inclusive)` |
| `cummax` | scan | — | `prefix_scan(monoid=max, inclusive)` |
| `avg_pool` | window | — | `reduce_mean` over the window axis of the pooled view (§6.13-0004 window attributes); divisor per `count_include_pad` |
| `max_pool` | window | — | `reduce(max)` over the window axis of the pooled view (OOB taps skipped; NaN propagates) |
| `index_select` | gather_scatter | — | `gather(oob=skip)` with a 1-D index operand |
| `embedding` | gather_scatter | — | `gather(oob=zero-fill)` with a 1-D index operand |
| `scatter_add` | gather_scatter | — | `scatter(combine=atomic-add, oob=skip)` |
| `im2col` | shape | — | closed-form structured `gather` (source coord from loop coords), expanding access |

- **KISS-OPS-6.13-0001** — Each non-primitive op MUST carry the reference decomposition
  in the table above, and that decomposition MUST define the op's pinned semantics; a
  consumer resolving an unrecognized op MUST use this decomposition. *Test:*
  `test_ops_reference_decompositions`.
- **KISS-OPS-6.13-0002** — Each reference decomposition MUST reference only ops of
  strictly lower level (§6.14) together with the scalar-source leaves of §6.12; a
  decomposition MUST NOT reference a higher-level op or an equal-level op. *Test:*
  `test_ops_decomposition_strictly_lower_level`.
- **KISS-OPS-6.13-0003** — For every op marked **✓** in the *Refine* column of the §6.13
  table (`expm1`, `log1p`, `pow`, `hypot`, `ldexp`), a conforming kernel MAY compute a
  more accurate result than the literal reference decomposition (e.g. an overflow-safe
  or near-zero-accurate form, or an exact integer-exponent `ldexp`), but MUST agree with
  the reference decomposition's pinned mathematical meaning (the function it denotes)
  within the op's declared ULP; an op **not** marked MUST reproduce the reference under
  its determinism class. *Test:* `test_ops_decomposition_accuracy_refinement`.
- **KISS-OPS-6.13-0004** — A parameterized non-primitive op MUST carry its semantics-
  affecting attributes explicitly: `reduce_var` and `reduce_std` default to the
  population form and MUST declare a Bessel correction as an attribute rather than
  changing the decomposition silently; `softmax` / `log_softmax` MUST declare the
  normalization axis; `avg_pool` and `max_pool` MUST declare the per-axis `window_size`,
  `stride`, `dilation`, and `padding`, and `avg_pool` MUST declare `count_include_pad`
  (which selects the divisor between the full window count and the valid-tap count via
  `extent`); `rms_norm` / `layer_norm` / `matmul` MUST use the operand-ordering
  convention stated above the table. The pooled **window view** is a structured `gather`
  that adds one window axis whose taps lie at `dilation`-spaced offsets over
  `window_size` positions with the given `stride` and `padding`, OOB taps skipped.
  KISS-Ops MUST NOT let an unstated attribute change an op's pinned result. *Test:*
  `test_ops_parameterized_attributes_explicit`.
- **KISS-OPS-6.13-0005** — `pow` MUST be pinned over its full domain: for `a>0`,
  `pow(a,b)` equals the reference `exp(mul(b, log(a)))` (refinement permitted,
  §6.13-0003); `pow(0,0)` MUST yield `1`; `pow(+0.0, b)` for `b>0` MUST yield `+0.0` and
  for `b<0` MUST yield `+∞`; for `a<0`, `pow(a,b)` MUST yield NaN unless `b` is an exact
  integer, in which case `pow(a,b)` MUST yield `|a|^b` when `b` is even and `-(|a|^b)`
  when `b` is odd. *Test:* `test_ops_pow_full_domain`.
- **KISS-OPS-6.13-0006** — A reference-decomposition body MUST conform to this grammar: a
  body is either (a) a single expression tree over KISS-Ops ops and the §6.12 scalar-
  source leaves, or (b) a sequence of single-assignment let-bindings `name = expr;`
  followed by a final result expression, where each `expr` is a tree over KISS-Ops ops,
  the §6.12 scalar-source leaves, and previously-bound names. Each `name` MUST be bound
  exactly once (static single assignment), MUST NOT reference itself or a
  not-yet-bound name, and is scoped to the body; a mechanical resolver MUST parse the
  body as this tree/binding form and no other. *Test:* `test_ops_decomposition_body_grammar`.

### 6.14 Termination guarantee (recursive resolution)

- **KISS-OPS-6.14-0001** — Each op MUST have exactly one integer level: `level(op)=0`
  for every primitive-floor op and every scalar-source leaf, and `level(op)=1 + max(level
  of every op referenced in its reference decomposition)` for every non-primitive op.
  *Test:* `test_ops_level_assignment`.
- **KISS-OPS-6.14-0002** — The decomposition graph over all ops MUST be **acyclic**.
  *Test:* `test_ops_decomposition_acyclic`.
- **KISS-OPS-6.14-0003** — Recursive resolution of any op MUST terminate at the primitive
  floor in a finite number of steps bounded by the op's level; because levels strictly
  decrease along every decomposition edge and level 0 is the floor, no resolution can
  diverge. *Test:* `test_ops_resolution_terminates`.
- **KISS-OPS-6.14-0004** — A consumer that does not natively recognize a non-primitive op
  MUST be able to resolve it by expanding its reference decomposition recursively until it
  reaches ops it recognizes or the primitive floor, and MUST NOT be required to recognize
  the high-level op itself to obtain a defined result. *Test:*
  `test_ops_consumer_resolves_unknown_op`.
- **KISS-OPS-6.14-0005** — An op MUST NOT appear, directly or transitively, in its own
  reference decomposition. *Test:* `test_ops_op_not_in_own_decomposition`.
- **KISS-OPS-6.14-0006** — An op referenced inside a structural atom's per-element scalar
  body (the body of `element_map`, the pre-map or epilogue of `reduce`, or the body of
  `prefix_scan`) within a non-primitive op's reference decomposition MUST count toward
  that non-primitive op's level exactly as a decomposition reference: the enclosing op's
  level (§6.14-0001) is `1 + max(level of every op referenced anywhere in its
  decomposition, including ops embedded in a structural atom's scalar body)`. A
  primitive-floor structural atom itself carries an abstract body and remains level 0.
  *Test:* `test_ops_embedded_body_level`.

### 6.15 Load-bearing non-mergeable distinctions

- **KISS-OPS-6.15-0001** — `max_prop` and `min_prop` (NaN-propagating) and `fmax_ieee`
  and `fmin_ieee` (NaN-suppressing, returning the non-NaN operand when exactly one operand
  is NaN) MUST be four distinct ops; an implementation MUST NOT merge, alias, or
  substitute one for another. *Test:* `test_ops_minmax_nan_split`.
- **KISS-OPS-6.15-0002** — `relu` MUST be NaN-propagating and `-0.0`-preserving per its
  decomposition `select(cmp_lt(x, const(0)), const(0), x)`; an implementation MUST NOT
  implement `relu` as `max(x, 0)`, which would scrub a NaN input and normalize `-0.0` to
  `+0.0`. *Test:* `test_ops_relu_not_max_zero`.
- **KISS-OPS-6.15-0003** — `rem_floor` (floored remainder, taking the sign of the
  **divisor**) and `rem_trunc` (truncated remainder / `fmod`, taking the sign of the
  **dividend**) MUST be distinct ops; an implementation MUST NOT merge or substitute one
  for the other. *Test:* `test_ops_rem_floor_vs_trunc`.
- **KISS-OPS-6.15-0004** — `gelu` (exact erf-based) and `gelu_tanh` (the tanh
  approximation, differing by up to ~1e-4) MUST be distinct ops; an implementation MUST
  NOT treat one as the other or fold them into a single op. *Test:*
  `test_ops_gelu_exact_vs_tanh`.

### 6.16 Self-contained dtype bit layouts

So that a foreign reader holding only this document plus the umbrella can compute every
`const` bit pattern and evaluate every float clause, the bit layout of each dtype to
which KISS-Ops assigns numeric meaning is pinned here. This inlining creates **no**
dependency edge on the data-vocabulary sub-standard (§4); the dtype *tokens* remain a
shared naming convention spelled identically in both foundational vocabularies.

| dtype | bits | kind | Pinned layout / semantics |
|---|---|---|---|
| `f16` | 16 | float | IEEE-754 binary16 (1 sign, 5 exp, 10 mantissa), bias 15 |
| `bf16` | 16 | float | bfloat16 (1 sign, 8 exp, 7 mantissa), bias 127; the binary32 exponent range with a truncated mantissa; **not** an IEEE-754 format |
| `f32` | 32 | float | IEEE-754 binary32 storage; reduced-mantissa compute PERMITTED per target |
| `f32s` | 32 | float | IEEE-754 binary32 storage, byte-identical to `f32`; full-precision bit-stable multiply-add REQUIRED |
| `f64` | 64 | float | IEEE-754 binary64 (1 sign, 11 exp, 52 mantissa), bias 1023 |
| `s8` | 8 | int | signed 8-bit two's-complement |
| `u8` | 8 | uint | unsigned 8-bit; also the physical storage of `bool` |
| `i32` | 32 | int | signed 32-bit two's-complement |
| `i64` | 64 | int | signed 64-bit two's-complement |
| `u32` | 32 | uint | index/address dtype only (§6.2-0007); container width matches `i32` |
| `bool` | 8 | bool | 1 byte; `0`=false, any non-zero byte=true; ops normalize to strictly `0`/`1` |
| `e4m3` | 8 | float | FP8 E4M3 (1 sign, 4 exp, 3 mantissa), bias 7; max finite ±448; **no infinities**; a single NaN encoding; conversion saturates to max-finite, round-half-to-even (OCP OFP8) |
| `e5m2` | 8 | float | FP8 E5M2 (1 sign, 5 exp, 2 mantissa), bias 15; max finite ±57344; IEEE-style inf/NaN; conversion saturates to max-finite, round-half-to-even (OCP OFP8) |
| `s4` | 4 | int | signed 4-bit, range [−8,+7]; packed pair per byte, low nibble = even logical index, sign-extended on read (**storage packing owned normatively by the data-vocabulary sub-standard §6.1-0008/0009**; restated here informatively) |
| `u4` | 4 | uint | unsigned 4-bit, range [0,15]; packed pair per byte, low nibble = even index, zero-extended on read (**storage packing owned normatively by the data-vocabulary sub-standard §6.1-0008/0009**; restated here informatively) |
| `b1` | 1 | uint | 1-bit binary-GEMM operand; storage packing (8 bits/byte, LSB = lowest logical index) **owned normatively by the data-vocabulary sub-standard §6.1-0008/0009** and restated here informatively; the **xor+popcount accumulation to raw `s32` output** is the Ops-owned computation semantics |
| `c32` | 64 | complex | interleaved (re,im) pair of `f32` (storage only) |
| `c64` | 128 | complex | interleaved (re,im) pair of `f64` (storage only) |

- **KISS-OPS-6.16-0001** — The dtype set to which KISS-Ops assigns numeric or structural
  meaning MUST be exactly the tokens in the table above, each with the bit width and
  layout pinned there; these layouts are inlined normatively and create no dependency
  edge on the data-vocabulary sub-standard. *Test:* `test_ops_dtype_bit_layouts`.
- **KISS-OPS-6.16-0002** — `f16`, `f32`, `f32s`, and `f64` MUST use their IEEE 754-2019
  encodings (binary16 / binary32 / binary64), including the standard inf/NaN encodings,
  signed zero, and subnormals; `f32` and `f32s` MUST be byte-identical binary32 storage
  differing only in the required compute precision (§Appendix D.1). *Test:*
  `test_ops_ieee754_dtype_encodings`.
- **KISS-OPS-6.16-0003** — `bf16` MUST be encoded as a 1-sign / 8-exp / 7-mantissa format
  (bias 127) with the binary32 exponent range and a truncated mantissa; it is **not** an
  IEEE 754-2019 format, and any op producing a `bf16` result MUST round to nearest, ties
  to even. *Test:* `test_ops_bf16_layout`.
- **KISS-OPS-6.16-0004** — `e4m3` MUST be encoded per OCP OFP8 as 1-sign / 4-exp /
  3-mantissa (bias 7) with maximum finite magnitude ±448, **no** infinity encoding, and a
  single NaN encoding; conversion into `e4m3` MUST saturate to the maximum finite
  magnitude under round-half-to-even. *Test:* `test_ops_e4m3_layout`.
- **KISS-OPS-6.16-0005** — `e5m2` MUST be encoded per OCP OFP8 as 1-sign / 5-exp /
  2-mantissa (bias 15) with maximum finite magnitude ±57344 and IEEE-style infinity and
  NaN encodings; conversion into `e5m2` MUST saturate to the maximum finite magnitude
  under round-half-to-even. *Test:* `test_ops_e5m2_layout`.
- **KISS-OPS-6.16-0006** — The integer dtypes MUST use the pinned layouts:
  `s8`/`i32`/`i64` two's-complement; `u8`/`u32` unsigned (with `u32` index-only,
  §6.2-0007); `bool` one byte normalized to `0`/`1`. The sub-byte **storage packing**
  conventions — `s4`/`u4` packed two-per-byte with the low nibble at the even logical
  index (sign-extended for `s4`, zero-extended for `u4`), and `b1` packed 8-bits-per-byte
  LSB-first — are **restated here informatively and owned normatively by the sibling
  data-vocabulary sub-standard (its §6.1-0008/0009)**, which is the single normative home
  for sub-byte packing; KISS-Ops restates them only so this document is self-contained and
  MUST NOT be read as a second normative pinning of the packing facts. The Ops-owned
  normative fact in this clause is the `b1` **xor+popcount accumulation to a raw `s32`**
  binary-GEMM computation semantics (a computation fact, not storage packing). *Test:*
  `test_ops_integer_dtype_layouts`.
- **KISS-OPS-6.16-0007** — `c32` and `c64` MUST be carried as interleaved (re,im) storage
  pairs of `f32` and `f64` respectively (64 and 128 bits); this version of KISS-Ops
  declares **no** complex-arithmetic op, so `c32`/`c64` are storage-only dtypes and an op
  MUST NOT be defined to perform complex arithmetic on them. *Test:*
  `test_ops_complex_storage_only`.

---

## 7. Capability, Profile & Extension model

### 7.1 Mandatory core

- **KISS-OPS-7.1-0001** — The mandatory core of KISS-Ops MUST be the primitive floor of
  §6.3; a conforming implementation of KISS-Ops MUST evaluate the pinned semantics of
  every floor op regardless of which higher-level ops it also recognizes. *Test:*
  `test_ops_mandatory_core_is_floor`.
- **KISS-OPS-7.1-0002** — An input that names an op an implementation does not recognize
  MUST produce a typed decline or trigger reference-decomposition resolution (§6.14-0004),
  and MUST NOT cause a panic, abort, crash, hang, or out-of-bounds read. *Test:*
  `test_ops_unknown_op_typed_decline`.

### 7.2 Profiles and negotiated op subsets

- **KISS-OPS-7.2-0001** — A KISS-Ops profile MUST be identified by an integer `>= 1`
  denoting a mutually-versioned op-vocabulary feature-set; the value `0` MUST denote
  absence and MUST NOT be a live profile (consistent with the umbrella profile
  mechanism). *Test:* `test_ops_profile_integer`.
- **KISS-OPS-7.2-0002** — Any op-vocabulary subset an implementation claims MUST be
  prerequisite-closed under decomposition: claiming a non-primitive op does not force a
  consumer to natively implement it (it may resolve it), but the primitive floor MUST be
  present in every claim. *Test:* `test_ops_claim_includes_floor`.

### 7.3 Extension model — add by definition, guard the floor

- **KISS-OPS-7.3-0001** — Adding a new **high-level (non-primitive) op** MUST be
  additive: the new op MUST be introduced with a reference decomposition over the existing
  primitive floor (and existing lower-level ops), and its addition MUST NOT require any
  existing consumer to change, because the op resolves via its decomposition. *Test:*
  `test_ops_add_high_level_additive`.
- **KISS-OPS-7.3-0002** — Adding a new **primitive-floor op** MUST be treated as a
  mandatory-core (axiom) change that every conforming consumer must eventually implement;
  a new primitive MUST be admitted only when the function is genuinely inexpressible over
  the existing floor (a true atom — a special function with no elementary closed form or a
  hardware intrinsic with no decomposition). *Test:* `test_ops_add_primitive_is_axiom`.
- **KISS-OPS-7.3-0003** — An op-name assignment (core, experimental, or vendor tier) MUST
  originate from a merged change to the PR-gated KISS op registry maintained by the
  steward; an implementation MUST NOT rely on an unregistered op name, and a
  vendor/experimental op name MUST be namespaced so it cannot collide with a core op
  token. *Test:* `test_ops_op_registry_pr_gated`.

### 7.4 Determinism-class advertisement

- **KISS-OPS-7.4-0001** — An implementation MUST advertise, per op it supports, the
  determinism/fidelity class of §6.0, so that a consumer and KISS-Conform select the
  correct comparator; the class MUST be drawn from the single canonical enum owned by
  KISS-Ops and MUST NOT be re-forked. *Test:* `test_ops_determinism_class_advertised`.

---

## 8. Versioning & Lifecycle

KISS-Ops tracks the umbrella's **two version axes**: the **op-vocabulary schema version**
(the versioned op set + pinned semantics + primitive floor) and the published reference-
crate **semver**. They move independently.

- **KISS-OPS-8-0001** — The op-vocabulary schema version and the reference-crate semver
  MUST be tracked as independent axes; a crate semver change (documentation, performance,
  helper APIs) MUST NOT be taken to imply an op-vocabulary change. *Test:*
  `test_ops_two_version_axes_independent`.
- **KISS-OPS-8-0002** — Adding a new high-level (non-primitive) op or a new registered op
  name MUST be additive and MUST NOT bump the primitive-floor version, though it advances
  the op-vocabulary schema version. *Test:* `test_ops_additive_high_level_no_floor_bump`.
- **KISS-OPS-8-0003** — Adding, removing, or redefining a **primitive-floor op**, or
  changing any op's pinned numeric semantics (NaN behavior, signed-zero behavior, rounding
  mode, integer wrapping, shift semantics, combine algebra, OOB policy, determinism class,
  or a transcendental ULP ceiling), MUST bump the op-vocabulary schema version. *Test:*
  `test_ops_floor_or_semantics_change_bumps_version`.
- **KISS-OPS-8-0004** — A retired op token or a retired clause ID MUST NOT be reused; a
  retired op token MUST remain burned across versions. *Test:* `test_ops_retired_tokens_burned`.
- **KISS-OPS-8-0005** — KISS-Ops MUST NOT be promoted from Draft to Frozen until ≥2
  structurally dissimilar implementations agree on the pinned per-op semantics of the
  primitive floor and on the reference decompositions, evaluated by the KISS-Conform
  oracle-differential harness (which shares no lowering code with any reference impl)
  under each op's declared determinism class. *Test:* `test_ops_freeze_gate_two_impls`
  (checklist gate; AUDIT-signed, not DESIGN).
- **KISS-OPS-8-0006** — KISS-Ops MUST NOT be promoted from Draft to Frozen until an
  independent CPU oracle derived from the §6 semantics tables (not from any reference
  impl's lowering code) reproduces the pinned per-op results, and until this sub-
  standard's KISS-Conform suite exists and passes with complete bidirectional clause-to-
  test traceability (umbrella §5.3). *Test:* `test_ops_freeze_gate_conform_suite_passes`
  (checklist gate; AUDIT-signed).

---

## 9. Conformance

An implementation conforms to KISS-Ops at a given op-vocabulary schema version if it (a)
evaluates the pinned semantics of every primitive-floor op (§6.3) exactly as pinned in
§6.4–§6.12 and §6.16, (b) either natively implements or correctly resolves via reference
decomposition every claimed non-primitive op (§6.13–§6.14), (c) declines cleanly (never
panics) on unrecognized ops and on `u32` compute operands, and (d) passes the KISS-Conform
suite for KISS-Ops at that version.

KISS-Ops is a **foundational root**: it has no incoming DAG edge, so its DAG prerequisite
closure is empty and claiming KISS-Ops forces no upstream co-claim. The dtype tokens and
data-vocabulary nouns it references are a shared naming convention (§2.8, §4), not a
dependency; the dtype bit layouts it needs for its float clauses are inlined in §6.16.
Determinism-class assignment follows §6.0: transcendental atoms and any op transitively
containing one (that does not also depend on a float summation/contraction) are evaluated
under ULP/tolerance; every op whose result depends on a floating-point `sum`/`prod`
reduction or scan, a `matmul`/contraction, or a floating-point atomic combine (the
`scatter` `atomic-add` combine and `scatter_add`) under order-invariant/nondeterministic;
all remaining ops under exact-byte (§6.0-0002 through §6.0-0005). The umbrella mark policy
(umbrella §9.3) — a modified conformance suite does not back a claim — applies via registry
eligibility and is not restated as a free-standing KISS-Ops clause.

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-OPS-6.0-0001 | `test_ops_determinism_class_enum` |
| KISS-OPS-6.0-0002 | `test_ops_exact_byte_ops` |
| KISS-OPS-6.0-0003 | `test_ops_ulp_class_ops` |
| KISS-OPS-6.0-0004 | `test_ops_nondeterministic_class_ops` |
| KISS-OPS-6.0-0005 | `test_ops_determinism_class_precedence` |
| KISS-OPS-6.0-0006 | `test_ops_no_fma_contraction_exact_byte` |
| KISS-OPS-6.1-0001 | `test_ops_op_set_closed` |
| KISS-OPS-6.1-0002 | `test_ops_op_token_spelling` |
| KISS-OPS-6.1-0003 | `test_ops_op_family_tags` |
| KISS-OPS-6.1-0004 | `test_ops_primitive_flags` |
| KISS-OPS-6.2-0001 | `test_ops_float_ieee754` |
| KISS-OPS-6.2-0002 | `test_ops_int_wrapping` |
| KISS-OPS-6.2-0003 | `test_ops_default_nan_propagation` |
| KISS-OPS-6.2-0004 | `test_ops_signed_zero_preserved` |
| KISS-OPS-6.2-0005 | `test_ops_compare_result_zero_one` |
| KISS-OPS-6.2-0006 | `test_ops_bool_normalization` |
| KISS-OPS-6.2-0007 | `test_ops_u32_index_only` |
| KISS-OPS-6.2-0008 | `test_ops_const_bits_roundtrip` |
| KISS-OPS-6.2-0009 | `test_ops_nonprimitive_semantics_from_decomposition` |
| KISS-OPS-6.3-0001 | `test_ops_primitive_floor_set` |
| KISS-OPS-6.3-0002 | `test_ops_floor_is_mandatory_core` |
| KISS-OPS-6.3-0003 | `test_ops_floor_ops_have_no_decomposition` |
| KISS-OPS-6.3-0004 | `test_ops_decomposition_terminates_at_floor` |
| KISS-OPS-6.3-0005 | `test_ops_floor_ops_justified_atoms` |
| KISS-OPS-6.4-0001 | `test_ops_add_sub_mul` |
| KISS-OPS-6.4-0002 | `test_ops_div_float_only` |
| KISS-OPS-6.4-0003 | `test_ops_neg` |
| KISS-OPS-6.4-0004 | `test_ops_abs_raw_bit` |
| KISS-OPS-6.4-0005 | `test_ops_int_neg_abs_wrap` |
| KISS-OPS-6.5-0001 | `test_ops_select_order` |
| KISS-OPS-6.5-0002 | `test_ops_select_raw_bit` |
| KISS-OPS-6.5-0003 | `test_ops_select_cond_zero_nan` |
| KISS-OPS-6.5-0004 | `test_ops_select_no_mask_multiply` |
| KISS-OPS-6.6-0001 | `test_ops_compare_predicates` |
| KISS-OPS-6.6-0002 | `test_ops_compare_nan_false` |
| KISS-OPS-6.6-0003 | `test_ops_cmp_ne_nan_true` |
| KISS-OPS-6.6-0004 | `test_ops_compare_signed_zero` |
| KISS-OPS-6.7-0001 | `test_ops_rounding_directions` |
| KISS-OPS-6.7-0002 | `test_ops_rounding_nan_signed_zero` |
| KISS-OPS-6.8-0001 | `test_ops_transcendental_declared_ulp` |
| KISS-OPS-6.8-0002 | `test_ops_transcendental_no_cross_lang_identity` |
| KISS-OPS-6.8-0003 | `test_ops_sqrt_correctly_rounded_or_ulp` |
| KISS-OPS-6.8-0004 | `test_ops_special_function_atoms` |
| KISS-OPS-6.9-0001 | `test_ops_atan2_quadrants` |
| KISS-OPS-6.9-0002 | `test_ops_copysign_raw_bit` |
| KISS-OPS-6.9-0003 | `test_ops_nextafter_own_lattice` |
| KISS-OPS-6.10-0001 | `test_ops_bitwise_integer_only` |
| KISS-OPS-6.10-0002 | `test_ops_bitwise_logic` |
| KISS-OPS-6.10-0003 | `test_ops_shift_arithmetic_vs_logical` |
| KISS-OPS-6.10-0004 | `test_ops_shift_out_of_range_target_defined` |
| KISS-OPS-6.10-0005 | `test_ops_popcount_clz_ctz` |
| KISS-OPS-6.11-0001 | `test_ops_element_map_base_access` |
| KISS-OPS-6.11-0002 | `test_ops_reduce_monoids` |
| KISS-OPS-6.11-0003 | `test_ops_prefix_scan_length_preserving` |
| KISS-OPS-6.11-0004 | `test_ops_gather_oob_policy` |
| KISS-OPS-6.11-0005 | `test_ops_scatter_combine_and_oob` |
| KISS-OPS-6.11-0006 | `test_ops_scatter_fp_atomic_add_nondeterministic` |
| KISS-OPS-6.11-0007 | `test_ops_sort_network_total_order` |
| KISS-OPS-6.11-0008 | `test_ops_reduce_keepdim_broadcast` |
| KISS-OPS-6.11-0009 | `test_ops_index_dtype_set` |
| KISS-OPS-6.11-0010 | `test_ops_scatter_atomic_minmax_nan` |
| KISS-OPS-6.12-0001 | `test_ops_scalar_source_leaves` |
| KISS-OPS-6.12-0002 | `test_ops_const_leaf_bits` |
| KISS-OPS-6.12-0003 | `test_ops_named_constant_bits` |
| KISS-OPS-6.13-0001 | `test_ops_reference_decompositions` |
| KISS-OPS-6.13-0002 | `test_ops_decomposition_strictly_lower_level` |
| KISS-OPS-6.13-0003 | `test_ops_decomposition_accuracy_refinement` |
| KISS-OPS-6.13-0004 | `test_ops_parameterized_attributes_explicit` |
| KISS-OPS-6.13-0005 | `test_ops_pow_full_domain` |
| KISS-OPS-6.13-0006 | `test_ops_decomposition_body_grammar` |
| KISS-OPS-6.14-0001 | `test_ops_level_assignment` |
| KISS-OPS-6.14-0002 | `test_ops_decomposition_acyclic` |
| KISS-OPS-6.14-0003 | `test_ops_resolution_terminates` |
| KISS-OPS-6.14-0004 | `test_ops_consumer_resolves_unknown_op` |
| KISS-OPS-6.14-0005 | `test_ops_op_not_in_own_decomposition` |
| KISS-OPS-6.14-0006 | `test_ops_embedded_body_level` |
| KISS-OPS-6.15-0001 | `test_ops_minmax_nan_split` |
| KISS-OPS-6.15-0002 | `test_ops_relu_not_max_zero` |
| KISS-OPS-6.15-0003 | `test_ops_rem_floor_vs_trunc` |
| KISS-OPS-6.15-0004 | `test_ops_gelu_exact_vs_tanh` |
| KISS-OPS-6.16-0001 | `test_ops_dtype_bit_layouts` |
| KISS-OPS-6.16-0002 | `test_ops_ieee754_dtype_encodings` |
| KISS-OPS-6.16-0003 | `test_ops_bf16_layout` |
| KISS-OPS-6.16-0004 | `test_ops_e4m3_layout` |
| KISS-OPS-6.16-0005 | `test_ops_e5m2_layout` |
| KISS-OPS-6.16-0006 | `test_ops_integer_dtype_layouts` |
| KISS-OPS-6.16-0007 | `test_ops_complex_storage_only` |
| KISS-OPS-7.1-0001 | `test_ops_mandatory_core_is_floor` |
| KISS-OPS-7.1-0002 | `test_ops_unknown_op_typed_decline` |
| KISS-OPS-7.2-0001 | `test_ops_profile_integer` |
| KISS-OPS-7.2-0002 | `test_ops_claim_includes_floor` |
| KISS-OPS-7.3-0001 | `test_ops_add_high_level_additive` |
| KISS-OPS-7.3-0002 | `test_ops_add_primitive_is_axiom` |
| KISS-OPS-7.3-0003 | `test_ops_op_registry_pr_gated` |
| KISS-OPS-7.4-0001 | `test_ops_determinism_class_advertised` |
| KISS-OPS-8-0001 | `test_ops_two_version_axes_independent` |
| KISS-OPS-8-0002 | `test_ops_additive_high_level_no_floor_bump` |
| KISS-OPS-8-0003 | `test_ops_floor_or_semantics_change_bumps_version` |
| KISS-OPS-8-0004 | `test_ops_retired_tokens_burned` |
| KISS-OPS-8-0005 | `test_ops_freeze_gate_two_impls` |
| KISS-OPS-8-0006 | `test_ops_freeze_gate_conform_suite_passes` |

Every normative clause above appears in this matrix exactly once. Under umbrella §3.3 the
KISS-Conform build fails if any clause ID lacks a passing mapped test (bidirectional
traceability); that failure condition is the umbrella's rule, restated here informatively
rather than as a free-standing KISS-Ops clause. Clause IDs are mirrored in the machine-
readable sidecar (`kiss-ops.validusage.json` analog) kept in sync by the traceability
lint, and — because KISS-Ops is a plain-old-data vocabulary — both the prose tables and
the sidecar are generated from the canonical op schema so they cannot drift.

---

## 10. Governance

- **Editor of record:** the KISS-Ops editor assignment is **proposed, pending
  ratification** in the umbrella governance record (which does not yet name an editor for
  this sub-standard). The editor holds the pen, allocates clause IDs (append-only, never
  reused after retirement), maintains the PR-gated op registry, and solicits comment from
  interested cosignatories — any project building a consumer, provider, lifter, or emitter
  over the op vocabulary — before deciding a change.
- **Steward:** ThinkersJournal hosts the spec, the op registry (PR-gated), the namespace
  registry, and the conformance registry; it free-certifies self-certified implementations
  on request as resources permit.
- **Ratifier / maturity transitions:** the KISS-Conform AUDIT role (not DESIGN) signs each
  maturity transition; the Draft→Frozen transition requires the freeze gate of §8-0005 /
  §8-0006 (umbrella §5.3): ≥2 structurally dissimilar implementations agreeing on the
  pinned semantics and decompositions, an independent CPU oracle derived from the §6 tables
  (not from any reference impl), and a passing KISS-Conform suite with complete
  traceability.
- **License:** this specification is dedicated to the public domain under CC0 1.0
  Universal; reference crates are MIT-OR-Apache-2.0; the KISS-Conform suite is permissive-
  to-run. Per the umbrella mark policy (umbrella §9.3), a modified conformance suite does
  not back a conformance claim; that policy is enforced via steward-registry listing, not
  restated as a normative KISS-Ops clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC
  contribution, with defensive termination, per the umbrella.
- **Conformance posture:** self-certification with published results plus the steward-
  maintained registry is the authoritative record of verified implementations; the
  reference implementation runs the same public suite with no exemption.

---

## Appendix A — Worked resolution chains (informative)

**A.1 `gelu` (level 1).** `gelu(x) = mul(mul(const(0.5), x), add(const(1), erf(div(x,
const(sqrt2)))))`. Every referenced op — `mul`, `add`, `erf`, `div` — is a primitive-floor
atom; `const` is a scalar-source leaf. Resolution terminates in one expansion. A consumer
with a native `gelu` matches the node directly and never expands it. Determinism class:
ULP/tolerance (contains `erf`; no float summation).

**A.2 `layer_norm` (level 3).** `layer_norm` → `reduce_mean` (level 1, over `reduce`
level 0), `reduce_var` (level 2, over `reduce_mean`), `rsqrt` (level 1, over `sqrt` level
0), `sub`, `mul`, `add`. The deepest chain is `layer_norm` → `reduce_var` → `reduce_mean` →
`reduce`, three strictly-decreasing steps to the floor. Acyclic and finite. Determinism
class: order-invariant/nondeterministic (depends on float `sum` reductions, §6.0-0004/-0005).

**A.3 `relu` versus `max(x,0)`.** `relu(-0.0)` = `select(cmp_lt(-0.0, 0), 0, -0.0)`;
`cmp_lt(-0.0, 0)` is false (−0.0 is not less than 0), so the result is the raw-bit `-0.0`,
preserved. `max_prop(-0.0, 0)` would return `+0.0` (via `cmp_ge`), and `relu` of a NaN input
returns NaN (the `select` moves the raw NaN through the else arm) whereas `max_prop(NaN, 0)`
returns `NaN` too but `fmax_ieee(NaN,0)` returns `0` — which is exactly why `relu` is pinned
to `select`, not to any `max`.

**A.4 The min/max quartet on a NaN operand.** With `a = NaN`, `b = 3.0`: `max_prop(a,b) =
NaN` (propagate); `fmax_ieee(a,b) = 3.0` (suppress); `min_prop(a,b) = NaN`; `fmin_ieee(a,b)
= 3.0`. Four different results from four non-mergeable ops.

**A.5 `argmax` via `sort_network`.** `argmax(x)` reads the rank-0 entry of the
**original-index vector** output of `sort_network(desc, keys=x)` (§6.11-0007). Because the
sort is stable and descending, ties resolve to the first original index (argmax ties-to-
first), and the values output is not consumed — only the index vector is.

## Appendix B — Glossary (informative)

- **Atom** — a primitive-floor op with no in-standard decomposition (§6.3-0003, §6.3-0005).
- **Combine algebra** — the write-combine operator of `scatter`: `assign`, `atomic-add`,
  `atomic-max`, `atomic-min` (§6.11-0005).
- **Declared ULP** — a per-target accuracy bound (units in the last place) for a
  transcendental atom, no looser than the §6.8 ceiling, carried by a kernel's contract
  (§6.8-0001).
- **Determinism class** — the comparator class: exact-byte, ULP/tolerance, or
  order-invariant/nondeterministic (§6.0); the canonical enum is owned by KISS-Ops.
- **extent(axis)** — the scalar-source leaf giving the runtime logical length of an
  iteration axis (§6.12-0001).
- **Level** — `level(primitive)=0`; `level(op)=1+max(level of decomposition ops, including
  embedded scalar-body ops)` (§6.14-0001, §6.14-0006).
- **Monoid** — the associative fold operator of `reduce`/`prefix_scan`: `sum`, `prod`,
  `max`, `min` (identities in §6.11-0002).
- **NaN-propagating / NaN-suppressing** — returns NaN when any operand is NaN / returns the
  non-NaN operand (§6.15-0001).
- **OOB policy** — index out-of-bounds behavior: `skip` / `clamp` / `zero-fill` for reads,
  `skip` for writes (§6.11-0004, §6.11-0005).
- **Primitive floor** — the mandatory-core op set at which every decomposition terminates
  (§6.3).
- **Raw-bit move** — a bit-copy with no arithmetic, preserving signed zero and NaN payload
  (§6.4-0004, §6.5-0002, §6.9-0002).
- **Reference decomposition** — the canonical lower-level expression that defines a non-
  primitive op's meaning (§6.13).
- **Scalar-source leaf** — `input(i)`, `const(bits)`, `param(i)`, `coord(axis)`,
  `reduced(stage)`, `extent(axis)` (§6.12).

## Appendix C — Provenance / acknowledgments (informative)

The op vocabulary, per-op semantics, and reference decompositions derive from the reference
seed crates `baracuda-kernel-vocab` and `baracuda-kernelgen` (Evans Laboratories project;
non-normative provenance). The load-bearing edge-case pins (the min/max NaN split, `relu`
≠ `max(x,0)`, `rem_floor` versus `rem_trunc`, `gelu` versus `gelu_tanh`, raw-bit `select`,
and the declared-ULP transcendental-atom posture) trace to the neutral-consumer analysis
recorded in the KISS Design Charter §4/§4a. Project and crate names in this appendix and in
§0 are non-normative provenance and reference-implementation pointers only; no normative
clause names any project or steward organization — normative clauses use only the generic
roles provider, consumer, implementation, kernel, contract, target, and steward.

## Appendix D — Open questions (informative)

These are recorded for the KISS-Ops / data-vocabulary RFC and do not bind conformance:

1. **`f32` versus `f32s`.** Byte-identical binary32 storage distinguished only by required
   compute precision (reduced-mantissa-permitted versus full-precision bit-stable multiply-
   add). Whether compute precision belongs in the dtype set at all, or moves to the
   contract's guarantees as a math-precision attribute leaving dtypes purely storage. If it
   stays a dtype, both foundational vocabularies must pin that `f32`/`f32s` share byte
   layout and differ only in the numeric-fidelity contract.
2. **`u32` index-only.** Whether the dtype set formally tags index-only dtypes as a distinct
   class, or whether index-only is an operand-role property carried on the gather/scatter
   index operand (this document pins it as a role restriction, §6.2-0007, with the legal
   index-dtype set `{u32, i32, i64}` in §6.11-0009).
3. **Sub-byte packing (`s4`, `u4`, `b1`).** KISS-Ops restates the packing layout in §6.16
   informatively because it is adjacent to the `popcount`/binary-GEMM semantics, but
   §6.16-0006 now defers **normative** ownership of the sub-byte storage packing to the
   sibling data-vocabulary sub-standard (§6.1-0008/0009), keeping only the `b1`
   xor+popcount→raw-`s32` accumulation as the Ops-owned computation fact. This resolves the
   earlier dual-pinning so exactly one standard carries the normative packing clause;
   whether the two spellings should be kept byte-identical (shared anchor) remains an RFC
   item, but the ownership boundary is no longer ambiguous.
4. **Complex dtypes (`c32`, `c64`).** This version declares **no** complex-arithmetic ops
   (§6.16-0007); `c32`/`c64` are carried as spectrum-domain storage dtypes only.
5. **Transcendental atoms as declared-ULP with a normative ceiling.** The §6.8 ceilings
   (4 ULP for the elementary transcendentals, 8 ULP for `lgamma`, correctly-rounded-or-2-ULP
   for `sqrt`) are reference values pending confirmation against real per-target math
   libraries; they make §6.8-0001 testable at the KISS-Ops layer without mandating a
   reference polynomial.
6. **Float reduction determinism.** This version classes float `sum`/`prod` reductions,
   scans, and contractions as order-invariant/nondeterministic (§6.0-0004) rather than
   pinning a canonical reduction order and accumulator width. Whether a future profile
   should offer an opt-in byte-exact reduction (pinned low-to-high order + accumulator
   dtype) is open.
7. **`extent(axis)` leaf.** Added to the scalar-source leaf set (§6.12-0001) to source
   shape-derived divisors (`reduce_mean`); whether the shared anchor's leaf list should
   adopt `extent(axis)` identically is an RFC item for the data-vocabulary sibling.
8. **`reduce` keepdim broadcast.** §6.11-0008 pins reduced axes as extent-1 / stride-0
   keepdim views; the `reduce_axes` empty-mask sentinel (last/trailing axis versus
   undetermined/non-reduction) remains a data-vocabulary encoding KISS-Ops references but
   does not own.
9. **Determinism-class enum ownership.** This version makes KISS-Ops the owner of the
   single canonical enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`
   (§6.0-0001), imported downstream by KISS-Synth and KISS-Conform, resolving the prior
   upstream-import cycle.
10. **Umbrella cross-reference (reconciled 2026-07-12).** Two corrections to the umbrella,
    now applied: (a) the umbrella §2.1 description of KISS-Ops reads "IEEE-fmax versus
    NaN-propagating-max" (matching §2.3/§6.15 here), not "saturating-max"; and (b) the
    umbrella §2.1 now names **KISS-Ops** as owner of the canonical determinism/fidelity enum
    `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` (§6.0-0001, Appendix D.9),
    with KISS-Synth, KISS-Emit, and KISS-Conform importing it — placing ownership in a
    foundational vocabulary so no lower tier imports upward from a protocol-tier sub-standard.

---

*End of KISS-Ops (Draft proposal). §0–§5 are informative; §6+ are normative. Every binding
requirement is an identified clause with a mapped KISS-Conform test. KISS-Ops depends on no
other sub-standard; it is the foundational computation vocabulary, consumed structurally by
KISS-Grammar, KISS-Contract, KISS-Synth/Provision, KISS-Consume, and KISS-Emit. Project and
product names appear only in non-normative examples, provenance, and reference-implementation
pointers; normative clauses use only the generic roles provider, consumer, implementation,
kernel, contract, target, and steward.*
