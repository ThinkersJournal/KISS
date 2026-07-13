# KISS reference conformance harness

Non-normative. This crate is *a* conformant implementation of the KISS
specification with **no privilege and no exemption** (KISS-Conform §0). The
normative source is the specification text under [`../spec/`](../spec); where this
code and a clause disagree, the clause governs.

## License

**MIT OR Apache-2.0** — the reference-implementation license (umbrella §9.2),
distinct from the specification text under `spec/`, which is CC0.

## What it verifies today

Two of the four KISS-Conform test modalities — **golden byte-vectors** (§6.4;
exact-byte comparator §6.8) and **negative/decline vectors** (§6.7) — over the
three POD-tier encodings. Every golden vector is transcribed from the spec's own
appendix; a reference codec must reproduce it byte-for-byte, and a reader must
refuse malformed input with a typed decline, never a panic.

- **KISS-Ops OpAttrs** ([`opattrs`], Ops §6.19): all 13 Appendix E golden vectors
  — the per-op schemas, the rank-aware `reduce_axes` precedence, and the
  **default-resolution byte-equality** that lets Grammar byte-compare an opaque
  blob (§6.19-0005) — plus reserved-ordinal / reserved-band / truncation declines.
- **KISS-Classify `structure_key`** ([`structure_key`], Classify §6.7): the 10
  Appendix A golden tokens, each checked in both directions (`to_token` and
  `from_token` round-trip byte-for-byte, §6.7-0008), plus structural declines
  (bad field count, version, work-class, rank, operand sub-key, uppercase hex).
- **KISS-Announce envelope** ([`announce`], Announce §6.1): the §2.5 reference
  56-byte handshake bytes, plus the §6.2 hard-reject discipline (bad magic,
  unknown version, non-zero reserved regions, profile-array violations).

This is the first point at which KISS byte-exactness claims are **proven on a
machine** rather than asserted on paper — the identity primitive (`structure_key`),
the handshake, and the newest encoding all pass their own golden vectors.

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
- **Phase 3** — the independent CPU-oracle differential harness (Conform §6.5):
  op *semantics* (not just wire bytes) checked against a from-scratch oracle that
  shares no lowering code with any generator.
- **Phase 4** — the IR-DAG fuzzer emitting to every backend (Conform §6.6;
  device-touching).

## Keeping the vectors in sync

The golden vectors are transcribed from the specification's own appendices. If a
spec encoding changes, the vectors here change with it — and any drift surfaces
as a failing test. That is the point: the harness makes the spec's byte-level
claims executable.
