//! Regenerate `conformance/corpus/structure_key_vectors.json` from the reference
//! codec. Prints the artifact to stdout; the committed file is exactly this output:
//!
//! ```sh
//! cargo run --bin emit_structure_key_vectors > conformance/corpus/structure_key_vectors.json
//! ```
//!
//! The `structure_key_vectors` test asserts the committed file equals this output
//! byte-for-byte, so a stale artifact fails CI (mirrors the dtype-manifest freshness
//! gate, but as a cargo test — the broader, both-legs CI surface).

fn main() {
    print!("{}", kiss_conformance::reference_vectors::emit_reference_vectors_json());
}
