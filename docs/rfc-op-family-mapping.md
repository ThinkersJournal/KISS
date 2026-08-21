# Proposal — the op→family mapping (#263)

**Status: PROPOSAL. This introduces no clause, no §9 row, and no coverage-floor movement**
(the ratchet is unchanged at `380/33/496`), and it is deliberately not drafted as normative
text. The reason is the finding itself: `op_family_tag` is a `structure_key` cell field, and
`structure_key` is the admissibility key — so **pinning a mapping is not an additive edit.**
If any deriver already assigns differently from what is pinned here, its tokens change, and
kernels that are admissible today stop being admissible. A mapping may be pinned only once
the derivers are known to agree; this document exists to find out whether they do.

**And the majority of it cannot be falsified by anything in the repository.** Of the 24
family codes, **5 are exercised by a positive vector** — `bin`, `gem`, `red`, `scn`, `une` —
across the 21 positive vectors in `conformance/corpus/structure_key_vectors.json`. The other
19 have no vector behind them. A mapping over 24 families where 19 are unexercised **is a
proposal about intent**, and every reader should treat the unexercised rows as stated intent
rather than measured behaviour.

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
§6.3 ∪ §6.13 ∪ **§6.18**, so a sweep scoped to 106 silently omits the entire complex family.
This mapping covers all 121.

