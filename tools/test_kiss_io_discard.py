# SPDX-License-Identifier: MIT OR Apache-2.0
"""Negative controls for kiss_io_discard (KISS #148).

A lint that has never been shown to flag the wrong form supplies no evidence, and one that flags
every form supplies none either: so each discard form must be reported AND each propagating form must
be left alone.

Run: python tools/test_kiss_io_discard.py
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_io_discard as lint


def forms(src):
    return [form for _, form in lint.scan_text(src)]


class Flags(unittest.TestCase):

    def test_let_ok_else_return_on_read_dir(self):
        self.assertEqual(forms("fn f(d: &Path) {\n    let Ok(e) = std::fs::read_dir(d) else { return; };\n}\n"), ["let Ok(..) ="])

    def test_let_ok_else_continue_on_read_to_string(self):
        self.assertEqual(forms("for p in ps {\n    let Ok(t) = std::fs::read_to_string(&p) else { continue };\n}\n"), ["let Ok(..) ="])

    def test_ok_question_on_read_dir(self):
        self.assertEqual(forms("let v: Vec<_> = std::fs::read_dir(r)\n    .ok()?\n    .collect();\n"), ["ok()"])

    def test_flatten_after_read_dir(self):
        self.assertEqual(forms("for e in std::fs::read_dir(r)?.flatten() { use_it(e); }\n"), ["flatten()"])

    def test_unwrap_or_default_on_read(self):
        self.assertEqual(forms("let t = std::fs::read_to_string(p).unwrap_or_default();\n"), ["unwrap_or_default()"])

    def test_if_let_ok_on_file_open(self):
        self.assertEqual(forms("if let Ok(f) = File::open(p) { use_it(f); }\n"), ["let Ok(..) ="])

    def test_line_number_is_the_io_call(self):
        self.assertEqual(lint.scan_text("fn f() {\n    let x = 1;\n    let t = std::fs::read(p).ok();\n}\n"), [(3, "ok()")])


class LeavesAlone(unittest.TestCase):

    def test_propagating_forms_are_clean(self):
        for src in (
            "let e = std::fs::read_dir(d)?;\n",
            "let t = std::fs::read_to_string(&p).map_err(|e| format!(\"{p:?}: {e}\"))?;\n",
            "let f = File::open(p).expect(\"fixture exists\");\n",
            "for e in std::fs::read_dir(d)? { let e = e?; use_it(e); }\n",
        ):
            with self.subTest(src=src):
                self.assertEqual(lint.scan_text(src), [])

    def test_ok_without_io_is_clean(self):
        """`.ok()` on a parse is not a filesystem read; flagging it would be the over-broad ceremony."""
        self.assertEqual(lint.scan_text("let n = s.parse::<u32>().ok()?;\nlet v = items.into_iter().flatten();\n"), [])

    def test_discard_in_a_comment_is_ignored(self):
        self.assertEqual(lint.scan_text("// std::fs::read_dir(d).ok()?\nlet x = 1;\n"), [])

    def test_the_io_call_and_the_discard_must_share_a_statement(self):
        self.assertEqual(lint.scan_text("let e = std::fs::read_dir(d)?;\nlet n = s.parse::<u32>().ok();\n"), [])


class Exemption(unittest.TestCase):
    SRC = "{marker}let v: Vec<_> = std::fs::read_dir(r)\n    .ok()?\n    .collect();\n"

    def test_marker_with_a_reason_exempts(self):
        src = self.SRC.format(marker="// io-discard-ok: absence means the toolchain is not installed\n")
        self.assertEqual(lint.scan_text(src), [])

    def test_marker_without_a_reason_does_not(self):
        self.assertEqual(forms(self.SRC.format(marker="// io-discard-ok:\n")), ["ok()"])

    def test_marker_too_far_above_does_not(self):
        src = "// io-discard-ok: a reason\n\n\n\n" + self.SRC.format(marker="")
        self.assertEqual(forms(src), ["ok()"])

    def test_no_marker_does_not(self):
        self.assertEqual(forms(self.SRC.format(marker="")), ["ok()"])


if __name__ == "__main__":
    unittest.main()
