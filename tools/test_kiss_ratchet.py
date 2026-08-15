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

The subprocess fixtures cannot stage a live lint<->harness move (a fixture clause
can never be lint-covered — lint coverage is discovered from the real `tools/`),
so the substitution / laundering / masked-downgrade cases call `classify_ratchet`
DIRECTLY with a synthetic previous lint set, and a git-fixture drives
`base_ledger_lint` to prove it reads the base by REF not disk. Together they lift
the old limit that a live substitution could not be exercised here.

Also proven (the #213 review finding, one level in): in a git checkout `--ratchet`
REQUIRES `--base-ref` — a COUNT must not gate the SET comparison, because a
constant-count lint<->harness swap leaves every count at the floor. Only a genuinely
git-less run may skip it, and it must ANNOUNCE the lint dimension went unchecked
rather than print at-the-floor.

Run: python tools/test_kiss_ratchet.py
"""
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace  # noqa: E402 — on sys.path above; classify_ratchet + base_ledger_lint

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
ran = []  # every control that executed — asserted against a pinned count so an early
          # return that skips controls cannot report success (a green run and a green run
          # that ran half the controls are otherwise the same exit code). This file is the
          # instrument that proves the instrument, so a silent skip here is the worst place.

EXPECTED_CONTROLS = 16


def check(name, cond, detail=""):
    ran.append(name)
    if not cond:
        failures.append(f"  {name}: {detail}")


def main():
    with tempfile.TemporaryDirectory() as d:
        # 1. CONTROL — floor matches live. Must be GREEN. Without this, a ratchet
        #    hardcoded to fail passes every case below.
        # The fixtures live in a git-less tempdir, so an at-floor run is GREEN but must ANNOUNCE
        # that the lint dimension went unchecked — a plain `at_floor` here is the constant-count
        # swap hole (#213), where a run that looks at-the-floor hides a harness->lint downgrade.
        ok, out = run_ratchet(os.path.join(d, "c1"), ROWS3, ALL3, FLOOR3)
        check("at-the-floor control (git-less)", ok and "at the floor" in out,
              f"expected green 'at the floor', got ok={ok}\n{out[-400:]}")
        check("git-less at-floor announces the unchecked lint dimension",
              "NOT CHARACTERIZED" in out,
              f"a git-less at-floor printed a clean at_floor instead of stating the lint "
              f"dimension went unchecked: {out[-400:]}")

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
        check("stale floor names the new value", "harness 3" in out,
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

    # ---- SUBSTITUTION vs LAUNDERING (classify_ratchet, the identity check #213) ----
    # These call the classifier directly so a real lint↔harness movement can be staged —
    # fixture clauses can never be lint-covered (lint coverage is discovered from the real
    # tools/), which is the STATED LIMIT the count-only test above could not get past.
    def cr(floor, live, live_lint, live_harness, prev_lint):
        return kiss_trace.classify_ratchet(
            floor, live, set(live_lint), set(live_harness),
            None if prev_lint is None else set(prev_lint))

    # clean lint→harness substitution: X leaves lint, arrives in harness. h+1 / l−1 / u flat.
    v, _ = cr({"harness": 2, "lint": 1, "untested": 0}, {"harness": 3, "lint": 0, "untested": 0},
              [], ["X", "A", "B"], ["X"])
    check("substitution recognized", v == "substitution", f"a lint→harness upgrade was not "
          f"recognized as a substitution: got {v}")

    # THE LAUNDERING FLIP (as important as any control here): X: lint→untested and, unrelated,
    # Y: untested→harness. Counts are BYTE-IDENTICAL to the substitution above (h+1 / l−1 / u
    # flat) but it is a regression plus a win. Identity separates them; counts cannot.
    v, lines = cr({"harness": 2, "lint": 1, "untested": 1}, {"harness": 3, "lint": 0, "untested": 1},
                  [], ["Y", "A", "B"], ["X"])
    check("laundering is NOT a substitution", v == "regression",
          f"a regression was laundered as a substitution — the detector swallowed it: got {v}")

    # MASKED DOWNGRADE under a flat count: X: lint→harness AND Z: harness→lint. Every count is
    # flat; Z silently lost its behavioral backing. harness_lost = |{X}| − Δharness(0) = 1.
    v, lines = cr({"harness": 3, "lint": 1, "untested": 0}, {"harness": 3, "lint": 1, "untested": 0},
                  ["Z"], ["X", "A", "B"], ["X"])
    check("masked harness→lint downgrade caught under a flat count", v == "regression",
          f"a downgrade hidden by an equal-count swap passed as clean: got {v}")

    # set-based at-floor: lint set unchanged, counts at floor.
    v, _ = cr({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
              ["X"], ["A", "B"], ["X"])
    check("set-based at-floor", v == "at_floor", f"got {v}")

    # git-free path (prev_lint=None, lint count unchanged) still discriminates on counts.
    v, _ = cr({"harness": 3, "lint": 0, "untested": 0}, {"harness": 2, "lint": 0, "untested": 0},
              [], ["A", "B"], None)
    check("git-free regression still fires", v == "regression", f"got {v}")

    # ---- CURRENCY HAZARD (base_ledger_lint reads the REF, not the disk, #213) ----
    with tempfile.TemporaryDirectory() as g:
        conf = os.path.join(g, "conformance")
        os.makedirs(conf)
        ledger = os.path.join(conf, "UNBACKED.tsv")

        def git(*a):
            subprocess.run(["git", "-C", g, *a], check=True, capture_output=True, text=True)
        git("init", "-q")
        git("config", "user.email", "t@t")
        git("config", "user.name", "t")
        # BASE (committed) state: X is lint.
        with open(ledger, "w", encoding="utf-8") as f:
            f.write("# ledger\nKISS-OPS-6.0-0042\ttest_x\tlint:kiss_ops\tnote\n")
        git("add", "-A")
        git("commit", "-q", "-m", "base")
        # NEW state ON DISK: X removed (moved to harness). NOT committed — precisely the
        # regenerate-the-ledger-then-run order that silences a disk-reading check.
        with open(ledger, "w", encoding="utf-8") as f:
            f.write("# ledger\n")
        base = kiss_trace.base_ledger_lint(ledger, "HEAD")
        check("base ledger is read from the REF, not the regenerated disk file",
              base == {"KISS-OPS-6.0-0042"},
              f"read the on-disk (already-new) ledger instead of the base ref: got {base}")
        # An indeterminable base MUST be None so the caller fails loud — never an empty diff,
        # which is the same silent pass reached through the environment.
        bad = kiss_trace.base_ledger_lint(ledger, "no-such-ref-xyz")
        check("indeterminable base returns None (fail-loud, not an empty diff)", bad is None,
              f"a missing ref degraded to a set instead of None: got {bad}")

    # ---- THE REVIEW FINDING (#213, one level in): a COUNT must not gate the SET check ----
    # A constant-count lint↔harness swap leaves every count at the floor, so a count gate on
    # the base read lets it pass as at_floor. Fix: in a git checkout --ratchet REQUIRES
    # --base-ref (unconditionally, not gated on the lint count); only a genuinely git-less run
    # may skip it, and it must SAY the lint dimension went unchecked, never print at_floor.
    with tempfile.TemporaryDirectory() as gd:
        gspec = os.path.join(gd, "spec")
        gconf = os.path.join(gd, "conformance")
        os.makedirs(gspec)
        os.makedirs(gconf)
        for s in STEMS:
            with open(os.path.join(gspec, s + ".md"), "w", encoding="utf-8") as f:
                f.write(spec_with(ROWS3) if s == "ops" else "")
        with open(os.path.join(gconf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
            f.write(harness_with(ALL3))
        with open(os.path.join(gconf, "UNBACKED.tsv"), "w", encoding="utf-8") as f:
            f.write("# ledger\n")
        with open(os.path.join(gconf, "COVERAGE_FLOOR.tsv"), "w", encoding="utf-8") as f:
            f.write("# floor\nharness\t3\nlint\t0\nuntested\t0\n")

        def gg(*a):
            subprocess.run(["git", "-C", gd, *a], check=True, capture_output=True, text=True)
        gg("init", "-q")
        gg("config", "user.email", "t@t")
        gg("config", "user.name", "t")
        gg("add", "-A")
        gg("commit", "-q", "-m", "base")
        r = subprocess.run([sys.executable, TOOL, "--ratchet", "--spec-dir", gspec,
                            "--conformance-dir", gconf], capture_output=True, text=True, timeout=300)
        gout = r.stdout + r.stderr
        check("git checkout --ratchet REQUIRES --base-ref (a count must not gate the set check)",
              r.returncode != 0 and "base-ref" in gout,
              f"a git-checkout --ratchet without --base-ref passed at-the-floor — the constant-"
              f"count swap hole: rc={r.returncode}\n{gout[-400:]}")

    if failures:
        print("FAIL - the coverage ratchet does not discriminate:")
        print("\n".join(failures))
        return 1
    print("ok - ratchet is green at the floor and red in BOTH directions, tells a")
    print("     substitution from a laundered regression by IDENTITY, catches a masked")
    print("     downgrade under a flat count, and reads the base ledger by ref not disk")
    return 0


def test_kiss_ratchet_discrimination():
    """Collected by pytest; CI also runs the file in script mode.

    Without this, `pytest tools/` collects ZERO tests from a `test_*.py` file and
    reports success having run none of the controls — the #158 shape, which is
    gated in CI and has already caught one omission in this session.
    """
    ran.clear()
    assert main() == 0, "the coverage ratchet failed its discrimination controls"
    # POPULATION: a green run and a green run that skipped half the controls are the
    # same exit code, so pin the count. An early return anywhere in main() leaves
    # len(ran) short and reddens here — checked AFTER main() so a mid-function return
    # cannot skip the check itself.
    assert len(ran) == EXPECTED_CONTROLS, (
        f"main() returned 0 but only {len(ran)}/{EXPECTED_CONTROLS} controls ran — an early "
        f"return skipped some, and a suite that skips controls must not report success")


if __name__ == "__main__":
    rc = main()
    if len(ran) != EXPECTED_CONTROLS:
        print(f"FAIL - only {len(ran)}/{EXPECTED_CONTROLS} controls ran (an early return "
              f"skipped some); a suite that skips controls must not exit 0")
        rc = 1
    sys.exit(rc)
