//! Regenerate `conformance/corpus/grammar_vectors.json` from the reference region codec.
//! Prints the artifact to stdout; the committed file is exactly this output:
//!
//! ```sh
//! cargo run --bin emit_grammar_vectors > conformance/corpus/grammar_vectors.json
//! ```
//!
//! The `grammar_vectors` test asserts the committed file equals this output byte-for-byte, so a
//! stale artifact fails CI (the #161 freshness pattern, as `emit_contract_vectors` does for
//! `contract_vectors.json`).
//!
//! ⚠️ WHY A BINARY AND NOT A TEST HELPER. A generator that lives inside a test cannot be RUN to
//! refresh the artifact, so the committed file becomes the only copy of a value nothing derives —
//! which is the state this artifact was created to leave (#466): the KISS-Grammar goldens existed
//! only as prose in an informative appendix and as a Rust `const`, with nothing tying the two
//! together.

fn main() {
    print!("{}", kiss_conformance::grammar::emit_grammar_vectors_json());
}
