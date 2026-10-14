# R2.4 design note — value-bearing OpAttrs vectors and a cross-implementation byte comparison

**Status: proposal, not decided.** Written 2026-10-14 against `origin/main` `e317315` (conformance 0.2.26) for
ROADMAP R2.4 (#504, #502, #23). No code accompanies it; the PM gates whether and in what slices to build it.
Every figure below names its command; anything not measured says so.

> **Correction, 2026-10-14 (after slice 1 landed and Fuel was re-read at `e6cbf782c`).** §4 below called Fuel "the
> live pair". That was wrong. Fuel's own `fuel-kernel-seam-types/tests/kiss_ops_619_divergence.rs` (GAP-305, OPEN,
> owner unallocated) states that `OpAttrs::to_canonical_bytes` is a u32-LE node envelope that diverges from §6.19.3
> on every carrier op and "is NOT an interop test". The live OpAttrs pair count is therefore still **0**, and the value
> file is an external reference for GAP-305's remedy, not a pass/fail run against a conformant second encoder. Slice 2
> was sent to Fuel as an optional body-diff against its clause-derived vectors (a second *reading*), not as a run.

## 1. The gap, restated at `e317315`

§6.19 owns the OpAttrs wire encoding (17 carrier ops, positional fields, default-resolved, LE). #504 measured on
2026-09-08 that **no party compares its OpAttrs bytes against another party's**; every guard is relative.

What exists today (measured by reading, not assumed):

| artifact | what it pins | what it cannot say |
|---|---|---|
| `spec/ops.md` §6.19.3 schema table | field names, widths, order | — (it is the source) |
| `conformance/corpus/opattrs_vectors.json` (`kiss-opattrs-spec-vectors/1`) | **layout only**: per op, field order, widths, offsets; parsed from the spec by `conformance/src/opattrs_schema.rs`. It states "NO expected VALUE for any op" on purpose | no attribute tuple maps to any byte |
| Appendix E (informative) | a few worked blobs and the sub-vocabulary ordinals | prose, not a machine-readable corpus; no equivalence or decline cases |
| `conformance/src/opattrs.rs` | hand encoders `encode_*` for **10 of the 17** carrier ops (reduce, prefix_scan, gather, scatter, sort_network, reduce_var, softmax, index_select, avg_pool, im2col) | `reduce_std`, `log_softmax`, `rms_norm`, `layer_norm`, `max_pool`, `embedding`, `scatter_add` have no hand encoder (they share a schema with a covered op, but nothing encodes them) |
| Fuel `OpAttrs::to_canonical_bytes` | the only second encoder; ~60 assertions, all against Fuel's own goldens, **0 production call sites** (#504, Fuel's measurement, not re-run here) | never compared to a KISS-produced blob |

