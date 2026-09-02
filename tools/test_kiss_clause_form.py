#!/usr/bin/env python3
"""
test_kiss_clause_form.py — controls for the clause-form lint.

The lint's own manual verification is institutionalised here, because a check
run once by hand is not a check. Three verdicts, each demonstrated:

    0  CLEAN               blocks present, none splitting a requirement
    1  CLAUSE FORM BROKEN  at least one block wedged inside its own sentence
    2  COULD NOT MEASURE   no blocks found at all

The third exists because a lint reporting CLEAN on an empty population cannot
tell "checked and fine" from "found nothing to check" -- and the second is what
happens when spec/ moves, the glob breaks, or the label is renamed. Codacy
flagged that on the lint's first revision; it was a real defect, not a nitpick.
"""
import pathlib
import subprocess
import sys
import tempfile
import unittest

LINT = pathlib.Path(__file__).resolve().parent / "kiss_clause_form.py"

WELL_FORMED = """\
- **KISS-CONFORM-9.9-0001** — A requirement whose sentence runs to its end
  without interruption, and which terminates properly.
  **Normalizes:** nothing at all.
  *Test:* `test_nine_nine`.
"""

SPLIT = """\
- **KISS-CONFORM-9.9-0001** — A requirement whose sentence is interrupted
  **Normalizes:** nothing at all.
  by a metadata block wedged inside it. *Test:* `test_nine_nine`.
"""


def run_against(spec_text):
    """Run the lint with spec/ pointed at a temporary tree; return (rc, stdout)."""
    with tempfile.TemporaryDirectory() as td:
        root = pathlib.Path(td)
        (root / "spec").mkdir()
        (root / "tools").mkdir()
        (root / "spec" / "conform.md").write_text(spec_text, encoding="utf-8")
        shim = root / "tools" / "kiss_clause_form.py"
        shim.write_text(LINT.read_text(encoding="utf-8"), encoding="utf-8")
        p = subprocess.run([sys.executable, str(shim)], capture_output=True, text=True)
        return p.returncode, p.stdout


class ClauseFormLint(unittest.TestCase):
    def test_a_well_formed_clause_is_CLEAN(self):
        rc, out = run_against(WELL_FORMED)
        self.assertEqual(rc, 0, out)
        self.assertIn("RESULT: CLEAN", out)

    def test_a_block_inside_its_own_sentence_is_BROKEN(self):
        rc, out = run_against(SPLIT)
        self.assertEqual(rc, 1, out)
        self.assertIn("CLAUSE FORM BROKEN", out)
        self.assertIn("[SPLIT]", out)

    def test_the_BROKEN_report_names_the_split_it_found(self):
        """A verdict without the offending text is not actionable."""
        _, out = run_against(SPLIT)
        self.assertIn("is interrupted", out, "the report must quote the line above")
        self.assertIn("by a metadata block", out, "and the line below")

    def test_an_EMPTY_population_is_NOT_clean(self):
        """The defect Codacy caught: no blocks found must not read as a pass."""
        rc, out = run_against("- **KISS-CONFORM-9.9-0001** — no metadata block here.\n")
        self.assertEqual(rc, 2, out)
        self.assertIn("COULD NOT MEASURE", out)
        self.assertNotIn("RESULT: CLEAN", out)

    def test_the_three_verdicts_have_DISTINCT_exit_codes(self):
        """0/1/2 must not collapse: 'could not measure' is not 'measured a violation'."""
        codes = {
            run_against(WELL_FORMED)[0],
            run_against(SPLIT)[0],
            run_against("- **KISS-CONFORM-9.9-0001** — nothing.\n")[0],
        }
        self.assertEqual(len(codes), 3, f"verdicts collapsed to {sorted(codes)}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
