# Data-type completeness audit, naming list and urgent-update process

**Informative. No normative text. First pass for the `sk8` event** (`rfcs/per-operand-dtype-sk8.md` §5.1).
Ruling, CireSnave 2026-11-13, board 430 Q5 (verbatim, via the PM): *"b.  A type KISS can't describe should
result in an urgent update to KISS so that it can.  KISS should support all types used by all devices that
might provide compute described via KISS standards.  The list is finite and describable."*

> **How far to trust this note.** The KISS-side facts (which tokens exist) are read from `spec/classify.md`
> §6.1-0001 and `spec/namespaces/vulkan.md` at `origin/main`. The **device-side facts** (which types each
> family exposes) are compiled from my knowledge of the Vulkan, PTX, HIP, SYCL, OpenCL and Metal documentation
> and are **not fetched in this session**. Each row carries a source to re-check and a confidence
> (**H** I am sure; **M** I believe so; **L** I do not know). Step 1 of the process in §3 is to have each
> device-family owner verify its rows against the registry. A row marked M or L is a question, not a fact.

## 1. What KISS names today (the left-hand side)

The Classify scalar dtype set is closed at twenty-seven tokens (§6.1-0001): twenty-four **usable**
(`f16 bf16 f32 f64 i8 i16 i32 i64 u8 u16 u32 u64 bool f8e4m3fn f8e5m2 f8e8m0 i4 u4 b1 c64 c128 f4e2m1 f6e2m3
f6e3m2`) and three **reserved**, recognized but with no computation semantics (`f8e4m3fnuz f8e5m2fnuz
f8e6m2`). The set is **pure storage** (§6.1-0005): a math mode such as TF32 is a precision coordinate
(`(acc+mp)`), not a dtype. Complex is `{c64, c128}` only (sk4 pin). The Vulkan namespace additionally names
`i8packed`/`u8packed`, usable only in cooperative-vector tuples (V-14).

## 2. The audit: every type of every device family in scope

Legend, KISS column: **named** = a usable token; **reserved** = recognized, not computable; **by design** =
deliberately not a dtype (with the clause); **GAP** = nothing names it. Source column is where to re-check.

### 2.1 Vulkan (`vulkan:`; registered namespace)

| Type (source) | KISS | Conf. | Note |
|---|---|---|---|
| `VkComponentTypeKHR` FLOAT16, FLOAT32, FLOAT64 (core coop-matrix) | `f16 f32 f64` named | H | |
| SINT8/16/32/64, UINT8/16/32/64 | `i8…i64`, `u8…u64` named | H | |
| BFLOAT16_KHR (`VK_KHR_shader_bfloat16`) | `bf16` named | H | `bf16` is in the V-tuple list |
| SINT8_PACKED_NV, UINT8_PACKED_NV (`VK_NV_cooperative_vector`) | `i8packed`, `u8packed` named **in the namespace only** | H | not Classify dtypes: a packed-int8 operand has no Classify token (§2.5) |
| FLOAT8_E4M3_EXT / FLOAT_E4M3_NV (alias), FLOAT8_E5M2_EXT / FLOAT_E5M2_NV | `f8e4m3fn`, `f8e5m2` named (V-10/V-11, vocabulary v3) | H | |
| Any enumerant assigned **after** `VK_HEADER_VERSION` 348 (the v5 baseline) | unknown to me | **L** | the registry must be re-read at the cut; this is exactly what the marker (rfc §5.1) is for |
| Shader capability bits (`shaderInt8/16/64`, `shaderFloat16/64`, storage 8/16-bit) | capabilities, `arith`/`st` fields (V-6, V-15) | H | not types |
| `VkFormat` (image/buffer-view texel formats, including packed and block-compressed) | **scope question** | H | storage formats of images, not compute element types; I assume out of scope, **ruling needed** (§4 Q1) |

**Vulkan remainder at baseline 348: none known.** Every enumerant I know of is named.

### 2.2 CUDA / PTX (`cuda:`; registered namespace; tokens carry no type names)

