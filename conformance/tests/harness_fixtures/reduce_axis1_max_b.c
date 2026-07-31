/* Correct axis-1 (per-row) max reduction, REVERSE scan — same result as max_a,
   different traversal order (proves the exact-byte comparator accepts a legitimate
   reordering for max). §6.5 ABI: (in, out, ein, eout, n). */
#include <math.h>
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long r = 0; r < rows; ++r) {
        if (cols == 0) { out[r] = -INFINITY; continue; } /* empty axis -> max identity (§6.11-0002) */
        float m = in[r * cols + cols - 1];
        for (long long c = cols - 2; c >= 0; --c) {
            float x = in[r * cols + c];
            if (x > m) m = x;
        }
        out[r] = m;
    }
}
