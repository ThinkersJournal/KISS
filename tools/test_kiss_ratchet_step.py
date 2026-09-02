"""Behavioural controls for the ratchet's CI step (#338).

The step is SHELL INSIDE YAML, which is the least-tested code in the repository: nothing
imports it, no linter reads it, and its only execution is on a runner where a mistake is
observed as a confusing red rather than as a failure. Both defects #338 records survived in
plain sight for exactly that reason.

    A DECLINE IS INDISTINGUISHABLE FROM A FAILURE AT EVERY SURFACE ABOVE THE LOG.

Actions renders exit 1 and exit 2 identically as `failure`, so the step's own message is the
only thing carrying the distinction -- and the old message GUESSED, naming as "the likelier
cause HERE" something the incident that found this had already ruled out. A reader who trusts
it spends their attention on a branch the tool had eliminated one line above.

WHAT THESE CONTROLS DO, and why it is not a grep: they EXTRACT the `run:` block from the
workflow and EXECUTE it under `bash -e`, with `git` and `python` replaced by shell functions
whose exit codes each case chooses. So what is asserted is the STEP'S BEHAVIOUR -- its exit
code and its annotation -- and not the presence of words in a YAML file.

    Shell functions, not PATH stubs, deliberately: a function shadows a command identically
    on a runner and on Git Bash, with no chmod, no PATH separator conversion, and no cygpath.

THE ONE TRANSFORMATION, stated because it makes this a test of a slightly smaller artefact
than CI runs: `${{ github.base_ref }}` cannot survive outside Actions, so it is substituted.
`test_the_only_expression_is_the_one_we_substitute` asserts the expression SET, so a new
`${{ ... }}` added later fails loudly here instead of silently evaluating to empty -- which
is the same defect this file exists to catch, one level up.

Run: python tools/test_kiss_ratchet_step.py
"""
import os
import pathlib
import re
import subprocess
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
WF = HERE.parent / ".github" / "workflows" / "traceability.yml"
STEP = "Coverage ratchet (blocking)"
RE_EXPR = re.compile(r"\$\{\{\s*([^}]+?)\s*\}\}")

# `git` and `python` become functions, so the block runs without touching the repo or the
# network. $LOG records every call, which is how "python was never reached" is PROVEN rather
# than inferred from an exit code that several paths could produce.
PRELUDE = """
git()    { echo "git $*" >> "$LOG"; return ${STUB_GIT_RC:-0}; }
python() { echo "python $*" >> "$LOG"; return ${STUB_PY_RC:-0}; }
"""


def run_block(text=None, step=STEP):
    """The step's `run:` script, dedented. Raises if the step or its block is missing."""
    lines = (text if text is not None else WF.read_text(encoding="utf-8")).splitlines()
    i = next((n for n, l in enumerate(lines) if l.strip() == "- name: " + step), None)
    if i is None:
        raise AssertionError("step %r not found in %s" % (step, WF))
    j = next((n for n in range(i + 1, len(lines)) if lines[n].strip().startswith("run:")), None)
    if j is None or not lines[j].strip().endswith("|"):
        raise AssertionError("step %r has no block `run: |`" % step)
    indent = len(lines[j]) - len(lines[j].lstrip())
    body = []
    for l in lines[j + 1:]:
        if l.strip() and (len(l) - len(l.lstrip())) <= indent:
            break
        body.append(l)
    # The block's own indent, MEASURED from its first non-empty line rather than assumed to
    # be `indent + 2` (#358 review). A hardcoded step mis-dedents under any other valid YAML
    # indentation -- and it does so SILENTLY, into a script that still parses, because
    # leading whitespace is insignificant to bash. The controls would then all pass against
    # something the runner never executes.
    first = next((l for l in body if l.strip()), None)
    if first is None:
        raise AssertionError("step %r has an EMPTY `run:` block" % step)
    off = len(first) - len(first.lstrip())
    return "\n".join(l[off:] if len(l) > off else "" for l in body)


def execute(base_ref="main", git_rc=0, py_rc=0):
    """Run the real block under `bash -e`; returns (exit code, output, [calls])."""
    script = RE_EXPR.sub(lambda m: base_ref, run_block())
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td)
        log = d / "calls.log"
        log.write_text("", encoding="utf-8")
        sh = d / "step.sh"
        sh.write_text('LOG="%s"\n' % log.as_posix() + PRELUDE + script + "\n", encoding="utf-8")
        # The HOST's environment, with only the stub knobs added (#358 review). The first
        # version pinned `PATH=/usr/bin:/bin`, which bought nothing -- `git` and `python` are
        # shadowed by shell FUNCTIONS, not by PATH order -- while adding a way to fail on any
        # host whose bash or coreutils live elsewhere.
        #
        # `bash` is deliberately NOT hardcoded to `/bin/bash`: that path does not exist as
        # written on Windows, where this suite is also developed and run.
        env = dict(os.environ)
        env.update({"STUB_GIT_RC": str(git_rc), "STUB_PY_RC": str(py_rc),
                    "LOG": log.as_posix()})
        # The runner's own invocation, reproduced: `-e` is what makes `|| rc=$?` load-bearing.
        # `cwd` is pinned to the repo root. It is not load-bearing TODAY -- verified by
        # running this suite from `tools/` and from `C:/`, both green, because the stubs are
        # functions so the block's `python tools/kiss_trace.py` is only ever recorded as text
        # and never resolved. It becomes load-bearing the moment a stub becomes a real
        # process, which is the cheap kind of trap to remove before it is set.
        r = subprocess.run(["bash", "--noprofile", "--norc", "-e", "-o", "pipefail",
                            sh.as_posix()],
                           capture_output=True, text=True, timeout=120,
                           cwd=str(WF.parent.parent), env=env)
        calls = [l for l in log.read_text(encoding="utf-8").splitlines() if l.strip()]
    return r.returncode, r.stdout + r.stderr, calls


