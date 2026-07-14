// KISS-Conform §6.6 — ON-DEVICE real-kernel differential slice.
//
// Non-normative. This is a Conform §6.5 differential oracle carried onto real
// hardware: a GPU kernel implementing a pinned KISS-Ops semantics is differenced,
// on-device, against a from-scratch CPU reference decomposition transcribed from
// the same spec clause. The normative source is the spec text; where this and a
// clause disagree, the clause governs.
//
// Op under test: `fmax_ieee` — the NaN-**suppressing** maximum (IEEE-754 maxNum),
// KISS-Ops §6.15:
//     select(cmp_ne(a,a), b,
//       select(cmp_ne(b,b), a,
//         select(cmp_ge(a,b), a, b)))
//
// The three load-bearing distinctions this pins, and which this program exercises
// on real silicon over the SAME corpus the Rust oracle uses (edge cases + the
// splitmix64 stream, seed 0x1234_5678, 4096 draws — see corpus_f32 in
// differential.rs):
//
//   1. NaN-SUPPRESSION: fmax_ieee(NaN, b) = b and fmax_ieee(a, NaN) = a. If BOTH
//      are NaN the first branch returns b (a NaN). This is the opposite of
//      max_prop (torch.maximum), so a kernel that lowered the wrong one diverges
//      immediately on the two NaN corpus entries.
//   2. SIGNED ZERO: cmp_ge(+0,-0) is TRUE (they compare equal), so with a=+0,b=-0
//      the a>=b branch yields a = +0.0; with a=-0,b=+0 it yields a = -0.0. The
//      result's sign-of-zero is order-dependent and pinned bit-exactly. A kernel
//      using CUDA's fmaxf() intrinsic (which returns +0 for the {+0,-0} pair
//      regardless of order) would diverge here — so we DO NOT use fmaxf; we
//      transcribe the §6.15 select decomposition by hand.
//   3. cmp_ge / cmp_ne are the IEEE ordered/unordered predicates: any NaN operand
//      makes >= false and != true. We rely on the hardware's IEEE compares (the
//      device is IEEE-754 for f32 compares), matching the CPU oracle's `>=`/`!=`.
//
// Comparison uses the same relation as differential.rs::agree: two results agree
// iff both are NaN, or their raw bits are identical. That still catches a
// suppress-vs-propagate bug (a NaN where a number is required, or vice versa) and
// the ±0 sign, without being fragile to NaN payload choice.
//
// Build (host compiler = MSVC cl.exe must be on PATH; sm_89 = RTX 4070 Ada):
//   nvcc -O2 -arch=sm_89 fmax_ieee.cu -o fmax_ieee
// Run: prints "PASS n=<pairs>" and exits 0, or "FAIL" + up to 16 divergences and
// exits 1. Any CUDA API/launch error prints "ERROR ..." and exits 2 (so the
// harness distinguishes a real divergence from an environment failure).

#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cuda_runtime.h>

// ---- device: the §6.15 fmax_ieee decomposition, transcribed by hand ----------
// Branch structure mirrors semantics.rs::fmax_ieee one-for-one:
//   if a != a { b } else if b != b { a } else if a >= b { a } else { b }
// `a != a` is exactly cmp_ne(a,a) (true iff NaN); `a >= b` is cmp_ge. These are
// the hardware IEEE compares. We must NOT use fmaxf(): it does not preserve the
// order-dependent sign of zero the clause pins.
__device__ __forceinline__ float fmax_ieee_dev(float a, float b) {
#ifdef USE_FMAXF_INTRINSIC
    // NEGATIVE CONTROL — the wrong-but-tempting lowering. CUDA's fmaxf() is
    // NaN-suppressing (so it passes the NaN corpus) but returns +0 for the
    // {+0,-0} tie regardless of operand order, so it does NOT preserve the
    // order-dependent sign of zero §6.15 pins. Building with -DUSE_FMAXF_INTRINSIC
    // MUST make this program FAIL on the ±0 pairs — that is how we prove the
    // on-device differential slice has teeth (Conform §6.5: catch the wrong impl).
    return fmaxf(a, b);
#else
    if (a != a)      return b;   // a is NaN -> suppress, take b
    else if (b != b) return a;   // b is NaN -> suppress, take a
    else if (a >= b) return a;   // cmp_ge true (incl. +0>=-0) -> a
    else             return b;
#endif
}

__global__ void fmax_ieee_kernel(const float* a, const float* b, float* out, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) out[i] = fmax_ieee_dev(a[i], b[i]);
}

// ---- host: reproduce corpus_f32 + the CPU oracle, generate all ordered pairs --

// splitmix64, bit-identical to differential.rs::SplitMix64.
struct SplitMix64 {
    uint64_t s;
    explicit SplitMix64(uint64_t seed) : s(seed) {}
    uint64_t next() {
        s += 0x9E3779B97F4A7C15ULL;
        uint64_t z = s;
        z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
        z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
        return z ^ (z >> 31);
    }
};

static float bits_to_f32(uint32_t b) { float f; memcpy(&f, &b, 4); return f; }
static uint32_t f32_to_bits(float f) { uint32_t b; memcpy(&b, &f, 4); return b; }

