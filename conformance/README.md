# KISS reference conformance harness

Non-normative. This crate is *a* conformant implementation of the KISS
specification with **no privilege and no exemption** (KISS-Conform §0). The
normative source is the specification text under [`../spec/`](../spec); where this
code and a clause disagree, the clause governs.

## License

**MIT OR Apache-2.0** — the reference-implementation license (umbrella §9.2),
distinct from the specification text under `spec/`, which is CC0.

## What it verifies today

All four KISS-Conform test modalities — **golden byte-vectors** (§6.4),
**negative/decline vectors** (§6.7), the **independent CPU-oracle differential**
(§6.5), and a **real on-device run** (§6.6) — under the determinism-class
comparators of §6.8 (exact-byte, ULP-tolerance, and order-invariant). Every golden
vector is transcribed from the spec's own appendix.
**127 tests pass by default; 3 more on-device under `--features cuda`.**

- **KISS-Ops OpAttrs** ([`opattrs`], Ops §6.19): all 13 Appendix E golden vectors
  — the per-op schemas, the rank-aware `reduce_axes` precedence, and the
  **default-resolution byte-equality** that lets Grammar byte-compare an opaque
  blob (§6.19-0005) — plus reserved-ordinal / reserved-band / truncation declines.
- **KISS-Classify `structure_key`** ([`structure_key`], Classify §6.7): the 10
  Appendix A golden tokens, each checked in both directions (`to_token` and
  `from_token` round-trip byte-for-byte, §6.7-0008), plus declines — structural
  (field count, version, work-class, rank, operand sub-key, uppercase hex) **and
  closed-set membership** (the 24 op-family codes of §6.5-0006 and the 17 dtype
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
- **On-device real-kernel differential** ([`tests/device.rs`], §6.6; opt-in
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
cargo test                     # 127 tests, CPU, no dependencies
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

## Keeping the vectors in sync

The golden vectors are transcribed from the specification's own appendices. If a
spec encoding changes, the vectors here change with it — and any drift surfaces
as a failing test. That is the point: the harness makes the spec's byte-level
claims executable.
