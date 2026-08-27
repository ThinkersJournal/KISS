//! Loader for the KISS-Conform oracle-vector corpus (§6.3-0003, §6.4, §6.5).
//! Parses a Wycheproof-shaped JSON bundle (see docs/superpowers/specs/
//! 2026-07-19-kiss-oracle-vector-corpus-design.md §4) into typed cells, decoding
//! every hex bit-pattern via the existing `parse_hex`. Class is one of the three
//! DeterminismClass members — `split` is NOT a class (§6.8-0005) and is rejected;
//! `provenance` MUST be `oracle` or a promoted/negative tag, never the circular
//! `reference-observed` (§6.5-0003).

use crate::json::{parse, Json};
use crate::{parse_hex, DeterminismClass};

#[derive(Debug, Clone)]
pub struct Corpus {
    pub schema: String,
    pub schema_version: u64,
    pub ulp_metric: String,
    pub vectors: Vec<Cell>,
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub tc_id: u64,
    pub op: String,
    pub dtype: String,
    pub rounding: String,
    pub inputs: Vec<Vec<u8>>,
    pub expected: Vec<u8>,
    pub class: DeterminismClass,
    pub ulp_bound: u64,
    pub provenance: String,
    pub tags: Vec<String>,
    pub has_certificate: bool,
    pub certificate_precision_bits: Option<u64>,
}

fn class_from_str(s: &str) -> Result<DeterminismClass, String> {
    match s {
        "exact-byte" => Ok(DeterminismClass::ExactByte),
        "ULP" => Ok(DeterminismClass::UlpTolerance),
        "order-invariant" => Ok(DeterminismClass::OrderInvariant),
        // §6.8-0005: split is an op-named comparator refinement, NOT a fourth class.
        other => Err(format!("`{other}` is not a determinism class (split is not a class, §6.8-0005)")),
    }
}

fn field<'a>(o: &'a Json, k: &str) -> Result<&'a Json, String> {
    o.get(k).ok_or_else(|| format!("missing field `{k}`"))
}
fn str_field(o: &Json, k: &str) -> Result<String, String> {
    field(o, k)?.as_str().map(|s| s.to_string()).ok_or_else(|| format!("`{k}` is not a string"))
}

/// The oracle-vector schemas and versions this build recognizes (§6.5-0017). RECOGNIZED-SETS,
/// not literal `==` comparisons: a future schema/version is then a DATA change here rather than a
/// comparison added at some call site and forgotten. That is the shape of KISS #271 — a hardcoded
/// required-key tuple whose new member was wired at the call site, so the gate silently stopped
/// covering it; the fix was a set, not a name. The clause's "unrecognized `schema`/`schema_version`"
/// language and this structure line up on purpose.
const RECOGNIZED_SCHEMAS: &[&str] = &["kiss-oracle-vectors-v1.json"];
const RECOGNIZED_VERSIONS: &[u64] = &[1];

pub fn load(json_text: &str) -> Result<Corpus, String> {
    let root = parse(json_text)?;
    // §6.5-0017: an unrecognized `schema` MUST typed-decline, never load-as-if-v1. Before this
    // clause the field was read and never validated — a `kiss-oracle-vectors-v99.json` ran as v1.
    let schema = str_field(&root, "schema")?;
    if !RECOGNIZED_SCHEMAS.contains(&schema.as_str()) {
        return Err(format!(
            "unrecognized `schema` `{schema}` (§6.5-0017); recognized: {RECOGNIZED_SCHEMAS:?}"
        ));
    }
    // §6.5-0017: `schema_version` MUST be READ and GATED — a corpus freezes at a version (§8), and
    // reading the field without gating on it does not satisfy the clause (cf. §6.8-0009). Before
    // this clause the field was never read at all, so a `schema_version: 999` bundle ran as v1.
    let schema_version = field(&root, "schema_version")?
        .as_u64()
        .ok_or("`schema_version` is not an integer")?;
    if !RECOGNIZED_VERSIONS.contains(&schema_version) {
        return Err(format!(
            "unrecognized `schema_version` {schema_version} (§6.5-0017); recognized: {RECOGNIZED_VERSIONS:?}"
        ));
    }
    let ulp_metric = str_field(&root, "ulp_metric")?;
    let raw = field(&root, "vectors")?.as_arr().ok_or("`vectors` is not an array")?;
    let mut vectors = Vec::with_capacity(raw.len());
    for (idx, v) in raw.iter().enumerate() {
        vectors.push(load_cell(v).map_err(|e| format!("vector[{idx}]: {e}"))?);
    }
    Ok(Corpus { schema, schema_version, ulp_metric, vectors })
}

