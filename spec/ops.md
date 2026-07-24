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

KISS-Ops also owns two attributes that live alongside the op semantics: the **compute-
fidelity (MathPrecision) attribute** (`{bit-stable, reduced-mantissa-permitted}`, §6.17),
orthogonal to the determinism class and the home of compute precision now that storage
dtypes are pure byte layout; and the **complex-arithmetic op family** for `c32`/`c64`
(§6.18), every member non-primitive over the real floor (no new axiom), pinned to ISO
C99/C11 Annex G. It further owns the **OpAttrs channel** (§6.19) — the per-op,
compile-time attribute record that is part of an op's semantics (a reduce's monoid and
axis set, a gather's out-of-bounds policy, a pool's window geometry, and the rest) — and
its canonical, default-resolved little-endian wire encoding, embedded by KISS-Grammar and
KISS-Contract as opaque, byte-comparable bytes and distinct from KISS-Grammar
`pattern_attrs` (matching hints).

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
**declared accuracy tier** (KISS-Contract §6.8-0002) — a tagged quantity carrying at
least one of `{max_ulp, max_relative, max_absolute}` — which is the **sole** conformance
gate; §6.8 carries an *informative advisory floor*, not a normative ceiling. KISS-Ops
does **not** mandate a reference polynomial and does **not** claim cross-language bit
identity for them — `sin` under one target's math library differs in the last bit from
another's. Mandating a polynomial would over-specify while still failing to deliver
cross-language identity, so the semantics is "the named function to within its declared
per-target accuracy tier," and the determinism class is ULP/tolerance, not exact-byte.

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

**Complex-arithmetic ops (over `c32`/`c64`; all non-primitive, resolving to the REAL floor
via §6.18):** `cadd`, `csub`, `cneg`, `cconj`, `cmul`, `cdiv`, `cabs`, `carg`, `cexp`,
`clog`, `csqrt`, `cpow` are the **advertised high-level** complex ops; `cmake`
(complex-construct from re,im), `cre` (real extract), and `cim` (imag extract) are the
component-bridge plumbing (not advertised high-level). Every complex op decomposes into the
real primitive-floor atoms (`add`/`sub`/`mul`/`div`/`neg`/`sqrt`/`exp`/`log`/`sin`/`cos`/
`atan2`/`copysign`) plus `element_map`, the real non-primitive `hypot`, and lower-level
complex ops of the family, so the complex family introduces **no** new primitive (the §6.3
floor is unchanged). Complex semantics are pinned
to ISO C99/C11 Annex G (§6.18).

