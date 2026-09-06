//! kiss_mint — mints the frozen oracle-vector corpus from the reference oracle. Three bundles:
//!   corpus/ops-arith.json              — Plan A slice: exact-byte `add` cells (incl. the
//!                                        signed-zero distinctions), provenance `oracle`.
//!   corpus/ops-transcendental-nan.json — T9/#352: transcendental COMPUTED-NaN cells — a
//!                                        computed NaN must be QUIET (§6.16-0010 / §6.8-0010).
//!   corpus/ops-narrow-move-nan.json    — §6.16-0009/-0011: narrow-float sign-edit MOVE cells —
//!                                        a moved NaN's bits (payload + post-edit sign) survive
//!                                        EXACTLY (no decline member; the vector is the enforcement).
//! All are the Wycheproof-shaped JSON of
//! docs/superpowers/specs/2026-07-19-kiss-oracle-vector-corpus-design.md §4.

use kiss_conformance::fp::{f32_to_bf16, f32_to_e4m3, f32_to_e5m2};
use kiss_conformance::nan_provenance::is_nan;
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

// ---- narrow-float sign-edit MOVE bundle (§6.16-0009/-0011, increment 1) --------------------
//
// A narrow-float neg/abs/copysign MOVES its operand's bits with a single sign-bit edit — every
// other bit, a moved NaN's payload included, survives EXACTLY (§6.16-0009). There is NO decline
// member: a conforming impl spells the op raw-bit, it does not refuse, so the vector IS the
// enforcement (Unpopped's fp8 defect + Baracuda's sm_89 hardware, independently). The bit the
// discrimination turns on: a promote-through-f32 impl QUIETS a signaling NaN (and a canonicalizing
// one drops a rich payload), so its output differs from these exact-byte expecteds.
//
// POPULATION / LIMITS (stated, not omitted): f16 is authored (raw-bit needs no conversion) though
// the promote-through-f32 demo cannot run for it (fp has no f16<->f32); f8e4m3fn has a single NaN
// encoding (§6.16-0004) so it carries no payload and the MOVE is non-discriminating — authored for
// completeness, vacuous. Selection ops (min/max) are increment 2.

/// The raw-bit MOVE oracle: the sign bit is the MSB of big-endian byte 0 for every narrow float
/// here. neg flips it, abs clears it, copysign takes `a`'s magnitude and `b`'s sign.
fn move_edit(op: &str, a: &[u8], b: Option<&[u8]>) -> Vec<u8> {
    let mut out = a.to_vec();
    match op {
        "neg" => out[0] ^= 0x80,
        "abs" => out[0] &= 0x7F,
        "copysign" => {
            out[0] &= 0x7F;
            out[0] |= b.expect("copysign needs b")[0] & 0x80;
        }
        _ => panic!("unknown sign-edit op {op}"),
    }
    out
}

/// Emit one MOVE cell (exact-byte class). A NaN output carries `nan_provenance: moved` (the
/// biconditional pairs it with the NaN); a finite output carries none. `inputs` is 1 (role `x`)
/// or 2 (roles `a`,`b`). Shared by the sign-edit and selection minters.
fn emit_cell(tc: u32, op: &str, dtype: &str, inputs: &[&[u8]], expected: &[u8], tags: &str) -> String {
    let inputs_json = match inputs {
        [x] => format!("[{{\"role\":\"x\",\"dtype\":\"{dtype}\",\"bits\":\"{}\"}}]", hex(x)),
        [a, b] => format!(
            "[{{\"role\":\"a\",\"dtype\":\"{dtype}\",\"bits\":\"{}\"}}, \
             {{\"role\":\"b\",\"dtype\":\"{dtype}\",\"bits\":\"{}\"}}]",
            hex(a),
            hex(b)
        ),
        _ => panic!("a cell has 1 or 2 inputs"),
    };
    let prov = if is_nan(dtype, expected) { " \"nan_provenance\": \"moved\"," } else { "" };
    let prec = if inputs[0].len() == 2 { 16 } else { 8 };
    format!(
        "    {{\"tcId\": {tc}, \"op\": \"{op}\", \"dtype\": \"{dtype}\", \"rounding\": \"roundTiesToEven\", \
         \"inputs\": {inputs_json}, \
         \"expected\": {{\"dtype\":\"{dtype}\",\"bits\":\"{}\"}}, \
         \"class\": \"exact-byte\", \"ulp_bound\": 0, \"provenance\": \"oracle\", \
         \"tags\": [{tags}],{prov} \
         \"certificate\": {{\"hardness_margin_bits\": 0, \"stabilized_precision_bits\": {prec}}}}}",
        hex(expected)
    )
}

