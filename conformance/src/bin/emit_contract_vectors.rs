// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/contract_vectors.json` from the reference codec.
//! Prints the artifact to stdout; the committed file is exactly this output:
//!
//! ```sh
//! cargo run --bin emit_contract_vectors > conformance/corpus/contract_vectors.json
//! ```
//!
//! The `contract_vectors` test asserts the committed file equals this output
//! byte-for-byte, so a stale artifact fails CI (the #161 freshness pattern, as
//! `emit_structure_key_vectors` does for `structure_key_vectors.json`).

fn main() {
    kiss_conformance::cli::basics("emit_contract_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::contract::emit_contract_vectors_json());
}
