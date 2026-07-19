#!/usr/bin/env python3
"""
Invariant tests for kiss_bundle.py — the properties the paste-into-an-LLM audit
bundle depends on. Runs against the real spec/. Stdlib only:

    python -m unittest tools/test_kiss_bundle.py      # from the repo root
    python tools/test_kiss_bundle.py
"""
import os
import re
import sys
import unittest

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
import kiss_bundle as kb  # noqa: E402

SPEC_DIR = os.path.join(os.path.dirname(_HERE), "spec")
NO_COV = "## Live traceability status\n(test: coverage omitted)\n"

DOC_OPEN = re.compile(r'^<document path="spec/([a-z]+)\.md".*mode="(full|outline)">$', re.M)


def doc_sequence(bundle):
    """[(stem, mode), ...] in the order documents appear in a bundle."""
    return [(m.group(1), m.group(2)) for m in DOC_OPEN.finditer(bundle)]


class BundleStructure(unittest.TestCase):
    def setUp(self):
        self.full = kb.build_full(SPEC_DIR, NO_COV)

    def test_audit_brief_present_and_closed(self):
        self.assertEqual(self.full.count("<audit-brief>"), 1)
        self.assertEqual(self.full.count("</audit-brief>"), 1)
        # the brief must teach the delimiter rule, or an auditor mis-splits files.
        self.assertIn("</document>", self.full.split("</audit-brief>")[0])

    def test_document_tags_balance(self):
        # the whole point: a file boundary is an unambiguous tag, not a heading.
        opens = self.full.count("\n<document ")
        closes = self.full.count("\n</document>")
        self.assertEqual(opens, closes)
        self.assertEqual(opens, len(kb.FULL_ORDER))

    def test_full_bundle_is_topologically_ordered(self):
        order = [s for s, _ in doc_sequence(self.full)]
        self.assertEqual(order, kb.FULL_ORDER)
        pos = {s: i for i, s in enumerate(order)}
        for stem in kb.SUITE_ORDER:
            for dep, _label in kb.DEPS[stem]:
                self.assertLess(pos[dep], pos[stem],
                                f"{dep} must precede its dependent {stem}")

    def test_every_document_is_full_in_the_full_bundle(self):
        self.assertTrue(all(mode == "full" for _, mode in doc_sequence(self.full)))


class PerDocBundle(unittest.TestCase):
    def test_dependency_closure_is_outline_then_target_is_full(self):
        bundle = kb.build_per_doc(SPEC_DIR, "contract", NO_COV)
        seq = doc_sequence(bundle)
        # contract's transitive deps, each as outline, then contract itself full.
        self.assertEqual(seq[-1], ("contract", "full"))
        dep_modes = {s: m for s, m in seq[:-1]}
        self.assertEqual(set(dep_modes), set(kb.transitive_deps("contract")))
        self.assertTrue(all(m == "outline" for m in dep_modes.values()))

    def test_target_full_text_is_verbatim(self):
        bundle = kb.build_per_doc(SPEC_DIR, "ops", NO_COV)
        source = kb.read_doc(SPEC_DIR, "ops")
        self.assertIn(source, bundle)  # the target document appears byte-for-byte


class OutlineReduction(unittest.TestCase):
    def test_outline_keeps_clause_ids_but_drops_bodies_and_matrix(self):
        source = kb.read_doc(SPEC_DIR, "ops")
        sk = kb.outline(source)
        lines = sk.splitlines()
        self.assertIn("KISS-OPS-6.0-0001", sk)          # clause ids survive
        self.assertIn("### 6.1 The op-set registry", sk)  # headings survive
        # the mid-document H1 is a heading, not a file boundary — kept as-is.
        self.assertIn("# NORMATIVE CONFORMANCE SPECIFICATION (§6+)", sk)
        # the §9 traceability matrix rows (| KISS-OPS-... | test |) are dropped.
        self.assertFalse(any(l.startswith("| KISS-OPS-") for l in lines))
        # a real reduction, not a copy.
        self.assertLess(len(sk), len(source) / 2)

    def test_outline_ignores_hashes_inside_code_fences(self):
        text = "## Real Heading\n```sh\n# not a heading\n```\n- **KISS-OPS-6.0-0001** — x\n"
        sk = kb.outline(text)
        self.assertIn("## Real Heading", sk)
        self.assertNotIn("# not a heading", sk)
        self.assertIn("KISS-OPS-6.0-0001", sk)


class Coverage(unittest.TestCase):
    def test_format_handles_error_gracefully(self):
        out = kb.format_coverage({"error": "boom"})
        self.assertIn("Live traceability status", out)
        self.assertIn("unavailable", out)

    def test_format_renders_real_numbers(self):
        out = kb.format_coverage({
            "total": 100, "backed": 25, "unbacked": 75, "lint_only": 10,
            "genuinely_untested": ["KISS-OPS-6.0-0001"], "per_sub": {"OPS": [25, 100]}})
        self.assertIn("25 of 100 normative clauses (25.0%)", out)
        self.assertIn("KISS-OPS-6.0-0001", out)

    def test_compute_coverage_never_raises(self):
        # a bogus conformance dir must degrade, not crash — concatenation is the job.
        cov = kb.compute_coverage(SPEC_DIR, os.path.join(_HERE, "no-such-dir"), _HERE)
        self.assertIsInstance(cov, dict)


class Estimates(unittest.TestCase):
    def test_token_bracket_is_ordered(self):
        lo, hi = kb.est_tokens("x" * 1000)
        self.assertLess(lo, hi)


if __name__ == "__main__":
    unittest.main(verbosity=2)
