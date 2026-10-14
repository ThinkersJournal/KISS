# SPDX-License-Identifier: MIT OR Apache-2.0
r"""Run a `tools/` suite on Windows with a bare `bash` resolved to Git Bash, not the WSL launcher.

WHY. Three suites launch `bash` by name (`test_kiss_ratchet_step.py`, `test_kiss_msrv_step.py`,
`test_kiss_orphan_step.py`): they extract a workflow step's `run:` block and execute it. On a Windows
box with WSL installed, `CreateProcess` resolves a bare `bash` to `C:\Windows\System32\bash.exe` (the WSL
launcher) BEFORE it consults `PATH`, so putting Git's `bin` first on `PATH` does not help; the suites then
fail with `execvpe(/bin/bash) failed` and look like they found a defect. They had been reported "unrun
here (WSL)" for a stretch of 2026-10-14 for exactly this reason. They are green under Git Bash.

WHAT. This wrapper rewrites ONLY the first argv element of `subprocess.run` / `subprocess.Popen` when it is
the bare string `bash`, to Git Bash's `bash.exe`, then runs the named suite unmodified (`runpy`). It
touches no repo file and changes nothing on Linux/macOS (a no-op there: `bash` is already the right one).

Run:
    python tools/run_with_git_bash.py tools/test_kiss_ratchet_step.py
    python tools/run_with_git_bash.py tools/test_kiss_msrv_step.py
    python tools/run_with_git_bash.py tools/test_kiss_orphan_step.py

Override the interpreter with `KISS_GIT_BASH=<path to bash.exe>` if Git is not under `Program Files`.
"""
import os
import runpy
import subprocess
import sys

CANDIDATES = (
    r"C:\Program Files\Git\bin\bash.exe",
    r"C:\Program Files (x86)\Git\bin\bash.exe",
)


def git_bash():
    """Path to Git Bash, or None when this is not a Windows box with one."""
    forced = os.environ.get("KISS_GIT_BASH")
    if forced:
        return forced if os.path.exists(forced) else None
    if os.name != "nt":
        return None
    return next((c for c in CANDIDATES if os.path.exists(c)), None)


def rewrite_argv(args, bash):
    """`args` with a leading bare `bash` replaced by `bash`; anything else is returned unchanged."""
    if bash and isinstance(args, (list, tuple)) and args and args[0] == "bash":
        return [bash, *args[1:]]
    return args


def install(bash):
    """Patch `subprocess.run` and `subprocess.Popen` so a bare `bash` argv[0] means `bash`."""
    orig_run, orig_popen = subprocess.run, subprocess.Popen

    def run(args, *a, **kw):
        return orig_run(rewrite_argv(args, bash), *a, **kw)

    class Popen(orig_popen):
        def __init__(self, args, *a, **kw):
            super().__init__(rewrite_argv(args, bash), *a, **kw)

    subprocess.run, subprocess.Popen = run, Popen


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    target = argv[1]
    bash = git_bash()
    if bash is None and os.name == "nt":
        print("run_with_git_bash: no Git Bash found; set KISS_GIT_BASH to its bash.exe", file=sys.stderr)
        return 2
    install(bash)
    sys.path.insert(0, os.path.dirname(os.path.abspath(target)))
    sys.argv = [target]
    runpy.run_path(target, run_name="__main__")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
