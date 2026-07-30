/* WRONG on purpose: divides by (n-1). The harness must CATCH this. */
__declspec(dllexport) void kiss_reduce_mean(const float* in, float* out, long long n) {
    float s = 0.0f;
    for (long long i = 0; i < n; ++i) s += in[i];
    out[0] = s / (float)(n - 1);
}
