// A correct elementwise-add kernel (implementation B, independent algorithm).
__declspec(dllexport) void kiss_add(const float* a, const float* b, float* o, long long n) {
    long long i = 0;
    // Unrolled loop for variety, but functionally identical.
    for (; i + 1 < n; i += 2) {
        o[i] = a[i] + b[i];
        o[i + 1] = a[i + 1] + b[i + 1];
    }
    // Remainder.
    for (; i < n; ++i) {
        o[i] = a[i] + b[i];
    }
}
