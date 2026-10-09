#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
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

    def test_slice_lookahead_rejects_6_16_when_querying_6_1(self):
        # (?!\d): querying "6.1" against text whose only matching heading is "6.16"
        # must return "" — not silently slice 6.16's body as if it were 6.1's.
        self.assertEqual(kt._section_slice("## 6.16 beta\nBBB\n## 7 g\nCCC\n", "6.1"), "")

    def test_classify_6_1_slice_excludes_informative_2_6(self):
        classify = open(os.path.join(SPEC_DIR, "classify.md"), encoding="utf-8").read()
        sl = kt._section_slice(classify, "6.1")
        self.assertIn("pinned scalar dtype set", sl)          # the normative §6.1 heading body
        self.assertNotIn("Readable catalog", sl)              # the informative §2.6 table is excluded
        self.assertNotEqual(sl, "")


class QuantProjection(unittest.TestCase):
    """KISS-CLASSIFY-6.3-0012 <-> the `quant` record of 6.3-0009."""

    REAL = open(os.path.join(SPEC_DIR, "classify.md"), encoding="utf-8").read()

    def test_real_spec_agrees(self):
        self.assertEqual(kt.quant_projection_violations(self.REAL), [])

    def test_record_fields_are_the_five_of_6_3_0009(self):
        self.assertEqual(kt._quant_record_fields(self.REAL),
                         ["family", "sub_byte_bits", "block_elems", "scale_placement", "dequant_form"])

    def test_table_has_ten_rows_five_projected(self):
        rows = kt._projection_rows(kt._clause_body(self.REAL, "CLASSIFY", "6.3-0012"))
        self.assertEqual(len(rows), 10)
        self.assertEqual(sum(1 for r in rows if r[1] is not None), 5)

    def _mutate(self, old, new):
        # CRLF checkouts: the spec text may carry \r\n; normalise both sides of the mutation.
        real = self.REAL.replace("\r\n", "\n")
        self.assertEqual(real.count(old), 1, "the mutation anchor must hit exactly once")
        return kt.quant_projection_violations(real.replace(old, new))

    def test_a_quant_field_nobody_projects_to_is_caught(self):
        v = self._mutate("| `quant_block.block_size` | `block_elems` |", "| `quant_block.block_size` | — |")
        self.assertTrue(any("block_elems" in x and "0 sidecar" in x for x in v), v)

    def test_two_rows_onto_one_field_are_caught(self):
        v = self._mutate("| `scale.dtype` | — |", "| `scale.dtype` | `family` |")
        self.assertTrue(any("family" in x and "2 sidecar" in x for x in v), v)

    def test_a_projection_onto_a_field_that_does_not_exist_is_caught(self):
        v = self._mutate("| `alignment` | — |", "| `alignment` | `zero_point` |")
        self.assertTrue(any("zero_point" in x and "not a field" in x for x in v), v)

    def test_a_wrong_count_word_is_caught(self):
        v = self._mutate("The sidecar has exactly ten fields.", "The sidecar has exactly nine fields.")
        self.assertTrue(any("nine" in x for x in v), v)

    def test_dropping_the_dequant_form_requirement_is_caught(self):
        v = self._mutate("| `dequant_form` | `dequant_form` |", "| `dequant_form` | — |")
        self.assertTrue(any("dequant_form" in x for x in v), v)

    def test_a_missing_clause_is_a_violation_not_a_pass(self):
        self.assertTrue(kt.quant_projection_violations("no clauses here"))


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
