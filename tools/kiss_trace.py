#!/usr/bin/env python3
"""
kiss_trace.py — the KISS-Conform traceability checker.

Implements the core of KISS-Conform §6.1 (the bidirectional clause<->test
traceability matrix) and §6.2 (the suite build FAILS on any untested normative
MUST). It parses the KISS specification files under spec/, extracts every
normative clause `KISS-<SUB>-<section>-<nnnn>[letter]` and its mapped
conformance test, builds the bidirectional matrix, and reports every
traceability violation across the whole suite at once:

  - a clause defined in the body but absent from its §9 matrix (or vice versa)
  - a clause whose body `*Test:*` tag disagrees with its matrix row
  - a clause with zero or multiple `*Test:*` tags
  - a duplicate clause ID, or a clause ID whose prefix != its document
  - a test name that lacks the suite `test_` prefix
  - a conformance test mapped to more than one clause (a Conform-owned
    cross-standard test cited by a deferring clause is reported as an allowed
    deferral, not a violation)
  - a normative clause defined in the informative-only umbrella

Exit status is 0 when the suite is clean and 1 when any violation is found, so
the checker doubles as the CI gate the standard describes. Stdlib only.

Usage:  python tools/kiss_trace.py [--spec-dir spec]
"""
from __future__ import annotations

import argparse
import os
import re
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

# spec order: umbrella first (must define no clauses), then the nine sub-standards.
SPECS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]


def sub_of(clause_id):
    m = RE_IDPART.match(clause_id)
    return m.group(1) if m else "?"


class DocResult:
    def __init__(self, stem):
        self.stem = stem
        self.prefix = "KISS-" + stem.upper()
        self.body = []       # (clause_id, line, [tests])
        self.matrix = []     # (clause_id, test, line)
        self.violations = []

    def add(self, msg):
        self.violations.append(msg)

    @property
    def clause_ids(self):
        return {c for c, _, _ in self.body} | {c for c, _, _ in self.matrix}


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
    for m in RE_MATRIX.finditer(text):
        res.matrix.append((m.group(1), m.group(2), lineno(m.start())))


def dup_scan(pairs, label, res):
    seen = defaultdict(list)
    for cid, ln in pairs:
        seen[cid].append(ln)
    for cid, lns in seen.items():
        if len(lns) > 1:
            res.add(f"duplicate {label}: {cid} at lines {', '.join(map(str, lns))}")


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

    # 5. test-naming convention: every test name starts with `test_`
    offenders = sorted({t for _, t, _ in res.matrix if not t.startswith("test_")})
    if offenders:
        eg = ", ".join(f"`{o}`" for o in offenders[:2])
        res.add(f"{len(offenders)} test name(s) lack the suite `test_` prefix (e.g. {eg})")


def main():
    ap = argparse.ArgumentParser(description="KISS-Conform traceability checker")
    ap.add_argument("--spec-dir", default=None, help="path to the spec/ directory")
    args = ap.parse_args()

    here = os.path.dirname(os.path.abspath(__file__))
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(here), "spec")

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
            subs = [sub_of(c) for c in clauses]
            if t.startswith("test_conform_") and subs.count("CONFORM") == 1:
                deferrals.append(f"cross-standard deferral: conform test `{t}` cited by {', '.join(sorted(clauses))}")
            else:
                suite_violations.append(f"test `{t}` maps to {len(clauses)} clauses: {', '.join(sorted(clauses))}")
    for cid, docs in sorted(clause_to_doc.items()):
        if len(docs) > 1:
            suite_violations.append(f"clause id {cid} appears in multiple docs: {', '.join(sorted(docs))}")

    # ---- report ----
    total_clauses = total_tests = 0
    any_fail = False
    print("KISS-Conform traceability check  —  " + spec_dir)
    print("=" * 68)
    for res in results:
        n = len(res.clause_ids)
        total_clauses += n
        if res.violations:
            any_fail = True
        note = "  (informative-only, as required)" if res.stem == "umbrella" and n == 0 and not res.violations else ""
        print(f"  [{'OK ' if not res.violations else 'FAIL'}] {res.stem:<9} {n:>4} clauses{note}")
        for v in res.violations:
            print(f"          - {v}")
    total_tests = len(test_to_clauses)
    print("-" * 68)
    if suite_violations:
        any_fail = True
        print("  SUITE-WIDE violations:")
        for v in suite_violations:
            print(f"          - {v}")
        print("-" * 68)
    if deferrals:
        print("  Cross-standard deferrals (allowed):")
        for d in deferrals:
            print(f"          · {d}")
        print("-" * 68)
    print(f"  {total_clauses} normative clauses, {total_tests} unique tests.")
    print(f"  RESULT: {'CLEAN — every clause maps 1:1 to a test, across all nine.' if not any_fail else 'VIOLATIONS FOUND'}")
    return 1 if any_fail else 0


if __name__ == "__main__":
    sys.exit(main())
