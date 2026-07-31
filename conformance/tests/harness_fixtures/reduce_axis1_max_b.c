/* Correct axis-1 (per-row) max reduction, REVERSE scan — same result as max_a,
   different traversal order (proves the exact-byte comparator accepts a legitimate
   reordering for max). §6.5 ABI: (in, out, ein, eout, n). */
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) {
        float m = in[r * cols + cols - 1];
        for (long long c = cols - 2; c >= 0; --c) {
            float x = in[r * cols + c];
            if (x > m) m = x;
        }
        out[r] = m;
    }
}
