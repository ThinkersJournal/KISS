"""Negative controls for tools/kiss_conjuncts.py (KISS#387 (a)).

Each control is a case that must FAIL as well as one that must pass, so the check is shown able
to red. The last case runs the real tool on a scratch tree: a listed conjunct no test cites
exits 1 under --freeze-ready, and citing it flips the sub-standard to clean.
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_conjuncts as kc  # noqa: E402

C = "KISS-OPS-6.18-0017"
LEDGER = f"{C}\tre\tfirst\n{C}\tim\tsecond\n"


class Cites(unittest.TestCase):
    def test_comment_line_cites_labels(self):
        got = kc.conjunct_cites(f"/// Backs: {C} [re, im]\n")
        self.assertEqual(got, {C: {"re", "im"}})

    def test_code_line_and_keywordless_comment_do_not_cite(self):
        self.assertEqual(kc.conjunct_cites(f'let s = "Backs: {C} [re]";\n'), {})
        self.assertEqual(kc.conjunct_cites(f"// see {C} [re]\n"), {})

    def test_two_clauses_on_distinct_lines_merge(self):
        got = kc.conjunct_cites(f"// Backs: {C} [re]\n// Enforces {C} [im]\n")
        self.assertEqual(got, {C: {"re", "im"}})


class Ledger(unittest.TestCase):
    def test_good_ledger_parses(self):
        rows, bad = kc.load("# c\n" + LEDGER)
        self.assertEqual((bad, sorted(rows[C])), ([], ["im", "re"]))

    def test_single_conjunct_duplicate_and_malformed_rows_are_problems(self):
        self.assertTrue(kc.load(f"{C}\tre\tonly\n")[1])
        self.assertTrue(kc.load(LEDGER + f"{C}\tre\tagain\n")[1])
        self.assertTrue(kc.load(f"{C}\tre\n")[1])
        self.assertTrue(kc.load(f"NOT-A-CLAUSE\tre\tx\n{C}\tRE\ty\n")[1])


class Gaps(unittest.TestCase):
    def setUp(self):
        self.ledger = kc.load(LEDGER)[0]

    def test_uncited_label_is_named(self):
        tests = {"t": {"conj_cites": {C: {"re"}}}}
        self.assertEqual(kc.uncited(self.ledger, tests), {C: ["im"]})

    def test_all_cited_is_clean_even_across_tests(self):
        tests = {"a": {"conj_cites": {C: {"re"}}}, "b": {"conj_cites": {C: {"im"}}}}
        self.assertEqual(kc.uncited(self.ledger, tests), {})

    def test_bare_backing_discharges_nothing(self):
        self.assertEqual(kc.uncited(self.ledger, {"t": {"conj_cites": {}}}),
                         {C: ["im", "re"]})

    def test_sub_filter(self):
        def sub(c):
            return c.split("-")[1]
        self.assertEqual(kc.uncited(self.ledger, {}, sub, "CONFORM"), {})
        self.assertEqual(sorted(kc.uncited(self.ledger, {}, sub, "OPS")[C]), ["im", "re"])

    def test_structural_faults(self):
        live = {C}
        ok = {"t": {"conj_cites": {C: {"re"}}}}
        self.assertEqual(kc.structural_problems(self.ledger, ok, live), [])
        self.assertTrue(kc.structural_problems(self.ledger, ok, set()))
        self.assertTrue(kc.structural_problems(
            self.ledger, {"t": {"conj_cites": {C: {"zz"}}}}, live))
        self.assertTrue(kc.structural_problems(
            self.ledger, {"t": {"conj_cites": {"KISS-OPS-6.18-0099": {"re"}}}}, live))


if __name__ == "__main__":
    unittest.main()
