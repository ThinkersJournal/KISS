# Corpus provenance — generated vectors

Some files in this directory are **generated**, not hand-written. Hand-editing one is a defect: the
next regeneration silently undoes it, and the corpus JSON is sha256-pinned.

| File | Generator | Version | Regenerate with | Verified |
|---|---|---|---|---|
| `ops-int-wrap.json` (KISS-OPS-6.2-0002, 440 exact-byte cells) | `conformance/src/bin/kiss_mint.rs` (`int_wrap_doc`) | `kiss_mint 0.1.0` (the file's own `generator` field) | `cargo run -p kiss-conformance --bin kiss_mint -- --write` (run from the repository root or `conformance/`) | at `origin/main` `294a9f2`: regenerated, `git status` clean (byte-identical); sha256 `d2215a92e3e172d0ad29bf46b6a6e9fba037b330c21200e553209c62f1153884` |

`kiss_mint` writes **all twelve** of its bundles in one run: `ops-arith`, `ops-transcendental-nan`,
`ops-narrow-move-nan`, `ops-narrow-select-nan`, `ops-rounding`, `ops-int-wrap`, `ops-compare-bool`,
`ops-atan2-quadrants`, `ops-sqrt-rounding`, `ops-nextafter`, `ops-minmax-nan`, `ops-ulp-edge` (added later, KISS-CONFORM-6.8-0002b; not part of the regeneration check below). The first eleven were
regenerated at that ref with no diff. Each of those files carries the same `generator` field; the
other corpus files here (`*_vectors.json`, the manifests, `ops-minmax-ordinary`,
`ops-minmax-signed-zero`) are **not** produced by `kiss_mint` and are not covered by this record.
`conformance/tests/mint_int_wrap.rs` recomputes every `ops-int-wrap` cell from an independent
`i128` model.
