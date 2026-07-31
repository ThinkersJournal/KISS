/* WRONG — computes the per-row MIN instead of the MAX (correct axis, wrong op).
   Caught by the exact-byte comparator on any row with distinct values. §6.5 ABI:
   (in, out, ein, eout, n). */
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) {
        float m = in[r * cols];
        for (long long c = 1; c < cols; ++c) {
            float x = in[r * cols + c];
            if (x < m) m = x;
        }
        out[r] = m;
    }
}
