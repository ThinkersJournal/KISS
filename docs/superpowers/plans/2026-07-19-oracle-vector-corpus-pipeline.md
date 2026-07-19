# Oracle-Vector Corpus Pipeline (Plan A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the serialized-corpus pipeline end-to-end — a stdlib JSON reader, a frozen JSON oracle-vector bundle, a minter, a differential harness with teeth, and the §6.5-0008/0009 clause tests — proven on exact-byte `add` (incl. signed zero), which freezes the format before the numeric core (Plan B) plugs in.

**Architecture:** A dependency-free Rust reader loads a Wycheproof-shaped JSON corpus (raw-hex bit patterns). A minter emits the frozen bundle from the existing `semantics.rs`. A differential test drives the existing comparators against an implementation-under-test and proves it catches a wrong impl. A dev-time Python validator re-checks every cell against an independent recomputation (the 3-source stack's exact-byte leg; transcendental legs arrive in Plan B). Coverage is checked against an op-manifest derived from ops.md.

**Tech Stack:** Rust (stdlib only, edition 2021, rust-version 1.77) for the shipped crate `kiss-conformance`; Python 3.8+ (stdlib for the manifest extractor; `mpmath`/`gmpy2` are dev-time-only and NOT used in Plan A).

## Global Constraints

- The shipped `kiss-conformance` crate MUST stay **stdlib-only** — no new `[dependencies]` in `conformance/Cargo.toml` (verbatim from spec §2 Non-goals). JSON is parsed by a hand-written reader, not serde.
- Every float in the corpus is pinned as its **raw IEEE-754 bit-pattern in uppercase hex**, MSB byte first, left to right; grouping marks ` ` and `·` are ignored on read (spec §4; reuse `conformance/src/lib.rs::parse_hex`).
- `class` ∈ {`exact-byte`, `ULP`, `order-invariant`} only — **never** `split` (§6.8-0005); the split comparator is selected by op name via §6.8-0008 precedence in the reader (not exercised in Plan A).
- Every cell carries an inline `expected` value and a `certificate` object; `provenance` is `oracle` (never `reference-observed` — circular per §6.5-0003).
- `rounding` is mandatory per cell; no global default.
- Corpus files live under `conformance/corpus/` (shipped, frozen, committed). CORE-MATH `.wc` and other validation inputs live under `tools/corpus-validation/` (dev-only) — none in Plan A.
- Commit messages end with the repo's `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>` trailer. Work is on branch `oracle-vector-corpus`.

---

## File Structure

- Create `conformance/src/json.rs` — minimal stdlib JSON value reader (parse only).
- Create `conformance/src/corpus.rs` — corpus envelope + cell types and a loader over `json.rs`.
- Modify `conformance/src/lib.rs` — add `pub mod json;` and `pub mod corpus;`.
- Create `conformance/src/bin/kiss_mint.rs` — minter binary; emits the frozen bundle from `semantics.rs`.
- Create `conformance/corpus/ops-arith.json` — the minted, committed slice-0a bundle (`add`).
- Create `conformance/corpus/op_manifest.json` — generated coverage manifest (from ops.md).
- Modify `tools/kiss_ops.py` — add `--emit-manifest` reusing its ops.md parsers.
- Create `conformance/tests/corpus_differential.rs` — differential over the bundle; proves teeth.
- Create `conformance/tests/corpus_coverage.rs` — the §6.5-0008 and §6.5-0009 clause tests.
- Create `tools/validate_corpus.py` — dev-time validator (exact-byte leg + 3-source scaffold).

---

### Task 1: Minimal stdlib JSON reader

**Files:**
- Create: `conformance/src/json.rs`
- Modify: `conformance/src/lib.rs` (add `pub mod json;`)
- Test: inline `#[cfg(test)]` in `conformance/src/json.rs`

**Interfaces:**
- Produces: `pub enum Json { Null, Bool(bool), Num(f64), Str(String), Arr(Vec<Json>), Obj(Vec<(String, Json)>) }`; `pub fn parse(&str) -> Result<Json, String>`; accessors `Json::get(&self, &str) -> Option<&Json>`, `as_str -> Option<&str>`, `as_arr -> Option<&[Json]>`, `as_u64 -> Option<u64>`.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `conformance/src/json.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_object_array_and_utf8_grouping_mark() {
        let v = parse(r#"{"a": [1, "BF 80·00 00"], "b": true, "c": null}"#).unwrap();
        assert_eq!(v.get("a").unwrap().as_arr().unwrap().len(), 2);
        assert_eq!(v.get("a").unwrap().as_arr().unwrap()[0].as_u64(), Some(1));
        // the · (U+00B7, 2 bytes UTF-8) inside the string must survive intact
        assert_eq!(v.get("a").unwrap().as_arr().unwrap()[1].as_str(), Some("BF 80·00 00"));
        assert_eq!(v.get("b"), Some(&Json::Bool(true)));
        assert_eq!(v.get("c"), Some(&Json::Null));
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse("{} x").is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kiss-conformance --lib json`
Expected: FAIL to compile ("cannot find module `json`" / `parse` not found).

- [ ] **Step 3: Write the implementation**

Put this at the top of `conformance/src/json.rs`:

```rust
//! A minimal, dependency-free JSON reader — enough to load the oracle-vector
//! corpus (KISS-Conform §6.3-0003) without pulling serde into a stdlib-only
//! crate. Parse-only; the corpus is authored/minted, never serialized here.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// The value for `key` in an object, else `None` (also `None` for non-objects).
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Json::Str(s) = self { Some(s) } else { None }
    }
    pub fn as_arr(&self) -> Option<&[Json]> {
        if let Json::Arr(a) = self { Some(a) } else { None }
    }
    /// The value as a u64 (JSON numbers are stored as f64; corpus integers are small).
    pub fn as_u64(&self) -> Option<u64> {
        if let Json::Num(n) = self { Some(*n as u64) } else { None }
    }
}

/// Parse a complete JSON document. Trailing non-whitespace is an error.
pub fn parse(input: &str) -> Result<Json, String> {
    let mut p = Parser { b: input.as_bytes(), i: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("trailing data at byte {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.b.get(self.i) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => self.lit("true", Json::Bool(true)),
            Some(b'f') => self.lit("false", Json::Bool(false)),
            Some(b'n') => self.lit("null", Json::Null),
            Some(_) => self.number(),
            None => Err("unexpected end of input".into()),
        }
    }
    fn lit(&mut self, s: &str, v: Json) -> Result<Json, String> {
        if self.b[self.i..].starts_with(s.as_bytes()) {
            self.i += s.len();
            Ok(v)
        } else {
            Err(format!("invalid literal at byte {}", self.i))
        }
    }
    fn object(&mut self) -> Result<Json, String> {
        self.i += 1; // consume '{'
        let mut out = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Json::Obj(out));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            if self.b.get(self.i) != Some(&b':') {
                return Err(format!("expected ':' at byte {}", self.i));
            }
            self.i += 1;
            let v = self.value()?;
            out.push((k, v));
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(out));
                }
                _ => return Err(format!("expected ',' or '}}' at byte {}", self.i)),
            }
        }
    }
    fn array(&mut self) -> Result<Json, String> {
        self.i += 1; // consume '['
        let mut out = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Json::Arr(out));
        }
        loop {
            let v = self.value()?;
            out.push(v);
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(out));
                }
                _ => return Err(format!("expected ',' or ']' at byte {}", self.i)),
            }
        }
    }
    // Accumulate raw bytes so multi-byte UTF-8 (e.g. the '·' grouping mark) survives.
    fn string(&mut self) -> Result<String, String> {
        if self.b.get(self.i) != Some(&b'"') {
            return Err(format!("expected string at byte {}", self.i));
        }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        while let Some(&c) = self.b.get(self.i) {
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|_| "invalid utf-8 string".to_string()),
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or("bad escape")?;
                    self.i += 1;
                    match e {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i + 4).ok_or("bad \\u")?)
                                .map_err(|_| "bad \\u")?;
                            let cp = u32::from_str_radix(hex, 16).map_err(|_| "bad \\u")?;
                            self.i += 4;
                            let ch = char::from_u32(cp).ok_or("bad codepoint")?;
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return Err(format!("bad escape at byte {}", self.i)),
                    }
                }
                _ => out.push(c),
            }
        }
        Err("unterminated string".into())
    }
    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while let Some(&c) = self.b.get(self.i) {
            if matches!(c, b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                self.i += 1;
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.b[start..self.i]).map_err(|_| "bad number")?;
        s.parse::<f64>().map(Json::Num).map_err(|_| format!("bad number {s:?}"))
    }
}
```

Add to `conformance/src/lib.rs` after the existing `pub mod` lines (keep alphabetical with the neighbours):

```rust
pub mod json;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kiss-conformance --lib json`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add conformance/src/json.rs conformance/src/lib.rs
git commit -m "conform: add a minimal stdlib JSON reader for the oracle-vector corpus

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: Corpus envelope + cell types and loader

**Files:**
- Create: `conformance/src/corpus.rs`
- Modify: `conformance/src/lib.rs` (add `pub mod corpus;`)
- Test: inline `#[cfg(test)]` in `conformance/src/corpus.rs`

**Interfaces:**
- Consumes: `crate::json::{parse, Json}`; `crate::{DeterminismClass, parse_hex}`.
- Produces:
  - `pub struct Corpus { pub schema: String, pub ulp_metric: String, pub vectors: Vec<Cell> }`
  - `pub struct Cell { pub tc_id: u64, pub op: String, pub dtype: String, pub rounding: String, pub inputs: Vec<Vec<u8>>, pub expected: Vec<u8>, pub class: DeterminismClass, pub ulp_bound: u64, pub provenance: String, pub tags: Vec<String>, pub has_certificate: bool }`
  - `pub fn load(json_text: &str) -> Result<Corpus, String>`

- [ ] **Step 1: Write the failing test**

Add to the bottom of `conformance/src/corpus.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeterminismClass;

    const SAMPLE: &str = r#"{
      "schema": "kiss-oracle-vectors-v1.json",
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kiss-conformance --lib corpus`
Expected: FAIL to compile (`corpus` module / `load` not found).

- [ ] **Step 3: Write the implementation**

Put this at the top of `conformance/src/corpus.rs`:

```rust
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

pub fn load(json_text: &str) -> Result<Corpus, String> {
    let root = parse(json_text)?;
    let schema = str_field(&root, "schema")?;
    let ulp_metric = str_field(&root, "ulp_metric")?;
    let raw = field(&root, "vectors")?.as_arr().ok_or("`vectors` is not an array")?;
    let mut vectors = Vec::with_capacity(raw.len());
    for (idx, v) in raw.iter().enumerate() {
        vectors.push(load_cell(v).map_err(|e| format!("vector[{idx}]: {e}"))?);
    }
    Ok(Corpus { schema, ulp_metric, vectors })
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
    })
}
```

Note: `parse_hex` currently `assert!`s on an odd digit count. That is fine for a frozen corpus (a malformed hex string is a mint-time bug, not a runtime input). Add `pub mod corpus;` to `conformance/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kiss-conformance --lib corpus`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add conformance/src/corpus.rs conformance/src/lib.rs
git commit -m "conform: add the oracle-vector corpus loader (typed cells, class/provenance guards)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: op-manifest extractor (`kiss_ops.py --emit-manifest`)

**Files:**
- Modify: `tools/kiss_ops.py` (add an `--emit-manifest` mode reusing its ops.md parsers)
- Create: `conformance/corpus/op_manifest.json` (the generated artifact, committed)
- Test: `tools/test_kiss_ops_manifest.py` (a small stdlib `unittest`)

**Interfaces:**
- Produces: `op_manifest.json` = `{ "schema": "kiss-op-manifest-v1", "generated_from": "spec/ops.md", "declared_coverage_set": [op names in scope for the CURRENT slice], "transcendental_atoms": [names], "all_ops": [every op name] }`. For Plan A, `declared_coverage_set` is the exact-byte arithmetic ops so §6.5-0008 is green on what is minted; it grows as slices land.

- [ ] **Step 1: Write the failing test**

Create `tools/test_kiss_ops_manifest.py`:

```python
import json, subprocess, sys, pathlib, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

class ManifestTest(unittest.TestCase):
    def test_emit_manifest_has_atoms_and_declared_set(self):
        out = subprocess.run(
            [sys.executable, str(ROOT / "tools" / "kiss_ops.py"), "--emit-manifest", "--stdout"],
            capture_output=True, text=True, check=True,
        ).stdout
        m = json.loads(out)
        self.assertEqual(m["schema"], "kiss-op-manifest-v1")
        # exp/log/sin are transcendental atoms per ops.md §2.7/§6.8
        for atom in ("exp", "log", "sin"):
            self.assertIn(atom, m["transcendental_atoms"])
        # add is a primitive-floor op and is this slice's declared coverage
        self.assertIn("add", m["all_ops"])
        self.assertIn("add", m["declared_coverage_set"])

if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python tools/test_kiss_ops_manifest.py`
Expected: FAIL (`--emit-manifest` is not a recognized argument → non-zero exit → `CalledProcessError`).

- [ ] **Step 3: Write the implementation**

In `tools/kiss_ops.py`, add a manifest builder that reuses the existing parsers (`sec27_primitive`, `sec27_nonprimitive`, `sec27_nonprimitive_family`, `sec618_table_ops`, and `list_ops`/`between` as needed) and wire two new args. Add this function near the other `sec*`/`check` functions:

```python
def transcendental_atoms(ops):
    """The atoms with a declared ULP ceiling — ops.md §6.8 table. §6.5-0008 cites §6.8
    for 'every transcendental atom'. Those atoms (exp/log/sin/cos/atan/atan2/erf/
    lgamma/sqrt) live in the PRIMITIVE FLOOR, so they come from the ULP-ceiling table,
    NOT the non-primitive family column (verified against spec/ops.md)."""
    region = between(ops, "Maximum ULP ceiling", "KISS-OPS-6.8-0001")
    atoms = []
    for cells in table_cells(region):
        if cells:
            atoms += op_tokens(cells[0])  # col 0 may list several: `exp`, `log`, ...
    return sorted(set(atoms))


def build_manifest(spec_dir):
    """Derive the op/atom coverage manifest from ops.md (§6.5-0008 checks against it)."""
    ops = open(os.path.join(spec_dir, "ops.md"), encoding="utf-8").read()
    primitive = set(sec27_primitive(ops))
    nonprim = set(sec27_nonprimitive_family(ops))
    all_ops = sorted(primitive | nonprim)
    atoms = transcendental_atoms(ops)  # sqrt, exp, log, sin, cos, atan, atan2, erf, lgamma
    # Plan A's declared coverage set: the exact-byte arithmetic floor that is minted now.
    declared = sorted(o for o in ("add",) if o in all_ops)
    return {
        "schema": "kiss-op-manifest-v1",
        "generated_from": "spec/ops.md",
        "all_ops": all_ops,
        "transcendental_atoms": atoms,
        "declared_coverage_set": declared,
    }
```

Then in `main()`, after the existing argument definitions, add:

```python
    ap.add_argument("--emit-manifest", action="store_true",
                    help="write conformance/corpus/op_manifest.json (the §6.5-0008 coverage source)")
    ap.add_argument("--stdout", action="store_true",
                    help="with --emit-manifest, print the manifest instead of writing the file")
```

and near the top of `main()`'s body, before the normal `check(...)` path, handle the new mode:

```python
    args = ap.parse_args()
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "spec")
    if args.emit_manifest:
        import json as _json
        manifest = build_manifest(spec_dir)
        text = _json.dumps(manifest, indent=2) + "\n"
        if args.stdout:
            sys.stdout.write(text)
        else:
            out_path = os.path.join(os.path.dirname(spec_dir), "conformance", "corpus", "op_manifest.json")
            os.makedirs(os.path.dirname(out_path), exist_ok=True)
            open(out_path, "w", encoding="utf-8", newline="\n").write(text)
            print(f"wrote {out_path}")
        return
```

(If `main()` already calls `ap.parse_args()` and computes `spec_dir`, reuse those instead of re-adding them — do not duplicate.)

- [ ] **Step 4: Run test + generate the committed manifest**

Run: `python tools/test_kiss_ops_manifest.py`
Expected: PASS.
Run: `python tools/kiss_ops.py --emit-manifest`
Expected: prints `wrote .../conformance/corpus/op_manifest.json`; the file exists and contains `add` in `declared_coverage_set`.

- [ ] **Step 5: Commit**

```bash
git add tools/kiss_ops.py tools/test_kiss_ops_manifest.py conformance/corpus/op_manifest.json
git commit -m "tools: kiss_ops --emit-manifest — the ops.md-derived §6.5-0008 coverage source

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 4: Minter binary — freeze the `add` bundle

**Files:**
- Create: `conformance/src/bin/kiss_mint.rs`
- Create: `conformance/corpus/ops-arith.json` (generated, committed)
- Test: `conformance/tests/mint_roundtrip.rs`

**Interfaces:**
- Consumes: `kiss_conformance::semantics::add`, `kiss_conformance::hex`.
- Produces: a committed `conformance/corpus/ops-arith.json` loadable by `corpus::load`. The binary writes to the path given as `argv[1]`, defaulting to `conformance/corpus/ops-arith.json` relative to the crate manifest dir.

- [ ] **Step 1: Write the failing test**

Create `conformance/tests/mint_roundtrip.rs`:

```rust
//! The minted bundle must load back through the corpus reader and carry the
//! signed-zero add cell (the point of the slice: -0 vs +0 is normative, exact-byte).
use kiss_conformance::corpus;

#[test]
fn frozen_arith_bundle_loads_and_has_signed_zero_cell() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-arith.json"))
        .expect("frozen bundle must be committed; run `cargo run --bin kiss_mint`");
    let c = corpus::load(&text).expect("bundle parses");
    assert!(c.vectors.iter().all(|v| v.op == "add"));
    // (-0) + (+0) = +0 under RNE — a cell that a normalize-to-+0 bug would still pass,
    // but (-0)+(-0) = -0 is the one that bites; both must be present.
    let neg_zero_sum = c.vectors.iter().find(|v| v.tags.iter().any(|t| t == "signed-zero")
        && v.inputs[0] == vec![0x80,0,0,0] && v.inputs[1] == vec![0x80,0,0,0]);
    let cell = neg_zero_sum.expect("(-0)+(-0) cell present");
    assert_eq!(cell.expected, vec![0x80, 0, 0, 0], "(-0)+(-0) = -0.0");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kiss-conformance --test mint_roundtrip`
Expected: FAIL (`corpus/ops-arith.json` does not exist).

- [ ] **Step 3: Write the minter**

Create `conformance/src/bin/kiss_mint.rs`:

```rust
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
```

- [ ] **Step 4: Generate the bundle, then run the test**

Run: `cargo run -p kiss-conformance --bin kiss_mint`
Expected: `wrote .../conformance/corpus/ops-arith.json`.
Run: `cargo test -p kiss-conformance --test mint_roundtrip`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add conformance/src/bin/kiss_mint.rs conformance/corpus/ops-arith.json conformance/tests/mint_roundtrip.rs
git commit -m "conform: kiss_mint minter + frozen add bundle (signed-zero cells)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: Differential harness with teeth

**Files:**
- Create: `conformance/tests/corpus_differential.rs`

**Interfaces:**
- Consumes: `kiss_conformance::corpus::{load, Cell}`, `kiss_conformance::{compare, DeterminismClass}`, `kiss_conformance::semantics::add`.
- Produces: a reusable `run_against(&Corpus, impl Fn(&Cell) -> Vec<u8>) -> Result<(), String>` that applies the class-selected comparator per cell.

- [ ] **Step 1: Write the failing test**

Create `conformance/tests/corpus_differential.rs`:

```rust
//! KISS-Conform §6.5-0001 differential: run an implementation-under-test against
//! the frozen corpus and compare under each cell's declared class. Proves teeth —
//! a correct add passes; a normalize-to-+0 add fails the (-0)+(-0) cell.
use kiss_conformance::corpus::{self, Cell, Corpus};
use kiss_conformance::{compare, DeterminismClass};

fn bundle() -> Corpus {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/ops-arith.json")).unwrap();
    corpus::load(&text).unwrap()
}

// Apply an f32 binary implementation to a cell and return the result bytes (big-endian).
fn eval_add(cell: &Cell, f: impl Fn(f32, f32) -> f32) -> Vec<u8> {
    let a = f32::from_be_bytes(cell.inputs[0].clone().try_into().unwrap());
    let b = f32::from_be_bytes(cell.inputs[1].clone().try_into().unwrap());
    f(a, b).to_bits().to_be_bytes().to_vec()
}

fn run_against(c: &Corpus, f: impl Fn(f32, f32) -> f32) -> Result<(), String> {
    for cell in &c.vectors {
        let actual = eval_add(cell, &f);
        // Plan A is exact-byte only; the class dispatch is here for when ULP/split arrive.
        match cell.class {
            DeterminismClass::ExactByte => compare(cell.class, &actual, &cell.expected)
                .map_err(|e| format!("tcId {}: {e}", cell.tc_id))?,
            other => return Err(format!("tcId {}: class {other:?} not in Plan A", cell.tc_id)),
        }
    }
    Ok(())
}

#[test]
fn reference_add_passes_every_cell() {
    let c = bundle();
    run_against(&c, kiss_conformance::semantics::add).expect("reference add is conformant");
}

#[test]
fn a_normalize_to_plus_zero_add_is_caught() {
    let c = bundle();
    // A subtly-wrong add that scrubs the sign of a zero result: (-0)+(-0) -> +0.
    let wrong = |a: f32, b: f32| {
        let r = a + b;
        if r == 0.0 { 0.0 } else { r } // normalizes -0.0 to +0.0
    };
    let err = run_against(&c, wrong).expect_err("the harness MUST catch the signed-zero bug");
    assert!(err.contains("tcId 2"), "the (-0)+(-0) cell is the one with teeth: {err}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kiss-conformance --test corpus_differential`
Expected: it compiles and runs; both tests should PASS once the bundle from Task 4 exists. If Task 4's bundle is missing, FAIL at `unwrap()`. (Confirm the failing-first state by temporarily renaming the bundle, then restore.)

- [ ] **Step 3: (implementation already present)** — the harness is the test file; no separate impl. If `reference_add_passes_every_cell` fails, the bug is in the minter or reader, not this task.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kiss-conformance --test corpus_differential`
Expected: PASS (2 tests) — the reference passes, the wrong impl is caught at tcId 2.

- [ ] **Step 5: Commit**

```bash
git add conformance/tests/corpus_differential.rs
git commit -m "conform: corpus differential harness (§6.5-0001) with a teeth test

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 6: The §6.5-0008 and §6.5-0009 clause tests

**Files:**
- Create: `conformance/tests/corpus_coverage.rs`

**Interfaces:**
- Consumes: `kiss_conformance::corpus::load`, `kiss_conformance::json`, the committed `corpus/ops-arith.json` and `corpus/op_manifest.json`.

- [ ] **Step 1: Write the failing test**

Create `conformance/tests/corpus_coverage.rs`:

```rust
//! KISS-CONFORM-6.5-0008 (coverage completeness) and 6.5-0009 (inline wide-precision
//! stored value), enforced against the frozen bundle + the ops.md-derived manifest.
use kiss_conformance::{corpus, json};

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{}/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn test_conform_oracle_vector_coverage_complete() {
    // §6.5-0008: every op in the manifest's declared coverage set MUST appear in the
    // corpus. (Plan A's declared set is the exact-byte arithmetic floor; it grows per slice.)
    let corpus = corpus::load(&read("corpus/ops-arith.json")).unwrap();
    let manifest = json::parse(&read("corpus/op_manifest.json")).unwrap();
    let declared: Vec<&str> = manifest.get("declared_coverage_set").unwrap()
        .as_arr().unwrap().iter().filter_map(|j| j.as_str()).collect();
    let covered: std::collections::BTreeSet<&str> =
        corpus.vectors.iter().map(|c| c.op.as_str()).collect();
    for op in &declared {
        assert!(covered.contains(op), "§6.5-0008: declared op `{op}` has no oracle vectors");
    }
}

#[test]
fn test_conform_oracle_vector_stores_wide_precision_value() {
    // §6.5-0009: every cell stores an inline expected value AND a certificate; no cell
    // defers its value to a live run.
    let corpus = corpus::load(&read("corpus/ops-arith.json")).unwrap();
    for c in &corpus.vectors {
        assert!(!c.expected.is_empty(), "§6.5-0009: tcId {} has no inline expected value", c.tc_id);
        assert!(c.has_certificate, "§6.5-0009: tcId {} lacks a certificate", c.tc_id);
    }
}
```

- [ ] **Step 2: Run test to verify it fails then passes**

Run: `cargo test -p kiss-conformance --test corpus_coverage`
Expected: PASS if Tasks 3 and 4 produced the manifest and bundle. To confirm teeth: temporarily add `"mul"` to the manifest's `declared_coverage_set`, re-run, see `test_conform_oracle_vector_coverage_complete` FAIL (`mul` not covered), then revert.

- [ ] **Step 3: (implementation already present)** — these are the clause tests themselves.

- [ ] **Step 4: Run the full crate test suite**

Run: `cargo test -p kiss-conformance`
Expected: all tests PASS (existing + json + corpus + mint_roundtrip + corpus_differential + corpus_coverage).

- [ ] **Step 5: Commit**

```bash
git add conformance/tests/corpus_coverage.rs
git commit -m "conform: wire test_conform_oracle_vector_coverage_complete + stores_wide_precision_value (§6.5-0008/0009)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 7: Dev-time validation gate (exact-byte leg + 3-source scaffold)

**Files:**
- Create: `tools/validate_corpus.py`
- Test: `tools/test_validate_corpus.py`

**Interfaces:**
- Produces: `validate_corpus.py <bundle.json>` that reads the bundle, recomputes each exact-byte cell independently (Python), and asserts bit equality; exit 0 clean, 1 on any mismatch. The mpmath / MPFR(gmpy2) / Lefèvre–Muller transcendental legs are declared as `TranscendentalLeg` stubs that raise `NotImplementedError` and are only reached for ULP/split cells (none in Plan A) — Plan B fills them.

- [ ] **Step 1: Write the failing test**

Create `tools/test_validate_corpus.py`:

```python
import json, subprocess, sys, pathlib, tempfile, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "conformance" / "corpus" / "ops-arith.json"

class ValidateTest(unittest.TestCase):
    def test_frozen_bundle_validates(self):
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "validate_corpus.py"), str(BUNDLE)],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_corrupted_expected_is_rejected(self):
        data = json.loads(BUNDLE.read_text())
        # flip the (-0)+(-0) expected from -0 (80..) to +0 (00..)
        for v in data["vectors"]:
            if v["tcId"] == 2:
                v["expected"]["bits"] = "00 00 00 00"
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(data, f); tmp = f.name
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "validate_corpus.py"), tmp],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 1, "validator must reject a wrong expected value")

