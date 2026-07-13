# KISS tooling

## `kiss_trace.py` — the traceability checker

Implements the core of **KISS-Conform §6.1** (the bidirectional clause↔test
traceability matrix) and **§6.2** (the suite build fails on any untested
normative MUST), run against the specification text itself.

It parses every `spec/*.md`, extracts each normative clause
`KISS-<SUB>-<section>-<nnnn>[letter]` together with its mapped conformance test,
builds the bidirectional matrix, and reports every traceability violation across
the whole suite at once:

- a clause defined in the body but absent from its §9 matrix (or vice versa);
- a clause whose body `*Test:*` tag disagrees with its §9 matrix row;
- a clause with zero or multiple `*Test:*` tags;
- a duplicate clause ID, or a clause ID whose prefix ≠ its document;
- a test name that lacks the suite `test_` prefix;
- a conformance test mapped to more than one clause (a Conform-owned
  *cross-standard* test cited by a deferring clause is reported as an allowed
  deferral, not a violation);
- a normative clause defined in the informative-only umbrella.

Exit status is `0` when the suite is clean and `1` otherwise, so the checker
doubles as the CI gate (see [`.github/workflows/traceability.yml`](../.github/workflows/traceability.yml)).

### Run

```sh
python tools/kiss_trace.py                 # from the repo root
python tools/kiss_trace.py --spec-dir spec # explicit spec directory
```

Python 3.8+, standard library only — no dependencies.

### Scope — what this does and does not prove

The checker verifies **traceability**: that every normative clause in the
specification has a named conformance test, and that the clause↔test mapping is
consistent and 1:1 across all nine sub-standards. This is the spec-level form of
Conform §6.2 — *no MUST without a test*.

It does **not** yet verify that those tests are *implemented* or that they
*pass*. The test names are stubs the specification pins; the executable
conformance harness (golden byte-vectors, the independent CPU-oracle
differential engine, the IR-DAG fuzzer, and the negative-vector battery of
Conform §6.4–§6.7) is the next phase, and lives in a separate reference crate.

### Latest result

```
853 normative clauses across the nine sub-standards, every one mapped 1:1 to a
named conformance test.  RESULT: CLEAN.
```
