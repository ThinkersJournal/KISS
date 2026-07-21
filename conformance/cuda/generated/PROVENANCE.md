# Generated-kernel provenance

`baracuda_gen_relu_add_f32_co_v4.cu` was **emitted verbatim by the reference
generator** — it is not hand-written. It is committed here as the fixed artifact
that the on-device differential (`../generated_relu_add_diff.cu`,
`tests/device.rs::generated_relu_add_matches_kiss_on_device`) certifies against
the KISS-pinned semantics.

| | |
|---|---|
| Generator | `baracuda-kernelgen` (the reference kernel generator) |
| Commit | `aca0aa85` (`feat/kiss-convergence` — D8 codec alignment; body byte-identical to the prior `1ba6f4ab` emission, [#60](https://github.com/ThinkersJournal/KISS/issues/60)) |
| Command | `cargo run --bin kernelgen -- <out-dir>` |
| Op | `relu_add` = `relu(input(0) + input(1))`, f32 |
| Cell (generator's key) | `sk2\|bin\|f32\|cuda:sm89\|ix32\|grid\|r1\|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f\|-` |
| Kernel signature | `extern "C" __global__ void baracuda_gen_relu_add_f32_co_v4(const float4*, const float4*, float4*, long long nv)` |

The generator's key is now schema-conformant `sk2|…|cuda:sm89`: the Baracuda
generator's D8 codec alignment (schema `1→2`, `sm*`→`cuda:sm*`, `i32/i64`→`ix32/ix64`
per KISS-CLASSIFY §6.7-0003 / §6.8-0002) closed the earlier `sk1|…|sm89` divergence
([#60](https://github.com/ThinkersJournal/KISS/issues/60)). The kernel body is
byte-identical across the codec change — the token lives only in the header comment,
not the emitted CUDA — so this remains the generator's verbatim output. Beyond the key,
this differential certifies the kernel's **numeric semantics**, which is what an
implementation must get right.

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
