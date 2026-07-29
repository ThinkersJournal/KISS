// A deliberately wrong kernel that computes a - b instead of a + b.
__declspec(dllexport) void kiss_add(const float* a, const float* b, float* o, long long n) {
    for (long long i = 0; i < n; ++i) {
        o[i] = a[i] - b[i];  // BUG: should be a[i] + b[i]
    }
}
