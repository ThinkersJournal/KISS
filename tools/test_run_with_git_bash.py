# SPDX-License-Identifier: MIT OR Apache-2.0
"""Controls for tools/run_with_git_bash.py: only a bare leading `bash` is rewritten, and nothing else."""
import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_with_git_bash as w  # noqa: E402

GB = r"C:\Program Files\Git\bin\bash.exe"


class RewriteArgv(unittest.TestCase):
    def test_bare_bash_is_replaced_and_the_rest_kept(self):
        self.assertEqual(w.rewrite_argv(["bash", "-e", "x.sh"], GB), [GB, "-e", "x.sh"])

    def test_a_full_path_is_left_alone(self):
        self.assertEqual(w.rewrite_argv(["/usr/bin/bash", "x"], GB), ["/usr/bin/bash", "x"])

    def test_bash_not_in_first_position_is_left_alone(self):
        self.assertEqual(w.rewrite_argv(["env", "bash", "x"], GB), ["env", "bash", "x"])

    def test_a_string_command_is_left_alone(self):
        self.assertEqual(w.rewrite_argv("bash x", GB), "bash x")

    def test_without_a_git_bash_nothing_changes(self):
        self.assertEqual(w.rewrite_argv(["bash", "x"], None), ["bash", "x"])

    def test_an_empty_argv_is_left_alone(self):
        self.assertEqual(w.rewrite_argv([], GB), [])

    def test_the_override_env_var_is_honoured_only_for_an_existing_file(self):
        old = os.environ.get("KISS_GIT_BASH")
        try:
            os.environ["KISS_GIT_BASH"] = os.path.join(HERE, "no-such-bash.exe")
            self.assertIsNone(w.git_bash())
            os.environ["KISS_GIT_BASH"] = os.path.abspath(__file__)
            self.assertEqual(w.git_bash(), os.path.abspath(__file__))
        finally:
            if old is None:
                os.environ.pop("KISS_GIT_BASH", None)
            else:
                os.environ["KISS_GIT_BASH"] = old


if __name__ == "__main__":
    unittest.main(verbosity=2)
