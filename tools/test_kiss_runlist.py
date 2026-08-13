"""Negative controls for the does-anything-run-it gate (`kiss_runlist.py`).

The gate's claim is that a clause backed by code no CI leg compiles is a FAILURE.
A gate that only ever says CLEAN would satisfy that claim vacuously, so every
positive assertion here is paired with the defect it is supposed to reject.

Shapes checked, and the LAST GROUP matters most, because those are the gate's own
ways of passing while measuring nothing:

  * a clause-backing test present on a leg           -> CLEAN
  * a clause-backing test present on NO leg          -> FAIL   (the defect)
  * a test present on only SOME legs                 -> CLEAN, and NAMED
  * a leg that is missing / empty / unreadable /
    unnamed / given TWICE                            -> FAIL, not "0 missing"

The duplicate-leg case is the sharpest of them: `--leg a=X --leg a=Y` would
overwrite the first leg, and `on EVERY leg` would then mean "on the one leg
left" — the gate printing CLEAN having read one leg is the exact verdict it
exists to prevent, produced by its own argument handling.

Run: python tools/test_kiss_runlist.py   (also collected by pytest)
"""
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt

TOOL = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kiss_runlist.py")

# The number of controls this file must run. ASSERTED, not printed — a count
# that is printed and never compared is capability-present/enforcement-absent,
# which is the defect class this suite exists to catch. Pinning it means a
# control that stops executing FAILS here rather than quietly lowering a number
# nobody reads. Changing the suite means changing this line deliberately.
#
# THE RULE, because this looks like a contradiction of its own sibling and is not:
# `tools/test_kiss_trace_gates.py` had a hardcoded `PASS: 7` replaced by a live
# counter, while this file pins a constant instead. Both are right, and the
# discriminator is what the number is FOR:
#
#     printed counts should be COMPUTED;  asserted counts should be PINNED.
#
# A printed constant drifts silently the moment a check is added — it understates
# and nothing notices. A compared constant fails loudly until someone updates it
# on purpose, and that update is the deliberate act recording that the suite
# changed. Getting the pair backwards gives you a number that is always wrong and
# never complains.
EXPECTED_CONTROLS = 12


class Checks:
    """Per-run accumulator. Deliberately NOT module-level state.

    Module globals would survive a second call to `main()` in one process —
    which is exactly what a pytest wrapper does — inflating `passed` and carrying
    earlier failures forward. In a suite whose printed count IS the evidence that
    the gate can fail, leaked state means **a run where a control never executed
    can still print the expected number**: wrong in the direction of looking fine.
    """

    def __init__(self):
        self.failures = []
        self.passed = 0

    def __call__(self, cond, msg):
        if cond:
            self.passed += 1
            print(f"  ok   {msg}")
        else:
            print(f"  FAIL {msg}")
            self.failures.append(msg)


def fid(ordinal):
    """A fixture clause ID assembled at run time — never a source literal.

    A literal would match the clause-ID grammar, be scanned out of this file by
    the citation sweep, and be reported as a dangling citation against the real
    suite. The `6.99` band is unused by any real document.
    """
    return "-".join(["KISS", "OPS", "6.99", ordinal])


