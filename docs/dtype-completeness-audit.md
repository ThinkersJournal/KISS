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

**Updated 2026-11-26 for kiss-conformance 0.24.0 (union batch 1, KISS#848, origin/main a9ad8cf5): the set is now closed at thirty-four tokens (31 usable, 3 reserved); the seven added are `f128 i128 u128 i256 u256 f8e3m4 f8ue4m3` (§6.1-0017..-0020). Every GAP below that names one of them is RESOLVED in 0.24.0 and is marked so; the text below it describes the 27-token set that existed when the audit was read.** The Classify scalar dtype set was closed at twenty-seven tokens (§6.1-0001): twenty-four **usable**
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
| FLOAT8_UNSIGNED_E8M0_EXT, FLOAT6_E2M3_EXT, FLOAT6_E3M2_EXT, FLOAT4_E2M1_EXT (`VK_EXT_shader_ocp_microscaling_types`, ext 673, header 368) | `f8e8m0`, `f6e2m3`, `f6e3m2`, `f4e2m1` named (V-17, vocabulary v8) | H | pm-kernels ruling 2026-11-19; the four Vulkan feature bits name no `arith` capability |
| MXINT8_EXT (same extension) | **GAP**: no Classify token | H (vulkane) | a naming-list candidate (item 8 in 2.8); **DECLINED at vocabulary v8** until a KISS dtype exists (pm-kernels ruling 2026-11-19): omitted whole by V-7, reported in `unnamed_types` |
| Any other enumerant after `VK_HEADER_VERSION` 348 | none: vulkane parsed `VkComponentTypeKHR` and every `extends=` at 348 and at 368; all other rows above match | H (vulkane) | re-read at each cut |
| Shader capability bits (`shaderInt8/16/64`, `shaderFloat16/64`, storage 8/16-bit) | capabilities, `arith`/`st` fields (V-6, V-15) | H | not types |
| `VkFormat` (image/buffer-view texel formats, including packed and block-compressed) | **scope question** | H | storage formats of images, not compute element types; I assume out of scope, **ruling needed** (§4 Q1) |

**Vulkan remainder at header 368: `MXINT8_EXT` only (GAP above).** Every other enumerant I know of is named.

### 2.2 CUDA / PTX (`cuda:`; registered namespace; tokens carry no type names)

| Type (source: PTX ISA, CUDA C headers) | KISS | Conf. | Note |
|---|---|---|---|
| `.f16`, `.bf16`, `.f32`, `.f64` | named | H | |
| `.s8 .s16 .s32 .s64`, `.u8 .u16 .u32 .u64` | named | H | |
| `.pred` | `bool` named | H | |
| `.s4`, `.u4` (wmma sub-byte), `.b1` (wmma) | `i4 u4 b1` named | H | |
| `.e4m3`, `.e5m2` (fp8, sm89+) | `f8e4m3fn`, `f8e5m2` named | H | |
| `.e2m3`, `.e3m2` (fp6), `.e2m1` (fp4), `.ue8m0` (scale) | `f6e2m3 f6e3m2 f4e2m1 f8e8m0` named | M | sm100 family; unpopped recalls the spellings as matching (not read from the PTX ISA); READ in PTX ISA 9.5 §5.2.3 (2026-11-25): `ue4m3` is a 7-bit unsigned float, 4 exponent + 3 mantissa bits, no infinity, NaN only `0x7f`, held in `.b8` with the MSB zero; `ue5m3` (8-bit) and `ue8m0` are its neighbours. KISS names `f8e8m0` only: `ue4m3` (the nvfp4 block scale) and `ue5m3` are **GAP**, and a byte-moving token needs CireSnave's word (board 501) |
| `.tf32` | **by design** (§6.1-0005; math mode in `(acc+mp)`) | H | storage is `f32`; confirm the by-design call (§4 Q2) |
| `.b8 .b16 .b32 .b64 .b128` (untyped bit-widths) | `b128` **GAP** if used as a data type; `.b8…b64` are bit patterns, reached by `bitcast` | M | READ in PTX ISA 9.5 §5.2.1 Table 8 (2026-11-25): `.b128` IS listed as an untyped bit type (introduced PTX 8.3; the sm minimum differs between archived pages, unresolved). It is a bit pattern, not a numeric type, which is why `bitcast` reaches it |
| `.u128`, `.s128` (recent PTX, wide integer ops and atomics) | **PER LAYER. PTX: NOT A PTX TYPE** (a language-level `__int128` lowers to 64-bit pairs; see section 5): absent from Table 8 (signed `.s8-.s64`, unsigned `.u8-.u64`, bits `.b8-.b128`); `mul.wide`/`mad.wide` stop at 32-bit inputs; 128-bit values are built from chained `.cc` ops | **H** (read PTX ISA 9.5; instruction sections after 9.7.9.6 `lop3`, incl. `mov`/`ld`/`st`/`atom`, were NOT readable, so `.b128` loads and atomics are unchecked) | PTX layer: no GAP. **Language layer: RESOLVED in 0.24.0** (board 501 "A" and board 515: `i128`, `u128`, `i256`, `u256` are tokens, §6.1-0017; the PTX lowering stays a legal lowering) |
| Packed pairs: `.f16x2`, `.bf16x2`, `.e4m3x2`, `.e5m2x2`, `.e2m1x2`, `.ue8m0x2`, `.s16x2`, `.u16x2` | modelled by vector-access width (§6.5-0009), not a dtype | M | |
| `.u8x4`/`.s8x4` (`dp4a` operands) | carried as `u32`/`i32` bits | M | `bitcast`-reachable (staged, §6.21-0005) |
| CUDA C `cuComplex`/`cuDoubleComplex` | `c64`, `c128` named | H | |
| Half-precision complex (`cuda::std::complex<__half>`, cuFFT half complex) | **GAP**: no `c32` | M | sk4 pinned `{c64, c128}` and said complex-f16 would carry its own migration note |

### 2.3 HIP / ROCm (no namespace file yet)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `_Float16`, `__hip_bfloat16`, `float`, `double`, ints 8–64, `bool` | named | H | rockfish (vendored ROCm 7.2.4 `hip/library_types.h`, `hipDataType` enum): **corrected**: `hipDataType` has no `bool` (a C++ type only), so it is not a HIP data type here; `_Float16` is the spelling in rocblas/hipblasLt; `__hip_bfloat16` is not in the vendored headers (no `hip_bf16.h`), so that spelling is unconfirmed (`HIP_R_16BF` is the enum value) |
| FP8 **FNUZ** (E4M3FNUZ, E5M2FNUZ: MI300 / gfx94x) | **reserved** (`f8e4m3fnuz`, `f8e5m2fnuz`) | **H** (rockfish: `HIP_R_8F_E4M3_FNUZ`=1000, `E5M2_FNUZ`=1001 in the enum) | **conflicts with the ruling**: a device computes on them, so "reserved, no computation semantics" cannot stand; naming-list item 1 |
| FP8 OCP E4M3/E5M2 (newer gfx) | `f8e4m3fn`, `f8e5m2` named | M | rockfish: `HIP_R_8F_E4M3`/`E5M2` exist (type confirmed); which gfx has them is not header-checkable |
| FP6, FP4 (gfx950 scaled MFMA) | `f6e2m3 f6e3m2 f4e2m1` named | M | rockfish: `HIP_R_6F_E2M3`/`E3M2`, `HIP_R_4F_E2M1` exist (type confirmed); the gfx950 claim is not header-checkable. Also `HIP_R_8F_UE8M0` (= `f8e8m0`, named), previously missing from this table |
| `iu4` (WMMA int4) | `i4 u4` named | M | rockfish: `HIP_R_4I`, `HIP_R_4U` exist (type confirmed); the WMMA claim is not header-checkable |
| `hipFloatComplex`/`hipDoubleComplex` | `c64 c128` named | H | rockfish: `HIP_C_32F`, `HIP_C_64F` confirmed. The enum is wider: `HIP_C_16F`, `C_16BF` and complex over 4/8/16/32/64-bit signed and unsigned ints exist and KISS names `c64`/`c128` only: a **GAP candidate, ruling needed** (the `c32` question of 2.2 and 2.4, and more). Nothing checked on hardware (no ROCm box) |

### 2.4 SYCL (`sycl:`; registered namespace)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `sycl::half`, `ext::oneapi::bfloat16`, `float`, `double`, ints, `bool` | named | H | |
| `precision::tf32` (joint matrix) | **by design** | H | as CUDA |
| Intel FP8 (`hf8`, `bf8` conversions / XMX) | likely `f8e4m3fn` / `f8e5m2`, but **which variant** each denotes is unverified | **L** | same layout-to-variant ratification the sk4 RFC §10 needed for Vulkan; bonefish (grep of oneAPI 2026.1 `sycl` headers): **not found** (0 hits for `hf8 bf8 e4m3 e5m2 fp8`), so not exposed there |
| `std::complex<float/double>` | `c64 c128` named | H | bonefish: also `ext::oneapi::experimental::complex<T>`, T in `{double float sycl::half}`; `complex<half>` is a 32-bit complex, so the `c32` **GAP** of 2.2 recurs here |
| 4-bit ints (joint matrix `int4`) | `i4 u4` named | M | bonefish: **not found** in the 2026.1 headers (0 hits in `ext/oneapi/matrix`, positive control `tf32`; `sycl::int4` is a 4-lane vector, not a 4-bit int) |

### 2.5 OpenCL (no namespace file yet)

| Type | KISS | Conf. | Note |
|---|---|---|---|
| `char uchar short ushort int uint long ulong half float double` | named | H | `half`/`double` by extension; openclownfish: confirmed (fp16 on 2 of 3 devices; a `half` argument still builds on all 3) |
| `bfloat16` (vendor/Khronos extensions) | `bf16` named | M | openclownfish (opencl-sys 0.6.5 headers, which may predate newer Khronos headers, and 3 devices): **corrected**: no OpenCL `bfloat16` type in those headers, no bf extension on any device, and `bfloat16` is an unknown type name on all 3 |
| Packed 4x8 integer dot (`cl_khr_integer_dot_product`) | carried in a 32-bit integer; the Vulkan `i8packed` analogue has no Classify token | M | openclownfish: confirmed, but only on the CPU runtime (not on the other 2 devices) |
| `bool` | not a kernel-argument type | H | openclownfish: confirmed, refused as a kernel parameter on all 3 devices |

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
| 2 | `i128`, `u128` | PTX wide-integer ops (was M) | **PTX layer resolved 2026-11-25: PTX 9.5 has no `.s128`/`.u128` type**; only `.b128` bits. The LANGUAGE-level question (CUDA `__int128`, Mojo `int128`) is **RESOLVED in 0.24.0**: `i128 u128 i256 u256` are tokens (§6.1-0017) |
| 3 | `c32` (complex `f16`) | half complex in cuFFT/CUTLASS (M) | name with the `c32` migration note the sk4 RFC §4 reserved |
| 4 | `f8e6m2` (reserved) | no device found that computes on it (L) | keep reserved or **retire**; the ruling covers only types a device uses |
| 5 | Intel hf8/bf8 | variant unknown (L) | ratify layout to variant (as sk4 did for Vulkan FP8) |
| 6 | Block-float formats (Tenstorrent `bfp*`) | shared-exponent block formats (L) | decide encoding+scale (sk4 §3.2) vs new dtype; needs the vendor's definition |
| 7 | `tf32` | exposed as a type by CUDA and SYCL | **confirm by design**: not a dtype (`(acc+mp)` carries it), so a device reporting it is not "unnamed" |
| 8 | `mxint8` (Vulkan `MXINT8_EXT`) | `VK_EXT_shader_ocp_microscaling_types`, found by vulkane at header 368 (H) | name, or decide it is block-scaled `i8` (an encoding plus scale, sk4 §3.2) |

Everything not in this table, among the rows I could populate, is already named. **The remainder is eight
items, of which two (item 1, and item 8 found by vulkane) are certain.**

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

## 5. Board-501 brief (read 2026-11-25; doc-only, nothing minted)

Sources are web pages read through a fetch tool that paraphrases and truncates; a row marked *paraphrased* was not seen verbatim. Nothing here moves a byte.

**5.1 Language-level 128-bit integers** (the union standard is about the LANGUAGES, not only PTX; the PTX layer above stays true).

| Layer | What the source says | Source |
|---|---|---|
| CUDA C++ `__int128` | CUDA 11.5 release notes: "Preview release of a new data type, __int128, usable with compatible host compilers", "no broad support for math operations"; CUDA 11.6: "Full release of 128-bit integer (__int128) data type including compiler and developer tools support", host compiler must support the type; libcu++ math functions take it. Operations list, atomics and the lowering to 64-bit pairs: **not found** in what was read; nvcc driver docs read only to 100,000 of 181,477 characters. | docs.nvidia.com/cuda/archive/11.6.1/cuda-toolkit-release-notes/index.html |
| CUDA tile code | "128-bit integer or floating point types" are listed as unsupported in tile code. | CUDA Programming Guide, 05-appendices/cpp-language-support.html, 5.3.10.8 |
| Mojo | `DType` lists `int128`, `uint128`, `int256`, `uint256` (and platform-width `int`/`uint`). Not checked: scalar structs named `Int128`/`UInt256`, `SIMD[DType.int128, N]` in practice, GPU-backend support. | mojolang.org/docs/stdlib/builtin/dtype/DType |

**Language-level: yes (leaning), for board 501.** Two languages (CUDA C++ `__int128`, Mojo `DType.int128/int256`) name a wide integer an IR must carry to be complete by the union standard; PTX lowers it, which is a legal lowering, not an absence. What KISS would name (`i128`/`u128`, `i256`/`u256`) and the legal lowering are CireSnave's word.

**5.2 Scale and alternate float formats, PTX ISA 9.5 s5.2.3 "Alternate Floating-Point Data Formats"** (packed forms: Table 9, s5.2.5.1; fundamental types: Table 8, s5.2.1). *Paraphrased* unless stated; sign convention for the signed rows is NOT stated in the text read.

| Format | Bits | Exp/Mant | Infinity | NaN | Held in |
|---|---|---|---|---|---|
| `e4m3` | 8 | 4/3 | none | `0x7f`, `0xff` (verbatim) | bit-size type |
| `e5m2` | 8 | 5/2 | not read | not read | bit-size type |
| `e2m3`, `e3m2` | 6 | 2/3, 3/2 | none | none | packed `x2` in `.b16` |
| `e2m1` | 4 | 2/1 | none (verbatim) | none (verbatim) | packed `e2m1x2` in `.b8` |
| `ue8m0` | 8 | 8/0, unsigned | none (verbatim) | `0xff` (verbatim) | packed `ue8m0x2` in `.b16` |
| `ue4m3` | 7 | 4/3, unsigned | none (verbatim) | `0x7f` (verbatim) | `.b8`, MSB zero |
| `ue5m3` | 8 | 5/3, unsigned | none | `0xff` | `.b8`; packed `.ue5m3x2` |

At the 27-token set KISS named `f8e8m0` (= `ue8m0`) only. **0.24.0: `ue4m3` is now the token `f8ue4m3` (§6.1-0020, RESOLVED)**; `ue5m3` is still a **GAP** (single-source, PTX Table 53; a later batch). `ue4m3` has no packed type in Table 9.

**5.3 `.b128` in instructions** (PTX ISA 8.4 page, archive 12.4; section numbers shift in 9.5). `ld` and `st`: `.b128` in the type list, PTX 8.3, `sm_70` or higher, `.sys` scope with `.b128` from PTX 8.4. `mov`: scalar `mov` has no `.b128`; the vector pack/unpack form takes `.b128`, PTX 8.3, `sm_70`. `atom`: **not read in any official page**; secondary sources show `atom...cas.b128` and `exch.b128` and one unverified report of `sm_90`. `red`: no `.b128` seen. The 9.5 page text cuts off before these sections.


## 6. tcgen05 instruction-descriptor type codes (read 2026-11-26 at PTX ISA 9.5 Tables 52/53 and CUTLASS main `cute/arch/mma_sm100_desc.hpp`; doc-only, nothing minted)

Both sources were read verbatim (the PTX page as text, the header as a file), not paraphrased.

| Field | `.kind::mxf8f6f4` (Table 52, bits 7-9 atype / 10-12 btype) | `.kind::mxf4`, `.kind::mxf4nvf4` (Table 53, bits 7-9 atype / 10-11 btype) | CUTLASS (`MXF8F6F4Format`, `MXF4Format`) |
|---|---|---|---|
| E4M3 / E5M2 | 0 / 1 | n/a | 0 / 1 (agrees) |
| E2M3 / E3M2 | 3 / 4 | n/a | 3 / 4 (agrees) |
| E2M1 | 5 | 1 | 5 in `MXF8F6F4Format`, 1 in `MXF4Format` (agrees with both tables) |
| Scale type (bit 23 / bits 23-24) | `UE8M0 = 1` only | `.kind::mxf4`: `UE8M0 = 1`; `.kind::mxf4nvf4`: `UE4M3 = 0`, `UE8M0 = 1`, `UE5M3 = 2` | `ScaleFormat`: `UE4M3 = 0`, `UE8M0 = 1`; **no `UE5M3`** in the header read |

Findings, all facts about those two sources:

- The code for E2M1 is **instruction-kind dependent** (5 under `mxf8f6f4`, 1 under `mxf4`/`mxf4nvf4`); a descriptor code is therefore not a dtype token, and KISS's `f4e2m1` names the storage type, not the code.
- The mx scale `UE8M0` is `1` in every kind that carries it, and is KISS's `f8e8m0`. `UE4M3` and `UE5M3` appear only as `.kind::mxf4nvf4` scale types: the same two GAPs as section 5.2, now with the instruction that consumes them (0.24.0 closes `UE4M3` as `f8ue4m3`; `UE5M3` stays a GAP). The PTX Table 53 value `UE4M3 = 0` is this audit's own s6 read (PTX ISA 9.5, Tables 52/53, 2026-11-25); §6.1-0020 cites CUTLASS only and does not yet cite it. CUTLASS lacks `UE5M3` at `main` while PTX ISA 9.5 has it.
- These codes are bits of a hardware instruction descriptor, i.e. kernel internals; under the KISS scope rule (outward inputs and outputs only) nothing here needs a KISS token, and the board-501 question stays only whether the block-scale storage formats are named.