// edge_f32() from differential.rs, in the SAME order.
static void edge_f32(uint32_t* out, int* n) {
    const float e[] = {
        0.0f, -0.0f, 1.0f, -1.0f, 2.0f, -2.0f, 0.5f, -0.5f,
    };
    int k = 0;
    for (float v : e) out[k++] = f32_to_bits(v);
    // infinities
    out[k++] = 0x7F800000u; // +inf
    out[k++] = 0xFF800000u; // -inf
    // NaNs: a bare quiet NaN and a payload-carrying quiet NaN
    out[k++] = 0x7FC00000u; // f32::NAN (canonical quiet)
    out[k++] = 0x7FC01234u; // quiet NaN with payload
    // extremes and subnormals
    out[k++] = 0x7F7FFFFFu; // f32::MAX
    out[k++] = 0xFF7FFFFFu; // f32::MIN (most negative)
    out[k++] = 0x00800000u; // f32::MIN_POSITIVE (smallest normal)
    out[k++] = 0x00000001u; // smallest positive subnormal
    *n = k;
}

// CPU reference oracle — semantics.rs::fmax_ieee, transcribed. Shares no code
// with the device path (Conform §6.5: the oracle is independent of the impl).
//
// The predicates are held in `volatile` temporaries. This is NOT cosmetic: at
// -O2 a host compiler will pattern-match the bare `if (a!=a)…else if (a>=b)…`
// shape into a single hardware max instruction (x86 maxss, ARM fmax) — and that
// intrinsic returns the SECOND operand on a signed-zero tie, i.e. it does NOT
// preserve the order-dependent sign of zero the §6.15 select decomposition pins.
// (Verified on MSVC 19.29 / CUDA 13.3: without the barrier, -O2 folds this oracle
// to maxss and it diverges from the spec on {+0,-0}; -O0 is faithful.) Forcing
// each predicate through a volatile load defeats the fold, so the oracle stays a
// faithful transcription independent of -O level and host compiler — exactly the
// fmax-suppression hazard this on-device slice exists to catch.
static float fmax_ieee_cpu(float a, float b) {
    volatile int a_is_nan = (a != a);
    if (a_is_nan) return b;
    volatile int b_is_nan = (b != b);
    if (b_is_nan) return a;
    volatile int a_ge_b = (a >= b);
    return a_ge_b ? a : b;
}

// agree(): both NaN, or identical bits (differential.rs::agree).
static bool agree(float x, float y) {
    bool xn = (x != x), yn = (y != y);
    if (xn && yn) return true;
    return f32_to_bits(x) == f32_to_bits(y);
}

#define CUDA_CHECK(call) do {                                            \
    cudaError_t err_ = (call);                                           \
    if (err_ != cudaSuccess) {                                           \
        printf("ERROR %s: %s\n", #call, cudaGetErrorString(err_));       \
        return 2;                                                        \
    }                                                                    \
} while (0)

int main() {
    // Build the corpus exactly as corpus_f32(seed, n): edges, then n random draws.
    const uint64_t seed = 0x12345678ULL;
    const int nrand = 4096;
    uint32_t edges[32]; int ne = 0; edge_f32(edges, &ne);
    int m = ne + nrand;

    uint32_t* corpus = (uint32_t*)malloc(sizeof(uint32_t) * m);
    for (int i = 0; i < ne; i++) corpus[i] = edges[i];
    SplitMix64 rng(seed);
    for (int i = 0; i < nrand; i++) corpus[ne + i] = (uint32_t)rng.next();

    // All ordered pairs (m*m), matching diff_binary's double loop.
    long long npairs = (long long)m * (long long)m;
    float* ha = (float*)malloc(sizeof(float) * npairs);
    float* hb = (float*)malloc(sizeof(float) * npairs);
    float* hout = (float*)malloc(sizeof(float) * npairs);
    for (int i = 0; i < m; i++)
        for (int j = 0; j < m; j++) {
            long long idx = (long long)i * m + j;
            ha[idx] = bits_to_f32(corpus[i]);
            hb[idx] = bits_to_f32(corpus[j]);
        }

    float *da, *db, *dout;
    CUDA_CHECK(cudaMalloc(&da, sizeof(float) * npairs));
    CUDA_CHECK(cudaMalloc(&db, sizeof(float) * npairs));
    CUDA_CHECK(cudaMalloc(&dout, sizeof(float) * npairs));
    CUDA_CHECK(cudaMemcpy(da, ha, sizeof(float) * npairs, cudaMemcpyHostToDevice));
    CUDA_CHECK(cudaMemcpy(db, hb, sizeof(float) * npairs, cudaMemcpyHostToDevice));

    int threads = 256;
    int blocks = (int)((npairs + threads - 1) / threads);
    fmax_ieee_kernel<<<blocks, threads>>>(da, db, dout, (int)npairs);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
    CUDA_CHECK(cudaMemcpy(hout, dout, sizeof(float) * npairs, cudaMemcpyDeviceToHost));

    // Difference on-device result vs CPU oracle over every pair.
    long long ndiv = 0;
    for (long long idx = 0; idx < npairs; idx++) {
        float o = fmax_ieee_cpu(ha[idx], hb[idx]);
        float c = hout[idx];
        if (!agree(o, c)) {
            if (ndiv < 16) {
                printf("  DIVERGE a=%08X b=%08X oracle=%08X device=%08X\n",
                       f32_to_bits(ha[idx]), f32_to_bits(hb[idx]),
                       f32_to_bits(o), f32_to_bits(c));
            }
            ndiv++;
        }
    }

    cudaFree(da); cudaFree(db); cudaFree(dout);
    free(ha); free(hb); free(hout); free(corpus);

    if (ndiv == 0) {
        printf("PASS n=%lld (corpus m=%d: %d edge + %d random)\n", npairs, m, ne, nrand);
        return 0;
    } else {
        printf("FAIL divergences=%lld of n=%lld\n", ndiv, npairs);
        return 1;
    }
}
