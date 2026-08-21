# Proposal — the op→family mapping (#263)

**Status: PROPOSAL. This introduces no clause, no §9 row, and no coverage-floor movement,**
and it is deliberately not drafted as normative text. The reason is the finding itself:
`op_family_tag` is a `structure_key` cell field, and `structure_key` is the admissibility
key — so **pinning a mapping is not an additive edit.** If any deriver already assigns
differently from what is pinned here, its tokens change, and kernels that are admissible
today stop being admissible. A mapping may be pinned only once the derivers are known to
agree; this document exists to find out whether they do.

**And the majority of it cannot be falsified by anything in the repository.** Of the 24
family codes, **5 are exercised by a positive vector** — `bin`, `une`, `red`, `gem`, `scn`
— across the 21 positive vectors in `conformance/corpus/structure_key_vectors.json`. The
other 19 have no vector behind them. A mapping over 24 families where 19 are unexercised
**is a proposal about intent**, and every reader should treat the unexercised rows as
stated intent rather than measured behaviour.

---

## 1. The population, corrected

Measured from `spec/ops.md` at `origin/main`, from the document's own text.

| Tier | Source | Ops |
|---|---|---|
| primitive floor | §2.7 primitive table / §6.3 | 43 |
| non-primitive | §2.7 non-primitive table / §6.13 | 63 |
| complex-arithmetic | §2.7 complex sentence / §6.18 | 15 |
| **total** | **§6.1-0001's op set** | **121** |

**The figure `106` in circulation is real and is from ops.md's own text — but it is
`43 + 63`, the op set *minus* the 15 complex ops.** §6.1-0001 defines the op set as
§6.3 ∪ §6.13 ∪ **§6.18**, so a sweep scoped to 106 silently omits the entire complex
family. This mapping covers all 121.

**A second correction, to #263's own measurement.** #263 reports that ops.md carries a
*nine-value* op category vocabulary, listing `shape 69 · transcendental 67 · reduction 60`
and so on. **Those are occurrence counts of nine words across the document, not op counts,
and the actual §2.7 family-tag set has 18 values.** The nine it misses —
`access-primitive`, `arithmetic`, `binary_math`, `bitwise`, `comparison`, `logical`,
`minmax`, `rounding`, `select` — carry **51 of the 106** ops. The issue's central claim
(two vocabularies that do not correspond) is unaffected and correct; its measurement of
the near side is not.

## 2. Why this is a re-partition, not a relabeling

The two vocabularies do not disagree about where an op goes. **They partition on different
axes.**

- **ops.md classifies by what an op *means*:** `arithmetic`, `transcendental`, `bitwise`,
  `rounding`, `comparison`, `minmax`, `logical`, `binary_math`.
- **Classify classifies by what a *kernel* looks like:** `une`/`bin`/`ter` are **arity**;
  `red`/`scn`/`gem`/`idx`/`srt`/`pol` are **access pattern**.

So `add` (arithmetic), `atan2` (binary_math), and `bit_and` (bitwise) — three different
ops.md families — are all one Classify family, `bin`, because all three are binary
elementwise. **No 18→24 table between the family names can exist**; the mapping is
necessarily per-op.

## 3. The discriminator Classify needs is not a field ops.md records

`une`/`bin`/`ter` are decided by **arity**, and **ops.md never states arity.** It is
recoverable, but from several different places, and for some ops not at all:

