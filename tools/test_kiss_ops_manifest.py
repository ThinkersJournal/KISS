# SPDX-License-Identifier: MIT OR Apache-2.0
import json, subprocess, sys, pathlib, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

class ManifestTest(unittest.TestCase):
    def test_emit_manifest_has_atoms_and_declared_set(self):
        out = subprocess.run(
            [sys.executable, str(ROOT / "tools" / "kiss_ops.py"), "--emit-manifest", "--stdout"],
            capture_output=True, text=True, check=True,
        ).stdout
        m = json.loads(out)
        self.assertEqual(m["schema"], "kiss-op-manifest-v1")
        # exp/log/sin are transcendental atoms per ops.md §2.7/§6.8
        for atom in ("exp", "log", "sin"):
            self.assertIn(atom, m["transcendental_atoms"])
        # add is a primitive-floor op and is this slice's declared coverage
        self.assertIn("add", m["all_ops"])
        self.assertIn("add", m["declared_coverage_set"])

    def test_registry_lists_every_in_scope_op(self):
        """§6.1-0001: the op set is primitive ∪ non-primitive ∪ complex. The manifest is the
        registry, so it MUST list the §6.18 complex ops (zero implementations is no excuse) and
        every op the §6.13 table defines, argmin included."""
        out = subprocess.run(
            [sys.executable, str(ROOT / "tools" / "kiss_ops.py"), "--emit-manifest", "--stdout"],
            capture_output=True, text=True, check=True,
        ).stdout
        ops = set(json.loads(out)["all_ops"])
        complex_ops = {"cadd", "csub", "cneg", "cconj", "cmul", "cdiv", "cabs", "carg",
                       "cexp", "clog", "csqrt", "cpow", "cmake", "cre", "cim"}
        self.assertEqual(len(complex_ops), 15)
        self.assertEqual(complex_ops - ops, set(), "complex ops missing from the registry")
        self.assertIn("argmin", ops)
        self.assertIn("argmax", ops)
        # discrimination: a token that is not an op must not be listed
        self.assertNotIn("cmplx", ops)


    def test_committed_manifest_matches_spec_byte_for_byte(self):
        """The committed op_manifest.json MUST equal a fresh generation BYTE FOR BYTE.

        NEW in #162: this artifact had NO freshness gate of any kind — not parsed,
        not byte. It could drift from the spec silently, and the only thing reading
        it (`conformance/tests/corpus_coverage.rs`) consumes its CONTENT, so a stale
        manifest would have produced a confidently wrong coverage answer rather than
        a failure.

        Bytes, in binary: a parsed comparison is blind to line endings, key order and
        whitespace — exactly the differences a consumer byte-hashing the file hits,
        and exactly the gap #162 was filed about.
        """
        committed = (ROOT / "conformance" / "corpus" / "op_manifest.json").read_bytes()
        fresh = subprocess.run(
            [sys.executable, str(ROOT / "tools" / "kiss_ops.py"), "--emit-manifest", "--stdout"],
            capture_output=True, check=True,
        ).stdout
        self.assertEqual(
            committed, fresh,
            "conformance/corpus/op_manifest.json is stale or has drifted in bytes — "
            "regenerate with `python tools/kiss_ops.py --emit-manifest`. A consumer "
            "byte-hashing this file sees what this test sees.",
        )


class DecompositionGraphTest(unittest.TestCase):
    """§6.3-0003/-0004, §6.14-0002/-0003/-0005 over the §6.13 table. Each mutation edits the
    REAL spec text, so a lint that parsed nothing could not catch it; the control runs first."""

    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, str(ROOT / "tools"))
        import kiss_ops
        cls.K = kiss_ops
        cls.ops = (ROOT / "spec" / "ops.md").read_text(encoding="utf-8").replace("\r\n", "\n")
        cls.prim = set(kiss_ops.sec27_primitive(cls.ops))

    def violations(self, ops):
        v = []
        self.K.check_decomposition_graph(self.K.decomposition_graph(ops), self.prim, v)
        return v

    def mutate(self, old, new):
        self.assertEqual(self.ops.count(old), 1, "mutation anchor must occur exactly once")
        return self.ops.replace(old, new)

    def test_control_real_spec_is_clean_and_parsed(self):
        graph = self.K.decomposition_graph(self.ops)
        self.assertEqual(len(graph), 64)  # a parse that found nothing would also be "clean"
        self.assertEqual(graph["sqr"], ["mul"])
        self.assertEqual(graph["reduce_std"], ["reduce_var", "sqrt"])
        self.assertEqual(self.violations(self.ops), [])

    def test_self_reference_is_a_cycle(self):
        v = self.violations(self.mutate("| `sqr` | arithmetic | — | `mul(x, x)` |",
                                        "| `sqr` | arithmetic | — | `mul(sqr(x), x)` |"))
        self.assertTrue(any("cycle sqr -> sqr" in m for m in v), v)

    def test_transitive_cycle_is_caught(self):
        v = self.violations(self.mutate("`div(reduce(sum, x), reduced_count)`",
                                        "`div(reduce(sum, reduce_var(x)), reduced_count)`"))
        self.assertTrue(any("cycle reduce_mean -> reduce_var -> reduce_mean" in m for m in v), v)

    def test_unresolved_reference_is_caught(self):
        v = self.violations(self.mutate("`mul(x, x)` |", "`frobnicate(x, x)` |"))
        self.assertTrue(any("frobnicate" in m for m in v), v)

    def test_floor_op_with_a_row_is_caught(self):
        row = "| `add` | arithmetic | — | `sub(x, neg(y))` |\n"
        anchor = "| `sqr` | arithmetic | — | `mul(x, x)` |\n"
        v = self.violations(self.mutate(anchor, anchor + row))
        self.assertTrue(any("§6.3-0003" in m and "`add`" in m for m in v), v)


if __name__ == "__main__":
    unittest.main()
