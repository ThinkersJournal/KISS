"""Discrimination controls for the comparator-blindness lint (§6.8-0012).

Every case must FAIL on the defect and PASS without it. A control that passes in
both states is measuring the fixture, not the rule -- which is the failure this
lint exists to catch, so it would be a poor joke to commit it here.

Run: python tools/test_kiss_comparators.py
"""
import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_comparators as kc  # noqa: E402

DECLARED = """\
- **KISS-CONFORM-9.9-0001** - The **example** comparator MUST be byte-identical.
  **Normalizes:** nothing.
  *Test:* `test_example`.
"""

UNDECLARED = """\
- **KISS-CONFORM-9.9-0002** - The **other** comparator MUST compare within a tolerance.
  *Test:* `test_other`.
"""

NO_COMPARATOR = """\
- **KISS-CONFORM-9.9-0003** - An unrelated obligation about matrix serialization.
  *Test:* `test_unrelated`.
"""


def scan_text(body):
    with tempfile.TemporaryDirectory() as td:
        p = pathlib.Path(td) / "conform.md"
        p.write_text(body, encoding="utf-8")
        return kc.scan(str(p))


class BlindnessLintTest(unittest.TestCase):
    def test_undeclared_comparator_is_caught(self):
        """The whole point: a comparison relation declaring nothing is a violation."""
        declared, undeclared, excluded, stale = scan_text(DECLARED + UNDECLARED)
        self.assertEqual(undeclared, ["KISS-CONFORM-9.9-0002"])
        self.assertEqual(declared, ["KISS-CONFORM-9.9-0001"])

    def test_declared_comparator_passes(self):
        """Control. Without this, a lint hardcoded to fail would pass the case above."""
        declared, undeclared, _, _ = scan_text(DECLARED)
        self.assertEqual(declared, ["KISS-CONFORM-9.9-0001"])
        self.assertEqual(undeclared, [])

    def test_a_clause_naming_no_comparator_is_out_of_scope(self):
        """The detector is over-broad by design; it must still not be UNBOUNDED.

        If every clause were in scope, the exclusion list would have to enumerate the
        whole document and would stop being reviewable.
        """
        declared, undeclared, excluded, _ = scan_text(NO_COMPARATOR)
        self.assertEqual((declared, undeclared, excluded), ([], [], []))

    def test_stale_exclusion_is_caught(self):
        """An exclusion for a clause no longer in scope exempts the WRONG clause later.

        This fired twice while the lint was being written -- both times on entries the
        author had assumed rather than measured -- which is the argument for keeping it.
        """
        kc.SELECTS_ONLY["KISS-CONFORM-9.9-0404"] = "does not exist"
        try:
            _, _, _, stale = scan_text(DECLARED)
            self.assertIn("KISS-CONFORM-9.9-0404", stale)
        finally:
            del kc.SELECTS_ONLY["KISS-CONFORM-9.9-0404"]

    def test_excluded_clause_is_not_a_violation(self):
        """An explicit exclusion suppresses the finding -- and only an explicit one."""
        kc.SELECTS_ONLY["KISS-CONFORM-9.9-0002"] = "test fixture"
        try:
            _, undeclared, excluded, _ = scan_text(DECLARED + UNDECLARED)
            self.assertEqual(undeclared, [])
            self.assertEqual(excluded, ["KISS-CONFORM-9.9-0002"])
        finally:
            del kc.SELECTS_ONLY["KISS-CONFORM-9.9-0002"]

    def test_the_real_document_is_clean(self):
        """Convention 9: assert the thing under test is actually in the state claimed.

        Without this the fixture cases could all pass against a spec/conform.md that
        violates the clause the lint enforces.
        """
        declared, undeclared, _, stale = kc.scan()
        self.assertEqual(undeclared, [], f"spec/conform.md has undeclared comparators: {undeclared}")
        self.assertEqual(stale, [], f"stale exclusions in SELECTS_ONLY: {stale}")
        self.assertGreaterEqual(len(declared), 6, "the real document must carry declarations")


if __name__ == "__main__":
    unittest.main()
