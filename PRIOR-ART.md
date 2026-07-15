# KISS — prior art, and the wedge

This document is **informative**. It exists because the suite did not have one, and that was
its most serious omission: across ~1MB of specification, `README.md`, and `DESIGN.md` — whose
stated job is recording design rationale — there was **not one mention of any incumbent**.
KISS had assumed its wedge rather than argued it.

The purpose here is not to sell KISS. It is to let a skeptical engineer decide. Where an
incumbent already solves a thing, this document says so and KISS should stop claiming it.
That outcome is more valuable than a flattering one: **six of the nine sub-standards are, in
whole or in part, occupied ground**, and knowing which six is what makes the remainder
credible.

> **Status.** Compiled 2026-07 against the then-current published specs. Every claim below is
> sourced to a primary document. Where KISS is behind an incumbent, that is stated plainly.

---

## 1. The five claims under test

KISS asserts five pieces of novelty. This document tests each against every incumbent:

1. **Kernel contract** — a vendor-neutral 7-section document saying what a kernel computes and
   exactly how to call it, *including launch geometry*.
2. **Specialization-cell identity** (`structure_key`) — a token two parties compute identically,
   so a provider's kernel and a consumer's need join by identity.
3. **Pinned per-op numeric semantics** + a determinism/fidelity class per op.
4. **Provision protocol** — ask by identity, receive `{artifact, contract}`, provider builds on
   a cache miss.
5. **Mechanically-checked conformance** — every clause 1:1 to a test.

## 2. The map

| Incumbent | 1 contract | 2 identity | 3 numerics | 4 provision | 5 conformance |
|---|---|---|---|---|---|
| StableHLO / OpenXLA / PJRT | not solved | **not solved** | partial | partial | partial |
| ONNX / ONNX Runtime (EPs) | partial | **not solved** | not solved | partial | partial |
| Triton / MLIR | partial | partial | not solved | partial | n/a |
| PyTorch (torch.library, Inductor, AOTInductor) | partial | **not solved** | partial | partial | partial |
| DLPack / Python array API | not solved | partial (dtypes: **solved**) | not solved | not solved | partial |
| Khronos SPIR-V / IREE | partial | **not solved** | partial | not solved | not solved |
| cuBLAS / cuDNN / CUTLASS / oneDNN / CK | not solved | partial | partial | not solved | not solved |
| IEEE-754 / Khronos ULP tables / CTS | partial | n/a | partial | n/a | **already solved** |

Read the columns, not the rows. **Claim 2 is the only one no incumbent solves.** Claim 5 —
the differentiator `DESIGN.md` leads with — is the one an incumbent solved twenty years ago.

## 3. What survives: a cache key is not a join key

The wedge is narrower than any of the five claims as written, and it is this:

> **A canonical, total function from a *need* to a byte-comparable token — computable by a
> party that possesses neither the kernel, nor the vendor's runtime, nor the artifact the
> token names.**

That is KISS-Classify §6.6-0011, §6.6-0012, and §6.7. Nothing else.

The obvious objection is that specialization-cell identity is thoroughly preempted, and it is:
`cublasLtMatmulAlgo_t`, cuDNN's engine-config + knob tuple, oneDNN's `primitive_desc`,
Composable Kernel's template-parameter tuple, Triton's
`sha256(fn.cache_key + attrs + sorted_sig + constants_key + backend + options + env)`, and
Inductor's `FXGraphCache` key are all specialization-cell identities. Five organizations built
five of them. KISS having a sixth would be worth nothing.

**But every one of those is a cache key, and a cache key's correctness requirement is the exact
inverse of a join key's.**

For a cache, a false hit is a silent wrong answer and a false miss costs only a rebuild. The
dominant strategy is therefore to **over-specify**: hash the source text, the compiler binary,
the environment, the config, the library version. This is not sloppiness — it is correct
engineering, and every party arrived at it deliberately. Triton hashes source text and the
backend binary *on purpose*. Inductor hashes the torch version and inductor config *by design*,
which is exactly why its key can never cross a version boundary. cuBLASLt's algo is serializable
but restorable only on the same cuBLAS version. These are not gaps awaiting a patch.