class RatchetStepTest(unittest.TestCase):
    def test_the_only_expression_is_the_one_we_substitute(self):
        """Guards the harness itself. A new `${{ ... }}` would otherwise become empty here
        and be tested as something CI never runs."""
        self.assertEqual(sorted(set(RE_EXPR.findall(run_block()))), ["github.base_ref"])

    def test_clean_exits_zero(self):
        """Control. Without it a block that exited non-zero unconditionally would satisfy
        every failing-case assertion below."""
        rc, out, calls = execute(py_rc=0)
        self.assertEqual(rc, 0, out)
        self.assertIn("CLEAN", out)
        self.assertTrue(any("kiss_trace.py --ratchet" in c for c in calls), calls)

    def test_a_floor_violation_exits_one_and_says_so(self):
        rc, out, _ = execute(py_rc=1)
        self.assertEqual(rc, 1, out)
        self.assertIn("FLOOR VIOLATION", out)

    def test_a_DECLINE_exits_two_and_is_NOT_reported_as_a_violation(self):
        """The distinction the whole issue is about. Exit 2 must stay non-zero -- a decline
        is UNCHECKED, not GREEN, and making it exit 0 would convert an uncharacterised
        dimension into a verified-looking one."""
        rc, out, _ = execute(py_rc=2)
        self.assertEqual(rc, 2, "a decline must stay distinct from CLEAN and from 1:\n" + out)
        self.assertIn("DECLINED", out)
        self.assertIn("NOT a floor violation", out)

    def test_the_decline_message_does_not_GUESS_a_cause(self):
        """#338's first defect, as a behavioural assertion.

        The old text named an unchecked fetch as "the likelier cause HERE" -- and in the
        incident that produced this issue it was not the cause at all: `main` had moved
        ahead, a third case the line did not list. The step cannot observe which cause
        occurred; the tool prints it one line above. A defence carrying a REASON where a
        MEASUREMENT was already available (convention 17).
        """
        _rc, out, _ = execute(py_rc=2)
        for guess in ("likelier", "likely cause", "most likely"):
            self.assertNotIn(guess, out.lower(),
                             "the decline message is guessing a cause again: " + out)
        self.assertTrue(re.search(r"prints the specific cause|read it", out, re.I),
                        "the message must point at the tool's own output: " + out)

    def test_a_FAILED_FETCH_is_its_own_result_and_never_reaches_the_ratchet(self):
        """#338's second defect, and the assertion that proves the fix rather than describing it.

        An unchecked fetch failed SILENTLY and resurfaced as an exit-2 decline several lines
        later, where it was indistinguishable from the ancestor case. The proof is not the
        exit code -- it is that `python` IS NEVER CALLED, so the run cannot be misread as a
        ratchet that ran and declined.
        """
        rc, out, calls = execute(git_rc=1)
        self.assertEqual(rc, 3, "a failed fetch must be its own code, not 1 or 2:\n" + out)
        self.assertIn("fetch FAILED", out)
        self.assertEqual([c for c in calls if c.startswith("python")], [],
                         "the ratchet RAN after the base fetch failed - its result is "
                         "meaningless and will be read as a decline: %r" % (calls,))

    def test_the_push_path_skips_the_fetch_and_uses_HEAD_parent(self):
        """The `else` arm. A base is passed on BOTH events deliberately (#213): a
        push-to-main run that cannot see a downgrade is the run nobody re-checks."""
        rc, out, calls = execute(base_ref="", py_rc=0)
        self.assertEqual(rc, 0, out)
        self.assertEqual([c for c in calls if c.startswith("git")], [],
                         "the push path must not fetch: %r" % (calls,))
        self.assertTrue(any("--base-ref HEAD^" in c for c in calls),
                        "the push path lost its base ref - the ratchet would DECLINE on "
                        "every push to main: %r" % (calls,))

    def test_the_harness_can_fail(self):
        """The extractor must raise on a missing step rather than silently testing nothing --
        otherwise a renamed step turns every control above into a vacuous pass."""
        with self.assertRaises(AssertionError):
            run_block(step="No Such Step")
        with self.assertRaises(AssertionError):
            run_block(text="jobs:\n  x:\n    steps:\n      - name: %s\n        run: echo hi\n" % STEP)


if __name__ == "__main__":
    unittest.main()