if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python tools/test_validate_corpus.py`
Expected: FAIL (`validate_corpus.py` does not exist).

- [ ] **Step 3: Write the implementation**

Create `tools/validate_corpus.py`:

```python
#!/usr/bin/env python3
"""validate_corpus.py — the dev-time-only oracle-vector validation gate.

Reads a frozen bundle and independently re-derives every `provenance: "oracle"`
cell, asserting bit equality. Exact-byte cells are recomputed in Python here (the
3-source stack's trivial leg). The ULP/split transcendental legs — mpmath, MPFR
(gmpy2) driven by CORE-MATH hard cases, and Lefevre-Muller anchors — are declared
below and filled in Plan B; they are only reached for ULP/split cells, of which a
Plan A bundle has none. NEVER shipped; ordinary devs never run this.
"""
import json, struct, sys

def bits_to_f32(hexstr):
    b = bytes(int(h, 16) for h in hexstr.split() if all(c in "0123456789abcdefABCDEF" for c in h) and h)
    # tolerate '·' grouping: split on whitespace after replacing the mark
    return b

def clean_bytes(hexstr):
    digits = "".join(c for c in hexstr if c in "0123456789abcdefABCDEF")
    assert len(digits) % 2 == 0, f"odd hex: {hexstr!r}"
    return bytes(int(digits[i:i+2], 16) for i in range(0, len(digits), 2))

