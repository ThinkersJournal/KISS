# RFC: Derive what is duplicated; prove that instruments can fail

| | |
|---|---|
| **Status** | **Draft** — awaiting maintainer ratification |
| **Date** | 2026-08-08 |
| **Affects** | KISS-Conform §6.1 (traceability matrix), §6.5 (oracle-differential harness) |
| **Normative text** | KISS architect |
| **Motivation & instance inventory** | Fuel architect |
| **Clause drafts §6.5-0011..0016, §6.1-0009a/0009b** | KISS Lane B |
| **Instances contributed by** | Fuel, KISS, Baracuda, Vulkane, kiss-ref, Unpopped |

---

## 0. The observation

Two defect shapes were found across six projects in a single day, by different people, in
unrelated code, for unrelated reasons — a wire-codec regen, a dtype audit, a conformance-tool
review, a build-system incident. Each was diagnosed on its own terms. They end in the same
sentence: **the check passed and should not have.**

They are **two** patterns, not one, and they need separate normative text (§4).

**On the instance counts below.** They reflect **who was looking**, not where the problem
lives. Four of Pattern A's instances and several of Pattern B's are Fuel's because Fuel audited
first and hardest. Presenting them as a distribution would be a measurement whose failure mode
is indistinguishable from success — which is this document's own subject. They are evidence the
shapes are **real**, and weak evidence about **frequency**.

---

## 1. The meta-failure, which is the real motivation

**Both patterns have already been recognized inside KISS, solved for one artifact, and never
generalized.** That has now happened twice, and it is a better argument for a clause than any
single defect:

- **KISS-CONFORM §6.1-0004** requires the traceability matrix be *"derived, not hand-authored"*,
  explicitly *"because a hand-maintained matrix can silently diverge."* The reasoning is correct
  and general. It binds **one artifact**. Three dtype tables in the same suite remain
  hand-maintained.
- **`conformance/tests/integer_differential.rs`** carries seeded-bug negative controls —
  `differential_catches_the_saturating_add_bug`, `..._saturating_abs_bug`,
  `..._always_logical_shr_bug`. These assert precisely that the differential *can detect a wrong
  kernel*. Someone understood the discipline, implemented it, and **no clause claims it.**

The principle is present in the suite twice — once as a local fix, once as an untitled practice.
A standard whose purpose is to make the other eight provable should not leave its own best
practices unstated.

**A note that reframes the whole document.** The prose solutions found in the field were not
laziness. Fuel's sharpest instance is a module doc comment in which the author writes *"an early
return from a `#[test]` is a **pass** … no line begins `SKIP:`"* — identifying the failure
exactly, naming the discriminator precisely, enforced by a human remembering to grep stderr.
**That author had nowhere to put a declaration.** The mechanism this RFC supplies is the thing
they wished for. The problem is not that people were careless; it is that the standard offered
them nowhere to be careful.

---

## 2. Pattern A — a normative fact duplicated across N artifacts, one of which is checked

**Fix: derive the duplicates from the single source of truth.** Drift stops being *detected* and
becomes *unrepresentable*.

| # | Where | The duplication | What went wrong |
|---|---|---|---|
| A1 | KISS `spec/` | `dtype_manifest.json` (SSOT), `classify.md` §6.1, `ops.md` §6.16 — **the lint validates one** | An `sk4` schema event respelled §6.1's table; §6.16 and eight clauses kept `sk3` spellings. §6.16 carries an **explicit written invariant** that tokens are *"spelled identically in both foundational vocabularies"* — and nothing checks it. **A stated invariant with no checker is this pattern's purest form.** |
| A2 | KISS `tools/kiss_trace.py` | §6.1-0003's prose vs the checker | `dangling` sets `any_fail = True`. `orphans` **does** print *"N executable tests cite no clause"* — but **never sets `any_fail`**, so the count is reported and the build passes, while §6.1-0003 calls an uncited test *"an orphan and a build-fail."* **The document asserts a rule the checker declines to enforce.** *(Recorded as an ambiguity, not a proven under-enforcement — see §6.)* |
| A3 | KISS `spec/conform.md` §6.1-0007 | prose vs a machine-readable sidecar | The clause states the sidecar **MUST be the sole authoritative clause source** and *"prose MUST NOT be treated as an independent clause source."* **No sidecar exists.** `kiss_trace` derives its entire clause set by regexing prose. Six clauses depend on machinery never built. |
| A4 | Fuel `fuel-dispatch` | a CI gate vs its registry rule | The gate enforced **14** patterns while the registry rule named **5**. Closed by a hand-coordinated companion commit, with a window in which they disagreed. |
| A5 | Fuel `docs/kernel-contracts/**` | `audited: true` vs the verification ledger | **794** contract sections declared `audited: true` with **no ledger entry**. The ledger covered 21 of 804 kernel names — **2.6%**. The claim existed; nothing made it true. |
| A6 | Fuel `fuel-graph` | fused-op destructiveness | Hardcoded in a match while the registry knew the ops. **A fact about a thing, stored away from the thing, kept in sync by memory.** Fixed by making it a mandatory registry field — which forced 28 construction sites at compile time. |
| A7 | Fuel `fuel-dispatch` | a doc comment vs the code | `dtype_byte_size`'s comment claimed a wiring relationship that never existed; a different mechanism had won and the comment outlived it. |

**What they share:** in every case the fact was correct *somewhere*. The defect is that a second
copy existed and was believed.

---

## 3. Pattern B — an instrument whose failure mode is indistinguishable from success

### 3.1 The mechanisms, and one outcome

**This taxonomy is the clause's scope, and it is what makes compliance checkable.** *"Am I doing
the bad thing?"* has no method. ***"Which of these five can my suite do?"*** does — and an
implementer can answer it about code they have never read.

| mechanism | the instrument… | canonical instance |
|---|---|---|
| **vacuity** | runs completely and **cannot fail** | B1 |
| **skip** | returns early at run time and reports `ok` | B2, B3 |
| **absence** | never runs at all, and says so in a form that reads as success | B4 |
| **degradation** | runs, and silently covers **less** than it claims | B5, B6 |
| **abstention** | reads a value successfully, then **declines to judge it** | B9 |