fn load_cell(v: &Json) -> Result<Cell, String> {
    let provenance = str_field(v, "provenance")?;
    if provenance == "reference-observed" {
        return Err("provenance `reference-observed` is circular and inadmissible (§6.5-0003)".into());
    }
    let inputs_json = field(v, "inputs")?.as_arr().ok_or("`inputs` is not an array")?;
    let mut inputs = Vec::with_capacity(inputs_json.len());
    for inp in inputs_json {
        inputs.push(parse_hex(str_field(inp, "bits")?.as_str()));
    }
    let expected = parse_hex(str_field(field(v, "expected")?, "bits")?.as_str());
    let tags = field(v, "tags")?
        .as_arr()
        .ok_or("`tags` is not an array")?
        .iter()
        .filter_map(|t| t.as_str().map(|s| s.to_string()))
        .collect();
    Ok(Cell {
        tc_id: field(v, "tcId")?.as_u64().ok_or("`tcId` is not an integer")?,
        op: str_field(v, "op")?,
        dtype: str_field(v, "dtype")?,
        rounding: str_field(v, "rounding")?,
        inputs,
        expected,
        class: class_from_str(&str_field(v, "class")?)?,
        ulp_bound: field(v, "ulp_bound")?.as_u64().ok_or("`ulp_bound` is not an integer")?,
        provenance,
        tags,
        has_certificate: v.get("certificate").is_some(),
        certificate_precision_bits: v.get("certificate")
            .and_then(|c| c.get("stabilized_precision_bits"))
            .and_then(|n| n.as_u64()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeterminismClass;

    const SAMPLE: &str = r#"{
      "schema": "kiss-oracle-vectors-v1.json",
      "schema_version": 1,
      "ulp_metric": "integer totalOrder distance",
      "vectors": [
        {
          "tcId": 1, "op": "add", "dtype": "f32", "rounding": "roundTiesToEven",
          "inputs": [ {"role":"a","dtype":"f32","bits":"80 00 00 00"},
                      {"role":"b","dtype":"f32","bits":"00 00 00 00"} ],
          "expected": {"dtype":"f32","bits":"00 00 00 00"},
          "class": "exact-byte", "ulp_bound": 0, "provenance": "oracle",
          "tags": ["signed-zero"],
          "certificate": {"hardness_margin_bits": 0, "stabilized_precision_bits": 24}
        }
      ]
    }"#;

    #[test]
    fn loads_a_cell_with_decoded_bits_and_class() {
        let c = load(SAMPLE).unwrap();
        assert_eq!(c.schema, "kiss-oracle-vectors-v1.json");
        assert_eq!(c.vectors.len(), 1);
        let cell = &c.vectors[0];
        assert_eq!(cell.op, "add");
        assert_eq!(cell.inputs.len(), 2);
        assert_eq!(cell.inputs[0], vec![0x80, 0x00, 0x00, 0x00]); // -0.0
        assert_eq!(cell.expected, vec![0x00, 0x00, 0x00, 0x00]); // +0.0
        assert_eq!(cell.class, DeterminismClass::ExactByte);
        assert!(cell.has_certificate);
        assert_eq!(cell.provenance, "oracle");
    }

    #[test]
    fn rejects_split_as_a_class() {
        let bad = SAMPLE.replace("\"exact-byte\"", "\"split\"");
        assert!(load(&bad).is_err(), "split is not a determinism class (§6.8-0005)");
    }

    #[test]
    fn rejects_reference_observed_provenance() {
        let bad = SAMPLE.replace("\"oracle\"", "\"reference-observed\"");
        assert!(load(&bad).is_err(), "reference-observed is circular (§6.5-0003)");
    }
}
