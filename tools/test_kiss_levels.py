# SPDX-License-Identifier: MIT OR Apache-2.0
r"""Controls for citation LEVELS (CireSnave decision 308 B, KISS#498).

A citation names the level it tests at: `// Backs[enforced]: KISS-X` is `enforced`, a bare
`// Backs: KISS-X` is `stated`, and the assertion-argument form is `enforced` by construction. An
unknown level word is malformed and is never credited. These controls drive the tracer's own parser
(`kiss_trace._backing_levels`, `_backing_clauses`) and the tools that read citations
(`kiss_cite_form`, `kiss_backs_matrix`, `kiss_uncited`, `kiss_cites`, `kiss_bundle`) with a levelled
citation, so a tool that did not learn the new form fails here rather than silently dropping it.

`unittest.TestCase`, not bare `assert` (Codacy gates new bare asserts).
"""
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_backs_matrix as bm  # noqa: E402
import kiss_cite_form as cf  # noqa: E402
import kiss_trace as kt  # noqa: E402

X = "KISS-OPS-6.7-0002"
Y = "KISS-OPS-6.7-0003"


class Levels(unittest.TestCase):
    def levels(self, scope, body=""):
        return kt._backing_levels(body, scope)

    def test_unmarked_is_stated(self):
        lv, bad = self.levels("// Backs: " + X)
        self.assertEqual(lv, {X: {"stated"}})
        self.assertEqual(bad, [])

    def test_enforced_is_enforced(self):
        lv, bad = self.levels("// Backs[enforced]: " + X)
        self.assertEqual(lv, {X: {"enforced"}})
        self.assertEqual(bad, [])

    def test_enforces_keyword_takes_a_level_too(self):
        lv, _ = self.levels("/// Enforces[enforced] " + X)
        self.assertEqual(lv, {X: {"enforced"}})

    def test_a_list_shares_one_level(self):
        lv, _ = self.levels("// Backs[enforced]: %s, %s" % (X, Y))
        self.assertEqual(lv, {X: {"enforced"}, Y: {"enforced"}})

    def test_assertion_argument_form_is_enforced(self):
        lv, _ = self.levels("", body='assert_golden("%s", &b, "00")' % X)
        self.assertEqual(lv, {X: {"enforced"}})

    def test_one_test_can_carry_both_levels(self):
        lv, _ = self.levels("// Backs: %s\n// Backs[enforced]: %s" % (X, X))
        self.assertEqual(lv, {X: {"stated", "enforced"}})

    def test_unknown_level_is_malformed_and_not_credited(self):
        scope = "// Backs[bogus]: " + X
        lv, bad = self.levels(scope)
        self.assertEqual(lv, {})
        self.assertEqual(bad, ["%s [bogus]" % X])
        # and the backing set that drives coverage credit does not contain it
        self.assertNotIn(X, kt._backing_clauses("", scope))

    def test_a_levelled_citation_still_backs_for_coverage(self):
        self.assertIn(X, kt._backing_clauses("", "// Backs[enforced]: " + X))

    def test_a_bare_mention_has_no_level(self):
        lv, bad = self.levels("// unlike " + X)
        self.assertEqual((lv, bad), ({}, []))


class CiteForm(unittest.TestCase):
    def scan(self, line):
        root = tempfile.mkdtemp(prefix="levels_")
        try:
            d = os.path.join(root, "conformance", "tests")
            os.makedirs(d)
            with open(os.path.join(d, "t.rs"), "w", encoding="utf-8") as fh:
                fh.write(line + "\n")
            return cf.scan(root)
        finally:
            shutil.rmtree(root, ignore_errors=True)

    def test_levelled_form_is_canonical(self):
        sites = self.scan("// Backs[enforced]: " + X)
        self.assertEqual(len(sites), 1)
        self.assertTrue(sites[0][4])

    def test_levelled_form_without_the_colon_is_flagged(self):
        sites = self.scan("// Backs[enforced] " + X)
        self.assertEqual(len(sites), 1)
        self.assertFalse(sites[0][4])


class BacksMatrix(unittest.TestCase):
    def test_a_levelled_citation_is_compared_like_any_other(self):
        tests = {"t_named": {"clauses": kt._backing_clauses("", "// Backs[enforced]: " + X)}}
        self.assertEqual(bm.disagreements(tests, {"t_named": {X}}), [])
        self.assertEqual(len(bm.disagreements(tests, {"t_named": {Y}})), 1)


if __name__ == "__main__":
    unittest.main()