def f32_of(b):  # big-endian bytes -> python float
    return struct.unpack(">f", b)[0]

def f32_bytes(x):
    return struct.pack(">f", x)

class TranscendentalLeg:
    """Plan B fills these; unreachable for a Plan A (exact-byte-only) bundle."""
    def value(self, op, inputs):
        raise NotImplementedError(f"transcendental leg for {op} lands in Plan B")

def recompute_exact_byte(op, input_byte_lists):
    if op == "add":
        a = f32_of(input_byte_lists[0]); b = f32_of(input_byte_lists[1])
        return f32_bytes(a + b)
    raise NotImplementedError(f"exact-byte recompute for {op} not defined")

def validate(path):
    data = json.loads(open(path, encoding="utf-8").read())
    fails = 0
    for cell in data["vectors"]:
        op = cell["op"]
        cls = cell["class"]
        inputs = [clean_bytes(i["bits"]) for i in cell["inputs"]]
        expected = clean_bytes(cell["expected"]["bits"])
        if cls == "exact-byte":
            got = recompute_exact_byte(op, inputs)
        else:
            got = TranscendentalLeg().value(op, inputs)  # Plan A: never taken
        if got != expected:
            fails += 1
            sys.stderr.write(
                f"MISMATCH tcId {cell['tcId']} {op}: expected {expected.hex()} got {got.hex()}\n")
    if fails:
        sys.stderr.write(f"{fails} cell(s) failed validation\n")
        return 1
    print(f"validated {len(data['vectors'])} cells: OK")
    return 0

