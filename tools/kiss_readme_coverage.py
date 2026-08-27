#!/usr/bin/env python3
"""The conformance README's coverage figures are BOUND to the artifacts that maintain them.

`conformance/README.md` carried three claims that had aged apart from the tree in the same
directory: `31 of 855 normative clauses`, `130 tests pass by default`, and `8 of 9
sub-standards are at 0.0%`. The ratcheted floor file beside it said `harness 380`. One is
maintained by CI on every merge; the other by whoever last remembered.

    A NUMBER IN A README WITH NO GENERATOR BEHIND IT DOES NOT FAIL -- IT AGES.

An abandoned branch (`salvage/docs-coverage-refresh`, 2026-07-23) carries a third set --
`114 / 13.2%` -- reached by regenerating the figures BY HAND. It is now wrong the same way
by the same mechanism, which is the argument for binding rather than refreshing.

THE METRIC DECISION IS THE DELIVERABLE, and the two numbers were NOT the same metric:

    README `31 of 855`   -- forward only. Its own sentence said the rest "name a conformance
                            test that does not exist", which is the NAMED tier.
    floor  `harness 380` -- NAMED + CITED: a clause is backed if the §9 name resolves OR
                            some test carries a backing-form citation for it.

Binding to `harness` and rewriting the sentence, because the section asks "how much of the
spec is actually executable" and `harness` answers that question, is what the floor ratchets,
and is the number CI already defends. The narrower NAMED figure is kept alongside rather than
dropped, so the finer claim is not silently lost.

WHAT IS BOUND AND WHAT IS ONLY MEASURED, stated because the difference matters: the clause
figures and the per-sub-standard claim are recomputed here and MUST match. The
"tests pass by default" count cannot be reached without executing the suite, so this lint
binds the STATIC count of discovered test fns and the README says which is which.

Run: python tools/kiss_readme_coverage.py
"""
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_trace import read_ledger  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
README = os.path.join(ROOT, "conformance", "README.md")
FLOOR = os.path.join(ROOT, "conformance", "COVERAGE_FLOOR.tsv")

# Machine-readable anchors. The README carries the number INSIDE the marker so a reader sees
# the figure in prose and the lint reads the same characters -- rather than a lint parsing
# prose, which is how the §6.7-0008 correspondence table drifted.
RE_BOUND = re.compile(r"<!--\s*bound:(?P<key>[a-z_]+)=(?P<val>[0-9]+)\s*-->")


def floor_values(path=FLOOR):
    out = {}
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            parts = line.strip().split("\t")
            if len(parts) >= 2 and parts[1].strip().isdigit():
                out[parts[0].strip()] = int(parts[1].strip())
    return out


def derived():
    """Figures recomputed from the tool that CI already runs, never from prose."""
    r = subprocess.run([sys.executable, os.path.join(HERE, "kiss_trace.py")],
                       capture_output=True, timeout=600)
    text = r.stdout.decode("utf-8", "replace")
    out = {}
    m = re.search(r"(\d+) normative clauses", text)
    if m:
        out["clauses"] = int(m.group(1))
    m = re.search(r"(\d+) executable test fns found", text)
    if m:
        out["test_fns"] = int(m.group(1))
    m = re.search(r"(\d+)/(\d+) clauses \([\d.]+%\) are backed", text)
    if m:
        out["harness"] = int(m.group(1))
    m = re.search(r"(\d+) executable tests cite no clause", text)
    if m:
        out["uncited_tests"] = int(m.group(1))
    m = re.search(r"(\d+) NAMED", text)
    if m:
        out["named"] = int(m.group(1))
    # the ledger's own row counts, so the README's breakdown cannot drift from it either
    try:
        led = read_ledger(os.path.join(ROOT, "conformance", "UNBACKED.tsv"))
        out["untested_rows"] = sum(1 for p in led.values() if p["category"] == "untested")
    except Exception:
        pass
    # sub-standards with ZERO traced clauses — the `8 of 9 at 0.0%` claim
    zero = len([1 for ln in text.splitlines()
                if re.search(r"\[FAIL\]\s+[A-Z]+\s+0/\d+", ln)])
    out["zero_coverage_subs"] = zero
    return out


def check(readme=None, actual=None):
    """[(key, readme_value, actual)] for every bound figure that disagrees.

    `readme` / `actual` are injection points for the controls ONLY. The shipped path passes
    neither, so `main()` exercises the real README and the real recompute — a control that
    only ever drove the injected form would prove the comparison and not the tool (#344).
    """
    with open(readme or README, encoding="utf-8") as fh:
        text = fh.read()
    claimed = {m.group("key"): int(m.group("val")) for m in RE_BOUND.finditer(text)}
    if not claimed:
        return None, {}, {}          # nothing bound: a vacuity, reported separately
    actual = dict(actual) if actual is not None else derived()
    fl = floor_values()
    if fl:
        actual["floor_harness"] = fl.get("harness")
        actual["floor_untested"] = fl.get("untested")
    bad = [(k, v, actual.get(k)) for k, v in sorted(claimed.items()) if actual.get(k) != v]
    return bad, claimed, actual


def main():
    # `--emit-coverage` FIRST, and cheaply. kiss_trace.py's discover_lint_coverage runs
    # EVERY sibling `kiss_*.py --emit-coverage` to collect lint-enforced clauses -- so a
    # tool that ignores the flag runs its whole main() instead. This one spawns kiss_trace,
    # which spawns this tool again: measured, it took kiss_trace from ~51s to ~200s on the
    # branch that added it, bounded only by discover_lint_coverage's 120s subprocess timeout.
    #
    # The docstring there says a lint lacking the flag "simply contributes no coverage",
    # which is true of the COVERAGE and not of the COST -- the contract assumed lints are
    # cheap to invoke, and this is the first one that is not. Emitting nothing is correct
    # here: this lint enforces no normative clause, it binds a README to the tree.
    if "--emit-coverage" in sys.argv:
        return 0

    bad, claimed, actual = check()
    print("KISS conformance README — coverage figures bound to their generators")
    print("=" * 68)
    if bad is None:
        print("  VACUITY: the README carries NO bound figures. A lint over an empty set")
        print("  passes for the wrong reason — the markers were removed or never added.")
        print("-" * 68)
        print("  RESULT: VIOLATIONS FOUND")
        return 1
    print(f"  {len(claimed):3d} figure(s) bound; recomputed from kiss_trace.py + COVERAGE_FLOOR.tsv")
    for k in sorted(claimed):
        mark = "ok " if claimed[k] == actual.get(k) else "DRIFT"
        print(f"     [{mark}] {k:22s} README {claimed[k]:>5}   actual {actual.get(k)}")
    if bad:
        print("-" * 68)
        print("  DRIFT: a README figure no longer matches what the tree reports. These")
        print("  numbers age silently — nothing else in the repository fails when they do:")
        for k, was, now in bad:
            print(f"          {k}: README says {was}, actual is {now}")
        print("  Update the README (the number AND its `<!-- bound:… -->` marker together).")
    print("-" * 68)
    print("  RESULT: VIOLATIONS FOUND" if bad else "  RESULT: CLEAN")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
