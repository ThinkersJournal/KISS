// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/packed_reduction_vectors.json` (KISS-OPS-6.11-0026..-0029).
//!
//! ```text
//! cargo run --bin emit_packed_reduction_vectors > conformance/corpus/packed_reduction_vectors.json
//! ```

fn main() {
    print!("{}", kiss_conformance::packed_reduce::emit_json());
}
