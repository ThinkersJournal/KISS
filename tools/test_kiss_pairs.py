# SPDX-License-Identifier: MIT OR Apache-2.0
"""Negative controls for tools/kiss_pairs.py (KISS-CONFORM-6.14, KISS#240).

Each control is a case that must FAIL as well as one that must pass, so the checker is shown able
to red. The last case runs the real tool over the real vector file and ledger.
"""
import contextlib
import io
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_pairs as kp  # noqa: E402

BASE = "sk6|ew2|f32|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f|-"
F = "vec.json"


def swap(token, k, value):
    f = token.split("|")
    f[k] = value
    return "|".join(f)


class Pairs(unittest.TestCase):
    def test_single_field_difference_is_a_pair(self):
        got = kp.pairs([BASE, swap(BASE, 2, "f16")])
        self.assertEqual(got[2], [(0, 1)])

    def test_two_field_difference_is_not_a_pair(self):
        got = kp.pairs([BASE, swap(swap(BASE, 2, "f16"), 6, "r3")])
        self.assertTrue(all(not v for v in got.values()))

    def test_identical_tokens_are_not_a_pair(self):
        self.assertTrue(all(not v for v in kp.pairs([BASE, BASE]).values()))

    def test_optional_trailer_counts_as_a_field(self):
        got = kp.pairs([BASE, BASE + "|ctll/d16/f32/f32/f32/st"])
        self.assertEqual(got[9], [(0, 1)])


class Ledger(unittest.TestCase):
    def test_good_row_parses(self):
        rows, bad = kp.load(f"# c\n{F}\t4\tunauthored\t{'x' * 25}\n")
        self.assertEqual((bad, list(rows)), ([], [(F, 4)]))

    def test_bad_rows_are_problems(self):
        for line in (f"{F}\t4\tunauthored\tshort", f"{F}\t4\tmaybe\t{'x' * 25}",
                     f"{F}\t99\tunauthored\t{'x' * 25}", f"{F}\tfour\tunauthored\t{'x' * 25}",
                     f"{F}\t4\tunauthored"):
            self.assertTrue(kp.load(line + "\n")[1], line)
        row = f"{F}\t4\tunauthored\t{'x' * 25}\n"
        self.assertTrue(kp.load(row + row)[1])


class Check(unittest.TestCase):
    def setUp(self):
        self.tokens = [BASE, swap(BASE, 2, "f16")]
        self.all_but_dtype = {(F, k): ("unauthored", "x" * 25) for k in range(len(kp.FIELDS)) if k != 2}

    def test_every_coordinate_accounted_is_clean(self):
        self.assertEqual(kp.check(F, self.tokens, self.all_but_dtype)[1], [])

    def test_a_coordinate_with_no_pair_and_no_row_fails(self):
        rows = dict(self.all_but_dtype)
        del rows[(F, 4)]
        bad = kp.check(F, self.tokens, rows)[1]
        self.assertEqual(len(bad), 1)
        self.assertTrue(bad[0].startswith("MISSING: field 4"))

    def test_a_row_that_outlives_its_gap_fails(self):
        rows = dict(self.all_but_dtype)
        rows[(F, 2)] = ("unauthored", "x" * 25)
        bad = kp.check(F, self.tokens, rows)[1]
        self.assertTrue(len(bad) == 1 and bad[0].startswith("STALE: field 2"))

    def test_a_row_for_another_file_does_not_excuse_this_one(self):
        rows = {("other.json", k): ("unauthored", "x" * 25) for k in range(len(kp.FIELDS))}
        self.assertTrue(kp.check(F, self.tokens, rows)[1])


class RealTree(unittest.TestCase):
    def test_the_committed_ledger_matches_the_committed_vectors(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            rc = kp.main()
        self.assertEqual(rc, 0, out.getvalue())
        self.assertIn("discriminating pair(s)", out.getvalue())


if __name__ == "__main__":
    unittest.main()
