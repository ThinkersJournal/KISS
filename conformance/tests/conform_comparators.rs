//! KISS-Conform §6.8 comparator refinements.
//!
//! Backs KISS-CONFORM-6.8-0010 (computed-NaN result comparison, KISS #88 ruling
//! 2026-07-23): a *computed* NaN result compares by NaN-**ness** under the value
//! comparators (payload/sign not checked), while a *moved* NaN — a byte-preserving
//! result or a wire constant — and signed zero stay exact-byte.

use kiss_conformance::differential::agree;
use kiss_conformance::{compare_c32_transcendental, compare_f32, DeterminismClass};

// two quiet NaNs with different payloads, and a NaN with the opposite sign bit.
const NAN_A: u32 = 0x7FC0_1234;
const NAN_B: u32 = 0x7FC0_5678;
const NAN_NEG: u32 = 0xFFC0_0001;

#[test]
fn test_conform_nan_result_compares_by_nanness() {
    let nan_a = f32::from_bits(NAN_A);
    let nan_b = f32::from_bits(NAN_B);
    let nan_neg = f32::from_bits(NAN_NEG);

    // ---- computed NaN, VALUE comparator (oracle-differential agreement) --------
    // both-NaN match regardless of payload or sign — the ruled behavior. `agree`
    // is the harness's computed-NaN comparator ("not fragile to NaN payload
    // choices"): a transcendental/arithmetic NaN result is conformant iff it is a
    // NaN, whatever payload the hardware produced.
    assert!(agree(nan_a, nan_b), "differing NaN payloads must match under the value comparator");
    assert!(agree(nan_a, nan_neg), "NaN sign bit must not be compared for a computed result");
    assert!(agree(f32::NAN, nan_b), "the canonical NaN and a payload-carrying NaN match");

    // §6.16-0010 (KISS #388) — QUIETNESS is compared where the format admits a signaling NaN,
    // and f32 does. A SIGNALING result where the oracle produced a QUIET one DISAGREES; this is
    // the born-red on the FOREIGN-kernel surface (`agree` is the oracle-differential, the one
    // that certifies someone else's kernel — a blind accept here passed a non-conformant kernel).
    // Payload and sign stay uncompared.
    let snan = f32::from_bits(0x7F80_1234); // exp all-1, quiet bit CLEAR, payload nonzero => sNaN
    let snan2 = f32::from_bits(0x7F80_5678);
    assert!(snan.is_nan() && (snan.to_bits() >> 22) & 1 == 0, "0x7F801234 is a signaling NaN");
    assert!(!agree(nan_a, snan), "signaling where quiet is expected MUST mismatch (§6.16-0010)");
    assert!(!agree(snan, nan_a), "quiet where signaling is expected MUST mismatch");
    assert!(agree(snan, snan2), "two signaling NaNs (differing payload) match: quietness agrees, payload uncompared");

    // one-sided NaN is a MISMATCH — the disagreement about NaN-ness is exactly the
    // conformance-relevant fact (a propagate-vs-suppress bug must still be caught).
    assert!(!agree(nan_a, 5.0), "NaN where a finite value is expected must mismatch");
    assert!(!agree(5.0, nan_a), "finite where NaN is expected must mismatch");
    assert!(!agree(nan_a, f32::INFINITY), "NaN vs infinity must mismatch");

    // finite/infinite values are unaffected — still bit-compared by `agree`.
    assert!(agree(1.5, 1.5));
    assert!(!agree(1.5, 1.5000001));
    assert!(agree(f32::INFINITY, f32::INFINITY));

    // ---- computed NaN, ULP/tolerance comparator (§6.8-0002) --------------------
    // The clause scopes the refinement to the ULP/tolerance comparator too, not
    // only the differential relation: a computed NaN result whose op carries the
    // ULP/tolerance class (the transcendentals — `sin`/`exp`/…) is conformant iff
    // it is a NaN, whatever payload/sign. A tiny bound is used so a match cannot be
    // an accident of tolerance — differing NaN payloads are ~1e4 ULP apart under
    // the total-order key, so a payload-comparing implementation would reject them.
    let ulp = DeterminismClass::UlpTolerance;
    assert!(compare_f32(ulp, nan_a, nan_b, 2).is_ok(), "computed NaN payloads must match under ULP/tolerance");
    assert!(compare_f32(ulp, nan_a, nan_neg, 2).is_ok(), "computed NaN sign must not be compared under ULP/tolerance");
    assert!(compare_f32(ulp, nan_a, 5.0, 2).is_err(), "NaN where a finite value is expected must mismatch (ULP)");
    assert!(compare_f32(ulp, 5.0, nan_a, 2).is_err(), "finite where NaN is expected must mismatch (ULP)");
    assert!(compare_f32(ulp, nan_a, f32::INFINITY, 2).is_err(), "NaN vs infinity must mismatch (ULP)");
    // §6.16-0010 quietness reaches the ULP/tolerance NaN arm too (#434 site 1): a SIGNALING
    // result where a QUIET one is expected mismatches (payload/sign still uncompared). Born-red —
    // under the old NaN-ness-only arm this passed. This arm is not reached by a live conformance
    // run (corpus NaN cells route to compare_nan_output first), but compare_f32 is a public
    // comparator with this direct contract, kept consistent with agree/c32/the reductions.
    assert!(
        compare_f32(ulp, snan, nan_a, 2).is_err(),
        "signaling where quiet is expected must mismatch under ULP/tolerance (§6.16-0010)"
    );
    // ordinary finite ULP behaviour is unchanged.
    assert!(compare_f32(ulp, 1.0, 1.0, 0).is_ok());
    assert!(compare_f32(ulp, 1.0, 2.0, 0).is_err());

    // ---- computed NaN, split-comparator magnitude arm (§6.8-0005 / §6.18-0017) --
    // The complex-transcendental split comparator routes an ordinary component
    // (not a zero, not a ±π endpoint) through ULP-tolerance; a computed-NaN
    // component there must likewise compare by NaN-ness, per the clause's explicit
    // scope. (`im` is an ordinary finite lane held equal so only the `re` NaN lane
    // is under test.)
    assert!(
        compare_c32_transcendental([nan_a, 1.0], [nan_b, 1.0], 2).is_ok(),
        "computed NaN component must match by NaN-ness under the split comparator"
    );
    assert!(
        compare_c32_transcendental([nan_a, 1.0], [5.0, 1.0], 2).is_err(),
        "one-sided NaN component must mismatch under the split comparator"
    );
    // §6.16-0010 quietness reaches the split comparator's NaN arm too (#434): a SIGNALING
    // component where a QUIET one is expected mismatches (payload/sign still uncompared). Born-red
    // — under the old NaN-ness-only arm this passed. (`snan` is the signaling NaN defined above;
    // the two-quiet-NaNs match is the control at the top of this block.)
    assert!(
        compare_c32_transcendental([snan, 1.0], [nan_a, 1.0], 2).is_err(),
        "split comparator: a signaling NaN component where a quiet one is expected must mismatch (§6.16-0010)"
    );

    // ---- the exemption MUST NOT leak: exact-byte still distinguishes payloads ---
    // A byte-preserving result (gather/scatter/flip/select/bitcast) carries a MOVED
    // NaN whose payload IS the contract; the exact-byte comparator must still fail
    // on a differing payload, so a corrupted raw-bit move is caught (§6.8-0010(a)).
    assert!(
        compare_f32(DeterminismClass::ExactByte, nan_a, nan_b, 0).is_err(),
        "exact-byte MUST still distinguish NaN payloads (byte-preserving domain)"
    );
    // ...and two byte-identical NaNs match under exact-byte (a clean move).
    assert!(compare_f32(DeterminismClass::ExactByte, nan_a, nan_a, 0).is_ok());

    // ---- signed zero is NOT exempted: only NaN is (§6.8-0010(c)) ---------------
    assert!(
        compare_f32(DeterminismClass::ExactByte, -0.0, 0.0, 0).is_err(),
        "signed zero MUST stay bit-distinguished; only NaN is exempt from bit comparison"
    );
}
