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
  - a test name that lacks the suite `test_` prefix
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

Exit status is 0 when the suite is clean and 1 when any violation is found, so
the checker doubles as the CI gate the standard describes. Stdlib only.

Usage:
  python tools/kiss_trace.py                  # both checks, ratcheted by the ledger
  python tools/kiss_trace.py --strict         # fail on ANY unbacked clause (§6.2 verbatim)
  python tools/kiss_trace.py --update-ledger  # rewrite the ledger to the current truth
  python tools/kiss_trace.py --report         # per-sub-standard coverage breakdown
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

# A Rust test function in the harness: `#[test]` (possibly with intervening
# attributes such as `#[cfg(feature = "cuda")]` or `#[ignore]`) then `fn name(`.
RE_RUST_TEST = re.compile(
    r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z_][a-z0-9_]*)\s*\(", re.M)
# A `#[cfg(feature = "...")]` gate anywhere in the attribute run before a test.
RE_RUST_TEST_CFG = re.compile(
    r"#\[cfg\(feature\s*=\s*\"([a-z0-9_-]+)\"\)\]\s*(?:#\[[^\]]*\]\s*)*"
    r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z_][a-z0-9_]*)\s*\(", re.M)

# spec order: umbrella first (must define no clauses), then the nine sub-standards.
SPECS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]

