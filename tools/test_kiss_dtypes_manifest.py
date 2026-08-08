import json, subprocess, sys, pathlib, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


def _emit():
    return subprocess.run(
        [sys.executable, str(ROOT / "tools" / "kiss_dtypes.py"), "--emit-manifest", "--stdout"],
        capture_output=True, text=True, check=True,
    ).stdout


class DtypeManifestTest(unittest.TestCase):
    def test_emit_manifest_has_all_22_dtypes_with_metadata(self):
        m = json.loads(_emit())
        self.assertEqual(m["schema"], "kiss-dtype-manifest-v1")
        # clause D (§3.4): the manifest is a persisted, indexed list of dtype sub-tokens,
        # so it MUST carry the structure_key schema version it belongs to (sourced from
        # KISS-CLASSIFY-6.4-0003). c64 changes meaning at sk4, so a version-less list
        # would be silently ambiguous once vendored.
        self.assertEqual(m["structure_key_schema_version"], 3)
        self.assertEqual(m["token_prefix"], "sk3")
        toks = set(m["all_dtypes"])
        # exactly the twenty-two §6.1-0001 tokens, including the five most commonly
        # dropped by hand-transcription (a token-deriving party shipped 17/22 without
        # them, and parsed the two reserved fnuz variants as *unknown*, which
        # §6.1-0001 forbids)
        self.assertEqual(len(toks), 22)
        for t in ("s16", "u16", "u64", "e4m3fnuz", "e5m2fnuz"):
            self.assertIn(t, toks)
        by = {d["token"]: d for d in m["dtypes"]}
        # the AMD fnuz variants are reserved (recognized on parse, distinct from unknown)
        self.assertTrue(by["e4m3fnuz"]["reserved"])
        self.assertTrue(by["e5m2fnuz"]["reserved"])
        self.assertFalse(by["e4m3fn"]["reserved"])
        # storage widths and kinds are carried
        self.assertEqual(by["f32"]["storage_bits"], 32)
        self.assertEqual(by["b1"]["storage_bits"], 1)
        self.assertEqual(by["c64"]["storage_bits"], 128)
        self.assertEqual(by["c32"]["kind"], "complex")
        self.assertEqual(set(m["kinds"]), {"float", "int", "uint", "bool", "complex"})

    def test_committed_manifest_matches_spec(self):
        """The committed dtype_manifest.json MUST equal a fresh generation — the CI
        drift gate. If this fails, run `python tools/kiss_dtypes.py --emit-manifest`."""
        committed = (ROOT / "conformance" / "corpus" / "dtype_manifest.json").read_text(encoding="utf-8")
        self.assertEqual(
            json.loads(committed), json.loads(_emit()),
            "conformance/corpus/dtype_manifest.json is stale — regenerate with "
            "`python tools/kiss_dtypes.py --emit-manifest`",
        )


if __name__ == "__main__":
    unittest.main()
