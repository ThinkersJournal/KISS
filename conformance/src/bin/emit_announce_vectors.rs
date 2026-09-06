//! Regenerate `conformance/corpus/announce_vectors.json` from the reference codec.
//! Prints the artifact to stdout; the committed file is exactly this output:
//!
//! ```sh
//! cargo run --bin emit_announce_vectors > conformance/corpus/announce_vectors.json
//! ```
//!
//! The `announce_framing` test asserts the committed file equals this output byte-for-byte,
//! so a stale artifact fails CI (the #161 freshness pattern, as `emit_contract_vectors` does
//! for `contract_vectors.json`).

fn main() {
    print!("{}", kiss_conformance::announce::emit_announce_vectors_json());
}
