#!/usr/bin/env python3
"""Controls for kiss_conflict_markers.py: it must go RED on the exact block that shipped on main
(4ac8803, conformance/README.md:55/57/59), stay green on a setext underline, and enumerate via
git ls-files (a marker in an untracked file is NOT seen)."""
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import kiss_conflict_markers as cm  # noqa: E402

LT, EQ, GT = "<" * 7, "=" * 7, ">" * 7  # built, so this file itself carries no column-0 marker


class Markers(unittest.TestCase):
    def test_shipped_block_is_red(self):
        t = f"a\n{LT} HEAD\nx 947\n{EQ}\nx 947\n{GT} origin/main\nb\n".encode()
        self.assertEqual([n for n, _ in cm.find_markers(t)], [2, 4, 6])

    def test_crlf_block_is_red(self):
        t = f"{LT} HEAD\r\nx\r\n{EQ}\r\ny\r\n{GT} o\r\n".encode()
        self.assertEqual(len(cm.find_markers(t)), 3)

    def test_setext_underline_is_green(self):
        self.assertEqual(cm.find_markers(f"Title\n{EQ}\ntext\n".encode()), [])

    def test_indented_or_inline_is_green(self):
        self.assertEqual(cm.find_markers(f"  {LT} x\nsee {GT} y\n".encode()), [])

    def test_git_enumeration_sees_tracked_not_untracked(self):
        with tempfile.TemporaryDirectory() as d:
            run = lambda *a: subprocess.run(["git", *a], cwd=d, check=True, capture_output=True)
            run("init", "-q")
            (Path(d) / "tracked.md").write_text(f"{LT} HEAD\nx\n{EQ}\ny\n{GT} o\n")
            (Path(d) / "untracked.md").write_text(f"{LT} HEAD\n")
            run("add", "tracked.md")
            r = subprocess.run([sys.executable, str(HERE / "kiss_conflict_markers.py")],
                               cwd=d, capture_output=True, text=True)
            self.assertEqual(r.returncode, 1, r.stdout)
            self.assertIn("tracked.md:1", r.stdout)
            self.assertNotIn("untracked.md", r.stdout)
            (Path(d) / "tracked.md").write_text("clean\n")
            r = subprocess.run([sys.executable, str(HERE / "kiss_conflict_markers.py")],
                               cwd=d, capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout)


if __name__ == "__main__":
    unittest.main()
