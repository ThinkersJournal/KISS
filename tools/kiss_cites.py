#!/usr/bin/env python3
"""KISS-Conform citation audit — WHAT backs a reverse-cited clause?

`kiss_trace.py` credits a clause as backed by either direction:

  * FORWARD  — the spec's `*Test:*` name exists as a `#[test] fn` in the harness.
  * REVERSE  — some test's scope (its body plus the contiguous `//` comment block
               directly above it) contains the clause ID.

The reverse direction is deliberate and documented: a citation, not a name, is
what ties a requirement to executable code when the two drift. But the scanner
matches a clause ID **anywhere** in that scope, so it cannot distinguish a test
that BACKS a clause from one that merely MENTIONS it — in a lookup-key string, a
fixture, or a comment discussing a neighbouring requirement.

That is not hypothetical. Writing `clause_block(&ops, "KISS-OPS-6.0-0001")` to
LOCATE a clause in the spec text credited the citing test with backing it; the
clause was lint-enforced, so the accident swapped a document lint for an
incidental string match and `ENFORCED` did not move (harness +1, lint -1). See
the issue linked from the burndown.

This tool answers the syntactic half of "by what, exactly?": for every clause
whose backing is REVERSE-ONLY — no forward-named test, so the citation is
load-bearing — where does the clause ID actually occur?

  assertion   inside an assert!/assert_eq!/assert_ne!/assert_golden/expect/panic
              statement — the ID is bound to something the test checks
  code        elsewhere in the body: a lookup key, a fixture path, a const
  comment     only in the `//` block the scanner also reads

A clause with NO assertion-position occurrence is a CANDIDATE over-credit. It is
not a finding: a test can legitimately back a clause its assertions never name.
The output is a list to adjudicate.

Scope note: forward-backed clauses are out of scope by construction. Their credit
comes from the test's NAME matching the spec's `*Test:*`, not from a mention, so
the mention-vs-backing ambiguity cannot arise for them.

Exit status is 0 unless `--strict` is passed; the audit reports, it does not gate.
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import kiss_trace as kt  # noqa: E402  (path set above)

# Any `assert…(` call — the std macros AND the project's own helpers
# (`assert_golden`, `assert_token`), which bind a clause ID as their first
# argument and are the strongest citation form in this harness. Matched by shape
# rather than by a fixed list, so a new helper does not silently read as data.
RE_ASSERT = re.compile(r"\bassert[a-z_]*!?\s*\(|\bpanic!|\bunreachable!|\.expect(?:_err)?\s*\(")

# Comment wording that DISCLAIMS backing rather than asserting it. A comment
# citation is the sanctioned reverse form, so the bucket is only actionable when
# the sentence around the ID says the test does *not* enforce it.
RE_CONTRASTIVE = re.compile(
    r"\b(never|not |unlike|distinct from|rather than|cross-reference|"
    r"cross-references|as opposed to|instead of|would reverse-cite|no longer)\b",
    re.I,
)


def enclosing_statement(scope, pos):
    """Text of the statement containing `pos`.

    Delimited by the nearest preceding `;`, `{` or `}` and the nearest following
    `;`. Crude by design: a Rust parser is not needed to answer "is this
    occurrence inside an assertion call", and a heuristic that is explainable in
    one sentence is auditable in a way a parser is not.
    """
    start = max(scope.rfind(ch, 0, pos) for ch in ";{}")
    end = scope.find(";", pos)
    if end == -1:
        end = len(scope)
    return scope[start + 1:end]


def classify_occurrence(scope, pos):
    """`comment` / `comment-contrastive` / `assertion` / `code` for one occurrence.

    A clause ID bound to a variable (`let home = "KISS-…";`) is `code` even when
    that variable is later interpolated into an assertion message. The indirection
    is deliberately NOT followed: in the case that motivated this audit the
    variable was a spec-lookup key whose assertions all cited a different clause,
    so following it would have concealed the finding rather than resolved it.
    """
    nl = scope.find("\n", pos)
    line_start = scope.rfind("\n", 0, pos) + 1
    line = scope[line_start:nl if nl != -1 else len(scope)]
    if line.lstrip().startswith("//"):
        return "comment-contrastive" if RE_CONTRASTIVE.search(line) else "comment"
    stmt = enclosing_statement(scope, pos)
    return "assertion" if RE_ASSERT.search(stmt) else "code"


def _scopes(conf_dir):
    """{test_name: scope} — the exact text `kiss_trace.discover_tests` scans for
    citations: the test's body PLUS the contiguous `//` comment block above it.

    This mirrors that function's loop rather than re-finding tests by name,
    because the anchor matters: `RE_RUST_TEST` matches from the `#[test]`
    attribute, so `_leading_comment` looks above the attribute and picks up the
    doc block. Anchoring on `fn` instead looks above the attribute line, finds no
    `//`, and silently loses every comment citation.
    """
    out = {}
    for root, dirs, files in os.walk(conf_dir):
        dirs[:] = [d for d in dirs if d != "target"]
        for fn in sorted(files):
            if not fn.endswith(".rs"):
                continue
            path = os.path.join(root, fn)
            try:
                src = open(path, encoding="utf-8").read()
            except OSError:
                continue
            for m in kt.RE_RUST_TEST.finditer(src):
                brace = src.find("{", m.end() - 1)
                body = src[m.start():kt._body_span(src, brace)] if brace != -1 else m.group(0)
                out[m.group(1)] = body + "\n" + kt._leading_comment(src, m.start())
    return out


def audit(spec_dir, conf_dir):
    """Return (rows, stats). One row per reverse-only-backed clause."""
    # Mirror kiss_trace's own discovery exactly — same SPECS list, same parser,
    # same DocResult — so this audit measures the population the gate measures. A
    # parallel walker that found a different clause set would answer a different
    # question and look like a disagreement.
    results = []
    for stem in kt.SPECS:
        path = os.path.join(spec_dir, stem + ".md")
        res = kt.DocResult(stem)
        if os.path.exists(path):
            kt.parse(path, res)
        results.append(res)

    harness = kt.discover_tests(conf_dir)
    scopes = _scopes(conf_dir)

    # clause -> its spec-named test, and the reverse citation index, exactly as
    # kiss_trace builds them (§9 matrix rows).
    clause_test = {}
    for res in results:
        for cid, t, _ in res.matrix:
            clause_test[cid] = t
    cited = {}
    for tname, info in harness.items():
        for c in info["clauses"]:
            cited.setdefault(c, set()).add(tname)

    # The population kiss_trace actually measures for coverage: the clauses with a
    # §9 matrix row. Iterating every ID found in the prose would report reverse
    # citations for clauses that have no matrix row at all — inflating
    # `reverse_only` and putting out-of-scope rows in the candidate list. Caught in
    # review of #190; the first published run overstated the count.
    all_ids = set(clause_test)

    rows, stats = [], {"forward": 0, "reverse_only": 0, "assertion": 0,
                       "code_no_assertion": 0, "comment_contrastive": 0,
                       "comment_affirmative": 0}
    for c in sorted(all_ids):
        fwd = clause_test.get(c) in harness
        rev = cited.get(c, set())
        if fwd:
            stats["forward"] += 1
            continue
        if not rev:
            continue
        stats["reverse_only"] += 1

        kinds, where = set(), []
        for t in sorted(rev):
            scope = scopes[t]
            for om in re.finditer(re.escape(c), scope):
                k = classify_occurrence(scope, om.start())
                kinds.add(k)
                where.append((t, k))
            # SELF-CHECK. kiss_trace said this test cites this clause; if our
            # reconstruction of the same scope cannot find the ID, the two
            # disagree and every bucket below is meaningless. Fail loudly rather
            # than silently classify — a reconstruction that misses the leading
            # comment block would quietly report every comment citation as
            # "no assertion", which is a fabricated candidate list.
            if not any(tt == t for tt, _ in where):
                raise SystemExit(
                    f"AUDIT SELF-CHECK FAILED: kiss_trace records `{t}` as citing "
                    f"{c}, but the reconstructed scope does not contain it. The "
                    f"scope reconstruction has drifted from kiss_trace.discover_tests."
                )

        if "assertion" in kinds:
            stats["assertion"] += 1
            continue
        if "code" in kinds:
            # The shape that motivated this audit: the ID appears in the body as
            # DATA — a spec-lookup key, a fixture, a const — and no assertion in
            # the test names it.
            bucket = "code_no_assertion"
        elif "comment-contrastive" in kinds:
            bucket = "comment_contrastive"
        else:
            bucket = "comment_affirmative"
        stats[bucket] += 1
        rows.append({"clause": c, "bucket": bucket, "tests": sorted(rev), "where": where})
    return rows, stats


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--conformance-dir", default=None)
    ap.add_argument("--strict", action="store_true",
                    help="exit 1 if any ACTIONABLE candidate is found — the "
                         "code-as-data and contrastive-comment buckets, never the "
                         "sanctioned one (off by default: this audit reports, it "
                         "does not gate)")
    a = ap.parse_args()
    root = os.path.dirname(HERE)
    spec_dir = a.spec_dir or os.path.join(root, "spec")
    conf_dir = a.conformance_dir or os.path.join(root, "conformance")

    rows, st = audit(spec_dir, conf_dir)

    print("KISS-Conform citation audit  -  what backs a reverse-cited clause?")
    print("=" * 68)
    print(f"  {st['forward']:4d} clauses backed FORWARD (by test name) - out of scope")
    print(f"  {st['reverse_only']:4d} clauses backed REVERSE-ONLY - the citation is load-bearing")
    print("-" * 68)
    print(f"  {st['assertion']:4d} named inside an ASSERTION            - strongest form, no action")
    print(f"  {st['code_no_assertion']:4d} in CODE as DATA, no assertion names it   <- PRIMARY candidate")
    print(f"  {st['comment_contrastive']:4d} COMMENT only, wording DISCLAIMS backing  <- secondary candidate")
    print(f"  {st['comment_affirmative']:4d} COMMENT only, affirmative wording        - sanctioned form")
    print("-" * 68)
    primary = [r for r in rows if r["bucket"] == "code_no_assertion"]
    secondary = [r for r in rows if r["bucket"] == "comment_contrastive"]
    actionable = primary + secondary

    def show(rs, heading):
        print(f"  {heading}")
        for r in rs:
            print(f"    {r['clause']}")
            for t in r["tests"]:
                kinds = sorted({k for tt, k in r["where"] if tt == t})
                print(f"        {t}  ({', '.join(kinds)})")

    if not actionable:
        print("  NO CANDIDATES. Every reverse-only-backed clause is either named")
        print("  inside an assertion, or cited in an affirmative comment - the")
        print("  reverse form kiss_trace documents as sanctioned.")
    else:
        print(f"  {len(actionable)} CANDIDATE(S) TO ADJUDICATE - NOT findings.")
        print("  A test may legitimately back a clause its assertions never name.")
        print()
        if primary:
            show(primary, "PRIMARY - clause ID used as DATA, no assertion names it:")
        if secondary:
            if primary:
                print()
            show(secondary, "SECONDARY - comment wording disclaims backing:")
    print()
    print(f"  NOT listed: the {st['comment_affirmative']} affirmative comment-only citations.")
    print("  A comment citation is the form kiss_trace documents ('the comment block")
    print("  above a test'), so flagging them all would flag the convention itself.")
    # `--strict` gates on ACTIONABLE candidates only. Keying it off every
    # reverse-only row would make it fail while reporting zero candidates, because
    # the sanctioned affirmative-comment bucket is in `rows` too — a strict flag
    # that cannot report success is one nobody leaves enabled.
    return 1 if (a.strict and actionable) else 0


if __name__ == "__main__":
    sys.exit(main())
