# How much of the unbacked backlog is actually testable?

**Informative.** A companion to [`UNBACKED.tsv`](UNBACKED.tsv): that file says *which*
clauses have no executable test; this one says *why*, and *which of them can be fixed
today*. (The suite has **857** normative clauses total; when this analysis was first run,
**~812** were unbacked. Track the live split with `python tools/kiss_trace.py --report`.)

The honest short answer: **not the whole backlog.** A clause names a conformance test, but a
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
| **untestable as written** | vague, self-contradictory, or unenforceable until reworded | not until reworded — **4 verified, all filed as issues (see below)** |

**Verified count: 72 clauses are executable-now with teeth** (of 239 candidates checked;
the rest reclassified — 73 to *blocked*, 49 to *no-teeth*, 20 to *untestable*, 16 to
*lint*). This is a **floor**: only the top ~30 candidates per sub-standard were verified,
so more exist in the tail — but 72 is the number that survived scrutiny, so it is the
number to quote.

The full verified list, with the wrong implementation each test catches, is
[`WORKLIST.tsv`](WORKLIST.tsv). By sub-standard: OPS 18, Classify 17, Grammar 14, Announce
10, Synth 6, Contract 6, Conform 1. By effort: 26 trivial, 35 small, 11 medium.

### Progress — 62 of the 72 are now written

Backing has gone **45 → 109 / 857 clauses (12.7%)**, with five new reference modules
(`grammar`, `fp`, `dtype`, `contract`, plus the `§6.18-0017` split comparator) and 192
passing tests. `WORKLIST.tsv` marks each done row `DONE`. Coverage by sub-standard:
OPS 25.6%, Announce 30.7%, Classify 23.1%, Grammar 19.2%, Contract 5.8%.

The **10 still `todo`** are exactly the ones that should wait:
- **Synth (6)** — the provision-protocol frames. Fuel implements provision (JitRequest/
  JitResponse, two-phase handover); these want its input before the frame shapes are pinned.
- **Classify (3)** — `§6.5-0009` (its own example is wrong — see PR #35 review), `§6.5-0010`
  (the M-vs-K work-class contradiction), `§6.6-0008` (touches the keepdim-stride
  contradiction). All want the single-classifier-division decision settled first.
- **Conform (1)** — `§6.2-0003` (the gate must reject a test citing a *retired* clause ID):
  a `kiss_trace.py` feature, not a Rust-harness oracle; a clean follow-up.

Writing the tests surfaced real defects in the reference harness along the way — see the
next section.

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

## The 4 untestable-as-written clauses (all filed)

The triage's "untestable" bucket was **unreliable** — most of its tags were refuted
test *proposals*, not unenforceable *clauses*. A dedicated verified pass (one auditor per
sub-standard, precision over recall) found that **7 of 9 sub-standards have ZERO** genuine
untestable clauses (Synth, Consume, Emit, Grammar, Announce, Classify, Conform are all
clean — verbose but byte-pinned). Only **4 clauses** are genuinely untestable as written,
each now tracked:

- **#41** — `KISS-OPS-6.13-0006`: the decomposition grammar cannot parse ~9 structured-op
  rows that are prose (matmul, pools, gather-family, im2col).
- **#42** — `KISS-OPS-6.0-0002`: `atan2`'s determinism class is self-contradictory —
  exact-byte by the §6.0-0002 iff, 4-ULP by §6.8-0001; an exact-byte comparator rejects
  every conforming `atan2`.
- **#43** — `KISS-CONTRACT-6.6-0004` vs `-6.6-0006`: `thread_mapping` / `addressing_rule`
  are functions of a per-thread index the Dispatch grammar has no symbol for (the Dispatch
  ungrammar, WIRE-FIRST §1.4).
- **#44** — `KISS-CONTRACT-6.7-0006` / `-6.8-0002`: §6.11 pins no byte layout for the
  compound `cost` and `per_backend_ulp_tiers` fields, and **no float encoding exists
  anywhere in KISS-Contract** — so no fractional value (rel/abs error) can be byte-compared.

(`KISS-OPS-6.8-0001`, the transcendental ULP ceilings, is #39 — arguably a fifth, filed
earlier.)

## What this session backed (12 clauses, 45 → 57)

Six reverse-citations of tests that already had teeth but cited no clause (`6.15-0001`,
`6.15-0002`, `6.6-0002`, `6.7-0001`) and two new oracles (`6.15-0003` rem_floor/trunc,
`6.13-0007` hypot). **Six fixed real defects in the reference harness**, each with a test
proven to fail against the old code:

- **`6.19-0038`** — `validate_reduce_axes` accepted `0x0000`, which the clause says a
  reader MUST reject on the OpAttrs channel.
- **`6.19-0037`** — `decode_gather` returned `axis`/`index_operand` unchecked, so
  `decode_gather(&[9,1,0,1])` was `Ok` with an axis that would index a nonexistent axis.
- **`6.4-0004` / `6.4-0002` / `6.8-0001` / `6.7-0002`** — the `structure_key` `from_token`
  reader had no length bound, no operand-count bound, no `target` grammar check, and
  accepted the non-canonical `sk02`. Now a validating reader.

The rest of the 72 are queued in [`WORKLIST.tsv`](WORKLIST.tsv) with the wrong
implementation each test catches.

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
