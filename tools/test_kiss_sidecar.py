# SPDX-License-Identifier: MIT OR Apache-2.0
"""Discrimination controls for the clause-sidecar drift check (#144, KISS-CONFORM-6.2-0004).

A drift check that is green on a tree that cannot drift has never been shown to fire, so every
drift case below is SEEDED against a copy of the real sidecar and the real one is checked clean.

Run: python tools/test_kiss_sidecar.py
"""
import json
import os
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_sidecar as s  # noqa: E402

STEM = "consume"


class SidecarDrift(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        _, self.doc = s.load(STEM)

    def seeded(self, doc, raw=None):
        path = pathlib.Path(self.tmp.name) / f"{STEM}.json"
        path.write_text(raw if raw is not None else s.dumps(doc), encoding="utf-8", newline="")
        return mock.patch.object(s, "SIDECAR", self.tmp.name)

    def test_the_real_sidecar_is_clean(self):
        """POSITIVE CONTROL: the real sidecar has no drift, so a non-empty result below is the seed."""
        self.assertEqual(s.drift(STEM), [])
        self.assertGreater(len(self.doc["clauses"]), 40)

    def test_a_clause_missing_from_the_sidecar_is_drift(self):
        doc = json.loads(s.dumps(self.doc))
        gone = doc["clauses"].pop(3)["clause_id"]
        with self.seeded(doc):
            self.assertTrue(any(gone in p and "not in the sidecar" in p for p in s.drift(STEM)))

    def test_a_clause_only_in_the_sidecar_is_drift(self):
        doc = json.loads(s.dumps(self.doc))
        extra = dict(doc["clauses"][0], clause_id="KISS-CONSUME-6.0-9999")
        doc["clauses"].append(extra)
        doc["clauses"].sort(key=lambda c: c["clause_id"])
        with self.seeded(doc):
            self.assertTrue(any("KISS-CONSUME-6.0-9999" in p and "not in the prose" in p for p in s.drift(STEM)))

    def test_a_keyword_change_is_drift(self):
        doc = json.loads(s.dumps(self.doc))
        doc["clauses"][0]["keyword"] = "MAY"
        with self.seeded(doc):
            self.assertTrue(any("keyword" in p for p in s.drift(STEM)))

    def test_non_deterministic_serialization_is_drift(self):
        with self.seeded(self.doc, raw=json.dumps(self.doc, indent=2) + "\n"):
            self.assertTrue(any("byte-deterministic" in p for p in s.drift(STEM)))

    def test_crlf_serialization_is_drift(self):
        with self.seeded(self.doc, raw=s.dumps(self.doc).replace("\n", "\r\n")):
            self.assertTrue(any("byte-deterministic" in p for p in s.drift(STEM)))

    def test_write_over_an_existing_sidecar_is_refused(self):
        """Regenerating from prose is #144 option B and inverts 6.1-0007."""
        self.assertEqual(s.main(["--write", STEM]), 2)

    def test_unset_fields_are_counted_not_hidden(self):
        self.assertEqual(s.unset_count(self.doc), 1 + len(self.doc["clauses"]))


if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("refusing to run under -O: assert-style controls would verify nothing")
    unittest.main()
