//! Plan B Slice-0 T5 — err_ulps SOUNDNESS: verify each atom's reported `err_ulps`
//! actually BOUNDS its true error. This is a SOUNDNESS test, not a tightness test:
//! an err_ulps that UNDER-states the true error silently corrupts the Ziv decision
//! (wrong correctly-rounded bits), while an over-estimate only costs escalation
//! time — so bounding is the property that matters, and it is what is asserted.
//!
//! The bounds here are deliberate over-estimates (×4 on the reduction error,
//! round-up rescaling, +1 truncation), so `|d| ≤ err_ulps` can only catch an
//! err_ulps wrong by MORE than that built-in slack — gross under-estimation. The
//! achieved `d/err_ulps` ratio is REPORTED per atom as the tightness FINDING, not
//! the test's claim. ⚠️ Interpret it in BOTH directions: ratio > 1 = UNSOUND (the
//! target failure); ratio ~ 0.01 = expected, given the construction; ratio ~ 0.9 =
//! SURPRISING and worth stopping for — that would be far tighter than three
//! deliberate over-estimates should produce, i.e. a derivation error or a
//! wide_reference not actually independent of the atom, NOT good news.
//!
//! References are the true value RNE-rounded to 256 bits (mpmath, 700-bit working
//! precision), MSW-first, bit 255 set — the same shape as `round_ziv.rs`'s.
//!
//! Two DISTINCT controls, neither substituting for the other: the in-test
//! `dist_gt_err` check proves the COMPARATOR has teeth (a bound below the measured
//! distance is flagged); separately (manual, recorded in the PR) shrinking the
//! atom's own `err_ulps` proves the ASSERTION discriminates over this real corpus —
//! the first exercises the primitive, the second the assertion's power over its
//! inputs.
//!
//! ⚠️ ESCALATION SINK (recorded, not just known): the Ziv 512/1024 path is a
//! PARTITION CLASS no real f32/f64 input reaches — the correctly-rounded decision
//! always resolves at 256 bits (the Table-Maker's-Dilemma worst case for these
//! functions at f64 is ~113 bits, far under 256). It is reachable ONLY by
//! `round_ziv.rs`'s hand-authored straddles (its positive control proves the branch
//! is live, not dead), and is kept as defence against exactly the wrong-err_ulps
//! failure THIS test guards. So "we never escalated" must NOT be read as "the
//! escalation path is tested"; its uncoveredness by real inputs is documented here.

use kiss_conformance::hp::{Atom, BigFloat, Exp, FpClass, Log, Sin};

type Ref = (f64, bool, i32, [u64; 4]); // (x, sign, ebin, 256-bit RNE mantissa MSW-first)

/// Any legitimate err_ulps for these atoms is small (exp at |k|~1010 needs ~8232); a
/// value at or above this ceiling is a SATURATED / overflowed bound — a degenerate
/// value that makes the soundness check `|d| ≤ err_ulps` VACUOUSLY TRUE, so it must
/// fail LOUD, not pass free. This guard — not the soundness assertion — is what
/// actually caught the log-of-power-of-2 bug (err_ulps had saturated to u128::MAX,
/// which the assertion is vacuously satisfied by; the first surfacing was in fact an
/// accidental `err_ulps + 1` overflow, then this ceiling made it a deliberate red).
const SANITY_CEILING: u128 = 1 << 16;

// ⚠️ The corpus spans small→large arguments on purpose: the TIGHTEST bound (highest
// d/err_ulps ratio) is NOT at the widest err_ulps. Measured — exp's max ratio is at
// exp(-700) (err_ulps 4164), not the widest exp(700) (err_ulps 8232). Stated as a
// caution beforehand, confirmed after. Do NOT "simplify" this to the large-argument
// cases: that drops exactly where the bound bites.
const EXP_CASES: &[Ref] = &[
    (1.0, false, 1, [0xADF85458A2BB4A9A, 0xAFDC5620273D3CF1, 0xD8B9C583CE2D3695, 0xA9E13641146433FC]),
    (-1.0, false, -2, [0xBC5AB1B16779BE35, 0x75BD8F0520A9F21B, 0xB5300B556AD8EE66, 0x604973A14A0FB5DB]),
    (50.0, false, 72, [0x8C881F20405A2B32, 0x6BBA067C62EC5A7A, 0x0521D074FD2D286E, 0x17448DDACAE67EA2]),
    (700.0, false, 1009, [0xECA2EFA7C7647118, 0x3392684A46E55D14, 0x4B56FBBB8C02CD18, 0xEB4E9E9D6F0CA814]),
    (-700.0, false, -1010, [0x8A79587DC983F855, 0xE586959F79E6C765, 0x31AD28D57B2CC5E1, 0xD24F00DAF9B93F1F]),
];

