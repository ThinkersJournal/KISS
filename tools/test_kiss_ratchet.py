"""Discrimination controls for the coverage ratchet (`kiss_trace.py --ratchet`).

The ratchet exists because `--strict` has been red since it was written, and a
check that has always been red teaches everyone to ignore it. A ratchet that
could not go red would be the same defect wearing the opposite colour, so what
has to be proven is that it fails in BOTH directions and stays green in between.

Every case builds a synthetic spec + harness, runs the REAL tool over it, and
asserts the outcome. Coverage is moved by ADDING OR REMOVING A TEST — a genuine
coverage change, not an edit to the floor — because a ratchet that only responds
to its own floor file has not been shown to respond to the thing it measures.

  1. AT THE FLOOR        floor == live                      -> green   (CONTROL)
  2. REGRESSION          a backing test disappears          -> red, "REGRESSION"
  3. STALE FLOOR         a new backing test appears         -> red, "STALE"
  4. UNTESTED RISES      a clause loses its only backing     -> red, "REGRESSION"
  5. LINT DIMENSION      lint shortfall masked by a harness
                         surplus — identical ENFORCED sums       -> red   (#187)
  6. INCOMPLETE FLOOR    a key missing from the floor file       -> red

Case 1 is the control: without it, a ratchet hardcoded to fail would pass 2-6.
Case 5 is the one a single ENFORCED number cannot see — floor 2+1 vs live 3+0
both total 3, and only comparing the parts separately catches it.

STATED LIMIT: case 5 approaches #187's substitution from the FLOOR side, not by
making a real document lint stop covering a clause. Lint coverage is discovered
from the actual `tools/` directory, so a fixture clause ID can never be
lint-covered and a live substitution cannot be staged here. What is proven is
that the two dimensions are compared independently; what is not proven is the
end-to-end path from a lint losing a clause to this check firing.

Run: python tools/test_kiss_ratchet.py
"""
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

STEMS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]
TOOL = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kiss_trace.py")


def fid(n):
    return f"KISS-OPS-6.0-{n}"


def spec_with(rows):
    """A minimal well-formed `ops.md`: body definitions + the §9 matrix."""
    out = ["## 6.0 Fixture\n\n"]
    for ordinal, test in rows:
        out.append(f"- **{fid(ordinal)}** — A fixture clause. An implementation "
                   f"MUST do the fixture thing. *Test:* `{test}`.\n")
    out.append("\n## 9. Traceability\n\n| Clause | Test |\n|---|---|\n")
    for ordinal, test in rows:
        out.append(f"| {fid(ordinal)} | `{test}` |\n")
    return "".join(out)


def harness_with(tests):
    return "\n".join(f"#[test]\nfn {t}() {{ assert_eq!(1 + 1, 2); }}\n" for t in tests)