| Where arity is recoverable from | Ops |
|---|---|
| a formula in a semantics table (`a + b`, `-x`) or a clause body (§6.9-0001's `(y=a, x=b)`) | 98 |
| a prose operand-ordering paragraph (§6.13 preamble: `input(0)=x, input(1)=gamma`) | 6 |
| **nowhere — prose only** (`ceil` “round toward +∞”), inferred from meaning | 11 |
| an access-primitive, assigned by op identity rather than arity | 6 |
| **total** | **121** |

Two of those rows deserve attention. The `§6.9` case was found only because an extractor
gated on a positive control **failed** on `atan2` — its arity is in a clause body, not a
table, so a table-only sweep silently reports it as unknown. And `element_map` is the sharp
case: its Classify family depends on the **arity of the body it carries**, which is not a
property of the op at all. **It cannot be assigned a fixed code.**

## 4. Coverage of the 24-code alphabet

| Class | Codes | |
|---|---|---|
| **firm** — reached by an uncontested assignment | 8 | `bin`, `idx`, `nrm`, `red`, `scn`, `srt`, `ter`, `une` |
| **contested-only** — reachable only if a contested pair rules toward it | 8 | `cnv`, `emb`, `gat`, `gem`, `lin`, `pol`, `sft`, `shp` |
| **no op under any reading** | 8 | `qnt`, `rnd`, `los`, `seg`, `img`, `fft`, `att`, `moe` |

**Eight of the twenty-four codes have no KISS-Ops op at all** — `qnt`, `rnd`, `los`, `seg`, `img`, `fft`, `att`, `moe`.
There is no quantize op, no loss op, no attention op, no FFT, no random, no segment, no
image, and no mixture-of-experts op in this version's set. This is **not** a defect in the
alphabet: per #263's own argument the family tag records **what a cell carries on the wire,
not what ops an implementation has**, and a producer that fuses may legitimately emit an
`att` or `moe` cell. But it does mean **a third of the alphabet is unpopulated by the op
set**, and no mapping from KISS-Ops can ever make those codes reachable.

**The inversion worth reading twice:** `gem` is the **most-exercised** family in the corpus
— 8 of the 21 positive vectors — and it is **contested-only**. Its sole op, `matmul`, is
not assigned to it by any normative text. **The corpus has already committed to one arm of
an undecided question, in the family it tests hardest.** Whatever the architect rules for
`gem`/`lin`, that ruling is not free: it either ratifies 8 existing vectors or invalidates
them.

## 5. The contested pairs — explicitly not decided here

**These are the architect's calls, not this document's.** Each is a case where two codes
both apply and no normative text picks one.

| Pair | Ops | Why both apply |
|---|---|---|
| `nrm` / `sft` | `softmax`, `log_softmax` | ops.md files both under `normalization`; Classify has a dedicated `sft` |
| `idx` / `emb` | `index_select`, `embedding` | ops.md files both under `gather_scatter`; Classify separates them (**#263's own example**) |
| `une` / `gat` | `silu`, `gelu`, `gelu_tanh`, `mish` | each is an `x·f(x)` gate, and each is also plain elementwise-unary |
| `pol` / `cnv` | `avg_pool`, `max_pool` | pooling by name; convolution-family by access pattern |
| `gem` / `lin` | `matmul` | `gem` is glossed “contraction (dense GEMM)” and ops.md's family is `contraction`, so the textual pull is strong; `lin` (linalg) could subsume it |
| `shp` / `cnv` | `im2col` | ops.md files it under `shape`; it exists to lower convolution |

Twelve of the 121 ops sit on one of these six pairs. The arms are **not** equally balanced
— `gem`/`lin` has a clear textual default and `une`/`gat` does not — but this document
records the contest rather than resolving it, because resolving it changes tokens.

## 6. The mapping

`basis` records **how the assignment was reached**, so a reader can tell a lookup from an
inference: `formula` = arity read off a formula; `prose` = arity from an operand-ordering
paragraph; `unstated` = **arity nowhere stated, inferred from the op's meaning**;
`op identity` = an access-primitive assigned directly.

| Op | Tier | ops.md family | Arity | → code | Basis |
|---|---|---|---|---|---|
| `cabs` | complex | complex | 2 | **bin** | formula |
| `cadd` | complex | complex | 2 | **bin** | formula |
| `carg` | complex | complex | 2 | **bin** | formula |
| `cconj` | complex | complex | 2 | **bin** | formula |
| `cdiv` | complex | complex | 2 | **bin** | formula |
| `cexp` | complex | complex | 2 | **bin** | formula |
| `cim` | complex | complex | 1 | **une** | unstated |
| `clog` | complex | complex | 1 | **une** | unstated |
| `cmake` | complex | complex | 1 | **une** | formula |
| `cmul` | complex | complex | 2 | **bin** | formula |
| `cneg` | complex | complex | 2 | **bin** | formula |
| `cpow` | complex | complex | 2 | **bin** | unstated |
| `cre` | complex | complex | 1 | **une** | unstated |
| `csqrt` | complex | complex | 2 | **bin** | formula |
| `csub` | complex | complex | 2 | **bin** | formula |
| `gelu` | non-primitive | activation | 1 | `une/gat` ⚠ | CONTESTED |
| `gelu_tanh` | non-primitive | activation | 1 | `une/gat` ⚠ | CONTESTED |
| `mish` | non-primitive | activation | 1 | `une/gat` ⚠ | CONTESTED |
| `relu` | non-primitive | activation | 1 | **une** | formula |
| `sigmoid` | non-primitive | activation | 1 | **une** | formula |
| `silu` | non-primitive | activation | 1 | `une/gat` ⚠ | CONTESTED |
| `softplus` | non-primitive | activation | 1 | **une** | formula |
| `step` | non-primitive | activation | 1 | **une** | formula |
| `recip` | non-primitive | arithmetic | 1 | **une** | formula |
| `sign` | non-primitive | arithmetic | 1 | **une** | formula |
| `sqr` | non-primitive | arithmetic | 1 | **une** | formula |
| `hypot` | non-primitive | binary_math | 2 | **bin** | formula |
| `ldexp` | non-primitive | binary_math | 2 | **bin** | formula |
| `pow` | non-primitive | binary_math | 2 | **bin** | formula |
| `rem_floor` | non-primitive | binary_math | 2 | **bin** | formula |
| `rem_trunc` | non-primitive | binary_math | 2 | **bin** | formula |
| `matmul` | non-primitive | contraction | 2 | `gem/lin` ⚠ | CONTESTED |
| `embedding` | non-primitive | gather_scatter | 1 | `idx/emb` ⚠ | CONTESTED |
| `index_select` | non-primitive | gather_scatter | 1 | `idx/emb` ⚠ | CONTESTED |
| `scatter_add` | non-primitive | gather_scatter | 3 | **idx** | prose |
| `logical_and` | non-primitive | logical | 2 | **bin** | formula |
| `logical_not` | non-primitive | logical | 1 | **une** | formula |
| `logical_or` | non-primitive | logical | 2 | **bin** | formula |
| `fmax_ieee` | non-primitive | minmax | 2 | **bin** | formula |
| `fmin_ieee` | non-primitive | minmax | 2 | **bin** | formula |
| `max_prop` | non-primitive | minmax | 2 | **bin** | formula |
| `min_prop` | non-primitive | minmax | 2 | **bin** | formula |
| `layer_norm` | non-primitive | normalization | 3 | **nrm** | prose |
| `log_softmax` | non-primitive | normalization | 1 | `nrm/sft` ⚠ | CONTESTED |
| `rms_norm` | non-primitive | normalization | 1 | **nrm** | formula |
| `softmax` | non-primitive | normalization | 1 | `nrm/sft` ⚠ | CONTESTED |
| `all` | non-primitive | reduction | 1 | **red** | formula |
| `any` | non-primitive | reduction | 1 | **red** | formula |
| `argmax` | non-primitive | reduction | 1 | **red** | formula |
| `logsumexp` | non-primitive | reduction | 1 | **red** | formula |
| `reduce_mean` | non-primitive | reduction | 1 | **red** | formula |
| `reduce_norm2` | non-primitive | reduction | 1 | **red** | formula |
| `reduce_std` | non-primitive | reduction | 1 | **red** | formula |
| `reduce_var` | non-primitive | reduction | 1 | **red** | formula |
| `frac` | non-primitive | rounding | 1 | **une** | formula |
| `cummax` | non-primitive | scan | 1 | **scn** | prose |
| `cumprod` | non-primitive | scan | 1 | **scn** | prose |
| `cumsum` | non-primitive | scan | 1 | **scn** | prose |
| `im2col` | non-primitive | shape | 1 | `shp/cnv` ⚠ | CONTESTED |
| `acos` | non-primitive | transcendental | 1 | **une** | formula |
| `acosh` | non-primitive | transcendental | 1 | **une** | formula |
| `asin` | non-primitive | transcendental | 1 | **une** | formula |
| `asinh` | non-primitive | transcendental | 1 | **une** | formula |
| `atanh` | non-primitive | transcendental | 1 | **une** | formula |
| `cbrt` | non-primitive | transcendental | 1 | **une** | formula |
| `cosh` | non-primitive | transcendental | 1 | **une** | formula |
| `erfc` | non-primitive | transcendental | 1 | **une** | formula |
| `exp2` | non-primitive | transcendental | 1 | **une** | formula |
| `expm1` | non-primitive | transcendental | 1 | **une** | formula |
| `log10` | non-primitive | transcendental | 1 | **une** | formula |
| `log1p` | non-primitive | transcendental | 1 | **une** | formula |
| `log2` | non-primitive | transcendental | 1 | **une** | formula |
| `rsqrt` | non-primitive | transcendental | 1 | **une** | formula |
| `sinh` | non-primitive | transcendental | 1 | **une** | formula |
| `tan` | non-primitive | transcendental | 1 | **une** | formula |
| `tanh` | non-primitive | transcendental | 1 | **une** | formula |
| `avg_pool` | non-primitive | window | 1 | `pol/cnv` ⚠ | CONTESTED |
| `max_pool` | non-primitive | window | 1 | `pol/cnv` ⚠ | CONTESTED |
| `element_map` | primitive | access-primitive | — | **une/bin/ter** | op identity |
| `gather` | primitive | access-primitive | 1 | **idx** | op identity |
| `prefix_scan` | primitive | access-primitive | 1 | **scn** | op identity |
| `reduce` | primitive | access-primitive | 2 | **red** | op identity |
| `scatter` | primitive | access-primitive | 2 | **idx** | op identity |
| `sort_network` | primitive | access-primitive | 2 | **srt** | op identity |
| `abs` | primitive | arithmetic | 1 | **une** | formula |
| `add` | primitive | arithmetic | 2 | **bin** | formula |
| `div` | primitive | arithmetic | 2 | **bin** | formula |
| `mul` | primitive | arithmetic | 2 | **bin** | formula |
| `neg` | primitive | arithmetic | 1 | **une** | formula |
| `sub` | primitive | arithmetic | 2 | **bin** | formula |
| `atan2` | primitive | binary_math | 2 | **bin** | formula |
| `copysign` | primitive | binary_math | 2 | **bin** | formula |
| `nextafter` | primitive | binary_math | 2 | **bin** | formula |
| `bit_and` | primitive | bitwise | 2 | **bin** | formula |
| `bit_not` | primitive | bitwise | 1 | **une** | formula |
| `bit_or` | primitive | bitwise | 2 | **bin** | formula |
| `bit_xor` | primitive | bitwise | 2 | **bin** | formula |
| `clz` | primitive | bitwise | 1 | **une** | unstated |
| `ctz` | primitive | bitwise | 1 | **une** | unstated |
| `popcount` | primitive | bitwise | 2 | **bin** | formula |
| `shl` | primitive | bitwise | 2 | **bin** | formula |
| `shr` | primitive | bitwise | 2 | **bin** | formula |
| `cmp_eq` | primitive | comparison | 2 | **bin** | formula |
| `cmp_ge` | primitive | comparison | 2 | **bin** | formula |
| `cmp_gt` | primitive | comparison | 2 | **bin** | formula |
| `cmp_le` | primitive | comparison | 2 | **bin** | formula |
| `cmp_lt` | primitive | comparison | 2 | **bin** | formula |
| `cmp_ne` | primitive | comparison | 2 | **bin** | formula |
| `ceil` | primitive | rounding | 1 | **une** | unstated |
| `floor` | primitive | rounding | 1 | **une** | formula |
| `round_even` | primitive | rounding | 1 | **une** | unstated |
| `trunc` | primitive | rounding | 1 | **une** | unstated |
| `select` | primitive | select | 3 | **ter** | formula |
| `atan` | primitive | transcendental | 1 | **une** | unstated |
| `cos` | primitive | transcendental | 1 | **une** | formula |
| `erf` | primitive | transcendental | 1 | **une** | unstated |
| `exp` | primitive | transcendental | 1 | **une** | formula |
| `lgamma` | primitive | transcendental | 1 | **une** | formula |
| `log` | primitive | transcendental | 1 | **une** | formula |
| `sin` | primitive | transcendental | 1 | **une** | formula |
| `sqrt` | primitive | transcendental | 1 | **une** | formula |

⚠ = contested pair, architect's call (§5).

## 7. What this proposal asks for

1. **A ruling on the six contested pairs** (§5). Until those are decided, twelve ops have
   no determinate family, and `gem`'s 8 vectors rest on an unstated assumption.
2. **Confirmation from the other derivers** that these assignments match what they already
   emit. This is the whole point of proposing rather than pinning — a mapping that
   contradicts a live deriver is a token change, not a clarification.
3. **A decision on `element_map`**, whose family is body-dependent and cannot be fixed.
4. **No pinning until 1–3 are answered**, because the edit is not additive.

## 8. Provenance

Measured at `origin/main`, not in the shared anchor. Population and family tags derived
from `spec/ops.md` §2.7/§6.13 by an extractor gated on a **positive control of 23 known
arities**, which failed twice before passing — first reading only table cells and missing
§6.9's clause-body formulas, then on `atan2`'s `(y=a, x=b)` spelling. The 24 codes are from
`conformance/src/structure_key.rs:26` and `spec/classify.md` §6.5. Vector coverage is field
1 of the `token` string in each row of `conformance/corpus/structure_key_vectors.json` —
**not** a JSON `op_family` key, which does not exist and returns 0 if grepped for.
