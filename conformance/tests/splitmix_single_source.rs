// SPDX-License-Identifier: MIT OR Apache-2.0
//! The splitmix64 corpus generator has one CUDA-side copy (`cuda/splitmix64.cuh`) and one Rust copy
//! (`differential.rs`). This pins the header's constants and shifts against the Rust constants and fails
//! if either on-device program re-declares the generator, so the corpora cannot drift apart silently.
//! The sources are read at COMPILE time (`include_str!`), never at run time.

use kiss_conformance::differential::{SPLITMIX_GAMMA, SPLITMIX_MIX1, SPLITMIX_MIX2};

const HEADER: &str = include_str!("../cuda/splitmix64.cuh");
const DEVICE_PROGRAMS: [(&str, &str); 2] = [
    ("fmax_ieee.cu", include_str!("../cuda/fmax_ieee.cu")),
    ("generated_relu_add_diff.cu", include_str!("../cuda/generated_relu_add_diff.cu")),
];

fn cuda_literal(v: u64) -> String {
    format!("0x{v:016X}ULL")
}

// harness-internal: pins the CUDA header's splitmix64 constants and shifts to differential.rs, no clause asserted
#[test]
fn the_cuda_header_carries_the_rust_splitmix_constants() {
    for (name, v) in [("GAMMA", SPLITMIX_GAMMA), ("MIX1", SPLITMIX_MIX1), ("MIX2", SPLITMIX_MIX2)] {
        assert_eq!(HEADER.matches(&cuda_literal(v)).count(), 1, "{name} must appear exactly once in the header");
    }
    for shift in ["z >> 30", "z >> 27", "z >> 31"] {
        assert_eq!(HEADER.matches(shift).count(), 1, "`{shift}` must appear exactly once in the header");
    }
}

// harness-internal: the on-device programs include the one header and never re-declare the generator, no clause asserted
#[test]
fn no_device_program_redeclares_splitmix() {
    for (name, src) in DEVICE_PROGRAMS {
        assert!(src.contains("#include \"splitmix64.cuh\""), "{name} must include splitmix64.cuh");
        assert!(!src.contains("struct SplitMix64"), "{name} re-declares SplitMix64");
        for v in [SPLITMIX_GAMMA, SPLITMIX_MIX1, SPLITMIX_MIX2] {
            assert!(!src.contains(&cuda_literal(v)), "{name} hand-copies {}", cuda_literal(v));
        }
    }
}

// harness-internal: positive control, the detector used above sees the constants where they ARE declared, no clause asserted
#[test]
fn the_redeclaration_detector_has_teeth() {
    assert!(HEADER.contains("struct SplitMix64"));
    let regressed = format!("{}\n{}", DEVICE_PROGRAMS[0].1, HEADER);
    assert!(regressed.contains(&cuda_literal(SPLITMIX_GAMMA)));
}
