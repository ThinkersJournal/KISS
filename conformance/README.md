# KISS reference conformance harness

Non-normative. This crate is *a* conformant implementation of the KISS
specification with **no privilege and no exemption** (KISS-Conform §0). The
normative source is the specification text under [`../spec/`](../spec); where this
code and a clause disagree, the clause governs.

## License

**MIT OR Apache-2.0** — the reference-implementation license (umbrella §9.2),
distinct from the specification text under `spec/`, which is CC0.

## What it verifies today

**Three of the four** KISS-Conform test modalities — **golden byte-vectors**
(§6.4), the **independent CPU-oracle differential** (§6.5), and
**negative/decline vectors** (§6.7) — under the determinism-class comparators of
§6.8 (exact-byte, ULP-tolerance, and order-invariant), plus a **real on-device
run** that is not one of the four modalities but exercises §6.5's differential on
real silicon. Every golden vector is transcribed from the spec's own appendix.
**538 test functions in the harness <!-- bound:test_fns=538 -->; 537 pass by default, 3 more on-device under `--features cuda`.** The discovered count is bound by `tools/kiss_readme_coverage.py`; the *passing* count is not, because reaching it requires executing the suite rather than reading it.

**Not implemented: modality 3 — the §6.6 structure-directed fuzzer**, which must
generate random-but-valid KISS-Ops IR DAGs and drive each through every backend.
Nothing here does that; the `*_under_fuzz` tests are seeded scalar corpus loops
over binary f32 functions (§6.5), not DAG generation. All 17 §6.6 clauses are in
[`UNBACKED.tsv`](UNBACKED.tsv). An earlier version of this README claimed all four
modalities and labelled the on-device run "§6.6"; §6.6 is the fuzzer.

### How much of the spec is actually executable

**380 of 932 normative clauses (40.8%) are backed by executable code**
<!-- bound:harness=380 --><!-- bound:clauses=932 --> — the figure the ratchet in
[`COVERAGE_FLOOR.tsv`](COVERAGE_FLOOR.tsv) defends on every merge. **Of those, 240 are
backed by NAME** <!-- bound:named=240 --> (the §9 row resolves to a real test fn) and the
rest **by CITATION** (some test carries a backing-form comment for the clause).

**That distinction is why this section's old figure was wrong in a way regenerating it
would not have fixed.** It read *31 of 855*, and the sentence beside it described the
clauses that "name a conformance test that does not exist" — **the NAME metric — while the
heading asks how much of the spec is executable, which is the BACKED one.** A number and a
sentence describing two different measurements, agreeing with each other only because
neither was checked.

The remaining 552 are listed in [`UNBACKED.tsv`](UNBACKED.tsv), and they are not one
population: **493 genuinely untested** <!-- bound:untested_rows=493 -->, 33 enforced by a
document lint, 20 `blocked`, 4 `untestable`, 2 `decredited`. The ledger is enforced as a
ratchet by `tools/kiss_trace.py`, whose floor tracks the untested figure separately from
the harness one for exactly this reason. Of this crate's 538 test fns, **124 cite no clause at all**
<!-- bound:uncited_tests=124 -->, so the traceability matrix cannot see them: real tests
doing real work that no clause claims credit for. Closing that is cheap and is the first
task below.

`boundary_rounding.rs` (§6.5-0006/-0007) is the model to copy: it was added with the
clause, under the name the clause names, so it counted the day it landed.

**Where an untested MUST is a hard error.** Not on every commit — a gate that can
never go green, including for the commits that would fix it, gets bypassed by habit
and decays back into a green check that means nothing. Instead an unbacked clause
hard-fails the two transitions it actually invalidates:

```sh
python tools/kiss_trace.py --freeze-ready        # all nine
python tools/kiss_trace.py --freeze-ready SYNTH  # one sub-standard
```

- **umbrella §5.3 condition 3** — a sub-standard advances Draft → Frozen only with
  "complete bidirectional clause-to-test traceability". One unbacked clause blocks
  the freeze. **Today: 0 of 9 sub-standards pass.**
