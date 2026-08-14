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


if __name__ == "__main__":
    unittest.main()