> **abstention** — the instrument matches, successfully extracts a value, and then declines to
> evaluate it because the value falls outside a set it recognizes. **Unexpected input is converted
> into a pass.** Nothing on that path resembles failure from inside the tool: it looks like a case
> the rule does not cover. The hazard is that the triggering condition is **novelty**, so the
> mechanism fires on precisely the change the instrument exists to catch.
>
> **Why it is not degradation.** Degradation covers fewer **inputs**; abstention covers fewer
> **judgements over an input it did examine**. The fix classes differ, which is the practical test
> for whether a row earns its place: degradation is fixed by making the *miss* an error, abstention
> by making the *unrecognized value* a violation. Contrast it with the nearest-looking failure,
> which is not this one:
>
> | | anchor-miss (a *degradation* shape) | **abstention** |
> |---|---|---|
> | what happens | the pattern does not match; there is no value to compare | the pattern **matches**; a value is extracted, then found outside the tool's lookup |
> | what the tool concludes | "nothing here" | "**not applicable to me**" |
> | fix | make the miss an error | make the unrecognized value a **violation**, not a skip |

### 3.2 Instances

- **B1 — vacuity (Baracuda).** `baracuda-kernels-bench/tests/synthesize_relu_add_repro.rs`
  computes the got-vs-`relu(a+b)` mismatch and only **prints** `"CORRECT"`/`"WRONG"`.
  `grep -c assert` = **0**. It runs to completion, exercises the kernel, and reports `ok` **when
  the kernel is wrong** — in a file **labelled a correctness guard.** No probe, no gate, no
  truncation. No assertion. It cannot fail by construction.
- **B2 — skip, documented and unenforced (Fuel).** `tri_backend_device_group_live.rs`'s module
  doc states the failure and the discriminator in the author's own words; nothing mechanizes it.
  **Documentation of a hazard is not a control for it.**
- **B3 — skip, structural (KISS).** Eleven tests across eight files decline via
  `let Some(m) = find_msvc() else { eprintln!("SKIP"); return; }`. `kiss_trace` *already* records
  a gate for `#[cfg(feature=…)]` tests and reports *"N of the backed are feature-gated and do not
  run in the default build"* — **so KISS already holds the concept that a test which did not run
  is not evidence.** A runtime skip has no attribute to find, so the mechanism is structurally
  unable to reach the case. **The affected tests are the §6.5 C-ABI differential — the §5.3
  external-implementor freeze evidence.**
- **B4 — absence (Vulkane).** `kiss_target_live.rs` is `#![cfg(feature = "kiss-target")]`; CI runs
  neither that feature nor `derive`. Those files compile to **nothing** and report
  `running 0 tests … ok`. **The two suites guarding the subtlest fixes never execute in CI at
  all.**
- **B5 — degradation (KISS).** `target_namespace_registry.rs`'s `scan()` does
  `let Ok(entries) = read_dir(dir) else { return; }` — silent, recursive, three degradation points
  (`read_dir`, `read_to_string`, `flatten`). **Proven exploitable:** a planted unregistered
  namespace is caught by the intact sweep (`FAILED. 3 passed; 1 failed`) and **passes** with one
  subdirectory read failed (`ok. 4 passed; 0 failed`). The function's own doc comment says it
  exists because *"a hand-maintained claim that nothing checks is the failure mode this whole
  clause-set is about."*
- **B6 — degradation in the gates themselves (Fuel).** `fkc_clause_coverage.rs` and
  `fkc_prose_clause_coverage.rs` — the artifacts built to prove every clause is cited — use
  `else { return }` and `else { continue }` on `read_dir`. The `continue` form is worse: a failed
  subdirectory read is skipped and the sweep **carries on**.
- **B7 — the measuring instrument under-reports itself (KISS).** `discover_tests` keys by bare
  test name tree-wide, so same-named tests overwrite: 141 occurrences reported as 139. And
  `kiss_trace.py`'s `except OSError: continue` drops an unreadable file's tests entirely —
  **totals, orphan counts and gate computations all shrink together**, so nothing looks
  inconsistent.
- **B8 — a green that compiled none of the code under test (Fuel).** `--all-targets` does **not**
  imply `--all-features`; `fuel-dispatch`'s non-default `telemetry` feature is where the sk4 token
  deriver lives. A six-crate sweep would have reported green having exercised nothing. Caught only
  because a worker saw *"running 0 tests; 737 filtered out"* and refused to read it as a pass.

- **B9 — abstention (KISS).** In `tools/kiss_tables.py`, the `words = {"four": 4}` count-word
  check (read at `19c3ad7`) verifies that KISS-Consume's prose count word
  matches the taxonomy's actual size:

  ```python
  words = {"four": 4}
  if cw and words.get(cw.group(1)) not in (None, len(REFUSAL_MEMBERS)):
      out.append(f"§6.4-0001 says '{cw.group(1)}' but names {len(REFUSAL_MEMBERS)}")
  ```

  The regex **matches**; the count word is **read**; and then `words.get("five")` returns `None`,
  `None not in (None, 4)` is `False`, and **no violation is raised**. The `None` arm, written to
  mean *"a word I don't know — skip"*, disables the comparison on the one edit it exists to
  notice: **the taxonomy growing from four to five.** The `words = {"two": 2, "three": 3}` check in
  the same file is the same shape for the `MathPrecision` `two-member` enum. **The instrument
  succeeded at every step and judged nothing.**

  > **On the citation form.** This names the **construct and the commit**, not a line number. The
  > defect spans three lines — the search, the lookup table, the comparison — so any single line
  > number is wrong for two of the three, and drifts on the next edit above it. The first draft of
  > this instance cited `:329` and `:290`; the true lines at `19c3ad7` are 330 and 291, and the
  > wrong ones were carried from a ruling into an issue into this document before a reviewer caught
  > them. **A citation without a commit is unfalsifiable in exactly the way a stale grep is** — and
  > three stale-tree readings were produced tonight, by three people, every one of them
  > clean-looking.

