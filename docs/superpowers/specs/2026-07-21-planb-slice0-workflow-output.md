# Plan B Slice-0 — workflow design output (durable reference)

Source: workflow wmz37lmwe (13/14 agents; hp-core design agent hit the StructuredOutput retry cap — recovered separately). ~1M subagent tokens.

## Synthesized TDD plan (ready_to_build = True)

**Sequencing:** Strict dependency spine per the prompt: T1 hp-core → T2 round-ziv → T3 constants → T4 reduction → {T5 exp/log, T6 sin} → T8 clog; then the wiring layer T9 minter → {T10 reader/differential, T11 §6.5-0009, T12 §6.5-0008, T13 validator}. T7 (evaluation_precision enum) is pulled EARLY — it depends only on T2's certificate shape and MUST land before T9 mints certificates and before T11 reads them, because it resolves an existing in-tree drift (corpus.rs aliases certificate.stabilized_precision_bits into a free-standing certificate_precision_bits at line 32/93, while corpus_coverage.rs keys off certificate_precision_bits) — the transcendental rollout is the designated place to introduce the authoritative enum rather than propagate the drift. Parallelism: T5 and T6 are independent once T4 lands; T10/T11/T12/T13 are independent of each other once T9 lands and can fan out. clog (T8) sits at the end of the atom chain because it is the heaviest (adds bf_sqrt+bf_atan+atan2+the pi table's dependents and four new integration surfaces) and reuses T5's log. Every task is a failing-test-first TDD unit: the hp.rs atom vectors are hand-pinned bit patterns (correctness anchors), and the clause tests (T11/T12) must be shown to FAIL against a crafted violating cell before the tightening is accepted. Critical fixes are folded into the task that owns the code, never deferred: avail==0 misround→T2; full-width constant verification→T3; exp k-overflow pre-clamp→T5; general div for log→T1; clog near-|z|=1 escalation-to-512→T8; evaluation_precision enum→T7; validator bound-based Ziv + exp overflow-not-special→T13.

### T1-hp-core — BigFloat<N> core: [u64;N] significand + i32 exp + sign + class, N∈{4,8,16}, limb-wise primitives + GENERAL div
- **depends_on:** (none)
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs, C:/Projects/kiss-planb/conformance/src/lib.rs
- **failing_test:** hp::tests: (a) from_f64/from_f32 round-trip is bit-exact for 1.0, 2.0, smallest subnormal, and a random mantissa; (b) mul then general div by the same value returns the input to within 1 ULP-at-W; (c) big-int cmp of two [u64;N] limb arrays orders correctly across a limb boundary (shift>64). All fail because hp.rs does not exist.
- **implementation:** Create hp.rs mirroring fp.rs's from-scratch, dependency-free, spec-pinned style. Define BigFloat<const N:usize>{sign,exp:i32,mant:[u64;N],class}. Wire N=4/8/16 (256/512/1024) via const generics. Implement limb-wise shl/shr/cmp/add/sub/mul (schoolbook u128 MAC), from_f64/from_f32/from_i64 (EXACT), mul_small_int, ldexp (exact exp add), neg/abs, round_to_i64, is_normal/class helpers. FOLD exp-log IMPORTANT finding: add a GENERAL div(&self,&BigFloat) via Newton–Raphson reciprocal at width W (log's s=(m-1)/(m+1) needs it; div_small alone is insufficient) and document its ≤1-ULP-at-W rounding so atoms can fold it into err_ulps. FOLD round-ziv MINOR: expose a big-int compare so the later D>err_ulps test is limb-wise, never truncated into u128.
- **verification:** cargo test -p conformance hp::tests passes; no external crates in Cargo.toml (stdlib-only invariant holds).

### T2-round-ziv — Round-to-f64/f32 RNE reducer + exact-integer midpoint distance + Ziv 256→512→1024 escalation + certificate
- **depends_on:** T1-hp-core
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs
- **failing_test:** Feed round_target hand-constructed BigFloats (no atom needed): (a) magnitude 1.1735·2^-1075 (the true value of exp(-745)) with small err_ulps MUST round to 0x0000000000000001, NOT +0 — this is the avail==0 case; (b) a value above the RNE overflow midpoint rounds to 0x7FF0000000000000; (c) a value below half the smallest subnormal rounds to +0; (d) the SAME 256-bit value rounds directly to f32 (single rounding), asserted != round-to-f64-then-f32 on a midpoint-adjacent construction; (e) a near-midpoint construction whose interval straddles at 256 escalates and reports stabilized_precision_bits∈{512,1024}. Tests fail: round_target unimplemented.
- **implementation:** Implement §2–§8 of the round-ziv spec: locate round position (avail = t for E>=emin, else t-(emin-E)); split q/R/F/S limb-wise; single RNE (round_up = R&&(S||odd)); mantissa-carry renormalize + re-check overflow→±Inf; exact Fmid=1<<(shift-1) and D=|F-Fmid| as a limb-wise big-int; DECIDED⇔D>err_ulps (strict, big-int compare); STRADDLE⇒re-evaluate atom at next N. Emit Certificate{hardness_margin_bits=leading-zero run of D, stabilized_precision_bits=64*N}. FOLD round-ziv CRITICAL: short-circuit to signed zero ONLY for avail<0; for avail==0 run q/R/S with shift==P, computing masks all-ones limb-wise so they do not overflow [u64;N]. FOLD round-ziv MINOR: D>err_ulps as a big-int compare, never a u128 truncation. Special-class (Inf/NaN/Zero) bypasses §2–§5 but still emits stabilized_precision_bits=64*N so the strict-> §6.5-0009 invariant holds. Panic on 1024-unresolved (red-flag).
- **verification:** cargo test hp::round passes including the avail==0, overflow, single-round-f32, and escalation cases.

### T3-constants — Full-width constants (2/pi 2304b, ln2/1/ln2/pi2 1152b, sqrt2) + FULL-WIDTH in-crate verification
- **depends_on:** T1-hp-core
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs
- **failing_test:** hp::consts tests: (a) Machin self-check recomputes pi/2 and 2/pi and asserts EVERY stored table bit minus a small low guard (not just top 960); (b) atanh self-check for ln2/1-over-ln2 to full 1152-bit width; (c) reciprocal identities PI_OVER_2·(2/pi)=1 and LN2·INV_LN2=1 to full width; (d) ALL ~36 TWO_OVER_PI words and all 18-word ln2/pi2 words match pinned fdlibm ipio2 / libm high-word anchors. Fail: constants absent.
- **implementation:** Store TWO_OVER_PI as [u64;36]=2304 bits (bit table), LN2/INV_LN2/PI_OVER_2 as [u64;18]=1152-bit significands, SQRT2, all at MAX width and truncated to working W (escalation re-reads the wider slice). Size per the reduction spec's formulas (i1_max=2176≤2304; ln2 needs ≥1099) — this OVERRIDES the design doc's under-sized 1280/256 figures. FOLD reduction-review CRITICAL: decouple in-crate verification width from MAX_L — recompute constants in a scratch big-int WIDER than the tables (≥2400 bits for 2/pi, ≥1200 for ln2) and assert every stored bit minus a small guard; pin ALL ~34 ipio2 words as static anchors, not just words 0-2, so a typo in a deep tail word (the load-bearing bits for large-|x| trig / exp,log at 1024) cannot survive. Dev-time gen_constants.py (3-engine) produces the literals but is not shipped.
- **verification:** cargo test hp::consts passes; a deliberately corrupted tail word (e.g. word 20) makes a test fail (guard the guard).

