# SPDX-License-Identifier: MIT OR Apache-2.0
import json, pathlib, sys, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import kiss_op_decomp  # noqa: E402


class OpDecompTest(unittest.TestCase):
    def test_committed_file_matches_spec_byte_for_byte(self):
        committed = (ROOT / "conformance" / "corpus" / "op_decomposition.json").read_bytes()
        self.assertEqual(committed, kiss_op_decomp.render(ROOT / "spec"),
                         "op_decomposition.json is stale: regenerate with `python tools/kiss_op_decomp.py`")

    def test_every_manifest_op_resolves_to_the_floor(self):
        d = json.loads(kiss_op_decomp.render(ROOT / "spec"))["ops"]
        manifest = json.loads((ROOT / "conformance" / "corpus" / "op_manifest.json").read_text("utf-8"))
        self.assertEqual(sorted(d), manifest["all_ops"])
        for o, e in d.items():
            self.assertTrue(e["floor"], o)
            for f in e["floor"]:
                self.assertEqual(d[f]["kind"], "primitive", (o, f))
            for r in e["direct"]:  # strictly decreasing level: the termination guarantee
                self.assertLess(d[r]["level"], e["level"], (o, r))
            self.assertEqual(e["level"] == 0, e["kind"] == "primitive", o)

    def test_outcomes_have_every_kind(self):
        kinds = {o["outcome"] for o in json.loads(kiss_op_decomp.render(ROOT / "spec"))["outcomes"]}
        self.assertEqual(kinds, {"resolved", "deferred", "decline"})


if __name__ == "__main__":
    unittest.main()