/// One sign-edit MOVE cell (neg/abs/copysign), expected computed by the raw-bit sign edit.
fn move_cell(tc: u32, op: &str, dtype: &str, a: &[u8], b: Option<&[u8]>, tags: &str) -> String {
    let expected = move_edit(op, a, b);
    match b {
        Some(bb) => emit_cell(tc, op, dtype, &[a, bb], &expected, tags),
        None => emit_cell(tc, op, dtype, &[a], &expected, tags),
    }
}

fn narrow_move_nan_doc() -> String {
    struct D {
        name: &'static str,
        bytes: usize,
        nans: &'static [(&'static str, u16)],
    }
    // Payloads: a signaling NaN, a quiet NaN with a RICH payload, and a negative NaN — so a
    // quieting impl reds the signaling cell and a canonicalizing/truncating impl reds the rich one.
    let dtypes = [
        D { name: "bf16", bytes: 2, nans: &[("snan", 0x7F81), ("qnan-rich", 0x7FD5), ("neg-qnan-rich", 0xFFD5)] },
        D { name: "f16", bytes: 2, nans: &[("snan", 0x7C01), ("qnan-rich", 0x7E55), ("neg-qnan-rich", 0xFE55)] },
        D { name: "f8e5m2", bytes: 1, nans: &[("snan", 0x7D), ("qnan-rich", 0x7F), ("neg-snan", 0xFD)] },
        // f8e4m3fn: single NaN encoding (S.1111.111) — 0x7F / 0xFF, no payload variety.
        D { name: "f8e4m3fn", bytes: 1, nans: &[("nan", 0x7F), ("neg-nan", 0xFF)] },
    ];
    let bytes_of = |val: u16, n: usize| -> Vec<u8> {
        if n == 2 { val.to_be_bytes().to_vec() } else { vec![val as u8] }
    };
    let mut cells = Vec::new();
    let mut tc = 1u32;
    for d in &dtypes {
        for (pname, val) in d.nans {
            let a = bytes_of(*val, d.bytes);
            let tags = format!("\"moved-nan\",\"sign-edit\",\"{pname}\",\"6.16-0009\"");
            cells.push(move_cell(tc, "neg", d.name, &a, None, &tags));
            tc += 1;
            cells.push(move_cell(tc, "abs", d.name, &a, None, &tags));
            tc += 1;
        }
        // copysign: the first NaN's magnitude with a negative sign source (sign bit only).
        let nan0 = bytes_of(d.nans[0].1, d.bytes);
        let neg_sign = bytes_of(if d.bytes == 2 { 0x8000 } else { 0x80 }, d.bytes);
        cells.push(move_cell(
            tc,
            "copysign",
            d.name,
            &nan0,
            Some(&neg_sign),
            "\"moved-nan\",\"sign-edit\",\"copysign-neg\",\"6.16-0009\"",
        ));
        tc += 1;
    }
    // Finite negative controls: neg(1.0) — a finite passes a promote-through-f32 impl too, so the
    // set does not red on everything. Encodings for convertible dtypes come from the fp oracle
    // (guaranteed representable); f16's is the standard binary16 1.0 (no fp f16 conversion exists).
    for (name, a) in [
        ("bf16", f32_to_bf16(1.0).to_be_bytes().to_vec()),
        ("f8e5m2", vec![f32_to_e5m2(1.0)]),
        ("f8e4m3fn", vec![f32_to_e4m3(1.0)]),
        ("f16", 0x3C00u16.to_be_bytes().to_vec()),
    ] {
        cells.push(move_cell(tc, "neg", name, &a, None, "\"finite-control\",\"sign-edit\",\"6.16-0009\""));
        tc += 1;
    }
    bundle_doc("OPS", "KISS-OPS-6.16-0009", &cells)
}

