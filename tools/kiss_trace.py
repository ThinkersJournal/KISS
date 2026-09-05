#!/usr/bin/env python3
"""
kiss_trace.py — the KISS-Conform traceability checker.

Implements KISS-Conform §6.1 (the bidirectional clause<->test traceability
matrix) and §6.2 (the suite build FAILS on any untested normative MUST).

It runs TWO independent checks, and both must pass:

  1. DOCUMENT CONSISTENCY (spec/ only) — that each clause names a test and that
     the clause body's `*Test:*` tag agrees with the §9 matrix row. This is a
     check of the markdown against itself; it says NOTHING about whether the
     named test exists.

  2. BINDING (spec/ <-> conformance/) — that the test each clause names is a
     real, executable test function in the conformance harness. §6.2 is a claim
     about *tests*, not about test *names*; without this check the suite can
     report "every clause maps 1:1 to a test" while naming tests that exist
     nowhere. Check 1 alone is a cross-reference spellchecker.

Violations reported by check 1:
  - a clause defined in the body but absent from its §9 matrix (or vice versa)
  - a clause whose body `*Test:*` tag disagrees with its matrix row
  - a clause with zero or multiple `*Test:*` tags
  - a duplicate clause ID, or a clause ID whose prefix != its document
  - a conformance test mapped to more than one clause (a Conform-owned
    cross-standard test cited by a deferring clause is reported as an allowed
    deferral, not a violation)
  - a normative clause defined in the informative-only umbrella

Violations reported by check 2:
  - UNBACKED: a clause names a test that does not exist in conformance/ and is
    not recorded in the unbacked ledger (a NEW untested MUST — always a failure)
  - STALE: a ledger entry whose test now exists (the ledger must only shrink)

The ledger (conformance/UNBACKED.tsv) is the honest, reviewable record of the
clauses that have no executable test yet. It is checked in so the gap is visible
in `git log` rather than hidden behind a green check, and it is a ratchet: a new
unbacked clause fails the build, and a clause that becomes backed must be
removed from the ledger. `--strict` ignores the ledger and fails on ANY unbacked
clause — that is §6.2 as literally written, and it is the target state.

WHERE AN UNTESTED MUST IS A *HARD* ERROR
----------------------------------------
An unbacked clause is deliberately NOT a blocking error on every commit. A gate
that can never go green — including for the very commits that would fix it —
gets bypassed by habit, and a gate bypassed by habit is worse than none: it
decays back into a green check that means nothing, which is the exact failure
this tool exists to remove.

Instead, an unbacked clause hard-fails the two transitions it actually
invalidates, via `--freeze-ready`:

  * umbrella §5.3 condition 3 — a sub-standard advances Draft->Frozen only with
    "complete bidirectional clause-to-test traceability". One unbacked clause
    blocks the freeze.
  * umbrella §8.1 — an implementation "conforms to a sub-standard ... if and
    only if it passes the unmodified KISS-Conform suite for that sub-standard".
    For an unbacked clause there is no suite, so the claim is unbacked for it.

Same predicate, two consequences. The 812 untested MUSTs are therefore errors
where they bite (no sub-standard may freeze; no conformance claim to one is
backed) and a recorded, ratcheted debt everywhere else.

Exit status is 0 when the suite is clean and 1 when any violation is found, so
the checker doubles as the CI gate the standard describes. Stdlib only.

Usage:
  python tools/kiss_trace.py                     # both checks, ratcheted by the ledger
  python tools/kiss_trace.py --strict            # fail on ANY unbacked clause (§6.2 verbatim)
  python tools/kiss_trace.py --freeze-ready      # §5.3 cond. 3 for all nine; fails unless all backed
  python tools/kiss_trace.py --freeze-ready OPS  # ... for one sub-standard
  python tools/kiss_trace.py --update-ledger     # rewrite the ledger to the current truth
  python tools/kiss_trace.py --report            # per-sub-standard coverage breakdown
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from collections import defaultdict

# A clause identifier: KISS-<SUB>-<section>-<4 digits>[optional atomicity letter].
CLAUSE_ID = r"KISS-[A-Z]+-\d+(?:\.\d+)?-\d{4}[a-z]?"

# A clause DEFINITION in the body: a list item opening with the bold id + em dash.
RE_DEF = re.compile(r"^\s*[-*]\s+\*\*(" + CLAUSE_ID + r")\*\*\s*[—–-]", re.M)
# A row of the §9 traceability matrix: | <clause> | <test> |  (test captured
# permissively so a non-conforming test name is parsed and reported, not dropped).
RE_MATRIX = re.compile(r"^\|\s*(" + CLAUSE_ID + r")\s*\|\s*`?([A-Za-z][A-Za-z0-9_]*)`?\s*\|", re.M)
# A markdown heading (a clause block ends at the next heading or the next clause).
RE_HEAD = re.compile(r"^#{1,6}\s+.*$", re.M)
# A `*Test:*` tag inside a clause block. `\s*` spans a line wrap between the tag
# and its backtick-quoted test name.
RE_TEST = re.compile(r"\*Test:\*\s*`([A-Za-z][A-Za-z0-9_]*)`")
RE_IDPART = re.compile(r"^KISS-([A-Z]+)-(\d+(?:\.\d+)?)-(\d{4}[a-z]?)$")

# --- BACKING vs MENTION (#187) ---------------------------------------------------------
# A clause ID counts as a BACKING (real coverage) iff it appears in one of two forms; every
# other occurrence — a fixture literal, a `panic!`/`assert!` message, an explanatory
# `// unlike KISS-X`, an ID spelled only to LOCATE a clause — is a MENTION and earns no credit.
# `cid_re.findall(scope)` credited them all; that is the defect this pair closes.
#
# Form 1 — ASSERTION-ARGUMENT: the clause ID as the FIRST argument of a designated backing
# assertion. The citation and the check are the SAME expression, so aboutness is settled by
# construction. Allow-list = the assertions that take a clause ID they enforce (assert_golden,
# assert_token — the two in the harness today). TWO spellings, both first-arg backings:
#   direct   — assert_golden("KISS-X", ...)
#   indirect — let VAR = "KISS-X"; ... assert_golden(VAR, ...)   (a one-hop variable binding)
# The indirect spelling is the same backing (the clause id IS the assertion's subject), and it
# is common — 21 GRAMMAR clauses use `let _clause = "KISS-X"; assert_golden(_clause, ...)`.
# Matching only the literal would reclassify all of them as MENTIONS, which is the exact
# over-crediting-in-reverse this primitive exists to avoid (#270 review; all 21 happen to be
# forward-backed today, so no floor moved, but a reverse-only indirect backing would be
# silently dropped).
RE_ASSERT_BACKING = re.compile(
    r'\b(?:assert_golden|assert_token)\s*\(\s*"(' + CLAUSE_ID + r')"')
# The two halves of the indirect spelling: a clause-id literal bound to a variable, and that
# variable used as the FIRST argument of a backing assertion. A binding is credited only when
# its variable is such a first argument (a `let x = "KISS-X"` never passed to an assertion is
# still a MENTION). `RE_ASSERT_VAR`'s first char is `[A-Za-z_]`, so it never matches the direct
# `assert_golden("KISS-...` (a `"`), keeping the two forms disjoint.
RE_CLAUSE_LET = re.compile(
    r'\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\s*(?::[^=;{]+)?=\s*"(' + CLAUSE_ID + r')"')
RE_ASSERT_VAR = re.compile(
    r'\b(?:assert_golden|assert_token)\s*\(\s*([A-Za-z_]\w*)\s*[,)]')
# Form 2 — DECLARED backing: a backing KEYWORD (`Backs:` / `Enforces`) in a comment, followed
# with ONLY separators before the clause ID(s). So `/// Enforces KISS-X` and `// Backs: KISS-X,
# KISS-Y` back; a bare `// KISS-X` and prose like `enforces the KISS-X rule` (a word sits
# between the keyword and the id) do NOT. Case-insensitive; captures the whole id run.
RE_BACKING_KEYWORD = re.compile(
    r'(?:Backs|Enforces)\b[:\s]*((?:' + CLAUSE_ID + r'[\s,]*(?:and\s+)?)+)', re.I)
RE_CLAUSE_ID = re.compile(CLAUSE_ID)


def _backing_clauses(body, scope):
    """The clause IDs a test BACKS (not merely MENTIONS, #187): assertion-argument IDs read
    from the executable BODY, plus keyworded-comment IDs read from the whole SCOPE (a citation
    legitimately lives in the doc comment). Everything else the old `cid_re.findall(scope)`
    swept up — messages, fixtures, locate-strings, keyword-less comment IDs — is dropped."""
    ids = {m.group(1) for m in RE_ASSERT_BACKING.finditer(body)}
    # Indirect assertion-argument: a clause-id literal bound to a variable that is then the
    # first arg of a backing assertion (`let VAR = "KISS-X"; ... assert_golden(VAR, ...)`).
    let_bound = {m.group(1): m.group(2) for m in RE_CLAUSE_LET.finditer(body)}
    if let_bound:
        for m in RE_ASSERT_VAR.finditer(body):
            cid = let_bound.get(m.group(1))
            if cid:
                ids.add(cid)
    for m in RE_BACKING_KEYWORD.finditer(scope):
        ids.update(RE_CLAUSE_ID.findall(m.group(1)))
    return ids


# --- PROVEN marker (#278) --------------------------------------------------------------
# A `// Proven:` marker is TESTIMONY that a demonstration exists at a ref — NOT a live
# re-run, and NOT "this is proven now". It records that a seeded mutation of the clause's
# obligation was shown to redden THIS test. Convention 15 makes it evidence only with the
# mutation SUBJECT (impl for an implementation clause, spec-text for a document clause);
# convention 16(a) makes it checkable only with a resolvable REF. So the well-formed shape
# is exactly:
#     // Proven: KISS-X (subject: impl; ref: <sha/PR/issue>)
# A `Proven:` that names a clause without that shape (no subject, no ref, malformed) is
# MALFORMED — flagged, never counted. Fail-closed, because a proof claim missing its
# subject or its ref is not a weaker proof, it is no proof: the whole point of the PROVEN
# tier is that nothing enters it without a demonstration a reader can go re-run.
RE_PROVEN_MARKER = re.compile(
    r'Proven:\s*(' + CLAUSE_ID + r')\s*'
    r'\(\s*subject:\s*(impl|spec-text)\s*;\s*ref:\s*([^)\s][^)]*?)\s*\)', re.I)
RE_PROVEN_LOOSE = re.compile(r'Proven:\s*(' + CLAUSE_ID + r')', re.I)


def _proven_markers(scope):
    """Parse `// Proven:` markers from a test's scope. Returns (well_formed, malformed):
      well_formed — {clause_id: (subject, ref)} for each marker carrying a subject
                    (impl|spec-text) and a non-empty ref;
      malformed   — sorted [clause_id, ...] named by a `Proven:` that does NOT parse to
                    that shape. Fail-closed: a proof claim without a subject or a ref is
                    FLAGGED, never counted.
    The ref is recorded as testimony (a demonstration exists at `ref`), not resolved here —
    the marker is not a live re-run, so this parser does not re-execute anything."""
    # Detect malformed markers by SPAN, not by clause id. Both regexes anchor on `Proven:`, so
    # a well-formed marker and its loose counterpart start at the SAME offset; a loose match that
    # begins nowhere a well-formed one does is a stray malformed marker. Subtracting by id instead
    # (the earlier form) let a well-formed `Proven: X` MASK a stray `// Proven: X` for the same X —
    # good evidence hiding bad, the exact defect this tier exists to expose (Copilot #295).
    well, well_starts = {}, set()
    for m in RE_PROVEN_MARKER.finditer(scope):
        well[m.group(1)] = (m.group(2).lower(), m.group(3).strip())
        well_starts.add(m.start())
    malformed = sorted({m.group(1) for m in RE_PROVEN_LOOSE.finditer(scope)
                        if m.start() not in well_starts})
    # Fail-closed REGARDLESS of what else the scope contains: a clause carrying ANY malformed
    # marker is flagged AND loses its credit here, even if a sibling well-formed marker exists.
    # Otherwise "fail-closed" would name something the code does not do — a second, correct
    # marker could talk the first out of failing.
    for cid in malformed:
        well.pop(cid, None)
    return well, malformed


def collect_proven(harness):
    """#278. From the discovered tests, collect the PROVEN map and its fail-closed violations.
    A clause is PROVEN iff SOME test both BACKS it (the clause is in that test's `clauses`) AND
    carries a well-formed `// Proven:` marker for it — PROVEN ⊆ CITED by construction.

    Returns (proven_map, violations), where proven_map is {clause_id: [proving_test, ...]}
    with the CLAUSE KEYS sorted and each clause's proving-test list sorted — fully
    deterministic, so a fixture may rely on the order and the ratchet's output is stable
    across runs. The MAP, not a bare set, because the 4th ratchet dimension's drop gate needs the
    proving TEST's identity, not just the clause's: a proof drop is green iff its proving test
    no longer exists in the harness, so the gate must know which test carried the testimony
    (the architect's #278 2b ruling). The proven clause SET is `set(proven_map)`.

    A violation is raised (and the clause NOT counted) for:
      * a MALFORMED marker — a `Proven:` naming a clause without a subject and ref;
      * a well-formed marker for a clause the SAME test does NOT back — a proof with no
        backing for it to be about.
    main() drives this; the born-red control in test_kiss_proven_marker.py drives it too, so
    the gate proves the real collector rather than a copy (#279)."""
    proven_map, violations = defaultdict(list), []
    for tname, info in sorted(harness.items()):
        for cid in info.get("proven_malformed", ()):
            violations.append(
                f"malformed `Proven:` in `{tname}` for {cid} — a proof marker must read "
                f"`Proven: {cid} (subject: impl|spec-text; ref: <sha/PR/issue>)`")
        for cid, (_subject, _ref) in sorted(info.get("proven", {}).items()):
            if cid not in info.get("clauses", set()):
                violations.append(
                    f"`Proven:` in `{tname}` for {cid} but that test does NOT back {cid} "
                    f"(proof without a backing — a proof presupposes the citation, PROVEN ⊆ CITED)")
            else:
                proven_map[cid].append(tname)
    return {c: sorted(proven_map[c]) for c in sorted(proven_map)}, violations

# A Rust test function in the harness: `#[test]` (possibly with intervening
# attributes such as `#[cfg(feature = "cuda")]` or `#[ignore]`) then `fn name(`.
RE_WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_]+")
RE_RUST_TEST = re.compile(
    r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z_][a-z0-9_]*)\s*\(", re.M)
# A `#[cfg(feature = "...")]` gate anywhere in the attribute run before a test.
RE_RUST_TEST_CFG = re.compile(
    r"#\[cfg\(feature\s*=\s*\"([a-z0-9_-]+)\"\)\]\s*(?:#\[[^\]]*\]\s*)*"
    r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z_][a-z0-9_]*)\s*\(", re.M)
# A DECLARED RUNTIME GATE inside a test body: `runtime_gate!("cuda", ...)` or
# `runtime_gate_some!("msvc", ...)`. A test that can decline to run at run time
# (absent toolchain, absent device) declares it this way, because a bare
# `eprintln!("SKIP"); return;` is invisible here — such a test reports `ok`
# while asserting nothing, and the clause it backs is credited anyway.
RE_RUNTIME_GATE = re.compile(r"runtime_gate(?:_some)?!\(\s*\"([a-z0-9_-]+)\"")

# spec order: umbrella first (must define no clauses), then the nine sub-standards.
SPECS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]

# DECLARED same-standard test shares (#286). Injective forward names are the default: a §9
# `*Test:*` names one clause. The cross-standard `test_conform_*` deferral is one sanctioned
# exception; this is the other — a test that genuinely backs TWO clauses of the SAME sub-standard,
# where no distinct injective test exists and splitting would manufacture a redundant test to
# satisfy a naming label (#247 ruled against that). A share is allowed ONLY when DECLARED here,
# with the exact clause set and a reason — an UNDECLARED share stays a violation, so a future
# collision must be added deliberately, never appear by omission (the empty-exclusion-list
# discipline). Keyed by test fn name; the value's `clauses` must match the citing set EXACTLY
# (a superset — a third clause naming the test — is not covered and reddens).
DECLARED_SHARES = {
    "test_classify_work_class_element_count": {
        "clauses": frozenset({"KISS-CLASSIFY-6.5-0007", "KISS-CLASSIFY-6.5-0010"}),
        "reason": "one test backs both the work-class ENUM (6.5-0007) and the element-count "
                  "BOUNDARIES (6.5-0010); both mutation-verified #187. No distinct injective "
                  "test exists and splitting would be redundant (#247).",
    },
}

# The categories a clause with no harness test may carry (its 3rd ledger column).
# Every category other than `untested` is an accounted-for state and MUST carry a
# 4th-column note (a lint tool, an issue link, or a one-line reason). `untested`
# is the honest default — a normative MUST we have neither tested, lint-enforced,
# nor explained. Driving THAT count to zero is the real Level-1 target; the others
# are the map of how the rest is handled.
CATEGORIES = {
    "untested",       # genuinely untested MUST, not yet accounted for (default)
    "lint",           # enforced by a document lint (note = tool name); binds the
                      # spec document, not an implementation — a linter's job
    "blocked",        # testable in principle, but the spec has not pinned the
                      # bytes/behaviour yet (note = issue/RFC ref)
    "untestable",     # untestable as written; contradictory/undefined until
                      # reworded (note = filed issue, e.g. #41)
    "definitional",   # a test would be a tautology — the clause states a
                      # definition/ownership with no implementation behaviour
                      # (note = one-line reason). Use sparingly and auditably.
    "decredited",     # an over-credit CORRECTED downward (#261): the clause was
                      # counted harness-backed by a citation that MENTIONS but does
                      # not ASSERT it (#187/#191), and the false credit is removed.
                      # note = the over-credit ref + that a real test is still OWED.
                      # `decredited` and `definitional` point in OPPOSITE directions
                      # in time: `definitional` is TERMINAL (a test would be a
                      # tautology, so it never leaves); `decredited` is a DEBT (a real
                      # test is owed, so it must eventually EMPTY). A bucket holding
                      # both cannot be burned down and cannot answer "how much real
                      # work is outstanding" for either — so they are separate.
}
# A `decredited` clause ROLLS INTO the `untested` count (see `untested_count`): before
# the de-credit it was counted BACKED — falsely; after, unbacked. THE GAP DID NOT
# CHANGE, only our knowledge of it, so `untested` is the only home consistent with the
# number meaning what it claims. Filing it OUTSIDE `untested` would make de-crediting
# IMPROVE the apparent numbers — and a measure that gets better when you find a defect
# in it is gamed by accident, which is worse than deliberately. It is tracked as its
# own SET (the category) only so the ratchet tells an honest de-crediting (#261) from a
# regression: a de-crediting is MARKED, a regression is silent.
#
# PRECONDITION for marking a clause `decredited` (the load-bearing rule; #187/#261). A
# clause enters `decredited` ONLY after a MUTATION confirms the named test no longer
# asserts the obligation. A scanner (or a human) ceasing to RECOGNIZE a citation is a
# FALSE NEGATIVE, not evidence: the backing may be live and merely written in a form the
# recognizer misses — migrate it, do not de-credit it. `decrediting_recorded` cannot tell
# "discovered never-backed" from "recognizer stopped seeing a live backing" — both are
# harness-N / untested+N with a marked ledger — so WITHOUT this precondition the verdict
# would launder a form-change regression into a green, recorded, permanent de-crediting.
#
# And the mutation must target the SUBJECT OF THE OBLIGATION, NOT THE TEXT THAT STATES IT:
# a clause that binds an implementation is backed only if mutating the IMPLEMENTATION
# reddens the test. A test that reddens ONLY when you mutate the SPEC TEXT is backing a
# document-consistency obligation (which may be a real clause) — not the implementation
# clause, and crediting it there is the exact #191 defect the gate exists to catch. So the
# `decredited` note MUST record the SUBJECT of the confirming mutation, not merely that
# "a mutation reddened it" — else the next de-crediting runs a spec-text mutation on an
# implementation clause, sees red, and credits it, passing the gate while committing the
# defect. (Worked example: KISS-EMIT-6.4-000x — the only mutation that reddens their test
# is a spec-text edit; their implementation obligations are untouched, so they de-credit.)
DECREDITED = "decredited"


def untested_count(by_category):
    """The real-gap number: genuinely `untested` plus `decredited` (an over-credit
    corrected still OWES a real test, so it is part of the gap, #261)."""
    return len(by_category.get("untested", ())) + len(by_category.get(DECREDITED, ()))

LEDGER_HEADER = """\
# KISS-Conform — the unbacked-clause ledger.
#
# Each line is a normative clause with NO harness test in conformance/, plus WHY.
# The honest record of the gap between what the spec claims is tested and what is
# actually executable — now categorized, so the gap is legible rather than a
# single undifferentiated number.
#
#   clause_id <TAB> test_name <TAB> category <TAB> note
#
# category is one of:
#   untested      no test, not yet accounted for  <- the number that must reach 0
#   lint:<tool>   enforced by a document lint (binds the doc, not an impl)
#   blocked       spec has not pinned the bytes/behaviour yet (note = issue ref)
#   untestable    contradictory/undefined until reworded (note = filed issue)
#   definitional  a test would be a tautology (note = one-line reason)
# Every non-`untested` category MUST carry a note; a bare `untested` needs none.
#
# It is a RATCHET, enforced by tools/kiss_trace.py:
#   * a clause that becomes unbacked and is not listed here FAILS the build;
#   * a listed clause whose harness test now exists FAILS until its line is
#     removed (the ledger only shrinks);
#   * a `lint:<tool>` category is cross-checked against that lint's declared
#     coverage, so the label cannot outrun the enforcement.
#
# `--strict` gates on `untested` + everything not harness/lint-backed (§6.2).
# `--update-ledger` PRESERVES the category/note of a still-unbacked clause and
# adds a newly-unbacked one as bare `untested`.
#
# clause_id\ttest_name\tcategory\tnote
"""


def _body_span(src, open_brace):
    """The index just past the brace-matched body starting at `open_brace`."""
    depth, i, n = 0, open_brace, len(src)
    while i < n:
        c = src[i]
        if c == '"':  # skip a string literal; a brace inside it is not syntax
            i += 1
            while i < n and src[i] != '"':
                i += 2 if src[i] == "\\" else 1
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def _leading_comment(src, start):
    """The contiguous run of `//` comment lines immediately above `start`."""
    lines, i = [], src.rfind("\n", 0, start)
    while i > 0:
        j = src.rfind("\n", 0, i)
        line = src[j + 1:i].strip()
        if line.startswith("//"):
            lines.append(line)
            i = j
        else:
            break
    return "\n".join(lines)


def discover_tests(conf_dir):
    """Every executable test function in the conformance harness.

    Returns {test_name: {"file", "gate", "clauses"}}. Rust's own test harness is
    the source of truth for what runs, so a test exists iff a `#[test] fn` of
    that name exists. Feature-gated tests (e.g. `cuda`) are recorded but flagged:
    they do not run in the default build.

    `clauses` is the REVERSE half of KISS-Conform §6.1's bidirectional
    traceability — the clause IDs a test cites as the requirements it enforces.
    The harness binds these at the assertion site (`assert_golden("KISS-OPS-...",
    ...)`) and in the comment block above a test; both are read here. A clause
    cited by a real test is backed even if the spec's `*Test:*` name and the
    harness's fn name have drifted, because the citation — not the name — is
    what actually ties a requirement to executable code.
    """
    found = {}
    if not os.path.isdir(conf_dir):
        return found
    for root, dirs, files in os.walk(conf_dir):
        # `target/` is build output, not source; it contains no authored tests.
        dirs[:] = [d for d in dirs if d != "target"]
        for fn in sorted(files):
            if not fn.endswith(".rs"):
                continue
            path = os.path.join(root, fn)
            try:
                src = open(path, encoding="utf-8").read()
            except OSError:
                continue
            rel = os.path.relpath(path, os.path.dirname(conf_dir)).replace(os.sep, "/")
            gated = {name: feat for feat, name in RE_RUST_TEST_CFG.findall(src)}
            for m in RE_RUST_TEST.finditer(src):
                name = m.group(1)
                brace = src.find("{", m.end() - 1)
                body = src[m.start():_body_span(src, brace)] if brace != -1 else m.group(0)
                scope = body + "\n" + _leading_comment(src, m.start())
                # Gate discovery reads the BODY ONLY. A declared gate is an executed
                # statement; prose mentioning `runtime_gate!` — this file's own
                # docs, or a comment explaining the convention — must not mark an
                # ungated test as gated. Citations still scan the wider `scope`,
                # because a citation legitimately lives in the doc comment.
                rt = RE_RUNTIME_GATE.search(body)
                proven_well, proven_bad = _proven_markers(scope)
                found[name] = {
                    "file": rel,
                    # A compile-time `cfg` gate and a declared runtime gate are the
                    # same fact for coverage purposes: this test may not have run.
                    "gate": gated.get(name) or (f"runtime:{rt.group(1)}" if rt else None),
                    # BACKINGS only, not every literal clause ID (#187): an assertion-arg
                    # (assert_golden/assert_token) or a keyworded comment (Backs:/Enforces).
                    # A fixture literal, a panic message, or a bare comment ID is a MENTION.
                    # This drives COVERAGE CREDIT — a clause is backed only by a real backing.
                    "clauses": _backing_clauses(body, scope),
                    # EVERY literal clause ID in the scope — backings AND mentions. Hygiene
                    # checks that care about a REFERENCE, not a backing, read this: the
                    # dangling-citation gate (a bare comment naming a RETIRED id is a stale
                    # reference worth flagging even though it backs nothing, #187/§3.3) and the
                    # citation audit (kiss_cites, which classifies the mentions). Splitting the
                    # two is the point of #187: credit is narrow, reference-hygiene is wide.
                    # lowercase word tokens of the body AND its doc comment (#246).
                    # The body alone would miss a test whose subject is named only in
                    # the comment above it, which is a legitimate style here and would
                    # read as a false negative.
                    "tokens": {t.lower() for t in RE_WORD.findall(scope)},
                    "cited_raw": set(RE_CLAUSE_ID.findall(scope)),
                    # PROVEN markers (#278): {cid: (subject, ref)} of well-formed proof
                    # testimony, and the clause ids of any malformed `Proven:` (flagged,
                    # never counted). A proof is credited only where the SAME test also
                    # backs the clause — enforced in main(), not here.
                    "proven": proven_well,
                    "proven_malformed": proven_bad,
                }
    return found


def _split_category(raw):
    """Split a category cell into (base, note-suffix). `lint:kiss_tables` -> the
    base is `lint`, the inline suffix `kiss_tables` merges into the note."""
    if ":" in raw:
        base, _, suffix = raw.partition(":")
        return base.strip(), suffix.strip()
    return raw.strip(), ""


def read_ledger(path):
    """The recorded unbacked clauses: {clause_id: {test, category, note}}.

    Backward-compatible: a 2-column (clause, test) line reads as `untested`.
    """
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        cid, test = parts[0].strip(), parts[1].strip()
        cat_raw = parts[2].strip() if len(parts) >= 3 and parts[2].strip() else "untested"
        note = parts[3].strip() if len(parts) >= 4 else ""
        base, suffix = _split_category(cat_raw)
        if base not in CATEGORIES:
            base = "untested"
        if suffix and not note:
            note = suffix
        out[cid] = {"test": test, "category": base, "note": note,
                    "lint": suffix if base == "lint" else ""}
    return out


# The ratchet's dimension SET — ONE constant driving BOTH the required-key check and the
# call-site dispatch (#271).
#
# THE BUG THIS FIXES WAS THE HARDCODED TUPLE, NOT A MISSING NAME. #297 added `proven` as a
# fourth BLOCKING dimension but wired it at the call site as `if "proven" in floor:`, while
# the required-key guard still read `("harness", "lint", "untested")`. That makes a blocking
# dimension OPT-IN FROM THE VERY FILE IT CONSTRAINS: delete the row, or typo it to `proen`,
# and the dimension is switched off while the ratchet reports CLEAN, exit 0. Measured at
# origin/main @ 083917d — no proven verdict emitted at all in either arm.
#
# A WRONG VALUE IS COMPARED AND REDS. A WRONG KEY WAS NEVER COMPARED AT ALL. Adding "proven"
# to a second hardcoded tuple would fix today and reproduce this the next time a dimension is
# added, so both sites read this constant and a test asserts EVERY member is load-bearing.
DIMENSIONS = ("harness", "lint", "untested", "proven")


def read_floor(path):
    """The committed ratchet floor: `({key: int}, [problem, ...])` from a `key<TAB>value` TSV.

    Hand-edited and never written by this tool. A floor the tool maintains is a
    state derived from the run rather than from a decision, and it would silently
    absorb a regression that coincided with an improvement elsewhere.

    Returns PROBLEMS alongside the floor rather than raising: an unknown key and a duplicate
    key are both states a reviewer cannot see by reading the file — a typo leaves the right
    NUMBER of rows, a duplicate leaves the right KEYS — so they are reported by the tool or
    not at all. The parser takes the LAST occurrence of a duplicated key (demonstrated, not
    assumed), which is why a duplicate is an error here rather than a silently-resolved shadow.
    """
    floor, problems, seen = {}, [], set()
    if not os.path.exists(path):
        return floor, problems
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("	")
            if len(parts) >= 2 and parts[1].strip().isdigit():
                key = parts[0].strip()
                if key in seen:
                    problems.append(f"duplicate key `{key}` — the parser takes the LAST "
                                    "occurrence, so an earlier row is silently shadowed.")
                seen.add(key)
                if key not in DIMENSIONS:
                    problems.append(f"unknown key `{key}` — not a ratchet dimension. A "
                                    "misspelled dimension leaves the file LOOKING complete "
                                    "while that dimension is switched off.")
                floor[key] = int(parts[1].strip())
    return floor, problems


def _ledger_lint_ids(text):
    """The `lint`-category clause IDs from an UNBACKED.tsv body (any `lint...` in the
    category column). The ledger is the only committed artifact that holds a PREVIOUS
    state, which the count-only floor cannot."""
    ids = set()
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("	")
        if len(parts) >= 3 and parts[2].strip().startswith("lint"):
            ids.add(parts[0].strip())
    return ids


def _ledger_decredited_ids(text):
    """The `decredited`-category clause IDs from an UNBACKED.tsv body (#261). A clause
    listed `decredited` at BASE was already a corrected over-credit before this PR; one
    that is `decredited` NOW but was not at base is THIS PR's honest de-crediting. The
    base-vs-disk diff on this set is what tells a de-crediting from a regression."""
    ids = set()
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("	")
        if len(parts) >= 3 and parts[2].strip() == DECREDITED:
            ids.add(parts[0].strip())
    return ids


def _ledger_all_ids(text):
    """EVERY clause ID in an UNBACKED.tsv body (col 0), any category. The ledger's domain is
    exactly the UNBACKED clauses, so `id in ledger` == `id is NOT harness-backed`. That
    equivalence is what the #267 condition-3 downgrade check rests on: a now-lint clause that
    is present in the base SPEC but ABSENT from the base ledger was harness-backed at base — a
    harness->lint downgrade, not a born-with-detector arrival. Free to compute: the same base
    ledger string `base_ledger_lint` already fetches and parses is reused here."""
    ids = set()
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if parts and RE_IDPART.match(parts[0].strip()):
            ids.add(parts[0].strip())
    return ids


def base_ledger_all_ids(ledger_path, base_ref):
    """Every clause ID in the ledger AT `base_ref` — the PRE-change UNBACKED domain, read by
    REF via `git show` (#213). Returns the ID set, or None when the base cannot be read; the
    caller treats None as 'condition 3 uncertifiable' and declines the arrival green."""
    ledger_dir = os.path.dirname(os.path.abspath(ledger_path))
    try:
        top = subprocess.run(["git", "-C", ledger_dir, "rev-parse", "--show-toplevel"],
                             capture_output=True, text=True, timeout=30)
        if top.returncode != 0:
            return None
        rel = os.path.relpath(ledger_path, top.stdout.strip()).replace(os.sep, "/")
        out = subprocess.run(["git", "-C", ledger_dir, "show", f"{base_ref}:{rel}"],
                             capture_output=True, text=True, timeout=30)
    except Exception:
        return None
    if out.returncode != 0:
        return None
    return _ledger_all_ids(out.stdout)


def base_spec_clause_ids(spec_dir, stem, base_ref):
    """The clause IDs DEFINED in spec/<stem>.md AT `base_ref`, read by REF via `git show`
    (#213). Used by the #267 condition-3 check to tell a BRAND-NEW clause (absent here -> a
    legitimate born arrival, e.g. 6.8-0012) from a PRE-EXISTING one (present here; if it is also
    absent from the base ledger it was harness-backed -> a downgrade). One `git show` per
    distinct sub-standard among the arrived clauses (typically one). Returns the ID set, or None
    when the base doc cannot be read (fail-closed: the caller declines the arrival green)."""
    spec_path = os.path.join(spec_dir, stem + ".md")
    base_dir = os.path.dirname(os.path.abspath(spec_path))
    try:
        top = subprocess.run(["git", "-C", base_dir, "rev-parse", "--show-toplevel"],
                             capture_output=True, text=True, timeout=30)
        if top.returncode != 0:
            return None
        rel = os.path.relpath(spec_path, top.stdout.strip()).replace(os.sep, "/")
        out = subprocess.run(["git", "-C", base_dir, "show", f"{base_ref}:{rel}"],
                             capture_output=True, text=True, timeout=30)
    except Exception:
        return None
    if out.returncode != 0:
        return None
    return {m.group(1) for m in RE_DEF.finditer(out.stdout)}


def _in_git_repo(path):
    """Whether `path` is inside a git work tree — i.e. whether a base ledger CAN be read.
    When it can, `--ratchet` requires --base-ref: a constant-count lint<->harness swap is
    invisible to the counts, so 'the counts didn't move' is not a licence to skip the set
    comparison (#213). Only a genuinely git-less checkout is exempt, and it must say so."""
    try:
        out = subprocess.run(["git", "-C", path, "rev-parse", "--is-inside-work-tree"],
                             capture_output=True, text=True, timeout=30)
    except Exception:
        return False
    return out.returncode == 0 and out.stdout.strip() == "true"


def base_is_current(ledger_path, base_ref):
    """Is `base_ref` an ancestor of the working head?

    RETURNS, and the caller contract is not truthiness:
      True  -- the base IS an ancestor; the figures are current.
      int   -- it is NOT, and this is how far ahead it has moved (always >= 1).
      None  -- unknowable (git absent, ref unresolvable, count unreadable).

    TEST `is True` / `is None` EXPLICITLY. `if not base_is_current(...)` is wrong for
    None, and `is False` never matches because False is never returned -- the distance
    is carried instead of a bare flag so the message can say HOW FAR the base has moved,
    which is what tells a reader whether they are one merge behind or a day behind.

    THE RATCHET COMPARES THE BRANCH'S FLOOR AGAINST THE BRANCH'S LIVE FIGURES. Those can
    agree with each other while BOTH disagree with the base -- so a branch that has sat
    while main moved reports CLEAN, correctly, about a tree nobody is merging into. That
    happened four times in one session and every time the signal was a GREEN.

    The question the ratchet cannot otherwise ask is one predicate: is the base still an
    ancestor of what I am measuring? `git merge-base --is-ancestor` answers it with no
    judgement, and returns the distance for the message.

    NOT A VIOLATION. A stale base means the figures were not certified against the tree
    they will merge into -- a refusal to certify CURRENCY, which is `INCONCLUSIVE`'s
    existing shape (exit 2: not a pass, not a breach). The counts are still compared and
    a genuine floor breach still OUTRANKS it, because a refusal must never mask one.

    `None` when ancestry cannot be determined (git absent, ref unknown). The caller does
    NOT claim staleness on None: the base-ledger read fails on the same conditions and
    already reports its own refusal, so claiming both would double-report one cause.
    """
    ledger_dir = os.path.dirname(os.path.abspath(ledger_path))
    try:
        r = subprocess.run(["git", "-C", ledger_dir, "merge-base", "--is-ancestor",
                            base_ref, "HEAD"], capture_output=True, text=True, timeout=30)
        if r.returncode not in (0, 1):
            return None
        if r.returncode == 0:
            return True
        c = subprocess.run(["git", "-C", ledger_dir, "rev-list", "--count",
                            f"HEAD..{base_ref}"], capture_output=True, text=True, timeout=30)
        # A count we could not read is UNKNOWABLE, not zero. Returning 0 here would claim
        # "moved 0 commit(s) ahead" -- a distance that cannot occur for a non-ancestor -- and
        # 0 is falsy, so a truthiness-testing caller would read the stale base as current.
        if c.returncode != 0 or not c.stdout.strip().isdigit():
            return None
        return int(c.stdout.strip()) or None
    except Exception:
        return None


def base_ledger_lint(ledger_path, base_ref):
    """The `lint`-category clause IDs in the ledger AT `base_ref` — the PRE-change state.

    Read by REF via `git show`, NEVER from disk. An on-disk ledger regenerated before
    `--ratchet` is already the POST-change state, so comparing to it reports a real
    lint<->harness movement as 'no movement' and passes silently — the currency hazard,
    convention 11 arriving inside the fix for convention 11 (#213).

    Resolved against the LEDGER's own repository (not the tool's), so a fixture ledger in a
    git-less tempdir reads as git-less rather than borrowing the tool's repo. Returns the ID
    set, or None when the base cannot be read (not a git repo, ref unknown, git absent). The
    caller MUST fail loud on None, never degrade to an empty diff — a degraded diff is the
    same silent pass, reached through the environment instead of through ordering.
    """
    ledger_dir = os.path.dirname(os.path.abspath(ledger_path))
    try:
        top = subprocess.run(["git", "-C", ledger_dir, "rev-parse", "--show-toplevel"],
                             capture_output=True, text=True, timeout=30)
        if top.returncode != 0:
            return None
        rel = os.path.relpath(ledger_path, top.stdout.strip()).replace(os.sep, "/")
        out = subprocess.run(["git", "-C", ledger_dir, "show", f"{base_ref}:{rel}"],
                             capture_output=True, text=True, timeout=30)
    except Exception:
        return None
    if out.returncode != 0:
        return None
    return _ledger_lint_ids(out.stdout)


def base_ledger_decredited(ledger_path, base_ref):
    """The `decredited`-category clause IDs in the ledger AT `base_ref` (#261) — the
    PRE-change set, read by REF via `git show`, never from disk (same currency hazard as
    `base_ledger_lint`, #213). Returns the set, or None when the base cannot be read; the
    caller MUST fail loud on None rather than treat an unreadable base as an empty set."""
    ledger_dir = os.path.dirname(os.path.abspath(ledger_path))
    try:
        top = subprocess.run(["git", "-C", ledger_dir, "rev-parse", "--show-toplevel"],
                             capture_output=True, text=True, timeout=30)
        if top.returncode != 0:
            return None
        rel = os.path.relpath(ledger_path, top.stdout.strip()).replace(os.sep, "/")
        out = subprocess.run(["git", "-C", ledger_dir, "show", f"{base_ref}:{rel}"],
                             capture_output=True, text=True, timeout=30)
    except Exception:
        return None
    if out.returncode != 0:
        return None
    return _ledger_decredited_ids(out.stdout)


def base_floor_harness(floor_path, base_ref):
    """The `harness` count in COVERAGE_FLOOR.tsv AT `base_ref` — the PRE-change floor,
    read by REF via `git show`, never from disk (#213). Used to gate the born-arrival
    verdict (#267): a genuine arrival adds a NEW clause and NEVER retires a harness backing,
    so `live.harness < base_floor_harness` means a harness clause was lost across the floor
    bump — a harness->lint downgrade the count-conservation check cannot see once the floor
    is bumped to absorb the drop (its `harness_delta` reads 0). Returns the int, or None when
    the base cannot be read; the caller treats None as 'condition 3 uncertifiable' and declines
    the green rather than passing an unchecked gate."""
    floor_dir = os.path.dirname(os.path.abspath(floor_path))
    try:
        top = subprocess.run(["git", "-C", floor_dir, "rev-parse", "--show-toplevel"],
                             capture_output=True, text=True, timeout=30)
        if top.returncode != 0:
            return None
        rel = os.path.relpath(floor_path, top.stdout.strip()).replace(os.sep, "/")
        out = subprocess.run(["git", "-C", floor_dir, "show", f"{base_ref}:{rel}"],
                             capture_output=True, text=True, timeout=30)
    except Exception:
        return None
    if out.returncode != 0:
        return None
    for line in out.stdout.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) >= 2 and parts[0].strip() == "harness" and parts[1].strip().isdigit():
            return int(parts[1].strip())
    return None


def classify_ratchet(floor, live, live_lint, live_harness, prev_lint, disk_lint=None,
                     live_decredited=None, prev_decredited=None, disk_decredited=None,
                     live_coverage=None, prev_floor_harness=None,
                     base_ledger_all=None, base_spec_ids=None):
    """Classify a `--ratchet` comparison. Returns (verdict, lines); verdict is one of
    incomplete | regression | substitution | substitution_recorded | decrediting |
    decrediting_recorded | arrival_recorded | ledger_unverifiable | uncharacterized |
    lint_drift | stale | at_floor | at_floor_unchecked. `at_floor`, `at_floor_unchecked`,
    `substitution_recorded`, `decrediting_recorded`, and `arrival_recorded` are green; the rest
    set `any_fail` in the caller.

    `substitution` is the IN-PROGRESS state (floor still at PRE, "bump the floor to N");
    `substitution_recorded` is the same move once the floor is bumped to POST and the ledger
    dropped the moved IDs — green, because the deliberate floor move is complete (#223).

    `decrediting` / `decrediting_recorded` are the same pair for an HONEST harness->decredited
    correction (#261): a false credit from a MENTION, not an assertion (#187/#191), removed
    downward. In the counts this is harness -N / untested +N — byte-identical to a regression;
    the base->disk `decredited` SET is the discriminator (a de-crediting is MARKED, a
    regression is silent). `decredited` rolls into the `untested` total but is its own set.

    `arrival_recorded` (#267) is the born-with-detector case: a new normative clause and its
    lint enforcement land in the SAME PR, so the clause arrives in the lint set (`arrived_lint`)
    with the floor's lint count already bumped. Its count shape is identical to `lint_drift` (and
    to a silent harness->doc downgrade), so it is green ONLY when genuine: every arrival is (1)
    recorded `lint:<tool>` in the current ledger, (2) declared by that tool's --emit-coverage
    (`live_coverage`), and (3) not harness-backed at base. Condition 3 is two fail-closed gates:
    the base FLOOR harness count (`prev_floor_harness`, a cheap first filter — the floor's harness
    must not have dropped) and the DECIDING base LEDGER + SPEC sets (`base_ledger_all`,
    `base_spec_ids`): since the ledger domain IS the unbacked set, an arrival present in the base
    spec but absent from the base ledger was harness-backed -> a downgrade, red. Unlike
    `decredited`, arrival earns NO ledger category: it is a TRANSITION (base vs current), green
    once at the landing PR, and self-disposes next run.

    When `prev_lint` (the base ledger's lint set) is given, the lint dimension is compared
    as a SET — the identity check that tells a lint->harness SUBSTITUTION (a strengthening
    ENFORCED conserves and so cannot show, #187/#213) apart from a laundered regression
    whose three COUNTS are byte-identical to it (X: lint->untested plus Y: untested->harness
    nets harness +1 / lint -1 / untested flat, exactly a substitution's signature). When
    `prev_lint` is None the run is GENUINELY GIT-LESS (in a git repo the caller requires
    --base-ref, #213): the count dimensions are characterized and the lint dimension is
    reported as NOT characterized — never `at_floor`, because a constant-count swap looks
    exactly like at-the-floor to the counts.
    """
    missing = [k for k in DIMENSIONS if k not in floor]
    if missing:
        return ("incomplete", [f"floor is missing key(s): {', '.join(missing)} — a dimension "
                               "silently absent is that dimension switched off."])

    untested_delta = live["untested"] - floor["untested"]   # +ve worse
    harness_delta = live["harness"] - floor["harness"]       # -ve worse
    # ONE lint comparison, read by BOTH paths (#320). It used to be written inline inside the
    # git-less branch below, so on the path that actually ships — a git checkout, where
    # --base-ref is required — the floor's `lint` VALUE was never compared at all: floor 28 /
    # 32 / 34 / 38 against live 33 each returned CLEAN, and the at-floor line printed the LIVE
    # number, so a floor off by five read as consistent. A SECOND comparison on the git path
    # would be the divergence `DIMENSIONS` was introduced to prevent in #271, one function over.
    lint_delta = live["lint"] - floor["lint"]                # -ve worse

    if prev_lint is None:
        # GIT-LESS: no base ledger to diff the lint SET against. The caller reaches here ONLY
        # in a genuinely git-less checkout (in a git repo, --base-ref is required, #213). We
        # characterize the two COUNT dimensions we can see and say PLAINLY that the lint
        # dimension was not characterized — printing `at_floor` here is the exact hole the
        # count-gated base read left: a constant-count lint<->harness swap looks at-the-floor.
        if lint_delta:
            return ("uncharacterized", [
                f"the lint count moved ({floor['lint']} -> {live['lint']}) but this is a git-less "
                "run: a substitution and a regression cannot be told apart without the base "
                "ledger. Re-run in a git checkout with --base-ref <base>."])
        reg = []
        if untested_delta > 0:
            reg.append(f"untested rose {floor['untested']} -> {live['untested']}: a clause lost "
                       "its only backing.")
        if harness_delta < 0:
            reg.append(f"harness fell {floor['harness']} -> {live['harness']}: a behavioral "
                       "backing disappeared.")
        if reg:
            return ("regression", reg)
        if harness_delta > 0 or untested_delta < 0:
            b = ([f"harness {floor['harness']} -> {live['harness']}"] if harness_delta > 0 else []) \
                + ([f"untested {floor['untested']} -> {live['untested']}"] if untested_delta < 0 else [])
            return ("stale", [f"coverage improved past the floor: {', '.join(b)}.",
                              f"Set the floor to harness {live['harness']}, untested "
                              f"{live['untested']}. Green means AT the floor, never under it."])
        return ("at_floor_unchecked", [
            f"at the floor on COUNTS - harness {live['harness']}, lint {live['lint']}, untested "
            f"{live['untested']}. LINT DIMENSION NOT CHARACTERIZED: git-less run, no --base-ref, "
            "so a constant-count lint<->harness swap would be invisible here."])

    # --- SET-based lint dimension (prev_lint given) ---
    left_lint = prev_lint - live_lint                              # were lint, no longer
    arrived_lint = live_lint - prev_lint                           # lint now, were not
    left_to_harness = {c for c in left_lint if c in live_harness}  # lint -> harness (upgrade)
    left_lost = left_lint - left_to_harness                        # documentary coverage gone
    # DE-CREDITING SET (#261): clauses `decredited` NOW but not at base. A harness->untested
    # correction of a false credit is harness -N / untested +N, IDENTICAL to a regression in the
    # counts; this SET is the discriminator — a de-crediting is MARKED, a regression is silent.
    newly_decredited = (set() if live_decredited is None or prev_decredited is None
                        else live_decredited - prev_decredited)

    # COMPLETED SUBSTITUTION (#223): the counts are already AT the floor — the PR bumped
    # harness +N / lint -N to record the move — AND the base-ledger SET shows a clean
    # lint->harness upgrade (clauses left lint, all now harness, nothing arrived, nothing lost).
    # `harness_lost` below MISFIRES here: Δharness is 0 against the bumped floor while
    # left_to_harness is N, so `N - 0 = N` reads as a loss and prints "N disappeared (harness
    # X -> X)", a message that contradicts its own parentheses. The count baseline (the floor,
    # bumped to POST) and the set baseline (the base ledger, still PRE) diverge in a
    # substitution PR; recognize the completed move before the count-conservation trick runs.
    counts_at_floor = (harness_delta == 0 and lint_delta == 0
                       and untested_delta == 0)
    if counts_at_floor and left_to_harness and not arrived_lint and not left_lost:
        # Green only if the ON-DISK ledger also dropped the moved IDs from its lint set. Else the
        # base ledger lists them as lint forever, left_to_harness stays non-empty, and this branch
        # fires green on every later UNRELATED PR — a substitution reported commits after it
        # happened. #215 said "remove the moved ID(s) from the ledger", but an instruction is not
        # a gate; the ledger update IS the gate.
        if disk_lint is None:
            # The ledger could not be read, so the gate could not run. `None or set()` would make
            # "I could not read the ledger" indistinguishable from "the ledger is clean" — and it
            # resolves GREEN, the exact degradation this gate exists to prevent, arriving through
            # the environment instead of through staleness (cf. an indeterminable base ref, #213).
            # Refuse to characterize rather than degrade to a silent pass.
            return ("ledger_unverifiable", [
                f"the floor records a lint->harness substitution of {len(left_to_harness)} "
                f"clause(s), but the on-disk ledger (UNBACKED.tsv) could not be read to confirm "
                "they were dropped from its lint set. An unreadable ledger is NOT a clean one; "
                "refusing the green rather than passing an unchecked gate."])
        stale = sorted(left_to_harness & disk_lint)
        if stale:
            return ("regression", [
                f"the floor records a lint->harness substitution of {len(left_to_harness)} "
                f"clause(s), but UNBACKED.tsv still lists {len(stale)} of them as lint: "
                f"{', '.join(stale)}. Remove them from the ledger — otherwise this reads as a "
                "fresh substitution on every later PR."])
        return ("substitution_recorded", [
            f"at the floor - harness {live['harness']}, lint {live['lint']}, untested "
            f"{live['untested']}.",
            f"A lint->harness substitution of {len(left_to_harness)} clause(s) is RECORDED: "
            f"{', '.join(sorted(left_to_harness))} left the base ledger's lint set, are now "
            "harness-backed, and both the floor and the ledger reflect the move. Green."])

    # COMPLETED DE-CREDITING (#261): counts already at the (bumped) floor AND the base->disk
    # `decredited` SET shows N clauses newly de-credited. Recognized before the count-conservation
    # regression below, exactly as the completed substitution is — the POST floor and the PRE base
    # ledger diverge in a de-crediting PR, so `harness_lost` would misread the recorded move as a
    # loss. `decredited` rolls into `untested`, so both counts are at the floor here.
    if counts_at_floor and newly_decredited and not left_to_harness:
        if disk_decredited is None:
            # Same degradation the substitution gate refuses: an unreadable ledger is not a clean
            # one, and `None or set()` would resolve GREEN through the environment (#213/#225).
            return ("ledger_unverifiable", [
                f"the floor records a de-crediting of {len(newly_decredited)} clause(s), but the "
                "on-disk ledger (UNBACKED.tsv) could not be read to confirm they are listed "
                "`decredited`. An unreadable ledger is NOT a clean one; refusing the green."])
        unrecorded = sorted(newly_decredited - disk_decredited)
        if unrecorded:
            return ("regression", [
                f"the base->disk diff shows {len(newly_decredited)} clause(s) newly de-credited, "
                f"but UNBACKED.tsv does not list {len(unrecorded)} of them `decredited`: "
                f"{', '.join(unrecorded)}. A de-crediting must be RECORDED in the ledger, else it "
                "reads as a fresh correction on every later PR."])
        return ("decrediting_recorded", [
            f"at the floor - harness {live['harness']}, lint {live['lint']}, untested "
            f"{live['untested']}.",
            f"An honest DE-CREDITING of {len(newly_decredited)} clause(s) is RECORDED: "
            f"{', '.join(sorted(newly_decredited))} were credited by a MENTION, not an assertion "
            "(#187/#191); the false credit is removed, they are listed `decredited`, and the floor "
            "reflects the move. Green — the gap did not change, only our knowledge of it."])

    # IN-PROGRESS DE-CREDITING (#261): floor still at PRE. harness fell and untested rose by
    # EXACTLY the newly-decredited set (they roll into untested), and the ledger already lists
    # them. Report "bump the floor", NOT a regression — intercept before the regression block
    # below, which would fire on both `untested rose` and `harness_lost`.
    if (newly_decredited and not left_to_harness and harness_delta < 0 and untested_delta > 0
            and disk_decredited is not None and not (newly_decredited - disk_decredited)
            and len(newly_decredited) == -harness_delta == untested_delta):
        return ("decrediting", [
            f"{len(newly_decredited)} over-credit(s) corrected downward (#261): "
            f"{', '.join(sorted(newly_decredited))} were credited by a mention, not an assertion "
            "(#187/#191).",
            f"Update the floor to harness {live['harness']}, untested {live['untested']} "
            "(lint unchanged), keeping them listed `decredited`. This is a DE-CREDITING, not a "
            "regression — the gap did not change, only our knowledge of it."])

    # Behavioral backings that vanished — directly, OR masked by a compensating lint->harness
    # arrival that held the harness count flat. Count conservation recovers the masked case
    # (`|left_to_harness| - Δharness`), so the floor need NOT carry a previous HARNESS set —
    # only the lint set, which the ledger already holds. It reads like an omission; it is not.
    harness_lost = len(left_to_harness) - harness_delta

    reg = []
    if untested_delta > 0:
        reg.append(f"untested rose {floor['untested']} -> {live['untested']}: a clause lost its "
                   "only backing.")
    if left_lost:
        reg.append(f"{len(left_lost)} clause(s) left the lint set without gaining a harness test "
                   f"— documentary coverage lost: {', '.join(sorted(left_lost))}.")
    if harness_lost > 0:
        if arrived_lint:
            m = " (masked by an offsetting substitution)" if left_to_harness else ""
            reg.append(f"{harness_lost} behavioral backing(s) replaced by documentary — a "
                       f"harness->lint downgrade{m}. Restore the BEHAVIORAL test for: "
                       f"{', '.join(sorted(arrived_lint))}.")
        else:
            reg.append(f"{harness_lost} behavioral backing(s) disappeared "
                       f"(harness {floor['harness']} -> {live['harness']}).")
    if reg:
        return ("regression", reg)

    if left_to_harness and not arrived_lint:
        return ("substitution", [
            f"{len(left_to_harness)} clause(s) moved from documentary (lint) to behavioral "
            f"(harness) backing: {', '.join(sorted(left_to_harness))}.",
            "A behavioral test is stronger evidence than a doc lint, but ENFORCED is conserved so "
            "the aggregate cannot show the gain — the count-only ratchet called this a regression "
            "and told you to undo it (#213).",
            f"Update the floor to harness {live['harness']}, lint {live['lint']}, untested "
            f"{live['untested']}, and remove the moved ID(s) from the ledger. This is a "
            "SUBSTITUTION, not a regression."])

    # BORN-WITH-DETECTOR ARRIVAL (#267): clauses that newly appear in the lint set because they
    # were CREATED with their lint detector in this PR — a new normative clause and its
    # enforcement landing together. In the counts this is a lint bump the floor already reflects
    # (counts_at_floor), harness and untested flat. It reads EXACTLY like `lint_drift` — the count
    # signature of a fine new enforcement and of a silent harness->doc downgrade are identical —
    # so it needs the same discipline as the recorded substitution/de-crediting: the move is GREEN
    # only when it is verifiably genuine, never on the count shape alone.
    #
    # Asymmetry with `decredited` (state) vs arrival (transition): a de-credited clause is a STATE
    # the ledger could not otherwise express (we thought it backed and it never was), so it earns
    # a category; an arrival has no distinguishing end state — next PR it is an ordinary lint row,
    # indistinguishable from one lint-backed since the file was created, AND IT SHOULD BE. So
    # arrival leaves NO ledger category: it is green once, at the landing PR, and self-disposes
    # (next run: arrived_lint is empty because the base now contains it) — no floor bookkeeping.
    #
    # `not left_to_harness`: keep this to the clean single-transition case; a PR that both
    # substitutes and arrives falls through to the substitution/lint_drift handling below.
    if counts_at_floor and arrived_lint and not left_to_harness:
        if disk_lint is None:
            # Condition 1 cannot be checked without the ledger. An unreadable ledger is not a
            # clean one; refuse the green rather than pass an unchecked gate (#223/#225).
            return ("ledger_unverifiable", [
                f"{len(arrived_lint)} clause(s) newly appear lint-backed, but the on-disk ledger "
                "(UNBACKED.tsv) could not be read to confirm they are recorded `lint:<tool>`. "
                "An unreadable ledger is NOT a clean one; refusing the green."])
        # Condition 1: every arrival is recorded `lint:<tool>` in the CURRENT on-disk ledger.
        not_in_ledger = sorted(arrived_lint - disk_lint)
        # Condition 2: every arrival is DECLARED by a running tool's --emit-coverage. In the real
        # caller `live_coverage` is the set of emit-covered clause IDs and `live_lint` is built as
        # (ledger-unbacked ∩ emit-coverage), so arrived_lint ⊆ live_coverage holds by construction
        # — the check is belt-and-suspenders AND the seam a fixture can break to prove the gate is
        # live. `None` means the coverage was not supplied (git-less/older caller): cannot certify
        # condition 2, so fall through to `lint_drift` rather than green on an unchecked condition.
        # NOTE / known limit (surface in the PR, per #267 review): --emit-coverage is an ASSERTION,
        # not a proof — an inert lint that confidently emits a clause ID it does not enforce passes
        # condition 2. That is the hollow-backing shape relocated into the ratchet's trust chain;
        # the guard (the declaring tool must carry discrimination controls, which CI already gates
        # via test_*.py) belongs with #187's backing-forms work, not baked in here.
        undeclared = None if live_coverage is None else sorted(arrived_lint - live_coverage)
        # Condition 3 (no arrival was harness-backed at base). Two gates, both fail-closed:
        #
        # 3a — cheap FIRST FILTER (base FLOOR harness). The `harness_lost` check ABOVE catches a
        # harness->lint downgrade ONLY while the floor still reflects the drop (`harness_delta<0`).
        # Once the floor is BUMPED to absorb it — which a "recorded" PR does by definition —
        # `harness_delta` reads 0 and the downgrade goes SILENT. A genuine arrival never retires a
        # harness backing, so the floor's harness must not have dropped (`live.harness >=
        # prev_floor_harness`). But a downgrade OFFSET by a promotion holds the COUNT flat and
        # slips this — hence 3b.
        #
        # 3b — the DECIDING check (base LEDGER + base SPEC sets, #267 review ruling). The ledger's
        # domain IS the unbacked set, so `id in base ledger` == `id NOT harness-backed at base`.
        # For each arrival: NEW (absent from the base spec) -> legit; in the base ledger (untested/
        # lint at base) -> legit upgrade; present in the base spec but ABSENT from the base ledger
        # -> it was HARNESS-backed -> a downgrade, not a birth. This closes the count-flat residual
        # 3a cannot, at the cost of one already-parsed ledger string plus one `git show` per stem.
        #
        # Any base read unavailable (None) -> the gate is uncertifiable -> decline the green and
        # fall to `lint_drift` (RED), never pass an unchecked condition (#213/#225).
        harness_not_dropped = (prev_floor_harness is not None
                               and live["harness"] >= prev_floor_harness)
        if base_ledger_all is None or base_spec_ids is None:
            downgraded = None
        else:
            downgraded = sorted(x for x in arrived_lint
                                if x in base_spec_ids and x not in base_ledger_all)
        if not not_in_ledger and undeclared == [] and harness_not_dropped and downgraded == []:
            return ("arrival_recorded", [
                f"at the floor - harness {live['harness']}, lint {live['lint']}, untested "
                f"{live['untested']}.",
                f"{len(arrived_lint)} clause(s) ARRIVED lint-backed, born with their detector: "
                f"{', '.join(sorted(arrived_lint))}. Each is recorded `lint:<tool>` in the ledger "
                "and declared by that tool's --emit-coverage; none was harness-backed at base "
                "(new to the spec, or already unbacked) and the floor's harness did not drop. "
                "Green — a new normative clause and its enforcement landed together (#267)."])
        # else: a condition failed — fall through to `lint_drift`, which tells the human to verify
        # each arrival and update the ledger deliberately (the un-bumped / un-recorded / downgrade
        # case). lint_drift is RED, so every uncertified case stays human-gated.

    if arrived_lint:
        return ("lint_drift", [
            f"{len(arrived_lint)} clause(s) newly appear in the lint set: "
            f"{', '.join(sorted(arrived_lint))}.",
            "Verify each was previously UNTESTED (a new documentary enforcement, fine) and NOT "
            "previously HARNESS (a behavioral->documentary downgrade, an evidence loss), then "
            "update the ledger deliberately."])

    # The lint COUNT reaches here only when no recorded move (substitution / arrival /
    # de-crediting) has accounted for it — each of those returns above — so a deviation at this
    # point is unexplained. NAMES BOTH NUMBERS (#320 review): a message saying only "lint
    # deviates" sends the reader to the file to work out which of the two is wrong, and that
    # invisibility was half of why this survived — the at-floor line printed the live number.
    if lint_delta < 0:
        return ("regression", [
            f"lint fell {floor['lint']} -> {live['lint']}: documentary enforcement was lost, and "
            "no recorded move (substitution / arrival / de-crediting) accounts for it."])

    if harness_delta > 0 or untested_delta < 0 or lint_delta > 0:
        b = ([f"harness {floor['harness']} -> {live['harness']}"] if harness_delta > 0 else []) \
            + ([f"lint {floor['lint']} -> {live['lint']}"] if lint_delta > 0 else []) \
            + ([f"untested {floor['untested']} -> {live['untested']}"] if untested_delta < 0 else [])
        return ("stale", [f"coverage improved past the floor: {', '.join(b)}.",
                          f"Set the floor to harness {live['harness']}, lint {live['lint']}, "
                          f"untested {live['untested']}. Green means AT the floor, never under it."])

    return ("at_floor", [f"at the floor - harness {live['harness']}, lint {live['lint']}, "
                         f"untested {live['untested']}; lint set matches the base."])


def classify_proven(floor_proven, live_proven_map, live_harness, base_proven_map=None):
    """The 4th ratchet dimension (#278). Classify the PROVEN count against its floor. Returns
    (verdict, lines); verdict is one of:
      proven_at_floor      — live == floor. GREEN.
      proven_improved      — live > floor: proofs were earned. Bump the floor up (action needed,
                             like `stale`), so green means AT the floor, not under it.
      proven_retired       — live < floor and EVERY dropped proof's proving test is GONE from the
                             harness. A legitimate retirement: the testimony was about that test,
                             the test no longer exists, so the proof is correctly void. Bump the
                             floor DOWN in the same PR (action needed, not a regression).
      proven_regression    — live < floor and some dropped proof's proving test SURVIVES. The
                             marker was removed while the test stands: a stale proof, or a silent
                             loss. RED, always — the architect's #278 2b ruling.
      proven_uncharacterized — live < floor but base_proven_map is None, so which proofs dropped,
                             and whether their tests survive, cannot be determined. Fail-closed:
                             an undeterminable drop is refused, never passed (#213 shape).

    The drop gate asks about the proving TEST, not whether any COUNT moved: deleting a redundant
    test while the clause stays backed elsewhere moves no harness count yet voids the testimony,
    so a count-based gate would false-green it (the false positive the architect caught in the
    piggyback proposal). `live_harness` is the set of test fn names that exist now.

    NOTE (convention 16(d)) — DEFERRED green path: a proof that expires while its test SURVIVES
    (e.g. an impl refactor makes the old mutation inapplicable, and a new proof is not in yet)
    reds here, and the author must re-prove. A deliberate `Proof-retired` record is the intended
    remedy and is deferred until a real instance appears — building it for a population of zero is
    speculation (#278). Likewise the live caller does not yet READ `base_proven_map` from the base
    tree: with PROVEN starting at 0 the drop gate is unexercised live, so the caller passes None
    and this function fail-closes any live drop; the reader lands with the first real proof
    retirement. classify_proven's drop LOGIC is exercised now by the fixture in
    test_kiss_proven_ratchet.py, which supplies base_proven_map directly (the tier reports 0->0
    forever, so only a fixture can prove the arm — the #279 lesson, same shape as the tier)."""
    live_count = len(live_proven_map)
    if live_count == floor_proven:
        return ("proven_at_floor", [f"proven at the floor - {live_count}."])
    if live_count > floor_proven:
        return ("proven_improved", [
            f"proven rose {floor_proven} -> {live_count}: {live_count - floor_proven} proof(s) "
            "earned. Bump the proven floor in this PR — green means AT the floor, never under it."])
    # live_count < floor_proven — a proof dropped.
    if base_proven_map is None:
        return ("proven_uncharacterized", [
            f"proven fell {floor_proven} -> {live_count} but the base proven set was not read, so "
            "which proof dropped and whether its test survives cannot be told. Fail-closed: an "
            "undeterminable proof drop is refused, not passed."])
    dropped = {c: ts for c, ts in base_proven_map.items() if c not in live_proven_map}
    stale = {c: [t for t in ts if t in live_harness] for c, ts in dropped.items()}
    stale = {c: ts for c, ts in stale.items() if ts}
    if stale:
        detail = "; ".join(f"{c} (proving test {', '.join(ts)} still present)"
                           for c, ts in sorted(stale.items()))
        return ("proven_regression", [
            f"proven fell {floor_proven} -> {live_count}, and {len(stale)} proof(s) lost their "
            f"marker while the proving test SURVIVES: {detail}.",
            "A stale proof claims aboutness it has lost — restore the marker or re-prove. A proof "
            "drop is green ONLY when its proving test no longer exists (#278)."])
    gone_tests = sorted({t for ts in dropped.values() for t in ts})
    detail = "; ".join(f"{c} (was proven by {', '.join(dropped[c])})" for c in sorted(dropped))
    return ("proven_retired", [
        f"proven fell {floor_proven} -> {live_count}: {len(dropped)} clause(s) lost their proof "
        f"because the proving test(s) were removed from the harness — {detail}.",
        f"The removed test(s): {', '.join(gone_tests)}. The testimony was about those tests and "
        f"they are gone, so the retirement is legitimate. Bump the proven floor DOWN to "
        f"{live_count} in this PR."])


def write_ledger(path, unbacked, prior=None):
    """Write the ledger, PRESERVING the category/note of any clause still unbacked
    (so --update-ledger never silently drops a curated categorization). A clause
    unbacked for the first time is written as bare `untested`."""
    prior = prior or {}
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(LEDGER_HEADER)
        for cid in sorted(unbacked):
            test = unbacked[cid]
            p = prior.get(cid)
            if p and p["category"] != "untested":
                cat = f"{p['category']}:{p['lint']}" if p["category"] == "lint" and p["lint"] else p["category"]
                fh.write(f"{cid}\t{test}\t{cat}\t{p['note']}\n")
            else:
                # An `untested` clause can still carry a CURATED note — why it is untested,
                # and what would close it. Preserve it, or --update-ledger silently drops the
                # ledger's institutional memory on a routine run (#272): the docstring promises
                # it "never silently drops a curated categorization", but the old empty-note
                # `untested\t\n` broke that for exactly the untested category. A clause unbacked
                # for the FIRST time (no prior) still writes an empty note, as before.
                note = p["note"] if p else ""
                fh.write(f"{cid}\t{test}\tuntested\t{note}\n")


def discover_lint_coverage(tools_dir):
    """The clauses each document lint declares it enforces: {clause_id: (tool, note)}.

    Runs every sibling `kiss_*.py --emit-coverage` and parses `clause_id<TAB>note`
    lines. A lint enforces a clause iff a violation of that clause fails the lint;
    the lint ASSERTS this by emitting the clause ID, and — because the tool is
    actually run — the label cannot outrun the enforcement. A lint that lacks the
    flag (exits non-zero, prints nothing) simply contributes no coverage.
    """
    cov = {}
    if not os.path.isdir(tools_dir):
        return cov
    self_name = os.path.basename(os.path.abspath(__file__))
    for fn in sorted(os.listdir(tools_dir)):
        if not (fn.startswith("kiss_") and fn.endswith(".py")) or fn == self_name:
            continue
        tool = fn[:-3]
        try:
            out = subprocess.run(
                [sys.executable, os.path.join(tools_dir, fn), "--emit-coverage"],
                capture_output=True, text=True, timeout=120)
        except Exception:
            continue
        if out.returncode != 0:
            continue
        for line in out.stdout.splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            cid = parts[0].strip()
            if RE_IDPART.match(cid):
                cov[cid] = (tool, parts[1].strip() if len(parts) > 1 else "")
    return cov


def sub_of(clause_id):
    m = RE_IDPART.match(clause_id)
    return m.group(1) if m else "?"


class DocResult:
    def __init__(self, stem):
        self.stem = stem
        self.prefix = "KISS-" + stem.upper()
        self.body = []       # (clause_id, line, [tests])
        self.terms = {}      # clause_id -> distinctive terms (#246)
        self.matrix = []     # (clause_id, test, line)
        self.violations = []

    def add(self, msg):
        self.violations.append(msg)

    @property
    def clause_ids(self):
        return {c for c, _, _ in self.body} | {c for c, _, _ in self.matrix}


# ABOUTNESS SIGNAL (#246). Binding is by NAME in both directions and reads no assertion, so
# a clause can name a test that exists, compiles and passes while checking something else --
# and be counted as backed. This does not close that; it MEASURES it, and never gates.
#
# The signal is deliberately the weakest defensible one: does the named test's text mention
# any DISTINCTIVE TERM the clause names? Backticked identifiers are the spec's own way of
# naming concrete artifacts (`bf16`, `structure_key`, `f8e4m3fn`), so they discriminate far
# better than prose words.
#
# THE CLAUSE'S OWN `*Test:*` NAME IS EXCLUDED, and everything depends on it: that name is
# backticked in the clause and is by construction present in the test, so leaving it in makes
# EVERY clause overlap and the measure reports nothing while looking like it works. A signal
# that cannot come out negative is not a signal.
RE_TICK = re.compile(r"`([^`\n]{2,80})`")
# words too common in this corpus to discriminate; a term that appears in most tests
# separates nothing, and including them would inflate the overlap count silently.
ABOUT_STOP = {
    "test", "tests", "the", "and", "not", "for", "with", "must", "may", "shall", "kiss",
    "true", "false", "none", "some", "self", "fn", "let", "use", "mod", "pub", "std",
    "assert", "assert_eq", "vec", "str", "string", "usize", "u8", "u16", "u32", "u64",
    "i8", "i16", "i32", "i64", "json", "rs", "md", "tsv", "toml",
}


def clause_terms(block):
    """Distinctive backticked identifiers in a clause block, MINUS the test(s) it names."""
    named = {t.lower() for t in RE_TEST.findall(block)}
    out = set()
    for m in RE_TICK.finditer(block):
        span = m.group(1)
        # A backticked CLAUSE ID is skipped WHOLE, not filtered afterwards: it tokenizes to
        # `kiss` and the sub-standard (`ops`, `classify`), and a post-filter on the assembled
        # id never sees those pieces. Caught by its own control, which is why the control
        # asserts the empty set rather than "does not contain the id".
        if RE_CLAUSE_ID.search(span):
            continue
        # RE_WORD, the SAME constant the test side tokenizes with (used at collection).
        # The two patterns were byte-identical, so this changes no result today -- the
        # 15 is unaffected and was not re-derived on the strength of this edit. What it
        # changes is the FAILURE MODE: nothing fails if a future edit alters one pattern
        # and not the other, and the intersection would keep returning a number, just a
        # wrong one -- and that number IS the deliverable. One constant makes the
        # agreement structural instead of coincidental (#367 review).
        for tok in RE_WORD.findall(span):
            t = tok.lower()
            if len(t) >= 3 and t not in ABOUT_STOP and t not in named:
                out.add(t)
    return out


def _mentions(term, tokens):
    """Does `term` occur in any token at an UNDERSCORE BOUNDARY or a string edge?

    Exact-token equality was the first rule and it is wrong in one direction (#192): a
    reverse-cited test is often named for the thing it tests -- `scatter_oob_writes_are_skipped`,
    `e2_index_select_u32` -- so the clause's term is PRESENT but embedded, and the report
    said "mentions NONE", which is FALSE.

    Boundary-anchored rather than a bare substring, because a bare one over-matches in the
    direction that shrinks the alarming bucket: `sort` inside `assert_sorted` is not a
    mention of `sort`. Underscore is the component separator in every identifier here, so
    the boundary IS the word boundary.

    THE ASYMMETRY IS WHY THIS SURVIVED #367: §9-named tests are conventionally
    `test_<sub>_<thing>` and never embed a clause identifier as a component, so exact
    matching is correct on the NAMED tier and wrong on the CITED one. An instrument's
    defects are visible only where its inputs differ from the ones it was developed against.
    """
    for tok in tokens:
        i = tok.find(term)
        while i != -1:
            if (i == 0 or tok[i - 1] == "_") and (
                    i + len(term) == len(tok) or tok[i + len(term)] == "_"):
                return True
            i = tok.find(term, i + 1)
    return False


def citing_tokens(cid, harness):
    """Tokens of every test CARRYING A BACKING CITATION for `cid`, or None if there is none.

    THE §9 ROW IS THE WRONG TEST HERE, and using it is the failure this function exists to
    prevent. A CITED clause is backed by whichever test cites it, which is frequently not the
    test the §9 matrix names -- reverse-only clauses have no §9-named test at all.

    Getting it wrong fails in the REASSURING direction: a §9-named test usually DOES mention
    its clause's terms, so comparing against it would report near-universal overlap and the
    miss bucket would quietly approach zero.
    """
    toks, found = set(), False
    for info in harness.values():
        if cid in info["clauses"]:
            toks |= info["tokens"]
            found = True
    return toks if found else None


def about_bucket(terms, tokens):
    """(bucket, matched) for one clause: 'hit' | 'miss' | 'unmeasurable'.

    UNMEASURABLE is a clause naming no distinctive term at all -- it is not clean, and it is
    not evidence of absence either.

    NO PROSE/IDENTIFIER CLASSIFIER, deliberately, and the absence is a measured result rather
    than an omission. A miss whose terms are `{each, yield}` is weakly informative, and it is
    tempting to reclassify it -- but two candidate groundings were measured and BOTH fail:
    corpus membership (the token set includes comment prose, so `normalizes` and `each` are
    both "in corpus") and spec-side document frequency (prose words are as RARE as real
    identifiers -- `exceed` 1, `s32` 1 -- while `structure_key` at 87 is the most common term
    in the suite). Pure-alpha fails too: `relu`, `argmax`, `sort`, `bool` are all identifiers.

    A row can be CORRECTLY COMPUTED AND UNINFORMATIVE. Reclassifying it would be adjusting a
    number to match an intuition about what it ought to say. The report prints each miss row's
    TERMS instead, so the reader who can judge it does the triage a classifier would do badly.
    """
    if not terms:
        return "unmeasurable", set()
    matched = {t for t in terms if _mentions(t, tokens)}
    return ("hit" if matched else "miss"), matched


def parse(path, res):
    text = open(path, encoding="utf-8").read()

    def lineno(off):
        return text.count("\n", 0, off) + 1

    defs = [(m.group(1), m.start()) for m in RE_DEF.finditer(text)]
    heads = [m.start() for m in RE_HEAD.finditer(text)]
    # a clause block runs from its definition to the next definition-or-heading.
    boundaries = sorted([p for _, p in defs] + heads + [len(text)])
    for cid, pos in defs:
        end = next(b for b in boundaries if b > pos)
        block = text[pos:end]
        res.body.append((cid, lineno(pos), RE_TEST.findall(block)))
        res.terms[cid] = clause_terms(block)
    for m in RE_MATRIX.finditer(text):
        res.matrix.append((m.group(1), m.group(2), lineno(m.start())))


def dup_scan(pairs, label, res):
    seen = defaultdict(list)
    for cid, ln in pairs:
        seen[cid].append(ln)
    for cid, lns in seen.items():
        if len(lns) > 1:
            res.add(f"duplicate {label}: {cid} at lines {', '.join(map(str, lns))}")


# --- RFC-allocated clause ids are TAKEN (#342) -----------------------------------------
#
# `dup_scan` above catches an id defined twice in one document. Clause ids are ALSO
# allocated in `rfcs/`, and nothing reconciled the two — so an id drafted in an in-tree RFC
# and not yet landed was invisible to the very gate that exists to prevent reuse.
#
#   AN ID REGISTER THAT DOES NOT READ THE REGISTER OF PENDING ALLOCATIONS IS A REGISTER OF
#   HALF THE ALLOCATIONS.
#
# Caught live on #332: `spec/conform.md` defines §6.5-0001..-0010, an RFC allocates a
# contiguous block of six (0011..0016), and a PR minted `KISS-CONFORM-6.5-0011` for a
# DIFFERENT obligation. Every existing gate passed — the id is absent from `spec/`, so
# append-only is satisfied, and the new clause has a test, so the trace gate is satisfied.
#
# THE CONSEQUENCE IS WORSE THAN ORDINARY REUSE. `§6.5-0011` is cited in prose by two shipped
# tools with the RFC's meaning. Today it resolves to nothing and that is VISIBLE. After a
# colliding merge it resolves to the WRONG clause, those citations become silently wrong,
# and the dangling-citation check GOES QUIET because the id now exists — converting a
# detectable defect into an undetectable one and silencing the gate that was catching it.
RFC_DIRNAME = "rfcs"
# An ALLOCATION is a drafted clause BLOCK, the same `- **ID** — ...` form the spec uses.
# A bare mention of an id elsewhere in an RFC is a CITATION of a landed clause and reserves
# NOTHING: measured, 32 ids are mentioned across `rfcs/` and only 10 are allocated. Keying
# on mentions would reserve every id any RFC discusses.
# Allocation uses the CANONICAL definition parser, not a parallel copy (#344 review).
# The first draft wrote its own `- **ID**` regex with `\d+` where CLAUSE_ID requires
# `\d{4}` -- so the gate's notion of ALLOCATED and the spec's notion of DEFINED were
# different sets, and a collision between them would have been invisible in exactly the
# direction this gate exists to catch. Worse, the control extracted with the private
# copy: it would have passed while the gate was broken, because it was not exercising
# the code that ships. Same object as a test that passes against a stub.
RE_RFC_ALLOC = RE_DEF
# A drafted id is released DELIBERATELY, never by absence — the same rule as an exclusion
# list asserted empty rather than deleted. If an RFC's allocation lands or is abandoned,
# that is an edit someone makes on purpose, not a silence anyone can produce by deleting.
# Colon INSIDE or OUTSIDE the bold. The first draft accepted only the outside form while
# the tool's own error message told authors to write the inside form -- the instruction
# and the parser disagreeing, which is the defect this suite keeps finding one level up.
# LINE-ANCHORED and MULTILINE (#344 review). Un-anchored, prose containing the marker
# releases an allocation -- a quoted example, a changelog line, or a paragraph explaining
# the gate. A GATE THAT CAN BE DISABLED BY DOCUMENTATION ABOUT THE GATE, in a repo that
# writes a great deal of prose about its own gates.
RE_RFC_RELEASE = re.compile(r"^\s*\*\*ALLOCATION (?:LANDED|RELEASED):?\*\*:?(.*)$", re.I | re.M)


def rfc_allocations(rfc_dir):
    """({id: [(file, line)]}, released_ids) from every `rfcs/**.md`."""
    allocs, released = defaultdict(list), set()
    if not os.path.isdir(rfc_dir):
        return allocs, released
    for dirpath, _dirs, files in os.walk(rfc_dir):
        for fn in sorted(files):
            if not fn.endswith(".md"):
                continue
            path = os.path.join(dirpath, fn)
            with open(path, encoding="utf-8") as fh:
                text = fh.read()
            rel = os.path.relpath(path, os.path.dirname(rfc_dir)).replace(os.sep, "/")
            for m in RE_RFC_ALLOC.finditer(text):
                allocs[m.group(1)].append((rel, text[:m.start()].count("\n") + 1))
            for m in RE_RFC_RELEASE.finditer(text):
                released |= set(re.findall(CLAUSE_ID, m.group(1)))
    return allocs, released


def check_rfc_collisions(spec_ids, rfc_dir):
    """Ids DEFINED in spec/ that an RFC still holds ALLOCATED. Returns [message]."""
    allocs, released = rfc_allocations(rfc_dir)
    out = []
    for cid in sorted(set(spec_ids) & set(allocs)):
        if cid in released:
            continue  # released on purpose, in an edit someone made
        where = "; ".join(f"{f}:{l}" for f, l in allocs[cid])
        out.append(
            f"{cid} is defined in spec/ AND still allocated in {where}. Either the RFC's "
            "clause is the one that landed — in which case mark it `**ALLOCATION LANDED:** "
            f"{cid}` in the RFC — or this is a COLLISION and the new clause needs the next "
            "free ordinal. A drafted id is released deliberately, never by absence.")
    return out


def check_doc(res):
    # 1. id format + prefix
    for cid, ln in [(c, l) for c, l, _ in res.body] + [(c, l) for c, _, l in res.matrix]:
        m = RE_IDPART.match(cid)
        if not m:
            res.add(f"malformed clause id: {cid} (line {ln})")
        elif "KISS-" + m.group(1) != res.prefix:
            res.add(f"foreign-prefix clause in {res.stem}.md: {cid} (line {ln})")

    # 2. duplicates
    dup_scan([(c, l) for c, l, _ in res.body], "body definition", res)
    dup_scan([(c, l) for c, _, l in res.matrix], "matrix row", res)

    body_ids = {c for c, _, _ in res.body}
    matrix_ids = {c for c, _, _ in res.matrix}

    if res.stem == "umbrella":
        for cid in sorted(body_ids | matrix_ids):
            res.add(f"umbrella is informative-only but defines a clause: {cid}")
        return

    # 3. body <-> matrix set equality (capped so one systemic slip doesn't flood)
    def report_set(ids, msg):
        ids = sorted(ids)
        for cid in ids[:8]:
            res.add(f"{msg}: {cid}")
        if len(ids) > 8:
            res.add(f"{msg}: ... and {len(ids) - 8} more")
    report_set(body_ids - matrix_ids, "clause defined in body but missing from §9 matrix")
    report_set(matrix_ids - body_ids, "clause in §9 matrix but no body definition")

    # 4. one test per body clause, and body test == matrix test
    matrix_test = {c: t for c, t, _ in res.matrix}
    for cid, ln, tests in res.body:
        uniq = sorted(set(tests))
        if len(uniq) == 0:
            res.add(f"clause has no *Test:* tag: {cid} (line {ln})")
        elif len(uniq) > 1:
            res.add(f"clause has multiple *Test:* tags {uniq}: {cid} (line {ln})")
        elif cid in matrix_test and uniq[0] != matrix_test[cid]:
            res.add(f"test disagreement for {cid}: body `{uniq[0]}` vs matrix `{matrix_test[cid]}`")

    # 5. REMOVED (#247): the `test_`-prefix convention. Its only real service was "the matrix
    #    names something that is actually a test", and the forward-EXISTENCE check (binding,
    #    check 2) delivers that STRICTLY BETTER — existence also catches a bogus name that
    #    happens to start with `test_`, which a prefix match never could. This is a subsumption,
    #    not a relaxation: proven born-red, not argued, by
    #    `test_conform_existence_subsumes_the_prefix_check` (trace_gate.rs), which reddens on a
    #    §9 entry naming a real non-#[test] symbol with the name test_-prefixed so the prefix
    #    check is neutral. The convention forced a matrix `*Test:*` to be a `test_*` placeholder
    #    even when the real backing test is descriptively named (`reject_bad_magic`,
    #    `gather_oob_policies`), which kept the matrix UNTRUE — the very drift #247 fixes. The
    #    tool's own authority model already says so: `discover_tests`' docstring — "the citation,
    #    not the name, is what ties a requirement to executable code."


def compute_masked_forward(clause_test, harness, cited):
    """#286 detector. Return {cid: (named_test, [backing_test, ...])} for every clause
    BACKED only by reverse citation whose §9 `*Test:*` names a test that resolves to nothing.

    A clause qualifies iff its named test is ABSENT from `harness` (so it is not
    forward-backed) AND some harness test cites it (so it IS backed, via reverse). Its §9
    row is then a false artifact that the `no such test` report cannot see — that report
    fires only for otherwise-UNBACKED clauses, so the reverse backing MASKS the dead row.

    The backing list is the FULL sorted set of citing tests, not the first — a masked clause
    may have several reverse citers, and the report's recommended fix (point the §9 row at
    the real test) must be chosen from ALL of them, not one arbitrarily-picked name that the
    message then calls the sole backer (the Copilot #291 finding: do not hide the choice the
    fix depends on).

    A clause whose named test is absent AND uncited is UNBACKED, not masked: its
    aspirational name is correct by design (the 553 that must never be flagged), so it is
    deliberately excluded here. This function is the instrument the born-red control in
    test_kiss_masked_forward.py drives — main() must call it, not inline the logic, or the
    control would prove a copy rather than the detector (#279)."""
    masked = {}
    for c, t in clause_test.items():
        if t not in harness and c in cited:
            masked[c] = (t, sorted(cited[c]))
    return masked


def compute_evidence_tiers(backed, cited, proven=None):
    """#278. Partition the BACKED clauses by evidence STRENGTH, the distinction convention
    15 draws — a citation's FORM settles deliberateness; only a MUTATION settles aboutness.
    Returns (named, cited_set, proven_set):

      named      — backed clauses with NO backing-form citation: the §9 name matches a real
                   fn but nothing asserts the tie. A name coincidence (convention 15's
                   "mention", one step up), and the population #278 exists to stop over-counting;
      cited_set  — backed clauses WITH a backing-form citation: a human deliberately tied test
                   to clause. Deliberateness;
      proven_set — cited clauses carrying a recorded mutation-proof: aboutness.

    INVARIANTS, asserted by the born-red control in test_kiss_evidence_tiers.py:
      * named and cited_set PARTITION backed (disjoint, union == backed);
      * proven_set ⊆ cited_set — a proof presupposes the citation; recording a proof for a
        NAMED-only clause means first MIGRATING it to cited (an evidence-adding move, never
        the bare-name→citation mass sweep that would inflate deliberateness without proof).

    `proven` is the set of clauses with a recorded proof; it defaults to empty because no
    machine-readable proof record exists yet. Ruled #278: the record is a `// Proven:` marker
    at the test (carrying the mutation SUBJECT and a resolvable REF — the marker is TESTIMONY
    that a demonstration exists at a ref, not a live re-run), enforced by a 4th blocking
    ratchet dimension from 0. Step 2 supplies `proven` from those markers; this function and
    the report only COUNT. Any element of `proven` not in cited_set is dropped, so the
    subset invariant holds by construction rather than by the caller's discipline."""
    proven = proven or set()
    cited_set = {c for c in backed if c in cited}
    named = {c for c in backed if c not in cited}
    proven_set = {c for c in proven if c in cited_set}
    return named, cited_set, proven_set


def classify_shared_test(t, clauses, declared_shares):
    """#286. For a test named by MORE THAN ONE clause across the §9 matrices, decide
    whether the share is sanctioned. Returns one of:

      'cross_standard'  — a Conform-owned `test_conform_*` test shared by EXACTLY TWO
                          clauses: its one CONFORM owner and one deferring sub-standard
                          clause (the original sanctioned exception). A third citer — one
                          CONFORM owner and TWO deferrers — is NOT covered: it must be
                          declared deliberately, not pass on a loose CONFORM count;
      'declared_share'  — an explicit same-standard entry in `declared_shares` whose clause
                          set matches the citing set EXACTLY (a superset is NOT covered);
      'violation'       — anything else: an undeclared collision that must reden.

    Called only when len(clauses) > 1. Injectivity is the default; both non-violation
    verdicts are opt-in and must be declared, so a new collision cannot pass by omission —
    which is why the cross-standard arm pins the count at two rather than admitting any
    number of deferrers behind a single CONFORM clause (the Copilot #291 finding: the code
    was one CONFORM among the citers, the prose was exactly one deferrer; they coincide only
    at width two)."""
    subs = [sub_of(c) for c in clauses]
    if t.startswith("test_conform_") and len(clauses) == 2 and subs.count("CONFORM") == 1:
        return "cross_standard"
    if t in declared_shares and set(clauses) == declared_shares[t]["clauses"]:
        return "declared_share"
    return "violation"


def main():
    ap = argparse.ArgumentParser(description="KISS-Conform traceability checker")
    ap.add_argument("--spec-dir", default=None, help="path to the spec/ directory")
    ap.add_argument("--conformance-dir", default=None,
                    help="path to the conformance/ harness")
    ap.add_argument("--ratchet", action="store_true",
                    help="gate on the committed floor in conformance/COVERAGE_FLOOR.tsv: "
                         "harness and lint MUST NOT fall, untested MUST NOT rise, and a "
                         "figure BETTER than the floor fails as a stale floor. A second "
                         "mode, not a replacement for --strict. THREE exit states: 0 clean, "
                         "1 the floor moved, 2 INCONCLUSIVE (the counts were compared but the "
                         "lint SET could not be, because --base-ref was absent). 2 is not a "
                         "pass and not a violation -- re-run with --base-ref to get either.")
    ap.add_argument("--strict", action="store_true",
                    help="fail on ANY unbacked clause, ignoring the ledger (§6.2 verbatim)")
    ap.add_argument("--assert-known-red", action="store_true",
                    help="#343: implies --strict. Exit 0 ONLY if the failure is exactly the "
                         "tolerated one (untested MUSTs). A different failure, or NO failure, "
                         "exits 1 -- a tolerated red that stops being the tolerated red is news "
                         "in both directions.")
    ap.add_argument("--why-red", action="store_true",
                    help="#343: print the set of failure reasons and exit 0. Diagnostic.")
    ap.add_argument("--freeze-ready", nargs="?", const="ALL", default=None,
                    metavar="SUB",
                    help="umbrella §5.3 condition 3: fail unless every clause of SUB "
                         "(or of all nine) is backed by an executable test. This is "
                         "the gate a Draft->Frozen transition must pass, and the same "
                         "predicate a §8.1 conformance claim to SUB depends on.")
    ap.add_argument("--update-ledger", action="store_true",
                    help="rewrite the unbacked ledger to the current truth")
    ap.add_argument("--report", action="store_true",
                    help="print the per-sub-standard executable-coverage breakdown")
    ap.add_argument("--base-ref", default=None, metavar="REF",
                    help="git ref of the PRE-change state (e.g. origin/main; in CI, "
                         "origin/$base_ref, since HEAD is the PR merge commit). REQUIRED by "
                         "--ratchet whenever the lint dimension moves: the substitution check "
                         "reads UNBACKED.tsv at this ref to tell a lint->harness upgrade from a "
                         "laundered regression, and a defaulted ref would silently compare "
                         "against a stale tree. Not consulted when the lint count is unchanged.")
    args = ap.parse_args()
    if args.assert_known_red:
        args.strict = True   # the tolerated red only exists in strict mode


    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(here)
    spec_dir = args.spec_dir or os.path.join(root, "spec")
    conf_dir = args.conformance_dir or os.path.join(root, "conformance")
    ledger_path = os.path.join(conf_dir, "UNBACKED.tsv")

    results = []
    for stem in SPECS:
        path = os.path.join(spec_dir, stem + ".md")
        res = DocResult(stem)
        if not os.path.exists(path):
            res.add(f"missing spec file: {path}")
        else:
            parse(path, res)
            check_doc(res)
        results.append(res)

    # #342: reconcile spec definitions against RFC allocations. Suite-wide because an
    # allocation in one RFC can collide with a definition in any document.
    _spec_ids = {c for r in results for c, _, _ in r.body}
    _rfc_msgs = check_rfc_collisions(_spec_ids, os.path.join(root, RFC_DIRNAME))
    for _m in _rfc_msgs:
        results[0].add(_m)

    # suite-wide: every conformance test maps to exactly one clause, except a
    # Conform-owned cross-standard test cited by a deferring sub-standard clause.
    test_to_clauses = defaultdict(set)
    clause_to_doc = defaultdict(set)
    for res in results:
        for cid, t, _ in res.matrix:
            test_to_clauses[t].add(cid)
        for cid in res.clause_ids:
            clause_to_doc[cid].add(res.stem)
    suite_violations, deferrals = [], []
    for t, clauses in sorted(test_to_clauses.items()):
        if len(clauses) > 1:
            kind = classify_shared_test(t, clauses, DECLARED_SHARES)
            if kind == "cross_standard":
                deferrals.append(f"cross-standard deferral: conform test `{t}` cited by {', '.join(sorted(clauses))}")
            elif kind == "declared_share":
                deferrals.append(f"declared same-standard share: `{t}` <- {', '.join(sorted(clauses))} "
                                 f"({DECLARED_SHARES[t]['reason']})")
            else:
                suite_violations.append(f"test `{t}` maps to {len(clauses)} clauses: {', '.join(sorted(clauses))}")
    for cid, docs in sorted(clause_to_doc.items()):
        if len(docs) > 1:
            suite_violations.append(f"clause id {cid} appears in multiple docs: {', '.join(sorted(docs))}")

    # ---- check 2: BINDING — does the named test actually exist? ----
    harness = discover_tests(conf_dir)
    ledger = read_ledger(ledger_path)
    lint_cov = discover_lint_coverage(here)

    # every (clause -> named test) pair across the suite, from the §9 matrices
    clause_test = {}
    for res in results:
        for cid, t, _ in res.matrix:
            clause_test[cid] = t
    all_terms = {}
    for res in results:
        all_terms.update(res.terms)

    # A clause is BACKED if executable code is tied to it, by either direction of
    # §6.1's bidirectional traceability:
    #   forward — the test the clause names exists in the harness; or
    #   reverse — some executable test cites this clause as one it enforces.
    # The reverse direction is what gives credit for a real test whose fn name has
    # drifted from the spec's `*Test:*` name.
    cited = defaultdict(set)          # clause_id -> {test_name, ...}
    for tname, info in harness.items():
        for cid in info["clauses"]:
            cited[cid].add(tname)

    backed, unbacked, by_name, by_citation = {}, {}, {}, {}
    for c, t in clause_test.items():
        if t in harness:
            backed[c] = t
            by_name[c] = t
        elif c in cited:
            backed[c] = sorted(cited[c])[0]
            by_citation[c] = backed[c]
        else:
            unbacked[c] = t
    # MASKED FORWARD NAME (#286): a clause BACKED via reverse citation whose §9 `*Test:*`
    # names a test that RESOLVES TO NOTHING. Every `by_citation` clause is exactly this — it
    # is here BECAUSE its named test is not in the harness (else it would be `by_name`), so its
    # §9 row is a false artifact. The `no such test` report fires only for UNBACKED clauses
    # (`new_unbacked`), so reverse backing MASKS the dead row — it is invisible to every other
    # check. For an UNBACKED clause the aspirational name is correct by design and is NOT a
    # defect (that is the 553 that must never be flagged); a BACKED clause with a dead row is.
    # Fix: point the §9 row at the real backing test, or DECLARE a same-standard share
    # (`DECLARED_SHARES`). Once the rows are true this set is EMPTY and the detector never fires
    # in normal operation — so its only evidence of function is the born-red control in
    # test_kiss_masked_forward.py (a detector that cannot be seen to fire is indistinguishable
    # from a broken one, the #279 finding).
    masked_forward = compute_masked_forward(clause_test, harness, cited)
    # EVIDENCE-STRENGTH TIERS (#278). The single `backed` count spans three strengths of
    # evidence, and convention 15 names the lines between them — a citation's FORM settles
    # DELIBERATENESS; only a MUTATION settles ABOUTNESS:
    #   NAMED  — the §9 name matches a real fn, but NO backing-form citation exists for the
    #            clause. The credit rests on a name coincidence — convention 15's "mention",
    #            one step up. This is the population #278 exists to stop over-counting.
    #   CITED  — a backing-form citation (`// Backs:`/`Enforces`/assert first-arg) asserts the
    #            link: a human deliberately tied this test to this clause. Deliberateness.
    #   PROVEN — a seeded mutation of the obligation's SUBJECT (impl for an impl clause, spec
    #            text for a document clause) reddens the test. Aboutness. PROVEN ⊆ CITED:
    #            recording a proof for a NAMED-only clause entails first migrating it to CITED
    #            (adding the citation) — an evidence-ADDING move, not the bare-name→citation
    #            mass sweep the report exists to prevent (that adds deliberateness, not proof).
    # There is NO machine-readable proof record yet (the `Mutation:` doc-comment is prose,
    # rare, not clause-tied), so PROVEN reports 0 — which is the finding, not a gap in this
    # tool: the ONLY tier that is actual evidence has no number. Ruled #278: the record is a
    # `// Proven:` marker at the test (carrying the mutation SUBJECT per conv 15 and a resolvable
    # REF per conv 16(a) — TESTIMONY that a demonstration exists at a ref, NOT a live re-run),
    # enforced by a 4th blocking ratchet dimension from 0. Step 1 counted the tiers; step 2
    # (this) supplies `proven` from the markers. The GATE (the 4th ratchet dimension) is the
    # next increment; here PROVEN is reported and its markers are validated fail-closed.
    #
    # A clause is PROVEN only where the SAME test both BACKS it (deliberateness) and carries a
    # well-formed `// Proven:` (aboutness) — PROVEN ⊆ CITED. Two fail-closed violations, because
    # a proof tier is worth having only if nothing enters it unearned: a MALFORMED marker
    # (missing subject/ref), and a proof for a clause the proving test does not even back
    # (testimony with no backing for it to be about).
    proven_map, proven_violations = collect_proven(harness)
    proven_clauses = set(proven_map)
    tier_named, tier_cited, tier_proven = compute_evidence_tiers(backed, cited, proven=proven_clauses)
    tier_cited_by_name = {c for c in tier_cited if clause_test.get(c) in cited.get(c, ())}
    # Every executable test backing a clause, by either direction — not just the one
    # `backed` happened to pick. A clause is only honestly gate-free if at least one
    # of its backing tests runs unconditionally.
    def backing_tests(c):
        ts = {t for t in (clause_test.get(c),) if t in harness}
        return ts | {t for t in cited.get(c, ()) if t in harness}

    # GATE-ONLY: every test backing this clause is cfg-gated or declares a runtime
    # gate, so in a run where those gates are unsatisfied NOTHING verified it — yet
    # the matrix still counts it backed. This is the honest qualifier on the number.
    gated = {c: sorted(backing_tests(c))[0] for c in backed
             if backing_tests(c) and all(harness[t].get("gate") for t in backing_tests(c))}

    # A citation naming a clause that does not exist is a dangling reference —
    # §3.3 burns retired IDs, so this catches a test pinned to a dead clause. This is
    # REFERENCE hygiene, not coverage credit, so it scans EVERY citation (`cited_raw`, backings
    # AND mentions): a bare comment naming a retired id is a stale reference worth flagging even
    # though #187 gives it no backing credit. Narrowing this to backings (the #187 change to
    # `cited`) would leave a bare-comment reference to a burned id caught by NOTHING — the exact
    # blind spot #187's own scanner was built to close, reopened one layer down.
    all_clause_ids = set()
    for res in results:
        all_clause_ids |= res.clause_ids
    cited_all = defaultdict(set)
    for tname, info in harness.items():
        for cid in info["cited_raw"]:
            cited_all[cid].add(tname)
    dangling = {cid: sorted(ts) for cid, ts in cited_all.items() if cid not in all_clause_ids}

    # Executable tests that cite no clause at all: real work the matrix cannot see.
    orphans = sorted(t for t, i in harness.items()
                     if not i["clauses"] and t not in set(clause_test.values()))

    # ---- the CATEGORY of each unbacked clause (why it has no harness test) ----
    # Priority: a live lint that declares it (authoritative, self-verifying) wins;
    # else the curated ledger category; else the honest default `untested`.
    def eff_category(c):
        if c in lint_cov:
            return "lint", lint_cov[c][0]
        p = ledger.get(c)
        if p and p["category"] != "untested":
            return p["category"], p["note"]
        return "untested", ""

    cat_of = {c: eff_category(c) for c in unbacked}
    lint_backed = {c: lint_cov[c] for c in unbacked if c in lint_cov}
    by_category = defaultdict(list)
    for c, (base, _note) in cat_of.items():
        by_category[base].append(c)

    # A `lint:<tool>` category in the ledger whose clause no live lint actually
    # declares is an unverified label — the enforcement did not back the claim.
    lint_label_unbacked = sorted(
        c for c, p in ledger.items()
        if p["category"] == "lint" and c in unbacked and c not in lint_cov)
    # A category that makes a CLAIM must say why; a category that makes none need not.
    # `blocked`, `untestable`, `definitional` and `decredited` each assert a falsifiable
    # reason -- "the bytes are unpinned", "the clause contradicts itself", "the credit was
    # false" -- and a claim with no reason cannot be audited or retired. Bare `untested`
    # asserts nothing beyond "no test", so demanding 492 reasons for it would manufacture
    # exactly the triage the ledger would then appear to record (#290).
    #
    # `lint` is deliberately absent: its note is explanatory, but its ENFORCEMENT is
    # verified independently by the tool's own `--emit-coverage`, so the label cannot
    # outrun the check even with an empty note.
    #
    # `decredited` was added by #261 and never added here -- a new claim category arrived
    # without the check that makes claim categories auditable.
    CLAIM_CATEGORIES = ("blocked", "untestable", "definitional", "decredited")
    missing_note = sorted(
        c for c, p in ledger.items()
        if p["category"] in CLAIM_CATEGORIES
        and c in unbacked and not p["note"])

    if args.update_ledger:
        # Preserve curated categories; write lint-covered clauses with their lint
        # category so the file reflects the live enforcement.
        eff_prior = dict(ledger)
        for c in unbacked:
            if c in lint_cov:
                eff_prior[c] = {"category": "lint", "lint": lint_cov[c][0],
                                "note": lint_cov[c][1] or "enforced by the lint"}
        write_ledger(ledger_path, unbacked, prior=eff_prior)
        nb = untested_count(by_category)
        print(f"Wrote {ledger_path}: {len(unbacked)} unbacked "
              f"({len(lint_backed)} lint, {nb} untested).")
        return 0

    # accounted-for = harness-backed OR enforced by a live lint. The ratchet and
    # §6.2 treat a lint-enforced document clause as backed (the lint is its test).
    def accounted(c):
        return c in backed or c in lint_cov

    # a NEW clause with no harness test, not in the ledger, not lint-covered.
    new_unbacked = {c: t for c, t in unbacked.items()
                    if c not in ledger and c not in lint_cov}
    # stale = a ledger entry now HARNESS-backed (remove it), or one a live lint now
    # covers that is not yet labeled `lint` (refresh it). A clause correctly recorded
    # as `lint:` and still lint-covered is consistent, not stale.
    stale = {c: p["test"] for c, p in ledger.items()
             if c not in unbacked
             or (c in lint_cov and p["category"] != "lint")}

    # ---- report ----
    total_clauses = 0
    # #343: a tolerated failure and a NEW one must be different results.
    # `strict` is expected to fail for ONE reason -- the ledger is not empty -- and
    # `continue-on-error` makes the run conclusion SUCCESS regardless, so a crash, a parse
    # error, or a new violation class looks identical to the tolerated state at every
    # surface above the log. A PERMANENTLY-TOLERATED RED IS THE SAME OBJECT AS A DELETED
    # EXCLUSION LIST UNLESS SOMETHING PINS WHAT IT IS TOLERATING.
    #
    # So every failure now names itself. `--why-red` prints the set; `--assert-known-red`
    # asserts it is exactly the tolerated one.
    fail_reasons = []

    def note_fail(tag):
        fail_reasons.append(tag)
        return True

    any_fail = False
    # A REFUSAL is not a VIOLATION. --ratchet without --base-ref cannot characterize the
    # lint dimension, and declining to answer is correct -- but reporting that decline as
    # VIOLATIONS FOUND spends the one word that must keep meaning `the floor moved`. This
    # file's own rationale is that a check which is always red teaches everyone to ignore
    # it; a default invocation that reddens for a usage problem builds exactly that habit.
    # Tracked separately so a real violation always OUTRANKS a refusal (see the report at
    # the end): otherwise this state would become a way to mask one.
    inconclusive = False
    print("KISS-Conform traceability check  —  " + spec_dir)
    print("=" * 68)
    for res in results:
        n = len(res.clause_ids)
        total_clauses += n
        if res.violations:
            any_fail = note_fail("doc")
        note = "  (informative-only, as required)" if res.stem == "umbrella" and n == 0 and not res.violations else ""
        print(f"  [{'OK ' if not res.violations else 'FAIL'}] {res.stem:<9} {n:>4} clauses{note}")
        for v in res.violations:
            print(f"          - {v}")
    total_tests = len(test_to_clauses)
    print("-" * 68)
    if suite_violations:
        any_fail = note_fail("suite_wide")
        print("  SUITE-WIDE violations:")
        for v in suite_violations:
            print(f"          - {v}")
        print("-" * 68)
    if deferrals:
        print("  Deferrals (allowed shares):")
        for d in deferrals:
            print(f"          · {d}")
        print("-" * 68)
    if masked_forward:
        any_fail = note_fail("masked_forward")
        print(f"  MASKED FORWARD NAME: {len(masked_forward)} clause(s) are backed by REVERSE "
              f"citation but their §9 `*Test:*` names a test that does not exist — a false")
        print("  row nothing else can see (the `no such test` report is UNBACKED-only, #286):")
        for cid in sorted(masked_forward):
            dead, reals = masked_forward[cid]
            joined = ", ".join(f"`{r}`" for r in reals)
            backers = "backing test" if len(reals) == 1 else f"{len(reals)} backing tests"
            print(f"          - {cid} names `{dead}` (no such test); actual {backers}: {joined}")
        print("    Fix: point the §9 row at the real backing test, or DECLARE a same-standard")
        print("    share in kiss_trace.py DECLARED_SHARES with a reason.")
        print("-" * 68)
    if proven_violations:
        any_fail = note_fail("proven_marker")
        print(f"  PROVEN MARKER (#278): {len(proven_violations)} `// Proven:` marker(s) do not earn")
        print("  the PROVEN tier — a proof claim missing its subject/ref, or asserted over a clause")
        print("  the proving test does not back, is rejected fail-closed (never silently counted):")
        for v in proven_violations:
            print(f"          - {v}")
        print("-" * 68)
    print(f"  DOCUMENT CONSISTENCY: {total_clauses} normative clauses, "
          f"{total_tests} unique test names.")

    # ---- binding report ----
    print("-" * 68)
    n_map = len(clause_test)
    pct = (100.0 * len(backed) / n_map) if n_map else 0.0
    print(f"  BINDING (spec/ <-> conformance/):")
    print(f"      {len(harness)} executable test fns found in the harness.")
    print(f"      {len(backed)}/{n_map} clauses ({pct:.1f}%) are backed by executable code")
    print(f"          {len(by_name):>4} via the named test existing (forward)")
    print(f"          {len(by_citation):>4} via a test citing the clause (reverse)")
    # By EVIDENCE STRENGTH (convention 15) — the same `backed` count, split by how strong the
    # tie actually is. NAMED + CITED partition `backed`; PROVEN is a subset of CITED. See #278.
    # CITED is defined by its RELATION, not left to the reader: a clause is CITED iff SOME
    # harness test carries a backing-form citation for it (by ANY test, not only the §9-named
    # one) — the label must name the relation because "cited by the named test" (125) and
    # "cited by any test" (139) are two defensible numbers for one word.
    tier_cited_other = len(tier_cited) - len(tier_cited_by_name)
    # ABOUTNESS SIGNAL over the NAMED tier (#246) -- REPORTED, NEVER GATED.
    #
    # A NAMED clause is backed by exactly one fact: a fn of that name exists. Nothing has
    # read what it asserts, so it may pass while checking something else. This partitions
    # that tier by whether the named test's text even MENTIONS a distinctive term the clause
    # names -- the weakest evidence of aboutness there is, and still more than a name match.
    #
    # IT CANNOT DE-CREDIT, and that is deliberate. A silent recognizer is not a finding:
    # zero overlap does not prove a test is off-subject, and overlap does not prove it is on
    # it. What the split gives is a POPULATION -- the clauses where a name match is the only
    # evidence AND nothing in the test names the subject -- which is what #246's proposal 2
    # needs before anyone decides whether to make reverse citation mandatory.
    # ABOUTNESS SIGNAL over BOTH tiers (#246 for NAMED, #192 for CITED) -- REPORTED, NEVER
    # GATED. For a NAMED clause the test is the §9 row; for a CITED clause it is THE TEST
    # THAT CARRIES THE CITATION, which is often not the §9 row at all. Comparing a CITED
    # clause against its §9 row would fail in the REASSURING direction, because a §9-named
    # test usually does mention its clause's terms.
    def _about(pop, test_of):
        buckets = {"hit": [], "miss": [], "unmeasurable": []}
        for c in sorted(pop):
            toks = test_of(c)
            if toks is None:
                continue
            b, _m = about_bucket(all_terms.get(c) or set(), toks)
            buckets[b].append(c)
        return buckets

    def _named_tokens(c):
        t = clause_test.get(c)
        return harness.get(t, {}).get("tokens") if t else None

    def _citing_tokens(c):
        return citing_tokens(c, harness)

    for label, pop, test_of, issue in (
            ("NAMED", tier_named, _named_tokens, "#246"),
            ("CITED", tier_cited, _citing_tokens, "#192")):
        b = _about(pop, test_of)
        n = sum(len(v) for v in b.values())
        if not n:
            continue
        which = "§9-named test" if label == "NAMED" else "CITING test"
        print(f"      — aboutness over the {len(pop)} {label} ({issue}); REPORTS, never gates —")
        print(f"          {len(b['hit']):>4} the {which}'s text mentions a term the clause names")
        print(f"          {len(b['miss']):>4} it mentions NONE — the citation is the only evidence")
        print(f"          {len(b['unmeasurable']):>4} the clause names no distinctive term: UNMEASURABLE, not clean")
        for c in b["miss"]:
            terms = sorted(all_terms.get(c) or set())[:4]
            print(f"                 {c:<26} terms {terms}")
        print("                 Terms are printed so a reader can triage: a row whose terms are")
        print("                 ordinary prose is CORRECTLY COMPUTED AND UNINFORMATIVE, and no")
        print("                 classifier reclassifies it — two groundings were measured and")
        print("                 both failed. Neither direction is proof, so this de-credits nothing.")
    print(f"      — by evidence strength (convention 15); NAMED+CITED partition the {len(backed)} backed —")
    print(f"          {len(tier_named):>4} NAMED   the §9 name matches a real fn, but NO backing-form citation")
    print(f"                 exists for the clause (by any test) — a name coincidence, not an asserted tie")
    print(f"          {len(tier_cited):>4} CITED   SOME test carries a backing-form citation (`// Backs:`/`Enforces`/")
    print(f"                 assert first-arg) for the clause, by ANY test — deliberateness. WHERE the")
    print(f"                 evidence lives splits it, and the split is a work queue, not noise:")
    print(f"                   {len(tier_cited_by_name):>4} the §9-named test carries it (pointer and evidence agree)")
    print(f"                   {tier_cited_other:>4} a DIFFERENT test carries it — backed, but the §9 row points")
    print(f"                        elsewhere than the evidence: a §9-alignment queue (#247's residue)")
    print(f"          {len(tier_proven):>4} PROVEN  a `// Proven:` marker records a seeded mutation of the obligation's")
    print(f"                 SUBJECT was shown to redden the test — ABOUTNESS, and PROVEN is a subset of CITED.")
    print(f"                 The marker is TESTIMONY that a demonstration exists at a ref, NOT a live re-run,")
    print(f"                 so it carries the mutation SUBJECT (conv 15) and a resolvable REF (conv 16(a)).")
    print(f"                 {len(tier_proven)} today — nothing is bulk-marked; each entry is an earned per-clause")
    print(f"                 demonstration. The 4th BLOCKING ratchet dimension from 0 is the next increment.")
    print(f"      {len(unbacked)} clauses have NO executable test.")
    if gated:
        cfg_only = {c: t for c, t in gated.items()
                    if not str(harness[t].get("gate", "")).startswith("runtime:")}
        rt_only = {c: t for c, t in gated.items()
                   if str(harness[t].get("gate", "")).startswith("runtime:")}
        print(f"      {len(gated)} of the backed are GATE-ONLY — every test backing them "
              f"is gated,")
        print(f"          so in a run where the gate is unsatisfied nothing verified them:")
        if cfg_only:
            print(f"          {len(cfg_only):>4} cfg-feature gated (do not run in the default build)")
        if rt_only:
            by_gate = defaultdict(int)
            for c, t in rt_only.items():
                by_gate[harness[t]["gate"]] += 1
            detail = ", ".join(f"{g.split(':', 1)[1]}={n}" for g, n in sorted(by_gate.items()))
            print(f"          {len(rt_only):>4} runtime gated ({detail}) — declared via runtime_gate!")
    if orphans:
        print(f"      {len(orphans)} executable tests cite no clause — real work the "
              f"matrix cannot see.")

    # ---- the honest category breakdown of the clauses with NO harness test ----
    print("-" * 68)
    print(f"  WHY the {len(unbacked)} clauses have no harness test (the real map):")
    order = [
        ("lint", "enforced by a document lint (binds the spec, not an impl)"),
        ("blocked", "spec has not pinned the bytes/behaviour yet (see note)"),
        ("untestable", "contradictory/undefined until reworded (filed issue)"),
        ("definitional", "a test would be a tautology (definition/ownership)"),
        ("untested", "neither tested, enforced, nor explained  <-- the real gap"),
    ]
    # The lint tools actually enforcing a clause (from live coverage, so a stale
    # `lint:` ledger label that no tool declares — reported below as UNVERIFIED —
    # never dereferences a missing `lint_cov` key here).
    lint_tools = sorted({tool for tool, _ in lint_backed.values()})
    for base, desc in order:
        n = len(by_category.get(base, []))
        if n or base == "untested":
            tag = f"lint:{'/'.join(lint_tools)}" if base == "lint" and lint_tools else base
            print(f"      {n:>4}  {tag:<24} {desc}")
    enforced = len(backed) + len(lint_backed)
    print(f"  ENFORCED (harness {len(backed)} + lint {len(lint_backed)}) = "
          f"{enforced}/{n_map} ({100.0*enforced/n_map:.1f}%). "
          f"Genuinely untested (incl. decredited): {untested_count(by_category)}.")
    # THE CAVEAT (#195). Every figure above is a CLAUSE<->TEST MAPPING measured by
    # reading spec markdown and Rust source. This tool never builds or runs the Rust crate; it does run sibling lint
    # subprocesses, but nothing it prints is conditioned on the crate
    # anything, so nothing it prints is conditioned on the code being buildable —
    # it will report a coverage figure for a crate that does not compile, in the
    # same format and with the same exit code as a true one. That happened: a
    # non-compiling `contract.rs` produced `361/913 backed, ENFORCED 396/913`.
    # A number that carries no evidence a build was attempted is unfalsifiable
    # with respect to buildability, so the claim is stated with its own limit.
    print("  CAVEAT: these are clause<->test MAPPING figures, UNVERIFIED AGAINST A")
    print("          BUILD. This tool reads spec markdown and Rust source; it never")
    print("          builds or runs the Rust crate. A named test EXISTS in the")
    print("          source — it is not known to compile, run, or pass. Run")
    print("          `cargo build && cargo test` for that; this figure cannot.")
    if gated:
        # The QUALIFIED figure. The headline above credits a clause whose only
        # backing test may not have executed; this one does not. kiss_trace never
        # runs the harness, so it cannot know whether a gate was satisfied in a
        # given run — it can only decline to count what it cannot vouch for.
        # Report both, and never let the unqualified number stand alone.
        unq = enforced - len(gated)
        print(f"  ENFORCED, EXCLUDING GATE-ONLY = {unq}/{n_map} "
              f"({100.0*unq/n_map:.1f}%) — the figure that credits no clause whose "
              f"backing may not have run.")

    if lint_label_unbacked:
        any_fail = note_fail("lint_label")
        print("-" * 68)
        print(f"  UNVERIFIED LINT LABEL: {len(lint_label_unbacked)} ledger clause(s) "
              f"claim a lint that does not declare them:")
        for cid in lint_label_unbacked[:6]:
            print(f"          - {cid}")
    if missing_note:
        any_fail = note_fail("unauditable_category")
        print("-" * 68)
        print(f"  UNAUDITABLE CATEGORY: {len(missing_note)} clause(s) are "
              f"{'/'.join(CLAIM_CATEGORIES)} with no note — a category that makes a CLAIM must say why:")
        for cid in missing_note[:6]:
            print(f"          - {cid}")

    if dangling:
        any_fail = note_fail("dangling_citation")
        print("-" * 68)
        print(f"  DANGLING CITATION: {len(dangling)} clause ID(s) cited by a test do "
              f"not exist in spec/:")
        for cid in sorted(dangling)[:8]:
            print(f"          - {cid} cited by {', '.join(dangling[cid])}")

    if args.report:
        print("-" * 68)
        print("  Executable coverage by sub-standard — HARNESS-BACKED ONLY:")
        print("  (a lint-enforced clause is NOT counted here. The FREEZE READINESS")
        print("  block below counts harness UNION lint, so its per-sub numbers are")
        print("  HIGHER by exactly that sub-standard's lint-enforced count. Two")
        print("  populations, two figures: neither is `the coverage of X` without")
        print("  naming which.)")
        per = defaultdict(lambda: [0, 0])
        for c, t in clause_test.items():
            per[sub_of(c)][1] += 1
            if c in backed:
                per[sub_of(c)][0] += 1
        for sub in sorted(per, key=lambda s: -per[s][1]):
            b, n = per[sub]
            bar = "#" * int(round(20.0 * b / n)) if n else ""
            print(f"      {sub:<9} {b:>4}/{n:<4} {100.0*b/n:>5.1f}%  {bar}")

    if args.freeze_ready:
        # umbrella §5.3 condition 3 — "The sub-standard's KISS-Conform suite exists
        # and passes, with complete bidirectional clause-to-test traceability."
        # A sub-standard with even one unbacked clause cannot advance Draft->Frozen,
        # and a §8.1 conformance claim to it ("passes the unmodified suite for that
        # sub-standard") is unbacked for exactly those clauses. Same predicate, two
        # consequences. This is where an untested MUST is a hard error rather than a
        # recorded one: it blocks the transition it invalidates, not every commit.
        want = args.freeze_ready.upper()
        subs = sorted({sub_of(c) for c in clause_test})
        if want != "ALL" and want not in subs:
            print(f"\n  unknown sub-standard `{want}`; known: {', '.join(subs)}")
            return 1
        targets = subs if want == "ALL" else [want]
        print("-" * 68)
        print("  FREEZE READINESS (umbrella §5.3 condition 3 — complete clause<->test")
        print("  traceability; also the §8.1 predicate a conformance claim rests on):")
        # A clause is TRACED for the freeze gate iff a harness test or a document
        # lint enforces it (a lint is the test of a document-binding clause, §3.3).
        # blocked / untestable / definitional / untested all leave it un-traced.
        ready = 0
        for sub in targets:
            miss = sorted(c for c in unbacked if sub_of(c) == sub and not accounted(c))
            tot = sum(1 for c in clause_test if sub_of(c) == sub)
            traced = tot - len(miss)
            if miss:
                any_fail = note_fail("freeze_ready")
                print(f"      [FAIL] {sub:<9} {traced:>3}/{tot:<4} traced (harness+lint) — "
                      f"{len(miss)} clause(s) neither harness-tested nor lint-enforced")
                for cid in miss[:3]:
                    base, note = cat_of[cid]
                    print(f"                 e.g. {cid} [{base}] {note or ''}".rstrip())
                if len(miss) > 3:
                    print(f"                 ... and {len(miss) - 3} more")
            else:
                ready += 1
                print(f"      [ OK ] {sub:<9} {tot:>3}/{tot:<4} traced (harness+lint) — may freeze on §5.3 cond. 3")
        print()
        print(f"      {ready} of {len(targets)} sub-standard(s) satisfy §5.3 condition 3.")
        if any_fail:
            print("      A Draft->Frozen transition is BLOCKED for each [FAIL] above, and a")
            print("      §8.1 conformance claim to it is unbacked for its untested clauses.")
        return 1 if any_fail else 0

    if args.ratchet:
        floor_path = os.path.join(conf_dir, "COVERAGE_FLOOR.tsv")
        floor, floor_problems = read_floor(floor_path)
        live = {
            "harness": len(backed),
            "lint": len(lint_backed),
            "untested": untested_count(by_category),
            "proven": len(proven_map),
        }
        # The three numbers are stored and compared SEPARATELY because ENFORCED = harness +
        # lint conserves under a lint->harness substitution (#187), and a one-number ratchet
        # reports a reclassification as progress (#177: untested 515 -> 502, ENFORCED unmoved).
        # But counts alone cannot tell a lint->harness UPGRADE from a laundered regression with
        # the identical signature (#213), so the lint dimension is compared as a SET against the
        # PRE-change ledger read at --base-ref. --base-ref is REQUIRED whenever a base can be
        # determined (a git checkout) — NOT gated on the lint count, because a constant-count
        # lint<->harness swap leaves every count flat and would slip through a count gate, which
        # is the very defect this check exists to close. Only a genuinely git-less run may skip
        # it, and it must SAY the lint dimension went unchecked rather than print at-the-floor.
        ledger_path = os.path.join(conf_dir, "UNBACKED.tsv")
        print("-" * 68)
        prev_lint, base_error = None, None
        prev_decredited = None
        prev_floor_harness = None
        base_ledger_all = None
        base_spec_ids = None
        if args.base_ref:
            prev_lint = base_ledger_lint(ledger_path, args.base_ref)
            prev_decredited = base_ledger_decredited(ledger_path, args.base_ref)
            prev_floor_harness = base_floor_harness(floor_path, args.base_ref)
            base_ledger_all = base_ledger_all_ids(ledger_path, args.base_ref)
            # #267 condition 3b: the base SPEC clause sets for exactly the stems of the arriving
            # clauses (arrived = live lint set minus base lint set). One `git show` per distinct
            # sub-standard, typically one. Fail-closed: if ANY needed base doc is unreadable,
            # base_spec_ids stays None and classify_ratchet declines the arrival green.
            if prev_lint is not None:
                arrived_preview = set(lint_backed.keys()) - prev_lint
                stems = set()
                for cid in arrived_preview:
                    m = RE_IDPART.match(cid)
                    if m:
                        stems.add(m.group(1).lower())
                acc, ok = set(), True
                for st in stems:
                    ids = base_spec_clause_ids(spec_dir, st, args.base_ref)
                    if ids is None:
                        ok = False
                        break
                    acc |= ids
                base_spec_ids = acc if ok else None
            if prev_lint is None:
                base_error = (f"cannot read the base ledger at `{args.base_ref}` "
                              "(ref unknown / git absent)")
        elif _in_git_repo(conf_dir):
            base_error = ("this is a git checkout but --base-ref was not given. A constant-count "
                          "lint<->harness swap is invisible to the counts, so --ratchet MUST diff "
                          "the lint SET against the base ledger")
        # else: genuinely git-less -> prev_lint stays None; classify_ratchet reports the lint
        #       dimension as NOT characterized instead of at-the-floor.
        # STALE BASE (#276). Separate from base_error: the base READ succeeded, so every
        # figure below is correct -- about a tree that is no longer what this branch merges
        # into. `None` is NOT treated as stale: the base-ledger read fails on the same
        # conditions and already reports its own refusal above.
        stale_by = base_is_current(ledger_path, args.base_ref) if args.base_ref else True
        if stale_by is not True and stale_by is not None:
            inconclusive = True
            print(f"  RATCHET: `{args.base_ref}` is NOT an ancestor of this head — it has moved "
                  f"{stale_by} commit(s)")
            print("          ahead. The floor and the live figures below were compared against")
            print("          EACH OTHER and agree; neither was certified against the tree this")
            print("          branch merges into. Rebase and re-derive before merging (#276).")
        if base_error:
            inconclusive = True
            print(f"  RATCHET: {base_error} — refusing to characterize the lint dimension rather")
            print("          than degrade to 'no movement' (the currency hazard, #213). Re-run")
            print("          with --base-ref <PR base>: origin/$base_ref on a PR, the push")
            print("          before-sha or HEAD^ on main. A defaulted base would compare against")
            print("          a stale tree.")
        # The refusal blinds the lint SET comparison ONLY. The harness/lint/untested COUNTS
        # do not need a base ref -- they are floor-vs-live -- so they are still compared
        # below. Skipping them here (the original shape) meant a genuine floor breach
        # reported INCONCLUSIVE whenever --base-ref was absent, which is strictly worse than
        # the false alarm: it turns a missing flag into a way to mask a regression. Caught by
        # the `a real floor breach OUTRANKS the missing-base refusal` control.
        # The CURRENT on-disk ledger's lint set — used to gate a completed substitution:
        # a green there requires the moved IDs to have actually left the ledger, not just
        # the base ref, or the substitution reports green on every later PR (#223).
        disk_lint = None
        disk_decredited = None
        if os.path.exists(ledger_path):
            with open(ledger_path, encoding="utf-8") as fh:
                _ledger_text = fh.read()
            disk_lint = _ledger_lint_ids(_ledger_text)
            disk_decredited = _ledger_decredited_ids(_ledger_text)
        # The clauses the CURRENT scan marks `decredited` — the live set the ratchet
        # diffs against the base to confirm an honest harness->decredited move (#261).
        live_decredited = set(by_category.get(DECREDITED, ()))
        verdict, lines = classify_ratchet(floor, live, set(lint_backed.keys()),
                                          set(backed.keys()), prev_lint, disk_lint,
                                          live_decredited=live_decredited,
                                          prev_decredited=prev_decredited,
                                          disk_decredited=disk_decredited,
                                          live_coverage=set(lint_cov.keys()),
                                          prev_floor_harness=prev_floor_harness,
                                          base_ledger_all=base_ledger_all,
                                          base_spec_ids=base_spec_ids)
        headers = {
            "incomplete": "RATCHET: floor file is incomplete.",
            "regression": "RATCHET REGRESSION: coverage moved backwards.",
            "substitution": "RATCHET SUBSTITUTION: documentary -> behavioral backing "
                            "(a strengthening the aggregate cannot show, #213).",
            "ledger_unverifiable": "RATCHET: a completed substitution is recorded but the "
                                   "on-disk ledger could not be read to verify it (#223).",
            "decrediting": "RATCHET: an honest de-crediting is recorded but the floor still "
                           "sits at the over-credited numbers — bump it (#261).",
            "lint_drift": "RATCHET: the lint SET changed under a flat count.",
            "stale": "RATCHET: the floor is STALE - coverage improved past it.",
            "uncharacterized": "RATCHET: the lint dimension moved but could not be "
                               "characterized (git-less).",
        }
        if verdict in ("at_floor", "at_floor_unchecked", "substitution_recorded",
                       "decrediting_recorded", "arrival_recorded"):
            print(f"  RATCHET: {lines[0]}"
                  + ("".join(f"\n          {ln}" for ln in lines[1:])
                     if verdict in ("substitution_recorded", "decrediting_recorded",
                                    "arrival_recorded")
                     else ""))
        else:
            any_fail = note_fail("ratchet")
            print(f"  {headers[verdict]}")
            for ln in lines:
                print(f"          {ln}")
        if floor_problems:
            any_fail = note_fail("floor_dimensions")
            print("  RATCHET: the floor file's DIMENSION SET is malformed (#271).")
            for ln in floor_problems:
                print(f"          {ln}")
        # 4th dimension (#278): the PROVEN ratchet. Independent of harness/lint/untested — it
        # is about proofs (aboutness), not backings — so it is classified on its own and reds
        # the run on its own. The count comparison needs no base ref; the DROP gate needs the
        # base proven map, which is deferred (None) while PROVEN is 0 — see classify_proven's
        # 16(d) note. At 0 -> 0 this is a one-line at-floor; the drop arm is fixture-proven.
        #
        # NOT `if "proven" in floor` (#271). That made a BLOCKING dimension opt-in from the
        # file it constrains — deleting the row switched it off silently and reported CLEAN.
        # `proven` is in DIMENSIONS, so its absence is already an `incomplete` verdict above;
        # the guard here is only to avoid indexing a floor we have just declared malformed,
        # and the run is failing either way.
        if verdict != "incomplete" and "proven" in floor:
            pverdict, plines = classify_proven(floor["proven"], proven_map, set(harness),
                                               base_proven_map=None)
            pheaders = {
                "proven_improved": "RATCHET: the PROVEN floor is STALE — proofs earned past it (#278).",
                "proven_retired": "RATCHET: a PROVEN retirement is recorded but the floor sits above "
                                  "it — bump it down (#278).",
                "proven_regression": "RATCHET REGRESSION: a proof marker was lost while its proving "
                                     "test survives (#278).",
                "proven_uncharacterized": "RATCHET: the PROVEN dimension dropped but could not be "
                                          "characterized (#278).",
            }
            if pverdict == "proven_at_floor":
                print(f"  RATCHET: {plines[0]}")
            else:
                any_fail = note_fail("proven_ratchet")
                print(f"  {pheaders[pverdict]}")
                for ln in plines:
                    print(f"          {ln}")
    elif args.strict:
        # §6.2 verbatim gates on a normative MUST with no test. A lint-enforced
        # document clause HAS a test (the lint), so strict gates on the rest.
        strict_miss = sorted(c for c in unbacked if not accounted(c))
        if strict_miss:
            any_fail = note_fail("strict_untested")
            print("-" * 68)
            print(f"  STRICT (§6.2 verbatim): {len(strict_miss)} normative MUSTs with no "
                  f"test and no lint ({len(lint_backed)} others are lint-enforced).")
            for cid in strict_miss[:8]:
                base, note = cat_of[cid]
                print(f"          - {cid} [{base}] {note or unbacked[cid]}")
            if len(strict_miss) > 8:
                print(f"          - ... and {len(strict_miss) - 8} more")
    else:
        if new_unbacked:
            any_fail = note_fail("regression_or_stale")
            print("-" * 68)
            print(f"  REGRESSION: {len(new_unbacked)} clause(s) name a test that does "
                  f"not exist and are not in the ledger:")
            for cid in sorted(new_unbacked)[:8]:
                print(f"          - {cid} names `{new_unbacked[cid]}` — no such test")
            if len(new_unbacked) > 8:
                print(f"          - ... and {len(new_unbacked) - 8} more")
            print(f"    Write the test, or record it: python tools/kiss_trace.py --update-ledger")
        if stale:
            any_fail = note_fail("stale_ledger")
            print("-" * 68)
            print(f"  STALE LEDGER: {len(stale)} clause(s) are now backed (a harness "
                  f"test or a lint) but still listed as untested-in-ledger:")
            for cid in sorted(stale)[:8]:
                print(f"          - {cid} — now enforced; refresh the ledger")
            print(f"    Fix with: python tools/kiss_trace.py --update-ledger")

    print("-" * 68)
    if any_fail:
        print("  RESULT: VIOLATIONS FOUND")
    elif inconclusive:
        # Non-zero (a caller must not treat this as a pass) but DISTINCT from 1, so a
        # script can tell "I could not measure" from "the floor moved".
        # NAME THE CAUSE, do not describe the exit code. A generic "re-run with --base-ref"
        # is actively wrong when staleness fired: --base-ref WAS passed, and the remedy is a
        # rebase, printed above. A footer that misdirects is worse than none -- the reader
        # follows it instead of the line that told them what to do (#258's shape).
        print("  RESULT: INCONCLUSIVE — the ratchet declined to answer; this is NOT a floor")
        print("          violation and NOT a clean run. The refusal is named above: either the")
        print("          base could not be read (pass --base-ref), or it is no longer an")
        print("          ancestor of this head (rebase, then re-derive).")
    else:
        untested_n = untested_count(by_category)
        print(f"  RESULT: CLEAN — document consistency holds; every clause is harness-"
              f"tested\n          ({len(backed)}), lint-enforced ({len(lint_backed)}), or "
              f"recorded as unbacked.")
        # "recorded with its reason" was FALSE for the overwhelming majority: when this was
        # corrected, 492 of 493 `untested` rows carried no reason at all. The CLAIM-bearing
        # categories DO carry one -- blocked/untestable/definitional/decredited were 100%
        # complete -- so the tool was not describing THEM wrongly; it was extending their
        # property to a category that never had it. A tool asserting a triage nobody
        # performed is the same defect as a ledger note nobody re-read (#290).
        # EXCLUDE lint: it is reported as its OWN limb above, so counting lint rows here
        # again put them in two limbs of a three-limb sentence and inflated the figure from
        # 26 to 59. "Of the unbacked" means the remainder AFTER harness and lint.
        _curated = sum(1 for _c, _p in ledger.items()
                       if _c in unbacked and _p["category"] not in ("untested", "lint")
                       and _p["note"])
        print(f"          Of the unbacked, {_curated} carry a curated reason; the rest are "
              f"bare `untested`, which records that no test exists and nothing more.")
        print(f"  NOTE:   the number that must reach 0 is the GENUINELY-UNTESTED count: "
              f"{untested_n}\n          ({'/'.join(CLAIM_CATEGORIES)} are accounted for; "
              f"see the breakdown above).")
    # An INCONCLUSIVE is a THIRD state, not the absence of a reason (#347 review). Without
    # this, `--why-red` prints `<none>` while the tool exits 2 because the ratchet DECLINED
    # to answer -- the instrument built to tell one red from another having a state where it
    # explains nothing. That is the decline-vs-failure collapse this PR exists to close,
    # inside the reporting half of the same PR.
    reason_set = set(fail_reasons) | ({"inconclusive"} if inconclusive else set())

    if args.why_red:
        print("-" * 68)
        print("  FAILURE REASONS: " + (", ".join(sorted(reason_set)) or "<none>"))
        if inconclusive:
            print("  `inconclusive` is a DECLINE, not a failure: the check could not be made,")
            print("  which is neither green nor a violation. See the run above for which one.")
        return 0

    if args.assert_known_red:
        # THE TOLERATED RED IS `strict_untested` AND NOTHING ELSE. `continue-on-error`
        # makes the run conclusion SUCCESS either way, so this is the only place the
        # distinction can be made -- and it is made in ONE run, never by comparing two
        # jobs, because two jobs on two checkouts is an inference rather than a check.
        seen = reason_set
        print("-" * 68)
        print("  FAILURE REASONS: " + (", ".join(sorted(seen)) or "<none>"))
        if inconclusive and seen == {"strict_untested", "inconclusive"}:
            print("  DECLINED, NOT CHARACTERIZED: a check could not be made, so whether the")
            print("  tolerated red is the only red is UNKNOWN rather than confirmed. Refusing")
            print("  the green rather than passing an unchecked condition.")
            return 1
        if seen == {"strict_untested"}:
            print("  KNOWN RED: `strict` is failing for the documented reason (untested MUSTs)")
            print("  and for no other. The tolerance is intact.")
            return 0
        if not seen:
            print("  THE TOLERANCE IS STALE: `strict` PASSED. That is news -- the untested")
            print("  MUSTs are gone, so the continue-on-error tolerance should be removed in")
            print("  the same change that earned it. Green is not the expected state here.")
            return 1
        print("  A DIFFERENT RED: `strict` is failing for reason(s) other than the documented")
        print("  one: " + ", ".join(sorted(seen - {"strict_untested"})) + ".")
        print("  A tolerated failure and a new one are different results, and this is the")
        print("  surface that tells them apart -- the run conclusion cannot.")
        return 1

    return 1 if any_fail else (2 if inconclusive else 0)


if __name__ == "__main__":
    sys.exit(main())
