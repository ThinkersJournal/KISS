// SPDX-License-Identifier: MIT OR Apache-2.0
//! kiss_mint — mints the frozen oracle-vector corpus from the reference oracle. Three bundles:
//!   corpus/ops-arith.json              — Plan A slice: exact-byte `add` cells (incl. the
//!                                        signed-zero distinctions), provenance `oracle`.
//!   corpus/ops-transcendental-nan.json — T9/#352: transcendental COMPUTED-NaN cells — a
//!                                        computed NaN must be QUIET (§6.16-0010 / §6.8-0010).
//!   corpus/ops-rounding.json          — KISS-OPS-6.7-0001/-0002: f32 floor/ceil/trunc/round_even exact-byte
//!                                        cells (ties, zero signs, integral, inf, subnormal); no NaN (#396).
//!   corpus/ops-int-wrap.json          — KISS-OPS-6.2-0002: i8..u64 add/sub/mul wrap at the boundaries
//!   corpus/ops-compare-bool.json      — KISS-OPS-6.2-0005/-0006: comparison -> one-byte bool mask; bool consumers read non-zero as true
//!   corpus/ops-atan2-quadrants.json   — KISS-OPS-6.9-0001: f32 atan2 four-quadrant + signed-zero + inf cells, class ULP (never exact-byte, §6.8-0005)
//!   corpus/ops-sqrt-rounding.json     — KISS-OPS-6.8-0003: f32 sqrt, class ULP bound 0 (the correctly-rounded arm; hard cases, exact squares, subnormals, +-0, +inf)
//!   corpus/ops-nextafter.json         — KISS-OPS-6.9-0003 (value arm): f32/f64 nextafter on the dtype's own lattice, exact-byte (zero crossing, signed zeros, subnormal/normal boundary, max finite <-> inf)
//!   corpus/ops-move-wide-nan.json     — informative, NO consumer obligation: f32/f64 neg/abs/copysign NaN-MOVE cells (the wide counterpart of ops-narrow-move-nan)
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
        // Family B, f64 (appended: tcIds stay monotone). f64 `log` of a signaling NaN: Fuel measured Apple's f32 `logf` returning the sNaN UNCHANGED
        // (a platform-libm result that a decomposition-computed reference would not give); the f64
        // `log` was unmeasured because this corpus had no cell for it.
        nan_cell_f64(13, "log", SNAN64, B),
    ];
    // The shared `ulp_metric` header says "totalOrder distance"; for THESE cells that is the wrong
    // comparator (a first-time reader, Fuel 2026-10-14, failed log(-1) on a negative qNaN).
    const RULE: &str = "every cell here has class ULP and a NaN expected (nan_provenance computed): that is a COMPUTED NaN \
(KISS-CONFORM-6.8-0010). Compare NaN-ness and QUIETNESS only; do NOT compare totalOrder distance, and never the sign or \
payload. ulp_bound applies to a non-NaN expected value only.";
    bundle_doc_with_rule("OPS", "KISS-CONFORM-6.8-0010", Some(RULE), &cells)
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

// ---- rounding-atom bundle (KISS-OPS-6.7-0001 / -0002) ---------------------------------------
//
// POPULATION — stated so the gaps are visible. The four rounding atoms are PRIMITIVE-FLOOR (§6.7:
// no decomposition by design), so the only thing that can pin them across implementations is a
// vector; before this bundle the oracle corpus carried none for `floor`/`ceil`/`trunc`/`round_even`.
//
// EMITTED (f32, exact-byte, unary): for each of the four ops, the inputs that separate the
// directions and the zero-sign edges — +-0, +-0.5 (`trunc(-0.5)` and `ceil(-0.5)` are `-0.0`: an
// impl that returns `+0.0` is wrong), the ties +-0.5/1.5/2.5 (`round_even` is banker's, not
// half-away), values one ulp either side of an integer, the 2^23 boundary where every f32 is
// already integral, the largest finite, +-inf (fixed points), and the smallest subnormals.
//
// NOT EMITTED, on purpose: a NaN operand. §6.7-0002 pins "propagates a NaN" but not WHICH NaN
// (payload / quietness) — issue #396. A byte-exact cell would over-constrain a conforming impl.
// narrow dtypes, f64 and the §6.17 fidelity axis are not covered either.
fn rounding_doc() -> String {
    let inputs: [(u32, &str); 18] = [
        (0x0000_0000, "signed-zero"),   // +0
        (0x8000_0000, "signed-zero"),   // -0
        (0x3F00_0000, "half"),          // +0.5
        (0xBF00_0000, "half"),          // -0.5
        (0x3FC0_0000, "tie"),           // +1.5
        (0xBFC0_0000, "tie"),           // -1.5
        (0x4020_0000, "tie"),           // +2.5
        (0xC020_0000, "tie"),           // -2.5
        (0x3EFF_FFFF, "near-integer"),  // 0.49999997 (just below 0.5)
        (0x3F80_0001, "near-integer"),  // 1.0000001 (just above 1)
        (0xBF7F_FFFF, "near-integer"),  // -0.99999994 (just above -1)
        (0x4B00_0000, "integral"),      // 2^23: every f32 here is already an integer
        (0x4AFF_FFFF, "near-integer"),  // 2^23 - 0.5
        (0x7F7F_FFFF, "integral"),      // largest finite
        (0x7F80_0000, "inf"),           // +inf
        (0xFF80_0000, "inf"),           // -inf
        (0x0000_0001, "subnormal"),     // smallest positive subnormal
        (0x8000_0001, "subnormal"),     // smallest negative subnormal
    ];
    let mut cells = Vec::new();
    let mut tc = 1u32;
    for op in ["floor", "ceil", "trunc", "round_even"] {
        for (bits, tag) in inputs {
            let x = f32::from_bits(bits);
            let r = match op {
                "floor" => semantics::floor(x),
                "ceil" => semantics::ceil(x),
                "trunc" => semantics::trunc(x),
                _ => semantics::round_even(x),
            };
            cells.push(emit_cell(
                tc,
                op,
                "f32",
                &[&bits.to_be_bytes()],
                &r.to_bits().to_be_bytes(),
                &format!("\"{tag}\",\"6.7-0001\""),
            ));
            tc += 1;
        }
    }
    bundle_doc("OPS", "KISS-OPS-6.7-0001", &cells)
}

