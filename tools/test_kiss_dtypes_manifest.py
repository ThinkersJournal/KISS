# SPDX-License-Identifier: MIT OR Apache-2.0
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


# Hand-transcribed from the KISS-Classify §6.1-0001 table (spec/classify.md, the 24 rows), NOT
# derived from the manifest: a table copied from the artifact under test could not disagree with
# it. (token, kind, storage_bits, reserved). KISS#239: width is necessary but nowhere near
# sufficient to identify a type, so every arm is pinned by identity, not spot-checked.
SPEC_ARMS = (
    ("f16", "float", 16, False), ("bf16", "float", 16, False),
    ("f32", "float", 32, False), ("f64", "float", 64, False),
    ("i8", "int", 8, False), ("i16", "int", 16, False),
    ("u8", "uint", 8, False), ("u16", "uint", 16, False),
    ("i32", "int", 32, False), ("i64", "int", 64, False),
    ("u32", "uint", 32, False), ("u64", "uint", 64, False),
    ("bool", "bool", 8, False),
    ("f8e4m3fn", "float", 8, False), ("f8e4m3fnuz", "float", 8, True),
    ("f8e5m2", "float", 8, False), ("f8e5m2fnuz", "float", 8, True),
    ("f8e8m0", "float", 8, False), ("f8e6m2", "float", 8, True),
    ("i4", "int", 4, False), ("u4", "uint", 4, False), ("b1", "uint", 1, False),
    ("c64", "complex", 64, False), ("c128", "complex", 128, False),
)

# Tokens per storage width, from the same table: the 8-bit cluster alone is nine tokens.
SPEC_WIDTH_CARDINALITY = {1: 1, 4: 2, 8: 9, 16: 4, 32: 3, 64: 4, 128: 1}


def _arm_failures(dtypes):
    """Every disagreement between a manifest's `dtypes` rows and SPEC_ARMS, as strings.
    Empty means each of the 24 arms matches on (kind, storage_bits, reserved) and the
    per-width token counts match."""
    out = []
    by = {d["token"]: d for d in dtypes}
    for token, kind, bits, reserved in SPEC_ARMS:
        d = by.get(token)
        if d is None:
            out.append(f"{token}: missing")
            continue
        got = (d["kind"], d["storage_bits"], d["reserved"])
        if got != (kind, bits, reserved):
            out.append(f"{token}: manifest {got}, spec {(kind, bits, reserved)}")
    counts = {}
    for d in dtypes:
        counts[d["storage_bits"]] = counts.get(d["storage_bits"], 0) + 1
    if counts != SPEC_WIDTH_CARDINALITY:
        out.append(f"per-width token counts {sorted(counts.items())}, spec {sorted(SPEC_WIDTH_CARDINALITY.items())}")
    return out


class DtypeManifestTest(unittest.TestCase):
    def test_emit_manifest_has_all_24_sk4_dtypes_with_metadata(self):
        m = json.loads(_emit())
        self.assertEqual(m["schema"], "kiss-dtype-manifest-v1")
        # clause D (§3.4): the manifest is a persisted, indexed list of dtype sub-tokens,
        # so it MUST carry the structure_key schema version it belongs to (sourced from
        # KISS-CLASSIFY-6.4-0003). c64 changes meaning at sk4 (pair-of-f32, was pair-of-f64),
        # so a version-less list would be silently ambiguous once vendored.
        self.assertEqual(m["structure_key_schema_version"], 6)
        self.assertEqual(m["token_prefix"], "sk6")
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

    def test_every_arm_is_pinned_by_identity_and_width_cardinality(self):
        """KISS#239 items 1 and 2: all 24 arms, not a spot-check. `(kind, storage_bits,
        reserved)` for each token, and exactly N tokens per width, against a table
        transcribed from spec/classify.md §6.1-0001."""
        self.assertEqual(len(SPEC_ARMS), 24)
        self.assertEqual(sum(SPEC_WIDTH_CARDINALITY.values()), 24)
        self.assertEqual(_arm_failures(json.loads(_emit())["dtypes"]), [])

    def test_arm_pin_detects_a_same_width_swap(self):
        """The control for the test above: a same-width swap preserves the token SET and
        every width count, so only the per-arm identity pin can see it. Each mutation is
        applied to a copy of the real manifest and MUST be reported (a pin that cannot
        fail is the defect #239 names)."""
        dtypes = json.loads(_emit())["dtypes"]
        self.assertEqual(_arm_failures(dtypes), [])  # unmutated: clean, so a hit below is the mutation
        swaps = (
            ("i16", "kind", "uint"),               # same width, kind swap
            ("f8e5m2", "storage_bits", 16),        # width moved within the FP8 cluster
            ("f8e4m3fn", "reserved", True),        # usable arm marked reserved
            ("f8e6m2", "reserved", False),         # reserved arm marked usable
            ("bf16", "kind", "int"),               # same-width 16b float turned integer
        )
        for token, field, value in swaps:
            with self.subTest(token=token, field=field):
                mutated = [dict(d) for d in dtypes]
                row = next(d for d in mutated if d["token"] == token)
                self.assertNotEqual(row[field], value)
                row[field] = value
                self.assertTrue(any(f.startswith(token + ":") for f in _arm_failures(mutated)))
        # a pure token-for-token exchange of two same-width rows' metadata
        mutated = [dict(d) for d in dtypes]
        a = next(d for d in mutated if d["token"] == "i16")
        b2 = next(d for d in mutated if d["token"] == "u16")
        a["kind"], b2["kind"] = b2["kind"], a["kind"]
        self.assertEqual(len(_arm_failures(mutated)), 2)

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
