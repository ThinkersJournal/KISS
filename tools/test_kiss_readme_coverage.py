"""Discrimination controls for the README coverage binding (#266).

The README's figures had aged apart from the tree in the same directory — `31 of 855`
against a ratcheted `harness 380`, and `8 of 9 sub-standards at 0.0%` when none were. An
abandoned branch carries a third set, reached by regenerating BY HAND, and it is wrong the
same way by the same mechanism.

    A NUMBER IN A README WITH NO GENERATOR BEHIND IT DOES NOT FAIL — IT AGES.

The fast controls drive `check()` with an injected `actual`, which proves the COMPARISON.
`test_the_lint_runs_end_to_end_on_the_real_readme` drives the shipped path, because a suite
that only ever exercises an injection point proves the comparison and not the tool — the
#344 lesson, applied here rather than re-learned.

Run: python tools/test_kiss_readme_coverage.py
"""
import os
import pathlib
import subprocess
import sys
import time
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_readme_coverage as rc  # noqa: E402

ACTUAL = {"harness": 380, "clauses": 932, "named": 240, "test_fns": 538,
          "uncited_tests": 124, "zero_coverage_subs": 0}


def readme_with(body):
    d = tempfile.mkdtemp()
    p = pathlib.Path(d) / "README.md"
    p.write_text(body, encoding="utf-8")
    return str(p)


class ReadmeBindingTest(unittest.TestCase):
    def test_an_agreeing_figure_is_quiet(self):
        """Control. Without it a lint hardcoded to report drift would pass every case below."""
        bad, claimed, _ = rc.check(readme_with("harness <!-- bound:harness=380 -->\n"), ACTUAL)
        self.assertEqual(bad, [])
        self.assertEqual(claimed, {"harness": 380})

    def test_a_DRIFTED_figure_is_caught(self):
        """The whole point: the README says one thing and the tree reports another."""
        bad, _c, _a = rc.check(readme_with("harness <!-- bound:harness=379 -->\n"), ACTUAL)
        self.assertEqual(bad, [("harness", 379, 380)])

    def test_NO_bound_figures_is_a_VACUITY_not_a_pass(self):
        """A lint over an empty set passes for the wrong reason.

        Deleting the markers — or never adding them — would otherwise leave the README free
        to say anything while the lint reported CLEAN. That is the same shape as an exclusion
        list deleted rather than asserted empty.
        """
        bad, _c, _a = rc.check(readme_with("no markers here, just prose about 380 clauses\n"),
                               ACTUAL)
        self.assertIsNone(bad, "an unbound README must be a VIOLATION, not a clean run")

    def test_a_figure_the_tree_does_not_report_is_caught(self):
        """A marker naming a key nothing derives cannot be silently ignored.

        Otherwise a typo in the key (`harnes=380`) would remove the figure from the gate
        while leaving it visible in the prose — the file looks bound and is not.
        """
        bad, _c, _a = rc.check(readme_with("<!-- bound:harnes=380 -->\n"), ACTUAL)
        self.assertEqual(bad, [("harnes", 380, None)])

    def test_every_kiss_lint_answers_emit_coverage_CHEAPLY(self):
        """Closes the class, not the instance (#266).

        `kiss_trace.discover_lint_coverage` runs EVERY sibling `kiss_*.py --emit-coverage`.
        A tool that ignores the flag runs its whole `main()` instead — and this one spawned
        `kiss_trace`, which spawned it again. Measured: kiss_trace went from ~51s to ~200s
        on the branch that added it, bounded only by that discovery's 120s subprocess
        timeout rather than by anything in the tool.

        The contract there says a lint lacking the flag "simply contributes no coverage" —
        true of the COVERAGE, not of the COST. This asserts the cost, so the next tool that
        forgets is caught by a test rather than by someone noticing CI got slower.
        """
        tools = sorted(p for p in HERE.glob("kiss_*.py") if p.name != "kiss_trace.py")
        self.assertGreaterEqual(len(tools), 5, "the tool glob found almost nothing")
        slow = []
        for t in tools:
            start = time.monotonic()
            subprocess.run([sys.executable, str(t), "--emit-coverage"],
                           capture_output=True, timeout=120)
            took = time.monotonic() - start
            if took > 20:
                slow.append((t.name, round(took, 1)))
        self.assertEqual(slow, [],
                         "these lints do expensive work under --emit-coverage, which "
                         "kiss_trace runs on every invocation: %r" % (slow,))

    def test_the_lint_runs_end_to_end_on_the_real_readme(self):
        """THE SHIPPED PATH, not the injection point (#344's lesson).

        Every control above passes an `actual` dict, so all of them would pass against a
        `derived()` that returned nothing at all, or a `main()` that never called `check()`.
        This runs the tool as a subprocess against the real tree and asserts the exit code
        of the thing CI runs.
        """
        r = subprocess.run([sys.executable, str(HERE / "kiss_readme_coverage.py")],
                           capture_output=True, timeout=600,
                           env={**os.environ, "PYTHONIOENCODING": "utf-8"})
        out = r.stdout.decode("utf-8", "replace")
        self.assertEqual(r.returncode, 0, "the live README does not match the tree:\n" + out[-600:])
        self.assertIn("figure(s) bound", out)
        # and it must actually be comparing something — a bound set of zero would be the
        # vacuity above arriving through the real path instead of a fixture.
        n = int(out.split("figure(s) bound")[0].strip().split()[-1])
        self.assertGreaterEqual(n, 5, "the README lost its bindings: only %d figure(s)" % n)


if __name__ == "__main__":
    unittest.main()
