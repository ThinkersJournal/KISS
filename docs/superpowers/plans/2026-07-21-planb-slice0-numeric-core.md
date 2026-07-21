# Plan B — Slice 0: wide-precision numeric core (execution plan)

**Status:** approved to build (maintainer: "build now, clog deferred", 2026-07-21). Branch `planb/slice0-numeric-core` off `origin/main`.
**Design:** `docs/superpowers/specs/2026-07-19-kiss-oracle-vector-corpus-design.md` (§5, §10 Slice 0).
**Vetted kernel specs + adversarial review findings:** `docs/superpowers/specs/2026-07-21-planb-slice0-workflow-output.md` (6 kernels; every critical/important finding is folded into the owning task below).
**T1 hp-core spec (recovered):** `docs/superpowers/specs/2026-07-21-planb-slice0-hp-core-spec.md`.

Goal: complete Slice 0 of the KISS-Conform oracle-vector corpus — the dependency-free 256-bit `hp.rs` core that mints **wide-precision correctly-rounded** transcendental cells (exp/log/sin at f32+f64), wiring the minter/reader/clause-tests/validator to cover transcendentals. **Format v1 stays frozen** (additive only). Satisfies KISS-CONFORM-6.5-0007/0008/0009 for the transcendental modality.

## Scope decisions
- **clog is DEFERRED to Slice 1** (maintainer call): it is the heaviest kernel (drags in bf_sqrt+bf_atan+atan2 and a c32 path) and has a near-|z|=1 correct-rounding difficulty; the primary consumer (Fuel) needs real atoms, not complex. clog rejoins Slice 1 once atan2 is built. So Slice 0 = exp/log/sin only on the atom side.
- **Two-tier corpus (flagged to maintainer, separate track):** an exact-byte DENSIFICATION pass (semantics.rs mint path, no hp.rs) is the cheap near-term unblock for Fuel's exact-floor gating (format v1 already consumable). Independent of this plan; sequencing is the maintainer's call.

## Critical fixes the adversarial review caught (folded in)
- `avail<=0 => signed zero` misrounds (2^−1075,2^−1074), e.g. `exp(-745)` — **T2** treats avail==0 as a straddle boundary, not round-to-zero.
- `exp` k-overflow: routing all finite x through the series overflows i64 `k` for large x — **T5** pre-clamps large |x| to ±Inf/±0 in the atom (semantics front door) before reduction.
- `log` needs a GENERAL big-float `div` (not just div_small) — **T1** provides it (restoring long division).
- Constant tables under-sized/under-verified: 2/pi bumped 1280->**2304 bits**, ln2 -> 1152 bits, with **full-width** in-crate verification — **T3**.
- `evaluation_precision` drift: corpus.rs aliases certificate.stabilized_precision_bits into a free-standing certificate_precision_bits — **T7** introduces the authoritative enum {compute-dtype|wider-than-compute} and nests the bits under it, early.

## Tasks (TDD: failing test -> minimal impl -> verify; full detail in the workflow-output + hp-core spec)

| Task | Title | Depends on | Files |
|---|---|---|---|
| **T1** hp-core | `BigFloat<N>` core (N in {4,8,16}) + limb primitives + **general div** | — | conformance/src/hp.rs |
| **T2** round-ziv | round-to-f64/f32 RNE + exact-integer midpoint distance + Ziv 256->512->1024 + certificate | T1 | hp.rs |
| **T3** constants | full-width 2/pi (2304b), ln2/1÷ln2/pi2 (1152b), sqrt2 + FULL-WIDTH in-crate verification | T1 | hp.rs |
| **T4** reduction | Payne–Hanek trig + exp/log reduction, mandatory table-length guards | T3, T1 | hp.rs |
| **T5** exp-log | exp/log atoms: reduction + series + folded truncation bound + err_ulps, f32 & f64; large-x pre-clamp | T4, T2, T1 | hp.rs, semantics.rs |
| **T6** sin | sin atom: octant reconstruction + Maclaurin on \|r\|<=pi/4 + special-value front door, f32 & f64 | T4, T2 | hp.rs, semantics.rs |
| **T7** eval-precision-enum | authoritative `evaluation_precision {compute-dtype\|wider-than-compute}`; nest certificate_precision_bits; resolve in-tree drift | T2 | corpus.rs, corpus_coverage.rs |
| **T9** minter | emit transcendental oracle cells (exp/log/sin f32+f64) with certificate + edge tags | T5, T6, T7 | bin/kiss_mint.rs, corpus/*.json |
| **T10** reader-differential | reader/differential: unary op+dtype dispatch, ulp_distance_f64/compare_f64 + teeth tests | T9 | corpus.rs, tests/corpus_differential.rs |
| **T11** clause-6.5-0009 | tighten: transcendental cells' stabilized_precision_bits STRICTLY > compute dtype width (exact keeps >=) | T9, T7 | tests/corpus_coverage.rs |
| **T12** clause-6.5-0008 | tighten: each transcendental atom's load-bearing EDGE TAGS must be present | T9 | tests/corpus_coverage.rs, corpus/op_manifest.json |
| **T13** validator | validate_corpus.py: real 3-source (mpmath + gmpy2/flint + L-M/C99 anchors), bound-based Ziv, provenance=="oracle" guard, exp finite/inf NOT a threshold | T9 | tools/validate_corpus.py |

(**T8 clog deferred to Slice 1.**)

## Residual risks / open items
- **NaN-output convention** (spec ruling, before T9): pinning a canonical qNaN as exact-byte over-constrains IUTs that propagate NaN payload. Maintainer lean: allow payload propagation. Decide before the minter emits NaN-input cells.
- **Validation toolchain:** a real freeze-certifying `validate_corpus.py` run (T13) needs `gmpy2` or `python-flint`; this machine has only mpmath (smoke mode). Before-freeze, not before-build.
- Large-|x| trig near 2^1023: the 2304-bit table correctly rounds, but the certificate's uniform ~244-bit-accuracy premise is overstated there (~197-bit reduced-arg accuracy) — document the actual margin.
- The 512/1024 escalation ladder is exercised in Slice 0 only by constructed near-midpoint inputs (by design).

## Sequencing
Strict spine: T1 -> T2 -> T3 -> T4 -> {T5, T6} -> T9 -> {T10, T11, T12, T13}. T7 pulled early (depends only on T2's certificate shape; must precede T9/T11). T5/T6 parallel once T4 lands; T10-T13 parallel once T9 lands. Each hp.rs atom task pins hand-authored bit-pattern vectors as correctness anchors; each clause test (T11/T12) must be shown to FAIL against a crafted violating cell before the tightening is accepted.
