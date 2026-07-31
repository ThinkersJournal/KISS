/* Correct axis-1 (per-row) sum reduction, SEQUENTIAL fold — the order-matching
   sibling of the oracle. §6.5 axis-reduce ABI: (in, out, ein, eout, n). */
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) {
        float s = 0.0f;
        for (long long c = 0; c < cols; ++c) s += in[r * cols + c];
        out[r] = s;
    }
}
