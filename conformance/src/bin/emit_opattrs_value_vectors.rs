// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/opattrs_value_vectors.json` (R2.4 slice 1).
//!
//! A LIBRARY GENERATOR, never a test helper (#365): the builder lives in
//! `kiss_conformance::opattrs_value` so the artifact and the tests read ONE implementation.
//!
//! ```text
//! cargo run --bin emit_opattrs_value_vectors > conformance/corpus/opattrs_value_vectors.json
//! ```
//!
//! The committed artifact is byte-identical to this output; `tests/opattrs_value_vectors.rs` checks it.
fn main() {
    kiss_conformance::cli::basics("emit_opattrs_value_vectors", "Prints the committed vector artifact to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::opattrs_value::emit_opattrs_value_vectors_json());
}