### 3.3 What B6 proves about the obvious fix

Fuel's gates already carry a mandated **non-triviality** assertion: the derived set must be
non-empty; the predicate must select neither all nor none. **It catches total failure and is
silent on partial.** An empty sweep trips the minimum count; a sweep reading nine of ten
directories stays above it and reports green with a tenth of the corpus invisible.

**So the counter-measure needs three properties, not two: non-trivial, discriminating, and
complete.** For a directory sweep that is total and cheap — **propagate the error instead of
returning. A discovery step that can fail should fail, not shrink.** The general case is harder
and is stated in §6 as a limit rather than solved here.

---

## 4. Why one clause cannot cover both patterns

**Pattern A is about where a fact lives. Pattern B is about whether a measurement can see.**

A clause requiring *"a normative duplicate of an SSOT MUST be derived from it"* does nothing for
B1 — a missing assertion is not a duplicated fact. A clause requiring instruments to demonstrate
discrimination does nothing for A1 — three tables can each be checked by a perfectly
discriminating lint and still disagree, because nothing says they must agree.

**The risk of merging them is concrete:** implementers derive their tables, believe the rule is
satisfied, and keep shipping instruments that cannot fail.

---

## 5. Scope — where each pattern stops

**KISS-Conform's declared remit *is* the instrument.** §2.1: *"the one sub-standard whose job is
not to describe a wire but to make the other eight provable."* An instrument whose failure is
indistinguishable from success is **the negation of that remit**, not adjacent to it. And it sits
on the freeze path: §2.7 rests Draft→Frozen on measurements an AUDIT role **signs**. **If the
instrument cannot discriminate, the AUDIT role signs a fact that was never established.**

**But Pattern B has a boundary, and a document that cannot say where a pattern stops is
overclaiming.** A suite taking **no environment input** cannot exhibit *skip* or *degradation*; a
project with **no glob-corpus harness** has nowhere for *degradation* to live.

- **kiss-ref audited to zero** — 364 run, 4 `#[ignore]`d and accounted for. Their tests take no
  environment input, so the class needs a probe they structurally do not have.
- **Baracuda's axis-3 zero** has the same shape: no datatest/glob-corpus harness, so the
  degradation case cannot exist there. Every runtime-discovered enumeration is non-empty- or
  exact-count-guarded.

**Two independent demonstrations that a component can be immune rather than lucky.** That is what
makes the mechanism list a *scope* rather than an accusation, and lets an implementer rule mechanisms
out on **structure** instead of on faith.

### 5.1 They are a decomposition, not a catalogue — demonstrated on the review layer

The mechanisms were derived entirely from **test harnesses**. If they are a real decomposition of
the failure rather than an artefact of where the authors happened to look, they should apply to an
unrelated instrument. They do — **four of the five, to code review**, discovered while this RFC was
open. The demonstration below predates `abstention`'s identification and does **not** cover it:
whether a review-layer abstention exists — a reviewer reading a construct they do not recognize and
passing over it — is **unexamined**, and is left stated rather than assumed, since claiming five of
five here would be the overclaim this section exists to guard against:

> *"A review comment that nothing routes is a finding nobody acts on, which looks identical to 'no
> findings.' That is the review-layer version of the silent skip."* — Baracuda

