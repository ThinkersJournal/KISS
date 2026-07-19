//! kiss_mint — mints the frozen oracle-vector corpus from the reference oracle.
//! Plan A slice: exact-byte `add` cells (incl. the signed-zero distinctions),
//! provenance `oracle`, class `exact-byte`. Emits the Wycheproof-shaped JSON of
//! docs/superpowers/specs/2026-07-19-kiss-oracle-vector-corpus-design.md §4.

use kiss_conformance::{hex, semantics};

/// One exact-byte `add` cell as a single JSON line. `tags` is the raw contents of
/// the tags array (e.g. `"\"signed-zero\""`, or `""` for none). Continuation `\`
/// at each line end joins the source lines with the single space before it.
fn cell(tc: u32, a: f32, b: f32, tags: &str) -> String {
    let r = semantics::add(a, b);
    let ab = hex(&a.to_bits().to_be_bytes());
    let bb = hex(&b.to_bits().to_be_bytes());
    let rb = hex(&r.to_bits().to_be_bytes());
    format!(
        "    {{\"tcId\": {tc}, \"op\": \"add\", \"dtype\": \"f32\", \"rounding\": \"roundTiesToEven\", \
         \"inputs\": [{{\"role\":\"a\",\"dtype\":\"f32\",\"bits\":\"{ab}\"}}, \
         {{\"role\":\"b\",\"dtype\":\"f32\",\"bits\":\"{bb}\"}}], \
         \"expected\": {{\"dtype\":\"f32\",\"bits\":\"{rb}\"}}, \
         \"class\": \"exact-byte\", \"ulp_bound\": 0, \"provenance\": \"oracle\", \
         \"tags\": [{tags}], \
         \"certificate\": {{\"hardness_margin_bits\": 0, \"stabilized_precision_bits\": 24}}}}"
    )
}

fn main() {
    let nz = f32::from_bits(0x8000_0000); // -0.0
    let pz = 0.0f32;
    let cells = [
        cell(1, nz, pz, "\"signed-zero\""),    // (-0)+(+0) = +0
        cell(2, nz, nz, "\"signed-zero\""),    // (-0)+(-0) = -0
        cell(3, pz, pz, "\"signed-zero\""),    // (+0)+(+0) = +0
        cell(4, 1.0, 1.0, ""),                 // 1+1 = 2
        cell(5, 1.0, -1.0, "\"signed-zero\""), // 1+(-1) = +0
    ];
    let mut doc = String::new();
    doc.push_str("{\n");
    doc.push_str("  \"schema\": \"kiss-oracle-vectors-v1.json\",\n");
    doc.push_str("  \"kiss_substandard\": \"OPS\",\n");
    doc.push_str("  \"schema_version\": 1,\n");
    doc.push_str("  \"spec_clause\": \"KISS-CONFORM-6.4-0002\",\n");
    doc.push_str("  \"generator\": \"kiss_mint 0.1.0\",\n");
    doc.push_str(&format!("  \"number_of_vectors\": {},\n", cells.len()));
    doc.push_str("  \"byte_order\": \"hex is the value's bytes most-significant first, left to right\",\n");
    doc.push_str("  \"hex_encoding\": \"uppercase hex bytes; ' ' and '\u{00b7}' are grouping marks (lib.rs::parse_hex)\",\n");
    doc.push_str("  \"ulp_metric\": \"integer totalOrder distance (lib.rs::ulp_distance_f32)\",\n");
    doc.push_str("  \"vectors\": [\n");
    doc.push_str(&cells.join(",\n"));
    doc.push_str("\n  ]\n}\n");

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-arith.json");
    let path = std::env::args().nth(1).unwrap_or_else(|| default.to_string());
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, doc).unwrap();
    eprintln!("wrote {path}");
}