- **umbrella §8.1** — an implementation conforms "if and only if it passes the
  unmodified KISS-Conform suite for that sub-standard". Where there is no test there
  is no suite, so a conformance claim to KISS-Synth (0/130) is backed by nothing.

Everywhere else the gap is a recorded, ratcheted debt: `UNBACKED.tsv` may only
shrink, and `--strict` reports the live count on every PR.

- **KISS-Ops OpAttrs** ([`opattrs`], Ops §6.19): all 13 Appendix E golden vectors
  — the per-op schemas, the rank-aware `reduce_axes` precedence, and the
  **default-resolution byte-equality** that lets Grammar byte-compare an opaque
  blob (§6.19-0005) — plus reserved-ordinal / reserved-band / truncation declines.
- **KISS-Classify `structure_key`** ([`structure_key`], Classify §6.7): the 10
  Appendix A golden tokens, each checked in both directions (`to_token` and
  `from_token` round-trip byte-for-byte, §6.7-0008), plus declines — structural
  (field count, version, work-class, rank, operand sub-key, uppercase hex) **and
  closed-set membership** (the 24 op-family codes of §6.5-0006 and the 20 dtype
  tokens of §6.1; an unknown `zzz` / `f99` is refused).
- **KISS-Announce envelope** ([`announce`], Announce §6.1): the §2.5 reference
  56-byte handshake bytes, plus the §6.2 hard-reject discipline (bad magic,
  unknown version, non-zero reserved regions, profile-array violations).
