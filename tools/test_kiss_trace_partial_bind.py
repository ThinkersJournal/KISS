# SPDX-License-Identifier: MIT OR Apache-2.0
"""Negative controls for the silent partial bind in kiss_trace (KISS #400).

The defect: `*Test:* `a`, `b`` binds only `a` (RE_TEST has no list form), so a clause author who
wrote two names reads as naming two tests while the tracer recorded one, and nothing reported the
difference. The check makes that case a violation. It does not give a clause two tests.

Each test runs the REAL `parse` + `check_doc` over a fixture document, so a violation proves the
wiring, not just the regex. The clean fixtures are the controls: a check that reds on everything
supplies no evidence either.

Run: python tools/test_kiss_trace_partial_bind.py
"""
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt

STEM = "ops"
CID = "KISS-OPS-6.1-0001"


def _doc(test_text):
    return (
        "# Fixture\n\n"
        "## 6.1 Clauses\n\n"
        f"- **{CID}** — Each op MUST carry a tag. {test_text}\n\n"
        "## 9. Matrix\n\n"
        "| Clause | Test |\n|---|---|\n"
        f"| {CID} | `first_test` |\n"
    )


def _violations(test_text):
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, STEM + ".md")
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(_doc(test_text))
        res = kt.DocResult(STEM)
        kt.parse(path, res)
        kt.check_doc(res)
    return res, [v for v in res.violations if "name list" in v]


class PartialBind(unittest.TestCase):

    def test_control_single_name_is_clean(self):
        res, v = _violations("*Test:* `first_test`.")
        self.assertEqual(res.body[0][2], ["first_test"])  # the clause really bound its test
        self.assertEqual(v, [])

    def test_control_prose_quoting_another_name_is_clean(self):
        """A backticked name later in the sentence is not a list continuation."""
        _, v = _violations("*Test:* `first_test` (asserts the `other_thing` field).")
        self.assertEqual(v, [])

    def test_comma_list_is_reported_and_binds_only_the_first(self):
        res, v = _violations("*Test:* `first_test`, `second_test`.")
        self.assertEqual(res.body[0][2], ["first_test"])  # the silent half, still true
        self.assertEqual(len(v), 1)
        self.assertIn(CID, v[0])

    def test_every_separator_form_is_reported(self):
        for sep in (", ", "; ", " + ", " & ", " / ", " and ", " or "):
            with self.subTest(sep=sep):
                _, v = _violations(f"*Test:* `first_test`{sep}`second_test`.")
                self.assertEqual(len(v), 1)

    def test_list_wrapped_across_a_line_is_reported(self):
        _, v = _violations("*Test:* `first_test`,\n  `second_test`.")
        self.assertEqual(len(v), 1)


if __name__ == "__main__":
    unittest.main()
