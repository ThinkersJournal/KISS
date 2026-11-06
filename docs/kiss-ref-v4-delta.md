# kiss-ref: what changes from sk5 / contract v3 to sk6 / contract v4

Measured 2026-10-30 against KISS `origin/main` 51153c6e (conformance 0.6.34) and kiss-ref `main` 924aa17
(workspace 0.4.5). ROADMAP R3.17 (sizing; the PM assigned the row to the KISS lane, 2026-10-30).

## What kiss-ref binds

kiss-ref (`C:\Projects\kiss-ref`, four crates: `kiss-classify-vocab`, `kiss-ops-vocab`, `kiss-ref-core`,
`kiss-ref-conformance`) vendors 15 of the 22 files in `conformance/corpus/` (`PROVENANCE.md`: KISS `9ea8c05`
plus two files from `034a4b1`, "`sk5` / `contract_version` 3"). It has **no Contract, Announce or
`structure_key` code**: `git grep -n "contract_version\|math_fidelity\|reduced-mantissa" crates` finds no `.rs`
hit (control: the same tree contains 26,139 lines of `.rs`). So the wire-side changes below do not reach it.

## Delta, by what kiss-ref actually vendors

Compared by `git hash-object` of each vendored file against `origin/main:conformance/corpus/<file>`:

| vendored file | status at 51153c6e |
|---|---|
| `op_manifest.json`, the 13 `ops-*.json` vector files | **byte-identical** (14 of 15) |
| `dtype_manifest.json` | **differs**: `structure_key_schema_version` 6, `token_prefix` `sk6`, 27 dtype rows (kiss-ref has 24: the MX element formats `f4e2m1`, `f6e2m3`, `f6e3m2` landed in #675, verified against OCP MX v1.0 in #681) |

Not vendored, not read by kiss-ref, so no kiss-ref change: `announce_vectors.json` (list_version 2,
`correlation_len`), `contract_vectors.json` (`contract_version` 4, declared-width `math_fidelity`),
`structure_key_vectors.json` / `structure_key_derivation_vectors.json` (`sk6`), `grammar_vectors.json`,
`opattrs_*`.

## Work, if kiss-ref moves to sk6

1. Re-vendor `dtype_manifest.json` and update `PROVENANCE.md` (one file, minutes).
2. Add three dtypes to `kiss-classify-vocab` with decode/encode tests; the existing
   `dtype_manifest_agreement.rs` test fails until the vocab and manifest agree, which is its job.
3. Bump the doc comments that say "unchanged at sk5" (`kiss-classify-vocab/src/lib.rs:6`).

**Estimate: 3-5 lane-hours**, dominated by the three float formats' value tables (bit-exact against OCP MX
v1.0, as KISS did in #681). This is an estimate; I have not built it.

## Larger, separate scope (not a delta)

A kiss-ref reader of the Contract or Announce wire (to witness `contract_version` 4 and `correlation_len`)
would be new code, not a version move; R1.31 (consumer-loader witness) is held by the PM on exactly that
ground. Size not attempted.

## What Unpopped's re-derivation needs from kiss-ref

Unverified: I did not read Unpopped's code. Its keyer re-derivation to `sk6` is driven by KISS's own
`structure_key_*_vectors.json` (kiss-ref does not vendor them), so from kiss-ref it needs at most the three new
dtype spellings in the vocabulary crate. If Unpopped depends on `kiss-classify-vocab` for dtype parsing, step 2
above is on its critical path; ask Unpopped lane 1.

## Held behind this (sk7)

#724/#725 will move the line again (`sk7`; `cast`/`concat` added, `transpose`/`unsqueeze` removed;
`docs/sk7-vector-changes.md`). Moving kiss-ref to sk6 and then sk7 is two vendorings; one vendoring at the sk7
batch close is cheaper, and kiss-ref's op vocabulary would change then (`op_manifest.json`).
