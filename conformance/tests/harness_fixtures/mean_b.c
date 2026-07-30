/* reduce_mean via pairwise (tree) summation, then ÷ n. A DIFFERENT reassociation
   than mean_a — bit-different partial sums, within the reassociation band. */
static float pairwise(const float* a, long long n) {
    if (n == 1) return a[0];
    long long h = n / 2;
    return pairwise(a, h) + pairwise(a + h, n - h);
}
__declspec(dllexport) void kiss_reduce_mean(const float* in, float* out, long long n) {
    out[0] = pairwise(in, n) / (float)n;
}
