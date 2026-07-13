# KISS reference conformance harness

Non-normative. This crate is *a* conformant implementation of the KISS
specification with **no privilege and no exemption** (KISS-Conform §0). The
normative source is the specification text under [`../spec/`](../spec); where this
code and a clause disagree, the clause governs.

## License

**MIT OR Apache-2.0** — the reference-implementation license (umbrella §9.2),
distinct from the specification text under `spec/`, which is CC0.

## What it verifies today

Phase 1 implements two of the four KISS-Conform test modalities for the KISS-Ops
**OpAttrs canonical wire encoding** (Ops §6.19):

- **Golden byte-vectors** (Conform §6.4; exact-byte comparator §6.8): a reference
  encoder reproduces all 13 exact little-endian vectors of Ops Appendix E,
  byte-for-byte — including the rank-aware `reduce_axes` precedence and the
  **default-resolution byte-equality** that lets KISS-Grammar byte-compare an
  opaque blob (§6.19-0005).
- **Negative / decline vectors** (Conform §6.7): a reader refuses a malformed
  blob (a reserved-`0` ordinal, the reserved `reduce_axes` band, truncation) with
  a typed decline — never a panic.

This is the first point at which a KISS byte-exactness claim is **proven on a
machine** rather than asserted on paper.

## Run

```sh
cd conformance
cargo test
```

No dependencies (standard library only) — a conformance harness must share no
lowering code with any implementation under test (Conform §6.5), so it starts
dependency-free.

## Roadmap

- **Phase 2** — the `structure_key` codec (Classify) and the Announce 56-byte
  envelope (more POD golden vectors + decline vectors).
- **Phase 3** — the independent CPU-oracle differential harness (Conform §6.5):
  op semantics checked against a from-scratch oracle that shares no code with any
  generator.
- **Phase 4** — the IR-DAG fuzzer emitting to every backend (Conform §6.6;
  device-touching).

## Keeping the vectors in sync

The golden vectors are transcribed from the specification's own appendices. If a
spec encoding changes, the vectors here change with it — and any drift surfaces
as a failing test. That is the point: the harness makes the spec's byte-level
claims executable.
