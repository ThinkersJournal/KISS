// SPDX-License-Identifier: MIT OR Apache-2.0
// The ONE CUDA-side copy of the splitmix64 generator the on-device differentials draw their corpus
// from. Bit-identical to differential.rs::SplitMix64; tests/splitmix_single_source.rs pins the
// constants against the Rust side and fails if a .cu file re-declares them.
#pragma once
#include <stdint.h>

struct SplitMix64 {
    uint64_t s;
    explicit SplitMix64(uint64_t seed) : s(seed) {}
    uint64_t next() {
        s += 0x9E3779B97F4A7C15ULL;
        uint64_t z = s;
        z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
        z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
        return z ^ (z >> 31);
    }
};
