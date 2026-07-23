# Generated-kernel provenance

`baracuda_gen_relu_add_f32_co_v4.cu` was **emitted verbatim by the reference
generator** — it is not hand-written. It is committed here as the fixed artifact
that the on-device differential (`../generated_relu_add_diff.cu`,
`tests/device.rs::generated_relu_add_matches_kiss_on_device`) certifies against
the KISS-pinned semantics.

| | |
|---|---|
| Generator | `baracuda-kernelgen` (the reference kernel generator) |
| Commit | `e3530a39` (`feat/sk3-codec-bump`; `structure_key.rs` untouched since the codec commit `b5082bc5`, so codec-identical; body byte-identical to the prior `aca0aa85` emission — sk3 regen [#81](https://github.com/ThinkersJournal/KISS/pull/81)) |
| Command | `cargo run --bin kernelgen -- <out-dir>` |
| Op | `relu_add` = `relu(input(0) + input(1))`, f32 |
| Cell (generator's key) | `sk3\|bin\|f32\|cuda:sm89\|ix32\|grid\|r1\|co/00/v4/d16/f;co/00/v4/d16/f;co/00/v4/d16/f\|-` |
| Kernel signature | `extern "C" __global__ void baracuda_gen_relu_add_f32_co_v4(const float4*, const float4*, float4*, long long nv)` |

The generator's key is schema-conformant `sk3|…|cuda:sm89`: the sk2→sk3 bump (the
sk3 GEMM-precision RFC / [PR #66](https://github.com/ThinkersJournal/KISS/pull/66),
`SCHEMA_VERSION 3`) re-prefixes every token, and for this non-`gem` `bin` cell the
change is **prefix-only** (`sk2|` → `sk3|`) — the cell carries no `gem` precision
group. The kernel body is byte-identical across the bump — the token lives only in
the header comment, not the emitted CUDA — so this remains the generator's verbatim
output. Beyond the key, this differential certifies the kernel's **numeric
semantics**, which is what an implementation must get right.

> **sk3 regen record (2026-07-23, [#81](https://github.com/ThinkersJournal/KISS/pull/81)).**
> The KISS reference codec is **sk3** (`SCHEMA_VERSION 3`). The `.cu` above was
> re-emitted by the Baracuda generator from `e3530a39` (codec-identical to
> `b5082bc5`, branch `feat/sk3-codec-bump`): the header token updated `sk2|` →
> `sk3|`, and the emitted CUDA is **byte-identical** to the prior sk2-era emission
> (Baracuda confirmed the `.cu` diff is exactly that one header-comment line; the
> `bin` cell carries no `gem` group). The three-way byte-match was re-recorded at
> sk3 — the same process as the sk2/[#60](https://github.com/ThinkersJournal/KISS/issues/60)
> regen: Baracuda emitted fresh from `e3530a39` (header token = the machine-checked
> `relu_add` golden byte-for-byte, sha256 `ee182f9f…`), and Fuel's independent
> `structure_key` deriver cross-verified the goldens (14/14 expressible, byte-identical).

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
