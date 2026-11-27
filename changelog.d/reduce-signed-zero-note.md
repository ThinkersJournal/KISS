kind: docs: options note, which zero a float reduce(max|min) returns (no bytes)
---
Adds `docs/options-reduce-max-min-signed-zero.md`: the gap (`spec/ops.md` 6.0-0002 files max/min reduce exact-byte; no clause pins the sign of a zero result; the harness already canonicalises it in `conformance/src/structural.rs`), consumers, options A/B/C and a recommendation (B). Options only: no clause, no corpus vector, no stored byte moves.
