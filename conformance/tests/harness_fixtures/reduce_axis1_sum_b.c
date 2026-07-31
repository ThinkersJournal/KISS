/* Correct axis-1 (per-row) sum reduction, PAIRWISE fold — a DIFFERENT (equally
   valid) reassociation than sum_a's sequential order. On the large-magnitude
   corpus row it lands a nonzero rounding distance from the sequential oracle,
   which the reassociation band must absorb. §6.5 ABI: (in, out, ein, eout, n). */
static float pw(const float* a, long long n) {
    if (n == 0) return 0.0f; /* empty axis -> Sum identity (§6.11-0002); also stops
                                the h=0 split from recursing forever on cols==0 */
    if (n == 1) return a[0];
    long long h = n / 2;
    return pw(a, h) + pw(a + h, n - h);
}
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) out[r] = pw(in + r * cols, cols);
}
