# Options note: what a kernel's outside view carries across the seam (board 449)

**Informative. No normative text. Nothing here is built. Every form below that changes bytes goes to CireSnave.**
Measured at `origin/main` `8c14914`, 2026-11-16, by reading `spec/contract.md`, `spec/ops.md`, `spec/classify.md` and
`spec/synth.md` at that ref, plus the consumer answers of Fuel and FireLine as the PM relayed them (read, not re-measured).

CireSnave ruled board 449, verbatim: *"449: Cost has to do with many different things from speed to accuracy to precision to
reproducibility and likely others.  The internals of a kernel are not what KISS (and thus FireLine) should specify or describe
in most cases.  Each individual metric I listed above about the outside view of the kernel is what should be passed across
seams."* So the earlier class-and-terms proposal (flops, bytes read, bytes written) is withdrawn: those describe how a kernel
works inside. The golden `cost = 1` and clause 6.7-0006 stay as they are until this note is ruled.

**The test used below.** A metric belongs on the seam if a caller can observe it from outside the call and a consumer reads
it to choose or place a kernel. A metric that only explains how the kernel arrives at it does not.

## 1. What the contract already carries about the outside view

| Outside-view metric | Field and clause | Form today | Declared or measured? |
|---|---|---|---|
| Accuracy | Guarantees `per_backend_ulp_tiers` (6.8-0002, `contract.md:1227`) against a named `reference_function` | per target backend, a tagged quantity with at least one of `max_ulp`, `max_relative`, `max_absolute`; no tier means no finite bound (`unbounded`, `unaudited`, 6.8-0010) | not stated |
| Accuracy that depends on the call | 6.8-0012 (`contract.md:1278`) | for accumulator reductions and the float `atomic-add` cell, a band `k(S,A) · N · eps_A · |max partial sum|`, where `N` is the runtime extent: already a function of the call's shape | not stated |
| Accuracy, summarised | Capabilities `precision_class` (6.7-0005, `contract.md:1198`; token set KISS-OPS-6.8-0007, derived by 6.8-0008) | one of `strict`, `correctly-rounded`, `bounded-ulp`, `bounded-tolerance`, `unbounded` | derived |
| Trust in the accuracy claim | Guarantees `audited_status` (6.8-0008 to -0010) | derived from the reference and the bound; `audited` or `unaudited` | derived |
| Precision (numeric resolution) | Capabilities `supported_dtype_set` (6.7-0003, `contract.md:1185`), Guarantees `math_fidelity` (6.8-0004, 6.8-0004a), `accumulation_type` (6.8-0011, `contract.md:1267`, the same dtype as the Classify key's `<acc>`, KISS-CLASSIFY-6.7-0006) | closed token sets | declared |
| Reproducibility | `determinism_class` (6.7-0004, 6.8-0003, from KISS-OPS-6.0-0001) and `bit_stability` (6.8-0005, 6.8-0013) | `exact-byte`, `ULP/tolerance`, `order-invariant/nondeterministic`; and `portable`, `same-hardware`, `none` | declared |
| Speed | Capabilities `cost` (6.7-0006, `contract.md:1209`) | a cost class plus expressions over the launch-scalar symbols; **no unit is named**; the golden is the bare scalar `cost = 1` (`contract.md:2127`) | `cost_provenance` (6.8-0006): `declared` or `measured` |
| Edge cases | NaN, signed zero, raw-bit moves (6.4-0004) | resolved from KISS-Ops, never restated | not applicable |
| Layout it can take | Capabilities `awkward_layout_strategy`, `in_place_eligible_variants` (6.7-0003) | declared families | declared |
| Scratch memory | the launch scalars `ws_ptr` and `ws_bytes` (6.5-0004a, `contract.md:891`) | a byte count the **caller passes at launch** | not applicable |

Two clauses that sound related are not: KISS-Synth's "implementation-declared maximum latency bound" (`synth.md:676`) bounds
how long a provider takes to *answer a provision request*, not how long a kernel runs.

**One finding.** Only the speed metric has a provenance tag (6.8-0006). Accuracy, memory and any other number a consumer
reads has no way to say whether it was declared or measured.

## 2. What is missing against CireSnave's list

- **Speed.** The seam has a cost expression with no unit, and the golden shows a bare scalar, which 6.7-0006 says an implementation must not carry in place of a shape-parameterized expression. A
  consumer cannot read "how long does this take for my shapes".
- **Accuracy.** The tiers and the call-dependent band exist. Missing: a provenance tag. The layout of the tier field is not
  pinned. FireLine reports that fractional tier values have no encoding; the encoding primitive exists
  (6.11-0010, `<dtype>:<hex>`, `contract.md:1621`), what is unpinned is the field layout that carries it (issue #44
  closed on exactly that: "no byte layout for compound fields cost (§6.7-0006) and per_backend_ulp_tiers (§6.8-0002)").
- **Precision.** Everything exists as closed token sets. Missing: provenance only. FireLine reads "precision" as numeric
  resolution, not accuracy, and has asked CireSnave to confirm; this note follows that reading.
- **Reproducibility.** Both axes exist. Missing: provenance, and, if `same-hardware` is to be checkable by a consumer, a way to
  say which hardware counts as the same (FireLine). Fuel reads only one bit today (stable on the same hardware).
- **Others.** *Memory footprint:* Fuel names a consumer (output and working-set bytes); today the contract gives the caller
  only a workspace size it must already know. *Energy:* nobody names a consumer. *Scaling:* covered by the speed form below,
  because a function of the call's shape is a scaling statement. *Not on the seam:* run-time load (measured per device by the
  consumer) and transfer cost (a topology price the consumer computes), per Fuel.