def run_ratchet(tmp, rows, tests, floor, ledger_rows=()):
    """Build a fixture suite + floor, run the REAL ratchet, return (ok, output)."""
    spec = os.path.join(tmp, "spec")
    conf = os.path.join(tmp, "conformance")
    os.makedirs(spec, exist_ok=True)
    os.makedirs(conf, exist_ok=True)
    for s in STEMS:
        with open(os.path.join(spec, s + ".md"), "w", encoding="utf-8") as f:
            f.write(spec_with(rows) if s == "ops" else "")
    with open(os.path.join(conf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
        f.write(harness_with(tests))
    with open(os.path.join(conf, "UNBACKED.tsv"), "w", encoding="utf-8") as f:
        f.write("# fixture ledger\n")
        for cid, test in ledger_rows:
            f.write(f"{cid}\t{test}\tuntested\t\n")
    with open(os.path.join(conf, "COVERAGE_FLOOR.tsv"), "w", encoding="utf-8") as f:
        f.write("# fixture floor\n")
        for k, v in floor.items():
            f.write(f"{k}\t{v}\n")
    try:
        out = subprocess.run(
            [sys.executable, TOOL, "--ratchet", "--spec-dir", spec,
             "--conformance-dir", conf],
            capture_output=True, text=True, timeout=300)
    except subprocess.TimeoutExpired:
        return False, "TIMEOUT: the ratchet did not finish within 300s"
    return out.returncode == 0, out.stdout + out.stderr


ROWS3 = [("0042", "test_ops_fixture_a"), ("0043", "test_ops_fixture_b"), ("0044", "test_ops_fixture_c")]
ALL3 = ["test_ops_fixture_a", "test_ops_fixture_b", "test_ops_fixture_c"]
FLOOR3 = {"harness": 3, "lint": 0, "untested": 0}

failures = []


def check(name, cond, detail=""):
    if not cond:
        failures.append(f"  {name}: {detail}")


def main():
    with tempfile.TemporaryDirectory() as d:
        # 1. CONTROL — floor matches live. Must be GREEN. Without this, a ratchet
        #    hardcoded to fail passes every case below.
        ok, out = run_ratchet(os.path.join(d, "c1"), ROWS3, ALL3, FLOOR3)
        check("at-the-floor control", ok and "at the floor" in out,
              f"expected green 'at the floor', got ok={ok}\n{out[-400:]}")

        # 2. REGRESSION — a backing test disappears, so harness falls below floor.
        ok, out = run_ratchet(os.path.join(d, "c2"), ROWS3, ["test_ops_fixture_a", "test_ops_fixture_b"], FLOOR3,
                              ledger_rows=[(fid("0044"), "test_ops_fixture_c")])
        check("regression detected", (not ok) and "RATCHET REGRESSION" in out,
              f"a lost backing test did not fail the ratchet: ok={ok}\n{out[-400:]}")

        # 3. STALE FLOOR — a new backing test appears, so harness rises past floor.
        #    Must ALSO be red: green means AT the floor, never under it.
        ok, out = run_ratchet(os.path.join(d, "c3"), ROWS3, ALL3,
                              {"harness": 2, "lint": 0, "untested": 0},
                              ledger_rows=[])
        check("stale floor detected", (not ok) and "STALE" in out,
              f"coverage past the floor did not fail: ok={ok}\n{out[-400:]}")
        check("stale floor names the new value", "set the floor to 3" in out,
              f"the stale message must say what to set: {out[-400:]}")

        # 4. UNTESTED RISES — same lost test, watched from the other side. The
        #    clause is in the ledger as untested, so `untested` goes 0 -> 1.
        ok, out = run_ratchet(os.path.join(d, "c4"), ROWS3, ["test_ops_fixture_a", "test_ops_fixture_b"],
                              {"harness": 2, "lint": 0, "untested": 0},
                              ledger_rows=[(fid("0044"), "test_ops_fixture_c")])
        check("untested rise detected", (not ok) and "untested" in out,
              f"a rise in untested did not fail: ok={ok}\n{out[-400:]}")

        # 5. LINT DIMENSION WATCHED SEPARATELY — the floor declares a lint clause
        #    that is not live. ENFORCED (harness + lint) would be 3 + 0 = 3 against
        #    a floor of 2 + 1 = 3: IDENTICAL SUMS. Only comparing the parts sees it.
        #    This is #187's substitution shape, measured from the floor side.
        ok, out = run_ratchet(os.path.join(d, "c5"), ROWS3, ALL3,
                              {"harness": 2, "lint": 1, "untested": 0})
        check("lint dimension enforced separately", (not ok) and "lint" in out,
              f"a lint shortfall masked by an equal harness surplus did not fail — "
              f"the ratchet is comparing the SUM: ok={ok} {out[-500:]}")

        # 6. INCOMPLETE FLOOR — a missing key must fail loudly rather than being
        #    treated as "no constraint". A floor with a key silently absent is a
        #    ratchet with that dimension switched off.
        ok, out = run_ratchet(os.path.join(d, "c6"), ROWS3, ALL3,
                              {"harness": 3, "untested": 0})
        check("incomplete floor detected", (not ok) and "incomplete" in out,
              f"a floor missing `lint` did not fail: ok={ok}\n{out[-400:]}")

    if failures:
        print("FAIL - the coverage ratchet does not discriminate:")
        print("\n".join(failures))
        return 1
    print("ok - ratchet is green at the floor and red in BOTH directions")
    print("     (control, regression, stale-floor, untested-rise, incomplete-floor)")
    return 0


def test_kiss_ratchet_discrimination():
    """Collected by pytest; CI also runs the file in script mode.

    Without this, `pytest tools/` collects ZERO tests from a `test_*.py` file and
    reports success having run none of the controls — the #158 shape, which is
    gated in CI and has already caught one omission in this session.
    """
    assert main() == 0, "the coverage ratchet failed its discrimination controls"


if __name__ == "__main__":
    sys.exit(main())
