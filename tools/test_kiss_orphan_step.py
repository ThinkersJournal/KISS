"""Behavioural controls for the orphaned-citation CI STEP (#407).

Shell inside YAML is the least-tested code in this repository: nothing imports it, no
linter reads it, and its only execution is on a runner where a mistake presents as a
confusing red rather than a failure. The ratchet step (#338) and the MSRV leg (#369) are
tested by EXTRACTING their `run:` block and EXECUTING it; this does the same.

⚠️ THE FAILURE THIS STEP EXISTS TO AVOID IS ITS OWN COLLAPSE. The scanner returns THREE
states — 0 CLEAN, 1 CANDIDATES, 2 COULD NOT MEASURE — and Actions collapses every non-zero
into "failed". A step that simply propagated the exit code would present a REFUSAL as a
finding, and a step that swallowed everything would present a refusal as CLEAN. Both are
the defect the scanner was built to avoid, reintroduced by its own wiring. So each state is
asserted BY BEHAVIOUR here, not by reading the YAML.

Run: python tools/test_kiss_orphan_step.py   (also collected by pytest)
"""
import os
import subprocess
import sys
import tempfile

if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "controls would report success having verified nothing")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_workflow import run_block  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
WF = os.path.join(ROOT, ".github", "workflows", "traceability.yml")
STEP = "Orphaned prose citations (#407, advisory)"


def _block():
    with open(WF, encoding="utf-8") as fh:
        text = fh.read()
    body = run_block(text, STEP)
    # `${{ github.base_ref }}` is Actions templating, not shell. Substitute the PUSH
    # case (empty base_ref) so the block is runnable; the PR arm is exercised by
    # supplying a non-empty value in the test that needs it.
    return body


def _run(scanner_exit, base_ref="", fetch_ok=True, echo_args=False):
    """Execute the real step body with `python` and `git` stubbed as shell functions.

    Functions, not PATH stubs, so behaviour is identical on a runner and here — the
    lesson the ratchet-step controls already record.
    """
    body = _block().replace("${{ github.base_ref }}", base_ref)
    fetch = "return 0" if fetch_ok else "return 1"
    script = (
        "set -e\n"
        f"git() {{ case \"$1\" in fetch) {fetch};; *) return 0;; esac; }}\n"
        + (f'python() {{ echo "INVOKED $*"; return {scanner_exit}; }}\n' if echo_args\
           else f"python() {{ return {scanner_exit}; }}\n")
        + body
    )
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "s.sh")
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(script)
        r = subprocess.run(["bash", path], capture_output=True, text=True, timeout=300)
    return r.returncode, r.stdout + r.stderr


def test_clean_is_green_and_says_so():
    rc, out = _run(0)
    assert rc == 0, f"a CLEAN scan must not fail the step: rc={rc}\n{out}"
    assert "CLEAN" in out, out


def test_candidates_are_ADVISORY_and_do_not_fail_the_step():
    """Exit 1 is "found candidates to adjudicate", not a defect. It must warn, not red."""
    rc, out = _run(1)
    assert rc == 0, f"CANDIDATES must be advisory (rc 0), got {rc}\n{out}"
    assert "::warning::" in out, f"candidates must surface as a warning:\n{out}"
    assert "does not block" in out, out


def test_COULD_NOT_MEASURE_fails_the_step_and_is_NOT_reported_as_clean():
    """⚠️ The one that matters. A refusal must not be swallowed into green, and must not
    be worded as a finding: it is the scanner saying it could not look."""
    rc, out = _run(2)
    assert rc != 0, f"a REFUSAL must fail the step, got rc={rc}\n{out}"
    assert "COULD NOT MEASURE" in out, out
    assert "REFUSAL" in out, f"the message must name it a refusal, not a finding:\n{out}"
    assert "CLEAN" not in out, "a refusal must never print CLEAN"


def test_the_three_states_produce_three_DIFFERENT_outcomes():
    """Collapse is the defect: if any two states looked alike the step would be unable
    to report the one it exists to distinguish."""
    seen = {}
    for code in (0, 1, 2):
        rc, out = _run(code)
        marker = ("clean" if "CLEAN" in out and rc == 0 else
                  "advisory" if "::warning::" in out and rc == 0 else
                  "refusal" if rc != 0 else "?")
        seen[code] = marker
    assert len(set(seen.values())) == 3, f"the three states collapsed: {seen}"


def test_an_unexpected_exit_code_is_not_silently_green():
    rc, out = _run(7)
    assert rc != 0, f"an unknown exit must not pass silently, got {rc}\n{out}"
    assert "unexpected exit 7" in out, out


def test_a_FAILED_BASE_FETCH_is_its_own_result_and_never_reaches_the_scanner():
    """An unchecked fetch fails silently and resurfaces as an exit-2 refusal, where it is
    indistinguishable from a broken control. The step checks it instead."""
    rc, out = _run(0, base_ref="main", fetch_ok=False)
    assert rc != 0, f"a failed base fetch must fail the step, got {rc}\n{out}"
    assert "could not fetch base" in out, out
    assert "NOT a clean scan" in out, out


def test_the_PR_arm_passes_the_FETCHED_BASE_and_the_push_arm_passes_HEAD_parent():
    """⚠️ Asserted on what EXECUTES, not on the source text. A first version checked that
    `HEAD^` did not appear after the PR branch in the YAML -- both WRONG (the push arm
    legitimately contains it) and a source-text assertion, the very thing this file exists
    to avoid. The stub now echoes the argument the scanner actually receives."""
    pr_rc, pr_out = _run(0, base_ref="main", fetch_ok=True, echo_args=True)
    push_rc, push_out = _run(0, base_ref="", echo_args=True)
    assert pr_rc == 0 and push_rc == 0, (pr_out, push_out)
    assert "--base-ref origin/main" in pr_out, f"the PR arm must use the fetched base:\n{pr_out}"
    assert "--base-ref HEAD^" in push_out, f"the push arm must use HEAD^:\n{push_out}"


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: CLEAN green, CANDIDATES advisory, COULD NOT "
          f"MEASURE red and named a refusal, a failed base fetch its own result")
    return 0


if __name__ == "__main__":
    sys.exit(main())