- **vacuity** — a review that ran and raised nothing because nothing in it could.
- **absence** — a PR never reviewed, reported alongside reviewed ones.
- **degradation** — a review run against a stale diff, reporting a defect already fixed at head
  (observed: a finding on PR #132 was already resolved at its own head commit).
- **skip** — findings that exist and that nothing routes to anyone. **Observed: twelve inline
  findings across six PRs, eleven unaddressed, while `gh pr list` reported `reviews=1` on each.**

**`reviews=1` is the whole document in one field of one CLI output: a count that reads as a
state.** A green-looking review count and an actually-read review are the same bytes.

**The general form, which covers every instance in this RFC:** *a state derived from an artefact's
**existence** is not a state derived from its **content**.* That is `reviews=1`; it is `E0004: 0`
from an aborted enumeration (B4); it is `running 0 tests … ok` (B4); and it is a registry row
marked `FIXED` with live residuals.

---

## 6. Stated limits — including the instances the authors committed while writing this

**Generation prevents the next class, not this one.** The drift that prompted A1 was entirely in
hand-written prose *around* the tables. Generating them would not have caught it. **A hardening
item that over-promises is itself a Pattern B failure** — an instrument trusted past its scope,
whose insufficiency is indistinguishable from sufficiency.

**The instances enumerated below were all committed by this document's authors, in its own
subject matter, within a day of articulating the principle. They are the closing argument, not a
footnote.** The count is stated once, in the closing note where it carries the argument — three
hand-maintained copies of a number that grows every few hours is the duplication §6.1-0010 exists
to stop, and this document had already let one of them go stale.

1. **KISS architect — "verified clean", reported upward.** An `sk4` spec respell was verified and
   reported to Fuel, to the maintainer, and to three other projects. **The sweep searched retired
   dtype spellings and never contained a version prefix.** Re-running for `sk3|` found **54
   sites**, including 13 in `classify.md`, one reading *"sk3 is the current supported version."*
   **A clean bill of health propagated across five projects.** The counterfactual makes the cost
   concrete: *a deriver reading that stale clause instead of `SCHEMA_VERSION` would have emitted
   `sk3|` and byte-matched against nobody.* The chain held only because Fuel took the version from
   the constant rather than the prose — **a correct choice that happened to route around the
   error, not a safety net that caught it.** *A failure survived by luck is more instructive than
   one that was caught.*
2. **Fuel architect — an audit pattern narrower than the class it audited.** A sweep for runtime
   skips searched `eprintln!("SKIP` — the uppercase form, **taken from the KISS instance that
   prompted it** — while Fuel's idiom is lowercase. **5 reported; 33 actual.** The number was
   already in a cross-project count.
3. **kiss-ref — a rigorous answer on the wrong axis.** Verified correctly that they define zero
   cargo features, concluding the feature-gate failure could not occur. True, and **silent on
   runtime early-return.** A check that discriminates perfectly on the axis it examines **tells
   you nothing about any other, and the reassurance does not know its own scope.**
4. **KISS architect — over-reading a requirement's scope.** Read *"the `sk3` decoder arm MUST be
   bounded"* as mandating that every implementation carry such an arm, and propagated it to a
   deriver as a ruling. The MUST attaches to *bounded*, not to *exists*. Corrected within ninety
   minutes, after a worker asked a scoping question rather than complying.
5. **KISS architect — an overstated backing claim, in the table added to prevent overstated
   backing claims.** §8's first revision recorded KISS-CONFORM-6.1-0009 as *backed*. It was not: the crediting
   half was unimplemented, and the implementing PR's own diff said so in a comment. **Caught by an
   automated reviewer, not by either author** — the only one of the five found by review rather
   than by the people who wrote the principle, and the only one already published when found. The
   clause has since been atomized (§7.4, §9.5) and the row corrected twice more; see §8, which
   records how close the corrected row came to being wrong a third time.

**The counter-measure to three of those nine is one practice, and only one project volunteered
it: Baracuda stated the axes its audit did NOT cover** — tautological asserts, timing. In every
one of the three, the defect was not the audit's scope but that **the scope went unstated**, so
the reassurance did not know its own limits.

**The clause form matters more than the practice.** *"An audit MUST state the axes it did not
examine"* is **unfalsifiable as written** — you cannot enumerate what you did not think of, and a
reviewer cannot check that you did. **What is mechanically checkable is presence**, which is why
§6.1-0011 below is worded as it is. **The act of writing the exclusion is what surfaces it:** none
of the three would have written *"searched dtype spellings only; did not examine version
prefixes"* and then reported *clean*.

**A related and cheaper practice, learned the same day.** The §8 falsity was found because a
reviewer asked **"what produced this?"** rather than **"is this true?"** Asking whether a claim is
true invites checking it against the same understanding that accepted it. **Asking what produced
it forces you to the instrument — and an instrument's limits are visible in a way a claim's are
not.** Neither author asked that question about their own sweep; both sweeps were wrong. This is **Pattern B applied to claims rather than
instruments** — a claim without stated scope is a measurement whose limits are indistinguishable
from absent.

**Bound it deliberately.** §6.1-0011 attaches to claims that **gate** something. Applied to
everything it becomes boilerplate; boilerplate gets templated; templated scope statements say
nothing; and then it is a doc comment — **which is B2, one level up.** Two remedies have already
threatened to recreate their own disease within one day: a blanket-panic proposal that would have
recreated silent skips, and over-broad ceremony that would recreate unenforced prose.

6. **KISS architect — reporting a merge-ready state without checking it.** Five PRs were reported
   to the maintainer as *queued for merge*. `gh pr list` showed `reviews=1` on each, which reads
   as **reviewed**, and it was taken as such. **Twelve inline findings sat underneath; eleven were
   unaddressed** — including the false KISS-CONFORM-6.1-0009 claim above, in this document. **A green-looking
   review count and an actually-read review are the same bytes.** Caught by the maintainer asking
   whether the PRs had comments and whether they had been addressed — by a question, not by any
   process.

7. **KISS Lane B — writing the clause, fixing three violations, then writing a fourth.** Named at
   the author's own request, because *"an anonymous instance reads as a hypothetical; a named one
   is the data point."* Their account, in their words: *"I wrote §6.5-0016. I found the defect in
   `target_namespace_registry.rs`, proved it exploitable with a planted fixture, fixed three
   instances of it, wrote a clause forbidding it, wrote a test backing that clause, and then — in
   the next file I authored, within the hour — wrote a fourth instance."* The fourth was
   `clause_definitions` silently `continue`-ing on a read error, in the test file backing the
   clause. **Not fatigue and not carelessness:** `let Ok(x) = read_to_string(p) else { continue }`
   is what Rust invites, the silent form is the **shorter** form, and nothing in the language or
   the review path resists it. **Knowing the rule, having authored the rule, and having just fixed
   three violations of it did not stop the author's hand reaching for it.** This is the argument
   for mechanization stated as strongly as it can honestly be stated. It also names the mechanism:
   **a lint for `else { continue }` / `else { return }` on an I/O result in a discovery context
   would have caught all four instances at authoring time** — the natural enforcement of
   §6.5-0016, rather than a rule people are asked to remember.

8. **KISS Lane B — an audit that almost reported a false absence.** While auditing whether each
   backing test is actually *executed* by CI, a first pass reported `corpus_is_deterministic` as
   **missing from the `cargo test` list** — which would have meant §6.5-0014 cites a test that never
   runs. It was wrong. `cargo test -- --list` emits fully-qualified paths, and the test is
   `harness::corpus::tests::corpus_is_deterministic`; the audit *script* compared bare names
   correctly, and an **ad-hoc shell check written beside it** compared the bare name against the
   full-path list and found nothing. Caught by re-checking before reporting, which is the only
   reason this is an instance and not an incident. **The lesson is about the cost asymmetry: a
   tooling audit that reports a false absence discredits the true findings beside it.** An audit's
   output is believed in proportion to its instrument's reputation, so a single fabricated finding
   is expensive out of all proportion to its size — which makes *verify before reporting* worth more
   here than in ordinary work, not less. Note also the shape: **the careful instrument was right and
   the convenient one beside it was wrong**, which is A7 (a second copy existed and was believed) in
   the auditor's own hands.

9. **KISS architect — ruling a clause backed because a PR implemented it.** The ruling that split
   §6.1-0009 (§9.5) recorded 0009a as *"backed today by #141."* It was not. #141 landed the
   reporting **code**; the only assertion anywhere was that gate *discovery* labels a gate — nothing
   asserted that the **report acts on the label**, so the report could have ignored gate labels
   entirely and every existing check would still have passed. **Implemented is not backed**, and the
   difference is invisible from the PR description that landed both. Caught by the worker
   implementing the ruling, who checked before writing the row and backed the clause for real
   instead. That is the **third consecutive revision** in which this one row overstated its
   backing — *backed*, then *partial*, then *backed-by-#141* — each time by a different person
   reading the same PR, and each time by someone who had just written the rule against it.

**Positive evidence, for balance.** This document is otherwise entirely failures. Two data points
show mechanisms working. First: while building §6.5-0016's backing test, the author's first
fixture wrote the namespace as a **source literal** — and the suite's own completeness check
caught it, because the literal was swept out of the test file. **The check caught the test.**
Obtained by accident while fixing the truncation, which is the most credible provenance available.

Second, and it is the better argument: **of the twelve review findings, the two that mattered were
both a claim outrunning its code** — a doc comment asserting a property the tool never had, and a
stated limit that a reviewer showed was narrower than claimed. **Neither was caught by any check
the author wrote. Both were caught by a reviewer reading carefully.** Every mechanism in this
document is an attempt to make a machine notice what a machine can notice; **the two findings that
most needed noticing were found by a reader.** That is the argument for the review step existing
at all, and it bounds the whole document: mechanization is the floor, not the ceiling.

**Sample bias.** Six projects, one day, people already looking for related things. Strong evidence
the shapes are real; **weak evidence about frequency.** Nothing here establishes how common
either pattern is in code nobody audited.

**A2 is an ambiguity, not a proven under-enforcement.** It is unresolved whether `kiss_trace`
under-enforces §6.1-0003 or the clause needs an internal-unit-test carve-out the tool silently
assumes. **The disagreement itself is the defect** — and the two admissible fixes are
**co-dependent, not alternative**: a carve-out for internal utilities, *plus* clauses claiming the
harness, *then* arm the gate. **Fixing "the obvious half" leaves a worse state than before**, with
a gate firing on a rule nobody finished.

**A closing note.** The obvious objection is *"this is a competence problem — write better
tests."* **Nine instances from the people who articulated the principle, inside the window in
which they articulated it, is the only refutation that cannot be dismissed as special pleading** —
a document about instruments that cannot fail, whose own authors produced that many in a single
day, the fifth **inside this document, caught by a reviewer after publication**, and the ninth a
ruling about the very row the fifth got wrong. That is not evidence the thesis is wrong. It is the
strongest available evidence that it is right.

> **This paragraph is the one place the count appears, and that is deliberate.** It was in three
> places until the sweep that added instances 8 and 9: the §6 heading (*"four"*, stale since the
> fourth), the §6 lead-in (*"seven"*), and here. The updating commit fixed two and missed the
> heading — **one fact, three copies, the checked one updated and an unchecked one left behind, in
> the document that names that pattern, in the commit adding two instances of it.** Not logged as
> a tenth instance; the argument does not need it. It is fixed the way §6.1-0010 prescribes —
> the prose upstream no longer carries the number at all, so there is nothing left to drift.

---

## 7. Normative text

Clause ordinals verified free against `spec/conform.md` at `origin/main` (§6.1 runs 0001..0008;
§6.5 runs 0001..0010).

### 7.1 Pattern A

- **KISS-CONFORM-6.1-0010** — Where a normative fact — a clause set, a vocabulary, a table of
  pinned values, or a coverage claim — appears in **more than one artifact** of the suite, exactly
  one artifact MUST be designated its **single source of truth**, and every other occurrence MUST
  be **derived from it by a mechanical process the build runs**. Where an occurrence cannot be
  derived, the suite MUST **verify its equality** against the source of truth and MUST fail on
  divergence. A duplicate that is neither derived nor verified MUST NOT be treated as normative.
  A **stated invariant that two artifacts agree is not a control** unless something checks it.
  *Test:* `test_conform_normative_duplicates_are_derived`.

### 7.2 Pattern B — the instrument's obligations

> **§6.5-0011 (capability) and §6.5-0016 (attribution) are the two halves of Pattern B, and
> neither implies the other.** An instrument lacking §6.5-0011 is not failing to produce a result
> — it is producing a confident negative, correctly, forever. An instrument lacking §6.5-0016 may
> discriminate perfectly and still lie, because a run that never happened is indistinguishable
> from a run that found nothing. **The predicted failure is an implementor satisfying one and
> believing both are done**, so each clause carries an explicit pointer to the other.
>
> They are presented below in **ascending numeric order** rather than adjacently. Renumbering to
> make them neighbours would churn an open PR that already cites §6.5-0016 by ordinal; the
> cross-references carry the same intent without that cost.

- **KISS-CONFORM-6.5-0011** — The oracle-differential harness of §6.5-0001 MUST be accompanied by
  **executable negative controls** proving it discriminates: for each determinism-class comparator
  (§6.8) the harness employs, the suite MUST include at least one **deliberately incorrect
  candidate implementation** that the harness detects as divergent, and at least one
  **independently written correct candidate** that the harness accepts without divergence. A
  negative control MUST target a behaviour the standard pins normatively, and MUST diverge **only**
  on the inputs where that pinned behaviour is violated — a control that diverges on unrelated
  inputs demonstrates noise, not discrimination. A harness that has never been shown to reject a
  wrong implementation supplies no evidence of conformance, and MUST NOT be presented as satisfying
  §6.5-0001. **See §6.5-0016, the attribution half of Pattern B; satisfying this clause does not
  satisfy that one.** *Test:* `test_conform_harness_negative_controls`.

- **KISS-CONFORM-6.5-0012** — When the harness detects a divergence it MUST report it as
  **structured, reproducible data**, carrying at minimum (a) the **corpus index** of the diverging
  vector, (b) the **operand values** supplied, and (c) **both** the oracle's expected output and the
  candidate's observed output, compared as raw bits where the governing comparator is exact-byte. A
  harness that reports only a boolean verdict or an aggregate count MUST NOT be presented as
  satisfying §6.5-0001, because a divergence that cannot be re-run cannot be triaged into either an
  implementation defect or a specification defect.
  *Test:* `test_conform_divergence_is_localized`.

- **KISS-CONFORM-6.5-0013** — Every failure at the harness's **foreign-artifact boundary** —
  loading a candidate artifact, resolving an entry point within it, or marshalling operands across
  the §6.5 C ABI — MUST be reported as a **typed harness error**. The harness MUST NOT panic, abort,
  or terminate the conformance run on a malformed, missing, or non-conforming artifact, and MUST
  attribute the failure to the **artifact** rather than recording it as a conformance divergence of
  the candidate's numerics. A harness that aborts on a hostile artifact cannot complete a
  conformance run against an untrusted external implementation — the population §5.3 exists to
  measure. *Test:* `test_conform_harness_boundary_typed_decline`.

- **KISS-CONFORM-6.5-0014** — Any **randomized or generated** corpus the harness draws over MUST be
  **reproducible from a recorded seed**: regenerating the corpus from the same seed MUST yield a
  byte-identical vector set, and the seed MUST be recorded alongside any reported divergence. A
  differential result over a corpus that cannot be regenerated is not evidence, because neither the
  divergence nor its subsequent absence can be confirmed.
  *Test:* `test_conform_differential_corpus_reproducible`.

- **KISS-CONFORM-6.5-0015** — The harness MUST define acceptance of a candidate artifact solely by
  its **stated ABI properties** — calling convention, exported symbol name, and the operand
  marshalling layout of the §6.5 C ABI — and MUST NOT condition acceptance on the **toolchain,
  compiler, or language** that produced the artifact. An artifact satisfying the stated ABI MUST be
  admitted for comparison regardless of provenance; an artifact failing it MUST be declined under
  §6.5-0013 as an artifact-boundary error rather than recorded as a numeric divergence.
  *Test:* `test_conform_acceptance_keys_on_abi_not_toolchain`.

> Per §2.7, naming a compiler in normative text would itself be the hazard §2.7 warns of: *"the
> `const_lit` C-ism is the cautionary proof that incidental impl choices leak."* §2.7 frames the
> conformance-relevant property as structural **dissimilarity**, never compiler identity.

- **KISS-CONFORM-6.5-0016** — Where KISS-Conform relies on the result of a **discovery,
  enumeration, or sweep** step — over source files, vectors, registry entries, or artifacts — that
  step MUST distinguish a **genuine negative** ("looked, and found nothing") from a **failure to
  look** ("could not enumerate, wholly or in part"). An enumeration that cannot complete MUST
  surface that fact to its consumer as an error or an explicit incompleteness signal, and MUST NOT
  return a **silently empty or silently truncated** result. A consumer MUST NOT treat a result of
  unknown completeness as authoritative. Where the sweep's purpose is to mechanize a claim that
  would otherwise be hand-maintained, a truncated sweep MUST be treated as a **failure** of that
  check, not as its satisfaction. **See §6.5-0011, the capability half of Pattern B; satisfying
  this clause does not satisfy that one.** *Test:*
  `test_conform_sweep_incompleteness_is_surfaced`.

> **An authoring hazard, recorded because it has now bitten three times in one day.** A test
> fixture that embeds the identifiers its own tooling scans for **will be picked up by that
> tooling**. A planted namespace literal was swept out of the test file that planted it; a
> fixture reusing real clause IDs came back lint-enforced and passed while proving nothing; a
> fixture's clause IDs were reported as dangling citations against the real suite. **Assemble
> fixture identifiers at run time**, not as source literals. The second of those is the dangerous
> one: it produced a green test that verified nothing, which is the very defect §6.5-0016 exists
> to prevent, inside the test written to back it.

### 7.4 Crediting and claims

- **KISS-CONFORM-6.1-0009a** *(static half)* — A conformance test whose execution is conditional —
  on a compile-time feature, or on a run-time precondition such as an absent toolchain or device —
  MUST **declare** that condition in a form the traceability lint can discover; a test that can
  decline to run without declaring it MUST be treated as a defect in the suite. Where **every** test
  backing a clause is so conditioned, the matrix MUST record that clause as **gate-only** and MUST
  report it distinctly from a clause backed by an unconditionally-executing test. A suite MUST NOT
  report a coverage figure that conflates the two: the qualified figure — the one crediting no
  clause whose backing may not have run — MUST be reported alongside the raw one.
  *Test:* `test_kiss_trace_gates.py` §2 (gate-only reporting).

- **KISS-CONFORM-6.1-0009b** *(dynamic half)* — Coverage MUST NOT credit a clause whose backing
  tests did **not execute** in the run being reported. The determination MUST be derived from a
  **run artifact** — the harness's own machine-readable test output (`libtest --format json`) and/or
  the `KISS-SKIP[<gate>]` lines a declining test emits — joined to the clause matrix. A coverage
  figure derived from the *existence* of a backing test rather than its *execution* MUST NOT be
  presented as a per-run figure. *Test:* `test_conform_coverage_excludes_unexecuted_backing`.

> **Why this is two clauses and not one.** The single KISS-CONFORM-6.1-0009 was unenforceable as written, and
> its enforcement gap was invisible in its prose: `kiss_trace` is a **static analyser that never
> executes the harness**, so "did this backing actually run?" is not a property it can observe at
> any level of effort. The tempting fix was to scope the clause down to what the instrument can
> do — which is *fitting the claim to the instrument on hand*, the exact failure this RFC exists to
> name, committed inside it. The other tempting fix was to leave a MUST with no test, which house
> style fails the build for.
>
> **Atomizing keeps the full claim and makes the shortfall a number.** 0009a is observable by a
> static reader and is backed today. 0009b is unbacked, and says why: the joining instrument does
> not exist. Their two figures differ by **exactly the gated backing**, so the gap is reported on
> every run instead of described once in a document. Where a sub-standard's gate-only set is empty
> the two figures coincide, 0009a computes that emptiness, and **a stated zero discharges 0009b for
> free** — which is why this is a freeze precondition only for sub-standards whose gate-only set is
> non-empty, rather than a blanket one.
>
> **The general rule, and it is the one worth carrying off this page:** when the enforcing
> instrument cannot observe the property, the fix is **a second instrument of the right class**, not
> a weaker clause fitted to the instrument on hand.

- **KISS-CONFORM-6.1-0011** — A coverage or conformance claim that **gates a decision** — a freeze
  gate, a published coverage figure, a byte-match leg report, or a conformance assertion — MUST
  carry **the method that produced it** and **an explicit statement of what that method did not
  examine**. A claim reporting a count, a percentage, or a pass without both MUST NOT be treated as
  evidence for the decision it gates. This obligation attaches to claims that gate a decision; it
  MUST NOT be required of every test run. *Test:* `test_conform_gating_claims_state_their_method`.

> **On testability.** §6.1-0011 is checkable as **presence**, not as completeness — a reviewer
> cannot verify that an author enumerated every axis they failed to consider, and a clause demanding
> that would be unfalsifiable. The mechanism is that composing the exclusion sentence surfaces the
> gap to its author. See §6.

---

## 8. Backing status on landing

**Method, per §6.1-0011.** The five "already backed" rows below were established by the clause
author naming the existing test for each and spot-checking, **not** by this document's author
re-running each named test. Coverage figures cited anywhere in this RFC were produced by
`tools/kiss_trace.py` — **the instrument A3 reports as non-conformant to §6.1-0007**, because no
other instrument exists. That provenance is stated rather than elided; a reader should weight
these rows accordingly. **Not examined:** whether these clause ordinals are unclaimed in sibling
projects' mirrors of KISS-Conform.

| clause | backing status | notes |
|---|---|---|
| §6.5-0011 | backed | `integer_differential.rs` seeded-bug controls |
| §6.5-0012 | backed | `harness/differ.rs` `Divergence` carries index + operands |
| §6.5-0013 | **backed — windows-leg-only AND runtime-gated** | `harness/loader.rs` typed missing-symbol path. Two conditions stacked, stated in the row rather than footnoted: the module is `#[cfg(windows)]` (`harness/mod.rs:11`) so it **does not compile on the ubuntu leg at all**, and on windows-latest its tests still `runtime_gate_some!("msvc", …)` and decline if MSVC is absent. **The typed-*decline* clause is evidenced only where the decline path is exercised — which is exactly where the evidence is weakest.** |
| §6.5-0014 | backed | `corpus_is_deterministic`, `corpus_is_reproducible` |
| §6.5-0015 | backed | `harness/abi.rs` marshals rustc-produced `extern "C"` kernels, no toolchain guard |
| §6.5-0016 | backed | `test_conform_sweep_incompleteness_is_surfaced` — landed in #143 (merged `e30de36`) |
| **§6.1-0009a** | **backed** | `test_kiss_trace_gates.py` §2 — fixture suite with one cfg-gated, one runtime-gated and one unconditional backing test; asserts the report names the 2-clause GATE-ONLY set, breaks it out by gate kind, and prints a qualified figure lower than the raw one by exactly that count. Paired with a control asserting an ungated suite reports **no** GATE-ONLY set and **no** qualified figure. Verified by seeded mutation: reverting the report to ignore gate labels fails 5 of the checks. |
| **§6.1-0009b** | **unbacked — structurally, not for want of effort** | `kiss_trace` is a static analyser that **never executes the harness**, so per-run execution is out of its reach by construction — no change to it can back this. It needs a **second instrument**: a joiner consuming `libtest --format json` and the `KISS-SKIP[<gate>]` lines #141's macros emit, matched against the clause matrix. That tool does not exist. |
| **§6.1-0010** | **unbacked** | requires the dtype-table generator (follow-up) |
| **§6.1-0011** | **unbacked** | requires a claim-format check |

**Every row above was then asked a question no row had been asked: *does anything run it?*** The
answers, from `cargo test -- --list` per leg rather than from source inspection: six of the seven
run unconditionally on both CI legs; **one — §6.5-0013 — does not**, as its row now records. The
§6.1-0009a row's test ran **nowhere** until the PR that wrote the row. Related counts from the same
sweep: of 494 discovered harness tests, **483** run unconditionally, **7** are windows-leg-only
*and* runtime-gated, **1** compiles on both legs and declines on ubuntu **every time** by
construction, and **3** never run anywhere — they are `#![cfg(feature = "cuda")]` and no CI leg
passes `--features cuda`. **Those three back no clause: zero, and the structural reason is that no
clause names them and no test cites them** — they are orphans as well as unrun. That zero is worth
stating because a compile-time-gated test is invisible in a way a runtime-gated one is not: it does
not decline, it does not exist, and a `--list` on a leg without the feature cannot report an absence
it never compiled. **Were the count ever non-zero, the report has no annotation that would warn
anyone.**

**So: seven backed, three unbacked.** Not the "seven land green" this section claimed in its first
revision, not the "one partial" of its second, and not the "six backed, three unbacked" of its
third — KISS-CONFORM-6.1-0009 became two clauses, one of which was then backed for real.

> **The backing for §6.1-0009a was written to make this row true, and the row was nearly wrong
> again.** The ruling that split the clause recorded 0009a as *"backed today by #141"*. It was not.
> #141 landed the reporting **code**, and `test_kiss_trace_gates.py` as it stood asserted only that
> `discover_tests` **labels** a gate correctly — nothing anywhere asserted that the report **acts**
> on the label. The report could have ignored gate labels entirely and every existing check would
> still have passed. **Implemented is not backed**, and the distinction is invisible from the PR
> description that landed both. The checks in §2 of that file were added here to close it.
>
> That makes **three** consecutive revisions in which this one row overstated its backing — backed,
> then partial, then backed-by-#141 — each time by an author who had just written the rule against
> it. The row is now the most-checked line in the document and it is still the one to distrust.
>
> **A second thing that check surfaced:** `test_kiss_trace_gates.py` ran **nowhere**. It was not in
> `traceability.yml` or `conformance.yml`, so from #141's merge until this PR, the suite's only
> demonstration that its traceability instrument can fail was itself never executed — **an unrun
> test and an absent test are the same evidence.** It is now a CI step. Note what this means for the
> rows above: a backing claim naming a test says nothing until you also ask **whether anything runs
> it**, which no row in this table had been asked.

> **This table deliberately names no pull requests, and that is the fix.** Its first revision
> recorded rows as *"PR #143, open, not merged"*. #143 merged minutes after this RFC landed, so the
> document shipped a claim the repository had already moved past — **Pattern A, in the document
> about Pattern A, about its own table.**
>
> The first attempt at a fix was to **date** the table. That failed too: the dated note said
> *"#141 still open"*, and #141 merged **while the correcting PR was itself in review.** Three
> staleness events in under two hours.
>
> **The durable fix was not a better timestamp — it was removing the mutable dependency.** Whether
> a clause is backed is a fact about the **repository**, answerable by looking; *which PR carried
> the backing* is provenance, and provenance that names an in-flight artifact ages the moment the
> artifact lands. The rows below name **tests**, which are stable, and not **PRs**, which are not.
>
> The general lesson, and it is stronger than the one §6.1-0010 states: where a duplicate can be
> neither derived nor verified, **the best available move is often to stop stating the volatile
> part at all.**

**How that error was found, and why it belongs in this document.** The KISS-CONFORM-6.1-0009 row originally
read *"backed by `test_kiss_trace_gates.py` (PR #141)"*. It was **wrong**, and it was caught by an
automated reviewer on PR #141 noticing that a doc comment in `conformance/src/lib.rs` claimed
behaviour the tool does not have. **A backing claim was overstated in the table added specifically
to be honest about backing, in the RFC about claims that overstate their backing** — and it was
caught by a reviewer, not by either author. That is the fifth author-committed instance of §6's
meta-failure, and the only one caught by review rather than by the authors themselves.

It also sharpens A2's shape: *a doc comment asserting behaviour the code does not have*, appearing
**inside the PR that fixes Pattern B**. The pattern reproduced itself inside its own remedy.

---

## 9. Open questions for ratification

1. **§6.1-0010's first application** — generating `classify.md` §6.1 and `ops.md` §6.16 from
   `dtype_manifest.json`. Note the limit in §6: generation would **not** have caught the drift that
   motivated it. It prevents the next class, not that one.
2. **A2 / §6.1-0003** — carve-out versus arming the gate. The two fixes are co-dependent; sequence
   is clauses → carve-out → gate.
3. **A3 / §6.1-0007** — whether the absent sidecar is a conformance defect or a pre-freeze
   obligation. KISS is Draft, so the current recommendation is a **freeze-gate item** plus honest
   recategorization of the six dependent clauses from `untested` to `blocked`, with the consequence
   stated: **every coverage figure KISS publishes is derived from prose that §6.1-0007 says must not
   be an authoritative clause source.**
4. **Scope of §6.1-0011** — whether "claims that gate a decision" is drawn tightly enough to avoid
   the boilerplate failure described in §6.
5. **~~Is KISS-CONFORM-6.1-0009 implementable as written?~~ RESOLVED — atomized into 0009a/0009b.** It required
   that coverage not credit a clause whose backing *"did not execute in the run being reported."*
   KISS's traceability instrument is a **static analyser that never executes the harness**, so it
   cannot know what ran. Three resolutions were on the table: **(a)** the instrument also consumes
   test-run output; **(b)** the clause is scoped down to what static analysis can guarantee, which
   is what #141 implements; **(c)** both.

   **The ruling took none of them: the clause was split** (§7.4), on the ground that (b) is *fitting
   the claim to the instrument on hand* — the failure this RFC exists to name — and (a) alone leaves
   a MUST with no test, which house style fails the build for. 0009a is the static half and is
   backed; 0009b is the dynamic half, is unbacked, and names the missing instrument as the reason.
   The split is preferred to either because the two figures **differ by exactly the gated backing**,
   so the gap is a number on every run rather than a caveat in a document.

   **Freeze consequence, scoped rather than blanket.** 0009b is a freeze precondition **for any
   sub-standard whose gate-only set is non-empty**, because a figure the AUDIT role signs at
   KISS-Conform §2.7 would otherwise rest on a test's existence rather than its execution — Pattern
   B sitting on the freeze gate itself. Where the set is empty the two figures coincide, 0009a
   computes that emptiness, and a **stated zero discharges it**. Most sub-standards will discharge
   it for free; the value is that it names which do not.

   **Measured, not predicted — and the answer is worse than the abstraction suggested.** Running
   0009a over the live suite: `ENFORCED = 387/909 (42.6%)`, `ENFORCED, EXCLUDING GATE-ONLY =
   386/909 (42.5%)`. The gate-only set is **one clause**, so eight of the nine sub-standards
   discharge 0009b today with a stated zero. The one that does not is **KISS-Conform**, and the
   single clause in the set is:

   > **KISS-CONFORM-6.13-0006** — backed solely by `test_conform_ops_oracle_and_freeze_gate`,
   > which is `runtime_gate!("msvc", …)`.

   **The one clause in the suite whose backing may not have run is the freeze gate.** On any
   machine without MSVC that test declines, reports `ok`, and — before #141 — the freeze-gate clause
   was credited as backed by a test that did not execute. This is the same clause already flagged
   for **circularity** (its backing test requires the toolchain diversity the gate exists to
   certify). Two independent defects on one clause, and it is the clause that authorizes the freeze.
   The scoped precondition therefore costs almost nothing and lands exactly where it matters: the
   1-clause delta between those two percentages **is** the freeze gate.

   **This is the clause the RFC got wrong three times** — backed, then partial, then
   backed-by-#141 — and the reason is the same each time: nobody asked whether the property was
   reachable by the tool that would have to enforce it. **A clause whose enforcement mechanism
   cannot exist is a claim nothing checks**, which is Pattern A, in this document, about itself.

   **The missing check, and it is cheap enough to be routine:** *name the enforcing instrument,
   then ask whether that instrument can observe the property at all.* KISS-CONFORM-6.1-0009 slipped past two
   readings because **the clause is true and desirable, and nothing about reading it tells you it
   is unreachable by a static analyser.** Desirability and enforceability are independent, and only
   the first is visible in the prose. This check belongs in whatever process ratifies clauses, not
   only in this RFC.