- **KISS-Ops numeric semantics** ([`semantics`], Ops §2.3/§6.15): a from-scratch
  oracle for the pinned float primitives — arithmetic atoms, sign-bit atoms
  (`neg`/`abs`/`copysign`, signed-zero exact), `sign`/`step`, rounding
  (`floor`/`ceil`/`trunc`/`round_even` banker's rounding), IEEE-ordered
  comparisons (`isnan == cmp_ne(x,x)`), and the min/max family. The load-bearing
  distinctions are executable: NaN-**propagating** `max_prop`/`min_prop` vs
  NaN-**suppressing** `fmax_ieee`/`fmin_ieee`, and `relu` ≠ `max(x,0)`. Plus the
  §6.8 "declared-ULP, not bit-identity" model (an f64 oracle vs f32-native `exp`
  within a declared ULP).
- **Randomized differential loop** ([`differential`], Conform §6.5): a candidate
  is differenced against the oracle over a **reproducible** corpus (edge cases +
  seeded-random bit patterns — same seed → same inputs). It demonstrably *catches*
  a wrong implementation: an IEEE `fmax` mistakenly built with NaN-propagating
  `max_prop` is flagged by the corpus's NaN inputs, with every divergence pinned
  to a NaN operand. A harness that only passed correct code would prove nothing.
- **Oracle boundary rounding + tightness** ([`tests/boundary_rounding.rs`], Conform
  §6.5-0006 / §6.5-0007): two oracle-hygiene disciplines from a Baracuda↔KISS review.
  A discontinuous op (`cmp_*`, a `select` condition, `sign`, `step`) is decided on the
  operand **rounded to the op's compute dtype first** — a pinned boundary golden vector
  (`1.0 + 2^-30` distinct from `1.0` in f64 but equal after narrowing to f32) shows the
  un-narrowed f64 decision flips spuriously. And the transcendental oracle is **strictly
  tighter than the declared ULP tolerance** it enforces (wider f64 eval, rounded once,
  ≤ 0.5 ULP) — a same-precision "oracle" is shown to give a vacuous 0-ULP false pass.
- **Integer atoms** ([`integer`], Ops §6.10/§6.16): wrapping two's-complement
  `add`/`sub`/`mul`/`neg`/`abs`, the bitwise atoms, and — the subtle ones —
  **arithmetic** (signed) vs **logical** (unsigned) `shr`, out-of-range shift as
  the single "target-defined" (`None`) case, and `popcount`/`clz`/`ctz` incl. at 0.
- **Structural ops** ([`structural`], Ops §6.11): multi-element oracles for
  `reduce` / `prefix_scan` / `gather` (skip/clamp/zero-fill OOB) / `scatter`
  (assign/atomic-add/atomic-max/atomic-min), and the **order-invariant comparator**
  (§6.8-0004) for the one nondeterministic op — FP scatter-atomic-add is invariant
  to visit order only up to reassociation, so it's compared within a
  contract-declared tolerance, not byte-for-byte. A differential *catches* a lossy
  scatter, and the `max_prop`-reduction signed-zero-on-tie order-dependence is pinned.
- **On-device real-kernel differential** ([`tests/device.rs`], §6.5 on real
  silicon; opt-in
  `--features cuda`): a hand-written CUDA `fmax_ieee` kernel is compiled with `nvcc`
  and run on the GPU over the corpus, differenced against the CPU oracle. On an
  RTX 4070 it passes 16.9M pairs, and a negative control using CUDA's `fmaxf`
  intrinsic is **caught** — it returns `+0.0` for `fmax_ieee(-0,+0)` where §6.15
  pins `-0.0`. **And the loop closes**: a `relu_add` kernel *emitted by the reference generator* (`baracuda-kernelgen`, not hand-written — see `cuda/generated/PROVENANCE.md`) matches the KISS `relu(a+b)` bit-for-bit over all 16.9M pairs and diverges from the naive `max(x,0)` at 172,266 of them — certifying the reference generator conformant for this op, on real silicon.

These are the points at which KISS's claims are **proven on a machine** rather than
asserted on paper: the identity primitive, the handshake, the wire encodings, the
numeric *semantics* across float and integer atoms and structural ops, and a real
GPU kernel — the bytes are right, the computation is right on CPU and on device,
and randomized loops catch the mistakes.

## Run

```sh
cd conformance
cargo test                     # 130 tests, CPU, no dependencies
cargo test --features cuda     # + 3 on-device tests (needs nvcc + an NVIDIA GPU)
```

No crate dependencies (standard library only) — a conformance harness must share no
lowering code with any implementation under test (Conform §6.5), so it starts
dependency-free. The `cuda` feature adds **no** crate dependency either: the
on-device test shells out to `nvcc` and skips gracefully if it is absent, so the
default build and CI stay GPU-free.

## Roadmap

- **Phase 1–2 (done)** — golden + decline vectors for the three POD encodings
  (OpAttrs, `structure_key`, Announce envelope).
- **Phase 3 (done, CPU)** — the CPU-oracle differential (Conform §6.5): the float
  primitive floor, the integer atoms, the structural ops (with the order-invariant
  comparator), and reproducible randomized differential loops that catch bugs.
- **Phase 4 (on-device, loop closed)** — real CUDA kernels differenced against
  the oracle on the GPU (`--features cuda`): a hand-written `fmax_ieee`, and — the
  closed loop — a `relu_add` kernel **emitted by the reference generator**, proven
  conformant on-device. Remaining: more generated ops and cells, and a single-source
  corpus (Rust emits → `.cu` consumes) so the two can never drift.
- **Phase 5 (next, cheap) — cite the clause in every test.** 95 of 128 test fns
  cite no clause, so ~15% of the suite's real coverage is invisible to the matrix.
  Pass the clause ID at the assertion site, as `opattrs_golden.rs` and
  `structure_key_golden.rs` already do (`assert_golden("KISS-OPS-6.19-0025", …)`),
  or name it in the comment above the test. `kiss_trace.py` reads both and will
  strike the clause from `UNBACKED.tsv`. This is annotation, not new testing, and
  it is the cheapest coverage in the repo.
- **Phase 6 — burn down `UNBACKED.tsv`.** 824 clauses, no executable test. Order
  by seam, not by document: the clauses two real implementations must agree on to
  exchange one kernel come first (see the wire-first list in the repo issues).
  0 of 9 sub-standards are at 0.0% <!-- bound:zero_coverage_subs=0 --> — the lowest is
  Synth at 31/130 and the highest Ops at 109/196; Announce is 42/76. Every one has a
  byte cross a process boundary.

## Keeping the vectors in sync

The golden vectors are transcribed from the specification's own appendices. If a
spec encoding changes, the vectors here change with it — and any drift surfaces
as a failing test. That is the point: the harness makes the spec's byte-level
claims executable.
