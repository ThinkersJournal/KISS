# Plan: the KISS key and contract formats as interface crates in FireLine (board 295)

**Informative. A plan, not a decision; nothing here changes a spec byte.** CireSnave 2026-11-07, via the PM: "Is 295 waiting for me to tell you whether we should create a plan first?...or am I missing something?  If so, create a plan for each." Board 295's idea: interface crates, both sides, move into one project with **a crate per interface** so nothing drifts. Agreed rules (PM): one project (FireLine, `ThinkersJournal/fireline`), a crate per interface; each crate's version follows its interface; a common number at major releases (first the KISS freeze, then `1.0.0-rcN` together); consumers require with `~` or an exact version, never `^`. Read at KISS `origin/main` `d47b99e` (0.8.1) and kiss-ref `0.4.7`.

## What exists today

- **One crate in KISS**, `kiss-conformance` (`publish = false`, standard library only, MIT OR Apache-2.0): the reference **codecs** (`structure_key.rs`, `grammar.rs` region wire, `contract.rs` text, `announce.rs`, `opattrs*.rs`, `shape_expr.rs`), the reference **evaluators** (`semantics.rs`, `cast.rs`, `structural.rs`, `window.rs`), the conformance harness, and the generators of the **vectors** in `corpus/` (24 JSON files at that ref). Beside it: the spec (`spec/`, CC0) and the checkers (`tools/*.py`).
- **Four crates in kiss-ref** (`ThinkersJournal/kiss-ref`, 0.4.7): `kiss-classify-vocab` (dtype vocabulary, 27 dtypes) and `kiss-ops-vocab` (op vocabulary, 122 ops), both published and generated from the vendored `dtype_manifest.json` / `op_manifest.json`; `kiss-ref-core` (evaluators); `kiss-ref-conformance` (runs the corpus).
- **Sub-standards with implementing code in KISS:** Classify (structure_key codec), Grammar (region codec), Contract (text codec), Ops (OpAttrs codecs, shape expressions, evaluators), Announce (codec), Conform (the harness). **Spec only, no implementing crate:** Synth, Emit, Consume, Umbrella (their obligations are checked by lints and clause tests, not by a codec).

## What would move, what stays

| Thing | Verdict | Why |
|---|---|---|
| `structure_key` codec (Classify) | **moves**: `fireline-kiss-key` | an interface implementation both sides compile |
| region wire codec (Grammar) | **moves**: `fireline-kiss-region` | same |
| contract text codec (Contract) | **moves**: `fireline-kiss-contract` | same |
| announce codec (Announce) | **moves**: `fireline-kiss-announce` | same |
| OpAttrs and shape-expression codecs (Ops) | **moves**: `fireline-kiss-opattrs` | same |
| `kiss-classify-vocab`, `kiss-ops-vocab` | **move** (republish under the FireLine names; old names deprecated, one release overlap) | vocabulary is interface, generated from the manifests |
| spec, `corpus/` vectors, `tools/` checkers, KISS-Conform clauses | **stay in KISS** | they are the standard |
| reference **evaluators** (`semantics.rs`, `cast.rs`, kiss-ref-core) | **stay** | the independent oracle must not share code with what it checks (KISS-Conform §6.5) |
| the harness | **stays**, stdlib-only | same reason |

## One source of truth, not two

Truth flows one way: **spec, then vectors (KISS), then FireLine crates**. A FireLine crate's CI runs the KISS vectors for the KISS version it pins; **KISS CI never imports FireLine**, so the oracle stays independent. This leaves two codebases for each codec **on purpose** (KISS's stdlib-only reference and FireLine's product crate) but one source of *truth*: a FireLine crate that disagrees with the vectors is the defective one. Generated tables (dtype and op manifests) are produced by KISS tools and vendored into FireLine verbatim, with the provenance record the vendoring already has (a file that fails its agreement test is never merged).

## First release: what changes for whom

The first FireLine release should **follow the 0.9.0 batch** (Source kind, keepdim and stability flags, `Max`, `norm_axes`, and the layout and squeeze/unsqueeze work), not precede it: those are Breaking wire and layout changes, and a crate released just before them breaks its consumers within a week. Fuel, Baracuda and Unpopped replace their private KISS type copies with `~` requirements on the FireLine crates: a one-time dependency change plus the 0.9.0 reader work they owe anyway. kiss-ref re-points its two vocab crates at FireLine and keeps `kiss-ref-core` and `kiss-ref-conformance` (the independent side); its pending evaluator work (kiss-ref#61) is unaffected.
**Breaks:** every consumer's `Cargo.toml`; the published names `kiss-classify-vocab` and `kiss-ops-vocab` (deprecated, not yanked); the "vendor the JSON" path becomes "depend on the crate" for the two vocabularies.

## What a suite both sides run adds

Today each side runs the vectors against **its own** codec, so a vector proves "my codec equals the spec" but not "my output is your input". A shared suite adds: **cross-reads** (A's writer into B's reader, both directions, per codec); **version skew** (a `~`-pinned reader against the previous writer, which is exactly what the pin rule promises); **decline agreement** (the same malformed bytes refused with the same typed decline); and a **drift alarm** when a consumer's private type diverges from the interface crate. It does not replace the vectors, which stay the standard's normative examples.

## Decisions for CireSnave

(1) Does the first FireLine release wait for the 0.9.0 batch (recommended)? (2) Do the evaluators stay out of FireLine (recommended, for oracle independence)? (3) Republish the two vocab crates under FireLine names, or keep their names and move only the repository? I recommend rename, because "one crate per interface" is the rule.