// ---- integer-wrapping bundle (KISS-OPS-6.2-0002) -------------------------------------------
//
// WHY: §6.2-0002 pins integer add/sub/mul as WRAPPING two's-complement modulo 2^bits — not
// saturating, not trapping — and the corpus carried no integer cell at all (ops-arith.json is f32).
// An implementation that saturates passes every float vector.
//
// EMITTED: for each of i8/u8/i16/u16/i32/u32/i64/u64, the five raw bit patterns that sit on the
// wrap boundaries — 0, 1, 2^(n-1)-1 (signed max), 2^(n-1) (signed min), 2^n-1 (unsigned max / -1)
// — in every ordered pair for `sub` (not commutative) and every unordered pair for `add`/`mul`.
// Add, sub and mul on raw two's-complement patterns are the SAME bit operation for signed and
// unsigned lanes, so one pattern list serves both; what differs is how a reader interprets the
// cell, and `tests/mint_int_wrap.rs` interprets it both ways against an i128 model.
// NOT EMITTED: i4/u4 (sub-byte packing is its own clause), division/shift, and the §6.2-0002 words
// "MUST NOT be undefined behavior" — a vector can only show the defined result, not the absence of UB.
fn int_wrap_doc() -> String {
    let lanes: [(&str, u32); 8] =
        [("i8", 8), ("u8", 8), ("i16", 16), ("u16", 16), ("i32", 32), ("u32", 32), ("i64", 64), ("u64", 64)];
    let mut cells = Vec::new();
    let mut tc = 1u32;
    for (dtype, bits) in lanes {
        let mask: u64 = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        let pats: [u64; 5] = [0, 1, (1u64 << (bits - 1)) - 1, 1u64 << (bits - 1), mask];
        let nbytes = (bits / 8) as usize;
        let be = |v: u64| v.to_be_bytes()[8 - nbytes..].to_vec();
        for op in ["add", "sub", "mul"] {
            for (i, &a) in pats.iter().enumerate() {
                for (j, &b) in pats.iter().enumerate() {
                    if op != "sub" && j < i {
                        continue; // commutative: unordered pairs only
                    }
                    let r = match op {
                        "add" => a.wrapping_add(b),
                        "sub" => a.wrapping_sub(b),
                        _ => a.wrapping_mul(b),
                    } & mask;
                    cells.push(emit_cell(tc, op, dtype, &[&be(a), &be(b)], &be(r), "\"int-wrap\",\"6.2-0002\""));
                    tc += 1;
                }
            }
        }
    }
    bundle_doc("OPS", "KISS-OPS-6.2-0002", &cells)
}

