//! kiss_mint — mints the frozen oracle-vector corpus from the reference oracle. Two bundles:
//!   corpus/ops-arith.json              — Plan A slice: exact-byte `add` cells (incl. the
//!                                        signed-zero distinctions), provenance `oracle`.
//!   corpus/ops-transcendental-nan.json — T9/#352: transcendental COMPUTED-NaN cells — a
//!                                        computed NaN must be QUIET (§6.16-0010 / §6.8-0010).
//! Both are the Wycheproof-shaped JSON of
//! docs/superpowers/specs/2026-07-19-kiss-oracle-vector-corpus-design.md §4.

use kiss_conformance::{hex, semantics};

const GENERATOR: &str = "kiss_mint 0.1.0";

/// One exact-byte `add` cell as a single JSON line. `tags` is the raw contents of
/// the tags array (e.g. `"\"signed-zero\""`, or `""` for none). Continuation `\`
/// at each line end joins the source lines with the single space before it.
fn add_cell(tc: u32, a: f32, b: f32, tags: &str) -> String {
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

fn arith_doc() -> String {
    let nz = f32::from_bits(0x8000_0000); // -0.0
    let pz = 0.0f32;
    let cells = [
        add_cell(1, nz, pz, "\"signed-zero\""),    // (-0)+(+0) = +0
        add_cell(2, nz, nz, "\"signed-zero\""),    // (-0)+(-0) = -0
        add_cell(3, pz, pz, "\"signed-zero\""),    // (+0)+(+0) = +0
        add_cell(4, 1.0, 1.0, ""),                 // 1+1 = 2
        add_cell(5, 1.0, -1.0, "\"signed-zero\""), // 1+(-1) = +0
    ];
    bundle_doc("OPS", "KISS-CONFORM-6.5-0008", &cells)
}

// ---- transcendental COMPUTED-NaN bundle (T9 step 3) ----------------------------------------
//
// POPULATION — stated so the corpus's gaps are visible, not inferred (a corpus that emits only
// the cases it was written for makes a corpus whose gaps all pass). Each cell is a COMPUTED NaN
// (an arithmetic/transcendental op MINTS or QUIETS a NaN — never a raw-bit move).
//
// EMITTED:
//   Family B — §6.16-0010: an op whose §6.13 decomposition contains arithmetic (every
//     transcendental) MUST deliver a QUIET NaN when an operand is signaling, and MUST NOT return
//     the operand's bits unchanged. exp/log/sin of an sNaN input → the canonical quiet NaN. THE
//     discriminating family: a promote-and-round impl that leaves the sNaN signaling fails the
//     quietness comparison (§6.8-0010 COMPUTED arm). Requires a dtype that admits an sNaN.
//   Family A — §6.2-0001 → IEEE-754 invalid operation: a NaN MINTED from a non-NaN input.
//     log(x<0), log(-inf), sin(±inf) → the canonical quiet NaN.
//
// NOT EMITTED, and why:
//   - qNaN-operand propagation (exp(qNaN), …): quiet-in→quiet-out, so the quietness comparison is
//     non-discriminating (quiet == quiet trivially) — it exercises nothing the families above do.
//   - any MOVED transcendental NaN: none exists — a transcendental contains arithmetic, so
//     §6.16-0010 forbids the raw-bit move; MOVED cells belong to sign/permutation ops (#352).
//   - narrow dtypes (bf16/f16/f8*): the atoms are f64/f32 only, so an oracle value would be
//     unavailable; f8e4m3fn additionally has a single NaN encoding (§6.16-0004) → vacuous
//     quietness, so Family B is meaningless there. Deferred to the narrow-atom rollout.
//   - an exp domain-mint: exp mints no NaN from a non-NaN input (exp(±inf)=+inf/+0) — a genuine
//     ABSENCE, not an omission, so no `exp` cell appears in Family A.

/// One transcendental COMPUTED-NaN cell (unary). `prec` = the compute-dtype width in the
/// certificate (24 for f32, 53 for f64); the value is compared by QUIETNESS (§6.16-0010), so
/// `ulp_bound` is inert for the row and set to a nominal transcendental tolerance.
#[allow(clippy::too_many_arguments)]
fn transc_nan_cell(tc: u32, op: &str, dtype: &str, xb: &str, rb: &str, prec: u32, tags: &str) -> String {
    format!(
        "    {{\"tcId\": {tc}, \"op\": \"{op}\", \"dtype\": \"{dtype}\", \"rounding\": \"roundTiesToEven\", \
         \"inputs\": [{{\"role\":\"x\",\"dtype\":\"{dtype}\",\"bits\":\"{xb}\"}}], \
         \"expected\": {{\"dtype\":\"{dtype}\",\"bits\":\"{rb}\"}}, \
         \"class\": \"ULP\", \"ulp_bound\": 2, \"provenance\": \"oracle\", \
         \"tags\": [{tags}], \"nan_provenance\": \"computed\", \
         \"certificate\": {{\"hardness_margin_bits\": 0, \"stabilized_precision_bits\": {prec}}}}}"
    )
}

/// An f32 cell: mint the oracle from `semantics`, assert it is a NaN (the population invariant),
/// and store the ORIGINAL input bits (so a signaling operand is stored as signaling, never a
/// value that a register move might have quieted before we read it back).
fn nan_cell_f32(tc: u32, op: &str, xbits: u32, tags: &str) -> String {
    let x = f32::from_bits(xbits);
    let r = match op {
        "exp" => semantics::exp(x),
        "log" => semantics::log(x),
        "sin" => semantics::sin(x),
        _ => panic!("cell {tc}: unknown op `{op}`"),
    };
    assert!(r.is_nan(), "cell {tc}: {op}(0x{xbits:08X}) must be a NaN to be a COMPUTED-NaN cell");
    transc_nan_cell(tc, op, "f32", &hex(&xbits.to_be_bytes()), &hex(&r.to_bits().to_be_bytes()), 24, tags)
}

fn nan_cell_f64(tc: u32, op: &str, xbits: u64, tags: &str) -> String {
    let x = f64::from_bits(xbits);
    let r = match op {
        "exp" => semantics::exp_f64(x),
        "log" => semantics::log_f64(x),
        "sin" => semantics::sin_f64(x),
        _ => panic!("cell {tc}: unknown op `{op}`"),
    };
    assert!(r.is_nan(), "cell {tc}: {op}(0x{xbits:016X}) must be a NaN to be a COMPUTED-NaN cell");
    transc_nan_cell(tc, op, "f64", &hex(&xbits.to_be_bytes()), &hex(&r.to_bits().to_be_bytes()), 53, tags)
}

fn transcendental_nan_doc() -> String {
    // f32/f64 signaling-NaN inputs: exp all-ones, quiet bit CLEAR, payload nonzero.
    const SNAN32: u32 = 0x7F80_0001;
    const SNAN64: u64 = 0x7FF0_0000_0000_0001;
    const B: &str = "\"computed-nan\",\"snan-operand\",\"6.16-0010\"";
    const A_LOG: &str = "\"computed-nan\",\"domain-error\",\"6.2-0001\"";
    const A_SIN: &str = "\"computed-nan\",\"no-limit-at-infinity\",\"6.2-0001\"";
    let cells = [
        // Family B — arithmetic quiets a signaling NaN operand (§6.16-0010).
        nan_cell_f32(1, "exp", SNAN32, B),
        nan_cell_f32(2, "log", SNAN32, B),
        nan_cell_f32(3, "sin", SNAN32, B),
        nan_cell_f64(4, "exp", SNAN64, B),
        nan_cell_f64(5, "sin", SNAN64, B),
        // Family A — a NaN minted from a non-NaN input (§6.2-0001 / IEEE invalid op).
        nan_cell_f32(6, "log", (-1.0f32).to_bits(), A_LOG),
        nan_cell_f32(7, "log", (-2.0f32).to_bits(), A_LOG),
        nan_cell_f32(8, "log", f32::NEG_INFINITY.to_bits(), A_LOG),
        nan_cell_f32(9, "sin", f32::INFINITY.to_bits(), A_SIN),
        nan_cell_f32(10, "sin", f32::NEG_INFINITY.to_bits(), A_SIN),
        nan_cell_f64(11, "log", (-1.0f64).to_bits(), A_LOG),
        nan_cell_f64(12, "sin", f64::INFINITY.to_bits(), A_SIN),
    ];
    bundle_doc("OPS", "KISS-CONFORM-6.8-0010", &cells)
}

/// Assemble a bundle document from its cells. Header strings are shared across bundles so the
/// arith bundle regenerates byte-for-byte.
fn bundle_doc(substandard: &str, spec_clause: &str, cells: &[String]) -> String {
    let mut doc = String::new();
    doc.push_str("{\n");
    doc.push_str("  \"schema\": \"kiss-oracle-vectors-v1.json\",\n");
    doc.push_str(&format!("  \"kiss_substandard\": \"{substandard}\",\n"));
    doc.push_str("  \"schema_version\": 1,\n");
    doc.push_str(&format!("  \"spec_clause\": \"{spec_clause}\",\n"));
    doc.push_str(&format!("  \"generator\": \"{GENERATOR}\",\n"));
    doc.push_str(&format!("  \"number_of_vectors\": {},\n", cells.len()));
    doc.push_str("  \"byte_order\": \"hex is the value's bytes most-significant first, left to right\",\n");
    doc.push_str("  \"hex_encoding\": \"uppercase hex bytes; ' ' and '\u{00b7}' are grouping marks (lib.rs::parse_hex)\",\n");
    doc.push_str("  \"ulp_metric\": \"integer totalOrder distance (lib.rs::ulp_distance_f32)\",\n");
    doc.push_str("  \"vectors\": [\n");
    doc.push_str(&cells.join(",\n"));
    doc.push_str("\n  ]\n}\n");
    doc
}

fn write_bundle(rel_path: &str, doc: &str) {
    let path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), rel_path);
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, doc).unwrap();
    eprintln!("wrote {path}");
}

fn main() {
    write_bundle("/corpus/ops-arith.json", &arith_doc());
    write_bundle("/corpus/ops-transcendental-nan.json", &transcendental_nan_doc());
}