const LOG_CASES: &[Ref] = &[
    (2.0, false, -1, [0xB17217F7D1CF79AB, 0xC9E3B39803F2F6AF, 0x40F343267298B62D, 0x8A0D175B8BAAFA2C]),
    (0.5, true, -1, [0xB17217F7D1CF79AB, 0xC9E3B39803F2F6AF, 0x40F343267298B62D, 0x8A0D175B8BAAFA2C]),
    (10.0, false, 1, [0x935D8DDDAAA8AC16, 0xEA56D62B82D30A28, 0xE28FECF9DA5DF90E, 0x83C61E8201F02D73]),
    (1e300, false, 9, [0xACB1A23FC3FDA9AB, 0xCCC0710FCD43F892, 0x47673618C3F96005, 0x0652CD7A917FF3CB]),
    (1e-300, true, 9, [0xACB1A23FC3FDA9AA, 0x670D35324E5C7C79, 0xE22C1F2C25E09392, 0x6988CC9ADB25029A]),
];

const SIN_CASES: &[Ref] = &[
    (1.0, false, -1, [0xD76AA47848677020, 0xC6E9E909C50F3C32, 0x89E511132F518B4D, 0xEFB6CA5FD6C649BE]),
    (100.0, true, -1, [0x81A12DBC626DC038, 0x47B0AAE841F590E3, 0x57892550CEA15742, 0x04D9F430294DF799]),
    (1e15, false, -1, [0xDBB7C409B675CE4A, 0x534230A93D64DF69, 0x729535AF86A3034E, 0x20C2296ED4DBDB28]),
];

/// The low 128 bits of a `[u64;4]` distance; the high two limbs must be zero (a
/// distance ≥ 2^128 ULPs is a gross bound violation, not a tightness question).
fn low_u128(d: &[u64; 4]) -> u128 {
    assert!(d[0] == 0 && d[1] == 0, "distance exceeds 2^128 ULPs — bound grossly violated");
    ((d[2] as u128) << 64) | (d[3] as u128)
}

/// Assert `err_ulps` bounds the true error for one case; return the tightness in
/// permille when `err_ulps ≥ 8` (else the ≤1-ULP reference rounding dominates).
fn check_one(name: &str, ev_val: &BigFloat<4>, err_ulps: u128, r: &Ref) -> Option<u128> {
    let (x, sign, ebin, refm) = *r;
    assert_eq!(ev_val.sign, sign, "{name}({x}): sign mismatch");
    assert_eq!(ev_val.ebin(), ebin, "{name}({x}): ebin mismatch (value straddles a power of 2 vs reference?)");
    let reference = BigFloat::<4>::from_limbs_ebin(sign, ebin, refm, FpClass::Normal);
    let d = ev_val.abs_diff_limbs(&reference);
    let du = low_u128(&d);
    // (1) SOUNDNESS — the dangerous direction: true error ≤ err_ulps (+1 = ref RNE).
    // saturating_add so a (buggy) err_ulps == u128::MAX cannot overflow the check
    // itself; a saturated bound is caught as vacuous by (2), not hidden by a panic.
    assert!(
        du <= err_ulps.saturating_add(1),
        "{name}({x}): UNSOUND — d={du} > err_ulps={err_ulps}+1 (err_ulps UNDER-states the true error)"
    );
    // (2) MEANINGFUL — not vacuously large (a saturated bound makes (1) vacuous).
    assert!(err_ulps < SANITY_CEILING, "{name}({x}): err_ulps={err_ulps} is vacuously large (bound saturated/overflowed?)");
    // (3) COMPARATOR TEETH — a bound one ULP below the measured distance MUST flag.
    if du > 0 {
        assert!(
            BigFloat::<4>::dist_gt_err(&d, du - 1),
            "{name}({x}): comparator control failed — dist_gt_err did not flag a bound below d"
        );
    }
    // Print the RAW distance and err_ulps, not just a rounded ratio: a permille that
    // rounds to 0 cannot distinguish a TIGHT bound (~1e-3 permille) from an ABSURD one
    // (a saturated err ~1e-38 permille) — "~0" is exactly what hid the log(2) bug in
    // the first summary. err_ulps printed whole is unmissable when it saturates.
    eprintln!("{name}({x}): d={du} err_ulps={err_ulps}");
    (err_ulps >= 8).then(|| du.saturating_sub(1).saturating_mul(1000) / err_ulps)
}