**Carrier ops and their OpAttrs (informative).** Several ops carry an **OpAttrs** record —
the per-op, compile-time attributes that select which member of the op's parameterized
family a node denotes (a reduce's `monoid` and `reduce_axes`, a gather's `oob_policy`, a
pool's window geometry, and so on). The carrier ops for this version are `reduce`,
`prefix_scan`, `gather`, `scatter`, `sort_network`, `reduce_var`, `reduce_std`, `softmax`,
`log_softmax`, `rms_norm`, `layer_norm`, `avg_pool`, `max_pool`, `im2col`, `index_select`,
`embedding`, and `scatter_add`. Their schemas, sub-vocabularies, and canonical
default-resolved little-endian wire encoding are pinned normatively in **§6.19**; worked
golden vectors are in Appendix E.

### 2.8 Terms are joined, not restated

KISS-Ops references the **dtype** tokens (`f16 bf16 f32 f64 s8 s16 u8 u16 i32 i64
u32 u64 bool e4m3fn e4m3fnuz e5m2 e5m2fnuz s4 u4 b1 c32 c64`), the **operand descriptor** field names (`rank`,
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
  `input(i)`, `const(bits)`, `param(i)`, `coord(axis)`, `reduced(stage)`, `extent(axis)`,
  and `reduced_count` (the product of extents over all reduced axes; the multi-axis mean
  divisor, §6.12-0001).
- **Compute dtype** — the dtype in which an op's arithmetic is performed, one of the
  pinned storage dtype set (§6.16). `u32` is an ordinary dtype and is **not** excluded;
  index-only-ness is an operand role (§6.11-0012), not a dtype class. A complex op's
  compute dtype is the `f32`/`f64` component of its `c32`/`c64` operand (§6.18-0015).
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
- **Compute-fidelity (MathPrecision) attribute** — a two-member enum
  `{bit-stable, reduced-mantissa-permitted}` owned by KISS-Ops (§6.17), orthogonal to the
  determinism class, recording whether an op's floating-point arithmetic is computed at
  full storage precision with each atom rounding independently (**bit-stable**) or MAY use
  a reduced-mantissa fast path (**reduced-mantissa-permitted**). It is the home of the
  compute-precision meaning formerly modeled as a strict-precision float dtype; storage
  dtypes carry no compute-precision meaning. Surfaced in a kernel's KISS-Contract
  guarantees.
- **Index operand role** — the operand-level role (carried by the `index_operand` and
  `index_dtype` fields, §6.11-0012) marking which operand of `gather` / `scatter` /
  `index_select` / `embedding` / `scatter_add` supplies the runtime index, and that
  operand's dtype. Index-only-ness is this role, not a dtype class; `u32` is an ordinary
  dtype that MAY serve it.
- **OpAttrs** — the per-op, compile-time attribute record that is part of an op's
  semantics (the fixed-at-build-time choices that select which member of an op's
  parameterized family a node denotes, e.g. a reduce's `monoid` and `reduce_axes`, a
  gather's `oob_policy`, a pool's window geometry). KISS-Ops is its single normative owner
  (§6.19). It is **distinct** from KISS-Grammar `pattern_attrs`, which are recognition/
  matching hints on the advertisable surface and do not change what an op computes.
- **Carrier op** — an op that carries a non-empty OpAttrs schema (§6.19); every other op
  has an empty OpAttrs blob.
- **OpAttrs sub-vocabulary** — a frozen enum or encoding (e.g. `oob_policy`, `monoid`,
  `reduce_axes`, the window-parameter vector, the reserved permutation encoding) that an
  OpAttrs field draws from, owned by KISS-Ops and pinned in §6.19; enums are additive-only
  little-endian unsigned ordinals with `0` reserved.
- **Canonical OpAttrs blob** — the default-resolved, per-op fixed-field-order, little-
  endian byte string encoding a carrier op's OpAttrs (§6.19); every field is emitted
  explicitly at its effective value, so a defaulted attribute and an explicitly-equal one
  produce identical bytes, and KISS-Grammar / KISS-Contract embed it as opaque bytes and
  byte-compare it.
- **Complex op** — a non-primitive op of the §6.18 complex-arithmetic family over
  `c32` / `c64`, decomposing entirely into the real primitive-floor atoms plus the
  `element_map` component bridge; it introduces no new primitive.
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
  IEEE-754 dtypes (`f16`, `f32`, `f64`) only; `bf16` and the FP8 formats
  `e4m3fn` / `e5m2` are **not** IEEE-754 formats and are pinned explicitly in §6.16.
- **Open Compute Project (OCP) 8-bit Floating Point Specification (OFP8), FP8 formats
  E4M3 and E5M2** — the normative reference for the `e4m3fn` and `e5m2` encodings,
  saturation, and NaN/infinity conventions restated in §6.16. `bf16` (bfloat16) is
  pinned directly in §6.16 as a truncated binary32 layout with round-to-nearest-even.
- **ISO/IEC 9899 (C99 or C11) Annex G — "IEC 60559-compatible complex arithmetic"** —
  the normative reference for the **complex-arithmetic op family** (§6.18) over the
  `c32` / `c64` dtypes: the principal branch cuts of `clog` / `csqrt` / `carg` / `cpow`,
  the signed zero of the real and imaginary components, and the infinity/NaN recovery
  rules restated in §6.18 (the `cmul` / `cdiv` Annex-G recovery of §6.18-0005 / §6.18-0006,
  applied with the Annex-G trigger verbatim). Every §6.18 op is non-primitive over the
  real primitive floor; Annex G governs only the complex edge cases the real
  decomposition alone does not fix.
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
  declared ULP of each transcendental atom (bounded by the §6.8 ceiling); the contract's
  guarantees section surfaces the KISS-Ops-owned compute-fidelity (MathPrecision) attribute
  (§6.17), imported from KISS-Ops, not re-forked.
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
  `nextafter`, the algebraic complex ops (`cadd`, `csub`, `cneg`, `cconj`, `cmul`, `cdiv`,
  `cmake`, `cre`, `cim`), `element_map`, `gather`, `scatter` with a deterministic combine, and the
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
  enumerated in §2.7 / §6.4–§6.13 / §6.18 (the primitive-floor ops of §6.3, the
  non-primitive ops of §6.13, and the complex-arithmetic ops of §6.18); an implementation
  MUST NOT treat a token outside this set as a KISS-Ops op of this version. *Test:*
  `test_ops_op_set_closed`.
- **KISS-OPS-6.1-0002** — Each op MUST be spelled as the exact token given in this
  document (case-sensitive, underscore-delimited); an implementation MUST NOT accept a
  synonym, alias, or alternative spelling as the same op. *Test:*
  `test_ops_op_token_spelling`.
- **KISS-OPS-6.1-0003** — Each op MUST carry exactly the op-family tag assigned to it in
  §2.7; an implementation MUST NOT re-classify an op into a different family. *Test:*
  `test_ops_op_family_tags`.
- **KISS-OPS-6.1-0004** — Each op MUST carry exactly the primitive-floor membership flag
  assigned to it (primitive = in §6.3; non-primitive = in §6.13 or §6.18); an
  implementation MUST NOT treat a non-primitive op as primitive or a primitive op as
  non-primitive. *Test:*
  `test_ops_primitive_flags`.

### 6.2 Shared numeric conventions

- **KISS-OPS-6.2-0001** — For every op whose compute dtype is an IEEE-754 float dtype
  (`f16` binary16, `f32` binary32, `f64` binary64), the arithmetic MUST follow
  IEEE 754-2019 for that operation and dtype, except where a specific op clause below
  pins a departure (e.g. the NaN-suppressing min/max family, the declared-ULP
  transcendental atoms). For the non-IEEE-754 float dtypes `bf16`, `e4m3fn`, and `e5m2`,
  the arithmetic MUST follow the encodings, rounding, saturation, and NaN/infinity
  conventions pinned in §6.16 (these formats are **not** governed by IEEE 754-2019).
  *Test:* `test_ops_float_ieee754`.
- **KISS-OPS-6.2-0002** — For every op whose compute dtype is an integer dtype (`s8`,
  `s16`, `u8`, `u16`, `u32`, `u64`, `i32`, `i64`, `s4`, `u4`), integer `add`/`sub`/`mul` MUST be **wrapping** two's-
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
- **KISS-OPS-6.2-0007** — The `u32` dtype MUST be treated as an **ordinary** unsigned
  32-bit integer storage/compute dtype (wrapping two's-complement per §6.2-0002), accepted
  on the integer arithmetic, comparison, bitwise, reduction, and scan paths like any other
  unsigned integer dtype; KISS-Ops MUST NOT define an index-only dtype class, and
  index-only-ness MUST be modeled as an operand role on the index operand (§6.11-0012),
  never as a property of the `u32` dtype. *Test:* `test_ops_u32_ordinary_dtype`.
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
  outside the union of the primitive floor, the non-primitive op set (§6.13, §6.18), and
  the scalar-source leaves. *Test:* `test_ops_decomposition_terminates_at_floor`.
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
`erf` (Gauss error function), `atan` (arctangent), `atan2` (two-argument arctangent),
and `lgamma` (`ln|Γ(x)|`). `atan2` is structurally an op-family `binary_math` atom
(§6.9) but is a **declared-ULP transcendental atom** for accuracy and determinism-class
purposes: it appears in the advisory-floor table below and is class ULP/tolerance
(§6.0-0003, §6.8-0005), never exact-byte. Each atom's accuracy is governed by the
**per-target accuracy tier its kernel's contract declares** (KISS-Contract §6.8-0002),
which KISS-Conform evaluates against an audited reference; the declared tier is the
**sole** normative gate. The table below is an *informative advisory floor* — a
reasonableness reference drawn from incumbent (Khronos/OpenCL) practice, **not** a
normative cap (the normative requirement is carried by KISS-OPS-6.8-0001; this
section-intro paragraph is an informative pointer to it):

| Atom | Advisory-floor ULP — *informative*, typical incumbent-conformant value (compute dtype ≥ 16-bit float); **not** a normative cap |
|---|---|
| `sqrt` | 0.5 ULP (correctly rounded) where the target guarantees it, else 2 ULP |
| `exp`, `log`, `sin`, `cos`, `atan`, `atan2` | 4 ULP |
| `erf` | 4 ULP |
| `lgamma` | 8 ULP |

- **KISS-OPS-6.8-0001** — Each transcendental atom MUST compute its named mathematical
  function to within the **per-target accuracy tier its kernel's contract declares** for
  that target (KISS-Contract §6.8-0002); the declared tier is the **sole** conformance
  gate. The accuracy tier is a tagged quantity carrying at least one of `{max_ulp,
  max_relative, max_absolute}`, and KISS-Conform MUST evaluate the atom against the
  declared tier under the ULP/tolerance determinism class (§6.0-0003), measured against an
  audited wide-precision reference — never a byte-exact comparison across languages or
  targets. KISS-Conform MUST NOT impose a fixed suite-wide ULP cap and MUST NOT reject a
  declared tier for exceeding the §6.8 advisory-floor table (that table is *informative* —
  a reasonableness reference, not a normative threshold; a truthful Khronos-conformant
  provider whose atom exceeds a table value MUST NOT be rejected for it). KISS-Ops MUST NOT
  mandate a specific reference polynomial or table for a transcendental atom. *Test:*
  `test_ops_transcendental_declared_tier_is_gate`.
- **KISS-OPS-6.8-0002** — KISS-Ops MUST NOT claim cross-language or cross-target bit
  identity for any transcendental atom; conformance for these atoms MUST be evaluated
  under the ULP/tolerance determinism class (§6.0-0003), and byte-exact identity MUST be
  claimed only same-language on-device (deferred to KISS-Emit / KISS-Conform). *Test:*
  `test_ops_transcendental_no_cross_lang_identity`.
- **KISS-OPS-6.8-0003** — `sqrt` MUST be correctly rounded per IEEE 754-2019 on any target
  that guarantees correctly-rounded square root, and MUST otherwise meet its **declared
  per-target accuracy tier** (§6.8-0001). *Test:* `test_ops_sqrt_correctly_rounded_or_ulp`.
- **KISS-OPS-6.8-0004** — `erf` and `lgamma` MUST be treated as special-function atoms
  with no elementary decomposition over other KISS-Ops ops; an implementation MUST NOT
  require them to be expressed via `exp`/`log`/etc. *Test:*
  `test_ops_special_function_atoms`.
- **KISS-OPS-6.8-0005** — `atan2` MUST be assigned the **ULP/tolerance** determinism
  class (§6.0-0003) and MUST NOT be assigned the exact-byte class or evaluated with a
  byte-exact comparator: although `atan2` is an op-family `binary_math` atom (§6.9), it
  is a declared-ULP transcendental atom (this section, 4-ULP advisory floor), so the exact-byte
  "if and only if" of §6.0-0002 MUST NOT apply to it (its condition (a) excludes any op
  containing a §6.8 transcendental atom) and no clause MUST require its byte-exact
  reproduction across targets — consistent with `carg`, which is derived from `atan2`
  (§6.18-0008) and whose determinism class is ULP/tolerance (§6.18-0014). *Test:*
  `test_ops_atan2_class_is_ulp`.
- **KISS-OPS-6.8-0006** — The v1 accuracy tier (§6.8-0001) is a **flat, argument-independent**
  quantity over the atom's declared input domain: a single `{max_ulp | max_relative |
  max_absolute}` per target, not a function of the argument. KISS-Ops **reserves** — as a
  **named post-v1 accuracy-model extension**, NOT required for v1 conformance — an
  *argument-dependent / range-scoped* form: accuracy expressed as a function of input
  magnitude (e.g. `3 + 2·|x|` ULP) or as an absolute error over a bounded range (e.g.
  `≤ 2⁻¹¹` on `[−π, π]`), the forms incumbent tables (Vulkan `sin`/`cos`/`exp`) already
  use. A v1 kernel whose true accuracy is argument-dependent MUST declare a flat tier that
  bounds it over the declared input domain; KISS-Conform MUST NOT require the reserved form
  in v1. *Test:* `test_ops_accuracy_tier_flat_v1`.

### 6.9 Binary-math atoms

- **KISS-OPS-6.9-0001** — `atan2` MUST compute the four-quadrant arctangent of `(y=a,
  x=b)` with the IEEE 754-2019 ±0 quadrant conventions (the sign of a zero operand selects
  the quadrant). *Test:* `test_ops_atan2_quadrants`.
- **KISS-OPS-6.9-0002** — `copysign` MUST produce a value with the magnitude of `a` and
  the sign bit of `b` as a raw-bit operation, so that the signed zero of `b` and the sign
  of a NaN `b` are carried into the result. *Test:* `test_ops_copysign_raw_bit`.
- **KISS-OPS-6.9-0003** — `nextafter` MUST produce the next representable value after `a`
  toward `b` in the **dtype's own** representation lattice; `nextafter` MUST reject `f16`,
  `bf16`, `e4m3fn`, and `e5m2` operands (each carries the identical promotion hazard —
  stepping in a promoted `f32` yields the wrong neighbor in the narrow lattice) with a
  typed decline. *Test:* `test_ops_nextafter_own_lattice`.

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
- **KISS-OPS-6.10-0006** — For every §6.10 bitwise, shift, or bit-count atom whose
  operand compute dtype is a **sub-32-bit** integer dtype (`s8`, `u8`, `s16`, `u16`,
  `s4`, `u4` — any width narrower than 32 bits), the value each operand contributes and
  the stored result MUST be the operand/result **truncated to that operand dtype's own
  bit width** (mod `2^bitwidth`): the atom MUST behave as promote-to-width → apply →
  **truncate-on-store**, and its observable result MUST be **independent of DAG
  sharing/hoisting**. Because C-family lowering promotes a narrow operand to `int`
  (32-bit) before the atom, a *composed* narrow operand can be observed as its
  un-truncated promoted value when the producing sub-expression is inlined but as its
  truncated narrow temporary when that sub-expression is hoisted; to remove this
  ambiguity an implementation MUST either (a) truncate every narrow operand to its own
  bit width before the atom consumes it (so the operand carries its on-store value
  regardless of sharing), or (b) require every operand of a narrow §6.10 atom to be a
  direct load (a leaf input) and never a composed sub-expression. This clause pins the
  operand/result *value* only; it does not widen the pinned shift domain of §6.10-0004
  (a shift amount outside `[0, bitwidth)` remains target-defined). *Test:*
  `test_ops_narrow_int_promote_truncate_composition`.

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
- **KISS-OPS-6.11-0007** — `sort_network` MUST carry a `direction` attribute in
  `{ascending, descending}` (default `ascending`) and MUST perform a **stable** per-row
  permutation under a total order on `(key, original-index)` pairs in which NaN orders as
  the greatest value (ascending → NaN last, descending → NaN first) and ties break by the
  lower original index (the stability rule). It MUST expose **two** outputs: (a) the values
  written back as a raw-bit permutation, and (b) the **original-index vector** — for each
  output rank, the source position it came from. It MUST be treated as a structural atom
  with no monoid. (`argmax` reads rank 0 of the index vector under `direction=descending`,
  §6.13 table.) *Test:* `test_ops_sort_network_total_order`.
- **KISS-OPS-6.11-0008** — `reduce` MUST retain each reduced axis as an extent-`1` axis
  with stride `0` (a keepdim result) so the reduced value broadcasts back over the
  original axis via extent-1 / stride-0, so a shifted-shape decomposition such as
  `sub(x, reduce(max, x))` reads the reduced value at every position of the original axis;
  `prefix_scan` MUST retain all axes (length-preserving). *Test:*
  `test_ops_reduce_keepdim_broadcast`.
- **KISS-OPS-6.11-0009** — The legal `index_dtype` set for the index operand of `gather`,
  `scatter`, `index_select`, `embedding`, and `scatter_add` MUST be exactly `{u32, i32,
  i64}`; an implementation MUST reject any other `index_dtype` with a typed decline. This
  is a restriction on the **index-operand role** (§6.11-0012), not on the dtypes elsewhere:
  `u32`, `i32`, and `i64` remain ordinary compute dtypes in every other operand position.
  The negative-index-is-out-of-bounds rule (§6.11-0004) applies to the signed index dtypes
  `i32` and `i64`; `u32` carries no negative value and the rule is vacuous for it. *Test:*
  `test_ops_index_dtype_set`.
- **KISS-OPS-6.11-0010** — The `scatter` floating-point `atomic-max` and `atomic-min`
  combines MUST be **NaN-propagating**, consistent with the `max` / `min` `reduce`
  monoids (§6.11-0002): if a NaN is scattered to a destination, or a destination already
  holds a NaN, the combined result MUST be NaN. *Test:*
  `test_ops_scatter_atomic_minmax_nan`.
- **KISS-OPS-6.11-0011** — `reduce` and `prefix_scan` MUST carry a `reduce_axes` descriptor
  that distinguishes **exactly four** non-overloaded categories of value — **all-axes**,
  **trailing-axis**, **none / not-a-reduction**, and an **explicit per-axis subset mask** —
  such that no single encoding is overloaded to mean more than one category. The
  **reduce-field encoding** is owned normatively by the data-vocabulary `structure_key`
  **token codec** (Classify §6.7-0005): the string tokens `-` (none / not-a-reduction),
  `rall` (all-axes), `rlast` (trailing-axis), and `x<hh>` (an 8-bit keepdim subset mask
  written as exactly two lowercase hex digits `00..ff`), which bumps
  `STRUCTURE_KEY_VERSION` for this split; **no binary/byte wire form is defined at this
  schema version** (Classify §6.7-0011). Those tokens are **restated here informatively** as
  the shared anchor, not a second normative pinning. The Ops-normative content of this clause
  is the **reduce/scan semantics** of each category and the emission constraints: (a)
  **all-axes** (`rall`) → fold over every axis; (b) **trailing-axis** (`rlast`) → fold over
  the single last (trailing) axis; (c) **none / not-a-reduction** (`-`) → not a reduction,
  and a `reduce` op MUST NOT be emitted in the none category; (d) **subset mask** (`x<hh>`)
  → fold over exactly the axes it selects (bit `k` selects axis `k`). For the subset-mask
  category to be expressible the encoding requires `MAX_RANK <= 8` (a u8 subset field, one
  bit per axis); KISS-Ops MUST NOT emit a `reduce` / `prefix_scan` whose selected-axis set is
  not representable in that field, and the `rall` / `rlast` / `-` tokens MUST remain disjoint
  from every legal `x<hh>` subset mask so no subset value can collide with a category token.
  For illustration only, an Ops-local **in-memory** rendering MAY model the field as a 16-bit
  value with `0xFFFF` = all-axes, `0xFFFE` = trailing-axis, `0x0000` = none, and
  `0x0001..0x00FF` = the u8 per-axis subset mask (the intervening `0x0100..0xFFFD` reserved
  and never emitted); this hex is an Ops-local in-memory illustration, **not** the
  data-vocabulary shared anchor and **not** a wire form — the normative shared anchor is the
  Classify §6.7-0005 token codec above. *Test:*
  `test_ops_reduce_axes_four_categories`.
- **KISS-OPS-6.11-0012** — `gather`, `scatter`, `index_select`, `embedding`, and
  `scatter_add` MUST carry the index-operand role explicitly on the operand: an
  `index_operand` field identifying which operand supplies the runtime index, and an
  `index_dtype` field giving that operand's dtype (in the legal set of §6.11-0009). The
  index role MUST be a property of that operand, not of any dtype; KISS-Ops MUST NOT infer
  index-ness from a dtype (in particular `u32` is not an index-only dtype, §6.2-0007).
  *Test:* `test_ops_index_operand_role`.
- **KISS-OPS-6.11-0013** — `gather`, `scatter`, `index_select`, `embedding`, and
  `scatter_add` MUST each carry an explicit **indexed-axis** attribute `axis` naming the
  single operand-or-output axis whose coordinate the runtime index substitutes (the axis
  addressed by §6.11-0004 for a data-dependent read and by §6.11-0005 for a data-dependent
  write). The `axis` attribute MUST be a non-negative integer in the range
  `0 <= axis < MAX_RANK` (the pinned `MAX_RANK` value of §6.19-0037), and MUST be carried
  as the `u8` OpAttrs `axis` field of §6.19-0027 / §6.19-0028 / §6.19-0034; an
  implementation MUST reject an `axis` outside that range with a typed decline, and MUST
  NOT let an unstated indexed axis change the op's pinned result. *Test:*
  `test_ops_index_axis_attribute`.
- **KISS-OPS-6.11-0014** — `sort_network` MUST carry an explicit `axis` attribute naming
  the single axis along which each row is permuted (§6.11-0007 pins a per-row permutation;
  this clause pins which axis a row lies along, its width, and its legal range). The `axis`
  attribute MUST be a non-negative integer in the range `0 <= axis < MAX_RANK` (§6.19-0037)
  and for this op-set version MUST resolve to the **trailing (innermost) axis** `r-1` of a
  sorted operand of rank `r`; it MUST be carried as the `u8` OpAttrs `axis` field of
  §6.19-0029. An implementation MUST reject an `axis` outside the range with a typed
  decline. *Test:* `test_ops_sort_network_axis_attribute`.
- **KISS-OPS-6.11-0015** — Under the `skip` out-of-bounds policy (§6.11-0004,
  §6.19-0015) `gather` MAY carry an **optional `base` operand** — a child-edge read at
  the output position of any index that is skipped. The output element at a skipped
  (out-of-bounds) position MUST take the value of the `base` operand at the
  corresponding output position (a read-modify-passthrough), while every in-bounds
  position MUST take the gathered value unchanged; `clamp` and `zero-fill` do not consult
  a base. The `base` requirement is **dynamic, not structural**: a `skip` `gather` with
  **no** `base` operand supplied MUST be legal and MUST evaluate to a defined result so
  long as no index is actually out of bounds at evaluation — the base is consulted only
  for an index that actually skips. An **actually out-of-bounds** `skip` read for which no
  `base` operand was supplied MUST be a **typed decline** (KISS-Conform §6.7), never a
  panic and never undefined behaviour. Because the base is required only on an actual
  out-of-bounds read, the §6.13 `index_select` reference decomposition (`gather(oob=skip)`
  with a 1-D index and no base) remains **valid and unchanged**: `index_select` carries no
  base and declines only if an index is genuinely out of bounds. *Test:*
  `test_ops_gather_skip_base_dynamic`.
- **KISS-OPS-6.11-0016** — `scatter` MUST carry an explicit **`dest` operand** — the
  owned destination the combined writes are applied onto. The op's output MUST equal
  `dest` with the §6.11-0005 / §6.11-0006 combined writes applied at their scattered
  positions; an out-of-bounds-skipped write (§6.11-0005) and any output position that is
  never written MUST **retain the `dest` value** at that position. For an accumulating
  combine (`atomic-add`, and the `scatter_add` wrapper of §6.13-0009) the accumulation
  base at each written position MUST be the `dest` value there (the result is
  `dest[j] + Σ updates`), so the accumulated result is fully defined. `scatter`'s
  **output shape MUST equal `dest`'s shape** (`SameAs(dest)`), the rule carried by the
  §6.20-0008 shape-rule enumeration. *Test:* `test_ops_scatter_dest_operand`.
- **KISS-OPS-6.11-0017** — A `scatter` **`updates` (source) operand MUST broadcast to
  the write shape under the full §6.11-0001 broadcast rules** (broadcast reads expressed
  by a stride of `0` along an axis), exactly as an elementwise read broadcasts to its
  output; the mapping from an `updates` element to each written destination position
  follows that broadcast. A **rank-0 (scalar) `updates` operand is the degenerate case**
  of that general broadcast (stride `0` on every axis), **not** a special-cased
  rank-0-only rule: `scatter` MUST NOT restrict `updates` to rank-0, and MUST admit any
  `updates` shape that broadcasts to the write shape under §6.11-0001. *Test:*
  `test_ops_scatter_updates_broadcast`.
- **KISS-OPS-6.11-0018** — The empty-axis (zero-extent) behaviour of the four
  non-`reduce` structural atoms MUST be the shape-implied result of each atom's rule,
  stated here so no implementation must infer it (the companion to the §6.11-0002
  empty-`reduce` = monoid-identity pin): (a) `prefix_scan` over an **empty axis** MUST
  produce **zero output positions** along that axis (length-preserving, §6.11-0003);
  (b) `gather` with an **empty index operand** MUST produce an **empty gathered axis**
  (§6.20-0008 with the index shape empty); (c) `scatter` with an **empty index** MUST
  perform **no writes** and MUST return the `dest` operand unchanged (§6.11-0016);
  (d) `sort_network` over an **empty row** MUST produce an **empty permutation** and an
  empty original-index vector (§6.11-0007). Each is a clarification of the existing
  shape/length rule, not a new behaviour. *Test:* `test_ops_structural_empty_axis`.
- **KISS-OPS-6.11-0019** — The **index-lane output** of `sort_network` (the
  original-index vector of §6.11-0007) MUST be dtype **`i64`**, pinned **producer-side
  with no wire field**: the output dtype is producer-defined, `i64` is the only member of
  the §6.11-0009 set covering every legal extent, and §6.11-0009 already obliges every
  index-consumer to accept `i64`, so a `sort_network` index output feeds a downstream
  `gather` / `index_select` index operand with no conversion. This pin MUST NOT be encoded
  in OpAttrs or any recipe field — it removes a conformance dimension rather than adding a
  byte. The §6.11-0009 legal index-**operand** set `{u32, i32, i64}` is **unchanged and
  remains scoped to the index-operand role** (§6.11-0012); this clause pins only the sort
  index-lane **output**, and thereby closes the §6.19 wire freeze-blocker for the sort
  index-output dtype without a wire field. A device that prefers `u32` internally MUST
  convert at the boundary it already owns. *Test:* `test_ops_sort_index_output_i64`.

### 6.12 Scalar-source leaves

- **KISS-OPS-6.12-0001** — The scalar-source leaves inside an op body MUST be exactly:
  `input(i)` (the `i`-th input operand's element), `const(bits)` (a dtype-typed constant
  pinned by its bit pattern), `param(i)` (the `i`-th scalar parameter), `coord(axis)` (the
  current iteration coordinate along `axis`), `reduced(stage)` (the accumulated result of
  a named reduction/scan stage), `extent(axis)` (the runtime logical extent — length — of a
  single iteration `axis`, a non-negative integer-valued scalar source), and
  `reduced_count` (the product of the logical extents of **all** axes selected by the op's
  `reduce_axes` descriptor, §6.11-0011 — the number of elements folded into one reduced
  result, and the correct divisor for a mean over one **or more** reduced axes; it equals
  `extent(axis)` when a single axis is reduced and the product `∏ extent(axis_i)` over the
  selected set when several are). These leaves are part of the op-semantics currency and are
  not themselves ops. *Test:* `test_ops_scalar_source_leaves`.
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
| `tanh` | transcendental | ✓ | `div(sub(exp(x), exp(neg(x))), add(exp(x), exp(neg(x))))` (refinement-permitted: an overflow-safe form, e.g. via `copysign`/`expm1`, MUST be used so `tanh` of a large argument yields ±1, not the reference `∞/∞ = NaN`) |
| `sinh` | transcendental | ✓ | `div(sub(exp(x), exp(neg(x))), const(2))` (refinement-permitted: near-zero-accurate / overflow-safe form) |
| `cosh` | transcendental | ✓ | `div(add(exp(x), exp(neg(x))), const(2))` (refinement-permitted: overflow-safe form) |
| `asinh` | transcendental | — | `log(add(x, sqrt(add(sqr(x), const(1)))))` |
| `acosh` | transcendental | — | `log(add(x, sqrt(sub(sqr(x), const(1)))))` |
| `atanh` | transcendental | — | `mul(const(0.5), log(div(add(const(1),x), sub(const(1),x))))` |
| `asin` | transcendental | — | `atan(div(x, sqrt(sub(const(1), sqr(x)))))` |
| `acos` | transcendental | — | `sub(const(pi/2), asin(x))` |
| `cbrt` | transcendental | — | `mul(sign(x), exp(div(log(abs(x)), const(3))))` |
| `erfc` | transcendental | — | `sub(const(1), erf(x))` |
| `sigmoid` | activation | — | `recip(add(const(1), exp(neg(x))))` |
| `relu` | activation | — | `select(cmp_lt(x, const(0)), const(0), x)` |
| `silu` | activation | ✓ | `mul(x, sigmoid(x))` (refinement-permitted: overflow-safe evaluation of the `exp` in `sigmoid`) |
| `softplus` | activation | ✓ | `log(add(const(1), exp(x)))` (refinement-permitted: an overflow-safe form, e.g. `max(x,0)+log1p(exp(-|x|))`, MUST be used so `softplus` of a large argument yields ≈`x`, not the reference `log(∞) = ∞`) |
| `mish` | activation | ✓ | `mul(x, tanh(softplus(x)))` (refinement-permitted: inherits the overflow-safe `tanh`/`softplus`) |
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
| `reduce_mean` | reduction | — | `div(reduce(sum, x), reduced_count)` (divisor is the product of extents over **all** reduced axes, §6.12-0001) |
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
| `im2col` | shape | — | closed-form structured `gather` mapping each output element `(patch, tap)` to the source element at `coord`-derived index `base(patch)·stride + tap_offset·dilation − padding` over `window_size` taps per axis (window/stride/dilation/padding and the output-column flatten order pinned as attributes per §6.13-0004; OOB taps zero-filled) |

- **KISS-OPS-6.13-0001** — Each non-primitive op MUST carry the reference decomposition
  in the table above (and, for the complex-arithmetic family, in the §6.18 tables), and
  that decomposition MUST define the op's pinned semantics; a consumer resolving an
  unrecognized op MUST use this decomposition. *Test:*
  `test_ops_reference_decompositions`.
- **KISS-OPS-6.13-0002** — Each reference decomposition MUST reference only ops of
  strictly lower level (§6.14) together with the scalar-source leaves of §6.12; a
  decomposition MUST NOT reference a higher-level op or an equal-level op. *Test:*
  `test_ops_decomposition_strictly_lower_level`.
- **KISS-OPS-6.13-0003** — For every op marked **✓** in the *Refine* column of the §6.13
  table (`expm1`, `log1p`, `tanh`, `sinh`, `cosh`, `silu`, `softplus`, `mish`, `pow`,
  `hypot`, `ldexp`), a conforming kernel MAY — and, where the literal reference
  decomposition would overflow or catastrophically cancel while the true function is finite
  (the exp-of-large-argument forms `tanh`, `sinh`, `cosh`, `silu`, `softplus`, `mish`),
  MUST — compute a more accurate result than the literal reference decomposition (e.g. an
  overflow-safe or near-zero-accurate form, or an exact integer-exponent `ldexp`), but MUST
  agree with the reference decomposition's pinned mathematical meaning (the function it
  denotes) within the op's declared ULP; an op **not** marked MUST reproduce the reference
  under its determinism class. *Test:* `test_ops_decomposition_accuracy_refinement`.
- **KISS-OPS-6.13-0004** — A parameterized non-primitive op MUST carry its semantics-
  affecting attributes explicitly: `reduce_var` and `reduce_std` default to the
  population form and MUST declare a Bessel correction as an attribute rather than
  changing the decomposition silently; `softmax` / `log_softmax` MUST declare the
  normalization axis; `avg_pool`, `max_pool`, and `im2col` MUST declare the per-axis
  `window_size`, `stride`, `dilation`, and `padding`; `avg_pool` MUST declare
  `count_include_pad` (which selects the divisor between the full window count —
  `reduced_count` — and the valid-tap count from the pooled view); `im2col` MUST
  additionally declare its
  **output-column ordering** (the flatten order of the window-tap and channel axes into the
  column dimension) so its index mapping is fully determined; `rms_norm` / `layer_norm` /
  `matmul` MUST use the operand-ordering convention stated above the table. The pooled
  **window view** is a structured `gather`
  that adds one window axis whose taps lie at `dilation`-spaced offsets over
  `window_size` positions with the given `stride` and `padding`, OOB taps skipped.
  KISS-Ops MUST NOT let an unstated attribute change an op's pinned result. *Test:*
  `test_ops_parameterized_attributes_explicit`.
- **KISS-OPS-6.13-0005** — `pow` MUST be pinned over its full domain: for `a>0`,
  `pow(a,b)` equals the reference `exp(mul(b, log(a)))` (refinement permitted,
  §6.13-0003); `pow(0,0)` MUST yield `1`; `pow(+0.0, b)` for `b>0` MUST yield `+0.0` and
  for `b<0` MUST yield `+∞`; for the negative-zero base, `pow(-0.0, b)` MUST yield `-0.0`
  when `b` is a positive odd integer, `+0.0` when `b` is a positive even integer or a
  positive non-integer, `-∞` when `b` is a negative odd integer, and `+∞` when `b` is a
  negative even integer or a negative non-integer (the sign follows IEEE 754 `pow` on the
  signed zero); for `a<0`, `pow(a,b)` MUST yield NaN unless `b` is an exact integer, in
  which case `pow(a,b)` MUST yield `|a|^b` when `b` is even and `-(|a|^b)` when `b` is odd.
  *Test:* `test_ops_pow_full_domain`.
- **KISS-OPS-6.13-0006** — A reference-decomposition body **that is a scalar-expression
  body** MUST conform to this grammar: a body is either (a) a single expression tree over
  KISS-Ops ops and the §6.12 scalar-source leaves, or (b) a sequence of single-assignment
  let-bindings `name = expr;` followed by a final result expression, where each `expr` is
  a tree over KISS-Ops ops, the §6.12 scalar-source leaves, and previously-bound names.
  Each `name` MUST be bound exactly once (static single assignment), MUST NOT reference
  itself or a not-yet-bound name, and is scoped to the body; a mechanical resolver MUST
  parse a scalar-expression body as this tree/binding form and no other. This clause does
  **not** constrain the body of a **structured op** (§6.13-0009), whose decomposition is a
  §6.11 structural-op reference rather than a scalar-expression tree and MUST NOT be
  required to parse as this grammar. *Test:* `test_ops_decomposition_body_grammar`.
- **KISS-OPS-6.13-0009** — A **structured op** — `matmul`, `argmax`, `argmin`,
  `avg_pool`, `max_pool`, `index_select`, `embedding`, `scatter_add`, and `im2col` — MUST
  have its reference decomposition expressed as a named §6.11 structural op (`reduce`,
  `prefix_scan`, `gather`, `scatter`, `sort_network`) or a `matmul` contraction,
  parameterized by the op's §6.13-0004 attribute record together with the operand-role and
  iteration-space facts named in the §6.13 table (e.g. a `matmul`'s M/N/K iteration space,
  a pool's window view, a gather/scatter's axis and index operand). Its pinned semantics
  and its inf/NaN/OOB edges MUST be those of the referenced §6.11 structural op under those
  attributes, evaluated by the §6.11 structural oracle; KISS-Ops MUST NOT require a
  structured op's body to be a §6.13-0006 scalar-expression tree. *Test:*
  `test_ops_structured_decomposition_reference`.
- **KISS-OPS-6.13-0007** — `hypot(a, b)` MUST yield `+∞` whenever either operand is
  infinite, **even if the other operand is NaN** (the IEEE 754 `hypot` infinity rule),
  overriding the naive `sqrt(add(sqr(a), sqr(b)))` — which would yield NaN for `(±∞, NaN)` —
  on any infinite input; for finite inputs a NaN operand MUST propagate a NaN. This pins the
  inf/NaN edge that `cabs` (§6.18-0007) relies on, so standalone `hypot` and `cabs` agree.
  *Test:* `test_ops_hypot_inf_nan`.
- **KISS-OPS-6.13-0008** — `rms_norm` and `layer_norm` MUST each carry an explicit
  **normalization-axis** attribute `norm_axis` — the axis over which the mean-square
  (`rms_norm`) or the mean and variance (`layer_norm`) is taken — of the same role, width,
  and legal range as the `softmax` / `log_softmax` normalization axis required by
  §6.13-0004: a non-negative integer in the range `0 <= norm_axis < MAX_RANK` (§6.19-0037),
  carried as the `u8` OpAttrs `norm_axis` field of §6.19-0031. The §6.13-0004 requirement
  that a parameterized op declare its normalization axis explicitly therefore covers all
  four of `softmax`, `log_softmax`, `rms_norm`, and `layer_norm`; an implementation MUST
  reject a `norm_axis` outside the range with a typed decline and MUST NOT let an unstated
  normalization axis change the op's pinned result. *Test:* `test_ops_norm_axis_all_four`.

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
- **KISS-OPS-6.15-0004** — `gelu` (exact erf-based, decomposing through `erf`) and
  `gelu_tanh` (the tanh approximation, decomposing through `tanh`) MUST be distinct ops
  with their distinct §6.13 decompositions; an implementation MUST NOT treat one as the
  other or fold them into a single op. (The two functions differ numerically — see the
  informative §2.3 — but the load-bearing requirement pinned here is op distinctness.)
  *Test:* `test_ops_gelu_exact_vs_tanh`.

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
| `f32` | 32 | float | IEEE-754 binary32 storage; **pure storage** — compute fidelity is the MathPrecision attribute (§6.17), not a dtype property |
| `f64` | 64 | float | IEEE-754 binary64 (1 sign, 11 exp, 52 mantissa), bias 1023 |
| `s8` | 8 | int | signed 8-bit two's-complement |
| `s16` | 16 | int | signed 16-bit two's-complement |
| `u8` | 8 | uint | unsigned 8-bit; also the physical storage of `bool` |
| `u16` | 16 | uint | unsigned 16-bit |
| `i32` | 32 | int | signed 32-bit two's-complement |
| `i64` | 64 | int | signed 64-bit two's-complement |
| `u32` | 32 | uint | unsigned 32-bit two's-complement; **ordinary** storage/compute dtype (§6.2-0007); MAY serve the index-operand role (§6.11-0012) |
| `u64` | 64 | uint | unsigned 64-bit |
| `bool` | 8 | bool | 1 byte; `0`=false, any non-zero byte=true; ops normalize to strictly `0`/`1` |
| `e4m3fn` | 8 | float | FP8 E4M3 (1 sign, 4 exp, 3 mantissa), bias 7; max finite ±448; **no infinities**; a single NaN encoding; conversion saturates to max-finite, round-half-to-even (OCP OFP8) |
| `e4m3fnuz` | 8 | float | FP8 E4M3 AMD `fnuz` variant (1 sign, 4 exp, 3 mantissa), bias 8; no −0; **no infinities**; byte-incompatible with `e4m3fn`; **reserved** (recognized on parse; no op assigns it computation semantics at this schema version) |
| `e5m2` | 8 | float | FP8 E5M2 (1 sign, 5 exp, 2 mantissa), bias 15; max finite ±57344; IEEE-style inf/NaN; conversion saturates to max-finite, round-half-to-even (OCP OFP8) |
| `e5m2fnuz` | 8 | float | FP8 E5M2 AMD `fnuz` variant (1 sign, 5 exp, 2 mantissa), bias 16; no −0; **no infinities**; byte-incompatible with `e5m2`; **reserved** (recognized on parse; no op assigns it computation semantics at this schema version) |
| `s4` | 4 | int | signed 4-bit, range [−8,+7]; packed pair per byte, low nibble = even logical index, sign-extended on read (**storage packing owned normatively by the data-vocabulary sub-standard §6.1-0008/0009**; restated here informatively) |
| `u4` | 4 | uint | unsigned 4-bit, range [0,15]; packed pair per byte, low nibble = even index, zero-extended on read (**storage packing owned normatively by the data-vocabulary sub-standard §6.1-0008/0009**; restated here informatively) |
| `b1` | 1 | uint | 1-bit binary-GEMM operand; storage packing (8 bits/byte, LSB = lowest logical index) **owned normatively by the data-vocabulary sub-standard §6.1-0008/0009** and restated here informatively; the **xor+popcount accumulation to raw `s32` output** is the Ops-owned computation semantics |
| `c32` | 64 | complex | interleaved (re,im) pair of `f32`; storage container, complex arithmetic via the §6.18 op family |
| `c64` | 128 | complex | interleaved (re,im) pair of `f64`; storage container, complex arithmetic via the §6.18 op family |

- **KISS-OPS-6.16-0001** — The dtype set to which KISS-Ops assigns numeric or structural
  meaning MUST be exactly the tokens in the table above, each with the bit width and
  layout pinned there; these layouts are inlined normatively and create no dependency
  edge on the data-vocabulary sub-standard. *Test:* `test_ops_dtype_bit_layouts`.
- **KISS-OPS-6.16-0002** — `f16`, `f32`, and `f64` MUST use their IEEE 754-2019 encodings
  (binary16 / binary32 / binary64), including the standard inf/NaN encodings, signed zero,
  and subnormals; the storage dtype set carries **no** compute-precision distinction (there
  is no strict-precision float dtype) — compute fidelity is the MathPrecision attribute
  (§6.17). *Test:* `test_ops_ieee754_dtype_encodings`.
- **KISS-OPS-6.16-0003** — `bf16` MUST be encoded as a 1-sign / 8-exp / 7-mantissa format
  (bias 127) with the binary32 exponent range and a truncated mantissa; it is **not** an
  IEEE 754-2019 format, and any op producing a `bf16` result MUST round to nearest, ties
  to even. *Test:* `test_ops_bf16_layout`.
- **KISS-OPS-6.16-0004** — `e4m3fn` MUST be encoded per OCP OFP8 as 1-sign / 4-exp /
  3-mantissa (bias 7) with maximum finite magnitude ±448, **no** infinity encoding, and a
  single NaN encoding; conversion into `e4m3fn` MUST saturate to the maximum finite
  magnitude under round-half-to-even. *Test:* `test_ops_e4m3_layout`.
- **KISS-OPS-6.16-0005** — `e5m2` MUST be encoded per OCP OFP8 as 1-sign / 5-exp /
  2-mantissa (bias 15) with maximum finite magnitude ±57344 and IEEE-style infinity and
  NaN encodings; conversion into `e5m2` MUST saturate to the maximum finite magnitude
  under round-half-to-even. *Test:* `test_ops_e5m2_layout`.
- **KISS-OPS-6.16-0006** — The integer dtypes MUST use the pinned layouts:
  `s8`/`s16`/`i32`/`i64` two's-complement; `u8`/`u16`/`u32`/`u64` unsigned (`u32` an ordinary unsigned dtype,
  §6.2-0007; index-only-ness is an operand role, §6.11-0012); `bool` one byte normalized to
  `0`/`1`. The sub-byte **storage packing**
  conventions — `s4`/`u4` packed two-per-byte with the low nibble at the even logical
  index (sign-extended for `s4`, zero-extended for `u4`), and `b1` packed 8-bits-per-byte
  LSB-first — are **restated here informatively and owned normatively by the sibling
  data-vocabulary sub-standard (its §6.1-0008/0009)**, which is the single normative home
  for sub-byte packing; KISS-Ops restates them only so this document is self-contained and
  MUST NOT be read as a second normative pinning of the packing facts. The Ops-owned
  normative fact in this clause is the `b1` **xor+popcount accumulation to a raw `s32`**
  binary-GEMM computation semantics (a computation fact, not storage packing). *Test:*
  `test_ops_integer_dtype_layouts`.
- **KISS-OPS-6.16-0007** — The interleaved (re,im) **storage layout** of `c32`/`c64` —
  pairs of `f32` and `f64` respectively (64 and 128 bits, real lane at offset 0 / the
  lower-addressed half, imaginary lane at offset 1) — is **restated here informatively and
  owned normatively by the sibling data-vocabulary sub-standard (Classify §6.1-0012)**, which
  is the single normative home for complex storage; KISS-Ops restates it only so this
  document is self-contained and MUST NOT be read as a second normative pinning of the
  storage facts (mirroring the §6.16-0006 sub-byte-packing pattern). The Ops-owned normative
  content of this clause is that complex arithmetic on `c32`/`c64` MUST be performed by the
  complex-arithmetic op family of §6.18, every member of which is non-primitive and
  decomposes into the real primitive floor (no new primitive is introduced, §6.18-0002).
  *Test:* `test_ops_complex_storage_layout`.
- **KISS-OPS-6.16-0008** — The dtype bit layouts inlined in §6.16 are a shared layout
  convention spelled per foundational vocabulary (the shared anchor), not an import, and
  create **no dependency edge** (§2.8, §4). Any non-additive change to a dtype's pinned layout
  facts — storage width, float sign/exponent/mantissa split and exponent bias, integer
  signedness, complex `(re,im)` interleave, or sub-byte packing — MUST co-bump **both** the
  KISS-Ops op-vocabulary schema version **and** the Classify `DTYPE_LAYOUT_VERSION`
  (KISS-CLASSIFY-8-0007), never a silent in-place change to one table; the Ops §6.16 and
  Classify §6.1 normative tables MUST agree on every layout fact they both state. For sub-byte
  packing (`s4`/`u4`/`b1`, §6.16-0006) and complex storage (`c32`/`c64`, §6.16-0007) whose
  normative owner is Classify, this obligation is that the Ops restatement tracks the Classify
  owner, not a symmetric dual pinning. *Test:* `test_ops_dtype_layout_coversioned`.

### 6.17 Compute-fidelity (math-precision) attribute

KISS-Ops owns a second per-kernel attribute, **orthogonal** to the §6.0 determinism class:
the **compute-fidelity (MathPrecision) attribute**, a two-member enum
`{bit-stable, reduced-mantissa-permitted}`. It records whether a target computes an op's
floating-point arithmetic at full storage precision with each atom rounding independently
(**bit-stable**) or MAY use a reduced-mantissa fast path (**reduced-mantissa-permitted**,
e.g. a TF32-style multiply). This attribute is the home of the compute-precision meaning
formerly (mis)modeled as a strict-precision float dtype: storage dtypes (§6.16) are pure
byte layout and carry no compute-precision meaning; compute precision is this attribute.
A determinism class selects a comparator; the MathPrecision attribute constrains per-atom
mantissa width. The attribute is defined once here and imported by KISS-Contract, which
surfaces it in a kernel's guarantees section.

- **KISS-OPS-6.17-0001** — KISS-Ops MUST own a single compute-fidelity (MathPrecision)
  attribute drawn from the two-member enum `{bit-stable, reduced-mantissa-permitted}`,
  defined once here and imported (not re-forked) by KISS-Contract as a kernel guarantee;
  this attribute MUST be distinct from and orthogonal to the §6.0 determinism/fidelity
  class. *Test:* `test_ops_math_precision_enum`.
- **KISS-OPS-6.17-0002** — Under the **bit-stable** value, every floating-point arithmetic
  atom in an op's evaluation MUST round independently at the storage dtype's full
  IEEE 754-2019 precision, and an implementation MUST NOT use a reduced-mantissa multiply,
  a fused-multiply-add, or any other contraction that removes an intermediate rounding
  (the same per-atom-rounding requirement as §6.0-0006). Bit-stable is the guarantee
  formerly modeled as the strict-precision `f32s` dtype and MUST NOT be re-encoded as a
  dtype. *Test:* `test_ops_math_precision_bit_stable`.
- **KISS-OPS-6.17-0003** — Under the **reduced-mantissa-permitted** value, a target MAY
  compute a floating-point multiply with its inputs rounded to a **reduced mantissa**
  (a mantissa narrower than the storage dtype), provided the reduced mantissa retains **no
  fewer than 10 explicit mantissa bits** (the pinned floor, so "reduced-mantissa" is a
  quantified bound and not an open-ended adjective — a target MUST NOT round inputs below
  this floor) and the result stays within the op's declared determinism class (§6.0); a
  kernel's contract MUST advertise the MathPrecision value so a consumer knows whether
  bit-stable reproduction is guaranteed. *Test:* `test_ops_math_precision_reduced`.
- **KISS-OPS-6.17-0004** — The MathPrecision attribute MUST be carried as a kernel-level
  (contract-guarantee) attribute and MAY be **refined per op** where a kernel guarantees a
  finer value for some ops than kernel-wide (recovering the per-operand granularity
  formerly modeled by a strict-precision dtype); it MUST NOT be encoded into the storage
  dtype set of §6.16, and a storage dtype MUST NOT carry a compute-precision distinction
  (there is no strict-precision float dtype in this version). *Test:*
  `test_ops_math_precision_not_dtype`.
- **KISS-OPS-6.17-0005** — MathPrecision MUST be verified as follows, so it is testable for
  every determinism class: for an **exact-byte** or **ULP/tolerance** op, **bit-stable** is
  verified directly under the op's own comparator (each atom at full storage-precision
  rounding), and **reduced-mantissa-permitted** is verified by detecting a per-atom multiply
  whose input mantissa is narrower than the §6.17-0003 floor. For an
  **order-invariant/nondeterministic** op (a float reduction, scan, or contraction),
  bit-stable MUST NOT be read as pinning the contraction order (which §6.0-0004 leaves
  unpinned); it constrains **each individual multiply and add atom** to full
  storage-precision rounding, verified under a **pinned bit-stable reference profile**
  (ascending-index reduction order with the accumulator at the storage dtype's precision),
  and reduced-mantissa-permitted is verified by the same per-atom mantissa-width probe. The
  attribute therefore constrains per-atom mantissa width independently of reduction order.
  *Test:* `test_ops_math_precision_order_invariant_scope`.
- **KISS-OPS-6.17-0006** — For each MathPrecision value, §6.17 MUST pin the exact
  input-rounding applied to each operand before compute, as
  `(retained_mantissa_bits, rounding_mode)`, precise enough for a spec-derived reference
  to (a) derive the accuracy bound (`u = 2^−(retained_mantissa_bits + 1)`) and (b)
  reproduce the rounded operand bit-for-bit. **bit-stable** applies no input rounding
  (each operand at its storage-dtype mantissa). **reduced-mantissa-permitted** rounds each
  operand's mantissa to a pinned width via the named mode before compute; the canonical
  value is **TF32** (`f32` primary, `cuda:sm80+`): 10 retained mantissa bits, exponent
  unchanged, round-to-nearest-even, `u = 2⁻¹¹`. The rounding MUST be a true round (RNE),
  not truncation, and MUST carry into the exponent (`1.11…1` → `10.0…0`, exponent + 1); a
  reference that truncates is non-conformant. Where a target admits more than one
  reduced-mantissa width for the same primary dtype and capability, each MUST be named
  separately (an additive refinement); the §6.17-0003 floor (no fewer than 10 mantissa
  bits) still governs. This pin is what the KISS-Classify `<mp>` key coordinate
  (KISS-CLASSIFY §6.7-0006, `st`/`rm`) resolves to per `(primary_dtype, target)`. *Test:*
  `test_ops_math_precision_input_rounding`.
- **KISS-OPS-6.17-0007** — Input-rounding (§6.17-0006) and accumulator width do NOT by
  themselves fix a float contraction's bits: float accumulation is non-associative, so the
  reduction order/schedule also moves bits. A contraction cell is bit-reproducible (admits
  a bit-pattern golden) ONLY if input-rounding is pinned per §6.17-0006 AND the accumulation
  schedule (order + per-step accumulator rounding) is deterministic and specified to the
  reference; otherwise the cell is `order-invariant/nondeterministic` (§6.0-0001) and MUST
  be compared under a declared tolerance, never a bit golden. A `reduced-mantissa-permitted`
  value shifts a tolerance cell's tolerance *magnitude* but MUST NOT flip a tolerance cell
  to bit-golden. A tolerance cell's declared tolerance MUST bound the combined
  input-rounding-plus-reduction-order error against the wide-precision truth — the
  KISS-Conform §6.5-0007 oracle evaluation. *Test:*
  `test_ops_math_precision_reproducibility_class`.

### 6.18 Complex-arithmetic op family (c32 / c64)

This version defines a **complex-arithmetic op family** over the interleaved-storage
complex dtypes `c32` (a (re,im) pair of `f32`) and `c64` (a (re,im) pair of `f64`). Every
op in the family is **non-primitive**: it carries a reference decomposition into the
existing **real** primitive-floor atoms — `add`, `sub`, `mul`, `div`, `neg`, `sqrt`,
`exp`, `log`, `sin`, `cos`, `atan2`, `copysign` — plus the structural atom `element_map`
(via the `cmake` / `cre` / `cim` component bridge), the lower-level real non-primitive
`hypot`, and lower-level complex ops of this same family (`clog` / `csqrt` use `cabs` /
`carg`; `cpow` uses `cexp` / `cmul` / `clog`). The family introduces **no** new
primitive-floor op: the §6.3 primitive floor is unchanged (the closure is pinned normatively
in §6.18-0002). Complex semantics are pinned to **ISO/IEC 9899 (C99/C11) Annex G**
(IEC 60559-compatible complex arithmetic): principal branch cuts for `clog` / `csqrt` /
`cpow`, signed zero of the real and imaginary components, and the Annex-G infinity/NaN
propagation and recovery rules for `cmul` / `cdiv` / `cabs` / `cexp`.

For a complex operand `z`, `cre(z)` and `cim(z)` denote its real and imaginary `f32`/`f64`
lanes and `cmake(x, y)` constructs a complex value with real lane `x` and imaginary lane
`y`. A complex op on `c32` evaluates its real-atom decomposition in `f32`; on `c64`, in
`f64` (§6.18-0015).

**Component bridge (plumbing; not advertised high-level):**

| Op | Result | Reference decomposition |
|---|---|---|
| `cmake` | complex from (re,im) | `element_map` writing lane 0 = `input(0)`, lane 1 = `input(1)` into the interleaved (re,im) storage of §6.16-0007 |
| `cre` | real lane → real | `element_map` reading lane 0 (stride-2 view) of the interleaved storage |
| `cim` | imag lane → real | `element_map` reading lane 1 (stride-2 view) of the interleaved storage |

**Complex arithmetic (advertised high-level):** with `a=cre(z)`, `b=cim(z)`, `c=cre(w)`,
`d=cim(w)`:

| Op | Family | Determinism | Reference decomposition (principal / finite case) |
|---|---|---|---|
| `cadd` | complex | exact-byte | `cmake(add(a,c), add(b,d))` |
| `csub` | complex | exact-byte | `cmake(sub(a,c), sub(b,d))` |
| `cneg` | complex | exact-byte | `cmake(neg(a), neg(b))` |
| `cconj` | complex | exact-byte | `cmake(a, neg(b))` |
| `cmul` | complex | exact-byte | `cmake(sub(mul(a,c), mul(b,d)), add(mul(a,d), mul(b,c)))` + Annex-G recovery (§6.18-0005) |
| `cdiv` | complex | exact-byte | `t=add(mul(c,c),mul(d,d)); cmake(div(add(mul(a,c),mul(b,d)),t), div(sub(mul(b,c),mul(a,d)),t))` + Annex-G recovery (§6.18-0006) |
| `cabs` | complex | ULP/tolerance | `hypot(a, b)` → real |
| `carg` | complex | ULP/tolerance | `atan2(b, a)` → real |
| `cexp` | complex | ULP/tolerance | `ea=exp(a); cmake(mul(ea,cos(b)), mul(ea,sin(b)))` |
| `clog` | complex | ULP/tolerance | `cmake(log(cabs(z)), carg(z))` (principal; imag ∈ [−π, +π], both endpoints reachable via signed zero) |
| `csqrt` | complex | ULP/tolerance | `m=cabs(z); cmake(sqrt(div(add(m,a),const(2))), mul(copysign(const(1),b), sqrt(div(sub(m,a),const(2)))))` (principal) |
| `cpow` | complex | ULP/tolerance | `cexp(cmul(w, clog(z)))` (principal) |

- **KISS-OPS-6.18-0001** — The complex-arithmetic op family for this version MUST be
  exactly `{cmake, cre, cim, cadd, csub, cneg, cconj, cmul, cdiv, cabs, carg, cexp, clog,
  csqrt, cpow}`, each with the signature pinned in the §6.18 tables (`cre`/`cim` yield the
  real component dtype; `cmake` consumes two real lanes; `cabs`/`carg` yield a real value;
  the remainder are complex→complex); an implementation MUST NOT treat a token outside this
  set as a complex op of this version. *Test:* `test_ops_complex_op_set`.
- **KISS-OPS-6.18-0002** — Every complex op MUST be **non-primitive**, carrying the
  reference decomposition in the §6.18 tables, and MUST reference only ops of strictly lower
  level (§6.14) drawn from the closure of: the existing real primitive-floor atoms —
  including `neg` — (`add`, `sub`, `mul`, `div`, `neg`, `sqrt`, `exp`, `log`, `sin`, `cos`,
  `atan2`, `copysign`), the structural atom `element_map`, the real non-primitive `hypot`,
  the component-bridge ops `cmake` / `cre` / `cim`, lower-level complex-arithmetic ops of
  this same family (e.g. `clog` and `csqrt` reference `cabs` / `carg`; `cpow` references
  `cexp` / `cmul` / `clog`), and the §6.12 scalar-source leaves (`const`, …). The complex
  family MUST introduce **no** new primitive-floor op, and the §6.3 primitive floor MUST
  remain exactly as pinned there (unchanged by this family). *Test:*
  `test_ops_complex_no_new_primitive`.
- **KISS-OPS-6.18-0003** — `cmake`, `cre`, and `cim` MUST bridge between a complex operand
  and its two real lanes exactly per the §6.16-0007 interleaved (re,im) storage:
  `cmake(x,y)` places `x` in the real lane and `y` in the imaginary lane; `cre(z)` reads
  the real lane (offset 0); `cim(z)` reads the imaginary lane (offset 1); each as an
  `element_map` over the interleaved storage. These bridge ops MUST preserve the signed
  zero and NaN payload of each lane (raw-lane move, no arithmetic). *Test:*
  `test_ops_complex_component_bridge`.
- **KISS-OPS-6.18-0004** — `cadd`, `csub`, `cneg`, and `cconj` MUST equal the componentwise
  decompositions of the §6.18 table (`cadd`/`csub` real `add`/`sub` per lane; `cneg`
  negating both lanes; `cconj` negating only the imaginary lane) and MUST therefore
  preserve IEEE signed zero per lane, with `cconj` flipping the sign bit of the imaginary
  lane (`+0.0 → −0.0`). *Test:* `test_ops_complex_add_sub_neg_conj`.
- **KISS-OPS-6.18-0005** — `cmul` MUST equal `(ac − bd) + (ad + bc)i` computed by the real
  decomposition of the §6.18 table for finite operands (each real atom rounding
  independently, exact-byte, §6.0-0006), and MUST additionally apply the ISO C99/C11
  **Annex G** infinity-recovery rule with the **Annex-G trigger, verbatim**: recovery
  applies if and only if **both** result components are NaN **and** at least one operand
  has an infinite lane (a lane is ±∞); in that case, and only that case, the result MUST be
  a complex infinity with the Annex-G-determined component signs (an infinity times a
  nonzero finite or infinite operand is an infinity, not a NaN). When only one result
  component is NaN, recovery MUST NOT be applied (Annex G leaves that result as computed).
  *Test:* `test_ops_cmul_annexg`.
- **KISS-OPS-6.18-0006** — `cdiv` MUST equal `((ac + bd) + (bc − ad)i) / (c² + d²)`
  computed by the real decomposition of the §6.18 table for finite operands with nonzero
  denominator, and MUST additionally apply the ISO C99/C11 **Annex G** rules: a
  complex-infinity numerator over a finite nonzero denominator MUST yield a complex
  infinity; a complex-infinity denominator MUST yield a complex zero (finite numerator);
  and division of a nonzero finite numerator by a complex **zero** denominator MUST yield a
  complex infinity per Annex G, never a trap. Because `cdiv` is class **exact-byte**
  (§6.18-0014) yet real `div` by zero is target-defined (target-UB, §6.4-0002), `cdiv` MUST
  NOT evaluate the zero-denominator or infinite-operand cases through the real `div` atom;
  these cases MUST instead be produced by the Annex-G recovery with the component signs
  pinned by Annex G (a nonzero finite `(a,b)` over `(±0, ±0)` yields the complex infinity
  `copysign(∞, a) + i·copysign(∞, b)` per Annex G G.5.1), so the exact-byte guarantee is
  achievable independently of the target's `div`/0 behavior. *Test:* `test_ops_cdiv_annexg`.
- **KISS-OPS-6.18-0007** — `cabs(z)` MUST equal `hypot(cre(z), cim(z))` (a real,
  non-negative value in the component dtype), which by the pinned real `hypot` inf/NaN rule
  (§6.13-0007) is `+∞` whenever either lane is infinite **even if the other lane is NaN**
  (an infinite component dominates a NaN component), matching Annex G; standalone `hypot`
  and `cabs` therefore agree on this edge. *Test:* `test_ops_cabs_annexg`.
- **KISS-OPS-6.18-0008** — `carg(z)` MUST equal `atan2(cim(z), cre(z))`, yielding the
  **principal** argument in `[−π, +π]` under the IEEE ±0 quadrant conventions of `atan2`
  (§6.9-0001), so the signed zero of each lane selects the correct quadrant. *Test:*
  `test_ops_carg_principal`.
- **KISS-OPS-6.18-0009** — `cexp(z)` MUST equal `exp(cre(z)) · (cos(cim(z)) +
  i·sin(cim(z)))` computed by the §6.18 decomposition **for finite arguments only**. For
  any argument with an infinite or NaN lane, the ISO C99/C11 **Annex G** `cexp`
  special-value rules **GOVERN over the naive decomposition** (which would otherwise yield
  a spurious `∞·0 = NaN`), and are the pinned semantics. The complete governing rows are:
  `cexp(±0 + i·0) = 1 + i·0`; `cexp(x + i·∞) = NaN + i·NaN` (finite `x`, invalid);
  `cexp(x + i·NaN) = NaN + i·NaN` (finite `x`); `cexp(+∞ + i·0) = +∞ + i·0`;
  `cexp(−∞ + i·y) = +0 · (cos y + i·sin y)` (finite `y`, signed zeros from `cos y`/`sin y`);
  `cexp(+∞ + i·y) = +∞ · (cos y + i·sin y)` (finite nonzero `y`);
  `cexp(−∞ + i·∞) = ±0 ± i·0` (signs unspecified); `cexp(+∞ + i·∞) = ±∞ + i·NaN` (invalid);
  `cexp(−∞ + i·NaN) = ±0 ± i·0`; `cexp(+∞ + i·NaN) = ±∞ + i·NaN`;
  `cexp(NaN + i·0) = NaN + i·0`; `cexp(NaN + i·y) = NaN + i·NaN` (nonzero `y`);
  `cexp(NaN + i·NaN) = NaN + i·NaN`. *Test:* `test_ops_cexp_annexg`.
- **KISS-OPS-6.18-0010** — `clog(z)` MUST equal `cmake(log(cabs(z)), carg(z))`, the
  **principal** complex logarithm whose branch cut lies along the negative real axis and is
  continuous from above (with the second quadrant); the imaginary part MUST lie in the
  **closed** interval `[−π, +π]` (both endpoints reachable, consistent with `carg`'s closed
  range §6.18-0008 and C99 Annex G) and MUST honor the signed zero of the imaginary lane so
  that `clog(−1 + i·0)` yields `0 + iπ` and `clog(−1 − i·0)` yields `0 − iπ` (the sign of the
  zero imaginary part selects the ±π branch endpoint, both of which are attainable). For any
  argument with an infinite or NaN lane the ISO C99/C11 **Annex G** `clog` special-value
  rules **GOVERN over the naive decomposition** and are the pinned semantics; the complete
  governing rows are: `clog(−0 + i·0) = −∞ + i·π`; `clog(+0 + i·0) = −∞ + i·0`;
  `clog(x + i·∞) = +∞ + i·π/2` (finite `x`); `clog(x + i·NaN) = NaN + i·NaN` (finite `x`);
  `clog(−∞ + i·y) = +∞ + i·π` (finite `y`); `clog(+∞ + i·y) = +∞ + i·0` (finite `y`);
  `clog(−∞ + i·∞) = +∞ + i·3π/4`; `clog(+∞ + i·∞) = +∞ + i·π/4`;
  `clog(±∞ + i·NaN) = +∞ + i·NaN`; `clog(NaN + i·∞) = +∞ + i·NaN`;
  `clog(NaN + i·y) = NaN + i·NaN` (finite `y`); `clog(NaN + i·NaN) = NaN + i·NaN`. *Test:*
  `test_ops_clog_principal_branch`.
- **KISS-OPS-6.18-0011** — `csqrt(z)` MUST equal the **principal** square root of the
  §6.18 decomposition (non-negative real part; branch cut on the negative real axis) **for
  finite arguments**, with the imaginary part's sign taken from the imaginary lane via
  `copysign` so the signed zero of the imaginary lane is honored: `csqrt(−4 + i·0)` MUST
  yield `0 + 2i` and `csqrt(−4 − i·0)` MUST yield `0 − 2i`. For any argument with an
  infinite or NaN lane the ISO C99/C11 **Annex G** `csqrt` special-value rules **GOVERN over
  the naive decomposition** (which would otherwise yield a spurious NaN) and are the pinned
  semantics; the complete governing rows are: `csqrt(±0 + i·0) = +0 + i·0`;
  `csqrt(x + i·∞) = +∞ + i·∞` (any `x`, including ±∞/NaN); `csqrt(x + i·NaN) = NaN + i·NaN`
  (finite `x`); `csqrt(−∞ + i·y) = +0 + i·∞` (finite positive `y`);
  `csqrt(+∞ + i·y) = +∞ + i·0` (finite positive `y`); `csqrt(−∞ + i·NaN) = NaN ± i·∞`;
  `csqrt(+∞ + i·NaN) = +∞ + i·NaN`; `csqrt(NaN + i·y) = NaN + i·NaN` (finite `y`);
  `csqrt(NaN + i·NaN) = NaN + i·NaN`. *Test:* `test_ops_csqrt_principal_branch`.
- **KISS-OPS-6.18-0012** — `cpow(z, w)` MUST equal the **principal** value
  `cexp(cmul(w, clog(z)))` of the §6.18 decomposition (using the principal `clog`).
  `cpow(z, 0)` MUST yield `1 + 0i` for every `z`. Because Annex G specifies `cpow` through
  this same `cexp`/`clog`/`cmul` composition, `cpow` on non-finite or zero inputs MUST be
  evaluated by applying the **governed** Annex-G special values of `clog` (§6.18-0010),
  `cmul` (§6.18-0005), and `cexp` (§6.18-0009) in that composition — so its result is fully
  determined, not left to the naive chain. In particular `cpow(0 + i·0, w)` MUST yield
  `0 + i·0` when `Re(w) > 0`, a complex infinity when `Re(w) < 0`, and NaN components when
  `Re(w) = 0` with `w ≠ 0`. *Test:* `test_ops_cpow_principal`.
- **KISS-OPS-6.18-0013** — For every complex op, on any input with an infinite or NaN lane
  the **ISO C99/C11 Annex G special-value / recovery rules GOVERN over the naive real
  decomposition**, and the governed value is the single pinned semantics (there is no case
  in which the decomposition and Annex G disagree without a tie-break): for `cmul` / `cdiv`
  the recovery of §6.18-0005 / §6.18-0006 governs; for `cexp` / `clog` / `csqrt` / `cpow`
  the enumerated Annex-G special-value rows of §6.18-0009 / §6.18-0010 / §6.18-0011 /
  §6.18-0012 govern; for `cabs` / `carg` the §6.13-0007 / §6.9-0001 inf/NaN rules govern.
  A complex op MUST NOT return a value that the naive decomposition would produce where the
  governing Annex-G rule pins a different value. *Test:* `test_ops_complex_nan_inf_annexg`.
- **KISS-OPS-6.18-0014** — The determinism/fidelity class (§6.0) of each complex op MUST
  follow from its decomposition: the purely-algebraic complex ops (`cmake`, `cre`, `cim`,
  `cadd`, `csub`, `cneg`, `cconj`, `cmul`, `cdiv`) MUST be class **exact-byte** (no
  transcendental atom, no float reduction), and the complex ops containing a transcendental
  real atom (`cabs`, `carg`, `cexp`, `clog`, `csqrt`, `cpow` — via `sqrt` / `hypot`,
  `atan2`, `exp`, `log`, `sin`, `cos`) MUST be class **ULP/tolerance**. *Test:*
  `test_ops_complex_determinism_class`.
- **KISS-OPS-6.18-0015** — A complex op on `c32` MUST evaluate its real-atom decomposition
  with `f32` component lanes, and on `c64` with `f64` component lanes; the component dtype
  MUST be the `f32`/`f64` element of the interleaved (re,im) storage (§6.16-0007) and MUST
  NOT be promoted or demoted across the family's atoms. *Test:*
  `test_ops_complex_component_dtype`.
- **KISS-OPS-6.18-0016** — The complex ops advertised as **high-level** (native-matchable)
  MUST be `{cadd, csub, cneg, cconj, cmul, cdiv, cabs, carg, cexp, clog, csqrt, cpow}`; the
  component bridge ops `{cmake, cre, cim}` MUST NOT be advertised high-level (they are
  decomposition plumbing). A consumer that natively implements an advertised complex op MAY
  match it directly; a consumer that does not MUST resolve it via its reference
  decomposition to the real floor. *Test:* `test_ops_complex_advertised_high_level`.
- **KISS-OPS-6.18-0017** — Several exact-signed-zero and branch-endpoint requirements
  (`carg` §6.18-0008, `clog` §6.18-0010, `csqrt` §6.18-0011, and `cexp` §6.18-0009) are
  pinned on ops whose determinism class is **ULP/tolerance** (§6.18-0014), under which a
  tolerance comparator cannot distinguish `+0.0` from `−0.0` nor detect a wrong ±π branch
  endpoint where the two values coincide in magnitude. So these requirements are testable,
  KISS-Conform MUST evaluate `carg` / `clog` / `csqrt` / `cexp` under a **split comparator**:
  an **exact-bit** comparator on (a) the sign bit of every zero-valued result component and
  (b) the ±π branch-endpoint selection (the sign of an imaginary part equal to π in
  magnitude), **combined with** the ULP/tolerance comparator on the component magnitudes. An
  implementation that returns the wrong sign of a zero component or the wrong ±π endpoint
  MUST be judged non-conforming even when the magnitude is within tolerance. *Test:*
  `test_ops_complex_branch_sign_exact`.

### 6.19 The OpAttrs channel and its canonical wire encoding

**OpAttrs** is the per-op, compile-time attribute record that is part of an op's
**semantics**: the small set of fixed-at-build-time choices (a reduce's monoid and
axis set, a gather's out-of-bounds policy, a pool's window geometry, and the rest)
that select *which* member of an op's parameterized family a node denotes. It is
distinct from KISS-Grammar's `pattern_attrs` (which are **matching hints** on the
advertisable-op surface): OpAttrs changes what an op *computes*, `pattern_attrs`
only guides recognition. KISS-Ops is the **single normative owner** of the OpAttrs
channel — its schema (which op carries which fields), its sub-vocabularies (the
frozen enums/encodings each field draws from), and its **canonical, default-resolved
wire encoding**. KISS-Grammar and KISS-Contract embed the encoded OpAttrs blob as
**opaque** bytes and byte-compare it; they never parse inside it and re-define none
of its sub-vocabularies. This section discharges the seam obligations that
KISS-Grammar §6.2-0006 (refuses to invent a sub-vocabulary KISS-Ops has not pinned)
and §8-0008 (gates OpAttrs golden vectors on this upstream freeze) defer upward, and
the KISS-Contract §6.4-0003 / §6.2-0003 Semantics-node OpAttrs-channel citation.

The design follows the ratified canonical-encoding decisions (explicit
default-resolution, per-op fixed field order as ABI, frozen little-endian enum
ordinals, fixed-width little-endian two's-complement integers, explicit optional
slots, definite length-prefixes, and version binding); the load-bearing property is
that a **defaulted** attribute and an **explicitly-stated equal** attribute produce
**identical bytes**, so byte-identity is decoupled from the (versioned) default
table — the failure mode of the prior-art elide-and-read-back-from-schema systems.

#### 6.19.1 OpAttrs channel definition

This subsection pins the OpAttrs channel itself: its single normative owner, its
distinctness from KISS-Grammar `pattern_attrs`, the closed carrier-op set, and the
general encoding invariants (per-op field-order ABI, explicit default-resolution,
reserve-`0` little-endian enum ordinals, fixed-width little-endian two's-complement
integers, definite lengths, opaque byte-compared embedding, and version binding). The
per-field frozen sub-vocabularies follow in §6.19.2, the per-op schemas in §6.19.3, the
`structure_key` reconciliation in §6.19.4, and the pinned foundational constants and
cross-op axis rules in §6.19.5.

- **KISS-OPS-6.19-0001** — KISS-Ops MUST be the single normative owner of the OpAttrs
  channel — the per-op compile-time attribute record that is part of an op's semantics
  — pinning its per-op schema, its sub-vocabularies, and its canonical wire encoding in
  this section; no downstream sub-standard MUST re-pin or fork an OpAttrs sub-vocabulary.
  *Test:* `test_ops_opattrs_channel_concept`.
- **KISS-OPS-6.19-0002** — The OpAttrs channel MUST be treated as distinct from
  KISS-Grammar `pattern_attrs`: OpAttrs is part of an op's computed semantics (it
  changes the result), whereas `pattern_attrs` are recognition/matching hints on the
  advertisable surface; an implementation MUST NOT collapse the two or derive one from
  the other. *Test:* `test_ops_opattrs_distinct_from_pattern_attrs`.
- **KISS-OPS-6.19-0003** — The set of **carrier** ops (ops with a non-empty OpAttrs
  schema) for this op-set version MUST be exactly `{reduce, prefix_scan, gather,
  scatter, sort_network, reduce_var, reduce_std, softmax, log_softmax, rms_norm,
  layer_norm, avg_pool, max_pool, im2col, index_select, embedding, scatter_add}`; every
  other op MUST have an empty OpAttrs blob (definite length `0`), and any semantics-
  affecting axis of a non-carrier non-primitive op MUST be determined by its reference
  decomposition's inner carrier node (§6.19-0036), not by a free OpAttrs field. *Test:*
  `test_ops_opattrs_carrier_set_closed`.
- **KISS-OPS-6.19-0004** — Each carrier op's OpAttrs blob MUST be exactly its schema
  fields (§6.19.3) concatenated in the canonical, frozen field order shown, and that
  per-op field order MUST be the ABI; attribute names MUST NOT appear on the wire, there
  MUST be no name-sorted dictionary and no self-describing tag stream, and a reader MUST
  replay the schema positionally after the `op_name` selects it. *Test:*
  `test_ops_opattrs_field_order_abi`.
- **KISS-OPS-6.19-0005** — Every OpAttrs schema field MUST be emitted **explicitly** and
  already **resolved** to its effective value (schema default applied at encode time);
  an encoder MUST NOT omit a field because it equals its default, so a defaulted
  attribute and an explicitly-stated equal attribute MUST produce identical bytes.
  *Test:* `test_ops_opattrs_explicit_default_resolution`.
- **KISS-OPS-6.19-0006** — Every enumerated OpAttrs sub-vocabulary MUST be encoded as a
  frozen little-endian unsigned-integer ordinal with ordinal `0` reserved as an
  invalid/unspecified sentinel that a conforming encoder never emits and a reader
  rejects; ordinal assignments MUST be additive-only and MUST NOT be reused after
  retirement, and no enum spelling MUST appear in the canonical wire form. *Test:*
  `test_ops_opattrs_enum_ordinal_reserve_zero`.
- **KISS-OPS-6.19-0007** — Every integer OpAttrs field MUST be encoded at the
  fixed width pinned in its schema (not chosen by magnitude), little-endian, two's-
  complement: axis-index and operand-index and vector length-prefixes and `u8` enum
  ordinals as one byte, `reduce_axes` as `u16` LE, and window/stride/dilation/padding
  elements as `u32` LE. *Test:* `test_ops_opattrs_int_fixed_width_le`.
- **KISS-OPS-6.19-0008** — The OpAttrs canonical wire form MUST be little-endian
  throughout, consistent with the KISS-Announce POD and KISS-Grammar region wire forms;
  KISS-Ops MUST NOT emit any OpAttrs field big-endian. *Test:*
  `test_ops_opattrs_little_endian`.
- **KISS-OPS-6.19-0009** — A genuinely optional OpAttrs attribute MUST occupy an explicit
  slot and MUST NOT be represented by omission. Every schema field in this op-set version
  is either **mandatory** (caller-supplied, e.g. `monoid`, a mandatory `reduce_axes`,
  `axis`, `index_operand`, `index_dtype`, `norm_axis`) or **mandatory-with-resolved-
  default** (e.g. `oob_policy`, `direction`, `exclusivity`, `count_include_pad`,
  `output_column_ordering`, and the fixed-by-decomposition or fixed-constant flags), and
  in all cases is **always present and explicitly emitted** at its effective value; a
  fixed-by-decomposition or fixed-constant value MUST still be emitted explicitly as its
  pinned constant, and no field's presence is ever signalled by a length the encoder may
  choose. The `reduce_axes` sub-vocabulary reserves `0x0000` for the none/not-a-reduction
  category, but that value is **unreachable** in any carrier OpAttrs blob of this version
  (§6.19-0038). *Test:* `test_ops_opattrs_optional_explicit_slot`.
- **KISS-OPS-6.19-0010** — OpAttrs MUST use definite lengths only: each ordered vector
  (window-parameter, permutation) MUST carry its `u8` element-count prefix immediately
  before its elements, and the whole OpAttrs blob MUST have a definite length equal to
  the sum of its fixed fields plus vector payloads. KISS-Ops owns this internal definite
  length; the embedding layer frames the blob length-prefixed (KISS-Grammar §6.8-0007
  wraps it as a `u16` LE byte-length followed by the verbatim blob; an empty blob frames
  as `0x0000`) and MUST NOT parse inside it. *Test:*
  `test_ops_opattrs_definite_length_prefix`.
- **KISS-OPS-6.19-0011** — The OpAttrs layout (the carrier-op set, per-op field order,
  field widths, and sub-vocabulary ordinals) MUST be bound to the KISS-Ops op-set /
  frozen-shape schema version; the shared `reduce_axes` anchor MUST additionally be
  co-versioned with the Classify `structure_key` schema version (`STRUCTURE_KEY_VERSION`)
  whose §6.7-0005 token codec it mirrors, and the pinned `MAX_RANK` / `MAX_OPERANDS`
  constants (§6.19-0037) MUST be co-versioned with their Classify shared-anchor values.
  Any non-additive change (reordering a field, re-widening, reusing an ordinal, changing
  the `reduce_axes` multiplexing, or changing a pinned constant) MUST be a new canonical
  byte form under a bumped version, never a silent in-place change. *Test:*
  `test_ops_opattrs_version_binding`.
- **KISS-OPS-6.19-0012** — KISS-Grammar and KISS-Contract MUST embed the encoded OpAttrs
  blob as **opaque** KISS-Ops-owned bytes and MUST byte-compare it without parsing inside
  it; because every field is explicit and default-resolved (§6.19-0005), an embedding
  layer MUST NOT perform any default-normalization of its own, and two producers that
  resolve an op's attributes to the same values MUST produce byte-identical blobs. *Test:*
  `test_ops_opattrs_opaque_embedding_byte_compare`.
- **KISS-OPS-6.19-0013** — Conformance of the OpAttrs encoding MUST be demonstrated by
  golden vectors (attributes → exact little-endian hex bytes) covering every
  sub-vocabulary and every carrier-op schema, reproduced by at least two structurally
  dissimilar implementations (the umbrella §5.3 freeze gate) before the encoding is
  declared frozen; Appendix E carries the informative worked vectors and each cites its
  pinning clause. *Test:* `test_ops_opattrs_golden_vector_conformance`.

#### 6.19.2 OpAttrs sub-vocabularies (frozen encodings)

Each sub-vocabulary below is frozen: enum assignments are additive-only, code points
are never reused after retirement, and no enum spelling ever appears on the wire.
Ordinal `0` is a reserved invalid/unspecified sentinel that a conforming encoder
never emits and a reader rejects (the boolean-flag encoding is exempt — both `0` and
`1` are meaningful there). The pinned integer values of `MAX_RANK` and `MAX_OPERANDS`
that bound the axis, operand-index, subset-mask, and vector-length fields below are
inlined normatively in §6.19-0037.

| Sub-vocabulary | Kind / width | Code points |
|---|---|---|
| `monoid` | enum, `u8` | `0`=RESERVED, `1`=sum, `2`=prod, `3`=max, `4`=min |
| `oob_policy` | enum, `u8` | `0`=RESERVED, `1`=skip, `2`=clamp, `3`=zero-fill |
| `scatter_combine` | enum, `u8` | `0`=RESERVED, `1`=assign, `2`=atomic-add, `3`=atomic-max, `4`=atomic-min |
| `index_dtype` | enum, `u8` | `0`=RESERVED, `1`=u32, `2`=i32, `3`=i64 |
| `sort_direction` | enum, `u8` | `0`=RESERVED, `1`=ascending, `2`=descending |
| `scan_exclusivity` | enum, `u8` | `0`=RESERVED, `1`=inclusive, `2`=exclusive |
| `column_ordering` | enum, `u8` | `0`=RESERVED, `1`=channel-major (tap-minor), `2`=tap-major (channel-minor) |
| `boolean-flag` | bool, `u8` | `0`=false, `1`=true (both meaningful; `0` NOT reserved) |
| `reduce_axes` | multiplexed category+bitmask, `u16` LE | `0x0000`=none/not-a-reduction; `0x0001..0x00FF`=per-axis subset keepdim mask (bit `k`⇔axis `k`, ≥1 bit set); `0x0100..0xFFFD`=RESERVED (never emitted); `0xFFFE`=trailing-axis (innermost) only; `0xFFFF`=all-axes. Category selected by the rank-aware total precedence of §6.19-0020 (all-axes beats trailing beats subset; `0xFFFF` covers the rank-1 sole-axis case and every all-axes case) |
| `window-param-vector` | ordered length-prefixed vector | `u8` element-count prefix (`0..MAX_RANK`), then count × `u32` LE elements, in ascending operand spatial-axis order (element `i` ⇔ spatial axis `i`, outermost spatial axis first; never sorted) |
| `permutation` | ordered length-prefixed vector (RESERVED) | `u8` element-count prefix (`0..MAX_RANK`), then count × `u8` axis-index elements forming a valid permutation of `0..rank-1`, in order (never sorted) |

- **KISS-OPS-6.19-0014** — The `monoid` OpAttrs field MUST be encoded as the frozen
  `u8` ordinal `{1=sum, 2=prod, 3=max, 4=min}` with `0` reserved (never emitted); it
  denotes the §6.11-0002 fold operator, is a member of an op's semantics (not part of
  the op token), and MUST distinguish e.g. `reduce(sum)` from `reduce(max)` by ordinal
  alone. *Test:* `test_ops_opattrs_monoid_enum`.
- **KISS-OPS-6.19-0015** — The `oob_policy` OpAttrs field MUST be encoded as the frozen
  `u8` ordinal `{1=skip, 2=clamp, 3=zero-fill}` with `0` reserved; a `gather` (read)
  MAY carry any of the three, a `scatter` (write) MUST carry only ordinal `1` (skip)
  per §6.11-0005, and this single enum is also the frozen home of the pool/`im2col`
  boundary-fill policy (no separate pad-policy enum is minted; pad-fill IS `oob_policy`,
  matching KISS-Grammar §6.2-0004 verbatim). *Test:* `test_ops_opattrs_oob_policy_enum`.
- **KISS-OPS-6.19-0016** — The `scatter_combine` OpAttrs field MUST be encoded as the
  frozen `u8` ordinal `{1=assign, 2=atomic-add, 3=atomic-max, 4=atomic-min}` with `0`
  reserved, denoting the §6.11-0005 write-combine algebra. *Test:*
  `test_ops_opattrs_scatter_combine_enum`.
- **KISS-OPS-6.19-0017** — The `index_dtype` OpAttrs field MUST be encoded as the frozen
  `u8` ordinal `{1=u32, 2=i32, 3=i64}` with `0` reserved, naming the §6.11-0009 legal
  index-operand dtype; this enum is a distinct three-value axis and MUST NOT be conflated
  with the data-vocabulary storage-dtype token set. *Test:*
  `test_ops_opattrs_index_dtype_enum`.
- **KISS-OPS-6.19-0018** — The `sort_direction` OpAttrs field MUST be encoded as the
  frozen `u8` ordinal `{1=ascending, 2=descending}` with `0` reserved (§6.11-0007
  default ascending). *Test:* `test_ops_opattrs_sort_direction_enum`.
- **KISS-OPS-6.19-0019** — The `scan_exclusivity` OpAttrs field MUST be encoded as the
  frozen `u8` ordinal `{1=inclusive, 2=exclusive}` with `0` reserved (§6.11-0003 default
  inclusive); the `prefix_scan` schema field `exclusivity` (§6.19-0026) draws from this
  sub-vocabulary and MUST emit the ordinal, never a boolean truth value. *Test:*
  `test_ops_opattrs_scan_exclusivity_enum`.
- **KISS-OPS-6.19-0020** — The `reduce_axes` OpAttrs field MUST be encoded as a single
  canonical `u16` little-endian value multiplexing the four §6.11-0011 categories:
  `0x0000`=none/not-a-reduction, `0x0001..0x00FF`=an explicit per-axis subset keepdim
  mask (bit `k` selects axis `k`, at least one bit set), `0x0100..0xFFFD`=RESERVED
  (never emitted), `0xFFFE`=trailing-axis only, `0xFFFF`=all-axes. The low byte doubling
  as a `u8` per-axis mask requires `MAX_RANK <= 8` (§6.19-0037). Because the operand rank
  is NOT carried in the blob, the category MUST be selected by the following rank-aware
  **total precedence**, stated here in full (not by cross-reference) so the Ops binary
  encoding is self-contained.

  **Reductions (`reduce`, `reduce_var`, `reduce_std`).** Let `S ⊆ {0,1,…,r-1}` be the set
  of reduced axes over an operand of rank `r` (`1 <= r <= MAX_RANK <= 8`). Apply these
  three tests in this exact order; they are mutually exclusive by construction: (1) emit
  `0xFFFF` **if and only if** `S == {0,1,…,r-1}` (every axis of the operand is selected);
  (2) otherwise emit `0xFFFE` **if and only if** `S == {r-1}` **and** `r > 1` (exactly the
  single trailing axis); (3) otherwise emit the `u8` subset mask `Σ over k∈S of (1 << k)`
  in the low byte with high byte `0x00`. Consequences, each load-bearing for byte-identity
  (§6.19-0012): a **rank-1** reduction over its sole axis MUST encode `0xFFFF` (it is
  all-axes), never the single-bit subset mask `0x0001`; a reduction whose `S` covers all
  `r` axes MUST encode `0xFFFF`, **never a subset mask, even one whose set bits cover all
  `r` axes** (e.g. a rank-3 reduction over `{0,1,2}` MUST encode `0xFFFF`, never `0x0007`);
  `0xFFFE` is emitted only for `r > 1` with `S` exactly the trailing axis; and a `reduce`
  / `reduce_var` / `reduce_std` MUST NOT emit `0x0000` (§6.19-0038).

  **Scan (`prefix_scan`).** A `prefix_scan` folds over exactly one axis `a`. Apply in
  order: (1) if `r > 1` **and** `a == r-1` (the trailing axis), emit `0xFFFE`; (2)
  otherwise — a non-trailing single axis, **or** the sole axis of a rank-1 operand — emit
  the single-bit subset mask `1 << a`. A `prefix_scan` MUST NOT emit `0xFFFF` and MUST NOT
  emit `0x0000`. *Test:* `test_ops_opattrs_reduce_axes_multiplex`.
- **KISS-OPS-6.19-0021** — The `output_column_ordering` OpAttrs field MUST be encoded as
  the frozen `u8` ordinal `{1=channel-major (tap-minor), 2=tap-major (channel-minor)}`
  with `0` reserved, drawing its ordinals from the `column_ordering` sub-vocabulary of
  the §6.19.2 table (the schema field is named `output_column_ordering` and draws from the
  `column_ordering` sub-vocabulary, exactly as the `exclusivity` field draws from the
  `scan_exclusivity` sub-vocabulary); it declares the `im2col` window-tap/channel flatten
  order into the column dimension (§6.13-0004) so its index mapping is fully determined.
  *Test:* `test_ops_opattrs_column_ordering_enum`.
- **KISS-OPS-6.19-0022** — Every boolean OpAttrs field (`keepdim`, `stability`,
  `bessel_correction`, `count_include_pad`) MUST be encoded as a `u8` with `0`=false and
  `1`=true; a boolean field is NOT subject to the reserve-`0` enum rule (both values are
  meaningful), and MUST be emitted explicitly at its resolved value even where this
  op-set version pins it to a constant (`keepdim`=1 per §6.11-0008, `stability`=1 per
  §6.11-0007). *Test:* `test_ops_opattrs_boolean_flags`.
- **KISS-OPS-6.19-0023** — Every window-parameter OpAttrs vector (`window_size`,
  `stride`, `dilation`, `padding`) MUST be encoded as a `u8` element-count prefix
  (`0..MAX_RANK`, §6.19-0037) immediately followed by that many `u32` little-endian
  elements, where **element `i` corresponds to spatial axis `i` in ascending operand
  spatial-axis index order (the outermost spatial axis first, the innermost spatial axis
  last)**; the elements MUST be preserved in that order and MUST NOT be sorted (an ordered
  vector). *Test:* `test_ops_opattrs_window_param_vector`.
- **KISS-OPS-6.19-0024** — The `permutation` sub-vocabulary MUST be encoded as a `u8`
  element-count prefix (`0..MAX_RANK`) followed by that many `u8` axis-index elements
  forming a valid permutation of `0..rank-1`, preserved in the given order (never
  sorted); it is **frozen and reserved** in this op-set version because KISS-Grammar
  §6.2-0006 names KISS-Ops as its owner, but NO op in this version carries a free
  `permutation` field (`sort_network` emits its permutation as a runtime index-vector
  output per §6.11-0007, not a compile-time attribute), so a conforming encoder MUST NOT
  emit a `permutation` field on any op of this version. *Test:*
  `test_ops_opattrs_permutation_reserved`.

#### 6.19.3 Per-op OpAttrs schema (canonical field order = ABI)

For each carrier op the OpAttrs blob is exactly the fields below, in the canonical
order shown, each at its pinned width, each emitted **explicitly** at its
**resolved** value. Attribute names never appear on the wire; the `op_name` (carried
by the embedding layer) selects the schema and the reader replays it positionally.

| Op | Canonical field order (name : encoding : resolved default) |
|---|---|
| `reduce` | `monoid`:enum `u8`:mandatory (identity-bearing, no default) — `reduce_axes`:`u16` LE:mandatory (`rall`/`rlast`/subset; `0x0000` forbidden, §6.19-0038) — `keepdim`:bool `u8`:`1` (fixed, §6.11-0008) |
| `prefix_scan` | `monoid`:enum `u8`:mandatory — `reduce_axes`:`u16` LE:mandatory (exactly one axis; §6.19-0020) — `exclusivity`:`scan_exclusivity` enum `u8`:`1` (inclusive) |
| `gather` | `axis`:`u8`:mandatory (`0..MAX_RANK-1`, §6.11-0013) — `oob_policy`:enum `u8`:`1` (skip) — `index_operand`:`u8`:mandatory (`0..MAX_OPERANDS-1`) — `index_dtype`:enum `u8`:mandatory |
| `scatter` | `axis`:`u8`:mandatory (§6.11-0013) — `combine`:enum `u8`:`1` (assign) — `oob_policy`:enum `u8`:`1` (skip, fixed) — `index_operand`:`u8`:mandatory — `index_dtype`:enum `u8`:mandatory |
| `sort_network` | `axis`:`u8`:trailing axis `r-1` (§6.11-0014) — `direction`:enum `u8`:`1` (ascending) — `stability`:bool `u8`:`1` (fixed, §6.11-0007) |
| `reduce_var` | `reduce_axes`:`u16` LE:mandatory (`0x0000` forbidden, §6.19-0038) — `keepdim`:bool `u8`:`1` — `bessel_correction`:bool `u8`:`0` (population) |
| `reduce_std` | `reduce_axes`:`u16` LE:mandatory (`0x0000` forbidden, §6.19-0038) — `keepdim`:bool `u8`:`1` — `bessel_correction`:bool `u8`:`0` (population) |
| `softmax` | `norm_axis`:`u8`:mandatory (§6.13-0004) |
| `log_softmax` | `norm_axis`:`u8`:mandatory (§6.13-0004) |
| `rms_norm` | `norm_axis`:`u8`:mandatory (§6.13-0008; `eps`=`param(0)`, `gamma`=`input(1)` are operands/params, NOT OpAttrs) |
| `layer_norm` | `norm_axis`:`u8`:mandatory (§6.13-0008; `eps`=`param(0)`, `gamma`=`input(1)`, `beta`=`input(2)` are operands/params, NOT OpAttrs) |
| `avg_pool` | `window_size`:vec — `stride`:vec — `dilation`:vec — `padding`:vec — `count_include_pad`:bool `u8`:`1` (include pad) |
| `max_pool` | `window_size`:vec — `stride`:vec — `dilation`:vec — `padding`:vec |
| `im2col` | `window_size`:vec — `stride`:vec — `dilation`:vec — `padding`:vec — `output_column_ordering`:enum `u8`:`1` (channel-major) |
| `index_select` | `axis`:`u8`:mandatory (§6.11-0013) — `index_operand`:`u8`:mandatory — `index_dtype`:enum `u8`:mandatory (oob FIXED skip by decomposition, carries no free `oob` field) |
| `embedding` | `axis`:`u8`:mandatory (§6.11-0013) — `index_operand`:`u8`:mandatory — `index_dtype`:enum `u8`:mandatory (oob FIXED zero-fill by decomposition) |
| `scatter_add` | `axis`:`u8`:mandatory (§6.11-0013) — `index_operand`:`u8`:mandatory — `index_dtype`:enum `u8`:mandatory (combine FIXED atomic-add, oob FIXED skip by decomposition) |

- **KISS-OPS-6.19-0025** — The `reduce` OpAttrs blob MUST be exactly `monoid` (`u8`,
  mandatory non-zero ordinal) then `reduce_axes` (`u16` LE, `rall`/`rlast`/subset;
  `0x0000` forbidden per §6.19-0020/§6.19-0038) then `keepdim` (bool `u8`, fixed `1`), in
  that order. *Test:* `test_ops_opattrs_reduce_schema`.
- **KISS-OPS-6.19-0026** — The `prefix_scan` OpAttrs blob MUST be exactly `monoid`
  (`u8`, mandatory) then `reduce_axes` (`u16` LE, selecting exactly one axis per the
  scan precedence of §6.19-0020: `0xFFFE` when that axis is the trailing axis of a
  rank-`>1` operand, otherwise the single-bit subset mask; `0xFFFF` and `0x0000` MUST NOT
  be emitted) then `exclusivity` (`scan_exclusivity` enum `u8`, default `1` inclusive;
  the field draws from the `scan_exclusivity` sub-vocabulary and MUST emit the enum
  ordinal — `1`=inclusive, `2`=exclusive — not a boolean truth value), in that order.
  *Test:* `test_ops_opattrs_prefix_scan_schema`.
- **KISS-OPS-6.19-0027** — The `gather` OpAttrs blob MUST be exactly `axis` (`u8`, the
  indexed axis of §6.11-0013, range `0..MAX_RANK-1`) then `oob_policy` (enum `u8`, default
  `1` skip) then `index_operand` (`u8`, range `0..MAX_OPERANDS-1`) then `index_dtype`
  (enum `u8`, mandatory), in that order. *Test:* `test_ops_opattrs_gather_schema`.
- **KISS-OPS-6.19-0028** — The `scatter` OpAttrs blob MUST be exactly `axis` (`u8`, the
  indexed axis of §6.11-0013) then `combine` (`scatter_combine` enum `u8`, default `1`
  assign) then `oob_policy` (enum `u8`, fixed `1` skip) then `index_operand` (`u8`) then
  `index_dtype` (enum `u8`, mandatory), in that order. *Test:*
  `test_ops_opattrs_scatter_schema`.
- **KISS-OPS-6.19-0029** — The `sort_network` OpAttrs blob MUST be exactly `axis` (`u8`,
  the permuted axis of §6.11-0014, resolving to the trailing axis `r-1`) then `direction`
  (`sort_direction` enum `u8`, default `1` ascending) then `stability` (bool `u8`, fixed
  `1` stable), in that order. *Test:* `test_ops_opattrs_sort_network_schema`.
- **KISS-OPS-6.19-0030** — The `reduce_var` and `reduce_std` OpAttrs blobs MUST each be
  exactly `reduce_axes` (`u16` LE, mandatory; `0x0000` forbidden per §6.19-0038) then
  `keepdim` (bool `u8`, fixed `1`) then `bessel_correction` (bool `u8`, default `0`
  population, §6.13-0004), in that order. *Test:*
  `test_ops_opattrs_reduce_var_std_schema`.
- **KISS-OPS-6.19-0031** — The `softmax`, `log_softmax`, `rms_norm`, and `layer_norm`
  OpAttrs blobs MUST each be exactly a single `norm_axis` (`u8`, mandatory, range
  `0..MAX_RANK-1`) field — the normalization axis pinned by §6.13-0004 for
  `softmax`/`log_softmax` and by §6.13-0008 for `rms_norm`/`layer_norm`; the
  `eps`/`gamma`/`beta` quantities of `rms_norm`/`layer_norm` are operands/params
  (§6.13 operand-ordering convention), NOT OpAttrs fields. *Test:*
  `test_ops_opattrs_norm_axis_schema`.
- **KISS-OPS-6.19-0032** — The `avg_pool` OpAttrs blob MUST be exactly `window_size`,
  `stride`, `dilation`, `padding` (each a window-parameter vector, §6.19-0023) then
  `count_include_pad` (bool `u8`, default `1`), in that order; the `max_pool` OpAttrs
  blob MUST be exactly `window_size`, `stride`, `dilation`, `padding` in that order with
  no trailing flag. *Test:* `test_ops_opattrs_pool_schema`.
- **KISS-OPS-6.19-0033** — The `im2col` OpAttrs blob MUST be exactly `window_size`,
  `stride`, `dilation`, `padding` (each a window-parameter vector) then
  `output_column_ordering` (`column_ordering` enum `u8`, default `1` channel-major), in
  that order. *Test:* `test_ops_opattrs_im2col_schema`.
- **KISS-OPS-6.19-0034** — The `index_select`, `embedding`, and `scatter_add` OpAttrs
  blobs MUST each be exactly `axis` (`u8`, the indexed axis of §6.11-0013) then
  `index_operand` (`u8`) then `index_dtype` (enum `u8`), in that order; these
  gather/scatter wrappers MUST NOT carry a free `oob_policy` or `combine` field, because
  those values are fixed by their §6.13 decompositions (`index_select`→skip,
  `embedding`→zero-fill, `scatter_add`→atomic-add + skip). *Test:*
  `test_ops_opattrs_gather_scatter_wrapper_schema`.

#### 6.19.4 Reconciliation with KISS-Classify `structure_key`

The `reduce_axes` OpAttrs field and the KISS-Classify `structure_key` reduce-field
share **one** four-category vocabulary (§6.11-0011): none / all-axes / trailing-axis /
subset mask. They are carried on **two distinct channels** — Classify's is the coarse
cell discriminator serialized by the `structure_key` **string token codec** (Classify
§6.7-0005: `-` / `rall` / `rlast` / `x<hh>`, the sole normative pinning of that field);
KISS-Ops's is the per-op **binary** OpAttrs field (`u16` LE, §6.19-0020). The two
agree on categories 1:1 (`-`↔`0x0000`, `rall`↔`0xFFFF`, `rlast`↔`0xFFFE`, `x<hh>`↔the
`u8` subset mask `0x00hh` in `0x0001..0x00FF`) and are co-versioned (§6.19-0011), but
the OpAttrs binary form is a KISS-Ops-owned encoding for the OpAttrs channel and does
NOT re-pin or contradict the Classify token codec, which remains the normative form for
`structure_key`; the earlier Ops-local in-memory hex illustration of §6.11-0011 is
promoted to this normative OpAttrs encoding for that channel only.

- **KISS-OPS-6.19-0035** — The `reduce_axes` OpAttrs `u16` encoding MUST reconcile with
  the KISS-Classify `structure_key` reduce-field 1:1 on the four categories
  (`-`↔`0x0000`, `rall`↔`0xFFFF`, `rlast`↔`0xFFFE`, `x<hh>`↔the `u8` subset mask), MUST
  be co-versioned with `STRUCTURE_KEY_VERSION`, and MUST NOT be read as re-pinning or
  overriding the Classify §6.7-0005 token codec (the normative `structure_key` form);
  the OpAttrs binary form is owned by KISS-Ops for the OpAttrs channel and the string
  token codec is owned by KISS-Classify for `structure_key`, the two being distinct
  channels that agree by construction. *Test:*
  `test_ops_opattrs_reduce_axes_classify_reconciliation`.

#### 6.19.5 Foundational constants and cross-op axis resolution

- **KISS-OPS-6.19-0036** — Several advertised, axis-parameterized non-primitive ops are
  **not** carriers (they hold no free `reduce_axes` or `axis` OpAttrs field):
  `reduce_mean`, `reduce_norm2`, `logsumexp`, `argmax`, `any`, `all`, `cumsum`, `cumprod`,
  and `cummax`. A consumer that natively matches one of these ops — and therefore does not
  expand its §6.13 reference decomposition (§6.14-0004) — MUST obtain the reduce/scan axis
  by resolving the axis of the **inner carrier node** of that op's reference decomposition
  (the `reduce`, `prefix_scan`, or `sort_network` the op is defined over), even though it
  does not fully expand the decomposition; the axis is therefore always determined
  (§6.19-0003) and is never unspecified for a native matcher. `reduce_var` and
  `reduce_std` are carriers of a free `reduce_axes` field **not** because their axis would
  otherwise be unspecified but solely because they additionally carry the
  `bessel_correction` attribute (§6.19-0030), which has no inner-carrier source and must be
  declared alongside the axis; `reduce_mean`, `reduce_norm2`, and `logsumexp` carry no such
  extra attribute and so remain non-carriers whose axis is resolved from their inner
  `reduce` node. *Test:* `test_ops_opattrs_noncarrier_axis_resolution`.
- **KISS-OPS-6.19-0037** — For the OpAttrs channel of this op-set version the pinned
  constants that bound the `axis`, `index_operand`, subset-mask, and vector-length fields
  MUST take the concrete values `MAX_RANK = 8` and `MAX_OPERANDS = 8`. These are the same
  shared-anchor constants referenced by name from the data vocabulary (§2.8); their values
  are inlined here (exactly as §6.16 inlines the dtype bit layouts) so this document is
  self-contained and every range MUST of §6.19 is verifiable from KISS-Ops plus the
  umbrella alone, and they are **co-versioned with Classify** (a change to either value in
  the shared anchor co-bumps the OpAttrs version, §6.19-0011). `MAX_RANK = 8` satisfies the
  KISS-Ops-local invariant `MAX_RANK <= 8` that the `reduce_axes` low-byte `u8` per-axis
  subset mask (§6.19-0020) requires. A conforming encoder MUST NOT emit an `axis` field
  `>= MAX_RANK`, an `index_operand` field `>= MAX_OPERANDS`, a subset mask with a bit set
  at a position `>= MAX_RANK`, or a window/permutation vector whose element count exceeds
  `MAX_RANK`; a reader MUST reject any such out-of-range field with a typed decline.
  *Test:* `test_ops_opattrs_max_rank_operands_pinned`.
- **KISS-OPS-6.19-0038** — `reduce_var` and `reduce_std` MUST NOT emit `reduce_axes=0x0000`,
  exactly as `reduce` and `prefix_scan` MUST NOT (§6.19-0020). Because every carrier op
  that owns a `reduce_axes` field (`reduce`, `prefix_scan`, `reduce_var`, `reduce_std`) is
  a reduction or a scan, the none/not-a-reduction sentinel `0x0000` is **unreachable** in
  any OpAttrs blob of this op-set version — its sole role is the Classify `structure_key`
  `-` token reconciliation (§6.19-0035), not the OpAttrs channel. A reader MUST reject
  `reduce_axes=0x0000` on any carrier OpAttrs blob as malformed. *Test:*
  `test_ops_opattrs_reduce_axes_zero_unreachable`.

#### 6.19.6 Index-lane wire references (recipe/FlatDag seam)

The recipe/FlatDag container wire that carries these references — its node list, its
value-lane `outputs` root list, and its framing — is **not yet pinned in the KISS spec
tree**; it is owned by the recipe-grammar consolidation (issue #67) and its byte grammar
(PR #78). The terms these clauses reference — the external index-input array, node ids,
and the `outputs` root list — are defined by that container wire (#67 / PR #78); this
subsection pins only their **index-lane encoding**. The two clauses below pin the encodings
the reference implementation and Baracuda have already locked (issue #76): how an
index-consuming node names its index operand, and how the recipe exports an index-lane
product. They ride the recipe wire's next `SCHEMA_VERSION` bump.

The field grammar is given in the kiss-ref draft form (a `u8` tag plus a `varint`); it is
**decoded-semantics normative** and will be reconciled onto the PR #78 byte-grammar owner's
conventions without changing these decoded semantics. To make that hand-off unambiguous,
the line between what any encoding MUST preserve and what PR #78 MAY re-spell is: the
**normative** decoded semantics — surviving any re-spelling — are (a) exactly two
index-source kinds, external-slot and node-product, distinguished by a leading discriminant;
(b) reserved-is-error on every undefined discriminant value; (c) the decode-time validations
(a `node` target that is out of range or has no index-lane product, and likewise an
`index_outputs` entry, is a typed decline); (d) the canonical form with no dual-form
aliasing; and (e) `index_outputs` ordered after `outputs`, symmetric, count-prefixed, with
unconditional presence at a version boundary. The **#78-reconcilable spelling** — changeable
without a further ruling — is the `varint` width of each integer payload (`ref_value`,
`count`, `node_id`) and whether the discriminant values are literally `0x00` / `0x01`; the
`u8` tag is already width- and endianness-neutral, so a fixed-width-LE realization is
conformant **iff** it preserves (a)–(e). The third §6.19 wire-pin — the `sort_network`
index-lane **output dtype** — is resolved **producer-side with no wire field** at
§6.11-0019, so it carries no clause here.

- **KISS-OPS-6.19-0039** — Every **index-operand** field of an index-consuming node
  (`gather.index`, `scatter.index`, and the §6.13-0009 gather/scatter wrappers) MUST be
  encoded as an **`index_ref`**: `index_ref := ref_tag:u8, ref_value:varint`.
  `ref_tag = 0x00` (**slot**) MUST mean `ref_value` is an index into the recipe's external
  index-input array (the only form v1 emitters produce). `ref_tag = 0x01` (**node**) MUST
  mean `ref_value` is the id of a node whose **index-lane product** (this op-set version:
  the `sort_network` original-index output of §6.11-0007) the operand consumes — a true
  dataflow edge that a scheduler MUST treat as a dependency, with the §6.14 acyclicity /
  cycle rules unchanged. Every `ref_tag >= 0x02` MUST be **reserved**, and a decoder MUST
  **reject it with a typed decline** (KISS-Conform §6.7), never skip it (reserved-is-error,
  matching the §6.19-0006 reserve-`0` and §6.19-0024 permutation precedents). Decode-time
  validation (MUST): a `node` reference whose target id is `>= node_count`, or whose target
  has **no** index-lane product, MUST be a typed decline. Canonical form (MUST): an
  external operand MUST be emitted as `slot`; the same operand MUST NOT be aliased through
  both the `slot` and `node` forms. *Test:* `test_ops_index_ref_wire_form`.
- **KISS-OPS-6.19-0040** — The recipe/FlatDag MUST carry an **`index_outputs`** root list
  — the node ids whose §6.11-0007 index-lane product is exported — immediately **after**
  the value-lane `outputs` root list and **symmetric** with it: `index_outputs :=
  count:varint, node_id:varint × count`, in output order. Its presence MUST be
  **unconditional**, riding the next `SCHEMA_VERSION` bump of the recipe wire (`count = 0`
  for a value-lane-only recipe — one byte — bit-compatible with a lattice that exports no
  index output). Each listed `node_id` MUST have an index-lane product, or the decoder MUST
  raise a **typed decline** (KISS-Conform §6.7), as in §6.19-0039.
  routed to #67 and not resolved here: whether the recipe-wire version axis this field
  rides is the **same** axis as the KISS-Classify `structure_key` `SCHEMA_VERSION` is
  deferred to the recipe-grammar consolidation (#67) sequencing; this clause pins the field
  and its unconditional presence, not which version counter advances.)* *Test:*
  `test_ops_index_outputs_root_list`.

### 6.20 Op shape rules — the shape-side oracle

Every op's output shape is a function of its operands' shapes together with its
OpAttrs (§6.19) and declared params. §6.13 pins each op's **value** behaviour (the
reference decomposition, whose fully-lowered form is the KISS-Contract §6.4-0006
value oracle); this subsection pins the **shape** behaviour as its companion, in a
small closed vocabulary that is evaluable against concrete operand shapes and
serializable under the §6.19 canonical discipline. Most ops derive their output
shape from their operands directly (elementwise, `matmul`, `concat`, axis-based
`reduce` with `keepdim`, `transpose`, `unsqueeze`, `cast`) and carry no shape attr;
the only *irreducible* free cases are a **broadcast target** (another operand's
whole shape) and a **slice/iota offset** (arithmetic on an operand's extent), so
the shared surface is two constructors.

- **KISS-OPS-6.20-0001** — Every op MUST have a **shape rule**: a function from the
  concrete shapes of its operands (in KISS-Classify canonical operand order, §6.5)
  together with its OpAttrs (§6.19) and declared params to the concrete shape of its
  output(s). The shape rule is the **shape-side companion to the KISS-Contract
  §6.4-0006 value oracle** — the value oracle pins what a kernel computes, the shape
  rule pins the shape that computation produces. An implementation MUST NOT leave an
  op's output shape underivable from its operand shapes plus attrs, and MUST NOT
  declare an output shape that disagrees with its op's shape rule. *Test:*
  `test_shape_rule_exists_and_matches`.
- **KISS-OPS-6.20-0002** — A shape rule MUST be expressed in the closed
  **shape-expression vocabulary** `ShapeExpr := SameAs(operand)` (the operand's whole
  shape) and `DimExpr := Extent(operand, axis) | Const(i64) | Param(field) | DimExpr
  BinOp DimExpr` with `BinOp ∈ {+, −, ×, ÷}`. An `operand` reference MUST be a
  **positional** operand index in KISS-Classify canonical operand order — an op_dag
  interior node carries no operand-role tuple (KISS-CONTRACT §6.4-0009), so a role
  name is a KISS-Grammar/Contract surface alias defined by the position mapping,
  never a second wire form. An `axis` MUST be a **non-negative** operand-axis index
  or the reserved **`last`** sentinel denoting the trailing axis — the KISS axis
  convention is non-negative (§6.19-0007, §6.13-0008); a signed/negative axis MUST
  NOT appear. An implementation MUST NOT introduce a constructor outside this
  vocabulary; `Reduce(operand, axis, keepdim)`, `WithDim(operand, axis, DimExpr)`, and
  `Dims([DimExpr, …])` are **reserved** and MUST NOT be emitted by a producer at this
  vocabulary version (they enter through the extension registry, umbrella §6.4).
  *Test:* `test_shape_expr_vocabulary_eval`.
- **KISS-OPS-6.20-0003** — The shape-expression evaluator MUST resolve the `last`
  sentinel to `rank − 1` against the referenced operand's rank, and MUST reject a
  concrete `axis >= rank` (and `last` on a rank-0 operand) with a typed decline; `÷`
  MUST be **floor division** (quotient toward −∞), and a `÷` by zero MUST be a typed
  decline. An implementation MUST NOT round `÷` toward zero, and MUST NOT panic on an
  out-of-range axis or a zero divisor (a producer relying on exact division, e.g. an
  even head dim, owns that invariant). *Test:* `test_shape_expr_axis_and_floordiv`.
- **KISS-OPS-6.20-0004** — When a referenced operand extent is **symbolic /
  data-dependent** (not a concrete integer at evaluation time), the evaluator MUST
  resolve the expression to a **surfaced gap** — never a typed decline and never a
  panic — consistent with the standard's treatment of symbolic reduction extents and
  data-dependent lengths. The gap MUST propagate through arithmetic and through a
  whole-shape `SameAs` over a partially-symbolic operand, and a consumer surfaces it
  as an opaque-op / telemetry gap. *Test:* `test_shape_expr_symbolic_gap`.
- **KISS-OPS-6.20-0005** — A shape expression MUST serialize in the §6.19 canonical
  form: a one-byte **tag** (`0` reserved per §6.19-0006; `SameAs=0x01`, `Extent=0x02`,
  `Const=0x03`, `Param=0x04`, `Add=0x05`, `Sub=0x06`, `Mul=0x07`, `Div=0x08`; the
  reserved `Reduce=0x09` / `WithDim=0x0A` / `Dims=0x0B`), fixed-width little-endian
  fields (`operand` and `field` as `u8`, `axis` as a non-negative `u8` with `0xFF`
  reserved as the `last` sentinel above the `0..MAX_RANK-1` concrete range — a
  **distinct** single-axis `u8` sentinel chosen high in the spirit of the §6.19-0020
  trailing-axis sentinel, **not** byte-identical to that `u16` axis-set mask `0xFFFE`;
  `Const` as `i64` LE, §6.19-0007), and each child expression
  **definite-length-prefixed** with a `u16` LE
  byte length (§6.19-0010). The encoding MUST be byte-deterministic so a shape-bearing
  blob is hashable and byte-comparable under the shared canonicalization; an encoder
  MUST NOT place a name on the wire and MUST NOT emit a tag outside this set. *Test:*
  `test_shape_expr_serialization_golden`.
- **KISS-OPS-6.20-0006** — A reader MUST decode a shape-expression blob with a
  **typed decline, never a panic** (KISS-Conform §6.7): the reserved `0` tag, a
  reserved-but-unregistered tag, a blob shorter than its tag's schema, and trailing
  bytes after a complete expression MUST each raise a typed decline; a well-formed
  blob MUST round-trip (`decode(encode(x)) = x`). *Test:*
  `test_shape_expr_decode_declines`.
- **KISS-OPS-6.20-0007** — The primitive-floor shape rules MUST be: an **elementwise**
  op's output shape is `SameAs` its (broadcast) operand; a **`reduce`-family** op's
  output shape is the input shape with the `reduce_axes` (§6.19-0020) removed when
  `keepdim = false` or set to `1` when `keepdim = true` — derived from the op's
  semantics, not a free shape attr; and the only **irreducible** free cases — a
  broadcast target and a slice/iota offset — use `SameAs` and a `DimExpr`
  respectively. An implementation MUST NOT bake an absolute constant output shape
  where the shape derives from an operand extent. *Test:*
  `test_shape_expr_primitive_floor_rules`.
- **KISS-OPS-6.20-0008** — The shape oracle MUST cover the class where the output
  shape equals **no** operand's shape — the class it most exists to catch. A `gather`
  / `index_select` / `embedding` output shape MUST be the data operand's shape with
  the gathered `axis` replaced by the index operand's shape (`data[..axis] ++ index
  ++ data[axis+1..]`); a **contraction** (`matmul`) output shape MUST be its
  role-vector-derived shape (KISS-Classify §6.6-0016 M/N/K axis roles — leading batch
  dims, then `[M, N]` — carried as axis roles, not a `ShapeExpr`). A **`scatter`** output
  shape MUST equal its **`dest` operand's** whole shape (`SameAs(dest)`, §6.11-0016) — the
  one index-family op whose output does coincide with an operand, enumerated here so the
  index/scatter family is complete and no `scatter` is left without a pinned output shape.
  An implementation
  MUST NOT advertise `SameAs(operand)` for an op whose output rank/extents differ from
  that operand (e.g. a gather declaring `same_as(data)`). *Test:*
  `test_shape_expr_out_differs_from_operands`.

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
decomposition every claimed non-primitive op (§6.13–§6.14, including the §6.18 complex
family), (c) declines cleanly (never panics) on unrecognized ops (`u32` is an ordinary
dtype and is not declined), and (d) passes the KISS-Conform suite for KISS-Ops at that
version. The compute-fidelity (MathPrecision) attribute (§6.17) is advertised per kernel
and surfaced as a KISS-Contract guarantee; the complex ops carry the determinism classes
of §6.18-0014.

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
| KISS-OPS-6.2-0007 | `test_ops_u32_ordinary_dtype` |
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
| KISS-OPS-6.8-0001 | `test_ops_transcendental_declared_tier_is_gate` |
| KISS-OPS-6.8-0002 | `test_ops_transcendental_no_cross_lang_identity` |
| KISS-OPS-6.8-0003 | `test_ops_sqrt_correctly_rounded_or_ulp` |
| KISS-OPS-6.8-0004 | `test_ops_special_function_atoms` |
| KISS-OPS-6.8-0005 | `test_ops_atan2_class_is_ulp` |
| KISS-OPS-6.8-0006 | `test_ops_accuracy_tier_flat_v1` |
| KISS-OPS-6.9-0001 | `test_ops_atan2_quadrants` |
| KISS-OPS-6.9-0002 | `test_ops_copysign_raw_bit` |
| KISS-OPS-6.9-0003 | `test_ops_nextafter_own_lattice` |
| KISS-OPS-6.10-0001 | `test_ops_bitwise_integer_only` |
| KISS-OPS-6.10-0002 | `test_ops_bitwise_logic` |
| KISS-OPS-6.10-0003 | `test_ops_shift_arithmetic_vs_logical` |
| KISS-OPS-6.10-0004 | `test_ops_shift_out_of_range_target_defined` |
| KISS-OPS-6.10-0005 | `test_ops_popcount_clz_ctz` |
| KISS-OPS-6.10-0006 | `test_ops_narrow_int_promote_truncate_composition` |
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
| KISS-OPS-6.11-0011 | `test_ops_reduce_axes_four_categories` |
| KISS-OPS-6.11-0012 | `test_ops_index_operand_role` |
| KISS-OPS-6.11-0013 | `test_ops_index_axis_attribute` |
| KISS-OPS-6.11-0014 | `test_ops_sort_network_axis_attribute` |
| KISS-OPS-6.11-0015 | `test_ops_gather_skip_base_dynamic` |
| KISS-OPS-6.11-0016 | `test_ops_scatter_dest_operand` |
| KISS-OPS-6.11-0017 | `test_ops_scatter_updates_broadcast` |
| KISS-OPS-6.11-0018 | `test_ops_structural_empty_axis` |
| KISS-OPS-6.11-0019 | `test_ops_sort_index_output_i64` |
| KISS-OPS-6.12-0001 | `test_ops_scalar_source_leaves` |
| KISS-OPS-6.12-0002 | `test_ops_const_leaf_bits` |
| KISS-OPS-6.12-0003 | `test_ops_named_constant_bits` |
| KISS-OPS-6.13-0001 | `test_ops_reference_decompositions` |
| KISS-OPS-6.13-0002 | `test_ops_decomposition_strictly_lower_level` |
| KISS-OPS-6.13-0003 | `test_ops_decomposition_accuracy_refinement` |
| KISS-OPS-6.13-0004 | `test_ops_parameterized_attributes_explicit` |
| KISS-OPS-6.13-0005 | `test_ops_pow_full_domain` |
| KISS-OPS-6.13-0006 | `test_ops_decomposition_body_grammar` |
| KISS-OPS-6.13-0007 | `test_ops_hypot_inf_nan` |
| KISS-OPS-6.13-0008 | `test_ops_norm_axis_all_four` |
| KISS-OPS-6.13-0009 | `test_ops_structured_decomposition_reference` |
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
| KISS-OPS-6.16-0007 | `test_ops_complex_storage_layout` |
| KISS-OPS-6.16-0008 | `test_ops_dtype_layout_coversioned` |
| KISS-OPS-6.17-0001 | `test_ops_math_precision_enum` |
| KISS-OPS-6.17-0002 | `test_ops_math_precision_bit_stable` |
| KISS-OPS-6.17-0003 | `test_ops_math_precision_reduced` |
| KISS-OPS-6.17-0004 | `test_ops_math_precision_not_dtype` |
| KISS-OPS-6.17-0005 | `test_ops_math_precision_order_invariant_scope` |
| KISS-OPS-6.17-0006 | `test_ops_math_precision_input_rounding` |
| KISS-OPS-6.17-0007 | `test_ops_math_precision_reproducibility_class` |
| KISS-OPS-6.18-0001 | `test_ops_complex_op_set` |
| KISS-OPS-6.18-0002 | `test_ops_complex_no_new_primitive` |
| KISS-OPS-6.18-0003 | `test_ops_complex_component_bridge` |
| KISS-OPS-6.18-0004 | `test_ops_complex_add_sub_neg_conj` |
| KISS-OPS-6.18-0005 | `test_ops_cmul_annexg` |
| KISS-OPS-6.18-0006 | `test_ops_cdiv_annexg` |
| KISS-OPS-6.18-0007 | `test_ops_cabs_annexg` |
| KISS-OPS-6.18-0008 | `test_ops_carg_principal` |
| KISS-OPS-6.18-0009 | `test_ops_cexp_annexg` |
| KISS-OPS-6.18-0010 | `test_ops_clog_principal_branch` |
| KISS-OPS-6.18-0011 | `test_ops_csqrt_principal_branch` |
| KISS-OPS-6.18-0012 | `test_ops_cpow_principal` |
| KISS-OPS-6.18-0013 | `test_ops_complex_nan_inf_annexg` |
| KISS-OPS-6.18-0014 | `test_ops_complex_determinism_class` |
| KISS-OPS-6.18-0015 | `test_ops_complex_component_dtype` |
| KISS-OPS-6.18-0016 | `test_ops_complex_advertised_high_level` |
| KISS-OPS-6.18-0017 | `test_ops_complex_branch_sign_exact` |
| KISS-OPS-6.19-0001 | `test_ops_opattrs_channel_concept` |
| KISS-OPS-6.19-0002 | `test_ops_opattrs_distinct_from_pattern_attrs` |
| KISS-OPS-6.19-0003 | `test_ops_opattrs_carrier_set_closed` |
| KISS-OPS-6.19-0004 | `test_ops_opattrs_field_order_abi` |
| KISS-OPS-6.19-0005 | `test_ops_opattrs_explicit_default_resolution` |
| KISS-OPS-6.19-0006 | `test_ops_opattrs_enum_ordinal_reserve_zero` |
| KISS-OPS-6.19-0007 | `test_ops_opattrs_int_fixed_width_le` |
| KISS-OPS-6.19-0008 | `test_ops_opattrs_little_endian` |
| KISS-OPS-6.19-0009 | `test_ops_opattrs_optional_explicit_slot` |
| KISS-OPS-6.19-0010 | `test_ops_opattrs_definite_length_prefix` |
| KISS-OPS-6.19-0011 | `test_ops_opattrs_version_binding` |
| KISS-OPS-6.19-0012 | `test_ops_opattrs_opaque_embedding_byte_compare` |
| KISS-OPS-6.19-0013 | `test_ops_opattrs_golden_vector_conformance` |
| KISS-OPS-6.19-0014 | `test_ops_opattrs_monoid_enum` |
| KISS-OPS-6.19-0015 | `test_ops_opattrs_oob_policy_enum` |
| KISS-OPS-6.19-0016 | `test_ops_opattrs_scatter_combine_enum` |
| KISS-OPS-6.19-0017 | `test_ops_opattrs_index_dtype_enum` |
| KISS-OPS-6.19-0018 | `test_ops_opattrs_sort_direction_enum` |
| KISS-OPS-6.19-0019 | `test_ops_opattrs_scan_exclusivity_enum` |
| KISS-OPS-6.19-0020 | `test_ops_opattrs_reduce_axes_multiplex` |
| KISS-OPS-6.19-0021 | `test_ops_opattrs_column_ordering_enum` |
| KISS-OPS-6.19-0022 | `test_ops_opattrs_boolean_flags` |
| KISS-OPS-6.19-0023 | `test_ops_opattrs_window_param_vector` |
| KISS-OPS-6.19-0024 | `test_ops_opattrs_permutation_reserved` |
| KISS-OPS-6.19-0025 | `test_ops_opattrs_reduce_schema` |
| KISS-OPS-6.19-0026 | `test_ops_opattrs_prefix_scan_schema` |
| KISS-OPS-6.19-0027 | `test_ops_opattrs_gather_schema` |
| KISS-OPS-6.19-0028 | `test_ops_opattrs_scatter_schema` |
| KISS-OPS-6.19-0029 | `test_ops_opattrs_sort_network_schema` |
| KISS-OPS-6.19-0030 | `test_ops_opattrs_reduce_var_std_schema` |
| KISS-OPS-6.19-0031 | `test_ops_opattrs_norm_axis_schema` |
| KISS-OPS-6.19-0032 | `test_ops_opattrs_pool_schema` |
| KISS-OPS-6.19-0033 | `test_ops_opattrs_im2col_schema` |
| KISS-OPS-6.19-0034 | `test_ops_opattrs_gather_scatter_wrapper_schema` |
| KISS-OPS-6.19-0035 | `test_ops_opattrs_reduce_axes_classify_reconciliation` |
| KISS-OPS-6.19-0036 | `test_ops_opattrs_noncarrier_axis_resolution` |
| KISS-OPS-6.19-0037 | `test_ops_opattrs_max_rank_operands_pinned` |
| KISS-OPS-6.19-0038 | `test_ops_opattrs_reduce_axes_zero_unreachable` |
| KISS-OPS-6.19-0039 | `test_ops_index_ref_wire_form` |
| KISS-OPS-6.19-0040 | `test_ops_index_outputs_root_list` |
| KISS-OPS-6.20-0001 | `test_shape_rule_exists_and_matches` |
| KISS-OPS-6.20-0002 | `test_shape_expr_vocabulary_eval` |
| KISS-OPS-6.20-0003 | `test_shape_expr_axis_and_floordiv` |
| KISS-OPS-6.20-0004 | `test_shape_expr_symbolic_gap` |
| KISS-OPS-6.20-0005 | `test_shape_expr_serialization_golden` |
| KISS-OPS-6.20-0006 | `test_shape_expr_decode_declines` |
| KISS-OPS-6.20-0007 | `test_shape_expr_primitive_floor_rules` |
| KISS-OPS-6.20-0008 | `test_shape_expr_out_differs_from_operands` |
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
`cmp_lt(-0.0, 0)` is false (−0.0 is not less than 0 under §6.6 signed-zero equality), so
the result is the raw-bit `-0.0`, preserved. The same signed-zero equality drives the
minmax ties: the four §6.13 minmax decompositions share the identical innermost select —
`cmp_ge(a,b) → a` for `max_prop`/`fmax_ieee`, `cmp_le(a,b) → a` for
`min_prop`/`fmin_ieee` — and differ only in their NaN arms, so on a `±0` tie, where
`cmp_ge` and `cmp_le` are both true, **operand `a` wins in all four ops**; no op in the
minmax family is b-biased on a tie. In particular `max_prop(-0.0, const(0))` returns
`-0.0` (the normative decomposition keeps `a` via `cmp_ge`), matching `relu(-0.0)` — and
under the normative decompositions `relu(x)` is pointwise-identical to a conformant
`max_prop(x, const(0))`: both propagate a NaN input raw (`relu` moves it through the
`select` else arm; `max_prop` keeps the NaN operand) and both keep `x` on a `±0` tie.
Argument order matters: `max_prop(const(0), x)` differs at `-0.0`, where
`cmp_ge(0, -0.0)` is true and yields `+0.0`. The separations are therefore precisely
these: `relu` versus `max_prop(x, const(0))` is a **decomposition-shape** separation — a
distinct pinned op token for classification, fusion identity, and lineage, not a
pointwise-semantics difference — while `relu` versus `fmax_ieee(x, const(0))` is a
**semantic** separation in the NaN arm (`fmax_ieee(NaN, 0)` scrubs the NaN to `0` where
`relu(NaN)` stays NaN through the `select` else arm). That is why `relu` is pinned to
`select` rather than lowered to whichever `max` a backend happens to supply. The `±0`
tie behavior of all four minmax ops is pinned bit-for-bit by the §6.13 signed-zero tie
conformance vectors (raw-bit comparison — a value compare of `0.0 == -0.0` passes
vacuously).

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
  `reduced(stage)`, `extent(axis)`, `reduced_count` (§6.12).
- **Compute-fidelity (MathPrecision) attribute** — the `{bit-stable,
  reduced-mantissa-permitted}` enum owned by KISS-Ops (§6.17), orthogonal to the determinism
  class; the home of compute precision now that storage dtypes are pure byte layout.
- **Complex op family** — the §6.18 ops over `c32`/`c64` (`cadd`, `csub`, `cneg`, `cconj`,
  `cmul`, `cdiv`, `cabs`, `carg`, `cexp`, `clog`, `csqrt`, `cpow`, plus the bridge ops
  `cmake`/`cre`/`cim`), all non-primitive over the real floor, pinned to ISO C99/C11
  Annex G.
- **Index operand role** — the `index_operand` + `index_dtype` operand-level role
  (§6.11-0012); index-only-ness is this role, not a dtype class.

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

1. **`f32` versus `f32s` (resolved 2026-07-12).** Ratified: strict-precision float is
   **not** a dtype. `f32s` is removed from the dtype set; the Classify dtype set is pure
   storage (byte layout only). Compute precision moves to KISS-Ops as the compute-fidelity
   (MathPrecision) attribute `{bit-stable, reduced-mantissa-permitted}` (§6.17), surfaced in
   a kernel's KISS-Contract guarantees. `f32` is now pure binary32 storage with no
   compute-precision meaning (§6.16-0002). The **reduced-mantissa floor** of 10 explicit
   mantissa bits (§6.17-0003) and the per-op-vs-per-kernel granularity and
   order-invariant-op testability rules (§6.17-0005) are the quantified replacement for the
   per-operand precision the retired `f32s` dtype carried; the exact floor value (or an
   enumerated permitted-format set) is a reference value pending confirmation against real
   reduced-precision hardware, but the clause pins a concrete number so the attribute is not
   an unquantified adjective.
2. **`u32` index-only (resolved 2026-07-12).** Ratified: index-only-ness is an **operand
   role**, not a dtype class. `u32` is an ordinary unsigned-integer storage/compute dtype
   (§6.2-0007); the index role is carried on the gather/scatter operand via `index_operand`
   and `index_dtype` (§6.11-0012), with the legal `index_dtype` set `{u32, i32, i64}`
   (§6.11-0009). No index-only dtype class is defined.
3. **Sub-byte packing (`s4`, `u4`, `b1`).** KISS-Ops restates the packing layout in §6.16
   informatively because it is adjacent to the `popcount`/binary-GEMM semantics, but
   §6.16-0006 now defers **normative** ownership of the sub-byte storage packing to the
   sibling data-vocabulary sub-standard (§6.1-0008/0009), keeping only the `b1`
   xor+popcount→raw-`s32` accumulation as the Ops-owned computation fact. This resolves the
   earlier dual-pinning so exactly one standard carries the normative packing clause;
   whether the two spellings should be kept byte-identical (shared anchor) remains an RFC
   item, but the ownership boundary is no longer ambiguous.
4. **Complex dtypes (`c32`, `c64`) (resolved 2026-07-12).** Ratified: `c32`/`c64` **do**
   get a complex-arithmetic op family (§6.18) — `cadd`/`csub`/`cmul`/`cdiv`/`cneg`/`cconj`/
   `cabs`/`carg`, complex-construct/`re`/`im` extract (`cmake`/`cre`/`cim`), and the
   transcendentals `cexp`/`clog`/`csqrt`/`cpow` — pinned to ISO C99/C11 Annex G (principal
   branch cuts, signed zero, NaN/inf recovery). Every complex op is non-primitive with a
   reference decomposition into the REAL floor atoms, so the primitive floor is unchanged
   (no new axiom, §6.18-0002); `c32`/`c64` remain interleaved (re,im) storage containers.
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
7. **`extent(axis)` and `reduced_count` leaves.** `extent(axis)` (single-axis length) and
   `reduced_count` (the product of extents over **all** reduced axes) are in the
   scalar-source leaf set (§6.12-0001); `reduced_count` is the correct divisor for a mean
   over one **or more** reduced axes, so `reduce_mean` / `reduce_var` / `reduce_std` are
   arithmetically correct under the multi-axis `reduce_axes` masks enabled by §6.11-0011.
   Whether the shared anchor's leaf list should adopt these identically is an RFC item for
   the data-vocabulary sibling.
8. **`reduce` keepdim broadcast + `reduce_axes` token (resolved 2026-07-12).**
   §6.11-0008 pins reduced axes as extent-1 / stride-0 keepdim views. The prior single
   overloaded empty-mask sentinel is **split** into three explicit, distinctly-encoded
   category tokens — **all-axes** (`rall`), **trailing-axis** (`rlast`), and **none /
   not-a-reduction** (`-`) — with an explicit per-axis subset mask `x<hh>` (a u8 keepdim
   mask, two lowercase hex digits) (§6.11-0011). The `reduce_axes` **reduce-field encoding**
   is owned normatively by the data-vocabulary `structure_key` **token codec** (Classify
   §6.7-0005, which bumps `STRUCTURE_KEY_VERSION` for this split, still unfrozen); **no
   binary/byte wire form is defined at this schema version** (Classify §6.7-0011). KISS-Ops
   restates those tokens **informatively** (as the shared anchor, not a second normative
   pinning — mirroring the D.3 sub-byte-packing resolution) and pins only the reduce/scan
   **semantics** of each value plus the emission constraints. The u8 per-axis subset field
   requires `MAX_RANK <= 8`, the category tokens stay disjoint from every legal subset mask,
   and any hex sentinel rendering (`0xFFFF`/`0xFFFE`/`0x0000`, reserved `0x0100..0xFFFD`) is
   an Ops-local in-memory illustration only, not the shared anchor and not a wire form;
   whether the two spellings should be kept identical remains an RFC item, but the ownership
   boundary is unambiguous.
9. **Determinism-class enum ownership.** This version makes KISS-Ops the owner of the
   single canonical enum `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`
   (§6.0-0001), imported downstream by KISS-Synth and KISS-Conform, resolving the prior
   upstream-import cycle.
10. **Umbrella cross-reference (reconciled 2026-07-12).** Three corrections to the umbrella,
    now applied: (a) the umbrella §2.1 description of KISS-Ops reads "IEEE-fmax versus
    NaN-propagating-max" (matching §2.3/§6.15 here), not "saturating-max"; (b) the
    umbrella §2.1 now names **KISS-Ops** as owner of the canonical determinism/fidelity enum
    `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}` (§6.0-0001, Appendix D.9),
    with KISS-Synth, KISS-Emit, and KISS-Conform importing it — placing ownership in a
    foundational vocabulary so no lower tier imports upward from a protocol-tier sub-standard;
    and (c) the enum is now spelled **byte-identically** in both documents — the earlier
    umbrella renderings `ULP-tolerance` (§2.1) and the truncated `order-invariant` (§2.1/§3
    "determinism class declared" bullet) were corrected to the canonical
    `{exact-byte, ULP/tolerance, order-invariant/nondeterministic}`, so the §6.0-0001
    "verbatim everywhere" invariant and byte-exact token matching (§7.3-0003, §7.4-0001) now
    hold across the suite.
11. **OpAttrs channel + canonical wire encoding (added 2026-07-13).** §6.19 adds the
    per-op, compile-time **OpAttrs** record and its canonical, default-resolved little-
    endian wire encoding (explicit default-resolution — no elision; per-op fixed field
    order as ABI; frozen reserve-`0` little-endian enum ordinals; fixed-width LE two's-
    complement integers; explicit optional slots; definite length-prefixes; version
    binding), discharging the KISS-Grammar §6.2-0006 / §8-0008 and KISS-Contract Semantics
    seam obligations. Several sub-questions remain open (recorded for the RFC, not binding):
    (a) **`sort_network` axis (covered 2026-07-13).** §6.11-0014 now normatively pins an
    explicit `axis` attribute (role, `u8` width, range `0..MAX_RANK-1`) resolving to the
    trailing (innermost) axis `r-1`, cited by the §6.19-0029 schema; no longer resolved only
    in this open-question list. (b) **`permutation` encoding** — frozen and reserved (Grammar §6.2-0006
    requires KISS-Ops to own it) but attached to NO op this version, since `sort_network`
    emits its permutation as a runtime index-vector output; confirm it is reserved for
    future shape/transpose/permute ops. (c) **`im2col` `output_column_ordering`** — the
    provisional `{1=channel-major, 2=tap-major}` member set is pending normative pinning.
    (d) **padding shape** — the per-axis `padding` is modeled as a single symmetric `u32`
    per axis; confirm no op needs asymmetric low/high pairs. (e) **`matmul`** — modeled as
    carrying NO free OpAttrs (M/N/K fixed by the §6.13 operand-ordering convention +
    Classify role hints); confirm it needs no free contract-axis field. (f) **`monoid` as
    OpAttrs vs op identity** — treated as an identity-bearing mandatory OpAttrs field;
    confirm KISS-Ops does not instead fold it into distinct op tokens. (g)
    **`norm_axis` for `rms_norm`/`layer_norm` (covered 2026-07-13).** §6.13-0008 now
    normatively requires an explicit `norm_axis` (same role, width, and range as the
    §6.13-0004 `softmax`/`log_softmax` normalization axis) for `rms_norm` and `layer_norm`,
    cited by the §6.19-0031 schema; no longer resolved only in this open-question list. The
    indexed `axis` of `gather`/`scatter`/`index_select`/`embedding`/`scatter_add` is
    likewise now pinned by §6.11-0013. (h)
    **fixed-constant flags** — `keepdim` (§6.11-0008) and `stability` (§6.11-0007) are
    pinned to `1` and emitted as explicit slots (R1 favors retaining for additive-friendly
    growth); confirm the design intent to keep them rather than drop for a smaller blob.
    (i) **single-axis `norm_axis`** — modeled as one `u8`; confirm `softmax`/`log_softmax`
    never normalize over multiple axes this version (which would need a `reduce_axes`-style
    `u16`).

---

## Appendix E — OpAttrs golden vectors (informative)

These worked vectors render OpAttrs values to exact little-endian hex bytes ("bytes on
the wire, left to right"). They are informative; the normative encoding is §6.19. Each
cites its pinning clause. A hex pair is one byte; `··` groups a multi-byte field for
readability only.

**E.1 Sub-vocabulary single-field vectors.**

| Sub-vocabulary (clause) | Value | Bytes (LE hex) |
|---|---|---|
| `monoid` (§6.19-0014) | sum / prod / max / min | `01` / `02` / `03` / `04` |
| `oob_policy` (§6.19-0015) | skip / clamp / zero-fill | `01` / `02` / `03` |
| `scatter_combine` (§6.19-0016) | assign / atomic-add / atomic-max / atomic-min | `01` / `02` / `03` / `04` |
| `index_dtype` (§6.19-0017) | u32 / i32 / i64 | `01` / `02` / `03` |
| `sort_direction` (§6.19-0018) | ascending / descending | `01` / `02` |
| `scan_exclusivity` (§6.19-0019) | inclusive / exclusive | `01` / `02` |
| `column_ordering` (§6.19-0021) | channel-major / tap-major | `01` / `02` |
| `boolean-flag` (§6.19-0022) | false / true | `00` / `01` |
| `reduce_axes` (§6.19-0020) | none¹ / all-axes / trailing (rank>1) / subset{axis0} (rank>1, non-trailing) / subset{axis0,axis2} | `00 00`¹ / `FF FF` / `FE FF` / `01 00` / `05 00` |
| `window-param-vector` (§6.19-0023) | `[3, 3]` (element 0 = axis 0, element 1 = axis 1) | `02·03 00 00 00·03 00 00 00` |
| `permutation` RESERVED (§6.19-0024) | `[2, 0, 1]` (illustrative; emitted on no op this version) | `03·02·00·01` |

**Rank-sensitivity of `reduce_axes`** (§6.19-0020): the `subset{axis0}` → `01 00` and
`trailing` → `FE FF` rows above assume rank > 1. Over a **rank-1** operand the sole axis
is all-axes, so a `reduce` MUST encode `FF FF`, never the single-bit mask `01 00`; over a
**rank-3** operand a reduction covering `{0,1,2}` MUST encode `FF FF`, never the subset
mask `07 00` (§6.19-0020 total precedence). ¹ `0x0000` (none) is a sub-vocabulary code
point shown for completeness but is **unreachable** in any carrier OpAttrs blob
(§6.19-0038).

**E.2 Full per-op OpAttrs blobs.**

- **`reduce(sum, all-axes, keepdim)`** (§6.19-0025): `monoid`=sum `01`, `reduce_axes`=rall
  `FF FF`, `keepdim`=true `01` → **`01 FF FF 01`** (4 bytes). Framed by KISS-Grammar
  §6.8-0007 as `u16` LE length + blob → **`04 00 01 FF FF 01`**.
- **`gather(axis=0, oob=clamp, index_operand=1, index_dtype=i32)`** (§6.19-0027; the
  KISS-Grammar §2.4 clamp worked example, clamp a non-default value): `axis` `00`,
  `oob_policy`=clamp `02`, `index_operand` `01`, `index_dtype`=i32 `02` →
  **`00 02 01 02`** (4 bytes).
- **`prefix_scan(sum, trailing-axis, inclusive)` over a rank>1 operand** (§6.19-0026):
  `monoid`=sum `01`, `reduce_axes`=rlast `FE FF` (trailing axis of a rank>1 operand;
  §6.19-0020 scan precedence), `exclusivity`=inclusive `01` (the `scan_exclusivity` enum
  ordinal, NOT a boolean) → **`01 FE FF 01`** (4 bytes).
- **`scatter(axis=0, combine=atomic-add, oob=skip, index_operand=1, index_dtype=i64)`**
  (§6.19-0028): `axis` `00`, `combine`=atomic-add `02`, `oob_policy`=skip `01`,
  `index_operand` `01`, `index_dtype`=i64 `03` → **`00 02 01 01 03`** (5 bytes).
- **`reduce_var(all-axes, keepdim, population)`** (§6.19-0030): `reduce_axes`=rall
  `FF FF`, `keepdim`=true `01`, `bessel_correction`=false `00` → **`FF FF 01 00`**
  (4 bytes).
- **`reduce(sum)` over a rank-1 operand's sole axis** (§6.19-0020/-0025): the sole axis is
  all-axes, so `reduce_axes`=`FF FF` (NOT the single-bit subset mask `01 00`), `monoid`=sum
  `01`, `keepdim`=true `01` → **`01 FF FF 01`** (4 bytes).
- **`reduce(max)` over a rank-3 operand's axes {0,1,2}** (§6.19-0020/-0025): the selected
  set covers all three axes, so `reduce_axes`=`FF FF` (NOT the subset mask `07 00`),
  `monoid`=max `03`, `keepdim`=true `01` → **`03 FF FF 01`** (4 bytes).
- **`sort_network(ascending, stable)` over a rank-2 operand** (§6.19-0029): `axis`=trailing
  `01`, `direction`=ascending `01`, `stability`=stable `01` → **`01 01 01`** (3 bytes).
- **`softmax(norm_axis=1)`** (§6.19-0031): `norm_axis` `01` → **`01`** (1 byte).
- **`index_select(axis=0, index_operand=1, index_dtype=u32)`** (§6.19-0034): `axis` `00`,
  `index_operand` `01`, `index_dtype`=u32 `01` → **`00 01 01`** (3 bytes).
- **`avg_pool` window=`[2,2]`, stride=`[2,2]`, dilation=`[1,1]`, padding=`[0,0]`,
  count_include_pad=true** (§6.19-0032): `window_size` `02·02 00 00 00·02 00 00 00`;
  `stride` `02·02 00 00 00·02 00 00 00`; `dilation` `02·01 00 00 00·01 00 00 00`;
  `padding` `02·00 00 00 00·00 00 00 00`; `count_include_pad` `01` →
  **`02 02 00 00 00 02 00 00 00 02 02 00 00 00 02 00 00 00 02 01 00 00 00 01 00 00 00 02 00 00 00 00 00 00 00 00 01`**
  (37 bytes).
- **`avg_pool` window=`[3,5]`, stride=`[1,2]`, dilation=`[1,1]`, padding=`[0,0]`,
  count_include_pad=true** (§6.19-0032; **non-square**, exercising the §6.19-0023
  ascending-spatial-axis order — element 0 = axis 0 = `3`/`1`, element 1 = axis 1 = `5`/`2`
  — so a reversed encoder that emitted `[5,3]`/`[2,1]` would diverge here): `window_size`
  `02·03 00 00 00·05 00 00 00`; `stride` `02·01 00 00 00·02 00 00 00`; `dilation`
  `02·01 00 00 00·01 00 00 00`; `padding` `02·00 00 00 00·00 00 00 00`;
  `count_include_pad` `01` →
  **`02 03 00 00 00 05 00 00 00 02 01 00 00 00 02 00 00 00 02 01 00 00 00 01 00 00 00 02 00 00 00 00 00 00 00 00 01`**
  (37 bytes).
- **`im2col` window=`[3,3]`, stride=`[1,1]`, dilation=`[1,1]`, padding=`[1,1]`,
  output_column_ordering=channel-major** (§6.19-0033): `window_size`
  `02·03 00 00 00·03 00 00 00`; `stride` `02·01 00 00 00·01 00 00 00`; `dilation`
  `02·01 00 00 00·01 00 00 00`; `padding` `02·01 00 00 00·01 00 00 00`;
  `output_column_ordering`=channel-major `01` →
  **`02 03 00 00 00 03 00 00 00 02 01 00 00 00 01 00 00 00 02 01 00 00 00 01 00 00 00 02 01 00 00 00 01 00 00 00 01`**
  (37 bytes).

**E.3 Default-resolution equality.** A `gather` written with the default `oob_policy`
(skip) and one written explicitly `oob_policy=skip`, all other fields equal (`axis=0`,
`index_operand=1`, `index_dtype=u32`), both emit **`00 01 01 01`** — identical bytes
(§6.19-0005), which is what lets KISS-Grammar byte-compare the blob without interpreting
it (§6.19-0012).

---

*End of KISS-Ops (Draft proposal). §0–§5 are informative; §6+ are normative. Every binding
requirement is an identified clause with a mapped KISS-Conform test. KISS-Ops depends on no
other sub-standard; it is the foundational computation vocabulary, consumed structurally by
KISS-Grammar, KISS-Contract, KISS-Synth/Provision, KISS-Consume, and KISS-Emit. Project and
product names appear only in non-normative examples, provenance, and reference-implementation
pointers; normative clauses use only the generic roles provider, consumer, implementation,
kernel, contract, target, and steward.*
