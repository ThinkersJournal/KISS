# Generated-kernel provenance

`baracuda_gen_relu_add_f32_co_v4.cu` was **emitted verbatim by the reference
generator** — it is not hand-written. It is committed here as the fixed artifact
that the on-device differential (`../generated_relu_add_diff.cu`,
`tests/device.rs::generated_relu_add_matches_kiss_on_device`) certifies against
the KISS-pinned semantics.

| | |
|---|---|
| Generator | `baracuda-kernelgen` (the reference kernel generator) |
| Commit | `1ba6f4abe158615ceba5118f55217faf14b4384a` |
| Command | `cargo run --bin kernelgen -- <out-dir>` |
| Op | `relu_add` = `relu(input(0) + input(1))`, f32 |
| Cell (generator's key) | `sk1\|bin\|f32\|sm89\|i32\|grid\|r1\|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f\|-` |
| Kernel signature | `extern "C" __global__ void baracuda_gen_relu_add_f32_co_v4(const float4*, const float4*, float4*, long long nv)` |

Note the generator's key is schema `sk1|…|sm89` (it predates the KISS-Classify
`structure_key` spec, which is `sk2|…|cuda:sm89`). That divergence is a *key
format* matter, out of scope for this differential — this test certifies the
kernel's **numeric semantics**, which is what an implementation must get right.

## Regenerating

From the `baracuda` repo:

```sh
cargo run --bin kernelgen -- /tmp/gen
cp /tmp/gen/baracuda_gen_relu_add_f32_co_v4.cu \
   <this-repo>/conformance/cuda/generated/
```

If the regenerated kernel's semantics ever drift from the KISS `relu` clause, the
on-device differential turns red — which is the point: this is where "the spec is
testable" becomes "the reference generator is conformant."
