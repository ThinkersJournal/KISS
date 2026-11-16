// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/op_identity_vectors.json` (KISS-CONTRACT-6.11-0009, 6.3-0003, 6.3-0004). Prints to stdout:
//!
//! ```sh
//! cargo run --bin emit_op_identity_vectors > conformance/corpus/op_identity_vectors.json
//! ```
//!
//! `test_contract_text_op_identity` asserts the committed file equals this output byte for byte.

fn main() {
    kiss_conformance::cli::basics("emit_op_identity_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::op_identity_vectors::emit_op_identity_vectors_json());
}
