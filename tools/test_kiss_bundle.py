#!/usr/bin/env python3
"""
Invariant tests for kiss_bundle.py — the properties the paste-into-an-LLM audit
bundle depends on, plus the derive-from-spec hardening (the suite shape is read
from umbrella §2.2 and reconciled against kiss_trace, so it cannot silently
drift). Runs against the real spec/. Stdlib only:

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

# The suite's topological order, pinned. Derived from umbrella §2.2 at run time;
# this test fails loudly if the derivation stops reproducing the intended order.
EXPECTED_ORDER = ["classify", "ops", "grammar", "contract",
                  "announce", "synth", "consume", "emit", "conform"]


def doc_sequence(bundle):
    """[(stem, mode), ...] in the order documents appear in a bundle."""
    return [(m.group(1), m.group(2)) for m in DOC_OPEN.finditer(bundle)]


class SuiteDerivation(unittest.TestCase):
    def setUp(self):
        self.suite = kb.load_suite(SPEC_DIR)

    def test_order_is_the_expected_topological_sequence(self):
        self.assertEqual(self.suite.suite_order, EXPECTED_ORDER)
        self.assertEqual(self.suite.full_order, ["umbrella"] + EXPECTED_ORDER)

    def test_known_edges_are_derived_from_umbrella(self):
        d = self.suite.deps
        self.assertEqual(set(d["grammar"]), {("classify", "STRUCTURAL"), ("ops", "STRUCTURAL")})
        self.assertEqual(set(d["announce"]), {("classify", "OPAQUE"), ("contract", "OPAQUE")})
        self.assertEqual(d["classify"], [])  # foundational: no incoming edges
        # conform depends on every other sub-standard, as tests, in suite order.
        self.assertEqual([s for s, _ in d["conform"]], EXPECTED_ORDER[:-1])
        self.assertTrue(all(lab == "TEST" for _, lab in d["conform"]))

    def test_derived_set_reconciles_with_kiss_trace(self):
        # load_suite would have raised SuiteError otherwise; assert the agreement.
        if kb.kt is not None:
            self.assertEqual(set(self.suite.suite_order), set(kb.kt.SPECS) - {"umbrella"})

    def test_roles_present_with_graph_fallback(self):
        for stem in self.suite.full_order:
            self.assertTrue(self.suite.role(stem))
        # an undeclared foundational doc still gets a sensible derived label.
        bare = kb.Suite("umbrella", ["x"], {"x": []}, {})
        self.assertEqual(bare.role("x"), "foundational")

    def test_edge_table_summarizes_conform_not_enumerates(self):
        et = self.suite.edge_table_text()
        self.assertIn("each of the other eight -> KISS-Conform  [TEST]", et)
        self.assertNotIn("KISS-Classify -> KISS-Conform", et)   # summarized, not listed
        self.assertIn("KISS-Classify -> KISS-Grammar  [STRUCTURAL]", et)


class EdgeTableParser(unittest.TestCase):
    MINIMAL = ("### 2.2 Dependency DAG\n"
               "| Dependency → Dependent | Label |\n|---|---|\n"
               "| KISS-Classify → KISS-Grammar | STRUCTURAL |\n"
               "| KISS-Ops → KISS-Grammar | STRUCTURAL |\n"
               "| each of the other two → KISS-Conform | test dependency |\n"
               "## 3. Next section\n")

    def test_parses_edges_and_synthesizes_conform(self):
        deps, nodes = kb.parse_edge_table(self.MINIMAL)
        self.assertEqual(nodes, {"classify", "ops", "grammar", "conform"})
        self.assertEqual(deps["grammar"], [("classify", "STRUCTURAL"), ("ops", "STRUCTURAL")])
        self.assertTrue(all(lab == "TEST" for _, lab in deps["conform"]))

    def test_missing_section_raises(self):
        with self.assertRaises(kb.SuiteError):
            kb.parse_edge_table("nothing resembling a DAG here\n")

    def test_unknown_label_raises(self):
        text = "### 2.2 DAG\n| KISS-Ops → KISS-Grammar | WOBBLY |\n## 3\n"
        with self.assertRaises(kb.SuiteError):
            kb.parse_edge_table(text)

    def test_synth_provision_name_maps_to_synth_stem(self):
        text = "### 2.2 DAG\n| KISS-Ops → KISS-Synth/Provision | STRUCTURAL |\n## 3\n"
        _deps, nodes = kb.parse_edge_table(text)
        self.assertIn("synth", nodes)
        self.assertNotIn("synth/provision", nodes)


class Reconcile(unittest.TestCase):
    def test_clean_when_all_three_agree(self):
        self.assertEqual(
            kb.reconcile({"a", "b"}, "umbrella", ["umbrella", "a", "b"], {"umbrella", "a", "b"}),
            [])

    def test_flags_umbrella_extra_and_trace_missing(self):
        errs = " ".join(kb.reconcile(
            {"a", "c"}, "umbrella", ["umbrella", "a", "b"], {"umbrella", "a", "c"}))
        self.assertIn("kiss_trace SPECS lacks", errs)      # c is in umbrella, not SPECS
        self.assertIn("absent from umbrella", errs)        # b is in SPECS, not umbrella

    def test_flags_ghost_doc_with_no_file(self):
        errs = kb.reconcile({"a", "b"}, "umbrella", ["umbrella", "a", "b"], {"umbrella", "a"})
        self.assertTrue(any("no spec/ file" in e for e in errs))

    def test_flags_unwired_stray_doc(self):
        errs = kb.reconcile({"a"}, "umbrella", ["umbrella", "a"], {"umbrella", "a", "stray"})
        self.assertTrue(any("not in the umbrella" in e for e in errs))


class TopoOrder(unittest.TestCase):
    def test_dependencies_precede_dependents(self):
        deps = {"a": [], "b": [("a", "X")], "c": [("b", "X")]}
        order = kb.topo_order(deps, key=lambda s: s)
        self.assertLess(order.index("a"), order.index("b"))
        self.assertLess(order.index("b"), order.index("c"))

    def test_cycle_raises(self):
        with self.assertRaises(kb.SuiteError):
            kb.topo_order({"a": [("b", "X")], "b": [("a", "X")]}, key=lambda s: s)


class BundleStructure(unittest.TestCase):
    def setUp(self):
        self.suite = kb.load_suite(SPEC_DIR)
        self.full = kb.build_full(SPEC_DIR, self.suite, NO_COV)

    def test_audit_brief_present_and_closed(self):
        self.assertEqual(self.full.count("<audit-brief>"), 1)
        self.assertEqual(self.full.count("</audit-brief>"), 1)
        # the brief must teach the delimiter rule, or an auditor mis-splits files.
        self.assertIn("</document>", self.full.split("</audit-brief>")[0])

    def test_document_tags_balance(self):
        opens = self.full.count("\n<document ")
        closes = self.full.count("\n</document>")
        self.assertEqual(opens, closes)
        self.assertEqual(opens, len(self.suite.full_order))

    def test_full_bundle_is_topologically_ordered(self):
        order = [s for s, _ in doc_sequence(self.full)]
        self.assertEqual(order, self.suite.full_order)
        pos = {s: i for i, s in enumerate(order)}
        for stem in self.suite.suite_order:
            for dep, _label in self.suite.deps[stem]:
                self.assertLess(pos[dep], pos[stem],
                                f"{dep} must precede its dependent {stem}")

    def test_every_document_is_full_in_the_full_bundle(self):
        self.assertTrue(all(mode == "full" for _, mode in doc_sequence(self.full)))


class PerDocBundle(unittest.TestCase):
    def setUp(self):
        self.suite = kb.load_suite(SPEC_DIR)

    def test_dependency_closure_is_outline_then_target_is_full(self):
        bundle = kb.build_per_doc(SPEC_DIR, self.suite, "contract", NO_COV)
        seq = doc_sequence(bundle)
        self.assertEqual(seq[-1], ("contract", "full"))
        dep_modes = {s: m for s, m in seq[:-1]}
        self.assertEqual(set(dep_modes), set(self.suite.transitive_deps("contract")))
        self.assertTrue(all(m == "outline" for m in dep_modes.values()))

    def test_target_full_text_is_verbatim(self):
        bundle = kb.build_per_doc(SPEC_DIR, self.suite, "ops", NO_COV)
        self.assertIn(kb.read_doc(SPEC_DIR, "ops"), bundle)  # byte-for-byte


class OutlineReduction(unittest.TestCase):
    def test_outline_keeps_clause_ids_but_drops_bodies_and_matrix(self):
        source = kb.read_doc(SPEC_DIR, "ops")
        sk = kb.outline(source)
        lines = sk.splitlines()
        self.assertIn("KISS-OPS-6.0-0001", sk)            # clause ids survive
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
