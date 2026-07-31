/* Correct axis-1 (per-row) max reduction, FORWARD scan. Max is exact-byte for
   any fold order (the only admitted reordering is the sign of a zero, canonicalized
   away by the comparator). §6.5 ABI: (in, out, ein, eout, n). */
#include <math.h>
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) {
        if (cols == 0) { out[r] = -INFINITY; continue; } /* empty axis -> max identity (§6.11-0002) */
        float m = in[r * cols];
        for (long long c = 1; c < cols; ++c) {
            float x = in[r * cols + c];
            if (x > m) m = x;
        }
        out[r] = m;
    }
}