| Type (source: PTX ISA, CUDA C headers) | KISS | Conf. | Note |
|---|---|---|---|
| `.f16`, `.bf16`, `.f32`, `.f64` | named | H | |
| `.s8 .s16 .s32 .s64`, `.u8 .u16 .u32 .u64` | named | H | |
| `.pred` | `bool` named | H | |
| `.s4`, `.u4` (wmma sub-byte), `.b1` (wmma) | `i4 u4 b1` named | H | |
| `.e4m3`, `.e5m2` (fp8, sm89+) | `f8e4m3fn`, `f8e5m2` named | H | |
| `.e2m3`, `.e3m2` (fp6), `.e2m1` (fp4), `.ue8m0` (scale) | `f6e2m3 f6e3m2 f4e2m1 f8e8m0` named | M | sm100 family; spelling of the variant to re-check |
| `.tf32` | **by design** (§6.1-0005; math mode in `(acc+mp)`) | H | storage is `f32`; confirm the by-design call (§4 Q2) |
| `.b8 .b16 .b32 .b64 .b128` (untyped bit-widths) | `b128` **GAP** if used as a data type; `.b8…b64` are bit patterns, reached by `bitcast` | M | |
| `.u128`, `.s128` (recent PTX, wide integer ops and atomics) | **GAP**: no `i128`/`u128` | **M** | candidate |
| Packed pairs: `.f16x2`, `.bf16x2`, `.e4m3x2`, `.e5m2x2`, `.e2m1x2`, `.ue8m0x2`, `.s16x2`, `.u16x2` | modelled by vector-access width (§6.5-0009), not a dtype | M | |
| `.u8x4`/`.s8x4` (`dp4a` operands) | carried as `u32`/`i32` bits | M | `bitcast`-reachable (staged, §6.21-0005) |
| CUDA C `cuComplex`/`cuDoubleComplex` | `c64`, `c128` named | H | |
| Half-precision complex (`cuda::std::complex<__half>`, cuFFT half complex) | **GAP**: no `c32` | M | sk4 pinned `{c64, c128}` and said complex-f16 would carry its own migration note |

### 2.3 HIP / ROCm (no namespace file yet)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `_Float16`, `__hip_bfloat16`, `float`, `double`, ints 8–64, `bool` | named | H | |
| FP8 **FNUZ** (E4M3FNUZ, E5M2FNUZ: MI300 / gfx94x) | **reserved** (`f8e4m3fnuz`, `f8e5m2fnuz`) | **H** | **conflicts with the ruling**: a device computes on them, so "reserved, no computation semantics" cannot stand; naming-list item 1 |
| FP8 OCP E4M3/E5M2 (newer gfx) | `f8e4m3fn`, `f8e5m2` named | M | |
| FP6, FP4 (gfx950 scaled MFMA) | `f6e2m3 f6e3m2 f4e2m1` named | M | |
| `iu4` (WMMA int4) | `i4 u4` named | M | |
| `hipFloatComplex`/`hipDoubleComplex` | `c64 c128` named | H | |

### 2.4 SYCL (`sycl:`; registered namespace)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `sycl::half`, `ext::oneapi::bfloat16`, `float`, `double`, ints, `bool` | named | H | |
| `precision::tf32` (joint matrix) | **by design** | H | as CUDA |
| Intel FP8 (`hf8`, `bf8` conversions / XMX) | likely `f8e4m3fn` / `f8e5m2`, but **which variant** each denotes is unverified | **L** | same layout-to-variant ratification the sk4 RFC §10 needed for Vulkan |
| `std::complex<float/double>` | `c64 c128` named | H | |
| 4-bit ints (joint matrix `int4`) | `i4 u4` named | M | |

### 2.5 OpenCL (no namespace file yet)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `char uchar short ushort int uint long ulong half float double` | named | H | `half`/`double` by extension |
| `bfloat16` (vendor/Khronos extensions) | `bf16` named | M | extension status to verify |
| Packed 4x8 integer dot (`cl_khr_integer_dot_product`) | carried in a 32-bit integer; the Vulkan `i8packed` analogue has no Classify token | M | |
| `bool` | not a kernel-argument type | H | |

### 2.6 Metal (no namespace file yet)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `bool char uchar short ushort int uint long ulong half float` | named | H | |
| `bfloat` (MSL 3.1+) | `bf16` named | H | |
| `double` | not supported by Metal | H | nothing to name |
| Metal 4 tensor element types (including 4-bit integers) | `i4 u4` named if present | **L** | re-read `MTLTensorDataType` |

### 2.7 Other targets worth naming in scope (first pass, mostly L)

| Family | Types KISS may lack | Conf. |
|---|---|---|
| Tenstorrent | block-float formats with a shared exponent per block (`bfp8_b`, `bfp4_b`, ...): an MX-like **encoding plus shared scale**, which sk4 §3.2 models as a quantized encoding, not a dtype | L |
| x86 (AVX512-FP16/BF16, AMX, AVX10 fp8 conversions), Arm (FEAT_FP8, SVE/SME bf16, int8/16) | fp8 variants as above; `bf16 f16` named | M |
| WebGPU/WGSL | `f16 f32 i32 u32 bool` named; packed `dot4I8Packed` as `u32` | M |
| x87 `long double`, `__float128`, POWER `ieee128` | `f80`/`f128`: not a compute type of any accelerator in scope | M, scope question (§4 Q1) |
| TPU/NPU families (bf16, int4/8, fp8) | named | L |

