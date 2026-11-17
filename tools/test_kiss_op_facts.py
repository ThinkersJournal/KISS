# SPDX-License-Identifier: MIT OR Apache-2.0
import json, pathlib, sys, tempfile, unittest, shutil

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import kiss_op_facts  # noqa: E402


class Result:
    def __init__(self, returncode, stdout=b"", stderr=""):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


def emit(spec_dir=None):
    """In-process (no subprocess): a SystemExit from the generator is its non-zero exit."""
    try:
        return Result(0, kiss_op_facts.render(spec_dir or ROOT / "spec"))
    except SystemExit as e:
        return Result(1, stderr=str(e))


class OpFactsTest(unittest.TestCase):
    def test_committed_file_matches_spec_byte_for_byte(self):
        committed = (ROOT / "conformance" / "corpus" / "op_facts.json").read_bytes()
        r = emit()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(committed, r.stdout,
                         "op_facts.json is stale: regenerate with `python tools/kiss_op_facts.py`")

    def test_facts_are_the_spec_sets(self):
        facts = json.loads(emit().stdout)["ops"]
        manifest = json.loads((ROOT / "conformance" / "corpus" / "op_manifest.json").read_text("utf-8"))
        self.assertEqual(sorted(facts), manifest["all_ops"])
        pick = lambda key: sorted(o for o, f in facts.items() if f[key])
        self.assertEqual(pick("is_contraction"), ["matmul"])
        self.assertEqual(pick("index_operand_role"),
                         ["embedding", "gather", "index_select", "scatter", "scatter_add"])
        self.assertEqual(pick("structured_op"),
                         ["argmax", "argmin", "avg_pool", "embedding", "im2col", "index_select",
                          "matmul", "max_pool", "scatter_add"])
        self.assertEqual(len(pick("carrier")), 31)
        self.assertEqual(facts["reduce"]["reduce_axes"], "reduce_axes")
        self.assertEqual(facts["softmax"]["reduce_axes"], "norm_axes")
        self.assertIsNone(facts["add"]["reduce_axes"])
        self.assertEqual(facts["matmul"]["opattrs"][0], "accumulator")  # encodings never leak in as names

    def test_declares_reduce_axis(self):
        """Fireline's 6.3-0006 rdc check: scans, argmax/argmin and pools declare an axis (6.19-0036)."""
        facts = json.loads(emit().stdout)["ops"]
        for o in ("cumsum", "cumprod", "cummax"):
            self.assertEqual((facts[o]["declares_reduce_axis"], facts[o]["reduce_axis_inner_carriers"]), (True, ["prefix_scan"]), o)
        for o in ("argmax", "argmin"):
            self.assertEqual(facts[o]["reduce_axis_inner_carriers"], ["sort_network"], o)
        for o in ("avg_pool", "max_pool"):
            self.assertEqual(facts[o]["reduce_axis_inner_carriers"], ["reduce"], o)
        self.assertEqual(facts["softmax"]["reduce_axis_source"], "opattrs")
        for o in ("add", "gather", "index_select", "sort_network", "cast", "cadd"):
            self.assertFalse(facts[o]["declares_reduce_axis"], o)
            self.assertIsNone(facts[o]["reduce_axis_source"], o)

    def test_reworded_anchor_or_dropped_row_fails_loudly(self):
        """A control that never applied reads like one that was not caught: assert the edit landed."""
        src = (ROOT / "spec" / "ops.md").read_text("utf-8")
        for old, new in (
            ("scatter, sort_network, reduce_var, reduce_std, softmax,", "scatter, sort_network, reduce_var, softmax,"),
            ("structured op** — `matmul`,", "structured op** — `matmu`,"),
        ):
            self.assertEqual(src.replace("\r\n", "\n").count(old), 1, old)
            with tempfile.TemporaryDirectory() as d:
                shutil.copy(ROOT / "spec" / "ops.md", pathlib.Path(d) / "ops.md")
                p = pathlib.Path(d) / "ops.md"
                p.write_text(p.read_text("utf-8").replace("\r\n", "\n").replace(old, new), "utf-8")
                r = emit(d)
                self.assertNotEqual(r.returncode, 0, f"mutation {old!r} was not caught")


if __name__ == "__main__":
    unittest.main()
