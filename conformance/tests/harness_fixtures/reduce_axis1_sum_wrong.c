/* WRONG — reduces axis 0 (per column) instead of axis 1 (per row). Writes `cols`
   output cells. The harness catches this either by output-length mismatch
   (non-square shapes) or by value divergence (the asymmetric square). This is the
   failure mode a rank-1-only reduction harness structurally cannot express.
   §6.5 ABI: (in, out, ein, eout, n). */
__declspec(dllexport) void kiss_reduce_axis1(const float* in, float* out, const long long* ein, const long long* eout, long long n) {
    long long rows = ein[0], cols = ein[1]; (void)eout; (void)n;
    for (long long c = 0; c < cols; ++c) {
        float s = 0.0f;
        for (long long r = 0; r < rows; ++r) s += in[r * cols + c];
        out[c] = s;
    }
}
