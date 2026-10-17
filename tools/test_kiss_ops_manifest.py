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

    def test_cycle_through_an_embedded_scalar_body_is_caught(self):
        # logsumexp's `reduce(sum, exp(sub(x,m)))` pre-map is an embedded body (§6.14-0006);
        # a cycle that exists ONLY through it must still be a cycle.
        v = self.violations(self.mutate("reduce(sum, exp(sub(x,m)))", "reduce(sum, logsumexp(x))"))
        self.assertTrue(any("cycle logsumexp -> logsumexp" in m for m in v), v)

    def test_complex_family_control_is_parsed_and_clean(self):
        cg = self.K.complex_decomposition_graph(self.ops)
        self.assertEqual(len(cg), 15)  # the §6.18-0001 family; a parse of nothing is also "clean"
        self.assertEqual(cg["cre"], ["element_map"])
        self.assertEqual(cg["cabs"], ["hypot"])
        self.assertEqual(cg["clog"], ["cabs", "carg", "cmake", "log"])
        v = []
        self.K.check_complex_closure(self.ops, cg, v)
        self.assertEqual(v, [])

    def complex_violations(self, ops):
        v = []
        cg = self.K.complex_decomposition_graph(ops)
        self.K.check_complex_closure(ops, cg, v)
        self.K.check_decomposition_graph({**self.K.decomposition_graph(ops), **cg}, self.prim, v)
        return v

    def test_complex_op_outside_the_closure_is_caught(self):
        # sinh is a real non-primitive op the §6.18-0002 closure does not name.
        v = self.complex_violations(self.mutate("| `cabs` | complex | ULP/tolerance | `hypot(a, b)`",
                                                "| `cabs` | complex | ULP/tolerance | `sinh(a)`"))
        self.assertTrue(any("`cabs` references `sinh`" in m for m in v), v)

    def test_complex_cycle_is_caught(self):
        v = self.complex_violations(self.mutate("| `cabs` | complex | ULP/tolerance | `hypot(a, b)`",
                                                "| `cabs` | complex | ULP/tolerance | `hypot(cabs(z), b)`"))
        self.assertTrue(any("cycle cabs -> cabs" in m for m in v), v)

    def test_complex_op_without_a_decomposition_is_caught(self):
        v = self.complex_violations(self.mutate("| `cim` | imag lane → real | `element_map` reading",
                                                "| `cim` | imag lane → real | no body, reading"))
        self.assertTrue(any("`cim` carries no reference decomposition" in m for m in v), v)

    def determinism_violations(self, ops):
        v = []
        self.K.check_complex_determinism(
            ops, {**self.K.decomposition_graph(ops), **self.K.complex_decomposition_graph(ops)},
            self.K.transcendental_atoms(ops), v)
        return v

    def test_complex_determinism_control_is_clean(self):
        self.assertEqual(self.determinism_violations(self.ops), [])

    def test_complex_determinism_table_column_drift_is_caught(self):
        v = self.determinism_violations(self.mutate("| `cmul` | complex | exact-byte |",
                                                    "| `cmul` | complex | ULP/tolerance |"))
        self.assertTrue(any("`cmul` is exact-byte by its decomposition" in m for m in v), v)

    def test_complex_determinism_follows_the_decomposition(self):
        # cadd gains a transcendental atom: its derived class flips though both copies still say exact-byte.
        v = self.determinism_violations(self.mutate("`cmake(add(a,c), add(b,d))`",
                                                    "`cmake(add(exp(a),c), add(b,d))`"))
        self.assertTrue(any("`cadd` is ULP/tolerance by its decomposition" in m for m in v), v)

    def test_complex_determinism_clause_list_drift_is_caught(self):
        v = self.determinism_violations(self.mutate("`cadd`, `csub`, `cneg`, `cconj`, `cmul`, `cdiv`) MUST be class",
                                                    "`cadd`, `csub`, `cneg`, `cconj`, `cmul`) MUST be class"))
        self.assertTrue(any("`cdiv`" in m for m in v), v)

    def carrier_violations(self, ops):
        v = []
        region = self.K.between(ops, "#### 6.19.3", "\n- **KISS-OPS-6.19-0025**")
        table = [o for cells in self.K.table_cells(region) if (o := self.K._op(cells[0]))]
        self.K.check_carrier_clauses(ops, table, self.K.decomposition_graph(ops), v)
        return v

    def test_carrier_clauses_control_is_parsed_and_clean(self):
        region = self.K.between(self.ops, "#### 6.19.3", "\n- **KISS-OPS-6.19-0025**")
        self.assertEqual(len([1 for c in self.K.table_cells(region) if self.K._op(c[0])]), 17)
        self.assertEqual(self.carrier_violations(self.ops), [])

    def test_carrier_clause_dropping_a_carrier_is_caught(self):
        v = self.carrier_violations(self.mutate("index_select, embedding, scatter_add}",
                                                "index_select, embedding}"))
        self.assertTrue(any("scatter_add" in m for m in v), v)

    def test_noncarrier_with_a_schema_row_is_caught(self):
        v = self.carrier_violations(self.mutate("`reduce_mean`, `reduce_norm2`, `logsumexp`, `argmax`",
                                                "`reduce_mean`, `reduce_norm2`, `logsumexp`, `reduce`, `argmax`"))
        self.assertTrue(any("`reduce` is named a non-carrier" in m for m in v), v)

    def test_noncarrier_without_an_inner_carrier_node_is_caught(self):
        v = self.carrier_violations(self.mutate("| `cumsum` | scan | — | `prefix_scan(monoid=sum, inclusive)` |",
                                                "| `cumsum` | scan | — | `add(x, x)` |"))
        self.assertTrue(any("`cumsum` has no reduce/prefix_scan/sort_network" in m for m in v), v)

    def fidelity_violations(self, ops):
        v = []
        self.K.check_math_fidelity(ops, v)
        return v

    def test_math_fidelity_control_is_parsed_and_clean(self):
        region = self.K.between(self.ops, "### 6.16 Self-contained dtype bit layouts",
                                "\n- **KISS-OPS-6.16-0001**")
        rows = [c[0] for c in self.K.table_cells(region) if c[0].startswith("`")]
        self.assertIn("`f32`", rows)
        self.assertEqual(len(rows), 24)  # a parse that found nothing would also be "clean"
        self.assertEqual(self.fidelity_violations(self.ops), [])

    def test_strict_precision_dtype_row_is_caught(self):
        v = self.fidelity_violations(self.mutate(
            "| `f64` | 64 | float | IEEE-754 binary64",
            "| `f32s` | 32 | float | strict binary32 |\n| `f64` | 64 | float | IEEE-754 binary64"))
        self.assertTrue(any("`f32s` row" in m for m in v), v)

    def test_fidelity_spelling_as_a_dtype_is_caught(self):
        v = self.fidelity_violations(self.mutate(
            "| `f64` | 64 | float | IEEE-754 binary64",
            "| `tf32` | 32 | float | reduced mantissa |\n| `f64` | 64 | float | IEEE-754 binary64"))
        self.assertTrue(any("`tf32` row" in m for m in v), v)

    def test_f32_row_losing_pure_storage_is_caught(self):
        v = self.fidelity_violations(self.mutate("**pure storage** — compute fidelity",
                                                 "storage — compute fidelity"))
        self.assertTrue(any("pure storage" in m for m in v), v)

    def test_tf32_bit_count_off_the_floor_is_caught(self):
        v = self.fidelity_violations(self.mutate("10 retained mantissa bits",
                                                 "11 retained mantissa bits"))
        self.assertTrue(any("floor is 10" in m for m in v), v)
        self.assertTrue(any("2^-11 but" in m or "is 2^-12" in m for m in v), v)

    def test_tf32_unit_roundoff_off_the_formula_is_caught(self):
        v = self.fidelity_violations(self.mutate("`u = 2⁻¹¹`", "`u = 2⁻¹⁰`"))
        self.assertTrue(any("2^-10 but" in m for m in v), v)

    def test_tf32_rounding_mode_change_is_caught(self):
        v = self.fidelity_violations(self.mutate("exponent\n  unchanged, round-to-nearest-even,",
                                                 "exponent\n  unchanged, round-toward-zero,"))
        self.assertTrue(any("round-to-nearest-even" in m for m in v), v)

    def index_violations(self, ops):
        v = []
        self.K.check_index_clauses(ops, v)
        return v

    def test_index_clauses_control_is_parsed_and_clean(self):
        region = self.K.between(self.ops, "#### 6.19.3", "\n- **KISS-OPS-6.19-0025**")
        rows = {self.K._op(c[0]) for c in self.K.table_cells(region) if self.K._op(c[0])}
        self.assertTrue(set(self.K.INDEX_OPS) | {"sort_network"} <= rows)  # a parse of nothing is "clean"
        self.assertEqual(self.index_violations(self.ops), [])

    def test_index_dtype_set_gaining_a_member_is_caught(self):
        v = self.index_violations(self.mutate("MUST be exactly `{u32, i32,\n  i64}`",
                                              "MUST be exactly `{u32, i32,\n  i64, u64}`"))
        self.assertTrue(any("§6.11-0009: index dtype set" in m for m in v), v)

    def test_index_dtype_enum_reordered_is_caught(self):
        v = self.index_violations(self.mutate("`1`=u32, `2`=i32, `3`=i64 |",
                                              "`1`=i32, `2`=u32, `3`=i64 |"))
        self.assertTrue(any("§6.19-0017: the index_dtype enum" in m for m in v), v)

    def test_index_op_row_losing_a_field_is_caught(self):
        v = self.index_violations(self.mutate(
            "| `embedding` | `axis`:`u8`:mandatory (§6.11-0013) —",
            "| `embedding` | `axis`:`u8`:`0` (§6.11-0013) —"))
        self.assertTrue(any("`embedding` has no mandatory `u8` `axis`" in m for m in v), v)

    def test_sort_network_gaining_an_index_wire_field_is_caught(self):
        v = self.index_violations(self.mutate(
            "`stability`:bool `u8`:`1` (fixed, §6.11-0007) |",
            "`stability`:bool `u8`:`1` (fixed, §6.11-0007) — `index_dtype`:enum `u8`:`3` |"))
        self.assertTrue(any("§6.11-0019: the `sort_network`" in m for m in v), v)

    def test_max_rank_beyond_a_u8_axis_is_caught(self):
        v = self.index_violations(self.mutate("`MAX_RANK = 8` and", "`MAX_RANK = 300` and"))
        self.assertTrue(any("MAX_RANK a `u8` axis can hold" in m for m in v), v)

    def atom_violations(self, ops):
        v = []
        self.K.check_structural_atoms(ops, v)
        return v

    def test_structural_atoms_control_is_parsed_and_clean(self):
        region = self.K.between(self.ops, "#### 6.19.3", "\n- **KISS-OPS-6.19-0025**")
        rows = {self.K._op(c[0]): c[1] for c in self.K.table_cells(region) if self.K._op(c[0])}
        self.assertIn("keepdim", rows["reduce"])  # a parse of nothing would also be "clean"
        self.assertIn("prefix_scan", rows)
        self.assertEqual(self.atom_violations(self.ops), [])

    def test_reduce_keepdim_no_longer_fixed_is_caught(self):
        v = self.atom_violations(self.mutate("`keepdim`:bool `u8`:`1` (fixed, §6.11-0008) —",
                                             "`keepdim`:bool `u8`:`0` (§6.11-0008) —"))
        self.assertTrue(any("`reduce` row no longer fixes" in m for m in v), v)

    def test_prefix_scan_gaining_keepdim_is_caught(self):
        v = self.atom_violations(self.mutate(
            "`exclusivity`:`scan_exclusivity` enum `u8`:`1` (inclusive) —",
            "`keepdim`:bool `u8`:`1` — `exclusivity`:`scan_exclusivity` enum `u8`:`1` (inclusive) —"))
        self.assertTrue(any("`prefix_scan` row" in m for m in v), v)

    def test_scatter_output_shape_rule_drift_is_caught(self):
        v = self.atom_violations(self.mutate("(`SameAs(dest)`, §6.11-0016)",
                                             "(`SameAs(updates)`, §6.11-0016)"))
        self.assertTrue(any("§6.20-0008: no longer pins" in m for m in v), v)

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