**A second correction, to #263's own measurement.** #263 reports that ops.md carries a
*nine-value* op category vocabulary, listing `shape 69 · transcendental 67 · reduction 60`
and so on. **Those are occurrence counts of nine words across the document, not op counts,
and the actual §2.7 family-tag set has 18 values.** The nine it misses — `access-primitive`,
`arithmetic`, `binary_math`, `bitwise`, `comparison`, `logical`, `minmax`, `rounding`,
`select` — carry **51 of the 106** ops. The issue's central claim (two vocabularies that do
not correspond) is unaffected and correct; its measurement of the near side is not.

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
| a formula in a semantics table (`a + b`, `-x`) or a clause body (§6.9-0001's `(y=a, x=b)`) | 85 |
| a prose operand-ordering paragraph (§6.13 preamble: `input(0)=x, input(1)=gamma`) | 6 |
| **nowhere — prose only** (`ceil` “round toward +∞”), inferred from meaning | 12 |
| an access-primitive, assigned by op identity rather than arity | 6 |
| settled by the most-specific-wins ruling (§5) rather than by arity | 12 |
| **total** | **121** |

Two of those rows deserve attention. The `§6.9` case was found only because an extractor
gated on a positive control **failed** on `atan2` — its arity is in a clause body, not a
table, so a table-only sweep silently reports it as unknown. And `element_map` is the sharp
case: its Classify family depends on the **arity of the body it carries**, which is not a
property of the op at all. **It cannot be assigned a fixed code.**

**Reading arity out of prose does not work, and this document got it wrong first.** Arity is
now read **only from inside backticked code spans**: matching operand names across whole
sentences also matches the English article *“a”*, which gave `popcount` — an op with **no**
formula anywhere — an arity of 2. §8 records every failure mode. The point here is that
**the prose ops.md uses to describe an op is not a substitute for a signature it never
states.**

## 4. Coverage of the 24-code alphabet

Under the §5 ruling:

| Class | Count | Codes |
|---|---|---|
| reachable by some KISS-Ops op | 14 | `bin`, `cnv`, `emb`, `gat`, `gem`, `idx`, `nrm`, `pol`, `red`, `scn`, `sft`, `srt`, `ter`, `une` |
| **no KISS-Ops op at all** | 10 | `shp`, `qnt`, `rnd`, `los`, `seg`, `img`, `fft`, `lin`, `att`, `moe` |

**Ten of the twenty-four codes have no KISS-Ops op.** There is no quantize op, no loss op,
no attention op, no FFT, no random, no segment, no image, and no mixture-of-experts op in
this version's set; and `shp`, `lin` lose their only candidates to the ruling
(below). This is **not** a defect in the alphabet: per #263's own argument the family tag
records **what a cell carries on the wire, not what ops an implementation has**, and a
producer that fuses may legitimately emit an `att` or `moe` cell. But it does mean **a
substantial part of the alphabet is unpopulated by the op set**, and no mapping from
KISS-Ops can make those codes reachable.

**The inversion worth reading twice:** `gem` is the **most-exercised** family in the corpus
— 8 of the 21 positive vectors — and until the §5 ruling it was assigned to `matmul` by no
normative text at all. **The corpus had already committed to one arm of an undecided
question, in the family it tests hardest.** The ruling ratifies those 8 vectors; the point
is that it could have invalidated them, and nothing recorded which way it would go.

## 5. The ruling on the six ambiguous pairs

**Ruled by the architect on #263: where two codes both apply, the most specific wins.**
This document does not decide it and does not restate it as normative text — the ruling is
itself a proposal, for the same reason this mapping is.

| Pair (specific / general) | Ops | Resolves to |
|---|---|---|
| `sft` / `nrm` | `softmax`, `log_softmax` | **`sft`** |
| `emb` / `idx` | `embedding` | **`emb`** (`index_select` stays `idx` — it is a gather, not an embedding) |
| `gat` / `une` | `silu`, `gelu`, `gelu_tanh`, `mish` | **`gat`** |
| `gem` / `lin` | `matmul` | **`gem`** |
| `pol` / `cnv` | `avg_pool`, `max_pool` | **`pol`** |
| `cnv` / `shp` | `im2col` | **`cnv`** |

**The rule is load-bearing rather than a preference, and the op set shows why.** If the
general code won whenever both applied, the specific codes would be unreachable *by
construction* — every op that would occupy them is already covered by a general code. Set
against the actual op set:

| Reading | Codes reachable | Codes with no op |
|---|---|---|
| **most-specific-wins** (ruled) | **14** | 10 |
| most-general-wins (rejected) | 11 | 13 |

**But the ruling does not remove unreachability — it moves it, and that is worth recording**
**rather than glossing.** Under most-specific-wins, `shp`, `lin` become
unreachable instead: `lin`'s only candidate was `matmul`, which now resolves to `gem`, and
`shp`'s only candidate was `im2col`, which now resolves to `cnv`. Both are defensible —
KISS-Ops has no linalg ops (no solve, no inverse, no decomposition), and it expresses shape
manipulation through **strides and `layout_tag` on the operand descriptor rather than**
**through ops**, so a version with no reshape op having no `shp` cell is coherent. The net is
still strongly in the ruling's favour (14 reachable versus 11). **This is recorded as**
**decided, not noticed** — a rule justified by *“otherwise these five are unreachable”* that
quietly makes two others unreachable is the **partial-enumeration defect** (convention 16(d))
appearing inside the ruling that cites it. Stating the exchange is what keeps the next reader
from finding `lin` and `shp` empty and concluding nobody looked.

**`shp` sitting empty is ruled intended, not an oversight.** KISS-Ops expresses shape
manipulation through **strides and `layout_tag` on the operand descriptor, not through ops**
— there is no reshape, no transpose, and no permute in the op set. A version with no shape
ops having no `shp` cell is therefore coherent. Recorded explicitly because an empty `shp`
otherwise reads as an oversight forever.

**The non-retroactivity condition, which the ruling carries and the mapping must not lose:**

> A family code MAY be added freely. **Existing ops MUST NOT be reassigned to it without a**
> **schema bump**, even where the new code is more specific — most-specific-wins binds at
> **assignment** time, not retroactively.

Without it the rule quietly makes every future vocabulary addition a breaking change under
`§7.2-0002`.

## 6. The mapping

`basis` records **how the assignment was reached**, so a reader can tell a lookup from an
inference: `formula` = arity read off a formula; `prose` = arity from an operand-ordering
paragraph; `unstated` = **arity nowhere stated, inferred from the op's meaning**;
`op identity` = an access-primitive assigned directly; `most-specific-wins` = settled by the
§5 ruling.

**Which rows are a judgement and which fell out of the definition.** A reviewer handed 121
rows checks the shape; a reviewer handed the discretionary subset checks the decisions. The
split:

| Basis | Rows | What a reviewer is checking |
|---|---|---|
| **derived** — arity from a formula, a prose operand-ordering paragraph, an access-primitive's identity, or ops.md's own family tag | 96 | that the derivation rule is right, once |
| **ruled** — the six most-specific-wins pairs (§5) | 12 | the architect's ruling, already given |
| **judgement** — arity stated **nowhere** in ops.md, inferred from the op's meaning | **12** | **each row, individually** |
| **unassigned** — deliberately left open (§7) | 1 | that leaving it open is right |
| **total** | 121 | |

**The 12 judgement rows are the ones worth a reviewer's attention:** `atan`, `ceil`, `cim`, `clz`, `cre`, `ctz`, `erf`, `floor`, `popcount`, `round_even`, `sqrt`, `trunc`.
Every one resolves to `une`, and every one is an op ops.md describes in prose without a
formula — `ceil` is *“round toward +∞”*, and nothing states its arity. They are marked
`unstated` in the basis column below so this subset is recoverable from the table itself.
**Review has already corrected one of these twelve** (`popcount`, which the extractor had
read as binary from prose), which is the evidence that this is the bucket where errors
live.

| Op | Tier | ops.md family | Arity | → code | Basis |
|---|---|---|---|---|---|
| `cabs` | complex | complex | 1 | **une** | formula |
| `cadd` | complex | complex | 2 | **bin** | formula |
| `carg` | complex | complex | 1 | **une** | formula |
| `cconj` | complex | complex | 1 | **une** | formula |
| `cdiv` | complex | complex | 2 | **bin** | formula |
| `cexp` | complex | complex | 1 | **une** | formula |
| `cim` | complex | complex | 1 | **une** | unstated |
| `clog` | complex | complex | 1 | **une** | formula |
| `cmake` | complex | complex | 2 | **bin** | formula |
| `cmul` | complex | complex | 2 | **bin** | formula |
| `cneg` | complex | complex | 1 | **une** | formula |
| `cpow` | complex | complex | 2 | **bin** | formula |
| `cre` | complex | complex | 1 | **une** | unstated |
| `csqrt` | complex | complex | 1 | **une** | formula |
| `csub` | complex | complex | 2 | **bin** | formula |
| `gelu` | non-primitive | activation | 1 | **gat** | most-specific-wins |
| `gelu_tanh` | non-primitive | activation | 1 | **gat** | most-specific-wins |
| `mish` | non-primitive | activation | 1 | **gat** | most-specific-wins |
| `relu` | non-primitive | activation | 1 | **une** | formula |
| `sigmoid` | non-primitive | activation | 1 | **une** | formula |
| `silu` | non-primitive | activation | 1 | **gat** | most-specific-wins |
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
| `matmul` | non-primitive | contraction | 2 | **gem** | most-specific-wins |
| `embedding` | non-primitive | gather_scatter | 2 | **emb** | most-specific-wins |
| `index_select` | non-primitive | gather_scatter | 2 | **idx** | most-specific-wins |
| `scatter_add` | non-primitive | gather_scatter | 3 | **idx** | prose |
| `logical_and` | non-primitive | logical | 2 | **bin** | formula |
| `logical_not` | non-primitive | logical | 1 | **une** | formula |
| `logical_or` | non-primitive | logical | 2 | **bin** | formula |
| `fmax_ieee` | non-primitive | minmax | 2 | **bin** | formula |
| `fmin_ieee` | non-primitive | minmax | 2 | **bin** | formula |
| `max_prop` | non-primitive | minmax | 2 | **bin** | formula |
| `min_prop` | non-primitive | minmax | 2 | **bin** | formula |
| `layer_norm` | non-primitive | normalization | 3 | **nrm** | prose |
| `log_softmax` | non-primitive | normalization | 1 | **sft** | most-specific-wins |
| `rms_norm` | non-primitive | normalization | 2 | **nrm** | prose |
| `softmax` | non-primitive | normalization | 1 | **sft** | most-specific-wins |
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
| `im2col` | non-primitive | shape | 1 | **cnv** | most-specific-wins |
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
| `avg_pool` | non-primitive | window | 1 | **pol** | most-specific-wins |
| `max_pool` | non-primitive | window | 1 | **pol** | most-specific-wins |
| `element_map` | primitive | access-primitive | — | **unassigned** ⚠ | unresolvable at the op level |
| `gather` | primitive | access-primitive | 2 | **idx** | op identity |
| `prefix_scan` | primitive | access-primitive | 1 | **scn** | op identity |
| `reduce` | primitive | access-primitive | 1 | **red** | op identity |
| `scatter` | primitive | access-primitive | 3 | **idx** | op identity |
| `sort_network` | primitive | access-primitive | 1 | **srt** | op identity |
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
| `popcount` | primitive | bitwise | 1 | **une** | unstated |
| `shl` | primitive | bitwise | 2 | **bin** | formula |
| `shr` | primitive | bitwise | 2 | **bin** | formula |
| `cmp_eq` | primitive | comparison | 2 | **bin** | formula |
| `cmp_ge` | primitive | comparison | 2 | **bin** | formula |
| `cmp_gt` | primitive | comparison | 2 | **bin** | formula |
| `cmp_le` | primitive | comparison | 2 | **bin** | formula |
| `cmp_lt` | primitive | comparison | 2 | **bin** | formula |
| `cmp_ne` | primitive | comparison | 2 | **bin** | formula |
| `ceil` | primitive | rounding | 1 | **une** | unstated |
| `floor` | primitive | rounding | 1 | **une** | unstated |
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
| `sqrt` | primitive | transcendental | 1 | **une** | unstated |

⚠ **`element_map` is deliberately left unassigned** — see §7. Assigning it a code would
be the error, not the omission.

## 7. What this proposal asks for

1. **Confirmation from the other derivers** that these assignments match what they already
   emit. This is the whole point of proposing rather than pinning — a mapping that
   contradicts a live deriver is a token change, not a clarification.
2. **A cross-party answer on `element_map`**, which is *not* a case of a missing assignment.
   Its family depends on the arity of **the body it carries**, so it is **not a property of
   the op at all**, and most-specific-wins cannot reach it — there is no fixed pair of
   candidates to choose between. **It is left unassigned deliberately.** The question for
   the derivers: *if `element_map`'s family varies by instance, then either the mapping is
   not a function of the op, or `element_map` needs its own rule.*

   **This may be the sharpest evidence that the mapping cannot be authored from `ops.md`
   alone.** #298 says the *input* is unstated; `element_map` says that for at least one op
   **there is no per-op input to state.**
3. **No pinning until 1–2 are answered**, because the edit is not additive.

The ruling's two displaced codes (`lin`, `shp`) are **settled** — §5 records the exchange
and the `shp` reading as ruled, rather than leaving them to be rediscovered.

## 8. Provenance

Measured at `origin/main`, not in the shared anchor. The 24 codes are from
`conformance/src/structure_key.rs:26` and `spec/classify.md` §6.5. Vector coverage is field 1
of the `token` string in each row of `conformance/corpus/structure_key_vectors.json` —
**not** a JSON `op_family` key, which does not exist and returns 0 if grepped for. Every
count in this document is computed from the mapping data at render time rather than written
by hand.

**The arity extractor was wrong five times, and not one was caught by reading it** — 3 by review, 2 by its own controls. They are recorded because the corrected mapping is only as trustworthy as the thing that found the errors:

| Failure | Effect | Caught by |
|---|---|---|
| read only table cells | §6.9’s clause-body formulas invisible; `atan2` unknown | positive control |
| matched variables in **prose**, so the English article *“a”* counted as operand `a` | `popcount` — which has **no formula at all** — came out arity 2 | **review (#304)** |
| read complex **component** vars as operands | `cabs` = `hypot(a, b)` over *one* operand read as binary | **review (#304)** |
| counted only component vars, missing whole-operand names | `cpow` = `cexp(cmul(w, clog(z)))` is **binary** and read as unary | positive control, after the fix |
| fixed the whole-operand case for `(w, z)` only, missing `(x, y)` | `cmake(x, y)` — which “consumes two real lanes” — still read as unary | **review (#304)** |

**Two of those rows are the fixes for the rows above them.** Correcting the prose-matching and complex-component errors introduced the `cpow` error; correcting `cpow` left its sibling `cmake` unfixed. **A fix for a miscount is itself a count, and inherits the same failure mode** — which is the argument for a control rather than for more care.

The extractor is now gated on **36 known arities and a 5-op negative control** asserting
that ops with no formula report *unknown* rather than guessing — the check the article bug
slipped through, because the original control tested only that known arities were RIGHT and
never that unknown ones stayed UNKNOWN. A control that checks one direction cannot see a
detector that fires on everything.

**Blast radius of the correction: 23 of 121 extracted arities changed, and 8 final Classify
codes** — `cabs`, `carg`, `cconj`, `cexp`, `cneg`, `csqrt`, `popcount` (`bin` → `une`) and
`cmake` (`une` → `bin`). Review reported 5 of those 8; the rest came from re-deriving rather
than patching the reported rows. These are one-time measurements of the correction, not live
counts — every other figure in this document is computed at render time.
count in this document is computed from the mapping data at render time rather than written
by hand.
