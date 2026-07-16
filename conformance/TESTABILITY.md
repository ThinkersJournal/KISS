# How much of the 806 is actually testable?

**Informative.** A companion to [`UNBACKED.tsv`](UNBACKED.tsv): that file says *which*
clauses have no executable test; this one says *why*, and *which of them can be fixed
today*.

The honest short answer: **not the whole 806.** A clause names a conformance test, but a
conformance test is only worth writing if it has **teeth** — a plausible wrong
implementation it catches. `DESIGN.md` and this crate's README both state the bar: *"A
harness that only ever passed correct code would prove nothing."* Roughly a third of the
unbacked clauses cannot meet that bar as written, and another large fraction cannot be
written *yet* because the spec has not decided the bytes they would check.

## Method

A multi-agent triage read every unbacked clause against its actual spec text and the
harness, sorted each into one bucket, then sent each *"testable today"* candidate to an
independent verifier whose only job was to **refute** it. **70% were refuted** — that
refutation rate is the point: it is the same discipline that should have been applied to
the original 852 named-but-nonexistent tests, and it is why the number below is small and
trustworthy rather than large and hopeful.

## The breakdown

| Bucket | What it means | Can a Rust test back it? |
|---|---|---|
| **executable now** | a real test with teeth is writable against today's spec + harness | **yes** |
| **blocked on a decision** | testable in principle, but the spec does not yet determine the bytes/behaviour (no message framing, no golden vectors for the frames or the contract document, no `artifact_format_tag` registry, Dispatch ungrammar — see [`../WIRE-FIRST.md`](../WIRE-FIRST.md)) | not until the spec decides |
| **document lint** | binds the *spec document*, not an implementation (an enum owned once and never re-forked; a version-bump rule; a cross-table closure) | by a linter like `kiss_trace.py`, never a harness |
| **no teeth possible** | a test could be written but no wrong implementation exists for it to catch — tautological or definitional | writing one adds a green light that proves nothing |
| **untestable as written** | vague, self-contradictory, or unenforceable until reworded | not until reworded |

**Verified count: 72 clauses are executable-now with teeth** (of 239 candidates checked;
the rest reclassified — 73 to *blocked*, 49 to *no-teeth*, 20 to *untestable*, 16 to
*lint*). This is a **floor**: only the top ~30 candidates per sub-standard were verified,
so more exist in the tail — but 72 is the number that survived scrutiny, so it is the
number to quote.

The full verified list, with the wrong implementation each test catches, is
[`WORKLIST.tsv`](WORKLIST.tsv). By sub-standard: OPS 18, Classify 17, Grammar 14, Announce
10, Synth 6, Contract 6, Conform 1. By effort: 26 trivial, 35 small, 11 medium.

## Why the buckets are shaped the way they are

- **OPS is the most testable and should be embarrassed by that, not comforted.** It pins
  IEEE-754 edge cases against a from-scratch CPU oracle — the one thing a std-only Rust
  harness does perfectly, with no framing or handshake required — and `src/semantics.rs`
  *already ships the wrong implementations* (`naive_max_x_zero`, with a comment saying it
  exists "so the divergence is a test rather than a footnote") and then never differenced
  them. The oracle was written; the tests were not.
- **Synth has the most *no-teeth* clauses (44)** — it is a protocol whose frames have no
  golden vectors, so most of its clauses are either blocked (need bytes) or tautological
  (restate a definition).
- **Emit and Conform are mostly blocked**, not testable — they depend on artifacts (the
  fuzzer, the frames, the contract document) that do not exist.

## What this session already did (6 clauses)

Four were reverse-citations of tests that already had teeth but cited no clause
(`6.15-0001`, `6.15-0002`, `6.6-0002`, `6.7-0001`). Two fixed **real defects in the
reference harness**, each with a test proven to fail against the old code:

- **`6.19-0038`** — `validate_reduce_axes` accepted `0x0000`, which the clause says a
  reader MUST reject on the OpAttrs channel.
- **`6.19-0037`** — `decode_gather` returned `axis`/`index_operand` unchecked, so
  `decode_gather(&[9,1,0,1])` was `Ok` with an axis that would index a nonexistent axis
  downstream.

Both are the important kind of finding: the green harness had bugs the old gate could not
see, and a test with teeth is what surfaced them.

## Spec defects the triage surfaced (verified, worth filing)

- **§2.8's shared dtype token list is missing `s16`/`u16`/`u64`** that §6.10-0006 and §6.16
  both carry. PRs #30/#31 updated the normative tables but not the shared anchor — the
  exact drift a table-parser lint (the *document lint* bucket) would catch permanently.
- **ISO C Annex G is not in §4's Normative References**, yet all 17 §6.18 complex clauses
  pin against it, and §6.18-0005 defers `cmul`'s recovery signs to "the Annex-G-determined
  component signs" without stating them — a dangling reference that falsifies §6.18's own
  "resolvable from this document plus the umbrella" goal.
- **§6.13's op table mixes two languages**: ~50 rows are expressions in the §6.13-0006
  grammar, but ~9 (`argmax`, `matmul`, `avg_pool`, …) are prose, so a resolver obeying
  §6.13-0006 ("parse the body as this tree form and no other") cannot read a third of its
  own table — the same shape as the known Dispatch ungrammar blocker.
- **§6.18-0017**: the harness's own ULP comparator (`total_order_f32`/`ulp_distance_f32`)
  maps `−0.0` and `+0.0` one ULP apart, so `ulp_distance_f32(-0.0, +0.0) == 1` — which
  means the §6.8 ULP ceilings silently accept a complex op returning the wrong sign of
  zero. The clause mandates a split comparator precisely because the plain one cannot see
  what §6.18 pins. A test here needs that comparator built first, or it has phantom teeth.