## 3. One candidate form per metric, options for the board

**Common rule for every number (Question P).** The rule FireLine proposes is that every value is what a caller observes, with a
unit, and a provenance of `declared` or `measured`.
**A.** Each numeric metric (speed, accuracy tiers, memory) carries its own `declared` or `measured` tag. **B.** One tag for the
whole Capabilities and Guarantees pair, as 6.8-0006 does for cost today. **C.** Leave provenance on speed only.
*Recommendation: A.* B would force a record that measured its speed but only declared its accuracy to lie about one of them;
C leaves a consumer unable to tell a measured accuracy bound from a promised one. *Bytes:* one new tag per numeric field;
`cost_provenance` becomes the speed one.

**Speed (Question S).** Fuel needs a latency in nanoseconds (`u64`) per call that **differs between 4 and 4 million elements**,
measured replacing declared. FireLine proposes a **list of reference points**: each point is an invocation spelled as Interface
launch-scalar values, a `latency_ns`, and an optional `bytes_per_s`, with the target named.
**A. A latency expression.** One integer expression in nanoseconds over the call's launch-scalar symbols (for example `1200 +
n * 3 / 2`), in the grammar 6.6-0006 already pins, evaluated for the call's shapes. **B. Reference points.** FireLine's list; the
consumer finds or interpolates the point for its shapes. **C. Both:** an expression where the provider models the time, points
where it measured, the provenance saying which.
*Where Fuel and FireLine differ:* Fuel wants a function of the shapes; FireLine wants measured points because one point cannot be
read at another shape. They agree on integer nanoseconds, on provenance, and that a lone scalar is not enough. *Which covers the
picker:* A gives a definite, different value for 4 and for 4 million elements with no further rule, so two consumers read the
same number. B only does so if the standard also pins how to read between points (nearest, linear, step); without that rule two
consumers disagree on the same record. *Recommendation: A for the first version, C later.* A needs no new grammar and reuses
6.6-0006's integers (fractions are written with `*` and `/`); a provider that measured points can fit them into an expression;
points can be added later as a second, additive form with a pinned reading rule. *Bytes:* the Capabilities `cost` line changes
value and the golden `cost = 1` changes; the field should be renamed `latency_ns`, because the name `cost` is what hid the unit
(a rename is a wire change, `contract_version` moves). 6.7-0006's "cost class" would go; Question P's tag stays.

**Accuracy (Question A).** **A. Keep 6.8-0002 and 6.8-0012 as they are**, add only the provenance tag (Question P). **B. Also pin
the field layout** that carries `max_relative` and `max_absolute` using the 6.11-0010 real encoding. **C. Also let the tier
depend on the call for every kernel**, as 6.8-0012 does for accumulators, by an expression in the same grammar.
*Recommendation: A now, B in the same change as the speed form* because the speed form needs a layout for its integers and the
tiers need one for theirs; C only if a consumer names a kernel whose accuracy depends on its shape outside the accumulator
cells (none is named). *Bytes:* A adds the tag; B pins tier-field bytes that no document carries today.

**Precision (Question R).** **A. Keep the closed token sets** (`supported_dtype_set`, `math_fidelity`, `accumulation_type`) and
add nothing but the confirmation of what "precision" means. **B. Add the output dtype's resolution as a number.** **C. Fold it
into accuracy.** *Recommendation: A:* the dtype tokens already carry resolution, and a second number would have to agree with
them. *Bytes:* none.

**Reproducibility (Question D).** **A. Keep `determinism_class` and `bit_stability`.** **B. Add a hardware-equivalence token**
beside `same-hardware`, naming what counts as the same hardware (a device family, a driver generation). **C. Collapse to one
bit** (Fuel's reading today). *Recommendation: A.* Fuel reads one bit; no consumer reads a cross-hardware class; B should wait
for one that does, and C would throw away `portable`, which a consumer outside Fuel may want. *Bytes:* none under A.

**Memory footprint (Question M).** **A. A byte expression** `workspace_bytes` and `output_bytes` over the same launch-scalar
symbols, evaluated before the call, so a caller can allocate and a consumer can place. **B. One `u64`** (Fuel's reading, a single
size). **C. Leave it out;** the caller learns the workspace size at launch. *Recommendation: A for the workspace,* since the
caller must allocate it before launch and today has no declared way to learn the size; the output size is already derivable
from the op's shape rule and need not be repeated. B cannot differ between 4 and 4 million elements. *Bytes:* one new
Capabilities field; the golden gains a line.

**Energy (Question E).** **A. Not now.** **B. A nanojoule expression.** **C. A declared class.** *Recommendation: A.* Nobody
names a consumer, and measuring it needs a device counter KISS does not define.

## 4. What this note does not decide

Whether "precision" means numeric resolution (FireLine's question to CireSnave). Whether load and transfer stay off the seam
(Fuel and FireLine both say yes; CireSnave's list does not name them). The unit and rounding of any later float-valued metric.
Which of the forms above CireSnave rules: **Questions P, S, A, R, D, M and E each need his answer, and none is built before it.**

— lane: kiss