// ---- atan2 quadrant bundle (KISS-OPS-6.9-0001) -----------------------------------------------
//
// WHY: §6.9-0001 pins `atan2(y=a, x=b)` as the FOUR-quadrant arctangent with the IEEE 754 +-0
// conventions (the sign of a zero operand selects the quadrant). `atan(y/x)` passes every x > 0 cell
// and fails every x < 0 / x = +-0 cell; the corpus carried no `atan2` cell at all.
//
// CLASS: ULP, never exact-byte (§6.8-0005 / `test_ops_atan2_class_is_ulp`). The expected value is
// libm-backed: f64 `atan2` rounded once to f32. That is NOT a correctly-rounded oracle (there is no
// `atan2` hp atom yet), so `ulp_bound` is the declared tier (4), and every ordinary cell sits well
// away from a rounding boundary; the special cells (+-0, +-inf) are fixed by IEEE 754 Annex F
// (+-pi, +-pi/2, +-pi/4, +-3pi/4, +-0) and carry the same class.
//
// EMITTED: one ordinary cell per quadrant and per axis direction, the signed-zero matrix
// (y,x in {+0,-0}), y = +-0 against a finite x of each sign, x = +-0 against a finite y of each sign,
// and the +-inf matrix. NOT EMITTED: NaN operands (which NaN is #396's question), f64/narrow dtypes.
/// The (y, x, tag) operand bit patterns of the atan2 bundle.
fn atan2_inputs() -> [(u32, u32, &'static str); 28] {
    const POS0: u32 = 0x0000_0000;
    const NEG0: u32 = 0x8000_0000;
    const ONE: u32 = 0x3F80_0000;
    const NEG1: u32 = 0xBF80_0000;
    const TWO: u32 = 0x4000_0000;
    const NEG2: u32 = 0xC000_0000;
    const PINF: u32 = 0x7F80_0000;
    const NINF: u32 = 0xFF80_0000;
    [
        (ONE, ONE, "quadrant-1"),
        (ONE, NEG1, "quadrant-2"),
        (NEG1, NEG1, "quadrant-3"),
        (NEG1, ONE, "quadrant-4"),
        (TWO, ONE, "quadrant-1"),
        (ONE, NEG2, "quadrant-2"),
        (NEG2, NEG1, "quadrant-3"),
        (NEG1, TWO, "quadrant-4"),
        (POS0, POS0, "signed-zero"),
        (POS0, NEG0, "signed-zero"),
        (NEG0, POS0, "signed-zero"),
        (NEG0, NEG0, "signed-zero"),
        (POS0, ONE, "y-zero"),
        (NEG0, ONE, "y-zero"),
        (POS0, NEG1, "y-zero"),
        (NEG0, NEG1, "y-zero"),
        (ONE, POS0, "x-zero"),
        (ONE, NEG0, "x-zero"),
        (NEG1, POS0, "x-zero"),
        (NEG1, NEG0, "x-zero"),
        (PINF, PINF, "inf"),
        (PINF, NINF, "inf"),
        (NINF, PINF, "inf"),
        (NINF, NINF, "inf"),
        (ONE, PINF, "inf"),
        (ONE, NINF, "inf"),
        (PINF, ONE, "inf"),
        (NINF, NEG1, "inf"),
    ]
}

fn atan2_doc() -> String {
    let mut cells = Vec::new();
    for (i, (yb, xb, tag)) in atan2_inputs().iter().enumerate() {
        let r = f64::atan2(f32::from_bits(*yb) as f64, f32::from_bits(*xb) as f64) as f32;
        cells.push(format!(
            "    {{\"tcId\": {tc}, \"op\": \"atan2\", \"dtype\": \"f32\", \"rounding\": \"roundTiesToEven\",              \"inputs\": [{{\"role\":\"a\",\"dtype\":\"f32\",\"bits\":\"{}\"}},              {{\"role\":\"b\",\"dtype\":\"f32\",\"bits\":\"{}\"}}],              \"expected\": {{\"dtype\":\"f32\",\"bits\":\"{}\"}},              \"class\": \"ULP\", \"ulp_bound\": 4, \"provenance\": \"oracle\",              \"tags\": [\"{tag}\",\"6.9-0001\"],              \"certificate\": {{\"hardness_margin_bits\": 0, \"stabilized_precision_bits\": 24}}}}",
            hex(&yb.to_be_bytes()),
            hex(&xb.to_be_bytes()),
            hex(&r.to_bits().to_be_bytes()),
            tc = i + 1,
        ));
    }
    bundle_doc("OPS", "KISS-OPS-6.9-0001", &cells)
}

// ---- sqrt correct-rounding bundle (KISS-OPS-6.8-0003) ------------------------------------------
//
// WHY: §6.8-0003 pins `sqrt` as CORRECTLY ROUNDED on any target that guarantees it. A `sqrt` built as
// `x * rsqrt(x)` or `exp(0.5 * log x)` passes a 2-ULP gate and is wrong by one ulp on specific inputs.
//
// SCOPE — what a vector can and cannot say: the clause has two arms. This bundle pins the FIRST
// (correctly rounded: class ULP, `ulp_bound` 0, i.e. the exact result). A target that does not guarantee
// correct rounding is held to its DECLARED tier (§6.8-0001) by the contract, not by this file; such a
// target is expected to fail these cells, and a runner applies them only where the guarantee is claimed.
// The expected value is Rust `f32::sqrt` (IEEE 754 correctly rounded on every Rust target); the test
// recomputes each cell from an independent integer square root. Under the ULP comparator a zero result's
// SIGN is not discriminated (totalOrder distance 0 between +0 and -0), so `sqrt(-0) = -0` is carried in the
// cell but not enforced by it.
//
// EMITTED: exact squares, non-squares at odd and even exponents, one ulp either side of 1, 2 and 4, the
// largest finite, the smallest normal, the largest and smallest subnormal, +-0, +inf.
// NOT EMITTED: negative operands and NaN (a NaN is minted; which NaN is #396), f64 and narrow dtypes.
fn sqrt_inputs() -> [(u32, &'static str); 24] {
    [
        (0x0000_0000, "zero"),
        (0x8000_0000, "signed-zero"),
        (0x3F80_0000, "exact-square"), // 1
        (0x4080_0000, "exact-square"), // 4
        (0x3E80_0000, "exact-square"), // 0.25
        (0x4010_0000, "exact-square"), // 2.25
        (0x3F80_0001, "one-ulp"),      // 1 + ulp
        (0x3F7F_FFFF, "one-ulp"),      // 1 - ulp/2
        (0x3F80_0003, "one-ulp"),
        (0x4000_0000, "irrational"), // 2
        (0x4000_0001, "irrational"), // 2 + ulp
        (0x4040_0000, "irrational"), // 3
        (0x4081_0000, "irrational"), // 4.03125
        (0x4049_0FDB, "irrational"), // pi
        (0x4120_0000, "irrational"), // 10
        (0x3F00_0000, "irrational"), // 0.5
        (0x7F7F_FFFF, "largest-finite"),
        (0x7F7F_FFFE, "large"),
        (0x0080_0000, "smallest-normal"),
        (0x0080_0001, "smallest-normal"),
        (0x007F_FFFF, "subnormal"), // largest subnormal
        (0x0000_0001, "subnormal"), // smallest subnormal
        (0x0000_0003, "subnormal"),
        (0x7F80_0000, "inf"),
    ]
}

fn sqrt_cell(tc: usize, bits: u32, tag: &str) -> String {
    let r = f32::from_bits(bits).sqrt();
    format!(
        "    {{\"tcId\": {tc}, \"op\": \"sqrt\", \"dtype\": \"f32\", \"rounding\": \"roundTiesToEven\", \
         \"inputs\": [{{\"role\":\"x\",\"dtype\":\"f32\",\"bits\":\"{}\"}}], \
         \"expected\": {{\"dtype\":\"f32\",\"bits\":\"{}\"}}, \
         \"class\": \"ULP\", \"ulp_bound\": 0, \"provenance\": \"oracle\", \
         \"tags\": [\"{tag}\",\"6.8-0003\"], \
         \"certificate\": {{\"hardness_margin_bits\": 0, \"stabilized_precision_bits\": 24}}}}",
        hex(&bits.to_be_bytes()),
        hex(&r.to_bits().to_be_bytes()),
    )
}

fn sqrt_doc() -> String {
    let cells: Vec<String> = sqrt_inputs().iter().enumerate().map(|(i, (b, t))| sqrt_cell(i + 1, *b, t)).collect();
    bundle_doc("OPS", "KISS-OPS-6.8-0003", &cells)
}

// ---- nextafter value-arm bundle (KISS-OPS-6.9-0003) --------------------------------------------
//
// WHY: §6.9-0003 pins `nextafter(a, b)` as the next representable value after `a` toward `b` in the
// dtype's OWN lattice. A promote-through-f64 `nextafter` returns the f64 neighbour of an f32 value,
// which is not a representable f32 step; and the zero / subnormal / infinity edges are where hand-rolled
// bit stepping goes wrong.
//
// SCOPE: only the VALUE half. The clause's second half (reject f16/bf16/f8e4m3fn/f8e5m2 with a typed
// decline) has no vector form (a vector carries an expected VALUE, not a decline), so 6.9-0003 is counted
// PARTIAL by the ledger, not backed. Class exact-byte (determinism.rs: `nextafter` is exact-byte).
// EMITTED: f32 and f64, each over the same 14 (a, b) shapes plus 6 raw-bit edge pairs (40 cells). NOT EMITTED: NaN operands (#396).
const NEXTAFTER_SHAPES: [(&str, [f64; 2]); 14] = [
    ("up", [1.0, 2.0]),
    ("down", [1.0, 0.0]),
    ("equal", [1.0, 1.0]),
    ("zero-up", [0.0, 1.0]),
    ("zero-down", [0.0, -1.0]),
    ("negative-up", [-1.0, 0.0]),
    ("pow2-down", [2.0, 1.0]),
    ("to-inf", [1.0, f64::INFINITY]),
    ("to-neg-inf", [1.0, f64::NEG_INFINITY]),
    ("inf-down", [f64::INFINITY, 0.0]),
    ("neg-inf-up", [f64::NEG_INFINITY, 0.0]),
    ("signed-zero-equal", [0.0, -0.0]),
    ("signed-zero-equal", [-0.0, 0.0]),
    ("negative-down", [-1.0, -2.0]),
];

fn next_toward_f32(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == b {
        return b;
    }
    if a == 0.0 {
        let tiny = f32::from_bits(1);
        return if b > 0.0 { tiny } else { -tiny };
    }
    let bits = a.to_bits();
    let up = (b > a) == (a > 0.0);
    f32::from_bits(if up { bits + 1 } else { bits - 1 })
}

fn next_toward_f64(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == b {
        return b;
    }
    if a == 0.0 {
        let tiny = f64::from_bits(1);
        return if b > 0.0 { tiny } else { -tiny };
    }
    let bits = a.to_bits();
    let up = (b > a) == (a > 0.0);
    f64::from_bits(if up { bits + 1 } else { bits - 1 })
}

/// The extra edge pairs expressed as raw bit patterns (they are not exact f64 literals in both lanes).
fn nextafter_edge_pairs() -> [(&'static str, (u64, u64), (u32, u32)); 6] {
    [
        ("max-to-inf", (0x7FEF_FFFF_FFFF_FFFF, 0x7FF0_0000_0000_0000), (0x7F7F_FFFF, 0x7F80_0000)),
        ("neg-max-to-neg-inf", (0xFFEF_FFFF_FFFF_FFFF, 0xFFF0_0000_0000_0000), (0xFF7F_FFFF, 0xFF80_0000)),
        ("min-subnormal-to-zero", (0x0000_0000_0000_0001, 0), (0x0000_0001, 0)),
        ("neg-min-subnormal-to-zero", (0x8000_0000_0000_0001, 0), (0x8000_0001, 0)),
        ("min-normal-down", (0x0010_0000_0000_0000, 0), (0x0080_0000, 0)),
        ("max-subnormal-up", (0x000F_FFFF_FFFF_FFFF, 0x0010_0000_0000_0000), (0x007F_FFFF, 0x0080_0000)),
    ]
}

fn nextafter_doc() -> String {
    let mut cells = Vec::new();
    let mut tc = 1u32;
    let mut push = |dtype: &str, a: &[u8], b: &[u8], r: &[u8], tag: &str| {
        let tags = format!("\"{tag}\",\"6.9-0003\"");
        cells.push(emit_cell(tc, "nextafter", dtype, &[a, b], r, &tags));
        tc += 1;
    };
    for (tag, [a, b]) in NEXTAFTER_SHAPES {
        let (a32, b32) = (a as f32, b as f32);
        let r32 = next_toward_f32(a32, b32);
        push("f32", &a32.to_bits().to_be_bytes(), &b32.to_bits().to_be_bytes(), &r32.to_bits().to_be_bytes(), tag);
        let r64 = next_toward_f64(a, b);
        push("f64", &a.to_bits().to_be_bytes(), &b.to_bits().to_be_bytes(), &r64.to_bits().to_be_bytes(), tag);
    }
    for (tag, (a64, b64), (a32, b32)) in nextafter_edge_pairs() {
        let r32 = next_toward_f32(f32::from_bits(a32), f32::from_bits(b32));
        push("f32", &a32.to_be_bytes(), &b32.to_be_bytes(), &r32.to_bits().to_be_bytes(), tag);
        let r64 = next_toward_f64(f64::from_bits(a64), f64::from_bits(b64));
        push("f64", &a64.to_be_bytes(), &b64.to_be_bytes(), &r64.to_bits().to_be_bytes(), tag);
    }
    bundle_doc("OPS", "KISS-OPS-6.9-0003", &cells)
}

// ---- comparison-mask + bool-normalization bundle (KISS-OPS-6.2-0005 / -0006) -----------------
//
// WHY: §6.2-0005 pins a comparison's result as a one-byte `bool` mask, byte 0 or 1 — "an `f32`
// comparison yields a one-byte mask, not a 4-byte `1.0`/`0.0`" — and §6.2-0006 pins that a `bool`
// consumer reads ANY non-zero byte as true while a `bool` producer normalizes to strictly 0/1. The
// corpus carried no comparison cell and no `bool` cell, so an implementation returning `1.0f32` or
// forwarding `0x02` passed every vector.
//
// EMITTED: (a) cmp_eq/ne/lt/le/gt/ge over f32 (ordinary, equal, signed zeros, NaN either side, inf),
// i32 and u32 — the u32 cells use the sign-bit pattern 0xFFFFFFFF vs 1 so a signed reading is
// observable (§6.2-0007's comparison path; the clause is NOT backed here: its bitwise/reduction/scan
// arms are not covered); (b) logical_and/logical_or/logical_not on `bool` operands holding non-0/1
// non-zero bytes (0x02, 0x7F, 0x80, 0xFF), results strictly 0/1.
// EXPECTED dtype is `bool`, one byte, whatever the compute dtype. NOT EMITTED: arrays/masks wider than one
// element, `select`/`reduce` consumers of a mask, and the "MUST NOT produce any other byte value" half as a
// REJECTION test (a vector can only show the defined result; the discrimination is the one-byte 0/1 expected).
fn typed_cell(tc: u32, op: &str, dtype: &str, inputs: &[&[u8]], expected_dtype: &str, expected: &[u8], tags: &str) -> String {
    let inputs_json = match inputs {
        [x] => format!("[{{\"role\":\"x\",\"dtype\":\"{dtype}\",\"bits\":\"{}\"}}]", hex(x)),
        [a, b] => format!(
            "[{{\"role\":\"a\",\"dtype\":\"{dtype}\",\"bits\":\"{}\"}},              {{\"role\":\"b\",\"dtype\":\"{dtype}\",\"bits\":\"{}\"}}]",
            hex(a),
            hex(b)
        ),
        _ => panic!("a cell has 1 or 2 inputs"),
    };
    format!(
        "    {{\"tcId\": {tc}, \"op\": \"{op}\", \"dtype\": \"{dtype}\", \"rounding\": \"roundTiesToEven\",          \"inputs\": {inputs_json},          \"expected\": {{\"dtype\":\"{expected_dtype}\",\"bits\":\"{}\"}},          \"class\": \"exact-byte\", \"ulp_bound\": 0, \"provenance\": \"oracle\",          \"tags\": [{tags}],          \"certificate\": {{\"hardness_margin_bits\": 0, \"stabilized_precision_bits\": 8}}}}",
        hex(expected)
    )
}

/// The comparison-mask cells (§6.2-0005), numbered from `tc`; returns the next free tcId.
fn compare_mask_cells(cells: &mut Vec<String>, mut tc: u32) -> u32 {
    let f32s: [(u32, u32); 12] = [
        (0x3F80_0000, 0x4000_0000), // 1 , 2
        (0x4000_0000, 0x3F80_0000), // 2 , 1
        (0x3F80_0000, 0x3F80_0000), // 1 , 1
        (0x0000_0000, 0x8000_0000), // +0, -0
        (0x8000_0000, 0x0000_0000), // -0, +0
        (0x7FC0_0000, 0x3F80_0000), // NaN, 1
        (0x3F80_0000, 0x7FC0_0000), // 1, NaN
        (0x7FC0_0000, 0x7FC0_0000), // NaN, NaN
        (0x7F80_0000, 0x7F80_0000), // +inf, +inf
        (0xFF80_0000, 0x7F80_0000), // -inf, +inf
        (0xBF80_0000, 0x3F80_0000), // -1 , 1
        (0x7F7F_FFFF, 0x7F80_0000), // max finite, +inf
    ];
    let ints: [(u32, u32); 5] = [
        (0xFFFF_FFFF, 0x0000_0001), // -1 / 4294967295 vs 1: signed and unsigned readings DISAGREE on lt/gt
        (0x0000_0001, 0xFFFF_FFFF),
        (0x8000_0000, 0x7FFF_FFFF), // signed min vs signed max
        (0x0000_0005, 0x0000_0005),
        (0x0000_0000, 0x0000_0001),
    ];
    let pred = |op: &str, o: Option<std::cmp::Ordering>| -> u8 {
        use std::cmp::Ordering::*;
        u8::from(match (op, o) {
            ("cmp_eq", o) => o == Some(Equal),
            ("cmp_ne", o) => o != Some(Equal),
            ("cmp_lt", o) => o == Some(Less),
            ("cmp_le", o) => matches!(o, Some(Less | Equal)),
            ("cmp_gt", o) => o == Some(Greater),
            _ => matches!(o, Some(Greater | Equal)),
        })
    };
    for op in ["cmp_eq", "cmp_ne", "cmp_lt", "cmp_le", "cmp_gt", "cmp_ge"] {
        for (a, b) in f32s {
            let o = f32::from_bits(a).partial_cmp(&f32::from_bits(b));
            cells.push(typed_cell(tc, op, "f32", &[&a.to_be_bytes(), &b.to_be_bytes()], "bool", &[pred(op, o)], "\"mask\",\"6.2-0005\""));
            tc += 1;
        }
        for (dtype, signed) in [("i32", true), ("u32", false)] {
            for (a, b) in ints {
                let o = if signed { (a as i32).partial_cmp(&(b as i32)) } else { a.partial_cmp(&b) };
                let tag = if signed { "\"mask\",\"6.2-0005\"" } else { "\"mask\",\"6.2-0005\",\"u32-unsigned\"" };
                cells.push(typed_cell(tc, op, dtype, &[&a.to_be_bytes(), &b.to_be_bytes()], "bool", &[pred(op, o)], tag));
                tc += 1;
            }
        }
    }
    tc
}

/// The `bool`-consumer normalization cells (§6.2-0006), numbered from `tc`; returns the next free tcId.
fn bool_normalization_cells(cells: &mut Vec<String>, mut tc: u32) -> u32 {
    let bytes: [u8; 7] = [0x00, 0x01, 0x02, 0x7F, 0x80, 0xFF, 0x40];
    let tag = "\"bool-normalize\",\"6.2-0006\"";
    for op in ["logical_and", "logical_or"] {
        for a in bytes {
            for b in [0x00u8, 0x02, 0xFF] {
                let (ta, tb) = (a != 0, b != 0);
                let r = u8::from(if op == "logical_and" { ta && tb } else { ta || tb });
                cells.push(typed_cell(tc, op, "bool", &[&[a], &[b]], "bool", &[r], tag));
                tc += 1;
            }
        }
    }
    for a in bytes {
        cells.push(typed_cell(tc, "logical_not", "bool", &[&[a]], "bool", &[u8::from(a == 0)], tag));
        tc += 1;
    }
    tc
}

fn compare_bool_doc() -> String {
    let mut cells = Vec::new();
    let tc = compare_mask_cells(&mut cells, 1);
    bool_normalization_cells(&mut cells, tc);
    bundle_doc("OPS", "KISS-OPS-6.2-0005", &cells)
}

// ---- f32/f64 min/max NaN bundle (#329) -----------------------------------------------------
//
// WHY: `max_prop`/`min_prop` (NaN-propagating) and `fmax_ieee`/`fmin_ieee` (NaN-suppressing) differ
// ONLY on a NaN operand, and until this bundle the corpus carried no f32/f64 NaN selection cell:
// the 48 signed-zero + 24 ordinary vectors all pass for an implementation with the two arms
// SWAPPED. The narrow dtypes already have NaN cells (ops-narrow-select-nan.json); f32 and f64 did not.
//
// EMITTED (every cell has a NaN operand, so §6.15's NaN arm decides and no value comparison runs):
//   PROPAGATING arm: the NaN operand is selected and MOVED — payload and sign exact (§6.16-0009):
//     NaN in a, NaN in b, a signaling NaN in each position, a NEGATIVE-signed quiet NaN, and
//     both-NaN (returns a, distinct payloads so the survivor is observable).
//   IEEE arm: the NaN is suppressed and the finite survivor's bits move: NaN in a, NaN in b,
//     a signaling NaN in each position; both-NaN returns b (distinct payloads).
// NOT EMITTED: inf and subnormal operands (no NaN -> covered by the value arm of
// ops-minmax-ordinary/-signed-zero); bf16/f16/f8 (ops-narrow-select-nan.json); a sNaN QUIETING
// claim — a MOVED NaN keeps its bits, §6.16-0010 quieting applies to COMPUTED NaNs only.
/// One float lane's NaN fixtures for the min/max bundle (bit patterns as integers, widest first).
struct MmLane {
    name: &'static str,
    bytes: usize,
    qa: u64,   // quiet NaN, payload A
    qb: u64,   // quiet NaN, payload B (distinct)
    sa: u64,   // signaling NaN
    sb: u64,   // signaling NaN, distinct
    negq: u64, // quiet NaN with the SIGN bit set
    one: u64,  // 1.0
}

const MM_LANES: [MmLane; 2] = [
    MmLane { name: "f32", bytes: 4, qa: 0x7FC1_2345, qb: 0x7FD5_5555, sa: 0x7F81_2345, sb: 0x7F80_0001, negq: 0xFFC0_1234, one: 0x3F80_0000 },
    MmLane {
        name: "f64",
        bytes: 8,
        qa: 0x7FF8_0000_0000_1234,
        qb: 0x7FFA_AAAA_AAAA_AAAA,
        sa: 0x7FF0_0000_0000_1234,
        sb: 0x7FF0_0000_0000_0001,
        negq: 0xFFF8_0000_0000_1234,
        one: 0x3FF0_0000_0000_0000,
    },
];

/// The cells of one lane: the propagating arm (6 per op) then the IEEE arm (5 per op).
fn minmax_nan_lane_cells(d: &MmLane, tc: &mut u32, cells: &mut Vec<String>) {
    let by = |v: u64| -> Vec<u8> { v.to_be_bytes()[8 - d.bytes..].to_vec() };
    let (qa, qb, sa, sb, nq, one) = (by(d.qa), by(d.qb), by(d.sa), by(d.sb), by(d.negq), by(d.one));
    for op in ["max_prop", "min_prop"] {
        let t = "\"moved-nan\",\"select-propagate\",\"6.16-0009\"";
        for (a, b) in [(&qa, &one), (&one, &qb), (&sa, &one), (&one, &sb), (&nq, &one), (&qa, &qb)] {
            cells.push(select_cell(*tc, op, d.name, a, b, t));
            *tc += 1;
        }
    }
    for op in ["fmax_ieee", "fmin_ieee"] {
        let t = "\"moved-survivor\",\"select-ieee-suppress\",\"6.15\"";
        for (a, b) in [(&qa, &one), (&one, &qb), (&sa, &one), (&one, &sb)] {
            cells.push(select_cell(*tc, op, d.name, a, b, t));
            *tc += 1;
        }
        cells.push(select_cell(*tc, op, d.name, &qa, &qb, "\"moved-nan\",\"select-ieee-both-nan\",\"6.15\""));
        *tc += 1;
    }
}

fn minmax_nan_doc() -> String {
    let mut cells = Vec::new();
    let mut tc = 1u32;
    for d in &MM_LANES {
        minmax_nan_lane_cells(d, &mut tc, &mut cells);
    }
    bundle_doc("OPS", "KISS-OPS-6.16-0009", &cells)
}

/// Assemble a bundle document from its cells. Header strings are shared across bundles so the
/// arith bundle regenerates byte-for-byte.
fn bundle_doc(substandard: &str, spec_clause: &str, cells: &[String]) -> String {
    bundle_doc_with_rule(substandard, spec_clause, None, cells)
}

/// As [`bundle_doc`], with an optional `harness_rule` header key (the key
/// `ops-minmax-ordinary.json` / `ops-minmax-signed-zero.json` already carry): how a runner must
/// compare THIS bundle's cells, where the shared `ulp_metric` line would mislead.
fn bundle_doc_with_rule(substandard: &str, spec_clause: &str, rule: Option<&str>, cells: &[String]) -> String {
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
    if let Some(r) = rule {
        doc.push_str(&format!("  \"harness_rule\": \"{r}\",\n"));
    }
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

// ---- f32/f64 sign-edit MOVE bundle (informative; KISS-OPS-6.16-0009 / KISS-CONFORM-6.8-0010(a)) ----
//
// WHY: `ops-narrow-move-nan.json` carries the move cells for the NARROW floats only, and 6.16-0009 is
// written about them. A moved NaN is exact-byte for every float dtype (6.8-0010(a)), and the same
// hazard exists wider: a path that promotes an `f32` through a wider register can quiet a moved
// signaling NaN. This bundle carries the same `neg`/`abs`/`copysign` family at `f32` and `f64`, so
// the move side of the move/compute boundary is visible next to the compute side already in
// `ops-transcendental-nan.json` (`exp`/`log`/`sin` of a signaling NaN -> a QUIET NaN, 6.16-0010).
//
// NO CONSUMER OBLIGATION. Offered, not required: nothing in the spec requires a consumer to run it.
// POPULATION / LIMITS (stated): single-op cells only. The multi-stage half of the boundary (6.16-0011:
// a reduction with a `post`, a row-reduction with an epilogue, traced to the observable OUTPUT) is NOT
// carried, because a cell has one scalar input per role and a fold needs a vector; that needs a
// composite cell shape the corpus does not have. Selection moves (`max_prop`/`min_prop`) at f32/f64
// are in `ops-minmax-nan.json`.
fn move_wide_doc() -> String {
    struct D {
        name: &'static str,
        nans: [(&'static str, u64); 3],
        neg_zero_sign: u64,
        one: u64,
        bytes: usize,
    }
    let dtypes = [
        D {
            name: "f32",
            nans: [("snan", 0x7F80_0001), ("qnan-rich", 0x7FD5_5555), ("neg-qnan-rich", 0xFFD5_5555)],
            neg_zero_sign: 0x8000_0000,
            one: u64::from(1.0f32.to_bits()),
            bytes: 4,
        },
        D {
            name: "f64",
            nans: [
                ("snan", 0x7FF0_0000_0000_0001),
                ("qnan-rich", 0x7FFA_AAAA_AAAA_AAAA),
                ("neg-qnan-rich", 0xFFFA_AAAA_AAAA_AAAA),
            ],
            neg_zero_sign: 0x8000_0000_0000_0000,
            one: 1.0f64.to_bits(),
            bytes: 8,
        },
    ];
    let be = |v: u64, n: usize| v.to_be_bytes()[8 - n..].to_vec();
    let mut cells = Vec::new();
    let mut tc = 1u32;
    for d in &dtypes {
        for (pname, val) in d.nans {
            let a = be(val, d.bytes);
            let tags = format!("\"moved-nan\",\"sign-edit\",\"{pname}\",\"6.16-0009\"");
            cells.push(move_cell(tc, "neg", d.name, &a, None, &tags));
            tc += 1;
            cells.push(move_cell(tc, "abs", d.name, &a, None, &tags));
            tc += 1;
        }
        // copysign: the signaling NaN's magnitude with a negative sign source (sign bit only).
        let nan0 = be(d.nans[0].1, d.bytes);
        let neg_sign = be(d.neg_zero_sign, d.bytes);
        cells.push(move_cell(
            tc,
            "copysign",
            d.name,
            &nan0,
            Some(&neg_sign),
            "\"moved-nan\",\"sign-edit\",\"copysign-neg\",\"6.16-0009\"",
        ));
        tc += 1;
        // Finite negative control: a finite passes a quieting path too, so the set does not red on everything.
        cells.push(move_cell(tc, "neg", d.name, &be(d.one, d.bytes), None, "\"finite-control\",\"sign-edit\",\"6.16-0009\""));
        tc += 1;
    }
    bundle_doc("OPS", "KISS-OPS-6.16-0009", &cells)
}

fn main() {
    let flags = kiss_conformance::cli::basics(
        "kiss_mint",
        "Mints the frozen oracle-vector corpus under conformance/corpus/.",
        &[("--write", "write the twelve bundles (nothing is written without it)")],
    );
    if flags.is_empty() {
        eprintln!("kiss_mint: nothing written; pass --write to mint the corpus (see --help)");
        return;
    }
    write_bundle("/corpus/ops-arith.json", &arith_doc());
    write_bundle("/corpus/ops-transcendental-nan.json", &transcendental_nan_doc());
    write_bundle("/corpus/ops-narrow-move-nan.json", &narrow_move_nan_doc());
    write_bundle("/corpus/ops-narrow-select-nan.json", &narrow_select_nan_doc());
    write_bundle("/corpus/ops-rounding.json", &rounding_doc());
    write_bundle("/corpus/ops-int-wrap.json", &int_wrap_doc());
    write_bundle("/corpus/ops-compare-bool.json", &compare_bool_doc());
    write_bundle("/corpus/ops-atan2-quadrants.json", &atan2_doc());
    write_bundle("/corpus/ops-sqrt-rounding.json", &sqrt_doc());
    write_bundle("/corpus/ops-nextafter.json", &nextafter_doc());
    write_bundle("/corpus/ops-minmax-nan.json", &minmax_nan_doc());
    write_bundle("/corpus/ops-move-wide-nan.json", &move_wide_doc());
}
