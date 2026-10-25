# Plan: the interface project (`<name>`)

**One page, a plan and not a decision. For CireSnave's board 295; A/B/C and the name are still open.**
PM recommends B (plan first). Written 2026-10-25 from `origin/main` of each repository named; items I could
not check say so. Coordination with the Fuel, Baracuda and Unpopped lanes is requested, not yet answered.

## What CireSnave has agreed (board 295, his words in the archive)

One project, one crate per interface, each crate's version following its interface. At a major release of
the project (the first being the KISS freeze) every crate is released at one common version. Before 1.0 a
breaking change moves the second number of that crate alone; near 1.0 all crates go through `1.0.0-rc1`,
`rc2`, ... to `1.0.0`; after 1.0 a breaking change confined to one interface raises that crate's **minor**
number ("it minorly breaks semver numbering"). Consumers require these crates with a tilde or exactly, never
a caret, and a CI check enforces that.

## What moves, what stays

| Moves into `<name>` | Stays where it is |
|---|---|
| The Fuel-to-baracuda seam as wire types, handshake and capability bits: today `fuel-kernel-seam`, `fuel-kernel-seam-types`, `fuel-kernel-seam-announce` in Fuel's workspace (the `SeamHello` envelope is defined there and again in `baracuda-seam`, which is the duplication that lets the two drift) | Anything naming Fuel tensors, graphs or devices; anything naming baracuda kernel internals |
| The Unpopped-to-emitter contract (IR and lowering contract), once Unpopped names its surface | Unpopped's IR construction and lowerings |
| The KISS key and contract formats as codecs: `structure_key`, Announce frames, Contract document | The KISS documents, vectors and the conformance checker (they are the standard) |
| A small shared-vocabulary crate at the bottom (dtypes, op tokens, namespaces) | Each project's thin adapter, unless it can be written without the owner's internals |

## Layout (one repository)

`<name>-vocab` (bottom) → per interface a `-types` crate (wire types and codec, no logic) and a `-conformance`
crate (vectors plus a checker both sides run) → `-provider` / `-consumer` helpers **only** where each can be
written without the owner's internals. A crate in this project never depends on Fuel, baracuda or Unpopped.
Each repository gets CI and branch protection that requires it (standing rule).

## One source of truth with KISS

KISS owns the **text and the vectors**; this project owns the **production codecs** that are checked against
those vectors. The `kiss-conformance` crate remains the reference oracle that *generates* the vectors, so a
codec here is correct iff it reproduces `conformance/corpus/*.json`, not iff it matches `kiss-conformance`'s
source. Open for the owner: the crates published from the `kiss-ref` repository (`kiss-classify-vocab`,
`kiss-ops-vocab`, `kiss-ref-core`) and `unpopped-vocab` overlap the vocabulary layer; I have not compared
them and do not propose which absorbs which.

## Versions

Crate versions follow their interface (Announce crates carry the Announce version). **Convergence rule:** the
next common version is one above the highest major in the project at that moment, so a crate that took its own
break between convergences is never behind. Pre-1.0 "major" means the **second** number. The project README
states: "a minor bump here may be breaking; require with `~`". The CI check fails if any project in the
portfolio requires one of these crates with a caret.

## First release and migration

One coordinated, breaking move, once: the seam types leave Fuel (Fuel's workspace currently carries all three
at its own version) and `baracuda-seam` depends on the published crate instead of its own copy. Both lanes
re-pin to `~` requirements; a conformance run on both sides proves byte identity before either releases. The
existing stopgap versions published from Fuel's branch are superseded by the first release (the PM's
description; not independently verified here).

## Tooling

The PM's publish tool assumes one workspace version. It needs: per-crate versions, publish order by the
dependency graph, a "convergence release" mode that sets one version across crates, and the caret check.

## Not decided here

A/B/C (PM recommends B: this plan first); the project name; whether Unpopped's contract moves in the first
release or the second; the comparison of the overlapping vocabulary crates above.
