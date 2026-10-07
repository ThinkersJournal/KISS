# SPDX-License-Identifier: MIT OR Apache-2.0
"""Controls for kiss_strict_report.py: the `strict` job reports the red instead of showing it.

A report that turns a red job green is only worth having if it can still go red. These controls
pin the failures it MUST NOT tolerate: an instrument that crashes, an output shape it cannot
parse, a red that is no longer the documented one, and an argparse refusal. The workflow runs the
`--_broken_invocation` case as a negative control on every run too.

Run as a script or under pytest.
"""
import io
import os
import subprocess
import sys
import time
import unittest
from contextlib import redirect_stdout
from unittest import mock

if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "controls would report success having verified nothing")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_strict_report as r  # noqa: E402

STRICT_OK = (
    "  ENFORCED (harness 396 + lint 33) = 429/955 (44.9%). "
    "Genuinely untested (incl. decredited): 479.\n")
FREEZE_OK = (
    "      [FAIL] ANNOUNCE   42/76   traced (harness+lint) - 34 clause(s) neither harness-tested\n"
    "      [ ok ] GRAMMAR    74/74   traced (harness+lint)\n"
    "      1 of 2 sub-standard(s) satisfy §5.3 condition 3.\n")
TRACEBACK = "Traceback (most recent call last):\n  File ...\nValueError: boom\n"


class StrictReport(unittest.TestCase):
    def test_parse_strict_reads_the_figures(self):
        self.assertEqual(r.parse_strict(STRICT_OK), (396, 33, 429, 955, "44.9", 479))

    def test_parse_strict_refuses_an_unrecognised_shape(self):
        # POSITIVE CONTROL is the test above: the same function reads the real shape.
        self.assertIsNone(r.parse_strict(TRACEBACK))
        self.assertIsNone(r.parse_strict(""))
        self.assertIsNone(r.parse_strict(STRICT_OK.replace("Genuinely untested", "Untested")))

    def test_parse_freeze_reads_rows_and_the_satisfy_line(self):
        rows, sat, subs = r.parse_freeze(FREEZE_OK)
        self.assertEqual(rows, {"ANNOUNCE": ("FAIL", 42, 76), "GRAMMAR": ("ok", 74, 74)})
        self.assertEqual((sat, subs), (1, 2))

    def test_parse_freeze_refuses_missing_rows_or_summary(self):
        self.assertIsNone(r.parse_freeze(TRACEBACK))
        self.assertIsNone(r.parse_freeze(FREEZE_OK.splitlines()[0]))        # rows, no summary
        self.assertIsNone(r.parse_freeze(FREEZE_OK.splitlines()[-1]))       # summary, no rows

    def test_decide_passes_on_the_known_red(self):
        code, text = r.decide(1, STRICT_OK, 1, FREEZE_OK, 0, "KNOWN RED")
        self.assertEqual(code, 0)
        self.assertIn("479", text)
        self.assertIn("1 of 2", text)
        self.assertIn("ANNOUNCE", text)

    def test_decide_fails_when_the_red_is_not_the_known_one(self):
        code, text = r.decide(1, STRICT_OK, 1, FREEZE_OK, 1, "FAILURE REASONS: other")
        self.assertEqual(code, 1)
        self.assertIn("no longer exactly the KNOWN one", text)

    def test_decide_fails_when_an_instrument_crashed(self):
        self.assertEqual(r.decide(1, TRACEBACK, 1, FREEZE_OK, 0, "")[0], 1)
        self.assertEqual(r.decide(1, STRICT_OK, 1, TRACEBACK, 0, "")[0], 1)
        # exit 0 with unparseable output is still a refusal, not a clean result
        self.assertEqual(r.decide(0, "all good", 0, "all good", 0, "")[0], 1)

    def test_emit_coverage_answers_at_once_and_spawns_nothing(self):
        """kiss_trace.discover_lint_coverage runs every sibling `kiss_*.py --emit-coverage` (#266).

        This tool spawns kiss_trace.py, which would spawn it again; so the flag is answered FIRST,
        before any subprocess. In-process: main() must return 0, print nothing and never call
        subprocess. (Mutation: delete the early return and this fails by name.)
        """
        out = io.StringIO()
        boom = AssertionError("spawned a subprocess")
        with mock.patch.object(r.subprocess, "run", side_effect=boom) as sp:
            with redirect_stdout(out):
                self.assertEqual(r.main(["--emit-coverage"]), 0)
        self.assertEqual(out.getvalue(), "")
        sp.assert_not_called()

    def test_emit_coverage_is_cheap_as_a_real_process(self):
        """The exact call test_every_kiss_lint_answers_emit_coverage_CHEAPLY makes, with a hard
        5s ceiling instead of its 120s: the real process, the real flag."""
        start = time.monotonic()
        p = subprocess.run([sys.executable, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                                         "kiss_strict_report.py"), "--emit-coverage"],
                           capture_output=True, timeout=20)
        self.assertEqual(p.returncode, 0)
        self.assertEqual(p.stdout, b"")
        self.assertLess(time.monotonic() - start, 5)

    def test_refuses_to_reenter_itself(self):
        boom = AssertionError("spawned a subprocess")
        with mock.patch.dict(os.environ, {"KISS_STRICT_REPORT_ACTIVE": "1"}):
            with mock.patch.object(r.subprocess, "run", side_effect=boom):
                self.assertEqual(r.main([]), 1)

    def test_instruments_run_with_the_recursion_marker_set(self):
        self.assertEqual(r._env()["KISS_STRICT_REPORT_ACTIVE"], "1")

    def test_broken_invocation_fails_end_to_end(self):
        # A kiss_trace.py invocation it refuses (argparse exit 2) must not report green.
        self.assertEqual(r.main(["--_broken_invocation"]), 1)


if __name__ == "__main__":
    unittest.main()
