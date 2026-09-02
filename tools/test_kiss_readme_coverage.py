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
import shutil
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


_TMPDIRS = []


def readme_with(body):
    """A fixture README. The tree is registered for cleanup rather than leaked (#350)."""
    d = tempfile.mkdtemp()
    _TMPDIRS.append(d)
    p = pathlib.Path(d) / "README.md"
    p.write_text(body, encoding="utf-8")
    return str(p)


def tearDownModule():
    for d in _TMPDIRS:
        shutil.rmtree(d, ignore_errors=True)


class ReadmeBindingTest(unittest.TestCase):
    def test_an_agreeing_figure_is_quiet(self):
        """Control. Without it a lint hardcoded to report drift would pass every case below."""
        bad, claimed, _ = rc.check(readme_with("harness 380<!-- bound:harness -->\n"), ACTUAL)
        self.assertEqual(bad, [])
        self.assertEqual(claimed, [("harness", 380, 1)])

    def test_a_DRIFTED_figure_is_caught(self):
        """The whole point: the README says one thing and the tree reports another."""
        bad, _c, _a = rc.check(readme_with("harness 379<!-- bound:harness -->\n"), ACTUAL)
        self.assertEqual(bad, [("harness", 379, 380, 1)])

    def test_the_marker_binds_the_VISIBLE_number(self):
        """The #350 review's finding, kept as a control.

        The first form put the value INSIDE the comment, so the bound value and the value
        a reader sees were two objects — and editing the prose alone reported CLEAN. The
        guard was invariant under the exact drift it exists to catch. With the marker as a
        POINTER there is only one number, so there is no shadow for it to agree with.
        """
        bad, claimed, _ = rc.check(
            readme_with("we have 999<!-- bound:harness --> clauses\n"), ACTUAL)
        self.assertEqual(bad, [("harness", 999, 380, 1)],
                         "editing the visible number did not redden — the marker is "
                         "carrying its own copy again")
        self.assertEqual(claimed, [("harness", 999, 1)])

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
        bad, _c, _a = rc.check(readme_with("380<!-- bound:harnes -->\n"), ACTUAL)
        self.assertEqual(bad, [("harnes", 380, None, 1)])

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


    def test_the_FIRST_of_two_copies_is_checked_too(self):
        """#359, and the ORDER is the whole control.

        `claimed` was a dict comprehension, so a repeated key kept the LAST occurrence.
        That means a wrong SECOND copy was caught all along -- the silent case is a wrong
        FIRST copy followed by a right one, which is what this fixture is. Under the old
        code this returned CLEAN: a marker that looked bound and was never compared.

        Getting this backwards would produce a control that passes before and after the
        fix, which is the shape of a test that proves nothing.
        """
        bad, claimed, _ = rc.check(readme_with("""we say 999<!-- bound:harness --> up here
and 380<!-- bound:harness --> down here
"""), ACTUAL)
        self.assertEqual(len(claimed), 2, "both occurrences must be collected: %r" % (claimed,))
        self.assertEqual(bad, [("harness", 999, 380, 1)],
                         "the FIRST copy was dropped before comparison — a repeated key is "
                         "silently overwriting again: %r" % (bad,))

    def test_two_copies_that_AGREE_are_legal(self):
        """Paired control, and it is what stops the fix from becoming 'ban duplicates'.

        A figure may honestly appear twice. Forbidding that would push authors back to
        unbound prose, which is the disease this tool treats. Agreement is the requirement,
        not uniqueness.
        """
        bad, claimed, _ = rc.check(readme_with("""380<!-- bound:harness --> backed
restated: 380<!-- bound:harness --> backed
"""), ACTUAL)
        self.assertEqual(bad, [], "two AGREEING copies must be clean: %r" % (bad,))
        self.assertEqual([(k, v) for k, v, _ln in claimed],
                         [("harness", 380), ("harness", 380)])

    def test_a_mismatch_names_the_LINE_so_the_stale_copy_can_be_found(self):
        """With repeats legal, `harness is wrong` no longer identifies WHICH copy.

        A message that names the key alone sends the reader to search the file for a figure
        that may appear anywhere -- and the copy they find first may be the correct one.
        """
        bad, _c, _a = rc.check(readme_with("""intro line
380<!-- bound:harness --> here is right
padding
999<!-- bound:harness --> here is wrong
"""), ACTUAL)
        self.assertEqual(bad, [("harness", 999, 380, 4)],
                         "the mismatch must carry its line number: %r" % (bad,))


if __name__ == "__main__":
    unittest.main()
