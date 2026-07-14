// KISS-Conform §6.5/§6.6 — ON-DEVICE differential of a GENERATED kernel.
//
// This closes the loop from "the spec is testable" to "the reference generator is
// conformant". The kernel under test is NOT hand-written — it is emitted verbatim
// by the reference generator (baracuda-kernelgen, commit 1ba6f4ab) for the op
// `relu_add` = relu(a + b), f32 (see generated/PROVENANCE.md). We difference its
// on-device output, over the same corpus the Rust oracle uses, against the
// KISS-pinned semantics — the generator vouches for nothing; the harness does.
//
// `relu` is KISS-Ops §2.3/§6.15: `x < 0 ? 0 : x` — NaN-PROPAGATING and
// -0.0-PRESERVING; explicitly NOT `max(x,0)`. The generator spells it exactly so:
//   ((v.x + w.x) < 0.0f ? 0.0f : (v.x + w.x))
//
// Two checks in ONE run, over all ordered pairs of the corpus:
//   KISS_REF : the generated kernel MUST match relu_add_kiss(a,b) bit-for-bit
//              (both are pure f32; `a+b` is a plain add, no FMA contraction) ->
//              divergences MUST be 0. This is the conformance verdict.
//   NAIVE_REF: the generated kernel MUST DIVERGE from the naive `max(a+b, 0)`
//              (fmaxf, which scrubs NaN and normalizes -0.0) -> divergences MUST
//              be > 0. This proves the emitted kernel carries the CORRECT relu
//              semantics, not the tempting-but-wrong one; a generator that
//              emitted max(x,0) would show 0 naive-divergences and fail here.
//
// Exit 0 iff (KISS divergences == 0 AND NAIVE divergences > 0). A CUDA API/launch
// error prints "ERROR ..." and exits 2, so the harness tells a real divergence
// apart from an environment failure.

#include <cstdio>
#include <cstdint>
#include <cstring>
#include <cstdlib>
#include <cuda_runtime.h>

// The kernel under test — emitted verbatim by baracuda-kernelgen:
#include "generated/baracuda_gen_relu_add_f32_co_v4.cu"

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

// edge_f32() from differential.rs, in the SAME order (16 values).
static int edge_f32(float* out) {
    const float e[] = { 0.0f, -0.0f, 1.0f, -1.0f, 2.0f, -2.0f, 0.5f, -0.5f };
    int k = 0;
    for (float v : e) out[k++] = v;
    out[k++] = bits_to_f32(0x7F800000u); // +inf
    out[k++] = bits_to_f32(0xFF800000u); // -inf
    out[k++] = bits_to_f32(0x7FC00000u); // canonical quiet NaN
    out[k++] = bits_to_f32(0x7FC01234u); // quiet NaN with payload
    out[k++] = bits_to_f32(0x7F7FFFFFu); // MAX
    out[k++] = bits_to_f32(0xFF7FFFFFu); // MIN (most negative)
    out[k++] = bits_to_f32(0x00800000u); // MIN_POSITIVE (smallest normal)
    out[k++] = bits_to_f32(0x00000001u); // smallest positive subnormal
    return k; // 16
}

// agree(): both NaN, or identical bits (differential.rs::agree).
static bool agree(float x, float y) {
    bool xn = (x != x), yn = (y != y);
    if (xn && yn) return true;
    return f32_to_bits(x) == f32_to_bits(y);
}

// KISS relu(a+b), volatile-barriered so -O2 cannot fold `s<0?0:s` to a
// NaN-scrubbing max (mirrors the fmax_ieee_cpu barrier; semantics.rs::relu∘add).
static float relu_add_kiss(float a, float b) {
    float s = a + b;
    volatile int neg = (s < 0.0f);
    return neg ? 0.0f : s;
}

// The wrong "relu" §2.3 warns against: max(x,0) via fmaxf (NaN-scrubbing).
static float relu_add_naive(float a, float b) {
    return fmaxf(a + b, 0.0f);
}

#define CK(call) do { cudaError_t e_ = (call); if (e_ != cudaSuccess) { \
    fprintf(stderr, "ERROR %s: %s\n", #call, cudaGetErrorString(e_)); return 2; } } while (0)

int main() {
    float corpus[16 + 4096];
    int m0 = edge_f32(corpus);
    SplitMix64 rng(0x12345678ULL);
    const int NR = 4096;
    for (int i = 0; i < NR; i++) corpus[m0 + i] = bits_to_f32((uint32_t)rng.next());
    int m = m0 + NR; // 4112

    long long n = (long long)m * (long long)m; // all ordered pairs
    size_t bytes = (size_t)n * sizeof(float);
    float* a = (float*)malloc(bytes);
    float* b = (float*)malloc(bytes);
    float* out = (float*)malloc(bytes);
    if (!a || !b || !out) { fprintf(stderr, "ERROR host alloc\n"); return 2; }
    for (long long k = 0; k < n; k++) { a[k] = corpus[k / m]; b[k] = corpus[k % m]; }

    float *da, *db, *dout;
    CK(cudaMalloc(&da, bytes));
    CK(cudaMalloc(&db, bytes));
    CK(cudaMalloc(&dout, bytes));
    CK(cudaMemcpy(da, a, bytes, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(db, b, bytes, cudaMemcpyHostToDevice));

    // The generated kernel is float4-vectorized; n is a multiple of 4 (m % 4 == 0).
    long long nv = n / 4;
    int tpb = 256;
    long long blocks = (nv + tpb - 1) / tpb;
    if (blocks > 65535) blocks = 65535; // grid-stride handles the rest
    baracuda_gen_relu_add_f32_co_v4<<<(unsigned)blocks, tpb>>>(
        (const float4*)da, (const float4*)db, (float4*)dout, nv);
    CK(cudaGetLastError());
    CK(cudaDeviceSynchronize());
    CK(cudaMemcpy(out, dout, bytes, cudaMemcpyDeviceToHost));

    long long kiss_div = 0, naive_div = 0, shown = 0;
    for (long long k = 0; k < n; k++) {
        float ref = relu_add_kiss(a[k], b[k]);
        if (!agree(out[k], ref)) {
            if (shown++ < 8)
                fprintf(stderr, "KISS DIVERGE a=%08X b=%08X dev=%08X ref=%08X\n",
                        f32_to_bits(a[k]), f32_to_bits(b[k]), f32_to_bits(out[k]), f32_to_bits(ref));
            kiss_div++;
        }
        if (!agree(out[k], relu_add_naive(a[k], b[k]))) naive_div++;
    }

    printf("generated: baracuda-kernelgen relu_add (commit 1ba6f4ab)\n");
    printf("KISS_REF  divergences=%lld of n=%lld\n", kiss_div, n);
    printf("NAIVE_REF divergences=%lld of n=%lld\n", naive_div, n);
    bool pass = (kiss_div == 0) && (naive_div > 0);
    printf("%s generated relu_add matches KISS relu(a+b) and diverges from max(x,0)\n",
           pass ? "PASS" : "FAIL");

    cudaFree(da); cudaFree(db); cudaFree(dout);
    free(a); free(b); free(out);
    return pass ? 0 : 1;
}
