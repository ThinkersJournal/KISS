/* reduce_mean over n elements, forward-order sum ÷ n. Entry per the §6.5 reduction ABI. */
__declspec(dllexport) void kiss_reduce_mean(const float* in, float* out, long long n) {
    float s = 0.0f;
    for (long long i = 0; i < n; ++i) s += in[i];
    out[0] = s / (float)n;
}
