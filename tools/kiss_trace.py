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
}

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
                fh.write(f"{cid}\t{test}\tuntested\t\n")


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
    lint_cov = discover_lint_coverage(here)

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
    # A curated non-`untested`, non-`lint` category with no note is unauditable.
    missing_note = sorted(
        c for c, p in ledger.items()
        if p["category"] in ("blocked", "untestable", "definitional")
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
        nb = len(by_category["untested"])
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
          f"Genuinely untested: {len(by_category.get('untested', []))}.")

    if lint_label_unbacked:
        any_fail = True
        print("-" * 68)
        print(f"  UNVERIFIED LINT LABEL: {len(lint_label_unbacked)} ledger clause(s) "
              f"claim a lint that does not declare them:")
        for cid in lint_label_unbacked[:6]:
            print(f"          - {cid}")
    if missing_note:
        any_fail = True
        print("-" * 68)
        print(f"  UNAUDITABLE CATEGORY: {len(missing_note)} clause(s) are "
              f"blocked/untestable/definitional with no note:")
        for cid in missing_note[:6]:
            print(f"          - {cid}")

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
                any_fail = True
                print(f"      [FAIL] {sub:<9} {traced:>3}/{tot:<4} traced — "
                      f"{len(miss)} clause(s) neither harness-tested nor lint-enforced")
                for cid in miss[:3]:
                    base, note = cat_of[cid]
                    print(f"                 e.g. {cid} [{base}] {note or ''}".rstrip())
                if len(miss) > 3:
                    print(f"                 ... and {len(miss) - 3} more")
            else:
                ready += 1
                print(f"      [ OK ] {sub:<9} {tot:>3}/{tot:<4} traced — may freeze on §5.3 cond. 3")
        print()
        print(f"      {ready} of {len(targets)} sub-standard(s) satisfy §5.3 condition 3.")
        if any_fail:
            print("      A Draft->Frozen transition is BLOCKED for each [FAIL] above, and a")
            print("      §8.1 conformance claim to it is unbacked for its untested clauses.")
        return 1 if any_fail else 0

    if args.strict:
        # §6.2 verbatim gates on a normative MUST with no test. A lint-enforced
        # document clause HAS a test (the lint), so strict gates on the rest.
        strict_miss = sorted(c for c in unbacked if not accounted(c))
        if strict_miss:
            any_fail = True
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
            print(f"  STALE LEDGER: {len(stale)} clause(s) are now backed (a harness "
                  f"test or a lint) but still listed as untested-in-ledger:")
            for cid in sorted(stale)[:8]:
                print(f"          - {cid} — now enforced; refresh the ledger")
            print(f"    Fix with: python tools/kiss_trace.py --update-ledger")

    print("-" * 68)
    if any_fail:
        print("  RESULT: VIOLATIONS FOUND")
    else:
        untested_n = len(by_category.get("untested", []))
        print(f"  RESULT: CLEAN — document consistency holds; every clause is harness-"
              f"tested\n          ({len(backed)}), lint-enforced ({len(lint_backed)}), or "
              f"recorded with its reason.")
        print(f"  NOTE:   the number that must reach 0 is the GENUINELY-UNTESTED count: "
              f"{untested_n}\n          (blocked/untestable/definitional are accounted for; "
              f"see the breakdown above).")
    return 1 if any_fail else 0


if __name__ == "__main__":
    sys.exit(main())
