# SPDX-License-Identifier: MIT OR Apache-2.0
"""Controls for kiss_corpus_pins: a clean corpus passes, and each seeded difference is caught."""
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_corpus_pins as kp  # noqa: E402


class CorpusPins(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.d, ignore_errors=True)
        for name, body in (("a.json", b'{"a":1}\n'), ("b.json", b'{"b":2}\n')):
            with open(os.path.join(self.d, name), "wb") as fh:
                fh.write(body)
        kp.write(self.d)

    def test_clean_corpus_passes(self):
        self.assertEqual(kp.check(self.d), [])

    def test_changed_byte_is_caught(self):  # mutant: one byte of a pinned file moves
        with open(os.path.join(self.d, "a.json"), "wb") as fh:
            fh.write(b'{"a":2}\n')
        self.assertEqual(kp.check(self.d), [("changed", "a.json")])

    def test_unpinned_file_is_caught(self):  # mutant: a new corpus file with no pin
        with open(os.path.join(self.d, "c.json"), "wb") as fh:
            fh.write(b"{}\n")
        self.assertEqual(kp.check(self.d), [("unpinned", "c.json")])

    def test_missing_file_is_caught(self):  # mutant: a pinned file is deleted
        os.remove(os.path.join(self.d, "b.json"))
        self.assertEqual(kp.check(self.d), [("missing", "b.json")])

    def test_crlf_checkout_is_not_a_difference(self):
        with open(os.path.join(self.d, "a.json"), "wb") as fh:
            fh.write(b'{"a":1}\r\n')
        self.assertEqual(kp.check(self.d), [])

    def test_malformed_pin_line_is_refused(self):
        with open(os.path.join(self.d, kp.PINS), "ab") as fh:
            fh.write(b"not-a-hash  x.json\n")
        with self.assertRaises(ValueError):
            kp.check(self.d)

    def test_live_corpus_matches_its_pins(self):
        self.assertEqual(kp.check(), [])


if __name__ == "__main__":
    unittest.main()
