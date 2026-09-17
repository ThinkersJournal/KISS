# SPDX-License-Identifier: MIT OR Apache-2.0
r"""Controls for the licence-split gate (#508).

⚠️ BORN-RED IS THE POINT. After the sweep the real tree is clean, so without a fixture that forces a
red, this gate ships as a check nobody has seen fail — and "never fired" is indistinguishable from
"cannot fire". Each fixture is a throwaway git repo, because the gate enumerates with `git ls-files`
and a plain directory would test a population the gate never reads.

`unittest.TestCase`, not bare `assert`: Bandit B101 flags bare asserts and Codacy gates on new
issues (#491); 15 of the tree's tool tests already use `self.assertX`.
"""
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_spdx as ks  # noqa: E402

HDR = "// SPDX-License-Identifier: MIT OR Apache-2.0"


def repo(files, reuse=None):
    """A throwaway git repo holding `files` ({path: text}), all tracked."""
    root = tempfile.mkdtemp(prefix="spdx_")
    for path, text in files.items():
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full) or root, exist_ok=True)
        with open(full, "w", encoding="utf-8", newline="") as fh:
            fh.write(text)
    if reuse is not None:
        with open(os.path.join(root, "REUSE.toml"), "w", encoding="utf-8") as fh:
            fh.write(reuse)
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "-A"], cwd=root, check=True)
    return root


class Gate(unittest.TestCase):
    def check(self, root):
        return ks.run(root, stamp=False)

    def tearDown(self):
        for r in getattr(self, "_roots", []):
            shutil.rmtree(r, ignore_errors=True)

    def mk(self, files, reuse=None):
        r = repo(files, reuse)
        self._roots = getattr(self, "_roots", []) + [r]
        return r

    # ---- BORN-RED ------------------------------------------------------------------------

    def test_born_red_an_unstamped_code_file_fails_the_gate(self):
        r = self.mk({"src/a.rs": "fn main() {}\n"})
        self.assertEqual(self.check(r), 1, "an unstamped .rs must fail the gate")

    def test_a_stamped_code_file_passes(self):
        r = self.mk({"src/a.rs": HDR + "\nfn main() {}\n"})
        self.assertEqual(self.check(r), 0)

    # ---- the population is the control ----------------------------------------------------

    def test_zero_code_files_is_could_not_measure_not_clean(self):
        """⚠️ `0 missing` over `0 files` is not a pass — a broken enumeration looks exactly like it."""
        r = self.mk({"README.md": "# text only\n"})
        self.assertEqual(self.check(r), 2)

    def test_text_files_are_never_required_to_carry_a_header(self):
        r = self.mk({"spec/x.md": "# spec\n", "c/v.json": "{}\n", "src/a.rs": HDR + "\n"})
        self.assertEqual(self.check(r), 0, "text/data are CC0 via REUSE.toml, never stamped in-file")

    # ---- the REUSE exemption must be exactly as wide as intended ---------------------------

    def test_a_reuse_annotated_code_file_is_exempt(self):
        reuse = ('# SPDX-License-Identifier: MIT OR Apache-2.0\nversion = 1\n[[annotations]]\npath = ["vendor/gen.cu"]\n'
                 'SPDX-FileCopyrightText = "x"\nSPDX-License-Identifier = "MIT OR Apache-2.0"\n')
        r = self.mk({"vendor/gen.cu": "// verbatim, do not edit\n", "src/a.rs": HDR + "\n"}, reuse)
        self.assertEqual(self.check(r), 0, "a verbatim third-party file must not be forced to change")

    def test_a_cc0_glob_does_NOT_exempt_code(self):
        """⚠️ The CC0 text annotation uses broad globs. If `**` there exempted code, the in-file rule
        would silently apply to nothing and the gate would stay green forever."""
        reuse = ('# SPDX-License-Identifier: MIT OR Apache-2.0\nversion = 1\n[[annotations]]\npath = ["**"]\n'
                 'SPDX-FileCopyrightText = "x"\nSPDX-License-Identifier = "CC0-1.0"\n')
        r = self.mk({"src/a.rs": "fn main() {}\n"}, reuse)
        self.assertEqual(self.check(r), 1, "a CC0 glob must never exempt a code file")

    # ---- placement -------------------------------------------------------------------------

    def test_a_rust_inner_attribute_is_not_mistaken_for_a_shebang(self):
        out = ks.stamp_text("#![cfg(windows)]\nfn f() {}\n", "//")
        self.assertTrue(out.startswith(HDR), f"header must precede #![...]; got {out[:60]!r}")

    def test_a_shebang_stays_first(self):
        out = ks.stamp_text("#!/usr/bin/env python3\nx = 1\n", "#")
        self.assertEqual(out.splitlines()[0], "#!/usr/bin/env python3")

    def test_crlf_is_preserved(self):
        out = ks.stamp_text("fn a() {}\r\nfn b() {}\r\n", "//")
        self.assertNotIn("\n\n", out.replace("\r\n", "~"), "stamping must not mix line endings")
        self.assertTrue(out.startswith(HDR + "\r\n"))


if __name__ == "__main__":
    r = unittest.main(exit=False, verbosity=0).result
    print(f"ok - {r.testsRun} controls pass" if r.wasSuccessful() else "FAILED")
    raise SystemExit(0 if r.wasSuccessful() else 1)
