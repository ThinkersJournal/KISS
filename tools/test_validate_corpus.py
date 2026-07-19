import json, subprocess, sys, pathlib, tempfile, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "conformance" / "corpus" / "ops-arith.json"

class ValidateTest(unittest.TestCase):
    def test_frozen_bundle_validates(self):
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "validate_corpus.py"), str(BUNDLE)],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_corrupted_expected_is_rejected(self):
        data = json.loads(BUNDLE.read_text())
        # flip the (-0)+(-0) expected from -0 (80..) to +0 (00..)
        for v in data["vectors"]:
            if v["tcId"] == 2:
                v["expected"]["bits"] = "00 00 00 00"
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(data, f); tmp = f.name
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "validate_corpus.py"), tmp],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 1, "validator must reject a wrong expected value")

if __name__ == "__main__":
    unittest.main()
