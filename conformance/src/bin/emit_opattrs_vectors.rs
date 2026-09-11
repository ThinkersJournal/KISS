//! Regenerate `conformance/corpus/opattrs_vectors.json` from `spec/ops.md` §6.19.3.
//!
//! ⚠️ A LIBRARY GENERATOR, never a test helper (#365): the builder lives in
//! `kiss_conformance::opattrs_schema` so the artifact and the tests read ONE implementation.
//! A generator that lived in a test would let the committed artifact and the asserted value
//! drift apart with nothing to notice.
//!
//! ```text
//! cargo run --bin emit_opattrs_vectors > conformance/corpus/opattrs_vectors.json
//! ```
//!
//! The committed artifact is byte-identical to this output — the #161 freshness pattern, checked
//! by `tests/opattrs_schema_spec.rs`.
fn main() {
    print!("{}", kiss_conformance::opattrs_schema::emit_opattrs_vectors_json());
}
