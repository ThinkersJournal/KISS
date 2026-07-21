#!/usr/bin/env python3
"""
Unit + integration tests for kiss_tables.py. Stdlib unittest only:

    python -m unittest tools/test_kiss_tables.py      # from repo root
    python tools/test_kiss_tables.py
"""
import os
import sys
import unittest

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
import kiss_tables as kt  # noqa: E402

SPEC_DIR = os.path.join(os.path.dirname(_HERE), "spec")


class LayoutFacts(unittest.TestCase):
    def test_ops_spelling(self):
        self.assertEqual(kt._layout_facts("bfloat16 (1 sign, 8 exp, 7 mantissa), bias 127"),
                         {"sign": 1, "exp": 8, "mantissa": 7, "bias": 127})

    def test_classify_spelling_no_bias(self):
        self.assertEqual(kt._layout_facts("bfloat16 (sign 1, exp 8, mantissa 7)"),
                         {"sign": 1, "exp": 8, "mantissa": 7})

    def test_compact_spelling(self):
        self.assertEqual(kt._layout_facts("bf16 (1s+8e+7m); f32 exponent range"),
                         {"sign": 1, "exp": 8, "mantissa": 7})

    def test_non_float_cell_has_no_split(self):
        self.assertEqual(kt._layout_facts("s16 two's-complement, 16-bit"), {})


class CompareLayoutFacts(unittest.TestCase):
    def test_mantissa_drift_is_a_violation(self):
        ops = {"bf16": {"sign": 1, "exp": 8, "mantissa": 7}}
        cls = {"bf16": {"sign": 1, "exp": 8, "mantissa": 8}}
        v = kt._compare_layout_facts(ops, cls)
        self.assertEqual(len(v), 1)
        self.assertIn("bf16", v[0])
        self.assertIn("mantissa", v[0])

    def test_fact_stated_by_only_one_table_is_not_a_conflict(self):
        ops = {"bf16": {"sign": 1, "exp": 8, "mantissa": 7, "bias": 127}}
        cls = {"bf16": {"sign": 1, "exp": 8, "mantissa": 7}}
        self.assertEqual(kt._compare_layout_facts(ops, cls), [])

    def test_agreement_is_clean(self):
        f = {"f32": {"sign": 1, "exp": 8, "mantissa": 23, "bias": 127}}
        self.assertEqual(kt._compare_layout_facts(f, dict(f)), [])


class SectionAnchor(unittest.TestCase):
    def test_slice_distinguishes_6_1_from_6_16(self):
        text = "## 6.1 alpha\nAAA\n## 6.16 beta\nBBB\n## 7 gamma\nCCC\n"
        self.assertIn("AAA", kt._section_slice(text, "6.1"))
        self.assertNotIn("BBB", kt._section_slice(text, "6.1"))   # must NOT bleed into 6.16
        self.assertIn("BBB", kt._section_slice(text, "6.16"))
        self.assertNotIn("CCC", kt._section_slice(text, "6.16"))  # stops at next same-level heading

    def test_classify_6_1_slice_excludes_informative_2_6(self):
        classify = open(os.path.join(SPEC_DIR, "classify.md"), encoding="utf-8").read()
        sl = kt._section_slice(classify, "6.1")
        self.assertIn("pinned scalar dtype set", sl)          # the normative §6.1 heading body
        self.assertNotIn("Readable catalog", sl)              # the informative §2.6 table is excluded
        self.assertNotEqual(sl, "")


class RealSpecIsClean(unittest.TestCase):
    def test_whole_lint_is_clean_on_shipped_spec(self):
        result = kt.check(SPEC_DIR)
        self.assertNotIsInstance(result, list, msg=f"fatal: {result}")
        violations, _auth = result
        self.assertEqual(violations, [], msg="\n".join(violations))

    def test_dtype_layouts_group_is_clean(self):
        self.assertEqual(kt.check_dtype_layouts(SPEC_DIR), [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
