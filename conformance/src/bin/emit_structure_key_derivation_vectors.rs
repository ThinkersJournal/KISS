// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/structure_key_derivation_vectors.json` from the reference deriver.
//! Prints the artifact to stdout; the committed file is exactly this output:
//!
//! ```sh
//! cargo run --bin emit_structure_key_derivation_vectors > conformance/corpus/structure_key_derivation_vectors.json
//! ```
//!
//! The `structure_key_derivation_vectors` test asserts the committed file equals this output
//! byte-for-byte, so a stale artifact fails CI.

fn main() {
    kiss_conformance::cli::basics("emit_structure_key_derivation_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::derivation_vectors::emit_json());
}