LEDGER_HEADER = """\
# KISS-Conform — the unbacked-clause ledger.
#
# Each line is a normative clause whose §9-mapped conformance test DOES NOT EXIST
# in conformance/. This file is the honest record of the gap between what the
# specification claims is tested and what is actually executable.
#
# It is a RATCHET, enforced by tools/kiss_trace.py:
#   * a clause that becomes unbacked and is not listed here FAILS the build
#     (a new untested MUST is always a regression);
#   * a listed clause whose test now exists FAILS the build until its line is
#     removed (the ledger may only shrink).
#
# `kiss_trace.py --strict` ignores this file and fails on every line below. That
# is KISS-Conform §6.2 as literally written; this ledger is the interim record
# of the distance to it. The length of this file is the number that matters.
#
# Regenerate with: python tools/kiss_trace.py --update-ledger
#
# clause_id\ttest_name
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
    cid_re = re.compile(CLAUSE_ID)
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
                scope = src[m.start():_body_span(src, brace)] if brace != -1 else m.group(0)
                scope += "\n" + _leading_comment(src, m.start())
                found[name] = {
                    "file": rel,
                    "gate": gated.get(name),
                    "clauses": set(cid_re.findall(scope)),
                }
    return found


def read_ledger(path):
    """The recorded unbacked clauses: {clause_id: test_name}."""
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) >= 2:
            out[parts[0]] = parts[1]
    return out


def write_ledger(path, unbacked):
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(LEDGER_HEADER)
        for cid, test in sorted(unbacked.items()):
            fh.write(f"{cid}\t{test}\n")


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
    ap.add_argument("--conformance-dir", default=None,
                    help="path to the conformance/ harness")
    ap.add_argument("--strict", action="store_true",
                    help="fail on ANY unbacked clause, ignoring the ledger (§6.2 verbatim)")
    ap.add_argument("--update-ledger", action="store_true",
                    help="rewrite the unbacked ledger to the current truth")
    ap.add_argument("--report", action="store_true",
                    help="print the per-sub-standard executable-coverage breakdown")
    args = ap.parse_args()

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

    # ---- check 2: BINDING — does the named test actually exist? ----
    harness = discover_tests(conf_dir)
    ledger = read_ledger(ledger_path)

    # every (clause -> named test) pair across the suite, from the §9 matrices
    clause_test = {}
    for res in results:
        for cid, t, _ in res.matrix:
            clause_test[cid] = t

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
    gated = {c: t for c, t in backed.items() if harness.get(t, {}).get("gate")}

    # A citation naming a clause that does not exist is a dangling reference —
    # §3.3 burns retired IDs, so this catches a test pinned to a dead clause.
    all_clause_ids = set()
    for res in results:
        all_clause_ids |= res.clause_ids
    dangling = {cid: sorted(ts) for cid, ts in cited.items() if cid not in all_clause_ids}

    # Executable tests that cite no clause at all: real work the matrix cannot see.
    orphans = sorted(t for t, i in harness.items()
                     if not i["clauses"] and t not in set(clause_test.values()))

    if args.update_ledger:
        write_ledger(ledger_path, unbacked)
        print(f"Wrote {ledger_path}: {len(unbacked)} unbacked clauses.")
        return 0

    # a NEW unbacked clause (not in the ledger) is always a failure
    new_unbacked = {c: t for c, t in unbacked.items() if c not in ledger}
    # a ledger entry that is now backed must be removed — the ledger only shrinks
    stale = {c: t for c, t in ledger.items() if c not in unbacked}

    # ---- report ----
    total_clauses = 0
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
    print(f"      {len(unbacked)} clauses have NO executable test.")
    if gated:
        print(f"      ({len(gated)} of the backed are feature-gated and do not "
              f"run in the default build.)")
    if orphans:
        print(f"      {len(orphans)} executable tests cite no clause — real work the "
              f"matrix cannot see.")
    if ledger:
        print(f"      ledger: {len(ledger)} clauses recorded as knowingly unbacked.")

    if dangling:
        any_fail = True
        print("-" * 68)
        print(f"  DANGLING CITATION: {len(dangling)} clause ID(s) cited by a test do "
              f"not exist in spec/:")
        for cid in sorted(dangling)[:8]:
            print(f"          - {cid} cited by {', '.join(dangling[cid])}")

    if args.report:
        print("-" * 68)
        print("  Executable coverage by sub-standard:")
        per = defaultdict(lambda: [0, 0])
        for c, t in clause_test.items():
            per[sub_of(c)][1] += 1
            if c in backed:
                per[sub_of(c)][0] += 1
        for sub in sorted(per, key=lambda s: -per[s][1]):
            b, n = per[sub]
            bar = "#" * int(round(20.0 * b / n)) if n else ""
            print(f"      {sub:<9} {b:>4}/{n:<4} {100.0*b/n:>5.1f}%  {bar}")

    if args.strict:
        if unbacked:
            any_fail = True
            print("-" * 68)
            print(f"  STRICT (§6.2 verbatim): {len(unbacked)} untested normative MUSTs.")
            for cid in sorted(unbacked)[:8]:
                print(f"          - {cid} names `{unbacked[cid]}` — no such test")
            if len(unbacked) > 8:
                print(f"          - ... and {len(unbacked) - 8} more")
    else:
        if new_unbacked:
            any_fail = True
            print("-" * 68)
            print(f"  REGRESSION: {len(new_unbacked)} clause(s) name a test that does "
                  f"not exist and are not in the ledger:")
            for cid in sorted(new_unbacked)[:8]:
                print(f"          - {cid} names `{new_unbacked[cid]}` — no such test")
            if len(new_unbacked) > 8:
                print(f"          - ... and {len(new_unbacked) - 8} more")
            print(f"    Write the test, or record it: python tools/kiss_trace.py --update-ledger")
        if stale:
            any_fail = True
            print("-" * 68)
            print(f"  STALE LEDGER: {len(stale)} clause(s) are now backed but still "
                  f"listed as unbacked. The ledger may only shrink:")
            for cid in sorted(stale)[:8]:
                print(f"          - {cid} — `{ledger[cid]}` now exists; remove the line")
            print(f"    Fix with: python tools/kiss_trace.py --update-ledger")

    print("-" * 68)
    if any_fail:
        print("  RESULT: VIOLATIONS FOUND")
    else:
        print(f"  RESULT: CLEAN — document consistency holds, and every clause either "
              f"names an\n          existing test ({len(backed)}) or is recorded as "
              f"unbacked ({len(unbacked)}).")
        if unbacked:
            print(f"  NOTE:   {len(unbacked)} normative MUSTs have NO executable test. "
                  f"KISS-Conform §6.2\n          is not met until that number is 0 "
                  f"(`--strict` gates on it).")
    return 1 if any_fail else 0


if __name__ == "__main__":
    sys.exit(main())
