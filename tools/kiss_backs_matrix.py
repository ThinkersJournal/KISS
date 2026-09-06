#!/usr/bin/env python3
"""Does a test's `Backs:` citation AGREE with the §9 matrix row that names it? (#479)

`kiss_trace` credits a clause two ways — FORWARD (the spec's `*Test:*` name exists as a test) and
REVERSE (a test's scope cites the clause ID). `kiss_cites` audits the reverse direction: for a
clause backed reverse-ONLY, is the mention load-bearing or incidental?

⚠️ AND `kiss_cites` STATES ITS OWN EXEMPTION, WHICH IS THE HOLE THIS FILLS:

    "forward-backed clauses are out of scope by construction. Their credit comes from the test's
     NAME matching the spec's *Test:*, not from a mention, so the mention-vs-backing ambiguity
     cannot arise for them."

That is TRUE and INCOMPLETE. The mention-vs-backing ambiguity cannot arise — but a DIFFERENT one
does: **the citation can name the WRONG clause, and nothing compares it to the matrix.**

MEASURED (#479), by re-introducing a real miscitation in both directions:

    cite an ALREADY-BACKED clause    ledger byte-identical, ratchet "at the floor"   SILENT
    cite an UNCOVERED clause         ratchet "the floor is STALE - coverage improved past it"

⚠️ NEITHER IS AN ERROR. The second is reported as coverage getting BETTER, and the printed remedy
("set the floor to N") converts it into a permanent false credit — an instrument that recruits a
correct operator into laundering the defect. The silent half accumulates; the loud half is
absorbed. `kiss_cites` run against that same miscitation produced byte-identical output, exit 0.

WHAT THIS CHECKS, AND ONLY THIS. A DISAGREEMENT between two things the repo already writes down:

    the §9 matrix says     clause  ->  test_name
    the test says          Backs:  ->  clause id(s)

If a test is matrix-bound to M and its citations C are non-empty and C ∩ M is empty, the two
records disagree. ⚠️ IT IS NOT A SEMANTIC CHECK AND MUST NOT BECOME ONE — whether a test is ABOUT
its clause needs a human and is #278's job. This catches only self-contradiction in the records.

⚠️ CANDIDATES, NOT DEFECTS, for a reason that is real rather than hedging: a test forward-bound to
A may legitimately cite B as a second, genuine backing. Then C ∩ M is empty and nothing is wrong.
Adjudicate each; the same posture `kiss_cites` takes.

Run: python tools/kiss_backs_matrix.py [--strict]
"""
import argparse
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)

# ⚠️ REUSE kiss_trace's RECOGNIZERS, never a second parser. A private regex here could see a
# different set of tests or citations than the tool whose credits this audits, and the
# disagreement would then be between two PARSERS rather than between two RECORDS. A census
# narrower than the claim built from it is the failure mode this avoids.
import kiss_trace as kt  # noqa: E402


def matrix_rows():
    """{test_name: {clause_id, ...}} from every spec's §9 traceability matrix."""
    out = {}
    spec_dir = os.path.join(ROOT, "spec")
    for fn in sorted(os.listdir(spec_dir)):
        if not fn.endswith(".md"):
            continue
        with open(os.path.join(spec_dir, fn), encoding="utf-8") as fh:
            text = fh.read()
        for cid, test in kt.RE_MATRIX.findall(text):
            out.setdefault(test, set()).add(cid)
    return out


def disagreements(tests, matrix):
    """(test, cited, bound) for every test whose citations name none of its matrix clauses."""
    out = []
    for name, info in sorted(tests.items()):
        bound = matrix.get(name)
        cited = info.get("clauses") or set()
        if not bound or not cited:
            continue          # forward-only or reverse-only: nothing to disagree
        if not (cited & bound):
            out.append((name, sorted(cited), sorted(bound)))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--strict", action="store_true",
                    help="exit 1 when candidates are found (default: report only)")
    args = ap.parse_args(argv)

    tests = kt.discover_tests(os.path.join(ROOT, "conformance"))
    matrix = matrix_rows()

    print("-" * 74)
    print("  `Backs:` vs §9 MATRIX DISAGREEMENT (#479)")
    # ⚠️ THE SCAN SIZES ARE THE CONTROL. A broken extractor shows here as a small scan rather
    # than as a clean zero — the shape that turned "55 of 111 ledger rows name missing tests"
    # into "0 of 111" once the recognizer was widened.
    print(f"  tests discovered        {len(tests)}")
    print(f"  §9 matrix test names    {len(matrix)}")
    both = [t for t in tests if t in matrix]
    cited = [t for t in both if tests[t].get('clauses')]
    print(f"  in BOTH records         {len(both)}")
    print(f"  ...and carrying a citation (the population checked)  {len(cited)}")

    # ⚠️ POSITIVE CONTROL, THROUGH THE COMPARATOR THAT PRODUCES THE VERDICT. A nil is only a
    # measurement if the comparator can produce a hit. Built from a REAL matrix-bound test with a
    # REAL citation, re-pointed at a clause it does not name.
    if not cited:
        print("-" * 74)
        print("  COULD NOT MEASURE: no test is both matrix-bound and carrying a citation, so")
        print("  the comparator has nothing to range over and a zero would mean nothing.")
        return 2
    probe_name = cited[0]
    probe = {probe_name: {"clauses": {"KISS-NOSUCH-9.9-9999"}}}
    if not disagreements(probe, {probe_name: matrix[probe_name]}):
        print("-" * 74)
        print("  COULD NOT MEASURE: the comparator failed to flag a synthetic disagreement")
        print(f"  built from `{probe_name}`. A zero below would report success having")
        print("  verified nothing.")
        return 2
    print(f"  control                 OK - a synthetic miscitation on `{probe_name[:34]}` IS flagged")

    found = disagreements(tests, matrix)
    print("-" * 74)
    if not found:
        print(f"  CLEAN - {len(cited)} matrix-bound test(s) carry a citation; every one names")
        print("  at least one clause the §9 matrix binds to that same test.")
        return 0

    print(f"  {len(found)} CANDIDATE(S) - the test's citation and the §9 matrix disagree:")
    for name, c, b in found:
        print(f"      {name}")
        print(f"          cites  : {', '.join(c)}")
        print(f"          matrix : {', '.join(b)}")
        print(f"          file   : {tests[name].get('file')}")
    print()
    print("  CANDIDATES, not defects: a test forward-bound to A may legitimately cite B as a")
    print("  second genuine backing. Adjudicate each — correct the citation, or confirm the")
    print("  test backs both. This checks record-vs-record, never whether a test is ABOUT its")
    print("  clause; that needs a human (#278).")
    return 1 if args.strict else 0


if __name__ == "__main__":
    sys.exit(main())