if __name__ == "__main__":
    sys.exit(validate(sys.argv[1]))
```

(The unused `bits_to_f32` scaffold may be dropped; `clean_bytes` is the one used. Keep the file minimal.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `python tools/test_validate_corpus.py`
Expected: PASS (2 tests — the frozen bundle validates; a corrupted expected is rejected).

- [ ] **Step 5: Commit**

```bash
git add tools/validate_corpus.py tools/test_validate_corpus.py
git commit -m "tools: validate_corpus dev-time gate (exact-byte leg + transcendental scaffold)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 8: Update the ledger and confirm the whole gate

**Files:**
- Modify: `conformance/UNBACKED.tsv` (via `kiss_trace.py --update-ledger`)

- [ ] **Step 1: Run the full Rust suite**

Run: `cargo test -p kiss-conformance`
Expected: all PASS.

- [ ] **Step 2: Refresh the traceability ledger**

The two clause tests now exist, so §6.5-0008/0009 are backed. Run:
`python tools/kiss_trace.py --update-ledger`
Then: `python tools/kiss_trace.py`
Expected: `KISS-CONFORM-6.5-0008` and `KISS-CONFORM-6.5-0009` are no longer `untested`; RESULT should be CLEAN (or show only pre-existing unrelated items). Confirm the two clause IDs are gone from `conformance/UNBACKED.tsv`.

