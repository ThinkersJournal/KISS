# SPDX-License-Identifier: MIT OR Apache-2.0
"""A test that cites no live clause is an orphan and fails the build (KISS-CONFORM-6.1-0003, KISS#136).

The one exception is a test of the suite's own machinery, tagged `// harness-internal: <reason>`.
Controls: an untagged uncited test is an orphan; a tagged one is not; a tag with no reason, or on a
test that cites or is named by a clause, is reported; a cited test is not an orphan.
Run: python tools/test_kiss_trace_orphans.py
"""
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt


def find(src, named=()):
    with tempfile.TemporaryDirectory() as root:
        conf = os.path.join(root, "conformance")
        os.makedirs(os.path.join(conf, "tests"))
        with open(os.path.join(conf, "tests", "t.rs"), "w", encoding="utf-8") as fh:
            fh.write(src)
        return kt.find_orphans(kt.discover_tests(conf), set(named))


class Orphans(unittest.TestCase):
    def test_untagged_uncited_test_is_an_orphan(self):
        orphans, bad = find("#[test]\nfn lone() {\n    assert!(true);\n}\n")
        self.assertEqual((orphans, bad), (["lone"], []))

    def test_tagged_test_is_not_an_orphan(self):
        orphans, bad = find("// harness-internal: unit test of the JSON parser\n#[test]\nfn lone() {}\n")
        self.assertEqual((orphans, bad), ([], []))

    def test_tag_above_a_doc_comment_and_attributes_still_counts(self):
        src = "/// docs\n// harness-internal: parser unit test\n#[test]\n#[ignore]\nfn lone() {}\n"
        self.assertEqual(find(src), ([], []))

    def test_tag_with_no_reason_is_reported_not_honoured(self):
        orphans, bad = find("// harness-internal:\n#[test]\nfn lone() {}\n")
        self.assertEqual(orphans, [])
        self.assertEqual([t for t, _ in bad], ["lone"])

    def test_tag_on_a_test_that_cites_a_clause_is_reported(self):
        src = "// harness-internal: parser unit test\n// Backs: KISS-OPS-6.1-0001 — x\n#[test]\nfn both() {}\n"
        orphans, bad = find(src)
        self.assertEqual(orphans, [])
        self.assertEqual([t for t, _ in bad], ["both"])

    def test_tag_on_a_test_a_clause_row_names_is_reported(self):
        src = "// harness-internal: parser unit test\n#[test]\nfn named_one() {}\n"
        orphans, bad = find(src, named={"named_one"})
        self.assertEqual(orphans, [])
        self.assertEqual([t for t, _ in bad], ["named_one"])

    def test_cited_test_is_not_an_orphan(self):
        src = "// Backs: KISS-OPS-6.1-0001 — x\n#[test]\nfn cited() {}\n"
        self.assertEqual(find(src), ([], []))

    def test_named_by_a_row_is_not_an_orphan(self):
        self.assertEqual(find("#[test]\nfn named_one() {}\n", named={"named_one"}), ([], []))

    def test_a_tag_two_comment_blocks_up_does_not_count(self):
        src = "// harness-internal: stray\n\n#[test]\nfn lone() {}\n"
        self.assertEqual(find(src)[0], ["lone"])


if __name__ == "__main__":
    unittest.main()