A **join** key needs the opposite. Two parties sharing no code, no version, and no vendor must
converge on the same bytes. That requires the key to be *under*-specified relative to the build,
deriving its correctness from being a **predicate over the need** rather than a **digest of the
build**.

The structural point is the direction of information flow:

| | Cache key | Join key |
|---|---|---|
| Computes | `artifact → key` | `invocation → key` |
| When | after the build | before any artifact exists |
| By whom | the party owning the artifact | a party who may never see the build |
| Wants | over-specification (avoid false hits) | under-specification (enable convergence) |

**You cannot refactor a cache key into a join key; the arrow points the other way.** This is why
five independent inventions produced five unjoinable keys, and why a sixth vendor will produce a
sixth.

### 3.1 The corollary the repo has never stated: the build matrix

A cache key **cannot express a build matrix**, because it requires the artifact to already
exist. You cannot enumerate hashes of source text you have not written.

Yet every serious provider *has* a build matrix: Composable Kernel's `GPU_TARGETS`/`DTYPES`
instance registry, CUTLASS's tile/warp/stage template tuples, cuBLASLt's heuristic table,
Triton's autotune config lists. Each is a set of cells, each is spelled in a private dialect,
and **none is publishable**.

An extent-free, need-derived, byte-comparable token is the only object that is simultaneously
(a) enumerable by a provider ahead of any invocation, and (b) derivable by a consumer at
runtime from an invocation the provider never sees. Extent-freeness (§6.6-0003) is not a
nicety — it is precisely what makes the cell set *finite*, and therefore *publishable*, and it
is why the key must be a **class token, not a fingerprint**.

**The first buyer is therefore not the consumer. It is the provider who wants to publish what he
built without publishing how he built it** — the thing cuBLAS/cuDNN/oneDNN have spent fifteen
years declining to do. A cell token, unlike a launch-geometry declaration, does not ask them to
surrender anything.

This also disposes of the sharpest objection on the table (§5.1): the XLA fusion-barrier
argument is an argument against *claim 1*, not against this. A `structure_key` does not create
a kernel boundary; it **names one that already exists**. XLA itself fingerprints HLO modules for
its compile cache — a key over a fused region. Claim 1 asks a vendor to surrender optimization
freedom; claim 2 asks him to publish a coverage list. Only one of those has fifteen years of
revealed refusal behind it.

## 4. What does not survive

Stated plainly, so KISS stops claiming it:

**Claim 5 (mechanical conformance) is not a differentiator — it is the weakest claim in the
suite.** The Khronos CTS has done clause-to-test conformance for vendor-neutral compute for
two decades, with an adopters program and a trademark to back it. `OpenCL-CTS`'s
`math_brute_force` tests numeric accuracy against exhaustive references across the entire fp32
input space. KISS currently has **29 of 853 clauses backed by an executable test (3.4%)**, and
the gate that claimed otherwise compared markdown to markdown (see `conformance/UNBACKED.tsv`
and `DESIGN.md` §1.4). Leading with this claim invites the comparison KISS loses worst.

**Claim 3 (numerics) is largely occupied, and StableHLO is ahead of KISS in places.**
- `ResultAccuracyAttr` ships today with `atol` / `rtol` / **`ulps`** across ~12 transcendental
  ops — essentially the same op list as KISS-Ops §6.8, which does not cite it.
- `DotAlgorithm` pins `lhs_precision_type` / `rhs_precision_type` / `accumulation_type` /
  `num_primitive_operations` / `allow_imprecise_accumulation`. Meanwhile **KISS-OPS-6.0-0004
  classifies matmul as order-invariant/nondeterministic and justifies it on the grounds that
  "neither a canonical reduction order nor a canonical accumulator width is pinned by
  KISS-Ops."** StableHLO pins accumulator width. On matmul numerics, KISS declares defeat on a
  problem StableHLO partially solved.
