# Design — per-op operand structure in `ops.md` (KISS#298, ROADMAP R1.12)

> **RETRACTION (2026-10-24, KISS lane): the premise below is out of date, and so are the
> passages that rest on it.** This document, and the architect ruling it implements, say arity
> is the input to `op_family` (`une`/`bin`/`ter`) and that #263 is blocked on this table. At
> `origin/main` `893cf57` neither holds: a33282c (#263 ruling D, 2026-10-11) dropped the
> twenty-four op-family codes, and KISS-CLASSIFY-6.5-0015 derives a 7-class `cell_class` from
> the cell's own operands and "MUST NOT read an op name, an op family, or any property of the
> computation, and no op-to-class table exists in this suite or is needed". **Struck: "Unblocks
> #263's residual" (step 3 below), "Unpopped derives `op_family`" (step 4), and every statement
> that the count feeds a family.** What survives is only the ruling's secondary motive (a closed
> role vocabulary so a role such as `weight` is defined once, #219) and the fact that `ops.md`
> still states no per-op operand structure. Whether that is worth a spec table with no named
> consumer is open (ROADMAP R1.12); the 122 rows and the extractor remain valid as measurements.

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

**UNVERIFIED — read this before relying on any row.** Every count and role below is a proposal. Each
row carries its anchor in `spec/ops.md` (line, section, and what that row says) and an **Evidence**
class, so a reader can check it mechanically against the quoted text:

- `formula` (77 rows): a spec table row for the op uses only the letters the count predicts (`x` for
  1, `a` and `b` for 2) in its leading code span. Mechanical, asserted by the script.
  **Counts only — the role names are not derived from anything**, they are the proposal in D3.
- `clause` (25 rows): the count and roles are stated by the named clause or convention, including the
  complex ops, whose `a`,`b` are the components of `z` and not operands.
- `meaning-only` (20 rows): the spec row names no operand letters, so the count is read from what the
  op *means* (`floor`, `clz`, `sqrt`, the scans and pools, …). **These 20 are the ones that need a
  human reader**; the anchor shows what they would be checked against. The issue records that a regex
  over prose once returned `select` as arity 1.

Rerun: `python -I docs/design-298-extract.py <repo-root> <out.md>` (it also asserts the row set equals
`op_manifest.json` `all_ops`, 122 = 122; cross-checks 51 rows against §6.13 decomposition free
variables with 0 mismatches; and asserts eight independently known arities — `add` 2, `select` 3,
`neg` 1, `layer_norm` 3, `matmul` 2, `scatter` 3, `cmake` 2, `sqr` 1 — as a positive control).

**D4 is UNVERIFIED** (see above): agreement with Classify's canonical operand order is not checked.

| Op | Count | Roles (position order) | Anchor: first spec row (ops.md line, §) and what it says | Evidence |
|---|---|---|---|---|
| `abs` | 1 | operand | ops.md:244 (§2.7) — arithmetic ` x ` by clearing the sign bit (raw-bit) | formula |
| `acos` | 1 | operand | ops.md:1278 (§6.13) — transcendental — `sub(const(pi/2), asin(x))` | formula |
| `acosh` | 1 | operand | ops.md:1275 (§6.13) — transcendental — `log(add(x, sqrt(sub(sqr(x), const(1)))))` | formula |
| `add` | 2 | lhs, rhs | ops.md:239 (§2.7) — arithmetic `a+b`; IEEE-754 float / wrapping int | formula |
| `all` | 1 | operand | ops.md:1304 (§6.13) — reduction — `reduce(min, cmp_ne(x, const(0)))` | formula |
| `any` | 1 | operand | ops.md:1303 (§6.13) — reduction — `reduce(max, cmp_ne(x, const(0)))` | formula |
| `argmax` | 1 | operand | ops.md:1301 (§6.13) — reduction — `original-index at rank 0 of sort_network(desc, keys=x)` (the §6.11-0007 index-vect… | formula |
| `argmin` | 1 | operand | ops.md:1302 (§6.13) — reduction — `original-index at rank 0 of sort_network(asc, keys=x)` (the §6.11-0007 index-vecto… | formula |
| `asin` | 1 | operand | ops.md:1277 (§6.13) — transcendental — `atan(div(x, sqrt(sub(const(1), sqr(x)))))` | formula |
| `asinh` | 1 | operand | ops.md:1274 (§6.13) — transcendental — `log(add(x, sqrt(add(sqr(x), const(1)))))` | formula |
| `atan` | 1 | operand | ops.md:262 (§2.7) — transcendental arctangent (declared-ULP atom) | meaning-only |
| `atan2` | 2 | lhs, rhs | ops.md:264 (§2.7) — binary_math four-quadrant arctangent, IEEE ±0 quadrants | meaning-only |
| `atanh` | 1 | operand | ops.md:1276 (§6.13) — transcendental — `mul(const(0.5), log(div(add(const(1),x), sub(const(1),x))))` | formula |
| `avg_pool` | 1 | operand | ops.md:1313 (§6.13) — window — `reduce_mean` over the window axis of the pooled view (§6.13-0004 window attributes); … | meaning-only |
| `bit_and` | 2 | lhs, rhs | ops.md:267 (§2.7) — bitwise `a & b` (integer only) | formula |
| `bit_not` | 1 | operand | ops.md:270 (§2.7) — bitwise `~x` (integer only) | formula |
| `bit_or` | 2 | lhs, rhs | ops.md:268 (§2.7) — bitwise `a b` (integer only) | formula |
| `bit_xor` | 2 | lhs, rhs | ops.md:269 (§2.7) — bitwise `a ^ b` (integer only) | formula |
| `cabs` | 1 | operand | ops.md:1824 (§6.18) — complex ULP/tolerance `hypot(a, b)` → real | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `cadd` | 2 | lhs, rhs | ops.md:1818 (§6.18) — complex exact-byte `cmake(add(a,c), add(b,d))` | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `carg` | 1 | operand | ops.md:1825 (§6.18) — complex ULP/tolerance `atan2(b, a)` → real | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `cbrt` | 1 | operand | ops.md:1279 (§6.13) — transcendental — `mul(sign(x), exp(div(log(abs(x)), const(3))))` | formula |
| `cconj` | 1 | operand | ops.md:1821 (§6.18) — complex exact-byte `cmake(a, neg(b))` | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `cdiv` | 2 | lhs, rhs | ops.md:1823 (§6.18) — complex exact-byte `t=add(mul(c,c),mul(d,d)); cmake(div(add(mul(a,c),mul(b,d)),t), div(sub(mul(… | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `ceil` | 1 | operand | ops.md:253 (§2.7) — rounding round toward +∞ | meaning-only |
| `cexp` | 1 | operand | ops.md:1826 (§6.18) — complex ULP/tolerance `ea=exp(a); cmake(mul(ea,cos(b)), mul(ea,sin(b)))` | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `cim` | 1 | operand | ops.md:1811 (§6.18) — imag lane → real `element_map` reading lane 1 (stride-2 view) of the interleaved storage | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `clog` | 1 | operand | ops.md:1827 (§6.18) — complex ULP/tolerance `cmake(log(cabs(z)), carg(z))` (principal; imag ∈ [−π, +π], both endpoint… | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `clz` | 1 | operand | ops.md:274 (§2.7) — bitwise count leading zeros (integer only, atom) | meaning-only |
| `cmake` | 2 | real, imag | ops.md:1809 (§6.18) — complex from (re,im) `element_map` writing lane 0 = `input(0)`, lane 1 = `input(1)` into the in… | clause: §6.18 table row (lane 0 = input(0), lane 1 = input(1)) |
| `cmp_eq` | 2 | lhs, rhs | ops.md:246 (§2.7) — comparison `a==b ? 1:0` (`bool` mask); any-NaN → 0 | formula |
| `cmp_ge` | 2 | lhs, rhs | ops.md:251 (§2.7) — comparison `a>=b ? 1:0` (`bool` mask); false on NaN | formula |
| `cmp_gt` | 2 | lhs, rhs | ops.md:250 (§2.7) — comparison `a>b ? 1:0` (`bool` mask); false on NaN | formula |
| `cmp_le` | 2 | lhs, rhs | ops.md:249 (§2.7) — comparison `a<=b ? 1:0` (`bool` mask); false on NaN; `-0.0<=+0.0` true | formula |
| `cmp_lt` | 2 | lhs, rhs | ops.md:248 (§2.7) — comparison `a<b ? 1:0` (`bool` mask); false on NaN | formula |
| `cmp_ne` | 2 | lhs, rhs | ops.md:247 (§2.7) — comparison `a!=b ? 1:0` (`bool` mask); any-NaN → 1 (isnan via `cmp_ne(x,x)`) | formula |
| `cmul` | 2 | lhs, rhs | ops.md:1822 (§6.18) — complex exact-byte `cmake(sub(mul(a,c), mul(b,d)), add(mul(a,d), mul(b,c)))` + Annex-G recovery… | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `cneg` | 1 | operand | ops.md:1820 (§6.18) — complex exact-byte `cmake(neg(a), neg(b))` | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `copysign` | 2 | lhs, rhs | ops.md:265 (§2.7) — binary_math magnitude of `a`, sign bit of `b` (raw-bit) | formula |
| `cos` | 1 | operand | ops.md:259 (§2.7) — transcendental `cos x` (declared-ULP atom) | formula |
| `cosh` | 1 | operand | ops.md:1273 (§6.13) — transcendental ✓ `div(add(exp(x), exp(neg(x))), const(2))` (refinement-permitted: overflow-safe… | formula |
| `cpow` | 2 | lhs, rhs | ops.md:1829 (§6.18) — complex ULP/tolerance `cexp(cmul(w, clog(z)))` (principal) | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `cre` | 1 | operand | ops.md:1810 (§6.18) — real lane → real `element_map` reading lane 0 (stride-2 view) of the interleaved storage | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `csqrt` | 1 | operand | ops.md:1828 (§6.18) — complex ULP/tolerance `m=cabs(z); cmake(sqrt(div(add(m,a),const(2))), mul(copysign(const(1),b),… | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `csub` | 2 | lhs, rhs | ops.md:1819 (§6.18) — complex exact-byte `cmake(sub(a,c), sub(b,d))` | clause: §6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z") |
| `ctz` | 1 | operand | ops.md:275 (§2.7) — bitwise count trailing zeros (integer only, atom) | meaning-only |
| `cummax` | 1 | operand | ops.md:1312 (§6.13) — scan — `prefix_scan(monoid=max, inclusive)` | meaning-only |
| `cumprod` | 1 | operand | ops.md:1311 (§6.13) — scan — `prefix_scan(monoid=prod, inclusive)` | meaning-only |
| `cumsum` | 1 | operand | ops.md:1310 (§6.13) — scan — `prefix_scan(monoid=sum, inclusive)` | meaning-only |
| `div` | 2 | lhs, rhs | ops.md:242 (§2.7) — arithmetic `a/b`; IEEE-754 **float only** | formula |
| `element_map` | n (variadic) | operand* | ops.md:276 (§2.7) — access-primitive per-element scalar body, broadcast reads via stride-0 | clause: KISS-OPS-6.11-0001 |
| `embedding` | 2 | data, index | ops.md:1316 (§6.13) — gather_scatter — `gather(oob=zero-fill)` with a 1-D index operand | clause: §6.13 row + KISS-OPS-6.11-0012 |
| `erf` | 1 | operand | ops.md:261 (§2.7) — transcendental Gauss error function (special-function atom) | meaning-only |
| `erfc` | 1 | operand | ops.md:1280 (§6.13) — transcendental — `sub(const(1), erf(x))` | formula |
| `exp` | 1 | operand | ops.md:256 (§2.7) — transcendental `e^x` (declared-ULP atom) | formula |
| `exp2` | 1 | operand | ops.md:1265 (§6.13) — transcendental — `exp(mul(x, const(ln2)))` | formula |
| `expm1` | 1 | operand | ops.md:1266 (§6.13) — transcendental ✓ `sub(exp(x), const(1))` | formula |
| `floor` | 1 | operand | ops.md:252 (§2.7) — rounding round toward −∞ | meaning-only |
| `fmax_ieee` | 2 | lhs, rhs | ops.md:1263 (§6.13) — minmax — `select(cmp_ne(a,a), b, select(cmp_ne(b,b), a, select(cmp_ge(a,b), a, b)))` | formula |
| `fmin_ieee` | 2 | lhs, rhs | ops.md:1264 (§6.13) — minmax — `select(cmp_ne(a,a), b, select(cmp_ne(b,b), a, select(cmp_le(a,b), a, b)))` | formula |
| `frac` | 1 | operand | ops.md:1258 (§6.13) — rounding — `sub(x, trunc(x))` | formula |
| `gather` | 2..3 | data, index, base? | ops.md:279 (§2.7) — access-primitive data-dependent indexed read; OOB `{skip,clamp,zero-fill}` | clause: KISS-OPS-6.11-0012, -0015 |
| `gelu` | 1 | operand | ops.md:1286 (§6.13) — activation — `mul(mul(const(0.5), x), add(const(1), erf(div(x, const(sqrt2)))))` | formula |
| `gelu_tanh` | 1 | operand | ops.md:1287 (§6.13) — activation — `mul(mul(const(0.5), x), add(const(1), tanh(mul(const(sqrt(2/pi)), add(x, mul(cons… | formula |
| `hypot` | 2 | lhs, rhs | ops.md:1289 (§6.13) — binary_math ✓ `sqrt(add(sqr(a), sqr(b)))` | formula |
| `im2col` | 1 | operand | ops.md:1318 (§6.13) — shape — closed-form structured `gather` mapping each output element `(patch, tap)` to the sourc… | meaning-only |
| `index_select` | 2 | data, index | ops.md:1315 (§6.13) — gather_scatter — `gather(oob=skip)` with a 1-D index operand | clause: §6.13 row + KISS-OPS-6.11-0012 |
| `layer_norm` | 3 | operand, scale, shift | ops.md:1309 (§6.13) — normalization — `mu=reduce_mean(input(0)); v=reduce_var(input(0)); out=add(mul(mul(sub(input(0)… | clause: §6.13 operand-ordering convention |
| `ldexp` | 2 | operand, exponent | ops.md:1292 (§6.13) — binary_math ✓ `mul(a, exp2(b))` (reference; exact scaling `a·2^b` for integer `b`, see §6.13-00… | formula |
| `lgamma` | 1 | operand | ops.md:263 (§2.7) — transcendental `ln Γ(x) ` (special-function atom) | formula |
| `log` | 1 | operand | ops.md:257 (§2.7) — transcendental `ln x` (declared-ULP atom) | formula |
| `log10` | 1 | operand | ops.md:1268 (§6.13) — transcendental — `div(log(x), const(ln10))` | formula |
| `log1p` | 1 | operand | ops.md:1269 (§6.13) — transcendental ✓ `log(add(const(1), x))` | formula |
| `log2` | 1 | operand | ops.md:1267 (§6.13) — transcendental — `div(log(x), const(ln2))` | formula |
| `log_softmax` | 1 | operand | ops.md:1307 (§6.13) — normalization — `sub(x, logsumexp(x))` | formula |
| `logical_and` | 2 | lhs, rhs | ops.md:1293 (§6.13) — logical — `mul(cmp_ne(a, const(0)), cmp_ne(b, const(0)))` | formula |
| `logical_not` | 1 | operand | ops.md:1295 (§6.13) — logical — `cmp_eq(x, const(0))` | formula |
| `logical_or` | 2 | lhs, rhs | ops.md:1294 (§6.13) — logical — `min_prop(const(1), add(cmp_ne(a,const(0)), cmp_ne(b,const(0))))` | formula |
| `logsumexp` | 1 | operand | ops.md:1300 (§6.13) — reduction — `m=reduce(max,x); out=add(m, log(reduce(sum, exp(sub(x,m)))))` | formula |
| `matmul` | 2 | lhs, rhs | ops.md:1305 (§6.13) — contraction — `reduce(sum, axis=K) of element_map(mul(input(0), input(1)))`, where over iterati… | clause: §6.13 operand-ordering convention (ops.md:%d) |
| `max_pool` | 1 | operand | ops.md:1314 (§6.13) — window — `reduce(max)` over the window axis of the pooled view (OOB taps skipped; NaN propagate… | meaning-only |
| `max_prop` | 2 | lhs, rhs | ops.md:1261 (§6.13) — minmax — `select(cmp_ne(a,a), a, select(cmp_ne(b,b), b, select(cmp_ge(a,b), a, b)))` | formula |
| `min_prop` | 2 | lhs, rhs | ops.md:1262 (§6.13) — minmax — `select(cmp_ne(a,a), a, select(cmp_ne(b,b), b, select(cmp_le(a,b), a, b)))` | formula |
| `mish` | 1 | operand | ops.md:1285 (§6.13) — activation ✓ `mul(x, tanh(softplus(x)))` (refinement-permitted: inherits the overflow-safe `tan… | formula |
| `mul` | 2 | lhs, rhs | ops.md:241 (§2.7) — arithmetic `a*b`; IEEE-754 float / wrapping int | formula |
| `neg` | 1 | operand | ops.md:243 (§2.7) — arithmetic `-x`; flips sign bit, `-(-0.0)=+0.0`, NaN propagates | formula |
| `nextafter` | 2 | lhs, rhs | ops.md:266 (§2.7) — binary_math next representable after `a` toward `b`, dtype's own lattice | formula |
| `popcount` | 1 | operand | ops.md:273 (§2.7) — bitwise set-bit count (integer only, atom) | meaning-only |
| `pow` | 2 | lhs, rhs | ops.md:1288 (§6.13) — binary_math ✓ `exp(mul(b, log(a)))` for finite `a>0`, `a≠1`, finite nonzero `b` only; every oth… | formula |
| `prefix_scan` | 1 | operand | ops.md:278 (§2.7) — access-primitive inclusive/exclusive running monoid fold, length-preserving | meaning-only |
| `recip` | 1 | operand | ops.md:1256 (§6.13) — arithmetic — `div(const(1), x)` | formula |
| `reduce` | 1 | operand | ops.md:277 (§2.7) — access-primitive associative-monoid fold over an axis set (keepdim result) | meaning-only |
| `reduce_mean` | 1 | operand | ops.md:1296 (§6.13) — reduction — `div(reduce(sum, x), reduced_count)` (divisor is the product of extents over **all*… | formula |
| `reduce_norm2` | 1 | operand | ops.md:1297 (§6.13) — reduction — `sqrt(reduce(sum, sqr(x)))` | formula |
| `reduce_std` | 1 | operand | ops.md:1299 (§6.13) — reduction — `sqrt(reduce_var(x))` | formula |
| `reduce_var` | 1 | operand | ops.md:1298 (§6.13) — reduction ✓ `mu=reduce_mean(x); out=reduce_mean(sqr(sub(x, mu)))` (two-pass, population divisor… | formula |
| `relu` | 1 | operand | ops.md:1282 (§6.13) — activation — `select(cmp_lt(x, const(0)), const(0), x)` | formula |
| `rem_floor` | 2 | lhs, rhs | ops.md:1290 (§6.13) — binary_math — `sub(a, mul(floor(div(a,b)), b))` | formula |
| `rem_trunc` | 2 | lhs, rhs | ops.md:1291 (§6.13) — binary_math — `sub(a, mul(trunc(div(a,b)), b))` | formula |
| `rms_norm` | 2 | operand, scale | ops.md:1308 (§6.13) — normalization — `ms=reduce_mean(sqr(input(0))); out=mul(mul(input(0), rsqrt(add(ms, param(0))))… | clause: §6.13 operand-ordering convention |
| `round_even` | 1 | operand | ops.md:255 (§2.7) — rounding round to nearest, ties to even | meaning-only |
| `rsqrt` | 1 | operand | ops.md:1257 (§6.13) — transcendental — `div(const(1), sqrt(x))` | formula |
| `scatter` | 3 | dest, index, updates | ops.md:280 (§2.7) — access-primitive data-dependent indexed write; combine `{assign,atomic-add,atomic-max,atomic-mi… | clause: KISS-OPS-6.11-0016, -0017 |
| `scatter_add` | 3 | dest, index, updates | ops.md:1317 (§6.13) — gather_scatter — `scatter(combine=atomic-add, oob=skip)` | clause: §6.13 row + KISS-OPS-6.11-0016, -0017 |
| `select` | 3 | cond, on_true, on_false | ops.md:245 (§2.7) — select `cond!=0 ? a : b`; raw-bit move, order `(cond,a,b)` | clause: KISS-OPS-6.5-0001 |
| `shl` | 2 | lhs, rhs | ops.md:271 (§2.7) — bitwise `a << b` (integer only) | formula |
| `shr` | 2 | lhs, rhs | ops.md:272 (§2.7) — bitwise `a >> b`; arithmetic on signed, logical on unsigned | formula |
| `sigmoid` | 1 | operand | ops.md:1281 (§6.13) — activation — `recip(add(const(1), exp(neg(x))))` | formula |
| `sign` | 1 | operand | ops.md:1259 (§6.13) — arithmetic — `select(cmp_gt(x,const(0)), const(1), select(cmp_lt(x,const(0)), const(-1), const(… | formula |
| `silu` | 1 | operand | ops.md:1283 (§6.13) — activation ✓ `mul(x, sigmoid(x))` (refinement-permitted: overflow-safe evaluation of the `exp` … | formula |
| `sin` | 1 | operand | ops.md:258 (§2.7) — transcendental `sin x` (declared-ULP atom) | formula |
| `sinh` | 1 | operand | ops.md:1272 (§6.13) — transcendental ✓ `div(sub(exp(x), exp(neg(x))), const(2))` (refinement-permitted: near-zero-acc… | formula |
| `softmax` | 1 | operand | ops.md:1306 (§6.13) — normalization — `m=reduce(max,x); e=exp(sub(x,m)); s=reduce(sum,e); out=div(e,s)` | formula |
| `softplus` | 1 | operand | ops.md:1284 (§6.13) — activation ✓ `log(add(const(1), exp(x)))` (refinement-permitted: an overflow-safe form, e.g. `m… | formula |
| `sort_network` | 1 | operand | ops.md:281 (§2.7) — access-primitive stable per-row permutation; exposes permuted values and original-index vector | meaning-only |
| `sqr` | 1 | operand | ops.md:1255 (§6.13) — arithmetic — `mul(x, x)` | formula |
| `sqrt` | 1 | operand | ops.md:260 (§2.7) — transcendental √x (IEEE correctly-rounded where guaranteed, else declared-ULP) | meaning-only |
| `step` | 1 | operand | ops.md:1260 (§6.13) — activation — `select(cmp_gt(x, const(0)), const(1), const(0))` | formula |
| `sub` | 2 | lhs, rhs | ops.md:240 (§2.7) — arithmetic `a-b`; IEEE-754 float / wrapping int | formula |
| `tan` | 1 | operand | ops.md:1270 (§6.13) — transcendental — `div(sin(x), cos(x))` | formula |
| `tanh` | 1 | operand | ops.md:1271 (§6.13) — transcendental ✓ `div(sub(exp(x), exp(neg(x))), add(exp(x), exp(neg(x))))` (refinement-permitte… | formula |
| `trunc` | 1 | operand | ops.md:254 (§2.7) — rounding round toward zero | meaning-only |

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
3. ~~Unblocks #263's residual (`une`/`bin`/`ter` follows from the count).~~ **Struck: `op_family` no
   longer exists; see the retraction at the top.**
4. Lands under the normative-text rule: PM + KISS + every affected agent (~~Unpopped derives
   `op_family`~~ struck; Fuel and Baracuda read the manifest).

## Not decided here

Whether `lhs`/`rhs` should be `a`/`b` (the spelling §6.13 bodies use) — the proposal prefers
`lhs`/`rhs` because `matmul` already pins them. Any broadcast fact (ruled out).
