// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/atomic_add_band_vectors.json` (KISS-OPS-6.11-0024). Prints to stdout:
//!
//! ```sh
//! cargo run --bin emit_atomic_add_band_vectors > conformance/corpus/atomic_add_band_vectors.json
//! ```
//!
//! `test_ops_scatter_atomic_add_band` asserts the committed file equals this output byte for byte.

fn main() {
    kiss_conformance::cli::basics("emit_atomic_add_band_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::tolerance::emit_atomic_add_band_vectors_json());
}
