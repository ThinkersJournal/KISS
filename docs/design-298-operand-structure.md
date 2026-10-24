# Design — per-op operand structure in `ops.md` (KISS#298, ROADMAP R1.12)

**Status: proposal for the PM gate. No spec, wire or tooling change in this PR.** It proposes the field,
the role vocabulary and the 122-row content so the spec PR that follows is a transcription, not a
design. Measured at `origin/main` `b6d0531`.

## What the architect ruling (KISS#298, first comment) already fixes

1. The field carries operand **count** and per-operand **role**, nothing else.
2. **No broadcast column** — broadcastability is §6.20's; value semantics are §6.13's.
3. Roles come from a **closed, append-only** set, never free text (extension additive under
   `KISS-CLASSIFY-7.2-0001`).
4. The count's **form admits variadic from day one**, unused until an op needs it.
5. Sequenced after #305 — now satisfied: `op_manifest.json` `all_ops` is 122 and contains the §6.18
   complex ops (the issue's "106" predates that).

## What this document decides (for the gate to accept or change)

**D1 — Where it lives.** A new table in §6.1 (the op-set registry), one row per op, next to the
registry the manifest is generated from; `op_manifest.json` gains `operand_structure`
(`{op: {count, roles}}`) generated from it and checked for completeness by the same tool that checks
`all_ops` (the omission #305 describes must be a failure, not a silent shrink).

**D2 — Count form.** A decimal `n`, a closed range `n..m` (an optional trailing operand), or
`variadic`. Of 122 ops: 77 are `1`, 39 are `2`, 4 are `3` (`select`, `layer_norm`, `scatter`,
`scatter_add`), `gather` is `2..3` (the §6.11-0015 optional `base` operand), and `element_map` is
`variadic` (its scalar body names however many `input(i)` it reads). No other op is variadic.

**D3 — Role vocabulary (closed, append-only).** Every role below is already a name the spec uses
for an operand position; nothing new is coined:

`operand` (the sole or generic value operand) · `lhs`, `rhs` · `cond`, `on_true`, `on_false` ·
`exponent` · `scale`, `shift` (`gamma`, `beta` of §6.13) · `data`, `index`, `dest`, `updates`,
`base` · `real`, `imag`.

`index` is the index-operand role of §6.11-0012 and is the same fact the OpAttrs `index_operand`
field names. Not in the set, deliberately: `weight` (KISS#219 closed — KISS-CLASSIFY-6.6-0019
defines it as a caller hint, not an op operand role) and anything about broadcast.

**D4 — Positional order is the order of the Roles column.** It must agree with KISS-Classify's
canonical operand order so no second wire form appears (cf. KISS-OPS-6.20-0003, which cites that
order as "§6.5" while classify.md itself speaks of "the canonical operand order (§6.6-0014)"). **I have
not verified that the two orders coincide for every op, nor which citation is right; the spec PR must
do both before the table is normative.**

## The 122 rows

How each row was checked (rerun: `python arity.py <repo> <out>`, kept with the spec PR):

- The row set equals `op_manifest.json` `all_ops` exactly (122 = 122; missing and extra both
  asserted empty).
- **51 rows are cross-checked** against the free variables of their §6.13 reference decomposition
  (`x` only ⇒ 1; `a`,`b` ⇒ 2): 0 mismatches. The extractor excludes bodies that use `input(i)`.
- Positive control: eight arities known independently from the spec's own formulas
  (`add` 2, `select` 3, `neg` 1, `layer_norm` 3, `matmul` 2, `scatter` 3, `cmake` 2, `sqr` 1) are
  asserted against the table.
- **The other 71 rows are NOT machine-cross-checked.** Their basis is the cited clause or catalog
  row (the Basis column). The issue records that a regex over prose returned `select` as arity 1;
  these rows were curated by hand for that reason and need a reader, not a re-run.

| Op | Count | Roles (position order) | Basis |
|---|---|---|---|
| `abs` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `acos` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `acosh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `add` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `all` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `any` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `argmax` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `argmin` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `asin` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `asinh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `atan` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `atan2` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `atanh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `avg_pool` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `bit_and` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `bit_not` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `bit_or` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `bit_xor` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cabs` | 1 | operand | §6.18 table: complex unary / component read |
| `cadd` | 2 | lhs, rhs | §6.18 table: complex binary |
| `carg` | 1 | operand | §6.18 table: complex unary / component read |
| `cbrt` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `cconj` | 1 | operand | §6.18 table: complex unary / component read |
| `cdiv` | 2 | lhs, rhs | §6.18 table: complex binary |
| `ceil` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `cexp` | 1 | operand | §6.18 table: complex unary / component read |
| `cim` | 1 | operand | §6.18 table: complex unary / component read |
| `clog` | 1 | operand | §6.18 table: complex unary / component read |
| `clz` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `cmake` | 2 | real, imag | §6.18: lane 0 = `input(0)`, lane 1 = `input(1)` |
| `cmp_eq` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cmp_ge` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cmp_gt` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cmp_le` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cmp_lt` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cmp_ne` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cmul` | 2 | lhs, rhs | §6.18 table: complex binary |
| `cneg` | 1 | operand | §6.18 table: complex unary / component read |
| `copysign` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `cos` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `cosh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `cpow` | 2 | lhs, rhs | §6.18 table: complex binary |
| `cre` | 1 | operand | §6.18 table: complex unary / component read |
| `csqrt` | 1 | operand | §6.18 table: complex unary / component read |
| `csub` | 2 | lhs, rhs | §6.18 table: complex binary |
| `ctz` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `cummax` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `cumprod` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `cumsum` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `div` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `element_map` | n (variadic) | operand* | KISS-OPS-6.11-0001: the base access over a per-element scalar body; the body names its `input(i)` |
| `embedding` | 2 | data, index | §6.13 `gather` with a 1-D index operand |
| `erf` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `erfc` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `exp` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `exp2` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `expm1` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `floor` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `fmax_ieee` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `fmin_ieee` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `frac` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `gather` | 2..3 | data, index, base? | KISS-OPS-6.11-0012/-0015: index operand + OPTIONAL `base` |
| `gelu` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `gelu_tanh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `hypot` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `im2col` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `index_select` | 2 | data, index | §6.13 `gather` with a 1-D index operand |
| `layer_norm` | 3 | operand, scale, shift | §6.13 convention: `input(0..2)=x,gamma,beta`; `eps`=`param(0)` |
| `ldexp` | 2 | operand, exponent | §6.13 `ldexp(a, b)`; `b` is the integer exponent |
| `lgamma` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `log` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `log10` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `log1p` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `log2` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `log_softmax` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `logical_and` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `logical_not` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `logical_or` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `logsumexp` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `matmul` | 2 | lhs, rhs | §6.13 operand-ordering convention: `input(0)=lhs`, `input(1)=rhs` |
| `max_pool` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `max_prop` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `min_prop` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `mish` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `mul` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `neg` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `nextafter` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `popcount` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `pow` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `prefix_scan` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `recip` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `reduce` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `reduce_mean` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `reduce_norm2` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `reduce_std` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `reduce_var` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `relu` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `rem_floor` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `rem_trunc` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `rms_norm` | 2 | operand, scale | §6.13 convention: `input(0)=x`, `input(1)=gamma`; `eps`=`param(0)` is a param, not an operand |
| `round_even` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `rsqrt` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `scatter` | 3 | dest, index, updates | KISS-OPS-6.11-0016/-0017: `dest` and `updates` operands + index operand |
| `scatter_add` | 3 | dest, index, updates | KISS-OPS-6.11-0016/-0017: `dest` and `updates` operands + index operand |
| `select` | 3 | cond, on_true, on_false | KISS-OPS-6.5-0001: order `(cond, a, b)` |
| `shl` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `shr` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `sigmoid` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `sign` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `silu` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `sin` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `sinh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `softmax` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `softplus` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `sort_network` | 1 | operand | §6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window |
| `sqr` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `sqrt` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `step` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `sub` | 2 | lhs, rhs | §6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b` |
| `tan` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `tanh` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |
| `trunc` | 1 | operand | catalog §2.7; §6.13 body free variable `x` |

## Rows most worth a second reader

`ldexp` (second operand named `exponent`: §6.13 says only `ldexp(a, b)` with `b` integral);
`im2col`, `sort_network`, `prefix_scan` and the `reduce` family (count 1 read from the
decomposition column; their window, axis and monoid are attributes, not operands); `gather` `2..3`
(whether an optional operand is a count range or a second row); `cmake` roles `real`/`imag`.

## What the spec PR after this one does

1. Adds the §6.1 table and a clause stating count form and the closed role set, each with a named
   test (`kiss_trace` clause↔test), and a sidecar update.
2. Extends the manifest generator and its completeness check; a negative control deletes one row
   and must fail.
3. Unblocks #263's residual: for ops no semantic family claims, `une`/`bin`/`ter` follows from the
   count. **It changes no existing token** (KISS-CLASSIFY-7.2-0002): this PR and that one only state
   what the families' input already was.
4. Lands under the normative-text rule: PM + KISS + every affected agent (Unpopped derives
   `op_family`; Fuel and Baracuda read the manifest).

## Not decided here

Whether `lhs`/`rhs` should be `a`/`b` (the spelling §6.13 bodies use) — the proposal prefers
`lhs`/`rhs` because `matmul` already pins them. Any broadcast fact (ruled out).
