// A correct elementwise-add kernel (implementation A).
__declspec(dllexport) void kiss_add(const float* a, const float* b, float* o, long long n) {
    for (long long i = 0; i < n; ++i) {
        o[i] = a[i] + b[i];
    }
}