### 2.8 The finite remainder, the **naming list for `sk8`** (proposed)

| # | Candidate | Why | Action proposed |
|---|---|---|---|
| 1 | `f8e4m3fnuz`, `f8e5m2fnuz` | AMD MI300 computes on them (H) | **un-reserve** (usable, with computation semantics and a conversion rule); the one item the ruling forces |
| 2 | `i128`, `u128` | PTX wide-integer ops (M) | verify the PTX instructions that take them, then name or decide they are never an element dtype |
| 3 | `c32` (complex `f16`) | half complex in cuFFT/CUTLASS (M) | name with the `c32` migration note the sk4 RFC §4 reserved |
| 4 | `f8e6m2` (reserved) | no device found that computes on it (L) | keep reserved or **retire**; the ruling covers only types a device uses |
| 5 | Intel hf8/bf8 | variant unknown (L) | ratify layout to variant (as sk4 did for Vulkan FP8) |
| 6 | Block-float formats (Tenstorrent `bfp*`) | shared-exponent block formats (L) | decide encoding+scale (sk4 §3.2) vs new dtype; needs the vendor's definition |
| 7 | `tf32` | exposed as a type by CUDA and SYCL | **confirm by design**: not a dtype (`(acc+mp)` carries it), so a device reporting it is not "unnamed" |

Everything not in this table, among the rows I could populate, is already named. **The remainder is seven
items, of which one (item 1) is certain.**

## 3. The urgent-update process (proposed; each line is a proposal for CireSnave and the PM to rule on)

**Trigger (either one):**
- **A device reports a non-empty `unnamed_types` marker** (rfc §5.1) to any consumer, who forwards the raw
  identifier to KISS (an issue labelled `unnamed-type`); or
- **A registry release** adds a type: a Vulkan header release, a PTX ISA release, a ROCm/HIP release, an
  SYCL/oneAPI release, an OpenCL or Metal SDK release.

**Who watches** (named by device family; an owner I do not know is blank for the PM to fill):

| Family | Watcher |
|---|---|
| Vulkan | the Vulkan vocabulary lane (Vulkane) |
| CUDA/PTX | the CUDA emitter lane (Unpopped / Baracuda) |
| HIP, SYCL, OpenCL, Metal, other | **to be assigned**; until then the KISS lane checks the registry at each of its releases |

**How fast.** Proposed targets: **triage within 2 working days** of a trigger (is it a real type, a
layout-to-variant ambiguity, or a by-design exclusion); **a draft naming PR within 5 working days**; the
type stays **out of the key** meanwhile (the marker is the stopgap, the point of the ruling).

**Who approves.**
- The KISS lane drafts the clause, the corpus entry and the migration note.
- **The PM gates and merges and allocates the version** (the standing rule).
- **CireSnave rules only when** the type does **not** fit the generative spelling grammar
  `{class}{bits}[{variant}]` (a new class, such as a block-float), or it touches a standing ruling (as TF32
  touches the pure-storage rule).

**What a naming costs.** A new dtype token is a Classify §6.1-0001 edit (the closed set is exactly N). It
is **additive** for every key that does not mention it. It **changes the token** of every device that
already exposes the type (its omitted tuples appear), so it bumps the affected **namespace vocabulary
version** (the Vulkan rule in rfc §5.1). **Open for the PM:** whether a pure additive dtype addition also
moves `STRUCTURE_KEY_VERSION` (sk4 folded the MX additions into a bump it needed anyway; this note has not
established a standalone rule).

**Closure.** The audit (§2) is re-run, and the naming list emptied, at each schema event, and the
marker's count is reported to the PM at each gate, so "unnamed types in the wild" is a number that must
reach zero.

## 4. Questions for CireSnave (plain, each with its choices; recommendation first)

1. **What counts as "data types of a device"?**
   - (a) Element types of compute operations (matrix, vector, scalar arithmetic and storage buffers).
     **Recommended.**
   - (b) Also image/texel formats (`VkFormat`, Metal pixel formats): a much larger list, mostly irrelevant
     to compute kernels.
   - (c) Also host-only extended types (`long double`, `__float128`).
2. **TF32 and similar math modes: dtype or precision coordinate?**
   - (a) Stay a precision coordinate (`(acc+mp)`); a device exposing "TF32" is not reporting an unnamed
     type. **Recommended** (it matches the pure-storage rule).
   - (b) Add a `tf32` dtype token.
3. **FNUZ FP8:** un-reserve now (the ruling forces it; AMD computes on it)? **Recommended: yes**, or say what
   a "reserved" type means once every device type must be supported.
4. **Process owners and speed:** accept the 2-day / 5-day targets and the PM-gates, CireSnave-rules-grammar
   split in §3, or name different ones.

*— lane: kiss*
