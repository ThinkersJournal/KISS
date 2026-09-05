"""Controls for the TWO coverage populations `kiss_trace` publishes (#392 review).

`--report`'s per-sub table counts HARNESS-BACKED clauses. The FREEZE READINESS block
counts harness UNION lint. They range over different sets and print the same shape, so
`CLASSIFY 64/109` and `CLASSIFY 72/109` are both correct and neither is "CLASSIFY
coverage" without naming which. The portfolio coordinator hit exactly that on #392 and
had to reconstruct the reconciliation by hand.

⚠️ THE STRONG CONTROL HERE IS THE ARITHMETIC, NOT THE LABEL. A test that asserts label
text is testing prose: it breaks on a reword and passes on a real divergence. So the
load-bearing assertion is that the two published tables RECONCILE — per sub-standard,
freeze_traced - report_backed is that sub-standard's lint-enforced count, and the
differences sum to the lint total the report itself states. That fires when the
populations actually drift apart, which is the failure a reader cannot see.

The label check is included and is deliberately minimal (each table names its population
at all), because without it a future edit could delete the disambiguation and nothing
would go red -- but it is a presence check, not a wording pin.

Run: python tools/test_kiss_coverage_populations.py
"""
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

RE_REPORT_ROW = re.compile(r"^\s+([A-Z]+)\s+(\d+)/(\d+)\s+\d+\.\d+%", re.M)
RE_FREEZE_ROW = re.compile(r"^\s+\[(?:FAIL|\s*OK\s*)\]\s+([A-Z]+)\s+(\d+)/(\d+)\s+traced", re.M)
RE_LINT_TOTAL = re.compile(r"^\s+(\d+)\s+lint:", re.M)


def _run(*args):
    env = dict(os.environ, PYTHONIOENCODING="utf-8")
    r = subprocess.run([sys.executable, os.path.join(HERE, "kiss_trace.py"), *args],
                       capture_output=True, text=True, cwd=ROOT, env=env, timeout=600)
    return r.stdout + r.stderr


def test_the_two_published_tables_reconcile_by_the_lint_count():
    """freeze_traced - report_backed == the lint-enforced count, per sub-standard."""
    report = _run("--report")
    freeze = _run("--freeze-ready", "ALL")

    rb = {m.group(1): (int(m.group(2)), int(m.group(3))) for m in RE_REPORT_ROW.finditer(report)}
    fz = {m.group(1): (int(m.group(2)), int(m.group(3))) for m in RE_FREEZE_ROW.finditer(freeze)}
    assert rb, "no rows parsed from --report; the table shape changed"
    assert fz, "no rows parsed from --freeze-ready; the table shape changed"

    shared = sorted(set(rb) & set(fz))
    assert len(shared) >= 9, f"expected the nine sub-standards, matched {shared}"

    total_gap = 0
    for sub in shared:
        backed, tot_r = rb[sub]
        traced, tot_f = fz[sub]
        # same denominator: both range over the same clause population, differing only
        # in what counts as covered. A denominator mismatch means they are no longer
        # two views of one set, which the reconciliation below would silently absorb.
        assert tot_r == tot_f, f"{sub}: denominators differ ({tot_r} vs {tot_f})"
        gap = traced - backed
        assert gap >= 0, f"{sub}: harness-only ({backed}) exceeds harness+lint ({traced})"
        total_gap += gap

    lints = [int(m.group(1)) for m in RE_LINT_TOTAL.finditer(report)]
    assert len(lints) == 1, f"expected one lint total line in --report, found {len(lints)}"
    assert total_gap == lints[0], (
        f"the two tables do not reconcile: the per-sub gaps sum to {total_gap} but the "
        f"report states {lints[0]} lint-enforced clauses. Either a clause is counted in "
        f"both populations, or one of them has changed what it counts.")


def test_each_table_names_the_population_it_counts():
    """A presence check, NOT a wording pin: without it the disambiguation could be
    deleted and nothing would go red. Deliberately matches on the distinguishing WORD
    each table must carry, not on a sentence."""
    report = _run("--report")
    freeze = _run("--freeze-ready", "ALL")
    assert "HARNESS-BACKED ONLY" in report, (
        "--report's per-sub table no longer says which population it counts; a reader "
        "cannot tell its figures from the freeze block's")
    assert "traced (harness+lint)" in freeze, (
        "the freeze rows no longer name their population")


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: the two coverage populations reconcile by the "
          f"lint count, and each table names which set it counts")
    return 0


if __name__ == "__main__":
    sys.exit(main())
