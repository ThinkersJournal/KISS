# KISS reference conformance harness

Non-normative. This crate is *a* conformant implementation of the KISS
specification with **no privilege and no exemption** (KISS-Conform §0). The
normative source is the specification text under [`../spec/`](../spec); where this
code and a clause disagree, the clause governs.

## License

**MIT OR Apache-2.0** — the reference-implementation license (umbrella §9.2),
distinct from the specification text under `spec/`, which is CC0.

## What it verifies today

Three of the four KISS-Conform test modalities — **golden byte-vectors** (§6.4),
**negative/decline vectors** (§6.7), and the **independent CPU-oracle differential**
(§6.5) — under the determinism-class comparators of §6.8 (exact-byte and
ULP-tolerance). Every golden vector is transcribed from the spec's own appendix.

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
  differential oracle for the pinned float primitives, making the load-bearing
  distinctions executable — NaN-**propagating** `max_prop`/`min_prop` vs
  NaN-**suppressing** `fmax_ieee`/`fmin_ieee`, and `relu` ≠ `max(x,0)` (NaN-keeping,
  −0.0-preserving) — plus the §6.8 "declared-ULP, not bit-identity" model for a
  transcendental (an f64 oracle vs the f32-native `exp` agree within a declared ULP).

These are the points at which KISS's claims are **proven on a machine** rather than
asserted on paper: the identity primitive, the handshake, the newest wire encoding,
and now the numeric *semantics* — the bytes are right, and the computation is right.

## Run

```sh
cd conformance
cargo test
```

No dependencies (standard library only) — a conformance harness must share no
lowering code with any implementation under test (Conform §6.5), so it starts
dependency-free.

## Roadmap

- **Phase 1–2 (done)** — golden + decline vectors for the three POD encodings
  (OpAttrs, `structure_key`, Announce envelope).
- **Phase 3 (started)** — the independent CPU-oracle differential harness
  (Conform §6.5): the scalar float primitives are covered here; broadening to the
  full primitive floor and driving random inputs is the remaining work.
- **Phase 4** — the IR-DAG fuzzer emitting to every backend (Conform §6.6;
  device-touching), and differencing a real generated kernel against the oracle.

## Keeping the vectors in sync

The golden vectors are transcribed from the specification's own appendices. If a
spec encoding changes, the vectors here change with it — and any drift surfaces
as a failing test. That is the point: the harness makes the spec's byte-level
claims executable.