So the missing artifact is not a harness (KISS has one, #504's control) but a **value-bearing target a second party can
be pointed at**: attribute tuple in, exact bytes out, produced by something that is not that party's own encoder.

## 2. Proposal: `opattrs_value_vectors.json`

A corpus file `kiss-opattrs-value-vectors/1` beside `opattrs_vectors.json`, three vector kinds:

1. **`encode`** — `{op, attrs: {name: value}, rank?, blob_hex, clauses[]}`. `attrs` are *resolved* values spelled with the
   sub-vocabulary tokens (`monoid: "sum"`, `oob_policy: "clamp"`, `keepdim: true`, `window_size: [3,3]`), never ordinals,
   so a consumer maps token -> its own type -> bytes. `rank` is present exactly where §6.19-0020 makes `reduce_axes`
   rank-sensitive.
2. **`equivalence`** — `{op, a: attrs, b: attrs, same_bytes: true}`; the §6.19-0005 obligation that a defaulted and an
   explicitly-stated-equal attribute produce identical bytes. (The converse, #502's *injectivity*, is the `distinct` kind below.)
3. **`distinct`** — `{op, a, b, same_bytes: false}` per op: two attribute tuples that must NOT collide. This is what #502
   asks to be a per-op clause; the vector exists whether or not the clause is written first.
4. *(optional, phase 2)* **`decline`** — a blob a reader must reject: reserved ordinal `0`, `reduce_axes = 0x0000`
   (unreachable, §6.19-0038), length not matching the schema, over-`MAX_RANK` subset bits.

### Population (sized from the schema table, not guessed)

- 17 ops. Per op: one all-defaults vector; one vector per enum value of each enum field (monoid x4, oob x3,
  scatter_combine x4, index_dtype x3, direction x2, exclusivity x2, column_ordering x2, flags x2);
- `reduce_axes` (reduce, prefix_scan, reduce_var, reduce_std): the E.1 rows plus the **rank-sensitivity** cases the spec
  singles out — rank-1 reduction must be `FF FF` not `01 00`; rank-3 full reduction `FF FF` not `07 00`; trailing `FE FF`;
- window-parameter vectors (avg_pool, max_pool, im2col): rank 1, 2, 3, `MAX_RANK`; the length prefix; element width;
- one `distinct` pair per op (axis 0 vs 1 where an axis field exists; the two `grammar_rebasing.rs` blobs are the seed).

Order of magnitude: on the order of 150-250 vectors. That is an estimate from the field table above, **not** a count; the
generator fixes the real number and the test pins it, as every other bundle here does.

## 3. How the expected bytes are produced, and what that does and does not buy

Two KISS-side derivations, required to agree on every vector:

- **A. A generic schema-driven encoder** over `opattrs_schema::parse_table()`: field values by name -> bytes by the parsed
  width/order. It knows nothing about any op by name, so it covers all 17.
- **B. The existing hand encoders** in `opattrs.rs`, for the 10 ops that have one.

The two share an author and a spec reading, so per the oracle-independence lineage they are *comprehension-correlated*:
agreement shows the code is self-consistent with the table, not that the table is read correctly. **The file's value is
therefore as a target for a second party, not as independent evidence on its own.** The note records that so the freeze
ledger does not over-credit it (umbrella §5.3 condition 1 still needs a second *party* per field).

Controls the generator and test must carry (a vector that cannot fail is not an instrument): a mutated width, a swapped
field order and a dropped default each turn the cross-check red; `rank` omitted where required fails to mint.

## 4. Who can be a second party

| party | produces OpAttrs bytes | role |
|---|---|---|
| **Fuel** | an envelope, **not** §6.19 (GAP-305, its own measurement) | **not a conformant second encoder.** Its divergence test derives expected bytes from the clause text, which is a second *reading*: diffing that against this file (body only) is the useful comparison. Optional, queued on Fuel's side |
| **kiss-ref** | **no, by design** (kiss-ref CHANGELOG 0.4.0, "What 0.4.0 does NOT model": OpAttrs bytes) | **ruled 2026-10-14 (PM): stays a non-party.** Adding a codec would have made kiss-ref a second implementation but made the repo's "models no wire" scope false |
| **Baracuda** | no (non-party, #504) | none |
| a third implementer | — | [ext]; this is the only route to *dissimilar* parties (E6) |

Consequence stated plainly (corrected 2026-10-14): **no party today can run the file as an interop test.** The pair
count stays 0 until a party emits a §6.19-conformant blob. Even then the pair would be KISS x that party, two readings
that trace to one interpretation (CireSnave's E6): that makes the claim *exercised* rather than *unexercised*, which is
what #504 asks; it does not discharge condition 1.

## 5. Clause and ledger effects (predictions, to be re-measured)

- Plausibly backable by vectors: 6.19-0004 (positional order), 6.19-0005 (default resolution), 6.19-0010 (definite
  length), the per-op schema clauses (6.19-0025 onward; ids to be confirmed) where a vector exercises every field. **Not** 6.19-0013: it demands
  two *structurally dissimilar* implementations, an attested gate; a KISS-side file cannot back it.
- Where a clause is half-covered it is counted PARTIAL as 6.8-0003 and 6.9-0003 were (floor unmoved, ledger note).
- #502's injectivity clause: writing it is a spec change (own PR, own version); the `distinct` vectors do not wait for it.

## 6. Slices (each one PR, PM-gated, version allocated at gate)

1. **KISS only:** generic encoder (A), `opattrs_value_vectors.json` generator in `kiss_mint`-style bin, the A-vs-B
   cross-check test, controls, README/CHANGELOG. No spec change, opt-in file.
2. **Fuel notice (sent 2026-10-14, optional on their side):** the file and blob hash, offered as GAP-305's external
   reference; a body-diff against their clause-derived vectors, any disagreement relayed as a [FINDING].
3. **kiss-ref decision:** only after the ruling in §4; otherwise record "no second in-repo party" in the file header.
4. **Decline kind** and the #502 clause, separately.

## 7. Open questions (none are mine to answer)

1. kiss-ref: does it gain an OpAttrs codec, or stay a non-party? (PM/CireSnave.)
2. Should the value file carry attribute *tokens* (proposed) or reuse Contract's field spellings? A consumer-friendliness
   question for Fuel; cheap to ask before minting.
3. `math_precision` / `accumulator` appear as `reduce` fields in the layout file; the precision-class vocabulary moved in
   0.2.x (§6.8-0007..0009, §6.17). Slice 1 must confirm the token spellings against the *current* §6.19.2 table before any
   vector names them — **not verified in this note**.
4. Is `distinct` allowed to land ahead of #502's clause (it asserts an obligation the spec has not written)? Proposed: yes,
   tagged `unclaimed: true` until the clause exists, so no clause id is cited.

## 8. What I did not do

Read §6.19.2/§6.19.3 end to end, run `opattrs_golden.rs`, or inspect Fuel's tree. §1's Fuel row is #504's measurement
from 2026-09-08, **not** re-measured at Fuel's current `origin/main`; slice 2 should re-measure it before relying on it.