// ---- narrow-float SELECTION MOVE bundle (§6.16-0009/-0011/§6.15, increment 2) --------------
//
// min/max is NOT a uniform family (corrected after Baracuda's sm_89 probe):
//   - PROPAGATING (max_prop/min_prop): the NaN operand is selected → MOVED, payload+sign exact.
//   - IEEE (fmax_ieee/fmin_ieee): NaN-SUPPRESSING → returns the OTHER operand; the NaN is
//     discarded, not moved, so the SURVIVOR's bits are the obligation (a payload-preservation
//     vector here would over-constrain a conforming impl).
//   - both-NaN under an IEEE arm: `f{max,min}_ieee(a,b)` returns `b` (§6.15 decomposition) — the
//     one case a NaN survives; the oracle authors it, not assumed equal to prop.
// Every cell here has a NaN operand, so the §6.15 NaN arm decides and no value comparison runs.

/// The raw-bit SELECTION oracle: which operand's bits move to the output (NaN arm only).
fn select_move(op: &str, a: &[u8], b: &[u8], dtype: &str) -> Vec<u8> {
    let (an, bn) = (is_nan(dtype, a), is_nan(dtype, b));
    let pick: &[u8] = match op {
        "max_prop" | "min_prop" => {
            if an { a } else if bn { b } else { panic!("select cell needs a NaN operand") }
        }
        "fmax_ieee" | "fmin_ieee" => {
            if an { b } else if bn { a } else { panic!("select cell needs a NaN operand") }
        }
        _ => panic!("unknown select op {op}"),
    };
    pick.to_vec()
}

fn select_cell(tc: u32, op: &str, dtype: &str, a: &[u8], b: &[u8], tags: &str) -> String {
    let expected = select_move(op, a, b, dtype);
    emit_cell(tc, op, dtype, &[a, b], &expected, tags)
}

fn narrow_select_nan_doc() -> String {
    struct D {
        name: &'static str,
        bytes: usize,
        snan: u16,      // signaling NaN (or the sole NaN for f8e4m3fn)
        qnan_rich: u16, // quiet NaN, rich payload (or the neg NaN for f8e4m3fn)
        one: u16,       // a representable finite (1.0), the non-NaN operand
    }
    let dtypes = [
        D { name: "bf16", bytes: 2, snan: 0x7F81, qnan_rich: 0x7FD5, one: f32_to_bf16(1.0) },
        D { name: "f16", bytes: 2, snan: 0x7C01, qnan_rich: 0x7E55, one: 0x3C00 },
        D { name: "f8e5m2", bytes: 1, snan: 0x7D, qnan_rich: 0x7F, one: f32_to_e5m2(1.0) as u16 },
        D { name: "f8e4m3fn", bytes: 1, snan: 0x7F, qnan_rich: 0xFF, one: f32_to_e4m3(1.0) as u16 },
    ];
    let bytes_of = |val: u16, n: usize| -> Vec<u8> {
        if n == 2 { val.to_be_bytes().to_vec() } else { vec![val as u8] }
    };
    let mut cells = Vec::new();
    let mut tc = 1u32;
    for d in &dtypes {
        let nan = bytes_of(d.snan, d.bytes);
        let other = bytes_of(d.qnan_rich, d.bytes);
        let one = bytes_of(d.one, d.bytes);
        // PROPAGATING: the NaN moves, both operand positions.
        for op in ["max_prop", "min_prop"] {
            cells.push(select_cell(tc, op, d.name, &nan, &one, "\"moved-nan\",\"select-propagate\",\"6.16-0009\"")); tc += 1;
            cells.push(select_cell(tc, op, d.name, &one, &nan, "\"moved-nan\",\"select-propagate\",\"6.16-0009\"")); tc += 1;
        }
        // IEEE: suppress the NaN, return the finite survivor (both positions).
        for op in ["fmax_ieee", "fmin_ieee"] {
            cells.push(select_cell(tc, op, d.name, &nan, &one, "\"moved-survivor\",\"select-ieee-suppress\",\"6.15\"")); tc += 1;
            cells.push(select_cell(tc, op, d.name, &one, &nan, "\"moved-survivor\",\"select-ieee-suppress\",\"6.15\"")); tc += 1;
            // both-NaN: returns b (the second operand) — distinct payloads so the survivor is observable.
            cells.push(select_cell(tc, op, d.name, &nan, &other, "\"moved-nan\",\"select-ieee-both-nan\",\"6.15\"")); tc += 1;
        }
    }
    bundle_doc("OPS", "KISS-OPS-6.16-0009", &cells)
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
    write_bundle("/corpus/ops-narrow-move-nan.json", &narrow_move_nan_doc());
    write_bundle("/corpus/ops-narrow-select-nan.json", &narrow_select_nan_doc());
}
