/* WRONG magnitude, RIGHT shape: a sequential per-row sum that DOUBLE-COUNTS each
   row's first element — the classic `float s = in[row];` init followed by a loop
   that still runs from c=0. The output is the same rows-shaped buffer as the
   oracle, so neither the length check nor the axis check can see it: only the
   reassociation band can, BY MAGNITUDE. The injected error equals the row's first
   element, which is >= 1 on every corpus row and exceeds that row's per-row band
   on all of them (including the 1e8 band-exerciser, where the error is 1e8 vs a
   band of ~179), so the band MUST REJECT it. This is the band's rejection edge —
   the agreeing sum_a/sum_b pair only exercises acceptance, and the wrong-axis
   kernel is caught by shape, not magnitude. §6.5 ABI: (in, out, ein, eout, n). */
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) {
        if (cols == 0) { out[r] = 0.0f; continue; } /* no first element to double */
        float s = in[r * cols];                     /* BUG: seed with in[0] ... */
        for (long long c = 0; c < cols; ++c) s += in[r * cols + c]; /* ... re-add at c=0 */
        out[r] = s;
    }
}
