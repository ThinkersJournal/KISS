import json, subprocess, sys, pathlib, unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


def _emit_bytes():
    """The generator's RAW output. `text=True` decodes and newline-translates, which is
    the blindness the byte gate exists to remove (#162)."""
    return subprocess.run(
        [sys.executable, str(ROOT / "tools" / "kiss_dtypes.py"), "--emit-manifest", "--stdout"],
        capture_output=True, check=True,
    ).stdout


def _emit():
    return subprocess.run(
        [sys.executable, str(ROOT / "tools" / "kiss_dtypes.py"), "--emit-manifest", "--stdout"],
        capture_output=True, text=True, check=True,
    ).stdout


class DtypeManifestTest(unittest.TestCase):
    def test_emit_manifest_has_all_24_sk4_dtypes_with_metadata(self):
        m = json.loads(_emit())
        self.assertEqual(m["schema"], "kiss-dtype-manifest-v1")
        # clause D (§3.4): the manifest is a persisted, indexed list of dtype sub-tokens,
        # so it MUST carry the structure_key schema version it belongs to (sourced from
        # KISS-CLASSIFY-6.4-0003). c64 changes meaning at sk4 (pair-of-f32, was pair-of-f64),
        # so a version-less list would be silently ambiguous once vendored.
        self.assertEqual(m["structure_key_schema_version"], 4)
        self.assertEqual(m["token_prefix"], "sk4")
        toks = set(m["all_dtypes"])
        # exactly the twenty-four sk4 §6.1-0001 tokens: renamed integers/FP8, the
        # total-width complex flip, and the two additive MX scales
        self.assertEqual(len(toks), 24)
        for t in ("i8", "i16", "i4", "f8e4m3fn", "f8e4m3fnuz", "f8e5m2", "f8e5m2fnuz",
                  "f8e8m0", "f8e6m2", "c64", "c128"):
            self.assertIn(t, toks)
        # the pre-sk4 spellings are renamed away, not retained — including the OLD
        # reserved FP8 spellings (`e4m3fnuz`/`e5m2fnuz` → `f8e4m3fnuz`/`f8e5m2fnuz`),
        # without which the assertion could not fail on a deprecated token surviving.
        for gone in ("s8", "s16", "s4", "e4m3fn", "e4m3fnuz", "e5m2", "e5m2fnuz", "c32"):
            self.assertNotIn(gone, toks)
        by = {d["token"]: d for d in m["dtypes"]}
        # the AMD fnuz variants are reserved (recognized on parse, distinct from unknown)
        self.assertTrue(by["f8e4m3fnuz"]["reserved"])
        self.assertTrue(by["f8e5m2fnuz"]["reserved"])
        self.assertFalse(by["f8e4m3fn"]["reserved"])
        # storage widths and kinds — note the sk4 complex total-width naming
        self.assertEqual(by["f32"]["storage_bits"], 32)
        self.assertEqual(by["b1"]["storage_bits"], 1)
        self.assertEqual(by["c64"]["storage_bits"], 64)    # sk4: pair-of-f32
        self.assertEqual(by["c128"]["storage_bits"], 128)  # sk4: pair-of-f64
        self.assertEqual(by["c64"]["kind"], "complex")
        self.assertEqual(by["f8e8m0"]["kind"], "float")    # unsigned MX scale, float kind
        self.assertEqual(set(m["kinds"]), {"float", "int", "uint", "bool", "complex"})

    def test_committed_manifest_matches_spec_byte_for_byte(self):
        """The committed dtype_manifest.json MUST equal a fresh generation BYTE FOR
        BYTE — not merely parse-equal.

        This compared `json.loads(committed) == json.loads(emit)` until #162. A parsed
        comparison checks a PROJECTION of the artifact, and is blind to exactly the
        class of difference a consumer hits on the artifact ITSELF: line endings, key
        order, whitespace, trailing newline. The gate could be green while every
        consumer byte-hashing the same file computed a different digest — the test
        surface and the consumer surface were different surfaces.

        Bytes, in binary, with no decoding: `read_text` would re-introduce on any
        newline-translating platform the exact blindness this replaces.
        """
        committed = (ROOT / "conformance" / "corpus" / "dtype_manifest.json").read_bytes()
        self.assertEqual(
            committed, _emit_bytes(),
            "conformance/corpus/dtype_manifest.json is stale or has drifted in bytes "
            "(line endings, ordering, whitespace) — regenerate with "
            "`python tools/kiss_dtypes.py --emit-manifest`. A consumer byte-hashing "
            "this file sees what this test sees.",
        )


if __name__ == "__main__":
    unittest.main()