fn run(name: &str, cases: &[Ref], eval: impl Fn(f64) -> (BigFloat<4>, u128)) {
    let mut max = 0u128;
    for r in cases {
        let (val, err) = eval(r.0);
        if let Some(p) = check_one(name, &val, err, r) {
            max = max.max(p);
        }
    }
    // Pre-registered expectation WITH a resolution floor: the bounds are deliberate
    // over-estimates, so a LOW ratio (single-digit-to-low-hundreds permille) is
    // expected and confirms the construction; ~0 permille (d≈0 vs a real err) is fine
    // too; but a SATURATED err is a distinct outcome, caught by SANITY_CEILING, not
    // absorbed into "low". "I predicted low, it's low, fine" is the trap — say how low.
    eprintln!("{name}: MAX tightness = {max} permille (d/err_ulps × 1000; a FINDING, not a claim)");
}

#[test]
fn exp_err_ulps_is_sound() {
    run("exp", EXP_CASES, |x| {
        let ev = Exp { x }.eval::<4>();
        (ev.val, ev.err_ulps)
    });
}

#[test]
fn log_err_ulps_is_sound() {
    run("log", LOG_CASES, |x| {
        let ev = Log { x }.eval::<4>();
        (ev.val, ev.err_ulps)
    });
}

#[test]
fn sin_err_ulps_is_sound() {
    run("sin", SIN_CASES, |x| {
        let ev = Sin { x }.eval::<4>();
        (ev.val, ev.err_ulps)
    });
}

/// ⚠️ The vacuous-bound bug found in Log (err_ulps → u128::MAX for m == 1 exactly)
/// is a CLASS, not a one-off: any atom whose reduction can yield a zero-valued
/// intermediate with a degenerate exponent can drive `rescale_ulps` past its
/// 128-bit shift and saturate. This sweeps the obvious zero-intermediate inputs
/// across ALL atoms — exp(0) (k=0, r=0), sin(±0) (octant 0, r=0), log(1) (t=0) —
/// which the small→large stress corpora above deliberately do NOT contain, and
/// asserts none produces a saturated (vacuous) bound. Values are checked too, so a
/// bound that is fine but a result that is wrong cannot pass.
#[test]
fn zero_intermediate_inputs_do_not_saturate_err() {
    let exp0 = Exp { x: 0.0 }.eval::<4>();
    let sin0 = Sin { x: 0.0 }.eval::<4>();
    let sin_neg0 = Sin { x: -0.0 }.eval::<4>();
    let log1 = Log { x: 1.0 }.eval::<4>();
    for (name, err) in [
        ("exp(0)", exp0.err_ulps),
        ("sin(0)", sin0.err_ulps),
        ("sin(-0)", sin_neg0.err_ulps),
        ("log(1)", log1.err_ulps),
    ] {
        eprintln!("{name}: err_ulps={err}");
        assert!(
            err < SANITY_CEILING,
            "{name}: err_ulps={err} SATURATED — vacuous bound, same class as the log(2) bug"
        );
    }
    // Values (the atom must be right, not merely non-saturated): exp(0)=1, sin(±0)=±0.
    assert_eq!(exp0.val.ebin(), 0, "exp(0) = 1 (ebin 0)");
    assert!(!exp0.val.sign, "exp(0) = +1");
    assert!(sin0.val.is_zero() && !sin0.val.sign, "sin(0) = +0");
    assert!(sin_neg0.val.is_zero() && sin_neg0.val.sign, "sin(-0) = -0 (odd)");
    assert!(log1.val.is_zero(), "log(1) = 0");
}