- ~140 of StableHLO's 152 ops have written semantics *plus an executable reference interpreter*.

  What genuinely survives here is narrow and worth keeping: **no incumbent carries a per-op,
  machine-readable determinism/fidelity enum that a harness reads to *select* a comparator.**
  StableHLO says "the order of reductions is implementation-defined" — which *is* KISS's
  order-invariant class — but as English prose on one op, not a field. Mechanical comparator
  selection (conform.md §2.5, "the comparator is selected, never chosen") is the delta. That is
  much less than "KISS pins per-op numeric semantics."

**Claim 4 (provision) is shape-preempted by PJRT.** `Compile(module) → PjRtLoadedExecutable` is
already "hand me a description, you build it, I execute it," proven across a C ABI and a process
boundary at production scale. KISS's version differs by returning `{artifact, *contract*}` and
by asking at *kernel* rather than *program* granularity — real differences, but KISS-Synth's 130
clauses are not carrying 130 clauses' worth of novelty.

**Claim 1 (contract) is partially preempted, and its live half is broken.** The Interface section
is preempted (SPIR-V `OpFunctionParameter`, Triton's AOT headers, LibTorch's stable ABI). The
Dispatch section is genuinely unoccupied by any *standard* — but see §5.1, and note that IREE's
`workgroup_count` region is a strictly more expressive solution to the same problem, and that
Dispatch is currently **unwritable in its own grammar** (11 confirmed hard blocks; the §6.6-0006
expression grammar has no array subscript, no thread-index symbol, no modulo, and no tuple
constructor, so all five mandated derivation fields cannot be spelled at any rank — including
for the spec's own rank-1 `add` example).

**KISS-Classify's dtype table is preempted by DLPack, and currently wrong.** DLPack carries 19
open-ended `DLDataTypeCode`s and grew specifically to absorb `kDLFloat4_e2m1fn`,
`kDLFloat8_e8m0fnu`, and the FP6 pair — exactly the formats issue #32 is about. Worse:
KISS's `e4m3` is defined as max-finite ±448, no infinities, single NaN — those are DLPack's
`kDLFloat8_e4m3fn` semantics, and DLPack *also* has a distinct inf-carrying `kDLFloat8_e4m3`.
**A bridge author mapping by name ships a silently wrong dtype.** And KISS-CLASSIFY-6.1-0001's
closed twenty-token set forecloses the exact mechanism — frozen container, extensible leaves —
that let both DLPack and Arrow's C Data Interface survive 2023–2026.

## 5. The objections KISS must answer

### 5.1 The fusion-barrier objection (against claim 1)

XLA hides launch geometry **deliberately**. Whole-program compilation lets it fuse across op
boundaries; a per-kernel contract that pins launch geometry is a fusion barrier. XLA FFI is the
proof: the handler computes its own geometry internally (`const int64_t block_dim = 64; const
int64_t grid_dim = 2048 / block_dim;` lives *inside* the handler, invisible to XLA), and the op
is documented as "an implementation-defined operation `call_target_name`."

**KISS-CONTRACT-6.6-0001's flat prohibition on declaring geometry "provider-internal" is
precisely the position XLA considered and rejected.** Nothing in `spec/contract.md` answers this.
An XLA engineer will raise it in the first meeting. KISS needs an answer, and "the consumer must
have this fact" is not one — XLA's counter is that the consumer must *not* have it, so that the
compiler may choose it.

### 5.2 The ULP-ceiling defect (against claim 3) — **fix before showing anyone**

`KISS-OPS-6.8-0001` sets maximum ULP ceilings and mandates that "KISS-Conform MUST reject a
declared ULP exceeding the ceiling." Against what a **conformant** OpenCL fp32 device is
*permitted*:

| Atom | OpenCL permits | KISS ceiling | Effect |
|---|---|---|---|
| `atan` | ≤ 5 ULP | **4 ULP** | conformant device rejected |
| `atan2` | ≤ 6 ULP | **4 ULP** | conformant device rejected |
| `erf` | ≤ 16 ULP | **4 ULP** | conformant device rejected |
| `sqrt` (non-CR) | ≤ 3 ULP | **2 ULP** | conformant device rejected |
| `lgamma` | unbounded | **8 ULP** | bounds an unbounded function |
| `exp`, `log` | ≤ 3 ULP | 4 ULP | fine |
| `sin`, `cos` | ≤ 4 ULP | 4 ULP | fine (fp32 OpenCL only — see below) |

**Five of eight rows are tighter than the incumbent floor.** As written, KISS-Conform *rejects a
truthful Khronos-conformant vendor and admits only one that misreports its numbers* — the exact
inversion of the clause's purpose. On Vulkan it is worse: `Atan` is 4096 ULP, and `Sin`/`Cos`
carry no ULP bound at all (they are specified as absolute error within [−π, π] and are undefined
outside it), so KISS's 4-ULP `sin` ceiling is not tight — it is **unsatisfiable in principle**.

This is findable by diffing one table against a document that has been public since 2008. It is a
90-second objection that ends the meeting before anything else is read.

## 6. Could KISS ride on an incumbent instead?

The honest answer is **partly, and it should**:

- **Dtypes:** adopt DLPack's `DLDataType` (code, bits, lanes) rather than a closed 20-token set.
  It already has the sub-byte and MX element formats, it is the de facto standard, and it solves
  issue #32 by adoption instead of by design.
- **Numerics:** cite the Khronos ULP tables as the floor and StableHLO's `ResultAccuracyAttr` as
  the per-op accuracy vocabulary. Keep only the determinism/fidelity enum, which is genuinely new.
- **Artifact:** SPIR-V where the target allows; `artifact_format_tag` becomes a registry pointer,
  not a KISS invention (it currently has no registry at all — a confirmed hard block).
- **Conformance methodology:** copy Khronos CTS structure rather than claiming novelty.

That leaves KISS-Classify's `structure_key` and the determinism enum standing alone — which is
the point of §3. **A standard that is one function and one enum, and that says so, is far more
adoptable than nine sub-standards that overclaim.**

## 7. Verdict

The wedge is real, and it is one function.

The rest of the suite is not worthless — a contract format and a provision protocol are useful
*plumbing* around that function — but they are plumbing, not novelty, and they should be
presented and versioned as such. The current framing, in which nine sub-standards are presented
as nine contributions, invites nine comparisons, and KISS loses six of them.

The recommendation this analysis supports:

1. **Fix §6.8's ULP ceilings** (§5.2). Non-negotiable, and cheap.
2. **Fix the dtype table** — adopt DLPack's codes, or at minimum disambiguate `e4m3` and open
   the set.
3. **Lead with claim 2 alone.** Retitle the pitch: *a publishable, joinable specialization-cell
   identity*. Explain the cache-key/join-key distinction (§3) in the README's first paragraph —
   it is the single most convincing thing the project has, and it appears nowhere today.
4. **Stop leading with claim 5** until `UNBACKED.tsv` is small. It invites the Khronos comparison.
5. **Answer the fusion-barrier objection** (§5.1), or scope Dispatch to targets where launch
   geometry is genuinely the caller's (and note it is unwritable today regardless).
6. **Demote what is preempted** to "profile over an existing standard" rather than "sub-standard."

---

*Sources: StableHLO spec + `StablehloAttrs.td` + compatibility policy; OpenCL C 3.0 §7.4 relative
error table (cross-checked against Intel's CUDA-vs-OpenCL precision comparison); Vulkan precision
appendix; DLPack `dlpack.h`; PJRT C API; XLA FFI docs; ONNX opset schemas + EP ABI; Triton
`JITFunction` cache key; PyTorch `FXGraphCache` / AOTInductor; SPIR-V + OpenCL SPIR-V Env; IREE
`workgroup_count`; OCP MX v1.0; cuBLASLt / cuDNN backend / CUTLASS / oneDNN / Composable Kernel
documentation.*
