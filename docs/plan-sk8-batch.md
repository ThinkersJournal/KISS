# Plan: the sk8 breaking batch (0.18.0), board 443 and the per-operand dtype event

**Informative plan. No spec, codec or vector change is made by this document.** For the PM, before any spec PR is
pushed. Read at `origin/main` `71d678a` (conformance 0.17.5; #820 carries the 0.17.6 stamp and is not yet merged).
Inputs: CireSnave's board 443 answers (2026-11-14, via the PM), `rfcs/per-operand-dtype-sk8.md`,
`docs/options-440-seam-only-audit.md` (#820), and the files named below.

## 1. What rides the one release

| # | Change | Layer | Source of authority | Byte-visible where |
|---|---|---|---|---|
| 1 | Per-operand dtype in every `structure_key` sub-key (`sk7` to `sk8`); §6.6-0015 replaced | Classify | board 430 Part 1 Q2 "2a", RFC §3.1 option A | every key |
| 2 | Remove the `x<n>` / `Other(n)` escape; the Announce `unnamed_types` record | Classify, Vulkan namespace, Announce | board 430 Q5 "5b", RFC §5.1 | keys containing `x<n>`; Announce device record (its own version rule) |
| 3 | Remove `precision_tag` and its dtype and fidelity rules | Ops | board 443 "remove the tag with sk8" | `matmul` OpAttrs blob |
| 4 | Optional requested accuracy bound on `matmul`, in the Contract tier shape | Ops (field), Contract (matching) | board 443 "Yes" | `matmul` OpAttrs blob |
| 5 | A kernel advertising a reduced-precision cell MUST declare its tier; refusal of a bound-less cell is kept, not relaxed to `unbounded` | Contract, Ops | board 443 "definitely" | none (a declaration rule; conformance only) |
| 6 | Wording: 6.17-0006 becomes a measuring reference only; float accumulator width stays the requester's floor | Ops | board 443 | none |

Not riding (ruled): accumulate-into-C stays option A (ordinary ops); no recipe list; no standalone tag release;
`reduce` does not get the bound field in 0.18.0 (board 443: "matmul (later reduce)").
**Undecided, would be byte-visible, need a yes or no before the freeze:** the R1.26 remainder (block length
`{16, 32}` and scale-group axis in the contraction field); the RFC §8 questions 2 and 3 (dtype position A/B/C;
narrow-index decline versus key rule). I have read no ruling for either. **Question for the PM: have they been ruled?**

## 2. Exact changes

### 2.1 The `matmul` OpAttrs blob (items 3, 4)

Today (`spec/ops.md` KISS-OPS-6.19-0049): `accumulator` u8, `math_fidelity` u8, `precision_tag` u8 (default `00 01 01`).

Proposed, 11 bytes, every field emitted explicitly (6.19-0005):

| Field | Width | Values |
|---|---|---|
| `accumulator` | u8 | unchanged |
| `math_fidelity` | u8 | unchanged |
| `accuracy_kind` | u8 | `0` reserved, `1` none (no bound requested), `2` max_ulp, `3` max_relative, `4` max_absolute |
| `accuracy_value` | 8 bytes LE | `max_ulp`: u64 count; `max_relative`, `max_absolute`: IEEE 754 binary64 bits; kind `none`: all zero |

Default blob `00 01 01 00 00 00 00 00 00 00 00`. A reader declines a length other than 11, a `0x00` kind, a kind above
4, a non-zero value under kind `none`, and a NaN, negative or infinite binary64 value under kinds 3 and 4.
**Alternative X:** omit `accuracy_value` under kind `none`, so the default stays the three bytes `00 01 01`.
It keeps every none-only blob byte-identical, but presence of the value then depends on the kind, the conditional
presence the RFC (§3.1) avoids; the blob length alone would still stop an old `3xtf32` blob (`00 02 02`) decoding as
a `max_ulp` request. **I recommend the fixed 11 bytes**: every consumer re-derives for sk8 anyway, so the saving
buys little. The spelling is the reference codec's to confirm, not this plan's.
The bound is **not** in the `structure_key` (the board 440 Q1 ruling, "tag stays out of the key", applies to the bound
too): it rides in the blob, so it is part of op identity beside the key.

**Matching rule (new clause, home to confirm):** a kernel is admitted for a requested bound when it declares, for the
target, a tier of the **same kind** with a value **no looser** than the request. A kernel that declares only other
kinds is not admitted (kinds are not convertible); a request of kind `none` admits every kernel that satisfies the
key. This is the Announce 6.6-0010 comparison, applied between request and declaration.

### 2.2 Clauses (indicative)

| Clause | Change |
|---|---|
| KISS-OPS-6.17-0011 | replaced: the tag text goes; it states the requested bound and the matching rule |
| KISS-OPS-6.17-0006 | reworded: the TF32 pin is the reference rounding that derives a bound, not a rule a kernel must follow |
| KISS-OPS-6.17-0008 | float accumulator stays "at least as wide" (the requester's floor); wording only |
| KISS-OPS-6.19-0049, the 6.19.2 table row, the defaults line, the worked vectors (3765 to 3769) | new blob |
| KISS-CONTRACT-6.8-0002 | carve-out: a cell advertising `reduced-mantissa-permitted` MUST declare a tier, so the "no finite bound declares no tier" allowance does not apply to it |
| KISS-CLASSIFY-6.6-0015, -0005, -0007, 6.7-0004, -0006, -0014, 6.6-0022 | per RFC §6 |
| Contract Appendix C golden | the embedded `structure_key` changes, so the golden bytes change |

Item 5's scope is a choice: **every `rm` cell of any op** (matches "a kernel that runs with lower precision must
specify that precision"; the `reduce` blob also carries `math_fidelity`), or `matmul` only. I will write it as every
`rm` cell unless the PM says otherwise.

### 2.3 Conformance and corpus

- Code: `conformance/src/opattrs.rs` (`PrecisionTag` removed, accuracy kind and value added); `structural.rs`
  (`TagNeedsReducedMantissa` and `TagComputeDtype` removed, `TagWithoutBound` becomes a refusal of an `rm` cell with no
  tier); `structure_key.rs` (`STRUCTURE_KEY_VERSION` 7 to 8, sub-key dtype component, four new declines); `contract.rs`,
  `derivation_vectors.rs`, `reference_vectors.rs`.
- Tests: `test_ops_precision_tag` is replaced; every new or reworded clause gets its own test name.
- Regenerated with the `emit_*` bins: `structure_key_vectors.json` (77 `sk7` lines), `structure_key_derivation_vectors.json`
  (54), `dtype_manifest.json` (1), `opattrs_value_vectors.json` (11 `precision_tag` entries), `opattrs_vectors.json` (1).
  New decline vectors: sub-key missing its dtype; operand-0 dtype disagreeing with field 2; reserved dtype in a
  non-primary operand; an `sk7` token at an `sk8` reader.
- Unchanged by this batch (no `sk7` or `matmul` text in them at 71d678a): `announce_vectors.json`, `grammar_vectors.json`,
  `packed_reduction_vectors.json`, `contract_vectors.json` (re-check after the codec change, since a contract embeds a key).
- Floor, README bound markers, `UNBACKED.tsv` and the sidecar are re-derived with the tools at the end.

## 3. Order of work

One release means one merge to `main` stamped 0.18.0: two breaking merges would be 0.18.0 and 0.19.0. So:

0. **Now.** #820 merges (0.17.6). This plan merges as docs.
1. **Close the ride-along list (RFC §7 step 2).** I send one `[NOTICE]` to Fuel, kiss-ref's lane, Unpopped, Baracuda
   and Vulkane with items 1 to 6 and ask for their own byte-visible items and index-dtype cases. Nothing freezes until
   it closes. The PM rules the R1.26 remainder and RFC §8 questions 2 and 3.
2. **An integration branch** `sk8` in its own worktree off `origin/main`, with sub-PRs into it reviewed one by one
   (A: Classify grammar and codec; B: Ops blob, bound and wording; C: Contract rule and goldens; D: Announce
   `unnamed_types` and the Vulkan note). Merge `main` in on every move; one `changelog.d` fragment, kind `**BREAKING**`.
3. **The codec emits the corpus; three other derivations regenerate and byte-match** (Fuel, kiss-ref, Unpopped's
   `unpopped-vocab`); Baracuda regenerates its emit corpus; mismatches are findings against the grammar text.
4. **One PR `sk8` to `main`**, gated and stamped **0.18.0, BREAKING** (pre-1.0: the second number moves; the stamp is
   the PM's). `conformance/Cargo.toml` and the top `CHANGELOG.md` heading move together.
5. After merge: tag `v0.18.0` on the exact commit; each re-vendoring party confirms; remove the worktrees and branches.

## 4. Who re-vendors

| Party | Why | Evidence |
|---|---|---|
| Unpopped (`unpopped-vocab`, vendored `kiss/`) | key re-derivation; `matmul` blob; none-only for the tag (Unpopped@9be7e15e, 0 hits) | its own answer, 2026-11-14 |
| Fuel | key re-derivation (a derivation leg) | RFC §7 |
| kiss-ref | key re-derivation (a derivation leg) | RFC §7 |
| Baracuda | emit corpus regenerated under `sk8` | RFC §7 |
| Vulkane | cosigns vocabulary; the `x<n>` removal bumps the Vulkan vocabulary version; derives no key | RFC §5.1, §7 |
| bonefish | maintains the `sycl` namespace (`spec/namespaces/sycl.md`); I do not know whether it vendors keys or `matmul` blobs | the `sycl` annex only |
| rockfish, fishstringer | **I found no mention of either in this repository** and do not know their role | none |

The last two rows are gaps. I will ask the PM for their roles and ask each whether it vendors KISS corpus files or
emits `matmul` OpAttrs; nothing here says they need to re-vendor.

## 5. Risks

- Main moves under a long integration branch; the merge-main-in cost is the CHANGELOG, Cargo and floor files, which the
  stamp procedure already isolates.
- The `x<n>` removal needs the dtype naming list of `docs/dtype-completeness-audit.md` §2 finished first (RFC §5.1);
  I have not checked its state.
- A requested bound in op identity splits caches by bound; accepted here by the ruling, and revisited only if a consumer
  reports it.

— lane: kiss