### T4-reduction — Payne–Hanek trig reduction + exp/log reduction, with mandatory table-length guards
- **depends_on:** T3-constants, T1-hp-core
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs
- **failing_test:** (a) reduce_trig(f64_nearest(pi)) yields octant==2 and |r+1.2246467991473532e-16|<2^-100; (b) reduce_trig for |x|~2^1023 succeeds (window i1~2176 fits) and reproduces a validator-pinned reduced argument with a long-cancellation worst-case double bit-exact; (c) the assert!(i1<=TWO_OVER_PI_BITS) FIRES (panics) for a synthetic over-range argument; (d) reduce_exp/reduce_log carry an equivalent width guard that panics if ln2 is ever too short. Fail: reduction absent.
- **implementation:** Implement reduce_trig (window [i0,i1], gather 2/pi slice, bigmul by M, align, split n_low3+frac, round-to-nearest integer via bit F-1, octant, build r=f'·pi/2, err_ubits=-(F-1)) with the HARD guard assert on i1. Implement reduce_exp (k=round(x·INV_LN2), r=x-k·ln2) and reduce_log (frexp octave + sqrt2 fold, m∈[√2/2,√2)) with equivalent width asserts (FOLD reduction missing-edge-case: exp/log need the same panic-not-certify guard as trig). FOLD sin-review IMPORTANT: consume THIS 2304-bit table, never sin's stale ~1280 figure; state width parametrically (E_max+L_max+P+guard). Constants truncate per working W so escalation widens the constant automatically.
- **verification:** cargo test hp::reduce passes; the over-range and short-ln2 guards are proven to panic (should_panic tests).

### T5-exp-log — exp/log atoms: reduction + Taylor/atanh series + folded truncation bound + err_ulps, f32 & f64
- **depends_on:** T4-reduction, T2-round-ziv, T1-hp-core
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs, C:/Projects/kiss-planb/conformance/src/semantics.rs
- **failing_test:** round_ziv(exp/log) vectors: exp(1.0)f64=0x4005BF0A8B145769, exp(1.0)f32=0x402DF854 (direct single-round), log(2.0)f64=0x3FE62E42FEFA39EF, log(1.0)=+0.0 via semantics special routing (must NOT enter core), exp(710)=+Inf, exp(-745)=0x…01, exp(-746)=+0; AND exp(1e19 / 0x43E158E460913D00)=+Inf with NO integer-overflow panic; log(1+2^-30) log1p-regime validator-pinned bits. Fail: atoms absent.
- **implementation:** Implement exp_hp (k·ln2 reduction, iterative Taylor exp(r), 2^k via exact ldexp) and log_hp (octave reduce, s=(m-1)/(m+1) via the GENERAL div from T1, atanh series, +e·ln2). Fold each series' alternating/geometric truncation tail into err_ulps (Ziv cannot see truncation otherwise). FOLD exp-log CRITICAL: pre-clamp large finite x in the semantics.rs dispatch BEFORE reduction — x>=~709.7827→+Inf, x<=~-745.1332→+0/subnormal band — so k never overflows i64/i32; keep the genuine overflow-midpoint hard case (x≈709.78) inside the core. FOLD IMPORTANT: log's division error is folded into err_bits. FOLD MINOR: state the exp |r| invariant as ≤1.5·ln2≈1.04 (off-by-one k) and size the series for it. Route exp(±0)/exp(±inf)/log(±0)/log(x<0)/log(1) through semantics.rs specials, never the series.
- **verification:** cargo test hp::exp hp::log passes all listed vectors; exp(1e19) and exp(1e300) return +Inf without panic.

### T6-sin — sin atom: octant reconstruction + Maclaurin on |r|<=pi/4 + special-value front door, f32 & f64
- **depends_on:** T4-reduction, T2-round-ziv
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs, C:/Projects/kiss-planb/conformance/src/semantics.rs
- **failing_test:** sin vectors: sin(1.0)f64=0x3FEAED548F090CEE, sin(0.5)f64=0x3FDEAEE8744B05F0, sin(1.0)f32=0x3F576AA4, sin(1e22)=0xBFEB453AB76BF397 (large-|x| Payne–Hanek), sin(f64_nearest(pi))=0x3CA1A62633145C07 (near-k-pi), sin(2^1023) validator-pinned large-|x| stressor, sin(±0)=±0 exact-byte, sin(±inf)/sin(NaN)=qNaN. Fail: sin absent.
- **implementation:** Implement sin_core: consume reduce_trig's (octant,r), evaluate sin(r)/cos(r) by exact-integer-recurrence Maclaurin (no factorial table), reconstruct via octant table sin=[s,c,-s,-c], carry first-omitted-term truncation bound into err_ulps, round once RNE. Front-door specials (±0 sign-preserving, ±inf→qNaN, NaN→qNaN) handled in semantics.rs as exact-byte cells, never ULP. FOLD reduction-review IMPORTANT: correct the sin(1.0) reduction note to octant==1, r≈-0.5708, sin=cos(r) (the octant-0 note was wrong). FOLD round-ziv IMPORTANT + sin missing-edge: include the |x|~2^1023 large-trig stressor (sin(1e22)=sin(2^73) alone under-exercises the table). FOLD sin MINOR: pin NaN handling as a spec decision (compare NaN by predicate vs exact-byte) — see residual risk; default to canonical qNaN exact-byte pending ruling.
- **verification:** cargo test hp::sin passes all vectors including sin(1e22) and the 2^1023 stressor.

### T7-eval-precision-enum — Introduce authoritative evaluation_precision enum {compute-dtype|wider-than-compute}; nest certificate_precision_bits; resolve in-tree drift
- **depends_on:** T2-round-ziv
- **files:** C:/Projects/kiss-planb/conformance/src/corpus.rs, C:/Projects/kiss-planb/conformance/src/bin/kiss_mint.rs, C:/Projects/kiss-planb/conformance/tests/corpus_coverage.rs
- **failing_test:** corpus::tests: a cell is classified wider-than-compute iff stabilized_precision_bits>dtype_width, and certificate_precision_bits is readable ONLY as the nested detail under wider-than-compute (a free-standing precision field with no enum context is rejected). Fail: no enum type exists; corpus.rs currently aliases certificate.stabilized_precision_bits into a free-standing certificate_precision_bits (line 32/93).
- **implementation:** FOLD clog-review IMPORTANT + load-bearing carryover: define EvalPrecision{ComputeDtype,WiderThanCompute} as the authoritative derived enum; the reader DERIVES it (stabilized_precision_bits==dtype_width→compute-dtype; >→wider-than-compute) and exposes certificate_precision_bits only as the detail under wider-than-compute, never as a drift-prone standalone field. Format v1 JSON is UNCHANGED (still {hardness_margin_bits,stabilized_precision_bits}); this is a reader/minter reconciliation, not a schema change. Update corpus.rs, kiss_mint.rs, and corpus_coverage.rs together so the transcendental rollout is where the enum is introduced rather than propagating the existing corpus.rs:93↔coverage certificate_precision_bits mismatch.
- **verification:** cargo test corpus:: passes; existing add-cell tests still green (compute-dtype path, 24==24).

### T8-clog — clog complex atom (+ bf_sqrt, bf_atan, atan2) — DEFERRAL CANDIDATE, kept with fixes
- **depends_on:** T5-exp-log, T2-round-ziv, T3-constants, T7-eval-precision-enum
- **files:** C:/Projects/kiss-planb/conformance/src/hp.rs, C:/Projects/kiss-planb/conformance/src/semantics.rs
- **failing_test:** clog c32 vectors: clog(3+4i)=(log5, atan2(4,3)); branch cuts clog(-1,+0)=(+0,+pi=0x40490FDB) and clog(-1,-0)=(+0,-pi); four signed-zero quadrants clog(±0,±0); overflow-safe clog(1e38,1e38) real≈87.845; NaN propagation; AND near-unit-circle z=(1.0, k·2^-126) whose certificate MUST report stabilized_precision_bits==512 (not 256) plus its (k·2^-126,1.0) mirror. Fail: clog absent.
- **implementation:** Add bf_sqrt (Newton reciprocal-sqrt), bf_atan (benign: no Payne–Hanek, half-angle + Maclaurin), atan2 (C99 special-value table + interior). Real part = 0.5·log(re²+im²) reusing T5 log; imag = atan2. FOLD clog CRITICAL: real part near |z|≈1 with one tiny component is NOT correctly-roundable at 256 — REQUIRE Ziv escalation to 512 that RE-FORMS sumsq (re-squares both components at the wider width) and sets sumsq.ulp_err to the one-sided alignment-truncation bound so the straddle test fires; use the log1p-form as a correctness requirement, not optional; delete all 'stabilizes at 256' language. FOLD clog IMPORTANT: enumerate the FULL C99 Annex G table including conj-mirror rows ((x,-inf)→(+inf,-pi/2), (-inf,y<0)→(+inf,-pi), (+inf,y<0)→(+inf,-0)) — these are exact-match table logic no engine validates. Route all specials through semantics.rs; the split comparator (compare_c32_transcendental) enforces sign/±pi exactly. Emit certificate via T7 enum. NOTE: this is the explicit cut candidate if schedule slips (see residual risks).
- **verification:** cargo test hp::clog passes incl. the near-unit-circle 512 assertion and the two branch-cut sides.

### T9-minter — Minter: emit transcendental oracle cells (exp/log/sin f32+f64, clog c32) with certificate + edge tags
- **depends_on:** T5-exp-log, T6-sin, T8-clog, T7-eval-precision-enum
- **files:** C:/Projects/kiss-planb/conformance/src/bin/kiss_mint.rs, C:/Projects/kiss-planb/conformance/corpus/op_manifest.json
- **failing_test:** mint_roundtrip: minting exp/log/sin/clog cells produces class:ULP, provenance:oracle, correct tags, and a certificate; a mint-time self-check PANICS if any transcendental cell's stabilized_precision_bits is not strictly > dtype_width, and if any zero/±pi component is emitted with the wrong sign. Fail: minter only handles exact-byte add.
- **implementation:** Add a unary_transcendental cell emitter calling round_ziv(exp/log/sin,…) and a clog emitter; serialize Certificate into the frozen v1 certificate object unchanged; set eval_precision (T7) as a minter invariant (all oracle transcendental cells are wider-than-compute). Attach the load-bearing edge tags per atom and seed the required inputs (specials, domain boundaries, overflow, near-midpoint, large-|x|-trig, near-k-pi, near-unit-circle for clog). Add exp/log/sin/clog to op_manifest.json declared_coverage_set and transcendental_atoms.required_edge_tags. Assert stabilized>dtype (strict) before writing each cell.
- **verification:** cargo run --bin kiss_mint mints a green bundle; mint_roundtrip test passes; the strict-> and sign self-checks fire on injected-bad inputs.

### T10-reader-differential — Reader/differential: unary op+dtype dispatch, ulp_distance_f64/compare_f64, clog op-named split comparator + teeth tests
- **depends_on:** T9-minter
- **files:** C:/Projects/kiss-planb/conformance/src/lib.rs, C:/Projects/kiss-planb/conformance/tests/corpus_differential.rs
- **failing_test:** corpus_differential: exp/log/sin f64 cells compare under a NEW compare_f64/ulp_distance_f64 (u64/i64 totalOrder mirror of ulp_distance_f32); clog cells ALWAYS route to compare_c32_transcendental regardless of stored class; teeth tests: an IUT returning +0 for clog(-1,-0) imag (should be -pi) is caught by case (b), and +pi for clog(+0,-0) imag (should be -0) caught by case (a). Fail: runner only knows eval_add.
- **implementation:** Extend the differential runner to dispatch by op∈{exp,log,sin,clog} and dtype∈{f32,f64}. Add ulp_distance_f64 + compare_f64 to lib.rs (mirroring the f32 versions). Op-named split-comparator dispatch for clog (overrides class default). decode_c32 for the 8-byte c32 lane pair. Clone the a_normalize_to_plus_zero_add_is_caught teeth pattern for the clog branch-cut/signed-zero cases.
- **verification:** cargo test corpus_differential passes incl. the clog teeth tests.

### T11-clause-6.5-0009 — §6.5-0009 tightening: transcendental cells' stabilized_precision_bits STRICTLY > compute dtype width
- **depends_on:** T9-minter, T7-eval-precision-enum
- **files:** C:/Projects/kiss-planb/conformance/tests/corpus_coverage.rs
- **failing_test:** corpus_coverage: for every cell whose op is in transcendental_atoms, assert certificate_precision_bits > significand_bits(dtype) (strict); exact-byte cells keep >=. A transcendental cell minted at exactly 24/53 fails. Currently line 60 uses a uniform >= for all cells, so a strict-> transcendental violation would pass — the test must fail first against a crafted 24-bit transcendental cell.
- **implementation:** FOLD load-bearing carryover: split the assertion — transcendental atoms require strict >, exact-byte cells require >=. Read the classification via the T7 EvalPrecision enum (wider-than-compute) rather than the raw field so the check rides the authoritative enum. Update the comment (already describes the intent at lines 46-52).
- **verification:** cargo test corpus_coverage §6.5-0009 passes on the real bundle and fails on the crafted narrow transcendental cell.

### T12-clause-6.5-0008 — §6.5-0008 tightening: each transcendental atom's load-bearing EDGE TAGS must be present
- **depends_on:** T9-minter
- **files:** C:/Projects/kiss-planb/conformance/tests/corpus_coverage.rs, C:/Projects/kiss-planb/conformance/corpus/op_manifest.json
- **failing_test:** corpus_coverage: for each transcendental atom, assert the union of tags over its cells contains every required edge tag — exp/log: {nan-propagation,signed-zero,domain-boundary,overflow,near-midpoint,deep-tail}; sin: {nan-propagation,signed-zero,domain-boundary,near-midpoint,large-|x|-trig,near-k-pi}; clog: {nan-propagation,signed-zero,branch-cut,axis,overflow,near-midpoint,near-unit-circle}. An atom covered only at interior points fails. Fail: no per-atom edge-tag coverage assertion exists.
- **implementation:** FOLD 6.5-0008 carryover + clog IMPORTANT (near-unit-circle tag) + validation MINOR (near-pole scoped to tan/atan in later slices, NOT sin — sin uses near-k-pi). Add required_edge_tags per atom to op_manifest.json and assert their presence. Document deliberately-omitted tags per atom (e.g. clog omits large-|x|-trig/near-pole/near-k-pi) as decisions, not gaps.
- **verification:** cargo test corpus_coverage §6.5-0008 passes; deleting a signed-zero or near-unit-circle cell makes it fail.

### T13-validator — validate_corpus.py: real 3-source validation (mpmath + flint/gmpy2 + L-M/C99 anchors), bound-based Ziv, provenance guard
- **depends_on:** T9-minter
- **files:** C:/Projects/kiss-planb/tools/validate_corpus.py, C:/Projects/kiss-planb/tools/test_validate_corpus.py
- **failing_test:** test_validate_corpus: (a) a transcendental oracle bundle certifies green (skipTest if no 2nd engine importable); (b) an off-by-1-ULP exp expected is REJECTED; (c) a signed-zero clog lane flip is REJECTED; (d) check_engine_independence fails on a corrupted constant literal; (e) a non-oracle (promoted-differential/negative) cell is skipped, not recomputed. Fail: current validator is an exact-byte-only stub.
- **implementation:** Replace the stub. FOLD validation IMPORTANT #1: per-engine BOUND-based stopping rule — form the proven interval (Arb ball [lower,upper]; MPFR correct-rounding-to-p bound; mpmath ±1ulp@p) and commit only when correctly_round(lo)==correctly_round(hi); do NOT use the two-precision-agreement heuristic (it is common-mode across mpmath+MPFR). FOLD validation IMPORTANT #2: remove finite-domain exp overflow from special_value() — the finite/inf boundary is correctly-rounded (exp(0x40862E42FEFA39EF)=0x7FEFFFFFFFFFFF2A is finite); only ±inf/NaN are specials; all finite x route through the interval Ziv + integer correctly_round (its kk>bias carry emits Inf exactly). Guard provenance=='oracle' first. Pure-integer correctly_round (no float()). Third leg = L-M anchors (real) + C99 Annex G table (clog, incl. conj-mirror rows). Independence spot-check of engine constants vs pinned decimal literals. Emit PROVENANCE header with engine/lib versions + hard-case hashes + seed.
- **verification:** python -m pytest tools/test_validate_corpus.py passes where a 2nd engine is present; the off-by-ULP, lane-flip, and constant-corruption rejections all fire.

## Format / clause changes

- Format v1 (kiss-oracle-vectors-v1.json) is UNCHANGED — frozen from Plan A. The certificate object stays {hardness_margin_bits, stabilized_precision_bits}. evaluation_precision is a reader/minter-DERIVED authoritative enum {compute-dtype | wider-than-compute}, not a new JSON field; certificate_precision_bits becomes the nested detail under wider-than-compute (resolving the corpus.rs↔corpus_coverage.rs drift), not a free-standing field. (T7)
- §6.5-0009 clause test tightened: transcendental-atom cells must have stabilized_precision_bits STRICTLY GREATER than the compute dtype width; exact-byte cells keep >=. Current corpus_coverage.rs:60 uses a uniform >= and must be split. (T11)
- §6.5-0008 clause test tightened: each transcendental atom must present its full set of load-bearing edge tags (per-atom required_edge_tags), not just interior points; near-unit-circle added for clog; near-pole scoped to tan/atan in later slices (NOT sin, which uses near-k-pi). (T12)
- op_manifest.json (conformance/corpus/op_manifest.json): add exp/log/sin/clog to declared_coverage_set and populate transcendental_atoms[atom].required_edge_tags. (T9/T12)
- Constant sizing OVERRIDES the design doc's ~1280-bit 2/pi and 256-bit ln2 figures: 2/pi=2304 bits (36 words), ln2/1-over-ln2/pi-over-2=1152 bits (18 words), sized for the 1024-bit escalation ceiling; in-crate verification widened to the full table width, not MAX_L. (T3)
- validate_corpus.py: exp finite/inf boundary is NOT a special-value pre-pass entry (it is correctly-rounded); Ziv stopping rule is bound-based interval-straddle per engine, not two-precision agreement. (T13)

## Residual risks

- clog (T8) is the explicit CUT CANDIDATE for Slice 0. No kernel was flagged 'flawed' (all six were 'needs-fixes'), but clog carries the most new machinery (bf_sqrt+bf_atan+atan2+the pi table's dependents + c32 dtype path + op-named split-comparator dispatch + the C99 Annex G exact-match table + a validator complex leg) and its one CRITICAL defect (near-|z|=1 real-part must escalate to 512 with re-squaring) is fixable but adds the only genuinely-exercised escalation path in the slice. If schedule slips, defer clog to Slice 1 and ship exp/log/sin at f32+f64 as a coherent real-only Slice 0 — the carryover clause tightenings (T11/T12) and the evaluation_precision enum (T7) do not depend on clog. The clog spec's own argument to KEEP it is that it is the point of picking a complex op now; the lead-engineer call is: keep unless the atom chain runs late.
- NaN canonicalization for sin/clog is an unresolved SPEC RULING (sin & validation reviews, MINOR): pinning a single canonical qNaN as exact-byte over-constrains conformant IUTs that propagate the input NaN payload. Default: canonical qNaN exact-byte pending the KISS spec author's decision to instead compare NaN cells by an is-NaN predicate or a pinned payload-propagation convention (semantics.rs style).
- No second arbitrary-precision Python engine (gmpy2 / python-flint) is installed on this machine — validate_corpus.py can only run in the non-certifying --allow-mpmath-only smoke mode here. A real freeze run REQUIRES `pip install python-flint` (Windows-friendly, covers real+complex, MPFR-independent, gives Arb-ball bound-based Ziv). clog cannot be certified without a complex engine (gmpy2 has no complex; flint acb is mandatory).
- Large-|x| trig near 2^1023 (round-ziv & sin IMPORTANT): the 2304-bit 2/pi table correctly rounds f64 there, but the certificate's uniform ~244-bit-accuracy premise is overstated in that regime (~197-bit reduced-arg accuracy). Mitigated by adding an explicit 2^1023 stressor (T4/T6) and by the table-length guard that panics-not-certifies; if a future dtype wider than f64 is oracled the table must regrow.
- The 512/1024 escalation ladder is exercised in Slice 0 ONLY by constructed near-midpoint (exp/log/sin) and near-unit-circle (clog) inputs — no naturally-occurring curated cell forces it. This is by design (exp/log/sin resolve at 256), but it means the escalation + non-escalating-constant interaction is validated only by those seeded stressors; the 1024-unresolved path remains a panic/red-flag, never a normal outcome.
- Slices 1–3 (cos/tan/atan/atan2/erf/lgamma/pow, more dtypes, coverage closure) are explicitly OUT of scope and fan out per-atom later; atan2/bf_atan land in Slice 0 only as clog's scoped internals with no standalone atan/atan2 corpus cells.

---
# Kernel design specs (6 of 7; hp-core recovered separately)

## round-ziv — round-to-f64/f32 + exact midpoint + Ziv escalation + certificate  (difficulty: high)
**Summary:** The rounding kernel for the wide-precision oracle: a single-rounding, round-ties-to-even reducer from the [u64;N] binary big-float to f64/f32, wrapped in a Ziv decidability loop. Because the big-float and every f64/f32 rounding boundary (midpoint) are exact binary numbers, the "does the true value's rounding decide?" test is EXACT integer arithmetic on the low limbs: compare the integer distance-to-nearest-midpoint D against the running error bound err_ulps. D > err_ulps ⇒ decided (round once, emit); D ≤ err_ulps ⇒ straddle ⇒ recompute at 512 then 1024 (which shrinks err_ulps) and retest. Each cell emits a certificate {hardness_margin_bits = leading-zero run of D (bit-distance to the nearest midpoint), stabilized_precision_bits = the P at which D>err first held}. 256 bits suffices because f64 needs only 53 + worst-run(~67, Lefèvre–Muller) + guard ≈ 128 bits, so escalation essentially never fires on the curated set; the loop exists for airtight certification and as a red-flag detector (a genuine transcendental value is never exactly a dyadic midpoint, so 1024-unresolved means a mis-routed special value or an error-bound bug). evaluation_precision is always "wider-than-compute" (256 > 53 > 24), and stabilized_precision_bits is strictly greater than the compute-dtype width by construction, satisfying the §6.5-0009 carryover.

**Review verdict:** needs-fixes (constant_width_adequate=False)
_The core midpoint/Ziv logic, single-rounding discipline, overflow threshold, and anchor bit-patterns are correct, but there is one CRITICAL misround: the `avail <= 0 ⇒ signed zero` underflow short-circuit is wrong at the `avail == 0` boundary and directly contradicts the spec's own exp(-745) → 0x0000000000000001 anchor (that value is 0.587× the smallest subnormal, E=-1075, avail=0, and rounds UP, but the rule emits 0x0). Beyond that, two structural soundness gaps: (a) the 512/1024 escalation cannot shrink error dominated by the fixed-width 256-bit ln2 / 1280-bit 2/π constants (they don't escalate with N), so a constant-table-limited straddle would spin to 1024 and panic with a misdiagnosis; (b) the §6 uniform ~244-bit-accuracy claim is false for large-|x| trig near 2^1023, where a 1280-bit 2/π table yields only ~197-bit reduced-argument accuracy — still correctly rounding f64, but the certificate premise is overstated and the regime is untested (the only large-x vector, sin(1e22)=sin(2^73), exercises almost none of the table). Outputs on the modest-|x| curated set are correct once the avail==0 bug is fixed; the escalation/constant and clog-cancellation issues are latent because slice 0 contains no cell that actually forces escalation or the |z|≈1 clog site._

**Review findings:**
- (critical) `avail <= 0 ⇒ signed zero` (§2/§7/edge_cases) misrounds every value in (2^-1075, 2^-1074). avail==0 is a straddle boundary, not round-to-zero. Concrete: exp(-745) = 1.1735·2^-1075 = 0.587× smallest subnormal, E=-1075, avail = 53-(-1022-(-1075)) = 0; RN rounds it UP to 0x0000000000000001 (the spec's own anchor), but the rule emits 0x0000000000000000. Same bug at f32 near 2^-150.
  - fix: Short-circuit to zero only for avail < 0. For avail == 0, run the q/R/S machinery: q=0, shift=P, R = leading bit = 1, round_up = S (sticky). Guard the shift==P case so masks (1<<shift)-1 are computed as all-ones limb-wise rather than overflowing the [u64;N] width.
- (important) Ziv escalation to 512/1024 cannot shrink error floored by a fixed-width constant, because the 256-bit ln2 and ~1280-bit 2/π tables are not escalated with N. A straddle whose dominant error is constant-table truncation would spin 256→512→1024 with err_ulps not shrinking, then hit the panic, misdiagnosed as 'mis-routed special value / error-bound bug'.
  - fix: Either escalate the ln2 / 2/π tables in lockstep with N, or explicitly document that the escalation ladder only shrinks op/rounding error and size the constants so their truncation floor is always below the hardest f64/f32 hard-to-round margin (~2^-130) for the entire curated argument range.
- (important) §6's blanket claim 'err_ulps < 2^12 (≥ P−12 ≈ 244 effective bits) for every curated cell, escalation essentially never fires' is false for large-|x| trig near 2^1023: a 1280-bit 2/π table gives only ~197-bit relative accuracy of the reduced argument there (table error ~2^-257 abs ÷ worst reduced-arg magnitude ~2^-60). Output still rounds f64 correctly (197 > ~130), but the certificate/theorem premise is overstated, and the regime is untested — the only large-x vector, sin(1e22)=sin(2^73), exercises almost none of the table's high bits.
  - fix: Add a genuine |x| ~ 2^1023 sin/large-trig stressor to the corpus, and either widen the 2/π table to the oracle's own 256-bit precision goal (≈1023 + 256 + worst-run + guard bits) or honestly fold the reduction error into err_ulps and accept that escalation DOES fire in this regime (contradicting §6).
- (important) clog real part 0.5·log(re²+im²) is a catastrophic-cancellation site when |z|≈1 (result ~0.5·ε). The kernel trusts err_ulps as a rigorous bound, but the spec only says 'via a hypot/log1p form' without pinning a cancellation-safe argument, and the required edge-tag list has no near-unit-circle tag, so §6.5-0008 coverage would miss it. Concrete: z=(1+2^-50)+0i.
  - fix: Pin the cancellation-safe reduction Re(clog)=0.5·log1p((|re|-1)(|re|+1)+im²) computed without forming re²+im² and subtracting 1, and add a 'near-unit-circle' load-bearing edge tag for clog to the §6.5-0008 required set.
- (minor) D ranges up to ~2^202 (f64, N=4) and up to ~2^(shift-1) generally, but err_ulps is declared u128 (max 2^128). A literal `D > err_ulps` compare must widen D to a big integer.
  - fix: Compute the D > err_ulps comparison as a big-integer compare (D as [u64;N] limbs), never truncating D into a u128.

**Algorithm:**

# Kernel: big-float → f64/f32 round + exact-midpoint + Ziv escalation + certificate

Scope: the rounding/decidability core of `conformance/src/hp.rs`. The atoms (exp/log/sin/clog) feed this kernel a `(BigFloat, err_ulps)` pair; the kernel produces the correctly-rounded target bits + a self-certifying certificate. Follows the from-scratch, dependency-free, spec-pinned precedent of `conformance/src/fp.rs`.

## 0. The big-float type (context the kernel operates on)

```rust
// N = 4 → 256 bit, 8 → 512, 16 → 1024. P = 64*N is the working precision.
#[derive(Clone)]
struct BigFloat<const N: usize> {
    sign: bool,        // true = negative
    exp:  i32,         // value = (-1)^sign * (mant as P-bit int) * 2^exp
    mant: [u64; N],    // NORMALIZED: bit (P-1) is 1  (mant ∈ [2^(P-1), 2^P))  for Normal
    class: FpClass,    // Normal | Zero | Inf | NaN   (Zero carries sign; Inf/NaN from atoms)
}
```

Value of a `Normal` = `(-1)^sign · m · 2^exp`, where `m` is the P-bit big integer in `mant`, `m ∈ [2^(P-1), 2^P)`. The big-float ULP is `2^exp`; the leading bit has weight `2^(P-1+exp)`, so the binary exponent (index of the leading 1) is `Ebin = P-1+exp` and `|value| ∈ [2^Ebin, 2^(Ebin+1))`.

**Error contract from the atoms.** Each atom returns `Evaluated { val: BigFloat<N>, err_ulps: u128 }`, the RIGOROUS statement `|val − v_true| ≤ err_ulps · 2^exp` (error in units of the big-float ULP). `err_ulps` is the running forward-error bound: each big-float op contributes ≤ a small constant; truncation error of series/asymptotics (`erf`, `lgamma` in later slices) is bounded and ADDED into `err_ulps` (a Ziv test on rounding alone would silently miss truncation — it must be in the interval). For a ~50-op reduction chain, `err_ulps < 2^12`, i.e. ≥ P−12 effective bits. The kernel treats `err_ulps` as an opaque non-negative bound.

## 1. Target-format parameters

```rust
struct Fmt { t: u32, emin: i32, emax: i32, width: u32 } // significand bits, exp range, dtype bits
const F64: Fmt = Fmt { t: 53, emin: -1022, emax: 1023, width: 64 };
const F32: Fmt = Fmt { t: 24, emin:  -126, emax:  127, width: 32 };
```
(`t` counts the implicit leading 1: 53 = 1+52 for f64, 24 = 1+23 for f32.)

## 2. Locating the round position (normal + subnormal)

Given a `Normal` big-float with binary exponent `Ebin = P-1+exp`:

```
E = Ebin                                   // tentative unbiased result exponent
avail = if E >= fmt.emin { fmt.t }          // # significand bits available in the target
        else { fmt.t as i32 - (fmt.emin - E) }  // subnormal: fewer bits (may be <= 0)
```
`avail <= 0` ⇒ magnitude is below half the smallest subnormal ⇒ result is signed zero (see §7). Otherwise split `m` at the round bit. Let `shift = P - avail` (number of low bits below the kept significand):

```
q     = m >> shift                          // candidate significand, round-toward-zero (avail bits)
R     = (m >> (shift - 1)) & 1              // the round bit
F     = m & ((1<<shift) - 1)                // the full fractional field below q  (0 <= F < 2^shift)
S     = (F & ((1<<(shift-1)) - 1)) != 0     // sticky = any 1 strictly below the round bit
```
(All via limb-indexed shifts on `[u64;N]`; `shift` can exceed 64, so operate limb-wise.)

## 3. Exact round-ties-to-even of the big-float (single rounding)

```
round_up = R==1 && (S || (q & 1)==1)        // R && (sticky OR odd) = ties-to-even
sig = q + round_up as u128                   // may carry to 2^avail
if sig == (1 << avail) {                      // mantissa carry
    sig >>= 1; E += 1;                         // renormalize; re-check overflow below
}
```
Encode `(sign, E, sig)` into the target IEEE bit pattern (implicit-bit strip for normals; for subnormals `E==emin` and `sig` already carries the denormalized significand; `sig == 1<<(t-1)` after a subnormal carry promotes cleanly to the smallest normal, exactly as `fp.rs::fp8_magnitude` promotes). **Round exactly ONCE from the big-float to the target.** For an f32 cell, round the SAME 256-bit `val` directly with `F32` — NEVER f64-then-f32 (double rounding changes the result on midpoint-adjacent values; see edge cases).

## 4. Exact distance to the nearest midpoint (the load-bearing primitive)

The midpoint between the two adjacent target floats straddling `X` sits at fractional field `Fmid = 2^(shift-1)` (round bit set, all sticky 0). Because `X` and `Fmid` are both exact binary integers scaled by `2^exp`, the distance is EXACT integer arithmetic on the low limbs:

```
Fmid = 1 << (shift - 1)
D    = abs_diff(F, Fmid)         // exact big-int subtraction on the low `shift` bits; D >= 0
```
`D` is the distance from `X` to the nearest midpoint, measured in big-float ULPs (`2^exp`). `D == 0` ⇔ `X` lands EXACTLY on a midpoint.

Only the upper midpoint (`Fmid`) is reachable: by construction `F ∈ [0, 2^shift)` so `X ≥ q`; the lower midpoint (between `q−1` and `q`) is at distance `F + 2^(shift-1) ≥ 2^(shift-1) ≫ err_ulps`, so it can never be straddled while `err_ulps < 2^(shift-1)` (asserted in §5). Hence a single `D` fully characterizes hardness.

**Hardness margin (certificate):**
```
hardness_margin_bits = if D == 0 { (shift - 1) as u32 /* cap; also set EXACT_MIDPOINT flag */ }
                        else { (shift - 1) - bitlength(D) }   // leading-zero run of D in a (shift-1)-bit field
```
This is the number of bits by which `X` misses the midpoint — the run length past the round bit (~57–67 for the hardest f64 transcendentals per Lefèvre–Muller). Larger = closer to the midpoint = harder.

## 5. The Ziv decidability test (exact) and escalation loop

The true value `v ∈ [X − err_ulps·2^exp, X + err_ulps·2^exp]`. Its rounding is DECIDED iff the whole interval stays strictly inside one rounding cell, i.e. it does not reach the nearest midpoint:

```
DECIDED  ⇔  D >  err_ulps        // strict, exact integer compare
STRADDLE ⇔  D <= err_ulps        // escalate
assert!(err_ulps < (1 << (shift - 1)));   // guarantees only the nearest midpoint matters
```
When `DECIDED`, every point of the interval rounds to `RN(X)` from §3 → emit it. When `STRADDLE`, recompute the atom at the next precision and retest:

```rust
fn round_target(atom: &dyn Fn(usize) -> Evaluated, fmt: Fmt) -> RoundResult {
    for &N in &[4usize, 8, 16] {              // 256 -> 512 -> 1024
        let Evaluated { val, err_ulps } = atom(N);   // atom re-evaluated at width N
        if !val.class.is_normal() { return special_case(val, fmt, N); } // Inf/NaN/Zero
        let (bits, D, shift) = round_and_measure(&val, fmt);
        if D > err_ulps {
            return RoundResult {
                bits,
                eval_precision: EvalPrecision::WiderThanCompute,
                cert: Certificate {
                    hardness_margin_bits: leading_run(D, shift),
                    stabilized_precision_bits: (64 * N) as u32,   // 256 | 512 | 1024
                },
            };
        }
        // else straddle: fall through to wider N
    }
    panic!("hp: rounding unresolved at 1024 bits for <op,input> — \
            mis-routed exact/special value or error-bound bug; investigate");
}
```

Escalation shrinks the interval: at width `N` the relative error floor is ~`2^-(64N - err_bits)`, so `err_ulps` (in the target-relative frame) drops by ~256 bits per step, while the true `D` is fixed. A genuine transcendental `v` at a dyadic (rational) argument is transcendental (Lindemann–Weierstrass) hence NEVER exactly a target midpoint, so `D_true > 0` and the loop terminates — usually at 256, and reaching 1024 unresolved is a genuine red flag, not a normal outcome. The trivial exact cases (`exp(0)=1`, `log(1)=0`, `sin(0)=0`) are routed to `semantics.rs` special-value logic BEFORE the transcendental core and never reach this loop.

`stabilized_precision_bits` = the `64*N` at which `D > err_ulps` first held. (Optional belt-and-suspenders: also require `RN(X)` identical across two consecutive `N`; the interval test already suffices when `err_ulps` is a true bound, so this is a redundant cross-check, not the gate.)

## 6. Why 256 bits suffices for f64 (and f32)

Correct rounding to a `t`-bit target needs `t + worst_run + guard` bits. For f64: `53 + 67 + ~8 = 128` (worst hard-to-round run ~57–67 bits, Lefèvre–Muller). 256 clears that by ~128 bits, so with `err_ulps < 2^12` the effective accuracy is ~244 bits ≫ 128 → `D > err_ulps` holds at 256 for every curated cell; escalation is exercised by construction/stress only. For f32 the floor is `24 + ~30 + 8 ≈ 62` bits, cleared even more comfortably. This makes the certificate a THEOREM on the frozen set (`hardness_margin_bits ≪ 256 − err_bits`), not an empirical hope. Escalation actually triggers only for a deliberately near-midpoint-constructed input whose run length approaches `256 − err_bits`, or a composed atom (`pow = exp(b·log a)` in later slices) whose error amplification inflates `err_ulps` — exactly the cases the 512/1024 wiring exists to certify.

## 7. Non-finite / zero / overflow / underflow in the kernel

- **Atom returned Inf/NaN/Zero** (`class != Normal`): bypass rounding. NaN → the target's canonical quiet NaN (payload per `semantics.rs`); Inf → target ±Inf; Zero → target ±0 with sign preserved. Certificate: `hardness_margin_bits = 0`, `stabilized_precision_bits = 64*N` (still wider-than-compute; keeps the §6.5-0009 strict-`>` invariant true even for special cells).
- **Overflow on round**: after §3, if `E > fmt.emax` (or the mantissa carry pushed `E` past `emax`) → ±Inf. The RNE overflow boundary (`X ≥ 2^(emax+1) − 2^(emax−t)`, the midpoint between max-finite and Inf) falls out of the same `q/R/S` machinery by treating the significand-overflow slot as "the value above max finite". Anchor: `exp(710)` → `7FF0000000000000`.
- **Underflow**: `avail <= 0` (from §2) or `X` below half the smallest subnormal → signed zero. `avail ∈ [1, t)` gives a subnormal with reduced precision; the identical `q/R/S/D` logic applies at the shifted round position. Anchors: `exp(-745)` → `00…01` (smallest subnormal), `exp(-746)` → `00…00`.

## 8. Certificate + evaluation_precision (carryover mapping, format v1 untouched)

```rust
enum EvalPrecision { ComputeDtype, WiderThanCompute }   // authoritative enum
struct Certificate { hardness_margin_bits: u32, stabilized_precision_bits: u32 }
struct RoundResult { bits: u64, eval_precision: EvalPrecision, cert: Certificate }
```
- The hp core ALWAYS sets `eval_precision = WiderThanCompute` (256/512/1024 ≫ 53/24). `stabilized_precision_bits` is the numeric DETAIL under that enum value — never a free-standing field that can drift (carryover honored).
- **Kernel invariant (assert at emit):** `stabilized_precision_bits > compute_dtype_width` strictly (256 > 53, 256 > 24). This is what `test_conform_oracle_vector_stores_wide_precision_value` (§6.5-0009) asserts on every transcendental cell (strict `>`, not `>=`).
- The minter serializes `cert` into the frozen v1 `certificate` object `{hardness_margin_bits, stabilized_precision_bits}` **unchanged** — format v1 is NOT modified. `eval_precision` is enforced as a minter-level invariant over every `provenance:"oracle"` transcendental cell (all are `wider-than-compute`), and `validate_corpus.py` guards `provenance == "oracle"` before recomputing — it does not require a new v1 field.

## 9. Wiring for the surrounding slice-0 components (for the implementer)

- **Reader/tests** (`corpus_coverage.rs` §6.5-0008): the load-bearing edge tags the minter must attach per transcendental atom are surfaced by the input set the atom hands this kernel — the kernel's job is only correctness+certificate, but the tags (`NaN-propagation`, `signed-zero`, `domain-boundary`, `overflow`, `near-midpoint`, `large-|x|-trig`, `near-pole`, `near-k-pi`, `deep-tail`) ride on the cell. The near-midpoint tag is exactly a cell whose `hardness_margin_bits` is large / that forced escalation.
- **f32 cells**: call `round_target(atom, F32)` on the same 256-bit atom closure; the certificate's `stabilized_precision_bits` (≥256) is trivially `> 24`.
- **clog** (complex): rounds each component (re, im) through this kernel independently at `≤0.5 ULP/component`; branch-cut/signed-zero components are exact per C99 Annex G (routed via `semantics.rs`, compared by `compare_c32_transcendental`), not through the tolerance path.

**Test vectors:**
- `op=exp dtype=f64 x=3FF0000000000000 (1.0)` -> `4005BF0A8B145769`  e; benign, decides at 256; hardware==correctly-rounded; anchor also cross-checks against Math.E const.
- `op=log dtype=f64 x=4000000000000000 (2.0)` -> `3FE62E42FEFA39EF`  ln2; exercises 256-bit ln2 reduction path; decides at 256.
- `op=sin dtype=f64 x=3FF0000000000000 (1.0)` -> `3FEAED548F090CEE`  benign trig; decides at 256.
- `op=sin dtype=f64 x=3FE0000000000000 (0.5)` -> `3FDEAEE8744B05F0`  benign trig interior point.
- `op=sin dtype=f64 x=4480F0CF064DD592 (1e22)` -> `BFEB453AB76BF397`  LARGE-|x| Payne–Hanek stressor (sin(1e22)=-0.8522008497671888); requires the ~1280-bit 2/pi table — a truncated table misrounds ~100%. tag: large-|x|-trig.
- `op=exp dtype=f64 x=4086240000000000 (709.0)` -> `7FDD422D2BE5DC9B`  near-overflow boundary (8.218e307); tag: overflow-adjacent.
- `op=exp dtype=f64 x=4086400000000000 (710.0)` -> `7FF0000000000000`  OVERFLOW → +Inf via RNE overflow threshold. tag: overflow.
- `op=exp dtype=f64 x=C0874F0000000000 (-745.0)` -> `0000000000000001`  smallest positive subnormal; exercises avail<t subnormal round position. tag: domain-boundary/underflow.
- `op=exp dtype=f64 x=C08750... (-746.0)` -> `0000000000000000`  UNDERFLOW → +0 (below half smallest subnormal). tag: underflow.
- `op=log dtype=f64 x=3FF0000000000000 (1.0)` -> `0000000000000000`  exact zero via semantics.rs special routing — must NOT enter the transcendental core / Ziv loop.
- `op=exp dtype=f32 x=3F800000 (1.0f)` -> `402DF854`  matches the design-doc §4 sample; f32 rounded DIRECTLY from 256-bit (no f64 intermediate). certificate stabilized_precision_bits=256 > 24 (strict).
- `op=log dtype=f32 x=40000000 (2.0f)` -> `3F317218`  ln2 in f32.
- `op=sin dtype=f32 x=3F800000 (1.0f)` -> `3F576AA4`  sin(1) f32; single-rounding from big-float.
- `NEAR-MIDPOINT construction (slice-1 mint): an argument whose exp/log result has hardness_margin_bits approaching 256−err_bits` -> `certificate.stabilized_precision_bits == 512 (or 1024)`  This is the case that FORCES escalation and exercises the 512/1024 wiring; the exact input is minted/certified in slice 1 against the 3 sources (CORE-MATH .wc hard cases). Pin: hardness_margin_bits large, stabilized_precision_bits>256. tag: near-midpoint.

**Constants required:** No mathematical constants are owned by the KERNEL itself — the kernel is pure integer/bit logic on the big-float limbs. It only consumes (BigFloat, err_ulps) from the atoms.; Format parameters the kernel hard-codes: f64 = {t:53, emin:-1022, emax:1023, width:64}; f32 = {t:24, emin:-126, emax:127, width:24-significand/32-bit}.; Escalation ladder: P ∈ {256, 512, 1024} i.e. N ∈ {4, 8, 16} u64 limbs.; Compute-dtype widths for the strict-> certificate invariant: 53 (f64), 24 (f32).; (Owned by the ATOMS, not this kernel, but the kernel's correctness depends on them being full-width: 256-bit ln2 for exp/log reduction; ~1280-bit 2/pi Payne–Hanek table for sin; these bound err_ulps.)
**Edge cases:** Exact midpoint of the TRUE value (D_true == 0): impossible for a transcendental at a dyadic argument (Lindemann–Weierstrass) — the loop would escalate forever, so it must panic/red-flag at 1024, signalling a special value mis-routed into the transcendental core or an err_ulps bug. Trivial exacts (exp(0), log(1), sin(0)) are intercepted by semantics.rs before the core.; Big-float exactly on a midpoint but err_ulps>0 (D==0 at width N): interval contains the midpoint ⇒ STRADDLE ⇒ escalate; at higher P the atom resolves whether v is just above/below.; Mantissa carry on round-up (sig == 2^avail): renormalize sig>>=1, E+=1, then RE-CHECK overflow — a carry at emax produces Inf.; Subnormal round position: avail = t - (emin - E) < t reduces the kept-bit count; same q/R/S/D logic at the shifted position; a subnormal round-up to 2^(t-1) promotes cleanly to the smallest normal.; avail <= 0: magnitude below half the smallest subnormal ⇒ signed zero (sign preserved).; Overflow: E > emax or carry past emax ⇒ ±Inf via the RNE overflow-midpoint threshold (exp(710)→+Inf).; Underflow to signed zero: exp(-746)→+0; smallest subnormal boundary exp(-745)→0x…01.; shift > 64: the split at the round bit spans multiple u64 limbs — all of q/R/F/S/D/Fmid must be computed limb-wise, never assuming shift fits in one word (P up to 1024).; err_ulps >= 2^(shift-1) (interval wider than half a rounding cell): assertion failure — means the atom's error bound is catastrophically loose or precision far too low; must not silently pass.; f32 double-rounding hazard: rounding 256-bit→f64→f32 can differ from 256-bit→f32 on values whose f64 round lands adjacent to an f32 midpoint; kernel MUST round the big-float directly to f32 (single rounding).; Signed zero and NaN: never enter the tolerance/midpoint path — routed as special class, sign/payload preserved, compared exact-byte (or split for complex).; Non-normal big-float from the atom (Inf/NaN/Zero): bypass §2–§5; certificate still emits stabilized_precision_bits=64*N to keep the strict-> §6.5-0009 invariant true.
**Open questions:** err_ulps concrete unit: is it cleaner to carry the error as an integer count of big-float ULPs (assumed here) or as a separate (mantissa, exp) big-float bound? Integer-ULP keeps the D>err_ulps test a single big-int compare; a big-float bound is tighter for composed atoms (pow). Recommend integer-ULP for slice 0, revisit when pow lands in slice 1.; Should the kernel also emit hardness_margin_bits for exact/special cells as 0, or as a sentinel (e.g. u32::MAX) to distinguish 'exact' from 'margin 0 but rounded'? Frozen v1 shows 0 for the add cells, so 0 is chosen; confirm no §6.5-0008 test needs the distinction.; Belt-and-suspenders 'RN identical across two consecutive P' cross-check: include it as a debug_assert only, or as a hard gate? The interval test is sufficient if err_ulps is sound; recommend debug_assert to catch an unsound atom error-bound during development without doubling mint cost.; 1024-unresolved policy: panic (chosen, matches the risk-table red flag) vs. emit with a 'needs-investigation' certificate flag. Panic is safer for a freeze gate but blocks minting the rest of the bundle; a collect-and-report mode may be friendlier for slice-1 fan-out.

---

## reduction — constants (2/pi, ln2) + Payne-Hanek trig reduction + exp/log reduction  (difficulty: very-high)
**Summary:** The argument-reduction kernel for the KISS wide-precision oracle (conformance/src/hp.rs). It supplies three things atop the BigFloat<L> core (L in {4,8,16} = 256/512/1024 bits): (1) hard-coded binary constant tables for 2/pi, ln2, 1/ln2, and pi/2, each emitted as [u64;N] big-endian literals, generated at dev time from three independent engines and RE-VERIFIED inside the shipped crate with zero external deps via an in-crate Machin/atanh recomputation plus published-hex anchors; (2) a big-integer Payne-Hanek reduction that takes any f64/f32 argument up to ~2^1023 to an octant (k mod 8) plus a reduced BigFloat r in [-pi/4, pi/4] carrying a bounded truncation error, retaining full relative precision even under the ~61-67 bit cancellation that hits x near a multiple of pi/2; and (3) Cody-Waite-style exp reduction (x = k*ln2 + r) and log reduction (x = 2^e * m). The load-bearing correction to the design doc: the "~1280-bit 2/pi" and "256-bit ln2" figures are sized only for the 256-bit level; because Ziv escalation to 512/1024 must rebuild r at that precision, the tables MUST be provisioned for the maximum escalation (2/pi ~= 2304 bits / 36 words; ln2 ~= 1152 bits / 18 words). A too-short table is the kernel's most dangerous failure because the error is systematic across precisions, so Ziv escalation cannot see it and would certify a confidently-wrong value. Every reduction therefore begins with a runtime assert that the required constant window fits the stored table.

**Review verdict:** needs-fixes (constant_width_adequate=True)
_The core numerics are sound. I verified the Payne-Hanek machinery end-to-end and it is correct: (a) dropping 2/pi bits with i<=E-3 is valid because p_i contributes M*2^(E-i), an integer multiple of 8 for E-i>=3, invisible to both octant (mod 8) and r; (b) the alignment identity E-i1+F=-p makes scaled = prod>>p = floor(x*(2/pi)*2^F) exactly; (c) octant round-up via the single bit F-1 is genuine round-to-nearest (exact ties impossible since x*2/pi is irrational for x!=0), and (n+1)&7 wraps mod 8 correctly; (d) r = f'*(pi/2) with f' in [-1/2,1/2] lands in [-pi/4,pi/4]. I re-derived the sin(pi_nearest) vector by hand: octant=2, r=-(pi-pi_double)~=-1.2246e-16, sin=-sin(r)=+1.2246e-16, matching the positive bit pattern 0x3CA1A62633145C07. Sign, cancellation (~61-67 bits, guard G_c=128 with margin), and NaN/inf/signed-zero delegation to semantics.rs all check out. Constant WIDTH sizing is adequate and correctly overrides the design doc's too-short ~1280/256-bit figures: i1_max = 971+1152+53 = 2176 <= 2304 (36 words), with the retained-fraction width F=64L+128 leaving >64L good bits of r even after worst-case cancellation, and the truncation tail 2^-(F-1) sitting far below working precision.

However, the spec is NOT ship-ready because its own named worst failure -- a systematic, precision-independent, Ziv-blind wrong bit deep in a constant table -- is not actually guarded against by the shipped mitigation. The in-crate self-checks (section 1.4) run the Machin/atanh recomputation in BigFloat<MAX_L=16> and assert only "the top 64*MAX_L-64 = 960 bits equal the literals." But the tables are wider than MAX_L: 2/pi is 2304 bits and ln2 is 1152 bits, and the LARGE-|x| reduction consumes 2/pi bits out to index ~2176 (exactly what sin(1e22) and sin(2^1023) exercise) and exp/log at L=16 consume ln2 bits out to ~1085. Those load-bearing tail bits (roughly indices 960..2176 of 2/pi and 960..1085 of ln2) are verified in the shipped crate by NOTHING: the Machin self-check tops out at 960 bits, the reciprocal-identity test uses TWO_OVER_PI truncated to a 1024-bit BigFloat (blind below ~1024), and the anchors pin only words 0-2 (192 bits). A transcription typo in, say, word 20 of TWO_OVER_PI survives all four section-1.4 tests, then produces a confidently-wrong sin for large arguments with a healthy-looking hardness margin -- precisely the scenario section 5 declares the kernel's most dangerous. This is a genuine, implementer-facing hole (the verification width is wrongly tied to MAX_L instead of to the table width), and it is fixable without changing the algorithm, so the verdict is needs-fixes rather than flawed._

**Review findings:**
- (critical) In-crate constant verification (section 1.4) covers only the top ~960 of the 2304-bit 2/pi table and 1152-bit ln2 table. The self-check runs in BigFloat<MAX_L=16> (1024-bit significand) and asserts only 'top 64*MAX_L-64 = 960 bits'; the reciprocal-identity test uses TWO_OVER_PI truncated to 1024 bits; anchors pin only words 0-2 (192 bits). The tail bits that ARE load-bearing for large |x| -- 2/pi indices ~960..2176 (sin(1e22), sin(2^1023)) and ln2 bits ~960..1085 (exp/log at L=16) -- are unverified in the shipped crate. A typo there is systematic across precisions, so Ziv escalation is blind to it and certifies a confidently-wrong value with a healthy hardness margin: exactly the failure section 5 names as most dangerous, whose mitigation #3 (in-crate recomputation) does not actually reach the dangerous bit range. FIX: decouple the recomputation width from MAX_L. Run the Machin (2/pi, pi/2) and atanh (ln2, 1/ln2) recomputations in a big-integer wide enough for the full table plus guard (>= ~2400 bits for 2/pi, >= ~1200 for ln2 -- i.e. BigFloat<~40> or a dedicated scratch big-int), and assert ALL stored table bits minus a small low guard. Additionally pin every published fdlibm ipio2 word (not just 3) as static anchors so the tail has an independent second check.
  - fix: Verify the full table width in-crate: recompute constants in a big-int/BigFloat wider than the tables (>=2400 bits for 2/pi, >=1200 for ln2), assert all bits minus a small guard, and pin all ~34 fdlibm ipio2 words as anchors rather than only words 0-2.
- (important) The sin(1.0) test-vector NOTE contradicts the kernel's own reduction. It states 'octant=0, r=1.0 (r=x since 1<pi/2 gives octant 0)'. But r is defined to lie in [-pi/4, pi/4] and 1.0 > pi/4 ~= 0.785, so r=1.0 is impossible. The actual reduction: x*2/pi ~= 0.6366 >= 1/2 -> half_set -> octant=(0+1)&7=1, fprime=(1-0.6366), r ~= -0.5708, and octant-1 gives sin(x)=cos(r)=cos(0.5708)=0.8415. The expected bits 0x3FEAED548F090CEE are correct, but section 6 mandates 'reduction-only unit assertions', and any assertion written from this note (octant==0, r==1.0) will fail against a correct implementation, or worse, be 'fixed' by breaking the code to match the note.
  - fix: Correct the sin(1.0) note to octant==1, r ~= -0.5708 (=-(1 - 1*2/pi)*pi/2), sin=cos(r); keep the expected bits unchanged.
- (minor) The f64 fast-path for exp's k, '(x/LN2).round()', can differ from the correctly-rounded BigFloat k near a half-integer x/ln2 and thereby violate the struct's documented invariant |r| <= ln2/2 (r could reach ~ln2). The spec correctly notes exp value-correctness is unaffected (any k works), but an atom that relies on the stated |r| <= ln2/2 bound (e.g. to pick a fixed Taylor term count or a series-convergence guard) would be operating on a false precondition.
  - fix: Either compute k via the BigFloat round_to_i64 reference (guarantees |r| <= ln2/2) or relax the documented invariant to |r| <= ln2 and size the exp series for that.

**Algorithm:**


# Argument-reduction kernel — implementation spec

This kernel sits between the `BigFloat<L>` core (sibling design) and the transcendental atoms (exp/log/sin, later cos/tan/etc.). It owns the constant tables and the range-reduction that turns an arbitrary argument into a small reduced argument plus an integer selector, with a bounded error the atom folds into its Ziv interval.

## 0. Dependency: the BigFloat<L> core contract I build on

Design is written against this minimal interface (the sibling core must expose it; see `depends_on`). `L` is the limb count: 4 = 256-bit, 8 = 512, 16 = 1024. `MAX_L = 16`.

Normalization convention (fixed here, must match the core):
- `struct BigFloat<const L:usize> { sign: i8 /* +1|-1, +1 for zero */, exp: i32, limbs: [u64; L] }`
- big-endian limbs: `limbs[0]` holds the most-significant 64 bits; for nonzero values bit 63 of `limbs[0]` is 1 (normalized).
- **value = sign · SIG · 2^(exp − (64L − 1))**, where `SIG = Σ limbs interpreted as a 64L-bit unsigned integer ∈ [2^(64L−1), 2^(64L))`.
  Equivalently: `exp` is the unbiased binary exponent of the value's leading (MSB) bit. Examples: `1.0` → exp=0; `2/pi≈0.6366` → exp=−1; `pi/2≈1.5708` → exp=0.

Required core ops (all take/produce `BigFloat<L>` unless noted): `from_f64(f64)` (exact), `from_u64`, `neg`, `abs`, `add`, `sub`, `mul`, `mul_small_int(i64)` (exact multiply by a ≤64-bit integer), `div`, `cmp`, `round_to_i64()` (nearest integer, ties-to-even, of a value known to be < 2^62), and a raw constructor `from_limbs_exp(sign, exp, [u64;L])`.

## 1. Constant tables (module `hp::consts`)

All constants are stored ONCE at `MAX_L` width (the 1024-level) as raw big-endian `[u64;N]` literals. A working-precision-`L` computation uses the top `L` limbs (the extra low limbs are the guard the escalation consumes). No constant is ever recomputed at runtime.

### 1.1 Storage forms

| Constant | Form | Words (N) | Bits | exp | Anchor (word 0, and word 1) |
|---|---|---|---|---|---|
| `TWO_OVER_PI` | raw fractional bit-string of 2/π (p₁ = 2^−1 bit is MSB of word 0) | **36** | 2304 | n/a (bit table) | `0xA2F9836E4E441529`, `0xFC2757D1F534DDC0`, (`0xDB6295993C439041`, …) |
| `LN2` | BigFloat significand of ln2 (value < 1) | **18** | 1152 | −1 | `0xB17217F7D1CF79AB`, `0xC9E3B39803F2F6AF` |
| `INV_LN2` | BigFloat significand of 1/ln2 | 18 | 1152 | 0 | `0xB8AA3B295C17F0BC`, … |
| `PI_OVER_2` | BigFloat significand of π/2 | 18 | 1152 | 0 | `0xC90FDAA22168C234`, `0xC4C6628B80DC1CD1` |

`TWO_OVER_PI` is a **pure bit table** (not a BigFloat): `p_i` (the 2^−i coefficient of 2/π) is bit `63 − ((i−1) mod 64)` of `word[(i−1)/64]`. The other three are BigFloat significands with the fixed `exp` shown.

### 1.2 Why these exact sizes (sizing formulas — this OVERRIDES the doc's "~1280-bit / 256-bit" figures)

Let `p` = significand width of the input dtype (f64:53, f32:24), `T` = current working precision in bits (= 64·L), `T_max = 1024`.

- **2/π window per reduction** spans bit indices `i ∈ [i0, i1]` with `i0 = max(1, E−2)`, `i1 = E + F + p`, where `F = T + G_c` and `G_c = 128` (cancellation + rounding guard; the documented f64 trig worst-case cancellation is ~61–67 bits, so 128 is a safe ceiling). For `E_max = 971` (largest finite f64, M~2^52 ⇒ E=1024−53) and `T=T_max`: `i1_max = 971 + (1024+128) + 53 = 2176`. Table must cover indices `1..2176` ⇒ ≥ 34 words; **36 words (2304 bits)** gives 128 bits of margin. The doc's 1280 bits is sized for `T=256` only and is even then ~130 bits short of `i1` at large E — it must not ship.
- **ln2** must supply `r = x − k·ln2` to `T` good bits after the ~11-bit leading cancellation, with `|k| ≤ 2^11`. Need `LN2_BITS ≥ T_max + 11 + guard = 1024 + 11 + 64 ≈ 1099` ⇒ **18 words (1152 bits)**. The doc's "256-bit ln2" is a 256-level figure only.
- **PI_OVER_2 / INV_LN2** are multiplied against `F`-bit / working-precision values; `T_max + 128` bits ⇒ 18 words.

### 1.3 Dev-time generation (never shipped) — `tools/gen_constants.py`

Emit the literals by computing each constant to `≥ 2400` bits in **three mutually independent ways** and requiring bit-for-bit agreement (dropping the low 32 guard bits to avoid last-bit disputes):
1. **mpmath**: `mp.prec = 2500; mp.mpf(2)/mp.pi`, `mp.log(2)`, `mp.pi/2`, `1/mp.log(2)`.
2. **MPFR via gmpy2**: `gmpy2.const_pi(2500)`, `gmpy2.log(mpfr(2))` — an algorithmically independent codebase.
3. **Pure-Python big-integer, library-free** (the true independence leg; uses only `int`):
   - π by **Machin**: `π·2^N = 16·atan(1/5)·2^N − 4·atan(1/239)·2^N`, each `atan(1/x)·2^N = Σ_{j≥0} (−1)^j · 2^N/((2j+1)·x^(2j+1))` with Python ints; then `2/π = (2·2^N·2^N)//π_scaled`, `π/2 = π_scaled//2`.
   - `ln2 = 2·atanh(1/3)·2^N = 2·Σ_{j≥0} 2^N/((2j+1)·3^(2j+1))`; `1/ln2 = (2^N·2^N)//ln2_scaled`.
   Extract the top `N` bits of each into the `[u64;N]` literals.
   Record engine versions (mpmath/gmpy2/GMP/MPFR) in a provenance header, per §8's reproducibility rule.

### 1.4 In-crate verification (ships; runs under `cargo test`, zero external deps)

The shipped crate re-derives its own constants by a **different algorithm** than whatever `gen_constants.py` used, so a transcription typo cannot survive:
- `test_pi_machin_selfcheck`: recompute π via the same Machin formula but in `BigFloat<MAX_L>` arithmetic; derive `PI_OVER_2` and (by `BigFloat::div(2, π)`) `TWO_OVER_PI`; assert the top `64·MAX_L − 64` bits equal the literals. This regenerates **both** the π/2 and 2/π tables in-crate.
- `test_ln2_atanh_selfcheck`: recompute `ln2 = 2·atanh(1/3)` and `1/ln2 = BigFloat::div(1, ln2)` in `BigFloat<MAX_L>`; compare to literals (top bits).
- `test_reciprocal_identities`: `PI_OVER_2 · (TWO_OVER_PI as BigFloat) == 1` and `LN2 · INV_LN2 == 1` within `2^−(64·MAX_L−16)` — ties the cross-derived tables together and catches a swapped/duplicated limb.
- `test_constant_anchors`: assert the published leading words verbatim: `LN2[0]==0xB17217F7D1CF79AB`, `TWO_OVER_PI[0]==0xA2F9836E4E441529`, `PI_OVER_2.sig[0]==0xC90FDAA22168C234`, `INV_LN2.sig[0]==0xB8AA3B295C17F0BC`. These are pinned facts (fdlibm `ipio2` / standard libm high-words), independent of both the generator and the in-crate recomputation — the constant analogue of the Lefèvre–Muller anchors.

## 2. Payne–Hanek trig reduction — `fn reduce_trig<const L>(x: f64, dtype_p: u32) -> TrigReduced<L>`

Returns:
```
struct TrigReduced<const L:usize> { octant: u32 /*k mod 8*/, r: BigFloat<L> /*∈[-pi/4,pi/4]*/, r_err_ubits: i32 /*|err_r| ≤ 2^r_err_ubits, absolute*/ }
```
The sin/cos/tan wrapper picks the identity from `octant` (sin/cos use `octant mod 4`, tan uses `octant mod 2`).

### 2.1 Preconditions & trivial case
- `x` finite, `|x| ≥ π/4` (caller returns `r=x, octant=0` for `|x| < π/4`; no reduction). NaN/±inf are special-valued in `semantics.rs`, never reach here.
- Reduce `|x|` only; the caller applies odd/even symmetry for `x<0` (sin odd, cos even, tan odd). So assume `x>0` here.

### 2.2 Decompose
`x = M · 2^E` with `M` the integer significand (`2^(p−1) ≤ M < 2^p`, `p = dtype_p`), `E` its integer exponent. (For a normal f64, `M = mantissa | 2^52`, `E = unbiased_exp − 52`.) Subnormal inputs have `|x|<π/4` so never reach PH; still, compute `M,E` via `frexp`-style bit surgery for totality.

### 2.3 Window + the mandatory table-length guard
```
F  = 64*L + 128;                 // fractional bits of x·(2/π) to retain
i0 = max(1, E - 2);              // highest 2/π bit that still matters (bits i≤E-3 are ≡ multiples of 8 in x·2/π ⇒ irrelevant to octant AND r)
i1 = E + F + (p as i32);         // lowest 2/π bit that matters (beyond it → the bounded r-truncation tail)
assert!(i1 <= TWO_OVER_PI_BITS,  // == 36*64 = 2304.  HARD GUARD — see §5 failure mode.
        "PH 2/pi table too short: need bit {i1} for |x|=2^{E}, have {TWO_OVER_PI_BITS}");
```
Justification for dropping `i ≤ E−3`: a bit `p_i` contributes `M·2^(E−i)` to `x·2/π`; written mod `2^(F+3)` after the `2^F` scaling (§2.5), bits at product-position `≥ F+3` vanish, which corresponds exactly to `i ≤ E−3`. So only `[i0,i1]` is needed.

### 2.4 Gather the window as a big integer
`win = Σ_{i=i0}^{i1} p_i · 2^(i1−i)` — the contiguous bit-slice `p_{i0..i1}` read MSB-first as an `(i1−i0+1)`-bit unsigned big integer. Implement by copying/masking/shifting the `TWO_OVER_PI` words into a scratch `[u64; ceil((i1-i0+1)/64)]` (≤ ~20 limbs). Pure bit-plumbing, no arithmetic.

### 2.5 Multiply, align, split
```
prod   = bigmul(win, M);                 // big int, ≤ (i1-i0+1)+p bits; schoolbook u128 MAC (dependency-free)
scaled = prod >> (p as u32);             // == floor( x·(2/π) · 2^F ),  because E - i1 + F = -p by construction
J      = scaled & ((BIG_ONE << (F+3)) - 1);   // low (F+3) bits: 3 integer bits (mod 8) + F fractional bits
n_low3 = (J >> F) as u32;                 // integer part of x·2/π, mod 8
frac   = J & ((BIG_ONE << F) - 1);        // F fractional bits; f = frac · 2^-F  ∈ [0,1)
```
(`bigmul`/shift/mask operate on a small scratch big-int, ~24 u64 limbs max.)

### 2.6 Round to nearest integer of `x·2/π`; form octant and f'
```
half_set = bit (F-1) of frac is 1;        // f ≥ 1/2 ?
if half_set {
    octant     = (n_low3 + 1) & 7;
    fprime     = (BIG_ONE << F) - frac;    // |f'| = 1 - f  ∈ (0, 1/2]
    fprime_neg = true;                     // f' = f - 1 < 0
} else {
    octant     = n_low3 & 7;
    fprime     = frac;                     // f' = f      ∈ [0, 1/2)
    fprime_neg = false;
}
```
This reduces modulo π/2 into `[−π/4, π/4]` (half-width), so sin/cos need `octant mod 4`, tan needs `octant mod 2`.

### 2.7 Build r
`fprime` is an `F`-bit integer = `|f'|·2^F`, `|f'| ≤ 1/2` (top bit clear).
```
r = bigfloat_from_scaled_fraction::<L>(fprime, F);  // value = fprime · 2^-F  ∈ [0,1/2); normalize: MSB→bit(64L-1), set exp, keep top 64L bits (low bits discarded ⊂ the F-guard, error ≤ 2^-64L relative)
r = r.mul(&PI_OVER_2::<L>());                        // ∈ [0, π/4)
if fprime_neg { r = r.neg(); }                       // ∈ (-π/4, π/4]
```

### 2.8 Error bound (fed to the atom's Ziv interval)
Discarded 2/π tail (bits `i>i1`) perturbs `f` by `< 2^-F`, hence `r` by `< 2^-F·(π/2) < 2^-(F-1)` absolute.
```
r_err_ubits = -(F - 1) = -(64*L + 127);
```
Relative to the worst small `|r| ≥ 2^-67` this is `< 2^-(64L+60)` — comfortably below the working precision, so the reduction is never the limiting error. If the atom's Ziv loop still cannot resolve at `L=16`, that is the "red flag to investigate" case in the design's risk table (do NOT silently return).

## 3. exp reduction — `fn reduce_exp<const L>(x: f64) -> ExpReduced<L>`
```
struct ExpReduced<const L:usize> { k: i64, r: BigFloat<L> /* |r| ≤ ln2/2 */, r_err_ubits: i32 }
```
- Only invoked when the atom has decided `exp(x)` is finite & normal (over/underflow, NaN, ±inf handled in `semantics.rs` with the `overflow` tag). So `|x| ≤ ~745`, `|k| ≤ 2^11`.
- `xb = BigFloat::<L>::from_f64(x)` (exact).
- `k = xb.mul(&INV_LN2::<L>()).round_to_i64()`. **Exactness of k is not required for correctness** — any `k` yields `exp(x)=2^k·exp(r)` with `r=x−k·ln2`; a mis-rounded `k` only slightly enlarges `|r|`, still fine for the series. So an f64 `(x/LN2).round()` is an acceptable fast path with the BigFloat form as the reference.
- `r = xb.sub(&LN2::<L>().mul_small_int(k))`. Because `x` and `k·ln2` are both exact-in-BigFloat except for ln2's truncation, the ONLY error is the discarded ln2 tail: `|err_r| = |k|·(ln2 − LN2_stored) < 2^11 · 2^-LN2_BITS`.
- `r_err_ubits = 11 - LN2_BITS = 11 - 1152 = -1141`.
- `2^k` is applied by the atom as an exact `exp` shift on the final BigFloat before rounding.

## 4. log reduction — `fn reduce_log<const L>(x: f64) -> LogReduced<L>`
```
struct LogReduced<const L:usize> { e: i64, m: BigFloat<L> /* ∈ [√2/2, √2) */, r_err_ubits: i32 /* = 11 - LN2_BITS */ }
```
- Domain (`x ≤ 0`, NaN, ±inf, `log(1)=+0`, `log(+0)=−inf`) handled in `semantics.rs`; here `x > 0` finite.
- Split exponent **exactly** (no rounding): `frexp(x) → (m0, e0)`, `x = m0·2^e0`, `m0 ∈ [0.5,1)`.
- Center around 1 to bound the series argument: `if m0 < SQRT1_2 { m = m0·2 (exact); e = e0−1 } else { m = m0; e = e0 }` ⇒ `m ∈ [√2/2, √2)`, `|m−1| < 0.4143`.
- `m = BigFloat::<L>::from_f64(m)` (exact). The atom computes `log(x) = e·ln2 + log(m)`, forming `e·ln2 = LN2::<L>().mul_small_int(e)` (`|e| ≤ 1074 < 2^11`) and `log(m)` via `2·atanh((m−1)/(m+1))`.
- The reduction itself introduces **no** error in `m` (frexp is exact); the only reduction error is in `e·ln2`, bounded `|e|·2^-LN2_BITS < 2^(11−LN2_BITS)`, hence `r_err_ubits = 11 − LN2_BITS`. This bound is the atom's to fold when it adds `e·ln2`.

## 5. Failure mode of a too-short table (why the §2.3 / §3 asserts are load-bearing)

A too-short constant table produces a **systematic, precision-independent** error, which is the one class of error the Ziv escalation is *blind to*:

- **Short 2/π**: for large `|x|`, the needed window `[i0,i1]` runs past the stored bits. If padded with zeros/garbage, `r`'s low bits are wrong. For a benign `x` the error may stay below an f64 ULP and pass casual tests; but for a hard-to-round large argument, or one where `x` sits near a multiple of π/2 (cancellation exposes exactly those deep bits), `r` loses all significance and sin/cos/tan is **~100% wrong, not 1 ULP**. Critically: the Ziv loop re-runs the reduction at 512 then 1024 **with the same truncated table**, gets the **same wrong `r`**, sees the two precisions "agree", and **certifies a confidently-wrong value with a healthy-looking hardness margin**. Escalation cannot rescue a reduction-table defect.
- **Short ln2**: identical mechanism — for large `|k|`/`|e|`, `r = x − k·ln2` (or `e·ln2`) carries `|k|·(tail)` error that is the same at every working precision, so Ziv certifies the wrong exp/log.

Mitigations, all mandatory:
1. Size tables by the §1.2 formulas for `T_max=1024` (2/π: 36 words; ln2/pi2/invln2: 18 words) — a proof, not a tuned guess.
2. A runtime `assert!(i1 <= TWO_OVER_PI_BITS)` on every trig reduction and an equivalent width check for exp/log; a table that is ever too short **panics** rather than certifies.
3. The §1.4 in-crate recomputation self-tests prove the committed literals are the true bits at ship time.

## 6. Test / self-check discipline summary
- Constants: §1.4 four tests (Machin self-check, atanh self-check, reciprocal identities, published-hex anchors).
- Reduction: the atom-level vectors in `test_vectors` pin end-to-end correctness; add reduction-only unit assertions — e.g. for `x = f64_nearest(π)`, assert `octant==2` and `|r + 1.2246467991473532e-16| < 2^-100`; for `x = 1e22`, assert the reduction succeeds (window fits) and reproduces the validator's `sin` bits.


**Test vectors:**
- `exp(1.0) f64` -> `0x4005BF0A8B145769 (2.718281828459045)`  e; exp reduction k=1, r=1-ln2~=0.3069. High-confidence anchor.
- `exp(1.0) f32` -> `0x402DF854 (2.7182817)`  matches the design-doc example (40 2D F8 54).
- `exp(0.0) f64` -> `0x3FF0000000000000 (1.0)`  reduction k=0, r=0 edge.
- `ln(2.0) f64` -> `0x3FE62E42FEFA39EF (0.6931471805599453)`  log reduction e=1, m=1 -> log(m)=0, result = 1*ln2. High-confidence anchor.
- `ln(1.0) f64` -> `0x0000000000000000 (+0.0)`  e=0, m=1, log(m)=0; signed-zero: log(1)=+0 (set by semantics.rs).
- `sin(1.0) f64` -> `0x3FEAED548F090CEE (0.8414709848078965)`  |x|<pi/4? No, 1.0>pi/4 -> PH: x*2/pi~=0.6366, octant=0, r=1.0 (r=x since 1<pi/2 gives octant 0, sin=sin(r)). High-confidence anchor.
- `sin( f64_nearest(pi) = 0x400921FB54442D18 ) f64` -> `0x3CA1A62633145C07 (1.2246467991473532e-16)`  STRONG reduction test: octant=2 (sin=-sin(r)), r from cancellation ~= -1.2246e-16 needs PI_OVER_2 accurate; classic sin(M_PI) value. Verify against 3-source validator.
- `sin( f64_nearest(pi/2) = 0x3FF921FB54442D18 ) f64` -> `0x3FF0000000000000 (1.0)`  octant=1 (sin=cos(r)), r~=6.1e-17, cos(r) rounds to exactly 1.0. Exercises octant-1 path and PI_OVER_2.
- `sin(0.0) f64` -> `0x0000000000000000 (+0.0)`  octant=0, r=0; signed zero preserved.
- `sin(1e22) f64  [1e22 is exactly representable = 5^22 * 2^22, exp field 0x448]` -> `~= -0.8522008497671888 (exact bits validator-pinned)`  THE canonical Payne-Hanek large-arg regression (historic glibc misround). Proves the window reaches deep fractional 2/pi bits. Expected bits come from the 3-source validator, not asserted from memory.
- `sin(0x1p+1023) f64  [2^1023]` -> `validator-pinned`  pure large-|x| stressor: window i1~=969 near the top of the table; confirms table length + guard. Expected from validator.

**Constants required:** TWO_OVER_PI: [u64;36] — 2304 fractional bits of 2/pi, big-endian (p_1 = 2^-1 is MSB of word 0). Anchors: word0=0xA2F9836E4E441529, word1=0xFC2757D1F534DDC0, word2=0xDB6295993C439041 (fdlibm ipio2); LN2: BigFloat significand of ln2, [u64;18] (1152 bits), exp=-1. Anchors: word0=0xB17217F7D1CF79AB, word1=0xC9E3B39803F2F6AF; INV_LN2: BigFloat significand of 1/ln2, [u64;18], exp=0. Anchor: word0=0xB8AA3B295C17F0BC; PI_OVER_2: BigFloat significand of pi/2, [u64;18], exp=0. Anchors: word0=0xC90FDAA22168C234, word1=0xC4C6628B80DC1CD1; Derived-at-runtime-free scalars: G_c=128 (trig cancellation+rounding guard), TWO_OVER_PI_BITS=2304, LN2_BITS=1152, MAX_L=16; SQRT1_2 = 0x1.6A09E667F3BCDp-1 (f64) for log's [sqrt2/2, sqrt2) centering split
**Edge cases:** |x| < pi/4 for trig: no reduction, return r=x, octant=0 (caller handles; PH never invoked); Large-|x| trig up to 2^1023: window i1 reaches ~2176; the 36-word table must cover it — assert guards it, else panic (never certify); x near k*pi/2 (sin near k*pi, tan near odd*pi/2): catastrophic cancellation eats up to ~61-67 bits of r; G_c=128 guard + full window preserve relative precision; Octant boundary f == exactly 1/2: ties-to-even is irrelevant to k here — we round x*2/pi to the nearest integer via the top fractional bit; f==1/2 exactly cannot occur for a finite float x (x*2/pi is irrational unless x=0), but the bit-test handles it deterministically; x < 0: reduce |x|; caller applies sin odd / cos even / tan odd symmetry (do NOT re-run PH on negative x); exp overflow/underflow (|x| beyond ~709/-745 f64, ~88/-104 f32): handled as special values in semantics.rs with 'overflow' tag; reduce_exp only sees finite-normal-result inputs so |k| <= 2^11; exp(0)=1, sin(0)=+0, log(1)=+0: reductions degenerate cleanly (k=0/r=0, octant=0/r=0, e=0/m=1); signed-zero of the RESULT is set by semantics.rs, not the reduction; log domain x<=0 / NaN / inf: handled upstream in semantics.rs; reduce_log only sees x>0 finite; k mis-rounding in exp: acceptable — any k gives exp(x)=2^k*exp(r); only |r| grows slightly, no correctness loss; f32 inputs (p=24): window far smaller (E<=104), table amply covers; same code path with dtype_p=24; subnormal input: |x|<pi/4 for trig (never reaches PH); for exp/log frexp handles it exactly; Ziv unresolved at L=16 (1024-bit): a red flag to investigate per the risk table — surface it, never silently return a value
**Open questions:** G_c guard width: I pinned 128 bits (covers the documented ~61-67 bit f64 trig cancellation with margin). Confirm no representable f64 argument drives r below 2^-96 relative — if a curated stressor does, bump G_c and regrow TWO_OVER_PI to match (formula given).; round_to_i64 exact-tie behavior for k in exp: I argued k need not be exactly rounded (any k works). Confirm the sibling core's round_to_i64 is total and cannot panic on x/ln2 up to ~1075 — else clamp.; Should PI/PI_OVER_4 also be stored, or is octant-selection + PI_OVER_2 sufficient? Current design needs only PI_OVER_2 (r is built from f'*pi/2). cos/tan atoms in Slice 1 may want PI for reflection — defer to their per-atom designs.; gen_constants.py placement: standalone dev-only script vs folded into validate_corpus.py. Leaning standalone (constants aren't corpus cells), but both must share the version-pinned provenance header.; TWO_OVER_PI margin: 36 words gives i1_max=2176 vs table 2304 (128-bit margin). If a future dtype wider than f64 (p>53) is ever oracled, i1 grows by (p-53) and the table must grow — flag at the Slice-2 dtype expansion.

---

## exp-log — exp and log atom evaluation at 256-bit  (difficulty: high)
**Summary:** Range-reduced, series-based exp and log evaluated in the hand-written HpFloat big-float (256→512→1024 bits) with a bounded, folded truncation error and a Ziv rounding test that rounds once to f32/f64 round-ties-to-even. exp reduces x=k·ln2+r (|r|≤ln2/2) then Taylor-sums exp(r) and rescales by 2^k via exact ldexp; log reduces x=m·2^e into an octave centered on √2 (|s|≤0.172) and atanh-sums log(m), then adds e·ln2. Both carry a per-evaluation absolute-error certificate (hardness margin + stabilizing width). For the whole exp/log input family the 256-bit width resolves every rounding (effective accuracy ~2^-245, worst hard-to-round runs ~67 bits), so 512/1024 escalation is an airtight safety net rather than a routine path — unlike pow/tan in later slices.

**Review verdict:** needs-fixes (constant_width_adequate=True)
_The numerical core is fundamentally sound: I re-derived the exp reduction error and confirmed r = x - k*ln2 is Sterbenz-exact (both operands within a factor of 2), so r's only error is the k*ln2 truncation ~k*2^-W ~2^-(W-11), giving relative error ~2^-245 in exp(r) at W=256 — far below the ~2^-126 worst-case hard-to-round margin, so the "always resolves at 256, escalation is a safety net" claim holds. The err_ulp derivation (2^((p-1)-err_bits), err_bits~W-12) is correct and conservative in every regime I checked, including subnormals (fixed 2^-1074 ULP makes the formula an over-estimate, i.e. safe) and the overflow midpoint. The atanh/Taylor series have condition number ~1 (no cancellation amplification), and the near-1 log1p regime genuinely avoids cancellation because m-1 is exact. All twelve pinned test-vector bit patterns check out (exp(1)=0x4005BF0A8B145769, log(2)=0x3FE62E42FEFA39EF, log(1)=+0.0, the f32 single-rounds, etc.). BUT two real defects block "sound": (1) the core routes ALL finite x through k=round(x*INV_LN2) as i64 and ldexp(k as i32), and the spec explicitly refuses to special-case overflow — for any finite x beyond ~6.4e18 (up to f64 max 1.8e308) k overflows i64 (and the i32 cast is worse), so exp(1e19) or exp(1e300) is an integer-overflow bug, not a clean +inf; (2) log's s=(m-1)/(m+1) needs a general big-float division, but the §0 core contract only supplies div_small (divide by a small integer), so log is not buildable from the stated primitives. Both are fixable without touching the design's core, hence needs-fixes rather than flawed._

**Review findings:**
- (critical) exp routes ALL finite x through the series with k = round_to_int(x*INV_LN2) as i64 and rescale ldexp(k as i32); the spec explicitly states overflow 'is NOT a special case — the core computes it.' For any finite x beyond ~6.4e18 (i64::MAX*ln2), k overflows i64; the k-as-i32 cast overflows far sooner. Concrete failure: exp(f64, x=1e19 / 0x43E158E460913D00) or exp(x=1e300 / 0x7E37E43C8800759C) yields k≈1.4e19 / 1.4e300, an integer overflow (panic or garbage), where the correct answer is a trivial +inf. Even x in [709.79, 6.4e18] wastefully builds a 2^k with astronomically large exponent before mapping to inf. Dispatch only guards ±inf/NaN/±0, not large-finite overflow/underflow.
  - fix: Pre-clamp in the semantics.rs dispatch (or at the top of exp_hp) BEFORE reduction: if x >= ~709.7827 return +inf, if x <= ~-745.1332 return +0/subnormal band start, routing only the finite non-overflow band into the k-reduction. Retain the genuine overflow-midpoint hard case (x≈709.78 crossing 2^1024-2^970) inside the core where k~1024 fits. Reword the false claim that overflow needs no special case.
- (important) log step 3c computes s = m.sub(&ONE).div(&m.add(&ONE)) — a full big-float division of two ~W-bit values (m+1 is not a small integer). The §0 'Required primitives' contract lists only div_small(&self, n: u64) (divide by a small integer). No general HpFloat division / reciprocal (e.g. Newton–Raphson) is specified. An implementer following §0 literally cannot construct log; the load-bearing division primitive and its own rounding-error contribution to err_bits are unaccounted for.
  - fix: Add a general div(&self,&HpFloat)->HpFloat (Newton–Raphson reciprocal at width W, round-to-nearest) to the §0 core contract, or restructure the log series to avoid a full division (e.g. carry the reduction as m and use a reciprocal computed once). Fold its <=1 ULP-at-W rounding into the log err_bits budget (still dominated by e*ln2, so no numeric impact, but it must be listed).
- (minor) The truncation-bound prose states the exp term ratio is 'ratio |r|/n <= |r| < 0.5', but the design explicitly permits an off-by-one k ('robustness, not a pin') which enlarges |r| up to ~1.04 (1.5*ln2), violating the stated |r|<0.5 invariant. The folded tail bound 2*|t_N| is still valid because at termination N is large and r/(N+1)<<1, but the written invariant is false as stated and could mislead a static-analysis reviewer or a fixed-term-count implementer.
  - fix: State the invariant as |r| <= 1.5*ln2 ≈ 1.04 (off-by-one branch) and justify 2*|t_N| from the termination-point ratio r/(N+1)<<1 rather than from |r|<0.5. If a fixed term count is chosen instead of dynamic termination, size N for |r|=1.04, not 0.347.
- (minor) §5 claims 'the worst real hard-to-round run is ~67 bits' as the basis for 'always returns at W=256,' but the design doc's own §1/§5.122 cites hardest binary64 margins at 2^-113…2^-126. The conclusion survives (err_ulp≈2^-192 < 2^-126, still decided at 256), so this is not a misround risk — but the certificate's hardness_margin_bits and the 'never escalates' claim should not be pinned to the optimistic 67-bit figure.
  - fix: Reconcile the figure to the doc's 2^-113…2^-126 worst-case and note the 256-bit margin (resolvable to ~2^-192) still dominates it, so escalation remains a genuine never-taken safety net for exp/log.

**Algorithm:**

# exp / log at ≥256 bits — implementation-ready design

This kernel adds two transcendental atoms to `conformance/src/hp.rs`. It assumes the
`HpFloat` big-float core and the shared Ziv/rounding driver (designed in sibling
Slice-0 kernels); this spec pins the **reduction, series, truncation-error bound,
and error budget** for `exp` and `log`, at both f32 and f64 output.

## 0. Contract with the core (dependencies this kernel calls)

`HpFloat<W>` is a binary big-float: `[u64; W/64]` significand (W ∈ {256,512,1024}) +
`i32` exponent + `sign`. Required primitives (all dependency-free, from-scratch, in
the `fp.rs` precedent style):

```
HpFloat::from_f64(x: f64) -> HpFloat        // EXACT (f64 significand ⊂ 256 bits)
add, sub, mul(&self,&other) -> HpFloat       // round-to-nearest at width W
div_small(&self, n: u64) -> HpFloat          // divide by a small integer (series)
mul_small(&self, n: i64) / from_i64(k)       // integer → HpFloat, exact
ldexp(&self, k: i32) -> HpFloat              // multiply by 2^k: EXACT (adds to exp field)
cmp(&self,&other), is_zero, sign, exp2()->i32 (binary exponent of the value)
// shared rounding/Ziv driver (separate kernel), parametrised by target p∈{24,53}:
to_target_round(v:&HpFloat, p, emax, emin) -> (bits, residual_g:HpFloat, exp_res:i32)
//   residual_g = exact trailing value below the target ULP, in [0,1) ULP units
//   handles overflow→±inf, underflow→+0/subnormal per the target format
CONST_at(width) // read a stored 1024-bit constant truncated to the working width W
```

Constants are stored **once at 1024 bits** and truncated to the working width, so a
512/1024 escalation re-reduces against a wider constant automatically.

## 1. Dispatch / special-value split (in `semantics.rs`, before the core)

The core only ever sees a **finite, positive** argument for `log`, and a finite
argument for `exp`. Non-finite and domain results are table logic, NOT series:

```
exp(NaN)=NaN   exp(+inf)=+inf   exp(-inf)=+0   exp(±0)=1.0 (positive)
log(NaN)=NaN   log(+inf)=+inf   log(x<0)=NaN   log(±0)=-inf   log(1)=+0 (positive)
```

`exp` overflow/underflow to a *finite* x (e.g. exp(1000)=+inf, exp(-1000)=+0) is
NOT a special case — the core computes it and the rounding stage maps it to
±inf/+0/subnormal. `log(1)` yields an exactly-zero `HpFloat`; the rounding stage
MUST emit **+0.0** (positive), never −0.0.

## 2. exp(x) — reduction + series

```
fn exp_hp(x: f64, W) -> Eval {
    let xh = HpFloat::from_f64(x);                    // exact
    // 2a. k = nearest integer to x/ln2  (INV_LN2 = 1/ln2 = log2 e)
    let k: i64 = round_to_int( xh.mul(&INV_LN2.at(W)) );
    // 2b. reduced argument  r = x - k*ln2,  |r| ≲ ln2/2 = 0.3466
    let r = xh.sub( &HpFloat::from_i64(k).mul(&LN2.at(W)) );
    // 2c. Taylor: exp(r) = Σ r^n/n!  — terms generated iteratively
    let mut sum  = HpFloat::one();
    let mut term = HpFloat::one();
    let mut n = 1u64;
    loop {
        term = term.mul(&r).div_small(n);             // term = r^n / n!
        sum  = sum.add(&term);
        if term.exp2() <= -( (W as i32) + GUARD ) { break; }   // |term| ≤ 2^-(W+G)
        n += 1;
    }
    let trunc_bound = 2 * |term|;                     // geometric tail, folded
    // 2d. rescale by 2^k  (EXACT)
    let val = sum.ldexp(k as i32);
    Eval{ val, err: combine_exp(x, W, n, trunc_bound) }
}
```

**Term count** (dynamic, self-adapting to W): |r|≤0.347 ⇒ N≈46 at 256b, ≈80 at
512b, ≈142 at 1024b. No factorial table — each term is `prev·r/n`.

**Why the reduction is safe.** k fits in i64; `k·ln2` uses the 1024-bit LN2 so its
only error is one W-bit rounding. The subtract `x − k·ln2` loses ≤11 leading bits to
cancellation (|x|≤~1024 vs |r|≤0.35), leaving r accurate to ~2^-(W-11) absolute.
Because exp(r+δ)=exp(r)(1+δ+…), that maps to **relative** error ≈2^-(W-11)≈2^-245 in
exp(r). k need **not** be the exact nearest integer — an off-by-one only enlarges |r|
to ≤0.72 and adds a few terms; correctness is unaffected (robustness, not a pin).

## 3. log(x) — octave reduction + atanh series

```
fn log_hp(x: f64, W) -> Eval {
    let xh = HpFloat::from_f64(x);                    // exact, x>0, x≠1
    // 3a. x = m·2^e,  m∈[1,2)   (normalise subnormals: shift, adjust e)
    let (mut m, mut e) = frexp_octave(xh);            // m∈[1,2), e = binary exponent
    // 3b. re-centre on √2 so |log m| ≤ ln(√2)=0.3466 and s is small/symmetric
    if m.cmp(&SQRT2.at(W)) == Greater { m = m.ldexp(-1); e += 1; }   // m∈[√2/2, √2)
    // 3c. s = (m-1)/(m+1),  |s| ≤ (√2-1)/(√2+1) = 0.1716
    let s  = m.sub(&ONE).div( &m.add(&ONE) );
    let s2 = s.mul(&s);
    // 3d. atanh series: log(m) = 2 Σ_{j≥0} s^(2j+1)/(2j+1)
    let mut termpow = s.clone();                      // s^(2j+1)
    let mut acc     = s.clone();                      // j=0 term
    let mut j = 1u64;
    loop {
        termpow = termpow.mul(&s2);                   // s^(2j+1)
        let t = termpow.div_small(2*j + 1);
        acc = acc.add(&t);
        if t.exp2() <= -((W as i32) + GUARD) { break; }
        j += 1;
    }
    let logm = acc.ldexp(1);                          // ×2
    let trunc_bound = |t_last| / (1 - s2)  (≤ |t_last|·1.03);   // folded
    // 3e. log(x) = e·ln2 + log(m)
    let val = HpFloat::from_i64(e).mul(&LN2.at(W)).add(&logm);
    Eval{ val, err: combine_log(x, e, W, j, trunc_bound) }
}
```

**Term count:** |s|≤0.1716 ⇒ 2j+1≈101 (≈50 terms) at 256b, ≈100 terms at 512b,
≈202 at 1024b.

**log1p regime is automatic.** For x near 1, e=0 and s=(m−1)/(m+1) is computed from
the **exact** HpFloat m, so there is no catastrophic cancellation: log(x)≈(x−1) is
delivered to full **relative** precision. `log(1)` gives s=0 ⇒ acc=0 ⇒ val=exact
zero ⇒ +0.0. The √2 fold threshold need not be exact (both branches yield the same
true log; √2 only minimises |s|), so SQRT2 precision is non-critical.

## 4. Bounded truncation error — folded into the Ziv interval

Rounding accuracy alone is not enough; the **series truncation** is a real error the
Ziv test cannot see unless it is bounded and added to the interval:

- **exp:** terms decay geometrically (ratio |r|/n ≤ |r| < 0.5); tail after the last
  kept term `t_N` is `< |t_N|·1/(1−|r|/(N+2))`, bounded by `2·|t_N| ≤ 2^-(W+G-1)`.
- **log:** tail `< |t_last|·1/(1−s²) ≤ 1.03·|t_last| ≤ 2^-(W+G)·1.03`.

Both are ≪ the reduction error (2^-(W-11)) and are added into `E_abs`.

**Per-evaluation absolute error bound `E_abs`** (tracked, conservative):

| source                    | exp                          | log                              |
|---------------------------|------------------------------|----------------------------------|
| reduction (k·ln2 / e·ln2) | ~2^(exp2(val) − (W−11))      | ~2^-(W−11) absolute (e·ln2)      |
| series rounding (~N ops)  | ~N·2^-W (relative)           | ~2j·2^-W (relative)              |
| truncation (folded)       | ≤2^-(W+G-1)                  | ≤1.03·2^-(W+G)                   |
| ldexp / octave shift      | exact (0)                    | exact (0)                        |

`E_abs` is carried as `err_bits`: `E_abs = 2^(exp2(val) − err_bits)`, with
`err_bits ≈ W − 12` (dominated by reduction). Express the interval in target-ULP
units: `err_ulp = 2^((p−1) − err_bits)` where p = target significand bits (24 or 53).
For W=256: `err_ulp ≈ 2^(52 − 244) = 2^-192` (f64), `2^-220` (f32).

## 5. Precision management — the Ziv rounding test (shared driver, pinned here)

```
fn round_ziv(eval_fn, x, target /*p,emax,emin*/) -> (bits, Certificate) {
    for W in [256, 512, 1024] {
        let ev = eval_fn(x, W);
        let (bits, g, _) = to_target_round(&ev.val, target);   // g∈[0,1) ULP
        let err_ulp = ev.err_ulp(target);
        let dist = |g − 0.5|;                                   // distance to midpoint
        if dist > err_ulp {                                     // rounding is DECIDED
            return (bits, Certificate{
                hardness_margin_bits:      floor(-log2(dist)),
                stabilized_precision_bits: W,                   // ∈{256,512,1024}
            });
        }
        // interval straddles an f64/f32 midpoint → recompute wider (re-reduce too)
    }
    panic!("exp/log unresolved at 1024 bits for input {x:#x} — INVESTIGATE");
}
```

- **`stabilized_precision_bits` is always ≥256 > 53 ≥ dtype** ⇒ satisfies the
  §6.5-0009 test's *strictly greater than compute-dtype width* requirement for every
  transcendental cell, at both f32 (24) and f64 (53).
- **`hardness_margin_bits`** = number of leading identical bits after the round bit
  (bits until g diverges from ½). Recorded at the stabilising width; this is the
  self-certifying hardness margin §6.5-0009 stores.
- For the exp/log family, `err_ulp≈2^-192` while the worst real hard-to-round run is
  ~67 bits, so the loop **always returns at W=256**. 512/1024 are the airtight
  safety net (a straddle there would be a red flag to investigate — see §6.13-0007
  escalation discipline), not a routine path. This is the key contrast with pow/tan
  (Slices 1) which routinely force escalation.

## 6. f32 vs f64

Identical algorithm; only `to_target_round` changes p=24/emax=127/emin=−126 (f32)
vs p=53/emax=1023/emin=−1022 (f64), and the caller narrows the input to the compute
dtype first (an f32 cell's input x is an exact f32, widened to HpFloat exactly). The
oracle **rounds once** from the ≥256-bit result to the compute dtype — never through
an intermediate f64 for f32 (the `pow`/`tanh_refined` double-rounding bug in
`semantics.rs` is exactly what this replaces for these atoms).

## 7. Integration points (noting; detailed in sibling kernels)

- **Minter** (`kiss_mint.rs`): add a `unary_transcendental` cell emitter that calls
  `round_ziv(exp_hp,…)` / `round_ziv(log_hp,…)`, emits `class:"ULP"`,
  `ulp_bound` = the §6.13 declared transcendental ceiling, `provenance:"oracle"`,
  `tags` = the atom's edge tags, and `certificate{hardness_margin_bits,
  stabilized_precision_bits}` from the driver.
- **Reader/differential** (`corpus_differential.rs`): current `eval_add` is binary
  f32 only; add unary dispatch on `op`∈{exp,log} and on `dtype`∈{f32,f64}, comparing
  under `compare_f32` (ULP) — and a f64 analogue `compare_f64`/`ulp_distance_f64`
  (new, mirrors `ulp_distance_f32` with u64/i64 totalOrder) since the corpus now
  holds f64 cells.
- **op_manifest**: add `exp`,`log` to `declared_coverage_set`; they are already in
  `transcendental_atoms`.
- **validate_corpus.py**: implement `TranscendentalLeg.value` for exp/log (mpmath +
  MPFR/gmpy2 + Lefèvre–Muller anchors), guarded on `provenance=="oracle"`, comparing
  the rounded target bytes. This replaces the exact-byte-only stub for these ops.

**Test vectors:**
- `exp, f64, x=0x3FF0000000000000 (1.0)` -> `0x4005BF0A8B145769`  exp(1)=e. Canonical accuracy anchor; g far from midpoint, stabilizes at 256.
- `exp, f32, x=0x3F800000 (1.0)` -> `0x402DF854`  exp(1)=e as f32 (matches design-doc §4 example line); single-round to f32, not via f64.
- `exp, f64, x=0x0000000000000000 (+0.0)` -> `0x3FF0000000000000`  exp(+0)=1.0 exactly (special-cased; sign positive).
- `exp, f64, x=0x408F400000000000 (1000.0)` -> `0x7FF0000000000000`  Overflow: exp(1000)≈2e434 → +inf. Core computes 2^k·exp(r); rounding stage maps to +inf. f32 analogue: x=0x447A0000 → 0x7F800000.
- `exp, f64, x=0xC08F400000000000 (-1000.0)` -> `0x0000000000000000`  Underflow deep tail: exp(-1000) → +0 (below smallest subnormal). Exercises to_target_round underflow path.
- `exp, f64, x=0xFFF0000000000000 (-inf)` -> `0x0000000000000000`  exp(-inf)=+0 (dispatch table). exp(+inf)=+inf=0x7FF0000000000000.
- `log, f64, x=0x3FF0000000000000 (1.0)` -> `0x0000000000000000`  log(1)=+0.0 POSITIVE zero (load-bearing sign pin). Exactly-zero HpFloat → +0.0. f32: 0x3F800000 → 0x00000000.
- `log, f64, x=0x4000000000000000 (2.0)` -> `0x3FE62E42FEFA39EF`  log(2)=ln2 (Math.LN2). x=2^1·1 ⇒ e=1,m=1,s=0 ⇒ result = 1·ln2 exactly — exercises the reduction constant. f32: x=0x40000000 → 0x3F317218.
- `log, f64, x=0x0000000000000000 (+0.0)` -> `0xFFF0000000000000`  log(±0)=-inf (dispatch). x=0x8000000000000000 (-0.0) → same -inf. x<0 → NaN; x=+inf → +inf.
- `log, f64, x=0x3FF0000000400000 (1 + 2^-30)` -> `oracle-emitted ≈ 9.3132257e-10 (≈ 2^-30·(1−2^-31)); VALIDATOR-PINNED`  log1p regime: e=0, s=(m-1)/(m+1) from exact m ⇒ no cancellation, full relative precision. Exact bits certified by mpmath+MPFR+Lefèvre–Muller, not hand-transcribed.
- `exp/log, f64, near-midpoint hard-to-round anchor (Lefèvre–Muller table)` -> `VALIDATOR-PINNED; certificate shows small hardness_margin_bits (~<70) and stabilized_precision_bits=256`  Structural test: minter emits the wide value; the 3-source gate certifies the rounded bits. Proves the Ziv margin/certificate machinery, resolved at 256 bits (escalation NOT triggered for exp/log).

**Constants required:** LN2 = ln(2) significand to 1024 bits, stored MSB-first as [u64;16]; truncated to the working width W. Used by exp (k·ln2) and log (e·ln2). Verified against a third source in validate_corpus.py.; INV_LN2 = 1/ln(2) = log2(e) to 1024 bits, [u64;16]. Used by exp to compute k = round(x·INV_LN2). Verified independently.; SQRT2 = sqrt(2) to 1024 bits, [u64;16] — the log octave-fold threshold (m>√2 ⇒ m/=2,e+=1). PRECISION NON-CRITICAL: only balances |s|; both branches yield the same true log. Can be a modest-precision constant or even an f64 compare.; GUARD: compile constant = 24 bits of series-tail/rounding headroom below the working width W (loop-termination threshold 2^-(W+GUARD)).; ONE / one() HpFloat literal for series accumulators (exact).
**Edge cases:** exp(±0) = 1.0 (positive) — handle as special in dispatch (or let core produce it; series gives exactly 1).; exp(+inf)=+inf, exp(-inf)=+0, exp(NaN)=NaN — dispatch table, never series.; exp overflow: finite x with exp(x) > max-finite (e.g. exp(1000), exp(710)) → core computes 2^k·exp(r), rounding stage maps to +inf. Includes the genuine hard case at the overflow midpoint (exp(x) vs 2^emax+ + ½ulp → inf vs max-finite) — decided by the 256-bit value against the 2^1024 overflow midpoint.; exp underflow / deep tail: very negative x (exp(-1000)) → +0 or subnormal via to_target_round; near the subnormal boundary the rounding stage must produce the correct subnormal, not flush.; log(1) = +0.0 (POSITIVE zero) — series yields an exactly-zero HpFloat; rounding stage MUST emit +0.0, never -0.0. Load-bearing signed-zero pin.; log(±0) = -inf, log(x<0)=NaN, log(+inf)=+inf, log(NaN)=NaN — dispatch table (domain boundary).; log near 1 (log1p regime): x = 1±2^-k. s=(m-1)/(m+1) computed from EXACT m ⇒ no cancellation; log(x)≈(x-1) delivered to full relative precision. Verify the tiny result rounds correctly (near-underflow of the result magnitude, not of x).; log of a power of two (x=2^e exactly, m=1, s=0): log = e·ln2 exactly — exercises the reduction constant, no series content.; near-midpoint hard-to-round: an input whose residual g lands within a few×2^-60 of ½. For exp/log these resolve at W=256; the certificate records a small hardness_margin_bits. Sourced from the Lefèvre–Muller anchors the validator pins.; k off-by-one (exp) / octave-fold boundary (log): both branches remain correct — assert via a boundary input (x ≈ (k+½)·ln2 for exp; x just above/below √2·2^e for log) that the result is unchanged.; f32 vs f64 single-rounding: the SAME wide value rounds once to each target; assert no double-rounding (round-to-f32 must not route through f64).
**Open questions:** Fixed conservative term-count per width vs dynamic termination: dynamic (stop when |term|≤2^-(W+GUARD)) is specified for simplicity and width-adaptivity; a fixed N (46/80/142 for exp, 50/100/202 half-terms for log) would be marginally faster and fully static-analyzable. Pick during implementation — dynamic recommended.; GUARD width: 24 bits is comfortable (truncation ≪ reduction error). Could be trimmed to ~12 if term-count matters, but 24 is safer and cheap at these widths.; ulp_bound for the ULP class of exp/log cells: use the §6.13 declared transcendental ceiling verbatim (likely 1 or 2 ULP) — confirm the exact number from ops.md when wiring the minter (out of this kernel's scope but needed for cell emission).; Whether exp(±0)=1.0 should be produced by the core (series gives exactly 1) or short-circuited in dispatch — recommend dispatch for uniformity with the other non-finite specials; either is correct.; The exact Lefèvre–Muller anchor inputs to vendor for the near-midpoint test (which .wc lines / Handbook entries) — deferred to the validator-wiring kernel; this kernel only pins that such a cell exists and resolves at 256 bits.

---

## sin — sin (+cos) atom: Payne-Hanek + near-k-pi cancellation  (difficulty: high)
**Summary:** Implementation-ready design for the sin and cos oracle atoms in conformance/src/hp.rs, at both f32 and f64. Consumes the (octant, reduced-argument r) pair the Payne-Hanek reduction kernel produces from the hard-coded 1280-bit 2/pi table, reconstructs sin/cos via a 4-entry octant table, evaluates sin(r)/cos(r) on |r|<=pi/4 by an exact-integer-recurrence Maclaurin series to >=256 bits with a bounded (alternating-series) truncation term folded into a Ziv interval, escalates 256->512->1024 on a straddle, and rounds ONCE to the target dtype RNE. The near-k-pi (sin) and near-(k+1/2)pi (cos) catastrophic-cancellation cases are handled for free because the reduction kernel is required to deliver r with full RELATIVE precision, so the tiny reduced argument still carries >=256 significant bits. Every cell self-certifies with hardness_margin_bits and stabilized_precision_bits (always 256 for slice-0 vectors, strictly > the 24/53-bit compute width per the 6.5-0009 carryover). Special values (signed zero, +-inf->NaN, NaN propagation) are handled at a front door as exact-byte cells, never plain ULP.

**Review verdict:** needs-fixes (constant_width_adequate=False)
_The evaluation kernel itself is numerically sound: the octant reconstruction tables (sin=[s,c,-s,-c], cos=[c,-s,-c,s]) are correct including the Euclidean-mod sign path for negative x; the alternating-decreasing Maclaurin series on |r|<=pi/4 with first-omitted-term truncation folded into the Ziv interval is rigorous; the Ziv resolving power at W=256 (round_error ~ n_ops*2^-256 ~ 2^-249 relative) dominates the worst known double sin/cos hard-to-round margin (~2^-113..2^-125 relative), so 'every slice-0 vector settles at 256' holds and the tie branch is genuinely unreachable by Lindemann; the near-k-pi correctness argument (avoid the subtraction, rely on full-RELATIVE-precision r) is correct, and the reduction-error propagation stays ~2^-W relative even for tiny results because r~V there. All 12 concrete test vectors (sin/cos f32/f64 at 1.0, 2^100, pi_f64, nextafter(pi), pi_f32, plus specials) reproduce bit-exact against libm. The reason it is not 'sound' is one load-bearing quantitative defect the spec itself cites and leans on: the '~1280-bit 2/pi table' figure is not a derived safe bound. It needs one fix (parametrize/enlarge the table width) plus a spec ruling on NaN canonicalization; the algorithm need not change._

**Review findings:**
- (important) The stated '~1280-bit 2/pi table covering |x| up to 2^1023' is under-derived and likely insufficient at the top exponent band. The binding constraint is the LOW/deep-fraction end, not the high-end 'reach': to deliver r at >=256-bit RELATIVE precision for a double x=M*2^E, the table must contain 2/pi bits down to absolute fractional index E + L + P + guard, where E_max=971 (|x|~2^1023), P=256, and L is the worst leading-zero cancellation run (globally ~61 bits for binary64). The provably-safe universal width is therefore ~971+61+256+24 = ~1312 bits, i.e. ~32 bits MORE than the cited 1280. A 1280-bit table only guarantees 256-bit-relative reduction for E<=939 (|x| up to ~2^991) in the worst-cancellation case; doubles in (2^991, 2^1023] with a long cancellation run can be reduced with FEWER than 256 relative bits — precisely the ~100% misround (truncated-table) failure the spec claims to prevent, and it would silently escape the Ziv test because the low-precision r looks self-consistent at width W. Fix: express the contract parametrically (P + max_cancellation_run + E_max + guard) rather than the magic number 1280, size the sibling Payne-Hanek table to >= ~1312 bits (with headroom), and add a reduction-kernel test that pins a top-band worst-case double (near 2^1023, long cancellation run) to a bit-exact reduced argument. Note ownership is the sibling PH kernel, but this atom cites the figure and states its correctness 'stands or falls' on the reduction contract, so the number must be corrected here too.
  - fix: Replace the fixed '~1280' with the parametric width E_max(971)+L_max(~61)+P(256)+guard and size the 2/pi table to >= ~1312 bits; add a kernel conformance test on a top-exponent (near 2^1023) worst-cancellation double.
- (minor) Special-value NaN handling pins a single canonical quiet NaN (7FF8.../7FC00000) as an exact-byte cell for sin/cos(NaN) and sin/cos(+-inf). Exact-byte comparison over-constrains conformance: IEEE 754 recommends propagating an input NaN's payload/sign, and many real libms return the input NaN (quieted) rather than a fixed canonical pattern, so a conformant implementation-under-test would FAIL the exact-byte cell. The signaling-NaN input path (quiet the result, raise invalid) is also collapsed into the same canonical output and the value-only corpus cannot assert the invalid flag. This is flagged as an open question in the spec, but the default chosen (canonical exact-byte) is the over-rejecting one.
  - fix: Get the KISS spec ruling before minting: either compare NaN cells by an is-NaN predicate (not exact byte), or pin payload-propagation semantics matching semantics.rs; document that sNaN inputs quiet+invalid and that the value-only oracle does not gate the invalid flag.
- (minor) Two correct-but-unjustified claims should cite their Diophantine basis so an implementer does not weaken them: (a) 'octant is unambiguous at W=256' relies on x*2/pi never landing within 2^-256 of a half-integer, which holds only because the worst-case closeness of a double's x*2/pi to a half-integer is ~2^-61 (same continued-fraction bound as the near-integer case) — far above 2^-256; (b) 'worst hard-to-round margin ~126 bits, far below 256's ~248-bit resolving power' is correct but assumes the seeded near-midpoint input is a genuine double/float hard case (<~2^-125), not an adversarial synthetic argument. State the ~2^-61 worst-case reduction bound explicitly and assert the minter rejects (Ziv-nonconverge error) rather than guesses if any seeded input ever escalates past 1024.
  - fix: Cite the ~2^-61 worst-case bound for both octant-resolution-at-256 and the 'settles at 256' claim; keep the 1024-bit non-convergence hard error wired (spec already does) and add a mint-time assertion that no slice-0 cell escalated unexpectedly.

**Algorithm:**

# sin / cos oracle atom — implementation-ready design

## 0. Scope and placement

Two public entry points per function, added to `conformance/src/hp.rs`:

```rust
pub fn sin_round_f64(x: f64) -> OracleResult;   // f64 sin atom
pub fn sin_round_f32(x: f32) -> OracleResult;   // f32 sin atom
pub fn cos_round_f64(x: f64) -> OracleResult;
pub fn cos_round_f32(x: f32) -> OracleResult;
```

`OracleResult` is the common return the minter consumes:

```rust
pub struct OracleResult {
    pub bits: u64,                     // rounded result, in the target dtype's raw bits
                                       // (f32 result occupies the low 32 bits)
    pub is_exact_special: bool,        // true for +-0 / +-1 / NaN front-door results
    pub hardness_margin_bits: u32,     // -log2(|V - nearest target midpoint| / ulp_target), floored
    pub stabilized_precision_bits: u32 // 256 | 512 | 1024  (the width the Ziv test settled at)
}
```

All four entry points funnel through **one** width-generic core so f32 and f64 share every line except the final rounding target:

```rust
fn sin_core(x: Wide, target: Target) -> OracleResult;   // Target = F32 | F64
fn cos_core(x: Wide, target: Target) -> OracleResult;
```

`Wide` is the escalating big-float from the hp.rs core kernel (the sibling slice-0
deliverable); see §1 for the exact surface this atom depends on.

---

## 1. Dependencies (sibling slice-0 kernels) — the surface this atom consumes

This atom does NOT implement the big-float or the argument reduction; it consumes them.
The two contracts it binds against:

### 1a. The big-float core (`Wide`)

A width `W in {256,512,1024}` binary big-float. This atom uses only:

```rust
Wide::from_f64(x: f64) -> Wide          // exact (f64 significand fits in any W)
Wide::from_f32(x: f32) -> Wide          // exact
Wide::from_i64(n: i64) -> Wide          // exact small integer
Wide::ZERO(width) -> Wide
fn width(&self) -> u32
fn neg(&self) -> Wide
fn add(&self, &Wide) -> Wide            // RNE to W bits
fn sub(&self, &Wide) -> Wide            // RNE to W bits
fn mul(&self, &Wide) -> Wide            // RNE to W bits
fn div_small_u64(&self, d: u64) -> Wide // divide by an exact small integer, RNE to W bits
fn is_sign_negative(&self) -> bool
fn abs(&self) -> Wide
// Directed rounding to a target dtype, returning the two enclosing representables
// and where V sits between them (used by the Ziv test, §4):
fn round_target(&self, t: Target) -> RoundProbe
```

`RoundProbe` reports, for the exact real `V` this `Wide` represents at width `W`:

```rust
struct RoundProbe {
    nearest_bits: u64,      // RNE result if V were exact
    nearest_midpoint_dist: Wide, // |V - m*|, m* = nearest dtype midpoint, as a Wide
    ulp_target: Wide,       // ulp of the dtype at V (handles subnormals)
}
```

If the core does not expose `round_target`, this atom implements it locally from
`Wide`'s significand/exponent (normal + subnormal + f32/f64 selectable); it is ~40
lines and belongs with the atom since the Ziv test is atom-agnostic. **Recommendation:
implement `round_target` once in hp.rs, shared by exp/log/sin.**

### 1b. The Payne-Hanek reduction kernel — the load-bearing contract

```rust
// Provided by the reduction kernel. Reduces a finite x against the hard-coded
// ~1280-bit 2/pi table, at working width W.
fn payne_hanek(x: Wide, width: u32) -> Reduced;

struct Reduced {
    octant: u8,   // = round(x * 2/pi) mod 4, in 0..=3
    r: Wide,      // x - octant_full*(pi/2), reduced to |r| <= pi/4, width W,
                  // sign preserved (r has the sign of x - k*pi/2)
}
```

**Three requirements this atom imposes on the reduction contract (all load-bearing;
state them in the reduction kernel's spec and test them there):**

1. **Range:** `|r| <= pi/4` exactly (nearest-integer `k`, not floor). This is what makes
   the Maclaurin series (§3) converge fast and stay alternating/decreasing.
2. **Full RELATIVE precision of `r`:** `r` is accurate to `>= W` significant bits
   *relative to r itself*, NOT relative to `x`. When `x` is near `k*pi`, `r` is tiny
   (its `Wide` exponent is very negative) but its significand still carries `>= W`
   meaningful bits, because the 1280-bit 2/pi table supplies enough fraction bits past
   the leading zeros of `x*2/pi`. **This single property is what makes near-k-pi
   cancellation (§5) correct.** A truncated table silently violates it and misrounds by
   ~100%, not 1 ULP.
3. **Reach:** the table covers `|x|` up to `2^1023 + pi/4 + guard`, so every finite f64
   (and a fortiori every finite f32) reduces exactly.

`octant` uses the convention verified in this design:
`octant = round(x / (pi/2)) mod 4`, with proper Euclidean mod for negative `x`
(e.g. `x=-3.3 -> k=-2 -> octant=2`).

---

## 2. Front door — special values (before any reduction)

Handled in the `*_round_*` wrappers on the **native** input, mirroring the
`semantics.rs` branch style. These produce `is_exact_special = true` cells that the
minter emits as **exact-byte** (class `ExactByte`), NEVER plain ULP — a plain ULP
bound >= 1 would accept `-0` where `+0` is pinned (`ulp_distance(-0,+0)=1`), and ULP
excludes NaN entirely.

| input x                | sin(x)      | cos(x)      | cell class  | tags |
|------------------------|-------------|-------------|-------------|------|
| `+0.0`                 | `+0.0`      | `+1.0`      | exact-byte  | signed-zero |
| `-0.0`                 | `-0.0`      | `+1.0`      | exact-byte  | signed-zero |
| `+inf` / `-inf`        | `NaN` (qNaN)| `NaN` (qNaN)| exact-byte  | domain-boundary, nan-propagation |
| `NaN` (any payload)    | `NaN` (q)   | `NaN` (q)   | exact-byte  | nan-propagation |

Rules:
- `sin` is odd: `sin(-0) = -0`. `cos` is even: `cos(-0) = +1` (sign of zero of the
  *result* is `+`). These are the signed-zero facts §6.5-0008 requires as edge cells.
- `+-inf -> NaN` is the IEEE "invalid" domain result (C99/IEEE `sin(inf)` is NaN).
  Emit the canonical quiet NaN `7FF8000000000000` (f64) / `7FC00000` (f32); do not
  attempt to preserve an infinity's (nonexistent) payload.
- `NaN -> NaN`: propagate quietly. Emit canonical qNaN; the corpus compares NaN cells
  exact-byte, so pin one canonical bit pattern (payload-propagation is not a KISS
  requirement for these atoms; canonicalize).
- `hardness_margin_bits` for a special = the general §4 formula, which yields `1` for
  `+-0` and `+-1` (they sit exactly `0.5*ulp` from the nearest midpoint) — trivially
  easy. `stabilized_precision_bits` for a special is set to `256` (the atom still
  declares wide evaluation; it did not need it, but the field must stay strictly `>`
  the compute width per the §6.5-0009 carryover — see §6).

Everything else (finite, non-zero) falls through to the core.

---

## 3. Core evaluation on the reduced argument

Given finite non-zero `x`, build `Wide x_w` (`from_f64`/`from_f32`, exact) at the
current width `W` (start `W=256`).

### 3a. Reduce
```
let red = payne_hanek(x_w, W);         // (octant, r), |r| <= pi/4, full rel. precision
```
Short-circuit: if `|x| <= pi/4` the reduction kernel returns `octant=0, r=x_w` with
zero reduction error. (This is an internal optimization of the reduction kernel, not
this atom's concern, but note it so the certificate's reduction-error term is 0 there.)

### 3b. Evaluate sin(r) and cos(r) by exact-recurrence Maclaurin series

On `|r| <= pi/4 < 1` both series are **alternating with strictly decreasing terms**, so
the truncation error is bounded by the magnitude of the first omitted term — a fact we
exploit for the Ziv interval (§4). Generate terms by an integer recurrence, so **no
transcendental constant and no stored factorial table is needed**:

```
sin(r) = sum_{k>=0} (-1)^k r^(2k+1)/(2k+1)!
cos(r) = sum_{k>=0} (-1)^k r^(2k)  /(2k)!
```

Compute both in one pass sharing `r2 = r*r`:

```
// cos
term = Wide::from_i64(1);           // k=0 term = 1
cos_acc = term; n = 0;
loop {
    // advance to next cos term: multiply by -r2 / ((n+1)(n+2)), then n += 2
    term = term.mul(&r2).neg();                       // * (-r^2)
    term = term.div_small_u64((n+1) as u64 * (n+2));  // / ((n+1)(n+2))
    cos_acc = cos_acc.add(&term);
    n += 2;
    if term_below_guard(&term, &cos_acc, W) { break; }   // |term| < 2^-(W+guard) * |acc|
}
cos_trunc = term;   // |first omitted term| bounds |cos(r) - cos_acc|

// sin: term_0 = r; step multiplies by -r2/((n+1)(n+2)) with n = 1,3,5,...
```

`term_below_guard`: stop once the next term's magnitude is `< 2^-(W+GUARD)` relative to
the accumulator, `GUARD = 24`. Because the series is alternating/decreasing, the stopped
`term` is a rigorous upper bound on the tail — **carry it as `*_trunc` into the interval,
do not discard it.** Term counts (for reference; the loop is self-terminating, these are
just sizing so an implementer can sanity-check): ~29 terms (degree ~57) at W=256, ~49
(degree ~97) at 512, ~82 (degree ~163) at 1024.

### 3c. Octant reconstruction (verified tables)

Let `s = sin(r)` (odd in r), `c = cos(r)` (even in r), `o = red.octant`:

```
sin(x) = [  s,   c,  -s,  -c ][o]
cos(x) = [  c,  -s,  -c,   s ][o]
```

Verified against mpmath at 500-bit precision for x in {1.0, pi_f64, 2^100, -3.3, 0.5}.
(e.g. x=1.0: o=1, sin=+c, cos=-s; x=pi_f64: o=2, sin=-s with s~1.2e-16.)

The result `V_wide` is one of `+-s` / `+-c` at width `W`.

---

## 4. Ziv rounding test + certificate

Build the error interval `[V_wide - eps, V_wide + eps]` where
`eps = round_error + trunc_error + reduction_error`:

- `round_error`: accumulated big-float rounding, `<= n_ops * 2^-W * |V|`. With
  `n_ops ~ 4*terms ~ 120` at 256 bits, `round_error/|V| <~ 2^-248`. (Track `n_ops` or
  bound it by a compile-time constant per width.)
- `trunc_error`: `|sin_trunc|` or `|cos_trunc|` selected by octant (the term carried out
  of §3b). Already `< 2^-(W+24)*|acc|`.
- `reduction_error`: `<= 2^-W * |r|` propagated through the series; the condition number
  of sin/cos on `|r|<=pi/4` is `<= ~1.9`, so it contributes `<~ 2^-(W-1)*|V|`.

Then probe the rounding:

```
let p = V_wide.round_target(target);
if p.nearest_midpoint_dist > eps {            // interval does NOT straddle a midpoint
    return settle(p, W, eps);                 // rounding decided at width W
}
// straddle: escalate
W = next_width(W);   // 256 -> 512 -> 1024
// recompute from 3a at the new width (re-reduce: PH re-run at wider W for a wider r)
```

If `W` reaches 1024 and the interval still straddles, that is a red-flag (return an
error the minter surfaces; it must never silently emit a guessed bit). For sin/cos the
worst hard-to-round margin is ~126 bits, far below 256's ~248-bit resolving power, so
**every slice-0 sin/cos vector settles at 256**; the 512/1024 ladder is the wired safety
net (load-bearing for tan-near-pole and pow later, exercised here only structurally).

`settle` computes the certificate:

```
hardness_margin_bits = floor( -log2( nearest_midpoint_dist / ulp_target ) )
stabilized_precision_bits = W          // the width the straddle test passed at
bits = p.nearest_bits
```

`hardness_margin_bits` orientation: **larger = harder** (closer to a midpoint). Easy
vectors read ~1-4 bits; genuine hard-to-round cases (slice-1 CORE-MATH `.wc` inputs)
read ~113-126. Because sin/cos of a nonzero algebraic argument is transcendental
(Lindemann), the exact value is **never** a dtype midpoint, so the RNE tie branch is
unreachable outside the §2 specials — no tie-breaking logic is needed in the core.

`round_target` handles the **f32 subnormal** result range (e.g. sin near a large `k*pi`
can round to an f32 subnormal or to 0): use the dtype's actual `ulp_target` at `V`
(gradual-underflow ulp), not a fixed `2^-23`/`2^-52`.

---

## 5. Near-zero (near-k-pi / near-(k+1/2)pi) cancellation — why it is already correct

When `x` is near `k*pi` (k != 0), `sin(x)` is tiny and equals `+-s = +-sin(r)` with `r`
tiny (octant even). Naively forming `r = x - k*(pi/2)` in f64 (or even at 200 bits, as
demonstrated: the 2^100 octant check mismatched by O(1) at prec=200 but agreed to 2.9e-122
at prec=500) catastrophically cancels. The design **avoids the subtraction entirely**:
the Payne-Hanek kernel produces `r` directly from the 2/pi table with full *relative*
precision (§1b req 2), so `r`'s significand is fully populated even though its exponent
is very negative. The Maclaurin `sin(r) ~ r - r^3/6 + ...` then delivers the correctly
rounded tiny result. Concretely for `x = pi_f64` (the nearest double to pi, which is
`pi - 1.2246e-16`): `r ~ -1.2246e-16 ~ -2^-52.9`, known to >=256 bits, and
`sin(pi_f64) = 1.2246467991473532e-16` rounds to `3CA1A62633145C07`.

`cos` has the identical phenomenon at the **odd** multiples `(k+1/2)*pi` (octant maps
`cos` to `+-sin(r)` with `r` tiny). Same mechanism, same guarantee. The minter tags both
families `near-k-pi` (see §6) so §6.5-0008's edge-coverage check sees them.

---

## 6. Minter / corpus integration (what this atom emits)

Per finite non-special cell the minter (`kiss_mint.rs`) writes, using this atom's
`OracleResult`:

- `op`: `"sin"` / `"cos"`; `dtype`: `"f32"` or `"f64"`; `rounding`: `"roundTiesToEven"`.
- `class`: `"ULP"` (transcendental accuracy tier, §6.8 / §6.13-0007);
  `ulp_bound`: the atom's declared ceiling from `accuracy::advisory_floor_ulp("sin")`
  (currently `4`). The `expected` value is the *correctly rounded* (<=0.5 ULP) oracle
  value; `ulp_bound` is the tolerance an implementation-under-test is granted, not the
  oracle's error.
- `provenance`: `"oracle"` (the validator, §7, guards on this).
- `expected.bits`: `hex(result_bits.to_be_bytes())` truncated to the dtype width.
- `certificate`: `{ "hardness_margin_bits": h, "stabilized_precision_bits": s }`.

**Special-value cells** (§2) are emitted as `class:"exact-byte", ulp_bound:0`.

**`evaluation_precision` enum reconciliation (carryover, without touching frozen v1):**
The frozen v1 schema stores `certificate.stabilized_precision_bits` as a plain integer
and has no `evaluation_precision` key — do not add one. The reader/minter *derive* the
authoritative enum: `stabilized_precision_bits == dtype_width  =>  compute-dtype`;
`stabilized_precision_bits >  dtype_width  =>  wider-than-compute`, and in the latter
case `stabilized_precision_bits` **is** the `certificate_precision_bits` detail (never a
free-standing field that can drift). For every transcendental (sin/cos) cell the value is
`256 > 53` (f64) and `256 > 24` (f32) — i.e. always `wider-than-compute`, and **strictly
greater** than the compute width, satisfying the §6.5-0009 assertion. The minter MUST
assert `stabilized_precision_bits > dtype_width` for every transcendental cell before
writing it (a `256`-vs-`24` regression is then a mint-time panic, not a silent bad cell).

**Required edge tags the minter MUST attach (so §6.5-0008 coverage passes for sin/cos):**
`nan-propagation`, `signed-zero`, `domain-boundary`, `near-midpoint`, `large-|x|-trig`,
`near-k-pi`. Each maps to at least one seeded input:
- `signed-zero`: `sin(+-0)`, `cos(+-0)`.
- `nan-propagation` + `domain-boundary`: `sin(NaN)`, `sin(+-inf)`, `cos(+-inf)`.
- `large-|x|-trig`: `sin(2^100)` (f32 and f64); optionally up to ~`2^1023` at f64.
- `near-k-pi`: `sin(pi_f64)`, `sin(nextafter(pi_f64,+inf))`; f32 `sin(pi_f32)`; cos gets
  its `(k+1/2)pi` analog cell.
- `near-midpoint`: at least one CORE-MATH-derived hard-to-round argument (slice-1 grows
  this set; slice-0 seeds one so the tag exists).
- interior sanity: `sin(1.0)`, `cos(1.0)`.

---

## 7. Validator integration (`tools/validate_corpus.py`, dev-time only)

The transcendental leg (currently a `NotImplementedError` stub) gains a `sin`/`cos`
recompute that:
1. Guards `cell["provenance"] == "oracle"` before trusting/validating (carryover).
2. Recomputes the correctly rounded value **three independent ways** and requires
   bit-for-bit agreement with `expected.bits`:
   - **mpmath** with its own Ziv loop: raise `mp.prec` until the rounded f64/f32 is stable
     across two precisions; round by exact high-precision midpoint sign comparison
     (never naive `float()` double-rounding; for f32 round the *real*, not via f64).
   - **MPFR via gmpy2** (`gmpy2.sin`/`cos` at high precision, then correct-round),
     driven additionally by CORE-MATH `.wc` hard cases in slice 1.
   - **Lefevre-Muller pinned anchors** for sin/cos (independent of both engines).
3. Constants independence: the Rust atom hard-codes its own 2/pi table (via the reduction
   kernel); mpmath and MPFR compute their own pi internally — no shared constant.

The f32 and f64 correct-rounding helpers used to *produce* the reference vectors in this
design (mpmath at prec>=400, real->f32 RNE by explicit significand extraction) are the
seed of that leg.

---

## 8. Difficulty & effort

- The **polynomial** on `|r|<=pi/4` is medium: fast-converging, alternating, exact integer
  recurrence, cleanly bounded truncation.
- The **hardness** is entirely in (a) the Payne-Hanek dependency delivering `r` at full
  relative precision (owned by the reduction kernel, but this atom's correctness stands or
  falls on it — near-k-pi is the proof), and (b) getting the octant reconstruction, signed
  zero, and NaN/inf front door exactly right so ULP cells never launder a wrong sign.
- Overall **high**, dominated by the reduction contract and the cancellation reasoning.

**Test vectors:**
- `sin, f64, x=+0.0 (bits 0000000000000000)` -> `0000000000000000 (+0.0)`  signed zero; sin is odd; emit exact-byte, tag signed-zero
- `sin, f64, x=-0.0 (bits 8000000000000000)` -> `8000000000000000 (-0.0)`  sin(-0)=-0; exact-byte; a plain ULP cell would wrongly accept +0
- `cos, f64, x=+0.0 (bits 0000000000000000)` -> `3FF0000000000000 (1.0)`  cos even; result sign +; exact-byte
- `cos, f32, x=+0.0 (bits 00000000)` -> `3F800000 (1.0f)`  f32 cos(0)=1
- `sin, f64, x=1.0 (bits 3FF0000000000000)` -> `3FEAED548F090CEE`  octant 1 -> +cos(r); interior sanity; hardness_margin_bits ~1, stabilized 256
- `cos, f64, x=1.0 (bits 3FF0000000000000)` -> `3FE14A280FB5068C`  octant 1 -> -sin(r); hardness_margin_bits ~4, stabilized 256
- `sin, f32, x=1.0 (bits 3F800000)` -> `3F576AA4`  f32 sin(1.0)
- `cos, f32, x=1.0 (bits 3F800000)` -> `3F0A5140`  f32 cos(1.0)
- `sin, f64, x=2^100 (bits 4630000000000000)` -> `BFEBE8ED97AC1F59`  large-|x|-trig; requires full Payne-Hanek (octant 3 -> -cos(r)); real=-0.8721836054; naive reduction is ~100% wrong
- `cos, f64, x=2^100 (bits 4630000000000000)` -> `3FDF4EB3FF66E36D`  large-|x| cos; real=0.4891786570
- `sin, f32, x=2^100 (bits 71800000)` -> `BF5F476D`  f32 large-|x|-trig
- `sin, f64, x=pi_f64 = nearest double to pi (bits 400921FB54442D18)` -> `3CA1A62633145C07`  NEAR-K-PI cancellation (k=1, octant 2 -> -sin(r)); true value 1.2246467991473532e-16 ~ 2^-52.9; correct only via full-relative-precision r; hardness_margin_bits ~1, stabilized 256
- `cos, f64, x=pi_f64 (bits 400921FB54442D18)` -> `BFF0000000000000 (-1.0)`  cos(pi_f64) rounds to exactly -1.0 (octant 2 -> -cos(r), r~ -1.2e-16, cos(r)~1)
- `sin, f64, x=nextafter(pi_f64,+inf) (bits 400921FB54442D19)` -> `BCB72CECE675D1FD`  near-k-pi, one ULP past pi; true value -3.2162452993532730e-16 (now negative); pins the sign flip across k*pi
- `sin, f32, x=pi_f32 = nearest f32 to pi (bits 40490FDB)` -> `B3BBBD2E`  f32 near-k-pi; pi_f32=3.1415927410 > pi, so sin is negative small (~ -8.74e-8); octant 2 -> -sin(r)
- `cos, f32, x=pi_f32 (bits 40490FDB)` -> `BF800000 (-1.0f)`  f32 cos near pi rounds to -1.0
- `sin, f64, x=+inf (bits 7FF0000000000000)` -> `7FF8000000000000 (qNaN)`  domain-boundary + nan-propagation; exact-byte
- `cos, f64, x=+inf (bits 7FF0000000000000)` -> `7FF8000000000000 (qNaN)`  domain boundary; exact-byte
- `sin, f64, x=NaN (bits 7FF8000000000000)` -> `7FF8000000000000 (qNaN)`  nan-propagation; canonical qNaN; exact-byte

**Constants required:** None owned by this atom: the Maclaurin coefficients 1/(2k+1)! and 1/(2k)! are generated by an exact integer recurrence (multiply by -r^2, divide by the small integers (n+1)(n+2)) — no stored factorial table, no transcendental constant.; GUARD = 24 (extra bits below the working width W at which the self-terminating series loop stops; the stopped term is the rigorous alternating-series truncation bound).; Canonical quiet-NaN bit patterns for special-value cells: f64 = 0x7FF8000000000000, f32 = 0x7FC00000.; DEPENDENCY (owned by the Payne-Hanek reduction kernel, not this atom): the hard-coded ~1280-bit 2/pi table (reach up to 2^1023 + pi/4 + guard) and pi/2 to 256/512/1024 bits. This atom requires the kernel to deliver r with full RELATIVE precision (>=W significant bits relative to r), which is the load-bearing property for near-k-pi correctness.
**Edge cases:** sin(+0)=+0 and sin(-0)=-0 (signed zero; sin is odd) — emitted exact-byte, never plain ULP (ulp_distance(-0,+0)=1 would accept the wrong sign).; cos(+0)=cos(-0)=+1 (cos is even; result sign is +).; sin(+-inf)=NaN, cos(+-inf)=NaN (IEEE invalid / domain boundary) — canonical qNaN, exact-byte.; sin(NaN)=NaN, cos(NaN)=NaN — quiet propagation, canonical qNaN, exact-byte (ULP excludes NaN).; |x| <= pi/4: no reduction, octant=0, r=x exactly, reduction_error=0.; Very small |x| (< ~2^-27) and subnormal x: sin(x) rounds toward x but the polynomial still runs and correctly rounds; no cancellation, but signed handling must survive (sin(-tiny) negative).; near k*pi (sin) / near (k+1/2)pi (cos): catastrophic cancellation — correct ONLY because Payne-Hanek delivers r with full relative precision; a truncated 2/pi table misrounds ~100%.; Large |x| up to ~2^1023 (f64) / ~2^128 (f32): naive reduction is ~100% wrong (demonstrated: octant check agreed only at >=500-bit precision, failed at 200); requires the full-width PH reduction. sin(2^100) is the seeded showcase.; Negative x: sin odd, cos even; Euclidean octant mod handles the sign (e.g. x=-3.3 -> octant 2); r keeps the sign of x-k*pi/2.; f32 result underflowing to a subnormal or to 0 (sin near a large k*pi can be a tiny f32): round_target must use the gradual-underflow ulp, not a fixed 2^-23.; RNE tie: sin/cos of a nonzero algebraic argument is transcendental (Lindemann), so the exact value is never a dtype midpoint — the tie branch is unreachable outside the handled specials; no tie-break logic needed in the core.; Ziv non-convergence at 1024 bits: must surface an error to the minter, never silently emit a guessed bit (red-flag, not expected for sin/cos).
**Open questions:** ulp_bound for sin/cos cells: use accuracy::advisory_floor_ulp (=4) as the declared ceiling, or a tighter per-atom value? The example exp cell in the design used ulp_bound:2. The oracle value is correctly rounded regardless; only the IUT tolerance is at stake. Recommend 4 (the advisory floor) pending the maintainer's accuracy-tier decision.; Whether round_target lives in the shared hp.rs core (recommended — exp/log/sin all need it) or is duplicated per atom. Recommend shared.; NaN payload canonicalization: this design canonicalizes sin(NaN)->qNaN 7FF8...; confirm KISS does not require payload propagation for trig atoms (semantics.rs suggests propagate-quiet is acceptable, but the corpus pins one pattern).; cos near-(k+1/2)pi cells: reuse the tag 'near-k-pi' (this design) vs introduce a distinct 'near-half-pi' tag for the §6.5-0008 required-tag set. Reusing keeps the sin/cos required-tag sets uniform; confirm with the coverage-test author.; Lefevre-Muller anchor set for sin/cos in the validator: which pinned worst cases to vendor for slice 0 (the design cites Arith-15 2001 / the Handbook).

---

## clog — clog complex atom (+ atan2 dependency) with C99 Annex G branch cuts  (difficulty: high)
**Summary:** Design for the clog complex atom (real = 0.5·log(re²+im²), imag = atan2(im,re)) as the single complex representative of Slice 0, computed correctly-rounded (≤0.5 ULP per component) in the hp.rs 256-bit core. RECOMMENDATION: KEEP clog in Slice 0 and resolve its atan2 dependency by adding a scoped, in-core atan2 (special-value table + a benign atan core + a hardcoded π constant) — NOT by deferring. The "hypot/log1p real-part form" in the design does not remove the atan2 need (the imaginary part IS atan2), so atan2 is unconditional; it is however cheap here because atan is monotone/well-conditioned and — unlike sin/cos/tan — needs NO Payne–Hanek reduction. clog reuses the Slice-0 log atom for the real part and adds only atan + a big-float sqrt + a π table. C99 Annex G branch cuts and the four signed-zero quadrants are handled as an EXACT-match special-value table (outside the precision core), which the compare_c32_transcendental split comparator enforces via its ±π (case b) and zero (case a) rules.

**Review verdict:** needs-fixes (constant_width_adequate=None)
_The atan2/imag path and the branch-cut comparator mapping are numerically sound, and all hand-verifiable test vectors check out. But the spec makes a false correctness claim about its real part and ships an incomplete special-value table — both fixable, neither invalidating the architecture. (1) HEADLINE: real = 0.5·log(re²+im²) in a fixed 256-bit window is NOT correctly-roundable for near-|z|=1 inputs with one tiny component. Failure input z=(1.0, k·2^-126): im²≈2^-252, the [u64;4] window reaches only 2^-255, capturing ~3 bits of im², but the result ≈0.5·im²≈2^-253 needs im² to ~24 bits (down to ~2^-276). Relative error in the signal-bearing im² is ~2^-3 — catastrophic. It is correct ONLY if Ziv escalates to 512 bits AND re-squares im at the wider width AND sumsq.ulp_err is set to the alignment-truncation bound. The spec instead asserts these cells 'stabilize at 256' and calls log1p 'optional, not a correctness requirement' — both false. §4a further overstates the exact window as ~200 binades when 256 bits allows only ~104. (2) §9 does not require a near-unit-circle/tiny-perturbation edge tag, so the one input class that forces the escalation the design depends on need never appear in the corpus. (3) The §5/§3a C99 tables — which the spec itself flags as the load-bearing exact-match trap numeric engines cannot validate — enumerate only the y≥0/+∞ halves and rely on unstated conj symmetry, omitting (x,−∞)→(+∞,−π/2), (−∞,y<0)→(+∞,−π), (+∞,y<0)→(+∞,−0). (4) The §7 cell shape emits free-standing certificate.stabilized_precision_bits, violating the mandated evaluation_precision-enum carryover (drift confirmed in-tree at corpus.rs:93). The core math (no-Payne-Hanek atan2, exact π/2·π/4 shifts, no-cancellation π−a0 quadrant assembly, alternating-series truncation bound) is correct.</parameter>
<parameter name="constant_width_adequate">true_

**Review findings:**
- (critical) Real part 0.5·log(re²+im²) is not correctly-roundable at 256 bits for near-|z|=1 inputs with one tiny component. Failure input z=(1.0, ~1.3·2^-126): im²≈2^-252 but the [u64;4] significand anchored at 2^0 reaches only 2^-255, capturing ~3 bits of im²; the result ≈0.5·im²≈2^-253 needs im² to ~24 significant bits. This is a fixed-window dynamic-range failure, not a significand-precision failure. The spec's claims that clog cells 'stabilize at 256' and that log1p is 'optional, not a correctness requirement' are false for this input class.
  - fix: State normatively that near-unit-circle tiny-component cells REQUIRE escalation to W=8 (512, window to 2^-511 captures im² to full f32 precision; W=16 never needed since worst case im=2^-149 gives im²=2^-298). §6 must re-FORM sumsq (re-square both components) at the escalated width, not merely re-round the 256-bit sumsq. Mandate sumsq.ulp_err = the one-sided (toward-zero) alignment-truncation bound (~1 ULP of the 256-bit rep) so the Ziv straddle test actually fires. Remove the 'stabilizes at 256 / log1p optional' language and record stabilized_precision_bits=512 for these cells.
- (important) §5/§3a special-value tables — which the spec itself designates the one load-bearing exact-match trap that numeric engines cannot validate — are incompletely enumerated. They give only y≥0 and +∞-imaginary rows and rely on unstated conj symmetry. Missing rows an implementer coding literally would get wrong: (x,−∞) finite x → (+∞,−π/2); (−∞,y) finite y<0 → (+∞,−π); (+∞,y) finite y<0 → (+∞,−0).
  - fix: Either enumerate all sign mirrors explicitly, or state clog(conj z)=conj(clog z) as a normative reduction applied before the table. Since the point is exact-match logic no engine validates, prefer full enumeration and pin each against C11 §G.6.3.2.
- (important) §9 required edge tags omit any near-unit-circle / tiny-perturbation class, so the exact input family that forces the real-part escalation the design's correctness depends on need never appear. The corpus can pass §6.5-0008 while never exercising the escalation path.
  - fix: Add a required edge tag (e.g. 'near-unit-circle' or 'tiny-perturbation') and seed at least one cell z=(1.0, k·2^-126) plus a z=(k·2^-126, 1.0) mirror; assert its certificate stabilized_precision_bits==512, not 256.
- (important) §7 clog cell emits certificate:{hardness_margin_bits, stabilized_precision_bits} as free-standing fields, violating the load-bearing carryover that evaluation_precision be an authoritative enum {compute-dtype|wider-than-compute} with certificate_precision_bits nested under wider-than-compute ('never a free-standing field that can drift'). Confirmed the drift already exists in-tree (corpus.rs:93-94 reads certificate.stabilized_precision_bits; corpus_coverage.rs reads certificate_precision_bits).
  - fix: Emit evaluation_precision:'wider-than-compute' with the precision detail nested beneath it, and update corpus.rs/kiss_mint.rs/corpus_coverage.rs together so the clog rollout is the point where the mandated enum is introduced rather than propagating the pre-existing free-standing drift.
- (minor) §4a overstates the in-core exact range: 'when |re|,|im| within ~200 binades, sumsq is EXACT.' Two 48-bit squares 200 binades apart span 2·200+48≈448 bits, exceeding the 256-bit significand. Exactness holds only within ~104 binades.
  - fix: Correct the bound to ~104 binades (256-bit) / ~232 (512-bit); it does not affect correctness because ulp_err+Ziv cover the inexact case, but the overstatement underpins the false 'stabilizes at 256' confidence and should be fixed.
- (minor) Certificate aggregation hardness_margin_bits=min(real_margin,imag_margin) is undefined for special-value cells whose real component is ±∞ (e.g. clog(−0,+0)=(−∞,+π)): distance-to-nearest-f32-midpoint has no meaning for an infinity.
  - fix: Define the margin for an exact special-value component as +∞ (or a sentinel) so min() takes the finite imag component's margin; specify stabilized_precision_bits for such cells as the width of the π constant used, and document that ±∞ real components are exact-by-construction, not Ziv-rounded.

**Algorithm:**

# clog (+ scoped atan2) — implementation-ready design

## 0. Recommendation: KEEP clog in Slice 0

The design (§10.1) names clog as Slice 0's one complex op. Keep it. Rationale:

- **The atan2 dependency is unavoidable and cannot be traded away.** The prompt frames a choice ("include atan2, *or* a hypot/log1p real-part form + a scoped atan2"), but the hypot/log1p form only concerns the **real** part. The **imaginary** part of clog *is* atan2(im,re); there is no clog without atan2. So the real answer is: **bring a scoped atan2 into Slice 0.**
- **It is cheap here.** atan is the benign atom (monotone, well-conditioned, §5). Crucially, atan2 needs **no Payne–Hanek** and **no 2/π table** — the argument is a bounded ratio, not a large angle. Its only new constant is π (which the exp/log Slice-0 work already implies a need for high-precision constants), plus a big-float `sqrt` primitive (which csqrt in Slice 1 needs anyway).
- **clog reuses Slice-0 `log`** for the real part, so the *only* genuinely new numeric code is the atan core.
- **Architectural value:** clog is the only Slice-0 cell that exercises the c32 dtype, the op-named split-comparator refinement (§6.8-0008), and the Annex-G exact-match path. Freezing that path in Slice 0 is the point of picking a complex op now.

"Scoped" means: land atan2/atan/sqrt/π in `hp.rs` as the machinery clog calls, but **mint only clog cells in Slice 0**. Standalone atan/atan2 corpus cells + their §6.5-0008 coverage rows + their own validator legs are deferred to Slice 1. If the team ever wants the lightest possible complex representative, `carg` (= atan2 alone, no log real part) exercises the identical split/branch/signed-zero path with strictly less code — but since Slice 0 already carries `log`, clog is the correct choice.

---

## 1. hp.rs primitives this kernel requires

Recap of the core type (defined by the hp.rs slice; this kernel consumes it):

```
struct BigFloat<const W: usize> {   // W ∈ {4, 8, 16} → 256/512/1024-bit significand
    sign: i8,          // +1 / -1
    exp:  i32,         // value = sign · significand · 2^exp, significand normalized to [1,2)
    sig:  [u64; W],    // fixed-width significand, sig[0] MSW
    // + a companion error bound `ulp_err: u64` (units of the LSB) so truncation is tracked
}
```

Operations assumed available from the core slice: `from_f32`/`from_f64` (EXACT — every f32/f64 injects with zero error), `add`, `sub`, `mul`, `shl/shr` (exponent shifts, exact for powers of two), `cmp`, `round_to_f32(bf) -> (f32, RoundInfo)` where `RoundInfo` reports whether the value straddles an f32 midpoint (the Ziv straddle flag) and the distance to the nearest midpoint.

**New primitives clog adds to hp.rs:**

### 1a. `bf_sqrt<W>(a: &BigFloat<W>) -> BigFloat<W>`  (needed by atan half-angle)
Newton–Raphson on the reciprocal square root (division-free inner loop):
1. Require `a > 0` (callers guarantee this: `1 + w²` ≥ 1).
2. Seed `y0 = f64::sqrt(a.leading_f64()).recip()` injected as a BigFloat (≈53 good bits).
3. Iterate `y ← y · (3 − a·y²) / 2`. Each step doubles the correct-bit count: 53 → 106 → 212 → 424 → 848 → …; run ⌈log2(W·64/53)⌉+1 steps (≤4 at W=4, ≤5 at W=8, ≤6 at W=16).
4. Return `s = a · y`; do one final Newton correction on `s` directly (`s ← (s + a/s)/2`) OR bound the residual `a − s²` and fold it into `s.ulp_err`. The residual bound is what the Ziv test consumes.

### 1b. `bf_atan<W>(w: &BigFloat<W>) -> BigFloat<W>`  for `w ≥ 0`  (the benign atom)
No poles, no cancellation. Octant + half-angle reduction, then Maclaurin:

1. **Sign/zero:** `w == 0` → return `+0` (exact, ulp_err = 0). Callers pass `|w|` and re-apply sign.
2. **Octant reduction (w > 1):** `atan(w) = π/2 − atan(1/w)`. Compute `1/w` (Newton reciprocal, or `bf_div`), recurse **once** (the recursed argument is < 1). π/2 comes from the π constant by an exact exponent shift (§2).
3. **Half-angle reduction (0 < w ≤ 1):** apply `w ← w / (1 + bf_sqrt(1 + w²))` repeatedly `r` times until `w < 2^-16`. The identity is `atan(w) = 2·atan(w/(1+√(1+w²)))`, so `atan(w_original) = 2^r · atan(w_r)`. `r ≤ 17` suffices for any w ≤ 1. The final multiply by `2^r` is an **exact exponent add** (r is an integer, 2^r is a power of two).
4. **Maclaurin:** `atan(w_r) = w_r − w_r³/3 + w_r⁵/5 − …`, alternating. With `w_r < 2^-16`, term k ≈ 2^(−16(2k+1)); stop when the next term < 2^−(64·W + 8). Terms needed: ≤9 at W=4, ≤17 at W=8, ≤33 at W=16. **Truncation error is bounded by the first omitted term** (alternating series) and folded into `ulp_err` — this is the truncation-bound discipline §5 requires (the Ziv test cannot catch series truncation unless it is bounded).
5. Multiply by `2^r`, return.

Cost note: no `2/π` table, no k·π catastrophic cancellation — this is why atan is the "benign" atom.

### 1c. `bf_log` — REUSE the Slice-0 log atom (do not rebuild).

---

## 2. The π constant (the only new table)

Hardcode **π to ≥ 1088 bits** (1024 + 64-bit guard) as the max-width literal `PI_1024: [u64; 16]` with exponent 1 (value in [2,4)? no — normalize to [1,2): π = 1.5707…·2¹, store sig for 1.570796… and exp = 1). Provide narrowing views `PI_256`/`PI_512` by truncation with the dropped-tail error recorded.

- **π/2** = `PI` with `exp −= 1` (EXACT — a binary halving).
- **π/4** = `PI` with `exp −= 2` (EXACT).
- **3π/4** = `bf_add(π/2, π/4)` in-core.
- Round-to-f32 anchors (for the special-value table and test cross-checks): `round_f32(π)=0x40490FDB`, `π/2=0x3FC90FDB`, `π/4=0x3F490FDB`, `3π/4=0x4016CBE4`. `round_f64(π)=0x400921FB54442D18`.

**Independence guard (§8):** hp.rs hardcodes its own π; mpmath and MPC compute their own; `validate_corpus.py` spot-checks the first ~200 bits of hp.rs's π against a third published source before trusting any clog cell. `ln2` is likewise hp.rs's own (already true for Slice-0 log).

---

## 3. atan2(y, x) — special-value table + interior

`atan2` is a **sign/branch problem** (§5): the special cases are EXACT-match table logic, the interior is one `bf_atan` call plus a π-multiple offset. Implement the table FIRST and return before touching the core.

### 3a. Exact special-value table (checked in order; y,x are f32 injected exactly)
Signs of zeros are load-bearing — test `is_sign_negative()`, not `== 0`.

| condition | result (exact) |
|---|---|
| `x` is NaN or `y` is NaN | NaN |
| `y = ±0`, `x > 0` (incl `+0`) | `±0` (sign of y) |
| `y = ±0`, `x < 0` (incl `−0`) | `±π` (sign of y) ← **branch cut** |
| `y > 0`, `x = ±0` | `+π/2` |
| `y < 0`, `x = ±0` | `−π/2` |
| `y = ±0`, `x = +0` | `±0` |
| `y = ±0`, `x = −0` | `±π` |
| `y = ±∞`, `x` finite | `±π/2` |
| `y` finite, `x = +∞` | `±0` (sign of y) |
| `y` finite, `x = −∞` | `±π` (sign of y) |
| `y = ±∞`, `x = +∞` | `±π/4` |
| `y = ±∞`, `x = −∞` | `±3π/4` |

The `±π`, `±π/2`, `±π/4`, `±3π/4` values are produced from the π constant, then rounded once to f32. Because they come from a **256-bit π**, their certificate `stabilized_precision_bits = 256` (> 24) even though the *result* is "just a constant" — satisfying the §6.5-0009 carryover (strictly wider than compute dtype).

### 3b. Interior (both x,y finite, not both zero)
1. `t = |y| / |x|` (Newton reciprocal · |y|, in-core) — but to avoid precision loss when `|x|≪|y|` or vice-versa, feed `bf_atan` the **ratio built in-core from the exactly-injected y,x** (both exact BigFloats; the division is the only rounding and it is ≥256-bit).
2. `a0 = bf_atan(t)` (range [0, π/2]).
3. Quadrant assembly by the signs of (x, y):
   - x > 0: `a = a0`
   - x < 0: `a = π − a0`
   - then if `y < 0` (or `y` is `−0`): `a = −a`.
4. Round once to f32 RNE. Record margin + stabilized precision (§6).

---

## 4. clog assembly

`clog(z) = ( 0.5·log(re²+im²),  atan2(im, re) )`, each component an independent correctly-rounded f32.

### 4a. Real part — `0.5·log(re²+im²)`, overflow-safe by construction
1. Inject `re, im` as EXACT BigFloats.
2. `sumsq = bf_add(bf_mul(re,re), bf_mul(im,im))`.
   - **No overflow / underflow in-core:** i32 exponent covers re² up to 2^254 (|re| ≤ f32::MAX ≈ 2^128) and im² down to ~2^−298 (subnormal f32). This is the wide-precision equivalent of an overflow-safe `hypot` magnitude — the naive f32 `re²+im²` would overflow near f32::MAX, but the core cannot.
   - **Exactness:** when |re|,|im| are within ~200 binades, `sumsq` is EXACT (each square is ≤48 significant bits; the sum fits the 256-bit window). When they span > ~256 bits (e.g. |im| ~ 2^−126 beside |re| ~ 1), the smaller square's tail is truncated **≥ 252 bits below the leading bit**; that truncation is recorded in `sumsq.ulp_err` and is far below any rounding threshold. This is why the direct form is correct in-core and the `log1p` trick (used by f64 libms to dodge cancellation near |z|=1) is an OPTIONAL refinement here, not a correctness requirement — see 4c.
3. `real = bf_log(sumsq)` then `exp −= 1` (the ·0.5 is an EXACT binary halving).
4. Ziv-round to f32 (§6).

### 4b. Imaginary part
`imag = atan2(im, re)` per §3.

### 4c. Optional real-part refinement (implementation note, not normative)
Near |z| = 1 the real part is a tiny value with cancellation *inside* log (true value ≈ (sumsq−1)/2, which can be ~2^−149). The 256-bit core already resolves this — verified: `clog(0.9999995, 0.001)` → real ≈ 2.32e−8 rounds correctly to `0x32C6F7FB`; `clog(1.0, 1e−4)` → real ≈ 5e−9 → `0x31ABCC76`. If an implementer wants fewer 512-bit escalations near the unit circle, compute `real = 0.5·log1p((|re|−1)(|re|+1) + im²)` (the standard cabs-accurate form). Only worthwhile if profiling shows escalation churn; the direct form is correct.

---

## 5. C99 Annex G exact-match special-value table for clog (outside the precision core)

Like `pow`'s domain table living in `semantics.rs` (§5), clog's special values are EXACT table logic, evaluated BEFORE the core. Place in a complex-dispatch layer (extend `semantics.rs` or a new `semantics_complex` section). The core (§4) runs only for finite, not-both-zero, non-NaN z. Signs test `is_sign_negative()`.

| input z = (re, im) | clog(z) = (real, imag) | tag |
|---|---|---|
| `(−0, +0)` | `(−∞, +π)` | signed-zero |
| `(+0, +0)` | `(−∞, +0)` | signed-zero |
| `(−0, −0)` | `(−∞, −π)` | signed-zero |
| `(+0, −0)` | `(−∞, −0)` | signed-zero |
| `(−x, +0)`, x>0 finite | `(log x, +π)` | branch-cut (approach from above) |
| `(−x, −0)`, x>0 finite | `(log x, −π)` | branch-cut (approach from below) |
| `(+x, ±0)`, x>0 finite | `(log x, ±0)` | axis |
| `(+∞, y)` finite y≥0 | `(+∞, +0)` | overflow |
| `(−∞, y)` finite y≥0 | `(+∞, +π)` | overflow |
| `(x, +∞)` finite x | `(+∞, +π/2)` | overflow |
| `(±∞, +∞)` | `(+∞, +π/4)` / `(+∞, +3π/4)` for −∞ | overflow |
| `(±∞, −∞)` | `(+∞, −π/4)` / `(+∞, −3π/4)` | overflow |
| `(NaN, finite)` or `(finite, NaN)` | `(NaN, NaN)` | nan-propagation |
| `(±∞, NaN)` | `(+∞, NaN)` | nan-propagation (magnitude known) |
| `(NaN, ±∞)` | `(+∞, NaN)` | nan-propagation |

The `±π`/`±π/2`/`±π/4`/`±3π/4` imag components are the f32-rounded π-multiples from §2. The `log x` real components for the axis rows run through the core (real transcendental). `(±∞, …)` real component is `+∞` exact.

**Why exact-match, not tolerance:** the split comparator (`compare_c32_transcendental`) enforces these — case (a) requires an exact sign bit for a **zero** component (so `(+0, −0)`'s `−0` imag is pinned, not accepted as `+0`), and case (b) requires exact sign AND exact magnitude for a **±π** component (so both branch-cut rows are pinned). `±π/2` and `±3π/4` are NOT ±π and NOT zero, so they fall under case (c) ULP-tolerance — correct, since they are ordinary irrational rounded values.

---

## 6. Ziv rounding + per-component + per-cell certificate

Per component value `v` (a BigFloat with `ulp_err`):
1. Form the interval `[v − ulp_err_lo, v + ulp_err_hi]`.
2. **Ziv straddle test:** if the interval, mapped to f32, contains an f32 **midpoint** (a value exactly halfway between two adjacent f32s), the rounding is undecided → recompute the WHOLE component at W=8 (512), then W=16 (1024). An input that reaches 1024 unresolved is a red flag to investigate (log it), not to ship.
3. Round ONCE, RNE, to f32.
4. Record `component_margin_bits` = distance from the true value to the nearest f32 midpoint, in bits. Definition: scale `v` so its ULP = 1; let `d = |frac(v) − 0.5|` (distance of its within-ULP position to the midpoint); `margin_bits = floor(−log2(d))`. Record `component_stabilized_bits` ∈ {256, 512, 1024} (or 256 for an exactly-determined special value — it is stable at every width).

**Per-cell certificate** (one certificate for the c32 cell, aggregating its two components):
- `hardness_margin_bits = min(real_margin, imag_margin)` (the harder component governs).
- `stabilized_precision_bits = max(real_stabilized, imag_stabilized)`.

Both components are computed at ≥256 bits, so `stabilized_precision_bits ≥ 256 > 24` — clog cells satisfy the §6.5-0009 carryover (transcendental stabilized precision STRICTLY greater than compute-dtype width 24) unconditionally, including the special-value cells (whose imag is a 256-bit π-multiple).

---

## 7. Minter (`kiss_mint.rs`) — clog cell shape

Add a `clog` path emitting c32 cells. One cell:
```
{"tcId": N, "op": "clog", "dtype": "c32", "rounding": "roundTiesToEven",
 "inputs":  [{"role":"z","dtype":"c32","bits":"<8 bytes: re(4) im(4), BE each>"}],
 "expected": {"dtype":"c32","bits":"<8 bytes: real(4) imag(4)>"},
 "class": "ULP", "ulp_bound": <KISS-Ops §6.18 clog ceiling; see open Q>,
 "provenance": "oracle",
 "tags": [ <edge tags, see §9> ],
 "certificate": {"hardness_margin_bits": <min>, "stabilized_precision_bits": <max>}}
```
Notes:
- `dtype: "c32"` (already in the envelope `dtypes` table: 8 bytes, two binary32 lanes [re, im], re first).
- `expected.bits` is ONE contiguous 8-byte string (real lane then imag lane), matching the §4 "complex result is one contiguous ·-grouped byte string" rule.
- `class` is the true class `ULP` (§6.18-0014). The **split comparator is selected by op-name in the reader**, NOT stored (§4 / §6.8-0008). Do not invent a "split" class — corpus.rs rejects it.
- The minter computes `expected` by: run the §5 Annex-G table; if not a special value, run the §4 core. For a special-value cell whose imag is `±π`/`±0`, the sign is pinned by construction.
- The minter MUST assert (self-check) that any component whose expected is a zero or ±π is emitted with the exact intended sign — mirroring the existing signed-zero minter guard.

---

## 8. Reader / differential harness changes

### 8a. `corpus.rs` — already generic; add nothing structural
The existing `load_cell` parses `inputs[]`, `expected`, `class`, `ulp_bound`, `tags`, `certificate` generically. A c32 cell parses as inputs=[8 bytes], expected=8 bytes. No change needed except confirming `dtype: "c32"` is accepted (it is a free string field today).

### 8b. `corpus_differential.rs` — op-named dispatch to the split comparator
Extend the runner beyond `eval_add`. For each cell dispatch by `op`:
```
match cell.op.as_str() {
  "add" => /* existing exact-byte path */,
  "clog" => {
     let z = decode_c32(&cell.inputs[0]);          // [re, im] f32 from 8 BE bytes
     let actual = iut_clog(z);                      // implementation-under-test → [f32;2]
     let expected = decode_c32(&cell.expected);     // [re, im] f32
     // §6.8-0008 op-named refinement: clog/carg/csqrt/cexp ALWAYS use the split comparator,
     // overriding the class-default comparator, regardless of the stored class.
     compare_c32_transcendental(actual, expected, cell.ulp_bound)
        .map_err(|e| format!("tcId {}: {e}", cell.tc_id))?;
  }
  other => return Err(...),
}
```
- `decode_c32(bytes) = [f32::from_be_bytes(bytes[0..4]), f32::from_be_bytes(bytes[4..8])]`.
- Add a teeth test analogous to `a_normalize_to_plus_zero_add_is_caught`: an IUT clog that returns `+0.0` for the imag of `clog(−1, −0)` (should be `−π`) MUST be caught by case (b); an IUT that returns `+π` for `clog(+0,−0)` imag (should be `−0`) is caught by case (a). These prove the branch-cut/signed-zero teeth.

---

## 9. Coverage (§6.5-0008) — clog required edge tags

Add to `op_manifest.json`:
```
"declared_coverage_set": [ ..., "clog" ],
"transcendental_atoms": {
  "clog": {
    "family": "complex-transcendental",
    "required_edge_tags": ["nan-propagation","signed-zero","branch-cut","axis","overflow","near-midpoint"]
  }
}
```
The §6.5-0008 test asserts, for `clog`, that the union of tags over all clog cells contains every required tag — a clog covered only at interior points (no `branch-cut`, no `signed-zero`) FAILS. Tags NOT applicable to clog and deliberately omitted from its required set: `large-|x|-trig`, `near-pole`, `near-k-pi`, `deep-tail` (atan2 has no pole, needs no large-angle reduction, and its output is bounded to [−π,π] — these belong to sin/tan/erf, not clog). Document this omission in the manifest so it is a decision, not a gap.

The minter's clog input set MUST seed at least: the 4 signed-zero quadrants (signed-zero), `(−x,±0)` (branch-cut), `(+x,±0)` and `(0,+y)` (axis), the ∞ rows and a near-f32-max magnitude (overflow), a NaN row (nan-propagation), and the hard-to-round interior point in §11 (near-midpoint).

---

## 10. Validator leg (`validate_corpus.py`) — clog, dev-time only

Guard first: **only process `cell["provenance"] == "oracle"`** (the §8 obligation; skip promoted-differential/negative). Then, for `op == "clog"`:

- **Split the cell into its two f32 lanes** and validate each component independently against the sources (real ↔ log magnitude, imag ↔ atan2). Round each source's high-precision value to f32 by **exact high-precision midpoint sign comparison**, never `float()` naive double-rounding.
- **Leg 1 — mpmath:** `z = mp.mpc(re, im); v = mp.log(z)` at rising `mp.prec` with its own Ziv loop; real = `mp.re(v)`, imag = `mp.im(v)`.
- **Leg 2 — MPFR/MPC via gmpy2:** `gmpy2.log(gmpy2.mpc(re, im, precision))` — MPC is algorithmically independent of mpmath and is correctly-rounded per component. Drive interior points from CORE-MATH's atan/log `.wc` hard-case inputs (vendored dev-only, subsampled with a recorded seed) reused for the imag/real components respectively.
- **Leg 3 — Lefèvre–Muller anchors:** cover the COMPONENT functions `atan` (imag) and `log` (real) — the design's L–M source covers exp/log/sin/cos/atan, which is exactly clog's two components. This is the leg independent of both mpmath and MPC.
- **CRITICAL — signed-zero / axis cases have NO numeric leg.** mpmath, MPC, and L–M do **not** model the sign of a zero *input*, so they cannot validate `clog(+0,−0)` vs `clog(−0,+0)` etc. Validate the §5 Annex-G rows against a **hardcoded C99 Annex G table** in the validator (a 4th "spec-table" leg for exactly the signed-zero/branch-cut/∞/NaN rows). This is the same "table logic is validated against the spec, not a numeric engine" pattern the design uses for `pow`'s domain table. Emit the table with its C11 §G.6.3.2 citation in the provenance header.
- Freeze only when all applicable legs agree bit-for-bit per component.

---

## 11. Precision budget summary

- **Real part:** log of an exactly (or 252-bits-cleanly) formed `sumsq`. Reuses Slice-0 log's 256/512/1024 discipline. No overflow (i32 exp). Cancellation near |z|=1 handled by core width (verified, §4c).
- **Imag part:** one `bf_atan` (≤17 half-angle steps + ≤9 Maclaurin terms at 256 bits) + a π-multiple offset. atan is benign; **no Payne–Hanek, no 2/π table** — the single biggest simplification vs sin/tan.
- **New constants:** π to ≥1088 bits (π/2, π/4 by exact shift; 3π/4 in-core). ln2 reused.
- **New primitive:** `bf_sqrt` (Newton reciprocal-sqrt) — shared with Slice-1 csqrt.
- **Escalation:** the near-midpoint case in §12 stabilizes at 256 (margin ≈ 20 bits). Deep (>128-bit-margin) hard cases would trigger 512/1024; those are seeded from CORE-MATH `.wc` inputs during Slice-1 validation. The escalation machinery must be wired now even though Slice-0 clog cells are expected to stabilize at 256.

---

## 12. Difficulty & risk

**High** — not because clog's math is hard (it isn't; atan is benign, log is reused), but because clog is the cell that lands FOUR new things at once: the c32 dtype path, the op-named split-comparator dispatch, `bf_sqrt` + `bf_atan` + the π table, and the validator's complex+Annex-G legs. Each is individually tractable; the integration surface is what earns the "high". The one subtle correctness trap is the **signed-zero / branch-cut sign discipline**, which is exact-match table logic (§5) that numeric engines cannot validate (§10) — get the table right by transcribing C11 §G.6.3.2 directly, and let the split comparator (which already exists in lib.rs, unchanged) enforce it.

**Test vectors:**
- `z c32 = 40 40 00 00 40 80 00 00  (3+4i)` -> `3F CE 02 10 3F 6D 63 38  (real=log5=0x3FCE0210, imag=atan2(4,3)=0x3F6D6338)`  Clean interior; real=log(5), imag=atan(4/3). Non-hard, stabilizes at 256. class ULP, split comparator case (c) both lanes.
- `z c32 = BF 80 00 00 00 00 00 00  (−1 + 0i)` -> `00 00 00 00 40 49 0F DB  (real=+0.0, imag=+π)`  BRANCH CUT from above. imag must be EXACTLY f32 π (0x40490FDB) with + sign — comparator case (b) requires exact magnitude+sign, not tolerance.
- `z c32 = BF 80 00 00 80 00 00 00  (−1 − 0i)` -> `00 00 00 00 C0 49 0F DB  (real=+0.0, imag=−π)`  BRANCH CUT from below (imag input is −0.0). imag=−π (0xC0490FDB). Distinguishes the two sides of the cut — the load-bearing signed-zero-input case no numeric engine models.
- `z c32 = C0 00 00 00 00 00 00 00  (−2 + 0i)` -> `3F 31 72 18 40 49 0F DB  (real=log2=0x3F317218, imag=+π)`  Branch cut with nonzero real part: real=log|−2|=log2, imag=+π. Real lane is a genuine transcendental (log), imag is the exact π endpoint.
- `z c32 = 80 00 00 00 00 00 00 00  (−0 + 0i)` -> `FF 80 00 00 40 49 0F DB  (real=−∞, imag=+π)`  Signed-zero quadrant, Annex G G.6.3.2. real=−∞ (log 0), imag=+π. Certificate stabilized_precision_bits=256 (>24) from the π constant despite the degenerate magnitude.
- `z c32 = 00 00 00 00 80 00 00 00  (+0 − 0i)` -> `FF 80 00 00 80 00 00 00  (real=−∞, imag=−0.0)`  Signed-zero quadrant with imag result −0.0 (0x80000000). Comparator case (a): a +0.0 imag here MUST fail. Teeth for the signed-zero-of-a-zero-result path.
- `z c32 = 00 00 00 00 3F 80 00 00  (0 + 1i)` -> `00 00 00 00 3F C9 0F DB  (real=+0.0, imag=+π/2)`  Positive imaginary axis: real=log1=+0.0, imag=+π/2 (0x3FC90FDB). π/2 falls under ULP case (c) (it is not ±π), validating the interior-tolerance path with a constant.
- `z c32 = 7F 80 00 00 3F 80 00 00  (+∞ + 1i)` -> `7F 80 00 00 00 00 00 00  (real=+∞, imag=+0.0)`  Infinity row: real=+∞ exact, imag=atan2(1,+∞)=+0.0. Overflow tag. ULP case (a) pins imag +0 sign; real compared as +∞==+∞ (distance 0).
- `z c32 = FF 80 00 00 7F 80 00 00  (−∞ + ∞i)` -> `7F 80 00 00 40 16 CB E4  (real=+∞, imag=+3π/4=0x4016CBE4)`  Double-infinity: imag=+3π/4 (NOT ±π, NOT zero → ULP case c). Exercises the 3π/4 = π/2+π/4 in-core construction.
- `z c32 = 7E 96 76 99 7E 96 76 99  (1e38 + 1e38 i)` -> `42 AF B0 8B 3F 49 0F DB  (real≈87.845=0x42AFB08B, imag=π/4=0x3F490FDB)`  Near-f32-max magnitude: naive f32 re²+im²=2e76 overflows to +∞; the wide core computes real=log(1e38·√2)≈87.845 finite. Overflow-safety tag.
- `z c32 = 40 4B BE D2 40 0B 9A BF  (3.18352175 + 2.18131995 i)` -> `3F AC DB 5E 3F 19 C8 A1  (real=0x3FACDB5E, imag=atan2 near-midpoint=0x3F19C8A1)`  NEAR-MIDPOINT hard-to-round: imag true atan2≈0.60071763 sits ~7.8e−7 ULP (~2^−20) from an f32 midpoint. Stabilizes at 256 bits (margin ~20 bits recorded in certificate). Seeds the 'near-midpoint' coverage tag; deeper cases from CORE-MATH .wc trigger 512/1024.
- `z c32 = 7F C0 00 00 3F 80 00 00  (NaN + 1i)` -> `7F C0 00 00 7F C0 00 00  (NaN, NaN) — canonical qNaN both lanes`  NaN propagation: any NaN input → both lanes NaN. (Exact NaN payload per the crate's canonical qNaN; compared out of ULP domain — exact-byte within the special table.)

**Constants required:** π to ≥1088 bits (1024 + 64-bit guard), hardcoded as PI_1024:[u64;16], normalized to [1,2) with exp=1; narrowed views PI_256/PI_512 with recorded truncation error; π/2 = π with exp−1 (EXACT binary halving; no separate constant); π/4 = π with exp−2 (EXACT); 3π/4 = bf_add(π/2, π/4) computed in-core (not a stored constant); ln2 to full width — REUSED from the Slice-0 log atom, not new; f32 round anchors for cross-checks/tests: π=0x40490FDB, π/2=0x3FC90FDB, π/4=0x3F490FDB, 3π/4=0x4016CBE4; f64 π=0x400921FB54442D18
**Edge cases:** Branch cut, negative real axis approached from above: clog(−x,+0)=(log x, +π) — imag is exactly f32 round(π)=0x40490FDB, pinned by split comparator case (b); Branch cut approached from below: clog(−x,−0)=(log x, −π)=(...,0xC0490FDB) — the −0 imaginary input flips the branch; numeric engines cannot see this, validated by Annex-G table; Four signed-zero quadrants: clog(±0,±0) all give real=−∞ with imag ∈ {+0,−0,+π,−π} by input zero signs; pinned by comparator cases (a)/(b); clog(+0,−0) imag = −0.0 (0x80000000): must NOT be accepted as +0.0 — comparator case (a) exact sign of zero; Infinity rows (Annex G): real=+∞ exact; imag ∈ {±0,±π,±π/2,±π/4,±3π/4} by which component(s) are infinite; ±π/2 and ±3π/4 imag results are NOT ±π and NOT zero → fall under ULP-tolerance (comparator case c), correctly (ordinary irrational rounded values); NaN propagation: clog(NaN,finite)=(NaN,NaN); clog(±∞,NaN)=(+∞,NaN) (magnitude known even when arg is NaN); clog(NaN,±∞)=(+∞,NaN); Real-part cancellation near |z|=1: real = 0.5·log(sumsq) is a tiny value (e.g. ~5e−9); resolved by 256-bit core (verified 0x31ABCC76, 0x32C6F7FB) — log1p form is optional; Overflow-safe magnitude: |re|~f32::MAX would overflow naive re²+im² in f32 but the i32-exponent core (re² up to 2^254) cannot — no intermediate ∞; sumsq truncation when components span >256 bits (|im|~2^−126 beside |re|~1): smaller square truncated ≥252 bits down, error folded into ulp_err, provably below rounding threshold; atan2 quadrant assembly sign: y<0 OR y is −0 negates the angle — must test is_sign_negative(), not y<0 (which misses −0); Near-midpoint hard-to-round interior (test vector §), e.g. atan2 lands ~2^−20 from an f32 midpoint; stabilizes at 256 bits; escalation to 512/1024 wired for rarer deeper cases; atan octant boundary w=1 exactly (z on |re|=|im| diagonal): atan(1)=π/4; ensure the w>1 vs w≤1 branch and the recursion agree at the seam
**Open questions:** clog's ulp_bound (acceptance ceiling for the implementation-under-test) — needs the exact KISS-Ops §6.18-0014 declared transcendental ULP for complex ops. The exp example in the design uses 2; confirm the clog/carg/csqrt/cexp value from the ops table before minting. (The ORACLE itself is correctly-rounded ≤0.5 ULP regardless — this bound only governs the IUT.); Placement of the §5 Annex-G clog special-value table: extend semantics.rs (alongside cdiv/cconj) vs a new semantics_complex module. Leaning: extend semantics.rs to match the existing complex-op locality.; Canonical qNaN payload for the (NaN,NaN) clog result — pin to the crate's existing canonical qNaN (0x7FC00000) and confirm the differential/validator compare NaN lanes as exact-byte (out of ULP domain), consistent with how fp.rs propagates NaN.; Whether to vendor CORE-MATH atan.wc AND log.wc subsamples for clog's validator now (Slice 0) or defer to Slice 1 with the rest of atan2 — leaning defer; Slice-0 clog cells stabilize at 256 and the mpmath+MPC+L-M(atan,log)+Annex-G-table legs already give 3-source coverage without the .wc adversarial set.; atan2 interior ratio construction: divide-then-atan vs a two-argument atan core that avoids forming y/x — the divide-then-atan form is specified here (simpler, ≥256-bit division is well below the error budget); confirm no edge where |x|≪|y| division loses more than the recorded ulp_err (analysis says no, but worth a scan during implementation).

---

## validation — three-source dev-time validation harness (validate_corpus.py)  (difficulty: high)
**Summary:** A dev-time-only, never-shipped Python gate that reads the frozen language-neutral JSON bundle and certifies every `provenance=="oracle"` cell's stored `expected` bit-pattern is the correctly-rounded (round-ties-to-even) true value, by re-deriving it with TWO algorithmically independent arbitrary-precision engines per cell (mpmath (pure-Python) + one of MPFR/gmpy2 or Arb/python-flint) and a THIRD anti-collusion leg of pinned Lefèvre–Muller published worst cases (real atoms) / C99 Annex G branch-cut table (clog). Each engine runs its own Ziv loop (raise precision until the rounded dtype value is stable), rounds via exact integer mantissa/exponent arithmetic (never naive `float()`), and the gate fails unless all engines agree bit-for-bit with each other AND with the stored value. It guards on provenance, verifies engine independence by spot-checking each engine's self-computed constants (π, ln2, 2/π, …) against pinned decimal literals, pins every engine/library version + hard-case file hash + subsample seed into a reproducible PROVENANCE header, and documents python-flint (Arb, MPFR-independent) as the Windows/complex engine when gmpy2's GMP/MPFR toolchain is unavailable.

**Review verdict:** needs-fixes (constant_width_adequate=True)
_The kernel is mostly sound and I could not break its arithmetic core: all six real/complex test vectors verify bit-exact under mpmath 1.4.1 (exp/log/sin at f64+f32, sin(1e300)=BFEA2C16B010E385, clog anchors), the correctly_round integer rounder is exact (no double-round; verified drop = man.bit_length()-1-fbits independent of exp; overflow/subnormal/carry paths correct), and the provenance guard, anchor-presence gate, independence spot-check, and exact-byte-add leg are all well-reasoned. But two correctness defects block a clean 0-ULP-from-truth certification.

(1) IMPORTANT — the §3 Ziv stopping rule is the two-precision-agreement HEURISTIC, not the bound-based interval-straddle test the design itself mandates ("if the result interval straddles an f64 midpoint, recompute at 512, then 1024"). "bits==prev across two doublings" is not a proof of correct rounding: mpmath transcendental results carry no exported error bound, so two same-side approximations can stabilize on the wrong side of a midpoint. Worse, this heuristic lives in the SHARED ziv() harness, so it is a common-mode failure across BOTH the mpmath and gmpy2/MPFR legs — engine independence does not protect against it, and it is weakest exactly on the hardest-to-round non-anchor cells (the ones the corpus exists to stress; L-M anchors only cover dozens of points). MPFR and Arb both hand you a provable ±0.5ulp@p (Arb via ball radius, MPFR via its correct-rounding-to-p guarantee), so the fix is cheap and matches the design: wrap every engine's result in its proven interval and require the WHOLE interval to round identically, rather than comparing two rounded scalars.

(2) IMPORTANT — §6 lists "exp(x) for x above the overflow threshold -> +inf" as a special-value PRE-PASS item. The finite/inf boundary for exp is a correctly-rounded decision, not a fixed threshold: exp(0x40862E42FEFA39EF / 709.782712893384) = 0x7FEFFFFFFFFFFF2A (FINITE, below DBL_MAX), while the next ULP up (709.7827128933841) overflows to inf, with the true switchover at ln(2^1024*(1-2^-54)) = 709.78271289338399679. A hard-coded "x>threshold -> inf" pre-pass misrounds arguments straddling this boundary, and it does so on a load-bearing §6.5-0008 edge tag ("overflow"). Only exp(+/-inf) and exp(NaN) are true specials; every FINITE x must route through Ziv+correctly_round, which already yields inf correctly via the kk>bias carry. The underflow side is handled correctly (routed to Ziv); the overflow side is asymmetrically and wrongly special-cased.

Neither defect corrupts the verified slice-0 vectors, but both violate the "proof, not heuristic" claim the gate advertises, so: needs-fixes._

**Review findings:**
- (important) §3 ziv() uses the two-precision-agreement heuristic (bits==prev after doubling) instead of the design-mandated interval-straddle test. This is not a proof of correct rounding for the mpmath/gmpy2 legs (mpmath exports no error bound), and because it lives in the SHARED harness it is a common-mode failure across both non-Arb engines — defeating the independence argument precisely on the hardest non-anchor cells the corpus is built to stress.
  - fix: Adopt the design's interval discipline for every engine: form [y - 0.5ulp@p, y + 0.5ulp@p] (Arb: use ball [lower,upper]; MPFR: its correct-rounding-to-p guarantee gives the bound; mpmath: wrap +/-1ulp@p) and commit only when correctly_round(lo)==correctly_round(hi). Escalate precision on a straddle. Do not certify on scalar agreement across two precisions.
- (important) §6 special-value pre-pass treats finite-argument exp overflow ('x above the overflow threshold -> +inf') as a table entry. The finite/inf boundary is correctly-rounded, not a threshold: exp(709.782712893384 / 0x40862E42FEFA39EF) = 0x7FEFFFFFFFFFFF2A is FINITE while the next f64 up overflows; a fixed-threshold special misrounds straddling args on the load-bearing 'overflow' edge tag.
  - fix: Remove finite-domain overflow from special_value(). Only exp(+/-inf)/exp(NaN) are specials; all finite x go through Ziv+correctly_round, whose kk>bias carry path already emits inf exactly at the RNE overflow midpoint 2^1024*(1-2^-54).
- (minor) §5 arb_correctly_round assumes flint.arb exposes exact endpoints via y.lower()/y.upper() returning exact rationals. python-flint's arb API exposes mid/rad (arf/mag), not guaranteed .lower()/.upper() exact-endpoint accessors across versions; exact_triple() extraction may not match the shipped API.
  - fix: Derive endpoints explicitly from mid()/rad() (or arf accessors) and convert each dyadic to an exact (sign,man,exp) triple; pin the python-flint version in requirements-dev.txt and unit-test the endpoint extraction.
- (minor) §10 constant spot-check asserts |engine_const - ref| < 2**-256 but does not pin the precision at which each engine computes the constant, and builds ref = mpf(literal) at 1024 bits (~308 decimal digits) from a >=400-digit literal. If an engine computes the constant at p<~256 the check falsely fails; the literal/ref width mismatch is silently truncating.
  - fix: Compute each engine constant and ref at a pinned precision >= 300 bits with guard, state the tolerance as absolute vs relative, and set ref precision >= the literal's ~1329 bits so no digits are discarded before comparison.
- (minor) HardToRound is documented as flagging a 'genuine exact rounding midpoint (tie)' for transcendentals. exp/log/sin of a nonzero dyadic argument is transcendental (Lindemann–Weierstrass) and can never equal a dtype midpoint (a dyadic rational), so a real tie is impossible for slice-0 atoms — a HardToRound is ALWAYS a bug or pmax-too-low, never a legitimate tie.
  - fix: Reword: HardToRound is unconditionally a failure/red-flag for slice-0 transcendentals; drop the 'genuine tie, accept after human review' framing which could rationalize a real bug.
- (minor) Edge-tag list attaches 'near-pole' to sin, which has no poles; near-pole applies to tan (slice 1). Cosmetic mismatch between the guarantees and the atom set could cause a spurious coverage assertion.
  - fix: Scope 'near-pole' to tan/atan in later slices; for sin keep near-k*pi and large-|x| tags only.

**Algorithm:**

# validate_corpus.py — three-source dev-time certification gate

## 0. Role, invariant, and the exact thing being certified

`tools/validate_corpus.py` is **dev-time only, never shipped, never in a consumer's build**. It reads a frozen bundle (`conformance/corpus/*.json`) as plain JSON — it never calls Rust, never imports `hp.rs`'s constants. It replaces the current exact-byte-only stub.

**Certified invariant (per oracle cell):** the stored `expected` bits are the **correctly-rounded** (IEEE round-to-nearest, ties-to-even) image of the *true real* value of `op(inputs)` in the cell's `dtype`. This is a **0-ULP-from-truth** claim about the stored value — NOT the cell's runtime `ulp_bound` (which is the tolerance an implementation-under-test is later judged by; the validator ignores `ulp_bound` for the value check). The freeze may proceed only when, for every `provenance=="oracle"` cell, two independent engines and (where applicable) the third published-anchor leg all agree bit-for-bit with the stored bits.

The gate certifies **rounding, not completeness** — coverage of the hard stressors is the minter's job (§6.5-0008). The validator's independence machinery only makes the agreement *meaningful*.

## 1. Top-level control flow

```
def main(argv):
    args = parse_cli(argv)                      # bundles..., --engine, --hard-cases, --seed,
                                                # --emit-provenance, --strict-anchors, --allow-mpmath-only
    engines = build_engines(args.engine)        # {"mpmath": E, "second": E, "complex": E?}
    require_two_independent(engines, args)      # ABORT unless >=2 real; complex needs flint
    check_engine_independence(engines)          # constants spot-check vs pinned literals
    anchors = load_anchors()                     # L-M real table + C99-AnnexG clog table
    hard = load_hard_cases(args.hard_cases, args.seed)
    prov = emit_provenance(engines, anchors, hard, args.bundles, args.emit_provenance)

    fails, seen = 0, defaultdict(list)
    for path in args.bundles:
        data = json.loads(read(path))
        assert_envelope(data)                    # schema/ulp_metric/dtypes present & known
        for cell in data["vectors"]:
            key = (cell["op"], cell["dtype"], canonical_inputs(cell))
            seen[cell["op"]].append(key)
            if cell["provenance"] != "oracle":   # ---- PROVENANCE GUARD ----
                record_skip(cell); continue
            fails += certify_cell(cell, engines, anchors)

    fails += assert_required_anchors_present(seen, anchors, strict=args.strict_anchors)
    fails += cross_check_hard_cases(engines, hard)   # adversarial engine-agreement
    print_summary(prov, fails)
    return 1 if fails else 0
```

The **provenance guard** is the first branch inside the cell loop: only `oracle` cells are re-derived. `promoted-differential` and `negative` cells are counted and skipped (a `negative` cell has no numeric truth; a `promoted-differential` cell's authority is the differential loop, not this gate). A cell whose `provenance` is anything other than the three admissible tags is a hard fail (matches `corpus.rs`, which already rejects `reference-observed`).

## 2. The correctly-rounding primitive (shared, spec-logic, unit-tested)

All engines feed their high-precision result through ONE rounder. Sharing it is sound because it is *deterministic IEEE spec logic* (not a transcendental value and not a mathematical constant), it is independently unit-tested against known vectors, and the L-M/C99 anchor leg (whose published bytes were rounded by a third-party implementation) cross-checks it. **The value computation is what must be independent; the rounding rule is the spec and is intentionally identical.**

```
DTYPE = {
  "f64": (11, 52, 1023),   # (exp_bits, frac_bits, bias)
  "f32": ( 8, 23,  127),
}

def correctly_round(sign:int, man:int, exp:int, dtype) -> bytes:
    """Round the EXACT real value (-1)^sign * man * 2^exp (man,exp Python ints)
    to `dtype` under round-ties-to-even; return big-endian IEEE bytes.
    man==0 -> signed zero. Handles normal / subnormal / overflow->inf / underflow->0."""
    ebits, fbits, bias = DTYPE[dtype]
    if man == 0: return pack(sign, 0, 0, dtype)              # signed zero
    # normalize so the value = f * 2^E with 2^fbits <= f < 2^(fbits+1) conceptually,
    # by locating the unbiased binary exponent of the value:
    #   value in [2^k, 2^(k+1))  where k = floor(log2(|value|)) computed EXACTLY from man,exp.
    k = man.bit_length() - 1 + exp
    emin = 1 - bias                                          # min normal unbiased exp
    if k < emin:                                             # subnormal / underflow region
        shift = exp - (emin - fbits)                        # target LSB weight = 2^(emin-fbits)
        q, r_is_half, r_gt_half = round_shift(man, -shift)  # exact ties-to-even on integer q
        # q is the subnormal significand (may carry up into smallest normal); pack handles it.
        return pack_from_significand_subnormal(sign, q, dtype)
    # normal path: target has fbits fraction bits at value-exponent k
    shift = exp - (k - fbits)                                # bits to drop (>=0 usually)
    q, _, _ = round_shift(man, -shift)                       # q in [2^fbits, 2^(fbits+1)] after ties-to-even
    kk = k
    if q == (1 << (fbits+1)):                                # mantissa carry bumped the binade
        q >>= 1; kk += 1
    if kk > bias:                                            # overflow -> infinity
        return pack(sign, (1<<ebits)-1, 0, dtype)
    biased = kk + bias
    frac = q & ((1<<fbits)-1)                                # drop implicit leading 1
    return pack(sign, biased, frac, dtype)

def round_shift(man:int, drop:int):
    """Drop `drop` low bits of man with round-ties-to-even; drop may be <=0 (left shift, exact)."""
    if drop <= 0: return (man << -drop, False, False)
    lo  = man & ((1<<drop)-1)
    q   = man >> drop
    half = 1 << (drop-1)
    if lo > half or (lo == half and (q & 1)): q += 1
    return (q, lo == half, lo > half)
```

`round_shift` is **pure integer arithmetic** — no float ever touches the rounding decision, so it is exact and immune to double-rounding. Each engine's job is only to hand `correctly_round` an exact `(sign, man, exp)` triple that is *close enough to truth* that the rounding is decided; the Ziv loop (§3) guarantees "close enough".

Subnormal note: `pack_from_significand_subnormal` writes `q` directly as the trailing field with biased-exponent 0; a `q == 2^fbits` (round-up out of subnormal range) promotes cleanly to the smallest normal (biased exp 1, frac 0) — same structure as `fp.rs::fp8_magnitude`.

## 3. Ziv loop, per engine (exact-integer variant)

Each engine implements `correctly_round_value(op, dtype, inputs) -> bytes`:

```
def ziv(engine_eval, dtype, inputs, p0=128, pmax=4096):
    prev = None
    p = p0
    while p <= pmax:
        sign, man, exp = engine_eval(inputs, p)     # exact triple of f(x) computed at p bits
        bits = correctly_round(sign, man, exp, dtype)
        if bits == prev:                            # stable across two precisions -> committed
            return bits, p
        prev = bits; p *= 2
    raise HardToRound(op, dtype, inputs, pmax)      # never stabilized -> flag (likely exact tie/bug)
```

Stability across two successive precisions is the classic Ziv termination: once the approximation error is below the distance from the computed value to the nearest dtype midpoint, the rounded result cannot change. Non-termination at `pmax` (4096 bits ≫ the 256/512/1024 the Rust core needs) means the argument is an *exact* rounding midpoint (a genuine tie, extremely rare for transcendentals — flag for human review) or a bug. `HardToRound` is a **failure**, never silently accepted.

**Engine `engine_eval` must return an EXACT `(sign, man, exp)`**, not a rounded float:
- **mpmath**: `y = f(x)` at `mp.prec = p`. `y` is an `mpf` whose value is *exactly* `y.man * 2**y.exp` (mpmath stores binary man/exp). Extract `sign = 1 if y < 0`, `man = abs(y).man`, `exp = y.exp`. This is exact — the only error is the (bounded, Ziv-controlled) difference between `y` and the true `f(x)`.
- **gmpy2/MPFR**: `gmpy2.get_context().precision = p`; `y = gmpy2.exp(x)` (etc.). `m, e = y.as_mantissa_exp()` gives exact Python ints; `sign` from `gmpy2.sign` or `m<0`.
- **python-flint/Arb**: §5 — uses the ball radius directly instead of the two-precision heuristic (a strictly stronger stopping rule).

The inputs `x` are constructed at high precision from the cell's stored **input bits** (exact dtype value → exact `mpf`/`mpfr`/`arb`), so the engine evaluates at the exact argument the corpus pins. There is no decimal round-trip.

## 4. Engine matrix and the "third source"

Per-cell certification uses exactly the engines applicable to the cell:

| cell kind | engine 1 (always) | engine 2 (one of) | third leg |
|---|---|---|---|
| real atom `exp`/`log`/`sin`, f32 & f64 | **mpmath** (`mpf`) | **gmpy2/MPFR** or **python-flint/Arb** | L-M anchor table (on anchor inputs only) |
| complex `clog`, c32 | **mpmath** (`mpc`) | **python-flint/Arb** (`acb`) — *gmpy2 has no complex* | C99 Annex G table (branch cuts / signed zero) |
| exact-byte `add` (oracle) | native single-rounding recompute (§7) | — | — |

**Independence rationale (faithful to the design):** per *ordinary* (non-anchor) cell there are **two** genuinely independent engines: mpmath is pure-Python power-series/AGM code; MPFR and Arb are distinct C libraries with different algorithms (Arb is ball arithmetic, categorically non-MPFR). The **third source is Lefèvre–Muller**, but L-M is only *dozens of pinned anchor points*, so it operates as a **global anti-collusion gate** (§8): the minter is REQUIRED to include the L-M worst cases, and on exactly those cells all three legs must agree — catching a bug mpmath+MPFR might share. For clog the third leg is the C99 Annex G pinned table (L-M does not cover complex). This matches §8/§11 of the design exactly: "mpmath + MPFR share a bug → Lefèvre–Muller anchors independent of both."

`require_two_independent` **aborts the whole gate** (exit 2, "cannot certify") if fewer than two real engines are importable, unless `--allow-mpmath-only` is passed — which is a *non-certifying* smoke mode for CI wiring only and prints `NOT A FREEZE-CAPABLE RUN` loudly. On THIS machine (mpmath present, gmpy2 & flint absent) a real freeze run must first `pip install python-flint`.

## 5. python-flint / Arb — the Windows-and-complex engine

`gmpy2` needs a GMP+MPFR build toolchain, painful on Windows. `python-flint` ships prebuilt Arb/FLINT wheels (`pip install python-flint`) and provides `flint.arb` (real) and `flint.acb` (complex). Arb is **ball arithmetic** — every result is an enclosure `[m ± r]` with a *proven* error bound — so it is inherently MPFR-independent AND gives a stronger Ziv stop:

```
def arb_correctly_round(op, dtype, inputs, p0=128, pmax=4096):
    from flint import ctx, arb
    p = p0
    while p <= pmax:
        ctx.prec = p
        y = arb_eval(op, inputs)              # an arb ball enclosing the true value
        lo, hi = y.lower(), y.upper()         # exact endpoints (Arb exports exact rationals)
        blo = correctly_round(*exact_triple(lo), dtype)
        bhi = correctly_round(*exact_triple(hi), dtype)
        if blo == bhi:                        # whole enclosure rounds to one value -> CERTIFIED
            return blo, p
        p *= 2
    raise HardToRound(...)
```

Because Arb *proves* the true value lies in `[lo,hi]`, `blo == bhi` is a **soundness certificate**, not a heuristic — the strongest of the three engines. `python-flint` is therefore the recommended primary #2 engine, is **mandatory for `clog`** (mpmath's `mpc` alone would leave clog with one engine), and is the documented erf fourth leg for slice 1. Complex path: `acb(re, im)`, `acb.log()`, then round each lane `[re, im]` to f32 **independently** via `correctly_round`, concatenate to the 8-byte c32 expected.

## 6. Special values (edge tags: NaN-propagation, signed-zero, domain boundary, overflow, near-pole)

Transcendental *series* code is undefined or divergent at special inputs, so a special-value **pre-pass** runs before Ziv. `special_value(op, dtype, inputs)` is a table **transcribed from C99 Annex F.10 (real) and Annex G.6 (complex)** — its independence is that it comes from the ISO C standard, an external authority distinct from both the Rust `semantics.rs` and the engines:

Real (both f32/f64):
- `exp`: `exp(±0)=1`; `exp(+inf)=+inf`; `exp(-inf)=+0`; `exp(NaN)=qNaN`; `exp(x)` for `x` above the overflow threshold → `+inf` (overflow tag); far-negative → `+0`/subnormal (underflow — falls through to normal Ziv path, not special).
- `log`: `log(+0)=log(-0)=-inf`; `log(1)=+0`; `log(x<0)=qNaN`; `log(+inf)=+inf`; `log(-inf)=log(NaN)=qNaN`.
- `sin`: `sin(±0)=±0` (**sign preserved** — signed-zero tag); `sin(±inf)=qNaN`; `sin(NaN)=qNaN`. (Near-k·π and large-|x| are *finite* results computed on the normal Ziv path via Payne–Hanek inside the engines, not specials.)

Complex `clog` (C99 Annex G.6, the exact-sign lanes `compare_c32_transcendental` enforces): e.g. `clog(-0+0i) = -inf + iπ`; `clog(+0+0i) = -inf + i0`; `clog(x+i·inf) = +inf + i·π/2`; `clog(-inf+i·(finite>0)) = +inf + iπ`; conj symmetry `clog(conj z)=conj(clog z)`. Zero-valued lanes and ±π lanes are pinned **exactly** (sign included).

For a special cell: compute `want = special_value(...)`; for each engine that *defines* the special (mpmath returns `mpf('-inf')`, `mpf('+0')` etc.; Arb has inf/NaN balls), assert `engine_special == want`; then assert `want == stored`. Where an engine leaves it undefined/raises, that engine is skipped for that cell (recorded), but the C99-table value still gates the stored bits and at least one engine must corroborate. NaN payload: the gate asserts *qNaN with the pinned sign convention*; it does not over-pin an arbitrary NaN payload beyond what the dtype/`semantics.rs` fix.

## 7. Exact-byte oracle cells (`add`, signed-zero)

Kept from the stub but hardened: `add`/`sub`/`mul` of two f32 inputs are **exact in f64** (24+24-bit operands: sum grows ≤1 bit, product = 48 bits, both < 53), so compute in Python f64 (exact) then round the exact f64 result to the cell dtype through `correctly_round` — a *single* rounding, no double-round. Assert `== stored`. (`div` and later exact-byte ops that are not f64-exact must instead go through an engine; flagged for slice 1, not slice 0.) Signed-zero results (`(-0)+(-0)=-0`, `(-0)+(+0)=+0`, `1+(-1)=+0`) are pinned by exact bytes — `correctly_round(sign=..., man=0,...)` preserves the sign the IEEE rule dictates.

## 8. The anchor gate (third leg + presence requirement)

`load_anchors()` reads two pinned, checked-in tables under the dev-only tree:
- `tools/corpus-validation/anchors/lefevre_muller.json`: `[{op, dtype, input_hex, expected_hex, ulp_run_bits, cite}]` — the published binary64/binary32 hardest-to-round cases for `exp/log/sin/cos/atan` (Lefèvre–Muller, *Arith-15* 2001 / *Handbook of Floating-Point Arithmetic*). Each row is a **fact** carrying its citation. Slice-0 uses the `exp/log/sin` rows.
- `tools/corpus-validation/anchors/c99_annexg_clog.json`: the clog branch-cut/signed-zero rows (from ISO C99 Annex G.6), same shape for c32.

Two obligations:
1. **Value agreement (Leg 3).** In `certify_cell`, after the two engines agree and match `stored`, if `anchors.match(op,dtype,inputs)` hits, assert `anchor.expected_hex == stored`. A mismatch means mpmath+MPFR agreed on a value that Lefèvre's independent implementation contradicts → **hard fail** (the exact shared-bug case the third source exists to catch). `anchor.expected_hex` is compared *raw*, decoded by the same `clean_bytes`.
2. **Presence (`assert_required_anchors_present`).** Every anchor row whose `op` is in the bundle's declared atoms MUST appear as an oracle cell in the corpus (matched by `(op,dtype,input)`). A missing anchor fails under `--strict-anchors` (the freeze setting); without the flag it's a loud warning (useful mid-authoring). This is what makes "agreement certifies rounding, meaningful only because the hard stressors are present" enforceable at gate time rather than trust.

## 9. CORE-MATH hard-case cross-check (adversarial engine agreement)

`load_hard_cases(dir, seed)` reads the vendored, **dev-only, git-tracked-but-not-shipped** subsamples under `tools/corpus-validation/hard-cases/` (`exp.wc.sample`, `log.wc.sample`, `sin.wc.sample`), each a deterministic subsample (recorded `seed`) of CORE-MATH's MIT-licensed `.wc` worst-case files (the full `exp.wc` is >10 MB — never vendored whole, never under `conformance/corpus/`, so consumers never pull them). `SUBSAMPLE.json` records `{upstream_url, upstream_sha256, seed, n_per_atom, license: "MIT"}`.

`cross_check_hard_cases` runs BOTH engines over each hard-case *input* (the `.wc` supplies adversarial arguments; the correctly-rounded result is derived by the engines, not trusted from the file) and asserts **engine1 CR == engine2 CR** for every one. This is pure engine-vs-engine adversarial validation on the hardest known arguments: if MPFR and mpmath ever diverge, it surfaces here on inputs designed to expose exactly that. Inputs that *also* appear as corpus cells are additionally value-checked against `stored` in §1's loop. A `.wc` line parser tolerates CORE-MATH's format (hexfloat or raw 64-bit hex argument; trailing hardness columns ignored).

## 10. Independence guards (engines must not share constants)

`check_engine_independence(engines)` runs at startup, before any cell:
- **Provenance:** assert the engines come from *different* importable packages (`mpmath` vs `gmpy2`/`flint`); refuse to count two views of the same library as two engines.
- **Constant spot-check.** `CONST_REFS` pins high-precision **decimal literals** (≥400 significant digits, each with a citation — these are facts) for `pi`, `ln2`, `two_over_pi`, `e`, `log_sqrt_2pi`, `two_over_sqrt_pi`. For each constant: build the neutral reference `ref = mpf(literal)` at 1024 bits (from the *literal string*, independent of any engine's internal computation), then have each engine compute the constant *itself* (`mpmath.pi` at prec; `gmpy2.const_pi()`; `flint.arb.pi()`) and assert `|engine_const − ref| < 2**-256`. If two engines matched each other but disagreed with the literal, this catches it; if an engine cannot produce a constant, it is skipped for that constant (recorded). **The Rust oracle's own hard-coded π/ln2/(2/π) are deliberately NOT imported** — the validator is language-neutral and the pinned decimal literal is the neutral third witness for constants, mirroring how the engines are the neutral witnesses for values.
- This directly discharges the design's "engines share constants ⇒ shared-constant bug passes silently" risk.

## 11. Version pinning and reproducible provenance

`emit_provenance` writes `tools/corpus-validation/PROVENANCE.txt` (and echoes to stdout) so "validated once" is reproducible:
- `python` version + platform;
- per engine: package version + underlying lib: `mpmath.__version__`; `gmpy2.version()`, `gmpy2.mp_version()` (GMP), `gmpy2.mpfr_version()` (MPFR); `flint.__version__` + Arb/FLINT version;
- `sha256` of each `hard-cases/*.sample` + `SUBSAMPLE.json` (`upstream_sha256`, `seed`);
- `sha256` of both anchor tables;
- `sha256` of each validated bundle + its `generator` string;
- pass/fail counts and any skipped/`HardToRound`/warned cells.

A committed `tools/requirements-dev.txt` pins exact versions (`mpmath==1.4.1`, `python-flint==<pin>`, `gmpy2==<pin>`) so a re-run reproduces the same certification. These are **dev-only** — nothing here enters the shipped conformance crate (still stdlib-only Rust).

## 12. CLI, exit codes, file layout

```
validate_corpus.py [--engine {auto,gmpy2,flint,both}] [--hard-cases DIR] [--seed N]
                   [--emit-provenance FILE] [--strict-anchors] [--allow-mpmath-only]
                   BUNDLE [BUNDLE...]
```
- `--engine auto` (default): pick `gmpy2` if importable else `python-flint`; complex cells always route to `flint` (abort if clog cells exist and flint absent). `both`: run all three real engines (mpmath+gmpy2+flint) for maximum assurance.
- Exit **0** all oracle cells certified; **1** any mismatch/HardToRound/anchor failure; **2** cannot certify (fewer than two independent engines and not `--allow-mpmath-only`).

Dev-only tree (added; none shipped in `conformance/corpus/`):
```
tools/
  validate_corpus.py                 # rewritten gate
  requirements-dev.txt               # pinned engine versions
  corpus-validation/
    README.md                        # how to run; Windows=flint; independence rationale
    PROVENANCE.txt                   # emitted each run
    anchors/lefevre_muller.json      # L-M real worst cases (facts, cited)
    anchors/c99_annexg_clog.json     # C99 Annex G clog branch/signed-zero table
    hard-cases/{exp,log,sin}.wc.sample
    hard-cases/SUBSAMPLE.json        # upstream sha, seed, n, MIT license
```
`test_validate_corpus.py` is extended: keep the existing "frozen bundle validates / corrupted expected rejected" cases, add (a) a transcendental oracle bundle certifies green under `--engine flint` (or skipTest if no 2nd engine present — so the suite still passes on a bare machine), (b) a deliberately off-by-1-ULP `exp` expected is rejected, (c) a signed-zero clog lane flip is rejected, (d) `check_engine_independence` fails if a constant literal is corrupted.

**Test vectors:**
- `exp, f64, x=1.0 (3F F0 00 00 00 00 00 00)` -> `40 05 BF 0A 8B 14 57 69`  Certifies mpmath+2nd engine agree with stored on the canonical exp; verified CR via mpmath 1.4.1
- `exp, f32, x=1.0 (3F 80 00 00)` -> `40 2D F8 54`  f32 leg; matches the design doc tcId-3 example
- `log, f64, x=2.0 (40 00 00 00 00 00 00 00)` -> `3F E6 2E 42 FE FA 39 EF`  log2 CR f64; also an independence spot-check target (ln2)
- `log, f32, x=2.0 (40 00 00 00)` -> `3F 31 72 18`  f32 log leg
- `sin, f64, x=1.0 (3F F0 00 00 00 00 00 00)` -> `3F EA ED 54 8F 09 0C EE`  sin CR f64; verified via mpmath high-precision
- `sin, f32, x=1.0 (3F 80 00 00)` -> `3F 57 6A A4`  f32 sin leg
- `exp, f64, x=-inf (FF F0 00 00 00 00 00 00)` -> `00 00 00 00 00 00 00 00`  Special-value pre-pass: exp(-inf)=+0 (C99 F.10); +0 sign exact
- `log, f64, x=1.0 (3F F0 00 00 00 00 00 00)` -> `00 00 00 00 00 00 00 00`  Special: log(1)=+0 exact
- `sin, f64, x=-0.0 (80 00 00 00 00 00 00 00)` -> `80 00 00 00 00 00 00 00`  Signed-zero tag: sin(-0)=-0, sign PRESERVED — a normalize-to-+0 impl fails
- `sin, f64, x=1e300 (7E 37 E4 3C 88 00 75 9C)` -> `BF EA 2C 16 B0 10 E3 85`  Large-|x| trig: requires Payne-Hanek reduction inside the engine at >=256 bits; value is the CR result the engines derive (illustrative — engines are authoritative, not glibc)
- `clog, c32, z=1+0i (re 3F 80 00 00, im 00 00 00 00)` -> `00 00 00 00 · 00 00 00 00`  re=0.5*log(1)=+0, im=atan2(+0,1)=+0; both lanes +0 exact (split comparator)
- `clog, c32, z=-1+0i (re BF 80 00 00, im 00 00 00 00)` -> `00 00 00 00 · 40 49 0F DB`  re=+0, im=pi (branch endpoint, +pi sign exact); C99 Annex G anchor for clog
- `add (exact-byte oracle), f32, (-0.0)+(-0.0)` -> `80 00 00 00`  Native f64-exact recompute leg preserved; -0 sign pinned (regression-guards the stub's teeth)

**Constants required:** Pinned decimal literals (>=400 sig digits, cited) in CONST_REFS for the independence spot-check: pi, ln2 (=log 2), two_over_pi, e, log_sqrt_2pi (=log(sqrt(2*pi))), two_over_sqrt_pi (=2/sqrt(pi)); DTYPE table: f64=(exp_bits 11, frac_bits 52, bias 1023); f32=(8, 23, 127) — the only params correctly_round needs for slice 0; Ziv precision schedule: p0=128 bits, doubling, pmax=4096 (>> the 256/512/1024 the Rust core uses; non-termination is a flagged failure); Lefevre-Muller published worst cases for exp/log/sin (Arith-15 2001 / Handbook of Floating-Point Arithmetic) -> tools/corpus-validation/anchors/lefevre_muller.json; C99 Annex G.6 clog branch-cut / signed-zero rows -> tools/corpus-validation/anchors/c99_annexg_clog.json; C99 Annex F.10 special-value rules for exp/log/sin (transcribed into special_value()); CORE-MATH .wc worst-case inputs (MIT) subsampled with recorded seed -> tools/corpus-validation/hard-cases/{exp,log,sin}.wc.sample + SUBSAMPLE.json; Pinned engine versions in tools/requirements-dev.txt: mpmath==1.4.1 (confirmed present), python-flint==<pin>, gmpy2==<pin>
**Edge cases:** provenance != 'oracle' (promoted-differential, negative) -> skipped, never certified; unknown provenance -> hard fail; Special inputs bypass Ziv: exp(+-0)=1, exp(+inf)=+inf, exp(-inf)=+0, exp(NaN)=qNaN, exp overflow -> +inf; log(+-0)=-inf, log(1)=+0, log(x<0)=NaN, log(+inf)=+inf, log(-inf)/log(NaN)=NaN; sin(+-0)=+-0 with sign PRESERVED (signed-zero tag), sin(+-inf)=NaN, sin(NaN)=NaN; Large-|x| trig (sin(1e300)) and near-k*pi: finite results requiring Payne-Hanek INSIDE the engine at high prec; naive float() would be ~100% wrong — Ziv+exact-triple rounding required; Exact rounding midpoint / genuine tie: Ziv never stabilizes by pmax -> HardToRound = FAILURE (flag for human), never silently accepted; Subnormal results and underflow-to-zero: correctly_round handles the subnormal significand path; round-up out of subnormal promotes to smallest normal; clog signed-zero lanes and +-pi branch endpoints must match sign EXACTLY (mirrors compare_c32_transcendental); a +0 where -0 is pinned is a fail; clog needs a complex engine: mpmath mpc + python-flint acb — gmpy2 has NO complex, so gmpy2-only cannot certify clog (abort); Only one real engine importable (this machine: mpmath yes, gmpy2/flint no) -> exit 2 unless --allow-mpmath-only (non-certifying smoke mode, printed loudly); Engine constant disagreement with pinned literal -> abort before any cell (shared/wrong-constant guard); Double-rounding trap: never float() an mpf/mpfr/acb; always extract exact (sign,man,exp) and round once. Exact-byte add uses f64 only because f32+f32 is exact in f64; L-M anchor present in bundle but stored value contradicts published bytes -> hard fail (the shared-bug case); anchor missing from corpus -> fail under --strict-anchors
**Open questions:** Exact Ziv pmax and p0 tuning: 4096 cap is generous vs the Rust core's 1024 ceiling — confirm no legitimate slice-0 transcendental input needs >1024, so a HardToRound is always a real red flag not a tuning artifact; python-flint vs gmpy2 as the canonical committed second engine for the reproducible PROVENANCE header — flint covers real+complex and installs cleanly on Windows (this machine has neither yet); recommend flint as primary, gmpy2 as optional third under --engine both; CORE-MATH .wc subsample size per atom and the seed — deferred to slice 1 per design open-question; slice 0 can ship a tiny hand-picked hard-cases set to prove the cross-check plumbing; Whether --strict-anchors is on by default at freeze (recommended) vs opt-in; and how many L-M rows per atom are minimally required present for the anchor gate to be non-vacuous; NaN payload policy: gate asserts qNaN + pinned sign only, or pin full payload where semantics.rs fixes it — confirm the corpus does not over-pin an arbitrary payload the engines cannot reproduce; Whether to add Arb (python-flint) as a MANDATORY erf fourth leg now (design defers erf to slice 1) — slice 0 has no erf so this stays documented-but-unused

---
