// SPDX-License-Identifier: MIT OR Apache-2.0
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
    kiss_conformance::cli::basics("emit_announce_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::announce::emit_announce_vectors_json());
}
