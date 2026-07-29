/* Correct elementwise add, independent implementation: pointer-walk + commuted
   operand order. `b + a == a + b` bit-for-bit in IEEE-754. */
__declspec(dllexport) void kiss_add(const float* in0, const float* in1, float* out, long long n) {
    const float* p = in0; const float* q = in1; float* r = out;
    for (long long i = 0; i < n; ++i) { *r = *q + *p; ++p; ++q; ++r; }
}
