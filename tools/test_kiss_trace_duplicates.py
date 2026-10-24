# SPDX-License-Identifier: MIT OR Apache-2.0
"""A same-named test in two files must be reported, not silently overwritten (KISS #137).

`discover_tests` keys by bare fn name. Rust allows the same name in two integration-test
binaries, so the second occurrence used to replace the first together with its citations.
Run: python tools/test_kiss_trace_duplicates.py
"""
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt

BODY = "#[test]\nfn shared_name() {\n    assert!(true);\n}\n"


def make_tree(root, files):
    conf = os.path.join(root, "conformance")
    os.makedirs(os.path.join(conf, "tests"))
    for name, src in files.items():
        with open(os.path.join(conf, "tests", name), "w", encoding="utf-8") as fh:
            fh.write(src)
    return conf


class DuplicateTestNames(unittest.TestCase):
    def test_repeat_across_files_is_reported(self):
        with tempfile.TemporaryDirectory() as root:
            conf = make_tree(root, {"a.rs": BODY, "b.rs": BODY})
            collisions = []
            kt.discover_tests(conf, collisions)
            self.assertEqual(len(collisions), 1)
            name, first, again = collisions[0]
            self.assertEqual(name, "shared_name")
            self.assertEqual({first, again}, {"conformance/tests/a.rs", "conformance/tests/b.rs"})

    def test_distinct_names_report_nothing(self):
        with tempfile.TemporaryDirectory() as root:
            conf = make_tree(root, {"a.rs": BODY, "b.rs": BODY.replace("shared_name", "other_name")})
            collisions = []
            found = kt.discover_tests(conf, collisions)
            self.assertEqual(collisions, [])
            self.assertEqual(set(found), {"shared_name", "other_name"})


if __name__ == "__main__":
    unittest.main()
