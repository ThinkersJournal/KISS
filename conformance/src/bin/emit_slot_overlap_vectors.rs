// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/slot_overlap_vectors.json` (KISS-CONTRACT-6.5-0019). Prints to stdout:
//!
//! ```sh
//! cargo run --bin emit_slot_overlap_vectors > conformance/corpus/slot_overlap_vectors.json
//! ```
//!
//! `test_contract_slot_overlap_vectors` asserts the committed file equals this output byte for byte.

fn main() {
    kiss_conformance::cli::basics("emit_slot_overlap_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::bind::emit_slot_overlap_vectors_json());
}
