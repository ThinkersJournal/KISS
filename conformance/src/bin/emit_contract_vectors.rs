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
    print!("{}", kiss_conformance::contract::emit_contract_vectors_json());
}