def fixture(tmp, rows, harness_src):
    """A minimal well-formed suite: spec stems + one harness file."""
    spec = os.path.join(tmp, "spec")
    conf = os.path.join(tmp, "conformance")
    os.makedirs(spec, exist_ok=True)
    os.makedirs(conf, exist_ok=True)
    body = ["## 6.0 Fixture\n\n"]
    for ordinal, test in rows:
        body.append(f"- **{fid(ordinal)}** — A fixture clause. An implementation "
                    f"MUST do the fixture thing. *Test:* `{test}`.\n")
    body.append("\n## 9. Traceability\n\n| Clause | Test |\n|---|---|\n")
    for ordinal, test in rows:
        body.append(f"| {fid(ordinal)} | `{test}` |\n")
    for s in kt.SPECS:                      # derived from kiss_trace, not restated
        with open(os.path.join(spec, s + ".md"), "w", encoding="utf-8") as f:
            f.write("".join(body) if s == "ops" else "")
    with open(os.path.join(conf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
        f.write(harness_src)
    return spec, conf


def listing(tmp, name, test_names):
    """A synthetic `cargo test -- --list` capture, in cargo's real format."""
    p = os.path.join(tmp, f"{name}.txt")
    with open(p, "w", encoding="utf-8") as f:
        for t in test_names:
            # fully-qualified, as cargo emits — the gate must match on bare names
            f.write(f"fixture::tests::{t}: test\n")
        f.write("src/lib.rs - some_doc (line 12): test\n")   # a doc-test, ignored
    return p


def run(spec, conf, legs):
    args = [sys.executable, TOOL, "--spec-dir", spec, "--conformance-dir", conf]
    for name, path in legs:
        args += ["--leg", f"{name}={path}"]
    out = subprocess.run(args, capture_output=True, text=True, timeout=300)
    return out.returncode, out.stdout + out.stderr


ROWS = [("0042", "test_ops_alpha"), ("0043", "test_ops_beta")]
HARNESS = """
#[test]
fn test_ops_alpha() { assert!(true); }

#[test]
fn test_ops_beta() { assert!(true); }
"""


def main():
    check = Checks()
    print("does-anything-run-it gate:")

    # CONTROL FIRST: a well-formed suite whose backing is compiled must pass.
    # Without this the three rejections below prove nothing — a gate that fails
    # unconditionally satisfies all of them.
    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        both = listing(tmp, "ubuntu", ["test_ops_alpha", "test_ops_beta"])
        win = listing(tmp, "windows", ["test_ops_alpha", "test_ops_beta"])
        rc, out = run(spec, conf, [("ubuntu", both), ("windows", win)])
    check(rc == 0, "control: a suite whose backing is compiled everywhere is CLEAN")
    check("2  on EVERY leg" in out or "2  on EVERY" in out,
          "both backing tests are reported as compiled on every leg")

    # THE DEFECT: a clause whose backing test no leg compiles.
    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        l1 = listing(tmp, "ubuntu", ["test_ops_alpha"])
        l2 = listing(tmp, "windows", ["test_ops_alpha"])
        rc, out = run(spec, conf, [("ubuntu", l1), ("windows", l2)])
    check(rc != 0, "a clause-backing test compiled by NO leg FAILS the gate")
    check("test_ops_beta" in out and fid("0043") in out,
          "the failure names the test AND the clause it falsely backs")

    # SINGLE-LEG is not a failure — but it must be NAMED, because it is the whole
    # evidence base for that clause. This is the §6.13-0006 shape.
    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        l1 = listing(tmp, "ubuntu", ["test_ops_alpha"])
        l2 = listing(tmp, "windows", ["test_ops_alpha", "test_ops_beta"])
        rc, out = run(spec, conf, [("ubuntu", l1), ("windows", l2)])
    check(rc == 0, "a test compiled on only ONE leg is not a failure")
    check("on SOME legs" in out and "test_ops_beta  [windows]" in out,
          "and it is named with the leg that carries it, not silently counted")

    # VACUITY, the gate's own failure modes. Both must FAIL rather than report a
    # clean zero: with no legs, or an empty capture, "0 missing" is a statement
    # about the input, not about the suite.
    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        rc, out = run(spec, conf, [])
    check(rc != 0 and "no --leg" in out,
          "NO leg supplied FAILS — it cannot report coverage it never measured")

    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        empty = os.path.join(tmp, "empty.txt")
        open(empty, "w", encoding="utf-8").close()
        rc, out = run(spec, conf, [("ubuntu", empty)])
    check(rc != 0 and "ZERO tests" in out,
          "an EMPTY capture FAILS — a broken build step is not a suite with no tests")

    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        rc, out = run(spec, conf, [("ubuntu", os.path.join(tmp, "nope.txt"))])
    check(rc != 0 and "not found" in out,
          "a MISSING capture file FAILS — a lost artifact is not a clean run")

    # ARGUMENT-HANDLING vacuity. A leg named twice would silently overwrite the
    # first, and every "on EVERY leg" verdict below would then be computed over
    # one leg. The workflow passes two near-identical adjacent lines, so a
    # copy-paste updating the path and not the name is the ordinary way in.
    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        l1 = listing(tmp, "a", ["test_ops_alpha", "test_ops_beta"])
        l2 = listing(tmp, "b", ["test_ops_alpha", "test_ops_beta"])
        rc, out = run(spec, conf, [("ubuntu", l1), ("ubuntu", l2)])
    check(rc != 0 and "given twice" in out,
          "a leg name given TWICE FAILS — it would collapse two legs into one")

    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        l1 = listing(tmp, "u", ["test_ops_alpha", "test_ops_beta"])
        rc, out = run(spec, conf, [("", l1)])
    check(rc != 0 and "empty NAME" in out,
          "an EMPTY leg name FAILS — it passes the NAME=PATH shape check")

    # An unreadable capture must be a TYPED failure, not a traceback: a stack
    # trace reads as a broken tool, when what broke is the artifact.
    with tempfile.TemporaryDirectory() as tmp:
        spec, conf = fixture(tmp, ROWS, HARNESS)
        adir = os.path.join(tmp, "a_directory")
        os.makedirs(adir, exist_ok=True)
        rc, out = run(spec, conf, [("ubuntu", adir)])
    check(rc != 0 and "unreadable" in out and "Traceback" not in out,
          "an UNREADABLE capture FAILS as a typed error, not a traceback")

    print()
    if check.failures:
        print(f"FAILED: {len(check.failures)} check(s), {check.passed} passed")
        return 1
    # The count is COMPARED, not just reported. A control that silently stopped
    # running would otherwise lower this number with nothing to notice.
    if check.passed != EXPECTED_CONTROLS:
        print(f"FAILED: expected {EXPECTED_CONTROLS} controls, ran {check.passed}. "
              f"A control that no longer executes is not a smaller suite, it is an "
              f"unproven gate.")
        return 1
    print(f"PASS: {check.passed} checks")
    return 0


def test_kiss_runlist_controls():
    """Collected by pytest; CI also runs this file in script mode.

    Safe to call repeatedly: `main()` builds its own accumulator, so a second
    invocation in the same process starts from zero.
    """
    assert main() == 0


if __name__ == "__main__":
    sys.exit(main())
