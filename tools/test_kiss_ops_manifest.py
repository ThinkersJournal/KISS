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

if __name__ == "__main__":
    unittest.main()
