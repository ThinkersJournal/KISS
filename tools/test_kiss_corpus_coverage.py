#!/usr/bin/env python3
"""Discrimination test for the corpus op-coverage ratchet (#459).

The ratchet must red for a coverage LOSS and for a DOMAIN move, and — the load-bearing part —
its MESSAGE must say WHICH. kiss_trace's `_untested_rose` shipped the wrong reason for a year
because every test pinned the VERDICT (`v == "regression"`) and none pinned the string; a
verdict can be right for a reason that is false. So these assert the message text, not just the
verdict. Written as a `unittest.TestCase` (pytest-collectable and script-runnable) — the repo's
dominant test style, and `self.assertX` avoids Bandit B101's "use of assert" flag on bare
`assert` (which is stripped under `python -O`; harmless in tests but the new-code gate reds it).
"""
import sys, os, io, tempfile, contextlib, unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_ops import (  # noqa: E402
    classify_corpus_coverage,
    read_corpus_floor,
    corpus_coverage_ratchet,
)


def _write_tsv(text):
    f = tempfile.NamedTemporaryFile("w", suffix=".tsv", delete=False, encoding="utf-8")
    f.write(text)
    f.close()
    return f.name


def _value_error(fn):
    """Return the ValueError message fn() raises, or None if it raised no ValueError."""
    try:
        fn()
    except ValueError as e:
        return str(e)
    return None


class CorpusCoverageRatchetTests(unittest.TestCase):
    def test_corpus_coverage_ratchet_verdicts_and_reasons(self):
        # AT FLOOR — the only green.
        ok, v, lines = classify_corpus_coverage(1, 106, 1, 106)
        self.assertTrue(ok and v == "at-floor", (v, lines))
        self.assertTrue(any("1/106" in l for l in lines), "at-floor must report the fraction")

        # REGRESSION — declared dropped below the floor (numerator down).
        ok, v, lines = classify_corpus_coverage(2, 106, 1, 106)
        msg = " ".join(lines)
        self.assertTrue((not ok) and v == "regression", v)
        self.assertIn("BACKWARD", msg)
        self.assertNotIn("DOMAIN moved", msg, "a loss must not be reported as a domain move")

        # STALE / PROGRESS — declared grew: bump the floor, not a regression.
        ok, v, lines = classify_corpus_coverage(1, 106, 2, 106)
        msg = " ".join(lines)
        self.assertTrue((not ok) and v == "stale", v)
        self.assertTrue("PROGRESS" in msg and "bump" in msg, msg)

        # DENOMINATOR MOVED — declared held, total grew (ops.md added an op): NOT a regression.
        ok, v, lines = classify_corpus_coverage(1, 106, 1, 107)
        msg = " ".join(lines)
        self.assertTrue((not ok) and v == "denominator-moved", v)
        self.assertIn("NOT a coverage regression", msg)
        self.assertNotIn("BACKWARD", msg)

        # ⚠️ THE #445 DISCRIMINATOR: the two FAILING-but-different cases (a real loss vs a domain
        # move that shifts the fraction) must NOT share a message — a reader acts on the reason,
        # not the verdict, and both are red.
        _, _, reg = classify_corpus_coverage(2, 106, 1, 106)
        _, _, den = classify_corpus_coverage(1, 106, 1, 107)
        self.assertNotEqual(" ".join(reg), " ".join(den),
                            "loss and domain-move must give DIFFERENT reasons")

    def test_read_corpus_floor_parses_valid_tsv(self):
        """Control: comments and blank lines are ignored, key/int pairs parsed."""
        p = _write_tsv("# the burn-down floor\ndeclared\t1\n\ntotal\t106\n")
        try:
            self.assertEqual(read_corpus_floor(p), {"declared": 1, "total": 106})
        finally:
            os.unlink(p)

    def test_read_corpus_floor_names_the_offending_line_on_malformed_input(self):
        """A hand-editable TSV: a dropped tab or a non-integer value must raise an error that NAMES
        the line, not a bare `int('')` / `int('many')` crash that leaves the editor guessing. #468."""
        # A tab replaced by a space — the classic hand-edit — must name the line and its number.
        p = _write_tsv("declared 1\n")
        try:
            msg = _value_error(lambda: read_corpus_floor(p))
            self.assertIsNotNone(msg, "a line without a tab must raise ValueError")
            self.assertIn("declared 1", msg)
            self.assertIn(":1:", msg)
            self.assertNotIn(":2:", msg)
        finally:
            os.unlink(p)
        # A non-integer value must name the KEY (not just repeat int()'s opaque message).
        p = _write_tsv("declared\tmany\n")
        try:
            msg = _value_error(lambda: read_corpus_floor(p))
            self.assertIsNotNone(msg)
            self.assertIn("declared", msg)
            self.assertIn("many", msg)
        finally:
            os.unlink(p)

    def test_corpus_coverage_ratchet_is_fatal_and_names_a_missing_key(self):
        """A floor missing `declared`/`total` must produce a clear FATAL + rc 1, not a bare KeyError
        traceback out of classify_corpus_coverage. #468."""
        spec_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "spec")
        p = _write_tsv("declared\t1\n")  # `total` absent
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = corpus_coverage_ratchet(spec_dir, p)
            out = buf.getvalue()
            self.assertEqual(rc, 1, out)
            self.assertIn("FATAL", out)
            self.assertIn("total", out)
        finally:
            os.unlink(p)


if __name__ == "__main__":
    unittest.main()
