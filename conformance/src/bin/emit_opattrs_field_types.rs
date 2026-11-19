// SPDX-License-Identifier: MIT OR Apache-2.0
//! Regenerate `conformance/corpus/opattrs_field_types.json` (per-field kind, vocabulary and range of every OpAttrs field). Prints to stdout:
//!
//! ```sh
//! cargo run --bin emit_opattrs_field_types > conformance/corpus/opattrs_field_types.json
//! ```
//!
//! `test_opattrs_field_types_*` asserts the committed file equals this output byte for byte.

fn main() {
    kiss_conformance::cli::basics("emit_opattrs_field_types", "Prints the committed field-type table to stdout (no files are written).", &[]);
    print!("{}", kiss_conformance::opattrs_field_types::emit_opattrs_field_types_json());
}