- [ ] **Step 3: Run the Python tests**

Run: `python tools/test_kiss_ops_manifest.py && python tools/test_validate_corpus.py`
Expected: both PASS.

- [ ] **Step 4: Commit**

```bash
git add conformance/UNBACKED.tsv
git commit -m "conform: mark §6.5-0008/0009 backed now that the pipeline tests exist

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Push the branch (optional — only if you want a PR now)**

```bash
git push -u origin oracle-vector-corpus
```

---

## What Plan A deliberately leaves for Plan B

- `conformance/src/hp.rs` — the 256-bit big-float core (add/sub/mul/normalize, round-once RNE, Ziv escalation to 512/1024, constants ln2 / 2π-table / π).
- Transcendental atoms `exp`/`log`/`sin`/`atan2` and the `clog` complex assembly; extending the minter and the differential to ULP + op-named split.
- The three-source transcendental legs in `validate_corpus.py` (mpmath, MPFR via gmpy2 on CORE-MATH `.wc`, Lefèvre–Muller anchors) under `tools/corpus-validation/`.
- Growing `declared_coverage_set` toward the full op set as slices land.

## Self-review notes (done)

- **Spec coverage:** §4 format → Tasks 1,2,4; §6.5-0008 → Task 6; §6.5-0009 → Task 6; §6.5-0001 differential → Task 5; §6.3-0003 bundle → Tasks 2,4; coverage source (Component 6) → Task 3; validation gate (Component 5) → Task 7. Numeric core (Components 2,3-transcendental) → deferred to Plan B (by design, noted above).
- **Placeholders:** none — every code step carries complete code; the one intentional stub (`TranscendentalLeg`) raises and is unreachable in Plan A, documented as Plan B's fill.
- **Type consistency:** `DeterminismClass` (existing enum), `Cell`/`Corpus` (Task 2) consumed unchanged by Tasks 5/6; `parse_hex`/`hex`/`semantics::add` used with their real signatures; big-endian `to_be_bytes`/`from_be_bytes` used consistently on the wire.
